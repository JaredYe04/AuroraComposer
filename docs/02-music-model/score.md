# Score Container Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Music AST Research Agent  
**Dependencies:** [ast.md](ast.md), [timeline.md](timeline.md), [voices.md](voices.md)

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

In Aurora terminology:

- **Composition** — the AST root node (see [terminology.md](../00-overview/terminology.md))
- **Score** — the **container** holding a Composition plus project context: metadata, parameters, history, and export cache

This document specifies the **Score container**, **global attributes**, **project structure**, and **metadata schemas** — the envelope around the musical AST.

> **Disambiguation:** `eval_score` (rule engine numeric evaluation) ≠ Score container (project envelope).

---

## 2. Existing Solutions

| System | Container Model |
|--------|----------------|
| MuseScore | `.mscz` archive: score + images + styles |
| DAW projects | JSON/binary + audio assets |
| MusicXML | Single score file, limited metadata |
| Git + ABC | Plain text, no parameters |
| Aurora (proposed) | `.aurora` project bundle |

---

## 3. Academic / Theoretical Foundation

### 3.1 Work vs Score vs Performance

Musicological distinction (Dahlhaus):

- **Work** — abstract musical entity
- **Score** — symbolic representation
- **Performance** — acoustic realization

Aurora **Composition** ≈ Work + Score; **MusicIr** ≈ Performance schedule; **Score container** ≈ archival project.

### 3.2 Metadata for Reproducibility

Computational creativity research emphasizes storing **generation parameters** for reproducibility — Aurora's `ParameterSnapshot` and `provenance_root.seed`.

---

## 4. Engineering Analysis

### 4.1 Project File Requirements

- Serializable Composition AST
- User parameters (for regeneration)
- Undo history (patch stack)
- Optional cached IR
- Plugin configuration
- Version migration support

### 4.2 Size Estimates

| Component | Typical Size |
|-----------|-------------|
| Composition AST | 500 KB – 5 MB JSON |
| Parameter snapshot | 10 KB |
| Patch history (100 edits) | 200 KB |
| Cached IR | 300 KB |
| Total `.aurora` bundle | 1–10 MB |

---

## 5. Comparison of Approaches

| Storage | Pros | Cons | Verdict |
|---------|------|------|---------|
| Single JSON file | Simple | Large, no assets | Draft export |
| **ZIP bundle `.aurora`** | Assets + manifest | Slightly complex | **Selected** |
| SQLite | Queryable | Overkill Phase 1 | Future |
| Directory project | Git-friendly | UX friction | Optional git mode |

---

## 6. Recommended Solution

```text
Project (Score container)
├── manifest.json
├── composition.cbor          // canonical AST
├── parameters.json
├── history/
│   └── patches/*.json
├── cache/
│   └── ir.cbor               // optional
├── assets/                   // optional audio/images
└── plugins.json
```

UI and Tauri operate on **`Project`**, which wraps **`Composition`**.

---

## 7. Architecture

```text
┌─────────────────────────────────────────┐
│              Project (Score)             │
│  ┌─────────────┐  ┌──────────────────┐  │
│  │ ProjectMeta │  │ ParameterSnapshot│  │
│  └─────────────┘  └──────────────────┘  │
│  ┌─────────────────────────────────────┐│
│  │         Composition (AST root)       ││
│  │  metadata · global · movements[]   ││
│  └─────────────────────────────────────┘│
│  ┌─────────────┐  ┌──────────────────┐  │
│  │ PatchHistory│  │ ExportCache      │  │
│  └─────────────┘  └──────────────────┘  │
└─────────────────────────────────────────┘
```

---

## 8. Data Structures

### 8.1 Project Root

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub manifest: ProjectManifest,
    pub composition: Composition,
    pub parameters: ParameterSnapshot,
    pub history: PatchHistory,
    pub export_cache: Option<ExportCache>,
    pub plugin_config: PluginConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project_id: String,           // UUID
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
```

### 8.2 CompositionMetadata (AST-Level)

Embedded in `Composition.metadata`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompositionMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub copyright: Option<String>,
    pub license: Option<String>,
    pub created_at: String,
    pub modified_at: String,
    pub language: Option<String>,
    pub parameters_used: ParameterSnapshot,
    pub emotion_profile: Option<EmotionProfile>,
    pub provenance_root: ProvenanceRoot,
    pub tags: Vec<String>,
    pub source: CompositionSource,
    pub layout: ScoreLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositionSource {
    Generated,
    Imported,
    Hybrid,
    Manual,
}
```

