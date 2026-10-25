//! Decode AirPods battery levels from Apple proximity-pairing BLE adverts.
//!
//! Current AirPods firmware zeroes the plaintext battery nibbles. The real levels
//! sit in the last 16 bytes, AES-128 encrypted with the pairing's MagicAccEncKey
//! (layout taken from LibrePods, linux/ble/blemanager.cpp and linux/battery.hpp).

use aes::Aes128;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use serde::Serialize;

pub const APPLE_COMPANY_ID: u16 = 0x004C;
pub type Key = [u8; 16];

const PROXIMITY_PAIRING_TYPE: u8 = 0x07;
const MIN_ADVERT_LEN: usize = 27;
const STATUS_INDEX: usize = 5;
const ENCRYPTED_PAYLOAD_LEN: usize = 16;
const STATUS_LEFT_IS_PRIMARY: u8 = 0x20;
const CHARGING_BIT: u8 = 0x80;
const LEVEL_MASK: u8 = 0x7F;
const MAX_LEVEL: u8 = 100;
const RPA_PART_LEN: usize = 3;
const AES_BLOCK_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub percent: u8,
    pub is_charging: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct AirPodsBattery {
    pub left: Option<Level>,
    pub right: Option<Level>,
    pub case: Option<Level>,
}

impl AirPodsBattery {
    /// Keep the last known value for every part the new advert does not report.
    pub fn merge(self, newer: AirPodsBattery) -> AirPodsBattery {
        AirPodsBattery {
            left: newer.left.or(self.left),
            right: newer.right.or(self.right),
            case: newer.case.or(self.case),
        }
    }
}

/// When each part last reported a level, in ms since the epoch. A pod lying in
/// the closed case goes silent while the other one keeps reporting, so every
/// part ages on its own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct PartsSeen {
    pub left: Option<u64>,
    pub right: Option<u64>,
    pub case: Option<u64>,
}

impl PartsSeen {
    pub fn record(self, reported: &AirPodsBattery, now_ms: u64) -> PartsSeen {
        let stamp = |level: Option<Level>, previous: Option<u64>| level.map(|_| now_ms).or(previous);
        PartsSeen {
            left: stamp(reported.left, self.left),
            right: stamp(reported.right, self.right),
            case: stamp(reported.case, self.case),
        }
    }
}

fn aes_block(key: &Key, data: &[u8], is_decrypt: bool) -> [u8; AES_BLOCK_LEN] {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut block = GenericArray::clone_from_slice(data);
    if is_decrypt {
        cipher.decrypt_block(&mut block);
    } else {
        cipher.encrypt_block(&mut block);
    }
    block.into()
}

/// Resolve a resolvable private address against an IRK (BT Core Vol 3 Part H 2.2.2).
/// `address` is the 48-bit Bluetooth address as Windows reports it.
pub fn is_own_address(address: u64, irk: &Key) -> bool {
    let raw = &address.to_be_bytes()[2..];
    let (prand, address_hash) = raw.split_at(RPA_PART_LEN);
    let mut padded = [0u8; AES_BLOCK_LEN];
    padded[AES_BLOCK_LEN - RPA_PART_LEN..].copy_from_slice(prand);
    aes_block(irk, &padded, false)[AES_BLOCK_LEN - RPA_PART_LEN..] == *address_hash
}

fn level(raw: u8) -> Option<Level> {
    let percent = raw & LEVEL_MASK;
    // 0x7F means "not reported", e.g. the case while it is closed
    if percent > MAX_LEVEL {
        return None;
    }
    Some(Level { percent, is_charging: raw & CHARGING_BIT != 0 })
}

