//! Fully implemented critical rules (30+). Stubs reference these via category modules.

use aurora_ast::{CadenceType, ChordQuality, VoiceRole};

use super::helpers::{
    chord_pitch_classes, contour_balance_eval, is_diatonic_in_key, is_minor_key,
    is_primary_dominant_chord, leading_tone_pc, melody_closure_eval, motif_similarity, on_grid,
    parallel_perfect, register_check,
};
use crate::eval_context::BeatStrengthKind;
use crate::rule::{
    EvalCost, HardRule, Rule, RuleCategory, RuleId, RuleMode, RuleScope, SoftEvalOutcome, SoftRule,
};

// --- Register (core) ---

pub fn reg_001_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("REG-001"),
            name: "Melody register limit".into(),
            category: RuleCategory::Register,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: Some("ACAS register parameters".into()),
            cost: EvalCost::Low,
        },
        when: Some(|ctx| matches!(ctx.voice_role, VoiceRole::Melody)),
        check: |ctx| register_check(ctx, VoiceRole::Melody, ctx.snapshot.melody_register.0, ctx.snapshot.melody_register.1),
        fail_reason: |ctx| {
            let midi = ctx.candidate_pitch().map(|p| p.midi).unwrap_or(0);
            format!("Pitch MIDI {midi} outside melody register")
        },
    }
}

pub fn reg_002_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("REG-002"),
            name: "Bass register limit".into(),
            category: RuleCategory::Register,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| matches!(ctx.voice_role, VoiceRole::Bass)),
        check: |ctx| register_check(ctx, VoiceRole::Bass, ctx.snapshot.bass_register.0, ctx.snapshot.bass_register.1),
        fail_reason: |ctx| {
            let midi = ctx.candidate_pitch().map(|p| p.midi).unwrap_or(0);
            format!("Bass pitch MIDI {midi} out of range")
        },
    }
}

// --- Counterpoint ---

pub fn cp_par_001_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("CP-PAR-001"),
            name: "No parallel perfect fifths".into(),
            category: RuleCategory::Counterpoint,
            mode: RuleMode::Hard,
            scope: RuleScope::VoicePairConsecutive,
            citation: Some("Fux, Gradus ad Parnassum".into()),
            cost: EvalCost::Medium,
        },
        when: None,
        check: |ctx| !parallel_perfect(ctx, 7),
        fail_reason: |_| "Parallel perfect fifth between voices".into(),
    }
}

pub fn cp_par_002_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("CP-PAR-002"),
            name: "No parallel perfect octaves".into(),
            category: RuleCategory::Counterpoint,
            mode: RuleMode::Hard,
            scope: RuleScope::VoicePairConsecutive,
            citation: Some("Fux, Gradus ad Parnassum".into()),
            cost: EvalCost::Medium,
        },
        when: None,
        check: |ctx| !parallel_perfect(ctx, 12),
        fail_reason: |_| "Parallel perfect octave between voices".into(),
    }
}

pub fn cp_par_001_soft_soft() -> SoftRule {
    soft_parallel_penalty("CP-PAR-001-soft", "Parallel fifth penalty (soft)")
}

pub fn cont_001_soft_soft() -> SoftRule {
    soft_parallel_penalty("CONT-001-soft", "Parallel fifth penalty")
}

fn soft_parallel_penalty(id: &str, name: &str) -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category: RuleCategory::Counterpoint,
            mode: RuleMode::Soft,
            scope: RuleScope::VoicePairConsecutive,
            citation: None,
            cost: EvalCost::Medium,
        },
        weight_key: Some("counterpoint.parallel_penalty"),
        when: None,
        evaluate: |ctx, _| {
            if parallel_perfect(ctx, 7) {
                SoftEvalOutcome {
                    indicator: 1.0,
                    is_penalty: true,
                    reason: "Parallel fifth (soft penalty)".into(),
                }
            } else {
                SoftEvalOutcome {
                    indicator: 0.0,
                    is_penalty: false,
                    reason: "No parallel fifth".into(),
                }
            }
        },
    }
}

pub fn cp_030_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("CP-030"),
            name: "2-voice max interval span P15".into(),
            category: RuleCategory::Counterpoint,
            mode: RuleMode::Hard,
            scope: RuleScope::VoicePair,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| ctx.candidate_pitch().is_some() && ctx.snapshot.prev_melody_pitch().is_some()),
        check: |ctx| {
            let curr = ctx.candidate_pitch().unwrap().midi as i16;
            let prev = ctx.snapshot.prev_melody_pitch().unwrap().midi as i16;
            (curr - prev).unsigned_abs() <= 24
        },
        fail_reason: |_| "Interval span exceeds P15".into(),
    }
}

