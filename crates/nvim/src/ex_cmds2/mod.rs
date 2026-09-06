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
//! Every function here takes editor state by raw pointer -- the `ExArg` of
//! the command being executed, or a `Buffer`/`Window`/`Tabpage` out of one
//! of the editor's own lists -- and every one of them runs on the main
//! thread with those lists live. That is the contract the `unsafe fn`s below
//! share; each states it once by reference and does not restate it.
//!
//! What the contract does *not* buy is stability. Nearly everything here can
//! run autocommands -- a write, a buffer switch, the command `:argdo` was
//! given -- and an autocommand can delete the very buffer under examination.
//! So the `BufferRef` re-checks that follow such a call are load-bearing, and
//! a walk that a callee can invalidate restarts from `firstbuf` instead of
//! trusting the `b_next` it read before. [`buffers`] and its two siblings
//! are only for the walks where that cannot happen.
//!
//! Original: `src/nvim/ex_cmds2.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

mod listdo;

use crate::arglist::{ex_all, ex_rewind, set_arglist};
use crate::buffer::{
    BufRef, buf_hide, buf_is_dontwrite, buf_set_name, buf_spname, find_buf, no_write_message,
    no_write_message_nobang, set_curbuf,
};
use crate::bufwrite::{WriteRequest, buf_write};
use crate::change::unchanged;
use crate::channel::channel_job_running;
use crate::cstr;
use crate::drawscreen::state::cmdline_row;
use crate::eval::eval_call_provider;
use crate::eval::typval::{
    tv_list_alloc, tv_list_append_allocated_string, tv_list_append_number, tv_list_append_string,
};
use crate::eval::vars::{do_unlet, get_var_value, set_internal_string_var, set_vim_var_string};
use crate::ex_cmds::{check_overwrite, set_swapcommand};
use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::{DoCmdOpts, cmdmod_has, dialog_msg, do_cmdline, do_cmdline_cmd};
use crate::ex_getln::script_get;
use crate::fileio::{buf_check_timestamp, check_timestamps};
use crate::getchar::state::vgetc_busy;
use crate::guard::{Allow, Suppress};
use crate::highlight_group::HLF_W;
use crate::memline::MlFlags;
use crate::memory::{xfree, xstrdup};
use crate::message::state::{msg_col, msg_didany, msg_didout, msg_row};
use crate::message::{
    VIM_ALL, VIM_DISCARDALL, VIM_NO, VIM_YES, emsg, msg, msg_source, vim_dialog_yesnoallcancel,
    vim_dialog_yesnocancel, wait_return,
};
use crate::message_fmt::c_str;
use crate::option::vars::{p_aw, p_awa, p_confirm, p_write};
use crate::os::cshim::gettext;
use crate::path::vim_full_name;
use crate::runtime::{RuntimeOpts, source_runtime_vim_lua};
use crate::semsg;
use crate::startup::exiting;
use crate::types::CmdIdx;
use crate::types::{
    Buffer, CmdModFlags, ExArg, Failed, LineNr, MAXPATHL, NUL, VarNumber, Vv, ptrdiff_t, size_t,
    ssize_t, uint64_t,
};
use crate::undo::buf_is_changed;
use crate::window::goto_tabpage_win;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win, buffers, first_buffer, tabs, windows, windows_in_tab};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::eval::typval::NumBuf;
use flag::{
    CCGD_ALLBUF, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, DIALOG_MSG_SIZE, DOBUF_GOTO,
    DOBUF_UNLOAD, VIM_QUESTION,
};

pub(crate) use listdo::ex_listdo;

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::c_int;
    use crate::types::{DoBufAction, DoBufStart};

    /// `check_changed` flags.
    pub(super) const CCGD_AW: c_int = 1;
    pub(super) const CCGD_MULTWIN: c_int = 2;
    pub(super) const CCGD_FORCEIT: c_int = 4;
    pub(super) const CCGD_ALLBUF: c_int = 8;
    pub(super) const CCGD_EXCMD: c_int = 16;

    /// `do_buffer` actions and starting points.
    pub(super) const DOBUF_GOTO: DoBufAction = 0;
    pub(super) const DOBUF_UNLOAD: DoBufAction = 2;
    pub(super) const DOBUF_FIRST: DoBufStart = 1;

    /// `do_dialog` types; the answers live in `message.rs`.
    pub(super) const VIM_QUESTION: c_int = 4;

    /// The buffer `dialog_msg` formats into.
    pub(super) const DIALOG_MSG_SIZE: usize = 1000;
}

