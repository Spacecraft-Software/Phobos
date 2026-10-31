// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sound effects, the MUS music sequencer and the OPL music backend.
//!
//! Audio runs on its own thread and is fed from the game through a bounded
//! queue that never blocks a tic.
