use aurora_ast::{CadenceType, Composition, EmotionProfile, PipelineStageId};
use aurora_core::ParameterBundle;
use std::collections::HashMap;

pub mod common;
pub mod counterpoint;
pub mod decoration;
pub mod drums;
pub mod emotion_resolver;
pub mod harmony;
pub mod melody;
pub mod phrase;
pub mod repair;
pub mod rhythm;
pub mod structure;
pub mod style_resolver;
pub mod theme;
pub mod validation;
pub mod bass;

pub use bass::generate_bass;
pub use counterpoint::generate_counterpoint;
pub use decoration::apply_decoration;
pub use drums::generate_drums;
pub use emotion_resolver::resolve_emotion;
pub use harmony::generate_harmony;
pub use melody::generate_melody;
pub use phrase::{
    apply_cadence_constraints, plan_phrases, validate_phrase_terminals, PhraseViolation,
};
pub use repair::repair_composition;
pub use rhythm::generate_rhythm;
pub use structure::plan_structure;
pub use style_resolver::{resolve_style, ResolvedStyle};
pub use theme::plan_themes;
pub use validation::{validate_composition, ValidationReport};

/// Mutable pipeline state threaded through all generative stages.
pub struct PipelineState {
    pub params: ParameterBundle,
    pub composition: Composition,
    pub style: ResolvedStyle,
    pub emotion: EmotionProfile,
    pub weight_deltas: HashMap<String, f32>,
    /// Per-measure accent weights (one vec per global measure index).
    pub rhythm_accents: Vec<Vec<f32>>,
    /// Phrase-end measure global number → expected cadence type.
    pub cadence_targets: HashMap<u32, CadenceType>,
    /// Phrase-end measure global number → required chord root PC for cadence lock.
    pub cadence_chord_roots: HashMap<u32, u8>,
    /// Soft violations flagged by PHRASE-HOOK-3 for Repair stage.
    pub phrase_violations: Vec<PhraseViolation>,
    pub validation_report: Option<ValidationReport>,
}

impl PipelineState {
    pub fn new(
        params: ParameterBundle,
        composition: Composition,
        style: ResolvedStyle,
        emotion: EmotionProfile,
        weight_deltas: HashMap<String, f32>,
    ) -> Self {
        Self {
            params,
            composition,
            style,
            emotion,
            weight_deltas,
            rhythm_accents: Vec::new(),
            cadence_targets: HashMap::new(),
            cadence_chord_roots: HashMap::new(),
            phrase_violations: Vec::new(),
            validation_report: None,
        }
    }
}

pub fn total_bars(params: &ParameterBundle) -> u16 {
    let sum: u16 = params.form.section_lengths.iter().sum();
    if sum > 0 {
        sum
    } else {
        16
    }
}

pub fn normalize_prototype_params(params: &mut ParameterBundle) {
    if params.form.section_lengths.is_empty() {
        params.form.section_lengths = vec![16];
    }
    params.form.section_count = params.form.section_count.max(1);
    params.rhythm.time_signature_beats = 4;
    params.rhythm.time_signature_beat_type = 4;
    if params.mode.mode.is_empty() {
        params.mode.mode = "major".into();
    }
    if params.search.beam_width == 0 {
        params.search.beam_width = 16;
    }
}

/// Stage 14 — Export placeholder: AST is complete; IR projection lives in `aurora-export`.
pub fn finalize_export(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    state.composition.metadata.modified_at = chrono::Utc::now().to_rfc3339();
    state.composition.metadata.provenance_root.completed_at =
        Some(chrono::Utc::now().to_rfc3339());
    state.composition.metadata.provenance_root.pipeline_config_hash = "phase3-v0.1".into();

    for movement in &mut state.composition.movements {
        for section in &mut movement.sections {
            for phrase in &mut section.phrases {
                for measure in &mut phrase.measures {
                    for slot in &mut measure.harmony_slots {
                        if slot.provenance.stage.is_none() {
                            slot.provenance.stage = Some(PipelineStageId::HarmonySkeleton);
                        }
                    }
                }
            }
        }
    }

    let _ = created_at;
    Ok(())
}
