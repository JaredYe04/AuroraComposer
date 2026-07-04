# Music AST Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Music AST Research Agent  
**Dependencies:** [ACAS v0.1](../00-overview/acas-v0.1.md), [terminology.md](../00-overview/terminology.md), [architecture.md](../01-architecture/architecture.md), [Deep Research Report](../../deep-research-report.md)

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

The Music AST (Abstract Syntax Tree) is the **single source of musical truth** in Aurora Composer. Every generative module, rule evaluator, search branch, UI editor, and export pipeline reads or writes the AST — never an alternate representation.

### 1.1 Problem Statement

Automatic composition requires a representation that simultaneously supports:

- **Hierarchical form** (movements, sections, phrases) for structure planning
- **Metric containment** (measures, beats) for rhythm and harmony engines
- **Polyphonic voices** with independent event streams
- **Explainability** — every generated note must carry provenance
- **Search efficiency** — immutable snapshots for beam search / A*
- **Export fidelity** — lossless projection to MusicXML, MIDI, ABC

No existing off-the-shelf library satisfies all constraints. Music21 excels at analysis but lacks provenance and generation semantics. MusicXML excels at interchange but is not algorithm-friendly. MIDI excels at playback but lacks form and theory.

### 1.2 Scope

This document specifies:

- Complete node type hierarchy with fields, types, and invariants
- Provenance metadata schema (required on every `Event`)
- AST versioning, patching, and immutability during search
- Relationship to Music21 concepts
- Cross-references to [events.md](events.md), [timeline.md](timeline.md), [voices.md](voices.md), [score.md](score.md), [ir.md](ir.md)

### 1.3 Design Principles (Binding)

From [philosophy.md](../00-overview/philosophy.md):

| Principle | AST Implication |
|-----------|-----------------|
| Everything Is Music AST | No parallel note lists; no hidden state |
| Everything Is Explainable | `Provenance` on every `Event` |
| Generation Is Search | Immutable `AstSnapshot` for search branches |
| Specification Before Code | This document is normative |

---

## 2. Existing Solutions

### 2.1 Music21

Music21 uses a universal **`Stream`** container that recursively holds `Measure`, `Note`, `Rest`, `Chord`, and nested `Stream` objects. Offset-based positioning (`element.offset`) locates items in quarter-note space.

**Relevant for Aurora:** Pitch (`Pitch`), duration (`Duration`), key (`Key`), meter (`TimeSignature`), interval/chord analysis utilities.

**Insufficient for Aurora:** No provenance, no form-aware hierarchy (Section/Phrase), no patch/undo model, Python mutability.

### 2.2 MusicXML

Partwise structure: `score-partwise` → `part` → `measure` → `note`. Multiple voices per measure use `<backup>` elements to rewind the timeline cursor.

**Relevant for Aurora:** Export target; voice numbering; `<harmony>` for chord symbols; `<direction>` for tempo/dynamics.

**Insufficient for Aurora:** Not a generation representation; XML overhead; no algorithm metadata.

### 2.3 Standard MIDI Files

Delta-time event streams per track. Channel 10 convention for drums.

**Relevant for Aurora:** IR timing model; channel mapping (see [voices.md](voices.md)).

**Insufficient for Aurora:** No form, no chord symbols, no explainability.

### 2.4 OpenMusic / PWGL

Visual algorithmic composition environments using Lisp object hierarchies (`chord-seq`, `multi-seq`).

**Relevant for Aurora:** Separation of planning (structure) from realization (notes).

**Insufficient for Aurora:** Closed systems; no standardized provenance.

### 2.5 DAW Clip Models (Lenardo, Ableton)

Track → clip → note events in absolute time.

**Relevant for Aurora:** UI timeline metaphors (see [08-ui/](../08-ui/)).

**Insufficient for Aurora:** Absolute-time-first; weak theory integration.

---

## 3. Academic / Theoretical Foundation

### 3.1 Hierarchical Musical Structure

Music theory recognizes nested organizational levels (Lerdahl & Jackendoff, 1983; Narmour, 1990):

```text
Composition → Movement → Section → Phrase → Measure → Beat → Event
```

Aurora's AST mirrors this hierarchy explicitly rather than inferring it from flat streams.

### 3.2 Tonal Hierarchy (Lerdahl)

Prolongational reduction and time-span reduction assume **events embedded in metric frames**. The `Measure` node is the primary metric container; beat offsets are rational numbers within the measure.

### 3.3 Voice Leading as Constrained Search

