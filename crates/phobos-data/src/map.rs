// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Map lumps: THINGS, LINEDEFS, SIDEDEFS, VERTEXES, SEGS, SSECTORS, NODES, SECTORS,
//! REJECT and BLOCKMAP.

use crate::{DataError, reader::Reader};

// Record sizes in the Doom map format (Unofficial Doom Specs).
const THING_SIZE: usize = 10;
const VERTEX_SIZE: usize = 4;
const LINEDEF_SIZE: usize = 14;
const SIDEDEF_SIZE: usize = 30;
const SEG_SIZE: usize = 12;
const SUBSECTOR_SIZE: usize = 4;
const NODE_SIZE: usize = 28;
const SECTOR_SIZE: usize = 26;
// Doom uses all bits set for an absent sidedef and a block list terminator.
const SENTINEL: u16 = 0xFFFF;
// NF_SUBSECTOR occupies the high bit of an on-disk node child.
const NF_SUBSECTOR: u16 = 0x8000;
// BLOCKMAP offsets count 16-bit words, not bytes.
const WORD_SIZE: usize = 2;

/// The ten map lumps in their directory order after a map marker.
pub const MAP_LUMP_NAMES: [&str; 10] = [
    "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS", "NODES", "SECTORS", "REJECT",
    "BLOCKMAP",
];

/// A placed actor or player start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thing {
    /// Horizontal map coordinate.
    pub x: i16,
    /// Vertical map coordinate.
    pub y: i16,
    /// Facing angle in degrees.
    pub angle: i16,
    /// Doom editor actor number.
    pub kind: i16,
    /// Placement flags.
    pub options: i16,
}

/// A map vertex in integer map units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vertex {
    /// Horizontal map coordinate.
    pub x: i16,
    /// Vertical map coordinate.
    pub y: i16,
}

/// A directed map line connecting two vertices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Linedef {
    /// Starting vertex index.
    pub v1: u16,
    /// Ending vertex index.
    pub v2: u16,
    /// Line flags.
    pub flags: i16,
    /// Line action number.
    pub special: i16,
    /// Sector action tag.
    pub tag: i16,
    /// Front sidedef index.
    pub front: u16,
    /// Optional back sidedef index.
    pub back: Option<u16>,
}

/// Texture placement and sector membership for one side of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sidedef {
    /// Horizontal texture offset.
    pub x_offset: i16,
    /// Vertical texture offset.
    pub y_offset: i16,
    /// Upper texture name, including any padding.
    pub top: [u8; 8],
    /// Lower texture name, including any padding.
    pub bottom: [u8; 8],
    /// Middle texture name, including any padding.
    pub middle: [u8; 8],
    /// Owning sector index.
    pub sector: u16,
}

/// A BSP segment of a linedef.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seg {
    /// Starting vertex index.
    pub v1: u16,
    /// Ending vertex index.
    pub v2: u16,
    /// Binary angle stored as its signed on-disk value.
    pub angle: i16,
    /// Parent linedef index.
    pub linedef: u16,
    /// Linedef side, zero for front or one for back.
    pub side: i16,
    /// Distance along the parent linedef.
    pub offset: i16,
}

/// A contiguous range of BSP segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subsector {
    /// Number of segments in the range.
    pub num_segs: u16,
    /// First segment index.
    pub first_seg: u16,
}

/// A decoded BSP child reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeChild {
    /// An internal node index.
    Node(u16),
    /// A subsector index with the on-disk flag removed.
    Subsector(u16),
}

/// A BSP partition with right and left children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    /// Partition origin's horizontal coordinate.
    pub x: i16,
    /// Partition origin's vertical coordinate.
    pub y: i16,
    /// Partition direction's horizontal component.
    pub dx: i16,
    /// Partition direction's vertical component.
    pub dy: i16,
    /// Right and left bounds, each ordered top, bottom, left, right.
    pub bbox: [[i16; 4]; 2],
    /// Right child followed by left child.
    pub children: [NodeChild; 2],
}

