use std::cmp::Ordering;

use aurora_core::ParameterBundle;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::eval_context::{AstSnapshot, CandidatePatch, EvaluationContext};
use aurora_ast::VoiceRole;
use crate::constraint::ConstraintEvaluator;
use crate::scoring::{ScoreResult, ScoringFunction};

/// Unique search state identifier for provenance linking (scoring.md §8.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateId(pub Uuid);

impl StateId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StateId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchState {
    pub id: StateId,
    pub snapshot: AstSnapshot,
    pub step_index: u32,
    pub eval_score: f64,
    pub applied_rules: Vec<crate::RuleEvalResult>,
    pub beam_rank: Option<u32>,
    pub parent: Option<StateId>,
    pub depth: u32,
    pub last_patch: Option<CandidatePatch>,
}

impl SearchState {
    pub fn initial(snapshot: AstSnapshot) -> Self {
        Self {
            id: StateId::new(),
            snapshot,
            step_index: 0,
            eval_score: 0.0,
            applied_rules: Vec::new(),
            beam_rank: None,
            parent: None,
            depth: 0,
            last_patch: None,
        }
    }

    pub fn extend(
        &self,
        patch: CandidatePatch,
        score: ScoreResult,
        parent_id: StateId,
    ) -> Self {
        Self {
            id: StateId::new(),
            snapshot: self.snapshot.apply(&patch),
            step_index: self.step_index + 1,
            eval_score: score.eval_score,
            applied_rules: score.step_deltas,
            beam_rank: None,
            parent: Some(parent_id),
            depth: self.depth + 1,
            last_patch: Some(patch),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BeamConfig {
    pub beam_width: usize,
    pub max_iterations: u32,
    pub temperature: f64,
}

impl BeamConfig {
    pub fn from_params(params: &ParameterBundle) -> Self {
        Self {
            beam_width: params.search.beam_width as usize,
            max_iterations: params.search.max_iterations,
            temperature: params.search.temperature as f64,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeamFrame {
    pub step: u32,
    pub states: Vec<SearchState>,
    pub pruned_count: u32,
    pub avg_score: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SearchStats {
    pub iterations: u32,
    pub total_candidates: u32,
    pub total_pruned: u32,
    pub beam_width: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RejectionLog {
    pub step: u32,
    pub parent_state: StateId,
    pub rule_id: Option<crate::RuleId>,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SearchTrace {
    pub mode: String,
    pub beam_width: usize,
    pub frames: Vec<BeamFrame>,
    pub rejections: Vec<RejectionLog>,
    pub winner_path: Vec<StateId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub best_state: SearchState,
    pub trace: SearchTrace,
    pub stats: SearchStats,
}

#[derive(Clone, Debug)]
pub struct SearchExhausted {
    pub trace: SearchTrace,
}

impl std::fmt::Display for SearchExhausted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "beam search exhausted after {} frames",
            self.trace.frames.len()
        )
    }
}

impl std::error::Error for SearchExhausted {}

pub trait TerminalCondition {
    fn is_terminal(&self, state: &SearchState) -> bool;
}

pub struct StepCountTerminal {
    pub max_steps: u32,
}

impl TerminalCondition for StepCountTerminal {
    fn is_terminal(&self, state: &SearchState) -> bool {
        state.step_index >= self.max_steps
    }
}

pub trait CandidateGenerator {
    fn generate(&self, state: &SearchState) -> Vec<CandidatePatch>;
}

/// Beam search engine per ADR-003 and scoring.md Appendix A.
#[derive(Clone)]
pub struct BeamSearchEngine {
    constraints: ConstraintEvaluator,
    scorer: ScoringFunction,
    config: BeamConfig,
}

impl BeamSearchEngine {
    pub fn new(
        constraints: ConstraintEvaluator,
        scorer: ScoringFunction,
        config: BeamConfig,
    ) -> Self {
        Self {
            constraints,
            scorer,
            config,
        }
    }

    pub fn from_bundle(ruleset: crate::RuleSet, params: ParameterBundle) -> Self {
        let config = BeamConfig::from_params(&params);
        Self {
            constraints: ConstraintEvaluator::new(ruleset.clone(), params.clone()),
            scorer: ScoringFunction::new(ruleset, params),
            config,
        }
    }

    /// Primary search entry point (ADR-003).
    pub fn run_beam<G, T>(
        &self,
        initial: SearchState,
        generator: &G,
        terminal: &T,
    ) -> Result<SearchResult, SearchExhausted>
    where
        G: CandidateGenerator,
        T: TerminalCondition,
    {
        let mut beam = vec![initial];
        let mut trace = SearchTrace {
            mode: "beam".into(),
            beam_width: self.config.beam_width,
            ..Default::default()
        };
        let mut stats = SearchStats {
            beam_width: self.config.beam_width,
            ..Default::default()
        };

        let mut iterations = 0u32;
        while beam.iter().any(|s| !terminal.is_terminal(s)) {
            if iterations >= self.config.max_iterations {
                break;
            }
            iterations += 1;
            stats.iterations = iterations;

            let mut candidates = Vec::new();
            let mut step_pruned = 0u32;
            let step = beam.first().map(|s| s.step_index).unwrap_or(0);

            for state in &beam {
                for patch in generator.generate(state) {
                    stats.total_candidates += 1;
                    let trial_snapshot = state
                        .snapshot
                        .clone()
                        .with_registers_from(self.constraints.params())
                        .for_step(state.step_index);
                    let ctx = EvaluationContext {
                        snapshot: &trial_snapshot,
                        patch: &patch,
                        voice_role: VoiceRole::Melody,
                        step_index: state.step_index,
                    };

                    if let Err(v) = self.constraints.check(&ctx) {
                        step_pruned += 1;
                        stats.total_pruned += 1;
                        trace.rejections.push(RejectionLog {
                            step,
                            parent_state: state.id,
                            rule_id: Some(v.rule_id),
                            reason: v.reason,
                        });
                        continue;
                    }

                    let score = self.scorer.evaluate(&ctx, state.eval_score);
                    candidates.push(state.extend(patch, score, state.id));
                }
            }

            if candidates.is_empty() {
                return Err(SearchExhausted { trace });
            }

            candidates.sort_by(|a, b| {
                b.eval_score
                    .partial_cmp(&a.eval_score)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| a.id.cmp(&b.id))
            });

            let width = self.config.beam_width.max(1);
            let pruned_at_step = candidates.len().saturating_sub(width) as u32;
            step_pruned += pruned_at_step;
            stats.total_pruned += pruned_at_step;

            let mut next_beam: Vec<SearchState> = candidates.into_iter().take(width).collect();
            for (rank, state) in next_beam.iter_mut().enumerate() {
                state.beam_rank = Some(rank as u32);
            }

            let avg_score = if next_beam.is_empty() {
                0.0
            } else {
                next_beam.iter().map(|s| s.eval_score).sum::<f64>() / next_beam.len() as f64
            };

            trace.frames.push(BeamFrame {
                step,
                states: next_beam.clone(),
                pruned_count: step_pruned,
                avg_score,
            });

            beam = next_beam;
        }

        let best = beam
            .into_iter()
            .max_by(|a, b| {
                a.eval_score
                    .partial_cmp(&b.eval_score)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| a.id.cmp(&b.id))
            })
            .expect("beam non-empty at termination");

        trace.winner_path = reconstruct_path(&best, &trace.frames);
        Ok(SearchResult {
            best_state: best,
            trace,
            stats,
        })
    }
}

fn reconstruct_path(winner: &SearchState, frames: &[BeamFrame]) -> Vec<StateId> {
    let mut path = vec![winner.id];
    let mut current_parent = winner.parent;
    while let Some(pid) = current_parent {
        path.push(pid);
        current_parent = frames
            .iter()
            .flat_map(|f| &f.states)
            .find(|s| s.id == pid)
            .and_then(|s| s.parent);
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use aurora_core::ParameterBundle;

    use super::*;
    use crate::{
        AstSnapshot, BeatStrength, BeatStrengthKind, CandidatePatch, ChordQuality, ChordSymbol,
        KeySignature, Mode, NodeId, PitchClass, VoiceId, search_note,
    };
    use crate::rules::prototype_rule_set;

    struct MockMelodyGenerator {
        pub options: Vec<u8>,
    }

    impl CandidateGenerator for MockMelodyGenerator {
        fn generate(&self, _state: &SearchState) -> Vec<CandidatePatch> {
            self.options
                .iter()
                .map(|&midi| {
                    CandidatePatch::single_note(
                        VoiceId(0),
                        NodeId::new(1, 0),
                        search_note(midi, NodeId::new(u64::from(midi), 0)),
                        format!("pitch_{midi}"),
                    )
                })
                .collect()
        }
    }

    fn melody_snapshot() -> AstSnapshot {
        AstSnapshot {
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
            beat_strength: BeatStrength(BeatStrengthKind::Strong),
            grid_subdivision: 4,
            ..Default::default()
        }
    }

    #[test]
    fn beam_search_selects_highest_scoring_melody_path() {
        let engine = BeamSearchEngine::from_bundle(prototype_rule_set(), ParameterBundle::default());
        let initial = SearchState::initial(melody_snapshot());
        let generator = MockMelodyGenerator {
            options: vec![60, 62, 64, 67, 80],
        };
        let terminal = StepCountTerminal { max_steps: 2 };

        let result = engine
            .run_beam(initial, &generator, &terminal)
            .expect("search should succeed");

        assert_eq!(result.best_state.step_index, 2);
        assert!(result.best_state.eval_score > 0.0);
        assert!(!result.best_state.applied_rules.is_empty());
        assert_eq!(result.trace.mode, "beam");
        assert_eq!(result.stats.beam_width, 16);
        assert!(result.stats.total_pruned >= 1);
    }

    #[test]
    fn beam_width_limits_frontier() {
        let mut params = ParameterBundle::default();
        params.search.beam_width = 2;
        let engine = BeamSearchEngine::from_bundle(prototype_rule_set(), params);
        let initial = SearchState::initial(melody_snapshot());
        let generator = MockMelodyGenerator {
            options: vec![60, 62, 64, 67],
        };
        let terminal = StepCountTerminal { max_steps: 1 };

        let result = engine
            .run_beam(initial, &generator, &terminal)
            .expect("search ok");

        assert!(result.trace.frames[0].states.len() <= 2);
    }

    #[test]
    fn exhaustive_when_all_pruned() {
        let mut params = ParameterBundle::default();
        params.register.melody_register_min = 70;
        params.register.melody_register_max = 72;
        let engine = BeamSearchEngine::from_bundle(prototype_rule_set(), params);
        let initial = SearchState::initial(melody_snapshot());
        let generator = MockMelodyGenerator {
            options: vec![60, 62, 64],
        };
        let terminal = StepCountTerminal { max_steps: 1 };

        assert!(engine.run_beam(initial, &generator, &terminal).is_err());
    }
}
