//! BANK parameters (`+00 00 00` within a bank) — spec *2-5.
//!
//! These four bytes configure how a bank's two effects combine when
//! `Foot Switch Mode = SIMUL` (the A/B simultaneous mode). The bank's three
//! patches (A/B/C) live at separate offsets and are modelled in [`crate::patch`].
//! Spec v1.00, not yet device-verified.

use serde::{Deserialize, Serialize};

use midi_access_core::byte_enum;
use midi_access_core::codec::bool_byte;

use crate::codec::CodecError;

/// Encoded length of the BANK parameter block.
pub const BANK_PARAMS_LEN: usize = 0x04;

byte_enum! {
    /// How the two effects are chained in SIMUL. (`Structure`)
    BankStructure { Series = 0, Parallel = 1 }
    valid = "0=series, 1=parallel"
}
byte_enum! {
    /// Where the bank's effects sit relative to the insert loop. (`Insert Position`)
    BankInsertPosition { Off = 0, Pre = 1, Post = 2, Middle = 3 }
    valid = "0=off, 1=pre, 2=post, 3=middle"
}
byte_enum! {
    /// Output routing for the two effects. (`Output Mode`)
    BankOutputMode { Mix = 0, Ab = 1 }
    valid = "0=mix, 1=a/b"
}

/// Typed view of a bank's SIMUL parameter block.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(deny_unknown_fields))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankParams {
    /// Series vs parallel routing of the two effects (offset 0x00).
    pub structure: BankStructure,
    /// Insert position relative to the loop (offset 0x01).
    pub insert_position: BankInsertPosition,
    /// Mix vs A/B output routing (offset 0x02).
    pub output_mode: BankOutputMode,
    /// Synchronise A and B tempo/LFO (offset 0x03).
    pub ab_sync_sw: bool,
}

impl BankParams {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() != BANK_PARAMS_LEN {
            return Err(CodecError::WrongLength {
                expected: BANK_PARAMS_LEN,
                actual: b.len(),
            });
        }
        Ok(Self {
            structure: BankStructure::from_byte(b[0x00])?,
            insert_position: BankInsertPosition::from_byte(b[0x01])?,
            output_mode: BankOutputMode::from_byte(b[0x02])?,
            ab_sync_sw: bool_byte(b[0x03], "ab_sync_sw")?,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        Ok(vec![
            self.structure.to_byte(),
            self.insert_position.to_byte(),
            self.output_mode.to_byte(),
            self.ab_sync_sw as u8,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let bytes = [0x01, 0x03, 0x01, 0x01];
        let bp = BankParams::from_bytes(&bytes).unwrap();
        assert_eq!(bp.structure, BankStructure::Parallel);
        assert_eq!(bp.insert_position, BankInsertPosition::Middle);
        assert_eq!(bp.output_mode, BankOutputMode::Ab);
        assert!(bp.ab_sync_sw);
        assert_eq!(bp.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn yaml_named_enums() {
        let bp = BankParams::from_bytes(&[0x00, 0x01, 0x00, 0x00]).unwrap();
        let y = serde_yaml::to_string(&bp).unwrap();
        assert!(y.contains("structure: series"));
        assert!(y.contains("insert_position: pre"));
        assert!(y.contains("output_mode: mix"));
    }
}
