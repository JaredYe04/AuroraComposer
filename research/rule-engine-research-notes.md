# Rule Engine Research Notes

**Topic:** Constraint-Based Composition Systems  
**Agent:** Rule Engine Research Agent  
**Date:** 2026-07-04  
**Status:** Raw research — feeds `docs/05-rule-engine/*`

---

## 1. Purpose

Survey prior art in algorithmic and constraint-based composition to inform Aurora Composer's Rule DSL, constraint solver, and search architecture. These notes are **not normative**; specifications in `docs/05-rule-engine/` supersede them.

---

## 2. Systems Surveyed

| System | Origin | Paradigm | Language | Notable Strength | Limitation for ACAS |
|--------|--------|----------|----------|------------------|---------------------|
| **OpenMusic** | IRCAM | Visual constraint networks + Lisp | Common Lisp / OM patches | Rich constraint propagation; academic pedigree | Steep learning curve; no explainable provenance model; not parameter-driven for end users |
| **Strasheela** | Torsten Anders | Constraint programming on top of OpenMusic | Oz/Mozart via OpenMusic | Declarative rules; global search | Performance; closed ecosystem; weak export pipeline |
| **Score-PMC** | Anders | Pure constraint solver for scores | Prolog-like | Theoretically complete constraint satisfaction | Not interactive; no modern UI |
| **Music21** | MIT | Analysis + limited generation | Python | Excellent theory toolkit; corpus analysis | Not a composition search engine; rules are code not DSL |
| **Lenardo / OM#** | IRCAM successor | Visual + text hybrid | Lisp | Modernized OpenMusic lineage | Same explainability gap for commercial users |
| **Bandit / OMax** | IRCAM | Machine learning improvisation | Max/MSP | Real-time reactivity | Black-box; contradicts theory-first principle |
| **AIVA** | Commercial | DL + hidden rules | Proprietary | Production quality output | No user-facing rule transparency |
| **FlowComposer** | Sony CSL | Markov + constraints | Internal | Hybrid approach | Limited documentation |
| **Euterpea** | Haskell | Functional music DSL | Haskell | Elegant embedding | No constraint solver; generation via probability |
| **MiniZinc / OR-Tools** | Generic CP | SMT/CP backends | MiniZinc | Industrial-strength solvers | Music requires heavy modeling layer |
| **Bol Processor** | N. R. | Grammar rules | BP2 language | Long-form grammar composition | Obsolete tooling; no AST integration |

---

## 3. OpenMusic (Deep Dive)

### 3.1 Architecture

OpenMusic represents musical objects as **patches** (visual programs). Constraint satisfaction uses **PW-Constraint** (PatchWork Constraint) and successors:

- Variables: pitch, duration, onset, voice assignment
- Domains: finite sets (scale degrees, rhythmic grids)
- Constraints: relations between variables (interval = M3, no parallel fifths)
- Search: backtracking with propagation

### 3.2 Relevant Patterns for Aurora

| OpenMusic Pattern | Aurora Adaptation |
|-------------------|-------------------|
| Variable domains on pitch classes | `domain()` clause in Rule DSL tied to current key/mode |
| Reified constraints (soft) | `mode: soft` with penalty weights |
| Multi-criteria optimization | Weighted scoring function (not pure SAT) |
| Hierarchical composition | Pipeline stages with localized search scopes |

### 3.3 Gaps

- No first-class **provenance** per note decision
- Rules embedded in Lisp patches, not portable declarative files
- User parameters are patch inputs, not a unified parameter system
- Export to MusicXML exists but is not generation-centric

**Reference:** Laurson, M. (1996). *PatchWork: A Visual Programming Language for Musical Composition.* IRCAM.

---

## 4. Strasheela (Deep Dive)

### 4.1 Architecture

Strasheela (Anders, 2007) layers a **constraint programming language** on OpenMusic:

```text
Score = collection of variables (pitch, start, duration)
Rules = relations (e.g., ForAll voices v: noParallelPerfect(v))
Solver = distribution + search strategy in Oz
```

Key insight: **global constraints** (e.g., entire phrase must form valid counterpoint) vs **local constraints** (next note must be stepwise).

### 4.2 Search Strategies Used

1. **First-fail variable ordering** — choose most constrained variable
2. **Value ordering** — prefer stepwise motion, chord tones
3. **Restart-based search** — abandon partial solutions
4. **Soft constraints via cost variables** — minimize violation sum

### 4.3 Lessons for Aurora

- **Separate constraint definition from search strategy** — Strasheela succeeds here; Aurora Rule DSL should compile independently of beam/A*/DP choice
- **Global vs local scope** — Aurora uses pipeline stages to decompose global problems
- **Oz performance** — full CP on desktop for 4-voice chorale-scale is feasible; full symphonic score is not. Aurora must scope search per stage/voice group

**Reference:** Anders, T. (2007). *Composing Music by Composing Rules: Design and Usage of a Generic Music Constraint System.* PhD thesis, Queen's University Belfast.

---

## 5. Pure Constraint Programming vs Weighted Scoring vs SMT

### 5.1 Pure Constraint Programming (CP)

**Model:** All rules are hard constraints; solution = any satisfying assignment.

| Pros | Cons |
|------|------|
| Guarantees correctness when satisfiable | All-or-nothing; unsat gives no partial result |
| Well-studied propagation algorithms | Difficult to express "prefer stepwise" |
| Natural for voice-leading prohibitions | Parameter "strictness" requires many rule variants |

**Verdict for Aurora:** Use CP **mechanisms** (propagation, domain pruning) for hard constraints only. Do not adopt pure CP as the sole paradigm.

### 5.2 Weighted Scoring (Soft Constraints)

