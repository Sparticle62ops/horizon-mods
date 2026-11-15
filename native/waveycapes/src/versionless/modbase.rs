//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.ModBase`.

#![allow(non_snake_case, dead_code)]

use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::versionless::config::{Config, ConfigUpgrader};
use crate::versionless::nms::MinecraftPlayer;
use crate::versionless::sim::{CapeMovement, CapeStyle, WindMode};
use crate::versionless::util::Vector3;

/// Java `public static final Logger LOGGER = LogManager.getLogger("WaveyCapes")`
pub struct Logger;

impl Logger {
    /// `LOGGER.info(String)`
    pub fn info(&self, message: &str) {
        println!("[WaveyCapes] {}", message);
    }

    /// `LOGGER.warn(String)` / `LOGGER.error(String)`
    pub fn error(&self, message: &str) {
        eprintln!("[WaveyCapes] {}", message);
    }
}

/// Java `public static final Logger LOGGER`
pub static LOGGER: Logger = Logger;

/// Java `public static Config config` (null until `init()` runs; the lock
/// models Java's static field).
pub static CONFIG: OnceLock<Mutex<Option<Config>>> = OnceLock::new();

/// Java `@Getter public static ModBase INSTANCE`
pub static INSTANCE: OnceLock<Mutex<Option<Box<dyn ModBase + Send>>>> = OnceLock::new();

pub fn config_cell() -> &'static Mutex<Option<Config>> {
    CONFIG.get_or_init(|| Mutex::new(None))
}

fn instance_cell() -> &'static Mutex<Option<Box<dyn ModBase + Send>>> {
    INSTANCE.get_or_init(|| Mutex::new(None))
}

/// `dev.tr7zw.waveycapes.versionless.ModBase`
///
/// The abstract class becomes a trait; the Java statics stay module statics.
pub trait ModBase: Send {
    /// `public abstract void initSupportHooks()`
    fn initSupportHooks(&mut self);

    /// `public abstract Vector3 applyModAnimations(MinecraftPlayer player, Vector3 pos)`
    fn applyModAnimations(&mut self, player: &dyn MinecraftPlayer, pos: Vector3) -> Vector3;

    /// `public void init()`. Java sets `INSTANCE = this`; Rust takes the
    /// instance by value.
    fn init(&mut self) {
        // Java: INSTANCE = this;
        // (the concrete mod installs itself via `ModBase::install_instance`)
    }

    /// `public void writeConfig()`
    fn writeConfig(&self) {
        let settings_file = settings_file();
        if settings_file.exists() {
            let _ = fs::remove_file(&settings_file);
        }
        if let Some(config) = config_cell().lock().unwrap().as_ref() {
            if let Err(e1) = fs::write(&settings_file, config.to_json()) {
                e1.print_stack_trace_shim();
            }
        }
    }
}

trait PrintStackTraceShim {
    fn print_stack_trace_shim(&self);
}

impl PrintStackTraceShim for std::io::Error {
    /// Java `e1.printStackTrace()`
    fn print_stack_trace_shim(&self) {
        eprintln!("{}", self);
    }
}

fn settings_file() -> std::path::PathBuf {
    // Java `private final File settingsFile = new File("config", "waveycapes.json")`
    Path::new("config").join("waveycapes.json")
}

/// Java `INSTANCE = this` + the rest of the default `init()` body.
pub fn install_instance_and_init(instance: Box<dyn ModBase + Send>) {
    *instance_cell().lock().unwrap() = Some(instance);
    let settings_file = settings_file();
    let mut loaded: Option<Config> = None;
    if settings_file.exists() {
        match fs::read_to_string(&settings_file) {
            Ok(text) => match Config::from_json(&text) {
                Some(config) => loaded = Some(config),
                None => {
                    // Java catch: error while loading config, keep config null
                    println!("Error while loading config! Creating a new one!");
                }
            },
            Err(ex) => {
                println!("Error while loading config! Creating a new one!");
                ex.print_stack_trace_shim();
            }
        }
    }
    if loaded.is_none() {
        // Java: config = new Config(); writeConfig();
        loaded = Some(Config::default());
        *config_cell().lock().unwrap() = loaded;
        write_config_static();
    } else {
        *config_cell().lock().unwrap() = loaded;
        if ConfigUpgrader::upgradeConfig(config_cell().lock().unwrap().as_mut().unwrap()) {
            write_config_static();
        }
    }
}

