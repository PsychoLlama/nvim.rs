//! Moving and validating a window's cursor position.
//!
//! Everything here speaks in [`Win`], [`Buf`], [`Pos`] and [`Line`] rather
//! than in references. Callers interleave these calls with reads of the
//! `curwin`/`curbuf` globals — which alias the same windows — and several of
//! them re-enter through `ml_replace` and the extmark bookkeeping, so a
//! `&mut` here would invalidate a pointer the caller still holds.
//!
//! The pointers therefore stay raw, but the dereferences do not spread
//! through the module: each of the four wraps one of them and makes its
//! *construction* the unsafe step, which happens at the caller. Every
//! accessor, and every call into a neighbouring module, then rests on that
//! single promise, so the bodies below do not repeat it line by line and most
//! of them are safe functions. The four wrappers themselves live in
//! [`winlayer`](crate::winlayer), shared with the viewport code,
//! and `Win::text_width`, `Win::sidescrolloff` and `Win::validate_virtcol`
//! come from
//! [`move`](crate::move) -- which owns `win_col_off()`, exactly as
//! `cursor.c` calls it out of `move.c`.
//!
//! The arithmetic that touches no pointer at all — the column clamps, the
//! 'wrap' target, the fold-skipping line count — lives in [`arith`], which
//! forbids unsafe outright and is what `tests/unit/cursor.rs` drives.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

pub mod arith;

use self::arith::{
    ColAdd, carried_coladd, checked_col, folded_line_span, gap_coladd, step_back, wrap_target_col,
};
use crate::change::inserted_bytes;
use crate::drawscreen::UPD_NOT_VALID;
use crate::main::{State, curwin, p_sel, restart_edit};
use crate::mbyte::{utf_head_off, utf_ptr2char};
use crate::memline::{dec, inc, ml_get_len, ml_replace};
use crate::memory::xmallocz;
use crate::r#move::{WinValid, changed_cline_bef_curs, set_valid_virtcol};
use crate::normal::{set_visual_anchor, visual_active, visual_anchor};
use crate::option::get_ve_flags;
use crate::options::{kOptVeFlagAll, kOptVeFlagOnemore};
use crate::plines::{init_charsize_arg, linetabsize, linetabsize_eol, win_charsize};
use crate::pos::MAXCOL;
use crate::state::{MODE_INSERT, MODE_TERMINAL, virtual_active};
use crate::types::{
    CharSize, CharsizeArg, CharsizeKind, NUL, StrCharInfo, colnr_T, int64_t, linenr_T, pos_T,
};
use crate::winlayer::{Buf, Line, Pos, Win};

const TAB: c_int = 9;

// ---------------------------------------------------------------------------
// The window layer, as this module uses it
//
// [`Win`], [`Buf`], [`Pos`] and [`Line`] are the shared wrappers, whose
// constructors carry the whole promise; what follows are the projections only
// the cursor's own arithmetic asks for.

impl Win {
    /// Screen columns the window's text occupies, borders and 'number' column
    /// included.
    #[inline(always)]
    fn view_width(self) -> c_int {
        self.w_view_width
    }

    #[inline(always)]
    fn wraps(self) -> bool {
        self.w_onebuf_opt.wo_wrap != 0
    }

    #[inline(always)]
    fn leftcol(self) -> colnr_T {
        self.w_leftcol
    }

    #[inline(always)]
    fn set_leftcol(mut self, leftcol: colnr_T) {
        self.w_leftcol = leftcol;
    }

    #[inline(always)]
    fn virtcol(self) -> colnr_T {
        self.w_virtcol
    }

    #[inline(always)]
    fn set_curswant(mut self, curswant: colnr_T) {
        self.w_curswant = curswant;
    }

    /// Ask for `w_curswant` to be recomputed from the cursor's new position.
    #[inline(always)]
    fn recompute_curswant(mut self) {
        self.w_set_curswant = true;
    }

    #[inline(always)]
    fn invalidate_virtcol(mut self) {
        self.w_valid.clear(WinValid::VIRTCOL);
    }

