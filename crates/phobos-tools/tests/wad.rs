// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Child-process checks for archive commands and their output contracts.

use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn command(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_phobos-tools"))
        .args(args)
        .output()
}

#[test]
fn freedoom_commands() {
    let Some(dir) = std::env::var_os("PHOBOS_FREEDOOM_DIR") else {
        eprintln!("skipped Freedoom integration test: PHOBOS_FREEDOOM_DIR is unset");
        return;
    };
    let first = Path::new(&dir).join("freedoom1.wad");
    let output = Command::new(env!("CARGO_BIN_EXE_phobos-tools"))
        .args(["wad", "identify", "--json"])
        .arg(first)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["data"]["mode"], "retail");
    assert_eq!(value["data"]["mission"], "doom");
    assert_eq!(value["data"]["freedoom"], true);
    assert!(
        value["metadata"]["timestamp"]
            .as_str()
            .unwrap()
            .ends_with('Z')
    );
    let second = Path::new(&dir).join("freedoom2.wad");
    let output = Command::new(env!("CARGO_BIN_EXE_phobos-tools"))
        .args(["wad", "ls"])
        .arg(second)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .last()
            .unwrap()
            .ends_with("lumps, IWAD")
    );
}

#[test]
fn errors_and_usage_exit_codes() {
    let missing = std::env::temp_dir()
        .join(format!("phobos-tools-missing-{}", std::process::id()))
        .join("missing.wad");
    for json in [false, true] {
        let mut invocation = Command::new(env!("CARGO_BIN_EXE_phobos-tools"));
        invocation.args(["wad", "ls"]).arg(&missing);
        if json {
            invocation.arg("--json");
        }
        let output = invocation.output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            std::str::from_utf8(&output.stderr).unwrap().lines().count(),
            1
        );
        if json {
            let value: Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(value["error"]["command"], "wad ls");
            assert!(
                value["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("could not read WAD")
            );
        } else {
            assert!(output.stderr.starts_with(b"error: "));
        }
    }
    for args in [
        &["wad", "bogus"][..],
        &["wad", "ls"],
        &["wad", "identify", "a", "b"],
        &["wad", "ls", "--bogus", "--json"],
    ] {
        let output = command(args).unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("--help"));
        if args.contains(&"--json") {
            let value: Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(value["error"]["command"], "wad ls");
        }
    }
}

#[test]
fn help_and_version_keep_attribution() {
    for args in [
        &["--help"][..],
        &["-h"],
        &["-help"],
        &["--version"],
        &["-version"],
        &["wad", "--help"],
    ] {
        let output = command(args).unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("Maintained by Mohamed Hammad"));
        assert!(text.contains("https://Phobos.SpacecraftSoftware.org/"));
        if args.iter().any(|arg| arg.contains("help") || *arg == "-h") {
            assert!(text.contains("wad ls"));
            assert!(text.contains("wad identify"));
        }
    }
}
