use aurora_ast::{
    Event, NoteEvent, NoteType, Pitch, PitchRole, PipelineStageId, Provenance, ProvenanceAgent,
    ProvenanceSource, RuleRef, SearchContext, StateRef, TieSpec, TimedEventBase, WrittenDuration,
};
use aurora_core::NodeId;
use aurora_core::{derived_chord_tone_bias, derived_neighbor_tone_bias, derived_passing_tone_bias};
use aurora_rules::{
    AstSnapshot, BeatStrengthKind, BeamSearchEngine, CandidateGenerator, CandidatePatch,
    ChordSymbol as RuleChord, KeySignature as RuleKey, NodeId as RuleNodeId,
    Pitch as RulePitch, PitchClass as RulePitchClass, SearchState, StepCountTerminal, search_note,
    scale::mode_scale_pcs,
};

use crate::motif::{Motif, MotifCursor, MotifDur};

use crate::progression::parse_mode;

use super::PipelineState;

/// Stage 7 — Melody: beam search over quarter-note slots using aurora-rules scoring.
pub fn generate_melody(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let bar_count = super::total_bars(&state.params);
    let total_steps = bar_count as usize * beats_per_measure;

    let chord_grid = super::common::collect_per_beat_chord_grid(state, total_steps);
    let measure_ids = collect_measure_ids(state);

    let tonic_pc = state.params.mode.key % 12;
    let ast_mode = parse_mode(&state.params.mode.mode);
    let rule_key = RuleKey {
        tonic: RulePitchClass { pc: tonic_pc },
        mode: ast_mode,
    };

    let phrase_length_beats =
        usize::from(state.params.form.phrase_length.max(1)) * beats_per_measure;
    let climax_step =
        ((total_steps as f32) * state.params.melody.climax_ratio.clamp(0.3, 0.9)) as usize;

    let initial_snapshot = AstSnapshot {
        key: rule_key.clone(),
        melody_register: (
            state.params.register.melody_register_min,
            state.params.register.melody_register_max,
        ),
        current_chord: chord_grid.first().cloned(),
        phrase_length_beats,
        total_melody_steps: total_steps,
        climax_step,
        ..AstSnapshot::default()
    }
    .with_chord_grid(chord_grid.clone(), u8::try_from(beats_per_measure).unwrap_or(4));

    let t = state.params.melody.tonal_conservatism;
    let scale_pcs = mode_scale_pcs(&rule_key);

    let generator = MelodyCandidateGenerator {
        chord_grid,
        measure_ids,
        beats_per_measure: u8::try_from(beats_per_measure).unwrap_or(4),
        melody_register: (
            state.params.register.melody_register_min,
            state.params.register.melody_register_max,
        ),
        scale_pcs,
        tonic_pc,
        motifs: state.motifs.clone(),
        phrase_plans: state.phrase_motif_plans.clone(),
        motif_weight: state.params.melody.motif_weight,
        leap_limit: state.params.melody.leap_limit_semitones,
        chord_tone_bias: derived_chord_tone_bias(t),
        neighbor_bias: derived_neighbor_tone_bias(t),
        passing_bias: derived_passing_tone_bias(t),
        tonal_conservatism: t,
        syncopation: state.params.rhythm.syncopation,
        double_stop_enabled: state.params.melody.double_stop_enabled
            || state.params.texture.homophony_polyphony_balance > 0.75,
        phrase_length_beats,
        climax_step,
        total_steps,
        search_seed: state.params.search.seed.unwrap_or(42),
    };

    let motif_expected_by_step = build_motif_expected_grid(total_steps, &generator);
    let initial_snapshot = initial_snapshot.with_motif_plan(motif_expected_by_step);

    let engine = BeamSearchEngine::from_bundle(aurora_rules::prototype_rule_set(), state.params.clone());
    let terminal = StepCountTerminal {
        max_steps: u32::try_from(total_steps).unwrap_or(64),
    };

    let result = match engine.run_beam(SearchState::initial(initial_snapshot.clone()), &generator, &terminal) {
        Ok(ok) => ok,
        Err(_) => {
            let mut relaxed_params = state.params.clone();
            relaxed_params.search.beam_width = relaxed_params.search.beam_width.max(24);
            relaxed_params.melody.tonal_conservatism =
                (relaxed_params.melody.tonal_conservatism - 0.12).max(0.45);
            let relaxed_t = relaxed_params.melody.tonal_conservatism;
            let mut relaxed_generator = generator.clone();
            relaxed_generator.chord_tone_bias = derived_chord_tone_bias(relaxed_t);
            relaxed_generator.neighbor_bias = derived_neighbor_tone_bias(relaxed_t);
            relaxed_generator.passing_bias = derived_passing_tone_bias(relaxed_t);
            relaxed_generator.tonal_conservatism = relaxed_t;
            relaxed_generator.motif_weight = relaxed_generator.motif_weight.min(0.45);
            let relaxed_engine = BeamSearchEngine::from_bundle(
                aurora_rules::prototype_rule_set(),
                relaxed_params,
            );
            relaxed_engine
                .run_beam(SearchState::initial(initial_snapshot), &relaxed_generator, &terminal)
                .map_err(|e| e.to_string())?
        }
    };

    let pitches = &result.best_state.snapshot.melody_pitches;
    if pitches.len() != total_steps {
        return Err(format!(
            "melody search produced {} notes, expected {total_steps}",
            pitches.len()
        ));
    }

    commit_melody(
        state,
        pitches,
        &result,
        created_at,
        beats_per_measure,
        &generator,
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
    generator: &MelodyCandidateGenerator,
) {
    let beam_width = state.params.search.beam_width;
    let mut pitch_run = 0u32;
    let mut prev_commit_midi: Option<u8> = None;
    for (step, pitch) in pitches.iter().enumerate() {
        let measure_idx = step / beats_per_measure;
        let beat = step % beats_per_measure;
        let motif_dur = generator
            .motif_rhythm_at(step)
            .unwrap_or_else(|| generator.default_rhythm_at(step));
        if Some(pitch.midi) == prev_commit_midi {
            pitch_run += 1;
        } else {
            pitch_run = 0;
        }
        let motif_dur = adjust_motif_dur_for_repeat(motif_dur, pitch_run);
        let prev_midi = pitches.get(step.saturating_sub(1)).map(|p| p.midi);
        let conservative = generator.tonal_conservatism >= 0.55;
        let sub_notes = rhythm_subdivisions(motif_dur, pitch.midi, prev_midi, conservative);
        prev_commit_midi = Some(pitch.midi);

        let top_rule = result
            .best_state
            .applied_rules
            .last()
            .map(|r| r.rule_id.as_str().to_string());

        for (sub_idx, sub) in sub_notes.iter().enumerate() {
            let note = NoteEvent {
                base: TimedEventBase {
                    id: NodeId::new(
                        u64::try_from(10_000 + step * 4 + sub_idx).unwrap_or(10_000),
                        0,
                    ),
                    offset: aurora_ast::BeatOffset::new(
                        (beat as u32 * 4 + (sub.offset_frac * 4.0) as u32).max(0),
                        4,
                    ),
                    duration: WrittenDuration {
                        note_type: sub.note_type,
                        dots: sub.dots,
                        tuplet: None,
                    },
                    provenance: Provenance {
                        source: ProvenanceSource::Generated,
                        stage: Some(PipelineStageId::Melody),
                        rule_ids: top_rule
                            .clone()
                            .map(|id| vec![id])
                            .unwrap_or_else(|| vec!["HARM-001".into()]),
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
                            "beam step {step} sub {sub_idx}, MIDI {}",
                            sub.midi
                        )),
                    },
                    visible: true,
                },
                pitch: Pitch::from_midi(sub.midi),
                velocity: sub.velocity,
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

    // Lengthen the final melody note for a conclusive, closed ending
    let last_step = pitches.len().saturating_sub(1);
    if last_step < pitches.len() {
        let last_measure_idx = last_step / beats_per_measure;
        if let Some(measure) = iter_measures_mut(&mut state.composition).nth(last_measure_idx) {
            if let Some(voice) = measure.voices.iter_mut().find(|v| v.voice_id.0 == 0) {
                if let Some(Event::Note(note)) = voice.events.last_mut() {
                    note.base.duration.note_type = NoteType::Half;
                    note.base.duration.dots = 1;
                    note.velocity = note.velocity.saturating_add(8).min(100);
                }
            }
        }
    }
}

fn adjust_motif_dur_for_repeat(dur: MotifDur, repeat_count: u32) -> MotifDur {
    if repeat_count == 0 {
        return dur;
    }
    match dur {
        MotifDur::TwoEighths | MotifDur::SyncopatedEighth => MotifDur::RestThenEighth,
        MotifDur::Quarter => MotifDur::DottedQuarter,
        _ => dur,
    }
}

struct SubNote {
    offset_frac: f32,
    note_type: NoteType,
    dots: u8,
    midi: u8,
    velocity: u8,
}

#[derive(Clone, Copy, Debug)]
enum CadenceFormula {
    Authentic,      // 7→1  (leading tone up to tonic)
    HalfCadence,    // ends on V (dominant)
    PlagalImplied,  // 4→1
    Deceptive,      // 7→6
}

impl CadenceFormula {
    fn penultimate_pcs(&self, tonic_pc: u8) -> Vec<u8> {
        match self {
            CadenceFormula::Authentic => {
                vec![(tonic_pc + 11) % 12, (tonic_pc + 2) % 12]
            }
            CadenceFormula::HalfCadence => {
                vec![(tonic_pc + 7) % 12]
            }
            CadenceFormula::PlagalImplied => {
                vec![(tonic_pc + 5) % 12]
            }
            CadenceFormula::Deceptive => {
                vec![(tonic_pc + 11) % 12, (tonic_pc + 9) % 12]
            }
        }
    }

    fn final_pcs(&self, tonic_pc: u8) -> Vec<u8> {
        match self {
            CadenceFormula::Authentic | CadenceFormula::PlagalImplied => {
                vec![tonic_pc]
            }
            CadenceFormula::HalfCadence => {
                vec![(tonic_pc + 7) % 12]
            }
            CadenceFormula::Deceptive => {
                vec![(tonic_pc + 9) % 12]
            }
        }
    }
}

fn build_motif_expected_grid(
    total_steps: usize,
    generator: &MelodyCandidateGenerator,
) -> Vec<Option<u8>> {
    (0..total_steps)
        .map(|step| {
            generator
                .motif_context(step)
                .map(|(cursor, motif)| motif.pitch_at(cursor.cell_index, cursor.base_midi))
        })
        .collect()
}

fn fill_passing_tone(prev: u8, target: u8, conservative: bool) -> Option<u8> {
    let diff = target as i16 - prev as i16;
    let direction = diff.signum();
    let abs_diff = diff.unsigned_abs();

    match abs_diff {
        2 => {
            // Whole step: insert the middle note (diatonic passing)
            Some((prev as i16 + direction) as u8)
        }
        1 => {
            // Half step: already stepwise, no passing needed
            None
        }
        3 | 4 | 5 if !conservative => {
            // Third to fourth: insert one passing tone
            Some((prev as i16 + direction * 2) as u8)
        }
        _ => {
            // Larger leaps: do NOT insert passing tones
            None
        }
    }
}

fn single_quarter_note(midi: u8) -> Vec<SubNote> {
    vec![SubNote {
        offset_frac: 0.0,
        note_type: NoteType::Quarter,
        dots: 0,
        midi,
        velocity: 82,
    }]
}

fn two_eighths_with_passing(midi: u8, _prev_midi: Option<u8>, passing: Option<u8>, _conservative: bool) -> Vec<SubNote> {
    let second_midi = passing.unwrap_or(midi);
    vec![
        SubNote {
            offset_frac: 0.0,
            note_type: NoteType::Eighth,
            dots: 0,
            midi,
            velocity: 84,
        },
        SubNote {
            offset_frac: 0.5,
            note_type: NoteType::Eighth,
            dots: 0,
            midi: second_midi,
            velocity: if second_midi == midi { 70 } else { 72 },
        },
    ]
}

fn syncopated_with_passing(midi: u8, passing: Option<u8>) -> Vec<SubNote> {
    let mut notes = vec![SubNote {
        offset_frac: 0.25,
        note_type: NoteType::Eighth,
        dots: 0,
        midi,
        velocity: 80,
    }];
    if let Some(p) = passing {
        notes.push(SubNote {
            offset_frac: 0.75,
            note_type: NoteType::Sixteenth,
            dots: 0,
            midi: p,
            velocity: 68,
        });
    }
    notes
}

fn single_dotted_quarter(midi: u8) -> Vec<SubNote> {
    vec![SubNote {
        offset_frac: 0.0,
        note_type: NoteType::Quarter,
        dots: 1,
        midi,
        velocity: 86,
    }]
}

fn rest_then_eighth(midi: u8) -> Vec<SubNote> {
    vec![SubNote {
        offset_frac: 0.5,
        note_type: NoteType::Eighth,
        dots: 0,
        midi,
        velocity: 76,
    }]
}

fn rhythm_subdivisions(
    dur: MotifDur,
    midi: u8,
    prev_midi: Option<u8>,
    conservative: bool,
) -> Vec<SubNote> {
    let passing = prev_midi.and_then(|prev| {
        if prev == midi {
            return None;
        }
        fill_passing_tone(prev, midi, conservative)
    });

    match dur {
        MotifDur::Quarter => single_quarter_note(midi),
        MotifDur::TwoEighths => two_eighths_with_passing(midi, prev_midi, passing, conservative),
        MotifDur::SyncopatedEighth => syncopated_with_passing(midi, passing),
        MotifDur::DottedQuarter => single_dotted_quarter(midi),
        MotifDur::RestThenEighth => rest_then_eighth(midi),
    }
}

#[derive(Clone)]
struct MelodyCandidateGenerator {
    chord_grid: Vec<RuleChord>,
    measure_ids: Vec<RuleNodeId>,
    beats_per_measure: u8,
    melody_register: (u8, u8),
    scale_pcs: Vec<u8>,
    tonic_pc: u8,
    motifs: std::collections::HashMap<String, Motif>,
    phrase_plans: Vec<super::PhraseMotifPlan>,
    motif_weight: f32,
    leap_limit: u8,
    chord_tone_bias: f32,
    neighbor_bias: f32,
    passing_bias: f32,
    tonal_conservatism: f32,
    syncopation: f32,
    double_stop_enabled: bool,
    phrase_length_beats: usize,
    climax_step: usize,
    total_steps: usize,
    search_seed: u64,
}

impl MelodyCandidateGenerator {
    fn consonant_weak_nct(&self, prev: u8, midi: u8, chord: &RuleChord) -> bool {
        let pc = midi % 12;
        if !self.scale_pcs.contains(&pc) {
            return false;
        }
        let step = (midi as i16 - prev as i16).unsigned_abs();
        if step == 0 || step > 2 {
            return false;
        }
        if chord.pitch_classes().contains(&pc) {
            return true;
        }
        chord
            .pitch_classes()
            .iter()
            .any(|&ct| (pc as i16 - ct as i16).rem_euclid(12) <= 2 || (ct as i16 - pc as i16).rem_euclid(12) <= 2)
    }

    fn phrase_anchor_at(&self, step: usize) -> u8 {
        for plan in &self.phrase_plans {
            let end = plan.phrase_start_beat + plan.region_beats.max(1);
            if step >= plan.phrase_start_beat && step < end {
                return plan.base_midi;
            }
        }
        for plan in &self.phrase_plans {
            if step >= plan.phrase_start_beat {
                return plan.base_midi;
            }
        }
        60
    }

    fn direction_streak(pitches: &[RulePitch], prev_midi: u8) -> (i8, u32) {
        if pitches.is_empty() {
            return (0, 0);
        }
        let last_sign = (prev_midi as i16 - pitches[pitches.len() - 1].midi as i16).signum() as i8;
        if last_sign == 0 {
            return (0, 0);
        }
        let mut run = 1u32;
        for w in pitches.windows(2).rev() {
            let s = (w[1].midi as i16 - w[0].midi as i16).signum() as i8;
            if s == last_sign {
                run += 1;
            } else {
                break;
            }
        }
        (last_sign, run)
    }

    fn expected_motif_pitch(&self, global_step: usize) -> Option<u8> {
        self.motif_context(global_step)
            .map(|(cursor, motif)| cursor.expected_pitch(&motif))
    }

    fn motif_context(&self, global_step: usize) -> Option<(MotifCursor, Motif)> {
        for plan in &self.phrase_plans {
            let phrase_end = plan.phrase_start_beat + plan.region_beats;
            if global_step >= plan.phrase_start_beat && global_step < phrase_end {
                let motif = self.motifs.get(&plan.motif_id)?.clone();
                let beat_in_phrase = global_step - plan.phrase_start_beat;
                let cell_index = beat_in_phrase % motif.len().max(1);
                let cursor = MotifCursor {
                    motif_id: Some(plan.motif_id.clone()),
                    cell_index,
                    base_midi: plan.base_midi,
                    active: true,
                    region_beats: plan.region_beats,
                };
                return Some((cursor, motif));
            }
        }
        None
    }

    fn motif_rhythm_at(&self, global_step: usize) -> Option<MotifDur> {
        for plan in &self.phrase_plans {
            let phrase_end = plan.phrase_start_beat + plan.region_beats;
            if global_step >= plan.phrase_start_beat && global_step < phrase_end {
                let motif = self.motifs.get(&plan.motif_id)?;
                let beat_in_phrase = global_step - plan.phrase_start_beat;
                let cell_index = beat_in_phrase % motif.len().max(1);
                return Some(motif.rhythm_at(cell_index));
            }
        }
        None
    }

    fn default_rhythm_at(&self, step: usize) -> MotifDur {
        const ROTATION: [MotifDur; 6] = [
            MotifDur::Quarter,
            MotifDur::TwoEighths,
            MotifDur::SyncopatedEighth,
            MotifDur::DottedQuarter,
            MotifDur::TwoEighths,
            MotifDur::RestThenEighth,
        ];
        ROTATION[(step + self.search_seed as usize) % ROTATION.len()]
    }
}

impl CandidateGenerator for MelodyCandidateGenerator {
    fn voice_role(&self) -> aurora_rules::VoiceRole {
        aurora_rules::VoiceRole::Melody
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

        let beat_strength = if beat == 0 || beat == 2 {
            BeatStrengthKind::Strong
        } else {
            BeatStrengthKind::Weak
        };

        let mut candidates = Vec::new();
        let (min_midi, max_midi) = self.melody_register;
        let prev_midi = state.snapshot.prev_melody_pitch().map(|p| p.midi);

        let repeats_prev = prev_midi.is_some_and(|p| {
            let recent: Vec<_> = state.snapshot.melody_pitches.iter().rev().take(2).collect();
            recent.len() >= 2 && recent.iter().all(|x| x.midi == p)
        });

        let phrase_start = step == 0
            || (self.phrase_length_beats > 0 && step % self.phrase_length_beats == 0);
        let conservative = self.tonal_conservatism >= 0.6;
        let anchor_midi = self.phrase_anchor_at(step);
        let is_final_step = step + 1 >= self.total_steps;
        let pos_in_phrase = if self.phrase_length_beats > 0 {
            step % self.phrase_length_beats
        } else {
            0
        };
        let is_phrase_end_step =
            self.phrase_length_beats > 0 && (step + 1) % self.phrase_length_beats == 0;
        let in_closure_zone = is_final_step
            || is_phrase_end_step
            || step + 4 >= self.total_steps
            || (self.phrase_length_beats > 2
                && pos_in_phrase + 2 >= self.phrase_length_beats);
        let leading_pc = (self.tonic_pc + 11) % 12;
        let tonic_mid = 60 + self.tonic_pc;

        // Return-home / anchor restatement (motif development without endless drift)
        for offset in [-12i16, 0, 12] {
            let midi = (anchor_midi as i16 + offset).clamp(0, 127) as u8;
            if midi >= min_midi && midi <= max_midi {
                candidates.push(make_patch(
                    measure_id,
                    beat,
                    midi,
                    beat_strength,
                    &chord,
                    "return_home",
                ));
            }
        }
        if let Some((cursor, motif)) = self.motif_context(step) {
            let motif_start = motif.pitch_at(0, cursor.base_midi);
            for offset in [-12i16, 0, 12] {
                let midi = (motif_start as i16 + offset).clamp(0, 127) as u8;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(
                        measure_id,
                        beat,
                        midi,
                        beat_strength,
                        &chord,
                        "motif_restate",
                    ));
                }
            }
        }

        // Motif-realization candidates (priority when in motif region)
        if self.motif_weight > 0.0 {
            if let Some((cursor, motif)) = self.motif_context(step) {
                let expected = cursor.expected_pitch(&motif);
                for octave_offset in [-12i16, 0, 12] {
                    let midi = (expected as i16 + octave_offset).clamp(0, 127) as u8;
                    if midi >= min_midi && midi <= max_midi && !(repeats_prev && midi == prev_midi.unwrap_or(0))
                    {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "motif_realization",
                        ));
                    }
                }
                for delta in [-2i16, -1, 1, 2] {
                    let midi = (expected as i16 + delta).clamp(0, 127) as u8;
                    if midi >= min_midi && midi <= max_midi {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "motif_variant",
                        ));
                    }
                }
            }
        }

        if self.chord_tone_bias > 0.2 {
            for pc in chord.voicing_pcs() {
                for octave in 3..=7 {
                    let midi = octave * 12 + pc;
                    if midi >= min_midi && midi <= max_midi {
                        if repeats_prev && Some(midi) == prev_midi && beat_strength != BeatStrengthKind::Strong
                        {
                            continue;
                        }
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
        }

        // Scale-degree pool — diatonic color on weak beats when conservative
        if beat_strength != BeatStrengthKind::Strong || self.tonal_conservatism < 0.55 {
            for &pc in &self.scale_pcs {
                for octave in 3..=7 {
                    let midi = octave * 12 + pc;
                    if midi >= min_midi && midi <= max_midi {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "scale_degree",
                        ));
                    }
                }
            }
        }

        // Diatonic stepwise motion through the mode
        if let Some(prev) = prev_midi {
            let prev_pc = prev % 12;
            if let Some(idx) = self.scale_pcs.iter().position(|&p| p == prev_pc) {
                let len = self.scale_pcs.len();
                for delta in [-1i16, 1, -2, 2] {
                    let ni = (idx as i16 + delta).rem_euclid(len as i16) as usize;
                    let pc = self.scale_pcs[ni];
                    for octave in 3..=7 {
                        let midi = octave * 12 + pc;
                        if midi >= min_midi && midi <= max_midi {
                            candidates.push(make_patch(
                                measure_id,
                                beat,
                                midi,
                                beat_strength,
                                &chord,
                                "diatonic_step",
                            ));
                        }
                    }
                }
            }
        }

        // Serial intervals from previous pitch — restricted when tonal
        if let Some(prev) = prev_midi {
            let deltas: &[i16] = if self.tonal_conservatism >= 0.65 {
                &[1, 2, 3, 4, 5, 7, 12, -1, -2, -3, -4, -5, -7, -12]
            } else if self.tonal_conservatism >= 0.5 {
                &[1, 2, 3, 4, 5, 7, 9, 12, -1, -2, -3, -4, -5, -7, -9, -12]
            } else {
                &[
                    2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, -2, -3, -4, -5, -6, -7, -8, -9, -10,
                    -12, -14,
                ]
            };
            for &delta in deltas {
                let midi = (prev as i16 + delta).clamp(0, 127) as u8;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(
                        measure_id,
                        beat,
                        midi,
                        beat_strength,
                        &chord,
                        "interval_from_prev",
                    ));
                }
            }
            if self.tonal_conservatism < 0.65 {
                for delta in [2i16, 3, 4, 5, 6, 7, 8, 9, 10, 12, -2, -3, -4, -5, -6, -7, -8, -9, -10, -12]
                {
                    let midi = (anchor_midi as i16 + delta).clamp(min_midi as i16, max_midi as i16) as u8;
                    if midi >= min_midi && midi <= max_midi {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "interval_from_anchor",
                        ));
                    }
                }
            }
        }

        // Closure zone: tonic + leading-tone approach candidates
        if in_closure_zone {
            for pc in [self.tonic_pc, leading_pc] {
                for octave in 4..=7 {
                    let midi = octave * 12 + pc;
                    if midi >= min_midi && midi <= max_midi {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "closure_tonic",
                        ));
                    }
                }
            }
            for offset in [-12i16, 0, 12] {
                let midi =
                    (tonic_mid as i16 + offset).clamp(min_midi as i16, max_midi as i16) as u8;
                candidates.push(make_patch(
                    measure_id,
                    beat,
                    midi,
                    beat_strength,
                    &chord,
                    "closure_home",
                ));
            }
        }

        if let Some(prev) = prev_midi {
            let neighbor_range = if self.neighbor_bias > 0.2 { 2 } else { 1 };
            for delta in -neighbor_range..=neighbor_range {
                if delta == 0 {
                    continue;
                }
                let midi = (prev as i16 + delta).clamp(0, 127) as u8;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(
                        measure_id,
                        beat,
                        midi,
                        beat_strength,
                        &chord,
                        "neighbor_tone",
                    ));
                }
            }

            if self.passing_bias > 0.15
                && beat_strength == BeatStrengthKind::Weak
                && !(conservative && self.tonal_conservatism >= 0.55)
            {
                for delta in [-2i16, -1, 1, 2] {
                    let midi = (prev as i16 + delta).clamp(0, 127) as u8;
                    if midi >= min_midi
                        && midi <= max_midi
                        && self.consonant_weak_nct(prev, midi, &chord)
                    {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "passing_tone",
                        ));
                    }
                }
            }

            if self.syncopation > 0.35
                && beat_strength == BeatStrengthKind::Weak
                && self.tonal_conservatism < 0.75
            {
                for delta in [-5i16, 5, -7, 7] {
                    let midi = (prev as i16 + delta).clamp(0, 127) as u8;
                    if midi >= min_midi && midi <= max_midi {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "syncopation_leap",
                        ));
                    }
                }
            }
            // Leap compensation: after large leap, prefer stepwise
            if let Some(last) = state.snapshot.melody_pitches.last() {
                let leap = (last.midi as i16 - prev as i16).unsigned_abs();
                if leap > u16::from(self.leap_limit) {
                    candidates.retain(|c| {
                        patch_midi(c)
                            .map(|m| (m as i16 - prev as i16).unsigned_abs() <= 2)
                            .unwrap_or(false)
                    });
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
        candidates.dedup_by(|a, b| patch_midi(a) == patch_midi(b));

        // P6: double-stop candidates (parallel thirds/sixths)
        if self.double_stop_enabled && beat_strength == BeatStrengthKind::Weak {
            if let Some(top) = prev_midi {
                for delta in [3i16, 4, 8, 9] {
                    let bot = (top as i16 - delta).clamp(0, 127) as u8;
                    if bot >= min_midi && bot <= max_midi {
                        let bot_pc = bot % 12;
                        if chord.pitch_classes().contains(&bot_pc) {
                            candidates.push(make_patch(
                                measure_id,
                                beat,
                                bot,
                                beat_strength,
                                &chord,
                                "double_stop_lower",
                            ));
                        }
                    }
                }
            }
        }

        candidates.sort_by_key(|c| c.nodes_to_add.len());
        candidates.dedup_by(|a, b| patch_midi(a) == patch_midi(b));

        candidates.sort_by_key(|c| c.nodes_to_add.len());
        candidates.dedup_by(|a, b| patch_midi(a) == patch_midi(b));

        if let Some(prev) = prev_midi {
            let (dir_sign, streak) =
                Self::direction_streak(&state.snapshot.melody_pitches, prev);
            if streak >= 3 && dir_sign != 0 {
                candidates.retain(|c| {
                    patch_midi(c)
                        .map(|m| {
                            let d = (m as i16 - prev as i16).signum();
                            d != i16::from(dir_sign) || d == 0
                        })
                        .unwrap_or(true)
                });
            }
            // Post-climax: encourage descent or return to anchor
            if step >= self.climax_step {
                for offset in [-12i16, 0, 12] {
                    let midi =
                        (anchor_midi as i16 + offset).clamp(min_midi as i16, max_midi as i16) as u8;
                    candidates.push(make_patch(
                        measure_id,
                        beat,
                        midi,
                        beat_strength,
                        &chord,
                        "post_climax_home",
                    ));
                }
            }
        }

        if conservative && phrase_start {
            let scale_set: std::collections::HashSet<u8> =
                self.scale_pcs.iter().copied().collect();
            candidates.retain(|c| {
                patch_midi(c)
                    .map(|m| scale_set.contains(&(m % 12)))
                    .unwrap_or(false)
            });
        }

        // Strong beats: chord tones (or planned motif) when tonal
        if beat_strength == BeatStrengthKind::Strong && self.tonal_conservatism >= 0.55 {
            let motif_exp = self.expected_motif_pitch(step);
            let chord_pcs: std::collections::HashSet<u8> =
                chord.voicing_pcs().into_iter().collect();
            candidates.retain(|c| {
                patch_midi(c)
                    .map(|m| {
                        chord_pcs.contains(&(m % 12))
                            || motif_exp
                                .is_some_and(|e| (m as i16 - e as i16).unsigned_abs() <= 2)
                    })
                    .unwrap_or(false)
            });
            }

        // Motif region: stay near planned contour + chord tones
        // NOTE: Disabled — the motif_region filter was too aggressive,
        // excluding tonic and other critical cadential pitches even
        // at non-closure steps (e.g., when chord is G major and motif
        // expects a pitch far from C). Melodic contour is rewarded by
        // soft scoring rules instead.
        if false && self.motif_weight > 0.55 && !in_closure_zone {
            if let Some(expected) = self.expected_motif_pitch(step) {
                let mut allowed: std::collections::HashSet<u8> =
                    chord.voicing_pcs().into_iter().collect();
                // Include all diatonic scale degrees so the melody can move freely
                for &pc in &self.scale_pcs {
                    allowed.insert(pc);
                }
                // Always include the key tonic so cadential pitches survive
                allowed.insert(self.tonic_pc);
                for delta in -3i16..=3 {
                    let midi = (expected as i16 + delta).clamp(0, 127) as u8;
                    allowed.insert(midi % 12);
                }
                candidates.retain(|c| {
                    patch_midi(c)
                        .map(|m| allowed.contains(&(m % 12)))
                        .unwrap_or(false)
                });
            }
        }

        // Phrase-wide diatonic guard when conservative (allow current chord tones + leading tone)
        if self.tonal_conservatism >= 0.55 && !is_final_step && !in_closure_zone {
            let mut allowed: std::collections::HashSet<u8> =
                self.scale_pcs.iter().copied().collect();
            allowed.insert(leading_pc);
            for pc in chord.voicing_pcs() {
                allowed.insert(pc);
            }
            candidates.retain(|c| {
                patch_midi(c)
                    .map(|m| allowed.contains(&(m % 12)))
                    .unwrap_or(false)
            });
        }

        // Cadence-aware closure zone filtering
        if is_final_step {
            let final_pcs = CadenceFormula::Authentic.final_pcs(self.tonic_pc);
            candidates.retain(|c| {
                patch_midi(c)
                    .map(|m| final_pcs.contains(&(m % 12)))
                    .unwrap_or(false)
            });
            if candidates.is_empty() {
                for octave in 4..=7 {
                    let midi = octave * 12 + self.tonic_pc;
                    if midi >= min_midi && midi <= max_midi {
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "final_tonic",
                        ));
                    }
                }
            }
        } else if is_phrase_end_step {
            // Penultimate step: try to force leading tone or approach tone to tonic
            let formula = CadenceFormula::Authentic;
            let pen_pcs = formula.penultimate_pcs(self.tonic_pc);
            let had_candidates = !candidates.is_empty();
            candidates.retain(|c| {
                patch_midi(c)
                    .map(|m| pen_pcs.contains(&(m % 12)))
                    .unwrap_or(false)
            });
            // If cadence filtering emptied the pool (e.g. narrow register or
            // earlier-stage constraints), fall back to a wider closure-friendly set
            if candidates.is_empty() && had_candidates {
                let mut allowed: std::collections::HashSet<u8> =
                    self.scale_pcs.iter().copied().collect();
                allowed.insert(self.tonic_pc);
                allowed.insert(leading_pc);
                for &pc in &pen_pcs {
                    allowed.insert(pc);
                }
                for pc in allowed {
                    for octave in 4..=7 {
                        let midi = octave * 12 + pc;
                        if midi >= min_midi && midi <= max_midi {
                            candidates.push(make_patch(
                                measure_id,
                                beat,
                                midi,
                                beat_strength,
                                &chord,
                                "closure_scale",
                            ));
                        }
                    }
                }
            } else if candidates.is_empty() {
                // No original candidates existed; generate formula pitches directly
                for pc in pen_pcs {
                    for octave in 4..=7 {
                        let midi = octave * 12 + pc;
                        if midi >= min_midi && midi <= max_midi {
                            candidates.push(make_patch(
                                measure_id,
                                beat,
                                midi,
                                beat_strength,
                                &chord,
                                "cadence_approach",
                            ));
                        }
                    }
                }
            }
        } else if in_closure_zone {
            let mut allowed: std::collections::HashSet<u8> =
                self.scale_pcs.iter().copied().collect();
            allowed.insert(self.tonic_pc);
            allowed.insert(leading_pc);
            candidates.retain(|c| {
                patch_midi(c)
                    .map(|m| allowed.contains(&(m % 12)))
                    .unwrap_or(false)
            });
        }

        // Global anti-repeat: allow planned motif repetition
        if let Some(prev) = prev_midi {
            let motif_exp = self.expected_motif_pitch(step);
            let allow_repeat = (beat_strength == BeatStrengthKind::Strong && step == 0)
                || motif_exp.is_some_and(|e| e == prev);
            if !allow_repeat && !is_final_step {
                let had = candidates.len();
                candidates.retain(|c| patch_midi(c) != Some(prev));
                if candidates.is_empty() && had > 0 {
                    // Stepwise escape only
                    for delta in [-2i16, -1, 1, 2, 3, -3] {
                        let midi = (prev as i16 + delta).clamp(min_midi as i16, max_midi as i16) as u8;
                        candidates.push(make_patch(
                            measure_id,
                            beat,
                            midi,
                            beat_strength,
                            &chord,
                            "anti_repeat_escape",
                        ));
                    }
                }
            }
        }

        if candidates.is_empty() {
            let fallback_midi = if is_final_step || is_phrase_end_step {
                tonic_mid.clamp(min_midi, max_midi)
            } else {
                prev_midi.unwrap_or_else(|| {
                    let tonic = 60 + chord.root.pc;
                    if tonic >= min_midi && tonic <= max_midi {
                        tonic
                    } else {
                        min_midi + (chord.root.pc % 12)
                    }
                })
            };
            if fallback_midi >= min_midi && fallback_midi <= max_midi {
                candidates.push(make_patch(
                    measure_id,
                    beat,
                    fallback_midi,
                    beat_strength,
                    &chord,
                    "fallback_tonic",
                ));
            }
        }

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
        style_resolver::resolve_style, theme::plan_themes, PipelineState,
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
        plan_themes(&mut state, "2026-01-01").unwrap();
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
        assert!(notes >= 8, "expected at least 8 melody notes, got {notes}");
    }
}
