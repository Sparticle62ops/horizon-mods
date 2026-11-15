//! 1:1 translation of `dev.tr7zw.waveycapes.WaveyCapesBase` and
//! `WaveyCapesMod` (the fabric branch; the forge/neoforge bootstrap variants
//! are inactive in this tree).

#![allow(non_snake_case, dead_code, unused_variables)]

use std::sync::{Arc, Mutex, OnceLock};

use crate::renderlayers::CapeLayerFallback;
use crate::render::CapeNodeCollector;
use crate::support::{self, SupportManager};
use crate::versionless::modbase::{self, ModBase, LOGGER};
use crate::versionless::nms::MinecraftPlayer;
use crate::versionless::util::Vector3;

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
        let entity: &dyn crate::support::LivingEntity = player;
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
pub use crate::config_screen::WaveyCapesConfigScreen;
