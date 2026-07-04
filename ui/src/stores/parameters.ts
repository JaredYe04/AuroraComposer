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

  function mergeDefaults(partial: Partial<UiParameterSnapshot>): UiParameterSnapshot {
    return { ...defaultParams(), ...partial };
  }

  async function load() {
    loading.value = true;
    try {
      const fromBackend = await getParameters();
      snapshot.value = mergeDefaults(fromBackend);
    } finally {
      loading.value = false;
    }
  }

  async function setParameters(partial: Partial<UiParameterSnapshot>) {
    const next = mergeDefaults({ ...snapshot.value, ...partial });
    snapshot.value = mergeDefaults(await setParamsCmd(next));
  }

  function reset() {
    snapshot.value = defaultParams();
  }

  return { snapshot, loading, load, setParameters, reset };
});
