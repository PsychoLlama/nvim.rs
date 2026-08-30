//! Leaving: a window, a tab page, a buffer, or the editor.
//!
//! Every quit is two questions asked in order. First, may we leave at all
//! — QuitPre and ExitPre autocommands run here and may close the window
//! under us, so everything is re-validated afterwards. Second, is anything
//! unsaved — which may put a dialog up, which may itself change the answer
//! to the first question.
//!
//! `exiting` is set *before* the checks and put back by `not_exiting` when
//! any of them refuses, because the checks themselves look at it.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::autocmd::{EVENT_EXITPRE, EVENT_QUITPRE, is_aucmd_win, may_trigger_vim_suspend_resume};

use crate::buffer::{BufRef, do_bufdel, no_write_message};

use crate::eval::vars::get_vim_var_str;

use crate::ex_cmds::do_write;
use crate::ex_cmds2::{autowrite_all, check_changed, dialog_changed};

use crate::ex_docmd::argopt::get_tabpage_arg;

use crate::ex_docmd::{
    CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, DOBUF_DEL, DOBUF_UNLOAD, DOBUF_WIPE, EXIT_FAILURE,
    cmdmod_has,
};

use crate::getchar::beep_flush;
use crate::keycodes::{Ctrl_C, KE_IGNORE, KE_XF1, KE_XF2};
use crate::main::{
    cmdwin_result, cmdwin_type, curbuf, curtab, curwin, e_autocmd_close, exiting, firstwin,
    lastwin, p_awa, p_confirm, p_write, topframe,
};

use crate::message::msg_ptr;

use crate::os::cshim::snprintf;

use crate::types::{
    CMD_SIZE, CMD_bdelete, CMD_bwipeout, CMD_close, CMD_hide, CMD_only, CMD_tabclose, CMD_tabonly,
    CMD_wq, CmdModFlags, FAIL, Failed, Integer, NUL, OK, Vv, buf_T, event_T, exarg_T, linenr_T,
    ptrdiff_t, tabpage_T, win_T,
};
use crate::ui::{ui_call_error_exit, ui_call_suspend, ui_flush};
use crate::undo::{buf_is_changed, curbuf_is_changed};

use crate::window::{
    find_tabpage, goto_tabpage, tabpage_index, trigger_tabclosedpre, valid_tabpage,
    win_close_othertab, win_goto, win_valid, window_layout_locked,
};

use crate::winlayer::{Buf, Ea, Win, WinId, first_tab, first_window, last_window, tabs, windows};

/// The key a command-line window sends back to close itself, with the
/// modifier bits an `xf1`/`xf2`/`ignore` special key carries.
const fn special_key(code: c_int) -> c_int {
    -(253 + (code << 8))
}

/// `:bdelete`, `:bwipeout` and `:bunload`.
pub(crate) unsafe fn ex_bunload(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let idx = eap.cmdidx as c_int;
    let action = if idx == CMD_bdelete as c_int {
        DOBUF_DEL
    } else if idx == CMD_bwipeout as c_int {
        DOBUF_WIPE
    } else {
        DOBUF_UNLOAD
    } as c_int;
    eap.errmsg = unsafe {
        do_bufdel(
            action,
            eap.arg,
            eap.addr_count,
            eap.line1 as c_int,
            eap.line2 as c_int,
            eap.forceit,
        )
    };
}

