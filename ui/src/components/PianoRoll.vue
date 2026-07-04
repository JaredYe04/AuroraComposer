<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { PianoRollNote } from '@/types/aurora';
import { PROVENANCE_COLORS, nodeIdKey } from '@/types/aurora';
import {
  globalBeatToX,
  noteWidth,
  noteX,
  pitchLabel,
  pitchToY,
  totalMeasuresFromNotes,
  visiblePitchRange,
  xToGlobalBeat,
  xToMeasure,
} from '@/utils/pianoRoll';
import HorizontalScrollBar from '@/components/HorizontalScrollBar.vue';
import { useCompositionStore } from '@/stores/composition';
import { usePlaybackStore } from '@/stores/playback';
import { useSelectionStore } from '@/stores/selection';
import { useSettingsStore } from '@/stores/settings';

const props = defineProps<{
  notes: PianoRollNote[];
  beatsPerMeasure?: number;
}>();

const selection = useSelectionStore();
const compStore = useCompositionStore();
const playback = usePlaybackStore();
const settings = useSettingsStore();

const rootRef = ref<HTMLDivElement | null>(null);
const viewportRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const rulerRef = ref<HTMLCanvasElement | null>(null);

const KEY_WIDTH = 48;
const RULER_HEIGHT = 28;
const viewportSize = ref({ width: 400, height: 200 });

const beatsPerMeasure = computed(() => props.beatsPerMeasure ?? 4);
const totalMeasures = computed(() =>
  totalMeasuresFromNotes(props.notes, compStore.summary?.bars ?? 8),
);

const contentWidth = computed(
  () => KEY_WIDTH + totalMeasures.value * selection.zoomX,
);

const pitchRange = computed(() => visiblePitchRange(props.notes));
const minMidi = computed(() => pitchRange.value[0]);
const maxMidi = computed(() => pitchRange.value[1]);
const rowCount = computed(() => maxMidi.value - minMidi.value + 1);
const rowHeight = computed(() =>
  Math.max(8, Math.min(16, viewportSize.value.height / Math.max(1, rowCount.value))),
);

const filteredNotes = computed(() => {
  if (selection.selectedMeasure == null) return props.notes;
  return props.notes.filter((n) => n.startMeasure === selection.selectedMeasure);
});

const displayNotes = computed(() =>
  selection.selectedMeasure != null ? filteredNotes.value : props.notes,
);

const tooltip = ref<{ x: number; y: number; note: PianoRollNote } | null>(null);

interface DragState {
  note: PianoRollNote;
  originalPitch: number;
  currentPitch: number;
}

const dragState = ref<DragState | null>(null);
const dragPreviewPitch = ref<number | null>(null);
const rulerDragging = ref(false);

function yToPitch(y: number): number {
  const pitch = maxMidi.value - Math.round(y / rowHeight.value);
  return Math.max(0, Math.min(127, pitch));
}

function displayPitch(note: PianoRollNote): number {
  if (dragState.value && nodeIdKey(dragState.value.note.nodeId) === nodeIdKey(note.nodeId)) {
    return dragPreviewPitch.value ?? note.pitchMidi;
  }
  return note.pitchMidi;
}

function hitTest(x: number, y: number): PianoRollNote | null {
  for (const note of displayNotes.value) {
    const nx = noteX(note, beatsPerMeasure.value, selection.zoomX, selection.scrollX) + KEY_WIDTH;
    const ny = pitchToY(note.pitchMidi, maxMidi.value, rowHeight.value);
    const nw = Math.max(noteWidth(note, beatsPerMeasure.value, selection.zoomX), 4);
    const nh = rowHeight.value - 2;
    if (x >= nx && x <= nx + nw && y >= ny && y <= ny + nh) return note;
  }
  return null;
}

function playheadX(): number {
  return globalBeatToX(
    playback.globalBeat,
    beatsPerMeasure.value,
    selection.zoomX,
    selection.scrollX,
    KEY_WIDTH,
  );
}

function ensurePlayheadVisible() {
  const px = playheadX();
  const viewW = viewportSize.value.width;
  const margin = 64;
  if (px > selection.scrollX + viewW - margin) {
    selection.setScrollX(px - viewW + margin);
  } else if (px < selection.scrollX + KEY_WIDTH + margin) {
    selection.setScrollX(Math.max(0, px - KEY_WIDTH - margin));
  }
}

