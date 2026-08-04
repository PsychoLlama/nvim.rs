//! Finding, opening and closing the quickfix window.
//!
//! [`ex_copen`] opens it — [`qf_goto_cwindow`] when it is already there,
//! [`qf_open_new_cwindow`] when it is not — and fills it through
//! `fill.rs`; [`ex_cwindow`] does that only when there is something to
//! show. [`qf_find_win`]/[`qf_find_buf`] are how the rest of the quickfix
//! code asks whether the window (or just its buffer) exists, and
//! [`qf_win_pos_update`] keeps its cursor on the current entry.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::types::kFalse;
use crate::src::nvim::window::{WSP_BELOW, WSP_BOT, WSP_NEWLOC, WSP_QUICKFIX, WSP_VERT};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// An option value holding a string constant.
pub(crate) const fn string_optval(text: &'static CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: static_cstring(text),
        },
    }
}

/// Call `wanted` on every window of every tab page, answering the first one
/// it accepts, or null.
///
/// # Safety
///
/// `wanted` must not close or reorder windows.
pub(crate) unsafe fn find_tab_win(mut wanted: impl FnMut(*mut win_T) -> bool) -> *mut win_T {
    // SAFETY: the window lists are walked front to back and not modified.
    unsafe {
        let mut tp = first_tabpage.get();
        while !tp.is_null() {
            // The current tab page's window list lives in `firstwin`.
            let mut wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if wanted(wp) {
                    return wp;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next;
        }
        ptr::null_mut()
    }
}

/// Whether `win` is the window showing the stack `qi`.
///
/// A window showing the quickfix buffer has no `w_llist_ref`; one showing a
/// location list buffer points at the list it shows.
///
/// # Safety
///
/// `win` and `qi` must be live.
unsafe fn shows_stack(win: *const win_T, qi: *const qf_info_T) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        buf_valid((*win).w_buffer)
            && bt_quickfix((*win).w_buffer)
            && ((*qi).qfl_type == QFLT_QUICKFIX && (*win).w_llist_ref.is_null()
                || (*qi).qfl_type == QFLT_LOCATION && ptr::eq((*win).w_llist_ref, qi))
    }
}

/// The window showing `qi` in the current tab page, or null.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_find_win(qi: *const qf_info_T) -> *mut win_T {
    // SAFETY: forwarded from the caller; only the current tab page.
    unsafe {
        let mut win = firstwin.get();
        while !win.is_null() {
            if shows_stack(win, qi) {
                return win;
            }
            win = (*win).w_next;
        }
        ptr::null_mut()
    }
}

/// The buffer the stack is shown in, from any tab page, or null.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_find_buf(qi: *mut qf_info_T) -> *mut buf_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*qi).qf_bufnr != INVALID_QFBUFNR {
            let qfbuf = buflist_findnr((*qi).qf_bufnr);
            if !qfbuf.is_null() {
                return qfbuf;
            }
            // The buffer is no longer present.
            (*qi).qf_bufnr = INVALID_QFBUFNR;
        }
        let win = find_tab_win(|wp| shows_stack(wp, qi));
        if win.is_null() {
            ptr::null_mut()
        } else {
            (*win).w_buffer
        }
    }
}

/// Go to the window showing `qi`, answering whether there was one.
///
/// With `resize` it is also given the size the command asked for, unless
/// there is no room for it below.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn qf_goto_cwindow(qi: *const qf_info_T, resize: bool, sz: c_int, vertsplit: bool) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let win = qf_find_win(qi);
        if win.is_null() {
            return false;
        }
        win_goto(win);
        if resize {
            if vertsplit {
                if sz != (*win).w_width {
                    win_setwidth(sz);
                }
            } else if sz != (*win).w_height
                && (*win).w_height
                    + (*win).w_hsep_height
                    + (*win).w_status_height
                    + tabline_height()
                    < cmdline_row.get()
            {
                win_setheight(sz);
            }
        }
        true
    }
}

/// Set the options the buffer in a quickfix or location list window wants.
///
/// # Safety
///
/// Must be called with the quickfix window current.
unsafe fn qf_set_cwindow_options() {
    // SAFETY: forwarded from the caller.
    unsafe {
        set_option_value_give_err(
            kOptSwapfile,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
            OPT_LOCAL as c_int,
        );
        set_option_value_give_err(kOptBuftype, string_optval(c"quickfix"), OPT_LOCAL as c_int);
        set_option_value_give_err(kOptBufhidden, string_optval(c"hide"), OPT_LOCAL as c_int);
        // RESET_BINDING: no 'scrollbind'/'cursorbind', and never a diff.
        (*curwin.get()).w_onebuf_opt.wo_scb = false as c_int;
        (*curwin.get()).w_onebuf_opt.wo_crb = false as c_int;
        (*curwin.get()).w_onebuf_opt.wo_diff = false as c_int;
        set_option_value_give_err(kOptFoldmethod, string_optval(c"manual"), OPT_LOCAL as c_int);
    }
}

