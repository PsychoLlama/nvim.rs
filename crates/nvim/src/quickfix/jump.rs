//! Going to the position an entry names.
//!
//! [`qf_jump_newwin`] is the entry point. It picks the entry, finds a window
//! to show it in (`switchbuf.rs`), opens the buffer (`qf_jump_edit_buffer`),
//! moves the cursor (`qf_jump_goto_line`) and reports what it did
//! (`qf_jump_print_msg`).
//!
//! Every one of those steps can run autocommands, and an autocommand may
//! replace or free the very list being jumped through. The list's identity,
//! its changed tick and the entry's presence are therefore re-checked after
//! each step; [`Jumped::Aborted`] is what says the list must not be written
//! back to at all.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::edit::BeginlineOpts;
use crate::search::SEARCH_KEEP;
use crate::types::{FAIL, IOSIZE, OK};
use core::ffi::{c_char, c_int, c_uint};
use core::{ptr, slice};

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

/// Whether the list is still the one the jump started on, and still holds
/// the entry. Reports E925 or E926 when it is not.
///
/// # Safety
///
/// `qi` must be a live stack and `qfl`/`qf_ptr` what it held.
unsafe fn list_still_current(
    qi: *mut qf_info_T,
    qfl: *mut qf_list_T,
    qf_ptr: *mut qfline_T,
    old_curlist: c_int,
    old_changedtick: c_int,
) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        if old_curlist == (*qi).qf_curlist
            && old_changedtick == (*qfl).qf_changedtick
            && is_qf_entry_present(qfl, qf_ptr)
        {
            return true;
        }
        emsg_list_changed((*qfl).qfl_type);
        false
    }
}

