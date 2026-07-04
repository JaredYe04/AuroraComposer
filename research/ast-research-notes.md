# Music AST Research Notes (Raw)

**Agent:** Music AST Research Agent  
**Date:** 2026-07-04  
**Status:** Raw notes — not normative; see `docs/02-music-model/` for specifications

---

## 1. Survey of Existing Representations

### Music21 (MIT)

- **Hierarchy:** `Stream` (recursive container) → `Measure` → `Note`/`Rest`/`Chord`
- **Strengths:** Mature MusicXML/MIDI import; pitch/duration as first-class objects; analytical tools
- **Weaknesses for Aurora:** No provenance; flat Stream model conflates form and notation; mutable Python objects; no search-state semantics
- **Borrow:** Pitch spelling (`Pitch` with `.nameWithOctave`), duration as `Fraction`, offset-based positioning within parent
- **Reject:** Universal Stream-only model (Aurora needs explicit form hierarchy)

### MusicXML 4.0

- **Hierarchy:** `score-partwise` → `part` → `measure` → `note`/`backup`/`forward`
- **Strengths:** Industry interchange standard; rich notation semantics
- **Weaknesses:** XML verbosity; voice handling via `<backup>` rewind; no generative metadata
- **Borrow:** Part/voice semantics, `<direction>` for tempo/dynamics, `<harmony>` for chord symbols
- **Reject:** Direct AST mirroring (export projection only)

### SMF (Standard MIDI File)

- **Model:** Track chunks with delta-time events
- **Strengths:** Playback-native; channel assignment
- **Weaknesses:** No form; no chord symbols; limited pitch bend/expression granularity in Type 0/1
- **Borrow:** Channel-per-voice mapping for IR; tick-based timing
- **Reject:** As primary representation

### OpenMusic (IRCAM)

- **Model:** Visual patch-based composition; hierarchical `chord-seq`, `multi-seq`
- **Strengths:** Algorithmic composition heritage; Lisp CLOS objects
- **Weaknesses:** Closed ecosystem; no explainability standard
- **Borrow:** Separation of structural planning from note filling

### Lenardo / Helio / MuseScore internal

- **Lenardo:** Track-based DAW model with clips
- **Helio:** Pattern-based sequencer with scales
- **MuseScore:** Engraving-first; `Score` → `Part` → `Staff` → `Segment` → `Element`
- **Insight:** Engraving models optimize for layout, not generation. Aurora optimizes for generation + explainability.

### ABC Notation

- **Model:** Monophonic/voice-per-staff text format
- **Strengths:** Human-readable; folk tradition
- **Weaknesses:** Limited polyphony expressiveness; no provenance
- **Borrow:** Key/meter header semantics for metadata

---

## 2. Key Design Questions (Resolved in Spec)

| Question | Decision | Rationale |
|----------|----------|-----------|
| Track vs Voice vs Part? | **Voice** in AST; **Channel** in IR | Terminology glossary; Voice = generative unit |
| Where do chord symbols live? | **Chord events** in dedicated harmony voice OR measure-level harmony slot | Both supported; harmony skeleton writes measure-level, voicing writes voice events |
| Absolute vs relative time in AST? | **Relative within Measure** (beat offset); absolute computed via Timeline | Matches measure-oriented generation; avoids floating-point drift |
| Immutable search states? | **Copy-on-write AST snapshots** with persistent node IDs | ADR pending; enables beam search without cloning full tree |
| UUID vs integer node IDs? | **`NodeId(u64)`** with generation counter | Compact; Rust-friendly; stable across patches |
| Pitch representation? | **`Pitch { midi: u8, spelling: Option<Spelling> }`** | MIDI for computation; spelling for export/engraving |
| Duration representation? | **`Duration { ticks: u32, written: WrittenDuration }`** | Ticks for IR alignment; written for notation |
| Provenance granularity? | **Per Event**, with optional aggregation for UI | Principle 2: Everything Is Explainable |
| AST version field? | **`AstSchemaVersion(major, minor, patch)`** on Composition | Forward-compatible serialization |
| Multi-movement support? | **Yes**, single Movement default for pop/jazz | Classical form requirement |

---

## 3. Music21 Mapping Table (Draft)

| Music21 Class | Aurora AST Node | Notes |
|---------------|-----------------|-------|
| `stream.Score` | `Composition` | Top container |
| `stream.Part` | `Voice` (within measures) | Aurora voices are per-measure containers |
| `stream.Measure` | `Measure` | Direct mapping |
| `note.Note` | `Event::Note` | |
| `note.Rest` | `Event::Rest` | |
| `chord.Chord` | `Event::Chord` | |
| `harmony.ChordSymbol` | `Event::HarmonyMarker` or measure `HarmonySlot` | |
| `tempo.MetronomeMark` | `GlobalAttributes::tempo_map` entry | Not per-event in AST |
| `meter.TimeSignature` | `MeterChange` in timeline | |
| `key.Key` | `KeyArea` in key map | |
| `expressions.TextExpression` | `Event::Marker` | |
| `spanner.Slur` | `Event::SpannerMarker` | Phase 2 |
| `stream.Voice` (voice id) | `Voice.voice_index` | MusicXML voice number |

