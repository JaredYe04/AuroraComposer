import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import {
  getTransportBpm,
  getTransportSeconds,
  playbackState,
  seekTransport,
  setOnPlaybackTick,
} from '@/services/player';

export const usePlaybackStore = defineStore('playback', () => {
  const isPlaying = ref(false);
  const globalBeat = ref(0);
  const tempoBpm = ref(120);
  const beatsPerMeasure = ref(4);
  const totalMeasures = ref(8);

  const totalGlobalBeats = computed(
    () => Math.max(1, totalMeasures.value * beatsPerMeasure.value),
  );

  const playheadMeasure = computed(
    () => Math.floor(globalBeat.value / beatsPerMeasure.value) + 1,
  );

  const playheadBeatInMeasure = computed(
    () => globalBeat.value % beatsPerMeasure.value,
  );

  const playheadRatio = computed(() =>
    Math.min(1, Math.max(0, globalBeat.value / totalGlobalBeats.value)),
  );

  function beatFromSeconds(seconds: number, bpm: number): number {
    return seconds * (bpm / 60);
  }

  function secondsFromBeat(beat: number, bpm: number): number {
    return beat * (60 / bpm);
  }

  function setTimelineContext(measures: number, bpm: number, bpmPerMeasure = 4) {
    totalMeasures.value = Math.max(1, measures);
    tempoBpm.value = bpm;
    beatsPerMeasure.value = bpmPerMeasure;
    if (globalBeat.value > totalGlobalBeats.value) {
      globalBeat.value = 0;
    }
  }

  function seekToGlobalBeat(beat: number) {
    const clamped = Math.max(0, Math.min(totalGlobalBeats.value, beat));
    globalBeat.value = clamped;
    if (isPlaying.value) {
      seekTransport(secondsFromBeat(clamped, tempoBpm.value));
    }
  }

  function seekToRatio(ratio: number) {
    seekToGlobalBeat(ratio * totalGlobalBeats.value);
  }

  function resetPlayhead() {
    globalBeat.value = 0;
  }

  function onPlayStart(bpm: number, durationSeconds: number) {
    tempoBpm.value = bpm;
    isPlaying.value = true;
    setOnPlaybackTick((seconds, state) => {
      if (state === 'playing') {
        globalBeat.value = beatFromSeconds(seconds, getTransportBpm());
        isPlaying.value = true;
      } else {
        isPlaying.value = false;
        if (state === 'stopped') {
          globalBeat.value = 0;
        }
      }
    });
    void durationSeconds;
  }

  function onPlayStop() {
    isPlaying.value = false;
    globalBeat.value = 0;
    setOnPlaybackTick(null);
  }

  function syncFromTransport() {
    if (playbackState() === 'playing') {
      globalBeat.value = beatFromSeconds(getTransportSeconds(), getTransportBpm());
    }
  }

  return {
    isPlaying,
    globalBeat,
    tempoBpm,
    beatsPerMeasure,
    totalMeasures,
    totalGlobalBeats,
    playheadMeasure,
    playheadBeatInMeasure,
    playheadRatio,
    setTimelineContext,
    seekToGlobalBeat,
    seekToRatio,
    resetPlayhead,
    onPlayStart,
    onPlayStop,
    syncFromTransport,
    beatFromSeconds,
    secondsFromBeat,
  };
});
