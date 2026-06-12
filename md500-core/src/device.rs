//! [`Device`] implementation for the BOSS MD-500 — the adapter that plugs the
//! MD-500's typed Roland-SysEx codec into the generic CLI engine
//! (`midi-access-cli`) and editor tooling, using `serde_yaml::Value` as the
//! document lingua franca.
//!
//! ## Areas
//!
//! - `setup`   — Setup Common (current patch number), `00 00 00 00`.
//! - `system`  — System Common **+** System Control, dumped/synced as two SysEx
//!   blocks (`10 00 00 00` and `10 00 10 00`).
//! - `pc-map`  — the 384-entry Program Change map, `20 00 00 00`.
//! - `bank`    — the Temporary bank's SIMUL parameters, `30 00 00 00`.
//! - `patch-a` / `patch-b` / `patch-c` — the Temporary bank's three patches
//!   (`30 00 10 00` / `…20…` / `…30…`).
//!
//! The 99 stored user banks are addressable from the wasm/editor layer
//! ([`crate::address::BankSlot`] / [`crate::address::PatchSlot`]); the CLI covers
//! the global areas and the live Temporary bank, mirroring the RE-202 split.

use serde_yaml::Value;

use midi_access_core::{split_sysex, Area, Catalogs, Device, DeviceError, Inbound, Params};

use crate::address::{
    add_offset, from_u28, size_field, to_u28, AddressSpace, BankSlot, PatchSlot,
    BANK_TEMPORARY_BASE, PC_MAP_BASE, SETUP_BASE, SYSTEM_BASE, SYSTEM_CONTROL_OFFSET,
};
use crate::bank::{BankParams, BANK_PARAMS_LEN};
use crate::patch::{Patch, PATCH_LEN};
use crate::pc_map::{ProgramChangeMap, PC_MAP_LEN};
use crate::setup::{Setup, SETUP_AREA_LEN};
use crate::sysex::{identify, Frame, CMD_DT1};
use crate::system::{System, SystemCommon, SystemControl, SYSTEM_COMMON_LEN, SYSTEM_CONTROL_LEN};

/// The BOSS MD-500 modulation pedal.
pub struct Md500;

/// One on-the-wire block: an absolute address and its byte length.
#[derive(Debug, Clone, Copy)]
struct Block {
    address: [u8; 4],
    len: usize,
}

/// What a CLI area maps to on the wire (one or more blocks).
enum Target {
    Setup,
    System,
    PcMap,
    Bank,
    Patch(PatchSlot),
}

const AREAS: &[Area] = &[
    Area {
        name: "setup",
        label: "Setup",
        about: "Setup Common — the currently-selected patch (3 bytes)",
    },
    Area {
        name: "system",
        label: "System",
        about: "Global system settings — System Common + System Control",
    },
    Area {
        name: "pc-map",
        label: "Program Change map",
        about: "384-entry MIDI Program Change → patch map",
    },
    Area {
        name: "bank",
        label: "Bank",
        about: "Temporary bank SIMUL parameters (4 bytes)",
    },
    Area {
        name: "patch-a",
        label: "Patch",
        about: "Temporary bank, PATCH A (586-byte modulation patch)",
    },
    Area {
        name: "patch-b",
        label: "Patch",
        about: "Temporary bank, PATCH B (586-byte modulation patch)",
    },
    Area {
        name: "patch-c",
        label: "Patch",
        about: "Temporary bank, PATCH C (586-byte modulation patch)",
    },
];

fn target_for(area: &str) -> Option<Target> {
    Some(match area {
        "setup" => Target::Setup,
        "system" => Target::System,
        "pc-map" => Target::PcMap,
        "bank" => Target::Bank,
        "patch-a" => Target::Patch(PatchSlot::A),
        "patch-b" => Target::Patch(PatchSlot::B),
        "patch-c" => Target::Patch(PatchSlot::C),
        _ => return None,
    })
}

impl Target {
    /// The wire blocks for this target, in send order.
    fn blocks(&self) -> Vec<Block> {
        match self {
            Target::Setup => vec![Block {
                address: SETUP_BASE,
                len: SETUP_AREA_LEN,
            }],
            Target::System => vec![
                Block {
                    address: SYSTEM_BASE,
                    len: SYSTEM_COMMON_LEN,
                },
                Block {
                    address: add_offset(SYSTEM_BASE, SYSTEM_CONTROL_OFFSET),
                    len: SYSTEM_CONTROL_LEN,
                },
            ],
            Target::PcMap => vec![Block {
                address: PC_MAP_BASE,
                len: PC_MAP_LEN,
            }],
            Target::Bank => vec![Block {
                address: BANK_TEMPORARY_BASE,
                len: BANK_PARAMS_LEN,
            }],
            Target::Patch(slot) => vec![Block {
                address: slot.base_address(BankSlot::Temporary),
                len: PATCH_LEN,
            }],
        }
    }
}

