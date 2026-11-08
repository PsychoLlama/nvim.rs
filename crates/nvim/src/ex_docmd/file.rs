//! Commands that name a file or a buffer: reading, editing, finding,
//! recovering, and the buffer list.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::buffer::DOBUF_CURRENT;
use crate::buffer::DOBUF_FIRST;
use crate::buffer::DOBUF_MOD;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds::newlnum;
use crate::ex_cmds2::CCGD_AW;
use crate::ex_cmds2::CCGD_EXCMD;
use crate::ex_cmds2::CCGD_FORCEIT;
use crate::ex_cmds2::CCGD_MULTWIN;
use crate::file_search::find_nth_file_in_path;
use crate::fileio::{ReadFlags, readfile_named};
use crate::guard::Allow;
use crate::memfile::buf_swap_name;
use crate::memline::MlFlags;
use crate::memline::ml_get_len;
use crate::memory::ThinCString;
use crate::memory::XString;
use crate::message::msg;
use crate::message_fmt::msg_bytes;
use crate::os::cshim::gettext;
use crate::search::ACTION_SHOW;
use crate::search::ACTION_SHOW_ALL;
use crate::search::CHECK_PATH;
use crate::semsg;
use crate::types::CmdIdx;
use crate::types::ColNr;
use crate::undo::UNDO_HASH_SIZE;
use crate::undo::u_compute_hash;
use crate::window::valid_win;
use crate::winlayer::WinId;
use core::ffi::{CStr, c_int};

use crate::arglist::check_arg_idx;
use crate::buffer::{buf_is_prompt, current_buf, maketitle, otherfile, setaltfname, setfname};

use crate::change::deleted_lines_mark;
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_curbuf_later};

use crate::ex_cmds2::{check_changed, check_fname};
use crate::ex_docmd::cmdline::do_cmdline_cmd;
use crate::ex_docmd::path::findfunc_find_file;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::{DOBUF_LAST, cmdmod_has, ex_pressedreturn, kDirectionNotSet};
use crate::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::ex_getln::{text_or_buf_locked, ui_ext_cmdline_block_leave};
use crate::file_search::FileNameOpts;

use crate::ex_docmd::state::{ex_no_reprint, global_busy};
use crate::getchar::stuff_readbuf;
use crate::mark::setpcmark;
use crate::message::e_trailing_arg;
use crate::message::state::{msg_scroll, need_wait_return};
use crate::option::vars::{P_SHADA, p_awa, p_shada};
use crate::startup::{readonlymode, recoverymode};
use crate::state::mode::{exmode_active, pending_exmode_active};

use crate::memline::{ml_delete, ml_preserve, ml_recover};

use crate::message::emsg;

use crate::normal::normal_enter;
use crate::option::{cpo_has, get_findfunc};

use crate::ex_eval::defs::Cleanup;
use crate::option::defs::CpoFlag;
use crate::path::path_fnamecmp;
use crate::pos::MAXLNUM;
use crate::search::{BACKWARD, FORWARD, find_pattern_in_path};
use crate::shada::{shada_read_everything, shada_write_file};
use crate::types::{CmdModFlags, ExArg, LineNr};
use crate::ui::state::kUICmdline;
use crate::ui::ui_has;
use crate::undo::{curbuf_is_changed, u_read_undo, u_save, u_savedel, u_write_undo};

use crate::window::{check_can_set_curbuf_forceit, win_close};
use crate::winfloat::win_float_remove;
use crate::winlayer::{Buf, Win};

/// Would editing `fnum`/`ffname` mean leaving the current buffer?
///
/// A buffer whose file could not be stat'ed is compared by its *short*
/// name, because the full name may have been resolved against a directory
/// that no longer exists.
pub(crate) fn is_other_file(fnum: c_int, ffname: Option<&CStr>) -> bool {
    if fnum != 0 {
        return fnum != Buf::current().handle.get();
    }
    let Some(ffname) = ffname else {
        return true;
    };
    // An empty name means "this buffer", not "no buffer".
    if ffname.is_empty() {
        return false;
    }
    if !Buf::current().file_id_valid
        && let Some(short) = Buf::current().name.short()
        && !short.is_empty()
    {
        return path_fnamecmp(ffname.to_bytes(), short.to_bytes()) != 0;
    }
    otherfile(ffname)
}