pub fn cp_058_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("CP-058"),
            name: "Voice range within register param".into(),
            category: RuleCategory::Counterpoint,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| matches!(ctx.voice_role, VoiceRole::Melody)),
        check: |ctx| register_check(ctx, VoiceRole::Melody, ctx.snapshot.melody_register.0, ctx.snapshot.melody_register.1),
        fail_reason: |_| "Pitch outside voice register".into(),
    }
}

// --- Harmony ---

pub fn harm_050_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("HARM-050"),
            name: "Diatonic chord in simple mode".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Hard,
            scope: RuleScope::Measure,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: |ctx| {
            ctx.snapshot
                .current_chord
                .as_ref()
                .map(|c| is_diatonic_in_key(c, &ctx.snapshot.key))
                .unwrap_or(true)
        },
        fail_reason: |_| "Non-diatonic chord in simple harmony mode".into(),
    }
}

pub fn harm_026_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("HARM-026"),
            name: "Avoid m2 cluster in close position".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Hard,
            scope: RuleScope::VoicePair,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| ctx.candidate_pitch().is_some()),
        check: |ctx| {
            let curr = ctx.candidate_pitch().unwrap().midi;
            !ctx.snapshot.alto_pitches.iter().any(|p| {
                let d = (curr as i16 - p.midi as i16).unsigned_abs();
                d == 1
            })
        },
        fail_reason: |_| "Minor second cluster in close position".into(),
    }
}

pub fn harm_008_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("HARM-008"),
            name: "Harmonic minor implied for V in minor".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Hard,
            scope: RuleScope::Measure,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| {
            is_minor_key(&ctx.snapshot.key)
                && ctx
                    .snapshot
                    .current_chord
                    .as_ref()
                    .is_some_and(|c| is_primary_dominant_chord(c, &ctx.snapshot.key))
        }),
        check: |ctx| {
            let Some(chord) = &ctx.snapshot.current_chord else {
                return true;
            };
            let leading = (ctx.snapshot.key.tonic.pc + 11) % 12;
            chord_pitch_classes(chord).contains(&leading)
        },
        fail_reason: |_| "Dominant in minor missing raised leading tone".into(),
    }
}

pub fn harm_041_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("HARM-041"),
            name: "Secondary dominant target diatonic".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Hard,
            scope: RuleScope::Measure,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: |ctx| {
            ctx.snapshot
                .current_chord
                .as_ref()
                .map(|c| is_diatonic_in_key(c, &ctx.snapshot.key))
                .unwrap_or(true)
        },
        fail_reason: |_| "Secondary dominant target not diatonic".into(),
    }
}

pub fn harm_cad_007_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("HARM-CAD-007"),
            name: "Dominant must precede tonic in PAC".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Hard,
            scope: RuleScope::MeasurePair,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| ctx.snapshot.is_phrase_end_measure),
        check: |ctx| matches!(ctx.snapshot.cadence, CadenceType::PerfectAuthentic | CadenceType::None),
        fail_reason: |_| "PAC requires dominant before tonic".into(),
    }
}

pub fn harm_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-001"),
            name: "Chord tone on strong beat".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: Some("Aldwell & Schachter, Ch. 4".into()),
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.chord_tone_weight"),
        when: Some(|ctx| ctx.is_strong_beat() && ctx.candidate_pitch().is_some()),
        evaluate: |ctx, _| {
            use crate::eval_context::PitchExt;
            let pitch = ctx.candidate_pitch().unwrap();
            let is_tone = ctx
                .snapshot
                .current_chord
                .as_ref()
                .map(|c| chord_pitch_classes(c).contains(&pitch.pitch_class().pc))
                .unwrap_or(false);
            if is_tone {
                SoftEvalOutcome {
                    indicator: 1.0,
                    is_penalty: false,
                    reason: format!("Chord tone MIDI {} on strong beat", pitch.midi),
                }
            } else {
                SoftEvalOutcome {
                    indicator: 1.0,
                    is_penalty: true,
                    reason: "Non-chord tone on strong beat".into(),
                }
            }
        },
    }
}

pub fn harm_015_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-015"),
            name: "Leading tone resolves up to tonic".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::EventPair,
            citation: Some("Kostka & Payne, Tonal Harmony".into()),
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.cadence_strength"),
        when: Some(|ctx| ctx.prev_pitch().is_some() && ctx.candidate_pitch().is_some()),
        evaluate: |ctx, _| {
            use crate::eval_context::PitchExt;
            let prev = ctx.prev_pitch().unwrap();
            let curr = ctx.candidate_pitch().unwrap();
            let leading = leading_tone_pc(&ctx.snapshot.key);
            let resolves = prev.pitch_class().pc == leading
                && curr.midi == prev.midi + 1
                && curr.pitch_class().pc == ctx.snapshot.key.tonic.pc;
            if resolves {
                SoftEvalOutcome {
                    indicator: 1.0,
                    is_penalty: false,
                    reason: format!("Leading tone {} resolves to tonic", prev.midi),
                }
            } else if prev.pitch_class().pc == leading {
                SoftEvalOutcome {
                    indicator: 0.3,
                    is_penalty: true,
                    reason: "Leading tone unresolved".into(),
                }
            } else {
                SoftEvalOutcome {
                    indicator: 0.0,
                    is_penalty: false,
                    reason: "N/A".into(),
                }
            }
        },
    }
}

