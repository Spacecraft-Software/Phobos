// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fixed-point arithmetic, binary angles, trigonometry tables and the random
//! tables of id Tech 1.
//!
//! Everything here is bit-exact with the 1997 source release, because demo
//! playback depends on it. This crate contains no floating point and reads
//! no clock.

// Determinism (PLAN.md, "Simulation invariants"): no floating point here.
// The clock bans live in this crate's clippy.toml.
#![deny(clippy::float_arithmetic)]
