//! Pipeline orchestrator — runs all 14 stages with phrase hooks and progress reporting.

use aurora_ast::Composition;
use aurora_core::ParameterBundle;

use crate::error::EngineError;
use crate::progress::{report, ProgressCallback};
use crate::stages::{
    apply_cadence_constraints, apply_decoration, finalize_export, generate_bass,
    generate_counterpoint, generate_drums, generate_harmony, generate_melody, generate_rhythm,
    normalize_prototype_params, plan_phrases, plan_structure, plan_themes, repair_composition,
    resolve_emotion, resolve_style, validate_composition, validate_phrase_terminals,
    PipelineState,
};

const TOTAL_STAGES: u8 = 14;

/// Full pipeline stage names indexed by stage number (1-based).
const STAGE_NAMES: [&str; 14] = [
    "Style Resolver",
    "Emotion Resolver",
    "Structure Planning",
    "Theme Planning",
    "Harmony Skeleton",
    "Rhythm Skeleton",
    "Melody",
    "Counterpoint",
    "Bass",
    "Drums",
    "Decoration",
    "Repair",
    "Validation",
    "Export",
];

pub struct PipelineOrchestrator {
    progress: Option<ProgressCallback>,
}

impl Default for PipelineOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineOrchestrator {
    #[must_use]
    pub fn new() -> Self {
        Self { progress: None }
    }

