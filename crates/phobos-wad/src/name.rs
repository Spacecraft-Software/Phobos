// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Vanilla normalization of fixed-width lump names.

use std::fmt::{self, Write};

// The WAD directory format reserves eight bytes for every lump name.
const NAME_SIZE: usize = 8;

/// An eight-byte, ASCII-uppercased, NUL-padded WAD lump name.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct LumpName([u8; NAME_SIZE]);

impl LumpName {
    /// Normalizes raw bytes, discarding the first NUL and everything after it.
    ///
    /// Non-ASCII bytes before the first NUL are preserved verbatim.
    #[must_use]
    pub fn from_raw(raw: [u8; NAME_SIZE]) -> Self {
        let mut name = [0; NAME_SIZE];
        for (dest, byte) in name.iter_mut().zip(raw).take_while(|(_, b)| *b != 0) {
            *dest = byte.to_ascii_uppercase();
        }
        Self(name)
    }

    /// Creates a name from one to eight non-NUL ASCII bytes.
    #[must_use]
    pub fn new(name: &str) -> Option<Self> {
        if name.is_empty() || name.len() > NAME_SIZE || !name.is_ascii() || name.contains('\0') {
            return None;
        }
        let mut raw = [0; NAME_SIZE];
        raw[..name.len()].copy_from_slice(name.as_bytes());
        Some(Self::from_raw(raw))
    }

    /// Returns the normalized name without trailing NUL padding.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        let len = self
            .0
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(NAME_SIZE);
        &self.0[..len]
    }
}

impl fmt::Display for LumpName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Names come from untrusted WADs: a raw control byte would reach the
        // terminal (an escape sequence, a newline splitting a row). Bytes
        // outside printable ASCII are shown as `\xNN`. A backslash stays
        // literal, because real sprite names use it (the Arch-vile's `VILE\1`).
        for &byte in self.as_bytes() {
            if byte.is_ascii_graphic() {
                f.write_char(char::from(byte))?;
            } else {
                write!(f, "\\x{byte:02X}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::LumpName;

    #[test]
    fn display_escapes_control_and_non_ascii_bytes() {
        let name = LumpName::from_raw(*b"\x1b[2J\nAB\xE9");
        assert_eq!(name.to_string(), "\\x1B[2J\\x0AAB\\xE9");
        assert_eq!(LumpName::from_raw(*b"VILE\\1\0\0").to_string(), "VILE\\1");
    }

    #[test]
    fn validates_and_normalizes_names() {
        for invalid in ["", "123456789", "a\0b", "é"] {
            assert_eq!(LumpName::new(invalid), None);
        }
        let name = LumpName::new("playpal").unwrap();
        assert_eq!(name.as_bytes(), b"PLAYPAL");
        assert_eq!(name.to_string(), "PLAYPAL");
        assert_eq!(LumpName::new("abcdefgh").unwrap().as_bytes(), b"ABCDEFGH");
        assert_eq!(
            LumpName::from_raw(*b"ab\0junk!"),
            LumpName::new("AB").unwrap()
        );
        assert_eq!(LumpName::from_raw([0; 8]).as_bytes(), b"");
        assert_eq!(LumpName::from_raw([0xff; 8]).as_bytes(), &[0xff; 8]);
    }
}
