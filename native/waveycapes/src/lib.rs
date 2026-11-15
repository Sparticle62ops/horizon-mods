//! Wavey Capes, exact 1:1 Rust translation.
//! Upstream: https://github.com/tr7zw/WaveyCapes (tr7zw Protective License).
//! Upstream logic: 27 main Java sources (`WaveyCapesBase`, `WaveyCapesMod`,
//! `WaveyCapesBootstrap`, `WaveyCapesConfigScreen`, `WaveyCapesModMenu`,
//! `NMSUtil`, `render/*`, `renderlayers/CustomCapeRenderLayer`,
//! `delegate/PlayerDelegate`, `support/*`, `mixin/*`) plus the versionless
//! core (`WCVersionless`: `CapeMovement`, `CapeHolder`, `CapeStyle`,
//! `WindMode`, `ModBase`, `sim/*`, `util/*`, `config/*`, `nms/MinecraftPlayer`).

pub mod versionless;
pub mod render;
pub mod nms_util;
pub mod renderlayers;
pub mod support;
pub mod delegate;
pub mod mixins;
pub mod wavey_capes_base;
pub mod config_screen;
