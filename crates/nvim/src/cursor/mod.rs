//! Moving and validating a window's cursor position.
//!
//! Everything here speaks in [`Win`], [`Buf`] and [`PosRef`] handles and
//! reads lines through [`Lines`]. Callers interleave these calls with reads
//! of the `curwin`/`curbuf` globals — which alias the same windows — and
//! several of them re-enter through `ml_replace` and the extmark bookkeeping,
//! so a walk that moves the window's own cursor runs on a copy and writes it
//! back ([`coladvance`], [`inc_cursor`]) rather than lending the window out.
//! `Win::text_width`, `Win::sidescrolloff` and `Win::validate_virtcol` come
//! from [`move`](crate::move) -- which owns `win_col_off()`, exactly as
//! `cursor.c` calls it out of `move.c`.
//!
//! The arithmetic that touches no pointer at all — the column clamps, the
//! 'wrap' target, the fold-skipping line count — lives in [`arith`], which
//! forbids unsafe outright and is what `tests/unit/cursor.rs` drives.

#![forbid(unsafe_code)]

use crate::option::vars::P_SEL;
use core::ffi::{c_char, c_int, c_uint};

pub mod arith;

use self::arith::{
    ColAdd, carried_coladd, checked_col, folded_line_span, gap_coladd, step_back, wrap_target_col,
};
use crate::change::inserted_bytes;
use crate::cstr;
use crate::drawscreen::UPD_NOT_VALID;
use crate::mark::mark_mb_adjustpos;
use crate::mbyte::{c_char_info, c_next_cluster, char_at, head_off};
use crate::memline::ml_replace_owned;
use crate::memline::{Lines, dec, inc, ml_get_len};
use crate::memory::XString;
use crate::r#move::{WinValid, changed_cline_bef_curs, set_valid_virtcol};
use crate::normal::{set_visual_anchor, visual_active, visual_anchor};
use crate::option::get_ve_flags;
use crate::options::{kOptVeFlagAll, kOptVeFlagOnemore};
use crate::plines::{getvcol, linetabsize, linetabsize_eol, win_charsize};
use crate::pos::MAXCOL;
use crate::state::mode::{State, restart_edit};
use crate::state::{MODE_INSERT, MODE_TERMINAL, virtual_active};
use crate::types::{CharsizeArg, ColNr, LineNr, NUL, Pos, int64_t};
use crate::winlayer::{Buf, PosRef, Win};

const TAB: c_int = 9;

