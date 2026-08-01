#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// The head every compiled pattern starts with, whichever engine built it.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regprog {
    pub engine: *mut regengine_T,
    pub regflags: ::core::ffi::c_uint,
    pub re_engine: ::core::ffi::c_uint,
    pub re_flags: ::core::ffi::c_uint,
    pub re_in_use: bool,
}
pub type regengine_T = regengine;
/// The vtable of a regexp engine (backtracking or NFA).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regengine {
    pub regcomp: Option<unsafe extern "C" fn(*mut uint8_t, ::core::ffi::c_int) -> *mut regprog_T>,
    pub regfree: Option<unsafe extern "C" fn(*mut regprog_T) -> ()>,
    pub regexec_nl: Option<
        unsafe extern "C" fn(*mut regmatch_T, *mut uint8_t, colnr_T, bool) -> ::core::ffi::c_int,
    >,
    pub regexec_multi: Option<
        unsafe extern "C" fn(
            *mut regmmatch_T,
            *mut win_T,
            *mut buf_T,
            linenr_T,
            colnr_T,
            *mut proftime_T,
            *mut ::core::ffi::c_int,
        ) -> ::core::ffi::c_int,
    >,
}

pub type magic_T = ::core::ffi::c_uint;
pub type optmagic_T = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct reg_extmatch_T {
    pub refcnt: int16_t,
    pub matches: [*mut uint8_t; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmmatch_T {
    pub regprog: *mut regprog_T,
    pub startpos: [lpos_T; 10],
    pub endpos: [lpos_T; 10],
    pub rmm_matchcol: colnr_T,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: colnr_T,
}

impl Default for regmmatch_T {
    fn default() -> Self {
        regmmatch_T {
            regprog: ::core::ptr::null_mut(),
            startpos: [lpos_T::default(); 10],
            endpos: [lpos_T::default(); 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        }
    }
}

impl Default for regmatch_T {
    fn default() -> Self {
        regmatch_T {
            regprog: ::core::ptr::null_mut(),
            startp: [::core::ptr::null_mut(); 10],
            endp: [::core::ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}
