import { defineStore } from 'pinia';
import { ref, toValue } from 'vue';
import {
  applyNotePatch,
  deleteNote as deleteNoteCmd,
  exportAbc,
  exportMidi,
  exportMusicXml,
  exportSvgPreview,
  generateComposition,
  getComposition,
  getTimeline,
  insertNote as insertNoteCmd,
  loadProject,
  onJobComplete,
  onJobProgress,
} from '@/services/tauri';
import { playMidiBytes, stopPlayback } from '@/services/player';
import { promptAndSaveBytes, promptAndSaveText } from '@/services/download';
import type {
  Composition,
  CompositionSummary,
  JobCompleteEvent,
  JobProgressEvent,
  NodeId,
  PianoRollNote,
  TimelineModel,
} from '@/types/aurora';
import { extractPianoRollNotes } from '@/utils/pianoRoll';
import { summaryFromComposition } from '@/utils/compositionSummary';
import { useParameterStore } from './parameters';
import { usePlaybackStore } from './playback';
import { usePatternsStore } from './patterns';
import { usePlaylistStore } from './playlist';
import { useSelectionStore } from './selection';

export interface InsertNoteParams {
  measureGlobal: number;
  voiceId: number;
  beatNumer: number;
  beatDenom: number;
  midi: number;
  isDrum: boolean;
}

