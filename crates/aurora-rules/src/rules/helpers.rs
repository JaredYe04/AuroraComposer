//! Rule factory helpers and shared predicates.

use aurora_ast::{ChordQuality, ChordSymbol, KeySignature, Mode, VoiceRole};

use crate::scale::mode_scale_pcs;

use crate::eval_context::{EvaluationContext, PitchExt};
use crate::rule::{
    EvalCost, HardRule, Rule, RuleCategory, RuleId, RuleMode, RuleScope, SoftEvalOutcome, SoftRule,
};

pub fn stub_soft(id: &str, name: &str, category: RuleCategory) -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: None,
        when: None,
        evaluate: soft_eval_for_category(category),
    }
}

pub fn stub_hard(id: &str, name: &str, category: RuleCategory) -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: hard_check_for(id, category),
        fail_reason: fail_reason_for(id, category),
    }
}

fn soft_eval_for_category(
    category: RuleCategory,
) -> fn(&EvaluationContext<'_>, f64) -> SoftEvalOutcome {
    match category {
        RuleCategory::Harmony => harmony_smart_soft_eval,
        RuleCategory::Counterpoint => counterpoint_smart_soft_eval,
        RuleCategory::VoiceLeading => voice_leading_smart_soft_eval,
        RuleCategory::Rhythm => rhythm_smart_soft_eval,
        RuleCategory::Form | RuleCategory::Motif => motif_smart_soft_eval,
        RuleCategory::Register => register_smart_soft_eval,
        RuleCategory::Jazz => jazz_smart_soft_eval,
        RuleCategory::Orchestration => orchestration_smart_soft_eval,
        _ => neutral_soft_eval,
    }
}

fn hard_check_for(id: &str, category: RuleCategory) -> fn(&EvaluationContext<'_>) -> bool {
    if id.starts_with("REG-") || category == RuleCategory::Register {
        smart_register_hard_check
    } else if id.starts_with("RHY-") || category == RuleCategory::Rhythm {
        smart_rhythm_hard_check
    } else if id.starts_with("CP-PAR-") || id.starts_with("CP-") && category == RuleCategory::Counterpoint {
        smart_cp_parallel_hard_check
    } else {
        permissive_hard_check
    }
}

fn fail_reason_for(
    id: &str,
    category: RuleCategory,
) -> fn(&EvaluationContext<'_>) -> String {
    if id.starts_with("REG-") || category == RuleCategory::Register {
        register_fail_reason
    } else if id.starts_with("RHY-") || category == RuleCategory::Rhythm {
        rhythm_fail_reason
    } else if id.starts_with("CP-PAR-") || id.starts_with("CP-") && category == RuleCategory::Counterpoint {
        cp_parallel_fail_reason
    } else {
        neutral_fail_reason
    }
}

fn harmony_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    let Some(pitch) = ctx.candidate_pitch() else {
        return neutral_soft_eval(ctx, _w);
    };
    if let Some(chord) = &ctx.snapshot.current_chord {
        let pcs = chord_pitch_classes(chord);
        let pc = pitch.pitch_class().pc;
        if pcs.contains(&pc) {
            return SoftEvalOutcome {
                indicator: if ctx.is_strong_beat() { 1.0 } else { 0.6 },
                is_penalty: false,
                reason: "Chord tone on harmony grid".into(),
            };
        }
        if is_diatonic_in_key(chord, &ctx.snapshot.key) {
            return SoftEvalOutcome {
                indicator: 0.3,
                is_penalty: false,
                reason: "Diatonic extension".into(),
            };
        }
        return SoftEvalOutcome {
            indicator: 0.5,
            is_penalty: true,
            reason: "Non-chord tone".into(),
        };
    }
    SoftEvalOutcome {
        indicator: 0.2,
        is_penalty: false,
        reason: "Harmony heuristic (no chord context)".into(),
    }
}

fn counterpoint_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    if parallel_perfect(ctx, 7) || parallel_perfect(ctx, 12) {
        return SoftEvalOutcome {
            indicator: 1.0,
            is_penalty: true,
            reason: "Parallel perfect interval".into(),
        };
    }
    if let Some(interval) = ctx.interval_semitones() {
        let abs = interval.unsigned_abs();
        if abs > 7 {
            let severity = ((abs - 7) as f64 / 5.0).min(1.0);
            return SoftEvalOutcome {
                indicator: severity,
                is_penalty: true,
                reason: format!("Large leap ({interval} semitones)"),
            };
        }
    }
    SoftEvalOutcome {
        indicator: 0.3,
        is_penalty: false,
        reason: "Contrapuntal motion acceptable".into(),
    }
}