// -- List walks -------------------------------------------------------------
//
// `winlayer`'s walks are the editor's buffer, window and tab page lists; only
// the tab-page-and-window PAIR has no spelling there, because nothing else in
// the tree wants the tab page back. Use them only for walks nothing inside
// can invalidate; see the module docs.

/// Every window of every tab page, paired with the tab page holding it --
/// `FOR_ALL_TAB_WINDOWS`, which is `tabs()` followed by `windows_in_tab()`.
fn tab_windows() -> impl Iterator<Item = (TabPage, Win)> {
    tabs().flat_map(|tp| windows_in_tab(tp).map(move |wp| (tp, wp)))
}

// -- The script-host commands ----------------------------------------------
//
// `:ruby`, `:python3` and `:perl` are not implemented here at all: each one
// hands its text, its file name or its range to the provider of that name
// and lets the remote plugin host do the work.

/// `:ruby`
pub(crate) unsafe fn ex_ruby(args: *mut ExArg) {
    unsafe { script_host_execute(c"ruby", args) }
}

/// `:rubyfile`
pub(crate) unsafe fn ex_rubyfile(args: *mut ExArg) {
    unsafe { script_host_execute_file(c"ruby", args) }
}

/// `:rubydo`
pub(crate) unsafe fn ex_rubydo(args: *mut ExArg) {
    unsafe { script_host_do_range(c"ruby", args) }
}

/// `:python3`
pub(crate) unsafe fn ex_python3(args: *mut ExArg) {
    unsafe { script_host_execute(c"python3", args) }
}

/// `:py3file`
pub(crate) unsafe fn ex_py3file(args: *mut ExArg) {
    unsafe { script_host_execute_file(c"python3", args) }
}

/// `:pydo3`
pub(crate) unsafe fn ex_pydo3(args: *mut ExArg) {
    unsafe { script_host_do_range(c"python3", args) }
}

/// `:perl`
pub(crate) unsafe fn ex_perl(args: *mut ExArg) {
    unsafe { script_host_execute(c"perl", args) }
}

/// `:perlfile`
pub(crate) unsafe fn ex_perlfile(args: *mut ExArg) {
    unsafe { script_host_execute_file(c"perl", args) }
}

/// `:perldo`
pub(crate) unsafe fn ex_perldo(args: *mut ExArg) {
    unsafe { script_host_do_range(c"perl", args) }
}

/// Hand the command's own text to the provider, with the range.
///
/// # Safety
/// Module contract.
unsafe fn script_host_execute(name: &CStr, args: *mut ExArg) {
    // SAFETY: module contract; `script_get` returns an owned string that
    // `tv_list_append_allocated_string` takes over.
    let mut len: size_t = 0;
    let script = unsafe { script_get(args, &raw mut len) };
    if script.is_null() {
        return;
    }
    let argv = unsafe { tv_list_alloc(3 as ptrdiff_t) };
    unsafe { tv_list_append_allocated_string(argv, script) };
    unsafe { tv_list_append_number(argv, (*args).line1 as c_int as VarNumber) };
    unsafe { tv_list_append_number(argv, (*args).line2 as c_int as VarNumber) };
    unsafe {
        eval_call_provider(
            name.as_ptr().cast_mut(),
            c"execute".as_ptr().cast_mut(),
            argv,
            true,
        )
    };
}

/// Hand the argument, as a full path, to the provider.
///
/// # Safety
/// Module contract.
unsafe fn script_host_execute_file(name: &CStr, args: *mut ExArg) {
    // SAFETY: module contract; `buffer` is `MAXPATHL` bytes as promised.
    if unsafe { (*args).skip } != 0 {
        return;
    }
    let mut buffer: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
    let _ = unsafe { vim_full_name((*args).arg, buffer.as_mut_ptr(), MAXPATHL as usize, false) };

    let argv = unsafe { tv_list_alloc(3 as ptrdiff_t) };
    unsafe { tv_list_append_string(argv, buffer.as_ptr(), -1 as ssize_t) };
    unsafe { tv_list_append_number(argv, (*args).line1 as c_int as VarNumber) };
    unsafe { tv_list_append_number(argv, (*args).line2 as c_int as VarNumber) };
    unsafe {
        eval_call_provider(
            name.as_ptr().cast_mut(),
            c"execute_file".as_ptr().cast_mut(),
            argv,
            true,
        )
    };
}

