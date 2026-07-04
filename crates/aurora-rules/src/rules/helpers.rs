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
        evaluate: neutral_soft_eval,
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
        check: |_| true,
        fail_reason: neutral_fail_reason,
    }
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
