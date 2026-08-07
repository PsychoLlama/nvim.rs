//! C indenting: `'cindent'`, `'cinoptions'`, `'cinkeys'` and `cindent()`.
//!
//! One question -- "what column should this line start at" -- answered by
//! reading the lines above it.  `get_c_indent` (in `engine`) is the answer;
//! everything else in this family exists to serve it.  The order it works in
//! is: skip out of any comment or string (`comment`), decide what kind of
//! line each candidate above is (`recog`, `decl`, `cpp`), find the enclosing
//! bracket (`paren`), and read an amount off the line it settles on
//! (`label`).  `cino` parses the option that sets every constant in that
//! arithmetic, and `keys` decides when the whole thing runs at all.
//!
//! This parent keeps no functions, only the vocabulary the children share:
//!
//! | name | what it names |
//! | --- | --- |
//! | `COM_*` | the flag letters of a `'comments'` item -- `s`tart, `m`iddle, `e`nd, `l`eft, `r`ight |
//! | `BRACE_*` | where the `{` opening the current block sat: column 0, start of line, end of line |
//! | `LOOKFOR_*` | the state of `get_c_indent`'s backwards scan -- what it is still searching for |
//! | `FIND_NAMESPACE_LIM` | how far back `LOOKFOR_*`'s namespace hunt may go |
//! | `FM_*` | `findmatchlimit` direction/stop flags |
//! | `KEY_*` | the pseudo-keys `in_cinkeys` is asked about that are not typed characters |
//! | `cpp_baseclass_cache_T` | one line's `cin_is_cpp_baseclass` answer, cached across the scan |

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{
    getdigits_int, getwhitecols_curline, skiptowhite, skipwhite, vim_isIDc, vim_iswordc,
    vim_iswordp, vim_strsize,
};
use crate::src::nvim::cursor::{get_cursor_line_ptr, get_cursor_pos_ptr};
use crate::src::nvim::eval::typval::tv_get_lnum;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{
    fixthisline, get_expr_indent, get_indent, get_indent_lnum, get_sw_value,
};
use crate::src::nvim::keycodes::get_special_key_code;
use crate::src::nvim::main::{State, curbuf, curwin, p_paste};
use crate::src::nvim::mbyte::{mb_prevptr, mb_strnicmp, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get, ml_get_pos};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::option::{copy_option_part, skip_to_option_part};
use crate::src::nvim::os::libc::{atoi, strcpy, strlen, strncmp, tolower};
use crate::src::nvim::plines::getvcol;
use crate::src::nvim::pos::{MAXCOL, MAXLNUM, lt};
use crate::src::nvim::search::{check_linecomment, findmatchlimit, linewhite};
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    EvalFuncData, buf_T, colnr_T, int64_t, intptr_t, linenr_T, lpos_T, oparg_T, pos_T, size_t,
    typval_T, uint8_t, varnumber_T,
};

// The carve of the transpiled module; see each child's docs.
mod cino;
mod comment;
mod cpp;
mod decl;
mod engine;
mod keys;
mod label;
mod paren;
mod recog;

pub use self::cino::*;
pub use self::comment::*;
pub(crate) use self::cpp::*;
pub(crate) use self::decl::*;
pub use self::engine::*;
pub use self::keys::*;
pub(crate) use self::label::*;
pub(crate) use self::paren::*;
pub use self::recog::*;

/// The screen column byte `col` of line `lnum` sits at.
///
/// Four of the amount answers in this family are a `getvcol` of one
/// position, and `getvcol` needs a `pos_T` and three out-parameters to say
/// so.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub(crate) unsafe fn line_vcol(lnum: linenr_T, col: colnr_T) -> ::core::ffi::c_int {
    unsafe {
        let mut fp = pos_T {
            lnum,
            col,
            coladd: 0,
        };
        let mut vcol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut fp,
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        vcol
    }
}

/// The byte at `i` of a NUL-terminated line held as a slice.
///
/// Past the end is the terminator: `CStr::to_bytes()` drops it, but the
/// memory is still there, and upstream reaches these strings through the NUL
/// rather than through a length.
pub(crate) fn byte_at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

pub const KEY_COMPLETE: ::core::ffi::c_int = 259;
pub const KEY_OPEN_BACK: ::core::ffi::c_int = 258;
pub const KEY_OPEN_FORW: ::core::ffi::c_int = 257;
pub const FM_BACKWARD: ::core::ffi::c_int = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cpp_baseclass_cache_T {
    pub found: ::core::ffi::c_int,
    pub lpos: lpos_T,
}
pub const FM_BLOCKSTOP: ::core::ffi::c_int = 4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_LEFT: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const COM_RIGHT: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const FIND_NAMESPACE_LIM: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const BRACE_IN_COL0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BRACE_AT_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BRACE_AT_END: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOOKFOR_INITIAL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LOOKFOR_IF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOOKFOR_DO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOOKFOR_CASE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOOKFOR_ANY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LOOKFOR_TERM: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LOOKFOR_UNTERM: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const LOOKFOR_SCOPEDECL: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const LOOKFOR_NOBREAK: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const LOOKFOR_CPP_BASECLASS: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const LOOKFOR_ENUM_OR_INIT: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const LOOKFOR_JS_KEY: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const LOOKFOR_COMMA: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
