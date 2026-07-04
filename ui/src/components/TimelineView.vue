<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { TimelineModel } from '@/types/aurora';
import { SECTION_COLORS, sectionRoleLabel } from '@/types/aurora';
import { measureToX, xToMeasure } from '@/utils/pianoRoll';
import { useSelectionStore } from '@/stores/selection';

const props = defineProps<{
  model: TimelineModel | null;
  playheadMeasure?: number | null;
}>();

const selection = useSelectionStore();
const containerRef = ref<HTMLDivElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const width = ref(800);
const height = 72;

const totalMeasures = computed(
  () => props.model?.total_measures ?? props.model?.sections.at(-1)?.end_measure ?? 8,
);

function sectionColor(role: unknown): string {
  const key = sectionRoleLabel(role as Parameters<typeof sectionRoleLabel>[0]);
  return SECTION_COLORS[key] ?? SECTION_COLORS.Default;
}

function sectionLabel(section: TimelineModel['sections'][0]): string {
  if (section.label) return section.label;
  return sectionRoleLabel(section.role);
}

function drawRuler() {
  const canvas = canvasRef.value;
  const container = containerRef.value;
  if (!canvas || !container) return;

  const dpr = window.devicePixelRatio || 1;
  const w = container.clientWidth;
  width.value = w;
  canvas.width = w * dpr;
  canvas.height = height * dpr;
  canvas.style.width = `${w}px`;
  canvas.style.height = `${height}px`;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.scale(dpr, dpr);
  ctx.fillStyle = '#0d1117';
  ctx.fillRect(0, 0, w, height);

  const zoom = selection.zoomX;
  const scroll = selection.scrollX;
  const startMeasure = Math.max(1, xToMeasure(0, zoom, scroll) - 1);
  const endMeasure = xToMeasure(w, zoom, scroll) + 1;

  ctx.strokeStyle = '#30363d';
  ctx.fillStyle = '#8b949e';
  ctx.font = '11px Segoe UI, system-ui, sans-serif';
  ctx.textAlign = 'center';

  for (let m = startMeasure; m <= endMeasure; m++) {
    const x = measureToX(m, zoom, scroll);
    if (x < -zoom || x > w + zoom) continue;

    ctx.beginPath();
    ctx.moveTo(x, 28);
    ctx.lineTo(x, height);
    ctx.stroke();

    ctx.fillStyle = m === selection.selectedMeasure ? '#58a6ff' : '#8b949e';
    ctx.fillText(String(m), x + zoom / 2, 20);
    ctx.fillStyle = '#8b949e';
  }

  if (props.playheadMeasure != null) {
    const px = measureToX(props.playheadMeasure, zoom, scroll);
    ctx.strokeStyle = '#f85149';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(px, 0);
    ctx.lineTo(px, height);
    ctx.stroke();
    ctx.lineWidth = 1;
  }
}

function onClick(e: MouseEvent) {
  if (!props.model) return;
  const rect = containerRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  const measure = xToMeasure(x, selection.zoomX, selection.scrollX);
  if (measure >= 1 && measure <= totalMeasures.value) {
    selection.selectMeasure(measure);
  }
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
  drawRuler();
  if (containerRef.value) {
    ro = new ResizeObserver(() => drawRuler());
    ro.observe(containerRef.value);
  }
});

onUnmounted(() => ro?.disconnect());

watch(
  () => [props.model, selection.zoomX, selection.scrollX, selection.selectedMeasure, props.playheadMeasure],
  () => drawRuler(),
  { deep: true },
);
</script>

<template>
  <div ref="containerRef" class="timeline" @click="onClick" @wheel="onWheel">
    <canvas ref="canvasRef" class="ruler-canvas" />
    <svg
      v-if="model"
      class="sections-svg"
      :width="width"
      :height="36"
      :viewBox="`0 0 ${width} 36`"
    >
      <g v-for="section in model.sections" :key="`${section.id.index}:${section.id.generation}`">
        <rect
          :x="measureToX(section.start_measure, selection.zoomX, selection.scrollX)"
          y="4"
          :width="
            (section.end_measure - section.start_measure + 1) * selection.zoomX
          "
          height="28"
          :fill="sectionColor(section.role)"
          opacity="0.85"
          rx="4"
          class="section-block"
          @click.stop="selection.selectSection(`${section.id.index}:${section.id.generation}`)"
        />
        <text
          :x="
            measureToX(section.start_measure, selection.zoomX, selection.scrollX) +
            ((section.end_measure - section.start_measure + 1) * selection.zoomX) / 2
          "
          y="22"
          text-anchor="middle"
          fill="#fff"
          font-size="11"
          font-family="Segoe UI, system-ui, sans-serif"
        >
          {{ sectionLabel(section) }}
        </text>
      </g>
    </svg>
    <div v-else class="empty">Generate a composition to view the timeline.</div>
  </div>
</template>

<style scoped>
.timeline {
  position: relative;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
  overflow: hidden;
  cursor: crosshair;
  user-select: none;
}

.ruler-canvas {
  display: block;
  width: 100%;
}

.sections-svg {
  display: block;
  margin-top: -4px;
}

.section-block {
  cursor: pointer;
}

.section-block:hover {
  opacity: 1;
}

.empty {
  padding: 1.5rem;
  color: #8b949e;
  font-size: 0.875rem;
  text-align: center;
}
</style>
