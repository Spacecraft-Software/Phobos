// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Archive inspection with serial, directory-ordered output to injected writers.

use crate::{Error, cli::WadAction, json};
use phobos_wad::{GameMission, GameMode, LumpInfo, Wad, WadKind, identify};
use serde::{Serialize, Serializer, ser::SerializeSeq};
use std::io::{self, Write};
use std::path::Path;

// Published WAD directory entries reserve eight bytes for lump names.
const NAME_WIDTH: usize = 8;

/// Reads an archive and writes the selected inspection result.
///
/// # Errors
/// Returns archive, identification, clock, or output errors.
pub(crate) fn run(
    action: WadAction,
    path: &Path,
    json: bool,
    out: &mut impl Write,
) -> Result<(), Error> {
    let wad = Wad::read(path).map_err(Error::failure)?;
    inspect(action, path, &wad, json, out)
}

fn inspect(
    action: WadAction,
    path: &Path,
    wad: &Wad,
    json: bool,
    out: &mut impl Write,
) -> Result<(), Error> {
    match action {
        WadAction::Ls => list(wad, json, out).map_err(Error::failure),
        WadAction::Identify => {
            let file_name = path.file_name().and_then(|name| name.to_str());
            let identity = identify(wad, file_name).map_err(Error::failure)?;
            let data = Identity {
                mode: match identity.mode() {
                    GameMode::Shareware => "shareware",
                    GameMode::Registered => "registered",
                    GameMode::Retail => "retail",
                    GameMode::Commercial => "commercial",
                },
                mission: match identity.mission() {
                    GameMission::Doom => "doom",
                    GameMission::Doom2 => "doom2",
                    GameMission::PackTnt => "pack_tnt",
                    GameMission::PackPlut => "pack_plut",
                },
                freedoom: identity.is_freedoom(),
            };
            if json {
                json::write(out, action.name(), data)
            } else {
                writeln!(
                    out,
                    "mode: {}\nmission: {}\nfreedoom: {}",
                    data.mode,
                    data.mission,
                    if data.freedoom { "yes" } else { "no" }
                )
            }
            .map_err(Error::failure)
        }
    }
}

#[derive(Debug, Serialize)]
struct Identity {
    mode: &'static str,
    mission: &'static str,
    freedoom: bool,
}

#[derive(Debug, Serialize)]
struct Directory<'a> {
    kind: &'static str,
    lumps: Lumps<'a>,
}

#[derive(Debug)]
struct Lumps<'a>(&'a [LumpInfo]);

#[derive(Debug, Serialize)]
struct Lump {
    index: usize,
    name: String,
    offset: u32,
    size: u32,
}

impl Serialize for Lumps<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Stream from the validated directory, without allocating a second lump array.
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for (index, lump) in self.0.iter().enumerate() {
            seq.serialize_element(&Lump {
                index,
                name: lump.name().to_string(),
                offset: lump.offset(),
                size: lump.size(),
            })?;
        }
        seq.end()
    }
}

fn list(wad: &Wad, json: bool, out: &mut impl Write) -> io::Result<()> {
    let kind = match wad.kind() {
        WadKind::Iwad => "IWAD",
        WadKind::Pwad => "PWAD",
    };
    if json {
        return json::write(
            out,
            WadAction::Ls.name(),
            Directory {
                kind,
                lumps: Lumps(wad.lumps()),
            },
        );
    }
    let index_width = wad.lumps().len().saturating_sub(1).to_string().len();
    let size_width = wad
        .lumps()
        .iter()
        .map(|lump| lump.size().to_string().len())
        .max()
        .unwrap_or(1);
    let offset_width = wad
        .lumps()
        .iter()
        .map(|lump| lump.offset().to_string().len())
        .max()
        .unwrap_or(1);
    for (index, lump) in wad.lumps().iter().enumerate() {
        // LumpName's Display does not implement padding itself.
        let name = lump.name().to_string();
        writeln!(
            out,
            "{index:>index_width$} {name:<NAME_WIDTH$} {:>size_width$} {:>offset_width$}",
            lump.size(),
            lump.offset()
        )?;
    }
    writeln!(out, "{} lumps, {kind}", wad.lumps().len())
}

#[cfg(test)]
mod tests {
    use super::{inspect, list, run};
    use crate::cli::WadAction;
    use phobos_wad::Wad;
    use serde_json::{Value, json};
    use std::{io, path::Path};

    fn bytes(names: &[&str]) -> Vec<u8> {
        let mut data = b"IWAD".to_vec();
        data.extend(i32::try_from(names.len()).unwrap().to_le_bytes());
        data.extend(12_i32.to_le_bytes());
        for name in names {
            data.extend(0_i32.to_le_bytes());
            data.extend(0_i32.to_le_bytes());
            let mut raw = [0_u8; 8];
            raw[..name.len()].copy_from_slice(name.as_bytes());
            data.extend(raw);
        }
        data
    }