fn voice_leading_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    let Some(interval) = ctx.interval_semitones() else {
        return neutral_soft_eval(ctx, _w);
    };
    let abs = interval.unsigned_abs();
    if abs <= 2 {
        SoftEvalOutcome {
            indicator: 1.0,
            is_penalty: false,
            reason: "Stepwise motion".into(),
        }
    } else if abs <= 4 {
        SoftEvalOutcome {
            indicator: 0.4,
            is_penalty: false,
            reason: "Small skip".into(),
        }
    } else {
        SoftEvalOutcome {
            indicator: ((abs - 4) as f64 / 8.0).min(1.0),
            is_penalty: true,
            reason: format!("Leap of {interval} semitones"),
        }
    }
}

fn rhythm_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    let beat = ctx.snapshot.beat_offset.numer as f64 / ctx.snapshot.beat_offset.denom.max(1) as f64;
    let sub = ctx.snapshot.grid_subdivision.max(1);
    if on_grid(beat, sub) {
        SoftEvalOutcome {
            indicator: 1.0,
            is_penalty: false,
            reason: "On rhythmic grid".into(),
        }
    } else {
        SoftEvalOutcome {
            indicator: 0.7,
            is_penalty: true,
            reason: "Off-grid placement".into(),
        }
    }
}

fn motif_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    let sim = motif_similarity(ctx);
    SoftEvalOutcome {
        indicator: sim,
        is_penalty: false,
        reason: format!("Motif similarity {sim:.2}"),
    }
}

fn register_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    let (min, max) = match ctx.voice_role {
        VoiceRole::Bass | VoiceRole::BassLine => ctx.snapshot.bass_register,
        _ => ctx.snapshot.melody_register,
    };
    if register_check(ctx, ctx.voice_role, min, max) {
        SoftEvalOutcome {
            indicator: 0.8,
            is_penalty: false,
            reason: "Within register".into(),
        }
    } else {
        SoftEvalOutcome {
            indicator: 1.0,
            is_penalty: true,
            reason: "Outside voice register".into(),
        }
    }
}

fn jazz_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    if let Some(chord) = &ctx.snapshot.current_chord {
        if !is_diatonic_in_key(chord, &ctx.snapshot.key) {
            return SoftEvalOutcome {
                indicator: 0.5,
                is_penalty: false,
                reason: "Borrowed/chromatic color (jazz tolerance)".into(),
            };
        }
    }
    harmony_smart_soft_eval(ctx, _w)
}

fn orchestration_smart_soft_eval(ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    register_smart_soft_eval(ctx, _w)
}

fn smart_register_hard_check(ctx: &EvaluationContext<'_>) -> bool {
    register_check(
        ctx,
        VoiceRole::Melody,
        ctx.snapshot.melody_register.0,
        ctx.snapshot.melody_register.1,
    ) && register_check(
        ctx,
        VoiceRole::Bass,
        ctx.snapshot.bass_register.0,
        ctx.snapshot.bass_register.1,
    )
}

fn smart_rhythm_hard_check(ctx: &EvaluationContext<'_>) -> bool {
    let beat = ctx.snapshot.beat_offset.numer as f64 / ctx.snapshot.beat_offset.denom.max(1) as f64;
    on_grid(beat, ctx.snapshot.grid_subdivision.max(1))
}

fn smart_cp_parallel_hard_check(ctx: &EvaluationContext<'_>) -> bool {
    !parallel_perfect(ctx, 7) && !parallel_perfect(ctx, 12)
}

fn permissive_hard_check(_ctx: &EvaluationContext<'_>) -> bool {
    true
}

fn register_fail_reason(ctx: &EvaluationContext<'_>) -> String {
    let midi = ctx.candidate_pitch().map(|p| p.midi).unwrap_or(0);
    format!("Pitch MIDI {midi} outside allowed register")
}

fn rhythm_fail_reason(_ctx: &EvaluationContext<'_>) -> String {
    "Event off rhythmic grid".into()
}

