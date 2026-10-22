//! Reading files into buffers, and the file-level helpers around it.
//!
//! `readfile_named` is the reader (`read`, `open`, `convert`, `lines`,
//! `report`); `timestamp` notices a file changing underneath its buffer,
//! `names` derives and moves file names, `tempfile` owns the temporary
//! directory, and `stream` holds `Fgets`, the line reader over any
//! `BufRead`.

#![forbid(unsafe_code)]

pub(crate) mod state;
use crate::autocmd::state::autocmd_busy;
use crate::autocmd::{
    apply_autocmds_exarg, aucmd_prepbuf, aucmd_restbuf, augroup_exists_of, do_doautocmd,
    fire_autocmds, fire_autocmds_for,
};
use crate::buffer::state::swap_exists_action;
use crate::buffer::{
    BufFlags, buf_contents_changed, buf_is_dontwrite, buf_is_empty, buf_is_nofilename,
    buf_is_normal, buflist_new, current_buf, do_modelines, setfname, wipe_buffer,
};
use crate::buffer_updates::buf_updates_unload;
use crate::change::{appended_lines_mark, save_file_ff, unchanged};
use crate::cursor::{check_cursor, check_cursor_lnum};
use crate::diff::diff_invalidate;
use crate::drawscreen::state::{redraw_cmdline, redraw_tabline};
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later, status_redraw_all};
use crate::edit::beginline;
use crate::eval::vars::{eval_charconvert, set_vim_var_string, vim_var_bytes};
use crate::ex_docmd::state::{ex_no_reprint, global_busy};
use crate::ex_eval::aborting;
use crate::fileio::state::{did_check_timestamps, need_check_timestamps, no_check_timestamps};
use crate::fold::{fold_update_all, foldmethod_is_manual};
use crate::getchar::state::got_int;
use crate::getchar::stuff_empty;
use crate::global_cell::GlobalCell;
use crate::guard::allbuf_lock;
use crate::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_WRN};
use crate::mbyte::{canon_props, canonize, utf_byte2len, utf_char2len};
use crate::memline::{check_need_swap, ml_append_terminated, ml_delete, ml_open};
use crate::memory::{XString, try_grow_zeroed};
use crate::message::e_interr;
use crate::message::state::{
    emsg_silent, in_assert_fails, keep_msg, msg_col, msg_listdo_overwrite, msg_scroll,
    msg_scrolled, msg_scrolled_ign, msg_silent, need_fileinfo, need_wait_return, no_wait_return,
};
use crate::message::{
    do_dialog, emsg, fit_head, msg, msg_check_for_delay, msg_clr_eos, msg_delay, msg_display_bytes,
    msg_end, msg_fit_head, msg_progress, msg_putchar, msg_start, msg_str, msg_str_hl, set_keep_msg,
};
use crate::message_fmt::msg_bytes;
use crate::r#move::update_topline;
use crate::option::vars::{P_FFS, p_ar, p_enc, p_fencs, p_ffs, p_fic, p_ur, p_verbose};
use crate::option::{
    default_fileformat, get_fileformat, get_fileformat_force, set_fileformat, set_option_direct,
    set_options_bin, shortmess,
};
use crate::options::kOptFileencoding;
use crate::optionstr::OptString;
use crate::os::cshim::{gettext, ngettext};
use crate::os::env::home_replace_in;
use crate::os::fs::{
    dir_exists, mark_stdin_cloexec, os_closedir, os_copy, os_file_is_writable, os_file_owned,
    os_free_acl, os_get_acl, os_isrealdir, os_mkdir, os_remove, os_rename, os_rmdir, os_scandir,
    os_scandir_next, os_set_acl, os_setperm, path_exists, reopen_stdin_on_stderr,
};
use crate::os::input::os_breakcheck;
use crate::path::{
    add_pathsep, after_pathsep, dir_of_file_exists, path_fnamecmp, path_is_absolute,
    path_shorten_fname, path_tail, path_with_url, vim_full_name,
};
use crate::pos::MAXLNUM;
use crate::regexp::vim_regexec;
use crate::sha256::Sha256;
use crate::shada::check_marks_read;
use crate::startup::{exiting, readonlymode, recoverymode, stdin_fd, vim_ignored};
use crate::state::mode::{State, exmode_active, restart_edit};
use crate::state::{MODE_CMDLINE, MODE_NORMAL_BUSY};
use crate::strings::{FmtArg, formatted};
use crate::types::AutoEvent;
use crate::types::CAR;
use crate::types::NL;
use crate::types::ui::kUIMessages;
use crate::types::{
    AcoSave, BlnFlags, Directory, ExArg, Failed, FileInfo, FileOffset, IOSIZE, LineNr, OptInt,
    OptVal, OptionSetFlags, RegMatch, ScriptId, ShmFlag, int64_t, uv_gid_t,
};
use crate::ui::{ui_flush, ui_has};
use crate::undo::{
    buf_is_changed, u_clearallandblockfree, u_clearline, u_compute_hash, u_find_first_changed,
    u_read_undo, u_savecommon, u_sync, u_unchanged, u_write_undo,
};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};
use std::fs::File;
use std::io::{ErrorKind, Seek, SeekFrom};

