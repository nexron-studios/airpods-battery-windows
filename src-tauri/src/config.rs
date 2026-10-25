//! `config.json` in the app config dir: the AirPods keys and which Windows
//! devices to hide. Written by hand, so it is validated here and trusted after.

use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;

use crate::airpods::Key;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading the config failed: {0}")]
    Read(#[from] std::io::Error),
    #[error("config is not valid JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("{field} must be 32 hex characters")]
    InvalidKey { field: &'static str },
}

#[derive(Debug, Clone, Copy)]
pub struct AirPodsKeys {
    /// Byte order as the advert check needs it; the Mac keychain stores it reversed.
    pub irk: Key,
    pub enc_key: Key,
}

#[derive(Debug, Default)]
pub struct Config {
    pub airpods_keys: Option<AirPodsKeys>,
    pub hidden_devices: HashSet<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawConfig {
    irk: Option<String>,
    enc_key: Option<String>,
    #[serde(default)]
    hidden_devices: HashSet<String>,
}

fn parse_key(hex: &str, field: &'static str) -> Result<Key, ConfigError> {
    let invalid = || ConfigError::InvalidKey { field };
    let digits = hex.as_bytes();
    if digits.len() != 2 * size_of::<Key>() {
        return Err(invalid());
    }
    let mut key = Key::default();
    for (byte, pair) in key.iter_mut().zip(digits.chunks(2)) {
        let pair = std::str::from_utf8(pair).map_err(|_| invalid())?;
        *byte = u8::from_str_radix(pair, 16).map_err(|_| invalid())?;
    }
    Ok(key)
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let raw: RawConfig = match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)?,
        Err(error) if error.kind() == ErrorKind::NotFound => RawConfig::default(),
        Err(error) => return Err(error.into()),
    };
    let airpods_keys = match (raw.irk, raw.enc_key) {
        (Some(irk), Some(enc_key)) => Some(AirPodsKeys {
            irk: parse_key(&irk, "irk")?,
            enc_key: parse_key(&enc_key, "encKey")?,
        }),
        _ => None,
    };
    Ok(Config { airpods_keys, hidden_devices: raw.hidden_devices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_hex_key() {
        let key = parse_key("000102030405060708090a0b0c0d0eFF", "irk").unwrap();

        assert_eq!(key, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255]);
    }

    #[test]
    fn rejects_a_short_key() {
        assert!(matches!(parse_key("0001", "irk"), Err(ConfigError::InvalidKey { field: "irk" })));
    }

    #[test]
    fn rejects_non_hex_characters() {
        let result = parse_key("zz0102030405060708090a0b0c0d0e0f", "encKey");

        assert!(matches!(result, Err(ConfigError::InvalidKey { field: "encKey" })));
    }
}