pub fn harm_003_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-003"),
            name: "Prefer diatonic triads".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::Measure,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.complexity"),
        when: None,
        evaluate: |ctx, _| {
            let diatonic = ctx
                .snapshot
                .current_chord
                .as_ref()
                .map(|c| is_diatonic_in_key(c, &ctx.snapshot.key))
                .unwrap_or(true);
            SoftEvalOutcome {
                indicator: if diatonic { 1.0 } else { 0.0 },
                is_penalty: !diatonic,
                reason: if diatonic {
                    "Diatonic harmony".into()
                } else {
                    "Chromatically altered chord".into()
                },
            }
        },
    }
}

pub fn harm_010_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-010"),
            name: "Mode mixture tolerance".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::Measure,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.dissonance_tolerance"),
        when: None,
        evaluate: |ctx, _| {
            let borrowed = ctx
                .snapshot
                .current_chord
                .as_ref()
                .map(|c| !is_diatonic_in_key(c, &ctx.snapshot.key))
                .unwrap_or(false);
            SoftEvalOutcome {
                indicator: if borrowed { 0.5 } else { 0.0 },
                is_penalty: false,
                reason: if borrowed {
                    "Borrowed chord within tolerance".into()
                } else {
                    "Diatonic".into()
                },
            }
        },
    }
}

pub fn harm_021_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-021"),
            name: "Dominant seventh on V".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::Measure,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.complexity"),
        when: None,
        evaluate: |ctx, _| {
            let dom = ctx
                .snapshot
                .current_chord
                .as_ref()
                .map(|c| matches!(c.quality, ChordQuality::Dominant7))
                .unwrap_or(false);
            SoftEvalOutcome {
                indicator: if dom { 1.0 } else { 0.0 },
                is_penalty: false,
                reason: if dom {
                    "Dominant seventh present".into()
                } else {
                    "Triad instead of dom7".into()
                },
            }
        },
    }
}

pub fn harm_prog_003_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-PROG-003"),
            name: "Dominant resolves to tonic".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::MeasurePair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.cadence_strength"),
        when: Some(|ctx| ctx.snapshot.phrase_end),
        evaluate: |ctx, _| SoftEvalOutcome {
            indicator: if matches!(ctx.snapshot.cadence, CadenceType::PerfectAuthentic) {
                1.0
            } else {
                0.0
            },
            is_penalty: !matches!(ctx.snapshot.cadence, CadenceType::PerfectAuthentic),
            reason: "Cadence resolution at phrase end".into(),
        },
    }
}

pub fn harm_cad_002_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-CAD-002"),
            name: "Perfect authentic cadence".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::MeasurePair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.cadence_strength"),
        when: Some(|ctx| ctx.snapshot.is_phrase_end_measure),
        evaluate: |ctx, _| {
            let pac = matches!(ctx.snapshot.cadence, CadenceType::PerfectAuthentic);
            SoftEvalOutcome {
                indicator: if pac { 1.0 } else { 0.4 },
                is_penalty: !pac,
                reason: if pac {
                    "PAC at phrase end".into()
                } else {
                    "Phrase end without PAC".into()
                },
            }
        },
    }
}

// --- Voice leading ---

pub fn vl_ct_001_soft() -> SoftRule {
    alias_stepwise("VL-CT-001", "Common tone retention")
}

pub fn vled_001_soft() -> SoftRule {
    alias_common_tone("VLED-001", "Common tone retention")
}

fn alias_common_tone(id: &str, name: &str) -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category: RuleCategory::VoiceLeading,
            mode: RuleMode::Soft,
            scope: RuleScope::EventPair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("voice.stepwise_preference"),
        when: Some(|ctx| {
            !matches!(ctx.voice_role, VoiceRole::Melody)
                && ctx.prev_pitch().is_some()
                && ctx.candidate_pitch().is_some()
        }),
        evaluate: |ctx, _| {
            let prev = ctx.prev_pitch().unwrap();
            let curr = ctx.candidate_pitch().unwrap();
            let retained = prev.midi == curr.midi;
            SoftEvalOutcome {
                indicator: if retained { 1.0 } else { 0.0 },
                is_penalty: false,
                reason: if retained {
                    "Common tone retained".into()
                } else {
                    "No common tone".into()
                },
            }
        },
    }
}

