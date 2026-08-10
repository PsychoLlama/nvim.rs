//! The ex commands that did not fit anywhere else: the script-host shims,
//! the "may I abandon this buffer" family, `:argdo` and its seven siblings,
//! `:compiler`, `:checktime` and `:drop`.
//!
//! Two groups carry the weight of the file.
//!
//! **Abandoning a changed buffer.** [`check_changed`] answers "is this
//! buffer modified in a way that forbids leaving it", and everything that
//! wants to leave one -- `:quit`, `:edit`, `:bdelete`, closing a window --
//! routes through that answer. With 'confirm' (or the `:confirm` modifier)
//! the answer comes from a dialog instead ([`dialog_changed`]), which may
//! write the file, write *every* file, or mark them all unchanged.
//! [`check_changed_any`] is the `:qall` form: it orders the buffers
//! most-interesting-first -- the current buffer, then the current tab
//! page's, then the other tab pages', then the rest -- and reports on the
//! first one that says no, making it current so the user can see it.
//!
//! **`:argdo` and friends.** [`ex_listdo`] runs one command once per
//! argument, window, tab page, buffer or quickfix entry. Upstream tells the
//! eight commands apart by comparing `eap->cmdidx` against a `CMD_*`
//! constant at a dozen separate points; [`ListDo`] makes that decision once,
//! at the top, and the rest of the walk asks the enum.
//!
//! # Safety
//!
//! Every function here takes editor state by raw pointer -- the `exarg_T` of
//! the command being executed, or a `buf_T`/`win_T`/`tabpage_T` out of one
//! of the editor's own lists -- and every one of them runs on the main
//! thread with those lists live. That is the contract the `unsafe fn`s below
//! share; each states it once by reference and does not restate it.
//!
//! What the contract does *not* buy is stability. Nearly everything here can
//! run autocommands -- a write, a buffer switch, the command `:argdo` was
//! given -- and an autocommand can delete the very buffer under examination.
//! So the `bufref_T` re-checks that follow such a call are load-bearing, and
//! a walk that a callee can invalidate restarts from `firstbuf` instead of
//! trusting the `b_next` it read before. [`buffers`] and its two siblings
//! are only for the walks where that cannot happen.
//!
//! Original: `src/nvim/ex_cmds2.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

mod listdo;

use crate::semsg_c;
use crate::src::nvim::arglist::{ex_all, ex_rewind, set_arglist};
use crate::src::nvim::buffer::{
    bt_dontwrite, buf_hide, buf_set_name, buf_spname, buflist_findnr, bufref_valid,
    no_write_message, no_write_message_nobang, set_bufref, set_curbuf,
};
use crate::src::nvim::bufwrite::{WriteRequest, buf_write};
use crate::src::nvim::change::unchanged;
use crate::src::nvim::channel::channel_job_running;
use crate::src::nvim::eval::eval_call_provider;
use crate::src::nvim::eval::typval::{
    tv_list_alloc, tv_list_append_allocated_string, tv_list_append_number, tv_list_append_string,
};
use crate::src::nvim::eval::vars::{
    do_unlet, get_var_value, set_internal_string_var, set_vim_var_string,
};
use crate::src::nvim::ex_cmds::{check_overwrite, set_swapcommand};
use crate::src::nvim::ex_docmd::{dialog_msg, do_cmdline, do_cmdline_cmd};
use crate::src::nvim::ex_getln::script_get;
use crate::src::nvim::fileio::{buf_check_timestamp, check_timestamps};
use crate::src::nvim::highlight_group::HLF_W;
use crate::src::nvim::main::{
    cmdline_row, cmdmod, curbuf, curtab, curwin, emsg_off, exiting, first_tabpage, firstbuf,
    firstwin, msg_col, msg_didany, msg_didout, msg_row, no_check_timestamps, no_wait_return, p_aw,
    p_awa, p_confirm, p_write, vgetc_busy,
};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::{
    VIM_ALL, VIM_DISCARDALL, VIM_NO, VIM_YES, emsg, msg, msg_source, vim_dialog_yesnoallcancel,
    vim_dialog_yesnocancel, wait_return,
};
use crate::src::nvim::os::libc::strlen;
use crate::src::nvim::path::vim_FullName;
use crate::src::nvim::runtime::{DIP_ALL, source_runtime_vim_lua};
use crate::src::nvim::types::{
    CMD_first, CMD_sfirst, CMOD_CONFIRM, VV_SWAPCOMMAND, aentry_T, buf_T, bufref_T, exarg_T,
    linenr_T, ptrdiff_t, size_t, ssize_t, tabpage_T, uint64_t, varnumber_T, win_T,
};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::window::goto_tabpage_win;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use flag::{
    CCGD_ALLBUF, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, DIALOG_MSG_SIZE, DOBUF_GOTO,
    DOBUF_UNLOAD, DOCMD_VERBOSE, FAIL, MAXPATHL, ML_EMPTY, NUL, OK, VIM_QUESTION,
};

