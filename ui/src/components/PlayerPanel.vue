<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue';
import * as Tone from 'tone';
import { useI18n } from '@/composables/useI18n';
import { useCompositionStore } from '@/stores/composition';
import { usePlaybackStore } from '@/stores/playback';

const { t } = useI18n();
const compStore = useCompositionStore();
const playback = usePlaybackStore();
const volume = ref(0.7);
const error = ref<string | null>(null);

async function play() {
  error.value = null;
  if (playback.isPlaying) {
    await stop();
    return;
  }
  try {
    Tone.getDestination().volume.value = Tone.gainToDb(volume.value);
    await compStore.play();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function stop() {
  await compStore.stop();
}

function onVolumeInput(e: Event) {
  const val = Number((e.target as HTMLInputElement).value);
  volume.value = val;
  Tone.getDestination().volume.value = Tone.gainToDb(val);
}

watch(
  () => playback.isPlaying,
  (playing) => {
    compStore.playing = playing;
  },
);

onUnmounted(() => {
  void stop();
});
</script>

<template>
  <div class="player-panel">
    <h3>{{ t('playback.title') }}</h3>
    <div class="controls">
      <button class="transport" :disabled="!compStore.summary" @click="play">
        {{ playback.isPlaying ? t('playback.stop') : t('playback.play') }}
      </button>
      <label class="volume">
        {{ t('playback.volume') }}
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          :value="volume"
          @input="onVolumeInput"
        />
        {{ Math.round(volume * 100) }}%
      </label>
    </div>
    <p v-if="!compStore.summary" class="hint">{{ t('playback.hint') }}</p>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.player-panel {
  padding: 0.5rem 0.75rem;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
}

.player-panel h3 {
  margin: 0 0 0.5rem;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.controls {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.transport {
  padding: 0.35rem 1rem;
  background: var(--success);
  border: 1px solid var(--success);
  border-radius: 6px;
  color: #fff;
  font-weight: 600;
  cursor: pointer;
}

.transport:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.volume {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8125rem;
  color: var(--text-muted);
  flex: 1;
  min-width: 120px;
}

.volume input {
  flex: 1;
}

.hint {
  margin: 0.35rem 0 0;
  font-size: 0.75rem;
  color: var(--text-faint);
}

.error {
  margin: 0.35rem 0 0;
  font-size: 0.75rem;
  color: var(--error);
}
</style>
