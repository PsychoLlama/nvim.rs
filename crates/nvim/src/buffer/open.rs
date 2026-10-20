//! Reading a file into a buffer -- `open_buffer()` and the scratch forms.
//!
//! [`open_buffer`] is what turns an empty `Buffer` into one with text: read the
//! file (or stdin), set `'filetype'` and run the `BufRead`/`BufNewFile`
//! autocommands, initialise undo and the swap file, and mark the buffer
//! loaded.  [`buf_open_scratch`] and [`read_buffer_into`] are the two forms
//! that skip the file entirely, and [`buf_contents_changed`] re-reads a file
//! into a hidden dummy buffer so it can be compared with what is in
//! memory.
//!
//! `readfile()` fires autocommands and so may change the current buffer under
//! us -- which is why [`open_buffer`] takes a [`BufRef`] before it reads and
//! re-`get`s it afterwards, and why every stage below re-reads `curbuf`
//! rather than holding one across a call.  [`in_buffer`] is the
//! `aucmd_prepbuf`/`aucmd_restbuf` pair, which must always be matched.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::memline::MlFlags;
use crate::types::AutoEvent;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::*;
use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::change::{changed, save_file_ff};
use crate::charset::buf_init_chartab;
use crate::fileio::{Loaded, ReadFlags, prep_exarg, readfile_named};
use crate::getchar::state::got_int;
use crate::help::get_local_additions;
use crate::indent_c::parse_cino_for;
use crate::memfile::MfDirty;
use crate::memline::ml_open;
use crate::r#move::WinValid;
use crate::option::{boolean_optval, cpo_has, set_option_value_give_err};
use crate::options::{kOptBufhidden, kOptBuftype, kOptSwapfile};
use crate::os::fs::getperm;
use crate::pos::MAXLNUM;
use crate::startup::{getout, readonlymode, v_dying};
use crate::types::{
    AcoSave, ColNr, CpoFlag, ExArg, Failed, Handle, LineNr, NUL, OptInt, OptVal, OptionSetFlags,
    ShmFlag, VarNumber, int64_t,
};
use crate::winlayer::buffers;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// `readfile()`: read `fname` into the current buffer between `from` and
/// `to`, appending after line `lnum`.
///
/// Fires `BufReadPre`/`BufReadPost` (or the `FileRead*` pair) and may leave
/// another buffer current: nothing held survives it.
#[expect(clippy::too_many_arguments)]
fn read_file(
    ffname: Option<&CStr>,
    fname: Option<&CStr>,
    lnum: LineNr,
    from: LineNr,
    to: LineNr,
    excmd: Option<&mut ExArg>,
    flags: ReadFlags,
    silent: bool,
) -> Result<Loaded, Failed> {
    readfile_named(ffname, fname, lnum, from, to, excmd, flags, silent)
}

/// The buffer's two names, copied: a read fires autocommands that may change
/// them while the reader holds what it was given.
fn names_of(buffer: Buf) -> (Option<CString>, Option<CString>) {
    (
        buffer.name.full().map(CStr::to_owned),
        buffer.name.shown().map(CStr::to_owned),
    )
}

/// Open the memline (and the swap file) for `buffer`.
fn open_memline(buffer: Buf) -> Result<(), Failed> {
    ml_open(buffer)
}

fn set_changed(buffer: Buf) {
    changed(buffer);
}

/// Remember the file format the buffer was read with.
fn save_fileformat(buffer: Buf) {
    save_file_ff(buffer);
}

fn init_chartab(buffer: Buf) {
    buf_init_chartab(buffer, false);
}

fn parse_cindent_options(buffer: Buf) {
    parse_cino_for(buffer);
}

/// Populate `*local-additions*` in `help.txt`.
fn collect_local_additions() {
    get_local_additions();
}

fn empty_buffer(buffer: Buf) -> bool {
    buf_is_empty(buffer)
}

/// `b:changedtick`.
fn changedtick(buffer: Buf) -> VarNumber {
    buf_get_changedtick(buffer)
}

