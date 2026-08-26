//! Visual mode: entering and leaving it, and the area two corners describe.
//!
//! The off-by-one rules live here. 'selection' decides whether the character
//! under the far end is part of the selection; `adjust_for_sel` moves the
//! cursor one on so an exclusive selection covers what it looks like it
//! covers, and `unadjust_for_sel` puts that back before anything reads the
//! area again.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ops::Op;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::cursor::{
    adjust_cursor_col, check_cursor, coladvance, gchar_cursor, get_cursor_line_len,
    get_cursor_line_ptr, inc_cursor,
};
use crate::drawscreen::{
    UPD_INVERTED, UPD_VALID, conceal_check_cursor_line, redraw_curbuf_later, showmode,
};
use crate::fold::fold_adjust_visual;
use crate::getchar::{beep_flush, stuff_empty, typeahead};
use crate::global_cell::GlobalCell;
use crate::main::{
    VIsual_reselect, VIsual_select_exclu_adj, VIsual_select_reg, curbuf, curwin, finish_op,
    motion_force, mouse_dragging, msg_silent, p_sel, p_slm, p_smd, redraw_cmdline,
    resel_VIsual_line_count, resel_VIsual_mode, resel_VIsual_vcol,
};
use crate::mark::mark_mb_adjustpos;
use crate::mbyte::utfc_ptr2len;
use crate::memline::{ml_get_len, ml_get_pos};
use crate::mouse::setmouse;
use crate::normal::{
    CA_NO_ADJ_OP_END, CmdArg, TAB, VIsual_mode_orig, clear_op_beep, may_clear_cmdline, nv_down,
    nv_g_cmd, nv_operator, nv_right,
};
use crate::ops::adjust_cursor_eol;
use crate::option::get_ve_flags;
use crate::options::kOptVeFlagBlock;
use crate::plines::{getvcol, getvcols};
use crate::pos::{MAXCOL, equalpos, lt};
use crate::state::{may_trigger_modechanged, virtual_active};
use crate::strings::vim_strchr;
use crate::textobject::{
    current_block, current_par, current_quote, current_sent, current_tagblock, current_word,
};
use crate::types::{NUL, OP_NOP, cmdarg_T, colnr_T, linenr_T, pos_T, size_t};
use core::ffi::{c_char, c_int, c_uint};

use crate::keycodes::{Ctrl_Q, Ctrl_V};
use crate::r#move::{update_curswant_force, update_topline, validate_virtcol};

/// Whether 'selection' is "exclusive": the character under the far end of the
/// selection is not part of it.
#[inline(always)]
pub(crate) fn sel_exclusive() -> bool {
    // SAFETY: 'selection' is a non-empty C string option.
    unsafe { *p_sel.get() as c_int == 'e' as c_int }
}

// ---------------------------------------------------------------------------
// The selection itself

/// The kind of area a selection covers: `v` charwise, `V` linewise, CTRL-V
/// blockwise.
///
/// A newtype over the character rather than an enum, because the value space
/// is not closed. `b_visual.vi_mode` is read back verbatim from an undo file
/// (`undo/file.rs`, `undo_read_4c`) and `gv` copies it into the live mode, so
/// any `c_int` is representable; upstream's chain of equality tests falls
/// through to charwise for anything else, and so does this.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct VisualMode(c_int);

impl VisualMode {
    /// Charwise, entered with `v`.
    pub(crate) const CHAR: Self = Self(b'v' as c_int);
    /// Linewise, entered with `V`.
    pub(crate) const LINE: Self = Self(b'V' as c_int);
    /// Blockwise, entered with CTRL-V.
    pub(crate) const BLOCK: Self = Self(Ctrl_V);
    /// The `NUL` that `resel_VIsual_mode` and `VIsual_mode_orig` use to mean
    /// "nothing remembered".
    pub(crate) const NONE: Self = Self(NUL);

    /// The mode a stored character names.
    pub(crate) const fn from_raw(mode: c_int) -> Self {
        Self(mode)
    }

    /// The character `b_visual.vi_mode`, `v:` variables and the undo file
    /// store.
    pub(crate) const fn raw(self) -> c_int {
        self.0
    }

