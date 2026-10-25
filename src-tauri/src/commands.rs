use std::path::PathBuf;

use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_opener::OpenerExt;

use crate::config::{self, ConfigError};
use crate::monitor::{Snapshot, SnapshotState};
use crate::window_position::{self, PositionError, PositionPath};

pub struct ConfigPath(pub PathBuf);

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("snapshot state is poisoned")]
    Poisoned,
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Position(#[from] PositionError),
    #[error("opening the config failed: {0}")]
    Open(#[from] tauri_plugin_opener::Error),
}

impl serde::Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// The window asks once on load; every later change arrives as an event.
#[tauri::command]
pub fn get_snapshot(state: State<'_, SnapshotState>) -> Result<Snapshot, CommandError> {
    state.0.lock().map(|snapshot| snapshot.clone()).map_err(|_| CommandError::Poisoned)
}

/// Opens `config.json` in the default editor, creating it from the template first.
#[tauri::command]
pub fn open_config(app: AppHandle, path: State<'_, ConfigPath>) -> Result<(), CommandError> {
    config::ensure_exists(&path.0)?;
    app.opener().open_path(path.0.to_string_lossy(), None::<&str>)?;
    Ok(())
}

/// The config is read once at start, so edited keys take effect after a restart.
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    app.restart();
}

/// Called by the window once a drag has come to rest.
#[tauri::command]
pub fn save_window_position(window: WebviewWindow, path: State<'_, PositionPath>) -> Result<(), CommandError> {
    window_position::save(&window, &path.0)?;
    Ok(())
}