/// Leave the editor with exit code `n` -- never returns.
fn bail_out(n: c_int) {
    getout(n);
}

/// Fill in `args` with the file format and encoding of `buffer`, as the reload
/// paths need.
fn prepare_exarg(args: &mut ExArg, buffer: Buf) {
    // SAFETY: a local to fill in.
    prep_exarg(args, buffer);
}

fn set_option_string(id: c_int, value: &'static CStr) {
    set_option_value_give_err(id, OptVal::static_string(value), OptionSetFlags::LOCAL);
}

fn set_option_false(id: c_int) {
    set_option_value_give_err(id, boolean_optval(Some(false)), OptionSetFlags::LOCAL);
}

/// Whether line `lnum` of `buffer` differs from the current buffer's.
fn lines_differ(buffer: Buf, lnum: LineNr) -> bool {
    let (mut theirs, mut ours) = (buffer.lines(), Buf::current().lines());
    theirs.line(lnum) != ours.line(lnum)
}

/// Run `f` with `buffer` current and in a window, then restore what was current.
///
/// The `aucmd_prepbuf`/`aucmd_restbuf` pair must always be matched, which is
/// what makes this a scope rather than two calls: only a panic can skip the
/// restore, and a panic already abandons the editor state upstream's `longjmp`
/// would have unwound.
fn in_buffer<R>(buffer: Buf, f: impl FnOnce() -> R) -> R {
    let mut aco = AcoSave::default();
    aucmd_prepbuf(&mut aco, buffer);
    let answer = f();
    aucmd_restbuf(&mut aco);
    answer
}

// ---------------------------------------------------------------------------
// Small answers about the buffer list

/// Calculate the percentage that `part` is of the `whole`.
pub fn calc_percentage(part: int64_t, whole: int64_t) -> c_int {
    // With 32 bit longs and more than 21,474,836 lines multiplying by 100
    // causes an overflow, thus for large numbers divide instead.
    if part > 1000000 {
        (part / (whole / 100)) as c_int
    } else {
        (part * 100 / whole) as c_int
    }
}

/// The highest buffer number handed out so far.
pub fn get_highest_fnum() -> c_int {
    top_file_num.get() - 1
}

// ---------------------------------------------------------------------------
// Reading a file into the current buffer

/// Read the current buffer's text back out of itself and append at the end,
/// then drop what was there before.
///
/// This is the retry `'fileformat'`/`'fileencoding'` guessed wrong needs: the
/// bytes are already in the buffer, so re-reading them with the corrected
/// options costs no file access.
fn read_buffer(
    read_stdin: bool,
    excmd: Option<&mut ExArg>,
    flags: ReadFlags,
) -> Result<Loaded, Failed> {
    let silent = shortmess(ShmFlag::FILEINFO);

    let line_count = Buf::current().line_count();
    let (ffname, fname) = match read_stdin {
        true => (None, None),
        false => names_of(Buf::current()),
    };
    let last = MAXLNUM;
    let mut retval = read_file(
        ffname.as_deref(),
        fname.as_deref(),
        line_count,
        0,
        last,
        excmd,
        flags | ReadFlags::BUFFER,
        silent,
    );
    if retval == Ok(Loaded::Read) {
        // Delete the binary lines.
        for _ in 0..line_count {
            delete_line(1 as LineNr);
        }
    } else {
        // Delete the converted lines.
        while Buf::current().line_count() > line_count {
            delete_line(line_count);
        }
    }
    // Put the cursor on the first line.
    let mut cursor = Win::current().cursor();
    cursor.lnum = 1 as LineNr;
    cursor.col = 0 as ColNr;

    if read_stdin {
        // Set or reset 'modified' before executing autocommands, so that it
        // can be changed there.
        let buf = Buf::current();
        if !readonlymode.get() && !empty_buffer(buf) {
            set_changed(buf);
        } else if retval.is_ok() {
            unchanged_now(buf, false, true);
        }
        fire_retval(AutoEvent::StdinReadPost, Buf::current(), &mut retval);
    }
    retval
}