export const useCompositionStore = defineStore('composition', () => {
  const summary = ref<CompositionSummary | null>(null);
  const composition = ref<Composition | null>(null);
  const timeline = ref<TimelineModel | null>(null);
  const pianoRollNotes = ref<PianoRollNote[]>([]);
  const progress = ref<JobProgressEvent | null>(null);
  const generating = ref(false);
  const playing = ref(false);
  const error = ref<string | null>(null);
  const exportError = ref<string | null>(null);
  const lastAbc = ref<string | null>(null);
  const lastSvgPreview = ref<string | null>(null);
  const revision = ref(0);

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
      error.value = null;
      composition.value = await getComposition();
      timeline.value = await getTimeline();
      if (composition.value) {
        summary.value = summaryFromComposition(composition.value);
      }
      const selection = useSelectionStore();
      const playback = usePlaybackStore();
      if (
        composition.value &&
        selection.activeVoiceId == null &&
        composition.value.voice_registry.voices.length > 0
      ) {
        selection.setActiveVoice(composition.value.voice_registry.voices[0].id);
      }
      pianoRollNotes.value = composition.value
        ? extractPianoRollNotes(
            composition.value,
            selection.activeVoiceId ?? undefined,
          )
        : [];
      revision.value += 1;
      ensureWorkspacePatterns();
      if (composition.value) {
        const bpm = composition.value.global.tempo_map.default_bpm;
        const bpmPerMeasure = composition.value.global.meter_map.default.beats;
        const bars = summary.value?.bars ?? 8;
        playback.setTimelineContext(bars, bpm, bpmPerMeasure);
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  function ensureWorkspacePatterns() {
    const patterns = usePatternsStore();
    const playlist = usePlaylistStore();
    const beatsPerMeasure = composition.value?.global.meter_map.default.beats ?? 4;
    const bars = summary.value?.bars ?? useParameterStore().snapshot.bars ?? 8;
    const tempo = summary.value?.tempo_bpm ?? 120;
    const pat = patterns.ensureDefault({
      bars,
      beatsPerMeasure,
      tempoBpm: tempo,
    });
    if (playlist.clips.length === 0) {
      playlist.seedFromPattern(pat.id, pat.bars * pat.beatsPerMeasure);
    }
  }

  async function initWorkspace() {
    await loadComposition();
  }

  function syncPatternsFromSummary() {
    if (!summary.value || !composition.value) return;
    const patterns = usePatternsStore();
    const playlist = usePlaylistStore();
    const beatsPerMeasure = composition.value.global.meter_map.default.beats;
    if (patterns.patterns.length <= 1) {
      const active = patterns.activePatternId
        ? patterns.patterns.find((p) => p.id === patterns.activePatternId)
        : null;
      if (active) {
        active.bars = summary.value.bars;
        active.beatsPerMeasure = beatsPerMeasure;
        active.tempoBpm = summary.value.tempo_bpm;
      }
    } else {
      patterns.registerFromComposition(summary.value, beatsPerMeasure);
    }
    if (playlist.clips.length === 0 && patterns.activePatternId) {
      const pat = patterns.patterns.find((p) => p.id === patterns.activePatternId);
      if (pat) {
        playlist.seedFromPattern(pat.id, pat.bars * pat.beatsPerMeasure);
      }
    }
  }

  async function generate() {
    const paramStore = useParameterStore();
    generating.value = true;
    error.value = null;
    progress.value = {
      job_id: '',
      stage_name: 'Pipeline',
      stage_index: 0,
      total_stages: 14,
      percent: 0,
      message: 'Starting generation…',
    };
    try {
      await subscribeEvents();
      summary.value = await generateComposition(toValue(paramStore.snapshot));
      await loadComposition();
      syncPatternsFromSummary();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      generating.value = false;
    }
  }

  async function downloadMidi() {
    exportError.value = null;
    try {
      const bytes = await exportMidi();
      const name = `${summary.value?.title ?? 'aurora'}.mid`;
      await promptAndSaveBytes(name, new Uint8Array(bytes), [
        { name: 'MIDI', extensions: ['mid', 'midi'] },
      ]);
    } catch (e) {
      exportError.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function downloadMusicXml() {
    exportError.value = null;
    try {
      const xml = await exportMusicXml();
      const name = `${summary.value?.title ?? 'aurora'}.musicxml`;
      await promptAndSaveText(name, xml, [
        { name: 'MusicXML', extensions: ['musicxml', 'xml'] },
      ]);
    } catch (e) {
      exportError.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function downloadAbc() {
    exportError.value = null;
    try {
      lastAbc.value = await exportAbc();
      const name = `${summary.value?.title ?? 'aurora'}.abc`;
      await promptAndSaveText(name, lastAbc.value, [
        { name: 'ABC', extensions: ['abc'] },
      ]);
    } catch (e) {
      exportError.value = e instanceof Error ? e.message : String(e);
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
    const playback = usePlaybackStore();
    try {
      error.value = null;
      const bytes = await exportMidi();
      const bpm = summary.value?.tempo_bpm ?? 120;
      const beatsPerMeasure =
        composition.value?.global.meter_map.default.beats ?? 4;
      playback.onPlayStart(bpm, 0, beatsPerMeasure);
      const duration = await playMidiBytes(bytes);
      playback.setFromMidi(duration, bpm, beatsPerMeasure);
      playing.value = true;
    } catch (e) {
      playing.value = false;
      playback.onPlayStop();
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function stop() {
    const playback = usePlaybackStore();
    await stopPlayback();
    playback.onPlayStop();
    playing.value = false;
  }

  async function patchNote(nodeId: NodeId, pitchMidi: number) {
    try {
      error.value = null;
      summary.value = await applyNotePatch(nodeId, pitchMidi);
      await loadComposition();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function deleteNote(nodeId: NodeId) {
    try {
      error.value = null;
      summary.value = await deleteNoteCmd(nodeId);
      await loadComposition();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function insertNote(params: InsertNoteParams) {
    try {
      error.value = null;
      summary.value = await insertNoteCmd(params);
      await loadComposition();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadFromProject(path: string) {
    try {
      error.value = null;
      summary.value = await loadProject(path);
      await loadComposition();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function resetWorkspace(summaryFromBackend?: CompositionSummary) {
    const playback = usePlaybackStore();
    playing.value = false;
    playback.onPlayStop();
    progress.value = null;
    error.value = null;
    exportError.value = null;
    lastAbc.value = null;
    lastSvgPreview.value = null;
    if (summaryFromBackend) {
      summary.value = summaryFromBackend;
    }
    usePlaylistStore().clearAll();
    usePatternsStore().resetToDefault({
      bars: summaryFromBackend?.bars ?? 8,
      tempoBpm: summaryFromBackend?.tempo_bpm ?? 120,
    });
    await loadComposition();
  }

  function clearAfterNew() {
    void resetWorkspace();
  }

  return {
    summary,
    composition,
    timeline,
    pianoRollNotes,
    progress,
    generating,
    error,
    exportError,
    revision,
    lastAbc,
    lastSvgPreview,
    playing,
    generate,
    loadComposition,
    initWorkspace,
    resetWorkspace,
    downloadMidi,
    downloadMusicXml,
    downloadAbc,
    loadSvgPreview,
    play,
    stop,
    patchNote,
    deleteNote,
    insertNote,
    loadFromProject,
    clearAfterNew,
  };
});
