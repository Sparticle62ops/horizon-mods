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

use crate::nms_util::{IntUnaryOperator, NMSUtil};
use crate::render::{
    CapeInfos, CapeRenderer, ModelPart, NO_OVERLAY, PlayerWrapper, PoseStack, RenderTypes,
    VertexConsumer,
};
use crate::versionless::modbase;
use crate::versionless::util::Vector3;

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
    use crate::render::CapeHolderAvatar;
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
                    dyn for<'a> Fn(&'a dyn crate::render::CapeHolderAvatar)
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
            dyn for<'a> Fn(&'a dyn crate::render::CapeHolderAvatar)
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
    pub fn player(wrapper: &dyn crate::render::PlayerWrapper) -> Option<Arc<dyn LabyPlayer>> {
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