/// Floor, ceiling, lighting, and action data for a sector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sector {
    /// Floor height in map units.
    pub floor_height: i16,
    /// Ceiling height in map units.
    pub ceiling_height: i16,
    /// Floor flat name, including any padding.
    pub floor_pic: [u8; 8],
    /// Ceiling flat name, including any padding.
    pub ceiling_pic: [u8; 8],
    /// Light level.
    pub light: i16,
    /// Sector action number.
    pub special: i16,
    /// Sector action tag.
    pub tag: i16,
}

/// A collision grid retaining every entry before each list terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blockmap {
    /// Grid origin's horizontal coordinate.
    pub origin_x: i16,
    /// Grid origin's vertical coordinate.
    pub origin_y: i16,
    /// Number of grid columns.
    pub columns: u16,
    /// Number of grid rows.
    pub rows: u16,
    // The complete lump, with any trailing partial word ignored.
    words: Vec<u16>,
    // Row-major half-open word ranges, excluding list terminators.
    cells: Vec<(u32, u32)>,
}

impl Blockmap {
    /// Returns the number of decoded grid cells.
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Returns a cell's linedef list, including its leading zero entry.
    #[must_use]
    pub fn cell(&self, index: usize) -> Option<&[u16]> {
        let &(start, end) = self.cells.get(index)?;
        self.words
            .get(usize::try_from(start).ok()?..usize::try_from(end).ok()?)
    }

    /// Returns the list at a grid coordinate, or none outside the grid.
    #[must_use]
    pub fn cell_at(&self, column: u16, row: u16) -> Option<&[u16]> {
        if column >= self.columns || row >= self.rows {
            return None;
        }
        let index = usize::from(row)
            .checked_mul(usize::from(self.columns))?
            .checked_add(usize::from(column))?;
        self.cell(index)
    }

    fn validate_entries(&self, linedefs: usize) -> Result<(), DataError> {
        // At most u16::MAX squared cells overlap, so signed i64 counts cannot overflow.
        let mut changes = vec![0_i64; self.words.len()];
        for &(start, end) in &self.cells {
            let start = usize::try_from(start)
                .map_err(|_error| DataError::invalid("BLOCKMAP", "cell range overflow"))?;
            let end = usize::try_from(end)
                .map_err(|_error| DataError::invalid("BLOCKMAP", "cell range overflow"))?;
            changes[start] += 1;
            changes[end] -= 1;
        }
        let mut coverage = 0_i64;
        for (&entry, change) in self.words.iter().zip(changes) {
            coverage += change;
            if coverage > 0 {
                check_index("BLOCKMAP", "entry", entry, linedefs)?;
            }
        }
        Ok(())
    }
}

/// Raw sector visibility rejection bits, without size validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reject {
    /// Unmodified lump bytes.
    pub bytes: Vec<u8>,
}

/// Borrowed bytes of the ten lumps belonging to a map.
#[derive(Debug, Clone, Copy)]
pub struct MapLumps<'a> {
    /// THINGS bytes.
    pub things: &'a [u8],
    /// LINEDEFS bytes.
    pub linedefs: &'a [u8],
    /// SIDEDEFS bytes.
    pub sidedefs: &'a [u8],
    /// VERTEXES bytes.
    pub vertexes: &'a [u8],
    /// SEGS bytes.
    pub segs: &'a [u8],
    /// SSECTORS bytes.
    pub ssectors: &'a [u8],
    /// NODES bytes.
    pub nodes: &'a [u8],
    /// SECTORS bytes.
    pub sectors: &'a [u8],
    /// REJECT bytes.
    pub reject: &'a [u8],
    /// BLOCKMAP bytes.
    pub blockmap: &'a [u8],
}

/// Decoded map records with validated cross-references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapData {
    /// Placed things.
    pub things: Vec<Thing>,
    /// Directed map lines.
    pub linedefs: Vec<Linedef>,
    /// Line sides.
    pub sidedefs: Vec<Sidedef>,
    /// Map vertices.
    pub vertexes: Vec<Vertex>,
    /// BSP segments.
    pub segs: Vec<Seg>,
    /// BSP leaves.
    pub subsectors: Vec<Subsector>,
    /// BSP internal nodes.
    pub nodes: Vec<Node>,
    /// Sector properties.
    pub sectors: Vec<Sector>,
    /// Raw visibility rejection data.
    pub reject: Reject,
    /// Collision grid.
    pub blockmap: Blockmap,
}

