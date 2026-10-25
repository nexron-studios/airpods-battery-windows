//! `config.json` in the app config dir: the AirPods keys and which Windows
//! devices to hide. Written by hand, so it is validated here and trusted after.

use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;

use crate::airpods::Key;

/// Written when the user asks to set up the keys and no config exists yet.
const TEMPLATE: &str = r#"{
  "magicAccIrk": "",
  "magicAccEncKey": "",
  "hiddenDevices": []
}
"#;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading or writing the config failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("config is not valid JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("{field} must contain exactly 32 hex digits")]
    InvalidKey { field: &'static str },
}

#[derive(Debug, Clone, Copy)]
pub struct AirPodsKeys {
    /// Byte order as the address check needs it, i.e. reversed from the keychain
    pub irk: Key,
    pub enc_key: Key,
}

#[derive(Debug, Default)]
pub struct Config {
    pub airpods_keys: Option<AirPodsKeys>,
    pub hidden_devices: HashSet<String>,
}

/// Field names and byte order exactly as the Mac keychain shows them, so the
/// values can be pasted without editing.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawConfig {
    #[serde(default)]
    magic_acc_irk: String,
    #[serde(default)]
    magic_acc_enc_key: String,
    #[serde(default)]
    hidden_devices: HashSet<String>,
}

/// Accepts the key with any separators the keychain or a copy adds:
/// `<0a1b2c3d 4e5f6071 ...>`, `0A-1B-...`, `0x0a1b...` all work.
fn parse_key(text: &str, field: &'static str) -> Result<Key, ConfigError> {
    let text = text.trim().trim_start_matches("0x");
    let digits: Vec<u8> = text.chars().filter_map(|c| c.to_digit(16)).map(|d| d as u8).collect();
    let has_only_separators = text.chars().all(|c| c.is_ascii_hexdigit() || " <>-:\t".contains(c));
    if digits.len() != 2 * size_of::<Key>() || !has_only_separators {
        return Err(ConfigError::InvalidKey { field });
    }
    let mut key = Key::default();
    for (byte, pair) in key.iter_mut().zip(digits.chunks(2)) {
        *byte = pair[0] << 4 | pair[1];
    }
    Ok(key)
}

fn parse(raw: RawConfig) -> Result<Config, ConfigError> {
    let has_keys = !raw.magic_acc_irk.trim().is_empty() && !raw.magic_acc_enc_key.trim().is_empty();
    let airpods_keys = if has_keys {
        let mut irk = parse_key(&raw.magic_acc_irk, "magicAccIrk")?;
        // The keychain stores the IRK little-endian; the address check needs it
        // the other way round. The EncKey is used as stored.
        irk.reverse();
        Some(AirPodsKeys { irk, enc_key: parse_key(&raw.magic_acc_enc_key, "magicAccEncKey")? })
    } else {
        None
    };
    Ok(Config { airpods_keys, hidden_devices: raw.hidden_devices })
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    match fs::read_to_string(path) {
        Ok(text) => parse(serde_json::from_str(&text)?),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(error.into()),
    }
}

/// Creates the config from the template unless it already exists.
pub fn ensure_exists(path: &Path) -> Result<(), ConfigError> {
    if path.exists() {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, TEMPLATE)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEQUENTIAL: Key = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255];

    #[test]
    fn parses_a_plain_hex_key() {
        assert_eq!(parse_key("000102030405060708090a0b0c0d0eFF", "irk").unwrap(), SEQUENTIAL);
    }

    #[test]
    fn parses_a_key_as_the_keychain_shows_it() {
        assert_eq!(parse_key("<00010203 04050607 08090a0b 0c0d0eff>", "irk").unwrap(), SEQUENTIAL);
    }

    #[test]
    fn parses_a_dashed_key() {
        let dashed = "00-01-02-03-04-05-06-07-08-09-0A-0B-0C-0D-0E-FF";

        assert_eq!(parse_key(dashed, "irk").unwrap(), SEQUENTIAL);
    }

    #[test]
    fn rejects_a_short_key() {
        assert!(matches!(parse_key("0001", "irk"), Err(ConfigError::InvalidKey { field: "irk" })));
    }

    #[test]
    fn rejects_letters_that_are_not_hex() {
        let result = parse_key("zz0102030405060708090a0b0c0d0e0f", "encKey");

        assert!(matches!(result, Err(ConfigError::InvalidKey { field: "encKey" })));
    }

    #[test]
    fn reverses_the_irk_but_not_the_enc_key() {
        let raw = RawConfig {
            magic_acc_irk: "000102030405060708090a0b0c0d0eff".into(),
            magic_acc_enc_key: "000102030405060708090a0b0c0d0eff".into(),
            hidden_devices: HashSet::new(),
        };

        let keys = parse(raw).unwrap().airpods_keys.unwrap();

        assert_eq!(keys.irk, [255, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
        assert_eq!(keys.enc_key, SEQUENTIAL);
    }

    #[test]
    fn treats_the_empty_template_as_no_keys() {
        let config = parse(serde_json::from_str(TEMPLATE).unwrap()).unwrap();

        assert!(config.airpods_keys.is_none());
    }
}
