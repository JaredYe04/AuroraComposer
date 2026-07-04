//! Fluent composition builder for tests and fixtures.

use aurora_core::{NodeId, ParameterBundle};

use crate::nodes::{
    AST_SCHEMA_VERSION, Clef, Composition, CompositionMetadata, CompositionSource,
    GlobalAttributes, GlobalDisplayOptions, InstrumentSpec, KeyMap, KeySignature, Margins,
    Measure, MeasureAttributes, MeasureNumber, MeasureVoice, MeterMap, Mode, Movement,
    MovementMetadata, PageLayout, Phrase, PhraseMetadata, PitchClass, ScoreLayout, Section,
    SectionMetadata, SectionRole, TempoMap, TimeSignature, VoiceDef, VoiceExportSpec, VoiceId,
    VoiceLayoutId, VoiceRegistry, VoiceRole,
};
use crate::provenance::ProvenanceRoot;
use crate::types::PitchRange;

/// Build minimal valid compositions for unit tests.
#[derive(Clone, Debug)]
pub struct CompositionBuilder {
    title: String,
    voice_count: u16,
    measure_count: u16,
    next_node_id: u64,
}

impl CompositionBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Untitled".into(),
            voice_count: 1,
            measure_count: 0,
            next_node_id: 1,
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    #[must_use]
    pub fn voices(mut self, count: u16) -> Self {
        self.voice_count = count.max(1);
        self
    }

    #[must_use]
    pub fn one_measure(mut self) -> Self {
        self.measure_count = 1;
        self
    }

    #[must_use]
    pub fn measures(mut self, count: u16) -> Self {
        self.measure_count = count;
        self
    }

    #[must_use]
    pub fn build(self) -> Composition {
        let now = "2026-01-01T00:00:00Z";
        let CompositionBuilder {
            title,
            voice_count,
            measure_count,
            mut next_node_id,
        } = self;
        let measure_count = measure_count.max(1);
        let mut alloc = || {
            let id = next_node_id;
            next_node_id += 1;
            NodeId::new(id, 0)
        };
        let voice_registry = default_voice_registry(voice_count);
        let measures: Vec<Measure> = (0..measure_count)
            .map(|i| {
                let global = u32::from(i + 1);
                Measure {
                    id: alloc(),
                    number: MeasureNumber {
                        local: i + 1,
                        global,
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
                    voices: voice_registry
                        .voices
                        .iter()
                        .map(|v| MeasureVoice {
                            voice_id: v.id,
                            events: Vec::new(),
                        })
                        .collect(),
                }
            })
            .collect();

        Composition {
            id: alloc(),
            schema_version: AST_SCHEMA_VERSION,
            metadata: CompositionMetadata {
                title,
                subtitle: None,
                composer: None,
                lyricist: None,
                copyright: None,
                license: None,
                created_at: now.into(),
                modified_at: now.into(),
                language: None,
                parameters_used: ParameterBundle::default(),
                emotion_profile: None,
                provenance_root: ProvenanceRoot {
                    session_id: "test-session".into(),
                    generator_version: "0.1.0".into(),
                    seed: Some(42),
                    pipeline_config_hash: "test".into(),
                    started_at: now.into(),
                    completed_at: None,
                },
                tags: vec![],
                source: CompositionSource::Generated,
                layout: ScoreLayout {
                    staff_spacing: 12.0,
                    measure_numbering: crate::nodes::MeasureNumberingStyle::EveryMeasure,
                    part_list_order: voice_registry.voices.iter().map(|v| v.id).collect(),
                },
            },
            global: default_global(),
            voice_registry,
            movements: vec![Movement {
                id: alloc(),
                metadata: MovementMetadata {
                    title: None,
                    ordinal: 1,
                    key_override: None,
                    tempo_override: None,
                },
                sections: vec![Section {
                    id: alloc(),
                    metadata: SectionMetadata {
                        role: SectionRole::Verse,
                        label: Some("A".into()),
                        theme_refs: vec![],
                        key_area: None,
                        repeat: None,
                        energy_level: None,
                    },
                    markers: vec![],
                    phrases: vec![Phrase {
                        id: alloc(),
                        metadata: PhraseMetadata {
                            phrase_id: "phrase-1".into(),
                            cadence: None,
                            motif_ref: None,
                            contour_hint: None,
                        },
                        measures,
                    }],
                }],
            }],
        }
    }
}

impl Default for CompositionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn default_global() -> GlobalAttributes {
    GlobalAttributes {
        default_key: KeySignature {
            tonic: PitchClass { pc: 0 },
            mode: Mode::Major,
        },
        default_meter: TimeSignature {
            beats: 4,
            beat_type: 4,
        },
        tempo_map: TempoMap {
            default_bpm: 120.0,
            segments: vec![],
        },
        key_map: KeyMap {
            default: KeySignature {
                tonic: PitchClass { pc: 0 },
                mode: Mode::Major,
            },
            changes: vec![],
        },
        meter_map: MeterMap {
            default: TimeSignature {
                beats: 4,
                beat_type: 4,
            },
            changes: vec![],
        },
        dynamics_baseline: crate::nodes::DynamicLevel::Mf,
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
    }
}

fn default_voice_registry(count: u16) -> VoiceRegistry {
    let voices: Vec<VoiceDef> = (0..count)
        .map(|i| {
            let role = if i == 0 {
                VoiceRole::Melody
            } else {
                VoiceRole::Inner
            };
            VoiceDef {
                id: VoiceId(i),
                name: format!("Voice {i}"),
                role,
                register: PitchRange {
                    min_midi: 48,
                    max_midi: 84,
                    preferred_min: 55,
                    preferred_max: 76,
                },
                midi_channel: u8::try_from(i + 1).unwrap_or(1),
                group: None,
                instrument: InstrumentSpec {
                    gm_program: 0,
                    name: "Piano".into(),
                    transposition: 0,
                    clef: Clef::Treble,
                    staff_lines: 5,
                },
                export: VoiceExportSpec {
                    musicxml_part_id: format!("P{}", i + 1),
                    staff_index: 0,
                    abbrev: None,
                    hide_if_empty: false,
                },
                priority: u8::try_from(i).unwrap_or(0),
                mutable: true,
            }
        })
        .collect();

    VoiceRegistry {
        voices,
        groups: vec![],
        default_layout: VoiceLayoutId(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_one_measure_with_voices() {
        let comp = CompositionBuilder::new().one_measure().voices(2).build();
        let measure = &comp.movements[0].sections[0].phrases[0].measures[0];
        assert_eq!(measure.voices.len(), 2);
        assert_eq!(comp.voice_registry.voices.len(), 2);
    }

    #[test]
    fn builder_sets_title() {
        let comp = CompositionBuilder::new().title("My Piece").one_measure().build();
        assert_eq!(comp.metadata.title, "My Piece");
    }

    #[test]
    fn builder_assigns_unique_node_ids() {
        let comp = CompositionBuilder::new().one_measure().build();
        let measure_id = comp.movements[0].sections[0].phrases[0].measures[0].id;
        assert_ne!(comp.id, measure_id);
    }
}
