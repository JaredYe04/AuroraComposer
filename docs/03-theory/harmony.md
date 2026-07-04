# Harmony Theory Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Theory System Research Agent (Harmony)  
**Dependencies:** [terminology.md](../00-overview/terminology.md), [acas-v0.1.md](../00-overview/acas-v0.1.md), [research-methodology.md](../00-overview/research-methodology.md), Music AST (pending `docs/02-music-model/ast.md`)

---

## Rule ID Naming Convention

| Prefix | Domain | Example |
|--------|--------|---------|
| `HARM` | Harmony (this document) | `HARM-001` |
| `HARM-VOICE` | Voicing sub-rules | `HARM-VOICE-012` |
| `HARM-CAD` | Cadence sub-rules | `HARM-CAD-003` |
| `HARM-PROG` | Progression sub-rules | `HARM-PROG-015` |

**Type suffix in catalog:** `[HARD]` = constraint (search prunes); `[SOFT]` = scoring rule (penalty/reward).

Sequential numbering within prefix; gaps reserved for plugin extensions.

---

## 1. Background

Harmony is the vertical dimension of Aurora Composer: chord vocabulary, functional relationships, progressions, harmonic rhythm, voicing, and cadences. The Harmony Theory module supplies **normative rules** consumed by the Harmony Engine (`docs/04-algorithms/harmony-engine.md`), Rule Engine, and Repair stage.

Aurora Composer targets **explainable, rule-driven** generation. Every chord choice must trace to a rule ID and parameter binding. This specification formalizes common-practice tonal harmony (Piston, Kostka & Payne), Hindemith's chord classification for voice-leading context, and an overview of jazz extensions (detailed jazz rules in [jazz.md](jazz.md)).

**Scope:** Major/minor tonal systems, modal interchange basics, diatonic and chromatic mediants, secondary dominants, borrowed chords, seventh-chord vocabulary, cadence taxonomy, SATB and keyboard voicing conventions. **Out of scope:** Full atonal set theory, microtonal harmony (future plugin).

---

## 2. Existing Solutions

| System | Approach | Strengths | Limitations for Aurora |
|--------|----------|-----------|------------------------|
| **Music21** | Python analysis + Roman numeral | Rich corpus, Bach chorale tools | Analysis-first; weak generative constraint API |
| **OpenMusic** | Visual patching, constraint objects | Flexible constraint programming | Steep learning curve; no unified rule IDs |
| **Lenardo / Morpheus** | Constraint satisfaction | Multi-voice generation | Limited jazz/pop vocabulary |
| **Hooktheory / AutoChords** | Statistical progressions | Popular templates | Black-box; no explainability |
| **Figured-bass engines** | Rule-based bass + figures | Baroque fidelity | Narrow style |
| **Jazz chord plugins (iReal Pro)** | Template libraries | Real-world jazz changes | Not compositional search |

**Recommendation:** Aurora implements a **declarative rule catalog** (this document) evaluated by the Rule DSL, with progression templates as *soft priors* not sole generators.

---

## 3. Academic / Theoretical Foundation

### 3.1 Functional Harmony (Riemann / Piston / Kostka & Payne)

Three primary functions in major:

| Function | Primary chords | Role |
|----------|----------------|------|
| **Tonic (T)** | I, vi, iii | Rest, stability |
| **Subdominant (S)** | IV, ii | Departure, preparation |
| **Dominant (D)** | V, vii° | Tension, resolution to T |

Minor mode adjusts: V may be major (harmonic minor), ii is diminished or minor depending on context, iv is minor.

**Scale-degree → chord quality (major):**

| Degree | Triad | Seventh (optional) |
|--------|-------|---------------------|
| I | major | maj7 |
| ii | minor | m7 |
| iii | minor | m7 |
| IV | major | maj7 |
| V | major | dom7 |
| vi | minor | m7 |
| vii° | diminished | m7♭5 |

### 3.2 Hindemith Chord Classification

Hindemith ranks intervals from most to least consonant and classifies chords by **root position of strongest bass interval**. Aurora uses Hindemith primarily for:

- Distinguishing **accented vs unaccented** dissonance treatment
- **Fourth/fifth stack** analysis in voicing validation
- **Bridge chords** between remote keys (chromatic mediants)

Not adopted wholesale: Hindemith's rejection of traditional dominant function in favor of pure interval hierarchy is **style-dependent** — classical/jazz presets retain functional analysis.

