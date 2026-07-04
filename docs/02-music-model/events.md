# Event Type Catalog Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Music AST Research Agent  
**Dependencies:** [ast.md](ast.md), [ir.md](ir.md), [timeline.md](timeline.md)

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

An **Event** is the atomic timed element in the Music AST. Events live in `MeasureVoice.events` and represent anything positioned on the timeline within a measure: pitched notes, rests, chords, annotations, and automation points.

This catalog is the **normative schema** for all event subtypes, their fields, validation rules, and IR lowering behavior.

### 1.1 Event Taxonomy

```text
Event
├── Note          (single pitched sound)
├── Chord         (simultaneous pitches, one voice)
├── Rest          (silence)
├── Marker        (non-sounding annotation)
└── Automation    (continuous control)
```

**HarmonySlot** (in `Measure.harmony_slots`) is measure-level, not an Event — but shares timing semantics.

---

## 2. Existing Solutions

| System | Event Model | Gap |
|--------|------------|-----|
| MusicXML `<note>` | note, rest, chord (via `<chord/>`) | No provenance |
| MIDI | Note On/Off, CC, Program Change | Unpitched merge |
| Music21 | `Note`, `Rest`, `Chord` objects | No marker/automation union |
| DAW clips | MIDI note events | Absolute time only |

Aurora unifies these under a tagged enum with mandatory provenance.

---

## 3. Academic / Theoretical Foundation

### 3.1 Event Ontology in Music

Events classify into:

- **Sounds** — notes, chords (pitch + duration + intensity)
- **Silences** — rests (structured absence)
- **Signs** — dynamics, articulations (Barthesian "notation")
- **Controls** — continuous parameters (expression)

### 3.2 Non-Chord Tones

Note events carry `PitchRole` for theory engine analysis (passing, neighbor, etc.) — essential for rule evaluation and education UI.

---

## 4. Engineering Analysis

### 4.1 Tagged Enum vs Trait Objects

Rust `enum Event` with serde tagging chosen over trait objects for:

- Exhaustive matching in rule engine
- Zero-cost dispatch
- JSON schema clarity

### 4.2 Shared Base Fields

`TimedEventBase` factored to avoid duplication; each variant embeds `base`.

### 4.3 Duration Semantics

`WrittenDuration` preserves notation intent; tick duration computed at IR projection. Tuplets stored explicitly.

---

## 5. Comparison of Approaches

| Design | Pros | Cons | Verdict |
|--------|------|------|---------|
| Single Note type only | Simple | Chords awkward | Rejected |
| Note + Chord separate | Clear | Duplication | **Selected** |
| Marker as Event attr | Fewer nodes | Mixed concerns | Rejected |
| Automation in separate track | DAW-like | Breaks voice model | Rejected |

---

## 6. Recommended Solution

Five top-level Event variants. All require `TimedEventBase` with `Provenance`. Markers and Automation are Events (not separate trees) to unify traversal and patching.

---

## 7. Architecture

```text
MeasureVoice
└── events: Vec<Event>  (sorted by base.offset)

Rule Engine ──► visit_event(match variant)
IR Projector  ──► lower_event(match variant)
UI Inspector  ──► display fields + provenance
```

---

## 8. Data Structures

### 8.1 TimedEventBase (All Events)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedEventBase {
    pub id: NodeId,
    pub offset: BeatOffset,
    pub duration: WrittenDuration,
    pub provenance: Provenance,
    pub visible: bool,               // false = suppressed in export (cue notes)
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | NodeId | yes | Stable identity |
| `offset` | BeatOffset | yes | Start within measure |
| `duration` | WrittenDuration | yes | Notated length (0 for point events) |
| `provenance` | Provenance | yes | Invariant I-PROV-1 |
| `visible` | bool | yes | Default true |

---

### 8.2 NoteEvent

Single pitched note.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteEvent {
    pub base: TimedEventBase,
    pub pitch: Pitch,
    pub velocity: u8,
    pub tie: TieSpec,
    pub articulations: Vec<Articulation>,
    pub ornaments: Vec<Ornament>,
    pub lyric: Option<String>,
    pub pitch_role: Option<PitchRole>,
    pub stem_direction: Option<StemDirection>,
    pub beam_group: Option<BeamGroupId>,
    pub is_drum: bool,
    pub drum_map: Option<DrumMapEntry>,
}
```

#### Field Reference

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `pitch.midi` | u8 | 0–127 | Computed pitch |
| `velocity` | u8 | 1–127 | MIDI velocity |
| `tie` | TieSpec | enum | Tie start/stop/continue |
| `pitch_role` | PitchRole? | enum | Theory classification |
| `is_drum` | bool | | Unpitched percussion |
| `drum_map` | DrumMapEntry? | | GM drum note mapping |

#### Invariants

- I-NOTE-1: `velocity` ∈ [1, 127] unless rest-equivalent (forbidden)
- I-NOTE-2: If `is_drum`, `drum_map` must be Some
- I-NOTE-3: Tie Start at measure end must have Continue/Stop in next measure

#### Articulation Enum

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Articulation {
    Staccato, Staccatissimo, Tenuto, Accent, Marcato,
    Sforzato, Legato, Spiccato, BreathMark,
}
```

