//! P3 — Secondary dominants and modal-interchange (borrowed) chords.

use aurora_ast::HarmonicFunction;

use super::roman::{diatonic_degree, make_chord, secondary_dominant_of};
use super::templates::PlannedChord;
use super::ProgressionPlanContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChromaticKind {
    SecondaryDominant { target_degree: u8 },
    BorrowedMinorIV,
    BorrowedFlatVI,
    BorrowedFlatVII,
}

fn deterministic_hit(seed: u64, index: usize, threshold: f32) -> bool {
    if threshold <= 0.0 {
        return false;
    }
    let h = index
        .wrapping_mul(2654435761)
        .wrapping_add(seed as usize)
        % 1000;
    (h as f32) / 1000.0 < threshold
}

fn secondary_rate(complexity: f32) -> f32 {
    if complexity < 0.35 {
        0.0
    } else {
        0.20 + (complexity - 0.35) * 0.9
    }
}

/// Insert secondary dominants before eligible targets (deterministic).
pub fn enrich_secondary_dominants(
    chords: &mut Vec<PlannedChord>,
    ctx: &ProgressionPlanContext<'_>,
    tonic: u8,
) {
    let rate = secondary_rate(ctx.params.harmony.complexity);
    if rate <= 0.0 {
        return;
    }
    let seed = ctx.params.search.seed.unwrap_or(42);
    let cadence: std::collections::HashSet<usize> =
        ctx.cadence_measures.iter().map(|&m| m as usize).collect();

    let mut replacements: Vec<(usize, PlannedChord)> = Vec::new();
    for (i, chord) in chords.iter().enumerate() {
        if i == 0 || cadence.contains(&i) || cadence.contains(&(i.saturating_sub(1))) {
            continue;
        }
        let Some(degree) = diatonic_degree(&chord.roman) else {
            continue;
        };
        let eligible = matches!(degree, 2 | 4 | 5 | 6) || (degree == 7 && ctx.params.harmony.complexity >= 0.6);
        if !eligible {
            continue;
        }
        if !deterministic_hit(seed, i.wrapping_mul(7), rate) {
            continue;
        }
        let mut sec = secondary_dominant_of(degree, tonic);
        sec.chromatic = Some(ChromaticKind::SecondaryDominant { target_degree: degree });
        sec.rule_ids = vec!["HARM-040".into(), "HARM-041".into()];
        replacements.push((i - 1, sec));
    }

    for (i, sec) in replacements {
        if i < chords.len() {
            chords[i] = sec;
        }
    }
}

/// Replace diatonic slots with borrowed chords from parallel minor.
pub fn apply_borrowed_substitutions(
    chords: &mut [PlannedChord],
    ctx: &ProgressionPlanContext<'_>,
    tonic: u8,
) {
    let tolerance = ctx.params.scale.borrowed_chord_tolerance;
    if tolerance < 0.15 {
        return;
    }
    let seed = ctx.params.search.seed.unwrap_or(42);
    let cadence: std::collections::HashSet<usize> =
        ctx.cadence_measures.iter().map(|&m| m as usize).collect();

    for (i, chord) in chords.iter_mut().enumerate() {
        if cadence.contains(&i) {
            continue;
        }
        let tension = ctx.tension_targets.get(i).copied().unwrap_or(0.5);
        if tension < 0.35 || tension > 0.85 {
            continue;
        }
        let prob = tolerance * 0.35;
        match chord.roman.as_str() {
            "IV" if tolerance >= 0.35
                && deterministic_hit(seed, i.wrapping_mul(3), prob) =>
            {
                let root = (tonic + 5) % 12;
                *chord = make_chord(root, aurora_ast::ChordQuality::Minor, "iv", HarmonicFunction::Subdominant);
                chord.chromatic = Some(ChromaticKind::BorrowedMinorIV);
                chord.rule_ids = vec!["HARM-010".into(), "HARM-045".into()];
            }
            "vi" if tolerance >= 0.45
                && deterministic_hit(seed, i.wrapping_mul(5), prob * 0.7) =>
            {
                let root = (tonic + 8) % 12;
                *chord = make_chord(root, aurora_ast::ChordQuality::Major, "bVI", HarmonicFunction::Subdominant);
                chord.chromatic = Some(ChromaticKind::BorrowedFlatVI);
                chord.rule_ids = vec!["HARM-044".into()];
            }
            "V" if tolerance >= 0.25
                && deterministic_hit(seed, i.wrapping_mul(11), prob * 0.8) =>
            {
                let root = (tonic + 10) % 12;
                *chord = make_chord(root, aurora_ast::ChordQuality::Major, "bVII", HarmonicFunction::Subdominant);
                chord.chromatic = Some(ChromaticKind::BorrowedFlatVII);
                chord.rule_ids = vec!["HARM-044".into()];
            }
            _ => {}
        }
    }
}
