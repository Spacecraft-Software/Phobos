// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Graphics lumps: PLAYPAL, COLORMAP, PNAMES, TEXTURE1 and TEXTURE2, patches
//! (including sprites), flats and ENDOOM.

use std::collections::HashMap;

use crate::{DataError, reader::Reader};

// Doom's PLAYPAL and COLORMAP formats use 256 palette indices and RGB triples.
const COLORS: usize = 256;
const RGB_BYTES: usize = 3;
const PALETTE_BYTES: usize = COLORS * RGB_BYTES;
// PNAMES and maptexture names occupy eight bytes in the Doom lump formats.
const NAME_BYTES: usize = 8;
// Texture and picture column directories contain four-byte offsets.
const OFFSET_BYTES: usize = 4;
// A Doom mappatch consists of five little-endian i16 fields.
const TEXTURE_PATCH_BYTES: usize = 10;
// The decoder's supported picture dimension limit is 4096 pixels.
const MAX_PATCH_DIMENSION: u16 = 4096;
// Doom picture columns end with the reserved top delta 0xFF.
const COLUMN_END: u8 = 0xFF;
// A picture post has top delta, length and two padding bytes, even with no pixels.
const MIN_POST_BYTES: usize = 4;
// Doom flats are 64 by 64 palette indices.
const FLAT_PIXELS: usize = 64 * 64;
// ENDOOM is an 80 by 25 text screen with character/attribute pairs.
const ENDOOM_CELLS: usize = 80 * 25;

/// RGB palettes from PLAYPAL.
#[derive(Debug)]
pub struct Palettes {
    /// Palettes in lump order, each containing 256 RGB triples.
    pub palettes: Vec<[[u8; 3]; 256]>,
}

impl Palettes {
    /// Decodes a nonempty sequence of complete PLAYPAL palettes.
    ///
    /// # Errors
    /// Returns an invalid-data error for an empty or partial palette sequence.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        if bytes.is_empty() || !bytes.len().is_multiple_of(PALETTE_BYTES) {
            return Err(DataError::invalid("PLAYPAL", "expected complete palettes"));
        }
        let mut reader = Reader::new("PLAYPAL", bytes);
        let mut palettes = Vec::with_capacity(bytes.len() / PALETTE_BYTES);
        while reader.remaining() > 0 {
            let mut palette = [[0; RGB_BYTES]; COLORS];
            for color in &mut palette {
                *color = reader.array()?;
            }
            palettes.push(palette);
        }
        Ok(Self { palettes })
    }
}

/// Palette index translations from COLORMAP.
#[derive(Debug)]
pub struct Colormaps {
    /// Maps in lump order.
    pub maps: Vec<[u8; 256]>,
}

impl Colormaps {
    /// Decodes a nonempty sequence of complete COLORMAP translations.
    ///
    /// # Errors
    /// Returns an invalid-data error for an empty or partial map sequence.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        if bytes.is_empty() || !bytes.len().is_multiple_of(COLORS) {
            return Err(DataError::invalid("COLORMAP", "expected complete maps"));
        }
        let mut reader = Reader::new("COLORMAP", bytes);
        let mut maps = Vec::with_capacity(bytes.len() / COLORS);
        while reader.remaining() > 0 {
            maps.push(reader.array()?);
        }
        Ok(Self { maps })
    }
}

/// Patch lump names referenced by texture definitions.
#[derive(Debug)]
pub struct PatchNames {
    /// Raw eight-byte names in PNAMES order.
    pub names: Vec<[u8; 8]>,
}

impl PatchNames {
    /// Decodes the count and names in PNAMES.
    ///
    /// # Errors
    /// Returns an error for a negative count or truncated names.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut reader = Reader::new("PNAMES", bytes);
        let count = read_count(&mut reader, "PNAMES")?;
        check_records(&reader, "PNAMES", bytes.len(), count, NAME_BYTES)?;
        let mut names = Vec::with_capacity(count);
        for _ in 0..count {
            names.push(reader.array()?);
        }
        Ok(Self { names })
    }
}

