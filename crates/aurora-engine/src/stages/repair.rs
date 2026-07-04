use aurora_ast::{
    CadenceType, Event, PipelineStageId, ProvenanceAgent, ProvenanceSource,
};

use super::common::{bass_voice_id, iter_measures_mut};
use super::PipelineState;

/// Stage 12 — Repair: fix soft violations (range, phrase terminals).
pub fn repair_composition(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    for violation in &state.phrase_violations.clone() {
        repair_phrase_terminal(state, violation, created_at);
    }

    clamp_register(
        state,
        aurora_ast::VoiceId(0),
        state.params.register.melody_register_min,
        state.params.register.melody_register_max,
        created_at,
    );

    if let Some(alto) = super::common::alto_voice_id(state) {
        let min = state.params.register.melody_register_min.saturating_sub(12);
        let max = state.params.register.melody_register_min.saturating_sub(4);
        clamp_register(state, alto, min, max, created_at);
    }

    clamp_register(
        state,
        bass_voice_id(state),
        state.params.register.bass_register_min,
        state.params.register.bass_register_max,
        created_at,
    );

    Ok(())
}

fn clamp_register(
    state: &mut PipelineState,
    voice_id: aurora_ast::VoiceId,
    min_midi: u8,
    max_midi: u8,
    created_at: &str,
) {
    for measure in iter_measures_mut(&mut state.composition) {
        if let Some(voice) = measure.voices.iter_mut().find(|v| v.voice_id == voice_id) {
            for event in &mut voice.events {
                let Event::Note(note) = event else {
                    continue;
                };
                let clamped = note.pitch.midi.clamp(min_midi, max_midi);
                if clamped != note.pitch.midi {
                    note.pitch.midi = clamped;
                    note.base.provenance.source = ProvenanceSource::Repaired;
                    note.base.provenance.stage = Some(PipelineStageId::Repair);
                    note.base.provenance.rule_ids = vec!["REPAIR-RANGE-001".into()];
                    note.base.provenance.agent = ProvenanceAgent::Engine {
                        stage: PipelineStageId::Repair,
                    };
                    note.base.provenance.explanation =
                        Some(format!("clamped MIDI to register [{min_midi},{max_midi}]"));
                    note.base.provenance.created_at = created_at.into();
                }
            }
        }
    }
}

fn repair_phrase_terminal(
    state: &mut PipelineState,
    violation: &super::phrase::PhraseViolation,
    created_at: &str,
) {
    let tonic_pc = state.params.mode.key % 12;
    let target_pc = if violation.message.contains("dominant") {
        (tonic_pc + 7) % 12
    } else {
        tonic_pc
    };

    for phrase in state
        .composition
        .movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
        .flat_map(|s| &mut s.phrases)
        .filter(|p| p.metadata.phrase_id == violation.phrase_id)
    {
        let cadence = phrase.metadata.cadence.unwrap_or(CadenceType::PerfectAuthentic);
        let Some(last_measure) = phrase.measures.last_mut() else {
            continue;
        };
        if let Some(voice) = last_measure.voices.iter_mut().find(|v| v.voice_id.0 == 0) {
            if let Some(Event::Note(note)) = voice.events.last_mut() {
                let octave = note.pitch.midi / 12;
                note.pitch.midi = octave * 12 + target_pc;
                note.base.provenance.source = ProvenanceSource::Repaired;
                note.base.provenance.stage = Some(PipelineStageId::Repair);
                note.base.provenance.rule_ids = vec![match cadence {
                    CadenceType::Half => "HARM-017".into(),
                    _ => "HARM-015".into(),
                }];
                note.base.provenance.agent = ProvenanceAgent::Engine {
                    stage: PipelineStageId::Repair,
                };
                note.base.provenance.explanation =
                    Some(format!("repaired phrase terminal for {}", violation.phrase_id));
                note.base.provenance.created_at = created_at.into();
            }
        }
    }
}