// ---------------------------------------------------------------------------
// The window layer, as this module uses it
//
// [`Win`], [`Buf`] and [`PosRef`] are the shared wrappers, whose
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
    fn leftcol(self) -> ColNr {
        self.w_leftcol
    }

    #[inline(always)]
    fn set_leftcol(mut self, leftcol: ColNr) {
        self.w_leftcol = leftcol;
    }

    #[inline(always)]
    fn virtcol(self) -> ColNr {
        self.w_virtcol
    }

    #[inline(always)]
    fn set_curswant(mut self, curswant: ColNr) {
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
    fn note_virtcol(self, vcol: ColNr) {
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
    fn linetabsize(self, lnum: LineNr) -> c_int {
        linetabsize(self, lnum)
    }

    /// As [`Win::linetabsize`], but counting the room 'list' mode's `eol`
    /// character needs.
    #[inline(always)]
    fn linetabsize_eol(self, lnum: LineNr) -> c_int {
        linetabsize_eol(self, lnum)
    }
}

impl PosRef {
    #[inline(always)]
    fn lnum(self) -> LineNr {
        self.lnum
    }

    #[inline(always)]
    fn set_lnum(mut self, lnum: LineNr) {
        self.lnum = lnum;
    }

    #[inline(always)]
    fn col(self) -> ColNr {
        self.col
    }

    #[inline(always)]
    fn set_col(mut self, col: ColNr) {
        self.col = col;
    }

    #[inline(always)]
    fn coladd(self) -> ColNr {
        self.coladd
    }

    #[inline(always)]
    fn set_coladd(mut self, coladd: ColNr) {
        self.coladd = coladd;
    }
}

/// Whether 'selection' is `"old"`, where the cursor may not rest on the NUL.
#[inline(always)]
fn selection_is_old() -> bool {
    P_SEL.first_byte() == b'o'
}

// ---------------------------------------------------------------------------
// The public interface

/// Virtual column of the cursor, as `getvvcol` reports it (list mode off).
pub fn getviscol() -> ColNr {
    let win = Win::current();
    win.virtual_vcol(win.cursor())
}

/// Like [`getviscol`], but for an arbitrary position in the cursor's line.
pub fn getviscol2(col: ColNr, coladd: ColNr) -> ColNr {
    let win = Win::current();
    let pos = Pos {
        lnum: win.cursor().lnum(),
        col,
        coladd,
    };
    win.virtual_vcol_span_at(pos).0
}

/// Move the cursor to virtual column `wcol`, inserting the spaces needed to
/// land there exactly. Answers whether the column was reached.
pub fn coladvance_force(wcol: ColNr) -> bool {
    let win = Win::current();
    let reached = advance_cursor(win, true, false, wcol);
    if wcol == MAXCOL {
        win.invalidate_virtcol();
    } else {
        win.note_virtcol(wcol);
    }
    reached
}

/// Move `win`'s cursor to virtual column `wcol`, or as close as the line
/// allows. Answers whether the column was reached.
pub fn coladvance(win: Win, wcol: ColNr) -> bool {
    let reached = advance_cursor(win, false, win.virtual_active(), wcol);
    let cursor = win.w_cursor;
    // The cached virtual column is only good if the cursor did not land on a
    // tab, whose width depends on where it starts rather than on `wcol`.
    if wcol == MAXCOL || !reached {
        win.invalidate_virtcol();
    // The cursor's line, and a column `coladvance2` just clamped to it.
    } else if c_int::from(cstr::byte_at(
        win.buffer().lines().line(cursor.lnum),
        cursor.col as usize,
    )) != TAB
    {
        // The current window, not `win` — which is what the C does.
        Win::current().note_virtcol(wcol);
    }
    reached
}

/// [`coladvance2`] of `win`'s cursor.
///
/// The walk runs on a copy: it reads the window as it goes, so the window's
/// own cursor is not lent out for its length. `coladvance2` moves only the
/// column, and only the column is written back.
#[inline(always)]
fn advance_cursor(mut win: Win, addspaces: bool, finetune: bool, wcol: ColNr) -> bool {
    let mut at = win.w_cursor;
    let reached = coladvance2(win, &mut at, addspaces, finetune, wcol);
    win.w_cursor.col = at.col;
    win.w_cursor.coladd = at.coladd;
    reached
}

/// The shared body of [`coladvance`] and [`coladvance_force`].
///
/// `addspaces` fills the gap with real spaces when 'virtualedit' put the
/// cursor past the end of the line or inside a tab; `finetune` lets the
/// cursor stop part-way into a wide character. Answers whether `wcol_arg`
/// was reached.
fn coladvance2(win: Win, pos: &mut Pos, addspaces: bool, finetune: bool, wcol_arg: ColNr) -> bool {
    // Inserting the spaces edits the buffer, which only the current window
    // may do.
    debug_assert!(
        win.raw() == Win::current_raw() || !addspaces,
        "wp == curwin || !addspaces"
    );
    let mut wcol = wcol_arg;
    let one_more = State.get() & MODE_INSERT != 0
        || State.get() & MODE_TERMINAL != 0
        || restart_edit.get() != NUL
        || (visual_active() && !selection_is_old())
        || (win.ve_flags() & kOptVeFlagOnemore != 0 && wcol < MAXCOL);
    let buf = win.buffer();
    let lnum = pos.lnum;

    let mut idx;
    let mut col: ColNr = 0;
    let mut csize: c_int = 0;

    // MAXCOL is i32::MAX, so '>=' in the C was an equality test.
    if wcol == MAXCOL {
        idx = buf.line_len_raw(lnum) - 1 + one_more as c_int;
        col = wcol;
        if (addspaces || finetune) && !visual_active() {
            let want = win.linetabsize(lnum) + one_more as c_int;
            win.set_curswant(if want > 0 { want - 1 } else { want });
        }
    } else {
        let width = win.text_width();
        if finetune && win.wraps() && win.view_width() != 0 && wcol >= width && width > 0 {
            // With 'wrap', a column past this line's last screen line means
            // "the end of that screen line" rather than "past the line".
            // `csize` keeps this value if the walk below never runs.
            csize = win.linetabsize_eol(lnum);
            if csize > 0 {
                csize -= 1;
            }
            wcol = wrap_target_col(wcol, width, csize, State.get() & MODE_INSERT != 0);
        }

        let mut lines = buf.lines();
        let line = lines.line(lnum);
        let linelen = line.len() as c_int;
        let (mut csarg, kind) = CharsizeArg::new(win, lnum, line);
        let mut at = 0;
        let mut chr = c_char_info(line, 0);
        let mut head: c_int = 0;
        while col <= wcol && at < line.len() {
            let cs = win_charsize(kind, col, at, chr.value, &mut csarg);
            csize = cs.width;
            head = cs.head;
            col += cs.width;
            (at, chr) = c_next_cluster(line, at, chr);
        }
        idx = at as c_int;
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
            if cstr::byte_at(line, idx as usize) == 0 {
                // Past the end of the line: pad it out with spaces.
                let correct = wcol - col;
                let size = idx as i64 + correct as i64;
                assert!(size >= 0, "STRICT_ADD overflow");
                let newline = padded(line, size as usize, idx, correct, 0, 0);
                replace_padded(lnum, newline, idx, 0, correct);
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
                let newline = padded(line, size as usize, idx, csize, 1, tail);
                replace_padded(lnum, newline, idx, 1, csize);
                idx += csize - 1 + correct;
                col += correct;
            }
        }
    }

    pos.col = idx.max(0);
    pos.coladd = 0;

    if finetune {
        if wcol == MAXCOL {
            // The cursor is at the end of the line and may not sit on the NUL,
            // so `coladd` spans the last character instead.
            if !one_more {
                let (start, _, end) = getvcol(win, pos);
                pos.coladd = end - start;
            }
        } else {
            let gap = wcol - col;
            pos.coladd = gap_coladd(gap, win.view_width());
            col += gap;
        }
    }

    mark_mb_adjustpos(buf, pos);
    wcol >= 0 && col >= wcol
}