pub use listdo::ex_listdo;

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::c_int;
    use crate::src::nvim::types::{dobuf_action_values, dobuf_start_values};

    /// `buf_T.b_flags`: the Syntax autocommands ran for this buffer.
    pub const BF_SYN_SET: c_int = 0x200;

    /// `check_changed` flags.
    pub const CCGD_AW: c_int = 1;
    pub const CCGD_MULTWIN: c_int = 2;
    pub const CCGD_FORCEIT: c_int = 4;
    pub const CCGD_ALLBUF: c_int = 8;
    pub const CCGD_EXCMD: c_int = 16;

    /// `do_buffer` actions and starting points.
    pub const DOBUF_GOTO: dobuf_action_values = 0;
    pub const DOBUF_UNLOAD: dobuf_action_values = 2;
    pub const DOBUF_FIRST: dobuf_start_values = 1;

    /// `do_cmdline` flags.
    pub const DOCMD_VERBOSE: c_int = 1;
    pub const DOCMD_NOWAIT: c_int = 2;

    /// `do_dialog` types; the answers live in `message.rs`.
    pub const VIM_QUESTION: c_int = 4;

    /// The buffer `dialog_msg` formats into.
    pub const DIALOG_MSG_SIZE: usize = 1000;

    /// `memline` flags: the buffer holds a single empty line.
    pub const ML_EMPTY: c_int = 0x1;

    pub const MAXPATHL: usize = 4096;
    pub const NUL: c_char = 0;
    pub const OK: c_int = 1;
    pub const FAIL: c_int = 0;

    use core::ffi::c_char;
}

// -- List walks -------------------------------------------------------------
//
// The editor's buffer, window and tab page lists as iterators. Only for
// walks nothing inside can invalidate; see the module docs.

/// Every buffer, oldest first.
fn buffers() -> impl Iterator<Item = *mut buf_T> {
    let mut next = firstbuf.get();
    core::iter::from_fn(move || {
        let buf = next;
        if buf.is_null() {
            return None;
        }
        // SAFETY: the buffer list is the editor's own and is live.
        next = unsafe { (*buf).b_next };
        Some(buf)
    })
}

/// Every tab page, in order.
fn tabpages() -> impl Iterator<Item = *mut tabpage_T> {
    let mut next = first_tabpage.get();
    core::iter::from_fn(move || {
        let tp = next;
        if tp.is_null() {
            return None;
        }
        // SAFETY: the tab page list is the editor's own and is live.
        next = unsafe { (*tp).tp_next };
        Some(tp)
    })
}

/// Every window of `tp`. The current tab page keeps its window list in
/// `firstwin` rather than in the tab page struct, which is why this is not a
/// plain walk of `tp_firstwin`.
fn windows_in_tab(tp: *mut tabpage_T) -> impl Iterator<Item = *mut win_T> {
    let mut next = if tp == curtab.get() {
        firstwin.get()
    } else {
        // SAFETY: `tp` is a live tab page.
        unsafe { (*tp).tp_firstwin }
    };
    core::iter::from_fn(move || {
        let wp = next;
        if wp.is_null() {
            return None;
        }
        // SAFETY: the window list is the editor's own and is live.
        next = unsafe { (*wp).w_next };
        Some(wp)
    })
}

/// Every window of every tab page, paired with the tab page holding it.
fn tab_windows() -> impl Iterator<Item = (*mut tabpage_T, *mut win_T)> {
    tabpages().flat_map(|tp| windows_in_tab(tp).map(move |wp| (tp, wp)))
}