/// Ensure buffer `buffer` is loaded.
///
pub fn buf_ensure_loaded(buffer: Buf) -> bool {
    if buffer.b_ml.ml_mfp.is_some() {
        // already open (common case)
        return true;
    }
    // Make sure the buffer is in a window.  `status` can be OK or NOTDONE
    // (which also means ok/done).
    let status = in_buffer(buffer, || open_buffer_inner(false, None, ReadFlags::NONE));
    status.is_ok()
}

/// Open the current buffer: open the memfile and read the file into memory.
///
/// With `read_stdin` the text comes from standard input instead; `excmd` forces
/// `'fileformat'`/`'fileencoding'` and `flags_arg` is passed on to
/// `readfile()`.
///
pub fn open_buffer(
    read_stdin: bool,
    excmd: Option<&mut ExArg>,
    flags_arg: ReadFlags,
) -> Result<Loaded, Failed> {
    open_buffer_inner(read_stdin, excmd, flags_arg)
}

fn open_buffer_inner(
    read_stdin: bool,
    mut excmd: Option<&mut ExArg>,
    flags_arg: ReadFlags,
) -> Result<Loaded, Failed> {
    let mut flags = flags_arg;
    let mut retval = Ok(Loaded::Read);
    let old_tw: OptInt = Buf::current().b_p_tw;
    let mut read_fifo = false;
    let silent = shortmess(ShmFlag::FILEINFO);

    // The 'readonly' flag is only set when BufFlags::NEVERLOADED is being reset.
    // When re-entering the same buffer, it should not change, because the
    // user may have reset the flag by hand.
    let mut buf = Buf::current();
    if readonlymode.get() && !buf.name.full().is_none() && buf.b_flags.has(BufFlags::NEVERLOADED) {
        buf.b_p_ro = 1;
    }

    if open_memline(buf).is_err() {
        return no_memfile(old_tw).map(|()| Loaded::Read);
    }

    // Do not sync this buffer yet, may first want to read the file.
    buf.set_memfile_dirty(MfDirty::YesNoSync);

    // The autocommands in readfile() may change the buffer, but only AFTER
    // reading the file.
    let old_curbuf = BufRef::of(buf);
    buf.b_modified_was_set = false;

    // mark cursor position as being invalid
    Win::current().w_valid = WinValid::NONE;

    // A buffer without an actual file should not use the buffer name to read
    // a file.
    if buf_is_nofileread(Some(buf)) {
        flags |= ReadFlags::NOFILE;
    }

    // Read the file if there is one.
    if !buf.name.full().is_none() {
        let save_bin = buf.b_p_bin;
        let perm = buf.name.full().map_or(-1, getperm);
        // `S_ISFIFO(perm) || S_ISSOCK(perm)`; the literals stay literals so
        // that ffigen does not export them as new C declarations.
        if perm >= 0 && (perm & __S_IFMT == 0o10000 || perm & __S_IFMT == 0o140000) {
            read_fifo = true;
        }
        if read_fifo {
            buf.b_p_bin = 1;
        }
        let fifo = ReadFlags::FIFO.when(read_fifo);
        let ((ffname, fname), last) = (names_of(buf), MAXLNUM);
        let read = flags | ReadFlags::NEW | fifo;
        retval = read_file(
            ffname.as_deref(),
            fname.as_deref(),
            0,
            0,
            last,
            excmd.as_deref_mut(),
            read,
            silent,
        );
        if read_fifo {
            Buf::current().b_p_bin = save_bin;
            if retval == Ok(Loaded::Read) {
                // don't add READ_FIFO here, otherwise we won't be able to
                // detect the encoding
                retval = read_buffer(false, excmd.as_deref_mut(), flags);
            }
        }
        // Help buffer: populate *local-additions* in help.txt
        if buf_is_help(Some(Buf::current())) {
            collect_local_additions();
        }
    } else if read_stdin {
        let save_bin = buf.b_p_bin;

        // First read the text in binary mode into the buffer.  Then read from
        // that same buffer and append at the end.  This makes it possible to
        // retry when 'fileformat' or 'fileencoding' was guessed wrong.
        buf.b_p_bin = 1;
        let read = flags | ReadFlags::NEW | ReadFlags::STDIN;
        retval = read_file(None, None, 0, 0, MAXLNUM, None, read, silent);
        Buf::current().b_p_bin = save_bin;
        if retval == Ok(Loaded::Read) {
            retval = read_buffer(true, excmd, flags);
        }
    }

    // Can now sync this buffer in ml_sync_all().
    let buf = Buf::current();
    if buf.memfile_dirty() == Some(MfDirty::YesNoSync) {
        buf.set_memfile_dirty(MfDirty::Yes);
    }

    // if first time loading this buffer, init b_chartab[]
    if buf.b_flags.has(BufFlags::NEVERLOADED) {
        init_chartab(buf);
        parse_cindent_options(buf);
    }

    // Set/reset the Changed flag first, autocmds may change the buffer.
    // Apply the automatic commands, before processing the modelines.  So the
    // modelines have priority over autocommands.
    //
    // When reading stdin, the buffer contents always needs writing, so set the
    // changed flag.  Unless in readonly mode: "ls | nvim -R -".  When
    // interrupted and 'cpoptions' contains 'i' set changed flag.
    if got_int.get() && cpo_has(CpoFlag::INTMOD)
        || buf.b_modified_was_set
        || aborting_now() && cpo_has(CpoFlag::INTMOD)
    {
        set_changed(buf);
    } else if retval.is_ok() && !read_stdin && !read_fifo {
        unchanged_now(buf, false, true);
    }
    // `changed()` notifies the `b:changedtick` watchers, which can re-enter
    // Lua and leave another buffer current -- so from here on `curbuf` and
    // `curwin` are re-read at each step, exactly as upstream reads the globals.
    save_fileformat(Buf::current()); // keep this fileformat

    // Set last_changedtick to avoid triggering a TextChanged autocommand right
    // after it was added.
    let mut buf = Buf::current();
    let tick = changedtick(buf);
    buf.b_last_changedtick = tick;
    buf.b_last_changedtick_i = tick;
    buf.b_last_changedtick_pum = tick;

    // require "!" to overwrite the file, because it wasn't read completely
    if aborting_now() {
        Buf::current().b_flags |= BufFlags::READERR;
    }

    // Need to update automatic folding.  Do this before the autocommands, they
    // may use the fold info.
    invalidate_window_folds(Win::current());

    // need to set w_topline, unless some autocommand already did that.
    let mut win = Win::current();
    if !win.w_valid.has(WinValid::TOPLINE) {
        win.w_topline = 1 as LineNr;
        win.w_topfill = 0;
    }
    fire_retval(AutoEvent::BufEnter, Buf::current(), &mut retval);
    retval?;

    // The autocommands may have changed the current buffer.  Apply the
    // modelines to the correct buffer, if it still exists and is loaded.
    let Some(old) = old_curbuf.get().filter(|b| b.b_ml.ml_mfp.is_some()) else {
        return retval;
    };
    // Go to the buffer that was opened, make sure it is in a window.
    in_buffer(old, || {
        do_modelines(OptionSetFlags::NONE);
        Buf::current()
            .b_flags
            .clear(BufFlags::CHECK_RO | BufFlags::NEVERLOADED);

        if !flags.has(ReadFlags::NOWINENTER) {
            fire_retval(AutoEvent::BufWinEnter, Buf::current(), &mut retval);
        }
    });
    retval
}

