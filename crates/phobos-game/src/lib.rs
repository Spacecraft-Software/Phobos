// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Game flow around the simulation: game modes and missions, level
//! progression, demo recording and playback, savegames and cheats.
//!
//! Like the simulation, this crate is deterministic: no floating point and no
//! clock reads.

// Determinism (PLAN.md, "Simulation invariants"): no floating point here.
// The clock bans live in this crate's clippy.toml.
#![deny(clippy::float_arithmetic)]
