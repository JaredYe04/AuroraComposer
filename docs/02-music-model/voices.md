# Voice Model Specification

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

A **Voice** (also **Part** in export terminology) is a single melodic or harmonic line within the texture. In Aurora:

- **VoiceDef** — global definition in `VoiceRegistry` (role, register, channel, instrument)
- **MeasureVoice** — per-measure event container referencing `VoiceId`

Voices are **not necessarily vocal** — they include melody, bass, inner parts, and drums.

This document specifies voice allocation, register constraints, voice groups, and MIDI channel mapping.

---

## 2. Existing Solutions

| System | Model | Notes |
|--------|-------|-------|
| MusicXML | `<part>` + `<staff>` + voice number | Up to 4 voices per staff |
| Bach chorales | SATB four parts | Fixed roles |
| MIDI | 16 channels, ch 10 drums | Channel = timbre routing |
| DAW | Track = channel + instrument | Clip-based |
| Music21 | `Part` in `Score` | Stream-based |

Aurora separates **generative voice** (AST) from **MIDI channel** (IR/export).

---

## 3. Academic / Theoretical Foundation

### 3.1 SATB Texture

Classical four-part writing assigns:

- **Soprano** — melody (highest)
- **Alto/Tenor** — inner voices
- **Bass** — harmonic foundation

Register constraints prevent crossing in strict counterpoint (configurable strictness).

### 3.2 Orchestration Registers

Adler/Orchestration texts define typical instrument ranges — mapped to `PitchRange` on `VoiceDef`.

### 3.3 Voice Crossing

Counterpoint rules penalize voice crossing (melody below bass) — rule engine queries voice roles and registers.

---

## 4. Engineering Analysis

### 4.1 Voice Count Scalability

Default layouts:

| Layout | Voices | Use Case |
|--------|--------|----------|
| Monophonic | 1 | Folk, single-line |
| Lead + accompaniment | 2–3 | Pop |
| SATB | 4 | Chorale |
| SATB + drums | 5 | Pop/rock |
| Full band | 8–16 | Jazz/big band |
| Orchestral | 16–32 | Future |

Hard limit proposal: **32 voices** per Composition.

### 4.2 Stable VoiceId

`VoiceId` assigned at Structure Planning — never reused within composition lifetime. Enables provenance and UI track headers.

---

## 5. Comparison of Approaches

| Approach | Generation | Export | Verdict |
|----------|-----------|--------|---------|
| Channel-first (MIDI) | Awkward for theory | Native | IR only |
| Staff-first (engraving) | Layout coupling | MusicXML native | Export mapping |
| **VoiceDef registry** | Clean roles | Map to channel/staff | **Selected** |
| Dynamic voice creation mid-piece | Flexible | ID churn | Rejected |

---

## 6. Recommended Solution

1. **VoiceRegistry** at Composition root defines all voices upfront (Structure stage)
2. Each **Measure** contains one **MeasureVoice** per active voice (may be empty)
3. **VoiceRole** drives rule sets and register defaults
4. **VoiceGroup** for texture parameters (rhythm section, strings)
5. **MIDI channel** assigned at registry creation; projected to IR

---

## 7. Architecture

```text
Composition
└── voice_registry: VoiceRegistry
    └── VoiceDef[]
        ├── id, role, register
        ├── midi_channel, instrument
        └── group: VoiceGroupId?

Measure
└── voices: MeasureVoice[]
    └── { voice_id, events[] }
```

```text
Structure Planning ──► allocate VoiceRegistry
Melody Engine      ──► write VoiceId(melody)
Counterpoint       ──► write VoiceId(inner)
Bass Engine        ──► write VoiceId(bass)
Drum Engine        ──► write VoiceId(drums)
IR Projector       ──► VoiceId → ChannelId
MusicXML Exporter  ──► VoiceId → Part/Staff
```

---

## 8. Data Structures

