# Algorithm Engines Research Notes

**Topic:** Generation Algorithm Engines (Structure, Harmony, Melody, Phrase, Motif, Drums, Emotion, Repair)  
**Agent:** Algorithm Engines Research Agent  
**Date:** 2026-07-04  
**Status:** Raw research — feeds `docs/04-algorithms/*`

---

## 1. Purpose

Survey algorithmic composition approaches, parameter mapping patterns, and data resources to inform Aurora Composer's eight algorithm engine specifications. These notes are **not normative**; specifications in `docs/04-algorithms/` supersede them.

---

## 2. Dependency Analysis (Step 1)

### Required Prior Documents (Read)

| Document | Used For |
|----------|----------|
| `docs/00-overview/acas-v0.1.md` | Parameter categories, module inventory |
| `docs/01-architecture/pipeline.md` | 14 stages, I/O contracts |
| `docs/02-music-model/ast.md` | Node types, provenance |
| `docs/03-theory/*` | Rule IDs (395 rules cataloged) |
| `docs/05-rule-engine/*` | Scoring, constraints, beam pseudocode |
| `deep-research-report.md` | Algorithm comparison, parameter table |

### Conflicts Flagged

| Issue | Resolution |
|-------|------------|
| Deep research React vs ACAS Vue | UI irrelevant to algorithms; no conflict |
| Phrase Engine not a numbered pipeline stage | Specified as cross-cutting service; hooks Stages 3–7 |
| Motif vs Theme Planning stage boundary | Motif Engine owns Stage 4 logic; phrase-engine separate |
| Harmony voicing depth | Skeleton voicing in Stage 5; full SATB in Stage 8 (counterpoint) |

### New Dependencies Discovered

- Phrase planning must precede harmony cadence enforcement → documented in harmony-engine.md
- Emotion deltas must be available before Stage 3 DP → pipeline order confirmed

---

## 3. Background Research (Step 2)

### 3.1 Academic Sources

| Domain | Source | Engine Application |
|--------|--------|-------------------|
| Form | Caplin *Classical Form* | structure-engine templates |
| Form | Schoenberg *Fundamentals* | motif development operators |
| Harmony | Kostka & Payne | harmony DP transitions |
| Jazz | Levine *Jazz Theory* | harmony beam / JAZZ graph |
| Counterpoint | Fux (via Salzer) | melody leap compensation |
| Phrase | Lerdahl & Jackendoff GTTM | phrase-engine roles |
| Emotion | Russell circumplex | emotion-engine axes |
| Rhythm/Drums | Gillick Groove MIDI paper | drum taxonomy |

### 3.2 Software Survey

| System | Relevant Pattern | Aurora Adoption |
|--------|------------------|-----------------|
| OpenMusic / Strasheela | Separate planning vs realization | Pipeline stages |
| Band-in-a-Box | Template + fill | structure templates, drum patterns |
| Music21 | Roman numeral analysis | Harmony state representation |
| Lenardo | Section clips | Section roles in AST |
| AIVA | End-to-end DL | Rejected as primary; plugin only |

### 3.3 Datasets

| Dataset | Size | Engine Use |
|---------|------|------------|
| **Groove MIDI** | 1150 files, 22k+ bars | drum-engine pattern index |
| **Bach Chorales** | 371 | motif library seeds, harmony validation |
| **Lakh MIDI** | 176k | future progression statistics plugin |
| **MAESTRO** | 2000+ hrs piano | melody contour statistics (optional) |
| **ABC folk corpus** | thousands | motif library folk tag |

---

## 4. Engineering Analysis (Step 3)

### 4.1 Search Algorithm Assignment

| Engine | Stage | Algorithm | Beam Width | Rationale |
|--------|-------|-----------|------------|-----------|
| Emotion | 2 | None | N/A | Table lookup |
| Structure | 3 | Optional DP | N/A | Discrete section lengths |
| Motif | 4 | Mini beam / A* | 8 / optional | Short motif length |
| Phrase | cross | Narrow beam (cadence) | 4 | Small discrete choice |
| Harmony | 5 | DP or beam | 12 default | Optimal substructure vs jazz branching |
| Melody | 7 | **Beam primary** | **16 default** | Core sequential search |
| Drums | 10 | Greedy + local | 4 fills | Pattern selection sufficient |
| Repair | 12 | Local beam | 6 | Patch depth bounded |

Consistent with `docs/05-rule-engine/scoring.md` §1.3 and ADR-002 (search over sampling).

### 4.2 Performance Estimates (Desktop 4-core)

| Scenario | Estimated Time |
|----------|----------------|
| Full pipeline 32 bars, standard quality | 15–30 s |
| Preview 2 bars (stages 1–7, beam 4) | 2–4 s |
| Harmony-only (stages 1–5) | 3–8 s |
| Repair-only | 1–3 s |

Parallel beam branch evaluation (Rayon) assumed in architecture.

### 4.3 Explainability Requirements

All engines must emit provenance compatible with AST schema:

