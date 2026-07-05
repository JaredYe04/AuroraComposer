import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getParameters, setParameters as setParamsCmd } from '@/services/tauri';
import { GENERATION_PRESETS } from '@/presets/generationPresets';
import type { UiParameterSnapshot } from '@/types/aurora';
import { normalizeSeed, rollSeed, sessionSeed } from '@/utils/seed';

const defaultParams = (): UiParameterSnapshot => ({
  key: 0,
  mode: 'major',
  style: 'classical',
  beam_width: 8,
  bars: 8,
  tempo_bpm: 120,
  emotion_valence: 0.5,
  emotion_arousal: 0.5,
  harmony_complexity: 0.5,
  counterpoint_strictness: 0.5,
  drum_density: 0.5,
  drum_accent_emphasis: 0.75,
  drum_hihat_density: 0.6,
  progression_mode: 'loop',
  tonal_conservatism: 0.75,
  accompaniment_instrument: 'auto',
  seed: sessionSeed(),
});

export const useParameterStore = defineStore('parameters', () => {
  const snapshot = ref<UiParameterSnapshot>(defaultParams());
  const selectedPresetId = ref<string>('');
  const loading = ref(false);

  function mergeDefaults(partial: Partial<UiParameterSnapshot>): UiParameterSnapshot {
    return { ...defaultParams(), ...partial };
  }

  async function applySeed(seed: number) {
    const nextSeed = normalizeSeed(seed);
    const next = { ...snapshot.value, seed: nextSeed };
    try {
      snapshot.value = mergeDefaults(await setParamsCmd(next));
    } catch {
      snapshot.value = mergeDefaults(next);
    }
  }

  async function load() {
    loading.value = true;
    try {
      const fromBackend = await getParameters();
      snapshot.value = mergeDefaults(fromBackend);
      await applySeed(sessionSeed());
    } catch {
      snapshot.value = defaultParams();
    } finally {
      loading.value = false;
    }
  }

  async function setParameters(partial: Partial<UiParameterSnapshot>) {
    const next = mergeDefaults({ ...snapshot.value, ...partial });
    try {
      snapshot.value = mergeDefaults(await setParamsCmd(next));
    } catch {
      snapshot.value = next;
    }
  }

  async function rollRandomSeed() {
    await applySeed(rollSeed());
  }

  async function applyPreset(presetId: string) {
    const preset = GENERATION_PRESETS.find((p) => p.id === presetId);
    if (!preset) return;
    selectedPresetId.value = presetId;
    await setParameters({
      ...preset.patch,
      seed: normalizeSeed(snapshot.value.seed),
    });
  }

  function reset() {
    snapshot.value = defaultParams();
    selectedPresetId.value = '';
  }

  return {
    snapshot,
    selectedPresetId,
    loading,
    load,
    setParameters,
    applySeed,
    rollRandomSeed,
    applyPreset,
    reset,
  };
});
