//! Loop seam scoring — evaluates last→first chord transition quality.

use aurora_ast::HarmonicFunction;

use super::templates::{PlannedChord, ProgressionTemplate, TonicAnchor};

/// Score the wrap-around transition from last chord to first in a loop (0.0–1.0).
pub fn seam_score(last: &PlannedChord, first: &PlannedChord) -> f32 {
    seam_score_functions(last.function, first.function, &last.roman, &first.roman)
}

fn seam_score_functions(
    last_fn: HarmonicFunction,
    first_fn: HarmonicFunction,
    last_roman: &str,
    first_roman: &str,
) -> f32 {
    // Strong plagal / authentic loop seams
    if last_roman.starts_with("IV") && first_roman.starts_with('I') {
        return 1.0;
    }
    if last_fn == HarmonicFunction::Dominant && first_fn == HarmonicFunction::Tonic {
        return 0.95;
    }
    // Deceptive loop (pop axis energy)
    if last_fn == HarmonicFunction::Dominant && first_roman.starts_with("vi") {
        return 0.85;
    }
    // Subdominant → tonic
    if last_fn == HarmonicFunction::Subdominant && first_fn == HarmonicFunction::Tonic {
        return 0.80;
    }
    // Tonic → relative minor entry
    if last_fn == HarmonicFunction::Tonic && first_roman.starts_with("vi") {
        return 0.70;
    }
    // Predominant → dominant (ii→V loop start)
    if last_fn == HarmonicFunction::Predominant && first_fn == HarmonicFunction::Dominant {
        return 0.75;
    }
    // Functional retrogression (D→S) — weak seam
    if last_fn == HarmonicFunction::Dominant && first_fn == HarmonicFunction::Subdominant {
        return 0.30;
    }
    // Same function continuation
    if last_fn == first_fn {
        return 0.55;
    }
    0.50
}

/// Score a full template including wrap-around seam.
pub fn score_loop_template(
    template: &ProgressionTemplate,
    tonic: u8,
    mode: aurora_ast::Mode,
    seam_weight: f32,
) -> f32 {
    let chords = template.realize_mode(tonic, mode);
    if chords.is_empty() {
        return 0.0;
    }
    let mut score = 0.5f32;
    if chords.len() >= 2 {
        if let (Some(last), Some(first)) = (chords.last(), chords.first()) {
            score = seam_score(last, first) * seam_weight + 0.5 * (1.0 - seam_weight);
        }
    }
    if template.loop_friendly {
        score += 0.1;
    }
    score += tonic_anchor_bonus(template, mode);
    score
}

fn tonic_anchor_bonus(template: &ProgressionTemplate, mode: aurora_ast::Mode) -> f32 {
    use aurora_ast::Mode;
    let minorish = matches!(
        mode,
        Mode::NaturalMinor
            | Mode::HarmonicMinor
            | Mode::MelodicMinor
            | Mode::Dorian
            | Mode::Phrygian
            | Mode::Locrian
    );
    match template.tonic_anchor {
        TonicAnchor::RelativeMinor if minorish => 0.18,
        TonicAnchor::Major if !minorish => 0.18,
        TonicAnchor::Any => 0.05,
        _ => 0.0,
    }
}

/// Pick the best loop template from candidates.
pub fn select_best_loop_template(
    candidates: &[ProgressionTemplate],
    tonic: u8,
    mode: aurora_ast::Mode,
    seam_weight: f32,
) -> ProgressionTemplate {
    candidates
        .iter()
        .max_by(|a, b| {
            score_loop_template(a, tonic, mode, seam_weight)
                .partial_cmp(&score_loop_template(b, tonic, mode, seam_weight))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or_else(|| candidates[0].clone())
}
