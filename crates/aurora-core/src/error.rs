use serde::Serialize;

use crate::ids::JobId;

/// Typed error surface for Aurora library crates and Tauri IPC.
///
/// See `docs/01-architecture/backend.md` §12.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "code", content = "details")]
pub enum AuroraError {
    #[error("invalid parameters: {0}")]
    InvalidParams(String),

    #[error("constraint unsatisfiable")]
    ConstraintUnsat {
        stage: String,
        rule_ids: Vec<String>,
        reasons: Vec<String>,
        suggestions: Vec<String>,
    },

    #[error("search exhausted")]
    SearchExhausted {
        stage: String,
        beam_width: u32,
        relax_suggestions: Vec<String>,
    },

    #[error("export failed: {format}")]
    ExportFailed {
        format: String,
        diagnostics: Vec<String>,
    },

    #[error("import failed")]
    ImportFailed {
        path: String,
        line: Option<u32>,
        message: String,
    },

    #[error("plugin error: {plugin_id}")]
    PluginError {
        plugin_id: String,
        message: String,
    },

    #[error("job cancelled")]
    JobCancelled { job_id: JobId },

    #[error("patch failed: {0}")]
    PatchFailed(String),

    #[error("serialization failed: {0}")]
    Serialization(String),

    #[error("internal error")]
    Internal { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_serializes_with_code_tag() {
        let err = AuroraError::InvalidParams("beam_width".into());
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "InvalidParams");
    }

    #[test]
    fn internal_error_has_message() {
        let err = AuroraError::Internal {
            message: "unexpected".into(),
        };
        assert!(err.to_string().contains("internal"));
    }
}
