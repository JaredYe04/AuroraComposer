use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, PartialEq)]
pub enum PluginError {
    #[error("manifest invalid: {0}")]
    ManifestInvalid(String),
    #[error("api version mismatch: plugin {plugin} vs host {host}")]
    ApiVersionMismatch { plugin: String, host: String },
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("apply failed: {0}")]
    ApplyFailed(String),
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("style preset not found: {0}")]
    PresetNotFound(String),
    #[error("sandbox violation: {0}")]
    SandboxViolation(String),
}
