<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import VerticalScrollBar from '@/components/VerticalScrollBar.vue';

const viewportRef = ref<HTMLDivElement | null>(null);
const contentRef = ref<HTMLDivElement | null>(null);
const scrollY = ref(0);
const viewportHeight = ref(0);
const contentHeight = ref(0);

const maxScroll = () => Math.max(0, contentHeight.value - viewportHeight.value);

function clampScroll(y: number): number {
  return Math.max(0, Math.min(maxScroll(), y));
}

function applyScroll() {
  const el = contentRef.value;
  if (el) {
    el.style.transform = `translateY(${-scrollY.value}px)`;
  }
}

function setScrollY(y: number) {
  scrollY.value = clampScroll(y);
  applyScroll();
}

function measure() {
  if (!viewportRef.value || !contentRef.value) return;
  viewportHeight.value = viewportRef.value.clientHeight;
  contentHeight.value = contentRef.value.scrollHeight;
  if (scrollY.value > maxScroll()) {
    setScrollY(maxScroll());
  }
}

function onWheel(e: WheelEvent) {
  if (maxScroll() <= 0) return;
  e.preventDefault();
  setScrollY(scrollY.value + e.deltaY);
}

let ro: ResizeObserver | null = null;

watch(scrollY, applyScroll);

onMounted(async () => {
  await nextTick();
  measure();
  if (viewportRef.value) {
    ro = new ResizeObserver(() => measure());
    ro.observe(viewportRef.value);
    if (contentRef.value) {
      ro.observe(contentRef.value);
    }
  }
});

onUnmounted(() => {
  ro?.disconnect();
});

defineExpose({ remeasure: measure });
</script>

<template>
  <div class="scroll-panel">
    <div ref="viewportRef" class="scroll-viewport" @wheel="onWheel">
      <div ref="contentRef" class="scroll-content">
        <slot />
      </div>
    </div>
    <VerticalScrollBar
      :scroll-y="scrollY"
      :content-height="contentHeight"
      :viewport-height="viewportHeight"
      @update:scroll-y="setScrollY"
    />
  </div>
</template>

<style scoped>
.scroll-panel {
  display: flex;
  flex: 1;
  min-height: 0;
  gap: 0.35rem;
  overflow: hidden;
}

.scroll-viewport {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  position: relative;
}

.scroll-content {
  will-change: transform;
}
</style>
