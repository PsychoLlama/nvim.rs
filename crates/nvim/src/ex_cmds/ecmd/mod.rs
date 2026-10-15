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

#![forbid(unsafe_code)]

// The buffer-swap machinery, carved out so that neither half is over the
// line cap; see its own docs.
mod switch;

use self::switch::{Switch, delbuf_msg, switch_to_other_buffer};
use super::{
    BFA_KEEP_UNDO, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, KEYMAP_INIT, READ_KEEP_UNDO,
    READ_NOWINENTER, SEA_DIALOG, SEA_QUIT,
};
use crate::arglist::check_arg_idx;
use crate::ex_cmds::say;
use crate::fileio::ReadFlags;
use crate::types::AutoEvent;
use crate::winlayer::WinId;
use core::ffi::CStr;
use std::ffi::CString;

use crate::buffer::state::swap_exists_action;
use crate::buffer::{
    BufFlags, BufRef, buf_clear_file, buf_freeall, do_autochdir, do_modelines, fileinfo,
    handle_swap_exists, maketitle, open_buffer, otherfile, set_buflisted, setaltfname,
};
use crate::buffer::{current_buf, fire_retval};
use crate::charset::skip;
use crate::cursor::{check_cursor, check_cursor_col, check_cursor_lnum};
use crate::diff::{diff_buf_add, diff_invalidate};
use crate::digraph::keymap_init;
use crate::drawscreen::state::skip_redraw;
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::eval::vars::{set_vim_var_string, with_vim_var_str};
use crate::ex_cmds2::{check_changed, check_fname};
use crate::ex_docmd::{DoCmdOpts, do_cmdline};
use crate::ex_eval::{aborting, should_abort_err};
use crate::fold::fold_update_all;
use crate::guard::Suppress;
use crate::help::prepare_help_buffer;
use crate::mark::set_last_cursor;
use crate::memline::Lines;
use crate::message::msg_check_for_delay;
use crate::message::state::{msg_listdo_overwrite, msg_scroll, msg_scrolled_ign};
use crate::r#move::{changed_line_abv_curs, update_topline};
use crate::normal::reset_visual;
use crate::option::vars::{p_awa, p_sol, p_ur, p_verbose};
use crate::option::{ScrollMargin, ScrollOff, shortmess};
use crate::optionstr::OptString;
use crate::os::time::os_time_raw;
use crate::path::fix_fname;
use crate::plines::plines_m_win_fill;
use crate::pos::equalpos;
use crate::spell::parse_spelllang;
use crate::startup::exiting;
use crate::state::mode::exmode_active;
use crate::tag::state::keep_help_flag;
use crate::terminal::buf_terminal_check_size;
use crate::types::{ExArg, Failed, LineNr, NUL, OptInt, OptionSetFlags, ShmFlag, Vv};
use crate::undo::{u_savecommon, u_sync, u_unchanged};
use crate::window::{check_lnums, curwin_init, win_valid};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use crate::winlayer::tab_windows;
use core::ffi::c_int;

/// Set `v:swapcommand` for the SwapExists autocommands: `[+cmd]` to be
/// executed (e.g. `+10`), or a `G` jump to `newlnum` when there is no command.
///
/// Returns true if `v:swapcommand` was actually set.
pub fn set_swapcommand(command: Option<&CStr>, newlnum: LineNr) -> bool {
    if command.is_none() && newlnum <= 0 || !with_vim_var_str(Vv::Swapcommand, CStr::is_empty) {
        return false;
    }
    let val = match command {
        None => format!("{newlnum}G").into_bytes(),
        Some(command) => [&b":"[..], command.to_bytes(), b"\r"].concat(),
    };
    set_vim_var_string(Vv::Swapcommand, Some(&val));
    true
}

