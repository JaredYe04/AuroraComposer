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
    if pcs.contains(&pc) {
        return MelodicNctKind::ChordTone;
    }

    let Some(prev) = prev else {
        return if is_diatonic_pc(pc, key) {
            MelodicNctKind::Other
        } else {
            MelodicNctKind::ChromaticNeighbor
        };
    };

    let interval = pitch.midi as i16 - prev.midi as i16;
    let abs = interval.unsigned_abs();

    // Leaps cannot be passing/neighbor tones (Kostka Ch. 4).
    if abs > 2 {
        return if is_diatonic_pc(pc, key) {
            MelodicNctKind::Other
        } else {
            MelodicNctKind::ChromaticPassing
        };
    }

    if abs == 1 {
        for &ctp in &pcs {
            let diff = (pc as i16 - ctp as i16).rem_euclid(12);
            if diff == 1 || diff == 11 {
                return if is_diatonic_pc(pc, key) {
                    MelodicNctKind::ApproachTone
                } else {
                    MelodicNctKind::ChromaticPassing
                };
            }
        }
        return if is_diatonic_pc(pc, key) {
            MelodicNctKind::DiatonicNeighbor
        } else {
            MelodicNctKind::ChromaticNeighbor
        };
    }

    // abs == 2: diatonic passing only if stepwise between prev and next would resolve
    if is_diatonic_pc(pc, key) {
        MelodicNctKind::DiatonicPassing
    } else {
        MelodicNctKind::ChromaticPassing
    }
}
