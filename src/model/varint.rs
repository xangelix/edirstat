use std::borrow::Cow;

/// Max bytes required to represent a 64-bit integer in LEB128 varint format.
/// Ceil(64 bits / 7 bits per byte) = 10 bytes.
pub const MAX_VARINT_BYTES: usize = 10;

/// Maps signed integers (`i64`) to unsigned integers (`u64`) such that
/// values with small absolute magnitudes (both positive and negative)
/// map to small unsigned values.
///
/// Positive values are mapped to even numbers: 0 -> 0, 1 -> 2, 2 -> 4...
/// Negative values are mapped to odd numbers: -1 -> 1, -2 -> 3, -3 -> 5...
#[must_use]
#[inline]
pub const fn zigzag_encode(val: i64) -> u64 {
    // Arithmetic right shift by 63 fills the register with the sign bit
    // (all 1s if negative, all 0s if positive).
    // XORing with this mask maps the negative values cleanly.
    ((val << 1) ^ (val >> 63)) as u64
}

/// Restores a signed integer (`i64`) from a ZigZag-encoded unsigned integer (`u64`).
#[must_use]
#[inline]
pub const fn zigzag_decode(val: u64) -> i64 {
    // Shifting right by 1 isolates the magnitude.
    // XORing with the negated lowest bit restores the original sign.
    ((val >> 1) as i64) ^ -((val & 1) as i64)
}

/// Writes an unsigned 64-bit integer to a stream using LEB128 varint encoding.
/// Returns the number of bytes written to the stream.
pub(super) fn write_u64_varint(buf: &mut Vec<u8>, mut val: u64) {
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;

        if val != 0 {
            byte |= 0x80;
            buf.push(byte);
        } else {
            buf.push(byte);
            break;
        }
    }
}

/// Reads an unsigned 64-bit integer from a stream using LEB128 varint decoding.
pub(super) const fn read_u64_varint(
    slice: &[u8],
    cursor: &mut usize,
) -> Result<u64, crate::EdirstatError> {
    let mut val = 0u64;
    let mut shift = 0;

    loop {
        if *cursor >= slice.len() {
            return Err(crate::EdirstatError::Decode(
                "varint stream ended unexpectedly",
            ));
        }

        let byte = slice[*cursor];
        *cursor += 1;
        let payload = (byte & 0x7F) as u64;

        // Prevent overflow attacks (larger than 64-bit shifts)
        if shift >= 64 {
            return Err(crate::EdirstatError::Decode(
                "overlong varint exceeds 64 bits",
            ));
        }

        val |= payload << shift;

        if (byte & 0x80) == 0 {
            break;
        }

        shift += 7;
    }
    Ok(val)
}

/// Writes a signed 64-bit integer using `ZigZag` and LEB128 varint encoding.
#[inline]
pub(super) fn write_i64_zigzag(buf: &mut Vec<u8>, val: i64) {
    write_u64_varint(buf, zigzag_encode(val));
}

/// Reads a signed 64-bit integer using `ZigZag` and LEB128 varint decoding.
#[inline]
pub(super) fn read_i64_zigzag(
    slice: &[u8],
    cursor: &mut usize,
) -> Result<i64, crate::EdirstatError> {
    let val = read_u64_varint(slice, cursor)?;
    Ok(zigzag_decode(val))
}

// =============================================================================
// Slice API (For In-Memory / Zero-Allocation Array Buffers)
// =============================================================================

/// Converts a slice of `u32` to little-endian representation.
/// Compiles down to an empty, zero-allocation borrow on little-endian platforms.
pub(super) fn u32_slice_to_le(slice: &[u32]) -> Cow<'_, [u32]> {
    if cfg!(target_endian = "little") {
        Cow::Borrowed(slice)
    } else {
        let mut le_slice = slice.to_vec();
        for val in &mut le_slice {
            *val = val.to_le();
        }
        Cow::Owned(le_slice)
    }
}

/// Safely transforms a raw, potentially unaligned `u8` slice into an aligned `Vec<u32>`
/// without pointer casting risks.
pub(super) fn u8_slice_to_u32_vec(bytes: &[u8]) -> Vec<u32> {
    let u32_size = std::mem::size_of::<u32>();
    let count = bytes.len() / u32_size;
    let mut vec = vec![0u32; count];

    let target_bytes = bytemuck::cast_slice_mut(&mut vec);
    target_bytes.copy_from_slice(bytes);
    vec
}
