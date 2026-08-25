//! [`do_ecmd`] -- the single function behind `:edit`, `:enew`, `:view`,
//! `:new`, `:read`, `:sview` and every other command that changes which file a
//! window shows.
//!
//! It has to decide whether the target is already in a buffer, whether the
//! current buffer can be abandoned (and if so whether to write, hide or wipe
//! it), fire BufLeave/BufUnload/BufEnter/BufWinEnter in the right order with
//! the right buffer current, survive an autocommand that deleted a buffer or
//! closed the window underneath it (`delbuf_msg`), and finally position the
//! cursor from the `+cmd` argument, a mark or the last-known position.
//! [`set_swapcommand`] is how the `+cmd` reaches the swap-file dialog.
//!
//! The stages below are upstream's, in upstream's order, and the order is the
//! contract: every one of them can run Vimscript, so hoisting a read past an
//! `apply_autocmds` or past a `curbuf`/`curwin` assignment changes behaviour
//! even where it looks pure.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

// The buffer-swap machinery, carved out so that neither half is over the
// line cap; see its own docs.
mod switch;

use self::switch::{Switch, delbuf_msg, switch_to_other_buffer};
use super::{
    BFA_KEEP_UNDO, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, ECMD_ADDBUF, ECMD_ALTBUF,
    ECMD_FORCEIT, ECMD_HIDE, ECMD_LAST, ECMD_LASTL, ECMD_NOWINENTER, ECMD_OLDBUF, ECMD_SET_HELP,
    FAIL, KEYMAP_INIT, READ_KEEP_UNDO, READ_NOWINENTER, SEA_DIALOG, SEA_QUIT,
};
use crate::arglist::check_arg_idx;
use crate::autocmd::{EVENT_BUFENTER, EVENT_BUFWINENTER, apply_autocmds_retval};
use crate::buffer::{
    BufFlags, buf_clear_file, buf_freeall, bufref_valid, do_autochdir, do_modelines, fileinfo,
    handle_swap_exists, maketitle, open_buffer, otherfile, set_buflisted, set_bufref, setaltfname,
};
use crate::charset::skipwhite;
use crate::cursor::{check_cursor, check_cursor_col, check_cursor_lnum, get_cursor_line_ptr};
use crate::diff::{diff_buf_add, diff_invalidate};
use crate::digraph::keymap_init;
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::ex_cmds2::{check_changed, check_fname};
use crate::ex_docmd::{DoCmdOpts, do_cmdline};
use crate::ex_eval::{aborting, should_abort};
use crate::fold::fold_update_all;
use crate::guard::Suppress;
use crate::help::prepare_help_buffer;
use crate::main::{
    curbuf, curwin, exiting, exmode_active, keep_help_flag, msg_listdo_overwrite, msg_scroll,
    msg_scrolled_ign, p_awa, p_sol, p_ur, p_verbose, skip_redraw, swap_exists_action,
};
use crate::mark::set_last_cursor;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::{msg_check_for_delay, msg_start};
use crate::r#move::{changed_line_abv_curs, update_topline};
use crate::normal::reset_VIsual;
use crate::option::{ScrollMargin, ScrollOff, shortmess};
use crate::path::fix_fname;
use crate::plines::plines_m_win_fill;
use crate::pos::equalpos;
use crate::spell::parse_spelllang;
use crate::strings::vim_snprintf_safelen;
use crate::terminal::terminal_check_size;
use crate::types::{
    NUL, OK, OptInt, OptionSetFlags, ShmFlag, String_0, Vv, bufref_T, exarg_T, linenr_T, ptrdiff_t,
    time_t, win_T,
};
use crate::undo::{u_savecommon, u_sync, u_unchanged};
use crate::window::{check_lnums, curwin_init, win_valid};
use crate::winlayer::Win;
use crate::winlayer::tab_windows;
use ::libc::time;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Set `v:swapcommand` for the SwapExists autocommands: `[+cmd]` to be
/// executed (e.g. `+10`), or a `G` jump to `newlnum` when there is no command.
///
/// Returns true if `v:swapcommand` was actually set.
///
/// # Safety
/// `command` must be live, or NULL.
pub unsafe fn set_swapcommand(command: *mut c_char, newlnum: linenr_T) -> bool {
    // SAFETY: caller's contract; `v:swapcommand` is a live string variable.
    if unsafe {
        command.is_null() && newlnum <= 0 || *get_vim_var_str(Vv::Swapcommand) as c_int != NUL
    } {
        return false;
    }
    // SAFETY: as above; `val.data` is `valsize` bytes and each format takes
    // exactly the one argument that follows it.
    unsafe {
        let valsize = if command.is_null() {
            30
        } else {
            strlen_of(command) + 3
        };
        let mut val = String_0::from_raw_parts(xmalloc(valsize) as *mut c_char, 0);
        val.set_len(if command.is_null() {
            vim_snprintf_safelen(val.data(), valsize, c"%ldG".as_ptr(), newlnum as i64)
        } else {
            vim_snprintf_safelen(val.data(), valsize, c":%s\r".as_ptr(), command)
        });
        set_vim_var_string(Vv::Swapcommand, val.data(), val.len() as ptrdiff_t);
        xfree(val.data().cast());
    }
    true
}

