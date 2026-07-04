# Project Vision

**Document:** Aurora Composer — Project Vision  
**Version:** 0.1  
**Status:** Draft  
**Phase:** Architecture Research

---

## 1. What Aurora Composer Is

Aurora Composer is **not** "automatic composition software."

Aurora Composer is a **parameterized, explainable, extensible Music Composition Engine** — an open platform comparable in ambition to a game engine or compiler toolchain, but for music.

It provides:

- A unified **Music AST** as the single source of musical truth
- A **Rule System** encoding music theory as constraints and scoring functions
- A **Search Engine** that explores candidate compositions under those constraints
- A **Generation Pipeline** that produces multi-voice scores from user parameters
- **Explainability** for every generated event (note, chord, rest, drum hit)
- **Plugin extensibility** for styles, algorithms, and optional AI modules

---

## 2. What Aurora Composer Is Not

| Not This | Why |
|----------|-----|
| MVP / demo | Quality and architecture come first |
| AI toy | AI is an optional plugin, not the core |
| Black-box generator | Every output must be explainable |
| MIDI randomizer | Generation is constraint-driven search |
| Single-style tool | Designed for multiple styles via plugins |

---

## 3. Core Value Proposition

### For Composers and Musicians

- Control every dimension of generation through parameters (emotion, harmony complexity, voice density, rhythm, form, etc.)
- Inspect **why** any note was generated — which rule, score, and reasoning chain produced it
- Edit, regenerate, or override any section while preserving structural coherence

### For Developers and Researchers

- Open, documented architecture (ACAS) as the single specification
- Pluggable engines for harmony, melody, rhythm, drums, form
- Standard export to MusicXML, ABC, MIDI, PDF
- Optional ML/AI plugins with well-defined interfaces

### For the Ecosystem

- Third-party style packs, rule sets, and algorithm plugins
- Long-term maintainability through specification-first development

---

## 4. Architectural Positioning

Aurora Composer sits at the intersection of:

```
Music Theory  +  Constraint Solving  +  Search  +  Parameterization
                              ↓
                    Music Composition Engine
                              ↓
              UI · Export · Playback · Plugins · (Optional AI)
```

**AI is a plugin.** The core is rule-driven search grounded in music theory.

---

## 5. Target Capabilities (Design Freeze Scope)

At Design Freeze v0.1, the architecture must support:

1. **Multi-voice composition** — melody, harmony, bass, counterpoint, drums
2. **Parameterized generation** — 30+ user-facing parameters mapped to internal weights
3. **Explainable output** — provenance chain for every musical event
4. **Multi-theme form** — A–B–A′ structures with transitions and motif development
5. **Multiple export formats** — MusicXML (primary), ABC, MIDI, PDF preview
6. **Real-time preview** — browser-based playback during parameter adjustment
7. **Plugin API** — unified interface for style, harmony, rhythm, theme, and AI plugins

Implementation of these capabilities begins only after Design Freeze (Phase 2: Prototype).

---

## 6. Long-Term Vision

Aurora Composer aims to become the **reference open-source music composition platform**:

- Used by educators to demonstrate music theory in action
- Used by indie game/film composers for procedural underscore
- Used by researchers studying algorithmic composition
- Extended by a community of plugin authors

The 500–1000 page specification corpus ensures the project survives beyond any single contributor or AI session.

---

## 7. Success Criteria

| Criterion | Measure |
|-----------|---------|
| Explainability | 100% of generated events have provenance metadata |
| Controllability | Parameter changes produce statistically predictable output changes |
| Theoretical correctness | Zero hard-constraint violations in validated output |
| Extensibility | New style plugin without core engine modification |
| Documentation completeness | All modules specified before any production code |

---

## References

- [Deep Research Report](../../deep-research-report.md)
- [Design Philosophy](philosophy.md)
- [ACAS v0.1](acas-v0.1.md)