// -- The script-host commands ----------------------------------------------
//
// `:ruby`, `:python3` and `:perl` are not implemented here at all: each one
// hands its text, its file name or its range to the provider of that name
// and lets the remote plugin host do the work.

/// `:ruby`
pub unsafe fn ex_ruby(eap: *mut exarg_T) {
    unsafe { script_host_execute(c"ruby", eap) }
}

/// `:rubyfile`
pub unsafe fn ex_rubyfile(eap: *mut exarg_T) {
    unsafe { script_host_execute_file(c"ruby", eap) }
}

/// `:rubydo`
pub unsafe fn ex_rubydo(eap: *mut exarg_T) {
    unsafe { script_host_do_range(c"ruby", eap) }
}

/// `:python3`
pub unsafe fn ex_python3(eap: *mut exarg_T) {
    unsafe { script_host_execute(c"python3", eap) }
}

/// `:py3file`
pub unsafe fn ex_py3file(eap: *mut exarg_T) {
    unsafe { script_host_execute_file(c"python3", eap) }
}

/// `:pydo3`
pub unsafe fn ex_pydo3(eap: *mut exarg_T) {
    unsafe { script_host_do_range(c"python3", eap) }
}

/// `:perl`
pub unsafe fn ex_perl(eap: *mut exarg_T) {
    unsafe { script_host_execute(c"perl", eap) }
}

/// `:perlfile`
pub unsafe fn ex_perlfile(eap: *mut exarg_T) {
    unsafe { script_host_execute_file(c"perl", eap) }
}

/// `:perldo`
pub unsafe fn ex_perldo(eap: *mut exarg_T) {
    unsafe { script_host_do_range(c"perl", eap) }
}

/// Hand the command's own text to the provider, with the range.
///
/// # Safety
/// Module contract.
unsafe fn script_host_execute(name: &CStr, eap: *mut exarg_T) {
    // SAFETY: module contract; `script_get` returns an owned string that
    // `tv_list_append_allocated_string` takes over.
    unsafe {
        let mut len: size_t = 0;
        let script = script_get(eap, &raw mut len);
        if script.is_null() {
            return;
        }
        let args = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_allocated_string(args, script);
        tv_list_append_number(args, (*eap).line1 as c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as c_int as varnumber_T);
        eval_call_provider(
            name.as_ptr().cast_mut(),
            c"execute".as_ptr().cast_mut(),
            args,
            true,
        );
    }
}

/// Hand the argument, as a full path, to the provider.
///
/// # Safety
/// Module contract.
unsafe fn script_host_execute_file(name: &CStr, eap: *mut exarg_T) {
    // SAFETY: module contract; `buffer` is `MAXPATHL` bytes as promised.
    unsafe {
        if (*eap).skip != 0 {
            return;
        }
        let mut buffer: [c_char; MAXPATHL] = [0; MAXPATHL];
        vim_FullName((*eap).arg, buffer.as_mut_ptr(), MAXPATHL, false);

        let args = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_string(args, buffer.as_ptr(), -1 as ssize_t);
        tv_list_append_number(args, (*eap).line1 as c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as c_int as varnumber_T);
        eval_call_provider(
            name.as_ptr().cast_mut(),
            c"execute_file".as_ptr().cast_mut(),
            args,
            true,
        );
    }
}

/// Hand the range and the command's text to the provider, range first.
///
/// # Safety
/// Module contract.
unsafe fn script_host_do_range(name: &CStr, eap: *mut exarg_T) {
    // SAFETY: module contract.
    unsafe {
        if (*eap).skip != 0 {
            return;
        }
        let args = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_number(args, (*eap).line1 as c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as c_int as varnumber_T);
        tv_list_append_string(args, (*eap).arg, -1 as ssize_t);
        eval_call_provider(
            name.as_ptr().cast_mut(),
            c"do_range".as_ptr().cast_mut(),
            args,
            true,
        );
    }
}

// -- Writing out, and asking about it --------------------------------------

