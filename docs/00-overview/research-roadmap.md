# Research Roadmap

**Document:** Aurora Composer — Research Roadmap  
**Version:** 0.1  
**Status:** Draft

---

## 1. Overview

This roadmap sequences research work from foundation to Design Freeze. Work within each wave runs **in parallel** where dependencies allow.

---

## Wave 0: Foundation ✅ In Progress

**Goal:** Establish documentation system and master specification.

| Document | Agent | Status |
|----------|-------|--------|
| ACAS v0.1 | Architecture | Draft |
| vision.md | Architecture | Draft |
| philosophy.md | Architecture | Draft |
| terminology.md | Architecture | Draft |
| goals.md | Architecture | Draft |
| research-methodology.md | Architecture | Draft |
| architecture.md | Architecture | Draft |
| module-overview.md | Architecture | Draft |
| pipeline.md | Architecture | Draft |
| roadmap.md | Architecture | Draft |

**Exit criteria:** All Wave 0 documents in Draft status; PROGRESS.md initialized.

---

## Wave 1: Music Model (P1)

**Goal:** Define canonical Music AST and IR — the foundation all other modules depend on.

| Document | Agent | Est. Pages | Depends On |
|----------|-------|------------|------------|
| ast.md | Music AST | 40 | Wave 0 |
| ir.md | Music AST | 20 | ast.md |
| timeline.md | Music AST | 15 | ast.md |
| events.md | Music AST | 15 | ast.md |
| voices.md | Music AST | 12 | ast.md |
| score.md | Music AST | 15 | ast.md |

**Exit criteria:** AST node catalog frozen; IR projection rules defined; all pipeline stages know their AST read/write sets.

**Key decisions:** ADR on AST vs graph representation; ADR on immutable vs mutable AST during search.

---

## Wave 2: Theory System (P2, Parallel)

**Goal:** Formalize music theory knowledge encoded in the rule engine.

| Document | Agent | Est. Pages |
|----------|-------|------------|
| harmony.md | Harmony | 50 |
| counterpoint.md | Counterpoint | 40 |
| voice-leading.md | Voice Leading | 30 |
| rhythm.md | Rhythm | 30 |
| form.md | Form | 35 |
| jazz.md | Jazz | 35 |
| orchestration.md | Orchestration | 25 |

**Exit criteria:** Rule categories cataloged; hard vs soft constraint classification per style mode.

**Parallelization:** All seven agents run simultaneously.

---

## Wave 3: Rule Engine (P4)

**Goal:** Define how theory becomes executable rules.

| Document | Agent | Est. Pages | Depends On |
|----------|-------|------------|------------|
| rule-dsl.md | Constraint Solver | 35 | Wave 2 |
| constraint.md | Constraint Solver | 30 | Wave 2 |
| scoring.md | Search Algorithm | 25 | Wave 2 |

**Exit criteria:** Rule DSL grammar defined; scoring function formalized; search algorithm selection justified.

---

## Wave 4: Algorithm Engines (P3)

**Goal:** Specify each pipeline stage's algorithms.

| Document | Agent | Est. Pages | Depends On |
|----------|-------|------------|------------|
| structure-engine.md | Form | 30 | Wave 1, 2 |
| harmony-engine.md | Harmony | 40 | Wave 1, 2, 3 |
| melody-engine.md | Melody | 40 | Wave 1, 2, 3 |
| phrase-engine.md | Theme | 25 | Wave 1, 2 |
| motif-engine.md | Theme | 25 | Wave 1, 2 |
| drum-engine.md | Drum | 30 | Wave 1, 2 |
| emotion-engine.md | Emotion | 20 | Wave 2 |
| repair-engine.md | Repair | 20 | Wave 3 |

**Exit criteria:** Each engine has I/O spec, algorithm pseudocode, parameter mappings, explainability model.

---

## Wave 5: Export & Playback (P5, Parallel)

**Goal:** Define AST → external format projections.

| Document | Agent | Est. Pages |
|----------|-------|------------|
| musicxml.md | MusicXML | 30 |
| midi.md | MIDI | 20 |
| abc.md | ABC | 20 |
| pdf.md | Notation | 15 |

**Exit criteria:** Round-trip fidelity requirements defined; MusicXML as primary interchange confirmed.

---

## Wave 6: Plugin & UI (P6)

**Goal:** Define extensibility and application shell.

| Document | Agent | Est. Pages | Depends On |
|----------|-------|------------|------------|
| api.md | Plugin | 25 | Wave 1 |
| sdk.md | Plugin | 20 | api.md |
| tauri.md | Tauri | 20 | Wave 0 |
| vue.md | Vue | 20 | Wave 0 |
| timeline-ui.md | Vue | 15 | ast.md |
| piano-roll.md | Vue | 15 | ast.md |
| inspector.md | Vue | 12 | ast.md, provenance |

**Exit criteria:** Plugin lifecycle defined; Tauri command API sketched; UI component tree mapped to AST views.

---

## Wave 7: Engineering & Review (P7)

**Goal:** Testing, performance, architecture review, Design Freeze.

| Document | Agent | Est. Pages |
|----------|-------|------------|
| testing.md | Testing | 20 |
| performance.md | Performance | 15 |
| coding-style.md | Documentation | 10 |
| architecture-review-v0.1.md | Architecture | 30 |
| design-freeze-v0.1.md | Architecture | 20 |

**Exit criteria:** No unresolved contradictions; Design Freeze Report published; Phase 2 authorized.

---

## Timeline Estimate

| Wave | Duration (est.) | Cumulative |
|------|-----------------|------------|
| Wave 0 | 1 week | 1 week |
| Wave 1 | 2 weeks | 3 weeks |
| Wave 2 | 3 weeks (parallel) | 6 weeks |
| Wave 3 | 2 weeks | 8 weeks |
| Wave 4 | 4 weeks (parallel) | 12 weeks |
| Wave 5 | 2 weeks (parallel) | 14 weeks |
| Wave 6 | 2 weeks | 16 weeks |
| Wave 7 | 2 weeks | 18 weeks |

*Estimates assume continuous multi-agent research. Actual duration depends on review cycles.*

---

## Risk Register

| Risk | Mitigation |
|------|------------|
| AST design changes cascade | Freeze AST in Wave 1 before Wave 4 |
| Theory specs too vague for rules | Wave 3 blocked until Wave 2 review passes |
| Scope creep into implementation | PROGRESS.md gate: no code until M8 |
| Agent output inconsistency | Architecture Agent merge after each wave |
| Duplicate concepts across specs | Architecture Review in Wave 7 |

---

## References

- [TASKS.md](../../TASKS.md)
- [PROGRESS.md](../../PROGRESS.md)
- [Research Methodology](research-methodology.md)
