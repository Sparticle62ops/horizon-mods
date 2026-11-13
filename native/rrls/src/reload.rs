//! Reload tracking: which reloads are active, whether the overlay should
//! render at all, and whether a finished reload may force-close the screen.

use crate::config::{Config, HideType};

/// One background reload's lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Reload started, packs are loading in the background.
    Loading,
    /// All packs applied; the screen is ready to dismiss.
    Finished,
    /// The reload was superseded by a newer one.
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reload {
    pub id: u64,
    pub phase: Phase,
    /// Total resource reloads this cycle expects, and how many finished.
    pub total: u32,
    pub done: u32,
}

impl Reload {
    pub fn new(id: u64, total: u32) -> Self {
        let mut r = Self { id, phase: Phase::Loading, total, done: 0 };
        // A no-op cycle (total 0) finishes immediately.
        if r.total == 0 {
            r.phase = Phase::Finished;
        }
        r
    }

    /// Advance the pack count; phase flips to Finished once every expected
    /// reload has landed (total 0 finishes immediately, matching upstream
    /// letting a no-op reload dismiss right away).
    pub fn advance(&mut self) {
        if self.phase != Phase::Loading {
            return;
        }
        self.done = self.done.saturating_add(1);
        if self.done >= self.total {
            self.phase = Phase::Finished;
        }
    }
}

/// Session-level tracker: the current reload and the config it runs under.
#[derive(Debug, Clone)]
pub struct ReloadTracker {
    pub config: Config,
    current: Option<Reload>,
    next_id: u64,
    /// Whether the game is still mid-startup (upstream keeps the overlay up
    /// for the initial load even in hidden mode until the first tick runs).
    startup: bool,
}

impl ReloadTracker {
    pub fn new(config: Config) -> Self {
        Self { config, current: None, next_id: 0, startup: true }
    }

    /// Start a reload cycle. A newer one supersedes the current reload.
    pub fn begin(&mut self, total: u32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        if let Some(cur) = self.current.as_mut() {
            if cur.phase == Phase::Loading {
                cur.phase = Phase::Superseded;
            }
        }
        self.current = Some(Reload::new(id, total));
        id
    }

    /// One of the cycle's resource reloads finished.
    pub fn advance(&mut self) {
        if let Some(cur) = self.current.as_mut() {
            cur.advance();
        }
    }

    /// Startup is over once the first reload cycle completes.
    pub fn tick_startup(&mut self) {
        if self.startup
            && self.current.as_ref().is_some_and(|r| r.phase == Phase::Finished)
        {
            self.startup = false;
        }
    }

    /// Should the vanilla loading overlay render right now? Hidden mode
    /// suppresses it once startup is past; RgbText and Vanilla keep it up
    /// while a reload is live, nothing otherwise.
    pub fn should_render_overlay(&self) -> bool {
        if !self.config.enabled {
            return true;
        }
        match self.config.hide_type {
            HideType::Hidden => self.startup && self.live_reload().is_some(),
            _ => self.live_reload().is_some(),
        }
    }

    /// The overlay text tint (None = default color, Some = RGB cycle step).
    pub fn overlay_rgb_step(&self) -> Option<u32> {
        if !self.config.enabled || self.config.hide_type != HideType::RgbText {
            return None;
        }
        self.live_reload().map(|r| r.done % 6)
    }

    /// May the finished reload force-close its own screen? Only when the
    /// config asks for it and the reload is done and still current.
    pub fn may_force_close(&self) -> bool {
        self.config.enabled
            && self.config.force_close
            && self.current.as_ref().is_some_and(|r| r.phase == Phase::Finished)
    }

    /// Dismiss the finished reload. Startup ends here, once the first cycle
    /// completes, because the overlay that carried it goes away with it.
    pub fn dismiss(&mut self) {
        if self.current.as_ref().is_some_and(|r| r.phase == Phase::Finished) {
            self.startup = false;
        }
        self.current = None;
    }

    /// The reload whose loading phase is live (overlay-bearing).
    fn live_reload(&self) -> Option<Reload> {
        self.current.filter(|r| r.phase == Phase::Loading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HideType;

    fn cfg(hide: HideType, force_close: bool, enabled: bool) -> Config {
        Config { enabled, hide_type: hide, force_close, skip_vanilla_overlay: false }
    }

    #[test]
    fn reload_lifecycle_finishes_on_total() {
        let mut t = ReloadTracker::new(cfg(HideType::Vanilla, false, true));
        t.begin(3);
        assert!(t.should_render_overlay(), "vanilla keeps overlay while loading");
        t.advance();
        t.advance();
        assert!(t.should_render_overlay());
        t.advance();
        assert!(!t.should_render_overlay(), "finished reload drops the overlay");
        assert!(!t.may_force_close(), "force_close off");
    }

    #[test]
    fn hidden_suppresses_after_startup() {
        let mut t = ReloadTracker::new(cfg(HideType::Hidden, false, true));
        t.begin(1);
        assert!(t.should_render_overlay(), "startup keeps the first overlay");
        t.advance();
        t.dismiss();
        assert!(!t.startup, "first finished cycle ends startup");
        t.begin(2);
        assert!(!t.should_render_overlay(), "hidden suppresses once startup is past");
        t.advance();
        t.advance();
        assert!(!t.should_render_overlay());
        t.dismiss();
    }

    #[test]
    fn newer_reload_supersedes() {
        let mut t = ReloadTracker::new(cfg(HideType::Vanilla, false, true));
        t.begin(2);
        t.advance();
        t.begin(1); // supersedes the old cycle; its progress is ignored
        t.advance();
        assert!(!t.should_render_overlay(), "the new cycle governs, not the superseded one");
        t.dismiss();
        assert!(!t.should_render_overlay());
    }

    #[test]
    fn rgb_step_cycles_by_finished_count() {
        let mut t = ReloadTracker::new(cfg(HideType::RgbText, false, true));
        assert_eq!(t.overlay_rgb_step(), None, "no live reload, no tint");
        t.begin(2);
        assert_eq!(t.overlay_rgb_step(), Some(0));
        t.advance();
        assert_eq!(t.overlay_rgb_step(), Some(1));
    }

    #[test]
    fn rgb_tint_absent_in_other_modes() {
        let mut t = ReloadTracker::new(cfg(HideType::Vanilla, false, true));
        t.begin(1);
        assert_eq!(t.overlay_rgb_step(), None);
    }

    #[test]
    fn force_close_gated_on_config_and_completion() {
        let mut t = ReloadTracker::new(cfg(HideType::Vanilla, true, true));
        t.begin(1);
        assert!(!t.may_force_close(), "still loading");
        t.advance();
        assert!(t.may_force_close());
        t.dismiss();
        assert!(!t.may_force_close(), "dismissed reload can't close");
    }

    #[test]
    fn disabled_config_always_renders_vanilla() {
        let mut t = ReloadTracker::new(cfg(HideType::Hidden, false, false));
        t.begin(1);
        assert!(t.should_render_overlay());
        t.advance();
        assert!(t.should_render_overlay());
    }

    #[test]
    fn zero_total_finishes_immediately() {
        let mut t = ReloadTracker::new(cfg(HideType::Vanilla, false, true));
        t.begin(0);
        assert!(!t.should_render_overlay(), "no-op reload drops the overlay");
    }

    #[test]
    fn advance_after_finish_is_noop() {
        let mut t = ReloadTracker::new(cfg(HideType::RgbText, false, true));
        t.begin(1);
        t.advance();
        t.advance();
        assert_eq!(t.overlay_rgb_step(), None, "finished reload drops the overlay, tint with it");
    }
}
