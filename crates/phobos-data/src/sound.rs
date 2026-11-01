// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Decodes DMX samples, PC speaker tones, MUS and MIDI music, and GENMIDI instruments.
//!
//! These decoders follow the Unofficial Doom Specs and Doom Wiki format
//! descriptions. They preserve encoded values for the audio subsystem, strip
//! DMX padding, and tolerate overstated MUS score lengths and missing bank names.

use crate::{DataError, reader::Reader};

// DMX format identifiers and padding sizes from the Doom sound lump format.
const DMX_FORMAT: u16 = 3;
const PC_SPEAKER_FORMAT: u16 = 0;
const DMX_PADDING: usize = 16;
// MUS header signature and channel limit from the MUS file format.
const MUS_MAGIC: [u8; 4] = *b"MUS\x1a";
const MUS_MAX_PRIMARY_CHANNELS: u16 = 15;
// Standard MIDI File header signature and minimum payload size from its format.
const MIDI_MAGIC: [u8; 4] = *b"MThd";
const MIDI_HEADER_LENGTH: usize = 6;
// GENMIDI layout from the Doom OPL instrument bank format.
const GENMIDI_MAGIC: [u8; 8] = *b"#OPL_II#";
const GENMIDI_INSTRUMENT_SIZE: usize = 36;
const GENMIDI_NAME_SIZE: usize = 32;

/// The GENMIDI bank contains 128 melodic and 47 percussion instruments.
pub const GENMIDI_INSTRUMENTS: usize = 175;

/// An unsigned eight-bit PCM sound with DMX padding removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmxSound {
    /// The playback sample rate in hertz.
    pub sample_rate: u16,
    /// The playable unsigned eight-bit samples.
    pub samples: Vec<u8>,
}

impl DmxSound {
    /// Decodes a DS lump and removes its leading and trailing padding.
    ///
    /// # Errors
    /// Returns an error for a truncated header or samples, an unsupported
    /// format, or a sample count with no playable samples after padding.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut r = Reader::new("DMX", bytes);
        if r.u16()? != DMX_FORMAT {
            return Err(DataError::invalid("DMX", "format must be 3"));
        }
        let sample_rate = r.u16()?;
        let count = usize::try_from(r.u32()?).map_err(|_overflow| {
            DataError::truncated("DMX", r.position(), usize::MAX, bytes.len())
        })?;
        let data = r.bytes(count)?;
        if count <= DMX_PADDING * 2 {
            return Err(DataError::invalid(
                "DMX",
                "no playable samples after padding",
            ));
        }
        let mut samples = Reader::new("DMX", data);
        samples.bytes(DMX_PADDING)?;
        Ok(Self {
            sample_rate,
            samples: samples.bytes(count - DMX_PADDING * 2)?.to_vec(),
        })
    }
}

/// A sequence of encoded PC speaker tones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcSpeakerSound {
    /// The tone bytes in playback order.
    pub tones: Vec<u8>,
}

impl PcSpeakerSound {
    /// Decodes a DP lump.
    ///
    /// # Errors
    /// Returns an error for a truncated header or tones, or a nonzero format.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut r = Reader::new("PC speaker", bytes);
        if r.u16()? != PC_SPEAKER_FORMAT {
            return Err(DataError::invalid("PC speaker", "format must be 0"));
        }
        let count = usize::from(r.u16()?);
        Ok(Self {
            tones: r.bytes(count)?.to_vec(),
        })
    }
}

/// A MUS header, instrument list, and encoded score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusLump {
    /// The number of primary channels.
    pub primary_channels: u16,
    /// The number of secondary channels.
    pub secondary_channels: u16,
    /// The instrument numbers referenced by the score.
    pub instruments: Vec<u16>,
    /// The encoded score, capped at the end of the lump.
    pub score: Vec<u8>,
}