/// Write `buf` if 'autowrite' or 'autowriteall' is set.
///
/// Careful: autocommands may make `buf` invalid.
///
/// # Safety
/// Module contract.
pub unsafe fn autowrite(buf: *mut buf_T, forceit: bool) -> c_int {
    // SAFETY: module contract.
    unsafe {
        if !(p_aw.get() != 0 || p_awa.get() != 0)
            || p_write.get() == 0
            // never autowrite a "nofile" or "nowrite" buffer
            || bt_dontwrite(buf)
            || (!forceit && (*buf).b_p_ro != 0)
            || (*buf).b_ffname.is_null()
        {
            return FAIL;
        }
        let mut bufref = bufref_T::default();
        set_bufref(&raw mut bufref, buf);
        let r = buf_write_all(buf, forceit);

        // The write can succeed and still leave the buffer changed, e.g. on
        // a conversion error. That is a failure.
        if bufref_valid(&raw mut bufref) && bufIsChanged(buf) {
            return FAIL;
        }
        r
    }
}

/// Flush every buffer except the ones that are readonly or never written.
///
/// # Safety
/// Module contract.
pub unsafe fn autowrite_all() {
    if !(p_aw.get() != 0 || p_awa.get() != 0) || p_write.get() == 0 {
        return;
    }
    // SAFETY: module contract. A write's autocommands can delete the buffer
    // being walked, which is why this is not `buffers()`: upstream resumes
    // from `firstbuf` when that happens.
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if bufIsChanged(buf) && (*buf).b_p_ro == 0 && !bt_dontwrite(buf) {
                let mut bufref = bufref_T::default();
                set_bufref(&raw mut bufref, buf);
                buf_write_all(buf, false);
                if !bufref_valid(&raw mut bufref) {
                    buf = firstbuf.get();
                }
            }
            buf = (*buf).b_next;
        }
    }
}

/// Whether `buf` was changed and so cannot be abandoned. `flags` is a set of
/// the `CCGD_*` values.
///
/// # Safety
/// Module contract.
pub unsafe fn check_changed(buf: *mut buf_T, flags: c_int) -> bool {
    let forceit = flags & CCGD_FORCEIT != 0;
    let mut bufref = bufref_T::default();
    // SAFETY: module contract, here and at every `unsafe` below.
    unsafe { set_bufref(&raw mut bufref, buf) };

    let blocked = unsafe {
        !forceit
            && bufIsChanged(buf)
            && (flags & CCGD_MULTWIN != 0 || (*buf).b_nwindows <= 1)
            && (flags & CCGD_AW == 0 || autowrite(buf, forceit) == FAIL)
    };
    if !blocked {
        return false;
    }

    let confirm = (p_confirm.get() != 0
        || cmdmod.with(|m| m.cmod_flags) & CMOD_CONFIRM as c_int != 0)
        && p_write.get() != 0;
    if !confirm {
        unsafe {
            if flags & CCGD_EXCMD != 0 {
                no_write_message();
            } else {
                no_write_message_nobang(curbuf.get());
            }
        }
        return true;
    }

    // Ask. "Save all" is only offered when more than one buffer would want
    // saving.
    let mut count = 0;
    if flags & CCGD_ALLBUF != 0 {
        for buf2 in buffers() {
            if unsafe { bufIsChanged(buf2) && !(*buf2).b_ffname.is_null() } {
                count += 1;
            }
        }
    }
    // An autocommand may have deleted the buffer; then it is not changed now.
    if !unsafe { bufref_valid(&raw mut bufref) } {
        return false;
    }
    unsafe { dialog_changed(buf, count > 1) };
    if !unsafe { bufref_valid(&raw mut bufref) } {
        return false;
    }
    unsafe { bufIsChanged(buf) }
}