impl MapData {
    /// Decodes all ten lumps and validates their cross-references.
    ///
    /// Trailing incomplete records are ignored, matching vanilla integer division.
    /// Decoding runs serially because these small record reads need no synchronization.
    ///
    /// # Errors
    ///
    /// Returns [`DataError`] for invalid references, missing front sides, invalid
    /// seg sides, truncated blockmap tables, or unterminated block lists.
    pub fn decode(lumps: &MapLumps<'_>) -> Result<Self, DataError> {
        let map = Self {
            things: records("THINGS", lumps.things, THING_SIZE, |r| {
                Ok(Thing {
                    x: r.i16()?,
                    y: r.i16()?,
                    angle: r.i16()?,
                    kind: r.i16()?,
                    options: r.i16()?,
                })
            })?,
            vertexes: records("VERTEXES", lumps.vertexes, VERTEX_SIZE, |r| {
                Ok(Vertex {
                    x: r.i16()?,
                    y: r.i16()?,
                })
            })?,
            linedefs: records("LINEDEFS", lumps.linedefs, LINEDEF_SIZE, read_linedef)?,
            sidedefs: records("SIDEDEFS", lumps.sidedefs, SIDEDEF_SIZE, |r| {
                Ok(Sidedef {
                    x_offset: r.i16()?,
                    y_offset: r.i16()?,
                    top: r.array()?,
                    bottom: r.array()?,
                    middle: r.array()?,
                    sector: r.u16()?,
                })
            })?,
            segs: records("SEGS", lumps.segs, SEG_SIZE, |r| {
                Ok(Seg {
                    v1: r.u16()?,
                    v2: r.u16()?,
                    angle: r.i16()?,
                    linedef: r.u16()?,
                    side: r.i16()?,
                    offset: r.i16()?,
                })
            })?,
            subsectors: records("SSECTORS", lumps.ssectors, SUBSECTOR_SIZE, |r| {
                Ok(Subsector {
                    num_segs: r.u16()?,
                    first_seg: r.u16()?,
                })
            })?,
            nodes: records("NODES", lumps.nodes, NODE_SIZE, read_node)?,
            sectors: records("SECTORS", lumps.sectors, SECTOR_SIZE, |r| {
                Ok(Sector {
                    floor_height: r.i16()?,
                    ceiling_height: r.i16()?,
                    floor_pic: r.array()?,
                    ceiling_pic: r.array()?,
                    light: r.i16()?,
                    special: r.i16()?,
                    tag: r.i16()?,
                })
            })?,
            reject: Reject {
                bytes: lumps.reject.to_vec(),
            },
            blockmap: read_blockmap(lumps.blockmap)?,
        };
        map.validate()?;
        Ok(map)
    }

    fn validate(&self) -> Result<(), DataError> {
        for line in &self.linedefs {
            check_index("LINEDEFS", "v1", line.v1, self.vertexes.len())?;
            check_index("LINEDEFS", "v2", line.v2, self.vertexes.len())?;
            check_index("LINEDEFS", "front", line.front, self.sidedefs.len())?;
            if let Some(back) = line.back {
                check_index("LINEDEFS", "back", back, self.sidedefs.len())?;
            }
        }
        for side in &self.sidedefs {
            check_index("SIDEDEFS", "sector", side.sector, self.sectors.len())?;
        }
        for seg in &self.segs {
            check_index("SEGS", "v1", seg.v1, self.vertexes.len())?;
            check_index("SEGS", "v2", seg.v2, self.vertexes.len())?;
            check_index("SEGS", "linedef", seg.linedef, self.linedefs.len())?;
            if !matches!(seg.side, 0 | 1) {
                return Err(DataError::invalid("SEGS", "side must be 0 or 1"));
            }
            if seg.side == 1 && self.linedefs[usize::from(seg.linedef)].back.is_none() {
                return Err(DataError::invalid("SEGS", "side 1 requires a back sidedef"));
            }
        }
        for subsector in &self.subsectors {
            // Widen before adding, so two on-disk u16 values cannot overflow.
            let end = usize::from(subsector.first_seg) + usize::from(subsector.num_segs);
            if end > self.segs.len() {
                return Err(DataError::index_out_of_range(
                    "SSECTORS",
                    "first_seg + num_segs",
                    end,
                    self.segs.len(),
                ));
            }
        }
        for node in &self.nodes {
            for child in node.children {
                match child {
                    NodeChild::Node(index) => {
                        check_index("NODES", "children node", index, self.nodes.len())?;
                    }
                    NodeChild::Subsector(index) => {
                        check_index("NODES", "children subsector", index, self.subsectors.len())?;
                    }
                }
            }
        }
        self.blockmap.validate_entries(self.linedefs.len())
    }
}