Species counterpoint (Fux; adapted by Salzer/Schachter) defines voice-leading rules as constraints on consecutive events across `Voice` nodes. The AST must expose voice adjacency for rule evaluation without flattening.

### 3.4 Provenance in Creative Systems

Explainable AI literature (Samek et al., 2019) and prior work in constraint-based composition (Pachet & Roy, 2001) demonstrate that **decision traces** improve user trust and debugging. Aurora treats provenance as first-class metadata, not logging.

---

## 4. Engineering Analysis

### 4.1 Evaluation Criteria

| Criterion | Requirement | AST Design Response |
|-----------|-------------|---------------------|
| Correctness | Support classical through jazz/pop forms | Full hierarchy + harmony slots |
| Controllability | Parameters map to writable fields | Explicit parameter refs in provenance |
| Explainability | Trace every event to rules | `Provenance` schema |
| Performance | Desktop beam search (<60s typical) | COW snapshots, `NodeId` indexing |
| Extensibility | Plugin patches | Patch algebra + versioned schema |
| Serialization | Tauri IPC + project files | JSON (IPC), CBOR (storage) |

### 4.2 Rust Implementation Considerations

- **`Arc<T>`** for shared immutable subtrees in snapshots
- **`im::HashMap`** or **`rpds`** for persistent maps in COW paths
- **`#[serde(tag = "type")]`** for event enum serialization
- **No `f64` for musical time** — use rational beats + tick projection
- **`thiserror`** for invariant violation errors

### 4.3 Memory Model

Typical composition (32 measures, 5 voices, ~8 events/measure):

- ~1,280 Event nodes
- ~200 structural nodes (sections, phrases, measures)
- Provenance ~200 bytes/event → ~256 KB provenance overhead
- Acceptable for in-memory generation; CBOR compression ~3× for disk

---

## 5. Comparison of Approaches

### 5.1 Representation Alternatives

| Approach | Form-Aware | Theory-Friendly | Search-Safe | Export-Easy | Verdict |
|----------|-----------|-----------------|-------------|-------------|---------|
| Flat event list | No | Low | Yes | Yes | **IR only** |
| Universal Stream (Music21) | Partial | High | No | Medium | Borrow concepts |
| MusicXML mirror | No | Medium | No | High | Export only |
| **Hierarchical AST (Aurora)** | **Yes** | **High** | **Yes** | **Medium** | **Selected** |
| Relational DB | Yes | Medium | Medium | Low | Rejected (complexity) |

### 5.2 Time Representation Alternatives

| Model | Pros | Cons | Verdict |
|-------|------|------|---------|
| Absolute seconds | Audio-native | Breaks with tempo changes | IR playback helper only |
| MIDI ticks only | Export-native | Opaque for theory modules | IR layer |
| Rational beats in measure | Theory-native, exact | Requires timeline for global position | **AST layer** |
| Floating quarter notes | Simple | Rounding errors | Rejected |

### 5.3 Mutability Alternatives

| Model | Pros | Cons | Verdict |
|-------|------|------|---------|
| In-place mutation + rollback | Simple API | Race conditions in parallel search | Rejected |
| Full deep clone per branch | Correct | O(n) per beam branch | Rejected |
| **COW snapshots (Arc)** | O(1) branch fork | Arc overhead | **Selected** |
| Persistent data structures | O(log n) updates | Learning curve | Optional optimization |

---

## 6. Recommended Solution

Aurora Composer adopts a **typed hierarchical AST** with:

1. **Form hierarchy:** `Composition` → `Movement` → `Section` → `Phrase` → `Measure` → `Voice` → `Event`
2. **Relative beat timing** within measures; global time via [timeline.md](timeline.md)
3. **Mandatory provenance** on every `Event`
4. **Copy-on-write snapshots** for search; atomic patches for commits
5. **Schema versioning** on `Composition` root
6. **Projection to IR** for export (see [ir.md](ir.md))

Single-movement pop/jazz compositions use one default `Movement`. Classical multi-movement works use multiple.

---

## 7. Architecture

### 7.1 Node Hierarchy

```text
Composition                          [root]
├── CompositionMetadata
├── GlobalAttributes                 (tempo_map, default_key, default_meter)
├── VoiceRegistry                    (voice definitions — see voices.md)
├── Movement[]
│   ├── MovementMetadata
│   └── Section[]
│       ├── SectionMetadata          (role: Verse, Chorus, Bridge, ...)
│       ├── SectionMarker[]          (optional boundary events)
│       └── Phrase[]
│           ├── PhraseMetadata       (cadence type, theme_ref)
│           └── Measure[]
│               ├── MeasureAttributes (meter change, key change, repeat)
│               ├── HarmonySlot[]     (optional chord skeleton)
│               └── Voice[]           (per-measure voice containers)
│                   └── Event[]
│                       ├── Note
│                       ├── Chord
│                       ├── Rest
│                       ├── Marker
│                       └── Automation
└── AstSchemaVersion
```

