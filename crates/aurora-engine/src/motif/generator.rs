//! Deterministic motif seed generation from scale and parameters.

use aurora_core::{ModeParams, ThemeParams};

use super::{Motif, MotifDur, Semitone};

/// Diatonic step sizes in major (semitones).
const MAJOR_STEPS: [i8; 7] = [2, 2, 1, 2, 2, 2, 1];

const RHYTHM_TEMPLATES: &[&[MotifDur]] = &[
    &[
        MotifDur::Quarter,
        MotifDur::TwoEighths,
        MotifDur::TwoEighths,
        MotifDur::SyncopatedEighth,
    ],
    &[
        MotifDur::TwoEighths,
        MotifDur::Quarter,
        MotifDur::SyncopatedEighth,
        MotifDur::TwoEighths,
    ],
    &[
        MotifDur::DottedQuarter,
        MotifDur::TwoEighths,
        MotifDur::Quarter,
        MotifDur::RestThenEighth,
    ],
    &[
        MotifDur::Quarter,
        MotifDur::SyncopatedEighth,
        MotifDur::TwoEighths,
        MotifDur::DottedQuarter,
    ],
    &[
        MotifDur::TwoEighths,
        MotifDur::TwoEighths,
        MotifDur::DottedQuarter,
        MotifDur::Quarter,
    ],
    &[
        MotifDur::SyncopatedEighth,
        MotifDur::Quarter,
        MotifDur::TwoEighths,
        MotifDur::TwoEighths,
    ],
];

/// Generate a motif seed from theme parameters, key, and RNG seed.
pub fn generate_motif(
    id: &str,
    theme: &ThemeParams,
    mode: &ModeParams,
    theme_index: u32,
    seed: u64,
) -> Motif {
    let len = theme.motif_length.max(2).min(8) as usize;
    let tonic = mode.key % 12;
    let base_midi = 60 + tonic;

    let patterns: &[&[i8]] = &[
        &[0, 2, 1, -1, 0],      // tonic→mediant→supertonic→tonic→tonic (classic arch)
        &[0, -1, 2, 1, 0],      // tonic→leading→mediant→supertonic→tonic
        &[0, 2, 4, 3, 1, 0],    // tonic→mediant→dominant→subdominant→supertonic→tonic
        &[0, 1, 3, 4, 3, 2, 0], // tonic→supertonic→subdominant→dominant→subdominant→mediant→tonic
        &[0, 4, 0, 4, 0],       // tonic→dominant→tonic→dominant→tonic (horn fifth)
        &[0, 1, 0, 2, 0],       // tonic→supertonic→tonic→mediant→tonic (neighbor emphasis)
        &[0, 2, 4, 2, 0],       // tonic→mediant→dominant→mediant→tonic (pure arch)
        &[0, -1, 0, 2, 1, 0],   // tonic→leading→tonic→mediant→supertonic→tonic
        &[0, 3, 4, 3, 2, 0],    // tonic→subdominant→dominant→subdominant→mediant→tonic
        &[0, 2, 1, 2, 4, 3, 2, 0], // longer: tonic→mediant→supertonic→mediant→dominant→subdominant→mediant→tonic
        &[0, 4, 5, 4, 2, 0],    // tonic→dominant→submediant→dominant→mediant→tonic
        &[0, 1, 4, 4, 3, 1, 0], // tonic→supertonic→dominant→dominant→subdominant→supertonic→tonic
    ];
    let mix = seed
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(u64::from(theme_index).wrapping_mul(0x85EB_CA6B));
    let pattern_idx = (mix % patterns.len() as u64) as usize;
    let rhythm_idx = ((mix >> 16) % RHYTHM_TEMPLATES.len() as u64) as usize;
    let pattern = patterns[pattern_idx];
    let rhythm_template = RHYTHM_TEMPLATES[rhythm_idx];

    let mut intervals: Vec<Semitone> = Vec::with_capacity(len);
    let mut contour: Vec<i8> = Vec::with_capacity(len);
    let mut rhythm: Vec<MotifDur> = Vec::with_capacity(len);
    for i in 0..len {
        let iv = pattern[i % pattern.len()];
        intervals.push(iv);
        rhythm.push(rhythm_template[i % rhythm_template.len()]);
        if i > 0 {
            contour.push(iv.signum());
        }
    }
    if contour.is_empty() {
        contour.push(0);
    }

    // Convert scale-degree patterns to mode-appropriate semitones
    let mode_steps = mode_step_pattern(&mode.mode);
    intervals = scale_degrees_to_semitones(&intervals, mode_steps);

    Motif {
        id: id.to_string(),
        intervals,
        contour,
        rhythm,
        tonic_anchor: tonic,
        base_midi,
    }
}

