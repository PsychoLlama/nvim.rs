//! Reading a file into a buffer -- `open_buffer()` and the scratch forms.
//!
//! [`open_buffer`] is what turns an empty `buf_T` into one with text: read the
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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::{ptr, slice};

use super::*;
use crate::autocmd::{
    EVENT_BUFENTER, EVENT_BUFFILEPOST, EVENT_BUFFILEPRE, EVENT_BUFWINENTER, EVENT_STDINREADPOST,
    aucmd_prepbuf, aucmd_restbuf,
};
use crate::change::{changed, save_file_ff};
use crate::charset::buf_init_chartab;
use crate::fileio::{prep_exarg, readfile};
use crate::help::get_local_additions;
use crate::indent_c::parse_cino;
use crate::main::{getout, got_int, p_cpo, readonlymode, v_dying};
use crate::memfile::MfDirty;
use crate::memline::{ml_get, ml_get_buf, ml_get_buf_len, ml_open};
use crate::memory::xrealloc;
use crate::r#move::WinValid;
use crate::option::{boolean_optval, set_option_value_give_err};
use crate::options::{kOptBufhidden, kOptBuftype, kOptSwapfile};
use crate::os::fs::os_getperm;
use crate::pos::MAXLNUM;
use crate::strings::vim_strchr;
use crate::types::{
    FAIL, NUL, OK, OptInt, OptVal, OptValData, OptionSetFlags, String_0, StringBuilder, aco_save_T,
    colnr_T, exarg_T, handle_T, int64_t, linenr_T, size_t, varnumber_T, win_T,
};
use crate::winlayer::buffers;
use ::libc::strcmp;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// `readfile()`: read `fname` into the current buffer between `from` and
/// `to`, appending after line `lnum`.
///
/// Fires `BufReadPre`/`BufReadPost` (or the `FileRead*` pair) and may leave
/// another buffer current: nothing held survives it.
#[expect(clippy::too_many_arguments)]
fn read_file(
    ffname: *mut c_char,
    fname: *mut c_char,
    lnum: linenr_T,
    from: linenr_T,
    to: linenr_T,
    eap: *mut exarg_T,
    flags: c_int,
    silent: bool,
) -> c_int {
    // SAFETY: two NUL-terminated names or nulls, and the caller's own `eap`.
    unsafe { readfile(ffname, fname, lnum, from, to, eap, flags, silent) }
}

/// Open the memline (and the swap file) for `buf`.
fn open_memline(mut buf: Buf) -> c_int {
    // SAFETY: a live buffer.
    unsafe { ml_open(buf.raw()) }
}

/// The mode bits of `fname`, negative when it cannot be stat'ed.
fn permissions_of(fname: *mut c_char) -> c_int {
    // SAFETY: a NUL-terminated file name.
    unsafe { os_getperm(fname) as c_int }
}

fn set_changed(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { changed(buf.raw()) };
}

/// Remember the file format the buffer was read with.
fn save_fileformat(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { save_file_ff(buf.raw()) };
}

fn init_chartab(mut buf: Buf) {
    // SAFETY: a live buffer; `false` is upstream's `global` flag.
    unsafe { buf_init_chartab(buf.raw(), false) };
}

fn parse_cindent_options(mut buf: Buf) {
    // SAFETY: a live buffer.
    unsafe { parse_cino(buf.raw()) };
}

/// Whether `'cpoptions'` contains `flag`.
fn cpo_has(flag: c_int) -> bool {
    // SAFETY: `p_cpo` is a NUL-terminated option value.
    !unsafe { vim_strchr(p_cpo.get(), flag) }.is_null()
}

/// Whether this buffer has no readable file behind its name.
fn no_file_to_read(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { bt_nofileread(buf.raw()) }
}

fn is_help_buffer(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { bt_help(buf.raw()) }
}

