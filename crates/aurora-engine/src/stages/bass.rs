use aurora_ast::{
    Event, NoteEvent, NoteType, Pitch, PitchRole, PipelineStageId, RuleRef, TieSpec, TimedEventBase,
    WrittenDuration,
};
use aurora_core::NodeId;
use aurora_rules::{
    AstSnapshot, BeamSearchEngine, KeySignature as RuleKey,
    NodeId as RuleNodeId, PitchClass as RulePitchClass, SearchState, StepCountTerminal,
};

use super::common::{
    bass_voice_id, iter_measures, iter_measures_mut, make_search_note_provenance, push_note,
};
use crate::progression::parse_mode;
use super::bass_generator::BassCandidateGenerator;
use super::PipelineState;

/// Stage 9 — Bass: narrow beam search for bass line (root motion / walking).
pub fn generate_bass(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let voice_id = bass_voice_id(state);
    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let bar_count = super::total_bars(&state.params);
    let total_steps = bar_count as usize * beats_per_measure;

    let chord_grid = super::common::collect_per_beat_chord_grid(state, total_steps);
    let measure_ids = collect_measure_ids(state);

    let tonic_pc = state.params.mode.key % 12;
    let rule_key = RuleKey {
        tonic: RulePitchClass { pc: tonic_pc },
        mode: parse_mode(&state.params.mode.mode),
    };

    let bass_register = (
        state.params.register.bass_register_min,
        state.params.register.bass_register_max,
    );

    let initial_snapshot = AstSnapshot {
        key: rule_key,
        bass_register,
        melody_register: (
            state.params.register.melody_register_min,
            state.params.register.melody_register_max,
        ),
        current_chord: chord_grid.first().cloned(),
        ..AstSnapshot::default()
    }
    .with_chord_grid(chord_grid.clone(), u8::try_from(beats_per_measure).unwrap_or(4));

    let mut params = state.params.clone();
    params.search.beam_width = 8;

    let engine = BeamSearchEngine::from_bundle(aurora_rules::prototype_rule_set(), params.clone());
    let generator = BassCandidateGenerator {
        chord_grid,
        measure_ids,
        beats_per_measure: u8::try_from(beats_per_measure).unwrap_or(4),
        bass_register,
        jazz_walk: state.style.jazz_harmony
            || state.params.style.genre.to_lowercase().contains("jazz"),
    };
    let terminal = StepCountTerminal {
        max_steps: u32::try_from(total_steps).unwrap_or(64),
    };

    let result = engine
        .run_beam(SearchState::initial(initial_snapshot), &generator, &terminal)
        .map_err(|e| e.to_string())?;

    let pitches = &result.best_state.snapshot.bass_pitches;
    if pitches.len() != total_steps {
        return Err(format!(
            "bass search produced {} notes, expected {total_steps}",
            pitches.len()
        ));
    }

    for (step, pitch) in pitches.iter().enumerate() {
        let measure_idx = step / beats_per_measure;
        let beat = step % beats_per_measure;
        let rule_refs: Vec<RuleRef> = result
            .best_state
            .applied_rules
            .iter()
            .map(|r| RuleRef {
                id: r.rule_id.as_str().to_string(),
                weight: None,
                score: Some(r.score_delta),
            })
            .collect();

        let note = NoteEvent {
            base: TimedEventBase {
                id: NodeId::new(u64::try_from(30_000 + step).unwrap_or(30_000), 0),
                offset: aurora_ast::BeatOffset::new(beat as u32, 1),
                duration: WrittenDuration {
                    note_type: NoteType::Quarter,
                    dots: 0,
                    tuplet: None,
                },
                provenance: make_search_note_provenance(
                    PipelineStageId::Bass,
                    created_at,
                    vec!["BASS-001".into()],
                    rule_refs,
                    result.best_state.eval_score,
                    u32::try_from(step).unwrap_or(0),
                    result.best_state.beam_rank.unwrap_or(0),
                    params.search.beam_width,
                    &result.best_state.id.0.to_string(),
                    format!("bass step {step}, MIDI {}", pitch.midi),
                ),
                visible: true,
            },
            pitch: Pitch::from_midi(pitch.midi),
            velocity: 75,
            tie: TieSpec::None,
            articulations: vec![],
            ornaments: vec![],
            lyric: None,
            pitch_role: Some(PitchRole::ChordTone),
            stem_direction: None,
            beam_group: None,
            is_drum: false,
            drum_map: None,
        };

        if let Some(measure) = iter_measures_mut(&mut state.composition).nth(measure_idx) {
            push_note(measure, voice_id, Event::Note(note));
        }
    }

    Ok(())
}

fn collect_measure_ids(state: &PipelineState) -> Vec<RuleNodeId> {
    iter_measures(&state.composition)
        .map(|m| RuleNodeId {
            index: m.id.index,
            generation: m.id.generation,
        })
        .collect()
}
