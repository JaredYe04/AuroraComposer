import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  Composition,
  CompositionSummary,
  EventLocator,
  NodeId,
  Provenance,
  ProvenanceChain,
  TimelineModel,
  UiParameterSnapshot,
  JobProgressEvent,
  JobCompleteEvent,
  PluginInfo,
} from '@/types/aurora';

export async function getParameters(): Promise<UiParameterSnapshot> {
  return invoke<UiParameterSnapshot>('get_parameters');
}

export async function setParameters(
  parameters: UiParameterSnapshot,
): Promise<UiParameterSnapshot> {
  return invoke<UiParameterSnapshot>('set_parameters', { parameters });
}

export async function generateComposition(
  params: UiParameterSnapshot,
): Promise<CompositionSummary> {
  return invoke<CompositionSummary>('generate_composition', { params });
}

export async function getComposition(): Promise<Composition | null> {
  return invoke<Composition | null>('get_composition');
}

export async function getTimeline(): Promise<TimelineModel | null> {
  return invoke<TimelineModel | null>('get_timeline');
}

export async function getEventProvenance(locator: EventLocator): Promise<Provenance> {
  return invoke<Provenance>('get_event_provenance', { locator });
}

export async function getProvenanceChain(locator: EventLocator): Promise<ProvenanceChain> {
  return invoke<ProvenanceChain>('get_provenance_chain', { locator });
}

export async function exportMidi(): Promise<number[]> {
  return invoke<number[]>('export_midi');
}

export async function exportMusicXml(): Promise<string> {
  return invoke<string>('export_musicxml');
}

export async function exportAbc(): Promise<string> {
  return invoke<string>('export_abc');
}

export async function exportSvgPreview(): Promise<string> {
  return invoke<string>('export_svg_preview');
}

export async function applyNotePatch(
  nodeId: NodeId,
  pitchMidi: number,
): Promise<CompositionSummary> {
  return invoke<CompositionSummary>('apply_note_patch', {
    node_id: nodeId,
    new_midi: pitchMidi,
  });
}

export async function saveProject(path: string): Promise<void> {
  return invoke<void>('save_project', { path });
}

export async function loadProject(path: string): Promise<CompositionSummary> {
  return invoke<CompositionSummary>('load_project', { path });
}

export async function newProject(): Promise<void> {
  return invoke<void>('new_project');
}

export async function exportPdfBytes(): Promise<number[]> {
  return invoke<number[]>('export_pdf_bytes');
}

export async function listWasmPlugins(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>('list_wasm_plugins');
}

export async function registerWasmPlugin(path: string): Promise<void> {
  return invoke<void>('register_wasm_plugin', { path });
}

export function onJobProgress(
  handler: (payload: JobProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<JobProgressEvent>('aurora://job-progress', (e) => handler(e.payload));
}

export function onJobComplete(
  handler: (payload: JobCompleteEvent) => void,
): Promise<UnlistenFn> {
  return listen<JobCompleteEvent>('aurora://job-complete', (e) => handler(e.payload));
}

export function onJobError(
  handler: (payload: { job_id: string; error: string }) => void,
): Promise<UnlistenFn> {
  return listen('aurora://job-error', (e) =>
    handler(e.payload as { job_id: string; error: string }),
  );
}

export function onAstChanged(
  handler: (payload: { project_id: string; patch_id: unknown }) => void,
): Promise<UnlistenFn> {
  return listen('aurora://ast-changed', (e) =>
    handler(e.payload as { project_id: string; patch_id: unknown }),
  );
}