### 3.3 Harmonic Rhythm

Harmonic rhythm = rate of chord change relative to meter.

| Parameter | Effect |
|-----------|--------|
| `harmonic_rhythm` (user) | Target chord changes per measure |
| `harmonic_rhythm_variance` | Allow syncopated change on weak beats |

Rules HARM-060–HARM-068 govern alignment with meter and phrase boundaries.

### 3.4 Cadence Taxonomy

| Cadence | Formula | Effect |
|---------|---------|--------|
| **Perfect Authentic (PAC)** | V → I, soprano ends on tonic, both in root position | Strong closure |
| **Imperfect Authentic (IAC)** | V → I, incomplete conditions | Moderate closure |
| **Half (HC)** | Ends on V | Open, continuation |
| **Plagal** | IV → I | Amen cadence, extension |
| **Deceptive** | V → vi | Surprise, extension |
| **Phrygian half** | iv⁶ → V (minor) | Classical minor color |

---

## 4. Engineering Analysis

| Criterion | Assessment |
|-----------|------------|
| **Correctness** | Rule catalog covers Bach chorale norms + common-practice progressions; jazz delegated to jazz.md |
| **Controllability** | `complexity`, `dissonance`, `cadence_strength`, `harmonic_rhythm` map directly |
| **Explainability** | Each chord node carries `harmony_rule_ids[]` in provenance |
| **Performance** | Progression search over finite template graph + local voicing CSP is O(n·k) per measure |
| **Extensibility** | Style plugins add chord symbols and progression edges |
| **Complexity** | ~85 rules; phased implementation: triads → sevenths → secondary dominants |

---

## 5. Comparison of Approaches

| Approach | Description | Verdict |
|----------|-------------|---------|
| **Template-only** | Pick from I–IV–V–I library | Fast, brittle; use as soft prior |
| **Markov on Roman numerals** | P(next\|current) | Good variety; weak cadence guarantee |
| **Constraint search on functions** | Build progression satisfying function grammar | **Recommended** for skeleton |
| **SAT voicing solver** | Complete chords after progression | **Recommended** for voicing stage |
| **ML chord prediction** | Transformer on symbol sequences | Optional plugin; not core |

**Hybrid recommended:** Function-grammar skeleton generation + constraint voicing + soft template matching score.

---

## 6. Recommended Solution

### 6.1 Two-Phase Harmony Generation

```text
Phase A: Harmony Skeleton (progression + harmonic rhythm)
  Input:  key, form sections, emotion weights
  Output: Chord symbol sequence on AST Measure nodes
  Rules:  HARM-PROG-*, HARM-CAD-*, HARM-001–040

Phase B: Voicing (SATB or keyboard)
  Input:  Chord symbols + voice ranges + prior voicing
  Output: Note events per Voice
  Rules:  HARM-VOICE-*, cross-ref voice-leading.md (VL-*)
```

### 6.2 Hard vs Soft Policy

| Category | Default | Parameter override |
|----------|---------|-------------------|
| Chord tone completeness (triad) | HARD | `complexity` ≥ 0.7 allows omitting fifth |
| Illegal pitch class in diatonic mode | HARD (strict mode) | `borrowed_chord_tolerance` softens |
| Parallel perfect fifths | SOFT in homophonic; HARD in chorale preset | `counterpoint.strictness` |
| Secondary dominant resolution | SOFT | `cadence_strength` increases weight |

---

## 7. Architecture

```text
Theory Module (harmony.md rules)
        ↓
Harmony Rule Registry (loaded at startup)
        ↓
┌───────────────────┐     ┌────────────────────┐
│ Progression       │     │ Voicing Engine     │
│ Planner           │────►│ (CSP + scoring)    │
└───────────────────┘     └────────────────────┘
        ↓                           ↓
   Chord AST nodes            Note events + provenance
```

**Integration points:**

- **Structure Engine:** section → key area → progression pool
- **Voice Leading:** VL-* rules after voicing
- **Jazz plugin:** replaces/supplements progression edges when `style=jazz`

---

## 8. Data Structures

### 8.1 ChordSymbol (logical)