    pub(crate) fn is_char(self) -> bool {
        self == Self::CHAR
    }

    pub(crate) fn is_line(self) -> bool {
        self == Self::LINE
    }

    pub(crate) fn is_block(self) -> bool {
        self == Self::BLOCK
    }
}

/// A Visual or Select selection: one of its two ends, and what kind of area
/// the pair covers.
///
/// The other end is the cursor, which belongs to the window. Holding one of
/// these is what it takes to read the anchor, and [`visual_selection`] is the
/// only thing that makes one -- which is the whole point: upstream leaves
/// `VIsual` set after a selection ends and guards every read of it with a
/// separate `VIsual_active`, an invariant no reader is obliged to honour.
#[derive(Clone, Copy)]
pub(crate) struct VisualSelection {
    /// Upstream's `VIsual`: where the selection was started.
    pub(crate) anchor: pos_T,
    pub(crate) mode: VisualMode,
    /// Select mode rather than Visual mode: printable input replaces the
    /// selection instead of being read as a command.
    pub(crate) select: bool,
}

/// `VIsual`, `VIsual_active`, `VIsual_mode` and `VIsual_select`, which
/// upstream keeps as four unrelated globals that have to agree.
#[derive(Clone, Copy)]
struct VisualState {
    /// Upstream's `VIsual_active`.
    active: bool,
    /// The selection the other three globals describe.
    ///
    /// It outlives `active` on purpose, because upstream's does: ending a
    /// selection only lowers the flag, and a dozen places lower it and put it
    /// back around code that must not see a selection (a `:normal` inside a
    /// statusline expression, the autocommand window, a scroll) and expect
    /// the anchor to still be there afterwards. Only [`visual_selection`]
    /// hands the three out together, and only while `active`.
    sel: VisualSelection,
}

/// The Visual selection, live or last.
static VISUAL: GlobalCell<VisualState> = GlobalCell::new(VisualState {
    active: false,
    sel: VisualSelection {
        anchor: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        mode: VisualMode::CHAR,
        select: false,
    },
});

/// The selection, or `None` when there is none.
///
/// This is the read to prefer: it cannot answer an anchor that no selection
/// is using.
pub(crate) fn visual_selection() -> Option<VisualSelection> {
    let visual = VISUAL.get();
    visual.active.then_some(visual.sel)
}

/// Whether a Visual or Select selection is up (upstream's `VIsual_active`).
pub(crate) fn visual_active() -> bool {
    VISUAL.get().active
}

/// Raise or lower the flag without disturbing the selection under it.
///
/// The savers -- `switchwin`, `aucmd_prepbuf`, the statusline, `:normal`,
/// paste, a page scroll -- restore the flag alone, so this deliberately does
/// not take a whole selection.
pub(crate) fn set_visual_active(active: bool) {
    VISUAL.set(VisualState {
        active,
        ..VISUAL.get()
    });
}

/// The kind of the selection, live or last (upstream's `VIsual_mode`).
pub(crate) fn visual_mode() -> VisualMode {
    VISUAL.get().sel.mode
}

pub(crate) fn set_visual_mode(mode: VisualMode) {
    let mut visual = VISUAL.get();
    visual.sel.mode = mode;
    VISUAL.set(visual);
}

/// Whether the selection is a Select-mode one (upstream's `VIsual_select`).
pub(crate) fn visual_select() -> bool {
    VISUAL.get().sel.select
}

pub(crate) fn set_visual_select(select: bool) {
    let mut visual = VISUAL.get();
    visual.sel.select = select;
    VISUAL.set(visual);
}

/// The stored anchor, whether or not a selection is up.
///
/// Upstream's bare `VIsual`. Prefer [`visual_selection`], which cannot hand
/// back an anchor nothing is selecting; this is for the callers that reach
/// the anchor with the flag tested somewhere further up the call stack.
pub(crate) fn visual_anchor() -> pos_T {
    VISUAL.get().sel.anchor
}