#### Ornament Enum

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Ornament {
    Trill { upper: Option<Pitch> },
    Mordent { inverted: bool },
    Turn { upper: Option<Pitch>, lower: Option<Pitch> },
    GraceNote { pitch: Pitch, steal_ratio: f32 },
    Tremolo { strokes: u8 },
}
```

#### IR Lowering

→ `IrEvent::Note` with optional ornament expansion (Phase 2: expand trills to note sequence).

---

### 8.3 ChordEvent

Simultaneous multiple pitches in one voice.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChordEvent {
    pub base: TimedEventBase,
    pub pitches: Vec<ChordTone>,
    pub velocity: u8,
    pub articulations: Vec<Articulation>,
    pub symbol: Option<ChordSymbol>,
    pub arpeggiate: Option<ArpeggiateDirection>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChordTone {
    pub pitch: Pitch,
    pub role: Option<ChordToneRole>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordToneRole {
    Root, Third, Fifth, Seventh, Ninth, Eleventh, Thirteenth, Added, Bass,
}
```

#### Invariants

- I-CHORD-1: `pitches.len()` ≥ 2
- I-CHORD-2: No duplicate MIDI numbers unless intentional (allowed for unison doubling)
- I-CHORD-3: All pitches share start offset and duration

#### IR Lowering

→ N × `IrEvent::Note` with shared `chord_group_id`.

If `arpeggiate` set, stagger start ticks by arpeggio step (parameterized).

---

### 8.4 RestEvent

