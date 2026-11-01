// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Checks all sound and music lumps in both Freedoom IWADs.

use std::path::Path;

use phobos_data::sound::{
    DmxSound, GENMIDI_INSTRUMENTS, GenMidi, Music, PcSpeakerSound, decode_music,
};

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

fn wad_lumps(path: &Path) -> TestResult<Vec<(String, Vec<u8>)>> {
    let bytes = std::fs::read(path)?;
    let header = &bytes[..12];
    assert_eq!(&header[..4], b"IWAD");
    let count = usize::try_from(u32::from_le_bytes(header[4..8].try_into()?))?;
    let directory = usize::try_from(u32::from_le_bytes(header[8..12].try_into()?))?;
    bytes[directory..directory + count * 16]
        .chunks_exact(16)
        .map(|entry| {
            let start = usize::try_from(u32::from_le_bytes(entry[..4].try_into()?))?;
            let length = usize::try_from(u32::from_le_bytes(entry[4..8].try_into()?))?;
            let name = &entry[8..16];
            let end = name
                .iter()
                .position(|&byte| byte == 0)
                .unwrap_or(name.len());
            Ok((
                String::from_utf8(name[..end].to_vec())?,
                bytes[start..start + length].to_vec(),
            ))
        })
        .collect()
}

#[test]
fn all_freedoom_sound_lumps_decode() -> TestResult<()> {
    let Some(directory) = std::env::var_os("PHOBOS_FREEDOOM_DIR") else {
        eprintln!("skipped Freedoom sound test: PHOBOS_FREEDOOM_DIR is unset");
        return Ok(());
    };
    for filename in ["freedoom1.wad", "freedoom2.wad"] {
        let mut counts = [0; 4];
        let mut mus_count = 0;
        let mut midi_count = 0;
        for (name, bytes) in wad_lumps(&Path::new(&directory).join(filename))? {
            let result = if name.starts_with("DS") {
                counts[0] += 1;
                DmxSound::decode(&bytes).map(|_| ())
            } else if name.starts_with("DP") {
                counts[1] += 1;
                PcSpeakerSound::decode(&bytes).map(|_| ())
            } else if name.starts_with("D_") {
                counts[2] += 1;
                decode_music(&bytes).map(|music| match music {
                    Music::Mus(_) => mus_count += 1,
                    Music::Midi(_) => midi_count += 1,
                })
            } else if name == "GENMIDI" {
                counts[3] += 1;
                GenMidi::decode(&bytes).map(|bank| {
                    assert_eq!(bank.instruments.len(), GENMIDI_INSTRUMENTS);
                    assert_eq!(bank.names.len(), GENMIDI_INSTRUMENTS);
                })
            } else {
                continue;
            };
            assert!(
                result.is_ok(),
                "{filename}: {name}: {result:?}; signature: {:?}",
                bytes.get(..4)
            );
        }
        assert!(
            counts.iter().all(|&count| count > 0),
            "{filename}: {counts:?}"
        );
        eprintln!(
            "{filename}: DS={}, DP={}, D_={} (MUS={mus_count}, MIDI={midi_count}), GENMIDI={}",
            counts[0], counts[1], counts[2], counts[3]
        );
    }
    Ok(())
}
