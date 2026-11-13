# rrls

Behavioral clean-room port of [Remove Reloading Screen](https://modrinth.com/mod/rrls) for Horizon.

Upstream: dima-dencep's Remove Reloading Screen (OSL-3.0), 28.5M downloads. It loads resource packs in the background so the game stays usable during reloads, with optional RGB text or a fully hidden loading overlay.

**License: OSL-3.0 is not on Horizon's registry allowlist, so this is a behavioral clean-room port. No upstream code is used; the crate is CC0-1.0.**

Ported logic (1:1 behavior):

- Config parsing: `enable`, `hide_type` (vanilla / rgb / hidden), `force_close`, `skip_vanilla_overlay`; unknown keys ignored, malformed values keep defaults.
- Reload lifecycle: a cycle tracks expected resource reloads, finishes when all land, a zero-total cycle finishes immediately, a newer cycle supersedes a live one.
- Overlay policy: vanilla mode keeps the overlay only while a reload is live; rgb mode adds an RGB tint cycling by finished count; hidden mode suppresses the overlay after startup, keeping only the first (startup) load visible.
- Force-close: only when configured, only for a finished, still-current reload.
- Disabled config always renders the vanilla overlay.

13 unit tests cover the state machine and config.

## Degradations (Horizon API gaps)

- The async/background pack loading itself is a host seam: Horizon's client loads resource packs synchronously and owns its loading overlay. This port exposes the tracker (begin/advance/dismiss) and the overlay policy; the client wiring needs a mixin point (`mixins/rrls/horizon-mixin.toml` targets the client game module). Tracked against the Horizon repo's resource-pack loading work (see horizon-rust/horizon #15369 for the connected-texture side of pack format work; the background-reload and overlay-control hook itself is horizon-rust/horizon #15868).
- The RGB text tint is exposed as a cycle step for the client's text renderer; no glyph-shader hook exists yet.

Credits: port by sparticle.
