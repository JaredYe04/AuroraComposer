<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import PluginPanel from '@/components/PluginPanel.vue';
import { useI18n } from '@/composables/useI18n';

const { t } = useI18n();
const open = ref(false);
const menuRef = ref<HTMLElement | null>(null);

function toggle() {
  open.value = !open.value;
}

function close() {
  open.value = false;
}

function onDocClick(e: MouseEvent) {
  if (!menuRef.value?.contains(e.target as Node)) {
    close();
  }
}

onMounted(() => {
  document.addEventListener('click', onDocClick);
});

onUnmounted(() => {
  document.removeEventListener('click', onDocClick);
});
</script>

<template>
  <div ref="menuRef" class="plugin-menu">
    <button class="menu-trigger" :class="{ open }" @click.stop="toggle">
      {{ t('plugins.menu') }}
      <span class="chevron">{{ open ? '▲' : '▼' }}</span>
    </button>

    <div v-if="open" class="dropdown" @click.stop>
      <PluginPanel embedded />
    </div>
  </div>
</template>

<style scoped>
.plugin-menu {
  position: relative;
}

.menu-trigger {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.65rem;
  font-size: 0.75rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  color: inherit;
  cursor: pointer;
}

.menu-trigger:hover,
.menu-trigger.open {
  background: var(--border-subtle);
  border-color: var(--accent);
}

.chevron {
  font-size: 0.55rem;
  color: var(--text-muted);
}

.dropdown {
  position: absolute;
  top: calc(100% + 0.35rem);
  left: 0;
  z-index: 100;
  width: min(360px, 90vw);
  max-height: 70vh;
  overflow: auto;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 8px;
  box-shadow: var(--shadow);
}
</style>
