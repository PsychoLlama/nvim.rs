#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

//! Virtual columns of a position.
//!
//! The `getvcol` family: where a `Pos` lands on screen, once tabs, inline
//! virtual text, double-width characters and 'virtualedit' have had their
//! say. All of it walks the line with the parent module's charsize
//! functions.

use super::*;
use core::ptr;

use crate::normal::{visual_active, visual_anchor};
use crate::pos::MAXCOL;
use crate::types::NUL;
use crate::winlayer::PosRef;

/// Virtual column of `pos`, in up to three flavours:
///
/// * `start` — the first column the character occupies,
/// * `cursor` — where the cursor sits on it (the last column of a tab, in
///   Normal mode without 'list'),
/// * `end` — the last column it occupies.
///
/// Called very often; keep it fast. A `pos.col` past the end of the line is
/// clamped to the line length on the way out.
///
/// # Safety
/// `pos` must be live; the out-parameters may each be null.
pub(crate) unsafe fn getvcol(
    window: Win,
    pos: *mut Pos,
    start: *mut ColNr,
    cursor: *mut ColNr,
    end: *mut ColNr,
) {
    let line = unsafe { ml_get_buf(window.buffer(), (*pos).lnum) };
    let end_col = unsafe { (*pos).col };

    let mut csarg = CharsizeArg::default();
    let cstype = unsafe { init_charsize_arg(&mut csarg, window, (*pos).lnum, line) };
    csarg.max_head_vcol = -1;

    let mut on_nul = false;
    let mut vcol: ColNr = 0;
    let mut char_size;
    let mut ci: StrCharInfo = unsafe { utf_ptr2str_char_info(line) };

    if cstype == CharsizeKind::Fast {
        let use_tabstop = csarg.use_tabstop;
        loop {
            if unsafe { *ci.ptr } == NUL as c_char {
                // The cursor on a NUL is treated like a one-cell char.
                char_size = CharSize { width: 1, head: 0 };
                break;
            }
            char_size =
                unsafe { charsize_fast_impl(window, ci.ptr, use_tabstop, vcol, ci.chr.value) };
            let next = unsafe { utfc_next(ci) };
            if unsafe { next.ptr.offset_from(line) } > end_col as isize {
                break;
            }
            ci = next;
            vcol += char_size.width;
        }
    } else {
        loop {
            char_size = unsafe { charsize_regular(&mut csarg, ci.ptr, vcol, ci.chr.value) };
            // Don't go past the end of the line.
            if unsafe { *ci.ptr } == NUL as c_char {
                // A NUL at the end of the line takes one column, unless
                // there is virtual text.
                char_size.width = 1 + csarg.cur_text_width_left + csarg.cur_text_width_right;
                on_nul = true;
                break;
            }
            let next = unsafe { utfc_next(ci) };
            if unsafe { next.ptr.offset_from(line) } > end_col as isize {
                break;
            }
            ci = next;
            vcol += char_size.width;
        }
    }

    if unsafe { *ci.ptr } == NUL as c_char
        && end_col < MAXCOL
        && end_col as isize > unsafe { ci.ptr.offset_from(line) }
    {
        unsafe { (*pos).col = ci.ptr.offset_from(line) as ColNr };
    }

    let head = char_size.head;
    let incr = char_size.width;

    if !start.is_null() {
        unsafe { *start = vcol + head };
    }
    if !end.is_null() {
        unsafe { *end = vcol + incr - 1 };
    }
    if !cursor.is_null() {
        let cursor_at_tab_end = ci.chr.value == TAB
            && State.get() & MODE_NORMAL != 0
            && window.w_onebuf_opt.wo_list == 0
            && !virtual_active(window)
            && !(visual_active()
                && (unsafe { *p_sel.get() } == b'e' as c_char
                    || ltoreq(unsafe { *pos }, visual_anchor())));
        if cursor_at_tab_end {
            unsafe { *cursor = vcol + incr - 1 };
        } else {
            vcol += virt_text_cursor_off(&csarg, on_nul);
            unsafe { *cursor = vcol + head };
        }
    }
}

/// Virtual cursor column in the current window, pretending 'list' is off.
///
/// # Safety
/// `posp` must be live.
pub(crate) unsafe fn getvcol_nolist(posp: *mut Pos) -> ColNr {
    let mut win = Win::current();
    let list_save = win.w_onebuf_opt.wo_list;
    let mut vcol: ColNr = 0;
    let null = ::core::ptr::null_mut::<ColNr>();

    win.w_onebuf_opt.wo_list = 0;
    if unsafe { (*posp).coladd } != 0 {
        unsafe { getvvcol(win, posp, null, &raw mut vcol, null) };
    } else {
        unsafe { getvcol(win, posp, null, &raw mut vcol, null) };
    }
    win.w_onebuf_opt.wo_list = list_save;
    vcol
}

