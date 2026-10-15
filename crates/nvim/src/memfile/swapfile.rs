//! The swap file's name, and opening it.
//!
//! Split out of [`super`] for the file-size cap: everything here is about
//! the *file* a memfile is backed by rather than the blocks in it - the
//! name it was given and the full one, freeing both, and the `open` that
//! [`MemFile::open`] and a rename go through.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `memfile/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use super::*;
use crate::message_fmt::c_str;
use crate::os::fs::{link_exists, os_fchown, os_set_cloexec};
use crate::path::full_name_save;
use crate::semsg;
use crate::types::uv_gid_t;
use ::libc::{__errno_location, strerror};

impl MemFile {
    /// The swap file's name as it was given, or `None` when the memfile is
    /// memory only.
    pub(crate) fn fname(&self) -> Option<&CStr> {
        self.fname.as_deref()
    }

    /// Release the swap file's names.
    pub(crate) fn free_fnames(&mut self) {
        self.fname = None;
        self.ffname = None;
    }

    /// Name the swap file.
    ///
    /// Only called when creating or renaming it, so the full path is always
    /// worked out afresh.
    pub(crate) fn set_fnames(&mut self, fname: CString) {
        let full = full_name_save(&fname, false);
        self.fname = Some(fname);
        self.ffname = Some(full.as_cstr().to_owned());
    }

    /// Make the swap file's name absolute — before a `:cd` makes the
    /// relative one mean something else.
    pub(crate) fn make_name_full(&mut self) {
        if self.fname.is_some() && self.ffname.is_some() {
            self.fname = self.ffname.take();
        }
    }

    /// Whether any block still owes the file a number.
    pub(crate) fn need_trans(&self) -> bool {
        self.fname.is_some() && self.mf_neg_count > 0
    }

    /// Open the swap file as `fname`. On failure the memfile stays memory
    /// only and keeps no name.
    pub(super) fn do_open(&mut self, fname: CString, mut flags: c_int) -> bool {
        self.set_fnames(fname);
        let name = self.fname.as_deref().expect("just named");

        // A swap file being created really should not exist yet. If it does
        // and it is a symlink, this is most likely an attack.
        if flags & O_CREAT != 0 && link_exists(name) {
            self.mf_fd = -1;
            emsg(gettext(c"E300: Swap file already exists (symlink attack?)"));
        } else {
            flags |= O_NOFOLLOW;
            self.mf_flags = flags;
            // SAFETY: a NUL-terminated name the memfile owns.
            self.mf_fd = unsafe { os_open(name.as_ptr(), flags, SWAPFILE_MODE) };
        }

        if self.mf_fd < 0 {
            self.free_fnames();
            return false;
        }

        // SAFETY: the descriptor just opened.
        unsafe { os_set_cloexec(self.mf_fd) };
        true
    }

    /// Close the swap file's descriptor and forget it, keeping the blocks:
    /// what renaming the swap file does first.
    pub(crate) fn close_fd(&mut self) {
        if self.mf_fd >= 0 {
            // SAFETY: the memfile's own descriptor, closed once.
            unsafe { close(self.mf_fd) };
            self.mf_fd = -1;
        }
    }

    /// Reopen the swap file read-write under the name it has, after a
    /// rename (or a failed one). Whether there is a descriptor now.
    pub(crate) fn reopen(&mut self) -> bool {
        let name = self
            .fname
            .as_deref()
            .map_or(core::ptr::null(), CStr::as_ptr);
        // SAFETY: the memfile's NUL-terminated name, or null, which
        // `os_open` refuses.
        self.mf_fd = unsafe { os_open(name, O_RDWR, 0) };
        if self.mf_fd < 0 {
            return false;
        }
        // SAFETY: the descriptor just opened.
        unsafe { os_set_cloexec(self.mf_fd) };
        true
    }
}

/// A copy of `buffer`'s swap file name, or `None` when it has none.
pub(crate) fn buf_swap_name(buffer: Buf) -> Option<CString> {
    buffer.b_ml.ml_mfp.as_ref()?.fname().map(CStr::to_owned)
}

/// `PERROR`: an error message with the failing call's `strerror` after it.
pub(super) fn perror_msg(message: &'static CStr) {
    // SAFETY: `strerror` answers a NUL-terminated message for any number,
    // read before anything can overwrite it.
    unsafe {
        semsg!(
            "{}: {}",
            c_str(gettext(message).as_ptr()),
            c_str(strerror(*__errno_location()))
        );
    }
}

impl Buf {
    /// Whether the buffer has a memfile with a swap file name. Allocates
    /// nothing, so the out-of-memory exit may ask.
    pub(crate) fn has_swap_file(self) -> bool {
        self.b_ml
            .ml_mfp
            .as_ref()
            .is_some_and(|memfile| memfile.fname.is_some())
    }

    /// Make the swap file's name absolute, if the buffer has one: before a
    /// `:cd` makes the relative one mean something else.
    pub(crate) fn make_swap_name_full(mut self) {
        if let Some(memfile) = self.b_ml.ml_mfp.as_deref_mut() {
            memfile.make_name_full();
        }
    }

    /// The memfile's dirty state, `None` when the buffer has no memfile.
    pub(crate) fn memfile_dirty(self) -> Option<MfDirty> {
        self.b_ml.ml_mfp.as_ref().map(|memfile| memfile.mf_dirty)
    }

    /// Set the memfile's dirty state, when the buffer has a memfile.
    pub(crate) fn set_memfile_dirty(mut self, state: MfDirty) {
        if let Some(memfile) = self.b_ml.ml_mfp.as_deref_mut() {
            memfile.mf_dirty = state;
        }
    }

    /// Whether the buffer has no swap file open: no memfile, or one with no
    /// descriptor.
    pub(crate) fn swap_file_closed(self) -> bool {
        self.b_ml
            .ml_mfp
            .as_ref()
            .is_none_or(|memfile| memfile.mf_fd < 0)
    }

    /// Give the buffer's swap file the group `gid`: whether that worked.
    /// False too when there is no swap file open.
    pub(crate) fn set_swap_file_group(self, gid: uv_gid_t) -> bool {
        // SAFETY: the memfile's own descriptor.
        self.b_ml
            .ml_mfp
            .as_ref()
            .is_some_and(|memfile| unsafe { os_fchown(memfile.mf_fd, u32::MAX, gid) } != -1)
    }

    /// The name of the buffer's swap file, if it has a memfile with one.
    pub(crate) fn swap_file_name(self) -> Option<CString> {
        buf_swap_name(self)
    }
}
