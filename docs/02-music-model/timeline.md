# Timeline Model Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Music AST Research Agent  
**Dependencies:** [ast.md](ast.md), [ir.md](ir.md), [score.md](score.md)

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

The **Timeline** is Aurora's temporal coordinate system. It bridges:

- **AST beat offsets** (rational, measure-relative) — used by generation engines
- **IR/MIDI ticks** (integer, absolute) — used by export and playback
- **Wall-clock seconds** — used by audio preview

The timeline model specifies **tempo maps**, **meter changes**, **key maps**, and the **beat grid** that unify these coordinate systems.

---

## 2. Existing Solutions

### 2.1 DAW Tempo Maps

Ableton, Logic: piecewise tempo with ramp curves; absolute bar/beats display.

**Adopted:** Segment-based tempo map with optional ramps.

### 2.2 Music21 `MetronomeMark` + offsets

Tempo marks stored as stream elements at quarter-note offsets.

**Adopted:** Beat-position indexing.

**Adapted:** Tempo stored in `GlobalAttributes.tempo_map`, not as Events (avoids duplication with MarkerEvent display marks).

### 2.3 MIDI Set Tempo Meta Events

Microseconds per quarter note at tick positions.

**Adopted:** IR projection emits `IrTempoEvent` equivalent.

### 2.4 MusicXML `<sound tempo="...">`

Per-measure or per-note tempo directives.

**Adopted:** Export from tempo map.

---

## 3. Academic / Theoretical Foundation

### 3.1 Metric Hierarchy (Lerdahl & Jackendoff)

Beat strength varies by metrical level. The beat grid encodes **strong/weak** classification for rhythm rules.

### 3.2 Tempo Rubato

Expressive tempo fluctuation modeled as continuous ramp segments between anchor BPM values — not random jitter.

### 3.3 Harmonic Rhythm vs Metric Grid

Harmony slots align to beat grid subdivisions. Timeline provides `snap_to_grid(beat, subdivision)`.

---

## 4. Engineering Analysis

### 4.1 Precision Requirements

- **No floating-point beats in AST** — use `BeatOffset { numer, denom }`
- Tempo BPM as `f64` acceptable (display/export only)
- Tick arithmetic uses `u64` — supports >24h compositions at 480 PPQ

### 4.2 Lookup Performance

Tempo/meter/key maps: sorted vectors + binary search O(log n). Typical n < 100 segments.

### 4.3 Measure Start Tick Cache

Precomputed `measure_start_ticks: Vec<u64>` during timeline resolution — O(1) measure lookup.

---

## 5. Comparison of Approaches

| Model | Tempo Changes | Meter Changes | Theory Fit | Verdict |
|-------|--------------|---------------|------------|---------|
| Absolute seconds only | Continuous | Awkward | Low | Playback helper |
| Tick-only | Step/ramp | OK | Medium | IR layer |
| **Beat + maps** | Segment map | Per-measure | High | **AST layer** |
| Embedded in each Event | Redundant | Redundant | Low | Rejected |

---

## 6. Recommended Solution

Store on `Composition.global`:

- **`tempo_map`** — BPM segments keyed by global beat position
- **`meter_map`** — time signature changes keyed by global measure number
- **`key_map`** — key changes keyed by global measure + beat

Provide **TimelineResolver** that materializes:

- Beat grid with strong/weak flags
- Measure boundary tick positions
- Key/meter/tempo at arbitrary query points

---

## 7. Architecture

```text
GlobalAttributes
├── tempo_map ──────┐
├── meter_map ──────┼──► TimelineResolver ──► ResolvedTimeline
├── key_map ────────┘           │
                                ├── BeatGrid
                                ├── measure_start_ticks[]
                                └── tick ↔ beat functions
                                        │
                                        ▼
                                  IR Projector / UI Timeline
```

---

## 8. Data Structures

