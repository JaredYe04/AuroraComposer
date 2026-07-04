export interface NodeId {
  index: number;
  generation: number;
}

export interface UiParameterSnapshot {
  key: number;
  mode: string;
  style: string;
  beam_width: number;
  bars: number;
  tempo_bpm: number;
  emotion_valence: number;
  emotion_arousal: number;
  harmony_complexity: number;
  counterpoint_strictness: number;
  drum_density: number;
  drum_accent_emphasis: number;
  drum_hihat_density: number;
  progression_mode: string;
  tonal_conservatism: number;
  accompaniment_instrument: string;
  seed: number;
}

export interface CompositionSummary {
  title: string;
  bars: number;
  voice_count: number;
  note_count: number;
  tempo_bpm: number;
  key: number;
}

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  plugin_type: string;
  execution_tier: string;
  state: string;
  load_path: string;
}

export interface JobProgressEvent {
  job_id: string;
  stage_name: string;
  stage_index: number;
  total_stages: number;
  percent: number;
  message: string;
}

/** Overall pipeline progress in [0, 1] from stage index + within-stage percent. */
export function overallJobProgress(event: JobProgressEvent | null, generating = false): number {
  if (!event) return generating ? 0 : 0;
  const total = Math.max(1, event.total_stages ?? 14);
  const stage = Math.max(1, Math.min(total, event.stage_index ?? 1));
  const within = Math.max(0, Math.min(1, event.percent ?? 0));
  return Math.min(1, (stage - 1 + within) / total);
}

export interface JobCompleteEvent {
  job_id: string;
  summary: CompositionSummary;
}

export type SectionRole =
  | 'Intro'
  | 'Verse'
  | 'PreChorus'
  | 'Chorus'
  | 'Bridge'
  | 'Breakdown'
  | 'Build'
  | 'Drop'
  | 'Outro'
  | 'Coda'
  | 'Exposition'
  | 'Development'
  | 'Recapitulation'
  | 'Transition'
  | 'Interlude'
  | { Custom: number };

export interface TimelineSection {
  id: NodeId;
  role: SectionRole;
  label: string | null;
  start_measure: number;
  end_measure: number;
}

export interface TimelinePhrase {
  id: NodeId;
  section_id: NodeId;
  start_measure: number;
  end_measure: number;
  cadence: string | null;
}

export interface TimelineModel {
  total_measures: number;
  sections: TimelineSection[];
  phrases: TimelinePhrase[];
}

export type ProvenanceSource =
  | 'Generated'
  | 'ManualEdit'
  | 'Imported'
  | 'Repaired'
  | 'Plugin'
  | 'Transformed';

export type PipelineStageId =
  | 'style_resolver'
  | 'emotion_resolver'
  | 'structure_planning'
  | 'theme_planning'
  | 'harmony_skeleton'
  | 'rhythm_skeleton'
  | 'melody'
  | 'counterpoint'
  | 'bass'
  | 'drums'
  | 'decoration'
  | 'repair'
  | 'manual'
  | { custom: number };

export interface RuleRef {
  id: string;
  weight: number | null;
  score: number | null;
}

export interface SearchContext {
  step_index: number;
  beam_rank: number;
  beam_width: number;
  state_ref: { id: string };
  accumulated_score: number;
}

export interface ProvenanceRef {
  node_id: NodeId;
  patch_id: { '0': number } | null;
}

export type ProvenanceAgent =
  | { type: 'Engine'; stage: PipelineStageId }
  | { type: 'User'; user_id: string | null }
  | { type: 'Plugin'; plugin_id: string }
  | { type: 'Import'; format: string };

export interface Provenance {
  source: ProvenanceSource;
  stage: PipelineStageId | null;
  rule_ids: string[];
  rule_refs: RuleRef[];
  eval_score: number | null;
  search: SearchContext | null;
  parent: ProvenanceRef | null;
  created_at: string;
  agent: ProvenanceAgent;
  parameters_hash: string | null;
  explanation: string | null;
}

export interface RuleDefinition {
  id: string;
  display_id: string;
  name: string;
  category: string;
  description: string;
  weight: number;
  contribution_score: number | null;
}

export interface EventSummary {
  kind: string;
  pitch_display: string | null;
  duration_display: string | null;
  voice_name: string;
  measure_global: number;
  beat_numer: number;
  beat_denom: number;
}

export interface ProvenanceChainEntry {
  provenance: Provenance;
  rules: RuleDefinition[];
  display_summary: string;
  depth: number;
}

export interface EventLocator {
  node_id?: NodeId;
  measure_global?: number;
  voice_index?: number;
  event_index?: number;
}

export interface ProvenanceChain {
  event_id: NodeId;
  event_summary: EventSummary;
  entries: ProvenanceChainEntry[];
}

export interface BeatOffset {
  numer: number;
  denom: number;
}

export interface Pitch {
  midi: number;
  spelling: unknown | null;
}

export interface WrittenDuration {
  note_type: string;
  dots: number;
  tuplet: unknown | null;
}

