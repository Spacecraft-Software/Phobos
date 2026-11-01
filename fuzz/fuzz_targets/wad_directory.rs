// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fuzzes WAD header and directory parsing, then every lump accessor.
#![no_main]

use libfuzzer_sys::fuzz_target;
use phobos_wad::{LumpNamespace, Wad, identify};

fuzz_target!(|data: &[u8]| {
    let Ok(wad) = Wad::from_bytes(data.to_vec()) else {
        return;
    };
    for index in 0..wad.lumps().len() {
        let _ = wad.lump_data(index);
    }
    let _ = identify(&wad, Some("tnt.wad"));
    let mut namespace = LumpNamespace::new();
    namespace.push(wad);
    if let (Some(start), Some(end)) = (
        phobos_wad::LumpName::new("S_START"),
        phobos_wad::LumpName::new("S_END"),
    ) {
        for id in namespace.between(start, end) {
            let _ = namespace.data(id);
        }
    }
});
