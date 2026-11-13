//! Upstream `SwapUtil.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */
//!
//! `AbstractContainerMenu` reaches the port through the `MenuSeam` trait
//! (its only reads here are `getSlot(int)` and `getCarried()`).

use std::sync::atomic::Ordering;

use crate::item::ItemStack;
use crate::slot::Slot;
use crate::state::{self, ASSUME_CURSOR_STACK_SLOT_INDEX};
use crate::swaps::item_to_cursor_inventory_swap::ItemToCursorInventorySwap;
use crate::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap;
use crate::swaps::{AnySwap, InventorySwap};
use crate::vec2::Vec2;

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
pub fn assign_i2c_swaps(less_stacks: &[crate::SwapStacks], mouse_pos: &Vec2, handler: &dyn MenuSeam) {
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
pub fn assign_i2i_swaps(more_stacks: &mut Vec<crate::SwapStacks>, less_stacks: &mut Vec<crate::SwapStacks>, handler: &dyn MenuSeam) {
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
