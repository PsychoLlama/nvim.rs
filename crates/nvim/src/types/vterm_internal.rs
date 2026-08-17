#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
//
// These are libvterm's types, Copyright (c) 2008 Paul Evans, under the MIT
// license; the notice is reproduced in licenses/libvterm-LICENSE.txt.
use core::mem::offset_of;

use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermEncoding {
    pub init: Option<unsafe extern "C" fn(*mut VTermEncoding, *mut ::core::ffi::c_void) -> ()>,
    pub decode: Option<
        unsafe extern "C" fn(
            *mut VTermEncoding,
            *mut ::core::ffi::c_void,
            *mut uint32_t,
            *mut ::core::ffi::c_int,
            ::core::ffi::c_int,
            *const ::core::ffi::c_char,
            *mut size_t,
            size_t,
        ) -> (),
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermEncodingInstance {
    pub enc: *mut VTermEncoding,
    pub data: [::core::ffi::c_char; 16],
}
pub type VTermEncodingType = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermKeyEncodingFlags {
    pub disambiguate_report_events_report_alternate_report_all_keys_report_associated: [u8; 1],
}
crate::bitfield_accessors! {
    impl VTermKeyEncodingFlags.disambiguate_report_events_report_alternate_report_all_keys_report_associated {
        0..=0 => disambiguate, set_disambiguate: bool;
        1..=1 => report_events, set_report_events: bool;
        2..=2 => report_alternate, set_report_alternate: bool;
        3..=3 => report_all_keys, set_report_all_keys: bool;
        4..=4 => report_associated, set_report_associated: bool;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermKeyEncodingStack {
    pub items: [VTermKeyEncodingFlags; 16],
    pub size: uint8_t,
}
pub type VTermParserState = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VTermPen {
    pub fg: VTermColor,
    pub bg: VTermColor,
    pub uri: ::core::ffi::c_int,
    pub bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline:
        [u8; 3],
    pub c2rust_padding: [u8; 1],
}
crate::bitfield_accessors! {
    impl VTermPen.bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline {
        0..=0 => bold, set_bold: ::core::ffi::c_uint;
        1..=2 => underline, set_underline: ::core::ffi::c_uint;
        3..=3 => italic, set_italic: ::core::ffi::c_uint;
        4..=4 => blink, set_blink: ::core::ffi::c_uint;
        5..=5 => reverse, set_reverse: ::core::ffi::c_uint;
        6..=6 => conceal, set_conceal: ::core::ffi::c_uint;
        7..=7 => strike, set_strike: ::core::ffi::c_uint;
        8..=11 => font, set_font: ::core::ffi::c_uint;
        12..=12 => small, set_small: ::core::ffi::c_uint;
        13..=14 => baseline, set_baseline: ::core::ffi::c_uint;
        15..=15 => dim, set_dim: ::core::ffi::c_uint;
        16..=16 => overline, set_overline: ::core::ffi::c_uint;
    }
}

// ---------------------------------------------------------------------------
// The layout, asserted.
//
// These types cross a C ABI in three directions: the emulator hands them to
// `terminal.rs`, the unit specs declare them through LuaJIT's FFI, and
// `unit-fixtures.so` compiles C against those same declarations. `repr(C)`
// fixes the *rules* the compiler lays them out by, not the answer, so a
// field that widens or a pair that swaps stays a valid `repr(C)` type and
// silently disagrees with everything on the other side of the boundary.
//
// The numbers are what x86-64 System V (and any other LP64 target) gives
// libvterm's own declarations. They are the same sizes, alignments and
// offsets the emulator's differential reads back out of a hand-written FFI
// declaration of these types, so a change that moves one moves the other.
// ---------------------------------------------------------------------------
const _: () = {
    assert!(size_of::<VTermPen>() == 16 && align_of::<VTermPen>() == 4);
    assert!(size_of::<VTermEncodingInstance>() == 24 && align_of::<VTermEncodingInstance>() == 8);
    assert!(size_of::<VTermKeyEncodingFlags>() == 1 && align_of::<VTermKeyEncodingFlags>() == 1);
    assert!(size_of::<VTermKeyEncodingStack>() == 17 && align_of::<VTermKeyEncodingStack>() == 1);

    assert!(offset_of!(VTermPen, fg) == 0);
    assert!(offset_of!(VTermPen, bg) == 4);
    assert!(offset_of!(VTermPen, uri) == 8);
    assert!(offset_of!(VTermKeyEncodingStack, items) == 0);
    assert!(offset_of!(VTermKeyEncodingStack, size) == 16);
};
