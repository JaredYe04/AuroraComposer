# Plugin SDK Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Plugin & UI Research Agent  
**Dependencies:** [api.md](api.md), [ast.md](../02-music-model/ast.md), [ADR-005](../decisions/ADR-005-plugin-sandbox-model.md)

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

The Aurora Plugin SDK enables third-party and internal developers to create plugins that extend the composition engine. The SDK provides scaffolding, type definitions, testing utilities, and publishing workflow — without requiring access to the full engine source.

### 1.1 Problem Statement

Plugin developers need:

- **Clear trait implementations** with compile-time API version checks
- **Project scaffolding** (`aurora-plugin new`)
- **Local testing** against a mock or dev host
- **Provenance helpers** to satisfy explainability invariants
- **Publishing** to local plugin directory or future marketplace

### 1.2 Scope

- Rust SDK crate: `aurora-plugin-sdk`
- CLI tool: `aurora-plugin` (scaffold, build, test, package)
- WASM SDK bindings for T1 plugins
- Testing harness and mock host
- Publishing workflow

Out of scope: Marketplace backend (Phase 3).

---

## 2. Existing Solutions

### 2.1 VS Code Extension Generator

`yo code` scaffolds extension with manifest, build config, and debug launch.

**Relevant:** CLI scaffolding, local debug loop.

### 2.2 Extism SDK

Language SDKs for WASM plugins with host function macros.

**Relevant:** Cross-language plugin development, `#[host_fn]` pattern.

### 2.3 Cargo `cargo-generate`

Template-based project creation.

**Relevant:** Rust plugin crate scaffolding.

### 2.4 Tauri Plugin Template

Official template for Tauri app-level plugins (not user plugins).

**Relevant:** Build integration patterns; distinct from Aurora user plugins.

---

## 3. Academic / Theoretical Foundation

### 3.1 API Stability and Versioning

SemVer (Preston-Werner, 2013) applied to `api_version` in manifest. Breaking AST schema changes increment major; plugin must rebuild.

### 3.2 Test-Driven Plugin Development

Plugins should ship with unit tests against mock AST fixtures before integration testing in Aurora host.

---

## 4. Engineering Analysis

| Criterion | SDK Response |
|-----------|--------------|
| Developer experience | Single CLI, template projects |
| Correctness | Provenance builder enforces invariants |
| Safety | `#[deny(missing_provenance)]` lint (Phase 2) |
| Cross-platform | `cargo build` targets for win/mac/linux |
| WASM support | `aurora-plugin-sdk-wasm` crate |

---

## 5. Comparison of Approaches

| Approach | DX | Safety | Verdict |
|----------|-----|--------|---------|
| Raw trait impl | Poor | Error-prone | Rejected |
| SDK + macros | Good | Guided | **Selected** |
| Code generation from manifest | Medium | Good | Optional v0.2 |
| Multiple language SDKs | Broad reach | WASM only initially | Phase 3 |

---

## 6. Recommended Solution

Ship **`aurora-plugin-sdk`** Rust crate and **`aurora-plugin`** CLI with:

1. Trait re-exports and version constants
2. `ProvenanceBuilder` helper
3. `PatchBuilder` for common AST mutations
4. Mock `PluginHostApi` for unit tests
5. Integration test runner against dev Aurora instance

---

## 7. Architecture

```text
Developer Workstation
├── aurora-plugin CLI
│   ├── new       → scaffold from template
│   ├── build     → cargo build --release + copy to plugins/
│   ├── test      → unit + integration tests
│   ├── package   → create .aurora-plugin zip
│   └── dev       → watch mode + hot reload (v0.2)
├── Plugin Project
│   ├── aurora-plugin.json
│   ├── Cargo.toml (depends on aurora-plugin-sdk)
│   ├── src/lib.rs (trait impl)
│   └── tests/
└── aurora-plugin-sdk (crate)
    ├── traits (re-export from api spec)
    ├── provenance
    ├── patch
    ├── mock_host
    └── fixtures
```

---

## 8. Data Structures

