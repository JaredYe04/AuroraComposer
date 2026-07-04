import { Midi } from '@tonejs/midi';
import * as Tone from 'tone';
import Soundfont from 'soundfont-player';
import {
  gmInstrumentName,
  instrumentCacheKey,
  isDrumChannel,
  PERCUSSION_SOUNDFONT_JS,
} from '@/services/gmInstruments';

export type PlaybackState = 'stopped' | 'playing' | 'paused';
export type MelodyEngine = 'gm' | 'sine' | 'square' | 'triangle' | 'sawtooth';

interface ScheduledNote {
  time: number;
  midi: number;
  duration: number;
  velocity: number;
  channel: number;
  program: number;
}

type InstrumentPlayer = Awaited<ReturnType<typeof Soundfont.instrument>>;
type ActiveVoice = { stop: (when?: number) => void };

const SCHEDULE_LOOKAHEAD_SEC = 0.08;

let playbackEndTime = 0;
let positionRaf: number | null = null;
let onPlaybackTick: ((seconds: number, state: PlaybackState) => void) | null = null;
let cachedMidi: Midi | null = null;
let scheduleOffset = 0;
let scheduleStartAc = 0;
let stopTimer: ReturnType<typeof setTimeout> | null = null;
let scheduleGeneration = 0;
let playingState: PlaybackState = 'stopped';
let suppressStopCallback = false;
let melodyEngine: MelodyEngine = 'gm';
let melodyChannel: number | null = null;

const instrumentLoaders = new Map<string, Promise<InstrumentPlayer>>();
const resolvedPlayers = new Map<string, InstrumentPlayer>();
const activeVoices: ActiveVoice[] = [];
const waveSynths = new Map<Exclude<MelodyEngine, 'gm'>, Tone.PolySynth>();

let nativeMasterGain: GainNode | null = null;

function getAudioContext(): AudioContext {
  return Tone.getContext().rawContext as AudioContext;
}

/** Native gain node for SoundFont output (must not use Tone-wrapped nodes). */
function ensureNativeMaster(): GainNode {
  if (!nativeMasterGain) {
    const ac = getAudioContext();
    nativeMasterGain = ac.createGain();
    nativeMasterGain.gain.value = 0.7;
    nativeMasterGain.connect(ac.destination);
  }
  return nativeMasterGain;
}

export function setMasterVolume(linear: number): void {
  const clamped = Math.max(0, Math.min(1, linear));
  ensureNativeMaster().gain.value = clamped;
  Tone.getDestination().volume.value = Tone.gainToDb(clamped);
}

export function setMelodyEngine(engine: MelodyEngine): void {
  melodyEngine = engine;
}

export function getMelodyEngine(): MelodyEngine {
  return melodyEngine;
}

function getWaveSynth(type: Exclude<MelodyEngine, 'gm'>): Tone.PolySynth {
  let synth = waveSynths.get(type);
  if (!synth) {
    synth = new Tone.PolySynth(Tone.Synth, {
      oscillator: { type },
      envelope: { attack: 0.02, decay: 0.2, sustain: 0.3, release: 0.4 },
    }).connect(Tone.getDestination());
    waveSynths.set(type, synth);
  }
  return synth;
}

function detectMelodyChannel(midi: Midi): void {
  melodyChannel = null;
  for (const track of midi.tracks) {
    const channel = track.channel ?? 0;
    if (!isDrumChannel(channel) && track.notes.length > 0) {
      melodyChannel = channel;
      return;
    }
  }
}

function usesWaveMelody(note: ScheduledNote): boolean {
  return (
    melodyEngine !== 'gm' &&
    melodyChannel != null &&
    note.channel === melodyChannel &&
    !isDrumChannel(note.channel)
  );
}

function needsSoundfont(note: ScheduledNote): boolean {
  return !usesWaveMelody(note);
}

function playerKey(channel: number, program: number): string {
  return isDrumChannel(channel) ? 'drums' : instrumentCacheKey(channel, program);
}

async function loadDrumKit(): Promise<InstrumentPlayer> {
  const ac = getAudioContext();
  const destination = ensureNativeMaster();
  return Soundfont.instrument(ac, PERCUSSION_SOUNDFONT_JS, { destination }).catch((err) => {
    const msg = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to load GM drum kit: ${msg}`);
  });
}

async function loadMelodicInstrument(channel: number, program: number): Promise<InstrumentPlayer> {
  const name = gmInstrumentName(channel, program);
  return Soundfont.instrument(getAudioContext(), name, {
    destination: ensureNativeMaster(),
    soundfont: 'FluidR3_GM',
  }).catch((err) => {
    const msg = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to load GM instrument "${name}": ${msg}`);
  });
}

