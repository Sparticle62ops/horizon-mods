//! Upstream `swaps/InventorySwap.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use crate::item::ItemStack;
use crate::vec2::Vec2;

/// `public class InventorySwap`.
#[derive(Debug, Clone)]
pub struct InventorySwap {
    /// `private double x, y` (animated remainder, mutable).
    pub x: f64,
    pub y: f64,
    /// `private final double distance, startX, startY, angle`.
    pub distance: f64,
    pub start_x: f64,
    pub start_y: f64,
    pub angle: f64,
    /// `private boolean renderDestinationSlot` (mutable).
    pub render_destination_slot: bool,
    /// `private final boolean checked`.
    pub checked: bool,
    /// `private final int amount`.
    pub amount: i32,
    /// `private final ItemStack swapItem`.
    pub swap_item: ItemStack,
}

impl InventorySwap {
    /// `new InventorySwap(Vec2 fromVec, Vec2 toVec, ItemStack swapItem,
    /// boolean checked, int amount)`. Upstream casts the angle through
    /// `(float)`; kept.
    pub fn new(from_vec: &Vec2, to_vec: &Vec2, swap_item: ItemStack, checked: bool, amount: i32) -> Self {
        let x = to_vec.v[0] - from_vec.v[0];
        let y = to_vec.v[1] - from_vec.v[1];
        let start_x = to_vec.v[0] - from_vec.v[0];
        let start_y = to_vec.v[1] - from_vec.v[1];
        let angle = (y.atan2(x) + std::f64::consts::PI) as f32 as f64;
        let distance = (x * x + y * y).sqrt();
        Self {
            x,
            y,
            distance,
            start_x,
            start_y,
            angle,
            render_destination_slot: false,
            checked,
            amount,
            swap_item,
        }
    }

    /// `getAngle()`.
    pub fn get_angle(&self) -> f64 {
        self.angle
    }

    /// `getX()` / `getY()`.
    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    /// `getStartX()` / `getStartY()`.
    pub fn get_start_x(&self) -> f64 {
        self.start_x
    }

    pub fn get_start_y(&self) -> f64 {
        self.start_y
    }

    /// `getDistance()`.
    pub fn get_distance(&self) -> f64 {
        self.distance
    }

    /// `renderDestinationSlot()`.
    pub fn render_destination_slot(&self) -> bool {
        self.render_destination_slot
    }

    /// `setRenderDestinationSlot(boolean)`.
    pub fn set_render_destination_slot(&mut self, render_destination_slot: bool) {
        self.render_destination_slot = render_destination_slot;
    }

    /// `isChecked()`.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// `getAmount()`.
    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    /// `getSwapItem()`.
    pub fn get_swap_item(&self) -> &ItemStack {
        &self.swap_item
    }

    /// `toString()` (Java shape kept, fields in upstream order).
    pub fn to_display_string(&self) -> String {
        format!(
            "InventorySwap{{x={}, y={}, distance={}, startX={}, startY={}, angle={}, renderDestinationSlot={}, checked={}, amount={}, swapItem={}}}",
            self.x,
            self.y,
            self.distance,
            self.start_x,
            self.start_y,
            self.angle,
            self.render_destination_slot,
            self.checked,
            self.amount,
            self.swap_item
        )
    }
}

