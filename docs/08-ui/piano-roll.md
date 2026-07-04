# Piano Roll UI Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Plugin & UI Research Agent  
**Dependencies:** [vue.md](vue.md), [events.md](../02-music-model/events.md), [ast.md](../02-music-model/ast.md), [inspector.md](inspector.md), [tauri.md](tauri.md)

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

The Piano Roll is Aurora Composer's **note-level editing grid** — pitch (vertical) vs time (horizontal). It displays `Note`, `Chord`, and `Rest` events from the AST, supports selection and editing, and surfaces **provenance on hover** with full detail delegated to the Inspector on click.

This component is central to the explainability UX: users must see at a glance *why* a note exists before drilling into the full provenance chain.

### 1.1 Scope

- `PianoRoll.vue` Canvas-based grid
- Multi-voice lane display
- Note selection, move, resize, delete
- Provenance tooltip on hover
- Snap-to-grid from rhythm parameters
- Sync with Timeline horizontal scroll/zoom

---

## 2. Existing Solutions

| Tool | Piano Roll | Provenance |
|------|-----------|------------|
| FL Studio | Industry standard grid | None |
| Ableton | MIDI clip editor | None |
| Signal (OSS) | Web Canvas piano roll | None |
| Magenta Studio | TensorFlow.js grid | None |
| MuseScore | Notation, not grid | None |

No prior art combines piano roll editing with rule-level provenance display. Aurora adds provenance-aware note styling and tooltips.

---

## 3. Academic / Theoretical Foundation

### 3.1 Pitch-Class vs MIDI Number

Grid rows map MIDI pitch (vertical axis). Enharmonic spelling from AST `Pitch.spelling` shown in tooltip, not on grid (MIDI row per semitone).

### 3.2 Metric Grid

Beat subdivisions derived from `TimeSignature` and `rhythm.subdivision` parameter. Rational `BeatOffset` from AST maps to pixel columns without floating-point drift.

---

## 4. Engineering Analysis

| Criterion | Response |
|-----------|----------|
| Performance | Canvas 2D; draw only visible region |
| Interaction | 60fps drag; debounce patch apply |
| Explainability | Color by `ProvenanceSource`; tooltip summary |
| Correctness | Edits via `apply_patch` with ManualEdit provenance |

---

## 5. Comparison of Approaches

| Rendering | Max Notes | Interaction | Verdict |
|-----------|-----------|-------------|---------|
| DOM per note | ~200 | Easy | Rejected |
| SVG | ~2000 | Medium | Fallback |
| **Canvas 2D** | 10000+ | Manual hit-test | **Selected** |
| WebGL | 50000+ | Complex | Phase 3 |

---

## 6. Recommended Solution

**HTML5 Canvas** piano roll with:

1. Separate layers: grid, notes, selection, playhead
2. Hit-test spatial index (grid bucketing)
3. Provenance tooltip via Vue overlay (not Canvas text)
4. Voice lanes (swimlanes) collapsible

---

## 7. Architecture

### 7.1 Component Structure

```text
PianoRoll.vue
├── PianoRollCanvas.vue       (main draw loop)
├── PianoRollKeyboard.vue     (pitch labels, C keys highlighted)
├── VoiceLaneHeader.vue       (voice name, mute, solo)
├── ProvenanceTooltip.vue     (hover overlay)
└── PianoRollToolbar.vue        (snap, quantize, draw mode)
```

### 7.2 Visual Layout

```text
┌────┬────────────────────────────────────────────────────────┐
│ C7 │                                                        │
│ B6 │     ┌──┐                                               │
│ A6 │     │  │  ← note block (color = provenance source)    │
│ G6 │ ┌───┘  └──┐                                            │
│ ...│ │         │                                            │
│ C4 │─┴─────────┴──────────────────────────────────────────  │
│    │ m1    m2    m3    m4    m5    m6    m7    m8          │
└────┴────────────────────────────────────────────────────────┘
     ▲                                                    ▲
  Keyboard                                          Timeline sync
```

### 7.3 Coordinate Systems

