# Rhythm Theory Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Theory System Research Agent (Rhythm)  
**Dependencies:** [terminology.md](../00-overview/terminology.md), [form.md](form.md), Music AST timeline (pending)

---

## Rule ID Naming Convention

| Prefix | Domain | Example |
|--------|--------|---------|
| `RHY` | Rhythm general | `RHY-001` |
| `RHY-MTR` | Meter | `RHY-MTR-003` |
| `RHY-SYNC` | Syncopation | `RHY-SYNC-005` |
| `RHY-GRO` | Groove / pattern | `RHY-GRO-012` |
| `RHY-SUB` | Subdivision | `RHY-SUB-002` |

**Type:** `[HARD]` / `[SOFT]`

---

## 1. Background

Rhythm defines **when** events occur relative to meter: time signatures, beat hierarchy, subdivisions, syncopation, pattern libraries, and groove (swing, shuffle, straight). The Rhythm Engine builds a **Rhythm Skeleton** before melody filling (pipeline stage after Harmony Skeleton).

Primary empirical source: **Groove MIDI Dataset** (Google, 13.6 hours, 1150 files, 22k+ bars) for pattern statistics. Theoretical basis: **Lerdahl & Jackendoff** metric grid, **Kostka & Payne** rhythmic vocabulary, pop/rock/electronic production conventions.

---

## 2. Existing Solutions

| System | Approach |
|--------|----------|
| **Groove MIDI / Magenta** | ML on drum patterns |
| **Euclidean rhythms** | Mathematical distribution |
| **DAW groove templates** | Style presets |
| **Music21 meter** | Analysis |
| **DrumAgent research** | Pattern generation |

Aurora: **template library + rule scoring**, optional ML plugin trained on Groove MIDI.

---

## 3. Academic / Theoretical Foundation

### 3.1 Meter and Beat Hierarchy

In 4/4:

| Level | Duration | Role |
|-------|----------|------|
| Measure | 4 beats | Hypermeter unit |
| Beat 1 | Downbeat | Strongest |
| Beat 3 | Secondary strong | Medium |
| Beats 2, 4 | Weak | Backbeat (snare) |

### 3.2 Subdivision Grid

| `subdivision` param | Grid | Min note |
|---------------------|------|----------|
| 0.2 (simple) | Quarter | 1/4 |
| 0.5 | Eighth | 1/8 |
| 0.8 | Sixteenth | 1/16 |
| 1.0 (max) | Triplets, 32nd | 1/32 |

### 3.3 Syncopation

Accent displaced from strong beat to weak subdivision. Measured by **syncopation index** (long note onset on weak portion of beat).

### 3.4 Groove

| Groove type | Characteristic |
|-------------|----------------|
| **Straight** | Even subdivisions |
| **Swing** | Long-short eighth pair (~2:1) |
| **Shuffle** | Triplet feel |
| **Latin** | Clave-aligned |

Groove MIDI provides velocity + timing micro-deviation statistics per style.

---

## 4. Engineering Analysis

Rhythm generation is **lower complexity** than harmony: finite pattern catalog + tiling across measures. Hard constraints ensure AST timeline alignment.

---

## 5. Comparison of Approaches

| Approach | Verdict |
|----------|---------|
| Pure Markov on IOI | Fast; weak meter |
| Template tiling + variation | **Recommended** |
| Groove MIDI k-NN retrieval | Optional plugin |
| L-system rhythm | Experimental |

---

## 6. Recommended Solution

```text
Phase 1: Select meter + tempo from form/style
Phase 2: Choose pattern family from library (RHY-GRO-*)
Phase 3: Tile patterns across section with fills at phrase ends
Phase 4: Apply syncopation perturbations per `syncopation` param
Phase 5: Quantize to subdivision grid; validate RHY-MTR HARD rules
```

---

## 7. Architecture

```text
rhythm.md → RHY Rule Registry
                  ↓
┌─────────────────────────────────┐
│ Pattern Library (Groove-derived)│
└───────────────┬─────────────────┘
                ↓
┌─────────────────────────────────┐
│ Rhythm Skeleton Generator       │
└───────────────┬─────────────────┘
                ↓
         AST Rest/Note onset grid
                ↓
    Melody / Drums consume skeleton
```

---

## 8. Data Structures

```rust
struct RhythmPattern {
    id: String,
    time_sig: TimeSignature,
    length_beats: Rational,
    events: Vec<RhythmEvent>,  // offset, duration, accent_weight
    style_tags: Vec<String>,
    source: Option<String>,  // "groove_midi:rock_004"
}

struct RhythmSkeleton {
    measures: Vec<MeasureRhythmPlan>,
    swing_ratio: f32,  // 0.5 = straight, 0.67 = triplet swing
    subdivision: u8,     // ticks per quarter
}
```