crate::flag_set! {
    /// How [`do_ecmd`] should go about switching buffers -- upstream's
    /// `ECMD_*` flag half.
    pub struct EcmdFlags;

    /// Do not free the buffer being left, even if nothing else holds it.
    const HIDE = 1;
    /// Set `b_help` on the new buffer before reading it.
    const SET_HELP = 2;
    /// The buffer already exists; do not read the file again.
    const OLDBUF = 4;
    /// `!` was given.
    const FORCEIT = 8;
    /// Do not edit: only add the file to the buffer list.
    const ADDBUF = 16;
    /// As [`Self::ADDBUF`], and make it the alternate file.
    const ALTBUF = 32;
    /// Do not trigger `BufWinEnter`.
    const NOWINENTER = 64;
}

/// Where [`do_ecmd`] should leave the cursor, where that is not a line number.
///
/// Upstream spells these `ECMD_*` too, beside the flags above, but they are
/// `LineNr` values for a different parameter and never share a word with
/// one.
pub mod newlnum {
    use crate::types::LineNr;

    /// The first line.
    pub const ONE: LineNr = 1;
    /// The last position in *any* file -- the one `'"` names.
    pub const LAST: LineNr = -1;
    /// The last position in *this* file, if it has been visited before.
    pub const LASTL: LineNr = 0;
}