/// Hand the range and the command's text to the provider, range first.
///
/// # Safety
/// Module contract.
unsafe fn script_host_do_range(name: &CStr, args: *mut ExArg) {
    // SAFETY: module contract.
    if unsafe { (*args).skip } != 0 {
        return;
    }
    let argv = unsafe { tv_list_alloc(3 as ptrdiff_t) };
    unsafe { tv_list_append_number(argv, (*args).line1 as c_int as VarNumber) };
    unsafe { tv_list_append_number(argv, (*args).line2 as c_int as VarNumber) };
    unsafe { tv_list_append_string(argv, (*args).arg, -1 as ssize_t) };
    unsafe {
        eval_call_provider(
            name.as_ptr().cast_mut(),
            c"do_range".as_ptr().cast_mut(),
            argv,
            true,
        )
    };
}

// -- Writing out, and asking about it --------------------------------------

/// Write `buffer` if 'autowrite' or 'autowriteall' is set.
///
/// Careful: autocommands may make `buffer` invalid.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn autowrite(buffer: *mut Buffer, forceit: bool) -> Result<(), Failed> {
    // SAFETY: module contract.
    if !(p_aw.get() != 0 || p_awa.get() != 0)
        || p_write.get() == 0
        // never autowrite a "nofile" or "nowrite" buffer
        || buf_is_dontwrite(unsafe { Buf::from_raw(buffer) })
        || (!forceit && unsafe { (*buffer) .b_p_ro } != 0)
        || unsafe { (*buffer) .b_ffname }.is_null()
    {
        return Err(Failed);
    }
    let bufref = BufRef::of_opt(unsafe { Buf::from_raw(buffer) });
    let r = unsafe { buf_write_all(Buf::new(buffer), forceit) };

    // The write can succeed and still leave the buffer changed, e.g. on
    // a conversion error. That is a failure.
    if bufref.valid() && buf_is_changed(unsafe { Buf::new(buffer) }) {
        return Err(Failed);
    }
    r
}

/// Flush every buffer except the ones that are readonly or never written.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn autowrite_all() {
    if !(p_aw.get() != 0 || p_awa.get() != 0) || p_write.get() == 0 {
        return;
    }
    // SAFETY: module contract. A write's autocommands can delete the buffer
    // being walked, which is why this is not `buffers()`: upstream resumes
    // from `firstbuf` when that happens.
    let mut cur = first_buffer();
    while let Some(b) = cur {
        if buf_is_changed(b) && b.b_p_ro == 0 && !buf_is_dontwrite(Some(b)) {
            let bufref = BufRef::of(b);
            let _ = unsafe { buf_write_all(b, false) };
            if !bufref.valid() {
                cur = first_buffer();
            }
        }
        cur = cur.and_then(Buf::next);
    }
}

/// Whether `buffer` was changed and so cannot be abandoned. `flags` is a set of
/// the `CCGD_*` values.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn check_changed(buffer: *mut Buffer, flags: c_int) -> bool {
    let forceit = flags & CCGD_FORCEIT != 0;
    // SAFETY: module contract, here and at every `unsafe` below.
    let bufref = BufRef::of_opt(unsafe { Buf::from_raw(buffer) });

    let blocked = unsafe {
        !forceit
            && buf_is_changed(Buf::new(buffer))
            && (flags & CCGD_MULTWIN != 0 || (*buffer).b_nwindows <= 1)
            && (flags & CCGD_AW == 0 || autowrite(buffer, forceit).is_err())
    };
    if !blocked {
        return false;
    }

    let confirm = (p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM)) && p_write.get() != 0;
    if !confirm {
        if flags & CCGD_EXCMD != 0 {
            no_write_message();
        } else {
            no_write_message_nobang(Buf::current());
        }
        return true;
    }

    // Ask. "Save all" is only offered when more than one buffer would want
    // saving.
    let mut count = 0;
    if flags & CCGD_ALLBUF != 0 {
        for buf2 in buffers() {
            if buf_is_changed(buf2) && !buf2.b_ffname.is_null() {
                count += 1;
            }
        }
    }
    // An autocommand may have deleted the buffer; then it is not changed now.
    if !bufref.valid() {
        return false;
    }
    unsafe { dialog_changed(Buf::new(buffer), count > 1) };
    if !bufref.valid() {
        return false;
    }
    buf_is_changed(unsafe { Buf::new(buffer) })
}

