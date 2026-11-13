//! Remove Reloading Screen (rrls) for Horizon (behavioral clean-room port of
//! the OSL-3.0 mod).
//!
//! Upstream lets resource packs load in the background so you can keep doing
//! things while the game reloads, optionally animating the vanilla overlay
//! away or hiding it. The actual async pack loading is a host seam (see
//! README); this crate carries the policy logic: the reload state machine,
//! the overlay-visibility policy, and the config.

pub mod config;
pub mod reload;