/// Populate `*local-additions*` in `help.txt`.
fn collect_local_additions() {
    // SAFETY: reads the runtime path and the current buffer.
    unsafe { get_local_additions() };
}

fn empty_buffer(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { buf_is_empty(buf.raw()) }
}

/// `b:changedtick`.
fn changedtick(mut buf: Buf) -> varnumber_T {
    // SAFETY: a live buffer.
    unsafe { buf_get_changedtick(buf.raw()) }
}

/// Leave the editor with exit code `n` -- never returns.
fn bail_out(n: c_int) {
    // SAFETY: unwinds and exits; nothing here is reached again.
    unsafe { getout(n) };
}

/// Fill in `eap` with the file format and encoding of `buf`, as the reload
/// paths need.
fn prepare_exarg(eap: &mut exarg_T, mut buf: Buf) {
    // SAFETY: a local to fill in, and a live buffer.
    unsafe { prep_exarg(eap, buf.raw()) };
}

fn set_option_string(id: c_int, value: &'static CStr) {
    let string = String_0 {
        data: value.as_ptr().cast_mut(),
        size: value.count_bytes(),
    };
    let val = OptVal {
        type_0: kOptValTypeString,
        data: OptValData { string },
    };
    set_option_value_give_err(id, val, OptionSetFlags::LOCAL);
}

fn set_option_false(id: c_int) {
    set_option_value_give_err(id, boolean_optval(Some(false)), OptionSetFlags::LOCAL);
}

/// Whether lines `a` and `b` of the two buffers differ.
fn lines_differ(mut buf: Buf, lnum: linenr_T) -> bool {
    // SAFETY: two live buffers and a line number inside both, the caller
    // having compared the line counts.
    unsafe { strcmp(ml_get_buf(buf.raw(), lnum), ml_get(lnum)) != 0 }
}

/// Line `lnum` of `buf` as bytes, its terminating NUL excluded.
fn line_bytes<'a>(mut buf: Buf, lnum: linenr_T) -> &'a [u8] {
    // SAFETY: a live buffer and a line of it; `ml_get_buf` answers that many
    // readable bytes, and the line stays put until the memline is touched.
    unsafe {
        let len = ml_get_buf_len(buf.raw(), lnum) as usize;
        slice::from_raw_parts(ml_get_buf(buf.raw(), lnum).cast::<u8>(), len)
    }
}

/// Run `f` with `buf` current and in a window, then restore what was current.
///
/// The `aucmd_prepbuf`/`aucmd_restbuf` pair must always be matched, which is
/// what makes this a scope rather than two calls: only a panic can skip the
/// restore, and a panic already abandons the editor state upstream's `longjmp`
/// would have unwound.
fn in_buffer<R>(mut buf: Buf, f: impl FnOnce() -> R) -> R {
    let mut aco = aco_save_T::default();
    // SAFETY: a local to save into, and a live buffer.
    unsafe { aucmd_prepbuf(&raw mut aco, buf.raw()) };
    let answer = f();
    // SAFETY: the state `aucmd_prepbuf` has just saved.
    unsafe { aucmd_restbuf(&raw mut aco) };
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
fn read_buffer(read_stdin: bool, eap: *mut exarg_T, flags: c_int) -> c_int {
    let silent = short_mess(SHM_FILEINFO as c_int);

    let line_count = cur_buf().line_count();
    let (ffname, fname) = match read_stdin {
        true => (ptr::null_mut(), ptr::null_mut()),
        false => (cur_buf().b_ffname, cur_buf().b_fname),
    };
    let last = MAXLNUM as linenr_T;
    let mut retval = read_file(
        ffname,
        fname,
        line_count,
        0,
        last,
        eap,
        flags | READ_BUFFER as c_int,
        silent,
    );
    if retval == OK {
        // Delete the binary lines.
        for _ in 0..line_count {
            delete_line(1 as linenr_T);
        }
    } else {
        // Delete the converted lines.
        while cur_buf().line_count() > line_count {
            delete_line(line_count);
        }
    }
    // Put the cursor on the first line.
    let mut cursor = cur_win().cursor();
    cursor.lnum = 1 as linenr_T;
    cursor.col = 0 as colnr_T;

    if read_stdin {
        // Set or reset 'modified' before executing autocommands, so that it
        // can be changed there.
        let buf = cur_buf();
        if !readonlymode.get() && !empty_buffer(buf) {
            set_changed(buf);
        } else if retval != FAIL {
            unchanged_now(buf, false, true);
        }
        fire_retval(EVENT_STDINREADPOST, cur_buf(), &mut retval);
    }
    retval
}

/// Ensure buffer `buf` is loaded.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_ensure_loaded(buf: *mut buf_T) -> bool {
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    if !buf.b_ml.ml_mfp.is_null() {
        // already open (common case)
        return true;
    }
    // Make sure the buffer is in a window.  `status` can be OK or NOTDONE
    // (which also means ok/done).
    let status = in_buffer(buf, || open_buffer_inner(false, ptr::null_mut(), 0));
    status != FAIL
}

