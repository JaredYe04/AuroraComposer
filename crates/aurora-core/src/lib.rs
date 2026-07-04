//! Aurora Composer core types: errors, identifiers, parameters, and configuration.
//!
//! See [ACAS v0.1](https://github.com/aurora-composer/docs) and `docs/09-engineering/coding-style.md`.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod config;
mod error;
mod ids;
mod params;
mod summary;

pub use config::SearchConfig;
pub use error::AuroraError;
pub use ids::{JobId, NodeId};
pub use params::{
    CadenceParams, CounterpointParams, DrumsParams, DynamicsParams, EmotionParams, FormParams,
    HarmonyParams, ModeParams, ParameterBundle, RegisterParams, RhythmParams, ScaleParams,
    SearchParams, StyleParams, TextureParams, ThemeParams, VoiceParams,
};
pub use summary::{CompositionSummary, ExportError, UiParameterSnapshot};
