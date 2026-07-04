import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getParameters, setParameters as setParamsCmd } from '@/services/tauri';
import type { UiParameterSnapshot } from '@/types/aurora';

const defaultParams = (): UiParameterSnapshot => ({
  key: 0,
  style: 'classical',
  beam_width: 8,
  bars: 8,
  tempo_bpm: 120,
  emotion_valence: 0.5,
  emotion_arousal: 0.5,
  harmony_complexity: 0.5,
  counterpoint_strictness: 0.5,
  drum_density: 0.5,
});

export const useParameterStore = defineStore('parameters', () => {
  const snapshot = ref<UiParameterSnapshot>(defaultParams());
  const loading = ref(false);

  async function load() {
    loading.value = true;
    try {
      snapshot.value = await getParameters();
    } finally {
      loading.value = false;
    }
  }

  async function setParameters(partial: Partial<UiParameterSnapshot>) {
    const next = { ...snapshot.value, ...partial };
    snapshot.value = await setParamsCmd(next);
  }

  function reset() {
    snapshot.value = defaultParams();
  }

  return { snapshot, loading, load, setParameters, reset };
});
