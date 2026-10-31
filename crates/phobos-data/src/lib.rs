// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Decoders that turn raw Doom lumps into typed map, graphics and sound data.
//!
//! Decoders validate every cross-reference (a sidedef index, a patch offset)
//! before it is used, and report problems as typed errors.
