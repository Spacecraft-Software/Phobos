// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Decodes every classic map in both Freedoom IWADs.

#![cfg(test)]

use std::{fs, path::Path};

use phobos_data::map::{MAP_LUMP_NAMES, MapData, MapLumps};

// The WAD format has a 12-byte header and 16-byte directory entries.
const HEADER_SIZE: usize = 12;
const DIRECTORY_ENTRY_SIZE: usize = 16;

fn read_wad(path: &Path) -> Vec<(String, Vec<u8>)> {
    let bytes = fs::read(path).unwrap();
    assert!(bytes.len() >= HEADER_SIZE);
    assert_eq!(&bytes[..4], b"IWAD");
    let number = |offset| {
        usize::try_from(u32::from_le_bytes(
            bytes[offset..offset + 4].try_into().unwrap(),
        ))
        .unwrap()
    };
    let count = number(4);
    let directory = number(8);
    (0..count)
        .map(|index| {
            let entry = directory + index * DIRECTORY_ENTRY_SIZE;
            let offset = number(entry);
            let length = number(entry + 4);
            let name = &bytes[entry + 8..entry + DIRECTORY_ENTRY_SIZE];
            let end = name
                .iter()
                .position(|&byte| byte == 0)
                .unwrap_or(name.len());
            (
                String::from_utf8(name[..end].to_vec()).unwrap(),
                bytes[offset..offset + length].to_vec(),
            )
        })
        .collect()
}

fn is_map_marker(name: &str, episode: bool) -> bool {
    let bytes = name.as_bytes();
    if episode {
        matches!(bytes, [b'E', b'0'..=b'9', b'M', b'0'..=b'9'])
    } else {
        matches!(bytes, [b'M', b'A', b'P', b'0'..=b'9', b'0'..=b'9'])
    }
}

#[test]
fn every_freedoom_map_decodes() {
    let Some(directory) = std::env::var_os("PHOBOS_FREEDOOM_DIR") else {
        eprintln!("skipped Freedoom map decoding: PHOBOS_FREEDOOM_DIR is unset");
        return;
    };
    for (filename, episode) in [("freedoom1.wad", true), ("freedoom2.wad", false)] {
        let entries = read_wad(&Path::new(&directory).join(filename));
        let mut count = 0;
        for (index, (name, _)) in entries.iter().enumerate() {
            if !is_map_marker(name, episode) {
                continue;
            }
            count += 1;
            let lumps = &entries[index + 1..index + 1 + MAP_LUMP_NAMES.len()];
            for ((actual, _), expected) in lumps.iter().zip(MAP_LUMP_NAMES) {
                assert_eq!(actual, expected, "{filename}: {name}");
            }
            let map = MapLumps {
                things: &lumps[0].1,
                linedefs: &lumps[1].1,
                sidedefs: &lumps[2].1,
                vertexes: &lumps[3].1,
                segs: &lumps[4].1,
                ssectors: &lumps[5].1,
                nodes: &lumps[6].1,
                sectors: &lumps[7].1,
                reject: &lumps[8].1,
                blockmap: &lumps[9].1,
            };
            let decoded =
                MapData::decode(&map).unwrap_or_else(|error| panic!("{filename}: {name}: {error}"));
            assert!(!decoded.vertexes.is_empty(), "{filename}: {name}");
        }
        assert!(count > 0, "no map markers found in {filename}");
        eprintln!("decoded {count} maps in {filename}");
    }
}
