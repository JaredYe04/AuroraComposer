mod projection;
mod state;

use aurora_ast::{patch::patch_update_note_pitch, Composition, Event, Provenance, Project};
use aurora_ast::project::{
    PatchHistory, PluginConfig, ProjectManifest, PROJECT_FORMAT_VERSION,
};
use aurora_core::{CompositionSummary, NodeId, ParameterBundle, UiParameterSnapshot};
use aurora_engine::{EngineError, PipelineOrchestrator};
use aurora_export::{AbcExportConfig, ExportPipeline, MidiExportConfig, MusicXmlExportConfig};
use aurora_plugin::PluginInfo;
use projection::{
    build_timeline, resolve_event_provenance, resolve_provenance_chain, EventLocator,
    ProvenanceChain, TimelineModel,
};
use state::AppState;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

#[derive(Clone, serde::Serialize)]
struct JobProgressEvent {
    job_id: String,
    stage_name: String,
    stage_index: u8,
    percent: f32,
    message: String,
}

#[derive(Clone, serde::Serialize)]
struct JobCompleteEvent {
    job_id: String,
    summary: CompositionSummary,
}

fn composition_summary(comp: &Composition) -> CompositionSummary {
    let bars = comp
        .movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
        .flat_map(|p| &p.measures)
        .count() as u16;
    let note_count = comp
        .movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
        .flat_map(|p| &p.measures)
        .flat_map(|m| &m.voices)
        .flat_map(|v| &v.events)
        .filter(|e| matches!(e, Event::Note(_) | Event::Chord(_)))
        .count() as u32;

    CompositionSummary {
        title: comp.metadata.title.clone(),
        bars,
        voice_count: comp.voice_registry.voices.len() as u16,
        note_count,
        tempo_bpm: comp.global.tempo_map.default_bpm,
        key: comp.global.key_map.default.tonic.pc,
    }
}

#[tauri::command]
fn get_parameters(state: State<'_, AppState>) -> Result<UiParameterSnapshot, String> {
    Ok(UiParameterSnapshot::from(&*state.parameters.lock().map_err(|e| e.to_string())?))
}

#[tauri::command]
fn set_parameters(
    parameters: UiParameterSnapshot,
    state: State<'_, AppState>,
) -> Result<UiParameterSnapshot, String> {
    let bundle: ParameterBundle = parameters.clone().into();
    let mut stored = state.parameters.lock().map_err(|e| e.to_string())?;
    *stored = bundle;
    Ok(UiParameterSnapshot::from(&*stored))
}

