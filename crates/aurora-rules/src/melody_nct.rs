//! Melodic non-chord tone classification for consonance scoring.

use aurora_ast::{ChordSymbol, KeySignature, Pitch};

use crate::scale::mode_scale_pcs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MelodicNctKind {
    ChordTone,
    DiatonicNeighbor,
    ChromaticNeighbor,
    DiatonicPassing,
    ChromaticPassing,
    ApproachTone,
    Other,
}

pub fn is_diatonic_pc(pc: u8, key: &KeySignature) -> bool {
    mode_scale_pcs(key).contains(&pc)
}

pub fn classify_melodic_nct(
    pitch: Pitch,
    prev: Option<Pitch>,
    chord: &ChordSymbol,
    key: &KeySignature,
) -> MelodicNctKind {
    let pc = pitch.midi % 12;
    let pcs: Vec<u8> = chord.pitch_classes().to_vec();

    // 1. Chord tone check
    if pcs.contains(&pc) {
        return MelodicNctKind::ChordTone;
    }

    let Some(prev) = prev else {
        // First note, not chord tone
        return if is_diatonic_pc(pc, key) {
            MelodicNctKind::Other
        } else {
            MelodicNctKind::ChromaticNeighbor
        };
    };

    let interval = pitch.midi as i16 - prev.midi as i16;
    let abs = interval.unsigned_abs();
    let prev_is_ct = pcs.contains(&(prev.midi % 12));
    let diatonic = is_diatonic_pc(pc, key);
    let approaches_ct = pcs.iter().any(|&ct_pc| {
        let dist = (pc as i16 - ct_pc as i16).rem_euclid(12);
        dist == 1 || dist == 11
    });

    // 2. Leaps cannot be passing/neighbor tones (Kostka Ch. 4).
    if abs > 2 {
        return if diatonic {
            MelodicNctKind::Other
        } else {
            MelodicNctKind::ChromaticPassing
        };
    }

    // 3. Stepwise motion (abs <= 2)
    if prev_is_ct {
        // Coming from a chord tone by step
        if approaches_ct {
            // True passing tone: moving between two chord tones by step
            if diatonic {
                MelodicNctKind::DiatonicPassing
            } else {
                MelodicNctKind::ChromaticPassing
            }
        } else if abs == 1 {
            // Step from chord tone, but not approaching another chord
            // This is a neighbor/auxiliary tone or escape tone
            if diatonic {
                MelodicNctKind::DiatonicNeighbor
            } else {
                MelodicNctKind::ChromaticNeighbor
            }
        } else {
            // abs == 2, from chord tone, but not approaching chord
            // Could be a double neighbor or just a passing step
            if diatonic {
                MelodicNctKind::DiatonicPassing
            } else {
                MelodicNctKind::ChromaticPassing
            }
        }
    } else {
        // Not coming from a chord tone by step
        if approaches_ct {
            // Chromatic approach to a chord tone (appoggiatura-like)
            MelodicNctKind::ApproachTone
        } else if abs == 1 {
            // Free stepwise motion not involving chord tones
            if diatonic {
                MelodicNctKind::Other
            } else {
                MelodicNctKind::ChromaticNeighbor
            }
        } else {
            // abs == 2, not from chord tone, not approaching chord
            if diatonic {
                MelodicNctKind::Other
            } else {
                MelodicNctKind::ChromaticPassing
            }
        }
    }
}
