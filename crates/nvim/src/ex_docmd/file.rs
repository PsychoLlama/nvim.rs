//! Commands that name a file or a buffer: reading, editing, finding,
//! recovering, and the buffer list.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::fileio::Loaded;
use crate::guard::Allow;
use crate::memline::MlFlags;
use crate::semsg;
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
use crate::ex_docmd::{
    ACTION_SHOW, ACTION_SHOW_ALL, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, CHECK_PATH,
    DOBUF_CURRENT, DOBUF_FIRST, DOBUF_LAST, DOBUF_MOD, ECMD_ADDBUF, ECMD_ALTBUF, ECMD_FORCEIT,
    ECMD_HIDE, ECMD_OLDBUF, ECMD_ONE, cmdmod_has, ex_pressedreturn, kDirectionNotSet,
};
use crate::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::ex_getln::{text_or_buf_locked, ui_ext_cmdline_block_leave};
use crate::file_search::{FileNameOpts, vim_findfile_cleanup};

use crate::getchar::stuff_readbuf;
use crate::main::{
    curbuf, curwin, e_trailing_arg, ex_no_reprint, exmode_active, global_busy, msg_scroll,
    need_wait_return, p_awa, p_shada, pending_exmode_active, readonlymode, recoverymode,
};
use crate::mark::setpcmark;

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
    CMD_badd, CMD_balt, CMD_edit, CMD_enew, CMD_new, CMD_rshada, CMD_rviminfo, CMD_split,
    CMD_sview, CMD_tabedit, CMD_tabnew, CMD_view, CMD_visual, CMD_vnew, CMD_vsplit, CmdModFlags,
    CpoFlag, Failed, NUL, buf_T, cleanup_T, exarg_T, linenr_T, memfile_T, size_t, uint8_t, win_T,
};
use crate::ui::ui_has;
use crate::undo::{curbuf_is_changed, u_read_undo, u_save, u_savedel, u_write_undo};

use crate::window::{check_can_set_curbuf_forceit, win_close, win_valid};
use crate::winfloat::win_float_remove;
use crate::winlayer::{Buf, Ea, Win};

/// Would editing `fnum`/`ffname` mean leaving the current buffer?
///
/// A buffer whose file could not be stat'ed is compared by its *short*
/// name, because the full name may have been resolved against a directory
/// that no longer exists.
pub(crate) unsafe fn is_other_file(fnum: c_int, ffname: *mut c_char) -> bool {
    if fnum != 0 {
        return fnum != cur_buf().handle;
    }
    if ffname.is_null() {
        return true;
    }
    // An empty name means "this buffer", not "no buffer".
    if byte(ffname) == NUL {
        return false;
    }
    if !cur_buf().file_id_valid && !cur_buf().b_sfname.is_null() && byte(cur_buf().b_sfname) != NUL
    {
        return unsafe { path_fnamecmp(ffname, cur_buf().b_sfname) } != 0;
    }
    unsafe { otherfile(ffname) }
}

/// `:buffer`.
pub(crate) unsafe fn ex_buffer(eap: *mut exarg_T) {
    do_exbuffer(unsafe { Ea::new(eap) });
}

/// `:buffer`, shared with `:pbuffer`.
pub(crate) fn do_exbuffer(mut eap: Ea) {
    // The buffer was already resolved from the argument by
    // `execute_cmd0`'s `ExArgt::BUFNAME` handling, so anything left is junk.
    if unsafe { *eap.arg } != 0 {
        eap.errmsg = Some(unsafe { ex_errmsg(e_trailing_arg.as_ptr(), eap.arg) });
        return;
    }
    if eap.addr_count == 0 {
        goto_buffer(eap.raw(), DOBUF_CURRENT as c_int, FORWARD as c_int, 0);
    } else {
        goto_buffer(
            eap.raw(),
            DOBUF_FIRST as c_int,
            FORWARD as c_int,
            eap.line2 as c_int,
        );
    }
    run_ecmd_cmd(eap);
}