/// Ask what to do about abandoning the changed buffer `buf`. The caller must
/// have checked 'write' first. `checkall` offers to deal with every changed
/// buffer at once.
///
/// # Safety
/// Module contract.
pub unsafe fn dialog_changed(buf: *mut buf_T, checkall: bool) {
    let mut buff: [c_char; DIALOG_MSG_SIZE] = [0; DIALOG_MSG_SIZE];
    // `check_overwrite` needs an exarg_T; upstream hands it an all-zero one.
    let mut ea = exarg_T::default();

    // SAFETY: module contract; `buff` is `DIALOG_MSG_SIZE` bytes, as
    // `dialog_msg` requires.
    unsafe {
        dialog_msg(
            buff.as_mut_ptr(),
            c"Save changes to \"%s\"?".as_ptr().cast_mut(),
            (*buf).b_fname,
        );
        let ret = if checkall {
            vim_dialog_yesnoallcancel(VIM_QUESTION, ptr::null_mut(), buff.as_mut_ptr(), 1)
        } else {
            vim_dialog_yesnocancel(VIM_QUESTION, ptr::null_mut(), buff.as_mut_ptr(), 1)
        };

        if ret == VIM_YES as c_int {
            let empty_bufname = (*buf).b_fname.is_null();
            if empty_bufname {
                buf_set_name((*buf).handle as c_int, c"Untitled".as_ptr().cast_mut());
            }
            if check_overwrite(&raw mut ea, buf, (*buf).b_fname, (*buf).b_ffname, false) == OK
                // didn't hit Cancel
                && buf_write_all(buf, false) == OK
            {
                return;
            }
            // Restore the empty name when the write failed or was cancelled.
            if empty_bufname {
                (*buf).b_fname = ptr::null_mut();
                xfree((*buf).b_ffname.cast());
                (*buf).b_ffname = ptr::null_mut();
                xfree((*buf).b_sfname.cast());
                (*buf).b_sfname = ptr::null_mut();
            }
        } else if ret == VIM_NO as c_int {
            unchanged(buf, true, false);
        } else if ret == VIM_ALL as c_int {
            write_all_writable();
        } else if ret == VIM_DISCARDALL as c_int {
            for buf2 in buffers() {
                unchanged(buf2, true, false);
            }
        }
    }
}

/// The "Save All" answer: write every modified buffer that can be written.
/// Readonly ones are skipped, since those need confirming individually.
///
/// # Safety
/// Module contract.
unsafe fn write_all_writable() {
    let mut ea = exarg_T::default();
    // SAFETY: module contract. As in `autowrite_all`, a write's
    // autocommands can delete the buffer being walked.
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if bufIsChanged(buf) && !(*buf).b_ffname.is_null() && (*buf).b_p_ro == 0 {
                let mut bufref = bufref_T::default();
                set_bufref(&raw mut bufref, buf);
                if !(*buf).b_fname.is_null()
                    && check_overwrite(&raw mut ea, buf, (*buf).b_fname, (*buf).b_ffname, false)
                        == OK
                {
                    // didn't hit Cancel
                    buf_write_all(buf, false);
                }
                if !bufref_valid(&raw mut bufref) {
                    buf = firstbuf.get();
                }
            }
            buf = (*buf).b_next;
        }
    }
}

/// Ask whether to close the terminal buffer `buf`.
///
/// # Safety
/// Module contract.
pub unsafe fn dialog_close_terminal(buf: *mut buf_T) -> bool {
    let mut buff: [c_char; DIALOG_MSG_SIZE] = [0; DIALOG_MSG_SIZE];
    // SAFETY: module contract; `buff` is `DIALOG_MSG_SIZE` bytes.
    unsafe {
        let name = if (*buf).b_fname.is_null() {
            c"?".as_ptr().cast_mut()
        } else {
            (*buf).b_fname
        };
        dialog_msg(
            buff.as_mut_ptr(),
            c"Close \"%s\"?".as_ptr().cast_mut(),
            name,
        );
        vim_dialog_yesnocancel(VIM_QUESTION, ptr::null_mut(), buff.as_mut_ptr(), 1)
            == VIM_YES as c_int
    }
}

/// Whether `buf` can be abandoned -- by hiding it, autowriting it or
/// unloading it.
///
/// # Safety
/// Module contract.
pub unsafe fn can_abandon(buf: *mut buf_T, forceit: bool) -> bool {
    // SAFETY: module contract.
    unsafe {
        buf_hide(buf)
            || !bufIsChanged(buf)
            || (*buf).b_nwindows > 1
            || autowrite(buf, forceit) == OK
            || forceit
    }
}

