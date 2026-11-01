// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Checked parsing of the published WAD header and directory format.

use std::path::Path;

use crate::{LumpName, WadError};

// WAD integers and the signature occupy four bytes in the published format.
const WORD_SIZE: usize = 4;
// A WAD header contains a signature, lump count, and directory offset.
const HEADER_SIZE: usize = 3 * WORD_SIZE;
// Directory entries contain two integers followed by an eight-byte name.
const ENTRY_SIZE: usize = 16;
// The name follows the file position and size in each directory entry.
const NAME_OFFSET: usize = 2 * WORD_SIZE;

/// The signature distinguishing a base archive from a patch archive.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WadKind {
    /// A base game archive with the IWAD signature.
    Iwad,
    /// A patch archive with the PWAD signature.
    Pwad,
}

/// A validated WAD directory entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LumpInfo {
    name: LumpName,
    offset: u32,
    size: u32,
}

impl LumpInfo {
    /// Returns the normalized lump name.
    #[must_use]
    pub fn name(&self) -> LumpName {
        self.name
    }

    /// Returns the file position recorded in the directory.
    #[must_use]
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Returns the byte size recorded in the directory.
    #[must_use]
    pub fn size(&self) -> u32 {
        self.size
    }
}

/// An owned WAD archive with a fully validated directory.
#[derive(Debug)]
pub struct Wad {
    kind: WadKind,
    lumps: Vec<LumpInfo>,
    data: Vec<u8>,
}

impl Wad {
    /// Parses an owned archive without copying its lump bytes.
    ///
    /// Directory allocation is bounded by the input length. Parsing is serial:
    /// entries are small and their original order must be retained.
    ///
    /// # Errors
    /// Returns an error for invalid magic, a missing or invalid header or
    /// directory, negative lump fields, or nonempty lump data outside the file.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, WadError> {
        let header = data
            .get(..HEADER_SIZE)
            .ok_or_else(|| WadError::truncated("header"))?;
        let kind = match &header[..WORD_SIZE] {
            b"IWAD" => WadKind::Iwad,
            b"PWAD" => WadKind::Pwad,
            _ => return Err(WadError::bad_magic()),
        };
        let count = usize::try_from(read_integer(&header[WORD_SIZE..NAME_OFFSET])?)
            .map_err(|_invalid_count| WadError::truncated("lump count"))?;
        let offset = usize::try_from(read_integer(&header[NAME_OFFSET..])?)
            .map_err(|_invalid_offset| WadError::truncated("directory offset"))?;
        let end = count
            .checked_mul(ENTRY_SIZE)
            .and_then(|size| offset.checked_add(size))
            .ok_or_else(|| WadError::truncated("directory"))?;
        let directory = data
            .get(offset..end)
            .ok_or_else(|| WadError::truncated("directory"))?;

        // Validate the complete directory extent before trusting its count.
        let mut lumps = Vec::with_capacity(count);
        for (index, entry) in directory.chunks_exact(ENTRY_SIZE).enumerate() {
            let offset = u32::try_from(read_integer(&entry[..WORD_SIZE])?)
                .map_err(|_negative_offset| WadError::lump_out_of_bounds(index))?;
            let size = u32::try_from(read_integer(&entry[WORD_SIZE..NAME_OFFSET])?)
                .map_err(|_negative_size| WadError::lump_out_of_bounds(index))?;
            if size != 0 && lump_bytes(&data, offset, size).is_none() {
                return Err(WadError::lump_out_of_bounds(index));
            }
            let raw = entry[NAME_OFFSET..]
                .try_into()
                .map_err(|_invalid_name_width| WadError::truncated("directory entry"))?;
            lumps.push(LumpInfo {
                name: LumpName::from_raw(raw),
                offset,
                size,
            });
        }
        Ok(Self { kind, lumps, data })
    }

    /// Reads the whole file and parses its archive directory.
    ///
    /// # Errors
    /// Returns I/O or format errors with the supplied path attached.
    pub fn read(path: impl AsRef<Path>) -> Result<Self, WadError> {
        let path = path.as_ref();
        std::fs::read(path)
            .map_err(WadError::io)
            .and_then(Self::from_bytes)
            .map_err(|error| error.with_path(path))
    }

    /// Returns the archive signature kind.
    #[must_use]
    pub fn kind(&self) -> WadKind {
        self.kind
    }

    /// Returns all directory entries in their original order.
    #[must_use]
    pub fn lumps(&self) -> &[LumpInfo] {
        &self.lumps
    }

    /// Returns lump bytes, including an empty slice for marker lumps.
    #[must_use]
    pub fn lump_data(&self, index: usize) -> Option<&[u8]> {
        let info = self.lumps.get(index)?;
        if info.size == 0 {
            return Some(&[]);
        }
        lump_bytes(&self.data, info.offset, info.size)
    }

    /// Returns the last directory index with the given name.
    #[must_use]
    pub fn find_last(&self, name: LumpName) -> Option<usize> {
        self.lumps.iter().rposition(|info| info.name == name)
    }
}

