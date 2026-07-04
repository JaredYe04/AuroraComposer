# Music IR Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Music AST Research Agent  
**Dependencies:** [ast.md](ast.md), [timeline.md](timeline.md), [events.md](events.md), [voices.md](voices.md), [score.md](score.md)

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

The Music IR (Intermediate Representation) is a **flattened, time-ordered, export-optimized projection** of the Music AST. Generation modules **must not** read or write IR. Exporters, audio playback, and timeline rendering consume IR exclusively.

### 1.1 Purpose

| Need | AST | IR |
|------|-----|-----|
| Form-aware generation | ✓ | ✗ |
| Rule evaluation on hierarchy | ✓ | Partial (via indices) |
| Explainability (full provenance) | ✓ | Summary only |
| MIDI tick timing | Via projection | ✓ |
| Sorted event streams per channel | ✗ | ✓ |
| MusicXML `<backup>`-free voice timing | ✗ | ✓ (pre-computed) |
| Real-time playback scheduling | Slow (traverse) | ✓ |

### 1.2 Compiler Analogy

```text
Source (AST)  →  IR (SSA-like flat events)  →  Backend (MIDI/MusicXML/ABC)
```

IR is rebuilt on every export or preview — it is **derived state**, never authoritative.

---

## 2. Existing Solutions

### 2.1 MIDI File Structure

SMF Type 1: header + track chunks. Each track: `(delta_time, event)*`. Running status for efficiency.

**Adopted:** Tick-based timing, per-track (channel) organization, delta-time encoding in export.

**Not adopted:** IR is not identical to SMF — it retains AST node references and optional provenance summaries.

### 2.2 MusicXML Partwise Export

Export iterates measures, voices, notes. `<backup>` rewinds cursor for polyphony.

**Adopted:** Voice-to-staff mapping semantics.

**Not adopted:** IR pre-flattens so exporters need not compute backup durations.

### 2.3 LLVM IR / WASM

Industry pattern: high-level AST lowered to flat instruction stream with explicit types and ordering.

**Adopted:** Single-pass projection with validation pass.

### 2.4 Web Audio Scheduling

`AudioContext.currentTime` + `schedule(time, event)` requires absolute seconds.

**Adopted:** IR includes both tick and second timestamps (computed from tempo map).

---

## 3. Academic / Theoretical Foundation

### 3.1 Multi-Representation Music Systems

Huron (2002) distinguishes **logical** (theory), **notational** (score), and **acoustic** (signal) representations. Aurora's AST is logical+notational; IR bridges to acoustic via MIDI playback.

### 3.2 Temporal Media Alignment

Alignment literature (Page et al., 2005) emphasizes **single timeline coordinates** for multimedia sync. IR provides the canonical timeline coordinate (`tick`, `seconds`).

---

## 4. Engineering Analysis

### 4.1 Projection Cost

Typical composition: 32 measures × 5 voices × 8 events = 1,280 events.

- Traversal: O(n)
- Tick computation per event: O(log t) for tempo map lookup (t = tempo segments)
- Sort per channel: O(k log k) where k = events per channel
- Total: <10ms on desktop — acceptable for interactive export

### 4.2 Immutability

IR is **immutable** once built. Re-projection on AST patch is cheap; exporters hold `Arc<MusicIr>`.

### 4.3 Memory Layout

Flat `Vec<IrEvent>` per channel — cache-friendly for playback iterators.

---

## 5. Comparison of Approaches

| Approach | Export Speed | Playback | Provenance | Verdict |
|----------|-------------|----------|------------|---------|
| Export directly from AST | Slow (recompute each format) | Slow | Full | Rejected |
| Cache IR per channel | Fast | Fast | Summary | **Selected** |
| Cache per format (MIDI bytes) | Fastest export | N/A | None | Cache layer above IR |
| Single global event list | Simple | Channel filter needed | Summary | Used as master list + channel index |

---

## 6. Recommended Solution

**MusicIr** structure:

1. **`header`** — PPQ, duration ticks, metadata snapshot
2. **`timeline`** — resolved tempo/meter/key at tick boundaries (from [timeline.md](timeline.md))
3. **`events`** — global time-ordered `IrEvent` list
4. **`channels`** — per-channel views (indices into `events`)
5. **`voice_map`** — VoiceId → channel mapping
6. **`provenance_index`** — optional NodeId → ProvenanceSummary

Projection is **deterministic**: same AST + same PPQ → identical IR.

---

## 7. Architecture

```text
                    ┌──────────────┐
                    │  Music AST   │
                    └──────┬───────┘
                           │
              ┌────────────▼────────────┐
              │   Timeline Resolver     │  tempo/meter/key → tick
              └────────────┬────────────┘
                           │
              ┌────────────▼────────────┐
              │   IR Projector          │  AST events → IrEvent
              └────────────┬────────────┘
                           │
              ┌────────────▼────────────┐
              │   IR Validator          │  monotonic ticks, no overlap
              └────────────┬────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
    MIDI Exporter    MusicXML Exporter   Audio Scheduler
```

