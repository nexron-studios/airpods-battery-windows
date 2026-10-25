//! Battery levels Windows already knows for paired Bluetooth LE devices, e.g. the iPhone.

use serde::Serialize;
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows_collections::IIterable;
use windows::Foundation::IReference;
use windows::core::{HSTRING, Interface};

const BATTERY_PROPERTY: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";
const BLE_DEVICES_FILTER: &str = r#"System.Devices.DeviceInstanceId:~<"BTHLE\DEV_""#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceBattery {
    pub name: String,
    pub percent: u8,
}

pub fn read() -> windows::core::Result<Vec<DeviceBattery>> {
    let key = HSTRING::from(BATTERY_PROPERTY);
    let properties = IIterable::<HSTRING>::from(vec![key.clone()]);
    let devices = DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(
        &HSTRING::from(BLE_DEVICES_FILTER),
        &properties,
        DeviceInformationKind::Device,
    )?
    .join()?;

    let mut batteries = Vec::new();
    for device in devices {
        let value = match device.Properties()?.Lookup(&key) {
            Ok(value) => value,
            // Devices without a battery service carry the key with a null value,
            // which windows-rs reports as an error with a success code
            Err(error) if error.code().is_ok() => continue,
            Err(error) => return Err(error),
        };
        let percent = value.cast::<IReference<u8>>()?.Value()?;
        batteries.push(DeviceBattery { name: device.Name()?.to_string(), percent });
    }
    Ok(batteries)
}

