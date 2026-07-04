use aurora_ast::{
    Event, NoteEvent, NoteType, Pitch, PitchRole, PipelineStageId, Provenance, ProvenanceAgent,
    ProvenanceSource, RuleRef, SearchContext, StateRef, TieSpec, TimedEventBase, WrittenDuration,
};
use aurora_core::NodeId;
use aurora_rules::{
    AstSnapshot, BeatStrengthKind, BeamSearchEngine, CandidateGenerator, CandidatePatch,
    ChordSymbol as RuleChord, KeySignature as RuleKey, Mode as RuleMode, NodeId as RuleNodeId,
    Pitch as RulePitch, PitchClass as RulePitchClass, SearchState, StepCountTerminal, search_note,
};

use super::PipelineState;

/// Stage 7 — Melody: beam search over quarter-note slots using aurora-rules scoring.
pub fn generate_melody(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
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

    let initial_snapshot = AstSnapshot {
        key: rule_key,
        melody_register: (
            state.params.register.melody_register_min,
            state.params.register.melody_register_max,
        ),
        current_chord: chord_grid.first().cloned(),
        ..AstSnapshot::default()
    }
    .with_chord_grid(chord_grid.clone(), u8::try_from(beats_per_measure).unwrap_or(4));

    let engine = BeamSearchEngine::from_bundle(aurora_rules::prototype_rule_set(), state.params.clone());
    let generator = MelodyCandidateGenerator {
        chord_grid,
        measure_ids,
        beats_per_measure: u8::try_from(beats_per_measure).unwrap_or(4),
        melody_register: (
            state.params.register.melody_register_min,
            state.params.register.melody_register_max,
        ),
    };
    let terminal = StepCountTerminal {
        max_steps: u32::try_from(total_steps).unwrap_or(64),
    };

    let result = engine
        .run_beam(SearchState::initial(initial_snapshot), &generator, &terminal)
        .map_err(|e| e.to_string())?;

    let pitches = &result.best_state.snapshot.melody_pitches;
    if pitches.len() != total_steps {
        return Err(format!(
            "melody search produced {} notes, expected {total_steps}",
            pitches.len()
        ));
    }

    commit_melody(state, pitches, &result, created_at, beats_per_measure);
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

fn iter_measures(comp: &aurora_ast::Composition) -> impl Iterator<Item = &aurora_ast::Measure> {
    comp.movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
        .flat_map(|p| &p.measures)
}

fn iter_measures_mut(
    comp: &mut aurora_ast::Composition,
) -> impl Iterator<Item = &mut aurora_ast::Measure> {
    comp.movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
        .flat_map(|s| &mut s.phrases)
        .flat_map(|p| &mut p.measures)
}

fn commit_melody(
    state: &mut PipelineState,
    pitches: &[RulePitch],
    result: &aurora_rules::SearchResult,
    created_at: &str,
    beats_per_measure: usize,
) {
    let beam_width = state.params.search.beam_width;
    for (step, pitch) in pitches.iter().enumerate() {
        let measure_idx = step / beats_per_measure;
        let beat = step % beats_per_measure;
        let top_rule = result
            .best_state
            .applied_rules
            .last()
            .map(|r| r.rule_id.as_str().to_string());

        let note = NoteEvent {
            base: TimedEventBase {
                id: NodeId::new(u64::try_from(10_000 + step).unwrap_or(10_000), 0),
                offset: aurora_ast::BeatOffset::new(beat as u32, 1),
                duration: WrittenDuration {
                    note_type: NoteType::Quarter,
                    dots: 0,
                    tuplet: None,
                },
                provenance: Provenance {
                    source: ProvenanceSource::Generated,
                    stage: Some(PipelineStageId::Melody),
                    rule_ids: top_rule.map(|id| vec![id]).unwrap_or_else(|| vec!["HARM-001".into()]),
                    rule_refs: result
                        .best_state
                        .applied_rules
                        .iter()
                        .map(|r| RuleRef {
                            id: r.rule_id.as_str().to_string(),
                            weight: None,
                            score: Some(r.score_delta),
                        })
                        .collect(),
                    eval_score: Some(result.best_state.eval_score),
                    search: Some(SearchContext {
                        step_index: u32::try_from(step).unwrap_or(0),
                        beam_rank: result.best_state.beam_rank.unwrap_or(0) as u16,
                        beam_width,
                        state_ref: StateRef {
                            id: result.best_state.id.0.to_string(),
                        },
                        accumulated_score: result.best_state.eval_score,
                    }),
                    parent: None,
                    created_at: created_at.into(),
                    agent: ProvenanceAgent::Engine {
                        stage: PipelineStageId::Melody,
                    },
                    parameters_hash: None,
                    explanation: Some(format!(
                        "beam search step {step}, MIDI {}",
                        pitch.midi
                    )),
                },
                visible: true,
            },
            pitch: Pitch::from_midi(pitch.midi),
            velocity: 80,
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
            if let Some(voice) = measure.voices.iter_mut().find(|v| v.voice_id.0 == 0) {
                voice.events.push(Event::Note(note));
            }
        }
    }
}

struct MelodyCandidateGenerator {
    chord_grid: Vec<RuleChord>,
    measure_ids: Vec<RuleNodeId>,
    beats_per_measure: u8,
    melody_register: (u8, u8),
}

impl CandidateGenerator for MelodyCandidateGenerator {
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

        let beat_strength = if beat == 0 || beat == 2 {
            BeatStrengthKind::Strong
        } else {
            BeatStrengthKind::Weak
        };

        let mut candidates = Vec::new();
        let (min_midi, max_midi) = self.melody_register;

        for pc in chord.pitch_classes() {
            for octave in 4..=6 {
                let midi = octave * 12 + pc;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(
                        measure_id,
                        beat,
                        midi,
                        beat_strength,
                        &chord,
                        "chord_tone",
                    ));
                }
            }
        }

        if let Some(prev) = state.snapshot.prev_melody_pitch() {
            for delta in [-2i16, -1, 1, 2] {
                let midi = (prev.midi as i16 + delta).clamp(0, 127) as u8;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(
                        measure_id,
                        beat,
                        midi,
                        beat_strength,
                        &chord,
                        "stepwise",
                    ));
                }
            }
        } else {
            let tonic_midi = 60 + chord.root.pc;
            if tonic_midi >= min_midi && tonic_midi <= max_midi {
                candidates.push(make_patch(
                    measure_id,
                    beat,
                    tonic_midi,
                    beat_strength,
                    &chord,
                    "start_tonic",
                ));
            }
        }

        candidates.sort_by_key(|c| c.nodes_to_add.len());
        candidates.dedup_by(|a, b| {
            patch_midi(a) == patch_midi(b)
        });
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