function cssColor(name: string, fallback: string): string {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return v || fallback;
}

function drawRuler() {
  const canvas = rulerRef.value;
  if (!canvas) return;
  const w = viewportSize.value.width;
  const h = RULER_HEIGHT;
  const dpr = window.devicePixelRatio || 1;
  canvas.width = w * dpr;
  canvas.height = h * dpr;
  canvas.style.width = `${w}px`;
  canvas.style.height = `${h}px`;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.scale(dpr, dpr);
  ctx.fillStyle = cssColor('--bg-panel', '#161b22');
  ctx.fillRect(0, 0, w, h);
  ctx.fillStyle = cssColor('--key-bg', '#161b22');
  ctx.fillRect(0, 0, KEY_WIDTH, h);

  ctx.fillStyle = cssColor('--text-muted', '#8b949e');
  ctx.font = '10px system-ui,sans-serif';
  ctx.textAlign = 'center';
  const startM = Math.max(1, xToMeasure(0, selection.zoomX, selection.scrollX));
  const endM = xToMeasure(w - KEY_WIDTH, selection.zoomX, selection.scrollX) + 1;
  for (let m = startM; m <= endM; m++) {
    const x = (m - 1) * selection.zoomX - selection.scrollX + KEY_WIDTH;
    ctx.strokeStyle = cssColor('--border-muted', '#30363d');
    ctx.beginPath();
    ctx.moveTo(x, h - 4);
    ctx.lineTo(x, h);
    ctx.stroke();
    ctx.fillText(String(m), x + selection.zoomX / 2, h - 10);
  }

  const px = playheadX();
  ctx.strokeStyle = cssColor('--playhead', '#f85149');
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(px, 0);
  ctx.lineTo(px, h);
  ctx.stroke();
  ctx.lineWidth = 1;

  ctx.fillStyle = cssColor('--playhead', '#f85149');
  ctx.beginPath();
  ctx.moveTo(px - 6, 0);
  ctx.lineTo(px + 6, 0);
  ctx.lineTo(px, 8);
  ctx.closePath();
  ctx.fill();
}

function drawGrid() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const w = viewportSize.value.width;
  const h = viewportSize.value.height;
  const dpr = window.devicePixelRatio || 1;
  canvas.width = w * dpr;
  canvas.height = h * dpr;
  canvas.style.width = `${w}px`;
  canvas.style.height = `${h}px`;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.scale(dpr, dpr);
  ctx.fillStyle = cssColor('--bg-input', '#0d1117');
  ctx.fillRect(0, 0, w, h);

  ctx.fillStyle = cssColor('--key-bg', '#161b22');
  ctx.fillRect(0, 0, KEY_WIDTH, h);
  ctx.fillStyle = cssColor('--text-muted', '#8b949e');
  ctx.font = '10px monospace';
  ctx.textAlign = 'right';
  for (let midi = maxMidi.value; midi >= minMidi.value; midi--) {
    const y = pitchToY(midi, maxMidi.value, rowHeight.value);
    if (midi % 12 === 0) {
      ctx.fillStyle = cssColor('--text-primary', '#e6edf3');
      ctx.fillText(pitchLabel(midi), KEY_WIDTH - 6, y + rowHeight.value - 3);
    }
    ctx.strokeStyle = midi % 12 === 0 ? cssColor('--border-muted', '#30363d') : cssColor('--border-subtle', '#21262d');
    ctx.beginPath();
    ctx.moveTo(KEY_WIDTH, y);
    ctx.lineTo(w, y);
    ctx.stroke();
  }

  const startM = Math.max(1, xToMeasure(KEY_WIDTH, selection.zoomX, selection.scrollX) - 1);
  const endM = xToMeasure(w, selection.zoomX, selection.scrollX) + 1;
  for (let m = startM; m <= endM; m++) {
    const x = (m - 1) * selection.zoomX - selection.scrollX + KEY_WIDTH;
    ctx.strokeStyle = m === selection.selectedMeasure ? cssColor('--accent', '#58a6ff') : cssColor('--border-muted', '#30363d');
    ctx.lineWidth = m === selection.selectedMeasure ? 2 : 1;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, h);
    ctx.stroke();
    ctx.lineWidth = 1;
  }

  for (const note of displayNotes.value) {
    const pitchMidi = displayPitch(note);
    const x = noteX(note, beatsPerMeasure.value, selection.zoomX, selection.scrollX) + KEY_WIDTH;
    const y = pitchToY(pitchMidi, maxMidi.value, rowHeight.value);
    const nw = Math.max(noteWidth(note, beatsPerMeasure.value, selection.zoomX), 4);
    const nh = rowHeight.value - 2;
    const colors = PROVENANCE_COLORS[note.provenanceSource];
    const selected = selection.isEventSelected(note.nodeId);
    const hovered = selection.hoveredEventId === nodeIdKey(note.nodeId);
    ctx.fillStyle = colors.fill;
    ctx.strokeStyle = selected ? '#fff' : colors.border;
    ctx.lineWidth = selected || hovered ? 2 : 1;
    ctx.setLineDash(note.provenanceSource === 'Repaired' ? [4, 2] : []);
    ctx.fillRect(x, y, nw, nh);
    ctx.strokeRect(x, y, nw, nh);
    ctx.setLineDash([]);
  }

  const px = playheadX();
  ctx.strokeStyle = cssColor('--playhead', '#f85149');
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(px, 0);
  ctx.lineTo(px, h);
  ctx.stroke();
  ctx.lineWidth = 1;
}