### 8.1 SDK Version Constants

```rust
pub const SDK_VERSION: &str = "0.1.0";
pub const API_VERSION: &str = "0.1.0";
pub const MIN_HOST_VERSION: &str = "0.1.0";
```

### 8.2 ProvenanceBuilder

```rust
pub struct ProvenanceBuilder {
    inner: Provenance,
}

impl ProvenanceBuilder {
    pub fn for_plugin(plugin_id: &str, stage: PipelineStageId) -> Self;
    pub fn rule(mut self, rule_id: RuleId) -> Self;
    pub fn score(mut self, eval_score: f64) -> Self;
    pub fn explanation(mut self, text: impl Into<String>) -> Self;
    pub fn search_context(mut self, ctx: SearchContext) -> Self;
    pub fn build(self) -> Provenance;
}
```

### 8.3 PatchBuilder

```rust
pub struct PatchBuilder {
    ops: Vec<PatchOp>,
    description: String,
}

impl PatchBuilder {
    pub fn new(description: impl Into<String>) -> Self;
    pub fn insert_note(
        &mut self,
        measure_id: NodeId,
        voice_id: VoiceId,
        event: NoteEvent,
    ) -> &mut Self;
    pub fn build(self) -> Patch;
}
```

### 8.4 Package Format (`.aurora-plugin`)

```text
my-plugin.aurora-plugin (zip)
├── aurora-plugin.json
├── README.md
├── lib/
│   ├── x86_64-pc-windows-msvc/aurora_my_plugin.dll
│   ├── x86_64-unknown-linux-gnu/libaurora_my_plugin.so
│   └── aarch64-apple-darwin/libaurora_my_plugin.dylib
└── wasm/aurora_my_plugin.wasm (optional T1)
```

---

## 9. Algorithms

### 9.1 Scaffold Algorithm (`aurora-plugin new`)

```text
function new_plugin(name, type):
    template = select_template(type)  // harmony, rhythm, style, ...
    copy_template(template, name)
    replace_placeholders(name, type, generate_id(name))
    init_cargo_toml_with_sdk_dependency()
    create_minimal_trait_impl()
    create_aurora_plugin_json()
    create_example_test()
```

### 9.2 Test Runner Algorithm

```text
function test_plugin(project_path):
    run cargo test --lib                    // unit tests with mock host
    build release binary
    copy to temp plugins directory
    spawn aurora dev --headless             // integration
    invoke test harness commands:
        load_plugin(id)
        activate_plugin(id)
        run_stage(HarmonySkeleton, fixture_ast)
        assert patch validates
        assert all events have provenance
```

### 9.3 Package Algorithm

```text
function package(project_path):
  validate manifest
  build all target triples (or current only for local)
  zip manifest + binaries + README
  compute checksum, embed in package metadata
```

---

## 10. Interfaces

### 10.1 CLI Commands

| Command | Description |
|---------|-------------|
| `aurora-plugin new <name> --type harmony` | Scaffold project |
| `aurora-plugin build [--release]` | Build plugin binary |
| `aurora-plugin test` | Run unit + integration tests |
| `aurora-plugin package` | Create `.aurora-plugin` archive |
| `aurora-plugin install` | Copy to user plugins directory |
| `aurora-plugin validate` | Lint manifest and permissions |
| `aurora-plugin dev` | Watch + reload (v0.2) |

### 10.2 SDK Public API

```rust
// aurora-plugin-sdk/lib.rs
pub use aurora_plugin_api::{
    Plugin, StylePlugin, HarmonyPlugin, RhythmPlugin,
    ThemePlugin, AIPlugin, ExportPlugin,
    PluginContext, PluginError, PluginType,
    AstSnapshot, Patch, PatchOp, Provenance, /* ... */
};
pub mod provenance;
pub mod patch;
pub mod mock;
pub mod fixtures;
```

### 10.3 Plugin Entry Point Macro

