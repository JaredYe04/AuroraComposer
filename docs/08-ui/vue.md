# Vue 3 Frontend Architecture Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Plugin & UI Research Agent  
**Dependencies:** [tauri.md](tauri.md), [ADR-002](../decisions/ADR-002-vue3-frontend.md), [architecture.md](../01-architecture/architecture.md), [ast.md](../02-music-model/ast.md)

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

The Vue 3 frontend is Aurora Composer's **Layer 5 Presentation** — parameter controls, timeline, piano roll, inspector, audio player, and plugin manager. It communicates exclusively with the Rust engine via Tauri IPC; it never generates music or evaluates rules locally.

### 1.1 Problem Statement

The frontend must:

- Provide reactive UI for 50+ composition parameters
- Display and edit AST-backed musical data
- Show **provenance on every selected event** (philosophy Principle 2)
- Handle async generation jobs with progress feedback
- Support undo/redo of AST patches
- Remain performant with 1000+ visible note events

### 1.2 Scope

- Application structure (Vue 3 Composition API)
- Pinia store design
- Vue Router routes
- Component hierarchy
- Tauri integration layer
- Audio preview integration

Component-level specs: [timeline-ui.md](timeline-ui.md), [piano-roll.md](piano-roll.md), [inspector.md](inspector.md).

---

## 2. Existing Solutions

### 2.1 DAW UI Patterns (Ableton, FL Studio)

Track-based timeline, piano roll, mixer. Mature UX but no explainability.

**Relevant:** Layout metaphors, keyboard shortcuts.

### 2.2 MuseScore (Qt)

Notation-centric. Strong engraving, weak generation UI.

### 2.3 Web Audio DAWs (BandLab, Soundtrap)

Vue/React SPAs with WebAudio. Browser-based.

**Relevant:** Tone.js integration, Vue component patterns.

### 2.4 Tauri + Vue Starters

Official `create-tauri-app` Vue template with Vite.

**Relevant:** Build pipeline, invoke wrappers.

---

## 3. Academic / Theoretical Foundation

### 3.1 Model-View-ViewModel (MVVM)

Vue's reactivity implements MVVM: Pinia stores hold view-model state; components are declarative views. Musical truth remains in backend AST — frontend holds **projections** and **selection state** only.

### 3.2 Direct Manipulation (Shneiderman)

Piano roll supports direct manipulation of notes; provenance Inspector provides feedback on system actions — combining direct manipulation with explainability.

---

## 4. Engineering Analysis

| Criterion | Design Response |
|-----------|-----------------|
| Reactivity | Pinia + computed projections |
| Type safety | TypeScript strict, generated IPC types |
| Performance | Virtualized lists, Canvas piano roll |
| Accessibility | Keyboard nav, ARIA on Inspector |
| Testability | Vitest + Vue Test Utils |

---

## 5. Comparison of Approaches

| State Management | Pros | Cons | Verdict |
|------------------|------|------|---------|
| Pinia | Official, modular, TS | — | **Selected** |
| Composables only | Lightweight | Scattered for large app | Supplement |
| Vuex 4 | Mature | Legacy | Rejected |
| Redux | — | Not Vue-native | Rejected |

| UI Library | Pros | Cons | Verdict |
|------------|------|------|---------|
| Naive UI | Vue 3, complete | Custom theming | **Selected v0.1** |
| Vuetify | Material | Heavier | Alternative |
| Custom only | Full control | Slow to build | Partial (music views) |

---

## 6. Recommended Solution

**Vue 3 + TypeScript + Vite + Pinia + Vue Router + Naive UI** with:

1. Feature-based folder structure
2. Pinia stores mirroring backend domains
3. Thin Tauri service layer
4. Custom Canvas components for music views
5. Tone.js for IR playback

---

## 7. Architecture

### 7.1 Directory Structure

```text
src/
├── main.ts
├── App.vue
├── router/
│   └── index.ts
├── stores/
│   ├── project.ts
│   ├── composition.ts
│   ├── parameters.ts
│   ├── generation.ts
│   ├── selection.ts
│   ├── plugins.ts
│   └── playback.ts
├── services/
│   └── tauri/
│       ├── commands.ts
│       ├── events.ts
│       └── types.ts
├── views/
│   ├── ComposerView.vue      # main workspace
│   ├── ProjectBrowser.vue
│   ├── PluginManager.vue
│   └── SettingsView.vue
├── components/
│   ├── layout/
│   │   ├── AppShell.vue
│   │   ├── MenuBar.vue
│   │   └── StatusBar.vue
│   ├── parameters/
│   │   ├── ParameterPanel.vue
│   │   └── EmotionPicker.vue
│   ├── timeline/
│   │   └── TimelineView.vue      # see timeline-ui.md
│   ├── piano-roll/
│   │   └── PianoRoll.vue         # see piano-roll.md
│   ├── inspector/
│   │   └── EventInspector.vue    # see inspector.md
│   ├── player/
│   │   └── TransportBar.vue
│   └── export/
│       └── ExportDialog.vue
├── composables/
│   ├── useTauriCommand.ts
│   ├── useJobProgress.ts
│   ├── useKeyboardShortcuts.ts
│   └── useProvenance.ts
└── assets/
```

