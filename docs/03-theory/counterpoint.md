# Counterpoint Theory Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Theory System Research Agent (Counterpoint)  
**Dependencies:** [terminology.md](../00-overview/terminology.md), [harmony.md](harmony.md), [voice-leading.md](voice-leading.md), Music AST (pending)

---

## Rule ID Naming Convention

| Prefix | Domain | Example |
|--------|--------|---------|
| `CP` | Counterpoint (general) | `CP-001` |
| `CP-S1` | First species | `CP-S1-005` |
| `CP-S2` | Second species | `CP-S2-003` |
| `CP-S3` | Third species | `CP-S3-007` |
| `CP-S4` | Fourth species | `CP-S4-002` |
| `CP-S5` | Fifth species (florid) | `CP-S5-004` |
| `CP-PAR` | Parallel interval rules | `CP-PAR-001` |

**Type:** `[HARD]` = prune; `[SOFT]` = penalty/reward. Parameter `counterpoint.strictness` (0.0–1.0) maps soft weights to near-hard thresholds at 1.0.

---

## 1. Background

Counterpoint governs **simultaneous independent melodic lines** in Aurora Composer. It applies during Counterpoint pipeline stage (after primary melody, before bass/drums) and in chorale/SATB presets where all voices are co-generated.

Primary sources: **Fux, *Gradus ad Parnassum*** (species method), **Hindemith** (two-voice writing), **Piston** (independent lines in harmonic context). Aurora separates **counterpoint** (horizontal independence + interval constraints) from **voice-leading** ([voice-leading.md](voice-leading.md), vertical motion between chords).

**Scope:** 2–4 voice counterpoint, species rules as pedagogical baseline, florid (5th species) as performance target, parallel interval prohibition, melodic independence metrics. **Out of scope:** 16th-century strict modal counterpoint without tonal harmony (future early-music plugin).

---

## 2. Existing Solutions

| System | Approach | Notes |
|--------|----------|-------|
| **Fux / automated tutors** | Species rule checkers | Good for education; limited multi-voice |
| **OpenMusic constraints** | User-defined interval constraints | Flexible; no standard rule IDs |
| **Morris / Maxwell** | CSP for species | Research prototypes |
| **Music21 `intervalStream`** | Post-hoc analysis | Not generative |
| **Bach chorale ML** | Neural harmonization | Black-box |

Aurora needs **generative** counterpoint with explainable CP-* rule IDs.

---

## 3. Academic / Theoretical Foundation

### 3.1 Species Counterpoint Overview (Fux)

| Species | Cantus firmus | Counterpoint voice | Focus |
|---------|---------------|-------------------|-------|
| **1st** | Whole notes | Whole notes | Consonance at each vertical moment |
| **2nd** | Whole notes | 2 notes per CF note | Strong beat consonance |
| **3rd** | Whole notes | 4 notes per CF note | Melodic shape, passing tones |
| **4th** | Whole notes | Syncopated (suspensions) | Dissonance treatment |
| **5th** | Whole notes | Mixed (florid) | Combined techniques |

### 3.2 Consonance Classification

**Perfect consonances (P1, P5, P8):** stable; P1 only between unison/octave doubling in strict 2-voice; P5/P8 start/end allowed with restrictions.

**Imperfect consonances (m3, M3, m6, M6):** preferred vertical intervals for 2-voice writing.

**Dissonances (P4*, m2, M2, m7, M7, tritone):** *P4 dissonant above bass in strict 2-voice; allowed as passing/neighbor in species 3+.

### 3.3 Motion Types (Between Voices)

| Motion | Definition | Preference |
|--------|------------|------------|
| **Contrary** | Voices move opposite directions | Strongly preferred |
| **Oblique** | One voice static | Neutral |
| **Similar** | Same direction | Allowed with caution |
| **Parallel** | Same direction, same interval | Restricted for P5/P8/P1 |

### 3.4 Parallel Interval Avoidance

**Forbidden (strict):** Parallel perfect unisons, fifths, octaves (P1, P5, P8).

**Hidden (direct):** Approached by similar motion into P5/P8 — penalized in strict mode (CP-PAR-010).

**Complementary:** Contrary motion into perfect interval — preferred.

### 3.5 Melodic Independence

Independent lines should exhibit:

- Distinct rhythmic profiles (CP-050+)
- Different contour (not constant parallel thirds)
- Unequal leap distribution
- Avoid voice shadowing (CP-055)

---

