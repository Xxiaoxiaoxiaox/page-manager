mod app;

use app::config::AppEntry;
use app::config::ManagerConfig;
use app::launch::launch_exe;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tauri::{
    Manager, PhysicalPosition, PhysicalSize, Position, Size, State,
    WebviewBuilder, WebviewUrl, WebviewWindowBuilder,
};

static SIDEBAR_WIDTH: AtomicU64 = AtomicU64::new(f64::to_bits(220.0));

fn get_sidebar_width() -> f64 {
    f64::from_bits(SIDEBAR_WIDTH.load(Ordering::Relaxed))
}

/// Compute usable client area in physical pixels.
/// On Windows, `inner_size()` returns the HWND client rect which may include
/// the DWM invisible resize border.  We detect the border width from the
/// horizontal difference between outer/inner and subtract it.
fn compute_usable_bounds(window: &tauri::Window) -> (i32, i32) {
    let inner = window.inner_size().unwrap_or(PhysicalSize { width: 1100, height: 720 });
    let outer = window.outer_size().unwrap_or(inner);

    // DWM invisible border per side (from horizontal difference)
    let h_diff = (outer.width as i64 - inner.width as i64).max(0);
    let border = (h_diff / 2) as i32;

    // Usable area = inner minus invisible borders
    let usable_w = inner.width as i32 - 2 * border;
    let usable_h = inner.height as i32 - border; // subtract bottom border only
    (usable_w.max(200), usable_h.max(200))
}

fn update_content_webview(window: &tauri::Window) {
    if let Some(wv) = window.get_webview("content_area") {
        let (usable_w, usable_h) = compute_usable_bounds(window);
        let scale = window.scale_factor().unwrap_or(1.0);
        let sx = ((get_sidebar_width() + 4.0) * scale) as i32;
        let sy = (44.0 * scale) as i32;
        let cw = (usable_w - sx).max(200);
        let ch = (usable_h - sy).max(200);
        let _ = wv.set_position(Position::Physical(PhysicalPosition::new(sx, sy)));
        let _ = wv.set_size(Size::Physical(PhysicalSize::new(cw as u32, ch as u32)));
    }
}




fn resolve_proxy_url(config: &ManagerConfig, target_url: &str) -> String {
    for app in &config.apps {
        if app.url == target_url {
            return match app.proxy.as_str() {
                "" | "direct" => String::new(),
                "local" => config.proxy_local.clone(),
                "7897" => config.proxy_7897.clone(),
                url if url.contains("://") => url.to_string(),
                _ => String::new(),
            };
        }
    }
    String::new()
}
pub struct AppState {
    pub config: Mutex<ManagerConfig>,
}

#[tauri::command]
fn get_apps(state: State<AppState>) -> Vec<AppEntry> {
    let config = state.config.lock().unwrap();
    config.apps.clone()
}

#[tauri::command]
fn add_app(state: State<AppState>, name: String, url: String, exe_path: String, proxy: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.apps.push(AppEntry { name, url, exe_path, proxy });
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
fn update_app(state: State<AppState>, index: usize, name: String, url: String, exe_path: String, proxy: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if index < config.apps.len() {
        config.apps[index] = AppEntry { name, url, exe_path, proxy };
        config.save();
    }
    Ok(())
}

#[tauri::command]
fn reorder_apps(state: State<AppState>, apps: Vec<AppEntry>) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.apps = apps;
    config.save();
    Ok(())
}

#[tauri::command]
fn launch_app(exe_path: String) -> Result<(), String> {
    launch_exe(&exe_path)
}

#[tauri::command]
fn get_proxies(state: State<AppState>) -> (String, String) {
    let config = state.config.lock().unwrap();
    (config.proxy_local.clone(), config.proxy_7897.clone())
}

#[tauri::command]
fn set_proxies(state: State<AppState>, local: String, proxy_7897: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.proxy_local = local;
    config.proxy_7897 = proxy_7897;
    config.save();
    Ok(())
}

#[tauri::command]
fn set_sidebar_width(width: f64) {
    SIDEBAR_WIDTH.store(f64::to_bits(width), Ordering::Relaxed);
}

#[tauri::command]
fn save_window_size(state: State<AppState>, width: f64, height: f64) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.window_width = width;
    config.window_height = height;
    config.save();
    Ok(())
}

#[tauri::command]
fn toggle_content_webview(app: tauri::AppHandle, visible: bool) -> Result<(), String> {
    if let Some(wv) = app.get_webview("content_area") {
        if visible { let _ = wv.show(); }
        else { let _ = wv.hide(); }
    }
    Ok(())
}

#[tauri::command]
fn navigate_to(app: tauri::AppHandle, url: String, sidebar_width: Option<f64>) -> Result<(), String> {
    if let Some(w) = sidebar_width {
        SIDEBAR_WIDTH.store(f64::to_bits(w), Ordering::Relaxed);
    }
    let parsed_url: tauri::Url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    let app_clone = app.clone();
    std::thread::spawn(move || {
        // Small delay to let the window finish layout/DPI adjustments
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Some(old) = app_clone.get_webview("content_area") {
            let _ = old.close();
        }
        if let Some(main_window) = app_clone.get_window("main") {
            let (usable_w, usable_h) = compute_usable_bounds(&main_window);
            let scale = main_window.scale_factor().unwrap_or(1.0);
            let sx = ((get_sidebar_width() + 4.0) * scale) as i32;
            let sy = (44.0 * scale) as i32;
            let cw = (usable_w - sx).max(200);
            let ch = (usable_h - sy).max(200);

            let proxy_str = {
                let st = app_clone.state::<AppState>();
                let cfg = st.config.lock().unwrap();
                resolve_proxy_url(&cfg, &url)
            };

            let mut builder = WebviewBuilder::new("content_area", WebviewUrl::External(parsed_url));
            if !proxy_str.is_empty() {
                let dir = std::env::temp_dir().join(format!("pake-proxy-{}", proxy_str.replace(":", "_").replace("/", "_")));
                let _ = std::fs::create_dir_all(&dir);
                builder = builder.data_directory(dir);
                if let Ok(pu) = proxy_str.parse::<tauri::Url>() {
                    builder = builder.proxy_url(pu);
                }
            }
            let _ = main_window.add_child(
                builder,
                PhysicalPosition::new(sx, sy),
                PhysicalSize::new(cw as u32, ch as u32),
            );
        }
    });
    Ok(())
}#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = ManagerConfig::load();

    tauri::Builder::default()
        .manage(AppState { config: Mutex::new(cfg.clone()) })
        .setup(move |app| {
            let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(std::path::PathBuf::new()))
                .title("Pake Manager")
                .min_inner_size(800.0, 500.0)
                .resizable(true);
            if cfg.maximized {
                builder = builder.maximized(true);
            } else {
                builder = builder.inner_size(cfg.window_width, cfg.window_height);
            }
            let _window = builder.build()?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Resized(_) = event {
                update_content_webview(window);
            }
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let st = window.state::<AppState>();
                let mut cfg = st.config.lock().unwrap();
                cfg.maximized = window.is_maximized().unwrap_or(false);
                if let Ok(s) = window.inner_size() {
                    let scale = window.scale_factor().unwrap_or(1.0);
                    cfg.window_width = s.width as f64 / scale;
                    cfg.window_height = s.height as f64 / scale;
                }
                cfg.save();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_apps, add_app, remove_app, update_app, reorder_apps, launch_app, navigate_to,
            get_proxies, set_proxies, set_sidebar_width, toggle_content_webview, save_window_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pake Manager");
}
