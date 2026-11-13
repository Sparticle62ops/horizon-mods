//! Port tests for the smooth-swapping core batch: Vec2 math (including
//! the upstream `=-` negation quirk), the swaps trio geometry, and
//! SwapStacks.
use std::sync::Mutex;

use crate::item::ItemStack;
use crate::slot::Slot;
use crate::swaps::inventory_swap::InventorySwap;
use crate::swaps::item_to_cursor_inventory_swap::ItemToCursorInventorySwap;
use crate::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap;
use crate::vec2::{java_format_g, Vec2};

fn stack(id: u32, count: i32) -> ItemStack {
    ItemStack::new(id, count)
}

#[test]
fn vec2_constructors() {
    let a = Vec2::new();
    assert_eq!(a.v, [0.0, 0.0]);
    let b = Vec2::with_xy(1.5, -2.0);
    assert_eq!(b.v, [1.5, -2.0]);
    assert_eq!(b.length(), 2);
    // Copy constructor.
    let c = Vec2::from_src(&b);
    assert_eq!(c.v, [1.5, -2.0]);
    // Angle constructor: theta in degrees.
    let d = Vec2::from_angle(90.0);
    assert!((d.v[0] - 0.0).abs() < 1e-12);
    assert!((d.v[1] - 1.0).abs() < 1e-12);
    assert!((Vec2::from_angle(45.0).v[0] - 2f64.sqrt() / 2.0).abs() < 1e-12);
}

#[test]
fn vec2_set_and_convert() {
    let a = Vec2::new().set(2.0, 3.0);
    // convert(): iv[0] = xoffset + x*72; iv[1] = yoffset - y*72.
    assert_eq!(a.convert(), [120 + 144, 120 - 216]);
}

#[test]
fn vec2_static_ops_and_magnitude() {
    let a = Vec2::with_xy(3.0, 4.0);
    assert!((a.magn() - 5.0).abs() < 1e-12);
    let b = Vec2::with_xy(1.0, 1.0);
    let s = Vec2::sum(&a, &b);
    assert_eq!(s.v, [4.0, 5.0]);
    let d = Vec2::diff_of(&a, &b);
    assert_eq!(d.v, [2.0, 3.0]);
    assert!((Vec2::distance(&a, &b) - (2.0f64 * 2.0 + 3.0 * 3.0).sqrt()).abs() < 1e-12);
}

#[test]
fn vec2_instance_diff_negates_upstream_quirk() {
    // Java: v[0] =- b.v[0] parses as v[0] = -b.v[0] (negation), kept 1:1.
    let mut a = Vec2::with_xy(7.0, 9.0);
    let b = Vec2::with_xy(2.0, 3.0);
    a.diff_negate(&b);
    assert_eq!(a.v, [-2.0, -3.0]);
}

#[test]
fn vec2_in_place_ops() {
    let mut a = Vec2::with_xy(4.0, 6.0);
    let b = Vec2::with_xy(1.0, 2.0);
    a.sum_in_place(&b);
    assert_eq!(a.v, [5.0, 8.0]);
    a.sum_scalar(1.0);
    assert_eq!(a.v, [6.0, 9.0]);
    a.diff_scalar(1.0);
    assert_eq!(a.v, [5.0, 8.0]);
    a.multiply_scalar(2.0);
    assert_eq!(a.v, [10.0, 16.0]);
    a.divide_scalar(2.0);
    assert_eq!(a.v, [5.0, 8.0]);
    a.translate(1.0, 1.0);
    assert_eq!(a.v, [6.0, 9.0]);
    a.translate_by(&b);
    assert_eq!(a.v, [7.0, 11.0]);
    a.perp();
    assert_eq!(a.v, [-11.0, 7.0]);
    let p = Vec2::perp_of(&Vec2::with_xy(1.0, 0.0));
    assert_eq!(p.v, [0.0, 1.0]);
}

