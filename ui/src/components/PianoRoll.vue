<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { PianoRollNote } from '@/types/aurora';
import { PROVENANCE_COLORS, nodeIdKey } from '@/types/aurora';
import {
  noteWidth,
  noteX,
  pitchLabel,
  pitchToY,
  visiblePitchRange,
  xToMeasure,
} from '@/utils/pianoRoll';
import { useSelectionStore } from '@/stores/selection';

const props = defineProps<{
  notes: PianoRollNote[];
  beatsPerMeasure?: number;
  playheadMeasure?: number | null;
}>();

const selection = useSelectionStore();
const containerRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const KEY_WIDTH = 48;
const rowHeight = 14;
const beatsPerMeasure = computed(() => props.beatsPerMeasure ?? 4);

const tooltip = ref<{ x: number; y: number; note: PianoRollNote } | null>(null);

const pitchRange = computed(() => visiblePitchRange(props.notes));
const minMidi = computed(() => pitchRange.value[0]);
const maxMidi = computed(() => pitchRange.value[1]);
const gridHeight = computed(() => (maxMidi.value - minMidi.value + 1) * rowHeight);

const filteredNotes = computed(() => {
  if (selection.selectedMeasure == null) return props.notes;
  return props.notes.filter((n) => n.startMeasure === selection.selectedMeasure);
});

const displayNotes = computed(() =>
  selection.selectedMeasure != null ? filteredNotes.value : props.notes,
);

function hitTest(x: number, y: number): PianoRollNote | null {
  for (const note of displayNotes.value) {
    const nx = noteX(note, beatsPerMeasure.value, selection.zoomX, selection.scrollX) + KEY_WIDTH;
    const ny = pitchToY(note.pitchMidi, maxMidi.value, rowHeight) + 24;
    const nw = Math.max(noteWidth(note, beatsPerMeasure.value, selection.zoomX), 4);
    const nh = rowHeight - 2;
    if (x >= nx && x <= nx + nw && y >= ny && y <= ny + nh) return note;
  }
  return null;
}

function draw() {
  const canvas = canvasRef.value;
  const container = containerRef.value;
  if (!canvas || !container) return;

  const dpr = window.devicePixelRatio || 1;
  const w = container.clientWidth;
  const h = Math.max(gridHeight.value + 24, 200);
  canvas.width = w * dpr;
  canvas.height = h * dpr;
  canvas.style.width = `${w}px`;
  canvas.style.height = `${h}px`;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.scale(dpr, dpr);
  ctx.fillStyle = '#0d1117';
  ctx.fillRect(0, 0, w, h);

  // Keyboard labels
  ctx.fillStyle = '#161b22';
  ctx.fillRect(0, 0, KEY_WIDTH, h);
  ctx.fillStyle = '#8b949e';
  ctx.font = '10px monospace';
  ctx.textAlign = 'right';
  for (let midi = maxMidi.value; midi >= minMidi.value; midi--) {
    const y = pitchToY(midi, maxMidi.value, rowHeight) + 24;
    if (midi % 12 === 0) {
      ctx.fillStyle = '#c9d1d9';
      ctx.fillText(pitchLabel(midi), KEY_WIDTH - 6, y + rowHeight - 3);
    }
    ctx.strokeStyle = midi % 12 === 0 ? '#30363d' : '#21262d';
    ctx.beginPath();
    ctx.moveTo(KEY_WIDTH, y);
    ctx.lineTo(w, y);
    ctx.stroke();
  }

  // Measure grid
  const startM = Math.max(1, xToMeasure(KEY_WIDTH, selection.zoomX, selection.scrollX) - 1);
  const endM = xToMeasure(w, selection.zoomX, selection.scrollX) + 1;
  for (let m = startM; m <= endM; m++) {
    const x = (m - 1) * selection.zoomX - selection.scrollX + KEY_WIDTH;
    ctx.strokeStyle = m === selection.selectedMeasure ? '#58a6ff' : '#30363d';
    ctx.lineWidth = m === selection.selectedMeasure ? 2 : 1;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, h);
    ctx.stroke();
    ctx.lineWidth = 1;
  }

  // Notes
  for (const note of displayNotes.value) {
    const x = noteX(note, beatsPerMeasure.value, selection.zoomX, selection.scrollX) + KEY_WIDTH;
    const y = pitchToY(note.pitchMidi, maxMidi.value, rowHeight) + 24;
    const nw = Math.max(noteWidth(note, beatsPerMeasure.value, selection.zoomX), 4);
    const nh = rowHeight - 2;
    const colors = PROVENANCE_COLORS[note.provenanceSource];
    const selected = selection.isEventSelected(note.nodeId);
    const hovered = selection.hoveredEventId === nodeIdKey(note.nodeId);

    ctx.fillStyle = colors.fill;
    ctx.strokeStyle = selected ? '#fff' : colors.border;
    ctx.lineWidth = selected || hovered ? 2 : 1;
    if (note.provenanceSource === 'Repaired') {
      ctx.setLineDash([4, 2]);
    } else {
      ctx.setLineDash([]);
    }
    ctx.fillRect(x, y, nw, nh);
    ctx.strokeRect(x, y, nw, nh);
    ctx.setLineDash([]);
  }

  // Playhead
  if (props.playheadMeasure != null) {
    const px = (props.playheadMeasure - 1) * selection.zoomX - selection.scrollX + KEY_WIDTH;
    ctx.strokeStyle = '#f85149';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(px, 0);
    ctx.lineTo(px, h);
    ctx.stroke();
    ctx.lineWidth = 1;
  }
}