/// The buffers to ask about, most interesting first: the current buffer, the
/// current tab page's, the other tab pages', then everything else. Each
/// buffer number appears once.
///
/// # Safety
/// Module contract, and there is at least one buffer.
unsafe fn changed_check_order() -> Vec<c_int> {
    fn push_unique(nrs: &mut Vec<c_int>, nr: c_int) {
        if !nrs.contains(&nr) {
            nrs.push(nr);
        }
    }

    // SAFETY: caller contract; none of these walks runs editor code.
    unsafe {
        let mut nrs = Vec::new();
        nrs.push((*curbuf.get()).handle as c_int);
        for wp in windows_in_tab(curtab.get()) {
            if (*wp).w_buffer != curbuf.get() {
                push_unique(&mut nrs, (*(*wp).w_buffer).handle as c_int);
            }
        }
        for (tp, wp) in tab_windows() {
            if tp != curtab.get() {
                push_unique(&mut nrs, (*(*wp).w_buffer).handle as c_int);
            }
        }
        for buf in buffers() {
            push_unique(&mut nrs, (*buf).handle as c_int);
        }
        nrs
    }
}

/// Whether any buffer was changed and cannot be abandoned; that buffer then
/// becomes the current one.
///
/// `hidden` checks only hidden buffers. `unload` unloads the buffer rather
/// than hiding it, which is what `:q!` wants.
///
/// # Safety
/// Module contract.
pub unsafe fn check_changed_any(hidden: bool, unload: bool) -> bool {
    if firstbuf.get().is_null() {
        return false;
    }
    // SAFETY: module contract.
    unsafe {
        let mut culprit = ptr::null_mut::<buf_T>();
        for nr in changed_check_order() {
            let buf = buflist_findnr(nr);
            if buf.is_null() || (hidden && (*buf).b_nwindows != 0) || !bufIsChanged(buf) {
                continue;
            }
            let mut bufref = bufref_T::default();
            set_bufref(&raw mut bufref, buf);
            // Try auto-writing the buffer. If that fails but the buffer no
            // longer exists it is not changed, and that is fine.
            let flags = if p_awa.get() != 0 { CCGD_AW } else { 0 } | CCGD_MULTWIN | CCGD_ALLBUF;
            if check_changed(buf, flags) && bufref_valid(&raw mut bufref) {
                // Didn't save -- still changed.
                culprit = buf;
                break;
            }
        }
        if culprit.is_null() {
            return false;
        }

        exiting.set(false);
        // With ":confirm" the dialog was the message; do not add an error.
        if !(p_confirm.get() != 0 || cmdmod.with(|m| m.cmod_flags) & CMOD_CONFIRM as c_int != 0) {
            report_unwritten(culprit);
        }

        // Try to find a window that already shows the buffer.
        if culprit != curbuf.get() {
            for (tp, wp) in tab_windows() {
                if (*wp).w_buffer != culprit {
                    continue;
                }
                let mut bufref = bufref_T::default();
                set_bufref(&raw mut bufref, culprit);
                goto_tabpage_win(tp, wp);
                // Paranoia: did autocommands wipe out the changed buffer?
                if !bufref_valid(&raw mut bufref) {
                    return true;
                }
                break;
            }
        }

        // Otherwise open the changed buffer in the current window.
        if culprit != curbuf.get() {
            set_curbuf(
                culprit,
                if unload { DOBUF_UNLOAD } else { DOBUF_GOTO } as c_int,
                true,
            );
        }
        true
    }
}

/// The "you have not written this" error for [`check_changed_any`], plus the
/// `wait_return` that keeps it readable when a redraw is about to follow.
///
/// # Safety
/// Module contract.
unsafe fn report_unwritten(buf: *mut buf_T) {
    // `wait_return` is a no-op while `vgetc` is busy (Quit used from a window
    // menu); make sure the message does not scroll up then.
    if vgetc_busy.get() > 0 {
        msg_row.set(cmdline_row.get());
        msg_col.set(0);
        msg_didout.set(false);
    }
    // SAFETY: module contract.
    unsafe {
        let shown =
            if !(*buf).terminal.is_null() && channel_job_running((*buf).b_p_channel as uint64_t) {
                semsg_c!(
                    c"E947: Job still running in buffer \"%s\"".as_ptr(),
                    (*buf).b_fname
                )
            } else {
                let name = if buf_spname(buf).is_null() {
                    (*buf).b_fname
                } else {
                    buf_spname(buf)
                };
                semsg_c!(
                    c"E162: No write since last change for buffer \"%s\"".as_ptr(),
                    name,
                )
            };
        // Only makes sense if the error is shown, which `cause_errthrow` may
        // prevent.
        if shown && msg_didany.get() {
            let save = no_wait_return.get();
            no_wait_return.set(0);
            wait_return(0);
            no_wait_return.set(save);
        }
    }
}

