// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fuzzes every sound decoder on the same input.
#![no_main]

use libfuzzer_sys::fuzz_target;
use phobos_data::sound::{DmxSound, GenMidi, PcSpeakerSound, decode_music};

fuzz_target!(|data: &[u8]| {
    let _ = DmxSound::decode(data);
    let _ = PcSpeakerSound::decode(data);
    let _ = decode_music(data);
    let _ = GenMidi::decode(data);
});
