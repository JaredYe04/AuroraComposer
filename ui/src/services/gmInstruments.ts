/** GM percussion channel (MIDI channel 10, zero-indexed 9). */
export const DRUM_CHANNEL = 9;

/** Also accept MIDI channel 10 when parsers expose it literally. */
export function isDrumChannel(channel: number): boolean {
  return channel === DRUM_CHANNEL || channel === 10;
}

const NOTE_NAMES = ['C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B'] as const;

/** Convert MIDI note number to soundfont-player note name (e.g. 36 → C2, 42 → Gb2). */
export function midiToNoteName(midi: number): string {
  return `${NOTE_NAMES[midi % 12]}${Math.floor(midi / 12) - 1}`;
}

/** GM Level 1 program numbers → soundfont-player instrument names (FluidR3). */
export const GM_PROGRAM_NAMES: readonly string[] = [
  'acoustic_grand_piano',
  'bright_acoustic_piano',
  'electric_grand_piano',
  'honkytonk_piano',
  'electric_piano_1',
  'electric_piano_2',
  'harpsichord',
  'clavinet',
  'celesta',
  'glockenspiel',
  'music_box',
  'vibraphone',
  'marimba',
  'xylophone',
  'tubular_bells',
  'dulcimer',
  'drawbar_organ',
  'percussive_organ',
  'rock_organ',
  'church_organ',
  'reed_organ',
  'accordion',
  'harmonica',
  'tango_accordion',
  'acoustic_guitar_nylon',
  'acoustic_guitar_steel',
  'electric_guitar_jazz',
  'electric_guitar_clean',
  'electric_guitar_muted',
  'overdriven_guitar',
  'distortion_guitar',
  'guitar_harmonics',
  'acoustic_bass',
  'electric_bass_finger',
  'electric_bass_pick',
  'fretless_bass',
  'slap_bass_1',
  'slap_bass_2',
  'synth_bass_1',
  'synth_bass_2',
  'violin',
  'viola',
  'cello',
  'contrabass',
  'tremolo_strings',
  'pizzicato_strings',
  'orchestral_harp',
  'timpani',
  'string_ensemble_1',
  'string_ensemble_2',
  'synth_strings_1',
  'synth_strings_2',
  'choir_aahs',
  'voice_oohs',
  'synth_choir',
  'orchestra_hit',
  'trumpet',
  'trombone',
  'tuba',
  'muted_trumpet',
  'french_horn',
  'brass_section',
  'synth_brass_1',
  'synth_brass_2',
  'soprano_sax',
  'alto_sax',
  'tenor_sax',
  'baritone_sax',
  'oboe',
  'english_horn',
  'bassoon',
  'clarinet',
  'piccolo',
  'flute',
  'recorder',
  'pan_flute',
  'blown_bottle',
  'shakuhachi',
  'whistle',
  'ocarina',
  'lead_1_square',
  'lead_2_sawtooth',
  'lead_3_calliope',
  'lead_4_chiff',
  'lead_5_charang',
  'lead_6_voice',
  'lead_7_fifths',
  'lead_8_bass__lead',
  'pad_1_new_age',
  'pad_2_warm',
  'pad_3_polysynth',
  'pad_4_choir',
  'pad_5_bowed',
  'pad_6_metallic',
  'pad_7_halo',
  'pad_8_sweep',
  'fx_1_rain',
  'fx_2_soundtrack',
  'fx_3_crystal',
  'fx_4_atmosphere',
  'fx_5_brightness',
  'fx_6_goblins',
  'fx_7_echoes',
  'fx_8_scifi',
  'sitar',
  'banjo',
  'shamisen',
  'koto',
  'kalimba',
  'bagpipe',
  'fiddle',
  'shanai',
  'tinkle_bell',
  'agogo',
  'steel_drums',
  'woodblock',
  'taiko_drum',
  'melodic_tom',
  'synth_drum',
  'reverse_cymbal',
  'guitar_fret_noise',
  'breath_noise',
  'seashore',
  'bird_tweet',
  'telephone_ring',
  'helicopter',
  'applause',
  'gunshot',
];

/**
 * FluidR3 GM channel-10 kit rendered on channel 10 (not synth_drum).
 * gleitz CDN lacks this; supersational fork provides it.
 */
export const PERCUSSION_CDN =
  'https://raw.githubusercontent.com/supersational/midi-js-soundfonts/gh-pages/FluidR3_GM';

export const PERCUSSION_SOUNDFONT_JS = `${PERCUSSION_CDN}/percussion-mp3.js`;

export function percussionSampleBase(name = 'percussion', format = 'mp3'): string {
  return `${PERCUSSION_CDN}/${name}-${format}/`;
}

export function gmInstrumentName(channel: number, program: number): string {
  if (isDrumChannel(channel)) return 'percussion';
  return GM_PROGRAM_NAMES[program] ?? 'acoustic_grand_piano';
}

export function instrumentCacheKey(channel: number, program: number): string {
  if (isDrumChannel(channel)) return 'drums';
  return `${channel}:${program}`;
}