### 8.3 GlobalAttributes

On `Composition.global`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalAttributes {
    pub default_key: KeySignature,
    pub default_meter: TimeSignature,
    pub tempo_map: TempoMap,
    pub key_map: KeyMap,
    pub meter_map: MeterMap,
    pub dynamics_baseline: DynamicLevel,
    pub pickup_measure: Option<PickupSpec>,
    pub display: GlobalDisplayOptions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PickupSpec {
    pub duration: BeatOffset,
    pub notated_measure_number: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalDisplayOptions {
    pub show_metronome: bool,
    pub show_rehearsal_marks: bool,
    pub page_layout: PageLayout,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageLayout {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub margins_mm: Margins,
    pub system_distance: f32,
}
```

See [timeline.md](timeline.md) for TempoMap, KeyMap, MeterMap.

### 8.4 ParameterSnapshot

Complete user parameter state at generation/edit time:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterSnapshot {
    pub version: u16,
    pub emotion: EmotionParams,
    pub style: StyleParams,
    pub mode: ModeParams,
    pub form: FormParams,
    pub theme: ThemeParams,
    pub harmony: HarmonyParams,
    pub voice: VoiceParams,
    pub texture: TextureParams,
    pub rhythm: RhythmParams,
    pub dynamics: DynamicsParams,
    pub counterpoint: CounterpointParams,
    pub drums: DrumsParams,
    pub search: SearchParams,
    pub register: RegisterParams,
    pub custom: HashMap<String, JsonValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmotionParams {
    pub valence: f32,
    pub arousal: f32,
    pub tension_curve: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormParams {
    pub section_count: u8,
    pub section_lengths: Vec<u16>,
    pub intro_bars: u16,
    pub outro_bars: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchParams {
    pub beam_width: u16,
    pub temperature: f32,
    pub max_iterations: u32,
    pub seed: Option<u64>,
}
```

Full parameter schemas deferred to Wave 4 engine docs — snapshot structure is stable.

### 8.5 EmotionProfile

Output of Emotion Resolver stage:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmotionProfile {
    pub valence: f32,
    pub arousal: f32,
    pub weight_deltas: HashMap<String, f32>,
    pub tempo_delta_bpm: f32,
    pub harmonic_color_bias: f32,
}
```

### 8.6 ProvenanceRoot

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceRoot {
    pub session_id: String,
    pub generator_version: String,
    pub seed: Option<u64>,
    pub pipeline_config_hash: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}
```

### 8.7 ScoreLayout

Visual / export layout hints:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreLayout {
    pub staff_spacing: f32,
    pub measure_numbering: MeasureNumberingStyle,
    pub part_list_order: Vec<VoiceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasureNumberingStyle {
    EveryMeasure, EverySystem, None,
}
```

### 8.8 PatchHistory

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchHistory {
    pub patches: Vec<PatchRecord>,
    pub cursor: usize,                // for undo/redo
    pub max_depth: usize,             // default 100
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchRecord {
    pub patch: Patch,
    pub inverse: Patch,
    pub timestamp: String,
    pub agent: ProvenanceAgent,
}
```

### 8.9 ExportCache

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportCache {
    pub ir: Option<MusicIr>,
    pub ir_source_hash: String,       // hash of composition + params
    pub exported_files: HashMap<String, ExportedFileMeta>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportedFileMeta {
    pub path: String,
    pub format: ExportFormat,
    pub exported_at: String,
    pub hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    MusicXml, Midi, Abc, Pdf, Json,
}
```

---

## 9. Algorithms

### 9.1 Project Creation

```text
function create_project(name, params) → Project:
    comp = empty Composition with schema_version
    comp.metadata = default CompositionMetadata from params
    comp.global = default GlobalAttributes from params.mode, params.rhythm
    comp.voice_registry = allocate_voices(params)
    return Project {
        manifest: new manifest,
        composition: comp,
        parameters: snapshot(params),
        history: empty,
        ...
    }
```

### 9.2 Save / Load

```text
function save_project(project, path):
    bundle = ZipWriter(path)
    bundle.write("manifest.json", project.manifest)
    bundle.write("composition.cbor", cbor_encode(project.composition))
    bundle.write("parameters.json", project.parameters)
    for record in project.history.patches:
        bundle.write("history/{id}.json", record)
    if project.export_cache:
        bundle.write("cache/ir.cbor", project.export_cache.ir)

function load_project(path) → Project:
    manifest = read manifest
    if manifest.format_version unsupported:
        migrate or error
    composition = cbor_decode(read composition.cbor)
    validate_invariants(composition)
    return assemble Project
```

### 9.3 Migration

```text
function migrate_composition(comp, from_version, to_version) → Composition:
    for step in migration_chain(from_version, to_version):
        comp = step.apply(comp)
    comp.schema_version = to_version
    return comp
```

Migration functions registered per `(major, minor)` bump.

### 9.4 IR Cache Invalidation

```text
function ir_cache_valid(project) → bool:
    if project.export_cache is None: return false
    current_hash = hash(project.composition, project.parameters)
    return current_hash == project.export_cache.ir_source_hash
```

Invalidate on any AST patch or parameter change.

### 9.5 Import from MusicXML (Future)

```text
function import_musicxml(xml) → Project:
    comp = parse_to_composition(xml)
    for event in all_events(comp):
        event.provenance = Imported { format: "musicxml" }
    return wrap in Project with source=Imported
```

---

## 10. Interfaces

### 10.1 Tauri Commands

| Command | Description |
|---------|-------------|
| `create_project` | New project from parameters |
| `open_project` | Load `.aurora` bundle |
| `save_project` | Write bundle |
| `get_project_metadata` | Manifest + composition metadata |
| `update_parameters` | Params + optional regen trigger |
| `get_global_attributes` | GlobalAttributes JSON |
| `set_tempo_map` | Patch tempo map |

### 10.2 Rust Project Store

```rust
pub struct ProjectStore {
    project: Project,
    ast_store: AstStore,
    dirty: bool,
}

impl ProjectStore {
    pub fn composition(&self) -> &Composition;
    pub fn parameters(&self) -> &ParameterSnapshot;
    pub fn apply_patch(&mut self, patch: Patch) -> Result<(), ProjectError>;
    pub fn invalidate_cache(&mut self);
    pub fn get_or_project_ir(&mut self) -> Result<&MusicIr, IrError>;
}
```

---

## 11. Parameter Mappings

Global attributes set during Structure / Style stages:

| Parameter | GlobalAttributes Field |
|-----------|----------------------|
| `mode.key`, `mode.mode` | `default_key`, `key_map` |
| `rhythm.time_signature` | `default_meter`, `meter_map` |
| `emotion.arousal` | `tempo_map.default_bpm` |
| `form.*` | structure drives maps indirectly |
| `dynamics.dynamic_range` | `dynamics_baseline` |
| Style preset | `display`, `layout` |

Full snapshot stored in `Composition.metadata.parameters_used` on generation complete.

---

## 12. Explainability Model

### 12.1 Session-Level

`ProvenanceRoot` links all events to one generation session:

- Reproducible with same `seed` + `parameters` + `pipeline_config_hash`
- UI "Regenerate" button uses stored snapshot

### 12.2 Project-Level Audit

```rust
pub struct ProjectAuditLog {
    pub entries: Vec<AuditEntry>,
}

pub struct AuditEntry {
    pub timestamp: String,
    pub action: AuditAction,
    pub patch_id: Option<PatchId>,
    pub agent: ProvenanceAgent,
}
```

Optional append-only log in bundle for education/research mode.

---

## 13. Future Expansion

| Feature | Score Container Impact |
|---------|-------------------------|
| Template library | `template_id` in manifest |
| Cloud sync | external ref in manifest |
| Multi-composition opus | Project → Vec<Composition> |
| Asset samples | `assets/` directory standard |
| Git plain-text mode | JSON composition + line-oriented patches |

---

## 14. Open Questions

| ID | Question |
|----|----------|
| OQ-SCR-1 | CBOR vs JSON for composition in bundle? |
| OQ-SCR-2 | Maximum patch history depth default? |
| OQ-SCR-3 | Include IR cache in default save or opt-in? |

---

## 15. References

- [ast.md](ast.md)
- [timeline.md](timeline.md)
- [voices.md](voices.md)
- [architecture.md](../01-architecture/architecture.md)
- MusicXML `<work>`, `<identification>`, `<defaults>`
- ACAS §6 Parameter System
- [terminology.md](../00-overview/terminology.md)

---

*End of Score Container Specification*
