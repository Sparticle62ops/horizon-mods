/*
 * Copyright (c) 2026 Schauweg.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as
 * published by the Free Software Foundation, version 3 of the License
 * (LGPL-3.0-only), or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this program. If not, see
 * <https://www.gnu.org/licenses/>.
 */
//!
//! Exact 1:1 translation of Smooth Swapping (upstream Java, LGPL-3.0-only),
//! branch 26.3/dev @88e2988. The Minecraft/Fabric/ModMenu APIs that do not
//! exist on Horizon are host seams with the same shapes and semantics. The
//! fabric entrypoint `SmoothSwappingFabric` maps to the carrier's client-init
//! hook (`fabric::on_initialize_client`); the ModMenu entrypoint joins the
//! ConfigScreen batch later.


//!
//! Upstream: https://github.com/Schauweg/Smooth-Swapping, branch 26.3/dev
//! @88e2988 (LGPL-3.0-only). Ported 1:1 per the port policy; credit
//! Schauweg. Architectury common + fabric sources; the neoforge loader
//! shim is not ported (Horizon is the loader surface).
//!
//! Core/common package first: Vec2, SwapStacks, the swaps trio, and the
//! SmoothSwapping object state; SwapUtil and the config screen, then the
//! mixins, follow in later batches.


/// `dev.shwg.smoothswapping.SwapStacks`, 1:1.
#[derive(Debug, Clone)]
pub struct SwapStacks {
    old_stack: crate::item::ItemStack,
    new_stack: crate::item::ItemStack,
    /// `public int itemCountToChange` (mutable field upstream).
    pub item_count_to_change: i32,
    slot_id: i32,
}

impl SwapStacks {
    /// `new SwapStacks(int slotID, ItemStack oldStack, ItemStack newStack,
    /// int itemCountToChange)`.
    pub fn new(
        slot_id: i32,
        old_stack: crate::item::ItemStack,
        new_stack: crate::item::ItemStack,
        item_count_to_change: i32,
    ) -> Self {
        Self {
            old_stack,
            new_stack,
            item_count_to_change,
            slot_id,
        }
    }

    /// `getOldStack()`.
    pub fn get_old_stack(&self) -> &crate::item::ItemStack {
        &self.old_stack
    }

    /// `getNewStack()`.
    pub fn get_new_stack(&self) -> &crate::item::ItemStack {
        &self.new_stack
    }

    /// `getSlotID()`.
    pub fn get_slot_id(&self) -> i32 {
        self.slot_id
    }

    /// `toString()` (Java shape kept).
    pub fn to_display_string(&self) -> String {
        format!(
            "SwapStacks{{oldStack={}, newStack={}, itemCountToChange={}}}",
            self.old_stack, self.new_stack, self.item_count_to_change
        )
    }
}
pub mod vec2 {
//! Upstream `dev.shwg.smoothswapping.Vec2.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

/// `public class Vec2 implements Comparable<Vec2>`.
///
/// Upstream quirks kept verbatim, including the `v[0] =- b.v[0]` instance
/// `diff` (Java parses it as `v[0] = -b.v[0]`, a negation, not a
/// subtraction) and the degree-based `radg` trig.
#[derive(Debug, Clone, PartialEq)]
pub struct Vec2 {
    /// `public double[] v; // coordinates of the Point`
    pub v: [f64; 2],
}

/// `static double Resolution = 72;`
pub static RESOLUTION: f64 = 72.0;
/// `static int xoffset = 120;`
pub static XOFFSET: i32 = 120;
/// `static int yoffset = 120;`
pub static YOFFSET: i32 = 120;
/// `final static double radg = Math.atan(1) / 45.0;` (pi/180).
pub fn radg() -> f64 {
    1f64.atan() / 45.0
}

impl Vec2 {
    /// No-argument constructor `Vec2()` -> `this(0, 0)`.
    pub fn new() -> Self {
        Self::with_xy(0.0, 0.0)
    }

    /// Constructor `Vec2(double x, double y)`.
    pub fn with_xy(x: f64, y: f64) -> Self {
        Self { v: [0.0; 2] }.set(x, y)
    }

    /// Copy constructor `Vec2(Vec2 src)`.
    pub fn from_src(src: &Vec2) -> Self {
        Self::with_xy(src.v[0], src.v[1])
    }

    /// Constructor (angle theta) `Vec2(double theta)`: theta in degrees.
    pub fn from_angle(theta: f64) -> Self {
        let theta = theta * radg();
        Self {
            v: [0.0; 2],
        }
        .set(theta.cos(), theta.sin())
    }

    /// `public int length()`.
    pub fn length(&self) -> usize {
        self.v.len()
    }

    /// Set x and y coordinates of Point.
    pub fn set(mut self, x: f64, y: f64) -> Self {
        self.v[0] = x;
        self.v[1] = y;
        self
    }

    /// `public int[] convert()`.
    pub fn convert(&self) -> [i32; 2] {
        let mut iv = [0; 2];
        self.convert_into(&mut iv);
        iv
    }

    /// `public void convert(int[] iv)`.
    pub fn convert_into(&self, iv: &mut [i32; 2]) {
        iv[0] = (XOFFSET as f64 + self.v[0] * RESOLUTION) as i32;
        iv[1] = (YOFFSET as f64 - self.v[1] * RESOLUTION) as i32;
    }

    /// `static public Vec2 sum(Vec2 a, Vec2 b)`.
    pub fn sum(a: &Vec2, b: &Vec2) -> Vec2 {
        Vec2::with_xy(a.v[0] + b.v[0], a.v[1] + b.v[1])
    }

    /// `public Vec2 sum(Vec2 b)` (in place).
    pub fn sum_in_place(&mut self, b: &Vec2) -> &mut Self {
        self.v[0] += b.v[0];
        self.v[1] += b.v[1];
        self
    }

    /// `public Vec2 sumScalar(double b)` (in place).
    pub fn sum_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] += b;
        self.v[1] += b;
        self
    }

    /// `public Vec2 diffScalar(double b)` (in place).
    pub fn diff_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] -= b;
        self.v[1] -= b;
        self
    }

    /// `public Vec2 divideScalar(double b)` (in place).
    pub fn divide_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] /= b;
        self.v[1] /= b;
        self
    }

    /// `public Vec2 multiplyScalar(double b)` (in place).
    pub fn multiply_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] *= b;
        self.v[1] *= b;
        self
    }

    /// `public Vec2 diff(Vec2 b)`. Upstream body is `v[0] =- b.v[0];
    /// v[1] =- b.v[1]`, a negation (the assignment operator `=-`), kept
    /// 1:1; see also the static [`Vec2::diff_of`].
    pub fn diff_negate(&mut self, b: &Vec2) -> &mut Self {
        self.v[0] = -b.v[0];
        self.v[1] = -b.v[1];
        self
    }

    /// `static public Vec2 diff(Vec2 a, Vec2 b)`.
    pub fn diff_of(a: &Vec2, b: &Vec2) -> Vec2 {
        Vec2::with_xy(a.v[0] - b.v[0], a.v[1] - b.v[1])
    }

    /// Returns magnitude of 2D vector.
    pub fn magn(&self) -> f64 {
        (self.v[0] * self.v[0] + self.v[1] * self.v[1]).sqrt()
    }

    /// Multiply point by a scalar.
    pub fn scale(mut self, s: f64) -> Self {
        self.v[0] *= s;
        self.v[1] *= s;
        self
    }

    /// `public Vec2 copy()`.
    pub fn copy(&self) -> Vec2 {
        Vec2::from_src(self)
    }

    /// Offset (translate) point by the amount (tx, ty).
    pub fn translate(&mut self, tx: f64, ty: f64) -> &mut Self {
        self.v[0] += tx;
        self.v[1] += ty;
        self
    }

    /// Offset (translate) point by a Vec2.
    pub fn translate_by(&mut self, b: &Vec2) -> &mut Self {
        self.v[0] += b.v[0];
        self.v[1] += b.v[1];
        self
    }

    /// Rotate point about the origin (theta in degrees).
    pub fn rotate(&mut self, theta: f64) -> &mut Self {
        let theta = theta * radg();
        let c = theta.cos();
        let s = theta.sin();
        let t = self.v[1] * c + self.v[0] * s;
        self.v[0] = self.v[0] * c - self.v[1] * s;
        self.v[1] = t;
        self
    }

    /// `public Vec2 perp()` (in place).
    pub fn perp(&mut self) -> &mut Self {
        let t = self.v[0];
        self.v[0] = -self.v[1];
        self.v[1] = t;
        self
    }

    /// `static public Vec2 perp(Vec2 a)`.
    pub fn perp_of(a: &Vec2) -> Vec2 {
        Vec2::with_xy(-a.v[1], a.v[0])
    }

    /// `static public double perpdot(Vec2 a, Vec2 b)`.
    pub fn perpdot(a: &Vec2, b: &Vec2) -> f64 {
        a.v[0] * b.v[1] - a.v[1] * b.v[0]
    }

    /// `static public double dot(Vec2 a, Vec2 b)`.
    pub fn dot(a: &Vec2, b: &Vec2) -> f64 {
        a.v[0] * b.v[0] + a.v[1] * b.v[1]
    }

    /// `static public double distance(Vec2 a, Vec2 b)`.
    pub fn distance(a: &Vec2, b: &Vec2) -> f64 {
        Vec2::diff_of(a, b).magn()
    }

    /// Returns angle (in degrees) between two vectors.
    pub fn angle(a: &Vec2, b: &Vec2) -> f64 {
        let c = Vec2::dot(a, b);
        let s = Vec2::perpdot(a, b);
        s.atan2(c) / radg()
    }

    /// `public void setSize(double sz)`.
    pub fn set_size(&mut self, sz: f64) {
        let vnorm = self.magn();
        if vnorm == 0.0 {
            return;
        }
        let sz = sz / vnorm;
        self.v[0] *= sz;
        self.v[1] *= sz;
    }

    /// Convert the point into a String representation (`String.format
    /// ("[%.3g %.3g]", ...)` with the default locale's decimal point).
    pub fn to_display_string(&self) -> String {
        format!("[{} {}]", java_format_g(self.v[0]), java_format_g(self.v[1]))
    }
}

/// Java `String.format("%.3g", x)` (ROUND_HALF_UP, significant digits,
/// trailing zeros stripped, exponent notation above ~10^-4 and below
/// 10^precision-1).
pub fn java_format_g(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let neg = x < 0.0;
    let a = x.abs();
    let exp10 = a.log10().floor() as i32;
    let decimals = 2 - exp10;
    if decimals < 0 {
        // %g switches to scientific notation when exp >= precision.
        let mantissa = a / 10f64.powi(exp10);
        format!("{}{}e+{:02}", if neg { "-" } else { "" }, java_g_digits(mantissa), exp10)
    } else if decimals > 20 {
        // Very small magnitudes: %g switches to scientific notation.
        let mantissa = a / 10f64.powi(exp10);
        format!("{}{}e-{:02}", if neg { "-" } else { "" }, java_g_digits(mantissa), -exp10)
    } else {
        let scaled = (a * 10f64.powi(decimals) + 0.5).floor() / 10f64.powi(decimals);
        let mut s = format!("{:.*}", decimals.max(0) as usize, scaled);
        if s.contains('.') {
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
        }
        format!("{}{}", if neg { "-" } else { "" }, s)
    }
}

fn java_g_digits(m: f64) -> String {
    let s = format!("{:.2}", m);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    s
}

/// `Comparable<Vec2>`: `compareTo` compares `v[0]` with `Double.compare`
/// semantics (used by `Collections.sort` in the config widget).
pub fn compare_to(a: &Vec2, b: &Vec2) -> std::cmp::Ordering {
    a.v[0].total_cmp(&b.v[0])
}
}

pub mod item {
//! `net.minecraft.world.item.ItemStack` and `Items.AIR` seams, test-shaped
//! and kept minimal for the port's logic: identity by item id, a count,
//! and `hashCode()` (`Objects.hash(item)`-shaped).
//!
//! The swap logic only reads item identity and counts, so the surface
//! mirrors exactly those reads 1:1.

use std::sync::atomic::{AtomicU32, Ordering};

/// `net.minecraft.world.item.ItemStack`.
#[derive(Debug, Clone)]
pub struct ItemStack {
    /// Item registry identity (Java: the `Item` reference identity the
    /// hash and `isSameItem` reads use).
    pub item_id: u32,
    /// `getCount()`.
    pub count: i32,
    /// `ItemStackMixin`'s `@Unique private boolean isSwapStack`
    /// (surfaced by `ItemStackAccessor`). Upstream does not participate in
    /// equals/hashCode, so it stays out of the impls here too.
    pub is_swap_stack: bool,
}

impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item_id == other.item_id && self.count == other.count
    }
}
impl Eq for ItemStack {}
impl std::hash::Hash for ItemStack {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.item_id.hash(state);
    }
}

static ITEM_SEQ: AtomicU32 = AtomicU32::new(1);

/// A fresh item id, for tests.
pub fn next_item_id() -> u32 {
    ITEM_SEQ.fetch_add(1, Ordering::SeqCst)
}

/// `net.minecraft.world.item.Items.AIR`.
pub const AIR_ID: u32 = 0;

impl ItemStack {
    /// `new ItemStack(item, count)`.
    pub fn new(item_id: u32, count: i32) -> Self {
        Self { item_id, count, is_swap_stack: false }
    }

    /// `Items.AIR.getDefaultInstance()`.
    pub fn air_default_instance() -> Self {
        Self::new(AIR_ID, 1)
    }

    /// `ItemStackAccessor.smooth_Swapping$setIsSwapStack(boolean)`.
    pub fn set_is_swap_stack(&mut self, is_swap_stack: bool) {
        self.is_swap_stack = is_swap_stack;
    }

    /// `ItemStackAccessor.smooth_Swapping$isSwapStack()`.
    pub fn is_swap_stack(&self) -> bool {
        self.is_swap_stack
    }

