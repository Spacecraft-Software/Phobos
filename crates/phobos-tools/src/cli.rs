// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hand-written command parsing that preserves native file paths.

use crate::Error;
use std::ffi::OsString;
use std::path::PathBuf;

/// An archive inspection action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WadAction {
    /// Lists directory entries.
    Ls,
    /// Identifies the game.
    Identify,
}

impl WadAction {
    /// Returns the command name used in metadata and diagnostics.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Ls => "wad ls",
            Self::Identify => "wad identify",
        }
    }
}

/// A fully validated invocation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Prints general or archive help.
    Help {
        /// Selects archive help.
        wad: bool,
    },
    /// Prints version and attribution.
    Version,
    /// Inspects the supplied archive.
    Wad {
        /// Selects the archive operation.
        action: WadAction,
        /// Preserves the native path encoding.
        path: PathBuf,
        /// Selects JSON output.
        json: bool,
    },
}

/// Returns diagnostic context even when argument parsing fails.
pub(crate) fn context(args: &[OsString]) -> (&'static str, bool) {
    if args.first().is_none_or(|arg| arg != "wad") {
        return ("phobos-tools", false);
    }
    let rest = &args[1..];
    let command = match rest
        .iter()
        .find(|arg| *arg != "--json")
        .and_then(|arg| arg.to_str())
    {
        Some("ls") => "wad ls",
        Some("identify") => "wad identify",
        _ => "wad",
    };
    (command, rest.iter().any(|arg| arg == "--json"))
}

/// Parses arguments after the executable name.
///
/// # Errors
/// Returns a usage error for unknown commands, options, or wrong file counts.
pub(crate) fn parse(args: &[OsString]) -> Result<Command, Error> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("--help" | "-h" | "-help") => Ok(Command::Help { wad: false }),
        Some("--version" | "-version") => Ok(Command::Version),
        Some("wad") => parse_wad(&args[1..]),
        Some(other) => Err(Error::usage(format!("unknown command or option: {other}"))),
        None => Err(Error::usage("missing command")),
    }
}

fn parse_wad(args: &[OsString]) -> Result<Command, Error> {
    let mut positional = args.iter().filter(|arg| *arg != "--json");
    let action = match positional.next().and_then(|arg| arg.to_str()) {
        Some("--help" | "-h" | "-help") => return Ok(Command::Help { wad: true }),
        Some("ls") => WadAction::Ls,
        Some("identify") => WadAction::Identify,
        Some(other) => return Err(Error::usage(format!("unknown wad command: {other}"))),
        None => return Err(Error::usage("missing wad command")),
    };
    let mut path = None;
    for arg in positional {
        if matches!(arg.to_str(), Some("--help" | "-h" | "-help")) {
            return Ok(Command::Help { wad: true });
        }
        if arg.as_encoded_bytes().starts_with(b"-") {
            return Err(Error::usage(format!("unknown option: {}", arg.display())));
        }
        if path.replace(PathBuf::from(arg)).is_some() {
            return Err(Error::usage("expected exactly one file argument"));
        }
    }
    let path = path.ok_or_else(|| Error::usage("missing file argument"))?;
    Ok(Command::Wad {
        action,
        path,
        json: args.iter().any(|arg| arg == "--json"),
    })
}

#[cfg(test)]
mod tests {
    use super::{Command, WadAction, context, parse};
    use std::ffi::OsString;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn preserves_help_and_version_aliases() {
        for flag in ["--help", "-help", "-h"] {
            assert_eq!(parse(&args(&[flag])).unwrap(), Command::Help { wad: false });
            assert_eq!(
                parse(&args(&["wad", flag])).unwrap(),
                Command::Help { wad: true }
            );
        }
        for flag in ["--version", "-version"] {
            assert_eq!(parse(&args(&[flag, "ignored"])).unwrap(), Command::Version);
        }
    }

    #[test]
    fn json_can_appear_anywhere_after_wad() {
        for action in [WadAction::Ls, WadAction::Identify] {
            let verb = action.name().strip_prefix("wad ").unwrap();
            for rest in [
                vec!["--json", verb, "test.wad"],
                vec![verb, "--json", "test.wad"],
                vec![verb, "test.wad", "--json"],
            ] {
                let mut values = vec!["wad"];
                values.extend(rest);
                let values = args(&values);
                assert_eq!(context(&values), (action.name(), true));
                assert_eq!(
                    parse(&values).unwrap(),
                    Command::Wad {
                        action,
                        path: "test.wad".into(),
                        json: true
                    }
                );
            }
            assert_eq!(
                parse(&args(&["wad", verb, "test.wad"])).unwrap(),
                Command::Wad {
                    action,
                    path: "test.wad".into(),
                    json: false
                }
            );
        }
    }

    #[test]
    fn rejects_every_usage_error() {
        for values in [
            &[][..],
            &["bogus"],
            &["--bogus"],
            &["wad"],
            &["wad", "--json"],
            &["wad", "bogus"],
            &["wad", "ls"],
            &["wad", "identify"],
            &["wad", "ls", "a", "b"],
            &["wad", "ls", "--bogus"],
            &["wad", "ls", "a", "--bogus"],
            &["wad", "identify", "a", "b"],
        ] {
            let error = parse(&args(values)).unwrap_err();
            assert!(error.is_usage());
            assert!(error.to_string().contains("--help"));
        }
        assert_eq!(
            context(&args(&["wad", "ls", "--bogus", "--json"])),
            ("wad ls", true)
        );
    }

    #[cfg(unix)]
    #[test]
    fn preserves_non_utf8_paths() {
        use std::os::unix::ffi::OsStringExt;
        let path = OsString::from_vec(b"file\xff.wad".to_vec());
        assert_eq!(
            parse(&["wad".into(), "ls".into(), path.clone()]).unwrap(),
            Command::Wad {
                action: WadAction::Ls,
                path: path.into(),
                json: false
            }
        );
    }
}
