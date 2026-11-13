//! continuity runtime (behavioral port of the LGPL-3.0 Continuity mod).
//!
//! Upstream lets resource packs define OptiFine-style connected-texture rules
//! and renders connected textures across neighboring blocks. This module is
//! the client-side wiring for Horizon's native-mod runtime: it owns the parsed
//! rule set and answers, for one block position and its neighbors, which tile
//! the quad should sample.
//!
//! Ported clean-room under CC0-1.0: no upstream code or assets are included.
//! The behavior described here is the public, observable behavior of the mod.

use crate::config::Rule;
use crate::ctm::Neighbors;

/// Per-pack state: every parsed CTM rule, first match wins at query time.
#[derive(Clone, Debug, Default)]
pub struct ContinuityRuntime {
    pub rules: Vec<Rule>,
}

impl ContinuityRuntime {
    pub fn load(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    /// Load rules from OptiFine-style `.properties` rule text; entries without
    /// blocks or tiles are skipped, so packs keep loading on unknown sections.
    pub fn load_properties(&mut self, text: &str) {
        for chunk in text.split("\n\n") {
            if let Some(rule) = Rule::parse(chunk) {
                self.rules.push(rule);
            }
        }
    }

    /// The tile the block's quads should sample at this position, or None when
    /// no rule applies and the pack's default mapping stands.
    pub fn tile_for(&self, block: &str, neighbors: &Neighbors, seed: u64) -> Option<String> {
        let rule = self.rules.iter().find(|r| r.matches(block))?;
        let idx = crate::ctm::select(rule, neighbors, seed);
        Some(rule.tile(idx))
    }
}
