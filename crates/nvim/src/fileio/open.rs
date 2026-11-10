//! Getting to the point where a file's bytes can be read.
//!
//! [`open_source`] is everything `readfile` does before the first `read()`:
//! letting `BufReadCmd`/`FileReadCmd` take over entirely, refusing names that
//! are too long or name a directory, working out whether the buffer is
//! read-only, recording the file's timestamp, opening it, making the swap
//! file, and running the `*ReadPre` autocommands — which may have changed
//! everything, so most of the work is checking that they did not.
//!
//! `prep_exarg`, `set_file_options` and `set_forced_fenc` are the `++opt`
//! side of the same question: what the caller has already decided about
//! `'fileformat'`, `'fileencoding'` and `'binary'`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::{Buffer, ColNr, Failed, Pos};
use std::fs::File;

use crate::autocmd::defs::AutoEvent;
use crate::buffer::BufFlags;
use crate::fileio::Loaded;
use crate::os::fs::{getperm, take_stdin_for_read};
use crate::os::uv_error::{UV_EFBIG, UV_ENOENT};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

use crate::bufwrite::translate;

use super::*;
use crate::option::defs::ShmFlag;
use crate::path::MAXPATHL;

/// `filemess` about the current buffer, with a translated note.
fn filemess_note(fname: Option<&CStr>, note: &'static CStr) {
    Buf::current().file_message(fname, translate(note));
}

/// One of the `*ReadPre`/`*ReadCmd` autocommands, which all take the same
/// shape: the short name as the io name, and either the current buffer or the
/// name again as the subject.
///
/// `for_file` picks the `File*` form, which names a file rather than a buffer.
fn read_autocmd(
    event: AutoEvent,
    sfname: Option<&CStr>,
    excmd: Option<&mut ExArg>,
    for_file: bool,
) -> bool {
    let sfname = sfname.map(XString::from_cstr);
    let (iofile, buf) = if for_file {
        (sfname.clone(), None)
    } else {
        (None, Buf::current_or_none())
    };
    apply_autocmds_exarg(event, iofile, sfname, false, buf, excmd)
}

/// The file, open and ready to read.
#[derive(Debug)]
pub(crate) struct Opened<'a> {
    /// The names, after the short one was substituted for the long one.
    pub fname: Option<&'a CStr>,
    pub sfname: Option<&'a CStr>,
    /// The file, or standard input; `None` when reading from the buffer.
    pub input: Option<File>,
    /// The file's mode bits, or a libuv error, or 0 when reading stdin.
    pub perm: c_int,
    /// Which line endings `'fileformats'` allows, read after the `*ReadPre`
    /// autocommands had their chance to change it.
    pub guess: FormatGuess,
}

/// The answer `readfile` gives when it stops before reading anything: why it
/// failed, or a directory, or `Ok` when an autocommand did the reading.
type Unread = Result<Loaded, ReadError>;

/// The answer for a refusal whose note is already on the screen.
const REFUSED: Unread = Err(ReadError::Reported(Failed));

/// The current buffer and its names as the read began, to tell when
/// autocommands change them -- which matters when "fname" and "sfname" are
/// the buffer's own names.
#[derive(Debug)]
struct BufferWatch {
    buffer: *mut Buffer,
    /// The addresses of the buffer's full and shown names: a name that
    /// moved was changed.
    full: usize,
    shown: usize,
    /// The name being read is the buffer's full name.
    using_full: bool,
    /// The name being read is the name the buffer shows.
    using_shown: bool,
}

impl BufferWatch {
    fn new(fname: Option<&CStr>, sfname: Option<&CStr>) -> Self {
        let is_named = |name: Option<&CStr>, own: Option<&CStr>| name.is_some() && name == own;
        let current = Buf::current();
        let (full, shown) = (current.name.full(), current.name.shown());
        BufferWatch {
            buffer: Buf::current_raw(),
            full: current.name.full_ptr().addr(),
            shown: current.name.shown_ptr().addr(),
            using_full: is_named(fname, full) || is_named(sfname, full),
            using_shown: is_named(fname, shown) || is_named(sfname, shown),
        }
    }

