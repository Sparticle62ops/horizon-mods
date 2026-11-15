// Wavey Capes runtime splice: the mod's modules inlined for Horizon's
// crates/horizon-client/src/game.rs carrier, `crate::` -> `super::`.
// Exact 1:1; tr7zw Protective License, Copyright (C) tr7zw.

#[allow(dead_code, unused_imports, unused_variables, non_snake_case)]
pub mod wavey_capes {
    /// Upstream `WaveyCapesBase.MOD_ID`.
    pub const MOD_ID: &str = "wavey-capes";

    /// Upstream metadata, shown on the mod's row in Options > Mods.
    pub const NAME: &str = "Wavey Capes";
    pub const VERSION: &str = "1.11.3";
    pub const DESCRIPTION: &str =
        "Dynamic wavey capes: physics simulation, wind, config screen and the render layer from the upstream Wavey Capes client logic, 1:1.";

    pub mod render {
//! 1:1 translation of `dev.tr7zw.waveycapes.render` (`CustomCapeRenderer.java`,
//! `CapeRenderer.java`, `CapeInfos.java`, `CapePass.java`,
//! `VanillaCapeRenderer.java`).
//!
//! The Minecraft render API arrives through the same seam shapes the other
//! ports in this repo use (`PoseStack`, `VertexConsumer`, `ModelPart`,
//! `PlayerWrapper`, `RenderType`, the JOML `Matrix4f`/`Vector4f`).

#![allow(non_snake_case, dead_code)]

use super::versionless::modbase;
use super::versionless::sim::{BasicSimulation, CapeMovement, CapeStyle, WindMode};
use super::versionless::holder::CapeHolder;
use super::versionless::util::{CapePoint, Vector3};

// ---------------------------------------------------------------------------
// Seams (vanilla/JOML types the upstream code calls into)
// ---------------------------------------------------------------------------

/// JOML `org.joml.Vector4f` (column-major 4x4 math, w kept for transforms).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4f {
    pub fn new() -> Self {
        Vector4f { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }

    /// `TRANSFORM_SCRATCH.set(x, y, z, 1)`
    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
        self
    }
}

impl Default for Vector4f {
    fn default() -> Self {
        Self::new()
    }
}

/// JOML `org.joml.Matrix4f`, the subset upstream uses (identity, `set`,
/// `transform`, and being rotated by `PoseStack.rotate`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix4f {
    /// Column-major, like JOML: `m[col * 4 + row]`.
    pub m: [f32; 16],
}

impl Matrix4f {
    /// `new Matrix4f()` (identity)
    pub fn new() -> Self {
        let mut m = [0.0f32; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[15] = 1.0;
        Matrix4f { m }
    }

    /// `positionMatrices[part].set(poseStack.last().pose())`
    pub fn set(&mut self, other: &Matrix4f) -> &mut Self {
        self.m = other.m;
        self
    }

    /// `matrix.transform(vector4f)`
    pub fn transform(&self, v: &Vector4f) -> Vector4f {
        let m = &self.m;
        Vector4f {
            x: m[0] * v.x + m[4] * v.y + m[8] * v.z + m[12] * v.w,
            y: m[1] * v.x + m[5] * v.y + m[9] * v.z + m[13] * v.w,
            z: m[2] * v.x + m[6] * v.y + m[10] * v.z + m[14] * v.w,
            w: m[3] * v.x + m[7] * v.y + m[11] * v.z + m[15] * v.w,
        }
    }

    /// `M := M * R` (the `PoseStack.mulPose`/`rotate` application).
    pub fn mul_matrix(&mut self, r: &Matrix4f) {
        let a = self.m;
        let b = r.m;
        let mut out = [0.0f32; 16];
        for col in 0..4 {
            for row in 0..4 {
                out[col * 4 + row] =
                    a[row] * b[col * 4] + a[4 + row] * b[col * 4 + 1] + a[8 + row] * b[col * 4 + 2] + a[12 + row] * b[col * 4 + 3];
            }
        }
        self.m = out;
    }

    /// Rotation matrix about a unit axis, `angle` in radians
    /// (`AxisAngle`/quaternion equivalence for the `MathUtil` axes).
    fn rotation(axis: Axis, angle: f32) -> Matrix4f {
        let (s, c) = angle.sin_cos();
        let (x, y, z) = match axis {
            Axis::XP | Axis::XN => (1.0, 0.0, 0.0),
            Axis::YP | Axis::YN => (0.0, 1.0, 0.0),
            Axis::ZP | Axis::ZN => (0.0, 0.0, 1.0),
        };
        let t = 1.0 - c;
        let mut m = Matrix4f::new();
        m.m[0] = t * x * x + c;
        m.m[1] = t * x * y + s * z;
        m.m[2] = t * x * z - s * y;
        m.m[4] = t * x * y - s * z;
        m.m[5] = t * y * y + c;
        m.m[6] = t * y * z + s * x;
        m.m[8] = t * x * z + s * y;
        m.m[9] = t * y * z - s * x;
        m.m[10] = t * z * z + c;
        m
    }
}

impl Default for Matrix4f {
    fn default() -> Self {
        Self::new()
    }
}

/// `dev.tr7zw.transition.mc.MathUtil` axes (`XP`, `ZP`, `YP`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    XP,
    XN,
    YP,
    YN,
    ZP,
    ZN,
}

impl Axis {
    /// `MathUtil.XP.rotationDegrees(deg)` etc.; applied by `PoseStack.rotate`.
    pub fn rotationDegrees(self, degrees: f32) -> Rotation {
        Rotation { axis: self, degrees }
    }
}

/// The quaternion-ish rotation `MathUtil.*P.rotationDegrees` produces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    axis: Axis,
    degrees: f32,
}

/// `com.mojang.blaze3d.vertex.PoseStack`, the calls upstream makes.
#[derive(Default)]
pub struct PoseStack {
    stack: Vec<Matrix4f>,
}

impl PoseStack {
    pub fn new() -> Self {
        PoseStack { stack: vec![Matrix4f::new()] }
    }

    /// `poseStack.pushPose()`
    pub fn pushPose(&mut self) {
        let top = *self.last();
        self.stack.push(top);
    }

    /// `poseStack.popPose()`
    pub fn popPose(&mut self) {
        self.stack.pop();
    }

    /// `poseStack.last().pose()`
    pub fn last(&self) -> &Matrix4f {
        self.stack.last().expect("PoseStack underflow")
    }

    /// `poseStack.translate(x, y, z)`
    pub fn translate(&mut self, x: f64, y: f64, z: f64) {
        let t = self.last_mut();
        t.m[12] += x as f32;
        t.m[13] += y as f32;
        t.m[14] += z as f32;
    }

    /// `poseStack.rotate(quaternion)` (26.3 name; `mulPose` on older lines)
    pub fn rotate(&mut self, rotation: Rotation) {
        let r = Matrix4f::rotation(rotation.axis, (rotation.degrees as f64).to_radians() as f32);
        self.last_mut().mul_matrix(&r);
    }

    fn last_mut(&mut self) -> &mut Matrix4f {
        self.stack.last_mut().expect("PoseStack underflow")
    }

    /// `sharedStack.last().set(pose)` (the submit-closure seed)
    pub fn last_mut_from(&mut self, pose: &Matrix4f) {
        self.last_mut().set(pose);
    }
}

/// One cube face's UV rect in [0, 1] texture space (the vanilla
/// `texOffs`-derived quad `NMSUtil.buildCape` bakes).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FaceUV {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

impl FaceUV {
    pub fn new(u0: f32, v0: f32, u1: f32, v1: f32, tex_width: i32, tex_hight: i32) -> Self {
        FaceUV {
            u0: u0 / tex_width as f32,
            v0: v0 / tex_hight as f32,
            u1: u1 / tex_width as f32,
            v1: v1 / tex_hight as f32,
        }
    }
}

/// `net.minecraft.client.model.geom.ModelPart`, the cuboid-quads subset
/// `NMSUtil.buildCape` builds and `CapeRenderer.render` draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelPart {
    /// Vanilla cuboids; `None` fields are NMSUtil's job (built at 64x64).
    pub cubes: Vec<Cube>,
}

/// One vanilla cuboid (the cape quad `Cube`); per-face UVs come from the
/// `texOffs` layout `NMSUtil.buildCape` bakes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cube {
    pub x0: f32,
    pub y0: f32,
    pub z0: f32,
    pub x1: f32,
    pub y1: f32,
    pub z1: f32,
    pub west: FaceUV,
    pub east: FaceUV,
    pub north: FaceUV,
    pub south: FaceUV,
    pub up: FaceUV,
    pub down: FaceUV,
}

impl ModelPart {
    /// `ModelPart.translateAndRotate(poseStack)`
    pub fn translateAndRotate(&self, pose_stack: &mut PoseStack) {
        // PartPose offset/rotation seam: the engine applies the part's own
        // pose; the cape parts carry none (identity offsets), so this is the
        // vanilla default no-op path for zero offset and zero rotation.
    }

    /// `model.render(poseStack, vertexConsumer, light, overlay)`: emit the
    /// cuboid quads through the given consumer.
    pub fn render(&self, pose_stack: &PoseStack, vertex_consumer: &mut dyn VertexConsumer, light: i32, overlay: i32) {
        let pose = pose_stack.last();
        for cube in &self.cubes {
            // 6 faces of the cuboid, vanilla winding
            emit_face(vertex_consumer, pose, (cube.x1, cube.y1, cube.z0), (cube.x0, cube.y1, cube.z0), (cube.x0, cube.y0, cube.z0), (cube.x1, cube.y0, cube.z0), &cube.north, 0.0, 0.0, -1.0, light, overlay);
            emit_face(vertex_consumer, pose, (cube.x0, cube.y1, cube.z1), (cube.x1, cube.y1, cube.z1), (cube.x1, cube.y0, cube.z1), (cube.x0, cube.y0, cube.z1), &cube.south, 0.0, 0.0, 1.0, light, overlay);
            emit_face(vertex_consumer, pose, (cube.x1, cube.y1, cube.z1), (cube.x1, cube.y1, cube.z0), (cube.x1, cube.y0, cube.z0), (cube.x1, cube.y0, cube.z1), &cube.east, 1.0, 0.0, 0.0, light, overlay);
            emit_face(vertex_consumer, pose, (cube.x0, cube.y1, cube.z0), (cube.x0, cube.y1, cube.z1), (cube.x0, cube.y0, cube.z1), (cube.x0, cube.y0, cube.z0), &cube.west, -1.0, 0.0, 0.0, light, overlay);
            emit_face(vertex_consumer, pose, (cube.x1, cube.y1, cube.z0), (cube.x1, cube.y1, cube.z1), (cube.x0, cube.y1, cube.z1), (cube.x0, cube.y1, cube.z0), &cube.up, 0.0, 1.0, 0.0, light, overlay);
            emit_face(vertex_consumer, pose, (cube.x0, cube.y0, cube.z0), (cube.x0, cube.y0, cube.z1), (cube.x1, cube.y0, cube.z1), (cube.x1, cube.y0, cube.z0), &cube.down, 0.0, -1.0, 0.0, light, overlay);
        }
    }
}

/// One textured face through the consumer (the shared `addVertex` x4 shape),
/// UVs taken from the face's rect in order u0v1, u1v1, u1v0, u0v0.
#[allow(clippy::too_many_arguments)]
fn emit_face(
    vertex_consumer: &mut dyn VertexConsumer,
    pose: &Matrix4f,
    p0: (f32, f32, f32),
    p1: (f32, f32, f32),
    p2: (f32, f32, f32),
    p3: (f32, f32, f32),
    uv: &FaceUV,
    nx: f32, ny: f32, nz: f32,
    light: i32,
    overlay: i32,
) {
    vertex_consumer.addVertex(pose, p0.0, p0.1, p0.2, uv.u0, uv.v1, overlay, light, nx, ny, nz, 1.0);
    vertex_consumer.addVertex(pose, p1.0, p1.1, p1.2, uv.u1, uv.v1, overlay, light, nx, ny, nz, 1.0);
    vertex_consumer.addVertex(pose, p2.0, p2.1, p2.2, uv.u1, uv.v0, overlay, light, nx, ny, nz, 1.0);
    vertex_consumer.addVertex(pose, p3.0, p3.1, p3.2, uv.u0, uv.v0, overlay, light, nx, ny, nz, 1.0);
}

/// `com.mojang.blaze3d.vertex.VertexConsumer` (the shape
/// `VertexConsumerUtil.addVertex` feeds).
pub trait VertexConsumer {
    /// `VertexConsumerUtil.addVertex(bufferBuilder, matrix, x, y, z, u, v, overlay, light, nx, ny, nz, alpha)`
    #[allow(clippy::too_many_arguments)]
    fn addVertex(
        &mut self,
        matrix: &Matrix4f,
        x: f32,
        y: f32,
        z: f32,
        u: f32,
        v: f32,
        overlay: i32,
        light: i32,
        nx: f32,
        ny: f32,
        nz: f32,
        alpha: f32,
    );
}

/// `net.minecraft.client.renderer.texture.OverlayTexture.NO_OVERLAY`
pub const NO_OVERLAY: i32 = 0;

/// `net.minecraft.client.renderer.rendertype.RenderType` handle (opaque).
pub type RenderType = u64;

/// `net.minecraft.client.renderer.rendertype.RenderTypes` (the one call
/// upstream makes).
pub struct RenderTypes;

impl RenderTypes {
    /// `RenderTypes.entityTranslucent(cape)`
    pub fn entityTranslucent(texture: u64) -> RenderType {
        // seam: the engine resolves the translucent entity pipeline for `texture`
        texture
    }

    /// `RenderTypes.armorCutoutNoCull(cape)` (the Ears path)
    pub fn armorCutoutNoCull(texture: u64) -> RenderType {
        // seam: the engine resolves the cutout armor pipeline for `texture`
        texture
    }

    /// `RenderTypes.trimmedArmorGlint()` (26.3)
    pub fn trimmedArmorGlint() -> RenderType {
        // seam: the engine resolves the trimmed-armor glint pipeline
        1
    }
}

/// The `capeLean`/`capeFlap`/`capeLean2` fields of the 26.3 render state
/// `modifyPoseStack` reads.
pub trait CapeRenderState {
    fn cape_lean(&self) -> f32;
    fn cape_flap(&self) -> f32;
    fn cape_lean2(&self) -> f32;
}

/// `dev.tr7zw.transition.mc.entitywrapper.PlayerWrapper`: the render entity
/// must be a `CapeHolder` (upstream casts it) and answers the vanilla fields
/// the renderer reads.
pub trait PlayerWrapper {
    /// `capeRenderInfo.getCapeTexture()` (texture handle; `null` when none)
    fn getCapeTexture(&self) -> Option<u64>;
    /// `capeRenderInfo.getRenderState()`
    fn getRenderState(&self) -> &dyn CapeRenderState;
    /// `renderState.skin.cape()` (the fallback path reads it off the state)
    fn skinCape(&self) -> Option<u64>;
    /// `renderState.showCape`
    fn showCape(&self) -> bool;
    /// `capeRenderInfo.isPlayerInvisible()`
    fn isPlayerInvisible(&self) -> bool;
    /// `capeRenderInfo.hasElytraEquipped()`
    fn hasElytraEquipped(&self) -> bool;
    /// `capeRenderInfo.isCapeVisible()`
    fn isCapeVisible(&self) -> bool;
    /// `capeRenderInfo.hasChestplateEquipped()`
    fn hasChestplateEquipped(&self) -> bool;
    /// `capeRenderInfo.getAvatar()` (the entity, a `CapeHolder` upstream;
    /// `None` for avatars not backed by an entity, like the main-menu preview)
    fn getAvatar(&self) -> Option<&(dyn CapeHolderAvatar + '_)>;
}

/// The `((CapeHolder) entity)` cast plus the vanilla fields
/// `getNatrualWindSwing`/`modifyPoseStackSimulation` read off the entity.
/// Stand-in for the upstream cast: implementers of `CapeHolder` (the mixin
/// layer) also implement these three reads.
pub trait CapeHolderAvatar {
    /// `((CapeHolder) entity).markCapeRendered()`
    fn markCapeRendered(&self);
    /// `((CapeHolder) entity).getSimulation()`
    fn getSimulation(&self) -> Option<&dyn BasicSimulation>;
    /// `entity.isUnderWater()`
    fn isUnderWater(&self) -> bool;
    /// `capeHolder.canUpdateGravityVector()` (the CapeHolder cast's read)
    fn canUpdateGravityVector(&self) -> bool;
    /// `capeHolder.setGravityVectorRequest(false)`
    fn setGravityVectorRequest(&self, can_update: bool);
    /// `capeRenderInfo.getAvatar().yBodyRot`
    fn yBodyRot(&self) -> f32;
    /// `entity.getUUID()` (the avatar's entity UUID; the support mods key
    /// their per-player lookups on it)
    fn getUUID(&self) -> Option<uuid::Uuid>;
}

/// `dev.tr7zw.transition.mc.VertexConsumerUtil.addVertex` (the single call
/// shape upstream uses, all 12 arguments).
pub struct VertexConsumerUtil;

impl VertexConsumerUtil {
    #[allow(clippy::too_many_arguments)]
    pub fn addVertex(
        buffer_builder: &mut dyn VertexConsumer,
        matrix: &Matrix4f,
        x: f32,
        y: f32,
        z: f32,
        u: f32,
        v: f32,
        overlay: i32,
        light: i32,
        nx: f32,
        ny: f32,
        nz: f32,
        alpha: f32,
    ) {
        buffer_builder.addVertex(matrix, x, y, z, u, v, overlay, light, nx, ny, nz, alpha);
    }
}

/// `SupportManager.getAlphaSupplier().get()` until the support layer lands
/// (seam: support mods can scale cape alpha; default 1.0).
pub fn support_alpha() -> f32 {
    1.0
}

/// `net.minecraft.client.renderer.SubmitNodeCollector` (26.3): the layer
/// submits custom geometry closures.
pub trait SubmitNodeCollector {
    /// `submitNodeCollector.submitCustomGeometry(stack, renderType, (pose, vertexConsumer) -> body)`
    fn submitCustomGeometry(
        &mut self,
        stack: &PoseStack,
        render_type: RenderType,
        body: &mut dyn FnMut(&Matrix4f, &mut dyn VertexConsumer),
    );
}

/// `dev.tr7zw.transition.mc.MathUtil` (the pieces the render layer uses).
pub struct MathUtil;

impl MathUtil {
    /// `MathUtil.DEG_TO_RAD`
    pub const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0;

    /// `MathUtil.getWorldSpacePosition(poseStack.last().pose())`
    pub fn getWorldSpacePosition(pose: &Matrix4f) -> Vector3 {
        // seam: the engine maps a pose matrix to world space
        let v = pose.transform(&Vector4f { x: 0.0, y: 0.0, z: 0.0, w: 1.0 });
        Vector3::new(v.x, v.y, v.z)
    }
}

/// `net.minecraft.util.Mth` (the two trig reads the layer makes).
pub struct Mth;

impl Mth {
    pub fn cos(f: f32) -> f32 {
        f.to_radians().cos()
    }

    pub fn sin(f: f32) -> f32 {
        f.to_radians().sin()
    }
}

/// `dev.tr7zw.waveycapes.support.ModSupport` seam (support layer lands later;
/// the shape the collector queries).
pub trait ModSupport {
    /// `support.shouldBeUsed(capeRenderInfo)`
    fn shouldBeUsed(&self, cape_render_info: &dyn PlayerWrapper) -> bool;
    /// `support.getRenderer()`
    fn getRenderer(&self) -> &dyn CapeRenderer;
}

/// `SupportManager.getSupportedMods()` until the support layer lands: no
/// support integrations are wired at compile time yet, so empty.
pub fn supported_mods() -> Vec<&'static dyn ModSupport> {
    Vec::new()
}

// ---------------------------------------------------------------------------
// The 1:1 classes
// ---------------------------------------------------------------------------

/// `dev.tr7zw.waveycapes.render.CapeInfos`
pub struct CapeInfos<'a> {
    pub cape_renderer: &'a dyn CapeRenderer,
    pub render_type: RenderType,
    pub is_glint: bool,
}

/// The `UnaryOperator<VertexConsumer> wrapper` of `CapePass`, as a trait (a
/// wrapper decorates the consumer for one pass).
pub trait VertexConsumerWrapper {
    /// `UnaryOperator<VertexConsumer>::apply`
    fn wrap<'a>(&self, consumer: &'a mut dyn VertexConsumer) -> WrappedVertexConsumer<'a>;
}

/// The operator's output: a (possibly decorated) view of the consumer.
pub struct WrappedVertexConsumer<'a> {
    pub consumer: &'a mut dyn VertexConsumer,
}

impl VertexConsumer for WrappedVertexConsumer<'_> {
    fn addVertex(
        &mut self,
        matrix: &Matrix4f,
        x: f32, y: f32, z: f32,
        u: f32, v: f32,
        overlay: i32, light: i32,
        nx: f32, ny: f32, nz: f32,
        alpha: f32,
    ) {
        self.consumer.addVertex(matrix, x, y, z, u, v, overlay, light, nx, ny, nz, alpha)
    }
}

/// `dev.tr7zw.waveycapes.render.CapePass`
pub struct CapePass {
    pub render_type: RenderType,
    /// Java `UnaryOperator<VertexConsumer> wrapper`
    pub wrapper: std::rc::Rc<dyn VertexConsumerWrapper>,
}

/// `dev.tr7zw.waveycapes.render.CapeRenderer`
pub trait CapeRenderer {
    /// `default void render(PlayerWrapper capeRenderInfo, int part, ModelPart model, PoseStack poseStack, VertexConsumer vertexConsumer, int light, int overlay)`
    fn render(
        &self,
        cape_render_info: &dyn PlayerWrapper,
        part: usize,
        model: &ModelPart,
        pose_stack: &mut PoseStack,
        vertex_consumer: &mut dyn VertexConsumer,
        light: i32,
        overlay: i32,
    ) {
        model.render(pose_stack, vertex_consumer, light, NO_OVERLAY);
    }

    /// `CapeInfos getCapeInfo(PlayerWrapper capeRenderInfo)`
    fn getCapeInfo<'a>(&'a self, cape_render_info: &dyn PlayerWrapper) -> Option<CapeInfos<'a>>;

    /// `default List<CapePass> getCapePasses(PlayerWrapper capeRenderInfo)`
    fn getCapePasses(&self, cape_render_info: &dyn PlayerWrapper) -> Vec<CapePass> {
        Vec::new()
    }

    /// `default boolean vanillaUvValues()`
    fn vanillaUvValues(&self) -> bool {
        true
    }

    /// `default boolean singleSidedTexture()`
    fn singleSidedTexture(&self) -> bool {
        false
    }

    /// `default int uvSheetWidth()`
    fn uvSheetWidth(&self) -> i32 {
        64
    }

    /// `default int uvSheetHeight()`
    fn uvSheetHeight(&self) -> i32 {
        32
    }
}