fn read_integer(bytes: &[u8]) -> Result<i32, WadError> {
    let raw = bytes
        .try_into()
        .map_err(|_invalid_integer_width| WadError::truncated("integer"))?;
    Ok(i32::from_le_bytes(raw))
}

fn lump_bytes(data: &[u8], offset: u32, size: u32) -> Option<&[u8]> {
    let start = usize::try_from(offset).ok()?;
    let end = start.checked_add(usize::try_from(size).ok()?)?;
    data.get(start..end)
}

#[cfg(test)]
mod tests {
    use super::{Wad, WadKind};
    use crate::{LumpName, test_support};

    #[test]
    fn rejects_short_headers_and_bad_magic() {
        for len in 0..12 {
            assert!(Wad::from_bytes(vec![0; len]).unwrap_err().is_truncated());
        }
        let mut data = test_support::bytes(&[]);
        data[..4].copy_from_slice(b"NOPE");
        assert!(Wad::from_bytes(data).unwrap_err().is_bad_magic());
    }

    #[test]
    fn rejects_invalid_directory_extents() {
        for (count, offset) in [
            (-1_i32, 12_i32),
            (0, -1),
            (1, 12),
            (0, 13),
            (i32::MAX, 12),
            (i32::MAX, i32::MAX),
        ] {
            let mut data = b"IWAD".to_vec();
            data.extend(count.to_le_bytes());
            data.extend(offset.to_le_bytes());
            assert!(Wad::from_bytes(data).unwrap_err().is_truncated());
        }
        let mut data = test_support::bytes(&["TEST"]);
        data.pop();
        assert!(Wad::from_bytes(data).unwrap_err().is_truncated());
    }

    #[test]
    fn rejects_invalid_lump_fields_and_ranges() {
        for (offset, size) in [
            (-1_i32, 0_i32),
            (0, -1),
            (28, 1),
            (27, 2),
            (i32::MAX, i32::MAX),
        ] {
            let mut data = test_support::bytes(&["TEST"]);
            data[12..16].copy_from_slice(&offset.to_le_bytes());
            data[16..20].copy_from_slice(&size.to_le_bytes());
            assert!(Wad::from_bytes(data).unwrap_err().is_lump_out_of_bounds());
        }
    }

    #[test]
    fn accepts_markers_duplicates_and_arbitrary_directory_order() {
        let mut data = test_support::bytes(&["test", "MARKER", "TEST"]);
        data[..4].copy_from_slice(b"PWAD");
        data[12..16].copy_from_slice(&60_i32.to_le_bytes());
        data[16..20].copy_from_slice(&3_i32.to_le_bytes());
        data[28..32].copy_from_slice(&i32::MAX.to_le_bytes());
        data[44..48].copy_from_slice(&61_i32.to_le_bytes());
        data[48..52].copy_from_slice(&2_i32.to_le_bytes());
        data.extend(b"abc");
        let wad = Wad::from_bytes(data).unwrap();
        assert_eq!(wad.kind(), WadKind::Pwad);
        assert_eq!(wad.find_last(LumpName::new("test").unwrap()), Some(2));
        assert_eq!(wad.find_last(LumpName::new("MISSING").unwrap()), None);
        assert_eq!(wad.lump_data(0), Some(b"abc".as_slice()));
        assert_eq!(wad.lump_data(1), Some(b"".as_slice()));
        assert_eq!(wad.lump_data(2), Some(b"bc".as_slice()));
        assert_eq!(wad.lump_data(usize::MAX), None);
        assert_eq!(wad.lumps()[0].offset(), 60);
        assert_eq!(wad.lumps()[0].size(), 3);
        assert_eq!(test_support::wad(&[]).kind(), WadKind::Iwad);
    }

    #[test]
    fn reading_attaches_path_to_io_and_parse_errors() {
        // An existing directory fails std::fs::read on all supported hosts.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let error = Wad::read(dir).unwrap_err();
        assert!(error.is_io());
        assert_eq!(error.path(), Some(dir));
        let path = dir.join("Cargo.toml");
        let error = Wad::read(&path).unwrap_err();
        assert!(error.is_bad_magic());
        assert_eq!(error.path(), Some(path.as_path()));
        assert!(error.to_string().contains(&path.display().to_string()));
    }
}