### 7.1 Projection Rules (Summary)

| AST Source | IR Output |
|------------|-----------|
| `NoteEvent` | `IrEvent::Note` with start_tick, duration_ticks |
| `ChordEvent` | Multiple `IrEvent::Note` (same start, shared chord_id) |
| `RestEvent` | `IrEvent::Rest` (or silence gap — exporter choice) |
| `MarkerEvent` | `IrEvent::Marker` |
| `AutomationEvent` | `IrEvent::ControlChange` or `IrEvent::PitchBend` |
| `HarmonySlot` | `IrEvent::Harmony` (analytical, no sound) |
| Tied notes | Merged or linked via `tie_group_id` |

Full rules in §9.

---

## 8. Data Structures

### 8.1 MusicIr Root

```rust
pub const DEFAULT_PPQ: u16 = 480;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MusicIr {
    pub header: IrHeader,
    pub timeline: ResolvedTimeline,
    pub events: Vec<IrEvent>,
    pub channel_index: Vec<ChannelView>,
    pub voice_map: HashMap<VoiceId, ChannelId>,
    pub provenance_index: HashMap<NodeId, ProvenanceSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrHeader {
    pub ppq: u16,
    pub total_ticks: u64,
    pub total_seconds: f64,
    pub title: String,
    pub ast_schema_version: AstSchemaVersion,
    pub projection_version: u16,       // IR schema version
    pub projected_at: String,
}
```

