# Orchestration Theory Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Theory System Research Agent (Orchestration)  
**Dependencies:** [voice-leading.md](voice-leading.md), [harmony.md](harmony.md), [terminology.md](../00-overview/terminology.md)

---

## Rule ID Naming Convention

| Prefix | Domain | Example |
|--------|--------|---------|
| `ORCH` | Orchestration general | `ORCH-001` |
| `ORCH-RNG` | Range | `ORCH-RNG-003` |
| `ORCH-DBL` | Doubling | `ORCH-DBL-002` |
| `ORCH-TEX` | Texture | `ORCH-TEX-005` |

**Type:** `[HARD]` / `[SOFT]`

---

## 1. Background

Orchestration governs **who plays what**: instrument/voice assignment, register, doubling, spacing, and texture type (homophonic, polyphonic, monophonic, mixed). Applied when `orchestration_preset` or `voice_count` > chamber default; maps AST `Voice` nodes to `VoiceGroup` and instrument ranges.

Sources: **Piston** (*Orchestration*), **Adler** (*The Study of Orchestration*), **Rimsky-Korsakov** (*Principles*), **Hindemith** (*CoMC* instrumentation chapter).

---

## 2. Existing Solutions

| System | Notes |
|--------|-------|
| **MIDI GM program map** | 128 instruments |
| **Orchestration VST templates** | DAW presets |
| **MusicXML part-list** | Export structure |

Aurora: register constraints + doubling rules before export MIDI channel assignment.

---

## 3. Academic / Theoretical Foundation

### 3.1 Standard Ranges (Written)

| Voice/Instrument | Low | High | Comfortable |
|------------------|-----|------|-------------|
| Soprano | C4 | C6 | D4–A5 |
| Alto | G3 | E5 | A3–D5 |
| Tenor | C3 | G4 | D3–C5 |
| Bass | E2 | E4 | G2–D4 |
| Violin | G3 | A6 | G3–E6 |
| Viola | C3 | E6 | C3–A5 |
| Cello | C2 | A5 | C2–G4 |
| Double bass | E1 | G4 | E1–D3 |
| Flute | C4 | C7 | D4–D6 |
| Oboe | B♭3 | A6 | D4–G5 |
| Clarinet (Bb) | D3 | B♭6 | E3–G5 |
| Horn (F) | B1 | F5 | C3–G4 |
| Trumpet | E3 | C6 | G3–E5 |
| Trombone | E2 | F5 | F2–D4 |

### 3.2 Doubling

| Practice | Rule |
|----------|------|
| Double melody at octave | Brilliance |
| Double bass at octave below cello | Weight |
| Avoid double leading tone | HARD (VL-DBL-002) |
| Unison on strong beats | Impact |
| Don't double fragile registers | SOFT |

### 3.3 Texture Types

| Texture | Definition | Aurora param |
|---------|------------|--------------|
| **Monophonic** | Single line | texture=0 |
| **Homophonic** | Melody + accompaniment | texture=0.3 |
| **Polyphonic** | Independent lines | texture=0.8 |
| **Mixed** | Section-dependent | texture=mixed |

### 3.4 Spacing Principles

- **Brass/wind:** avoid m2 in same section in low register (ORCH-TEX-006)
- **Strings:** divisi when chord tones exceed players
- **Woodwind balance:** don't bury oboe in heavy brass

---

## 4. Engineering Analysis

Orchestration rules primarily **constrain pitch assignment and voice count** after musical content generated. HARD range rules prevent unplayable output.

---

## 5. Comparison of Approaches

| Approach | Verdict |
|----------|---------|
| Fixed SATB only | Simple default |
| Preset → range map | **Recommended** |
| Full instrumental CSP | Phase 2 |

---

## 6. Recommended Solution

```text
orchestration_preset → VoiceGroup template + ORCH-RNG tables
Generated pitches → validate ORCH-RNG HARD
Scoring → ORCH-DBL, ORCH-TEX SOFT for assignment
Export → MIDI program per VoiceGroup
```

