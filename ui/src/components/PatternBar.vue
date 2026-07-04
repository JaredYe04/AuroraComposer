<script setup lang="ts">
import { ref } from 'vue';
import IconButton from '@/components/IconButton.vue';
import { useI18n } from '@/composables/useI18n';
import { usePatternsStore } from '@/stores/patterns';
import { usePlaylistStore } from '@/stores/playlist';

const { t } = useI18n();
const patterns = usePatternsStore();
const playlist = usePlaylistStore();

const renamingId = ref<string | null>(null);
const renameValue = ref('');

function addPattern() {
  const pat = patterns.createPattern();
  playlist.addClip(pat.id, playlist.totalBeats, pat.bars * pat.beatsPerMeasure);
}

function duplicateActive() {
  const id = patterns.activePatternId;
  if (!id) return;
  const copy = patterns.duplicatePattern(id);
  if (copy) {
    playlist.addClip(copy.id, playlist.totalBeats, copy.bars * copy.beatsPerMeasure);
  }
}

function deleteActive() {
  const id = patterns.activePatternId;
  if (!id || patterns.patterns.length <= 1) return;
  playlist.removeClipsForPattern(id);
  patterns.deletePattern(id);
}

function startRename(id: string, name: string) {
  renamingId.value = id;
  renameValue.value = name;
}

function commitRename() {
  if (renamingId.value) {
    patterns.renamePattern(renamingId.value, renameValue.value);
  }
  renamingId.value = null;
}
</script>

<template>
  <div class="pattern-bar">
    <span class="label">{{ t('playlist.patterns') }}</span>
    <div class="pattern-tabs">
      <div
        v-for="pat in patterns.patterns"
        :key="pat.id"
        class="pat-item"
        :class="{ active: patterns.activePatternId === pat.id }"
      >
        <button
          class="pat-chip"
          :style="{ '--pat-color': pat.color }"
          @click="patterns.setActivePattern(pat.id)"
          @dblclick="startRename(pat.id, pat.name)"
        >
          <span v-if="renamingId !== pat.id">{{ pat.name }}</span>
          <input
            v-else
            v-model="renameValue"
            class="rename-input"
            @keydown.enter="commitRename"
            @blur="commitRename"
          />
        </button>
      </div>
    </div>
    <div class="pattern-actions">
      <IconButton icon="pattern" :title="t('playlist.addPattern')" @click="addPattern" />
      <IconButton
        icon="duplicate"
        :title="t('playlist.duplicatePattern')"
        :disabled="!patterns.activePatternId"
        @click="duplicateActive"
      />
      <IconButton
        icon="delete"
        :title="t('playlist.deletePattern')"
        :disabled="!patterns.activePatternId || patterns.patterns.length <= 1"
        @click="deleteActive"
      />
      <IconButton
        icon="brush"
        :title="t('playlist.randomColor')"
        :disabled="!patterns.activePatternId"
        @click="patterns.activePatternId && patterns.randomizeColor(patterns.activePatternId)"
      />
    </div>
  </div>
</template>

<style scoped>
.pattern-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.65rem;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  flex-shrink: 0;
  min-height: 2.5rem;
}

.label {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  flex-shrink: 0;
}

.pattern-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  flex: 1;
  min-width: 0;
}

.pat-chip {
  padding: 0.2rem 0.55rem;
  font-size: 0.75rem;
  border: 1px solid var(--pat-color, var(--accent));
  border-radius: 4px;
  background: color-mix(in srgb, var(--pat-color, var(--accent)) 22%, transparent);
  color: var(--text-primary);
  cursor: pointer;
}

.pat-item.active .pat-chip {
  font-weight: 700;
  box-shadow: inset 0 0 0 1px var(--pat-color, var(--accent));
}

.rename-input {
  width: 5.5rem;
  font-size: 0.75rem;
  padding: 0.1rem 0.25rem;
  border: 1px solid var(--border-muted);
  border-radius: 3px;
  background: var(--bg-input);
  color: inherit;
}

.pattern-actions {
  display: flex;
  gap: 0.2rem;
  flex-shrink: 0;
}
</style>
