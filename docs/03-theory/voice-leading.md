# Voice Leading Theory Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Theory System Research Agent (Voice Leading)  
**Dependencies:** [harmony.md](harmony.md), [counterpoint.md](counterpoint.md), [terminology.md](../00-overview/terminology.md)

---

## Rule ID Naming Convention

| Prefix | Domain | Example |
|--------|--------|---------|
| `VL` | Voice leading | `VL-001` |
| `VL-CT` | Common tone | `VL-CT-003` |
| `VL-MOT` | Motion preference | `VL-MOT-007` |
| `VL-X` | Voice crossing | `VL-X-002` |
| `VL-RNG` | Range rules | `VL-RNG-005` |

**Type:** `[HARD]` / `[SOFT]` as in harmony.md.

---

## 1. Background

Voice leading is the **horizontal logic connecting consecutive sonorities**: how individual voices move from chord to chord. Aurora treats voice leading as distinct from species counterpoint: counterpoint governs simultaneous interval laws; voice leading governs **successive** pitch motion, tendency tones, doublings, and register.

Sources: **Piston**, **Aldwell & Schachter**, **Kostka & Payne**, Bach chorale practice. Applied in Harmony voicing, Counterpoint, Melody, Bass, and Repair stages.

---

## 2. Existing Solutions

| Tool | Capability |
|------|------------|
| Music21 voice leading reduction | Analysis |
| Tonal theory textbooks (automated exercises) | Manual |
| SAT solvers in OpenMusic | Custom constraints |
| Spreadsheets for chorale grading | Educational |

Aurora unifies VL rules for all vertical-to-vertical transitions on AST `Voice` events.

---

## 3. Academic / Theoretical Foundation

### 3.1 Four Principles (Aldwell / Schachter)

1. **Independence of lines** — each voice has own contour (cross-ref CP-050+)
2. **Covered or uncovered** — preference for complete sonorities
3. **Minimal motion** — smallest total voice movement (parsimony)
4. **Proper treatment of tendency tones** — 7ths down, leading tones up, chordal 7ths resolve

### 3.2 Common Tones

When two consecutive chords share pitch class(s), **retain common tone in same voice** when possible (VL-CT-001). Reduces parallel risk and improves continuity.

### 3.3 Stepwise Motion Preference

| Motion | Preference |
|--------|------------|
| Step (M2/m2) | Highest |
| Third (skip) | Good |
| Fourth/fifth leap | Moderate; context-dependent |
| Sixth+ leap | Penalized; often requires compensation |

### 3.4 Voice Crossing and Overlap

| Term | Definition | Policy |
|------|------------|--------|
| **Crossing** | Upper voice goes below lower neighbor voice | SOFT penalty; brief OK in piano |
| **Overlap (voice exchange)** | Upper voice leaps below previous note of lower voice | HARD in SATB |
| **Spacing collapse** | Voices within m2 | HARD in orchestral |

### 3.5 Range Rules

Standard SATB and instrument ranges (see [orchestration.md](orchestration.md)). User parameters `register_min/max` per voice override defaults.

---

## 4. Engineering Analysis

Voice leading evaluation runs **O(voices × chord_changes)** per phrase. Cached last pitch per voice enables incremental checking during beam search.

---

## 5. Comparison of Approaches

| Approach | Verdict |
|----------|---------|
| Global voice-leading optimization (Hungarian) | Optimal but expensive |
| Greedy per-chord with backtrack | **Recommended** |
| Post-hoc repair only | Insufficient; use Repair as fallback |

---

## 6. Recommended Solution

Evaluate VL rules on every `(chord_n, chord_n+1)` transition:

```text
for each voice v:
  compute motion type (step, leap, common tone)
  apply VL-MOT-*, VL-CT-*, VL-RNG-*
for each voice pair (i,j):
  check parallel (delegate CP-PAR-*)
  check crossing VL-X-*
aggregate eval_score
```

---

## 7. Architecture

```text
voice-leading.md → VL Rule Registry
                         ↓
              ┌──────────────────────┐
              │ TransitionEvaluator  │
              └──────────┬───────────┘
                         ↓
    Harmony Voicing / Counterpoint / Repair
```

Shared with Counterpoint: parallel detection unified in one module, dual-tagged CP-PAR + VL-PAR.

---

## 8. Data Structures

```rust
struct VoiceTransition {
    voice: VoiceId,
    pitch_from: Pitch,
    pitch_to: Pitch,
    motion_semitones: i8,
    is_common_tone: bool,
    is_tendency_tone: bool,
    resolves_correctly: bool,
}

struct VerticalTransition {
    from_chord: ChordSymbol,
    to_chord: ChordSymbol,
    voices: Vec<VoiceTransition>,
}
```

---

## 9. Algorithms

### 9.1 Common Tone Assignment

```text
shared_pcs = intersection(from_chord.tones, to_chord.tones)
for pc in shared_pcs:
  assign to voice that held pc if available (VL-CT-001)
  else assign to minimize total motion (Hungarian lite)
```

### 9.2 Tendency Tone Resolution Check

```text
if pitch_from is chord_7th of from_chord:
  require pitch_to == pitch_from - 2 semitones (mod 12) (VL-MOT-010 HARD*)
if pitch_from is leading_tone:
  require pitch_to == tonic (VL-MOT-011 HARD*)
```

---

## 10. Interfaces

| API | Returns |
|-----|---------|
| `evaluate_transition(prev, next) -> (score, violations)` | Per chord pair |
| `assign_voices_greedy(from, to, ranges) -> Voicing` | Pitch assignment |
| `check_range(voice, pitch) -> bool` | VL-RNG |

---

## 11. Parameter Mappings

