# Form & Structure Theory Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Theory System Research Agent (Form)  
**Dependencies:** [harmony.md](harmony.md), [rhythm.md](rhythm.md), [terminology.md](../00-overview/terminology.md)

---

## Rule ID Naming Convention

| Prefix | Domain | Example |
|--------|--------|---------|
| `FORM` | Form general | `FORM-001` |
| `FORM-SEC` | Sectional | `FORM-SEC-005` |
| `FORM-SON` | Sonata | `FORM-SON-003` |
| `FORM-DEV` | Theme development | `FORM-DEV-007` |
| `FORM-ABA` | A-B-A structures | `FORM-ABA-002` |

**Type:** `[HARD]` / `[SOFT]`

---

## 1. Background

Form theory governs **large-scale temporal organization**: sections, repetitions, contrasts, developmental processes, and cadential articulation of phrases. The Structure Planning and Theme Planning pipeline stages consume FORM-* rules before harmony/rhythm skeletons.

Sources: **Caplin** (*Classical Form*), **Schoenberg** (*Fundamentals of Musical Composition*), **Kostka & Payne** (small forms), pop song structure conventions.

---

## 2. Existing Solutions

| System | Approach |
|--------|----------|
| **Smaller forms catalogs** | Static templates |
| **Sonata theory (Hepokoski/Darcy)** | Analysis |
| **Pop form ML** | Section labeling from audio |
| **Aurora deep research** | Multi-theme A–B–A′ planning |

Aurora: **parametric form grammar** + theme library with similarity metrics.

---

## 3. Academic / Theoretical Foundation

### 3.1 Sectional Forms

| Form | Structure | Typical genres |
|------|-----------|----------------|
| **Binary (AB)** | ||: A :|| ||: B :|| | Baroque dance |
| **Ternary (ABA)** | A – B – A | Minuet, da capo aria |
| **Rounded binary** | A – B – A′ | Classical period |
| **Arch (ABCBA)** | Symmetrical | Pop ballad |
| **Strophic** | A – A – A | Folk, verse |

### 3.2 Sonata Form

| Section | Function | Key behavior |
|---------|----------|--------------|
| **Exposition** | Present themes | Primary (P) in tonic; Secondary (S) in dominant/relative |
| **Transition** | Connect P → S | Modulation |
| **Development** | Fragment, sequence, modulate | Instability |
| **Recapitulation** | Return themes | S in tonic |
| **Coda** | Reinforce closure | Tonic prolongation |

### 3.3 Theme and Motif Development

| Technique | Description | Rule prefix |
|-----------|-------------|-------------|
| **Repetition** | Exact recall | FORM-DEV-001 |
| **Sequence** | Transposed restatement | FORM-DEV-002 |
| **Inversion** | Interval mirror | FORM-DEV-003 |
| **Retrograde** | Reverse order | FORM-DEV-004 |
| **Augmentation** | Longer note values | FORM-DEV-005 |
| **Diminution** | Shorter note values | FORM-DEV-006 |
| **Fragmentation** | Motive segment | FORM-DEV-007 |
| **Extension** | Lengthen phrase | FORM-DEV-008 |

### 3.4 A-B-A and Pop Forms

| Pop form | Map |
|----------|-----|
| Verse–Chorus | A – B |
| Verse–Chorus–Bridge | A – B – C – B |
| 32-bar AABA | Jazz standard |

---

## 4. Engineering Analysis

Form planning is **discrete structure search** with low branching (section types × lengths). Hard constraints enforce minimum section count and cadence placement.

---

## 5. Comparison of Approaches

| Approach | Verdict |
|----------|---------|
| Fixed template only | Fast; low variety |
| Grammar + parameters | **Recommended** |
| ML section boundary detection | Import/analysis only |

---

## 6. Recommended Solution

```text
User: form_type, section_count, theme_count, total_duration
  ↓
Form Planner: instantiate section graph (FORM-SEC-*)
  ↓
Theme Planner: assign motifs/themes per section (FORM-DEV-*)
  ↓
Key/Tempo map per section → Harmony/Rhythm engines
```

---

## 7. Architecture

```text
form.md → FORM Rule Registry
              ↓
┌─────────────────────┐
│ Structure Planner   │ → Section AST nodes + Markers
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│ Theme Planner       │ → Motif assignments + dev strategy
└─────────────────────┘
```

---

## 8. Data Structures