---

## 9. Algorithms

### 9.1 Pattern Selection

```text
candidates = library.filter(style, time_sig, density_param)
score each with RHY-GRO SOFT rules
select top; vary with RHY-GRO-020 (fill insertion)
```

### 9.2 Syncopation Injection

```text
for each event on strong beat:
  with prob(syncopation_param):
    shift onset by +subdivision/2 if RHY-SYNC rules pass
```

### 9.3 Groove MIDI Import Pipeline (offline)

```text
parse MIDI → extract drum track
quantize to grid → normalize to 1-bar pattern
tag style via metadata
store in pattern library JSON
```

---

## 10. Interfaces

| API | Description |
|-----|-------------|
| `build_rhythm_skeleton(section, params) -> RhythmSkeleton` | Main |
| `quantize_onset(t, grid) -> Rational` | Grid snap |
| `score_pattern_fit(pattern, context) -> f32` | RHY-GRO |
| `validate_meter_alignment(events) -> Violation[]` | RHY-MTR |

---

## 11. Parameter Mappings

| Parameter | Mapping |
|-----------|---------|
| `rhythm.density` | Event count per bar |
| `rhythm.syncopation` | RHY-SYNC weights |
| `rhythm.subdivision` | Grid resolution |
| `rhythm.swing` | Swing ratio |
| `style` | Pattern library filter |
| `drums.density` | Cross-ref drum engine |

---

## 12. Explainability Model

```json
{
  "pattern_id": "groove_rock_backbeat_01",
  "rules": ["RHY-GRO-003", "RHY-MTR-001"],
  "syncopation_events": [2.5, 3.5]
}
```

---

## 13. Future Expansion

- Polyrhythm (3:2) rules
- Metric modulation
- Tempo rubato curve
- Clave-specific HARD constraints for Afro-Cuban

---

## 14. Open Questions

1. Share pattern library with Drum Engine or separate?
2. Micro-timing from Groove MIDI: apply in export or playback only?

---

## 15. References

- Lerdahl, F. & Jackendoff, R. — *A Generative Theory of Tonal Music* (grouping/meter)
- Kostka & Payne — rhythmic examples
- Gillick, J. et al. — *Learning to Groove* (Groove MIDI paper)
- Groove MIDI Dataset (Google Magenta)
- Aurora [deep-research-report.md](../../deep-research-report.md)
- Aurora [form.md](form.md)

---

## Appendix A: Pattern Library Taxonomy

| Family | ID prefix | Typical use |
|--------|-----------|-------------|
| Rock backbeat | `RHY-GRO-RK` | Snare 2+4 |
| Pop straight | `RHY-GRO-POP` | Four-on-floor variant |
| Jazz swing ride | `RHY-GRO-JZ` | Triplet feel |
| Bossa | `RHY-GRO-BO` | Syncopated bass |
| Waltz | `RHY-GRO-WZ` | 3/4 strong 1 |
| Electronic | `RHY-GRO-ED` | 16th hi-hats |
| Ballad | `RHY-GRO-BL` | Sparse |
| Fill / turnaround | `RHY-GRO-FL` | Phrase end |

Target: ≥ 50 patterns per family from Groove MIDI clustering (offline research task).

---

## Appendix B: Complete Rule Catalog

### Meter (RHY-MTR)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| RHY-MTR-001 | All event onsets align to subdivision grid | HARD | — |
| RHY-MTR-002 | Measure duration = time_sig × beat unit | HARD | — |
| RHY-MTR-003 | Downbeat (beat 1) has accent weight ≥ 0.8 | SOFT | 1.0 |
| RHY-MTR-004 | Beat 3 secondary accent in 4/4 | SOFT | 0.7 |
| RHY-MTR-005 | Anacrusis duration < full measure | HARD | — |
| RHY-MTR-006 | Pickup aligns to phrase start | SOFT | 0.8 |
| RHY-MTR-007 | Tempo map monotonic unless rit/accel marked | HARD | — |
| RHY-MTR-008 | Time signature change only at section boundary | SOFT | 0.9 |
| RHY-MTR-009 | Hypermeter: 4-bar phrase accent on m1 | SOFT | 0.6 |
| RHY-MTR-010 | Irregular meter: beat strength from pattern table | SOFT | 0.7 |

### Subdivision (RHY-SUB)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| RHY-SUB-001 | Note duration ≥ grid unit | HARD | — |
| RHY-SUB-002 | Duration set ⊆ allowed for subdivision level | HARD | — |
| RHY-SUB-003 | Tuplets require grouping marker | SOFT | 0.5 |
| RHY-SUB-004 | Prefer binary subdivisions when param low | SOFT | 0.8 |
| RHY-SUB-005 | 16th density max per `density` param | SOFT | 1.0 |
| RHY-SUB-006 | Tie across barline preserves beat class | SOFT | 0.7 |
| RHY-SUB-007 | Dotted rhythm clarity (dot = half again) | HARD | — |
| RHY-SUB-008 | Hemiola only in 3/4 or 6/8 with flag | SOFT | 0.6 |

