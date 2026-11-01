// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Canonical errors for archive reading and identification.

use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum ErrorKind {
    Io(std::io::Error),
    BadMagic,
    Truncated(&'static str),
    LumpOutOfBounds(usize),
    NotAGame,
}

#[derive(Debug)]
struct ErrorData {
    kind: ErrorKind,
    path: Option<PathBuf>,
    backtrace: Backtrace,
}

/// An archive I/O, format, bounds, or game-identification error.
#[derive(Debug)]
pub struct WadError {
    inner: Box<ErrorData>,
}

impl WadError {
    fn new(kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(ErrorData {
                kind,
                path: None,
                backtrace: Backtrace::capture(),
            }),
        }
    }

    pub(crate) fn io(source: std::io::Error) -> Self {
        Self::new(ErrorKind::Io(source))
    }

    pub(crate) fn bad_magic() -> Self {
        Self::new(ErrorKind::BadMagic)
    }

    pub(crate) fn truncated(problem: &'static str) -> Self {
        Self::new(ErrorKind::Truncated(problem))
    }

    pub(crate) fn lump_out_of_bounds(index: usize) -> Self {
        Self::new(ErrorKind::LumpOutOfBounds(index))
    }

    pub(crate) fn not_a_game() -> Self {
        Self::new(ErrorKind::NotAGame)
    }

    pub(crate) fn with_path(mut self, path: &Path) -> Self {
        self.inner.path = Some(path.to_path_buf());
        self
    }

    /// Returns whether reading the archive failed at the I/O layer.
    #[must_use]
    pub fn is_io(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::Io(_))
    }

    /// Returns whether the archive signature is neither IWAD nor PWAD.
    #[must_use]
    pub fn is_bad_magic(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::BadMagic)
    }

    /// Returns whether the header or directory is invalid or does not fit.
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::Truncated(_))
    }

    /// Returns whether a lump has negative fields or bytes outside the archive.
    #[must_use]
    pub fn is_lump_out_of_bounds(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::LumpOutOfBounds(_))
    }

    /// Returns whether no supported game map marker was found.
    #[must_use]
    pub fn is_not_a_game(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::NotAGame)
    }

    /// Returns the path supplied to the archive reader, when available.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.inner.path.as_deref()
    }

    /// Returns the backtrace captured when this error was created.
    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }
}

impl fmt::Display for WadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            write!(f, "{}: ", path.display())?;
        }
        match &self.inner.kind {
            ErrorKind::Io(source) => write!(f, "could not read WAD: {source}")?,
            ErrorKind::BadMagic => f.write_str("bad WAD magic: expected IWAD or PWAD")?,
            ErrorKind::Truncated(problem) => write!(f, "invalid or truncated WAD {problem}")?,
            ErrorKind::LumpOutOfBounds(index) => {
                write!(f, "WAD lump {index} has invalid or out-of-bounds data")?;
            }
            ErrorKind::NotAGame => f.write_str("WAD is not a supported game: no map marker")?,
        }
        if self.backtrace().status() == BacktraceStatus::Captured {
            write!(f, "\n{}", self.backtrace())?;
        }
        Ok(())
    }
}

impl std::error::Error for WadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.inner.kind {
            ErrorKind::Io(source) => Some(source),
            _ => None,
        }
    }
}

// Keep these error bounds part of the compile-time contract.
const _: fn() = || {
    fn assert_error<T: std::error::Error + Send + Sync + 'static>() {}
    assert_error::<WadError>();
};

#[cfg(test)]
mod tests {
    use super::WadError;
    use std::error::Error;
    use std::path::Path;

    #[test]
    fn categories_and_diagnostics() {
        let errors = [
            WadError::io(std::io::Error::from(std::io::ErrorKind::NotFound)),
            WadError::bad_magic(),
            WadError::truncated("directory"),
            WadError::lump_out_of_bounds(7),
            WadError::not_a_game(),
        ];
        for (index, error) in errors.into_iter().enumerate() {
            assert_eq!(error.is_io(), index == 0);
            assert_eq!(error.is_bad_magic(), index == 1);
            assert_eq!(error.is_truncated(), index == 2);
            assert_eq!(error.is_lump_out_of_bounds(), index == 3);
            assert_eq!(error.is_not_a_game(), index == 4);
            assert_eq!(error.source().is_some(), index == 0);
            let error = error.with_path(Path::new("broken.wad"));
            assert!(error.to_string().starts_with("broken.wad: "));
            assert_eq!(error.path(), Some(Path::new("broken.wad")));
            assert!(!format!("{error:?}").is_empty());
        }
    }
}