```rust
#[aurora_plugin(type = "harmony", id = "com.example.my-harmony")]
pub struct MyHarmonyPlugin;

#[aurora_plugin_impl]
impl HarmonyPlugin for MyHarmonyPlugin {
    fn generate_harmony(&self, snapshot, section_id, ctx) -> Result<Patch, PluginError> {
        let mut pb = PatchBuilder::new("my harmony generation");
        let prov = ProvenanceBuilder::for_plugin(self.id(), ctx.stage)
            .rule(RuleId::new("harmony.custom-ii-v-i"))
            .score(13.0)
            .explanation("ii-V-I in jazz style")
            .build();
        // ... build note events with prov
        Ok(pb.build())
    }
}

aurora_plugin_export!(MyHarmonyPlugin);
```

### 10.4 Mock Host (Testing)

```rust
pub struct MockPluginHost {
    pub parameters: HashMap<String, JsonValue>,
    pub logs: Vec<(LogLevel, String)>,
}

impl PluginHostApi for MockPluginHost { /* ... */ }

pub fn load_fixture(name: &str) -> AstSnapshot;  // from tests/fixtures/*.json
```

### 10.5 Example Test

```rust
#[test]
fn generates_harmony_with_provenance() {
    let plugin = MyHarmonyPlugin;
    let snapshot = fixtures::load("empty_8bar_section");
    let ctx = mock::context_with_defaults();
    let patch = plugin.generate_harmony(&snapshot, section_id, &ctx).unwrap();
    for event in patch.events() {
        assert!(event.provenance().agent.is_plugin());
        assert!(event.provenance().explanation.is_some());
    }
}
```

---

## 11. Parameter Mappings

SDK provides `ParameterAccessor` wrapping `PluginContext.parameters`:

```rust
pub struct ParameterAccessor<'a> {
    params: &'a ParameterSnapshot,
}

impl ParameterAccessor<'_> {
    pub fn get_f32(&self, key: &str, default: f32) -> f32;
    pub fn get_bool(&self, key: &str, default: bool) -> bool;
    pub fn harmony_complexity(&self) -> f32;  // typed accessors for common params
}
```

Manifest `parameters[]` auto-generates typed accessor stubs in v0.2.

---

## 12. Explainability Model

### 12.1 SDK Lint Rules (Recommended)

| Rule | Severity | Description |
|------|----------|-------------|
| `missing_provenance` | Error | Event without provenance in patch |
| `missing_explanation` | Warn | Plugin event without explanation |
| `invalid_agent` | Error | Agent not `Plugin { plugin_id }` |

### 12.2 Documentation Template

Scaffolded README includes provenance section:

```markdown
## Explainability

This plugin tags all generated events with:
- `rule_ids`: [list your rules]
- `explanation`: human-readable reason per event

Example Inspector output:
> Generated by: My Harmony Plugin
> Rule: harmony.custom-ii-v-i
> Score: +13
> Reason: ii-V-I turnaround
```

---

## 13. Future Expansion

| Feature | Version |
|---------|---------|
| TypeScript WASM SDK | v0.2 |
| Plugin config UI schema generator | v0.2 |
| `aurora-plugin publish` to marketplace | Phase 3 |
| Fuzz testing harness | v0.2 |
| AST diff visualizer for tests | v0.3 |

---

## 14. Open Questions

| ID | Question | Status |
|----|----------|--------|
| OQ-SDK-1 | Publish SDK on crates.io before engine? | Yes, with `api` feature flag |
| OQ-SDK-2 | Include Music21 import helper? | Defer — export plugin territory |
| OQ-SDK-3 | Cross-compile CI template? | GitHub Actions template in v0.2 |

---

## 15. References

### Internal

- [Plugin API](api.md)
- [ast.md](../02-music-model/ast.md)
- [ADR-005](../decisions/ADR-005-plugin-sandbox-model.md)

### External

- cargo-generate: https://github.com/cargo-generate/cargo-generate
- Extism SDK: https://extism.org/docs/quickstart/plugin-quickstart
- Semantic Versioning: https://semver.org/

---

*End of Plugin SDK Specification*
