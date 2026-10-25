//! Remembers where the widget was dragged to.
//!
//! Replaces tauri-plugin-window-state, which restored the position scaled by
//! the monitor's DPI factor on mixed-DPI setups and pushed the widget off-screen
//! a little further on every start. Everything here stays in physical pixels.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{Monitor, PhysicalPosition, WebviewWindow};

/// How much of the widget must lie on a monitor for the saved spot to count.
const MIN_VISIBLE_PX: i32 = 40;

pub struct PositionPath(pub PathBuf);

#[derive(Debug, thiserror::Error)]
pub enum PositionError {
    #[error("reading or writing the position failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("saved position is not valid JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("window call failed: {0}")]
    Window(#[from] tauri::Error),
}

#[derive(Serialize, Deserialize)]
struct SavedPosition {
    x: i32,
    y: i32,
}

fn is_on(monitor: &Monitor, position: &SavedPosition) -> bool {
    let (origin, size) = (monitor.position(), monitor.size());
    let right = origin.x + size.width as i32;
    let bottom = origin.y + size.height as i32;
    position.x + MIN_VISIBLE_PX > origin.x
        && position.x + MIN_VISIBLE_PX < right
        && position.y + MIN_VISIBLE_PX > origin.y
        && position.y + MIN_VISIBLE_PX < bottom
}

/// Moves the window to the saved spot, unless that monitor is gone; then the
/// window keeps its default place instead of opening somewhere invisible.
pub fn restore(window: &WebviewWindow, path: &Path) -> Result<(), PositionError> {
    let saved: SavedPosition = match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)?,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !window.available_monitors()?.iter().any(|monitor| is_on(monitor, &saved)) {
        log::info!("saved position {},{} is on no monitor, using the default", saved.x, saved.y);
        return Ok(());
    }
    window.set_position(PhysicalPosition::new(saved.x, saved.y))?;
    Ok(())
}

pub fn save(window: &WebviewWindow, path: &Path) -> Result<(), PositionError> {
    let position = window.outer_position()?;
    let saved = SavedPosition { x: position.x, y: position.y };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, serde_json::to_string(&saved)?)?;
    Ok(())
}