async function ensurePlayer(key: string, loader: () => Promise<InstrumentPlayer>): Promise<InstrumentPlayer> {
  const cached = resolvedPlayers.get(key);
  if (cached) return cached;

  let pending = instrumentLoaders.get(key);
  if (!pending) {
    pending = loader().catch((err) => {
      instrumentLoaders.delete(key);
      throw err;
    });
    instrumentLoaders.set(key, pending);
  }

  const player = await pending;
  resolvedPlayers.set(key, player);
  return player;
}

function clearStopTimer() {
  if (stopTimer != null) {
    clearTimeout(stopTimer);
    stopTimer = null;
  }
}

function stopPositionLoop() {
  if (positionRaf != null) {
    cancelAnimationFrame(positionRaf);
    positionRaf = null;
  }
}

function currentPlaybackSeconds(): number {
  if (playingState !== 'playing') {
    return scheduleOffset;
  }
  const elapsed = getAudioContext().currentTime - scheduleStartAc;
  return Math.min(playbackEndTime, scheduleOffset + Math.max(0, elapsed));
}

function startPositionLoop() {
  stopPositionLoop();
  const loop = () => {
    onPlaybackTick?.(currentPlaybackSeconds(), playingState);
    if (playingState === 'playing') {
      positionRaf = requestAnimationFrame(loop);
    } else {
      positionRaf = null;
    }
  };
  positionRaf = requestAnimationFrame(loop);
}

function clearWaveSynths() {
  for (const synth of waveSynths.values()) {
    try {
      synth.releaseAll(0);
    } catch {
      /* ignore */
    }
    try {
      synth.dispose();
    } catch {
      /* ignore */
    }
  }
  waveSynths.clear();
}

function clearVoices() {
  for (const voice of activeVoices) {
    try {
      voice.stop();
    } catch {
      /* ignore */
    }
  }
  activeVoices.length = 0;
  clearWaveSynths();
}

function collectNotes(midi: Midi): ScheduledNote[] {
  const notes: ScheduledNote[] = [];
  for (const track of midi.tracks) {
    const channel = track.channel ?? 0;
    const program = track.instrument?.number ?? 0;
    for (const note of track.notes) {
      notes.push({
        time: note.time,
        midi: note.midi,
        duration: note.duration,
        velocity: note.velocity,
        channel,
        program,
      });
    }
  }
  notes.sort((a, b) => a.time - b.time);
  return notes;
}

function computePlaybackEndTime(notes: ScheduledNote[]): number {
  let end = 0.01;
  for (const note of notes) {
    if (isDrumChannel(note.channel)) {
      end = Math.max(end, note.time + 0.05);
    } else {
      end = Math.max(end, note.time + Math.max(note.duration, 0.01));
    }
  }
  return end;
}

function emitTick(state: PlaybackState) {
  if (suppressStopCallback && state === 'stopped') return;
  const seconds = state === 'stopped' ? scheduleOffset : currentPlaybackSeconds();
  onPlaybackTick?.(seconds, state);
}

async function preloadInstruments(notes: ScheduledNote[], offsetSec: number) {
  const keys = new Set<string>();
  for (const note of notes) {
    if (note.time < offsetSec - 0.001 || !needsSoundfont(note)) continue;
    keys.add(playerKey(note.channel, note.program));
  }

  await Promise.all(
    [...keys].map((key) => {
      if (key === 'drums') {
        return ensurePlayer(key, loadDrumKit);
      }
      const [channelStr, programStr] = key.split(':');
      const channel = Number(channelStr);
      const program = Number(programStr);
      return ensurePlayer(key, () => loadMelodicInstrument(channel, program));
    }),
  );
}

