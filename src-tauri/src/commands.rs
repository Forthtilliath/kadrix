//! Commandes appelées par l'interface. Le front n'a aucun accès direct au
//! système de fichiers : tout passe par ici, paramètres revérifiés.

use crate::batch;
use crate::models::{self, ModelInfo};
use crate::params::Params;
use crate::pipeline::{self, background::Canvas, export, Issue};
use crate::report::BatchEvent;
use crate::state::{models_dir, AppState};
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    model: &'static str,
    received: u64,
    total: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    image: Option<String>,
    issues: Vec<Issue>,
}

#[tauri::command]
pub fn model_status(app: AppHandle) -> Result<Vec<ModelInfo>, String> {
    Ok(models::status(&models_dir(&app)?))
}

#[tauri::command]
pub async fn download_models(app: AppHandle, on_progress: Channel<DownloadProgress>) -> Result<(), String> {
    let state = app.state::<AppState>();
    if state.downloading.swap(true, Ordering::SeqCst) {
        return Err("Téléchargement déjà en cours.".into());
    }
    let result = download_missing(&app, &on_progress).await;
    state.downloading.store(false, Ordering::SeqCst);
    result
}

async fn download_missing(app: &AppHandle, on_progress: &Channel<DownloadProgress>) -> Result<(), String> {
    let dir = models_dir(app)?;
    for spec in models::ALL {
        if models::is_present(&dir, spec) {
            continue;
        }
        let report = |received| {
            let _ = on_progress.send(DownloadProgress { model: spec.id, received, total: spec.size });
        };
        models::download(spec, &dir, report)
            .await
            .map_err(|e| format!("{} : {e:#}", spec.label))?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_images(dir: String) -> Result<Vec<String>, String> {
    let files = batch::list_images(Path::new(&dir)).map_err(|e| format!("Lecture du dossier impossible : {e}"))?;
    Ok(files.into_iter().map(|p| p.display().to_string()).collect())
}

#[tauri::command]
pub async fn preview(app: AppHandle, path: String, params: Params) -> Result<PreviewResult, String> {
    params.validate()?;
    if !batch::is_image(Path::new(&path)) {
        return Err("Fichier image introuvable.".into());
    }
    let task = tauri::async_runtime::spawn_blocking(move || {
        let canvas = Canvas::render(&params.background, params.width, params.height).map_err(|e| format!("{e:#}"))?;
        app.state::<AppState>().with_models(&app, |models| {
            let outcome = pipeline::process(models, Path::new(&path), &params, &canvas);
            let image = outcome.output.as_ref().and_then(|o| export::data_url(&o.composed, 900).ok());
            PreviewResult { image, issues: outcome.issues }
        })
    });
    task.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn run_batch(
    app: AppHandle,
    input_dir: String,
    output_dir: String,
    params: Params,
    on_event: Channel<BatchEvent>,
) -> Result<String, String> {
    params.validate()?;
    let (input, output) = batch::check_dirs(&input_dir, &output_dir)?;
    let state = app.state::<AppState>();
    if state.busy.swap(true, Ordering::SeqCst) {
        return Err("Un traitement est déjà en cours.".into());
    }
    state.cancel.store(false, Ordering::SeqCst);
    *state.last_output.lock().unwrap_or_else(|e| e.into_inner()) = Some(output.clone());

    let worker = app.clone();
    let task = tauri::async_runtime::spawn_blocking(move || {
        let state = worker.state::<AppState>();
        state.with_models(&worker, |models| {
            let emit = |event| {
                let _ = on_event.send(event);
            };
            batch::run(models, &input, &output, &params, &state.cancel, emit).map_err(|e| format!("{e:#}"))
        })
    });
    let result = task.await;
    state.busy.store(false, Ordering::SeqCst);
    let report = result.map_err(|e| e.to_string())???;
    Ok(report.display().to_string())
}

#[tauri::command]
pub fn cancel_batch(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

/// N'ouvre que le dernier dossier de sortie utilisé (aucun chemin du front).
#[tauri::command]
pub fn open_output(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let last = state.last_output.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let dir = last.ok_or("Aucun traitement lancé.")?;
    app.opener()
        .open_path(dir.display().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}
