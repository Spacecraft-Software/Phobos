// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! IWAD identification and environment-injected archive discovery.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{LumpName, Wad, WadError};

/// The supported game's episode or commercial release mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GameMode {
    /// The first episode of Doom.
    Shareware,
    /// The three-episode registered Doom release.
    Registered,
    /// The four-episode Ultimate Doom release.
    Retail,
    /// A MAP01-based Doom II or Final Doom release.
    Commercial,
}

/// The game or mission pack associated with an archive.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GameMission {
    /// Doom or The Ultimate Doom.
    Doom,
    /// Doom II or another commercial-mode archive.
    Doom2,
    /// Final Doom: TNT Evilution.
    PackTnt,
    /// Final Doom: The Plutonia Experiment.
    PackPlut,
}

/// A game identity inferred from map markers and an optional filename.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IwadIdentity {
    mode: GameMode,
    mission: GameMission,
    freedoom: bool,
}

impl IwadIdentity {
    /// Returns the release mode selected by the archive's map markers.
    #[must_use]
    pub fn mode(&self) -> GameMode {
        self.mode
    }

    /// Returns the mission selected by mode and filename.
    #[must_use]
    pub fn mission(&self) -> GameMission {
        self.mission
    }

    /// Returns whether a FREEDOOM or FREEDM marker exists.
    #[must_use]
    pub fn is_freedoom(&self) -> bool {
        self.freedoom
    }
}

/// Identifies a game's mode, mission, and Freedoom marker presence.
///
/// MAP01 takes precedence over E4M1, E3M1, and E1M1, in that order.
/// The filename's last path component only affects commercial missions.
///
/// # Errors
/// Returns an error with [`WadError::is_not_a_game`] when no game marker exists.
pub fn identify(wad: &Wad, file_name: Option<&str>) -> Result<IwadIdentity, WadError> {
    // These map marker names distinguish the published Doom game layouts.
    const MODES: [([u8; 8], GameMode); 4] = [
        (*b"MAP01\0\0\0", GameMode::Commercial),
        (*b"E4M1\0\0\0\0", GameMode::Retail),
        (*b"E3M1\0\0\0\0", GameMode::Registered),
        (*b"E1M1\0\0\0\0", GameMode::Shareware),
    ];
    // Freedoom and FreeDM publish these identification lumps.
    const FREEDOOM_MARKERS: [[u8; 8]; 2] = [*b"FREEDOOM", *b"FREEDM\0\0"];
    let mode = MODES
        .into_iter()
        .find(|&(name, _)| wad.find_last(LumpName::from_raw(name)).is_some())
        .map(|(_, mode)| mode)
        .ok_or_else(WadError::not_a_game)?;
    let basename = file_name
        .and_then(|name| Path::new(name).file_name())
        .and_then(|name| name.to_str());
    let mission = if mode == GameMode::Commercial {
        match basename {
            Some(name) if name.eq_ignore_ascii_case("tnt.wad") => GameMission::PackTnt,
            Some(name) if name.eq_ignore_ascii_case("plutonia.wad") => GameMission::PackPlut,
            _ => GameMission::Doom2,
        }
    } else {
        GameMission::Doom
    };
    let freedoom = FREEDOOM_MARKERS
        .into_iter()
        .any(|name| wad.find_last(LumpName::from_raw(name)).is_some());
    Ok(IwadIdentity {
        mode,
        mission,
        freedoom,
    })
}

/// IWAD filenames in the engine's preferred discovery order.
pub const IWAD_FILE_NAMES: &[&str] = &[
    "doom2.wad",
    "plutonia.wad",
    "tnt.wad",
    "doom.wad",
    "doom1.wad",
    "freedoom2.wad",
    "freedoom1.wad",
];

/// Computes ordered IWAD search directories from an injected environment.
///
/// Empty variables are treated as unset. The order is DOOMWADDIR,
/// DOOMWADPATH, user data, system data, then the current directory.
#[must_use]
pub fn iwad_search_dirs(env: impl Fn(&str) -> Option<OsString>) -> Vec<PathBuf> {
    // The XDG Base Directory specification defines this default data path.
    const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";
    let nonempty = |key| env(key).filter(|value| !value.is_empty());
    let mut dirs = Vec::new();
    if let Some(dir) = nonempty("DOOMWADDIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let Some(paths) = nonempty("DOOMWADPATH") {
        dirs.extend(std::env::split_paths(&paths));
    }
    if let Some(home) = nonempty("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(home).join("doom"));
    } else if let Some(home) = nonempty("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/doom"));
    }
    let system = nonempty("XDG_DATA_DIRS").unwrap_or_else(|| OsString::from(DEFAULT_DATA_DIRS));
    dirs.extend(std::env::split_paths(&system).map(|dir| dir.join("doom")));
    dirs.push(PathBuf::from("."));
    dirs
}