```rust
// Conceptual — formal schema in ast.md
struct ChordSymbol {
    root: PitchClass,
    quality: ChordQuality,      // maj, min, dom7, dim, aug, ...
    extensions: Vec<Extension>, // 7, 9, 11, 13, alt, sus
    bass: Option<PitchClass>,   // slash chord
    function: Option<Function>, // T, S, D, or None if non-functional
    roman: Option<RomanNumeral>,
}
```

### 8.2 HarmonicContext

```rust
struct HarmonicContext {
    key: Key,
    mode: Mode,
    previous_chord: Option<ChordSymbol>,
    measure_position: BeatPosition,
    section_role: SectionRole,
    tension_curve: f32,  // 0.0–1.0 from emotion resolver
}
```

### 8.3 ProgressionEdge (template graph)

```rust
struct ProgressionEdge {
    from_function: Function,
    to_function: Function,
    typical_romans: Vec<String>,  // e.g. ["V7", "I"]
    weight: f32,
    rule_id: String,              // e.g. "HARM-PROG-001"
}
```

---

## 9. Algorithms

### 9.1 Progression Generation (Beam Search on Function Graph)

```text
state = { roman_sequence[], function_sequence[], score }
for each measure m in section:
  candidates = expand_edges(state.last_function, key, params)
  for each c in candidates:
    apply HARD rules → prune
    score += SOFT rules
  beam = top_k(candidates)
return best complete progression
```

### 9.2 Voicing Algorithm (Greedy + Backtrack)

```text
for each chord C in progression:
  candidates = all_pitch_assignments(C, voice_ranges)
  filter HARD: range, doubling limits, spacing
  rank SOFT: common tone, stepwise bass, register balance
  select best; backtrack if dead-end (max 3 retries)
```

### 9.3 Secondary Dominant Insertion

When `complexity ≥ 0.5`, insert V7/x before target diatonic chord x if:

- Target is not already preceded by its dominant (SOFT: HARM-045)
- Insertion does not break cadence plan (HARD at phrase end: HARM-CAD-010)

---

## 10. Interfaces

### 10.1 Theory → Engine

| Function | Input | Output |
|----------|-------|--------|
| `validate_chord_in_key` | ChordSymbol, Key | bool + rule violations |
| `score_progression_step` | HarmonicContext, ChordSymbol | eval_score, rule_ids |
| `suggest_voicing` | ChordSymbol, VoiceRanges, PriorVoicing | Vec<Voicing> ranked |
| `cadence_at_measure` | MeasureIndex, Section | CadenceType requirement |

### 10.2 Rule Engine

Rules registered as:

```yaml
id: HARM-001
type: HARD
category: harmony
predicate: chord_root_in_scale_or_borrowed_allowlist
params: [borrowed_chord_tolerance]
weight_param: null
```

---

## 11. Parameter Mappings

| User Parameter | Internal Mapping | Affected Rules |
|----------------|------------------|----------------|
| `complexity` | Extension probability, secondary dominant rate | HARM-030–055 |
| `dissonance` | Allow 7ths, sus, altered dominants | HARM-020–028 |
| `cadence_strength` | Weight of HARM-CAD-* at phrase ends | HARM-CAD-001–015 |
| `harmonic_rhythm` | Chord changes per measure target | HARM-060–068 |
| `borrowed_chord_tolerance` | Modal interchange allowlist size | HARM-040–044 |
| `mode` / `key` | Scale pitch class set | HARM-001–010 |
| `style` | Progression graph selection | HARM-PROG-* |

Cross-reference: [deep-research-report.md](../../deep-research-report.md) parameter table.

---

## 12. Explainability Model

Every `Chord` AST node and voicing `Note` carries:

```json
{
  "provenance": {
    "rule_ids": ["HARM-PROG-005", "HARM-VOICE-003"],
    "eval_score": 0.82,
    "reason": "Subdominant preparation before dominant at phrase midpoint",
    "alternatives_rejected": [
      { "roman": "vi", "score": 0.61, "violation": "HARM-PROG-012" }
    ]
  }
}
```

UI Inspector displays Roman numeral, function color (T/S/D), and rule tooltip.

---

## 13. Future Expansion

- Neapolitan and augmented sixth chords (HARM-090+ reserved)
- Pop/Rock loop progressions (vi–IV–I–V) as style plugin
- Post-tonal pitch-class set rules (separate spec)
- Automated harmonic analysis feedback loop (Music21 import)

---

## 14. Open Questions