    /// Static `ItemStack.isSameItem(stack, other)`: same item identity.
    pub fn is_same_item(a: &ItemStack, b: &ItemStack) -> bool {
        a.item_id == b.item_id
    }

    /// `copy()`.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// `getCount()`.
    pub fn get_count(&self) -> i32 {
        self.count
    }

    /// `hashCode()` (upstream compares `s.hashCode() == stack.hashCode()`).
    pub fn hash_code(&self) -> i32 {
        // Java Objects.hash(item): 31 * 1 + item.hashCode(); ids are small
        // enough that the id IS the hash for our purposes.
        self.item_id as i32
    }
}

impl std::fmt::Display for ItemStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Java's default Object.toString(): identity-shaped.
        write!(f, "ItemStack@{:x}", self.item_id)
    }
}

impl ItemStack {
    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.get_count() <= 0
    }

    /// The item slot is `Items.AIR`.
    pub fn is_air(&self) -> bool {
        ItemStack::is_same_item(self, &ItemStack::air_default_instance())
    }

    /// `ItemStack.matches(ItemStack, ItemStack)`: count + item (components
    /// stay on the seam; the port's stack carries none).
    pub fn matches(a: &ItemStack, b: &ItemStack) -> bool {
        a.get_count() == b.get_count() && ItemStack::is_same_item(a, b)
    }

    /// `isBarVisible()` — durability-bar seam (test-shaped static; the
    /// launcher integration supplies the real read).
    pub fn is_bar_visible(&self) -> bool {
        BAR_VISIBLE.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// `isBarVisible()` seam.
pub static BAR_VISIBLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
pub fn set_bar_visible_for_tests(value: bool) {
    BAR_VISIBLE.store(value, std::sync::atomic::Ordering::SeqCst);
}
}

pub mod slot {
//! `net.minecraft.world.inventory.Slot` seam: the public `x`/`y` screen
//! positions the swap constructors read and `getItem()` for SwapUtil. One
//! slot per id. Identity (equals/hashCode) stays on the slot position,
//! like the game's own slot identity.

use super::item::ItemStack;

/// `net.minecraft.world.inventory.Slot`.
#[derive(Debug, Clone)]
pub struct Slot {
    /// `public int x`.
    pub x: i32,
    /// `public int y`.
    pub y: i32,
    /// `getItem()` — the stack the slot currently holds (test-shaped:
    /// the SwapUtil reads are `isSameItem(air)` and `copy()`).
    item: ItemStack,
    /// `allowModification(Player)` — vanilla gates on the slot's container
    /// type; the port pins it per slot.
    pub allow_modification: bool,
}

impl PartialEq for Slot {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for Slot {}
impl std::hash::Hash for Slot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl Slot {
    /// Position-only slot (item defaults to `Items.AIR`).
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y, item: ItemStack::air_default_instance(), allow_modification: true }
    }

    /// Slot with its current stack.
    pub fn with_item(x: i32, y: i32, item: ItemStack) -> Self {
        Self { x, y, item, allow_modification: true }
    }

    /// `hasItem()`.
    pub fn has_item(&self) -> bool {
        !ItemStack::is_same_item(&self.item, &ItemStack::air_default_instance())
    }

    /// `getItem()`.
    pub fn get_item(&self) -> &ItemStack {
        &self.item
    }
}
}

pub mod state {
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

use super::item::ItemStack;
use super::swaps::AnySwap;

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
    super::config::ConfigManager::initialize_config();
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
    super::config::ConfigManager::reset_for_tests();
}
}

pub mod swap_util {
//! Upstream `SwapUtil.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */
//!
//! `AbstractContainerMenu` reaches the port through the `MenuSeam` trait
//! (its only reads here are `getSlot(int)` and `getCarried()`).

use std::sync::atomic::Ordering;

use super::item::ItemStack;
use super::slot::Slot;
use super::state::{self, ASSUME_CURSOR_STACK_SLOT_INDEX};
use super::swaps::item_to_cursor_inventory_swap::ItemToCursorInventorySwap;
use super::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap;
use super::swaps::{AnySwap, InventorySwap};
use super::vec2::Vec2;

use std::f64::consts::PI;

/// `AbstractContainerMenu` seam.
pub trait MenuSeam {
    /// `getSlot(int slotIndex)`.
    fn get_slot(&self, slot_id: i32) -> Slot;
    /// `getCarried()`.
    fn get_carried(&self) -> ItemStack;
    /// `screenHandler.slots.size()` (`ServerboundContainerClickPacketMixin`).
    fn slots_len(&self) -> usize { 0 }
    /// `handledScreen instanceof CreativeModeInventoryScreen`
    /// (`AbstractContainerScreenMixin`'s early return).
    fn is_creative_mode_inventory_screen(&self) -> bool { false }
}

/// `public static boolean hasArrived(InventorySwap swap)`.
pub fn has_arrived(swap: &InventorySwap) -> bool {
    let quadrant = get_quadrant(swap.get_angle());
    let x = swap.get_x();
    let y = swap.get_y();
    if quadrant == 0 && x > 0.0 && y > 0.0 {
        true
    } else if quadrant == 1 && x < 0.0 && y > 0.0 {
        true
    } else if quadrant == 2 && x < 0.0 && y < 0.0 {
        true
    } else {
        quadrant == 3 && x > 0.0 && y < 0.0
    }
}

/// `public static int getSlotIndex(ItemStack stack)`.
pub fn get_slot_index(stack: &ItemStack) -> i32 {
    let current = state::CURRENT_STACKS.lock().unwrap();
    for (i, s) in current.iter().enumerate() {
        if s.hash_code() == stack.hash_code() {
            return i as i32;
        }
    }
    -1
}

/// `public static void setRenderToTrue(List<InventorySwap> swapList)`.
pub fn set_render_to_true(swap_list: &mut [AnySwap]) {
    for swap in swap_list.iter_mut() {
        swap.set_render_destination_slot(true);
    }
}

/// `private static int getQuadrant(double angle)`. Java casts the whole
/// `floor(2 * angle / PI) % 4 + 4` expression to `int` (truncate) before
/// the final `% 4`.
fn get_quadrant(angle: f64) -> i32 {
    let expr = (2.0 * angle / PI).floor() % 4.0 + 4.0;
    (expr as i64 % 4) as i32
}

/// `public static double map(double in, double inMin, double inMax,
/// double outMax, double outMin)`. Upstream names the last two parameters
/// outMax/outMin in that order and uses them exactly as written; the
/// order is kept.
pub fn map(input: f64, in_min: f64, in_max: f64, out_max: f64, out_min: f64) -> f64 {
    (input - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}

/// `public static void addI2IInventorySwap(int index, Slot fromSlot,
/// Slot toSlot, boolean checked, int amount)`.
pub fn add_i2i_inventory_swap(index: i32, from_slot: &Slot, to_slot: &Slot, checked: bool, amount: i32) {
    if ItemStack::is_same_item(&to_slot.get_item(), &ItemStack::air_default_instance()) {
        return;
    }

    let mut swap_stack = to_slot.get_item().copy();
    // ((ItemStackAccessor) (Object) swapStack).smooth_Swapping$setIsSwapStack(true);
    swap_stack.set_is_swap_stack(true);

    let swap = AnySwap::ItemToItem(ItemToItemInventorySwap::new(from_slot, to_slot, checked, amount, swap_stack));
    state::SWAPS.lock().unwrap().entry(index).or_default().push(swap);
}

/// `public static void assignI2CSwaps(List<SwapStacks> lessStacks,
/// Vec2 mousePos, AbstractContainerMenu handler)`.
pub fn assign_i2c_swaps(less_stacks: &[super::SwapStacks], mouse_pos: &Vec2, handler: &dyn MenuSeam) {
    let cursor_stack = handler.get_carried();

    for less_stack in less_stacks {
        let less_slot = handler.get_slot(less_stack.get_slot_id());
        // Java re-reads the map per iteration via getOrDefault(ASSUME_..., new ArrayList<>()).
        if ItemStack::is_same_item(&cursor_stack, &ItemStack::air_default_instance()) {
            return;
        }

        let mut swap_stack = less_stack.get_old_stack().copy();
        swap_stack.set_is_swap_stack(true);

        let swap = AnySwap::ItemToCursor(ItemToCursorInventorySwap::new(
            &less_slot,
            mouse_pos,
            less_stack.get_old_stack().copy(),
            false,
            less_stack.item_count_to_change,
        ));
        state::SWAPS
            .lock()
            .unwrap()
            .entry(ASSUME_CURSOR_STACK_SLOT_INDEX)
            .or_default()
            .push(swap);
    }
}

/// `public static void assignI2ISwaps(List<SwapStacks> moreStacks,
/// List<SwapStacks> lessStacks, AbstractContainerMenu handler)`.
///
/// Java mutates both lists in place inside indexed for-loops; each
/// `remove(element)` call removes the object the local refers to (the
/// element at the current index), and the loop index still increments,
/// so the element that shifts into that index is skipped. The clones
/// below keep that exact flow: the local keeps its own mutated count,
/// and the value is written back while the element is still in the list.
pub fn assign_i2i_swaps(more_stacks: &mut Vec<super::SwapStacks>, less_stacks: &mut Vec<super::SwapStacks>, handler: &dyn MenuSeam) {
    let mut i: usize = 0;
    while i < more_stacks.len() {
        let mut more_stack = more_stacks[i].clone();
        let removed_more = more_stack.item_count_to_change == 0;
        if removed_more {
            more_stacks.remove(i);
        }
        let more_slot = handler.get_slot(more_stack.get_slot_id());

        let mut c = 0;
        while more_stack.item_count_to_change < 0 && c < 64 {
            c += 1;
            let mut j: usize = 0;
            while j < less_stacks.len() {
                let mut less_stack = less_stacks[j].clone();

                let mut amount = 0;
                while less_stack.item_count_to_change != 0 && more_stack.item_count_to_change != 0 {
                    less_stack.item_count_to_change -= 1;
                    more_stack.item_count_to_change += 1;
                    amount += 1;
                }
                // Java's local is the list element; write the mutated count back.
                if j < less_stacks.len() {
                    less_stacks[j].item_count_to_change = less_stack.item_count_to_change;
                }

                let less_slot = handler.get_slot(less_stack.get_slot_id());
                add_i2i_inventory_swap(
                    more_stack.get_slot_id(),
                    &less_slot,
                    &more_slot,
                    ItemStack::is_same_item(&more_stack.get_old_stack(), &more_stack.get_new_stack()),
                    amount,
                );
                if less_stack.item_count_to_change == 0 {
                    less_stacks.remove(j);
                }
                if more_stack.item_count_to_change == 0 {
                    break;
                }
                j += 1;
            }
        }
        if !removed_more && i < more_stacks.len() {
            more_stacks[i].item_count_to_change = more_stack.item_count_to_change;
        }
        i += 1;
    }
}

/// `public static int getCount(ItemStack stack)`.
pub fn get_count(stack: &ItemStack) -> i32 {
    if ItemStack::is_same_item(stack, &ItemStack::air_default_instance()) {
        0
    } else {
        stack.get_count()
    }
}

/// `public static void copyStacks(NonNullList<ItemStack> src,
/// NonNullList<ItemStack> dst)`.
pub fn copy_stacks(src: &[ItemStack], dst: &mut Vec<ItemStack>) {
    dst.clear();
    for s in src {
        dst.push(s.copy());
    }
}

/// `public static int swapListIndexOf(List<InventorySwap> list,
/// Function<InventorySwap, Boolean> prediction)`.
pub fn swap_list_index_of(list: &[AnySwap], prediction: &dyn Fn(&AnySwap) -> bool) -> i32 {
    for (i, s) in list.iter().enumerate() {
        if prediction(s) {
            return i as i32;
        }
    }
    -1
}

/// `public static void reset()`.
pub fn reset() {
    state::CLICK_SWAP.store(false, Ordering::SeqCst);
    *state::CLICK_SWAP_STACK.lock().unwrap() = None;
    state::SWAPS.lock().unwrap().clear();
    state::OLD_STACKS.lock().unwrap().clear();
    state::CURRENT_STACKS.lock().unwrap().clear();
    *state::OLD_CURSOR_STACK.lock().unwrap() = None;
}
}

pub mod catmull_rom {
//! Upstream `config/CatmullRomWidget.java`, math half 1:1:
//! `splinesFromPoints`, `getProgress`, `getSegmentForT`, and the
//! `CatmullRomSpline`/`Segment` inner classes. The widget UI half
//! (extractWidgetRenderState, mouse handling, tooltip state) lands with
//! the config-screen batch.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use super::swap_util;
use super::vec2::Vec2;

/// `static float tension = 0f`.
pub const TENSION: f32 = 0.0;
/// `static float alpha = 0f`.
pub const ALPHA: f32 = 0.0;

/// `public static class CatmullRomSpline`.
#[derive(Debug, Clone)]
pub struct CatmullRomSpline {
    segment: Segment,
    /// `float x` (package-private).
    pub x: f32,
    /// `float oldX` (package-private).
    pub old_x: f32,
}

/// Private `record Segment(Vec2 a, Vec2 b, Vec2 c, Vec2 d)`.
#[derive(Debug, Clone)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
    pub d: Vec2,
}

impl Segment {
    /// `getPoint(double t)`.
    pub fn get_point(&self, t: f64) -> Vec2 {
        let mut a = self.a.copy();
        a.multiply_scalar(t * t * t);
        let mut b = self.b.copy();
        b.multiply_scalar(t * t);
        let mut c = self.c.copy();
        c.multiply_scalar(t);
        let bc = Vec2::sum(&b, &c);
        let abc = Vec2::sum(&a, &bc);
        Vec2::sum(&abc, &self.d)
    }
}

impl CatmullRomSpline {
    /// `new CatmullRomSpline(Vec2 p0, Vec2 p1, Vec2 p2, Vec2 p3)`.
    pub fn new(p0: &Vec2, p1: &Vec2, p2: &Vec2, p3: &Vec2) -> Self {
        let x = p2.v[0] as f32;
        let old_x = p1.v[0] as f32;

        let t01 = Vec2::distance(p0, p1).powf(ALPHA as f64);
        let t12 = Vec2::distance(p1, p2).powf(ALPHA as f64);
        let t23 = Vec2::distance(p2, p3).powf(ALPHA as f64);

        // m1 = Vec2.sum(Vec2.diff(p2, p1), (Vec2.diff(
        //     Vec2.diff(p1, p0).divideScalar(t01),
        //     Vec2.diff(p2, p0).divideScalar(t01 + t12))).multiplyScalar(t12))
        //     .multiplyScalar(1.0f - tension);
        let mut d10 = Vec2::diff_of(p1, p0);
        d10.divide_scalar(t01);
        let mut d20 = Vec2::diff_of(p2, p0);
        d20.divide_scalar(t01 + t12);
        let mut inner = Vec2::diff_of(&d10, &d20);
        inner.multiply_scalar(t12);
        let mut m1 = Vec2::diff_of(p2, p1);
        m1.sum_in_place(&inner);
        m1.multiply_scalar((1.0 - TENSION) as f64);

        // m2 = Vec2.sum(Vec2.diff(p2, p1), (Vec2.diff(
        //     Vec2.diff(p3, p2).divideScalar(t23),
        //     Vec2.diff(p3, p1).divideScalar(t12 + t23))).multiplyScalar(t12))
        //     .multiplyScalar(1.0f - tension);
        let mut d32 = Vec2::diff_of(p3, p2);
        d32.divide_scalar(t23);
        let mut d31 = Vec2::diff_of(p3, p1);
        d31.divide_scalar(t12 + t23);
        let mut inner2 = Vec2::diff_of(&d32, &d31);
        inner2.multiply_scalar(t12);
        let mut m2 = Vec2::diff_of(p2, p1);
        m2.sum_in_place(&inner2);
        m2.multiply_scalar((1.0 - TENSION) as f64);

        // segment = new Segment(
        //     Vec2.sum(Vec2.diff(p1, p2).multiplyScalar(2.0d), Vec2.sum(m1, m2)),
        //     Vec2.diff(Vec2.diff(Vec2.diff(Vec2.diff(p1, p2).multiplyScalar(-3), m1), m1), m2),
        //     m1, p1);
        let mut twice = Vec2::diff_of(p1, p2);
        twice.multiply_scalar(2.0);
        let m12 = Vec2::sum(&m1, &m2);
        let a = Vec2::sum(&twice, &m12);

        let mut neg_three = Vec2::diff_of(p1, p2);
        neg_three.multiply_scalar(-3.0);
        let s1 = Vec2::diff_of(&neg_three, &m1);
        let s2 = Vec2::diff_of(&s1, &m1);
        let b = Vec2::diff_of(&s2, &m2);

        let c = m1;
        let d = p1.copy();

        Self { segment: Segment { a, b, c, d }, x, old_x }
    }

