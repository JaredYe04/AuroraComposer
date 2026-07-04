<script setup lang="ts">
import { onUnmounted, ref } from 'vue';
import * as Tone from 'tone';
import { exportMidi } from '@/services/tauri';
import { playMidiBytes, stopPlayback, playbackState } from '@/services/player';
import { useCompositionStore } from '@/stores/composition';

const compStore = useCompositionStore();
const isPlaying = ref(false);
const volume = ref(0.7);
const error = ref<string | null>(null);

let pollId: number | null = null;

function clearPoll() {
  if (pollId != null) {
    window.clearInterval(pollId);
    pollId = null;
  }
}

async function play() {
  error.value = null;
  if (isPlaying.value) {
    await stop();
    return;
  }

  try {
    Tone.getDestination().volume.value = Tone.gainToDb(volume.value);
    const bytes = await exportMidi();
    await playMidiBytes(bytes);
    isPlaying.value = true;
    clearPoll();
    pollId = window.setInterval(() => {
      if (playbackState() === 'stopped') {
        isPlaying.value = false;
        clearPoll();
      }
    }, 250);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    isPlaying.value = false;
  }
}

async function stop() {
  await stopPlayback();
  clearPoll();
  isPlaying.value = false;
}

function onVolumeInput(e: Event) {
  const val = Number((e.target as HTMLInputElement).value);
  volume.value = val;
  Tone.getDestination().volume.value = Tone.gainToDb(val);
}

onUnmounted(() => {
  void stop();
});
</script>

<template>
  <div class="player-panel">
    <h3>Playback</h3>
    <div class="controls">
      <button class="transport" :disabled="!compStore.summary" @click="play">
        {{ isPlaying ? 'Stop' : 'Play' }}
      </button>
      <label class="volume">
        Vol
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
    <p v-if="!compStore.summary" class="hint">Generate a composition to enable playback.</p>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.player-panel {
  padding: 0.75rem;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
}

.player-panel h3 {
  margin: 0 0 0.75rem;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #8b949e;
}

.controls {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex-wrap: wrap;
}

.transport {
  padding: 0.4rem 1.25rem;
  background: #238636;
  border: 1px solid #238636;
  border-radius: 6px;
  color: #fff;
  font-weight: 600;
  cursor: pointer;
}

.transport:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.transport:not(:disabled):hover {
  background: #2ea043;
}

.volume {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8125rem;
  color: #8b949e;
  flex: 1;
  min-width: 140px;
}

.volume input {
  flex: 1;
}

.hint {
  margin: 0.5rem 0 0;
  font-size: 0.75rem;
  color: #6e7681;
}

.error {
  margin: 0.5rem 0 0;
  font-size: 0.75rem;
  color: #f85149;
}
</style>