## 4. Engineering Analysis

| Criterion | Assessment |
|-----------|------------|
| Correctness | Species + Bach chorale norms at strictness ≥ 0.9 |
| Controllability | `counterpoint.strictness`, `voice_count`, species preset |
| Explainability | Per-interval provenance on simultaneous events |
| Performance | O(voices² × events) interval checking; cache last vertical |
| Extensibility | Early music plugin relaxes tonal consonance |
| Complexity | 62 rules; implement 2-voice first, extend to 4 |

---

## 5. Comparison of Approaches

| Approach | Verdict |
|----------|---------|
| Generate voices sequentially with interval check | Simple; order bias |
| Simultaneous CSP on all voices | **Recommended** for chorale |
| Species-scaffold then florid fill | **Recommended** for pedagogical mode |
| Ignore species, only VL rules | Insufficient for polyphonic texture |

---

## 6. Recommended Solution

### 6.1 Mode Matrix

| Preset | Species base | Parallel P5/P8 | Dissonance on weak beat |
|--------|--------------|----------------|-------------------------|
| `chorale_strict` | 5th species | HARD | Prepared only |
| `classical_homophony` | Relaxed | SOFT (high penalty) | Passing allowed |
| `pop_polyphony` | Minimal | SOFT | Free |
| `counterpoint_study` | User selects 1–5 | Per species | Per species |

### 6.2 Generation Flow

```text
Cantus firmus (existing melody or bass) → fixed
For each counterpoint voice (alto, tenor):
  beam search note choices at subdivision grid
  apply CP-* HARD → prune
  apply CP-* SOFT → score
  cross-check VL-* with voice-leading module
```

---

## 7. Architecture

```text
counterpoint.md (CP-* rules)
       ↓
Counterpoint Rule Registry
       ↓
┌─────────────────────┐
│ Species Profile     │ ← user preset
└──────────┬──────────┘
           ↓
┌─────────────────────┐     ┌──────────────────┐
│ Voice Generator     │────►│ Interval Checker │
│ (per voice)         │     │ (pairwise)       │
└─────────────────────┘     └──────────────────┘
           ↓
    AST Voice events + CP provenance
```

---

## 8. Data Structures

### 8.1 VerticalSnapshot

```rust
struct VerticalSnapshot {
    beat: Rational,
    pitches: Vec<(VoiceId, Pitch)>,
    intervals: Vec<(VoicePair, Interval)>,
}
```

### 8.2 MotionRecord

```rust
struct MotionRecord {
    voice_pair: (VoiceId, VoiceId),
    motion_type: MotionType,  // Contrary, Oblique, Similar, Parallel
    interval_from: Interval,
    interval_to: Interval,
}
```

### 8.3 SpeciesConfig

```rust
struct SpeciesConfig {
    species_level: u8,  // 1–5, 0 = florid auto
    cantus_voice: VoiceId,
    notes_per_cf_note: u8,
    dissonance_beat_mask: BitMask,
}
```

---

## 9. Algorithms

### 9.1 Parallel Fifth/Octave Detector

```text
for each adjacent beat pair (t, t+1):
  for each voice pair (i, j):
    if motion(i) parallel motion(j):
      if interval(t) in {P1,P5,P8} and interval(t+1) in {P1,P5,P8}:
        if not allowed_exception(species, context):
          report CP-PAR-001 HARD violation
```

### 9.2 First Species Candidate Generation

```text
for each CF note at beat b:
  candidates = scale_degrees_within_range(voice_range)
  filter: vertical interval with CF is consonant (CP-S1-002 HARD)
  filter: no parallel P5/P8 with any other voice
  score: contrary motion preferred (CP-020 SOFT)
```

### 9.3 Florid (5th Species) Composition

Combine species 2–4 rules with weighted relaxation:

- Strong beat: consonance (HARD at strictness > 0.8)
- Weak beat: passing/neighbor dissonance (SOFT preparation check)

---

## 10. Interfaces

| API | Description |
|-----|-------------|
| `check_vertical_intervals(snapshot) -> Violation[]` | All CP-PAR and CP-S* |
| `score_melodic_independence(voices) -> f32` | CP-050–058 |
| `generate_counterpoint_voice(cf, voice_id, config) -> Voice AST` | Main generator |
| `species_preset(level) -> SpeciesConfig` | 1–5 |

---

## 11. Parameter Mappings