/// A patch placed within a texture.
#[derive(Debug)]
pub struct TexturePatch {
    /// Horizontal origin in texture pixels.
    pub origin_x: i16,
    /// Vertical origin in texture pixels.
    pub origin_y: i16,
    /// Index into PNAMES, to be resolved by the renderer.
    pub patch: u16,
}

/// A wall texture definition.
#[derive(Debug)]
pub struct TextureDef {
    /// Raw eight-byte texture name.
    pub name: [u8; 8],
    /// Whether the on-disk masked field is nonzero.
    pub masked: bool,
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
    /// Patches in drawing order.
    pub patches: Vec<TexturePatch>,
}

/// Texture definitions from TEXTURE1 or TEXTURE2.
#[derive(Debug)]
pub struct TextureDefs {
    /// Textures in directory order.
    pub textures: Vec<TextureDef>,
}

impl TextureDefs {
    /// Decodes a TEXTURE1 or TEXTURE2 lump without resolving PNAMES indices.
    ///
    /// # Errors
    /// Returns an error for negative fields, bad offsets or truncated records.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut directory = Reader::new("TEXTURE", bytes);
        let count = read_count(&mut directory, "TEXTURE")?;
        check_records(&directory, "TEXTURE", bytes.len(), count, OFFSET_BYTES)?;
        let mut textures = Vec::with_capacity(count);
        let mut patch_records = 0;
        for _ in 0..count {
            let offset = usize::try_from(directory.i32()?).map_err(|_conversion| {
                DataError::truncated("TEXTURE", bytes.len(), 1, bytes.len())
            })?;
            let mut reader = Reader::new("TEXTURE", bytes);
            reader.seek(offset)?;
            textures.push(read_texture(&mut reader, bytes.len(), &mut patch_records)?);
        }
        Ok(Self { textures })
    }
}

fn read_texture(
    reader: &mut Reader<'_>,
    len: usize,
    patch_records: &mut usize,
) -> Result<TextureDef, DataError> {
    let name = reader.array()?;
    let masked = reader.i32()? != 0;
    let width = read_nonnegative(reader, "TEXTURE", "negative width")?;
    let height = read_nonnegative(reader, "TEXTURE", "negative height")?;
    reader.i32()?; // Obsolete column directory.
    let count = usize::from(read_nonnegative(reader, "TEXTURE", "negative patch count")?);
    check_records(reader, "TEXTURE", len, count, TEXTURE_PATCH_BYTES)?;
    // Repeated texture offsets must not multiply decoding work or allocation.
    if count > len / TEXTURE_PATCH_BYTES - *patch_records {
        return Err(DataError::invalid("TEXTURE", "texture records overlap"));
    }
    *patch_records += count;
    let mut patches = Vec::with_capacity(count);
    for _ in 0..count {
        let origin_x = reader.i16()?;
        let origin_y = reader.i16()?;
        let patch = read_nonnegative(reader, "TEXTURE", "negative patch index")?;
        reader.i16()?; // Unused step direction.
        reader.i16()?; // Unused colormap.
        patches.push(TexturePatch {
            origin_x,
            origin_y,
            patch,
        });
    }
    Ok(TextureDef {
        name,
        masked,
        width,
        height,
        patches,
    })
}

/// A run of opaque palette indices in a picture column.
#[derive(Debug)]
pub struct PostSpan {
    /// Raw top delta, preserved for tall-patch interpretation by the renderer.
    pub top_delta: u8,
    /// Byte offset of the first pixel in the patch lump.
    pub start: u32,
    /// Number of palette indices in the post.
    pub len: u8,
}

/// A column-based Doom picture, including wall patches and sprites.
#[derive(Debug)]
pub struct Patch {
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
    /// Signed horizontal drawing offset.
    pub left_offset: i16,
    /// Signed vertical drawing offset.
    pub top_offset: i16,
    data: Vec<u8>,
    posts: Vec<PostSpan>,
    columns: Vec<(u32, u32)>,
}