/// `strlen`, spelled so that the one call site reads.
///
/// # Safety
/// `s` must be a live NUL-terminated string.
unsafe fn strlen_of(s: *const c_char) -> usize {
    // SAFETY: caller's contract.
    unsafe { core::ffi::CStr::from_ptr(s) }.to_bytes().len()
}

/// The line and column `do_ecmd`'s stages hand each other, plus the flags that
/// say how far the switch has got.
struct Ecmd {
    /// Where to put the cursor: `> 0` a line number, `ECMD_LASTL` the last
    /// position in the loaded file, `ECMD_LAST` the last position in any file.
    newlnum: linenr_T,
    /// Column an autocommand moved the cursor to, or `-1`.
    newcol: c_int,
    /// Last known column for `newlnum`, or `-1`; used when 'sol' is off.
    solcol: c_int,
    /// The window's `w_topline` before the file was read, or 0 when the
    /// autocommands left it alone.
    topline: linenr_T,
    /// The buffer already had a memfile: nothing to read.
    oldbuf: bool,
    /// Autocommands brought us into the buffer unexpectedly, so most of the
    /// window setup has already happened.
    auto_buf: bool,
    /// `get_winopts` ran, so the window's options may need a spell reload.
    did_get_winopts: bool,
    /// Extra `readfile` flags the undo bookkeeping asked for.
    readfile_flags: c_int,
}

/// The immutable half of what [`do_ecmd`] was asked for, once the file name
/// has been resolved.
struct EcmdArgs {
    /// The buffer number to switch to, or zero to go by name.
    fnum: c_int,
    /// The full path of the file to edit.
    ffname: *mut c_char,
    /// Its short name.
    sfname: *mut c_char,
    /// The Ex command that asked, or NULL.
    eap: *mut exarg_T,
    /// The `ECMD_*` flags.
    flags: c_int,
    /// The `+cmd` to run once the file is loaded, or NULL.
    command: *mut c_char,
}

/// What resolving the target file decided.
enum Target {
    /// The file is already being edited in this very buffer: nothing to do.
    AlreadyHere,
    /// Nothing to edit (an empty `:badd`, say).
    Nothing,
    /// Go ahead; true when this is a different file from the current one.
    Editing(bool),
}

