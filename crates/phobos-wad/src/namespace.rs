// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Ordered archive stacking and vanilla last-match lookup.

use crate::{LumpInfo, LumpName, Wad};

/// A global lump index in a namespace's insertion order.
///
/// Handles remain valid as more WADs are pushed. A handle carries no namespace
/// identity; using it with another namespace addresses that namespace's index.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LumpId(usize);

impl LumpId {
    /// Returns the zero-based global index.
    #[must_use]
    pub fn index(self) -> usize {
        self.0
    }
}

/// An owned archive stack with vanilla insertion order and last-wins lookup.
#[derive(Debug, Default)]
pub struct LumpNamespace {
    wads: Vec<Wad>,
    indices: Vec<(usize, usize)>,
}

impl LumpNamespace {
    /// Creates an empty lump namespace.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends every lump of an archive in directory order.
    pub fn push(&mut self, wad: Wad) {
        let wad_index = self.wads.len();
        self.indices
            .extend((0..wad.lumps().len()).map(|index| (wad_index, index)));
        self.wads.push(wad);
    }

    /// Returns the total lump count across all archives.
    #[must_use]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns whether the namespace contains no lumps.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Returns the last matching lump in global order.
    #[must_use]
    pub fn find(&self, name: LumpName) -> Option<LumpId> {
        self.indices
            .iter()
            .rposition(|&(wad, index)| self.wads[wad].lumps()[index].name() == name)
            .map(LumpId)
    }

    /// Returns the directory entry addressed by a global handle.
    #[must_use]
    pub fn info(&self, id: LumpId) -> Option<&LumpInfo> {
        let &(wad, index) = self.indices.get(id.0)?;
        self.wads.get(wad)?.lumps().get(index)
    }

    /// Returns the bytes addressed by a global handle.
    #[must_use]
    pub fn data(&self, id: LumpId) -> Option<&[u8]> {
        let &(wad, index) = self.indices.get(id.0)?;
        self.wads.get(wad)?.lump_data(index)
    }

    /// Returns lumps strictly between the last occurrences of two markers.
    ///
    /// Missing markers, equal markers, and reversed markers yield an empty list.
    #[must_use]
    pub fn between(&self, start: LumpName, end: LumpName) -> Vec<LumpId> {
        match (self.find(start), self.find(end)) {
            (Some(start), Some(end)) if start < end => {
                // A valid start index is below Vec's length, so its successor fits.
                (start.0.saturating_add(1)..end.0).map(LumpId).collect()
            }
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LumpId, LumpNamespace};
    use crate::{LumpName, test_support};

    #[test]
    fn stack_order_last_wins_and_stable_handles() {
        let name = LumpName::new("A").unwrap();
        let mut ns = LumpNamespace::new();
        assert!(ns.is_empty());
        assert_eq!(ns.find(name), None);
        ns.push(test_support::wad(&[]));
        ns.push(test_support::wad(&["A", "B", "A"]));
        let old = ns.find(name).unwrap();
        assert_eq!(old.index(), 2);
        ns.push(test_support::wad(&["C", "A"]));
        assert_eq!(ns.len(), 5);
        assert!(!ns.is_empty());
        assert_eq!(ns.find(name), Some(LumpId(4)));
        assert_eq!(ns.info(old).unwrap().name(), name);
        assert_eq!(ns.data(old), Some(b"".as_slice()));
        assert_eq!(ns.info(LumpId(5)), None);
        assert_eq!(ns.data(LumpId(usize::MAX)), None);
    }

    #[test]
    fn marker_ranges_use_last_markers_across_wads() {
        let start = LumpName::new("S_START").unwrap();
        let end = LumpName::new("S_END").unwrap();
        let mut ns = LumpNamespace::default();
        assert!(ns.between(start, end).is_empty());
        ns.push(test_support::wad(&["S_START", "A", "S_END", "S_START"]));
        assert!(ns.between(start, end).is_empty());
        ns.push(test_support::wad(&["B", "S_END", "C"]));
        assert_eq!(ns.between(start, end), vec![LumpId(4)]);
        assert!(ns.between(start, start).is_empty());
        ns.push(test_support::wad(&["S_START", "S_END"]));
        assert!(ns.between(start, end).is_empty());
        assert!(
            ns.between(LumpName::new("MISSING").unwrap(), end)
                .is_empty()
        );
        assert!(
            ns.between(start, LumpName::new("MISSING").unwrap())
                .is_empty()
        );
    }
}