#[tauri::command]
async fn generate_composition(
    params: UiParameterSnapshot,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CompositionSummary, String> {
    let bundle: ParameterBundle = params.into();
    {
        let mut stored = state.parameters.lock().map_err(|e| e.to_string())?;
        *stored = bundle.clone();
    }

    let job_id = Uuid::new_v4().to_string();
    let app_clone = app.clone();
    let job_id_str = job_id.clone();

    let result = tokio::task::spawn_blocking(move || {
        PipelineOrchestrator::new()
            .with_progress(Box::new({
                let app = app_clone.clone();
                let jid = job_id_str.clone();
                move |p| {
                    let _ = app.emit(
                        "aurora://job-progress",
                        JobProgressEvent {
                            job_id: jid.clone(),
                            stage_name: p.stage_name,
                            stage_index: p.stage_index,
                            percent: p.percent,
                            message: p.message,
                        },
                    );
                }
            }))
            .run(&bundle)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(engine_err)?;

    let summary = composition_summary(&result);

    {
        let mut comp = state.composition.lock().map_err(|e| e.to_string())?;
        *comp = Some(result);
    }

    let _ = app.emit(
        "aurora://job-complete",
        JobCompleteEvent {
            job_id,
            summary: summary.clone(),
        },
    );

    Ok(summary)
}

fn engine_err(err: EngineError) -> String {
    match err {
        EngineError::Aurora(a) => a.to_string(),
        EngineError::StageFailed { stage, message } => {
            format!("Stage {stage} failed: {message}")
        }
    }
}

fn require_composition(state: &State<'_, AppState>) -> Result<Composition, String> {
    state
        .composition
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "No composition generated yet".to_string())
}

#[tauri::command]
fn export_midi(state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let comp = require_composition(&state)?;
    ExportPipeline::to_midi(&comp, &MidiExportConfig::default()).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_musicxml(state: State<'_, AppState>) -> Result<String, String> {
    let comp = require_composition(&state)?;
    ExportPipeline::to_musicxml(&comp, &MusicXmlExportConfig::default())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_abc(state: State<'_, AppState>) -> Result<String, String> {
    let comp = require_composition(&state)?;
    ExportPipeline::to_abc(&comp, &AbcExportConfig::default()).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_svg_preview(state: State<'_, AppState>) -> Result<String, String> {
    let comp = require_composition(&state)?;
    ExportPipeline::to_svg_preview(&comp)
        .map(|r| r.svg)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_pdf_bytes(state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let comp = require_composition(&state)?;
    ExportPipeline::to_pdf_bytes(&comp).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_composition(state: State<'_, AppState>) -> Result<Option<Composition>, String> {
    Ok(state.composition.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
fn get_timeline(state: State<'_, AppState>) -> Result<Option<TimelineModel>, String> {
    let comp = state
        .composition
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    Ok(comp.as_ref().map(build_timeline))
}

#[tauri::command]
fn apply_note_patch(
    node_id: NodeId,
    new_midi: u8,
    state: State<'_, AppState>,
) -> Result<CompositionSummary, String> {
    let mut comp_guard = state.composition.lock().map_err(|e| e.to_string())?;
    let comp = comp_guard
        .as_mut()
        .ok_or_else(|| "No composition generated yet".to_string())?;
    let updated = patch_update_note_pitch(comp, node_id, new_midi).map_err(|e| e.to_string())?;
    *comp = updated;
    Ok(composition_summary(comp))
}

#[tauri::command]
fn save_project(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let comp = require_composition(&state)?;
    let params = state.parameters.lock().map_err(|e| e.to_string())?.clone();
    let now = iso_timestamp();
    let project = Project {
        manifest: ProjectManifest {
            project_id: Uuid::new_v4().to_string(),
            format_version: PROJECT_FORMAT_VERSION,
            name: comp.metadata.title.clone(),
            created_at: now.clone(),
            modified_at: now,
            author: None,
            description: None,
            tags: Vec::new(),
            aurora_engine_version: env!("CARGO_PKG_VERSION").into(),
        },
        composition: comp,
        parameters: params,
        history: PatchHistory::default(),
        export_cache: None,
        plugin_config: PluginConfig::default(),
    };
    project.save(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_project(path: String, state: State<'_, AppState>) -> Result<CompositionSummary, String> {
    let project = Project::load(&path).map_err(|e| e.to_string())?;
    let summary = composition_summary(&project.composition);
    {
        let mut comp = state.composition.lock().map_err(|e| e.to_string())?;
        *comp = Some(project.composition);
    }
    {
        let mut params = state.parameters.lock().map_err(|e| e.to_string())?;
        *params = project.parameters;
    }
    Ok(summary)
}

#[tauri::command]
fn new_project(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut comp = state.composition.lock().map_err(|e| e.to_string())?;
        *comp = None;
    }
    {
        let mut params = state.parameters.lock().map_err(|e| e.to_string())?;
        *params = ParameterBundle::default();
    }
    Ok(())
}

#[tauri::command]
fn list_wasm_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    let host = state.plugin_host.lock().map_err(|e| e.to_string())?;
    Ok(host
        .list_wasm_plugins()
        .iter()
        .map(PluginInfo::from)
        .collect())
}

#[tauri::command]
fn register_wasm_plugin(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut host = state.plugin_host.lock().map_err(|e| e.to_string())?;
    host.register_wasm_plugin(&path).map_err(|e| e.to_string())
}

fn iso_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[tauri::command]
fn get_event_provenance(
    locator: EventLocator,
    state: State<'_, AppState>,
) -> Result<Provenance, String> {
    let comp = require_composition(&state)?;
    resolve_event_provenance(&comp, &locator)
        .ok_or_else(|| "Event not found".to_string())
}

#[tauri::command]
fn get_provenance_chain(
    locator: EventLocator,
    state: State<'_, AppState>,
) -> Result<ProvenanceChain, String> {
    let comp = require_composition(&state)?;
    resolve_provenance_chain(&comp, &locator)
        .ok_or_else(|| "Event not found".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_parameters,
            set_parameters,
            generate_composition,
            export_midi,
            export_musicxml,
            export_abc,
            export_svg_preview,
            export_pdf_bytes,
            get_composition,
            get_timeline,
            get_event_provenance,
            get_provenance_chain,
            apply_note_patch,
            save_project,
            load_project,
            new_project,
            list_wasm_plugins,
            register_wasm_plugin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Aurora Composer");
}
