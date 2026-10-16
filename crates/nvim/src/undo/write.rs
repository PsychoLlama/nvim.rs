//! `u_write_undo`: writing a buffer's undo tree out to its undo file.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::fs::{File, OpenOptions, Permissions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

use super::file::*;
use super::format::*;
use super::store::Marks;
use super::*;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::os::fs::{file_info, getperm, path_exists};
use crate::semsg;
use crate::smsg;
use crate::winlayer::Buf;
use core::ffi::CStr;

/// Writes `buffer`'s undo tree to `name`, or to the file `'undodir'` picks for
/// it when `name` is `None` — which is what `:wundo` with no argument means.
///
/// `forceit` is `:wundo!`: overwrite whatever is there without checking that
/// it looks like an undo file first.
pub fn u_write_undo(
    name: Option<&CStr>,
    forceit: bool,
    buffer: Buf,
    hash: &[u8; UNDO_HASH_SIZE as usize],
) {
    let picked;
    let file_name: &CStr = match name {
        Some(name) => name,
        None => {
            let full = buffer.name.full().map(XString::from_cstr);
            match u_get_undo_file_name(full.as_ref().map(XString::as_cstr), false) {
                Some(found) => {
                    picked = found;
                    picked.as_cstr()
                }
                None => {
                    verbosely(true, || {
                        let text = msg_cstr(gettext(NO_UNDODIR));
                        smsg!(0, "{text}");
                    });
                    return;
                }
            }
        }
    };
    write_undo_file(file_name, name.is_none(), forceit, buffer, hash);
}

/// "no directory in 'undodir' will take it", the one message a write can
/// give before it has a path at all.
const NO_UNDODIR: &CStr = c"Cannot write undo file in any directory in 'undodir'";

/// The write itself, once the target path is known. `automatic` says whether
/// the caller left the name to `'undodir'`.
fn write_undo_file(
    file_name: &CStr,
    automatic: bool,
    forceit: bool,
    buffer: Buf,
    hash: &[u8; UNDO_HASH_SIZE as usize],
) {
    if path_exists(file_name) {
        // Never clobber a file that is not an undo file, unless `:wundo!`
        // said to.
        if (automatic || !forceit) && !looks_like_undo_file(file_name, automatic) {
            return;
        }
        os_remove(file_name);
    }
    if buffer.b_u_numhead == 0 && buffer.b_u_line.is_none() {
        if p_verbose() > 0 {
            verb_msg(gettext(c"Skipping undo file write, nothing to undo"));
        }
        return;
    }

    // The undo file inherits the edited file's permissions, minus
    // anything but read/write: it holds the same text. A copy of the name:
    // the write below syncs undo, which reaches the editor.
    let edited = buffer.name.full().map(XString::from_cstr);
    let perm = edited
        .as_ref()
        .map_or(0o600, |name| match getperm(name.as_cstr()) {
            perm if perm < 0 => 0o600,
            perm => perm,
        })
        & 0o666;

    // Created here and nowhere else: never through a link, never over a
    // file someone raced into place.
    let path = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(file_name.to_bytes());
    let mode = u32::try_from(perm).expect("masked to the permission bits");
    let opened = OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(::libc::O_NOFOLLOW)
        .mode(mode)
        .open(path);
    let Ok(file) = opened else {
        let file_name = msg_cstr(file_name);
        semsg!("E828: Cannot open undo file for writing: {file_name}");
        return;
    };
    // The mode `open` was given went through the umask; this one does not.
    // A failure leaves the file as `open` made it, as upstream ignored it.
    let _ = file.set_permissions(Permissions::from_mode(mode));
    // Always under 'verbose', even when the user named the file.
    verbosely(true, || {
        let file_name = msg_cstr(file_name);
        smsg!(0, "Writing undo file: {file_name}");
    });
    if let Some(edited) = edited.as_ref() {
        match_group(&file, edited.as_cstr(), mode);
    }

    u_sync(true);
    let mut bi = BufInfo::writer(buffer, file);
    let write_ok = write_tree(&mut bi, buffer, hash);
    drop(bi);
    if !write_ok {
        let file_name = msg_cstr(file_name);
        semsg!("E829: Write error in undo file: {file_name}");
    }
    if let Some(edited) = edited.as_ref() {
        let acl: VimAcl = os_get_acl(edited.as_cstr());
        os_set_acl(file_name, acl);
        os_free_acl(acl);
    }
}

/// Whether the file already at `file_name` starts with [`UF_START_MAGIC`],
/// saying so when it does not.
fn looks_like_undo_file(file_name: &CStr, automatic: bool) -> bool {
    let path = <std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(file_name.to_bytes());
    let Ok(mut file) = File::open(path) else {
        verbosely(automatic, || {
            let file_name = msg_cstr(file_name);
            smsg!(
                0,
                "Will not overwrite with undo file, cannot read: {file_name}"
            );
        });
        return false;
    };
    let mut magic = [0u8; UF_START_MAGIC.len()];
    if file.read_exact(&mut magic).is_ok() && magic == UF_START_MAGIC {
        return true;
    }
    verbosely(automatic, || {
        let file_name = msg_cstr(file_name);
        smsg!(
            0,
            "Will not overwrite, this is not an undo file: {file_name}"
        );
    });
    false
}

/// Gives the undo file the edited file's group when they differ, and drops
/// the group's read bit if it cannot. Asked of the open file, not its name.
fn match_group(file: &File, edited: &CStr, perm: u32) {
    let Some(edited) = file_info(edited) else {
        return;
    };
    let Ok(written) = file.metadata() else {
        return;
    };
    let gid = u32::try_from(edited.stat.st_gid).expect("a group id fits a `gid_t`");
    if gid == written.gid() || std::os::unix::fs::fchown(file, None, Some(gid)).is_ok() {
        return;
    }
    // The group could not be changed: make sure it cannot read the undo file
    // either.
    let _ = file.set_permissions(Permissions::from_mode(perm & 0o707 | (perm & 0o7) << 3));
}

/// Lays down the file header, every header in the tree, and the end marker.
fn write_tree(bi: &mut BufInfo, buffer: Buf, hash: &[u8; UNDO_HASH_SIZE as usize]) -> bool {
    if !serialize_header(bi, hash) {
        return false;
    }
    // Every header exactly once, in whatever order the depth-first walk
    // reaches them: the links are sequence numbers, so the reader does
    // not care which order they arrive in. One stamp does for both of the
    // walk's marks, because "reached" is the whole question here.
    let tree = buffer.tree_walk(buffer.b_u_oldhead, Marks::next_once());
    for visit in tree {
        if !visit.first {
            continue;
        }
        if let Some(uh) = buffer.header(visit.link)
            && !serialize_uhp(bi, uh)
        {
            return false;
        }
    }
    let mut write_ok = bi.write_bytes(UF_HEADER_END_MAGIC as uintmax_t, 2);
    // 'fsync' asks for the bytes to be on the disk before we call the
    // write done.
    let fsync_wanted = if buffer.b_p_fs >= 0 {
        buffer.b_p_fs != 0
    } else {
        p_fs()
    };
    if fsync_wanted && !bi.sync() {
        write_ok = false;
    }
    write_ok
}