/// Wire device id used for RQ1/DT1.
///
/// The MD-500's device id spans `0x00..=0x1F` (UI 1–32) and defaults to `0x10`
/// on real units — outside the generic engine's `--device 0..=15` range, so a
/// channel-derived id can't reach a stock device. Roland gear answers a
/// **broadcast** (`0x7F`) request with its own id regardless of how it's set
/// (device-confirmed on the MD-500, 2026-06-12, and on the RE-202 before it), so
/// we address every frame to broadcast. This makes the CLI work with any single
/// MD-500 without configuring `--device`; targeting one unit among several
/// identical devices isn't supported (not needed for the editor's workflow).
fn device_id(_ch: u8) -> u8 {
    crate::sysex::DEVICE_ID_BROADCAST
}

/// Reassemble `expected` bytes of a block starting at `base` from a collected
/// dump stream. The MD-500 splits large regions (a patch, the PC map) into
/// several DT1 frames at successive addresses (device-confirmed: a 586-byte
/// patch arrives as 242 + 242 + 102), so we concatenate the data of every DT1
/// frame whose address chains contiguously from `base`.
fn collect_block(dump: &[u8], base: [u8; 4], expected: usize) -> Result<Vec<u8>, DeviceError> {
    let frames: Vec<Frame> = split_sysex(dump)
        .into_iter()
        .filter_map(|f| Frame::decode(&f).ok())
        .filter(|f| f.command == CMD_DT1)
        .collect();
    let mut out = Vec::with_capacity(expected);
    let mut addr = to_u28(base);
    while out.len() < expected {
        match frames.iter().find(|f| to_u28(f.address) == addr) {
            Some(f) => {
                out.extend_from_slice(&f.data);
                addr = addr.wrapping_add(f.data.len() as u32);
            }
            None => break,
        }
    }
    if out.len() < expected {
        return Err(DeviceError::Decode(format!(
            "incomplete dump at {base:02X?}: got {} of {expected} bytes",
            out.len()
        )));
    }
    out.truncate(expected);
    Ok(out)
}

fn dec(e: impl std::fmt::Display) -> DeviceError {
    DeviceError::Decode(e.to_string())
}
fn enc(e: impl std::fmt::Display) -> DeviceError {
    DeviceError::Encode(e.to_string())
}
fn unknown(area: &str) -> DeviceError {
    DeviceError::UnknownArea(area.to_string())
}

fn area_for_space(space: AddressSpace) -> Option<String> {
    Some(
        match space {
            AddressSpace::Setup => "setup",
            AddressSpace::System => "system",
            AddressSpace::PcMap => "pc-map",
            AddressSpace::Bank => return None, // could be any bank/patch; left unresolved
            AddressSpace::Unknown => return None,
        }
        .to_string(),
    )
}

impl Device for Md500 {
    const NAME: &'static str = "md500";

