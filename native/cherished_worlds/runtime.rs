//! cherished_worlds runtime (behavioral port of the LGPL-3.0 Cherished Worlds mod).
//!
//! Upstream lets players favorite (pin/bookmark) worlds so they always sort
//! to the top of the world list and are protected from deletion. This module
//! is the client-side wiring for Horizon's native-mod runtime: it owns the
//! favorite set loaded from the launcher's world index and answers the list
//! sort and delete-guard calls.
//!
//! Ported clean-room under CC0-1.0: no upstream code or assets are included.
//! The behavior described here is the public, observable behavior of the mod.

use crate::Favorites;

/// Runtime state: the favorite set persisted with the launcher's world data.
#[derive(Clone, Debug, Default)]
pub struct CherishedWorldsRuntime {
    pub favorites: Favorites,
}

impl CherishedWorldsRuntime {
    pub fn load(favorites: Favorites) -> Self {
        Self { favorites }
    }

    /// Toggle one world's favorite state and persist the set.
    pub fn toggle(&mut self, key: &str) -> bool {
        let now = self.favorites.toggle(key);
        self.save();
        now
    }

    /// The order the world list renders in.
    pub fn sorted_worlds(&self, worlds: &mut [(String, u64)]) {
        self.favorites.sort_worlds(worlds);
    }

    /// Whether the delete button may act on this world.
    pub fn can_delete(&self, key: &str) -> bool {
        self.favorites.can_delete(key)
    }

    fn save(&self) {
        // Host seam: the launcher persists the set with its world index.
    }
}
