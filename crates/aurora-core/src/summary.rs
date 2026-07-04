use serde::{Deserialize, Serialize};

use crate::params::ParameterBundle;

/// Flat parameter DTO for Phase 2 prototype UI (maps to [`ParameterBundle`]).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UiParameterSnapshot {
    pub key: u8,
    pub style: String,
    pub beam_width: u16,
    pub bars: u16,
    pub tempo_bpm: f64,
}

impl Default for UiParameterSnapshot {
    fn default() -> Self {
        Self {
            key: 0,
            style: "classical".into(),
            beam_width: 8,
            bars: 8,
            tempo_bpm: 120.0,
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
            style: p.style.genre.clone(),
            beam_width: p.search.beam_width,
            bars,
            tempo_bpm: 120.0,
        }
    }
}

impl From<UiParameterSnapshot> for ParameterBundle {
    fn from(ui: UiParameterSnapshot) -> Self {
        let mut bundle = ParameterBundle::default();
        bundle.mode.key = ui.key;
        bundle.style.genre = ui.style;
        bundle.search.beam_width = ui.beam_width;
        bundle.form.section_lengths = vec![ui.bars];
        bundle.form.section_count = 1;
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
