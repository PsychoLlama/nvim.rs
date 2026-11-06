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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::c_int;

use crate::ascii::ascii_iswhite;
use crate::mbyte::{char_at, utf_iscomposing_first};
use crate::memline::Lines;
use crate::option::vars::p_paste;
use crate::optionstr::OptStringRef;
use crate::types::FoFlag;
use crate::window::win_fdccol_count;
use crate::winlayer::graph::cmdwin_buf;

mod auto;
mod lines;
mod para;
mod wrap;

pub use self::auto::*;
pub(crate) use self::lines::*;
pub(crate) use self::para::*;
pub use self::wrap::*;

pub const kBufOptFormatexpr: c_int = 36;

// The 'formatoptions' flag letters this family reads. Each is the option's
// own spelling, so a flag test is `has_format_option(FoFlag::WRAP)`.

/// Whether 'formatoptions' flag `x` is in effect. Always false under 'paste',
/// which turns formatting off wholesale.
///
/// Safe: the only promise is that the editor exists, which `cur_buf()`
/// carries. The dereference stays behind the `&&`: with 'paste' set the
/// left half is what keeps the right one from running.
pub(crate) fn has_format_option(x: FoFlag) -> bool {
    !p_paste() && x.is_among(Buf::current().b_p_fo.bytes())
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
/// Upstream reads the byte after the cursor even when the cursor is on the
/// line's terminator; past the end reads as no character here.
#[inline]
pub(crate) fn whitechar(cc: c_int) -> bool {
    ascii_iswhite(cc) && !composing_after_cursor()
}

/// Whether the character one byte after the cursor is a composing one. An
/// ASCII byte, the end of the line among them, never is -- the common case,
/// and the only one that needs no decode.
fn composing_after_cursor() -> bool {
    let cursor = Win::current().w_cursor;
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    let after = usize::try_from(cursor.col).unwrap_or(0) + 1;
    line.get(after)
        .is_some_and(|&b| b >= 0x80 && utf_iscomposing_first(char_at(&line[after..])))
}

/// The width to format to: 'textwidth' if set, else the window width less
/// 'wrapmargin', else zero. `ff` forces a usable answer for `gq`, which is
/// the window width capped at 79.
pub fn comp_textwidth(ff: bool) -> c_int {
    let win = Win::current();
    let mut textwidth = c_int::try_from(Buf::current().b_p_tw).unwrap_or(c_int::MAX);
    if textwidth == 0 && Buf::current().b_p_wm != 0 {
        let margin = c_int::try_from(Buf::current().b_p_wm).unwrap_or(c_int::MAX);
        textwidth = win.w_view_width - margin;
        if cmdwin_buf.get() == Buf::current_or_none().map(Buf::id) {
            textwidth -= 1;
        }
        textwidth -= win_fdccol_count(win);
        textwidth -= win.w_scwidth;
        if win.w_onebuf_opt.wo_nu != 0 || win.w_onebuf_opt.wo_rnu != 0 {
            textwidth -= 8;
        }
    }
    textwidth = textwidth.max(0);
    if ff && textwidth == 0 {
        textwidth = (win.w_view_width - 1).min(79);
    }
    textwidth
}
