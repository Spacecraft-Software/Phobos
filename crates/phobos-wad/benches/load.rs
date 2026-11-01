// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Baseline benchmark for loading an IWAD (PLAN.md P-022).
//!
//! Measures, on `freedoom2.wad` from `PHOBOS_FREEDOOM_DIR`:
//! - `parse_directory`: header and directory parsing from bytes in memory;
//! - `decode_all`: parsing plus decoding every map, texture definition, patch
//!   and flat, which is the work a level load will do.
//!
//! The load is serial on purpose for this baseline; P-043 decides from these
//! figures whether parallel decoding pays for itself.
//!
//! Run: `PHOBOS_FREEDOOM_DIR=… cargo bench -p phobos-wad`. Without the
//! variable the benchmark prints that it was skipped.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use phobos_data::graphics::{Flat, Patch, PatchNames, TextureDefs};
use phobos_data::map::{MAP_LUMP_NAMES, MapData, MapLumps};
use phobos_wad::{LumpName, LumpNamespace, Wad};

fn freedoom2() -> Option<Vec<u8>> {
    let dir = std::env::var_os("PHOBOS_FREEDOOM_DIR")?;
    std::fs::read(std::path::Path::new(&dir).join("freedoom2.wad")).ok()
}

fn name(text: &str) -> LumpName {
    LumpName::new(text).unwrap_or_else(|| unreachable!("{text} is a valid lump name"))
}

/// Decodes everything a level load touches; returns a count so the work is
/// observable to `black_box`.
fn decode_all(bytes: &[u8]) -> usize {
    let Ok(wad) = Wad::from_bytes(bytes.to_vec()) else {
        return 0;
    };
    let mut decoded = 0;
    let lumps = wad.lumps();
    for (index, info) in lumps.iter().enumerate() {
        let is_map = info.name().as_bytes().starts_with(b"MAP")
            && lumps
                .get(index + 1)
                .is_some_and(|next| next.name() == name(MAP_LUMP_NAMES[0]));
        if !is_map {
            continue;
        }
        let part = |offset: usize| wad.lump_data(index + 1 + offset).unwrap_or_default();
        let map = MapLumps {
            things: part(0),
            linedefs: part(1),
            sidedefs: part(2),
            vertexes: part(3),
            segs: part(4),
            ssectors: part(5),
            nodes: part(6),
            sectors: part(7),
            reject: part(8),
            blockmap: part(9),
        };
        decoded += usize::from(MapData::decode(&map).is_ok());
    }
    let mut namespace = LumpNamespace::new();
    namespace.push(wad);
    let lump = |text: &str| namespace.find(name(text)).and_then(|id| namespace.data(id));
    if let Some(pnames) = lump("PNAMES").and_then(|b| PatchNames::decode(b).ok()) {
        for raw in &pnames.names {
            let patch = namespace
                .find(LumpName::from_raw(*raw))
                .and_then(|id| namespace.data(id));
            decoded += usize::from(patch.is_some_and(|b| Patch::decode(b).is_ok()));
        }
    }
    for texture in ["TEXTURE1", "TEXTURE2"] {
        decoded += usize::from(lump(texture).is_some_and(|b| TextureDefs::decode(b).is_ok()));
    }
    for id in namespace.between(name("F_START"), name("F_END")) {
        decoded += usize::from(namespace.data(id).is_some_and(|b| Flat::decode(b).is_ok()));
    }
    decoded
}

fn bench_load(c: &mut Criterion) {
    let Some(bytes) = freedoom2() else {
        eprintln!("load benchmark skipped: PHOBOS_FREEDOOM_DIR is not set");
        return;
    };
    c.bench_function("parse_directory", |b| {
        b.iter(|| Wad::from_bytes(black_box(bytes.clone())).map(|wad| wad.lumps().len()));
    });
    c.bench_function("decode_all", |b| b.iter(|| decode_all(black_box(&bytes))));
}

criterion_group!(benches, bench_load);
criterion_main!(benches);