    #[inline(always)]
    fn note_virtcol(self, vcol: colnr_T) {
        // SAFETY: a live window.
        set_valid_virtcol(self, vcol);
    }

    #[inline(always)]
    fn cursor_line_changed(self) {
        // SAFETY: a live window.
        changed_cline_bef_curs(self);
    }

    #[inline(always)]
    fn ve_flags(self) -> c_uint {
        // SAFETY: a live window.
        get_ve_flags(self)
    }

    #[inline(always)]
    fn virtual_active(self) -> bool {
        // SAFETY: a live window.
        virtual_active(self)
    }

    /// Virtual columns line `lnum` occupies.
    #[inline(always)]
    fn linetabsize(self, lnum: linenr_T) -> c_int {
        // SAFETY: a live window and a line of its buffer.
        unsafe { linetabsize(self, lnum) }
    }

    /// As [`Win::linetabsize`], but counting the room 'list' mode's `eol`
    /// character needs.
    #[inline(always)]
    fn linetabsize_eol(self, lnum: linenr_T) -> c_int {
        // SAFETY: a live window and a line of its buffer.
        unsafe { linetabsize_eol(self, lnum) }
    }

    /// Prepare to measure the characters of `line`, which must be line `lnum`
    /// of this window's buffer.
    #[inline(always)]
    fn measure(self, lnum: linenr_T, line: Line) -> Measure {
        let mut arg = CharsizeArg::default();
        // SAFETY: a live window, and `line` is its line `lnum`.
        let kind = unsafe { init_charsize_arg(&mut arg, self, lnum, line.raw()) };
        Measure { arg, kind }
    }
}

/// The per-line state `win_charsize` walks with, prepared by [`Win::measure`].
struct Measure {
    arg: CharsizeArg,
    kind: CharsizeKind,
}

impl Measure {
    /// Cells the character at `ci` takes, starting from virtual column `vcol`.
    #[inline(always)]
    fn char_size(&mut self, vcol: c_int, ci: StrCharInfo) -> CharSize {
        // SAFETY: `ci` is a character of the line `measure` was prepared for.
        unsafe { win_charsize(self.kind, vcol, ci.ptr, ci.chr.value, &mut self.arg) }
    }
}

impl Pos {
    #[inline(always)]
    fn lnum(self) -> linenr_T {
        self.lnum
    }

    #[inline(always)]
    fn set_lnum(mut self, lnum: linenr_T) {
        self.lnum = lnum;
    }

    #[inline(always)]
    fn col(self) -> colnr_T {
        self.col
    }

    #[inline(always)]
    fn set_col(mut self, col: colnr_T) {
        self.col = col;
    }

    #[inline(always)]
    fn coladd(self) -> colnr_T {
        self.coladd
    }

    #[inline(always)]
    fn set_coladd(mut self, coladd: colnr_T) {
        self.coladd = coladd;
    }
}

/// Whether 'selection' is `"old"`, where the cursor may not rest on the NUL.
#[inline(always)]
fn selection_is_old() -> bool {
    unsafe { *p_sel.get() == b'o' as c_char }
}

// ---------------------------------------------------------------------------
// The public interface

/// Virtual column of the cursor, as `getvvcol` reports it (list mode off).
///
/// # Safety
/// The current window must be valid.
pub unsafe fn getviscol() -> colnr_T {
    let win = unsafe { Win::current() };
    win.virtual_vcol(win.cursor())
}

/// Like [`getviscol`], but for an arbitrary position in the cursor's line.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn getviscol2(col: colnr_T, coladd: colnr_T) -> colnr_T {
    let win = unsafe { Win::current() };
    let mut pos = pos_T {
        lnum: win.cursor().lnum(),
        col,
        coladd,
    };
    win.virtual_vcol(unsafe { Pos::new(&raw mut pos) })
}

/// Move the cursor to virtual column `wcol`, inserting the spaces needed to
/// land there exactly. Answers whether the column was reached.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn coladvance_force(wcol: colnr_T) -> bool {
    let win = unsafe { Win::current() };
    let reached = unsafe { coladvance2(win, win.cursor(), true, false, wcol) };
    if wcol == MAXCOL {
        win.invalidate_virtcol();
    } else {
        win.note_virtcol(wcol);
    }
    reached
}