/// Start editing a new file.
///
/// `fnum` is the file number, or zero to use `ffname`/`sfname`.  `ffname` is
/// the full path if `sfname` is given, any file name if it is NULL, an empty
/// string to re-edit the same file name (possibly in another directory), or
/// NULL to start an empty buffer.  `eap` carries the command to run after
/// loading and the forced 'ff'/'fenc', and can be NULL.  `newlnum` is the line
/// to put the cursor on, or one of `ECMD_LASTL`/`ECMD_LAST`/`ECMD_ONE`.
/// `oldwin` should be `curwin` when editing in the current window, and NULL
/// when the window was split first; when it is not NULL, the previous buffer's
/// position is remembered for it.
///
/// The `flags` are `ECMD_HIDE` (don't free the current buffer), `ECMD_SET_HELP`
/// (set `b_help` on the new buffer first), `ECMD_OLDBUF` (use the existing
/// buffer), `ECMD_FORCEIT` (`!` was given), `ECMD_ADDBUF` (don't edit, just add
/// to the buffer list), `ECMD_ALTBUF` (as `ECMD_ADDBUF`, and set the alternate
/// file) and `ECMD_NOWINENTER` (do not trigger BufWinEnter).
///
/// Returns `FAIL` for failure, `OK` otherwise.
///
/// # Safety
/// The names, `eap` and `oldwin` must be live, or NULL where that is allowed
/// above.
pub unsafe fn do_ecmd(
    fnum: c_int,
    ffname: *mut c_char,
    sfname: *mut c_char,
    eap: *mut exarg_T,
    newlnum: linenr_T,
    flags: c_int,
    oldwin: *mut win_T,
) -> c_int {
    let mut ffname = ffname;
    let mut sfname = sfname;
    let mut oldwin = oldwin;
    let mut free_fname = ptr::null_mut::<c_char>();
    let mut retval = FAIL;
    // Held from "the buffer is settled" to "the cursor is in the right
    // line", which is past the block below on one path and inside it on the
    // other.
    let mut redraw_off = None;
    let mut state = Ecmd {
        newlnum,
        newcol: -1,
        solcol: -1,
        topline: 0,
        oldbuf: false,
        auto_buf: false,
        did_get_winopts: false,
        readfile_flags: 0,
    };
    // SAFETY: `curwin` is the live current window, and the handle is used
    // only inside this call.
    let so = ScrollOff::of(unsafe { Win::current() }, ScrollMargin::Lines);
    // SAFETY: `eap` is live when non-NULL.
    let command = if eap.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*eap).do_ecmd_cmd }
    };

    let mut old_curbuf = bufref_T::default();
    // SAFETY: `curbuf` is the live current buffer.
    unsafe { set_bufref(&raw mut old_curbuf, curbuf.get()) };
    let mut did_set_swapcommand = false;

    'theend: {
        // SAFETY: the names are the caller's.
        let other_file =
            match unsafe { resolve_target(fnum, &mut ffname, &mut sfname, flags, &mut free_fname) }
            {
                Target::AlreadyHere => return OK,
                Target::Nothing => break 'theend,
                Target::Editing(other) => other,
            };
        let args = EcmdArgs {
            fnum,
            ffname,
            sfname,
            eap,
            flags,
            command,
        };

        // Re-editing a terminal buffer: skip most buffer re-initialization.
        // SAFETY: `curbuf`/`curwin` are live.
        if !other_file && !unsafe { (*curbuf.get()).terminal }.is_null() {
            // Needed when called from do_argfile(); the title may show the
            // arg index, e.g. "(2 of 5)".
            unsafe {
                check_arg_idx(curwin.get());
                maketitle();
            }
            retval = OK;
            break 'theend;
        }

        // If the file was changed we may not be allowed to abandon it:
        // - if we are going to re-edit the same file
        // - or if we are the only window on this file and ECMD_HIDE is false
        // SAFETY: `curbuf` is live.
        let must_ask = (!other_file && flags & ECMD_OLDBUF as c_int == 0)
            || (unsafe { (*curbuf.get()).b_nwindows } == 1
                && flags & (ECMD_HIDE as c_int | ECMD_ADDBUF as c_int | ECMD_ALTBUF as c_int) == 0);
        // SAFETY: as above.
        if must_ask
            && unsafe {
                check_changed(
                    curbuf.get(),
                    (if p_awa.get() != 0 {
                        CCGD_AW as c_int
                    } else {
                        0
                    }) | (if other_file { 0 } else { CCGD_MULTWIN as c_int })
                        | (if flags & ECMD_FORCEIT as c_int != 0 {
                            CCGD_FORCEIT as c_int
                        } else {
                            0
                        })
                        | (if eap.is_null() {
                            0
                        } else {
                            CCGD_EXCMD as c_int
                        }),
                )
            }
        {
            if fnum == 0 && other_file && !ffname.is_null() {
                // SAFETY: the names are live.
                unsafe {
                    setaltfname(
                        ffname,
                        sfname,
                        if state.newlnum < 0 { 0 } else { state.newlnum },
                    );
                }
            }
            break 'theend;
        }

        // End Visual mode before switching to another buffer, so the text can
        // be copied into the GUI selection buffer.  Careful: may trigger a
        // ModeChanged autocommand.  Should we block autocommands here?
        reset_VIsual();

        // autocommands freed window :(
        // SAFETY: `oldwin` is the caller's, and `win_valid` tolerates a stale
        // pointer -- that is what it is for.
        if !oldwin.is_null() && !unsafe { win_valid(oldwin) } {
            oldwin = ptr::null_mut();
        }

        // SAFETY: `command` is live when non-NULL.
        did_set_swapcommand = unsafe { set_swapcommand(command, state.newlnum) };

        // If we are starting to edit another file, open a (new) buffer.
        // Otherwise we re-use the current buffer.
        if other_file {
            // SAFETY: everything in the stage is the caller's or the editor's.
            match unsafe { switch_to_other_buffer(&args, &mut oldwin, &mut old_curbuf, &mut state) }
            {
                Switch::Abandon => break 'theend,
                Switch::Ready => {}
            }
            // SAFETY: `curwin` is live.
            unsafe {
                (*curwin.get()).w_pcmark.lnum = 1;
                (*curwin.get()).w_pcmark.col = 0;
            }
        } else if flags & (ECMD_ADDBUF as c_int | ECMD_ALTBUF as c_int) != 0
            // SAFETY: main thread, message state.
            || unsafe { check_fname() } == FAIL
        {
            break 'theend;
        } else {
            state.oldbuf = flags & ECMD_OLDBUF as c_int != 0;
        }

        // Don't redraw until the cursor is in the right line, otherwise
        // autocommands may cause ml_get errors.
        redraw_off = Some(Suppress::redraw());

        let buf = curbuf.get();
        // SAFETY: `curbuf` is live.
        if flags & ECMD_SET_HELP as c_int != 0 || keep_help_flag.get() {
            unsafe { prepare_help_buffer() };
        } else if !unsafe { (*curbuf.get()).b_help } {
            // Don't make a buffer listed if it's a help buffer.  Useful when
            // using CTRL-O to go back to a help file.
            unsafe { set_buflisted(1) };
        }

        // If autocommands change buffers under our fingers, forget about
        // editing the file.  Autocmds may also abort script processing.
        if buf != curbuf.get() || aborting() {
            break 'theend;
        }

        // Since we are starting to edit a file, consider the filetype to be
        // unset.  Helps for when an autocommand changes files and expects
        // syntax highlighting to work in the other file.
        // SAFETY: `curbuf` is live.
        unsafe { (*curbuf.get()).b_did_filetype = false };

        // other_file oldbuf
        //  false     false       re-edit same file, buffer is re-used
        //  false     true        re-edit same file, nothing changes
        //  true      false       start editing new file, new buffer
        //  true      true        start editing in existing buffer (nothing)
        if !other_file && !state.oldbuf {
            // SAFETY: `curbuf`/`curwin` are live.
            if !unsafe { reuse_current_buffer(&mut state) } {
                break 'theend;
            }
        }

        // If we get here we are sure to start editing.  Assume success now.
        retval = OK;

        // If the file name was changed, reset the not-edit flag so that
        // ":write" works.
        if !other_file {
            // SAFETY: `curbuf` is live.
            unsafe { (*curbuf.get()).b_flags.clear(BufFlags::NOTEDITED) };
        }

        // Check if we are editing the w_arg_idx file in the argument list.
        // SAFETY: `curwin` is live.
        unsafe { check_arg_idx(curwin.get()) };

        if !state.auto_buf {
            // SAFETY: the editor's own state; `eap` is the caller's.
            unsafe { enter_new_buffer(&args, &mut old_curbuf, &mut state, &mut retval) };
        }

        // Tell the diff stuff that this buffer is new and/or needs updating.
        // Also needed when re-editing the same buffer, because unloading will
        // have removed it as a diff buffer.
        // SAFETY: `curwin`/`curbuf` are live.
        unsafe {
            if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
                diff_buf_add(curbuf.get());
                diff_invalidate(curbuf.get());
            }
            // If the window options were changed we may need to set the spell
            // language.  Can only be done once the buffer is properly set up.
            if state.did_get_winopts
                && (*curwin.get()).w_onebuf_opt.wo_spell != 0
                && *(*(*curwin.get()).w_s).b_p_spl as c_int != NUL
            {
                parse_spelllang(curwin.get());
            }
        }

        if command.is_null() {
            // SAFETY: `curwin`/`curbuf` are live.
            unsafe { place_cursor(&state) };
        }

        // Check if cursors in other windows on the same buffer are still valid
        check_lnums(false);

        // Did not read the file, need to show some info about the file.
        // Do this after setting the cursor.
        if state.oldbuf && !state.auto_buf {
            // SAFETY: message state.
            unsafe { report_file_info() };
        }

        // SAFETY: `curbuf` is live and `command` the caller's.
        unsafe {
            (*curbuf.get()).b_last_used = time(ptr::null_mut::<time_t>());
            if !command.is_null() {
                do_cmdline(command, None, ptr::null_mut(), DoCmdOpts::VERBOSE);
            }
            if (*curbuf.get()).b_kmap_state as c_int & KEYMAP_INIT != 0 {
                keymap_init();
            }
        }

        drop(redraw_off.take());
        if !skip_redraw.get() {
            // SAFETY: `so_ptr` points into the current window or at the global
            // 'scrolloff', both live for this call.
            unsafe { recenter(so, state.topline, command) };
        }

        // Change directories when the 'acd' option is set.
        do_autochdir();
    }

    // SAFETY: the two buffers are live when their bufrefs say so.
    unsafe {
        if bufref_valid(&raw mut old_curbuf) && !(*old_curbuf.br_buf).terminal.is_null() {
            terminal_check_size((*old_curbuf.br_buf).terminal);
        }
        if (!bufref_valid(&raw mut old_curbuf) || curbuf.get() != old_curbuf.br_buf)
            && !(*curbuf.get()).terminal.is_null()
        {
            terminal_check_size((*curbuf.get()).terminal);
        }
    }

    drop(redraw_off.take());
    if did_set_swapcommand {
        // SAFETY: main thread; a NULL clears the variable.
        unsafe { set_vim_var_string(Vv::Swapcommand, ptr::null(), -1) };
    }
    // SAFETY: our own allocation, or NULL.
    unsafe { xfree(free_fname.cast()) };
    retval
}

