<script setup lang="ts">
import AuroraIcon from '@/components/AuroraIcon.vue';
import type { IconName } from '@/assets/icons';

withDefaults(
  defineProps<{
    icon: IconName;
    title?: string;
    active?: boolean;
    disabled?: boolean;
    size?: number;
    variant?: 'default' | 'transport';
  }>(),
  { size: 20, variant: 'default' },
);

defineEmits<{ click: [e: MouseEvent] }>();
</script>

<template>
  <button
    type="button"
    class="icon-btn"
    :class="[{ active }, variant !== 'default' ? variant : null]"
    :disabled="disabled"
    :title="title"
    :aria-label="title"
    @click="$emit('click', $event)"
  >
    <AuroraIcon :name="icon" :size="size ?? 20" />
  </button>
</template>

<style scoped>
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  padding: 0;
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  background: var(--bg-panel-elevated);
  color: var(--text-muted);
  cursor: pointer;
  transition:
    background 0.12s,
    color 0.12s,
    border-color 0.12s;
}

.icon-btn:hover:not(:disabled) {
  background: var(--border-subtle);
  color: var(--text-primary);
}

.icon-btn.active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
}

.icon-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.icon-btn.transport {
  background: var(--success);
  border-color: var(--success);
  color: #fff;
  width: 2.25rem;
  height: 2.25rem;
}

.icon-btn.transport:hover:not(:disabled) {
  filter: brightness(1.08);
  color: #fff;
}
</style>