/// Open the file (or help file) the entry names in the current window.
///
/// # Safety
///
/// `qi` must be a live stack and `qf_ptr` an entry in its current list.
unsafe fn qf_jump_edit_buffer(
    qi: *mut qf_info_T,
    qf_ptr: *mut qfline_T,
    forceit: c_int,
    prev_winid: c_int,
    opened_window: &mut bool,
) -> Jumped {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qfl = qf_get_curlist(qi);
        let old_changedtick = (*qfl).qf_changedtick;
        let old_curlist = (*qi).qf_curlist;
        let qfl_type = (*qfl).qfl_type;
        let save_qfid = (*qfl).qf_id;

        let opened = if (*qf_ptr).qf_type == 1 {
            // A help file: `do_ecmd` sets 'buftype', `readfile` sets
            // 'readonly'.
            if !can_abandon(curbuf.get(), forceit != 0) {
                no_write_message();
                return Jumped::Restore;
            }
            do_ecmd(
                (*qf_ptr).qf_fnum,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                1,
                ECMD_HIDE as c_int + ECMD_SET_HELP as c_int,
                if prev_winid == (*curwin.get()).handle {
                    curwin.get()
                } else {
                    ptr::null_mut()
                },
            ) == OK
        } else {
            // Read before the window juggling below, which changes it.
            let fnum = (*qf_ptr).qf_fnum;
            match escape_winfixbuf(qi, fnum, forceit, opened_window) {
                None => return Jumped::Restore,
                Some(false) => false,
                Some(true) => {
                    buflist_getfile(
                        fnum,
                        1,
                        GETF_SETMARK as c_int | GETF_SWITCH as c_int,
                        forceit,
                    ) == OK
                }
            }
        };

        // For a location list, the window it belongs to may be gone.
        if qfl_type == QFLT_LOCATION {
            if win_id2wp(prev_winid).is_null() && (*curwin.get()).w_llist != qi {
                emsg(gettext(c"E924: Current window was closed".as_ptr()));
                *opened_window = false;
                return Jumped::Aborted;
            }
        }
        if qfl_type == QFLT_QUICKFIX && !qflist_valid(ptr::null_mut(), save_qfid) {
            emsg(gettext(E_QUICKFIX_LIST_CHANGED.as_ptr()));
            return Jumped::Aborted;
        }
        if !list_still_current(qi, qfl, qf_ptr, old_curlist, old_changedtick) {
            return Jumped::Aborted;
        }
        if opened {
            Jumped::Done
        } else {
            Jumped::Restore
        }
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
///
/// # Safety
///
/// `qi` must be a live stack and `opened_window` writable.
unsafe fn escape_winfixbuf(
    qi: *mut qf_info_T,
    fnum: c_int,
    forceit: c_int,
    opened_window: &mut bool,
) -> Option<bool> {
    // SAFETY: forwarded from the caller.
    unsafe {
        if forceit != 0
            || (*curwin.get()).w_onebuf_opt.wo_wfb == 0
            || (*curbuf.get()).handle == fnum
        {
            return Some(true);
        }
        if (*qi).qfl_type == QFLT_LOCATION {
            // A location list cannot split or reassign its window.
            emsg(gettext(
                &raw const e_winfixbuf_cannot_go_to_buffer as *const c_char,
            ));
            return None;
        }
        // Try the previously used window, if it can take another buffer.
        if win_valid(prevwin.get())
            && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
            && !bt_quickfix((*prevwin.get()).w_buffer)
        {
            win_goto(prevwin.get());
        }
        if (*curwin.get()).w_onebuf_opt.wo_wfb == 0 {
            return Some(true);
        }
        // Split off a window, which is 'nowinfixbuf'.
        if win_split(0, 0) == OK {
            *opened_window = true;
        }
        if (*curwin.get()).w_onebuf_opt.wo_wfb == 0 {
            return Some(true);
        }
        // The split failed, or autocommands set 'winfixbuf' again or sent
        // us to another window that has it.
        emsg(gettext(
            &raw const e_winfixbuf_cannot_go_to_buffer as *const c_char,
        ));
        Some(false)
    }
}

/// Put the cursor on the position an entry names: its line and column, or
/// wherever its search pattern matches.
///
/// # Safety
///
/// `qf_pattern` must be null or NUL-terminated, and the buffer loaded.
unsafe fn qf_jump_goto_line(
    qf_lnum: linenr_T,
    qf_col: c_int,
    qf_viscol: c_char,
    qf_pattern: *mut c_char,
) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !qf_pattern.is_null() {
            // Search from before the first line, and stay put if the
            // pattern is not there any more.
            let save_cursor = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor.lnum = 0;
            let found = do_search(
                ptr::null_mut(),
                '/' as c_int,
                '/' as c_int,
                qf_pattern,
                strlen(qf_pattern),
                1,
                SEARCH_KEEP as c_int,
                ptr::null_mut(),
            );
            if found == 0 {
                (*curwin.get()).w_cursor = save_cursor;
            }
            return;
        }

        // A line number of 0 means the entry names no line.
        if qf_lnum > 0 {
            (*curwin.get()).w_cursor.lnum = qf_lnum.min((*curbuf.get()).b_ml.ml_line_count);
        }
        if qf_col <= 0 {
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            return;
        }
        (*curwin.get()).w_cursor.coladd = 0;
        if qf_viscol as c_int == 1 {
            coladvance(curwin.get(), qf_col as colnr_T - 1);
        } else {
            (*curwin.get()).w_cursor.col = (qf_col - 1) as colnr_T;
        }
        (*curwin.get()).w_set_curswant = true as c_int;
        check_cursor(curwin.get());
    }
}

/// Say which entry of how many the jump landed on, and what it said.
///
/// # Safety
///
/// `qi` must be a live stack and `qf_ptr` an entry in its current list.
unsafe fn qf_jump_print_msg(
    qi: *mut qf_info_T,
    qf_index: c_int,
    qf_ptr: *mut qfline_T,
    old_curbuf: *mut buf_T,
    old_lnum: linenr_T,
) {
    // SAFETY: forwarded from the caller.
    unsafe {
        // Update the screen before showing the message, unless messages
        // have scrolled.
        if msg_scrolled.get() == 0 {
            update_topline(curwin.get());
            if must_redraw.get() != 0 {
                update_screen();
            }
        }
        let len = vim_snprintf_safelen(
            IObuff.ptr().cast(),
            IOSIZE as size_t,
            gettext(c"(%d of %d)%s%s: ".as_ptr()),
            qf_index,
            (*qf_get_curlist(qi)).qf_count,
            if (*qf_ptr).qf_cleared != 0 {
                gettext(c" (line deleted)".as_ptr())
            } else {
                c"".as_ptr()
            },
            qf_types((*qf_ptr).qf_type as c_int, (*qf_ptr).qf_nr),
        );
        let text = build_line(|out| {
            out.extend_from_slice(slice::from_raw_parts(IObuff.ptr().cast::<u8>(), len));
            // The message itself, without leading whitespace or newlines.
            qf_fmt_text(out, skipwhite((*qf_ptr).qf_text));
        });

        // Overwrite rather than scroll when 'shortmess' holds "O" — but
        // print the whole message when the jump did not actually move.
        let old_msg_scroll = msg_scroll.get();
        if curbuf.get() == old_curbuf && (*curwin.get()).w_cursor.lnum == old_lnum {
            msg_scroll.set(true as c_int);
        } else if (msg_scrolled.get() == 0 || p_ch.get() == 0 && msg_scrolled.get() == 1)
            && shortmess(SHM_OVERALL as c_int)
        {
            msg_scroll.set(false as c_int);
        }
        msg_ext_set_kind(c"quickfix".as_ptr());
        msg_keep(text.as_ptr().cast(), 0, true, false);
        msg_scroll.set(old_msg_scroll);
        release_scratch();
    }
}

