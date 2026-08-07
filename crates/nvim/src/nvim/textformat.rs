//! Formatting text: where a line is broken, and what a paragraph is.
//!
//! Three entry points, one per way the user asks for it, and they meet in
//! `insertchar` (`edit.rs`), which calls `internal_format`:
//!
//! | file | asked by | what |
//! | --- | --- | --- |
//! | `wrap` | typing past 'textwidth' | break the line being typed |
//! | `lines` | `gq` / `gw` / 'formatexpr' | reflow a range of paragraphs |
//! | `auto` | 'formatoptions' `a` | reflow the paragraph after every change |
//! | `para` | all three | where a paragraph ends, and whether two lines share a leader |
//!
//! The parent keeps the `FO_*` / `COM_*` / `OPENLINE_*` vocabulary,
//! [`has_format_option`] (which every one of those files asks) and
//! [`comp_textwidth`] (the margin they are all measured against).

use crate::src::nvim::main::{cmdwin_buf, curbuf, curwin, p_paste};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::window::win_fdccol_count;

mod auto;
mod lines;
mod para;
mod wrap;

pub use self::auto::*;
pub(crate) use self::lines::*;
pub(crate) use self::para::*;
pub use self::wrap::*;

pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const kBufOptFormatexpr: C2Rust_Unnamed_14 = 36;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_15 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_15 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_15 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_15 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_15 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const INDENT_SET: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_19 = 4;
pub const BL_WHITE: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const SIN_CHANGED: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_23 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FO_WRAP: ::core::ffi::c_int = 't' as ::core::ffi::c_int;
pub const FO_WRAP_COMS: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const FO_Q_COMS: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const FO_Q_NUMBER: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const FO_Q_SECOND: ::core::ffi::c_int = '2' as ::core::ffi::c_int;
pub const FO_INS_VI: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const FO_INS_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const FO_MBYTE_BREAK: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const FO_ONE_LETTER: ::core::ffi::c_int = '1' as ::core::ffi::c_int;
pub const FO_WHITE_PAR: ::core::ffi::c_int = 'w' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const FO_RIGOROUS_TW: ::core::ffi::c_int = ']' as ::core::ffi::c_int;
pub const FO_PERIOD_ABBR: ::core::ffi::c_int = 'p' as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_FIRST: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
pub unsafe extern "C" fn has_format_option(mut x: ::core::ffi::c_int) -> bool {
    if p_paste.get() != 0 {
        return false;
    }
    return !vim_strchr((*curbuf.get()).b_p_fo, x).is_null();
}
pub unsafe extern "C" fn comp_textwidth(mut ff: bool) -> ::core::ffi::c_int {
    let mut textwidth: ::core::ffi::c_int = (*curbuf.get()).b_p_tw as ::core::ffi::c_int;
    if textwidth == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_wm != 0 {
        textwidth = (*curwin.get()).w_view_width - (*curbuf.get()).b_p_wm as ::core::ffi::c_int;
        if curbuf.get() == cmdwin_buf.get() {
            textwidth -= 1 as ::core::ffi::c_int;
        }
        textwidth -= win_fdccol_count(curwin.get());
        textwidth -= (*curwin.get()).w_scwidth;
        if (*curwin.get()).w_onebuf_opt.wo_nu != 0 || (*curwin.get()).w_onebuf_opt.wo_rnu != 0 {
            textwidth -= 8 as ::core::ffi::c_int;
        }
    }
    textwidth = if textwidth > 0 as ::core::ffi::c_int {
        textwidth
    } else {
        0 as ::core::ffi::c_int
    };
    if ff as ::core::ffi::c_int != 0 && textwidth == 0 as ::core::ffi::c_int {
        textwidth = if ((*curwin.get()).w_view_width - 1 as ::core::ffi::c_int)
            < 79 as ::core::ffi::c_int
        {
            (*curwin.get()).w_view_width - 1 as ::core::ffi::c_int
        } else {
            79 as ::core::ffi::c_int
        };
    }
    return textwidth;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
