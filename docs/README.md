# Aurora Composer Documentation Index

**ACAS Version:** 0.1 (Design Freeze Draft)  
**Status:** Wave 0–7 specifications drafted — Architecture Review pending

---

## Start Here

1. [ACAS v0.1 — Master Specification](00-overview/acas-v0.1.md)
2. [Project Vision](00-overview/vision.md)
3. [Design Philosophy](00-overview/philosophy.md)
4. [Terminology & Glossary](00-overview/terminology.md)
5. [Progress Tracker](../PROGRESS.md)
6. [Task List](../TASKS.md)

---

## Document Tree

### 00-overview/ — Worldview & Governance

| Document | Description |
|----------|-------------|
| [acas-v0.1.md](00-overview/acas-v0.1.md) | Master architecture specification |
| [vision.md](00-overview/vision.md) | Project vision and positioning |
| [philosophy.md](00-overview/philosophy.md) | Eight design principles |
| [terminology.md](00-overview/terminology.md) | Normative glossary |
| [goals.md](00-overview/goals.md) | Primary and non-goals |
| [research-methodology.md](00-overview/research-methodology.md) | How research agents work |
| [research-roadmap.md](00-overview/research-roadmap.md) | Wave-based research plan |

### 01-architecture/ — System Structure

| Document | Description |
|----------|-------------|
| [architecture.md](01-architecture/architecture.md) | Layer model, component diagram |
| [module-overview.md](01-architecture/module-overview.md) | Full module catalog |
| [pipeline.md](01-architecture/pipeline.md) | 14-stage generation pipeline |
| [backend.md](01-architecture/backend.md) | Rust crate structure, Tauri integration |
| [frontend.md](01-architecture/frontend.md) | Vue 3 architecture overview |

### 02-music-model/ — Music AST & IR

| Document | Description |
|----------|-------------|
| [ast.md](02-music-model/ast.md) | Full AST node schemas, provenance, patching |
| [ir.md](02-music-model/ir.md) | AST → IR projection |
| [timeline.md](02-music-model/timeline.md) | Tempo map, beat grid |
| [events.md](02-music-model/events.md) | Event type catalog |
| [voices.md](02-music-model/voices.md) | Voice allocation, MIDI mapping |
| [score.md](02-music-model/score.md) | Project container (`.aurora` bundle) |

### 03-theory/ — Music Theory Rules (395 rules)

| Document | Rules | Description |
|----------|-------|-------------|
| [harmony.md](03-theory/harmony.md) | 78 | Functional harmony, progressions, cadences |
| [counterpoint.md](03-theory/counterpoint.md) | 62 | Species counterpoint, parallel intervals |
| [voice-leading.md](03-theory/voice-leading.md) | 47 | Common tones, stepwise motion |
| [rhythm.md](03-theory/rhythm.md) | 54 | Meter, syncopation, Groove MIDI taxonomy |
| [form.md](03-theory/form.md) | 58 | Sectional forms, theme development |
| [jazz.md](03-theory/jazz.md) | 56 | ii–V–I, extensions, substitutions |
| [orchestration.md](03-theory/orchestration.md) | 40 | Ranges, doubling, texture |

### 04-algorithms/ — Generation Engines

| Document | Pipeline Stage |
|----------|---------------|
| [structure-engine.md](04-algorithms/structure-engine.md) | Stage 3: Structure Planning |
| [emotion-engine.md](04-algorithms/emotion-engine.md) | Stage 2: Emotion Resolver |
| [motif-engine.md](04-algorithms/motif-engine.md) | Stage 4: Theme Planning |
| [phrase-engine.md](04-algorithms/phrase-engine.md) | Cross-cutting: Phrase/Cadence |
| [harmony-engine.md](04-algorithms/harmony-engine.md) | Stage 5: Harmony Skeleton |
| [melody-engine.md](04-algorithms/melody-engine.md) | Stage 7: Melody |
| [drum-engine.md](04-algorithms/drum-engine.md) | Stage 10: Drums |
| [repair-engine.md](04-algorithms/repair-engine.md) | Stage 12: Repair |

### 05-rule-engine/ — Rules, Constraints, Search

| Document | Description |
|----------|-------------|
| [rule-dsl.md](05-rule-engine/rule-dsl.md) | Rule DSL grammar and compilation |
| [constraint.md](05-rule-engine/constraint.md) | Hard/soft constraint system |
| [scoring.md](05-rule-engine/scoring.md) | Scoring function, beam search, A*, DP |

### 06-export/ — Format Projection

| Document | Description |
|----------|-------------|
| [musicxml.md](06-export/musicxml.md) | MusicXML 4.0 (primary interchange) |
| [midi.md](06-export/midi.md) | Standard MIDI File export |
| [abc.md](06-export/abc.md) | ABC notation (scoped) |
| [pdf.md](06-export/pdf.md) | Verovio WASM / MuseScore CLI rendering |

### 07-plugin/ — Extensibility

| Document | Description |
|----------|-------------|
| [api.md](07-plugin/api.md) | Plugin traits, lifecycle, sandbox |
| [sdk.md](07-plugin/sdk.md) | Plugin developer guide |

### 08-ui/ — Application Shell

| Document | Description |
|----------|-------------|
| [tauri.md](08-ui/tauri.md) | 42 Tauri commands, IPC, events |
| [vue.md](08-ui/vue.md) | Vue 3 app, Pinia stores |
| [timeline-ui.md](08-ui/timeline-ui.md) | Section/measure timeline |
| [piano-roll.md](08-ui/piano-roll.md) | Note grid with provenance tooltip |
| [inspector.md](08-ui/inspector.md) | Full provenance chain inspector |

### 09-engineering/ — Quality & Process

| Document | Description |
|----------|-------------|
| [roadmap.md](09-engineering/roadmap.md) | Phase 1–3 development roadmap |
| [testing.md](09-engineering/testing.md) | Test strategy, 395 rule validation |
| [performance.md](09-engineering/performance.md) | Performance targets, optimization |
| [coding-style.md](09-engineering/coding-style.md) | Rust + TypeScript conventions |

---

## Supporting Materials

| Path | Purpose |
|------|---------|
| [research/](../research/) | Raw research notes from agents |
| [decisions/](../decisions/) | Architecture Decision Records |
| [reviews/](../reviews/) | Architecture review records |
| [deep-research-report.md](../deep-research-report.md) | Initial algorithm survey |

---

## Architecture Decisions

| ADR | Decision |
|-----|----------|
| [ADR-001](../decisions/ADR-001-specification-first.md) | Specification-first development |
| [ADR-002](../decisions/ADR-002-vue3-frontend.md) | Vue 3 frontend |
| [ADR-003](../decisions/ADR-003-search-algorithm-primary.md) | Beam search as primary algorithm |
| [ADR-004](../decisions/ADR-004-musicxml-primary-export.md) | MusicXML as primary export |
| [ADR-005](../decisions/ADR-005-plugin-sandbox-model.md) | Tiered plugin sandbox |

---

## For AI Agents

Before implementing any module:

1. Read [acas-v0.1.md](00-overview/acas-v0.1.md)
2. Read [philosophy.md](00-overview/philosophy.md)
3. Read [terminology.md](00-overview/terminology.md)
4. Read the module's specification in the relevant `docs/` subdirectory
5. Check [decisions/](../decisions/) for binding ADRs
6. **Do not write production code** until Design Freeze