#[test]
fn vec2_rotate_and_angle() {
    let mut a = Vec2::with_xy(1.0, 0.0);
    a.rotate(90.0);
    assert!((a.v[0] - 0.0).abs() < 1e-12);
    assert!((a.v[1] - 1.0).abs() < 1e-12);
    // angle() in degrees between the x axis and the y axis.
    let deg = Vec2::angle(&Vec2::with_xy(1.0, 0.0), &Vec2::with_xy(0.0, 1.0));
    assert!((deg - 90.0).abs() < 1e-9);
    assert!((Vec2::dot(&Vec2::with_xy(2.0, 3.0), &Vec2::with_xy(1.0, 4.0)) - 14.0).abs() < 1e-12);
    assert!((Vec2::perpdot(&Vec2::with_xy(1.0, 0.0), &Vec2::with_xy(0.0, 1.0)) - 1.0).abs() < 1e-12);
}

#[test]
fn vec2_set_size_and_scale() {
    let mut a = Vec2::with_xy(3.0, 4.0);
    a.set_size(10.0);
    assert!((a.magn() - 10.0).abs() < 1e-12);
    // Zero vector is left alone.
    let mut z = Vec2::new();
    z.set_size(5.0);
    assert_eq!(z.v, [0.0, 0.0]);
    let s = Vec2::with_xy(1.0, 1.0).scale(2.0);
    assert_eq!(s.v, [2.0, 2.0]);
    assert_eq!(s.copy().v, s.v);
}

#[test]
fn vec2_to_string_java_g() {
    assert_eq!(Vec2::with_xy(1.5, -2.25).to_display_string(), "[1.5 -2.25]");
    assert_eq!(Vec2::with_xy(0.0, 0.0).to_display_string(), "[0 0]");
    assert_eq!(java_format_g(123.456), "123");
    assert_eq!(java_format_g(-123.456), "-123");
    assert_eq!(java_format_g(0.000123), "0.000123");
    assert_eq!(java_format_g(1234.0), "1.23e+03");
    assert_eq!(java_format_g(7.0), "7");
}

#[test]
fn inventory_swap_geometry() {
    // ItemToItem swap from (0,0) to (3,4): dx=3, dy=4, distance 5,
    // angle atan2(4,3)+PI cast through f32.
    let from = Slot::new(0, 0);
    let to = Slot::new(3, 4);
    let swap = ItemToItemInventorySwap::new(&from, &to, true, 1, stack(7, 4));
    let b = &swap.base;
    assert_eq!(b.get_x(), 3.0);
    assert_eq!(b.get_y(), 4.0);
    assert_eq!(b.get_start_x(), 3.0);
    assert_eq!(b.get_start_y(), 4.0);
    assert_eq!(b.get_distance(), 5.0);
    let expect_angle = (4f64.atan2(3.0) + std::f64::consts::PI) as f32 as f64;
    assert!((b.get_angle() - expect_angle).abs() < 1e-12);
    assert!(!b.render_destination_slot());
    assert!(b.is_checked());
    assert_eq!(b.get_amount(), 1);
    assert_eq!(b.get_swap_item().item_id, 7);
    assert_eq!(b.get_swap_item().get_count(), 4);
}

#[test]
fn item_to_cursor_swap_center_offset_and_defaults() {
    let slot = Slot::new(10, 20);
    let mut swap = ItemToCursorInventorySwap::new(&slot, &Vec2::with_xy(30.0, 40.0), stack(2, 1), false, 3);
    let b = &swap.base;
    // fromVec is the slot center: (x+8, y+8).
    assert_eq!(b.get_x(), 30.0 - 18.0);
    assert_eq!(b.get_y(), 40.0 - 28.0);
    assert!((b.get_distance() - (12f64 * 12.0 + 12.0 * 12.0).sqrt()).abs() < 1e-12);
    // Defaults: startedRender=false, arrived=false, targetStackHash=-1.
    assert!(!swap.is_started_render());
    assert!(!swap.is_arrived());
    assert_eq!(swap.get_target_stack_hash(), -1);
    assert_eq!(swap.get_copied_stack_hash(), 0);
    swap.set_started_render(true);
    swap.set_arrived(true);
    swap.set_target_stack_hash(55);
    swap.set_copied_stack_hash(66);
    assert!(swap.is_started_render() && swap.is_arrived());
    assert_eq!(swap.get_target_stack_hash(), 55);
    assert_eq!(swap.get_copied_stack_hash(), 66);
    swap.base.set_render_destination_slot(true);
    assert!(swap.base.render_destination_slot());
}