1. **AST pending:** Exact `Chord` node schema — slash bass as separate field or voicing hint?
2. **Conflict with jazz.md:** When `style=jazz`, do HARM functional rules defer entirely or run in parallel?
3. **Hindemith vs functional:** Default weight when both classify dissonance differently?
4. **Harmonic rhythm syncopation:** HARD prohibition on beat-4 change in chorale preset?

---

## 15. References

- Piston, W. *Harmony* (5th ed., revised Mark DeVoto)
- Kostka, S. & Payne, S. *Tonal Harmony* (8th ed.)
- Hindemith, P. *The Craft of Musical Composition*, Vol. I
- Aldwell, E., Schachter, C., & Cadwallader, C. *Harmony and Voice Leading*
- Bach, J.S. *371 Four-Part Chorales* (Riemenschneider ed.) — validation corpus
- Music21 documentation — Roman numeral analysis
- Aurora [deep-research-report.md](../../deep-research-report.md)
- Aurora [jazz.md](jazz.md), [voice-leading.md](voice-leading.md)

---

## Appendix A: Chord Vocabulary

### A.1 Diatonic Triads and Sevenths (Major)

Full spelling with pitch classes in C major:

| Roman | Symbol | Pitches | Function |
|-------|--------|---------|----------|
| I | C, Cmaj7 | C-E-G(-B) | T |
| ii | Dm, Dm7 | D-F-A(-C) | S |
| iii | Em, Em7 | E-G-B(-D) | T (weak) |
| IV | F, Fmaj7 | F-A-C(-E) | S |
| V | G, G7 | G-B-D(-F) | D |
| vi | Am, Am7 | A-C-E(-G) | T |
| vii° | Bdim, Bm7♭5 | B-D-F(-A) | D |

### A.2 Borrowed and Chromatic (Overview)

| Type | Example in C | Origin |
|------|--------------|--------|
| ♭VII | B♭ major | Mixolydian borrow |
| ♭III | E♭ major | Parallel minor |
| iv | F minor | Parallel minor |
| Neapolitan | D♭ major | ♭II (future) |
| Secondary V | D7 → G | V/V |

### A.3 Jazz Extensions Overview

Seventh chords are baseline in jazz; extensions (9, 11, 13) and alterations (♭9, ♯11, ♭13) detailed in [jazz.md](jazz.md). Harmony module recognizes symbols for AST storage; voicing rules split between HARM-VOICE and JAZZ-VOICE.

---

## Appendix B: Standard Progression Libraries

### B.1 Common-Practice Templates

| ID | Progression | Use |
|----|-------------|-----|
| CP-01 | I – IV – V – I | Fundamental |
| CP-02 | I – vi – IV – V | Pop derivative |
| CP-03 | I – ii – V – I | Strongest tonal |
| CP-04 | I – vi – ii – V | Cycle fragment |
| CP-05 | I – V/IV – IV – V | Secondary dominant |
| CP-06 | I – ♭VI – ♭VII – I | Modal rock |

### B.2 Phrase-Level Plans

| Section role | Typical progression arc |
|--------------|-------------------------|
| Exposition | T → S → D → T (PAC) |
| Development | Sequential modulations, increased chromaticism |
| Recapitulation | Return to tonic, confirm PAC |

---

## Appendix C: Voicing Rules Summary

| Rule | Description | Type |
|------|-------------|------|
| HARM-VOICE-001 | All chord tones present in 4-voice (or justified omission) | HARD |
| HARM-VOICE-002 | Root in bass unless inverted symbol | SOFT |
| HARM-VOICE-003 | Prefer doubled root in triads | SOFT |
| HARM-VOICE-004 | Never double leading tone | HARD |
| HARM-VOICE-005 | Never double altered extension | HARD |
| HARM-VOICE-006 | Spacing: max 12 semitones between adjacent upper voices | SOFT |
| HARM-VOICE-007 | Spacing: bass–tenor ≤ octave (SATB) | SOFT |
| HARM-VOICE-008 | Avoid all voices in same octave | SOFT |
| HARM-VOICE-009 | Resolve tendency tones (7th down, leading tone up) | HARD* |
| HARM-VOICE-010 | Keep 7th in prepared voice when possible | SOFT |

*HARD in chorale preset; SOFT in pop with `dissonance` high.

---

## Appendix D: Complete Rule Catalog

### D.1 Scale and Diatonic Membership

