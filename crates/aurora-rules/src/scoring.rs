use aurora_core::ParameterBundle;
use serde::{Deserialize, Serialize};

use crate::eval_context::{EvaluationContext, RuleWeightMapping};
use crate::rule::{RuleId, RuleSet, SoftRule};

/// Per-rule evaluation output (rule-dsl.md §8.3, scoring.md §12).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuleEvalResult {
    pub rule_id: RuleId,
    pub applied: bool,
    pub score_delta: f64,
    pub reason: String,
    pub indicator: f64,
    pub is_penalty: bool,
}

/// Accumulated score for a candidate patch (scoring.md §8.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreResult {
    pub eval_score: f64,
    pub step_deltas: Vec<RuleEvalResult>,
    pub provenance: Vec<RuleEvalResult>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoredCandidate {
    pub eval_score: f64,
    pub parent_score: f64,
    pub step_deltas: Vec<RuleEvalResult>,
    pub provenance: Vec<RuleEvalResult>,
}

/// Scoring engine: eval_score = Σ(weight × indicator) − Σ(penalty × violation).
#[derive(Clone)]
pub struct ScoringFunction {
    ruleset: RuleSet,
    params: ParameterBundle,
}

impl ScoringFunction {
    pub fn new(ruleset: RuleSet, params: ParameterBundle) -> Self {
        Self { ruleset, params }
    }

    pub fn params(&self) -> &ParameterBundle {
        &self.params
    }

    pub fn evaluate(
        &self,
        ctx: &EvaluationContext<'_>,
        parent_score: f64,
    ) -> ScoreResult {
        let mut total = 0.0;
        let mut deltas = Vec::new();

        for rule in &self.ruleset.soft {
            if let Some(outcome) = rule.eval(ctx, self.resolve_weight(rule)) {
                let weight = self.resolve_weight(rule);
                let magnitude = outcome.indicator.abs();
                let delta = if outcome.is_penalty {
                    -weight * magnitude
                } else {
                    weight * magnitude
                };
                total += delta;
                deltas.push(RuleEvalResult {
                    rule_id: rule.meta.id.clone(),
                    applied: true,
                    score_delta: delta,
                    reason: outcome.reason,
                    indicator: outcome.indicator,
                    is_penalty: outcome.is_penalty,
                });
            }
        }

        let eval_score = parent_score + total;
        ScoreResult {
            eval_score,
            provenance: deltas.clone(),
            step_deltas: deltas,
        }
    }

    fn resolve_weight(&self, rule: &SoftRule) -> f64 {
        match rule.meta.id.as_str() {
            "HARM-001" => self.params.chord_tone_weight(),
            "HARM-015" | "HARM-CAD-002" | "FORM-001" | "HARM-PROG-003" => {
                self.params.cadence_strength_weight()
            }
            "VLED-003" | "VLED-001" => self.params.stepwise_preference(),
            "VLED-010" => self.params.leap_penalty(),
            "CONT-001-soft" => self.params.parallel_penalty(),
            "MOTI-001" => {
                self.params.repetition_ratio() * 20.0
                    * (0.5 + f64::from(self.params.melody.motif_weight) * 0.75)
            }
            "RHYT-005" => self.params.syncopation() * 10.0,
            "HARM-003" => self.params.harmony_complexity() * 10.0,
            "HARM-021" => self.params.harmony_complexity() * 8.5,
            "RHYT-001" => self.params.dynamics.accent_strength as f64 * 10.0,
            "DRUM-003" => self.params.drums.pattern_complexity as f64 * 5.0,
            "HARM-010" => self.params.dissonance_tolerance() * 15.0,
            "HARM-MEL-001" => self.params.nct_penalty_weight(),
            "MEL-CONT-001" => self.params.contour_balance_weight(),
            "MEL-CLOSE-001" => self.params.cadence_strength_weight() * 1.8,
            "MEL-REP-001" => self.params.contour_balance_weight() * 2.5,
            "ORCH-010" => self.params.contour_balance_weight() * 2.0,
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use aurora_core::ParameterBundle;

    use crate::{
        AstSnapshot, BeatStrength, BeatStrengthKind, CandidatePatch, ChordQuality, ChordSymbol,
        EvaluationContext, KeySignature, Mode, NodeId, Pitch, PitchClass, ScoringFunction, VoiceId,
        VoiceRole, search_note,
    };
    use crate::rules::prototype_rule_set;

    fn eval_ctx(midi: u8, prev: Option<u8>, strong: bool) -> EvaluationContext<'static> {
        let mut snapshot = AstSnapshot {
            key: KeySignature {
                tonic: PitchClass { pc: 0 },
                mode: Mode::Major,
            },
            current_chord: Some(ChordSymbol {
                root: PitchClass { pc: 0 },
                quality: ChordQuality::Major,
                extensions: Vec::new(),
                bass: None,
                raw: "C".into(),
            }),
            beat_strength: BeatStrength(if strong {
                BeatStrengthKind::Strong
            } else {
                BeatStrengthKind::Weak
            }),
            ..Default::default()
        };
        if let Some(p) = prev {
            snapshot.melody_pitches.push(Pitch::from_midi(p));
        }
        let patch = CandidatePatch::single_note(
            VoiceId(0),
            NodeId::new(1, 0),
            search_note(midi, NodeId::new(2, 0)),
            "n",
        );
        EvaluationContext {
            snapshot: Box::leak(Box::new(snapshot)),
            patch: Box::leak(Box::new(patch)),
            voice_role: VoiceRole::Melody,
            step_index: 1,
        }
    }

    #[test]
    fn scoring_equation_rewards_and_penalties() {
        let scorer = ScoringFunction::new(prototype_rule_set(), ParameterBundle::default());
        let ctx = eval_ctx(67, Some(65), true);
        let result = scorer.evaluate(&ctx, 0.0);
        assert!(result.eval_score > 0.0);
        assert!(result.provenance.iter().any(|r| r.rule_id.as_str() == "HARM-001"));
        assert!(result.provenance.iter().any(|r| r.rule_id.as_str() == "VLED-003"));
    }

    #[test]
    fn large_leap_incurs_penalty() {
        let scorer = ScoringFunction::new(prototype_rule_set(), ParameterBundle::default());
        let ctx = eval_ctx(72, Some(60), false);
        let result = scorer.evaluate(&ctx, 0.0);
        assert!(
            result
                .provenance
                .iter()
                .any(|r| r.rule_id.as_str() == "VLED-010" && r.is_penalty)
        );
    }
}
