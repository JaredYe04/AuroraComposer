# Jazz Harmony Theory Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Theory System Research Agent (Jazz)  
**Dependencies:** [harmony.md](harmony.md), [voice-leading.md](voice-leading.md), [rhythm.md](rhythm.md)

---

## Rule ID Naming Convention

| Prefix | Domain | Example |
|--------|--------|---------|
| `JAZZ` | Jazz harmony general | `JAZZ-001` |
| `JAZZ-IIV` | ii–V–I | `JAZZ-IIV-003` |
| `JAZZ-EXT` | Extensions | `JAZZ-EXT-007` |
| `JAZZ-SUB` | Substitutions | `JAZZ-SUB-004` |
| `JAZZ-VOICE` | Voicing conventions | `JAZZ-VOICE-005` |

**Type:** `[HARD]` / `[SOFT]`

---

## 1. Background

Jazz harmony extends common-practice tonality with **tertian extensions**, **altered dominants**, **tritone substitution**, **modal interchange**, and **voicing conventions** (drop 2, shell, rootless). This spec activates when `style` resolves to jazz or jazz-adjacent presets (bebop, standard, modal, fusion).

Sources: **Levine** (*The Jazz Theory Book*), **Coker** (*Hearin' the Changes*), **Nettles & Graf** (*The Chord Scale Theory*), **Mark Levine** voicing chapters, Real Book harmonic practice.

---

## 2. Existing Solutions

| System | Notes |
|--------|-------|
| **iReal Pro** | Progression playback |
| **JazzGuitar.be voicing charts** | Static |
| **Weimar Jazz Database** | Corpus analysis |
| **Impro-Visor** | Lead sheet + solos |

Aurora: rule catalog + progression templates from standards corpus.

---

## 3. Academic / Theoretical Foundation

### 3.1 ii–V–I

Core cadential cycle in major and minor:

| Key | Major ii–V–I | Minor ii–V–I |
|-----|--------------|--------------|
| C | Dm7 – G7 – Cmaj7 | Dm7♭5 – G7 – Cm6 |
| Generic | ii7 – V7 – Imaj7 | iiø7 – V7(alt) – i |

### 3.2 Extensions

| Chord function | Common extensions | Avoid |
|----------------|-------------------|-------|
| maj7 I | 9, 13, 6 | ♭9 on maj |
| min7 ii | 9, 11 | natural 11 on iii minor |
| dom7 V | 9, 13, ♭9, ♯9, ♭13 | — |
| m7♭5 | 11, ♭9 | — |

### 3.3 Substitutions

| Type | Example in C | Mechanism |
|------|--------------|-----------|
| **Tritone sub** | Dm7 – D♭7 – Cmaj7 | Shared tritone G–F# |
| **Relative minor** | Am7 for C | Shared tones |
| **Diminished passing** | C#dim7 between C–Dm | Symmetric |
| **Backdoor** | Fm7 – B♭7 – C | ♭VII cadence |
| **Coltrane sub** | Multi-tonic cycle | Future |

### 3.4 Voicing Conventions

| Voicing type | Structure | Use |
|--------------|-----------|-----|
| **Shell** | Root, 3rd, 7th | Comping baseline |
| **Rootless A** | 3–5–7–9 | Piano left hand |
| **Rootless B** | 7–9–3–5 | Inversion |
| **Drop 2** | Drop second voice from close | Guitar/horns |
| **Drop 3** | Drop third voice | Spread |

Guide tones: 3rd and 7th define chord quality; must resolve correctly on V→I (JAZZ-VOICE-001).

---

## 4. Engineering Analysis

Jazz harmony increases **chord symbol vocabulary** and **voicing search space**. Separate progression graph from classical HARM-PROG; shared VL tendency rules with jazz exceptions (avoid notes, extensions).

---

## 5. Comparison of Approaches

| Approach | Verdict |
|----------|---------|
| Real Book template matching | Good baseline |
| Chord-scale theory for melody | Cross-ref melody engine |
| Rule-based ii–V chains | **Recommended** core |
| ML from Weimar DB | Optional style plugin |

---

## 6. Recommended Solution

When jazz preset active:

1. Replace HARM progression graph with JAZZ-IIV graph
2. Apply JAZZ-EXT based on `complexity` / `dissonance`
3. Insert JAZZ-SUB with probability from `complexity`
4. Voicing via JAZZ-VOICE rules (shell → rootless by density)

---

## 7. Architecture

```text
Style Resolver (jazz) → Jazz Rule Pack
                           ↓
┌────────────────────────────────┐
│ Jazz Progression Planner       │
└───────────────┬────────────────┘
                ↓
┌────────────────────────────────┐
│ Jazz Voicing Engine            │
└───────────────┬────────────────┘
                ↓
         AST Chord + Note events
```

---

## 8. Data Structures

```rust
struct JazzChordSymbol {
    base: ChordSymbol,
    extensions: Vec<JazzExtension>,
    alterations: Vec<Alteration>,
    voicing_template: VoicingTemplate,  // Shell, RootlessA, Drop2
}

struct SubstitutionEdge {
    from: ChordSymbol,
    to: ChordSymbol,
    sub_type: SubType,
    rule_id: String,
}
```

---

## 9. Algorithms

### 9.1 ii–V–I Chain Generator

```text
for each cadence slot:
  insert ii7 (key) → V7 → Imaj7
  optionally tritone sub on V (JAZZ-SUB-001)
  add extensions per JAZZ-EXT-*
```

### 9.2 Rootless Voicing

```text
if voicing_density >= 0.6:
  omit root (assume bass or rootless)
  place 3 and 7 in guide tone voices
  add 9/13 on top per JAZZ-VOICE-003
```

---

## 10. Interfaces

| API | Description |
|-----|-------------|
| `jazz_progression(key, bars) -> ChordSequence` | |
| `apply_tritone_sub(prog) -> ChordSequence` | |
| `voicing_jazz(chord, template) -> Voicing` | |
| `chord_scale(chord) -> Scale` | Melody hint |

---

## 11. Parameter Mappings

| Parameter | Jazz mapping |
|-----------|--------------|
| `complexity` | Extension density, sub rate |
| `dissonance` | Altered dominants |
| `harmonic_rhythm` | Two chords per bar (typical) |
| `style=jazz_bebop` | Faster changes, JAZZ-IIV-005 |
| `style=modal` | Pedal modes, fewer dominants |

---

## 12. Explainability Model

```json
{
  "chord": "G7alt",
  "rules": ["JAZZ-EXT-005", "JAZZ-SUB-001"],
  "substitution": "tritone_from Db7",
  "voicing": "rootless_A"
}
```

---

## 13. Future Expansion

- Coltrane changes (Giant Steps cycle)
- Polytonality
- Slash chord bass reharm
- Voicing leading in big band sections

---

## 14. Open Questions

1. Default voicing template per instrument combo?
2. Conflict resolution when HARM-CAD and JAZZ-IIV disagree on phrase end?

---

## 15. References

- Levine, M. — *The Jazz Theory Book*
- Coker, J. — *Hearin' the Changes*
- Nettles, B. & Graf, R. — *The Chord Scale Theory & Jazz Harmony*
- Mulholland, J. & Hiramatsu, T. — *The Berklee Book of Jazz Harmony*
- Weimar Jazz Database (MIREX)
- Real Book / iReal Pro standards
- Aurora [harmony.md](harmony.md)

---

## Appendix A: ii–V–I Rules (JAZZ-IIV)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| JAZZ-IIV-001 | ii–V–I preferred cadence in major | SOFT | 1.3 |
| JAZZ-IIV-002 | Minor iiø7 – V7 – i | SOFT | 1.2 |
| JAZZ-IIV-003 | V7 duration ≥ 1 beat before resolution | SOFT | 0.9 |
| JAZZ-IIV-004 | ii and V share key center | HARD | — |
| JAZZ-IIV-005 | Bebop: ii–V may appear every 2 beats | SOFT | 0.7 |
| JAZZ-IIV-006 | Turnaround: iii–VI–ii–V at section end | SOFT | 0.85 |
| JAZZ-IIV-007 | Imaj7 after V7 not triad (jazz color) | SOFT | 0.8 |
| JAZZ-IIV-008 | Secondary ii–V before any target | SOFT | 0.9 |
| JAZZ-IIV-009 | Rhythm changes bridge: III7–VI7–II7–V7 | SOFT | 0.7 |
| JAZZ-IIV-010 | Bird changes variant optional | SOFT | 0.5 |

### Turnaround variants

| ID | Progression |
|----|-------------|
| JAZZ-IIV-011 | I – VI7 – ii – V |
| JAZZ-IIV-012 | I – ♯Idim – ii – V |
| JAZZ-IIV-013 | iii – VI – ii – V |

---

## Appendix B: Extensions (JAZZ-EXT)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| JAZZ-EXT-001 | maj7: add 9 or 6, not ♭9 | SOFT | 1.0 |
| JAZZ-EXT-002 | min7: add 9, 11 cautiously | SOFT | 0.8 |
| JAZZ-EXT-003 | dom7: 9 and 13 baseline | SOFT | 0.9 |
| JAZZ-EXT-004 | dom7 alt: ♭9, ♯9, ♭13 mix | SOFT | 0.85 |
| JAZZ-EXT-005 | Altered scale on V7alt | SOFT | 0.7 |
| JAZZ-EXT-006 | Avoid ♮11 on maj7 (#11 OK) | SOFT | 1.0 |
| JAZZ-EXT-007 | Extension must appear in voicing if in symbol | HARD | — |
| JAZZ-EXT-008 | Omit 5th in extended voicings | SOFT | 0.5 |
| JAZZ-EXT-009 | sus4 on dom before resolution | SOFT | 0.6 |
| JAZZ-EXT-010 | ♭9 resolves down on dom | SOFT | 0.9 |

---

## Appendix C: Substitutions (JAZZ-SUB)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| JAZZ-SUB-001 | Tritone sub for V7 | SOFT | 0.9 |
| JAZZ-SUB-002 | Tritone sub preserves resolution to I | HARD | — |
| JAZZ-SUB-003 | Relative minor sub for I | SOFT | 0.6 |
| JAZZ-SUB-004 | Diminished passing dim7 | SOFT | 0.7 |
| JAZZ-SUB-005 | Backdoor ♭VII7 to I | SOFT | 0.8 |
| JAZZ-SUB-006 | Chromatic approach chord down half-step | SOFT | 0.65 |
| JAZZ-SUB-007 | Max one sub per ii–V cell | SOFT | 0.8 |
| JAZZ-SUB-008 | Modal interchange ♭VI–♭VII | SOFT | 0.6 |

---

## Appendix D: Voicing (JAZZ-VOICE)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| JAZZ-VOICE-001 | Guide tones 3+7 present in comping | HARD | — |
| JAZZ-VOICE-002 | Guide tones resolve on V→I | SOFT | 1.1 |
| JAZZ-VOICE-003 | Rootless when bass present | SOFT | 0.8 |
| JAZZ-VOICE-004 | Drop 2 spacing for 4-note sets | SOFT | 0.7 |
| JAZZ-VOICE-005 | Top note melody priority | SOFT | 0.9 |
| JAZZ-VOICE-006 | Avoid m9 cluster in low register | HARD | — |
| JAZZ-VOICE-007 | Shell: R-3-7 or R-7-3 | SOFT | 0.85 |
| JAZZ-VOICE-008 | Left hand voicing ≤ A3–C5 | SOFT | 0.6 |
| JAZZ-VOICE-009 | Spread voicing for sustained chords | SOFT | 0.5 |
| JAZZ-VOICE-010 | Voice lead 7th down, 3rd up on V→I | SOFT | 1.0 |
| JAZZ-VOICE-011 | Omit root on guitar comp | SOFT | 0.7 |
| JAZZ-VOICE-012 | Doubled extension only octave | SOFT | 0.8 |

---

## Appendix E: Modal Jazz (JAZZ-MOD)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| JAZZ-MOD-001 | Mode from chord/scales table | SOFT | 0.9 |
| JAZZ-MOD-002 | Pedal point under mode solo | SOFT | 0.7 |
| JAZZ-MOD-003 | Fewer functional dominants | SOFT | 0.8 |
| JAZZ-MOD-004 | Quartal voicing allowed | SOFT | 0.6 |
| JAZZ-MOD-005 | So What: D Dorian / Eb Dorian | SOFT | 0.7 |

---

## Appendix F: General Jazz Rules (JAZZ)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| JAZZ-001 | Jazz preset activates jazz rule pack over classical HARM-PROG | HARD | — |
| JAZZ-002 | Harmonic rhythm ≥ 1 change per 2 beats in bebop | SOFT | 0.7 |
| JAZZ-003 | Swing feel from rhythm param when jazz | SOFT | 0.9 |
| JAZZ-004 | Lead sheet chord symbol format on AST | HARD | — |
| JAZZ-005 | Blues form: 12-bar default optional | SOFT | 0.8 |
| JAZZ-006 | Standard turnaround at section end | SOFT | 0.85 |
| JAZZ-007 | Melody chord-scale compatibility (cross-ref melody) | SOFT | 0.9 |
| JAZZ-008 | Avoid diatonic maj7 on V in strict jazz | SOFT | 0.8 |

## Appendix G: Chord-Scale Reference (Melody Engine)

| Chord | Scale options |
|-------|---------------|
| maj7 | Ionian, Lydian |
| min7 | Dorian, Aeolian |
| dom7 | Mixolydian, altered, lydian dominant |
| m7♭5 | Locrian, locrian #2 |
| min6 | Melodic minor i |

**Total JAZZ rules cataloged: 56**

---

## Appendix H: Standard Progressions

| Name | Changes (C) |
|------|-------------|
| Autumn Leaves | ii–V–I major + minor |
| Rhythm Changes A | I – vi – ii – V |
| Blues | I7 – IV7 – I7 – V7 |
| Giant Steps | Coltrane cycle (future) |

Each mapped to JAZZ-IIV and JAZZ-SUB rule triggers in template library.