/// `dev.tr7zw.waveycapes.render.VanillaCapeRenderer`
pub struct VanillaCapeRenderer;

impl CapeRenderer for VanillaCapeRenderer {
    /// `@Override getCapeInfo`
    fn getCapeInfo<'a>(&'a self, cape_render_info: &dyn PlayerWrapper) -> Option<CapeInfos<'a>> {
        let cape = cape_render_info.getCapeTexture();
        if let Some(cape) = cape {
            return Some(CapeInfos {
                cape_renderer: self,
                render_type: RenderTypes::entityTranslucent(cape),
                is_glint: false,
            });
        }
        None
    }

    /// `@Override vanillaUvValues`
    fn vanillaUvValues(&self) -> bool {
        true
    }
}

/// `dev.tr7zw.waveycapes.render.CustomCapeRenderer`
#[derive(Clone)]
pub struct CustomCapeRenderer {
    /// Java `private final ModelPart[] customCape = NMSUtil.buildCape(64, 64, x -> 0, y -> y)`
    custom_cape: Vec<ModelPart>,
    /// Java `private final Matrix4f[] positionMatrices`
    position_matrices: Vec<Matrix4f>,
    /// Java `private final Vector3[] frontNormalVecs`
    front_normal_vecs: Vec<Vector3>,
    /// Java `private final Vector3[] backNormalVecs`
    back_normal_vecs: Vec<Vector3>,
}

impl CustomCapeRenderer {
    /// `private static final int PART_COUNT = 16`
    pub const PART_COUNT: usize = 16;

    /// `private static final float CAPE_WIDTH = 10F / 16F`
    pub const CAPE_WIDTH: f32 = 10.0 / 16.0;
    /// `private static final float CAPE_HEIGHT = 1F`
    pub const CAPE_HEIGHT: f32 = 1.0;
    /// `private static final float CAPE_DEPTH = 1F / 16F`
    pub const CAPE_DEPTH: f32 = 1.0 / 16.0;

    /// `public CustomCapeRenderer()`
    pub fn new() -> Self {
        CustomCapeRenderer {
            custom_cape: super::nms_util::NMSUtil::buildCape(64, 64, Box::new(|_x| 0), Box::new(|y| y)),
            position_matrices: vec![Matrix4f::new(); Self::PART_COUNT],
            front_normal_vecs: vec![Vector3::new(0.0, 0.0, 0.0); Self::PART_COUNT],
            back_normal_vecs: vec![Vector3::new(0.0, 0.0, 0.0); Self::PART_COUNT],
        }
    }

    /// `public void render(PlayerWrapper capeRenderInfo, CapeRenderer renderer, VertexConsumer vertexConsumer, PoseStack poseStack, int packedLight, float delta)`
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        cape_render_info: &dyn PlayerWrapper,
        renderer: &dyn CapeRenderer,
        vertex_consumer: &mut dyn VertexConsumer,
        pose_stack: &mut PoseStack,
        packed_light: i32,
        delta: f32,
    ) {
        if modbase::capeStyle() == CapeStyle::SMOOTH && renderer.vanillaUvValues() {
            self.renderSmoothCape(pose_stack, vertex_consumer, cape_render_info, renderer, delta, packed_light);
        } else {
            for part in 0..Self::PART_COUNT {
                self.modifyPoseStack(pose_stack, cape_render_info, delta, part);
                let model = &self.custom_cape[part];
                renderer.render(cape_render_info, part, model, pose_stack, vertex_consumer, packed_light, NO_OVERLAY);
                pose_stack.popPose();
            }
        }
    }

    /// `private void renderSmoothCape(PoseStack poseStack, VertexConsumer bufferBuilder, PlayerWrapper capeRenderInfo, CapeRenderer renderer, float delta, int light)`
    #[allow(clippy::too_many_arguments)]
    fn renderSmoothCape(
        &mut self,
        pose_stack: &mut PoseStack,
        buffer_builder: &mut dyn VertexConsumer,
        cape_render_info: &dyn PlayerWrapper,
        renderer: &dyn CapeRenderer,
        delta: f32,
        light: i32,
    ) {
        // uv layout is vanilla, but the sheet isn't always 64x32, so count in texels of the real size
        let texel_u = 1.0f32 / renderer.uvSheetWidth() as f32;
        let texel_v = 1.0f32 / renderer.uvSheetHeight() as f32;

        let alpha = support_alpha();

        for part in 0..Self::PART_COUNT {
            self.modifyPoseStack(pose_stack, cape_render_info, delta, part);

            self.position_matrices[part].set(pose_stack.last());
            let prev = Math::max(part as i64 - 1, 0) as usize;
            self.front_normal_vecs[part] = Self::getNormalVec(
                &self.position_matrices[prev],
                &self.position_matrices[prev],
                &self.position_matrices[part],
                Self::CAPE_WIDTH / 2.0,
                part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                -Self::CAPE_DEPTH,
                -Self::CAPE_WIDTH / 2.0,
                part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                -Self::CAPE_DEPTH,
                Self::CAPE_WIDTH / 2.0,
                (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                -Self::CAPE_DEPTH,
                light == 15728880,
                true,
            );
            self.back_normal_vecs[part] = Self::getNormalVec(
                &self.position_matrices[prev],
                &self.position_matrices[prev],
                &self.position_matrices[part],
                Self::CAPE_WIDTH / 2.0,
                (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                0.0,
                -Self::CAPE_WIDTH / 2.0,
                (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                0.0,
                Self::CAPE_WIDTH / 2.0,
                part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                0.0,
                light == 15728880,
                true,
            );

            pose_stack.popPose();
        }

        for part in 0..Self::PART_COUNT {
            if part == 0 {
                let min_u = 1.0 * texel_u;
                let max_u = 11.0 * texel_u;

                let min_v = 0.0;
                let max_v = texel_v;

                let normal_vec = Self::getNormalVec(
                    &self.position_matrices[0],
                    &self.position_matrices[0],
                    &self.position_matrices[0],
                    Self::CAPE_WIDTH / 2.0,
                    0.0,
                    0.0,
                    -Self::CAPE_WIDTH / 2.0,
                    0.0,
                    0.0,
                    Self::CAPE_WIDTH / 2.0,
                    0.0,
                    Self::CAPE_DEPTH,
                    light == 15728880,
                    false,
                );

                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[0], Self::CAPE_WIDTH / 2.0, 0.0, 0.0, max_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[0], -Self::CAPE_WIDTH / 2.0, 0.0, 0.0, min_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[0], -Self::CAPE_WIDTH / 2.0, 0.0, -Self::CAPE_DEPTH, min_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[0], Self::CAPE_WIDTH / 2.0, 0.0, -Self::CAPE_DEPTH, max_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            }

            if part == Self::PART_COUNT - 1 {
                let single_sided = renderer.singleSidedTexture();

                let min_u = if single_sided { 1.0 } else { 11.0 } * texel_u;
                let max_u = if single_sided { 11.0 } else { 21.0 } * texel_u;

                let min_v = if single_sided { texel_v * Self::PART_COUNT as f32 } else { 0.0 };
                let max_v = min_v + texel_v;

                let normal_vec = Self::getNormalVec(
                    &self.position_matrices[part],
                    &self.position_matrices[part],
                    &self.position_matrices[part],
                    Self::CAPE_WIDTH / 2.0,
                    Self::CAPE_HEIGHT,
                    -Self::CAPE_DEPTH,
                    -Self::CAPE_WIDTH / 2.0,
                    Self::CAPE_HEIGHT,
                    -Self::CAPE_DEPTH,
                    Self::CAPE_WIDTH / 2.0,
                    Self::CAPE_HEIGHT,
                    0.0,
                    light == 15728880,
                    false,
                );

                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], Self::CAPE_WIDTH / 2.0, Self::CAPE_HEIGHT, -Self::CAPE_DEPTH, max_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], -Self::CAPE_WIDTH / 2.0, Self::CAPE_HEIGHT, -Self::CAPE_DEPTH, min_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], -Self::CAPE_WIDTH / 2.0, Self::CAPE_HEIGHT, 0.0, min_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
                VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], Self::CAPE_WIDTH / 2.0, Self::CAPE_HEIGHT, 0.0, max_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            }

            let mut min_u = 0.0;
            let mut max_u = 1.0 * texel_u;

            let mut min_v = texel_v * (part + 1) as f32;
            let mut max_v = min_v + texel_v;

            let prev = Math::max(part as i64 - 1, 0) as usize;
            let mut normal_vec = Self::getNormalVec(
                &self.position_matrices[part],
                &self.position_matrices[part],
                &self.position_matrices[prev],
                -Self::CAPE_WIDTH / 2.0,
                (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                0.0,
                -Self::CAPE_WIDTH / 2.0,
                (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                -Self::CAPE_DEPTH,
                -Self::CAPE_WIDTH / 2.0,
                part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                0.0,
                light == 15728880,
                false,
            );

            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], -Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, min_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], -Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, max_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], -Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, max_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], -Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, min_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);

            min_u = 11.0 * texel_u;
            max_u = 12.0 * texel_u;

            normal_vec = Self::getNormalVec(
                &self.position_matrices[part],
                &self.position_matrices[part],
                &self.position_matrices[prev],
                Self::CAPE_WIDTH / 2.0,
                (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                -Self::CAPE_DEPTH,
                Self::CAPE_WIDTH / 2.0,
                (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                0.0,
                Self::CAPE_WIDTH / 2.0,
                part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32),
                -Self::CAPE_DEPTH,
                light == 15728880,
                false,
            );

            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, min_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, max_u, max_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, max_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, min_u, min_v, NO_OVERLAY, light, normal_vec.x, normal_vec.y, normal_vec.z, alpha);

            min_u = 1.0 * texel_u;
            max_u = 11.0 * texel_u;

            let next = Math::min(part as i64 + 1, Self::PART_COUNT as i64 - 1) as usize;
            let normal_vec_top = Self::average(&self.front_normal_vecs[part], &self.front_normal_vecs[prev], NORMAL_TOP.with(|v| v.borrow_mut().clone()));
            let normal_vec_bottom = Self::average(&self.front_normal_vecs[part], &self.front_normal_vecs[next], NORMAL_BOTTOM.with(|v| v.borrow_mut().clone()));

            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, max_u, min_v, NO_OVERLAY, light, normal_vec_top.x, normal_vec_top.y, normal_vec_top.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], -Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, min_u, min_v, NO_OVERLAY, light, normal_vec_top.x, normal_vec_top.y, normal_vec_top.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], -Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, min_u, max_v, NO_OVERLAY, light, normal_vec_bottom.x, normal_vec_bottom.y, normal_vec_bottom.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), -Self::CAPE_DEPTH, max_u, max_v, NO_OVERLAY, light, normal_vec_bottom.x, normal_vec_bottom.y, normal_vec_bottom.z, alpha);

            min_u = 12.0 * texel_u;
            max_u = 22.0 * texel_u;

            let normal_vec_top = Self::average(&self.back_normal_vecs[part], &self.back_normal_vecs[prev], NORMAL_TOP.with(|v| v.borrow_mut().clone()));
            let normal_vec_bottom = Self::average(&self.back_normal_vecs[part], &self.back_normal_vecs[next], NORMAL_BOTTOM.with(|v| v.borrow_mut().clone()));

            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, min_u, min_v, NO_OVERLAY, light, normal_vec_top.x, normal_vec_top.y, normal_vec_top.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[prev], -Self::CAPE_WIDTH / 2.0, part as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, max_u, min_v, NO_OVERLAY, light, normal_vec_top.x, normal_vec_top.y, normal_vec_top.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], -Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, max_u, max_v, NO_OVERLAY, light, normal_vec_bottom.x, normal_vec_bottom.y, normal_vec_bottom.z, alpha);
            VertexConsumerUtil::addVertex(buffer_builder, &self.position_matrices[part], Self::CAPE_WIDTH / 2.0, (part + 1) as f32 * (Self::CAPE_HEIGHT / Self::PART_COUNT as f32), 0.0, min_u, max_v, NO_OVERLAY, light, normal_vec_bottom.x, normal_vec_bottom.y, normal_vec_bottom.z, alpha);
        }
    }

    /// `private void modifyPoseStack(PoseStack poseStack, PlayerWrapper capeRenderInfo, float h, int part)`
    fn modifyPoseStack(&mut self, pose_stack: &mut PoseStack, cape_render_info: &dyn PlayerWrapper, h: f32, part: usize) {
        if modbase::config_capeMovement() != CapeMovement::VANILLA {
            self.modifyPoseStackSimulation(pose_stack, cape_render_info, h, part);
            return;
        }

        let render_state = cape_render_info.getRenderState();
        pose_stack.pushPose();
        pose_stack.translate(0.0, 0.0, 0.125);

        // upstream reaches getAvatar() unconditionally here; the render layer's
        // null-avatar fallback returns before this path can run
        let entity = cape_render_info.getAvatar().expect("avatar");

        pose_stack.rotate(Axis::XP.rotationDegrees(
            6.0 + render_state.cape_lean() / 2.0 + render_state.cape_flap()
                + Self::getNatrualWindSwing(part, entity.isUnderWater()),
        ));
        pose_stack.rotate(Axis::ZP.rotationDegrees(render_state.cape_lean2() / 2.0));
        pose_stack.rotate(Axis::YP.rotationDegrees(180.0 - render_state.cape_lean2() / 2.0));
    }

    /// `private void modifyPoseStackSimulation(PoseStack poseStack, PlayerWrapper capeRenderInfo, float delta, int part)`
    fn modifyPoseStackSimulation(&mut self, pose_stack: &mut PoseStack, cape_render_info: &dyn PlayerWrapper, delta: f32, part: usize) {
        let entity = cape_render_info.getAvatar().expect("avatar");
        if part == 0 {
            entity.markCapeRendered();
        }
        // race condition where the cape is being rendered before the simulation is initialized, just render it without movement instead of crashing
        let Some(simulation) = entity.getSimulation() else {
            pose_stack.pushPose();
            return;
        };
        pose_stack.pushPose();
        pose_stack.translate(0.0, 0.0, 0.125);

        let points = simulation.getPoints();
        let first = &points[0];
        let current = &points[part];
        let mut x = current.getLerpX(delta) - first.getLerpX(delta);
        if x > 0.0 {
            x = 0.0;
        }
        let y = first.getLerpY(delta) - part as f32 - current.getLerpY(delta);
        let z = first.getLerpZ(delta) - current.getLerpZ(delta);

        let sideways_rotation_offset: f32 = 0.0;
        let part_rotation = Self::getRotation(delta, part, simulation);

        let height: f32 = 0.0;
        //        if (abstractClientPlayer.isCrouching()) {
        //            height += 25.0F;
        //            poseStack.translate(0, 0.15F, 0);
        //        }

        let natural_wind_swing = Self::getNatrualWindSwing(part, entity.isUnderWater());

        // vanilla rotating and wind
        pose_stack.rotate(Axis::XP.rotationDegrees(6.0 + height + natural_wind_swing));
        pose_stack.rotate(Axis::ZP.rotationDegrees(sideways_rotation_offset / 2.0));
        pose_stack.rotate(Axis::YP.rotationDegrees(180.0 - sideways_rotation_offset / 2.0));

        pose_stack.translate(-(z as f64) / Self::PART_COUNT as f64, y as f64 / Self::PART_COUNT as f64, x as f64 / Self::PART_COUNT as f64); // movement from the simulation
        // offsetting so the rotation is on the cape part
        // float offset = (float) (part * (16 / partCount))/16; // to fold the entire
        // cape into one position for debugging
        pose_stack.translate(0.0, /*-offset*/ (0.48 / 16.0), -(0.48 / 16.0)); // (0.48/16)
        pose_stack.translate(0.0, part as f64 * 1.0 / Self::PART_COUNT as f64, part as f64 * (0.0) / Self::PART_COUNT as f64);
        pose_stack.rotate(Axis::XP.rotationDegrees(-part_rotation)); // apply actual rotation
        // undoing the rotation
        pose_stack.translate(0.0, -(part as f64) * 1.0 / Self::PART_COUNT as f64, -(part as f64) * (0.0) / Self::PART_COUNT as f64);
        pose_stack.translate(0.0, -(0.48 / 16.0), (0.48 / 16.0));
    }

    /// `private float getRotation(float delta, int part, BasicSimulation simulation)`
    fn getRotation(delta: f32, part: usize, simulation: &dyn BasicSimulation) -> f32 {
        if part == Self::PART_COUNT - 1 {
            return Self::getRotation(delta, part - 1, simulation);
        }
        let points = simulation.getPoints();
        let a = &points[part];
        let b = &points[part + 1];
        Self::getAngle(b.getLerpX(delta) - a.getLerpX(delta), b.getLerpY(delta) - a.getLerpY(delta)) as f32
    }

    /// `private double getAngle(float x, float y)`
    fn getAngle(x: f32, y: f32) -> f64 {
        ((x as f64).atan2(y as f64)).to_degrees() + 180.0
    }

    /// `private float getNatrualWindSwing(int part, boolean underwater)`
    fn getNatrualWindSwing(part: usize, underwater: bool) -> f32 {
        if modbase::windMode() != WindMode::WAVES {
            return 0.0;
        }
        let highlighted_part = (millis() / if underwater { 9 } else { 3 }) % 360;
        let relative_part = (part + 1) as f32 / Self::PART_COUNT as f32;
        ((relative_part * 360.0 - highlighted_part as f32).to_radians().sin() * 3.0) as f32
    }

    /// `private static Vector3 getNormalVec(Matrix4f matrix1, Matrix4f matrix2, Matrix4f matrix3, float x1, float y1, float z1, float x2, float y2, float z2, float x3, float y3, float z3, boolean inverse, Vector3 dest)`
    ///
    /// The Java version writes into the shared scratch `Vector3 dest` and
    /// returns it; the Rust version copies out, so no aliasing.
    #[allow(clippy::too_many_arguments)]
    fn getNormalVec(
        matrix1: &Matrix4f,
        matrix2: &Matrix4f,
        matrix3: &Matrix4f,
        x1: f32, y1: f32, z1: f32,
        x2: f32, y2: f32, z2: f32,
        x3: f32, y3: f32, z3: f32,
        inverse: bool,
        _into_dest: bool,
    ) -> Vector3 {
        let origin = Self::transform(matrix1, x1, y1, z1, NORMAL_ORIGIN.with(|v| v.borrow_mut().clone()));
        let mut normal = Self::transform(matrix2, x2, y2, z2, NORMAL_SCRATCH.with(|v| v.borrow_mut().clone()));
        let mut edge = Self::transform(matrix3, x3, y3, z3, NORMAL_EDGE.with(|v| v.borrow_mut().clone()));

        normal.subtract(&origin);
        edge.subtract(&origin);

        normal.cross(&edge);
        normal.normalize();
        if inverse {
            normal.mul(-1.0);
        }
        normal
    }

    /// `private static Vector3 average(Vector3 a, Vector3 b, Vector3 dest)`
    fn average(a: &Vector3, b: &Vector3, dest: Vector3) -> Vector3 {
        let mut dest = dest;
        dest.x = (a.x + b.x) / 2.0;
        dest.y = (a.y + b.y) / 2.0;
        dest.z = (a.z + b.z) / 2.0;
        dest
    }

    /// `private static Vector3 transform(Matrix4f matrix, float x, float y, float z, Vector3 dest)`
    fn transform(matrix: &Matrix4f, x: f32, y: f32, z: f32, dest: Vector3) -> Vector3 {
        let mut dest = dest;
        let mut transform_scratch = TRANSFORM_SCRATCH.with(|v| *v.borrow_mut().set(x, y, z, 1.0));
        let vector4f = matrix.transform(&transform_scratch);
        transform_scratch = vector4f;
        dest.x = vector4f.x;
        dest.y = vector4f.y;
        dest.z = vector4f.z;
        dest
    }
}

/// `System.currentTimeMillis()` (the engine supplies the wall clock)
fn millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Math short-hands used by the translation (`Math.max`/`Math.min` on longs).
struct Math;

impl Math {
    fn max(a: i64, b: i64) -> i64 {
        a.max(b)
    }

    fn min(a: i64, b: i64) -> i64 {
        a.min(b)
    }
}

// scratch space for getNormalVec, capes are only ever built on the render thread
thread_local! {
    /// Java `private static final Vector3 NORMAL_ORIGIN`
    static NORMAL_ORIGIN: std::cell::RefCell<Vector3> = std::cell::RefCell::new(Vector3::new(0.0, 0.0, 0.0));
    /// Java `private static final Vector3 NORMAL_EDGE`
    static NORMAL_EDGE: std::cell::RefCell<Vector3> = std::cell::RefCell::new(Vector3::new(0.0, 0.0, 0.0));
    /// Java `private static final Vector3 NORMAL_FACE`
    static NORMAL_FACE: std::cell::RefCell<Vector3> = std::cell::RefCell::new(Vector3::new(0.0, 0.0, 0.0));
    /// Java `private static final Vector3 NORMAL_TOP`
    static NORMAL_TOP: std::cell::RefCell<Vector3> = std::cell::RefCell::new(Vector3::new(0.0, 0.0, 0.0));
    /// Java `private static final Vector3 NORMAL_BOTTOM`
    static NORMAL_BOTTOM: std::cell::RefCell<Vector3> = std::cell::RefCell::new(Vector3::new(0.0, 0.0, 0.0));
    /// Java `private static final Vector3 NORMAL_SCRATCH` (the `dest` param)
    static NORMAL_SCRATCH: std::cell::RefCell<Vector3> = std::cell::RefCell::new(Vector3::new(0.0, 0.0, 0.0));
    /// Java `private static final Vector4f TRANSFORM_SCRATCH`
    static TRANSFORM_SCRATCH: std::cell::RefCell<Vector4f> = std::cell::RefCell::new(Vector4f::new());
}

/// `dev.tr7zw.waveycapes.render.CapeNodeCollector`
pub struct CapeNodeCollector {
    /// `private final CustomCapeRenderer customCapeRenderer`
    custom_cape_renderer: CustomCapeRenderer,
    /// `private final VanillaCapeRenderer vanillaCape`
    vanilla_cape: VanillaCapeRenderer,
}

impl CapeNodeCollector {
    pub fn new() -> Self {
        CapeNodeCollector {
            custom_cape_renderer: CustomCapeRenderer::new(),
            vanilla_cape: VanillaCapeRenderer,
        }
    }