```rust
struct FormGraph {
    sections: Vec<SectionPlan>,
    form_type: FormType,
}

struct SectionPlan {
    role: SectionRole,  // intro, verse, chorus, expo, dev, ...
    length_measures: u32,
    key_area: KeyArea,
    theme_ids: Vec<ThemeId>,
    development_level: f32,  // 0.0 exact repeat → 1.0 fragmented
    energy: f32,
}

struct ThemeRecord {
    id: ThemeId,
    motif_pitches: Vec<Pitch>,
    contour_signature: Vec<i8>,
    first_occurrence: SectionId,
}
```

---

## 9. Algorithms

### 9.1 Form Instantiation

```text
template = lookup(form_type, params)
for section in template.sections:
  adjust length to fit total_duration (FORM-SEC-010)
  assign key_area per FORM-SON or pop rules
validate FORM HARD constraints
```

### 9.2 Theme Similarity

```text
similarity(A, B) = w_contour * contour_dist + w_interval * interval_dist + w_rhythm * rhythm_dist
bridge_needed if similarity(A,B) < threshold (FORM-ABA-005)
```

### 9.3 Development Operator Selection

```text
if section.role == Development:
  ops = [sequence, fragmentation, modulation] weighted by FORM-DEV-*
apply op chain to motif from exposition
```

---

## 10. Interfaces

| API | Output |
|-----|--------|
| `plan_form(params) -> FormGraph` | Section plan |
| `assign_themes(form, theme_count) -> ThemeMap` | Theme allocation |
| `select_development(motif, level) -> Transform[]` | Dev operators |
| `validate_form(graph) -> Violation[]` | FORM-* |

---

## 11. Parameter Mappings

| Parameter | Mapping |
|-----------|---------|
| `form.section_count` | Section nodes |
| `form.section_lengths` | Measures per section |
| `theme_count` | Distinct themes |
| `theme.repetition_ratio` | FORM-DEV-001 weight |
| `motif_length` | Theme seed size |
| `emotion.tension_curve` | Development intensity |

---

## 12. Explainability Model

```json
{
  "section": "development",
  "form_rules": ["FORM-SON-004", "FORM-DEV-007"],
  "theme": "A",
  "transform": "fragmentation",
  "reason": "Raise tension curve to 0.75 at 60% composition"
}
```

---

## 13. Future Expansion

- Rondo (ABACA)
- Through-composed form
- Minimalism (phase shifting)
- Film cue form (sync points)

---

## 14. Open Questions

1. Default form for `genre=unspecified`?
2. Automatic coda generation length?

---

## 15. References

- Caplin, W. — *Analyzing Classical Form*
- Hepokoski, J. & Darcy, J. — *Elements of Sonata Theory*
- Schoenberg, A. — *Fundamentals of Musical Composition*
- Kostka & Payne — small forms chapters
- Aurora [deep-research-report.md](../../deep-research-report.md) § multi-theme
- Aurora [harmony.md](harmony.md), [rhythm.md](rhythm.md)

---

## Appendix A: Complete Rule Catalog

### General (FORM)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| FORM-001 | Total measures = sum section lengths | HARD | — |
| FORM-002 | Section boundaries on measure lines | HARD | — |
| FORM-003 | Intro length ≤ 25% total unless param | SOFT | 0.7 |
| FORM-004 | Outro/coda optional; max 15% | SOFT | 0.6 |
| FORM-005 | Climax section identifiable (max energy) | SOFT | 0.9 |
| FORM-006 | Energy curve follows tension param | SOFT | 1.0 |
| FORM-007 | No zero-length sections | HARD | — |
| FORM-008 | Form type must match genre preset or custom | SOFT | 0.8 |

### Sectional (FORM-SEC)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| FORM-SEC-001 | Verse before first chorus (pop) | SOFT | 0.9 |
| FORM-SEC-002 | Chorus ≥ verse energy | SOFT | 0.85 |
| FORM-SEC-003 | Bridge provides contrast (key or theme) | SOFT | 0.9 |
| FORM-SEC-004 | Section length multiple of phrase (4 or 8) | SOFT | 0.8 |
| FORM-SEC-005 | Max 3 consecutive same section type | SOFT | 0.7 |
| FORM-SEC-006 | Reintro after bridge before final chorus | SOFT | 0.6 |
| FORM-SEC-007 | Instrumental interlude ≥ 4 measures | SOFT | 0.5 |
| FORM-SEC-008 | Section marker on AST for each boundary | HARD | — |
| FORM-SEC-009 | Tempo change ≤ 20% between sections | SOFT | 0.7 |
| FORM-SEC-010 | Adjust lengths to fit duration target | SOFT | 1.0 |
| FORM-SEC-011 | Dual phrase period (8+8) in classical | SOFT | 0.8 |
| FORM-SEC-012 | Parallel period: cadence mid-phrase | SOFT | 0.7 |

