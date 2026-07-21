// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKey {
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub canonflags: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_uchar,
    pub buffstart: size_t,
    pub buffcount: size_t,
    pub buffsize: size_t,
    pub hightide: size_t,
    pub restore_termios: termios,
    pub restore_termios_valid: ::core::ffi::c_char,
    pub ti_getstr_hook: Option<TermKey_Terminfo_Getstr_Hook>,
    pub ti_getstr_hook_data: *mut ::core::ffi::c_void,
    pub waittime: ::core::ffi::c_int,
    pub is_closed: ::core::ffi::c_char,
    pub is_started: ::core::ffi::c_char,
    pub nkeynames: ::core::ffi::c_int,
    pub keynames: *mut *const ::core::ffi::c_char,
    pub c0: [keyinfo; 32],
    pub drivers: *mut TermKeyDriverNode,
    pub method: TermKey_method,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyCsi {
    pub tk: *mut TermKey,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKey_method {
    pub emit_codepoint:
        Option<unsafe extern "C" fn(*mut TermKey, ::core::ffi::c_int, *mut TermKeyKey) -> ()>,
    pub peekkey_simple: Option<
        unsafe extern "C" fn(
            *mut TermKey,
            *mut TermKeyKey,
            ::core::ffi::c_int,
            *mut size_t,
        ) -> TermKeyResult,
    >,
    pub peekkey_mouse:
        Option<unsafe extern "C" fn(*mut TermKey, *mut TermKeyKey, *mut size_t) -> TermKeyResult>,
}
