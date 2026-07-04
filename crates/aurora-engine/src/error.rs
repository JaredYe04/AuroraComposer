use aurora_core::AuroraError;
use aurora_rules::SearchExhausted;

/// Errors surfaced by the generation pipeline.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Aurora(#[from] AuroraError),

    #[error("pipeline stage {stage} failed: {message}")]
    StageFailed { stage: u8, message: String },
}

impl From<SearchExhausted> for EngineError {
    fn from(_err: SearchExhausted) -> EngineError {
        EngineError::Aurora(AuroraError::SearchExhausted {
            stage: "Melody".into(),
            beam_width: 16,
            relax_suggestions: vec![
                "Increase search.beam_width".into(),
                "Widen register.melody_register_min/max".into(),
            ],
        })
    }
}