    /// `private Segment getSegment()` (crate-visible for getProgress and
    /// the widget batch).
    pub fn segment(&self) -> &Segment {
        &self.segment
    }
}

/// `public static List<CatmullRomSpline> splinesFromPoints(List<Vec2> points)`.
pub fn splines_from_points(points: &[Vec2]) -> Vec<CatmullRomSpline> {
    let mut splines = Vec::new();
    // Java: for (int i = 1; i < points.size() - 2; i++) — an int bound,
    // so short lists just produce no iterations.
    let end = (points.len() as isize - 2).max(0) as usize;
    for i in 1..end {
        let p0 = &points[i - 1];
        let p1 = &points[i];
        let p2 = &points[i + 1];
        let p3 = &points[i + 2];
        splines.push(CatmullRomSpline::new(p0, p1, p2, p3));
    }
    splines
}

/// `public static double getProgress(double t, List<CatmullRomSpline> segments)`.
pub fn get_progress(t: f64, segments: &[CatmullRomSpline]) -> f64 {
    let current_segment = get_segment_for_t(t, segments);
    // SwapUtil.map(t, currentSegment.oldX, currentSegment.x, 1, 0)
    let progress = swap_util::map(t, current_segment.old_x as f64, current_segment.x as f64, 1.0, 0.0);
    current_segment.segment().get_point(progress).v[1]
}

/// `private static CatmullRomSpline getSegmentForT(double t,
/// List<CatmullRomSpline> segments)`. Falls back to `segments.get(0)`
/// (which throws when empty, as upstream does).
fn get_segment_for_t(t: f64, segments: &[CatmullRomSpline]) -> &CatmullRomSpline {
    for spline in segments {
        if t >= spline.old_x as f64 && t < spline.x as f64 {
            return spline;
        }
    }
    &segments[0]
}
}

pub mod config {
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

use super::catmull_rom::{self, CatmullRomSpline};
use super::state::{Gson, GSON, MOD_ID};
use super::vec2::Vec2;

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
}

pub mod swaps {

pub use inventory_swap::InventorySwap;

/// Java stores `List<InventorySwap>` with both concrete swap types; the
/// flattened Rust structs are held behind this enum.
#[derive(Debug, Clone)]
pub enum AnySwap {
    ItemToItem(item_to_item_inventory_swap::ItemToItemInventorySwap),
    ItemToCursor(item_to_cursor_inventory_swap::ItemToCursorInventorySwap),
}

impl AnySwap {
    pub fn base(&self) -> &InventorySwap {
        match self {
            AnySwap::ItemToItem(s) => &s.base,
            AnySwap::ItemToCursor(s) => &s.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut InventorySwap {
        match self {
            AnySwap::ItemToItem(s) => &mut s.base,
            AnySwap::ItemToCursor(s) => &mut s.base,
        }
    }

    /// `setRenderDestinationSlot(boolean)`.
    pub fn set_render_destination_slot(&mut self, render: bool) {
        self.base_mut().set_render_destination_slot(render);
    }
}
pub mod inventory_swap {
//! Upstream `swaps/InventorySwap.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use super::super::item::ItemStack;
use super::super::vec2::Vec2;

/// `public class InventorySwap`.
#[derive(Debug, Clone)]
pub struct InventorySwap {
    /// `private double x, y` (animated remainder, mutable).
    pub x: f64,
    pub y: f64,
    /// `private final double distance, startX, startY, angle`.
    pub distance: f64,
    pub start_x: f64,
    pub start_y: f64,
    pub angle: f64,
    /// `private boolean renderDestinationSlot` (mutable).
    pub render_destination_slot: bool,
    /// `private final boolean checked`.
    pub checked: bool,
    /// `private final int amount`.
    pub amount: i32,
    /// `private final ItemStack swapItem`.
    pub swap_item: ItemStack,
}

impl InventorySwap {
    /// `new InventorySwap(Vec2 fromVec, Vec2 toVec, ItemStack swapItem,
    /// boolean checked, int amount)`. Upstream casts the angle through
    /// `(float)`; kept.
    pub fn new(from_vec: &Vec2, to_vec: &Vec2, swap_item: ItemStack, checked: bool, amount: i32) -> Self {
        let x = to_vec.v[0] - from_vec.v[0];
        let y = to_vec.v[1] - from_vec.v[1];
        let start_x = to_vec.v[0] - from_vec.v[0];
        let start_y = to_vec.v[1] - from_vec.v[1];
        let angle = (y.atan2(x) + std::f64::consts::PI) as f32 as f64;
        let distance = (x * x + y * y).sqrt();
        Self {
            x,
            y,
            distance,
            start_x,
            start_y,
            angle,
            render_destination_slot: false,
            checked,
            amount,
            swap_item,
        }
    }

    /// `getAngle()`.
    pub fn get_angle(&self) -> f64 {
        self.angle
    }

    /// `getX()` / `getY()`.
    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    /// `getStartX()` / `getStartY()`.
    pub fn get_start_x(&self) -> f64 {
        self.start_x
    }

    pub fn get_start_y(&self) -> f64 {
        self.start_y
    }

    /// `getDistance()`.
    pub fn get_distance(&self) -> f64 {
        self.distance
    }

    /// `renderDestinationSlot()`.
    pub fn render_destination_slot(&self) -> bool {
        self.render_destination_slot
    }

    /// `setRenderDestinationSlot(boolean)`.
    pub fn set_render_destination_slot(&mut self, render_destination_slot: bool) {
        self.render_destination_slot = render_destination_slot;
    }

    /// `isChecked()`.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// `getAmount()`.
    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    /// `getSwapItem()`.
    pub fn get_swap_item(&self) -> &ItemStack {
        &self.swap_item
    }

    /// `toString()` (Java shape kept, fields in upstream order).
    pub fn to_display_string(&self) -> String {
        format!(
            "InventorySwap{{x={}, y={}, distance={}, startX={}, startY={}, angle={}, renderDestinationSlot={}, checked={}, amount={}, swapItem={}}}",
            self.x,
            self.y,
            self.distance,
            self.start_x,
            self.start_y,
            self.angle,
            self.render_destination_slot,
            self.checked,
            self.amount,
            self.swap_item
        )
    }
}
}

pub mod item_to_cursor_inventory_swap {
//! Upstream `swaps/ItemToCursorInventorySwap.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use super::super::item::ItemStack;
use super::super::slot::Slot;
use super::inventory_swap::InventorySwap;
use super::super::vec2::Vec2;

/// `public class ItemToCursorInventorySwap extends InventorySwap`.
#[derive(Debug, Clone)]
pub struct ItemToCursorInventorySwap {
    /// The `InventorySwap` base state, flattened (Java single inheritance).
    pub base: InventorySwap,
    /// `private boolean startedRender`.
    pub started_render: bool,
    /// `private int copiedStackHash`.
    pub copied_stack_hash: i32,
    /// `private int targetStackHash`.
    pub target_stack_hash: i32,
    /// `private boolean arrived`.
    pub arrived: bool,
}

impl ItemToCursorInventorySwap {
    /// `new ItemToCursorInventorySwap(Slot fromSlot, Vec2
    /// relativeCursorPos, ItemStack fromStack, boolean checked, int
    /// amount)`. The slot half-size offset (16px slot, +8 center) is kept
    /// with upstream's comment.
    pub fn new(
        from_slot: &Slot,
        relative_cursor_pos: &Vec2,
        from_stack: ItemStack,
        checked: bool,
        amount: i32,
    ) -> Self {
        // slot has fixed size: 16
        // defined in net.minecraft.client.gui.screen.ingame.HandledScreen#drawSlot(
        //      net.minecraft.client.util.math.MatrixStack matrices,
        //      net.minecraft.screen.slot.Slot slot
        // )
        let base = InventorySwap::new(
            &Vec2::with_xy(from_slot.x as f64 + 8.0, from_slot.y as f64 + 8.0),
            relative_cursor_pos,
            from_stack,
            checked,
            amount,
        );
        Self {
            base,
            started_render: false,
            copied_stack_hash: 0,
            target_stack_hash: -1,
            arrived: false,
        }
    }
}

impl ItemToCursorInventorySwap {
    /// `isStartedRender()` / `setStartedRender(boolean)`.
    pub fn is_started_render(&self) -> bool {
        self.started_render
    }

    pub fn set_started_render(&mut self, started_render: bool) {
        self.started_render = started_render;
    }

    /// `getCopiedStackHash()` / `setCopiedStackHash(int)`.
    pub fn get_copied_stack_hash(&self) -> i32 {
        self.copied_stack_hash
    }

    pub fn set_copied_stack_hash(&mut self, copied_stack_hash: i32) {
        self.copied_stack_hash = copied_stack_hash;
    }

    /// `getTargetStackHash()` / `setTargetStackHash(int)`.
    pub fn get_target_stack_hash(&self) -> i32 {
        self.target_stack_hash
    }

    pub fn set_target_stack_hash(&mut self, target_stack_hash: i32) {
        self.target_stack_hash = target_stack_hash;
    }

    /// `isArrived()` / `setArrived(boolean)`.
    pub fn is_arrived(&self) -> bool {
        self.arrived
    }

    pub fn set_arrived(&mut self, arrived: bool) {
        self.arrived = arrived;
    }
}
}

pub mod item_to_item_inventory_swap {
//! Upstream `swaps/ItemToItemInventorySwap.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use super::super::item::ItemStack;
use super::super::slot::Slot;
use super::inventory_swap::InventorySwap;
use super::super::vec2::Vec2;

/// `public class ItemToItemInventorySwap extends InventorySwap`.
#[derive(Debug, Clone)]
pub struct ItemToItemInventorySwap {
    /// The `InventorySwap` base state, flattened (Java single inheritance).
    pub base: InventorySwap,
}

impl ItemToItemInventorySwap {
    /// `new ItemToItemInventorySwap(Slot fromSlot, Slot toSlot, boolean
    /// checked, int amount, ItemStack swapStack)`.
    pub fn new(
        from_slot: &Slot,
        to_slot: &Slot,
        checked: bool,
        amount: i32,
        swap_stack: ItemStack,
    ) -> Self {
        Self {
            base: InventorySwap::new(
                &Vec2::with_xy(from_slot.x as f64, from_slot.y as f64),
                &Vec2::with_xy(to_slot.x as f64, to_slot.y as f64),
                swap_stack,
                checked,
                amount,
            ),
        }
    }
}
}

}

pub mod mixins {
//! Upstream `mixin/` package, 1:1. Java mixins inject into game classes;
//! the port carries each mixin's handler as a plain module against the
//! game seams, keeping every injection site's shape and comment.


pub mod item_stack {
//! Upstream `mixin/ItemStackMixin.java`, 1:1. The `@Unique isSwapStack`
//! field is injected into `ItemStack` upstream; the port carries it on the
//! ItemStack seam, and this module is the `ItemStackAccessor` interface
//! these accessors implement.

use super::super::item::ItemStack;

/// `dev.shwg.smoothswapping.ItemStackAccessor`.
pub trait ItemStackAccessor {
    /// `smooth_Swapping$setIsSwapStack(boolean)`.
    fn smooth_swapping_set_is_swap_stack(&mut self, is_swap_stack: bool);
    /// `smooth_Swapping$isSwapStack()`.
    fn smooth_swapping_is_swap_stack(&self) -> bool;
}

impl ItemStackAccessor for ItemStack {
    fn smooth_swapping_set_is_swap_stack(&mut self, is_swap_stack: bool) {
        self.set_is_swap_stack(is_swap_stack);
    }

