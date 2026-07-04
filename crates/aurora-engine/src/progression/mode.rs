//! Mode-aware scale degrees and chord realization.

use aurora_ast::{ChordQuality, HarmonicFunction, Mode};

/// Semitone steps from tonic for degrees 1–7.
#[derive(Clone, Copy, Debug)]
pub struct ModeScale {
    pub mode: Mode,
    pub steps: [u8; 7],
}

const MAJOR: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
const AEOLIAN: [u8; 7] = [0, 2, 3, 5, 7, 8, 10];
const DORIAN: [u8; 7] = [0, 2, 3, 5, 7, 9, 10];
const PHRYGIAN: [u8; 7] = [0, 1, 3, 5, 7, 8, 10];
const LYDIAN: [u8; 7] = [0, 2, 4, 6, 7, 9, 11];
const MIXOLYDIAN: [u8; 7] = [0, 2, 4, 5, 7, 9, 10];

impl ModeScale {
    pub fn from_mode(mode: Mode) -> Self {
        let steps = match mode {
            Mode::Major => MAJOR,
            Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor => AEOLIAN,
            Mode::Dorian => DORIAN,
            Mode::Phrygian => PHRYGIAN,
            Mode::Lydian => LYDIAN,
            Mode::Mixolydian => MIXOLYDIAN,
            Mode::Locrian | Mode::Custom(_) => AEOLIAN,
        };
        Self { mode, steps }
    }

    pub fn degree_pc(&self, tonic: u8, degree: u8, accidental: i8) -> u8 {
        let idx = (degree.saturating_sub(1) % 7) as usize;
        let base = (tonic + self.steps[idx]) % 12;
        ((base as i16 + accidental as i16).rem_euclid(12)) as u8
    }
}

/// Parse user mode string into AST mode.
pub fn parse_mode(s: &str) -> Mode {
    match s.to_lowercase().as_str() {
        "major" | "ionian" => Mode::Major,
        "minor" | "aeolian" | "natural_minor" => Mode::NaturalMinor,
        "dorian" => Mode::Dorian,
        "phrygian" => Mode::Phrygian,
        "lydian" => Mode::Lydian,
        "mixolydian" => Mode::Mixolydian,
        _ => Mode::Major,
    }
}

/// Default triad quality for a scale degree in a given mode.
pub fn default_triad_quality(mode: Mode, degree: u8) -> ChordQuality {
    match (mode, degree) {
        (Mode::Major, 1 | 4 | 5) => ChordQuality::Major,
        (Mode::Major, 2 | 3 | 6) => ChordQuality::Minor,
        (Mode::Major, 7) => ChordQuality::Diminished,

        (Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor, 1 | 4) => {
            ChordQuality::Minor
        }
        (Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor, 3 | 6 | 7) => {
            ChordQuality::Major
        }
        (Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor, 2 | 5) => {
            ChordQuality::Minor
        }

        (Mode::Dorian, 1 | 4 | 5) => ChordQuality::Minor,
        (Mode::Dorian, 2 | 3 | 6 | 7) => ChordQuality::Major,

        (Mode::Phrygian, 1 | 4 | 5) => ChordQuality::Minor,
        (Mode::Phrygian, 2 | 3 | 6 | 7) => ChordQuality::Major,

        (Mode::Lydian, 1 | 2 | 4 | 5) => ChordQuality::Major,
        (Mode::Lydian, 3 | 6 | 7) => ChordQuality::Minor,

        (Mode::Mixolydian, 1 | 4) => ChordQuality::Major,
        (Mode::Mixolydian, 2 | 3 | 6 | 7) => ChordQuality::Minor,
        (Mode::Mixolydian, 5) => ChordQuality::Minor,

        _ => ChordQuality::Major,
    }
}

/// Harmonic function for a diatonic degree.
pub fn default_function(mode: Mode, degree: u8) -> HarmonicFunction {
    match degree {
        1 | 3 | 6 => HarmonicFunction::Tonic,
        2 | 4 | 7 => HarmonicFunction::Subdominant,
        5 => HarmonicFunction::Dominant,
        _ => HarmonicFunction::Tonic,
    }
}

pub fn mode_degree_pc(tonic: u8, mode: Mode, degree: u8) -> u8 {
    ModeScale::from_mode(mode).degree_pc(tonic, degree, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dorian_iv_is_major() {
        let scale = ModeScale::from_mode(Mode::Dorian);
        assert_eq!(scale.degree_pc(0, 4, 0), 5); // F in C dorian
    }

    #[test]
    fn mixolydian_bvii() {
        let scale = ModeScale::from_mode(Mode::Mixolydian);
        assert_eq!(scale.degree_pc(0, 7, 0), 10); // Bb in C mixolydian
    }
}
