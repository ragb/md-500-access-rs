//! Setup area (`00 00 00 00`) — Setup Common.
//!
//! A single field: the device's currently-selected patch. Spec §3 *2-1.
//! Spec v1.00, not yet device-verified.

use serde::{Deserialize, Serialize};

use crate::codec::CodecError;
use crate::common::{PatchRef, PATCH_REF_WIDTH};

/// Encoded length of the Setup Common block (one 3-byte patch index).
pub const SETUP_AREA_LEN: usize = PATCH_REF_WIDTH;

/// Typed view of the Setup area.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(deny_unknown_fields))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    /// The currently-selected patch (read/write: writing recalls that patch).
    pub current_patch: PatchRef,
}

impl Setup {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != SETUP_AREA_LEN {
            return Err(CodecError::WrongLength {
                expected: SETUP_AREA_LEN,
                actual: bytes.len(),
            });
        }
        Ok(Self {
            current_patch: PatchRef::from_bytes(bytes, "current_patch")?,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        self.current_patch.to_bytes("current_patch")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::PatchLetter;

    #[test]
    fn round_trips() {
        let s = Setup {
            current_patch: PatchRef {
                bank: 42,
                slot: PatchLetter::C,
            },
        };
        let bytes = s.to_bytes().unwrap();
        assert_eq!(bytes.len(), SETUP_AREA_LEN);
        assert_eq!(Setup::from_bytes(&bytes).unwrap(), s);
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(matches!(
            Setup::from_bytes(&[0x00, 0x00]),
            Err(CodecError::WrongLength { .. })
        ));
    }

    #[test]
    fn yaml_shape() {
        let s = Setup::from_bytes(&[0x00, 0x00, 0x00]).unwrap();
        let y = serde_yaml::to_string(&s).unwrap();
        assert!(y.contains("current_patch:"));
        assert!(y.contains("bank: 1"));
        assert!(y.contains("slot: a"));
    }
}
