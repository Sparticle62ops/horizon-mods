//! `net.minecraft.world.inventory.Slot` seam: the public `x`/`y` screen
//! positions the swap constructors read and `getItem()` for SwapUtil. One
//! slot per id. Identity (equals/hashCode) stays on the slot position,
//! like the game's own slot identity.

use crate::item::ItemStack;

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
