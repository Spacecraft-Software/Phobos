// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! WAD archive reading, IWAD identification and the lump namespace.
//!
//! Every byte of a WAD is untrusted input: parsing uses checked bounds and
//! returns typed errors, and never panics on malformed data.

mod error;
mod iwad;
mod name;
mod namespace;
mod wad;

#[doc(inline)]
pub use error::WadError;
#[doc(inline)]
pub use iwad::{
    GameMission, GameMode, IWAD_FILE_NAMES, IwadIdentity, find_iwad, identify, iwad_search_dirs,
};
#[doc(inline)]
pub use name::LumpName;
#[doc(inline)]
pub use namespace::{LumpId, LumpNamespace};
#[doc(inline)]
pub use wad::{LumpInfo, Wad, WadKind};

#[cfg(test)]
mod test_support {
    //! Hand-built archives shared by unit tests.

    pub(crate) fn bytes(names: &[&str]) -> Vec<u8> {
        let mut data = b"IWAD".to_vec();
        data.extend(i32::try_from(names.len()).unwrap().to_le_bytes());
        data.extend(12_i32.to_le_bytes());
        for name in names {
            data.extend(0_i32.to_le_bytes());
            data.extend(0_i32.to_le_bytes());
            let mut raw = [0; 8];
            raw[..name.len()].copy_from_slice(name.as_bytes());
            data.extend(raw);
        }
        data
    }

    pub(crate) fn wad(names: &[&str]) -> crate::Wad {
        crate::Wad::from_bytes(bytes(names)).unwrap()
    }
}