| ID | Rule | Type | Default Weight | Notes |
|----|------|------|----------------|-------|
| HARM-001 | Chord root must belong to current key or borrowed allowlist | HARD | — | Overridden by `borrowed_chord_tolerance` |
| HARM-002 | All chord tones should be explainable by symbol quality | HARD | — | Invalid symbol = generation error |
| HARM-003 | Prefer diatonic triads when `complexity` < 0.3 | SOFT | 1.0 | |
| HARM-004 | Allow diatonic sevenths when `complexity` ≥ 0.3 | SOFT | 0.8 | |
| HARM-005 | Avoid consecutive roots by tritone without preparation | SOFT | 1.2 | |
| HARM-006 | Scale degree 7 (leading tone) should appear in V or vii° | SOFT | 0.9 | Minor: raised 7 |
| HARM-007 | Do not use pitch outside key without functional label | SOFT | 1.5 | Chromatic approach |
| HARM-008 | Harmonic minor implied for V in minor keys | HARD | — | Style classical |
| HARM-009 | Melodic minor available for ascending passages only | SOFT | 0.5 | Melody cross-ref |
| HARM-010 | Mode mixture limited to preset borrow list | SOFT | `borrowed_chord_tolerance` | |

### D.2 Chord Quality and Extensions

| ID | Rule | Type | Default Weight |
|----|------|------|----------------|
| HARM-020 | Triads default when complexity < 0.4 | SOFT | 1.0 |
| HARM-021 | Dominant sevenths on V when complexity ≥ 0.4 | SOFT | 0.85 |
| HARM-022 | Major sevenths on I, IV when jazz/pop preset | SOFT | 0.7 |
| HARM-023 | Diminished vii°7 preferred over V triad in strict chorale | SOFT | 0.6 |
| HARM-024 | Sus4 resolves to major triad on strong beat | SOFT | 1.1 |
| HARM-025 | Add9 acceptable on tonic in pop | SOFT | 0.5 |
| HARM-026 | Avoid simultaneous m2 cluster in close position | HARD | — | Orchestration cross-ref |
| HARM-027 | Extension must not contradict base quality | HARD | — |
| HARM-028 | Altered extensions only on dominant function | SOFT | 0.9 |

### D.3 Functional Progression

| ID | Rule | Type | Default Weight |
|----|------|------|----------------|
| HARM-PROG-001 | T may follow T, S, or D | SOFT | 0.5 |
| HARM-PROG-002 | S typically precedes D | SOFT | 1.0 |
| HARM-PROG-003 | D should resolve to T at cadence | SOFT | 1.5 |
| HARM-PROG-004 | Avoid D → S direct (unprepared) | SOFT | 1.2 |
| HARM-PROG-005 | ii–V–I is preferred jazz/classical cadential loop | SOFT | 1.3 |
| HARM-PROG-006 | IV–V–I valid plagal approach | SOFT | 0.8 |
| HARM-PROG-007 | vi substitutes for I (deceptive setup) | SOFT | 0.7 |
| HARM-PROG-008 | iii connects T to vi | SOFT | 0.6 |
| HARM-PROG-009 | Circle-of-fifths descent preferred in development | SOFT | 0.9 |
| HARM-PROG-010 | Parallel root motion by P5/P4 penalized in strict mode | SOFT | 1.0 |
| HARM-PROG-011 | Do not repeat same chord quality > 3 measures unless pedal | SOFT | 0.8 |
| HARM-PROG-012 | Phrase start favors T or S, not bare D | SOFT | 0.7 |
| HARM-PROG-013 | Avoid root-position V–vi without common tone plan | SOFT | 0.9 |
| HARM-PROG-014 | Sequential transposition max 3 steps without pivot | SOFT | 1.1 |
| HARM-PROG-015 | Return to tonic at section end | HARD | — | Form cross-ref |

### D.4 Secondary Dominants and Chromaticism

| ID | Rule | Type | Default Weight |
|----|------|------|----------------|
| HARM-040 | V7/x must resolve to x (or its substitute) | SOFT | 1.4 |
| HARM-041 | Secondary dominant target must be diatonic | HARD | — |
| HARM-042 | Max one secondary dominant per measure (default) | SOFT | 0.8 |
| HARM-043 | Chromatic mediants change mode color | SOFT | 0.5 |
| HARM-044 | ♭VI–♭VII–I valid in rock preset | SOFT | 0.6 |
| HARM-045 | Insert V7 before modulatory pivot | SOFT | 0.9 |

