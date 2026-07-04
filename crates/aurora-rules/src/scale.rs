//! Mode scale pitch classes for diatonic checks.

use aurora_ast::{KeySignature, Mode};

/// Pitch classes of the active mode (degree roots 1–7).
pub fn mode_scale_pcs(key: &KeySignature) -> Vec<u8> {
    let pattern: [u8; 7] = match key.mode {
        Mode::Major => [0, 2, 4, 5, 7, 9, 11],
        Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor | Mode::Locrian => {
            [0, 2, 3, 5, 7, 8, 10]
        }
        Mode::Dorian => [0, 2, 3, 5, 7, 9, 10],
        Mode::Phrygian => [0, 1, 3, 5, 7, 8, 10],
        Mode::Lydian => [0, 2, 4, 6, 7, 9, 11],
        Mode::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
        Mode::Custom(_) => [0, 2, 4, 5, 7, 9, 11],
    };
    pattern.iter().map(|s| (key.tonic.pc + s) % 12).collect()
}