/// Open the current buffer: open the memfile and read the file into memory.
///
/// With `read_stdin` the text comes from standard input instead; `eap` forces
/// `'fileformat'`/`'fileencoding'` and `flags_arg` is passed on to
/// `readfile()`.
///
/// # Safety
/// `curbuf` and `curwin` must be set, and `eap` be null or a live `exarg_T`.
pub unsafe fn open_buffer(read_stdin: bool, eap: *mut exarg_T, flags_arg: c_int) -> c_int {
    open_buffer_inner(read_stdin, eap, flags_arg)
}

fn open_buffer_inner(read_stdin: bool, eap: *mut exarg_T, flags_arg: c_int) -> c_int {
    let mut flags = flags_arg;
    let mut retval = OK;
    let old_tw: OptInt = cur_buf().b_p_tw;
    let mut read_fifo = false;
    let silent = short_mess(SHM_FILEINFO as c_int);

    // The 'readonly' flag is only set when BufFlags::NEVERLOADED is being reset.
    // When re-entering the same buffer, it should not change, because the
    // user may have reset the flag by hand.
    let mut buf = cur_buf();
    if readonlymode.get() && !buf.b_ffname.is_null() && buf.b_flags.has(BufFlags::NEVERLOADED) {
        buf.b_p_ro = 1;
    }

    if open_memline(buf) == FAIL {
        return no_memfile(old_tw);
    }

    // Do not sync this buffer yet, may first want to read the file.
    set_dirty(buf, MfDirty::YesNoSync);

    // The autocommands in readfile() may change the buffer, but only AFTER
    // reading the file.
    let old_curbuf = BufRef::of(buf);
    buf.b_modified_was_set = false;

    // mark cursor position as being invalid
    cur_win().w_valid = WinValid::NONE;

    // A buffer without an actual file should not use the buffer name to read
    // a file.
    if no_file_to_read(buf) {
        flags |= READ_NOFILE as c_int;
    }

    // Read the file if there is one.
    if !buf.b_ffname.is_null() {
        let save_bin = buf.b_p_bin;
        let perm = permissions_of(buf.b_ffname);
        // `S_ISFIFO(perm) || S_ISSOCK(perm)`; the literals stay literals so
        // that ffigen does not export them as new C declarations.
        if perm >= 0 && (perm & __S_IFMT == 0o10000 || perm & __S_IFMT == 0o140000) {
            read_fifo = true;
        }
        if read_fifo {
            buf.b_p_bin = 1;
        }
        let fifo = if read_fifo { READ_FIFO as c_int } else { 0 };
        let (ffname, fname, last) = (buf.b_ffname, buf.b_fname, MAXLNUM as linenr_T);
        let read = flags | READ_NEW as c_int | fifo;
        retval = read_file(ffname, fname, 0, 0, last, eap, read, silent);
        if read_fifo {
            cur_buf().b_p_bin = save_bin;
            if retval == OK {
                // don't add READ_FIFO here, otherwise we won't be able to
                // detect the encoding
                retval = read_buffer(false, eap, flags);
            }
        }
        // Help buffer: populate *local-additions* in help.txt
        if is_help_buffer(cur_buf()) {
            collect_local_additions();
        }
    } else if read_stdin {
        let save_bin = buf.b_p_bin;

        // First read the text in binary mode into the buffer.  Then read from
        // that same buffer and append at the end.  This makes it possible to
        // retry when 'fileformat' or 'fileencoding' was guessed wrong.
        buf.b_p_bin = 1;
        let (none, last) = (ptr::null_mut::<c_char>(), MAXLNUM as linenr_T);
        let read = flags | (READ_NEW as c_int + READ_STDIN as c_int);
        retval = read_file(none, none, 0, 0, last, ptr::null_mut(), read, silent);
        cur_buf().b_p_bin = save_bin;
        if retval == OK {
            retval = read_buffer(true, eap, flags);
        }
    }

    // Can now sync this buffer in ml_sync_all().
    let mut buf = cur_buf();
    if dirty(buf) == Some(MfDirty::YesNoSync) {
        set_dirty(buf, MfDirty::Yes);
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
    if got_int.get() && cpo_has(CPO_INTMOD)
        || buf.b_modified_was_set
        || aborting_now() && cpo_has(CPO_INTMOD)
    {
        set_changed(buf);
    } else if retval != FAIL && !read_stdin && !read_fifo {
        unchanged_now(buf, false, true);
    }
    // `changed()` notifies the `b:changedtick` watchers, which can re-enter
    // Lua and leave another buffer current -- so from here on `curbuf` and
    // `curwin` are re-read at each step, exactly as upstream reads the globals.
    save_fileformat(cur_buf()); // keep this fileformat

    // Set last_changedtick to avoid triggering a TextChanged autocommand right
    // after it was added.
    let mut buf = cur_buf();
    let tick = changedtick(buf);
    buf.b_last_changedtick = tick;
    buf.b_last_changedtick_i = tick;
    buf.b_last_changedtick_pum = tick;

    // require "!" to overwrite the file, because it wasn't read completely
    if aborting_now() {
        cur_buf().b_flags |= BufFlags::READERR;
    }

    // Need to update automatic folding.  Do this before the autocommands, they
    // may use the fold info.
    fold_update_all(cur_win());

    // need to set w_topline, unless some autocommand already did that.
    let mut win = cur_win();
    if !win.w_valid.has(WinValid::TOPLINE) {
        win.w_topline = 1 as linenr_T;
        win.w_topfill = 0;
    }
    fire_retval(EVENT_BUFENTER, cur_buf(), &mut retval);

    if retval == FAIL {
        return retval;
    }

    // The autocommands may have changed the current buffer.  Apply the
    // modelines to the correct buffer, if it still exists and is loaded.
    let Some(old) = old_curbuf.get().filter(|b| !b.b_ml.ml_mfp.is_null()) else {
        return retval;
    };
    // Go to the buffer that was opened, make sure it is in a window.
    in_buffer(old, || {
        do_modelines(OptionSetFlags::NONE);
        cur_buf()
            .b_flags
            .clear(BufFlags::CHECK_RO | BufFlags::NEVERLOADED);

        if flags & READ_NOWINENTER as c_int == 0 {
            fire_retval(EVENT_BUFWINENTER, cur_buf(), &mut retval);
        }
    });
    retval
}

/// There MUST be a memfile, otherwise we can't do anything.  If we can't
/// create one for the current buffer, take another buffer.
fn no_memfile(old_tw: OptInt) -> c_int {
    // SAFETY: the current buffer, which has not been freed yet.
    unsafe { close_buffer(ptr::null_mut::<win_T>(), curbuf.get(), 0, false, false) };

    curbuf.set(ptr::null_mut::<buf_T>());
    if let Some(buf) = buffers().find(|b| !b.b_ml.ml_mfp.is_null()) {
        curbuf.set(buf.raw());
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
    enter_buffer(cur_buf());
    if old_tw != cur_buf().b_p_tw {
        recheck_colorcolumn(cur_win());
    }
    FAIL
}

/// The memfile's dirty state, `None` when the buffer has no memfile.
fn dirty(mut buf: Buf) -> Option<MfDirty> {
    let mfp = buf.b_ml.ml_mfp;
    // SAFETY: a live buffer's memfile is live.
    (!mfp.is_null()).then(|| unsafe { (*mfp).mf_dirty })
}

fn set_dirty(mut buf: Buf, state: MfDirty) {
    let mfp = buf.b_ml.ml_mfp;
    if !mfp.is_null() {
        // SAFETY: a live buffer's memfile is live.
        unsafe { (*mfp).mf_dirty = state };
    }
}

// ---------------------------------------------------------------------------
// Comparing a buffer with the file behind it

/// Whether the file `buf` was read from now differs from what is in memory.
///
/// The file is read into a hidden dummy buffer and compared line by line.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn buf_contents_changed(buf: *mut buf_T) -> bool {
    // SAFETY: the caller's promise -- a live buffer.
    let buf = unsafe { Buf::new(buf) };
    let mut differ = true;

    // SAFETY: two null names ask for a nameless buffer.
    let newbuf = unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1, BLN_DUMMY as c_int) };
    if newbuf.is_null() {
        return true;
    }
    // SAFETY: `buflist_new` has just answered a live buffer.
    let newbuf = unsafe { Buf::new(newbuf) };

    let mut ea = exarg_T::default();
    prepare_exarg(&mut ea, buf);
    in_buffer(newbuf, || {
        block_autocmds_now();
        let read = READ_NEW as c_int | READ_DUMMY as c_int;
        let (ffname, fname, last) = (buf.b_ffname, buf.b_fname, MAXLNUM as linenr_T);
        if open_memline(cur_buf()) == OK
            && read_file(ffname, fname, 0, 0, last, &raw mut ea, read, false) == OK
            && buf.line_count() == cur_buf().line_count()
        {
            differ = (1..=cur_buf().line_count()).any(|lnum| lines_differ(buf, lnum));
        }
        free(ea.cmd);
    });
    if cur_buf() != newbuf {
        // SAFETY: `buflist_new` answered it and nothing has freed it: the
        // dummy is not in any window, so `close_buffer` cannot have run.
        unsafe { wipe_buffer(newbuf.raw(), false) };
    }
    unblock_autocmds_now();
    differ
}