/// Ask what to do about abandoning the changed buffer `buffer`. The caller must
/// have checked 'write' first. `checkall` offers to deal with every changed
/// buffer at once.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn dialog_changed(mut buffer: Buf, checkall: bool) {
    let mut buff: [c_char; DIALOG_MSG_SIZE] = [0; DIALOG_MSG_SIZE];
    // `check_overwrite` needs an ExArg; upstream hands it an all-zero one.
    let mut ea = ExArg::default();

    // SAFETY: module contract; `buff` is `DIALOG_MSG_SIZE` bytes, as
    // `dialog_msg` requires.
    unsafe {
        dialog_msg(
            buff.as_mut_ptr(),
            c"Save changes to \"%s\"?".as_ptr().cast_mut(),
            buffer.b_fname,
        )
    };
    let ret = if checkall {
        unsafe { vim_dialog_yesnoallcancel(VIM_QUESTION, ptr::null_mut(), buff.as_mut_ptr(), 1) }
    } else {
        unsafe { vim_dialog_yesnocancel(VIM_QUESTION, ptr::null_mut(), buff.as_mut_ptr(), 1) }
    };

    if ret == VIM_YES as c_int {
        let empty_bufname = buffer.b_fname.is_null();
        if empty_bufname {
            unsafe { buf_set_name(buffer.handle as c_int, c"Untitled".as_ptr().cast_mut()) };
        }
        let target = buffer;
        if unsafe { check_overwrite(&mut ea, target, buffer.b_fname, buffer.b_ffname, false) }.is_ok()
            // didn't hit Cancel
            && unsafe { buf_write_all(buffer, false) }.is_ok()
        {
            return;
        }
        // Restore the empty name when the write failed or was cancelled.
        if empty_bufname {
            buffer.b_fname = ptr::null_mut();
            unsafe { xfree(buffer.b_ffname.cast()) };
            buffer.b_ffname = ptr::null_mut();
            unsafe { xfree(buffer.b_sfname.cast()) };
            buffer.b_sfname = ptr::null_mut();
        }
    } else if ret == VIM_NO as c_int {
        unchanged(buffer, true, false);
    } else if ret == VIM_ALL as c_int {
        unsafe { write_all_writable() };
    } else if ret == VIM_DISCARDALL as c_int {
        for buf2 in buffers() {
            unchanged(buf2, true, false);
        }
    }
}

/// The "Save All" answer: write every modified buffer that can be written.
/// Readonly ones are skipped, since those need confirming individually.
///
/// # Safety
/// Module contract.
unsafe fn write_all_writable() {
    let mut ea = ExArg::default();
    // SAFETY: module contract. As in `autowrite_all`, a write's
    // autocommands can delete the buffer being walked.
    let mut cur = first_buffer();
    while let Some(target) = cur {
        if buf_is_changed(target) && !target.b_ffname.is_null() && target.b_p_ro == 0 {
            let bufref = BufRef::of(target);
            if !target.b_fname.is_null()
                && unsafe {
                    check_overwrite(&mut ea, target, target.b_fname, target.b_ffname, false)
                }
                .is_ok()
            {
                // didn't hit Cancel
                let _ = unsafe { buf_write_all(target, false) };
            }
            if !bufref.valid() {
                cur = first_buffer();
            }
        }
        cur = cur.and_then(Buf::next);
    }
}

/// Ask whether to close the terminal buffer `buffer`.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn dialog_close_terminal(buffer: Buf) -> bool {
    let mut buff: [c_char; DIALOG_MSG_SIZE] = [0; DIALOG_MSG_SIZE];
    // SAFETY: module contract; `buff` is `DIALOG_MSG_SIZE` bytes.
    let name = if buffer.b_fname.is_null() {
        c"?".as_ptr().cast_mut()
    } else {
        buffer.b_fname
    };
    unsafe {
        dialog_msg(
            buff.as_mut_ptr(),
            c"Close \"%s\"?".as_ptr().cast_mut(),
            name,
        )
    };
    unsafe {
        vim_dialog_yesnocancel(VIM_QUESTION, ptr::null_mut(), buff.as_mut_ptr(), 1)
            == VIM_YES as c_int
    }
}

