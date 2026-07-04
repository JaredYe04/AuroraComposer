//! P5 — Jazz substitutions, extensions, and beam-style enrichment.

use aurora_ast::nodes::Extension;
use aurora_ast::{ChordQuality, HarmonicFunction};

use super::roman::make_chord;
use super::templates::PlannedChord;
use super::ProgressionPlanContext;

fn deterministic_hit(seed: u64, index: usize, threshold: f32) -> bool {
    if threshold <= 0.0 {
        return false;
    }
    let h = index.wrapping_mul(7919).wrapping_add(seed as usize) % 1000;
    (h as f32) / 1000.0 < threshold
}

fn tritone_sub_prob(complexity: f32, jazz: bool) -> f32 {
    if jazz {
        0.25 + complexity * 0.35
    } else if complexity >= 0.6 {
        0.15 + (complexity - 0.6) * 0.4
    } else {
        0.0
    }
}

/// Apply jazz extensions based on complexity/dissonance.
pub fn apply_extensions(chord: &mut PlannedChord, ctx: &ProgressionPlanContext<'_>) {
    let c = ctx.params.harmony.complexity;
    let d = ctx.params.harmony.dissonance;
    let mut ext = Vec::new();

    match chord.symbol.quality {
        ChordQuality::Dominant7 => {
            if c >= 0.4 {
                ext.push(Extension::Ninth);
            }
            if c >= 0.55 {
                ext.push(Extension::Thirteenth);
            }
            if c >= 0.7 && d >= 0.4 {
                ext.push(if d > 0.6 {
                    Extension::Flat9
                } else {
                    Extension::Sharp11
                });
            }
        }
        ChordQuality::Major7 => {
            if c >= 0.4 {
                ext.push(Extension::Ninth);
            }
            if c >= 0.6 {
                ext.push(Extension::Thirteenth);
            }
        }
        ChordQuality::Minor7 => {
            if c >= 0.45 {
                ext.push(Extension::Ninth);
            }
        }
        _ => {}
    }

    if !ext.is_empty() {
        chord.symbol.extensions = ext;
        chord.rule_ids.push("JAZZ-EXT-001".into());
        append_ext_to_raw(&mut chord.symbol.raw, &chord.symbol.extensions);
    }
}

fn append_ext_to_raw(raw: &mut String, extensions: &[Extension]) {
    for e in extensions {
        match e {
            Extension::Ninth | Extension::Add9 => raw.push_str("9"),
            Extension::Flat9 => raw.push_str("b9"),
            Extension::Thirteenth => raw.push_str("13"),
            Extension::Sharp11 => raw.push_str("#11"),
            _ => {}
        }
    }
}

/// Jazz enrichment pass: tritone subs, passing dim, backdoor, extensions.
pub fn enrich_jazz(
    chords: &mut [PlannedChord],
    ctx: &ProgressionPlanContext<'_>,
    tonic: u8,
) {
    let jazz = ctx.style.jazz_harmony;
    let complexity = ctx.params.harmony.complexity;
    if !jazz && complexity < 0.55 {
        return;
    }

    let seed = ctx.params.search.seed.unwrap_or(42);
    let sub_prob = tritone_sub_prob(complexity, jazz);
    let cadence: std::collections::HashSet<usize> =
        ctx.cadence_measures.iter().map(|&m| m as usize).collect();

    let len = chords.len();
    for i in 0..len {
        apply_extensions(&mut chords[i], ctx);

        // Tritone sub: replace V7 with SubV7 (root m2 below)
        if chords[i].roman.starts_with("V7") || chords[i].roman == "V" {
            if deterministic_hit(seed, i.wrapping_mul(13), sub_prob) {
                let orig = chords[i].symbol.root.pc;
                let sub_root = (orig + 6) % 12;
                chords[i].symbol.root.pc = sub_root;
                chords[i].roman = "SubV7".into();
                chords[i].rule_ids.push("JAZZ-SUB-001".into());
            }
        }

        // Passing dim7 between I and ii (insert before ii)
        if i > 0
            && chords[i].roman.starts_with("ii")
            && chords[i - 1].function == HarmonicFunction::Tonic
            && complexity >= 0.55
            && deterministic_hit(seed, i.wrapping_mul(17), 0.25)
        {
            let mut dim = make_chord(
                (chords[i - 1].symbol.root.pc + 1) % 12,
                ChordQuality::Diminished7,
                "#Idim7",
                HarmonicFunction::Dominant,
            );
            dim.rule_ids = vec!["JAZZ-SUB-004".into()];
            // Replace current ii with dim, push ii to next if possible
            if i + 1 < chords.len() {
                chords[i + 1] = chords[i].clone();
            }
            chords[i] = dim;
        }

        // Backdoor cadence before final tonic (Fm7–Bb7–I)
        if cadence.contains(&i)
            && ctx.params.harmony.cadence_strength >= 0.5
            && jazz
            && i >= 2
            && deterministic_hit(seed, i.wrapping_mul(19), 0.3)
        {
            let fm = (tonic + 5) % 12;
            let bb = (tonic + 10) % 12;
            chords[i - 2] = make_chord(fm, ChordQuality::Minor7, "iv7", HarmonicFunction::Subdominant);
            chords[i - 2].rule_ids = vec!["JAZZ-SUB-005".into()];
            chords[i - 1] = make_chord(bb, ChordQuality::Dominant7, "bVII7", HarmonicFunction::Dominant);
            chords[i - 1].rule_ids = vec!["JAZZ-SUB-005".into()];
        }
    }

    // High complexity: use jazz beam-style neighbor swap on flow arcs
    if complexity >= 0.7 && !jazz {
        jazz_beam_touchup(chords, ctx, tonic, seed);
    }
}

/// Lightweight beam-style touchup for high-complexity non-jazz flow.
fn jazz_beam_touchup(
    chords: &mut [PlannedChord],
    ctx: &ProgressionPlanContext<'_>,
    tonic: u8,
    seed: u64,
) {
    for i in 1..chords.len().saturating_sub(1) {
        let t = ctx.tension_targets.get(i).copied().unwrap_or(0.5);
        if t > 0.75 && chords[i].function == HarmonicFunction::Dominant {
            if deterministic_hit(seed, i.wrapping_mul(23), 0.4) {
                // Tritone sub on dominant prolongation
                let sub = (chords[i].symbol.root.pc + 6) % 12;
                chords[i].symbol.root.pc = sub;
                chords[i].roman = "SubV7".into();
                chords[i].rule_ids.push("JAZZ-SUB-002".into());
            }
        }
        // Turnaround fragment at mid-section
        if i == chords.len() / 2 && deterministic_hit(seed, i, 0.35) {
            let vi7 = (tonic + 9) % 12;
            chords[i] = make_chord(vi7, ChordQuality::Dominant7, "VI7", HarmonicFunction::Dominant);
            chords[i].rule_ids.push("JAZZ-IIV-011".into());
        }
    }
}
