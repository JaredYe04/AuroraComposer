<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { PianoRollNote } from '@/types/aurora';
import { PROVENANCE_COLORS, nodeIdKey } from '@/types/aurora';
import {
  DRUM_MAP,
  drumMidiToRowIndex,
  drumRowCountForNotes,
  drumRowLabel,
  drumRowToY,
  globalBeatToX,
  isBlackKey,
  noteWidth,
  noteX,
  pitchLabel,
  pitchToY,
  rowIndexToDrumMidi,
  totalMeasuresFromNotes,
  visiblePitchRange,
  xToGlobalBeat,
  xToMeasure,
  yToDrumRowIndex,
} from '@/utils/pianoRoll';
import HorizontalScrollBar from '@/components/HorizontalScrollBar.vue';
import { useCompositionStore } from '@/stores/composition';
import { usePianoToolStore } from '@/stores/pianoTool';
import { usePlaybackStore } from '@/stores/playback';
import { useSelectionStore } from '@/stores/selection';
import { useSettingsStore } from '@/stores/settings';
import { useSnapGridStore } from '@/stores/snapGrid';

const props = defineProps<{
  notes: PianoRollNote[];
  beatsPerMeasure?: number;
  isDrum?: boolean;
}>();

const selection = useSelectionStore();
const compStore = useCompositionStore();
const playback = usePlaybackStore();
const pianoTool = usePianoToolStore();
const settings = useSettingsStore();
const snapGrid = useSnapGridStore();

const rootRef = ref<HTMLDivElement | null>(null);
const viewportRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const rulerRef = ref<HTMLCanvasElement | null>(null);

const KEY_WIDTH = 56;
const RULER_HEIGHT = 28;
const ROW_HEIGHT = 14;
const viewportSize = ref({ width: 400, height: 200 });
const scrollY = ref(0);

const beatsPerMeasure = computed(() => props.beatsPerMeasure ?? playback.beatsPerMeasure ?? 4);
const totalMeasures = computed(() =>
  totalMeasuresFromNotes(props.notes, compStore.summary?.bars ?? 8),
);

const contentWidth = computed(
  () => KEY_WIDTH + totalMeasures.value * selection.zoomX,
);

const pitchRange = computed((): [number, number] => {
  if (props.isDrum) return [0, Math.max(DRUM_MAP.length, drumRowCountForNotes(props.notes)) - 1];
  return visiblePitchRange(props.notes);
});
const minMidi = computed(() => pitchRange.value[0]);
const maxMidi = computed(() => pitchRange.value[1]);
const rowCount = computed(() =>
  props.isDrum ? drumRowCountForNotes(props.notes) : maxMidi.value - minMidi.value + 1,
);
const rowHeight = computed(() => ROW_HEIGHT);
const contentHeight = computed(() => rowCount.value * ROW_HEIGHT);
const maxScrollY = computed(() => Math.max(0, contentHeight.value - viewportSize.value.height));

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

interface BoxSelectState {
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
  additive: boolean;
}

const boxSelectState = ref<BoxSelectState | null>(null);

function pitchRowY(pitchMidi: number): number {
  if (props.isDrum) {
    return drumRowToY(drumMidiToRowIndex(pitchMidi), rowHeight.value);
  }
  return pitchToY(pitchMidi, minMidi.value, maxMidi.value, rowHeight.value);
}

function noteToY(pitchMidi: number): number {
  return pitchRowY(pitchMidi) - scrollY.value;
}

function yToPitch(y: number): number {
  const contentY = y + scrollY.value;
  if (props.isDrum) {
    return rowIndexToDrumMidi(yToDrumRowIndex(contentY, rowHeight.value, rowCount.value));
  }
  const pitch = maxMidi.value - Math.round(contentY / rowHeight.value);
  return Math.max(minMidi.value, Math.min(maxMidi.value, pitch));
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
    const ny = noteToY(displayPitch(note));
    const nw = Math.max(noteWidth(note, beatsPerMeasure.value, selection.zoomX), 4);
    const nh = rowHeight.value - 2;
    if (x >= nx && x <= nx + nw && y >= ny && y <= ny + nh) return note;
  }
  return null;
}

function notesInBox(x1: number, y1: number, x2: number, y2: number): PianoRollNote[] {
  const minX = Math.min(x1, x2);
  const maxX = Math.max(x1, x2);
  const minY = Math.min(y1, y2);
  const maxY = Math.max(y1, y2);
  return displayNotes.value.filter((note) => {
    const nx = noteX(note, beatsPerMeasure.value, selection.zoomX, selection.scrollX) + KEY_WIDTH;
    const ny = noteToY(displayPitch(note));
    const nw = Math.max(noteWidth(note, beatsPerMeasure.value, selection.zoomX), 4);
    const nh = rowHeight.value - 2;
    return nx + nw >= minX && nx <= maxX && ny + nh >= minY && ny <= maxY;
  });
}

function finishBoxSelect() {
  const box = boxSelectState.value;
  if (!box) return;
  const hits = notesInBox(box.startX, box.startY, box.currentX, box.currentY);
  selection.selectEvents(
    hits.map((n) => n.nodeId),
    box.additive,
  );
  boxSelectState.value = null;
  redraw();
}