    fn smooth_swapping_is_swap_stack(&self) -> bool {
        self.is_swap_stack()
    }
}
}

pub mod simple_container_accessor {
//! Upstream `mixin/SimpleContainerAccessor.java`, 1:1.

use super::super::item::ItemStack;

/// `net.minecraft.world.SimpleContainer` access via `@Accessor("items")`.
pub trait SimpleContainerAccessor {
    /// `NonNullList<ItemStack> getStacks()`.
    fn get_stacks(&self) -> Vec<ItemStack>;
}
}

pub mod serverbound_container_click_packet {
//! Upstream `mixin/ServerboundContainerClickPacketMixin.java`, 1:1. The
//! `<init>` `@At("TAIL")` injection becomes `on_init`, called where the
//! game builds the packet; `Minecraft` and the player's menu reach the
//! handler through the `ClickPacketHost` seam.

use super::super::config::ConfigManager;
use super::super::item::ItemStack;
use super::super::slot::Slot;
use super::super::state;
use super::super::swap_util;
use std::collections::HashMap;

/// `net.minecraft.world.inventory.ContainerInput`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerInput {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickUpAll,
    QuickMoveAll,
    SwapWithOffhand,
}

/// `net.minecraft.network.HashedStack` (the shape the handler reads).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashedStack {
    /// `HashedStack.EMPTY`.
    Empty,
    /// `HashedStack.ActualItem`.
    ActualItem { item_id: u32, count: i32 },
}

impl HashedStack {
    /// `ActualItem.count()`.
    pub fn count(&self) -> i32 {
        match self {
            HashedStack::ActualItem { count, .. } => *count,
            HashedStack::Empty => 0,
        }
    }

