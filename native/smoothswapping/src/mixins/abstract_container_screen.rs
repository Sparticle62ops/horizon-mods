//! Upstream `mixin/AbstractContainerScreenMixin.java`, 1:1. The
//! `extractRenderState` `@At("HEAD")` injection becomes `on_render`; the
//! screen's `menu`/`leftPos`/`topPos` arrive as parameters and `Minecraft`
//! reaches the handler through the `ScreenRenderHost` seam.

use crate::config::ConfigManager;
use crate::item::ItemStack;
use crate::state;
use crate::swap_util;
use crate::vec2::Vec2;
use std::collections::HashMap;

/// `Minecraft`/player seams the render pass reads.
pub trait ScreenRenderHost {
    /// `Minecraft.getInstance().gui.screen()` — identity token for the
    /// screen reference comparison.
    fn gui_screen_token(&self) -> u64;
    /// `client.player == null || client.player.containerMenu == null`.
    fn player_menu_present(&self) -> bool;
    /// `client.player.containerMenu.getItems()`.
    fn player_menu_items(&self) -> Vec<ItemStack>;
    /// `client.player.containerMenu.getCarried()`.
    fn player_menu_carried(&self) -> ItemStack;
}

/// `@Inject(method = "extractRenderState", at = @At("HEAD"))
/// onRender(...)`. The `catch (Exception e) { SwapUtil.reset(); }` becomes
/// the Err path; `current_screen` is the `@Unique smooth_Swapping$currentScreen`
/// field the caller carries for this mixin instance.
pub fn on_render(
    mouse_x: f64,
    mouse_y: f64,
    menu: &dyn swap_util::MenuSeam,
    left_pos: i32,
    top_pos: i32,
    host: &dyn ScreenRenderHost,
    current_screen: &mut Option<u64>,
) {
    if do_render(mouse_x, mouse_y, menu, left_pos, top_pos, host, current_screen).is_err() {
        swap_util::reset();
    }
}

