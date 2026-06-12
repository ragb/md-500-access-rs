//! MD-500-specific codec helpers, layered on the kit's shared primitives.
//!
//! The kit ([`midi_access_core::codec`]) already covers single-byte shapes —
//! [`ranged`], [`bool_byte`], [`signed_center`], [`read_ascii`]/[`write_ascii`],
//! and the [`byte_enum!`](midi_access_core::byte_enum) macro. The MD-500 adds two
//! recurring shapes the kit doesn't:
//!
//! - **variable-width nibble packing** — a value spread MSN-first across N bytes,
//!   each carrying its low nibble (`0000 dddd`); see [`read_nibbles`] /
//!   [`write_nibbles`]. (The kit's [`read_u16_nibbles`](midi_access_core::codec::read_u16_nibbles)
//!   is the fixed-4-byte special case.)
//! - **display offset** — the spec stores some signed/biased values with a fixed
//!   bias (e.g. `offset:100`, display `-100..23000`); see [`from_offset`] /
//!   [`to_offset`].
//!
//! The error type is the kit's [`CodecError`], re-exported so area modules name a
//! single error.

pub use midi_access_core::codec::CodecError;

/// Decode a nibble-packed value: `bytes` MSN-first, each byte's low 4 bits used.
///
/// ```
/// use md500_core::codec::read_nibbles;
/// // Rate = 500 → 0x1F4 → nibbles 1, F, 4 across 3 bytes.
/// assert_eq!(read_nibbles(&[0x01, 0x0F, 0x04]), 500);
/// ```
pub fn read_nibbles(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |acc, &b| (acc << 4) | (b as u32 & 0x0F))
}

/// Encode `v` into `width` nibble-carrying bytes, MSN first.
///
/// ```
/// use md500_core::codec::write_nibbles;
/// assert_eq!(write_nibbles(500, 3), vec![0x01, 0x0F, 0x04]);
/// assert_eq!(write_nibbles(0, 5), vec![0, 0, 0, 0, 0]);
/// ```
pub fn write_nibbles(v: u32, width: usize) -> Vec<u8> {
    (0..width)
        .map(|i| ((v >> (4 * (width - 1 - i))) & 0x0F) as u8)
        .collect()
}

/// Decode and range-check a nibble-packed value into `lo..=hi` (the *raw* range).
pub fn read_nibbles_ranged(
    bytes: &[u8],
    lo: u32,
    hi: u32,
    field: &'static str,
) -> Result<u32, CodecError> {
    let v = read_nibbles(bytes);
    if (lo..=hi).contains(&v) {
        Ok(v)
    } else {
        Err(CodecError::OutOfRange {
            field,
            value: v as i32,
            valid: leak_range(lo as i64, hi as i64),
        })
    }
}

/// Encode a nibble-packed value into `width` bytes, range-checked to `lo..=hi`.
pub fn write_nibbles_ranged(
    v: u32,
    width: usize,
    lo: u32,
    hi: u32,
    field: &'static str,
) -> Result<Vec<u8>, CodecError> {
    if (lo..=hi).contains(&v) {
        Ok(write_nibbles(v, width))
    } else {
        Err(CodecError::OutOfRange {
            field,
            value: v as i32,
            valid: leak_range(lo as i64, hi as i64),
        })
    }
}

/// Decode a biased value: `display = raw - offset`. The spec lists the raw range
/// and notes `offset:N`; this returns the signed display value.
///
/// ```
/// use md500_core::codec::from_offset;
/// // Exp target Min: raw 0..23100, offset 100 → display -100..23000.
/// assert_eq!(from_offset(0, 100), -100);
/// assert_eq!(from_offset(100, 100), 0);
/// ```
pub fn from_offset(raw: u32, offset: i64) -> i64 {
    raw as i64 - offset
}

/// Encode a biased display value back to its raw stored value (`raw = display +
/// offset`), validated to the raw `0..=raw_hi` range.
pub fn to_offset(
    display: i64,
    offset: i64,
    raw_hi: u32,
    field: &'static str,
) -> Result<u32, CodecError> {
    let raw = display + offset;
    if (0..=raw_hi as i64).contains(&raw) {
        Ok(raw as u32)
    } else {
        Err(CodecError::OutOfRange {
            field,
            value: display as i32,
            valid: leak_range(-offset, raw_hi as i64 - offset),
        })
    }
}

/// Format a `lo..=hi` range into a `'static` string for error messages.
fn leak_range(lo: i64, hi: i64) -> &'static str {
    Box::leak(format!("{lo}..={hi}").into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nibbles_round_trip_various_widths() {
        for (v, w) in [(0u32, 3), (500, 3), (2000, 3), (96000, 5), (2400100, 6)] {
            let bytes = write_nibbles(v, w);
            assert_eq!(bytes.len(), w);
            assert!(bytes.iter().all(|&b| b < 0x10), "all low-nibble bytes");
            assert_eq!(read_nibbles(&bytes), v, "round trip {v} width {w}");
        }
    }

    #[test]
    fn nibbles_ranged_rejects_out_of_band() {
        assert!(read_nibbles_ranged(&[0x0F, 0x0F, 0x0F], 1, 2000, "rate").is_err());
        assert_eq!(
            read_nibbles_ranged(&[0x01, 0x0F, 0x04], 1, 2000, "rate").unwrap(),
            500
        );
        assert!(write_nibbles_ranged(2001, 3, 1, 2000, "rate").is_err());
    }

    #[test]
    fn offset_round_trips() {
        assert_eq!(from_offset(0, 100), -100);
        assert_eq!(from_offset(23000, 100), 22900);
        assert_eq!(to_offset(-100, 100, 23100, "min").unwrap(), 0);
        assert_eq!(to_offset(0, 100, 23100, "min").unwrap(), 100);
        assert!(to_offset(-101, 100, 23100, "min").is_err());
    }
}
