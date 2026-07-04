//! Aurora Composer rule engine, constraint checking, scoring, and beam search.
//!
//! Implements frozen specs:
//! - docs/05-rule-engine/scoring.md
//! - docs/05-rule-engine/constraint.md
//! - docs/05-rule-engine/rule-dsl.md
//! - decisions/ADR-003-search-algorithm-primary.md

pub mod constraint;
pub mod eval_context;
pub mod melody_nct;
pub mod rule;
pub mod rules;
pub mod scale;
pub mod scoring;
pub mod search;

pub use aurora_ast::{
    BeatOffset, CadenceType, ChordQuality, ChordSymbol, Event, KeySignature, Mode, NoteEvent,
    Pitch, PitchClass, TimedEventBase, VoiceId, VoiceRole,
};
pub use aurora_core::NodeId;
pub use eval_context::{
    AstSnapshot, BeatStrength, BeatStrengthKind, CandidatePatch, EvaluationContext, PatchId,
    PitchExt, RuleWeightMapping, search_note,
};
pub use constraint::{ConstraintEvaluator, ConstraintViolation, ConstraintViolationKind};
pub use rule::{
    EvalCost, HardCheckResult, HardRule, Rule, RuleCategory, RuleId, RuleMode, RuleScope,
    RuleSet, SoftRule,
};
pub use rules::{full_rule_set, prototype_rule_set};
pub use scoring::{RuleEvalResult, ScoreResult, ScoredCandidate, ScoringFunction};
pub use search::{
    BeamConfig, BeamFrame, BeamSearchEngine, CandidateGenerator, SearchExhausted, SearchResult,
    SearchState, SearchStats, SearchTrace, StateId, StepCountTerminal, TerminalCondition,
};