/// Line `line` rewritten as: its first `idx` bytes, then `spaces` spaces, then
/// `tail` of its bytes from `idx + skipped` on. `size` is the length the
/// caller sized, which is exactly that.
fn padded(
    line: &[u8],
    size: usize,
    idx: c_int,
    spaces: c_int,
    skipped: c_int,
    tail: c_int,
) -> XString {
    let (idx, spaces) = (idx as usize, spaces as usize);
    let from = idx + skipped as usize;
    let mut bytes = Vec::with_capacity(size + 1);
    bytes.extend_from_slice(&line[..idx]);
    bytes.resize(idx + spaces, b' ');
    bytes.extend_from_slice(&line[from..from + tail as usize]);
    debug_assert_eq!(bytes.len(), size, "the caller sized the padded line");
    XString::from_vec(bytes)
}

/// Put the line [`padded`] built in as line `lnum` of the current buffer.
fn replace_padded(lnum: LineNr, newline: XString, idx: c_int, skipped: c_int, spaces: c_int) {
    let _ = ml_replace_owned(lnum, newline);
    inserted_bytes(lnum, idx, skipped, spaces);
}

/// Set `pos` to the position at virtual column `wcol` in its own line,
/// without editing the buffer. Answers whether the column was reached.
pub fn getvpos(win: Win, pos: &mut Pos, wcol: ColNr) -> bool {
    coladvance2(win, pos, false, win.virtual_active(), wcol)
}

/// Move the cursor one character forward; see `inc`.
///
/// Safe: the only promise is that the editor exists, which `Win::current()`
/// carries; `inc` clamps the line it walks.
pub fn inc_cursor() -> c_int {
    let mut win = Win::current();
    let mut at = win.w_cursor;
    let moved = inc(&mut at);
    win.w_cursor = at;
    moved
}

/// Move the cursor one character back; see `dec`.
///
/// Safe: as [`inc_cursor`].
pub fn dec_cursor() -> c_int {
    let mut win = Win::current();
    let mut at = win.w_cursor;
    let moved = dec(&mut at);
    win.w_cursor = at;
    moved
}

/// How far `lnum` is from the cursor, counting each closed fold in between
/// as a single line.
///
pub fn get_cursor_rel_lnum(win: Win, lnum: LineNr) -> LineNr {
    let cursor = win.cursor().lnum();
    if lnum == cursor || !win.has_any_folding() {
        return lnum - cursor;
    }
    let span = folded_line_span(lnum.min(cursor), lnum.max(cursor), |line| {
        win.fold_last(line)
    });
    if lnum < cursor { -span } else { span }
}

