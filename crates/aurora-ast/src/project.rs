//! Score container: project envelope with CBOR persistence (ADR-006).
//!
//! See `docs/02-music-model/score.md` and `decisions/ADR-006-cbor-project-serialization.md`.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use aurora_core::{AuroraError, ParameterBundle};
use serde::{Deserialize, Serialize};

use crate::nodes::Composition;
use crate::patch::PatchRecord;

pub const PROJECT_FORMAT_VERSION: ProjectFormatVersion = ProjectFormatVersion {
    major: 0,
    minor: 1,
};

/// Alias for parameter snapshot stored on projects (score.md terminology).
pub type ParameterSnapshot = ParameterBundle;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub manifest: ProjectManifest,
    pub composition: Composition,
    pub parameters: ParameterSnapshot,
    pub history: PatchHistory,
    pub export_cache: Option<ExportCache>,
    pub plugin_config: PluginConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project_id: String,
    pub format_version: ProjectFormatVersion,
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub aurora_engine_version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFormatVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatchHistory {
    pub patches: Vec<PatchRecord>,
    pub cursor: usize,
    pub max_depth: usize,
}

impl Default for PatchHistory {
    fn default() -> Self {
        Self {
            patches: Vec::new(),
            cursor: 0,
            max_depth: 100,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportCache {
    pub ir_source_hash: String,
    pub exported_files: std::collections::HashMap<String, ExportedFileMeta>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportedFileMeta {
    pub path: String,
    pub format: ExportFormat,
    pub exported_at: String,
    pub hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    MusicXml,
    Midi,
    Abc,
    Pdf,
    Json,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled_plugins: Vec<String>,
}

impl Project {
    /// Serialize the full project to CBOR bytes.
    pub fn to_cbor(&self) -> Result<Vec<u8>, AuroraError> {
        serde_cbor::to_vec(self).map_err(|e| AuroraError::Serialization(e.to_string()))
    }

    /// Deserialize a project from CBOR bytes.
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, AuroraError> {
        serde_cbor::from_slice(bytes).map_err(|e| AuroraError::Serialization(e.to_string()))
    }

    /// Write project to a `.aurora` CBOR file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), AuroraError> {
        let bytes = self.to_cbor()?;
        let mut file =
            File::create(path.as_ref()).map_err(|e| AuroraError::Serialization(e.to_string()))?;
        file.write_all(&bytes)
            .map_err(|e| AuroraError::Serialization(e.to_string()))
    }

    /// Load project from a `.aurora` CBOR file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, AuroraError> {
        let mut file =
            File::open(path.as_ref()).map_err(|e| AuroraError::Serialization(e.to_string()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|e| AuroraError::Serialization(e.to_string()))?;
        Self::from_cbor(&bytes)
    }

    /// Serialize only the composition AST to CBOR (bundle `composition.cbor` entry).
    pub fn composition_to_cbor(composition: &Composition) -> Result<Vec<u8>, AuroraError> {
        serde_cbor::to_vec(composition).map_err(|e| AuroraError::Serialization(e.to_string()))
    }

    /// Deserialize composition AST from CBOR.
    pub fn composition_from_cbor(bytes: &[u8]) -> Result<Composition, AuroraError> {
        serde_cbor::from_slice(bytes).map_err(|e| AuroraError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::CompositionBuilder;

    #[test]
    fn project_cbor_roundtrip() {
        let comp = CompositionBuilder::new().title("Test Song").one_measure().build();
        let project = Project {
            manifest: ProjectManifest {
                project_id: "proj-1".into(),
                format_version: PROJECT_FORMAT_VERSION,
                name: "Test Song".into(),
                created_at: "2026-01-01".into(),
                modified_at: "2026-01-01".into(),
                author: None,
                description: None,
                tags: vec![],
                aurora_engine_version: "0.1.0".into(),
            },
            composition: comp,
            parameters: ParameterBundle::default(),
            history: PatchHistory::default(),
            export_cache: None,
            plugin_config: PluginConfig::default(),
        };
        let bytes = project.to_cbor().unwrap();
        let loaded = Project::from_cbor(&bytes).unwrap();
        assert_eq!(loaded.manifest.name, "Test Song");
        assert_eq!(loaded.composition.metadata.title, "Test Song");
    }

    #[test]
    fn composition_cbor_roundtrip() {
        let comp = CompositionBuilder::new().one_measure().build();
        let bytes = Project::composition_to_cbor(&comp).unwrap();
        let loaded = Project::composition_from_cbor(&bytes).unwrap();
        assert_eq!(loaded.id, comp.id);
    }
}
