// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Writes seed corpora for the cargo-fuzz targets from the Freedoom IWADs.
//!
//! An empty corpus leaves libFuzzer guessing at magic numbers and record
//! layouts; seeds cut from real lumps let it start from valid inputs and
//! mutate them. CI runs this before the fuzz job (PLAN.md P-021).
//!
//! Usage: `cargo run -p phobos-wad --example fuzz_seeds -- <freedoom-dir> <fuzz/corpus>`

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use phobos_wad::{LumpName, Wad};

/// The map lumps that follow a map marker, in on-disk order.
const MAP_LUMPS: usize = 10;
/// The map fuzz target reads one u16 length per map lump.
const MAP_HEADER: usize = 2 * MAP_LUMPS;
/// Seeds per kind of lump, so the corpus stays small enough for a 60 s run.
const PER_KIND: usize = 24;
/// Upper bound on a seed WAD's lump data, keeping seeds under `-max_len`.
const SEED_WAD_BYTES: usize = 60_000;
/// Size of a WAD header and of one directory entry.
const WAD_HEADER: usize = 12;
const WAD_ENTRY: usize = 16;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let (Some(freedoom), Some(corpus)) = (args.next(), args.next()) else {
        return Err("usage: fuzz_seeds <freedoom-dir> <fuzz/corpus>".into());
    };
    let corpus = PathBuf::from(corpus);
    for target in [
        "wad_directory",
        "map_decode",
        "graphics_decode",
        "sound_decode",
    ] {
        fs::create_dir_all(corpus.join(target))?;
    }
    let mut written = 0;
    for iwad in ["freedoom1", "freedoom2"] {
        let wad = Wad::read(Path::new(&freedoom).join(format!("{iwad}.wad")))?;
        written += wad_seeds(&wad, &corpus.join("wad_directory"), iwad)?;
        written += map_seeds(&wad, &corpus.join("map_decode"), iwad)?;
        written += lump_seeds(&wad, &corpus.join("graphics_decode"), iwad, is_graphics)?;
        written += lump_seeds(&wad, &corpus.join("sound_decode"), iwad, is_sound)?;
    }
    println!(
        "fuzz_seeds: wrote {written} seeds under {}",
        corpus.display()
    );
    Ok(())
}

/// Small PWADs built from runs of consecutive lumps.
fn wad_seeds(wad: &Wad, dir: &Path, iwad: &str) -> Result<usize, Box<dyn Error>> {
    let mut written = 0;
    let mut start = 0;
    while start < wad.lumps().len() && written < PER_KIND {
        let mut end = start;
        let mut bytes = 0;
        while end < wad.lumps().len() {
            let size = wad.lump_data(end).map_or(0, <[u8]>::len);
            if bytes + size > SEED_WAD_BYTES {
                break;
            }
            bytes += size;
            end += 1;
        }
        if end == start {
            start += 1;
            continue;
        }
        fs::write(
            dir.join(format!("{iwad}-{start}.wad")),
            build_pwad(wad, start..end)?,
        )?;
        written += 1;
        start = end;
    }
    Ok(written)
}

fn build_pwad(wad: &Wad, lumps: std::ops::Range<usize>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut data = Vec::new();
    let mut directory = Vec::new();
    for index in lumps.clone() {
        let lump = wad.lump_data(index).unwrap_or_default();
        let offset = i32::try_from(WAD_HEADER + data.len())?;
        directory.extend(offset.to_le_bytes());
        directory.extend(i32::try_from(lump.len())?.to_le_bytes());
        let mut raw = [0_u8; 8];
        let name = wad.lumps()[index].name();
        raw[..name.as_bytes().len()].copy_from_slice(name.as_bytes());
        directory.extend(raw);
        data.extend_from_slice(lump);
    }
    let mut out = Vec::with_capacity(WAD_HEADER + data.len() + lumps.len() * WAD_ENTRY);
    out.extend(b"PWAD");
    out.extend(i32::try_from(lumps.len())?.to_le_bytes());
    out.extend(i32::try_from(WAD_HEADER + data.len())?.to_le_bytes());
    out.extend(data);
    out.extend(directory);
    Ok(out)
}

/// The map target's layout: ten u16 lengths, then the ten lumps.
fn map_seeds(wad: &Wad, dir: &Path, iwad: &str) -> Result<usize, Box<dyn Error>> {
    let things = LumpName::new("THINGS").ok_or("THINGS is a valid name")?;
    let mut written = 0;
    for index in 0..wad.lumps().len() {
        let is_map = wad
            .lumps()
            .get(index + 1)
            .is_some_and(|next| next.name() == things);
        if !is_map || written >= PER_KIND {
            continue;
        }
        let mut header = Vec::with_capacity(MAP_HEADER);
        let mut body = Vec::new();
        for offset in 1..=MAP_LUMPS {
            let lump = wad.lump_data(index + offset).unwrap_or_default();
            let lump = &lump[..lump.len().min(usize::from(u16::MAX))];
            header.extend(u16::try_from(lump.len())?.to_le_bytes());
            body.extend_from_slice(lump);
        }
        header.extend(body);
        let name = wad.lumps()[index].name();
        fs::write(dir.join(format!("{iwad}-{name}")), header)?;
        written += 1;
    }
    Ok(written)
}

fn lump_seeds(
    wad: &Wad,
    dir: &Path,
    iwad: &str,
    wanted: fn(&[u8]) -> Option<&'static str>,
) -> Result<usize, Box<dyn Error>> {
    let mut per_kind = std::collections::HashMap::<&str, usize>::new();
    let mut written = 0;
    for (index, info) in wad.lumps().iter().enumerate() {
        let Some(kind) = wanted(info.name().as_bytes()) else {
            continue;
        };
        let count = per_kind.entry(kind).or_default();
        let data = wad.lump_data(index).unwrap_or_default();
        if *count >= PER_KIND || data.is_empty() {
            continue;
        }
        *count += 1;
        fs::write(dir.join(format!("{iwad}-{index}-{}", info.name())), data)?;
        written += 1;
    }
    Ok(written)
}

fn is_graphics(name: &[u8]) -> Option<&'static str> {
    match name {
        b"PLAYPAL" | b"COLORMAP" | b"PNAMES" | b"TEXTURE1" | b"TEXTURE2" | b"ENDOOM" => {
            Some("table")
        }
        // Wall patches, title screens and status bar graphics share the
        // picture format; sprites too.
        _ if name.starts_with(b"WALL") || name.starts_with(b"ST") => Some("patch"),
        _ if name.starts_with(b"TITLE") || name.starts_with(b"HELP") => Some("screen"),
        _ if name.starts_with(b"FLOOR") || name.starts_with(b"FLAT") => Some("flat"),
        _ => None,
    }
}

fn is_sound(name: &[u8]) -> Option<&'static str> {
    match name {
        b"GENMIDI" => Some("genmidi"),
        _ if name.starts_with(b"DS") => Some("dmx"),
        _ if name.starts_with(b"DP") => Some("pcspeaker"),
        _ if name.starts_with(b"D_") => Some("music"),
        _ => None,
    }
}
