# MIDI Export Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Export Format Research Agent (MIDI)  
**Dependencies:** `docs/02-music-model/ir.md` *(pending)*, [musicxml.md](musicxml.md), [ACAS v0.1](../00-overview/acas-v0.1.md)

---

## 1. Background

Standard MIDI Files (SMF) provide universal playback interchange. Aurora Composer exports SMF Type 1 (multi-track) files from **Music IR** — a flattened, time-ordered event stream — not directly from the hierarchical AST. MIDI is a **secondary format**: optimized for DAW import, hardware playback, and browser preview synthesis; it does not carry notation layout, chord symbols, form hierarchy, or provenance.

The MIDI export module is a deterministic projection:

```text
Music AST  →  Music IR  →  MidiExporter  →  .mid file
```

MIDI export runs in parallel with MusicXML export from the same IR snapshot. No MIDI → AST import is planned for v0.1 (lossy); users import notation via MusicXML.

---

## 2. Existing Solutions

### 2.1 midly (Rust)

Pure-Rust SMF read/write. Supports Type 0/1, meta events, variable-length quantities. **Recommended** for Aurora backend.

### 2.2 Music21 `write('midi')`

Reference behavior for multi-voice track layout and tempo maps. Aurora mirrors track-per-voice convention.

### 2.3 DAW Conventions

- Ableton Live: prefers Type 1, one track per channel
- Logic Pro: imports Type 1; merges Type 0
- FL Studio: channel-based mixer routing

### 2.4 Browser Playback

`@tonejs/midi` parses SMF for Web Audio scheduling (see [playback.md](../../research/playback.md)). MIDI export format must be compatible with Tone.js parser (standard SMF, no vendor extensions).

---

## 3. Academic / Theoretical Foundation

SMF encodes **MIDI event messages** on a **tick timeline** (delta times). Musical semantics are limited to:

- Note On/Off (pitch, velocity, channel)
- Control Change (sustain, modulation)
- Program Change (instrument)
- Meta events (tempo, time signature, track name, lyrics)

SMF does not encode:

- Spelling (enharmonic equivalence only via pitch number)
- Notation (beaming, stems, ties as visual objects)
- Harmonic analysis
- Structural form

MIDI 1.0 General MIDI (GM) Level 1 specifies 128 programs and percussion map on channel 10 — Aurora defaults to GM Level 1 for interoperability.

---

## 4. Engineering Analysis

| Criterion | Assessment |
|-----------|------------|
| **Correctness** | Pitch and timing must match IR within ±0 ticks; velocity mapping documented |
| **Controllability** | `export.midi.tpqn`, `export.midi.format` parameters |
| **Explainability** | None in file — provenance lost (documented limitation) |
| **Performance** | <100 ms for 10-minute score |
| **Extensibility** | SysEx hook for future vendor extensions (disabled default) |
| **Complexity** | Low — ~800 LOC mapper + midly integration |

---

## 5. Comparison of Approaches

### 5.1 SMF Type 0 vs Type 1

| Type | Structure | Aurora Use |
|------|-----------|------------|
| Type 0 | Single track, all channels | Optional compact mode |
| Type 1 | One track per logical voice + conductor | **Default** |
| Type 2 | Multiple sequences | Not supported |

**Recommendation:** Type 1 default; Type 0 optional for hardware with track limits.

### 5.2 Track vs Channel Mapping

| Strategy | Description | Verdict |
|----------|-------------|---------|
| 1 track : 1 channel : 1 voice | Clean DAW import | **Default** |
| 1 track : multiple channels | Rare; confusing | Rejected |
| All drums merged to track 10 | Standard GM | Required |

### 5.3 Tempo Map Representation

| Approach | Mechanism |
|----------|-----------|
| Single tempo | One `Set Tempo` at tick 0 |
| Multi-tempo | Multiple `Set Tempo` meta events at IR marker positions |
| Ramp (accelerando) | Not native — approximate with stepped tempo or NRPN (non-standard) |

**Recommendation:** Stepwise tempo map from IR; document accelerando as limitation.

---

## 6. Recommended Solution