impl MusLump {
    /// Decodes a MUS lump, tolerating an overstated score length.
    ///
    /// # Errors
    /// Returns an error for a truncated header or instrument list, an invalid
    /// signature, more than 15 primary channels, or a score start outside the lump.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut r = Reader::new("MUS", bytes);
        if r.array::<4>()? != MUS_MAGIC {
            return Err(DataError::invalid("MUS", "incorrect signature"));
        }
        let score_length = usize::from(r.u16()?);
        let score_start = usize::from(r.u16()?);
        let primary_channels = r.u16()?;
        let secondary_channels = r.u16()?;
        let instrument_count = usize::from(r.u16()?);
        let _reserved = r.u16()?;
        if primary_channels > MUS_MAX_PRIMARY_CHANNELS {
            return Err(DataError::invalid("MUS", "more than 15 primary channels"));
        }
        let mut instrument_reader = Reader::new("MUS", r.bytes(instrument_count * 2)?);
        let mut instruments = Vec::with_capacity(instrument_count);
        for _ in 0..instrument_count {
            instruments.push(instrument_reader.u16()?);
        }
        r.seek(score_start)?;
        if r.remaining() == 0 {
            return Err(DataError::truncated("MUS", score_start, 1, bytes.len()));
        }
        let score = r.bytes(score_length.min(r.remaining()))?.to_vec();
        Ok(Self {
            primary_channels,
            secondary_channels,
            instruments,
            score,
        })
    }
}

/// A MIDI lump with its header checked and its tracks preserved for sequencing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiLump {
    /// The complete MIDI file, including its header and encoded tracks.
    pub bytes: Vec<u8>,
}

/// Music encoded as a MUS score or a standard MIDI file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Music {
    /// A decoded MUS header, instrument list, and score.
    Mus(MusLump),
    /// A MIDI file with a checked header.
    Midi(MidiLump),
}

/// Decodes music according to its MUS or MIDI signature.
///
/// MIDI header extensions and tracks are preserved without interpreting them.
/// Format, track count, and division are read as big-endian header fields;
/// their semantic validation belongs to the sequencer.
///
/// # Errors
/// Returns an error for an unknown signature, an invalid or truncated MUS
/// lump, or a MIDI header shorter than six bytes or extending past the lump.
pub fn decode_music(bytes: &[u8]) -> Result<Music, DataError> {
    let mut r = Reader::new("music", bytes);
    let magic = r
        .array::<4>()
        .map_err(|_error| DataError::invalid("music", "unknown signature"))?;
    match magic {
        MUS_MAGIC => MusLump::decode(bytes).map(Music::Mus),
        MIDI_MAGIC => {
            let length = usize::try_from(u32::from_be_bytes(r.array()?)).map_err(|_overflow| {
                DataError::truncated("MIDI", r.position(), usize::MAX, bytes.len())
            })?;
            if length < MIDI_HEADER_LENGTH {
                return Err(DataError::invalid(
                    "MIDI",
                    "header length must be at least 6",
                ));
            }
            let mut header = Reader::new("MIDI", r.bytes(length)?);
            let _format = u16::from_be_bytes(header.array()?);
            let _track_count = u16::from_be_bytes(header.array()?);
            let _division = u16::from_be_bytes(header.array()?);
            Ok(Music::Midi(MidiLump {
                bytes: bytes.to_vec(),
            }))
        }
        _ => Err(DataError::invalid("music", "unknown signature")),
    }
}

/// The six encoded register values for one OPL operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OplOperator {
    /// The tremolo, vibrato, sustain, envelope, and multiplier bits.
    pub tremolo: u8,
    /// The attack and decay rates.
    pub attack: u8,
    /// The sustain level and release rate.
    pub sustain: u8,
    /// The waveform selector.
    pub waveform: u8,
    /// The key scale level.
    pub scale: u8,
    /// The output level.
    pub level: u8,
}

impl OplOperator {
    fn read(r: &mut Reader<'_>) -> Result<Self, DataError> {
        Ok(Self {
            tremolo: r.u8()?,
            attack: r.u8()?,
            sustain: r.u8()?,
            waveform: r.u8()?,
            scale: r.u8()?,
            level: r.u8()?,
        })
    }
}

/// One two-operator voice in a GENMIDI instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenMidiVoice {
    /// The modulating operator.
    pub modulator: OplOperator,
    /// The feedback and connection bits.
    pub feedback: u8,
    /// The carrier operator.
    pub carrier: OplOperator,
    /// The unused byte, preserved from the bank.
    pub unused: u8,
    /// The signed base note offset.
    pub base_note_offset: i16,
}