| Parameter | Effect |
|-----------|--------|
| `counterpoint.strictness` | VL tendency rules HARD threshold |
| `melody_register`, `bass_register` | VL-RNG bounds |
| `voice_count` | Active voices |
| `complexity` | Allow parallel thirds/sixths in pop |

---

## 12. Explainability Model

```json
{
  "transition": "m3:IV → m3:I",
  "voice": "tenor",
  "motion": "common_tone",
  "rules": ["VL-CT-001", "VL-MOT-001"]
}
```

---

## 13. Future Expansion

- Neo-Riemannian voice leading (PLR transforms)
- Jazz guide-tone lines (cross-ref JAZZ-VL-*)
- Sliding voice leading in planing chords

---

## 14. Open Questions

1. Unify CP-PAR and VL parallel checks under single ID namespace?
2. Allow unresolved 7th in dominant pedal?

---

## 15. References

- Aldwell, Schachter, Cadwallader — *Harmony and Voice Leading*
- Piston — *Harmony*
- Kostka & Payne — *Tonal Harmony*
- Bach 371 Chorales
- Aurora [counterpoint.md](counterpoint.md), [harmony.md](harmony.md)

---

## Appendix A: Complete Rule Catalog

### Common Tone (VL-CT)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| VL-CT-001 | Retain common tone in same voice | SOFT | 1.2 |
| VL-CT-002 | Common tone on strong beat held ≥ beat duration | SOFT | 0.8 |
| VL-CT-003 | If no common tone, prefer stepwise in one voice | SOFT | 1.0 |
| VL-CT-004 | Avoid swapping voices' pitch classes unnecessarily | SOFT | 0.9 |
| VL-CT-005 | Pedal tone: bass static ≥ 2 chords | SOFT | 0.6 |

### Motion (VL-MOT)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| VL-MOT-001 | Prefer stepwise motion in inner voices | SOFT | 1.1 |
| VL-MOT-002 | Soprano may leap more freely than inner | SOFT | 0.5 |
| VL-MOT-003 | Bass prefers step or fourth/fifth | SOFT | 0.9 |
| VL-MOT-004 | Total voice motion minimized (parsimony) | SOFT | 0.8 |
| VL-MOT-005 | Leap > P4 in inner voice: next motion step opposite | SOFT | 1.0 |
| VL-MOT-006 | Two consecutive leaps same direction penalized | SOFT | 1.1 |
| VL-MOT-007 | Augmented interval in melody avoided | SOFT | 1.3 |
| VL-MOT-008 | Chromatic motion in one voice: others diatonic | SOFT | 0.7 |
| VL-MOT-009 | Doubled note resolves simultaneously | SOFT | 0.9 |
| VL-MOT-010 | Chord 7th resolves down by step | HARD* | — |
| VL-MOT-011 | Leading tone resolves up to tonic | HARD* | — |
| VL-MOT-012 | #4 resolves up to 5 (when in V/V context) | SOFT | 1.0 |
| VL-MOT-013 | Avoid augmented 2nd in minor melody | SOFT | 1.2 |
| VL-MOT-014 | Passing tone fills step between chord tones | SOFT | 0.6 |
| VL-MOT-015 | Neighbor tone returns to origin | SOFT | 0.7 |

### Voice Crossing (VL-X)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| VL-X-001 | No voice crossing between soprano and alto | SOFT | 1.0 |
| VL-X-002 | No voice crossing between alto and tenor | SOFT | 1.0 |
| VL-X-003 | Bass may cross tenor briefly in piano texture | SOFT | 0.3 |
| VL-X-004 | Voice overlap (exchange) forbidden in SATB | HARD | — |
| VL-X-005 | Crossing duration ≤ 1 beat unless `texture=piano` | SOFT | 0.8 |
| VL-X-006 | Maintain voice ordering at chord change | SOFT | 0.9 |

### Range (VL-RNG)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| VL-RNG-001 | Soprano within user/register default | HARD | — |
| VL-RNG-002 | Bass within user/register default | HARD | — |
| VL-RNG-003 | Inner voices within combined range | HARD | — |
| VL-RNG-004 | Avoid soprano below C4 in choral preset | SOFT | 0.9 |
| VL-RNG-005 | Avoid bass above G3 in choral preset | SOFT | 0.9 |
| VL-RNG-006 | Melody climax in upper 60% of range | SOFT | 0.6 |
| VL-RNG-007 | Tessitura: 70% notes in comfortable band | SOFT | 0.5 |
| VL-RNG-008 | Extreme register requires dynamic adjustment | SOFT | 0.4 |

### Doubling and Spacing (VL-DBL)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| VL-DBL-001 | Double primary chord tone (root or bass note) | SOFT | 0.7 |
| VL-DBL-002 | Never double leading tone | HARD | — |
| VL-DBL-003 | Never double chord 7th | HARD | — |
| VL-DBL-004 | Avoid double chromatic pitch | HARD | — |
| VL-DBL-005 | Spacing soprano-alto ≤ P8 | SOFT | 0.8 |
| VL-DBL-006 | Spacing alto-tenor ≤ P8 | SOFT | 0.8 |
| VL-DBL-007 | Spacing tenor-bass flexible to P12 | SOFT | 0.5 |
| VL-DBL-008 | Open spacing (soprano-bass > P8) rewarded in orchestral | SOFT | 0.4 |

**Total VL rules cataloged: 47**

---

## Appendix B: Worked Transition

**IV → V in C major, SATB:**

| Voice | From | To | Rules |
|-------|------|-----|-------|
| S | F | G | VL-MOT-002 |
| A | C | B | VL-MOT-011 (leading tone) |
| T | F | G | VL-CT-001 (common G) |
| B | F | G | VL-MOT-003 |

No CP-PAR violation; contrary motion outer voices (CP-020).
