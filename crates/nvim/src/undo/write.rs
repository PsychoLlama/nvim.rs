//! `u_write_undo`: writing a buffer's undo tree out to its undo file.

#![deny(unsafe_op_in_unsafe_fn)]

use super::file::*;
use super::format::*;
use super::store::Marks;
use super::*;
use crate::winlayer::Buf;
use crate::{semsg_c, smsg_c};

/// Writes `buf`'s undo tree to `name`, or to the file `'undodir'` picks for
/// it when `name` is NULL.
///
/// `forceit` is `:wundo!`: overwrite whatever is there without checking that
/// it looks like an undo file first.
///
/// # Safety
///
/// `name` is NULL or a NUL-terminated path, and `hash` points at
/// [`UNDO_HASH_SIZE`] readable bytes.
pub unsafe fn u_write_undo(name: *const c_char, forceit: bool, buf: Buf, hash: *mut uint8_t) {
    let file_name: *mut c_char = if name.is_null() {
        // SAFETY: `b_ffname` is the buffer's own name or NULL.
        let picked = unsafe { u_get_undo_file_name(buf.b_ffname, false) };
        if picked.is_null() {
            verbosely(true, || {
                // SAFETY: a NUL-terminated literal.
                unsafe { smsg_c!(0, c"%s".as_ptr(), gettext(NO_UNDODIR.as_ptr()),) };
            });
            return;
        }
        picked
    } else {
        name.cast_mut()
    };
    // SAFETY: a live buffer and a NUL-terminated path, by the above.
    unsafe { write_undo_file(file_name, name.is_null(), forceit, buf, hash) };
    if !ptr::eq(file_name.cast_const(), name) {
        // SAFETY: `u_get_undo_file_name`'s allocation, which the caller's own
        // `name` is never.
        unsafe { xfree(file_name.cast()) };
    }
}

/// "no directory in 'undodir' will take it", the one message a write can
/// give before it has a path at all.
const NO_UNDODIR: &core::ffi::CStr = c"Cannot write undo file in any directory in 'undodir'";

/// The write itself, once the target path is known.
///
/// # Safety
///
/// As [`u_write_undo`], with `file_name` the resolved path and `automatic`
/// saying whether the caller left the name to `'undodir'`.
unsafe fn write_undo_file(
    file_name: *mut c_char,
    automatic: bool,
    forceit: bool,
    buf: Buf,
    hash: *mut uint8_t,
) {
    // SAFETY: a NUL-terminated path, by the contract above.
    if unsafe { os_path_exists(file_name) } {
        // Never clobber a file that is not an undo file, unless `:wundo!`
        // said to.
        // SAFETY: that same path, and the file is there.
        if (automatic || !forceit) && !unsafe { looks_like_undo_file(file_name, automatic) } {
            return;
        }
        // SAFETY: as above.
        unsafe { os_remove(file_name) };
    }
    if buf.b_u_numhead == 0 && buf.b_u_line_ptr.is_null() {
        if p_verbose.get() > 0 {
            // SAFETY: a NUL-terminated literal.
            let mesg = unsafe { gettext(c"Skipping undo file write, nothing to undo".as_ptr()) };
            // SAFETY: as above.
            unsafe { verb_msg(mesg) };
        }
        return;
    }

    // The undo file inherits the edited file's permissions, minus
    // anything but read/write: it holds the same text.
    let mut perm: c_int = 0o600;
    if !buf.b_ffname.is_null() {
        // SAFETY: the buffer's own name, NUL-terminated.
        perm = unsafe { os_getperm(buf.b_ffname) } as c_int;
        if perm < 0 {
            perm = 0o600;
        }
    }
    perm &= 0o666;

    // SAFETY: a NUL-terminated path, by the contract above.
    let fd = unsafe { os_open(file_name, O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW, perm) };
    if fd < 0 {
        // SAFETY: a NUL-terminated literal and path.
        unsafe {
            semsg_c!(
                gettext(c"E828: Cannot open undo file for writing: %s".as_ptr()),
                file_name,
            );
        }
        return;
    }
    // SAFETY: a NUL-terminated path, by the contract above.
    unsafe { os_setperm(file_name, perm) };
    // Always under 'verbose', even when the user named the file.
    verbosely(true, || {
        // SAFETY: a NUL-terminated literal and path.
        unsafe { smsg_c!(0, gettext(c"Writing undo file: %s".as_ptr()), file_name) };
    });
    // SAFETY: the descriptor just opened on that path, and a live buffer.
    unsafe { match_group(fd, file_name, perm, buf) };

    // SAFETY: our own descriptor, and a NUL-terminated mode.
    let fp: *mut FILE = unsafe { fdopen(fd, c"w".as_ptr()) };
    if fp.is_null() {
        // SAFETY: a NUL-terminated literal and path, and a descriptor that
        // `fdopen` did not take over.
        unsafe {
            semsg_c!(
                gettext(c"E828: Cannot open undo file for writing: %s".as_ptr()),
                file_name,
            );
            close(fd);
            os_remove(file_name);
        }
        return;
    }
    u_sync(true);
    let mut bi = bufinfo_T {
        bi_buf: buf,
        bi_fp: fp,
    };
    // SAFETY: an open undo file on a live buffer, and `hash` readable for
    // [`UNDO_HASH_SIZE`] bytes by the contract above.
    let write_ok = unsafe { write_tree(&raw mut bi, buf, hash, fd, fp) };
    // SAFETY: the stream opened just above, closed once.
    unsafe { fclose(fp) };
    if !write_ok {
        // SAFETY: a NUL-terminated literal and path.
        unsafe {
            semsg_c!(
                gettext(c"E829: Write error in undo file: %s".as_ptr()),
                file_name,
            );
        }
    }
    if !buf.b_ffname.is_null() {
        let acl: vim_acl_T = os_get_acl(buf.b_ffname);
        os_set_acl(file_name, acl);
        os_free_acl(acl);
    }
}