| Parameter | Mapping |
|-----------|---------|
| `counterpoint.strictness` | Scales all CP SOFT penalties; at 1.0, CP-PAR-001 → HARD |
| `voice_count` | Enables alto/tenor generation |
| `texture` polyphony weight | Activates counterpoint stage intensity |
| `complexity` | Allows florid subdivisions (species 3+) |
| `rhythm.subdivision` | Grid for counterpoint note placement |

---

## 12. Explainability Model

Simultaneous notes at beat *b* share vertical provenance:

```json
{
  "vertical_check": {
    "beat": "2.0",
    "intervals": [
      { "pair": ["soprano", "alto"], "interval": "M6", "motion": "contrary", "rules": ["CP-020"] }
    ],
    "violations": []
  }
}
```

Parallel violation example:

```json
{
  "violation": "CP-PAR-001",
  "voices": ["tenor", "bass"],
  "measure": 3,
  "beat": "1.0",
  "detail": "Parallel P5 G–D to A–E"
}
```

---

## 13. Future Expansion

- Imitation and canon rules (CP-070+ reserved)
- Invertible counterpoint at octave
- Palestrina style (triadic only, no harmonic minor)
- Imitation distance metrics for fugue plugin

---

## 14. Open Questions

1. Is P4 consonant above bass in 3+ voice texture when not lowest pair?
2. Order of voice generation: alto before tenor or simultaneous?
3. Interaction with jazz voicing (locked guide tones) — skip CP or relax?

---

## 15. References

- Fux, J.J. *Gradus ad Parnassum* (Mann/Kerman translation)
- Hindemith, P. *Elementary Training for Musicians*; *Traditional Harmony*
- Piston, W. *Counterpoint*
- Jeppesen, K. *The Style of Palestrina and the Dissonance*
- Bach 371 Chorales — empirical validation
- Salzer, F. & Schachter, C. *Counterpoint in Composition*
- Aurora [voice-leading.md](voice-leading.md), [harmony.md](harmony.md)

---

## Appendix A: Species Rules Detail

### First Species (CP-S1)

| ID | Rule | Type |
|----|------|------|
| CP-S1-001 | One note in counterpoint voice per cantus note | HARD |
| CP-S1-002 | All vertical intervals consonant | HARD |
| CP-S1-003 | Begin on P1/P8/P5 (above CF) | HARD |
| CP-S1-004 | End on P1/P8 | HARD |
| CP-S1-005 | Penultimate bar: leading tone in counterpoint if CF has 2–1 | SOFT |
| CP-S1-006 | Avoid consecutive leaps in same direction > P4 | SOFT |
| CP-S1-007 | Max leap P8 in counterpoint voice | SOFT |
| CP-S1-008 | Prefer contrary motion to CF | SOFT |

### Second Species (CP-S2)

| ID | Rule | Type |
|----|------|------|
| CP-S2-001 | Two counterpoint notes per one CF note | HARD |
| CP-S2-002 | First beat consonant | HARD |
| CP-S2-003 | Second beat may be dissonant if passing | SOFT |
| CP-S2-004 | Passing tone must step between consonances | HARD |
| CP-S2-005 | Do not accent dissonance on beat 1 | HARD |

### Third Species (CP-S3)

| ID | Rule | Type |
|----|------|------|
| CP-S3-001 | Four notes per CF note | HARD |
| CP-S3-002 | Beat 1 consonant | HARD |
| CP-S3-003 | Passing tones on weak divisions | SOFT |
| CP-S3-004 | Neighbor tones return by step | HARD |
| CP-S3-005 | Avoid three consecutive same direction leaps | SOFT |
| CP-S3-006 | Cambiata figure allowed (5-4-3-2) | SOFT |

### Fourth Species (CP-S4)

| ID | Rule | Type |
|----|------|------|
| CP-S4-001 | Syncopation: tie dissonance over bar | HARD |
| CP-S4-002 | Suspension prepares by common tone | HARD |
| CP-S4-003 | Resolve suspension down by step | HARD |
| CP-S4-004 | Consonant skip after resolution | SOFT |
| CP-S4-005 | 4-3 suspension most common | SOFT |

### Fifth Species / Florid (CP-S5)

| ID | Rule | Type |
|----|------|------|
| CP-S5-001 | Mix species 2–4 techniques | SOFT |
| CP-S5-002 | Strong beats consonant (strict) | HARD* |
| CP-S5-003 | Idiomatic melodic curve ( climax once ) | SOFT |
| CP-S5-004 | Do not obscure CF rhythm | SOFT |

