//! Program Change map (`20 00 00 00`) — spec *2-4.
//!
//! 384 entries: three program-change banks (Bank Select MSB `0/1/2`) of 128
//! program numbers each, every entry a [`PatchRef`] (`0..=296`, nibble-packed in
//! 3 bytes). Entry `i` is selected by Bank Select MSB `i / 128` and Program
//! Change `i % 128`. Spec v1.00, not yet device-verified.

use serde::{Deserialize, Serialize};

use crate::codec::CodecError;
use crate::common::{PatchRef, PATCH_REF_WIDTH};

/// Program-change banks (Bank Select MSB values 0..=2).
pub const PC_MAP_BANKS: usize = 3;
/// Program numbers per bank.
pub const PC_MAP_PROGRAMS: usize = 128;
/// Total number of map entries.
pub const PC_MAP_ENTRIES: usize = PC_MAP_BANKS * PC_MAP_PROGRAMS; // 384
/// Encoded length of the whole map.
pub const PC_MAP_LEN: usize = PC_MAP_ENTRIES * PATCH_REF_WIDTH; // 1152

/// Typed view of the Program Change map.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramChangeMap {
    /// The 384 mapped patches, in (bank-MSB, program) row-major order. Entry `i`
    /// is reached by Bank Select MSB `i / 128`, Program Change `i % 128`.
    pub entries: Vec<PatchRef>,
}

impl ProgramChangeMap {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() != PC_MAP_LEN {
            return Err(CodecError::WrongLength {
                expected: PC_MAP_LEN,
                actual: b.len(),
            });
        }
        let entries = b
            .chunks_exact(PATCH_REF_WIDTH)
            .map(|c| PatchRef::from_bytes(c, "pc_map_entry"))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { entries })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        if self.entries.len() != PC_MAP_ENTRIES {
            return Err(CodecError::OutOfRange {
                field: "entries",
                value: self.entries.len() as i32,
                valid: "exactly 384 entries",
            });
        }
        let mut out = Vec::with_capacity(PC_MAP_LEN);
        for (i, e) in self.entries.iter().enumerate() {
            let _ = i;
            out.extend_from_slice(&e.to_bytes("pc_map_entry")?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::PatchLetter;

    #[test]
    fn full_map_round_trips() {
        // Default-ish map: every PC maps to patch 01A.
        let map = ProgramChangeMap {
            entries: (0..PC_MAP_ENTRIES)
                .map(|i| PatchRef::from_index((i % 297) as u32, "e").unwrap())
                .collect(),
        };
        let bytes = map.to_bytes().unwrap();
        assert_eq!(bytes.len(), PC_MAP_LEN);
        assert_eq!(ProgramChangeMap::from_bytes(&bytes).unwrap(), map);
    }

    #[test]
    fn rejects_wrong_entry_count() {
        let map = ProgramChangeMap {
            entries: vec![PatchRef {
                bank: 1,
                slot: PatchLetter::A,
            }],
        };
        assert!(map.to_bytes().is_err());
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(matches!(
            ProgramChangeMap::from_bytes(&[0u8; 10]),
            Err(CodecError::WrongLength { .. })
        ));
    }
}
