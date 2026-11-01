// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fuzzes map decoding: the first 20 bytes give the ten lump lengths, and the
//! rest of the input is cut into the ten lumps in MAP_LUMP_NAMES order.
#![no_main]

use libfuzzer_sys::fuzz_target;
use phobos_data::map::{MapData, MapLumps};

/// Ten little-endian u16 lengths precede the lump bytes.
const HEADER: usize = 20;

fuzz_target!(|data: &[u8]| {
    let Some((header, mut rest)) = data.split_at_checked(HEADER) else {
        return;
    };
    let mut lumps: [&[u8]; 10] = [&[]; 10];
    for (slot, length) in lumps.iter_mut().zip(header.chunks_exact(2)) {
        let length = usize::from(u16::from_le_bytes([length[0], length[1]])).min(rest.len());
        let (lump, tail) = rest.split_at(length);
        *slot = lump;
        rest = tail;
    }
    let [
        things,
        linedefs,
        sidedefs,
        vertexes,
        segs,
        ssectors,
        nodes,
        sectors,
        reject,
        blockmap,
    ] = lumps;
    let _ = MapData::decode(&MapLumps {
        things,
        linedefs,
        sidedefs,
        vertexes,
        segs,
        ssectors,
        nodes,
        sectors,
        reject,
        blockmap,
    });
});