/// `:buffer`.
pub(crate) fn ex_buffer(excmd: &mut ExArg) {
    do_exbuffer(excmd);
}

/// `:buffer`, shared with `:pbuffer`.
pub(crate) fn do_exbuffer(excmd: &mut ExArg) {
    // The buffer was already resolved from the argument by
    // `execute_cmd0`'s `ExArgt::BUFNAME` handling, so anything left is junk.
    if excmd.line.byte_at(excmd.line.arg) != 0 {
        excmd.errmsg = Some(ex_errmsg(
            e_trailing_arg,
            excmd.line.cstr_from(excmd.line.arg),
        ));
        return;
    }
    if excmd.addr_count == 0 {
        goto_buffer(excmd, DOBUF_CURRENT, FORWARD as c_int, 0);
    } else {
        goto_buffer(excmd, DOBUF_FIRST, FORWARD as c_int, excmd.line2.get());
    }
    run_ecmd_cmd(excmd);
}

/// Run the `+cmd` argument, once the buffer it applies to is current.
fn run_ecmd_cmd(excmd: &mut ExArg) {
    if let Some(cmd) = excmd.do_ecmd_cmd.cstr(&excmd.line) {
        let _ = do_cmdline_cmd(cmd);
    }
}

/// `:bmodified`.
pub(crate) fn ex_bmodified(excmd: &mut ExArg) {
    goto_buffer(excmd, DOBUF_MOD, FORWARD as c_int, excmd.line2.get());
    run_ecmd_cmd(excmd);
}

/// `:bnext`.
pub(crate) fn ex_bnext(excmd: &mut ExArg) {
    goto_buffer(excmd, DOBUF_CURRENT, FORWARD as c_int, excmd.line2.get());
    run_ecmd_cmd(excmd);
}

/// `:bprevious` and `:bNext`.
pub(crate) fn ex_bprevious(excmd: &mut ExArg) {
    goto_buffer(excmd, DOBUF_CURRENT, BACKWARD as c_int, excmd.line2.get());
    run_ecmd_cmd(excmd);
}

/// `:brewind` and `:bfirst`.
pub(crate) fn ex_brewind(excmd: &mut ExArg) {
    goto_buffer(excmd, DOBUF_FIRST, FORWARD as c_int, 0);
    run_ecmd_cmd(excmd);
}

/// `:blast`.
pub(crate) fn ex_blast(excmd: &mut ExArg) {
    goto_buffer(excmd, DOBUF_LAST, BACKWARD as c_int, 0);
    run_ecmd_cmd(excmd);
}

/// `:preserve` — flush the swap file to disk now.
pub(crate) fn ex_preserve(_excmd: &mut ExArg) {
    ml_preserve(Buf::current(), true, true);
}

/// `:recover` — read the buffer back out of a swap file.
pub(crate) fn ex_recover(excmd: &mut ExArg) {
    // The flag changes what the swap-file machinery does with what it
    // finds, and is read from several modules.
    recoverymode.set(true);
    let unsaved = check_changed(
        Buf::current(),
        (if p_awa() { CCGD_AW } else { 0 })
            | CCGD_MULTWIN
            | (if excmd.forceit { CCGD_FORCEIT } else { 0 })
            | CCGD_EXCMD,
    );
    if !unsaved
        && (excmd.line.byte_at(excmd.line.arg) == 0
            || setfname(
                Buf::current(),
                Some(excmd.line.cstr_from(excmd.line.arg)),
                None,
                true,
            )
            .is_ok())
    {
        ml_recover(true);
    }
    recoverymode.set(false);
}