/// Open a new quickfix or location list window, load the quickfix buffer
/// and set the window's options. Answers false when there was no room.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn qf_open_new_cwindow(qi: *mut qf_info_T, height: c_int) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut oldwin = curwin.get();
        let prevtab = curtab.get();
        let qf_buf = qf_find_buf(qi);
        // The current window becomes the previous window afterwards.
        let win = curwin.get();

        // Default is to open the window below the current one or at the
        // bottom, except when :belowright or :aboveleft is used.
        let is_qf_stack = (*qi).qfl_type == QFLT_QUICKFIX;
        let mut flags = if (*cmdmod.ptr()).cmod_split == 0 {
            if is_qf_stack {
                WSP_BOT as c_int
            } else {
                WSP_BELOW as c_int
            }
        } else {
            0
        };
        flags |= WSP_NEWLOC as c_int;
        if is_qf_stack {
            // Snapshot the layout, so closing the window can restore it.
            flags |= WSP_QUICKFIX as c_int;
        }

        if win_split(height, flags) == FAIL {
            return false; // not enough room for the window
        }
        // RESET_BINDING.
        (*curwin.get()).w_onebuf_opt.wo_scb = false as c_int;
        (*curwin.get()).w_onebuf_opt.wo_crb = false as c_int;

        if (*qi).qfl_type == QFLT_LOCATION {
            // The location list window references the stack it shows.
            (*curwin.get()).w_llist_ref = qi;
            (*qi).qf_refcount += 1;
        }

        if oldwin != curwin.get() {
            oldwin = ptr::null_mut(); // don't store info when in another window
        }
        if !qf_buf.is_null() {
            // Use the existing quickfix buffer.
            if do_ecmd(
                (*qf_buf).handle,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ECMD_ONE as linenr_T,
                ECMD_HIDE as c_int + ECMD_OLDBUF as c_int + ECMD_NOWINENTER as c_int,
                oldwin,
            ) == FAIL
            {
                return false;
            }
        } else {
            // Create a new quickfix buffer and remember its number.
            if do_ecmd(
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ECMD_ONE as linenr_T,
                ECMD_HIDE as c_int + ECMD_NOWINENTER as c_int,
                oldwin,
            ) == FAIL
            {
                return false;
            }
            (*qi).qf_bufnr = (*curbuf.get()).handle;
        }

        // Set the options for the quickfix buffer/window even if the buffer
        // was already present: an autocommand may have :bdeleted it since.
        if !bt_quickfix(curbuf.get()) {
            qf_set_cwindow_options();
        }

        // Only set the height when still in the same tab page and there is
        // no window to the side.
        if curtab.get() == prevtab && (*curwin.get()).w_width == Columns.get() {
            win_setheight(height);
        }
        (*curwin.get()).w_onebuf_opt.wo_wfh = true as c_int; // 'winfixheight'
        if win_valid(win) {
            prevwin.set(win);
        }
        true
    }
}

/// Set `w:quickfix_title` from the list's title, if it has one.
///
/// # Safety
///
/// `qfl` must be a live list, and the quickfix window current.
unsafe fn qf_set_title_var(qfl: *mut qf_list_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if !(*qfl).qf_title.is_null() {
            set_internal_string_var(c"w:quickfix_title".as_ptr(), (*qfl).qf_title);
        }
    }
}

/// Set `w:quickfix_title` in every window showing the stack, in every tab
/// page.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_update_win_titlevar(qi: *mut qf_info_T) {
    // SAFETY: forwarded from the caller. `qf_set_title_var` only writes a
    // window variable, so the window list is stable.
    unsafe {
        let qfl = qf_get_curlist(qi);
        let save_curwin = curwin.get();
        find_tab_win(|wp| {
            if shows_stack(wp, qi) {
                curwin.set(wp);
                qf_set_title_var(qfl);
            }
            false
        });
        curwin.set(save_curwin);
    }
}

