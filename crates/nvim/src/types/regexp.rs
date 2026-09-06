#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::winlayer::{Buf, Win};

/// The head every compiled pattern starts with, whichever engine built it.
#[repr(C)]
pub struct RegProg {
    pub engine: *mut RegEngine,
    pub regflags: ::core::ffi::c_uint,
    pub re_engine: ::core::ffi::c_uint,
    pub re_flags: ::core::ffi::c_uint,
    pub re_in_use: bool,
}
/// Compile a pattern, or null if it does not parse.
pub type RegComp = Option<unsafe fn(*mut uint8_t, ::core::ffi::c_int) -> *mut RegProg>;
/// Release a compiled pattern.
pub type RegFree = Option<unsafe fn(*mut RegProg) -> ()>;
/// Match within one line.
pub type RegExecNl =
    Option<unsafe fn(*mut RegMatch, *mut uint8_t, ColNr, bool) -> ::core::ffi::c_int>;
/// Match across lines of a buffer, with a timeout.
pub type RegExecMulti = Option<
    unsafe fn(
        *mut RegMMatch,
        Option<Win>,
        Buf,
        LineNr,
        ColNr,
        *mut ProfTime,
        *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
/// The vtable of a regexp engine (backtracking or NFA).
pub struct RegEngine {
    pub regcomp: RegComp,
    pub regfree: RegFree,
    pub regexec_nl: RegExecNl,
    pub regexec_multi: RegExecMulti,
}

pub type Magic = ::core::ffi::c_uint;
pub type OptMagic = ::core::ffi::c_uint;
pub struct RegExtMatch {
    pub refcnt: int16_t,
    pub matches: [*mut uint8_t; 10],
}
#[derive(Clone)]
pub struct RegMatch {
    pub regprog: *mut RegProg,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: ColNr,
    pub rm_ic: bool,
}
#[derive(Clone)]
pub struct RegMMatch {
    pub regprog: *mut RegProg,
    pub startpos: [LPos; 10],
    pub endpos: [LPos; 10],
    pub rmm_matchcol: ColNr,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: ColNr,
}

impl Default for RegMMatch {
    fn default() -> Self {
        RegMMatch {
            regprog: ::core::ptr::null_mut(),
            startpos: [LPos::default(); 10],
            endpos: [LPos::default(); 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        }
    }
}

impl Default for RegMatch {
    fn default() -> Self {
        RegMatch {
            regprog: ::core::ptr::null_mut(),
            startp: [::core::ptr::null_mut(); 10],
            endp: [::core::ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}