    /// `newMouseStackHash instanceof HashedStack.ActualItem`.
    pub fn is_actual_item(&self) -> bool {
        matches!(self, HashedStack::ActualItem { .. })
    }
}

/// Game seams the packet handler reads: `Minecraft`, the screen class
/// check, and the player's container menu (`screenHandler`).
pub trait ClickPacketHost {
    /// `Minecraft.getInstance().gui.screen() instanceof AbstractContainerScreen`.
    fn gui_screen_is_container_screen(&self) -> bool;
    /// `Minecraft.getInstance().player != null` (asserted non-null upstream).
    fn player_present(&self) -> bool;
    /// `screenHandler.slots.size()`.
    fn screen_handler_slots_len(&self) -> usize;
    /// `screenHandler.getSlot(slot)`.
    fn screen_handler_get_slot(&self, slot_id: i32) -> Slot;
}

/// `@Inject(method = "<init>", at = @At("TAIL")) onInit(...)`.
pub fn on_init(
    sync_id: i32,
    revision: i32,
    slot: i16,
    button: i8,
    container_input: &ContainerInput,
    modified_stacks: &HashMap<i32, HashedStack>,
    cursor: &HashedStack,
    host: &dyn ClickPacketHost,
) {
    let _ = (sync_id, revision, button, cursor);
    if !ConfigManager::get_config().get_toggle_mod() {
        return;
    }
    // remove swap when stack gets moved before it arrived
    state::SWAPS.lock().unwrap().remove(&(slot as i32));

    if (*container_input == ContainerInput::QuickMove || *container_input == ContainerInput::Swap)
        && modified_stacks.len() > 1
        && host.gui_screen_is_container_screen()
    {
        // assert Minecraft.getInstance().player != null;
        debug_assert!(host.player_present());
        // LocalPlayer player = Minecraft.getInstance().player;
        // AbstractContainerMenu screenHandler = player.containerMenu;

        if slot >= 0 && (slot as i32) < host.screen_handler_slots_len() as i32 {
            let slot_id = slot as i32;
            let mouse_hover_slot = host.screen_handler_get_slot(slot_id);

            if *container_input == ContainerInput::QuickMove
                && !mouse_hover_slot.allow_modification
            {
                let new_mouse_stack_hash = modified_stacks.get(&slot_id);
                let old_mouse_stack = get_safe_old_stack(slot_id);

                // only if new items are less or equal (crafting table output for example)
                // (oldMouseStack != null: dead upstream once copyStacks fills
                // the list; the port reads ItemStack.EMPTY out of range)
                if let Some(HashedStack::ActualItem { count: new_mouse_count, .. }) =
                    new_mouse_stack_hash
                {
                    if new_mouse_count - old_mouse_stack.get_count() <= 0 {
                        *state::CLICK_SWAP_STACK.lock().unwrap() = Some(slot);
                    }
                }
            } else if *container_input == ContainerInput::Swap {
                state::CLICK_SWAP
                    .store(true, std::sync::atomic::Ordering::SeqCst);

                // for (Map.Entry<Integer, HashedStack> stackEntry :
                //     modifiedStacks.int2ObjectEntrySet())
                for (&destination_slot_id, _stack_entry) in modified_stacks {
                    if destination_slot_id >= 0
                        && destination_slot_id < host.screen_handler_slots_len() as i32
                        && destination_slot_id != slot_id
                    {
                        let destination_slot = host.screen_handler_get_slot(destination_slot_id);

                        let destination_old_stack = get_safe_old_stack(destination_slot_id);

                        if !mouse_hover_slot.allow_modification && destination_slot.allow_modification {
                            if destination_old_stack.is_empty() {
                                swap_util::add_i2i_inventory_swap(
                                    destination_slot_id,
                                    &mouse_hover_slot,
                                    &destination_slot,
                                    false,
                                    destination_slot.get_item().get_count(),
                                );
                            }
                        } else if mouse_hover_slot.allow_modification && destination_slot.allow_modification {
                            if destination_slot.has_item() {
                                swap_util::add_i2i_inventory_swap(
                                    destination_slot_id,
                                    &mouse_hover_slot,
                                    &destination_slot,
                                    false,
                                    destination_slot.get_item().get_count(),
                                );
                            }
                            if mouse_hover_slot.has_item() {
                                swap_util::add_i2i_inventory_swap(
                                    slot_id,
                                    &destination_slot,
                                    &mouse_hover_slot,
                                    false,
                                    mouse_hover_slot.get_item().get_count(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `@Unique smooth_Swapping$getSafeOldStack(int slot)`. `SmoothSwapping.oldStacks`
/// is the static in the port (Java lazily creates it); out-of-range reads
/// return `ItemStack.EMPTY`.
fn get_safe_old_stack(slot: i32) -> ItemStack {
    let stacks = state::OLD_STACKS.lock().unwrap();
    if slot < 0 || slot as usize >= stacks.len() {
        return ItemStack::air_default_instance();
    }
    stacks[slot as usize].copy()
}
}

pub mod abstract_container_screen {
//! Upstream `mixin/AbstractContainerScreenMixin.java`, 1:1. The
//! `extractRenderState` `@At("HEAD")` injection becomes `on_render`; the
//! screen's `menu`/`leftPos`/`topPos` arrive as parameters and `Minecraft`
//! reaches the handler through the `ScreenRenderHost` seam.

use super::super::config::ConfigManager;
use super::super::item::ItemStack;
use super::super::state;
use super::super::swap_util;
use super::super::vec2::Vec2;
use std::collections::HashMap;

/// `Minecraft`/player seams the render pass reads.
pub trait ScreenRenderHost {
    /// `Minecraft.getInstance().gui.screen()` — identity token for the
    /// screen reference comparison.
    fn gui_screen_token(&self) -> u64;
    /// `client.player == null || client.player.containerMenu == null`.
    fn player_menu_present(&self) -> bool;
    /// `client.player.containerMenu.getItems()`.
    fn player_menu_items(&self) -> Vec<ItemStack>;
    /// `client.player.containerMenu.getCarried()`.
    fn player_menu_carried(&self) -> ItemStack;
}

/// `@Inject(method = "extractRenderState", at = @At("HEAD"))
/// onRender(...)`. The `catch (Exception e) { SwapUtil.reset(); }` becomes
/// the Err path; `current_screen` is the `@Unique smooth_Swapping$currentScreen`
/// field the caller carries for this mixin instance.
pub fn on_render(
    mouse_x: f64,
    mouse_y: f64,
    menu: &dyn swap_util::MenuSeam,
    left_pos: i32,
    top_pos: i32,
    host: &dyn ScreenRenderHost,
    current_screen: &mut Option<u64>,
) {
    if do_render(mouse_x, mouse_y, menu, left_pos, top_pos, host, current_screen).is_err() {
        swap_util::reset();
    }
}

/// The injection body, kept whole. `Result<(), ()>` carries the upstream
/// `throw`/`catch`.
pub fn do_render(
    mouse_x: f64,
    mouse_y: f64,
    menu: &dyn swap_util::MenuSeam,
    left_pos: i32,
    top_pos: i32,
    host: &dyn ScreenRenderHost,
    current_screen: &mut Option<u64>,
) -> Result<(), ()> {
    if !ConfigManager::get_config().get_toggle_mod() {
        return Ok(());
    }

    // if (handledScreen instanceof CreativeModeInventoryScreen) return;
    if menu.is_creative_mode_inventory_screen() {
        return Ok(());
    }

    if !host.player_menu_present() {
        return Ok(());
    }

    *state::CURRENT_STACKS.lock().unwrap() = host.player_menu_items();

    // try { lock(); ... } finally { unlock(); } — the Mutex is the lock.
    {
        let cursor_stack = host.player_menu_carried();
        let mut prev = state::CURRENT_CURSOR_STACK.lock().unwrap();
        let change = match prev.as_ref() {
            None => true,
            Some(prev_stack) => {
                prev_stack.get_count() != cursor_stack.get_count()
                    || !ItemStack::is_same_item(prev_stack, &cursor_stack)
            }
        };
        if change {
            *prev = Some(cursor_stack.copy());
        }
    }

    let screen = host.gui_screen_token();

    if state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst) {
        state::CLICK_SWAP.store(false, std::sync::atomic::Ordering::SeqCst);
        let current = state::CURRENT_STACKS.lock().unwrap().clone();
        swap_util::copy_stacks(&current, &mut state::OLD_STACKS.lock().unwrap());
        return Ok(());
    }

    if *current_screen != Some(screen) {
        state::SWAPS.lock().unwrap().clear();
        let current = state::CURRENT_STACKS.lock().unwrap().clone();
        swap_util::copy_stacks(&current, &mut state::OLD_STACKS.lock().unwrap());
        *current_screen = Some(screen);
        return Ok(());
    }

    let old_stacks = state::OLD_STACKS.lock().unwrap().clone();
    let current_stacks = state::CURRENT_STACKS.lock().unwrap().clone();
    let mut changed_stacks = get_changed_stacks(&old_stacks, &current_stacks)?;

    if !state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst) {
        let changed_stacks_size = changed_stacks.len();
        if changed_stacks_size > 1 {
            let mut more_stacks: Vec<super::super::SwapStacks> = Vec::new();
            let mut less_stacks: Vec<super::super::SwapStacks> = Vec::new();

            let mut total_amount = 0;
            for (slot_id, new_stack) in changed_stacks.iter() {
                let slot_id = *slot_id;
                let new_stack = new_stack;
                let old_stack = &old_stacks[slot_id as usize];
                // whether the stack got more items or less and if slot is output slot
                if swap_util::get_count(new_stack) > swap_util::get_count(old_stack)
                    && menu.get_slot(slot_id).allow_modification
                {
                    more_stacks.push(super::super::SwapStacks::new(
                        slot_id,
                        old_stack.copy(),
                        new_stack.copy(),
                        swap_util::get_count(new_stack) - swap_util::get_count(old_stack),
                    ));
                    total_amount +=
                        swap_util::get_count(new_stack) - swap_util::get_count(old_stack);
                } else if swap_util::get_count(new_stack) < swap_util::get_count(old_stack)
                    && menu.get_slot(slot_id).allow_modification
                    && state::CLICK_SWAP_STACK.lock().unwrap().is_none()
                {
                    less_stacks.push(super::super::SwapStacks::new(
                        slot_id,
                        old_stack.copy(),
                        new_stack.copy(),
                        swap_util::get_count(old_stack) - swap_util::get_count(new_stack),
                    ));
                }
            }
            if let Some(click_swap_stack) = state::CLICK_SWAP_STACK.lock().unwrap().take() {
                less_stacks.clear();
                let new_stack = menu.get_slot(click_swap_stack as i32).get_item().copy();
                let old_stack = old_stacks[click_swap_stack as usize].copy();
                less_stacks.push(super::super::SwapStacks::new(
                    click_swap_stack as i32,
                    old_stack,
                    new_stack,
                    total_amount,
                ));
            }

            if more_stacks.is_empty() {
                swap_util::assign_i2c_swaps(
                    &less_stacks,
                    &Vec2::with_xy(mouse_x - left_pos as f64, mouse_y - top_pos as f64),
                    menu,
                );
            } else {
                swap_util::assign_i2i_swaps(&mut more_stacks, &mut less_stacks, menu);
            }
        } else if changed_stacks_size == 1 {
            let current_cursor_stack = state::CURRENT_CURSOR_STACK.lock().unwrap().clone();
            let old_cursor_stack = state::OLD_CURSOR_STACK.lock().unwrap().clone();
            if let (Some(current_cursor_stack), Some(old_cursor_stack)) =
                (current_cursor_stack, old_cursor_stack)
            {
                if ItemStack::is_same_item(&current_cursor_stack, &old_cursor_stack)
                    && current_cursor_stack.get_count() != old_cursor_stack.get_count()
                {
                    // changedStacks.entrySet().stream().findFirst()
                    if let Some((changed_slot, _new_stack)) = changed_stacks.iter().next() {
                        let changed_slot = *changed_slot;
                        let old_stack = old_stacks[changed_slot as usize].copy();
                        let current_stack = current_stacks[changed_slot as usize].copy();
                        let cursor_stack_count_diff =
                            current_cursor_stack.get_count() - old_cursor_stack.get_count();
                        if (ItemStack::is_same_item(&old_stack, &current_stack)
                            && old_stack.get_count() - current_stack.get_count()
                                == cursor_stack_count_diff)
                            || ItemStack::is_same_item(
                                &current_stack,
                                &ItemStack::air_default_instance(),
                            )
                        {
                            let less_amount = swap_util::get_count(&old_stack)
                                - swap_util::get_count(&current_stack);
                            let less_stack = super::super::SwapStacks::new(
                                changed_slot, old_stack, current_stack, less_amount,
                            );
                            swap_util::assign_i2c_swaps(
                                std::slice::from_ref(&less_stack),
                                &Vec2::with_xy(mouse_x - left_pos as f64, mouse_y - top_pos as f64),
                                menu,
                            );
                        }
                    }
                }
            }
        }
    }

    if !are_stacks_equal(&old_stacks, &current_stacks) {
        swap_util::copy_stacks(&current_stacks, &mut state::OLD_STACKS.lock().unwrap());
        *state::OLD_CURSOR_STACK.lock().unwrap() = state::CURRENT_CURSOR_STACK.lock().unwrap().clone();
    }
    Ok(())
}

/// `@Unique smooth_Swapping$getChangedStacks(...)`. `NonNullList.get`
/// out-of-range throws upstream; that becomes the Err path the injection's
/// catch turns into `SwapUtil.reset()`.
pub fn get_changed_stacks(
    old_stacks: &[ItemStack],
    new_stacks: &[ItemStack],
) -> Result<HashMap<i32, ItemStack>, ()> {
    let mut changed_stacks = HashMap::new();
    for slot_id in 0..old_stacks.len() {
        let new_stack = new_stacks.get(slot_id).ok_or(())?;
        let old_stack = &old_stacks[slot_id];
        if !ItemStack::matches(old_stack, new_stack) {
            changed_stacks.insert(slot_id as i32, new_stack.copy());
        }
    }
    Ok(changed_stacks)
}

/// `@Unique smooth_Swapping$areStacksEqual(...)`. Java's null checks map
/// to the slice parameters (never null in the port).
pub fn are_stacks_equal(old_stacks: &[ItemStack], new_stacks: &[ItemStack]) -> bool {
    if old_stacks.len() != new_stacks.len() {
        return false;
    }
    for slot_id in 0..old_stacks.len() {
        if !ItemStack::matches(&new_stacks[slot_id], &old_stacks[slot_id]) {
            return false;
        }
    }
    true
}
}

pub mod abstract_recipe_book_screen {
//! Upstream `mixin/AbstractRecipeBookScreenMixin.java`, 1:1. Same injection
//! shape as the container-screen mixin minus the
//! `CreativeModeInventoryScreen` early return; the two static helpers are
//! duplicated in both upstream classes and shared here once.

use super::super::config::ConfigManager;
use super::super::item::ItemStack;
use super::super::state;
use super::super::swap_util;
use super::super::vec2::Vec2;
use super::abstract_container_screen::{are_stacks_equal, get_changed_stacks, ScreenRenderHost};

/// `@Inject(method = "extractRenderState", at = @At("HEAD"))
/// onRender(...)`; `current_screen` is this mixin class's own
/// `@Unique smooth_Swapping$currentScreen` field.
pub fn on_render(
    mouse_x: f64,
    mouse_y: f64,
    menu: &dyn swap_util::MenuSeam,
    left_pos: i32,
    top_pos: i32,
    host: &dyn ScreenRenderHost,
    current_screen: &mut Option<u64>,
) {
    if do_render(mouse_x, mouse_y, menu, left_pos, top_pos, host, current_screen).is_err() {
        swap_util::reset();
    }
}

/// The injection body, kept whole.
pub fn do_render(
    mouse_x: f64,
    mouse_y: f64,
    menu: &dyn swap_util::MenuSeam,
    left_pos: i32,
    top_pos: i32,
    host: &dyn ScreenRenderHost,
    current_screen: &mut Option<u64>,
) -> Result<(), ()> {
    if !ConfigManager::get_config().get_toggle_mod() {
        return Ok(());
    }

    if !host.player_menu_present() {
        return Ok(());
    }

    *state::CURRENT_STACKS.lock().unwrap() = host.player_menu_items();

    {
        let cursor_stack = host.player_menu_carried();
        let mut prev = state::CURRENT_CURSOR_STACK.lock().unwrap();
        let change = match prev.as_ref() {
            None => true,
            Some(prev_stack) => {
                prev_stack.get_count() != cursor_stack.get_count()
                    || !ItemStack::is_same_item(prev_stack, &cursor_stack)
            }
        };
        if change {
            *prev = Some(cursor_stack.copy());
        }
    }

    let screen = host.gui_screen_token();

    if state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst) {
        state::CLICK_SWAP.store(false, std::sync::atomic::Ordering::SeqCst);
        let current = state::CURRENT_STACKS.lock().unwrap().clone();
        swap_util::copy_stacks(&current, &mut state::OLD_STACKS.lock().unwrap());
        return Ok(());
    }

    if *current_screen != Some(screen) {
        state::SWAPS.lock().unwrap().clear();
        let current = state::CURRENT_STACKS.lock().unwrap().clone();
        swap_util::copy_stacks(&current, &mut state::OLD_STACKS.lock().unwrap());
        *current_screen = Some(screen);
        return Ok(());
    }

    let old_stacks = state::OLD_STACKS.lock().unwrap().clone();
    let current_stacks = state::CURRENT_STACKS.lock().unwrap().clone();
    let mut changed_stacks = get_changed_stacks(&old_stacks, &current_stacks)?;

    if !state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst) {
        let changed_stacks_size = changed_stacks.len();
        if changed_stacks_size > 1 {
            let mut more_stacks: Vec<super::super::SwapStacks> = Vec::new();
            let mut less_stacks: Vec<super::super::SwapStacks> = Vec::new();

            let mut total_amount = 0;
            for (slot_id, new_stack) in changed_stacks.iter() {
                let slot_id = *slot_id;
                let old_stack = &old_stacks[slot_id as usize];
                // whether the stack got more items or less and if slot is output slot
                if swap_util::get_count(new_stack) > swap_util::get_count(old_stack)
                    && menu.get_slot(slot_id).allow_modification
                {
                    more_stacks.push(super::super::SwapStacks::new(
                        slot_id,
                        old_stack.copy(),
                        new_stack.copy(),
                        swap_util::get_count(new_stack) - swap_util::get_count(old_stack),
                    ));
                    total_amount +=
                        swap_util::get_count(new_stack) - swap_util::get_count(old_stack);
                } else if swap_util::get_count(new_stack) < swap_util::get_count(old_stack)
                    && menu.get_slot(slot_id).allow_modification
                    && state::CLICK_SWAP_STACK.lock().unwrap().is_none()
                {
                    less_stacks.push(super::super::SwapStacks::new(
                        slot_id,
                        old_stack.copy(),
                        new_stack.copy(),
                        swap_util::get_count(old_stack) - swap_util::get_count(new_stack),
                    ));
                }
            }
            if let Some(click_swap_stack) = state::CLICK_SWAP_STACK.lock().unwrap().take() {
                less_stacks.clear();
                let new_stack = menu.get_slot(click_swap_stack as i32).get_item().copy();
                let old_stack = old_stacks[click_swap_stack as usize].copy();
                less_stacks.push(super::super::SwapStacks::new(
                    click_swap_stack as i32,
                    old_stack,
                    new_stack,
                    total_amount,
                ));
            }

            if more_stacks.is_empty() {
                swap_util::assign_i2c_swaps(
                    &less_stacks,
                    &Vec2::with_xy(mouse_x - left_pos as f64, mouse_y - top_pos as f64),
                    menu,
                );
            } else {
                swap_util::assign_i2i_swaps(&mut more_stacks, &mut less_stacks, menu);
            }
        } else if changed_stacks_size == 1 {
            let current_cursor_stack = state::CURRENT_CURSOR_STACK.lock().unwrap().clone();
            let old_cursor_stack = state::OLD_CURSOR_STACK.lock().unwrap().clone();
            if let (Some(current_cursor_stack), Some(old_cursor_stack)) =
                (current_cursor_stack, old_cursor_stack)
            {
                if ItemStack::is_same_item(&current_cursor_stack, &old_cursor_stack)
                    && current_cursor_stack.get_count() != old_cursor_stack.get_count()
                {
                    if let Some((changed_slot, _new_stack)) = changed_stacks.iter().next() {
                        let changed_slot = *changed_slot;
                        let old_stack = old_stacks[changed_slot as usize].copy();
                        let current_stack = current_stacks[changed_slot as usize].copy();
                        let cursor_stack_count_diff =
                            current_cursor_stack.get_count() - old_cursor_stack.get_count();
                        if (ItemStack::is_same_item(&old_stack, &current_stack)
                            && old_stack.get_count() - current_stack.get_count()
                                == cursor_stack_count_diff)
                            || ItemStack::is_same_item(
                                &current_stack,
                                &ItemStack::air_default_instance(),
                            )
                        {
                            let less_amount = swap_util::get_count(&old_stack)
                                - swap_util::get_count(&current_stack);
                            let less_stack = super::super::SwapStacks::new(
                                changed_slot, old_stack, current_stack, less_amount,
                            );
                            swap_util::assign_i2c_swaps(
                                std::slice::from_ref(&less_stack),
                                &Vec2::with_xy(mouse_x - left_pos as f64, mouse_y - top_pos as f64),
                                menu,
                            );
                        }
                    }
                }
            }
        }
    }

    if !are_stacks_equal(&old_stacks, &current_stacks) {
        swap_util::copy_stacks(&current_stacks, &mut state::OLD_STACKS.lock().unwrap());
        *state::OLD_CURSOR_STACK.lock().unwrap() = state::CURRENT_CURSOR_STACK.lock().unwrap().clone();
    }
    Ok(())
}
}

pub mod gui_graphics_extractor {
//! Upstream `mixin/GuiGraphicsExtractorMixin.java`, 1:1. The `item(...)` and
//! `itemDecorations(...)` `@At("HEAD")` cancellable injections become
//! `on_item_draw` / `on_draw_item_in_slot`; `Minecraft`, the pose stack and
//! the player's menu reach the handlers through the `GraphicsHost` seam.

use super::super::config::ConfigManager;
use super::super::item::ItemStack;
use super::super::state;
use super::super::swap_util;
use super::super::swaps::AnySwap;
use std::sync::atomic::{AtomicBool, Ordering};

/// `private static boolean smooth_Swapping$isRendering`.
static IS_RENDERING: AtomicBool = AtomicBool::new(false);

/// `Minecraft`, the pose stack and the player's menu seams the handlers read.
pub trait GraphicsHost {
    /// `minecraft.gui.screen() instanceof ConfigScreen`.
    fn is_config_screen(&self) -> bool;
    /// `minecraft.getDeltaTracker().getGameTimeDeltaTicks()`.
    fn get_game_time_delta_ticks(&self) -> f32;
    /// `pose.pushMatrix()`.
    fn push_matrix(&self);
    /// `pose.translate(float, float)`.
    fn translate(&self, x: f32, y: f32);
    /// `pose.popMatrix()`.
    fn pop_matrix(&self);
    /// `pose.m20()` (the hotbar x-offset read).
    fn m20(&self) -> f32;
    /// `item(ItemStack, int, int)`.
    fn item(&self, stack: &ItemStack, x: i32, y: i32);
    /// `itemDecorations(Font, ItemStack, int, int, String)` — the `Font`
    /// argument is the host's `minecraft.font`.
    fn item_decorations(&self, stack: &ItemStack, x: i32, y: i32, count_override: Option<&str>);
    /// `minecraft.player != null && player.containerMenu != null`.
    fn handler_present(&self) -> bool;
    /// `handler.getItems()`.
    fn handler_items(&self) -> Vec<ItemStack>;
    /// `handler.getCarried()`.
    fn handler_carried(&self) -> ItemStack;
}

/// `@Inject(method = "item(Lnet/minecraft/world/entity/LivingEntity;
/// Lnet/minecraft/world/item/ItemStack;IIIIF)V", at = @At("HEAD"),
/// cancellable = true) onItemDraw(...)`. Returns whether the original
/// render is cancelled (`cbi.cancel()`).
pub fn on_item_draw(stack: &ItemStack, x: i32, y: i32, host: &dyn GraphicsHost) -> bool {
    if IS_RENDERING.load(Ordering::SeqCst) {
        return false;
    }
    IS_RENDERING.store(true, Ordering::SeqCst);
    let result = do_item_draw(stack, x, y, host);
    IS_RENDERING.store(false, Ordering::SeqCst);
    match result {
        Ok(cancelled) => cancelled,
        Err(()) => {
            swap_util::reset();
            false
        }
    }
}

fn do_item_draw(stack: &ItemStack, x: i32, y: i32, host: &dyn GraphicsHost) -> Result<bool, ()> {
    // if (smooth_Swapping$isHotbar() && !(minecraft.gui.screen() instanceof ConfigScreen)) return;
    if is_hotbar(host) && !host.is_config_screen() {
        return Ok(false);
    }
    // if (((ItemStackAccessor) stack).isSwapStack()) return;
    if stack.is_swap_stack() {
        return Ok(false);
    }
    do_swap(stack, x, y, host)
}

/// The injection body, kept whole. `Result<(), ()>` carries the upstream
/// `throw`/`catch`.
fn do_swap(stack: &ItemStack, x: i32, y: i32, host: &dyn GraphicsHost) -> Result<bool, ()> {
    let index = swap_util::get_slot_index(stack);
    let mut swaps = state::SWAPS.lock().unwrap();

    if let Some(swap_list) = swaps.get_mut(&index) {
        let mut render_destination_slot = true;

        // render all swaps for one slot (Java indexes with `for (int i...)`
        // and removes the arrived swap inside the loop, which skips the next
        // element — `i += 1` after `remove` keeps that behavior).
        let mut i = 0usize;
        while i < swap_list.len() {
            let checked = swap_list[i].base().checked;
            swap_list[i].set_render_destination_slot(checked);
            if !swap_list[i].base().render_destination_slot() {
                render_destination_slot = false;
            }
            let swap_item = swap_list[i].base().get_swap_item().copy();
            render_swap(swap_list[i].base_mut(), x, y, &swap_item, host)?;
            if swap_util::has_arrived(swap_list[i].base()) {
                swap_util::set_render_to_true(swap_list);
                swap_list.remove(i);
            }
            i += 1;
        }

        // whether the destination slot should be rendered
        if render_destination_slot {
            host.item(stack, x, y);
        }
        if swap_list.is_empty() {
            swaps.remove(&index);
        }
        return Ok(true);
    } else if let Some(cursor_list) = swaps.get(&state::ASSUME_CURSOR_STACK_SLOT_INDEX) {
        // swapList.stream().filter(swap -> !((ItemToCursorInventorySwap) swap).isArrived())
        //     .collect(Collectors.toList()) — upstream's cast would CCE on
        // anything else in this bucket; the bucket only receives i2c swaps.
        let filtered: Vec<usize> = cursor_list
            .iter()
            .enumerate()
            .filter(|(_, swap)| matches!(swap, AnySwap::ItemToCursor(s) if !s.arrived))
            .map(|(i, _)| i)
            .collect();

        if !filtered.is_empty() {
            // swapListIndexOf(swapList, swap -> swap.getCopiedStackHash() == stack.hashCode()) == -1
            let already_present = filtered.iter().any(|&i| match &cursor_list[i] {
                AnySwap::ItemToCursor(s) => s.copied_stack_hash == stack.hash_code(),
                _ => false,
            });
            if !already_present {
                // LocalPlayer player = Minecraft.getInstance().player;
                // AbstractContainerMenu handler = player != null ? player.containerMenu : null;
                // (getCarried()/getItems() read once per loop upstream; the
                // values cannot change inside one pass)
                if host.handler_present() {
                    let inventory_stacks = host.handler_items();
                    let carried = host.handler_carried();
                    let cursor = swaps
                        .get_mut(&state::ASSUME_CURSOR_STACK_SLOT_INDEX)
                        .unwrap();
                    for &i in &filtered {
                        let swap = match &mut cursor[i] {
                            AnySwap::ItemToCursor(s) => s,
                            // filtered above only keeps i2c entries
                            _ => continue,
                        };
                        if !swap.started_render {
                            swap.started_render = true;
                        } else if !swap.arrived {
                            // if (swap.getTargetStackHash() == -1 || handler.getCarried().hashCode() == swap.getTargetStackHash())
                            if swap.target_stack_hash == -1
                                || carried.hash_code() == swap.target_stack_hash
                            {
                                // if (!inventoryStacks.contains(stack))
                                if !inventory_stacks
                                    .iter()
                                    .any(|s| ItemStack::matches(s, stack))
                                {
                                    let copied_stack = swap.base.get_swap_item().copy();
                                    swap.copied_stack_hash = copied_stack.hash_code();
                                    if swap.target_stack_hash == -1 {
                                        swap.target_stack_hash = stack.hash_code();
                                    }
                                    render_swap(&mut swap.base, x, y, &copied_stack, host)?;
                                    if swap_util::has_arrived(&swap.base) {
                                        swap.arrived = true;
                                    }
                                }
                            } else {
                                // cursor stack has changed, enforcing to arrive
                                swap.arrived = true;
                            }
                        }
                    }
                }
            }
        }

        // if (swapList.stream().allMatch(swap ->
        //     ((ItemToCursorInventorySwap) swap).isArrived()))
        //     SmoothSwapping.swaps.remove(ASSUME_CURSOR_STACK_SLOT_INDEX);
        // — allMatch over the filtered non-arrived list holds exactly when
        // it is empty.
        if filtered.is_empty() {
            swaps.remove(&state::ASSUME_CURSOR_STACK_SLOT_INDEX);
        }
    }
    Ok(false)
}

/// `@Unique smooth_Swapping$renderSwap(InventorySwap, int, int, ItemStack)`.
fn render_swap(
    swap: &mut super::super::swaps::InventorySwap,
    x: i32,
    y: i32,
    copied_stack: &ItemStack,
    host: &dyn GraphicsHost,
) -> Result<(), ()> {
    let last_frame_duration = host.get_game_time_delta_ticks();
    let config = ConfigManager::get_config();

    let swap_x = swap.get_x();
    let swap_y = swap.get_y();
    let angle = swap.get_angle();

    // double progress = 1D - SwapUtil.map(Math.hypot(swapX, swapY), 0, swap.getDistance(), 1D, 0D);
    let progress =
        1.0 - swap_util::map(swap_x.hypot(swap_y), 0.0, swap.get_distance(), 1.0, 0.0);

    let splines = config.get_splines();
    let ease = super::super::catmull_rom::get_progress(progress, &splines);

    let render_x = -swap.get_start_x() - angle.cos() * swap.get_distance() * ease;
    let render_y = swap.get_start_y() + angle.sin() * swap.get_distance() * ease;

    host.push_matrix();
    host.translate(render_x as f32, -render_y as f32);

    host.item(copied_stack, x, y);

    let speed = swap.get_distance() / 10.0 * config.get_animation_speed_formatted() as f64;

    // swap.setX(swapX + lastFrameDuration * speed * Math.cos(angle)) — the
    // float lastFrameDuration widens to double; same evaluation order.
    swap.x = swap_x + last_frame_duration as f64 * speed * angle.cos();
    swap.y = swap_y + last_frame_duration as f64 * speed * angle.sin();
    host.pop_matrix();
    Ok(())
}

/// `@Unique smooth_Swapping$isHotbar()` — `Math.round(pose.m20()) <= 0`
/// (`Math.round(x)` is `floor(x + 0.5)`).
fn is_hotbar(host: &dyn GraphicsHost) -> bool {
    let x_offset = host.m20();
    (x_offset + 0.5).floor() as i32 <= 0
}

/// `@Inject(method = "itemDecorations(Lnet/minecraft/client/font/TextRenderer;
/// Lnet/minecraft/world/item/ItemStack;IIILjava/lang/String;)V",
/// at = @At("HEAD"), cancellable = true) onDrawItemInSlot(...)`.
pub fn on_draw_item_in_slot(
    stack: &ItemStack,
    x: i32,
    y: i32,
    _count_override: Option<&str>,
    host: &dyn GraphicsHost,
) -> bool {
    // if (smooth_Swapping$isHotbar() && !(minecraft.gui.screen() instanceof ConfigScreen)) return;
    if is_hotbar(host) && !host.is_config_screen() {
        return false;
    }
    // if (((ItemStackAccessor) stack).isSwapStack()) return;
    if stack.is_swap_stack() {
        return false;
    }
    match do_overlay_render(stack, x, y, host) {
        Ok(cancelled) => cancelled,
        Err(()) => {
            swap_util::reset();
            false
        }
    }
}

/// The injection body, kept whole.
fn do_overlay_render(
    stack: &ItemStack,
    x: i32,
    y: i32,
    host: &dyn GraphicsHost,
) -> Result<bool, ()> {
    let index = swap_util::get_slot_index(stack);
    let mut swaps = state::SWAPS.lock().unwrap();

    if let Some(swap_list) = swaps.get(&index) {
        let swap_list = swap_list.clone();
        let config = ConfigManager::get_config();

        let mut stack_count = stack.get_count();
        let mut render_to_slot = true;

        for swap_any in &swap_list {
            let swap = swap_any.base();
            // if (!ItemStack.isSameItem(stack, swap.getSwapItem()))
            if !ItemStack::is_same_item(stack, swap.get_swap_item()) {
                swaps.remove(&index);
                return Ok(false);
            }
            // int stackCount = stack.getCount();
            stack_count -= swap.get_amount();
            if !swap.render_destination_slot() {
                render_to_slot = false;
            }

            // if (swap.getAmount() > 1 || stack.isBarVisible())
            if swap.get_amount() > 1 || stack.is_bar_visible() {
                // (the same math as renderSwap, duplicated upstream)
                let swap_x = swap.get_x();
                let swap_y = swap.get_y();
                let angle = swap.get_angle();
                let progress =
                    1.0 - swap_util::map(swap_x.hypot(swap_y), 0.0, swap.get_distance(), 1.0, 0.0);
                let splines = config.get_splines();
                let ease = super::super::catmull_rom::get_progress(progress, &splines);
                let render_x = -swap.get_start_x() - angle.cos() * swap.get_distance() * ease;
                let render_y = swap.get_start_y() + angle.sin() * swap.get_distance() * ease;

                host.push_matrix();
                host.translate(render_x as f32, -render_y as f32);

                if stack.is_bar_visible() {
                    host.item_decorations(&stack.copy(), x, y, None);
                } else {
                    host.item_decorations(&stack.copy(), x, y, Some(swap.get_amount().to_string().as_str()));
                }
                host.pop_matrix();
            }
        }

        // whether the destination slot should be rendered
        if render_to_slot && stack_count > 1 {
            host.item_decorations(&stack.copy(), x, y, Some(stack_count.to_string().as_str()));
        }
        return Ok(true);
    }
    Ok(false)
}
}

}

pub mod fabric {
//! Upstream `fabric/src/main/java/dev/shwg/smoothswapping/fabric/SmoothSwappingFabric.java`,
//! 1:1. `ClientModInitializer` has no Fabric surface on Horizon: the carrier's
//! client-init hook calls `on_initialize_client()`.

/// `onInitializeClient() { SmoothSwapping.init(); }`.
pub fn on_initialize_client() {
    super::state::init();
}

}

#[cfg(test)]
pub mod tests {
//! Port tests for the smooth-swapping core batch: Vec2 math (including
//! the upstream `=-` negation quirk), the swaps trio geometry, and
//! SwapStacks.
use std::sync::Mutex;

use super::item::ItemStack;
use super::slot::Slot;
use super::swaps::inventory_swap::InventorySwap;
use super::swaps::item_to_cursor_inventory_swap::ItemToCursorInventorySwap;
use super::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap;
use super::vec2::{java_format_g, Vec2};

fn stack(id: u32, count: i32) -> ItemStack {
    ItemStack::new(id, count)
}

#[test]
fn vec2_constructors() {
    let a = Vec2::new();
    assert_eq!(a.v, [0.0, 0.0]);
    let b = Vec2::with_xy(1.5, -2.0);
    assert_eq!(b.v, [1.5, -2.0]);
    assert_eq!(b.length(), 2);
    // Copy constructor.
    let c = Vec2::from_src(&b);
    assert_eq!(c.v, [1.5, -2.0]);
    // Angle constructor: theta in degrees.
    let d = Vec2::from_angle(90.0);
    assert!((d.v[0] - 0.0).abs() < 1e-12);
    assert!((d.v[1] - 1.0).abs() < 1e-12);
    assert!((Vec2::from_angle(45.0).v[0] - 2f64.sqrt() / 2.0).abs() < 1e-12);
}

#[test]
fn vec2_set_and_convert() {
    let a = Vec2::new().set(2.0, 3.0);
    // convert(): iv[0] = xoffset + x*72; iv[1] = yoffset - y*72.
    assert_eq!(a.convert(), [120 + 144, 120 - 216]);
}

#[test]
fn vec2_static_ops_and_magnitude() {
    let a = Vec2::with_xy(3.0, 4.0);
    assert!((a.magn() - 5.0).abs() < 1e-12);
    let b = Vec2::with_xy(1.0, 1.0);
    let s = Vec2::sum(&a, &b);
    assert_eq!(s.v, [4.0, 5.0]);
    let d = Vec2::diff_of(&a, &b);
    assert_eq!(d.v, [2.0, 3.0]);
    assert!((Vec2::distance(&a, &b) - (2.0f64 * 2.0 + 3.0 * 3.0).sqrt()).abs() < 1e-12);
}

#[test]
fn vec2_instance_diff_negates_upstream_quirk() {
    // Java: v[0] =- b.v[0] parses as v[0] = -b.v[0] (negation), kept 1:1.
    let mut a = Vec2::with_xy(7.0, 9.0);
    let b = Vec2::with_xy(2.0, 3.0);
    a.diff_negate(&b);
    assert_eq!(a.v, [-2.0, -3.0]);
}

#[test]
fn vec2_in_place_ops() {
    let mut a = Vec2::with_xy(4.0, 6.0);
    let b = Vec2::with_xy(1.0, 2.0);
    a.sum_in_place(&b);
    assert_eq!(a.v, [5.0, 8.0]);
    a.sum_scalar(1.0);
    assert_eq!(a.v, [6.0, 9.0]);
    a.diff_scalar(1.0);
    assert_eq!(a.v, [5.0, 8.0]);
    a.multiply_scalar(2.0);
    assert_eq!(a.v, [10.0, 16.0]);
    a.divide_scalar(2.0);
    assert_eq!(a.v, [5.0, 8.0]);
    a.translate(1.0, 1.0);
    assert_eq!(a.v, [6.0, 9.0]);
    a.translate_by(&b);
    assert_eq!(a.v, [7.0, 11.0]);
    a.perp();
    assert_eq!(a.v, [-11.0, 7.0]);
    let p = Vec2::perp_of(&Vec2::with_xy(1.0, 0.0));
    assert_eq!(p.v, [0.0, 1.0]);
}

#[test]
fn vec2_rotate_and_angle() {
    let mut a = Vec2::with_xy(1.0, 0.0);
    a.rotate(90.0);
    assert!((a.v[0] - 0.0).abs() < 1e-12);
    assert!((a.v[1] - 1.0).abs() < 1e-12);
    // angle() in degrees between the x axis and the y axis.
    let deg = Vec2::angle(&Vec2::with_xy(1.0, 0.0), &Vec2::with_xy(0.0, 1.0));
    assert!((deg - 90.0).abs() < 1e-9);
    assert!((Vec2::dot(&Vec2::with_xy(2.0, 3.0), &Vec2::with_xy(1.0, 4.0)) - 14.0).abs() < 1e-12);
    assert!((Vec2::perpdot(&Vec2::with_xy(1.0, 0.0), &Vec2::with_xy(0.0, 1.0)) - 1.0).abs() < 1e-12);
}

#[test]
fn vec2_set_size_and_scale() {
    let mut a = Vec2::with_xy(3.0, 4.0);
    a.set_size(10.0);
    assert!((a.magn() - 10.0).abs() < 1e-12);
    // Zero vector is left alone.
    let mut z = Vec2::new();
    z.set_size(5.0);
    assert_eq!(z.v, [0.0, 0.0]);
    let s = Vec2::with_xy(1.0, 1.0).scale(2.0);
    assert_eq!(s.v, [2.0, 2.0]);
    assert_eq!(s.copy().v, s.v);
}

#[test]
fn vec2_to_string_java_g() {
    assert_eq!(Vec2::with_xy(1.5, -2.25).to_display_string(), "[1.5 -2.25]");
    assert_eq!(Vec2::with_xy(0.0, 0.0).to_display_string(), "[0 0]");
    assert_eq!(java_format_g(123.456), "123");
    assert_eq!(java_format_g(-123.456), "-123");
    assert_eq!(java_format_g(0.000123), "0.000123");
    assert_eq!(java_format_g(1234.0), "1.23e+03");
    assert_eq!(java_format_g(7.0), "7");
}

#[test]
fn inventory_swap_geometry() {
    // ItemToItem swap from (0,0) to (3,4): dx=3, dy=4, distance 5,
    // angle atan2(4,3)+PI cast through f32.
    let from = Slot::new(0, 0);
    let to = Slot::new(3, 4);
    let swap = ItemToItemInventorySwap::new(&from, &to, true, 1, stack(7, 4));
    let b = &swap.base;
    assert_eq!(b.get_x(), 3.0);
    assert_eq!(b.get_y(), 4.0);
    assert_eq!(b.get_start_x(), 3.0);
    assert_eq!(b.get_start_y(), 4.0);
    assert_eq!(b.get_distance(), 5.0);
    let expect_angle = (4f64.atan2(3.0) + std::f64::consts::PI) as f32 as f64;
    assert!((b.get_angle() - expect_angle).abs() < 1e-12);
    assert!(!b.render_destination_slot());
    assert!(b.is_checked());
    assert_eq!(b.get_amount(), 1);
    assert_eq!(b.get_swap_item().item_id, 7);
    assert_eq!(b.get_swap_item().get_count(), 4);
}

#[test]
fn item_to_cursor_swap_center_offset_and_defaults() {
    let slot = Slot::new(10, 20);
    let mut swap = ItemToCursorInventorySwap::new(&slot, &Vec2::with_xy(30.0, 40.0), stack(2, 1), false, 3);
    let b = &swap.base;
    // fromVec is the slot center: (x+8, y+8).
    assert_eq!(b.get_x(), 30.0 - 18.0);
    assert_eq!(b.get_y(), 40.0 - 28.0);
    assert!((b.get_distance() - (12f64 * 12.0 + 12.0 * 12.0).sqrt()).abs() < 1e-12);
    // Defaults: startedRender=false, arrived=false, targetStackHash=-1.
    assert!(!swap.is_started_render());
    assert!(!swap.is_arrived());
    assert_eq!(swap.get_target_stack_hash(), -1);
    assert_eq!(swap.get_copied_stack_hash(), 0);
    swap.set_started_render(true);
    swap.set_arrived(true);
    swap.set_target_stack_hash(55);
    swap.set_copied_stack_hash(66);
    assert!(swap.is_started_render() && swap.is_arrived());
    assert_eq!(swap.get_target_stack_hash(), 55);
    assert_eq!(swap.get_copied_stack_hash(), 66);
    swap.base.set_render_destination_slot(true);
    assert!(swap.base.render_destination_slot());
}

#[test]
fn item_stack_seam() {
    let a = stack(3, 16);
    let b = stack(3, 1);
    assert!(ItemStack::is_same_item(&a, &b));
    assert!(!ItemStack::is_same_item(&a, &stack(4, 16)));
    assert_eq!(a.hash_code(), b.hash_code());
    assert_eq!(a.get_count(), 16);
    assert_eq!(a.copy().item_id, 3);
    // AIR default instance is the AIR item with count 1.
    assert!(ItemStack::is_same_item(&a, &ItemStack::air_default_instance()) == (3 == 0));
    assert!(ItemStack::is_same_item(&ItemStack::new(0, 5), &ItemStack::air_default_instance()));
    assert!(!a.to_string().is_empty());
}

#[test]
fn swap_stacks_fields() {
    let mut s = super::SwapStacks::new(4, stack(1, 8), stack(1, 3), -5);
    assert_eq!(s.get_slot_id(), 4);
    assert_eq!(s.get_old_stack().get_count(), 8);
    assert_eq!(s.get_new_stack().get_count(), 3);
    assert_eq!(s.item_count_to_change, -5);
    s.item_count_to_change += 1;
    assert_eq!(s.item_count_to_change, -4);
    assert!(s.to_display_string().contains("itemCountToChange=-4"));
}

// ---- SwapUtil batch ----

use super::state::{self, ASSUME_CURSOR_STACK_SLOT_INDEX};
use super::swap_util::{self, MenuSeam};
use super::SwapStacks;

// The smooth-swapping statics are process-wide like the Java statics, so
// the state-touching tests serialize on one guard.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

struct TestMenu {
    slots: Vec<Slot>,
    carried: ItemStack,
}

impl TestMenu {
    fn slot(&mut self, x: i32, y: i32, item: ItemStack) -> &mut Self {
        self.slots.push(Slot::with_item(x, y, item));
        self
    }
}

impl MenuSeam for TestMenu {
    fn slots_len(&self) -> usize { self.slots.len() }
    fn get_slot(&self, slot_id: i32) -> Slot {
        self.slots[slot_id as usize].clone()
    }
    fn get_carried(&self) -> ItemStack {
        self.carried.clone()
    }
}

#[test]
fn has_arrived_quadrants() {
    // angle = atan2(y, x) + PI (cast through f32), computed by
    // InventorySwap::new; getQuadrant = floor(2*angle/PI) % 4.
    // Q0: to (-1,-1) -> angle = pi/4.
    let mut swap = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(-1.0, -1.0), stack(1, 1), false, 1);
    // Upstream casts the angle through f32; compare at f32 precision.
    assert!((swap.get_angle() - ((-1f64).atan2(-1.0) + std::f64::consts::PI)).abs() < 1e-6);
    swap.x = 1.0;
    swap.y = 1.0;
    assert!(swap_util::has_arrived(&swap));
    // Same quadrant, wrong side.
    swap.x = -1.0;
    swap.y = -1.0;
    assert!(!swap_util::has_arrived(&swap));
    // Q1: to (1,-1) -> angle = 3pi/4; arrives at x<0, y>0.
    let mut q1 = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(1.0, -1.0), stack(1, 1), false, 1);
    q1.x = -1.0;
    q1.y = 1.0;
    assert!(swap_util::has_arrived(&q1));
    // Q2: to (1,1) -> angle = 5pi/4; arrives at x<0, y<0.
    let mut q2 = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(1.0, 1.0), stack(1, 1), false, 1);
    q2.x = -1.0;
    q2.y = -1.0;
    assert!(swap_util::has_arrived(&q2));
    q2.x = 1.0;
    q2.y = -1.0;
    assert!(!swap_util::has_arrived(&q2));
    // Q3: to (-1,1) -> angle = 7pi/4; arrives at x>0, y<0.
    let mut q3 = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(-1.0, 1.0), stack(1, 1), false, 1);
    q3.x = 1.0;
    q3.y = -1.0;
    assert!(swap_util::has_arrived(&q3));
}

fn map_keeps_upstream_parameter_order_quirk() {
    // map(in, inMin, inMax, outMax, outMin) uses outMax where the name
    // says outMin; both call shapes are pinned.
    assert_eq!(swap_util::map(0.25, 0.0, 1.0, 1.0, 0.0), 0.25);
    assert_eq!(swap_util::map(0.25, 0.0, 1.0, 0.0, 1.0), 0.75);
}

#[test]
fn get_count_and_copy_stacks() {
    assert_eq!(swap_util::get_count(&ItemStack::air_default_instance()), 0);
    assert_eq!(swap_util::get_count(&stack(5, 3)), 3);

    let src = vec![stack(1, 2), stack(2, 1)];
    let mut dst = vec![stack(9, 9)];
    swap_util::copy_stacks(&src, &mut dst);
    assert_eq!(dst.len(), 2);
    assert_eq!(dst[0], stack(1, 2));
    assert_eq!(dst[1], stack(2, 1));
}

#[test]
fn add_i2i_inventory_swap_gates_air_and_flags_swap_stack() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    let mut menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    menu.slot(10, 10, stack(3, 1));
    menu.slot(20, 20, stack(3, 5));
    menu.slot(30, 30, ItemStack::air_default_instance());

    swap_util::add_i2i_inventory_swap(0, &menu.slots[0], &menu.slots[1], false, 2);
    // Destination slot empty: no swap recorded.
    swap_util::add_i2i_inventory_swap(0, &menu.slots[0], &menu.slots[2], false, 2);

    let (map_len, list_len, swap_stack, amount) = {
        let swaps = state::SWAPS.lock().unwrap();
        let list = swaps.get(&0);
        (swaps.len(), list.map(|l| l.len()).unwrap_or(0), list.map(|l| l[0].base().get_swap_item().is_swap_stack()).unwrap_or(false), list.map(|l| l[0].base().get_amount()).unwrap_or(-1))
    };
    assert_eq!(map_len, 1);
    assert_eq!(list_len, 1);
    assert!(swap_stack);
    assert_eq!(amount, 2);
}

#[test]
fn assign_i2c_swaps_and_slot_index() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    state::CURRENT_STACKS.lock().unwrap().extend([stack(1, 1), stack(2, 7)]);
    assert_eq!(swap_util::get_slot_index(&stack(2, 7)), 1);
    assert_eq!(swap_util::get_slot_index(&stack(9, 1)), -1);

