//! The file being written over: checking it, backing it up, opening it, and
//! putting it back when the write goes wrong.
//!
//! [`get_fileinfo`] is the pre-flight: does the target exist, is it a regular
//! file, is it writable, and has it changed on disk since we read it.
//! [`open_write_file`] and [`finish_write`] bracket the write itself, the
//! second putting the original's ownership, permissions and ACL onto what
//! replaced it.
//! [`buf_write_make_backup`] then makes the backup that `'backup'`,
//! `'writebackup'` and `'patchmode'` ask for — either by copying the original
//! aside or by renaming it out of the way, which `'backupcopy'` chooses
//! between. [`backup_name`] is where `'backupdir'` and `'backupext'` turn
//! into a path.

#![forbid(unsafe_code)]

use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt as _;

use crate::cstr;
use crate::memline::{file_in_dir, percent_swname};
use crate::option::split_option_part;
use crate::option::vars::{P_BDIR, P_BEX};
use crate::os::fs::{
    close_file, copy_xattr, dir_exists, file_info, fsync_file, getperm, link_info, mkdir_recurse,
    path_exists,
};
use crate::os::users::{os_getgid, os_getuid};
use crate::os::uv_error::UV_ENOTSUP;
use crate::semsg;
use core::ffi::{CStr, c_int, c_uint};

use super::*;
use crate::highlight_group::HLF_E;
use crate::message::{emsg, msg};
use crate::message_fmt::msg_cstr;
use crate::option::cpo_has;
use crate::types::{CpoFlag, IOSIZE};

/// `st_mode & S_IFMT` for a regular file.
const S_IFREG: uint64_t = 0o100000;
/// `st_mode & S_IFMT` for a directory.
const S_IFDIR: uint64_t = 0o40000;

/// What the pre-flight learned about the file being written over.
pub(crate) struct TargetFile {
    /// The permission bits to give the written file, or -1 when there is no
    /// existing file to copy them from.
    pub perm: c_int,
    /// The target is a device or fifo: writable, but not backup-able.
    pub device: bool,
    /// The target does not exist yet.
    pub newfile: bool,
    /// The target exists and is not writable. Only reachable with `!`,
    /// which is what makes it worth remembering.
    pub readonly: bool,
    /// `!` made the file writable for the write; the 'w' bit has to come
    /// back off afterwards.
    pub made_writable: bool,
}

/// The backup a write left behind, if any.
pub(crate) struct Backup {
    /// The backup is a *copy*; the original file is still where it was.
    /// When false the original was renamed to the backup, so putting it
    /// back means renaming it back.
    pub copy: bool,
    /// Path to the backup, or `None` when `!` overrode a failure to make
    /// one.
    pub path: Option<XString>,
}

/// Open `path` for writing as `open(2)` would with `flags` beyond the access
/// mode, creating it with `mode`: libuv's `os_open`, answering the file or
/// libuv's error code.
fn open_for_write(path: &CStr, options: &mut OpenOptions, mode: c_int) -> Result<File, c_int> {
    use std::os::unix::ffi::OsStrExt as _;
    let path = std::ffi::OsStr::from_bytes(path.to_bytes());
    options
        .write(true)
        .mode(mode.cast_unsigned())
        .open(path)
        // libuv's code for an error is the negated `errno` on Unix.
        .map_err(|err| -err.raw_os_error().unwrap_or(::libc::EIO))
}

/// Warn about writing over a file that changed on disk since it was read.
///
/// The size is not checked: a tool like `gzip` keeps the timestamp but
/// cannot keep the size. Returns false if the user answers "no".
fn check_mtime(buffer: Buf, file_info: &FileInfo) -> bool {
    if buffer.b_mtime_read == 0
        || !time_differs(file_info, buffer.b_mtime_read, buffer.b_mtime_read_ns)
    {
        return true;
    }
    msg_scroll.set(1); // don't overwrite messages here
    msg_silent.set(0); // must give this prompt
    // Not emsg(): that would flush the buffers.
    msg(
        translate(c"WARNING: The file has been changed since reading it!!!"),
        HLF_E,
    );
    if ask_yesno(translate(c"Do you really want to write to it")) == 'n' as c_int {
        return false;
    }
    msg_scroll.set(0); // always overwrite the file message now
    true
}

