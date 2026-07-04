# Aurora Composer — Task List

**Phase 1 Active:** Architecture Research & Documentation  
**Rule:** No production code until Design Freeze.

---

## Priority Queue

**Phase 2 Active:** Prototype Implementation (Design Freeze approved 2026-07-04)

### P0 — Prototype Core (Active)

- [x] Complete Phase 1 documentation (49 specs)
- [x] Design Freeze Report
- [ ] Cargo workspace + crate scaffold
- [ ] aurora-core + aurora-ast
- [ ] aurora-rules (subset + beam search)
- [ ] aurora-engine (structure, harmony, melody)
- [ ] aurora-export (MIDI, MusicXML)
- [ ] Tauri shell + Vue UI
- [ ] E2E: generate → export → preview

### P1 — Core Model (Next)

- [ ] Music AST Agent → `docs/02-music-model/ast.md`
- [ ] Music IR Agent → `docs/02-music-model/ir.md`
- [ ] Timeline & Events Agent → `docs/02-music-model/timeline.md`, `events.md`
- [ ] Voices & Score Agent → `docs/02-music-model/voices.md`, `score.md`

### P2 — Theory System (Parallel)

- [ ] Harmony Agent → `docs/03-theory/harmony.md`
- [ ] Counterpoint Agent → `docs/03-theory/counterpoint.md`
- [ ] Voice Leading Agent → `docs/03-theory/voice-leading.md`
- [ ] Rhythm Agent → `docs/03-theory/rhythm.md`
- [ ] Form Agent → `docs/03-theory/form.md`
- [ ] Jazz Agent → `docs/03-theory/jazz.md`
- [ ] Orchestration Agent → `docs/03-theory/orchestration.md`

### P3 — Algorithms (Depends on P1 + P2)

- [ ] Structure Engine → `docs/04-algorithms/structure-engine.md`
- [ ] Harmony Engine → `docs/04-algorithms/harmony-engine.md`
- [ ] Melody Engine → `docs/04-algorithms/melody-engine.md`
- [ ] Phrase Engine → `docs/04-algorithms/phrase-engine.md`
- [ ] Motif Engine → `docs/04-algorithms/motif-engine.md`
- [ ] Drum Engine → `docs/04-algorithms/drum-engine.md`
- [ ] Emotion Engine → `docs/04-algorithms/emotion-engine.md`
- [ ] Repair Engine → `docs/04-algorithms/repair-engine.md`

### P4 — Rule Engine (Depends on P2)

- [x] Constraint Solver Agent → `docs/05-rule-engine/constraint.md`
- [x] Rule DSL Agent → `docs/05-rule-engine/rule-dsl.md`
- [x] Search Algorithm Agent → `docs/05-rule-engine/scoring.md`

### P5 — Export & Playback (Parallel)

- [ ] MIDI Agent → `docs/06-export/midi.md`
- [ ] ABC Agent → `docs/06-export/abc.md`
- [ ] MusicXML Agent → `docs/06-export/musicxml.md`
- [ ] Notation Agent → `docs/06-export/pdf.md`
- [ ] Playback Agent → research note in `research/playback.md`

### P6 — Plugin & UI (Depends on P1)

- [ ] Plugin Architecture Agent → `docs/07-plugin/api.md`, `sdk.md`
- [ ] Tauri Agent → `docs/08-ui/tauri.md`
- [ ] Vue Agent → `docs/08-ui/vue.md`, timeline, piano-roll, inspector

### P7 — Engineering & Review

- [ ] Testing Agent → `docs/09-engineering/testing.md`
- [ ] Performance Agent → `docs/09-engineering/performance.md`
- [ ] Documentation Agent → `docs/09-engineering/coding-style.md`
- [ ] Architecture Review → `reviews/architecture-review-v0.1.md`
- [ ] Design Freeze Report → `reviews/design-freeze-v0.1.md`

---

## Dependency Graph

```
00-overview ──────────────────────────────────────────┐
     │                                                  │
01-architecture ───────────────────────────────────────┤
     │                                                  │
     ├──► 02-music-model (AST/IR) ──► 04-algorithms     │
     │         │                         │              │
     │         └──► 07-plugin            │              │
     │         └──► 08-ui                │              │
     │                                   │              │
     ├──► 03-theory ──► 05-rule-engine ─┘              │
     │         │                                        │
     │         └──► 04-algorithms                       │
     │                                                  │
     ├──► 06-export (parallel)                          │
     │                                                  │
     └──► 09-engineering ──► Architecture Review ──► FREEZE
```

---

## Research Agent Assignment Rules

1. Each agent researches **one topic only** — no implementation.
2. Output format: **Markdown Specification** with required sections (see `docs/00-overview/research-methodology.md`).
3. Minimum depth: substantial specification (target 20–100 pages per module).
4. Architecture Agent merges, reviews consistency, resolves conflicts.
5. Coding Agent is **blocked** until Design Freeze.

---

## Automatic Workflow (After Each Document)

1. Review consistency with existing docs
2. Find missing dependencies → generate new tasks
3. Update PROGRESS.md
4. Continue to next document
5. Repeat until architecture complete
