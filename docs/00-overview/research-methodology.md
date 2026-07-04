# Research Methodology

**Document:** Aurora Composer — Research Methodology  
**Version:** 0.1  
**Status:** Draft

---

## 1. Purpose

This document defines **how** Aurora Composer research is conducted — by human architects and by AI research agents. All research outputs are specifications, never production code.

---

## 2. Research Agent Workflow

Aurora Composer uses a **multi-agent research model**. No single agent completes the entire architecture.

```
Research Phase
    ├── Harmony Agent          → Harmony Engine Design
    ├── Counterpoint Agent     → Counterpoint Design
    ├── Jazz Agent             → Jazz Harmony Design
    ├── Rhythm Agent           → Rhythm Theory Design
    ├── Form Agent             → Form & Structure Design
    ├── Drum Agent             → Drum Composition Design
    ├── MusicXML Agent         → MusicXML Export Design
    ├── MIDI Agent             → MIDI Export Design
    ├── Music AST Agent        → AST & IR Design
    ├── Rule Engine Agent      → Rule DSL Design
    ├── Plugin Agent           → Plugin API Design
    ├── Tauri Agent            → Backend Architecture
    ├── Vue Agent              → Frontend Architecture
    └── ... (see TASKS.md)
            ↓
    Architecture Agent
    (Review · Merge · Resolve Conflicts)
            ↓
    Design Freeze Report
            ↓
    Coding Agent (Phase 2 only)
```

### Agent Rules

1. **Single topic only** — each agent researches one domain
2. **No implementation** — output is Markdown specification only
3. **Required sections** — see Section 4 below
4. **Reference prior art** — academic sources, existing tools, open datasets
5. **Recommend, don't decide alone** — major architecture choices need ADR
6. **Flag conflicts** — if findings contradict existing docs, report explicitly

---

## 3. Research Process (Per Topic)

### Step 1: Dependency Analysis

Before writing, identify:

- Required prior documents
- Related documents that may conflict
- Unknown areas requiring further research

Update dependency graph in `TASKS.md` if new dependencies discovered.

### Step 2: Background Research

Survey:

- Academic music theory (textbooks, papers)
- Existing software (Music21, OpenMusic, Lenardo, AIVA internals where documented)
- Open datasets (Lakh MIDI, MAESTRO, Bach Chorales, Groove MIDI)
- Standards (MusicXML, SMF MIDI, ABC notation)

Store raw notes in `research/<topic>-notes.md`.

### Step 3: Engineering Analysis

Evaluate approaches:

| Criterion | Questions |
|-----------|-----------|
| Correctness | Does it satisfy music theory for target styles? |
| Controllability | Can parameters map cleanly? |
| Explainability | Can every decision be traced to a rule? |
| Performance | Feasible on desktop CPU? |
| Extensibility | Plugin-friendly? |
| Complexity | Implementation cost vs. benefit |

### Step 4: Specification Writing

Produce the module specification in `docs/` or `specifications/` with all required sections.

### Step 5: Consistency Review

Architecture Agent checks:

- Terminology alignment with glossary
- Interface compatibility with AST and pipeline
- No contradictions with design principles
- Dependencies satisfied

### Step 6: Task Generation

Missing dependencies → new research tasks added to `TASKS.md`.

---

## 4. Required Specification Sections

Every research output document **must** contain:

```markdown
# [Module Name] Specification

**Version:** 0.1
**Status:** Draft
**Agent:** [Agent Name]
**Dependencies:** [list]

## 1. Background
## 2. Existing Solutions
## 3. Academic / Theoretical Foundation
## 4. Engineering Analysis
## 5. Comparison of Approaches
## 6. Recommended Solution
## 7. Architecture
## 8. Data Structures
## 9. Algorithms
## 10. Interfaces
## 11. Parameter Mappings
## 12. Explainability Model
## 13. Future Expansion
## 14. Open Questions
## 15. References
```

Minimum target depth: **20 pages** for small modules, **50–100 pages** for core engines.

---

## 5. Parallel Research Strategy

Research must **not** be sequential. Independent topics run in parallel:

| Parallel Group | Topics |
|----------------|--------|
| Group A | Music AST, IR, Timeline, Events |
| Group B | Harmony, Counterpoint, Voice Leading, Jazz |
| Group C | Rhythm, Form, Drums, Orchestration |
| Group D | Rule DSL, Constraints, Scoring/Search |
| Group E | MIDI, ABC, MusicXML, PDF export |
| Group F | Plugin API, Tauri backend, Vue frontend |

Groups with dependencies wait for upstream specs:

- Group D depends on Group A + partial Group B
- Algorithm specs (04-algorithms/) depend on Groups A + B + D
- UI specs depend on Group A + F

---

## 6. Decision Recording

Important decisions require an ADR in `decisions/`:

```
decisions/
  ADR-001-music-ast-primary-representation.md
  ADR-002-search-over-sampling.md
  ADR-003-musicxml-primary-export.md
  ...
```

ADR template:

```markdown
# ADR-NNN: [Title]
**Status:** Proposed | Accepted | Superseded
**Date:** YYYY-MM-DD

## Context
## Decision
## Consequences
## Alternatives Considered
```

---

## 7. Review Process

### Document Review

Each completed spec undergoes:

1. **Terminology check** — against glossary
2. **Principle check** — against design philosophy
3. **Interface check** — AST node types, pipeline stage I/O
4. **Dependency check** — upstream specs referenced correctly

Reviews recorded in `reviews/doc-review-<name>.md`.

### Architecture Review (Pre-Freeze)

When sufficient documents exist:

1. Build dependency graph
2. Find contradictions
3. Merge duplicated concepts
4. Resolve conflicts via ADRs
5. Publish Design Freeze Report

---

## 8. Integration with Deep Research

The [Deep Research Report](../../deep-research-report.md) provides initial findings on:

- Algorithm comparison (rules+search vs Markov vs DL vs hybrid)
- Module architecture sketch
- Parameter mapping table
- Data resources (Lakh MIDI, MAESTRO, Groove MIDI, Bach Chorales)
- Technology stack (Tauri + Vue 3 + Rust engine)

Individual module specs **extend and formalize** this report — they do not duplicate it wholesale. Cross-reference rather than copy.

---

## References

- [Research Roadmap](research-roadmap.md)
- [TASKS.md](../../TASKS.md)
- [Design Philosophy](philosophy.md)
