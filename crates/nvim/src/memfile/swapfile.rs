//! The swap file's name, and opening it.
//!
//! Split out of [`super`] for the file-size cap: everything here is about
//! the *file* a memfile is backed by rather than the blocks in it - the
//! name it was given and the full one, freeing both, and the `open` that
//! `mf_open` and a rename go through.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `memfile/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use super::*;

/// The swap file's name as it was given, or null when the memfile is memory
/// only. It stays valid until the name is changed or the memfile closed.
///
/// # Safety
/// `mfp` must point at a memfile.
pub(crate) unsafe fn mf_fname(mfp: *const MemFile) -> *const c_char {
    match unsafe { &(*mfp).fname } {
        Some(fname) => fname.as_ptr(),
        None => core::ptr::null(),
    }
}

/// Take over an allocated C string, which is released.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string, unaliased for the call.
pub(super) unsafe fn take_cstring(p: *mut c_char) -> CString {
    unsafe {
        let owned = CStr::from_ptr(p).to_owned();
        xfree(p.cast::<c_void>());
        owned
    }
}

/// Release the swap file's names.
///
/// # Safety
///
/// `mfp` must point at a live memfile, unaliased for the call.
pub(crate) unsafe fn mf_free_fnames(mfp: *mut MemFile) {
    unsafe {
        (*mfp).fname = None;
        (*mfp).ffname = None;
    }
}

/// Name the swap file. `fname` must be allocated, and is consumed.
///
/// Only called when creating or renaming it, so the full path is always
/// worked out afresh.
///
/// # Safety
///
/// `mfp` must point at a live memfile, unaliased for the call. `fname` must
/// point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn mf_set_fnames(mfp: *mut MemFile, fname: *mut c_char) {
    unsafe {
        let full = full_name_save(fname, false);
        (*mfp).fname = Some(take_cstring(fname));
        (*mfp).ffname = (!full.is_null()).then(|| take_cstring(full));
    }
}

/// Make the swap file's name absolute — before a `:cd` makes the relative
/// one mean something else.
///
/// # Safety
///
/// `mfp` must point at a live memfile, unaliased for the call.
pub(crate) unsafe fn mf_fullname(mfp: *mut MemFile) {
    unsafe {
        if mfp.is_null() || (*mfp).fname.is_none() || (*mfp).ffname.is_none() {
            return;
        }
        (*mfp).fname = (*mfp).ffname.take();
    }
}

/// Whether any block still owes the file a number.
///
/// # Safety
///
/// `mfp` must point at a live memfile, unaliased for the call.
pub(crate) unsafe fn mf_need_trans(mfp: *mut MemFile) -> bool {
    unsafe { (*mfp).fname.is_some() && (*mfp).mf_neg_count > 0 }
}

/// Open the swap file. `fname` must be allocated, and is consumed — also
/// when this fails, in which case the memfile stays memory-only.
///
/// # Safety
///
/// `mfp` must point at a live memfile, unaliased for the call. `fname` must
/// point at a NUL-terminated string, unaliased for the call.
pub(super) unsafe fn mf_do_open(mfp: *mut MemFile, fname: *mut c_char, mut flags: c_int) -> bool {
    unsafe {
        // `fname` has to have been allocated.
        mf_set_fnames(mfp, fname);
        debug_assert!(!mf_fname(mfp).is_null());

        // A swap file being created really should not exist yet. If it does
        // and it is a symlink, this is most likely an attack.
        let mut file_info: FileInfo = core::mem::zeroed();
        if flags & O_CREAT != 0 && os_fileinfo_link(mf_fname(mfp), &raw mut file_info) {
            (*mfp).mf_fd = -1;
            emsg(gettext(c"E300: Swap file already exists (symlink attack?)"));
        } else {
            flags |= O_NOFOLLOW;
            (*mfp).mf_flags = flags;
            (*mfp).mf_fd = os_open(mf_fname(mfp), flags, SWAPFILE_MODE);
        }

        if (*mfp).mf_fd < 0 {
            mf_free_fnames(mfp);
            return false;
        }

        os_set_cloexec((*mfp).mf_fd);
        true
    }
}

/// `PERROR`: an error message with the failing call's `strerror` after it.
pub(super) fn perror_msg(message: &'static CStr) {
    unsafe {
        semsg!(
            "{}: {}",
            c_str(gettext(message).as_ptr()),
            c_str(strerror(*__errno_location()))
        );
    }
}