/// Work out whether `do_ecmd` is being asked for another file, expanding the
/// name it was given on the way.
///
/// # Safety
/// The names must be live, or NULL; `free_fname` receives an allocation the
/// caller must free.
unsafe fn resolve_target(
    fnum: c_int,
    ffname: &mut *mut c_char,
    sfname: &mut *mut c_char,
    flags: c_int,
    free_fname: &mut *mut c_char,
) -> Target {
    if fnum != 0 {
        // SAFETY: `curbuf` is live.
        if fnum == unsafe { (*curbuf.get()).handle } {
            // file is already being edited, nothing to do
            return Target::AlreadyHere;
        }
        return Target::Editing(true);
    }

    // if no short name given, use ffname for short name
    if sfname.is_null() {
        *sfname = *ffname;
    }

    // SAFETY: `ffname` is live when non-NULL.
    if flags & (ECMD_ADDBUF as c_int | ECMD_ALTBUF as c_int) != 0
        && (ffname.is_null() || unsafe { **ffname } as c_int == NUL)
    {
        return Target::Nothing;
    }
    if ffname.is_null() {
        return Target::Editing(true);
    }
    // SAFETY: as above; `curbuf` is live.
    unsafe {
        if **ffname as c_int == NUL && (*curbuf.get()).b_ffname.is_null() {
            // there is no file name
            return Target::Editing(false);
        }
        if **ffname as c_int == NUL {
            // re-edit with same file name
            *ffname = (*curbuf.get()).b_ffname;
            *sfname = (*curbuf.get()).b_fname;
        }
        // may expand to full path name
        *free_fname = fix_fname(*ffname);
        if !free_fname.is_null() {
            *ffname = *free_fname;
        }
        Target::Editing(otherfile(*ffname))
    }
}