/// Run the `+cmd` argument, once the buffer it applies to is current.
fn run_ecmd_cmd(eap: Ea) {
    if !eap.do_ecmd_cmd.is_null() {
        let _ = unsafe { do_cmdline_cmd(eap.do_ecmd_cmd) };
    }
}

/// `:bmodified`.
pub(crate) unsafe fn ex_bmodified(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    goto_buffer(
        eap.raw(),
        DOBUF_MOD as c_int,
        FORWARD as c_int,
        eap.line2 as c_int,
    );
    run_ecmd_cmd(eap);
}

/// `:bnext`.
pub(crate) unsafe fn ex_bnext(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    goto_buffer(
        eap.raw(),
        DOBUF_CURRENT as c_int,
        FORWARD as c_int,
        eap.line2 as c_int,
    );
    run_ecmd_cmd(eap);
}

/// `:bprevious` and `:bNext`.
pub(crate) unsafe fn ex_bprevious(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    goto_buffer(
        eap.raw(),
        DOBUF_CURRENT as c_int,
        BACKWARD as c_int,
        eap.line2 as c_int,
    );
    run_ecmd_cmd(eap);
}

/// `:brewind` and `:bfirst`.
pub(crate) unsafe fn ex_brewind(eap: *mut exarg_T) {
    goto_buffer(eap, DOBUF_FIRST as c_int, FORWARD as c_int, 0);
    run_ecmd_cmd(unsafe { Ea::new(eap) });
}

/// `:blast`.
pub(crate) unsafe fn ex_blast(eap: *mut exarg_T) {
    goto_buffer(eap, DOBUF_LAST as c_int, BACKWARD as c_int, 0);
    run_ecmd_cmd(unsafe { Ea::new(eap) });
}

/// `:preserve` — flush the swap file to disk now.
pub(crate) unsafe fn ex_preserve(_eap: *mut exarg_T) {
    unsafe { ml_preserve(curbuf.get(), true, true) };
}

/// `:recover` — read the buffer back out of a swap file.
pub(crate) unsafe fn ex_recover(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    // The flag changes what the swap-file machinery does with what it
    // finds, and is read from several modules.
    recoverymode.set(true);
    let unsaved = unsafe {
        check_changed(
            curbuf.get(),
            (if p_awa.get() != 0 {
                CCGD_AW as c_int
            } else {
                0
            }) | CCGD_MULTWIN as c_int
                | (if eap.forceit != 0 {
                    CCGD_FORCEIT as c_int
                } else {
                    0
                })
                | CCGD_EXCMD as c_int,
        )
    };
    if !unsaved
        && (byte(eap.arg) == NUL
            || unsafe { setfname(Buf::current(), eap.arg, ptr::null_mut(), true) }.is_ok())
    {
        unsafe { ml_recover(true) };
    }
    recoverymode.set(false);
}

/// `:find` — edit the first file of that name on 'path', or the `count`'th.
pub(crate) unsafe fn ex_find(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if !check_can_set_curbuf_forceit(eap.forceit) {
        return;
    }
    let count = if eap.addr_count > 0 {
        eap.line2 as c_int
    } else {
        1
    };
    let fname = if byte(get_findfunc()) != NUL {
        unsafe { findfunc_find_file(eap.arg, cstr::bytes_at(eap.arg).len(), count) }
    } else {
        unsafe { find_nth_on_path(eap.arg, eap.addr_count, eap.line2) }
    };
    if fname.is_null() {
        return;
    }
    eap.arg = fname;
    unsafe { do_exedit(eap.raw(), ptr::null_mut()) };
    xfree(fname as *mut c_void);
}

