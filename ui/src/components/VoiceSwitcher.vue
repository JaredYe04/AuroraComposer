<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from '@/composables/useI18n';
import { useSelectionStore } from '@/stores/selection';

const emit = defineEmits<{
  change: [voiceId: number | null];
}>();

const { t } = useI18n();
const selection = useSelectionStore();

const tabs = computed(() => {
  const voices = selection.voices;
  if (voices.length === 0) return [];
  return [{ id: null as number | null, name: t('voices.all') }, ...voices.map((v) => ({ id: v.id, name: v.name }))];
});

function selectVoice(voiceId: number | null) {
  selection.setActiveVoice(voiceId);
  emit('change', voiceId);
}
</script>

<template>
  <div v-if="tabs.length > 0" class="voice-switcher">
    <span class="label">{{ t('voices.title') }}</span>
    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id ?? 'all'"
        class="tab"
        :class="{ active: selection.activeVoiceId === tab.id }"
        @click="selectVoice(tab.id)"
      >
        {{ tab.name }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.voice-switcher {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.65rem;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  flex-shrink: 0;
}

.label {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  flex-shrink: 0;
}

.tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.tab {
  padding: 0.25rem 0.55rem;
  font-size: 0.75rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  color: var(--text-muted);
  cursor: pointer;
}

.tab:hover {
  background: var(--border-subtle);
  color: var(--text-primary);
}

.tab.active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}
</style>
