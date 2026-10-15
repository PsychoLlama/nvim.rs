//! A buffer's name -- setting it, comparing it, and the alternate file.
//!
//! [`setfname`] gives a buffer its file name, which means resolving it to a
//! full path, computing the file id used to recognise the same file under
//! another name, and telling the alternate-file and argument lists about it.
//! [`otherfile`] and [`otherfile_buf`] are the comparison, [`setaltfname`]
//! and [`buflist_add`] maintain the `#` entry, and [`buflist_name_nr`] is the
//! `:buffers`-style lookup by number.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::*;
use crate::arglist::check_arg_idx;
use crate::drawscreen::status_redraw_all;
use crate::ex_docmd::cmdmod_has;
use crate::mark::fmarks_check_names;
use crate::memline::{ml_setname, ml_timestamp};
use crate::memory::XString;
use crate::message::e_noalt;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::os::fs::file_id;
use crate::path::{fix_fname, path_fnamecmp};
use crate::types::{CmdModFlags, Failed, FileID, LineNr};
use crate::winlayer::{Buf, Win, tab_windows};

fn current_win() -> Win {
    Win::current()
}

// ---------------------------------------------------------------------------
// Looking a name up

/// The buffer `fnum` names, when it has a name, and the line remembered for
/// it.
pub fn buflist_name_nr(fnum: c_int) -> Option<(Buf, LineNr)> {
    let buf = find_buf(fnum)?;
    if buf.name.is_unnamed() {
        return None;
    }
    Some((buf, buflist_findlnum(buf)))
}

// ---------------------------------------------------------------------------
// Setting one

/// Give `buffer` the file name `ffname_arg` (short form `sfname_arg`).
///
/// Fails, with `message`, when another *loaded* buffer already has the name;
/// an unloaded one is wiped to make room.
///
/// `None` for either name is upstream's NULL: no name at all, which is not
/// the same as the empty one.
pub fn setfname(
    buffer: Buf,
    ffname_arg: Option<&CStr>,
    sfname_arg: Option<&CStr>,
    message: bool,
) -> Result<(), Failed> {
    let mut b = buffer;
    let mut file_id = None;

    match fname_expand(ffname_arg.filter(|name| !name.is_empty()), sfname_arg) {
        None => {
            // Removing the name. Upstream's three lines here are the
            // `b_sfname != b_ffname` dance; the name owns its blocks now.
            b.name.clear();
        }
        Some((ffname, sfname)) => {
            // If the file name is already used in another buffer:
            // - if the buffer is loaded, fail
            // - if the buffer is not loaded, delete it from the list
            file_id = crate::os::fs::file_id(ffname.as_cstr());
            let obuf = if b.b_flags.has(BufFlags::DUMMY) {
                None
            } else {
                buflist_findname_file_id(ffname.as_cstr(), file_id.as_ref())
            };
            if let Some(o) = obuf.filter(|&o| o != buffer) {
                // During startup a window may use a buffer that is not
                // loaded yet.
                let in_use = tab_windows().any(|win| win.w_buffer == o);
                if o.b_ml.ml_mfp.is_some() || in_use {
                    // It is loaded or used in a window: fail.
                    if message {
                        emsg(gettext(c"E95: Buffer with this name already exists"));
                    }
                    return Err(Failed);
                }
                // Delete it from the list: a live, unloaded buffer shown in
                // no window.
                close_buffer(None, o, DOBUF_WIPE.cast_signed(), false, false);
            }
            b.name.set(ffname, Some(sfname));
        }
    }
    b.file_id_valid = file_id.is_some();
    if let Some(file_id) = file_id {
        b.file_id = file_id;
    }

    buf_name_changed(buffer);
    Ok(())
}

/// A crude way of changing a buffer's name; use with care. The name is
/// relative to the current directory: it becomes the short name, and its
/// expansion the full one.
pub(crate) fn buf_set_name(fnum: c_int, name: &CStr) {
    let Some(mut b) = find_buf(fnum) else {
        return;
    };
    if let Some((ffname, sfname)) = fname_expand(Some(name), None) {
        b.name.set(ffname, Some(sfname));
    }
}

