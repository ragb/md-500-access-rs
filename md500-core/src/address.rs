//! Parameter address map for the MD-500.
//!
//! Top-level layout (official MIDI Implementation v1.00 §3, see
//! `docs/sysex-notes.md`):
//!
//! | Address       | Area                                                  |
//! |---------------|-------------------------------------------------------|
//! | `00 00 00 00` | Setup (Setup Common)                                  |
//! | `10 00 00 00` | System (System Common `+00 00 00`, Control `+00 10 00`)|
//! | `20 00 00 00` | Program Change map                                    |
//! | `30 00 00 00` | BANK Temporary (bank params + PATCH A/B/C)            |
//! | `30 04 00 00` | BANK 01                                                |
//! |  :            |  :  (stride `00 04 00 00`)                            |
//! | `33 0C 00 00` | BANK 99                                                |
//!
//! Within a BANK: bank params `+00 00 00`, PATCH A `+00 10 00`, B `+00 20 00`,
//! C `+00 30 00` (patch stride `00 10 00`).
//!
//! Roland addresses are four **7-bit** bytes — a 28-bit big-endian value with a
//! carry at `0x80`, not `0x100`. All offset arithmetic goes through
//! [`to_u28`] / [`from_u28`] so the carry is handled in one place. (Verified
//! against the spec anchor: BANK 99 = `30 00 00 00` + 99·`00 04 00 00` =
//! `33 0C 00 00`.)

#![allow(dead_code)]

/// Base address of the Setup area.
pub const SETUP_BASE: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

/// Base address of the System area.
pub const SYSTEM_BASE: [u8; 4] = [0x10, 0x00, 0x00, 0x00];

/// Offset of System Common within the System area.
pub const SYSTEM_COMMON_OFFSET: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
/// Offset of System Control within the System area.
pub const SYSTEM_CONTROL_OFFSET: [u8; 4] = [0x00, 0x00, 0x10, 0x00];

/// Base address of the MIDI Program Change map.
pub const PC_MAP_BASE: [u8; 4] = [0x20, 0x00, 0x00, 0x00];

/// Base address of the BANK Temporary area (the live, edited bank).
pub const BANK_TEMPORARY_BASE: [u8; 4] = [0x30, 0x00, 0x00, 0x00];

/// Stride between consecutive user banks (`00 04 00 00`).
pub const BANK_STRIDE: [u8; 4] = [0x00, 0x04, 0x00, 0x00];

/// Offset of the per-bank SIMUL parameter block (the bank "header").
pub const BANK_PARAMS_OFFSET: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
/// Offset of PATCH A within a bank.
pub const PATCH_A_OFFSET: [u8; 4] = [0x00, 0x00, 0x10, 0x00];
/// Offset of PATCH B within a bank.
pub const PATCH_B_OFFSET: [u8; 4] = [0x00, 0x00, 0x20, 0x00];
/// Offset of PATCH C within a bank.
pub const PATCH_C_OFFSET: [u8; 4] = [0x00, 0x00, 0x30, 0x00];

/// Highest user bank number.
pub const BANK_MAX: u8 = 99;

/// Pack a 4-byte Roland address into its 28-bit integer value (7 bits/byte).
pub fn to_u28(addr: [u8; 4]) -> u32 {
    ((addr[0] as u32 & 0x7F) << 21)
        | ((addr[1] as u32 & 0x7F) << 14)
        | ((addr[2] as u32 & 0x7F) << 7)
        | (addr[3] as u32 & 0x7F)
}

/// Unpack a 28-bit value into a 4-byte Roland address (7 bits/byte).
pub fn from_u28(v: u32) -> [u8; 4] {
    [
        ((v >> 21) & 0x7F) as u8,
        ((v >> 14) & 0x7F) as u8,
        ((v >> 7) & 0x7F) as u8,
        (v & 0x7F) as u8,
    ]
}

/// Add an offset to a base address with correct 7-bit-per-byte carry.
pub fn add_offset(base: [u8; 4], offset: [u8; 4]) -> [u8; 4] {
    from_u28(to_u28(base).wrapping_add(to_u28(offset)))
}

/// Encode a byte count as Roland's 4-byte, 7-bit big-endian size field (for RQ1).
pub fn size_field(n: usize) -> [u8; 4] {
    from_u28(n as u32)
}

/// Top-level partition of the MD-500 address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressSpace {
    /// `00 xx xx xx` — Setup.
    Setup,
    /// `10 xx xx xx` — System.
    System,
    /// `20 xx xx xx` — Program Change map.
    PcMap,
    /// `30 xx xx xx`..`33 xx xx xx` — temporary + 99 user banks (incl. patches).
    Bank,
    /// Anything we haven't classified.
    Unknown,
}

impl AddressSpace {
    pub fn classify(address: [u8; 4]) -> Self {
        match address[0] {
            0x00 => Self::Setup,
            0x10 => Self::System,
            0x20 => Self::PcMap,
            0x30..=0x33 => Self::Bank,
            _ => Self::Unknown,
        }
    }
}

