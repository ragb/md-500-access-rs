//! WASM bindings — exposes `md500-core` to JavaScript / TypeScript.
//!
//! Typed encode/decode functions accept and return the typed core models (Setup
//! / System / ProgramChangeMap / BankParams / Patch), which render in the
//! generated `.d.ts` thanks to the `tsify` derives on those types. Address
//! helpers expose all 99 user banks (the CLI only reaches the Temporary bank).

use md500_core::address::{
    size_field, BankSlot, PatchSlot, BANK_MAX, BANK_TEMPORARY_BASE, PC_MAP_BASE, SETUP_BASE,
    SYSTEM_BASE, SYSTEM_CONTROL_OFFSET,
};
use md500_core::bank::{BankParams, BANK_PARAMS_LEN};
use md500_core::patch::{Patch, PATCH_LEN};
use md500_core::pc_map::{ProgramChangeMap, PC_MAP_LEN};
use md500_core::setup::{Setup, SETUP_AREA_LEN};
use md500_core::system::{SystemCommon, SystemControl, SYSTEM_COMMON_LEN, SYSTEM_CONTROL_LEN};
use md500_core::{
    classify_inbound as core_classify, identify, identity_request, Frame, InboundMessage,
};

use serde::Serialize;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

fn js_err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

// === Raw frame codec ===

#[wasm_bindgen(js_name = encodeFrame)]
pub fn encode_frame(frame: JsValue) -> Result<Vec<u8>, JsError> {
    let frame: Frame = serde_wasm_bindgen::from_value(frame).map_err(js_err)?;
    Ok(frame.encode())
}

#[wasm_bindgen(js_name = decodeFrame)]
pub fn decode_frame(bytes: &[u8]) -> Result<JsValue, JsError> {
    let frame = Frame::decode(bytes).map_err(js_err)?;
    serde_wasm_bindgen::to_value(&frame).map_err(js_err)
}

// === Typed per-area encode/decode ===

#[wasm_bindgen(js_name = encodeSetup)]
pub fn encode_setup(v: Setup) -> Result<Vec<u8>, JsError> {
    v.to_bytes().map_err(js_err)
}
#[wasm_bindgen(js_name = decodeSetup)]
pub fn decode_setup(bytes: &[u8]) -> Result<Setup, JsError> {
    Setup::from_bytes(bytes).map_err(js_err)
}

#[wasm_bindgen(js_name = encodeSystemCommon)]
pub fn encode_system_common(v: SystemCommon) -> Result<Vec<u8>, JsError> {
    v.to_bytes().map_err(js_err)
}
#[wasm_bindgen(js_name = decodeSystemCommon)]
pub fn decode_system_common(bytes: &[u8]) -> Result<SystemCommon, JsError> {
    SystemCommon::from_bytes(bytes).map_err(js_err)
}
#[wasm_bindgen(js_name = encodeSystemControl)]
pub fn encode_system_control(v: SystemControl) -> Result<Vec<u8>, JsError> {
    v.to_bytes().map_err(js_err)
}
#[wasm_bindgen(js_name = decodeSystemControl)]
pub fn decode_system_control(bytes: &[u8]) -> Result<SystemControl, JsError> {
    SystemControl::from_bytes(bytes).map_err(js_err)
}

#[wasm_bindgen(js_name = encodePcMap)]
pub fn encode_pc_map(v: ProgramChangeMap) -> Result<Vec<u8>, JsError> {
    v.to_bytes().map_err(js_err)
}
#[wasm_bindgen(js_name = decodePcMap)]
pub fn decode_pc_map(bytes: &[u8]) -> Result<ProgramChangeMap, JsError> {
    ProgramChangeMap::from_bytes(bytes).map_err(js_err)
}

#[wasm_bindgen(js_name = encodeBank)]
pub fn encode_bank(v: BankParams) -> Result<Vec<u8>, JsError> {
    v.to_bytes().map_err(js_err)
}
#[wasm_bindgen(js_name = decodeBank)]
pub fn decode_bank(bytes: &[u8]) -> Result<BankParams, JsError> {
    BankParams::from_bytes(bytes).map_err(js_err)
}

#[wasm_bindgen(js_name = encodePatch)]
pub fn encode_patch(v: Patch) -> Result<Vec<u8>, JsError> {
    v.to_bytes().map_err(js_err)
}
#[wasm_bindgen(js_name = decodePatch)]
pub fn decode_patch(bytes: &[u8]) -> Result<Patch, JsError> {
    Patch::from_bytes(bytes).map_err(js_err)
}