/// The `count`'th match for `pat` on 'path'.
///
/// The search context is what makes the second and later matches cheap:
/// each `find_file_in_path(NULL, …)` resumes the walk the first one
/// started.
unsafe fn find_nth_on_path(pat: *mut c_char, addr_count: c_int, count: linenr_T) -> *mut c_char {
    let mut file_to_find: *mut c_char = ptr::null_mut();
    let mut search_ctx: *mut c_char = ptr::null_mut();
    let pat_len = unsafe { cstr::bytes_at(pat) }.len();
    let mut fname = {
        find_file_in_path(
            pat,
            pat_len,
            FileNameOpts::MESS,
            true,
            cur_buf().b_ffname,
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
                cur_buf().b_ffname,
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
pub(crate) unsafe fn ex_edit(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let ffname = if eap.cmdidx as c_int == CMD_enew as c_int {
        ptr::null_mut()
    } else {
        eap.arg
    };
    // `:badd` and `:balt` only add to the buffer list; they never leave
    // the current buffer, so they are not asked about it.
    if eap.cmdidx as c_int != CMD_badd as c_int
        && eap.cmdidx as c_int != CMD_balt as c_int
        && unsafe { is_other_file(0, ffname) }
        && !check_can_set_curbuf_forceit(eap.forceit)
    {
        return;
    }
    if buf_is_prompt(current_buf())
        && eap.cmdidx as c_int == CMD_edit as c_int
        && byte(eap.arg) == NUL
    {
        emsg(c"cannot :edit a prompt buffer");
        return;
    }
    unsafe { do_exedit(eap.raw(), ptr::null_mut()) };
}

/// The shared body of every command that opens a file into a window.
///
/// `old_curwin` is the window a *split* came from, and is null for a plain
/// `:edit`. It is what tells the failure path that there is a new window
/// to close again, and what makes the alternate file be set on the window
/// left behind.
pub unsafe fn do_exedit(eap: *mut exarg_T, old_curwin: *mut win_T) {
    let mut ea = unsafe { Ea::new(eap) };
    // `:visual` and `:view` with no argument leave Ex mode.
    if exmode_active.get()
        && (ea.cmdidx as c_int == CMD_visual as c_int || ea.cmdidx as c_int == CMD_view as c_int)
    {
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
                unsafe { redraw_all_later(UPD_NOT_VALID) };
                pending_exmode_active.set(true);
                normal_enter(false, true);
                pending_exmode_active.set(false);
                msg_scroll.set(save_ms);
            }
            return;
        }
    }

    let idx = ea.cmdidx as c_int;
    if (idx == CMD_new as c_int
        || idx == CMD_tabnew as c_int
        || idx == CMD_tabedit as c_int
        || idx == CMD_vnew as c_int)
        && byte(ea.arg) == NUL
    {
        // A new, empty buffer.
        setpcmark();
        let _ = do_ecmd(
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            eap,
            ECMD_ONE as linenr_T,
            ECMD_HIDE as c_int
                + if ea.forceit != 0 {
                    ECMD_FORCEIT as c_int
                } else {
                    0
                },
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ptr::null_mut()
            },
        );
    } else if idx != CMD_split as c_int && idx != CMD_vsplit as c_int || byte(ea.arg) != NUL {
        if byte(ea.arg) != NUL && unsafe { text_or_buf_locked() } {
            return;
        }
        let saved_readonly = readonlymode.get();
        if idx == CMD_view as c_int || idx == CMD_sview as c_int {
            readonlymode.set(true);
        } else if idx == CMD_enew as c_int {
            readonlymode.set(false);
        }
        if idx != CMD_balt as c_int && idx != CMD_badd as c_int {
            setpcmark();
        }

        let opened = do_ecmd(
            0,
            if idx == CMD_enew as c_int {
                ptr::null_mut()
            } else {
                ea.arg
            },
            ptr::null_mut(),
            eap,
            ea.do_ecmd_lnum,
            (if buf_hide(curbuf.get()) {
                ECMD_HIDE as c_int
            } else {
                0
            }) + (if ea.forceit != 0 {
                ECMD_FORCEIT as c_int
            } else {
                0
            }) + (if old_curwin.is_null() {
                0
            } else {
                ECMD_OLDBUF as c_int
            }) + (if idx == CMD_badd as c_int {
                ECMD_ADDBUF as c_int
            } else {
                0
            }) + (if idx == CMD_balt as c_int {
                ECMD_ALTBUF as c_int
            } else {
                0
            }),
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ptr::null_mut()
            },
        );

        if opened.is_err() {
            // The split has already happened; close it again. The
            // cleanup pair keeps an exception from the failed edit from
            // being lost while the window is closed.
            if !old_curwin.is_null() {
                let need_hide = curbuf_is_changed() && cur_buf().b_nwindows <= 1;
                if !need_hide || buf_hide(curbuf.get()) {
                    let mut cs: cleanup_T = unsafe { core::mem::zeroed() };
                    unsafe { enter_cleanup(&raw mut cs) };
                    unsafe {
                        win_close(curwin.get(), !need_hide && !buf_hide(curbuf.get()), false)
                    };
                    unsafe { leave_cleanup(&raw mut cs) };
                }
            }
        } else if readonlymode.get() && cur_buf().b_nwindows == 1 {
            cur_buf().b_p_ro = 1;
        }
        readonlymode.set(saved_readonly);
    } else {
        // A `:split` with no file name: the window is already there.
        run_ecmd_cmd(ea);
        let was_invalid = cur_win().w_arg_idx_invalid;
        check_arg_idx(cur_win());
        if was_invalid != cur_win().w_arg_idx_invalid {
            unsafe { maketitle() };
        }
    }

    if !old_curwin.is_null()
        && byte(ea.arg) != NUL
        && curwin.get() != old_curwin
        && win_valid(old_curwin)
        && unsafe { (*old_curwin).w_buffer } != curbuf.get()
        && !cmdmod_has(CmdModFlags::KEEPALT)
    {
        unsafe { (*old_curwin).w_alt_fnum = cur_buf().handle as c_int };
    }
    ex_no_reprint.set(true);
}