    let less = vec![SwapStacks::new(1, stack(2, 7), stack(2, 4), 3)];
    let mut menu = TestMenu { slots: vec![Slot::new(0, 0), Slot::new(8, 8)], carried: stack(2, 1) };
    swap_util::assign_i2c_swaps(&less, &super::vec2::Vec2::with_xy(100.0, 100.0), &menu);

    let probe = {
        let swaps = state::SWAPS.lock().unwrap();
        match swaps.get(&ASSUME_CURSOR_STACK_SLOT_INDEX) {
            Some(list) if list.len() == 1 => match &list[0] {
                super::swaps::AnySwap::ItemToCursor(s) => Some((s.base.get_amount(), s.base.get_start_x(), s.base.get_start_y(), s.is_arrived())),
                _ => None,
            },
            _ => None,
        }
    };
    let (amount, start_x, start_y, arrived) = probe.expect("one i2c swap in the cursor bucket");
    assert_eq!(amount, 3);
    assert_eq!(start_x, 100.0 - (8.0 + 8.0));
    assert_eq!(start_y, 100.0 - (8.0 + 8.0));
    assert!(!arrived);

    // Cursor empty: nothing is assigned.
    state::reset_vars_for_tests();
    menu.carried = ItemStack::air_default_instance();
    swap_util::assign_i2c_swaps(&less, &super::vec2::Vec2::with_xy(100.0, 100.0), &menu);
    assert!(state::SWAPS.lock().unwrap().is_empty());
}

#[test]
fn assign_i2i_swaps_balances_and_removes_exhausted_less_stacks() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    let mut menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    // slots 0,1 = less sources; slot 2 = more destination.
    menu.slot(0, 0, stack(3, 1));
    menu.slot(10, 10, stack(3, 1));
    menu.slot(20, 20, stack(3, 1));