/// `:find` — edit the first file of that name on 'path', or the `count`'th.
pub(crate) fn ex_find(excmd: &mut ExArg) {
    if check_can_set_curbuf_forceit(excmd.forceit).is_err() {
        return;
    }
    let count = if excmd.addr_count > 0 {
        excmd.line2.get()
    } else {
        1
    };
    let fname = if !get_findfunc().is_empty() {
        findfunc_find_file(excmd.line.arg(), count)
    } else {
        let nth = if excmd.addr_count > 0 {
            excmd.line2
        } else {
            LineNr::ONE
        };
        find_nth_file_in_path(excmd.line.arg(), FileNameOpts::MESS, nth.get())
            .map(ThinCString::from)
    };
    let Some(fname) = fname else {
        return;
    };
    excmd.set_arg_text(fname.as_bytes());
    excmd.exedit(None);
}

/// `:edit`, `:enew`, `:view`, `:badd`, `:balt`.
pub(crate) fn ex_edit(excmd: &mut ExArg) {
    let ffname = (excmd.cmdidx != CmdIdx::enew).then(|| excmd.line.cstr_from(excmd.line.arg));
    // `:badd` and `:balt` only add to the buffer list; they never leave
    // the current buffer, so they are not asked about it.
    if excmd.cmdidx != CmdIdx::badd
        && excmd.cmdidx != CmdIdx::balt
        && is_other_file(0, ffname)
        && check_can_set_curbuf_forceit(excmd.forceit).is_err()
    {
        return;
    }
    if buf_is_prompt(current_buf())
        && excmd.cmdidx == CmdIdx::edit
        && excmd.line.byte_at(excmd.line.arg) == 0
    {
        emsg(c"cannot :edit a prompt buffer");
        return;
    }
    excmd.exedit(None);
}