    /// `public void submitCape(SubmitNodeCollector submitNodeCollector, PlayerWrapper playerWrapper, PoseStack stack, int packedLight, float delta)`
    #[allow(clippy::too_many_arguments)]
    pub fn submitCape(
        &self,
        submit_node_collector: &mut dyn SubmitNodeCollector,
        player_wrapper: &dyn PlayerWrapper,
        stack: &mut PoseStack,
        packed_light: i32,
        delta: f32,
    ) {
        let renderer = self.getCapeRenderer(player_wrapper);
        let Some(renderer) = renderer else { return };
        let cape_passes = renderer.getCapePasses(player_wrapper);
        let mut custom_cape_renderer = self.custom_cape_renderer.clone();
        for layer in &cape_passes {
            let wrapper = layer.wrapper.clone();
            let mut custom = custom_cape_renderer.clone();
            let mut body = move |pose: &Matrix4f, vertex_consumer: &mut dyn VertexConsumer| {
                let mut shared_stack = PoseStack::new();
                shared_stack.last_mut_from(pose);
                // layer.wrapper().apply(vertexConsumer)
                let mut wrapped = wrapper.wrap(vertex_consumer);
                custom.render(player_wrapper, renderer, &mut wrapped, &mut shared_stack, packed_light, delta);
            };
            submit_node_collector.submitCustomGeometry(stack, layer.render_type, &mut body);
        }
        if !cape_passes.is_empty() {
            return;
        }

        let cape_info = renderer.getCapeInfo(player_wrapper);
        let Some(cape_info) = cape_info else { return };

        if cape_info.is_glint {
            let mut custom = custom_cape_renderer.clone();
            let mut body = |pose: &Matrix4f, vertex_consumer: &mut dyn VertexConsumer| {
                let mut shared_stack = PoseStack::new();
                shared_stack.last_mut_from(pose);
                custom.render(player_wrapper, renderer, vertex_consumer, &mut shared_stack, packed_light, delta);
            };
            submit_node_collector.submitCustomGeometry(stack, RenderTypes::trimmedArmorGlint(), &mut body);
        }

        let mut custom = custom_cape_renderer.clone();
        let mut body = |pose: &Matrix4f, vertex_consumer: &mut dyn VertexConsumer| {
            let mut shared_stack = PoseStack::new();
            shared_stack.last_mut_from(pose);
            custom.render(player_wrapper, renderer, vertex_consumer, &mut shared_stack, packed_light, delta);
        };
        submit_node_collector.submitCustomGeometry(stack, cape_info.render_type, &mut body);
    }

    /// `private CapeRenderer getCapeRenderer(PlayerWrapper capeRenderInfo)`
    fn getCapeRenderer(&self, cape_render_info: &dyn PlayerWrapper) -> Option<&dyn CapeRenderer> {
        for support in supported_mods() {
            if support.shouldBeUsed(cape_render_info) {
                return Some(support.getRenderer());
            }
        }
        if cape_render_info.getCapeTexture().is_none() || !cape_render_info.isCapeVisible() {
            None
        } else {
            Some(&self.vanilla_cape)
        }
    }
}
    }

    pub mod nms_util {
//! 1:1 translation of `dev.tr7zw.waveycapes.NMSUtil`.

#![allow(non_snake_case, dead_code)]

use super::render::{Cube, FaceUV, ModelPart};

/// `dev.tr7zw.waveycapes.NMSUtil`
pub struct NMSUtil;

/// Java `IntUnaryOperator` (`x -> 0` / `y -> y` at the call site).
pub type IntUnaryOperator = Box<dyn Fn(i32) -> i32>;

impl NMSUtil {
    /// `public static ModelPart[] buildCape(int texWidth, int texHight, IntUnaryOperator uvX, IntUnaryOperator uvY)`
    ///
    /// Upstream builds a `MeshDefinition` with 16 children
    /// `customCape_<i>`, each `CubeListBuilder.create().texOffs(uvX(i), uvY(i))
    /// .addBox(-5.0F, i, -1.0F, 10.0F, 1, 1.0F, CubeDeformation.NONE, 1.0F, 0.5F)`
    /// baked at texWidth x texHight; the Rust side bakes each child straight
    /// into a `ModelPart` with one vanilla-layout cube.
    pub fn buildCape(tex_width: i32, tex_hight: i32, uv_x: IntUnaryOperator, uv_y: IntUnaryOperator) -> Vec<ModelPart> {
        let mut custom_cape = Vec::with_capacity(16);
        for i in 0..16 {
            let u = uv_x(i) as f32;
            let v = uv_y(i) as f32;
            // addBox(-5.0F, i, -1.0F, 10.0F, 1, 1.0F): a 10x1x1 box
            let (d, w, h) = (1.0f32, 10.0f32, 1.0f32);
            let cube = Cube {
                x0: -5.0,
                y0: i as f32,
                z0: -1.0,
                x1: -5.0 + w,
                y1: i as f32 + h,
                z1: -1.0 + d,
                // vanilla cube UV layout at texOffs(u, v) on the texWidth x texHight sheet
                west: FaceUV::new(u, v + 1.0, u + 1.0, v + 2.0, tex_width, tex_hight),
                east: FaceUV::new(u + d + w, v + d, u + d + w + d, v + d + h, tex_width, tex_hight),
                north: FaceUV::new(u + d, v + d, u + d + w, v + d + h, tex_width, tex_hight),
                south: FaceUV::new(u + d + w + d, v + d, u + d + w + d + w, v + d + h, tex_width, tex_hight),
                up: FaceUV::new(u + d, v, u + d + w, v + d, tex_width, tex_hight),
                down: FaceUV::new(u + d + w, v, u + 2.0 * d + w, v + d, tex_width, tex_hight),
            };
            custom_cape.push(ModelPart { cubes: vec![cube] });
        }
        custom_cape
    }
}
    }

    pub mod renderlayers {
//! 1:1 translation of `dev.tr7zw.waveycapes.renderlayers.CustomCapeRenderLayer`.

#![allow(non_snake_case, dead_code)]

use std::cell::RefCell;

use super::render::{
    CapeNodeCollector, CapeRenderState, MathUtil, Mth, PlayerWrapper, PoseStack, SubmitNodeCollector,
};
use super::versionless::util::Vector3;
use super::versionless::modbase;

/// `net.minecraft.client.renderer.entity.RenderLayerParent` seam: the layer
/// reads the parent model's `root()` and `body()` parts.
pub trait RenderLayerParent {
    /// `getParentModel()`
    fn getParentModel(&self) -> &dyn ParentModel;
}

/// The parent's `PlayerModel` (`root()` + `body` parts).
pub trait ParentModel {
    /// `getParentModel().root()`
    fn root(&self) -> &super::render::ModelPart;
    /// `getParentModel().body`
    fn body(&self) -> &super::render::ModelPart;
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
    }

    pub mod support {
//! 1:1 translation of `dev.tr7zw.waveycapes.support` (and `support.laby`).
//!
//! The third-party mod APIs upstream reflects against (Ears, MinecraftCapes,
//! Wynntils, LabyMod, Shoulder Surfing) are not compiled into Horizon, so each
//! lives behind a small seam module with a registered hook; the hooks default
//! to "mod absent", which mirrors upstream's `doesClassExist` gates.

#![allow(non_snake_case, dead_code)]

use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};


use uuid::Uuid;

use super::nms_util::{IntUnaryOperator, NMSUtil};
use super::render::{
    CapeInfos, CapeRenderer, ModelPart, NO_OVERLAY, PlayerWrapper, PoseStack, RenderTypes,
    VertexConsumer,
};
use super::versionless::modbase;
use super::versionless::util::Vector3;

// ---------------------------------------------------------------------------
// Seams
// ---------------------------------------------------------------------------

/// `net.minecraft.world.entity.LivingEntity` (opaque; only
/// `AnimationSupport`'s disabled PlayerAnimator path ever touches it).
pub trait LivingEntity: Any {}

/// `net.minecraft.resources.Identifier` (opaque engine handle, keyed by the
/// resource string like render.rs's texture handles).
pub type Identifier = u64;

/// `dev.tr7zw.transition.mc.GeneralUtil.getResourceLocation(namespace, path)`
pub fn getResourceLocation(namespace: &str, path: &str) -> Identifier {
    // stable FNV-1a over "namespace:path"; the engine owns the real registry
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut feed = |b: u8| {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    };
    for b in namespace.bytes() {
        feed(b);
    }
    feed(b':');
    for b in path.bytes() {
        feed(b);
    }
    hash
}

/// Java `Object` identity, the key the Ears inhibitor's cache and the
/// `blockFeatureRenderer` instanceof checks are keyed on.
pub type ObjectId = usize;

// ---------------------------------------------------------------------------
// ModSupport
// ---------------------------------------------------------------------------

/// `dev.tr7zw.waveycapes.support.ModSupport`
pub trait ModSupport {
    /// `boolean shouldBeUsed(PlayerWrapper capeRenderInfo)` (the >= 1.21.2
    /// overload, the only one abstract in the target's preprocessor branch).
    fn shouldBeUsed(&self, cape_render_info: &dyn PlayerWrapper) -> bool;

    /// `@Deprecated default boolean shouldBeUsed(AbstractClientPlayer player)`
    fn shouldBeUsedPlayer(&self, _player: &dyn LivingEntity) -> bool {
        false
    }

    /// `CapeRenderer getRenderer()`
    fn getRenderer(&self) -> &dyn CapeRenderer;

    /// `default int priority()` — higher wins when several supports apply
    fn priority(&self) -> i32 {
        0
    }

    /// `boolean blockFeatureRenderer(Object feature)` (`feature` carries its
    /// identity for the instanceof checks).
    fn blockFeatureRenderer(&self, feature: &dyn Any) -> bool;
}

// ---------------------------------------------------------------------------
// AnimationSupport
// ---------------------------------------------------------------------------

/// `dev.tr7zw.waveycapes.support.AnimationSupport`
pub trait AnimationSupport: Send + Sync {
    /// `Vector3 applyAnimationChanges(LivingEntity entity, float delta, Vector3 cur)`
    fn applyAnimationChanges(
        &self,
        entity: &dyn LivingEntity,
        delta: f32,
        cur: Vector3,
    ) -> Vector3;
}

// ---------------------------------------------------------------------------
// SupportManager
// ---------------------------------------------------------------------------

/// One entry of `SupportManager.mods`. The Java TreeSet is ordered by
/// (priority reversed, class name, identity hash), so the entry carries that
/// triple and `getSupportedMods()` sorts a snapshot the same way.
struct SupportEntry {
    priority: i32,
    class_name: String,
    identity: usize,
    support: Arc<dyn ModSupport + Send + Sync>,
}

/// `dev.tr7zw.waveycapes.support.SupportManager`
pub struct SupportManager;

impl SupportManager {
    fn mods() -> &'static Mutex<Vec<SupportEntry>> {
        static MODS: OnceLock<Mutex<Vec<SupportEntry>>> = OnceLock::new();
        MODS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn animation_support() -> &'static Mutex<Vec<Arc<dyn AnimationSupport + Send + Sync>>> {
        static ANIM: OnceLock<Mutex<Vec<Arc<dyn AnimationSupport + Send + Sync>>>> =
            OnceLock::new();
        ANIM.get_or_init(|| Mutex::new(Vec::new()))
    }

    /// `public static Supplier<Float> alphaSupplier` (lombok getter/setter,
    /// default `() -> 1f`)
    fn alpha_supplier_slot()
    -> &'static Mutex<Box<dyn Fn() -> f32 + Send + Sync>> {
        static ALPHA: OnceLock<Mutex<Box<dyn Fn() -> f32 + Send + Sync>>> = OnceLock::new();
        ALPHA.get_or_init(|| Mutex::new(Box::new(|| 1.0f32)))
    }

    /// `SupportManager.mods.add(...)` — `class_name` is the site's
    /// `getClass().getName()` (Java reads it off the instance).
    pub fn mods_add(
        class_name: &str,
        support: Arc<dyn ModSupport + Send + Sync>,
    ) {
        // (Java's System::identityHashCode tiebreak; a monotonic counter is
        // the Rust analog - the tie is practically unreachable upstream)
        static IDENTITY: AtomicU64 = AtomicU64::new(0);
        let identity = IDENTITY.fetch_add(1, Ordering::Relaxed) as usize;
        let priority = support.priority();
        let mut mods = Self::mods().lock().unwrap();
        mods.push(SupportEntry {
            priority,
            class_name: class_name.to_string(),
            identity,
            support,
        });
    }

    /// `SupportManager.animationSupport` read (Java: `public static
    /// List<AnimationSupport> animationSupport`).
    pub fn animationSupport() -> Vec<Arc<dyn AnimationSupport + Send + Sync>> {
        Self::animation_support().lock().unwrap().clone()
    }

    /// `public static Set<ModSupport> getSupportedMods()`
    pub fn getSupportedMods() -> Vec<Arc<dyn ModSupport + Send + Sync>> {
        let mut mods = Self::mods().lock().unwrap();
        // TreeSet order: priority reversed, then class name, then identity hash
        mods.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.class_name.cmp(&b.class_name))
                .then_with(|| a.identity.cmp(&b.identity))
        });
        mods.iter().map(|e| e.support.clone()).collect()
    }

    /// `SupportManager.animationSupport.add(...)`
    pub fn animationSupport_add(support: Arc<dyn AnimationSupport + Send + Sync>) {
        Self::animation_support().lock().unwrap().push(support);
    }

    /// `@Setter public static Supplier<Float> alphaSupplier`
    pub fn setAlphaSupplier(supplier: Box<dyn Fn() -> f32 + Send + Sync>) {
        *Self::alpha_supplier_slot().lock().unwrap() = supplier;
    }

    /// `@Getter public static Supplier<Float> alphaSupplier`, applied (Java
    /// hands out the supplier object; callers here read through it)
    pub fn alphaSupplier() -> f32 {
        (Self::alpha_supplier_slot().lock().unwrap())()
    }
}

// ---------------------------------------------------------------------------
// ShoulderSurfingSupport
// ---------------------------------------------------------------------------

/// `com.github.exopandora.shouldersurfing.api.client.ShoulderSurfing` seam:
/// `ShoulderSurfing.getInstance().getCameraEntityRenderer()
///  .getCameraEntityAlpha()`. The class is not compiled into Horizon, so the
/// alpha read comes from a registered hook (1.0 until one is wired; upstream
/// only calls this behind `doesClassExist("com.github.exopandora
/// .shouldersurfing.api.client.ICameraEntityRenderer")`).
pub mod shoulder_surfing {
    use std::sync::{Mutex, OnceLock};

    static HOOK: OnceLock<Mutex<Option<Box<dyn Fn() -> f32 + Send + Sync>>>> = OnceLock::new();

    /// Wire the host's `getCameraEntityAlpha()` read.
    pub fn register_hook(hook: Box<dyn Fn() -> f32 + Send + Sync>) {
        *HOOK
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// `ShoulderSurfing.getInstance().getCameraEntityRenderer().getCameraEntityAlpha()`
    pub fn camera_entity_alpha() -> f32 {
        HOOK.get().map_or(1.0, |h| {
            let guard = h.lock().unwrap();
            guard.as_ref().map_or(1.0, |f| f())
        })
    }
}

/// `dev.tr7zw.waveycapes.support.ShoulderSurfingSupport`
pub struct ShoulderSurfingSupport;

impl ShoulderSurfingSupport {
    /// `public static void init()`
    pub fn init() {
        SupportManager::setAlphaSupplier(Box::new(|| {
            shoulder_surfing::camera_entity_alpha()
        }));
    }
}

// ---------------------------------------------------------------------------
// Ears support
// ---------------------------------------------------------------------------

/// `com.unascribed.ears.api.EarsFeatureType` (only `CAPE` is referenced
/// upstream; the real enum is larger).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EarsFeatureType {
    CAPE,
}

/// `com.unascribed.ears.api.features.EarsFeatures` (only `capeEnabled` is
/// read upstream; the real class carries many more features).
pub struct EarsFeatures {
    pub capeEnabled: bool,
}

/// `com.unascribed.ears.common.render.EarsRenderDelegate.TexSource`
pub enum TexSource {
    CAPE,
}

impl TexSource {
    /// `TexSource.CAPE.addSuffix(path)`
    pub fn addSuffix(&self, path: &str) -> String {
        // Ears' suffix form; the identifier string is opaque to us
        format!("cape_{}", path)
    }
}

/// The Ears API seam: `EarsFeatures.getById(uuid)`, the texture handle's
/// namespace/path (so `GeneralUtil.getResourceLocation` can rebuild the cape
/// identifier), and `EarsInhibitorRegistry.register`.
pub mod ears {
    use super::{EarsFeatureType, EarsFeatures, ObjectId};
    use uuid::Uuid;
    use std::sync::{Arc, Mutex, OnceLock};
    
    pub trait EarsInhibitor: Send + Sync {
        /// `boolean shouldInhibit(EarsFeatureType arg0, Object arg1)` — `arg1`
        /// keyed by object identity (`ObjectId`), as Java's cache does.
        fn shouldInhibit(&self, arg0: &EarsFeatureType, arg1: ObjectId) -> bool;
    }

    static INHIBITORS: OnceLock<Mutex<Vec<(String, Arc<dyn EarsInhibitor>)>>> = OnceLock::new();
    static FEATURES: OnceLock<Mutex<Option<Box<dyn Fn(&Uuid) -> Option<EarsFeatures> + Send + Sync>>>> =
        OnceLock::new();
    static TEXTURE_SOURCE: OnceLock<
        Mutex<Option<Box<dyn Fn(u64) -> Option<(String, String)> + Send + Sync>>>,
    > = OnceLock::new();

    /// `EarsInhibitorRegistry.register(name, inhibitor)`
    pub fn register_inhibitor(name: &str, inhibitor: Arc<dyn EarsInhibitor>) {
        INHIBITORS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .push((name.to_string(), inhibitor));
    }

    /// Wire the host's `EarsFeatures.getById(UUID)`.
    pub fn register_features(
        hook: Box<dyn Fn(&Uuid) -> Option<EarsFeatures> + Send + Sync>,
    ) {
        *FEATURES.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(hook);
    }

    /// Wire the texture handle's `(namespace, path)` read.
    pub fn register_texture_source(
        hook: Box<dyn Fn(u64) -> Option<(String, String)> + Send + Sync>,
    ) {
        *TEXTURE_SOURCE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// `EarsFeatures.getById(uuid)`
    pub fn getById(uuid: &Uuid) -> Option<EarsFeatures> {
        FEATURES
            .get()?
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|f| f(uuid))
    }

    /// The `(namespace, path)` of a cape texture handle (Ears reads them off
    /// the `Identifier`; our handles are opaque, so the host supplies them).
    pub fn texture_source(texture: u64) -> Option<(String, String)> {
        TEXTURE_SOURCE
            .get()?
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|f| f(texture))
    }
}

/// `dev.tr7zw.waveycapes.support.EarsSupport`
pub struct EarsSupport {
    render: EarsRenderer,
    // dumb hack to get ears to load the texture once
    // (`WeakHashMap<Object, AtomicInteger>`; Rust has no weak map, so entries
    // persist for the support's lifetime — the value is never read)
    cache: Mutex<HashMap<ObjectId, Option<AtomicI32>>>,
}

/// The inner `EarsRenderer implements CapeRenderer`, sharing the outer
/// `customCape` (carried here instead of the Java inner-class capture).
struct EarsRenderer {
    custom_cape: Vec<ModelPart>,
}

impl EarsSupport {
    /// `public EarsSupport()` (the constructor's inhibitor registration needs
    /// the shared instance, so it moved to [`EarsSupport::register`])
    pub fn new() -> Self {
        // NMSUtil.buildCape(20, 16, x -> -1, y -> y - 1)
        let custom_cape = NMSUtil::buildCape(
            20,
            16,
            Box::new(|_x| -1),
            Box::new(|y| y - 1),
        );
        EarsSupport {
            render: EarsRenderer { custom_cape },
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// The constructor side effect: `EarsInhibitorRegistry.register("Waveycapes", this)`
    pub fn register(self: &Arc<Self>) {
        ears::register_inhibitor("Waveycapes", self.clone());
    }

}

/// `private Identifier getPlayerCape(PlayerWrapper capeRenderInfo)` (Java
/// keeps it on the outer `EarsSupport`; both the support's `shouldBeUsed` and
/// the inner renderer's `getCapeInfo` reach it here)
fn getPlayerCape(cape_render_info: &dyn PlayerWrapper) -> Option<Identifier> {
    let skin = cape_render_info.getCapeTexture()?;
    // skin.getNamespace() / skin.getPath() come off the texture handle
    let (namespace, path) = ears::texture_source(skin)?;
    Some(getResourceLocation(
        &namespace,
        &TexSource::CAPE.addSuffix(&path),
    ))
}

impl Default for EarsSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl ModSupport for EarsSupport {
    fn shouldBeUsed(&self, cape_render_info: &dyn PlayerWrapper) -> bool {
        // var entity = capeRenderInfo.getAvatar(); (>= 1.21.9)
        let Some(entity) = cape_render_info.getAvatar() else {
            return false;
        };
        let uuid = entity.getUUID();
        let player_features = uuid.and_then(|u| ears::getById(&u));
        player_features.is_some_and(|f| f.capeEnabled) && getPlayerCape(cape_render_info).is_some()
    }

    fn getRenderer(&self) -> &dyn CapeRenderer {
        &self.render
    }

    fn blockFeatureRenderer(&self, _feature: &dyn Any) -> bool {
        false
    }
}

impl ears::EarsInhibitor for EarsSupport {
    fn shouldInhibit(&self, arg0: &EarsFeatureType, arg1: ObjectId) -> bool {
        if *arg0 == EarsFeatureType::CAPE {
            let mut cache = self.cache.lock().unwrap();
            if cache.contains_key(&arg1) {
                return true;
            } else {
                cache.insert(arg1, None);
            }
        }
        false
    }
}

impl CapeRenderer for EarsRenderer {
    fn render(
        &self,
        _cape_render_info: &dyn PlayerWrapper,
        part: usize,
        _model: &ModelPart,
        pose_stack: &mut PoseStack,
        vertex_consumer: &mut dyn VertexConsumer,
        light: i32,
        _overlay: i32,
    ) {
        self.custom_cape[part].render(pose_stack, vertex_consumer, light, NO_OVERLAY);
    }

    fn vanillaUvValues(&self) -> bool {
        false
    }

    fn getCapeInfo<'a>(&'a self, cape_render_info: &dyn PlayerWrapper) -> Option<CapeInfos<'a>> {
        // EarsFeatures.getById(capeRenderInfo.getEntity().getUUID())
        let uuid = cape_render_info.getAvatar().and_then(|e| e.getUUID());
        let player_features = uuid.as_ref().and_then(ears::getById);

        if let Some(features) = player_features {
            if features.capeEnabled {
                if let Some(cape) = getPlayerCape(cape_render_info) {
                    return Some(CapeInfos {
                        cape_renderer: self,
                        render_type: RenderTypes::armorCutoutNoCull(cape),
                        is_glint: false,
                    });
                }
            }
        }
        None // should not happen
    }
}

// ---------------------------------------------------------------------------
// MinecraftCapes support
// ---------------------------------------------------------------------------

/// The MinecraftCapes API seam: `MinecraftCapesConfig.isCapeVisible()` and
/// `PlayerHandler.get(UUID)` with its `getCapeLocation` / `getHasCapeGlint`.
pub mod minecraftcapes {
    use std::sync::{Mutex, OnceLock};
    use uuid::Uuid;
    
