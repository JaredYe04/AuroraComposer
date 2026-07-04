//! Provenance metadata — required on every generated event (invariant I-PROV-1).
//!
//! See `docs/02-music-model/ast.md` §8.9.

use aurora_core::NodeId;
use serde::{Deserialize, Serialize};

/// Reference to a contributing rule with optional evaluation contribution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuleRef {
    pub id: String,
    pub weight: Option<f64>,
    pub score: Option<f64>,
}

/// Pipeline stage identifier for provenance and orchestration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStageId {
    StyleResolver,
    EmotionResolver,
    StructurePlanning,
    ThemePlanning,
    HarmonySkeleton,
    RhythmSkeleton,
    Melody,
    Counterpoint,
    Bass,
    Drums,
    Decoration,
    Repair,
    Manual,
    Custom(u32),
}

/// Search beam context recorded on committed events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchContext {
    pub step_index: u32,
    pub beam_rank: u16,
    pub beam_width: u16,
    pub state_ref: StateRef,
    pub accumulated_score: f64,
}

/// Opaque reference to a transient search state (not persisted in project files).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRef {
    pub id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvenanceSource {
    Generated,
    ManualEdit,
    Imported,
    Repaired,
    Plugin,
    Transformed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRef {
    pub node_id: NodeId,
    pub patch_id: Option<PatchId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatchId(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProvenanceAgent {
    Engine { stage: PipelineStageId },
    User { user_id: Option<String> },
    Plugin { plugin_id: String },
    Import { format: String },
}

/// Mandatory provenance on every [`crate::events::Event`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: ProvenanceSource,
    pub stage: Option<PipelineStageId>,
    pub rule_ids: Vec<String>,
    pub rule_refs: Vec<RuleRef>,
    pub eval_score: Option<f64>,
    pub search: Option<SearchContext>,
    pub parent: Option<ProvenanceRef>,
    pub created_at: String,
    pub agent: ProvenanceAgent,
    pub parameters_hash: Option<String>,
    pub explanation: Option<String>,
}

impl Provenance {
    /// Minimal generated provenance for tests and skeleton construction.
    #[must_use]
    pub fn generated(stage: PipelineStageId, created_at: &str) -> Self {
        Self {
            source: ProvenanceSource::Generated,
            stage: Some(stage),
            rule_ids: Vec::new(),
            rule_refs: Vec::new(),
            eval_score: None,
            search: None,
            parent: None,
            created_at: created_at.into(),
            agent: ProvenanceAgent::Engine { stage },
            parameters_hash: None,
            explanation: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRoot {
    pub session_id: String,
    pub generator_version: String,
    pub seed: Option<u64>,
    pub pipeline_config_hash: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_generated_has_source_and_timestamp() {
        let p = Provenance::generated(PipelineStageId::Melody, "2026-01-01T00:00:00Z");
        assert!(matches!(p.source, ProvenanceSource::Generated));
        assert_eq!(p.created_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn rule_ref_roundtrips_json() {
        let r = RuleRef {
            id: "melody.stepwise".into(),
            weight: Some(1.0),
            score: Some(0.8),
        };
        let json = serde_json::to_string(&r).unwrap();
        let parsed: RuleRef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "melody.stepwise");
    }

    #[test]
    fn search_context_includes_beam_width() {
        let ctx = SearchContext {
            step_index: 3,
            beam_rank: 1,
            beam_width: 16,
            state_ref: StateRef {
                id: "state-1".into(),
            },
            accumulated_score: 0.75,
        };
        let json = serde_json::to_value(&ctx).unwrap();
        assert_eq!(json["beam_width"], 16);
    }
}
