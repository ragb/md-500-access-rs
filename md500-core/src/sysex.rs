//! Roland SysEx framing for the MD-500.
//!
//! Frame layout (DT1, write):
//!
//! ```text
//! F0 41 [dev] 00 00 00 43 12 [a a a a] [d d d ...] [chk] F7
//! └ SOX                    └ DT1
//!    └ Roland      └ Model ID (4 bytes)
//!       └ Device ID (0x00..=0x1F, or 0x7F broadcast)
//! ```
//!
//! Checksum is the Roland standard: `(128 - ((sum of addr + data) mod 128)) mod 128`,
//! over the address + data section only (not F0, manufacturer, device, model, or
//! command bytes). This is exactly [`midi_access_core::codec::checksum`].
//!
//! Source: official "MD-500 MIDI Implementation" v1.00 (2017-07-01). Framing,
//! commands, model id, and the Identity Reply are quoted directly from §1–2.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SYSEX_START: u8 = 0xF0;
pub const SYSEX_END: u8 = 0xF7;
pub const ROLAND_ID: u8 = 0x41;

/// 4-byte Roland model ID for the MD-500.
pub const MD500_MODEL_ID: [u8; 4] = [0x00, 0x00, 0x00, 0x43];

/// Data Set 1 — write data to the device.
pub const CMD_DT1: u8 = 0x12;
/// Data Request 1 — ask the device to send data back.
pub const CMD_RQ1: u8 = 0x11;

/// Broadcast device id: every device replies regardless of its configured id.
pub const DEVICE_ID_BROADCAST: u8 = 0x7F;

/// Minimum frame length: F0 41 dev 00 00 00 43 cmd a a a a chk F7 = 14 bytes.
const MIN_FRAME_LEN: usize = 14;

/// Offset of the command byte: F0 41 dev [4-byte model] cmd … → index 7.
const CMD_INDEX: usize = 7;
/// Offset of the first address byte.
const ADDR_INDEX: usize = 8;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SysExError {
    #[error("frame too short ({0} bytes, need at least {MIN_FRAME_LEN})")]
    TooShort(usize),
    #[error("missing F0 start sentinel")]
    MissingStart,
    #[error("missing F7 end sentinel")]
    MissingEnd,
    #[error("not a Roland frame (manufacturer id {0:#04x})")]
    NotRoland(u8),
    #[error("invalid device id {0:#04x} (must be 0x00..=0x1F or 0x7F)")]
    InvalidDeviceId(u8),
    #[error("wrong model id (expected MD-500 {MD500_MODEL_ID:02x?})")]
    WrongModel,
    #[error("unknown command byte {0:#04x}")]
    UnknownCommand(u8),
    #[error("checksum mismatch: expected {expected:#04x}, got {actual:#04x}")]
    ChecksumMismatch { expected: u8, actual: u8 },
    #[error("data byte out of range (>= 0x80) at index {0}")]
    DataByteOutOfRange(usize),
}

/// `true` for a device id the MD-500 accepts (`0x00..=0x1F`, or the `0x7F`
/// broadcast id).
fn valid_device_id(dev: u8) -> bool {
    dev <= 0x1F || dev == DEVICE_ID_BROADCAST
}

/// Compute the Roland SysEx checksum over the address + data section.
pub fn checksum(addr_and_data: &[u8]) -> u8 {
    midi_access_core::codec::checksum(addr_and_data)
}

/// A decoded Roland SysEx frame addressed to / from the MD-500.
///
/// The `data` field is the raw payload between the address and the checksum.
/// For RQ1 frames it is a 4-byte "size" field; for DT1 frames it is the value(s)
/// being written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub device_id: u8,
    pub command: u8,
    pub address: [u8; 4],
    pub data: Vec<u8>,
}

impl Frame {
    /// Build a Data Set (DT1) frame. The recipient writes `data` to `address`.
    ///
    /// ```
    /// use md500_core::Frame;
    /// // Write Setup / current patch number = 0 (patch 01A) at 00 00 00 00.
    /// let frame = Frame::data_set(0x10, [0x00, 0x00, 0x00, 0x00], vec![0x00, 0x00, 0x00]);
    /// let bytes = frame.encode();
    /// assert_eq!(bytes[0], 0xF0);
    /// assert_eq!(&bytes[1..8], &[0x41, 0x10, 0x00, 0x00, 0x00, 0x43, 0x12]);
    /// assert_eq!(*bytes.last().unwrap(), 0xF7);
    /// ```
    pub fn data_set(device_id: u8, address: [u8; 4], data: Vec<u8>) -> Self {
        Self {
            device_id,
            command: CMD_DT1,
            address,
            data,
        }
    }