/// Run QuitPre, and ExitPre when this really is the last window.
///
/// Answers `true` when the quit must be abandoned. An autocommand can
/// close the window, lock the buffer or start a text operation, so both
/// events are followed by the same three-part re-validation.
///
/// # Safety
/// `wp` must be a live window on entry. It need not survive the call: the
/// autocommands may close it, which is what `quit_was_cancelled` is for.
pub(crate) unsafe fn before_quit_autocmds(wp: *mut win_T, quit_all: bool, forceit: bool) -> bool {
    // `v:exitreason` is set for the autocommands to read, and cleared
    // again if the quit does not happen.
    if byte(unsafe { get_vim_var_str(Vv::Exitreason) }) == NUL {
        set_vim_var_string(Vv::Exitreason, c"quit".as_ptr(), 4 as ptrdiff_t);
    }
    unsafe {
        apply_autocmds(
            EVENT_QUITPRE,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            (*wp).w_buffer,
        )
    };
    // The buffer is read *through* `wp`, and only after `win_valid`
    // has said `wp` is still there — QuitPre may have closed it.
    if unsafe { quit_was_cancelled(wp, || (*wp).w_buffer) } {
        return true;
    }

    // ExitPre is only for a quit that would end the process.
    if quit_all || check_more(false, forceit) == OK && only_one_window() {
        apply_autocmds(
            EVENT_EXITPRE,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        if quit_was_cancelled(wp, || curbuf.get()) {
            return true;
        }
    }
    false
}

/// Did an autocommand make the quit impossible — by closing the window,
/// locking the buffer, or starting something that must finish first?
///
/// **`buf` is a closure, and that is load-bearing.** The C's
/// `!win_valid(wp) || curbuf_locked() || (wp->w_buffer->…)` reads the
/// buffer only when the first two tests are false, because a QuitPre
/// autocommand may have closed `wp` — and an *argument* would be evaluated
/// before the call, which is a use-after-free ASan catches on
/// `test_tabpage`.
fn quit_was_cancelled(wp: *mut win_T, buf: impl FnOnce() -> *mut buf_T) -> bool {
    if win_valid(wp) && !curbuf_locked() {
        let buf = buf();
        if !(unsafe { (*buf).b_nwindows } == 1 && unsafe { (*buf).b_locked } > 0) {
            return false;
        }
    }
    set_vim_var_string(Vv::Exitreason, ptr::null(), -1 as ptrdiff_t);
    true
}

/// `:quit`.
pub(crate) unsafe fn ex_quit(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdwin_type.get() != 0 {
        // In the command-line window, `:q` closes that instead.
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let wp = if eap.addr_count > 0 {
        window_at(eap.line2)
    } else {
        curwin.get()
    };
    if curbuf_locked() {
        return;
    }
    // SAFETY: `wp` is the window this `:quit` resolved to.
    if unsafe { before_quit_autocmds(wp, false, eap.forceit != 0) } {
        return;
    }

    let save_exiting = exiting.get();
    if check_more(false, eap.forceit != 0) == OK && only_one_window() {
        exiting.set(true);
    }
    // The three refusals: unsaved changes in this buffer, files left in
    // the argument list, unsaved changes anywhere else.
    if !unsafe { buf_hide((*wp).w_buffer) }
        && unsafe {
            check_changed(
                (*wp).w_buffer,
                (if p_awa.get() != 0 {
                    CCGD_AW as c_int
                } else {
                    0
                }) | (if eap.forceit != 0 {
                    CCGD_FORCEIT as c_int
                } else {
                    0
                }) | CCGD_EXCMD as c_int,
            )
        }
        || check_more(true, eap.forceit != 0) == FAIL
        || only_one_window() && check_changed_any(eap.forceit != 0, true)
    {
        not_exiting(save_exiting);
        return;
    }
    // `:1quit` with one window open closes the window rather than
    // exiting; a bare `:quit` exits.
    if only_one_window() && (firstwin.get() == lastwin.get() || eap.addr_count == 0) {
        getout(0);
    }
    not_exiting(save_exiting);
    unsafe {
        win_close(
            wp,
            !buf_hide((*wp).w_buffer) || eap.forceit != 0,
            eap.forceit != 0,
        )
    };
}

/// The `nr`'th window of the current tab page, clamped to the last one.
fn window_at(nr: linenr_T) -> *mut win_T {
    let mut wp = first_win();
    let mut n = nr;
    while let Some(next) = wp.next() {
        n -= 1;
        if n <= 0 {
            break;
        }
        wp = next;
    }
    wp.raw()
}

/// The head of the current tab page's window list, which exists from
/// startup to exit — upstream dereferences `firstwin` here unguarded.
fn first_win() -> Win {
    first_window().expect("the editor always has a window")
}

/// `:cquit` — exit with a status, never returning.
///
/// The signature still says `()` because the command table holds one fn
/// pointer type and a `-> !` fn item does not coerce to it.
pub(crate) unsafe fn ex_cquit(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let status = if eap.addr_count > 0 {
        eap.line2 as c_int
    } else {
        EXIT_FAILURE
    };
    // Tell the UI *why* the process is about to vanish, before it does.
    ui_call_error_exit(status as Integer);
    getout(status);
}

/// The checks `:qall`, `:xall` and `:wqall` share before any of them
/// starts writing.
pub unsafe fn before_quit_all(eap: *mut exarg_T) -> Result<(), Failed> {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdwin_type.get() != 0 {
        cmdwin_result.set(special_key(if eap.forceit != 0 {
            KE_XF1 as c_int
        } else {
            KE_XF2 as c_int
        }));
        return Err(Failed);
    }
    if text_locked() {
        text_locked_msg();
        return Err(Failed);
    }
    // SAFETY: `curwin` is set from startup to exit.
    if unsafe { before_quit_autocmds(curwin.get(), true, eap.forceit != 0) } {
        return Err(Failed);
    }
    Ok(())
}

/// `:qall`.
pub(crate) unsafe fn ex_quitall(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if unsafe { before_quit_all(eap.raw()) }.is_err() {
        return;
    }
    let save_exiting = exiting.get();
    exiting.set(true);
    if eap.forceit != 0 || !check_changed_any(false, false) {
        getout(0);
    }
    not_exiting(save_exiting);
}

/// `:close`.
pub(crate) unsafe fn ex_close(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdwin_type.get() != 0 {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() || curbuf_locked() {
        return;
    }
    let win = if eap.addr_count == 0 {
        curwin.get()
    } else {
        numbered_window(eap.line2)
    };
    unsafe { ex_win_close(eap.forceit, win, ptr::null_mut()) };
}

/// The window with this number in the current tab page, or the last one.
///
/// Unlike `window_at`, this counts from one and falls back to `lastwin`
/// rather than stopping at the end.
fn numbered_window(nr: linenr_T) -> *mut win_T {
    let mut winnr = 0;
    for wp in windows() {
        winnr += 1;
        if winnr as linenr_T == nr {
            return wp.raw();
        }
    }
    last_window().map_or(ptr::null_mut(), Win::raw)
}

/// `:pclose` — close the preview window, wherever it is.
pub(crate) unsafe fn ex_pclose(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    for win in windows() {
        if win.w_onebuf_opt.wo_pvw != 0 {
            unsafe { ex_win_close(eap.forceit, win.raw(), ptr::null_mut()) };
            return;
        }
    }
}

/// Close one window, asking about unsaved changes first.
///
/// `tp` is the tab page the window belongs to, or null for this one; a
/// window in another tab page cannot simply be entered, so it takes the
/// other close path.
pub unsafe fn ex_win_close(forceit: c_int, win: *mut win_T, tp: *mut tabpage_T) {
    if is_aucmd_win(win) {
        emsg(gettext(e_autocmd_close.as_ptr()));
        return;
    }
    // A floating window is not part of the layout, so a locked layout
    // does not protect it.
    if !unsafe { (*win).w_floating } && window_layout_locked(CMD_close) {
        return;
    }

    let buf = unsafe { (*win).w_buffer };
    // Only the last window on a changed buffer has to ask.
    let mut need_hide =
        buf_is_changed(unsafe { Buf::new(buf) }) && unsafe { (*buf).b_nwindows } <= 1;
    if need_hide && !buf_hide(buf) && forceit == 0 {
        if (p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM)) && p_write.get() != 0 {
            let bufref = BufRef::of_opt(unsafe { Buf::from_raw(buf) });
            unsafe { dialog_changed(buf, false) };
            // The dialog may have wiped the buffer, or written it.
            if bufref.valid() && buf_is_changed(unsafe { Buf::new(buf) }) {
                return;
            }
            need_hide = false;
        } else {
            no_write_message();
            return;
        }
    }

    if tp.is_null() {
        win_close(win, !need_hide && !buf_hide(buf), forceit != 0);
    } else {
        unsafe {
            win_close_othertab(
                win,
                (!need_hide && !buf_hide(buf)) as c_int,
                tp,
                forceit != 0,
            )
        };
    }
}

/// `:tabclose`.
pub(crate) unsafe fn ex_tabclose(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdwin_type.get() != 0 {
        cmdwin_result.set(special_key(KE_IGNORE as c_int));
        return;
    }
    if only_tab() {
        emsg(gettext(c"E784: Cannot close last tab page".as_ptr()));
        return;
    }
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    let tab_number = get_tabpage_arg(eap);
    if eap.errmsg.is_some() {
        return;
    }
    let tp = find_tabpage(tab_number);
    if tp.is_null() {
        beep_flush();
        return;
    }
    if tp != curtab.get() {
        unsafe { tabpage_close_other(tp, eap.forceit) };
    } else if !text_locked() && !curbuf_locked() {
        unsafe { tabpage_close(eap.forceit) };
    }
}

/// `:tabonly`.
pub(crate) unsafe fn ex_tabonly(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdwin_type.get() != 0 {
        cmdwin_result.set(special_key(KE_IGNORE as c_int));
        return;
    }
    if only_tab() {
        unsafe { msg_ptr(gettext(c"Already only one tab page".as_ptr()), 0) };
        return;
    }
    if window_layout_locked(CMD_tabonly) {
        return;
    }
    let tab_number = get_tabpage_arg(eap);
    if eap.errmsg.is_some() {
        return;
    }
    goto_tabpage(tab_number);

    // Close the first tab page that is not this one, and start again —
    // closing one may close others through autocommands, so the list
    // has to be walked from the top each time. The counter is a guard
    // against a tab page that refuses to close.
    let mut done = 0;
    while done < 1000 {
        for tp in tabs() {
            if tp.tp_topframe != topframe.get() {
                unsafe { tabpage_close_other(tp.raw(), eap.forceit) };
                if valid_tabpage(tp.raw()) {
                    done = 1000;
                }
                break;
            }
        }
        debug_assert!(first_tab().is_some());
        if only_tab() {
            break;
        }
        done += 1;
    }
}

/// Close the current tab page, by closing every window in it.
pub unsafe fn tabpage_close(forceit: c_int) {
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    trigger_tabclosedpre(curtab.get());
    // The flag stops the per-window closes triggering TabClosedPre
    // again; it is cleared only if this is still the tab page it was
    // set on.
    unsafe { (*curtab.get()).tp_did_tabclosedpre = true };
    let save_curtab = curtab.get();

    while cur_win().w_floating {
        unsafe { ex_win_close(forceit, curwin.get(), ptr::null_mut()) };
    }
    if firstwin.get() != lastwin.get() {
        close_others(1, forceit);
    }
    if firstwin.get() == lastwin.get() {
        unsafe { ex_win_close(forceit, curwin.get(), ptr::null_mut()) };
    }
    if curtab.get() == save_curtab {
        unsafe { (*curtab.get()).tp_did_tabclosedpre = false };
    }
}

/// Close a tab page that is not the current one.
///
/// Its windows are closed from the last backwards; the loop stops as soon
/// as one refuses, which is what `tp_lastwin` not changing means.
pub unsafe fn tabpage_close_other(tp: *mut tabpage_T, forceit: c_int) {
    if window_layout_locked(CMD_SIZE) {
        return;
    }
    trigger_tabclosedpre(tp);
    unsafe { (*tp).tp_did_tabclosedpre = true };

    let mut done = 0;
    let mut prev_idx: [c_char; 65] = [0; 65];
    loop {
        done += 1;
        if done >= 1000 {
            break;
        }
        // Written for its side effect on `prev_idx`, which upstream
        // keeps for a message it no longer prints.
        unsafe {
            snprintf(
                &raw mut prev_idx as *mut c_char,
                size_of::<[c_char; 65]>(),
                c"%i".as_ptr(),
                tabpage_index(tp),
            )
        };
        let wp = unsafe { (*tp).tp_lastwin };
        unsafe {
            ex_win_close(
                forceit,
                wp.and_then(WinId::get).map_or(ptr::null_mut(), Win::raw),
                tp,
            )
        };
        if !valid_tabpage(tp) {
            break;
        }
        if unsafe { (*tp).tp_lastwin } == wp {
            // Nothing closed: give up.
            done = 1000;
            break;
        }
    }
    if done >= 1000 {
        unsafe { (*tp).tp_did_tabclosedpre = false };
    }
}

/// `:only`.
pub(crate) unsafe fn ex_only(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if window_layout_locked(CMD_only) {
        return;
    }
    if eap.addr_count > 0 {
        let wp = window_at_stepwise(eap.line2);
        if wp != curwin.get() {
            unsafe { win_goto(wp) };
        }
    }
    close_others(1, eap.forceit);
}

/// The `nr`'th window, counting down rather than up.
///
/// `:1only` is the *current* window: the count is spent before the walk
/// starts, unlike `window_at`, which always steps at least once.
fn window_at_stepwise(nr: linenr_T) -> *mut win_T {
    let mut wp = first_win();
    let mut n = nr;
    loop {
        n -= 1;
        let (true, Some(next)) = (n > 0, wp.next()) else {
            break;
        };
        wp = next;
    }
    wp.raw()
}

/// `:hide` used as a command rather than as a modifier.
pub(crate) unsafe fn ex_hide(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if eap.skip != 0 {
        return;
    }
    let win = if eap.addr_count == 0 {
        curwin.get()
    } else {
        numbered_window(eap.line2)
    };
    if !unsafe { (*win).w_floating } && window_layout_locked(CMD_hide) {
        return;
    }
    win_close(win, false, eap.forceit != 0);
}

/// `:stop` and `:suspend`.
pub(crate) unsafe fn ex_stop(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if eap.forceit == 0 {
        unsafe { autowrite_all() };
    }
    may_trigger_vim_suspend_resume(true);
    ui_call_suspend();
    unsafe { ui_flush() };
}

/// `:xit` and `:wq` — write, then quit.
pub(crate) unsafe fn ex_exit(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cmdwin_type.get() != 0 {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let save_exiting = exiting.get();
    if check_more(false, eap.forceit != 0) == OK && only_one_window() {
        exiting.set(true);
    }
    // `:wq` always writes; `:x` only writes a changed buffer.
    if (eap.cmdidx as c_int == CMD_wq as c_int || curbuf_is_changed())
        && unsafe { do_write(eap.raw()) }.is_err()
        // SAFETY: `curwin` is set from startup to exit.
        || unsafe { before_quit_autocmds(curwin.get(), false, eap.forceit != 0) }
        || check_more(true, eap.forceit != 0) == FAIL
        || only_one_window() && check_changed_any(eap.forceit != 0, false)
    {
        not_exiting(save_exiting);
        return;
    }
    if only_one_window() {
        getout(0);
    }
    not_exiting(save_exiting);
    win_close(
        curwin.get(),
        !buf_hide(cur_win().w_buffer),
        eap.forceit != 0,
    );
}

/// Whether the editor has exactly one tab page. Upstream's
/// `first_tabpage->tp_next == NULL`, which it writes out four times here.
fn only_tab() -> bool {
    first_tab().is_none_or(|tp| tp.next().is_none())
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// `apply_autocmds()` as checked code.
fn apply_autocmds(
    event: event_T,
    fname: *mut ::core::ffi::c_char,
    fname_io: *mut ::core::ffi::c_char,
    force: bool,
    buf: *mut buf_T,
) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::autocmd::apply_autocmds(event, fname, fname_io, force, buf) }
}

/// `buf_hide()` as checked code.
fn buf_hide(buf: *const buf_T) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::buffer::buf_hide(buf) }
}