### D.5 Cadences (HARM-CAD)

| ID | Rule | Type | Default Weight |
|----|------|------|----------------|
| HARM-CAD-001 | Phrase ending requires cadence class match | SOFT | `cadence_strength` |
| HARM-CAD-002 | PAC: V–I root position, soprano on tonic | SOFT | 1.5 |
| HARM-CAD-003 | HC: end on V | SOFT | 1.0 |
| HARM-CAD-004 | Deceptive: V–vi max once per 8 measures unless requested | SOFT | 0.7 |
| HARM-CAD-005 | Plagal follows authentic in codas | SOFT | 0.5 |
| HARM-CAD-006 | Cadence chord change on strong beat | SOFT | 1.1 |
| HARM-CAD-007 | Dominant must precede tonic in PAC | HARD | — | Except evaded |
| HARM-CAD-008 | Half cadence on phrase odd boundaries | SOFT | 0.6 |
| HARM-CAD-009 | Evaded cadence requires pickup plan | SOFT | 0.8 |
| HARM-CAD-010 | No secondary dominant insertion at final cadence | HARD | — |

### D.6 Harmonic Rhythm

| ID | Rule | Type | Default Weight |
|----|------|------|----------------|
| HARM-060 | Chord change rate ≈ `harmonic_rhythm` parameter | SOFT | 1.0 |
| HARM-061 | Prefer change on downbeat | SOFT | 0.9 |
| HARM-062 | Hold tonic through anacrusis | SOFT | 0.5 |
| HARM-063 | Accelerate harmonic rhythm into cadence | SOFT | 0.8 |
| HARM-064 | Development section: faster rhythm | SOFT | 0.7 |
| HARM-065 | Pedal point: bass static ≥ 2 measures | SOFT | 0.6 |
| HARM-066 | Do not change harmony mid-tie | HARD | — |
| HARM-067 | Syncopated change requires strong bass support | SOFT | 0.8 |
| HARM-068 | Final measure often prolongs I | SOFT | 0.7 |

### D.7 Voicing (HARM-VOICE)

| ID | Rule | Type | Default Weight |
|----|------|------|----------------|
| HARM-VOICE-001 | Complete chord tones in 4-part | HARD | — |
| HARM-VOICE-002 | Root position bass default | SOFT | 0.7 |
| HARM-VOICE-003 | Double root in triads | SOFT | 0.6 |
| HARM-VOICE-004 | No doubled leading tone | HARD | — |
| HARM-VOICE-005 | No doubled tendency tone | HARD | — |
| HARM-VOICE-006 | Upper voice spacing ≤ octave | SOFT | 0.8 |
| HARM-VOICE-007 | Bass–tenor ≤ octave | SOFT | 0.7 |
| HARM-VOICE-008 | Avoid voice pile-up (3+ in same octave) | SOFT | 1.0 |
| HARM-VOICE-009 | Resolve chord 7th downward | HARD* | — |
| HARM-VOICE-010 | Prepare chord 7th by common tone or suspension | SOFT | 0.9 |
| HARM-VOICE-011 | Leading tone resolves up to tonic | HARD* | — |
| HARM-VOICE-012 | First inversion triad: double bass note | SOFT | 0.5 |
| HARM-VOICE-013 | Second inversion: double bass, resolve to first inversion | SOFT | 0.8 |
| HARM-VOICE-014 | Omit fifth in extended chords if voiced | SOFT | 0.4 |
| HARM-VOICE-015 | Keep extension in soprano for melody | SOFT | 0.6 |

**Total HARM rules cataloged: 78**

---

## Appendix E: Worked Example

**Input:** C major, 4/4, `complexity=0.5`, `cadence_strength=0.8`, 4-measure phrase

**Generated skeleton:**

| Measure | Roman | Function | Rules fired |
|---------|-------|----------|-------------|
| 1 | I | T | HARM-PROG-012 |
| 2 | IV | S | HARM-PROG-002 |
| 3 | V7 | D | HARM-021 |
| 4 | I | T | HARM-CAD-002, HARM-PROG-003 |

Provenance on measure 4 chord: `["HARM-CAD-002", "HARM-PROG-003", "HARM-VOICE-011"]`