function clickToInsertParams(x: number, y: number) {
  const gridX = Math.max(0, x - KEY_WIDTH);
  const measure = xToMeasure(gridX, selection.zoomX, selection.scrollX);
  const measureStartX = (measure - 1) * selection.zoomX - selection.scrollX;
  const beatInMeasure = ((gridX - measureStartX) / selection.zoomX) * beatsPerMeasure.value;
  const snappedBeat = snapGrid.snapBeat(beatInMeasure, beatsPerMeasure.value);
  const numer = Math.max(0, Math.floor(snappedBeat));
  const denom = snappedBeat % 1 > 0.001 ? Math.round(1 / (snappedBeat % 1)) : 1;
  return {
    measureGlobal: measure,
    voiceId: selection.activeVoiceId ?? 0,
    beatNumer: numer,
    beatDenom: denom,
    midi: yToPitch(y),
    isDrum: props.isDrum ?? false,
  };
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

  for (let row = 0; row < rowCount.value; row++) {
    const midi = props.isDrum
      ? rowIndexToDrumMidi(row)
      : maxMidi.value - row;
    const y = row * rowHeight.value - scrollY.value;
    if (y + rowHeight.value < 0 || y > h) continue;

    const black = !props.isDrum && isBlackKey(midi);
    ctx.fillStyle = black
      ? cssColor('--key-black', '#0d1117')
      : cssColor('--key-white', '#21262d');
    ctx.fillRect(0, y, KEY_WIDTH, rowHeight.value - 1);

    ctx.fillStyle = black ? cssColor('--text-muted', '#8b949e') : cssColor('--text-primary', '#e6edf3');
    ctx.font = black ? '8px monospace' : '9px monospace';
    ctx.textAlign = 'right';
    const label = props.isDrum ? drumRowLabel(midi) : pitchLabel(midi);
    ctx.fillText(label, KEY_WIDTH - 4, y + rowHeight.value - 4);

    ctx.strokeStyle = black ? cssColor('--border-subtle', '#21262d') : cssColor('--border-muted', '#30363d');
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
    const y = noteToY(pitchMidi);
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

  const box = boxSelectState.value;
  if (box) {
    const bx = Math.min(box.startX, box.currentX);
    const by = Math.min(box.startY, box.currentY);
    const bw = Math.abs(box.currentX - box.startX);
    const bh = Math.abs(box.currentY - box.startY);
    ctx.fillStyle = 'rgba(88, 166, 255, 0.12)';
    ctx.strokeStyle = cssColor('--accent', '#58a6ff');
    ctx.lineWidth = 1;
    ctx.setLineDash([4, 2]);
    ctx.fillRect(bx, by, bw, bh);
    ctx.strokeRect(bx, by, bw, bh);
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

  if (pianoTool.mode === 'brush') {
    e.preventDefault();
    void compStore.insertNote(clickToInsertParams(x, y));
    return;
  }

  if (pianoTool.mode === 'box') {
    e.preventDefault();
    boxSelectState.value = {
      startX: x,
      startY: y,
      currentX: x,
      currentY: y,
      additive: e.shiftKey,
    };
    if (!e.shiftKey) {
      selection.clearSelection();
    }
    redraw();
    return;
  }

  if (pianoTool.mode === 'eraser') {
    const note = hitTest(x, y);
    if (note) {
      e.preventDefault();
      void compStore.deleteNote(note.nodeId);
    }
    return;
  }

  const note = hitTest(x, y);
  if (!note) return;
  e.preventDefault();
  selection.selectEvent(note.nodeId, e.shiftKey);
  if (pianoTool.mode !== 'pointer') return;
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
  if (boxSelectState.value) {
    boxSelectState.value.currentX = x;
    boxSelectState.value.currentY = y;
    redraw();
    return;
  }
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
  if (boxSelectState.value) {
    finishBoxSelect();
    return;
  }
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
    return;
  }
  e.preventDefault();
  scrollY.value = Math.max(0, Math.min(maxScrollY.value, scrollY.value + e.deltaY));
  redraw();
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
  () => [props.notes, props.isDrum, minMidi.value, maxMidi.value],
  () => {
    scrollY.value = Math.min(scrollY.value, maxScrollY.value);
    if (props.notes.length === 0 || props.isDrum) return;
    let min = 127;
    let max = 0;
    for (const n of props.notes) {
      min = Math.min(min, n.pitchMidi);
      max = Math.max(max, n.pitchMidi);
    }
    const mid = (min + max) / 2;
    const centerY = pitchRowY(mid);
    scrollY.value = Math.max(
      0,
      Math.min(maxScrollY.value, centerY - viewportSize.value.height / 2),
    );
  },
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
    boxSelectState.value,
    rowHeight.value,
    scrollY.value,
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
      :class="{
        dragging: dragState !== null,
        'tool-box': pianoTool.mode === 'box',
        'tool-brush': pianoTool.mode === 'brush',
        'tool-eraser': pianoTool.mode === 'eraser',
      }"
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
  cursor: default;
}

.piano-viewport.tool-box {
  cursor: crosshair;
}

.piano-viewport.tool-brush {
  cursor: cell;
}

.piano-viewport.tool-eraser {
  cursor: not-allowed;
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