function onClick(e: MouseEvent) {
  const rect = containerRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const note = hitTest(x, y);
  if (note) {
    selection.selectEvent(note.nodeId, e.shiftKey);
  }
}

function onMouseMove(e: MouseEvent) {
  const rect = containerRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const note = hitTest(x, y);
  if (note) {
    selection.setHoveredEvent(note.nodeId);
    tooltip.value = { x: e.clientX - rect.left + 12, y: e.clientY - rect.top - 8, note };
  } else {
    selection.setHoveredEvent(null);
    tooltip.value = null;
  }
}

function onMouseLeave() {
  selection.setHoveredEvent(null);
  tooltip.value = null;
}

function onWheel(e: WheelEvent) {
  if (e.shiftKey) {
    e.preventDefault();
    selection.setScrollX(selection.scrollX + e.deltaY);
  } else if (e.ctrlKey) {
    e.preventDefault();
    selection.setZoom(selection.zoomX + (e.deltaY > 0 ? -4 : 4));
  }
}

let ro: ResizeObserver | null = null;

onMounted(() => {
  draw();
  if (containerRef.value) {
    ro = new ResizeObserver(() => draw());
    ro.observe(containerRef.value);
  }
});

onUnmounted(() => ro?.disconnect());

watch(
  () => [
    props.notes,
    displayNotes.value,
    selection.zoomX,
    selection.scrollX,
    selection.selectedMeasure,
    selection.selectedEventIds,
    selection.hoveredEventId,
    props.playheadMeasure,
  ],
  () => draw(),
  { deep: true },
);
</script>

<template>
  <div
    ref="containerRef"
    class="piano-roll"
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
      <p class="click-hint">Click for full provenance</p>
    </div>
    <p v-if="notes.length === 0" class="empty">No notes to display.</p>
  </div>
</template>

<style scoped>
.piano-roll {
  position: relative;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
  overflow: auto;
  min-height: 200px;
  cursor: crosshair;
}

canvas {
  display: block;
}

.provenance-tooltip {
  position: absolute;
  z-index: 10;
  max-width: 280px;
  padding: 0.5rem 0.75rem;
  background: #1c2128;
  border: 1px solid #444c56;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  pointer-events: none;
  font-size: 0.75rem;
}

.source-badge {
  display: inline-block;
  padding: 0.1rem 0.35rem;
  border-radius: 3px;
  background: #388bfd26;
  color: #58a6ff;
  font-size: 0.65rem;
  text-transform: uppercase;
  margin-bottom: 0.25rem;
}

.summary {
  margin: 0.25rem 0;
  color: #e6edf3;
  line-height: 1.4;
}

.hint {
  margin: 0;
  color: #8b949e;
}

.click-hint {
  margin: 0.35rem 0 0;
  color: #6e7681;
  font-size: 0.65rem;
}

.empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #8b949e;
  font-size: 0.875rem;
  pointer-events: none;
}
</style>