### 8.1 TempoMap

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempoMap {
    pub default_bpm: f64,
    pub segments: Vec<TempoSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempoSegment {
    pub start: TimelinePosition,      // where segment begins
    pub bpm: f64,
    pub ramp: Option<TempoRamp>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempoRamp {
    pub end: TimelinePosition,
    pub end_bpm: f64,
    pub curve: RampCurve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RampCurve {
    Linear, EaseIn, EaseOut, EaseInOut, Exponential,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimelinePosition {
    pub global_measure: u32,         // 1-based
    pub beat: BeatOffset,              // offset within measure
}
```

**Invariant:** Segments sorted by `start`; no overlaps; first segment starts at measure 1, beat 0.

### 8.2 MeterMap

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeterMap {
    pub default: TimeSignature,
    pub changes: Vec<MeterChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeterChange {
    pub at_measure: u32,               // global measure number
    pub meter: TimeSignature,
}
```

### 8.3 KeyMap

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyMap {
    pub default: KeySignature,
    pub changes: Vec<KeyChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyChange {
    pub at: TimelinePosition,
    pub key: KeySignature,
}
```

### 8.4 BeatGrid

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeatGrid {
    pub subdivisions_per_beat: u8,     // e.g., 4 = sixteenth notes in 4/4
    pub measures: Vec<MeasureGrid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasureGrid {
    pub global_measure: u32,
    pub meter: TimeSignature,
    pub beats: Vec<BeatCell>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeatCell {
    pub beat_index: u8,                // 0-based within measure
    pub offset: BeatOffset,
    pub strength: BeatStrength,
    pub subdivisions: Vec<SubdivisionCell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeatStrength {
    Downbeat, Strong, Medium, Weak, Offbeat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubdivisionCell {
    pub offset: BeatOffset,
    pub strength: BeatStrength,
}
```

### 8.5 ResolvedTimeline

```rust
#[derive(Clone, Debug)]
pub struct ResolvedTimeline {
    pub ppq: u16,
    pub tempo_map: TempoMap,
    pub meter_map: MeterMap,
    pub key_map: KeyMap,
    pub beat_grid: BeatGrid,
    pub measure_start_ticks: Vec<u64>,
    pub measure_start_beats: Vec<TimelinePosition>,
    pub total_quarter_beats: f64,
    pub total_ticks: u64,
}
```

---

## 9. Algorithms

### 9.1 Timeline Resolution

```text
function resolve_timeline(comp, ppq) → ResolvedTimeline:
    measure_start_ticks = [0]
    measure_start_beats = [TimelinePosition { m: 1, beat: 0 }]
    total_beats = 0.0
    for each measure m in traversal order:
        meter = meter_at(m, comp.global.meter_map)
        measure_quarter_beats = meter.beats * 4 / meter.beat_type
        total_beats += measure_quarter_beats
        if not last measure:
            tick = beat_position_to_tick(measure_start_beats[-1], measure_quarter_beats, ...)
            measure_start_ticks.push(tick)
            measure_start_beats.push(next position)
    beat_grid = build_beat_grid(comp, measure_start_beats)
    return ResolvedTimeline { ... }
```

### 9.2 BPM at Position

```text
function bpm_at(pos: TimelinePosition, tempo_map) → f64:
    segment = find_segment(tempo_map, pos)
    if segment.ramp is None:
        return segment.bpm
    else:
        t = interpolate(segment.ramp, pos)
        return lerp(segment.bpm, segment.ramp.end_bpm, t)
```

### 9.3 Beat Position to Tick (with Tempo Ramps)

Integration over tempo segments:

```text
function beat_position_to_tick(start: TimelinePosition, delta_beats: f64, timeline) → u64:
    tick = measure_start_ticks[start.global_measure - 1]
    tick += local_beat_to_ticks(start.beat, start.global_measure, timeline)
    remaining = delta_beats
    while remaining > 0:
        bpm = bpm_at(current_position, timeline.tempo_map)
        beat_duration_ticks = timeline.ppq * 60.0 / bpm  // ticks per quarter at this bpm
        consume = min(remaining, beats_until_next_tempo_change)
        tick += consume * beat_duration_ticks
        remaining -= consume
        advance position
    return tick
```

### 9.4 Beat Grid Construction

```text
function build_beat_grid(comp, subdivisions_per_beat) → BeatGrid:
    for each measure m:
        meter = meter_at(m)
        for beat in 0..meter.beats:
            strength = if beat == 0 { Downbeat }
                       else if meter.beats == 4 && beat == 2 { Strong }
                       else { Weak }
            subdivisions = split_beat(subdivisions_per_beat)
            cells.push(BeatCell { beat_index: beat, strength, subdivisions })
```

Strength heuristics parameterized by `rhythm.accent_strength`.

### 9.5 Snap to Grid

```text
function snap_to_grid(offset: BeatOffset, grid: BeatGrid, measure: u32) → BeatOffset:
    candidates = grid.measures[measure].all_subdivision_offsets
    return nearest(offset, candidates)
```

Used by Melody/Rhythm engines for quantization.

### 9.6 Swing Application (Projection-Time)

```text
function apply_swing(offset: BeatOffset, swing_ratio: f64) → BeatOffset:
    // Delay offbeat eighths: ratio 0.5 = straight, 0.67 = triplet swing
    if is_offbeat_eighth(offset):
        return offset + swing_offset(swing_ratio)
    return offset
```

Applied in IR projector when `rhythm.swing > 0` — not stored in AST (preserves straight notation).

---

## 10. Interfaces

```rust
pub struct TimelineResolver {
    pub ppq: u16,
    pub subdivisions_per_beat: u8,
}

impl TimelineResolver {
    pub fn resolve(&self, comp: &Composition) -> ResolvedTimeline;
    pub fn bpm_at(&self, timeline: &ResolvedTimeline, pos: TimelinePosition) -> f64;
    pub fn meter_at(&self, timeline: &ResolvedTimeline, global_measure: u32) -> TimeSignature;
    pub fn key_at(&self, timeline: &ResolvedTimeline, pos: TimelinePosition) -> KeySignature;
    pub fn beat_to_tick(&self, timeline: &ResolvedTimeline, pos: TimelinePosition) -> u64;
    pub fn tick_to_beat(&self, timeline: &ResolvedTimeline, tick: u64) -> TimelinePosition;
    pub fn snap_to_grid(&self, timeline: &ResolvedTimeline, pos: TimelinePosition) -> TimelinePosition;
}
```

### UI Timeline View

Frontend receives `ResolvedTimeline` + `MusicIr` for:

- Measure bar lines
- Tempo ramp visualization
- Playhead position (tick ↔ beat)

---

## 11. Parameter Mappings

| Parameter | Timeline Effect |
|-----------|-----------------|
| `emotion.arousal` | ↑ default BPM |
| `emotion.tension_curve` | tempo_map segment BPM deltas at section boundaries |
| `form.section_lengths` | measure count → grid length |
| `rhythm.subdivision` | `BeatGrid.subdivisions_per_beat` |
| `rhythm.swing` | IR projection swing (not AST) |
| `rhythm.syncopation` | weak-beat strength adjustment in grid |
| `mode.modulation_policy` | entries in `key_map` |
| Structure stage | initial tempo_map, meter_map, key_map population |

---

## 12. Explainability Model

Tempo/meter/key changes from Structure Planning carry provenance at map entry level:

```rust
pub struct MapEntryProvenance {
    pub stage: PipelineStageId,
    pub rule_ids: Vec<RuleId>,
    pub reason: String,  // e.g., "Bridge modulation to dominant"
}
```

Individual notes do not duplicate tempo provenance — query timeline at note position for context.

---

## 13. Future Expansion

| Feature | Timeline Extension |
|---------|-------------------|
| Metric modulation | `MeterChange` with beat remapping |
| Polyrhythm layers | multiple BeatGrids per voice group |
| Tempo tap import | user-defined tempo_map from tap |
| Display vs performance tempo | dual maps |

---

## 14. Open Questions

| ID | Question |
|----|----------|
| OQ-TL-1 | Store swing in AST for notation accuracy? |
| OQ-TL-2 | Pickup measures (anacrusis) — negative beat offsets? |
| OQ-TL-3 | Large BPM ramps: step size for IR tempo events? |

---

## 15. References

- [ast.md](ast.md)
- [ir.md](ir.md)
- [score.md](score.md)
- Lerdahl & Jackendoff (1983)
- MusicXML `<sound>`, `<metronome>`
- MIDI Set Tempo meta event specification

---

*End of Timeline Model Specification*
