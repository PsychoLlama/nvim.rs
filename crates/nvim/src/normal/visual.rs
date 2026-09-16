//! Visual mode: entering and leaving it, and the area two corners describe.
//!
//! The off-by-one rules live here. 'selection' decides whether the character
//! under the far end is part of the selection; `adjust_for_sel` moves the
//! cursor one on so an exclusive selection covers what it looks like it
//! covers, and `unadjust_for_sel` puts that back before anything reads the
//! area again.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::ops::Op;
use crate::strings::has_char;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::cursor::{
    adjust_cursor_col, check_cursor, coladvance, gchar_cursor, get_cursor_line_len,
    get_cursor_line_ptr, inc_cursor,
};
use crate::drawscreen::state::redraw_cmdline;
use crate::drawscreen::{
    UPD_INVERTED, UPD_VALID, conceal_check_cursor_line, redraw_curbuf_later, showmode,
};
use crate::fold::fold_adjust_visual;
use crate::getchar::{beep_flush, stuff_empty, typeahead};
use crate::global_cell::GlobalCell;
use crate::mark::mark_mb_adjustpos;
use crate::mbyte::utfc_ptr2len;
use crate::memline::{ml_get_len, ml_get_pos};
use crate::message::state::msg_silent;
use crate::mouse::setmouse;
use crate::mouse::state::mouse_dragging;
use crate::normal::{
    CA_NO_ADJ_OP_END, TAB, VIsual_mode_orig, clear_op_beep, may_clear_cmdline, nv_down, nv_g_cmd,
    nv_operator, nv_right,
};
use crate::ops::adjust_cursor_eol;
use crate::option::get_ve_flags;
use crate::option::vars::{p_sel, p_slm, p_smd};
use crate::options::kOptVeFlagBlock;
use crate::plines::{getvcol, getvcols};
use crate::pos::{MAXCOL, equalpos, lt};
use crate::state::mode::{
    VIsual_reselect, VIsual_select_exclu_adj, VIsual_select_reg, finish_op, motion_force,
    resel_VIsual_line_count, resel_VIsual_mode, resel_VIsual_vcol,
};
use crate::state::{may_trigger_modechanged, virtual_active};
use crate::textobject::{
    current_block, current_par, current_quote, current_sent, current_tagblock, current_word,
};
use crate::types::{CmdArg, ColNr, LineNr, NUL, OpType, Pos, size_t};
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
    pub(crate) anchor: Pos,
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
        anchor: Pos {
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
pub(crate) fn visual_anchor() -> Pos {
    VISUAL.get().sel.anchor
}

pub(crate) fn set_visual_anchor(anchor: Pos) {
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
pub(crate) fn with_visual_anchor<R>(f: impl FnOnce(&mut Pos) -> R) -> R {
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
    Buf::current().b_visual.vi_mode = visual_mode().raw();
    Buf::current().b_visual.vi_start = visual_anchor();
    Buf::current().b_visual.vi_end = Win::current().w_cursor;
    Buf::current().b_visual.vi_curswant = Win::current().w_curswant;
    Buf::current().b_visual_mode_eval = visual_mode().raw();
    if !virtual_active(Win::current()) {
        Win::current().w_cursor.coladd = 0;
    }
    may_clear_cmdline();
    adjust_cursor_eol();
    may_trigger_modechanged();
}

/// Leave Visual mode and forget the selection, so `gv` will not bring it back.
pub(crate) fn reset_visual_and_resel() {
    if visual_active() {
        end_visual_mode();
        // SAFETY: schedules a redraw of the current buffer.
        redraw_curbuf_later(UPD_INVERTED);
    }
    VIsual_reselect.set(0);
}

/// As [`reset_visual_and_resel`], but only when there was a selection.
pub(crate) fn reset_visual() {
    if visual_active() {
        end_visual_mode();
        // SAFETY: schedules a redraw of the current buffer.
        redraw_curbuf_later(UPD_INVERTED);
        VIsual_reselect.set(0);
    }
}

/// Put back the Visual mode `v_visop` forced to linewise for an uppercase
/// operator.
pub(crate) fn restore_visual_mode() {
    if VIsual_mode_orig.get() != VisualMode::NONE {
        // SAFETY: `curbuf` is the current buffer.
        Buf::current().b_visual.vi_mode = VIsual_mode_orig.get().raw();
        VIsual_mode_orig.set(VisualMode::NONE);
    }
}

/// The text the Visual selection covers, for a command that wants it as a
/// string rather than as an operator target.
///
/// Refuses -- and beeps, when it was given an operator to clear -- for a
/// selection spanning more than one line. Leaves Visual mode either way it
/// succeeds.
///
/// # Safety
///
/// `cursor` must point at a writable `*mut c_char` slot the caller owns for
/// the call. `lenp` must point at a writable `size_t` the caller owns.
pub(crate) unsafe fn get_visual_text(
    cmd_arg: Option<&mut CmdArg>,
    cursor: *mut *mut c_char,
    lenp: *mut size_t,
) -> bool {
    if !visual_mode().is_line() {
        // SAFETY: adjusts the current window's cursor or `VIsual`.
        unadjust_for_sel();
    }
    let anchor = visual_anchor();
    if anchor.lnum != Win::current().w_cursor.lnum {
        // A selection spanning lines is not a name; whoever asked for one
        // through a command gets the refusal.
        if let Some(command) = cmd_arg {
            clear_op_beep(command.op());
        }
        return false;
    }
    if visual_mode().is_line() {
        unsafe { *cursor = get_cursor_line_ptr() };
        unsafe { *lenp = get_cursor_line_len() as size_t };
    } else {
        // The earlier of the two ends is the start; the length is the
        // column difference, inclusive.
        if lt(Win::current().w_cursor, anchor) {
            unsafe { *cursor = ml_get_pos(&raw mut (*Win::current_raw()).w_cursor) };
            unsafe { *lenp = (anchor.col - Win::current().w_cursor.col + 1) as size_t };
        } else {
            unsafe { *cursor = ml_get_pos(&raw const anchor) };
            unsafe { *lenp = (Win::current().w_cursor.col - anchor.col + 1) as size_t };
        }
        if unsafe { **cursor } as c_int == NUL {
            unsafe { *lenp = 0 };
        }
        // The last character may be multibyte; take the rest of it.
        //
        // `utfc_ptr2len` answers 0 for a NUL, and upstream adds `0 - 1`
        // as a `size_t` -- which wraps and so takes one *off* the length.
        // Reachable: a blockwise selection whose last line is short ends
        // on the terminator. Kept wrapping, deliberately.
        if unsafe { *lenp } > 0 {
            let tail = unsafe { utfc_ptr2len((*cursor).add(*lenp - 1)) };
            unsafe { *lenp = (*lenp).wrapping_add((tail - 1) as size_t) };
        }
    }
    reset_visual_and_resel();
    true
}

/// Swap the two ends of the selection.
///
/// `o` swaps them outright. `O` in blockwise mode swaps only the *columns*,
/// which means moving both ends -- and the second half of this only runs when
/// the first attempt left the cursor where it started, which happens when the
/// two columns are the same width.
pub(crate) fn v_swap_corners(cmdchar: c_int) {
    // Only the blockwise `O` path below reads this; the charwise path returns
    // first, having set the anchor itself.
    let mut anchor = visual_anchor();
    // SAFETY: `curwin` is the current window and `VIsual` a live position.
    if cmdchar != 'O' as c_int || !visual_mode().is_block() {
        let old_cursor = Win::current().w_cursor;
        Win::current().w_cursor = visual_anchor();
        set_visual_anchor(old_cursor);
        Win::current().w_set_curswant = true;
        return;
    }

    let (mut left, mut right): (ColNr, ColNr) = (0, 0);
    let mut old_cursor = Win::current().w_cursor;
    let win = Win::current();
    let (from, to) = (&raw mut old_cursor, &raw mut anchor);
    let (l, r) = (&raw mut left, &raw mut right);
    unsafe { getvcols(win, from, to, l, r) };
    Win::current().w_cursor.lnum = visual_anchor().lnum;
    coladvance(Win::current(), left);
    set_visual_anchor(Win::current().w_cursor);
    Win::current().w_cursor.lnum = old_cursor.lnum;
    Win::current().w_curswant = right;
    // An exclusive selection ends one past the last column it covers.
    if old_cursor.lnum >= visual_anchor().lnum && sel_exclusive() {
        Win::current().w_curswant += 1;
    }
    coladvance(Win::current(), Win::current().w_curswant);

    // Nothing moved: the block's two columns are the same width, so swap
    // them the other way round instead.
    if Win::current().w_cursor.col == old_cursor.col
        && (!virtual_active(Win::current()) || Win::current().w_cursor.coladd == old_cursor.coladd)
    {
        Win::current().w_cursor.lnum = visual_anchor().lnum;
        if old_cursor.lnum <= visual_anchor().lnum && sel_exclusive() {
            right += 1;
        }
        coladvance(Win::current(), right);
        set_visual_anchor(Win::current().w_cursor);
        Win::current().w_cursor.lnum = old_cursor.lnum;
        coladvance(Win::current(), left);
        Win::current().w_curswant = left;
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
pub(crate) fn v_visop(cmd_arg: &mut CmdArg) {
    if cmd_arg.cmdchar >= 'A' as c_int && cmd_arg.cmdchar <= 'Z' as c_int {
        if !visual_mode().is_block() {
            VIsual_mode_orig.set(visual_mode());
            set_visual_mode(VisualMode::LINE);
        } else if cmd_arg.cmdchar == 'C' as c_int || cmd_arg.cmdchar == 'D' as c_int {
            Win::current().w_curswant = MAXCOL as ColNr;
        }
    }
    let typed = cmd_arg.cmdchar as u8;
    cmd_arg.cmdchar = VISUAL_OPS
        .iter()
        .find(|(from, _)| *from == typed)
        .expect("v_visop is only reached for a character in VISUAL_OPS")
        .1 as c_int;
    nv_operator(cmd_arg);
}

/// Reselect the previous selection, `count` times as large.
///
/// Only reached with a count: `3v` means "three times whatever was selected
/// last". The line count and the column count multiply separately, which is
/// why the charwise and blockwise cases are spelled out.
fn reselect_scaled(cmd_arg: &mut CmdArg) {
    set_visual_anchor(Win::current().w_cursor);
    set_visual_active(true);
    VIsual_reselect.set(1);
    if cmd_arg.arg == 0 {
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
        Win::current().w_cursor.lnum = Win::current().w_cursor.lnum.wrapping_add(
            resel_VIsual_line_count
                .get()
                .wrapping_mul(cmd_arg.count0 as LineNr)
                .wrapping_sub(1),
        );
        check_cursor(Win::current());
    }
    set_visual_mode(resel_VIsual_mode.get());

    if visual_mode().is_char() {
        if resel_VIsual_line_count.get() <= 1 {
            update_curswant_force();
            let count0 = cmd_arg.count0;
            let extra = resel_VIsual_vcol.get().wrapping_mul(count0) as ColNr;
            Win::current().w_curswant = Win::current().w_curswant.wrapping_add(extra);
            if !sel_exclusive() {
                Win::current().w_curswant -= 1;
            }
        } else {
            Win::current().w_curswant = resel_VIsual_vcol.get();
        }
        coladvance(Win::current(), Win::current().w_curswant);
    }

    if resel_VIsual_vcol.get() == MAXCOL as c_int {
        Win::current().w_curswant = MAXCOL as ColNr;
        coladvance(Win::current(), MAXCOL as c_int);
    } else if visual_mode().is_block() {
        // The width is measured from the *start* line, so the cursor goes
        // there while 'curswant' is recomputed and comes back after.
        let lnum = Win::current().w_cursor.lnum;
        Win::current().w_cursor.lnum = visual_anchor().lnum;
        update_curswant_force();
        Win::current().w_curswant = Win::current().w_curswant.wrapping_add(
            resel_VIsual_vcol
                .get()
                .wrapping_mul(cmd_arg.count0)
                .wrapping_sub(1) as ColNr,
        );
        Win::current().w_cursor.lnum = lnum;
        if sel_exclusive() {
            Win::current().w_curswant += 1;
        }
        coladvance(Win::current(), Win::current().w_curswant);
    } else {
        Win::current().w_set_curswant = true;
    }
    redraw_curbuf_later(UPD_INVERTED);
}

/// `v`, `V`, `CTRL-V` and their Select-mode twins.
///
/// Keeps the raw signature: this is an `nv_cmds` row's handler, so `NvFunc`
/// fixes it.
pub(crate) fn nv_visual(cmd_arg: &mut CmdArg) {
    if cmd_arg.cmdchar == Ctrl_Q {
        cmd_arg.cmdchar = Ctrl_V;
    }
    // After an operator these are not commands but a forced motion kind:
    // `dv`, `dV`, `d CTRL-V`.
    if cmd_arg.op().op_type != OpType::Nop {
        cmd_arg.op().motion_force = cmd_arg.cmdchar;
        motion_force.set(cmd_arg.op().motion_force);
        finish_op.set(false);
        return;
    }

    set_visual_select(cmd_arg.arg != 0);
    if visual_active() {
        // The same key again leaves Visual mode; a different one switches
        // to that kind of selection.
        if visual_mode() == VisualMode::from_raw(cmd_arg.cmdchar) {
            end_visual_mode();
        } else {
            set_visual_mode(VisualMode::from_raw(cmd_arg.cmdchar));
            showmode();
            may_trigger_modechanged();
        }
        redraw_curbuf_later(UPD_INVERTED);
    } else if cmd_arg.count0 > 0 && resel_VIsual_mode.get() != VisualMode::NONE {
        reselect_scaled(cmd_arg);
    } else {
        if cmd_arg.arg == 0 {
            may_start_select('c' as c_int);
        }
        n_start_visual_mode(cmd_arg.cmdchar);
        // An exclusive selection needs one more character to cover the
        // same text, so the count is raised before it is spent.
        if !visual_mode().is_line() && sel_exclusive() {
            cmd_arg.count1 += 1;
        } else {
            VIsual_select_exclu_adj.set(false);
        }
        // A count means "select this many characters or lines".
        if cmd_arg.count0 > 0 && {
            cmd_arg.count1 -= 1;
            cmd_arg.count1 > 0
        } {
            if visual_mode().is_char() || visual_mode().is_block() {
                nv_right(cmd_arg);
            } else if visual_mode().is_line() {
                nv_down(cmd_arg);
            }
        }
    }
}

/// Start a charwise selection because a shifted key was pressed.
pub(crate) fn start_selection() {
    may_start_select('k' as c_int);
    n_start_visual_mode('v' as c_int);
}

/// Decide between Visual and Select mode for a selection about to start.
///
/// `c` says how it is starting -- 'k'ey, 'm'ouse or 'c'ommand -- and
/// 'selectmode' says which of those mean Select. A command-started selection
/// only counts as typed when nothing is being replayed.
pub(crate) fn may_start_select(c: c_int) {
    // SAFETY: 'selectmode' is a C string option.
    let by_selectmode = has_char(unsafe { cstr::at(p_slm.get()) }, c);
    let typed = c == 'o' as c_int || (stuff_empty() && typeahead().maplen() == 0);
    set_visual_select(typed && by_selectmode);
}

/// Enter Visual mode of kind `c` at the cursor.
pub(crate) fn n_start_visual_mode(c: c_int) {
    set_visual_mode(VisualMode::from_raw(c));
    set_visual_active(true);
    VIsual_reselect.set(1);
    // SAFETY: `curwin` is the current window.
    // A block selection starting inside a TAB starts at the column the
    // cursor is displayed at, not at the TAB's first column.
    if c == Ctrl_V
        && get_ve_flags(Win::current()) & kOptVeFlagBlock as c_int as c_uint != 0
        && gchar_cursor() == TAB
    {
        validate_virtcol(Win::current());
        coladvance(Win::current(), Win::current().w_virtcol);
    }
    set_visual_anchor(Win::current().w_cursor);
    fold_adjust_visual();
    may_trigger_modechanged();
    setmouse();
    conceal_check_cursor_line();
    if p_smd.get() != 0 && msg_silent.get() == 0 {
        redraw_cmdline.set(true);
    }
    // Seed the "what was highlighted last time" pair so the first redraw
    // has something to compare against.
    if Win::current().w_redr_type < UPD_INVERTED {
        Win::current().w_old_cursor_lnum = Win::current().w_cursor.lnum;
        Win::current().w_old_visual_lnum = Win::current().w_cursor.lnum;
    }
    redraw_curbuf_later(UPD_VALID);
}

/// `gv`: select what was selected last.
///
/// Doing it while a selection is up *swaps* the two, so `gv` twice comes back
/// where it started.
pub(crate) fn nv_gv_cmd(cmd_arg: &mut CmdArg) {
    let vi = unsafe { &raw mut (*Buf::current_raw()).b_visual };
    if unsafe { (*vi).vi_start.lnum } == 0
        || unsafe { (*vi).vi_start.lnum } > Buf::current().b_ml.ml_line_count
        || unsafe { (*vi).vi_end.lnum } == 0
    {
        beep_flush();
        return;
    }

    let tpos;
    if visual_active() {
        let mode = visual_mode();
        set_visual_mode(VisualMode::from_raw(unsafe { (*vi).vi_mode }));
        unsafe { (*vi).vi_mode = mode.raw() };
        Buf::current().b_visual_mode_eval = mode.raw();
        let curswant = Win::current().w_curswant;
        Win::current().w_curswant = unsafe { (*vi).vi_curswant };
        unsafe { (*vi).vi_curswant = curswant };
        tpos = unsafe { (*vi).vi_end };
        unsafe { (*vi).vi_end = Win::current().w_cursor };
        Win::current().w_cursor = unsafe { (*vi).vi_start };
        unsafe { (*vi).vi_start = visual_anchor() };
    } else {
        set_visual_mode(VisualMode::from_raw(unsafe { (*vi).vi_mode }));
        Win::current().w_curswant = unsafe { (*vi).vi_curswant };
        tpos = unsafe { (*vi).vi_end };
        Win::current().w_cursor = unsafe { (*vi).vi_start };
    }

    set_visual_active(true);
    VIsual_reselect.set(1);
    // Both ends are checked against the buffer: it may have shrunk since.
    check_cursor(Win::current());
    set_visual_anchor(Win::current().w_cursor);
    Win::current().w_cursor = tpos;
    check_cursor(Win::current());
    update_topline(Win::current());
    if cmd_arg.arg != 0 {
        set_visual_select(true);
        VIsual_select_reg.set(0);
    } else {
        may_start_select('c' as c_int);
    }
    setmouse();
    redraw_curbuf_later(UPD_INVERTED);
    showmode();
}

/// Make an exclusive selection cover the character the cursor is on, so the
/// operator about to run sees what the highlight showed.
pub(crate) fn adjust_for_sel(cmd_arg: &mut CmdArg) {
    if visual_active()
        && cmd_arg.op().inclusive
        && sel_exclusive()
        && gchar_cursor() != NUL
        && lt(visual_anchor(), Win::current().w_cursor)
    {
        inc_cursor();
        cmd_arg.op().inclusive = false;
        VIsual_select_exclu_adj.set(true);
    }
}

/// Undo [`adjust_for_sel`] on whichever end is the later one.
///
/// Answers whether the position moved to the previous line.
pub(crate) fn unadjust_for_sel() -> bool {
    if sel_exclusive() && !equalpos(visual_anchor(), Win::current().w_cursor) {
        if lt(visual_anchor(), Win::current().w_cursor) {
            let mut win = Win::current();
            return unadjust_for_sel_inner(&mut win.w_cursor);
        }
        return with_visual_anchor(unadjust_for_sel_inner);
    }
    false
}

/// Move one position back, across a line break if there is nothing else left.
///
/// Answers whether it crossed one.
pub(crate) fn unadjust_for_sel_inner(pos: &mut Pos) -> bool {
    VIsual_select_exclu_adj.set(false);
    if pos.coladd > 0 {
        pos.coladd -= 1;
    } else if pos.col > 0 {
        pos.col -= 1;
        // SAFETY: `curbuf` is set from startup to exit, and `pos` is lent for
        // the length of the call.
        unsafe { mark_mb_adjustpos(Buf::current(), pos) };
        // Inside a TAB, stepping back a byte means stepping to the last
        // screen column the TAB covers.
        // SAFETY: `curwin` is set from startup to exit.
        if virtual_active(Win::current()) {
            let (mut cs, mut ce): (ColNr, ColNr) = (0, 0);
            let win = Win::current();
            unsafe { getvcol(win, pos, &raw mut cs, ptr::null_mut(), &raw mut ce) };
            pos.coladd = ce - cs;
        }
    } else if pos.lnum > 1 {
        pos.lnum -= 1;
        pos.col = ml_get_len(pos.lnum);
        return true;
    }
    false
}

/// `gh`, `gH`, `g CTRL-H`: Select mode, either fresh or from a reselection.
pub(crate) fn nv_select(cmd_arg: &mut CmdArg) {
    if visual_active() {
        set_visual_select(true);
        VIsual_select_reg.set(0);
    } else if VIsual_reselect.get() != 0 {
        // Re-enter through `gv`, which is where the reselection lives.
        cmd_arg.nchar = 'v' as c_int;
        cmd_arg.arg = 1;
        nv_g_cmd(cmd_arg);
    }
}

/// A text object: `iw`, `a(`, `it` and the rest.
///
/// 'matchpairs' is forced to the four bracket pairs for the duration, because
/// a text object's idea of a block is fixed and must not follow the option.
pub(crate) fn nv_object(cmd_arg: &mut CmdArg) {
    let include = cmd_arg.cmdchar != 'i' as c_int;
    let mps_save = Buf::current().b_p_mps;
    Buf::current().b_p_mps = c"(:),{:},[:],<:>".as_ptr().cast_mut();

    let op = cmd_arg.op();
    let n = cmd_arg.count1;
    let found = match u8::try_from(cmd_arg.nchar).unwrap_or(0) {
        b'w' => unsafe { current_word(op.raw(), n, include, false).is_ok() },
        b'W' => unsafe { current_word(op.raw(), n, include, true).is_ok() },
        b'b' | b'(' | b')' => block(op, n, include, '(', ')'),
        b'B' | b'{' | b'}' => block(op, n, include, '{', '}'),
        b'[' | b']' => block(op, n, include, '[', ']'),
        b'<' | b'>' => block(op, n, include, '<', '>'),
        b't' => {
            // A tag block's end is already where it should be; the
            // operator must not push it back over the closing tag.
            cmd_arg.retval |= CA_NO_ADJ_OP_END as c_int;
            unsafe { current_tagblock(op.raw(), n, include) != 0 }
        }
        b'p' => unsafe { current_par(op.raw(), n, include, 'p' as c_int) != 0 },
        b's' => unsafe { current_sent(op.raw(), n, include).is_ok() },
        b'"' | b'\'' | b'`' => unsafe { current_quote(op.raw(), n, include, cmd_arg.nchar) },
        _ => false,
    };

    Buf::current().b_p_mps = mps_save;
    if !found {
        clear_op_beep(op);
    }
    adjust_cursor_col();
    Win::current().w_set_curswant = true;
}

/// The `i(`/`a{`-family text object: the block `open`..`close` around the
/// cursor, `n` levels out.
fn block(op: Op, n: c_int, include: bool, open: char, close: char) -> bool {
    // SAFETY: `op` is a live operator and the cursor is in its own buffer.
    unsafe { current_block(op.raw(), n, include, open as c_int, close as c_int).is_ok() }
}