impl ExArg<'_> {
    /// The shared body of every command that opens a file into a window.
    ///
    /// `old_curwin` is the window a *split* came from, and is `None` for a plain
    /// `:edit`. It is what tells the failure path that there is a new window
    /// to close again, and what makes the alternate file be set on the window
    /// left behind.
    pub(crate) fn exedit(&mut self, old_curwin: Option<WinId>) {
        // `:visual` and `:view` with no argument leave Ex mode.
        if exmode_active.get() && (self.cmdidx == CmdIdx::visual || self.cmdidx == CmdIdx::view) {
            exmode_active.set(false);
            ex_pressedreturn.set(false);
            if ui_has(kUICmdline) {
                ui_ext_cmdline_block_leave();
            }
            if self.line.byte_at(self.line.arg) == 0 {
                // Inside `:global`, normal mode is entered for the rest of
                // the line and Ex mode resumes afterwards.
                if global_busy.get() != 0 {
                    if let Some(next) = self.line.next {
                        stuff_readbuf(self.line.cstr_from(next).to_bytes());
                        self.line.next = None;
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

        let idx = self.cmdidx;
        if (idx == CmdIdx::new
            || idx == CmdIdx::tabnew
            || idx == CmdIdx::tabedit
            || idx == CmdIdx::vnew)
            && self.line.byte_at(self.line.arg) == 0
        {
            // A new, empty buffer.
            setpcmark();
            let flags = EcmdFlags::HIDE | EcmdFlags::FORCEIT.when(self.forceit);
            let _ = do_ecmd(
                None,
                None,
                None,
                Some(&mut *self),
                newlnum::ONE as LineNr,
                flags,
                old_curwin.is_none().then(|| Win::current().id()),
            );
        } else if idx != CmdIdx::split && idx != CmdIdx::vsplit
            || self.line.byte_at(self.line.arg) != 0
        {
            if self.line.byte_at(self.line.arg) != 0 && text_or_buf_locked() {
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

            // A copy: `do_ecmd` is handed the command as well as its argument,
            // and the two cannot be borrowed out of the same `ExArg` at once.
            // Upstream lends it a pointer into a line `do_ecmd`'s own
            // autocommands may reallocate, which is the same hazard written
            // down.
            let name = (idx != CmdIdx::enew).then(|| self.line.cstr_from(self.line.arg).to_owned());
            let flags = EcmdFlags::HIDE.when(buf_hide(Buf::current()))
                | EcmdFlags::FORCEIT.when(self.forceit)
                | EcmdFlags::OLDBUF.when(old_curwin.is_some())
                | EcmdFlags::ADDBUF.when(idx == CmdIdx::badd)
                | EcmdFlags::ALTBUF.when(idx == CmdIdx::balt);
            let lnum = self.do_ecmd_lnum;
            let opened = do_ecmd(
                None,
                name.as_deref(),
                None,
                Some(&mut *self),
                lnum,
                flags,
                old_curwin.is_none().then(|| Win::current().id()),
            );

            if opened.is_err() {
                // The split has already happened; close it again. The
                // cleanup pair keeps an exception from the failed edit from
                // being lost while the window is closed.
                if old_curwin.is_some() {
                    let need_hide = curbuf_is_changed() && Buf::current().b_nwindows <= 1;
                    if !need_hide || buf_hide(Buf::current()) {
                        let mut cs = Cleanup {
                            pending: 0,
                            exception: None,
                        };
                        enter_cleanup(&mut cs);
                        let free = !need_hide && !buf_hide(Buf::current());
                        let _ = win_close(Win::current(), free, false);
                        leave_cleanup(&mut cs);
                    }
                }
            } else if readonlymode.get() && Buf::current().b_nwindows == 1 {
                Buf::current().b_p_ro = 1;
            }
            readonlymode.set(saved_readonly);
        } else {
            // A `:split` with no file name: the window is already there.
            run_ecmd_cmd(self);
            let was_invalid = Win::current().w_arg_idx_invalid;
            check_arg_idx(Win::current());
            if was_invalid != Win::current().w_arg_idx_invalid {
                maketitle();
            }
        }

        if let Some(mut old) = old_curwin.and_then(valid_win)
            && self.line.byte_at(self.line.arg) != 0
            && !old.is_current()
            && !old.w_buffer.is_current()
            && !cmdmod_has(CmdModFlags::KEEPALT)
        {
            old.w_alt_fnum = Some(Buf::current().handle);
        }
        ex_no_reprint.set(true);
    }
}

/// `:swapname`.
pub(crate) fn ex_swapname(_excmd: &mut ExArg) {
    match buf_swap_name(Buf::current()) {
        None => msg(gettext(c"No swap file"), 0),
        Some(name) => msg(&name, 0),
    };
}

/// `:read` — insert a file, or the output of a command.
pub(crate) fn ex_read(excmd: &mut ExArg) {
    let was_empty = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);
    if excmd.usefilter {
        do_bang(1, excmd, false, false, true);
        return;
    }
    if u_save(excmd.line2, excmd.line2 + 1).is_err() {
        return;
    }

    let read = if excmd.line.byte_at(excmd.line.arg) == 0 {
        if check_fname().is_err() {
            return;
        }
        // Copies: `readfile` takes the command as well, and the buffer's
        // own name is not borrowable across it.
        let full = Buf::current().name.full().map(CStr::to_owned);
        let shown = Buf::current().name.shown().map(CStr::to_owned);
        readfile_named(
            full.as_deref(),
            shown.as_deref(),
            excmd.line2,
            0,
            MAXLNUM.get(),
            Some(excmd),
            ReadFlags::NONE,
            false,
        )
    } else {
        // 'cpoptions' `a` makes `:read file` set the alternate file.
        if cpo_has(CpoFlag::ALTREAD) {
            let name = excmd.line.cstr_from(excmd.line.arg);
            setaltfname(Some(name), Some(name), LineNr::ONE);
        }
        // A copy, as `ex_edit`'s: `readfile` takes the command too.
        let name = excmd.line.cstr_from(excmd.line.arg).to_owned();
        readfile_named(
            Some(&name),
            None,
            excmd.line2,
            0,
            MAXLNUM.get(),
            Some(excmd),
            ReadFlags::NONE,
            false,
        )
    };

    if read.is_err() {
        if !aborting() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E484: Can't open file {arg}");
        }
        return;
    }
    // Reading into an empty buffer in Ex mode leaves the empty line the
    // buffer started with; drop it.
    if was_empty && exmode_active.get() {
        let lnum = if excmd.line2 == LineNr::ZERO {
            Buf::current().b_ml.ml_line_count
        } else {
            LineNr::ONE
        };
        if ml_get_len(lnum) == ColNr::ZERO && u_savedel(lnum, 1).is_ok() {
            let _ = ml_delete(lnum);
            if Win::current().w_cursor.lnum > LineNr::ONE && Win::current().w_cursor.lnum >= lnum {
                Win::current().w_cursor.lnum -= 1;
            }
            deleted_lines_mark(lnum, 1);
        }
    }
    redraw_curbuf_later(UPD_VALID);
}

/// `:!cmd`.
pub(crate) fn ex_bang(excmd: &mut ExArg) {
    let (addr_count, forceit) = (excmd.addr_count, excmd.forceit);
    do_bang(addr_count, excmd, forceit, true, true);
}

/// `:wundo` — write the undo tree to a file, tagged with a hash of the
/// buffer text so that reading it back into a different buffer is refused.
pub(crate) fn ex_wundo(excmd: &mut ExArg) {
    let mut hash = [0; UNDO_HASH_SIZE as usize];
    u_compute_hash(Buf::current(), &mut hash);
    let name = excmd.line.cstr_from(excmd.line.arg);
    u_write_undo(Some(name), excmd.forceit, Buf::current(), &hash);
}

/// `:rundo`.
pub(crate) fn ex_rundo(excmd: &mut ExArg) {
    let mut hash = [0; UNDO_HASH_SIZE as usize];
    u_compute_hash(Buf::current(), &mut hash);
    u_read_undo(Some(excmd.line.cstr_from(excmd.line.arg)), &hash, None);
}

/// `:checkpath` — every file 'path' reaches from the includes of this one.
pub(crate) fn ex_checkpath(excmd: &mut ExArg) {
    find_pattern_in_path(
        b"",
        kDirectionNotSet,
        false,
        false,
        CHECK_PATH,
        1,
        if excmd.forceit {
            ACTION_SHOW_ALL
        } else {
            ACTION_SHOW
        },
        LineNr::ONE,
        MAXLNUM,
        excmd.forceit,
        false,
    );
}

/// `:rshada`, `:wshada` and their `viminfo` spellings.
pub(crate) fn ex_shada(excmd: &mut ExArg) {
    // An empty 'shada' would mean "save nothing", which is not what an
    // explicit command means.
    let save_shada =
        p_shada(CStr::is_empty).then(|| P_SHADA.swap(Some(XString::from_cstr(c"'100"))));
    if excmd.cmdidx == CmdIdx::rviminfo || excmd.cmdidx == CmdIdx::rshada {
        shada_read_everything(
            Some(excmd.line.cstr_from(excmd.line.arg)),
            excmd.forceit,
            false,
        );
    } else {
        shada_write_file(Some(excmd.line.cstr_from(excmd.line.arg)), excmd.forceit);
    }
    if let Some(saved) = save_shada {
        P_SHADA.restore(saved);
    }
}

/// `:fclose` — close a floating window by its handle.
pub(crate) fn ex_fclose(excmd: &mut ExArg) {
    win_float_remove(excmd.forceit, excmd.line1.get());
}

/// `buf_hide()` as checked code.
fn buf_hide(buffer: Buf) -> bool {
    crate::buffer::buf_hide(buffer)
}

/// `do_bang()` as checked code.
fn do_bang(addr_count: c_int, args: &mut ExArg, forceit: bool, do_in: bool, do_out: bool) {
    crate::ex_cmds::do_bang(addr_count, args, forceit, do_in, do_out)
}

/// `goto_buffer()` as checked code.
fn goto_buffer(excmd: &mut ExArg, start: c_int, dir: c_int, count: c_int) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    crate::buffer::goto_buffer(excmd, start, dir, count)
}
