# Battery Widget

Small always-on-top desktop widget for Windows: AirPods (left, right, case) and
every Bluetooth device whose battery Windows knows, e.g. the iPhone.

Tauri 2 (Rust) + Vue 3 + Tailwind 4.

## How it reads the levels

- **AirPods:** current firmware zeroes the plaintext battery byte of the BLE
  proximity-pairing advert. The real levels sit in the last 16 bytes, AES-128
  encrypted with the pairing's `MagicAccEncKey`. `src-tauri/src/airpods.rs`
  decrypts them and filters out foreign AirPods with the `MagicAccIRK`.
- **Everything else:** the PnP property `{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2`
  on the `BTHLE\DEV_*` device nodes (`src-tauri/src/device_battery.rs`).

The case only reports while at least one pod sits in it.

## Config

`%APPDATA%\com.nexron.batterywidget\config.json`:

```json
{
  "irk": "<MagicAccIRK, byte-reversed as the Mac keychain stores it reversed>",
  "encKey": "<MagicAccEncKey as stored>",
  "hiddenDevices": ["MX KEYS S"]
}
```

Logs: `%LOCALAPPDATA%\com.nexron.batterywidget\logs\`.

## Develop

```sh
npm install
npm run tauri dev     # Vite on port 1440
npm test              # Vitest
cd src-tauri && cargo test
npm run tauri build   # NSIS installer in src-tauri/target/release/bundle/nsis
```
