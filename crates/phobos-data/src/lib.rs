// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Decoders that turn raw Doom lumps into typed map, graphics and sound data.
//!
//! Decoders take a lump's bytes and validate every cross-reference (a
//! sidedef index, a patch offset) before it is used, reporting problems as a
//! [`DataError`]. This crate does not open WADs: callers hand it lump bytes.

#[allow(
    dead_code,
    reason = "scaffold for the M1 decoders; removed when they merge"
)]
mod error;
pub mod graphics;
pub mod map;
#[allow(
    dead_code,
    reason = "scaffold for the M1 decoders; removed when they merge"
)]
mod reader;
pub mod sound;

#[doc(inline)]
pub use error::DataError;
