//! Upstream `mixin/GuiGraphicsExtractorMixin.java`, 1:1. The `item(...)` and
//! `itemDecorations(...)` `@At("HEAD")` cancellable injections become
//! `on_item_draw` / `on_draw_item_in_slot`; `Minecraft`, the pose stack and
//! the player's menu reach the handlers through the `GraphicsHost` seam.

use crate::config::ConfigManager;
use crate::item::ItemStack;
use crate::state;
use crate::swap_util;
use crate::swaps::AnySwap;
use std::sync::atomic::{AtomicBool, Ordering};

/// `private static boolean smooth_Swapping$isRendering`.
static IS_RENDERING: AtomicBool = AtomicBool::new(false);

/// `Minecraft`, the pose stack and the player's menu seams the handlers read.
pub trait GraphicsHost {
    /// `minecraft.gui.screen() instanceof ConfigScreen`.
    fn is_config_screen(&self) -> bool;
    /// `minecraft.getDeltaTracker().getGameTimeDeltaTicks()`.
    fn get_game_time_delta_ticks(&self) -> f32;
    /// `pose.pushMatrix()`.
    fn push_matrix(&self);
    /// `pose.translate(float, float)`.
    fn translate(&self, x: f32, y: f32);
    /// `pose.popMatrix()`.
    fn pop_matrix(&self);
    /// `pose.m20()` (the hotbar x-offset read).
    fn m20(&self) -> f32;
    /// `item(ItemStack, int, int)`.
    fn item(&self, stack: &ItemStack, x: i32, y: i32);
    /// `itemDecorations(Font, ItemStack, int, int, String)` — the `Font`
    /// argument is the host's `minecraft.font`.
    fn item_decorations(&self, stack: &ItemStack, x: i32, y: i32, count_override: Option<&str>);
    /// `minecraft.player != null && player.containerMenu != null`.
    fn handler_present(&self) -> bool;
    /// `handler.getItems()`.
    fn handler_items(&self) -> Vec<ItemStack>;
    /// `handler.getCarried()`.
    fn handler_carried(&self) -> ItemStack;
}

/// `@Inject(method = "item(Lnet/minecraft/world/entity/LivingEntity;
/// Lnet/minecraft/world/item/ItemStack;IIIIF)V", at = @At("HEAD"),
/// cancellable = true) onItemDraw(...)`. Returns whether the original
/// render is cancelled (`cbi.cancel()`).
pub fn on_item_draw(stack: &ItemStack, x: i32, y: i32, host: &dyn GraphicsHost) -> bool {
    if IS_RENDERING.load(Ordering::SeqCst) {
        return false;
    }
    IS_RENDERING.store(true, Ordering::SeqCst);
    let result = do_item_draw(stack, x, y, host);
    IS_RENDERING.store(false, Ordering::SeqCst);
    match result {
        Ok(cancelled) => cancelled,
        Err(()) => {
            swap_util::reset();
            false
        }
    }
}

fn do_item_draw(stack: &ItemStack, x: i32, y: i32, host: &dyn GraphicsHost) -> Result<bool, ()> {
    // if (smooth_Swapping$isHotbar() && !(minecraft.gui.screen() instanceof ConfigScreen)) return;
    if is_hotbar(host) && !host.is_config_screen() {
        return Ok(false);
    }
    // if (((ItemStackAccessor) stack).isSwapStack()) return;
    if stack.is_swap_stack() {
        return Ok(false);
    }
    do_swap(stack, x, y, host)
}