pub(crate) fn set_visual_anchor(anchor: pos_T) {
    let mut visual = VISUAL.get();
    visual.sel.anchor = anchor;
    VISUAL.set(visual);
}

/// Whether any selection has been made since startup.
///
/// `reg_match_visual` (`\%V`) asks this as `VIsual.lnum == 0`, and asks it of
/// the *global* anchor rather than of the current buffer's remembered area --
/// so `\%V` in a buffer that has never been in Visual mode still matches
/// nothing until some buffer has.
pub(crate) fn visual_ever_started() -> bool {
    VISUAL.get().sel.anchor.lnum != 0
}

/// Read-modify-write of the anchor.
///
/// The anchor is handed to `f` as a *copy* and put back afterwards, rather
/// than borrowed out of the cell: the callers run buffer code and, through
/// `has_folding`, 'foldexpr' -- user code that reads the same state -- so a
/// borrow held across them would be reentrant.
pub(crate) fn with_visual_anchor<R>(f: impl FnOnce(&mut pos_T) -> R) -> R {
    let mut anchor = visual_anchor();
    let r = f(&mut anchor);
    set_visual_anchor(anchor);
    r
}

// ---------------------------------------------------------------------------
// Entering and leaving

/// Leave Visual mode, remembering the selection for `gv` and `'<`/`'>`.
pub(crate) fn end_visual_mode() {
    VIsual_select_exclu_adj.set(false);
    set_visual_active(false);
    // SAFETY: all of this is the current buffer's and window's own state.
    setmouse();
    mouse_dragging.set(0);
    cur_buf().b_visual.vi_mode = visual_mode().raw();
    cur_buf().b_visual.vi_start = visual_anchor();
    cur_buf().b_visual.vi_end = cur_win().w_cursor;
    cur_buf().b_visual.vi_curswant = cur_win().w_curswant;
    cur_buf().b_visual_mode_eval = visual_mode().raw();
    if !unsafe { virtual_active(curwin.get()) } {
        cur_win().w_cursor.coladd = 0;
    }
    may_clear_cmdline();
    unsafe { adjust_cursor_eol() };
    unsafe { may_trigger_modechanged() };
}

/// Leave Visual mode and forget the selection, so `gv` will not bring it back.
pub(crate) fn reset_VIsual_and_resel() {
    if visual_active() {
        end_visual_mode();
        // SAFETY: schedules a redraw of the current buffer.
        unsafe { redraw_curbuf_later(UPD_INVERTED) };
    }
    VIsual_reselect.set(0);
}

/// As [`reset_VIsual_and_resel`], but only when there was a selection.
pub(crate) fn reset_VIsual() {
    if visual_active() {
        end_visual_mode();
        // SAFETY: schedules a redraw of the current buffer.
        unsafe { redraw_curbuf_later(UPD_INVERTED) };
        VIsual_reselect.set(0);
    }
}

/// Put back the Visual mode `v_visop` forced to linewise for an uppercase
/// operator.
pub(crate) fn restore_visual_mode() {
    if VIsual_mode_orig.get() != VisualMode::NONE {
        // SAFETY: `curbuf` is the current buffer.
        cur_buf().b_visual.vi_mode = VIsual_mode_orig.get().raw();
        VIsual_mode_orig.set(VisualMode::NONE);
    }
}