/// `data` is the Apple manufacturer data without the company id.
pub fn parse_advert(data: &[u8], enc_key: &Key) -> Option<AirPodsBattery> {
    if data.len() < MIN_ADVERT_LEN || data[0] != PROXIMITY_PAIRING_TYPE {
        return None;
    }

    let status = data[STATUS_INDEX];
    let plain = aes_block(enc_key, &data[data.len() - ENCRYPTED_PAYLOAD_LEN..], true);
    let (primary, secondary) = (level(plain[1]), level(plain[2]));
    let (left, right) = if status & STATUS_LEFT_IS_PRIMARY != 0 {
        (primary, secondary)
    } else {
        (secondary, primary)
    };
    // No status bit gates the case level: while charging, AirPods 4 send status
    // 0x04 without the "this pod in case" bit (0x40) and still a valid case
    // level. An unreported level is 0x7F, which `level` already drops.
    Some(AirPodsBattery { left, right, case: level(plain[3]) })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: Key = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    const PLAINTEXT_HEADER: [u8; 5] = [0x07, 0x19, 0x01, 0x1b, 0x20];
    // Captured from the real AirPods 4: plaintext battery byte is 00 on current firmware
    const PLAINTEXT_TAIL: [u8; 5] = [0x00, 0x8f, 0x11, 0x00, 0x05];

    fn advert(status: u8, levels: [u8; 4]) -> Vec<u8> {
        let mut decrypted = [0u8; AES_BLOCK_LEN];
        decrypted[..4].copy_from_slice(&levels);
        let mut data = PLAINTEXT_HEADER.to_vec();
        data.push(status);
        data.extend_from_slice(&PLAINTEXT_TAIL);
        data.extend_from_slice(&aes_block(&KEY, &decrypted, false));
        data
    }

    const fn lvl(percent: u8, is_charging: bool) -> Option<Level> {
        Some(Level { percent, is_charging })
    }

    #[test]
    fn left_primary_in_case_reports_all_three() {
        let battery = parse_advert(&advert(0x20 | 0x40, [0, 80, 0x80 | 75, 50]), &KEY);

        assert_eq!(
            battery,
            Some(AirPodsBattery { left: lvl(80, false), right: lvl(75, true), case: lvl(50, false) })
        );
    }

    #[test]
    fn right_primary_swaps_pods() {
        let battery = parse_advert(&advert(0x0B, [0, 80, 75, 0x7F]), &KEY);

        assert_eq!(battery, Some(AirPodsBattery { left: lvl(75, false), right: lvl(80, false), case: None }));
    }

    #[test]
    fn reads_the_case_while_charging_without_the_in_case_bit() {
        // Captured from AirPods 4 charging in the open case: status 0x04, one pod
        // at 26 % charging, the other unreported, case at 92 %
        let battery = parse_advert(&advert(0x04, [0x00, 0x9a, 0xff, 0x5c]), &KEY);

        assert_eq!(battery, Some(AirPodsBattery { left: None, right: lvl(26, true), case: lvl(92, false) }));
    }

    #[test]
    fn unreported_level_is_none() {
        let battery = parse_advert(&advert(0x20 | 0x40, [0, 0x7F, 60, 0x7F]), &KEY);

        assert_eq!(battery, Some(AirPodsBattery { left: None, right: lvl(60, false), case: None }));
    }

    #[test]
    fn non_proximity_advert_is_ignored() {
        let mut data = vec![0x10, 0x05];
        data.extend_from_slice(&[0u8; 25]);

        assert_eq!(parse_advert(&data, &KEY), None);
    }

    #[test]
    fn merge_keeps_parts_the_newer_advert_does_not_report() {
        let older = AirPodsBattery { left: lvl(40, false), right: lvl(41, false), case: lvl(90, true) };
        let newer = AirPodsBattery { left: lvl(39, false), right: None, case: None };

        assert_eq!(
            older.merge(newer),
            AirPodsBattery { left: lvl(39, false), right: lvl(41, false), case: lvl(90, true) }
        );
    }

    #[test]
    fn record_only_stamps_the_parts_an_advert_reported() {
        let earlier = PartsSeen { left: Some(1_000), right: Some(1_000), case: None };
        let reported = AirPodsBattery { left: None, right: lvl(24, false), case: None };

        assert_eq!(earlier.record(&reported, 5_000), PartsSeen { left: Some(1_000), right: Some(5_000), case: None });
    }

    #[test]
    fn resolves_core_spec_sample_address() {
        // BT Core Vol 3 Part H Appendix D.7: k=ec02..7d9b, prand=708194 -> hash=0dfbaa
        let irk: Key = [
            0xec, 0x02, 0x34, 0xa3, 0x57, 0xc8, 0xad, 0x05, 0x34, 0x10, 0x10, 0xa6, 0x0a, 0x39, 0x7d, 0x9b,
        ];

        assert!(is_own_address(0x7081_940D_FBAA, &irk));
        assert!(!is_own_address(0x7081_940D_FBAB, &irk));
    }
}