// === Address helpers (all 99 user banks + temporary) ===

fn bank_slot(bank: u8) -> Result<BankSlot, JsError> {
    BankSlot::from_index(bank)
        .ok_or_else(|| JsError::new("bank out of range (0=temporary, 1..=99)"))
}

fn patch_slot(slot: &str) -> Result<PatchSlot, JsError> {
    match slot.to_ascii_lowercase().as_str() {
        "a" => Ok(PatchSlot::A),
        "b" => Ok(PatchSlot::B),
        "c" => Ok(PatchSlot::C),
        _ => Err(JsError::new("patch slot must be 'a', 'b', or 'c'")),
    }
}

/// 4-byte base address of bank `n` (0 = Temporary, 1..=99 = user banks).
#[wasm_bindgen(js_name = bankBase)]
pub fn bank_base(bank: u8) -> Result<Vec<u8>, JsError> {
    Ok(bank_slot(bank)?.base_address().to_vec())
}

/// 4-byte base address of a patch (`slot` = "a"/"b"/"c") within `bank`.
#[wasm_bindgen(js_name = patchBase)]
pub fn patch_base(bank: u8, slot: &str) -> Result<Vec<u8>, JsError> {
    Ok(patch_slot(slot)?.base_address(bank_slot(bank)?).to_vec())
}

#[wasm_bindgen(js_name = setupBase)]
pub fn setup_base() -> Vec<u8> {
    SETUP_BASE.to_vec()
}
#[wasm_bindgen(js_name = systemBase)]
pub fn system_base() -> Vec<u8> {
    SYSTEM_BASE.to_vec()
}
#[wasm_bindgen(js_name = systemControlAddress)]
pub fn system_control_address() -> Vec<u8> {
    md500_core::address::add_offset(SYSTEM_BASE, SYSTEM_CONTROL_OFFSET).to_vec()
}
#[wasm_bindgen(js_name = pcMapBase)]
pub fn pc_map_base() -> Vec<u8> {
    PC_MAP_BASE.to_vec()
}
#[wasm_bindgen(js_name = bankTemporaryBase)]
pub fn bank_temporary_base() -> Vec<u8> {
    BANK_TEMPORARY_BASE.to_vec()
}

/// Highest user bank number (99).
#[wasm_bindgen(js_name = bankMax)]
pub fn bank_max() -> u8 {
    BANK_MAX
}

// === Block lengths + RQ1/identity builders ===

#[wasm_bindgen(js_name = setupAreaLen)]
pub fn setup_area_len() -> usize {
    SETUP_AREA_LEN
}
#[wasm_bindgen(js_name = systemCommonLen)]
pub fn system_common_len() -> usize {
    SYSTEM_COMMON_LEN
}
#[wasm_bindgen(js_name = systemControlLen)]
pub fn system_control_len() -> usize {
    SYSTEM_CONTROL_LEN
}
#[wasm_bindgen(js_name = pcMapLen)]
pub fn pc_map_len() -> usize {
    PC_MAP_LEN
}
#[wasm_bindgen(js_name = bankParamsLen)]
pub fn bank_params_len() -> usize {
    BANK_PARAMS_LEN
}
#[wasm_bindgen(js_name = patchLen)]
pub fn patch_len() -> usize {
    PATCH_LEN
}

/// Build an RQ1 (data request) frame for `address` of `len` bytes from device `dev`.
#[wasm_bindgen(js_name = dataRequest)]
pub fn data_request(dev: u8, address: &[u8], len: usize) -> Result<Vec<u8>, JsError> {
    let addr: [u8; 4] = address
        .try_into()
        .map_err(|_| JsError::new("address must be 4 bytes"))?;
    Ok(Frame::data_request(dev, addr, size_field(len)).encode())
}

/// Build a DT1 (data set) frame writing `data` to `address` on device `dev`.
#[wasm_bindgen(js_name = dataSet)]
pub fn data_set(dev: u8, address: &[u8], data: Vec<u8>) -> Result<Vec<u8>, JsError> {
    let addr: [u8; 4] = address
        .try_into()
        .map_err(|_| JsError::new("address must be 4 bytes"))?;
    Ok(Frame::data_set(dev, addr, data).encode())
}

/// Universal Identity Request for device `dev` (`0x7F` = broadcast).
#[wasm_bindgen(js_name = identityRequest)]
pub fn identity_request_wasm(dev: u8) -> Vec<u8> {
    identity_request(dev)
}

