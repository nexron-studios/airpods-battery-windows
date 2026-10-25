use tauri::State;

use crate::monitor::{Snapshot, SnapshotState};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("snapshot state is poisoned")]
    Poisoned,
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
