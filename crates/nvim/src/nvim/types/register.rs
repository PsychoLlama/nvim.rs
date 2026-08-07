#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type GRegFlags = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct block_def {
    pub startspaces: ::core::ffi::c_int,
    pub endspaces: ::core::ffi::c_int,
    pub textlen: ::core::ffi::c_int,
    pub textstart: *mut ::core::ffi::c_char,
    pub textcol: colnr_T,
    pub start_vcol: colnr_T,
    pub end_vcol: colnr_T,
    pub is_short: ::core::ffi::c_int,
    pub is_MAX: ::core::ffi::c_int,
    pub is_oneChar: ::core::ffi::c_int,
    pub pre_whitesp: ::core::ffi::c_int,
    pub pre_whitesp_c: ::core::ffi::c_int,
    pub end_char_vcols: colnr_T,
    pub start_char_vcols: colnr_T,
}

impl block_def {
    /// All zeros — the state `block_prep` and `charwise_block_prep` overwrite.
    ///
    /// C declares these uninitialised and fills them in; every caller in the
    /// tree does exactly that, so the zeros are never read.
    pub const ZERO: Self = block_def {
        startspaces: 0,
        endspaces: 0,
        textlen: 0,
        textstart: ::core::ptr::null_mut(),
        textcol: 0,
        start_vcol: 0,
        end_vcol: 0,
        is_short: 0,
        is_MAX: 0,
        is_oneChar: 0,
        pre_whitesp: 0,
        pre_whitesp_c: 0,
        end_char_vcols: 0,
        start_char_vcols: 0,
    };
}

impl Default for block_def {
    fn default() -> Self {
        Self::ZERO
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct yankreg_T {
    pub y_array: *mut String_0,
    pub y_size: size_t,
    pub y_type: MotionType,
    pub y_width: colnr_T,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}

/// `do_put()` flags — upstream's anonymous enum in `register_defs.h`.
///
/// c2rust typed this `c_uint`, so every use site is `PUT_X as c_int`;
/// retyping belongs to the slice that deletes those casts.
pub type PutFlags = ::core::ffi::c_uint;

/// make the indent look nice
pub const PUT_FIXINDENT: PutFlags = 1;
/// leave the cursor after the end of the new text
pub const PUT_CURSEND: PutFlags = 2;
/// leave the cursor on the last line of the new text
pub const PUT_CURSLINE: PutFlags = 4;
/// put the register as lines
pub const PUT_LINE: PutFlags = 8;
/// split the line for a linewise register
pub const PUT_LINE_SPLIT: PutFlags = 16;
/// put a linewise register below the Visual selection
pub const PUT_LINE_FORWARD: PutFlags = 32;
/// in block mode, do not add trailing spaces
pub const PUT_BLOCK_INNER: PutFlags = 64;
