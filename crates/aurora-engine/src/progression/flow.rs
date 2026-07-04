//! Flow-mode progression planner — non-repeating functional arc via DP.

use aurora_ast::{ChordQuality, HarmonicFunction};

use super::templates::{PlannedChord, ProgressionTemplate, template_library};

/// Transition weight between harmonic functions (log-scale bonus).
fn transition_weight(from: HarmonicFunction, to: HarmonicFunction) -> f32 {
    match (from, to) {
        (HarmonicFunction::Tonic, HarmonicFunction::Subdominant) => 0.0,
        (HarmonicFunction::Tonic, HarmonicFunction::Predominant) => -0.1,
        (HarmonicFunction::Tonic, HarmonicFunction::Dominant) => -0.2,
        (HarmonicFunction::Subdominant, HarmonicFunction::Dominant) => 0.3,
        (HarmonicFunction::Subdominant, HarmonicFunction::Tonic) => 0.2,
        (HarmonicFunction::Predominant, HarmonicFunction::Dominant) => 0.4,
        (HarmonicFunction::Dominant, HarmonicFunction::Tonic) => 0.5,
        (HarmonicFunction::Dominant, HarmonicFunction::Subdominant) => -1.2,
        (HarmonicFunction::Tonic, HarmonicFunction::Tonic) => -0.3,
        _ => 0.0,
    }
}

fn function_tension_center(f: HarmonicFunction) -> f32 {
    match f {
        HarmonicFunction::Tonic => 0.15,
        HarmonicFunction::Subdominant => 0.40,
        HarmonicFunction::Predominant => 0.55,
        HarmonicFunction::Dominant => 0.85,
        _ => 0.30,
    }
}

fn tension_fit(chord_fn: HarmonicFunction, target: f32) -> f32 {
    let ideal = function_tension_center(chord_fn);
    1.0 - (ideal - target).abs() * 2.0
}

/// Build a tension target curve for `total_measures` bars.
pub fn tension_curve(total_measures: usize, emotion_curve: &[f32]) -> Vec<f32> {
    if total_measures == 0 {
        return vec![];
    }
    if emotion_curve.len() >= total_measures {
        return emotion_curve[..total_measures].to_vec();
    }
    let mut curve = Vec::with_capacity(total_measures);
    for i in 0..total_measures {
        let t = i as f32 / total_measures.max(1) as f32;
        // Default Schenkerian arc: low → build → peak → resolve
        let base = if t < 0.25 {
            0.2 + t * 0.8
        } else if t < 0.625 {
            0.4 + (t - 0.25) * 0.8
        } else if t < 0.875 {
            0.7 + (t - 0.625) * 0.6
        } else {
            0.1 + (1.0 - t) * 0.3
        };
        let emotion = if emotion_curve.is_empty() {
            0.5
        } else {
            emotion_curve[i % emotion_curve.len()]
        };
        curve.push(base * 0.7 + emotion * 0.3);
    }
    // Force resolution on final measure
    if let Some(last) = curve.last_mut() {
        *last = (*last * 0.3).min(0.15);
    }
    curve
}

/// Diatonic chord pool for flow planning in major key.
fn major_pool() -> Vec<(HarmonicFunction, u8, ChordQuality, &'static str)> {
    vec![
        (HarmonicFunction::Tonic, 0, ChordQuality::Major, "I"),
        (HarmonicFunction::Tonic, 9, ChordQuality::Minor, "vi"),
        (HarmonicFunction::Subdominant, 5, ChordQuality::Major, "IV"),
        (HarmonicFunction::Predominant, 2, ChordQuality::Minor, "ii"),
        (HarmonicFunction::Dominant, 7, ChordQuality::Major, "V"),
    ]
}

