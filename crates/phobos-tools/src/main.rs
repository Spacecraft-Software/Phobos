// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `phobos-tools` executable: non-interactive inspection and CI tools.
//!
//! Until M1 lands, it answers only `--help` and `--version`.

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
            println!("phobos-tools {}\n{ATTRIBUTION}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-help" | "-h") => {
            println!(
                "Usage: phobos-tools [--help | --version]\n\nNon-interactive tools for WADs, demos and maps. The first commands\n(wad ls, wad identify) arrive with milestone M1.\n\n{ATTRIBUTION}"
            );
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "error: phobos-tools has no commands yet; the first arrive with milestone M1 (try --help)"
            );
            ExitCode::from(2)
        }
    }
}