// The carve of the transpiled module; see each child's docs.
mod convert;
pub use self::convert::*;
mod lines;
pub(crate) use self::lines::*;
mod open;
pub(crate) use self::open::*;
mod read;
pub(crate) use self::read::*;
mod report;
pub(crate) use self::report::*;
mod timestamp;
pub use self::timestamp::*;
mod names;
pub use self::names::*;
mod tempfile;
pub use self::tempfile::*;
mod stream;
pub(crate) use self::stream::*;
pub const BLN_DUMMY: BlnFlags = 4;
pub const BAD_REPLACE: c_int = b'?' as c_int;
pub const BAD_KEEP: c_int = -1;
pub const BAD_DROP: c_int = -2;
pub const FORCE_BIN: c_int = 1;
pub const FORCE_NOBIN: c_int = 2;
pub const ENC_UCSBOM: &CStr = c"ucs-bom";
pub const EOL_UNKNOWN: c_int = -1;
pub const EOL_UNIX: c_int = 0;
pub const EOL_DOS: c_int = 1;
pub const EOL_MAC: c_int = 2;
const VIM_WARNING: c_int = 2;
const SEA_QUIT: c_int = 2;
const EOVERFLOW: c_int = ::libc::EOVERFLOW;
/// The file-type bits of a mode.
const S_IFMT: c_int = 0o170000;

crate::flag_set! {
    /// What `readfile_named` is asked to do: upstream's `READ_*` bits.
    pub struct ReadFlags;

    /// Starting to edit a new buffer.
    const NEW = 0x01;
    /// Reading filter output.
    const FILTER = 0x02;
    /// Read from stdin.
    const STDIN = 0x04;
    /// Read from `curbuf`, to convert what stdin gave us.
    const BUFFER = 0x08;
    /// Read into a dummy buffer, to see whether the file changed.
    const DUMMY = 0x10;
    /// Don't clear the undo info or read it from a file.
    const KEEP_UNDO = 0x20;
    /// Read from a fifo or socket.
    const FIFO = 0x40;
    /// Don't trigger `BufWinEnter` (`open_buffer`'s, carried through).
    const NOWINENTER = 0x80;
    /// Don't read a file, only trigger `BufReadCmd`.
    const NOFILE = 0x100;
}

/// What a read of a file into a buffer came to.
///
/// `readfile`'s `NOTDONE` as a value rather than a status: the file was not
/// read and that is *not* an error -- a `BufReadCmd` autocommand did the
/// reading itself, or the name is a directory and the caller still wants
/// `BufEnter` to fire. Callers that only ask "did it work" read the
/// `Result`; the two that ask "were the lines actually read" match on this,
/// which is the distinction `retval == OK` versus `retval != FAIL` carried
/// in the C and which a bare `Result` would have quietly lost.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Loaded {
    /// The file was read into the buffer.
    Read,
    /// Nothing was read, and nothing is wrong. The C's `NOTDONE`.
    Skipped,
}

/// One of the buffer-lifecycle autocommands this file fires about the current
/// buffer: `apply_autocmds(event, NULL, NULL, false, curbuf)`.
fn autocmd_for_curbuf(event: AutoEvent) {
    fire_autocmds(event, false, Buf::current_or_none());
}
static E_AUCHANGEDBUF: &CStr = c"E812: Autocommands changed buffer or buffer name";
/// The top bit of every byte of a word: a word ANDed with it is zero when
/// all eight bytes are ASCII.
const NONASCII_MASK: u64 = u64::MAX / 0xff * 0x80;
/// Report which file is being read or written.
///
/// `note` is appended; an empty one means the message is progress on a
/// write that is still running.
pub fn filemess(buffer: Buf, name: Option<&CStr>, note: &CStr) {
    // The report. Upstream builds it in `IObuff` and then calls
    // `msg_progress`/`msg_display`, which write it again.
    let prev_msg_col = msg_col.get();
    if msg_silent.get() != 0 {
        return;
    }
    let mut report = Report::quoted_fname(buffer, name, IOSIZE as usize - 100);
    // Avoid an over-long translation causing trouble.
    report.push(note.to_bytes());

    // For the first message we may have to start a new line. Further ones
    // overwrite the previous one; reset `msg_scroll` before calling this.
    let msg_scroll_save = msg_scroll.get();
    if shortmess(ShmFlag::OVERALL)
        && msg_listdo_overwrite.get() == 0
        && !exiting.get()
        && p_verbose() == 0
    {
        msg_scroll.set(false as c_int);
    }
    if msg_scroll.get() == 0 {
        // Wait a bit when overwriting an error message.
        msg_check_for_delay(false);
    }
    msg_start();
    if prev_msg_col != 0 && msg_col.get() == 0 {
        msg_putchar(c_int::from(b'\r')); // overwrite any previous message
    }
    msg_scroll.set(msg_scroll_save);
    msg_scrolled_ign.set(true);
    let text = report.as_cstr();
    if note.is_empty() {
        msg_progress(text, c"bufwrite", c"running", 0, false, true);
    } else {
        // May truncate the message to avoid a hit-return prompt.
        msg_display_bytes(&fit_head(text.to_bytes(), false), 0, false);
    }
    msg_clr_eos();
    msg_scrolled_ign.set(false);
}