/// `FAIL` and an error message when the current buffer has no file name,
/// `OK` when it has one.
///
/// # Safety
/// Module contract.
pub unsafe fn check_fname() -> c_int {
    // SAFETY: module contract.
    unsafe {
        if (*curbuf.get()).b_ffname.is_null() {
            emsg(c"E32: No file name".as_ptr());
            return FAIL;
        }
    }
    OK
}

/// Write out the whole of `buf`.
///
/// # Safety
/// Module contract.
pub unsafe fn buf_write_all(buf: *mut buf_T, forceit: bool) -> c_int {
    let old_curbuf = curbuf.get();
    // SAFETY: module contract.
    let retval = unsafe {
        buf_write(
            buf,
            (*buf).b_ffname,
            (*buf).b_fname,
            1 as linenr_T,
            (*buf).b_ml.ml_line_count,
            ptr::null_mut(),
            WriteRequest {
                append: false,
                forceit,
                reset_changed: true,
                filtering: false,
            },
        )
    };
    if curbuf.get() != old_curbuf {
        // SAFETY: module contract.
        unsafe {
            msg_source(HLF_W);
            msg(
                c"Warning: Entered other buffer unexpectedly (check autocommands)".as_ptr(),
                0,
            );
        }
    }
    retval
}

// -- The rest ---------------------------------------------------------------

/// `:compiler[!] {name}`
///
/// The compiler plugin is expected to set `current_compiler`, so the name is
/// unlet first and read back afterwards. Without `!` the setting is local to
/// the buffer, which means saving and restoring the global the plugin wrote.
///
/// # Safety
/// Module contract.
pub unsafe fn ex_compiler(eap: *mut exarg_T) {
    const CURRENT_COMPILER: &CStr = c"g:current_compiler";
    const B_CURRENT_COMPILER: &CStr = c"b:current_compiler";

    // SAFETY: module contract; `eap->arg` is NUL-terminated.
    unsafe {
        if *(*eap).arg == NUL {
            // List all compiler scripts.
            do_cmdline_cmd(c"echo globpath(&rtp, 'compiler/*.vim')".as_ptr());
            do_cmdline_cmd(c"echo globpath(&rtp, 'compiler/*.lua')".as_ptr());
            return;
        }

        // To stay backwards compatible "current_compiler" is always what the
        // plugin sets; "g:" is explicit so that this works inside a
        // function. Save the old value, then set "b:current_compiler" from
        // whatever the plugin leaves behind and put the old value back.
        let mut old_cur_comp = ptr::null_mut();
        if (*eap).forceit != 0 {
            // ":compiler! {name}" sets global options.
            do_cmdline_cmd(c"command -nargs=* -keepscript CompilerSet set <args>".as_ptr());
        } else {
            old_cur_comp = get_var_value(CURRENT_COMPILER.as_ptr());
            if !old_cur_comp.is_null() {
                old_cur_comp = xstrdup(old_cur_comp);
            }
            do_cmdline_cmd(c"command -nargs=* -keepscript CompilerSet setlocal <args>".as_ptr());
        }
        do_unlet(
            CURRENT_COMPILER.as_ptr(),
            CURRENT_COMPILER.count_bytes(),
            true,
        );
        do_unlet(
            B_CURRENT_COMPILER.as_ptr(),
            B_CURRENT_COMPILER.count_bytes(),
            true,
        );

        let mut pattern = Vec::with_capacity(strlen((*eap).arg) + 12);
        pattern.extend_from_slice(b"compiler/");
        pattern.extend_from_slice(CStr::from_ptr((*eap).arg).to_bytes());
        pattern.extend_from_slice(b".*\0");
        if source_runtime_vim_lua(pattern.as_mut_ptr().cast(), DIP_ALL as c_int) == FAIL {
            semsg_c!(c"E666: Compiler not supported: %s".as_ptr(), (*eap).arg);
        }

        do_cmdline_cmd(c":delcommand CompilerSet".as_ptr());

        // Set "b:current_compiler" from "current_compiler".
        let p = get_var_value(CURRENT_COMPILER.as_ptr());
        if !p.is_null() {
            set_internal_string_var(B_CURRENT_COMPILER.as_ptr(), p);
        }

        // Restore "current_compiler" for ":compiler {name}".
        if (*eap).forceit == 0 {
            if old_cur_comp.is_null() {
                do_unlet(
                    CURRENT_COMPILER.as_ptr(),
                    CURRENT_COMPILER.count_bytes(),
                    true,
                );
            } else {
                set_internal_string_var(CURRENT_COMPILER.as_ptr(), old_cur_comp);
                xfree(old_cur_comp.cast());
            }
        }
    }
}

