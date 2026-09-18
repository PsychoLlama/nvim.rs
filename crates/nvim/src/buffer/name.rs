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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::arglist::check_arg_idx;
use crate::drawscreen::status_redraw_all;
use crate::ex_docmd::cmdmod_has;
use crate::mark::fmarks_check_names;
use crate::memline::{ml_setname, ml_timestamp};
use crate::memory::{XString, xfree, xstrdup};
use crate::message::e_noalt;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::os::fs::{os_fileid, os_fileid_equal};
use crate::path::{fix_fname, path_fnamecmp};
use crate::types::{BufName, CmdModFlags, Failed, FileID, LineNr};
use crate::winlayer::{Buf, Win, tab_windows};

// ---------------------------------------------------------------------------
// The neighbours, wrapped

fn free(p: *mut c_char) {
    // SAFETY: an owned allocation or null.
    unsafe { xfree(p.cast::<c_void>()) };
}

fn dup(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string; upstream passes the short name, which
    // `fname_expand` has just made non-null.
    unsafe { xstrdup(p) }
}

/// Give `name` the pair [`fname_expand`] leaves behind.
///
/// `full` is the block `fix_fname` allocated. `short` is null, its own
/// block, or -- where `fix_fname` answers the very pointer it was given --
/// `full` itself, which is the case upstream's `b_sfname != b_ffname`
/// guard exists for.
///
/// # Safety
///
/// Both pointers are live `xmalloc`-family blocks, or null, that nobody
/// else will free.
unsafe fn adopt_names(name: &mut BufName, full: *mut c_char, short: *mut c_char) {
    // SAFETY: the caller's promise.
    let owned_full = unsafe { XString::from_raw(full) };
    if ptr::eq(short, full) {
        name.set_shared(owned_full);
    } else {
        // SAFETY: the caller's promise.
        let owned_short = (!short.is_null()).then(|| unsafe { XString::from_raw(short) });
        name.set(owned_full, owned_short);
    }
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

fn same_file_id(buffer: &mut Buf, file_id: *mut FileID) -> bool {
    // SAFETY: the buffer's own file id, and the caller's, both live.
    buffer.file_id_valid && unsafe { os_fileid_equal(&raw mut buffer.file_id, file_id) }
}

/// Whether a name slot holds nothing: null or the empty string.
fn is_empty_name(p: *const c_char) -> bool {
    // SAFETY: null or a NUL-terminated name.
    p.is_null() || unsafe { *p } == 0
}

fn names_equal(a: *const c_char, b: *const c_char) -> bool {
    // SAFETY: two NUL-terminated paths, both non-null by the tests above.
    unsafe { path_fnamecmp(cstr::at(a), cstr::at(b)) == 0 }
}

fn current_win() -> Win {
    Win::current()
}

// ---------------------------------------------------------------------------
// Looking a name up

/// The file name and remembered line number of buffer `fnum`.
///
/// # Safety
///
/// `fname` must point at a writable `*mut c_char` slot the caller owns for
/// the call. `lnum` must point at a writable line number the caller owns.
pub unsafe fn buflist_name_nr(
    fnum: c_int,
    fname: *mut *mut c_char,
    lnum: *mut LineNr,
) -> Result<(), Failed> {
    let Some(buf) = find_buf(fnum) else {
        return Err(Failed);
    };
    if buf.name.is_unnamed() {
        return Err(Failed);
    }
    // SAFETY: the caller's promise -- two out-parameters to fill in.
    let (fname, lnum) = unsafe { (&mut *fname, &mut *lnum) };
    *fname = buf.name.shown_ptr();
    *lnum = buflist_findlnum(buf);
    Ok(())
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
    // The names below this point are locals `fname_expand` replaces with
    // allocations of its own, which the buffer then adopts; the caller's
    // bytes are only read, so the `cast_mut` is the C signature's.
    let mut ffname = ffname_arg.map_or(ptr::null_mut(), |n| n.as_ptr().cast_mut());
    let mut sfname = sfname_arg.map_or(ptr::null_mut(), |n| n.as_ptr().cast_mut());
    let mut file_id = FileID {
        inode: 0,
        device_id: 0,
    };
    let mut file_id_valid = false;

    if is_empty_name(ffname) {
        // Removing the name. Upstream's three lines here are the
        // `b_sfname != b_ffname` dance; the name owns its blocks now.
        b.name.clear();
    } else {
        // SAFETY: two locals holding a name each.
        unsafe { fname_expand(&raw mut ffname, &raw mut sfname) };
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
        if let Some(o) = obuf.filter(|&o| o != buffer) {
            let obuf = o.raw();
            // During startup a window may use a buffer that is not loaded yet.
            let in_use = tab_windows().any(|win| win.w_buffer == obuf);
            if !o.b_ml.ml_mfp.is_null() || in_use {
                // It is loaded or used in a window: fail.
                if message {
                    emsg(gettext(c"E95: Buffer with this name already exists"));
                }
                free(ffname);
                return Err(Failed);
            }
            // Delete it from the list.
            // SAFETY: a live, unloaded buffer shown in no window.
            unsafe { close_buffer(None, Buf::new(obuf), DOBUF_WIPE.cast_signed(), false, false) };
        }
        // SAFETY: `ffname` is the block `fix_fname` just allocated and
        // `sfname` the copy taken here; the buffer takes over both, and
        // what it held is released with them.
        unsafe { adopt_names(&mut b.name, ffname, dup(sfname)) };
    }
    b.file_id_valid = file_id_valid;
    if file_id_valid {
        b.file_id = file_id;
    }

    buf_name_changed(buffer);
    Ok(())
}

/// A crude way of changing a buffer's name; use with care. The name is
/// relative to the current directory.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn buf_set_name(fnum: c_int, name: *mut c_char) {
    let Some(mut b) = find_buf(fnum) else {
        return;
    };

    // Allocate ffname and expand into a full path. The copy `fname_expand`
    // is handed becomes the *short* name; the full one is what `fix_fname`
    // answers.
    let mut ffname = dup(name);
    let mut sfname = ptr::null_mut();
    // SAFETY: two locals holding a name each.
    unsafe { fname_expand(&raw mut ffname, &raw mut sfname) };
    // SAFETY: the two blocks `fname_expand` left behind, which nothing
    // else holds.
    unsafe { adopt_names(&mut b.name, ffname, sfname) };
}

