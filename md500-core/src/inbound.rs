//! Classify any byte sequence that arrived from the device.
//!
//! Owning this routing in one function keeps callers from having to know about
//! command bytes and address prefixes (the ml10x/re202 lesson).

use crate::address::AddressSpace;
use crate::sysex::{Frame, SysExError, CMD_DT1, CMD_RQ1, SYSEX_START};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundMessage {
    /// A well-formed DT1 frame, tagged with the area its address falls in.
    DataSet {
        device_id: u8,
        space: AddressSpace,
        address: [u8; 4],
        data: Vec<u8>,
    },
    /// A well-formed RQ1 frame (we don't normally receive these, but support it).
    DataRequest {
        device_id: u8,
        address: [u8; 4],
        size: Vec<u8>,
    },
    /// SysEx we couldn't decode as a Roland MD-500 frame.
    UnparseableSysEx { bytes: Vec<u8>, error: SysExError },
    /// Bytes that weren't a SysEx frame at all (channel messages, etc.).
    NonSysEx(Vec<u8>),
}

pub fn classify_inbound(bytes: &[u8]) -> InboundMessage {
    if bytes.first() != Some(&SYSEX_START) {
        return InboundMessage::NonSysEx(bytes.to_vec());
    }
    match Frame::decode(bytes) {
        Ok(frame) => match frame.command {
            CMD_DT1 => InboundMessage::DataSet {
                device_id: frame.device_id,
                space: AddressSpace::classify(frame.address),
                address: frame.address,
                data: frame.data,
            },
            CMD_RQ1 => InboundMessage::DataRequest {
                device_id: frame.device_id,
                address: frame.address,
                size: frame.data,
            },
            _ => unreachable!("Frame::decode rejected unknown commands"),
        },
        Err(error) => InboundMessage::UnparseableSysEx {
            bytes: bytes.to_vec(),
            error,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysex::Frame;

    #[test]
    fn classifies_system_dt1() {
        let bytes = Frame::data_set(0x10, [0x10, 0x00, 0x00, 0x00], vec![0x00]).encode();
        match classify_inbound(&bytes) {
            InboundMessage::DataSet { space, address, .. } => {
                assert_eq!(space, AddressSpace::System);
                assert_eq!(address, [0x10, 0x00, 0x00, 0x00]);
            }
            other => panic!("expected DataSet, got {other:?}"),
        }
    }

    #[test]
    fn classifies_non_sysex_and_unparseable() {
        assert!(matches!(
            classify_inbound(&[0x90, 0x40, 0x7F]),
            InboundMessage::NonSysEx(_)
        ));
        assert!(matches!(
            classify_inbound(&[0xF0, 0x42, 0x00, 0xF7]),
            InboundMessage::UnparseableSysEx { .. }
        ));
    }
}