### 6.1 File Format

| Property | Value |
|----------|-------|
| Format | SMF Type 1 |
| TPQN | 480 (default), configurable 96–960 |
| Time division | Metrical (ticks per quarter) |
| Running status | Enabled on write |
| End of track | FF 2F 00 on every track |

### 6.2 Limitations vs AST (Explicit Contract)

The following AST/IR concepts are **lost or degraded** in MIDI export:

| AST / IR Concept | MIDI Fate | Severity |
|------------------|-----------|----------|
| **Provenance** | Lost entirely | High (by design) |
| **Section / Phrase / Form** | Lost; no meta standard | High |
| **Theme / Motif references** | Lost | High |
| **Chord symbols** | Lost (notes remain) | Medium |
| **Roman numeral analysis** | Lost | Medium |
| **Enharmonic spelling** | Collapsed to MIDI pitch 0–127 | Medium |
| **Ties across bars** | Note duration merge (OK) | Low |
| **Slurs / phrasing** | Lost | Medium |
| **Articulations** | Partial via CC or velocity | Medium |
| **Ornaments (trill, mordent)** | Lost unless realized to notes | Medium |
| **Grace notes** | Lost unless realized | Medium |
| **Dynamics (pp–ff)** | Approximated via velocity curve | Medium |
| **Hairpins (crescendo)** | Lost | Medium |
| **Tuplets** | Quantized to tick grid | Low–Medium |
| **Microtones** | Lost (12-TET only) | High if used |
| **Staff layout** | Lost | N/A |
| **Lyrics** | Optional `FF 05` lyric meta (Phase 2) | Low |
| **Voice leading semantics** | Lost (pitch remains) | Low |
| **Unpitched display step** | GM note number only | Low |
| **Accelerando / ritardando** | Stepped approximation or lost | Medium |

**User-facing summary:** MIDI is for **listening and DAW editing**, not for restoring Aurora projects. Use MusicXML for interchange.

---

## 7. Architecture

```text
┌──────────────────────────────────────────────────────────┐
│                    MIDI Export Pipeline                   │
├──────────────────────────────────────────────────────────┤
│  Music IR Snapshot                                        │
│       │                                                   │
│       ▼                                                   │
│  IrToMidiMapper                                           │
│    ├── VoiceToTrackAssigner                               │
│    ├── DrumChannelRouter (→ ch 10)                        │
│    ├── TempoMapBuilder                                    │
│    ├── NoteEventQuantizer (TPQN alignment)                │
│    └── VelocityMapper (dynamic → velocity)                │
│       │                                                   │
│       ▼                                                   │
│  Midly Smf struct                                         │
│       │                                                   │
│       ▼                                                   │
│  midly::write() → Vec<u8> → .mid file                     │
└──────────────────────────────────────────────────────────┘
```

### 7.1 Conductor Track (Track 0)

Track 0 contains **no note events**. It carries:

- `Sequence/Track Name`: "Aurora Composer"
- `Set Tempo` events
- `Time Signature` events
- `Key Signature` events (optional, advisory)

All musical content begins on Track 1+.

---

## 8. Data Structures

### 8.1 MidiExportConfig

```rust
pub struct MidiExportConfig {
    pub format: SmfFormat,           // Type0 | Type1 (default)
    pub tpqn: u16,                   // default 480
    pub velocity_curve: VelocityCurve,
    pub include_key_signature: bool, // default true
    pub include_track_names: bool,   // default true
    pub drum_channel: u8,            // default 10 (1-indexed MIDI convention)
    pub realize_grace_notes: bool,   // default false
    pub realize_or naments: bool,    // default false
}
```

### 8.2 Voice → Track Mapping

```rust
struct TrackAssignment {
    ir_voice_id: VoiceId,
    track_index: u8,        // 1-based (0 = conductor)
    midi_channel: u8,       // 1–16
    is_percussion: bool,
    program: u8,            // 0–127 GM program
}
```

Default assignment table:

