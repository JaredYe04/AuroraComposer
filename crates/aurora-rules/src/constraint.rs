use aurora_core::ParameterBundle;

use crate::eval_context::{AstSnapshot, CandidatePatch, EvaluationContext, RuleWeightMapping};
use aurora_ast::VoiceRole;
use crate::rule::{HardCheckResult, HardRule, RuleSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintViolation {
    pub rule_id: crate::RuleId,
    pub reason: String,
    pub kind: ConstraintViolationKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintViolationKind {
    HardReject,
}

/// Owns enriched snapshot + borrows patch for safe evaluation lifetimes.
pub struct EvaluationBundle<'a> {
    pub enriched: AstSnapshot,
    pub patch: &'a CandidatePatch,
    pub voice_role: VoiceRole,
    pub step_index: u32,
}

impl<'a> EvaluationBundle<'a> {
    pub fn context(&self) -> EvaluationContext<'_> {
        EvaluationContext {
            snapshot: &self.enriched,
            patch: self.patch,
            voice_role: self.voice_role,
            step_index: self.step_index,
        }
    }
}

/// Evaluates hard constraints and prunes inadmissible candidates (constraint.md §9.1).
#[derive(Clone)]
pub struct ConstraintEvaluator {
    ruleset: RuleSet,
    params: ParameterBundle,
}

impl ConstraintEvaluator {
    pub fn new(ruleset: RuleSet, params: ParameterBundle) -> Self {
        Self { ruleset, params }
    }

    pub fn params(&self) -> &ParameterBundle {
        &self.params
    }

    pub fn ruleset(&self) -> &RuleSet {
        &self.ruleset
    }

