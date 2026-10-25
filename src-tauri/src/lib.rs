mod airpods;
mod ble_scanner;
mod commands;
mod config;
mod device_battery;
mod monitor;

use tauri::{App, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::StateFlags;

const CONFIG_FILE: &str = "config.json";

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = app.path().app_config_dir()?.join(CONFIG_FILE);
    let config = config::load(&config_path).inspect_err(|error| log::error!("{}: {error}", config_path.display()))?;
    monitor::start(app.handle(), config);
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    if let Err(error) = window.show().and_then(|()| window.set_focus()) {
        log::warn!("bringing the widget to the front failed: {error}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_main_window(app)))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([Target::new(TargetKind::LogDir { file_name: None }), Target::new(TargetKind::Stdout)])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::new().with_state_flags(StateFlags::POSITION).build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(setup)
        .invoke_handler(tauri::generate_handler![commands::get_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