| IR Voice Role | Track | Channel | Program |
|---------------|-------|---------|---------|
| Melody | 1 | 1 | 0 (Acoustic Grand Piano) |
| Countervoice 1 | 2 | 2 | 48 (String Ensemble) |
| Countervoice 2 | 3 | 3 | 48 |
| Bass | 4 | 4 | 32 (Acoustic Bass) |
| Drums | 5 | **10** | 0 (drum kit) |

Channel 10 uses **note numbers** per GM percussion map, not program change.

### 8.3 IR Event → MIDI Message Mapping

| IR Event | MIDI Output |
|----------|-------------|
| `IrNote { pitch, velocity, duration_ticks, channel }` | Note On + Note Off (delta) |
| `IrRest { duration_ticks }` | Silence (no message) |
| `IrControlChange { controller, value }` | CC message |
| `IrProgramChange { program }` | Program Change |
| `IrTempoChange { bpm }` | Meta Set Tempo (track 0) |
| `IrTimeSignatureChange { num, den }` | Meta Time Signature (track 0) |
| `IrKeyChange { fifths, mode }` | Meta Key Signature (track 0) |

---

## 9. Algorithms

### 9.1 IR to SMF Conversion

```
function export_midi(ir: MusicIR, config: MidiExportConfig) -> Vec<u8>:
    smf = new Smf(format=Type1, division=config.tpqn)
    conductor = smf.track(0)

    // Tempo map on conductor
    for tempo_event in ir.tempo_events_ordered():
        conductor.add(SetTempo(microseconds_per_quarter=60000000 / tempo_event.bpm))
        conductor.delta = tempo_event.tick - conductor.current_tick

    // Assign tracks
    assignments = assign_tracks(ir.voices, config)

    for assignment in assignments:
        track = smf.add_track()
        track.add(MetaTrackName(ir.voice_name(assignment.ir_voice_id)))

        if not assignment.is_percussion:
            track.add(ProgramChange(assignment.channel, assignment.program))

        for event in ir.voice_events(assignment.ir_voice_id):
            if event is IrNote:
                tick_delta = event.start_tick - track.current_tick
                track.add(NoteOn(assignment.channel, event.pitch, event.velocity), delta=tick_delta)
                track.add(NoteOff(assignment.channel, event.pitch, 0), delta=event.duration_ticks)
            // rests: advance tick counter only

        track.add(EndOfTrack())

    return midly_write(smf)
```

### 9.2 Drum Channel 10 Conventions

Per GM Level 1:

- Channel 10 (index 9 in 0-based APIs) is **reserved for percussion**
- `Program Change` on channel 10 is ignored by GM devices
- Each `IrNote.pitch` for drum voice maps through `GM_DRUM_MAP`:

```rust
const GM_DRUM_MAP: &[(DrumId, u8)] = &[
    (DrumId::Kick, 36),
    (DrumId::Snare, 38),
    (DrumId::SideStick, 37),
    (DrumId::ClosedHiHat, 42),
    (DrumId::OpenHiHat, 46),
    (DrumId::PedalHiHat, 44),
    (DrumId::Crash, 49),
    (DrumId::Ride, 51),
    (DrumId::Tom1, 48),
    (DrumId::Tom2, 45),
];
```

**Export rule:** Drum `IrNote` always routed to channel 10 regardless of `Voice.midi_channel` field override (with warning log if conflict).

### 9.3 Velocity Mapping

```text
velocity = clamp(1, 127, round(dynamic_to_velocity(event.dynamic) * event.velocity_scale))
```

Default dynamic curve:

| Dynamic | Velocity |
|---------|----------|
| ppp | 16 |
| pp | 32 |
| p | 48 |
| mp | 64 |
| mf | 80 |
| f | 96 |
| ff | 112 |
| fff | 127 |

Accent: `velocity += 12` (capped at 127).

### 9.4 Tempo Meta Events

Set Tempo meta event (FF 51 03):

```text
microseconds_per_quarter = 60_000_000 / bpm
```

For compound tempo map (from AST `GlobalAttributes.tempo_map`):

```text
IR tempo markers at tick positions → conductor track Set Tempo events
```

Time signature changes co-located at same ticks when measure-aligned.