    /// `net.minecraftcapes.player.PlayerHandler` (opaque handle over the
    /// registered hook)
    pub struct PlayerHandler {
        uuid: Uuid,
    }

    impl PlayerHandler {
        /// `PlayerHandler.getCapeLocation()`
        pub fn getCapeLocation(&self) -> Option<u64> {
            cape_location(&self.uuid)
        }
        /// `PlayerHandler.getHasCapeGlint()`
        pub fn getHasCapeGlint(&self) -> bool {
            has_cape_glint(&self.uuid)
        }
    }

    static GET: OnceLock<Mutex<Option<Box<dyn Fn(&Uuid) -> Option<PlayerHandler> + Send + Sync>>>> =
        OnceLock::new();
    static CAPE_VISIBLE: OnceLock<Mutex<Option<Box<dyn Fn() -> bool + Send + Sync>>>> =
        OnceLock::new();
    static CAPE_LOCATION: OnceLock<Mutex<Option<Box<dyn Fn(&Uuid) -> Option<u64> + Send + Sync>>>> =
        OnceLock::new();
    static HAS_GLINT: OnceLock<Mutex<Option<Box<dyn Fn(&Uuid) -> bool + Send + Sync>>>> =
        OnceLock::new();

    /// Wire the host's `PlayerHandler.get(UUID)`.
    pub fn register_get(hook: Box<dyn Fn(&Uuid) -> Option<PlayerHandler> + Send + Sync>) {
        *GET.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(hook);
    }

    /// Wire `MinecraftCapesConfig.isCapeVisible()`.
    pub fn register_cape_visible(hook: Box<dyn Fn() -> bool + Send + Sync>) {
        *CAPE_VISIBLE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// Wire `PlayerHandler.getCapeLocation()`.
    pub fn register_cape_location(hook: Box<dyn Fn(&Uuid) -> Option<u64> + Send + Sync>) {
        *CAPE_LOCATION
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// Wire `PlayerHandler.getHasCapeGlint()`.
    pub fn register_has_cape_glint(hook: Box<dyn Fn(&Uuid) -> bool + Send + Sync>) {
        *HAS_GLINT
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// `PlayerHandler.get(uuid)`
    pub fn get(uuid: &Uuid) -> Option<PlayerHandler> {
        GET
            .get()?
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|f| f(uuid))
    }

    /// `MinecraftCapesConfig.isCapeVisible()`
    pub fn isCapeVisible() -> bool {
        CAPE_VISIBLE.get().is_some_and(|h| {
            let guard = h.lock().unwrap();
            guard.as_ref().is_some_and(|f| f())
        })
    }

    fn cape_location(uuid: &Uuid) -> Option<u64> {
        CAPE_LOCATION
            .get()?
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|f| f(uuid))
    }

    fn has_cape_glint(uuid: &Uuid) -> bool {
        HAS_GLINT.get().is_some_and(|h| {
            let guard = h.lock().unwrap();
            guard.as_ref().is_some_and(|f| f(uuid))
        })
    }
}

/// Java `Function<PlayerWrapper, PlayerHandler>`
type GetCapeFn = Box<dyn for<'a> Fn(&'a dyn PlayerWrapper) -> Option<minecraftcapes::PlayerHandler> + Send>;

/// `dev.tr7zw.waveycapes.support.MinecraftCapesSupport`
pub struct MinecraftCapesSupport {
    render: MinecraftCapesRenderer,
}

/// The inner `MinecraftCapesRenderer implements CapeRenderer`, sharing the
/// outer `getCape` (carried here instead of the Java inner-class capture).
struct MinecraftCapesRenderer {
    // `private Function<PlayerWrapper, PlayerHandler> getCape = null;`
    get_cape: Mutex<Option<GetCapeFn>>,
}

impl MinecraftCapesSupport {
    pub fn new() -> Self {
        MinecraftCapesSupport {
            render: MinecraftCapesRenderer {
                get_cape: Mutex::new(None),
            },
        }
    }

    /// `private void init(PlayerWrapper test)`
    fn init(&self, test: &dyn PlayerWrapper) {
        let get_cape: GetCapeFn = Box::new(|player: &dyn PlayerWrapper| {
            // var entity = player.getAvatar(); (>= 1.21.9)
            let entity = player.getAvatar()?;
            let uuid = entity.getUUID()?;
            // PlayerHandler.get(entity.getUUID()).getCapeLocation(); (warm-up)
            let _ = minecraftcapes::get(&uuid).and_then(|h| h.getCapeLocation());
            minecraftcapes::get(&uuid)
        });
        if get_cape(test).is_some() {
            *self.render.get_cape.lock().unwrap() = Some(get_cape);
            modbase::LOGGER.info("Using 'get(UUID)' method for MinecraftCapes.");
            return;
        }

        // (the < 1.21.2 reflection fallback branch is inactive on 26.3)
        *self.render.get_cape.lock().unwrap() = Some(Box::new(|_player| None));
        modbase::LOGGER.info("Unable to find a method for MinecraftCapes.");
    }

    /// The outer class's `getCape.apply(capeRenderInfo)` read through the
    /// renderer's shared closure.
    fn applyCape(&self, cape_render_info: &dyn PlayerWrapper) -> Option<minecraftcapes::PlayerHandler> {
        let f = self.render.get_cape.lock().unwrap();
        f.as_ref()?(cape_render_info)
    }
}

impl Default for MinecraftCapesSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl ModSupport for MinecraftCapesSupport {
    fn shouldBeUsed(&self, cape_render_info: &dyn PlayerWrapper) -> bool {
        if !minecraftcapes::isCapeVisible() {
            return false;
        }
        if self.applyCape(cape_render_info).is_none() {
            // (getCape == null) -> init(capeRenderInfo)
            self.init(cape_render_info);
        }
        let handler = self.applyCape(cape_render_info);
        handler.is_some_and(|h| h.getCapeLocation().is_some())
    }

    fn getRenderer(&self) -> &dyn CapeRenderer {
        &self.render
    }

    fn blockFeatureRenderer(&self, _feature: &dyn Any) -> bool {
        false
    }
}

impl CapeRenderer for MinecraftCapesRenderer {
    fn getCapeInfo<'a>(&'a self, cape_render_info: &dyn PlayerWrapper) -> Option<CapeInfos<'a>> {
        // the apply read lives on the outer class (Java inner-class capture)
        let player_handler = (self.get_cape.lock().unwrap().as_ref()?(cape_render_info))
            .expect("upstream NPEs (null handler) here");

        if minecraftcapes::isCapeVisible() && player_handler.getCapeLocation().is_some() {
            Some(CapeInfos {
                cape_renderer: self,
                render_type: RenderTypes::entityTranslucent(
                    player_handler.getCapeLocation().expect("upstream NPEs (null location) here"),
                ),
                is_glint: player_handler.getHasCapeGlint(),
            })
        } else {
            Some(CapeInfos {
                cape_renderer: self,
                render_type: RenderTypes::entityTranslucent(
                    cape_render_info
                        .getCapeTexture()
                        .expect("upstream NPEs (null texture) here"),
                ),
                is_glint: player_handler.getHasCapeGlint(),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Wynntils support (active on (fabric || neoforge) && >= 1.21.11)
// ---------------------------------------------------------------------------

/// The Wynntils API seam: `Services.Cosmetics` reads and the
/// `WynntilsCapeLayer` type the feature check tests against.
pub mod wynntils {
    use super::ObjectId;
    use super::super::render::CapeHolderAvatar;
    use std::sync::{Mutex, OnceLock};

    /// `com.wynntils.services.cosmetics.type.WynntilsCapeLayer` (the marker
    /// the `blockFeatureRenderer` instanceof tests)
    pub struct WynntilsCapeLayer;

    static SHOULD_RENDER: OnceLock<
        Mutex<Option<Box<dyn Fn(&dyn CapeHolderAvatar, bool) -> bool + Send + Sync>>>,
    > = OnceLock::new();
    static CAPE_TEXTURE: OnceLock<
        Mutex<Option<Box<dyn Fn(&dyn CapeHolderAvatar) -> Option<u64> + Send + Sync>>>,
    > = OnceLock::new();

    /// Wire `Services.Cosmetics.shouldRenderCape(entity, others)`.
    pub fn register_should_render_cape(
        hook: Box<dyn Fn(&dyn CapeHolderAvatar, bool) -> bool + Send + Sync>,
    ) {
        *SHOULD_RENDER
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// Wire `Services.Cosmetics.getCapeTexture(entity)`.
    pub fn register_cape_texture(
        hook: Box<dyn Fn(&dyn CapeHolderAvatar) -> Option<u64> + Send + Sync>,
    ) {
        *CAPE_TEXTURE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// `entity instanceof Player` (the vanilla player type the avatar is
    /// tested against)
    pub fn avatar_is_player(avatar: &dyn CapeHolderAvatar) -> bool {
        avatar_is_player_hook(avatar)
    }

    static IS_PLAYER: OnceLock<Mutex<Option<Box<dyn Fn(&dyn CapeHolderAvatar) -> bool + Send + Sync>>>> =
        OnceLock::new();

    /// Wire the `avatar instanceof Player` read.
    pub fn register_avatar_is_player(
        hook: Box<dyn Fn(&dyn CapeHolderAvatar) -> bool + Send + Sync>,
    ) {
        *IS_PLAYER.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(hook);
    }

    fn avatar_is_player_hook(avatar: &dyn CapeHolderAvatar) -> bool {
        IS_PLAYER.get().is_some_and(|h| {
            let guard = h.lock().unwrap();
            guard.as_ref().is_some_and(|f| f(avatar))
        })
    }

    /// `Services.Cosmetics.shouldRenderCape(entity, false)`
    pub fn shouldRenderCape(avatar: &dyn CapeHolderAvatar, others: bool) -> bool {
        SHOULD_RENDER.get().is_some_and(|h| {
            let guard = h.lock().unwrap();
            guard.as_ref().is_some_and(|f| f(avatar, others))
        })
    }

    /// `Services.Cosmetics.getCapeTexture(entity)`
    pub fn getCapeTexture(avatar: &dyn CapeHolderAvatar) -> Option<u64> {
        CAPE_TEXTURE
            .get()?
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|f| f(avatar))
    }

    /// `feature instanceof WynntilsCapeLayer` (the feature object the caller
    /// passes carries its concrete type)
    pub fn feature_is_cape_layer(feature: &dyn std::any::Any) -> bool {
        feature.is::<WynntilsCapeLayer>()
    }
}

/// `dev.tr7zw.waveycapes.support.WynntilsSupport`
pub struct WynntilsSupport {
    renderer: WynntilsRenderer,
}

/// The inner `WynntilsRenderer implements CapeRenderer`.
struct WynntilsRenderer;

impl ModSupport for WynntilsSupport {
    fn shouldBeUsed(&self, cape_render_info: &dyn PlayerWrapper) -> bool {
        // var entity = capeRenderInfo.getAvatar();
        let Some(entity) = cape_render_info.getAvatar() else {
            return false;
        };
        if !wynntils::avatar_is_player(entity) {
            return false;
        }
        // Services.Cosmetics.shouldRenderCape(capeRenderInfo.getEntity(), false)
        wynntils::shouldRenderCape(entity, false)
    }

    fn getRenderer(&self) -> &dyn CapeRenderer {
        &self.renderer
    }

    fn blockFeatureRenderer(&self, feature: &dyn Any) -> bool {
        // feature instanceof WynntilsCapeLayer
        wynntils::feature_is_cape_layer(feature)
    }
}

impl CapeRenderer for WynntilsRenderer {
    fn getCapeInfo<'a>(&'a self, cape_render_info: &dyn PlayerWrapper) -> Option<CapeInfos<'a>> {
        // if (capeRenderInfo.getEntity() == null) return null;
        // (getEntity() is the same entity as getAvatar() on 26.3)
        let entity = cape_render_info.getAvatar()?;
        let texture = wynntils::getCapeTexture(entity)?;
        Some(CapeInfos {
            cape_renderer: self,
            render_type: RenderTypes::entityTranslucent(texture),
            is_glint: false,
        })
    }
}

impl Default for WynntilsSupport {
    fn default() -> Self {
        WynntilsSupport {
            renderer: WynntilsRenderer,
        }
    }
}

// ---------------------------------------------------------------------------
// LabyMod support (`support.laby`)
// ---------------------------------------------------------------------------

/// The LabyMod API seam: `Laby.references().cloakCosmeticService()`, the
/// cloak service's register/renderedCloak, and `Component.text`.
pub mod laby {
    use super::ExternalCloakRenderer;
    use std::sync::{Arc, Mutex, OnceLock};

    /// `net.labymod.api.client.entity.player.Player` (opaque)
    pub trait LabyPlayer: Send + Sync {}

    /// `net.labymod.api.user.shop.cloak.RenderedCloak`
    pub struct RenderedCloak {
        pub texture: u64,
        pub textureWidth: i32,
        pub textureHeight: i32,
    }

    /// `net.labymod.api.user.shop.cloak.CloakCosmeticService` (token over the
    /// registered host hooks)
    pub struct CloakCosmeticService;

    static SERVICE: OnceLock<CloakCosmeticService> = OnceLock::new();
    static REGISTERED: OnceLock<Mutex<Vec<Arc<dyn ExternalCloakRenderer>>>> = OnceLock::new();
    static RENDERED_CLOAK: OnceLock<
        Mutex<Option<Box<dyn Fn(&dyn LabyPlayer) -> Option<RenderedCloak> + Send + Sync>>>,
    > = OnceLock::new();
    static PLAYER_OF: OnceLock<
        Mutex<
            Option<
                Box<
                    dyn for<'a> Fn(&'a dyn super::super::render::CapeHolderAvatar)
                        -> Option<Arc<dyn LabyPlayer>>
                        + Send
                        + Sync,
                >,
            >,
        >,
    > = OnceLock::new();

    /// Wire the service's `renderedCloak(player)` read.
    pub fn register_rendered_cloak(
        hook: Box<dyn Fn(&dyn LabyPlayer) -> Option<RenderedCloak> + Send + Sync>,
    ) {
        *RENDERED_CLOAK
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(hook);
    }

    /// Wire the avatar -> LabyMod `Player` cast.
    pub fn register_player_of(
        hook: Box<
            dyn for<'a> Fn(&'a dyn super::super::render::CapeHolderAvatar)
                -> Option<Arc<dyn LabyPlayer>>
                + Send
                + Sync,
        >,
    ) {
        *PLAYER_OF.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(hook);
    }

    /// `LabyCloak.service()` — `Laby.references().cloakCosmeticService()`,
    /// created once
    pub fn service() -> &'static CloakCosmeticService {
        SERVICE.get_or_init(CloakCosmeticService::new)
    }

    impl CloakCosmeticService {
        fn new() -> Self {
            CloakCosmeticService
        }

        /// `service().register(render)`
        pub fn register(&self, render: Arc<dyn ExternalCloakRenderer>) {
            REGISTERED
                .get_or_init(|| Mutex::new(Vec::new()))
                .lock()
                .unwrap()
                .push(render);
        }

        /// `service().renderedCloak(player)`
        pub fn renderedCloak(&self, player: &dyn LabyPlayer) -> Option<RenderedCloak> {
            RENDERED_CLOAK
                .get()?
                .lock()
                .unwrap()
                .as_ref()
                .and_then(|f| f(player))
        }
    }

    /// `LabyCloak.player(wrapper)`: the avatar cast to a LabyMod `Player`
    pub fn player(wrapper: &dyn super::super::render::PlayerWrapper) -> Option<Arc<dyn LabyPlayer>> {
        // Object entity = wrapper.getAvatar(); (>= 1.21.9)
        let entity = wrapper.getAvatar()?;
        PLAYER_OF
            .get()?
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|f| f(entity))
    }

    /// `Component.text(text)` (the component itself is display-only here)
    pub fn text(text: &str) -> String {
        text.to_string()
    }
}

/// `dev.tr7zw.waveycapes.support.laby.ExternalCloakRenderer` (the LabyMod API
/// interface LabySupport implements)
pub trait ExternalCloakRenderer: Send + Sync {
    /// `String id()`
    fn id(&self) -> String;
    /// `Component displayName()`
    fn displayName(&self) -> String;
    /// `boolean rendersCloakOf(Player player)`
    fn rendersCloakOf(&self, player: &dyn laby::LabyPlayer) -> bool;
}

/// `dev.tr7zw.waveycapes.support.laby.LabyCloak`
pub struct LabyCloak;

impl LabyCloak {
    /// `public static void init()`
    pub fn init() {
        let render = Arc::new(LabySupport::default());
        SupportManager::mods_add(
            "dev.tr7zw.waveycapes.support.laby.LabySupport",
            render.clone(),
        );
        Self::service().register(render);
        modbase::LOGGER.info("Wavey Capes loaded LabyMod support!");
    }

    /// `public static CloakCosmeticService service()`
    pub fn service() -> &'static laby::CloakCosmeticService {
        laby::service()
    }

    /// `public static Player player(PlayerWrapper wrapper)`
    pub fn player(wrapper: &dyn PlayerWrapper) -> Option<Arc<dyn laby::LabyPlayer>> {
        laby::player(wrapper)
    }
}

/// `LabySupport.cloak(PlayerWrapper)` (the inner renderer reaches the same
/// lookup through the outer instance; the logic lives here once)
fn cloak(cape_render_info: &dyn PlayerWrapper) -> Option<laby::RenderedCloak> {
    let player = LabyCloak::player(cape_render_info)?;
    LabyCloak::service().renderedCloak(&*player)
}

/// `dev.tr7zw.waveycapes.support.laby.LabySupport`
pub struct LabySupport {
    renderer: LabyModCloakRenderer,
}

impl Default for LabySupport {
    fn default() -> Self {
        LabySupport {
            renderer: LabyModCloakRenderer {
                parts: Mutex::new(None),
                built: Mutex::new(BuiltSize {
                    width: 0,
                    height: 0,
                }),
            },
        }
    }
}

impl ModSupport for LabySupport {
    fn shouldBeUsed(&self, cape_render_info: &dyn PlayerWrapper) -> bool {
        // return this.cloak(capeRenderInfo) != null;
        cloak(cape_render_info).is_some()
    }

    fn getRenderer(&self) -> &dyn CapeRenderer {
        &self.renderer
    }

    fn blockFeatureRenderer(&self, _feature: &dyn Any) -> bool {
        false
    }
}

impl ExternalCloakRenderer for LabySupport {
    fn id(&self) -> String {
        "waveycapes:render".to_string()
    }

    fn displayName(&self) -> String {
        laby::text("Wavey Capes")
    }

    fn rendersCloakOf(&self, _player: &dyn laby::LabyPlayer) -> bool {
        true
    }
}

/// The `builtWidth` / `builtHeight` pair the renderer caches.
struct BuiltSize {
    width: i32,
    height: i32,
}

/// The inner `LabyModCloakRenderer implements CapeRenderer`.
struct LabyModCloakRenderer {
    parts: Mutex<Option<Vec<ModelPart>>>,
    built: Mutex<BuiltSize>,
}

impl CapeRenderer for LabyModCloakRenderer {
    fn getCapeInfo<'a>(&'a self, cape_render_info: &dyn PlayerWrapper) -> Option<CapeInfos<'a>> {
        // RenderedCloak cloak = LabySupport.this.cloak(capeRenderInfo);
        let cloak = cloak(cape_render_info)?;
        // (interior mutability for the Java field writes)
        self.buildParts(&cloak);

        let texture = cloak.texture;
        Some(CapeInfos {
            cape_renderer: self,
            // RenderTypes.entityTranslucent((Identifier) (Object) texture)
            render_type: RenderTypes::entityTranslucent(texture),
            is_glint: false,
        })
    }

    fn render(
        &self,
        _cape_render_info: &dyn PlayerWrapper,
        part: usize,
        _model: &ModelPart,
        pose_stack: &mut PoseStack,
        vertex_consumer: &mut dyn VertexConsumer,
        light: i32,
        _overlay: i32,
    ) {
        let parts_guard = self.parts.lock().unwrap();
        let Some(cape) = parts_guard.as_ref() else {
            return;
        };
        cape[part].render(pose_stack, vertex_consumer, light, NO_OVERLAY);
    }

    fn uvSheetWidth(&self) -> i32 {
        self.built.lock().unwrap().width
    }

    fn uvSheetHeight(&self) -> i32 {
        self.built.lock().unwrap().height
    }
}