/// Clamp `pos` to a line and column that exist in `buffer`.
pub fn check_pos(buffer: Buf, pos: &mut Pos) {
    pos.lnum = pos.lnum.min(buffer.line_count());
    if pos.col > 0 {
        pos.col = pos.col.min(buffer.line_len_raw(pos.lnum));
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
    let len = buf.line_len_raw(cursor.lnum());

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

/// Clamp the cursor to a position that exists in `window`'s buffer.
pub fn check_cursor(window: Win) {
    check_cursor_lnum(window);
    check_cursor_col(window);
}

/// Clamp the start of the Visual area to the current buffer.
pub fn check_visual_pos() {
    let visual = visual_anchor();
    let last = Buf::current().line_count();
    if visual.lnum > last {
        set_visual_anchor(Pos {
            lnum: last,
            col: 0,
            coladd: 0,
        });
    } else {
        let len = ml_get_len(visual.lnum);
        if visual.col > len {
            set_visual_anchor(Pos {
                col: len,
                coladd: 0,
                ..visual
            });
        }
    }
}

/// Step the cursor back off the NUL at the end of the line, where Normal
/// mode does not let it rest.
pub fn adjust_cursor_col() {
    let cursor = Win::current().cursor();
    if cursor.col() > 0 && (!visual_active() || selection_is_old()) && gchar_cursor() == NUL {
        cursor.set_col(cursor.col() - 1);
    }
}

/// Scroll the current window horizontally to `leftcol`, pulling the cursor
/// along if 'sidescrolloff' demands it. Answers whether the cursor moved.
pub fn set_leftcol(leftcol: ColNr) -> bool {
    let win = Win::current();
    if win.leftcol() == leftcol {
        return false;
    }
    win.set_leftcol(leftcol);
    win.cursor_line_changed();
    let lastcol = (win.leftcol() + win.text_width() - 1) as int64_t;
    win.validate_virtcol();

    let mut moved = false;
    let siso = win.sidescrolloff();
    if win.virtcol() > (lastcol - siso) as ColNr {
        moved = true;
        coladvance(win, (lastcol - siso) as ColNr);
    } else if (win.virtcol() as int64_t) < win.leftcol() as int64_t + siso {
        moved = true;
        coladvance(win, (win.leftcol() as int64_t + siso) as ColNr);
    }

    // A wide character straddling either edge is not fully visible; step the
    // cursor off it.
    let (start, end) = win.virtual_vcol_span(win.cursor());
    if end > lastcol as ColNr {
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
/// The cursor sitting inside its own line is the editor's invariant, not the
/// caller's -- `check_cursor_col` is what restores it; a column past the end
/// reads as the NUL there.
pub fn gchar_cursor() -> c_int {
    let cursor = Win::current().w_cursor;
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    char_at(line.get(cursor.col as usize..).unwrap_or_default())
}

/// The character before the cursor, or -1 at the start of the line.
pub fn char_before_cursor() -> c_int {
    let cursor = Win::current().w_cursor;
    if cursor.col == 0 {
        return -1;
    }
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    let col = (cursor.col as usize).min(line.len());
    if col == 0 {
        return -1;
    }
    let prev_len = head_off(line, col - 1) + 1;
    char_at(&line[col - prev_len..])
}

/// Overwrite the byte under the cursor.
pub fn pchar_cursor(c: c_char) {
    let cursor = Win::current().w_cursor;
    let mut lines = Lines::current();
    if let Some(byte) = lines.line_mut(cursor.lnum).get_mut(cursor.col as usize) {
        *byte = c as u8;
    }
}

/// The cursor's line.
///
/// Safe: the only promise is that the editor exists, which
/// `Win::current()`/`Buf::current()` carry. The answer is a raw pointer;
/// reading through it is still the caller's business.
pub fn get_cursor_line_ptr() -> *mut c_char {
    Buf::current()
        .line_raw(Win::current().cursor().lnum())
        .raw()
}

/// The cursor's line, from the cursor onwards.
///
/// Safe: as [`get_cursor_line_ptr`] -- an address, computed, not read.
pub fn get_cursor_pos_ptr() -> *mut c_char {
    let cursor = Win::current().cursor();
    Buf::current()
        .line_raw(cursor.lnum())
        .raw()
        .wrapping_offset(cursor.col() as isize)
}

/// The length of the cursor's line.
///
/// Safe: as [`get_cursor_line_ptr`].
pub fn get_cursor_line_len() -> ColNr {
    Buf::current().line_len_raw(Win::current().cursor().lnum())
}

/// The number of bytes from the cursor to the end of its line.
///
/// Safe: as [`get_cursor_line_ptr`].
pub fn get_cursor_pos_len() -> ColNr {
    let cursor = Win::current().cursor();
    Buf::current().line_len_raw(cursor.lnum()) - cursor.col()
}
