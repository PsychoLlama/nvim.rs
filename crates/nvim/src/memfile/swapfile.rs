//! The swap file's name, and opening it.
//!
//! Split out of [`super`] for the file-size cap: everything here is about
//! the *file* a memfile is backed by rather than the blocks in it - the
//! name it was given and the full one, freeing both, and the `open` that
//! [`MemFile::open`] and a rename go through.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::event::uv::uv_gid_t;
use crate::message_fmt::msg_cstr;
use crate::os::fs::{last_errno_text, link_exists, open_file};
use crate::path::full_name_save;
use crate::semsg;

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
        // The file is opened close-on-exec, which upstream's
        // `os_set_cloexec` after the open made sure of a second time.
        if flags & O_CREAT != 0 && link_exists(name) {
            self.file = None;
            emsg(gettext(c"E300: Swap file already exists (symlink attack?)"));
        } else {
            flags |= O_NOFOLLOW;
            self.open_flags = flags;
            self.file = open_file(name, flags, SWAPFILE_MODE).ok();
        }

        if self.file.is_none() {
            self.free_fnames();
            return false;
        }
        true
    }

    /// Close the swap file and forget it, keeping the blocks: what renaming
    /// the swap file does first. How the close went does not matter.
    pub(crate) fn close_file(&mut self) {
        self.file = None;
    }

    /// Reopen the swap file read-write under the name it has, after a
    /// rename (or a failed one). Whether it is open now.
    pub(crate) fn reopen(&mut self) -> bool {
        self.file = self
            .fname
            .as_deref()
            .and_then(|name| open_file(name, O_RDWR, 0).ok());
        self.file.is_some()
    }
}

/// A copy of `buffer`'s swap file name, or `None` when it has none.
pub(crate) fn buf_swap_name(buffer: Buf) -> Option<CString> {
    buffer.b_ml.ml_mfp.as_ref()?.fname().map(CStr::to_owned)
}

/// `PERROR`: an error message with the failing call's `strerror(errno)`
/// after it.
pub(super) fn perror_msg(message: &'static CStr) {
    let why = last_errno_text();
    semsg!("{}: {}", msg_cstr(gettext(message)), msg_cstr(&why));
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
    /// file.
    pub(crate) fn swap_file_closed(self) -> bool {
        self.b_ml
            .ml_mfp
            .as_ref()
            .is_none_or(|memfile| !memfile.has_file())
    }

    /// Give the buffer's swap file the group `gid`. False when there is no
    /// memfile, or when the change was refused for want of permission.
    ///
    /// Upstream compares `os_fchown`'s libuv code against -1, which is
    /// `UV_EPERM`, so `EPERM` is the one failure it sees: any other (`EBADF`
    /// for a memfile with no file open included) reads as success, and still
    /// does.
    pub(crate) fn set_swap_file_group(self, gid: uv_gid_t) -> bool {
        let Some(memfile) = self.b_ml.ml_mfp.as_ref() else {
            return false;
        };
        let Some(file) = memfile.file.as_ref() else {
            return true;
        };
        std::os::unix::fs::fchown(file, None, Some(gid)).map_or_else(
            |error| error.raw_os_error() != Some(::libc::EPERM),
            |()| true,
        )
    }

    /// The name of the buffer's swap file, if it has a memfile with one.
    pub(crate) fn swap_file_name(self) -> Option<CString> {
        buf_swap_name(self)
    }
}
