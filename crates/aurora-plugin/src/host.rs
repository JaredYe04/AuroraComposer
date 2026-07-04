use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::builtin;
use crate::error::PluginError;
use crate::types::PluginManifest;
use crate::traits::{DynAiPlugin, DynStylePlugin, Plugin, PluginHostApi, StylePlugin};
use crate::types::{PluginDescriptor, PluginState, StylePreset, StyleResolveRequest, StyleResolveResult};
use crate::wasm_loader;
use crate::{API_VERSION, HOST_VERSION};

/// Plugin discovery, load, and stage invocation stub (api.md §10.9).
pub struct PluginHost {
    style_plugins: HashMap<String, DynStylePlugin>,
    ai_plugins: HashMap<String, DynAiPlugin>,
    descriptors: HashMap<String, PluginDescriptor>,
    plugins_dir: PathBuf,
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::with_builtins()
    }
}

impl PluginHost {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_builtins() -> Self {
        let mut host = Self {
            style_plugins: HashMap::new(),
            ai_plugins: HashMap::new(),
            descriptors: HashMap::new(),
            plugins_dir: PathBuf::from("plugins"),
        };
        host.register_style(Arc::new(builtin::ClassicalStylePlugin));
        host.register_style(Arc::new(builtin::JazzStylePlugin));
        host.register_style(Arc::new(builtin::PopStylePlugin));
        host.register_ai(Arc::new(builtin::AiStubPlugin));
        let _ = host.discover();
        host
    }

    #[must_use]
    pub fn with_plugins_dir(dir: impl Into<PathBuf>) -> Self {
        let mut host = Self::with_builtins();
        host.plugins_dir = dir.into();
        host
    }

    pub fn register_style(&mut self, plugin: DynStylePlugin) {
        let id = plugin.id().to_string();
        let descriptor = PluginDescriptor {
            manifest: PluginManifest::from_plugin(plugin.as_ref()),
            state: PluginState::Loaded,
            load_path: PathBuf::from(format!("builtin://{id}")),
        };
        self.descriptors.insert(id.clone(), descriptor);
        self.style_plugins.insert(id, plugin);
    }

    pub fn register_ai(&mut self, plugin: DynAiPlugin) {
        let id = plugin.id().to_string();
        let descriptor = PluginDescriptor {
            manifest: PluginManifest::from_plugin(plugin.as_ref()),
            state: PluginState::Loaded,
            load_path: PathBuf::from(format!("builtin://{id}")),
        };
        self.descriptors.insert(id.clone(), descriptor);
        self.ai_plugins.insert(id, plugin);
    }

    #[must_use]
    pub fn list_plugins(&self) -> Vec<PluginDescriptor> {
        let mut list: Vec<_> = self.descriptors.values().cloned().collect();
        list.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
        list
    }

    #[must_use]
    pub fn style_plugin_ids(&self) -> Vec<String> {
        let mut ids: Vec<_> = self.style_plugins.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn get_style_plugin(&self, id: &str) -> Option<DynStylePlugin> {
        self.style_plugins.get(id).cloned()
    }

    pub fn discover(&mut self) -> Result<Vec<PluginDescriptor>, PluginError> {
        let discovered = wasm_loader::scan_plugins_dir(&self.plugins_dir)?;
        for descriptor in discovered {
            self.descriptors
                .entry(descriptor.manifest.id.clone())
                .or_insert(descriptor);
        }
        Ok(self.list_plugins())
    }

    pub fn register_wasm_plugin(&mut self, path: impl AsRef<Path>) -> Result<(), PluginError> {
        let descriptor = wasm_loader::load_wasm_manifest(path.as_ref())?;
        self.descriptors
            .insert(descriptor.manifest.id.clone(), descriptor);
        Ok(())
    }

    #[must_use]
    pub fn list_wasm_plugins(&self) -> Vec<PluginDescriptor> {
        self.descriptors
            .values()
            .filter(|d| {
                d.load_path
                    .extension()
                    .is_some_and(|ext| ext == "wasm" || ext == "json")
                    || d.manifest.execution_tier == "t1_wasm"
            })
            .cloned()
            .collect()
    }

    pub fn load(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        if self.style_plugins.contains_key(plugin_id) {
            return Ok(());
        }
        Err(PluginError::NotFound(plugin_id.into()))
    }

    pub fn resolve_style(
        &self,
        plugin_id: &str,
        request: &StyleResolveRequest,
    ) -> Result<StyleResolveResult, PluginError> {
        let plugin = self
            .style_plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.into()))?;
        plugin.resolve_style(request)
    }

    pub fn resolve_style_for_genre(
        &self,
        genre: &str,
        params: &aurora_core::ParameterBundle,
    ) -> Result<StyleResolveResult, PluginError> {
        let plugin_id = map_genre_to_plugin(genre);
        let request = StyleResolveRequest {
            preset_id: genre.to_lowercase(),
            user_overrides: params.clone(),
        };
        self.resolve_style(plugin_id, &request)
    }

    #[must_use]
    pub fn all_style_presets(&self) -> Vec<StylePreset> {
        let mut presets = Vec::new();
        for plugin in self.style_plugins.values() {
            presets.extend(plugin.style_presets().iter().cloned());
        }
        presets
    }
}

fn map_genre_to_plugin(genre: &str) -> &'static str {
    match genre.to_lowercase().as_str() {
        "jazz" | "blues" | "fusion" | "swing" | "bebop" => "com.aurora.plugins.jazz-style",
        "classical" | "baroque" | "romantic" | "chamber" => "com.aurora.plugins.classical-style",
        _ => "com.aurora.plugins.pop-style",
    }
}

/// Minimal host API for plugin lifecycle.
pub struct StubPluginHostApi;

impl PluginHostApi for StubPluginHostApi {
    fn engine_version(&self) -> &str {
        HOST_VERSION
    }

    fn log(&self, _level: &str, _message: &str) {}
}

impl PluginManifest {
    #[must_use]
    pub fn from_plugin(plugin: &dyn Plugin) -> Self {
        Self {
            id: plugin.id().into(),
            name: plugin.id().into(),
            version: plugin.version().into(),
            api_version: API_VERSION.into(),
            description: String::new(),
            author: "Aurora Team".into(),
            license: "MIT".into(),
            plugin_type: plugin.plugin_type(),
            trust_level: "bundled".into(),
            execution_tier: "t0_native".into(),
            min_engine_version: HOST_VERSION.into(),
            parameters: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_plugins_discovered() {
        let host = PluginHost::new();
        let plugins = host.list_plugins();
        assert_eq!(plugins.len(), 4);
        assert_eq!(host.style_plugin_ids().len(), 3);
    }

    #[test]
    fn jazz_genre_resolves_jazz_plugin() {
        let host = PluginHost::new();
        let params = aurora_core::ParameterBundle::default();
        let result = host
            .resolve_style_for_genre("jazz", &params)
            .expect("resolve");
        assert!(result.jazz_harmony);
        assert!(result.active_bundles.iter().any(|b| b.contains("JAZZ")));
    }
}