// ---------------------------------------------------------------------------
// The forms with no file behind them

/// Open a scratch buffer (`'buftype'` `nofile`, hidden, no swap file) in the
/// current window, named `bufname` if that is not null.
///
/// # Safety
/// `bufname` must be null or NUL-terminated, and `curwin` be set.
pub unsafe fn buf_open_scratch(bufnr: handle_T, bufname: *mut c_char) -> c_int {
    let none = ptr::null_mut::<c_char>();
    let one = ECMD_ONE as c_int as linenr_T;
    let hide = ECMD_HIDE as c_int;
    if edit_file(bufnr, none, none, ptr::null_mut(), one, hide, cur_win()) == FAIL {
        return FAIL;
    }
    if !bufname.is_null() {
        fire(EVENT_BUFFILEPRE, cur_buf());
        // SAFETY: the current buffer, and the caller's NUL-terminated name.
        unsafe { setfname(curbuf.get(), bufname, ptr::null_mut(), true) };
        fire(EVENT_BUFFILEPOST, cur_buf());
    }
    set_option_string(kOptBufhidden, c"hide");
    set_option_string(kOptBuftype, c"nofile");
    set_option_false(kOptSwapfile);
    let mut win = cur_win();
    win.w_onebuf_opt.wo_scb = 0; // reset 'scrollbind'
    win.w_onebuf_opt.wo_crb = 0; // reset 'cursorbind'
    OK
}