impl Patch {
    /// Decodes a picture header, column directory and terminated posts.
    ///
    /// # Errors
    /// Returns an error for invalid dimensions, truncated columns or overlapping data.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut directory = Reader::new("PATCH", bytes);
        let width = read_dimension(&mut directory)?;
        let height = read_dimension(&mut directory)?;
        let left_offset = directory.i16()?;
        let top_offset = directory.i16()?;
        check_records(
            &directory,
            "PATCH",
            bytes.len(),
            usize::from(width),
            OFFSET_BYTES,
        )?;
        let mut columns = Vec::with_capacity(usize::from(width));
        let mut posts = Vec::new();
        let mut ranges = HashMap::new();
        for _ in 0..width {
            let offset = usize::try_from(directory.u32()?).map_err(|_conversion| {
                DataError::truncated("PATCH", bytes.len(), 1, bytes.len())
            })?;
            let range = if let Some(&range) = ranges.get(&offset) {
                range
            } else {
                let mut reader = Reader::new("PATCH", bytes);
                reader.seek(offset)?;
                let start = post_index(posts.len())?;
                read_column(&mut reader, &mut posts, bytes.len() / MIN_POST_BYTES)?;
                let range = (start, post_index(posts.len())?);
                ranges.insert(offset, range);
                range
            };
            columns.push(range);
        }
        Ok(Self {
            width,
            height,
            left_offset,
            top_offset,
            data: bytes.to_vec(),
            posts,
            columns,
        })
    }

    /// Returns a column's posts, or `None` when `x` is outside the picture.
    #[must_use]
    pub fn column(&self, x: usize) -> Option<&[PostSpan]> {
        let &(start, end) = self.columns.get(x)?;
        self.posts
            .get(usize::try_from(start).ok()?..usize::try_from(end).ok()?)
    }

    /// Returns the post's pixels, or an empty slice for a span outside this lump.
    #[must_use]
    pub fn post_pixels(&self, post: &PostSpan) -> &[u8] {
        let Ok(start) = usize::try_from(post.start) else {
            return &[];
        };
        let Some(end) = start.checked_add(usize::from(post.len)) else {
            return &[];
        };
        self.data.get(start..end).unwrap_or(&[])
    }
}

fn post_index(index: usize) -> Result<u32, DataError> {
    u32::try_from(index)
        .map_err(|_conversion| DataError::invalid("PATCH", "post offset exceeds u32"))
}

fn read_column(
    reader: &mut Reader<'_>,
    posts: &mut Vec<PostSpan>,
    budget: usize,
) -> Result<(), DataError> {
    loop {
        let top_delta = reader.u8()?;
        if top_delta == COLUMN_END {
            return Ok(());
        }
        if posts.len() >= budget {
            return Err(DataError::invalid("PATCH", "column data overlaps"));
        }
        let len = reader.u8()?;
        reader.u8()?; // Unused leading padding byte.
        let start = post_index(reader.position())?;
        reader.bytes(usize::from(len))?;
        reader.u8()?; // Unused trailing padding byte.
        posts.push(PostSpan {
            top_delta,
            start,
            len,
        });
    }
}

/// A 64 by 64 flat image.
#[derive(Debug)]
pub struct Flat {
    /// Palette indices in row order.
    pub pixels: Box<[u8; 4096]>,
}

impl Flat {
    /// Decodes the first 4096 bytes as a flat.
    ///
    /// # Errors
    /// Returns a truncation error when fewer than 4096 bytes are available.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut reader = Reader::new("FLAT", bytes);
        Ok(Self {
            pixels: Box::new(reader.array::<FLAT_PIXELS>()?),
        })
    }
}

/// An 80 by 25 ENDOOM text screen.
#[derive(Debug)]
pub struct Endoom {
    /// Character and attribute pairs in row order.
    pub cells: Box<[[u8; 2]; 2000]>,
}

