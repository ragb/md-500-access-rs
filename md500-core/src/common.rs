//! Value types shared across MD-500 areas and modulation modes.
//!
//! These mirror recurring spec encodings: the controller-source list
//! ([`CcNumber`]), the two MIDI-channel encodings ([`MidiRxChannel`] /
//! [`MidiTxChannel`]), and the standard musical note-division list
//! ([`NoteValue`], reused by every mode's "Note" / tempo-sync parameter).

use serde::{Deserialize, Serialize};

use crate::codec::CodecError;

/// A controller selector — `OFF`, or a MIDI CC number (`CC#1..#31`, `CC#64..#95`).
///
/// Wire encoding (the spec's "MIDI … Control Change#" fields, range `0 - 63`):
/// `0 = OFF`, `1..=31 = CC#1..#31`, `32..=63 = CC#64..#95`. YAML: `off`, or the
/// integer CC number (e.g. `17`, `64`).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CcNumber {
    Symbolic(CcOff),
    Number(u8),
}

#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CcOff {
    Off,
}

impl CcNumber {
    pub const OFF: CcNumber = CcNumber::Symbolic(CcOff::Off);

    pub fn from_byte(b: u8, field: &'static str) -> Result<Self, CodecError> {
        match b {
            0 => Ok(CcNumber::OFF),
            1..=31 => Ok(CcNumber::Number(b)),       // CC#1..#31
            32..=63 => Ok(CcNumber::Number(b + 32)), // CC#64..#95
            _ => Err(CodecError::InvalidValue {
                field,
                value: b,
                valid: "0=off, 1..=31=CC#1..#31, 32..=63=CC#64..#95",
            }),
        }
    }

    pub fn to_byte(self, field: &'static str) -> Result<u8, CodecError> {
        match self {
            CcNumber::OFF => Ok(0),
            CcNumber::Number(n @ 1..=31) => Ok(n),
            CcNumber::Number(n @ 64..=95) => Ok(n - 32),
            CcNumber::Number(other) => Err(CodecError::OutOfRange {
                field,
                value: other as i32,
                valid: "1..=31 or 64..=95",
            }),
        }
    }
}

/// MIDI receive channel. Wire `0 - 16`: `0..=15 = Ch.1..Ch.16`, `16 = OFF`.
/// YAML: an integer `1..=16`, or `off`.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MidiRxChannel {
    Channel(u8),
    Symbolic(RxSymbol),
}

#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RxSymbol {
    Off,
}

impl MidiRxChannel {
    pub fn from_byte(b: u8) -> Result<Self, CodecError> {
        match b {
            0..=15 => Ok(MidiRxChannel::Channel(b + 1)),
            16 => Ok(MidiRxChannel::Symbolic(RxSymbol::Off)),
            _ => Err(CodecError::InvalidValue {
                field: "rx_channel",
                value: b,
                valid: "0..=15 (=Ch.1..16) or 16 (=off)",
            }),
        }
    }
    pub fn to_byte(self) -> Result<u8, CodecError> {
        match self {
            MidiRxChannel::Channel(n @ 1..=16) => Ok(n - 1),
            MidiRxChannel::Symbolic(RxSymbol::Off) => Ok(16),
            MidiRxChannel::Channel(other) => Err(CodecError::OutOfRange {
                field: "rx_channel",
                value: other as i32,
                valid: "1..=16",
            }),
        }
    }
}

/// MIDI transmit channel. Wire `0 - 17`: `0..=15 = Ch.1..Ch.16`, `16 = Rx`
/// (follow receive channel), `17 = OFF`. YAML: an integer `1..=16`, `rx`, or `off`.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MidiTxChannel {
    Channel(u8),
    Symbolic(TxSymbol),
}

#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxSymbol {
    Rx,
    Off,
}

impl MidiTxChannel {
    pub fn from_byte(b: u8) -> Result<Self, CodecError> {
        match b {
            0..=15 => Ok(MidiTxChannel::Channel(b + 1)),
            16 => Ok(MidiTxChannel::Symbolic(TxSymbol::Rx)),
            17 => Ok(MidiTxChannel::Symbolic(TxSymbol::Off)),
            _ => Err(CodecError::InvalidValue {
                field: "tx_channel",
                value: b,
                valid: "0..=15 (=Ch.1..16), 16 (=rx) or 17 (=off)",
            }),
        }
    }
    pub fn to_byte(self) -> Result<u8, CodecError> {
        match self {
            MidiTxChannel::Channel(n @ 1..=16) => Ok(n - 1),
            MidiTxChannel::Symbolic(TxSymbol::Rx) => Ok(16),
            MidiTxChannel::Symbolic(TxSymbol::Off) => Ok(17),
            MidiTxChannel::Channel(other) => Err(CodecError::OutOfRange {
                field: "tx_channel",
                value: other as i32,
                valid: "1..=16",
            }),
        }
    }
}

midi_access_core::byte_enum! {
    /// Which of a bank's three patches a [`PatchRef`] points at.
    PatchLetter {
        A = 0,
        B = 1,
        C = 2,
    }
    valid = "0=A, 1=B, 2=C"
}

/// A reference to a stored patch — bank `1..=99` and slot A/B/C — as used by the
/// Setup "current patch" field and every Program-Change-map entry.
///
/// Wire encoding: a single index `0..=296` (`0 = 01A`, `1 = 01B`, `2 = 01C`,
/// `3 = 02A`, … `296 = 99C`), nibble-packed across 3 bytes. YAML: `{ bank: 12,
/// slot: b }`.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchRef {
    /// Bank number, `1..=99`.
    pub bank: u8,
    /// Slot within the bank.
    pub slot: PatchLetter,
}

