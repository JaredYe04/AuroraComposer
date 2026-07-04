<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useI18n } from '@/composables/useI18n';
import { usePatternsStore } from '@/stores/patterns';
import { usePianoToolStore } from '@/stores/pianoTool';
import { usePlaybackStore } from '@/stores/playback';
import { usePlaylistStore } from '@/stores/playlist';
import { useSelectionStore } from '@/stores/selection';
import { useSnapGridStore } from '@/stores/snapGrid';
import { globalBeatToX, xToGlobalBeat } from '@/utils/pianoRoll';

const ROW_HEIGHT = 48;
const LABEL_WIDTH = 120;
const BEAT_WIDTH = 24;

const { t } = useI18n();
const patterns = usePatternsStore();
const playlist = usePlaylistStore();
const pianoTool = usePianoToolStore();
const playback = usePlaybackStore();
const selection = useSelectionStore();
const snapGrid = useSnapGridStore();

const rootRef = ref<HTMLDivElement | null>(null);
const viewportRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const viewportSize = ref({ width: 600, height: 200 });

const selectedClipId = ref<string | null>(null);
const dragClipId = ref<string | null>(null);
const resizeClipId = ref<string | null>(null);
const splitLine = ref<{ x1: number; y1: number; x2: number; y2: number } | null>(null);
const splitDragging = ref(false);

const beatsPerMeasure = computed(() => playback.beatsPerMeasure);
const cellBeats = computed(() => beatsPerMeasure.value);
const beatWidth = computed(() => BEAT_WIDTH * (selection.zoomX / 48));

const activePattern = computed(() =>
  patterns.patterns.find((p) => p.id === patterns.activePatternId),
);

function patternById(id: string) {
  return patterns.patterns.find((p) => p.id === id);
}

function clipRect(clip: { startBeat: number; durationBeats: number }) {
  return {
    x: LABEL_WIDTH + clip.startBeat * beatWidth.value - selection.scrollX,
    w: clip.durationBeats * beatWidth.value,
  };
}

function cssColor(name: string, fallback: string): string {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return v || fallback;
}

function hitClip(x: number, y: number) {
  if (y < 0 || y > ROW_HEIGHT) return null;
  for (const clip of [...playlist.clips].reverse()) {
    const { x: cx, w } = clipRect(clip);
    if (x >= cx && x <= cx + w) return clip;
  }
  return null;
}

function isResizeEdge(clip: { startBeat: number; durationBeats: number }, x: number): boolean {
  const { x: cx, w } = clipRect(clip);
  return x >= cx + w - 8 && x <= cx + w;
}