#[test]
fn item_stack_seam() {
    let a = stack(3, 16);
    let b = stack(3, 1);
    assert!(ItemStack::is_same_item(&a, &b));
    assert!(!ItemStack::is_same_item(&a, &stack(4, 16)));
    assert_eq!(a.hash_code(), b.hash_code());
    assert_eq!(a.get_count(), 16);
    assert_eq!(a.copy().item_id, 3);
    // AIR default instance is the AIR item with count 1.
    assert!(ItemStack::is_same_item(&a, &ItemStack::air_default_instance()) == (3 == 0));
    assert!(ItemStack::is_same_item(&ItemStack::new(0, 5), &ItemStack::air_default_instance()));
    assert!(!a.to_string().is_empty());
}

#[test]
fn swap_stacks_fields() {
    let mut s = crate::SwapStacks::new(4, stack(1, 8), stack(1, 3), -5);
    assert_eq!(s.get_slot_id(), 4);
    assert_eq!(s.get_old_stack().get_count(), 8);
    assert_eq!(s.get_new_stack().get_count(), 3);
    assert_eq!(s.item_count_to_change, -5);
    s.item_count_to_change += 1;
    assert_eq!(s.item_count_to_change, -4);
    assert!(s.to_display_string().contains("itemCountToChange=-4"));
}

// ---- SwapUtil batch ----

use crate::state::{self, ASSUME_CURSOR_STACK_SLOT_INDEX};
use crate::swap_util::{self, MenuSeam};
use crate::SwapStacks;

// The smooth-swapping statics are process-wide like the Java statics, so
// the state-touching tests serialize on one guard.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

struct TestMenu {
    slots: Vec<Slot>,
    carried: ItemStack,
}

impl TestMenu {
    fn slot(&mut self, x: i32, y: i32, item: ItemStack) -> &mut Self {
        self.slots.push(Slot::with_item(x, y, item));
        self
    }
}

impl MenuSeam for TestMenu {
    fn slots_len(&self) -> usize { self.slots.len() }
    fn get_slot(&self, slot_id: i32) -> Slot {
        self.slots[slot_id as usize].clone()
    }
    fn get_carried(&self) -> ItemStack {
        self.carried.clone()
    }
}

#[test]
fn has_arrived_quadrants() {
    // angle = atan2(y, x) + PI (cast through f32), computed by
    // InventorySwap::new; getQuadrant = floor(2*angle/PI) % 4.
    // Q0: to (-1,-1) -> angle = pi/4.
    let mut swap = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(-1.0, -1.0), stack(1, 1), false, 1);
    // Upstream casts the angle through f32; compare at f32 precision.
    assert!((swap.get_angle() - ((-1f64).atan2(-1.0) + std::f64::consts::PI)).abs() < 1e-6);
    swap.x = 1.0;
    swap.y = 1.0;
    assert!(swap_util::has_arrived(&swap));
    // Same quadrant, wrong side.
    swap.x = -1.0;
    swap.y = -1.0;
    assert!(!swap_util::has_arrived(&swap));
    // Q1: to (1,-1) -> angle = 3pi/4; arrives at x<0, y>0.
    let mut q1 = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(1.0, -1.0), stack(1, 1), false, 1);
    q1.x = -1.0;
    q1.y = 1.0;
    assert!(swap_util::has_arrived(&q1));
    // Q2: to (1,1) -> angle = 5pi/4; arrives at x<0, y<0.
    let mut q2 = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(1.0, 1.0), stack(1, 1), false, 1);
    q2.x = -1.0;
    q2.y = -1.0;
    assert!(swap_util::has_arrived(&q2));
    q2.x = 1.0;
    q2.y = -1.0;
    assert!(!swap_util::has_arrived(&q2));
    // Q3: to (-1,1) -> angle = 7pi/4; arrives at x>0, y<0.
    let mut q3 = InventorySwap::new(&Vec2::with_xy(0.0, 0.0), &Vec2::with_xy(-1.0, 1.0), stack(1, 1), false, 1);
    q3.x = 1.0;
    q3.y = -1.0;
    assert!(swap_util::has_arrived(&q3));
}