/// Number of bytes a [`PatchRef`] occupies on the wire (nibble-packed index).
pub const PATCH_REF_WIDTH: usize = 3;
const PATCH_REF_MAX_INDEX: u32 = 296;

impl PatchRef {
    /// Build from the wire index `0..=296`.
    pub fn from_index(index: u32, field: &'static str) -> Result<Self, CodecError> {
        if index > PATCH_REF_MAX_INDEX {
            return Err(CodecError::OutOfRange {
                field,
                value: index as i32,
                valid: "0..=296",
            });
        }
        Ok(Self {
            bank: (index / 3) as u8 + 1,
            slot: PatchLetter::from_byte((index % 3) as u8)?,
        })
    }

    /// The wire index `0..=296`.
    pub fn to_index(self, field: &'static str) -> Result<u32, CodecError> {
        if !(1..=99).contains(&self.bank) {
            return Err(CodecError::OutOfRange {
                field,
                value: self.bank as i32,
                valid: "bank 1..=99",
            });
        }
        Ok((self.bank as u32 - 1) * 3 + self.slot.to_byte() as u32)
    }

    /// Decode from `PATCH_REF_WIDTH` nibble-packed bytes.
    pub fn from_bytes(bytes: &[u8], field: &'static str) -> Result<Self, CodecError> {
        Self::from_index(crate::codec::read_nibbles(bytes), field)
    }

    /// Encode into `PATCH_REF_WIDTH` nibble-packed bytes.
    pub fn to_bytes(self, field: &'static str) -> Result<Vec<u8>, CodecError> {
        Ok(crate::codec::write_nibbles(
            self.to_index(field)?,
            PATCH_REF_WIDTH,
        ))
    }
}

midi_access_core::byte_enum! {
    /// Standard tempo-sync note division (`0 - 17`), reused by every mode's
    /// "Note" parameter. The MD-500 names them as fractions of a bar.
    NoteValue {
        ThirtySecond = 0,
        SixteenthTriplet = 1,
        DottedThirtySecond = 2,
        Sixteenth = 3,
        EighthTriplet = 4,
        DottedSixteenth = 5,
        Eighth = 6,
        QuarterTriplet = 7,
        DottedEighth = 8,
        Quarter = 9,
        HalfTriplet = 10,
        DottedQuarter = 11,
        Half = 12,
        WholeTriplet = 13,
        DottedHalf = 14,
        Whole = 15,
        DottedWhole = 16,
        DoubleWhole = 17,
    }
    valid = "0..=17 (note divisions 32th..double-whole)"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cc_number_maps_index_to_cc() {
        assert_eq!(CcNumber::from_byte(0, "f").unwrap(), CcNumber::OFF);
        assert_eq!(CcNumber::from_byte(1, "f").unwrap(), CcNumber::Number(1));
        assert_eq!(CcNumber::from_byte(31, "f").unwrap(), CcNumber::Number(31));
        assert_eq!(CcNumber::from_byte(32, "f").unwrap(), CcNumber::Number(64));
        assert_eq!(CcNumber::from_byte(63, "f").unwrap(), CcNumber::Number(95));
        assert!(CcNumber::from_byte(64, "f").is_err());
        for b in 0u8..=63 {
            let cc = CcNumber::from_byte(b, "f").unwrap();
            assert_eq!(cc.to_byte("f").unwrap(), b);
        }
    }

    #[test]
    fn cc_number_yaml_is_off_or_int() {
        assert_eq!(serde_yaml::to_string(&CcNumber::OFF).unwrap().trim(), "off");
        assert_eq!(
            serde_yaml::to_string(&CcNumber::Number(64)).unwrap().trim(),
            "64"
        );
    }

    #[test]
    fn rx_channel_round_trips() {
        for b in 0u8..=16 {
            let c = MidiRxChannel::from_byte(b).unwrap();
            assert_eq!(c.to_byte().unwrap(), b);
        }
        assert!(MidiRxChannel::from_byte(17).is_err());
        assert_eq!(
            MidiRxChannel::from_byte(0).unwrap(),
            MidiRxChannel::Channel(1)
        );
    }

    #[test]
    fn tx_channel_round_trips() {
        for b in 0u8..=17 {
            let c = MidiTxChannel::from_byte(b).unwrap();
            assert_eq!(c.to_byte().unwrap(), b);
        }
        assert!(MidiTxChannel::from_byte(18).is_err());
        assert_eq!(
            MidiTxChannel::from_byte(16).unwrap(),
            MidiTxChannel::Symbolic(TxSymbol::Rx)
        );
    }

    #[test]
    fn patch_ref_index_round_trips() {
        assert_eq!(
            PatchRef::from_index(0, "p").unwrap(),
            PatchRef {
                bank: 1,
                slot: PatchLetter::A
            }
        );
        assert_eq!(
            PatchRef::from_index(296, "p").unwrap(),
            PatchRef {
                bank: 99,
                slot: PatchLetter::C
            }
        );
        assert!(PatchRef::from_index(297, "p").is_err());
        for i in 0u32..=296 {
            assert_eq!(
                PatchRef::from_index(i, "p").unwrap().to_index("p").unwrap(),
                i
            );
        }
        // 3-byte nibble wire form.
        let r = PatchRef {
            bank: 12,
            slot: PatchLetter::B,
        };
        assert_eq!(
            PatchRef::from_bytes(&r.to_bytes("p").unwrap(), "p").unwrap(),
            r
        );
    }

    #[test]
    fn note_value_round_trips() {
        for b in 0u8..=17 {
            assert_eq!(NoteValue::from_byte(b).unwrap().to_byte(), b);
        }
        assert!(NoteValue::from_byte(18).is_err());
    }
}