/// The text the Visual selection covers, for a command that wants it as a
/// string rather than as an operator target.
///
/// Refuses -- and beeps, when it was given an operator to clear -- for a
/// selection spanning more than one line. Leaves Visual mode either way it
/// succeeds.
pub(crate) unsafe fn get_visual_text(
    cap: *mut cmdarg_T,
    pp: *mut *mut c_char,
    lenp: *mut size_t,
) -> bool {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if !visual_mode().is_line() {
        // SAFETY: adjusts the current window's cursor or `VIsual`.
        unsafe { unadjust_for_sel() };
    }
    let anchor = visual_anchor();
    // SAFETY: `cap` is null or the caller's live command argument, and `pp`
    // and `lenp` are its out-parameters.
    if anchor.lnum != cur_win().w_cursor.lnum {
        if !cap.is_null() {
            clear_op_beep(ca.op());
        }
        return false;
    }
    if visual_mode().is_line() {
        unsafe { *pp = get_cursor_line_ptr() };
        unsafe { *lenp = get_cursor_line_len() as size_t };
    } else {
        // The earlier of the two ends is the start; the length is the
        // column difference, inclusive.
        if lt(cur_win().w_cursor, anchor) {
            unsafe { *pp = ml_get_pos(&raw mut (*curwin.get()).w_cursor) };
            unsafe { *lenp = (anchor.col - cur_win().w_cursor.col + 1) as size_t };
        } else {
            unsafe { *pp = ml_get_pos(&raw const anchor) };
            unsafe { *lenp = (cur_win().w_cursor.col - anchor.col + 1) as size_t };
        }
        if unsafe { **pp } as c_int == NUL {
            unsafe { *lenp = 0 };
        }
        // The last character may be multibyte; take the rest of it.
        //
        // `utfc_ptr2len` answers 0 for a NUL, and upstream adds `0 - 1`
        // as a `size_t` -- which wraps and so takes one *off* the length.
        // Reachable: a blockwise selection whose last line is short ends
        // on the terminator. Kept wrapping, deliberately.
        if unsafe { *lenp } > 0 {
            let tail = unsafe { utfc_ptr2len((*pp).add(*lenp - 1)) };
            unsafe { *lenp = (*lenp).wrapping_add((tail - 1) as size_t) };
        }
    }
    reset_VIsual_and_resel();
    true
}

/// Swap the two ends of the selection.
///
/// `o` swaps them outright. `O` in blockwise mode swaps only the *columns*,
/// which means moving both ends -- and the second half of this only runs when
/// the first attempt left the cursor where it started, which happens when the
/// two columns are the same width.
pub(crate) unsafe fn v_swap_corners(cmdchar: c_int) {
    // Only the blockwise `O` path below reads this; the charwise path returns
    // first, having set the anchor itself.
    let mut anchor = visual_anchor();
    // SAFETY: `curwin` is the current window and `VIsual` a live position.
    if cmdchar != 'O' as c_int || !visual_mode().is_block() {
        let old_cursor = cur_win().w_cursor;
        cur_win().w_cursor = visual_anchor();
        set_visual_anchor(old_cursor);
        cur_win().w_set_curswant = true;
        return;
    }

    let (mut left, mut right): (colnr_T, colnr_T) = (0, 0);
    let mut old_cursor = cur_win().w_cursor;
    let win = cur_win();
    let (from, to) = (&raw mut old_cursor, &raw mut anchor);
    let (l, r) = (&raw mut left, &raw mut right);
    unsafe { getvcols(win.raw(), from, to, l, r) };
    cur_win().w_cursor.lnum = visual_anchor().lnum;
    unsafe { coladvance(curwin.get(), left) };
    set_visual_anchor(cur_win().w_cursor);
    cur_win().w_cursor.lnum = old_cursor.lnum;
    cur_win().w_curswant = right;
    // An exclusive selection ends one past the last column it covers.
    if old_cursor.lnum >= visual_anchor().lnum && sel_exclusive() {
        cur_win().w_curswant += 1;
    }
    unsafe { coladvance(curwin.get(), cur_win().w_curswant) };

    // Nothing moved: the block's two columns are the same width, so swap
    // them the other way round instead.
    if cur_win().w_cursor.col == old_cursor.col
        && (!unsafe { virtual_active(curwin.get()) }
            || cur_win().w_cursor.coladd == old_cursor.coladd)
    {
        cur_win().w_cursor.lnum = visual_anchor().lnum;
        if old_cursor.lnum <= visual_anchor().lnum && sel_exclusive() {
            right += 1;
        }
        unsafe { coladvance(curwin.get(), right) };
        set_visual_anchor(cur_win().w_cursor);
        cur_win().w_cursor.lnum = old_cursor.lnum;
        unsafe { coladvance(curwin.get(), left) };
        cur_win().w_curswant = left;
    }
}

