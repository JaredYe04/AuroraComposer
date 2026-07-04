//! Aurora Composer generation pipeline (Phase 3 — full 14-stage pipeline).

mod error;
mod motif;
mod orchestrator;
mod progress;
mod progression;
mod stages;

pub use error::EngineError;
pub use orchestrator::{generate_composition, PipelineOrchestrator};
pub use progress::{ProgressCallback, StageProgress};
