//! Going to the position an entry names.
//!
//! [`qf_jump_newwin`] is the entry point. It picks the entry, finds a window
//! to show it in (`switchbuf.rs`), opens the buffer (`qf_jump_edit_buffer`),
//! moves the cursor (`qf_jump_goto_line`) and reports what it did
//! (`qf_jump_print_msg`).
//!
//! Every one of those steps can run autocommands, and an autocommand may
//! replace or free the very list being jumped through. So the entry is held
//! by position, the list by its slot, and the list's identity, its change
//! tick and the entry's presence are re-checked after each step;
//! [`Jumped::Aborted`] is what says the list must not be written back to at
//! all.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::edit::BeginlineOpts;
use crate::ex_cmds::{EcmdFlags, edit_buffer_number};
use crate::ex_docmd::cmdmod_tab;
use crate::message::msg_keep_text;
use crate::search::search_forward_keep;
use crate::tr;
use crate::types::ShmFlag;
use crate::winlayer::prev_window;
use core::ffi::{CStr, c_char, c_int, c_uint};

/// What became of one jump.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Jumped {
    /// Went there.
    Done,
    /// Could not; the current entry should go back to what it was.
    Restore,
    /// The entry names no file, so there was nowhere to go — but the walk
    /// still counts, or `:cnext` would stick on it forever.
    Nowhere,
    /// An autocommand replaced or freed the list.
    Aborted,
}

/// The list a jump step started on, to check it against afterwards.
struct Started {
    qfl: Qfl,
    curlist: c_int,
    changedtick: c_int,
}

impl Started {
    fn now(qi: Qi) -> Started {
        let qfl = qi.current_slot();
        Started {
            qfl,
            curlist: qi.current,
            changedtick: qfl.changedtick,
        }
    }

    /// Whether the list is still the one the step started on, and still
    /// holds entry `at`. Reports E925 or E926 when it is not.
    fn still_current(&self, at: usize) -> bool {
        let qi = self.qfl.stack();
        if self.curlist == qi.current
            && self.changedtick == self.qfl.changedtick
            && is_qf_entry_present(&self.qfl, at)
        {
            return true;
        }
        emsg_list_changed(self.qfl.kind);
        false
    }
}

/// Open the file (or help file) entry `at` names in the current window.
fn qf_jump_edit_buffer(
    qi: Qi,
    at: usize,
    forceit: bool,
    prev_winid: c_int,
    opened_window: &mut bool,
) -> Jumped {
    let started = Started::now(qi);
    let (kind, save_qfid) = (started.qfl.kind, started.qfl.id);
    let entry = &started.qfl.entries[at];
    let (entry_kind, fnum) = (entry.kind, entry.fnum);

    let opened = if entry_kind == 1 {
        // A help file: `do_ecmd` sets 'buftype', `readfile` sets
        // 'readonly'.
        if !can_abandon(Buf::current(), forceit) {
            no_write_message();
            return Jumped::Restore;
        }
        let oldwin = (prev_winid == Win::current().handle).then(|| Win::current().id());
        edit_buffer_number(fnum, 1, EcmdFlags::HIDE | EcmdFlags::SET_HELP, oldwin).is_ok()
    } else {
        match escape_winfixbuf(qi, fnum, forceit, opened_window) {
            None => return Jumped::Restore,
            Some(false) => false,
            Some(true) => buflist_getfile(
                fnum,
                1,
                GETF_SETMARK.cast_signed() | GETF_SWITCH.cast_signed(),
                c_int::from(forceit),
            )
            .is_ok(),
        }
    };

    // For a location list, the window it belongs to may be gone.
    if kind == QFLT_LOCATION
        && win_by_id(prev_winid).is_none()
        && Win::current().w_llist != Some(qi.id())
    {
        emsg(gettext(c"E924: Current window was closed"));
        *opened_window = false;
        return Jumped::Aborted;
    }
    if kind == QFLT_QUICKFIX && !qflist_valid(None, save_qfid) {
        emsg(gettext(E_QUICKFIX_LIST_CHANGED));
        return Jumped::Aborted;
    }
    if !started.still_current(at) {
        return Jumped::Aborted;
    }
    if opened {
        Jumped::Done
    } else {
        Jumped::Restore
    }
}