/// The Unix half of the pre-flight: stat the target and classify it.
///
/// `file_info_old` receives the target's stat, or a zeroed one when there is
/// no target.
fn get_fileinfo_os(fname: &CStr, file_info_old: &mut FileInfo) -> Result<TargetFile, WriteError> {
    let found = file_info(fname);
    let exists = found.is_some();
    *file_info_old = found.unwrap_or_default();
    let mode = file_info_old.stat.st_mode;
    let (perm, device, newfile) = if !exists {
        (-1, false, true)
    } else if mode & __S_IFMT as uint64_t == S_IFREG {
        (mode as c_int, false, false)
    } else if mode & __S_IFMT as uint64_t == S_IFDIR {
        return Err(WriteError::numbered(c"E502", c"is a directory"));
    } else if os_nodetype(fname) != NODE_WRITABLE {
        return Err(WriteError::numbered(
            c"E503",
            c"is not a file or writable device",
        ));
    } else {
        // A device of some kind (or a fifo): we can write to it, but
        // there is nothing to back up and no permissions worth copying.
        (-1, true, true)
    };
    Ok(TargetFile {
        perm,
        device,
        newfile,
        readonly: false,
        made_writable: false,
    })
}

/// Pre-flight for a write: what is at `fname`, and may we write over it?
///
/// `Err(None)` is the user declining the "file has changed since reading it"
/// prompt — a failure with nothing left to report.
pub(crate) fn get_fileinfo(
    buffer: Buf,
    fname: &CStr,
    overwriting: bool,
    forceit: bool,
    file_info_old: &mut FileInfo,
) -> Result<TargetFile, Option<WriteError>> {
    let mut target = get_fileinfo_os(fname, file_info_old).map_err(Some)?;
    if target.device || target.newfile {
        return Ok(target);
    }

    // Check now whether the file is really writable: renaming it to make
    // a backup would otherwise hide the problem until it is too late.
    target.readonly = os_file_is_writable(fname) == 0;
    if !forceit && target.readonly {
        return Err(Some(if cpo_has(CpoFlag::FWRITE) {
            WriteError::numbered(c"E504", E_READONLY_CPO)
        } else {
            WriteError::numbered(c"E505", c"is read-only (add ! to override)")
        }));
    }
    // Without `!`, check the timestamp has not changed since the read.
    if overwriting && !forceit && !check_mtime(buffer, file_info_old) {
        return Err(None);
    }
    Ok(target)
}

/// Build the backup file name for one entry `dir` of `'backupdir'`; `last`
/// says it is the list's last, which is the one a missing directory is
/// made for.
pub(crate) fn backup_name(
    fname: &CStr,
    dir: &[u8],
    last: bool,
    no_prepend_dot: bool,
    backup_ext: &[u8],
) -> Option<XString> {
    if last
        && let Ok(dir_name) = CString::new(dir)
        && !dir_exists(&dir_name)
    {
        // That was the last entry, and it names a directory that does not
        // exist yet.
        if let Err(failure) = mkdir_recurse(&dir_name, 0o755) {
            let failed = failure.dir.as_deref().unwrap_or(c"");
            let (failed, why) = (msg_cstr(failed), msg_cstr(failure.why));
            semsg!("E303: Unable to create directory \"{failed}\" for backup file: {why}");
        }
    }

    let len = dir.len();
    if len > 1 && after_pathsep(dir, len) && dir[len - 1] == dir[len - 2] {
        // The entry ends with '//': encode the file's full path into the
        // backup's name so two files never collide.
        let mangled = percent_swname(&dir[..len - 1], Some(fname));
        if let Some(backup) = modname(Some(mangled.as_cstr()), backup_ext, no_prepend_dot) {
            return Some(backup);
        }
    }
    let rootname = file_in_dir(fname.to_bytes(), dir);
    modname(Some(rootname.as_cstr()), backup_ext, no_prepend_dot)
}

/// Walk `'backupdir'` (a copy: the walk runs code that may set it), calling
/// `each` with the backup name each entry gives until it answers true.
fn each_backup_dir(fname: &CStr, backup_ext: &[u8], mut each: impl FnMut(XString) -> bool) {
    let bdir = P_BDIR.get();
    let mut rest: &[u8] = &bdir;
    while !rest.is_empty() {
        let (dir, after) = split_option_part(rest, IOSIZE as usize, b",");
        rest = after;
        if let Some(backup) = backup_name(fname, &dir, rest.is_empty(), false, backup_ext)
            && each(backup)
        {
            return;
        }
    }
}