function redraw() {
  drawRuler();
  drawGrid();
}

function measureViewport() {
  if (!viewportRef.value) return;
  viewportSize.value = {
    width: viewportRef.value.clientWidth,
    height: viewportRef.value.clientHeight,
  };
  redraw();
}

function seekFromX(clientX: number, rulerEl: HTMLElement) {
  const rect = rulerEl.getBoundingClientRect();
  const x = clientX - rect.left;
  const beat = xToGlobalBeat(
    x,
    KEY_WIDTH,
    beatsPerMeasure.value,
    selection.zoomX,
    selection.scrollX,
    playback.totalGlobalBeats,
  );
  playback.seekToGlobalBeat(beat);
  redraw();
}

function onRulerMouseDown(e: MouseEvent) {
  e.preventDefault();
  rulerDragging.value = true;
  const rulerEl = e.currentTarget as HTMLElement;
  seekFromX(e.clientX, rulerEl);

  function onMove(ev: MouseEvent) {
    seekFromX(ev.clientX, rulerEl);
  }
  function onUp() {
    rulerDragging.value = false;
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  }
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}

function onMouseDown(e: MouseEvent) {
  const rect = viewportRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const note = hitTest(x, y);
  if (!note) return;
  e.preventDefault();
  selection.selectEvent(note.nodeId, e.shiftKey);
  dragState.value = { note, originalPitch: note.pitchMidi, currentPitch: note.pitchMidi };
  dragPreviewPitch.value = note.pitchMidi;
  tooltip.value = null;
}

function onClick(e: MouseEvent) {
  if (dragState.value && dragState.value.currentPitch !== dragState.value.originalPitch) return;
  const rect = viewportRef.value?.getBoundingClientRect();
  if (!rect) return;
  const note = hitTest(e.clientX - rect.left, e.clientY - rect.top);
  if (note) selection.selectEvent(note.nodeId, e.shiftKey);
}

async function finishDrag() {
  const drag = dragState.value;
  dragState.value = null;
  dragPreviewPitch.value = null;
  if (!drag || drag.currentPitch === drag.originalPitch) return;
  await compStore.patchNote(drag.note.nodeId, drag.currentPitch);
}

function onMouseMove(e: MouseEvent) {
  const rect = viewportRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  if (dragState.value) {
    dragPreviewPitch.value = yToPitch(y);
    dragState.value.currentPitch = dragPreviewPitch.value;
    redraw();
    return;
  }
  const note = hitTest(x, y);
  if (note) {
    selection.setHoveredEvent(note.nodeId);
    tooltip.value = { x: x + 12, y: y - 8, note };
  } else {
    selection.setHoveredEvent(null);
    tooltip.value = null;
  }
}

function onMouseLeave() {
  if (!dragState.value) {
    selection.setHoveredEvent(null);
    tooltip.value = null;
  }
}

