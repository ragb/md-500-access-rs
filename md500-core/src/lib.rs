//! Pure codec for the BOSS MD-500 modulation pedal SysEx protocol.
//!
//! No I/O, no threads, no MIDI backend — compiles for `wasm32-unknown-unknown`.

#![forbid(unsafe_code)]

pub mod address;
pub mod assigncat;
#[allow(deprecated)]
// tsify `into_wasm_abi`/`from_wasm_abi` (incl. `byte_enum!`); kept for the wasm ABI
pub mod bank;
pub mod catalog;
pub mod codec;
#[allow(deprecated)]
// tsify `into_wasm_abi`/`from_wasm_abi` (incl. `byte_enum!`); kept for the wasm ABI
pub mod common;
pub mod device;
pub mod inbound;
#[allow(deprecated)]
// tsify `into_wasm_abi`/`from_wasm_abi` (incl. `byte_enum!`); kept for the wasm ABI
pub mod patch;
#[allow(deprecated)]
// tsify `into_wasm_abi`/`from_wasm_abi` (incl. `byte_enum!`); kept for the wasm ABI
pub mod pc_map;
#[allow(deprecated)]
// tsify `into_wasm_abi`/`from_wasm_abi` (incl. `byte_enum!`); kept for the wasm ABI
pub mod setup;
pub mod sysex;
#[allow(deprecated)]
// tsify `into_wasm_abi`/`from_wasm_abi` (incl. `byte_enum!`); kept for the wasm ABI
pub mod system;
pub mod yaml;

pub use bank::BankParams;
pub use catalog::{MD500_CATALOGS, Md500Catalogs};
pub use codec::CodecError;
pub use common::{CcNumber, MidiRxChannel, MidiTxChannel, NoteValue, PatchLetter, PatchRef};
pub use device::Md500;
pub use inbound::{InboundMessage, classify_inbound};
pub use patch::{ModulationMode, PATCH_LEN, Patch};
pub use pc_map::ProgramChangeMap;
pub use setup::Setup;
pub use sysex::{
    CMD_DT1, CMD_RQ1, Frame, MD500_MODEL_ID, ROLAND_ID, SysExError, identify, identity_request,
};
pub use system::{System, SystemCommon, SystemControl};