/// An operator typed in Visual mode, as the pairs of "what was typed" and
/// "what it means".
///
/// Upstream spells this as the string `"YyDdCcxdXdAAIIrr"` and finds the
/// character with `strchr`, taking the byte after it.
const VISUAL_OPS: [(u8, u8); 8] = [
    (b'Y', b'y'),
    (b'D', b'd'),
    (b'C', b'c'),
    (b'x', b'd'),
    (b'X', b'd'),
    (b'A', b'A'),
    (b'I', b'I'),
    (b'r', b'r'),
];

/// Run an operator typed in Visual mode.
///
/// An uppercase one forces the selection linewise -- except in blockwise
/// mode, where `C` and `D` instead extend every line to its end.
pub(crate) unsafe fn v_visop(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.cmdchar >= 'A' as c_int && ca.cmdchar <= 'Z' as c_int {
        if !visual_mode().is_block() {
            VIsual_mode_orig.set(visual_mode());
            set_visual_mode(VisualMode::LINE);
        } else if ca.cmdchar == 'C' as c_int || ca.cmdchar == 'D' as c_int {
            cur_win().w_curswant = MAXCOL as colnr_T;
        }
    }
    let typed = ca.cmdchar as u8;
    ca.cmdchar = VISUAL_OPS
        .iter()
        .find(|(from, _)| *from == typed)
        .expect("v_visop is only reached for a character in VISUAL_OPS")
        .1 as c_int;
    unsafe { nv_operator(cap) };
}

/// Reselect the previous selection, `count` times as large.
///
/// Only reached with a count: `3v` means "three times whatever was selected
/// last". The line count and the column count multiply separately, which is
/// why the charwise and blockwise cases are spelled out.
unsafe fn reselect_scaled(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    set_visual_anchor(cur_win().w_cursor);
    set_visual_active(true);
    VIsual_reselect.set(1);
    if ca.arg == 0 {
        may_start_select('c' as c_int);
    }
    setmouse();
    if p_smd.get() != 0 && msg_silent.get() == 0 {
        redraw_cmdline.set(true);
    }
    // The count multiplies the size of the remembered selection, and it
    // is user input: `999999999v` after a three-column selection
    // overflows. Upstream does this arithmetic in C, where it wraps, and
    // `check_cursor`/`coladvance` clamp whatever comes out -- so wrapping
    // is both what the C produces and safe. The transpile used Rust's
    // checked operators here and aborted the debug build instead.
    if !resel_VIsual_mode.get().is_char() || resel_VIsual_line_count.get() > 1 {
        cur_win().w_cursor.lnum = cur_win().w_cursor.lnum.wrapping_add(
            resel_VIsual_line_count
                .get()
                .wrapping_mul(ca.count0 as linenr_T)
                .wrapping_sub(1),
        );
        unsafe { check_cursor(curwin.get()) };
    }
    set_visual_mode(resel_VIsual_mode.get());

    if visual_mode().is_char() {
        if resel_VIsual_line_count.get() <= 1 {
            unsafe { update_curswant_force() };
            let count0 = ca.count0;
            let extra = resel_VIsual_vcol.get().wrapping_mul(count0) as colnr_T;
            cur_win().w_curswant = cur_win().w_curswant.wrapping_add(extra);
            if !sel_exclusive() {
                cur_win().w_curswant -= 1;
            }
        } else {
            cur_win().w_curswant = resel_VIsual_vcol.get();
        }
        unsafe { coladvance(curwin.get(), cur_win().w_curswant) };
    }

    if resel_VIsual_vcol.get() == MAXCOL as c_int {
        cur_win().w_curswant = MAXCOL as colnr_T;
        unsafe { coladvance(curwin.get(), MAXCOL as c_int) };
    } else if visual_mode().is_block() {
        // The width is measured from the *start* line, so the cursor goes
        // there while 'curswant' is recomputed and comes back after.
        let lnum = cur_win().w_cursor.lnum;
        cur_win().w_cursor.lnum = visual_anchor().lnum;
        unsafe { update_curswant_force() };
        cur_win().w_curswant = cur_win().w_curswant.wrapping_add(
            resel_VIsual_vcol
                .get()
                .wrapping_mul(ca.count0)
                .wrapping_sub(1) as colnr_T,
        );
        cur_win().w_cursor.lnum = lnum;
        if sel_exclusive() {
            cur_win().w_curswant += 1;
        }
        unsafe { coladvance(curwin.get(), cur_win().w_curswant) };
    } else {
        cur_win().w_set_curswant = true;
    }
    unsafe { redraw_curbuf_later(UPD_INVERTED) };
}