/// Re-edit the current file in the buffer it already has: store the contents
/// so the reload can be undone, then empty it out.
///
/// Answers false when an autocommand pulled the buffer out from under us.
///
/// # Safety
/// `curbuf` and `curwin` must be the live current buffer and window.
unsafe fn reuse_current_buffer(state: &mut Ecmd) -> bool {
    // SAFETY: caller's contract.
    unsafe {
        // may set b_last_cursor
        set_last_cursor(curwin.get());
        if state.newlnum == ECMD_LAST as linenr_T || state.newlnum == ECMD_LASTL as linenr_T {
            state.newlnum = (*curwin.get()).w_cursor.lnum;
            state.solcol = (*curwin.get()).w_cursor.col;
        }
        let buf = curbuf.get();
        let new_name = if (*buf).b_fname.is_null() {
            ptr::null_mut()
        } else {
            xstrdup((*buf).b_fname)
        };
        let mut bufref = bufref_T::default();
        set_bufref(&raw mut bufref, buf);

        // If the buffer was used before, store the current contents so that
        // the reload can be undone.  Do not do this if the (empty) buffer is
        // being re-used for another file.
        if !(*curbuf.get()).b_flags.has(BufFlags::NEVERLOADED)
            && (p_ur.get() < 0 || (*curbuf.get()).b_ml.ml_line_count as OptInt <= p_ur.get())
        {
            // Sync first so that this is a separate undo-able action.
            u_sync(false);
            if u_savecommon(
                curbuf.get(),
                0,
                (*curbuf.get()).b_ml.ml_line_count + 1,
                0,
                true,
            ) == FAIL
            {
                xfree(new_name.cast());
                return false;
            }
            u_unchanged(curbuf.get());
            buf_freeall(curbuf.get(), BFA_KEEP_UNDO as c_int);
            // Tell readfile() not to clear or reload undo info.
            state.readfile_flags = READ_KEEP_UNDO as c_int;
        } else {
            // Free all things for buffer.
            buf_freeall(curbuf.get(), 0);
        }

        // If autocommands deleted the buffer we were going to re-edit, give up
        // and jump to the end.  `delbuf_msg` frees `new_name`.
        if !bufref_valid(&raw mut bufref) {
            delbuf_msg(new_name);
            return false;
        }
        xfree(new_name.cast());

        // If autocommands change buffers under our fingers, forget about
        // re-editing the file.  Should do the buf_clear_file(), but perhaps
        // the autocommands changed the buffer...  They may also abort script
        // processing.
        if buf != curbuf.get() || aborting() {
            return false;
        }
        buf_clear_file(curbuf.get());
        // clear '[ and '] marks
        (*curbuf.get()).b_op_start.lnum = 0;
        (*curbuf.get()).b_op_end.lnum = 0;
    }
    true
}