/// A bank: the live Temporary bank, or one of 99 user banks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankSlot {
    Temporary,
    User(u8),
}

impl BankSlot {
    /// Base address of this bank's block.
    ///
    /// ```
    /// use md500_core::address::BankSlot;
    /// assert_eq!(BankSlot::Temporary.base_address(), [0x30, 0x00, 0x00, 0x00]);
    /// assert_eq!(BankSlot::User(1).base_address(),   [0x30, 0x04, 0x00, 0x00]);
    /// assert_eq!(BankSlot::User(99).base_address(),  [0x33, 0x0C, 0x00, 0x00]);
    /// ```
    pub fn base_address(self) -> [u8; 4] {
        match self {
            BankSlot::Temporary => BANK_TEMPORARY_BASE,
            BankSlot::User(n) => {
                let v = to_u28(BANK_TEMPORARY_BASE) + (n as u32) * to_u28(BANK_STRIDE);
                from_u28(v)
            }
        }
    }

    /// Build from a user-facing index: `0` = Temporary, `1..=99` = user banks.
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(BankSlot::Temporary),
            n if n <= BANK_MAX => Some(BankSlot::User(n)),
            _ => None,
        }
    }
}

/// One of a bank's three patches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchSlot {
    A,
    B,
    C,
}

impl PatchSlot {
    /// This patch's offset within its bank.
    pub fn offset(self) -> [u8; 4] {
        match self {
            PatchSlot::A => PATCH_A_OFFSET,
            PatchSlot::B => PATCH_B_OFFSET,
            PatchSlot::C => PATCH_C_OFFSET,
        }
    }

    /// Absolute base address of this patch within `bank`.
    ///
    /// ```
    /// use md500_core::address::{BankSlot, PatchSlot};
    /// assert_eq!(
    ///     PatchSlot::A.base_address(BankSlot::Temporary),
    ///     [0x30, 0x00, 0x10, 0x00]
    /// );
    /// assert_eq!(
    ///     PatchSlot::C.base_address(BankSlot::User(1)),
    ///     [0x30, 0x04, 0x30, 0x00]
    /// );
    /// ```
    pub fn base_address(self, bank: BankSlot) -> [u8; 4] {
        add_offset(bank.base_address(), self.offset())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u28_round_trips() {
        for a in [
            [0x00, 0x00, 0x00, 0x00],
            [0x33, 0x0C, 0x00, 0x00],
            [0x7F, 0x7F, 0x7F, 0x7F],
            [0x30, 0x00, 0x10, 0x00],
        ] {
            assert_eq!(from_u28(to_u28(a)), a);
        }
    }

    #[test]
    fn bank_addresses_match_spec_anchors() {
        assert_eq!(BankSlot::Temporary.base_address(), [0x30, 0x00, 0x00, 0x00]);
        assert_eq!(BankSlot::User(1).base_address(), [0x30, 0x04, 0x00, 0x00]);
        assert_eq!(BankSlot::User(99).base_address(), [0x33, 0x0C, 0x00, 0x00]);
    }

    #[test]
    fn patch_addresses() {
        assert_eq!(
            PatchSlot::A.base_address(BankSlot::Temporary),
            [0x30, 0x00, 0x10, 0x00]
        );
        assert_eq!(
            PatchSlot::B.base_address(BankSlot::Temporary),
            [0x30, 0x00, 0x20, 0x00]
        );
        assert_eq!(
            PatchSlot::C.base_address(BankSlot::Temporary),
            [0x30, 0x00, 0x30, 0x00]
        );
        // Patch base in a user bank carries through the bank stride.
        assert_eq!(
            PatchSlot::A.base_address(BankSlot::User(1)),
            [0x30, 0x04, 0x10, 0x00]
        );
    }

    #[test]
    fn classify_known_addresses() {
        assert_eq!(AddressSpace::classify(SETUP_BASE), AddressSpace::Setup);
        assert_eq!(AddressSpace::classify(SYSTEM_BASE), AddressSpace::System);
        assert_eq!(AddressSpace::classify(PC_MAP_BASE), AddressSpace::PcMap);
        assert_eq!(
            AddressSpace::classify(BANK_TEMPORARY_BASE),
            AddressSpace::Bank
        );
        assert_eq!(
            AddressSpace::classify([0x33, 0x0C, 0x00, 0x00]),
            AddressSpace::Bank
        );
        assert_eq!(
            AddressSpace::classify([0x40, 0x00, 0x00, 0x00]),
            AddressSpace::Unknown
        );
    }

    #[test]
    fn size_field_encodes_7bit() {
        assert_eq!(size_field(0x22), [0x00, 0x00, 0x00, 0x22]);
        assert_eq!(size_field(128), [0x00, 0x00, 0x01, 0x00]);
        assert_eq!(size_field(0), [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn from_index_rejects_out_of_range() {
        assert!(matches!(BankSlot::from_index(0), Some(BankSlot::Temporary)));
        assert!(matches!(BankSlot::from_index(99), Some(BankSlot::User(99))));
        assert!(BankSlot::from_index(100).is_none());
    }
}
