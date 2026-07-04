# Playback Architecture Research

**Version:** 0.1  
**Status:** Draft  
**Agent:** Export Format Research Agent (Playback)  
**Dependencies:** [midi.md](../docs/06-export/midi.md), `docs/02-music-model/ir.md` *(pending)*, [ACAS v0.1](../docs/00-overview/acas-v0.1.md)

---

## 1. Background

Aurora Composer requires **in-app audio preview** so users hear compositions during parameter tweaking, pipeline progress, and post-generation review. Playback is **not** a file export — it is a real-time rendering path from **Music IR** to **Web Audio API** output in the Vue 3 frontend.

```text
Music AST  →  Music IR  →  PlaybackScheduler  →  Web Audio / Tone.js  →  speakers
```

Design goals:

- Latency < 50 ms for transport start
- No system MIDI driver dependency (default path)
- General MIDI (GM) soundfont-based timbre
- Synchronized with timeline UI (playhead)
- Provenance remains in parallel AST (not in audio stream)

Deep research report recommends Tone.js; ACAS specifies Web Audio API / Tone.js for cross-platform preview.

---

## 2. Existing Solutions

### 2.1 Tone.js + @tonejs/midi

- **Tone.js:** Web Audio framework — Transport, Synth, Sampler, scheduling
- **@tonejs/midi:** Parses SMF ArrayBuffer into note events with timing
- **Pattern:** MIDI bytes → Midi.parse() → Tone.Part per track

### 2.2 MIDI.js / soundfont-player

- Loads GM soundfonts as JSON or binary
- Simpler API but less maintained than Tone.js ecosystem
- Used by many abcjs integrations

### 2.3 Web MIDI API

- Sends events to hardware synthesizers
- Requires user device; not default for Aurora desktop app

### 2.4 FluidSynth (Tauri backend)

- High-quality offline render
- Heavy native dependency (~30 MB+)
- Rejected as default; optional Phase 3 plugin for export-to-WAV

### 2.5 abcjs synth

- ABC-specific; limited to ABC export path preview tab

---

## 3. Academic / Theoretical Foundation

Real-time music playback requires:

1. **Score time → wall clock mapping** via tempo map and transport
2. **Event scheduling** with lookahead buffer (typically 25–100 ms) to avoid glitches
3. **Polyphonic mixing** with per-channel gain staging
4. **Percussion isolation** on channel 10 with unpitched samples

Web Audio API uses **sample-accurate scheduling** via `AudioContext.currentTime`. Tone.js Transport abstracts BPM and tick conversion.

---

## 4. Engineering Analysis

| Criterion | IR → Tone.js Direct | IR → MIDI bytes → @tonejs/midi |
|-----------|---------------------|--------------------------------|
| Latency | Lower (skip MIDI encode) | +5–15 ms encode overhead |
| Code reuse | Separate scheduler | Reuses MIDI exporter |
| Debugging | Custom event log | Standard MIDI tools |
| Drift | Must sync Transport | Midi timing proven |

**Recommendation:** **IR → PlaybackScheduler → Tone.js** direct scheduling for preview; optional MIDI byte path for regression testing against SMF export.

| Criterion | Sampler (soundfont) | Oscillator Synth |
|-----------|---------------------|------------------|
| Timbre | Realistic GM | Poor for strings/brass |
| Load time | 2–20 MB font | Instant |
| CPU | Moderate | Low |

**Recommendation:** GM soundfont via Tone.Sampler.

---

## 5. Comparison of Approaches

### 5.1 Scheduling Architectures

| Architecture | Description | Verdict |
|--------------|-------------|---------|
| **Transport + Part** | Tone.Part schedules note callbacks | **Default** |
| **ScriptProcessor** | Deprecated | Rejected |
| **AudioWorklet custom** | Low-level; high effort | Phase 3 |
| **Backend FluidSynth stream** | IPC audio buffer | Optional offline |

### 5.2 Soundfont Strategies

| Strategy | Size | Quality | Offline |
|----------|------|---------|---------|
| MusyngKite (full GM) | ~20 MB | Excellent | Yes |
| FluidR3 GM (subset) | ~2 MB | Good | Yes |
| Salamander Piano only | ~15 MB | Piano only | Yes |
| Synthetic default | 0 MB | Poor | Yes |

**Recommendation:** Ship **FluidR3 subset** (~2 MB) default; optional download MusyngKite in settings.

---

## 6. Recommended Solution

### 6.1 Real-Time Preview Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                  Vue 3 Frontend (Playback Layer)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  CompositionStore (Pinia)                                        │
│    ├── ast: CompositionHandle                                    │
│    ├── ir: MusicIRSnapshot                                       │
│    └── revision: u64                                             │
│           │                                                      │
│           ▼                                                      │
│  PlaybackController                                              │
│    ├── state: stopped | playing | paused                         │
│    ├── playhead_tick: u64                                        │
│    ├── loop_region: Option<Range>                                │
│    └── output: PlaybackOutput                                    │
│           │                                                      │
│           ▼                                                      │
│  IrPlaybackScheduler                                             │
│    ├── build_schedule(ir, tempo_map) → ScheduledEvent[]          │
│    ├── apply_transport(Tone.Transport, bpm)                      │
│    └── register_parts(instruments)                             │
│           │                                                      │
│           ▼                                                      │
│  InstrumentBank (Tone.Sampler per channel/program)               │
│    ├── melodic: Map<(channel, program), Sampler>               │
│    └── drums: DrumSampler (channel 10 map)                       │
│           │                                                      │
│           ▼                                                      │
│  Tone.context → destination (speakers)                           │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Data Flow: Preview vs Full Export