| Axis | Unit | Origin |
|------|------|--------|
| X (time) | Pixels | Measure 1, beat 0 = left + scrollX |
| Y (pitch) | Pixels | MIDI 127 = top; row height configurable |

---

## 8. Data Structures

### 8.1 PianoRollNote (View Projection)

```typescript
interface PianoRollNote {
  nodeId: string;
  voiceId: number;
  pitchMidi: number;
  startMeasure: number;
  startBeat: BeatOffset;      // rational { numer, denom }
  durationBeats: BeatOffset;
  velocity: number;
  pitchRole: PitchRole | null;
  provenanceSource: ProvenanceSource;
  provenanceSummary: string;  // precomputed display_summary from backend
  selected: boolean;
  hovered: boolean;
}
```

Loaded via `get_measure_events` for visible measure range; cached in component state.

### 8.2 View State

```typescript
interface PianoRollViewState {
  zoomX: number;              // px per measure (synced with Timeline)
  zoomY: number;              // px per semitone, default 14
  scrollX: number;
  scrollY: number;
  visibleMeasureRange: [number, number];
  activeVoiceIds: number[];
  snapDivision: number;       // 1, 2, 4, 8, 16 from rhythm.subdivision
  tool: 'select' | 'draw' | 'erase';
}
```

### 8.3 Provenance Color Map

| ProvenanceSource | Fill Color | Border |
|------------------|------------|--------|
| Generated | `#4299e1` (blue) | `#2b6cb0` |
| ManualEdit | `#48bb78` (green) | `#2f855a` |
| Repaired | `#ed8936` (orange) | `#c05621` |
| Plugin | `#9f7aea` (purple) | `#6b46c1` |
| Imported | `#a0aec0` (gray) | `#718096` |
| Transformed | `#38b2ac` (teal) | `#2c7a7b` |

`pitch_role` adds icon overlay: passing tone `→`, neighbor `↕`, chord tone `●`.

---

## 9. Algorithms

### 9.1 Visible Note Query

```text
function loadVisibleNotes(projectId, measureStart, measureEnd, voiceIds):
    notes = []
    for m in measureStart..measureEnd:
        events = get_measure_events(projectId, m, voiceIds)
        for event in events:
            if event is Note or Chord:
                notes.push(projectToPianoRollNote(event, m))
    return notes
```

### 9.2 Draw Loop

```text
function drawFrame(ctx, state, notes):
    clearCanvas()
    drawGrid(ctx, state.visibleMeasureRange, state.snapDivision)
    drawNotes(ctx, notes filtered by viewport)
    drawSelectionRect(ctx, state.selectionBox)
    drawPlayhead(ctx, playbackStore.currentTick)
```

### 9.3 Hit Test (Grid Bucketing)

```text
function buildSpatialIndex(notes, cellSize=20px):
    buckets = HashMap<cellKey, noteId[]>
    for note in notes:
        for cell in note.boundingCells(cellSize):
            buckets[cell].push(note.id)

function hitTest(x, y, buckets):
    cell = cellKey(x, y)
    for id in buckets[cell]:
        if noteBounds(id).contains(x, y):
            return id
    return null
```

### 9.4 Note Edit → Patch

```text
function onNoteDragEnd(note, newPitch, newStart, newDuration):
    patch = PatchBuilder("User moved note")
        .updateField(note.nodeId, pitch, newPitch)
        .updateField(note.nodeId, offset, newStart)
        .updateField(note.nodeId, duration, newDuration)
        .setProvenance(ManualEdit, parent=note.nodeId,
            explanation="User moved note")
    apply_patch(projectId, patch)
```

### 9.5 Hover → Tooltip

```text
function onMouseMove(x, y):
    noteId = hitTest(x, y)
    if noteId != hoveredEventId:
        hoveredEventId = noteId
        if noteId:
            note = notes.get(noteId)
            showTooltip(note.provenanceSummary, x, y)
            // e.g. "Generated by: Harmony Rule #42, Score: +13, Reason: Passing Tone"
        else:
            hideTooltip()
```