fn map_keeps_upstream_parameter_order_quirk() {
    // map(in, inMin, inMax, outMax, outMin) uses outMax where the name
    // says outMin; both call shapes are pinned.
    assert_eq!(swap_util::map(0.25, 0.0, 1.0, 1.0, 0.0), 0.25);
    assert_eq!(swap_util::map(0.25, 0.0, 1.0, 0.0, 1.0), 0.75);
}

#[test]
fn get_count_and_copy_stacks() {
    assert_eq!(swap_util::get_count(&ItemStack::air_default_instance()), 0);
    assert_eq!(swap_util::get_count(&stack(5, 3)), 3);

    let src = vec![stack(1, 2), stack(2, 1)];
    let mut dst = vec![stack(9, 9)];
    swap_util::copy_stacks(&src, &mut dst);
    assert_eq!(dst.len(), 2);
    assert_eq!(dst[0], stack(1, 2));
    assert_eq!(dst[1], stack(2, 1));
}

#[test]
fn add_i2i_inventory_swap_gates_air_and_flags_swap_stack() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    let mut menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    menu.slot(10, 10, stack(3, 1));
    menu.slot(20, 20, stack(3, 5));
    menu.slot(30, 30, ItemStack::air_default_instance());

    swap_util::add_i2i_inventory_swap(0, &menu.slots[0], &menu.slots[1], false, 2);
    // Destination slot empty: no swap recorded.
    swap_util::add_i2i_inventory_swap(0, &menu.slots[0], &menu.slots[2], false, 2);

    let (map_len, list_len, swap_stack, amount) = {
        let swaps = state::SWAPS.lock().unwrap();
        let list = swaps.get(&0);
        (swaps.len(), list.map(|l| l.len()).unwrap_or(0), list.map(|l| l[0].base().get_swap_item().is_swap_stack()).unwrap_or(false), list.map(|l| l[0].base().get_amount()).unwrap_or(-1))
    };
    assert_eq!(map_len, 1);
    assert_eq!(list_len, 1);
    assert!(swap_stack);
    assert_eq!(amount, 2);
}

#[test]
fn assign_i2c_swaps_and_slot_index() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    state::CURRENT_STACKS.lock().unwrap().extend([stack(1, 1), stack(2, 7)]);
    assert_eq!(swap_util::get_slot_index(&stack(2, 7)), 1);
    assert_eq!(swap_util::get_slot_index(&stack(9, 1)), -1);

    let less = vec![SwapStacks::new(1, stack(2, 7), stack(2, 4), 3)];
    let mut menu = TestMenu { slots: vec![Slot::new(0, 0), Slot::new(8, 8)], carried: stack(2, 1) };
    swap_util::assign_i2c_swaps(&less, &crate::vec2::Vec2::with_xy(100.0, 100.0), &menu);

    let probe = {
        let swaps = state::SWAPS.lock().unwrap();
        match swaps.get(&ASSUME_CURSOR_STACK_SLOT_INDEX) {
            Some(list) if list.len() == 1 => match &list[0] {
                crate::swaps::AnySwap::ItemToCursor(s) => Some((s.base.get_amount(), s.base.get_start_x(), s.base.get_start_y(), s.is_arrived())),
                _ => None,
            },
            _ => None,
        }
    };
    let (amount, start_x, start_y, arrived) = probe.expect("one i2c swap in the cursor bucket");
    assert_eq!(amount, 3);
    assert_eq!(start_x, 100.0 - (8.0 + 8.0));
    assert_eq!(start_y, 100.0 - (8.0 + 8.0));
    assert!(!arrived);

    // Cursor empty: nothing is assigned.
    state::reset_vars_for_tests();
    menu.carried = ItemStack::air_default_instance();
    swap_util::assign_i2c_swaps(&less, &crate::vec2::Vec2::with_xy(100.0, 100.0), &menu);
    assert!(state::SWAPS.lock().unwrap().is_empty());
}

