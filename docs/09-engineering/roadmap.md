# Development Roadmap

**Document:** Aurora Composer — Development Roadmap  
**Version:** 0.1  
**Status:** Draft

---

## Phase Overview

```text
Phase 1: Architecture Design     ← CURRENT (documentation only)
    ↓ Design Freeze
Phase 2: Prototype                 (core engine + minimal UI)
    ↓ Prototype Validation
Phase 3: Production Development    (styles, plugins, AI, polish)
```

---

## Phase 1: Architecture Design

**Duration:** ~18 weeks (estimated)  
**Deliverable:** 500–1000 pages specification, Design Freeze Report  
**Constraint:** **No production code**

### Waves

| Wave | Focus | Exit Criteria |
|------|-------|---------------|
| 0 | Overview & ACAS | Master spec drafted |
| 1 | Music Model (AST/IR) | Node schemas frozen |
| 2 | Theory System | Rule categories defined |
| 3 | Rule Engine | DSL grammar defined |
| 4 | Algorithm Engines | All pipeline stages specified |
| 5 | Export | Format specs complete |
| 6 | Plugin & UI | API and shell specified |
| 7 | Review & Freeze | No contradictions; freeze report |

See [research-roadmap.md](../00-overview/research-roadmap.md) for details.

---

## Phase 2: Prototype

**Prerequisite:** Design Freeze Report approved  
**Goal:** End-to-end proof of architecture

### Implementation Order

```text
1. Music AST (Rust)           — core data structures
2. Rule Engine (Rust)         — basic rule evaluation
3. Harmony Engine             — chord progression generation
4. Melody Engine              — beam search melody
5. Export (MIDI + MusicXML)   — AST → files
6. Minimal UI (Tauri + Vue)   — params + generate + preview
7. Player                     — browser MIDI playback
```

### Prototype Scope

| In Scope | Out of Scope |
|----------|--------------|
| Single style (pop/classical) | Multiple style plugins |
| 2-voice (melody + chords) | Full counterpoint |
| 16-bar generation | Multi-movement forms |
| MIDI + MusicXML export | PDF, ABC |
| Basic parameter panel | Piano roll editing |
| Explainability inspector | Full provenance UI |

### Prototype Success Criteria

- Generate 16-bar 2-voice piece from parameters
- Every note has provenance metadata
- Export valid MusicXML and MIDI
- Parameter change produces measurable output change
- Generation completes in < 30s on 4-core CPU

---

## Phase 3: Production Development

**Prerequisite:** Prototype validated  
**Goal:** Full-featured composition platform

### Expansion Areas

| Area | Features |
|------|----------|
| Styles | Jazz, Baroque, Electronic, Folk presets |
| Voices | Counterpoint, bass, drums, full orchestration |
| Form | Multi-section, multi-theme, transitions |
| Export | ABC, PDF, audio render |
| UI | Timeline, piano roll, full inspector |
| Plugins | Plugin SDK, marketplace infrastructure |
| AI | Optional AI plugins (emotion, motif transform) |
| Performance | Parallel search, caching, incremental gen |
| Testing | Automated quality metrics, regression suite |
| Documentation | User guide, plugin developer guide |

---

## Specification-First Mandate

Throughout all phases:

```
需求 (Requirements)
    ↓
设计 (Design Document)
    ↓
Review (Architecture Agent)
    ↓
冻结 (Design Freeze for module)
    ↓
实现 (Implementation)
```

**No module may be implemented before its specification is frozen.**

Violations must be reported and rectified before merge.

---

## Milestone Timeline (Estimated)

| Milestone | Target | Phase |
|-----------|--------|-------|
| M1: ACAS v0.1 draft | Week 1 | 1 |
| M2: Music AST frozen | Week 3 | 1 |
| M3: Theory specs complete | Week 6 | 1 |
| M4: Algorithm specs complete | Week 12 | 1 |
| M5: Full spec corpus | Week 16 | 1 |
| M6: Design Freeze | Week 18 | 1 |
| M7: Prototype AST + Rules | Week 22 | 2 |
| M8: Prototype E2E | Week 28 | 2 |
| M9: Production alpha | Week 40 | 3 |
| M10: Production release | Week 52+ | 3 |

---

## References

- [Research Roadmap](../00-overview/research-roadmap.md)
- [PROGRESS.md](../../PROGRESS.md)
- [TASKS.md](../../TASKS.md)
- [ACAS v0.1](../00-overview/acas-v0.1.md)