fn make_patch(
    measure_id: RuleNodeId,
    beat: usize,
    midi: u8,
    beat_strength: BeatStrengthKind,
    chord: &RuleChord,
    label: &str,
) -> CandidatePatch {
    let _ = (beat, beat_strength, chord);
    CandidatePatch::single_note(
        aurora_rules::VoiceId(0),
        measure_id,
        search_note(midi, RuleNodeId::new(u64::from(midi), 0)),
        format!("{label}_{midi}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::ParameterBundle;

    use crate::stages::{
        emotion_resolver::resolve_emotion, harmony::generate_harmony, structure::plan_structure,
        style_resolver::resolve_style, PipelineState,
    };

    fn pipeline_state(params: ParameterBundle) -> PipelineState {
        let style = resolve_style(&params);
        let (emotion, deltas) = resolve_emotion(&params);
        PipelineState::new(
            params,
            aurora_ast::Composition {
                id: NodeId::new(0, 0),
                schema_version: aurora_ast::AST_SCHEMA_VERSION,
                metadata: aurora_ast::CompositionMetadata {
                    title: String::new(),
                    subtitle: None,
                    composer: None,
                    lyricist: None,
                    copyright: None,
                    license: None,
                    created_at: String::new(),
                    modified_at: String::new(),
                    language: None,
                    parameters_used: ParameterBundle::default(),
                    emotion_profile: None,
                    provenance_root: aurora_ast::ProvenanceRoot {
                        session_id: String::new(),
                        generator_version: String::new(),
                        seed: None,
                        pipeline_config_hash: String::new(),
                        started_at: String::new(),
                        completed_at: None,
                    },
                    tags: vec![],
                    source: aurora_ast::CompositionSource::Generated,
                    layout: aurora_ast::ScoreLayout {
                        staff_spacing: 12.0,
                        measure_numbering: aurora_ast::MeasureNumberingStyle::EveryMeasure,
                        part_list_order: vec![],
                    },
                },
                global: aurora_ast::GlobalAttributes {
                    default_key: aurora_ast::KeySignature {
                        tonic: aurora_ast::PitchClass { pc: 0 },
                        mode: aurora_ast::Mode::Major,
                    },
                    default_meter: aurora_ast::TimeSignature {
                        beats: 4,
                        beat_type: 4,
                    },
                    tempo_map: aurora_ast::TempoMap {
                        default_bpm: 120.0,
                        segments: vec![],
                    },
                    key_map: aurora_ast::KeyMap {
                        default: aurora_ast::KeySignature {
                            tonic: aurora_ast::PitchClass { pc: 0 },
                            mode: aurora_ast::Mode::Major,
                        },
                        changes: vec![],
                    },
                    meter_map: aurora_ast::MeterMap {
                        default: aurora_ast::TimeSignature {
                            beats: 4,
                            beat_type: 4,
                        },
                        changes: vec![],
                    },
                    dynamics_baseline: aurora_ast::DynamicLevel::Mf,
                    pickup_measure: None,
                    display: aurora_ast::GlobalDisplayOptions {
                        show_metronome: true,
                        show_rehearsal_marks: true,
                        page_layout: aurora_ast::PageLayout {
                            page_width_mm: 210.0,
                            page_height_mm: 297.0,
                            margins_mm: aurora_ast::Margins {
                                top: 20.0,
                                bottom: 20.0,
                                left: 15.0,
                                right: 15.0,
                            },
                            system_distance: 10.0,
                        },
                    },
                },
                voice_registry: aurora_ast::VoiceRegistry {
                    voices: vec![],
                    groups: vec![],
                    default_layout: aurora_ast::VoiceLayoutId(0),
                },
                movements: vec![],
            },
            style,
            emotion,
            deltas,
        )
    }

    #[test]
    fn melody_beam_fills_all_quarter_slots() {
        let mut params = ParameterBundle::default();
        params.form.section_lengths = vec![2];
        params.search.beam_width = 8;
        let mut state = pipeline_state(params);
        plan_structure(&mut state, "2026-01-01").unwrap();
        generate_harmony(&mut state, "2026-01-01").unwrap();
        generate_melody(&mut state, "2026-01-01").unwrap();

        let notes: u32 = state
            .composition
            .movements
            .iter()
            .flat_map(|m| &m.sections)
            .flat_map(|s| &s.phrases)
            .flat_map(|p| &p.measures)
            .flat_map(|m| &m.voices)
            .flat_map(|v| &v.events)
            .filter(|e| matches!(e, Event::Note(_)))
            .count() as u32;
        assert_eq!(notes, 8);
    }
}