/// Whether `buffer` can be abandoned -- by hiding it, autowriting it or
/// unloading it.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn can_abandon(buffer: Buf, forceit: bool) -> bool {
    // SAFETY: module contract.
    let hidden = unsafe { buf_hide(buffer) };
    hidden
        || !buf_is_changed(buffer)
        || buffer.b_nwindows > 1
        || unsafe { autowrite(buffer.raw(), forceit) }.is_ok()
        || forceit
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
    let mut nrs = Vec::new();
    nrs.push(Buf::current().handle as c_int);
    for wp in windows().map(Win::raw) {
        if unsafe { (*wp).w_buffer } != Buf::current_raw() {
            push_unique(&mut nrs, unsafe { (*(*wp).w_buffer).handle } as c_int);
        }
    }
    for (tp, wp) in tab_windows() {
        if !tp.is_current() {
            push_unique(&mut nrs, wp.buffer().handle as c_int);
        }
    }
    for buf in buffers() {
        push_unique(&mut nrs, buf.handle as c_int);
    }
    nrs
}

/// Whether any buffer was changed and cannot be abandoned; that buffer then
/// becomes the current one.
///
/// `hidden` checks only hidden buffers. `unload` unloads the buffer rather
/// than hiding it, which is what `:q!` wants.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn check_changed_any(hidden: bool, unload: bool) -> bool {
    if first_buffer().is_none() {
        return false;
    }
    // SAFETY: module contract.
    let mut culprit = ptr::null_mut::<Buffer>();
    for nr in unsafe { changed_check_order() } {
        let buf = find_buf(nr).map_or(ptr::null_mut(), |b| b.raw());
        if buf.is_null()
            || hidden && unsafe { (*buf).b_nwindows } != 0
            || !buf_is_changed(unsafe { Buf::new(buf) })
        {
            continue;
        }
        let bufref = BufRef::of_opt(unsafe { Buf::from_raw(buf) });
        // Try auto-writing the buffer. If that fails but the buffer no
        // longer exists it is not changed, and that is fine.
        let flags = if p_awa.get() != 0 { CCGD_AW } else { 0 } | CCGD_MULTWIN | CCGD_ALLBUF;
        if unsafe { check_changed(buf, flags) } && bufref.valid() {
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
    if !(p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM)) {
        unsafe { report_unwritten(Buf::new(culprit)) };
    }

    // Try to find a window that already shows the buffer.
    if culprit != Buf::current_raw() {
        for (tp, wp) in tab_windows() {
            if wp.buffer().raw() != culprit {
                continue;
            }
            let bufref = BufRef::of_opt(unsafe { Buf::from_raw(culprit) });
            unsafe { goto_tabpage_win(tp, wp) };
            // Paranoia: did autocommands wipe out the changed buffer?
            if !bufref.valid() {
                return true;
            }
            break;
        }
    }

    // Otherwise open the changed buffer in the current window.
    if culprit != Buf::current_raw() {
        // SAFETY: a live buffer.
        let culprit = unsafe { Buf::new(culprit) };
        unsafe {
            set_curbuf(
                culprit,
                if unload { DOBUF_UNLOAD } else { DOBUF_GOTO } as c_int,
                true,
            )
        };
    }
    true
}

/// The "you have not written this" error for [`check_changed_any`], plus the
/// `wait_return` that keeps it readable when a redraw is about to follow.
///
/// # Safety
/// Module contract.
unsafe fn report_unwritten(buffer: Buf) {
    // `wait_return` is a no-op while `vgetc` is busy (Quit used from a window
    // menu); make sure the message does not scroll up then.
    if vgetc_busy.get() > 0 {
        msg_row.set(cmdline_row.get());
        msg_col.set(0);
        msg_didout.set(false);
    }
    // SAFETY: module contract.
    let shown = if !buffer.terminal.is_null()
        && unsafe { channel_job_running(buffer.b_p_channel as uint64_t) }
    {
        unsafe {
            semsg!(
                "E947: Job still running in buffer \"{}\"",
                c_str(buffer.b_fname)
            )
        }
    } else {
        let name = if unsafe { buf_spname(buffer) }.is_null() {
            buffer.b_fname
        } else {
            unsafe { buf_spname(buffer) }
        };
        unsafe {
            semsg!(
                "E162: No write since last change for buffer \"{}\"",
                c_str(name)
            )
        }
    };
    // Only makes sense if the error is shown, which `cause_errthrow` may
    // prevent.
    if shown && msg_didany.get() {
        let _prompt = Allow::wait_return();
        unsafe { wait_return(0) };
    }
}