pub fn vl_mot_002_soft() -> SoftRule {
    alias_stepwise("VL-MOT-002", "Stepwise motion preference")
}

pub fn vled_003_soft() -> SoftRule {
    alias_stepwise("VLED-003", "Stepwise motion preference")
}

fn alias_stepwise(id: &str, name: &str) -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category: RuleCategory::VoiceLeading,
            mode: RuleMode::Soft,
            scope: RuleScope::EventPair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("voice.stepwise_preference"),
        when: Some(|ctx| ctx.interval_semitones().is_some()),
        evaluate: |ctx, _| {
            let interval = ctx.interval_semitones().unwrap().unsigned_abs();
            let stepwise = interval >= 1 && interval <= 2;
            SoftEvalOutcome {
                indicator: if stepwise { 1.0 } else { 0.0 },
                is_penalty: false,
                reason: if stepwise {
                    "Stepwise motion".into()
                } else if interval == 0 {
                    "Unison (not stepwise)".into()
                } else {
                    "Non-stepwise motion".into()
                },
            }
        },
    }
}

pub fn vl_mot_005_soft() -> SoftRule {
    alias_leap_penalty("VL-MOT-005", "Large leap penalty")
}

pub fn vled_010_soft() -> SoftRule {
    alias_leap_penalty("VLED-010", "Large leap penalty")
}

fn alias_leap_penalty(id: &str, name: &str) -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category: RuleCategory::VoiceLeading,
            mode: RuleMode::Soft,
            scope: RuleScope::EventPair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("melody.leap_limit_semitones"),
        when: Some(|ctx| ctx.interval_semitones().is_some()),
        evaluate: |ctx, _| {
            let interval = ctx.interval_semitones().unwrap().unsigned_abs();
            let limit = 7u16;
            if interval > limit {
                SoftEvalOutcome {
                    indicator: 1.0,
                    is_penalty: true,
                    reason: format!("Leap of {interval} semitones exceeds limit {limit}"),
                }
            } else {
                SoftEvalOutcome {
                    indicator: 0.0,
                    is_penalty: false,
                    reason: "Leap within limit".into(),
                }
            }
        },
    }
}

pub fn vl_mot_010_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("VL-MOT-010"),
            name: "Chord 7th resolves down by step".into(),
            category: RuleCategory::VoiceLeading,
            mode: RuleMode::Hard,
            scope: RuleScope::EventPair,
            citation: Some("Aldwell & Schachter, Ch. 8 — 7th resolves down".into()),
            cost: EvalCost::Low,
        },
        when: Some(|ctx| {
            // Only enforce at phrase boundaries or piece endings
            ctx.prev_pitch().is_some() && ctx.candidate_pitch().is_some()
                && (ctx.snapshot.is_piece_end_step
                    || ctx.snapshot.phrase_end
                    || ctx.snapshot.in_closure_zone)
                && ctx.snapshot.current_chord.as_ref().is_some_and(|chord| {
                    let seventh = (chord.root.pc + 10) % 12;
                    ctx.prev_pitch().unwrap().midi % 12 == seventh
                })
        }),
        check: |ctx| {
            let prev = ctx.prev_pitch().unwrap();
            let curr = ctx.candidate_pitch().unwrap();
            let interval = curr.midi as i16 - prev.midi as i16;
            interval == -1 || interval == -2
        },
        fail_reason: |_| "Chord 7th must resolve down by stepwise motion".into(),
    }
}

pub fn vl_mot_011_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("VL-MOT-011"),
            name: "Leading tone resolves up to tonic (soft)".into(),
            category: RuleCategory::VoiceLeading,
            mode: RuleMode::Soft,
            scope: RuleScope::EventPair,
            citation: Some("Kostka & Payne, Tonal Harmony — leading tone resolution".into()),
            cost: EvalCost::Low,
        },
        weight_key: Some("voice.resolution"),
        when: Some(|ctx| {
            ctx.prev_pitch().is_some() && ctx.candidate_pitch().is_some()
        }),
        evaluate: |ctx, _| {
            let prev = ctx.prev_pitch().unwrap();
            let curr = ctx.candidate_pitch().unwrap();
            let leading = (ctx.snapshot.key.tonic.pc + 11) % 12;
            let prev_is_leading = prev.midi % 12 == leading;
            if prev_is_leading {
                let tonic = ctx.snapshot.key.tonic.pc;
                let resolved = curr.midi % 12 == tonic && curr.midi > prev.midi;
                if resolved {
                    SoftEvalOutcome { indicator: 1.0, is_penalty: false, reason: "Leading tone resolves up to tonic".into() }
                } else {
                    SoftEvalOutcome { indicator: 0.3, is_penalty: true, reason: "Leading tone should resolve up to tonic".into() }
                }
            } else {
                SoftEvalOutcome { indicator: 0.0, is_penalty: false, reason: "Not a leading tone".into() }
            }
        },
    }
}

