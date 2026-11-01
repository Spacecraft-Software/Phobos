// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `phobos-tools` executable: non-interactive inspection and CI tools.

mod cli;
mod json;
mod wad_cmd;

use std::backtrace::Backtrace;
use std::fmt;
use std::io::{self, Write};
use std::process::ExitCode;

/// The attribution block every Phobos binary prints (Standard §15.2).
const ATTRIBUTION: &str = "\
Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software  |  License: GPL-3.0-or-later
https://Phobos.SpacecraftSoftware.org/";

// POSIX command-line convention reserves status 2 for usage errors.
const USAGE_EXIT: u8 = 2;

#[derive(Debug)]
enum ErrorKind {
    Usage(String),
    Failure(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug)]
struct ErrorData {
    kind: ErrorKind,
    backtrace: Backtrace,
}

/// A command usage, archive, clock, or output error with its creation backtrace.
#[derive(Debug)]
pub struct Error {
    inner: Box<ErrorData>,
}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(ErrorData {
                kind,
                backtrace: Backtrace::capture(),
            }),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Usage(message.into()))
    }

    fn failure(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::new(ErrorKind::Failure(Box::new(source)))
    }

    /// Returns whether invalid arguments caused the error.
    #[must_use]
    pub fn is_usage(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::Usage(_))
    }

    /// Returns whether executing the command failed.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::Failure(_))
    }

    /// Returns the backtrace captured when this error was created.
    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner.kind {
            ErrorKind::Usage(message) => write!(f, "{message}; try --help"),
            ErrorKind::Failure(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.inner.kind {
            ErrorKind::Usage(_) => None,
            ErrorKind::Failure(source) => Some(source.as_ref()),
        }
    }
}

// Pin the error's thread-safety and lifetime contract at compile time.
const _: fn() = || {
    fn assert_error<T: std::error::Error + Send + Sync + 'static>() {}
    assert_error::<Error>();
};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (command, json) = cli::context(&args);
    let result = cli::parse(&args).and_then(|command| dispatch(command, &mut io::stdout().lock()));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Hostile paths and upstream backtraces must not split diagnostics into lines.
            let message = error.to_string().replace(['\n', '\r'], " ");
            let mut stderr = io::stderr().lock();
            let _ = if json {
                json::write_error(&mut stderr, command, &message)
            } else {
                writeln!(stderr, "error: {message}")
            };
            if error.is_usage() {
                ExitCode::from(USAGE_EXIT)
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

fn dispatch(command: cli::Command, out: &mut impl Write) -> Result<(), Error> {
    match command {
        cli::Command::Help { wad } => {
            let usage = if wad {
                "Usage: phobos-tools wad <ls|identify> [--json] <file>"
            } else {
                "Usage: phobos-tools [--help | --version]\n       phobos-tools wad <ls|identify> [--json] <file>"
            };
            writeln!(out, "{usage}\n\nNon-interactive tools for WADs, demos and maps.\n\nCommands:\n  wad ls <file>        List lumps in directory order\n  wad identify <file>  Identify the game and Freedoom markers\n\nOptions:\n  --json               Write JSON (anywhere after wad)\n  --help, -h, -help     Print help\n  --version, -version  Print version\n\n{ATTRIBUTION}").map_err(Error::failure)
        }
        cli::Command::Version => writeln!(
            out,
            "phobos-tools {}\n{ATTRIBUTION}",
            env!("CARGO_PKG_VERSION")
        )
        .map_err(Error::failure),
        cli::Command::Wad { action, path, json } => wad_cmd::run(action, &path, json, out),
    }
}