impl GenMidiVoice {
    fn read(r: &mut Reader<'_>) -> Result<Self, DataError> {
        Ok(Self {
            modulator: OplOperator::read(r)?,
            feedback: r.u8()?,
            carrier: OplOperator::read(r)?,
            unused: r.u8()?,
            base_note_offset: r.i16()?,
        })
    }
}

/// A GENMIDI instrument with its two encoded voices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenMidiInstrument {
    /// The instrument behavior flags.
    pub flags: u16,
    /// The fine tuning value for the second voice.
    pub fine_tuning: u8,
    /// The fixed note number.
    pub fixed_note: u8,
    /// The two OPL voices.
    pub voices: [GenMidiVoice; 2],
}

impl GenMidiInstrument {
    fn read(r: &mut Reader<'_>) -> Result<Self, DataError> {
        Ok(Self {
            flags: r.u16()?,
            fine_tuning: r.u8()?,
            fixed_note: r.u8()?,
            voices: [GenMidiVoice::read(r)?, GenMidiVoice::read(r)?],
        })
    }
}

/// A GENMIDI instrument bank and its optional raw instrument names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenMidi {
    /// All 175 melodic and percussion instruments.
    pub instruments: Vec<GenMidiInstrument>,
    /// Up to 175 complete names, preserved as raw bytes.
    pub names: Vec<[u8; 32]>,
}