/// `:copen`/`:lopen`: open a window showing the list.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_copen(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }
        incr_quickfix_busy();

        let height = if (*eap).addr_count != 0 {
            (*eap).line2 as c_int
        } else {
            QF_WINHEIGHT as c_int
        };
        reset_VIsual_and_resel(); // stop Visual mode

        // Find an existing quickfix window, or open a new one.
        let found = (*cmdmod.ptr()).cmod_tab == 0
            && qf_goto_cwindow(
                qi,
                (*eap).addr_count != 0,
                height,
                (*cmdmod.ptr()).cmod_split & WSP_VERT as c_int != 0,
            );
        if !found && !qf_open_new_cwindow(qi, height) {
            decr_quickfix_busy();
            return;
        }

        let qfl = qf_get_curlist(qi);
        qf_set_title_var(qfl);
        // Save the current index here: updating the buffer may free the
        // list.
        let lnum = (*qfl).qf_index;

        qf_fill_buffer(qfl, curbuf.get(), ptr::null_mut(), (*curwin.get()).handle);

        decr_quickfix_busy();

        (*curwin.get()).w_cursor.lnum = lnum as linenr_T;
        (*curwin.get()).w_cursor.col = 0;
        check_cursor(curwin.get());
        update_topline(curwin.get()); // scroll to show the line
    }
}

/// `:cwindow`/`:lwindow`: open the window if there is something to show,
/// close it if there is not.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cwindow(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }
        let qfl = qf_get_curlist(qi);
        let win = qf_find_win(qi);
        if qf_stack_empty(qi) || (*qfl).qf_nonevalid || qf_list_empty(qfl) {
            if !win.is_null() {
                ex_cclose(eap);
            }
        } else if win.is_null() {
            ex_copen(eap);
        }
    }
}

/// `:cclose`/`:lclose`: close the window showing the list.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cclose(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, false);
        if qi.is_null() {
            return;
        }
        let win = qf_find_win(qi);
        if !win.is_null() {
            win_close(win, false, false);
        }
    }
}

/// Move the cursor in the quickfix window to `lnum`.
///
/// # Safety
///
/// `win` must be a live window.
pub(crate) unsafe fn qf_win_goto(win: *mut win_T, lnum: linenr_T) {
    // SAFETY: forwarded from the caller. The window is made current for
    // the cursor move only; nothing in between can leave it current.
    unsafe {
        let old_curwin = curwin.get();
        curwin.set(win);
        curbuf.set((*win).w_buffer);
        (*win).w_cursor.lnum = lnum;
        (*win).w_cursor.col = 0;
        (*win).w_cursor.coladd = 0;
        (*win).w_curswant = 0;
        update_topline(win); // scroll to show the line
        redraw_later(win, UPD_VALID);
        (*win).w_redr_status = true; // update ruler
        curwin.set(old_curwin);
        curbuf.set((*old_curwin).w_buffer);
    }
}

/// `:cbottom`/`:lbottom`: put the cursor on the last line of the window.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cbottom(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }
        let win = qf_find_win(qi);
        if !win.is_null() && (*win).w_cursor.lnum != (*(*win).w_buffer).b_ml.ml_line_count {
            qf_win_goto(win, (*(*win).w_buffer).b_ml.ml_line_count);
        }
    }
}

/// The line of the quickfix window holding the current entry, which is what
/// the display code highlights.
///
/// # Safety
///
/// `wp` must be a live window showing a quickfix buffer.
pub unsafe fn qf_current_entry(wp: *mut win_T) -> linenr_T {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qi = ql_info.get();
        debug_assert!(!qi.is_null());
        if is_ll_window(wp) {
            // In the location list window, the referenced list is the one.
            qi = (*wp).w_llist_ref;
        }
        (*qf_get_curlist(qi)).qf_index as linenr_T
    }
}

/// Put the cursor of the quickfix window on the current entry, answering
/// whether there is such a window.
///
/// # Safety
///
/// `qi` must be a live stack.
pub(crate) unsafe fn qf_win_pos_update(qi: *mut qf_info_T, old_qf_index: c_int) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qf_index = (*qf_get_curlist(qi)).qf_index;
        let win = qf_find_win(qi);
        if !win.is_null()
            && qf_index as linenr_T <= (*(*win).w_buffer).b_ml.ml_line_count
            && old_qf_index != qf_index
        {
            // Both the old and the new line need redrawing.
            (*win).w_redraw_top = old_qf_index.min(qf_index) as linenr_T;
            (*win).w_redraw_bot = old_qf_index.max(qf_index) as linenr_T;
            qf_win_goto(win, qf_index as linenr_T);
        }
        !win.is_null()
    }
}

/// Process the `'quickfixtextfunc'` option value.
///
/// # Safety
///
/// Called by the option code with a live `optset_T`.
pub unsafe extern "C" fn did_set_quickfixtextfunc(_args: *mut optset_T) -> *const c_char {
    // SAFETY: the callback and the option value are the option code's.
    unsafe {
        if option_set_callback_func(p_qftf.get(), qftf_cb.ptr()) == FAIL {
            return &raw const e_invarg as *const c_char;
        }
        ptr::null()
    }
}
