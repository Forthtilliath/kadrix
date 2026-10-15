mod batch;
mod commands;
mod models;
mod params;
mod pipeline;
mod report;
mod state;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::model_status,
            commands::download_models,
            commands::list_images,
            commands::preview,
            commands::run_batch,
            commands::cancel_batch,
            commands::open_output,
        ])
        .run(tauri::generate_context!())
        .expect("impossible de démarrer Kadrix");
}