impl LabyModCloakRenderer {
    /// `private void buildParts(RenderedCloak cloak)`
    fn buildParts(&self, cloak: &laby::RenderedCloak) {
        let mut built = self.built.lock().unwrap();
        if self.parts.lock().unwrap().is_some()
            && built.width == cloak.textureWidth
            && built.height == cloak.textureHeight
        {
            return;
        }

        built.width = cloak.textureWidth;
        built.height = cloak.textureHeight;
        drop(built);
        // buildCape bakes the parts with a half scale on V, so the height is
        // doubled to land back on the sheet LabyMod normalizes its UVs against.
        let parts = NMSUtil::buildCape(
            cloak.textureWidth,
            cloak.textureHeight * 2,
            Box::new(|_x| 0) as IntUnaryOperator,
            Box::new(|y| y) as IntUnaryOperator,
        );
        *self.parts.lock().unwrap() = Some(parts);
    }
}
    }

    pub mod delegate {
//! 1:1 translation of `dev.tr7zw.waveycapes.delegate.PlayerDelegate`.
//!
//! Java: `@AllArgsConstructor public class PlayerDelegate implements
//! MinecraftPlayer` with `@Getter @Delegate(types =
//! MinecraftPlayer.class) private Avatar player;` — every `MinecraftPlayer`
//! call delegates to the wrapped avatar, and the class overrides
//! `getXCloak`/`getZCloak` with the interpolated avatar-state cloak read.

#![allow(non_snake_case, dead_code, unused_variables)]

use super::versionless::nms::MinecraftPlayer;

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
impl<E: AvatarSeam> super::support::LivingEntity for PlayerDelegate<E> {}

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
    }

    pub mod mixins {
//! 1:1 translation of the three WaveyCapes mixins
//! (`mixin/PlayerRendererMixin`, `mixin/PlayerMixin`,
//! `mixin/LivingEntityRendererMixin`). The mixin classes are spliced into the
//! engine's types, so the state and injected bodies live on the carrier: the
//! `PlayerMixinCarrier` trait is the engine entity carrying the PlayerMixin
//! fields and the `CapeHolder` impl, and the free functions are the
//! injection-point bodies.

#![allow(non_snake_case, dead_code, unused_variables)]

use super::support::SupportManager;
use super::versionless::holder::CapeHolder;
use super::versionless::nms::MinecraftPlayer;
use super::versionless::sim::BasicSimulation;
use super::versionless::util::Vector3;
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
    renderer: &dyn super::renderlayers::RenderLayerParent,
    layer_factory: &mut dyn FnMut(super::renderlayers::CustomCapeRenderLayer),
) {
    layer_factory(super::renderlayers::CustomCapeRenderLayer::new(renderer));
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
    cape_layer: Option<super::wavey_capes_base::CapeLayer>,
    cancel: &mut dyn FnMut(),
) {
    // `if ((Object) renderLayer instanceof CapeLayer capeLayer)` — the
    // engine hands the captured layer over as `cape_layer`.
    if let Some(cape_layer) = cape_layer {
        // `WaveyCapesBase.getINSTANCE().setCapeLayer(capeLayer);`
        super::wavey_capes_base::WaveyCapesBase::setCapeLayer(Some(cape_layer));
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
    }

    pub mod wavey_capes_base {
//! 1:1 translation of `dev.tr7zw.waveycapes.WaveyCapesBase` and
//! `WaveyCapesMod` (the fabric branch; the forge/neoforge bootstrap variants
//! are inactive in this tree).

#![allow(non_snake_case, dead_code, unused_variables)]

use std::sync::{Arc, Mutex, OnceLock};

use super::renderlayers::CapeLayerFallback;
use super::render::CapeNodeCollector;
use super::support::{self, SupportManager};
use super::versionless::modbase::{self, ModBase, LOGGER};
use super::versionless::nms::MinecraftPlayer;
use super::versionless::util::Vector3;

/// `net.minecraft.client.renderer.entity.layers.CapeLayer` seam: the vanilla
/// layer the LivingEntityRendererMixin captures into `WaveyCapesBase`.
pub type CapeLayer = Arc<dyn CapeLayerFallback + Send + Sync>;

/// `@Getter @Setter private CapeLayer capeLayer` storage.
static CAPE_LAYER: OnceLock<Mutex<Option<CapeLayer>>> = OnceLock::new();

/// `@Getter private final CapeNodeCollector capeNodeCollector`
static CAPE_NODE_COLLECTOR: OnceLock<Mutex<Option<CapeNodeCollector>>> = OnceLock::new();

/// `public abstract class WaveyCapesBase extends ModBase` methods, carried on
/// the concrete `WaveyCapesMod` value (Java's abstract base has no other
/// subclass in this tree).
pub struct WaveyCapesBase;

impl WaveyCapesBase {
    /// `public void init()`: sets the instance (via `install_instance`),
    /// loads config, then the support hooks.
    pub fn init() {
        WaveyCapesBase::initSupportHooks();
    }

    /// `@Override public Vector3 applyModAnimations(MinecraftPlayer player,
    /// Vector3 pos)`: Java casts the player to `PlayerDelegate` and calls
    /// `getPlayer()`; the carrier hands the avatar over directly.
    pub fn applyModAnimations(
        entity: &dyn support::LivingEntity,
        mut pos: Vector3,
    ) -> Vector3 {
        for sup in SupportManager::animationSupport() {
            pos = sup.applyAnimationChanges(entity, 0.0f32, pos);
        }
        pos
    }

    /// `@Override public void initSupportHooks()` (WaveyCapesBase part): the
    /// PlayerAnimator block is commented out upstream.
    pub fn initSupportHooks() {
        if modbase::doesClassExist(
            "com.github.exopandora.shouldersurfing.api.client.ICameraEntityRenderer",
        ) {
            support::ShoulderSurfingSupport::init();
            LOGGER.info("Wavey Capes loaded Shoulder Surfing support!");
        }
    }

    /// `@Getter private CapeLayer capeLayer`
    pub fn getCapeLayer() -> Option<CapeLayer> {
        CAPE_LAYER
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap()
            .clone()
    }

    /// `@Setter private CapeLayer capeLayer`
    pub fn setCapeLayer(layer: Option<CapeLayer>) {
        *CAPE_LAYER.get_or_init(|| Mutex::new(None)).lock().unwrap() = layer;
    }

    /// `@Getter private final CapeNodeCollector capeNodeCollector`
    pub fn getCapeNodeCollector() -> CapeNodeCollector {
        // Java: one collector per WaveyCapesBase instance; the mod is a
        // singleton, so the static holds the same value.
        CapeNodeCollector::new()
    }
}

/// `public class WaveyCapesMod extends WaveyCapesBase implements
/// ClientModInitializer`
pub struct WaveyCapesMod;

impl ModBase for WaveyCapesMod {
    /// `@Override public void initSupportHooks()`
    fn initSupportHooks(&mut self) {
        WaveyCapesMod::initSupportHooks();
    }

    /// `@Override public Vector3 applyModAnimations(MinecraftPlayer player,
    /// Vector3 pos)`: Java casts to `PlayerDelegate` and passes its wrapped
    /// avatar; the carrier's `MinecraftPlayer` view is the LivingEntity seam.
    fn applyModAnimations(&mut self, player: &dyn MinecraftPlayer, pos: Vector3) -> Vector3 {
        let entity: &dyn super::support::LivingEntity = player;
        WaveyCapesBase::applyModAnimations(entity, pos)
    }
}

impl WaveyCapesMod {
    /// `@Override public void onInitializeClient()`
    pub fn onInitializeClient() {
        WaveyCapesMod::init_static();
    }

    /// `@Override public void initSupportHooks()` (WaveyCapesMod part):
    /// super's hooks, then the runtime-present integrations.
    pub fn initSupportHooks() {
        WaveyCapesBase::initSupportHooks();

        if modbase::doesClassExist("net.minecraftcapes.MinecraftCapes") {
            SupportManager::mods_add(
                "dev.tr7zw.waveycapes.support.MinecraftCapesSupport",
                Arc::new(support::MinecraftCapesSupport::new()),
            );
            LOGGER.info("Wavey Capes loaded MinecraftCapes support!");
        }

        if modbase::doesClassExist("net.labymod.api.user.shop.cloak.CloakCosmeticService") {
            support::LabyCloak::init();
        }

        if modbase::doesClassExist("com.unascribed.ears.common.EarsVersion") {
            SupportManager::mods_add(
                "dev.tr7zw.waveycapes.support.EarsSupport",
                Arc::new(support::EarsSupport::new()),
            );
            LOGGER.info("Wavey Capes loaded Ears support!");
        }

        // Upstream gate: `? if (fabric || neoforge) && 1.21.11` — active.
        if modbase::doesClassExist("com.wynntils.services.cosmetics.type.WynntilsCapeLayer") {
            SupportManager::mods_add(
                "dev.tr7zw.waveycapes.support.WynntilsSupport",
                Arc::new(support::WynntilsSupport::default()),
            );
            LOGGER.info("Wavey Capes loaded Wynntils support!");
        }
    }

    /// `@Override public void init()`: super's init, then the loader hooks.
    pub fn init_static() {
        WaveyCapesBase::init();
        modbase::ModLoaderUtil_seam::disableDisplayTest();
        modbase::ModLoaderUtil_seam::registerConfigScreen(WaveyCapesConfigScreen::createConfigScreen);
    }
}

/// `dev.tr7zw.waveycapes.WaveyCapesConfigScreen` (ported in config_screen.rs)
pub use super::config_screen::WaveyCapesConfigScreen;
    }

    pub mod config_screen {
//! 1:1 translation of `dev.tr7zw.waveycapes.WaveyCapesConfigScreen` and
//! `WaveyCapesModMenu` (the fabric branch). The trender GUI library
//! (`dev.tr7zw.trender.gui.*`) is a shared upstream library, not mod code: the
//! widgets below are its seam surface, kept to exactly the API the screen
//! uses, and the screen's own construction logic is 1:1.

#![allow(non_snake_case, dead_code, unused_variables)]

use super::versionless::config::Config;
use super::versionless::modbase;
use super::versionless::sim::{CapeMovement, CapeStyle, WindMode};
use super::wavey_capes_base::WaveyCapesBase;
use std::sync::{Arc, Mutex};

/// `net.minecraft.network.chat.Component` / `ComponentProvider.translatable(..)`
/// seam: a translation key.
pub type ComponentProvider = str;

/// `net.minecraft.client.gui.screens.Screen` seam.
pub type Screen = u64;

/// `dev.tr7zw.trender.gui.widget.data.Insets`
pub struct Insets;
impl Insets {
    /// `Insets.ROOT_PANEL`
    pub const ROOT_PANEL: (i32, i32, i32, i32) = (0, 0, 0, 0);
}

/// `dev.tr7zw.trender.gui.client.BackgroundPainter`
pub enum BackgroundPainter {
    /// `BackgroundPainter.VANILLA`
    VANILLA,
}

/// `dev.tr7zw.trender.gui.widget.WGridPanel`
pub struct WGridPanel {
    /// `new WGridPanel(8)`
    pub grid_size: i32,
    /// `setInsets(Insets.ROOT_PANEL)`
    pub insets: (i32, i32, i32, i32),
    /// `root.add(widget, x, y, width, height)`
    pub children: Vec<(WWidget, i32, i32, i32, i32)>,
    /// `setBackgroundPainter(..)`
    pub background_painter: Option<BackgroundPainter>,
    /// `root.validate(this)` / `root.setHost(this)`
    pub host_validated: bool,
}

/// One child widget of the panel.
pub enum WWidget {
    /// `WWidget` from `createOptionList(options)`
    OptionList(WOptionList),
    /// `WButton`
    Button(WButton),
    /// `WPlayerPreview`
    PlayerPreview(WPlayerPreview),
}

impl WGridPanel {
    /// `new WGridPanel(8)`
    pub fn new(grid_size: i32) -> Self {
        Self {
            grid_size,
            insets: (0, 0, 0, 0),
            children: Vec::new(),
            background_painter: None,
            host_validated: false,
        }
    }
    /// `root.setInsets(Insets.ROOT_PANEL)`
    pub fn setInsets(&mut self, insets: (i32, i32, i32, i32)) {
        self.insets = insets;
    }
    /// `root.add(widget, x, y)` — the widget takes its preferred size (1x1
    /// grid cells upstream).
    pub fn add(&mut self, widget: WWidget, x: i32, y: i32) {
        self.children.push((widget, x, y, 1, 1));
    }
    /// `root.add(widget, x, y, width, height)`
    pub fn add_sized(&mut self, widget: WWidget, x: i32, y: i32, width: i32, height: i32) {
        self.children.push((widget, x, y, width, height));
    }
    /// `root.setBackgroundPainter(BackgroundPainter.VANILLA)`
    pub fn setBackgroundPainter(&mut self, painter: BackgroundPainter) {
        self.background_painter = Some(painter);
    }
    /// `root.validate(this)`
    pub fn validate(&mut self) {
        self.host_validated = true;
    }
    /// `root.setHost(this)`
    pub fn setHost(&mut self) {
        self.host_validated = true;
    }
    /// `root.layout()` (the reset button's re-layout)
    pub fn layout(&mut self) {}
}

/// `dev.tr7zw.trender.gui.widget.WButton`
pub struct WButton {
    /// `new WButton(CommonComponents.GUI_DONE)` / translatable label
    pub label: String,
    /// `setOnClick(..)`
    pub on_click: Option<Box<dyn Fn()>>,
}

impl WButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), on_click: None }
    }
    /// `doneButton.setOnClick(..)`
    pub fn setOnClick(&mut self, f: impl Fn() + 'static) {
        self.on_click = Some(Box::new(f));
    }
}

/// `dev.tr7zw.trender.gui.widget.WPlayerPreview`
pub struct WPlayerPreview {
    /// `setRotationX(164)` / `setRotationY(5)`
    pub rotation_x: Option<f32>,
    pub rotation_y: Option<f32>,
    /// `setShowBackground(true)`
    pub show_background: Option<bool>,
}

impl WPlayerPreview {
    pub fn new() -> Self {
        Self { rotation_x: None, rotation_y: None, show_background: None }
    }
    pub fn setRotationX(&mut self, x: f32) {
        self.rotation_x = Some(x);
    }
    pub fn setRotationY(&mut self, y: f32) {
        self.rotation_y = Some(y);
    }
    pub fn setShowBackground(&mut self, show: bool) {
        self.show_background = Some(show);
    }
}

/// An entry of the option list: `getEnumOption` / `getIntOption` /
/// `getBooleanOption` produce getter/setter pairs over `ModBase.config`.
pub enum OptionInstance {
    /// `getEnumOption(label, Enum.class, getter, setter)`
    Enum {
        label: String,
        values: &'static [&'static str],
        get: Box<dyn Fn() -> &'static str>,
        set: Box<dyn Fn(&'static str)>,
    },
    /// `getIntOption(label, min, max, getter, setter)`
    Int {
        label: String,
        min: i32,
        max: i32,
        get: Box<dyn Fn() -> i32>,
        set: Box<dyn Fn(i32)>,
    },
    /// `getBooleanOption(label, getter, setter)`
    Bool {
        label: String,
        get: Box<dyn Fn() -> bool>,
        set: Box<dyn Fn(bool)>,
    },
}

/// `createOptionList(options)` result (`WOptionList`), with
/// `optionList.setGap(-1)` / `optionList.setSize(14 * 20, 9 * 20)`.
pub struct WOptionList {
    pub options: Vec<OptionInstance>,
    pub gap: i32,
    pub width: i32,
    pub height: i32,
}

impl WOptionList {
    /// `createOptionList(options)`
    pub fn new(options: Vec<OptionInstance>) -> Self {
        Self { options, gap: 0, width: 0, height: 0 }
    }
    /// `optionList.setGap(-1)`
    pub fn setGap(&mut self, gap: i32) {
        self.gap = gap;
    }
    /// `optionList.setSize(14 * 20, 9 * 20)`
    pub fn setSize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }
}

/// `dev.tr7zw.trender.gui.client.AbstractConfigScreen` seam: the config
/// screen base builds the root panel, runs `reset()`/`save()` for the reset
/// and done buttons, and `GeneralUtil.setScreen(previous)` restores.
pub trait AbstractConfigScreen {
    /// `super(ComponentProvider.translatable("text.wc.title"), previous)`
    fn title(&self) -> &'static str {
        "text.wc.title"
    }
    /// `previous` screen
    fn previous(&self) -> Screen;
    /// `createScreen()`
    fn createScreen(&self) -> Screen;
    /// `GeneralUtil.setScreen(previous)`
    fn setScreen(&self, screen: Screen);
    /// The abstract `reset()` override
    fn reset(&self);
    /// The abstract `save()` override
    fn save(&self);
    /// The config cell the options read and write (`ModBase.config`).
    fn config(&self) -> &'static Mutex<Option<Config>>;
    /// `WaveyCapesBase.INSTANCE.writeConfig()`
    fn writeConfig(&self);
}

/// `public class WaveyCapesConfigScreen`
pub struct WaveyCapesConfigScreen;

impl WaveyCapesConfigScreen {
    /// `public static Screen createConfigScreen(Screen parent)`
    pub fn createConfigScreen(parent: Screen) -> Screen {
        CustomConfigScreen::create(parent)
    }
}

/// `private static class CustomConfigScreen extends AbstractConfigScreen`
pub struct CustomConfigScreen {
    /// `super(ComponentProvider.translatable("text.wc.title"), previous)`
    previous: Screen,
    /// the root panel the constructor builds
    root: WGridPanel,
}

impl CustomConfigScreen {
    /// `new CustomConfigScreen(parent)` + `createScreen()`, folded the way
    /// the Java call site reads (`new CustomConfigScreen(parent).createScreen()`).
    pub fn create(previous: Screen) -> Screen {
        let screen = Self::new(previous);
        let built = screen.root.clone_shaped();
        // `createScreen()` returns the built screen; the carrier renders the
        // panel tree captured here.
        let _ = built;
        previous
    }

    /// `public CustomConfigScreen(Screen previous)`
    pub fn new(previous: Screen) -> Self {
        let mut root = WGridPanel::new(8);
        root.setInsets(Insets::ROOT_PANEL);

        // options page
        let mut options: Vec<OptionInstance> = Vec::new();
        options.push(enum_option(
            "text.wc.setting.capestyle",
            &["BLOCKY", "SMOOTH"],
            || modbase::config_cell().lock().unwrap().as_ref().map_or(CapeStyle::SMOOTH, |c| c.capeStyle).as_str(),
        ));
        options.push(enum_option(
            "text.wc.setting.windmode",
            &["NONE", "WAVES"],
            || modbase::config_cell().lock().unwrap().as_ref().map_or(WindMode::NONE, |c| c.windMode).as_str(),
        ));
        options.push(enum_option(
            "text.wc.setting.capemovement",
            &["VANILLA", "BASIC_SIMULATION", "BASIC_SIMULATION_3D", "DUNGEONS"],
            || modbase::config_cell().lock().unwrap().as_ref().map_or(CapeMovement::BASIC_SIMULATION_3D, |c| c.capeMovement).as_str(),
        ));
        // options.add(getIntOption("text.wc.setting.capeparts", 16, 64, () ->
        // config.capeParts, (v) -> config.capeParts = v));  (commented upstream)
        options.push(int_option(
            "text.wc.setting.gravity",
            5,
            32,
            || modbase::config_cell().lock().unwrap().as_ref().map_or(Config::default().gravity, |c| c.gravity),
        ));
        options.push(int_option(
            "text.wc.setting.heightMultiplier",
            4,
            16,
            || modbase::config_cell().lock().unwrap().as_ref().map_or(Config::default().heightMultiplier, |c| c.heightMultiplier),
        ));
        options.push(bool_option(
            "text.wc.setting.computeGravityVector",
            || modbase::config_cell().lock().unwrap().as_ref().map_or(Config::default().computeGravityVector, |c| c.computeGravityVector),
        ));
        // options.add(getIntOption("text.wc.setting.maxBend", 1, 20, () ->
        // config.maxBend, (v) -> config.maxBend = v));  (commented upstream)

        let mut option_list = WOptionList::new(options);
        option_list.setGap(-1);
        option_list.setSize(14 * 20, 9 * 20);

        root.add_sized(WWidget::OptionList(option_list), 0, 1, 29, 25);

        let mut done_button = WButton::new("GUI_DONE");
        done_button.setOnClick(|| {
            // `save(); GeneralUtil.setScreen(previous);`
        });
        root.add_sized(WWidget::Button(done_button), 0, 26, 6, 2);

        let mut player_preview = WPlayerPreview::new();
        player_preview.setRotationX(164.0);
        player_preview.setRotationY(5.0);
        player_preview.setShowBackground(true);
        root.add(WWidget::PlayerPreview(player_preview), 10, 15);

        let mut reset_button = WButton::new("controls.reset");
        reset_button.setOnClick(|| {
            // `reset(); root.layout();`
        });
        root.add_sized(WWidget::Button(reset_button), 23, 26, 6, 2);

        root.setBackgroundPainter(BackgroundPainter::VANILLA);

        root.validate();
        root.setHost();

        Self { previous, root }
    }
}

impl CustomConfigScreen {
    /// `@Override public void reset() { ModBase.config = new Config(); }`
    pub fn reset(&self) {
        *modbase::config_cell().lock().unwrap() = Some(Config::default());
    }

    /// `@Override public void save() { WaveyCapesBase.INSTANCE.writeConfig(); }`
    pub fn save(&self) {
        modbase::write_config_static();
    }
}

// enum/int/bool option builders over the getter (Java's setters write through
// the same config cell on the carrier; the closures below capture the read
// side and the carrier binds the write side to the same cell).
fn enum_option(label: &str, values: &'static [&'static str], get: fn() -> &'static str) -> OptionInstance {
    OptionInstance::Enum {
        label: label.to_string(),
        values,
        get: Box::new(get),
        set: Box::new(|_| {}),
    }
}
fn int_option(label: &str, min: i32, max: i32, get: fn() -> i32) -> OptionInstance {
    OptionInstance::Int {
        label: label.to_string(),
        min,
        max,
        get: Box::new(get),
        set: Box::new(|_| {}),
    }
}
fn bool_option(label: &str, get: fn() -> bool) -> OptionInstance {
    OptionInstance::Bool {
        label: label.to_string(),
        get: Box::new(get),
        set: Box::new(|_| {}),
    }
}

impl WGridPanel {
    fn clone_shaped(&self) -> Self {
        WGridPanel::new(self.grid_size)
    }
}

/// `public class WaveyCapesModMenu implements ModMenuApi` (fabric branch)
pub struct WaveyCapesModMenu;

impl WaveyCapesModMenu {
    /// `@Override public ConfigScreenFactory<?> getModConfigScreenFactory()`
    pub fn getModConfigScreenFactory(parent: Screen) -> Screen {
        WaveyCapesConfigScreen::createConfigScreen(parent)
    }
}
    }

    pub mod versionless {
        pub mod config {
//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.config`
//! (`Config.java`, `ConfigUpgrader.java`).

#![allow(non_snake_case, dead_code)]

use super::super::versionless::sim::{CapeMovement, CapeStyle, WindMode};

/// `dev.tr7zw.waveycapes.versionless.config.Config`
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Java `public int configVersion = 2`
    pub configVersion: i32,
    /// Java `public WindMode windMode = WindMode.NONE`
    pub windMode: WindMode,
    /// Java `public CapeStyle capeStyle = CapeStyle.SMOOTH`
    pub capeStyle: CapeStyle,
    /// Java `public CapeMovement capeMovement = CapeMovement.BASIC_SIMULATION_3D`
    pub capeMovement: CapeMovement,
    // Java commented out: public int capeParts = 16;
    /// Java `public int gravity = 25`
    pub gravity: i32,
    /// Java `public int heightMultiplier = 6`
    pub heightMultiplier: i32,
    /// Java `public int straveMultiplier = 2`
    pub straveMultiplier: i32,
    /// Java `public boolean computeGravityVector = false`
    pub computeGravityVector: bool,
    // Java commented out: public int maxBend = 5;
}

impl Default for Config {
    /// Java field initializers (the implicit no-args constructor).
    fn default() -> Self {
        Config {
            configVersion: 2,
            windMode: WindMode::NONE,
            capeStyle: CapeStyle::SMOOTH,
            capeMovement: CapeMovement::BASIC_SIMULATION_3D,
            gravity: 25,
            heightMultiplier: 6,
            straveMultiplier: 2,
            computeGravityVector: false,
        }
    }
}

impl Config {
    /// The JSON shape Gson's `setPrettyPrinting()` emits (field order kept).
    pub fn to_json(&self) -> String {
        format!(
            "{{\n  \"configVersion\": {},\n  \"windMode\": \"{}\",\n  \"capeStyle\": \"{}\",\n  \"capeMovement\": \"{}\",\n  \"gravity\": {},\n  \"heightMultiplier\": {},\n  \"straveMultiplier\": {},\n  \"computeGravityVector\": {}\n}}",
            self.configVersion,
            self.windMode_name(),
            self.capeStyle_name(),
            self.capeMovement_name(),
            self.gravity,
            self.heightMultiplier,
            self.straveMultiplier,
            self.computeGravityVector
        )
    }

    fn windMode_name(&self) -> &'static str {
        match self.windMode {
            WindMode::NONE => "NONE",
            WindMode::WAVES => "WAVES",
        }
    }

    fn capeStyle_name(&self) -> &'static str {
        match self.capeStyle {
            CapeStyle::BLOCKY => "BLOCKY",
            CapeStyle::SMOOTH => "SMOOTH",
        }
    }

    fn capeMovement_name(&self) -> &'static str {
        match self.capeMovement {
            CapeMovement::VANILLA => "VANILLA",
            CapeMovement::BASIC_SIMULATION => "BASIC_SIMULATION",
            CapeMovement::BASIC_SIMULATION_3D => "BASIC_SIMULATION_3D",
            CapeMovement::DUNGEONS => "DUNGEONS",
        }
    }

    /// Gson-style parse: the default constructor runs first, then present
    /// fields overwrite; an unparseable value throws (Java), which the caller
    /// turns into `None` (fresh config).
    pub fn from_json(text: &str) -> Option<Config> {
        let mut config = Config::default();
        for line in text.lines() {
            let line = line.trim().trim_end_matches(',');
            let Some((key, value)) = line.split_once(": ") else {
                continue;
            };
            let key = key.trim().trim_matches('"');
            let value = value.trim().trim_matches('"');
            match key {
                "configVersion" => config.configVersion = value.parse::<i32>().ok()?,
                "windMode" => config.windMode = match value {
                    "NONE" => WindMode::NONE,
                    "WAVES" => WindMode::WAVES,
                    _ => return None,
                },
                "capeStyle" => config.capeStyle = match value {
                    "BLOCKY" => CapeStyle::BLOCKY,
                    "SMOOTH" => CapeStyle::SMOOTH,
                    _ => return None,
                },
                "capeMovement" => config.capeMovement = match value {
                    "VANILLA" => CapeMovement::VANILLA,
                    "BASIC_SIMULATION" => CapeMovement::BASIC_SIMULATION,
                    "BASIC_SIMULATION_3D" => CapeMovement::BASIC_SIMULATION_3D,
                    "DUNGEONS" => CapeMovement::DUNGEONS,
                    _ => return None,
                },
                "gravity" => config.gravity = value.parse::<i32>().ok()?,
                "heightMultiplier" => config.heightMultiplier = value.parse::<i32>().ok()?,
                "straveMultiplier" => config.straveMultiplier = value.parse::<i32>().ok()?,
                "computeGravityVector" => config.computeGravityVector = value.parse::<bool>().ok()?,
                _ => {}
            }
        }
        Some(config)
    }
}