/// Finds the first existing explicit path or ordered IWAD filename candidate.
///
/// Each directory is searched in [`IWAD_FILE_NAMES`] order, trying lowercase
/// then uppercase for each name. A missing explicit path permits fallback.
#[must_use]
pub fn find_iwad(
    explicit: Option<&Path>,
    dirs: &[PathBuf],
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if let Some(path) = explicit.filter(|path| exists(path)) {
        return Some(path.to_path_buf());
    }
    for dir in dirs {
        for name in IWAD_FILE_NAMES {
            let lower = dir.join(name);
            if exists(&lower) {
                return Some(lower);
            }
            let upper = dir.join(name.to_ascii_uppercase());
            if exists(&upper) {
                return Some(upper);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{GameMission, GameMode, IWAD_FILE_NAMES, find_iwad, identify, iwad_search_dirs};
    use crate::test_support;
    use std::cell::RefCell;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    #[test]
    fn modes_follow_map_precedence() {
        let cases: &[(&[&str], GameMode)] = &[
            (&["E1M1"], GameMode::Shareware),
            (&["E1M1", "E3M1"], GameMode::Registered),
            (&["E1M1", "E3M1", "E4M1"], GameMode::Retail),
            (&["E1M1", "E3M1", "E4M1", "MAP01"], GameMode::Commercial),
        ];
        for &(names, mode) in cases {
            let id = identify(&test_support::wad(names), Some("tnt.wad")).unwrap();
            assert_eq!(id.mode(), mode);
            assert_eq!(
                id.mission(),
                if mode == GameMode::Commercial {
                    GameMission::PackTnt
                } else {
                    GameMission::Doom
                }
            );
            assert!(!id.is_freedoom());
        }
        assert!(
            identify(&test_support::wad(&["PLAYPAL"]), None)
                .unwrap_err()
                .is_not_a_game()
        );
    }

    #[test]
    fn missions_use_case_insensitive_basename_and_freedoom_markers() {
        let wad = test_support::wad(&["MAP01", "FREEDOOM"]);
        for (name, mission) in [
            (None, GameMission::Doom2),
            (Some("doom2.wad"), GameMission::Doom2),
            (Some("data/TnT.WaD"), GameMission::PackTnt),
            (Some("data/PLUTONIA.WAD"), GameMission::PackPlut),
            (Some("tnt.wad/other.wad"), GameMission::Doom2),
        ] {
            let id = identify(&wad, name).unwrap();
            assert_eq!(id.mission(), mission);
            assert!(id.is_freedoom());
        }
        assert!(
            identify(&test_support::wad(&["E1M1", "FREEDM"]), None)
                .unwrap()
                .is_freedoom()
        );
    }

    #[test]
    fn search_directories_have_exact_precedence() {
        let wad_paths = std::env::join_paths(["first", "second"]).unwrap();
        let data_paths = std::env::join_paths(["system1", "system2"]).unwrap();
        let dirs = iwad_search_dirs(|key| match key {
            "DOOMWADDIR" => Some("override".into()),
            "DOOMWADPATH" => Some(wad_paths.clone()),
            "XDG_DATA_HOME" => Some("user-data".into()),
            "HOME" => Some("ignored-home".into()),
            "XDG_DATA_DIRS" => Some(data_paths.clone()),
            _ => None,
        });
        assert_eq!(
            dirs,
            [
                "override",
                "first",
                "second",
                "user-data/doom",
                "system1/doom",
                "system2/doom",
                "."
            ]
            .map(PathBuf::from)
        );
    }

    #[test]
    fn empty_environment_values_use_defaults_and_home_fallback() {
        let defaults = iwad_search_dirs(|_| None);
        let mut expected: Vec<_> = std::env::split_paths("/usr/local/share:/usr/share")
            .map(|path| path.join("doom"))
            .collect();
        expected.push(PathBuf::from("."));
        assert_eq!(defaults, expected);
        assert_eq!(iwad_search_dirs(|_| Some(OsString::new())), expected);
        expected.insert(0, PathBuf::from("home/.local/share/doom"));
        assert_eq!(
            iwad_search_dirs(|key| Some(if key == "HOME" {
                "home".into()
            } else {
                OsString::new()
            })),
            expected
        );
    }

    #[test]
    fn discovery_checks_explicit_then_every_candidate_in_order() {
        let dirs = [PathBuf::from("first"), PathBuf::from("second")];
        let explicit = Path::new("elsewhere/custom.wad");
        let visited = RefCell::new(Vec::new());
        let found = find_iwad(Some(explicit), &dirs, |path| {
            visited.borrow_mut().push(path.to_path_buf());
            true
        });
        assert_eq!(found, Some(explicit.to_path_buf()));
        assert_eq!(*visited.borrow(), vec![explicit.to_path_buf()]);
        visited.borrow_mut().clear();
        assert_eq!(
            find_iwad(Some(explicit), &dirs, |path| {
                visited.borrow_mut().push(path.to_path_buf());
                false
            }),
            None
        );
        let mut expected = vec![explicit.to_path_buf()];
        for dir in &dirs {
            for name in IWAD_FILE_NAMES {
                expected.push(dir.join(name));
                expected.push(dir.join(name.to_ascii_uppercase()));
            }
        }
        assert_eq!(*visited.borrow(), expected);
        let candidates = [
            dirs[0].join("DOOM2.WAD"),
            dirs[0].join("plutonia.wad"),
            dirs[1].join("doom2.wad"),
        ];
        assert_eq!(
            find_iwad(Some(explicit), &dirs, |path| candidates
                .contains(&path.to_path_buf())),
            Some(candidates[0].clone())
        );
        assert_eq!(find_iwad(None, &[], |_| true), None);
    }

    #[cfg(unix)]
    #[test]
    fn environment_paths_preserve_non_unicode_bytes() {
        use std::os::unix::ffi::OsStringExt;
        let raw = OsString::from_vec(b"/wads/\xff".to_vec());
        let dirs = iwad_search_dirs(|key| (key == "DOOMWADDIR").then(|| raw.clone()));
        assert_eq!(dirs[0], PathBuf::from(raw));
    }
}