#[test]
fn assign_i2i_swaps_balances_and_removes_exhausted_less_stacks() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    let mut menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    // slots 0,1 = less sources; slot 2 = more destination.
    menu.slot(0, 0, stack(3, 1));
    menu.slot(10, 10, stack(3, 1));
    menu.slot(20, 20, stack(3, 1));

    let mut more = vec![SwapStacks::new(2, stack(3, 1), stack(3, 4), -3)];
    let mut less = vec![
        SwapStacks::new(0, stack(3, 1), stack(3, 1), 2),
        SwapStacks::new(1, stack(3, 1), stack(3, 1), 1),
    ];
    swap_util::assign_i2i_swaps(&mut more, &mut less, &menu);

    // The more-stack is balanced to zero and stays in its list.
    assert_eq!(more.len(), 1);
    assert_eq!(more[0].item_count_to_change, 0);
    // Both less stacks drained to zero and removed (Java removes each
    // exhausted element inside the loop).
    assert!(less.is_empty());

    let probe = {
        let swaps = state::SWAPS.lock().unwrap();
        swaps.get(&2).map(|list| (list.len(), list[0].base().get_amount(), list[1].base().get_amount(), list[0].base().get_swap_item().is_swap_stack()))
    };
    let (len, a0, a1, is_swap) = probe.expect("two swaps in slot 2 bucket");
    assert_eq!(len, 2);
    assert_eq!(a0, 2);
    assert_eq!(a1, 1);
    assert!(is_swap);
}

#[test]
fn swap_list_index_and_set_render_to_true() {
    let mut list = vec![
        crate::swaps::AnySwap::ItemToItem(crate::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap::new(
            &Slot::new(0, 0), &Slot::new(8, 8), false, 1, stack(1, 1))),
        crate::swaps::AnySwap::ItemToItem(crate::swaps::item_to_item_inventory_swap::ItemToItemInventorySwap::new(
            &Slot::new(1, 1), &Slot::new(9, 9), true, 3, stack(1, 1))),
    ];
    let idx = swap_util::swap_list_index_of(&list, &|s| s.base().is_checked());
    assert_eq!(idx, 1);
    swap_util::set_render_to_true(&mut list);
    assert!(list[0].base().render_destination_slot());
    assert!(list[1].base().render_destination_slot());
}