/// Set the cursor and initialise the window, then read the file (or fire
/// BufEnter/BufWinEnter when there is nothing to read).
///
/// # Safety
/// The editor's window and buffer state must be live, and `old_curbuf` the
/// bufref [`do_ecmd`] took on entry.
unsafe fn enter_new_buffer(
    args: &EcmdArgs,
    old_curbuf: &mut bufref_T,
    state: &mut Ecmd,
    retval: &mut c_int,
) {
    let (eap, flags) = (args.eap, args.flags);
    // Set cursor and init window before reading the file and executing
    // autocommands.  This allows for the autocommands to position the cursor.
    curwin_init();

    // It's possible that all lines in the buffer changed.  Need to update
    // automatic folding for all windows where it's used.
    for win in tab_windows() {
        if win.w_buffer == curbuf.get() {
            // SAFETY: `win` is a live window.
            unsafe { fold_update_all(win.raw()) };
        }
    }

    // Change directories when the 'acd' option is set.
    do_autochdir();

    // Careful: open_buffer() and apply_autocmds() may change the current
    // buffer and window.
    // SAFETY: `curwin` is live.
    let orig_pos = unsafe { (*curwin.get()).w_cursor };
    state.topline = unsafe { (*curwin.get()).w_topline };
    if !state.oldbuf {
        // need to read the file
        swap_exists_action.set(SEA_DIALOG);
        // set/reset 'ro' flag
        // SAFETY: `curbuf` is live and `eap` the caller's.
        unsafe {
            (*curbuf.get()).b_flags |= BufFlags::CHECK_RO;
            // Open the buffer and read the file.
            if flags & ECMD_NOWINENTER as c_int != 0 {
                state.readfile_flags |= READ_NOWINENTER as c_int;
            }
            if should_abort(open_buffer(false, eap, state.readfile_flags)) {
                *retval = FAIL;
            }
            if swap_exists_action.get() == SEA_QUIT {
                *retval = FAIL;
            }
            handle_swap_exists(&raw mut *old_curbuf);
        }
    } else {
        // Read the modelines, but only to set window-local options.  Any
        // buffer-local options have already been set and may have been changed
        // by the user.
        // SAFETY: `curbuf` is live.
        unsafe {
            do_modelines(OptionSetFlags::WINONLY);
            apply_autocmds_retval(
                EVENT_BUFENTER,
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
                retval,
            );
            if flags & ECMD_NOWINENTER as c_int == 0 {
                apply_autocmds_retval(
                    EVENT_BUFWINENTER,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    false,
                    curbuf.get(),
                    retval,
                );
            }
        }
    }
    // SAFETY: `curwin` is live.
    unsafe { check_arg_idx(curwin.get()) };

    // If autocommands change the cursor position or topline, we should keep
    // it.  Also when it moves within a line.  But not when it moves to the
    // first non-blank.
    // SAFETY: `curwin` is live and the cursor is on a line of its buffer.
    unsafe {
        if !equalpos((*curwin.get()).w_cursor, orig_pos) {
            let text = get_cursor_line_ptr();
            if (*curwin.get()).w_cursor.lnum != orig_pos.lnum
                || (*curwin.get()).w_cursor.col != skipwhite(text).offset_from(text) as c_int
            {
                state.newlnum = (*curwin.get()).w_cursor.lnum;
                state.newcol = (*curwin.get()).w_cursor.col;
            }
        }
        if (*curwin.get()).w_topline == state.topline {
            state.topline = 0;
        }

        // Even when the cursor didn't move we need to recompute topline.
        changed_line_abv_curs();
        maketitle();
    }
}