**Model:** `eval_score = Σ(w_i × indicator_i) − Σ(p_j × violation_j)`

| Pros | Cons |
|------|------|
| Graceful degradation; always returns best effort | No guarantee of theoretical correctness |
| Direct parameter mapping (weights) | Local optima in search |
| Explainable per-rule contributions | Weight tuning requires calibration |

**Verdict for Aurora:** **Primary paradigm** per ACAS §7.3 and design philosophy Principle 6 (hard first, soft second).

### 5.3 SMT Solvers (Z3, CVC5)

**Model:** Encode music theory as logical formulas; ask SAT/OPT solver for model.

| Pros | Cons |
|------|------|
| Complete for decidable fragments | Encoding music theory is labor-intensive |
| Can optimize linear objectives | Non-linear preferences (melodic contour) awkward |
| Industrial tooling | Latency unpredictable for incremental generation |
| Good for validation | Poor for interactive step-by-step composition |

**Verdict for Aurora:** Optional **validation backend** for hard constraint verification post-generation. Not primary search engine. Consider Z3 plugin for "strict counterpoint proof" mode.

### 5.4 Comparison Matrix

| Criterion | Pure CP | Weighted Scoring | SMT |
|-----------|---------|------------------|-----|
| Controllability | Low (binary) | **High** | Medium |
| Explainability | Medium | **High** | Low |
| Partial solutions | No | **Yes** | No |
| Interactive latency | Medium | **Good (beam)** | Poor |
| Implementation cost | Medium | **Medium** | High |
| Theory correctness | **Strong** | Configurable | **Strong** |

**Recommendation:** Hybrid — hard constraints with propagation + soft scoring + beam search; SMT optional for validation.

---

## 6. Music21 and Rule Encoding

Music21 encodes theory as **Python code**, not declarative rules:

```python
# Illustrative — not Aurora code
s = stream.Part()
s.append(note.Note('C4'))
s.analyze('key')
s.show('text')
```

**Relevant for Aurora:**

- Interval/chord classifiers → Theory Engine API consumed by Rule DSL `expr` clauses
- `voiceLeading.VoiceLeadingQuartet` analysis → reference for counterpoint rule predicates
- Corpus of Bach chorales → regression test fixtures

Music21 is **not** a search engine; Aurora fills that gap.

---

## 7. Commercial Systems

### AIVA

- Neural network generates MIDI; rules applied opaquely in post-processing (reported in press materials, not open spec)
- **Anti-pattern for Aurora:** black-box generation

### Amper / Soundraw (historical)

- Template + ML hybrid; user controls mood/genre sliders
- **Relevant:** parameter → weight mapping UX patterns

---

## 8. Academic Foundations

| Source | Relevance |
|--------|-----------|
| Rameau, *Treatise on Harmony* | Harmonic rule categories |
| Fux, *Gradus ad Parnassum* | Counterpoint rule IDs |
| Aldwell & Schachter | Voice-leading soft preferences |
| Pachet & Roy, *Markov Constraints* | Hybrid CP + probability (future plugin) |
| Anders & Miranda, *Constraint Programming for Music* | Strasheela theoretical basis |
| Laurson & Kuuskankare, *PWGL* | Visual constraint patching |

---

## 9. Engineering Analysis (Per Methodology §3)

| Criterion | Finding |
|-----------|---------|
| **Correctness** | Hard constraints for parallel fifths, range limits; soft for preferences |
| **Controllability** | Weighted scoring maps cleanly to ACAS parameter system |
| **Explainability** | Rule DSL with provenance beats OpenMusic/Strasheela |
| **Performance** | Beam search with width 8–32 feasible on desktop; full CP global search not |
| **Extensibility** | Plugin-delivered `.rule` files; compile to RuleSet |
| **Complexity** | Rule DSL compiler + evaluator moderate; SMT backend optional high cost |

---

## 10. Conflicts with Existing Docs

| Item | Conflict | Resolution |
|------|----------|------------|
| Deep research mentions React | ACAS specifies Vue 3 | No impact on rule engine |
| ADR-002 numbering | User requests ADR-003 for search; ACAS appendix lists ADR-003 as MusicXML | **Flag:** Architecture Agent must renumber; this agent proposes `ADR-003-search-algorithm-primary.md` as requested |
| AST spec pending | Rule DSL references AST paths tentatively | Use provisional paths from ACAS §4; update when `ast.md` lands |

---

## 11. Recommended Direction (Summary)

1. **Declarative Rule DSL** (`.rule` files) — inspired by Strasheela declarative style, not OpenMusic visual patches
2. **Hybrid constraint model** — hard (prune) + soft (score)
3. **Beam search primary** — see ADR-003
4. **A* for optional global passes** — structure planning, theme alignment
5. **DP for skeleton planning** — harmony rhythm grids
6. **SMT validation optional** — strict mode plugin
7. **Provenance mandatory** — differentiator vs all surveyed systems

---

## 12. Open Questions

1. Should Rule DSL support temporal logic (LTL) for form constraints?
2. Maximum RuleSet size before compilation caching required?
3. WASM compilation of rule evaluator for plugin sandbox?
4. Standard rule library versioning (semver per rule pack)?

---

## 13. References

- Anders, T. (2007). *Composing Music by Composing Rules.* PhD thesis.
- Laurson, M., & Kuuskankare, M. (2000). *PWGL: A Novel Visual Language for Music.*
- Pachet, F., & Roy, P. (2001). *Markov Constraints.*
- Cope, D. (2000). *The Algorithmic Composer.*
- Music21 Documentation: https://web.mit.edu/music21/
- OpenMusic: https://openmusic-project.github.io/
- Deep Research Report: `deep-research-report.md`
- ACAS v0.1: `docs/00-overview/acas-v0.1.md`
