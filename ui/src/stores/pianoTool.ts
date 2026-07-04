import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { BeatOffset, PianoRollNote } from '@/types/aurora';

export type PianoToolMode = 'pointer' | 'box' | 'brush' | 'eraser' | 'split';

export interface ClipboardNote {
  pitchMidi: number;
  startBeat: BeatOffset;
  durationBeats: number;
  velocity: number;
  voiceId: number;
}

export const usePianoToolStore = defineStore('pianoTool', () => {
  const mode = ref<PianoToolMode>('pointer');
  const clipboard = ref<ClipboardNote[]>([]);

  function setMode(next: PianoToolMode) {
    mode.value = next;
  }

  function copyNotes(notes: PianoRollNote[]) {
    clipboard.value = notes.map((n) => ({
      pitchMidi: n.pitchMidi,
      startBeat: { ...n.startBeat },
      durationBeats: n.durationBeats,
      velocity: n.velocity,
      voiceId: n.voiceId,
    }));
  }

  function clearClipboard() {
    clipboard.value = [];
  }

  return {
    mode,
    clipboard,
    setMode,
    copyNotes,
    clearClipboard,
  };
});