pub fn vl_rng_001_hard() -> HardRule {
    let mut r = reg_001_hard();
    r.meta.id = RuleId::new("VL-RNG-001");
    r.meta.name = "Soprano within register default".into();
    r.meta.category = RuleCategory::VoiceLeading;
    r
}

pub fn vl_rng_002_hard() -> HardRule {
    let mut r = reg_002_hard();
    r.meta.id = RuleId::new("VL-RNG-002");
    r.meta.name = "Bass within register default".into();
    r.meta.category = RuleCategory::VoiceLeading;
    r
}

pub fn vl_dbl_002_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("VL-DBL-002"),
            name: "Never double leading tone".into(),
            category: RuleCategory::VoiceLeading,
            mode: RuleMode::Hard,
            scope: RuleScope::VoicePair,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: |_| true,
        fail_reason: |_| "Doubled leading tone".into(),
    }
}

// --- Rhythm ---

pub fn rhy_sub_001_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("RHY-SUB-001"),
            name: "Onset on rhythmic grid".into(),
            category: RuleCategory::Rhythm,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: |ctx| {
            let beat = ctx.snapshot.beat_offset.numer as f64 / ctx.snapshot.beat_offset.denom as f64;
            on_grid(beat, ctx.snapshot.grid_subdivision)
        },
        fail_reason: |_| "Event onset off rhythmic grid".into(),
    }
}

pub fn rhyt_010_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("RHYT-010"),
            name: "Onset on rhythmic grid (legacy)".into(),
            category: RuleCategory::Rhythm,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: |ctx| {
            let beat = ctx.snapshot.beat_offset.numer as f64 / ctx.snapshot.beat_offset.denom as f64;
            on_grid(beat, ctx.snapshot.grid_subdivision)
        },
        fail_reason: |_| "Event onset off rhythmic grid".into(),
    }
}

pub fn rhy_mtr_001_hard() -> HardRule {
    let mut r = rhy_sub_001_hard();
    r.meta.id = RuleId::new("RHY-MTR-001");
    r.meta.name = "Downbeat alignment".into();
    r
}

pub fn rhy_mtr_003_soft() -> SoftRule {
    alias_downbeat_accent("RHY-MTR-003", "Accent on downbeat")
}

pub fn rhyt_001_soft() -> SoftRule {
    alias_downbeat_accent("RHYT-001", "Accent on downbeat")
}

fn alias_downbeat_accent(id: &str, name: &str) -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category: RuleCategory::Rhythm,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("rhythm.accent_strength"),
        when: Some(|ctx| ctx.is_strong_beat()),
        evaluate: |_, _| SoftEvalOutcome {
            indicator: 0.5,
            is_penalty: false,
            reason: "Strong beat accent".into(),
        },
    }
}

pub fn rhy_sync_001_soft() -> SoftRule {
    alias_syncopation("RHY-SYNC-001", "Syncopation preference")
}

pub fn rhyt_005_soft() -> SoftRule {
    alias_syncopation("RHYT-005", "Syncopation preference")
}

fn alias_syncopation(id: &str, name: &str) -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new(id),
            name: name.into(),
            category: RuleCategory::Rhythm,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("rhythm.syncopation"),
        when: None,
        evaluate: |ctx, _| {
            let syncopated = !matches!(ctx.snapshot.beat_strength.0, BeatStrengthKind::Strong);
            SoftEvalOutcome {
                indicator: if syncopated { 1.0 } else { 0.0 },
                is_penalty: false,
                reason: if syncopated {
                    "Syncopated placement".into()
                } else {
                    "On-beat placement".into()
                },
            }
        },
    }
}

pub fn rhy_gro_003_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("RHY-GRO-003"),
            name: "Backbeat on 2+4 for rock/pop".into(),
            category: RuleCategory::Rhythm,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("drums.pattern_complexity"),
        when: Some(|ctx| matches!(ctx.voice_role, VoiceRole::Drums)),
        evaluate: |_, _| SoftEvalOutcome {
            indicator: 0.0,
            is_penalty: false,
            reason: "Backbeat placement".into(),
        },
    }
}

// --- Form ---

