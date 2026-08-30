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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::arglist::check_arg_idx;
use crate::drawscreen::status_redraw_all;
use crate::ex_docmd::cmdmod_has;
use crate::main::e_noalt;
use crate::mark::fmarks_check_names;
use crate::memline::{ml_setname, ml_timestamp};
use crate::memory::{xfree, xstrdup};
use crate::message::emsg_ptr;
use crate::os::cshim::gettext_ptr;
use crate::os::fs::{os_fileid, os_fileid_equal};
use crate::path::{fix_fname, path_fnamecmp};
use crate::types::{CmdModFlags, Failed, FileID, linenr_T};
use crate::winlayer::{Buf, Win, tab_windows};

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// `_()`.
fn tr(msg: &CStr) -> *mut c_char {
    tr_raw(msg.as_ptr())
}

/// `_()` over a pointer, for the message statics `main.rs` holds as byte
/// arrays.
fn tr_raw(msg: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated literal or message static.
    unsafe { gettext_ptr(msg).as_ptr().cast_mut() }
}

fn err(msg: *mut c_char) {
    // SAFETY: a NUL-terminated message.
    unsafe { emsg_ptr(msg) };
}

/// `XFREE_CLEAR`.
fn xfree_clear(slot: &mut *mut c_char) {
    // SAFETY: an owned allocation or null; `xfree` accepts both.
    unsafe { xfree((*slot).cast::<c_void>()) };
    *slot = ptr::null_mut();
}

fn free(p: *mut c_char) {
    // SAFETY: an owned allocation or null.
    unsafe { xfree(p.cast::<c_void>()) };
}

fn dup(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string; upstream passes the short name, which
    // `fname_expand` has just made non-null.
    unsafe { xstrdup(p) }
}

/// The file id of `fname`, and whether the file exists at all.
fn file_id_of(fname: *const c_char) -> (FileID, bool) {
    let mut file_id = FileID {
        inode: 0,
        device_id: 0,
    };
    // SAFETY: a NUL-terminated path, and a local to fill in.
    let valid = unsafe { os_fileid(fname, &raw mut file_id) };
    (file_id, valid)
}

fn same_file_id(buf: &mut Buf, file_id: *mut FileID) -> bool {
    // SAFETY: the buffer's own file id, and the caller's, both live.
    buf.file_id_valid && unsafe { os_fileid_equal(&raw mut buf.file_id, file_id) }
}

/// Whether a name slot holds nothing: null or the empty string.
fn is_empty_name(p: *const c_char) -> bool {
    // SAFETY: null or a NUL-terminated name.
    p.is_null() || unsafe { *p } == 0
}

fn names_equal(a: *const c_char, b: *const c_char) -> bool {
    // SAFETY: two NUL-terminated paths, both non-null by the tests above.
    unsafe { path_fnamecmp(a, b) == 0 }
}

fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

// ---------------------------------------------------------------------------
// Looking a name up

/// The file name and remembered line number of buffer `fnum`.
pub unsafe fn buflist_name_nr(
    fnum: c_int,
    fname: *mut *mut c_char,
    lnum: *mut linenr_T,
) -> Result<(), Failed> {
    let Some(mut buf) = find_buf(fnum) else {
        return Err(Failed);
    };
    if buf.b_fname.is_null() {
        return Err(Failed);
    }
    // SAFETY: the caller's promise -- two out-parameters to fill in.
    let (fname, lnum) = unsafe { (&mut *fname, &mut *lnum) };
    *fname = buf.b_fname;
    // SAFETY: a live buffer.
    *lnum = unsafe { buflist_findlnum(buf) };
    Ok(())
}

// ---------------------------------------------------------------------------
// Setting one