/// There MUST be a memfile, otherwise we can't do anything.  If we can't
/// create one for the current buffer, take another buffer.
fn no_memfile(old_tw: OptInt) -> Result<(), Failed> {
    close_buffer(None, Buf::current(), 0, false, false);

    leave_curbuf();
    if let Some(buf) = buffers().find(|b| b.b_ml.ml_mfp.is_some()) {
        buf.make_current();
    }

    // If there is no memfile at all, exit.  This is OK, since there are no
    // changes to lose.
    if current_buf().is_none() {
        err(c"E82: Cannot allocate any buffer, exiting...");
        // Don't try to do any saving, with "curbuf" NULL almost nothing will
        // work.
        v_dying.set(2);
        bail_out(2);
    }

    err(c"E83: Cannot allocate buffer, using other one...");
    enter_buffer(Buf::current());
    if old_tw != Buf::current().b_p_tw {
        recheck_colorcolumn(Win::current());
    }
    Err(Failed)
}

// ---------------------------------------------------------------------------
// Comparing a buffer with the file behind it

/// Whether the file `buffer` was read from now differs from what is in memory.
///
/// The file is read into a hidden dummy buffer and compared line by line.
///
pub fn buf_contents_changed(buffer: Buf) -> bool {
    let mut differ = true;

    let newbuf = buflist_new(None, None, 1, BLN_DUMMY as c_int);
    if newbuf.is_none() {
        return true;
    }
    let newbuf = newbuf.expect("`buflist_new` has just answered a buffer");

    let mut ea = ExArg::default();
    prepare_exarg(&mut ea, buffer);
    in_buffer(newbuf, || {
        block_autocmds_now();
        let read = ReadFlags::NEW | ReadFlags::DUMMY;
        let ((ffname, fname), last) = (names_of(buffer), MAXLNUM);
        let (ffname, fname) = (ffname.as_deref(), fname.as_deref());
        if open_memline(Buf::current()).is_ok()
            && read_file(ffname, fname, 0, 0, last, Some(&mut ea), read, false) == Ok(Loaded::Read)
            && buffer.line_count() == Buf::current().line_count()
        {
            differ = (1..=Buf::current().line_count()).any(|lnum| lines_differ(buffer, lnum));
        }
    });
    if Buf::current() != newbuf {
        // SAFETY: `buflist_new` answered it and nothing has freed it: the
        // dummy is not in any window, so `close_buffer` cannot have run.
        wipe_buffer(newbuf, false);
    }
    unblock_autocmds_now();
    differ
}