/// The injection body, kept whole. `Result<(), ()>` carries the upstream
/// `throw`/`catch`.
pub fn do_render(
    mouse_x: f64,
    mouse_y: f64,
    menu: &dyn swap_util::MenuSeam,
    left_pos: i32,
    top_pos: i32,
    host: &dyn ScreenRenderHost,
    current_screen: &mut Option<u64>,
) -> Result<(), ()> {
    if !ConfigManager::get_config().get_toggle_mod() {
        return Ok(());
    }

    // if (handledScreen instanceof CreativeModeInventoryScreen) return;
    if menu.is_creative_mode_inventory_screen() {
        return Ok(());
    }

    if !host.player_menu_present() {
        return Ok(());
    }

    *state::CURRENT_STACKS.lock().unwrap() = host.player_menu_items();

    // try { lock(); ... } finally { unlock(); } — the Mutex is the lock.
    {
        let cursor_stack = host.player_menu_carried();
        let mut prev = state::CURRENT_CURSOR_STACK.lock().unwrap();
        let change = match prev.as_ref() {
            None => true,
            Some(prev_stack) => {
                prev_stack.get_count() != cursor_stack.get_count()
                    || !ItemStack::is_same_item(prev_stack, &cursor_stack)
            }
        };
        if change {
            *prev = Some(cursor_stack.copy());
        }
    }

    let screen = host.gui_screen_token();

    if state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst) {
        state::CLICK_SWAP.store(false, std::sync::atomic::Ordering::SeqCst);
        let current = state::CURRENT_STACKS.lock().unwrap().clone();
        swap_util::copy_stacks(&current, &mut state::OLD_STACKS.lock().unwrap());
        return Ok(());
    }

    if *current_screen != Some(screen) {
        state::SWAPS.lock().unwrap().clear();
        let current = state::CURRENT_STACKS.lock().unwrap().clone();
        swap_util::copy_stacks(&current, &mut state::OLD_STACKS.lock().unwrap());
        *current_screen = Some(screen);
        return Ok(());
    }

    let old_stacks = state::OLD_STACKS.lock().unwrap().clone();
    let current_stacks = state::CURRENT_STACKS.lock().unwrap().clone();
    let mut changed_stacks = get_changed_stacks(&old_stacks, &current_stacks)?;

    if !state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst) {
        let changed_stacks_size = changed_stacks.len();
        if changed_stacks_size > 1 {
            let mut more_stacks: Vec<crate::SwapStacks> = Vec::new();
            let mut less_stacks: Vec<crate::SwapStacks> = Vec::new();

            let mut total_amount = 0;
            for (slot_id, new_stack) in changed_stacks.iter() {
                let slot_id = *slot_id;
                let new_stack = new_stack;
                let old_stack = &old_stacks[slot_id as usize];
                // whether the stack got more items or less and if slot is output slot
                if swap_util::get_count(new_stack) > swap_util::get_count(old_stack)
                    && menu.get_slot(slot_id).allow_modification
                {
                    more_stacks.push(crate::SwapStacks::new(
                        slot_id,
                        old_stack.copy(),
                        new_stack.copy(),
                        swap_util::get_count(new_stack) - swap_util::get_count(old_stack),
                    ));
                    total_amount +=
                        swap_util::get_count(new_stack) - swap_util::get_count(old_stack);
                } else if swap_util::get_count(new_stack) < swap_util::get_count(old_stack)
                    && menu.get_slot(slot_id).allow_modification
                    && state::CLICK_SWAP_STACK.lock().unwrap().is_none()
                {
                    less_stacks.push(crate::SwapStacks::new(
                        slot_id,
                        old_stack.copy(),
                        new_stack.copy(),
                        swap_util::get_count(old_stack) - swap_util::get_count(new_stack),
                    ));
                }
            }
            if let Some(click_swap_stack) = state::CLICK_SWAP_STACK.lock().unwrap().take() {
                less_stacks.clear();
                let new_stack = menu.get_slot(click_swap_stack as i32).get_item().copy();
                let old_stack = old_stacks[click_swap_stack as usize].copy();
                less_stacks.push(crate::SwapStacks::new(
                    click_swap_stack as i32,
                    old_stack,
                    new_stack,
                    total_amount,
                ));
            }

            if more_stacks.is_empty() {
                swap_util::assign_i2c_swaps(
                    &less_stacks,
                    &Vec2::with_xy(mouse_x - left_pos as f64, mouse_y - top_pos as f64),
                    menu,
                );
            } else {
                swap_util::assign_i2i_swaps(&mut more_stacks, &mut less_stacks, menu);
            }
        } else if changed_stacks_size == 1 {
            let current_cursor_stack = state::CURRENT_CURSOR_STACK.lock().unwrap().clone();
            let old_cursor_stack = state::OLD_CURSOR_STACK.lock().unwrap().clone();
            if let (Some(current_cursor_stack), Some(old_cursor_stack)) =
                (current_cursor_stack, old_cursor_stack)
            {
                if ItemStack::is_same_item(&current_cursor_stack, &old_cursor_stack)
                    && current_cursor_stack.get_count() != old_cursor_stack.get_count()
                {
                    // changedStacks.entrySet().stream().findFirst()
                    if let Some((changed_slot, _new_stack)) = changed_stacks.iter().next() {
                        let changed_slot = *changed_slot;
                        let old_stack = old_stacks[changed_slot as usize].copy();
                        let current_stack = current_stacks[changed_slot as usize].copy();
                        let cursor_stack_count_diff =
                            current_cursor_stack.get_count() - old_cursor_stack.get_count();
                        if (ItemStack::is_same_item(&old_stack, &current_stack)
                            && old_stack.get_count() - current_stack.get_count()
                                == cursor_stack_count_diff)
                            || ItemStack::is_same_item(
                                &current_stack,
                                &ItemStack::air_default_instance(),
                            )
                        {
                            let less_amount = swap_util::get_count(&old_stack)
                                - swap_util::get_count(&current_stack);
                            let less_stack = crate::SwapStacks::new(
                                changed_slot, old_stack, current_stack, less_amount,
                            );
                            swap_util::assign_i2c_swaps(
                                std::slice::from_ref(&less_stack),
                                &Vec2::with_xy(mouse_x - left_pos as f64, mouse_y - top_pos as f64),
                                menu,
                            );
                        }
                    }
                }
            }
        }
    }

    if !are_stacks_equal(&old_stacks, &current_stacks) {
        swap_util::copy_stacks(&current_stacks, &mut state::OLD_STACKS.lock().unwrap());
        *state::OLD_CURSOR_STACK.lock().unwrap() = state::CURRENT_CURSOR_STACK.lock().unwrap().clone();
    }
    Ok(())
}

/// `@Unique smooth_Swapping$getChangedStacks(...)`. `NonNullList.get`
/// out-of-range throws upstream; that becomes the Err path the injection's
/// catch turns into `SwapUtil.reset()`.
pub fn get_changed_stacks(
    old_stacks: &[ItemStack],
    new_stacks: &[ItemStack],
) -> Result<HashMap<i32, ItemStack>, ()> {
    let mut changed_stacks = HashMap::new();
    for slot_id in 0..old_stacks.len() {
        let new_stack = new_stacks.get(slot_id).ok_or(())?;
        let old_stack = &old_stacks[slot_id];
        if !ItemStack::matches(old_stack, new_stack) {
            changed_stacks.insert(slot_id as i32, new_stack.copy());
        }
    }
    Ok(changed_stacks)
}

/// `@Unique smooth_Swapping$areStacksEqual(...)`. Java's null checks map
/// to the slice parameters (never null in the port).
pub fn are_stacks_equal(old_stacks: &[ItemStack], new_stacks: &[ItemStack]) -> bool {
    if old_stacks.len() != new_stacks.len() {
        return false;
    }
    for slot_id in 0..old_stacks.len() {
        if !ItemStack::matches(&new_stacks[slot_id], &old_stacks[slot_id]) {
            return false;
        }
    }
    true
}
