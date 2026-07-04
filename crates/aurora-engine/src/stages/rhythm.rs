use aurora_ast::PipelineStageId;

use super::common::{iter_measures_mut, make_note_provenance};
use super::PipelineState;

/// Stage 6 — Rhythm Skeleton: metric accent grid per measure (pattern selection).
pub fn generate_rhythm(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let beats = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let density = state.params.rhythm.density;
    let syncopation = state.params.rhythm.syncopation;
    let bar_count = super::total_bars(&state.params) as usize;

    state.rhythm_accents.clear();
    state.rhythm_accents.reserve(bar_count);

    for (mi, measure) in iter_measures_mut(&mut state.composition).enumerate() {
        let mut accents = Vec::with_capacity(beats);
        for beat in 0..beats {
            let base = match beat {
                0 => 1.0_f32,
                2 => 0.7,
                _ => 0.4,
            };
            let sync_boost = if syncopation > 0.5 && beat % 2 == 1 {
                syncopation * 0.3
            } else {
                0.0
            };
            accents.push((base * density + sync_boost).clamp(0.1, 1.0));
        }
        state.rhythm_accents.push(accents);

        let pattern_id = if state.style.jazz_harmony {
            "RHY-JAZZ-SWING-4"
        } else {
            "RHY-CLASSICAL-4"
        };
        measure.attributes.rehearsal_mark = Some(format!("{pattern_id}-m{}", mi + 1));
        let _ = make_note_provenance(
            PipelineStageId::RhythmSkeleton,
            created_at,
            vec![pattern_id.into()],
            format!("rhythm skeleton measure {}", measure.number.global),
        );
    }

    Ok(())
}