- Structural nodes: `StructuralProvenance`
- Events: full `Provenance` with `rule_id`, `score_delta`, search metadata
- Repair: `RepairProvenance` chain append

No engine may write events without provenance (Principle 2).

---

## 5. Parameter Mapping Research

Consolidated from deep-research-report §2.2 and ACAS §6:

| User Parameter | Primary Engine(s) | Internal Mapping |
|----------------|-------------------|------------------|
| valence / arousal / tension | emotion → all | WeightDeltaTable |
| form.type / section_count | structure | FormGraph template |
| theme.count / motif_length | motif | Theme slots, beam depth |
| harmony.complexity | harmony | \|States\|, JAZZ extensions |
| melody.chord_tone_bias | melody | Candidate mixture |
| counterpoint.strictness | repair, counterpoint | CONT hard/soft mode |
| drums.density | drums | Groove query density_score |
| search.beam_width | melody, harmony | Algorithm width |

Mapping function pattern (all engines):

```text
weight = lerp(min, max, normalize(param)) × (1 + emotion_delta[category])
```

---

## 6. Rule Category Integration Matrix

| Engine | Primary Rule Prefixes |
|--------|----------------------|
| structure-engine | FORM-*, FORM-SEC-*, FORM-SON-* |
| emotion-engine | (modifies weights for all categories) |
| motif-engine | MOTI-*, FORM-DEV-* |
| phrase-engine | HARM-015..020, FORM-*, MOTI-* |
| harmony-engine | HARM-*, JAZZ-*, HARM-PROG-* |
| melody-engine | HARM-*, VLED-*, MOTI-*, RHYT-*, REG-* |
| drum-engine | DRUM-*, RHYT-* |
| repair-engine | REG-*, CONT-*, VLED-*, HARM-*, DRUM-* |

Total rules cataloged in theory docs: **395** — engines activate subset via style preset.

---

## 7. Comparison of Approaches (Step 3 continued)

### Harmony Progression

| Approach | Score | Notes |
|----------|-------|-------|
| Fixed templates | Fast, low variety | Preview |
| Markov from Lakh | Style plugin data | Not v0.1 primary |
| DP diatonic | Optimal under model | **Pop/classical default** |
| Beam jazz graph | Handles substitutions | **Jazz default** |

### Melody Generation

| Approach | Score | Notes |
|----------|-------|-------|
| Greedy | Fast, local minima | Preview fallback |
| Beam + rules | Controllable quality | **Selected** |
| Transformer | High variety | AI plugin optional |

### Drums

| Approach | Score | Notes |
|----------|-------|-------|
| Procedural random | Poor groove | Rejected |
| Groove MIDI index | Realistic, tagged | **Selected** |
| RNN groove | Good fill | Future plugin |

---

## 8. Specification Outputs (Step 4)

Created normative specs:

| File | Stage | Approx. Scope |
|------|-------|---------------|
| `structure-engine.md` | 3 | Form, key map, tempo, phrases |
| `emotion-engine.md` | 2 | Valence/arousal/tension → deltas |
| `motif-engine.md` | 4 | Motif gen, transforms, library |
| `phrase-engine.md` | cross | Cadence, connection |
| `harmony-engine.md` | 5 | Progression DP/beam, voicing |
| `melody-engine.md` | 7 | Primary beam search |
| `drum-engine.md` | 10 | Groove taxonomy, fills |
| `repair-engine.md` | 12 | Local fix search |

All include 15 required sections per `research-methodology.md`.

---

## 9. Consistency Review Notes (Step 5)

| Check | Status |
|-------|--------|
| Terminology vs glossary | Aligned (Section, Phrase, Motif, Provenance) |
| AST I/O vs ast.md | Consistent node names |
| Pipeline stage numbers | Match pipeline.md |
| Beam widths vs scoring.md | Melody 16, Harmony 12, Repair 6 — documented per engine |
| No contradictions with Principle 3 | Search primary for melody/harmony |

### Items for Architecture Agent

1. Confirm phrase-engine invocation point in orchestrator (after Stage 4 recommended)
2. ADR for harmony DP vs beam auto-switch threshold
3. Groove MIDI redistribution license check before bundling

---

## 10. Open Research Items

1. Admissible heuristic for melody A* — not solved v0.1
2. Pareto beam for advanced users — deferred
3. Cross-stage joint optimization (harmony + melody) — expensive; rejected
4. Learned emotion delta table from labeled MIDI — optional ML phase
5. Optimal Groove subset size for shipping (~200 patterns vs full 1150)

---

## 11. References

- [research-methodology.md](../docs/00-overview/research-methodology.md)
- [deep-research-report.md](../deep-research-report.md)
- [pipeline.md](../docs/01-architecture/pipeline.md)
- [scoring.md](../docs/05-rule-engine/scoring.md)
- Anders, T. (2007) — Strasheela constraint composition
- Gillick et al. — Groove MIDI Dataset (2019)

---

*End of Algorithm Engines Research Notes*
