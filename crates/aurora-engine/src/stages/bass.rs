use aurora_ast::{
    Event, NoteEvent, NoteType, Pitch, PitchRole, PipelineStageId, RuleRef, TieSpec, TimedEventBase,
    WrittenDuration,
};
use aurora_core::NodeId;
use aurora_rules::{
    AstSnapshot, BeamSearchEngine, CandidateGenerator, CandidatePatch, ChordSymbol as RuleChord,
    KeySignature as RuleKey, Mode as RuleMode, NodeId as RuleNodeId, Pitch as RulePitch,
    PitchClass as RulePitchClass, SearchState, StepCountTerminal, VoiceId as RuleVoiceId,
    search_note,
};

use super::common::{
    bass_voice_id, iter_measures, iter_measures_mut, make_search_note_provenance, push_note,
};
use super::PipelineState;

/// Stage 9 — Bass: narrow beam search for bass line (root motion / walking).
pub fn generate_bass(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let voice_id = bass_voice_id(state);
    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let bar_count = super::total_bars(&state.params);
    let total_steps = bar_count as usize * beats_per_measure;

    let chord_grid = collect_chord_grid(state, bar_count as usize);
    let measure_ids = collect_measure_ids(state);

    let tonic_pc = state.params.mode.key % 12;
    let rule_key = RuleKey {
        tonic: RulePitchClass { pc: tonic_pc },
        mode: if state.params.mode.mode.to_lowercase().contains("minor") {
            RuleMode::NaturalMinor
        } else {
            RuleMode::Major
        },
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
    params.register.melody_register_min = bass_register.0;
    params.register.melody_register_max = bass_register.1;

    let engine = BeamSearchEngine::from_bundle(aurora_rules::prototype_rule_set(), params.clone());
    let generator = BassCandidateGenerator {
        chord_grid,
        measure_ids,
        beats_per_measure: u8::try_from(beats_per_measure).unwrap_or(4),
        bass_register,
        jazz_walk: state.style.jazz_harmony,
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

struct BassCandidateGenerator {
    chord_grid: Vec<RuleChord>,
    measure_ids: Vec<RuleNodeId>,
    beats_per_measure: u8,
    bass_register: (u8, u8),
    jazz_walk: bool,
}

impl CandidateGenerator for BassCandidateGenerator {
    fn generate(&self, state: &SearchState) -> Vec<CandidatePatch> {
        let step = state.step_index as usize;
        let beat = step % usize::from(self.beats_per_measure);
        let measure_idx = step / usize::from(self.beats_per_measure);
        let chord = self
            .chord_grid
            .get(measure_idx)
            .cloned()
            .unwrap_or_else(|| RuleChord::simple(0, aurora_ast::ChordQuality::Major, "C"));

        let measure_id = self
            .measure_ids
            .get(measure_idx)
            .copied()
            .unwrap_or(RuleNodeId::new(1, 0));

        let (min_midi, max_midi) = self.bass_register;
        let root_midi = 36 + chord.root.pc;
        let mut candidates = Vec::new();

        if beat == 0 || !self.jazz_walk {
            if root_midi >= min_midi && root_midi <= max_midi {
                candidates.push(make_patch(measure_id, beat, root_midi, "bass_root"));
            }
        }

        if self.jazz_walk {
            for delta in [0i16, 2, 4, 7] {
                let midi = (root_midi as i16 + delta).clamp(min_midi as i16, max_midi as i16) as u8;
                candidates.push(make_patch(measure_id, beat, midi, "bass_walk"));
            }
        }

        if let Some(prev) = state.snapshot.last_pitch(aurora_rules::VoiceRole::Bass) {
            for delta in [-2i16, -1, 1, 2, 5] {
                let midi = (prev.midi as i16 + delta).clamp(min_midi as i16, max_midi as i16) as u8;
                candidates.push(make_patch(measure_id, beat, midi, "bass_step"));
            }
        }

        for pc in chord.pitch_classes() {
            let midi = 36 + pc;
            if midi >= min_midi && midi <= max_midi {
                candidates.push(make_patch(measure_id, beat, midi, "bass_chord_tone"));
            }
        }

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
        RuleVoiceId(2),
        measure_id,
        search_note(midi, RuleNodeId::new(u64::from(midi) + 200, 0)),
        label,
    )
}
