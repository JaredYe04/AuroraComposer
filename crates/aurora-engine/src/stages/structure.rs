use aurora_ast::{
    CadenceType, CompositionMetadata, CompositionSource, GlobalAttributes, GlobalDisplayOptions,
    Margins, Measure, MeasureAttributes, MeasureNumber, MeasureVoice, MeterMap, Mode, Movement,
    MovementMetadata, PageLayout, Phrase, PhraseMetadata, PitchClass, ScoreLayout, Section,
    SectionMetadata, SectionRole, TempoMap, TempoSegment, TimeSignature, TimelinePosition,
    VoiceDef, VoiceExportSpec, VoiceId, VoiceLayoutId, VoiceRegistry, VoiceRole, AST_SCHEMA_VERSION,
    Clef, InstrumentSpec, KeyMap, KeySignature, PitchRange,
};
use aurora_ast::nodes::{VoiceGroup, VoiceGroupId, VoiceGroupKind};

use crate::progression::plan_key_changes;

use super::common::{bass_voice_id, counterpoint_enabled, drums_voice_id, harmony_pad_enabled, harmony_pad_voice_id, accompaniment_enabled, accompaniment_voice_id, resolve_accompaniment_instrument};
use crate::progression::parse_mode;
use aurora_core::NodeId;
use aurora_ast::provenance::ProvenanceRoot;

use super::{total_bars, PipelineState};