    fn areas() -> &'static [Area] {
        AREAS
    }

    fn params() -> Params {
        crate::catalog::params()
    }

    fn catalogs() -> &'static dyn Catalogs {
        &crate::catalog::MD500_CATALOGS
    }

    fn defaults(_area: &str) -> Option<Value> {
        None
    }

    fn schema(area: &str) -> Option<String> {
        #[cfg(feature = "schema")]
        {
            use midi_access_core::schema::schema_json;
            Some(match target_for(area)? {
                Target::Setup => schema_json::<Setup>(),
                Target::System => schema_json::<System>(),
                Target::PcMap => schema_json::<ProgramChangeMap>(),
                Target::Bank => schema_json::<BankParams>(),
                Target::Patch(_) => schema_json::<Patch>(),
            })
        }
        #[cfg(not(feature = "schema"))]
        {
            let _ = area;
            None
        }
    }

    fn request(area: &str, ch: u8) -> Result<Vec<u8>, DeviceError> {
        let id = device_id(ch);
        let mut out = Vec::new();
        for b in target_for(area).ok_or_else(|| unknown(area))?.blocks() {
            out.extend_from_slice(&Frame::data_request(id, b.address, size_field(b.len)).encode());
        }
        Ok(out)
    }

    fn decode(area: &str, dump: &[u8]) -> Result<Value, DeviceError> {
        match target_for(area).ok_or_else(|| unknown(area))? {
            Target::Setup => {
                let d = collect_block(dump, SETUP_BASE, SETUP_AREA_LEN)?;
                serde_yaml::to_value(Setup::from_bytes(&d).map_err(dec)?).map_err(dec)
            }
            Target::System => {
                let common = collect_block(dump, SYSTEM_BASE, SYSTEM_COMMON_LEN)?;
                let control = collect_block(
                    dump,
                    add_offset(SYSTEM_BASE, SYSTEM_CONTROL_OFFSET),
                    SYSTEM_CONTROL_LEN,
                )?;
                let system = System {
                    common: SystemCommon::from_bytes(&common).map_err(dec)?,
                    control: SystemControl::from_bytes(&control).map_err(dec)?,
                };
                serde_yaml::to_value(system).map_err(dec)
            }
            Target::PcMap => {
                let d = collect_block(dump, PC_MAP_BASE, PC_MAP_LEN)?;
                serde_yaml::to_value(ProgramChangeMap::from_bytes(&d).map_err(dec)?).map_err(dec)
            }
            Target::Bank => {
                let d = collect_block(dump, BANK_TEMPORARY_BASE, BANK_PARAMS_LEN)?;
                serde_yaml::to_value(BankParams::from_bytes(&d).map_err(dec)?).map_err(dec)
            }
            Target::Patch(slot) => {
                let d = collect_block(dump, slot.base_address(BankSlot::Temporary), PATCH_LEN)?;
                serde_yaml::to_value(Patch::from_bytes(&d).map_err(dec)?).map_err(dec)
            }
        }
    }

    fn encode(area: &str, doc: &Value, ch: u8) -> Result<Vec<u8>, DeviceError> {
        let id = device_id(ch);
        let target = target_for(area).ok_or_else(|| unknown(area))?;
        let mut out = Vec::new();
        match target {
            Target::Setup => {
                let s: Setup = serde_yaml::from_value(doc.clone()).map_err(enc)?;
                push_dt1(&mut out, id, SETUP_BASE, s.to_bytes().map_err(enc)?);
            }
            Target::System => {
                let s: System = serde_yaml::from_value(doc.clone()).map_err(enc)?;
                push_dt1(&mut out, id, SYSTEM_BASE, s.common.to_bytes().map_err(enc)?);
                push_dt1(
                    &mut out,
                    id,
                    add_offset(SYSTEM_BASE, SYSTEM_CONTROL_OFFSET),
                    s.control.to_bytes().map_err(enc)?,
                );
            }
            Target::PcMap => {
                let m: ProgramChangeMap = serde_yaml::from_value(doc.clone()).map_err(enc)?;
                push_dt1(&mut out, id, PC_MAP_BASE, m.to_bytes().map_err(enc)?);
            }
            Target::Bank => {
                let b: BankParams = serde_yaml::from_value(doc.clone()).map_err(enc)?;
                push_dt1(
                    &mut out,
                    id,
                    BANK_TEMPORARY_BASE,
                    b.to_bytes().map_err(enc)?,
                );
            }
            Target::Patch(slot) => {
                let p: Patch = serde_yaml::from_value(doc.clone()).map_err(enc)?;
                push_dt1(
                    &mut out,
                    id,
                    slot.base_address(BankSlot::Temporary),
                    p.to_bytes().map_err(enc)?,
                );
            }
        }
        Ok(out)
    }

    fn classify_inbound(bytes: &[u8]) -> Inbound {
        if let Some(is_md500) = identify(bytes) {
            return Inbound::Identity {
                bytes: bytes.to_vec(),
                model: is_md500.then(|| "MD-500".to_string()),
            };
        }
        use crate::inbound::{classify_inbound as core, InboundMessage};
        match core(bytes) {
            InboundMessage::DataSet {
                space,
                address,
                data,
                ..
            } => Inbound::Dump {
                area: area_for_space(space),
                address: address.to_vec(),
                data,
            },
            InboundMessage::DataRequest { address, .. } => Inbound::Request {
                address: address.to_vec(),
            },
            InboundMessage::UnparseableSysEx { bytes, .. } | InboundMessage::NonSysEx(bytes) => {
                Inbound::Other(bytes)
            }
        }
    }

    fn accepts(area: &str, doc: &Value) -> bool {
        let v = doc.clone();
        match target_for(area) {
            Some(Target::Setup) => serde_yaml::from_value::<Setup>(v).is_ok(),
            Some(Target::System) => serde_yaml::from_value::<System>(v).is_ok(),
            Some(Target::PcMap) => serde_yaml::from_value::<ProgramChangeMap>(v).is_ok(),
            Some(Target::Bank) => serde_yaml::from_value::<BankParams>(v).is_ok(),
            Some(Target::Patch(_)) => serde_yaml::from_value::<Patch>(v).is_ok(),
            None => false,
        }
    }
}