/// `:swapname`.
pub(crate) unsafe fn ex_swapname(_eap: *mut exarg_T) {
    let mfp = cur_buf().b_ml.ml_mfp;
    if mfp.is_null() || mf_fname(mfp).is_null() {
        msg(gettext(c"No swap file".as_ptr()), 0);
    } else {
        msg(mf_fname(mfp), 0);
    }
}

/// `:read` — insert a file, or the output of a command.
pub(crate) unsafe fn ex_read(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let was_empty = cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY);
    if eap.usefilter != 0 {
        do_bang(1, eap.raw(), false, false, true);
        return;
    }
    if u_save(eap.line2, eap.line2 + 1).is_err() {
        return;
    }

    let read = if byte(eap.arg) == NUL {
        if unsafe { check_fname() }.is_err() {
            return;
        }
        readfile(
            cur_buf().b_ffname,
            cur_buf().b_fname,
            eap.line2,
            0,
            MAXLNUM as linenr_T,
            eap.raw(),
            0,
            false,
        )
    } else {
        // 'cpoptions' `a` makes `:read file` set the alternate file.
        if cpo_has(CpoFlag::ALTREAD) {
            unsafe { setaltfname(eap.arg, eap.arg, 1) };
        }
        readfile(
            eap.arg,
            ptr::null_mut(),
            eap.line2,
            0,
            MAXLNUM as linenr_T,
            eap.raw(),
            0,
            false,
        )
    };

    if read.is_err() {
        if !aborting() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(eap.arg) };
            semsg!("E484: Can't open file {arg}");
        }
        return;
    }
    // Reading into an empty buffer in Ex mode leaves the empty line the
    // buffer started with; drop it.
    if was_empty && exmode_active.get() {
        let lnum = if eap.line2 == 0 {
            cur_buf().b_ml.ml_line_count
        } else {
            1
        };
        if byte(ml_get(lnum)) == NUL && u_savedel(lnum, 1).is_ok() {
            let _ = unsafe { ml_delete(lnum) };
            if cur_win().w_cursor.lnum > 1 && cur_win().w_cursor.lnum >= lnum {
                cur_win().w_cursor.lnum -= 1;
            }
            unsafe { deleted_lines_mark(lnum, 1) };
        }
    }
    redraw_curbuf_later(UPD_VALID);
}

/// `:!cmd`.
pub(crate) unsafe fn ex_bang(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    do_bang(eap.addr_count, eap.raw(), eap.forceit != 0, true, true);
}

