// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fuzzes every graphics decoder on the same input.
#![no_main]

use libfuzzer_sys::fuzz_target;
use phobos_data::graphics::{Colormaps, Endoom, Flat, Palettes, Patch, PatchNames, TextureDefs};

fuzz_target!(|data: &[u8]| {
    let _ = Palettes::decode(data);
    let _ = Colormaps::decode(data);
    let _ = PatchNames::decode(data);
    let _ = TextureDefs::decode(data);
    if let Ok(patch) = Patch::decode(data) {
        for x in 0..usize::from(patch.width) {
            for post in patch.column(x).unwrap_or_default() {
                let _ = patch.post_pixels(post);
            }
        }
    }
    let _ = Flat::decode(data);
    let _ = Endoom::decode(data);
});