/// Move `win`'s cursor to virtual column `wcol`, or as close as the line
/// allows. Answers whether the column was reached.
pub fn coladvance(win: Win, wcol: colnr_T) -> bool {
    let cursor = win.cursor();
    // SAFETY: a window's cursor names a line of its own buffer -- the
    // invariant `check_cursor` maintains and every caller here leans on.
    let reached = unsafe { coladvance2(win, cursor, false, win.virtual_active(), wcol) };
    // The cached virtual column is only good if the cursor did not land on a
    // tab, whose width depends on where it starts rather than on `wcol`.
    if wcol == MAXCOL || !reached {
        win.invalidate_virtcol();
    // SAFETY: the cursor's line, and a column `coladvance2` just clamped to
    // it.
    } else if unsafe { win.buffer().line(cursor.lnum()).byte(cursor.col()) } as c_int != TAB {
        // The current window, not `win` — which is what the C does.
        // SAFETY: `curwin` is set from startup to exit.
        unsafe { Win::current() }.note_virtcol(wcol);
    }
    reached
}

/// The shared body of [`coladvance`] and [`coladvance_force`].
///
/// `addspaces` fills the gap with real spaces when 'virtualedit' put the
/// cursor past the end of the line or inside a tab; `finetune` lets the
/// cursor stop part-way into a wide character. Answers whether `wcol_arg`
/// was reached.
///
/// # Safety
/// `pos` must name a line of `win`'s buffer.
unsafe fn coladvance2(
    win: Win,
    pos: Pos,
    addspaces: bool,
    finetune: bool,
    wcol_arg: colnr_T,
) -> bool {
    // Inserting the spaces edits the buffer, which only the current window
    // may do.
    debug_assert!(
        win.raw() == curwin.get() || !addspaces,
        "wp == curwin || !addspaces"
    );
    let mut wcol = wcol_arg;
    let one_more = State.get() & MODE_INSERT != 0
        || State.get() & MODE_TERMINAL != 0
        || restart_edit.get() != NUL
        || (visual_active() && !selection_is_old())
        || (win.ve_flags() & kOptVeFlagOnemore != 0 && wcol < MAXCOL);
    let buf = win.buffer();
    let line = unsafe { buf.line(pos.lnum()) };
    let linelen = unsafe { buf.line_len(pos.lnum()) };

    let mut idx;
    let mut col: colnr_T = 0;
    let mut csize: c_int = 0;

    // MAXCOL is i32::MAX, so '>=' in the C was an equality test.
    if wcol == MAXCOL {
        idx = linelen - 1 + one_more as c_int;
        col = wcol;
        if (addspaces || finetune) && !visual_active() {
            let want = win.linetabsize(pos.lnum()) + one_more as c_int;
            win.set_curswant(if want > 0 { want - 1 } else { want });
        }
    } else {
        let width = win.text_width();
        if finetune && win.wraps() && win.view_width() != 0 && wcol >= width && width > 0 {
            // With 'wrap', a column past this line's last screen line means
            // "the end of that screen line" rather than "past the line".
            // `csize` keeps this value if the walk below never runs.
            csize = win.linetabsize_eol(pos.lnum());
            if csize > 0 {
                csize -= 1;
            }
            wcol = wrap_target_col(wcol, width, csize, State.get() & MODE_INSERT != 0);
        }

        let mut measure = win.measure(pos.lnum(), line);
        let mut ci = line.first_char();
        let mut head: c_int = 0;
        while col <= wcol && !unsafe { line.ended(ci) } {
            let cs = measure.char_size(col, ci);
            csize = cs.width;
            head = cs.head;
            col += cs.width;
            ci = unsafe { line.next_char(ci) };
        }
        idx = line.index_of(ci);
        // The walk stepped one character too far, unless it stopped on the
        // NUL and the cursor is allowed to rest there.
        if col > wcol || (!win.virtual_active() && !one_more) {
            (idx, col, csize) = step_back(idx, col, csize, head);
        }

        if win.virtual_active()
            && addspaces
            && wcol >= 0
            && ((col != wcol && col != wcol + 1) || csize > 1)
        {
            if unsafe { line.byte(idx) } == 0 {
                // Past the end of the line: pad it out with spaces.
                let correct = wcol - col;
                let size = idx as i64 + correct as i64;
                assert!(size >= 0, "STRICT_ADD overflow");
                unsafe { pad_line(pos.lnum(), line, size as usize, idx, correct, 0, 0) };
                idx += correct;
                col = wcol;
            } else {
                // Inside a wide character (a tab, normally): replace it with
                // the spaces it occupied and land among them.
                let correct = wcol - col - csize + 1;
                if -correct > csize {
                    return false;
                }
                let size = (linelen - 1) as i64 + csize as i64;
                assert!(size >= 0, "STRICT_ADD overflow");
                assert!(linelen - idx >= 1, "STRICT_SUB overflow");
                let tail = linelen - idx - 1;
                unsafe { pad_line(pos.lnum(), line, size as usize, idx, csize, 1, tail) };
                idx += csize - 1 + correct;
                col += correct;
            }
        }
    }

    pos.set_col(idx.max(0));
    pos.set_coladd(0);

    if finetune {
        if wcol == MAXCOL {
            // The cursor is at the end of the line and may not sit on the NUL,
            // so `coladd` spans the last character instead.
            if !one_more {
                let (start, end) = win.vcol_span(pos);
                pos.set_coladd(end - start);
            }
        } else {
            let gap = wcol - col;
            pos.set_coladd(gap_coladd(gap, win.view_width()));
            col += gap;
        }
    }

    buf.snap_to_char(pos);
    wcol >= 0 && col >= wcol
}