/// `Err` and an error message when the current buffer has no file name,
/// `Ok` when it has one.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn check_fname() -> Result<(), Failed> {
    if Buf::current().b_ffname.is_null() {
        emsg(gettext(c"E32: No file name"));
        return Err(Failed);
    }
    Ok(())
}

/// Write out the whole of `buffer`.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn buf_write_all(buffer: Buf, forceit: bool) -> Result<(), Failed> {
    let old_curbuf = Buf::current_raw();
    // SAFETY: module contract.
    let retval = unsafe {
        buf_write(
            buffer,
            buffer.b_ffname,
            buffer.b_fname,
            1 as LineNr,
            buffer.b_ml.ml_line_count,
            ptr::null_mut(),
            WriteRequest {
                append: false,
                forceit,
                reset_changed: true,
                filtering: false,
            },
        )
    };
    if Buf::current_raw() != old_curbuf {
        // SAFETY: module contract.
        unsafe { msg_source(HLF_W) };
        msg(
            c"Warning: Entered other buffer unexpectedly (check autocommands)",
            0,
        );
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
pub(crate) unsafe fn ex_compiler(args: *mut ExArg) {
    let mut numbuf = NumBuf::new();
    const CURRENT_COMPILER: &CStr = c"g:current_compiler";
    const B_CURRENT_COMPILER: &CStr = c"b:current_compiler";

    // SAFETY: module contract; `args.arg` is NUL-terminated.
    if unsafe { *(*args).arg } == NUL as c_char {
        // List all compiler scripts.
        let _ = unsafe { do_cmdline_cmd(c"echo globpath(&rtp, 'compiler/*.vim')".as_ptr()) };
        let _ = unsafe { do_cmdline_cmd(c"echo globpath(&rtp, 'compiler/*.lua')".as_ptr()) };
        return;
    }

    // To stay backwards compatible "current_compiler" is always what the
    // plugin sets; "g:" is explicit so that this works inside a
    // function. Save the old value, then set "b:current_compiler" from
    // whatever the plugin leaves behind and put the old value back.
    let mut old_cur_comp = ptr::null_mut();
    if unsafe { (*args).forceit } != 0 {
        // ":compiler! {name}" sets global options.
        let cmd = c"command -nargs=* -keepscript CompilerSet set <args>".as_ptr();
        let _ = unsafe { do_cmdline_cmd(cmd) };
    } else {
        old_cur_comp = unsafe { get_var_value(CURRENT_COMPILER.as_ptr(), &mut numbuf) };
        if !old_cur_comp.is_null() {
            old_cur_comp = unsafe { xstrdup(old_cur_comp) };
        }
        let cmd = c"command -nargs=* -keepscript CompilerSet setlocal <args>".as_ptr();
        let _ = unsafe { do_cmdline_cmd(cmd) };
    }
    let (name, len) = (CURRENT_COMPILER.as_ptr(), CURRENT_COMPILER.count_bytes());
    let _ = unsafe { do_unlet(name, len, true) };
    let (name, len) = (
        B_CURRENT_COMPILER.as_ptr(),
        B_CURRENT_COMPILER.count_bytes(),
    );
    let _ = unsafe { do_unlet(name, len, true) };

    let mut pattern = Vec::with_capacity(unsafe { cstr::bytes_at((*args).arg) }.len() + 12);
    pattern.extend_from_slice(b"compiler/");
    pattern.extend_from_slice(unsafe { CStr::from_ptr((*args).arg) }.to_bytes());
    pattern.extend_from_slice(b".*\0");
    if unsafe { source_runtime_vim_lua(pattern.as_mut_ptr().cast(), RuntimeOpts::ALL) }.is_err() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str((*args).arg) };
        semsg!("E666: Compiler not supported: {arg}");
    }

    let _ = unsafe { do_cmdline_cmd(c":delcommand CompilerSet".as_ptr()) };

    // Set "b:current_compiler" from "current_compiler".
    let p = unsafe { get_var_value(CURRENT_COMPILER.as_ptr(), &mut numbuf) };
    if !p.is_null() {
        unsafe { set_internal_string_var(B_CURRENT_COMPILER.as_ptr(), p) };
    }

    // Restore "current_compiler" for ":compiler {name}".
    if unsafe { (*args).forceit } == 0 {
        if old_cur_comp.is_null() {
            let _ = unsafe {
                do_unlet(
                    CURRENT_COMPILER.as_ptr(),
                    CURRENT_COMPILER.count_bytes(),
                    true,
                )
            };
        } else {
            unsafe { set_internal_string_var(CURRENT_COMPILER.as_ptr(), old_cur_comp) };
            unsafe { xfree(old_cur_comp.cast()) };
        }
    }
}