/// Should the backup be a copy of the original rather than a rename of it?
///
/// A rename is cheaper, but it changes the original's inode, so it is wrong
/// for a file that is a link or that we could not recreate in place.
fn want_backup_copy(
    fname: &CStr,
    file_info_old: &FileInfo,
    perm: c_int,
    bkc: c_uint,
    append: bool,
) -> bool {
    if bkc & kOptBkcFlagYes as c_uint != 0 || append {
        return true;
    }
    if bkc & kOptBkcFlagAuto as c_uint == 0 {
        return false;
    }

    // "auto": don't rename the file when it is a hard link, a symbolic
    // link, or in a directory we have no write permission in.
    if file_info_old.stat.st_nlink > 1
        || link_info(fname).is_none_or(|link| !link.same_file_as(file_info_old))
    {
        return true;
    }

    // Can we create a file here and give it the original's owner and
    // group? Find a name that does not exist yet (some arbitrary
    // numbers) and try it out.
    let dir = &fname.to_bytes()[..tail_index(fname.to_bytes())];
    let mut i: c_int = 4913;
    let tmp_fname = loop {
        let mut name = dir.to_vec();
        name.extend_from_slice(i.to_string().as_bytes());
        let name = CString::new(name).expect("a file name holds no NUL");
        if link_info(&name).is_none() {
            break name;
        }
        i += 123;
    };

    let mut options = OpenOptions::new();
    options.create_new(true).custom_flags(O_NOFOLLOW);
    let Ok(file) = open_for_write(&tmp_fname, &mut options, perm) else {
        return true; // can't write in this directory
    };
    let (uid, gid) = (file_info_old.stat.st_uid, file_info_old.stat.st_gid);
    let _ = std::os::unix::fs::fchown(&file, Some(uid as uv_uid_t), Some(gid as uv_gid_t));
    let copy = file_info(&tmp_fname).is_none_or(|info| {
        info.stat.st_uid != uid || info.stat.st_gid != gid || info.stat.st_mode as c_int != perm
    });
    // Close before removing: on Windows an open file cannot be deleted.
    drop(file);
    os_remove(&tmp_fname);
    copy
}

/// Does `'backupcopy'` want the link at `fname` broken?
///
/// Renaming the original out of the way is exactly how that happens, so this
/// overrides the choice [`want_backup_copy`] made.
fn breaks_link(fname: &CStr, file_info_old: &FileInfo, bkc: c_uint) -> bool {
    let link = link_info(fname);
    let same = link
        .as_ref()
        .is_some_and(|link| link.same_file_as(file_info_old));
    let symlink = bkc & kOptBkcFlagBreaksymlink as c_uint != 0 && link.is_some() && !same;
    let hardlink = bkc & kOptBkcFlagBreakhardlink as c_uint != 0
        && file_info_old.stat.st_nlink > 1
        && (link.is_none() || same);
    symlink || hardlink
}

/// Step the character just before the extension down from `z`, looking for a
/// backup name that is not taken. False once they are all taken.
///
/// Used when the backup is not being kept: an existing file with the wanted
/// name is somebody else's and must not be deleted.
///
/// `stat` receives the stat of the last name tried, because the copy path
/// reads it afterwards; `None` tests existence only.
fn step_backup_name(
    backup: &mut XString,
    backup_ext: &[u8],
    mut stat: Option<&mut FileInfo>,
) -> bool {
    // Upstream computes `backup + strlen(backup) - 1 - strlen(ext)` and
    // clamps it up to `backup`; the clamp is what makes an empty name
    // safe, so do the arithmetic where it cannot go below zero.
    let mut name = core::mem::take(backup).into_vec();
    let at = (name.len() - 1).saturating_sub(1 + backup_ext.len());
    name[at] = b'z';
    while name[at] > b'a' {
        let path = cstr::in_bytes(&name);
        let taken = match stat.as_deref_mut() {
            Some(info) => {
                let found = file_info(path);
                let taken = found.is_some();
                *info = found.unwrap_or_default();
                taken
            }
            None => path_exists(path),
        };
        if !taken {
            break;
        }
        name[at] -= 1;
    }
    let free = name[at] != b'a';
    name.pop(); // the terminator `into_vec` kept
    *backup = XString::from_vec(name);
    free
}

