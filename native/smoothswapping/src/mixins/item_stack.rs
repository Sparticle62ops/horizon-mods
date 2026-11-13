//! Upstream `mixin/ItemStackMixin.java`, 1:1. The `@Unique isSwapStack`
//! field is injected into `ItemStack` upstream; the port carries it on the
//! ItemStack seam, and this module is the `ItemStackAccessor` interface
//! these accessors implement.

use crate::item::ItemStack;

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
