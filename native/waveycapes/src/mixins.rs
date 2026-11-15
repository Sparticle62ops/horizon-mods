//! 1:1 translation of the three WaveyCapes mixins
//! (`mixin/PlayerRendererMixin`, `mixin/PlayerMixin`,
//! `mixin/LivingEntityRendererMixin`). The mixin classes are spliced into the
//! engine's types, so the state and injected bodies live on the carrier: the
//! `PlayerMixinCarrier` trait is the engine entity carrying the PlayerMixin
//! fields and the `CapeHolder` impl, and the free functions are the
//! injection-point bodies.

#![allow(non_snake_case, dead_code, unused_variables)]

use crate::support::SupportManager;
use crate::versionless::holder::CapeHolder;
use crate::versionless::nms::MinecraftPlayer;
use crate::versionless::sim::BasicSimulation;
use crate::versionless::util::Vector3;
use std::any::Any;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// PlayerMixin
// ---------------------------------------------------------------------------

/// `@Mixin(value = LivingEntity.class) public abstract class PlayerMixin
/// extends Entity implements CapeHolder` — the >= 1.21.9 branch. The carrier
/// owns the fields below and the `CapeHolder` impl.
pub trait PlayerMixinCarrier: CapeHolder {
    /// `@Unique private BasicSimulation simulation` read
    fn simulation_slot_is_null(&self) -> bool;
    /// `@Unique private BasicSimulation simulation = null`
    fn simulation_set_null(&mut self);
    /// `@Unique private boolean dirty`
    fn is_dirty(&self) -> bool;
    /// `@Unique private boolean dirty = false` — `setDirty()`'s write
    fn set_dirty_flag(&mut self, dirty: bool);
    /// `@Unique private int capeIdleTicks` / `CAPE_IDLE_TICKS` reads+writes
    fn capeIdleTicks(&self) -> i32;
    fn capeIdleTicks_set(&mut self, ticks: i32);
    /// `((Object) this instanceof Avatar)` — the >= 1.21.9 instanceof gate
    fn isAvatar(&self) -> bool;
    /// `PlayerDelegate playerDelegate = new PlayerDelegate(entity);
    /// simulate(playerDelegate);` — the carrier builds its delegate view and
    /// runs the CapeHolder simulate step with it.
    fn simulateWithPlayerDelegate(&mut self);
}

/// `@Unique private static final int CAPE_IDLE_TICKS = 100`
pub const CAPE_IDLE_TICKS: i32 = 100;

/// `@Override public void setDirty() { this.dirty = true; }`
pub fn setDirty<C: PlayerMixinCarrier + ?Sized>(carrier: &mut C) {
    carrier.set_dirty_flag(true);
}

/// `@Override public void markCapeRendered()`
pub fn markCapeRendered<C: PlayerMixinCarrier + ?Sized>(carrier: &mut C) {
    carrier.capeIdleTicks_set(0);
    if carrier.simulation_slot_is_null() {
        simulateCape(carrier);
    }
}

/// `@Inject(method = "tick", at = @At("TAIL")) private void
/// moveCloakUpdate(CallbackInfo info)`
pub fn moveCloakUpdate<C: PlayerMixinCarrier + ?Sized>(carrier: &mut C) {
    if carrier.capeIdleTicks() >= CAPE_IDLE_TICKS {
        carrier.simulation_set_null();
        return;
    }
    carrier.capeIdleTicks_set(carrier.capeIdleTicks() + 1);
    simulateCape(carrier);
}

/// `@Unique private void simulateCape()`
pub fn simulateCape<C: PlayerMixinCarrier + ?Sized>(carrier: &mut C) {
    // >= 1.21.9: `if (!((Object) this instanceof Avatar)) return;`
    if !carrier.isAvatar() {
        return;
    }
    carrier.updateSimulation(16);
    carrier.setGravityVectorRequest(true);
    if carrier.is_dirty() {
        carrier.set_dirty_flag(false);
        carrier.with_simulation(|simulation| {
            simulation.applyMovement(&Vector3::new(1.0f32, 1.0f32, 0.0));
        });
        // quickly doing a few simulation steps to get the cape int a stable
        // configuration
        for _ in 0..5 {
            carrier.simulateWithPlayerDelegate();
        }
    }
    carrier.simulateWithPlayerDelegate();
}

/// `@Override public UUID getWCUUID() { return getUUID(); }` — the carrier's
/// entity UUID read is its `getWCUUID` impl.

// ---------------------------------------------------------------------------
// PlayerRendererMixin
// ---------------------------------------------------------------------------

/// `@Mixin(net.minecraft.client.renderer.entity.player.AvatarRenderer.class)
/// public abstract class PlayerRendererMixin`. `@Unique private boolean
/// injectedCape = false;` and the `onCreate` constructor RETURN injection:
/// `injectedCape = true; addLayer(new CustomCapeRenderLayer(this));`
///
/// `layer_factory` is the engine's `addLayer(RenderLayer)` on the renderer
/// being constructed; the pre-1.17.0 `renderLegacyWorkaround` branch is
/// inactive in this tree.
pub fn onCreate(
    renderer: &dyn crate::renderlayers::RenderLayerParent,
    layer_factory: &mut dyn FnMut(crate::renderlayers::CustomCapeRenderLayer),
) {
    layer_factory(crate::renderlayers::CustomCapeRenderLayer::new(renderer));
}

// ---------------------------------------------------------------------------
// LivingEntityRendererMixin
// ---------------------------------------------------------------------------

/// `@Mixin(LivingEntityRenderer.class) public class LivingEntityRendererMixin
/// <S extends LivingEntity, T extends LivingEntityRenderState, M extends
/// EntityModel<? super T>>`. The `addLayer` HEAD injection, cancellable.
///
/// `render_layer` carries the layer's runtime identity (`instanceof CapeLayer`
/// and the supports' `blockFeatureRenderer` checks read it); `cancel` reports
/// the Java `info.cancel()`.
pub fn addLayer(
    render_layer: &dyn Any,
    cape_layer: Option<crate::wavey_capes_base::CapeLayer>,
    cancel: &mut dyn FnMut(),
) {
    // `if ((Object) renderLayer instanceof CapeLayer capeLayer)` — the
    // engine hands the captured layer over as `cape_layer`.
    if let Some(cape_layer) = cape_layer {
        // `WaveyCapesBase.getINSTANCE().setCapeLayer(capeLayer);`
        crate::wavey_capes_base::WaveyCapesBase::setCapeLayer(Some(cape_layer));
        cancel();
        return;
    }
    for support in SupportManager::getSupportedMods() {
        if support.blockFeatureRenderer(render_layer) {
            cancel();
            return;
        }
    }
}