    let mut more = vec![SwapStacks::new(2, stack(3, 1), stack(3, 4), -3)];
    let mut less = vec![
        SwapStacks::new(0, stack(3, 1), stack(3, 1), 2),
        SwapStacks::new(1, stack(3, 1), stack(3, 1), 1),
    ];
    swap_util::assign_i2i_swaps(&mut more, &mut less, &menu);

    // The more-stack is balanced to zero and stays in its list.
    assert_eq!(more.len(), 1);
    assert_eq!(more[0].item_count_to_change, 0);
    // Both less stacks drained to zero and removed (Java removes each
    // exhausted element inside the loop).
    assert!(less.is_empty());

    let probe = {
        let swaps = state::SWAPS.lock().unwrap();
        swaps.get(&2).map(|list| (list.len(), list[0].base().get_amount(), list[1].base().get_amount(), list[0].base().get_swap_item().is_swap_stack()))
    };
    let (len, a0, a1, is_swap) = probe.expect("two swaps in slot 2 bucket");
    assert_eq!(len, 2);
    assert_eq!(a0, 2);
    assert_eq!(a1, 1);
    assert!(is_swap);
}

#[test]
fn swap_list_index_and_set_render_to_true() {
    let mut list = vec![
        super::swaps::AnySwap::ItemToItem(super::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap::new(
            &Slot::new(0, 0), &Slot::new(8, 8), false, 1, stack(1, 1))),
        super::swaps::AnySwap::ItemToItem(super::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap::new(
            &Slot::new(1, 1), &Slot::new(9, 9), true, 3, stack(1, 1))),
    ];
    let idx = swap_util::swap_list_index_of(&list, &|s| s.base().is_checked());
    assert_eq!(idx, 1);
    swap_util::set_render_to_true(&mut list);
    assert!(list[0].base().render_destination_slot());
    assert!(list[1].base().render_destination_slot());
}

