//! Aurora Composer plugin API and host (docs/07-plugin/api.md).

pub mod builtin;
pub mod error;
pub mod host;
pub mod manifest;
pub mod traits;
pub mod types;

pub use error::PluginError;
pub use host::PluginHost;
pub use manifest::PluginManifest;
pub use traits::{Plugin, PluginHostApi, PluginType, StylePlugin};
pub use types::{
    HealthStatus, PluginActivation, PluginDescriptor, PluginHealth, PluginState, StylePreset,
    StyleResolveRequest, StyleResolveResult,
};

pub const API_VERSION: &str = "0.1.0";
pub const HOST_VERSION: &str = "0.1.0";