### 7.2 Layer Placement

```text
┌─────────────────────────────────────────┐
│  L3 Pipeline Engines                    │  read/write AST
├─────────────────────────────────────────┤
│  L2 Rule/Search                         │  read AstSnapshot only
├─────────────────────────────────────────┤
│  L1 Music AST  ◄── this specification   │
├─────────────────────────────────────────┤
│  L0 Export                              │  read IR (projected from AST)
└─────────────────────────────────────────┘
```

### 7.3 Identity Graph

Every node carries a stable **`NodeId`**. References between nodes (ties, theme refs, provenance parent) use `NodeId`, not pointers, enabling serialization and patch operations.

```text
NodeId = { index: u64, generation: u32 }
```

Generation increments on node reuse after deletion (avoid stale refs).

### 7.4 Module Boundaries

| Module | AST Access |
|--------|-----------|
| Structure Engine | Creates Movement/Section/Phrase/Measure skeleton |
| Harmony Engine | Writes HarmonySlot + Chord events |
| Melody Engine | Writes Note events in melody Voice |
| Rule Engine | Read-only traversal |
| Search Engine | AstSnapshot fork/merge |
| Repair Engine | Patch violating Events |
| UI Editor | Patch with ManualEdit provenance |
| Export | Read-only → IR projection |

---

## 8. Data Structures

### 8.1 Primitive Types

```rust
/// Stable node identity across patches and serialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    pub index: u64,
    pub generation: u32,
}

/// Semantic version of the AST schema stored on Composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstSchemaVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

pub const AST_SCHEMA_VERSION: AstSchemaVersion = AstSchemaVersion {
    major: 0,
    minor: 1,
    patch: 0,
};

/// Beat offset within a measure as exact rational.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BeatOffset {
    pub numer: u32,  // numerator in quarter-note fractions
    pub denom: u32,  // denominator (power of 2 preferred)
}

/// Pitch: MIDI number for computation, optional spelling for notation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pitch {
    pub midi: u8,                        // 0–127
    pub spelling: Option<PitchSpelling>, // e.g., C#4 vs Db4
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchSpelling {
    pub step: Step,       // C, D, E, F, G, A, B
    pub alter: i8,        // -2..2 (double-flat .. double-sharp)
    pub octave: i8,       // scientific pitch notation octave
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step { C, D, E, F, G, A, B }

/// Written duration for notation; ticks computed via timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrittenDuration {
    pub note_type: NoteType,     // Whole, Half, Quarter, ...
    pub dots: u8,                // 0, 1, 2
    pub tuplet: Option<TupletSpec>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteType {
    Maxima, Longa, Breve, Whole, Half, Quarter, Eighth,
    Sixteenth, ThirtySecond, SixtyFourth, OneHundredTwentyEighth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TupletSpec {
    pub actual: u8,   // e.g., 3
    pub normal: u8,   // e.g., 2
    pub normal_type: NoteType,
}
```