/// Read lines `start` to `end` of `buf` into `sb`, NL-separated, with the
/// buffer's embedded NULs translated back from newlines.
///
/// # Safety
/// `buf` must be a live buffer, `sb` a live `StringBuilder`, and `start` and
/// `end` lines of the buffer.
pub unsafe fn read_buffer_into(
    buf: *mut buf_T,
    start: linenr_T,
    end: linenr_T,
    sb: *mut StringBuilder,
) {
    debug_assert!(!buf.is_null(), "buf");
    debug_assert!(!sb.is_null(), "sb");
    // SAFETY: the caller's promise -- a live buffer and a live builder.
    let (buf, mut out) = unsafe { (Buf::new(buf), Builder::of(&mut *sb)) };
    if buf.b_ml.ml_flags & ML_EMPTY != 0 {
        return;
    }

    let mut lnum = start;
    let mut line = line_bytes(buf, lnum);
    let mut written = 0usize;
    loop {
        let len = if line.is_empty() {
            0
        } else if line[written] == NL as u8 {
            // NL -> NUL translation
            out.push(NUL as c_char);
            1
        } else {
            let rest = &line[written..];
            let len = rest
                .iter()
                .position(|&b| b == NL as u8)
                .unwrap_or(rest.len());
            out.extend(&rest[..len]);
            len
        };

        if len == line.len() - written {
            // Finished a line, add a NL, unless this line should not have one.
            if lnum != end
                || buf.b_p_bin == 0 && buf.b_p_fixeol != 0
                || lnum != buf.b_no_eol_lnum && (lnum != buf.line_count() || buf.b_p_eol != 0)
            {
                out.push(NL as c_char);
            }
            lnum += 1;
            if lnum > end {
                break;
            }
            line = line_bytes(buf, lnum);
            written = 0;
        } else if len > 0 {
            written += len;
        }
    }
}