function draw() {
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
  ctx.fillRect(0, 0, LABEL_WIDTH, h);

  const gridSize = cellBeats.value / 4;
  const startBeat = Math.max(0, selection.scrollX / beatWidth.value);
  const endBeat = startBeat + (w - LABEL_WIDTH) / beatWidth.value + 1;

  for (let beat = Math.floor(startBeat / gridSize) * gridSize; beat <= endBeat; beat += gridSize) {
    const x = LABEL_WIDTH + beat * beatWidth.value - selection.scrollX;
    ctx.strokeStyle =
      beat % cellBeats.value === 0
        ? cssColor('--border-muted', '#30363d')
        : cssColor('--border-subtle', '#21262d');
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, h);
    ctx.stroke();
  }

  ctx.fillStyle = cssColor('--text-muted', '#8b949e');
  ctx.font = '11px system-ui,sans-serif';
  ctx.textAlign = 'left';
  ctx.fillText(t('playlist.rowLabel'), 8, ROW_HEIGHT / 2 + 4);

  for (const clip of playlist.clips) {
    const pat = patternById(clip.patternId);
    const { x, w } = clipRect(clip);
    const y = 4;
    const nh = ROW_HEIGHT - 8;
    ctx.fillStyle = pat?.color ?? '#58a6ff';
    ctx.globalAlpha = selectedClipId.value === clip.id ? 1 : 0.85;
    ctx.fillRect(x, y, w, nh);
    ctx.globalAlpha = 1;
    ctx.strokeStyle =
      selectedClipId.value === clip.id ? '#fff' : 'rgba(0,0,0,0.25)';
    ctx.lineWidth = selectedClipId.value === clip.id ? 2 : 1;
    ctx.strokeRect(x, y, w, nh);
    ctx.fillStyle = '#fff';
    ctx.font = 'bold 11px system-ui,sans-serif';
    ctx.fillText(pat?.name ?? '?', x + 6, y + nh / 2 + 4);
  }

  if (splitLine.value) {
    ctx.strokeStyle = cssColor('--error', '#f85149');
    ctx.lineWidth = 2;
    ctx.setLineDash([6, 4]);
    ctx.beginPath();
    ctx.moveTo(splitLine.value.x1, splitLine.value.y1);
    ctx.lineTo(splitLine.value.x2, splitLine.value.y2);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  const px = globalBeatToX(
    playback.globalBeat,
    beatsPerMeasure.value,
    beatWidth.value * beatsPerMeasure.value,
    selection.scrollX,
    LABEL_WIDTH,
  );
  ctx.strokeStyle = cssColor('--playhead', '#f85149');
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(px, 0);
  ctx.lineTo(px, h);
  ctx.stroke();
}

function measureViewport() {
  if (!viewportRef.value) return;
  viewportSize.value = {
    width: viewportRef.value.clientWidth,
    height: viewportRef.value.clientHeight,
  };
  draw();
}

function insertAt(x: number) {
  const patternId = patterns.activePatternId;
  const pat = activePattern.value;
  if (!patternId || !pat) return;
  const beat = xToGlobalBeat(
    x,
    LABEL_WIDTH,
    beatsPerMeasure.value,
    beatWidth.value * beatsPerMeasure.value,
    selection.scrollX,
    playlist.totalBeats,
  );
  const snapped = snapGrid.snapBeat(beat, beatsPerMeasure.value, cellBeats.value);
  const duration = pat.bars * pat.beatsPerMeasure;
  playlist.addClip(patternId, snapped, duration);
  draw();
}

function onMouseDown(e: MouseEvent) {
  const rect = viewportRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;

  if (pianoTool.mode === 'split') {
    splitDragging.value = true;
    splitLine.value = { x1: x, y1: y, x2: x, y2: y };
    return;
  }

  if (pianoTool.mode === 'brush') {
    insertAt(x);
    return;
  }

  const clip = hitClip(x, y);
  if (!clip) {
    selectedClipId.value = null;
    draw();
    return;
  }

  if (pianoTool.mode === 'eraser') {
    playlist.removeClip(clip.id);
    selectedClipId.value = null;
    draw();
    return;
  }

  selectedClipId.value = clip.id;
  if (pianoTool.mode === 'pointer') {
    if (isResizeEdge(clip, x)) {
      resizeClipId.value = clip.id;
    } else {
      dragClipId.value = clip.id;
    }
  }
  draw();
}

function onMouseMove(e: MouseEvent) {
  const rect = viewportRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;

  if (splitDragging.value && splitLine.value) {
    splitLine.value = { ...splitLine.value, x2: x, y2: e.clientY - rect.top };
    draw();
    return;
  }

  if (dragClipId.value) {
    const beat = xToGlobalBeat(
      x,
      LABEL_WIDTH,
      beatsPerMeasure.value,
      beatWidth.value * beatsPerMeasure.value,
      selection.scrollX,
      playlist.totalBeats,
    );
    const snapped = snapGrid.snapBeat(beat, beatsPerMeasure.value, cellBeats.value);
    playlist.moveClip(dragClipId.value, snapped);
    draw();
    return;
  }

  if (resizeClipId.value) {
    const clip = playlist.clips.find((c) => c.id === resizeClipId.value);
    if (clip) {
      const beat = xToGlobalBeat(
        x,
        LABEL_WIDTH,
        beatsPerMeasure.value,
        beatWidth.value * beatsPerMeasure.value,
        selection.scrollX,
        playlist.totalBeats,
      );
      const duration = Math.max(
        cellBeats.value,
        snapGrid.snapBeat(beat, beatsPerMeasure.value, cellBeats.value) - clip.startBeat,
      );
      playlist.resizeClip(resizeClipId.value, duration, snapGrid.preset, cellBeats.value);
      draw();
    }
  }
}

