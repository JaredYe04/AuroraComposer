# Timeline UI Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Plugin & UI Research Agent  
**Dependencies:** [vue.md](vue.md), [timeline.md](../02-music-model/timeline.md), [ast.md](../02-music-model/ast.md), [tauri.md](tauri.md)

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

The Timeline View displays **form-level structure** — movements, sections, phrases, and measures — as a horizontal navigable strip. It is the primary spatial orientation component: users see where they are in the composition, select regions for regeneration, and sync playback position.

Unlike a DAW clip launcher, Aurora's timeline is **form-aware**, showing section roles (Verse, Chorus, Bridge) derived from AST `SectionMetadata`.

### 1.1 Scope

- Section/measure timeline component (`TimelineView.vue`)
- Zoom and scroll behavior
- Selection and playback cursor sync
- Integration with Piano Roll (vertical alignment)
- Section regeneration affordances

---

## 2. Existing Solutions

| Tool | Timeline Model | Relevant Feature |
|------|---------------|------------------|
| Ableton | Clip/scene grid | Horizontal zoom, loop region |
| FL Studio | Pattern blocks | Section color coding |
| Logic Pro | Marker track | Section labels |
| MuseScore | Linear measure ruler | Measure numbers |
| AIVA | Section blocks | Form labels (no edit) |

Aurora combines **measure ruler** (MuseScore) with **section blocks** (AIVA/DAW) and adds **regenerate section** actions.

---

## 3. Academic / Theoretical Foundation

### 3.1 Form Visualization

Music form theory (Caplin, 1998) distinguishes formal sections. Timeline encodes `SectionRole` visually — color, label, boundary markers — supporting compositional literacy.

### 3.2 Overview + Detail (Shneiderman)

Timeline is the **overview**; Piano Roll is **detail**. Linked brushing: selecting a section filters Piano Roll to that measure range.

---

## 4. Engineering Analysis

| Criterion | Response |
|-----------|----------|
| Scalability | Virtualize > 128 measures |
| Sync | Playback cursor at 60fps via `requestAnimationFrame` |
| Accessibility | Section labels readable at min zoom |

---

## 5. Comparison of Approaches

| Rendering | Pros | Cons | Verdict |
|-----------|------|------|---------|
| DOM divs | Simple, accessible | Slow > 200 sections | Rejected |
| SVG | Scalable, moderate perf | Complex interaction | Section layer |
| Canvas | Fast | Poor accessibility | Measure grid |
| **Hybrid SVG + Canvas** | Best of both | Two layers | **Selected** |

---

## 6. Recommended Solution

**Hybrid timeline**: Canvas for measure grid and playhead; SVG overlay for section blocks and labels.

---

## 7. Architecture

### 7.1 Component Hierarchy

```text
TimelineView.vue
├── TimelineRuler.vue        (measure numbers, time sig markers)
├── SectionTrack.vue           (colored section blocks)
├── PhraseTrack.vue            (optional phrase boundaries)
├── PlayheadOverlay.vue        (cursor synced to playback)
└── TimelineControls.vue       (zoom slider, snap toggle)
```

### 7.2 Layout

```text
┌─────────────────────────────────────────────────────────────┐
│ 1   2   3   4 │ 5   6   7   8 │ 9  10  11  12 │  ← measures │
├───────────────┴───────────────┴───────────────┤             │
│ [    Verse A (8 bars)    ][  Chorus (8)  ]... │  ← sections │
├───────────────────────────────────────────────┤             │
│                    ▼ playhead                 │             │
└─────────────────────────────────────────────────────────────┘
```

Vertical alignment: measure `N` left edge aligns with Piano Roll measure `N` column.

---

## 8. Data Structures

### 8.1 TimelineModel (from Tauri `get_timeline`)

```typescript
interface TimelineModel {
  total_measures: number;
  default_meter: TimeSignature;
  tempo_map: TempoMapEntry[];
  sections: TimelineSection[];
  phrases: TimelinePhrase[];
}

interface TimelineSection {
  id: string;
  role: SectionRole;
  label: string | null;
  start_measure: number;  // global, 1-based
  end_measure: number;    // inclusive
  key_area: KeySignature | null;
  theme_refs: string[];
}

interface TimelinePhrase {
  id: string;
  section_id: string;
  start_measure: number;
  end_measure: number;
  cadence: CadenceType | null;
}
```

### 8.2 View State

```typescript
interface TimelineViewState {
  zoom: number;           // pixels per measure, default 48
  scrollX: number;        // horizontal scroll offset
  snapToMeasure: boolean;
  selectedSectionId: string | null;
  selectedMeasureRange: [number, number] | null;
  hoverMeasure: number | null;
}
```