### 8.2 IrEvent

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrEvent {
    Note(IrNote),
    Rest(IrRest),
    ControlChange(IrControlChange),
    PitchBend(IrPitchBend),
    ProgramChange(IrProgramChange),
    Marker(IrMarker),
    Harmony(IrHarmony),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrEventBase {
    pub tick: u64,                    // absolute start tick
    pub channel: ChannelId,
    pub voice_id: VoiceId,
    pub ast_node_id: NodeId,          // back-reference to AST
    pub provenance_summary: Option<ProvenanceSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrNote {
    pub base: IrEventBase,
    pub midi: u8,
    pub velocity: u8,
    pub duration_ticks: u32,
    pub end_tick: u64,
    pub seconds_start: f64,
    pub seconds_end: f64,
    pub tie_group_id: Option<u64>,
    pub chord_group_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrRest {
    pub base: IrEventBase,
    pub duration_ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrControlChange {
    pub base: IrEventBase,
    pub controller: u8,
    pub value: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrHarmony {
    pub base: IrEventBase,
    pub symbol: String,
    pub duration_ticks: u32,
}
```

### 8.3 Channel View

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u8);  // 0–15

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelView {
    pub channel: ChannelId,
    pub voice_id: VoiceId,
    pub program: u8,                  // GM program number
    pub event_indices: Vec<usize>,    // indices into MusicIr.events
    pub is_drum: bool,
}
```

### 8.4 Resolved Timeline (Embedded)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedTimeline {
    pub tempo_events: Vec<IrTempoEvent>,
    pub meter_events: Vec<IrMeterEvent>,
    pub key_events: Vec<IrKeyEvent>,
    pub tick_to_beat: BeatGrid,       // see timeline.md
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrTempoEvent {
    pub tick: u64,
    pub bpm: f64,
    pub ramp: Option<RampKind>,
}
```

---

## 9. Algorithms

### 9.1 Projection Pipeline

```text
function project_ast_to_ir(comp: Composition, ppq: u16) → MusicIr:
    timeline = resolve_timeline(comp.global, comp, ppq)
    events = []
    for (ctx, event) in iter_events(comp):
        tick = beat_to_tick(ctx.global_measure, ctx.beat_offset, timeline)
        ir_events = lower_event(event, ctx, tick, timeline, ppq)
        events.extend(ir_events)
    events.sort_by_key(|e| (e.tick(), e.channel()))
    channel_index = build_channel_index(events, comp.voice_registry)
    validate_ir(events)
    return MusicIr { header, timeline, events, channel_index, ... }
```

### 9.2 Beat-to-Tick Conversion

```text
function beat_to_tick(global_measure, beat_offset, timeline) → u64:
    measure_start_tick = timeline.measure_start_ticks[global_measure]
    meter = timeline.meter_at(global_measure)
    beats_per_measure = meter.beats * 4 / meter.beat_type
    beat_fraction = beat_offset.as_quarters()
    quarter_ticks = timeline.ppq * 4  // per whole note
    return measure_start_tick + (beat_fraction * quarter_ticks) as u64
```

Tempo changes between beats use segment integration (see [timeline.md](timeline.md) §9.3).

### 9.3 Event Lowering Rules

#### NoteEvent → IrNote

```text
duration_ticks = written_duration_to_ticks(event.duration, meter, ppq)
end_tick = start_tick + duration_ticks
seconds = tick_to_seconds(start_tick, timeline.tempo_events)
```

#### ChordEvent → Multiple IrNotes

All pitches share `chord_group_id = allocate_id()`.

#### Tie Handling

```text
function process_ties(notes: Vec<IrNote>) → Vec<IrNote>:
    for chain in find_tie_chains(notes):
        if export_mode == MidiMergeTies:
            merge chain into single note (duration sum)
        else:
            assign shared tie_group_id
```

MusicXML export prefers separate tied notes; MIDI export may merge.

#### RestEvent → IrRest

Rests produce explicit rest events OR gaps (configurable). Default: explicit `IrRest` for notation exporters.

#### MarkerEvent → IrMarker

Non-sounding; `duration_ticks = 0`.

#### AutomationEvent → IrControlChange / IrPitchBend

Map `AutomationTarget` to MIDI CC:

| Target | CC |
|--------|-----|
| Volume | 7 |
| Expression | 11 |
| Modulation | 1 |
| Pan | 10 |
| CustomCc(n) | n |

#### HarmonySlot → IrHarmony

Projected from measure harmony slots (not voice events). One `IrHarmony` per slot.

### 9.4 Channel Assignment

```text
function assign_channel(voice_id, registry) → ChannelId:
    return ChannelId(registry[voice_id].midi_channel - 1)  // 0-indexed
```

Drums: channel 9 (0-indexed) = MIDI channel 10.

### 9.5 IR Validation

| Check | Action |
|-------|--------|
| Events sorted by (tick, channel) | Error if violated |
| No note overlap same channel (unless chord) | Warning |
| end_tick ≤ total_ticks | Error |
| All ast_node_id resolve | Error |
| Drum channel notes unpitched or GM map | Warning |

### 9.6 Tick to Seconds

```text
function tick_to_seconds(tick, tempo_events) → f64:
    seconds = 0.0
    prev_tick = 0
    for segment in tempo_events up to tick:
        dt_ticks = min(tick, segment.end_tick) - prev_tick
        seconds += dt_ticks / (ppq * segment.bpm / 60.0)
        prev_tick = segment.end_tick
    return seconds
```

---

## 10. Interfaces

### 10.1 Projector API

```rust
pub struct IrProjector {
    pub ppq: u16,
    pub tie_mode: TieExportMode,
    pub rest_mode: RestExportMode,
}

impl IrProjector {
    pub fn project(&self, comp: &Composition) -> Result<MusicIr, IrError>;
}

pub enum TieExportMode { Separate, Merged }
pub enum RestExportMode { Explicit, GapOnly }
```

### 10.2 Exporter Input

```rust
pub trait IrExporter {
    fn export(&self, ir: &MusicIr) -> Result<Vec<u8>, ExportError>;
}

// Implementations: MidiExporter, MusicXmlExporter, AbcExporter
```

### 10.3 Playback Scheduler

```rust
pub struct PlaybackSchedule {
    pub events: Vec<ScheduledNote>,  // seconds-based
}

pub fn schedule_playback(ir: &MusicIr, start_seconds: f64) -> PlaybackSchedule;
```

### 10.4 Tauri IPC

| Command | Returns |
|---------|---------|
| `project_ir` | `MusicIr` JSON |
| `export_midi` | binary (projects IR internally) |
| `get_playback_schedule` | `PlaybackSchedule` |

---

## 11. Parameter Mappings

| Parameter | IR Effect |
|-----------|-----------|
| `export.ppq` | `IrHeader.ppq` (default 480) |
| `export.merge_ties_midi` | `TieExportMode::Merged` |
| `voice.*` | channel assignment via voice_map |
| `style.orchestration_preset` | `ChannelView.program` (GM numbers) |
| `rhythm.swing` | tick adjustment during projection (swing grid) |

---

## 12. Explainability Model

IR carries **ProvenanceSummary** only — not full provenance chain:

```rust
pub struct ProvenanceSummary {
    pub source: ProvenanceSource,
    pub stage: Option<PipelineStageId>,
    pub top_rule_id: Option<RuleId>,
    pub eval_score: Option<f64>,
}
```

Full provenance remains in AST. UI Event Inspector reads AST directly by `ast_node_id`.

Export logs may include optional `X-Aurora-Provenance` metadata in MusicXML `<miscellaneous>`.

---

## 13. Future Expansion

| Feature | IR Extension |
|---------|-------------|
| Audio sample events | `IrEvent::Sample { path, ... }` |
| Surround pan | `IrControlChange` multi-channel |
| MPE (MIDI Poly Expression) | per-note CC channels |
| Microtonal | `IrNote::pitch_bend_cents` |
| Compressed IR cache | binary format on disk |

---

## 14. Open Questions

| ID | Question |
|----|----------|
| OQ-IR-1 | Swing quantization: AST attribute vs IR projection parameter? |
| OQ-IR-2 | Cache IR in project file or always re-project? |
| OQ-IR-3 | Separate IR for notation vs playback (different tie handling)? |

---

## 15. References

- [ast.md](ast.md)
- [timeline.md](timeline.md)
- [events.md](events.md)
- [voices.md](voices.md)
- SMF Specification
- MusicXML 4.0
- [Deep Research Report §4](../../deep-research-report.md)

---

*End of Music IR Specification*