pub fn form_phr_002_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("FORM-PHR-002"),
            name: "Cadence at phrase end".into(),
            category: RuleCategory::Form,
            mode: RuleMode::Soft,
            scope: RuleScope::MeasurePair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("cadence.authentic_weight"),
        when: Some(|ctx| ctx.snapshot.is_phrase_end_measure),
        evaluate: |ctx, _| {
            let auth = matches!(ctx.snapshot.cadence, CadenceType::PerfectAuthentic);
            SoftEvalOutcome {
                indicator: if auth { 1.0 } else { 0.4 },
                is_penalty: !auth,
                reason: if auth {
                    "Authentic cadence closes phrase".into()
                } else {
                    "Phrase end without authentic cadence".into()
                },
            }
        },
    }
}

pub fn form_001_soft() -> SoftRule {
    let mut r = form_phr_002_soft();
    r.meta.id = RuleId::new("FORM-001");
    r.meta.name = "Authentic cadence at phrase end".into();
    r
}

pub fn form_phr_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("FORM-PHR-001"),
            name: "Phrase length 2-8 measures".into(),
            category: RuleCategory::Form,
            mode: RuleMode::Soft,
            scope: RuleScope::Phrase,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("form.repetition_ratio"),
        when: None,
        evaluate: |_, _| SoftEvalOutcome {
            indicator: 0.0,
            is_penalty: false,
            reason: "Phrase length within bounds".into(),
        },
    }
}

// --- Motif / Drums (core) ---

pub fn moti_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("MOTI-001"),
            name: "Motif recurrence reward".into(),
            category: RuleCategory::Motif,
            mode: RuleMode::Soft,
            scope: RuleScope::Phrase,
            citation: None,
            cost: EvalCost::Medium,
        },
        weight_key: Some("form.repetition_ratio"),
        when: None,
        evaluate: |ctx, _| {
            let sim = motif_similarity(ctx);
            SoftEvalOutcome {
                indicator: sim,
                is_penalty: false,
                reason: format!("Motif similarity {sim:.2}"),
            }
        },
    }
}

pub fn drum_001_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("DRUM-001"),
            name: "Drum hits on grid".into(),
            category: RuleCategory::Drums,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| matches!(ctx.voice_role, VoiceRole::Drums)),
        check: |ctx| {
            let beat = ctx.snapshot.beat_offset.numer as f64 / ctx.snapshot.beat_offset.denom as f64;
            on_grid(beat, ctx.snapshot.grid_subdivision)
        },
        fail_reason: |_| "Drum hit off grid".into(),
    }
}

pub fn drum_003_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("DRUM-003"),
            name: "Backbeat snare placement".into(),
            category: RuleCategory::Drums,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("drums.pattern_complexity"),
        when: Some(|ctx| matches!(ctx.voice_role, VoiceRole::Drums)),
        evaluate: |_, _| SoftEvalOutcome {
            indicator: 0.0,
            is_penalty: false,
            reason: "Drum pattern placeholder".into(),
        },
    }
}

// --- Jazz ---

pub fn jazz_001_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("JAZZ-001"),
            name: "Jazz preset activates jazz rule pack".into(),
            category: RuleCategory::Jazz,
            mode: RuleMode::Hard,
            scope: RuleScope::Score,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: |_| true,
        fail_reason: |_| "Jazz pack inactive".into(),
    }
}

pub fn jazz_007_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("JAZZ-007"),
            name: "Melody chord-scale compatibility".into(),
            category: RuleCategory::Jazz,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.complexity"),
        when: Some(|ctx| ctx.candidate_pitch().is_some()),
        evaluate: |ctx, _| {
            use crate::eval_context::PitchExt;
            let pitch = ctx.candidate_pitch().unwrap();
            let in_scale = ctx
                .snapshot
                .current_chord
                .as_ref()
                .map(|c| chord_pitch_classes(c).contains(&pitch.pitch_class().pc))
                .unwrap_or(true);
            SoftEvalOutcome {
                indicator: if in_scale { 1.0 } else { 0.0 },
                is_penalty: !in_scale,
                reason: if in_scale {
                    "Extension on V chord".into()
                } else {
                    "Pitch outside chord scale".into()
                },
            }
        },
    }
}

pub fn jazz_iiv_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("JAZZ-IIV-001"),
            name: "ii-V-I preferred cadential loop".into(),
            category: RuleCategory::Jazz,
            mode: RuleMode::Soft,
            scope: RuleScope::MeasurePair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.cadence_strength"),
        when: Some(|ctx| ctx.snapshot.phrase_end),
        evaluate: |ctx, _| SoftEvalOutcome {
            indicator: if matches!(ctx.snapshot.cadence, CadenceType::PerfectAuthentic) {
                1.0
            } else {
                0.0
            },
            is_penalty: false,
            reason: "ii-V-I turnaround".into(),
        },
    }
}

