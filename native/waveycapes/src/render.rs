//! 1:1 translation of `dev.tr7zw.waveycapes.render` (`CustomCapeRenderer.java`,
//! `CapeRenderer.java`, `CapeInfos.java`, `CapePass.java`,
//! `VanillaCapeRenderer.java`).
//!
//! The Minecraft render API arrives through the same seam shapes the other
//! ports in this repo use (`PoseStack`, `VertexConsumer`, `ModelPart`,
//! `PlayerWrapper`, `RenderType`, the JOML `Matrix4f`/`Vector4f`).

#![allow(non_snake_case, dead_code)]

use crate::versionless::modbase;
use crate::versionless::sim::{BasicSimulation, CapeMovement, CapeStyle, WindMode};
use crate::versionless::holder::CapeHolder;
use crate::versionless::util::{CapePoint, Vector3};

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
            custom_cape: crate::nms_util::NMSUtil::buildCape(64, 64, Box::new(|_x| 0), Box::new(|y| y)),
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