/// The injection body, kept whole. `Result<(), ()>` carries the upstream
/// `throw`/`catch`.
fn do_swap(stack: &ItemStack, x: i32, y: i32, host: &dyn GraphicsHost) -> Result<bool, ()> {
    let index = swap_util::get_slot_index(stack);
    let mut swaps = state::SWAPS.lock().unwrap();

    if let Some(swap_list) = swaps.get_mut(&index) {
        let mut render_destination_slot = true;

        // render all swaps for one slot (Java indexes with `for (int i...)`
        // and removes the arrived swap inside the loop, which skips the next
        // element — `i += 1` after `remove` keeps that behavior).
        let mut i = 0usize;
        while i < swap_list.len() {
            let checked = swap_list[i].base().checked;
            swap_list[i].set_render_destination_slot(checked);
            if !swap_list[i].base().render_destination_slot() {
                render_destination_slot = false;
            }
            let swap_item = swap_list[i].base().get_swap_item().copy();
            render_swap(swap_list[i].base_mut(), x, y, &swap_item, host)?;
            if swap_util::has_arrived(swap_list[i].base()) {
                swap_util::set_render_to_true(swap_list);
                swap_list.remove(i);
            }
            i += 1;
        }

        // whether the destination slot should be rendered
        if render_destination_slot {
            host.item(stack, x, y);
        }
        if swap_list.is_empty() {
            swaps.remove(&index);
        }
        return Ok(true);
    } else if let Some(cursor_list) = swaps.get(&state::ASSUME_CURSOR_STACK_SLOT_INDEX) {
        // swapList.stream().filter(swap -> !((ItemToCursorInventorySwap) swap).isArrived())
        //     .collect(Collectors.toList()) — upstream's cast would CCE on
        // anything else in this bucket; the bucket only receives i2c swaps.
        let filtered: Vec<usize> = cursor_list
            .iter()
            .enumerate()
            .filter(|(_, swap)| matches!(swap, AnySwap::ItemToCursor(s) if !s.arrived))
            .map(|(i, _)| i)
            .collect();

        if !filtered.is_empty() {
            // swapListIndexOf(swapList, swap -> swap.getCopiedStackHash() == stack.hashCode()) == -1
            let already_present = filtered.iter().any(|&i| match &cursor_list[i] {
                AnySwap::ItemToCursor(s) => s.copied_stack_hash == stack.hash_code(),
                _ => false,
            });
            if !already_present {
                // LocalPlayer player = Minecraft.getInstance().player;
                // AbstractContainerMenu handler = player != null ? player.containerMenu : null;
                // (getCarried()/getItems() read once per loop upstream; the
                // values cannot change inside one pass)
                if host.handler_present() {
                    let inventory_stacks = host.handler_items();
                    let carried = host.handler_carried();
                    let cursor = swaps
                        .get_mut(&state::ASSUME_CURSOR_STACK_SLOT_INDEX)
                        .unwrap();
                    for &i in &filtered {
                        let swap = match &mut cursor[i] {
                            AnySwap::ItemToCursor(s) => s,
                            // filtered above only keeps i2c entries
                            _ => continue,
                        };
                        if !swap.started_render {
                            swap.started_render = true;
                        } else if !swap.arrived {
                            // if (swap.getTargetStackHash() == -1 || handler.getCarried().hashCode() == swap.getTargetStackHash())
                            if swap.target_stack_hash == -1
                                || carried.hash_code() == swap.target_stack_hash
                            {
                                // if (!inventoryStacks.contains(stack))
                                if !inventory_stacks
                                    .iter()
                                    .any(|s| ItemStack::matches(s, stack))
                                {
                                    let copied_stack = swap.base.get_swap_item().copy();
                                    swap.copied_stack_hash = copied_stack.hash_code();
                                    if swap.target_stack_hash == -1 {
                                        swap.target_stack_hash = stack.hash_code();
                                    }
                                    render_swap(&mut swap.base, x, y, &copied_stack, host)?;
                                    if swap_util::has_arrived(&swap.base) {
                                        swap.arrived = true;
                                    }
                                }
                            } else {
                                // cursor stack has changed, enforcing to arrive
                                swap.arrived = true;
                            }
                        }
                    }
                }
            }
        }

        // if (swapList.stream().allMatch(swap ->
        //     ((ItemToCursorInventorySwap) swap).isArrived()))
        //     SmoothSwapping.swaps.remove(ASSUME_CURSOR_STACK_SLOT_INDEX);
        // — allMatch over the filtered non-arrived list holds exactly when
        // it is empty.
        if filtered.is_empty() {
            swaps.remove(&state::ASSUME_CURSOR_STACK_SLOT_INDEX);
        }
    }
    Ok(false)
}