/// A `StringBuilder` -- `kvec_t(char)` -- borrowed part by part, so that
/// appending to it is ordinary code.
struct Builder<'a> {
    size: &'a mut size_t,
    capacity: &'a mut size_t,
    items: &'a mut *mut c_char,
}

impl<'a> Builder<'a> {
    fn of(sb: &'a mut StringBuilder) -> Self {
        Builder {
            size: &mut sb.size,
            capacity: &mut sb.capacity,
            items: &mut sb.items,
        }
    }

    /// `kv_resize` to `capacity` bytes.
    fn resize(&mut self, capacity: size_t) {
        *self.capacity = capacity;
        let old = self.items.cast();
        // SAFETY: `items` is null or this array's own allocation, and the new
        // size counts the same element type.
        *self.items = unsafe { xrealloc(old, size_of::<c_char>() * capacity) }.cast::<c_char>();
    }

    /// `kv_push`: append one byte, doubling the array when it is full.
    fn push(&mut self, byte: c_char) {
        if *self.size == *self.capacity {
            self.resize(if *self.capacity != 0 {
                *self.capacity << 1
            } else {
                8
            });
        }
        // SAFETY: the array has room for one more byte at `size`.
        unsafe { *self.items.add(*self.size) = byte };
        *self.size += 1;
    }

    /// `kv_concat_len`: append `bytes`, growing to the next power of two.
    fn extend(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        if *self.capacity < *self.size + bytes.len() {
            self.resize((*self.size + bytes.len()).next_power_of_two());
        }
        assert!(!self.items.is_null(), "(*sb).items");
        // SAFETY: the array has just been grown to hold `size + len` bytes,
        // and `bytes` is a buffer line, which is a different allocation.
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.items.add(*self.size).cast(),
                bytes.len(),
            )
        };
        *self.size += bytes.len();
    }
}
