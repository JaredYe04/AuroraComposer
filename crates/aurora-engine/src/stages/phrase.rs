use aurora_ast::CadenceType;

use super::common::iter_measures;
use super::PipelineState;

/// Cross-cutting phrase metadata stored on pipeline state for downstream stages.
#[derive(Clone, Debug)]
pub struct PhraseViolation {
    pub phrase_id: String,
    pub message: String,
}

/// PHRASE-HOOK-1 — after Stage 4: cadence expectations per phrase.
pub fn plan_phrases(state: &mut PipelineState) -> Result<(), String> {
    state.cadence_targets.clear();
    state.phrase_violations.clear();

    let prefer_authentic = state
        .params
        .cadence
        .cadence_type_preference
        .to_lowercase()
        .contains("authentic");

    let mut phrase_idx = 0usize;
    for section in state
        .composition
        .movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
    {
        for phrase in &mut section.phrases {
            let cadence = if prefer_authentic {
                if phrase_idx % 2 == 0 {
                    CadenceType::Half
                } else {
                    CadenceType::PerfectAuthentic
                }
            } else if phrase_idx % 3 == 0 {
                CadenceType::Half
            } else {
                CadenceType::PerfectAuthentic
            };
            phrase.metadata.cadence = Some(cadence);

            if let Some(last) = phrase.measures.last() {
                state.cadence_targets.insert(last.number.global, cadence);
            }
            phrase_idx += 1;
        }
    }

    Ok(())
}

/// PHRASE-HOOK-2 — before Stage 5: lock cadence chord requirements on phrase-end measures.
pub fn apply_cadence_constraints(state: &mut PipelineState) -> Result<(), String> {
    for (global, cadence) in &state.cadence_targets {
        state
            .cadence_chord_roots
            .insert(*global, super::common::cadence_chord_root(state.params.mode.key % 12, *cadence));
    }
    Ok(())
}

/// PHRASE-HOOK-3 — after Stage 7: validate phrase terminal contour; flag soft violations.
pub fn validate_phrase_terminals(state: &mut PipelineState) -> Result<(), String> {
    state.phrase_violations.clear();

    for phrase in state
        .composition
        .movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
    {
        let cadence = phrase.metadata.cadence.unwrap_or(CadenceType::PerfectAuthentic);
        let Some(last_measure) = phrase.measures.last() else {
            continue;
        };

        let melody_notes: Vec<_> = last_measure
            .voices
            .iter()
            .find(|v| v.voice_id.0 == 0)
            .map(|v| {
                v.events
                    .iter()
                    .filter_map(|e| match e {
                        aurora_ast::Event::Note(n) => Some(n.pitch.midi),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        if melody_notes.is_empty() {
            state.phrase_violations.push(PhraseViolation {
                phrase_id: phrase.metadata.phrase_id.clone(),
                message: "phrase ends with no melody notes".into(),
            });
            continue;
        }

        let last_midi = *melody_notes.last().unwrap();
        let tonic_midi = 60 + (state.params.mode.key % 12);
        match cadence {
            CadenceType::PerfectAuthentic | CadenceType::ImperfectAuthentic => {
                if last_midi % 12 != tonic_midi % 12 {
                    state.phrase_violations.push(PhraseViolation {
                        phrase_id: phrase.metadata.phrase_id.clone(),
                        message: format!(
                            "PAC expected tonic pitch class, got MIDI {last_midi}"
                        ),
                    });
                }
            }
            CadenceType::Half => {
                let dominant_pc = (state.params.mode.key + 7) % 12;
                if last_midi % 12 != dominant_pc {
                    state.phrase_violations.push(PhraseViolation {
                        phrase_id: phrase.metadata.phrase_id.clone(),
                        message: format!("HC expected dominant pitch class, got MIDI {last_midi}"),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Count phrase-end measures for progress reporting.
pub fn phrase_end_count(state: &PipelineState) -> usize {
    state.cadence_targets.len().max(
        iter_measures(&state.composition)
            .filter(|m| m.number.local == 4)
            .count(),
    )
}