/// `v`, `V`, `CTRL-V` and their Select-mode twins.
///
/// Keeps the raw signature: this is an `nv_cmds` row's handler, so `nv_func_T`
/// fixes it.
pub(crate) unsafe fn nv_visual(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.cmdchar == Ctrl_Q {
        ca.cmdchar = Ctrl_V;
    }
    // After an operator these are not commands but a forced motion kind:
    // `dv`, `dV`, `d CTRL-V`.
    if ca.op().op_type != OP_NOP {
        ca.op().motion_force = ca.cmdchar;
        motion_force.set(ca.op().motion_force);
        finish_op.set(false);
        return;
    }

    set_visual_select(ca.arg != 0);
    if visual_active() {
        // The same key again leaves Visual mode; a different one switches
        // to that kind of selection.
        if visual_mode() == VisualMode::from_raw(ca.cmdchar) {
            end_visual_mode();
        } else {
            set_visual_mode(VisualMode::from_raw(ca.cmdchar));
            unsafe { showmode() };
            unsafe { may_trigger_modechanged() };
        }
        unsafe { redraw_curbuf_later(UPD_INVERTED) };
    } else if ca.count0 > 0 && resel_VIsual_mode.get() != VisualMode::NONE {
        unsafe { reselect_scaled(cap) };
    } else {
        if ca.arg == 0 {
            may_start_select('c' as c_int);
        }
        unsafe { n_start_visual_mode(ca.cmdchar) };
        // An exclusive selection needs one more character to cover the
        // same text, so the count is raised before it is spent.
        if !visual_mode().is_line() && sel_exclusive() {
            ca.count1 += 1;
        } else {
            VIsual_select_exclu_adj.set(false);
        }
        // A count means "select this many characters or lines".
        if ca.count0 > 0 && {
            ca.count1 -= 1;
            ca.count1 > 0
        } {
            if visual_mode().is_char() || visual_mode().is_block() {
                unsafe { nv_right(cap) };
            } else if visual_mode().is_line() {
                unsafe { nv_down(cap) };
            }
        }
    }
}

/// Start a charwise selection because a shifted key was pressed.
pub(crate) fn start_selection() {
    may_start_select('k' as c_int);
    // SAFETY: enters Visual mode on the current window.
    unsafe { n_start_visual_mode('v' as c_int) };
}

/// Decide between Visual and Select mode for a selection about to start.
///
/// `c` says how it is starting -- 'k'ey, 'm'ouse or 'c'ommand -- and
/// 'selectmode' says which of those mean Select. A command-started selection
/// only counts as typed when nothing is being replayed.
pub(crate) fn may_start_select(c: c_int) {
    // SAFETY: 'selectmode' is a C string option.
    let by_selectmode = !unsafe { vim_strchr(p_slm.get(), c) }.is_null();
    let typed = c == 'o' as c_int || (stuff_empty() && typeahead().maplen() == 0);
    set_visual_select(typed && by_selectmode);
}

