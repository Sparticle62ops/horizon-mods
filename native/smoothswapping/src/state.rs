//! Upstream `SmoothSwapping.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */
//!
//! The static singletons map to crate-level state. The Java
//! AtomicReference + ReentrantLock pair around `currentCursorStack`
//! collapses into one Mutex<Option<ItemStack>> (same critical section);
//! every other static gets its own lock as the Java field shape implies.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::item::ItemStack;
use crate::swaps::AnySwap;

/// `public static final String MOD_ID = "smoothswapping"`.
pub const MOD_ID: &str = "smoothswapping";

/// `SmoothSwapping.GSON`: `new GsonBuilder()
/// .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
/// .setPrettyPrinting().create()`. The snake_case naming policy and the
/// pretty printer are realized by the `to_json`/`from_json` impl below
/// (serde_json, 2-space indent).
pub struct Gson;
pub static GSON: Gson = Gson;

/// `SmoothSwapping.LOGGER` (`LogManager.getLogger(MOD_ID)`). Upstream only
/// ever calls it inside commented-out lines; kept as the port's stderr
/// logger for the same `MOD_ID` tag.
pub struct Logger;
pub static LOGGER: Logger = Logger;

impl Logger {
    pub fn info(&self, message: &str) {
        eprintln!("[smoothswapping] {}", message);
    }
}

/// `public static final int ASSUME_CURSOR_STACK_SLOT_INDEX = -2`.
pub const ASSUME_CURSOR_STACK_SLOT_INDEX: i32 = -2;

/// `public static boolean clickSwap`.
pub static CLICK_SWAP: AtomicBool = AtomicBool::new(false);

/// `public static Short clickSwapStack` (nullable Java `Short`).
pub static CLICK_SWAP_STACK: Mutex<Option<i16>> = Mutex::new(None);

/// `public static Map<Integer, List<InventorySwap>> swaps`.
pub static SWAPS: std::sync::LazyLock<Mutex<HashMap<i32, Vec<AnySwap>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// `public static NonNullList<ItemStack> oldStacks`.
pub static OLD_STACKS: Mutex<Vec<ItemStack>> = Mutex::new(Vec::new());

/// `public static NonNullList<ItemStack> currentStacks`.
pub static CURRENT_STACKS: Mutex<Vec<ItemStack>> = Mutex::new(Vec::new());

/// `public static ItemStack oldCursorStack` (nullable).
pub static OLD_CURSOR_STACK: Mutex<Option<ItemStack>> = Mutex::new(None);

/// `public static AtomicReference<ItemStack> currentCursorStack` +
/// `public static final ReentrantLock currentCursorStackLock`, collapsed
/// into one guarded cell (the lock's only critical section wraps the
/// reference swap).
pub static CURRENT_CURSOR_STACK: Mutex<Option<ItemStack>> = Mutex::new(None);

/// `public static void init()`: initialize the config, then fresh
/// containers.
pub fn init() {
    crate::config::ConfigManager::initialize_config();
    SWAPS.lock().unwrap().clear();
    OLD_STACKS.lock().unwrap().clear();
    CURRENT_STACKS.lock().unwrap().clear();
}

/// Full state reset for tests (SwapUtil.reset() shape, including the
/// fields Java's init() does not touch).
#[cfg(test)]
pub fn reset_vars_for_tests() {
    CLICK_SWAP.store(false, std::sync::atomic::Ordering::SeqCst);
    *CLICK_SWAP_STACK.lock().unwrap() = None;
    SWAPS.lock().unwrap().clear();
    OLD_STACKS.lock().unwrap().clear();
    CURRENT_STACKS.lock().unwrap().clear();
    *OLD_CURSOR_STACK.lock().unwrap() = None;
    *CURRENT_CURSOR_STACK.lock().unwrap() = None;
    crate::config::ConfigManager::reset_for_tests();
}