#[test]
fn config_roundtrip_and_curve_padding() {
    let _serial = serial_guard();
    state::reset_vars_for_tests();
    let dir = std::env::temp_dir().join(format!("ssw-cfg-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    crate::config::ConfigManager::set_config_dir_for_tests(dir.clone());

    let config = crate::config::ConfigManager::initialize_config();
    assert!(config.get_toggle_mod());
    assert_eq!(config.get_animation_speed(), 100);
    assert_eq!(config.get_animation_speed_formatted(), 1.0);

    {
    let mut config = crate::config::ConfigManager::get_config();
    config.set_toggle_mod(false);
    config.set_animation_speed(150);
    // 6 points -> 2 stored floats rows (inner points only).
    let points = [
        Vec2::with_xy(0.0, 0.0), Vec2::with_xy(0.0, 0.0),
        Vec2::with_xy(0.3, 0.4), Vec2::with_xy(0.7, 0.6),
        Vec2::with_xy(1.0, 1.0), Vec2::with_xy(1.0, 1.0),
    ];
    config.set_curve_points(&points);
    }
    crate::config::ConfigManager::save();

    crate::config::ConfigManager::reload_for_tests();
    let read = crate::config::ConfigManager::initialize_config();
    assert!(!read.get_toggle_mod());
    assert_eq!(read.get_animation_speed(), 150);
    assert_eq!(read.curve_points_len_for_tests(), 2);

    let curve = read.get_curve_points();
    assert_eq!(curve.len(), 6);
    assert_eq!(curve[0].v, [0.0, 0.0]);
    assert_eq!(curve[2].v[0] as f32, 0.3f32);
    assert_eq!(curve[2].v[1] as f32, 0.4f32);
    assert_eq!(curve[4].v, [1.0, 1.0]);
    assert_eq!(read.get_splines().len(), 3);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn splines_progress_endpoints() {
    let points = [
        Vec2::with_xy(0.0, 0.0), Vec2::with_xy(0.0, 0.0),
        Vec2::with_xy(0.5, 0.5), Vec2::with_xy(1.0, 1.0),
        Vec2::with_xy(1.0, 1.0), Vec2::with_xy(1.0, 1.0),
    ];
    let splines = crate::catmull_rom::splines_from_points(&points);
    assert_eq!(splines.len(), 3);
    // Segment x/oldX carry the p2/p1 x values.
    assert_eq!(splines[0].old_x, 0.0);
    assert_eq!(splines[0].x, 0.5);
    // t inside the first segment maps through it; t outside falls back to
    // segments.get(0).
    let y = crate::catmull_rom::get_progress(0.25, &splines);
    assert!(y > 0.0 && y < 1.0);
    let fallback = crate::catmull_rom::get_progress(5.0, &splines);
    assert!(!fallback.is_nan());
}

// ================= mixins batch (upstream mixin/, 1:1) =================

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::mixins::abstract_container_screen as acs;
use crate::mixins::gui_graphics_extractor::{self, GraphicsHost};
use crate::mixins::item_stack::ItemStackAccessor;
use crate::mixins::simple_container_accessor::SimpleContainerAccessor;
use crate::mixins::serverbound_container_click_packet::{self, ClickPacketHost, ContainerInput, HashedStack};
use crate::swaps::AnySwap;

struct TestPacketHost {
    slots: Vec<Slot>,
    player: bool,
    container_screen: bool,
}

impl ClickPacketHost for TestPacketHost {
    fn gui_screen_is_container_screen(&self) -> bool {
        self.container_screen
    }
    fn player_present(&self) -> bool {
        self.player
    }
    fn screen_handler_slots_len(&self) -> usize {
        self.slots.len()
    }
    fn screen_handler_get_slot(&self, slot_id: i32) -> Slot {
        self.slots[slot_id as usize].clone()
    }
}

struct TestRenderHost {
    token: u64,
    present: bool,
    items: Vec<ItemStack>,
    carried: ItemStack,
}

impl acs::ScreenRenderHost for TestRenderHost {
    fn gui_screen_token(&self) -> u64 {
        self.token
    }
    fn player_menu_present(&self) -> bool {
        self.present
    }
    fn player_menu_items(&self) -> Vec<ItemStack> {
        self.items.clone()
    }
    fn player_menu_carried(&self) -> ItemStack {
        self.carried.copy()
    }
}

struct TestGraphicsHost {
    delta: f32,
    item_calls: Rc<RefCell<Vec<(i32, i32, i32)>>>,
    deco_calls: Rc<RefCell<Vec<Option<String>>>>,
}

impl GraphicsHost for TestGraphicsHost {
    fn is_config_screen(&self) -> bool {
        false
    }
    fn get_game_time_delta_ticks(&self) -> f32 {
        self.delta
    }
    fn push_matrix(&self) {}
    fn translate(&self, _x: f32, _y: f32) {}
    fn pop_matrix(&self) {}
    // round(1.5) = 1 > 0: not the hotbar
    fn m20(&self) -> f32 {
        1.0
    }
    fn item(&self, stack: &ItemStack, x: i32, y: i32) {
        self.item_calls.borrow_mut().push((stack.get_count(), x, y));
    }
    fn item_decorations(&self, _stack: &ItemStack, _x: i32, _y: i32, count_override: Option<&str>) {
        self.deco_calls.borrow_mut().push(count_override.map(|s| s.to_string()));
    }
    fn handler_present(&self) -> bool {
        false
    }
    fn handler_items(&self) -> Vec<ItemStack> {
        vec![]
    }
    fn handler_carried(&self) -> ItemStack {
        ItemStack::air_default_instance()
    }
}

#[test]
fn item_stack_accessor_set_and_read() {
    let _g = serial_guard();
    let mut s = stack(3, 7);
    assert!(!s.smooth_swapping_is_swap_stack());
    s.smooth_swapping_set_is_swap_stack(true);
    assert!(s.smooth_swapping_is_swap_stack());
}

struct TestContainer {
    stacks: Vec<ItemStack>,
}

impl crate::mixins::simple_container_accessor::SimpleContainerAccessor for TestContainer {
    fn get_stacks(&self) -> Vec<ItemStack> {
        self.stacks.clone()
    }
}

#[test]
fn simple_container_get_stacks() {
    let c = TestContainer { stacks: vec![stack(1, 1), stack(2, 2)] };
    assert_eq!(c.get_stacks().len(), 2);
}

#[test]
fn click_packet_removes_pending_swap() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    // seed a pending i2c swap at slot 5; the <init> TAIL removes it
    let from = Slot::new(0, 0);
    let swap = ItemToCursorInventorySwap::new(&from, &Vec2::with_xy(10.0, 10.0), stack(1, 1), false, 1);
    state::SWAPS.lock().unwrap().insert(5, vec![AnySwap::ItemToCursor(swap)]);
    let host = TestPacketHost { slots: vec![Slot::new(0, 0)], player: true, container_screen: false };
    let modified: HashMap<i32, HashedStack> = HashMap::new();
    serverbound_container_click_packet::on_init(0, 0, 5, 0, &ContainerInput::Pickup, &modified, &HashedStack::Empty, &host);
    assert!(state::SWAPS.lock().unwrap().is_empty());
}

#[test]
fn click_packet_swap_branch_adds_i2i() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    let mut modified: HashMap<i32, HashedStack> = HashMap::new();
    modified.insert(0, HashedStack::ActualItem { item_id: 3, count: 2 });
    modified.insert(2, HashedStack::Empty);
    let slots = vec![
        Slot::with_item(0, 0, stack(3, 2)),
        Slot::new(4, 4),
        Slot::with_item(8, 8, stack(5, 7)),
    ];
    let host = TestPacketHost { slots, player: true, container_screen: true };
    serverbound_container_click_packet::on_init(0, 0, 0, 1, &ContainerInput::Swap, &modified, &HashedStack::Empty, &host);
    // SWAP input sets the copy-on-next-render flag
    assert!(state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst));
    // both directions recorded: destination slot 2 (7 items) and mouse slot 0 (2 items)
    let swaps = state::SWAPS.lock().unwrap();
    assert_eq!(swaps.get(&2).unwrap().len(), 1);
    assert_eq!(swaps.get(&2).unwrap()[0].base().get_amount(), 7);
    assert_eq!(swaps.get(&0).unwrap().len(), 1);
    assert_eq!(swaps.get(&0).unwrap()[0].base().get_amount(), 2);
}

