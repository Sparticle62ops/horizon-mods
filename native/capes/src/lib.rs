//! capes (Horizon Rust port), 1:1.
//!
//! Upstream: https://github.com/CaelTheColher/Capes
//! Copyright (C) CaelTheColher and contributors. Licensed under LGPL-2.1; this
//! port carries the upstream license and credit per the port policy.
//!
//! Upstream logic: 14 Kotlin sources (`Capes.kt`, `CapeConfig.kt`,
//! `CapeType.kt`, `util/Utils.kt`, `handler/PlayerHandler.kt` + data,
//! `menu/*.kt`, `render/PlaceholderEntity*.kt`) and 5 client mixins
//! (`MixinPlayerInfo`, `MixinCapeFeatureRenderer`,
//! `MixinEntityRenderDispatcher`, `AccessorEntityRenderDispatcher`,
//! `MixinSkinCustomizationScreen`). Engine-facing surfaces (screens, rendering,
//! textures, HTTP, config file) run behind seams mirroring the upstream calls.
// Clippy noise from the literal 1:1 translation and its tests; the code
// stays upstream-shaped.
#![allow(
    clippy::manual_pattern_char_comparison,
    clippy::bind_instead_of_map,
    clippy::manual_strip,
    clippy::eq_op,
    clippy::match_single_binding,
    clippy::unnecessary_unwrap,
    clippy::same_item_push,
    clippy::field_reassign_with_default,
    clippy::bool_assert_comparison,
    non_snake_case
)]

pub mod capes_mod;
pub mod config;
pub mod handler;
pub mod menus;
pub mod mixins;
pub mod render;
#[cfg(test)]
pub mod tests;
pub mod util;