/// Get out of a `'winfixbuf'` window, so that another buffer can be opened
/// at all.
///
/// Answers `None` when the jump must be given up straight away, and
/// otherwise whether the window will take the buffer. A window that is
/// still `'winfixbuf'` reports E1513 but does not return early: the
/// autocommands that got us here may have changed the list, and the caller
/// has to notice that.
fn escape_winfixbuf(qi: Qi, fnum: c_int, forceit: bool, opened_window: &mut bool) -> Option<bool> {
    if forceit || Win::current().w_onebuf_opt.wo_wfb == 0 || Buf::current().handle == fnum {
        return Some(true);
    }
    if qi.kind == QFLT_LOCATION {
        // A location list cannot split or reassign its window.
        qf_emsg(e_winfixbuf_cannot_go_to_buffer);
        return None;
    }
    // Try the previously used window, if it can take another buffer.
    let usable = prev_window().filter(|p| {
        win_valid(p.id()) && p.w_onebuf_opt.wo_wfb == 0 && !buf_is_quickfix(p.buffer_or_none())
    });
    if let Some(prev) = usable {
        win_goto(prev);
    }
    if Win::current().w_onebuf_opt.wo_wfb == 0 {
        return Some(true);
    }
    // Split off a window, which is 'nowinfixbuf'.
    if win_split(0, 0).is_ok() {
        *opened_window = true;
    }
    if Win::current().w_onebuf_opt.wo_wfb == 0 {
        return Some(true);
    }
    // The split failed, or autocommands set 'winfixbuf' again or sent
    // us to another window that has it.
    qf_emsg(e_winfixbuf_cannot_go_to_buffer);
    Some(false)
}

/// Where an entry sends the cursor, copied out of the list before the
/// cursor moves: a search can print, and printing can run user code.
struct Position {
    lnum: LineNr,
    col: c_int,
    viscol: c_char,
    pattern: Option<Vec<u8>>,
}

/// Put the cursor on the position an entry names: its line and column, or
/// wherever its search pattern matches.
fn qf_jump_goto_line(position: &Position) {
    if let Some(pattern) = &position.pattern {
        // Search from before the first line, and stay put if the
        // pattern is not there any more.
        let save_cursor = Win::current().w_cursor;
        Win::current().w_cursor.lnum = 0;
        if !search_forward_keep(pattern) {
            Win::current().w_cursor = save_cursor;
        }
        return;
    }

    // A line number of 0 means the entry names no line.
    if position.lnum > 0 {
        Win::current().w_cursor.lnum = position.lnum.min(Buf::current().b_ml.ml_line_count);
    }
    if position.col <= 0 {
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        return;
    }
    Win::current().w_cursor.coladd = 0;
    if c_int::from(position.viscol) == 1 {
        coladvance(Win::current(), position.col - 1);
    } else {
        Win::current().w_cursor.col = position.col - 1;
    }
    Win::current().w_set_curswant = true;
    check_cursor(Win::current());
}

