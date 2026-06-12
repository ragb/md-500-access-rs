//! Pure codec for the BOSS MD-500 modulation pedal SysEx protocol.
//!
//! No I/O, no threads, no MIDI backend — compiles for `wasm32-unknown-unknown`.

#![forbid(unsafe_code)]

pub mod address;
pub mod assigncat;
pub mod bank;
pub mod catalog;
pub mod codec;
pub mod common;
pub mod device;
pub mod inbound;
pub mod patch;
pub mod pc_map;
pub mod setup;
pub mod sysex;
pub mod system;
pub mod yaml;

pub use bank::BankParams;
pub use catalog::{Md500Catalogs, MD500_CATALOGS};
pub use codec::CodecError;
pub use common::{CcNumber, MidiRxChannel, MidiTxChannel, NoteValue, PatchLetter, PatchRef};
pub use device::Md500;
pub use inbound::{classify_inbound, InboundMessage};
pub use patch::{ModulationMode, Patch, PATCH_LEN};
pub use pc_map::ProgramChangeMap;
pub use setup::Setup;
pub use sysex::{
    identify, identity_request, Frame, SysExError, CMD_DT1, CMD_RQ1, MD500_MODEL_ID, ROLAND_ID,
};
pub use system::{System, SystemCommon, SystemControl};
