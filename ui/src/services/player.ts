import { Midi } from '@tonejs/midi';
import * as Tone from 'tone';

export type PlaybackState = 'stopped' | 'playing' | 'paused';

let synth: Tone.PolySynth | null = null;
let scheduledStop: Tone.ToneEvent | null = null;
let playbackEndTime = 0;

function getSynth(): Tone.PolySynth {
  if (!synth) {
    synth = new Tone.PolySynth(Tone.Synth, {
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.02, decay: 0.1, sustain: 0.3, release: 0.5 },
    }).toDestination();
    synth.volume.value = -8;
  }
  return synth;
}

export async function ensureAudioStarted(): Promise<void> {
  if (Tone.getContext().state !== 'running') {
    await Tone.start();
  }
}

export async function playMidiBytes(bytes: number[] | Uint8Array): Promise<void> {
  await ensureAudioStarted();
  await stopPlayback();

  const buffer = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  const midi = new Midi(buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength));
  const instrument = getSynth();

  Tone.getTransport().cancel();
  Tone.getTransport().position = 0;

  if (midi.header.tempos.length > 0) {
    Tone.getTransport().bpm.value = midi.header.tempos[0].bpm;
  }

  playbackEndTime = midi.duration;

  for (const track of midi.tracks) {
    for (const note of track.notes) {
      instrument.triggerAttackRelease(
        note.name,
        note.duration,
        note.time,
        Math.max(0.05, note.velocity),
      );
    }
  }

  Tone.getTransport().start();

  scheduledStop = new Tone.ToneEvent((time) => {
    instrument.releaseAll(time);
    Tone.getTransport().stop();
    Tone.getTransport().position = 0;
  });
  scheduledStop.start(playbackEndTime + 0.1);
}

export async function stopPlayback(): Promise<void> {
  scheduledStop?.dispose();
  scheduledStop = null;
  Tone.getTransport().stop();
  Tone.getTransport().cancel();
  Tone.getTransport().position = 0;
  synth?.releaseAll();
}

export async function pausePlayback(): Promise<void> {
  Tone.getTransport().pause();
}

export async function resumePlayback(): Promise<void> {
  await ensureAudioStarted();
  Tone.getTransport().start();
}

export function playbackState(): PlaybackState {
  if (Tone.getTransport().state === 'started') {
    return 'playing';
  }
  if (Tone.getTransport().state === 'paused') {
    return 'paused';
  }
  return 'stopped';
}
