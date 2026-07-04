<script setup lang="ts">
const emit = defineEmits<{ resize: [delta: number] }>();

function onMouseDown(e: MouseEvent) {
  e.preventDefault();
  let lastX = e.clientX;

  function onMove(ev: MouseEvent) {
    const delta = ev.clientX - lastX;
    lastX = ev.clientX;
    emit('resize', delta);
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
  <div class="resize-handle" @mousedown="onMouseDown" />
</template>

<style scoped>
.resize-handle {
  flex-shrink: 0;
  width: 5px;
  margin: 0 -2px;
  cursor: col-resize;
  background: transparent;
  position: relative;
  z-index: 2;
}

.resize-handle::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: 2px;
  width: 1px;
  background: var(--border-muted);
  transition: background 0.15s, width 0.15s;
}

.resize-handle:hover::after {
  width: 3px;
  left: 1px;
  background: var(--accent);
}
</style>
