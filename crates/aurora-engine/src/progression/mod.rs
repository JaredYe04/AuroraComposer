//! Chord progression planning — loop/flow templates + P3–P5 enrichment + P7 harmonic rhythm.

mod chromatic;
mod flow;
mod harmonic_rhythm;
mod jazz;
mod loop_seam;
mod modulation;
mod mode;
mod roman;
pub mod templates;

pub use chromatic::ChromaticKind;
pub use flow::{plan_flow_progression, tension_curve};
pub use harmonic_rhythm::{
    build_per_beat_chord_grid, chord_at_beat, chords_per_measure, expand_measure_harmony_slots,
    is_harmony_boundary,
};
pub use loop_seam::{score_loop_template, seam_score, select_best_loop_template};
pub use modulation::{plan_key_changes, tonic_at_measure};
pub use mode::{parse_mode, ModeScale};
pub use templates::{
    HarmonicBasis, PlannedChord, ProgressionTemplate, TonicAnchor, select_template,
    template_library, templates_for_mode,
};

use aurora_ast::KeyMap;
use aurora_core::{ParameterBundle, ProgressionMode};

use crate::stages::style_resolver::ResolvedStyle;

/// Context for progression generation.
pub struct ProgressionPlanContext<'a> {
    pub params: &'a ParameterBundle,
    pub style: &'a ResolvedStyle,
    pub key_map: &'a KeyMap,
    pub total_measures: usize,
    pub tension_targets: Vec<f32>,
    pub cadence_measures: Vec<u32>,
}

/// Plan the full chord progression with P3–P5 enrichment passes.
pub fn plan_progression(ctx: &ProgressionPlanContext<'_>) -> Vec<PlannedChord> {
    let tonic = tonic_at_measure(ctx.key_map, 0);
    let loop_mode = ctx.params.harmony.progression_mode == ProgressionMode::Loop;

    let mut chords = if loop_mode {
        plan_loop(ctx, tonic)
    } else {
        plan_flow(ctx, tonic)
    };

    // P3: secondary dominants + borrowed chords
    if ctx.params.harmony.complexity >= 0.35 {
        chromatic::enrich_secondary_dominants(&mut chords, ctx, tonic);
    }
    chromatic::apply_borrowed_substitutions(&mut chords, ctx, tonic);

    // Trim/pad to total_measures after insertions
    chords = normalize_length(chords, ctx.total_measures, ctx, tonic);

    // P4: modulation reharmonization
    modulation::apply_modulation(&mut chords, ctx.key_map, ctx);

    // P5: jazz extensions and substitutions
    jazz::enrich_jazz(&mut chords, ctx, tonic_at_measure(ctx.key_map, 0));

    chords
}

fn normalize_length(
    mut chords: Vec<PlannedChord>,
    target: usize,
    ctx: &ProgressionPlanContext<'_>,
    tonic: u8,
) -> Vec<PlannedChord> {
    if chords.len() > target {
        chords.truncate(target);
    }
    while chords.len() < target {
        if let Some(last) = chords.last().cloned() {
            chords.push(last);
        } else {
            chords.push(roman::make_chord(
                tonic,
                aurora_ast::ChordQuality::Major,
                "I",
                aurora_ast::HarmonicFunction::Tonic,
            ));
        }
    }
    let _ = ctx;
    chords
}

fn plan_loop(ctx: &ProgressionPlanContext<'_>, tonic: u8) -> Vec<PlannedChord> {
    let mode = mode::parse_mode(&ctx.params.mode.mode);
    let genre = ctx.params.style.genre.to_lowercase();

    if ctx.style.jazz_harmony {
        let template = template_library()
            .into_iter()
            .find(|t| t.id == "JAZZ-IIV")
            .unwrap_or_else(|| select_template(&ctx.params.style.genre, true, true));
        return tile_and_cadence(&template, tonic, mode, ctx);
    }

    let mut candidates = templates_for_mode(mode, ctx.params.harmony.complexity, true);
    candidates.retain(|t| {
        if genre.contains("classical") {
            t.style_tags.iter().any(|s| ["classical", "folk"].contains(s))
        } else {
            t.loop_friendly
        }
    });

    if candidates.is_empty() {
        candidates = vec![select_template(&ctx.params.style.genre, false, true)];
    }

    let template = select_best_loop_template(
        &candidates,
        tonic,
        mode,
        ctx.params.harmony.seam_quality_weight,
    );

    tile_and_cadence(&template, tonic, mode, ctx)
}

fn tile_and_cadence(
    template: &ProgressionTemplate,
    tonic: u8,
    mode: aurora_ast::Mode,
    ctx: &ProgressionPlanContext<'_>,
) -> Vec<PlannedChord> {
    let cell = template.realize_mode(tonic, mode);
    if cell.is_empty() {
        return vec![];
    }

    let mut result = Vec::with_capacity(ctx.total_measures);
    for m in 0..ctx.total_measures {
        let idx = m % cell.len();
        result.push(cell[idx].clone());
    }

    apply_cadence_overrides(&mut result, ctx, tonic);
    result
}

