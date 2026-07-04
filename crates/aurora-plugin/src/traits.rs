use std::collections::HashMap;
use std::sync::Arc;

use aurora_ast::PipelineStageId;
use aurora_core::ParameterBundle;
use serde::{Deserialize, Serialize};

use crate::error::PluginError;
use crate::types::{
    HealthStatus, PluginHealth, StylePreset, StyleResolveRequest, StyleResolveResult,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    Style,
    Harmony,
    Rhythm,
    Theme,
    Ai,
    Export,
}

/// Host API surface exposed to plugins (api.md §10.8).
pub trait PluginHostApi: Send + Sync {
    fn engine_version(&self) -> &str;
    fn log(&self, level: &str, message: &str);
}

/// Base plugin trait (api.md §10.1).
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn version(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn parameters(&self) -> &[&'static str];
    fn on_load(&self, _host: &dyn PluginHostApi) -> Result<(), PluginError> {
        Ok(())
    }
    fn on_unload(&self) -> Result<(), PluginError> {
        Ok(())
    }
    fn health(&self) -> PluginHealth {
        PluginHealth {
            status: HealthStatus::Ok,
            message: None,
            last_invoked: None,
        }
    }
}

/// Style preset resolver (api.md §10.2).
pub trait StylePlugin: Plugin {
    fn style_presets(&self) -> &[StylePreset];
    fn resolve_style(
        &self,
        request: &StyleResolveRequest,
    ) -> Result<StyleResolveResult, PluginError>;
}

/// AI plugin stub — proposes candidates or adjusts weights (api.md §10.6).
pub trait AiPlugin: Plugin {
    fn target_stages(&self) -> &[PipelineStageId];

    fn adjust_weights(
        &self,
        base_weights: &HashMap<String, f32>,
        params: &ParameterBundle,
    ) -> Result<HashMap<String, f32>, PluginError> {
        let _ = params;
        Ok(base_weights.clone())
    }

    fn requires_network(&self) -> bool;
    fn requires_api_key(&self) -> bool;
}

pub type DynStylePlugin = Arc<dyn StylePlugin>;
pub type DynAiPlugin = Arc<dyn AiPlugin>;