### Sonata (FORM-SON)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| FORM-SON-001 | Exposition before development | HARD | — |
| FORM-SON-002 | Recapitulation after development | HARD | — |
| FORM-SON-003 | S theme in dominant/relative in expo | SOFT | 1.0 |
| FORM-SON-004 | S theme in tonic in recapitulation | HARD | — |
| FORM-SON-005 | Development shorter than expo+recap | SOFT | 0.7 |
| FORM-SON-006 | Modulation in development required | SOFT | 0.9 |
| FORM-SON-007 | Transition connects P and S | SOFT | 0.8 |
| FORM-SON-008 | Coda optional after recap | SOFT | 0.5 |
| FORM-SON-009 | Half cadence before S theme | SOFT | 0.7 |
| FORM-SON-010 | PAC at recap S closure | SOFT | 1.0 |

### Theme Development (FORM-DEV)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| FORM-DEV-001 | Exact repetition when repetition_ratio high | SOFT | 1.0 |
| FORM-DEV-002 | Sequence up/down diatonic or chromatic | SOFT | 0.8 |
| FORM-DEV-003 | Inversion at form dev level > 0.5 | SOFT | 0.6 |
| FORM-DEV-004 | Retrograde rare; dev level > 0.7 | SOFT | 0.4 |
| FORM-DEV-005 | Augmentation in slow sections | SOFT | 0.5 |
| FORM-DEV-006 | Diminution in development | SOFT | 0.7 |
| FORM-DEV-007 | Fragmentation in development | SOFT | 0.9 |
| FORM-DEV-008 | Extension at phrase cadence | SOFT | 0.6 |
| FORM-DEV-009 | Motif recall at climax | SOFT | 0.85 |
| FORM-DEV-010 | Transform preserves ≥ 50% pitch classes | SOFT | 0.8 |
| FORM-DEV-011 | New theme contrasts contour | SOFT | 0.9 |
| FORM-DEV-012 | Bridge theme transitional (≤ 8 bars) | SOFT | 0.7 |

### A-B-A (FORM-ABA)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| FORM-ABA-001 | A′ returns with variation not exact (classical) | SOFT | 0.8 |
| FORM-ABA-002 | B section contrasts mode or register | SOFT | 0.9 |
| FORM-ABA-003 | Final A may be abbreviated | SOFT | 0.6 |
| FORM-ABA-004 | 32-bar: A 8 + A 8 + B 8 + A 8 | SOFT | 1.0 |
| FORM-ABA-005 | Bridge if theme similarity < threshold | SOFT | 0.85 |
| FORM-ABA-006 | Da capo repeat marked on AST | SOFT | 0.5 |
| FORM-ABA-007 | B section max 50% total in ternary | SOFT | 0.7 |
| FORM-ABA-008 | A return at original key | HARD | — |

### Phrase & Cadence (FORM-PHR)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| FORM-PHR-001 | Phrase length 2–8 measures | SOFT | 0.8 |
| FORM-PHR-002 | Cadence at phrase end (cross-ref HARM-CAD) | SOFT | 1.0 |
| FORM-PHR-003 | Antecedent-consequent (question-answer) | SOFT | 0.8 |
| FORM-PHR-004 | Phrase overlap connective | SOFT | 0.5 |
| FORM-PHR-005 | Hyperphrase: 2 phrases → period | SOFT | 0.7 |

**Total FORM rules cataloged: 58**

---

## Appendix B: Form Template Library

| Template ID | Structure | Default measures |
|-------------|-----------|------------------|
| `FORM-T-01` | Intro – Verse – Chorus – Verse – Chorus – Bridge – Chorus – Outro | 32–64 |
| `FORM-T-02` | A – B – A′ | 24 |
| `FORM-T-03` | Sonata (Expo – Dev – Recap – Coda) | 80–200 |
| `FORM-T-04` | AABA (32-bar) | 32 |
| `FORM-T-05` | Through-composed sections | user-defined |

---

## Appendix C: Multi-Theme Transition (from Deep Research)

```text
Theme A ends on pitch P
  → Bridge: harmonic sequence or drum fill (FORM-SEC-007)
  → Theme B: start on P or P±2 for smooth voice leading
  → Return A′: FORM-DEV-001 with variation level from param
```

Rules: FORM-ABA-005, FORM-DEV-010, HARM-PROG-014
