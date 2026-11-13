//! Upstream `config/Config.java` and `config/ConfigManager.java`, 1:1,
//! plus the fabric loader shim `config/fabric/ConfigManagerImpl.java`
//! (the `@ExpectPlatform getConfigPath` seam). The neoforge shim is not
//! ported (Horizon is the loader surface). Gson serialization is realized
//! through serde_json with the LOWER_CASE_WITH_UNDERSCORES naming policy
//! and Gson's pretty printing; missing JSON fields keep the constructor
//! defaults, like Gson's reflective fill.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::Value;

use crate::catmull_rom::{self, CatmullRomSpline};
use crate::state::{Gson, GSON, MOD_ID};
use crate::vec2::Vec2;

/// `public class Config`.
#[derive(Debug, Clone)]
pub struct Config {
    /// `private boolean toggleMod = true`.
    toggle_mod: bool,
    /// `private int animationSpeed = 100`.
    animation_speed: i32,
    /// `private float[][] curvePoints = new float[][]{}`.
    curve_points: Vec<[f32; 2]>,
}

impl Default for Config {
    fn default() -> Self {
        Self { toggle_mod: true, animation_speed: 100, curve_points: Vec::new() }
    }
}

impl Config {
    /// `getAnimationSpeed()`.
    pub fn get_animation_speed(&self) -> i32 {
        self.animation_speed
    }

    /// `setAnimationSpeed(int)`.
    pub fn set_animation_speed(&mut self, animation_speed: i32) {
        self.animation_speed = animation_speed;
    }

    /// `getAnimationSpeedFormatted()` (`animationSpeed / 100F`).
    pub fn get_animation_speed_formatted(&self) -> f32 {
        self.animation_speed as f32 / 100.0
    }

    /// `getSplines()`.
    pub fn get_splines(&self) -> Vec<CatmullRomSpline> {
        catmull_rom::splines_from_points(&self.get_curve_points())
    }

    /// `setCurvePoints(List<Vec2> points)`. Java builds
    /// `new float[points.size() - 4][2]` from the inner points.
    pub fn set_curve_points(&mut self, points: &[Vec2]) {
        let mut curve_points = vec![[0.0f32; 2]; points.len() - 4];

        for (i, p) in points.iter().enumerate().take(points.len() - 2).skip(2) {
            curve_points[i - 2][0] = p.v[0] as f32;
            curve_points[i - 2][1] = p.v[1] as f32;
        }
        self.curve_points = curve_points;
    }

    /// `getCurvePoints()`: the stored floats bracketed by (0,0)x2 and
    /// (1,1)x2.
    pub fn get_curve_points(&self) -> Vec<Vec2> {
        let mut points = Vec::new();

        points.push(Vec2::with_xy(0.0, 0.0));
        points.push(Vec2::with_xy(0.0, 0.0));

        for curve_point in &self.curve_points {
            points.push(Vec2::with_xy(curve_point[0] as f64, curve_point[1] as f64));
        }

        points.push(Vec2::with_xy(1.0, 1.0));
        points.push(Vec2::with_xy(1.0, 1.0));

        points
    }

    /// `getToggleMod()`.
    pub fn get_toggle_mod(&self) -> bool {
        self.toggle_mod
    }

    /// `setToggleMod(Boolean value)`.
    pub fn set_toggle_mod(&mut self, value: bool) {
        self.toggle_mod = value;
    }

    /// Gson `toJson` shape: field order follows the Java declaration
    /// (toggleMod, animationSpeed, curvePoints) under the snake_case
    /// naming policy.
    fn to_json_value(&self) -> Value {
        serde_json::json!({
            "toggle_mod": self.toggle_mod,
            "animation_speed": self.animation_speed,
            "curve_points": self.curve_points,
        })
    }

    /// Gson `fromJson` fill: absent fields keep the constructor defaults.
    fn from_json_value(value: &Value) -> Option<Config> {
        let mut config = Config::default();
        let obj = value.as_object()?;
        if let Some(v) = obj.get("toggle_mod").and_then(Value::as_bool) {
            config.toggle_mod = v;
        }
        if let Some(v) = obj.get("animation_speed").and_then(Value::as_i64) {
            config.animation_speed = v as i32;
        }
        if let Some(v) = obj.get("curve_points") {
            config.curve_points = v
                .as_array()
                .map(|rows| {
                    rows.iter()
                        .map(|row| {
                            let row = row.as_array().cloned().unwrap_or_default();
                            [
                                row.first().and_then(Value::as_f64).unwrap_or(0.0) as f32,
                                row.get(1).and_then(Value::as_f64).unwrap_or(0.0) as f32,
                            ]
                        })
                        .collect()
                })
                .unwrap_or_default();
        }
        Some(config)
    }
}

impl Gson {
    /// `SmoothSwapping.GSON.toJson(config)` (pretty printed).
    pub fn to_json(&self, config: &Config) -> String {
        serde_json::to_string_pretty(&config.to_json_value()).expect("config serialization")
    }

