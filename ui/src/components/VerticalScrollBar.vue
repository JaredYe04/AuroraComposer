<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  scrollY: number;
  contentHeight: number;
  viewportHeight: number;
}>();

const emit = defineEmits<{ 'update:scrollY': [value: number] }>();

const maxScroll = computed(() => Math.max(0, props.contentHeight - props.viewportHeight));

const thumbHeightPct = computed(() => {
  if (props.contentHeight <= 0 || props.viewportHeight <= 0) return 100;
  return Math.max(8, (props.viewportHeight / props.contentHeight) * 100);
});

const thumbTopPct = computed(() => {
  if (maxScroll.value <= 0) return 0;
  return (props.scrollY / maxScroll.value) * (100 - thumbHeightPct.value);
});

function scrollFromClientY(track: HTMLElement, clientY: number) {
  const rect = track.getBoundingClientRect();
  const ratio = Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));
  emit('update:scrollY', ratio * maxScroll.value);
}

function onTrackClick(e: MouseEvent) {
  scrollFromClientY(e.currentTarget as HTMLElement, e.clientY);
}

function onThumbMouseDown(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  const track = (e.currentTarget as HTMLElement).parentElement;
  if (!track) return;
  const startY = e.clientY;
  const startScroll = props.scrollY;

  function onMove(ev: MouseEvent) {
    if (!track) return;
    const rect = track.getBoundingClientRect();
    const thumbTravel = rect.height * (1 - thumbHeightPct.value / 100);
    if (thumbTravel <= 0) return;
    const delta = ev.clientY - startY;
    const next = startScroll + (delta / thumbTravel) * maxScroll.value;
    emit('update:scrollY', Math.max(0, Math.min(maxScroll.value, next)));
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
  <div
    class="v-scroll"
    :class="{ disabled: maxScroll <= 0 }"
    @click="onTrackClick"
  >
    <div
      class="v-scroll-thumb"
      :style="{ height: `${thumbHeightPct}%`, top: `${thumbTopPct}%` }"
      @mousedown="onThumbMouseDown"
      @click.stop
    />
  </div>
</template>

<style scoped>
.v-scroll {
  position: relative;
  width: 8px;
  flex-shrink: 0;
  border-radius: 4px;
  background: var(--scrollbar-track);
  cursor: pointer;
}

.v-scroll.disabled {
  opacity: 0.35;
  pointer-events: none;
}

.v-scroll-thumb {
  position: absolute;
  left: 1px;
  width: 6px;
  border-radius: 3px;
  background: var(--scrollbar-thumb);
  transition: background 0.15s;
}

.v-scroll-thumb:hover {
  background: var(--accent);
}
</style>