/// From the buffer's line count before this read and the bytes read since,
/// estimate the line number we are now on. Used for error messages.
///
/// @param linecnt  the line count before `text` was read
pub(crate) fn readfile_linenr(linecnt: LineNr, text: &[u8]) -> LineNr {
    let newlines = text.iter().filter(|&&b| b == b'\n').count();
    Buf::current().b_ml.ml_line_count - linecnt + 1 + newlines as LineNr
}

/// Set the name of the current buffer, for a `:r` or `:w` command with a file
/// name given for a buffer that has none.
///
/// The names are the caller's, held across the autocommands this fires.
pub fn set_rw_fname(fname: Option<&CStr>, sfname: Option<&CStr>) -> Result<(), Failed> {
    let buf = Buf::current_raw();

    // It's like the unnamed buffer is deleted...
    if Buf::current().b_p_bl != 0 {
        autocmd_for_curbuf(AutoEvent::BufDelete);
    }
    autocmd_for_curbuf(AutoEvent::BufWipeout);
    if aborting() {
        // Autocommands may abort script processing.
        return Err(Failed);
    }
    if Buf::current_raw() != buf {
        // We are in another buffer now, don't do the renaming.
        emsg(gettext(E_AUCHANGEDBUF));
        return Err(Failed);
    }

    if setfname(Buf::current(), fname, sfname, false).is_ok() {
        Buf::current().b_flags |= BufFlags::NOTEDITED;
    }

    // ...and a new named one is created.
    autocmd_for_curbuf(AutoEvent::BufNew);
    if Buf::current().b_p_bl != 0 {
        autocmd_for_curbuf(AutoEvent::BufAdd);
    }
    if aborting() {
        return Err(Failed);
    }

    // Do filetype detection now if 'filetype' is empty.
    if Buf::current().b_p_ft.first_byte() == 0 {
        if augroup_exists_of(c"filetypedetect") {
            let _ = do_doautocmd(c"filetypedetect BufRead", false, None);
        }
        do_modelines(OptionSetFlags::NONE);
    }
    Ok(())
}

/// Fill `excmd` so that `'fileencoding'`, `'fileformat'` and `'binary'` are
/// forced to what buffer `buffer` already has. Used when calling `readfile` to
/// re-read a buffer that is already open.
pub fn prep_exarg(excmd: &mut ExArg, buffer: Buf) {
    let mut line = b"e ++enc=".to_vec();
    line.extend_from_slice(&buffer.b_p_fenc.get());
    excmd.set_cmd_text(&line);
    // Where the encoding name starts in that command.
    excmd.force_enc = 8;
    excmd.bad_char = buffer.b_bad_char;
    excmd.force_ff = c_int::from(buffer.b_p_ff.first_byte());
    excmd.force_bin = if buffer.b_p_bin != 0 {
        FORCE_BIN
    } else {
        FORCE_NOBIN
    };
    excmd.read_edit = false;
    excmd.forceit = false;
}

/// Set the default or forced `'fileformat'` and `'binary'`.
///
pub fn set_file_options(set_options: bool, excmd: Option<&mut ExArg>) {
    let forced = excmd.as_deref();
    // Set the default 'fileformat'.
    if set_options {
        if forced.is_some_and(|command| command.force_ff != 0) {
            set_fileformat(
                get_fileformat_force(Buf::current(), forced),
                OptionSetFlags::LOCAL,
            );
        } else if P_FFS.first_byte() != 0 {
            set_fileformat(default_fileformat(), OptionSetFlags::LOCAL);
        }
    }

    // Set or reset 'binary'.
    if let Some(forced_bin) = forced.map(|command| command.force_bin).filter(|&b| b != 0) {
        let oldval = Buf::current().b_p_bin;
        Buf::current().b_p_bin = (forced_bin == FORCE_BIN) as c_int;
        let bin = Buf::current().b_p_bin != 0;
        set_options_bin(oldval != 0, bin, OptionSetFlags::LOCAL);
    }
}

/// Set the forced `'fileencoding'` from a `++enc=` argument.
pub fn set_forced_fenc(excmd: &mut ExArg) {
    if excmd.force_enc == 0 {
        return;
    }
    let fenc = canonize(
        excmd
            .line
            .rest_of(excmd.line.cmd + excmd.force_enc as usize),
    );
    set_option_direct(
        kOptFileencoding,
        OptVal::from_cstr(fenc.as_cstr()),
        OptionSetFlags::LOCAL,
        0 as ScriptId,
    );
}
