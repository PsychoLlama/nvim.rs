//! Commands that name a file or a buffer: reading, editing, finding,
//! recovering, and the buffer list.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::fileio::Loaded;
use crate::guard::Allow;
use crate::memline::MlFlags;
use crate::semsg;
use crate::types::CmdIdx;
use crate::window::valid_win;
use crate::winlayer::WinId;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::arglist::check_arg_idx;
use crate::buffer::{buf_is_prompt, current_buf, maketitle, otherfile, setaltfname, setfname};

use crate::change::deleted_lines_mark;
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_curbuf_later};

use crate::ex_cmds2::{check_changed, check_fname};
use crate::ex_docmd::cmdline::do_cmdline_cmd;
use crate::ex_docmd::path::findfunc_find_file;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::xfree;
use crate::ex_docmd::{
    ACTION_SHOW, ACTION_SHOW_ALL, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, CHECK_PATH,
    DOBUF_CURRENT, DOBUF_FIRST, DOBUF_LAST, DOBUF_MOD, cmdmod_has, ex_pressedreturn,
    kDirectionNotSet,
};
use crate::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::ex_getln::{text_or_buf_locked, ui_ext_cmdline_block_leave};
use crate::file_search::{FileNameOpts, vim_findfile_cleanup};

use crate::ex_docmd::state::{ex_no_reprint, global_busy};
use crate::getchar::stuff_readbuf;
use crate::mark::setpcmark;
use crate::message::e_trailing_arg;
use crate::message::state::{msg_scroll, need_wait_return};
use crate::option::vars::{p_awa, p_shada};
use crate::startup::{readonlymode, recoverymode};
use crate::state::mode::{exmode_active, pending_exmode_active};

use crate::memline::{ml_delete, ml_get, ml_preserve, ml_recover};

use crate::message::emsg;
use crate::message_fmt::c_str;

use crate::normal::normal_enter;
use crate::option::{cpo_has, get_findfunc};

use crate::path::path_fnamecmp;
use crate::pos::MAXLNUM;
use crate::search::{BACKWARD, FORWARD, find_pattern_in_path};
use crate::shada::{shada_read_everything, shada_write_file};
use crate::types::ui::kUICmdline;
use crate::types::{
    Cleanup, CmdModFlags, CpoFlag, ExArg, Failed, LineNr, MemFile, NUL, size_t, uint8_t,
};
use crate::ui::ui_has;
use crate::undo::{curbuf_is_changed, u_read_undo, u_save, u_savedel, u_write_undo};

use crate::window::{check_can_set_curbuf_forceit, win_close};
use crate::winfloat::win_float_remove;
use crate::winlayer::{Buf, Ea, Win};

/// Would editing `fnum`/`ffname` mean leaving the current buffer?
///
/// A buffer whose file could not be stat'ed is compared by its *short*
/// name, because the full name may have been resolved against a directory
/// that no longer exists.
pub(crate) unsafe fn is_other_file(fnum: c_int, ffname: *mut c_char) -> bool {
    if fnum != 0 {
        return fnum != Buf::current().handle;
    }
    if ffname.is_null() {
        return true;
    }
    // An empty name means "this buffer", not "no buffer".
    if byte(ffname) == NUL {
        return false;
    }
    if !Buf::current().file_id_valid
        && !Buf::current().b_sfname.is_null()
        && byte(Buf::current().b_sfname) != NUL
    {
        return unsafe { path_fnamecmp(ffname, Buf::current().b_sfname) } != 0;
    }
    unsafe { otherfile(ffname) }
}

/// `:buffer`.
pub(crate) unsafe fn ex_buffer(args: *mut ExArg) {
    do_exbuffer(unsafe { Ea::new(args) });
}

/// `:buffer`, shared with `:pbuffer`.
pub(crate) fn do_exbuffer(mut args: Ea) {
    // The buffer was already resolved from the argument by
    // `execute_cmd0`'s `ExArgt::BUFNAME` handling, so anything left is junk.
    if unsafe { *args.arg } != 0 {
        args.errmsg = Some(unsafe { ex_errmsg(e_trailing_arg.as_ptr(), args.arg) });
        return;
    }
    if args.addr_count == 0 {
        goto_buffer(args.raw(), DOBUF_CURRENT as c_int, FORWARD as c_int, 0);
    } else {
        goto_buffer(
            args.raw(),
            DOBUF_FIRST as c_int,
            FORWARD as c_int,
            args.line2 as c_int,
        );
    }
    run_ecmd_cmd(args);
}