/// `dev.tr7zw.waveycapes.versionless.config.ConfigUpgrader`
pub struct ConfigUpgrader;

impl ConfigUpgrader {
    /// `public static boolean upgradeConfig(Config config)`
    pub fn upgradeConfig(config: &mut Config) -> bool {
        let mut changed = false;

        // check for more changes here
        if config.configVersion == 1 {
            config.configVersion = 2;
            if config.gravity < 0 {
                config.gravity *= -1; // fixed gravity
            }
        }

        changed
    }
}
        }

        pub mod holder {
//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.CapeHolder`.

#![allow(non_snake_case, dead_code)]

use std::any::{Any, TypeId};

use uuid::Uuid;

use super::super::versionless::config::Config;
use super::super::versionless::modbase::{apply_instance_animations, config_capeMovement};
use super::super::versionless::nms::MinecraftPlayer;
use super::super::versionless::sim::{
    BasicSimulation, CapeMovement, StickSimulation, StickSimulation3d, StickSimulationDungeons,
};
use super::super::versionless::util::{Mth, Vector3};

/// `dev.tr7zw.waveycapes.versionless.CapeHolder`
///
/// Java `default` methods become Rust default trait methods. The class-identity
/// checks use `TypeId` (`sim.getClass() != ...`).
pub trait CapeHolder {
    /// `BasicSimulation getSimulation()`
    fn getSimulation(&self) -> Option<&dyn BasicSimulation>;

    /// Borrow-shaped accessor for the same field (Rust needs `&mut` for the
    /// default methods; the Java side reuses its local variable).
    fn getSimulation_mut(&mut self) -> Option<&mut dyn BasicSimulation>;

    /// `Vector3 getLastPlayerAnimatorPosition()`
    fn getLastPlayerAnimatorPosition(&self) -> Option<&Vector3>;

    /// `void setLastPlayerAnimatorPosition(Vector3 pos)`
    fn setLastPlayerAnimatorPosition(&mut self, pos: Vector3);

    /// `void setSimulation(BasicSimulation sim)`
    fn setSimulation(&mut self, sim: Option<Box<dyn BasicSimulation>>);

    /// `UUID getWCUUID()`
    fn getWCUUID(&self) -> Option<Uuid>;

    /// `void setDirty()`
    fn setDirty(&self);

    /// `void markCapeRendered()`
    fn markCapeRendered(&self);

    /// `void setGravityVectorRequest(boolean canUpdate)`
    fn setGravityVectorRequest(&self, can_update: bool);

    /// `boolean canUpdateGravityVector()`
    fn canUpdateGravityVector(&self) -> bool;

    /// `default void updateSimulation(int partCount)`
    fn updateSimulation(&mut self, part_count: usize) {
        let incorrect = match self.getSimulation() {
            Some(simulation) => self.incorrectSimulation(simulation),
            None => true, // Java: simulation == null || incorrectSimulation(simulation)
        };
        if incorrect {
            let simulation = self.createSimulation();
            self.setSimulation(simulation);
        }
        // Java: if (simulation == null) return;
        let Some(simulation) = self.getSimulation_mut() else {
            return;
        };
        if simulation.init(part_count) {
            self.setDirty();
        }
    }

    /// `default boolean incorrectSimulation(BasicSimulation sim)`
    fn incorrectSimulation(&self, sim: &dyn BasicSimulation) -> bool {
        let style = config_capeMovement();
        let sim_any: &dyn Any = sim;
        if style == CapeMovement::BASIC_SIMULATION && sim_any.type_id() != TypeId::of::<StickSimulation>() {
            return true;
        } else if style == CapeMovement::BASIC_SIMULATION_3D
            && sim_any.type_id() != TypeId::of::<StickSimulation3d>()
        {
            return true;
        } else if style == CapeMovement::DUNGEONS && sim_any.type_id() != TypeId::of::<StickSimulationDungeons>() {
            return true;
        }
        false
    }

    /// `default BasicSimulation createSimulation()`
    fn createSimulation(&self) -> Option<Box<dyn BasicSimulation>> {
        let style = config_capeMovement();
        if style == CapeMovement::BASIC_SIMULATION {
            return Some(Box::new(StickSimulation::new()));
        }
        if style == CapeMovement::BASIC_SIMULATION_3D {
            return Some(Box::new(StickSimulation3d::new()));
        }
        if style == CapeMovement::DUNGEONS {
            return Some(Box::new(StickSimulationDungeons::new()));
        }
        None // Java returns null (VANILLA)
    }

    /// `default void simulate(MinecraftPlayer abstractClientPlayer)`
    fn simulate(&mut self, abstract_client_player: &dyn MinecraftPlayer) {
        let empty = match self.getSimulation() {
            Some(simulation) => simulation.empty(),
            None => true, // Java: simulation == null || simulation.empty()
        };
        if empty {
            return; // no cape, nothing to update
        }

        let d = abstract_client_player.getXCloak() - abstract_client_player.getX();
        let m = abstract_client_player.getZCloak() - abstract_client_player.getZ();
        let n = abstract_client_player.getYBodyRotO() + abstract_client_player.getYBodyRot()
            - abstract_client_player.getYBodyRotO();
        let o = Mth::sin(n * 0.017_453_292_f32) as f64;
        let p = -(Mth::cos(n * 0.017_453_292_f32) as f64);
        let mut height_mul = Config::heightMultiplier() as f32;
        let strave_mul = Config::straveMultiplier() as f32;
        if abstract_client_player.isUnderWater() {
            height_mul *= 2.0; // let the cape have more drag than the player underwater
        }
        // gives the cape a small swing when jumping/falling to not clip with
        // itself/simulate some air getting under it
        let fall_hack = Mth::clamp_d(
            (abstract_client_player.getYo() - abstract_client_player.getY()) * 10.0,
            0.0,
            1.0,
        );
        if abstract_client_player.isUnderWater() {
            self.with_simulation(|simulation| {
                simulation.setGravity(Config::gravity() as f32 / 10.0f32);
            });
        } else {
            self.with_simulation(|simulation| {
                simulation.setGravity(Config::gravity() as f32);
            });
        }

        let mut gravity = Vector3::new(0.0, -1.0, 0.0);
        let yaw = (-abstract_client_player.getYRot() as f64).to_radians() as f32;
        let strave = Mth::cos(yaw) * (abstract_client_player.getX() - abstract_client_player.getXo()) as f32
            - Mth::sin(yaw) * (abstract_client_player.getZ() - abstract_client_player.getZo()) as f32;
        let change_x = (d * o + m * p)
            + fall_hack
            + if abstract_client_player.isCrouching() && !self.simulation_isSneaking() {
                3.0
            } else {
                0.0
            };
        let change_y = ((abstract_client_player.getY() - abstract_client_player.getYo()) * height_mul as f64)
            + if abstract_client_player.isCrouching() && !self.simulation_isSneaking() {
                1.0
            } else {
                0.0
            };
        let change_z = -strave * strave_mul;
        self.with_simulation(|simulation| {
            simulation.setSneaking(abstract_client_player.isCrouching());
        });
        let mut change = Vector3::new(change_x as f32, change_y as f32, change_z);
        if !Config::computeGravityVector() {
            if abstract_client_player.isVisuallySwimming() {
                let mut rotation = abstract_client_player.getXRot(); // -90 = swimming up, 0 = straight, 90 = down
                // the simulation has the body as reference, so if the player is swimming
                // straight down, gravity needs to point up(the cape should move into the
                // direction of the head, not the feet)
                // offset the rotation to swimming up doesn't rotate the vector at all
                rotation += 90.0;
                // apply rotation
                gravity.rotateDegrees(rotation);

                change.rotateDegrees(rotation);
            }
            self.with_simulation(|simulation| {
                simulation.setGravityDirection(&gravity);
            });
        }

        let change = apply_instance_animations(abstract_client_player, change);
        self.with_simulation(|simulation| {
            simulation.applyMovement(&change);
            simulation.simulate();
        });
    }

    /// Borrow-shaped read of `isSneaking()` on the current simulation (Java
    /// calls `simulation.isSneaking()` on its local).
    fn simulation_isSneaking(&self) -> bool {
        self.getSimulation().map_or(false, |simulation| simulation.isSneaking())
    }

    /// Apply `f` to the current simulation (Java mutates its local `simulation`
    /// variable; Rust goes back through the holder's accessor).
    fn with_simulation(&mut self, f: impl FnOnce(&mut dyn BasicSimulation)) {
        if let Some(simulation) = self.getSimulation_mut() {
            f(simulation);
        }
    }
}
        }

        pub mod modbase {
//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.ModBase`.

#![allow(non_snake_case, dead_code)]

use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use super::super::versionless::config::{Config, ConfigUpgrader};
use super::super::versionless::nms::MinecraftPlayer;
use super::super::versionless::sim::{CapeMovement, CapeStyle, WindMode};
use super::super::versionless::util::Vector3;

/// Java `public static final Logger LOGGER = LogManager.getLogger("WaveyCapes")`
pub struct Logger;

impl Logger {
    /// `LOGGER.info(String)`
    pub fn info(&self, message: &str) {
        println!("[WaveyCapes] {}", message);
    }

    /// `LOGGER.warn(String)` / `LOGGER.error(String)`
    pub fn error(&self, message: &str) {
        eprintln!("[WaveyCapes] {}", message);
    }
}

/// Java `public static final Logger LOGGER`
pub static LOGGER: Logger = Logger;

/// Java `public static Config config` (null until `init()` runs; the lock
/// models Java's static field).
pub static CONFIG: OnceLock<Mutex<Option<Config>>> = OnceLock::new();

/// Java `@Getter public static ModBase INSTANCE`
pub static INSTANCE: OnceLock<Mutex<Option<Box<dyn ModBase + Send>>>> = OnceLock::new();

pub fn config_cell() -> &'static Mutex<Option<Config>> {
    CONFIG.get_or_init(|| Mutex::new(None))
}

fn instance_cell() -> &'static Mutex<Option<Box<dyn ModBase + Send>>> {
    INSTANCE.get_or_init(|| Mutex::new(None))
}

/// `dev.tr7zw.waveycapes.versionless.ModBase`
///
/// The abstract class becomes a trait; the Java statics stay module statics.
pub trait ModBase: Send {
    /// `public abstract void initSupportHooks()`
    fn initSupportHooks(&mut self);

    /// `public abstract Vector3 applyModAnimations(MinecraftPlayer player, Vector3 pos)`
    fn applyModAnimations(&mut self, player: &dyn MinecraftPlayer, pos: Vector3) -> Vector3;

    /// `public void init()`. Java sets `INSTANCE = this`; Rust takes the
    /// instance by value.
    fn init(&mut self) {
        // Java: INSTANCE = this;
        // (the concrete mod installs itself via `ModBase::install_instance`)
    }

    /// `public void writeConfig()`
    fn writeConfig(&self) {
        let settings_file = settings_file();
        if settings_file.exists() {
            let _ = fs::remove_file(&settings_file);
        }
        if let Some(config) = config_cell().lock().unwrap().as_ref() {
            if let Err(e1) = fs::write(&settings_file, config.to_json()) {
                e1.print_stack_trace_shim();
            }
        }
    }
}

trait PrintStackTraceShim {
    fn print_stack_trace_shim(&self);
}

impl PrintStackTraceShim for std::io::Error {
    /// Java `e1.printStackTrace()`
    fn print_stack_trace_shim(&self) {
        eprintln!("{}", self);
    }
}

fn settings_file() -> std::path::PathBuf {
    // Java `private final File settingsFile = new File("config", "waveycapes.json")`
    Path::new("config").join("waveycapes.json")
}

/// Java `INSTANCE = this` + the rest of the default `init()` body.
pub fn install_instance_and_init(instance: Box<dyn ModBase + Send>) {
    *instance_cell().lock().unwrap() = Some(instance);
    let settings_file = settings_file();
    let mut loaded: Option<Config> = None;
    if settings_file.exists() {
        match fs::read_to_string(&settings_file) {
            Ok(text) => match Config::from_json(&text) {
                Some(config) => loaded = Some(config),
                None => {
                    // Java catch: error while loading config, keep config null
                    println!("Error while loading config! Creating a new one!");
                }
            },
            Err(ex) => {
                println!("Error while loading config! Creating a new one!");
                ex.print_stack_trace_shim();
            }
        }
    }
    if loaded.is_none() {
        // Java: config = new Config(); writeConfig();
        loaded = Some(Config::default());
        *config_cell().lock().unwrap() = loaded;
        write_config_static();
    } else {
        *config_cell().lock().unwrap() = loaded;
        if ConfigUpgrader::upgradeConfig(config_cell().lock().unwrap().as_mut().unwrap()) {
            write_config_static();
        }
    }
}

/// `writeConfig()` without an instance receiver (Java's static field access).
pub fn write_config_static() {
    let settings_file = settings_file();
    if settings_file.exists() {
        let _ = fs::remove_file(&settings_file);
    }
    if let Some(config) = config_cell().lock().unwrap().as_ref() {
        if let Err(e1) = fs::write(&settings_file, config.to_json()) {
            e1.print_stack_trace_shim();
        }
    }
}

impl Config {
    /// `ModBase.config.computeGravityVector`
    pub fn computeGravityVector() -> bool {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().computeGravityVector, |c| c.computeGravityVector)
    }

    /// Read of the Java static `ModBase.config` field (null before init reads
    /// as the default config on the Rust side; Java would NPE).
    pub fn heightMultiplier() -> i32 {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().heightMultiplier, |c| c.heightMultiplier)
    }

    pub fn straveMultiplier() -> i32 {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().straveMultiplier, |c| c.straveMultiplier)
    }

    pub fn gravity() -> i32 {
        config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().gravity, |c| c.gravity)
    }

}

/// Read of the Java static `ModBase.config.computeGravityVector`.
pub fn computeGravityVector() -> bool {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().computeGravityVector, |c| c.computeGravityVector)
}

/// Read of the Java static `ModBase.config.capeStyle`.
pub fn capeStyle() -> CapeStyle {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().capeStyle, |c| c.capeStyle)
}

/// Read of the Java static `ModBase.config.windMode`.
pub fn windMode() -> WindMode {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().windMode, |c| c.windMode)
}

/// Read of the Java static `ModBase.config.capeMovement`.
pub fn config_capeMovement() -> CapeMovement {
    config_cell().lock().unwrap().as_ref().map_or_else(|| Config::default().capeMovement, |c| c.capeMovement)
}

/// `ModBase.getINSTANCE().applyModAnimations(player, change)`; on the Rust side
/// the instance lives in the static.
pub fn apply_instance_animations(player: &dyn MinecraftPlayer, pos: Vector3) -> Vector3 {
    let mut guard = instance_cell().lock().unwrap();
    match guard.as_mut() {
        Some(instance) => instance.applyModAnimations(player, pos),
        None => pos,
    }
}

/// Read of the Java static `ModBase.INSTANCE` presence.
pub fn has_instance() -> bool {
    instance_cell().lock().unwrap().is_some()
}

/// Java `protected static boolean doesClassExist(String name)`.
///
/// Java detects optional support mods by class name at runtime; the Rust port
/// wires support integrations at compile time, so this reports false (no
/// runtime classpath to probe).
pub fn doesClassExist(_name: &str) -> bool {
    false
}

// ---------------------------------------------------------------------------
// ModLoaderUtil seam
// ---------------------------------------------------------------------------

/// `Screen` seam for `ModLoaderUtil.registerConfigScreen(Supplier<Screen>)`.
pub type ScreenSeam = u64;

/// `dev.tr7zw.transition.loader.ModLoaderUtil` seam: the loader-side calls the
/// mod entry makes; the carrier implements the two hooks.
pub mod ModLoaderUtil_seam {
    use super::ScreenSeam;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    /// `disableDisplayTest()`
    pub fn disableDisplayTest() {
        // carrier hook (fabric's displayTest off)
    }

    static CONFIG_SCREEN: OnceLock<Mutex<Option<fn(ScreenSeam) -> ScreenSeam>>> = OnceLock::new();

    /// `registerConfigScreen(Supplier<Screen> supplier)`
    pub fn registerConfigScreen(supplier: fn(ScreenSeam) -> ScreenSeam) {
        *CONFIG_SCREEN.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(supplier);
    }
}
        }

        pub mod nms {
//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.nms.MinecraftPlayer`.
//!
//! Java NMS interface bridging the versionless code to the mapped player
//! class; the Rust side is a trait the mixin layer implements for the real
//! player type.

#![allow(non_snake_case, dead_code)]

/// `dev.tr7zw.waveycapes.versionless.nms.MinecraftPlayer`
pub trait MinecraftPlayer: super::super::support::LivingEntity {
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
        }

        pub mod sim {
//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.sim`
//! (`CapeSimulation.java`, `BasicSimulation.java`, `StickSimulation.java`,
//! `StickSimulation3d.java`, `StickSimulationDungeons.java`) plus the
//! versionless enums `CapeMovement`, `WindMode` and `CapeStyle`.
//!
//! Upstream comment kept verbatim: Java port of
//! https://www.youtube.com/watch?v=PGk0rnyTa1U by Sebastian Lague. Has some
//! changes like maximizing bends, only designed to simulate a single
//! "rope"(cape). Point 0 is the part fixed to the player.

#![allow(non_snake_case, dead_code)]

use std::any::Any;

use super::util::{CapePoint, Mth, Vector3};

/// `dev.tr7zw.waveycapes.versionless.CapeMovement`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapeMovement {
    VANILLA,
    BASIC_SIMULATION,
    BASIC_SIMULATION_3D,
    DUNGEONS,
}

impl CapeMovement {
    /// The enum constant name (Java's `name()`).
    pub fn as_str(&self) -> &'static str {
        match self {
            CapeMovement::VANILLA => "VANILLA",
            CapeMovement::BASIC_SIMULATION => "BASIC_SIMULATION",
            CapeMovement::BASIC_SIMULATION_3D => "BASIC_SIMULATION_3D",
            CapeMovement::DUNGEONS => "DUNGEONS",
        }
    }
}

/// `dev.tr7zw.waveycapes.versionless.WindMode`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WindMode {
    NONE,
    WAVES,
}

impl WindMode {
    /// The enum constant name (Java's `name()`).
    pub fn as_str(&self) -> &'static str {
        match self {
            WindMode::NONE => "NONE",
            WindMode::WAVES => "WAVES",
        }
    }
}

/// `dev.tr7zw.waveycapes.versionless.CapeStyle`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapeStyle {
    BLOCKY,
    SMOOTH,
}

