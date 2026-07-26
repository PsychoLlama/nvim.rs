//! Accessors for C bit-packed struct fields.
//!
//! The transpiled tree stores C bitfields as little `[u8; N]` arrays inside
//! `repr(C)` structs, with numbered bits (bit `i` lives in byte `i / 8` at
//! bit `i % 8` — LSB-first, matching C bitfield layout on little-endian
//! targets). [`bitfield_accessors!`] generates the `name()`/`set_name()`
//! methods that read and write those bit ranges.
//!
//! Semantics match what a C compiler does with the original bitfields:
//! - Getters read bits `lo..=hi` into the low bits of the value type;
//!   `bool` reads as "any bit in the range set".
//! - Setters write the low bits of the value into `lo..=hi`, ignoring
//!   higher bits; `bool` fills the whole range with the flag.
//! - Only unsigned value types are supported. No signed bitfield exists in
//!   the tree; add the sign-extension here if one ever appears.
#![forbid(unsafe_code)]

/// Marker for types a bitfield range can be read as / written from.
pub trait FieldValue: Copy {
    fn from_bits(bits: u64) -> Self;
    fn to_bits(self) -> u64;
}

macro_rules! impl_field_value {
    ($($ty:ty),+) => {
        $(impl FieldValue for $ty {
            fn from_bits(bits: u64) -> Self {
                bits as Self
            }
            fn to_bits(self) -> u64 {
                self as u64
            }
        })+
    };
}

impl_field_value! {u8, u16, u32, u64, usize}

impl FieldValue for bool {
    fn from_bits(bits: u64) -> Self {
        bits != 0
    }
    // All-ones so the setter fills every bit of the range, matching how C
    // assigns a flag to a multi-bit range.
    fn to_bits(self) -> u64 {
        if self { u64::MAX } else { 0 }
    }
}

/// Read bits `lo..=hi` of `storage` into the low bits of the result.
pub fn get_bits(storage: &[u8], lo: u32, hi: u32) -> u64 {
    let mut bits = 0;
    for (i, bit) in (lo..=hi).enumerate() {
        if storage[(bit / 8) as usize] & (1 << (bit % 8)) != 0 {
            bits |= 1 << i;
        }
    }
    bits
}

/// Write the low bits of `bits` into bits `lo..=hi` of `storage`.
pub fn set_bits(storage: &mut [u8], lo: u32, hi: u32, bits: u64) {
    for (i, bit) in (lo..=hi).enumerate() {
        let byte = &mut storage[(bit / 8) as usize];
        let mask = 1 << (bit % 8);
        if bits >> i & 1 != 0 {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_reads_lsb_first_within_and_across_bytes() {
        // 0b0000_0110 in byte 0, plus bit 8 (LSB of byte 1).
        let storage = [0b0000_0110u8, 0b0000_0001];
        assert_eq!(get_bits(&storage, 0, 0), 0);
        assert_eq!(get_bits(&storage, 1, 2), 0b11);
        // A range spanning the byte boundary: bits 7..=8 are 0 and 1.
        assert_eq!(get_bits(&storage, 7, 8), 0b10);
    }

    #[test]
    fn set_only_touches_the_range() {
        let mut storage = [0xFFu8, 0x00];
        set_bits(&mut storage, 2, 5, 0);
        assert_eq!(storage, [0b1100_0011, 0x00]);
        set_bits(&mut storage, 6, 9, 0b1111);
        assert_eq!(storage, [0b1100_0011, 0b0000_0011]);
    }

    #[test]
    fn set_truncates_to_the_range_width() {
        let mut storage = [0u8];
        // Only the low two bits of the value land in a two-bit range.
        set_bits(&mut storage, 0, 1, 0b111);
        assert_eq!(storage, [0b11]);
    }

    #[test]
    fn roundtrip_through_the_value_types() {
        let mut storage = [0u8; 4];
        set_bits(&mut storage, 3, 12, u32::to_bits(0x2AB));
        assert_eq!(u32::from_bits(get_bits(&storage, 3, 12)), 0x2AB);
        assert_eq!(usize::from_bits(get_bits(&storage, 3, 12)), 0x2AB);
    }

    #[test]
    fn bool_fills_and_clears_the_whole_range() {
        let mut storage = [0u8];
        set_bits(&mut storage, 1, 3, bool::to_bits(true));
        assert_eq!(storage, [0b0000_1110]);
        assert!(bool::from_bits(get_bits(&storage, 1, 3)));
        set_bits(&mut storage, 2, 2, bool::to_bits(false));
        // A single cleared bit still reads back true: any set bit counts.
        assert!(bool::from_bits(get_bits(&storage, 1, 3)));
        set_bits(&mut storage, 1, 3, bool::to_bits(false));
        assert!(!bool::from_bits(get_bits(&storage, 1, 3)));
    }
}

/// Generate `name()`/`set_name()` accessors for bit ranges of one `[u8; N]`
/// storage field:
///
/// ```ignore
/// crate::bitfield_accessors! {
///     impl VTermGlyphInfo.protected_cell_dwl_dhl {
///         0..=0 => protected_cell, set_protected_cell: ::core::ffi::c_uint;
///         1..=1 => dwl, set_dwl: ::core::ffi::c_uint;
///         2..=3 => dhl, set_dhl: ::core::ffi::c_uint;
///     }
/// }
/// ```
#[macro_export]
macro_rules! bitfield_accessors {
    (
        impl $Struct:ident . $storage:ident {
            $( $lo:literal ..= $hi:literal => $getter:ident, $setter:ident : $Ty:ty; )+
        }
    ) => {
        impl $Struct {
            $(
                #[inline]
                pub fn $getter(&self) -> $Ty {
                    <$Ty as $crate::src::nvim::bitfield::FieldValue>::from_bits(
                        $crate::src::nvim::bitfield::get_bits(&self.$storage, $lo, $hi),
                    )
                }

                #[inline]
                pub fn $setter(&mut self, value: $Ty) {
                    $crate::src::nvim::bitfield::set_bits(
                        &mut self.$storage,
                        $lo,
                        $hi,
                        <$Ty as $crate::src::nvim::bitfield::FieldValue>::to_bits(value),
                    );
                }
            )+
        }
    };
}
