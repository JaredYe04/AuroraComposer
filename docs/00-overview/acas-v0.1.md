# Aurora Composer Architecture Specification (ACAS)

**Version:** 0.1 (Design Freeze)  
**Status:** Frozen  
**Date:** 2026-07-04  
**Authority:** This document and the `docs/` tree constitute the normative specification for Aurora Composer.

---

## Document Control

| Field | Value |
|-------|-------|
| Document ID | ACAS-0.1 |
| Phase | Architecture Research (Phase 1) |
| Implementation | **Forbidden** until Design Freeze |
| Supersedes | None (initial release) |
| Related | [Deep Research Report](../../deep-research-report.md) |

---

## Table of Contents

1. [Project Vision](#1-project-vision)
2. [Design Philosophy](#2-design-philosophy)
3. [System Architecture](#3-system-architecture)
4. [Music AST Overview](#4-music-ast-overview)
5. [Generation Pipeline](#5-generation-pipeline)
6. [Parameter System](#6-parameter-system)
7. [Rule Engine Overview](#7-rule-engine-overview)
8. [Plugin System Overview](#8-plugin-system-overview)
9. [Export Architecture](#9-export-architecture)
10. [Technology Stack](#10-technology-stack)
11. [Research Agent Workflow](#11-research-agent-workflow)
12. [Development Roadmap](#12-development-roadmap)
13. [Document Index](#13-document-index)

---

## 1. Project Vision

Aurora Composer is a **parameterized, explainable, extensible Music Composition Engine**.

It is **not** automatic composition software, an MVP, a demo, or an AI toy.

### Core Equation

```
Music Theory + Rule System + Constraint + Search = Composition Engine
                                                      ↓
                                    AI (optional plugin)
```

Full vision: [vision.md](vision.md)

---

## 2. Design Philosophy

Eight immutable principles govern all architecture:

| # | Principle | Summary |
|---|-----------|---------|
| 1 | Everything Is Music AST | All modules operate on AST only |
| 2 | Everything Is Explainable | Provenance chain for every event |
| 3 | Generation Is Search | Constraint-driven search, not random/AI |
| 4 | Everything Is Parameterized | No magic constants |
| 5 | Specification Before Code | Research → Spec → Review → Freeze → Code |
| 6 | Music Theory First | Rules before statistics |
| 7 | Modular and Pluggable | Knowledge in plugins |
| 8 | Documents Are Source of Truth | Specs authoritative over code |

Full principles: [philosophy.md](philosophy.md)

---

## 3. System Architecture

### 3.1 Layer Diagram

```text
┌─────────────────────────────────────────────────────────┐
│                    Vue 3 Frontend                        │
│  Parameter Panel · Timeline · Piano Roll · Inspector    │
└─────────────────────────┬───────────────────────────────┘
                          │ Tauri IPC (JSON)
┌─────────────────────────▼───────────────────────────────┐
│                   Tauri Shell (Rust)                     │
│  Command Router · Async Jobs · File I/O · Plugin Loader  │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│              Composition Engine (Rust Core)              │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Pipeline   │  │ Rule Engine  │  │Search Engine │  │
│  │  Orchestrator│◄─┤ Constraints  │◄─┤ Beam/A*/DP   │  │
│  └──────┬───────┘  │ Scoring      │  └──────────────┘  │
│         │          └──────────────┘                      │
│         ▼                                                │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Algorithm Engines                    │   │
│  │ Structure · Harmony · Melody · Rhythm · Drums   │   │
│  │ Phrase · Motif · Counterpoint · Emotion · Repair │   │
│  └──────────────────────┬───────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────▼───────────────────────────┐   │
│  │                  Music AST                        │   │
│  │     (Single Source of Musical Truth)              │   │
│  └──────────────────────┬───────────────────────────┘   │
│                         │                                │
│  ┌──────────────────────▼───────────────────────────┐   │
│  │              Music IR → Exporters                 │   │
│  │         MusicXML · ABC · MIDI · PDF               │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│                    Plugin Host                           │
│  Style · Harmony · Rhythm · Theme · AI · Export       │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Module Inventory

| Module | Responsibility | Spec |
|--------|---------------|------|
| Composition Engine | Pipeline orchestration | [architecture.md](../01-architecture/architecture.md) |
| Music AST | Canonical representation | [ast.md](../02-music-model/ast.md) |
| Music IR | Export projection | [ir.md](../02-music-model/ir.md) |
| Theory Engine | Scales, intervals, degrees | [harmony.md](../03-theory/harmony.md) |
| Structure Engine | Form, sections, phrases | [structure-engine.md](../04-algorithms/structure-engine.md) |
| Harmony Engine | Chord progressions, voicings | [harmony-engine.md](../04-algorithms/harmony-engine.md) |
| Melody Engine | Melodic line generation | [melody-engine.md](../04-algorithms/melody-engine.md) |
| Rhythm Engine | Metric patterns | [rhythm.md](../03-theory/rhythm.md) |
| Phrase/Motif Engine | Theme development | [phrase-engine.md](../04-algorithms/phrase-engine.md) |
| Counterpoint Engine | Multi-voice rules | [counterpoint.md](../03-theory/counterpoint.md) |
| Drum Engine | Percussion patterns | [drum-engine.md](../04-algorithms/drum-engine.md) |
| Emotion Engine | Parameter → weight mapping | [emotion-engine.md](../04-algorithms/emotion-engine.md) |
| Rule Engine | Rules, constraints, scoring | [rule-dsl.md](../05-rule-engine/rule-dsl.md) |
| Search Engine | Candidate exploration | [scoring.md](../05-rule-engine/scoring.md) |
| Repair Engine | Post-generation fixes | [repair-engine.md](../04-algorithms/repair-engine.md) |
| Export Engine | Format projection | [musicxml.md](../06-export/musicxml.md) |
| Plugin Host | Extension loading | [api.md](../07-plugin/api.md) |
| UI Shell | Tauri + Vue frontend | [vue.md](../08-ui/vue.md) |

Full module details: [module-overview.md](../01-architecture/module-overview.md)

---

## 4. Music AST Overview

The Music AST is the **single source of musical truth**. All generation modules read and write AST nodes.

### 4.1 Node Hierarchy (Summary)

```text
Composition
├── Metadata (title, parameters, provenance_root)
├── GlobalAttributes (tempo_map, key, time_signature)
└── Movement[]
    └── Section[]
        ├── SectionMarker (role: verse, chorus, bridge, ...)
        └── Phrase[]
            └── Measure[]
                └── Voice[]
                    └── Event[]
                        ├── Note
                        ├── Chord
                        ├── Rest
                        └── Marker
```

### 4.2 Design Constraints

- Every `Event` node carries `Provenance` metadata
- AST is versioned; patches are atomic and reversible
- Search operates on AST snapshots (copy-on-write or persistent data structures)
- Detailed node schemas: [ast.md](../02-music-model/ast.md) *(pending Wave 1)*

### 4.3 AST vs IR

| Aspect | AST | IR |
|--------|-----|-----|
| Purpose | Generation, editing, explainability | Export, playback |
| Structure | Hierarchical (form-aware) | Flattened (time-ordered) |
| Mutability | Mutable during generation | Immutable snapshot |
| Provenance | Full chain | Optional summary |

---

## 5. Generation Pipeline

The pipeline transforms user parameters into a validated Composition AST.

### 5.1 Pipeline Stages

```text
User Parameters
       ↓
┌──────────────────┐
│  Style Resolver  │  Map genre preset → parameter bundle + plugins
└────────┬─────────┘
         ↓
┌──────────────────┐
│ Emotion Resolver │  Map emotion labels → weight adjustments
└────────┬─────────┘
         ↓
┌──────────────────┐
│Structure Planning│  Form, sections, key areas, tempo map
└────────┬─────────┘
         ↓
┌──────────────────┐
│ Theme Planning   │  Theme allocation, motif strategy
└────────┬─────────┘
         ↓
┌──────────────────┐
│Harmony Skeleton  │  Chord progression + harmonic rhythm
└────────┬─────────┘
         ↓
┌──────────────────┐
│ Rhythm Skeleton  │  Metric patterns, subdivisions
└────────┬─────────┘
         ↓
┌──────────────────┐
│     Melody       │  Primary melodic lines (search)
└────────┬─────────┘
         ↓
┌──────────────────┐
│  Counterpoint    │  Additional voices
└────────┬─────────┘
         ↓
┌──────────────────┐
│      Bass        │  Bass line generation
└────────┬─────────┘
         ↓
┌──────────────────┐
│     Drums        │  Percussion patterns
└────────┬─────────┘
         ↓
┌──────────────────┐
│   Decoration     │  Ornaments, passing tones
└────────┬─────────┘
         ↓
┌──────────────────┐
│     Repair       │  Fix soft violations
└────────┬─────────┘
         ↓
┌──────────────────┐
│   Validation     │  Hard constraint check
└────────┬─────────┘
         ↓
┌──────────────────┐
│     Export       │  AST → IR → formats
└──────────────────┘
```

### 5.2 Stage Properties

Each stage specifies (in its module doc):

- **Input:** AST read set + parameters
- **Output:** AST write set
- **Search involvement:** yes/no, beam width parameter
- **Explainability:** provenance fields added
- **Failure mode:** retry, repair, or abort

Full pipeline spec: [pipeline.md](../01-architecture/pipeline.md)

---

## 6. Parameter System

All generative behavior is controlled through named parameters. Parameters map to internal weights — never hardcoded.

### 6.1 Parameter Categories

| Category | Parameters | Maps To |
|----------|-----------|---------|
| **Emotion** | valence, arousal, tension_curve | Harmonic color, tempo, dynamics |
| **Style** | genre, era, orchestration_preset | Plugin selection, rule set |
| **Mode** | key, mode, modulation_policy | Scale, chord vocabulary |
| **Scale** | scale_type, borrowed_chord_tolerance | Pitch class set |
| **Theme** | theme_count, motif_length, repetition_ratio | Form engine |
| **Harmony** | complexity, dissonance, cadence_strength, harmonic_rhythm | Harmony engine + rules |
| **Voice** | voice_count, density, register_min/max | Voice allocation |
| **Texture** | homophony_polyphony_balance | Engine activation |
| **Rhythm** | density, syncopation, subdivision, swing | Rhythm engine |
| **Dynamics** | dynamic_range, accent_strength | Event velocities |
| **Cadence** | cadence_type_preference, half_cadence_freq | Harmony rules |
| **Register** | melody_register, bass_register | Range constraints |
| **Counterpoint** | strictness, parallel_penalty | Voice-leading rules |
| **Drums** | density, fill_frequency, pattern_complexity | Drum engine |
| **Search** | beam_width, temperature, max_iterations | Search engine |
| **Form** | section_count, section_lengths, intro_outro | Structure engine |

Detailed parameter schemas and default values will be specified per engine in Wave 4.

---

## 7. Rule Engine Overview

### 7.1 Concepts

| Concept | Definition |
|---------|------------|
| **Rule** | Named condition with ID, category, description, weight binding |
| **Constraint** | Boolean predicate; hard (prune) or soft (penalty) |
| **Score** | Numeric evaluation of candidate state |
| **Rule DSL** | Language for defining rules declaratively |

### 7.2 Evaluation Flow

```text
Candidate AST State
       ↓
Apply Hard Constraints → prune if violated
       ↓
Evaluate Soft Rules → accumulate score
       ↓
Return (score, violations[], provenance[])
```

### 7.3 Scoring Function (Abstract)

```
eval_score = Σ (weight_i × indicator_i) − Σ (penalty_j × violation_j)
```

Weights derive from user parameters. Full formalization: [scoring.md](../05-rule-engine/scoring.md)

### 7.4 Search Integration

- **Beam Search:** primary algorithm for melody/harmony generation
- **A*:** optional for global optimization with admissible heuristic
- **Dynamic Programming:** structure/harmony skeleton planning

Selection rationale documented in [scoring.md](../05-rule-engine/scoring.md).

---

## 8. Plugin System Overview

### 8.1 Plugin Types

| Type | Interface | Example |
|------|-----------|---------|
| Style | `StylePlugin` | "Baroque", "Jazz Standard" |
| Harmony | `HarmonyPlugin` | Figured bass, jazz voicing |
| Rhythm | `RhythmPlugin` | Bossa pattern library |
| Theme | `ThemePlugin` | Motif inversion algorithms |
| AI | `AIPlugin` | Candidate proposal, weight adjustment |
| Export | `ExportPlugin` | Custom format |

### 8.2 Plugin Contract

All plugins:

- Receive AST snapshots (read) and return AST patches (write)
- Must not bypass provenance requirements
- Register parameters they consume
- Are hot-loadable via Plugin Host

Full API: [api.md](../07-plugin/api.md)

---

## 9. Export Architecture

```text
Music AST
    ↓ (projection)
Music IR (time-ordered events per channel)
    ↓
┌─────────┬─────────┬───────────┬─────────┐
│MusicXML │   ABC   │   MIDI    │   PDF   │
│(primary)│ (folk)  │(playback) │(render) │
└─────────┴─────────┴───────────┴─────────┘
                              ↓
                      Audio Preview (WebAudio)
```

- **MusicXML** is the primary interchange format (ADR pending)
- **PDF** rendered via Verovio WASM or MuseScore CLI from MusicXML
- **Playback** via browser MIDI synthesis (Tone.js or equivalent)

Per-format specs: [docs/06-export/](../06-export/)

---

## 10. Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Core Engine | Rust | Performance, safety, WASM target |
| Desktop Shell | Tauri 2.x | Lightweight, Rust-native IPC |
| Frontend | Vue 3 + TypeScript | Reactive UI, component ecosystem |
| Audio Preview | Web Audio API / Tone.js | Cross-platform, no system deps |
| Notation Render | Verovio WASM | In-browser MusicXML → SVG |
| Plugin Format | Rust dynamic libs / WASM modules | TBD in plugin spec |

Note: Deep research report referenced React; **ACAS specifies Vue 3** per project direction.

---

## 11. Research Agent Workflow

Aurora Composer development uses specialized research agents:

```text
Research Agents (parallel, one topic each)
    ↓ Markdown Specifications
Architecture Agent (merge, review, resolve)
    ↓
Design Freeze Report
    ↓
Coding Agent (Phase 2 only — BLOCKED until freeze)
```

Agent rules and output format: [research-methodology.md](research-methodology.md)

Current agent assignments: [TASKS.md](../../TASKS.md) · Progress: [PROGRESS.md](../../PROGRESS.md)

---

## 12. Development Roadmap

### Phase 1: Architecture Design ← **CURRENT**

- Output: 500–1000 pages specification
- **No production code**
- Exit: Design Freeze Report

### Phase 2: Prototype

Implement: Music AST → Rule Engine → Harmony → Melody → Export → Player

### Phase 3: Production

Expand: styles, rules, algorithms, plugins, AI

Full roadmap: [roadmap.md](../09-engineering/roadmap.md)

### Specification-First Mandate

```
需求 → 设计 → Review → 冻结 → 实现
```

Implementation without frozen specification is **not permitted**.

---

## 13. Document Index

```text
docs/
├── 00-overview/          ← You are here (ACAS root)
├── 01-architecture/        System structure, pipeline, modules
├── 02-music-model/         AST, IR, timeline, events, voices
├── 03-theory/              Harmony, counterpoint, rhythm, form, jazz
├── 04-algorithms/          Generation engine specifications
├── 05-rule-engine/         Rule DSL, constraints, scoring
├── 06-export/              MIDI, ABC, MusicXML, PDF
├── 07-plugin/              Plugin API, SDK
├── 08-ui/                  Tauri, Vue, timeline, piano roll, inspector
└── 09-engineering/         Testing, performance, coding style, roadmap
```

---

## Appendix A: Deep Research Integration

The [Deep Research Report](../../deep-research-report.md) establishes:

- **Recommended approach:** Rule engine + constraint search (primary), with optional ML plugins
- **Algorithm comparison:** Rules+search vs Markov vs DL vs hybrid (table in report §1)
- **Initial module sketch:** Mermaid architecture diagram (report §2.1)
- **Parameter mapping table:** User params → algorithm weights (report §2.2)
- **Data resources:** Lakh MIDI, MAESTRO, Bach Chorales, Groove MIDI, ABC libraries
- **Validation plan:** Objective metrics + subjective listening tests

ACAS formalizes and extends these findings into module-level specifications.

---

## Appendix B: Open Items (v0.1) — Resolved at Design Freeze

| Item | Status | ADR / Document |
|------|--------|----------------|
| AST node schema detail | ✅ Frozen | [ast.md](../02-music-model/ast.md) |
| Rule DSL grammar | ✅ Frozen | [rule-dsl.md](../05-rule-engine/rule-dsl.md) |
| Beam search as primary algorithm | ✅ Accepted | [ADR-003](../../decisions/ADR-003-search-algorithm-primary.md) |
| MusicXML as primary export | ✅ Accepted | [ADR-004](../../decisions/ADR-004-musicxml-primary-export.md) |
| Plugin sandbox model | ✅ Accepted | [ADR-005](../../decisions/ADR-005-plugin-sandbox-model.md) |
| Project serialization (CBOR) | ✅ Accepted | [ADR-006](../../decisions/ADR-006-cbor-project-serialization.md) |
| Jazz vs classical style switch | ✅ Accepted | [ADR-007](../../decisions/ADR-007-style-rule-bundle-switch.md) |
| COW AST during search | ✅ Accepted | [ADR-008](../../decisions/ADR-008-cow-ast-snapshots.md) |
| Vue 3 over React | ✅ Accepted | [ADR-002](../../decisions/ADR-002-vue3-frontend.md) |
| Plugin loading mechanism | ✅ Frozen | [api.md](../07-plugin/api.md) |
| Phrase Engine orchestration | ✅ Resolved | [pipeline.md](../01-architecture/pipeline.md) PHRASE-HOOK-* |
| Decoration engine spec | ✅ Frozen | [decoration-engine.md](../04-algorithms/decoration-engine.md) |
| Groove MIDI licensing | ⚠️ Deferred | Legal review before bundling; see drum-engine.md §14 |

**Design Freeze:** See [design-freeze-v0.1.md](../../reviews/design-freeze-v0.1.md). Phase 2 implementation authorized.

---

*End of ACAS v0.1 Draft*