impl Endoom {
    /// Decodes the first 4000 bytes as character and attribute pairs.
    ///
    /// # Errors
    /// Returns a truncation error when fewer than 4000 bytes are available.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut reader = Reader::new("ENDOOM", bytes);
        let mut cells = [[0; 2]; ENDOOM_CELLS];
        for cell in &mut cells {
            *cell = reader.array()?;
        }
        Ok(Self {
            cells: Box::new(cells),
        })
    }
}

fn read_count(reader: &mut Reader<'_>, lump: &'static str) -> Result<usize, DataError> {
    usize::try_from(reader.i32()?).map_err(|_conversion| DataError::invalid(lump, "negative count"))
}

fn read_nonnegative(
    reader: &mut Reader<'_>,
    lump: &'static str,
    reason: &'static str,
) -> Result<u16, DataError> {
    u16::try_from(reader.i16()?).map_err(|_conversion| DataError::invalid(lump, reason))
}

fn read_dimension(reader: &mut Reader<'_>) -> Result<u16, DataError> {
    let dimension = read_nonnegative(reader, "PATCH", "negative dimension")?;
    if !(1..=MAX_PATCH_DIMENSION).contains(&dimension) {
        return Err(DataError::invalid("PATCH", "dimension outside 1..=4096"));
    }
    Ok(dimension)
}

