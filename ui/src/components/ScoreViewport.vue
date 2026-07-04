<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import IconButton from '@/components/IconButton.vue';
import { useI18n } from '@/composables/useI18n';

const props = withDefaults(
  defineProps<{
    pageCount?: number;
    currentPage?: number;
    showPaging?: boolean;
  }>(),
  { pageCount: 1, currentPage: 1, showPaging: false },
);

const emit = defineEmits<{
  'update:currentPage': [page: number];
  fit: [];
}>();

const { t } = useI18n();

const viewportRef = ref<HTMLDivElement | null>(null);
const contentRef = ref<HTMLDivElement | null>(null);

const scale = ref(1);
const panX = ref(0);
const panY = ref(0);
const panning = ref(false);
const panStart = ref({ x: 0, y: 0, panX: 0, panY: 0 });

const transformStyle = computed(() => ({
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${scale.value})`,
  transformOrigin: 'top left',
}));

function fitToContainer() {
  const viewport = viewportRef.value;
  const content = contentRef.value;
  if (!viewport || !content) return;

  const svg = content.querySelector('svg');
  const target = svg ?? content.firstElementChild;
  if (!target) return;

  const vw = viewport.clientWidth - 16;
  const vh = viewport.clientHeight - 16;
  const rect = (target as HTMLElement).getBoundingClientRect();
  const cw = rect.width / scale.value || content.scrollWidth || 1;
  const ch = rect.height / scale.value || content.scrollHeight || 1;

  scale.value = Math.min(vw / cw, vh / ch, 2);
  panX.value = Math.max(0, (vw - cw * scale.value) / 2);
  panY.value = Math.max(0, (vh - ch * scale.value) / 2);
  emit('fit');
}

function onWheel(e: WheelEvent) {
  if (!e.ctrlKey) return;
  e.preventDefault();
  const delta = e.deltaY > 0 ? 0.9 : 1.1;
  scale.value = Math.min(4, Math.max(0.15, scale.value * delta));
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  panning.value = true;
  panStart.value = {
    x: e.clientX,
    y: e.clientY,
    panX: panX.value,
    panY: panY.value,
  };
  e.preventDefault();
}

function onMouseMove(e: MouseEvent) {
  if (!panning.value) return;
  panX.value = panStart.value.panX + (e.clientX - panStart.value.x);
  panY.value = panStart.value.panY + (e.clientY - panStart.value.y);
}

function onMouseUp() {
  panning.value = false;
}

function prevPage() {
  if (props.currentPage > 1) {
    emit('update:currentPage', props.currentPage - 1);
  }
}

function nextPage() {
  if (props.currentPage < props.pageCount) {
    emit('update:currentPage', props.currentPage + 1);
  }
}

function zoomIn() {
  scale.value = Math.min(4, scale.value * 1.15);
}

function zoomOut() {
  scale.value = Math.max(0.15, scale.value / 1.15);
}

defineExpose({ fitToContainer });

let ro: ResizeObserver | null = null;

watch(
  () => props.currentPage,
  () => {
    requestAnimationFrame(() => fitToContainer());
  },
);

onMounted(() => {
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
  if (viewportRef.value) {
    ro = new ResizeObserver(() => fitToContainer());
    ro.observe(viewportRef.value);
  }
});

onUnmounted(() => {
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseup', onMouseUp);
  ro?.disconnect();
});
</script>

<template>
  <div class="score-viewport-root">
    <div class="viewport-toolbar">
      <IconButton
        icon="zoomOut"
        :title="t('score.zoomOut')"
        @click="zoomOut"
      />
      <IconButton icon="zoomIn" :title="t('score.zoomIn')" @click="zoomIn" />
      <IconButton icon="zoomFit" :title="t('score.zoomFit')" @click="fitToContainer" />
      <span class="zoom-label">{{ Math.round(scale * 100) }}%</span>
      <span class="pan-hint">{{ t('score.panHint') }}</span>
      <template v-if="showPaging && pageCount > 1">
        <IconButton
          icon="pagePrev"
          :title="t('score.pagePrev')"
          :disabled="currentPage <= 1"
          @click="prevPage"
        />
        <span class="page-label">{{ currentPage }} / {{ pageCount }}</span>
        <IconButton
          icon="pageNext"
          :title="t('score.pageNext')"
          :disabled="currentPage >= pageCount"
          @click="nextPage"
        />
      </template>
    </div>

    <div
      ref="viewportRef"
      class="viewport"
      :class="{ panning }"
      @wheel="onWheel"
      @mousedown="onMouseDown"
    >
      <div ref="contentRef" class="viewport-content" :style="transformStyle">
        <slot />
      </div>
    </div>
  </div>
</template>

<style scoped>
.score-viewport-root {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  gap: 0.35rem;
}

.viewport-toolbar {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.zoom-label,
.page-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  min-width: 2.5rem;
}

.pan-hint {
  font-size: 0.7rem;
  color: var(--text-faint);
  margin-left: auto;
}

.viewport {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background: var(--score-paper);
  border-radius: 4px;
  cursor: grab;
  position: relative;
}

.viewport.panning {
  cursor: grabbing;
}

.viewport-content {
  display: inline-block;
  min-width: min-content;
  padding: 0.5rem;
}

.viewport-content :deep(svg) {
  display: block;
  max-width: none;
  height: auto;
}
</style>
