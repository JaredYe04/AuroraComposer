# Project Goals

**Document:** Aurora Composer — Project Goals  
**Version:** 0.1  
**Status:** Draft

---

## 1. Primary Goals

### G1: Build a Professional Composition Engine

Create a production-grade, multi-voice music composition engine — not a prototype or demo — with architecture comparable to game engines or compiler toolchains.

### G2: Achieve Full Explainability

Every note, chord, rest, and drum event in generated output must trace to a rule, score, and human-readable reason.

### G3: Enable Full Parameterization

Users control emotion, style, harmony complexity, rhythm, form, voice leading, texture, and more through a unified parameter system with documented internal mappings.

### G4: Complete Architecture Before Code

Deliver 500–1000 pages of specification across 40+ documents. Freeze architecture before any production implementation.

### G5: Support Extensibility

Provide a plugin API allowing third parties to add styles, rules, algorithms, and optional AI modules without modifying core engine code.

---

## 2. Secondary Goals

### G6: Multi-Format Export

Support MusicXML (primary interchange), ABC (folk/simple notation), MIDI (playback), and PDF (via MusicXML rendering).

### G7: Real-Time Interactive Preview

Allow parameter adjustment with incremental regeneration and browser-based audio preview.

### G8: Multi-Theme Form Generation

Support sectional forms (A–B–A′, sonata-like structures) with motif development, transitions, and key changes.

### G9: Cross-Platform Desktop Application

Deliver via Tauri (Rust backend) + Vue 3 (frontend), compilable to native desktop and optionally WASM.

### G10: Community and Research Value

Publish as open documentation and eventually open source, serving educators, researchers, and indie creators.

---

## 3. Non-Goals (Phase 1)

The following are explicitly **out of scope** during Architecture Research:

- Production code implementation
- UI mockups or prototypes
- ML model training
- Audio synthesis beyond preview playback spec
- DAW integration
- Real-time performance (live improvisation)
- Lyrics / vocal generation
- Microtonal / extended tuning systems (future expansion)

---

## 4. Quality Targets (Post-Prototype)

| Target | Metric |
|--------|--------|
| Theoretical correctness | 0 hard-constraint violations in validated mode |
| Parameter responsiveness | Measurable output change within 1 regeneration |
| Generation latency | < 30s for 32-bar multi-voice piece on 4-core CPU |
| Export fidelity | Round-trip MusicXML → AST → MusicXML lossless for core elements |
| Subjective quality | ≥ 7/10 listener rating for 3 reference styles |

---

## 5. Documentation Goals

| Deliverable | Target |
|-------------|--------|
| Overview & ACAS | ~80 pages |
| Music Model | ~120 pages |
| Theory System | ~250 pages |
| Algorithms | ~200 pages |
| Rule Engine | ~90 pages |
| Export & Plugin | ~100 pages |
| UI & Engineering | ~80 pages |
| **Total** | **500–1000 pages** |

---

## References

- [Vision](vision.md)
- [Roadmap](../09-engineering/roadmap.md)
- [PROGRESS.md](../../PROGRESS.md)
