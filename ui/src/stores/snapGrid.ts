import { defineStore } from 'pinia';
import { ref } from 'vue';

/** FL-style snap presets shared by piano roll and pattern playlist. */
export type SnapPreset =
  | 'cell'
  | 'step_1_6'
  | 'step_1_4'
  | 'step_1_3'
  | 'step_1_2'
  | 'beat_1_6'
  | 'beat_1_4'
  | 'beat_1_3'
  | 'beat_1_2';

export const SNAP_PRESETS: SnapPreset[] = [
  'cell',
  'step_1_6',
  'step_1_4',
  'step_1_3',
  'step_1_2',
  'beat_1_6',
  'beat_1_4',
  'beat_1_3',
  'beat_1_2',
];

export function snapPresetLabel(preset: SnapPreset, locale: 'en' | 'zh'): string {
  const labels: Record<SnapPreset, { en: string; zh: string }> = {
    cell: { en: 'Cell', zh: '单元格' },
    step_1_6: { en: '1/6 step', zh: '1/6 步' },
    step_1_4: { en: '1/4 step', zh: '1/4 步' },
    step_1_3: { en: '1/3 step', zh: '1/3 步' },
    step_1_2: { en: '1/2 step', zh: '1/2 步' },
    beat_1_6: { en: '1/6 beat', zh: '1/6 拍' },
    beat_1_4: { en: '1/4 beat', zh: '1/4 拍' },
    beat_1_3: { en: '1/3 beat', zh: '1/3 拍' },
    beat_1_2: { en: '1/2 beat', zh: '1/2 拍' },
  };
  return labels[preset][locale];
}

function gridSize(preset: SnapPreset, beatsPerMeasure: number, cellBeats: number): number {
  const cell = Math.max(0.001, cellBeats);
  switch (preset) {
    case 'cell':
      return cell;
    case 'step_1_6':
      return cell / 6;
    case 'step_1_4':
      return cell / 4;
    case 'step_1_3':
      return cell / 3;
    case 'step_1_2':
      return cell / 2;
    case 'beat_1_6':
      return 1 / 6;
    case 'beat_1_4':
      return 0.25;
    case 'beat_1_3':
      return 1 / 3;
    case 'beat_1_2':
      return 0.5;
    default:
      return beatsPerMeasure;
  }
}

export function snapValue(
  value: number,
  preset: SnapPreset,
  beatsPerMeasure: number,
  cellBeats = beatsPerMeasure,
): number {
  const size = gridSize(preset, beatsPerMeasure, cellBeats);
  return Math.round(value / size) * size;
}

export const useSnapGridStore = defineStore('snapGrid', () => {
  const preset = ref<SnapPreset>('beat_1_4');

  function setPreset(next: SnapPreset) {
    preset.value = next;
  }

  function snapBeat(
    beat: number,
    beatsPerMeasure: number,
    cellBeats = beatsPerMeasure,
  ): number {
    return snapValue(beat, preset.value, beatsPerMeasure, cellBeats);
  }

  return { preset, setPreset, snapBeat, snapValue };
});