/// Run the `+cmd` argument, once the buffer it applies to is current.
fn run_ecmd_cmd(args: Ea) {
    if !args.do_ecmd_cmd.is_null() {
        let _ = unsafe { do_cmdline_cmd(args.do_ecmd_cmd) };
    }
}

/// `:bmodified`.
pub(crate) unsafe fn ex_bmodified(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    goto_buffer(
        args.raw(),
        DOBUF_MOD as c_int,
        FORWARD as c_int,
        args.line2 as c_int,
    );
    run_ecmd_cmd(args);
}

/// `:bnext`.
pub(crate) unsafe fn ex_bnext(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    goto_buffer(
        args.raw(),
        DOBUF_CURRENT as c_int,
        FORWARD as c_int,
        args.line2 as c_int,
    );
    run_ecmd_cmd(args);
}

/// `:bprevious` and `:bNext`.
pub(crate) unsafe fn ex_bprevious(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    goto_buffer(
        args.raw(),
        DOBUF_CURRENT as c_int,
        BACKWARD as c_int,
        args.line2 as c_int,
    );
    run_ecmd_cmd(args);
}

/// `:brewind` and `:bfirst`.
pub(crate) unsafe fn ex_brewind(args: *mut ExArg) {
    goto_buffer(args, DOBUF_FIRST as c_int, FORWARD as c_int, 0);
    run_ecmd_cmd(unsafe { Ea::new(args) });
}

/// `:blast`.
pub(crate) unsafe fn ex_blast(args: *mut ExArg) {
    goto_buffer(args, DOBUF_LAST as c_int, BACKWARD as c_int, 0);
    run_ecmd_cmd(unsafe { Ea::new(args) });
}

/// `:preserve` — flush the swap file to disk now.
pub(crate) unsafe fn ex_preserve(_args: *mut ExArg) {
    unsafe { ml_preserve(Buf::current(), true, true) };
}

/// `:recover` — read the buffer back out of a swap file.
pub(crate) unsafe fn ex_recover(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    // The flag changes what the swap-file machinery does with what it
    // finds, and is read from several modules.
    recoverymode.set(true);
    let unsaved = unsafe {
        check_changed(
            Buf::current(),
            (if p_awa.get() != 0 {
                CCGD_AW as c_int
            } else {
                0
            }) | CCGD_MULTWIN as c_int
                | (if args.forceit != 0 {
                    CCGD_FORCEIT as c_int
                } else {
                    0
                })
                | CCGD_EXCMD as c_int,
        )
    };
    if !unsaved
        && (byte(args.arg) == NUL
            || unsafe { setfname(Buf::current(), args.arg, ptr::null_mut(), true) }.is_ok())
    {
        unsafe { ml_recover(true) };
    }
    recoverymode.set(false);
}

/// `:find` — edit the first file of that name on 'path', or the `count`'th.
pub(crate) unsafe fn ex_find(args: *mut ExArg) {
    let mut args = unsafe { Ea::new(args) };
    if !check_can_set_curbuf_forceit(args.forceit) {
        return;
    }
    let count = if args.addr_count > 0 {
        args.line2 as c_int
    } else {
        1
    };
    let fname = if byte(get_findfunc()) != NUL {
        unsafe { findfunc_find_file(args.arg, cstr::bytes_at(args.arg).len(), count) }
    } else {
        unsafe { find_nth_on_path(args.arg, args.addr_count, args.line2) }
    };
    if fname.is_null() {
        return;
    }
    args.arg = fname;
    unsafe { do_exedit(args.raw(), None) };
    xfree(fname as *mut c_void);
}

/// The `count`'th match for `pat` on 'path'.
///
/// The search context is what makes the second and later matches cheap:
/// each `find_file_in_path(NULL, …)` resumes the walk the first one
/// started.
unsafe fn find_nth_on_path(pat: *mut c_char, addr_count: c_int, count: LineNr) -> *mut c_char {
    let mut file_to_find: *mut c_char = ptr::null_mut();
    let mut search_ctx: *mut c_char = ptr::null_mut();
    let pat_len = unsafe { cstr::bytes_at(pat) }.len();
    let mut fname = {
        find_file_in_path(
            pat,
            pat_len,
            FileNameOpts::MESS,
            true,
            Buf::current().b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        )
    };
    if addr_count > 0 {
        let mut n = count;
        while !fname.is_null() && {
            n -= 1;
            n > 0
        } {
            xfree(fname as *mut c_void);
            fname = find_file_in_path(
                ptr::null_mut(),
                0 as size_t,
                FileNameOpts::MESS,
                false,
                Buf::current().b_ffname,
                &raw mut file_to_find,
                &raw mut search_ctx,
            );
        }
    }
    xfree(file_to_find as *mut c_void);
    unsafe { vim_findfile_cleanup(search_ctx as *mut c_void) };
    fname
}

