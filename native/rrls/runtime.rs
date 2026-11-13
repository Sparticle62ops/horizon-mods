//! rrls runtime wiring for Horizon.
//!
//! Carrier seam: Horizon's client loads resource packs synchronously and owns
//! its own overlay, so the background load itself is host-side. This runtime
//! exposes the tracker: begin/advance/dismiss on reload cycles and the
//! overlay policy the client consults. The seam is tracked in the README.

use crate::config::Config;
use crate::reload::ReloadTracker;
use std::sync::Mutex;

pub struct RrlsState {
    tracker: Mutex<ReloadTracker>,
}

impl RrlsState {
    pub fn from_config_text(text: &str) -> Self {
        Self { tracker: Mutex::new(ReloadTracker::new(Config::parse(text))) }
    }

    pub fn begin_reload(&self, total: u32) -> u64 {
        self.tracker.lock().expect("rrls tracker").begin(total)
    }

    pub fn advance_reload(&self) {
        self.tracker.lock().expect("rrls tracker").advance();
    }

    pub fn overlay_visible(&self) -> bool {
        self.tracker.lock().expect("rrls tracker").should_render_overlay()
    }

    pub fn rgb_step(&self) -> Option<u32> {
        self.tracker.lock().expect("rrls tracker").overlay_rgb_step()
    }

    pub fn force_close(&self) -> bool {
        self.tracker.lock().expect("rrls tracker").may_force_close()
    }

    pub fn dismiss(&self) {
        self.tracker.lock().expect("rrls tracker").dismiss();
    }
}