    /// Build a Data Request (RQ1) frame asking the device to send `size` bytes
    /// starting at `address`. `size` is encoded as 4 big-endian 7-bit-safe bytes.
    ///
    /// ```
    /// use md500_core::Frame;
    /// // Read the whole System Common area.
    /// let rq1 = Frame::data_request(0x10, [0x10, 0x00, 0x00, 0x00], [0, 0, 0, 0x22]);
    /// assert_eq!(rq1.encode().len(), 18);
    /// ```
    pub fn data_request(device_id: u8, address: [u8; 4], size: [u8; 4]) -> Self {
        Self {
            device_id,
            command: CMD_RQ1,
            address,
            data: size.to_vec(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(MIN_FRAME_LEN + self.data.len());
        buf.push(SYSEX_START);
        buf.push(ROLAND_ID);
        buf.push(self.device_id);
        buf.extend_from_slice(&MD500_MODEL_ID);
        buf.push(self.command);
        buf.extend_from_slice(&self.address);
        buf.extend_from_slice(&self.data);

        let mut cksum_input = Vec::with_capacity(4 + self.data.len());
        cksum_input.extend_from_slice(&self.address);
        cksum_input.extend_from_slice(&self.data);
        buf.push(checksum(&cksum_input));
        buf.push(SYSEX_END);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SysExError> {
        if bytes.len() < MIN_FRAME_LEN {
            return Err(SysExError::TooShort(bytes.len()));
        }
        if bytes[0] != SYSEX_START {
            return Err(SysExError::MissingStart);
        }
        if *bytes.last().unwrap() != SYSEX_END {
            return Err(SysExError::MissingEnd);
        }
        if bytes[1] != ROLAND_ID {
            return Err(SysExError::NotRoland(bytes[1]));
        }
        let device_id = bytes[2];
        if !valid_device_id(device_id) {
            return Err(SysExError::InvalidDeviceId(device_id));
        }
        if bytes[3..7] != MD500_MODEL_ID {
            return Err(SysExError::WrongModel);
        }
        let command = bytes[CMD_INDEX];
        if command != CMD_DT1 && command != CMD_RQ1 {
            return Err(SysExError::UnknownCommand(command));
        }
        let address: [u8; 4] = bytes[ADDR_INDEX..ADDR_INDEX + 4].try_into().unwrap();

        let payload_end = bytes.len() - 2; // exclude checksum + F7
        let data = bytes[ADDR_INDEX + 4..payload_end].to_vec();
        for (i, &b) in data.iter().enumerate() {
            if b >= 0x80 {
                return Err(SysExError::DataByteOutOfRange(i));
            }
        }

        let expected = bytes[payload_end];
        let mut cksum_input = Vec::with_capacity(4 + data.len());
        cksum_input.extend_from_slice(&address);
        cksum_input.extend_from_slice(&data);
        let actual = checksum(&cksum_input);
        if expected != actual {
            return Err(SysExError::ChecksumMismatch { expected, actual });
        }

        Ok(Self {
            device_id,
            command,
            address,
            data,
        })
    }
}

/// Build a Universal Identity Request: `F0 7E <dev> 06 01 F7`.
pub fn identity_request(device_id: u8) -> Vec<u8> {
    vec![0xF0, 0x7E, device_id, 0x06, 0x01, 0xF7]
}

/// If `bytes` is a Universal Identity Reply, return whether it identifies an
/// MD-500 (`Some(true)`), an unrecognised device (`Some(false)`), or — when the
/// message is not an identity reply at all — `None`.
///
/// The MD-500 reply carries `41 43 03 00 00` (Roland, family code `43 03`,
/// family number `00 00`); see the MIDI Implementation §2 Identity Reply.
pub fn identify(bytes: &[u8]) -> Option<bool> {
    let is_identity_reply = bytes.len() >= 6
        && bytes[0] == 0xF0
        && bytes[1] == 0x7E
        && bytes[3] == 0x06
        && bytes[4] == 0x02;
    if !is_identity_reply {
        return None;
    }
    let is_md500 = bytes.len() >= 8 && bytes[5] == 0x41 && bytes[6] == 0x43 && bytes[7] == 0x03;
    Some(is_md500)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_setup_current_patch() {
        // DT1 to Setup / current patch number (00 00 00 00) = 0 (3 nibble bytes).
        let f = Frame::data_set(0x10, [0x00, 0x00, 0x00, 0x00], vec![0x00, 0x00, 0x00]);
        let bytes = f.encode();
        assert_eq!(bytes[0], SYSEX_START);
        assert_eq!(bytes[1], ROLAND_ID);
        assert_eq!(bytes[2], 0x10);
        assert_eq!(&bytes[3..7], &MD500_MODEL_ID);
        assert_eq!(bytes[7], CMD_DT1);
        assert_eq!(&bytes[8..12], &[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(*bytes.last().unwrap(), SYSEX_END);
        // sum of addr+data = 0 → checksum 0.
        assert_eq!(bytes[bytes.len() - 2], 0x00);
    }

    #[test]
    fn checksum_matches_roland_spec() {
        // sum = 0x10 → checksum = 0x70.
        assert_eq!(checksum(&[0x10, 0x00, 0x00, 0x00, 0x00]), 0x70);
        assert_eq!(checksum(&[]), 0);
        assert_eq!(checksum(&[127]), 1);
        assert_eq!(checksum(&[64, 64]), 0);
    }

    #[test]
    fn round_trip_random_dt1() {
        let f = Frame::data_set(0x1F, [0x30, 0x10, 0x00, 0x00], vec![0x5A, 0x33, 0x7F, 0x00]);
        let bytes = f.encode();
        let decoded = Frame::decode(&bytes).unwrap();
        assert_eq!(f, decoded);
    }

    #[test]
    fn round_trip_rq1() {
        let f = Frame::data_request(0x10, [0x10, 0x00, 0x00, 0x00], [0x00, 0x00, 0x00, 0x22]);
        let bytes = f.encode();
        let decoded = Frame::decode(&bytes).unwrap();
        assert_eq!(decoded.command, CMD_RQ1);
        assert_eq!(decoded.data, vec![0x00, 0x00, 0x00, 0x22]);
        assert_eq!(f, decoded);
    }

    #[test]
    fn rq1_request_length_is_18() {
        // F0 41 dev 00 00 00 43 11 [4 addr] [4 size] chk F7 = 18.
        let rq1 = Frame::data_request(0x10, [0x10, 0x00, 0x00, 0x00], [0, 0, 0, 0x22]);
        assert_eq!(rq1.encode().len(), 18);
    }

    #[test]
    fn accepts_broadcast_device_id() {
        let f = Frame::data_request(DEVICE_ID_BROADCAST, [0x00, 0x00, 0x00, 0x00], [0, 0, 0, 1]);
        let bytes = f.encode();
        assert_eq!(
            Frame::decode(&bytes).unwrap().device_id,
            DEVICE_ID_BROADCAST
        );
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut bytes = Frame::data_set(0x10, [0x10, 0x00, 0x00, 0x00], vec![0x01]).encode();
        let last = bytes.len() - 2;
        bytes[last] ^= 0x01;
        assert!(matches!(
            Frame::decode(&bytes).unwrap_err(),
            SysExError::ChecksumMismatch { .. }
        ));
    }

    #[test]
    fn rejects_wrong_model() {
        let mut bytes = Frame::data_set(0x10, [0x10, 0x00, 0x00, 0x00], vec![0x01]).encode();
        bytes[6] = 0x18; // RE-202's model byte, not MD-500's 0x43
        assert_eq!(Frame::decode(&bytes).unwrap_err(), SysExError::WrongModel);
    }

    #[test]
    fn rejects_non_roland() {
        let mut bytes = Frame::data_set(0x10, [0x10, 0x00, 0x00, 0x00], vec![0x01]).encode();
        bytes[1] = 0x42; // Korg
        assert_eq!(
            Frame::decode(&bytes).unwrap_err(),
            SysExError::NotRoland(0x42)
        );
    }

    #[test]
    fn rejects_invalid_device_id() {
        let mut bytes = Frame::data_set(0x10, [0x10, 0x00, 0x00, 0x00], vec![0x01]).encode();
        bytes[2] = 0x20; // out of 0x00..=0x1F and not broadcast
        assert_eq!(
            Frame::decode(&bytes).unwrap_err(),
            SysExError::InvalidDeviceId(0x20)
        );
    }

    #[test]
    fn identity_round_trip() {
        assert_eq!(
            identity_request(0x7F),
            vec![0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7]
        );
        let reply = [
            0xF0, 0x7E, 0x10, 0x06, 0x02, 0x41, 0x43, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0xF7,
        ];
        assert_eq!(identify(&reply), Some(true));
        // Roland device, but a different family code.
        let other = [0xF0, 0x7E, 0x10, 0x06, 0x02, 0x41, 0x18, 0x04, 0xF7];
        assert_eq!(identify(&other), Some(false));
        // Not an identity reply.
        assert_eq!(identify(&[0x90, 0x40, 0x7F]), None);
    }
}