/// Find a window to show the entry in, when the jump starts from a quickfix
/// or location list window.
///
/// # Safety
///
/// `qi` must be a live stack and `qf_ptr` an entry in its current list.
unsafe fn qf_jump_open_window(
    qi: *mut qf_info_T,
    qf_ptr: *mut qfline_T,
    newwin: bool,
    opened_window: &mut bool,
) -> Jumped {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qfl = qf_get_curlist(qi);
        let old_changedtick = (*qfl).qf_changedtick;
        let old_curlist = (*qi).qf_curlist;

        // A `:helpgrep` entry wants a help window.
        if (*qf_ptr).qf_type == 1
            && (!bt_help((*curwin.get()).w_buffer) || (*cmdmod.ptr()).cmod_tab != 0)
            && jump_to_help_window(qi, newwin, opened_window) == FAIL
        {
            return Jumped::Restore;
        }
        if !list_still_current(qi, qfl, qf_ptr, old_curlist, old_changedtick) {
            return Jumped::Aborted;
        }

        if bt_quickfix(curbuf.get()) && !*opened_window {
            if (*qf_ptr).qf_fnum == 0 {
                return Jumped::Nowhere;
            }
            if qf_jump_to_usable_window((*qf_ptr).qf_fnum, newwin, opened_window) == FAIL {
                return Jumped::Restore;
            }
        }
        if !list_still_current(qi, qfl, qf_ptr, old_curlist, old_changedtick) {
            return Jumped::Aborted;
        }
        Jumped::Done
    }
}

/// Open the entry's file, go to its position, and say so.
///
/// # Safety
///
/// `qi` must be a live stack and `qf_ptr` an entry in its current list.
#[allow(clippy::too_many_arguments)]
unsafe fn qf_jump_to_buffer(
    qi: *mut qf_info_T,
    qf_index: c_int,
    qf_ptr: *mut qfline_T,
    forceit: c_int,
    prev_winid: c_int,
    opened_window: &mut bool,
    openfold: bool,
    print_message: bool,
) -> Jumped {
    // SAFETY: forwarded from the caller.
    unsafe {
        let old_curbuf = curbuf.get();
        let old_lnum = (*curwin.get()).w_cursor.lnum;

        if (*qf_ptr).qf_fnum != 0 {
            let edited = qf_jump_edit_buffer(qi, qf_ptr, forceit, prev_winid, opened_window);
            if edited != Jumped::Done {
                return edited;
            }
        }
        // Staying in the same buffer still sets the previous-context mark.
        if curbuf.get() == old_curbuf {
            setpcmark();
        }
        qf_jump_goto_line(
            (*qf_ptr).qf_lnum,
            (*qf_ptr).qf_col,
            (*qf_ptr).qf_viscol,
            (*qf_ptr).qf_pattern,
        );
        if fdo_flags.get() & kOptFdoFlagQuickfix as c_uint != 0 && openfold {
            foldOpenCursor();
        }
        if print_message {
            qf_jump_print_msg(qi, qf_index, qf_ptr, old_curbuf, old_lnum);
        }
        Jumped::Done
    }
}

