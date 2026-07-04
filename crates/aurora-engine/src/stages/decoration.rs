use aurora_ast::{Event, Ornament, PipelineStageId, Pitch, PitchRole, ProvenanceSource};

use super::common::iter_measures_mut;
use super::PipelineState;

/// Stage 11 — Decoration: ornaments on melody notes (does not alter structural pitches).
pub fn apply_decoration(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let density = state.params.theme.repetition_ratio * 0.15;
    if density < 0.05 {
        return Ok(());
    }

    let mut note_idx = 0usize;
    for measure in iter_measures_mut(&mut state.composition) {
        if let Some(voice) = measure.voices.iter_mut().find(|v| v.voice_id.0 == 0) {
            for event in &mut voice.events {
                let Event::Note(note) = event else {
                    continue;
                };
                if note_idx % 4 == 2 && note.ornaments.is_empty() {
                    note.ornaments.push(Ornament::Mordent { inverted: false });
                    note.base.provenance.stage = Some(PipelineStageId::Decoration);
                    note.base.provenance.rule_ids = vec!["ORNA-MORD-001".into()];
                    note.base.provenance.explanation = Some(format!(
                        "mordent on melody note at measure {}",
                        measure.number.global
                    ));
                    note.base.provenance.created_at = created_at.into();
                    note.pitch_role = Some(PitchRole::Ornament);
                } else if note_idx % 7 == 0 && note.ornaments.is_empty() {
                    let upper = note.pitch.midi.saturating_add(2);
                    note.ornaments.push(Ornament::GraceNote {
                        pitch: Pitch::from_midi(upper.min(127)),
                        steal_ratio: 0.25,
                    });
                    note.base.provenance.stage = Some(PipelineStageId::Decoration);
                    note.base.provenance.rule_ids = vec!["ORNA-GRACE-001".into()];
                    note.base.provenance.source = ProvenanceSource::Generated;
                    note.base.provenance.created_at = created_at.into();
                }
                note_idx += 1;
            }
        }
    }

    Ok(())
}