/// Make the backup file for a write.
///
/// The backup may come back empty: with `!` a failure to make one is not
/// worth stopping the write for.
pub(crate) fn buf_write_make_backup(
    fname: &CStr,
    file_info_old: &FileInfo,
    target: &TargetFile,
    acl: VimAcl,
    bkc: c_uint,
    append: bool,
    forceit: bool,
) -> Result<Backup, WriteError> {
    let perm = target.perm;
    let mut copy = want_backup_copy(fname, file_info_old, perm, bkc, append);
    if bkc & (kOptBkcFlagBreaksymlink as c_uint | kOptBkcFlagBreakhardlink as c_uint) != 0
        && breaks_link(fname, file_info_old, bkc)
    {
        copy = false;
    }

    // Make sure there is a valid backup extension to use. A copy, because
    // the backup below writes a file and may report through the message
    // machinery.
    let bex = P_BEX.get();
    let backup_ext = if bex.is_empty() { &b".bak"[..] } else { &bex };

    let path = if copy {
        backup_by_copy(fname, file_info_old, acl, perm, backup_ext, forceit)?
    } else {
        backup_by_rename(fname, target, backup_ext, forceit)?
    };
    Ok(Backup { copy, path })
}

/// Make the backup by copying the original aside.
///
/// Unix semantics has it that a writable file may not be recreatable with a
/// plain `open(..., O_CREAT)`: the directory may not be writable, the file
/// may be a symbolic link, it may belong to another user. So the existing
/// file is truncated and reused, and the backup is a copy.
fn backup_by_copy(
    fname: &CStr,
    file_info_old: &FileInfo,
    acl: VimAcl,
    perm: c_int,
    backup_ext: &[u8],
    forceit: bool,
) -> Result<Option<XString>, WriteError> {
    let cannot_create =
        || WriteError::plain(c"E509: Cannot create backup file (add ! to override)");
    let mut err = None;
    let mut made = None;

    // Try to make the backup in each directory in 'backupdir'.
    each_backup_dir(fname, backup_ext, |mut backup| {
        // A stat that fails leaves `file_info_new` zeroed, so it is defined
        // below whether or not the backup file already existed.
        let mut file_info_new = FileInfo::default();
        if let Some(found) = file_info(backup.as_cstr()) {
            if found.same_file_as(file_info_old) {
                // The backup file *is* the original file: modname() gave
                // the same name back (a silly link, say). Copying onto it
                // would ruin the file, and erasing it afterwards would
                // erase the file.
                return false;
            }
            file_info_new = found;
            if !p_bk() && !step_backup_name(&mut backup, backup_ext, Some(&mut file_info_new)) {
                // Not keeping the backup, so an existing one must not be
                // deleted — and every alternative name is taken too.
                return false;
            }
        }

        let path = backup.as_cstr();
        os_remove(path); // remove an old backup, if present
        if os_copy(fname, path, UV_FS_COPYFILE_FICLONE) != 0 {
            err = Some(cannot_create());
            return false;
        }

        // Same protection as the original file, minus the s-bit.
        os_setperm(path, perm & 0o777);
        // Try to give the backup the original's group. Failing that, give
        // the group the same bits as others.
        let gid = file_info_old.stat.st_gid;
        if file_info_new.stat.st_gid != gid
            && os_chown(path, -1i32 as uv_uid_t, gid as uv_gid_t) != 0
        {
            os_setperm(path, (perm & 0o707) | ((perm & 0o7) << 3));
        }
        let atime = file_info_old.stat.st_atim.tv_sec as f64;
        let mtime = file_info_old.stat.st_mtim.tv_sec as f64;
        os_file_settime(path, atime, mtime);
        os_set_acl(path, acl);
        copy_xattr(fname, path);
        err = None;
        made = Some(backup);
        true
    });

    if made.is_none() && err.is_none() {
        err = Some(cannot_create());
    }
    // Errors are ignored when forceit is true.
    if !forceit && let Some(err) = err {
        return Err(err);
    }
    Ok(made)
}

