//! Rule factory helpers and shared predicates.

use aurora_ast::{ChordQuality, ChordSymbol, KeySignature, Mode, VoiceRole};

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
    let scale = major_scale(key.tonic.pc);
    scale.contains(&chord.root.pc)
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
    let pitches: Vec<u8> = ctx.snapshot.melody_pitches.iter().map(|p| p.midi).collect();
    if pitches.len() < 2 {
        return 0.0;
    }
    let mut matches = 0usize;
    for w in pitches.windows(2) {
        if w[0] == w[1] {
            matches += 1;
        }
    }
    matches as f64 / (pitches.len() - 1) as f64
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