fn records<'a, T>(
    lump: &'static str,
    bytes: &'a [u8],
    size: usize,
    mut read: impl FnMut(&mut Reader<'a>) -> Result<T, DataError>,
) -> Result<Vec<T>, DataError> {
    let count = bytes.len() / size;
    let mut reader = Reader::new(lump, bytes);
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        result.push(read(&mut reader)?);
    }
    Ok(result)
}

fn read_linedef(r: &mut Reader<'_>) -> Result<Linedef, DataError> {
    let v1 = r.u16()?;
    let v2 = r.u16()?;
    let flags = r.i16()?;
    let special = r.i16()?;
    let tag = r.i16()?;
    let front = r.u16()?;
    let back = r.u16()?;
    if front == SENTINEL {
        return Err(DataError::invalid("LINEDEFS", "missing front sidedef"));
    }
    Ok(Linedef {
        v1,
        v2,
        flags,
        special,
        tag,
        front,
        back: (back != SENTINEL).then_some(back),
    })
}

fn read_node(r: &mut Reader<'_>) -> Result<Node, DataError> {
    let x = r.i16()?;
    let y = r.i16()?;
    let dx = r.i16()?;
    let dy = r.i16()?;
    let mut bbox = [[0; 4]; 2];
    for bounds in &mut bbox {
        for value in bounds {
            *value = r.i16()?;
        }
    }
    let children = [read_child(r.u16()?), read_child(r.u16()?)];
    Ok(Node {
        x,
        y,
        dx,
        dy,
        bbox,
        children,
    })
}

fn read_child(value: u16) -> NodeChild {
    if value & NF_SUBSECTOR == 0 {
        NodeChild::Node(value)
    } else {
        NodeChild::Subsector(value & !NF_SUBSECTOR)
    }
}

fn check_index(
    lump: &'static str,
    field: &'static str,
    index: u16,
    len: usize,
) -> Result<(), DataError> {
    if usize::from(index) >= len {
        return Err(DataError::index_out_of_range(
            lump,
            field,
            usize::from(index),
            len,
        ));
    }
    Ok(())
}

