//! Listens for Apple BLE adverts through the WinRT advertisement watcher.

use windows::Devices::Bluetooth::Advertisement::{
    BluetoothLEAdvertisementReceivedEventArgs, BluetoothLEAdvertisementWatcher,
    BluetoothLEAdvertisementWatcherStatus, BluetoothLEScanningMode,
};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::DataReader;
use windows::core::Ref;

use crate::airpods::APPLE_COMPANY_ID;

pub struct AppleAdvert {
    pub address: u64,
    pub data: Vec<u8>,
}

/// Owns the watcher; dropping it stops the scan.
pub struct BleScanner {
    watcher: BluetoothLEAdvertisementWatcher,
}

fn apple_data(args: &BluetoothLEAdvertisementReceivedEventArgs) -> windows::core::Result<Option<Vec<u8>>> {
    for section in args.Advertisement()?.ManufacturerData()? {
        if section.CompanyId()? != APPLE_COMPANY_ID {
            continue;
        }
        let buffer = section.Data()?;
        let mut data = vec![0u8; buffer.Length()? as usize];
        DataReader::FromBuffer(&buffer)?.ReadBytes(&mut data)?;
        return Ok(Some(data));
    }
    Ok(None)
}

impl BleScanner {
    pub fn start(on_advert: impl Fn(AppleAdvert) + Send + 'static) -> windows::core::Result<Self> {
        let watcher = BluetoothLEAdvertisementWatcher::new()?;
        watcher.SetScanningMode(BluetoothLEScanningMode::Active)?;
        watcher.Received(&TypedEventHandler::new(
            move |_, args: Ref<BluetoothLEAdvertisementReceivedEventArgs>| {
                let Some(args) = args.as_ref() else { return Ok(()) };
                if let Some(data) = apple_data(args)? {
                    on_advert(AppleAdvert { address: args.BluetoothAddress()?, data });
                }
                Ok(())
            },
        ))?;
        watcher.Start()?;
        Ok(Self { watcher })
    }

    /// The watcher aborts when Bluetooth is switched off and never resumes on
    /// its own, so the caller restarts it periodically.
    pub fn ensure_running(&self) -> windows::core::Result<bool> {
        if self.watcher.Status()? == BluetoothLEAdvertisementWatcherStatus::Started {
            return Ok(true);
        }
        self.watcher.Start()?;
        Ok(false)
    }
}