### 8.1 VoiceRegistry

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceRegistry {
    pub voices: Vec<VoiceDef>,
    pub groups: Vec<VoiceGroup>,
    pub default_layout: VoiceLayoutId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceDef {
    pub id: VoiceId,
    pub name: String,
    pub role: VoiceRole,
    pub register: PitchRange,
    pub midi_channel: u8,             // 1–16 (SMF convention)
    pub group: Option<VoiceGroupId>,
    pub instrument: InstrumentSpec,
    pub export: VoiceExportSpec,
    pub priority: u8,                 // search ordering (melody first)
    pub mutable: bool,              // can engines write? (false for imported-only)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceRole {
    Melody,
    Alto,
    Tenor,
    Bass,
    Inner,
    HarmonyPad,
    Drums,
    Percussion,
    Lead,              // jazz lead
    Accompaniment,
    BassLine,          // walking bass distinct from choral bass
    Guitar,
    Piano,
    Strings,
    Brass,
    Woodwinds,
    Custom(u32),
}
```

### 8.2 PitchRange

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchRange {
    pub min_midi: u8,
    pub max_midi: u8,
    pub preferred_min: u8,
    pub preferred_max: u8,
}
```

**Invariant I-VREG-1:** `min_midi ≤ preferred_min ≤ preferred_max ≤ max_midi`

### 8.3 InstrumentSpec

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstrumentSpec {
    pub gm_program: u8,               // 0–127 General MIDI
    pub name: String,
    pub transposition: i8,            // semitones (Bb clarinet = -2)
    pub clef: Clef,
    pub staff_lines: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clef {
    Treble, Bass, Alto, Tenor, Percussion, Tab,
}
```

### 8.4 VoiceExportSpec

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceExportSpec {
    pub musicxml_part_id: String,
    pub staff_index: u8,
    pub abbrev: Option<String>,
    pub hide_if_empty: bool,
}
```

### 8.5 VoiceGroup

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceGroupId(pub u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceGroup {
    pub id: VoiceGroupId,
    pub name: String,
    pub kind: VoiceGroupKind,
    pub member_voices: Vec<VoiceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceGroupKind {
    RhythmSection,
    HarmonicBed,
    LeadSection,
    StringSection,
    BrassSection,
    Custom,
}
```

### 8.6 MeasureVoice

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasureVoice {
    pub voice_id: VoiceId,
    pub events: Vec<Event>,
}
```

---

## 9. Algorithms

### 9.1 Voice Allocation (Structure Planning)

```text
function allocate_voices(params) → VoiceRegistry:
    layout = select_layout(params.voice.voice_count, params.style.orchestration)
    voices = []
    for spec in layout.voice_specs:
        voices.push(VoiceDef {
            id: VoiceId(next_id),
            role: spec.role,
            register: compute_register(spec.role, params.register.*),
            midi_channel: assign_channel(spec.role, voices),
            instrument: spec.instrument,
            ...
        })
    groups = build_default_groups(voices)
    return VoiceRegistry { voices, groups, ... }
```

### 9.2 Default Layouts

#### Pop/Rock (5 voices)

| VoiceId | Role | Channel | Register (MIDI) |
|---------|------|---------|-----------------|
| 0 | Melody | 1 | 60–84 |
| 1 | HarmonyPad | 2 | 48–72 |
| 2 | Inner | 3 | 48–67 |
| 3 | BassLine | 4 | 36–60 |
| 4 | Drums | 10 | unpitched |

#### SATB Chorale (4 voices)

| VoiceId | Role | Channel | Register |
|---------|------|---------|----------|
| 0 | Melody (Soprano) | 1 | 60–84 |
| 1 | Alto | 2 | 55–74 |
| 2 | Tenor | 3 | 48–69 |
| 3 | Bass | 4 | 40–64 |

#### Monophonic (1 voice)

| VoiceId | Role | Channel | Register |
|---------|------|---------|----------|
| 0 | Melody | 1 | params.register.melody_* |

### 9.3 Channel Assignment Rules

```text
function assign_channel(role, existing_voices) → u8:
    if role == Drums || role == Percussion:
        return 10  // GM percussion channel
    used = set(v.midi_channel for v in existing_voices)
    for ch in 1..=16:
        if ch != 10 and ch not in used:
            return ch
    error VoiceChannelExhausted
```

**Invariant I-VCH-1:** At most one voice per MIDI channel (except multi-staff same instrument — future).

### 9.4 Register Computation

```text
function compute_register(role, params) → PitchRange:
    match role:
        Melody → params.register.melody_min/max
        Bass, BassLine → params.register.bass_min/max
        Drums → unpitched (ignored)
        _ → interpolate(params.register.melody, params.register.bass, role_weight)
```

### 9.5 Voice Crossing Detection

```text
function detect_crossings(comp, measure) → Vec<Violation>:
    pitches_by_voice = { v: highest_pitched_at_beat(v) for v in voices }
    sorted = sort by pitch descending
    if role_order(sorted) != role_order(voice_registry priority):
        if counterpoint.strictness > threshold:
            violations.push(VoiceCrossing { ... })
```

### 9.6 Range Validation

```text
function check_range(event: NoteEvent, voice: VoiceDef) → Option<Violation>:
    if event.pitch.midi < voice.register.min_midi: return Some(BelowRange)
    if event.pitch.midi > voice.register.max_midi: return Some(AboveRange)
    return None
```

Hard constraint if strict; soft penalty otherwise.

### 9.7 GM Drum Map (Standard)

| GM Note | Name | Typical Role |
|---------|------|--------------|
| 36 | Bass Drum 1 | Kick |
| 38 | Acoustic Snare | Snare |
| 42 | Closed Hi-Hat | Hi-hat |
| 46 | Open Hi-Hat | Open hat |
| 49 | Crash Cymbal 1 | Crash |
| 51 | Ride Cymbal 1 | Ride |

Full map in export MIDI spec.

---

## 10. Interfaces

```rust
pub struct VoiceAllocator;

impl VoiceAllocator {
    pub fn allocate(params: &VoiceParams, style: &StylePreset) -> VoiceRegistry;
    pub fn get_voice<'a>(registry: &'a VoiceRegistry, id: VoiceId) -> Option<&'a VoiceDef>;
    pub fn voices_by_role(registry: &VoiceRegistry, role: VoiceRole) -> Vec<&VoiceDef>;
    pub fn melody_voice(registry: &VoiceRegistry) -> Option<VoiceId>;
    pub fn bass_voice(registry: &VoiceRegistry) -> Option<VoiceId>;
    pub fn drum_voice(registry: &VoiceRegistry) -> Option<VoiceId>;
}

pub struct VoiceContext {
    pub voice_id: VoiceId,
    pub def: VoiceDef,
    pub sibling_voices: Vec<VoiceId>,
}
```

Rule engine receives `VoiceContext` in `EventContext`.

---

## 11. Parameter Mappings

| Parameter | Voice Effect |
|-----------|-------------|
| `voice.voice_count` | `VoiceRegistry.voices.len()` |
| `voice.density` | how many inner voices active |
| `texture.homophony_polyphony_balance` | HarmonyPad vs independent inner |
| `register.melody_register` | Melody `PitchRange` |
| `register.bass_register` | Bass `PitchRange` |
| `style.orchestration_preset` | layout template selection |
| `counterpoint.strictness` | crossing/range rule hardness |
| `drums.*` | Drums voice activation |

---

## 12. Explainability Model

Voice assignment provenance stored at registry level:

```rust
pub struct VoiceAllocationProvenance {
    pub stage: PipelineStageId,
    pub layout_id: VoiceLayoutId,
    pub parameters_hash: String,
    pub explanation: String,
}
```

Attached to `Composition.metadata` or `VoiceRegistry` extension field.

Per-event provenance includes `voice_id` implicitly via `EventContext` — UI track header shows role + register when inspecting.

---

## 13. Future Expansion

| Feature | Voice Model Extension |
|---------|----------------------|
| Transposing instruments | `InstrumentSpec.transposition` active in IR |
| Divisi (string section split) | temporary VoiceId fork |
| Consort layouts | preset VoiceGroup templates |
| User-defined voice order | UI reorder without ID change |
| MPE | one channel, multiple note expression |

---

## 14. Open Questions

| ID | Question |
|----|----------|
| OQ-VOC-1 | Allow two voices on same MIDI channel for piano L/R hands? |
| OQ-VOC-2 | Auto re-allocate channels when voice_count changes mid-edit? |
| OQ-VOC-3 | Percussion on non-channel-10 for non-GM export? |

---

## 15. References

- [ast.md](ast.md)
- [ir.md](ir.md)
- [score.md](score.md)
- [events.md](events.md)
- MusicXML `<score-part>` and `<part-group>`
- GM MIDI specification
- Fux, *Gradus ad Parnassum* (voice ranges)

---

*End of Voice Model Specification*
