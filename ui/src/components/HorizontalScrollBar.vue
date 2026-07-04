<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  scrollX: number;
  contentWidth: number;
  viewportWidth: number;
}>();

const emit = defineEmits<{ 'update:scrollX': [value: number] }>();

const maxScroll = computed(() => Math.max(0, props.contentWidth - props.viewportWidth));

const thumbWidthPct = computed(() => {
  if (props.contentWidth <= 0 || props.viewportWidth <= 0) return 100;
  return Math.max(8, (props.viewportWidth / props.contentWidth) * 100);
});

const thumbLeftPct = computed(() => {
  if (maxScroll.value <= 0) return 0;
  return (props.scrollX / maxScroll.value) * (100 - thumbWidthPct.value);
});

function scrollFromClientX(track: HTMLElement, clientX: number) {
  const rect = track.getBoundingClientRect();
  const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
  emit('update:scrollX', ratio * maxScroll.value);
}

function onTrackClick(e: MouseEvent) {
  scrollFromClientX(e.currentTarget as HTMLElement, e.clientX);
}

function onThumbMouseDown(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  const track = (e.currentTarget as HTMLElement).parentElement;
  if (!track) return;
  const startX = e.clientX;
  const startScroll = props.scrollX;

  function onMove(ev: MouseEvent) {
    if (!track) return;
    const rect = track.getBoundingClientRect();
    const thumbTravel = rect.width * (1 - thumbWidthPct.value / 100);
    if (thumbTravel <= 0) return;
    const delta = ev.clientX - startX;
    const next = startScroll + (delta / thumbTravel) * maxScroll.value;
    emit('update:scrollX', Math.max(0, Math.min(maxScroll.value, next)));
  }

  function onUp() {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  }

  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}
</script>

<template>
  <div class="h-scroll" @click="onTrackClick">
    <div
      class="h-scroll-thumb"
      :style="{ width: `${thumbWidthPct}%`, left: `${thumbLeftPct}%` }"
      @mousedown="onThumbMouseDown"
      @click.stop
    />
  </div>
</template>

<style scoped>
.h-scroll {
  position: relative;
  height: 10px;
  flex-shrink: 0;
  margin-top: 2px;
  border-radius: 5px;
  background: var(--scrollbar-track);
  cursor: pointer;
}

.h-scroll-thumb {
  position: absolute;
  top: 1px;
  height: 8px;
  border-radius: 4px;
  background: var(--scrollbar-thumb);
  transition: background 0.15s;
}

.h-scroll-thumb:hover {
  background: var(--accent);
}
</style>