    /// Whether the current buffer, or the name being read, changed since.
    fn changed(&self) -> bool {
        Buf::current_raw() != self.buffer
            || (self.using_full && self.full != Buf::current().name.full_ptr().addr())
            || (self.using_shown && self.shown != Buf::current().name.shown_ptr().addr())
    }
}

/// Set the '[ mark to the line above where the lines go, line 1 if zero.
/// Answers the mark it replaced.
fn mark_insert_point(from: LineNr) -> Pos {
    let orig_start = Buf::current().b_op_start;
    Buf::current().b_op_start.lnum = if from == LineNr::ZERO {
        LineNr::ONE
    } else {
        from
    };
    Buf::current().b_op_start.col = ColNr::ZERO;
    orig_start
}

/// Open what `readfile` was asked to read.
///
/// `Err` carries the answer `readfile` should give: why it failed, or a
/// directory, or `Ok` when an autocommand did the reading itself.
#[allow(clippy::too_many_arguments)]
pub(crate) fn open_source<'a>(
    fname: Option<&'a CStr>,
    sfname: Option<&'a CStr>,
    from: LineNr,
    mut excmd: Option<&mut ExArg>,
    how: &How,
    silent: bool,
    msg_save: c_int,
) -> Result<Opened<'a>, Unread> {
    let watch = BufferWatch::new(fname, sfname);

    // After reading a file the cursor line changes, but we don't want
    // to display the line.
    EX_NO_REPRINT.set(true);
    // Don't display the file info for another buffer now.
    NEED_FILEINFO.set(false);

    // Use the short file name whenever possible: it avoids problems
    // with networks and when directory names change.
    let sfname = sfname.or(fname);
    let fname = sfname;

    // The BufReadCmd and FileReadCmd events intercept the reading
    // process by running the associated commands instead.
    if !how.filtering
        && !how.stdin
        && !how.buffer
        && let Some(answer) = run_read_cmd(sfname, from, excmd.as_deref_mut(), how)
    {
        return Err(answer);
    }

    MSG_SCROLL.set(c_int::from(
        !((shortmess(ShmFlag::OVER) && MSG_LISTDO_OVERWRITE.get() == 0) || Buf::current().b_help)
            || p_verbose() != 0,
    ));

    let perm = check_readable(fname, how, silent, msg_save)?;

    // Set the default or forced 'fileformat' and 'binary'.
    set_file_options(how.set_options, excmd.as_deref_mut());

    // When opening a new file take the readonly flag from the file.
    // The default is r/w and can be set to r/o below; don't reset it
    // in readonly mode, and only touch b_p_ro when BufFlags::CHECK_RO is set.
    let check_readonly = how.newfile && Buf::current().b_flags.has(BufFlags::CHECK_RO);
    if check_readonly && !READONLYMODE.get() {
        Buf::current().b_p_ro = c_int::from(false);
    }

    let (file_info, swap_mode) = if how.newfile && !how.stdin && !how.buffer && !how.fifo {
        remember_file_time(fname)
    } else {
        (None, -1)
    };

    // Check readonly.
    let mut file_readonly = false;
    // The file once it is open, or why it could not be opened.
    let mut input: Result<File, c_int> = Err(0);
    if !how.buffer && !how.stdin {
        let name = fname.unwrap_or_default();
        if !how.newfile || READONLYMODE.get() || perm & 0o222 == 0 || os_file_is_writable(name) == 0
        {
            file_readonly = true;
        }
        input = open_named(fname);
        if let Err(fd) = input {
            // Cannot open at all.
            MSG_SCROLL.set(msg_save);
            return Err(cannot_open(fd, fname, perm, how, silent, excmd, &watch));
        }
    }

    // Only set the 'ro' flag for readonly files the first time they
    // are loaded. Help files always get readonly mode.
    if (check_readonly && file_readonly) || Buf::current().b_help {
        Buf::current().b_p_ro = c_int::from(true);
    }

    if how.set_options {
        // Don't change 'eol' when reading from a buffer: it was
        // already set correctly when stdin was read.
        if !how.buffer {
            Buf::current().b_p_eof = c_int::from(false);
            Buf::current().b_start_eof = c_int::from(false);
            Buf::current().b_p_eol = c_int::from(true);
            Buf::current().b_start_eol = c_int::from(true);
        }
        Buf::current().b_p_bomb = c_int::from(false);
        Buf::current().b_start_bomb = c_int::from(false);
    }

    // Create a swap file now, so that other Nvims are warned that we
    // are editing this file. Not for a "nofile" or "nowrite" buffer.
    if !buf_is_dontwrite(current_buf()) {
        // On an error the file is closed with `input`.
        make_swap_file(how, swap_mode, file_info.as_ref(), &watch)?;
    }

    // If "Quit" was selected at the ATTENTION dialog, don't load it.
    if SWAP_EXISTS_ACTION.get() == SEA_QUIT {
        return Err(REFUSED); // the file is closed with `input`
    }

    NO_WAIT_RETURN.set(NO_WAIT_RETURN.get() + 1); // don't wait for return yet
    let orig_start = mark_insert_point(from);

    if !how.buffer {
        run_read_pre(
            fname, sfname, excmd, how, msg_save, &watch, &mut input, orig_start,
        )?;
    }
    // Read only now: the autocommands may have changed 'fileformats'.
    let guess = FormatGuess::from_ffs();

    // Standard input is taken over only now, when it will be read: every
    // early return above leaves it open.
    let input = if how.buffer {
        None
    } else if how.stdin {
        Some(take_stdin_for_read())
    } else {
        input.ok()
    };
    Ok(Opened {
        fname,
        sfname,
        input,
        perm,
        guess,
    })
}