/// Put the cursor where the caller, the autocommands or the buffer's last
/// known position asked for.
///
/// # Safety
/// `curwin` and `curbuf` must be live.
unsafe fn place_cursor(state: &Ecmd) {
    // SAFETY: caller's contract.
    unsafe {
        if state.newcol >= 0 {
            // position set by autocommands
            (*curwin.get()).w_cursor.lnum = state.newlnum;
            (*curwin.get()).w_cursor.col = state.newcol;
            check_cursor(curwin.get());
        } else if state.newlnum > 0 {
            // line number from caller or old position
            (*curwin.get()).w_cursor.lnum = state.newlnum;
            check_cursor_lnum(curwin.get());
            if state.solcol >= 0 && p_sol.get() == 0 {
                // 'sol' is off: use the last known column.
                (*curwin.get()).w_cursor.col = state.solcol;
                check_cursor_col(curwin.get());
                (*curwin.get()).w_cursor.coladd = 0;
                (*curwin.get()).w_set_curswant = true;
            } else {
                beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
            }
        } else {
            // no line number, go to last line in Ex mode
            if exmode_active.get() {
                (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            }
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        }
    }
}

/// The "file, N lines" line `:edit` prints when it did not read the file.
///
/// # Safety
/// Message state, main thread.
unsafe fn report_file_info() {
    let msg_scroll_save = msg_scroll.get();
    // Obey the 'O' flag in 'cpoptions': overwrite any previous file message.
    if shortmess(ShmFlag::OVERALL)
        && msg_listdo_overwrite.get() == 0
        && !exiting.get()
        && p_verbose.get() == 0
    {
        msg_scroll.set(0);
    }
    if msg_scroll.get() == 0 {
        // wait a bit when overwriting an error msg
        // SAFETY: caller's contract.
        unsafe { msg_check_for_delay(false) };
    }
    // SAFETY: as above.
    unsafe { msg_start() };
    msg_scroll.set(msg_scroll_save);
    msg_scrolled_ign.set(true);

    if !shortmess(ShmFlag::FILEINFO) {
        // SAFETY: as above.
        unsafe { fileinfo(0, 1, false) };
    }

    msg_scrolled_ign.set(false);
}

/// Recompute the scroll position for the new cursor line, centring it when the
/// autocommands left the window's top line alone and there is no `+cmd`.
///
/// # Safety
/// `curwin` must be live, and `so` its scroll margin.
unsafe fn recenter(so: ScrollOff, topline: linenr_T, command: *mut c_char) {
    let n = so.get();
    if topline == 0 && command.is_null() {
        // force the cursor to be vertically centered in the window
        so.set(999);
    }
    // SAFETY: caller's contract; `curwin` is live.
    unsafe {
        update_topline(curwin.get());
        (*curwin.get()).w_scbind_pos =
            plines_m_win_fill(curwin.get(), 1, (*curwin.get()).w_topline);
    }
    so.set(n);
    // redraw this buffer later
    // SAFETY: no argument beyond the redraw type.
    unsafe { redraw_curbuf_later(UPD_NOT_VALID) };
}
