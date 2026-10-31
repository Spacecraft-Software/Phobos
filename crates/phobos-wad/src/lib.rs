// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! WAD archive reading, IWAD identification and the lump namespace.
//!
//! Every byte of a WAD is untrusted input: parsing uses checked bounds and
//! returns typed errors, and never panics on malformed data.