function playScheduledNote(audioTime: number, note: ScheduledNote) {
  const isDrum = isDrumChannel(note.channel);
  if (!isDrum && note.duration <= 0) return;

  const gain = Math.max(0.05, note.velocity);

  if (usesWaveMelody(note)) {
    getWaveSynth(melodyEngine as Exclude<MelodyEngine, 'gm'>).triggerAttackRelease(
      Tone.Frequency(note.midi, 'midi').toFrequency(),
      note.duration,
      audioTime,
      gain,
    );
    return;
  }

  const key = playerKey(note.channel, note.program);
  const player = resolvedPlayers.get(key);
  if (!player) {
    console.error(`SoundFont player "${key}" not preloaded`);
    return;
  }

  const handle = player.play(
    note.midi,
    audioTime,
    isDrum ? { gain } : { duration: note.duration, gain },
  );
  if (handle) {
    activeVoices.push(handle);
  }
}

function scheduleNotesAtOffset(offsetSec: number, generation: number) {
  if (!cachedMidi) return;

  const ac = getAudioContext();
  scheduleStartAc = ac.currentTime + SCHEDULE_LOOKAHEAD_SEC;
  scheduleOffset = offsetSec;

  const allNotes = collectNotes(cachedMidi);
  for (const note of allNotes) {
    if (note.time < offsetSec - 0.001) continue;
    const when = scheduleStartAc + (note.time - offsetSec);
    playScheduledNote(when, note);
  }

  const remainingSec = Math.max(0, playbackEndTime - offsetSec);
  clearStopTimer();
  stopTimer = setTimeout(() => {
    if (generation !== scheduleGeneration) return;
    void stopPlayback();
  }, remainingSec * 1000 + SCHEDULE_LOOKAHEAD_SEC * 1000 + 120);
}

async function scheduleFromOffset(offsetSec: number) {
  if (!cachedMidi) return;

  clearStopTimer();
  clearVoices();
  scheduleGeneration += 1;
  const generation = scheduleGeneration;

  const allNotes = collectNotes(cachedMidi);
  await preloadInstruments(allNotes, offsetSec);

  if (generation !== scheduleGeneration) return;

  scheduleNotesAtOffset(offsetSec, generation);

  if (cachedMidi.header.tempos.length > 0) {
    Tone.getTransport().bpm.value = cachedMidi.header.tempos[0].bpm;
  }
  Tone.getTransport().seconds = offsetSec;
  Tone.getTransport().position = offsetSec;
  if (Tone.getTransport().state !== 'started') {
    Tone.getTransport().start();
  }

  playingState = 'playing';
  startPositionLoop();
  emitTick('playing');
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
  ensureNativeMaster();
}

export async function playMidiBytes(bytes: number[] | Uint8Array): Promise<number> {
  await ensureAudioStarted();

  suppressStopCallback = true;
  clearStopTimer();
  clearVoices();
  stopPositionLoop();
  playingState = 'stopped';
  Tone.getTransport().stop();
  Tone.getTransport().cancel();
  suppressStopCallback = false;

  const buffer = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  cachedMidi = new Midi(
    buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength),
  );

  detectMelodyChannel(cachedMidi);

  const allNotes = collectNotes(cachedMidi);
  playbackEndTime = computePlaybackEndTime(allNotes);
  scheduleOffset = 0;

  await scheduleFromOffset(0);
  return playbackEndTime;
}

export async function stopPlayback(): Promise<void> {
  scheduleGeneration += 1;
  clearStopTimer();
  clearVoices();
  stopPositionLoop();
  playingState = 'stopped';
  Tone.getTransport().stop();
  Tone.getTransport().cancel();
  emitTick('stopped');
}

export async function pausePlayback(): Promise<void> {
  if (playingState !== 'playing') return;
  scheduleGeneration += 1;
  clearStopTimer();
  scheduleOffset = currentPlaybackSeconds();
  clearVoices();
  stopPositionLoop();
  playingState = 'paused';
  Tone.getTransport().pause();
  emitTick('paused');
}

export async function resumePlayback(): Promise<void> {
  if (playingState !== 'paused') return;
  await scheduleFromOffset(scheduleOffset);
}

export function seekTransport(seconds: number): void {
  const clamped = Math.max(0, Math.min(playbackEndTime, seconds));
  void scheduleFromOffset(clamped);
  emitTick(playbackState());
}

export function playbackState(): PlaybackState {
  return playingState;
}

export function getTransportSeconds(): number {
  return currentPlaybackSeconds();
}

export function getTransportBpm(): number {
  return Tone.getTransport().bpm.value;
}

export function getPlaybackDuration(): number {
  return playbackEndTime;
}