### 7.2 Main Workspace Layout

```text
┌────────────────────────────────────────────────────────────┐
│ MenuBar · TransportBar · Job Progress                      │
├──────────────┬─────────────────────────────┬───────────────┤
│              │                             │               │
│  Parameter   │      TimelineView           │   Inspector   │
│  Panel       ├─────────────────────────────┤   (Provenance)│
│  (240px)     │      PianoRoll              │   (280px)     │
│              │                             │               │
├──────────────┴─────────────────────────────┴───────────────┤
│ StatusBar · Plugin indicators · Validation warnings        │
└────────────────────────────────────────────────────────────┘
```

### 7.3 Data Flow

```text
User Action → Component → Pinia Store Action → Tauri Command
                ↑                                    │
                └──────── Store Mutation ←──────────┘
                              ↑
                    Tauri Event (job-progress, ast-changed)
```

Frontend **never** mutates composition data except via `apply_patch` command.

---

## 8. Data Structures

### 8.1 Pinia Store: Project

```typescript
export const useProjectStore = defineStore('project', () => {
  const projectId = ref<string | null>(null);
  const metadata = ref<ProjectMetadata | null>(null);
  const recentProjects = ref<RecentProject[]>([]);
  const isDirty = ref(false);

  async function createProject(name: string): Promise<void>;
  async function openProject(path: string): Promise<void>;
  async function saveProject(): Promise<void>;
  async function closeProject(): Promise<void>;

  return { projectId, metadata, recentProjects, isDirty, /* actions */ };
});
```

### 8.2 Pinia Store: Composition

```typescript
export const useCompositionStore = defineStore('composition', () => {
  const composition = ref<Composition | null>(null);
  const timeline = ref<TimelineModel | null>(null);
  const validationReport = ref<ValidationReport | null>(null);
  const canUndo = ref(false);
  const canRedo = ref(false);

  async function loadComposition(): Promise<void>;
  async function applyPatch(patch: Patch): Promise<ValidationReport>;
  async function undo(): Promise<void>;
  async function redo(): Promise<void>;

  return { composition, timeline, validationReport, canUndo, canRedo };
});
```

### 8.3 Pinia Store: Selection

```typescript
export const useSelectionStore = defineStore('selection', () => {
  const selectedEventIds = ref<Set<string>>(new Set());
  const selectedMeasure = ref<number | null>(null);
  const selectedSectionId = ref<string | null>(null);
  const hoveredEventId = ref<string | null>(null);
  const primaryEventId = computed(() => /* first selected */);

  function selectEvent(nodeId: string, additive = false): void;
  function clearSelection(): void;

  return { selectedEventIds, primaryEventId, hoveredEventId, selectEvent };
});
```

### 8.4 Pinia Store: Generation

```typescript
export const useGenerationStore = defineStore('generation', () => {
  const activeJobId = ref<string | null>(null);
  const progress = ref<JobProgress | null>(null);
  const isGenerating = computed(() => activeJobId.value !== null);

  async function generate(parameters: ParameterSnapshot): Promise<void>;
  async function cancel(): Promise<void>;

  // Subscribes to aurora://job-progress on mount
  return { activeJobId, progress, isGenerating, generate, cancel };
});
```

### 8.5 Pinia Store: Parameters

```typescript
export const useParameterStore = defineStore('parameters', () => {
  const snapshot = ref<ParameterSnapshot>(defaultParameters());
  const schema = ref<ParameterSchema | null>(null);

  async function load(projectId: string): Promise<void>;
  async function set(key: string, value: unknown): Promise<void>;
  async function reset(category?: string): Promise<void>;

  return { snapshot, schema, load, set, reset };
});
```

### 8.6 Pinia Store: Plugins

```typescript
export const usePluginStore = defineStore('plugins', () => {
  const installed = ref<PluginDescriptor[]>([]);
  const stylePresets = ref<StylePreset[]>([]);

  async function refresh(): Promise<void>;
  async function enable(pluginId: string): Promise<void>;
  async function disable(pluginId: string): Promise<void>;

  return { installed, stylePresets, refresh, enable, disable };
});
```

### 8.7 Pinia Store: Playback

```typescript
export const usePlaybackStore = defineStore('playback', () => {
  const isPlaying = ref(false);
  const currentTick = ref(0);
  const musicIr = ref<MusicIr | null>(null);

  async function loadIr(): Promise<void>;
  function play(): void;
  function pause(): void;
  function seek(tick: number): void;

  return { isPlaying, currentTick, play, pause, seek };
});
```

---

## 9. Algorithms

### 9.1 App Bootstrap

```text
function bootstrap():
    createPinia()
    createRouter()
    registerTauriEventListeners()
    onMounted:
        loadParameterSchema()
        listRecentProjects()
```

### 9.2 Job Progress Subscription

