// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Decode graphics in both Freedoom IWADs without depending on phobos-wad.

#![cfg(test)]

use std::path::Path;

use phobos_data::graphics::{Colormaps, Endoom, Flat, Palettes, Patch, PatchNames, TextureDefs};

// The WAD format has a 12-byte header and 16-byte directory entries.
const HEADER_BYTES: usize = 12;
const DIRECTORY_BYTES: usize = 16;

fn read_wad(bytes: &[u8]) -> Vec<([u8; 8], &[u8])> {
    assert!(bytes.len() >= HEADER_BYTES);
    assert_eq!(&bytes[..4], b"IWAD");
    let count = usize::try_from(i32::from_le_bytes(bytes[4..8].try_into().unwrap())).unwrap();
    let directory = usize::try_from(i32::from_le_bytes(bytes[8..12].try_into().unwrap())).unwrap();
    let mut lumps = Vec::with_capacity(count);
    for index in 0..count {
        let start = directory + index * DIRECTORY_BYTES;
        let entry = &bytes[start..start + DIRECTORY_BYTES];
        let offset = usize::try_from(i32::from_le_bytes(entry[..4].try_into().unwrap())).unwrap();
        let length = usize::try_from(i32::from_le_bytes(entry[4..8].try_into().unwrap())).unwrap();
        let name = entry[8..16].try_into().unwrap();
        lumps.push((name, &bytes[offset..offset + length]));
    }
    lumps
}

fn required<'a>(lumps: &[([u8; 8], &'a [u8])], name: [u8; 8]) -> &'a [u8] {
    lumps
        .iter()
        .rev()
        .find(|(candidate, _)| *candidate == name)
        .unwrap()
        .1
}

fn check_patch(iwad_path: &Path, name: [u8; 8], data: &[u8]) {
    let patch = Patch::decode(data)
        .unwrap_or_else(|err| panic!("{}: {name:?}: {err}", iwad_path.display()));
    for x in 0..usize::from(patch.width) {
        for post in patch.column(x).unwrap() {
            assert_eq!(patch.post_pixels(post).len(), usize::from(post.len));
        }
    }
    assert!(patch.column(usize::from(patch.width)).is_none());
}

fn check_iwad(path: &Path) {
    let bytes = std::fs::read(path).unwrap();
    let lumps = read_wad(&bytes);
    assert_eq!(
        Palettes::decode(required(&lumps, *b"PLAYPAL\0"))
            .unwrap()
            .palettes
            .len(),
        14
    );
    assert!(
        Colormaps::decode(required(&lumps, *b"COLORMAP"))
            .unwrap()
            .maps
            .len()
            >= 34
    );
    let names = PatchNames::decode(required(&lumps, *b"PNAMES\0\0")).unwrap();
    TextureDefs::decode(required(&lumps, *b"TEXTURE1")).unwrap();
    for (name, data) in &lumps {
        if name == b"TEXTURE2" {
            TextureDefs::decode(data).unwrap();
        }
    }
    for name in names.names {
        if let Some((_, data)) = lumps.iter().rev().find(|(candidate, _)| *candidate == name) {
            check_patch(path, name, data);
        }
    }
    let mut sprites = false;
    let mut flats = false;
    let mut sprite_count = 0;
    let mut flat_count = 0;
    for (name, data) in &lumps {
        match name {
            b"S_START\0" => sprites = true,
            b"S_END\0\0\0" => sprites = false,
            b"F_START\0" => flats = true,
            b"F_END\0\0\0" => flats = false,
            _ if !data.is_empty() => {
                if sprites {
                    check_patch(path, *name, data);
                    sprite_count += 1;
                }
                if flats {
                    Flat::decode(data)
                        .unwrap_or_else(|err| panic!("{}: {name:?}: {err}", path.display()));
                    flat_count += 1;
                }
            }
            _ => {}
        }
    }
    assert!(sprite_count > 0);
    assert!(flat_count > 0);
    assert!(!sprites && !flats);
    Endoom::decode(required(&lumps, *b"ENDOOM\0\0")).unwrap();
}

#[test]
fn all_freedoom_graphics_decode() {
    let Some(directory) = std::env::var_os("PHOBOS_FREEDOOM_DIR") else {
        eprintln!("skipped Freedoom graphics: PHOBOS_FREEDOOM_DIR is unset");
        return;
    };
    for filename in ["freedoom1.wad", "freedoom2.wad"] {
        check_iwad(&Path::new(&directory).join(filename));
    }
}