    #[must_use]
    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress = Some(callback);
        self
    }

    fn stage_err(stage: u8, msg: String) -> EngineError {
        EngineError::StageFailed { stage, message: msg }
    }

    fn search_err(msg: String) -> EngineError {
        if msg.contains("exhausted") {
            EngineError::from(aurora_rules::SearchExhausted {
                trace: Default::default(),
            })
        } else {
            EngineError::StageFailed {
                stage: 7,
                message: msg,
            }
        }
    }

    /// Run the full 14-stage pipeline and return a validated [`Composition`].
    pub fn run(&self, params: &ParameterBundle) -> Result<Composition, EngineError> {
        let mut params = params.clone();
        normalize_prototype_params(&mut params);

        let created_at = chrono::Utc::now().to_rfc3339();

        let style = resolve_style(&params);
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[0],
            1,
            TOTAL_STAGES,
            1.0,
            "Resolved style preset and rule bundles",
        );

        let (emotion, weight_deltas) = resolve_emotion(&params);
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[1],
            2,
            TOTAL_STAGES,
            1.0,
            "Mapped emotion dimensions to weight deltas",
        );

        let mut state = PipelineState::new(
            params.clone(),
            empty_composition(),
            style,
            emotion,
            weight_deltas,
        );

        plan_structure(&mut state, &created_at).map_err(|msg| Self::stage_err(3, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[2],
            3,
            TOTAL_STAGES,
            1.0,
            format!("Planned {} bars in 4/4", crate::stages::total_bars(&params)),
        );

        plan_themes(&mut state, &created_at).map_err(|msg| Self::stage_err(4, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[3],
            4,
            TOTAL_STAGES,
            1.0,
            format!(
                "Assigned {} theme slot(s)",
                state.params.theme.theme_count.max(1)
            ),
        );

        plan_phrases(&mut state).map_err(|msg| Self::stage_err(4, msg))?;
        apply_cadence_constraints(&mut state).map_err(|msg| Self::stage_err(4, msg))?;

        generate_harmony(&mut state, &created_at).map_err(|msg| Self::stage_err(5, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[4],
            5,
            TOTAL_STAGES,
            1.0,
            "Generated chord progression skeleton",
        );

        generate_rhythm(&mut state, &created_at).map_err(|msg| Self::stage_err(6, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[5],
            6,
            TOTAL_STAGES,
            1.0,
            "Applied rhythm skeleton patterns",
        );

        generate_melody(&mut state, &created_at).map_err(|msg| Self::search_err(msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[6],
            7,
            TOTAL_STAGES,
            1.0,
            "Committed beam-search melody with provenance",
        );

        validate_phrase_terminals(&mut state).map_err(|msg| Self::stage_err(7, msg))?;

        generate_counterpoint(&mut state, &created_at)
            .map_err(|msg| Self::stage_err(8, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[7],
            8,
            TOTAL_STAGES,
            1.0,
            if crate::stages::common::counterpoint_enabled(&state) {
                "Generated inner alto voice via beam search"
            } else {
                "Skipped counterpoint (homophonic texture)"
            },
        );

        generate_bass(&mut state, &created_at).map_err(|msg| Self::stage_err(9, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[8],
            9,
            TOTAL_STAGES,
            1.0,
            "Generated bass line via narrow beam search",
        );

        generate_drums(&mut state, &created_at).map_err(|msg| Self::stage_err(10, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[9],
            10,
            TOTAL_STAGES,
            1.0,
            "Generated drum patterns on channel 10",
        );

        apply_decoration(&mut state, &created_at).map_err(|msg| Self::stage_err(11, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[10],
            11,
            TOTAL_STAGES,
            1.0,
            "Applied ornamental enrichment",
        );

        repair_composition(&mut state, &created_at).map_err(|msg| Self::stage_err(12, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[11],
            12,
            TOTAL_STAGES,
            1.0,
            format!(
                "Repaired {} soft violation(s)",
                state.phrase_violations.len()
            ),
        );

        validate_composition(&mut state).map_err(|msg| Self::stage_err(13, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[12],
            13,
            TOTAL_STAGES,
            1.0,
            "Passed hard constraint validation",
        );

        finalize_export(&mut state, &created_at).map_err(|msg| Self::stage_err(14, msg))?;
        report(
            &self.progress.as_ref(),
            STAGE_NAMES[13],
            14,
            TOTAL_STAGES,
            1.0,
            "Pipeline complete — ready for IR export",
        );

        Ok(state.composition)
    }
}

fn empty_composition() -> Composition {
    use aurora_ast::{
        CompositionMetadata, CompositionSource, GlobalAttributes, GlobalDisplayOptions, KeyMap,
        KeySignature, Margins, MeterMap, Mode, PageLayout, PitchClass, ProvenanceRoot,
        ScoreLayout, TempoMap, TimeSignature, VoiceLayoutId, VoiceRegistry, AST_SCHEMA_VERSION,
    };
    use aurora_core::NodeId;

    let now = chrono::Utc::now().to_rfc3339();
    Composition {
        id: NodeId::new(0, 0),
        schema_version: AST_SCHEMA_VERSION,
        metadata: CompositionMetadata {
            title: "Untitled".into(),
            subtitle: None,
            composer: None,
            lyricist: None,
            copyright: None,
            license: None,
            created_at: now.clone(),
            modified_at: now.clone(),
            language: None,
            parameters_used: ParameterBundle::default(),
            emotion_profile: None,
            provenance_root: ProvenanceRoot {
                session_id: uuid::Uuid::new_v4().to_string(),
                generator_version: env!("CARGO_PKG_VERSION").into(),
                seed: None,
                pipeline_config_hash: "phase3-v0.1".into(),
                started_at: now,
                completed_at: None,
            },
            tags: vec![],
            source: CompositionSource::Generated,
            layout: ScoreLayout {
                staff_spacing: 12.0,
                measure_numbering: aurora_ast::MeasureNumberingStyle::EveryMeasure,
                part_list_order: vec![],
            },
        },
        global: GlobalAttributes {
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
            voices: vec![],
            groups: vec![],
            default_layout: VoiceLayoutId(0),
        },
        movements: vec![],
    }
}

/// Public entry point: generate a complete composition through all 14 stages.
pub fn generate_composition(params: ParameterBundle) -> Result<Composition, EngineError> {
    PipelineOrchestrator::new().run(&params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_ast::{Event, VoiceRole};

    #[test]
    fn orchestrator_runs_full_pipeline() {
        let mut params = ParameterBundle::default();
        params.style.genre = "classical".into();
        params.form.section_lengths = vec![16];
        params.mode.key = 0;
        params.mode.mode = "major".into();
        params.texture.homophony_polyphony_balance = 0.5;

        let comp = generate_composition(params).expect("generation should succeed");
        assert_eq!(count_measures(&comp), 16);

        let voices: Vec<_> = comp
            .voice_registry
            .voices
            .iter()
            .map(|v| v.role)
            .collect();
        assert!(voices.contains(&VoiceRole::Melody));
        assert!(voices.contains(&VoiceRole::Bass));
        assert!(voices.contains(&VoiceRole::Drums));

        let melody_notes = count_voice_notes(&comp, 0);
        assert_eq!(melody_notes, 64);

        let drums_channel = comp
            .voice_registry
            .voices
            .iter()
            .find(|v| v.role == VoiceRole::Drums)
            .map(|v| v.midi_channel)
            .unwrap_or(0);
        assert_eq!(drums_channel, 10);
    }

    fn count_measures(comp: &Composition) -> usize {
        comp.movements
            .iter()
            .flat_map(|m| &m.sections)
            .flat_map(|s| &s.phrases)
            .flat_map(|p| &p.measures)
            .count()
    }

    fn count_voice_notes(comp: &Composition, voice_id: u16) -> usize {
        comp.movements
            .iter()
            .flat_map(|m| &m.sections)
            .flat_map(|s| &s.phrases)
            .flat_map(|p| &p.measures)
            .flat_map(|m| &m.voices)
            .filter(|v| v.voice_id.0 == voice_id)
            .flat_map(|v| &v.events)
            .filter(|e| matches!(e, Event::Note(_)))
            .count()
    }
}