/// Make the backup by renaming the original out of the way.
///
/// For safety the backup is not removed until the write has finished
/// successfully — and if 'backup' is set, not even then.
fn backup_by_rename(
    fname: &CStr,
    target: &TargetFile,
    backup_ext: &[u8],
    forceit: bool,
) -> Result<Option<XString>, WriteError> {
    // 'cpoptions' "W" means not overwriting a read-only file. A rename
    // may be possible anyway, so it needs its own check here.
    if target.readonly && cpo_has(CpoFlag::FWRITE) {
        return Err(WriteError::numbered(c"E504", E_READONLY_CPO));
    }

    // path/fo.o.h becomes path/fo.o.h.bak, in the first directory of
    // 'backupdir' that works.
    let mut made = None;
    each_backup_dir(fname, backup_ext, |mut backup| {
        if !p_bk()
            && path_exists(backup.as_cstr())
            && !step_backup_name(&mut backup, backup_ext, None)
        {
            return false;
        }
        if rename_file(fname, backup.as_cstr()) {
            made = Some(backup);
            return true;
        }
        false // don't do the rename below
    });
    if made.is_none() && !forceit {
        return Err(WriteError::plain(
            c"E510: Can't make backup file (add ! to override)",
        ));
    }
    Ok(made)
}

/// Put the original file back after failing to open it for writing.
///
/// The backup is not needed then, so it is thrown away — but if the original
/// was moved or removed to make it, it goes back first. Returns false when
/// the original is gone anyway, which makes the write a total loss.
///
/// `direct` says the write went to `fname` itself rather than a temp file.
pub(crate) fn restore_backup(backup: &Backup, fname: &CStr, direct: bool, newfile: bool) -> bool {
    if let Some(path) = &backup.path
        && direct
    {
        let path = path.as_cstr();
        if backup.copy {
            // There is a small chance the original was removed; try to
            // move the copy into its place. That may fail, in which case
            // the copy is left where it is.
            if !path_exists(fname) {
                rename_file(path, fname);
            }
            // If the original does exist, throw the copy away.
            if path_exists(fname) {
                os_remove(path);
            }
        } else {
            rename_file(path, fname);
        }
    }
    newfile || path_exists(fname)
}

/// Put the backup back over the file a failed write left behind.
///
/// The new file is probably corrupt, and writing again would otherwise make
/// a backup *of the corrupt file* and lose the original for good. True when
/// the original is back, which spares the user the extra warning.
pub(crate) fn recover_from_backup(backup: &Backup, fname: &CStr) -> bool {
    let Some(path) = &backup.path else {
        return false;
    };
    if !backup.copy {
        return rename_file(path.as_cstr(), fname);
    }
    // Copying may take a while; if we were interrupted, let the user
    // know the message arrived.
    if got_int.get() {
        msg(gettext(e_interr), 0);
        ui_flush();
    }
    os_copy(path.as_cstr(), fname, UV_FS_COPYFILE_FICLONE) == 0
}

/// `'patchmode'`: keep the file as it was before the write, under a name of
/// its own, so that a patch can be generated later.
///
/// The backup already *is* the original, so it only has to be renamed. With
/// no backup — the file did not exist — an empty file records that.
pub(crate) fn apply_patchmode(
    fname: &CStr,
    backup: &mut Backup,
    perm: c_int,
    file_info_old: &FileInfo,
) {
    let org = p_pm(|value| modname(Some(fname), value.to_bytes(), false));
    if let Some(path) = &backup.path {
        match &org {
            None => {
                emsg(translate(c"E205: Patchmode: can't save original file"));
            }
            Some(org) if !path_exists(org.as_cstr()) => {
                rename_file(path.as_cstr(), org.as_cstr());
                backup.path = None; // don't delete the file
                let atime = file_info_old.stat.st_atim.tv_sec as f64;
                let mtime = file_info_old.stat.st_mtim.tv_sec as f64;
                os_file_settime(org.as_cstr(), atime, mtime);
            }
            Some(_) => {}
        }
    } else {
        // No backup, so remember that a (new) file was created.
        let mode = if perm < 0 { 0o666 } else { perm & 0o777 };
        let touched = org.as_ref().and_then(|org| {
            let mut options = OpenOptions::new();
            options.create_new(true).custom_flags(O_NOFOLLOW);
            open_for_write(org.as_cstr(), &mut options, mode).ok()
        });
        if touched.is_none() {
            emsg(translate(
                c"E206: Patchmode: can't touch empty original file",
            ));
        }
    }
    if let Some(org) = &org {
        os_setperm(org.as_cstr(), getperm(fname) & 0o777);
    }
}

