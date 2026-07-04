# Generation Pipeline

**Document:** Aurora Composer — Generation Pipeline  
**Version:** 0.1  
**Status:** Draft

---

## 1. Pipeline Overview

The generation pipeline is the **ordered execution of algorithm stages** that transform user parameters into a complete, validated Composition AST.

Each stage is a **pure function** at the architectural level:

```
(stage_input: AST_read + Parameters) → (AST_write + Provenance[])
```

Stages execute sequentially. Within a stage, search may explore multiple candidates before committing.

---

## 2. Pipeline Diagram

```text
                    ┌─────────────────┐
                    │ User Parameters │
                    └────────┬────────┘
                             │
              ┌──────────────▼──────────────┐
              │      1. Style Resolver       │
              │  genre → param bundle        │
              │  genre → plugin set          │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │     2. Emotion Resolver      │
              │  emotion → weight deltas     │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │    3. Structure Planning     │
              │  form, sections, keys, tempo │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │     4. Theme Planning        │
              │  theme slots, motif strategy │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │    5. Harmony Skeleton       │
              │  chord progression + rhythm  │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │     6. Rhythm Skeleton       │
              │  patterns, subdivisions      │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │       7. Melody              │◄── Search
              │  primary melodic lines       │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │     8. Counterpoint          │◄── Search
              │  inner voices                │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │        9. Bass               │◄── Search
              │  bass line                   │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │       10. Drums              │
              │  percussion patterns         │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │     11. Decoration           │
              │  ornaments, grace notes      │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │       12. Repair             │
              │  fix soft violations         │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │      13. Validation          │
              │  hard constraint check       │
              └──────────────┬──────────────┘
                             │
              ┌──────────────▼──────────────┐
              │       14. Export             │
              │  AST → IR → formats          │
              └─────────────────────────────┘
```

---

## 3. Stage Specifications (Summary)

### Stage 1: Style Resolver

| Property | Value |
|----------|-------|
| Search | No |
| AST Write | Composition.metadata, plugin_selection |
| Input | `style.genre`, `style.era`, `style.orchestration` |
| Output | Expanded parameter bundle; active plugin list |

Resolves style presets (e.g., "Jazz Standard", "Baroque Chorale") into concrete parameter values and plugin activations.

---

### Stage 2: Emotion Resolver

| Property | Value |
|----------|-------|
| Search | No |
| AST Write | Composition.metadata.emotion_profile |
| Input | `emotion.valence`, `emotion.arousal`, `emotion.tension_curve` |
| Output | Weight delta table for downstream engines |

Maps emotion dimensions to harmonic, rhythmic, and melodic weight adjustments. See [emotion-engine.md](../04-algorithms/emotion-engine.md).

---

### Stage 3: Structure Planning

| Property | Value |
|----------|-------|
| Search | Optional (DP for form optimization) |
| AST Write | Movement[], Section[], tempo_map, key_map |
| Input | `form.*`, `mode.key`, total duration |
| Output | Section tree with roles and boundaries |

Creates the formal skeleton: how many sections, their roles (A, B, bridge, etc.), key areas, tempo changes.

---

### Stage 4: Theme Planning

| Property | Value |
|----------|-------|
| Search | Optional |
| AST Write | Section.theme_refs, Motif definitions |
| Input | `theme.count`, `theme.motif_length`, `theme.repetition_ratio` |
| Output | Theme slot assignments per section |

Allocates themes to sections and defines motif development strategy (sequence, inversion, augmentation).

---

### Cross-Cutting: Phrase Engine Hook (after Stage 4)

> **REV-001 resolution:** Phrase Engine is not a numbered pipeline stage; it runs as a **cross-cutting service** at defined hooks.

| Hook | When | Action |
|------|------|--------|
| **PHRASE-HOOK-1** | After Stage 4 (Theme Planning) | Compute cadence expectations per phrase; write `PhraseMetadata.cadence_target` |
| **PHRASE-HOOK-2** | Before Stage 5 (Harmony Skeleton) | Lock cadence slot chord requirements (e.g., V→I at phrase end) |
| **PHRASE-HOOK-3** | After Stage 7 (Melody) | Validate phrase terminal contour; flag soft violations for Repair |

```text
Stage 4 Theme Planning
       ↓
  [PHRASE-HOOK-1]  PhraseEngine.plan_phrases()
       ↓
  [PHRASE-HOOK-2]  PhraseEngine.apply_cadence_constraints()
       ↓
Stage 5 Harmony Skeleton
       ...
Stage 7 Melody
       ↓
  [PHRASE-HOOK-3]  PhraseEngine.validate_phrase_terminals()
       ↓
Stage 8 Counterpoint
```

Orchestrator invokes hooks via `PhraseEngine` trait. See [phrase-engine.md](../04-algorithms/phrase-engine.md).

---

### Stage 5: Harmony Skeleton

