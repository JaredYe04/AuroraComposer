use serde::{Deserialize, Serialize};

use crate::params::{
    derived_chord_tone_bias, derived_neighbor_tone_bias, derived_passing_tone_bias,
    sanitize_generation_bundle, ParameterBundle,
};

/// Flat parameter DTO for UI (maps to [`ParameterBundle`]).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UiParameterSnapshot {
    pub key: u8,
    pub mode: String,
    pub style: String,
    pub beam_width: u16,
    pub bars: u16,
    pub tempo_bpm: f64,
    pub emotion_valence: f32,
    pub emotion_arousal: f32,
    pub harmony_complexity: f32,
    pub counterpoint_strictness: f32,
    pub drum_density: f32,
    pub drum_accent_emphasis: f32,
    pub drum_hihat_density: f32,
    /// `"loop"` or `"flow"` — harmony progression structure.
    pub progression_mode: String,
    /// 0 = open/chromatic; 1 = strict consonance (default ~0.65).
    pub tonal_conservatism: f32,
    /// `"auto"`, `"piano"`, or `"strings"`.
    pub accompaniment_instrument: String,
    /// Deterministic RNG seed for generation (always set from UI).
    pub seed: u64,
}

impl Default for UiParameterSnapshot {
    fn default() -> Self {
        Self {
            key: 0,
            mode: "major".into(),
            style: "classical".into(),
            beam_width: 8,
            bars: 8,
            tempo_bpm: 120.0,
            emotion_valence: 0.5,
            emotion_arousal: 0.5,
            harmony_complexity: 0.5,
            counterpoint_strictness: 0.5,
            drum_density: 0.5,
            drum_accent_emphasis: 0.75,
            drum_hihat_density: 0.6,
            progression_mode: "loop".into(),
            tonal_conservatism: 0.75,
            accompaniment_instrument: "auto".into(),
            seed: 0,
        }
    }
}

impl From<&ParameterBundle> for UiParameterSnapshot {
    fn from(p: &ParameterBundle) -> Self {
        let bars = p
            .form
            .section_lengths
            .first()
            .copied()
            .unwrap_or(8);
        Self {
            key: p.mode.key,
            mode: p.mode.mode.clone(),
            style: p.style.genre.clone(),
            beam_width: p.search.beam_width,
            bars,
            tempo_bpm: 120.0,
            emotion_valence: p.emotion.valence,
            emotion_arousal: p.emotion.arousal,
            harmony_complexity: p.harmony.complexity,
            counterpoint_strictness: p.counterpoint.strictness,
            drum_density: p.drums.density,
            drum_accent_emphasis: p.drums.accent_emphasis,
            drum_hihat_density: p.drums.hihat_density,
            progression_mode: match p.harmony.progression_mode {
                crate::ProgressionMode::Loop => "loop".into(),
                crate::ProgressionMode::Flow => "flow".into(),
            },
            tonal_conservatism: p.melody.tonal_conservatism,
            accompaniment_instrument: p.accompaniment.instrument.clone(),
            seed: p.search.seed.unwrap_or(0),
        }
    }
}

impl From<UiParameterSnapshot> for ParameterBundle {
    fn from(ui: UiParameterSnapshot) -> Self {
        let mut bundle = ParameterBundle::default();
        bundle.mode.key = ui.key;
        bundle.mode.mode = ui.mode;
        bundle.style.genre = ui.style;
        bundle.search.beam_width = ui.beam_width;
        bundle.form.section_lengths = vec![ui.bars];
        bundle.form.section_count = 1;
        bundle.emotion.valence = ui.emotion_valence;
        bundle.emotion.arousal = ui.emotion_arousal;
        bundle.harmony.complexity = ui.harmony_complexity;
        bundle.counterpoint.strictness = ui.counterpoint_strictness;
        bundle.drums.density = ui.drum_density;
        bundle.drums.accent_emphasis = ui.drum_accent_emphasis;
        bundle.drums.hihat_density = ui.drum_hihat_density;
        bundle.harmony.progression_mode = if ui.progression_mode == "flow" {
            crate::ProgressionMode::Flow
        } else {
            crate::ProgressionMode::Loop
        };
        let t = ui.tonal_conservatism.clamp(0.0, 1.0);
        bundle.melody.tonal_conservatism = t;
        bundle.melody.chord_tone_bias = derived_chord_tone_bias(t);
        bundle.melody.neighbor_tone_bias = derived_neighbor_tone_bias(t);
        bundle.melody.passing_tone_bias = derived_passing_tone_bias(t);
        bundle.accompaniment.instrument = ui.accompaniment_instrument;
        bundle.search.seed = Some(ui.seed);
        sanitize_generation_bundle(&mut bundle);
        bundle
    }
}

/// Summary returned after generation for the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompositionSummary {
    pub title: String,
    pub bars: u16,
    pub voice_count: u16,
    pub note_count: u32,
    pub tempo_bpm: f64,
    pub key: u8,
}

/// Export-specific errors (mapped to [`AuroraError::ExportFailed`] at IPC boundary).
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ExportError {
    #[error("IR projection failed: {0}")]
    Projection(String),

    #[error("MIDI export failed: {0}")]
    Midi(String),

    #[error("MusicXML export failed: {0}")]
    MusicXml(String),

    #[error("ABC export failed: {0}")]
    Abc(String),

    #[error("SVG preview failed: {0}")]
    SvgPreview(String),
}
