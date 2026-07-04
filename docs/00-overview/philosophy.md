# Design Philosophy

**Document:** Aurora Composer — Design Philosophy  
**Version:** 0.1  
**Status:** Draft

---

## Overview

These principles are **immutable constraints** on Aurora Composer's architecture. All modules, agents, and future code must comply. Violations require an Architecture Decision Record (ADR) and review before acceptance.

---

## Principle 1: Everything Is Music AST

> All modules operate exclusively on the Music AST.

**Rules:**

- No module may generate MIDI, MusicXML, or audio directly
- All generation, transformation, and validation pass through AST nodes
- Export is a **downstream projection** from AST → IR → format
- Plugins receive AST snapshots and return AST patches

**Rationale:** A single representation eliminates format drift, enables explainability, and allows any exporter or analyzer to consume the same data.

```
❌ Harmony Engine → MIDI file
✅ Harmony Engine → AST (Chord nodes) → Exporter → MIDI
```

---

## Principle 2: Everything Is Explainable

> Every generated musical event must carry a provenance chain.

**Provenance model:**

```
Event (Note / Chord / Rest / DrumHit)
  └── Reason          ("Passing tone between C and E")
        └── Rule      (HarmonyRule #42: "Allow passing tones on weak beats")
              └── Score (+13)
                    └── History (search step, beam rank, parent state)
```

**UI requirement:** Clicking any note in the inspector must display this chain.

**Rationale:** Commercial auto-composition tools are black boxes. Explainability is Aurora Composer's primary differentiator and essential for debugging, education, and trust.

---

## Principle 3: Generation Is Search

> Composition is constraint-driven search, not random sampling or opaque AI inference.

**Rules:**

- Candidate generation produces a finite set of possibilities at each step
- A scoring function (rules + weights) evaluates each candidate
- A search algorithm (beam search, A*, DP) selects optimal paths
- Randomness, when used, is seeded and parameterized (temperature), never uncontrolled

**Rationale:** Search provides controllability, reproducibility, and explainability. AI plugins may propose candidates or adjust weights, but the core decision loop remains search-based.

---

## Principle 4: Everything Is Parameterized

> Every generative behavior must be controllable through named parameters.

**Parameter categories:**

| Category | Examples |
|----------|----------|
| Emotion | valence, arousal, tension curve |
| Style | genre preset, era, orchestration |
| Harmony | complexity, dissonance tolerance, cadence strength |
| Melody | motif length, leap limit, ornament density |
| Rhythm | density, syncopation, subdivision fineness |
| Form | section count, theme count, repetition ratio |
| Voice | density, register range, counterpoint strictness |
| Texture | homophonic ↔ polyphonic balance |
| Dynamics | range, accent pattern |
| Drums | density, fill frequency, pattern complexity |

**Rules:**

- Parameters have documented defaults, ranges, and internal mappings
- No "magic constants" in algorithms — all weights derive from parameters
- Parameter changes must produce predictable, testable output shifts

---

## Principle 5: Specification Before Code

> No production code until the architecture is frozen.

**Workflow:**

```
Research → Specification → Review → Freeze → Implementation
```

**Rules:**

- Every module has a specification document before coding
- Code that conflicts with specs is wrong
- Specs are never retrofitted to match code

---

## Principle 6: Music Theory First

> The engine encodes established music theory; it does not approximate it statistically.

**Priority order:**

1. Hard constraints from theory (e.g., no parallel perfect fifths in strict counterpoint mode)
2. Soft scoring from theory (e.g., prefer stepwise motion)
3. Statistical/ML augmentation (optional plugins only)

**Rationale:** Rule-driven generation guarantees correctness under specified style constraints. ML may assist (emotion mapping, motif transformation) but cannot override hard constraints without explicit user opt-in.

---

## Principle 7: Modular and Pluggable

> Core engine provides infrastructure; specialized knowledge lives in plugins.

**Plugin types:**

- Style plugins (genre presets, parameter bundles)
- Harmony plugins (jazz voicings, baroque figured bass)
- Rhythm plugins (pattern libraries)
- Theme plugins (motif transformation algorithms)
- AI plugins (candidate proposal, weight adjustment)
- Export plugins (custom formats)

**Rules:**

- All plugins implement documented interfaces (see `docs/07-plugin/`)
- Plugins cannot bypass AST or explainability requirements
- Core engine remains functional with zero plugins installed (default classical/pop rule set)

---

## Principle 8: Documents Are the Source of Truth

> The documentation repository (`docs/`) is authoritative.

- Agents read `docs/` before implementing
- Architecture decisions are recorded in `decisions/`
- Reviews are recorded in `reviews/`
- Research notes live in `research/` and feed into specifications

---

## Anti-Patterns (Explicitly Forbidden)

| Anti-Pattern | Why Forbidden |
|--------------|---------------|
| Direct MIDI generation | Breaks AST centrality |
| Undocumented magic numbers | Breaks parameterization |
| Black-box generation steps | Breaks explainability |
| Coding before spec | Breaks specification-first |
| Monolithic god-module | Breaks modularity |
| AI as default generator | Breaks theory-first |

---

## References

- [ACAS v0.1](acas-v0.1.md)
- [Terminology](terminology.md)
- [Research Methodology](research-methodology.md)
