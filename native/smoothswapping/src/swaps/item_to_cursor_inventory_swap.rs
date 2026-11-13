//! Upstream `swaps/ItemToCursorInventorySwap.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use crate::item::ItemStack;
use crate::slot::Slot;
use crate::swaps::inventory_swap::InventorySwap;
use crate::vec2::Vec2;

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