/// `writeConfig()` without an instance receiver (Java's static field access).
pub fn write_config_static() {
    let settings_file = settings_file();
    if settings_file.exists() {
        let _ = fs::remove_file(&settings_file);
    }
    if let Some(config) = config_cell().lock().unwrap().as_ref() {
        if let Err(e1) = fs::write(&settings_file, config.to_json()) {
            e1.print_stack_trace_shim();
        }
    }
}

impl Config {
    /// `ModBase.config.computeGravityVector`
    pub fn computeGravityVector() -> bool {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().computeGravityVector, |c| c.computeGravityVector)
    }

    /// Read of the Java static `ModBase.config` field (null before init reads
    /// as the default config on the Rust side; Java would NPE).
    pub fn heightMultiplier() -> i32 {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().heightMultiplier, |c| c.heightMultiplier)
    }

    pub fn straveMultiplier() -> i32 {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().straveMultiplier, |c| c.straveMultiplier)
    }

    pub fn gravity() -> i32 {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().gravity, |c| c.gravity)
    }

}

/// Read of the Java static `ModBase.config.computeGravityVector`.
pub fn computeGravityVector() -> bool {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().computeGravityVector, |c| c.computeGravityVector)
}

/// Read of the Java static `ModBase.config.capeStyle`.
pub fn capeStyle() -> CapeStyle {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().capeStyle, |c| c.capeStyle)
}

/// Read of the Java static `ModBase.config.windMode`.
pub fn windMode() -> WindMode {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().windMode, |c| c.windMode)
}

/// Read of the Java static `ModBase.config.capeMovement`.
pub fn config_capeMovement() -> CapeMovement {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().capeMovement, |c| c.capeMovement)
}

/// `ModBase.getINSTANCE().applyModAnimations(player, change)`; on the Rust side
/// the instance lives in the static.
pub fn apply_instance_animations(player: &dyn MinecraftPlayer, pos: Vector3) -> Vector3 {
    let mut guard = instance_cell().lock().unwrap();
    match guard.as_mut() {
        Some(instance) => instance.applyModAnimations(player, pos),
        None => pos,
    }
}

/// Read of the Java static `ModBase.INSTANCE` presence.
pub fn has_instance() -> bool {
    instance_cell().lock().unwrap().is_some()
}

/// Java `protected static boolean doesClassExist(String name)`.
///
/// Java detects optional support mods by class name at runtime; the Rust port
/// wires support integrations at compile time, so this reports false (no
/// runtime classpath to probe).
pub fn doesClassExist(_name: &str) -> bool {
    false
}

// ---------------------------------------------------------------------------
// ModLoaderUtil seam
// ---------------------------------------------------------------------------

/// `Screen` seam for `ModLoaderUtil.registerConfigScreen(Supplier<Screen>)`.
pub type ScreenSeam = u64;

/// `dev.tr7zw.transition.loader.ModLoaderUtil` seam: the loader-side calls the
/// mod entry makes; the carrier implements the two hooks.
pub mod ModLoaderUtil_seam {
    use super::ScreenSeam;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    /// `disableDisplayTest()`
    pub fn disableDisplayTest() {
        // carrier hook (fabric's displayTest off)
    }

    static CONFIG_SCREEN: OnceLock<Mutex<Option<fn(ScreenSeam) -> ScreenSeam>>> = OnceLock::new();

    /// `registerConfigScreen(Supplier<Screen> supplier)`
    pub fn registerConfigScreen(supplier: fn(ScreenSeam) -> ScreenSeam) {
        *CONFIG_SCREEN.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(supplier);
    }
}
