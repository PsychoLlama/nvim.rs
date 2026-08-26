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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::Buf;
use core::ffi::{CStr, c_int, c_uint, c_void};

use crate::ascii::ascii_iswhite;
use crate::cursor::get_cursor_pos_ptr;
use crate::main::{cmdwin_buf, curbuf, curwin, p_paste};
use crate::mbyte::{utf_iscomposing_first, utf_ptr2char};
use crate::types::FoFlag;
use crate::window::win_fdccol_count;

mod auto;
mod lines;
mod para;
mod wrap;

pub use self::auto::*;
pub(crate) use self::lines::*;
pub(crate) use self::para::*;
pub use self::wrap::*;

pub const kBufOptFormatexpr: c_int = 36;
pub const OPENLINE_FORMAT: c_uint = 32;
pub const OPENLINE_COM_LIST: c_uint = 16;
pub const OPENLINE_MARKFIX: c_uint = 8;
pub const OPENLINE_KEEPTRAIL: c_uint = 4;
pub const OPENLINE_DO_COM: c_uint = 2;
pub const OPENLINE_DELSPACES: c_uint = 1;
pub const INDENT_SET: c_uint = 1;
pub const SIN_CHANGED: c_uint = 1;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();

// The 'formatoptions' flag letters this family reads. Each is the option's
// own spelling, so a flag test is `has_format_option(FoFlag::WRAP)`.

// The 'comments' item flag letters: `s`tart, `m`iddle, `e`nd, `f`irst.
pub const COM_START: c_int = 's' as c_int;
pub const COM_MIDDLE: c_int = 'm' as c_int;
pub const COM_END: c_int = 'e' as c_int;
pub const COM_FIRST: c_int = 'f' as c_int;

/// Whether 'formatoptions' flag `x` is in effect. Always false under 'paste',
/// which turns formatting off wholesale.
///
/// # Safety
/// There must be a current buffer.
pub unsafe fn has_format_option(x: FoFlag) -> bool {
    // The dereference stays behind the `&&`: with no current buffer the
    // left half is what keeps the right one from running.
    unsafe { p_paste.get() == 0 && x.is_in(CStr::from_ptr(cur_buf().b_p_fo)) }
}

/// `WHITECHAR` (`v0.12.4:textformat.c:50`): `cc` is white space, and the
/// character at the cursor is not the base of a combining sequence.
///
/// The two halves are about different positions on purpose. The argument is
/// whatever the caller has in hand, while the composing test always reads the
/// byte *after* the cursor -- so a caller passing `line[col - 1]` is asking
/// about two different characters at once. Upstream is a macro, and every
/// call site inherits that.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
pub(crate) unsafe fn whitechar(cc: c_int) -> bool {
    ascii_iswhite(cc)
        && !utf_iscomposing_first(unsafe { utf_ptr2char(get_cursor_pos_ptr().add(1)) })
}

/// The width to format to: 'textwidth' if set, else the window width less
/// 'wrapmargin', else zero. `ff` forces a usable answer for `gq`, which is
/// the window width capped at 79.
///
/// # Safety
/// There must be a current window and buffer.
pub unsafe fn comp_textwidth(ff: bool) -> c_int {
    let win = curwin.get();
    let mut textwidth = cur_buf().b_p_tw as c_int;
    if textwidth == 0 && cur_buf().b_p_wm != 0 {
        textwidth = unsafe { (*win).w_view_width } - cur_buf().b_p_wm as c_int;
        if curbuf.get() == cmdwin_buf.get() {
            textwidth -= 1;
        }
        textwidth -= unsafe { win_fdccol_count(win) };
        textwidth -= unsafe { (*win).w_scwidth };
        if unsafe { (*win).w_onebuf_opt.wo_nu } != 0 || unsafe { (*win).w_onebuf_opt.wo_rnu } != 0 {
            textwidth -= 8;
        }
    }
    textwidth = textwidth.max(0);
    if ff && textwidth == 0 {
        textwidth = (unsafe { (*win).w_view_width } - 1).min(79);
    }
    textwidth
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