fn cp_parallel_fail_reason(_ctx: &EvaluationContext<'_>) -> String {
    "Parallel perfect fifth or octave".into()
}

fn neutral_soft_eval(_ctx: &EvaluationContext<'_>, _w: f64) -> SoftEvalOutcome {
    SoftEvalOutcome {
        indicator: 0.0,
        is_penalty: false,
        reason: "Rule registered (neutral stub)".into(),
    }
}

fn neutral_fail_reason(_ctx: &EvaluationContext<'_>) -> String {
    "Hard rule violation (stub)".into()
}

pub fn chord_pitch_classes(chord: &ChordSymbol) -> Vec<u8> {
    let r = chord.root.pc;
    let mut pcs = vec![r];
    match chord.quality {
        ChordQuality::Major => {
            pcs.extend([(r + 4) % 12, (r + 7) % 12]);
        }
        ChordQuality::Minor => {
            pcs.extend([(r + 3) % 12, (r + 7) % 12]);
        }
        ChordQuality::Dominant7 => {
            pcs.extend([(r + 4) % 12, (r + 7) % 12, (r + 10) % 12]);
        }
        ChordQuality::Major7 => {
            pcs.extend([(r + 4) % 12, (r + 7) % 12, (r + 11) % 12]);
        }
        ChordQuality::Minor7 => {
            pcs.extend([(r + 3) % 12, (r + 7) % 12, (r + 10) % 12]);
        }
        _ => {
            pcs.extend([(r + 4) % 12, (r + 7) % 12]);
        }
    }
    pcs
}

pub fn on_grid(beat: f64, subdivision: u8) -> bool {
    let sub = subdivision.max(1) as f64;
    let step = 1.0 / sub;
    let scaled = beat / step;
    (scaled - scaled.round()).abs() < 1e-6
}

pub fn is_diatonic_in_key(chord: &ChordSymbol, key: &KeySignature) -> bool {
    mode_scale_pcs(key).contains(&chord.root.pc)
}

pub fn primary_dominant_pc(key: &KeySignature) -> u8 {
    (key.tonic.pc + 7) % 12
}

pub fn is_primary_dominant_chord(chord: &ChordSymbol, key: &KeySignature) -> bool {
    matches!(
        chord.quality,
        ChordQuality::Major | ChordQuality::Dominant7 | ChordQuality::Major7
    ) && chord.root.pc == primary_dominant_pc(key)
}

pub fn major_scale(tonic: u8) -> Vec<u8> {
    let pattern = [0, 2, 4, 5, 7, 9, 11];
    pattern.iter().map(|s| (tonic + s) % 12).collect()
}

pub fn leading_tone_pc(key: &KeySignature) -> u8 {
    (key.tonic.pc + 11) % 12
}

pub fn parallel_perfect(ctx: &EvaluationContext<'_>, interval: i16) -> bool {
    let mel_prev = ctx.snapshot.prev_melody_pitch();
    let mel_curr = ctx.candidate_pitch();
    let alto_prev = ctx.snapshot.alto_pitches.last().copied();
    match (mel_prev, mel_curr, alto_prev) {
        (Some(p1), Some(p2), Some(a1)) => {
            let v1_motion = p2.midi as i16 - p1.midi as i16;
            let v2_motion = p2.midi as i16 - a1.midi as i16;
            v1_motion != 0
                && v2_motion != 0
                && v1_motion.signum() == v2_motion.signum()
                && (p2.midi as i16 - a1.midi as i16).rem_euclid(12) as i16 == interval
        }
        _ => false,
    }
}