/// Jump to an entry, reusing a window where possible.
///
/// # Safety
///
/// `qi` must be null (meaning the quickfix stack) or a live stack.
pub unsafe fn qf_jump(qi: *mut qf_info_T, dir: c_int, errornr: c_int, forceit: c_int) {
    // SAFETY: forwarded from the caller.
    unsafe { qf_jump_newwin(qi, dir, errornr, forceit, false) };
}

/// Jump to an entry.
///
/// `dir` is `FORWARD`/`BACKWARD` to move `errornr` entries, or
/// `FORWARD_FILE`/`BACKWARD_FILE` to move that many *files*; with `dir` 0,
/// `errornr` names the entry to go to, and 0 redisplays the current one.
/// `forceit` allows abandoning a changed buffer, and `newwin` always splits
/// a new window.
///
/// # Safety
///
/// `qi` must be null (meaning the quickfix stack) or a live stack.
pub(crate) unsafe fn qf_jump_newwin(
    mut qi: *mut qf_info_T,
    dir: c_int,
    errornr: c_int,
    forceit: c_int,
    newwin: bool,
) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if qi.is_null() {
            qi = ql_info.get();
        }
        if qf_stack_empty(qi) || qf_list_empty(qf_get_curlist(qi)) {
            emsg(gettext(&raw const e_no_errors as *const c_char));
            return;
        }
        let old_swb = p_swb.get();
        let old_swb_flags = swb_flags.get();
        // Getting the file may reset it.
        let old_key_typed = KeyTyped.get();

        incr_quickfix_busy();
        let qfl = qf_get_curlist(qi);
        let old_qf_ptr = (*qfl).qf_ptr;
        let old_qf_index = (*qfl).qf_index;
        let mut qf_index = old_qf_index;
        let qf_ptr = qf_get_entry(qfl, errornr, dir, &mut qf_index);

        // Which entry the list should be left pointing at. `None` means the
        // list is not ours to write to any more.
        let mut settle = Some((old_qf_ptr, old_qf_index));
        if !qf_ptr.is_null() {
            (*qfl).qf_index = qf_index;
            (*qfl).qf_ptr = qf_ptr;
            settle = Some((qf_ptr, qf_index));

            // No need to print the message when the quickfix window shows it.
            let print_message = !qf_win_pos_update(qi, old_qf_index);
            let prev_winid = (*curwin.get()).handle as c_int;
            let mut opened_window = false;

            match qf_jump_open_window(qi, qf_ptr, newwin, &mut opened_window) {
                // No window could be found. A window opened on the way is
                // deliberately left open, as upstream does.
                Jumped::Restore => settle = Some((old_qf_ptr, old_qf_index)),
                Jumped::Aborted => settle = None,
                // The entry named no file: stay on it, so that the next
                // `:cnext` moves past it.
                Jumped::Nowhere => {}
                Jumped::Done => {
                    let jumped = qf_jump_to_buffer(
                        qi,
                        qf_index,
                        qf_ptr,
                        forceit,
                        prev_winid,
                        &mut opened_window,
                        old_key_typed,
                        print_message,
                    );
                    if jumped != Jumped::Done {
                        if opened_window {
                            win_close(curwin.get(), true, false);
                        }
                        if jumped == Jumped::Aborted {
                            settle = None;
                        } else if (*qf_ptr).qf_fnum != 0 {
                            // The file would not open — it was readonly and
                            // something had been changed, say. Put the
                            // current entry back where it was.
                            settle = Some((old_qf_ptr, old_qf_index));
                        }
                    }
                }
            }
        }
        if let Some((entry, index)) = settle {
            (*qfl).qf_ptr = entry;
            (*qfl).qf_index = index;
        }

        // Put 'switchbuf' back, unless an autocommand or a modeline changed
        // it meanwhile.
        if p_swb.get() != old_swb && p_swb.get() == empty_string_option.ptr().cast() {
            p_swb.set(old_swb);
            swb_flags.set(old_swb_flags);
        }
        decr_quickfix_busy();
    }
}

/// Jump to the first entry of the list with the given id, after making that
/// list current again.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_jump_first(qi: *mut qf_info_T, save_qfid: c_uint, forceit: c_int) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if qf_restore_list(qi, save_qfid) == FAIL || !check_can_set_curbuf_forceit(forceit) {
            return;
        }
        // Autocommands may have cleared the list.
        if !qf_list_empty(qf_get_curlist(qi)) {
            qf_jump(qi, 0, 0, forceit);
        }
    }
}
