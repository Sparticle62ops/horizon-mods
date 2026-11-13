//! Upstream `mixin/ServerboundContainerClickPacketMixin.java`, 1:1. The
//! `<init>` `@At("TAIL")` injection becomes `on_init`, called where the
//! game builds the packet; `Minecraft` and the player's menu reach the
//! handler through the `ClickPacketHost` seam.

use crate::config::ConfigManager;
use crate::item::ItemStack;
use crate::slot::Slot;
use crate::state;
use crate::swap_util;
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