/// Rewrite line `lnum` of the current buffer as: its first `idx` bytes, then
/// `spaces` spaces, then its bytes from `idx + skipped` onwards, of which
/// there are `tail`. `size` is the allocation the caller sized.
///
/// # Safety
/// `line` must be line `lnum` of the current buffer, `idx` must be within it,
/// and `size` must be at least `idx + spaces + tail`.
unsafe fn pad_line(
    lnum: linenr_T,
    line: Line,
    size: usize,
    idx: c_int,
    spaces: c_int,
    skipped: c_int,
    tail: c_int,
) {
    unsafe {
        let newline = xmallocz(size).cast::<c_char>();
        ptr::copy_nonoverlapping(line.raw(), newline, idx as usize);
        ptr::write_bytes(newline.offset(idx as isize), b' ', spaces as usize);
        ptr::copy_nonoverlapping(
            line.raw().offset((idx + skipped) as isize),
            newline.offset((idx + spaces) as isize),
            tail as usize,
        );
        let _ = ml_replace(lnum, newline, false);
        inserted_bytes(lnum, idx, skipped, spaces);
    }
}

/// Set `pos` to the position at virtual column `wcol` in its own line,
/// without editing the buffer. Answers whether the column was reached.
///
/// # Safety
/// `pos` must name a line of `win`'s buffer.
pub unsafe fn getvpos(win: Win, pos: Pos, wcol: colnr_T) -> bool {
    // SAFETY: the caller's promise, forwarded.
    unsafe { coladvance2(win, pos, false, win.virtual_active(), wcol) }
}

/// Move the cursor one character forward; see `inc`.
///
/// Safe: the only promise is that the editor exists, which `Win::current()`
/// carries; `inc` clamps the line it walks.
pub fn inc_cursor() -> c_int {
    unsafe { inc(&mut *Win::current().cursor().raw()) }
}

/// Move the cursor one character back; see `dec`.
///
/// Safe: as [`inc_cursor`].
pub fn dec_cursor() -> c_int {
    unsafe { dec(&mut *Win::current().cursor().raw()) }
}

