//! Shared helpers for pipeline stages.

use aurora_ast::{
    BeatOffset, CadenceType, Composition, Event, Measure, MeasureVoice, NoteEvent, NoteType,
    PipelineStageId, Provenance, ProvenanceAgent, ProvenanceSource, RuleRef, SearchContext,
    StateRef, TimedEventBase, VoiceId, WrittenDuration,
};
use aurora_core::NodeId;

use super::PipelineState;

pub fn counterpoint_enabled(state: &PipelineState) -> bool {
    !state.params.accompaniment.enabled
        && state.params.texture.homophony_polyphony_balance <= 0.85
        && state.params.counterpoint.strictness > 0.1
}

pub fn accompaniment_enabled(state: &PipelineState) -> bool {
    state.params.accompaniment.enabled && !harmony_pad_enabled(state)
}

pub fn harmony_pad_enabled(state: &PipelineState) -> bool {
    state.params.texture.homophony_polyphony_balance > 0.85
        && state.params.texture.harmony_pad_enabled
}

pub fn harmony_pad_voice_id() -> VoiceId {
    VoiceId(1)
}

pub fn accompaniment_voice_id() -> VoiceId {
    VoiceId(1)
}

/// GM program and display name for accompaniment instrument preset.
pub fn resolve_accompaniment_instrument(genre: &str, preset: &str) -> (u8, &'static str) {
    match preset.to_lowercase().as_str() {
        "strings" => (48, "String Ensemble"),
        "piano" => (0, "Acoustic Piano"),
        _ => match genre.to_lowercase().as_str() {
            "classical" | "film" | "orchestral" => (48, "String Ensemble"),
            "jazz" | "lofi" => (4, "Electric Piano"),
            _ => (0, "Acoustic Piano"),
        },
    }
}

pub fn bass_voice_id(state: &PipelineState) -> VoiceId {
    if counterpoint_enabled(state)
        || harmony_pad_enabled(state)
        || accompaniment_enabled(state)
    {
        VoiceId(2)
    } else {
        VoiceId(1)
    }
}

pub fn drums_voice_id(state: &PipelineState) -> VoiceId {
    if counterpoint_enabled(state)
        || harmony_pad_enabled(state)
        || accompaniment_enabled(state)
    {
        VoiceId(3)
    } else {
        VoiceId(2)
    }
}

pub fn alto_voice_id(state: &PipelineState) -> Option<VoiceId> {
    counterpoint_enabled(state).then_some(VoiceId(1))
}

pub fn iter_measures(comp: &Composition) -> impl Iterator<Item = &Measure> {
    comp.movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
        .flat_map(|p| &p.measures)
}

pub fn iter_measures_mut(comp: &mut Composition) -> impl Iterator<Item = &mut Measure> {
    comp.movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
        .flat_map(|s| &mut s.phrases)
        .flat_map(|p| &mut p.measures)
}

pub fn ensure_measure_voice(measure: &mut Measure, voice_id: VoiceId) -> &mut MeasureVoice {
    if !measure.voices.iter().any(|v| v.voice_id == voice_id) {
        measure.voices.push(MeasureVoice {
            voice_id,
            events: vec![],
        });
    }
    measure
        .voices
        .iter_mut()
        .find(|v| v.voice_id == voice_id)
        .expect("voice slot just inserted")
}

pub fn make_note_provenance(
    stage: PipelineStageId,
    created_at: &str,
    rule_ids: Vec<String>,
    explanation: impl Into<String>,
) -> Provenance {
    Provenance {
        source: ProvenanceSource::Generated,
        stage: Some(stage),
        rule_ids,
        rule_refs: vec![],
        eval_score: None,
        search: None,
        parent: None,
        created_at: created_at.into(),
        agent: ProvenanceAgent::Engine { stage },
        parameters_hash: None,
        explanation: Some(explanation.into()),
    }
}

pub fn make_search_note_provenance(
    stage: PipelineStageId,
    created_at: &str,
    rule_ids: Vec<String>,
    rule_refs: Vec<RuleRef>,
    eval_score: f64,
    step: u32,
    beam_rank: u32,
    beam_width: u16,
    state_id: &str,
    explanation: impl Into<String>,
) -> Provenance {
    Provenance {
        source: ProvenanceSource::Generated,
        stage: Some(stage),
        rule_ids,
        rule_refs,
        eval_score: Some(eval_score),
        search: Some(SearchContext {
            step_index: step,
            beam_rank: beam_rank as u16,
            beam_width,
            state_ref: StateRef {
                id: state_id.to_string(),
            },
            accumulated_score: eval_score,
        }),
        parent: None,
        created_at: created_at.into(),
        agent: ProvenanceAgent::Engine { stage },
        parameters_hash: None,
        explanation: Some(explanation.into()),
    }
}

