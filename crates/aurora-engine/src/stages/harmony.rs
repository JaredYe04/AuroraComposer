use aurora_ast::{
    HarmonicFunction, HarmonySlot, PipelineStageId, Provenance, ProvenanceAgent, ProvenanceSource,
};
use aurora_core::NodeId;

use crate::progression::{
    build_per_beat_chord_grid, expand_measure_harmony_slots, plan_progression,
    tension_curve, ProgressionPlanContext,
};

use super::common::iter_measures_mut;
use super::PipelineState;

/// Stage 5 — Harmony Skeleton with P3–P5 enrichment and P7 harmonic rhythm.
pub fn generate_harmony(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let total_measures = super::total_bars(&state.params) as usize;
    let cadence_measures: Vec<u32> = state.cadence_chord_roots.keys().copied().collect();
    let tension_targets = tension_curve(total_measures, &state.params.emotion.tension_curve);

    let ctx = ProgressionPlanContext {
        params: &state.params,
        style: &state.style,
        key_map: &state.composition.global.key_map,
        total_measures,
        tension_targets,
        cadence_measures,
    };

    let progression = plan_progression(&ctx);
    let loop_mode = state.params.harmony.progression_mode
        == aurora_core::ProgressionMode::Loop;
    let beats = state.params.rhythm.time_signature_beats.max(1);

    for (i, measure) in iter_measures_mut(&mut state.composition).enumerate() {
        let global = measure.number.global;
        let mut chord = progression
            .get(i)
            .cloned()
            .ok_or_else(|| format!("progression missing slot {i}"))?;

        if let Some(&root_pc) = state.cadence_chord_roots.get(&global) {
            chord.symbol.root.pc = root_pc;
            chord.symbol.raw = format!(
                "{}{}",
                crate::progression::templates::chord_root_name(root_pc),
                crate::progression::templates::quality_suffix(chord.symbol.quality)
            );
            chord.function = HarmonicFunction::Dominant;
        }

        let rule_id = if state.style.jazz_harmony {
            "JAZZ-IIV-001".to_string()
        } else if loop_mode {
            "HARM-LOOP-001".to_string()
        } else {
            "HARM-PROG-015".to_string()
        };

        measure.harmony_slots.clear();
        measure.harmony_slots.push(HarmonySlot {
            id: NodeId::new(u64::try_from(i + 1000).unwrap_or(1000), 0),
            offset: aurora_ast::BeatOffset::zero(),
            duration: aurora_ast::BeatOffset::new(u32::from(beats), 1),
            symbol: chord.symbol.clone(),
            roman_numeral: Some(chord.roman.clone()),
            function: Some(chord.function),
            provenance: Provenance {
                source: ProvenanceSource::Generated,
                stage: Some(PipelineStageId::HarmonySkeleton),
                rule_ids: {
                    let mut ids = vec![rule_id];
                    ids.extend(chord.rule_ids.clone());
                    ids
                },
                rule_refs: vec![],
                eval_score: None,
                search: None,
                parent: None,
                created_at: created_at.into(),
                agent: ProvenanceAgent::Engine {
                    stage: PipelineStageId::HarmonySkeleton,
                },
                parameters_hash: None,
                explanation: Some(format!(
                    "{} mode: {} at m{}",
                    if loop_mode { "loop" } else { "flow" },
                    chord.roman,
                    global
                )),
            },
        });
    }

    // P7: expand to per-beat harmonic rhythm
    if state.params.harmony.harmonic_rhythm > 0.35 {
        expand_all_harmonic_rhythm(state, &progression, beats);
    }

    let measures: Vec<aurora_ast::Measure> = state
        .composition
        .movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
        .flat_map(|p| &p.measures)
        .cloned()
        .collect();
    state.per_beat_chord_grid = build_per_beat_chord_grid(&measures, beats);

    Ok(())
}

fn expand_all_harmonic_rhythm(
    state: &mut PipelineState,
    progression: &[crate::progression::PlannedChord],
    beats: u8,
) {
    let hr = state.params.harmony.harmonic_rhythm;
    let mut idx = 0usize;
    for movement in &mut state.composition.movements {
        for section in &mut movement.sections {
            for phrase in &mut section.phrases {
                expand_measure_harmony_slots(&mut phrase.measures, progression, hr, beats);
                idx += phrase.measures.len();
                let _ = idx;
            }
        }
    }
}