#[test]
fn config_roundtrip_and_curve_padding() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    let dir = std::env::temp_dir().join(format!("ssw-cfg-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    super::config::ConfigManager::set_config_dir_for_tests(dir.clone());

    let config = super::config::ConfigManager::initialize_config();
    assert!(config.get_toggle_mod());
    assert_eq!(config.get_animation_speed(), 100);
    assert_eq!(config.get_animation_speed_formatted(), 1.0);

    {
    let mut config = super::config::ConfigManager::get_config();
    config.set_toggle_mod(false);
    config.set_animation_speed(150);
    // 6 points -> 2 stored floats rows (inner points only).
    let points = [
        Vec2::with_xy(0.0, 0.0), Vec2::with_xy(0.0, 0.0),
        Vec2::with_xy(0.3, 0.4), Vec2::with_xy(0.7, 0.6),
        Vec2::with_xy(1.0, 1.0), Vec2::with_xy(1.0, 1.0),
    ];
    config.set_curve_points(&points);
    }
    super::config::ConfigManager::save();

    super::config::ConfigManager::reload_for_tests();
    let read = super::config::ConfigManager::initialize_config();
    assert!(!read.get_toggle_mod());
    assert_eq!(read.get_animation_speed(), 150);
    assert_eq!(read.curve_points_len_for_tests(), 2);

    let curve = read.get_curve_points();
    assert_eq!(curve.len(), 6);
    assert_eq!(curve[0].v, [0.0, 0.0]);
    assert_eq!(curve[2].v[0] as f32, 0.3f32);
    assert_eq!(curve[2].v[1] as f32, 0.4f32);
    assert_eq!(curve[4].v, [1.0, 1.0]);
    assert_eq!(read.get_splines().len(), 3);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn splines_progress_endpoints() {
    let points = [
        Vec2::with_xy(0.0, 0.0), Vec2::with_xy(0.0, 0.0),
        Vec2::with_xy(0.5, 0.5), Vec2::with_xy(1.0, 1.0),
        Vec2::with_xy(1.0, 1.0), Vec2::with_xy(1.0, 1.0),
    ];
    let splines = super::catmull_rom::splines_from_points(&points);
    assert_eq!(splines.len(), 3);
    // Segment x/oldX carry the p2/p1 x values.
    assert_eq!(splines[0].old_x, 0.0);
    assert_eq!(splines[0].x, 0.5);
    // t inside the first segment maps through it; t outside falls back to
    // segments.get(0).
    let y = super::catmull_rom::get_progress(0.25, &splines);
    assert!(y > 0.0 && y < 1.0);
    let fallback = super::catmull_rom::get_progress(5.0, &splines);
    assert!(!fallback.is_nan());
}

// ================= mixins batch (upstream mixin/, 1:1) =================

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use super::mixins::abstract_container_screen as acs;
use super::mixins::gui_graphics_extractor::{self, GraphicsHost};
use super::mixins::item_stack::ItemStackAccessor;
use super::mixins::simple_container_accessor::SimpleContainerAccessor;
use super::mixins::serverbound_container_click_packet::{self, ClickPacketHost, ContainerInput, HashedStack};
use super::swaps::AnySwap;

struct TestPacketHost {
    slots: Vec<Slot>,
    player: bool,
    container_screen: bool,
}

impl ClickPacketHost for TestPacketHost {
    fn gui_screen_is_container_screen(&self) -> bool {
        self.container_screen
    }
    fn player_present(&self) -> bool {
        self.player
    }
    fn screen_handler_slots_len(&self) -> usize {
        self.slots.len()
    }
    fn screen_handler_get_slot(&self, slot_id: i32) -> Slot {
        self.slots[slot_id as usize].clone()
    }
}

struct TestRenderHost {
    token: u64,
    present: bool,
    items: Vec<ItemStack>,
    carried: ItemStack,
}

impl acs::ScreenRenderHost for TestRenderHost {
    fn gui_screen_token(&self) -> u64 {
        self.token
    }
    fn player_menu_present(&self) -> bool {
        self.present
    }
    fn player_menu_items(&self) -> Vec<ItemStack> {
        self.items.clone()
    }
    fn player_menu_carried(&self) -> ItemStack {
        self.carried.copy()
    }
}

struct TestGraphicsHost {
    delta: f32,
    item_calls: Rc<RefCell<Vec<(i32, i32, i32)>>>,
    deco_calls: Rc<RefCell<Vec<Option<String>>>>,
}

impl GraphicsHost for TestGraphicsHost {
    fn is_config_screen(&self) -> bool {
        false
    }
    fn get_game_time_delta_ticks(&self) -> f32 {
        self.delta
    }
    fn push_matrix(&self) {}
    fn translate(&self, _x: f32, _y: f32) {}
    fn pop_matrix(&self) {}
    // round(1.5) = 1 > 0: not the hotbar
    fn m20(&self) -> f32 {
        1.0
    }
    fn item(&self, stack: &ItemStack, x: i32, y: i32) {
        self.item_calls.borrow_mut().push((stack.get_count(), x, y));
    }
    fn item_decorations(&self, _stack: &ItemStack, _x: i32, _y: i32, count_override: Option<&str>) {
        self.deco_calls.borrow_mut().push(count_override.map(|s| s.to_string()));
    }
    fn handler_present(&self) -> bool {
        false
    }
    fn handler_items(&self) -> Vec<ItemStack> {
        vec![]
    }
    fn handler_carried(&self) -> ItemStack {
        ItemStack::air_default_instance()
    }
}

#[test]
fn item_stack_accessor_set_and_read() {
    let _g = serial_guard();
    let mut s = stack(3, 7);
    assert!(!s.smooth_swapping_is_swap_stack());
    s.smooth_swapping_set_is_swap_stack(true);
    assert!(s.smooth_swapping_is_swap_stack());
}

struct TestContainer {
    stacks: Vec<ItemStack>,
}

impl super::mixins::simple_container_accessor::SimpleContainerAccessor for TestContainer {
    fn get_stacks(&self) -> Vec<ItemStack> {
        self.stacks.clone()
    }
}

#[test]
fn simple_container_get_stacks() {
    let c = TestContainer { stacks: vec![stack(1, 1), stack(2, 2)] };
    assert_eq!(c.get_stacks().len(), 2);
}

#[test]
fn click_packet_removes_pending_swap() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    // seed a pending i2c swap at slot 5; the <init> TAIL removes it
    let from = Slot::new(0, 0);
    let swap = ItemToCursorInventorySwap::new(&from, &Vec2::with_xy(10.0, 10.0), stack(1, 1), false, 1);
    state::SWAPS.lock().unwrap().insert(5, vec![AnySwap::ItemToCursor(swap)]);
    let host = TestPacketHost { slots: vec![Slot::new(0, 0)], player: true, container_screen: false };
    let modified: HashMap<i32, HashedStack> = HashMap::new();
    serverbound_container_click_packet::on_init(0, 0, 5, 0, &ContainerInput::Pickup, &modified, &HashedStack::Empty, &host);
    assert!(state::SWAPS.lock().unwrap().is_empty());
}

#[test]
fn click_packet_swap_branch_adds_i2i() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    let mut modified: HashMap<i32, HashedStack> = HashMap::new();
    modified.insert(0, HashedStack::ActualItem { item_id: 3, count: 2 });
    modified.insert(2, HashedStack::Empty);
    let slots = vec![
        Slot::with_item(0, 0, stack(3, 2)),
        Slot::new(4, 4),
        Slot::with_item(8, 8, stack(5, 7)),
    ];
    let host = TestPacketHost { slots, player: true, container_screen: true };
    serverbound_container_click_packet::on_init(0, 0, 0, 1, &ContainerInput::Swap, &modified, &HashedStack::Empty, &host);
    // SWAP input sets the copy-on-next-render flag
    assert!(state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst));
    // both directions recorded: destination slot 2 (7 items) and mouse slot 0 (2 items)
    let swaps = state::SWAPS.lock().unwrap();
    assert_eq!(swaps.get(&2).unwrap().len(), 1);
    assert_eq!(swaps.get(&2).unwrap()[0].base().get_amount(), 7);
    assert_eq!(swaps.get(&0).unwrap().len(), 1);
    assert_eq!(swaps.get(&0).unwrap()[0].base().get_amount(), 2);
}

#[test]
fn click_packet_quick_move_sets_click_swap_stack() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::OLD_STACKS.lock().unwrap() = vec![stack(3, 5)];
    let mut modified: HashMap<i32, HashedStack> = HashMap::new();
    modified.insert(0, HashedStack::ActualItem { item_id: 3, count: 2 });
    modified.insert(1, HashedStack::Empty);
    let mut slots = vec![Slot::with_item(0, 0, stack(3, 2)), Slot::new(4, 4)];
    slots[0].allow_modification = false;
    let host = TestPacketHost { slots, player: true, container_screen: true };
    serverbound_container_click_packet::on_init(0, 0, 0, 1, &ContainerInput::QuickMove, &modified, &HashedStack::Empty, &host);
    // new count 2 <= old count 5 (crafting-table-output gate)
    assert_eq!(*state::CLICK_SWAP_STACK.lock().unwrap(), Some(0));
}

#[test]
fn screen_render_single_changed_assigns_i2c() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::OLD_STACKS.lock().unwrap() = vec![stack(3, 5)];
    *state::OLD_CURSOR_STACK.lock().unwrap() = Some(stack(3, 1));
    *state::CURRENT_CURSOR_STACK.lock().unwrap() = Some(stack(3, 1));
    let menu = TestMenu { slots: vec![Slot::new(10, 10)], carried: stack(3, 4) };
    let host = TestRenderHost { token: 7, present: true, items: vec![stack(3, 2)], carried: stack(3, 4) };
    let mut current_screen: Option<u64> = Some(7);
    acs::on_render(50.0, 50.0, &menu, 10, 10, &host, &mut current_screen);
    // cursor gain (+3) matches the slot loss (5 -> 2): one i2c swap, keyed
    // in the assume-cursor bucket like upstream's assignI2CSwaps
    let swaps = state::SWAPS.lock().unwrap();
    assert_eq!(swaps.len(), 1);
    assert_eq!(swaps.get(&state::ASSUME_CURSOR_STACK_SLOT_INDEX).unwrap().len(), 1);
}

#[test]
fn screen_render_click_swap_copies_only() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::OLD_STACKS.lock().unwrap() = vec![stack(1, 1)];
    state::CLICK_SWAP.store(true, std::sync::atomic::Ordering::SeqCst);
    let menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    let host = TestRenderHost { token: 7, present: true, items: vec![stack(1, 9)], carried: stack(1, 9) };
    let mut current_screen: Option<u64> = Some(7);
    acs::on_render(0.0, 0.0, &menu, 0, 0, &host, &mut current_screen);
    assert!(!state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(*state::OLD_STACKS.lock().unwrap(), vec![stack(1, 9)]);
    assert!(state::SWAPS.lock().unwrap().is_empty());
}

#[test]
fn screen_render_screen_change_clears_swaps() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    state::SWAPS.lock().unwrap().insert(3, vec![]);
    let menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    let host = TestRenderHost { token: 9, present: true, items: vec![stack(1, 1)], carried: ItemStack::air_default_instance() };
    let mut current_screen: Option<u64> = None;
    acs::on_render(0.0, 0.0, &menu, 0, 0, &host, &mut current_screen);
    assert_eq!(current_screen, Some(9));
    assert!(state::SWAPS.lock().unwrap().is_empty());
    assert_eq!(*state::OLD_STACKS.lock().unwrap(), vec![stack(1, 1)]);
}

#[test]
fn graphics_render_progresses_and_arrives() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::CURRENT_STACKS.lock().unwrap() = vec![stack(9, 3)];
    let from = Slot::new(0, 0);
    let swap = ItemToCursorInventorySwap::new(&from, &Vec2::with_xy(-1.0, -1.0), stack(9, 3), false, 1);
    state::SWAPS.lock().unwrap().insert(0, vec![AnySwap::ItemToCursor(swap)]);
    let item_calls = Rc::new(RefCell::new(vec![]));
    let deco_calls = Rc::new(RefCell::new(vec![]));
    let host = TestGraphicsHost { delta: 1.0, item_calls: item_calls.clone(), deco_calls: deco_calls.clone() };
    let stack = stack(9, 3);
    // first call renders (i2c render_destination_slot is false: the slot's
    // own item stays hidden) and advances the swap toward the cursor.
    // The swap starts at the full start->cursor delta (-9, -9) and each
    // tick moves distance/10 * speed in the arrival direction (no clamp):
    // one tick moves ~0.9 per axis, so |x| shrinks below 9.
    assert!(gui_graphics_extractor::on_item_draw(&stack, 5, 5, &host));
    let mid_x = state::SWAPS.lock().unwrap().get(&0).unwrap()[0].base().get_x();
    assert!(mid_x.abs() < 9.0);
    // keep ticking until it arrives and is removed from the map
    for _ in 0..30 {
        gui_graphics_extractor::on_item_draw(&stack, 5, 5, &host);
    }
    assert!(state::SWAPS.lock().unwrap().get(&0).is_none());
    // once the swap is gone, later draws fall through to the plain item()
    // path again, so the slot item renders normally
    assert!(!item_calls.borrow().is_empty());
}

#[test]
fn graphics_overlay_splits_counts() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::CURRENT_STACKS.lock().unwrap() = vec![stack(4, 5)];
    let from = Slot::with_item(0, 0, stack(4, 5));
    let to = Slot::with_item(8, 8, stack(4, 3));
    super::swap_util::add_i2i_inventory_swap(0, &from, &to, false, 3);
    state::SWAPS.lock().unwrap().get_mut(&0).unwrap()[0].set_render_destination_slot(true);
    let item_calls = Rc::new(RefCell::new(vec![]));
    let deco_calls = Rc::new(RefCell::new(vec![]));
    let host = TestGraphicsHost { delta: 1.0, item_calls, deco_calls: deco_calls.clone() };
    let stack = stack(4, 5);
    assert!(gui_graphics_extractor::on_draw_item_in_slot(&stack, 5, 5, None, &host));
    // the moving 3 renders its own count; 5 - 3 stays at the slot
    let calls = deco_calls.borrow();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].as_deref(), Some("3"));
    assert_eq!(calls[1].as_deref(), Some("2"));
}
}