fn mode_step_pattern(mode_name: &str) -> &[i8] {
    match mode_name.to_lowercase().as_str() {
        "major" | "ionian" => &[2, 2, 1, 2, 2, 2, 1],
        "dorian" => &[2, 1, 2, 2, 2, 1, 2],
        "phrygian" => &[1, 2, 2, 2, 1, 2, 2],
        "lydian" => &[2, 2, 2, 1, 2, 2, 1],
        "mixolydian" => &[2, 2, 1, 2, 2, 1, 2],
        "natural minor" | "aeolian" => &[2, 1, 2, 2, 1, 2, 2],
        "harmonic minor" => &[2, 1, 2, 2, 1, 3, 1],
        "melodic minor" => &[2, 1, 2, 2, 2, 2, 1],
        _ => &[2, 2, 1, 2, 2, 2, 1], // default to major
    }
}

fn scale_degrees_to_semitones(degrees: &[Semitone], mode_steps: &[i8]) -> Vec<Semitone> {
    let mut result = Vec::with_capacity(degrees.len());
    let mut current_degree: i8 = 0;  // Current scale degree position (0-6)
    
    for &delta in degrees {
        let semitone_delta = scale_delta_to_semitone(delta, mode_steps, &mut current_degree);
        result.push(semitone_delta);
    }
    result
}

/// Convert a scale-degree delta to a semitone delta, tracking current degree position.
/// delta=+2 in major from degree 0 means: mode_steps[0] + mode_steps[1] = 2 + 2 = 4 semitones.
fn scale_delta_to_semitone(delta: i8, mode_steps: &[i8], current_degree: &mut i8) -> i8 {
    if delta == 0 {
        return 0;
    }
    
    let direction = delta.signum();
    let steps = delta.unsigned_abs() as usize;
    let mut semitones: i8 = 0;
    
    for _ in 0..steps {
        let idx = if direction > 0 {
            // Going up: use the step from current degree to next
            current_degree.rem_euclid(7) as usize
        } else {
            // Going down: use the step from previous degree to current
            // (previous degree is the one below current)
            let prev_degree = (*current_degree - 1).rem_euclid(7) as usize;
            prev_degree
        };
        semitones += mode_steps[idx] * direction;
        *current_degree = (*current_degree + direction).rem_euclid(7);
    }
    
    semitones
}

fn snap_to_diatonic(intervals: &[Semitone], major: bool) -> Vec<Semitone> {
    let diatonic_steps: &[i8] = if major {
        &[1, 2, 3, 4, 5, 7, 8, 9, 10, 12]
    } else {
        &[1, 2, 3, 4, 5, 7, 8, 9, 10]
    };
    let mut result = vec![0i8];
    for &target in intervals.iter().skip(1) {
        let sign = if target >= 0 { 1i8 } else { -1i8 };
        let mut best = target.clamp(-12, 12);
        let mut best_dist = i8::MAX;
        for &step in diatonic_steps {
            for s in [sign, -sign] {
                let candidate = step * s;
                let dist = (candidate - target).unsigned_abs();
                if dist < best_dist as u8 {
                    best_dist = dist as i8;
                    best = candidate;
                }
            }
        }
        result.push(best);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::{ModeParams, ThemeParams};

    #[test]
    fn generates_motif_with_rhythm() {
        let theme = ThemeParams {
            motif_length: 4,
            ..Default::default()
        };
        let mode = ModeParams::default();
        let motif = generate_motif("test", &theme, &mode, 0, 42);
        assert_eq!(motif.intervals.len(), 4);
        assert_eq!(motif.rhythm.len(), 4);
    }

    #[test]
    fn different_seeds_produce_different_interval_patterns() {
        let theme = ThemeParams {
            motif_length: 4,
            ..Default::default()
        };
        let mode = ModeParams::default();
        let a = generate_motif("a", &theme, &mode, 0, 1);
        let b = generate_motif("b", &theme, &mode, 0, 999);
        assert_ne!(a.intervals, b.intervals);
    }
}