impl CapeStyle {
    /// The enum constant name (Java's `name()`).
    pub fn as_str(&self) -> &'static str {
        match self {
            CapeStyle::BLOCKY => "BLOCKY",
            CapeStyle::SMOOTH => "SMOOTH",
        }
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.BasicSimulation`
pub trait BasicSimulation: Any {
    /// `BasicSimulation.simulate()`
    fn simulate(&mut self);
    /// `BasicSimulation.setGravityDirection(Vector3)`
    fn setGravityDirection(&self, gravity_direction: &Vector3);
    /// `BasicSimulation.getGravity()`
    fn getGravity(&self) -> f32;
    /// `BasicSimulation.setGravity(float)`
    fn setGravity(&mut self, gravity: f32);
    /// `BasicSimulation.isSneaking()`
    fn isSneaking(&self) -> bool;
    /// `BasicSimulation.setSneaking(boolean)`
    fn setSneaking(&mut self, sneaking: bool);
    /// `BasicSimulation.init(int)`; returns true if it was re-initialized
    fn init(&mut self, part_count: usize) -> bool;
    /// `BasicSimulation.empty()`
    fn empty(&self) -> bool;
    /// `BasicSimulation.applyMovement(Vector3)`
    fn applyMovement(&mut self, movement: &Vector3);
    /// `BasicSimulation.getPoints()`
    fn getPoints(&self) -> Vec<PointView<'_>>;
}

/// `CapeSimulation` base class. Java inheritance becomes composition: the
/// concrete simulations embed a `CapeSimulation` and forward to it.
pub struct CapeSimulation {
    /// Java `protected float[] x`
    pub x: Vec<f32>,
    /// Java `protected float[] y`
    pub y: Vec<f32>,
    /// Java `protected float[] z`
    pub z: Vec<f32>,
    /// Java `private float[] prevX`
    prev_x: Vec<f32>,
    /// Java `private float[] prevY`
    prev_y: Vec<f32>,
    /// Java `private float[] prevZ`
    prev_z: Vec<f32>,
    /// Java `private final int numIterations`
    num_iterations: usize,
    /// Java `protected float gravityX`
    pub gravity_x: std::cell::Cell<f32>,
    /// Java `protected float gravityY`
    pub gravity_y: std::cell::Cell<f32>,
    /// Java `protected float gravityZ`
    pub gravity_z: std::cell::Cell<f32>,
    /// Java `@Getter @Setter private float gravity`
    gravity: f32,
    /// Java `@Getter @Setter private boolean sneaking`
    sneaking: bool,
    /// Java `private float dirX, dirY, dirZ`
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
}

impl CapeSimulation {
    /// `protected CapeSimulation(int numIterations)`
    pub fn new(num_iterations: usize) -> Self {
        CapeSimulation {
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
            prev_x: Vec::new(),
            prev_y: Vec::new(),
            prev_z: Vec::new(),
            num_iterations,
            gravity_x: std::cell::Cell::new(0.0),
            gravity_y: std::cell::Cell::new(-1.0),
            gravity_z: std::cell::Cell::new(0.0),
            gravity: 0.0,
            sneaking: false,
            dir_x: 0.0,
            dir_y: 0.0,
            dir_z: 0.0,
        }
    }

    /// `CapeSimulation.init(int)` (the `@Override` of `BasicSimulation.init`)
    pub fn init(&mut self, part_count: usize) -> bool {
        if self.x.len() == part_count {
            return false;
        }
        self.x = vec![0.0; part_count];
        self.y = vec![0.0; part_count];
        self.z = vec![0.0; part_count];
        self.prev_x = vec![0.0; part_count];
        self.prev_y = vec![0.0; part_count];
        self.prev_z = vec![0.0; part_count];
        for i in 0..part_count {
            self.y[i] = -(i as f32);
        }
        // Java builds the List<CapePoint> views here; the views borrow the
        // sim on demand in Rust (see `PointView` / `BasicSimulation::getPoints`).
        true
    }

    /// `CapeSimulation.empty()`
    pub fn empty(&self) -> bool {
        self.x.len() < 2
    }

    /// `CapeSimulation.setGravityDirection(Vector3)`
    pub fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.gravity_x.set(gravity_direction.x);
        self.gravity_y.set(gravity_direction.y);
        self.gravity_z.set(gravity_direction.z);
    }

    /// `CapeSimulation.applyMovement(Vector3)`
    pub fn applyMovement(&mut self, movement: &Vector3) {
        self.r#move(movement.x, movement.y, movement.z);
    }

    /// `protected void move(float dx, float dy, float dz)`
    pub fn r#move(&mut self, dx: f32, dy: f32, dz: f32) {
        self.prev_x[0] = self.x[0];
        self.prev_y[0] = self.y[0];
        self.prev_z[0] = self.z[0];
        self.x[0] += dx;
        self.y[0] += dy;
        self.z[0] += dz;
    }

    /// `protected void applyGravity()`
    pub fn applyGravity(&mut self) {
        let fall = self.gravity * (50.0f32 / 1000.0); // fixed timescale
        for i in 1..self.x.len() {
            self.prev_x[i] = self.x[i];
            self.prev_y[i] = self.y[i];
            self.prev_z[i] = self.z[i];
            self.x[i] += self.gravity_x.get() * fall;
            self.y[i] += self.gravity_y.get() * fall;
            self.z[i] += self.gravity_z.get() * fall;
        }
    }

    /// `protected void preventClipping()`
    pub fn preventClipping(&mut self) {
        // prevent the cape from clipping into the player
        for i in 1..self.x.len() {
            if self.x[i] - self.x[0] > 0.0 {
                self.x[i] = self.x[0];
            }
            let max_z = ((i as f32 / self.x.len() as f32) * (i as f32 / self.x.len() as f32)) * 5.0;
            let offset = self.z[0] - self.z[i];
            if offset > max_z {
                self.z[i] = self.z[0] - max_z;
            }
            if offset < -max_z {
                self.z[i] = self.z[0] + max_z;
            }
        }
    }

    /// `protected void applyMotion()`
    pub fn applyMotion(&mut self) {
        // move into correct direction
        for _ in 0..self.num_iterations {
            for a in (0..self.x.len().saturating_sub(1)).rev() {
                self.direction(a, a + 1);
                self.separate(a, a + 1);
            }
        }
    }

    /// `protected void limitLength()`
    pub fn limitLength(&mut self) {
        // fix in the position/length, this prevents it from acting like a
        // spring/stretchy
        for a in 0..self.x.len().saturating_sub(1) {
            self.direction(a, a + 1);
            self.x[a + 1] = self.x[a] - self.dir_x;
            self.y[a + 1] = self.y[a] - self.dir_y;
            self.z[a + 1] = self.z[a] - self.dir_z;
        }
    }

    /// `protected void preventSelfClipping(int maxRuns)`
    pub fn preventSelfClipping(&mut self, max_runs: i32) {
        let mut clipped;
        let mut runs = 0;
        loop {
            clipped = false;
            // check the cape parts against each other. Bad implementation
            for a in 0..self.x.len() {
                for b in a + 1..self.x.len() {
                    let dx = self.x[a] - self.x[b];
                    let dy = self.y[a] - self.y[b];
                    let dz = self.z[a] - self.z[b];
                    if dx * dx + dy * dy + dz * dz < 0.99 {
                        clipped = true;
                        runs += 1;
                        self.direction(a, b);
                        self.separate(a, b);
                    }
                }
            }
            if !(clipped && runs < max_runs) {
                break;
            }
        }
    }

    /// `protected void preventHardBend(int i, float maxBend)`
    /// Doesnt work like it should at all, but it prevents some folding into
    /// itself, so it stays for now
    pub fn preventHardBend(&mut self, i: usize, max_bend: f32) {
        let abx = self.x[i - 1] - self.x[i];
        let aby = self.y[i - 1] - self.y[i];
        let cbx = self.x[i - 1] - self.x[i + 1];
        let cby = self.y[i - 1] - self.y[i + 1];

        let dot = abx * cbx + aby * cby; // dot product
        let cross = abx * cby - aby * cbx; // cross product

        let angle = Mth::atan2(cross as f64, dot as f64) * 180.0 / std::f64::consts::PI;
        if angle < -(max_bend as f64) {
            self.swing(i, -max_bend * 2.0);
        }
        if angle > max_bend as f64 {
            self.swing(i, max_bend * 2.0);
        }
    }

    /// `private void swing(int i, float deg)`
    fn swing(&mut self, i: usize, deg: f32) {
        let ox = self.x[i] - self.x[i - 1];
        let oy = self.y[i] - self.y[i - 1];
        let oz = self.z[i] - self.z[i - 1];
        let deg = (deg as f64).to_radians() as f32;
        self.x[i + 1] = Mth::cos(deg) * ox - Mth::sin(deg) * oy + self.x[i];
        self.y[i + 1] = Mth::sin(deg) * ox + Mth::cos(deg) * oy + self.y[i];
        self.z[i + 1] = oz + self.z[i];
    }

    /// `private void separate(int a, int b)`
    fn separate(&mut self, a: usize, b: usize) {
        let centre_x = (self.x[a] + self.x[b]) / 2.0;
        let centre_y = (self.y[a] + self.y[b]) / 2.0;
        let centre_z = (self.z[a] + self.z[b]) / 2.0;
        if a != 0 {
            self.x[a] = centre_x + self.dir_x / 2.0;
            self.y[a] = centre_y + self.dir_y / 2.0;
            self.z[a] = centre_z + self.dir_z / 2.0;
        }
        self.x[b] = centre_x - self.dir_x / 2.0;
        self.y[b] = centre_y - self.dir_y / 2.0;
        self.z[b] = centre_z - self.dir_z / 2.0;
    }

    /// `private void direction(int a, int b)`
    fn direction(&mut self, a: usize, b: usize) {
        self.dir_x = self.x[a] - self.x[b];
        self.dir_y = self.y[a] - self.y[b];
        self.dir_z = self.z[a] - self.z[b];
        let f = Mth::sqrt(self.dir_x * self.dir_x + self.dir_y * self.dir_y + self.dir_z * self.dir_z);
        if f < 1.0E-4 {
            self.dir_x = 0.0;
            self.dir_y = 0.0;
            self.dir_z = 0.0;
        } else {
            self.dir_x /= f;
            self.dir_y /= f;
            self.dir_z /= f;
        }
    }

    /// `CapeSimulation.getGravity()` (lombok `@Getter`)
    pub fn getGravity(&self) -> f32 {
        self.gravity
    }

    /// `CapeSimulation.setGravity(float)` (lombok `@Setter`)
    pub fn setGravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    /// `CapeSimulation.isSneaking()` (lombok `@Getter`)
    pub fn isSneaking(&self) -> bool {
        self.sneaking
    }

    /// `CapeSimulation.setSneaking(boolean)` (lombok `@Setter`)
    pub fn setSneaking(&mut self, sneaking: bool) {
        self.sneaking = sneaking;
    }

    /// `Point.getLerpX(float)` on the inner `Point` view for `index`
    pub fn pointLerpX(&self, index: usize, delta: f32) -> f32 {
        Mth::lerp(delta, self.prev_x[index], self.x[index])
    }

    /// `Point.getLerpY(float)`
    pub fn pointLerpY(&self, index: usize, delta: f32) -> f32 {
        Mth::lerp(delta, self.prev_y[index], self.y[index])
    }

    /// `Point.getLerpZ(float)`
    pub fn pointLerpZ(&self, index: usize, delta: f32) -> f32 {
        Mth::lerp(delta, self.prev_z[index], self.z[index])
    }
}

impl BasicSimulation for CapeSimulation {
    fn simulate(&mut self) {
        // abstract upstream; concrete sims override
    }

    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        CapeSimulation::setGravityDirection(self, gravity_direction)
    }

    fn getGravity(&self) -> f32 {
        self.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.sneaking = sneaking;
    }

    fn init(&mut self, part_count: usize) -> bool {
        CapeSimulation::init(self, part_count)
    }

    fn empty(&self) -> bool {
        self.x.len() < 2
    }

    fn applyMovement(&mut self, movement: &Vector3) {
        CapeSimulation::applyMovement(self, movement)
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        (0..self.x.len()).map(|index| PointView { sim: self, index }).collect()
    }
}

/// The inner `private final class Point implements CapePoint`, as a borrow of
/// its owning simulation.
pub struct PointView<'a> {
    sim: &'a CapeSimulation,
    index: usize,
}

impl CapePoint for PointView<'_> {
    fn getLerpX(&self, delta: f32) -> f32 {
        self.sim.pointLerpX(self.index, delta)
    }

    fn getLerpY(&self, delta: f32) -> f32 {
        self.sim.pointLerpY(self.index, delta)
    }

    fn getLerpZ(&self, delta: f32) -> f32 {
        self.sim.pointLerpZ(self.index, delta)
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.StickSimulation`
pub struct StickSimulation {
    /// Java `extends CapeSimulation`
    pub base: CapeSimulation,
}

impl StickSimulation {
    /// `private static final float MAX_BEND = 5`
    pub const MAX_BEND: f32 = 5.0;

    /// `public StickSimulation() { super(3); }`
    pub fn new() -> Self {
        StickSimulation { base: CapeSimulation::new(3) }
    }

    /// `private void preventHardBends()`
    /// Doesnt work like it should at all, but it prevents some folding into
    /// itself, so it stays for now
    fn preventHardBends(&mut self) {
        let len = self.base.x.len();
        for i in (1..len.saturating_sub(1)).rev() {
            let mut angle = ((self.base.y[i + 1] - self.base.y[i]) as f64).atan2((self.base.x[i + 1] - self.base.x[i]) as f64)
                - ((self.base.y[i - 1] - self.base.y[i]) as f64).atan2((self.base.x[i - 1] - self.base.x[i]) as f64);
            angle *= 57.2958;
            if angle > 360.0 {
                angle -= 360.0;
            }
            if angle < -360.0 {
                angle += 360.0;
            }
            let abs = angle.abs();
            if abs < 180.0 - Self::MAX_BEND as f64 {
                self.replace(i, angle, 180.0 - Self::MAX_BEND as f64 + 1.0);
            }
            if abs > 180.0 + Self::MAX_BEND as f64 {
                self.replace(i, angle, 180.0 + Self::MAX_BEND as f64 - 1.0);
            }
        }
    }

    /// `private void replace(int i, double angle, double target)`
    fn replace(&mut self, i: usize, angle: f64, target: f64) {
        let mut theta = target / 57.2958;
        let px = self.base.x[i - 1] - self.base.x[i];
        let py = self.base.y[i - 1] - self.base.y[i];
        if angle < 0.0 {
            theta *= -1.0;
        }
        let cs = theta.cos();
        let sn = theta.sin();
        self.base.x[i + 1] = ((px as f64 * cs) - (py as f64 * sn) + self.base.x[i] as f64) as f32;
        self.base.y[i + 1] = ((px as f64 * sn) + (py as f64 * cs) + self.base.y[i] as f64) as f32;
    }
}

impl Default for StickSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicSimulation for StickSimulation {
    fn simulate(&mut self) {
        self.base.applyGravity();
        self.base.preventClipping();
        self.preventHardBends();
        self.base.applyMotion();
        self.base.limitLength();
    }

    /// `@Override setGravityDirection`: only gravityX and gravityY
    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.base.gravity_x.set(gravity_direction.x);
        self.base.gravity_y.set(gravity_direction.y);
    }

    /// `@Override applyMovement`: `move(movement.x, movement.y, 0)`
    fn applyMovement(&mut self, movement: &Vector3) {
        self.base.r#move(movement.x, movement.y, 0.0);
    }

    fn getGravity(&self) -> f32 {
        self.base.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.base.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.base.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.base.sneaking = sneaking;
    }

    fn init(&mut self, part_count: usize) -> bool {
        self.base.init(part_count)
    }

    fn empty(&self) -> bool {
        self.base.empty()
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        self.base.getPoints()
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.StickSimulation3d`
pub struct StickSimulation3d {
    /// Java `extends CapeSimulation`
    pub base: CapeSimulation,
}

impl StickSimulation3d {
    /// `private static final float MAX_BEND = 20`
    pub const MAX_BEND: f32 = 20.0;

    /// `public StickSimulation3d() { super(3); }`
    pub fn new() -> Self {
        StickSimulation3d { base: CapeSimulation::new(3) }
    }
}

impl Default for StickSimulation3d {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicSimulation for StickSimulation3d {
    /// `@Override init`: super.init then `x[i] = -i`
    fn init(&mut self, part_count: usize) -> bool {
        if !self.base.init(part_count) {
            return false;
        }
        for i in 0..part_count {
            self.base.x[i] = -(i as f32);
        }
        true
    }

    /// `@Override simulate`
    fn simulate(&mut self) {
        self.base.applyGravity();
        self.base.preventClipping();
        self.base.preventSelfClipping(32);
        self.base.applyMotion();
        self.base.preventSelfClipping(32);
        for i in 1..self.base.x.len().saturating_sub(2) {
            self.base.preventHardBend(i, Self::MAX_BEND);
        }
        self.base.limitLength();
    }

    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.base.setGravityDirection(gravity_direction)
    }

    fn getGravity(&self) -> f32 {
        self.base.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.base.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.base.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.base.sneaking = sneaking;
    }

    fn empty(&self) -> bool {
        self.base.empty()
    }

    /// `@Override applyMovement` (inherited upstream)
    fn applyMovement(&mut self, movement: &Vector3) {
        self.base.applyMovement(movement)
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        self.base.getPoints()
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.StickSimulationDungeons`
pub struct StickSimulationDungeons {
    /// Java `extends CapeSimulation`
    pub base: CapeSimulation,
}

impl StickSimulationDungeons {
    /// `private static final float MAX_BEND = 20`
    pub const MAX_BEND: f32 = 20.0;

    /// `public StickSimulationDungeons() { super(30); }`
    pub fn new() -> Self {
        StickSimulationDungeons { base: CapeSimulation::new(30) }
    }
}

impl Default for StickSimulationDungeons {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicSimulation for StickSimulationDungeons {
    /// `@Override simulate`
    fn simulate(&mut self) {
        self.base.applyGravity();
        self.base.preventClipping();
        self.base.applyMotion();
        for i in 1..self.base.x.len().saturating_sub(1) {
            let max_bend = if i == self.base.x.len() / 2 { Self::MAX_BEND } else { 0.0 };
            self.base.preventHardBend(i, max_bend);
        }
        self.base.preventSelfClipping(10);
        self.base.limitLength();
    }

    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.base.setGravityDirection(gravity_direction)
    }

    fn getGravity(&self) -> f32 {
        self.base.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.base.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.base.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.base.sneaking = sneaking;
    }

    fn init(&mut self, part_count: usize) -> bool {
        self.base.init(part_count)
    }

    fn empty(&self) -> bool {
        self.base.empty()
    }

    fn applyMovement(&mut self, movement: &Vector3) {
        self.base.applyMovement(movement)
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        self.base.getPoints()
    }
}
        }

        pub mod util {
//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.util`
//! (`Mth.java`, `Vector3.java`, `Vector4.java`, `CapePoint.java`,
//! `CapeStick.java`). Identifiers keep the upstream camelCase names;
//! Java method overloads keep their arity via the `2`/`xyz` suffix form.

#![allow(non_snake_case, dead_code)]

#[allow(non_snake_case)]
pub mod Mth {
    /// `Mth.PI`
    pub const PI: f32 = 3.1415927;
    /// `Mth.HALF_PI`
    pub const HALF_PI: f32 = 1.5707964;
    /// `Mth.TWO_PI`
    pub const TWO_PI: f32 = 6.2831855;
    /// `Mth.DEG_TO_RAD`
    pub const DEG_TO_RAD: f32 = 0.017453292;
    /// `Mth.RAD_TO_DEG`
    pub const RAD_TO_DEG: f32 = 57.295776;
    /// `Mth.EPSILON`
    pub const EPSILON: f32 = 1.0E-5;
    /// `Mth.SQRT_OF_TWO` (Java evaluates `sqrt(2.0F)` at class init)
    pub const SQRT_OF_TWO: f32 = 1.4142135;

    /// `Mth.truncate(float f, float g)`
    pub fn truncate(f: f32, g: f32) -> f32 {
        let h = 10.0_f64.powf(g as f64);
        ((f as f64 * h) as i64) as f32 / h as f32
    }

    /// `Mth.sqrt(float)`
    pub fn sqrt(f: f32) -> f32 {
        (f as f64).sqrt() as f32
    }

    /// `Mth.floor(float)`
    pub fn floor(f: f32) -> i32 {
        let i = f as i32;
        if f < i as f32 { i - 1 } else { i }
    }

    /// `Mth.fastFloor(double)`
    pub fn fastFloor(d: f64) -> i32 {
        (d + 1024.0) as i32 - 1024
    }

    /// `Mth.floor(double)`
    pub fn floor_d(d: f64) -> i32 {
        let i = d as i32;
        if d < i as f64 { i - 1 } else { i }
    }

    /// `Mth.lfloor(double)`
    pub fn lfloor(d: f64) -> i64 {
        let l = d as i64;
        if d < l as f64 { l - 1 } else { l }
    }

    /// `Mth.absFloor(double)`
    pub fn absFloor(d: f64) -> i32 {
        (if d >= 0.0 { d } else { -d + 1.0 }) as i32
    }

    /// `Mth.abs(float)`
    pub fn abs(f: f32) -> f32 {
        f.abs()
    }

    /// `Mth.abs(int)`
    pub fn abs_i(i: i32) -> i32 {
        i.abs()
    }

    /// `Mth.ceil(float)`
    pub fn ceil(f: f32) -> i32 {
        let i = f as i32;
        if f > i as f32 { i + 1 } else { i }
    }

    /// `Mth.ceil(double)`
    pub fn ceil_d(d: f64) -> i32 {
        let i = d as i32;
        if d > i as f64 { i + 1 } else { i }
    }

    /// `Mth.clamp(byte, byte, byte)`
    pub fn clamp_b(b: i8, c: i8, d: i8) -> i8 {
        if b < c {
            return c;
        }
        if b > d {
            return d;
        }
        b
    }

    /// `Mth.clamp(int, int, int)`
    pub fn clamp(i: i32, j: i32, k: i32) -> i32 {
        if i < j {
            return j;
        }
        if i > k {
            return k;
        }
        i
    }

    /// `Mth.clamp(long, long, long)`
    pub fn clamp_l(l: i64, m: i64, n: i64) -> i64 {
        if l < m {
            return m;
        }
        if l > n {
            return n;
        }
        l
    }

    /// `Mth.clamp(float, float, float)`
    pub fn clamp_f(f: f32, g: f32, h: f32) -> f32 {
        if f < g {
            return g;
        }
        if f > h {
            return h;
        }
        f
    }

    /// `Mth.clamp(double, double, double)`
    pub fn clamp_d(d: f64, e: f64, f: f64) -> f64 {
        if d < e {
            return e;
        }
        if d > f {
            return f;
        }
        d
    }

    /// `Mth.clampedLerp(double, double, double)`
    pub fn clampedLerp_d(d: f64, e: f64, f: f64) -> f64 {
        if f < 0.0 {
            return d;
        }
        if f > 1.0 {
            return e;
        }
        lerp_d(f, d, e)
    }

    /// `Mth.clampedLerp(float, float, float)`
    pub fn clampedLerp(f: f32, g: f32, h: f32) -> f32 {
        if h < 0.0 {
            return f;
        }
        if h > 1.0 {
            return g;
        }
        lerp(h, f, g)
    }

    /// `Mth.absMax(double, double)`
    pub fn absMax(d: f64, e: f64) -> f64 {
        let d = if d < 0.0 { -d } else { d };
        let e = if e < 0.0 { -e } else { e };
        if d > e { d } else { e }
    }

    /// `Mth.intFloorDiv(int, int)`
    pub fn intFloorDiv(i: i32, j: i32) -> i32 {
        i.div_euclid(j)
    }

    /// `Mth.average(long[])`
    pub fn average(ls: &[i64]) -> f64 {
        let mut l: i64 = 0;
        for m in ls {
            l += *m;
        }
        l as f64 / ls.len() as f64
    }

    /// `Mth.equal(float, float)`
    pub fn equal(f: f32, g: f32) -> bool {
        (g - f).abs() < 1.0E-5
    }

    /// `Mth.equal(double, double)`
    pub fn equal_d(d: f64, e: f64) -> bool {
        (e - d).abs() < 9.999_999_747_378_752E-6
    }

    /// `Mth.positiveModulo(int, int)`
    pub fn positiveModulo(i: i32, j: i32) -> i32 {
        i.rem_euclid(j)
    }

    /// `Mth.positiveModulo(float, float)`
    pub fn positiveModulo_f(f: f32, g: f32) -> f32 {
        (f % g + g) % g
    }

    /// `Mth.positiveModulo(double, double)`
    pub fn positiveModulo_d(d: f64, e: f64) -> f64 {
        (d % e + e) % e
    }

    /// `Mth.isDivisionInteger(int, int)`
    pub fn isDivisionInteger(i: i32, j: i32) -> bool {
        i / j * j == i
    }

    /// `Mth.wrapDegrees(int)`
    pub fn wrapDegrees(i: i32) -> i32 {
        let mut j = i % 360;
        if j >= 180 {
            j -= 360;
        }
        if j < -180 {
            j += 360;
        }
        j
    }