pub fn motif_similarity(ctx: &EvaluationContext<'_>) -> f64 {
    if let Some(expected) = ctx.snapshot.motif_expected_pitch {
        if let Some(curr) = ctx.candidate_pitch() {
            let diff = (curr.midi as i16 - expected as i16).unsigned_abs();
            return match diff {
                0 => 1.0,
                1 | 2 => 0.82,
                3 | 4 => 0.45,
                _ => 0.12,
            };
        }
    }

    let pitches: Vec<u8> = ctx.snapshot.melody_pitches.iter().map(|p| p.midi).collect();
    if pitches.len() < 2 {
        return 0.0;
    }

    // Penalize exact pitch repetition (monotony)
    let mut repeat_streak = 0usize;
    for w in pitches.windows(2) {
        if w[0] == w[1] {
            repeat_streak += 1;
        }
    }
    let repeat_penalty = if repeat_streak >= 2 {
        0.75
    } else if repeat_streak >= 1 {
        0.45
    } else {
        0.0
    };

    // Penalize long monotonic runs (endless scale-walk)
    let max_monotonic_run = max_same_direction_run(&pitches);
    let monotonic_penalty = match max_monotonic_run {
        n if n >= 6 => 0.65,
        n if n >= 5 => 0.45,
        n if n >= 4 => 0.25,
        _ => 0.0,
    };

    // Penalize cumulative drift from phrase anchor without return
    let drift_penalty = phrase_drift_penalty(&pitches, 16);

    // Interval-pattern similarity: compare consecutive semitone deltas
    let melody_intervals: Vec<i8> = pitches
        .windows(2)
        .map(|w| (w[1] as i16 - w[0] as i16).clamp(-12, 12) as i8)
        .collect();

    // Look for repeating interval patterns (motivic recall)
    let pattern_len = 3.min(melody_intervals.len());
    if pattern_len == 0 {
        return 0.0;
    }

    let seed = &melody_intervals[..pattern_len];
    let mut matches = 0usize;
    let mut comparisons = 0usize;
    for window in melody_intervals.windows(pattern_len).skip(1) {
        comparisons += 1;
        let same_intervals = window
            .iter()
            .zip(seed.iter())
            .filter(|(a, b)| a == b)
            .count();
        let same_direction = window
            .iter()
            .zip(seed.iter())
            .filter(|(a, b)| a.signum() == b.signum())
            .count();
        if same_intervals == pattern_len {
            matches += 1;
        } else if same_direction >= pattern_len - 1 {
            matches += 1;
        }
    }

    if comparisons == 0 {
        // Fallback: stepwise coherence
        let stepwise = melody_intervals.iter().filter(|&&i| i.abs() <= 2).count();
        (stepwise as f64 / melody_intervals.len() as f64 - repeat_penalty - monotonic_penalty - drift_penalty).max(0.0)
    } else {
        (matches as f64 / comparisons as f64 - repeat_penalty - monotonic_penalty - drift_penalty).max(0.0)
    }
}

fn max_same_direction_run(pitches: &[u8]) -> usize {
    if pitches.len() < 2 {
        return 0;
    }
    let mut max_run = 1usize;
    let mut run = 1usize;
    let mut last_sign = 0i8;
    for w in pitches.windows(2) {
        let d = w[1] as i16 - w[0] as i16;
        let sign = d.signum() as i8;
        if sign == 0 {
            run = 1;
            last_sign = 0;
            continue;
        }
        if sign == last_sign {
            run += 1;
            max_run = max_run.max(run);
        } else {
            run = 1;
            last_sign = sign;
        }
    }
    max_run
}

fn phrase_drift_penalty(pitches: &[u8], phrase_len: usize) -> f64 {
    if pitches.len() < phrase_len / 2 {
        return 0.0;
    }
    let plen = phrase_len.max(4);
    let pos = pitches.len() % plen;
    let start = pitches.len().saturating_sub(pos);
    let Some(&anchor) = pitches.get(start) else {
        return 0.0;
    };
    let Some(&current) = pitches.last() else {
        return 0.0;
    };
    let drift = (current as i16 - anchor as i16).unsigned_abs();
    if pos > plen / 2 && drift > 7 {
        0.35
    } else if pos > plen / 3 && drift > 10 {
        0.2
    } else {
        0.0
    }
}

