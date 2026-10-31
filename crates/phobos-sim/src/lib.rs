// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The deterministic id Tech 1 game simulation.
//!
//! A tic advances the world by one 35 Hz step from player commands only.
//! The simulation uses fixed-point arithmetic, contains no floating point,
//! reads no clock, and runs on a single thread: its update order decides the
//! random-number sequence that demo playback depends on.

// Determinism (PLAN.md, "Simulation invariants"): no floating point here.
// The clock bans live in this crate's clippy.toml.
#![deny(clippy::float_arithmetic)]