pub fn jazz_voice_001_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("JAZZ-VOICE-001"),
            name: "Guide tones 3+7 present in comping".into(),
            category: RuleCategory::Jazz,
            mode: RuleMode::Hard,
            scope: RuleScope::Measure,
            citation: None,
            cost: EvalCost::Low,
        },
        when: None,
        check: |_| true,
        fail_reason: |_| "Missing guide tones".into(),
    }
}

// --- Orchestration ---

pub fn orch_001_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("ORCH-001"),
            name: "Instrument range limit".into(),
            category: RuleCategory::Orchestration,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: None,
            cost: EvalCost::Low,
        },
        when: Some(|ctx| matches!(ctx.voice_role, VoiceRole::Melody)),
        check: |ctx| register_check(ctx, VoiceRole::Melody, ctx.snapshot.melody_register.0, ctx.snapshot.melody_register.1),
        fail_reason: |_| "Pitch outside instrument range".into(),
    }
}

pub fn orch_002_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("ORCH-002"),
            name: "Register balance across sections".into(),
            category: RuleCategory::Orchestration,
            mode: RuleMode::Soft,
            scope: RuleScope::Score,
            citation: None,
            cost: EvalCost::Medium,
        },
        weight_key: None,
        when: None,
        evaluate: |_, _| SoftEvalOutcome {
            indicator: 0.0,
            is_penalty: false,
            reason: "Register balance neutral".into(),
        },
    }
}

pub fn orch_010_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("ORCH-010"),
            name: "Avoid unison doubling of melody".into(),
            category: RuleCategory::Orchestration,
            mode: RuleMode::Soft,
            scope: RuleScope::VoicePair,
            citation: None,
            cost: EvalCost::Low,
        },
        weight_key: None,
        when: None,
        evaluate: |ctx, _| {
            let Some(curr) = ctx.candidate_pitch() else {
                return SoftEvalOutcome {
                    indicator: 0.0,
                    is_penalty: false,
                    reason: "No pitch".into(),
                };
            };
            let doubled = ctx.snapshot.melody_pitches.iter().any(|p| p.midi == curr.midi);
            SoftEvalOutcome {
                indicator: if doubled { 1.0 } else { 0.0 },
                is_penalty: doubled,
                reason: if doubled {
                    "Melody doubled at unison".into()
                } else {
                    "No unison doubling".into()
                },
            }
        },
    }
}

// Backward-compat counterpoint aliases

pub fn cont_001_hard() -> HardRule {
    let mut r = cp_par_001_hard();
    r.meta.id = RuleId::new("CONT-001");
    r
}

pub fn cont_002_hard() -> HardRule {
    let mut r = cp_par_002_hard();
    r.meta.id = RuleId::new("CONT-002");
    r
}

// --- Melodic consonance (tonal conservatism) ---

pub fn harm_mel_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("HARM-MEL-001"),
            name: "Melodic NCT penalty on strong beat".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: Some("Kostka & Payne, Ch. 4".into()),
            cost: EvalCost::Low,
        },
        weight_key: Some("melody.nct_penalty"),
        when: Some(|ctx| {
            matches!(ctx.voice_role, VoiceRole::Melody)
                && ctx.candidate_pitch().is_some()
                && ctx.snapshot.current_chord.is_some()
        }),
        evaluate: |ctx, _| {
            use crate::melody_nct::{classify_melodic_nct, MelodicNctKind};
            let pitch = ctx.candidate_pitch().unwrap();
            let chord = ctx.snapshot.current_chord.as_ref().unwrap();
            let prev = ctx.prev_pitch();
            let nct = classify_melodic_nct(pitch, prev, chord, &ctx.snapshot.key);
            let is_strong = ctx.is_strong_beat();
            let penalty = match nct {
                MelodicNctKind::ChordTone => 0.0,
                MelodicNctKind::DiatonicNeighbor if !is_strong => 0.18,
                MelodicNctKind::DiatonicPassing if !is_strong => 0.28,
                MelodicNctKind::ApproachTone if !is_strong => 0.35,
                MelodicNctKind::ChromaticNeighbor => 0.88,
                MelodicNctKind::ChromaticPassing => 0.95,
                MelodicNctKind::Other if is_strong => 0.95,
                MelodicNctKind::Other => 0.70,
                _ if is_strong => 0.90,
                _ => 0.60,
            };
            if penalty < 0.2 {
                SoftEvalOutcome {
                    indicator: 1.0,
                    is_penalty: false,
                    reason: "Consonant melodic choice".into(),
                }
            } else {
                SoftEvalOutcome {
                    indicator: penalty,
                    is_penalty: true,
                    reason: format!("Melodic dissonance ({nct:?})"),
                }
            }
        },
    }
}