/// `@Unique smooth_Swapping$renderSwap(InventorySwap, int, int, ItemStack)`.
fn render_swap(
    swap: &mut crate::swaps::InventorySwap,
    x: i32,
    y: i32,
    copied_stack: &ItemStack,
    host: &dyn GraphicsHost,
) -> Result<(), ()> {
    let last_frame_duration = host.get_game_time_delta_ticks();
    let config = ConfigManager::get_config();

    let swap_x = swap.get_x();
    let swap_y = swap.get_y();
    let angle = swap.get_angle();

    // double progress = 1D - SwapUtil.map(Math.hypot(swapX, swapY), 0, swap.getDistance(), 1D, 0D);
    let progress =
        1.0 - swap_util::map(swap_x.hypot(swap_y), 0.0, swap.get_distance(), 1.0, 0.0);

    let splines = config.get_splines();
    let ease = crate::catmull_rom::get_progress(progress, &splines);

    let render_x = -swap.get_start_x() - angle.cos() * swap.get_distance() * ease;
    let render_y = swap.get_start_y() + angle.sin() * swap.get_distance() * ease;

    host.push_matrix();
    host.translate(render_x as f32, -render_y as f32);

    host.item(copied_stack, x, y);

    let speed = swap.get_distance() / 10.0 * config.get_animation_speed_formatted() as f64;

    // swap.setX(swapX + lastFrameDuration * speed * Math.cos(angle)) — the
    // float lastFrameDuration widens to double; same evaluation order.
    swap.x = swap_x + last_frame_duration as f64 * speed * angle.cos();
    swap.y = swap_y + last_frame_duration as f64 * speed * angle.sin();
    host.pop_matrix();
    Ok(())
}

/// `@Unique smooth_Swapping$isHotbar()` — `Math.round(pose.m20()) <= 0`
/// (`Math.round(x)` is `floor(x + 0.5)`).
fn is_hotbar(host: &dyn GraphicsHost) -> bool {
    let x_offset = host.m20();
    (x_offset + 0.5).floor() as i32 <= 0
}

/// `@Inject(method = "itemDecorations(Lnet/minecraft/client/font/TextRenderer;
/// Lnet/minecraft/world/item/ItemStack;IIILjava/lang/String;)V",
/// at = @At("HEAD"), cancellable = true) onDrawItemInSlot(...)`.
pub fn on_draw_item_in_slot(
    stack: &ItemStack,
    x: i32,
    y: i32,
    _count_override: Option<&str>,
    host: &dyn GraphicsHost,
) -> bool {
    // if (smooth_Swapping$isHotbar() && !(minecraft.gui.screen() instanceof ConfigScreen)) return;
    if is_hotbar(host) && !host.is_config_screen() {
        return false;
    }
    // if (((ItemStackAccessor) stack).isSwapStack()) return;
    if stack.is_swap_stack() {
        return false;
    }
    match do_overlay_render(stack, x, y, host) {
        Ok(cancelled) => cancelled,
        Err(()) => {
            swap_util::reset();
            false
        }
    }
}

/// The injection body, kept whole.
fn do_overlay_render(
    stack: &ItemStack,
    x: i32,
    y: i32,
    host: &dyn GraphicsHost,
) -> Result<bool, ()> {
    let index = swap_util::get_slot_index(stack);
    let mut swaps = state::SWAPS.lock().unwrap();

    if let Some(swap_list) = swaps.get(&index) {
        let swap_list = swap_list.clone();
        let config = ConfigManager::get_config();

        let mut stack_count = stack.get_count();
        let mut render_to_slot = true;

        for swap_any in &swap_list {
            let swap = swap_any.base();
            // if (!ItemStack.isSameItem(stack, swap.getSwapItem()))
            if !ItemStack::is_same_item(stack, swap.get_swap_item()) {
                swaps.remove(&index);
                return Ok(false);
            }
            // int stackCount = stack.getCount();
            stack_count -= swap.get_amount();
            if !swap.render_destination_slot() {
                render_to_slot = false;
            }

            // if (swap.getAmount() > 1 || stack.isBarVisible())
            if swap.get_amount() > 1 || stack.is_bar_visible() {
                // (the same math as renderSwap, duplicated upstream)
                let swap_x = swap.get_x();
                let swap_y = swap.get_y();
                let angle = swap.get_angle();
                let progress =
                    1.0 - swap_util::map(swap_x.hypot(swap_y), 0.0, swap.get_distance(), 1.0, 0.0);
                let splines = config.get_splines();
                let ease = crate::catmull_rom::get_progress(progress, &splines);
                let render_x = -swap.get_start_x() - angle.cos() * swap.get_distance() * ease;
                let render_y = swap.get_start_y() + angle.sin() * swap.get_distance() * ease;

                host.push_matrix();
                host.translate(render_x as f32, -render_y as f32);

                if stack.is_bar_visible() {
                    host.item_decorations(&stack.copy(), x, y, None);
                } else {
                    host.item_decorations(&stack.copy(), x, y, Some(swap.get_amount().to_string().as_str()));
                }
                host.pop_matrix();
            }
        }

        // whether the destination slot should be rendered
        if render_to_slot && stack_count > 1 {
            host.item_decorations(&stack.copy(), x, y, Some(stack_count.to_string().as_str()));
        }
        return Ok(true);
    }
    Ok(false)
}