```text
function useJobProgress():
    onMounted:
        unlisten = listen('aurora://job-progress', handler)
        unlistenComplete = listen('aurora://job-complete', onComplete)
    onComplete:
        compositionStore.loadComposition()
        generationStore.clearJob()
    onUnmounted:
        unlisten()
```

### 9.3 Selection → Inspector Sync

```text
watch(primaryEventId):
    if id:
        chain = await getProvenanceChain(projectId, id)
        inspectorStore.setChain(chain)
    else:
        inspectorStore.clear()
```

---

## 10. Interfaces

### 10.1 Vue Router Routes

| Path | Component | Description |
|------|-----------|-------------|
| `/` | `ProjectBrowser` | Recent projects, create/open |
| `/composer/:projectId` | `ComposerView` | Main workspace |
| `/plugins` | `PluginManager` | Install, enable, configure |
| `/settings` | `SettingsView` | App preferences |

```typescript
const routes = [
  { path: '/', name: 'home', component: () => import('@/views/ProjectBrowser.vue') },
  { path: '/composer/:projectId', name: 'composer', component: () => import('@/views/ComposerView.vue') },
  { path: '/plugins', name: 'plugins', component: () => import('@/views/PluginManager.vue') },
  { path: '/settings', name: 'settings', component: () => import('@/views/SettingsView.vue') },
];
```

### 10.2 Component Props Interfaces

```typescript
// ParameterPanel.vue
interface ParameterPanelProps {
  collapsed?: boolean;
}

// ComposerView.vue — orchestrates child panels
interface ComposerViewProps {
  projectId: string;
}
```

### 10.3 Composable: useProvenance

```typescript
export function useProvenance(eventId: Ref<string | null>) {
  const chain = ref<ProvenanceChain | null>(null);
  const summary = computed(() => chain.value?.entries[0]?.display_summary ?? null);
  const loading = ref(false);

  watch(eventId, async (id) => {
    if (!id) { chain.value = null; return; }
    loading.value = true;
    chain.value = await getProvenanceChain(projectStore.projectId!, id);
    loading.value = false;
  }, { immediate: true });

  return { chain, summary, loading };
}
```

### 10.4 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play/pause |
| `G` | Generate (with confirmation if dirty) |
| `Escape` | Clear selection |
| `I` | Focus Inspector |
| `Ctrl+Z` / `Ctrl+Y` | Undo/redo |
| `Delete` | Delete selected events (patch) |

---

## 11. Parameter Mappings

Parameter Panel organized by ACAS categories:

| Panel Section | Store Keys | Component |
|---------------|-----------|-----------|
| Style | `style.*` | `StylePresetSelect` |
| Emotion | `emotion.*` | `EmotionPicker` (2D valence/arousal) |
| Form | `form.*` | `FormEditor` |
| Harmony | `harmony.*` | `HarmonySliders` |
| Melody | `melody.*` | `MelodySliders` |
| Rhythm | `rhythm.*` | `RhythmSliders` |
| Voice | `voice.*`, `register.*` | `VoiceConfig` |
| Search | `search.*` | `AdvancedSearchPanel` (collapsed) |

Changes debounce 300ms then call `set_parameters`. Generate button sends full snapshot.

---

## 12. Explainability Model

### 12.1 UI Surfaces for Provenance

| Surface | Trigger | Content |
|---------|---------|---------|
| Inspector panel | Click note | Full chain |
| Piano roll tooltip | Hover note | `display_summary` one-liner |
| Status bar | Single selection | Truncated summary |
| Validation panel | Constraint violation | Rule ID + measure |

### 12.2 Primary Display Format

Per philosophy requirement:

```text
Generated by: Harmony Rule #42, Score: +13, Reason: Passing Tone
```

Rendered by `EventInspector.vue` from `ProvenanceChainEntry.display_summary`. See [inspector.md](inspector.md).

---

## 13. Future Expansion

| Feature | Version |
|---------|---------|
| Split editor views | v0.2 |
| Theming (dark/light) | v0.1 |
| i18n (vue-i18n) | v0.2 |
| Notation view (Verovio) | v0.2 |
| Mobile Tauri (read-only) | Phase 4 |

---

## 14. Open Questions

| ID | Question | Status |
|----|----------|--------|
| OQ-VUE-1 | Naive UI vs custom design system? | Naive UI v0.1 |
| OQ-VUE-2 | Pinia persistence plugin for params? | Defer |
| OQ-VUE-3 | WebGL piano roll fallback? | Only if Canvas insufficient |

---

## 15. References

### Internal

- [tauri.md](tauri.md)
- [timeline-ui.md](timeline-ui.md)
- [piano-roll.md](piano-roll.md)
- [inspector.md](inspector.md)
- [ADR-002](../decisions/ADR-002-vue3-frontend.md)

### External

- Vue 3 Composition API: https://vuejs.org/guide/
- Pinia: https://pinia.vuejs.org/
- Naive UI: https://www.naiveui.com/
- Tone.js: https://tonejs.github.io/

---

*End of Vue 3 Frontend Architecture Specification*
