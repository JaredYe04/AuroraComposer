use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable node identity across patches and serialization.
///
/// See `docs/02-music-model/ast.md` §7.3.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    pub index: u64,
    pub generation: u32,
}

impl NodeId {
    #[must_use]
    pub const fn new(index: u64, generation: u32) -> Self {
        Self { index, generation }
    }
}

/// Async generation job identifier (Tauri IPC / orchestrator).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl JobId {
    #[must_use]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new_v4()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_equality_uses_index_and_generation() {
        let a = NodeId::new(1, 0);
        let b = NodeId::new(1, 1);
        assert_eq!(a, NodeId::new(1, 0));
        assert_ne!(a, b);
    }

    #[test]
    fn job_id_roundtrips_json() {
        let id = JobId::new_v4();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: JobId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }
}