---

## 4. Timing Model Research

### Approaches Considered

1. **Seconds-based (DAW):** `start_time: f64` — simple for audio, bad for meter changes
2. **Tick-based (MIDI):** `tick: u32` with PPQ — export-friendly, opaque for theory
3. **Rational beats (Music21):** `offset: Fraction` in quarter notes — theory-friendly
4. **Hybrid (chosen):** Beat offset within measure + Timeline projection to ticks

### PPQ Choice

- **480 PPQ** (MIDI convention) for IR
- AST uses **rational beats** (`BeatOffset = (num: u32, den: u32)`)
- Conversion: `ticks = beats * PPQ * 4 / beat_unit` adjusted by tempo map

### Tempo Map

- Piecewise-linear segments: `(beat_position, bpm, ramp: Option<RampType>)`
- Supports ritardando/accelerando as ramp segments
- Stored in `GlobalAttributes`, referenced by Timeline engine

---

## 5. Provenance Schema Research

### Required Fields (from philosophy)

- `source`: enum (Generated, ManualEdit, Imported, Repaired, Plugin)
- `stage`: pipeline stage ID
- `rule_ids`: Vec<RuleId> that contributed
- `eval_score`: Option<f64>
- `search_step`: Option<u32>
- `parent_state_ref`: Option<StateRef>
- `timestamp`: generation timestamp
- `parameters_snapshot`: Hash of relevant params at generation time

### UI Requirements

- Inspector shows provenance chain
- Click rule ID → jump to rule definition
- Diff view: before/after patch for Repair stage

---

## 6. Patch / Versioning Research

### Operations

- `InsertNode`, `DeleteNode`, `ReplaceNode`, `MoveNode`, `UpdateField`
- Patches are **atomic** and **reversible** (inverse patch computable)
- Undo stack stores patch sequence, not full AST copies

### Immutability during Search

- Search operates on `AstSnapshot` (Arc-based COW)
- Commit: selected snapshot → single `ApplyPatch` to canonical AST
- Alternative rejected: in-place mutation with rollback (error-prone, hard to parallelize)

### Serialization

- JSON for IPC (Tauri)
- Binary CBOR for project files (smaller)
- Schema evolution via `AstSchemaVersion` + migration functions

---

## 7. Voice / Channel Research

### Default Voice Layout (4-part + drums)

| VoiceId | Role | Register (MIDI) | MIDI Channel |
|---------|------|-----------------|--------------|
| V0 | Melody | 60–84 | 1 |
| V1 | Alto/Inner | 55–72 | 2 |
| V2 | Tenor/Inner | 48–67 | 3 |
| V3 | Bass | 36–60 | 4 |
| V4 | Drums | N/A (unpitched) | 10 |

### Voice Groups

- `RhythmSection`: bass + drums
- `HarmonyBed`: inner voices
- `Lead`: melody
- Used by structure engine for texture parameters

---

## 8. Event Type Inventory (Draft)

Core: Note, Chord, Rest  
Markers: SectionBoundary, RehearsalMark, DynamicMark, ArticulationRegion  
Control: TempoChange (marker only — actual tempo in map), DynamicAutomation, PitchBendAutomation  
Harmony: ChordSymbol, FiguredBass  
Percussion: DrumHit (unpitched note subtype)

---

## 9. Invariant Checklist

1. Sum of event durations in a voice ≤ measure duration (unless tie extends)
2. Events within a voice are non-overlapping (unless chord tones in Chord event)
3. All Event nodes have non-empty Provenance
4. Measure numbers are contiguous within Section
5. Key at any beat is derivable from key_map
6. Voice indices are stable within Composition
7. NodeId references are valid (no dangling)
8. Ties connect same pitch across measure boundaries only via TieLink metadata

---

## 10. Open Research Items

- [ ] Microtonal pitch support (quarter tones) — defer to v0.2
- [ ] Tuplets nested >2 levels — specify in rhythm spec
- [ ] Cross-staff beaming — engraving concern, not AST
- [ ] Linked parts (transposing instruments) — future
- [ ] Real-time collab editing (CRDT) — future
- [ ] WASM size budget for full AST in browser — profile in Phase 2

---

## 11. References (Raw)

- Music21 v9 documentation: https://web.mit.edu/music21/doc/
- MusicXML 4.0 W3C: https://www.w3.org/2021/06/musicxml40/
- SMF specification
- "Representation of Music" — Huron, 2002
- OpenMusic reference manual
- Bach chorale dataset structure (HMM analysis papers)
- Groove MIDI Dataset (Google Magenta)
- RFC-style immutable data structures for Rust: `im` crate, `rpds`

---

*End of raw research notes*
