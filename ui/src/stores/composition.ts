import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  exportAbc,
  exportMidi,
  exportMusicXml,
  exportSvgPreview,
  generateComposition,
  getComposition,
  getTimeline,
  onJobComplete,
  onJobProgress,
} from '@/services/tauri';
import { playMidiBytes, stopPlayback, playbackState } from '@/services/player';
import type {
  Composition,
  CompositionSummary,
  JobCompleteEvent,
  JobProgressEvent,
  PianoRollNote,
  TimelineModel,
} from '@/types/aurora';
import { extractPianoRollNotes } from '@/utils/pianoRoll';
import { useParameterStore } from './parameters';

export const useCompositionStore = defineStore('composition', () => {
  const summary = ref<CompositionSummary | null>(null);
  const composition = ref<Composition | null>(null);
  const timeline = ref<TimelineModel | null>(null);
  const pianoRollNotes = ref<PianoRollNote[]>([]);
  const progress = ref<JobProgressEvent | null>(null);
  const generating = ref(false);
  const error = ref<string | null>(null);
  const lastAbc = ref<string | null>(null);
  const lastSvgPreview = ref<string | null>(null);
  const playing = ref(false);

  let eventsSubscribed = false;

  async function subscribeEvents() {
    if (eventsSubscribed) return;
    eventsSubscribed = true;
    await onJobProgress((event) => {
      progress.value = event;
    });
    await onJobComplete(async (event: JobCompleteEvent) => {
      summary.value = event.summary;
      generating.value = false;
      progress.value = null;
      await loadComposition();
    });
  }

  async function loadComposition() {
    try {
      composition.value = await getComposition();
      timeline.value = await getTimeline();
      pianoRollNotes.value = composition.value
        ? extractPianoRollNotes(composition.value)
        : [];
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function generate() {
    const paramStore = useParameterStore();
    generating.value = true;
    error.value = null;
    progress.value = null;
    try {
      await subscribeEvents();
      summary.value = await generateComposition(paramStore.snapshot);
      await loadComposition();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      generating.value = false;
    }
  }

  async function downloadMidi() {
    try {
      const bytes = await exportMidi();
      const blob = new Blob([new Uint8Array(bytes)], { type: 'audio/midi' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${summary.value?.title ?? 'aurora'}.mid`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function downloadMusicXml() {
    try {
      const xml = await exportMusicXml();
      const blob = new Blob([xml], { type: 'application/xml' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${summary.value?.title ?? 'aurora'}.musicxml`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function downloadAbc() {
    try {
      lastAbc.value = await exportAbc();
      const blob = new Blob([lastAbc.value], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${summary.value?.title ?? 'aurora'}.abc`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadSvgPreview() {
    try {
      lastSvgPreview.value = await exportSvgPreview();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function play() {
    try {
      error.value = null;
      const bytes = await exportMidi();
      playing.value = true;
      await playMidiBytes(bytes);
      const poll = window.setInterval(() => {
        if (playbackState() === 'stopped') {
          playing.value = false;
          window.clearInterval(poll);
        }
      }, 250);
    } catch (e) {
      playing.value = false;
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function stop() {
    await stopPlayback();
    playing.value = false;
  }

  return {
    summary,
    composition,
    timeline,
    pianoRollNotes,
    progress,
    generating,
    error,
    lastAbc,
    lastSvgPreview,
    playing,
    generate,
    loadComposition,
    downloadMidi,
    downloadMusicXml,
    downloadAbc,
    loadSvgPreview,
    play,
    stop,
  };
});
