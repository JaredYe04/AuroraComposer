use aurora_ast::{
    Event, NoteEvent, NoteType, Pitch, PitchRole, PipelineStageId, RuleRef, TieSpec, TimedEventBase,
    WrittenDuration,
};
use aurora_core::NodeId;
use aurora_rules::{
    AstSnapshot, BeamSearchEngine, CandidateGenerator, CandidatePatch, ChordSymbol as RuleChord,
    KeySignature as RuleKey, NodeId as RuleNodeId, Pitch as RulePitch,
    PitchClass as RulePitchClass, SearchState, StepCountTerminal, VoiceId as RuleVoiceId,
    search_note,
};

use super::common::{
    alto_voice_id, collect_melody_pitches, collect_per_beat_chord_grid, iter_measures,
    iter_measures_mut, make_search_note_provenance, push_note,
};
use crate::progression::parse_mode;
use super::chord_voice::generate_chord_voice;
use super::harmony_pad::generate_harmony_pad;
use super::PipelineState;

/// Stage 8 — Counterpoint: alto beam search, or HarmonyPad when homophonic.
pub fn generate_counterpoint(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    if super::common::harmony_pad_enabled(state) {
        return generate_harmony_pad(state, created_at);
    }
    if super::common::accompaniment_enabled(state) {
        return generate_chord_voice(state, created_at);
    }

    let Some(alto_id) = alto_voice_id(state) else {
        return Ok(());
    };

    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let bar_count = super::total_bars(&state.params);
    let total_steps = bar_count as usize * beats_per_measure;

    let chord_grid = collect_per_beat_chord_grid(state, total_steps);
    let measure_ids = collect_measure_ids(state);

    let tonic_pc = state.params.mode.key % 12;
    let rule_key = RuleKey {
        tonic: RulePitchClass { pc: tonic_pc },
        mode: parse_mode(&state.params.mode.mode),
    };

    let alto_register = (
        state.params.register.melody_register_min.saturating_sub(12),
        state.params.register.melody_register_min.saturating_sub(4),
    );

    let initial_snapshot = AstSnapshot {
        key: rule_key,
        melody_register: alto_register,
        bass_register: (
            state.params.register.bass_register_min,
            state.params.register.bass_register_max,
        ),
        current_chord: chord_grid.first().cloned(),
        ..AstSnapshot::default()
    }
    .with_chord_grid(chord_grid.clone(), u8::try_from(beats_per_measure).unwrap_or(4));

    let mut params = state.params.clone();
    params.search.beam_width = (params.search.beam_width / 2).max(4);
    params.register.melody_register_min = alto_register.0;
    params.register.melody_register_max = alto_register.1;

    let melody_grid = collect_melody_pitches(state);

    let engine = BeamSearchEngine::from_bundle(aurora_rules::prototype_rule_set(), params.clone());
    let generator = AltoCandidateGenerator {
        chord_grid,
        measure_ids,
        melody_grid,
        beats_per_measure: u8::try_from(beats_per_measure).unwrap_or(4),
        alto_register,
    };
    let terminal = StepCountTerminal {
        max_steps: u32::try_from(total_steps).unwrap_or(64),
    };

    let result = engine
        .run_beam(SearchState::initial(initial_snapshot), &generator, &terminal)
        .map_err(|e| e.to_string())?;

    let pitches = &result.best_state.snapshot.alto_pitches;
    if pitches.len() != total_steps {
        return Err(format!(
            "counterpoint search produced {} notes, expected {total_steps}",
            pitches.len()
        ));
    }

    commit_inner_voice(
        state,
        alto_id,
        pitches,
        &result,
        created_at,
        beats_per_measure,
        PipelineStageId::Counterpoint,
        params.search.beam_width,
    );
    Ok(())
}

fn collect_chord_grid(state: &PipelineState, bars: usize) -> Vec<RuleChord> {
    let mut grid = Vec::with_capacity(bars);
    for measure in iter_measures(&state.composition) {
        let slot = measure.harmony_slots.first();
        grid.push(slot.map(|s| s.symbol.clone()).unwrap_or_else(|| RuleChord::simple(
            state.params.mode.key % 12,
            aurora_ast::ChordQuality::Major,
            "I",
        )));
    }
    while grid.len() < bars {
        if let Some(last) = grid.last().cloned() {
            grid.push(last);
        } else {
            break;
        }
    }
    grid
}