/// Give `buf` the file name `ffname_arg` (short form `sfname_arg`).
///
/// Fails, with `message`, when another *loaded* buffer already has the name;
/// an unloaded one is wiped to make room.
pub unsafe fn setfname(
    buf: Buf,
    ffname_arg: *mut c_char,
    sfname_arg: *mut c_char,
    message: bool,
) -> Result<(), Failed> {
    let mut b = buf;
    let mut ffname = ffname_arg;
    let mut sfname = sfname_arg;
    let mut file_id = FileID {
        inode: 0,
        device_id: 0,
    };
    let mut file_id_valid = false;

    if is_empty_name(ffname) {
        // Removing the name.
        if b.b_sfname != b.b_ffname {
            xfree_clear(&mut b.b_sfname);
        } else {
            b.b_sfname = ptr::null_mut();
        }
        xfree_clear(&mut b.b_ffname);
    } else {
        // SAFETY: two locals holding a name each.
        unsafe { fname_expand(buf, &raw mut ffname, &raw mut sfname) };
        if ffname.is_null() {
            // Out of memory.
            return Err(Failed);
        }

        // If the file name is already used in another buffer:
        // - if the buffer is loaded, fail
        // - if the buffer is not loaded, delete it from the list
        (file_id, file_id_valid) = file_id_of(ffname);
        let obuf = if b.b_flags.has(BufFlags::DUMMY) {
            None
        } else {
            buflist_findname_file_id(ffname, &file_id, file_id_valid)
        };
        if let Some(mut o) = obuf.filter(|&o| o != buf) {
            let obuf = o.raw();
            // During startup a window may use a buffer that is not loaded yet.
            let in_use = tab_windows().any(|win| win.w_buffer == obuf);
            if !o.b_ml.ml_mfp.is_null() || in_use {
                // It is loaded or used in a window: fail.
                if message {
                    err(tr(c"E95: Buffer with this name already exists"));
                }
                free(ffname);
                return Err(Failed);
            }
            // Delete it from the list.
            // SAFETY: a live, unloaded buffer shown in no window.
            unsafe { close_buffer(None, Buf::new(obuf), DOBUF_WIPE as c_int, false, false) };
        }
        sfname = dup(sfname);
        if b.b_sfname != b.b_ffname {
            free(b.b_sfname);
        }
        free(b.b_ffname);
        b.b_ffname = ffname;
        b.b_sfname = sfname;
    }
    b.b_fname = b.b_sfname;
    b.file_id_valid = file_id_valid;
    if file_id_valid {
        b.file_id = file_id;
    }

    // SAFETY: a live buffer.
    unsafe { buf_name_changed(buf) };
    Ok(())
}

/// A crude way of changing a buffer's name; use with care. The name is
/// relative to the current directory.
pub unsafe fn buf_set_name(fnum: c_int, name: *mut c_char) {
    let Some(mut b) = find_buf(fnum) else {
        return;
    };

    if b.b_sfname != b.b_ffname {
        free(b.b_sfname);
    }
    free(b.b_ffname);
    b.b_ffname = dup(name);
    b.b_sfname = ptr::null_mut();
    // Allocate ffname and expand into a full path.
    // SAFETY: the buffer's own two name slots.
    unsafe { fname_expand(b, &raw mut b.b_ffname, &raw mut b.b_sfname) };
    b.b_fname = b.b_sfname;
}

/// What has to happen once a buffer's name has changed.
pub unsafe fn buf_name_changed(b: Buf) {
    if !b.b_ml.ml_mfp.is_null() {
        // The swap file's name follows the buffer's.
        // SAFETY: a live buffer with a memline.
        unsafe { ml_setname(b.raw()) };
    }
    let mut cur = current_win();
    if cur.w_buffer == b.raw() {
        // Check the file name against the argument list.
        // SAFETY: a live window.
        check_arg_idx(cur);
    }
    // SAFETY: the window title and the status lines are drawn from globals.
    unsafe { maketitle() };
    // SAFETY: as above.
    unsafe { status_redraw_all() };
    // SAFETY: a live buffer, whose named file marks and timestamp follow its
    // name.
    unsafe { fmarks_check_names(b.raw()) };
    // SAFETY: as above.
    unsafe { ml_timestamp(b.raw()) };
}

// ---------------------------------------------------------------------------
// The alternate file

/// Set the alternate file name for the current window.
pub unsafe fn setaltfname(ffname: *mut c_char, sfname: *mut c_char, lnum: linenr_T) -> Option<Buf> {
    // Create a buffer; 'buflisted' is not set if it is a new one.
    // SAFETY: two names to hand over, either of which may be null; the
    // answer is a live buffer or null.
    let buf = unsafe { Buf::from_raw(buflist_new(ffname, sfname, lnum, 0)) };
    if let Some(buf) = buf
        && !cmdmod_has(CmdModFlags::KEEPALT)
    {
        current_win().w_alt_fnum = buf.handle as c_int;
    }
    buf
}