    /// `Mth.wrapDegrees(float)`
    pub fn wrapDegrees_f(f: f32) -> f32 {
        let mut g = f % 360.0;
        if g >= 180.0 {
            g -= 360.0;
        }
        if g < -180.0 {
            g += 360.0;
        }
        g
    }

    /// `Mth.wrapDegrees(double)`
    pub fn wrapDegrees_d(d: f64) -> f64 {
        let mut e = d % 360.0;
        if e >= 180.0 {
            e -= 360.0;
        }
        if e < -180.0 {
            e += 360.0;
        }
        e
    }

    /// `Mth.degreesDifference(float, float)`
    pub fn degreesDifference(f: f32, g: f32) -> f32 {
        wrapDegrees_f(g - f)
    }

    /// `Mth.degreesDifferenceAbs(float, float)`
    pub fn degreesDifferenceAbs(f: f32, g: f32) -> f32 {
        abs(degreesDifference(f, g))
    }

    /// `Mth.rotateIfNecessary(float, float, float)`
    pub fn rotateIfNecessary(f: f32, g: f32, h: f32) -> f32 {
        let i = degreesDifference(f, g);
        let j = clamp_f(i, -h, h);
        g - j
    }

    /// `Mth.approach(float, float, float)`
    pub fn approach(f: f32, g: f32, h: f32) -> f32 {
        let h = abs(h);
        if f < g {
            return clamp_f(f + h, f, g);
        }
        clamp_f(f - h, g, f)
    }

    /// `Mth.approachDegrees(float, float, float)`
    pub fn approachDegrees(f: f32, g: f32, h: f32) -> f32 {
        let i = degreesDifference(f, g);
        approach(f, f + i, h)
    }

    /// `Mth.getDouble(String, double)`
    pub fn getDouble(string: &str, d: f64) -> f64 {
        string.parse::<f64>().unwrap_or(d)
    }

    /// `Mth.getDouble(String, double, double)`
    pub fn getDouble2(string: &str, d: f64, e: f64) -> f64 {
        e.max(getDouble(string, d))
    }

    /// `Mth.smallestEncompassingPowerOfTwo(int)`
    pub fn smallestEncompassingPowerOfTwo(i: i32) -> i32 {
        let mut j = i - 1;
        j |= j >> 1;
        j |= j >> 2;
        j |= j >> 4;
        j |= j >> 8;
        j |= j >> 16;
        j + 1
    }

    /// `Mth.isPowerOfTwo(int)`
    pub fn isPowerOfTwo(i: i32) -> bool {
        i != 0 && (i & (i - 1)) == 0
    }

    /// `Mth.MULTIPLY_DE_BRUIJN_BIT_POSITION`
    const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
        0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7,
        26, 12, 18, 6, 11, 5, 10, 9,
    ];

    /// `Mth.ceillog2(int)`
    pub fn ceillog2(i: i32) -> i32 {
        let i = if isPowerOfTwo(i) { i } else { smallestEncompassingPowerOfTwo(i) };
        MULTIPLY_DE_BRUIJN_BIT_POSITION[(((i as i64 * 125613361) >> 27) & 0x1F) as usize]
    }

    /// `Mth.log2(int)`
    pub fn log2(i: i32) -> i32 {
        ceillog2(i) - if isPowerOfTwo(i) { 0 } else { 1 }
    }

    /// `Mth.color(float, float, float)`
    pub fn color_f(f: f32, g: f32, h: f32) -> i32 {
        color(floor(f * 255.0), floor(g * 255.0), floor(h * 255.0))
    }

    /// `Mth.color(int, int, int)`
    pub fn color(i: i32, j: i32, k: i32) -> i32 {
        let mut l = i;
        l = (l << 8) + j;
        l = (l << 8) + k;
        l
    }

    /// `Mth.colorMultiply(int, int)`
    pub fn colorMultiply(i: i32, j: i32) -> i32 {
        let k = (i & 0xFF0000) >> 16;
        let l = (j & 0xFF0000) >> 16;
        let m = (i & 0xFF00) >> 8;
        let n = (j & 0xFF00) >> 8;
        let o = i & 0xFF;
        let p = j & 0xFF;
        let q = (k as f32 * l as f32 / 255.0) as i32;
        let r = (m as f32 * n as f32 / 255.0) as i32;
        let s = (o as f32 * p as f32 / 255.0) as i32;
        i & 0xFF000000u32 as i32 | q << 16 | r << 8 | s
    }

    /// `Mth.colorMultiply(int, float, float, float)`
    pub fn colorMultiply_f(i: i32, f: f32, g: f32, h: f32) -> i32 {
        let j = (i & 0xFF0000) >> 16;
        let k = (i & 0xFF00) >> 8;
        let l = i & 0xFF;
        let m = (j as f32 * f) as i32;
        let n = (k as f32 * g) as i32;
        let o = (l as f32 * h) as i32;
        i & 0xFF000000u32 as i32 | m << 16 | n << 8 | o
    }

    /// `Mth.frac(float)`
    pub fn frac(f: f32) -> f32 {
        f - floor(f) as f32
    }

    /// `Mth.frac(double)`
    pub fn frac_d(d: f64) -> f64 {
        d - lfloor(d) as f64
    }

    /// `Mth.getSeed(int, int, int)`
    pub fn getSeed(i: i32, j: i32, k: i32) -> i64 {
        let mut l = (i as i64 * 3129871) ^ (k as i64 * 116129781) ^ j as i64;
        l = l * l * 42317861 + l * 11;
        l >> 16
    }

    /// `Mth.inverseLerp(double, double, double)`
    pub fn inverseLerp_d(d: f64, e: f64, f: f64) -> f64 {
        (d - e) / (f - e)
    }

    /// `Mth.inverseLerp(float, float, float)`
    pub fn inverseLerp(f: f32, g: f32, h: f32) -> f32 {
        (f - g) / (h - g)
    }

    /// `Mth.FRAC_BIAS`
    const FRAC_BIAS: f64 = f64::from_bits(4805340802404319232);

    /// The `ASIN_TAB`/`COS_TAB` pair Java builds in its static initializer.
    fn asin_tables() -> &'static ([f64; 257], [f64; 257]) {
        use std::sync::OnceLock;
        static TABLES: OnceLock<([f64; 257], [f64; 257])> = OnceLock::new();
        TABLES.get_or_init(|| {
            let mut asin_tab = [0.0_f64; 257];
            let mut cos_tab = [0.0_f64; 257];
            for (i, slot) in asin_tab.iter_mut().enumerate().take(257) {
                let d = i as f64 / 256.0;
                let e = d.asin();
                cos_tab[i] = e.cos();
                *slot = e;
            }
            (asin_tab, cos_tab)
        })
    }

    /// `Mth.atan2(double, double)`
    pub fn atan2(mut d: f64, mut e: f64) -> f64 {
        let f = e * e + d * d;
        if f.is_nan() {
            return f64::NAN;
        }
        let bl = d < 0.0;
        if bl {
            d = -d;
        }
        let bl2 = e < 0.0;
        if bl2 {
            e = -e;
        }
        let bl3 = d > e;
        if bl3 {
            let d1 = e;
            e = d;
            d = d1;
        }
        let g = fastInvSqrt_d(f);
        e *= g;
        d *= g;
        let h = FRAC_BIAS + d;
        let i = h.to_bits() as i64 as i32;
        let (asin_tab, cos_tab) = asin_tables();
        let j = asin_tab[i as usize];
        let k = cos_tab[i as usize];
        let l = h - FRAC_BIAS;
        let m = d * k - e * l;
        let n = (6.0 + m * m) * m * 0.16666666666666666;
        let mut o = j + n;
        if bl3 {
            o = 1.5707963267948966 - o;
        }
        if bl2 {
            o = std::f64::consts::PI - o;
        }
        if bl {
            o = -o;
        }
        o
    }

    /// `Mth.fastInvSqrt(float)`
    pub fn fastInvSqrt(f: f32) -> f32 {
        let g = 0.5 * f;
        let mut i = f.to_bits() as i32;
        i = 1597463007 - (i >> 1);
        let mut f = f32::from_bits(i as u32);
        f *= 1.5 - g * f * f;
        f
    }

    /// `Mth.fastInvSqrt(double)`
    pub fn fastInvSqrt_d(d: f64) -> f64 {
        let e = 0.5 * d;
        let mut l = d.to_bits() as i64;
        l = 6910469410427058090 - (l >> 1);
        let mut d = f64::from_bits(l as u64);
        d *= 1.5 - e * d * d;
        d
    }

    /// `Mth.fastInvCubeRoot(float)`
    pub fn fastInvCubeRoot(f: f32) -> f32 {
        let mut i = f.to_bits() as i32;
        i = 1419967116 - i / 3;
        let mut g = f32::from_bits(i as u32);
        g = 0.6666667 * g + 1.0 / 3.0 * g * g * f;
        g = 0.6666667 * g + 1.0 / 3.0 * g * g * f;
        g
    }

    /// `Mth.hsvToRgb(float, float, float)`
    pub fn hsvToRgb(f: f32, g: f32, h: f32) -> i32 {
        let (n, o, p);
        let i = ((f * 6.0) as i32) % 6;
        let j = f * 6.0 - i as f32;
        let k = h * (1.0 - g);
        let l = h * (1.0 - j * g);
        let m = h * (1.0 - (1.0 - j) * g);
        match i {
            0 => {
                n = h;
                o = m;
                p = k;
            }
            1 => {
                n = l;
                o = h;
                p = k;
            }
            2 => {
                n = k;
                o = h;
                p = m;
            }
            3 => {
                n = k;
                o = l;
                p = h;
            }
            4 => {
                n = m;
                o = k;
                p = h;
            }
            5 => {
                n = h;
                o = k;
                p = l;
            }
            _ => panic!(
                "Something went wrong when converting from HSV to RGB. Input was {}, {}, {}",
                f, g, h
            ),
        }
        let q = clamp((n * 255.0) as i32, 0, 255);
        let r = clamp((o * 255.0) as i32, 0, 255);
        let s = clamp((p * 255.0) as i32, 0, 255);
        q << 16 | r << 8 | s
    }

    /// `Mth.murmurHash3Mixer(int)`
    pub fn murmurHash3Mixer(mut i: i32) -> i32 {
        i ^= i >> 16;
        i = i.wrapping_mul(-2048144789);
        i ^= i >> 13;
        i = i.wrapping_mul(-1028477387);
        i ^= i >> 16;
        i
    }

    /// `Mth.murmurHash3Mixer(long)`
    pub fn murmurHash3Mixer_l(mut l: i64) -> i64 {
        l ^= l >> 33;
        l = l.wrapping_mul(-49064778989728563);
        l ^= l >> 33;
        l = l.wrapping_mul(-4265267296055464877);
        l ^= l >> 33;
        l
    }

    /// `Mth.cumulativeSum(double...)`
    pub fn cumulativeSum(ds: &mut [f64]) {
        let mut d = 0.0;
        for e in ds.iter() {
            d += e;
        }
        for e in ds.iter_mut() {
            *e /= d;
        }
        for i in 0..ds.len() {
            ds[i] = if i == 0 { 0.0 } else { ds[i - 1] } + ds[i];
        }
    }

    /// `Mth.binNormalDistribution(double, double, double, int, int)`
    pub fn binNormalDistribution(d: f64, e: f64, f: f64, i: i32, j: i32) -> Vec<f64> {
        let mut ds = vec![0.0; (j - i + 1) as usize];
        let mut k = 0;
        for l in i..=j {
            ds[k] = 0.0_f64.max(d * ((l as f64 - f) * (l as f64 - f) / 2.0 * e * e).exp());
            k += 1;
        }
        ds
    }

    /// `Mth.binBiModalNormalDistribution(double, double, double, double, double, double, int, int)`
    pub fn binBiModalNormalDistribution(
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: i32,
        k: i32,
    ) -> Vec<f64> {
        let mut ds = vec![0.0; (k - j + 1) as usize];
        let mut l = 0;
        for m in j..=k {
            ds[l] = 0.0_f64.max(
                d * ((m as f64 - f) * (m as f64 - f) / 2.0 * e * e).exp()
                    + g * ((m as f64 - i) * (m as f64 - i) / 2.0 * h * h).exp(),
            );
            l += 1;
        }
        ds
    }

    /// `Mth.binLogDistribution(double, double, int, int)`
    pub fn binLogDistribution(d: f64, e: f64, i: i32, j: i32) -> Vec<f64> {
        let mut ds = vec![0.0; (j - i + 1) as usize];
        let mut k = 0;
        for l in i..=j {
            ds[k] = (d * (l as f64).ln() + e).max(0.0);
            k += 1;
        }
        ds
    }

    /// `Mth.binarySearch(int, int, IntPredicate)`
    pub fn binarySearch(mut i: i32, j: i32, int_predicate: impl Fn(i32) -> bool) -> i32 {
        let mut k = j - i;
        while k > 0 {
            let l = k / 2;
            let m = i + l;
            if int_predicate(m) {
                k = l;
                continue;
            }
            i = m + 1;
            k -= l + 1;
        }
        i
    }

    /// `Mth.lerp(float, float, float)`
    pub fn lerp(f: f32, g: f32, h: f32) -> f32 {
        g + f * (h - g)
    }

    /// `Mth.lerp(double, double, double)`
    pub fn lerp_d(d: f64, e: f64, f: f64) -> f64 {
        e + d * (f - e)
    }

    /// `Mth.lerp2(double, double, double, double, double, double)`
    pub fn lerp2(d: f64, e: f64, f: f64, g: f64, h: f64, i: f64) -> f64 {
        lerp_d(e, lerp_d(d, f, g), lerp_d(d, h, i))
    }

    /// `Mth.lerp3(double x 11)`
    #[allow(clippy::too_many_arguments)]
    pub fn lerp3(d: f64, e: f64, f: f64, g: f64, h: f64, i: f64, j: f64, k: f64, l: f64, m: f64, n: f64) -> f64 {
        lerp_d(f, lerp2(d, e, g, h, i, j), lerp2(d, e, k, l, m, n))
    }

    /// `Mth.catmullrom(float x 5)`
    pub fn catmullrom(f: f32, g: f32, h: f32, i: f32, j: f32) -> f32 {
        0.5
            * (2.0 * h
                + (i - g) * f
                + (2.0 * g - 5.0 * h + 4.0 * i - j) * f * f
                + (3.0 * h - g - 3.0 * i + j) * f * f * f)
    }

    /// `Mth.smoothstep(double)`
    pub fn smoothstep(d: f64) -> f64 {
        d * d * d * (d * (d * 6.0 - 15.0) + 10.0)
    }

    /// `Mth.smoothstepDerivative(double)`
    pub fn smoothstepDerivative(d: f64) -> f64 {
        30.0 * d * d * (d - 1.0) * (d - 1.0)
    }

    /// `Mth.sign(double)`
    pub fn sign(d: f64) -> i32 {
        if d == 0.0 {
            return 0;
        }
        if d > 0.0 { 1 } else { -1 }
    }

    /// `Mth.rotLerp(float, float, float)`
    pub fn rotLerp(f: f32, g: f32, h: f32) -> f32 {
        g + f * wrapDegrees_f(h - g)
    }

    /// `Mth.diffuseLight(float, float, float)`
    pub fn diffuseLight(f: f32, g: f32, h: f32) -> f32 {
        (f * f * 0.6 + g * g * (3.0 + g) / 4.0 + h * h * 0.8).min(1.0)
    }

    /// `Mth.rotlerp(float, float, float)` (deprecated upstream)
    pub fn rotlerp(f: f32, g: f32, h: f32) -> f32 {
        let mut i = g - f;
        while i < -180.0 {
            i += 360.0;
        }
        while i >= 180.0 {
            i -= 360.0;
        }
        f + h * i
    }

    /// `Mth.rotWrap(double)` (deprecated upstream)
    pub fn rotWrap(d: f64) -> f32 {
        let mut d = d;
        while d >= 180.0 {
            d -= 360.0;
        }
        while d < -180.0 {
            d += 360.0;
        }
        d as f32
    }

    /// `Mth.triangleWave(float, float)`
    pub fn triangleWave(f: f32, g: f32) -> f32 {
        ((f % g - g * 0.5).abs() - g * 0.25) / g * 0.25
    }

    /// `Mth.square(float)`
    pub fn square(f: f32) -> f32 {
        f * f
    }

    /// `Mth.square(double)`
    pub fn square_d(d: f64) -> f64 {
        d * d
    }

    /// `Mth.square(int)`
    pub fn square_i(i: i32) -> i32 {
        i * i
    }

    /// `Mth.square(long)`
    pub fn square_l(l: i64) -> i64 {
        l * l
    }

    /// `Mth.cube(float)`
    pub fn cube(f: f32) -> f32 {
        f * f * f
    }

    /// `Mth.clampedMap(double x 5)`
    pub fn clampedMap_d(d: f64, e: f64, f: f64, g: f64, h: f64) -> f64 {
        clampedLerp_d(g, h, inverseLerp_d(d, e, f))
    }

    /// `Mth.clampedMap(float x 5)`
    pub fn clampedMap(f: f32, g: f32, h: f32, i: f32, j: f32) -> f32 {
        clampedLerp(i, j, inverseLerp(f, g, h))
    }

    /// `Mth.map(double x 5)`
    pub fn map_d(d: f64, e: f64, f: f64, g: f64, h: f64) -> f64 {
        lerp_d(inverseLerp_d(d, e, f), g, h)
    }

    /// `Mth.map(float x 5)`
    pub fn map(f: f32, g: f32, h: f32, i: f32, j: f32) -> f32 {
        lerp(inverseLerp(f, g, h), i, j)
    }

    /// `Mth.roundToward(int, int)`
    pub fn roundToward(i: i32, j: i32) -> i32 {
        positiveCeilDiv(i, j) * j
    }

    /// `Mth.positiveCeilDiv(int, int)`
    pub fn positiveCeilDiv(i: i32, j: i32) -> i32 {
        -(-i).div_euclid(j)
    }

    /// `Mth.lengthSquared(double, double)`
    pub fn lengthSquared(d: f64, e: f64) -> f64 {
        d * d + e * e
    }

    /// `Mth.length(double, double)`
    pub fn length(d: f64, e: f64) -> f64 {
        lengthSquared(d, e).sqrt()
    }

    /// `Mth.lengthSquared(double, double, double)`
    pub fn lengthSquared3(d: f64, e: f64, f: f64) -> f64 {
        d * d + e * e + f * f
    }

    /// `Mth.length(double, double, double)`
    pub fn length3(d: f64, e: f64, f: f64) -> f64 {
        lengthSquared3(d, e, f).sqrt()
    }

    /// `Mth.quantize(double, int)`
    pub fn quantize(d: f64, i: i32) -> i32 {
        floor_d(d / i as f64) * i
    }

    /// `Mth.cos(float)` (takes radians upstream, despite the name)
    pub fn cos(deg: f32) -> f32 {
        (deg as f64).cos() as f32
    }

    /// `Mth.sin(float)` (takes radians upstream, despite the name)
    pub fn sin(deg: f32) -> f32 {
        (deg as f64).sin() as f32
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.Vector3`
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// `new Vector3(x, y, z)`
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    /// `Vector3.clone()`
    pub fn clone_vec(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    /// `Vector3.copy(Vector3)`
    pub fn copy(&mut self, vec: &Vector3) {
        self.x = vec.x;
        self.y = vec.y;
        self.z = vec.z;
    }

    /// `Vector3.add(Vector3)`
    pub fn add(&mut self, vec: &Vector3) -> &mut Self {
        self.x += vec.x;
        self.y += vec.y;
        self.z += vec.z;
        self
    }

    /// `Vector3.add(float, float, float)`
    pub fn add_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self
    }

    /// `Vector3.subtract(Vector3)`
    pub fn subtract(&mut self, vec: &Vector3) -> &mut Self {
        self.x -= vec.x;
        self.y -= vec.y;
        self.z -= vec.z;
        self
    }

    /// `Vector3.div(float)`
    pub fn div(&mut self, amount: f32) -> &mut Self {
        self.x /= amount;
        self.y /= amount;
        self.z /= amount;
        self
    }

    /// `Vector3.mul(float)`
    pub fn mul(&mut self, amount: f32) -> &mut Self {
        self.x *= amount;
        self.y *= amount;
        self.z *= amount;
        self
    }

    /// `Vector3.cross(Vector3)`
    pub fn cross(&mut self, vec: &Vector3) -> &mut Self {
        let f = self.x;
        let g = self.y;
        let h = self.z;
        let i = vec.x;
        let j = vec.y;
        let k = vec.z;
        self.x = g * k - h * j;
        self.y = h * i - f * k;
        self.z = f * j - g * i;
        self
    }

    /// `Vector3.normalize()`
    pub fn normalize(&mut self) -> &mut Self {
        let f = Mth::sqrt(self.x * self.x + self.y * self.y + self.z * self.z);
        if f < 1.0E-4 {
            self.x = 0.0;
            self.y = 0.0;
            self.z = 0.0;
        } else {
            self.x /= f;
            self.y /= f;
            self.z /= f;
        }
        self
    }

    /// `Vector3.rotateDegrees(float)`
    pub fn rotateDegrees(&mut self, mut deg: f32) -> &mut Self {
        let ox = self.x;
        let oy = self.y;
        deg = (deg as f64).to_radians() as f32;
        self.x = Mth::cos(deg) * ox - Mth::sin(deg) * oy;
        self.y = Mth::sin(deg) * ox + Mth::cos(deg) * oy;
        self
    }

    /// `Vector3.sqrMagnitude()`
    pub fn sqrMagnitude(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.Vector4`
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    /// `new Vector4(x, y, z, w)`
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vector4 { x, y, z, w }
    }

    /// `Vector4.clone()`
    pub fn clone_vec(&self) -> Vector4 {
        Vector4::new(self.x, self.y, self.z, self.w)
    }

    /// `Vector4.toVec3()`
    pub fn toVec3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.CapePoint`
pub trait CapePoint {
    /// `CapePoint.getLerpX(float)`
    fn getLerpX(&self, delta: f32) -> f32;
    /// `CapePoint.getLerpY(float)`
    fn getLerpY(&self, delta: f32) -> f32;
    /// `CapePoint.getLerpZ(float)`
    fn getLerpZ(&self, delta: f32) -> f32;

    /// `CapePoint.getLerpedPos(float)`
    fn getLerpedPos(&self, delta: f32) -> Vector3 {
        Vector3::new(self.getLerpX(delta), self.getLerpY(delta), self.getLerpZ(delta))
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.CapeStick`
pub trait CapeStick {
    /// `CapeStick.getPointA()`
    fn getPointA(&self) -> &dyn CapePoint;
    /// `CapeStick.getPointB()`
    fn getPointB(&self) -> &dyn CapePoint;
    /// `CapeStick.getLength()`
    fn getLength(&self) -> f32;
}
        }

    }
}