fn collect_measure_ids(state: &PipelineState) -> Vec<RuleNodeId> {
    iter_measures(&state.composition)
        .map(|m| RuleNodeId {
            index: m.id.index,
            generation: m.id.generation,
        })
        .collect()
}

fn commit_inner_voice(
    state: &mut PipelineState,
    voice_id: aurora_ast::VoiceId,
    pitches: &[RulePitch],
    result: &aurora_rules::SearchResult,
    created_at: &str,
    beats_per_measure: usize,
    stage: PipelineStageId,
    beam_width: u16,
) {
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
                id: NodeId::new(u64::try_from(20_000 + step).unwrap_or(20_000), 0),
                offset: aurora_ast::BeatOffset::new(beat as u32, 1),
                duration: WrittenDuration {
                    note_type: NoteType::Quarter,
                    dots: 0,
                    tuplet: None,
                },
                provenance: make_search_note_provenance(
                    stage,
                    created_at,
                    vec!["CP-001".into()],
                    rule_refs,
                    result.best_state.eval_score,
                    u32::try_from(step).unwrap_or(0),
                    result.best_state.beam_rank.unwrap_or(0),
                    beam_width,
                    &result.best_state.id.0.to_string(),
                    format!("counterpoint step {step}, MIDI {}", pitch.midi),
                ),
                visible: true,
            },
            pitch: Pitch::from_midi(pitch.midi),
            velocity: 70,
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
}

struct AltoCandidateGenerator {
    chord_grid: Vec<RuleChord>,
    measure_ids: Vec<RuleNodeId>,
    melody_grid: Vec<u8>,
    beats_per_measure: u8,
    alto_register: (u8, u8),
}

impl CandidateGenerator for AltoCandidateGenerator {
    fn voice_role(&self) -> aurora_rules::VoiceRole {
        aurora_rules::VoiceRole::Alto
    }

    fn generate(&self, state: &SearchState) -> Vec<CandidatePatch> {
        let step = state.step_index as usize;
        let beat = step % usize::from(self.beats_per_measure);
        let measure_idx = step / usize::from(self.beats_per_measure);
        let chord = self
            .chord_grid
            .get(step)
            .or_else(|| self.chord_grid.get(measure_idx))
            .cloned()
            .unwrap_or_else(|| RuleChord::simple(0, aurora_ast::ChordQuality::Major, "C"));

        let measure_id = self
            .measure_ids
            .get(measure_idx)
            .copied()
            .unwrap_or(RuleNodeId::new(1, 0));

        let (min_midi, max_midi) = self.alto_register;
        let mut candidates = Vec::new();

        for pc in chord.pitch_classes() {
            for octave in 3..=5 {
                let midi = octave * 12 + pc;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(measure_id, beat, midi, "alto_chord_tone"));
                }
            }
        }

        if let Some(prev) = state.snapshot.last_pitch(aurora_rules::VoiceRole::Alto) {
            for delta in [-2i16, -1, 1, 2] {
                let midi = (prev.midi as i16 + delta).clamp(0, 127) as u8;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(measure_id, beat, midi, "alto_stepwise"));
                }
            }
        } else if let Some(&melody_midi) = self.melody_grid.get(step) {
            let midi = (melody_midi as i16 - 7).clamp(min_midi as i16, max_midi as i16) as u8;
            candidates.push(make_patch(measure_id, beat, midi, "alto_start"));
        } else if let Some(melody) = state.snapshot.prev_melody_pitch() {
            let midi = (melody.midi as i16 - 7).clamp(min_midi as i16, max_midi as i16) as u8;
            candidates.push(make_patch(measure_id, beat, midi, "alto_start"));
        }

        let _ = beat;
        candidates.sort_by_key(|c| c.nodes_to_add.len());
        candidates.dedup_by(|a, b| patch_midi(a) == patch_midi(b));
        candidates
    }
}

fn patch_midi(patch: &CandidatePatch) -> Option<u8> {
    patch.nodes_to_add.iter().find_map(|e| {
        if let Event::Note(n) = e {
            Some(n.pitch.midi)
        } else {
            None
        }
    })
}

fn make_patch(measure_id: RuleNodeId, beat: usize, midi: u8, label: &str) -> CandidatePatch {
    let _ = beat;
    CandidatePatch::single_note(
        RuleVoiceId(1),
        measure_id,
        search_note(midi, RuleNodeId::new(u64::from(midi) + 100, 0)),
        label,
    )
}