/// `check_changed_any()` as checked code.
fn check_changed_any(hidden: bool, unload: bool) -> bool {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ex_cmds2::check_changed_any(hidden, unload) }
}

/// `check_more()` as checked code.
fn check_more(message: bool, forceit: bool) -> c_int {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ex_docmd::argopt::check_more(message, forceit) }
}

/// `close_others()` as checked code.
fn close_others(message: c_int, forceit: c_int) {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::window::close_others(message, forceit) }
}

/// `curbuf_locked()` as checked code.
fn curbuf_locked() -> bool {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ex_getln::curbuf_locked() }
}

/// `emsg()` as checked code.
fn emsg(s: *const c_char) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::emsg_ptr(s) }
}

/// `getout()` as checked code.
fn getout(exitval: c_int) -> ! {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::main::getout(exitval) }
}

/// `gettext()` as checked code.
fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    // SAFETY: a NUL-terminated message; `gettext` answers one too.
    unsafe { crate::os::cshim::gettext_ptr(__msgid).as_ptr().cast_mut() }
}

/// `not_exiting()` as checked code.
fn not_exiting(save_exiting: bool) {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ex_docmd::source::not_exiting(save_exiting) }
}

/// `only_one_window()` as checked code.
fn only_one_window() -> bool {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::window::only_one_window() }
}

/// `set_vim_var_string()` as checked code.
fn set_vim_var_string(idx: Vv, val: *const c_char, len: ptrdiff_t) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::eval::vars::set_vim_var_string(idx, val, len) }
}

/// `text_locked()` as checked code.
fn text_locked() -> bool {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ex_getln::text_locked() }
}

/// `text_locked_msg()` as checked code.
fn text_locked_msg() {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::ex_getln::text_locked_msg() }
}

/// `win_close()` as checked code.
fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> c_int {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::window::win_close(win, free_buf, force) }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}