| Property | Value |
|----------|-------|
| Search | Yes (beam search over chord sequences) |
| AST Write | Measure.chord_events, harmonic rhythm |
| Input | key_map, `harmony.*` parameters |
| Output | Chord symbol per harmonic rhythm point |

Generates chord progressions respecting style rules. Voicing details deferred to later stages.

---

### Stage 6: Rhythm Skeleton

| Property | Value |
|----------|-------|
| Search | No (pattern selection) |
| AST Write | Measure.rhythm_pattern, subdivision attrs |
| Input | `rhythm.*`, time signature, style |
| Output | Metric framework per measure |

Selects and applies rhythm patterns from style-appropriate libraries.

---

### Stage 7: Melody

| Property | Value |
|----------|-------|
| Search | **Yes** (primary beam search) |
| AST Write | Voice[melody].note_events |
| Input | Chords, rhythm skeleton, theme motifs |
| Output | Melodic notes with full provenance |

Core generative stage. Beam search over candidate notes at each beat position, scored by rule engine.

**Candidate generation probabilities** (defaults, parameterized):

| Type | Default Weight | Parameter |
|------|---------------|-----------|
| Chord tone | 70% | `melody.chord_tone_bias` |
| Neighbor tone | 15% | `melody.neighbor_tone_bias` |
| Passing tone | 10% | `melody.passing_tone_bias` |
| Ornament | 5% | `melody.ornament_density` |

---

### Stage 8: Counterpoint

| Property | Value |
|----------|-------|
| Search | Yes |
| AST Write | Voice[inner].note_events |
| Input | Melody, chords, `counterpoint.strictness` |
| Output | Additional voice lines |

Generates inner voices respecting voice-leading rules. Parallel fifth/octave avoidance weighted by `counterpoint.strictness`.

---

### Stage 9: Bass

| Property | Value |
|----------|-------|
| Search | Yes (narrow beam) |
| AST Write | Voice[bass].note_events |
| Input | Harmony skeleton, melody, `register.bass_*` |
| Output | Bass line |

Root motion, walking bass, or pedal patterns depending on style.

---

### Stage 10: Drums

| Property | Value |
|----------|-------|
| Search | No (pattern + variation) |
| AST Write | Voice[drums].note_events |
| Input | Structure, rhythm skeleton, `drums.*` |
| Output | Percussion events (MIDI channel 10) |

Pattern selection from Groove MIDI-derived libraries with parameterized density and fill frequency.

---

### Stage 11: Decoration

| Property | Value |
|----------|-------|
| Search | No |
| AST Write | Note ornament attrs |
| Input | Melody voices, `melody.ornament_density` |
| Output | Trills, mordents, grace notes |

Post-hoc ornamental enrichment. Does not alter structural notes.

---

### Stage 12: Repair

| Property | Value |
|----------|-------|
| Search | Limited (local fix search) |
| AST Write | Patches to violating events |
| Input | Full AST, validation report |
| Output | Corrected AST |

Fixes soft-constraint violations: range overflows, voice-leading issues, unresolved dissonances.

---

### Stage 13: Validation

| Property | Value |
|----------|-------|
| Search | No |
| AST Write | Validation report only |
| Input | Full AST |
| Output | Pass/fail + violation list |

Final hard-constraint check. Failure aborts export and reports to UI.

---

### Stage 14: Export

| Property | Value |
|----------|-------|
| Search | No |
| AST Read | Full AST → IR projection |
| Output | Files (MusicXML, MIDI, ABC, PDF) |

Not a generative stage. Projects AST to IR and invokes format exporters.

---

## 4. Incremental Generation

The pipeline supports **partial execution** for interactive preview:

| Mode | Stages Executed | Use Case |
|------|----------------|----------|
| Full | 1–14 | Complete composition |
| Preview | 1–7 (2 bars) | Parameter tweaking |
| Section | 3–12 (one section) | Section regeneration |
| Harmony-only | 1–5 | Chord progression preview |
| Repair-only | 12–13 | Post-edit validation |

Stage boundaries are checkpointable: AST state saved after each stage for undo/resume.

---

## 5. Progress Reporting

```text
StageProgress {
  stage_name: string       // e.g., "Melody"
  stage_index: u8          // 7 of 14
  percent: f32             // 0.0–1.0 within stage
  message: string         // e.g., "Generating measure 12/32"
}
```

Emitted via Tauri events to frontend progress bar.

---

## 6. Pipeline Configuration

Stages may be skipped or reordered via style plugins:

```text
PipelineConfig {
  stages: StageId[]         // ordered list
  skip: StageId[]           // disabled stages
  plugins: HashMap<StageId, PluginId>  // stage overrides
}
```

Default pipeline defined in ACAS. Style plugins may substitute engines but not remove validation or provenance requirements.

---

## References

- [Architecture](architecture.md)
- [Module Overview](module-overview.md)
- [Deep Research Report §2.2](../../deep-research-report.md)
- [Melody Engine](../04-algorithms/melody-engine.md) *(pending)*