fn read_blockmap(bytes: &[u8]) -> Result<Blockmap, DataError> {
    let mut r = Reader::new("BLOCKMAP", bytes);
    let origin_x = r.i16()?;
    let origin_y = r.i16()?;
    let columns = r.u16()?;
    let rows = r.u16()?;
    let count = usize::from(columns)
        .checked_mul(usize::from(rows))
        .ok_or_else(|| DataError::invalid("BLOCKMAP", "grid dimensions overflow"))?;
    // Check the entire table before allocating anything from its dimensions.
    if count > r.remaining() / WORD_SIZE {
        return Err(DataError::truncated(
            "BLOCKMAP",
            r.position(),
            count.saturating_mul(WORD_SIZE),
            bytes.len(),
        ));
    }
    let words = records("BLOCKMAP", bytes, WORD_SIZE, Reader::u16)?;
    // One reverse pass finds every suffix's first terminator, even for shared lists.
    // The word count represents "none" because it is outside the word vector.
    let mut next_ends = vec![words.len(); words.len()];
    let mut end = words.len();
    for (index, &word) in words.iter().enumerate().rev() {
        if word == SENTINEL {
            end = index;
        }
        next_ends[index] = end;
    }
    let mut cells = Vec::with_capacity(count);
    for _ in 0..count {
        let start = r.u16()?;
        let Some(&end) = next_ends.get(usize::from(start)) else {
            return Err(DataError::invalid(
                "BLOCKMAP",
                "cell offset is outside the lump",
            ));
        };
        if end == words.len() {
            return Err(DataError::invalid(
                "BLOCKMAP",
                "cell list has no terminator",
            ));
        }
        let end = u32::try_from(end)
            .map_err(|_error| DataError::invalid("BLOCKMAP", "cell range overflow"))?;
        cells.push((u32::from(start), end));
    }
    Ok(Blockmap {
        origin_x,
        origin_y,
        columns,
        rows,
        words,
        cells,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(values: &[u16]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn fixture() -> [Vec<u8>; 10] {
        [
            words(&[1, 2, 90, 3001, 7]),
            words(&[0, 1, 2, 3, 4, 0, SENTINEL]),
            vec![0; SIDEDEF_SIZE],
            words(&[0, 1, 2, 3]),
            words(&[0, 1, 5, 0, 0, 6]),
            words(&[1, 0]),
            words(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, NF_SUBSECTOR, 0]),
            vec![0; SECTOR_SIZE],
            vec![0xAA],
            words(&[0, 0, 1, 1, 5, 0, SENTINEL]),
        ]
    }

    fn decode(data: &[Vec<u8>; 10]) -> Result<MapData, DataError> {
        MapData::decode(&MapLumps {
            things: &data[0],
            linedefs: &data[1],
            sidedefs: &data[2],
            vertexes: &data[3],
            segs: &data[4],
            ssectors: &data[5],
            nodes: &data[6],
            sectors: &data[7],
            reject: &data[8],
            blockmap: &data[9],
        })
    }

    fn set_word(bytes: &mut [u8], index: usize, value: u16) {
        bytes[index * WORD_SIZE..(index + 1) * WORD_SIZE].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn decodes_fields_and_ignores_partial_records() {
        let mut data = fixture();
        let map = decode(&data).unwrap();
        assert_eq!(
            map.things[0],
            Thing {
                x: 1,
                y: 2,
                angle: 90,
                kind: 3001,
                options: 7
            }
        );
        assert_eq!(map.vertexes, [Vertex { x: 0, y: 1 }, Vertex { x: 2, y: 3 }]);
        assert_eq!(
            map.linedefs[0],
            Linedef {
                v1: 0,
                v2: 1,
                flags: 2,
                special: 3,
                tag: 4,
                front: 0,
                back: None
            }
        );
        assert_eq!(
            map.segs[0],
            Seg {
                v1: 0,
                v2: 1,
                angle: 5,
                linedef: 0,
                side: 0,
                offset: 6
            }
        );
        assert_eq!(
            map.subsectors[0],
            Subsector {
                num_segs: 1,
                first_seg: 0
            }
        );
        assert_eq!(
            map.nodes[0],
            Node {
                x: 1,
                y: 2,
                dx: 3,
                dy: 4,
                bbox: [[5, 6, 7, 8], [9, 10, 11, 12]],
                children: [NodeChild::Subsector(0), NodeChild::Node(0)]
            }
        );
        assert_eq!(map.blockmap.cell_count(), 1);
        assert_eq!(map.blockmap.cell(0), Some([0].as_slice()));
        assert_eq!(map.reject.bytes, [0xAA]);
        for bytes in &mut data[..8] {
            bytes.push(0xFF);
        }
        assert_eq!(decode(&data).unwrap(), map);
    }

    #[test]
    fn preserves_signed_values_and_texture_names() {
        let mut data = fixture();
        data[0] = words(&[0xFFFF; 5]);
        data[2] = words(&[0xFFFF, 0x8000]);
        data[2].extend_from_slice(b"TOP12345BOTTOM12MIDDLE12");
        data[2].extend_from_slice(&0_u16.to_le_bytes());
        data[7] = words(&[0x8000, 0x7FFF]);
        data[7].extend_from_slice(b"FLOOR123CEIL1234");
        data[7].extend_from_slice(&words(&[0xFFFF, 2, 3]));
        let map = decode(&data).unwrap();
        assert_eq!(
            map.things[0],
            Thing {
                x: -1,
                y: -1,
                angle: -1,
                kind: -1,
                options: -1
            }
        );
        assert_eq!(
            map.sidedefs[0],
            Sidedef {
                x_offset: -1,
                y_offset: i16::MIN,
                top: *b"TOP12345",
                bottom: *b"BOTTOM12",
                middle: *b"MIDDLE12",
                sector: 0
            }
        );
        assert_eq!(
            map.sectors[0],
            Sector {
                floor_height: i16::MIN,
                ceiling_height: i16::MAX,
                floor_pic: *b"FLOOR123",
                ceiling_pic: *b"CEIL1234",
                light: -1,
                special: 2,
                tag: 3
            }
        );
    }

    #[test]
    fn rejects_each_out_of_range_reference() {
        for (lump, word, value, field) in [
            (1, 0, 2, "v1"),
            (1, 1, 2, "v2"),
            (1, 5, 1, "front"),
            (1, 6, 1, "back"),
            (2, 14, 1, "sector"),
            (4, 0, 2, "v1"),
            (4, 1, 2, "v2"),
            (4, 3, 1, "linedef"),
            (5, 0, 2, "first_seg + num_segs"),
            (5, 1, u16::MAX, "first_seg + num_segs"),
            (6, 12, 1, "children node"),
            (6, 13, NF_SUBSECTOR | 1, "children subsector"),
            (9, 5, 1, "entry"),
        ] {
            let mut data = fixture();
            set_word(&mut data[lump], word, value);
            let error = decode(&data).unwrap_err();
            assert!(error.is_index_out_of_range(), "{error}");
            assert_eq!(error.lump(), MAP_LUMP_NAMES[lump]);
            assert!(error.to_string().contains(field), "{error}");
        }
    }

    #[test]
    fn rejects_missing_front_and_invalid_seg_sides() {
        for (lump, word, value) in [(1, 5, SENTINEL), (4, 4, 2), (4, 4, SENTINEL), (4, 4, 1)] {
            let mut data = fixture();
            set_word(&mut data[lump], word, value);
            let error = decode(&data).unwrap_err();
            assert!(error.is_invalid(), "{error}");
            assert_eq!(error.lump(), MAP_LUMP_NAMES[lump]);
        }
        let mut data = fixture();
        set_word(&mut data[1], 6, 0);
        set_word(&mut data[4], 4, 1);
        assert_eq!(decode(&data).unwrap().linedefs[0].back, Some(0));
    }

    #[test]
    fn checks_blockmap_header_and_table_before_allocation() {
        for length in 0..8 {
            let error = read_blockmap(&[0; 8][..length]).unwrap_err();
            assert!(error.is_truncated());
        }
        for bytes in [words(&[0, 0, 1, 1]), words(&[0, 0, u16::MAX, u16::MAX])] {
            assert!(read_blockmap(&bytes).unwrap_err().is_truncated());
        }
    }

    #[test]
    fn rejects_outside_offsets_and_unterminated_lists() {
        for offset in [7, SENTINEL] {
            let bytes = words(&[0, 0, 1, 1, offset, 0, SENTINEL]);
            assert!(read_blockmap(&bytes).unwrap_err().is_invalid());
        }
        for tail in [vec![], vec![0], vec![0, 0], vec![0, 0, 0]] {
            let mut bytes = words(&[0, 0, 1, 1, 5]);
            bytes.extend(tail);
            assert!(read_blockmap(&bytes).unwrap_err().is_invalid());
        }
    }

    #[test]
    fn supports_shared_lists_and_preserves_zero_entries() {
        let bytes = words(&[0xFFFF, 0x8000, 2, 1, 6, 6, 0, 2, 0, SENTINEL]);
        let map = read_blockmap(&bytes).unwrap();
        assert_eq!(map.origin_x, -1);
        assert_eq!(map.origin_y, i16::MIN);
        assert_eq!(map.cell_count(), 2);
        assert_eq!(map.cell(0), Some([0, 2, 0].as_slice()));
        assert_eq!(map.cell(1), map.cell(0));
    }

    fn shared_blockmap() -> Vec<u8> {
        let count = 32_000_u16;
        let start = 4 + count;
        let mut values = vec![0, 0, count, 1];
        values.extend(std::iter::repeat_n(start, usize::from(count)));
        values.extend(std::iter::repeat_n(0, usize::from(count)));
        values.push(SENTINEL);
        words(&values)
    }

    #[test]
    fn many_cells_share_one_long_list_without_copying_words() {
        let mut data = fixture();
        data[9] = shared_blockmap();
        let map = decode(&data).unwrap().blockmap;
        assert_eq!(map.words.len(), data[9].len() / WORD_SIZE);
        assert_eq!(map.cell_count(), 32_000);
        let first = map.cell(0).unwrap();
        assert_eq!(first.len(), 32_000);
        assert!(first.iter().all(|&entry| entry == 0));
        for index in 0..map.cell_count() {
            assert_eq!(map.cells[index], map.cells[0]);
            assert_eq!(map.cell(index).unwrap().as_ptr(), first.as_ptr());
        }
    }

    #[test]
    fn rejects_out_of_range_entry_in_a_shared_long_list() {
        let mut data = fixture();
        data[9] = shared_blockmap();
        let last_entry = data[9].len() / WORD_SIZE - 2;
        set_word(&mut data[9], last_entry, 1);
        let error = decode(&data).unwrap_err();
        assert!(error.is_index_out_of_range());
        assert_eq!(error.lump(), "BLOCKMAP");
        assert!(error.to_string().contains("entry"));
    }

    #[test]
    fn cell_access_checks_coordinates_and_row_major_order() {
        let bytes = words(&[0, 0, 2, 2, 8, 9, 10, 11, 0, 0, 0, SENTINEL]);
        let map = read_blockmap(&bytes).unwrap();
        for (column, row, index) in [(0, 0, 0), (1, 0, 1), (0, 1, 2), (1, 1, 3)] {
            assert_eq!(map.cell_at(column, row), map.cell(index));
            assert_eq!(map.cell(index).unwrap().len(), 3 - index);
        }
        assert_eq!(map.cell(4), None);
        assert_eq!(map.cell(usize::MAX), None);
        assert_eq!(map.cell_at(2, 0), None);
        assert_eq!(map.cell_at(0, 2), None);
        assert_eq!(map.cell_at(u16::MAX, u16::MAX), None);
        assert_eq!(map.cell(3), Some([].as_slice()));
    }

    #[test]
    fn validates_only_covered_words_and_ignores_odd_trailing_byte() {
        let mut data = fixture();
        // The unreferenced word 123 and the terminator are not linedef entries.
        data[9] = words(&[0, 0, 3, 1, 8, 9, 11, 123, 0, 0, SENTINEL, SENTINEL]);
        let map = decode(&data).unwrap();
        assert_eq!(map.blockmap.cell(0), Some([0, 0].as_slice()));
        assert_eq!(map.blockmap.cell(1), Some([0].as_slice()));
        assert_eq!(map.blockmap.cell(2), Some([].as_slice()));
        data[9].push(0xFF);
        assert_eq!(decode(&data).unwrap(), map);
        // A terminator before the list cannot terminate a later list.
        data[9] = words(&[0, 0, 1, 1, 6, SENTINEL, 0]);
        assert!(decode(&data).unwrap_err().is_invalid());
    }

    #[test]
    fn accepts_empty_records_reject_and_grid() {
        let mut data: [Vec<u8>; 10] = std::array::from_fn(|_| Vec::new());
        data[9] = words(&[0; 4]);
        let map = decode(&data).unwrap();
        assert!(map.reject.bytes.is_empty());
        assert!(map.blockmap.cells.is_empty());
        let mut data = fixture();
        data[8].clear();
        decode(&data).unwrap();
        set_word(&mut data[5], 0, 0);
        set_word(&mut data[5], 1, 1);
        decode(&data).unwrap();
    }
}