---

## Appendix B: Parallel and Hidden Interval Rules (CP-PAR)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| CP-PAR-001 | No parallel P5 | HARD* | — |
| CP-PAR-002 | No parallel P8 | HARD* | — |
| CP-PAR-003 | No parallel P1 (except doubling policy) | HARD | — |
| CP-PAR-004 | No parallel P4 above bass in 2-voice | HARD | — |
| CP-PAR-005 | No parallel tritone | SOFT | 1.5 |
| CP-PAR-006 | No parallel M3/m3 (strict Palestrina) | SOFT | 0.5 |
| CP-PAR-007 | Similar motion into P5: check inner voice | SOFT | 1.2 |
| CP-PAR-008 | Hidden P5 in outer voices penalized | SOFT | 1.0 |
| CP-PAR-009 | Hidden P8 in outer voices penalized | SOFT | 1.0 |
| CP-PAR-010 | Direct P5/P8 approached by similar motion in soprano+bass | SOFT | 1.3 |

*SOFT when `counterpoint.strictness` < 0.7; HARD otherwise.

---

## Appendix C: Independence and Range (CP-050+)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| CP-050 | Voices should not shadow within M2 for > 2 beats | SOFT | 1.0 |
| CP-051 | Rhythmic differentiation between voices | SOFT | 0.8 |
| CP-052 | Similar contour in all voices penalized | SOFT | 0.9 |
| CP-053 | Imitation at octave rewarded (if enabled) | SOFT | 0.7 |
| CP-054 | Voice crossing duration ≤ 1 beat unless specified | SOFT | 0.8 |
| CP-055 | Spacing between adjacent voices ≥ M3 | SOFT | 0.6 |
| CP-056 | Cantus firmus should not be doubled exactly | HARD | — |
| CP-057 | Maximum simultaneous rests: one voice | SOFT | 0.5 |
| CP-058 | Total pitch range per voice within register param | HARD | — |

---

## Appendix D: Complete Rule Index

**Total CP rules cataloged: 62**

| Range | Count | Category |
|-------|-------|----------|
| CP-S1-001 – CP-S1-008 | 8 | First species |
| CP-S2-001 – CP-S2-005 | 5 | Second species |
| CP-S3-001 – CP-S3-006 | 6 | Third species |
| CP-S4-001 – CP-S4-005 | 5 | Fourth species |
| CP-S5-001 – CP-S5-004 | 4 | Fifth species |
| CP-PAR-001 – CP-PAR-010 | 10 | Parallel/hidden |
| CP-020 – CP-028 | 9 | General motion preference |
| CP-050 – CP-058 | 9 | Independence |
| CP-030 – CP-038 | 6 | Voice count / texture |

### General Motion (CP-020–038)

| ID | Rule | Type |
|----|------|------|
| CP-020 | Prefer contrary motion between outer voices | SOFT |
| CP-021 | Oblique motion acceptable on common tone | SOFT |
| CP-022 | Similar motion into imperfect consonance OK | SOFT |
| CP-023 | Avoid all voices moving same direction | SOFT |
| CP-024 | Bass leaps > P4 penalized | SOFT |
| CP-025 | Upper voice leap after leap compensates | SOFT |
| CP-026 | No voice repeats same pitch > 4 beats | SOFT |
| CP-027 | Melodic climax not simultaneous in all voices | SOFT |
| CP-028 | Cross relation avoided between voices | SOFT |
| CP-030 | 2-voice: max interval span P15 | HARD |
| CP-031 | 3-voice: inner voices fill harmonic gap | SOFT |
| CP-032 | 4-voice: complete chord on strong beats | SOFT |
| CP-033 | Avoid empty voice (all rests) > 2 measures | SOFT |
| CP-034 | Counterpoint stage skips if texture homophonic | — |
| CP-035 | CF may be melody or bass per preset | — |
| CP-036 | Imitation delay 1–2 beats optional | SOFT |
| CP-037 | Inversion imitation optional plugin | — |
| CP-038 | Canon at interval requires exact transposition | HARD |

---

## Appendix E: Validation Corpus

Use **Bach 371 Chorales** for regression:

| Metric | Target at strictness 1.0 |
|--------|--------------------------|
| Parallel P5/P8 rate | 0% in generated chorales |
| Consonance on downbeat | > 95% |
| Resolution of 7ths | > 90% |

Music21 `parallel.fifths` analyzer as external benchmark.