/// `:wundo` — write the undo tree to a file, tagged with a hash of the
/// buffer text so that reading it back into a different buffer is refused.
pub(crate) unsafe fn ex_wundo(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let mut hash: [uint8_t; 32] = [0; 32];
    unsafe { u_compute_hash(Buf::current(), &raw mut hash as *mut uint8_t) };
    unsafe {
        u_write_undo(
            eap.arg,
            eap.forceit != 0,
            Buf::current(),
            &raw mut hash as *mut uint8_t,
        )
    };
}

/// `:rundo`.
pub(crate) unsafe fn ex_rundo(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let mut hash: [uint8_t; 32] = [0; 32];
    unsafe { u_compute_hash(Buf::current(), &raw mut hash as *mut uint8_t) };
    unsafe { u_read_undo(eap.arg, &raw mut hash as *mut uint8_t, ptr::null()) };
}

/// `:checkpath` — every file 'path' reaches from the includes of this one.
pub(crate) unsafe fn ex_checkpath(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    unsafe {
        find_pattern_in_path(
            ptr::null_mut(),
            kDirectionNotSet,
            0 as size_t,
            false,
            false,
            CHECK_PATH as c_int,
            1,
            if eap.forceit != 0 {
                ACTION_SHOW_ALL as c_int
            } else {
                ACTION_SHOW as c_int
            },
            1,
            MAXLNUM as linenr_T,
            eap.forceit != 0,
            false,
        )
    };
}

/// `:rshada`, `:wshada` and their `viminfo` spellings.
pub(crate) unsafe fn ex_shada(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    // An empty 'shada' would mean "save nothing", which is not what an
    // explicit command means.
    let save_shada = p_shada.get();
    if byte(p_shada.get()) == NUL {
        p_shada.set(c"'100".as_ptr() as *mut c_char);
    }
    if eap.cmdidx as c_int == CMD_rviminfo as c_int || eap.cmdidx as c_int == CMD_rshada as c_int {
        let _ = unsafe { shada_read_everything(eap.arg, eap.forceit != 0, false) };
    } else {
        unsafe { shada_write_file(eap.arg, eap.forceit != 0) };
    }
    p_shada.set(save_shada);
}

/// `:fclose` — close a floating window by its handle.
pub(crate) unsafe fn ex_fclose(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    unsafe { win_float_remove(eap.forceit != 0, eap.line1 as c_int) };
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

/// `buf_hide()` as checked code.
fn buf_hide(buf: *const buf_T) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::buffer::buf_hide(buf) }
}

/// `do_bang()` as checked code.
fn do_bang(addr_count: c_int, eap: *mut exarg_T, forceit: bool, do_in: bool, do_out: bool) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_cmds::do_bang(addr_count, eap, forceit, do_in, do_out) }
}

/// `do_ecmd()` as checked code.
#[allow(clippy::too_many_arguments)]
fn do_ecmd(
    fnum: c_int,
    ffname: *mut c_char,
    sfname: *mut c_char,
    eap: *mut exarg_T,
    newlnum: linenr_T,
    flags: c_int,
    oldwin: *mut win_T,
) -> Result<(), Failed> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_cmds::do_ecmd(fnum, ffname, sfname, eap, newlnum, flags, oldwin) }
}

/// `find_file_in_path()` as checked code.
#[allow(clippy::too_many_arguments)]
fn find_file_in_path(
    ptr: *mut c_char,
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
            ptr,
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
fn goto_buffer(eap: *mut exarg_T, start: c_int, dir: c_int, count: c_int) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::buffer::goto_buffer(eap, start, dir, count) }
}

/// `mf_fname()` as checked code.
fn mf_fname(mfp: *const memfile_T) -> *const c_char {
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
    from: linenr_T,
    lines_to_skip: linenr_T,
    lines_to_read: linenr_T,
    eap: *mut exarg_T,
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
            eap,
            flags,
            silent,
        )
    }
}

/// `u_compute_hash()` as checked code.
fn u_compute_hash(buf: Buf, hash: *mut uint8_t) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::undo::u_compute_hash(buf, hash) }
}

/// `xfree()` as checked code.
fn xfree(ptr: *mut c_void) {
    // SAFETY: `xmalloc`ed, or null.
    unsafe { crate::memory::xfree(ptr) }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}
