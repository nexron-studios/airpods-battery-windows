//! Keeps the current battery snapshot and pushes it to the window on change.

use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::airpods::{self, AirPodsBattery};
use crate::ble_scanner::{AppleAdvert, BleScanner};
use crate::config::{AirPodsKeys, Config};
use crate::device_battery::{self, DeviceBattery};

pub const SNAPSHOT_EVENT: &str = "snapshot-changed";
const DEVICE_POLL_INTERVAL: Duration = Duration::from_secs(60);
/// AirPods advertise several times a second; an unchanged level only refreshes
/// "last seen" in the window this often.
const SEEN_REFRESH_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum AirPodsStatus {
    MissingKeys,
    /// Bluetooth is off or the adapter is gone
    Unavailable,
    Waiting,
    Seen { battery: AirPodsBattery, last_seen_ms: u64 },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub airpods: AirPodsStatus,
    pub devices: Vec<DeviceBattery>,
}

pub struct SnapshotState(pub Mutex<Snapshot>);

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as u64)
}

/// Applies `change` and emits the result if it changed. The lock is released
/// before emitting, so a slow window never blocks the scanner thread.
fn update(app: &AppHandle, change: impl FnOnce(&mut Snapshot) -> bool) {
    let state = app.state::<SnapshotState>();
    let snapshot = {
        let Ok(mut snapshot) = state.0.lock() else {
            log::error!("snapshot lock poisoned, dropping update");
            return;
        };
        if !change(&mut snapshot) {
            return;
        }
        snapshot.clone()
    };
    if let Err(error) = app.emit(SNAPSHOT_EVENT, snapshot) {
        log::error!("emitting the snapshot failed: {error}");
    }
}

fn handle_advert(app: &AppHandle, keys: &AirPodsKeys, advert: AppleAdvert) {
    let Some(battery) = airpods::parse_advert(&advert.data, &keys.enc_key) else { return };
    // The IRK check filters out other people's AirPods nearby
    if !airpods::is_own_address(advert.address, &keys.irk) {
        return;
    }
    let now = now_ms();
    update(app, |snapshot| {
        let (merged, is_due) = match snapshot.airpods {
            AirPodsStatus::Seen { battery: known, last_seen_ms } => {
                let merged = known.merge(battery);
                (merged, merged != known || now - last_seen_ms >= SEEN_REFRESH_MS)
            }
            _ => (battery, true),
        };
        if is_due {
            snapshot.airpods = AirPodsStatus::Seen { battery: merged, last_seen_ms: now };
        }
        is_due
    });
}

/// Once the scan runs again the window waits for the next advert instead of
/// still reporting Bluetooth as off.
fn clear_unavailable(app: &AppHandle) {
    update(app, |snapshot| {
        let is_unavailable = snapshot.airpods == AirPodsStatus::Unavailable;
        if is_unavailable {
            snapshot.airpods = AirPodsStatus::Waiting;
        }
        is_unavailable
    });
}

fn set_airpods_status(app: &AppHandle, status: AirPodsStatus) {
    update(app, |snapshot| {
        let is_changed = snapshot.airpods != status;
        snapshot.airpods = status;
        is_changed
    });
}

fn start_scanner(app: &AppHandle, keys: AirPodsKeys) -> Option<BleScanner> {
    let handle = app.clone();
    match BleScanner::start(move |advert| handle_advert(&handle, &keys, advert)) {
        Ok(scanner) => {
            clear_unavailable(app);
            Some(scanner)
        }
        Err(error) => {
            log::error!("starting the Bluetooth scan failed: {error}");
            set_airpods_status(app, AirPodsStatus::Unavailable);
            None
        }
    }
}

fn keep_scanner_alive(app: &AppHandle, scanner: &mut Option<BleScanner>, keys: AirPodsKeys) {
    let Some(running) = scanner else {
        *scanner = start_scanner(app, keys);
        return;
    };
    match running.ensure_running() {
        Ok(true) => {}
        Ok(false) => {
            log::info!("Bluetooth scan had stopped, restarted it");
            clear_unavailable(app);
        }
        Err(error) => {
            log::warn!("Bluetooth scan is down: {error}");
            set_airpods_status(app, AirPodsStatus::Unavailable);
        }
    }
}

fn poll_devices(app: &AppHandle, config: &Config) {
    let devices = match device_battery::read() {
        Ok(devices) => devices,
        Err(error) => return log::error!("reading Windows battery levels failed: {error}"),
    };
    let visible: Vec<_> = devices.into_iter().filter(|device| !config.hidden_devices.contains(&device.name)).collect();
    update(app, |snapshot| {
        let is_changed = snapshot.devices != visible;
        snapshot.devices = visible;
        is_changed
    });
}

pub fn start(app: &AppHandle, config: Config) {
    let airpods = match config.airpods_keys {
        Some(_) => AirPodsStatus::Waiting,
        None => AirPodsStatus::MissingKeys,
    };
    app.manage(SnapshotState(Mutex::new(Snapshot { airpods, devices: Vec::new() })));

    let app = app.clone();
    thread::spawn(move || {
        let mut scanner = None;
        loop {
            if let Some(keys) = config.airpods_keys {
                keep_scanner_alive(&app, &mut scanner, keys);
            }
            poll_devices(&app, &config);
            thread::sleep(DEVICE_POLL_INTERVAL);
        }
    });
}
