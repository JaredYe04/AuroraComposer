use std::collections::HashMap;

use aurora_ast::EmotionProfile;
use aurora_core::{EmotionParams, ParameterBundle};

/// Stage 2 — Emotion Resolver: circumplex → weight deltas + emotion profile.
pub fn resolve_emotion(params: &ParameterBundle) -> (EmotionProfile, HashMap<String, f32>) {
    let EmotionParams {
        valence,
        arousal,
        tension_curve,
    } = &params.emotion;

    let tension = tension_curve.first().copied().unwrap_or(0.5);

    let mut deltas = HashMap::new();
    deltas.insert(
        "HARM-020".into(),
        lerp(-0.4, 0.4, *valence),
    );
    deltas.insert(
        "HARM-025".into(),
        lerp(0.3, -0.3, *valence),
    );
    deltas.insert(
        "tempo.scale".into(),
        lerp(-0.15, 0.25, *arousal),
    );
    deltas.insert(
        "VLED-010".into(),
        lerp(0.2, -0.15, *arousal),
    );
    deltas.insert(
        "harmony.dissonance_tolerance".into(),
        lerp(-0.3, 0.4, tension),
    );
    deltas.insert(
        "HARM-015".into(),
        lerp(-0.2, 0.3, tension),
    );

    let tempo_delta_bpm = *arousal * 12.0 - 6.0;
    let harmonic_color_bias = (*valence - 0.5) * 0.4;

    let profile = EmotionProfile {
        valence: *valence,
        arousal: *arousal,
        weight_deltas: deltas.clone(),
        tempo_delta_bpm,
        harmonic_color_bias,
    };

    (profile, deltas)
}

fn lerp(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_valence_biases_major_preference() {
        let mut params = ParameterBundle::default();
        params.emotion.valence = 0.9;
        let (_, deltas) = resolve_emotion(&params);
        assert!(deltas["HARM-020"] > 0.0);
    }
}
