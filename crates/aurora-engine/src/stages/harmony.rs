use aurora_ast::{
    ChordQuality, ChordSymbol, HarmonicFunction, HarmonySlot, PitchClass, PipelineStageId,
    Provenance, ProvenanceAgent, ProvenanceSource,
};
use aurora_core::NodeId;

use super::PipelineState;

/// Stage 5 — Harmony Skeleton: I–IV–V–I (classical/pop) or ii–V–I (jazz).
pub fn generate_harmony(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let tonic = state.params.mode.key % 12;
    let progression = if state.style.jazz_harmony {
        jazz_progression(tonic)
    } else {
        diatonic_progression(tonic)
    };

    let measures = state
        .composition
        .movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
        .flat_map(|s| &mut s.phrases)
        .flat_map(|p| &mut p.measures);

    for (i, measure) in measures.enumerate() {
        let global = measure.number.global;
        let mut chord = progression[i % progression.len()].clone();

        if let Some(&root_pc) = state.cadence_chord_roots.get(&global) {
            chord.symbol.root.pc = root_pc;
            chord.symbol.raw = format!("{}{}", chord_root_name(root_pc), quality_suffix(chord.symbol.quality));
            chord.function = aurora_ast::HarmonicFunction::Dominant;
        }

        measure.harmony_slots.push(HarmonySlot {
            id: NodeId::new(u64::try_from(i + 1000).unwrap_or(1000), 0),
            offset: aurora_ast::BeatOffset::zero(),
            duration: aurora_ast::BeatOffset::new(4, 1),
            symbol: chord.symbol.clone(),
            roman_numeral: Some(chord.roman.clone()),
            function: Some(chord.function),
            provenance: Provenance {
                source: ProvenanceSource::Generated,
                stage: Some(PipelineStageId::HarmonySkeleton),
                rule_ids: vec![if state.style.jazz_harmony {
                    "JAZZ-012".into()
                } else {
                    "HARM-PROG-003".into()
                }],
                rule_refs: vec![],
                eval_score: None,
                search: None,
                parent: None,
                created_at: created_at.into(),
                agent: ProvenanceAgent::Engine {
                    stage: PipelineStageId::HarmonySkeleton,
                },
                parameters_hash: None,
                explanation: Some(format!(
                    "{} progression slot {}",
                    if state.style.jazz_harmony {
                        "ii-V-I"
                    } else {
                        "I-IV-V-I"
                    },
                    i % progression.len()
                )),
            },
        });
    }

    Ok(())
}

#[derive(Clone)]
struct PlannedChord {
    symbol: ChordSymbol,
    roman: String,
    function: HarmonicFunction,
}

fn diatonic_progression(tonic: u8) -> Vec<PlannedChord> {
    vec![
        make_triad(tonic, ChordQuality::Major, "I", HarmonicFunction::Tonic),
        make_triad(
            (tonic + 5) % 12,
            ChordQuality::Major,
            "IV",
            HarmonicFunction::Subdominant,
        ),
        make_triad(
            (tonic + 7) % 12,
            ChordQuality::Major,
            "V",
            HarmonicFunction::Dominant,
        ),
        make_triad(tonic, ChordQuality::Major, "I", HarmonicFunction::Tonic),
    ]
}

fn jazz_progression(tonic: u8) -> Vec<PlannedChord> {
    let ii = (tonic + 2) % 12;
    let v = (tonic + 7) % 12;
    vec![
        PlannedChord {
            symbol: ChordSymbol {
                root: PitchClass { pc: ii },
                quality: ChordQuality::Minor7,
                extensions: vec![],
                bass: None,
                raw: format!("{}m7", chord_root_name(ii)),
            },
            roman: "ii7".into(),
            function: HarmonicFunction::Predominant,
        },
        PlannedChord {
            symbol: ChordSymbol {
                root: PitchClass { pc: v },
                quality: ChordQuality::Dominant7,
                extensions: vec![],
                bass: None,
                raw: format!("{}7", chord_root_name(v)),
            },
            roman: "V7".into(),
            function: HarmonicFunction::Dominant,
        },
        make_triad(tonic, ChordQuality::Major7, "I", HarmonicFunction::Tonic),
    ]
}

fn make_triad(
    root: u8,
    quality: ChordQuality,
    roman: &str,
    function: HarmonicFunction,
) -> PlannedChord {
    PlannedChord {
        symbol: ChordSymbol {
            root: PitchClass { pc: root },
            quality,
            extensions: vec![],
            bass: None,
            raw: format!("{}{}", chord_root_name(root), quality_suffix(quality)),
        },
        roman: roman.into(),
        function,
    }
}

fn quality_suffix(q: ChordQuality) -> &'static str {
    match q {
        ChordQuality::Major => "",
        ChordQuality::Major7 => "maj7",
        ChordQuality::Minor => "m",
        ChordQuality::Minor7 => "m7",
        ChordQuality::Dominant7 => "7",
        _ => "",
    }
}

fn chord_root_name(pc: u8) -> &'static str {
    match pc % 12 {
        0 => "C",
        1 => "C#",
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

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::ParameterBundle;

    use crate::stages::{
        emotion_resolver::resolve_emotion, structure::plan_structure, style_resolver::resolve_style,
        PipelineState,
    };

    #[test]
    fn classical_progression_has_four_slots() {
        let mut params = ParameterBundle::default();
        params.style.genre = "classical".into();
        params.form.section_lengths = vec![4];
        let style = resolve_style(&params);
        let (emotion, deltas) = resolve_emotion(&params);
        let mut state = PipelineState::new(
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
                        tonic: PitchClass { pc: 0 },
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
                            tonic: PitchClass { pc: 0 },
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
        );
        plan_structure(&mut state, "2026-01-01").unwrap();
        generate_harmony(&mut state, "2026-01-01").unwrap();
        let measure = &state.composition.movements[0].sections[0].phrases[0].measures[0];
        assert!(!measure.harmony_slots.is_empty());
        assert_eq!(measure.harmony_slots[0].roman_numeral.as_deref(), Some("I"));
    }
}