    #[test]
    fn directory_order_padding_and_kinds() {
        let mut data = bytes(&["PLAYPAL", "X"]);
        data[..4].copy_from_slice(b"PWAD");
        data[12..16].copy_from_slice(&12_i32.to_le_bytes());
        data[16..20].copy_from_slice(&20_i32.to_le_bytes());
        data[28..32].copy_from_slice(&100_i32.to_le_bytes());
        let wad = Wad::from_bytes(data).unwrap();
        let mut out = Vec::new();
        list(&wad, false, &mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "0 PLAYPAL  20  12\n1 X         0 100\n2 lumps, PWAD\n"
        );
        let mut out = Vec::new();
        list(&wad, true, &mut out).unwrap();
        let value: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            value["data"],
            json!({"kind":"PWAD", "lumps":[{"index":0,"name":"PLAYPAL","offset":12,"size":20},{"index":1,"name":"X","offset":100,"size":0}]})
        );
        out.clear();
        list(&Wad::from_bytes(bytes(&[])).unwrap(), false, &mut out).unwrap();
        assert_eq!(out, b"0 lumps, IWAD\n");
        out.clear();
        list(
            &Wad::from_bytes(bytes(&["X"; 11])).unwrap(),
            false,
            &mut out,
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with(" 0 X"));
        assert!(text.contains("\n10 X"));
    }

    #[test]
    fn all_modes_missions_and_freedoom() {
        for (names, path, mode, mission, freedoom) in [
            (&["E1M1"][..], "doom1.wad", "shareware", "doom", false),
            (&["E3M1"][..], "doom.wad", "registered", "doom", false),
            (
                &["E4M1", "FREEDOOM"][..],
                "freedoom1.wad",
                "retail",
                "doom",
                true,
            ),
            (
                &["MAP01"][..],
                "sub/doom2.wad",
                "commercial",
                "doom2",
                false,
            ),
            (
                &["MAP01"][..],
                "sub/TNT.WAD",
                "commercial",
                "pack_tnt",
                false,
            ),
            (
                &["MAP01", "FREEDM"][..],
                "sub/plutonia.wad",
                "commercial",
                "pack_plut",
                true,
            ),
        ] {
            let wad = Wad::from_bytes(bytes(names)).unwrap();
            let mut out = Vec::new();
            inspect(WadAction::Identify, Path::new(path), &wad, false, &mut out).unwrap();
            assert_eq!(
                String::from_utf8(out).unwrap(),
                format!(
                    "mode: {mode}\nmission: {mission}\nfreedoom: {}\n",
                    if freedoom { "yes" } else { "no" }
                )
            );
            let mut out = Vec::new();
            inspect(WadAction::Identify, Path::new(path), &wad, true, &mut out).unwrap();
            let value: Value = serde_json::from_slice(&out).unwrap();
            assert_eq!(
                value["data"],
                json!({"mode":mode,"mission":mission,"freedoom":freedoom})
            );
            assert_eq!(value["metadata"]["command"], "wad identify");
        }
    }

    #[test]
    fn identification_and_output_errors() {
        let wad = Wad::from_bytes(bytes(&[])).unwrap();
        for json in [false, true] {
            let error = inspect(
                WadAction::Identify,
                Path::new("empty.wad"),
                &wad,
                json,
                &mut Vec::new(),
            )
            .unwrap_err();
            assert!(error.is_failure());
            assert!(error.to_string().contains("no map marker"));
            let wad = Wad::from_bytes(bytes(&["E1M1"])).unwrap();
            for action in [WadAction::Ls, WadAction::Identify] {
                let mut full = io::Cursor::new([0_u8; 0]);
                assert!(
                    inspect(action, Path::new("test.wad"), &wad, json, &mut full)
                        .unwrap_err()
                        .is_failure()
                );
            }
        }
    }

    #[test]
    fn read_and_parse_errors() {
        let dir = std::env::temp_dir().join(format!("phobos-tools-unit-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("invalid.wad");
        assert!(
            run(WadAction::Ls, &path, false, &mut Vec::new())
                .unwrap_err()
                .is_failure()
        );
        let mut cases = vec![vec![], b"invalid header".to_vec()];
        for (count, offset) in [(-1_i32, 12_i32), (1, 12), (i32::MAX, i32::MAX)] {
            let mut data = b"IWAD".to_vec();
            data.extend(count.to_le_bytes());
            data.extend(offset.to_le_bytes());
            cases.push(data);
        }
        for (offset, size) in [(-1_i32, 0_i32), (0, -1), (28, 1), (i32::MAX, i32::MAX)] {
            let mut data = bytes(&["X"]);
            data[12..16].copy_from_slice(&offset.to_le_bytes());
            data[16..20].copy_from_slice(&size.to_le_bytes());
            cases.push(data);
        }
        for data in cases {
            std::fs::write(&path, data).unwrap();
            for action in [WadAction::Ls, WadAction::Identify] {
                let mut out = Vec::new();
                assert!(run(action, &path, true, &mut out).unwrap_err().is_failure());
                assert!(out.is_empty());
            }
        }
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