pub fn contour_balance_eval(ctx: &EvaluationContext<'_>) -> SoftEvalOutcome {
    let pitches: Vec<u8> = ctx
        .snapshot
        .melody_pitches
        .iter()
        .map(|p| p.midi)
        .collect();
    let Some(curr) = ctx.candidate_pitch() else {
        return SoftEvalOutcome {
            indicator: 0.0,
            is_penalty: false,
            reason: "No candidate".into(),
        };
    };
    let plen = ctx.snapshot.phrase_length_beats.max(4);
    let pos_in_phrase = pitches.len() % plen;
    let phrase_start = pitches.len().saturating_sub(pos_in_phrase);
    let anchor = pitches.get(phrase_start).copied().unwrap_or(curr.midi);
    let prev = ctx.prev_pitch().map(|p| p.midi);

    let mut penalty = 0.0f64;
    let mut reward = 0.0f64;

    if let Some(prev_midi) = prev {
        let interval = curr.midi as i16 - prev_midi as i16;
        let sign = interval.signum();
        if sign != 0 {
            let mut run = 1usize;
            for w in pitches.windows(2).rev() {
                let s = (w[1] as i16 - w[0] as i16).signum();
                if s == sign {
                    run += 1;
                } else {
                    break;
                }
            }
            if run >= 4 {
                penalty += 0.75;
            } else if run >= 3 {
                penalty += 0.45;
            }
        }
    }

    let dist_anchor = (curr.midi as i16 - anchor as i16).unsigned_abs();
    let on_chord = ctx
        .snapshot
        .current_chord
        .as_ref()
        .map(|ch| {
            use crate::eval_context::PitchExt;
            ch.pitch_classes().contains(&curr.pitch_class().pc)
        })
        .unwrap_or(true);
    if on_chord {
        if dist_anchor <= 2 {
            reward += 0.55;
        } else if dist_anchor <= 5 {
            reward += 0.35;
        } else if dist_anchor <= 8 {
            reward += 0.15;
        }
    } else if dist_anchor <= 3 {
        reward += 0.12;
    }

    if let Some(prev_midi) = prev {
        let abs_iv = (curr.midi as i16 - prev_midi as i16).unsigned_abs();
        if abs_iv <= 2 {
            reward += 0.28;
        } else if abs_iv >= 8 {
            penalty += 0.22;
        }
    }

    let step = ctx.step_index as usize;
    let climax = ctx.snapshot.climax_step;
    if step.abs_diff(climax) <= 2 {
        if let Some(prev_midi) = prev {
            if curr.midi > prev_midi && on_chord {
                reward += 0.32;
            }
        }
    }
    let total = ctx.snapshot.total_melody_steps.max(1);
    let climax_clamped = climax.min(total);
    let progress = step as f64 / total as f64;
    let climax_ratio = climax_clamped as f64 / total as f64;
    if progress > climax_ratio {
        if let Some(prev_midi) = prev {
            if curr.midi < prev_midi {
                reward += 0.35;
            } else if (curr.midi as i16 - prev_midi as i16) > 2 {
                penalty += 0.25;
            }
        }
    }

    if penalty > reward {
        SoftEvalOutcome {
            indicator: penalty.min(1.0),
            is_penalty: true,
            reason: "Monotonic / drift contour".into(),
        }
    } else {
        SoftEvalOutcome {
            indicator: reward.max(0.1),
            is_penalty: false,
            reason: "Balanced contour / return home".into(),
        }
    }
}

/// Rewards melodic closure on tonic at phrase/piece endings.
pub fn melody_closure_eval(ctx: &EvaluationContext<'_>) -> SoftEvalOutcome {
    let Some(curr) = ctx.candidate_pitch() else {
        return SoftEvalOutcome {
            indicator: 0.0,
            is_penalty: false,
            reason: "No candidate".into(),
        };
    };
    let tonic = ctx.snapshot.key.tonic.pc;
    let curr_pc = curr.midi % 12;
    let leading = leading_tone_pc(&ctx.snapshot.key);
    let on_tonic = curr_pc == tonic;
    let prev_leading = ctx
        .prev_pitch()
        .is_some_and(|p| p.midi % 12 == leading);

    if ctx.snapshot.is_piece_end_step {
        if on_tonic {
            return SoftEvalOutcome {
                indicator: 1.0,
                is_penalty: false,
                reason: "Piece ends on tonic".into(),
            };
        }
        return SoftEvalOutcome {
            indicator: 0.85,
            is_penalty: true,
            reason: "Final note must resolve to tonic".into(),
        };
    }

    if ctx.snapshot.phrase_end {
        if on_tonic {
            return SoftEvalOutcome {
                indicator: 0.95,
                is_penalty: false,
                reason: "Phrase closes on tonic".into(),
            };
        }
        if prev_leading && curr.midi == ctx.prev_pitch().unwrap().midi + 1 {
            return SoftEvalOutcome {
                indicator: 0.9,
                is_penalty: false,
                reason: "Leading tone resolves at phrase end".into(),
            };
        }
        return SoftEvalOutcome {
            indicator: 0.55,
            is_penalty: true,
            reason: "Phrase end prefers tonic resolution".into(),
        };
    }

    if ctx.snapshot.in_closure_zone {
        if on_tonic {
            return SoftEvalOutcome {
                indicator: 0.75,
                is_penalty: false,
                reason: "Closure zone tonic".into(),
            };
        }
        if curr_pc == leading {
            return SoftEvalOutcome {
                indicator: 0.5,
                is_penalty: false,
                reason: "Approach tone before closure".into(),
            };
        }
        let dist = (curr.midi as i16 - (60 + tonic as i16)).unsigned_abs();
        if dist <= 4 {
            return SoftEvalOutcome {
                indicator: 0.35,
                is_penalty: false,
                reason: "Near tonic in closure zone".into(),
            };
        }
    }

    SoftEvalOutcome {
        indicator: 0.0,
        is_penalty: false,
        reason: "N/A".into(),
    }
}