    /// `SmoothSwapping.GSON.fromJson(reader, Config.class)`: `null`
    /// (None) for blank input; a malformed body throws upstream and
    /// propagates here as a panic, kept 1:1.
    pub fn from_json(&self, json: &str) -> Option<Config> {
        if json.trim().is_empty() {
            return None;
        }
        let value: Value = serde_json::from_str(json).expect("config deserialization");
        Config::from_json_value(&value)
    }
}

/// Live handle over the `ConfigManager` static, standing in for Java's
/// shared `INSTANCE` reference returned by `getConfig()`.
pub struct ConfigRef(std::sync::MutexGuard<'static, Option<Config>>);

impl std::ops::Deref for ConfigRef {
    type Target = Config;
    fn deref(&self) -> &Config {
        self.0.as_ref().expect("config")
    }
}

impl std::ops::DerefMut for ConfigRef {
    fn deref_mut(&mut self) -> &mut Config {
        self.0.as_mut().expect("config")
    }
}

/// `public class ConfigManager`.
pub struct ConfigManager;

static FILE: Mutex<Option<PathBuf>> = Mutex::new(None);
static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

/// Test/launcher seam for `getConfigPath()`: the fabric shim returns
/// FabricLoader's config dir; the Horizon launcher integration sets the
/// real directory through this override, tests pin a temp dir.
static CONFIG_DIR_SEAM: Mutex<Option<PathBuf>> = Mutex::new(None);

/// `@ExpectPlatform public static Path getConfigPath()`.
pub fn get_config_path() -> PathBuf {
    if let Some(dir) = CONFIG_DIR_SEAM.lock().unwrap().clone() {
        return dir;
    }
    // fabric/ConfigManagerImpl: FabricLoader.getInstance().getConfigDir().
    PathBuf::from("config")
}

impl ConfigManager {
    /// `private static void prepareConfigFile()`.
    fn prepare_config_file() {
        let mut file = FILE.lock().unwrap();
        if file.is_some() {
            return;
        }
        *file = Some(get_config_path().join(format!("{}.json", MOD_ID)));
    }

    /// `public static Config initializeConfig()`.
    pub fn initialize_config() -> Config {
        let mut config = CONFIG.lock().unwrap();
        if let Some(existing) = config.as_ref() {
            return existing.clone();
        }

        let fresh = Config::default();
        *config = Some(fresh.clone());
        drop(config);
        Self::load();

        CONFIG.lock().unwrap().as_ref().expect("config after load").clone()
    }

    /// `private static void load()`.
    fn load() {
        Self::prepare_config_file();

        let path = FILE.lock().unwrap().clone().expect("config file path");
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                // catch (FileNotFoundException) reverts to defaults; an
                // existing-but-unreadable file is not that case upstream.
                if let Some(parsed) = GSON.from_json(&content) {
                    *CONFIG.lock().unwrap() = Some(parsed);
                }
            }
            Err(_) => {
                if !path.exists() {
                    Self::save();
                    // A fresh save is then read back through the same
                    // branch as upstream's second file.exists() check.
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Some(parsed) = GSON.from_json(&content) {
                            *CONFIG.lock().unwrap() = Some(parsed);
                        }
                    }
                }
            }
        }
    }

    /// `public static void save()`.
    pub fn save() {
        Self::prepare_config_file();
        let path = FILE.lock().unwrap().clone().expect("config file path");
        let json_string = GSON.to_json(&CONFIG.lock().unwrap().as_ref().expect("config").clone());

        if let Err(_e) = std::fs::write(&path, json_string) {
            // catch (IOException): System.err.println(...)
            eprintln!("Couldn't save Smooth Swapping configuration file");
        }
    }

    /// `public static Config getConfig()`.
    ///
    /// Java returns the shared INSTANCE, so writes through it are visible to
    /// `save()`; the guard keeps that reference semantics.
    pub fn get_config() -> ConfigRef {
        let mut config = CONFIG.lock().unwrap();
        if config.is_none() {
            *config = Some(Config::default());
        }
        ConfigRef(config)
    }

    #[cfg(test)]
    pub fn reset_for_tests() {
        *CONFIG.lock().unwrap() = None;
        *FILE.lock().unwrap() = None;
        *CONFIG_DIR_SEAM.lock().unwrap() = None;
    }

    #[cfg(test)]
    pub fn set_config_dir_for_tests(dir: PathBuf) {
        *CONFIG_DIR_SEAM.lock().unwrap() = Some(dir);
    }
}

#[cfg(test)]
impl Config {
    pub fn curve_points_len_for_tests(&self) -> usize {
        self.curve_points.len()
    }
}

#[cfg(test)]
impl ConfigManager {
    /// Drops the in-memory config so the next `initialize_config` re-reads
    /// the saved file from the pinned dir.
    pub fn reload_for_tests() {
        *CONFIG.lock().unwrap() = None;
        FILE.lock().unwrap().take();
    }
}
