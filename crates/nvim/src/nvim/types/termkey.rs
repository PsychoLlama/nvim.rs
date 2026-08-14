#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
//
// These are libtermkey's types, Copyright (c) 2007-2011 Paul Evans, under
// the MIT license; the notice is reproduced in
// licenses/libtermkey-LICENSE.txt.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKey {
    pub flags: ::core::ffi::c_int,
    pub canonflags: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_uchar,
    /// Where the unread input starts, and how much of it there is. The pair
    /// walks forward through `buffer` until a read compacts it.
    pub buffstart: size_t,
    pub buffcount: size_t,
    pub buffsize: size_t,
    /// Bytes of an unrecognised control sequence held back so the consumer can
    /// re-read them, and discarded at the start of the next read.
    pub hightide: size_t,
    pub ti_getstr_hook: Option<TermKey_Terminfo_Getstr_Hook>,
    pub ti_getstr_hook_data: *mut ::core::ffi::c_void,
    pub is_started: ::core::ffi::c_char,
    /// Terminfo driver state (`TerminfoDriver`), opaque outside
    /// `tui::termkey::driver_ti`.
    pub ti: *mut ::core::ffi::c_void,
    pub csi: *mut TermKeyCsi,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyCsi {
    pub saved_string_id: ::core::ffi::c_int,
    pub saved_string: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyCsiParam {
    pub param: *const ::core::ffi::c_uchar,
    pub length: size_t,
}
pub type TermKeyEvent = ::core::ffi::c_uint;
pub type TermKeyFormat = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyKey {
    pub type_0: TermKeyType,
    pub code: TermKeyKey_code,
    pub modifiers: ::core::ffi::c_int,
    pub event: TermKeyEvent,
    pub utf8: [::core::ffi::c_char; 7],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union TermKeyKey_code {
    pub codepoint: ::core::ffi::c_int,
    pub number: ::core::ffi::c_int,
    pub sym: TermKeySym,
    pub mouse: [::core::ffi::c_char; 4],
}
pub type TermKeyMouseEvent = ::core::ffi::c_uint;
pub type TermKeyResult = ::core::ffi::c_uint;
pub type TermKeySym = ::core::ffi::c_int;
pub type TermKeyType = ::core::ffi::c_int;
