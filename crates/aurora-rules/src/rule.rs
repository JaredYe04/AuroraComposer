use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Stable rule identifier, e.g. `HARM-001` (rule-dsl.md §6.3).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct RuleId(pub String);

impl RuleId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RuleId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RuleId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Rule category prefix per rule-dsl.md §6.3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    Harmony,
    Counterpoint,
    VoiceLeading,
    Rhythm,
    Form,
    Register,
    Motif,
    Texture,
    Drums,
    Jazz,
    Orchestration,
    Custom,
}

impl RuleCategory {
    pub fn priority_order(self) -> u8 {
        match self {
            Self::Register => 0,
            Self::Counterpoint => 1,
            Self::Harmony => 2,
            Self::VoiceLeading => 3,
            Self::Rhythm => 4,
            Self::Motif => 5,
            Self::Form => 6,
            Self::Texture => 7,
            Self::Drums => 8,
            Self::Jazz => 9,
            Self::Orchestration => 10,
            Self::Custom => 11,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleMode {
    Hard,
    Soft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum RuleScope {
    Event,
    EventPair,
    Measure,
    MeasurePair,
    Phrase,
    Voice,
    VoicePair,
    VoicePairConsecutive,
    Score,
}

impl RuleScope {
    pub fn narrowness(self) -> u8 {
        match self {
            Self::Event => 0,
            Self::EventPair => 1,
            Self::Voice => 2,
            Self::VoicePair => 3,
            Self::VoicePairConsecutive => 4,
            Self::Measure => 5,
            Self::MeasurePair => 6,
            Self::Phrase => 7,
            Self::Score => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvalCost {
    Low,
    Medium,
    High,
}

/// Unified rule descriptor (compile-time / runtime metadata).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub category: RuleCategory,
    pub mode: RuleMode,
    pub scope: RuleScope,
    pub citation: Option<String>,
    pub cost: EvalCost,
}

/// Hard constraint rule — prunes candidates on violation (constraint.md §1.2).
#[derive(Clone)]
pub struct HardRule {
    pub meta: Rule,
    pub check: fn(&crate::EvaluationContext<'_>) -> bool,
    pub fail_reason: fn(&crate::EvaluationContext<'_>) -> String,
    pub when: Option<fn(&crate::EvaluationContext<'_>) -> bool>,
}

impl HardRule {
    pub fn applies(&self, ctx: &crate::EvaluationContext<'_>) -> bool {
        self.when.map_or(true, |w| w(ctx))
    }

    pub fn evaluate(&self, ctx: &crate::EvaluationContext<'_>) -> HardCheckResult {
        if !self.applies(ctx) {
            return HardCheckResult::Skipped;
        }
        if (self.check)(ctx) {
            HardCheckResult::Passed
        } else {
            HardCheckResult::Failed {
                rule_id: self.meta.id.clone(),
                reason: (self.fail_reason)(ctx),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HardCheckResult {
    Skipped,
    Passed,
    Failed { rule_id: RuleId, reason: String },
}

/// Soft scoring rule — contributes to eval_score (scoring.md §9.1).
#[derive(Clone)]
pub struct SoftRule {
    pub meta: Rule,
    pub weight_key: Option<&'static str>,
    pub evaluate: fn(&crate::EvaluationContext<'_>, f64) -> SoftEvalOutcome,
    pub when: Option<fn(&crate::EvaluationContext<'_>) -> bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoftEvalOutcome {
    pub indicator: f64,
    pub is_penalty: bool,
    pub reason: String,
}

impl SoftRule {
    pub fn applies(&self, ctx: &crate::EvaluationContext<'_>) -> bool {
        self.when.map_or(true, |w| w(ctx))
    }

    pub fn eval(&self, ctx: &crate::EvaluationContext<'_>, weight: f64) -> Option<SoftEvalOutcome> {
        if !self.applies(ctx) {
            return None;
        }
        Some((self.evaluate)(ctx, weight))
    }
}

/// Active rule collection for a pipeline stage.
#[derive(Clone, Default)]
pub struct RuleSet {
    pub hard: Vec<HardRule>,
    pub soft: Vec<SoftRule>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_hard(mut self, rules: Vec<HardRule>) -> Self {
        self.hard = rules;
        self.sort();
        self
    }

    pub fn with_soft(mut self, rules: Vec<SoftRule>) -> Self {
        self.soft = rules;
        self.sort();
        self
    }

    pub fn sort(&mut self) {
        self.hard.sort_by_key(|r| (r.meta.scope.narrowness(), r.meta.id.clone()));
        self.soft.sort_by_key(|r| {
            (
                r.meta.category.priority_order(),
                r.meta.scope.narrowness(),
                r.meta.id.clone(),
            )
        });
    }

    pub fn rule_count(&self) -> usize {
        self.hard.len() + self.soft.len()
    }
}