/// The line and column `do_ecmd`'s stages hand each other, plus the flags that
/// say how far the switch has got.
struct Ecmd {
    /// Where to put the cursor: `> 0` a line number, or one of
    /// [`newlnum`]'s three sentinels.
    newlnum: LineNr,
    /// Column an autocommand moved the cursor to, or `-1`.
    newcol: c_int,
    /// Last known column for `newlnum`, or `-1`; used when 'sol' is off.
    solcol: c_int,
    /// The window's `w_topline` before the file was read, or 0 when the
    /// autocommands left it alone.
    topline: LineNr,
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
struct EcmdArgs<'a, 'e> {
    /// The buffer number to switch to, or zero to go by name.
    fnum: c_int,
    /// The full path of the file to edit.
    ffname: Option<&'a CStr>,
    /// Its short name.
    sfname: Option<&'a CStr>,
    /// The Ex command that asked, when one did.
    excmd: Option<&'a mut ExArg<'e>>,
    flags: EcmdFlags,
    /// The `+cmd` to run once the file is loaded.
    command: Option<&'a CStr>,
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

/// [`do_ecmd`] for buffer `fnum` — or a new, empty one for 0 — with no
/// file name and no command: what the quickfix code edits with.
pub(crate) fn edit_buffer_number(
    fnum: c_int,
    newlnum: LineNr,
    flags: EcmdFlags,
    oldwin: Option<WinId>,
) -> Result<(), Failed> {
    do_ecmd(fnum, None, None, None, newlnum, flags, oldwin)
}

/// Start editing a new file.
///
/// `fnum` is the file number, or zero to use `ffname`/`sfname`.  `ffname` is
/// the full path if `sfname` is given, any file name if it is NULL, an empty
/// string to re-edit the same file name (possibly in another directory), or
/// NULL to start an empty buffer.  `eap` carries the command to run after
/// loading and the forced 'ff'/'fenc', and can be NULL.  `newlnum` is the line
/// to put the cursor on, or one of [`newlnum`]'s sentinels.
/// `oldwin` should be the current window when editing in it, and `None` when
/// the window was split first; when it is `Some`, the previous buffer's
/// position is remembered for it.
///
/// Answers `Err` for failure.
///
/// `oldwin` is a handle: an autocommand may close that window, and
/// `win_valid` below is what asks -- the one case `winlayer`'s docs reserve
/// for an identity rather than a [`Win`]. The names are copied on entry:
/// the autocommands this runs may free whatever the caller's borrow of them
/// pointed into.
pub(crate) fn do_ecmd(
    fnum: c_int,
    ffname: Option<&CStr>,
    sfname: Option<&CStr>,
    excmd: Option<&mut ExArg>,
    newlnum: LineNr,
    flags: EcmdFlags,
    oldwin: Option<WinId>,
) -> Result<(), Failed> {
    let mut ffname: Option<CString> = ffname.map(CStr::to_owned);
    let mut sfname: Option<CString> = sfname.map(CStr::to_owned);
    let mut oldwin = oldwin;
    let mut retval = Err(Failed);
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
    let so = ScrollOff::of(Win::current(), ScrollMargin::Lines);
    // A copy: the command line is the caller's, and `excmd` is lent on to
    // the read below while the command is still wanted.
    let command_text = excmd
        .as_deref()
        .and_then(|asked| asked.do_ecmd_cmd.cstr(&asked.line))
        .map(CStr::to_owned);
    let command = command_text.as_deref();
    let has_command = excmd.is_some();

    let mut old_curbuf = BufRef::of_opt(current_buf());
    let mut did_set_swapcommand = false;

    'theend: {
        let other_file = match resolve_target(fnum, &mut ffname, &mut sfname, flags) {
            Target::AlreadyHere => return Ok(()),
            Target::Nothing => break 'theend,
            Target::Editing(other) => other,
        };
        let mut ecmd = EcmdArgs {
            fnum,
            ffname: ffname.as_deref(),
            sfname: sfname.as_deref(),
            excmd,
            flags,
            command,
        };

        // Re-editing a terminal buffer: skip most buffer re-initialization.
        if !other_file && !Buf::current().terminal.is_null() {
            // Needed when called from do_argfile(); the title may show the
            // arg index, e.g. "(2 of 5)".
            check_arg_idx(Win::current());
            maketitle();
            retval = Ok(());
            break 'theend;
        }

        // If the file was changed we may not be allowed to abandon it:
        // - if we are going to re-edit the same file
        // - or if we are the only window on this file and EcmdFlags::HIDE is false
        let must_ask = (!other_file && !flags.has(EcmdFlags::OLDBUF))
            || (Buf::current().b_nwindows == 1
                && !flags.has(EcmdFlags::HIDE | EcmdFlags::ADDBUF | EcmdFlags::ALTBUF));
        // What "may we abandon this buffer" should take into account.  Plain
        // arithmetic, so it belongs outside the call's region.
        let mut ccgd = 0;
        if p_awa() {
            ccgd |= CCGD_AW as c_int;
        }
        if !other_file {
            ccgd |= CCGD_MULTWIN as c_int;
        }
        if flags.has(EcmdFlags::FORCEIT) {
            ccgd |= CCGD_FORCEIT as c_int;
        }
        if has_command {
            ccgd |= CCGD_EXCMD as c_int;
        }
        if must_ask && check_changed(Buf::current(), ccgd) {
            if fnum == 0 && other_file && ecmd.ffname.is_some() {
                let lnum = state.newlnum.max(0);
                setaltfname(ecmd.ffname, ecmd.sfname, lnum);
            }
            break 'theend;
        }

        // End Visual mode before switching to another buffer, so the text can
        // be copied into the GUI selection buffer.  Careful: may trigger a
        // ModeChanged autocommand.  Should we block autocommands here?
        reset_visual();

        // autocommands freed window :(
        // A stale id answers `None`, which is what this check is for.
        oldwin = oldwin.filter(|&w| win_valid(w));

        did_set_swapcommand = set_swapcommand(command, state.newlnum);

        // If we are starting to edit another file, open a (new) buffer.
        // Otherwise we re-use the current buffer.
        if other_file {
            match switch_to_other_buffer(&mut ecmd, &mut oldwin, &mut old_curbuf, &mut state) {
                Switch::Abandon => break 'theend,
                Switch::Ready => {}
            }
            Win::current().w_pcmark.lnum = 1;
            Win::current().w_pcmark.col = 0;
        } else if flags.has(EcmdFlags::ADDBUF | EcmdFlags::ALTBUF) || check_fname().is_err() {
            break 'theend;
        } else {
            state.oldbuf = flags.has(EcmdFlags::OLDBUF);
        }

        // Don't redraw until the cursor is in the right line, otherwise
        // autocommands may cause ml_get errors.
        redraw_off = Some(Suppress::redraw());

        let buf = Buf::current_raw();
        if flags.has(EcmdFlags::SET_HELP) || keep_help_flag.get() {
            prepare_help_buffer();
        } else if !Buf::current().b_help {
            // Don't make a buffer listed if it's a help buffer.  Useful when
            // using CTRL-O to go back to a help file.
            set_buflisted(1);
        }

        // If autocommands change buffers under our fingers, forget about
        // editing the file.  Autocmds may also abort script processing.
        if buf != Buf::current_raw() || aborting() {
            break 'theend;
        }

        // Since we are starting to edit a file, consider the filetype to be
        // unset.  Helps for when an autocommand changes files and expects
        // syntax highlighting to work in the other file.
        Buf::current().b_did_filetype = false;

        // other_file oldbuf
        //  false     false       re-edit same file, buffer is re-used
        //  false     true        re-edit same file, nothing changes
        //  true      false       start editing new file, new buffer
        //  true      true        start editing in existing buffer (nothing)
        if !other_file && !state.oldbuf && !reuse_current_buffer(&mut state) {
            break 'theend;
        }

        // If we get here we are sure to start editing.  Assume success now.
        retval = Ok(());

        // If the file name was changed, reset the not-edit flag so that
        // ":write" works.
        if !other_file {
            Buf::current().b_flags.clear(BufFlags::NOTEDITED);
        }

        // Check if we are editing the w_arg_idx file in the argument list.
        check_arg_idx(Win::current());

        if !state.auto_buf {
            enter_new_buffer(&mut ecmd, &mut old_curbuf, &mut state, &mut retval);
        }

        // Tell the diff stuff that this buffer is new and/or needs updating.
        // Also needed when re-editing the same buffer, because unloading will
        // have removed it as a diff buffer.
        if Win::current().w_onebuf_opt.wo_diff != 0 {
            diff_buf_add(Buf::current());
            diff_invalidate(Buf::current());
        }
        // If the window options were changed we may need to set the spell
        // language.  Can only be done once the buffer is properly set up.
        if state.did_get_winopts
            && Win::current().w_onebuf_opt.wo_spell != 0
            && Win::current().syntax().b_p_spl.first_byte() as c_int != NUL
        {
            let _ = parse_spelllang(Win::current());
        }

        if command.is_none() {
            place_cursor(&state);
        }

        // Check if cursors in other windows on the same buffer are still valid
        check_lnums(false);

        // Did not read the file, need to show some info about the file.
        // Do this after setting the cursor.
        if state.oldbuf && !state.auto_buf {
            report_file_info();
        }

        Buf::current().b_last_used = os_time_raw();
        if let Some(command) = &command_text {
            let _ = do_cmdline(Some(command.to_bytes()), None, DoCmdOpts::VERBOSE);
        }
        if Buf::current().b_kmap_state as c_int & KEYMAP_INIT != 0 {
            let _ = keymap_init();
        }

        drop(redraw_off.take());
        if !skip_redraw.get() {
            recenter(so, state.topline, command.is_some());
        }

        // Change directories when the 'acd' option is set.
        do_autochdir();
    }