### 9.5 Quantization

IR events already on tick grid from projection. If floating-point drift detected:

```text
aligned_tick = round(event.tick / gcd) * gcd
```

where `gcd` derived from smallest note duration in score. Log warning if adjustment > 2 ticks.

---

## 10. Interfaces

### 10.1 Rust API

```rust
pub fn export_midi(ir: &MusicIR, config: &MidiExportConfig) -> Result<Vec<u8>, ExportError>;

pub fn export_midi_to_file(
    ir: &MusicIR,
    path: &Path,
    config: &MidiExportConfig,
) -> Result<(), ExportError>;

pub fn validate_midi(bytes: &[u8]) -> Result<MidiInfo, ExportError>;
```

### 10.2 Tauri IPC

```typescript
export_composition_midi(composition_id: string, config: MidiExportConfig): Promise<ArrayBuffer>
```

### 10.3 Playback Integration

Same `Vec<u8>` output fed to frontend via base64 or temp file for `@tonejs/midi` playback — no separate "preview MIDI" format.

---

## 11. Parameter Mappings

| User Parameter | Config / Behavior |
|----------------|-------------------|
| `export.midi.tpqn` | `MidiExportConfig.tpqn` |
| `export.midi.format` | Type0 / Type1 |
| `style.orchestration_preset` | Track program assignments |
| `dynamics.dynamic_range` | Velocity curve scaling |
| `drums.density` | *(no MIDI-specific)* — affects IR content |
| `voice.count` | Number of melodic tracks |
| `GlobalAttributes.tempo_map` | Conductor Set Tempo events |
| `rhythm.time_signature` | Meta Time Signature |

### 11.1 AST / IR → MIDI Track Mapping Table

| AST / IR Entity | MIDI Destination |
|-----------------|------------------|
| `Voice` (non-drum) | Type 1 track + dedicated channel |
| `Voice[drums]` | Track N, channel 10 only |
| `IrNote` | Note On/Off pair |
| `Event::Note.pitch` | Note number 0–127 |
| `Event::Note.velocity` | Note On velocity |
| `Event::Note.duration` | Note Off delta |
| `Marker::TempoChange` | Conductor Set Tempo |
| `Measure.time_signature` | Conductor Time Signature |
| `GlobalAttributes.key` | Conductor Key Signature (advisory) |
| `Voice.name` | Track Name meta |
| `Composition.metadata.title` | Sequence name (optional) |

---

## 12. Explainability Model

MIDI export **does not carry provenance**. When user exports MIDI:

1. UI displays notice: "MIDI export does not include generation history. Use MusicXML for full data."
2. Export audit log records `provenance_included: false`
3. Inspector remains available in-app (AST source) but not in exported file

For explainability-preserving playback, use in-app IR → Web Audio path ([playback.md](../../research/playback.md)) which reads provenance from parallel AST handle.

---

## 13. Future Expansion

| Phase | Feature |
|-------|---------|
| v0.2 | Lyric meta events (`FF 05`) |
| v0.2 | RPN/NRPN for fine pitch bend (microtonal) |
| v0.3 | MIDI 2.0 export (when library support matures) |
| v0.3 | Optional ornament realization pre-export |
| v0.4 | SMF import (lossy, no provenance) for MIDI drag-drop |

---

## 14. Open Questions

1. Should Type 0 be default for simpler email attachment use cases?
2. Include copyright meta (`FF 02`) from composition metadata?
3. Sustain pedal CC#64 from AST pedal markers — Phase 1 or 2?
4. Reconcile TPQN default (480) with MusicXML divisions (480) — unified export config?

---

## 15. References

- MIDI Manufacturers Association: MIDI 1.0 Detailed Specification
- RP-016: Standard MIDI Files 1.0
- General MIDI Level 1 Specification
- midly crate documentation
- [MusicXML Specification](musicxml.md) — primary interchange
- [Playback Research](../../research/playback.md)
- [Export Research Notes](../../research/export-research-notes.md)
- [ACAS v0.1 §9](../00-overview/acas-v0.1.md)

---

*End of MIDI Export Specification v0.1*
