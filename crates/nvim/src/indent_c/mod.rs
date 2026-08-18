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

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{
    getdigits_int, getwhitecols_curline, skiptowhite, skipwhite, vim_isIDc, vim_iswordc,
    vim_iswordp, vim_strsize,
};
use crate::cursor::{get_cursor_line_ptr, get_cursor_pos_ptr};
use crate::eval::typval::tv_get_lnum;
use crate::global_cell::GlobalCell;
use crate::indent::{fixthisline, get_expr_indent, get_indent, get_indent_lnum, get_sw_value};
use crate::keycodes::get_special_key_code;
use crate::main::{State, curbuf, curwin, p_paste};
use crate::mbyte::{mb_prevptr, mb_strnicmp, utfc_ptr2len};
use crate::memline::{ml_get, ml_get_pos};
use crate::memory::{xfree, xstrdup};
use crate::option::{copy_option_part, skip_to_option_part};
use crate::os::cshim::strncmp;
use crate::plines::getvcol;
use crate::pos::{MAXCOL, MAXLNUM, lt};
use crate::search::{check_linecomment, findmatchlimit, linewhite};
use crate::state::MODE_INSERT;
use crate::strings::vim_strchr;
use crate::types::{
    EvalFuncData, buf_T, colnr_T, int64_t, linenr_T, lpos_T, oparg_T, pos_T, size_t, typval_T,
    varnumber_T,
};
use ::libc::{atoi, strlen, tolower};

// The carve of the transpiled module; see each child's docs.
mod cino;
mod comment;
mod cpp;
mod decl;
mod engine;
mod inblock;
mod incomment;
mod inparen;
mod keys;
mod label;
mod lookfor;
mod paren;
mod recog;
mod toplevel;

pub use self::cino::*;
pub use self::comment::*;
pub(crate) use self::cpp::*;
pub(crate) use self::decl::*;
pub use self::engine::*;
pub(crate) use self::inblock::*;
pub use self::keys::*;
pub(crate) use self::label::*;
pub(crate) use self::paren::*;
pub use self::recog::*;

// `line_vcol` (the `getvcol` of one position four amount answers here want)
// and `byte_at` (a NUL-terminated line read past its slice) moved to
// `indent.rs` at B15-17, which needs both as well. Re-exported rather than
// imported so the children keep reaching them through `use super::*`.
pub(crate) use crate::indent::{byte_at, line_vcol};

pub const KEY_COMPLETE: ::core::ffi::c_int = 259;
pub const KEY_OPEN_BACK: ::core::ffi::c_int = 258;
pub const KEY_OPEN_FORW: ::core::ffi::c_int = 257;
pub const FM_BACKWARD: ::core::ffi::c_int = 1;
#[derive(Copy, Clone)]
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
