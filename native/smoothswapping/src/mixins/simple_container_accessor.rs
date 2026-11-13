//! Upstream `mixin/SimpleContainerAccessor.java`, 1:1.

use crate::item::ItemStack;

/// `net.minecraft.world.SimpleContainer` access via `@Accessor("items")`.
pub trait SimpleContainerAccessor {
    /// `NonNullList<ItemStack> getStacks()`.
    fn get_stacks(&self) -> Vec<ItemStack>;
}
