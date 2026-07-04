import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { snapValue, type SnapPreset } from './snapGrid';

let clipCounter = 0;

export interface PlaylistClip {
  id: string;
  patternId: string;
  startBeat: number;
  durationBeats: number;
}

export const usePlaylistStore = defineStore('playlist', () => {
  const clips = ref<PlaylistClip[]>([]);

  const totalBeats = computed(() => {
    if (clips.value.length === 0) return 32;
    return Math.max(
      32,
      ...clips.value.map((c) => c.startBeat + c.durationBeats),
    );
  });

  function addClip(patternId: string, startBeat: number, durationBeats: number) {
    clipCounter += 1;
    clips.value = [
      ...clips.value,
      {
        id: `clip-${Date.now()}-${clipCounter}`,
        patternId,
        startBeat,
        durationBeats,
      },
    ];
  }

  function removeClip(id: string) {
    clips.value = clips.value.filter((c) => c.id !== id);
  }

  function moveClip(id: string, startBeat: number) {
    const clip = clips.value.find((c) => c.id === id);
    if (clip) clip.startBeat = Math.max(0, startBeat);
  }

  function resizeClip(id: string, durationBeats: number, preset: SnapPreset, cellBeats: number) {
    const clip = clips.value.find((c) => c.id === id);
    if (!clip) return;
    const snapped = snapValue(durationBeats, preset, cellBeats, cellBeats);
    clip.durationBeats = Math.max(cellBeats, snapped);
  }

  function splitClipAt(id: string, splitBeat: number, preset: SnapPreset, cellBeats: number) {
    const clip = clips.value.find((c) => c.id === id);
    if (!clip) return;
    const at = snapValue(splitBeat, preset, cellBeats, cellBeats);
    if (at <= clip.startBeat || at >= clip.startBeat + clip.durationBeats) return;

    const leftDuration = at - clip.startBeat;
    const rightDuration = clip.durationBeats - leftDuration;
    clip.durationBeats = leftDuration;

    clipCounter += 1;
    clips.value = [
      ...clips.value,
      {
        id: `clip-${Date.now()}-${clipCounter}`,
        patternId: clip.patternId,
        startBeat: at,
        durationBeats: rightDuration,
      },
    ];
  }

  function duplicateClip(id: string, preset: SnapPreset, cellBeats: number) {
    const clip = clips.value.find((c) => c.id === id);
    if (!clip) return;
    const start = snapValue(clip.startBeat + clip.durationBeats, preset, cellBeats, cellBeats);
    addClip(clip.patternId, start, clip.durationBeats);
  }

  function seedFromPattern(patternId: string, durationBeats: number) {
    clips.value = [];
    addClip(patternId, 0, durationBeats);
  }

  function removeClipsForPattern(patternId: string) {
    clips.value = clips.value.filter((c) => c.patternId !== patternId);
  }

  function clearAll() {
    clips.value = [];
    clipCounter = 0;
  }

  return {
    clips,
    totalBeats,
    addClip,
    removeClip,
    moveClip,
    resizeClip,
    splitClipAt,
    duplicateClip,
    seedFromPattern,
    removeClipsForPattern,
    clearAll,
  };
});