Silence for notated duration.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestEvent {
    pub base: TimedEventBase,
    pub rest_type: RestType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestType {
    Normal,
    Measure,              // whole-measure rest
    MultiMeasure(u16),    // spanning measures (display)
}
```

#### Invariants

- I-REST-1: Rest does not overlap pitched events in same voice (soft: allow cross-voice)
- I-REST-2: `MultiMeasure(n)` implies duration spans n measures

#### IR Lowering

→ `IrEvent::Rest` or omitted (gap) per export mode.

---

### 8.5 MarkerEvent

Non-sounding timeline annotations.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkerEvent {
    pub base: TimedEventBase,
    pub marker: MarkerKind,
}
```

#### MarkerKind Catalog

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarkerKind {
    /// Section boundary for UI/export
    SectionBoundary {
        section_id: NodeId,
        label: Option<String>,
    },
    /// Rehearsal mark (A, B, C...)
    RehearsalMark {
        label: String,
    },
    /// Dynamic marking (p, mp, mf, f, ff, crescendo hairpin)
    DynamicMark {
        level: DynamicLevel,
        hairpin: Option<Hairpin>,
    },
    /// Display tempo text (Allegro, Adagio) — authoritative BPM in tempo_map
    TempoMark {
        text: String,
        bpm_hint: Option<f64>,
    },
    /// Articulation affecting following notes until cancelled
    ArticulationRegion {
        articulation: Articulation,
        scope: MarkerScope,
    },
    /// Figured bass figures
    FiguredBass {
        figures: String,
    },
    /// Sustain pedal
    Pedal {
        action: PedalAction,
    },
    /// Fermata (hold)
    Fermata {
        shape: FermataShape,
    },
    /// Caesura (break)
    Caesura,
    /// Cue marker
    Cue,
    /// Custom text expression
    TextExpression {
        text: String,
        placement: Placement,
    },
    /// Theme/motif label for analysis
    MotifLabel {
        motif_id: String,
    },
}
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicLevel {
    Pppp, Ppp, Pp, P, Mp, Mf, F, Ff, Fff, Ffff, Sfz, Fp, Rf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hairpin {
    Crescendo, Decrescendo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PedalAction {
    Down, Up, Half, SostenutoDown, SostenutoUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerScope {
    NextEvent, UntilNextMarker, Voice, Measure,
}
```

#### Invariants

- I-MARK-1: Point markers have `duration` = zero (whole note type with dots=0 acceptable)
- I-MARK-2: `SectionBoundary.section_id` must resolve

#### IR Lowering

→ `IrEvent::Marker` (no sound) or exporter directives (MusicXML `<direction>`).

---

### 8.6 AutomationEvent

Continuous control curves.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutomationEvent {
    pub base: TimedEventBase,
    pub target: AutomationTarget,
    pub value: f32,
    pub curve: AutomationCurve,
    pub end_value: Option<f32>,       // for ramps
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationTarget {
    Velocity,
    Expression,       // CC 11
    Modulation,       // CC 1
    Volume,           // CC 7
    Pan,              // CC 10
    PitchBend,
    TempoFactor,      // multiplier on local BPM (display/playback)
    CustomCc(u8),
}
```

#### Invariants

- I-AUTO-1: `value` normalized 0.0–1.0 for CC targets; pitch bend −1.0–1.0
- I-AUTO-2: If `end_value` set, duration > 0 (ramp)

#### IR Lowering

→ `IrControlChange` or `IrPitchBend`; ramps interpolated to step sequence (configurable resolution).

---

### 8.7 HarmonySlot (Measure-Level, Not Event)

Documented here for completeness — lives on `Measure`, not in `Event` enum.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarmonySlot {
    pub id: NodeId,
    pub offset: BeatOffset,
    pub duration: BeatOffset,
    pub symbol: ChordSymbol,
    pub roman_numeral: Option<String>,
    pub function: Option<HarmonicFunction>,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarmonicFunction {
    Tonic, Subdominant, Dominant, Predominant, Passing, Neighbor, Cadential,
}
```

---

### 8.8 DrumMapEntry

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrumMapEntry {
    pub gm_note: u8,          // General MIDI percussion key
    pub name: &'static str,   // "Snare", "Kick", etc.
}
```

Standard map in [voices.md](voices.md) §8.4.

---

## 9. Algorithms

### 9.1 Event Ordering

```text
function sort_events(voice: MeasureVoice):
    voice.events.sort_by_key(|e| e.offset())
    stable_sort ties broken by NodeId.index
```

### 9.2 Overlap Detection

```text
function check_voice_overlap(events) → Vec<Violation>:
    active = []
    for e in sorted events:
        if e is pitched (Note or Chord):
            for a in active:
                if overlaps(a, e): violations.push(...)
            active.extend(end_time filtered)
```

### 9.3 Duration Sum Check

```text
function total_duration(events) → BeatOffset:
    // max(end) not sum — polyphony allowed across voices only
    return max(e.offset + e.duration for e in events)
```

Must be ≤ measure length unless pickup/ties extend.

### 9.4 Ornament Expansion (Future)

```text
function expand_ornaments(note: NoteEvent, context) → Vec<NoteEvent>:
    match note.ornaments:
        Trill → alternating upper/aux for duration
        GraceNote → insert short note before main
```

Expansion occurs at IR projection or Decoration stage — source AST preserved.

---

## 10. Interfaces

```rust
impl Event {
    pub fn offset(&self) -> BeatOffset { self.base().offset }
    pub fn duration(&self) -> &WrittenDuration { &self.base().duration }
    pub fn provenance(&self) -> &Provenance { &self.base().provenance }
    pub fn id(&self) -> NodeId { self.base().id }
    pub fn is_pitched(&self) -> bool;
    pub fn is_sounding(&self) -> bool;
}

pub fn patch_insert_event(
    store: &mut AstStore,
    measure_id: NodeId,
    voice_id: VoiceId,
    index: usize,
    event: Event,
) -> Result<PatchId, AstError>;
```

---

## 11. Parameter Mappings

| Parameter | Event Field |
|-----------|-------------|
| `melody.ornament_density` | `NoteEvent.ornaments.len()` |
| `dynamics.dynamic_range` | `NoteEvent.velocity` spread |
| `dynamics.accent_strength` | `Articulation::Accent` frequency |
| `melody.chord_tone_bias` | `PitchRole::ChordTone` ratio |
| `drums.density` | drum `NoteEvent` count |
| `harmony.complexity` | `HarmonySlot.symbol.extensions` |
| Decoration stage | adds `Ornament` variants |

---

## 12. Explainability Model

Each event's provenance explains **why this event exists**:

| Event Type | Typical Provenance |
|------------|-------------------|
| Note (melody) | stage=Melody, rules=[melody.chord_tone, melody.stepwise] |
| Note (drums) | stage=Drums, rules=[drums.pattern_match] |
| Chord | stage=Harmony or Counterpoint |
| Rest | stage=Rhythm, rules=[rhythm.rest_placement] |
| Marker | stage=Structure or ManualEdit |
| Automation | stage=Emotion, rules=[dynamics.tension_curve] |
| HarmonySlot | stage=Harmony Skeleton |

`PitchRole` provides **theory-level explanation** independent of provenance — UI shows both.

---

## 13. Future Expansion

| Subtype | Purpose |
|---------|---------|
| `Event::Sample` | Audio sample trigger |
| `Event::Spanner` | Slur/tie as first-class (v0.2) |
| `Event::Custom` | Plugin-defined payload |
| Microtonal `Pitch` extensions | Quarter-tone |
| `ClusterEvent` | Dense chromatic cluster notation |

---

## 14. Open Questions

| ID | Question |
|----|----------|
| OQ-EVT-1 | Grace notes: separate Event or Ornament only? |
| OQ-EVT-2 | Cross-voice beamed notes — shared BeamGroupId scope? |
| OQ-EVT-3 | Maximum ornaments per note? |

---

## 15. References

- [ast.md](ast.md)
- [ir.md](ir.md)
- [timeline.md](timeline.md)
- MusicXML 4.0 `<note>`, `<direction>`, `<harmony>`
- Music21 articulations and expressions modules
- GM Drum Map (MIDI standard)

---

*End of Event Type Catalog Specification*