| Path | Source | Use |
|------|--------|-----|
| Preview playback | IR via IPC (live) | Real-time |
| MIDI file export | IR → midly | File share |
| MIDI regression test | IR → midly → @tonejs/midi | CI parity check |

Preview does **not** write temp MIDI files to disk.

### 6.3 Lookahead Scheduling

```typescript
const LOOKAHEAD_MS = 100;
const SCHEDULE_INTERVAL_MS = 25;

function schedulerTick() {
  const transportTime = Tone.Transport.seconds;
  const horizon = transportTime + LOOKAHEAD_MS / 1000;

  while (nextEventIndex < schedule.length &&
         schedule[nextEventIndex].time <= horizon) {
    const event = schedule[nextEventIndex];
    instrumentBank.trigger(event, event.time);
    nextEventIndex++;
  }
}

setInterval(schedulerTick, SCHEDULE_INTERVAL_MS);
```

Tone.Part may replace manual loop when schedule is static for full composition.

---

## 7. Architecture (Detailed)

### 7.1 Component Responsibilities

| Component | Layer | Role |
|-----------|-------|------|
| `PlaybackController` | Vue composable | Transport control API |
| `IrPlaybackScheduler` | TypeScript service | IR → timed events |
| `InstrumentBank` | TypeScript service | Sample loading, note trigger |
| `DrumSampler` | TypeScript | GM percussion map |
| `PlayheadSync` | Vue | Emits tick for timeline UI |
| `AudioUnlock` | Vue | User gesture → `Tone.start()` |

### 7.2 IPC from Tauri

```typescript
// Fetch IR snapshot for playback
get_playback_ir(composition_id: string): Promise<MusicIRJson>

// Optional: partial IR for 2-bar preview mode
get_preview_ir(composition_id: string, bar_range: [number, number]): Promise<MusicIRJson>
```

IR JSON schema mirrors pending `ir.md` — flat event arrays per voice with tick timestamps.

### 7.3 Synchronization with UI

```text
Tone.Transport.scheduleRepeat((time) => {
  const tick = secondsToTick(Tone.Transport.seconds, tempoMap);
  playheadStore.setTick(tick);
}, '32n');
```

Timeline and piano roll subscribe to `playheadStore.tick`.

---

## 8. Data Structures

### 8.1 ScheduledEvent

```typescript
interface ScheduledEvent {
  type: 'noteOn' | 'noteOff' | 'controlChange';
  tick: number;
  channel: number;       // 1–16
  pitch?: number;        // 0–127
  velocity?: number;     // 0–127
  durationTicks?: number;
  program?: number;
}
```

### 8.2 PlaybackConfig

```typescript
interface PlaybackConfig {
  soundfont: 'fluidr3_subset' | 'musyngkite' | 'synthetic';
  masterVolume: number;      // 0.0–1.0, default 0.8
  drumVolume: number;        // relative, default 1.0
  metronomeEnabled: boolean;
  metronomeVolume: number;
  loopEnabled: boolean;
}
```

### 8.3 InstrumentBank State

```typescript
interface InstrumentBank {
  samplers: Map<string, Tone.Sampler>;  // key: `${channel}:${program}`
  drumKit: Tone.Sampler;
  loadPromise: Promise<void>;
  isLoaded: boolean;
}
```

---

## 9. Algorithms

### 9.1 IR to Schedule Conversion

```
function build_schedule(ir: MusicIR, tpqn: number = 480): ScheduledEvent[]:
    events = []

    for voice in ir.voices:
        channel = voice.midi_channel
        program = voice.program

        for note in voice.notes:
            events.add(ScheduledEvent(
                type: noteOn,
                tick: note.start_tick,
                channel, pitch: note.pitch, velocity: note.velocity,
                durationTicks: note.duration_ticks,
                program
            ))
            events.add(ScheduledEvent(
                type: noteOff,
                tick: note.start_tick + note.duration_ticks,
                channel, pitch: note.pitch
            ))

    return events.sort(by tick, then noteOff before noteOn at same tick)
```

### 9.2 Tempo Map → Tone.Transport

```
function apply_tempo_map(transport: Transport, tempo_map: TempoMap):
    if tempo_map.is_single_tempo():
        transport.bpm.value = tempo_map.initial_bpm
    else:
        for point in tempo_map.points:
            transport.schedule((time) => {
                transport.bpm.value = point.bpm
            }, secondsAtTick(point.tick))
```

### 9.3 GM Soundfont Loading