/// How far `lnum` is from the cursor, counting each closed fold in between
/// as a single line.
///
pub fn get_cursor_rel_lnum(win: Win, lnum: linenr_T) -> linenr_T {
    let cursor = win.cursor().lnum();
    if lnum == cursor || !win.has_any_folding() {
        return lnum - cursor;
    }
    let span = folded_line_span(lnum.min(cursor), lnum.max(cursor), |line| {
        win.fold_last(line)
    });
    if lnum < cursor { -span } else { span }
}

/// Clamp `pos` to a line and column that exist in `buf`.
pub fn check_pos(buf: Buf, pos: &mut pos_T) {
    pos.lnum = pos.lnum.min(buf.line_count());
    if pos.col > 0 {
        // SAFETY: `lnum` was just clamped to a line the buffer has.
        pos.col = pos.col.min(unsafe { buf.line_len(pos.lnum) });
    }
}

/// Clamp the cursor's line number to the buffer, preferring the start of a
/// closed fold over a line inside it.
///
pub fn check_cursor_lnum(win: Win) {
    let cursor = win.cursor();
    let last = win.buffer().line_count();
    if cursor.lnum() > last {
        // With a closed fold at the end of the buffer, the cursor belongs on
        // its first line rather than on the buffer's last.
        cursor.set_lnum(win.fold_first(last).unwrap_or(last));
    }
    if cursor.lnum() <= 0 {
        cursor.set_lnum(1);
    }
}

/// Clamp the cursor's column to the current line, honouring the modes and
/// 'virtualedit' settings that allow it one position past the last character.
///
pub fn check_cursor_col(win: Win) {
    let buf = win.buffer();
    let cursor = win.cursor();
    let oldcol = cursor.col();
    let oldcoladd = cursor.col() + cursor.coladd();
    let cur_ve_flags = win.ve_flags();
    // SAFETY: a window's cursor names a line of its own buffer, which is what
    // `check_cursor_lnum` -- the other half of `check_cursor` -- maintains.
    let len = unsafe { buf.line_len(cursor.lnum()) };

    let (col, snap) = checked_col(oldcol, len, || {
        State.get() & MODE_INSERT != 0
            || restart_edit.get() != 0
            || State.get() & MODE_TERMINAL != 0
            || (visual_active() && !selection_is_old())
            || cur_ve_flags & kOptVeFlagOnemore != 0
            || win.virtual_active()
    });
    cursor.set_col(col);
    if snap {
        buf.snap_to_char(cursor);
    }

    match carried_coladd(
        oldcol,
        oldcoladd,
        cursor.col(),
        cur_ve_flags == kOptVeFlagAll,
    ) {
        ColAdd::Keep => {}
        ColAdd::Zero => cursor.set_coladd(0),
        ColAdd::Carry(coladd) => {
            cursor.set_coladd(coladd);
            // Don't let the cursor point past the character it is inside.
            if cursor.col() + 1 < len {
                debug_assert!(coladd > 0, "win->w_cursor.coladd > 0");
                let (start, end) = win.vcol_span(cursor);
                cursor.set_coladd(coladd.min(end - start));
            }
        }
    }
}

/// Clamp the cursor to a position that exists in `wp`'s buffer.
pub fn check_cursor(wp: Win) {
    check_cursor_lnum(wp);
    check_cursor_col(wp);
}

/// Clamp the start of the Visual area to the current buffer.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn check_visual_pos() {
    let visual = visual_anchor();
    let last = unsafe { Buf::current() }.line_count();
    if visual.lnum > last {
        set_visual_anchor(pos_T {
            lnum: last,
            col: 0,
            coladd: 0,
        });
    } else {
        let len = ml_get_len(visual.lnum);
        if visual.col > len {
            set_visual_anchor(pos_T {
                col: len,
                coladd: 0,
                ..visual
            });
        }
    }
}

/// Step the cursor back off the NUL at the end of the line, where Normal
/// mode does not let it rest.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn adjust_cursor_col() {
    let cursor = unsafe { Win::current() }.cursor();
    if cursor.col() > 0 && (!visual_active() || selection_is_old()) && gchar_cursor() == NUL {
        cursor.set_col(cursor.col() - 1);
    }
}