/// Let `BufReadCmd` (for a new file) or `FileReadCmd` do the reading
/// instead. Answers what `readfile` should, when one did or when there is
/// nothing to read (`READ_NOFILE`); `None` to go on and read the file.
fn run_read_cmd(
    sfname: Option<&CStr>,
    from: LineNr,
    excmd: Option<&mut ExArg>,
    how: &How,
) -> Option<Unread> {
    let orig_start = mark_insert_point(from);
    let intercepted = if how.newfile {
        read_autocmd(AutoEvent::BufReadCmd, sfname, excmd, false)
    } else {
        read_autocmd(AutoEvent::FileReadCmd, sfname, excmd, true)
    };
    if intercepted {
        if aborting() {
            return Some(Err(Failed.into()));
        }
        // The BufReadCmd code usually uses ":read" to get the text and
        // perhaps ":file" to change the buffer name, but this should work
        // like ":edit", so reset BufFlags::NOTEDITED and let ":write"
        // overwrite the file.
        if how.newfile {
            Buf::current().b_flags.clear(BufFlags::NOTEDITED);
        }
        return Some(Ok(Loaded::Read));
    }

    Buf::current().b_op_start = orig_start;

    // NOTDONE rather than FAIL, so that BufEnter can still be triggered
    // and other operations don't fail.
    how.nofile.then_some(Ok(Loaded::Skipped))
}

/// Refuse a name that is too long, ends in a path separator, or names
/// something other than a file, with a note saying so. Answers the file's
/// mode bits (a libuv error when it is missing, 0 for stdin).
fn check_readable(
    fname: Option<&CStr>,
    how: &How,
    silent: bool,
    msg_save: c_int,
) -> Result<c_int, Unread> {
    let msg_is_a_directory = c"is a directory";
    let refuse = |answer: Unread| {
        msg_end();
        MSG_SCROLL.set(msg_save);
        Err(answer)
    };

    if let Some(name) = fname.filter(|name| !name.is_empty()) {
        let fnamelen = name.count_bytes();
        // If the name is too long we might crash further on, quit
        // here.
        if fnamelen >= MAXPATHL as usize {
            filemess_note(fname, c"Illegal file name");
            return refuse(REFUSED);
        }
        // A name ending in a path separator can't be opened. Check it
        // here, because reading may actually work but then creating
        // the swap file would destroy it.
        if after_pathsep(name.to_bytes(), fnamelen) {
            if !silent {
                filemess_note(fname, msg_is_a_directory);
            }
            return refuse(Ok(Loaded::Skipped));
        }
    }

    let perm = match fname {
        Some(name) if !how.stdin => getperm(name),
        _ => 0,
    };

    if !how.stdin && !how.buffer && !how.fifo {
        let kind = u64::from(perm.cast_unsigned()) & S_IFMT;
        if perm >= 0 && kind != 0o100000 && kind != 0o10000 && kind != 0o140000 {
            // On Unix it is possible to read a directory, so check for
            // one before opening it.
            if kind == 0o40000 {
                if !silent {
                    filemess_note(fname, msg_is_a_directory);
                }
                return refuse(Ok(Loaded::Skipped));
            }
            filemess_note(fname, c"is not a file");
            return refuse(REFUSED);
        }
    }
    Ok(perm)
}