/// The alternate file name for the current window, null when there is none.
pub unsafe fn getaltfname(errmsg: bool) -> *mut c_char {
    let mut fname: *mut c_char = ptr::null_mut();
    let mut dummy: linenr_T = 0;
    // SAFETY: two locals to fill in.
    if unsafe { buflist_name_nr(0, &raw mut fname, &raw mut dummy) }.is_err() {
        if errmsg {
            err(tr_raw(e_noalt.as_ptr()));
        }
        return ptr::null_mut();
    }
    fname
}

/// Add a file name to the buffer list and answer its number. Takes
/// [`buflist_new`]'s flags, except `BLN_DUMMY`.
pub unsafe fn buflist_add(fname: *mut c_char, flags: c_int) -> c_int {
    // SAFETY: a name to hand over, which may be null.
    let buf = unsafe { buflist_new(fname, ptr::null_mut(), 0 as linenr_T, flags) };
    if buf.is_null() {
        return 0;
    }
    // SAFETY: non-null, hence live.
    unsafe { Buf::new(buf) }.handle as c_int
}

/// Record the alternate cursor position for the current buffer in `win`,
/// saving its window-local options too.
pub unsafe fn buflist_altfpos(win: Win) {
    let (lnum, col) = (win.w_cursor.lnum, win.w_cursor.col);
    // SAFETY: reads the window's options into the buffer's saved entry.
    unsafe { buflist_setfpos(Buf::current(), Some(win), lnum, col, true) };
}

// ---------------------------------------------------------------------------
// Is this the same file?

/// Whether `ffname` (a full path) names a different file from the current
/// buffer's.
pub unsafe fn otherfile(ffname: *mut c_char) -> bool {
    // SAFETY: the current buffer and a NUL-terminated full path.
    unsafe { otherfile_buf(Buf::current(), ffname, ptr::null_mut(), false) }
}

/// Whether `ffname` (a full path) names a different file from `buf`'s.
///
/// `file_id_p` is the caller's already-computed file id for `ffname`, null to
/// have it looked up here.
pub(crate) unsafe fn otherfile_buf(
    mut b: Buf,
    ffname: *mut c_char,
    file_id_p: *mut FileID,
    file_id_valid: bool,
) -> bool {
    if is_empty_name(ffname) || b.b_ffname.is_null() {
        return true;
    }
    if names_equal(ffname, b.b_ffname) {
        return false;
    }

    let mut own = FileID {
        inode: 0,
        device_id: 0,
    };
    let (file_id_p, file_id_valid) = if file_id_p.is_null() {
        let (id, valid) = file_id_of(ffname);
        own = id;
        (&raw mut own, valid)
    } else {
        (file_id_p, file_id_valid)
    };
    if !file_id_valid {
        return true;
    }

    if same_file_id(&mut b, file_id_p) {
        // SAFETY: a live buffer.
        unsafe { buf_set_file_id(b) };
        if same_file_id(&mut b, file_id_p) {
            return false;
        }
    }
    true
}

/// Record the file id of `buf`'s file, for recognising it under another name.
pub unsafe fn buf_set_file_id(mut b: Buf) {
    if b.b_fname.is_null() {
        b.file_id_valid = false;
        return;
    }
    let (file_id, valid) = file_id_of(b.b_fname);
    b.file_id_valid = valid;
    if valid {
        b.file_id = file_id;
    }
}

/// Make `*ffname` a full file name and point `*sfname` at the name given, if
/// it had none. The value `*ffname` comes back as should be treated as not
/// allocated.
pub unsafe fn fname_expand(_buf: Buf, ffname: *mut *mut c_char, sfname: *mut *mut c_char) {
    // SAFETY: the caller's promise -- two name slots to read and write.
    let (ffname, sfname) = unsafe { (&mut *ffname, &mut *sfname) };
    if ffname.is_null() {
        // No file name given, nothing to do.
        return;
    }
    if sfname.is_null() {
        // No short file name given, use ffname.
        *sfname = *ffname;
    }
    // SAFETY: a NUL-terminated name.
    *ffname = unsafe { fix_fname(*ffname) };
}