/// `:edit`, `:enew`, `:view`, `:badd`, `:balt`.
pub(crate) unsafe fn ex_edit(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    let ffname = if args.cmdidx == CmdIdx::enew {
        ptr::null_mut()
    } else {
        args.arg
    };
    // `:badd` and `:balt` only add to the buffer list; they never leave
    // the current buffer, so they are not asked about it.
    if args.cmdidx != CmdIdx::badd
        && args.cmdidx != CmdIdx::balt
        && unsafe { is_other_file(0, ffname) }
        && !check_can_set_curbuf_forceit(args.forceit)
    {
        return;
    }
    if buf_is_prompt(current_buf()) && args.cmdidx == CmdIdx::edit && byte(args.arg) == NUL {
        emsg(c"cannot :edit a prompt buffer");
        return;
    }
    unsafe { do_exedit(args.raw(), None) };
}

/// The shared body of every command that opens a file into a window.
///
/// `old_curwin` is the window a *split* came from, and is `None` for a plain
/// `:edit`. It is what tells the failure path that there is a new window
/// to close again, and what makes the alternate file be set on the window
/// left behind.
pub(crate) unsafe fn do_exedit(args: *mut ExArg, old_curwin: Option<WinId>) {
    let mut ea = unsafe { Ea::new(args) };
    // `:visual` and `:view` with no argument leave Ex mode.
    if exmode_active.get() && (ea.cmdidx == CmdIdx::visual || ea.cmdidx == CmdIdx::view) {
        exmode_active.set(false);
        ex_pressedreturn.set(false);
        if ui_has(kUICmdline) {
            ui_ext_cmdline_block_leave();
        }
        if byte(ea.arg) == NUL {
            // Inside `:global`, normal mode is entered for the rest of
            // the line and Ex mode resumes afterwards.
            if global_busy.get() != 0 {
                if !ea.nextcmd.is_null() {
                    unsafe { stuff_readbuf(ea.nextcmd) };
                    ea.nextcmd = ptr::null_mut();
                }
                let _redraw = Allow::redraw();
                let _prompt = Allow::wait_return();
                need_wait_return.set(false);
                let save_ms = msg_scroll.get();
                msg_scroll.set(0);
                redraw_all_later(UPD_NOT_VALID);
                pending_exmode_active.set(true);
                normal_enter(false, true);
                pending_exmode_active.set(false);
                msg_scroll.set(save_ms);
            }
            return;
        }
    }

    let idx = ea.cmdidx;
    if (idx == CmdIdx::new
        || idx == CmdIdx::tabnew
        || idx == CmdIdx::tabedit
        || idx == CmdIdx::vnew)
        && byte(ea.arg) == NUL
    {
        // A new, empty buffer.
        setpcmark();
        let _ = do_ecmd(
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            args,
            newlnum::ONE as LineNr,
            EcmdFlags::HIDE | EcmdFlags::FORCEIT.when(ea.forceit != 0),
            old_curwin.is_none().then(|| Win::current().id()),
        );
    } else if idx != CmdIdx::split && idx != CmdIdx::vsplit || byte(ea.arg) != NUL {
        if byte(ea.arg) != NUL && text_or_buf_locked() {
            return;
        }
        let saved_readonly = readonlymode.get();
        if idx == CmdIdx::view || idx == CmdIdx::sview {
            readonlymode.set(true);
        } else if idx == CmdIdx::enew {
            readonlymode.set(false);
        }
        if idx != CmdIdx::balt && idx != CmdIdx::badd {
            setpcmark();
        }

        let opened = do_ecmd(
            0,
            if idx == CmdIdx::enew {
                ptr::null_mut()
            } else {
                ea.arg
            },
            ptr::null_mut(),
            args,
            ea.do_ecmd_lnum,
            EcmdFlags::HIDE.when(buf_hide(Buf::current()))
                | EcmdFlags::FORCEIT.when(ea.forceit != 0)
                | EcmdFlags::OLDBUF.when(old_curwin.is_some())
                | EcmdFlags::ADDBUF.when(idx == CmdIdx::badd)
                | EcmdFlags::ALTBUF.when(idx == CmdIdx::balt),
            old_curwin.is_none().then(|| Win::current().id()),
        );

        if opened.is_err() {
            // The split has already happened; close it again. The
            // cleanup pair keeps an exception from the failed edit from
            // being lost while the window is closed.
            if old_curwin.is_some() {
                let need_hide = curbuf_is_changed() && Buf::current().b_nwindows <= 1;
                if !need_hide || buf_hide(Buf::current()) {
                    let mut cs: Cleanup = unsafe { core::mem::zeroed() };
                    unsafe { enter_cleanup(&raw mut cs) };
                    let free = !need_hide && !buf_hide(Buf::current());
                    win_close(Win::current(), free, false);
                    unsafe { leave_cleanup(&raw mut cs) };
                }
            }
        } else if readonlymode.get() && Buf::current().b_nwindows == 1 {
            Buf::current().b_p_ro = 1;
        }
        readonlymode.set(saved_readonly);
    } else {
        // A `:split` with no file name: the window is already there.
        run_ecmd_cmd(ea);
        let was_invalid = Win::current().w_arg_idx_invalid;
        check_arg_idx(Win::current());
        if was_invalid != Win::current().w_arg_idx_invalid {
            unsafe { maketitle() };
        }
    }

    if let Some(mut old) = old_curwin.and_then(valid_win)
        && byte(ea.arg) != NUL
        && !old.is_current()
        && old.w_buffer != Buf::current_raw()
        && !cmdmod_has(CmdModFlags::KEEPALT)
    {
        old.w_alt_fnum = Buf::current().handle as c_int;
    }
    ex_no_reprint.set(true);
}