fn plan_flow(ctx: &ProgressionPlanContext<'_>, tonic: u8) -> Vec<PlannedChord> {
    let cadence = ctx.cadence_measures.last().copied();
    let mode = mode::parse_mode(&ctx.params.mode.mode);
    let mut result = plan_flow_progression(
        tonic,
        ctx.total_measures,
        &ctx.tension_targets,
        mode,
        ctx.style.jazz_harmony,
        cadence,
    );

    if result.len() < ctx.total_measures {
        let fallback = flow::fallback_flow_template(ctx.style.jazz_harmony);
        let cell = fallback.realize(tonic);
        result.clear();
        for m in 0..ctx.total_measures {
            result.push(cell[m % cell.len()].clone());
        }
    }

    apply_cadence_overrides(&mut result, ctx, tonic);
    result
}

fn apply_cadence_overrides(
    chords: &mut [PlannedChord],
    ctx: &ProgressionPlanContext<'_>,
    tonic: u8,
) {
    use aurora_ast::HarmonicFunction;

    for &global in &ctx.cadence_measures {
        let idx = global as usize;
        if idx >= chords.len() {
            continue;
        }
        let cadence_strength = ctx.params.harmony.cadence_strength;
        if cadence_strength < 0.3 {
            continue;
        }

        if idx > 0 && cadence_strength >= 0.6 {
            let prev = idx - 1;
            if prev < chords.len() {
                chords[prev] = roman::make_chord(
                    (tonic + 7) % 12,
                    aurora_ast::ChordQuality::Major,
                    "V",
                    HarmonicFunction::Dominant,
                );
            }
        }

        chords[idx] = roman::make_chord(
            tonic,
            aurora_ast::ChordQuality::Major,
            "I",
            aurora_ast::HarmonicFunction::Tonic,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_ast::{KeyMap, KeySignature, Mode, PitchClass};
    use aurora_core::ParameterBundle;

    fn test_key_map() -> KeyMap {
        KeyMap {
            default: KeySignature {
                tonic: PitchClass { pc: 0 },
                mode: Mode::Major,
            },
            changes: vec![],
        }
    }

    #[test]
    fn loop_mode_tiles_four_chords() {
        let params = ParameterBundle::default();
        let style = crate::stages::style_resolver::resolve_style(&params);
        let km = test_key_map();
        let ctx = ProgressionPlanContext {
            params: &params,
            style: &style,
            key_map: &km,
            total_measures: 8,
            tension_targets: tension_curve(8, &[]),
            cadence_measures: vec![3, 7],
        };
        let prog = plan_progression(&ctx);
        assert_eq!(prog.len(), 8);
        assert_eq!(prog[0].roman, prog[4].roman);
    }

    #[test]
    fn flow_mode_resolves_on_last() {
        let mut params = ParameterBundle::default();
        params.harmony.progression_mode = ProgressionMode::Flow;
        let style = crate::stages::style_resolver::resolve_style(&params);
        let km = test_key_map();
        let ctx = ProgressionPlanContext {
            params: &params,
            style: &style,
            key_map: &km,
            total_measures: 8,
            tension_targets: tension_curve(8, &[]),
            cadence_measures: vec![7],
        };
        let prog = plan_progression(&ctx);
        assert_eq!(prog[7].function, aurora_ast::HarmonicFunction::Tonic);
    }

    #[test]
    fn chromatic_enrichment_when_complexity_high() {
        let mut params = ParameterBundle::default();
        params.harmony.complexity = 0.8;
        params.scale.borrowed_chord_tolerance = 0.5;
        params.search.seed = Some(3);
        let style = crate::stages::style_resolver::resolve_style(&params);
        let km = test_key_map();
        let ctx = ProgressionPlanContext {
            params: &params,
            style: &style,
            key_map: &km,
            total_measures: 8,
            tension_targets: tension_curve(8, &[]),
            cadence_measures: vec![7],
        };
        let prog = plan_progression(&ctx);
        let has_chromatic = prog.iter().any(|c| {
            c.roman.contains("V7/")
                || c.roman.starts_with("iv")
                || c.roman.starts_with("bVI")
                || c.roman.starts_with("bVII")
                || c.chromatic.is_some()
        });
        assert!(has_chromatic, "expected chromatic enrichment in progression");
    }

    #[test]
    fn pop_axis_seam_scores_high() {
        let library = template_library();
        let axis = library.iter().find(|t| t.id == "POP-AXIS").unwrap();
        assert!(score_loop_template(axis, 0, aurora_ast::Mode::Major, 0.9) > 0.7);
    }
}
