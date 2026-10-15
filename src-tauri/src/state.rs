use crate::models;
use crate::pipeline::Models;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct AppState {
    /// Chargés au premier usage (empreintes vérifiées à ce moment-là).
    models: Mutex<Option<Models>>,
    pub busy: AtomicBool,
    pub downloading: AtomicBool,
    pub cancel: AtomicBool,
    pub last_output: Mutex<Option<PathBuf>>,
}

pub fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("models"))
        .map_err(|e| format!("Dossier de données introuvable : {e}"))
}

impl AppState {
    /// Exécute `f` avec les modèles, en les chargeant si besoin.
    pub fn with_models<R>(&self, app: &AppHandle, f: impl FnOnce(&mut Models) -> R) -> Result<R, String> {
        let mut guard = self.models.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            let dir = models_dir(app)?;
            let face = models::verify(&dir, &models::FACE).map_err(|e| format!("{e:#}"))?;
            let matting = models::verify(&dir, &models::MATTING).map_err(|e| format!("{e:#}"))?;
            let loaded = Models::load(&face, &matting).map_err(|e| format!("Chargement des modèles : {e:#}"))?;
            *guard = Some(loaded);
        }
        Ok(f(guard.as_mut().expect("modèles chargés juste au-dessus")))
    }
}