pub fn quarter_note(
    id: u64,
    beat: usize,
    midi: u8,
    velocity: u8,
    provenance: Provenance,
    is_drum: bool,
) -> NoteEvent {
    NoteEvent {
        base: TimedEventBase {
            id: NodeId::new(id, 0),
            offset: BeatOffset::new(beat as u32, 1),
            duration: WrittenDuration {
                note_type: NoteType::Quarter,
                dots: 0,
                tuplet: None,
            },
            provenance,
            visible: true,
        },
        pitch: aurora_ast::Pitch::from_midi(midi),
        velocity,
        tie: aurora_ast::TieSpec::None,
        articulations: vec![],
        ornaments: vec![],
        lyric: None,
        pitch_role: None,
        stem_direction: None,
        beam_group: None,
        is_drum,
        drum_map: None,
    }
}

pub fn push_note(measure: &mut Measure, voice_id: VoiceId, event: Event) {
    let voice = ensure_measure_voice(measure, voice_id);
    voice.events.push(event);
}

pub fn collect_melody_pitches(state: &PipelineState) -> Vec<u8> {
    let beats_per = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let bar_count = super::total_bars(&state.params);
    let total = bar_count as usize * beats_per;
    collect_melody_per_beat(state, total)
        .into_iter()
        .map(|m| m.unwrap_or(60))
        .collect()
}

/// One melody pitch per beat (first attack in each beat slot).
pub fn collect_melody_per_beat(state: &PipelineState, total_steps: usize) -> Vec<Option<u8>> {
    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let mut out = vec![None; total_steps];
    let mut global_beat = 0usize;

    for measure in iter_measures(&state.composition) {
        if let Some(voice) = measure.voices.iter().find(|v| v.voice_id.0 == 0) {
            for beat in 0..beats_per_measure {
                if global_beat >= total_steps {
                    break;
                }
                let beat_start = beat as f64;
                let beat_end = beat_start + 1.0;
                let mut best: Option<(f64, u8)> = None;
                for event in &voice.events {
                    if let Event::Note(n) = event {
                        let q = n.base.offset.numer as f64 / f64::from(n.base.offset.denom.max(1));
                        if q >= beat_start && q < beat_end {
                            let ord = q;
                            if best.map(|(b, _)| ord < b).unwrap_or(true) {
                                best = Some((ord, n.pitch.midi));
                            }
                        }
                    }
                }
                out[global_beat] = best.map(|(_, m)| m);
                global_beat += 1;
            }
        } else {
            global_beat += beats_per_measure;
        }
    }
    out
}

/// Per-beat chord grid from pipeline state (P7) or fallback from measures.
pub fn collect_per_beat_chord_grid(state: &PipelineState, total_steps: usize) -> Vec<aurora_ast::ChordSymbol> {
    if state.per_beat_chord_grid.len() >= total_steps {
        return state.per_beat_chord_grid[..total_steps].to_vec();
    }
    let beats = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let bar_count = total_steps.div_ceil(beats);
    let mut grid = Vec::with_capacity(total_steps);
    for measure in iter_measures(&state.composition) {
        let slot = measure.harmony_slots.first();
        let chord = slot.map(|s| s.symbol.clone()).unwrap_or_else(|| {
            aurora_ast::ChordSymbol::simple(
                state.params.mode.key % 12,
                aurora_ast::ChordQuality::Major,
                "I",
            )
        });
        for _ in 0..beats {
            grid.push(chord.clone());
        }
    }
    while grid.len() < total_steps {
        if let Some(last) = grid.last().cloned() {
            grid.push(last);
        } else {
            break;
        }
    }
    grid.truncate(total_steps);
    let _ = bar_count;
    grid
}

pub fn cadence_chord_root(tonic: u8, cadence: CadenceType) -> u8 {
    match cadence {
        CadenceType::Half => (tonic + 7) % 12,
        CadenceType::Deceptive => (tonic + 9) % 12,
        CadenceType::Plagal => (tonic + 5) % 12,
        _ => (tonic + 7) % 12,
    }
}
