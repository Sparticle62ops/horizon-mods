//! 1:1 translation of `dev.tr7zw.waveycapes.renderlayers.CustomCapeRenderLayer`.

#![allow(non_snake_case, dead_code)]

use std::cell::RefCell;

use crate::render::{
    CapeNodeCollector, CapeRenderState, MathUtil, Mth, PlayerWrapper, PoseStack, SubmitNodeCollector,
};
use crate::versionless::util::Vector3;
use crate::versionless::modbase;

/// `net.minecraft.client.renderer.entity.RenderLayerParent` seam: the layer
/// reads the parent model's `root()` and `body()` parts.
pub trait RenderLayerParent {
    /// `getParentModel()`
    fn getParentModel(&self) -> &dyn ParentModel;
}

/// The parent's `PlayerModel` (`root()` + `body` parts).
pub trait ParentModel {
    /// `getParentModel().root()`
    fn root(&self) -> &crate::render::ModelPart;
    /// `getParentModel().body`
    fn body(&self) -> &crate::render::ModelPart;
}

/// `WaveyCapesBase.getINSTANCE().getCapeLayer()` until the main class lands
/// (the null-avatar fallback layer; none wired yet).
pub fn getCapeLayer() -> Option<&'static dyn CapeLayerFallback> {
    None
}

/// The fallback layer's `submit` shape.
pub trait CapeLayerFallback {
    fn submit(
        &self,
        pose_stack: &mut PoseStack,
        submit_node_collector: &mut dyn SubmitNodeCollector,
        packed_light: i32,
        render_state: &dyn PlayerWrapper,
        f: f32,
        g: f32,
    );
}

/// `Minecraft.getInstance().getDeltaTracker().getGameTimeDeltaPartialTick(false)`
/// (the engine supplies the tick clock).
pub fn game_time_delta_partial_tick() -> f32 {
    0.0
}

/// `dev.tr7zw.waveycapes.renderlayers.CustomCapeRenderLayer`
pub struct CustomCapeRenderLayer<'a> {
    render_layer_parent: &'a dyn RenderLayerParent,
}

impl<'a> CustomCapeRenderLayer<'a> {
    /// `public CustomCapeRenderLayer(RenderLayerParent<...> renderLayerParent)`
    pub fn new(render_layer_parent: &'a dyn RenderLayerParent) -> Self {
        CustomCapeRenderLayer { render_layer_parent }
    }

    /// `@Override public void submit(PoseStack poseStack, SubmitNodeCollector submitNodeCollector, int packedLight, AvatarRenderState renderState, float f, float g)`
    #[allow(clippy::too_many_arguments)]
    pub fn submit(
        &self,
        pose_stack: &mut PoseStack,
        submit_node_collector: &mut dyn SubmitNodeCollector,
        packed_light: i32,
        render_state: &dyn PlayerWrapper,
        f: f32,
        g: f32,
    ) {
        let cape_render_info = render_state;
        let delta = game_time_delta_partial_tick();
        if cape_render_info.isPlayerInvisible() {
            return;
        }
        if cape_render_info.hasElytraEquipped() {
            return;
        }

        if !cape_render_info.isCapeVisible() {
            return;
        }
        // Fallback for avatars not backed by an entity, like the player preview in the main menu
        if cape_render_info.getAvatar().is_none()
            && cape_render_info.skinCape().is_some()
            && cape_render_info.showCape()
            && getCapeLayer().is_some()
        {
            getCapeLayer().unwrap().submit(pose_stack, submit_node_collector, packed_light, render_state, f, g);
            return;
        }

        pose_stack.pushPose();
        self.render_layer_parent.getParentModel().root().translateAndRotate(pose_stack);
        self.render_layer_parent.getParentModel().body().translateAndRotate(pose_stack);

        if cape_render_info.hasChestplateEquipped() {
            pose_stack.translate(0.0, -0.053125, 0.06875);
        }

        if modbase::computeGravityVector() {
            if let Some(cape_holder) = cape_render_info.getAvatar() {
                if cape_holder.canUpdateGravityVector() {
                    if let Some(simulation) = cape_holder.getSimulation() {
                        let body_pos = MathUtil::getWorldSpacePosition(pose_stack.last());

                        pose_stack.pushPose();
                        pose_stack.translate(0.0, 1.0, 0.0);
                        let up_pos = MathUtil::getWorldSpacePosition(pose_stack.last());
                        pose_stack.popPose();

                        //net.minecraft.gizmos.Gizmos.line(bodyPos.add(1, 1, 1), upPos.add(1, 1, 1), 0xFF00FF00, 10.0F);
                        let mut orientation = up_pos;
                        orientation.subtract(&body_pos);
                        orientation.normalize();
                        // remove the players yaw from the orientation vector, so the cape is not affected by the players yaw
                        let body_y_rot = cape_holder.yBodyRot() - 90.0;
                        let relative_orientation = Vector3::new(
                            (orientation.x as f64 * Mth::cos(-body_y_rot * MathUtil::DEG_TO_RAD) as f64
                                - orientation.z as f64 * Mth::sin(-body_y_rot * MathUtil::DEG_TO_RAD) as f64)
                                as f32,
                            orientation.y,
                            (orientation.x as f64 * Mth::sin(-body_y_rot * MathUtil::DEG_TO_RAD) as f64
                                + orientation.z as f64 * Mth::cos(-body_y_rot * MathUtil::DEG_TO_RAD) as f64)
                                as f32,
                        );
                        /*
                        net.minecraft.gizmos.Gizmos.line(bodyPos.add(-1, 1, -1),
                            bodyPos.add(-1 + relativeOrientation.x, 1 + relativeOrientation.y, -1 + relativeOrientation.z),
                            0xFFFFFF00, 10.0F);
                         */
                        simulation.setGravityDirection(&relative_orientation);
                        cape_holder.setGravityVectorRequest(false);
                    }
                }
            }
        }

        wavey_base::INSTANCE()
            .getCapeNodeCollector()
            .submitCape(submit_node_collector, cape_render_info, pose_stack, packed_light, delta);

        pose_stack.popPose();
    }
}

/// `WaveyCapesBase.INSTANCE` seam (the main class ports later): one shared
/// `CapeNodeCollector` per render thread.
pub mod wavey_base {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        static BASE: RefCell<WaveyCapesBase> = RefCell::new(WaveyCapesBase::new());
    }

    /// `WaveyCapesBase.getINSTANCE()`
    pub fn INSTANCE() -> &'static WaveyCapesBase {
        BASE.with(|i| unsafe { &*(i.as_ptr()) })
    }

    pub struct WaveyCapesBase {
        cape_node_collector: RefCell<CapeNodeCollector>,
    }

    impl WaveyCapesBase {
        fn new() -> Self {
            WaveyCapesBase { cape_node_collector: RefCell::new(CapeNodeCollector::new()) }
        }

        /// `WaveyCapesBase.INSTANCE.getCapeNodeCollector()`
        pub fn getCapeNodeCollector(&self) -> std::cell::Ref<'_, CapeNodeCollector> {
            self.cape_node_collector.borrow()
        }
    }
}