/// Scroll the current window horizontally to `leftcol`, pulling the cursor
/// along if 'sidescrolloff' demands it. Answers whether the cursor moved.
///
/// # Safety
/// The current window must be valid.
pub unsafe fn set_leftcol(leftcol: colnr_T) -> bool {
    let win = unsafe { Win::current() };
    if win.leftcol() == leftcol {
        return false;
    }
    win.set_leftcol(leftcol);
    win.cursor_line_changed();
    let lastcol = (win.leftcol() + win.text_width() - 1) as int64_t;
    win.validate_virtcol();

    let mut moved = false;
    let siso = win.sidescrolloff();
    if win.virtcol() > (lastcol - siso) as colnr_T {
        moved = true;
        coladvance(win, (lastcol - siso) as colnr_T);
    } else if (win.virtcol() as int64_t) < win.leftcol() as int64_t + siso {
        moved = true;
        coladvance(win, (win.leftcol() as int64_t + siso) as colnr_T);
    }

    // A wide character straddling either edge is not fully visible; step the
    // cursor off it.
    let (start, end) = win.virtual_vcol_span(win.cursor());
    if end > lastcol as colnr_T {
        moved = true;
        coladvance(win, start - 1);
    } else if start < win.leftcol() {
        moved = true;
        if !coladvance(win, end + 1) {
            // There is nothing to move onto; keep the character visible by
            // scrolling to it instead.
            win.set_leftcol(start);
            win.cursor_line_changed();
        }
    }

    if moved {
        win.recompute_curswant();
    }
    win.redraw_later(UPD_NOT_VALID);
    moved
}

/// The character under the cursor.
///
/// Safe: the only promise is that the editor exists. The cursor sitting
/// inside its own line is the editor's invariant, not the caller's --
/// `check_cursor_col` is what restores it.
pub fn gchar_cursor() -> c_int {
    unsafe { utf_ptr2char(get_cursor_pos_ptr()) }
}

/// The character before the cursor, or -1 at the start of the line.
///
/// # Safety
/// The current window and buffer must be valid.
pub unsafe fn char_before_cursor() -> c_int {
    let col = unsafe { Win::current() }.cursor().col();
    if col == 0 {
        return -1;
    }
    unsafe {
        let line = get_cursor_line_ptr();
        let p = line.offset(col as isize);
        let prev_len = utf_head_off(line, p.offset(-1)) + 1;
        utf_ptr2char(p.offset(-(prev_len as isize)))
    }
}

/// Overwrite the byte under the cursor.
///
/// # Safety
/// The current window and buffer must be valid, and the cursor's column must
/// lie within the line.
pub unsafe fn pchar_cursor(c: c_char) {
    let cursor = unsafe { Win::current() }.cursor();
    unsafe {
        *Buf::current()
            .line_mut(cursor.lnum())
            .raw()
            .offset(cursor.col() as isize) = c;
    }
}

/// The cursor's line.
///
/// Safe: the only promise is that the editor exists, which
/// `Win::current()`/`Buf::current()` carry. The answer is a raw pointer;
/// reading through it is still the caller's business.
pub fn get_cursor_line_ptr() -> *mut c_char {
    unsafe { Buf::current().line(Win::current().cursor().lnum()).raw() }
}

/// The cursor's line, from the cursor onwards.
///
/// Safe: as [`get_cursor_line_ptr`] -- an address, computed, not read.
pub fn get_cursor_pos_ptr() -> *mut c_char {
    let cursor = unsafe { Win::current() }.cursor();
    unsafe {
        Buf::current()
            .line(cursor.lnum())
            .raw()
            .offset(cursor.col() as isize)
    }
}

/// The length of the cursor's line.
///
/// Safe: as [`get_cursor_line_ptr`].
pub fn get_cursor_line_len() -> colnr_T {
    unsafe { Buf::current().line_len(Win::current().cursor().lnum()) }
}

/// The number of bytes from the cursor to the end of its line.
///
/// Safe: as [`get_cursor_line_ptr`].
pub fn get_cursor_pos_len() -> colnr_T {
    let cursor = unsafe { Win::current() }.cursor();
    unsafe { Buf::current().line_len(cursor.lnum()) - cursor.col() }
}