    if let Some(old) = old_curbuf.get() {
        buf_terminal_check_size(old);
    }
    if !old_curbuf.valid() || !old_curbuf.is(Buf::current_or_none()) {
        buf_terminal_check_size(Buf::current());
    }

    drop(redraw_off.take());
    if did_set_swapcommand {
        set_vim_var_string(Vv::Swapcommand, None);
    }
    retval
}

/// Work out whether `do_ecmd` is being asked for another file, expanding the
/// name it was given on the way.
fn resolve_target(
    fnum: c_int,
    ffname: &mut Option<CString>,
    sfname: &mut Option<CString>,
    flags: EcmdFlags,
) -> Target {
    if fnum != 0 {
        if fnum == Buf::current().handle {
            // file is already being edited, nothing to do
            return Target::AlreadyHere;
        }
        return Target::Editing(true);
    }

    // if no short name given, use ffname for short name
    if sfname.is_none() {
        sfname.clone_from(ffname);
    }

    if flags.has(EcmdFlags::ADDBUF | EcmdFlags::ALTBUF)
        && ffname.as_deref().is_none_or(CStr::is_empty)
    {
        return Target::Nothing;
    }
    let Some(name) = ffname.as_deref() else {
        return Target::Editing(true);
    };
    if name.is_empty() {
        let current = &Buf::current().name;
        if current.full().is_none() {
            // there is no file name
            return Target::Editing(false);
        }
        // re-edit with same file name
        *ffname = current.full().map(CStr::to_owned);
        *sfname = current.shown().map(CStr::to_owned);
    }
    // may expand to full path name
    let name = ffname.as_deref().unwrap_or_default();
    *ffname = Some(fix_fname(name).as_cstr().to_owned());
    Target::Editing(otherfile(ffname.as_deref().unwrap_or_default()))
}

/// Re-edit the current file in the buffer it already has: store the contents
/// so the reload can be undone, then empty it out.
///
/// Answers false when an autocommand pulled the buffer out from under us.
fn reuse_current_buffer(state: &mut Ecmd) -> bool {
    set_last_cursor(Win::current());
    if state.newlnum == newlnum::LAST as LineNr || state.newlnum == newlnum::LASTL as LineNr {
        state.newlnum = Win::current().w_cursor.lnum;
        state.solcol = Win::current().w_cursor.col;
    }
    let buf = Buf::current();
    // The buffer's own file name; see [`switch`]'s copy for why it is owned
    // rather than borrowed.
    let new_name: Option<CString> = buf.name.shown().map(CStr::to_owned);
    let bufref = BufRef::of(buf);

    // If the buffer was used before, store the current contents so that
    // the reload can be undone.  Do not do this if the (empty) buffer is
    // being re-used for another file.
    if !Buf::current().b_flags.has(BufFlags::NEVERLOADED)
        && (p_ur() < 0 || Buf::current().b_ml.ml_line_count as OptInt <= p_ur())
    {
        // Sync first so that this is a separate undo-able action.
        u_sync(false);
        if u_savecommon(
            Buf::current(),
            0,
            Buf::current().b_ml.ml_line_count + 1,
            0,
            true,
        )
        .is_err()
        {
            return false;
        }
        u_unchanged(Buf::current());
        buf_freeall(Buf::current(), BFA_KEEP_UNDO as c_int);
        // Tell readfile() not to clear or reload undo info.
        state.readfile_flags = READ_KEEP_UNDO as c_int;
    } else {
        // Free all things for buffer.
        buf_freeall(Buf::current(), 0);
    }

    // If autocommands deleted the buffer we were going to re-edit, give up
    // and jump to the end.
    if !bufref.valid() {
        delbuf_msg(new_name.as_deref());
        return false;
    }
    drop(new_name);

    // If autocommands change buffers under our fingers, forget about
    // re-editing the file.  Should do the buf_clear_file(), but perhaps
    // the autocommands changed the buffer...  They may also abort script
    // processing.
    if !buf.is_current() || aborting() {
        return false;
    }
    buf_clear_file(Buf::current());
    // clear '[ and '] marks
    Buf::current().b_op_start.lnum = 0;
    Buf::current().b_op_end.lnum = 0;
    true
}

/// Set the cursor and initialise the window, then read the file (or fire
/// BufEnter/BufWinEnter when there is nothing to read).
fn enter_new_buffer(
    args: &mut EcmdArgs<'_, '_>,
    old_curbuf: &mut BufRef,
    state: &mut Ecmd,
    retval: &mut Result<(), Failed>,
) {
    let flags = args.flags;
    // Set cursor and init window before reading the file and executing
    // autocommands.  This allows for the autocommands to position the cursor.
    curwin_init();

    // It's possible that all lines in the buffer changed.  Need to update
    // automatic folding for all windows where it's used.
    for win in tab_windows() {
        if win.w_buffer.is_current() {
            fold_update_all(win);
        }
    }

    // Change directories when the 'acd' option is set.
    do_autochdir();

    // Careful: open_buffer() and apply_autocmds() may change the current
    // buffer and window.
    let orig_pos = Win::current().w_cursor;
    state.topline = Win::current().w_topline;
    if !state.oldbuf {
        // need to read the file
        swap_exists_action.set(SEA_DIALOG);
        // set/reset 'ro' flag
        Buf::current().b_flags |= BufFlags::CHECK_RO;
        // Open the buffer and read the file.
        if flags.has(EcmdFlags::NOWINENTER) {
            state.readfile_flags |= READ_NOWINENTER as c_int;
        }
        let opened = open_buffer(
            false,
            args.excmd.as_deref_mut(),
            ReadFlags::from_bits(state.readfile_flags),
        );
        if should_abort_err(opened) {
            *retval = Err(Failed);
        }
        if swap_exists_action.get() == SEA_QUIT {
            *retval = Err(Failed);
        }
        handle_swap_exists(Some(*old_curbuf));
    } else {
        // Read the modelines, but only to set window-local options.  Any
        // buffer-local options have already been set and may have been changed
        // by the user.
        do_modelines(OptionSetFlags::WINONLY);
        fire_retval(AutoEvent::BufEnter, Buf::current(), retval);
        if !flags.has(EcmdFlags::NOWINENTER) {
            fire_retval(AutoEvent::BufWinEnter, Buf::current(), retval);
        }
    }
    check_arg_idx(Win::current());

    // If autocommands change the cursor position or topline, we should keep
    // it.  Also when it moves within a line.  But not when it moves to the
    // first non-blank.
    if !equalpos(Win::current().w_cursor, orig_pos) {
        let cursor = Win::current().w_cursor;
        let first_nonblank = skip::white(Lines::current().line(cursor.lnum)) as c_int;
        if cursor.lnum != orig_pos.lnum || cursor.col != first_nonblank {
            state.newlnum = Win::current().w_cursor.lnum;
            state.newcol = Win::current().w_cursor.col;
        }
    }
    if Win::current().w_topline == state.topline {
        state.topline = 0;
    }

    // Even when the cursor didn't move we need to recompute topline.
    changed_line_abv_curs();
    maketitle();
}

/// Put the cursor where the caller, the autocommands or the buffer's last
/// known position asked for.
fn place_cursor(state: &Ecmd) {
    if state.newcol >= 0 {
        // position set by autocommands
        Win::current().w_cursor.lnum = state.newlnum;
        Win::current().w_cursor.col = state.newcol;
        check_cursor(Win::current());
    } else if state.newlnum > 0 {
        // line number from caller or old position
        Win::current().w_cursor.lnum = state.newlnum;
        check_cursor_lnum(Win::current());
        if state.solcol >= 0 && !p_sol() {
            // 'sol' is off: use the last known column.
            Win::current().w_cursor.col = state.solcol;
            check_cursor_col(Win::current());
            Win::current().w_cursor.coladd = 0;
            Win::current().w_set_curswant = true;
        } else {
            beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
        }
    } else {
        // no line number, go to last line in Ex mode
        if exmode_active.get() {
            Win::current().w_cursor.lnum = Buf::current().b_ml.ml_line_count;
        }
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }
}

/// The "file, N lines" line `:edit` prints when it did not read the file.
fn report_file_info() {
    let msg_scroll_save = msg_scroll.get();
    // Obey the 'O' flag in 'cpoptions': overwrite any previous file message.
    if shortmess(ShmFlag::OVERALL)
        && msg_listdo_overwrite.get() == 0
        && !exiting.get()
        && p_verbose() == 0
    {
        msg_scroll.set(0);
    }
    if msg_scroll.get() == 0 {
        // wait a bit when overwriting an error msg
        msg_check_for_delay(false);
    }
    say::start();
    msg_scroll.set(msg_scroll_save);
    msg_scrolled_ign.set(true);

    if !shortmess(ShmFlag::FILEINFO) {
        fileinfo(0, 1, false);
    }

    msg_scrolled_ign.set(false);
}

/// Recompute the scroll position for the new cursor line, centring it when the
/// autocommands left the window's top line alone and there is no `+cmd`.
/// `so` is the current window's scroll margin.
fn recenter(so: ScrollOff, topline: LineNr, has_command: bool) {
    let n = so.get();
    if topline == 0 && !has_command {
        // force the cursor to be vertically centered in the window
        so.set(999);
    }
    update_topline(Win::current());
    Win::current().w_scbind_pos = plines_m_win_fill(Win::current(), 1, Win::current().w_topline);
    so.set(n);
    // redraw this buffer later
    redraw_curbuf_later(UPD_NOT_VALID);
}
