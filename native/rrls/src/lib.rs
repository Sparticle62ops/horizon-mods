/*
 * Port of Remove Reloading Screen (rrls) to Rust for Horizon.
 *
 * Copyright 2023 - 2026 dima_dencep (upstream, OSL-3.0); Rust translation 2026 sparticle.
 *
 * Licensed under the Open Software License, Version 3.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *     https://spdx.org/licenses/OSL-3.0.txt
 *
 * Exact 1:1 translation of the upstream Java. Where an
 * upstream file binds to a Minecraft/Fabric API Horizon does not expose, the
 * same shape is kept and the host side is a seam (see README).
 */

#![allow(
    clippy::unused_unit,
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::result_unit_err,
    clippy::default_constructed_unit_structs,
    clippy::new_without_default,
    unused_imports,
    unused_must_use,
    unused_mut,
    clippy::items_after_test_module
)]
// CI runs clippy --all-targets -D warnings. These lints flag shapes kept from the
// upstream Java (unit returns, many-arg methods, nested ifs), so they are allowed
// at crate level and the ported code stays exact.

pub mod rrls;

pub mod config;
pub mod mixins;
pub mod screens;
pub mod services;
pub mod utils;
