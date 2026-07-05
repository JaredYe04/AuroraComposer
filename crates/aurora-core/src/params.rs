use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::progression::ProgressionMode;
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
    pub melody: MelodyParams,
    pub voice: VoiceParams,
    pub texture: TextureParams,
    pub accompaniment: AccompanimentParams,
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
            melody: MelodyParams::default(),
            voice: VoiceParams::default(),
            texture: TextureParams::default(),
            accompaniment: AccompanimentParams::default(),
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
    /// Phrase model: "period", "sentence", or "free".
    pub phrase_model: String,
    /// Measures per phrase (default 4).
    pub phrase_length: u8,
}

impl Default for FormParams {
    fn default() -> Self {
        Self {
            section_count: 2,
            section_lengths: vec![8, 8],
            intro_bars: 0,
            outro_bars: 0,
            phrase_model: "period".into(),
            phrase_length: 4,
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
            repetition_ratio: 0.70,
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
    /// Loop (repeating cell) vs flow (non-repeating arc).
    pub progression_mode: ProgressionMode,
    /// Chords in one loop cell (typically 4).
    pub loop_length: u8,
    /// Weight for last→first chord seam quality in loop mode (0–1).
    pub seam_quality_weight: f32,
}

impl Default for HarmonyParams {
    fn default() -> Self {
        Self {
            complexity: 0.5,
            dissonance: 0.2,
            cadence_strength: 0.7,
            harmonic_rhythm: 0.72,
            progression_mode: ProgressionMode::Loop,
            loop_length: 4,
            seam_quality_weight: 0.85,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MelodyParams {
    /// Bias toward chord tones in candidate pool (0–1).
    pub chord_tone_bias: f32,
    /// Bias toward neighbor tones (0–1).
    pub neighbor_tone_bias: f32,
    /// Bias toward passing tones (0–1).
    pub passing_tone_bias: f32,
    /// Maximum leap in semitones before requiring stepwise return.
    pub leap_limit_semitones: u8,
    /// Target position of melodic climax within phrase (0–1, default 0.65).
    pub climax_ratio: f32,
    /// Weight for motif-realization candidates (0–1).
    pub motif_weight: f32,
    /// Enable double-stop melody candidates (3rds/sixths).
    pub double_stop_enabled: bool,
    /// 0 = chromatic/jazz openness; 1 = strict common-practice consonance.
    pub tonal_conservatism: f32,
}

/// Bias helpers derived from tonal conservatism (t).
#[inline]
pub fn derived_chord_tone_bias(t: f32) -> f32 {
    (0.50 + 0.3077 * t).clamp(0.0, 1.0)
}

#[inline]
pub fn derived_neighbor_tone_bias(t: f32) -> f32 {
    (0.22 - 0.1077 * t).clamp(0.0, 1.0)
}

#[inline]
pub fn derived_passing_tone_bias(t: f32) -> f32 {
    (0.28 - 0.2769 * t).clamp(0.0, 1.0)
}

impl Default for MelodyParams {
    fn default() -> Self {
        let t = 0.75;
        Self {
            chord_tone_bias: derived_chord_tone_bias(t),
            neighbor_tone_bias: derived_neighbor_tone_bias(t),
            passing_tone_bias: derived_passing_tone_bias(t),
            leap_limit_semitones: 6,
            climax_ratio: 0.65,
            motif_weight: 0.72,
            double_stop_enabled: false,
            tonal_conservatism: t,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AccompanimentParams {
    /// Generate block-chord accompaniment voice (piano/strings).
    pub enabled: bool,
    /// `"auto"`, `"piano"`, or `"strings"`.
    pub instrument: String,
    /// 0 = sparse shell; 1 = full triad block voicing.
    pub voicing_density: f32,
    /// Inner voice register bounds.
    pub register_min: u8,
    pub register_max: u8,
}

impl Default for AccompanimentParams {
    fn default() -> Self {
        Self {
            enabled: true,
            instrument: "auto".into(),
            voicing_density: 0.65,
            register_min: 48,
            register_max: 72,
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
    /// Enable HarmonyPad inner voices when homophonic (>0.85 balance).
    pub harmony_pad_enabled: bool,
}

impl Default for TextureParams {
    fn default() -> Self {
        Self {
            homophony_polyphony_balance: 0.5,
            harmony_pad_enabled: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RhythmParams {
    /// Global tempo in BPM (used by structure/tempo map).
    pub tempo_bpm: f64,
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
            tempo_bpm: 120.0,
            density: 0.5,
            syncopation: 0.25,
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
            // Lowered by ~2 octaves so bass sits as true low foundation.
            bass_register_min: 12,
            bass_register_max: 36,
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
    /// Emphasis on downbeats / backbeats (0 = even, 1 = strict kick 1&3 / snare 2&4).
    pub accent_emphasis: f32,
    /// Hi-hat density (0 = sparse quarters, 1 = dense 16ths with openings).
    pub hihat_density: f32,
}

impl Default for DrumsParams {
    fn default() -> Self {
        Self {
            density: 0.5,
            fill_frequency: 0.1,
            pattern_complexity: 0.4,
            accent_emphasis: 0.75,
            hihat_density: 0.6,
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

/// Clamp risky parameter combos so extreme UI settings stay musically usable.
pub fn sanitize_generation_bundle(bundle: &mut ParameterBundle) {
    let mut t = bundle.melody.tonal_conservatism.clamp(0.35, 1.0);
    let c = bundle.harmony.complexity.clamp(0.0, 1.0);

    if t < 0.45 && c > 0.55 {
        t = 0.45;
    }

    bundle.melody.tonal_conservatism = t;
    bundle.melody.chord_tone_bias = derived_chord_tone_bias(t);
    bundle.melody.neighbor_tone_bias = derived_neighbor_tone_bias(t);
    bundle.melody.passing_tone_bias = derived_passing_tone_bias(t);

    if t < 0.6 {
        bundle.harmony.dissonance = bundle.harmony.dissonance.min(0.25);
        bundle.scale.borrowed_chord_tolerance = bundle.scale.borrowed_chord_tolerance.min(0.15);
    }
    if t < 0.5 && c > 0.6 {
        bundle.harmony.complexity = c.min(0.6);
    }

    if bundle.theme.theme_count <= 1 {
        let seed = bundle.search.seed.unwrap_or(42);
        bundle.theme.theme_count = 2 + ((seed % 3) as u8);
    }

    bundle.rhythm.tempo_bpm = bundle.rhythm.tempo_bpm.clamp(40.0, 220.0);
    if bundle.register.bass_register_min > bundle.register.bass_register_max {
        std::mem::swap(
            &mut bundle.register.bass_register_min,
            &mut bundle.register.bass_register_max,
        );
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
            "melody",
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