/// `:checktime [buffer]`
///
/// # Safety
/// Module contract.
pub unsafe fn ex_checktime(eap: *mut exarg_T) {
    let save_no_check_timestamps = no_check_timestamps.get();
    no_check_timestamps.set(0);
    // SAFETY: module contract.
    unsafe {
        if (*eap).addr_count == 0 {
            // The default is all buffers.
            check_timestamps(0);
        } else {
            let buf = buflist_findnr((*eap).line2 as c_int);
            if !buf.is_null() {
                // Cannot happen?
                buf_check_timestamp(buf);
            }
        }
    }
    no_check_timestamps.set(save_no_check_timestamps);
}

/// `:drop`: open the first argument in a window, redefining the argument
/// list.
///
/// # Safety
/// Module contract.
pub unsafe fn ex_drop(eap: *mut exarg_T) {
    // SAFETY: module contract.
    unsafe {
        // Check whether the first argument is already being edited in a
        // window and jump there if so. Checking all of them would be
        // complicated and mostly only one file is dropped. Wildcards are
        // ignored too, since a file name containing one is very unlikely.
        set_arglist((*eap).arg);

        // Expanding wildcards may leave the argument list empty, e.g. when
        // editing "foo.pyc" with ".pyc" in 'wildignore'. Assume an error
        // message was already given for that.
        if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 {
            return;
        }

        if cmdmod.with(|m| m.cmod_tab) != 0 {
            // ":tab drop file ...": open a tab for each argument not yet
            // edited in a window. Like ":tab all" but without closing
            // windows or tabs.
            ex_all(eap);
            cmdmod.with_mut(|m| m.cmod_tab = 0);
            ex_rewind(eap);
            return;
        }

        // ":drop file ...": edit the first argument, jumping to an existing
        // window if there is one, editing in the current window if its
        // buffer can be abandoned, and otherwise opening a new window.
        let buf =
            buflist_findnr((*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)).ae_fnum);
        for (tp, wp) in tab_windows() {
            if (*wp).w_buffer != buf {
                continue;
            }
            goto_tabpage_win(tp, wp);
            (*curwin.get()).w_arg_idx = 0;
            if !bufIsChanged(curbuf.get()) {
                // Reload the file if it is newer.
                let save_ar = (*curbuf.get()).b_p_ar;
                (*curbuf.get()).b_p_ar = 1;
                buf_check_timestamp(curbuf.get());
                (*curbuf.get()).b_p_ar = save_ar;
            }
            if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                ex_rewind(eap);
            }
            // Execute [+cmd]. No need to execute [++opts]: those only apply
            // to newly loaded buffers.
            if !(*eap).do_ecmd_cmd.is_null() {
                let did_set_swapcommand = set_swapcommand((*eap).do_ecmd_cmd, 0 as linenr_T);
                do_cmdline((*eap).do_ecmd_cmd, None, ptr::null_mut(), DOCMD_VERBOSE);
                if did_set_swapcommand {
                    set_vim_var_string(VV_SWAPCOMMAND, ptr::null(), -1 as ptrdiff_t);
                }
            }
            return;
        }

        // Is the current buffer changed? If so the current window has to be
        // split or data could be lost. 'hidden' makes that unnecessary,
        // since then the buffer is not lost.
        let mut split = false;
        if !buf_hide(curbuf.get()) {
            emsg_off.set(emsg_off.get() + 1);
            split = check_changed(curbuf.get(), CCGD_AW | CCGD_EXCMD);
            emsg_off.set(emsg_off.get() - 1);
        }

        // Fake a ":sfirst" or ":first" to edit the first argument.
        if split {
            (*eap).cmdidx = CMD_sfirst;
            *(*eap).cmd = b's' as c_char;
        } else {
            (*eap).cmdidx = CMD_first;
        }
        ex_rewind(eap);
    }
}