// ---------------------------------------------------------------------------
// The forms with no file behind them

/// Open a scratch buffer (`'buftype'` `nofile`, hidden, no swap file) in the
/// current window, named `bufname` when given.
pub fn buf_open_scratch(bufnr: Handle, bufname: Option<&CStr>) -> Result<(), Failed> {
    let one = newlnum::ONE as LineNr;
    let hide = EcmdFlags::HIDE;
    edit_file(bufnr, None, one, hide, Win::current())?;
    if let Some(bufname) = bufname {
        fire(AutoEvent::BufFilePre, Buf::current());
        let _ = setfname(Buf::current(), Some(bufname), None, true);
        fire(AutoEvent::BufFilePost, Buf::current());
    }
    set_option_string(kOptBufhidden, c"hide");
    set_option_string(kOptBuftype, c"nofile");
    set_option_false(kOptSwapfile);
    let mut win = Win::current();
    win.w_onebuf_opt.wo_scb = 0; // reset 'scrollbind'
    win.w_onebuf_opt.wo_crb = 0; // reset 'cursorbind'
    Ok(())
}

/// Append lines `start` to `end` of `buffer` to `out`, NL-separated, with the
/// buffer's embedded NULs translated back from newlines. Line `start` is
/// read even when it is past `end`.
pub fn read_buffer_into(buffer: Buf, start: LineNr, end: LineNr, out: &mut Vec<u8>) {
    if buffer.b_ml.ml_flags.has(MlFlags::EMPTY) {
        return;
    }
    let mut lines = buffer.lines();
    let mut lnum = start;
    loop {
        // NL -> NUL translation.
        let line = lines.line(lnum);
        out.extend(
            line.iter()
                .map(|&b| if b == NL as u8 { NUL as u8 } else { b }),
        );
        // Finished a line, add a NL, unless this line should not have one.
        if lnum != end
            || buffer.b_p_bin == 0 && buffer.b_p_fixeol != 0
            || lnum != buffer.b_no_eol_lnum && (lnum != buffer.line_count() || buffer.b_p_eol != 0)
        {
            out.push(NL as u8);
        }
        lnum += 1;
        if lnum > end {
            break;
        }
    }
}