/// What has to happen once a buffer's name has changed.
pub fn buf_name_changed(b: Buf) {
    if b.b_ml.ml_mfp.is_some() {
        // The swap file's name follows the buffer's.
        ml_setname(b);
    }
    let cur = current_win();
    if cur.w_buffer == b {
        // Check the file name against the argument list.
        // SAFETY: a live window.
        check_arg_idx(cur);
    }
    maketitle();
    status_redraw_all();
    fmarks_check_names(b);
    ml_timestamp(b);
}

// ---------------------------------------------------------------------------
// The alternate file

/// Set the alternate file name for the current window.
///
/// `None` for either name is upstream's NULL.
pub fn setaltfname(ffname: Option<&CStr>, sfname: Option<&CStr>, lnum: LineNr) -> Option<Buf> {
    // Create a buffer; 'buflisted' is not set if it is a new one.
    let buf = buflist_new(ffname, sfname, lnum, 0);
    if let Some(buf) = buf
        && !cmdmod_has(CmdModFlags::KEEPALT)
    {
        current_win().w_alt_fnum = buf.handle as c_int;
    }
    buf
}

/// The alternate file's name (the buffer's own storage, so read before
/// anything runs), null when there is none.
pub fn getaltfname(errmsg: bool) -> *mut c_char {
    match buflist_name_nr(0) {
        Some((alt, _)) => alt.name.shown_ptr(),
        None => {
            if errmsg {
                emsg(gettext(e_noalt));
            }
            ptr::null_mut()
        }
    }
}

/// Record the alternate cursor position for the current buffer in `win`,
/// saving its window-local options too.
pub fn buflist_altfpos(win: Win) {
    let (lnum, col) = (win.w_cursor.lnum, win.w_cursor.col);
    buflist_setfpos(Buf::current(), Some(win), lnum, col, true);
}

// ---------------------------------------------------------------------------
// Is this the same file?

/// Whether `ffname` (a full path) names a different file from the current
/// buffer's. The name is only read.
pub fn otherfile(ffname: &CStr) -> bool {
    otherfile_buf(Buf::current(), ffname, None)
}

/// Whether `ffname` (a full path) names a different file from `buffer`'s.
///
/// `file_id` is the caller's already-computed file id for `ffname` (itself
/// `None` when the file does not exist), or `None` to have it looked up here.
pub(crate) fn otherfile_buf(b: Buf, ffname: &CStr, file_id: Option<Option<&FileID>>) -> bool {
    let Some(full) = b.name.full().filter(|_| !ffname.is_empty()) else {
        return true;
    };
    if path_fnamecmp(ffname.to_bytes(), full.to_bytes()) == 0 {
        return false;
    }

    let own;
    let file_id = match file_id {
        Some(known) => known,
        None => {
            own = crate::os::fs::file_id(ffname);
            own.as_ref()
        }
    };
    let Some(file_id) = file_id else {
        return true;
    };

    if b.file_id_valid && b.file_id == *file_id {
        buf_set_file_id(b);
        if b.file_id_valid && b.file_id == *file_id {
            return false;
        }
    }
    true
}

/// Record the file id of `buffer`'s file, for recognising it under another name.
pub fn buf_set_file_id(mut b: Buf) {
    let found = b.name.shown().and_then(file_id);
    b.file_id_valid = found.is_some();
    if let Some(found) = found {
        b.file_id = found;
    }
}

/// The full form of `ffname` and the short form a buffer keeps beside it:
/// `sfname`, or `ffname` as given when there is no short name. `None` when
/// no name was given at all.
pub fn fname_expand(ffname: Option<&CStr>, sfname: Option<&CStr>) -> Option<(XString, XString)> {
    // No file name given, nothing to do.
    let ffname = ffname?;
    // No short file name given, use ffname.
    let sfname = XString::from_cstr(sfname.unwrap_or(ffname));
    Some((fix_fname(ffname), sfname))
}