export interface TimedEventBase {
  id: NodeId;
  offset: BeatOffset;
  duration: WrittenDuration;
  provenance: Provenance;
  visible: boolean;
}

export interface NoteEvent {
  base: TimedEventBase;
  pitch: Pitch;
  velocity: number;
  pitch_role: PitchRole | null;
}

export interface ChordEvent {
  base: TimedEventBase;
  pitches: { pitch: Pitch }[];
}

export type Event =
  | { kind: 'Note' } & NoteEvent
  | { kind: 'Chord' } & ChordEvent
  | { kind: 'Rest'; base: TimedEventBase }
  | { kind: 'Marker'; base: TimedEventBase }
  | { kind: 'Automation'; base: TimedEventBase };

export type PitchRole =
  | 'ChordTone'
  | 'PassingTone'
  | 'NeighborTone'
  | 'Appoggiatura'
  | 'Suspension'
  | 'Retardation'
  | 'EscapeTone'
  | 'PedalTone'
  | 'Ornament'
  | 'Unclassified';

export interface MeasureVoice {
  voice_id: number;
  events: Event[];
}

export interface Measure {
  id: NodeId;
  number: { local: number; global: number };
  voices: MeasureVoice[];
}

export interface Phrase {
  id: NodeId;
  measures: Measure[];
}

export interface Section {
  id: NodeId;
  metadata: { role: SectionRole; label: string | null };
  phrases: Phrase[];
}

export interface Movement {
  sections: Section[];
}

export interface VoiceDef {
  id: number;
  name: string;
}

export interface TimeSignature {
  beats: number;
  beat_type: number;
}

export interface MeterMap {
  default: TimeSignature;
  changes: Array<{ at_measure: number; meter: TimeSignature }>;
}

export interface KeySignature {
  tonic: { pc: number };
  mode: string;
}

export interface KeyMap {
  default: KeySignature;
}

export interface Composition {
  metadata: { title: string; parameters_used: unknown };
  global: {
    tempo_map: { default_bpm: number };
    meter_map: MeterMap;
    key_map: KeyMap;
  };
  voice_registry: { voices: VoiceDef[] };
  movements: Movement[];
}

export interface PianoRollNote {
  nodeId: NodeId;
  voiceId: number;
  voiceName: string;
  pitchMidi: number;
  startMeasure: number;
  startBeat: BeatOffset;
  durationBeats: number;
  velocity: number;
  pitchRole: PitchRole | null;
  provenanceSource: ProvenanceSource;
  provenanceSummary: string;
}

const KEY_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

export function keyName(pc: number): string {
  return KEY_NAMES[pc % 12] ?? 'C';
}

export function nodeIdKey(id: NodeId): string {
  return `${id.index}:${id.generation}`;
}

export function nodeIdsEqual(a: NodeId, b: NodeId): boolean {
  return a.index === b.index && a.generation === b.generation;
}

export function midiToName(midi: number): string {
  const octave = Math.floor(midi / 12) - 1;
  return `${KEY_NAMES[midi % 12]}${octave}`;
}

export function beatToFloat(b: BeatOffset): number {
  return b.numer / b.denom;
}

export function durationToBeats(d: WrittenDuration): number {
  const base: Record<string, number> = {
    Whole: 4,
    Half: 2,
    Quarter: 1,
    Eighth: 0.5,
    Sixteenth: 0.25,
    ThirtySecond: 0.125,
  };
  let beats = base[d.note_type] ?? 1;
  if (d.dots === 1) beats *= 1.5;
  else if (d.dots === 2) beats *= 1.75;
  return beats;
}

export function sectionRoleLabel(role: SectionRole): string {
  if (typeof role === 'object' && 'Custom' in role) return `Custom ${role.Custom}`;
  return String(role);
}

export const SECTION_COLORS: Record<string, string> = {
  Intro: '#4a5568',
  Verse: '#3182ce',
  PreChorus: '#2b6cb0',
  Chorus: '#d69e2e',
  Bridge: '#805ad5',
  Breakdown: '#718096',
  Build: '#ed8936',
  Drop: '#e53e3e',
  Outro: '#4a5568',
  Coda: '#4a5568',
  Exposition: '#3182ce',
  Development: '#805ad5',
  Recapitulation: '#38a169',
  Transition: '#718096',
  Interlude: '#4a5568',
  Default: '#2d3748',
};

export const PROVENANCE_COLORS: Record<ProvenanceSource, { fill: string; border: string }> = {
  Generated: { fill: '#4299e1', border: '#2b6cb0' },
  ManualEdit: { fill: '#48bb78', border: '#2f855a' },
  Repaired: { fill: '#ed8936', border: '#c05621' },
  Plugin: { fill: '#9f7aea', border: '#6b46c1' },
  Imported: { fill: '#a0aec0', border: '#718096' },
  Transformed: { fill: '#38b2ac', border: '#2c7a7b' },
};

export function eventBase(event: Event): TimedEventBase {
  if ('base' in event) return event.base;
  throw new Error('Event missing base');
}

export function eventId(event: Event): NodeId {
  return eventBase(event).id;
}