function onWindowMouseUp() {
  if (dragState.value) {
    finishDrag().catch(() => {
      /* surfaced via store */
    });
  }
}

function onWheel(e: WheelEvent) {
  if (e.ctrlKey) {
    e.preventDefault();
    selection.setZoom(selection.zoomX + (e.deltaY > 0 ? -4 : 4));
  }
}

let ro: ResizeObserver | null = null;

watch(
  () => compStore.summary,
  (s) => {
    if (s) {
      playback.setTimelineContext(s.bars, s.tempo_bpm, beatsPerMeasure.value);
    }
  },
  { immediate: true },
);

watch(
  () => settings.theme,
  () => redraw(),
);

watch(
  () => [
    props.notes,
    displayNotes.value,
    selection.zoomX,
    selection.scrollX,
    selection.selectedMeasure,
    selection.selectedEventIds,
    selection.hoveredEventId,
    playback.globalBeat,
    playback.isPlaying,
    dragPreviewPitch.value,
    rowHeight.value,
  ],
  () => {
    redraw();
    if (playback.isPlaying) ensurePlayheadVisible();
  },
  { deep: true },
);

onMounted(() => {
  measureViewport();
  window.addEventListener('mouseup', onWindowMouseUp);
  if (rootRef.value) {
    ro = new ResizeObserver(() => measureViewport());
    ro.observe(rootRef.value);
  }
});

onUnmounted(() => {
  ro?.disconnect();
  window.removeEventListener('mouseup', onWindowMouseUp);
});
</script>

<template>
  <div ref="rootRef" class="piano-roll-root">
    <div
      class="transport-ruler"
      :class="{ dragging: rulerDragging }"
      @mousedown="onRulerMouseDown"
    >
      <canvas ref="rulerRef" class="ruler-canvas" />
    </div>

    <div
      ref="viewportRef"
      class="piano-viewport"
      :class="{ dragging: dragState !== null }"
      @mousedown="onMouseDown"
      @click="onClick"
      @mousemove="onMouseMove"
      @mouseleave="onMouseLeave"
      @wheel="onWheel"
    >
      <canvas ref="canvasRef" />
      <div
        v-if="tooltip"
        class="provenance-tooltip"
        :style="{ left: `${tooltip.x}px`, top: `${tooltip.y}px` }"
      >
        <span class="source-badge">{{ tooltip.note.provenanceSource }}</span>
        <p class="summary">{{ tooltip.note.provenanceSummary }}</p>
        <p v-if="tooltip.note.pitchRole" class="hint">{{ tooltip.note.pitchRole }}</p>
      </div>
      <p v-if="notes.length === 0" class="empty">No notes to display.</p>
    </div>

    <HorizontalScrollBar
      :scroll-x="selection.scrollX"
      :content-width="contentWidth"
      :viewport-width="viewportSize.width"
      @update:scroll-x="selection.setScrollX"
    />
  </div>
</template>

<style scoped>
.piano-roll-root {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: var(--panel-bg);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  overflow: hidden;
}

.transport-ruler {
  flex-shrink: 0;
  height: 28px;
  cursor: ew-resize;
  border-bottom: 1px solid var(--border-muted);
  overflow: hidden;
}

.transport-ruler.dragging {
  cursor: grabbing;
}

.ruler-canvas {
  display: block;
  width: 100%;
  height: 28px;
}

.piano-viewport {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  cursor: crosshair;
}

.piano-viewport.dragging {
  cursor: ns-resize;
}

canvas {
  display: block;
}

.provenance-tooltip {
  position: absolute;
  z-index: 10;
  max-width: 260px;
  padding: 0.5rem 0.75rem;
  background: var(--tooltip-bg);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  box-shadow: var(--shadow);
  pointer-events: none;
  font-size: 0.75rem;
}

.source-badge {
  display: inline-block;
  padding: 0.1rem 0.35rem;
  border-radius: 3px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 0.65rem;
  text-transform: uppercase;
  margin-bottom: 0.25rem;
}

.summary {
  margin: 0.25rem 0;
  color: var(--text-primary);
  line-height: 1.4;
}

.hint {
  margin: 0;
  color: var(--text-muted);
}

.empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 0.875rem;
  pointer-events: none;
}
</style>