impl GenMidi {
    /// Decodes a GENMIDI lump and all complete names within its bank size.
    ///
    /// # Errors
    /// Returns an error for an invalid signature or a truncated instrument bank.
    pub fn decode(bytes: &[u8]) -> Result<Self, DataError> {
        let mut r = Reader::new("GENMIDI", bytes);
        if r.array::<8>()? != GENMIDI_MAGIC {
            return Err(DataError::invalid("GENMIDI", "incorrect signature"));
        }
        let mut bank = Reader::new(
            "GENMIDI",
            r.bytes(GENMIDI_INSTRUMENTS * GENMIDI_INSTRUMENT_SIZE)?,
        );
        let mut instruments = Vec::with_capacity(GENMIDI_INSTRUMENTS);
        for _ in 0..GENMIDI_INSTRUMENTS {
            instruments.push(GenMidiInstrument::read(&mut bank)?);
        }
        let name_count = GENMIDI_INSTRUMENTS.min(r.remaining() / GENMIDI_NAME_SIZE);
        let mut names = Vec::with_capacity(name_count);
        for _ in 0..name_count {
            names.push(r.array()?);
        }
        Ok(Self { instruments, names })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dmx(count: u32) -> Vec<u8> {
        let mut bytes = vec![3, 0, 0x11, 0x2b];
        bytes.extend_from_slice(&count.to_le_bytes());
        bytes
    }

    fn mus(start: u16, length: u16, channels: u16, instruments: u16) -> Vec<u8> {
        let mut bytes = MUS_MAGIC.to_vec();
        for field in [length, start, channels, 2, instruments, 0] {
            bytes.extend_from_slice(&field.to_le_bytes());
        }
        bytes
    }

    fn bank() -> Vec<u8> {
        let mut bytes = GENMIDI_MAGIC.to_vec();
        bytes.resize(8 + GENMIDI_INSTRUMENTS * GENMIDI_INSTRUMENT_SIZE, 0);
        bytes
    }

    fn midi(length: u32) -> Vec<u8> {
        let mut bytes = MIDI_MAGIC.to_vec();
        bytes.extend_from_slice(&length.to_be_bytes());
        for field in [1_u16, 2, 480] {
            bytes.extend_from_slice(&field.to_be_bytes());
        }
        bytes
    }

    #[test]
    fn music_dispatches_mus_and_keeps_mus_decoder_strict() {
        let mut bytes = mus(16, 1, 0, 0);
        bytes.push(0x60);
        assert_eq!(
            decode_music(&bytes).unwrap(),
            Music::Mus(MusLump::decode(&bytes).unwrap())
        );
        assert!(MusLump::decode(&midi(6)).unwrap_err().is_invalid());
        assert!(decode_music(&mus(16, 1, 16, 0)).unwrap_err().is_invalid());
        assert!(decode_music(&MUS_MAGIC).unwrap_err().is_truncated());
    }

    #[test]
    fn midi_preserves_the_whole_lump_and_accepts_header_extensions() {
        let bytes = midi(6);
        assert_eq!(
            decode_music(&bytes).unwrap(),
            Music::Midi(MidiLump {
                bytes: bytes.clone()
            })
        );
        let mut extended = midi(8);
        extended.extend_from_slice(&[0xab, 0xcd]);
        // Track parsing is deferred, so arbitrary trailing bytes are preserved.
        extended.extend_from_slice(b"MTrk\0\0\0\x04\0\xff\x2f\0");
        assert_eq!(
            decode_music(&extended).unwrap(),
            Music::Midi(MidiLump { bytes: extended })
        );
    }

    #[test]
    fn midi_rejects_every_truncated_header_prefix() {
        let bytes = midi(6);
        for end in 4..bytes.len() {
            assert!(decode_music(&bytes[..end]).unwrap_err().is_truncated());
        }
        assert!(decode_music(&midi(7)).unwrap_err().is_truncated());
        assert!(decode_music(&midi(u32::MAX)).unwrap_err().is_truncated());
        // Little-endian 6 must be interpreted as a huge big-endian length.
        let mut wrong_endian = midi(6);
        wrong_endian[4..8].copy_from_slice(&6_u32.to_le_bytes());
        assert!(decode_music(&wrong_endian).unwrap_err().is_truncated());
    }

    #[test]
    fn music_rejects_unknown_magic_and_short_midi_header_lengths() {
        for bytes in [b"".as_slice(), b"M", b"MT", b"MTh", b"RIFF", b"MUS\0"] {
            assert!(decode_music(bytes).unwrap_err().is_invalid());
        }
        for length in 0..6 {
            assert!(decode_music(&midi(length)).unwrap_err().is_invalid());
        }
    }

    #[test]
    fn dmx_removes_both_pads_and_ignores_trailing_bytes() {
        let mut bytes = dmx(35);
        bytes.extend_from_slice(&[9; 16]);
        bytes.extend_from_slice(&[0, 128, 255]);
        bytes.extend_from_slice(&[8; 16]);
        bytes.push(7);
        let sound = DmxSound::decode(&bytes).unwrap();
        assert_eq!(sound.sample_rate, 11025);
        assert_eq!(sound.samples, [0, 128, 255]);
        for end in 0..43 {
            assert!(DmxSound::decode(&bytes[..end]).unwrap_err().is_truncated());
        }
    }

    #[test]
    fn dmx_rejects_invalid_formats_and_unplayable_counts() {
        assert!(DmxSound::decode(&[0, 0]).unwrap_err().is_invalid());
        for count in [0, 1, 16, 31, 32] {
            let mut bytes = dmx(count);
            bytes.resize(8 + usize::try_from(count).unwrap(), 0);
            assert!(DmxSound::decode(&bytes).unwrap_err().is_invalid());
        }
        assert!(DmxSound::decode(&dmx(u32::MAX)).unwrap_err().is_truncated());
        let mut minimal = dmx(33);
        minimal.resize(41, 128);
        assert_eq!(DmxSound::decode(&minimal).unwrap().samples, [128]);
    }

    #[test]
    fn pc_speaker_reads_exact_tone_count() {
        let bytes = [0, 0, 3, 0, 0, 42, 255, 99];
        assert_eq!(PcSpeakerSound::decode(&bytes).unwrap().tones, [0, 42, 255]);
        assert!(
            PcSpeakerSound::decode(&[0, 0, 0, 0])
                .unwrap()
                .tones
                .is_empty()
        );
        for end in 0..7 {
            assert!(
                PcSpeakerSound::decode(&bytes[..end])
                    .unwrap_err()
                    .is_truncated()
            );
        }
        assert!(PcSpeakerSound::decode(&[3, 0]).unwrap_err().is_invalid());
    }

    #[test]
    fn mus_preserves_fields_and_uses_score_offset() {
        let mut bytes = mus(22, 2, 15, 2);
        bytes.extend_from_slice(&[0x34, 0x12, 0xff, 0xff, 9, 9, 0x60, 0x80, 7]);
        let music = MusLump::decode(&bytes).unwrap();
        assert_eq!(music.primary_channels, 15);
        assert_eq!(music.secondary_channels, 2);
        assert_eq!(music.instruments, [0x1234, 0xffff]);
        assert_eq!(music.score, [0x60, 0x80]);
        for end in 0..20 {
            assert!(MusLump::decode(&bytes[..end]).unwrap_err().is_truncated());
        }
    }

    #[test]
    fn mus_caps_overstated_length_and_accepts_empty_scores_inside_lump() {
        let mut bytes = mus(16, u16::MAX, 0, 0);
        bytes.extend_from_slice(&[0x60, 0x00]);
        assert_eq!(MusLump::decode(&bytes).unwrap().score, [0x60, 0]);
        bytes[4..6].copy_from_slice(&0_u16.to_le_bytes());
        assert!(MusLump::decode(&bytes).unwrap().score.is_empty());
    }

    #[test]
    fn mus_rejects_signature_channels_and_outside_score_offsets() {
        let mut bytes = mus(16, 1, 0, 0);
        bytes[0] = b'X';
        assert!(MusLump::decode(&bytes).unwrap_err().is_invalid());
        assert!(
            MusLump::decode(&mus(16, 1, 16, 0))
                .unwrap_err()
                .is_invalid()
        );
        for start in [16, 17, u16::MAX] {
            assert!(
                MusLump::decode(&mus(start, 0, 0, 0))
                    .unwrap_err()
                    .is_truncated()
            );
        }
        assert!(
            MusLump::decode(&mus(16, 1, 0, u16::MAX))
                .unwrap_err()
                .is_truncated()
        );
    }

    #[test]
    fn genmidi_preserves_every_voice_field() {
        let mut bytes = bank();
        bytes[8..12].copy_from_slice(&[0x34, 0x12, 99, 60]);
        let voice = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 0xfe, 0xff];
        bytes[12..28].copy_from_slice(&voice);
        bytes[28..44].copy_from_slice(&voice);
        bytes[42..44].copy_from_slice(&1234_i16.to_le_bytes());
        let decoded = GenMidi::decode(&bytes).unwrap();
        assert_eq!(decoded.instruments.len(), GENMIDI_INSTRUMENTS);
        assert!(decoded.names.is_empty());
        let instrument = decoded.instruments[0];
        assert_eq!(instrument.flags, 0x1234);
        assert_eq!(instrument.fine_tuning, 99);
        assert_eq!(instrument.fixed_note, 60);
        let first = instrument.voices[0];
        assert_eq!(
            first.modulator,
            OplOperator {
                tremolo: 1,
                attack: 2,
                sustain: 3,
                waveform: 4,
                scale: 5,
                level: 6,
            }
        );
        assert_eq!(first.feedback, 7);
        assert_eq!(
            first.carrier,
            OplOperator {
                tremolo: 8,
                attack: 9,
                sustain: 10,
                waveform: 11,
                scale: 12,
                level: 13,
            }
        );
        assert_eq!(first.unused, 14);
        assert_eq!(first.base_note_offset, -2);
        assert_eq!(instrument.voices[1].base_note_offset, 1234);
    }

    #[test]
    fn genmidi_reads_only_complete_names_and_caps_the_count() {
        let mut bytes = bank();
        bytes.extend_from_slice(&[0xff; 32]);
        bytes.extend_from_slice(&[1; 31]);
        assert_eq!(GenMidi::decode(&bytes).unwrap().names, [[0xff; 32]]);
        bytes.push(1);
        assert_eq!(
            GenMidi::decode(&bytes).unwrap().names,
            [[0xff; 32], [1; 32]]
        );
        bytes.resize(bank().len() + (GENMIDI_INSTRUMENTS + 1) * 32, 0);
        assert_eq!(
            GenMidi::decode(&bytes).unwrap().names.len(),
            GENMIDI_INSTRUMENTS
        );
    }

    #[test]
    fn genmidi_rejects_bad_signature_and_every_truncated_bank_prefix() {
        let mut bytes = bank();
        for end in 0..bytes.len() {
            assert!(GenMidi::decode(&bytes[..end]).unwrap_err().is_truncated());
        }
        bytes[0] = b'!';
        assert!(GenMidi::decode(&bytes).unwrap_err().is_invalid());
    }
}
