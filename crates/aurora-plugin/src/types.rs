use std::path::PathBuf;

use aurora_core::ParameterBundle;
use serde::{Deserialize, Serialize};

use crate::traits::PluginType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub plugin_type: PluginType,
    pub trust_level: String,
    pub execution_tier: String,
    pub min_engine_version: String,
    pub parameters: Vec<ParameterOverride>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParameterOverride {
    pub key: String,
    pub default_override: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub load_path: PathBuf,
}

/// Serializable plugin summary for Tauri IPC (`list_wasm_plugins`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub execution_tier: String,
    pub state: PluginState,
    pub load_path: PathBuf,
}

impl From<&PluginDescriptor> for PluginInfo {
    fn from(d: &PluginDescriptor) -> Self {
        Self {
            id: d.manifest.id.clone(),
            name: d.manifest.name.clone(),
            version: d.manifest.version.clone(),
            plugin_type: d.manifest.plugin_type,
            execution_tier: d.manifest.execution_tier.clone(),
            state: d.state,
            load_path: d.load_path.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Discovered,
    Loaded,
    Active,
    Error,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Ok,
    Degraded,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_invoked: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StylePreset {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub era: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleResolveRequest {
    pub preset_id: String,
    pub user_overrides: ParameterBundle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleResolveResult {
    pub parameters: ParameterBundle,
    pub active_plugins: Vec<PluginActivation>,
    pub active_bundles: Vec<String>,
    pub jazz_harmony: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginActivation {
    pub plugin_id: String,
    pub priority: u16,
}