/// Open the file to write to.
///
/// It may be opened twice: if the file cannot be written and `!` was given,
/// the existing one is removed and a new one created. Should that fail too,
/// the original may be lost — which happens when the user has reached their
/// file quota. Appending fails if the file does not exist and `!` was not
/// given.
///
/// `None` means giving up, with the reason in `err`.
pub(crate) fn open_write_file(
    wfname: &CStr,
    fname: &CStr,
    target: &mut TargetFile,
    file_info_old: &FileInfo,
    req: WriteRequest,
    err: &mut Option<WriteError>,
) -> Option<File> {
    let options = || {
        let mut options = OpenOptions::new();
        if req.append {
            options.append(true).create(req.forceit);
        } else {
            options.create(true).truncate(true);
        }
        options
    };
    // Deliberately computed once: the retry below raises `target.perm`
    // for the later chmod, not for this open.
    let mode = if target.perm < 0 {
        0o666
    } else {
        target.perm & 0o777
    };
    loop {
        let code = match open_for_write(wfname, &mut options(), mode) {
            Ok(file) => return Some(file),
            Err(code) => code,
        };
        if err.is_some() {
            // The retry failed as well; report why the first try did.
            return None;
        }

        // Don't delete the file when it is a hard or symbolic link.
        if (!target.newfile && file_info_old.stat.st_nlink > 1)
            || link_info(fname).is_some_and(|link| !link.same_file_as(file_info_old))
        {
            *err = Some(WriteError::plain(
                c"E166: Can't open linked file for writing",
            ));
            return None;
        }
        *err = Some(WriteError::errno(
            c"E212: Can't open file for writing: %s",
            code,
        ));
        if !(req.forceit && !cpo_has(CpoFlag::FWRITE) && target.perm >= 0) {
            return None;
        }
        // We are going to write to the file after all, so it should be
        // marked writable.
        if target.perm & 0o200 == 0 {
            target.made_writable = true;
        }
        target.perm |= 0o200;
        if file_info_old.stat.st_uid != uint64_t::from(os_getuid())
            || file_info_old.stat.st_gid != uint64_t::from(os_getgid())
        {
            target.perm &= 0o777;
        }
        if !req.append {
            os_remove(wfname); // don't remove when appending
        }
    }
}

/// Sync and close the file just written, and give it the original's
/// ownership, permissions and ACL.
pub(crate) fn finish_write(
    buffer: Buf,
    file: File,
    wfname: &CStr,
    target: &TargetFile,
    backup: &Backup,
    acl: VimAcl,
    file_info_old: &FileInfo,
) -> Option<WriteError> {
    let mut err = None;
    // Many journalling file systems lose both the original and the
    // backup when the system halts right after a write, because only the
    // meta-data is journalled. Syncing slows the system down but assures
    // the data reached the disk. For a device the fsync is attempted but
    // not complained about; it could be a pipe.
    let fsync = if buffer.b_p_fs >= 0 {
        buffer.b_p_fs != 0
    } else {
        p_fs()
    };
    if fsync {
        // UV_ENOTSUP is "this storage does not do fsync".
        if let Err(error) = fsync_file(&file)
            && error != UV_ENOTSUP
            && !target.device
        {
            err = Some(WriteError::shared(e_fsync, error));
        }
    }

    if !backup.copy
        && let Some(path) = &backup.path
    {
        copy_xattr(path.as_cstr(), wfname);
    }
    if backup.path.is_some() && !backup.copy {
        // The original was renamed away, so this is a new file: give it
        // the original's owner and group. Not when they are already
        // right — some systems drop permission or ACL data over it.
        let (uid, gid) = (file_info_old.stat.st_uid, file_info_old.stat.st_gid);
        if file_info(wfname).is_none_or(|info| info.stat.st_uid != uid || info.stat.st_gid != gid) {
            let _ = std::os::unix::fs::fchown(&file, Some(uid as uv_uid_t), Some(gid as uv_gid_t));
            if target.perm >= 0 {
                os_setperm(wfname, target.perm); // may have changed
            }
        }
        buf_set_file_id(buffer);
    } else if !buffer.file_id_valid {
        buf_set_file_id(buffer); // the file is new
    }

    if let Err(error) = close_file(file) {
        err = Some(WriteError::errno(c"E512: Close failed: %s", error));
    }

    let mut perm = target.perm;
    if target.made_writable {
        perm &= !0o200; // reset the 'w' bit for security reasons
    }
    if perm >= 0 {
        os_setperm(wfname, perm); // same permissions as the old file
    }
    // The ACL goes on before the user changes: an ACL cannot be set on a
    // file the user does not own.
    if !backup.copy {
        os_set_acl(wfname, acl);
    }
    err
}