### 8.3 Section Role Colors

| SectionRole | Color (dark theme) |
|-------------|-------------------|
| Intro | `#4a5568` |
| Verse | `#3182ce` |
| Chorus | `#d69e2e` |
| Bridge | `#805ad5` |
| Outro | `#4a5568` |
| Default | `#2d3748` |

---

## 9. Algorithms

### 9.1 Measure → Pixel

```text
function measureToX(measure: number, zoom: number, scrollX: number):
    return (measure - 1) * zoom - scrollX
```

### 9.2 Pixel → Measure

```text
function xToMeasure(x: number, zoom: number, scrollX: number):
    return floor((x + scrollX) / zoom) + 1
```

### 9.3 Section Hit Test

```text
function hitTestSection(x, y, sections, zoom, scrollX):
    measure = xToMeasure(x, zoom, scrollX)
    return sections.find(s => s.start_measure <= measure <= s.end_measure)
```

### 9.4 Playhead Sync

```text
function syncPlayhead(currentTick, tempoMap, zoom, scrollX):
    globalBeat = tickToBeat(currentTick, tempoMap)
    measure = beatToMeasure(globalBeat)
    playheadX = measureToX(measure, zoom, scrollX)
    if playheadX outside viewport:
        scrollToCenter(measure)
```

### 9.5 Virtualization

Render only sections/measures in viewport ± 2 measures buffer:

```text
visibleStart = xToMeasure(viewport.left, zoom, scrollX) - 2
visibleEnd = xToMeasure(viewport.right, zoom, scrollX) + 2
```

---

## 10. Interfaces

### 10.1 TimelineView Props & Emits

```typescript
interface TimelineViewProps {
  model: TimelineModel;
  zoom?: number;
  playheadMeasure?: number | null;
}

interface TimelineViewEmits {
  (e: 'select-section', sectionId: string): void;
  (e: 'select-measure-range', range: [number, number]): void;
  (e: 'seek', measure: number): void;
  (e: 'regenerate-section', sectionId: string): void;
  (e: 'zoom-change', zoom: number): void;
}
```

### 10.2 Context Menu Actions

| Action | Trigger | Backend Command |
|--------|---------|-----------------|
| Regenerate Section | Right-click section | `regenerate_section` |
| Select All in Section | Click section header | Updates selection store |
| Set Loop Region | Drag on ruler | Updates playback store |
| Add Rehearsal Mark | Right-click measure | `apply_patch` (Marker) |

### 10.3 Keyboard Navigation

| Key | Action |
|-----|--------|
| `←` / `→` | Previous/next measure |
| `Shift+←/→` | Extend measure range selection |
| `Home` / `End` | First/last measure |
| `+` / `-` | Zoom in/out |

---

## 11. Parameter Mappings

| Parameter | Timeline Effect |
|-----------|-----------------|
| `form.section_count` | Number of section blocks |
| `form.section_lengths` | Block widths |
| `rhythm.subdivision` | Ruler subdivision ticks (when zoomed) |
| Playback position | Playhead (not a parameter) |

---

## 12. Explainability Model

### 12.1 Section-Level Provenance

Right-click section → "Show Section Provenance" opens Inspector with aggregated summary:

```typescript
interface SectionProvenanceSummary {
  section_id: string;
  dominant_stage: string;
  rule_frequency: Record<string, number>;
  event_count: number;
}
```

Computed backend-side on demand (not stored in AST).

### 12.2 Regeneration Feedback

When `regenerate_section` completes, timeline flashes section border green; Inspector shows diff summary ("14 events replaced, 2 repaired").

---

## 13. Future Expansion

| Feature | Version |
|---------|---------|
| Drag-to-reorder sections | v0.3 (structure patch) |
| Tempo ramp visualization | v0.2 |
| Key change markers | v0.2 |
| Mini waveform lane | Phase 3 |

---

## 14. Open Questions

| ID | Question | Status |
|----|----------|--------|
| OQ-TL-1 | Show phrase track by default? | Collapsed by default |
| OQ-TL-2 | Multi-movement tabs? | Yes, tab per Movement |

---

## 15. References

### Internal

- [timeline.md](../02-music-model/timeline.md)
- [vue.md](vue.md)
- [piano-roll.md](piano-roll.md)
- [tauri.md](tauri.md)

### External

- Caplin (1998), *Classical Form*
- Shneiderman (1996), *UI Design Principles*

---

*End of Timeline UI Specification*
