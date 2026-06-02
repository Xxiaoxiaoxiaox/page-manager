mod app;

use app::config::ManagerConfig;
use app::launch::launch_exe;
use tauri::State;
use std::sync::Mutex;

pub struct AppState {
    pub config: Mutex<ManagerConfig>,
}

#[tauri::command]
fn get_apps(state: State<AppState>) -> Vec<app::config::AppEntry> {
    let config = state.config.lock().unwrap();
    config.apps.clone()
}

#[tauri::command]
fn add_app(state: State<AppState>, name: String, url: String, exe_path: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.apps.push(app::config::AppEntry { name, url, exe_path });
    config.save();
    Ok(())
}

#[tauri::command]
fn remove_app(state: State<AppState>, index: usize) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if index < config.apps.len() {
        config.apps.remove(index);
        config.save();
    }
    Ok(())
}

#[tauri::command]
fn update_app(state: State<AppState>, index: usize, name: String, url: String, exe_path: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if index < config.apps.len() {
        config.apps[index] = app::config::AppEntry { name, url, exe_path };
        config.save();
    }
    Ok(())
}

#[tauri::command]
fn launch_app(exe_path: String) -> Result<(), String> {
    launch_exe(&exe_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = ManagerConfig::load();
    tauri::Builder::default()
        .manage(AppState { config: Mutex::new(cfg) })
        .invoke_handler(tauri::generate_handler![
            get_apps, add_app, remove_app, update_app, launch_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pake Manager");
}
