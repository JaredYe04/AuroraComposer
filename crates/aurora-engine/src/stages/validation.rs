use aurora_ast::Event;

use super::common::{bass_voice_id, counterpoint_enabled, drums_voice_id, iter_measures};
use super::PipelineState;

#[derive(Clone, Debug)]
pub struct ValidationReport {
    pub passed: bool,
    pub hard_violations: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            passed: true,
            hard_violations: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Stage 13 — Validation: hard constraint check before export.
pub fn validate_composition(state: &mut PipelineState) -> Result<ValidationReport, String> {
    let mut report = ValidationReport {
        passed: true,
        ..Default::default()
    };

    let bar_count = super::total_bars(&state.params) as usize;
    let measures: Vec<_> = iter_measures(&state.composition).collect();
    if measures.len() != bar_count {
        report.hard_violations.push(format!(
            "expected {bar_count} measures, found {}",
            measures.len()
        ));
    }

    for measure in &measures {
        if measure.harmony_slots.is_empty() {
            report
                .hard_violations
                .push(format!("measure {} missing harmony", measure.number.global));
        }

        let melody_notes = measure
            .voices
            .iter()
            .find(|v| v.voice_id.0 == 0)
            .map(|v| {
                v.events
                    .iter()
                    .filter(|e| matches!(e, Event::Note(_)))
                    .count()
            })
            .unwrap_or(0);
        if melody_notes == 0 {
            report.hard_violations.push(format!(
                "measure {} missing melody notes",
                measure.number.global
            ));
        }

        let bass_id = bass_voice_id(state);
        let bass_notes = measure
            .voices
            .iter()
            .find(|v| v.voice_id == bass_id)
            .map(|v| {
                v.events
                    .iter()
                    .filter(|e| matches!(e, Event::Note(_)))
                    .count()
            })
            .unwrap_or(0);
        if bass_notes == 0 {
            report.hard_violations.push(format!(
                "measure {} missing bass notes",
                measure.number.global
            ));
        }

        let drums_id = drums_voice_id(state);
        let drum_notes = measure
            .voices
            .iter()
            .find(|v| v.voice_id == drums_id)
            .map(|v| {
                v.events
                    .iter()
                    .filter(|e| matches!(e, Event::Note(n) if n.is_drum))
                    .count()
            })
            .unwrap_or(0);
        if drum_notes == 0 {
            report.hard_violations.push(format!(
                "measure {} missing drum events",
                measure.number.global
            ));
        }
    }

    if counterpoint_enabled(state) {
        let alto_notes: usize = measures
            .iter()
            .flat_map(|m| &m.voices)
            .filter(|v| v.voice_id.0 == 1)
            .flat_map(|v| &v.events)
            .filter(|e| matches!(e, Event::Note(_)))
            .count();
        if alto_notes == 0 {
            report
                .hard_violations
                .push("counterpoint enabled but no alto notes".into());
        }
    }

    for voice_def in &state.composition.voice_registry.voices {
        if voice_def.role == aurora_ast::VoiceRole::Drums && voice_def.midi_channel != 10 {
            report.hard_violations.push(format!(
                "drums voice {} not on MIDI channel 10",
                voice_def.name
            ));
        }
    }

    for measure in &measures {
        for voice in &measure.voices {
            for event in &voice.events {
                let Event::Note(note) = event else {
                    continue;
                };
                if !note.base.provenance.stage.is_some() {
                    report.warnings.push(format!(
                        "note in measure {} voice {} missing stage provenance",
                        measure.number.global, voice.voice_id.0
                    ));
                }
                if note.is_drum {
                    let drums_id = drums_voice_id(state);
                    if voice.voice_id != drums_id {
                        report.hard_violations.push(format!(
                            "drum note on non-drum voice {}",
                            voice.voice_id.0
                        ));
                    }
                } else if voice.voice_id.0 == 0 {
                    let midi = note.pitch.midi;
                    if midi < state.params.register.melody_register_min
                        || midi > state.params.register.melody_register_max
                    {
                        report.hard_violations.push(format!(
                            "melody MIDI {midi} out of register in measure {}",
                            measure.number.global
                        ));
                    }
                }
            }
        }
    }

    report.passed = report.hard_violations.is_empty();
    state.validation_report = Some(report.clone());

    if report.passed {
        Ok(report)
    } else {
        Err(format!(
            "validation failed: {}",
            report.hard_violations.join("; ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::ParameterBundle;

    #[test]
    fn validation_report_defaults_to_passed() {
        let report = ValidationReport::default();
        assert!(report.passed);
        assert!(report.hard_violations.is_empty());
    }

    #[test]
    fn empty_composition_fails_validation() {
        let mut state = PipelineState::new(
            ParameterBundle::default(),
            aurora_ast::Composition {
                id: aurora_core::NodeId::new(0, 0),
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
            crate::stages::style_resolver::resolve_style(&ParameterBundle::default()),
            aurora_ast::EmotionProfile {
                valence: 0.5,
                arousal: 0.5,
                weight_deltas: std::collections::HashMap::new(),
                tempo_delta_bpm: 0.0,
                harmonic_color_bias: 0.0,
            },
            std::collections::HashMap::new(),
        );
        assert!(validate_composition(&mut state).is_err());
    }
}
