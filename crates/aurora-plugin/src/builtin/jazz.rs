use std::sync::OnceLock;

use aurora_core::ParameterBundle;

use crate::error::PluginError;
use crate::traits::{Plugin, PluginType, StylePlugin};
use crate::types::{
    HealthStatus, PluginActivation, PluginHealth, StylePreset, StyleResolveRequest,
    StyleResolveResult,
};

pub struct JazzStylePlugin;

impl Plugin for JazzStylePlugin {
    fn id(&self) -> &str {
        "com.aurora.plugins.jazz-style"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Style
    }

    fn parameters(&self) -> &[&'static str] {
        &["harmony.complexity", "harmony.dissonance", "rhythm.syncopation"]
    }
}

impl StylePlugin for JazzStylePlugin {
    fn style_presets(&self) -> &[StylePreset] {
        static PRESETS: OnceLock<Vec<StylePreset>> = OnceLock::new();
        PRESETS.get_or_init(|| {
            vec![
                StylePreset {
                    id: "jazz".into(),
                    display_name: "Jazz Standard".into(),
                    description: "ii-V-I, extensions, swing feel".into(),
                    era: Some("swing".into()),
                    tags: vec!["jazz".into(), "ii-v-i".into()],
                },
                StylePreset {
                    id: "bebop".into(),
                    display_name: "Bebop".into(),
                    description: "Fast harmonic rhythm and chromatic approach".into(),
                    era: Some("bebop".into()),
                    tags: vec!["jazz".into()],
                },
                StylePreset {
                    id: "blues".into(),
                    display_name: "Blues".into(),
                    description: "12-bar blues and dominant seventh vocabulary".into(),
                    era: Some("blues".into()),
                    tags: vec!["blues".into()],
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
        params.style.era = "jazz".into();
        params.harmony.complexity = params.harmony.complexity.max(0.5);
        params.harmony.dissonance = params.harmony.dissonance.max(0.4);
        params.rhythm.syncopation = params.rhythm.syncopation.max(0.4);

        Ok(StyleResolveResult {
            parameters: params,
            active_plugins: vec![PluginActivation {
                plugin_id: self.id().into(),
                priority: 100,
            }],
            active_bundles: vec![
                "JAZZ-*".into(),
                "HARM-*".into(),
                "VL-*".into(),
                "RHY-*".into(),
            ],
            jazz_harmony: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jazz_enables_jazz_bundles() {
        let plugin = JazzStylePlugin;
        let result = plugin
            .resolve_style(&StyleResolveRequest {
                preset_id: "jazz".into(),
                user_overrides: ParameterBundle::default(),
            })
            .unwrap();
        assert!(result.jazz_harmony);
        assert!(result.active_bundles.iter().any(|b| b.starts_with("JAZZ")));
    }
}