/// Say which entry of how many the jump landed on, and what it said.
fn qf_jump_print_msg(qi: Qi, qf_index: c_int, at: usize, old_curbuf: Buf, old_lnum: LineNr) {
    if msg_scrolled.get() == 0 {
        update_topline(Win::current());
        if must_redraw.get() != 0 {
            let _ = update_screen();
        }
    }
    // Copied out: building the text runs nothing, but showing it can. A
    // redraw can run user code too, so the entry may have gone.
    let (count, cleared, types, text) = {
        let qfl = qi.current_list();
        let Some(entry) = qfl.entries.get(at) else {
            return;
        };
        (
            qfl.count(),
            entry.cleared,
            qf_types(c_int::from(entry.kind), entry.nr),
            entry.text.clone(),
        )
    };
    let deleted = if cleared {
        gettext(c" (line deleted)")
    } else {
        c""
    };
    let head = tr!(
        "({} of {}){}{}: ",
        qf_index,
        count,
        deleted.to_string_lossy(),
        types.to_string_lossy()
    );
    let mut line = head.into_bytes();
    // The message itself, without leading whitespace or newlines.
    qf_fmt_text(&mut line, skip_white(&text));
    line.push(0);
    let line = CStr::from_bytes_until_nul(&line).expect("terminated above");

    // Overwrite rather than scroll when 'shortmess' holds "O" — but
    // print the whole message when the jump did not actually move.
    let old_msg_scroll = msg_scroll.get();
    if Buf::current_or_none() == Some(old_curbuf) && Win::current().w_cursor.lnum == old_lnum {
        msg_scroll.set(c_int::from(true));
    } else if (msg_scrolled.get() == 0 || p_ch() == 0 && msg_scrolled.get() == 1)
        && shortmess(ShmFlag::OVERALL)
    {
        msg_scroll.set(c_int::from(false));
    }
    msg_ext_set_kind(c"quickfix");
    msg_keep_text(line, 0);
    msg_scroll.set(old_msg_scroll);
}

/// Find a window to show entry `at` in, when the jump starts from a
/// quickfix or location list window.
fn qf_jump_open_window(qi: Qi, at: usize, newwin: bool, opened_window: &mut bool) -> Jumped {
    let started = Started::now(qi);

    // A `:helpgrep` entry wants a help window.
    if started.qfl.entries[at].kind == 1
        && (!buf_is_help(Win::current().buffer_or_none()) || cmdmod_tab() != 0)
        && jump_to_help_window(qi, newwin, opened_window).is_err()
    {
        return Jumped::Restore;
    }
    if !started.still_current(at) {
        return Jumped::Aborted;
    }

    if buf_is_quickfix(current_buf()) && !*opened_window {
        let fnum = started.qfl.entries[at].fnum;
        if fnum == 0 {
            return Jumped::Nowhere;
        }
        if qf_jump_to_usable_window(fnum, newwin, opened_window).is_err() {
            return Jumped::Restore;
        }
    }
    if !started.still_current(at) {
        return Jumped::Aborted;
    }
    Jumped::Done
}

/// Open entry `at`'s file, go to its position, and say so.
#[allow(clippy::too_many_arguments)]
fn qf_jump_to_buffer(
    qi: Qi,
    qf_index: c_int,
    at: usize,
    forceit: bool,
    prev_winid: c_int,
    opened_window: &mut bool,
    openfold: bool,
    print_message: bool,
) -> Jumped {
    // Held, not re-derived: `qf_jump_edit_buffer` below runs autocommands
    // that can wipe this buffer, and everything downstream only compares it.
    let old_curbuf = Buf::current();
    let old_lnum = Win::current().w_cursor.lnum;

    if qi.current_list().entries[at].fnum != 0 {
        let edited = qf_jump_edit_buffer(qi, at, forceit, prev_winid, opened_window);
        if edited != Jumped::Done {
            return edited;
        }
    }
    // Staying in the same buffer still sets the previous-context mark.
    if Buf::current_or_none() == Some(old_curbuf) {
        setpcmark();
    }
    let position = {
        let entry = &qi.current_list().entries[at];
        Position {
            lnum: entry.lnum,
            col: entry.col,
            viscol: entry.viscol,
            pattern: entry.pattern.as_ref().map(|p| p.to_vec()),
        }
    };
    qf_jump_goto_line(&position);
    if fdo_flags.get() & kOptFdoFlagQuickfix as c_uint != 0 && openfold {
        fold_open_cursor();
    }
    if print_message {
        qf_jump_print_msg(qi, qf_index, at, old_curbuf, old_lnum);
    }
    Jumped::Done
}

/// Jump to an entry, reusing a window where possible.
pub(crate) fn qf_jump(qi: Qi, dir: c_int, errornr: c_int, forceit: bool) {
    qf_jump_newwin(qi, dir, errornr, forceit, false);
}