/// Largest DT1 data payload the MD-500 accepts in one frame. The device *sends*
/// regions in 242-byte chunks, and a single oversized DT1 is mis-parsed
/// (device-confirmed 2026-06-12: a 586-byte patch write corrupted a byte), so
/// writes are split the same way.
const MAX_DT1_DATA: usize = 242;

/// Append one or more DT1 frames writing `data` to `address`, splitting into
/// [`MAX_DT1_DATA`]-byte chunks at successive (7-bit) addresses.
fn push_dt1(out: &mut Vec<u8>, id: u8, address: [u8; 4], data: Vec<u8>) {
    let mut addr = to_u28(address);
    for chunk in data.chunks(MAX_DT1_DATA) {
        out.extend_from_slice(&Frame::data_set(id, from_u28(addr), chunk.to_vec()).encode());
        addr = addr.wrapping_add(chunk.len() as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(area: &str, dump_bytes: Vec<u8>) {
        // dump_bytes = the DT1 frames a device would send for this area.
        let doc = Md500::decode(area, &dump_bytes).unwrap();
        let encoded = Md500::encode(area, &doc, 0).unwrap();
        let doc2 = Md500::decode(area, &encoded).unwrap();
        assert_eq!(doc, doc2, "round trip for {area}");
    }

    fn dt1(address: [u8; 4], data: Vec<u8>) -> Vec<u8> {
        Frame::data_set(0x00, address, data).encode()
    }

    #[test]
    fn setup_round_trips_through_value() {
        round_trip("setup", dt1(SETUP_BASE, vec![0x00, 0x00, 0x00]));
    }

    #[test]
    fn system_round_trips_two_blocks() {
        let mut dump = dt1(SYSTEM_BASE, vec![0u8; SYSTEM_COMMON_LEN]);
        // control block: set tap function and a valid exp target.
        let mut control = vec![0u8; SYSTEM_CONTROL_LEN];
        control[0x08..0x0C].copy_from_slice(&crate::codec::write_nibbles(100, 4));
        control[0x0C..0x10].copy_from_slice(&crate::codec::write_nibbles(100, 4));
        dump.extend_from_slice(&dt1(
            add_offset(SYSTEM_BASE, SYSTEM_CONTROL_OFFSET),
            control,
        ));
        round_trip("system", dump);
    }

    #[test]
    fn patch_round_trips_through_value() {
        // A minimal valid patch block (assign active_range_hi must be >= 1).
        let mut data = vec![0u8; PATCH_LEN];
        for s in 0..8 {
            data[0x180 + s * 25 + 0x13] = 1;
        }
        round_trip(
            "patch-a",
            dt1(PatchSlot::A.base_address(BankSlot::Temporary), data),
        );
    }

    #[test]
    fn request_emits_two_frames_for_system() {
        let req = Md500::request("system", 0).unwrap();
        let frames = split_sysex(&req);
        assert_eq!(frames.len(), 2);
        // first frame requests System Common at 10 00 00 00.
        let f0 = Frame::decode(&frames[0]).unwrap();
        assert_eq!(f0.command, crate::sysex::CMD_RQ1);
        assert_eq!(f0.address, SYSTEM_BASE);
        let f1 = Frame::decode(&frames[1]).unwrap();
        assert_eq!(f1.address, add_offset(SYSTEM_BASE, SYSTEM_CONTROL_OFFSET));
    }

    #[test]
    fn request_uses_broadcast_device_id() {
        // The MD-500 is addressed by broadcast (see `device_id`), so the channel
        // value doesn't change the wire id.
        let req = Md500::request("setup", 5).unwrap();
        assert_eq!(
            Frame::decode(&split_sysex(&req)[0]).unwrap().device_id,
            crate::sysex::DEVICE_ID_BROADCAST
        );
    }

    #[test]
    fn identity_detects_md500() {
        let reply = [
            0xF0, 0x7E, 0x10, 0x06, 0x02, 0x41, 0x43, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0xF7,
        ];
        match Md500::classify_inbound(&reply) {
            Inbound::Identity { model, .. } => assert_eq!(model.as_deref(), Some("MD-500")),
            other => panic!("expected Identity, got {other:?}"),
        }
    }

    #[test]
    fn accepts_distinguishes_areas() {
        let setup_doc = Md500::decode("setup", &dt1(SETUP_BASE, vec![0, 0, 0])).unwrap();
        assert!(Md500::accepts("setup", &setup_doc));
        assert!(!Md500::accepts("bank", &setup_doc));
    }

    #[test]
    fn unknown_area_errors() {
        assert!(matches!(
            Md500::request("nope", 0),
            Err(DeviceError::UnknownArea(_))
        ));
    }
}
