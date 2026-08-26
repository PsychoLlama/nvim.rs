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
use crate::ex_docmd::cmdmod_tab;
use crate::optionstr::is_empty_option;
use crate::search::SEARCH_KEEP;
use crate::types::{FAIL, IOSIZE, OK, ShmFlag};
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
    // SAFETY: the caller's promise -- a live `qf_list_T`.
    let qfl = unsafe { Qfl::new(qfl) };
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    if old_curlist == qi.qf_curlist
        && old_changedtick == qfl.qf_changedtick
        && unsafe { is_qf_entry_present(qfl.raw(), qf_ptr) }
    {
        return true;
    }
    unsafe { emsg_list_changed(qfl.qfl_type) };
    false
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
    // SAFETY: the caller's promise -- a live `qfline_T`.
    let qf_ptr = unsafe { Qfe::new(qf_ptr) };
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    let qfl = unsafe { qf_get_curlist(qi.raw()) };
    let old_changedtick = unsafe { (*qfl).qf_changedtick };
    let old_curlist = qi.qf_curlist;
    let qfl_type = unsafe { (*qfl).qfl_type };
    let save_qfid = unsafe { (*qfl).qf_id };

    let opened = if qf_ptr.qf_type == 1 {
        // A help file: `do_ecmd` sets 'buftype', `readfile` sets
        // 'readonly'.
        if !unsafe { can_abandon(curbuf.get(), forceit != 0) } {
            no_write_message();
            return Jumped::Restore;
        }
        unsafe {
            do_ecmd(
                qf_ptr.qf_fnum,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                1,
                ECMD_HIDE as c_int + ECMD_SET_HELP as c_int,
                if prev_winid == cur_win().handle {
                    curwin.get()
                } else {
                    ptr::null_mut()
                },
            ) == OK
        }
    } else {
        // Read before the window juggling below, which changes it.
        let fnum = qf_ptr.qf_fnum;
        match unsafe { escape_winfixbuf(qi.raw(), fnum, forceit, opened_window) } {
            None => return Jumped::Restore,
            Some(false) => false,
            Some(true) => unsafe {
                buflist_getfile(
                    fnum,
                    1,
                    GETF_SETMARK as c_int | GETF_SWITCH as c_int,
                    forceit,
                ) == OK
            },
        }
    };

    // For a location list, the window it belongs to may be gone.
    if qfl_type == QFLT_LOCATION && win_by_id(prev_winid).is_none() && cur_win().w_llist != qi.raw()
    {
        unsafe { emsg(gettext(c"E924: Current window was closed".as_ptr())) };
        *opened_window = false;
        return Jumped::Aborted;
    }
    if qfl_type == QFLT_QUICKFIX && !unsafe { qflist_valid(ptr::null_mut(), save_qfid) } {
        unsafe { emsg(gettext(E_QUICKFIX_LIST_CHANGED.as_ptr())) };
        return Jumped::Aborted;
    }
    if !unsafe { list_still_current(qi.raw(), qfl, qf_ptr.raw(), old_curlist, old_changedtick) } {
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
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    if forceit != 0 || cur_win().w_onebuf_opt.wo_wfb == 0 || cur_buf().handle == fnum {
        return Some(true);
    }
    if qi.qfl_type == QFLT_LOCATION {
        // A location list cannot split or reassign its window.
        qf_emsg(&raw const e_winfixbuf_cannot_go_to_buffer as *const c_char);
        return None;
    }
    // Try the previously used window, if it can take another buffer.
    if unsafe { win_valid(prevwin.get()) }
        && unsafe { (*prevwin.get()).w_onebuf_opt.wo_wfb } == 0
        && !unsafe { bt_quickfix((*prevwin.get()).w_buffer) }
    {
        unsafe { win_goto(prevwin.get()) };
    }
    if cur_win().w_onebuf_opt.wo_wfb == 0 {
        return Some(true);
    }
    // Split off a window, which is 'nowinfixbuf'.
    if win_split(0, 0) == OK {
        *opened_window = true;
    }
    if cur_win().w_onebuf_opt.wo_wfb == 0 {
        return Some(true);
    }
    // The split failed, or autocommands set 'winfixbuf' again or sent
    // us to another window that has it.
    qf_emsg(&raw const e_winfixbuf_cannot_go_to_buffer as *const c_char);
    Some(false)
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
    if !qf_pattern.is_null() {
        // Search from before the first line, and stay put if the
        // pattern is not there any more.
        let save_cursor = cur_win().w_cursor;
        cur_win().w_cursor.lnum = 0;
        let oap = ptr::null_mut();
        let dirc = '/' as c_int;
        let search_delim = '/' as c_int;
        let patlen = unsafe { strlen(qf_pattern) };
        let count = 1;
        let options = SEARCH_KEEP as c_int;
        let sia = ptr::null_mut();
        let found = unsafe {
            do_search(
                oap,
                dirc,
                search_delim,
                qf_pattern,
                patlen,
                count,
                options,
                sia,
            )
        };
        if found == 0 {
            cur_win().w_cursor = save_cursor;
        }
        return;
    }

    // A line number of 0 means the entry names no line.
    if qf_lnum > 0 {
        cur_win().w_cursor.lnum = qf_lnum.min(cur_buf().b_ml.ml_line_count);
    }
    if qf_col <= 0 {
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        return;
    }
    cur_win().w_cursor.coladd = 0;
    if qf_viscol as c_int == 1 {
        unsafe { coladvance(curwin.get(), qf_col as colnr_T - 1) };
    } else {
        cur_win().w_cursor.col = (qf_col - 1) as colnr_T;
    }
    cur_win().w_set_curswant = true;
    unsafe { check_cursor(curwin.get()) };
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
    // SAFETY: the caller's promise -- a live entry on a live stack.
    let qf_ptr = unsafe { Qfe::new(qf_ptr) };
    // SAFETY: as above.
    let qi = unsafe { Qi::new(qi) };
    let mut head = [0 as c_char; IOSIZE as usize];
    // SAFETY: forwarded from the caller.
    // Update the screen before showing the message, unless messages
    // have scrolled.
    if msg_scrolled.get() == 0 {
        unsafe { update_topline(curwin.get()) };
        if must_redraw.get() != 0 {
            unsafe { update_screen() };
        }
    }
    let dirc = IOSIZE as size_t;
    let search_delim = unsafe { gettext(c"(%d of %d)%s%s: ".as_ptr()) };
    let patlen = qf_current_list(qi).qf_count;
    let count = if qf_ptr.qf_cleared != 0 {
        unsafe { gettext(c" (line deleted)".as_ptr()) }
    } else {
        c"".as_ptr()
    };
    let types = qf_types(qf_ptr.qf_type as c_int, qf_ptr.qf_nr);
    let options = types.as_ptr();
    let len = unsafe {
        vim_snprintf_safelen(
            head.as_mut_ptr(),
            dirc,
            search_delim,
            qf_index,
            patlen,
            count,
            options,
        )
    };
    let text = build_line(|out| {
        out.extend_from_slice(unsafe { slice::from_raw_parts(head.as_ptr().cast::<u8>(), len) });
        // The message itself, without leading whitespace or newlines.
        unsafe { qf_fmt_text(out, skipwhite(qf_ptr.qf_text)) };
    });

    // Overwrite rather than scroll when 'shortmess' holds "O" — but
    // print the whole message when the jump did not actually move.
    if curbuf.get() == old_curbuf && cur_win().w_cursor.lnum == old_lnum {
        msg_scroll.set(true as c_int);
    } else if (msg_scrolled.get() == 0 || p_ch.get() == 0 && msg_scrolled.get() == 1)
        && shortmess(ShmFlag::OVERALL)
    {
        msg_scroll.set(false as c_int);
    }
    unsafe { msg_ext_set_kind(c"quickfix".as_ptr()) };
    unsafe { msg_keep(text.as_ptr().cast(), 0, true, false) };
    msg_scroll.set(msg_scroll.get());
    release_scratch();
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
    // SAFETY: the caller's promise -- a live `qfline_T`.
    let qf_ptr = unsafe { Qfe::new(qf_ptr) };
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    let qfl = unsafe { qf_get_curlist(qi.raw()) };
    let old_changedtick = unsafe { (*qfl).qf_changedtick };
    let old_curlist = qi.qf_curlist;

    // A `:helpgrep` entry wants a help window.
    if qf_ptr.qf_type == 1
        && (!unsafe { bt_help(cur_win().w_buffer) } || cmdmod_tab() != 0)
        && unsafe { jump_to_help_window(qi.raw(), newwin, opened_window) } == FAIL
    {
        return Jumped::Restore;
    }
    if !unsafe { list_still_current(qi.raw(), qfl, qf_ptr.raw(), old_curlist, old_changedtick) } {
        return Jumped::Aborted;
    }

    if unsafe { bt_quickfix(curbuf.get()) } && !*opened_window {
        if qf_ptr.qf_fnum == 0 {
            return Jumped::Nowhere;
        }
        if unsafe { qf_jump_to_usable_window(qf_ptr.qf_fnum, newwin, opened_window) } == FAIL {
            return Jumped::Restore;
        }
    }
    if !unsafe { list_still_current(qi.raw(), qfl, qf_ptr.raw(), old_curlist, old_changedtick) } {
        return Jumped::Aborted;
    }
    Jumped::Done
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
    // SAFETY: the caller's promise -- a live `qfline_T`.
    let qf_ptr = unsafe { Qfe::new(qf_ptr) };
    // SAFETY: forwarded from the caller.
    let old_curbuf = curbuf.get();
    let old_lnum = cur_win().w_cursor.lnum;

    if qf_ptr.qf_fnum != 0 {
        let edited =
            unsafe { qf_jump_edit_buffer(qi, qf_ptr.raw(), forceit, prev_winid, opened_window) };
        if edited != Jumped::Done {
            return edited;
        }
    }
    // Staying in the same buffer still sets the previous-context mark.
    if curbuf.get() == old_curbuf {
        setpcmark();
    }
    let lnum2 = qf_ptr.qf_lnum;
    let col = qf_ptr.qf_col;
    let viscol = qf_ptr.qf_viscol;
    let pattern = qf_ptr.qf_pattern;
    unsafe { qf_jump_goto_line(lnum2, col, viscol, pattern) };
    if fdo_flags.get() & kOptFdoFlagQuickfix as c_uint != 0 && openfold {
        unsafe { fold_open_cursor() };
    }
    if print_message {
        unsafe { qf_jump_print_msg(qi, qf_index, qf_ptr.raw(), old_curbuf, old_lnum) };
    }
    Jumped::Done
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
    // SAFETY: the caller's stack, which may be null for the quickfix one.
    let qi = qf_opt(qi).unwrap_or_else(qf_global);
    if qf_is_empty(qi) || qfl_is_empty(qf_current_list(qi)) {
        qf_emsg(&raw const e_no_errors as *const c_char);
        return;
    }
    let old_swb = p_swb.get();
    // Getting the file may reset it.

    incr_quickfix_busy();
    let mut qfl = qf_current_list(qi);
    let old_qf_ptr = qfl.qf_ptr;
    let old_qf_index = qfl.qf_index;
    let mut qf_index = old_qf_index;
    let qf_ptr = unsafe { qf_get_entry(qfl.raw(), errornr, dir, &mut qf_index) };

    // Which entry the list should be left pointing at. `None` means the
    // list is not ours to write to any more.
    let mut settle = Some((old_qf_ptr, old_qf_index));
    if !qf_ptr.is_null() {
        qfl.qf_index = qf_index;
        qfl.qf_ptr = qf_ptr;
        settle = Some((qf_ptr, qf_index));

        // No need to print the message when the quickfix window shows it.
        let print_message = !unsafe { qf_win_pos_update(qi.raw(), old_qf_index) };
        let prev_winid = cur_win().handle as c_int;
        let mut opened_window = false;

        match unsafe { qf_jump_open_window(qi.raw(), qf_ptr, newwin, &mut opened_window) } {
            // No window could be found. A window opened on the way is
            // deliberately left open, as upstream does.
            Jumped::Restore => settle = Some((old_qf_ptr, old_qf_index)),
            Jumped::Aborted => settle = None,
            // The entry named no file: stay on it, so that the next
            // `:cnext` moves past it.
            Jumped::Nowhere => {}
            Jumped::Done => {
                let oap = qi.raw();
                let count = &mut opened_window;
                let jumped = unsafe {
                    qf_jump_to_buffer(
                        oap,
                        qf_index,
                        qf_ptr,
                        forceit,
                        prev_winid,
                        count,
                        KeyTyped.get(),
                        print_message,
                    )
                };
                if jumped != Jumped::Done {
                    if opened_window {
                        unsafe { win_close(curwin.get(), true, false) };
                    }
                    if jumped == Jumped::Aborted {
                        settle = None;
                    } else if unsafe { (*qf_ptr).qf_fnum } != 0 {
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
        qfl.qf_ptr = entry;
        qfl.qf_index = index;
    }

    // Put 'switchbuf' back, unless an autocommand or a modeline changed
    // it meanwhile.
    if p_swb.get() != old_swb && is_empty_option(p_swb.get()) {
        p_swb.set(old_swb);
        swb_flags.set(swb_flags.get());
    }
    qf_busy_end();
}

/// Jump to the first entry of the list with the given id, after making that
/// list current again.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_jump_first(qi: *mut qf_info_T, save_qfid: c_uint, forceit: c_int) {
    // SAFETY: the caller's promise -- a live stack.
    let qi = unsafe { Qi::new(qi) };
    // SAFETY: as above.
    let restored = unsafe { qf_restore_list(qi.raw(), save_qfid) };
    if restored == FAIL || !check_can_set_curbuf_forceit(forceit) {
        return;
    }
    // Autocommands may have cleared the list.
    if !qfl_is_empty(qf_current_list(qi)) {
        // SAFETY: as above.
        qf_goto(qi, 0, 0, forceit);
    }
}
