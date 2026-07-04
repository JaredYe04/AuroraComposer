//! Musical primitive types: beats, pitch, duration.
//!
//! See `docs/02-music-model/ast.md` §8.1.

use serde::{Deserialize, Serialize};

/// Beat offset within a measure as an exact rational (quarter-note fractions).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BeatOffset {
    pub numer: u32,
    pub denom: u32,
}

impl BeatOffset {
    #[must_use]
    pub const fn new(numer: u32, denom: u32) -> Self {
        Self { numer, denom }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self { numer: 0, denom: 1 }
    }
}

/// Pitch: MIDI number for computation, optional spelling for notation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pitch {
    pub midi: u8,
    pub spelling: Option<PitchSpelling>,
}

impl Pitch {
    #[must_use]
    pub const fn from_midi(midi: u8) -> Self {
        Self {
            midi,
            spelling: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchSpelling {
    pub step: Step,
    pub alter: i8,
    pub octave: i8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

/// Written duration for notation; tick duration is computed via timeline projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrittenDuration {
    pub note_type: NoteType,
    pub dots: u8,
    pub tuplet: Option<TupletSpec>,
}

impl Default for WrittenDuration {
    fn default() -> Self {
        Self {
            note_type: NoteType::Quarter,
            dots: 0,
            tuplet: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteType {
    Maxima,
    Longa,
    Breve,
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    OneHundredTwentyEighth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TupletSpec {
    pub actual: u8,
    pub normal: u8,
    pub normal_type: NoteType,
}

/// MIDI register bounds for a voice (see `voices.md` §8.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchRange {
    pub min_midi: u8,
    pub max_midi: u8,
    pub preferred_min: u8,
    pub preferred_max: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_offset_orders_lexicographically() {
        assert!(BeatOffset::zero() < BeatOffset::new(1, 4));
    }

    #[test]
    fn pitch_serializes_midi() {
        let pitch = Pitch::from_midi(60);
        let json = serde_json::to_value(&pitch).unwrap();
        assert_eq!(json["midi"], 60);
    }

    #[test]
    fn written_duration_defaults_to_quarter() {
        let d = WrittenDuration::default();
        assert!(matches!(d.note_type, NoteType::Quarter));
    }
}