---

## 7. Architecture

```text
orchestration.md → ORCH Rule Registry
                       ↓
┌────────────────────────────┐
│ VoiceGroup Assigner        │
└─────────────┬──────────────┘
              ↓
┌────────────────────────────┐
│ Range Validator (Repair)   │
└─────────────┬──────────────┘
              ↓
         Export / Playback
```

---

## 8. Data Structures

```rust
struct InstrumentRange {
    id: InstrumentId,
    written_low: Pitch,
    written_high: Pitch,
    comfortable_low: Pitch,
    comfortable_high: Pitch,
}

struct VoiceGroupPlan {
    group_id: String,
    instruments: Vec<InstrumentId>,
    texture: TextureType,
    doubling_policy: DoublingPolicy,
}
```

---

## 9. Algorithms

### 9.1 Range Check

```text
for each note in voice:
  if pitch not in instrument.comfortable:
    if HARD ORCH-RNG: transpose octave or reject
    else: penalty ORCH-RNG-010
```

### 9.2 Doubling Assignment

```text
for chord_tone in sonority:
  if needs weight and register allows:
    assign ORCH-DBL-001 octave double
  check ORCH-DBL-003 max doublings per chord
```

---

## 10. Interfaces

| API | Description |
|-----|-------------|
| `preset_ranges(preset) -> HashMap<Instrument, Range>` | |
| `validate_orchestration(ast) -> Violation[]` | |
| `assign_voice_groups(form, preset) -> VoiceGroupPlan[]` | |

---

## 11. Parameter Mappings

| Parameter | Mapping |
|-----------|---------|
| `orchestration_preset` | Instrument set |
| `voice_count` | Active parts |
| `texture` | ORCH-TEX weights |
| `melody_register`, `bass_register` | Override defaults |
| `dynamic_range` | Velocity scaling per group |

---

## 12. Explainability Model

```json
{
  "voice": "violin_1",
  "rule": "ORCH-RNG-001",
  "note": "G6",
  "action": "down_octave",
  "reason": "Exceeds comfortable upper range"
}
```

---

## 13. Future Expansion

- Full symphonic section balance
- Percussion orchestration (non-pitch)
- Extended techniques
- Microtonal instruments

---

## 14. Open Questions

1. Divisi automatic splitting rules?
2. Transposing instruments (Bb clarinet) in AST as written or concert?

---

## 15. References

- Piston, W. — *Orchestration*
- Adler, S. — *The Study of Orchestration*
- Rimsky-Korsakov — *Principles of Orchestration*
- Hindemith — *The Craft of Musical Composition*
- Aurora [voice-leading.md](voice-leading.md)

---

## Appendix A: Complete Rule Catalog

### Range (ORCH-RNG)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| ORCH-RNG-001 | Note within instrument absolute range | HARD | — |
| ORCH-RNG-002 | Prefer comfortable range | SOFT | 1.0 |
| ORCH-RNG-003 | Melody in upper tessitura of ensemble | SOFT | 0.7 |
| ORCH-RNG-004 | Bass below middle C when possible | SOFT | 0.8 |
| ORCH-RNG-005 | Avoid crossing instrument family ranges illogically | SOFT | 0.6 |
| ORCH-RNG-006 | String section spans ≥ 2 octaves total | SOFT | 0.5 |
| ORCH-RNG-007 | High woodwinds not below written C4 | SOFT | 0.9 |
| ORCH-RNG-008 | Trombone gliss zone marked optional | SOFT | 0.4 |
| ORCH-RNG-009 | Double bass sounds octave lower (8vb) | HARD | — |
| ORCH-RNG-010 | Out-of-comfort penalty scales with distance | SOFT | 1.0 |

