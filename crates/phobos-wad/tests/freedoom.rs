// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Checks real Freedoom archives when their directory is supplied.

use phobos_wad::{GameMission, GameMode, LumpName, LumpNamespace, Wad, identify};

#[test]
fn freedoom_archives_and_namespace() {
    let Some(dir) = std::env::var_os("PHOBOS_FREEDOOM_DIR") else {
        eprintln!("skipped Freedoom integration test: PHOBOS_FREEDOOM_DIR is unset");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let first = Wad::read(dir.join("freedoom1.wad")).unwrap();
    let second = Wad::read(dir.join("freedoom2.wad")).unwrap();
    for (wad, mode, mission) in [
        (&first, GameMode::Retail, GameMission::Doom),
        (&second, GameMode::Commercial, GameMission::Doom2),
    ] {
        let id = identify(wad, None).unwrap();
        assert_eq!(id.mode(), mode);
        assert_eq!(id.mission(), mission);
        assert!(id.is_freedoom());
        assert!(wad.find_last(LumpName::new("PLAYPAL").unwrap()).is_some());
    }
    let first_len = first.lumps().len();
    let mut namespace = LumpNamespace::new();
    namespace.push(first);
    assert!(
        !namespace
            .between(
                LumpName::new("S_START").unwrap(),
                LumpName::new("S_END").unwrap(),
            )
            .is_empty()
    );
    namespace.push(second);
    let palette = namespace.find(LumpName::new("PLAYPAL").unwrap()).unwrap();
    assert!(palette.index() >= first_len);
    assert!(!namespace.data(palette).unwrap().is_empty());
}