/// `true`/`false` if `bytes` is an Identity Reply (MD-500 or not), `undefined` if
/// it is not an identity reply at all.
#[wasm_bindgen(js_name = identifyMd500)]
pub fn identify_md500(bytes: &[u8]) -> Option<bool> {
    identify(bytes)
}

// === Catalog / parameter metadata ===
//
// The editor reads labels / help / groups from here instead of maintaining a
// parallel TS table, so the Rust `catalog.rs` stays the single source of truth.

/// The full catalog bundle `{ device, params, catalogs, defaults }` as JSON.
#[wasm_bindgen(js_name = catalog)]
pub fn catalog() -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(&md500_core::catalog::bundle()).map_err(js_err)
}

/// Just the parameter table: `[{ path, label, group, help, level?, kind?, catalog? }]`.
#[wasm_bindgen(js_name = paramCatalog)]
pub fn param_catalog() -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(md500_core::catalog::params().as_slice()).map_err(js_err)
}

/// Help text for a field path (e.g. `"patch.chorus.chorus_type"`), or `undefined`.
#[wasm_bindgen(js_name = fieldHelp)]
pub fn field_help(path: &str) -> Option<String> {
    md500_core::catalog::params().help(path).map(String::from)
}

/// Full metadata for a field path, or `null` if the path isn't in the catalog.
#[wasm_bindgen(js_name = fieldMeta)]
pub fn field_meta(path: &str) -> Result<JsValue, JsError> {
    match md500_core::catalog::params().get(path) {
        Some(m) => serde_wasm_bindgen::to_value(m).map_err(js_err),
        None => Ok(JsValue::NULL),
    }
}

// === Inbound classification ===

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WasmInbound {
    DataSet {
        device_id: u8,
        space: String,
        #[tsify(type = "Uint8Array")]
        address: Vec<u8>,
        #[tsify(type = "Uint8Array")]
        data: Vec<u8>,
    },
    DataRequest {
        device_id: u8,
        #[tsify(type = "Uint8Array")]
        address: Vec<u8>,
        #[tsify(type = "Uint8Array")]
        size: Vec<u8>,
    },
    UnparseableSysEx {
        #[tsify(type = "Uint8Array")]
        bytes: Vec<u8>,
        error: String,
    },
    NonSysEx {
        #[tsify(type = "Uint8Array")]
        bytes: Vec<u8>,
    },
}

impl From<InboundMessage> for WasmInbound {
    fn from(m: InboundMessage) -> Self {
        match m {
            InboundMessage::DataSet {
                device_id,
                space,
                address,
                data,
            } => WasmInbound::DataSet {
                device_id,
                space: format!("{space:?}").to_lowercase(),
                address: address.to_vec(),
                data,
            },
            InboundMessage::DataRequest {
                device_id,
                address,
                size,
            } => WasmInbound::DataRequest {
                device_id,
                address: address.to_vec(),
                size,
            },
            InboundMessage::UnparseableSysEx { bytes, error } => WasmInbound::UnparseableSysEx {
                bytes,
                error: error.to_string(),
            },
            InboundMessage::NonSysEx(bytes) => WasmInbound::NonSysEx { bytes },
        }
    }
}

/// Classify a complete inbound MIDI byte sequence into a tagged-union variant.
#[wasm_bindgen(js_name = classifyInbound)]
pub fn classify_inbound(bytes: &[u8]) -> WasmInbound {
    core_classify(bytes).into()
}

#[cfg(test)]
mod tests {
    use md500_core::address::{BankSlot, PatchSlot};
    use md500_core::patch::PATCH_LEN;
    use md500_core::setup::Setup;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn setup_round_trip() {
        let s = Setup::from_bytes(&[0x00, 0x00, 0x00]).unwrap();
        assert_eq!(s.to_bytes().unwrap(), vec![0x00, 0x00, 0x00]);
    }

    #[wasm_bindgen_test]
    fn patch_address_helpers() {
        assert_eq!(super::bank_base(0).unwrap(), vec![0x30, 0x00, 0x00, 0x00]);
        assert_eq!(super::bank_base(99).unwrap(), vec![0x33, 0x0C, 0x00, 0x00]);
        assert_eq!(
            super::patch_base(0, "a").unwrap(),
            PatchSlot::A.base_address(BankSlot::Temporary).to_vec()
        );
        assert!(super::bank_base(100).is_err());
        assert!(super::patch_base(0, "z").is_err());
        assert_eq!(super::patch_len(), PATCH_LEN);
    }
}
