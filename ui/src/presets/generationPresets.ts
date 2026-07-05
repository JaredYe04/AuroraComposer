import type { UiParameterSnapshot } from '@/types/aurora';

export interface GenerationPreset {
  id: string;
  label: string;
  patch: Partial<UiParameterSnapshot>;
}

export const GENERATION_PRESETS: GenerationPreset[] = [
  { id: 'classic-sonatina', label: 'Classical Sonatina (120 BPM)', patch: { style: 'classical', mode: 'major', key: 0, tempo_bpm: 120, progression_mode: 'flow', harmony_complexity: 0.42, tonal_conservatism: 0.86, accompaniment_instrument: 'piano', bars: 16 } },
  { id: 'classic-adagio', label: 'Classical Adagio (68 BPM)', patch: { style: 'classical', mode: 'minor', key: 9, tempo_bpm: 68, progression_mode: 'flow', harmony_complexity: 0.36, tonal_conservatism: 0.9, accompaniment_instrument: 'strings', bars: 16 } },
  { id: 'waltz-lyrical', label: 'Lyrical Waltz (96 BPM)', patch: { style: 'classical', mode: 'major', key: 5, tempo_bpm: 96, progression_mode: 'loop', harmony_complexity: 0.4, tonal_conservatism: 0.88, accompaniment_instrument: 'piano', bars: 12 } },
  { id: 'pop-axis', label: 'Pop Axis 1-5-6-4 (116 BPM)', patch: { style: 'pop', mode: 'major', key: 0, tempo_bpm: 116, progression_mode: 'loop', harmony_complexity: 0.45, tonal_conservatism: 0.8, accompaniment_instrument: 'piano', bars: 8 } },
  { id: 'pop-ballad', label: 'Pop Ballad (84 BPM)', patch: { style: 'pop', mode: 'major', key: 7, tempo_bpm: 84, progression_mode: 'flow', harmony_complexity: 0.4, tonal_conservatism: 0.84, accompaniment_instrument: 'strings', bars: 16 } },
  { id: 'edm-house', label: 'EDM House (126 BPM)', patch: { style: 'pop', mode: 'major', key: 2, tempo_bpm: 126, progression_mode: 'loop', harmony_complexity: 0.52, tonal_conservatism: 0.72, accompaniment_instrument: 'piano', bars: 8 } },
  { id: 'edm-future-bass', label: 'Future Bass (150 BPM)', patch: { style: 'ambient', mode: 'major', key: 9, tempo_bpm: 150, progression_mode: 'flow', harmony_complexity: 0.58, tonal_conservatism: 0.7, accompaniment_instrument: 'strings', bars: 8 } },
  { id: 'lofi-chill', label: 'Lo-fi Chill (78 BPM)', patch: { style: 'jazz', mode: 'minor', key: 2, tempo_bpm: 78, progression_mode: 'loop', harmony_complexity: 0.5, tonal_conservatism: 0.78, accompaniment_instrument: 'piano', bars: 8 } },
  { id: 'jazz-ii-v-i', label: 'Jazz ii-V-I (130 BPM)', patch: { style: 'jazz', mode: 'major', key: 10, tempo_bpm: 130, progression_mode: 'flow', harmony_complexity: 0.7, tonal_conservatism: 0.65, accompaniment_instrument: 'piano', bars: 12 } },
  { id: 'jazz-swing-medium', label: 'Jazz Swing (145 BPM)', patch: { style: 'jazz', mode: 'major', key: 5, tempo_bpm: 145, progression_mode: 'flow', harmony_complexity: 0.74, tonal_conservatism: 0.62, accompaniment_instrument: 'piano', bars: 16 } },
  { id: 'neo-soul', label: 'Neo Soul (92 BPM)', patch: { style: 'jazz', mode: 'major', key: 4, tempo_bpm: 92, progression_mode: 'loop', harmony_complexity: 0.66, tonal_conservatism: 0.7, accompaniment_instrument: 'strings', bars: 8 } },
  { id: 'ambient-pad', label: 'Ambient Pad (72 BPM)', patch: { style: 'ambient', mode: 'lydian', key: 0, tempo_bpm: 72, progression_mode: 'flow', harmony_complexity: 0.48, tonal_conservatism: 0.82, accompaniment_instrument: 'strings', bars: 16 } },
  { id: 'ambient-dorian', label: 'Ambient Dorian (82 BPM)', patch: { style: 'ambient', mode: 'dorian', key: 2, tempo_bpm: 82, progression_mode: 'loop', harmony_complexity: 0.5, tonal_conservatism: 0.8, accompaniment_instrument: 'strings', bars: 12 } },
  { id: 'cinematic-minor', label: 'Cinematic Minor (100 BPM)', patch: { style: 'classical', mode: 'minor', key: 0, tempo_bpm: 100, progression_mode: 'flow', harmony_complexity: 0.62, tonal_conservatism: 0.76, accompaniment_instrument: 'strings', bars: 16 } },
  { id: 'cinematic-heroic', label: 'Cinematic Heroic (108 BPM)', patch: { style: 'classical', mode: 'major', key: 7, tempo_bpm: 108, progression_mode: 'flow', harmony_complexity: 0.58, tonal_conservatism: 0.8, accompaniment_instrument: 'strings', bars: 16 } },
  { id: 'folk-major', label: 'Folk Major (104 BPM)', patch: { style: 'pop', mode: 'major', key: 2, tempo_bpm: 104, progression_mode: 'loop', harmony_complexity: 0.34, tonal_conservatism: 0.9, accompaniment_instrument: 'piano', bars: 8 } },
  { id: 'folk-dorian', label: 'Folk Dorian (100 BPM)', patch: { style: 'pop', mode: 'dorian', key: 9, tempo_bpm: 100, progression_mode: 'loop', harmony_complexity: 0.42, tonal_conservatism: 0.86, accompaniment_instrument: 'piano', bars: 8 } },
  { id: 'phrygian-dark', label: 'Dark Phrygian (98 BPM)', patch: { style: 'ambient', mode: 'phrygian', key: 4, tempo_bpm: 98, progression_mode: 'flow', harmony_complexity: 0.52, tonal_conservatism: 0.82, accompaniment_instrument: 'strings', bars: 12 } },
  { id: 'mixolydian-rock', label: 'Mixolydian Rock (124 BPM)', patch: { style: 'pop', mode: 'mixolydian', key: 7, tempo_bpm: 124, progression_mode: 'loop', harmony_complexity: 0.48, tonal_conservatism: 0.78, accompaniment_instrument: 'piano', bars: 8 } },
  { id: 'slow-minimal', label: 'Slow Minimal (60 BPM)', patch: { style: 'ambient', mode: 'major', key: 0, tempo_bpm: 60, progression_mode: 'flow', harmony_complexity: 0.3, tonal_conservatism: 0.92, accompaniment_instrument: 'strings', bars: 16 } },
];
