//! WASM plugin discovery stub — scans `plugins/` for manifest metadata.
//!
//! Full WASM execution via wasmtime is deferred; this module registers discovered
//! plugin manifests for listing and future loading.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::PluginError;
use crate::manifest::PluginManifest;
use crate::traits::PluginType;
use crate::types::{PluginDescriptor, PluginState};

const MANIFEST_NAME: &str = "aurora-plugin.json";

/// Scan a directory tree for `aurora-plugin.json` manifests.
pub fn scan_plugins_dir(dir: &Path) -> Result<Vec<PluginDescriptor>, PluginError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut results = Vec::new();
    scan_dir_recursive(dir, dir, &mut results)?;
    results.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
    Ok(results)
}

fn scan_dir_recursive(
    root: &Path,
    current: &Path,
    results: &mut Vec<PluginDescriptor>,
) -> Result<(), PluginError> {
    let entries = fs::read_dir(current).map_err(|e| PluginError::Io(e.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|e| PluginError::Io(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(root, &path, results)?;
            continue;
        }
        if path.file_name().is_some_and(|n| n == MANIFEST_NAME) {
            results.push(parse_manifest_file(&path)?);
        }
    }
    Ok(())
}

/// Load manifest from a `.wasm` sibling directory or direct manifest path.
pub fn load_wasm_manifest(path: &Path) -> Result<PluginDescriptor, PluginError> {
    if path.file_name().is_some_and(|n| n == MANIFEST_NAME) {
        return parse_manifest_file(path);
    }
    if path.extension().is_some_and(|ext| ext == "wasm") {
        let manifest_path = path.with_file_name(MANIFEST_NAME);
        if manifest_path.exists() {
            return parse_manifest_file(&manifest_path);
        }
        return Ok(wasm_fallback_descriptor(path));
    }
    let manifest_in_dir = path.join(MANIFEST_NAME);
    if manifest_in_dir.exists() {
        return parse_manifest_file(&manifest_in_dir);
    }
    Err(PluginError::NotFound(format!(
        "no {} found near {}",
        MANIFEST_NAME,
        path.display()
    )))
}

fn parse_manifest_file(manifest_path: &Path) -> Result<PluginDescriptor, PluginError> {
    let bytes = fs::read(manifest_path).map_err(|e| PluginError::Io(e.to_string()))?;
    let manifest: PluginManifest =
        serde_json::from_slice(&bytes).map_err(|e| PluginError::ManifestInvalid(e.to_string()))?;
    validate_manifest(&manifest)?;
    let plugin_dir = manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let wasm_path = plugin_dir.join(format!("{}.wasm", manifest.id.rsplit('.').next().unwrap_or("plugin")));
    let load_path = if wasm_path.exists() {
        wasm_path
    } else {
        manifest_path.to_path_buf()
    };
    Ok(PluginDescriptor {
        manifest,
        state: PluginState::Discovered,
        load_path,
    })
}

fn wasm_fallback_descriptor(wasm_path: &Path) -> PluginDescriptor {
    let stem = wasm_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("plugin");
    PluginDescriptor {
        manifest: PluginManifest {
            id: format!("com.aurora.plugins.{stem}"),
            name: stem.to_string(),
            version: "0.0.0".into(),
            api_version: crate::API_VERSION.into(),
            description: "WASM plugin (manifest inferred)".into(),
            author: "unknown".into(),
            license: "unknown".into(),
            plugin_type: PluginType::Style,
            trust_level: "community".into(),
            execution_tier: "t1_wasm".into(),
            min_engine_version: crate::HOST_VERSION.into(),
            parameters: Vec::new(),
        },
        state: PluginState::Discovered,
        load_path: wasm_path.to_path_buf(),
    }
}

fn validate_manifest(manifest: &PluginManifest) -> Result<(), PluginError> {
    if manifest.id.is_empty() {
        return Err(PluginError::ManifestInvalid("missing id".into()));
    }
    if manifest.api_version != crate::API_VERSION {
        return Err(PluginError::ManifestInvalid(format!(
            "api_version {} != host {}",
            manifest.api_version,
            crate::API_VERSION
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_missing_dir_returns_empty() {
        let found = scan_plugins_dir(Path::new("/nonexistent/aurora/plugins")).unwrap();
        assert!(found.is_empty());
    }
}