/// What has to happen once a buffer's name has changed.
pub fn buf_name_changed(b: Buf) {
    if !b.b_ml.ml_mfp.is_null() {
        // The swap file's name follows the buffer's.
        ml_setname(b);
    }
    let cur = current_win();
    if cur.w_buffer == b.raw() {
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
    // SAFETY: two names to hand over, either of which may be null; the
    // answer is a live buffer or null.
    let ffname = ffname.map_or(ptr::null_mut(), |n| n.as_ptr().cast_mut());
    let sfname = sfname.map_or(ptr::null_mut(), |n| n.as_ptr().cast_mut());
    let buf = unsafe { buflist_new(ffname, sfname, lnum, 0) };
    if let Some(buf) = buf
        && !cmdmod_has(CmdModFlags::KEEPALT)
    {
        current_win().w_alt_fnum = buf.handle as c_int;
    }
    buf
}

/// The alternate file name for the current window, null when there is none.
pub fn getaltfname(errmsg: bool) -> *mut c_char {
    let mut fname: *mut c_char = ptr::null_mut();
    let mut dummy: LineNr = 0;
    // SAFETY: two locals to fill in.
    if unsafe { buflist_name_nr(0, &raw mut fname, &raw mut dummy) }.is_err() {
        if errmsg {
            emsg(gettext(e_noalt));
        }
        return ptr::null_mut();
    }
    fname
}

/// Add a file name to the buffer list and answer its number. Takes
/// [`buflist_new`]'s flags, except `BLN_DUMMY`.
///
/// # Safety
///
/// `fname` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn buflist_add(fname: *mut c_char, flags: c_int) -> c_int {
    // SAFETY: a name to hand over, which may be null.
    let buf = unsafe { buflist_new(fname, ptr::null_mut(), 0 as LineNr, flags) };
    if buf.is_none() {
        return 0;
    }
    // SAFETY: non-null, hence live.
    buf.expect("a live handle").handle as c_int
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
/// buffer's.
///
/// # Safety
///
/// `ffname` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn otherfile(ffname: *mut c_char) -> bool {
    // SAFETY: the current buffer and a NUL-terminated full path.
    unsafe { otherfile_buf(Buf::current(), ffname, ptr::null_mut(), false) }
}

/// Whether `ffname` (a full path) names a different file from `buffer`'s.
///
/// `file_id_p` is the caller's already-computed file id for `ffname`, null to
/// have it looked up here.
///
/// # Safety
///
/// `ffname` must point at a NUL-terminated string, unaliased for the call.
/// `file_id_p` must point at a live `FileID`, unaliased for the call.
pub(crate) unsafe fn otherfile_buf(
    mut b: Buf,
    ffname: *mut c_char,
    file_id_p: *mut FileID,
    file_id_valid: bool,
) -> bool {
    if is_empty_name(ffname) || b.name.full().is_none() {
        return true;
    }
    if names_equal(ffname, b.name.full_ptr()) {
        return false;
    }

    let mut own;
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
        buf_set_file_id(b);
        if same_file_id(&mut b, file_id_p) {
            return false;
        }
    }
    true
}

/// Record the file id of `buffer`'s file, for recognising it under another name.
pub fn buf_set_file_id(mut b: Buf) {
    if b.name.is_unnamed() {
        b.file_id_valid = false;
        return;
    }
    let (file_id, valid) = file_id_of(b.name.shown_ptr());
    b.file_id_valid = valid;
    if valid {
        b.file_id = file_id;
    }
}

/// Make `*ffname` a full file name and point `*sfname` at the name given, if
/// it had none. The value `*ffname` comes back as should be treated as not
/// allocated.
///
/// # Safety
///
/// `ffname` must point at a writable `*mut c_char` slot the caller owns for
/// the call. `sfname` must point at a writable `*mut c_char` slot the caller
/// owns for the call.
pub unsafe fn fname_expand(ffname: *mut *mut c_char, sfname: *mut *mut c_char) {
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
