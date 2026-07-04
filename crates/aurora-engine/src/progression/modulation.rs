//! P4 — Modulation planning and application via KeyMap.

use aurora_ast::{KeyMap, KeySignature, Mode, PitchClass, TimelinePosition};

use super::roman::{make_chord, reanalyze_chord};
use super::templates::{template_library, PlannedChord};
use super::ProgressionPlanContext;

/// Plan key changes for multi-section pieces.
pub fn plan_key_changes(
    key_map: &mut KeyMap,
    total_measures: usize,
    section_lengths: &[u16],
    section_count: u8,
    tonic: u8,
    mode: Mode,
    policy: &str,
) {
    if section_count < 2 || total_measures < 8 || section_lengths.len() < 2 {
        return;
    }
    if policy == "none" {
        return;
    }

    let boundary = section_lengths[0] as u32;
    if boundary >= total_measures as u32 {
        return;
    }

    let new_tonic = if policy == "aggressive" {
        (tonic + 5) % 12 // up P4
    } else {
        (tonic + 7) % 12 // dominant key (conservative pivot)
    };

    key_map.changes.push(aurora_ast::KeyChange {
        at: TimelinePosition {
            global_measure: boundary + 1,
            beat: aurora_ast::BeatOffset::zero(),
        },
        key: KeySignature {
            tonic: PitchClass { pc: new_tonic },
            mode,
        },
    });
}

/// Apply modulation at key-change boundaries in the progression.
pub fn apply_modulation(
    chords: &mut [PlannedChord],
    key_map: &KeyMap,
    ctx: &ProgressionPlanContext<'_>,
) {
    if key_map.changes.is_empty() {
        return;
    }
    let policy = ctx.params.mode.modulation_policy.as_str();
    let default_tonic = key_map.default.tonic.pc;
    let default_mode = key_map.default.mode;

    for change in &key_map.changes {
        let m = change.at.global_measure.saturating_sub(1) as usize;
        if m >= chords.len() {
            continue;
        }
        let to_tonic = change.key.tonic.pc;
        let from_tonic = if m > 0 {
            key_at_measure(key_map, m as u32 - 1)
        } else {
            default_tonic
        };

        if policy == "aggressive" {
            // Direct cut to I in new key
            chords[m] = make_chord(
                to_tonic,
                aurora_ast::ChordQuality::Major,
                "I",
                aurora_ast::HarmonicFunction::Tonic,
            );
            chords[m].rule_ids = vec!["HARM-PROG-014".into()];
        } else {
            // Conservative: V7 of new key on penultimate, ii-V-I in new key
            if m > 0 {
                chords[m - 1] = make_chord(
                    (to_tonic + 7) % 12,
                    aurora_ast::ChordQuality::Dominant7,
                    "V7",
                    aurora_ast::HarmonicFunction::Dominant,
                );
            }
            if let Some(template) = template_library().iter().find(|t| t.id == "JAZZ-IIV") {
                let cell = template.realize(to_tonic);
                for (j, ch) in cell.iter().enumerate() {
                    if m + j < chords.len() {
                        chords[m + j] = ch.clone();
                    }
                }
            }
            let _ = from_tonic;
        }

        // Reanalyze romans from modulation point onward
        for chord in &mut chords[m..] {
            let (roman, func) = reanalyze_chord(chord, to_tonic, change.key.mode);
            chord.roman = roman;
            chord.function = func;
            chord.rule_ids.push("HARM-045".into());
        }
    }

    let _ = default_mode;
}

fn key_at_measure(key_map: &KeyMap, measure: u32) -> u8 {
    key_map
        .changes
        .iter()
        .filter(|c| c.at.global_measure <= measure + 1)
        .last()
        .map(|c| c.key.tonic.pc)
        .unwrap_or(key_map.default.tonic.pc)
}

/// Tonic PC effective at a given measure index.
pub fn tonic_at_measure(key_map: &KeyMap, measure: usize) -> u8 {
    key_at_measure(key_map, measure as u32)
}