/// Whether the file already at `file_name` starts with [`UF_START_MAGIC`],
/// saying so when it does not.
///
/// # Safety
///
/// `file_name` is a NUL-terminated path to an existing file.
unsafe fn looks_like_undo_file(file_name: *mut c_char, automatic: bool) -> bool {
    // SAFETY: a NUL-terminated path, by the contract above.
    let fd = unsafe { os_open(file_name, O_RDONLY, 0) };
    if fd < 0 {
        verbosely(automatic, || {
            // SAFETY: a NUL-terminated literal and path.
            unsafe {
                let fmt = gettext(c"Will not overwrite with undo file, cannot read: %s".as_ptr());
                smsg_c!(0, fmt, file_name);
            }
        });
        return false;
    }
    let mut magic = [0u8; UF_START_MAGIC.len()];
    // SAFETY: an open descriptor and a buffer of exactly that many bytes.
    let len = unsafe { read_eintr(fd, magic.as_mut_ptr().cast(), magic.len()) };
    // SAFETY: our own descriptor.
    unsafe { close(fd) };
    if len == magic.len() as ssize_t && magic == UF_START_MAGIC {
        return true;
    }
    verbosely(automatic, || {
        // SAFETY: a NUL-terminated literal and path.
        unsafe {
            let fmt = gettext(c"Will not overwrite, this is not an undo file: %s".as_ptr());
            smsg_c!(0, fmt, file_name);
        }
    });
    false
}

/// Gives the undo file the edited file's group when they differ, and drops
/// the group's read bit if it cannot.
///
/// # Safety
///
/// `fd` is open on `file_name` and `buf` points at a live buffer.
unsafe fn match_group(fd: c_int, file_name: *mut c_char, perm: c_int, buf: Buf) {
    if buf.b_ffname.is_null() {
        return;
    }
    let mut edited = FileInfo::default();
    let mut written = FileInfo::default();
    // SAFETY: the buffer's own name and the undo file's path, both
    // NUL-terminated, two writable records of ours, and an open descriptor —
    // all by the contract above.
    let group_stuck = unsafe {
        os_fileinfo(buf.b_ffname, &raw mut edited)
            && os_fileinfo(file_name, &raw mut written)
            && edited.stat.st_gid != written.stat.st_gid
            && os_fchown(fd, u32::MAX as uv_uid_t, edited.stat.st_gid as uv_gid_t) != 0
    };
    if group_stuck {
        // The group could not be changed: make sure it cannot read the
        // undo file either.
        // SAFETY: a NUL-terminated path, by the contract above.
        unsafe { os_setperm(file_name, perm & 0o707 | (perm & 0o7) << 3) };
    }
}

/// Lays down the file header, every header in the tree, and the end marker.
///
/// # Safety
///
/// `bi` is open on `buf`'s undo file, `buf` points at a live buffer, `hash`
/// at [`UNDO_HASH_SIZE`] readable bytes, and `fd`/`fp` are the same open
/// file as `bi`.
unsafe fn write_tree(
    bi: *mut bufinfo_T,
    buf: Buf,
    hash: *mut uint8_t,
    fd: c_int,
    fp: *mut FILE,
) -> bool {
    // SAFETY: an open undo file, and `hash` readable for [`UNDO_HASH_SIZE`]
    // bytes, both by the contract above.
    if !unsafe { serialize_header(bi, hash) } {
        return false;
    }
    // Every header exactly once, in whatever order the depth-first walk
    // reaches them: the links are sequence numbers, so the reader does
    // not care which order they arrive in. One stamp does for both of the
    // walk's marks, because "reached" is the whole question here.
    let tree = buf.tree_walk(buf.b_u_oldhead, Marks::next_once());
    for visit in tree {
        // SAFETY: an open undo file, and a header the walk reached, which is
        // therefore live.
        if visit.first && !unsafe { serialize_uhp(bi, visit.header.raw()) } {
            return false;
        }
    }
    // SAFETY: an open undo file, by the contract above.
    let mut write_ok = unsafe { undo_write_bytes(bi, UF_HEADER_END_MAGIC as uintmax_t, 2) };
    // 'fsync' asks for the bytes to be on the disk before we call the
    // write done.
    let fsync_wanted = if buf.b_p_fs >= 0 {
        buf.b_p_fs
    } else {
        p_fs.get()
    };
    // SAFETY: `fp` and `fd` are the same open file as `bi`, by the contract
    // above.
    if fsync_wanted != 0 && unsafe { fflush(fp) } == 0 && unsafe { os_fsync(fd) } != 0 {
        write_ok = false;
    }
    write_ok
}
