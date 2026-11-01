// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! A bounds-checked little-endian cursor over a lump's bytes.
//!
//! Every read checks the remaining length first and returns a truncation
//! error instead of panicking, so decoders built on it never index past the
//! end of untrusted input.

use crate::DataError;

/// A cursor over one lump. All multi-byte values are little-endian, as in
/// every Doom lump format.
#[derive(Debug, Clone)]
pub(crate) struct Reader<'a> {
    lump: &'static str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Starts reading `bytes` at offset 0. `lump` names the lump kind in errors.
    pub(crate) fn new(lump: &'static str, bytes: &'a [u8]) -> Self {
        Self {
            lump,
            bytes,
            pos: 0,
        }
    }

    /// The current offset from the start of the lump.
    pub(crate) fn position(&self) -> usize {
        self.pos
    }

    /// The number of bytes left after the current offset.
    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    /// Moves to `offset`, which may equal the lump length but not exceed it.
    pub(crate) fn seek(&mut self, offset: usize) -> Result<(), DataError> {
        if offset > self.bytes.len() {
            return Err(DataError::truncated(self.lump, offset, 0, self.bytes.len()));
        }
        self.pos = offset;
        Ok(())
    }

    /// Reads the next `n` bytes.
    pub(crate) fn bytes(&mut self, n: usize) -> Result<&'a [u8], DataError> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&end| end <= self.bytes.len())
            .ok_or_else(|| DataError::truncated(self.lump, self.pos, n, self.bytes.len()))?;
        let out = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    /// Reads exactly `N` bytes into an array.
    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], DataError> {
        let mut out = [0_u8; N];
        out.copy_from_slice(self.bytes(N)?);
        Ok(out)
    }

    /// Reads one byte.
    pub(crate) fn u8(&mut self) -> Result<u8, DataError> {
        Ok(self.array::<1>()?[0])
    }

    /// Reads an unsigned 16-bit value.
    pub(crate) fn u16(&mut self) -> Result<u16, DataError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    /// Reads a signed 16-bit value.
    pub(crate) fn i16(&mut self) -> Result<i16, DataError> {
        Ok(i16::from_le_bytes(self.array()?))
    }

    /// Reads an unsigned 32-bit value.
    pub(crate) fn u32(&mut self) -> Result<u32, DataError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    /// Reads a signed 32-bit value.
    pub(crate) fn i32(&mut self) -> Result<i32, DataError> {
        Ok(i32::from_le_bytes(self.array()?))
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn reads_little_endian_fields_in_order() {
        let bytes = [0x01, 0x02, 0xFF, 0xFF, 0x78, 0x56, 0x34, 0x12];
        let mut r = Reader::new("TEST", &bytes);
        assert_eq!(r.u16().unwrap(), 0x0201);
        assert_eq!(r.i16().unwrap(), -1);
        assert_eq!(r.u32().unwrap(), 0x1234_5678);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn short_read_is_a_truncation_error_and_does_not_advance() {
        let bytes = [0x01, 0x02, 0x03];
        let mut r = Reader::new("TEST", &bytes);
        let err = r.u32().unwrap_err();
        assert!(err.is_truncated());
        assert_eq!(err.lump(), "TEST");
        assert_eq!(r.position(), 0);
    }

    #[test]
    fn seek_past_end_fails() {
        let bytes = [0_u8; 4];
        let mut r = Reader::new("TEST", &bytes);
        r.seek(4).unwrap();
        assert!(r.seek(5).unwrap_err().is_truncated());
    }

    #[test]
    fn huge_length_does_not_overflow() {
        let bytes = [0_u8; 4];
        let mut r = Reader::new("TEST", &bytes);
        r.seek(2).unwrap();
        assert!(r.bytes(usize::MAX).unwrap_err().is_truncated());
    }
}
