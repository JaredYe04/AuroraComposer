use std::sync::OnceLock;

use aurora_core::ParameterBundle;

use crate::error::PluginError;
use crate::traits::{Plugin, PluginType, StylePlugin};
use crate::types::{PluginActivation, StylePreset, StyleResolveRequest, StyleResolveResult};

pub struct PopStylePlugin;

impl Plugin for PopStylePlugin {
    fn id(&self) -> &str {
        "com.aurora.plugins.pop-style"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Style
    }

    fn parameters(&self) -> &[&'static str] {
        &["harmony.complexity", "rhythm.syncopation", "form.repetition_ratio"]
    }
}

impl StylePlugin for PopStylePlugin {
    fn style_presets(&self) -> &[StylePreset] {
        static PRESETS: OnceLock<Vec<StylePreset>> = OnceLock::new();
        PRESETS.get_or_init(|| {
            vec![
                StylePreset {
                    id: "pop".into(),
                    display_name: "Pop".into(),
                    description: "Verse-chorus loops and syncopated rhythm".into(),
                    era: Some("contemporary".into()),
                    tags: vec!["pop".into()],
                },
                StylePreset {
                    id: "rock".into(),
                    display_name: "Rock".into(),
                    description: "Backbeat groove and power-chord harmony".into(),
                    era: Some("rock".into()),
                    tags: vec!["rock".into()],
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
        params.style.era = "contemporary".into();
        params.harmony.complexity = params.harmony.complexity.clamp(0.3, 0.6);
        params.rhythm.syncopation = params.rhythm.syncopation.max(0.3);
        params.theme.repetition_ratio = params.theme.repetition_ratio.max(0.6);

        Ok(StyleResolveResult {
            parameters: params,
            active_plugins: vec![PluginActivation {
                plugin_id: self.id().into(),
                priority: 100,
            }],
            active_bundles: vec![
                "HARM-*".into(),
                "RHY-*".into(),
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
    fn pop_resolves_rhythm_bundle() {
        let plugin = PopStylePlugin;
        let result = plugin
            .resolve_style(&StyleResolveRequest {
                preset_id: "pop".into(),
                user_overrides: ParameterBundle::default(),
            })
            .unwrap();
        assert!(result.active_bundles.iter().any(|b| b.starts_with("RHY")));
    }
}
