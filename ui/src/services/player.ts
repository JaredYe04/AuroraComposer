import { Midi } from '@tonejs/midi';
import * as Tone from 'tone';

export type PlaybackState = 'stopped' | 'playing' | 'paused';

let synth: Tone.PolySynth | null = null;
let scheduledStop: Tone.ToneEvent | null = null;
let playbackEndTime = 0;
let positionRaf: number | null = null;
let onPlaybackTick: ((seconds: number, state: PlaybackState) => void) | null = null;

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

function stopPositionLoop() {
  if (positionRaf != null) {
    cancelAnimationFrame(positionRaf);
    positionRaf = null;
  }
}

function startPositionLoop() {
  stopPositionLoop();
  const loop = () => {
    const state = playbackState();
    onPlaybackTick?.(Tone.getTransport().seconds, state);
    if (state === 'playing') {
      positionRaf = requestAnimationFrame(loop);
    } else {
      positionRaf = null;
      onPlaybackTick?.(Tone.getTransport().seconds, state);
    }
  };
  positionRaf = requestAnimationFrame(loop);
}

export function setOnPlaybackTick(
  cb: ((seconds: number, state: PlaybackState) => void) | null,
): void {
  onPlaybackTick = cb;
}

export async function ensureAudioStarted(): Promise<void> {
  if (Tone.getContext().state !== 'running') {
    await Tone.start();
  }
}

export async function playMidiBytes(bytes: number[] | Uint8Array): Promise<number> {
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
  startPositionLoop();

  scheduledStop = new Tone.ToneEvent((time) => {
    instrument.releaseAll(time);
    Tone.getTransport().stop();
    Tone.getTransport().position = 0;
    stopPositionLoop();
    onPlaybackTick?.(0, 'stopped');
  });
  scheduledStop.start(playbackEndTime + 0.1);

  return playbackEndTime;
}

export async function stopPlayback(): Promise<void> {
  scheduledStop?.dispose();
  scheduledStop = null;
  stopPositionLoop();
  Tone.getTransport().stop();
  Tone.getTransport().cancel();
  Tone.getTransport().position = 0;
  synth?.releaseAll();
  onPlaybackTick?.(0, 'stopped');
}

export async function pausePlayback(): Promise<void> {
  Tone.getTransport().pause();
  stopPositionLoop();
  onPlaybackTick?.(Tone.getTransport().seconds, 'paused');
}

export async function resumePlayback(): Promise<void> {
  await ensureAudioStarted();
  Tone.getTransport().start();
  startPositionLoop();
}

export function seekTransport(seconds: number): void {
  Tone.getTransport().seconds = Math.max(0, Math.min(playbackEndTime, seconds));
  onPlaybackTick?.(Tone.getTransport().seconds, playbackState());
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

export function getTransportSeconds(): number {
  return Tone.getTransport().seconds;
}

export function getTransportBpm(): number {
  return Tone.getTransport().bpm.value;
}

export function getPlaybackDuration(): number {
  return playbackEndTime;
}