/// [`getvcol`] in virtual-edit mode, where the cursor can sit past the end of
/// a line or inside a tab.
///
/// # Safety
/// As [`getvcol`].
pub(crate) unsafe fn getvvcol(
    window: Win,
    pos: *mut Pos,
    start: *mut ColNr,
    cursor: *mut ColNr,
    end: *mut ColNr,
) {
    if !virtual_active(window) {
        unsafe { getvcol(window, pos, start, cursor, end) };
        return;
    }

    // In virtual mode only one value is wanted.
    let null = ::core::ptr::null_mut::<ColNr>();
    let mut col: ColNr = 0;
    unsafe { getvcol(window, pos, &raw mut col, null, null) };

    let mut coladd = unsafe { (*pos).coladd };
    let mut endadd: ColNr = 0;

    // The cursor cannot sit on part of a wide character.
    let ptr = unsafe { ml_get_buf(window.buffer(), (*pos).lnum) };
    if unsafe { (*pos).col } < unsafe { ml_get_buf_len(window.buffer(), (*pos).lnum) } {
        let c = unsafe { utf_ptr2char(ptr.offset((*pos).col as isize)) };
        if c != TAB && vim_isprintc(c) {
            endadd = unsafe { ptr2cells(ptr.offset((*pos).col as isize)) } - 1;
            if coladd > endadd {
                // Past the end of the line.
                endadd = 0;
            } else {
                coladd = 0;
            }
        }
    }
    col += coladd;

    if !start.is_null() {
        unsafe { *start = col };
    }
    if !cursor.is_null() {
        unsafe { *cursor = col };
    }
    if !end.is_null() {
        unsafe { *end = col + endadd };
    }
}

/// Leftmost and rightmost virtual column of `pos1` and `pos2`, for Visual
/// block mode.
///
/// # Safety
/// All pointers must be live; `left` and `right` are always written.
pub(crate) unsafe fn getvcols(
    window: Win,
    pos1: *mut Pos,
    pos2: *mut Pos,
    left: *mut ColNr,
    right: *mut ColNr,
) {
    let (first, second) = if lt(unsafe { *pos1 }, unsafe { *pos2 }) {
        (pos1, pos2)
    } else {
        (pos2, pos1)
    };

    let null = ::core::ptr::null_mut::<ColNr>();
    let mut from1: ColNr = 0;
    let mut from2: ColNr = 0;
    let mut to1: ColNr = 0;
    let mut to2: ColNr = 0;
    unsafe { getvvcol(window, first, &raw mut from1, null, &raw mut to1) };
    unsafe { getvvcol(window, second, &raw mut from2, null, &raw mut to2) };

    unsafe { *left = from1.min(from2) };
    // With 'selection' exclusive the block stops one column short of the
    // second position -- but only when that still leaves the first one's
    // last column inside the block.
    let before_second = from2 - 1;
    unsafe {
        *right = if to2 > to1 {
            if *p_sel.get() == b'e' as c_char && before_second >= to1 {
                before_second
            } else {
                to2
            }
        } else {
            to1
        }
    };
}

/// [`getvcol`]/[`getvvcol`] with the out-parameters as locals: the single
/// `unsafe` the [`Win`] projections below share.
///
/// `wanted` picks which of start, cursor and end to ask for. The C spells
/// "not this one" as a null pointer and skips the work for it, which is what
/// the callers below were passing by hand.
#[inline(always)]
fn columns(win: Win, pos: PosRef, virtual_edit: bool, wanted: [bool; 3]) -> [ColNr; 3] {
    let (mut start, mut cursor, mut end) = (0 as ColNr, 0 as ColNr, 0 as ColNr);
    let pick = |want: bool, p: *mut ColNr| if want { p } else { ptr::null_mut() };
    let (s, c, e) = (
        pick(wanted[0], &raw mut start),
        pick(wanted[1], &raw mut cursor),
        pick(wanted[2], &raw mut end),
    );
    // Two arms rather than one call through a function pointer: an indirect
    // call here is not inlined, and `getvcol` is on the cursor and statusline
    // paths -- it cost 1.3% of `scrbench` when this was `let get = if ...`.
    // SAFETY (both): a live window, a live position in its buffer, and three
    // out-parameters that are each null or a local of this frame.
    if virtual_edit {
        unsafe { getvvcol(win, pos.raw(), s, c, e) };
    } else {
        unsafe { getvcol(win, pos.raw(), s, c, e) };
    }
    [start, cursor, end]
}

/// The virtual-column questions a caller asks *of a window*, as [`Win`]
/// methods.
///
/// They live beside [`getvcol`] rather than in `winlayer` for the reason that
/// module's docs give: a family hangs its own projections off `impl Win`, so
/// the shared module stays the minimum.
impl Win {
    /// First and last virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_span(self, pos: PosRef) -> (ColNr, ColNr) {
        let c = columns(self, pos, false, [true, false, true]);
        (c[0], c[2])
    }

    /// Start, cursor and end virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_triple(self, pos: PosRef) -> (ColNr, ColNr, ColNr) {
        let c = columns(self, pos, false, [true, true, true]);
        (c[0], c[1], c[2])
    }

    /// The first virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol(self, pos: PosRef) -> ColNr {
        self.vcol_span(pos).0
    }

    /// [`Win::vcol_span`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_span(self, pos: PosRef) -> (ColNr, ColNr) {
        let c = columns(self, pos, true, [true, false, true]);
        (c[0], c[2])
    }

    /// [`Win::vcol_triple`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_triple(self, pos: PosRef) -> (ColNr, ColNr, ColNr) {
        let c = columns(self, pos, true, [true, true, true]);
        (c[0], c[1], c[2])
    }

    /// The first virtual column of the character at `pos`, 'virtualedit'
    /// included.
    #[inline(always)]
    pub fn virtual_vcol(self, pos: PosRef) -> ColNr {
        self.virtual_vcol_span(pos).0
    }

    /// The virtual column the *cursor* shows at within the character at
    /// `pos`, which is not its first column when the character is a tab.
    #[inline(always)]
    pub fn virtual_cursor_vcol(self, pos: PosRef) -> ColNr {
        columns(self, pos, true, [false, true, false])[1]
    }
}
