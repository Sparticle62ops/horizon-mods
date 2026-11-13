# cherished-worlds

Behavioral clean-room port of [Cherished Worlds](https://modrinth.com/mod/cherished-worlds) (LGPL-3.0-or-later) for the Horizon native mod API. No upstream code was used; ported under CC0-1.0 per the Horizon modport policy.

## What this carries

- The favorite set keyed by world key (folder name + level name), persisted with the launcher's world index.
- Toggle/unfavorite with a persisted result, count and emptiness accessors.
- Sort order for the world list: favorites first, then most recently played, stable and deterministic.
- Delete guard: a favorite world cannot be deleted until it is unfavored.

## Host seams (degradations, tracked upstream)

The world-list screen, the star/bookmark button, and the confirmation dialog are Horizon launcher UI seams; the port ships the full policy logic (7 tests) and the launcher wires the set into its world index. Without the launcher wiring nothing is visible in-game; tracked as a Horizon tracker item and the port upgrades to full 1:1 when it lands.

## Tests

`cargo test`: 7 tests cover toggle state, sort order with favorites and recency, recency-only sorting, the delete guard, and unknown-key safety.

Credits: sparticle.