fn jazz_pool() -> Vec<(HarmonicFunction, u8, ChordQuality, &'static str)> {
    vec![
        (HarmonicFunction::Tonic, 0, ChordQuality::Major7, "Imaj7"),
        (HarmonicFunction::Tonic, 9, ChordQuality::Minor7, "vim7"),
        (HarmonicFunction::Predominant, 2, ChordQuality::Minor7, "ii7"),
        (HarmonicFunction::Dominant, 7, ChordQuality::Dominant7, "V7"),
        (HarmonicFunction::Dominant, 9, ChordQuality::Dominant7, "VII7"),
    ]
}

/// Plan a non-repeating progression using dynamic programming.
pub fn plan_flow_progression(
    tonic: u8,
    total_measures: usize,
    tension_targets: &[f32],
    jazz: bool,
    cadence_measure: Option<u32>,
) -> Vec<PlannedChord> {
    if total_measures == 0 {
        return vec![];
    }
    let pool = if jazz { jazz_pool() } else { major_pool() };
    let n = pool.len();

    // DP: dp[m][state] = (score, prev_state)
    let mut dp: Vec<Vec<(f32, Option<usize>)>> =
        vec![vec![(f32::NEG_INFINITY, None); n]; total_measures];

    // Initialize measure 0 — prefer tonic
    for (s, (func, ..)) in pool.iter().enumerate() {
        let t = tension_targets.first().copied().unwrap_or(0.2);
        let score = tension_fit(*func, t);
        if *func == HarmonicFunction::Tonic {
            dp[0][s] = (score + 0.3, None);
        } else {
            dp[0][s] = (score, None);
        }
    }

    for m in 1..total_measures {
        let t = tension_targets.get(m).copied().unwrap_or(0.5);
        let is_last = m == total_measures - 1;
        let is_cadence = cadence_measure.map(|c| c as usize == m).unwrap_or(is_last);

        for (s, (func, ..)) in pool.iter().enumerate() {
            let mut best = (f32::NEG_INFINITY, None);
            for (prev_s, (prev_func, ..)) in pool.iter().enumerate() {
                let trans = transition_weight(*prev_func, *func);
                if trans <= -1.0 {
                    continue;
                }
                let prev_score = dp[m - 1][prev_s].0;
                if prev_score <= f32::NEG_INFINITY / 2.0 {
                    continue;
                }
                let mut score = prev_score + trans + tension_fit(*func, t);
                // Cadence: force dominant→tonic resolution
                if is_cadence {
                    if *func == HarmonicFunction::Tonic {
                        score += 0.8;
                    }
                    if *prev_func == HarmonicFunction::Dominant && *func == HarmonicFunction::Tonic {
                        score += 1.0;
                    }
                }
                if score > best.0 {
                    best = (score, Some(prev_s));
                }
            }
            dp[m][s] = best;
        }
    }

    // Backtrack
    let mut state = (0..n)
        .max_by(|&a, &b| {
            dp[total_measures - 1][a]
                .0
                .partial_cmp(&dp[total_measures - 1][b].0)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(0);

    let mut path = vec![state];
    for m in (1..total_measures).rev() {
        let prev = dp[m][state].1.unwrap_or(0);
        path.push(prev);
        state = prev;
    }
    path.reverse();

    path.iter()
        .map(|&s| {
            let (func, offset, quality, roman) = &pool[s];
            PlannedChord {
                symbol: aurora_ast::ChordSymbol {
                    root: aurora_ast::PitchClass {
                        pc: (tonic + offset) % 12,
                    },
                    quality: *quality,
                    extensions: vec![],
                    bass: None,
                    raw: format!(
                        "{}{}",
                        super::templates::chord_root_name((tonic + offset) % 12),
                        super::templates::quality_suffix(*quality)
                    ),
                },
                roman: (*roman).to_string(),
                function: *func,
                chromatic: None,
                rule_ids: vec![],
            }
        })
        .collect()
}

/// Fallback flow template when DP fails.
pub fn fallback_flow_template(jazz: bool) -> ProgressionTemplate {
    template_library()
        .into_iter()
        .find(|t| if jazz { t.id == "JAZZ-IIV" } else { t.id == "CP-CYCLE" })
        .unwrap_or_else(|| template_library().remove(0))
}