// Check before allocating; division avoids overflow from an untrusted count.
fn check_records(
    reader: &Reader<'_>,
    lump: &'static str,
    len: usize,
    count: usize,
    size: usize,
) -> Result<(), DataError> {
    if count > reader.remaining() / size {
        return Err(DataError::truncated(
            lump,
            reader.position(),
            count.saturating_mul(size),
            len,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texture() -> Vec<u8> {
        let mut bytes = 1_i32.to_le_bytes().to_vec();
        bytes.extend(8_i32.to_le_bytes());
        bytes.extend(*b"TEXTURE\0");
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(64_i16.to_le_bytes());
        bytes.extend(128_i16.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        for value in [-2_i16, 3, 7, 0, 0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes
    }

    fn patch() -> Vec<u8> {
        let mut bytes = Vec::new();
        for value in [1_i16, 2, -3, 4] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(12_u32.to_le_bytes());
        bytes.extend([0, 2, 99, 5, 6, 88, 255]);
        bytes
    }

    #[test]
    fn palette_and_colormap_sequences() {
        let mut bytes = vec![7; PALETTE_BYTES * 2];
        bytes[0..3].copy_from_slice(&[1, 2, 3]);
        let palettes = Palettes::decode(&bytes).unwrap();
        assert_eq!(palettes.palettes.len(), 2);
        assert_eq!(palettes.palettes[0][0], [1, 2, 3]);
        assert_eq!(palettes.palettes[1][255], [7; 3]);
        let maps = Colormaps::decode(&[9; COLORS * 2]).unwrap();
        assert_eq!(maps.maps.len(), 2);
        assert_eq!(maps.maps[1], [9; COLORS]);
        for bytes in [&[][..], &[0][..], &[0; PALETTE_BYTES + 1][..]] {
            assert!(Palettes::decode(bytes).unwrap_err().is_invalid());
        }
        for bytes in [&[][..], &[0][..], &[0; COLORS + 1][..]] {
            assert!(Colormaps::decode(bytes).unwrap_err().is_invalid());
        }
    }

    #[test]
    fn names_counts_and_truncation() {
        let mut bytes = 1_i32.to_le_bytes().to_vec();
        bytes.extend(*b"PATCH\0\0\0");
        assert_eq!(PatchNames::decode(&bytes).unwrap().names, [*b"PATCH\0\0\0"]);
        assert!(
            PatchNames::decode(&0_i32.to_le_bytes())
                .unwrap()
                .names
                .is_empty()
        );
        assert!(
            PatchNames::decode(&(-1_i32).to_le_bytes())
                .unwrap_err()
                .is_invalid()
        );
        assert!(
            PatchNames::decode(&i32::MAX.to_le_bytes())
                .unwrap_err()
                .is_truncated()
        );
        for end in 0..bytes.len() {
            assert!(
                PatchNames::decode(&bytes[..end])
                    .unwrap_err()
                    .is_truncated()
            );
        }
    }

    #[test]
    fn texture_fields_and_errors() {
        let bytes = texture();
        let decoded = TextureDefs::decode(&bytes).unwrap();
        let def = &decoded.textures[0];
        assert_eq!(def.name, *b"TEXTURE\0");
        assert!(def.masked);
        assert_eq!((def.width, def.height), (64, 128));
        assert_eq!(
            (
                def.patches[0].origin_x,
                def.patches[0].origin_y,
                def.patches[0].patch
            ),
            (-2, 3, 7)
        );
        for end in 0..bytes.len() {
            assert!(
                TextureDefs::decode(&bytes[..end])
                    .unwrap_err()
                    .is_truncated()
            );
        }
        assert!(
            TextureDefs::decode(&(-1_i32).to_le_bytes())
                .unwrap_err()
                .is_invalid()
        );
        assert!(
            TextureDefs::decode(&i32::MAX.to_le_bytes())
                .unwrap_err()
                .is_truncated()
        );
        for offset in [-1_i32, i32::MAX, i32::try_from(bytes.len()).unwrap()] {
            let mut bad = bytes.clone();
            bad[4..8].copy_from_slice(&offset.to_le_bytes());
            assert!(TextureDefs::decode(&bad).unwrap_err().is_truncated());
        }
        for position in [20, 22, 28, 34] {
            let mut bad = bytes.clone();
            bad[position..position + 2].copy_from_slice(&(-1_i16).to_le_bytes());
            assert!(TextureDefs::decode(&bad).unwrap_err().is_invalid());
        }
        assert!(
            TextureDefs::decode(&0_i32.to_le_bytes())
                .unwrap()
                .textures
                .is_empty()
        );
        let mut zero = bytes;
        for position in [20, 22, 28] {
            zero[position..position + 2].copy_from_slice(&0_i16.to_le_bytes());
        }
        let decoded = TextureDefs::decode(&zero).unwrap();
        assert_eq!(
            (decoded.textures[0].width, decoded.textures[0].height),
            (0, 0)
        );
        assert!(decoded.textures[0].patches.is_empty());
    }

    #[test]
    fn picture_posts_and_errors() {
        let bytes = patch();
        let decoded = Patch::decode(&bytes).unwrap();
        assert_eq!(
            (
                decoded.width,
                decoded.height,
                decoded.left_offset,
                decoded.top_offset
            ),
            (1, 2, -3, 4)
        );
        let column = decoded.column(0).unwrap();
        assert_eq!(column[0].top_delta, 0);
        assert_eq!((column[0].start, column[0].len), (15, 2));
        assert_eq!(decoded.post_pixels(&column[0]), [5, 6]);
        assert!(decoded.column(1).is_none());
        assert!(decoded.column(usize::MAX).is_none());
        assert!(
            decoded
                .post_pixels(&PostSpan {
                    top_delta: 0,
                    start: u32::MAX,
                    len: u8::MAX,
                })
                .is_empty()
        );
        for end in 0..bytes.len() {
            assert!(Patch::decode(&bytes[..end]).unwrap_err().is_truncated());
        }
        for position in [0, 2] {
            for dimension in [-1_i16, 0, 4097] {
                let mut bad = bytes.clone();
                bad[position..position + 2].copy_from_slice(&dimension.to_le_bytes());
                assert!(Patch::decode(&bad).unwrap_err().is_invalid());
            }
        }
        for offset in [u32::MAX, u32::try_from(bytes.len()).unwrap()] {
            let mut bad = bytes.clone();
            bad[8..12].copy_from_slice(&offset.to_le_bytes());
            assert!(Patch::decode(&bad).unwrap_err().is_truncated());
        }
        let mut empty = bytes[..12].to_vec();
        empty.push(COLUMN_END);
        assert!(Patch::decode(&empty).unwrap().column(0).unwrap().is_empty());
        let mut posts = bytes[..12].to_vec();
        posts.extend([254, 0, 0, 0, 1, 1, 0, 42, 0, COLUMN_END]);
        let decoded = Patch::decode(&posts).unwrap();
        let column = decoded.column(0).unwrap();
        assert_eq!(column.len(), 2);
        assert_eq!(column[0].top_delta, 254);
        assert!(decoded.post_pixels(&column[0]).is_empty());
        assert_eq!(decoded.post_pixels(&column[1]), [42]);
    }

    #[test]
    fn shared_texture_records_exceed_budget() {
        let record = texture()[8..].to_vec();
        let count = 8_i32;
        let offset = 4 + count * 4;
        let mut bytes = count.to_le_bytes().to_vec();
        for _ in 0..count {
            bytes.extend(offset.to_le_bytes());
        }
        bytes.extend(record);
        let error = TextureDefs::decode(&bytes).unwrap_err();
        assert!(error.is_invalid());
        assert!(error.to_string().contains("texture records overlap"));
    }

    #[test]
    fn shared_columns_are_parsed_once() {
        let width = 4096_i16;
        let offset = 8 + u32::try_from(width).unwrap() * 4;
        let mut bytes = Vec::new();
        for value in [width, 2, -3, 4] {
            bytes.extend(value.to_le_bytes());
        }
        for _ in 0..width {
            bytes.extend(offset.to_le_bytes());
        }
        bytes.extend([0, 2, 99, 5, 6, 88, COLUMN_END]);
        let decoded = Patch::decode(&bytes).unwrap();
        assert_eq!(decoded.posts.len(), 1);
        assert_eq!(decoded.data, bytes);
        for x in 0..usize::try_from(width).unwrap() {
            let column = decoded.column(x).unwrap();
            assert!(std::ptr::eq(column, decoded.column(0).unwrap()));
            assert_eq!(decoded.post_pixels(&column[0]), [5, 6]);
        }
    }

    #[test]
    fn overlapping_columns_exceed_post_budget() {
        let mut bytes = Vec::new();
        for value in [2_i16, 1, 0, 0] {
            bytes.extend(value.to_le_bytes());
        }
        // Each offset is valid alone, but their shared suffix is parsed twice.
        bytes.extend(16_u32.to_le_bytes());
        bytes.extend(20_u32.to_le_bytes());
        for _ in 0..6 {
            bytes.extend([0, 0, 0, 0]);
        }
        bytes.push(COLUMN_END);
        let budget = bytes.len() / MIN_POST_BYTES;
        assert_eq!(budget, 10);
        assert!(6 + 5 > budget);
        let error = Patch::decode(&bytes).unwrap_err();
        assert!(error.is_invalid());
        assert!(error.to_string().contains("column data overlaps"));
        for offset in [16_u32, 20] {
            let mut single = bytes.clone();
            single[12..16].copy_from_slice(&offset.to_le_bytes());
            single[8..12].copy_from_slice(&offset.to_le_bytes());
            Patch::decode(&single).unwrap();
        }
    }

    #[test]
    fn fixed_images_accept_trailing_bytes() {
        let mut flat = vec![9; FLAT_PIXELS + 1];
        flat[FLAT_PIXELS - 1] = 7;
        let decoded = Flat::decode(&flat).unwrap();
        assert_eq!(decoded.pixels[0], 9);
        assert_eq!(decoded.pixels[FLAT_PIXELS - 1], 7);
        assert!(
            Flat::decode(&flat[..FLAT_PIXELS - 1])
                .unwrap_err()
                .is_truncated()
        );
        let mut screen = vec![0; ENDOOM_CELLS * 2 + 1];
        screen[..2].copy_from_slice(&[65, 15]);
        screen[3998..4000].copy_from_slice(&[66, 7]);
        let decoded = Endoom::decode(&screen).unwrap();
        assert_eq!(decoded.cells[0], [65, 15]);
        assert_eq!(decoded.cells[1999], [66, 7]);
        assert!(Endoom::decode(&screen[..3999]).unwrap_err().is_truncated());
    }
}