/// Enter Visual mode of kind `c` at the cursor.
pub(crate) unsafe fn n_start_visual_mode(c: c_int) {
    set_visual_mode(VisualMode::from_raw(c));
    set_visual_active(true);
    VIsual_reselect.set(1);
    // SAFETY: `curwin` is the current window.
    // A block selection starting inside a TAB starts at the column the
    // cursor is displayed at, not at the TAB's first column.
    if c == Ctrl_V
        && unsafe { get_ve_flags(curwin.get()) } & kOptVeFlagBlock as c_int as c_uint != 0
        && unsafe { gchar_cursor() } == TAB
    {
        unsafe { validate_virtcol(curwin.get()) };
        unsafe { coladvance(curwin.get(), cur_win().w_virtcol) };
    }
    set_visual_anchor(cur_win().w_cursor);
    unsafe { fold_adjust_visual() };
    unsafe { may_trigger_modechanged() };
    setmouse();
    unsafe { conceal_check_cursor_line() };
    if p_smd.get() != 0 && msg_silent.get() == 0 {
        redraw_cmdline.set(true);
    }
    // Seed the "what was highlighted last time" pair so the first redraw
    // has something to compare against.
    if cur_win().w_redr_type < UPD_INVERTED {
        cur_win().w_old_cursor_lnum = cur_win().w_cursor.lnum;
        cur_win().w_old_visual_lnum = cur_win().w_cursor.lnum;
    }
    unsafe { redraw_curbuf_later(UPD_VALID) };
}

/// `gv`: select what was selected last.
///
/// Doing it while a selection is up *swaps* the two, so `gv` twice comes back
/// where it started.
pub(crate) unsafe fn nv_gv_cmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let vi = unsafe { &raw mut (*curbuf.get()).b_visual };
    if unsafe { *vi }.vi_start.lnum == 0
        || unsafe { *vi }.vi_start.lnum > cur_buf().b_ml.ml_line_count
        || unsafe { *vi }.vi_end.lnum == 0
    {
        unsafe { beep_flush() };
        return;
    }

    let tpos;
    if visual_active() {
        let mode = visual_mode();
        set_visual_mode(VisualMode::from_raw(unsafe { *vi }.vi_mode));
        unsafe { *vi }.vi_mode = mode.raw();
        cur_buf().b_visual_mode_eval = mode.raw();
        let curswant = cur_win().w_curswant;
        cur_win().w_curswant = unsafe { *vi }.vi_curswant;
        unsafe { *vi }.vi_curswant = curswant;
        tpos = unsafe { *vi }.vi_end;
        unsafe { *vi }.vi_end = cur_win().w_cursor;
        cur_win().w_cursor = unsafe { *vi }.vi_start;
        unsafe { *vi }.vi_start = visual_anchor();
    } else {
        set_visual_mode(VisualMode::from_raw(unsafe { *vi }.vi_mode));
        cur_win().w_curswant = unsafe { *vi }.vi_curswant;
        tpos = unsafe { *vi }.vi_end;
        cur_win().w_cursor = unsafe { *vi }.vi_start;
    }

    set_visual_active(true);
    VIsual_reselect.set(1);
    // Both ends are checked against the buffer: it may have shrunk since.
    unsafe { check_cursor(curwin.get()) };
    set_visual_anchor(cur_win().w_cursor);
    cur_win().w_cursor = tpos;
    unsafe { check_cursor(curwin.get()) };
    unsafe { update_topline(curwin.get()) };
    if ca.arg != 0 {
        set_visual_select(true);
        VIsual_select_reg.set(0);
    } else {
        may_start_select('c' as c_int);
    }
    setmouse();
    unsafe { redraw_curbuf_later(UPD_INVERTED) };
    unsafe { showmode() };
}

/// Make an exclusive selection cover the character the cursor is on, so the
/// operator about to run sees what the highlight showed.
pub(crate) unsafe fn adjust_for_sel(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_active()
        && ca.op().inclusive
        && sel_exclusive()
        && unsafe { gchar_cursor() } != NUL
        && lt(visual_anchor(), cur_win().w_cursor)
    {
        unsafe { inc_cursor() };
        ca.op().inclusive = false;
        VIsual_select_exclu_adj.set(true);
    }
}

/// Undo [`adjust_for_sel`] on whichever end is the later one.
///
/// Answers whether the position moved to the previous line.
pub(crate) unsafe fn unadjust_for_sel() -> bool {
    // SAFETY (throughout): `curwin` is the current window and `VIsual` a live position.
    if sel_exclusive() && !equalpos(visual_anchor(), cur_win().w_cursor) {
        if lt(visual_anchor(), cur_win().w_cursor) {
            return unsafe { unadjust_for_sel_inner(&raw mut (*curwin.get()).w_cursor) };
        }
        return with_visual_anchor(|anchor| unsafe { unadjust_for_sel_inner(anchor) });
    }
    false
}

