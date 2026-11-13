# continuity

Behavioral clean-room port of [Continuity](https://modrinth.com/mod/continuity) (LGPL-3.0-only) for the Horizon native mod API. No upstream code was used; ported under CC0-1.0 per the Horizon modport policy.

## What this carries

- OptiFine-style `.properties` CTM rule parsing: `matchBlocks`, `method` (`ctm`, `ctm_compact`, `horizontal`, `vertical`, `random`), `tiles` as names or numeric ranges, `#` comments, unknown keys ignored.
- Neighbor-mask computation over the 8-neighborhood and per-method tile selection:
  - `horizontal` / `vertical`: 4-tile layouts (none, one side, other side, both).
  - `ctm_compact`: 5-tile orthogonal logic (none, edge, outer corner, straight, center).
  - `ctm`: canonical 47-tile layout, orthogonal base cases refined by inner-corner diagonals in the surrounded row.
  - `random`: deterministic seeded tile pick.
- First-matching-rule-wins resolution across a pack's rules.

## Host seams (degradations, tracked upstream)

The resource-pack texture loading, atlas packing, and the block-model quad override that actually swap UVs at render time need a Horizon hook for per-block-state quad UV override driven by pack textures. Without it the policy logic above runs but nothing renders connected; this is tracked in horizon-rust/horizon issue #15369 (open umbrella: connected textures pack formats) and the port upgrades to full 1:1 when it lands.

## Tests

`cargo test`: 17 tests cover rule parsing, every method's tile mapping, inner-corner refinement, and seeded random selection.

Credits: sparticle.
