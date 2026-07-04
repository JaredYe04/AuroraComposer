//! Built-in AI plugin stub — adjusts rule weights from emotion parameters.

use std::collections::HashMap;

use aurora_ast::PipelineStageId;
use aurora_core::ParameterBundle;

use crate::error::PluginError;
use crate::traits::{AiPlugin, Plugin, PluginType};
use crate::types::{HealthStatus, PluginHealth};

pub struct AiStubPlugin;

impl Plugin for AiStubPlugin {
    fn id(&self) -> &str {
        "com.aurora.plugins.ai-stub"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Ai
    }

    fn parameters(&self) -> &[&'static str] {
        &[
            "emotion.valence",
            "emotion.arousal",
            "emotion.tension_curve",
        ]
    }

    fn health(&self) -> PluginHealth {
        PluginHealth {
            status: HealthStatus::Ok,
            message: Some("AI stub (offline weight adjustment)".into()),
            last_invoked: None,
        }
    }
}

impl AiPlugin for AiStubPlugin {
    fn target_stages(&self) -> &[PipelineStageId] {
        &[
            PipelineStageId::EmotionResolver,
            PipelineStageId::Melody,
            PipelineStageId::HarmonySkeleton,
        ]
    }

    fn adjust_weights(
        &self,
        base_weights: &HashMap<String, f32>,
        params: &ParameterBundle,
    ) -> Result<HashMap<String, f32>, PluginError> {
        let mut adjusted = base_weights.clone();
        let valence = params.emotion.valence;
        let arousal = params.emotion.arousal;
        let tension = params
            .emotion
            .tension_curve
            .first()
            .copied()
            .unwrap_or(0.5);

        *adjusted
            .entry("HARM-001".into())
            .or_insert(1.0) += lerp(-0.2, 0.3, valence);
        *adjusted
            .entry("VLED-010".into())
            .or_insert(1.0) += lerp(0.15, -0.1, arousal);
        *adjusted
            .entry("RHY-SYNC-001".into())
            .or_insert(1.0) += lerp(-0.1, 0.25, arousal);
        *adjusted
            .entry("HARM-015".into())
            .or_insert(1.0) += lerp(-0.15, 0.25, tension);
        *adjusted
            .entry("harmony.dissonance_tolerance".into())
            .or_insert(1.0) += lerp(-0.2, 0.3, tension);

        Ok(adjusted)
    }

    fn requires_network(&self) -> bool {
        false
    }

    fn requires_api_key(&self) -> bool {
        false
    }
}

fn lerp(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_arousal_boosts_syncopation_weight() {
        let plugin = AiStubPlugin;
        let mut params = ParameterBundle::default();
        params.emotion.arousal = 0.9;
        let base = HashMap::new();
        let adjusted = plugin.adjust_weights(&base, &params).unwrap();
        assert!(adjusted["RHY-SYNC-001"] > 0.0);
    }
}
