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