/// Stage 3 — Structure Planning: 16-bar section skeleton with phrases and tempo/key maps.
pub fn plan_structure(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let bars = total_bars(&state.params);
    let beats = state.params.rhythm.time_signature_beats;
    let beat_type = state.params.rhythm.time_signature_beat_type;
    let tempo = 120.0 + f64::from(state.emotion.tempo_delta_bpm);

    let tonic_pc = state.params.mode.key % 12;
    let mode = parse_mode(&state.params.mode.mode);
    let key = KeySignature {
        tonic: PitchClass { pc: tonic_pc },
        mode,
    };

    let phrase_len = u16::from(state.params.form.phrase_length.max(1));
    let phrase_count = (bars / phrase_len).max(1);
    let mut measures: Vec<Measure> = Vec::new();
    let mut next_id = 1u64;

    let cp_enabled = counterpoint_enabled(state);
    let pad_enabled = harmony_pad_enabled(state);
    let bass_id = bass_voice_id(state);
    let drums_id = drums_voice_id(state);
    let mut voice_slots = vec![VoiceId(0), bass_id, drums_id];
    if cp_enabled {
        voice_slots.insert(1, VoiceId(1));
    } else if pad_enabled {
        voice_slots.insert(1, harmony_pad_voice_id());
    } else if accompaniment_enabled(state) {
        voice_slots.insert(1, accompaniment_voice_id());
    }

    for global in 1..=bars {
        measures.push(Measure {
            id: NodeId::new(next_id, 0),
            number: MeasureNumber {
                local: ((global - 1) % phrase_len + 1) as u16,
                global: u32::from(global),
            },
            attributes: MeasureAttributes {
                meter: None,
                key: None,
                repeat_start: false,
                repeat_end: false,
                repeat_count: None,
                volta: None,
                rehearsal_mark: None,
            },
            harmony_slots: vec![],
            voices: voice_slots
                .iter()
                .map(|&vid| MeasureVoice {
                    voice_id: vid,
                    events: vec![],
                })
                .collect(),
        });
        next_id += 1;
    }

    let phrases: Vec<Phrase> = (0..phrase_count)
        .map(|pi| {
            let start = usize::from(pi * phrase_len);
            let end = start + usize::from(phrase_len.min(bars - pi * phrase_len));
            Phrase {
                id: NodeId::new(next_id, 0),
                metadata: PhraseMetadata {
                    phrase_id: format!("phrase-{}", pi + 1),
                    cadence: Some(CadenceType::PerfectAuthentic),
                    motif_ref: None,
                    contour_hint: None,
                },
                measures: measures[start..end.min(measures.len())].to_vec(),
            }
        })
        .collect();
    next_id += 1;

    let mut registry_voices = vec![VoiceDef {
        id: VoiceId(0),
        name: "Melody".into(),
        role: VoiceRole::Melody,
        register: PitchRange {
            min_midi: state.params.register.melody_register_min,
            max_midi: state.params.register.melody_register_max,
            preferred_min: state.params.register.melody_register_min + 4,
            preferred_max: state.params.register.melody_register_max - 4,
        },
        midi_channel: 1,
        group: None,
        instrument: InstrumentSpec {
            gm_program: 0,
            name: "Piano".into(),
            transposition: 0,
            clef: Clef::Treble,
            staff_lines: 5,
        },
        export: VoiceExportSpec {
            musicxml_part_id: "P1".into(),
            staff_index: 0,
            abbrev: None,
            hide_if_empty: false,
        },
        priority: 0,
        mutable: true,
    }];

    if cp_enabled {
        registry_voices.push(VoiceDef {
            id: VoiceId(1),
            name: "Alto".into(),
            role: VoiceRole::Alto,
            register: PitchRange {
                min_midi: state.params.register.melody_register_min.saturating_sub(12),
                max_midi: state.params.register.melody_register_min.saturating_sub(4),
                preferred_min: state.params.register.melody_register_min.saturating_sub(10),
                preferred_max: state.params.register.melody_register_min.saturating_sub(6),
            },
            midi_channel: 2,
            group: Some(VoiceGroupId(0)),
            instrument: InstrumentSpec {
                gm_program: 0,
                name: "Piano".into(),
                transposition: 0,
                clef: Clef::Treble,
                staff_lines: 5,
            },
            export: VoiceExportSpec {
                musicxml_part_id: "P2".into(),
                staff_index: 1,
                abbrev: Some("A".into()),
                hide_if_empty: false,
            },
            priority: 1,
            mutable: true,
        });
    } else if pad_enabled {
        registry_voices.push(VoiceDef {
            id: harmony_pad_voice_id(),
            name: "Harmony".into(),
            role: VoiceRole::HarmonyPad,
            register: PitchRange {
                min_midi: 48,
                max_midi: 72,
                preferred_min: 52,
                preferred_max: 67,
            },
            midi_channel: 2,
            group: Some(VoiceGroupId(0)),
            instrument: InstrumentSpec {
                gm_program: 0,
                name: "Piano".into(),
                transposition: 0,
                clef: Clef::Treble,
                staff_lines: 5,
            },
            export: VoiceExportSpec {
                musicxml_part_id: "P2".into(),
                staff_index: 1,
                abbrev: Some("H".into()),
                hide_if_empty: false,
            },
            priority: 1,
            mutable: true,
        });
    } else if accompaniment_enabled(state) {
        let (gm, name) = resolve_accompaniment_instrument(
            &state.style.genre,
            &state.params.accompaniment.instrument,
        );
        registry_voices.push(VoiceDef {
            id: accompaniment_voice_id(),
            name: "Chords".into(),
            role: VoiceRole::HarmonyPad,
            register: PitchRange {
                min_midi: state.params.accompaniment.register_min,
                max_midi: state.params.accompaniment.register_max,
                preferred_min: state.params.accompaniment.register_min + 2,
                preferred_max: state.params.accompaniment.register_max - 2,
            },
            midi_channel: 2,
            group: Some(VoiceGroupId(0)),
            instrument: InstrumentSpec {
                gm_program: gm,
                name: name.into(),
                transposition: 0,
                clef: Clef::Treble,
                staff_lines: 5,
            },
            export: VoiceExportSpec {
                musicxml_part_id: "P2".into(),
                staff_index: 1,
                abbrev: Some("Ch".into()),
                hide_if_empty: false,
            },
            priority: 1,
            mutable: true,
        });
    }

    registry_voices.push(VoiceDef {
        id: bass_id,
        name: "Bass".into(),
        role: VoiceRole::Bass,
        register: PitchRange {
            min_midi: state.params.register.bass_register_min,
            max_midi: state.params.register.bass_register_max,
            preferred_min: state.params.register.bass_register_min + 2,
            preferred_max: state.params.register.bass_register_max - 2,
        },
        midi_channel: if cp_enabled || accompaniment_enabled(state) { 3 } else { 2 },
        group: Some(VoiceGroupId(0)),
        instrument: InstrumentSpec {
            gm_program: 32,
            name: "Acoustic Bass".into(),
            transposition: 0,
            clef: Clef::Bass,
            staff_lines: 5,
        },
        export: VoiceExportSpec {
            musicxml_part_id: if cp_enabled || accompaniment_enabled(state) {
                "P3"
            } else {
                "P2"
            }
            .into(),
            staff_index: if cp_enabled || accompaniment_enabled(state) {
                2
            } else {
                1
            },
            abbrev: Some("B".into()),
            hide_if_empty: false,
        },
        priority: 2,
        mutable: true,
    });

    registry_voices.push(VoiceDef {
        id: drums_id,
        name: "Drums".into(),
        role: VoiceRole::Drums,
        register: PitchRange {
            min_midi: 35,
            max_midi: 81,
            preferred_min: 36,
            preferred_max: 51,
        },
        midi_channel: 10,
        group: None,
        instrument: InstrumentSpec {
            gm_program: 0,
            name: "Standard Kit".into(),
            transposition: 0,
            clef: Clef::Percussion,
            staff_lines: 5,
        },
        export: VoiceExportSpec {
            musicxml_part_id: if cp_enabled || accompaniment_enabled(state) {
                "P4"
            } else {
                "P3"
            }
            .into(),
            staff_index: if cp_enabled || accompaniment_enabled(state) {
                3
            } else {
                2
            },
            abbrev: Some("Dr".into()),
            hide_if_empty: false,
        },
        priority: 3,
        mutable: true,
    });

    let part_order: Vec<VoiceId> = registry_voices.iter().map(|v| v.id).collect();

    state.composition = aurora_ast::Composition {
        id: NodeId::new(0, 0),
        schema_version: AST_SCHEMA_VERSION,
        metadata: CompositionMetadata {
            title: format!(
                "Aurora {} in {}",
                state.style.genre,
                chord_root_name(tonic_pc)
            ),
            subtitle: None,
            composer: Some("Aurora Composer".into()),
            lyricist: None,
            copyright: None,
            license: None,
            created_at: created_at.into(),
            modified_at: created_at.into(),
            language: None,
            parameters_used: state.params.clone(),
            emotion_profile: Some(state.emotion.clone()),
            provenance_root: ProvenanceRoot {
                session_id: uuid::Uuid::new_v4().to_string(),
                generator_version: env!("CARGO_PKG_VERSION").into(),
                seed: state.params.search.seed,
                pipeline_config_hash: "phase3-v0.1".into(),
                started_at: created_at.into(),
                completed_at: None,
            },
            tags: vec![state.style.genre.clone()],
            source: CompositionSource::Generated,
            layout: ScoreLayout {
                staff_spacing: 12.0,
                measure_numbering: aurora_ast::MeasureNumberingStyle::EveryMeasure,
                part_list_order: part_order.clone(),
            },
        },
        global: GlobalAttributes {
            default_key: key,
            default_meter: TimeSignature { beats, beat_type },
            tempo_map: TempoMap {
                default_bpm: tempo,
                segments: vec![TempoSegment {
                    start: TimelinePosition {
                        global_measure: 1,
                        beat: aurora_ast::BeatOffset::zero(),
                    },
                    bpm: tempo,
                    ramp: None,
                }],
            },
            key_map: {
                let mut km = KeyMap {
                    default: key,
                    changes: vec![],
                };
                plan_key_changes(
                    &mut km,
                    bars as usize,
                    &state.params.form.section_lengths,
                    state.params.form.section_count,
                    tonic_pc,
                    mode,
                    &state.params.mode.modulation_policy,
                );
                km
            },
            meter_map: MeterMap {
                default: TimeSignature { beats, beat_type },
                changes: vec![],
            },
            dynamics_baseline: aurora_ast::DynamicLevel::Mf,
            pickup_measure: None,
            display: GlobalDisplayOptions {
                show_metronome: true,
                show_rehearsal_marks: true,
                page_layout: PageLayout {
                    page_width_mm: 210.0,
                    page_height_mm: 297.0,
                    margins_mm: Margins {
                        top: 20.0,
                        bottom: 20.0,
                        left: 15.0,
                        right: 15.0,
                    },
                    system_distance: 10.0,
                },
            },
        },
        voice_registry: VoiceRegistry {
            voices: registry_voices,
            groups: vec![VoiceGroup {
                id: VoiceGroupId(0),
                name: "Ensemble".into(),
                kind: VoiceGroupKind::HarmonicBed,
                member_voices: part_order
                    .iter()
                    .copied()
                    .filter(|id| id.0 != drums_id.0)
                    .collect(),
            }],
            default_layout: VoiceLayoutId(0),
        },
        movements: vec![Movement {
            id: NodeId::new(next_id, 0),
            metadata: MovementMetadata {
                title: None,
                ordinal: 1,
                key_override: None,
                tempo_override: None,
            },
            sections: vec![Section {
                id: NodeId::new(next_id + 1, 0),
                metadata: SectionMetadata {
                    role: SectionRole::Verse,
                    label: Some("A".into()),
                    theme_refs: vec![],
                    key_area: Some(key),
                    repeat: None,
                    energy_level: Some(0.6),
                },
                markers: vec![],
                phrases,
            }],
        }],
    };

    Ok(())
}

fn chord_root_name(pc: u8) -> &'static str {
    match pc % 12 {
        0 => "C",
        1 => "Db",
        2 => "D",
        3 => "Eb",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "Ab",
        9 => "A",
        10 => "Bb",
        11 => "B",
        _ => "C",
    }
}