/// `:swapname`.
pub(crate) unsafe fn ex_swapname(_args: *mut ExArg) {
    let mfp = Buf::current().b_ml.ml_mfp;
    if mfp.is_null() || mf_fname(mfp).is_null() {
        msg(gettext(c"No swap file".as_ptr()), 0);
    } else {
        msg(mf_fname(mfp), 0);
    }
}

/// `:read` — insert a file, or the output of a command.
pub(crate) unsafe fn ex_read(args: *mut ExArg) {
    let mut args = unsafe { Ea::new(args) };
    let was_empty = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);
    if args.usefilter != 0 {
        do_bang(1, &mut args, false, false, true);
        return;
    }
    if u_save(args.line2, args.line2 + 1).is_err() {
        return;
    }

    let read = if byte(args.arg) == NUL {
        if check_fname().is_err() {
            return;
        }
        readfile(
            Buf::current().b_ffname,
            Buf::current().b_fname,
            args.line2,
            0,
            MAXLNUM as LineNr,
            args.raw(),
            0,
            false,
        )
    } else {
        // 'cpoptions' `a` makes `:read file` set the alternate file.
        if cpo_has(CpoFlag::ALTREAD) {
            unsafe { setaltfname(args.arg, args.arg, 1) };
        }
        readfile(
            args.arg,
            ptr::null_mut(),
            args.line2,
            0,
            MAXLNUM as LineNr,
            args.raw(),
            0,
            false,
        )
    };

    if read.is_err() {
        if !aborting() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(args.arg) };
            semsg!("E484: Can't open file {arg}");
        }
        return;
    }
    // Reading into an empty buffer in Ex mode leaves the empty line the
    // buffer started with; drop it.
    if was_empty && exmode_active.get() {
        let lnum = if args.line2 == 0 {
            Buf::current().b_ml.ml_line_count
        } else {
            1
        };
        if byte(ml_get(lnum)) == NUL && u_savedel(lnum, 1).is_ok() {
            let _ = unsafe { ml_delete(lnum) };
            if Win::current().w_cursor.lnum > 1 && Win::current().w_cursor.lnum >= lnum {
                Win::current().w_cursor.lnum -= 1;
            }
            unsafe { deleted_lines_mark(lnum, 1) };
        }
    }
    redraw_curbuf_later(UPD_VALID);
}

/// `:!cmd`.
pub(crate) unsafe fn ex_bang(args: *mut ExArg) {
    let mut args = unsafe { Ea::new(args) };
    let (addr_count, forceit) = (args.addr_count, args.forceit != 0);
    do_bang(addr_count, &mut args, forceit, true, true);
}

/// `:wundo` — write the undo tree to a file, tagged with a hash of the
/// buffer text so that reading it back into a different buffer is refused.
pub(crate) unsafe fn ex_wundo(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(Buf::current(), &raw mut hash as *mut uint8_t);
    let buffer = Buf::current();
    let hash = hash.as_mut_ptr();
    unsafe { u_write_undo(args.arg, args.forceit != 0, buffer, hash) };
}

