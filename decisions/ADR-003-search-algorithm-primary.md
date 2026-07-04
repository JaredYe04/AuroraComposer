# ADR-003: Beam Search as Primary Search Algorithm

**Status:** Accepted  
**Date:** 2026-07-04  
**Accepted:** 2026-07-04 (Design Freeze v0.1)  
**Deciders:** Rule Engine Research Agent; Architecture Agent

---

## Context

Aurora Composer's generation pipeline requires a default search algorithm for sequential decision stages (melody, counterpoint, bass, and related note-level engines). The architecture specifies three candidates:

- **Beam search** — bounded-width frontier exploration
- **A\*** — best-first with heuristic
- **Dynamic programming** — optimal substructure on discretized states

The [Deep Research Report](../../deep-research-report.md) recommends beam search for melody generation. ACAS §7.4 lists all three with beam as primary but without formal ADR rationale. The [Scoring and Search Specification](../docs/05-rule-engine/scoring.md) requires a normative decision.

Constraints from design philosophy:

- **Principle 3:** Generation is search with parameterized algorithms
- **Principle 2:** Provenance must trace search decisions (beam rank, parent state)
- **Principle 4:** Quality/speed trade-off must be user-parameterized

Research survey ([rule-engine-research-notes.md](../../research/rule-engine-research-notes.md)) shows constraint-based systems (OpenMusic, Strasheela) embed search in solver runtime without explainable beam structure — Aurora must differ.

**Note:** ACAS Appendix B lists "ADR-003: MusicXML as primary export" — numbering conflict. Architecture Agent should renumber MusicXML ADR to ADR-004 or later. This document uses ADR-003 per Rule Engine Agent assignment.

---

## Decision

Adopt **beam search** as the **primary default search algorithm** for all sequential note-level generation stages in Aurora Composer v0.1.

Specifically:

1. Default `search.mode = auto` resolves to beam search for Melody, Counterpoint, and Bass pipeline stages.
2. Default beam width is controlled by `search.beam_width` (default: 16).
3. A* and DP remain **supported alternatives** for stages where they are better suited (structure planning, harmony skeleton).
4. Hard constraints prune before beam ranking ([constraint.md](../docs/05-rule-engine/constraint.md)).
5. Soft rules score survivors via the formal scoring function ([scoring.md](../docs/05-rule-engine/scoring.md)).
6. Every committed event records beam provenance: `search_step`, `beam_rank`, `parent_state_id`.

---

## Consequences

### Positive

- **Bounded memory:** O(beam_width × state_size) — predictable on desktop targets
- **Parameter mapping:** `search.beam_width` gives direct quality/speed knob for users
- **Explainability:** Beam rank and parent pointers map cleanly to Inspector UI (Principle 2)
- **Parallelism:** Independent beam branch evaluation parallelizes on Rayon (architecture §5)
- **No heuristic requirement:** Unlike A*, works without admissible `h(n)` for subjective melodic goals
- **Incremental generation:** Natural step-by-step alignment with beat/event candidate generators
- **Deterministic mode:** `search.temperature = 0` with stable tie-breaking enables reproducible output

### Negative

- **Non-optimal:** Beam search does not guarantee globally optimal scores; may miss best path outside beam
- **Sensitivity to width:** Too-narrow beams fail on tight constraint problems (empty beam)
- **Local scoring bias:** Greedy accumulation of soft scores may conflict with long-range form goals
- **Tuning burden:** Default beam width must be calibrated per stage in performance testing

### Mitigations

| Risk | Mitigation |
|------|------------|
| Suboptimal melody | Increase `search.beam_width`; optional A* for theme planning |
| Empty beam | Relaxation ladder in constraint solver; user warning |
| Long-range form | DP for structure/harmony skeleton; FORM/MOTI rules in scoring |
| Performance | Preview mode beam_width=4; parallel branch eval |

---

## Alternatives Considered

### Alternative 1: A* as Primary

**Description:** Use A* with music-theoretic heuristic for all generation stages.

| Pros | Cons |
|------|------|
| Optimal if heuristic admissible | Admissible melodic heuristics difficult to define |
| Well-studied | Priority queue memory unbounded without beam cap |
| | Heuristic tuning becomes hidden "magic" (violates Principle 4) |

**Rejected** as primary. **Accepted** as optional mode for structure/theme stages with conservative heuristics.

### Alternative 2: Dynamic Programming as Primary

**Description:** Discretize all decisions; solve globally via DP.

| Pros | Cons |
|------|------|
| Globally optimal on model | State explosion for multi-voice note-level generation |
| Fast for small state spaces | Discretization loses rhythmic/melodic nuance |
| | Poor fit for continuous parameter spaces |

**Rejected** as primary. **Accepted** for harmony skeleton and form template optimization.

### Alternative 3: Pure Constraint Programming (No Scoring Search)

**Description:** Strasheela-style CP backtracking without weighted beam.

| Pros | Cons |
|------|------|
| Strong correctness | All-or-nothing; poor parameter gradation |
| | Slow for interactive multi-voice generation |
| | Weak provenance compared to beam trace |

**Rejected.** Hard constraints only; soft rules require scored search.

### Alternative 4: SMT Optimization (Z3)

**Description:** Encode generation as SMT/OMT problem.

| Pros | Cons |
|------|------|
| Provable satisfaction | Encoding cost prohibitive |
| | Latency unsuitable for interactive UI |
| | Poor explainability for musicians |

**Rejected** for search. Optional validation plugin only.

### Alternative 5: Greedy / Best-First (beam_width = 1)

**Description:** Single-path greedy generation.

| Pros | Cons |
|------|------|
| Fastest | Lowest quality; no recovery from early mistakes |

**Rejected** as default. Available via `search.beam_width = 1` for preview.

### Alternative 6: Stochastic Sampling (Markov / MCMC)

**Description:** Sample from weighted distribution without beam structure.

| Pros | Cons |
|------|------|
| Fast | Violates Principle 3 (search-based generation) |
| | Poor explainability |

**Rejected** as core algorithm. ML plugins may propose candidates; beam search selects.

---

## Implementation Notes

- `SearchEngine::run_beam` is the required Phase 2 prototype implementation
- Pipeline stages declare `search_involvement: yes` with default beam config in module specs
- Inspector must display beam provenance fields from ACAS §7
- Performance benchmarks in `docs/09-engineering/performance.md` (pending) shall validate width 16 target

---

## Related Documents

- [Scoring and Search Specification](../docs/05-rule-engine/scoring.md)
- [Constraint System Specification](../docs/05-rule-engine/constraint.md)
- [Rule DSL Specification](../docs/05-rule-engine/rule-dsl.md)
- [Design Philosophy — Principle 3](../docs/00-overview/philosophy.md)
- [System Architecture §6.2](../docs/01-architecture/architecture.md)
- [Research Notes](../../research/rule-engine-research-notes.md)

---

## Status History

| Date | Status | Notes |
|------|--------|-------|
| 2026-07-04 | Proposed | Rule Engine Research Agent recommendation |
| 2026-07-04 | Accepted | Architecture Review v0.1 — Design Freeze |