pub fn register_check(ctx: &EvaluationContext<'_>, role: VoiceRole, min: u8, max: u8) -> bool {
    let Some(pitch) = ctx.candidate_pitch() else {
        return true;
    };
    if ctx.voice_role != role {
        return true;
    }
    pitch.midi >= min && pitch.midi <= max
}

pub fn is_minor_key(key: &KeySignature) -> bool {
    matches!(
        key.mode,
        Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor
    )
}

pub fn neutral_soft(id: &str, name: &str, category: RuleCategory) -> SoftRule {
    stub_soft(id, name, category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval_context::{AstSnapshot, CandidatePatch, search_note};
    use aurora_ast::{BeatOffset, ChordQuality, ChordSymbol, PitchClass, VoiceId};
    use aurora_core::NodeId;

    fn ctx_with_chord(midi: u8) -> EvaluationContext<'static> {
        static SNAPSHOT: std::sync::OnceLock<AstSnapshot> = std::sync::OnceLock::new();
        static PATCH: std::sync::OnceLock<CandidatePatch> = std::sync::OnceLock::new();
        let snapshot = SNAPSHOT.get_or_init(|| {
            AstSnapshot {
                current_chord: Some(ChordSymbol {
                    root: PitchClass { pc: 0 },
                    quality: ChordQuality::Major,
                    extensions: vec![],
                    bass: None,
                    raw: "C".into(),
                }),
                ..Default::default()
            }
        });
        let patch = PATCH.get_or_init(|| {
            CandidatePatch::single_note(
                VoiceId(0),
                NodeId::new(1, 0),
                search_note(midi, NodeId::new(2, 0)),
                "test",
            )
        });
        EvaluationContext {
            snapshot,
            patch,
            voice_role: VoiceRole::Melody,
            step_index: 0,
        }
    }

    #[test]
    fn harmony_stub_rewards_chord_tone() {
        let ctx = ctx_with_chord(60);
        let outcome = harmony_smart_soft_eval(&ctx, 1.0);
        assert!(!outcome.is_penalty);
        assert!(outcome.indicator > 0.0);
    }

    #[test]
    fn rhythm_stub_detects_off_grid() {
        let snapshot = AstSnapshot {
            beat_offset: BeatOffset::new(1, 3),
            grid_subdivision: 4,
            ..Default::default()
        };
        let patch = CandidatePatch::single_note(
            VoiceId(0),
            NodeId::new(1, 0),
            search_note(60, NodeId::new(2, 0)),
            "test",
        );
        let ctx = EvaluationContext {
            snapshot: &snapshot,
            patch: &patch,
            voice_role: VoiceRole::Melody,
            step_index: 0,
        };
        let outcome = rhythm_smart_soft_eval(&ctx, 1.0);
        assert!(outcome.is_penalty);
    }

    #[test]
    fn stub_hard_register_checks_range() {
        let snapshot = AstSnapshot {
            melody_register: (60, 72),
            ..Default::default()
        };
        let patch = CandidatePatch::single_note(
            VoiceId(0),
            NodeId::new(1, 0),
            search_note(80, NodeId::new(2, 0)),
            "test",
        );
        let ctx = EvaluationContext {
            snapshot: &snapshot,
            patch: &patch,
            voice_role: VoiceRole::Melody,
            step_index: 0,
        };
        assert!(!smart_register_hard_check(&ctx));
    }
}