### 8.2 Root: Composition

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Composition {
    pub id: NodeId,
    pub schema_version: AstSchemaVersion,
    pub metadata: CompositionMetadata,
    pub global: GlobalAttributes,
    pub voice_registry: VoiceRegistry,
    pub movements: Vec<Movement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompositionMetadata {
    pub title: String,
    pub composer: Option<String>,
    pub copyright: Option<String>,
    pub created_at: String,           // ISO 8601
    pub modified_at: String,
    pub parameters_used: ParameterSnapshot,
    pub emotion_profile: Option<EmotionProfile>,
    pub provenance_root: ProvenanceRoot,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalAttributes {
    pub default_key: KeySignature,
    pub default_meter: TimeSignature,
    pub tempo_map: TempoMap,          // see timeline.md
    pub key_map: KeyMap,
    pub meter_map: MeterMap,
    pub dynamics_baseline: DynamicLevel,
}
```

See [score.md](score.md) for full metadata and project structure.

### 8.3 Movement

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Movement {
    pub id: NodeId,
    pub metadata: MovementMetadata,
    pub sections: Vec<Section>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementMetadata {
    pub title: Option<String>,        // e.g., "Allegro"
    pub ordinal: u16,                 // 1, 2, 3...
    pub key_override: Option<KeySignature>,
    pub tempo_override: Option<Bpm>,
}
```

**Invariants:**

- `ordinal` is unique within Composition
- At least one Section per Movement

### 8.4 Section

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
    pub id: NodeId,
    pub metadata: SectionMetadata,
    pub markers: Vec<SectionMarker>,
    pub phrases: Vec<Phrase>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionMetadata {
    pub role: SectionRole,
    pub label: Option<String>,        // "A", "B", "Chorus"
    pub theme_refs: Vec<ThemeRef>,
    pub key_area: Option<KeySignature>,
    pub repeat: Option<RepeatSpec>,
    pub energy_level: Option<f32>,    // 0.0–1.0, from emotion engine
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionRole {
    Intro, Verse, PreChorus, Chorus, Bridge, Breakdown, Build,
    Drop, Outro, Coda, Exposition, Development, Recapitulation,
    Transition, Interlude, Custom(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeRef {
    pub theme_id: String,
    pub transformation: ThemeTransform,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeTransform {
    Original, Sequence, Inversion, Retrograde, Augmentation,
    Diminution, Fragmentation, ModalInterchange,
}
```

**Invariants:**

- Sections are ordered; global measure numbering is computed by traversal
- `theme_refs` reference themes defined in Theme Planning stage

### 8.5 Phrase

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Phrase {
    pub id: NodeId,
    pub metadata: PhraseMetadata,
    pub measures: Vec<Measure>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhraseMetadata {
    pub phrase_id: String,
    pub cadence: Option<CadenceType>,
    pub motif_ref: Option<String>,
    pub contour_hint: Option<ContourHint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CadenceType {
    PerfectAuthentic, ImperfectAuthentic, Half, Plagal, Deceptive, Phrygian, None,
}
```

**Invariants:**

- Phrase contains 1–32 measures (typical 2–8)
- Last measure of phrase should align with cadence harmonic rhythm (soft constraint)

### 8.6 Measure

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Measure {
    pub id: NodeId,
    pub number: MeasureNumber,
    pub attributes: MeasureAttributes,
    pub harmony_slots: Vec<HarmonySlot>,
    pub voices: Vec<MeasureVoice>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureNumber {
    pub local: u16,                   // within phrase
    pub global: u32,                  // across composition
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasureAttributes {
    pub meter: Option<TimeSignature>, // if change at this measure
    pub key: Option<KeySignature>,
    pub repeat_start: bool,
    pub repeat_end: bool,
    pub repeat_count: Option<u8>,
    pub volta: Option<VoltaSpec>,
    pub rehearsal_mark: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarmonySlot {
    pub id: NodeId,
    pub offset: BeatOffset,
    pub duration: BeatOffset,
    pub symbol: ChordSymbol,
    pub roman_numeral: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasureVoice {
    pub voice_id: VoiceId,
    pub events: Vec<Event>,
}
```

**Invariants:**

- `MeasureVoice.voice_id` must exist in `VoiceRegistry`
- Sum of event durations per voice ≤ measure length (in beats), accounting for ties
- Events sorted by `offset` ascending within each voice
- No overlapping pitched events in same voice (rests may underlie — see Event spec)
- `harmony_slots` sorted by offset; durations do not exceed measure

### 8.7 Voice Registry

Voice definitions live at Composition level; per-measure `MeasureVoice` containers hold events. See [voices.md](voices.md) for allocation rules.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceRegistry {
    pub voices: Vec<VoiceDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceDef {
    pub id: VoiceId,
    pub name: String,
    pub role: VoiceRole,
    pub register: PitchRange,
    pub midi_channel: u8,             // 1–16, 10 = drums
    pub group: Option<VoiceGroupId>,
    pub instrument: InstrumentSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceRole {
    Melody, Alto, Tenor, Bass, Inner, HarmonyPad,
    Drums, Percussion, Lead, Accompaniment, Custom(u32),
}
```

### 8.8 Event (Sum Type)

All event subtypes defined in [events.md](events.md). Core enum:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Event {
    Note(NoteEvent),
    Chord(ChordEvent),
    Rest(RestEvent),
    Marker(MarkerEvent),
    Automation(AutomationEvent),
}

/// Common header for timed events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedEventBase {
    pub id: NodeId,
    pub offset: BeatOffset,
    pub duration: WrittenDuration,
    pub provenance: Provenance,       // REQUIRED — invariant I-PROV-1
}
```

#### 8.8.1 NoteEvent

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteEvent {
    pub base: TimedEventBase,
    pub pitch: Pitch,
    pub velocity: u8,                 // 1–127
    pub tie: TieSpec,
    pub articulations: Vec<Articulation>,
    pub ornaments: Vec<Ornament>,
    pub lyric: Option<String>,
    pub pitch_role: Option<PitchRole>, // ChordTone, Passing, Neighbor, ...
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TieSpec {
    None, Start, Stop, Continue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PitchRole {
    ChordTone, PassingTone, NeighborTone, Appoggiatura, Suspension,
    Retardation, EscapeTone, PedalTone, Ornament, Unclassified,
}
```

#### 8.8.2 ChordEvent

Simultaneous pitch collection in one voice slot (e.g., piano voicing).

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChordEvent {
    pub base: TimedEventBase,
    pub pitches: Vec<ChordTone>,
    pub velocity: u8,
    pub articulations: Vec<Articulation>,
    pub symbol: Option<ChordSymbol>,  // optional analytical label
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChordTone {
    pub pitch: Pitch,
    pub role: Option<ChordToneRole>,  // Root, Third, Fifth, Seventh, ...
}
```

#### 8.8.3 RestEvent

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestEvent {
    pub base: TimedEventBase,
    pub rest_type: RestType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestType {
    Normal, Measure, MultiMeasure(u16),
}
```

#### 8.8.4 MarkerEvent

Annotations not producing sound. See [events.md](events.md) for full catalog.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkerEvent {
    pub base: TimedEventBase,
    pub marker: MarkerKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarkerKind {
    SectionBoundary { section_id: NodeId },
    RehearsalMark { label: String },
    DynamicMark { level: DynamicLevel },
    TempoMark { text: String },       // display only; tempo_map is authoritative
    ArticulationRegion { articulation: Articulation },
    FiguredBass { figures: String },
    Pedal { action: PedalAction },
    Cue, Fermata, Caesura,
}
```

#### 8.8.5 AutomationEvent

Continuous control curves (exporter projects to MIDI CC or MusicXML `<sound>`).

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutomationEvent {
    pub base: TimedEventBase,
    pub target: AutomationTarget,
    pub value: f32,
    pub curve: AutomationCurve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationTarget {
    Velocity, Expression, Modulation, Volume, Pan,
    PitchBend, TempoFactor, CustomCc(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationCurve {
    Step, Linear, Exponential, Logarithmic,
}
```

### 8.9 Provenance Schema

**Invariant I-PROV-1:** Every `Event` MUST have non-empty `Provenance` with `source` and `created_at`.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub source: ProvenanceSource,
    pub stage: Option<PipelineStageId>,
    pub rule_ids: Vec<RuleId>,
    pub eval_score: Option<f64>,
    pub search: Option<SearchContext>,
    pub parent: Option<ProvenanceRef>,
    pub created_at: String,           // ISO 8601
    pub agent: ProvenanceAgent,
    pub parameters_hash: Option<String>,
    pub explanation: Option<String>,  // human-readable summary
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvenanceSource {
    Generated,
    ManualEdit,
    Imported,
    Repaired,
    Plugin,
    Transformed,                      // theme transform, e.g., inversion
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchContext {
    pub step_index: u32,
    pub beam_rank: u16,
    pub beam_width: u16,
    pub state_ref: StateRef,
    pub accumulated_score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceRef {
    pub node_id: NodeId,
    pub patch_id: Option<PatchId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProvenanceAgent {
    Engine { stage: PipelineStageId },
    User { user_id: Option<String> },
    Plugin { plugin_id: String },
    Import { format: String },
}
```

**ProvenanceRoot** on Composition aggregates generation session info:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceRoot {
    pub session_id: String,
    pub generator_version: String,
    pub seed: Option<u64>,
    pub pipeline_config_hash: String,
}
```

### 8.10 Theory Types (Shared)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySignature {
    pub tonic: PitchClass,
    pub mode: Mode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchClass {
    pub pc: u8,  // 0–11
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Major, NaturalMinor, HarmonicMinor, MelodicMinor,
    Dorian, Phrygian, Lydian, Mixolydian, Locrian,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    pub beats: u8,
    pub beat_type: u8,  // denominator (4 = quarter)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChordSymbol {
    pub root: PitchClass,
    pub quality: ChordQuality,
    pub extensions: Vec<Extension>,
    pub bass: Option<PitchClass>,
    pub raw: String,                  // original symbol, e.g., "Am7/G"
}
```

### 8.11 Complete Invariant Registry

| ID | Scope | Rule |
|----|-------|------|
| I-PROV-1 | Event | Provenance required |
| I-EVT-1 | MeasureVoice | Events sorted by offset |
| I-EVT-2 | MeasureVoice | No pitch overlap (except chord tones within ChordEvent) |
| I-MSR-1 | Measure | Voice durations ≤ measure length |
| I-MSR-2 | Measure | harmony_slots within measure bounds |
| I-NID-1 | Global | NodeId references resolve |
| I-VOC-1 | MeasureVoice | voice_id ∈ VoiceRegistry |
| I-TIE-1 | Note | Tie Start must pair with Stop/Continue same pitch |
| I-KEY-1 | Global | key at beat derivable from key_map |
| I-MTR-1 | Global | meter at measure derivable from meter_map |
| I-VER-1 | Composition | schema_version supported by engine |

---

## 9. Algorithms

### 9.1 AST Construction (Structure Planning)

```text
function build_skeleton(params) → Composition:
    comp = empty Composition with schema_version
    comp.voice_registry = allocate_voices(params.voice.*)
    comp.global = default GlobalAttributes from params.mode.*
    movement = single Movement
    for section_plan in params.form.sections:
        section = Section with role, theme_refs
        for phrase_plan in section_plan.phrases:
            phrase = Phrase with measure_count
            for i in 0..phrase_plan.length:
                measure = empty Measure with global number
                for voice in comp.voice_registry:
                    measure.voices.push(empty MeasureVoice)
            section.phrases.push(phrase)
        movement.sections.push(section)
    comp.movements.push(movement)
    return comp
```

### 9.2 Traversal Algorithms

**Depth-first visitor** for rule engine:

```rust
pub trait AstVisitor {
    fn visit_composition(&mut self, node: &Composition);
    fn visit_movement(&mut self, node: &Movement);
    fn visit_section(&mut self, node: &Section);
    fn visit_phrase(&mut self, node: &Phrase);
    fn visit_measure(&mut self, node: &Measure);
    fn visit_event(&mut self, event: &Event, ctx: &EventContext);
}

pub struct EventContext {
    pub voice_id: VoiceId,
    pub measure_id: NodeId,
    pub global_measure: u32,
    pub beat_offset: BeatOffset,
    pub current_key: KeySignature,
    pub current_meter: TimeSignature,
}
```

**Event iterator** (flattened with context, read-only):

```rust
pub fn iter_events(comp: &Composition) -> impl Iterator<Item = (EventContext, &Event)>;
```

### 9.3 Copy-on-Write Snapshot

```rust
pub struct AstSnapshot {
    inner: Arc<Composition>,
    overlay: im::HashMap<NodeId, PatchOverlay>,
}

impl AstSnapshot {
    pub fn fork(&self) -> Self { /* Arc clone — O(1) */ }

    pub fn insert_event(&mut self, measure_id: NodeId, voice_id: VoiceId, event: Event) {
        // COW: clone path from root to measure if not unique
    }
}
```

Search beam maintains `Vec<AstSnapshot>`. On commit, winning snapshot diff → `Patch` applied to canonical `Composition`.

### 9.4 Patch Algebra

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Patch {
    pub id: PatchId,
    pub ops: Vec<PatchOp>,
    pub inverse: Option<PatchId>,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    InsertNode { parent: NodeId, index: usize, node: AstNodePayload },
    DeleteNode { node_id: NodeId },
    ReplaceNode { node_id: NodeId, node: AstNodePayload },
    MoveNode { node_id: NodeId, new_parent: NodeId, new_index: usize },
    UpdateField { node_id: NodeId, path: FieldPath, value: JsonValue },
}
```

**Properties:**

- Patches are **atomic** — all ops succeed or none apply
- **Inverse patch** computed for undo
- Validation runs post-apply (invariant checker)

```text
function apply_patch(comp, patch) → Result<Composition>:
    snapshot = comp.clone()
    for op in patch.ops:
        snapshot = apply_op(snapshot, op)
    validate_invariants(snapshot)?
    return snapshot
```

### 9.5 Invariant Validation

```text
function validate(comp) → Vec<Violation>:
    violations = []
    for measure in all_measures(comp):
        meter = resolve_meter(comp.global.meter_map, measure.number)
        for mv in measure.voices:
            violations.extend(check_event_ordering(mv))
            violations.extend(check_duration_sum(mv, meter))
            violations.extend(check_provenance(mv))
    violations.extend(check_tie_consistency(comp))
    return violations
```

Hard violations block export; soft violations route to Repair stage.

### 9.6 Global Measure Numbering

```text
function compute_global_numbers(comp):
    n = 1
    for movement in comp.movements:
        for section in movement.sections:
            for phrase in section.phrases:
                for measure in phrase.measures:
                    measure.number.global = n
                    measure.number.local = index_in_phrase
                    n += 1
```

---

## 10. Interfaces

### 10.1 Rust Core API (Specification)

```rust
pub struct AstStore {
    composition: Composition,
    history: Vec<Patch>,
    history_index: usize,
}

impl AstStore {
    pub fn root(&self) -> &Composition;
    pub fn snapshot(&self) -> AstSnapshot;
    pub fn apply(&mut self, patch: Patch) -> Result<(), AstError>;
    pub fn undo(&mut self) -> Result<(), AstError>;
    pub fn redo(&mut self) -> Result<(), AstError>;
    pub fn validate(&self) -> ValidationReport;
}
```

### 10.2 Tauri IPC Commands

| Command | Input | Output |
|---------|-------|--------|
| `get_composition` | `project_id` | `Composition` JSON |
| `apply_patch` | `project_id`, `Patch` | `ValidationReport` |
| `get_event_provenance` | `node_id` | `Provenance` chain |
| `validate_ast` | `project_id` | `ValidationReport` |
| `project_to_ir` | `project_id` | `MusicIr` |

### 10.3 Plugin Interface

Plugins receive `AstSnapshot` (read) and return `Patch` (write):

```rust
pub trait AstPlugin {
    fn id(&self) -> &str;
    fn read_set(&self) -> AstReadSet;   // which node types accessed
    fn write_set(&self) -> AstWriteSet;
    fn apply(&self, snapshot: &AstSnapshot, params: &Value) -> Result<Patch, PluginError>;
}
```

### 10.4 Rule Engine Interface

```rust
pub trait RuleContext {
    fn composition(&self) -> &Composition;
    fn event_context(&self) -> &EventContext;
    fn key_at(&self, global_measure: u32, offset: BeatOffset) -> KeySignature;
    fn chord_at(&self, global_measure: u32, offset: BeatOffset) -> Option<&HarmonySlot>;
    fn previous_event(&self) -> Option<&Event>;
    fn parallel_voice_events(&self) -> impl Iterator<Item = &Event>;
}
```

---

## 11. Parameter Mappings

User parameters from ACAS §6 map to AST fields:

| Parameter | AST Target | Stage |
|-----------|-----------|-------|
| `form.section_count` | `Movement.sections.len()` | Structure |
| `form.section_lengths` | `Phrase.measures.len()` per section | Structure |
| `mode.key` | `GlobalAttributes.default_key` | Structure |
| `mode.mode` | `KeySignature.mode` | Structure |
| `voice.voice_count` | `VoiceRegistry.voices.len()` | Structure |
| `register.melody_register` | `VoiceDef[M].register` | Structure |
| `register.bass_register` | `VoiceDef[B].register` | Structure |
| `rhythm.subdivision` | min `WrittenDuration` granularity | Rhythm |
| `harmony.complexity` | `HarmonySlot.symbol` extensions | Harmony |
| `melody.*` | `NoteEvent.pitch_role`, ornament density | Melody |
| `counterpoint.strictness` | rule weights (not AST field) | Counterpoint |
| `drums.density` | drum `NoteEvent` count per measure | Drums |
| `dynamics.dynamic_range` | `NoteEvent.velocity` spread | Melody+ |
| `theme.count` | `SectionMetadata.theme_refs` | Theme Planning |
| `search.beam_width` | `Provenance.search.beam_width` | Search |

---

## 12. Explainability Model

### 12.1 Provenance Chain

Every generated event traces:

```text
User Parameter → Pipeline Stage → Rule IDs → Search Step → Event
```

UI Inspector renders:

1. **Summary:** "Generated by Melody stage, step 42, beam rank 2"
2. **Rules:** clickable list of contributing rules with individual scores
3. **Parameters:** snapshot of relevant params at generation
4. **Parent:** link to replaced event (if Repair) or motif source (if Transformed)

### 12.2 Aggregation

For dense passages, UI may aggregate:

```rust
pub struct ProvenanceSummary {
    pub event_count: u32,
    pub dominant_stage: PipelineStageId,
    pub rule_frequency: HashMap<RuleId, u32>,
}
```

Aggregation is **display-only**; full provenance retained in AST.

### 12.3 Manual Edits

User edits via Piano Roll produce:

```rust
Provenance {
    source: ManualEdit,
    stage: None,
    agent: ProvenanceAgent::User { user_id: ... },
    parent: Some(ProvenanceRef { node_id: original_event, ... }),
    explanation: Some("User changed pitch C4 → D4"),
    ...
}
```

---

## 13. Future Expansion

| Feature | AST Impact | Target Version |
|---------|-----------|----------------|
| Spanners (slur, tie across voices) | `SpannerRef` on events | 0.2 |
| Linked parts (transposing instruments) | `PartLink` in VoiceRegistry | 0.3 |
| Microtonal pitch | `Pitch` extension | 0.3 |
| Audio clip references | `SampleEvent` subtype | 0.4 |
| CRDT collaborative editing | Operation-based patches | 1.0 |
| Nested tuplets >2 deep | `TupletSpec` nesting | 0.2 |
| Custom event plugins | `Event::Custom` variant | 0.2 |

---

## 14. Open Questions

| ID | Question | Status |
|----|----------|--------|
| OQ-AST-1 | Should `HarmonySlot` be per-measure or per-phrase? | Per-measure (current); phrase-level aggregation is derived |
| OQ-AST-2 | Adopt `im` crate vs custom COW? | Decide in Phase 2 prototype |
| OQ-AST-3 | Binary project format: CBOR vs MessagePack? | CBOR preferred; benchmark pending |
| OQ-AST-4 | Maximum composition size limits? | Propose 10k measures, 32 voices |
| OQ-AST-5 | ADR: Immutable AST during search | Proposed — see `decisions/` |

---

## 15. References

### Internal

- [ACAS v0.1](../00-overview/acas-v0.1.md)
- [Terminology](../00-overview/terminology.md)
- [Architecture](../01-architecture/architecture.md)
- [Pipeline](../01-architecture/pipeline.md)
- [events.md](events.md)
- [timeline.md](timeline.md)
- [voices.md](voices.md)
- [score.md](score.md)
- [ir.md](ir.md)
- [Raw research notes](../../research/ast-research-notes.md)

### External

- Music21 Documentation (v9): https://web.mit.edu/music21/doc/
- MusicXML 4.0: https://www.w3.org/2021/06/musicxml40/
- Lerdahl & Jackendoff (1983), *A Generative Theory of Tonal Music*
- Pachet & Roy (2001), *Musical Data Mining*
- Huron (2002), *Music Information Processing Using the Humdrum Toolkit*
- Samek et al. (2019), explainable AI survey

---

## Appendix A: Music21 Concept Mapping

| Music21 | Aurora AST | Conversion Notes |
|---------|-----------|------------------|
| `stream.Score` | `Composition` | Import creates Movements from implicit structure |
| `stream.Opus` | `Composition` with multiple Movements | |
| `stream.Part` | `VoiceDef` + per-measure events | Part-wide streams split by measure |
| `stream.Measure` | `Measure` | Offsets → BeatOffset |
| `note.Note` | `Event::Note` | `.pitch` → `Pitch`, `.quarterLength` → `WrittenDuration` |
| `note.Rest` | `Event::Rest` | |
| `chord.Chord` | `Event::Chord` | |
| `harmony.ChordSymbol` | `HarmonySlot` | |
| `tempo.MetronomeMark` | `TempoMap` entry | Not an Event |
| `meter.TimeSignature` | `MeterMap` entry | |
| `key.Key` | `KeyMap` entry | |
| `expressions.*` | `MarkerEvent` variants | |
| `spanner.Slur` | Future `SpannerRef` | v0.2 |

## Appendix B: JSON Serialization Example (Abbreviated)

```json
{
  "schema_version": { "major": 0, "minor": 1, "patch": 0 },
  "metadata": { "title": "Example", "provenance_root": { "session_id": "..." } },
  "global": { "default_key": { "tonic": { "pc": 0 }, "mode": "Major" } },
  "movements": [{
    "sections": [{
      "metadata": { "role": "Verse", "label": "A" },
      "phrases": [{
        "measures": [{
          "number": { "local": 1, "global": 1 },
          "voices": [{
            "voice_id": 0,
            "events": [{
              "kind": "Note",
              "base": {
                "offset": { "numer": 0, "denom": 1 },
                "provenance": { "source": "Generated", "stage": "Melody" }
              },
              "pitch": { "midi": 60 }
            }]
          }]
        }]
      }]
    }]
  }]
}
```

---

*End of Music AST Specification*
