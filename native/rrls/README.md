# rrls

Exact 1:1 Rust translation of [Remove Reloading Screen](https://modrinth.com/mod/rrls) for Horizon.

Upstream: [dima-dencep/rrls](https://github.com/dima-dencep/rrls), copyright 2023 - 2026 dima_dencep, licensed under the [Open Software License, Version 3.0](https://spdx.org/licenses/OSL-3.0.txt). This crate translates the upstream Java 1:1 into Rust and carries the upstream license and credit in full: the OSL-3.0 text ships in LICENSE and every source file keeps the upstream header.

Ported: the core lifecycle (`Rrls`, the epoch-locked `MOD_RELOADERS_FUTURE`), the full config tree (AprilFool calendar math, DoubleLoad, HideType, Type), the utils (Ease with the upstream enum ids, Easing with the literal formulas, rainbow, WaitingSharedState, texture and atlas utils, OverlayHelper, DummyGuiGraphics + GuiGraphicsGenerator), the reloader service, the loading screens, and every mixin: core (Gui, MinecraftClient, LoadingOverlay, ducks), animation, compat, and all nine workaround mixins (CommandEncoder, CompilationCache, TextureManager, EntityRenderDispatcher, GuiGraphicsExtractor, ReloadableResourceManager, AbstractTexture; GlCommandEncoder and VulkanRenderPass stay as upstream's commented-out stubs). 98 unit tests cover the translated logic; host seams are marked where Horizon supplies the Minecraft/Fabric equivalents.

## Degradations (Horizon API gaps)

- The async/background pack loading itself is a host seam: Horizon's client loads resource packs synchronously and owns its loading overlay. This port exposes the tracker and the overlay policy; the client wiring targets the client game module via `mixins/rrls/horizon-mixin.toml`. Tracked as horizon-rust/horizon #15868.
- The RGB text tint is exposed as a cycle step for the client's text renderer; no glyph-shader hook exists yet.

Credits: upstream by dima_dencep (OSL-3.0); Rust port by sparticle.