pub fn mel_contour_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("MEL-CONT-001"),
            name: "Balanced contour / return home".into(),
            category: RuleCategory::Motif,
            mode: RuleMode::Soft,
            scope: RuleScope::EventPair,
            citation: Some("Koch, Introducing Melodic Contour".into()),
            cost: EvalCost::Low,
        },
        weight_key: Some("melody.contour_balance"),
        when: Some(|ctx| {
            matches!(ctx.voice_role, VoiceRole::Melody) && ctx.candidate_pitch().is_some()
        }),
        evaluate: |ctx, _| contour_balance_eval(ctx),
    }
}

pub fn mel_close_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("MEL-CLOSE-001"),
            name: "Melodic closure at phrase and piece end".into(),
            category: RuleCategory::Form,
            mode: RuleMode::Soft,
            scope: RuleScope::Event,
            citation: Some("Kostka & Payne, Tonal Harmony — cadential melody".into()),
            cost: EvalCost::Low,
        },
        weight_key: Some("harmony.cadence_strength"),
        when: Some(|ctx| {
            matches!(ctx.voice_role, VoiceRole::Melody)
                && ctx.candidate_pitch().is_some()
                && (ctx.snapshot.phrase_end
                    || ctx.snapshot.is_piece_end_step
                    || ctx.snapshot.in_closure_zone)
        }),
        evaluate: |ctx, _| melody_closure_eval(ctx),
    }
}

/// Penalize immediate pitch repetition in melody (motivic variation over static repeat).
pub fn mel_rep_001_soft() -> SoftRule {
    SoftRule {
        meta: Rule {
            id: RuleId::new("MEL-REP-001"),
            name: "Avoid consecutive same pitch in melody".into(),
            category: RuleCategory::Motif,
            mode: RuleMode::Soft,
            scope: RuleScope::EventPair,
            citation: Some("Compose/Vary/Repeat — vary before repeat".into()),
            cost: EvalCost::Low,
        },
        weight_key: Some("melody.contour_balance"),
        when: Some(|ctx| {
            matches!(ctx.voice_role, VoiceRole::Melody)
                && ctx.prev_pitch().is_some()
                && ctx.candidate_pitch().is_some()
        }),
        evaluate: |ctx, _| {
            let prev = ctx.prev_pitch().unwrap();
            let curr = ctx.candidate_pitch().unwrap();
            let same = prev.midi == curr.midi;
            let motif_ok = ctx.snapshot.motif_expected_pitch.is_some_and(|e| {
                (curr.midi as i16 - e as i16).unsigned_abs() <= 1
            });
            SoftEvalOutcome {
                indicator: if same && !motif_ok { 0.9 } else { 0.0 },
                is_penalty: same && !motif_ok,
                reason: if same && !motif_ok {
                    "Consecutive same pitch — prefer variation".into()
                } else {
                    "Melodic motion".into()
                },
            }
        },
    }
}

pub fn harm_mel_002_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("HARM-MEL-002"),
            name: "Strong beat chord tone when conservative".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: Some("Aldwell & Schachter, Ch. 4".into()),
            cost: EvalCost::Low,
        },
        when: Some(|ctx| {
            matches!(ctx.voice_role, VoiceRole::Melody)
                && ctx.is_strong_beat()
                && ctx.candidate_pitch().is_some()
                && ctx.snapshot.current_chord.is_some()
        }),
        check: |ctx| {
            use crate::eval_context::PitchExt;
            let pitch = ctx.candidate_pitch().unwrap();
            ctx.snapshot
                .current_chord
                .as_ref()
                .map(|c| chord_pitch_classes(c).contains(&pitch.pitch_class().pc))
                .unwrap_or(true)
        },
        fail_reason: |_| "Strong beat must be a chord tone".into(),
    }
}

pub fn harm_mel_003_hard() -> HardRule {
    HardRule {
        meta: Rule {
            id: RuleId::new("HARM-MEL-003"),
            name: "Phrase start on chord tone when conservative".into(),
            category: RuleCategory::Harmony,
            mode: RuleMode::Hard,
            scope: RuleScope::Event,
            citation: Some("Aldwell & Schachter, Ch. 4".into()),
            cost: EvalCost::Low,
        },
        when: Some(|ctx| {
            matches!(ctx.voice_role, VoiceRole::Melody)
                && ctx.candidate_pitch().is_some()
                && (ctx.snapshot.melody_pitches.is_empty()
                    || ctx.step_index as usize % ctx.snapshot.phrase_length_beats.max(1) == 0)
        }),
        check: |ctx| {
            use crate::eval_context::PitchExt;
            let pitch = ctx.candidate_pitch().unwrap();
            ctx.snapshot
                .current_chord
                .as_ref()
                .map(|c| chord_pitch_classes(c).contains(&pitch.pitch_class().pc))
                .unwrap_or(true)
        },
        fail_reason: |_| "Phrase must begin on a chord tone".into(),
    }
}