/// Remember the time of the file being edited, and reset the "new file"
/// flag (set again when the file turns out not to exist). Answers what the
/// file's metadata said, and the protection bits for its swap file (-1
/// when there is no file to take them from).
fn remember_file_time(fname: Option<&CStr>) -> (Option<FileInfo>, c_int) {
    let mut swap_mode = -1;
    let file_info = fname.and_then(crate::os::fs::file_info);
    if let Some(info) = &file_info {
        buf_store_file_info(Buf::current(), info);
        Buf::current().b_mtime_read = Buf::current().b_mtime;
        Buf::current().b_mtime_read_ns = Buf::current().b_mtime_ns;
        // Use the protection bits of the original file for the swap file,
        // so that others can read the name of the edited file from it, but
        // only if they can read the file itself. Remove the write and
        // execute bits for group and others (they must not write the swap
        // file), and add read and write for the user, or we might not be
        // able to write it ourselves. The bits are set after the swap file
        // is created.
        swap_mode = (info.mode() & 0o644) | 0o600;
    } else {
        Buf::current().b_mtime = 0;
        Buf::current().b_mtime_ns = 0;
        Buf::current().b_mtime_read = 0;
        Buf::current().b_mtime_read_ns = 0;
        Buf::current().b_orig_size = 0;
        Buf::current().b_orig_mode = 0;
    }
    Buf::current()
        .b_flags
        .clear(BufFlags::NEW | BufFlags::NEW_W);
    (file_info, swap_mode)
}

/// The file would not open (libuv's `fd`): a new file when it does not
/// exist and a new buffer is being edited, otherwise a note on the screen.
fn cannot_open(
    fd: c_int,
    fname: Option<&CStr>,
    perm: c_int,
    how: &How,
    silent: bool,
    mut excmd: Option<&mut ExArg>,
    watch: &BufferWatch,
) -> Unread {
    let sfname = fname;
    if !how.newfile {
        return REFUSED;
    }
    if perm != UV_ENOENT {
        // libuv only returns -errno, and on Windows open() does not set
        // EOVERFLOW.
        let note = if fd == UV_EFBIG || fd == -EOVERFLOW {
            c"[File too big]"
        } else {
            c"[Permission Denied]"
        };
        Buf::current().file_message(sfname, translate(note));
        Buf::current().b_p_ro = c_int::from(true); // must use "w!" now
        return REFUSED;
    }

    // The file does not exist. Set the 'new-file' flag, so that a ":w"
    // complains if someone else created it since.
    Buf::current().b_flags |= BufFlags::NEW;

    // Create a swap file now, so that other Nvims are warned that we are
    // editing this file. Not for a "nofile" or "nowrite" buffer type.
    if !buf_is_dontwrite(current_buf()) {
        check_need_swap(how.newfile);
        // The SwapExists autocommand may mess things up.
        if watch.changed() {
            return Err(ReadError::AutocmdsChangedBuffer);
        }
    }
    if !silent {
        let note = if dir_of_file_exists(fname.unwrap_or_default().to_bytes()) {
            c"[New]"
        } else {
            c"[New DIRECTORY]"
        };
        Buf::current().file_message(sfname, translate(note));
    }
    // Even though this is a new file, it might have been edited before and
    // deleted. Get the old marks.
    check_marks_read();
    // Set the forced 'fileencoding'.
    if let Some(command) = excmd.as_deref_mut() {
        set_forced_fenc(command);
    }
    apply_autocmds_exarg(
        AutoEvent::BufNewFile,
        sfname.map(XString::from_cstr),
        sfname.map(XString::from_cstr),
        false,
        Buf::current_or_none(),
        excmd,
    );
    // Remember the current fileformat.
    save_file_ff(Buf::current());

    if aborting() {
        // Autocommands may abort script processing.
        return REFUSED;
    }
    Ok(Loaded::Read) // a new file is not an error
}