#[test]
fn click_packet_quick_move_sets_click_swap_stack() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::OLD_STACKS.lock().unwrap() = vec![stack(3, 5)];
    let mut modified: HashMap<i32, HashedStack> = HashMap::new();
    modified.insert(0, HashedStack::ActualItem { item_id: 3, count: 2 });
    modified.insert(1, HashedStack::Empty);
    let mut slots = vec![Slot::with_item(0, 0, stack(3, 2)), Slot::new(4, 4)];
    slots[0].allow_modification = false;
    let host = TestPacketHost { slots, player: true, container_screen: true };
    serverbound_container_click_packet::on_init(0, 0, 0, 1, &ContainerInput::QuickMove, &modified, &HashedStack::Empty, &host);
    // new count 2 <= old count 5 (crafting-table-output gate)
    assert_eq!(*state::CLICK_SWAP_STACK.lock().unwrap(), Some(0));
}

#[test]
fn screen_render_single_changed_assigns_i2c() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::OLD_STACKS.lock().unwrap() = vec![stack(3, 5)];
    *state::OLD_CURSOR_STACK.lock().unwrap() = Some(stack(3, 1));
    *state::CURRENT_CURSOR_STACK.lock().unwrap() = Some(stack(3, 1));
    let menu = TestMenu { slots: vec![Slot::new(10, 10)], carried: stack(3, 4) };
    let host = TestRenderHost { token: 7, present: true, items: vec![stack(3, 2)], carried: stack(3, 4) };
    let mut current_screen: Option<u64> = Some(7);
    acs::on_render(50.0, 50.0, &menu, 10, 10, &host, &mut current_screen);
    // cursor gain (+3) matches the slot loss (5 -> 2): one i2c swap, keyed
    // in the assume-cursor bucket like upstream's assignI2CSwaps
    let swaps = state::SWAPS.lock().unwrap();
    assert_eq!(swaps.len(), 1);
    assert_eq!(swaps.get(&state::ASSUME_CURSOR_STACK_SLOT_INDEX).unwrap().len(), 1);
}

