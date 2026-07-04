# Decoration Engine Specification

**Version:** 0.1  
**Status:** Frozen (Design Freeze v0.1)  
**Agent:** Algorithm Engines Research Agent (Decoration)  
**Dependencies:** [pipeline.md](../01-architecture/pipeline.md), [melody-engine.md](melody-engine.md), [ast.md](../02-music-model/ast.md), [harmony.md](../03-theory/harmony.md)

---

## Table of Contents

1. [Background](#1-background)
2. [Existing Solutions](#2-existing-solutions)
3. [Academic / Theoretical Foundation](#3-academic--theoretical-foundation)
4. [Engineering Analysis](#4-engineering-analysis)
5. [Comparison of Approaches](#5-comparison-of-approaches)
6. [Recommended Solution](#6-recommended-solution)
7. [Architecture](#7-architecture)
8. [Data Structures](#8-data-structures)
9. [Algorithms](#9-algorithms)
10. [Interfaces](#10-interfaces)
11. [Parameter Mappings](#11-parameter-mappings)
12. [Explainability Model](#12-explainability-model)
13. [Future Expansion](#13-future-expansion)
14. [Open Questions](#14-open-questions)
15. [References](#15-references)

---

## 1. Background

### 1.1 Purpose

The **Decoration Engine** implements **Pipeline Stage 11: Decoration** — post-hoc ornamental enrichment of existing melodic/harmonic lines without altering structural skeleton notes.

### 1.2 Pipeline I/O

| Property | Value |
|----------|-------|
| **Stage** | 11 — Decoration |
| **Search** | No (rule-filtered greedy insertion) |
| **AST Read** | `Voice[melody]`, `Voice[inner]*`, rhythm skeleton, harmony slots |
| **AST Write** | `Note.ornaments[]`, grace notes, `Event.provenance` (decoration source) |
| **Input** | `melody.ornament_density`, `style.era`, `texture.*` |

**Invariant:** Decoration MUST NOT delete or transpose structural notes committed by Stages 7–9.

---

## 2. Existing Solutions

| System | Ornamentation |
|--------|---------------|
| **MusicXML `<ornaments>`** | Trill, mordent, turn, grace | Export target |
| **Baroque figured bass realizations** | Rule-based fills | Theoretical basis |
| **DAW MIDI CC** | Controller-based | Not symbolic |
| **Deep research** | 5% ornament candidate in melody beam | **Split out** to dedicated stage |

Aurora separates **structural melody** (Stage 7) from **decoration** (Stage 11) for explainability and user override.

---

## 3. Academic / Theoretical Foundation

### 3.1 Ornament Types (Baroque through Classical)

| Type | Description | Typical Context |
|------|-------------|-----------------|
| **Grace note** | Acciaccatura, appoggiatura | Strong beat approach |
| **Trill** | Rapid alternation | Hold notes, cadences |
| **Mordent** | Lower/upper neighbor flick | Baroque, Classical |
| **Turn** | Enclosing neighbor group | Classical |
| **Passing tone fill** | Short connective note | Already in Stage 7; not duplicated |

### 3.2 Style Constraints

- Baroque/Classical: trills, mordents on long notes
- Pop/Jazz: grace notes, slides (gliss as marker)
- Minimal/Electronic: decoration skipped when `melody.ornament_density < 0.1`

---

## 4. Engineering Analysis

### 4.1 Candidate Selection

For each structural `Note` with duration ≥ quarter note:

1. Compute ornament eligibility score from duration, beat strength, cadence proximity
2. Filter by style preset ornament vocabulary
3. Insert ornament events as child metadata or adjacent grace notes
4. Validate against rhythm grid (RHY-012) and voice range (REG-*)

### 4.2 Performance

O(n) over melody notes; no search. Typical 32-bar piece: <50ms.

---

## 5. Comparison of Approaches

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| Beam search ornaments | High quality | Slow; confuses structural provenance | Rejected |
| **Rule-filtered greedy** | Fast, explainable | Less global optimization | **Selected** |
| ML ornament model | Expressive | Black box | Plugin only (Phase 3) |
| Realize in export only | Simple | No AST editability | Rejected |

---

## 6. Recommended Solution

Greedy ornament insertion with:

- Style vocabulary table
- Density parameter controlling insertion probability
- Hard skip when note is tied across barline or part of motif anchor

---

## 7. Architecture

```text
DecorationEngine
├── OrnamentVocabulary (style → allowed types)
├── EligibilityScorer (note context → score)
├── OrnamentRealizer (type → AST patch)
└── DecorationValidator (RHY + REG checks)
```

**Pipeline position:** After Drums (10), before Repair (12).

---

## 8. Data Structures

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrnamentKind {
    GraceAcciaccatura,
    GraceAppoggiatura,
    Trill,
    UpperMordent,
    LowerMordent,
    Turn,
    Slide, // marker only
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrnamentAttachment {
    pub kind: OrnamentKind,
    pub parent_note_id: NodeId,
    pub grace_notes: Vec<GraceNoteSpec>,
    pub provenance: Provenance,
}

pub struct DecorationPlan {
    pub candidates: Vec<OrnamentCandidate>,
}
```

Ornaments attach to `NoteEvent.ornaments: Vec<OrnamentAttachment>` per [events.md](../02-music-model/events.md).

---

## 9. Algorithms

```text
function Decorate(composition, params):
    if params.melody.ornament_density < 0.05:
        return composition  // skip stage

    vocab = OrnamentVocabulary.for_style(params.style)
    patches = []

    for note in structural_melody_notes(composition):
        if not eligible(note, params):
            continue
        if random(params) > params.melody.ornament_density * eligibility(note):
            continue
        kind = vocab.select(note.context)
        ornament = realize(kind, note)
        if validate(ornament, composition):
            patches.append(attach_ornament(note.id, ornament))

    return apply_patches(composition, patches)
```

---

## 10. Interfaces

```rust
pub trait DecorationEngine: Send + Sync {
    fn decorate(
        &self,
        ast: &Composition,
        params: &ParameterBundle,
    ) -> Result<(Composition, Vec<Patch>), EngineError>;
}
```

---

## 11. Parameter Mappings

| Parameter | Range | Effect |
|-----------|-------|--------|
| `melody.ornament_density` | 0.0–1.0 | Insertion probability multiplier |
| `style.era` | enum | Vocabulary selection |
| `style.genre` | enum | Jazz slides vs Baroque trills |
| `texture.homophony_polyphony_balance` | 0–1 | Skip inner voices when >0.7 |

---

## 12. Explainability Model

```json
{
  "source": "decoration_engine",
  "stage": 11,
  "rule_id": "DECOR-003",
  "reason": "Upper mordent on quarter note at phrase midpoint",
  "score_delta": 0,
  "parent_note_id": "node://note/42"
}
```

Inspector displays ornament as child of structural note; structural provenance unchanged.

---

## 13. Future Expansion

- User-defined ornament macros
- AI ornament plugin proposing candidates (beam selection optional)
- Automatic trill realization speed from tempo

---

## 14. Open Questions

1. Store grace notes as separate Event nodes or Note attributes only?
   - **v0.1 decision:** Attributes on parent Note; IR expands to separate events for MIDI
2. Counterpoint voice decoration in strict polyphonic mode?

---

## 15. References

- [Melody Engine](melody-engine.md)
- [Events Specification](../02-music-model/events.md)
- Donington, *The Interpretation of Early Music*
- ACAS Pipeline Stage 11

---

## Appendix A: Ornament Rule Catalog

| Rule ID | Type | Constraint |
|---------|------|------------|
| DECOR-001 | HARD | No ornament on rests |
| DECOR-002 | HARD | Grace notes must fit within beat grid |
| DECOR-003 | SOFT | Prefer ornaments on beat 2/4 in 4/4 |
| DECOR-004 | SOFT | Avoid ornaments at cadence approach tone |
| DECOR-005 | HARD | No ornament if `melody.ornament_density = 0` |