```
async function loadInstrumentBank(config: PlaybackConfig):
    if config.soundfont == 'synthetic':
        return createPolySynthBank()

    const baseUrl = config.soundfont == 'musyngkite'
        ? '/soundfonts/musyngkite/'
        : '/soundfonts/fluidr3/'

    bank = new InstrumentBank()

    // Load default programs used by Aurora orchestration preset
    for program in DEFAULT_PROGRAMS:
        bank.samplers.set(`1:${program}`, await Sampler.fromUrl(
            `${baseUrl}/${program}.mp3`,  // or .sf2 decoded at build
            { onload: () => {} }
        ))

    bank.drumKit = await loadDrumKit(`${baseUrl}/drums/`)
    return bank
```

**Build pipeline:** Convert SF2 to per-instrument MP3/OGG samples at build time (reduces runtime parse cost).

### 9.4 Drum Channel 10 Playback

```typescript
const DRUM_SAMPLE_MAP: Record<number, string> = {
  36: 'kick',
  38: 'snare',
  42: 'hihat-closed',
  46: 'hihat-open',
  49: 'crash',
  51: 'ride',
};

function triggerDrum(pitch: number, time: number, velocity: number) {
  const sample = DRUM_SAMPLE_MAP[pitch] ?? 'unknown';
  drumKit.triggerAttackRelease(sample, '8n', time, velocity / 127);
}
```

Matches [midi.md](../docs/06-export/midi.md) GM map for export/playback parity.

### 9.5 Partial Preview (2-Bar Mode)

Pipeline preview mode generates 2-bar IR subset:

```
function get_preview_ir(composition_id, bar_range):
    full_ir = project_ast_to_ir(composition)
    return filter_ir_by_bar_range(full_ir, bar_range)
```

PlaybackController auto-loops preview region when `loopEnabled`.

---

## 10. Interfaces

### 10.1 PlaybackController API (Vue Composable)

```typescript
function usePlayback() {
  return {
    play: () => Promise<void>,
    pause: () => void,
    stop: () => void,
    seek: (tick: number) => void,
    setLoop: (start: number, end: number) => void,
    state: Ref<'stopped' | 'playing' | 'paused'>,
    playheadTick: Ref<number>,
    loadComposition: (id: string) => Promise<void>,
    config: PlaybackConfig,
  };
}
```

### 10.2 Tauri Commands

```typescript
get_playback_ir(composition_id: string): Promise<MusicIRJson>
get_preview_ir(composition_id: string, start_bar: number, end_bar: number): Promise<MusicIRJson>
```

### 10.3 Audio Context Lifecycle

```typescript
// Required user gesture on first play (browser policy)
async function ensureAudioStarted() {
  if (Tone.context.state !== 'running') {
    await Tone.start();
  }
}
```

Tauri desktop WebView follows same autoplay policy — bind first `play()` to button click.

---

## 11. Parameter Mappings

| User Parameter | Playback Effect |
|----------------|-----------------|
| `GlobalAttributes.tempo_map` | Transport BPM schedule |
| `dynamics.dynamic_range` | Velocity scaling in scheduler |
| `style.orchestration_preset` | Which samplers to preload |
| `drums.density` | IR content (event count) |
| `voice.count` | Polyphony / mixer load |
| `preview.mode` | 2-bar IR subset |
| User setting `playback.soundfont` | InstrumentBank loader |

---

## 12. Explainability Model

Playback uses IR derived from AST — **provenance is not audible**. During playback:

- UI Inspector still shows provenance for note at playhead (AST query by tick + pitch)
- Click note in piano roll during playback → highlight + provenance panel

Optional debug overlay: display `rule-id` on hover when interchange MusicXML was loaded with Tier B metadata.

---

## 13. Future Expansion

| Phase | Feature |
|-------|---------|
| v0.2 | Offline bounce to WAV (FluidSynth plugin or Tone.Offline) |
| v0.2 | Metronome with downbeat accent |
| v0.3 | Follow-mode: scroll score with playhead |
| v0.3 | Web MIDI output option (user hardware) |
| v0.4 | Spatial audio / reverb send per section |
| v0.5 | Real-time parameter tweak → partial regeneration → seamless resume |

---

## 14. Open Questions

1. Preload all GM programs vs lazy load on first note (memory vs latency)?
2. SF2 vs pre-rendered MP3 samples — quality/size tradeoff for Tauri bundle?
3. Share InstrumentBank singleton across ScorePreview and PlaybackController?
4. Export-to-WAV: Tone.Offline vs backend FluidSynth for CI golden audio tests?
5. Licensing: FluidR3 / MusyngKite redistribution in commercial Aurora release?

---

## 15. References

- Tone.js documentation: https://tonejs.github.io/
- @tonejs/midi: https://github.com/Tonejs/Midi
- Web Audio API W3C specification
- General MIDI Level 1 sound set
- soundfont-player (MIT) — reference loader patterns
- [MIDI Export Specification](../docs/06-export/midi.md)
- [Export Research Notes](export-research-notes.md)
- [ACAS v0.1 §10](../docs/00-overview/acas-v0.1.md)
- [Deep Research Report §4](../deep-research-report.md)

---

*End of Playback Architecture Research v0.1*