    pub fn make_context<'a>(
        &self,
        snapshot: &'a AstSnapshot,
        patch: &'a CandidatePatch,
        voice_role: VoiceRole,
        step_index: u32,
    ) -> EvaluationBundle<'a> {
        EvaluationBundle {
            enriched: snapshot.clone().with_registers_from(&self.params),
            patch,
            voice_role,
            step_index,
        }
    }

    /// Returns `Ok(())` or first hard violation (short-circuit per constraint.md §9.1).
    pub fn check(&self, ctx: &EvaluationContext<'_>) -> Result<(), ConstraintViolation> {
        for rule in &self.ruleset.hard {
            if !rule_active(rule, &self.params) {
                continue;
            }
            match rule.evaluate(ctx) {
                HardCheckResult::Skipped | HardCheckResult::Passed => {}
                HardCheckResult::Failed { rule_id, reason } => {
                    return Err(ConstraintViolation {
                        rule_id,
                        reason,
                        kind: ConstraintViolationKind::HardReject,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn check_batch(
        &self,
        snapshot: &AstSnapshot,
        patches: &[CandidatePatch],
        voice_role: VoiceRole,
        step_index: u32,
    ) -> Vec<Result<(), ConstraintViolation>> {
        patches
            .iter()
            .map(|patch| {
                self.check(
                    &self
                        .make_context(snapshot, patch, voice_role, step_index)
                        .context(),
                )
            })
            .collect()
    }

    pub fn should_prune(
        &self,
        snapshot: &AstSnapshot,
        patch: &CandidatePatch,
        voice_role: VoiceRole,
        step_index: u32,
    ) -> bool {
        self.check(
            &self
                .make_context(snapshot, patch, voice_role, step_index)
                .context(),
        )
        .is_err()
    }

    pub fn prune_count(
        &self,
        snapshot: &AstSnapshot,
        patches: &[CandidatePatch],
        voice_role: VoiceRole,
        step_index: u32,
    ) -> usize {
        self.check_batch(snapshot, patches, voice_role, step_index)
            .into_iter()
            .filter(Result::is_err)
            .count()
    }
}

fn rule_active(rule: &HardRule, params: &ParameterBundle) -> bool {
    let id = rule.meta.id.as_str();
    if id == "CONT-001"
        || id == "CONT-002"
        || id == "CP-PAR-001"
        || id == "CP-PAR-002"
        || id.starts_with("CP-PAR-")
        || (id.starts_with("CP-") && rule.meta.category == crate::rule::RuleCategory::Counterpoint)
    {
        return params.counterpoint_strictness() >= 0.8;
    }
    if id == "HARM-050" {
        return params.harmony_complexity() <= 0.2;
    }
    // Chromatic progressions (V7/x, bVII, iv) are valid above moderate complexity;
    // the legacy check rejects every melody candidate on those chords.
    if id == "HARM-041" {
        return params.harmony_complexity() < 0.35;
    }
    if id == "HARM-MEL-002" || id == "HARM-MEL-003" {
        return params.tonal_conservatism() >= 0.55;
    }
    true
}

#[cfg(test)]
mod tests {
    use aurora_core::ParameterBundle;

    use crate::{
        AstSnapshot, BeatStrength, BeatStrengthKind, CandidatePatch, NodeId, VoiceId, VoiceRole,
        search_note,
    };
    use crate::rules::prototype_rule_set;
    use super::ConstraintEvaluator;

    fn note_patch(midi: u8) -> CandidatePatch {
        CandidatePatch::single_note(
            VoiceId(0),
            NodeId::new(1, 0),
            search_note(midi, NodeId::new(u64::from(midi), 0)),
            "test",
        )
    }

    #[test]
    fn register_hard_constraint_prunes_out_of_range() {
        let ruleset = prototype_rule_set();
        let mut params = ParameterBundle::default();
        params.register.melody_register_min = 60;
        params.register.melody_register_max = 72;
        let evaluator = ConstraintEvaluator::new(ruleset, params);
        let snapshot = AstSnapshot {
            beat_strength: BeatStrength(BeatStrengthKind::Strong),
            ..Default::default()
        };

        let in_patch = note_patch(64);
        let bundle = evaluator.make_context(&snapshot, &in_patch, VoiceRole::Melody, 0);
        assert!(evaluator.check(&bundle.context()).is_ok());

        let out_patch = note_patch(80);
        let bundle = evaluator.make_context(&snapshot, &out_patch, VoiceRole::Melody, 0);
        let err = evaluator.check(&bundle.context()).unwrap_err();
        assert!(
            matches!(
                err.rule_id.as_str(),
                "REG-001" | "VL-RNG-001" | "CP-058" | "ORCH-001"
            ),
            "expected register rule, got {}",
            err.rule_id
        );
    }

    #[test]
    fn harm_041_inactive_when_complexity_moderate() {
        use crate::{ChordQuality, ChordSymbol, KeySignature, Mode, PitchClass};

        let ruleset = prototype_rule_set();
        let mut params = ParameterBundle::default();
        params.harmony.complexity = 0.5;
        let evaluator = ConstraintEvaluator::new(ruleset, params);

        let mut snapshot = AstSnapshot::default();
        snapshot.current_chord = Some(ChordSymbol {
            root: PitchClass { pc: 4 },
            quality: ChordQuality::Dominant7,
            extensions: vec![],
            bass: None,
            raw: "E7".into(),
        });
        snapshot.key = KeySignature {
            tonic: PitchClass { pc: 0 },
            mode: Mode::Major,
        };

        let patch = note_patch(64);
        let bundle = evaluator.make_context(&snapshot, &patch, VoiceRole::Melody, 0);
        assert!(
            evaluator.check(&bundle.context()).is_ok(),
            "HARM-041 must not block melody on secondary dominants at complexity 0.5"
        );
    }

    #[test]
    fn prune_count_reports_rejections() {
        let ruleset = prototype_rule_set();
        let evaluator = ConstraintEvaluator::new(ruleset, ParameterBundle::default());
        let snapshot = AstSnapshot::default();
        let patches = vec![note_patch(62), note_patch(64), note_patch(100)];
        assert_eq!(
            evaluator.prune_count(&snapshot, &patches, VoiceRole::Melody, 0),
            1
        );
    }
}