/// `qf_jump(NULL, 0, 0, FALSE)`: go to the current entry of the quickfix
/// list, as `-q` asks at startup.
pub fn qf_jump_to_current() {
    qf_jump(Qi::global(), 0, 0, false);
}

/// Jump to an entry.
///
/// `dir` is `FORWARD`/`BACKWARD` to move `errornr` entries, or
/// `FORWARD_FILE`/`BACKWARD_FILE` to move that many *files*; with `dir` 0,
/// `errornr` names the entry to go to, and 0 redisplays the current one.
/// `forceit` allows abandoning a changed buffer, and `newwin` always splits
/// a new window.
pub(crate) fn qf_jump_newwin(qi: Qi, dir: c_int, errornr: c_int, forceit: bool, newwin: bool) {
    if qi.is_empty() || qi.current_list().is_empty() {
        qf_emsg(e_no_errors);
        return;
    }
    // A copy of 'switchbuf', to put back if the jump empties it: the split
    // path below clears the option so that the *next* entry does not split
    // again, and the option owning nothing is how that is recognised.
    let old_swb = (!P_SWB.is_unset()).then(|| P_SWB.get());
    let old_swb_flags = swb_flags.get();
    // Getting the file may reset it.
    let old_key_typed = KeyTyped.get();

    let busy = QuickfixBusy::hold();
    let mut qfl = qi.current_slot();
    let old_cursor = qfl.cursor;
    let old_index = qfl.index;
    let mut qf_index = old_index;
    let found = qf_get_entry(&qfl, errornr, dir, &mut qf_index);
    if found == Err(NoEntry::NoMoreItems) {
        emsg(gettext(E_NO_MORE_ITEMS));
    }

    // Which entry the list should be left pointing at. `None` means the
    // list is not ours to write to any more.
    let mut settle = Some((old_cursor, old_index));
    if let Ok(at) = found {
        qfl.index = qf_index;
        qfl.cursor = at;
        settle = Some((at, qf_index));

        // No need to print the message when the quickfix window shows it.
        let print_message = !qf_win_pos_update(qi, old_index);
        let prev_winid = Win::current().handle;
        let names_a_file = qfl.entries[at].fnum != 0;
        let mut opened_window = false;

        match qf_jump_open_window(qi, at, newwin, &mut opened_window) {
            // No window could be found. A window opened on the way is
            // deliberately left open, as upstream does.
            Jumped::Restore => settle = Some((old_cursor, old_index)),
            Jumped::Aborted => settle = None,
            // The entry named no file: stay on it, so that the next
            // `:cnext` moves past it.
            Jumped::Nowhere => {}
            Jumped::Done => {
                let jumped = qf_jump_to_buffer(
                    qi,
                    qf_index,
                    at,
                    forceit,
                    prev_winid,
                    &mut opened_window,
                    old_key_typed,
                    print_message,
                );
                if jumped != Jumped::Done {
                    if opened_window {
                        win_close(Win::current(), true, false);
                    }
                    if jumped == Jumped::Aborted {
                        settle = None;
                    } else if names_a_file {
                        // The file would not open — it was readonly and
                        // something had been changed, say. Put the
                        // current entry back where it was.
                        settle = Some((old_cursor, old_index));
                    }
                }
            }
        }
    }
    if let Some((cursor, index)) = settle {
        qfl.cursor = cursor;
        qfl.index = index;
    }

    // Put 'switchbuf' back, unless an autocommand or a modeline changed
    // it meanwhile -- in which case it owns a value of its own.
    if P_SWB.is_unset() {
        P_SWB.restore(old_swb);
        swb_flags.set(old_swb_flags);
    }
    drop(busy);
}

/// Jump to the first entry of the list with the given id, after making that
/// list current again.
pub(crate) fn qf_jump_first(qi: Qi, save_qfid: c_uint, forceit: bool) {
    if qf_restore_list(qi, save_qfid).is_err()
        || !check_can_set_curbuf_forceit(c_int::from(forceit))
    {
        return;
    }
    // Autocommands may have cleared the list.
    if !qi.current_list().is_empty() {
        qf_jump(qi, 0, 0, forceit);
    }
}
