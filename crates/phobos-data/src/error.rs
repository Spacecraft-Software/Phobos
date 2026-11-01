// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The error type every decoder in this crate returns.

use std::backtrace::Backtrace;
use std::fmt;

/// A lump could not be decoded.
///
/// Every decoder in this crate returns this error. It names the lump kind
/// that failed and what was wrong with it; the `is_*` methods let a caller
/// tell the failure classes apart without matching on internals.
#[derive(Debug)]
pub struct DataError {
    lump: &'static str,
    kind: ErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
enum ErrorKind {
    /// The lump ended before a field could be read.
    Truncated {
        offset: usize,
        needed: usize,
        len: usize,
    },
    /// A cross-reference names an element that does not exist.
    IndexOutOfRange {
        field: &'static str,
        index: usize,
        len: usize,
    },
    /// A value is outside what the format allows.
    Invalid { reason: String },
}

impl DataError {
    /// The lump ended before `needed` bytes could be read at `offset`.
    pub(crate) fn truncated(lump: &'static str, offset: usize, needed: usize, len: usize) -> Self {
        Self::new(
            lump,
            ErrorKind::Truncated {
                offset,
                needed,
                len,
            },
        )
    }

    /// The `field` cross-reference holds `index`, but only `len` elements exist.
    pub(crate) fn index_out_of_range(
        lump: &'static str,
        field: &'static str,
        index: usize,
        len: usize,
    ) -> Self {
        Self::new(lump, ErrorKind::IndexOutOfRange { field, index, len })
    }

    /// A value is outside what the format allows; `reason` says which.
    pub(crate) fn invalid(lump: &'static str, reason: impl Into<String>) -> Self {
        Self::new(
            lump,
            ErrorKind::Invalid {
                reason: reason.into(),
            },
        )
    }

    fn new(lump: &'static str, kind: ErrorKind) -> Self {
        Self {
            lump,
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// The kind of lump that failed to decode, such as `"LINEDEFS"`.
    #[must_use]
    pub fn lump(&self) -> &'static str {
        self.lump
    }

    /// Whether the lump was shorter than its format requires.
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        matches!(self.kind, ErrorKind::Truncated { .. })
    }

    /// Whether a cross-reference pointed past the end of what it references.
    #[must_use]
    pub fn is_index_out_of_range(&self) -> bool {
        matches!(self.kind, ErrorKind::IndexOutOfRange { .. })
    }

    /// Whether a value was outside what the format allows.
    #[must_use]
    pub fn is_invalid(&self) -> bool {
        matches!(self.kind, ErrorKind::Invalid { .. })
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Truncated {
                offset,
                needed,
                len,
            } => write!(
                f,
                "{} lump truncated: needed {needed} bytes at offset {offset}, lump is {len} bytes",
                self.lump
            )?,
            ErrorKind::IndexOutOfRange { field, index, len } => write!(
                f,
                "{} lump: {field} {index} is out of range (only {len} exist)",
                self.lump
            )?,
            ErrorKind::Invalid { reason } => write!(f, "{} lump is invalid: {reason}", self.lump)?,
        }
        if self.backtrace.status() == std::backtrace::BacktraceStatus::Captured {
            write!(f, "\n{}", self.backtrace)?;
        }
        Ok(())
    }
}

impl std::error::Error for DataError {}

// Errors cross threads (level loading may run on a worker); pin the bounds.
const _: () = {
    const fn assert_bounds<T: Send + Sync + 'static>() {}
    assert_bounds::<DataError>();
};