**Performance:** `provenanceSummary` pre-fetched with note data; no IPC on every mousemove.

---

## 10. Interfaces

### 10.1 PianoRoll Props & Emits

```typescript
interface PianoRollProps {
  projectId: string;
  measureRange?: [number, number];
  syncScrollX?: number;
  syncZoomX?: number;
}

interface PianoRollEmits {
  (e: 'select-event', nodeId: string, additive: boolean): void;
  (e: 'scroll-sync', scrollX: number): void;
  (e: 'zoom-sync', zoomX: number): void;
  (e: 'seek', measure: number, beat: BeatOffset): void;
}
```

### 10.2 ProvenanceTooltip Component

```typescript
interface ProvenanceTooltipProps {
  visible: boolean;
  x: number;
  y: number;
  summary: string;
  pitchRole: PitchRole | null;
  source: ProvenanceSource;
}
```

Template:

```html
<div class="provenance-tooltip" v-if="visible" :style="{ left: x, top: y }">
  <span class="source-badge">{{ source }}</span>
  <p class="summary">{{ summary }}</p>
  <p class="hint" v-if="pitchRole">{{ pitchRole }}</p>
  <p class="click-hint">Click for full provenance</p>
</div>
```

### 10.3 Click → Inspector

```text
onNoteClick(noteId, additive):
    selectionStore.selectEvent(noteId, additive)
    // Inspector auto-updates via useProvenance(primaryEventId)
```

### 10.4 Toolbar Actions

| Tool | Action |
|------|--------|
| Select | Click select, drag move, edge resize |
| Draw | Click empty cell → insert Note with ManualEdit |
| Erase | Click delete → patch DeleteNode |
| Quantize | Snap selected notes to grid |

---

## 11. Parameter Mappings

| Parameter | Piano Roll Effect |
|-----------|-------------------|
| `rhythm.subdivision` | Snap grid fineness |
| `register.melody_register` | Highlight range band |
| `register.bass_register` | Highlight range band |
| `voice.voice_count` | Number of swimlanes |
| Zoom | UI state (not parameter) |

---

## 12. Explainability Model

### 12.1 Hover Display (Primary Requirement)

On hover, tooltip shows **one-line summary** from backend `display_summary`:

```text
Generated by: Harmony Rule #42, Score: +13, Reason: Passing Tone
```

Format produced by Tauri `get_provenance_chain` entry[0] at note load time.

### 12.2 Visual Encoding

Beyond tooltip:

- **Fill color** = `ProvenanceSource` (who/what created it)
- **Icon** = `PitchRole` (theoretical function)
- **Dashed border** = `Repaired` events
- **Glow** = selected + has constraint violation

### 12.3 Click Display

Click opens full Inspector ([inspector.md](inspector.md)):

- Complete provenance chain
- All contributing rules with individual scores
- Search context (step, beam rank)
- Parameter snapshot at generation
- Parent event link (repair/transform)

### 12.4 Multi-Select

Multiple selected notes: Inspector shows **aggregate** or **primary** (first selected) with tab switcher "1 of 3".

---

## 13. Future Expansion

| Feature | Version |
|---------|---------|
| Chord event as stacked block | v0.2 |
| MPE per-note expression | Phase 3 |
| Velocity lane (below grid) | v0.2 |
| Provenance filter (show only Generated) | v0.2 |
| MIDI controller input | Phase 3 |

---

## 14. Open Questions

| ID | Question | Status |
|----|----------|--------|
| OQ-PR-1 | Precompute all summaries at load? | Per visible range on load |
| OQ-PR-2 | Show harmony chord row? | Separate HarmonyLane v0.2 |
| OQ-PR-3 | Max notes before WebGL? | Benchmark 5k |

---

## 15. References

### Internal

- [inspector.md](inspector.md)
- [events.md](../02-music-model/events.md)
- [ast.md](../02-music-model/ast.md)
- [vue.md](vue.md)
- [philosophy.md](../00-overview/philosophy.md)

### External

- Signal piano roll (reference implementation)
- FL Studio MIDI editor UX patterns

---

*End of Piano Roll UI Specification*
