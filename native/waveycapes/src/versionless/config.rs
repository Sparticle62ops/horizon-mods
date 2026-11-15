//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.config`
//! (`Config.java`, `ConfigUpgrader.java`).

#![allow(non_snake_case, dead_code)]

use crate::versionless::sim::{CapeMovement, CapeStyle, WindMode};

/// `dev.tr7zw.waveycapes.versionless.config.Config`
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Java `public int configVersion = 2`
    pub configVersion: i32,
    /// Java `public WindMode windMode = WindMode.NONE`
    pub windMode: WindMode,
    /// Java `public CapeStyle capeStyle = CapeStyle.SMOOTH`
    pub capeStyle: CapeStyle,
    /// Java `public CapeMovement capeMovement = CapeMovement.BASIC_SIMULATION_3D`
    pub capeMovement: CapeMovement,
    // Java commented out: public int capeParts = 16;
    /// Java `public int gravity = 25`
    pub gravity: i32,
    /// Java `public int heightMultiplier = 6`
    pub heightMultiplier: i32,
    /// Java `public int straveMultiplier = 2`
    pub straveMultiplier: i32,
    /// Java `public boolean computeGravityVector = false`
    pub computeGravityVector: bool,
    // Java commented out: public int maxBend = 5;
}

impl Default for Config {
    /// Java field initializers (the implicit no-args constructor).
    fn default() -> Self {
        Config {
            configVersion: 2,
            windMode: WindMode::NONE,
            capeStyle: CapeStyle::SMOOTH,
            capeMovement: CapeMovement::BASIC_SIMULATION_3D,
            gravity: 25,
            heightMultiplier: 6,
            straveMultiplier: 2,
            computeGravityVector: false,
        }
    }
}

impl Config {
    /// The JSON shape Gson's `setPrettyPrinting()` emits (field order kept).
    pub fn to_json(&self) -> String {
        format!(
            "{{\n  \"configVersion\": {},\n  \"windMode\": \"{}\",\n  \"capeStyle\": \"{}\",\n  \"capeMovement\": \"{}\",\n  \"gravity\": {},\n  \"heightMultiplier\": {},\n  \"straveMultiplier\": {},\n  \"computeGravityVector\": {}\n}}",
            self.configVersion,
            self.windMode_name(),
            self.capeStyle_name(),
            self.capeMovement_name(),
            self.gravity,
            self.heightMultiplier,
            self.straveMultiplier,
            self.computeGravityVector
        )
    }

    fn windMode_name(&self) -> &'static str {
        match self.windMode {
            WindMode::NONE => "NONE",
            WindMode::WAVES => "WAVES",
        }
    }

    fn capeStyle_name(&self) -> &'static str {
        match self.capeStyle {
            CapeStyle::BLOCKY => "BLOCKY",
            CapeStyle::SMOOTH => "SMOOTH",
        }
    }

    fn capeMovement_name(&self) -> &'static str {
        match self.capeMovement {
            CapeMovement::VANILLA => "VANILLA",
            CapeMovement::BASIC_SIMULATION => "BASIC_SIMULATION",
            CapeMovement::BASIC_SIMULATION_3D => "BASIC_SIMULATION_3D",
            CapeMovement::DUNGEONS => "DUNGEONS",
        }
    }

    /// Gson-style parse: the default constructor runs first, then present
    /// fields overwrite; an unparseable value throws (Java), which the caller
    /// turns into `None` (fresh config).
    pub fn from_json(text: &str) -> Option<Config> {
        let mut config = Config::default();
        for line in text.lines() {
            let line = line.trim().trim_end_matches(',');
            let Some((key, value)) = line.split_once(": ") else {
                continue;
            };
            let key = key.trim().trim_matches('"');
            let value = value.trim().trim_matches('"');
            match key {
                "configVersion" => config.configVersion = value.parse::<i32>().ok()?,
                "windMode" => config.windMode = match value {
                    "NONE" => WindMode::NONE,
                    "WAVES" => WindMode::WAVES,
                    _ => return None,
                },
                "capeStyle" => config.capeStyle = match value {
                    "BLOCKY" => CapeStyle::BLOCKY,
                    "SMOOTH" => CapeStyle::SMOOTH,
                    _ => return None,
                },
                "capeMovement" => config.capeMovement = match value {
                    "VANILLA" => CapeMovement::VANILLA,
                    "BASIC_SIMULATION" => CapeMovement::BASIC_SIMULATION,
                    "BASIC_SIMULATION_3D" => CapeMovement::BASIC_SIMULATION_3D,
                    "DUNGEONS" => CapeMovement::DUNGEONS,
                    _ => return None,
                },
                "gravity" => config.gravity = value.parse::<i32>().ok()?,
                "heightMultiplier" => config.heightMultiplier = value.parse::<i32>().ok()?,
                "straveMultiplier" => config.straveMultiplier = value.parse::<i32>().ok()?,
                "computeGravityVector" => config.computeGravityVector = value.parse::<bool>().ok()?,
                _ => {}
            }
        }
        Some(config)
    }
}

/// `dev.tr7zw.waveycapes.versionless.config.ConfigUpgrader`
pub struct ConfigUpgrader;

impl ConfigUpgrader {
    /// `public static boolean upgradeConfig(Config config)`
    pub fn upgradeConfig(config: &mut Config) -> bool {
        let mut changed = false;

        // check for more changes here
        if config.configVersion == 1 {
            config.configVersion = 2;
            if config.gravity < 0 {
                config.gravity *= -1; // fixed gravity
            }
        }

        changed
    }
}