/// `:rundo`.
pub(crate) unsafe fn ex_rundo(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(Buf::current(), &raw mut hash as *mut uint8_t);
    unsafe { u_read_undo(args.arg, &raw mut hash as *mut uint8_t, ptr::null()) };
}

/// `:checkpath` — every file 'path' reaches from the includes of this one.
pub(crate) unsafe fn ex_checkpath(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    unsafe {
        find_pattern_in_path(
            ptr::null_mut(),
            kDirectionNotSet,
            0 as size_t,
            false,
            false,
            CHECK_PATH as c_int,
            1,
            if args.forceit != 0 {
                ACTION_SHOW_ALL as c_int
            } else {
                ACTION_SHOW as c_int
            },
            1,
            MAXLNUM as LineNr,
            args.forceit != 0,
            false,
        )
    };
}

/// `:rshada`, `:wshada` and their `viminfo` spellings.
pub(crate) unsafe fn ex_shada(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    // An empty 'shada' would mean "save nothing", which is not what an
    // explicit command means.
    let save_shada = p_shada.get();
    if byte(p_shada.get()) == NUL {
        p_shada.set(c"'100".as_ptr() as *mut c_char);
    }
    if args.cmdidx == CmdIdx::rviminfo || args.cmdidx == CmdIdx::rshada {
        let _ = unsafe { shada_read_everything(args.arg, args.forceit != 0, false) };
    } else {
        unsafe { shada_write_file(args.arg, args.forceit != 0) };
    }
    p_shada.set(save_shada);
}

/// `:fclose` — close a floating window by its handle.
pub(crate) unsafe fn ex_fclose(args: *mut ExArg) {
    let args = unsafe { Ea::new(args) };
    unsafe { win_float_remove(args.forceit != 0, args.line1 as c_int) };
}

/// `buf_hide()` as checked code.
fn buf_hide(buffer: Buf) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::buffer::buf_hide(buffer) }
}

/// `do_bang()` as checked code.
fn do_bang(addr_count: c_int, args: &mut ExArg, forceit: bool, do_in: bool, do_out: bool) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_cmds::do_bang(addr_count, args, forceit, do_in, do_out) }
}

/// `do_ecmd()` as checked code.
#[allow(clippy::too_many_arguments)]
fn do_ecmd(
    fnum: c_int,
    ffname: *mut c_char,
    sfname: *mut c_char,
    args: *mut ExArg,
    newlnum: LineNr,
    flags: EcmdFlags,
    oldwin: Option<WinId>,
) -> Result<(), Failed> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_cmds::do_ecmd(fnum, ffname, sfname, args, newlnum, flags, oldwin) }
}

/// `find_file_in_path()` as checked code.
#[allow(clippy::too_many_arguments)]
fn find_file_in_path(
    name: *mut c_char,
    len: size_t,
    options: FileNameOpts,
    first: bool,
    rel_fname: *mut c_char,
    file_to_find: *mut *mut c_char,
    search_ctx: *mut *mut c_char,
) -> *mut c_char {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe {
        crate::file_search::find_file_in_path(
            name,
            len,
            options,
            first,
            rel_fname,
            file_to_find,
            search_ctx,
        )
    }
}

/// `gettext()` as checked code.
fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    // SAFETY: a NUL-terminated message; `gettext` answers one too.
    unsafe { crate::os::cshim::gettext_ptr(__msgid).as_ptr().cast_mut() }
}

/// `goto_buffer()` as checked code.
fn goto_buffer(args: *mut ExArg, start: c_int, dir: c_int, count: c_int) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::buffer::goto_buffer(args, start, dir, count) }
}

/// `mf_fname()` as checked code.
fn mf_fname(mfp: *const MemFile) -> *const c_char {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::memfile::mf_fname(mfp) }
}

/// `msg()` as checked code.
fn msg(s: *const c_char, hl_id: c_int) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::msg_ptr(s, hl_id) }
}

/// `readfile()` as checked code.
#[allow(clippy::too_many_arguments)]
fn readfile(
    fname: *mut c_char,
    sfname: *mut c_char,
    from: LineNr,
    lines_to_skip: LineNr,
    lines_to_read: LineNr,
    args: *mut ExArg,
    flags: c_int,
    silent: bool,
) -> Result<Loaded, Failed> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe {
        crate::fileio::readfile(
            fname,
            sfname,
            from,
            lines_to_skip,
            lines_to_read,
            args,
            flags,
            silent,
        )
    }
}

/// `u_compute_hash()` as checked code.
fn u_compute_hash(buffer: Buf, hash: *mut uint8_t) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::undo::u_compute_hash(buffer, hash) }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}