### Syncopation (RHY-SYNC)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| RHY-SYNC-001 | Syncopation index ≈ `syncopation` param | SOFT | 1.0 |
| RHY-SYNC-002 | Offbeat onset followed by onbeat resolution | SOFT | 0.8 |
| RHY-SYNC-003 | Avoid syncopation on every beat (monotony) | SOFT | 0.7 |
| RHY-SYNC-004 | Phrase end: reduce syncopation | SOFT | 0.9 |
| RHY-SYNC-005 | Anticipation: early by subdivision | SOFT | 0.6 |
| RHY-SYNC-006 | Suspended rhythm over barline | SOFT | 0.7 |
| RHY-SYNC-007 | Do not syncopate downbeat of final measure | SOFT | 1.1 |
| RHY-SYNC-008 | Cross-rhythm 3:2 requires 2-measure min span | SOFT | 0.5 |

### Groove / Patterns (RHY-GRO)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| RHY-GRO-001 | Pattern must match current time signature | HARD | — |
| RHY-GRO-002 | Pattern density within param bounds | SOFT | 1.0 |
| RHY-GRO-003 | Backbeat on 2+4 for rock/pop when tagged | SOFT | 0.9 |
| RHY-GRO-004 | Swing ratio from param | SOFT | 1.0 |
| RHY-GRO-005 | Velocity accent curve from pattern template | SOFT | 0.6 |
| RHY-GRO-006 | Variation: max 25% event flip per repeat | SOFT | 0.7 |
| RHY-GRO-007 | Consecutive identical bars max 4 unless chorus | SOFT | 0.8 |
| RHY-GRO-008 | Fill at phrase boundary (last 1-2 beats) | SOFT | 0.9 |
| RHY-GRO-009 | Intro sparse: density < verse | SOFT | 0.7 |
| RHY-GRO-010 | Chorus density ≥ verse | SOFT | 0.8 |
| RHY-GRO-011 | Bridge: pattern family change | SOFT | 0.7 |
| RHY-GRO-012 | Groove MIDI attribution in provenance | — | — |
| RHY-GRO-013 | Hi-hat 8ths/16ths per electronic preset | SOFT | 0.8 |
| RHY-GRO-014 | Latin: clave alignment HARD when preset | HARD | — |
| RHY-GRO-015 | Waltz: bass on 1, chord on 2-3 | SOFT | 0.9 |
| RHY-GRO-016 | Reggae: emphasis on 3 | SOFT | 0.8 |
| RHY-GRO-017 | Half-time: snare on 3 only | SOFT | 0.7 |
| RHY-GRO-018 | Double-time: subdivision double | SOFT | 0.6 |
| RHY-GRO-019 | Breakdown: remove pattern layer | SOFT | 0.5 |
| RHY-GRO-020 | Turnaround pattern last 2 bars of section | SOFT | 0.85 |

### General (RHY)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| RHY-001 | Rests allowed; max consecutive rest from param | SOFT | 0.6 |
| RHY-002 | Melody rhythm independent of harmonic rhythm | SOFT | 0.5 |
| RHY-003 | Phrase rhythm matches lyric slot if vocal | SOFT | 0.7 |
| RHY-004 | Accelerando not automatic unless param | SOFT | 0.4 |
| RHY-005 | Ritard at cadence optional | SOFT | 0.6 |
| RHY-006 | Drum rhythm authoritative for groove (cross-ref) | — | — |
| RHY-007 | Polymeter: secondary voice tagged | SOFT | 0.5 |
| RHY-008 | Minimum note duration for readability export | SOFT | 0.5 |

**Total RHY rules cataloged: 54**

---

## Appendix C: Groove MIDI Integration Notes

| Statistic | Use in Aurora |
|-----------|---------------|
| Onset histogram per style | RHY-GRO pattern priors |
| Velocity mean/variance | Accent rules RHY-GRO-005 |
| Timing deviation (ms) | Playback humanization (not HARD rules) |
| Bar length distribution | Validate 4/4 dominant |

Dataset: 1150 files, ~22k bars — sufficient for template extraction; not for end-to-end ML melody.

---

## Appendix D: Example Pattern (Rock Backbeat)

```text
Beat:     1    2    3    4
Kick:     X    .    .    .
Snare:    .    X    .    X
Hat:      x    x    x    x   (8ths, accent on downbeats)
```

Pattern ID: `RHY-GRO-RK-001`; rules: RHY-GRO-003, RHY-MTR-003, RHY-MTR-004
