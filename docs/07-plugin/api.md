# Plugin API Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Plugin & UI Research Agent  
**Dependencies:** [ACAS v0.1](../00-overview/acas-v0.1.md), [philosophy.md](../00-overview/philosophy.md), [ast.md](../02-music-model/ast.md), [architecture.md](../01-architecture/architecture.md), [pipeline.md](../01-architecture/pipeline.md), [ADR-005](../decisions/ADR-005-plugin-sandbox-model.md)

---

## Table of Contents

1. [Background](#1-background)
2. [Existing Solutions](#2-existing-solutions)
3. [Academic / Theoretical Foundation](#3-academic--theoretical-foundation)
4. [Engineering Analysis](#4-engineering-analysis)
5. [Comparison of Approaches](#5-comparison-of-approaches)
6. [Recommended Solution](#6-recommended-solution)
7. [Architecture](#7-architecture)
8. [Data Structures](#8-data-structures)
9. [Algorithms](#9-algorithms)
10. [Interfaces](#10-interfaces)
11. [Parameter Mappings](#11-parameter-mappings)
12. [Explainability Model](#12-explainability-model)
13. [Future Expansion](#13-future-expansion)
14. [Open Questions](#14-open-questions)
15. [References](#15-references)

---

## 1. Background

Aurora Composer's core engine provides infrastructure — AST, rule engine, search, pipeline orchestration — while **specialized musical knowledge** lives in plugins. Style-specific harmony rules, rhythm pattern libraries, motif transformation algorithms, optional AI augmentation, and custom export formats are all extension points.

### 1.1 Problem Statement

The engine must support:

- **Hot-loadable extensions** without recompiling the host
- **AST-only I/O** — plugins never bypass the music model
- **Mandatory provenance** on every plugin-generated event
- **Sandboxed execution** for third-party code (see ADR-005)
- **Pipeline integration** — plugins attach to specific stages
- **Parameter registration** — plugins declare consumed parameters

### 1.2 Scope

This document specifies:

- Six plugin trait families: `StylePlugin`, `HarmonyPlugin`, `RhythmPlugin`, `ThemePlugin`, `AIPlugin`, `ExportPlugin`
- Base `Plugin` trait and lifecycle
- Manifest format (`aurora-plugin.json`)
- Plugin Host integration with Tauri shell
- Loading, activation, sandboxing, and error handling

Out of scope: SDK tooling (see [sdk.md](sdk.md)), UI plugin manager (see [vue.md](../08-ui/vue.md)).

### 1.3 Design Principles (Binding)

| Principle | Plugin Implication |
|-----------|-------------------|
| Everything Is Music AST | Input: `AstSnapshot`; Output: `Patch` |
| Everything Is Explainable | `ProvenanceAgent::Plugin` required |
| Modular and Pluggable | Core works with zero plugins |
| Specification Before Code | This document is normative |

---

## 2. Existing Solutions

### 2.1 Audio Plugin Standards (VST/AU/CLAP)

Binary plugins loaded into DAW host process. Communicate via audio buffers and MIDI events.

**Relevant:** Hot-load pattern, preset/state serialization, vendor metadata.

**Insufficient:** No composition AST, no rule provenance, audio-centric ABI.

### 2.2 VS Code Extension Model

Manifest (`package.json`) declares capabilities. Extensions run in extension host with API surface restrictions.

**Relevant:** Capability-based sandbox, marketplace distribution, semver compatibility.

**Insufficient:** JavaScript runtime; not suitable for Rust search performance.

### 2.3 Obsidian Community Plugins

JavaScript plugins with `manifest.json`, loaded from user data directory.

**Relevant:** Simple manifest, community distribution, enable/disable per plugin.

**Insufficient:** No sandbox for untrusted code in early versions.

### 2.4 Extism / WASM Plugin Systems

Host functions expose limited API; guest WASM module calls host for I/O.

**Relevant:** Memory-safe sandbox, cross-language plugins.

**Insufficient:** Serialization overhead for large AST; requires host function design.

### 2.5 OpenMusic Libraries

Lisp packages extending composition environment.

**Relevant:** Algorithmic composition extension pattern.

**Insufficient:** Closed ecosystem, no provenance, no desktop sandbox.

---

## 3. Academic / Theoretical Foundation

### 3.1 Separation of Mechanism and Policy

Operating systems research (Lampson, 1974) distinguishes **mechanism** (how to do something) from **policy** (what to do). Aurora's core provides mechanism (search, AST, rules); plugins provide policy (jazz voicing rules, bossa rhythm patterns).

### 3.2 Constraint-Based Composition Plugins

Pachet's Continuator and E-Chord systems demonstrate pluggable constraint modules for style-specific generation. Aurora generalizes this with typed plugin traits bound to pipeline stages.

### 3.3 Explainable Plugin Outputs

Plugin decisions must be traceable (Samek et al., 2019). Every plugin mutation records which plugin rule or algorithm produced it, with eval_score where applicable.

---

## 4. Engineering Analysis

### 4.1 Evaluation Criteria

| Criterion | Requirement | Design Response |
|-----------|-------------|-----------------|
| Correctness | Plugins cannot corrupt AST invariants | Patch validation post-apply |
| Controllability | User enables/disables plugins | Manifest + ProjectStore config |
| Explainability | Full provenance on plugin events | Host rejects missing provenance |
| Performance | Harmony search < 60s | T0 native for bundled plugins |
| Extensibility | New plugin types without host rewrite | Trait registry pattern |
| Security | Third-party isolation | Tiered sandbox (ADR-005) |

### 4.2 Rust Implementation Considerations

- **`libloading`** for T0 dynamic library load
- **`wasmtime`** for T1 WASM execution
- **`async_trait`** for plugins with async I/O (export, AI)
- **Plugin API version** checked at load time
- **`abi_stable`** or manual C ABI for cross-compiler plugin builds (Phase 2 decision)

### 4.3 Threading Model

- Plugin `apply()` called from pipeline stage thread (sync) or job pool (async for AI/export)
- Plugins MUST be `Send` if invoked from thread pool
- Reentrant calls prohibited — one `apply()` per plugin instance at a time

---

## 5. Comparison of Approaches

### 5.1 Loading Mechanism

| Approach | Load Time | Sandbox | Performance | Verdict |
|----------|-----------|---------|-------------|---------|
| Static linking | N/A | N/A | Best | Bundled core only |
| Dynamic lib (T0) | ~10ms | Capability | Best | **Bundled + signed** |
| WASM (T1) | ~50ms | Strong | Good | **Community** |
| Subprocess (T2) | ~200ms | Strongest | IPC overhead | **AI only** |
| Scripting (Lua/Rhai) | ~5ms | Moderate | Poor for search | Rejected |

### 5.2 Plugin Granularity

| Granularity | Pros | Cons | Verdict |
|-------------|------|------|---------|
| Monolithic style pack | Simple UX | Inflexible | Bundled presets only |
| Per-engine plugin | Fine-grained control | Complex selection | **Selected** |
| Per-rule plugin | Maximum modularity | Overhead | Rejected for v0.1 |

---

## 6. Recommended Solution

Aurora adopts a **trait-based plugin system** with:

1. **Six typed traits** extending a common `Plugin` base
2. **Manifest-driven discovery** and capability enforcement
3. **Tiered sandbox** per ADR-005 (T0/T1/T2)
4. **Pipeline stage binding** via `stage_hooks` in manifest
5. **Unified Plugin Host** in Tauri shell (Layer 4)

Core engine ships with default classical/pop rule set — fully functional with zero user plugins.

---

## 7. Architecture

### 7.1 Component Diagram

```text
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Shell (L4)                         │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │CommandRouter│──│ PluginHost   │──│ CapabilityEnforcer  │  │
│  └─────────────┘  └──────┬───────┘  └─────────────────────┘  │
│                          │                                   │
│         ┌────────────────┼────────────────┐                 │
│         ▼                ▼                ▼                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐            │
│  │ T0 Native  │  │ T1 WASM    │  │ T2 Process │            │
│  │ Loader     │  │ Runtime    │  │ Spawner    │            │
│  └────────────┘  └────────────┘  └────────────┘            │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│              Pipeline Orchestrator (L3)                        │
│  StyleResolver ──► HarmonyEngine ──► ... ──► Export         │
│       │                │                              │        │
│       ▼                ▼                              ▼        │
│  StylePlugin      HarmonyPlugin                  ExportPlugin  │
└───────────────────────────────────────────────────────────────┘
```

### 7.2 Plugin Lifecycle

```text
Discover → Validate Manifest → Load Binary → Register Traits
    → Activate (enable in project) → Invoke (per stage) → Deactivate → Unload
```

| Phase | Actor | Action |
|-------|-------|--------|
| Discover | PluginHost | Scan plugin directories |
| Validate | PluginHost | Check manifest schema, API version, signature |
| Load | Loader (T0/T1/T2) | Map binary, init runtime |
| Register | PluginHost | Insert into trait registry by `plugin_type` |
| Activate | User/StyleResolver | Add to project's active plugin set |
| Invoke | Pipeline stage | Call trait method with AstSnapshot + params |
| Deactivate | User | Remove from active set; keep loaded |
| Unload | PluginHost | Drop instance, free resources |

### 7.3 Discovery Paths

```text
{app_data}/Aurora Composer/plugins/     — user-installed
{app_resources}/plugins/                 — bundled (Baroque, Jazz Standard, ...)
{project}/.aurora/plugins/               — project-local overrides
```

### 7.4 Tauri Integration

Plugin Host runs in Tauri backend Rust process. Frontend accesses plugins only via Tauri commands:

- `list_plugins`, `install_plugin`, `enable_plugin`, `disable_plugin`, `get_plugin_config`, `set_plugin_config`

Pipeline invocation is internal — frontend never calls plugin `apply()` directly.

---

## 8. Data Structures

### 8.1 Plugin Manifest (`aurora-plugin.json`)

```json
{
  "$schema": "https://aurora-composer.dev/schemas/plugin-manifest-v1.json",
  "id": "com.aurora.plugins.jazz-standard",
  "name": "Jazz Standard Style Pack",
  "version": "1.0.0",
  "api_version": "0.1.0",
  "description": "ii-V-I progressions, walking bass patterns, jazz voicing rules",
  "author": "Aurora Team",
  "license": "MIT",
  "plugin_type": "style",
  "trust_level": "bundled",
  "execution_tier": "t0_native",
  "entry": {
    "native": "libaurora_jazz_standard.so"
  },
  "stage_hooks": [
    { "stage": "StyleResolver", "priority": 100 },
    { "stage": "HarmonySkeleton", "priority": 50, "plugin_trait": "harmony" }
  ],
  "parameters": [
    { "key": "harmony.complexity", "default_override": 0.7 },
    { "key": "harmony.dissonance", "default_override": 0.4 }
  ],
  "permissions": {
    "ast_read": ["Composition", "Section", "Measure", "HarmonySlot", "Event"],
    "ast_write": ["HarmonySlot", "Event"],
    "network": false,
    "filesystem": [],
    "max_memory_mb": 128,
    "max_cpu_ms_per_call": 10000
  },
  "dependencies": [],
  "min_engine_version": "0.1.0"
}
```

### 8.2 Plugin Descriptor (Runtime)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub load_path: PathBuf,
    pub instance_id: Option<PluginInstanceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Discovered,
    Loaded,
    Active,
    Error,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginInstanceId(pub u64);
```

### 8.3 Plugin Context

```rust
#[derive(Clone, Debug)]
pub struct PluginContext {
    pub project_id: ProjectId,
    pub session_id: String,
    pub stage: PipelineStageId,
    pub parameters: ParameterSnapshot,
    pub weight_table: WeightTable,
    pub seed: Option<u64>,
    pub host_version: String,
    pub logger: PluginLogger,
}
```

### 8.4 Plugin Error

```rust
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum PluginError {
    #[error("manifest invalid: {0}")]
    ManifestInvalid(String),
    #[error("api version mismatch: plugin {plugin} vs host {host}")]
    ApiVersionMismatch { plugin: String, host: String },
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("apply failed: {0}")]
    ApplyFailed(String),
    #[error("provenance missing on event {node_id:?}")]
    ProvenanceMissing { node_id: NodeId },
    #[error("timeout after {ms}ms")]
    Timeout { ms: u64 },
    #[error("sandbox violation: {0}")]
    SandboxViolation(String),
}
```

### 8.5 Plugin Registry

```rust
pub struct PluginRegistry {
    style: HashMap<PluginId, Arc<dyn StylePlugin>>,
    harmony: HashMap<PluginId, Arc<dyn HarmonyPlugin>>,
    rhythm: HashMap<PluginId, Arc<dyn RhythmPlugin>>,
    theme: HashMap<PluginId, Arc<dyn ThemePlugin>>,
    ai: HashMap<PluginId, Arc<dyn AIPlugin>>,
    export: HashMap<PluginId, Arc<dyn ExportPlugin>>,
}
```

---

## 9. Algorithms

### 9.1 Discovery Algorithm

```text
function discover_plugins(paths[]) → PluginDescriptor[]:
    results = []
    for path in paths:
        for manifest_path in glob(path, "**/aurora-plugin.json"):
            manifest = parse_json(manifest_path)
            validate_schema(manifest)
            entry_binary = resolve_entry(manifest, manifest_path.parent)
            results.push(PluginDescriptor { manifest, state: Discovered, ... })
    return results sorted by manifest.id
```

### 9.2 Load Algorithm

```text
function load_plugin(descriptor) → Result<PluginInstance>:
    if descriptor.manifest.api_version != HOST_API_VERSION:
        return Err(ApiVersionMismatch)
    if descriptor.manifest.trust_level == "community" && !user_accepted(descriptor.id):
        return Err(UserConsentRequired)
    tier = descriptor.manifest.execution_tier
    instance = match tier:
        T0 → NativeLoader::load(descriptor.entry.native)
        T1 → WasmRuntime::load(descriptor.entry.wasm)
        T2 → ProcessSpawner::spawn(descriptor.entry.binary)
    register_traits(instance, descriptor.manifest.plugin_type)
    descriptor.state = Loaded
    return instance
```

### 9.3 Stage Invocation Algorithm

```text
function invoke_stage_plugins(stage, snapshot, params, active_plugins[]):
    hooks = filter_hooks(active_plugins, stage) sorted by priority desc
    accumulated_patch = empty Patch
    for hook in hooks:
        plugin = registry.get(hook.plugin_trait, hook.plugin_id)
        ctx = PluginContext::new(stage, params)
        patch = plugin.apply(snapshot, ctx) with timeout(hook.max_cpu_ms)
        validate_permissions(patch, plugin.manifest.permissions)
        validate_provenance(patch)
        accumulated_patch = merge_patches(accumulated_patch, patch)
        snapshot = apply_patch(snapshot, patch)  // sequential composition
    return accumulated_patch
```

### 9.4 Provenance Validation

```text
function validate_provenance(patch):
    for op in patch.ops:
        if op creates or modifies Event:
            if event.provenance is empty OR event.provenance.agent is not Plugin:
                return Err(ProvenanceMissing)
            if event.provenance.explanation is None:
                log_warn("plugin should provide explanation")
    return Ok
```

### 9.5 Patch Merge

Patches from multiple plugins on same stage merge sequentially. Conflicting writes (same `NodeId`) → later plugin wins with provenance chain linking `parent` to prior event.

---

## 10. Interfaces

### 10.1 Base Plugin Trait

```rust
/// Common interface for all Aurora plugins.
pub trait Plugin: Send + Sync {
    /// Unique plugin ID matching manifest `id`.
    fn id(&self) -> &str;

    /// Semantic version from manifest.
    fn version(&self) -> &semver::Version;

    /// Plugin type discriminator.
    fn plugin_type(&self) -> PluginType;

    /// AST node types this plugin reads (for permission audit).
    fn read_set(&self) -> AstReadSet;

    /// AST node types this plugin may write.
    fn write_set(&self) -> AstWriteSet;

    /// Parameters this plugin consumes or overrides.
    fn parameters(&self) -> &[ParameterSpec];

    /// Lifecycle: called once after load.
    fn on_load(&self, host: &PluginHostApi) -> Result<(), PluginError>;

    /// Lifecycle: called before unload.
    fn on_unload(&self) -> Result<(), PluginError>;

    /// Health check for plugin manager UI.
    fn health(&self) -> PluginHealth;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    Style,
    Harmony,
    Rhythm,
    Theme,
    Ai,
    Export,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_invoked: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus { Ok, Degraded, Error }
```

### 10.2 StylePlugin

Resolves genre presets into parameter bundles and plugin activations. Runs at pipeline Stage 1.

```rust
pub trait StylePlugin: Plugin {
    /// Human-readable style names this plugin provides.
    fn style_presets(&self) -> &[StylePreset];

    /// Resolve user style selection into parameter bundle + plugin list.
    fn resolve_style(
        &self,
        request: &StyleResolveRequest,
    ) -> Result<StyleResolveResult, PluginError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StylePreset {
    pub id: String,           // e.g., "jazz-standard"
    pub display_name: String,
    pub description: String,
    pub era: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleResolveRequest {
    pub preset_id: String,
    pub user_overrides: ParameterSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleResolveResult {
    pub parameters: ParameterSnapshot,
    pub active_plugins: Vec<PluginActivation>,
    pub pipeline_overrides: Option<PipelineConfig>,
    pub provenance: Provenance,  // metadata-level, not per-event
}
```

### 10.3 HarmonyPlugin

Extends harmony skeleton and voicing generation. Hooks: Stage 5 (Harmony Skeleton), optionally voicing sub-stage.

```rust
pub trait HarmonyPlugin: Plugin {
    /// Chord vocabulary this plugin contributes.
    fn chord_vocabulary(&self) -> &ChordVocabulary;

    /// Generate or refine harmony skeleton for a section.
    fn generate_harmony(
        &self,
        snapshot: &AstSnapshot,
        section_id: NodeId,
        ctx: &PluginContext,
    ) -> Result<Patch, PluginError>;

    /// Optional: generate voicing for a chord event.
    fn voice_chord(
        &self,
        snapshot: &AstSnapshot,
        harmony_slot_id: NodeId,
        ctx: &PluginContext,
    ) -> Result<Patch, PluginError> {
        Err(PluginError::ApplyFailed("voice_chord not implemented".into()))
    }

    /// Rules contributed to rule engine when this plugin is active.
    fn contributed_rules(&self) -> &[RuleContribution];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleContribution {
    pub rule_id: RuleId,
    pub category: String,
    pub description: String,
    pub default_weight: f64,
}
```

### 10.4 RhythmPlugin

Provides metric patterns and subdivision strategies. Hooks: Stage 6 (Rhythm Skeleton).

```rust
pub trait RhythmPlugin: Plugin {
    /// Pattern library metadata.
    fn pattern_catalog(&self) -> &[RhythmPatternMeta];

    /// Apply rhythm skeleton to measures in scope.
    fn apply_rhythm(
        &self,
        snapshot: &AstSnapshot,
        measure_range: MeasureRange,
        ctx: &PluginContext,
    ) -> Result<Patch, PluginError>;

    /// Optional: fill drum-adjacent rhythmic accents on melodic voices.
    fn apply_accent_pattern(
        &self,
        snapshot: &AstSnapshot,
        voice_id: VoiceId,
        measure_range: MeasureRange,
        ctx: &PluginContext,
    ) -> Result<Patch, PluginError> {
        Ok(Patch::empty())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RhythmPatternMeta {
    pub id: String,
    pub name: String,
    pub time_signatures: Vec<TimeSignature>,
    pub style_tags: Vec<String>,
    pub density_range: (f32, f32),
}
```

### 10.5 ThemePlugin

Motif transformation and theme development algorithms. Hooks: Stage 4 (Theme Planning), decoration.

```rust
pub trait ThemePlugin: Plugin {
    /// Available motif transformation strategies.
    fn transformations(&self) -> &[ThemeTransformSpec];

    /// Plan theme allocation across sections.
    fn plan_themes(
        &self,
        snapshot: &AstSnapshot,
        ctx: &PluginContext,
    ) -> Result<Patch, PluginError>;

    /// Transform a source motif into target section.
    fn transform_motif(
        &self,
        snapshot: &AstSnapshot,
        source_theme_id: &str,
        target_section_id: NodeId,
        transform: ThemeTransform,
        ctx: &PluginContext,
    ) -> Result<Patch, PluginError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeTransformSpec {
    pub transform: ThemeTransform,
    pub display_name: String,
    pub parameter_keys: Vec<String>,
}
```

### 10.6 AIPlugin

Proposes search candidates or adjusts weights. **Never** writes events directly — only influences search. Hooks: any search stage (7–9). Execution tier T2.

```rust
pub trait AIPlugin: Plugin {
    /// Declares which pipeline stages this AI plugin may influence.
    fn target_stages(&self) -> &[PipelineStageId];

    /// Propose additional candidates for search beam.
    fn propose_candidates(
        &self,
        snapshot: &AstSnapshot,
        search_ctx: &SearchContext,
        ctx: &PluginContext,
    ) -> Result<Vec<CandidateProposal>, PluginError>;

    /// Optional: adjust rule weights dynamically.
    fn adjust_weights(
        &self,
        base_weights: &WeightTable,
        snapshot: &AstSnapshot,
        ctx: &PluginContext,
    ) -> Result<WeightTable, PluginError> {
        Ok(base_weights.clone())
    }

    /// Requires explicit user opt-in and API key.
    fn requires_network(&self) -> bool;
    fn requires_api_key(&self) -> bool;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateProposal {
    pub event: Event,
    pub prior_score: Option<f64>,
    pub explanation: String,
    /// AI proposals still get provenance with source Generated + agent Plugin
}
```

**Invariant:** AIPlugin MUST NOT bypass search scoring. All proposals evaluated by rule engine.

### 10.7 ExportPlugin

Custom export formats beyond built-in MusicXML/MIDI/ABC/PDF.

```rust
pub trait ExportPlugin: Plugin {
    /// File extension and MIME type.
    fn format_info(&self) -> ExportFormatInfo;

    /// Export IR to custom format. File write via host `export_write`.
    fn export(
        &self,
        ir: &MusicIr,
        options: &ExportOptions,
        ctx: &PluginContext,
    ) -> Result<ExportResult, PluginError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportFormatInfo {
    pub id: String,
    pub display_name: String,
    pub extension: String,
    pub mime_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub bytes: Vec<u8>,
    pub suggested_filename: String,
}
```

### 10.8 Plugin Host API (Host → Plugin)

```rust
pub trait PluginHostApi {
    fn log(&self, level: LogLevel, message: &str);
    fn get_parameter(&self, key: &str) -> Option<JsonValue>;
    fn export_write(&self, path: &Path, bytes: &[u8]) -> Result<(), PluginError>;
    fn request_network(&self, url: &str, body: &[u8]) -> Result<Vec<u8>, PluginError>;
    fn engine_version(&self) -> &str;
}
```

### 10.9 Plugin Host (Internal Rust API)

```rust
pub struct PluginHost {
    registry: PluginRegistry,
    descriptors: HashMap<PluginId, PluginDescriptor>,
    capability_enforcer: CapabilityEnforcer,
}

impl PluginHost {
    pub fn discover(&mut self) -> Result<Vec<PluginDescriptor>, PluginError>;
    pub fn load(&mut self, plugin_id: &str) -> Result<(), PluginError>;
    pub fn unload(&mut self, plugin_id: &str) -> Result<(), PluginError>;
    pub fn activate(&mut self, project_id: ProjectId, plugin_id: &str) -> Result<(), PluginError>;
    pub fn deactivate(&mut self, project_id: ProjectId, plugin_id: &str) -> Result<(), PluginError>;
    pub fn invoke_for_stage(
        &self,
        stage: PipelineStageId,
        snapshot: &AstSnapshot,
        ctx: &PluginContext,
        active: &[PluginId],
    ) -> Result<Patch, PluginError>;
    pub fn get_style_plugin(&self, id: &str) -> Option<Arc<dyn StylePlugin>>;
    // ... getters for other trait types
}
```

### 10.10 Tauri Commands (Plugin Management)

| Command | Input | Output |
|---------|-------|--------|
| `list_plugins` | — | `PluginDescriptor[]` |
| `get_plugin` | `plugin_id` | `PluginDescriptor` |
| `install_plugin` | `path: string` | `PluginDescriptor` |
| `uninstall_plugin` | `plugin_id` | `()` |
| `enable_plugin` | `project_id`, `plugin_id` | `()` |
| `disable_plugin` | `project_id`, `plugin_id` | `()` |
| `get_plugin_config` | `plugin_id` | `JsonValue` |
| `set_plugin_config` | `plugin_id`, `config` | `()` |
| `get_style_presets` | — | `StylePreset[]` |

---

## 11. Parameter Mappings

Plugins register parameters via manifest `parameters[]` and trait `parameters()`.

| Plugin Type | Typical Parameters | Maps To |
|-------------|-------------------|---------|
| StylePlugin | genre, era, orchestration | Full parameter bundle |
| HarmonyPlugin | harmony.*, cadence.* | Rule weights, chord vocabulary |
| RhythmPlugin | rhythm.* | Pattern selection, subdivision |
| ThemePlugin | theme.* | Transform selection, repetition |
| AIPlugin | search.temperature, ai.* | Candidate count, weight deltas |
| ExportPlugin | export.<format>.* | Format-specific options |

Style plugins may override defaults:

```json
{ "key": "harmony.complexity", "default_override": 0.7 }
```

User overrides in UI always take precedence over plugin defaults.

---

## 12. Explainability Model

### 12.1 Plugin Provenance Requirements

Every Event created or modified by a plugin MUST have:

```rust
Provenance {
    source: ProvenanceSource::Plugin,
    stage: Some(current_stage),
    rule_ids: vec![...],           // from contributed_rules if applicable
    eval_score: Some(score),
    agent: ProvenanceAgent::Plugin { plugin_id: "com.aurora.plugins.jazz-standard" },
    explanation: Some("ii-V-I turnaround in measure 8"),
    ...
}
```

### 12.2 UI Display (Inspector)

When user clicks a plugin-generated note:

```text
Generated by: Jazz Standard Harmony Plugin
Rule: Harmony Rule #42 — Allow passing tones on weak beats
Score: +13
Reason: Passing tone between C4 and E4
```

See [inspector.md](../08-ui/inspector.md).

### 12.3 AI Plugin Transparency

AI proposals include `explanation` field. Inspector shows:

```text
Proposed by: AI Motif Plugin (candidate rank 3)
Evaluated by: Melody stage search, beam rank 2
Final Score: +8.5 (after rule engine)
```

---

## 13. Future Expansion

| Feature | Target | Notes |
|---------|--------|-------|
| Plugin marketplace | Phase 3 | Signed packages, revenue share TBD |
| Hot reload (dev) | v0.2 | `aurora-plugin dev --watch` |
| Lua scripting layer | Phase 3 | Thin scripts calling host API |
| Inter-plugin dependencies | v0.2 | manifest `dependencies[]` |
| Plugin presets | v0.2 | Save plugin config as preset |
| Cloud AI plugins | Phase 4 | T2 with OAuth |

---

## 14. Open Questions

| ID | Question | Status |
|----|----------|--------|
| OQ-PLG-1 | `abi_stable` vs C ABI for T0 plugins? | Prototype benchmark |
| OQ-PLG-2 | Allow multiple active HarmonyPlugins? | Yes, priority order; document conflicts |
| OQ-PLG-3 | Plugin config UI schema in manifest? | Defer to v0.2 |
| OQ-PLG-4 | WASM AST chunking for large compositions? | Benchmark at 10k measures |
| OQ-PLG-5 | Code signing authority for marketplace? | Phase 3 |

---

## 15. References

### Internal

- [ACAS v0.1](../00-overview/acas-v0.1.md)
- [philosophy.md](../00-overview/philosophy.md)
- [ast.md](../02-music-model/ast.md)
- [pipeline.md](../01-architecture/pipeline.md)
- [sdk.md](sdk.md)
- [tauri.md](../08-ui/tauri.md)
- [ADR-005](../decisions/ADR-005-plugin-sandbox-model.md)
- [Plugin & UI Research Notes](../../research/plugin-ui-research-notes.md)

### External

- Pachet & Roy (2001), *Musical Data Mining*
- Lampson (1974), Protection
- Extism: https://extism.org/
- Tauri 2 Plugin Development: https://v2.tauri.app/develop/plugins/
- CLAP: https://github.com/free-audio/clap
- WebAssembly System Interface (WASI): https://wasi.dev/

---

*End of Plugin API Specification*