### Doubling (ORCH-DBL)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| ORCH-DBL-001 | Double melody at octave for brightness | SOFT | 0.6 |
| ORCH-DBL-002 | Double bass line cello + bass 8vb | SOFT | 0.8 |
| ORCH-DBL-003 | Max 3-fold doubling same pitch class | SOFT | 0.9 |
| ORCH-DBL-004 | Unison doubling on climax | SOFT | 0.7 |
| ORCH-DBL-005 | Avoid double leading tone (cross-ref VL) | HARD | — |
| ORCH-DBL-006 | Horn doubles harmony not melody by default | SOFT | 0.6 |
| ORCH-DBL-007 | Woodwind unison: max 2 instruments | SOFT | 0.8 |
| ORCH-DBL-008 | Brass unison: watch dynamic overload | SOFT | 0.7 |

### Texture (ORCH-TEX)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| ORCH-TEX-001 | Texture matches `texture` param | SOFT | 1.0 |
| ORCH-TEX-002 | Homophonic: accompaniment rhythmic subordination | SOFT | 0.8 |
| ORCH-TEX-003 | Polyphonic: min 2 independent rhythm profiles | SOFT | 0.9 |
| ORCH-TEX-004 | Monophonic: single active voice group | HARD | — |
| ORCH-TEX-005 | Mixed: section role drives texture | SOFT | 0.7 |
| ORCH-TEX-006 | No m2 cluster in same woodwind section low | HARD | — |
| ORCH-TEX-007 | Brass chords spaced open | SOFT | 0.6 |
| ORCH-TEX-008 | Strings arco vs pizz section marker | SOFT | 0.5 |
| ORCH-TEX-009 | Percussion non-pitch on separate VoiceGroup | HARD | — |
| ORCH-TEX-010 | Tutti for climax sections | SOFT | 0.85 |

### Balance & Scoring (ORCH)

| ID | Rule | Type | Weight |
|----|------|------|--------|
| ORCH-001 | Every active VoiceGroup has defined instrument | HARD | — |
| ORCH-002 | Max simultaneous soloists = 1 unless countermelody | SOFT | 0.8 |
| ORCH-003 | Accompaniment velocity < melody | SOFT | 0.7 |
| ORCH-004 | Brass rest every 2 measures in long passages | SOFT | 0.5 |
| ORCH-005 | Woodwind stagger breathing | SOFT | 0.5 |
| ORCH-006 | Cello/Viola fill inner harmony | SOFT | 0.7 |
| ORCH-007 | Harp/piano not simultaneous same register | SOFT | 0.6 |
| ORCH-008 | Section balance: strings baseline | SOFT | 0.6 |
| ORCH-009 | Pitches assigned to lowest capable instrument | SOFT | 0.7 |
| ORCH-010 | Export MIDI channel per VoiceGroup | HARD | — |

**Total ORCH rules cataloged: 41** (ORCH-004 listed once in source count)

---

## Appendix B: Preset Templates

| Preset ID | VoiceGroups | Default texture |
|-----------|-------------|-----------------|
| `SATB_choral` | S, A, T, B | Homophonic |
| `string_quartet` | Vln1, Vln2, Vla, Vc | Polyphonic |
| `jazz_combo` | Piano, Bass, Drums, Solo | Mixed |
| `symphony_wind` | Full orchestra | Mixed |
| `piano_solo` | Piano | Homophonic |

---

## Appendix C: Range Table (Orchestral Summary)

| Section | Instruments | Combined span |
|---------|-------------|---------------|
| Woodwinds | Fl, Ob, Cl, Bsn | B1–C7 (written) |
| Brass | Hn, Tpt, Tbn, Tuba | E1–F5 |
| Strings | Vln–Cb | E1–A6 (8vb bass) |

Rules ORCH-RNG-001–010 apply per instrument assignment.

---

## Appendix D: Texture Decision Matrix

| Section energy | Melody present | Recommended texture |
|----------------|----------------|---------------------|
| Low | Yes | Homophonic sparse |
| Medium | Yes | Homophonic full |
| High | Yes | Mixed / tutti |
| Any | No | Polyphonic or pedal |

Mapped to ORCH-TEX-001, ORCH-TEX-005, FORM-SEC energy values.
