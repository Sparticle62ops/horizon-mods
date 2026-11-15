//! 1:1 translation of `dev.tr7zw.waveycapes.delegate.PlayerDelegate`.
//!
//! Java: `@AllArgsConstructor public class PlayerDelegate implements
//! MinecraftPlayer` with `@Getter @Delegate(types =
//! MinecraftPlayer.class) private Avatar player;` — every `MinecraftPlayer`
//! call delegates to the wrapped avatar, and the class overrides
//! `getXCloak`/`getZCloak` with the interpolated avatar-state cloak read.

#![allow(non_snake_case, dead_code, unused_variables)]

use crate::versionless::nms::MinecraftPlayer;

/// The `net.minecraft.world.entity.Avatar` seam: the carrier's real avatar
/// type implements the delegated `MinecraftPlayer` surface plus the cloak
/// interpolation the >= 1.21.9 overrides read. The pre-1.21.9 branches
/// (`player.xCloak` / `player.zCloak` fields) are inactive in this tree.
pub trait AvatarSeam: MinecraftPlayer {
    /// `Minecraft.getInstance().getDeltaTracker().getGameTimeDeltaPartialTick(false)`
    fn gameTimeDeltaPartialTick(&self, partial_tick: bool) -> f32;
    /// `acp.avatarState().getInterpolatedCloakX(delta)`
    fn avatarState_getInterpolatedCloakX(&self, delta: f32) -> f64;
    /// `acp.avatarState().getInterpolatedCloakZ(delta)`
    fn avatarState_getInterpolatedCloakZ(&self, delta: f32) -> f64;
}

/// `PlayerDelegate`
#[derive(Debug, Clone, Copy)]
pub struct PlayerDelegate<E: AvatarSeam> {
    /// `private Avatar player`
    pub player: E,
}

/// `@AllArgsConstructor`
impl<E: AvatarSeam> PlayerDelegate<E> {
    pub fn new(player: E) -> Self {
        Self { player }
    }

    /// `@Getter getPlayer()`
    pub fn getPlayer(&self) -> &E {
        &self.player
    }

    /// Lombok `@Delegate(types = MinecraftPlayer.class)`: forward every
    /// interface method to `player`.
    pub fn player_minecraft(&self) -> &dyn MinecraftPlayer {
        &self.player
    }

    /// `public double getXCloak()` override: the interpolated avatar cloak X.
    pub fn getXCloak(&self) -> f64 {
        let delta = self.player.gameTimeDeltaPartialTick(false);
        self.player.avatarState_getInterpolatedCloakX(delta)
    }

    /// `public double getZCloak()` override: the interpolated avatar cloak Z.
    pub fn getZCloak(&self) -> f64 {
        let delta = self.player.gameTimeDeltaPartialTick(false);
        self.player.avatarState_getInterpolatedCloakZ(delta)
    }

    /// `public float getYBodyRotO()`
    pub fn getYBodyRotO(&self) -> f32 {
        self.player.getYBodyRotO()
    }

    /// `public float getYBodyRot()`
    pub fn getYBodyRot(&self) -> f32 {
        self.player.getYBodyRot()
    }

    /// `public double getYo()`
    pub fn getYo(&self) -> f64 {
        self.player.getYo()
    }

    /// `public double getXo()`
    pub fn getXo(&self) -> f64 {
        self.player.getXo()
    }

    /// `public double getZo()`
    pub fn getZo(&self) -> f64 {
        self.player.getZo()
    }
}

/// `LivingEntity` marker: the delegate's wrapped value is the avatar.
impl<E: AvatarSeam> crate::support::LivingEntity for PlayerDelegate<E> {}

/// The `@Delegate` counterpart: `PlayerDelegate<E>` implements
/// `MinecraftPlayer` by forwarding to the wrapped avatar, with the two
/// overridden cloak getters.
impl<E: AvatarSeam> MinecraftPlayer for PlayerDelegate<E> {
    fn isVisuallySwimming(&self) -> bool {
        self.player.isVisuallySwimming()
    }
    fn getXRot(&self) -> f32 {
        self.player.getXRot()
    }
    fn isCrouching(&self) -> bool {
        self.player.isCrouching()
    }
    fn getY(&self) -> f64 {
        self.player.getY()
    }
    fn getYRot(&self) -> f32 {
        self.player.getYRot()
    }
    fn getZ(&self) -> f64 {
        self.player.getZ()
    }
    fn getX(&self) -> f64 {
        self.player.getX()
    }
    fn isUnderWater(&self) -> bool {
        self.player.isUnderWater()
    }
    fn getXCloak(&self) -> f64 {
        PlayerDelegate::getXCloak(self)
    }
    fn getZCloak(&self) -> f64 {
        PlayerDelegate::getZCloak(self)
    }
    fn getYBodyRotO(&self) -> f32 {
        self.player.getYBodyRotO()
    }
    fn getYBodyRot(&self) -> f32 {
        self.player.getYBodyRot()
    }
    fn getYo(&self) -> f64 {
        self.player.getYo()
    }
    fn getXo(&self) -> f64 {
        self.player.getXo()
    }
    fn getZo(&self) -> f64 {
        self.player.getZo()
    }
}
