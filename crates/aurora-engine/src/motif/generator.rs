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
        &[0, 2, 1, -3],
        &[0, 3, -1, -2],
        &[0, -2, 2, 0],
        &[0, 1, 2, -3],
        &[0, 4, -3, -1],
        &[0, 2, -1, -1],
        &[0, -1, 3, -2],
        &[0, 3, -2, -1],
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

    if mode.mode.to_lowercase().contains("major") {
        intervals = snap_to_diatonic(&intervals, true);
    } else if mode.mode.to_lowercase().contains("minor")
        || mode.mode.to_lowercase().contains("dorian")
        || mode.mode.to_lowercase().contains("phrygian")
    {
        intervals = snap_to_diatonic(&intervals, false);
    }

    Motif {
        id: id.to_string(),
        intervals,
        contour,
        rhythm,
        tonic_anchor: tonic,
        base_midi,
    }
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