/// Create the swap file, and give it the protection bits `swap_mode` (when
/// positive) taken from the file in `file_info`.
fn make_swap_file(
    how: &How,
    mut swap_mode: c_int,
    file_info: Option<&FileInfo>,
    watch: &BufferWatch,
) -> Result<(), Unread> {
    check_need_swap(how.newfile);
    if !how.stdin && watch.changed() {
        return Err(Err(ReadError::AutocmdsChangedBuffer));
    }
    // Set the swap file's protection bits now that it exists.
    if swap_mode > 0
        && let Some(swap_fname) = Buf::current().swap_file_name()
    {
        // If the group-read bit is set but not the world-read bit, the
        // group must equal the group of the original file. If we can't
        // make that happen, reset the group-read bit; that avoids making
        // the swap file readable to more users than the file itself when
        // the user's primary group is too permissive.
        if swap_mode & 0o44 == 0o40
            && let Some(info) = file_info
            && crate::os::fs::file_info(&swap_fname)
                .is_some_and(|swap| info.stat.st_gid != swap.stat.st_gid)
            && !Buf::current().set_swap_file_group(info.gid().unwrap_or(uv_gid_t::MAX))
        {
            swap_mode &= 0o600;
        }
        let _ = os_setperm(&swap_fname, swap_mode);
    }
    Ok(())
}

/// Run the `*ReadPre` autocommands, with the file closed: they may want to
/// change it before it is read. Then make sure they did not change the
/// current buffer or its name, and re-open the file into `input`. The '[
/// mark goes back to `orig_start`.
#[allow(clippy::too_many_arguments)]
fn run_read_pre(
    fname: Option<&CStr>,
    sfname: Option<&CStr>,
    mut excmd: Option<&mut ExArg>,
    how: &How,
    msg_save: c_int,
    watch: &BufferWatch,
    input: &mut Result<File, c_int>,
    orig_start: Pos,
) -> Result<(), Unread> {
    let m = MSG_SCROLL.get();
    let n = MSG_SCROLLED.get();

    if !how.stdin {
        *input = Err(0); // ignore errors
    }

    // The output from the autocommands should neither overwrite anything
    // nor be overwritten: set msg_scroll, and restore it if no output was
    // done.
    MSG_SCROLL.set(c_int::from(true));
    if how.filtering {
        read_autocmd(
            AutoEvent::FilterReadPre,
            sfname,
            excmd.as_deref_mut(),
            false,
        );
    } else if how.stdin {
        read_autocmd(AutoEvent::StdinReadPre, sfname, excmd.as_deref_mut(), false);
    } else if how.newfile {
        read_autocmd(AutoEvent::BufReadPre, sfname, excmd.as_deref_mut(), false);
    } else {
        read_autocmd(AutoEvent::FileReadPre, sfname, excmd, true);
    }
    Buf::current().b_op_start = orig_start;

    if MSG_SCROLLED.get() == n {
        MSG_SCROLL.set(m);
    }

    let give_up = |answer: Unread| {
        NO_WAIT_RETURN.set(NO_WAIT_RETURN.get() - 1);
        MSG_SCROLL.set(msg_save);
        Buf::current().b_p_ro = c_int::from(true); // must use "w!" now
        Err(answer)
    };
    if aborting() {
        // Autocommands may abort script processing.
        return give_up(REFUSED);
    }

    // Don't allow the autocommands to change the current buffer, and don't
    // allow them to change its name either (a `:cd`, for instance) if that
    // invalidates fname or sfname. Try to re-open the file.
    if !how.stdin && watch.changed() {
        return give_up(Err(ReadError::ReadPreChangedBuffer));
    }
    if !how.stdin {
        *input = open_named(fname);
        if input.is_err() {
            return give_up(Err(ReadError::ReadPreUnreadable));
        }
    }
    Ok(())
}

/// [`open_for_read`] of a name that may be missing, which libuv refuses as
/// `EINVAL`.
fn open_named(name: Option<&CStr>) -> Result<File, c_int> {
    name.map_or(Err(crate::os::uv_error::UV_EINVAL), open_for_read)
}
