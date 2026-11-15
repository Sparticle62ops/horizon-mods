//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.nms.MinecraftPlayer`.
//!
//! Java NMS interface bridging the versionless code to the mapped player
//! class; the Rust side is a trait the mixin layer implements for the real
//! player type.

#![allow(non_snake_case, dead_code)]

/// `dev.tr7zw.waveycapes.versionless.nms.MinecraftPlayer`
pub trait MinecraftPlayer: crate::support::LivingEntity {
    /// `boolean isVisuallySwimming()`
    fn isVisuallySwimming(&self) -> bool;
    /// `float getXRot()`
    fn getXRot(&self) -> f32;
    /// `boolean isCrouching()`
    fn isCrouching(&self) -> bool;
    /// `double getY()`
    fn getY(&self) -> f64;
    /// `float getYRot()`
    fn getYRot(&self) -> f32;
    /// `double getZ()`
    fn getZ(&self) -> f64;
    /// `double getX()`
    fn getX(&self) -> f64;
    /// `boolean isUnderWater()`
    fn isUnderWater(&self) -> bool;
    /// `double getXCloak()`
    fn getXCloak(&self) -> f64;
    /// `double getZCloak()`
    fn getZCloak(&self) -> f64;
    /// `float getYBodyRotO()`
    fn getYBodyRotO(&self) -> f32;
    /// `float getYBodyRot()`
    fn getYBodyRot(&self) -> f32;
    /// `double getYo()`
    fn getYo(&self) -> f64;
    /// `double getXo()`
    fn getXo(&self) -> f64;
    /// `double getZo()`
    fn getZo(&self) -> f64;
}
