pub mod inventory_swap;
pub mod item_to_cursor_inventory_swap;
pub mod item_to_item_inventory_swap;

pub use crate::swaps::inventory_swap::InventorySwap;

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