/// Move one position back, across a line break if there is nothing else left.
///
/// Answers whether it crossed one.
pub(crate) unsafe fn unadjust_for_sel_inner(pp: *mut pos_T) -> bool {
    VIsual_select_exclu_adj.set(false);
    // SAFETY: `pp` is the caller's live position in the current buffer.
    if unsafe { *pp }.coladd > 0 {
        unsafe { *pp }.coladd -= 1;
    } else if unsafe { *pp }.col > 0 {
        unsafe { *pp }.col -= 1;
        unsafe { mark_mb_adjustpos(curbuf.get(), pp) };
        // Inside a TAB, stepping back a byte means stepping to the last
        // screen column the TAB covers.
        if unsafe { virtual_active(curwin.get()) } {
            let (mut cs, mut ce): (colnr_T, colnr_T) = (0, 0);
            unsafe { getvcol(curwin.get(), pp, &raw mut cs, ptr::null_mut(), &raw mut ce) };
            unsafe { *pp }.coladd = ce - cs;
        }
    } else if unsafe { *pp }.lnum > 1 {
        unsafe { *pp }.lnum -= 1;
        unsafe { *pp }.col = unsafe { ml_get_len((*pp).lnum) };
        return true;
    }
    false
}

/// `gh`, `gH`, `g CTRL-H`: Select mode, either fresh or from a reselection.
pub(crate) unsafe fn nv_select(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_active() {
        set_visual_select(true);
        VIsual_select_reg.set(0);
    } else if VIsual_reselect.get() != 0 {
        // Re-enter through `gv`, which is where the reselection lives.
        // SAFETY: `cap` is the caller's live command argument.
        ca.nchar = 'v' as c_int;
        ca.arg = 1;
        unsafe { nv_g_cmd(cap) };
    }
}

/// A text object: `iw`, `a(`, `it` and the rest.
///
/// 'matchpairs' is forced to the four bracket pairs for the duration, because
/// a text object's idea of a block is fixed and must not follow the option.
pub(crate) unsafe fn nv_object(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let include = ca.cmdchar != 'i' as c_int;
    let mps_save = cur_buf().b_p_mps;
    cur_buf().b_p_mps = c"(:),{:},[:],<:>".as_ptr().cast_mut();

    let mut op = ca.op();
    let n = ca.count1;
    let found = match u8::try_from(ca.nchar).unwrap_or(0) {
        b'w' => unsafe { current_word(op.raw(), n, include, false) != 0 },
        b'W' => unsafe { current_word(op.raw(), n, include, true) != 0 },
        b'b' | b'(' | b')' => block(op, n, include, '(', ')'),
        b'B' | b'{' | b'}' => block(op, n, include, '{', '}'),
        b'[' | b']' => block(op, n, include, '[', ']'),
        b'<' | b'>' => block(op, n, include, '<', '>'),
        b't' => {
            // A tag block's end is already where it should be; the
            // operator must not push it back over the closing tag.
            ca.retval |= CA_NO_ADJ_OP_END as c_int;
            unsafe { current_tagblock(op.raw(), n, include) != 0 }
        }
        b'p' => unsafe { current_par(op.raw(), n, include, 'p' as c_int) != 0 },
        b's' => unsafe { current_sent(op.raw(), n, include) != 0 },
        b'"' | b'\'' | b'`' => unsafe { current_quote(op.raw(), n, include, ca.nchar) },
        _ => false,
    };

    cur_buf().b_p_mps = mps_save;
    if !found {
        clear_op_beep(op);
    }
    unsafe { adjust_cursor_col() };
    cur_win().w_set_curswant = true;
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The `i(`/`a{`-family text object: the block `open`..`close` around the
/// cursor, `n` levels out.
fn block(op: Op, n: c_int, include: bool, open: char, close: char) -> bool {
    // SAFETY: `op` is a live operator and the cursor is in its own buffer.
    unsafe { current_block(op.raw(), n, include, open as c_int, close as c_int) != 0 }
}
