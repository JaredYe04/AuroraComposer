use std::sync::OnceLock;

use aurora_core::ParameterBundle;

use crate::error::PluginError;
use crate::traits::{Plugin, PluginType, StylePlugin};
use crate::types::{
    HealthStatus, PluginActivation, PluginHealth, StylePreset, StyleResolveRequest,
    StyleResolveResult,
};

pub struct ClassicalStylePlugin;

impl Plugin for ClassicalStylePlugin {
    fn id(&self) -> &str {
        "com.aurora.plugins.classical-style"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Style
    }

    fn parameters(&self) -> &[&'static str] {
        &["harmony.complexity", "counterpoint.strictness", "cadence.authentic_weight"]
    }

    fn health(&self) -> PluginHealth {
        PluginHealth {
            status: HealthStatus::Ok,
            message: Some("Classical style pack ready".into()),
            last_invoked: None,
        }
    }
}

impl StylePlugin for ClassicalStylePlugin {
    fn style_presets(&self) -> &[StylePreset] {
        static PRESETS: OnceLock<Vec<StylePreset>> = OnceLock::new();
        PRESETS.get_or_init(|| {
            vec![
                StylePreset {
                    id: "classical".into(),
                    display_name: "Classical".into(),
                    description: "Common-practice tonal harmony and strict counterpoint".into(),
                    era: Some("common-practice".into()),
                    tags: vec!["tonal".into(), "satb".into()],
                },
                StylePreset {
                    id: "baroque".into(),
                    display_name: "Baroque".into(),
                    description: "Figured-bass oriented voice leading".into(),
                    era: Some("baroque".into()),
                    tags: vec!["bach".into()],
                },
            ]
        })
    }

    fn resolve_style(
        &self,
        request: &StyleResolveRequest,
    ) -> Result<StyleResolveResult, PluginError> {
        let mut params = request.user_overrides.clone();
        params.style.genre = request.preset_id.clone();
        params.style.era = "common-practice".into();
        params.harmony.complexity = params.harmony.complexity.clamp(0.2, 0.7);
        params.counterpoint.strictness = params.counterpoint.strictness.max(0.6);
        params.harmony.cadence_strength = params.harmony.cadence_strength.max(0.7);

        Ok(StyleResolveResult {
            parameters: params,
            active_plugins: vec![PluginActivation {
                plugin_id: self.id().into(),
                priority: 100,
            }],
            active_bundles: vec![
                "HARM-*".into(),
                "VL-*".into(),
                "CP-*".into(),
                "FORM-*".into(),
            ],
            jazz_harmony: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classical_preset_resolves() {
        let plugin = ClassicalStylePlugin;
        let result = plugin
            .resolve_style(&StyleResolveRequest {
                preset_id: "classical".into(),
                user_overrides: ParameterBundle::default(),
            })
            .unwrap();
        assert!(!result.jazz_harmony);
        assert!(result.active_bundles.contains(&"HARM-*".into()));
    }
}
