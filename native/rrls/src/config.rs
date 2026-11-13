//! Config: which overlay behaviors rrls applies to reloads.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HideType {
    /// Keep the vanilla overlay while packs load in the background.
    Vanilla,
    /// Render the overlay but with animated RGB text.
    RgbText,
    /// Hide the overlay entirely while packs load.
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub enabled: bool,
    pub hide_type: HideType,
    /// Force-close the loading screen when the reload finishes, even if the
    /// client is mid-frame.
    pub force_close: bool,
    /// When true, blocked-input protection is skipped (the reload never
    /// freezes the UI).
    pub skip_vanilla_overlay: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            hide_type: HideType::Vanilla,
            force_close: false,
            skip_vanilla_overlay: false,
        }
    }
}

impl Config {
    /// Parse a flat key=value config (the mod's own file format); unknown
    /// keys are ignored, malformed values keep the current value per key.
    pub fn parse(text: &str) -> Self {
        let mut cfg = Config::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let value = value.trim();
            let flag = parse_flag(value);
            match key.trim() {
                "enable" | "enabled" => {
                    if let Some(f) = flag {
                        cfg.enabled = f;
                    }
                }
                "hide_type" | "hideType" => {
                    cfg.hide_type = match value {
                        "hidden" => HideType::Hidden,
                        "rgb" | "rgb_text" => HideType::RgbText,
                        _ => HideType::Vanilla,
                    }
                }
                "force_close" | "forceClose" => {
                    if let Some(f) = flag {
                        cfg.force_close = f;
                    }
                }
                "skip_vanilla_overlay" | "skipVanillaOverlay" => {
                    if let Some(f) = flag {
                        cfg.skip_vanilla_overlay = f;
                    }
                }
                _ => {}
            }
        }
        cfg
    }
}

/// A recognized boolean; anything else keeps the existing value.
fn parse_flag(value: &str) -> Option<bool> {
    match value {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_upstream_shape() {
        let cfg = Config::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.hide_type, HideType::Vanilla);
        assert!(!cfg.force_close);
    }

    #[test]
    fn parse_all_keys() {
        let cfg = Config::parse("enable=true\nhide_type=hidden\nforce_close=true\nskip_vanilla_overlay=true\n");
        assert!(cfg.enabled);
        assert_eq!(cfg.hide_type, HideType::Hidden);
        assert!(cfg.force_close);
        assert!(cfg.skip_vanilla_overlay);
    }

    #[test]
    fn unknown_and_malformed_lines_fall_back() {
        let cfg = Config::parse("enable=banana\nhide_type=weird\nfoo=bar\n# comment\n\n");
        assert!(cfg.enabled, "malformed bool keeps default");
        assert_eq!(cfg.hide_type, HideType::Vanilla);
    }

    #[test]
    fn disable_is_honored() {
        let cfg = Config::parse("enable=false\n");
        assert!(!cfg.enabled);
    }

    #[test]
    fn rgb_text_variant_parses() {
        let cfg = Config::parse("hide_type=rgb\n");
        assert_eq!(cfg.hide_type, HideType::RgbText);
    }
}
