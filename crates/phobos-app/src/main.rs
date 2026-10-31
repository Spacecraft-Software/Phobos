// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `phobos` executable.
//!
//! Until M3 lands, it answers only `--help` and `--version`.

use std::process::ExitCode;

/// The attribution block every Phobos binary prints (Standard §15.2).
const ATTRIBUTION: &str = "\
Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software  |  License: GPL-3.0-or-later
https://Phobos.SpacecraftSoftware.org/";

fn main() -> ExitCode {
    let first = std::env::args().nth(1);
    match first.as_deref() {
        Some("--version" | "-version") => {
            println!("phobos {}\n{ATTRIBUTION}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-help" | "-h") => {
            println!(
                "Usage: phobos [--help | --version]\n\nPhobos plays Doom, The Ultimate Doom, Doom II and Final Doom from a\nuser-supplied IWAD. It cannot play yet; the game arrives with milestone M3.\n\n{ATTRIBUTION}"
            );
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "error: phobos cannot play yet; the game arrives with milestone M3 (try --help)"
            );
            ExitCode::from(2)
        }
    }
}