function onMouseUp() {
  if (splitDragging.value && splitLine.value && selectedClipId.value) {
    const clip = playlist.clips.find((c) => c.id === selectedClipId.value);
    if (clip) {
      const rect = viewportRef.value?.getBoundingClientRect();
      if (rect) {
        const midX = (splitLine.value.x1 + splitLine.value.x2) / 2;
        const beat = xToGlobalBeat(
          midX,
          LABEL_WIDTH,
          beatsPerMeasure.value,
          beatWidth.value * beatsPerMeasure.value,
          selection.scrollX,
          playlist.totalBeats,
        );
        playlist.splitClipAt(clip.id, beat, snapGrid.preset, cellBeats.value);
      }
    }
  }
  splitDragging.value = false;
  splitLine.value = null;
  dragClipId.value = null;
  resizeClipId.value = null;
  draw();
}

function onWheel(e: WheelEvent) {
  if (e.ctrlKey) {
    e.preventDefault();
    selection.setZoom(selection.zoomX + (e.deltaY > 0 ? -4 : 4));
  }
}

let ro: ResizeObserver | null = null;

watch(
  () => [
    playlist.clips,
    patterns.patterns,
    selection.zoomX,
    selection.scrollX,
    playback.globalBeat,
    pianoTool.mode,
    snapGrid.preset,
    selectedClipId.value,
  ],
  () => draw(),
  { deep: true },
);

onMounted(() => {
  measureViewport();
  window.addEventListener('mouseup', onMouseUp);
  if (rootRef.value) {
    ro = new ResizeObserver(() => measureViewport());
    ro.observe(rootRef.value);
  }
});

onUnmounted(() => {
  ro?.disconnect();
  window.removeEventListener('mouseup', onMouseUp);
});
</script>

<template>
  <div ref="rootRef" class="playlist-root">
    <div
      ref="viewportRef"
      class="playlist-viewport"
      :class="[`tool-${pianoTool.mode}`]"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @wheel="onWheel"
    >
      <canvas ref="canvasRef" />
    </div>
  </div>
</template>

<style scoped>
.playlist-root {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: var(--panel-bg);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  overflow: hidden;
}

.pattern-list {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.35rem 0.5rem;
  border-bottom: 1px solid var(--border-muted);
  flex-wrap: wrap;
  flex-shrink: 0;
}

.label {
  font-size: 0.7rem;
  text-transform: uppercase;
  color: var(--text-muted);
}

.pat-chip {
  padding: 0.2rem 0.55rem;
  font-size: 0.75rem;
  border: 1px solid var(--pat-color, var(--accent));
  border-radius: 4px;
  background: color-mix(in srgb, var(--pat-color, var(--accent)) 25%, transparent);
  color: var(--text-primary);
  cursor: pointer;
}

.pat-chip.active {
  font-weight: 700;
  box-shadow: inset 0 0 0 1px var(--pat-color);
}

.pat-action {
  padding: 0.15rem 0.4rem;
  font-size: 0.75rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  cursor: pointer;
}

.playlist-viewport {
  flex: 1;
  min-height: 120px;
  overflow: hidden;
  cursor: default;
}

.playlist-viewport.tool-brush {
  cursor: cell;
}

.playlist-viewport.tool-box,
.playlist-viewport.tool-split {
  cursor: crosshair;
}

.playlist-viewport.tool-eraser {
  cursor: not-allowed;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