#[test]
fn screen_render_click_swap_copies_only() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::OLD_STACKS.lock().unwrap() = vec![stack(1, 1)];
    state::CLICK_SWAP.store(true, std::sync::atomic::Ordering::SeqCst);
    let menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    let host = TestRenderHost { token: 7, present: true, items: vec![stack(1, 9)], carried: stack(1, 9) };
    let mut current_screen: Option<u64> = Some(7);
    acs::on_render(0.0, 0.0, &menu, 0, 0, &host, &mut current_screen);
    assert!(!state::CLICK_SWAP.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(*state::OLD_STACKS.lock().unwrap(), vec![stack(1, 9)]);
    assert!(state::SWAPS.lock().unwrap().is_empty());
}

#[test]
fn screen_render_screen_change_clears_swaps() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    state::SWAPS.lock().unwrap().insert(3, vec![]);
    let menu = TestMenu { slots: vec![], carried: ItemStack::air_default_instance() };
    let host = TestRenderHost { token: 9, present: true, items: vec![stack(1, 1)], carried: ItemStack::air_default_instance() };
    let mut current_screen: Option<u64> = None;
    acs::on_render(0.0, 0.0, &menu, 0, 0, &host, &mut current_screen);
    assert_eq!(current_screen, Some(9));
    assert!(state::SWAPS.lock().unwrap().is_empty());
    assert_eq!(*state::OLD_STACKS.lock().unwrap(), vec![stack(1, 1)]);
}

#[test]
fn graphics_render_progresses_and_arrives() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::CURRENT_STACKS.lock().unwrap() = vec![stack(9, 3)];
    let from = Slot::new(0, 0);
    let swap = ItemToCursorInventorySwap::new(&from, &Vec2::with_xy(-1.0, -1.0), stack(9, 3), false, 1);
    state::SWAPS.lock().unwrap().insert(0, vec![AnySwap::ItemToCursor(swap)]);
    let item_calls = Rc::new(RefCell::new(vec![]));
    let deco_calls = Rc::new(RefCell::new(vec![]));
    let host = TestGraphicsHost { delta: 1.0, item_calls: item_calls.clone(), deco_calls: deco_calls.clone() };
    let stack = stack(9, 3);
    // first call renders (i2c render_destination_slot is false: the slot's
    // own item stays hidden) and advances the swap toward the cursor.
    // The swap starts at the full start->cursor delta (-9, -9) and each
    // tick moves distance/10 * speed in the arrival direction (no clamp):
    // one tick moves ~0.9 per axis, so |x| shrinks below 9.
    assert!(gui_graphics_extractor::on_item_draw(&stack, 5, 5, &host));
    let mid_x = state::SWAPS.lock().unwrap().get(&0).unwrap()[0].base().get_x();
    assert!(mid_x.abs() < 9.0);
    // keep ticking until it arrives and is removed from the map
    for _ in 0..30 {
        gui_graphics_extractor::on_item_draw(&stack, 5, 5, &host);
    }
    assert!(state::SWAPS.lock().unwrap().get(&0).is_none());
    // once the swap is gone, later draws fall through to the plain item()
    // path again, so the slot item renders normally
    assert!(!item_calls.borrow().is_empty());
}

#[test]
fn graphics_overlay_splits_counts() {
    let _g = serial_guard();
    state::reset_vars_for_tests();
    *state::CURRENT_STACKS.lock().unwrap() = vec![stack(4, 5)];
    let from = Slot::with_item(0, 0, stack(4, 5));
    let to = Slot::with_item(8, 8, stack(4, 3));
    crate::swap_util::add_i2i_inventory_swap(0, &from, &to, false, 3);
    state::SWAPS.lock().unwrap().get_mut(&0).unwrap()[0].set_render_destination_slot(true);
    let item_calls = Rc::new(RefCell::new(vec![]));
    let deco_calls = Rc::new(RefCell::new(vec![]));
    let host = TestGraphicsHost { delta: 1.0, item_calls, deco_calls: deco_calls.clone() };
    let stack = stack(4, 5);
    assert!(gui_graphics_extractor::on_draw_item_in_slot(&stack, 5, 5, None, &host));
    // the moving 3 renders its own count; 5 - 3 stays at the slot
    let calls = deco_calls.borrow();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].as_deref(), Some("3"));
    assert_eq!(calls[1].as_deref(), Some("2"));
}
