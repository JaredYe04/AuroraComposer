use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Complete user parameter state for generation and reproducibility.
///
/// Categories align with ACAS §6.1 and `docs/02-music-model/score.md` §8.4.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParameterBundle {
    pub version: u16,
    pub emotion: EmotionParams,
    pub style: StyleParams,
    pub mode: ModeParams,
    pub scale: ScaleParams,
    pub form: FormParams,
    pub theme: ThemeParams,
    pub harmony: HarmonyParams,
    pub voice: VoiceParams,
    pub texture: TextureParams,
    pub rhythm: RhythmParams,
    pub dynamics: DynamicsParams,
    pub cadence: CadenceParams,
    pub register: RegisterParams,
    pub counterpoint: CounterpointParams,
    pub drums: DrumsParams,
    pub search: SearchParams,
    pub custom: HashMap<String, Value>,
}

impl Default for ParameterBundle {
    fn default() -> Self {
        Self {
            version: 1,
            emotion: EmotionParams::default(),
            style: StyleParams::default(),
            mode: ModeParams::default(),
            scale: ScaleParams::default(),
            form: FormParams::default(),
            theme: ThemeParams::default(),
            harmony: HarmonyParams::default(),
            voice: VoiceParams::default(),
            texture: TextureParams::default(),
            rhythm: RhythmParams::default(),
            dynamics: DynamicsParams::default(),
            cadence: CadenceParams::default(),
            register: RegisterParams::default(),
            counterpoint: CounterpointParams::default(),
            drums: DrumsParams::default(),
            search: SearchParams::default(),
            custom: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmotionParams {
    pub valence: f32,
    pub arousal: f32,
    pub tension_curve: Vec<f32>,
}

impl Default for EmotionParams {
    fn default() -> Self {
        Self {
            valence: 0.5,
            arousal: 0.5,
            tension_curve: vec![0.5],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StyleParams {
    pub genre: String,
    pub era: String,
    pub orchestration_preset: String,
}

impl Default for StyleParams {
    fn default() -> Self {
        Self {
            genre: "pop".into(),
            era: "contemporary".into(),
            orchestration_preset: "lead_plus_accompaniment".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModeParams {
    pub key: u8,
    pub mode: String,
    pub modulation_policy: String,
}

impl Default for ModeParams {
    fn default() -> Self {
        Self {
            key: 0,
            mode: "major".into(),
            modulation_policy: "conservative".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScaleParams {
    pub scale_type: String,
    pub borrowed_chord_tolerance: f32,
}

impl Default for ScaleParams {
    fn default() -> Self {
        Self {
            scale_type: "diatonic".into(),
            borrowed_chord_tolerance: 0.2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FormParams {
    pub section_count: u8,
    pub section_lengths: Vec<u16>,
    pub intro_bars: u16,
    pub outro_bars: u16,
}

impl Default for FormParams {
    fn default() -> Self {
        Self {
            section_count: 2,
            section_lengths: vec![8, 8],
            intro_bars: 0,
            outro_bars: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeParams {
    pub theme_count: u8,
    pub motif_length: u8,
    pub repetition_ratio: f32,
}

impl Default for ThemeParams {
    fn default() -> Self {
        Self {
            theme_count: 1,
            motif_length: 4,
            repetition_ratio: 0.6,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HarmonyParams {
    pub complexity: f32,
    pub dissonance: f32,
    pub cadence_strength: f32,
    pub harmonic_rhythm: f32,
}

impl Default for HarmonyParams {
    fn default() -> Self {
        Self {
            complexity: 0.5,
            dissonance: 0.3,
            cadence_strength: 0.7,
            harmonic_rhythm: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VoiceParams {
    pub voice_count: u8,
    pub density: f32,
    pub register_min: u8,
    pub register_max: u8,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            voice_count: 4,
            density: 0.5,
            register_min: 36,
            register_max: 84,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextureParams {
    pub homophony_polyphony_balance: f32,
}

impl Default for TextureParams {
    fn default() -> Self {
        Self {
            homophony_polyphony_balance: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RhythmParams {
    pub density: f32,
    pub syncopation: f32,
    pub subdivision: u8,
    pub swing: f32,
    pub time_signature_beats: u8,
    pub time_signature_beat_type: u8,
}

impl Default for RhythmParams {
    fn default() -> Self {
        Self {
            density: 0.5,
            syncopation: 0.3,
            subdivision: 4,
            swing: 0.0,
            time_signature_beats: 4,
            time_signature_beat_type: 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DynamicsParams {
    pub dynamic_range: f32,
    pub accent_strength: f32,
}

impl Default for DynamicsParams {
    fn default() -> Self {
        Self {
            dynamic_range: 0.6,
            accent_strength: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CadenceParams {
    pub cadence_type_preference: String,
    pub half_cadence_freq: f32,
}

impl Default for CadenceParams {
    fn default() -> Self {
        Self {
            cadence_type_preference: "authentic".into(),
            half_cadence_freq: 0.2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegisterParams {
    pub melody_register_min: u8,
    pub melody_register_max: u8,
    pub bass_register_min: u8,
    pub bass_register_max: u8,
}

impl Default for RegisterParams {
    fn default() -> Self {
        Self {
            melody_register_min: 60,
            melody_register_max: 84,
            bass_register_min: 36,
            bass_register_max: 60,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CounterpointParams {
    pub strictness: f32,
    pub parallel_penalty: f32,
}

impl Default for CounterpointParams {
    fn default() -> Self {
        Self {
            strictness: 0.5,
            parallel_penalty: 0.8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DrumsParams {
    pub density: f32,
    pub fill_frequency: f32,
    pub pattern_complexity: f32,
}

impl Default for DrumsParams {
    fn default() -> Self {
        Self {
            density: 0.5,
            fill_frequency: 0.1,
            pattern_complexity: 0.4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchParams {
    pub beam_width: u16,
    pub temperature: f32,
    pub max_iterations: u32,
    pub seed: Option<u64>,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            beam_width: 16,
            temperature: 1.0,
            max_iterations: 10_000,
            seed: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_bundle_json_roundtrip() {
        let bundle = ParameterBundle::default();
        let json = serde_json::to_string(&bundle).unwrap();
        let parsed: ParameterBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.search.beam_width, 16);
        assert_eq!(parsed.form.section_count, 2);
    }

    #[test]
    fn all_acas_categories_present() {
        let bundle = ParameterBundle::default();
        let json = serde_json::to_value(&bundle).unwrap();
        for key in [
            "emotion",
            "style",
            "mode",
            "scale",
            "form",
            "theme",
            "harmony",
            "voice",
            "texture",
            "rhythm",
            "dynamics",
            "cadence",
            "register",
            "counterpoint",
            "drums",
            "search",
        ] {
            assert!(json.get(key).is_some(), "missing category {key}");
        }
    }
}