/// `:checktime [buffer]`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_checktime(args: *mut ExArg) {
    let _checked = Allow::timestamp_checks();
    // SAFETY: module contract.
    if unsafe { (*args).addr_count } == 0 {
        // The default is all buffers.
        unsafe { check_timestamps(0) };
    } else {
        if let Some(buf) = find_buf(unsafe { (*args).line2 } as c_int) {
            // Cannot happen?
            unsafe { buf_check_timestamp(buf) };
        }
    }
}

/// `:drop`: open the first argument in a window, redefining the argument
/// list.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_drop(args: *mut ExArg) {
    // SAFETY: module contract.
    // Check whether the first argument is already being edited in a
    // window and jump there if so. Checking all of them would be
    // complicated and mostly only one file is dropped. Wildcards are
    // ignored too, since a file name containing one is very unlikely.
    unsafe { set_arglist((*args).arg) };

    // Expanding wildcards may leave the argument list empty, e.g. when
    // editing "foo.pyc" with ".pyc" in 'wildignore'. Assume an error
    // message was already given for that.
    if unsafe { (*Win::current().w_alist).al_ga.len() as c_int } == 0 {
        return;
    }

    if cmdmod.with(|m| m.cmod_tab) != 0 {
        // ":tab drop file ...": open a tab for each argument not yet
        // edited in a window. Like ":tab all" but without closing
        // windows or tabs.
        unsafe { ex_all(args) };
        cmdmod.with_mut(|m| m.cmod_tab = 0);
        unsafe { ex_rewind(args) };
        return;
    }

    // ":drop file ...": edit the first argument, jumping to an existing
    // window if there is one, editing in the current window if its
    // buffer can be abandoned, and otherwise opening a new window.
    let buf = find_buf(unsafe { *((*Win::current().w_alist).al_ga.as_mut_ptr()) }.ae_fnum)
        .map_or(ptr::null_mut(), |b| b.raw());
    for (tp, wp) in tab_windows() {
        if wp.buffer().raw() != buf {
            continue;
        }
        unsafe { goto_tabpage_win(tp, wp) };
        Win::current().w_arg_idx = 0;
        if !buf_is_changed(Buf::current()) {
            // Reload the file if it is newer.
            let save_ar = Buf::current().b_p_ar;
            Buf::current().b_p_ar = 1;
            unsafe { buf_check_timestamp(Buf::current()) };
            Buf::current().b_p_ar = save_ar;
        }
        if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
            unsafe { ex_rewind(args) };
        }
        // Execute [+cmd]. No need to execute [++opts]: those only apply
        // to newly loaded buffers.
        if !unsafe { (*args).do_ecmd_cmd }.is_null() {
            let did_set_swapcommand = unsafe { set_swapcommand((*args).do_ecmd_cmd, 0 as LineNr) };
            let verbose = DoCmdOpts::VERBOSE;
            let _ = unsafe { do_cmdline((*args).do_ecmd_cmd, None, ptr::null_mut(), verbose) };
            if did_set_swapcommand {
                unsafe { set_vim_var_string(Vv::Swapcommand, ptr::null(), -1 as ptrdiff_t) };
            }
        }
        return;
    }

    // Is the current buffer changed? If so the current window has to be
    // split or data could be lost. 'hidden' makes that unnecessary,
    // since then the buffer is not lost.
    let mut split = false;
    if !unsafe { buf_hide(Buf::current()) } {
        let _no_emsg = Suppress::emsg();
        split = unsafe { check_changed(Buf::current_raw(), CCGD_AW | CCGD_EXCMD) };
    }

    // Fake a ":sfirst" or ":first" to edit the first argument.
    if split {
        unsafe { (*args).cmdidx = CmdIdx::sfirst };
        unsafe { *(*args).cmd = b's' as c_char };
    } else {
        unsafe { (*args).cmdidx = CmdIdx::first };
    }
    unsafe { ex_rewind(args) };
}
