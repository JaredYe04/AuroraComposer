<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import IconButton from '@/components/IconButton.vue';
import { useI18n } from '@/composables/useI18n';
import { useCompositionStore } from '@/stores/composition';
import { usePlaybackStore } from '@/stores/playback';
import { useSettingsStore, type MelodyEngine } from '@/stores/settings';
import { setMelodyEngine, setMasterVolume } from '@/services/player';

const { t } = useI18n();
const compStore = useCompositionStore();
const playback = usePlaybackStore();
const settings = useSettingsStore();
const volume = ref(0.7);
const error = ref<string | null>(null);

const melodyOptions: { value: MelodyEngine; labelKey: string }[] = [
  { value: 'gm', labelKey: 'playback.melodyGm' },
  { value: 'sine', labelKey: 'playback.melodySine' },
  { value: 'square', labelKey: 'playback.melodySquare' },
  { value: 'triangle', labelKey: 'playback.melodyTriangle' },
  { value: 'sawtooth', labelKey: 'playback.melodySawtooth' },
];

async function togglePlay() {
  error.value = null;
  if (playback.isPlaying) {
    await compStore.stop();
    return;
  }
  try {
    setMasterVolume(volume.value);
    await compStore.play();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

function onVolumeInput(e: Event) {
  const val = Number((e.target as HTMLInputElement).value);
  volume.value = val;
  setMasterVolume(val);
}

function onMelodyEngineChange(e: Event) {
  const next = (e.target as HTMLSelectElement).value as MelodyEngine;
  settings.setMelodyEngine(next);
  setMelodyEngine(next);
}

watch(
  () => playback.isPlaying,
  (playing) => {
    compStore.playing = playing;
  },
);

onMounted(() => {
  setMelodyEngine(settings.melodyEngine);
  setMasterVolume(volume.value);
});

onUnmounted(() => {
  void compStore.stop();
});
</script>

<template>
  <div class="player-panel">
    <div class="controls">
      <div class="transport-wrap">
        <IconButton
          variant="transport"
          :icon="playback.isPlaying ? 'stop' : 'play'"
          :title="playback.isPlaying ? t('playback.stop') : t('playback.play')"
          :disabled="!compStore.summary"
          @click="togglePlay"
        />
      </div>
      <label class="melody-engine">
        <span>{{ t('playback.melodyEngine') }}</span>
        <select :value="settings.melodyEngine" @change="onMelodyEngineChange">
          <option v-for="opt in melodyOptions" :key="opt.value" :value="opt.value">
            {{ t(opt.labelKey as 'playback.melodyGm') }}
          </option>
        </select>
      </label>
      <label class="volume">
        <span>{{ t('playback.volume') }}</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          :value="volume"
          @input="onVolumeInput"
        />
        <span>{{ Math.round(volume * 100) }}%</span>
      </label>
    </div>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.player-panel {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.transport-wrap {
  display: flex;
  justify-content: center;
  min-width: 2.25rem;
}

.melody-engine {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.melody-engine select {
  font-size: 0.75rem;
  padding: 0.15rem 0.3rem;
  border-radius: 4px;
  border: 1px solid var(--border-muted);
  background: var(--bg-panel-elevated);
  color: var(--text-primary);
}

.volume {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  color: var(--text-muted);
  min-width: 100px;
}

.volume input {
  width: 72px;
}

.error {
  margin: 0;
  font-size: 0.75rem;
  color: var(--error);
}
</style>
