//! Shared rhythmic comping patterns for harmonic voices (quarter-grid with offsets).

use aurora_ast::{BeatOffset, NoteType};

/// One hit in a comping pattern for a single beat slot.
#[derive(Clone, Copy, Debug)]
pub struct CompHit {
    /// Fraction of beat (0.0 = on beat, 0.5 = offbeat eighth).
    pub offset_frac: f32,
    pub note_type: NoteType,
    pub velocity_scale: f32,
}

/// Comping mask for one beat in 4/4 (may have 0–2 hits).
pub fn comp_hits_for_beat(
    beat: usize,
    measure_idx: usize,
    syncopation: f32,
    density: f32,
) -> Vec<CompHit> {
    let pattern_idx = (measure_idx + beat / 2) % 4;
    let hits = match pattern_idx {
        0 => vec![
            CompHit {
                offset_frac: 0.0,
                note_type: NoteType::Quarter,
                velocity_scale: 1.0,
            },
            if syncopation > 0.4 {
                CompHit {
                    offset_frac: 0.5,
                    note_type: NoteType::Eighth,
                    velocity_scale: 0.65,
                }
            } else {
                CompHit {
                    offset_frac: 0.0,
                    note_type: NoteType::Quarter,
                    velocity_scale: 0.0,
                }
            },
        ],
        1 => vec![
            CompHit {
                offset_frac: 0.0,
                note_type: NoteType::Eighth,
                velocity_scale: 0.85,
            },
            CompHit {
                offset_frac: 0.5,
                note_type: NoteType::Eighth,
                velocity_scale: 0.7,
            },
        ],
        2 => vec![CompHit {
            offset_frac: if syncopation > 0.5 { 0.25 } else { 0.0 },
            note_type: if density > 0.6 {
                NoteType::Quarter
            } else {
                NoteType::Eighth
            },
            velocity_scale: 0.9,
        }],
        _ => vec![
            CompHit {
                offset_frac: 0.0,
                note_type: NoteType::Eighth,
                velocity_scale: 0.75,
            },
            if syncopation > 0.35 {
                CompHit {
                    offset_frac: 0.75,
                    note_type: NoteType::Sixteenth,
                    velocity_scale: 0.55,
                }
            } else {
                CompHit {
                    offset_frac: 0.0,
                    note_type: NoteType::Quarter,
                    velocity_scale: 0.0,
                }
            },
        ],
    };
    hits.into_iter()
        .filter(|h| h.velocity_scale > 0.01)
        .collect()
}

pub fn beat_offset_from_frac(beat: u32, frac: f32, subdiv: u32) -> BeatOffset {
    BeatOffset::new(beat * subdiv + (frac * subdiv as f32).round() as u32, subdiv)
}
