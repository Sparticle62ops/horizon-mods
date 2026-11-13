//! Smooth Swapping (Horizon Rust port), 1:1.
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
//! Upstream: https://github.com/Schauweg/Smooth-Swapping, branch 26.3/dev
//! @88e2988 (LGPL-3.0-only). Ported 1:1 per the port policy; credit
//! Schauweg. Architectury common + fabric sources; the neoforge loader
//! shim is not ported (Horizon is the loader surface).
//!
//! Core/common package first: Vec2, SwapStacks, the swaps trio, and the
//! SmoothSwapping object state; SwapUtil and the config screen, then the
//! mixins, follow in later batches.

pub mod catmull_rom;
pub mod config;
pub mod item;
pub mod mixins;
pub mod slot;
pub mod state;
pub mod swap_util;
pub mod swaps;
pub mod vec2;

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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

