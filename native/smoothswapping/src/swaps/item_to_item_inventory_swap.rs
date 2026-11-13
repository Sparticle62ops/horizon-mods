//! Upstream `swaps/ItemToItemInventorySwap.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use crate::item::ItemStack;
use crate::slot::Slot;
use crate::swaps::inventory_swap::InventorySwap;
use crate::vec2::Vec2;

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
