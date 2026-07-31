//! The backup file, and deciding whether the write may happen at all.
//!
//! [`get_fileinfo`] is the pre-flight: does the target exist, is it a regular
//! file, is it writable, and has it changed on disk since we read it.
//! [`buf_write_make_backup`] then makes the backup that `'backup'`,
//! `'writebackup'` and `'patchmode'` ask for — either by copying the original
//! aside or by renaming it out of the way, which `'backupcopy'` chooses
//! between. [`buf_get_backup_name`] is where `'backupdir'` and `'backupext'`
//! turn into a path.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};

#[allow(unused_imports)]
use super::*;

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
}

/// The backup a write left behind, if any.
pub(crate) struct Backup {
    /// The backup is a *copy*; the original file is still where it was.
    /// When false the original was renamed to the backup, so putting it
    /// back means renaming it back.
    pub copy: bool,
    /// Path to the backup, or null when `!` overrode a failure to make one.
    /// Owned: `buf_write` frees it.
    pub path: *mut c_char,
}

/// Warn about writing over a file that changed on disk since it was read.
///
/// The size is not checked: a tool like `gzip` keeps the timestamp but
/// cannot keep the size. Returns false if the user answers "no".
unsafe fn check_mtime(buf: *mut buf_T, file_info: *mut FileInfo) -> bool {
    unsafe {
        if (*buf).b_mtime_read == 0
            || !time_differs(file_info, (*buf).b_mtime_read, (*buf).b_mtime_read_ns)
        {
            return true;
        }
        msg_scroll.set(1); // don't overwrite messages here
        msg_silent.set(0); // must give this prompt
        // Not emsg(): that would flush the buffers.
        msg(
            translate(c"WARNING: The file has been changed since reading it!!!").as_ptr(),
            HLF_E as c_int,
        );
        if ask_yesno(translate(c"Do you really want to write to it").as_ptr()) == 'n' as c_int {
            return false;
        }
        msg_scroll.set(0); // always overwrite the file message now
        true
    }
}

/// The Unix half of the pre-flight: stat the target and classify it.
unsafe fn get_fileinfo_os(
    fname: *mut c_char,
    file_info_old: *mut FileInfo,
) -> Result<TargetFile, WriteError> {
    unsafe {
        let exists = os_fileinfo(fname, file_info_old);
        let mode = (*file_info_old).stat.st_mode;
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
        })
    }
}

/// Pre-flight for a write: what is at `fname`, and may we write over it?
///
/// `Err(None)` is the user declining the "file has changed since reading it"
/// prompt — a failure with nothing left to report.
pub(crate) unsafe fn get_fileinfo(
    buf: *mut buf_T,
    fname: *mut c_char,
    overwriting: bool,
    forceit: bool,
    file_info_old: *mut FileInfo,
) -> Result<TargetFile, Option<WriteError>> {
    unsafe {
        let mut target = get_fileinfo_os(fname, file_info_old).map_err(Some)?;
        if target.device || target.newfile {
            return Ok(target);
        }

        // Check now whether the file is really writable: renaming it to make
        // a backup would otherwise hide the problem until it is too late.
        target.readonly = os_file_is_writable(fname) == 0;
        if !forceit && target.readonly {
            return Err(Some(if !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null() {
                WriteError::numbered(c"E504", E_READONLY_CPO)
            } else {
                WriteError::numbered(c"E505", c"is read-only (add ! to override)")
            }));
        }
        // Without `!`, check the timestamp has not changed since the read.
        if overwriting && !forceit && !check_mtime(buf, file_info_old) {
            return Err(None);
        }
        Ok(target)
    }
}

/// Build the backup file name for one entry of `'backupdir'`.
///
/// `dirp` is advanced past the entry used, so a caller walks the option by
/// calling this until `**dirp` is NUL. The result is allocated.
pub(crate) unsafe fn buf_get_backup_name(
    fname: *mut c_char,
    dirp: &mut *mut c_char,
    no_prepend_dot: bool,
    backup_ext: *mut c_char,
) -> *mut c_char {
    unsafe {
        let iobuff = IObuff.ptr() as *mut c_char;
        // Isolate one directory name from 'backupdir'.
        let dir_len = copy_option_part(dirp, iobuff, IOSIZE as size_t, c",".as_ptr().cast_mut());
        let mut p = iobuff.add(dir_len as usize);
        if **dirp as c_int == NUL && !os_isdir(iobuff) {
            // That was the last entry, and it names a directory that does not
            // exist yet.
            let mut failed_dir: *mut c_char = core::ptr::null_mut();
            let ret = os_mkdir_recurse(iobuff, 0o755, &raw mut failed_dir, core::ptr::null_mut());
            if ret != 0 {
                semsg(
                    translate(c"E303: Unable to create directory \"%s\" for backup file: %s")
                        .as_ptr(),
                    failed_dir,
                    uv_strerror(ret),
                );
                xfree(failed_dir.cast());
            }
        }

        let mut backup = core::ptr::null_mut();
        if dir_len > 1 && after_pathsep(iobuff, p) != 0 && *p.offset(-1) == *p.offset(-2) {
            // The entry ends with '//': encode the file's full path into the
            // backup's name so two files never collide.
            p = make_percent_swname(iobuff, p, fname);
            if !p.is_null() {
                backup = modname(p, backup_ext, no_prepend_dot);
                xfree(p.cast());
            }
        }
        if backup.is_null() {
            let rootname = get_file_in_dir(fname, iobuff);
            if !rootname.is_null() {
                backup = modname(rootname, backup_ext, no_prepend_dot);
                xfree(rootname.cast());
            }
        }
        backup
    }
}

/// Should the backup be a copy of the original rather than a rename of it?
///
/// A rename is cheaper, but it changes the original's inode, so it is wrong
/// for a file that is a link or that we could not recreate in place.
unsafe fn want_backup_copy(
    fname: *mut c_char,
    file_info_old: *mut FileInfo,
    perm: c_int,
    bkc: c_uint,
    append: bool,
) -> bool {
    unsafe {
        if bkc & kOptBkcFlagYes as c_uint != 0 || append {
            return true;
        }
        if bkc & kOptBkcFlagAuto as c_uint == 0 {
            return false;
        }

        // "auto": don't rename the file when it is a hard link, a symbolic
        // link, or in a directory we have no write permission in.
        let mut file_info = FileInfo::default();
        if os_fileinfo_hardlinks(file_info_old) > 1
            || !os_fileinfo_link(fname, &raw mut file_info)
            || !os_fileinfo_id_equal(&raw mut file_info, file_info_old)
        {
            return true;
        }

        // Can we create a file here and give it the original's owner and
        // group? Find a name that does not exist yet (some arbitrary
        // numbers) and try it out.
        let dirlen = path_tail(fname).offset_from(fname) as usize;
        assert!(dirlen < MAXPATHL as usize);
        let mut tmp_fname = [0 as c_char; MAXPATHL as usize];
        xmemcpyz(
            tmp_fname.as_mut_ptr().cast(),
            fname.cast(),
            dirlen as size_t,
        );
        let mut i: c_int = 4913;
        loop {
            snprintf(
                tmp_fname.as_mut_ptr().add(dirlen),
                tmp_fname.len() - dirlen,
                c"%d".as_ptr(),
                i,
            );
            if !os_fileinfo_link(tmp_fname.as_mut_ptr(), &raw mut file_info) {
                break;
            }
            i += 123;
        }

        let fd = os_open(
            tmp_fname.as_mut_ptr(),
            O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW,
            perm,
        );
        if fd < 0 {
            return true; // can't write in this directory
        }
        os_fchown(
            fd,
            (*file_info_old).stat.st_uid as uv_uid_t,
            (*file_info_old).stat.st_gid as uv_gid_t,
        );
        let copy = !os_fileinfo(tmp_fname.as_mut_ptr(), &raw mut file_info)
            || file_info.stat.st_uid != (*file_info_old).stat.st_uid
            || file_info.stat.st_gid != (*file_info_old).stat.st_gid
            || file_info.stat.st_mode as c_int != perm;
        // Close before removing: on Windows an open file cannot be deleted.
        close(fd);
        os_remove(tmp_fname.as_mut_ptr());
        copy
    }
}

/// Does `'backupcopy'` want the link at `fname` broken?
///
/// Renaming the original out of the way is exactly how that happens, so this
/// overrides the choice [`want_backup_copy`] made.
unsafe fn breaks_link(fname: *mut c_char, file_info_old: *mut FileInfo, bkc: c_uint) -> bool {
    unsafe {
        let mut file_info = FileInfo::default();
        let link_ok = os_fileinfo_link(fname, &raw mut file_info);
        let symlink = bkc & kOptBkcFlagBreaksymlink as c_uint != 0
            && link_ok
            && !os_fileinfo_id_equal(&raw mut file_info, file_info_old);
        let hardlink = bkc & kOptBkcFlagBreakhardlink as c_uint != 0
            && os_fileinfo_hardlinks(file_info_old) > 1
            && (!link_ok || os_fileinfo_id_equal(&raw mut file_info, file_info_old));
        symlink || hardlink
    }
}

/// Step the character just before the extension down from `z`, looking for a
/// backup name that is not taken. False once they are all taken.
///
/// Used when the backup is not being kept: an existing file with the wanted
/// name is somebody else's and must not be deleted.
///
/// `stat` receives the stat of the last name tried, because the copy path
/// reads it afterwards; `None` tests existence only.
unsafe fn step_backup_name(
    backup: *mut c_char,
    backup_ext: *mut c_char,
    mut stat: Option<&mut FileInfo>,
) -> bool {
    unsafe {
        // Upstream computes `backup + strlen(backup) - 1 - strlen(ext)` and
        // clamps it up to `backup`; the clamp is what makes an empty name
        // safe, so do the arithmetic where it cannot go below zero.
        let at = strlen(backup).saturating_sub(1 + strlen(backup_ext) as usize);
        let p = backup.add(at);
        *p = b'z' as c_char;
        while *p > b'a' as c_char
            && match stat.as_deref_mut() {
                Some(file_info) => os_fileinfo(backup, file_info),
                None => os_path_exists(backup),
            }
        {
            *p -= 1;
        }
        *p != b'a' as c_char
    }
}

/// Make the backup file for a write.
///
/// The backup may come back empty: with `!` a failure to make one is not
/// worth stopping the write for.
pub(crate) unsafe fn buf_write_make_backup(
    fname: *mut c_char,
    file_info_old: *mut FileInfo,
    target: &TargetFile,
    acl: vim_acl_T,
    bkc: c_uint,
    append: bool,
    forceit: bool,
) -> Result<Backup, WriteError> {
    unsafe {
        let perm = target.perm;
        let mut copy = want_backup_copy(fname, file_info_old, perm, bkc, append);
        if bkc & (kOptBkcFlagBreaksymlink as c_uint | kOptBkcFlagBreakhardlink as c_uint) != 0
            && breaks_link(fname, file_info_old, bkc)
        {
            copy = false;
        }

        // Make sure there is a valid backup extension to use.
        let backup_ext = if *p_bex.get() as c_int == NUL {
            c".bak".as_ptr().cast_mut()
        } else {
            p_bex.get()
        };

        let path = if copy {
            backup_by_copy(fname, file_info_old, acl, perm, backup_ext, forceit)?
        } else {
            backup_by_rename(fname, target, backup_ext, forceit)?
        };
        Ok(Backup { copy, path })
    }
}

/// Make the backup by copying the original aside.
///
/// Unix semantics has it that a writable file may not be recreatable with a
/// plain `open(..., O_CREAT)`: the directory may not be writable, the file
/// may be a symbolic link, it may belong to another user. So the existing
/// file is truncated and reused, and the backup is a copy.
unsafe fn backup_by_copy(
    fname: *mut c_char,
    file_info_old: *mut FileInfo,
    acl: vim_acl_T,
    perm: c_int,
    backup_ext: *mut c_char,
    forceit: bool,
) -> Result<*mut c_char, WriteError> {
    unsafe {
        let cannot_create =
            || WriteError::plain(c"E509: Cannot create backup file (add ! to override)");
        let mut err = None;
        let mut backup: *mut c_char = core::ptr::null_mut();

        // Try to make the backup in each directory in 'backupdir'.
        let mut dirp = p_bdir.get();
        while *dirp != 0 {
            backup = buf_get_backup_name(fname, &mut dirp, false, backup_ext);
            if backup.is_null() {
                break; // out of memory
            }

            // `os_fileinfo` zeroes what it is given even when it fails, so
            // `file_info_new` is defined below whether or not the backup file
            // already existed.
            let mut file_info_new = FileInfo::default();
            if os_fileinfo(backup, &raw mut file_info_new) {
                if os_fileinfo_id_equal(&raw mut file_info_new, file_info_old) {
                    // The backup file *is* the original file: modname() gave
                    // the same name back (a silly link, say). Copying onto it
                    // would ruin the file, and erasing it afterwards would
                    // erase the file.
                    xfree(backup.cast());
                    backup = core::ptr::null_mut();
                } else if p_bk.get() == 0
                    && !step_backup_name(backup, backup_ext, Some(&mut file_info_new))
                {
                    // Not keeping the backup, so an existing one must not be
                    // deleted — and every alternative name is taken too.
                    xfree(backup.cast());
                    backup = core::ptr::null_mut();
                }
            }
            if backup.is_null() {
                continue;
            }

            os_remove(backup); // remove an old backup, if present
            if os_copy(fname, backup, UV_FS_COPYFILE_FICLONE) != 0 {
                err = Some(cannot_create());
                xfree(backup.cast());
                backup = core::ptr::null_mut();
                continue;
            }

            // Same protection as the original file, minus the s-bit.
            os_setperm(backup, perm & 0o777);
            // Try to give the backup the original's group. Failing that, give
            // the group the same bits as others.
            if file_info_new.stat.st_gid != (*file_info_old).stat.st_gid
                && os_chown(
                    backup,
                    -1i32 as uv_uid_t,
                    (*file_info_old).stat.st_gid as uv_gid_t,
                ) != 0
            {
                os_setperm(backup, (perm & 0o707) | ((perm & 0o7) << 3));
            }
            os_file_settime(
                backup,
                (*file_info_old).stat.st_atim.tv_sec as f64,
                (*file_info_old).stat.st_mtim.tv_sec as f64,
            );
            os_set_acl(backup, acl);
            os_copy_xattr(fname, backup);
            err = None;
            break;
        }

        if backup.is_null() && err.is_none() {
            err = Some(cannot_create());
        }
        // Errors are ignored when forceit is true.
        if !forceit && let Some(err) = err {
            return Err(err);
        }
        Ok(backup)
    }
}

/// Make the backup by renaming the original out of the way.
///
/// For safety the backup is not removed until the write has finished
/// successfully — and if 'backup' is set, not even then.
unsafe fn backup_by_rename(
    fname: *mut c_char,
    target: &TargetFile,
    backup_ext: *mut c_char,
    forceit: bool,
) -> Result<*mut c_char, WriteError> {
    unsafe {
        // 'cpoptions' "W" means not overwriting a read-only file. A rename
        // may be possible anyway, so it needs its own check here.
        if target.readonly && !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null() {
            return Err(WriteError::numbered(c"E504", E_READONLY_CPO));
        }

        // path/fo.o.h becomes path/fo.o.h.bak, in the first directory of
        // 'backupdir' that works.
        let mut backup: *mut c_char = core::ptr::null_mut();
        let mut dirp = p_bdir.get();
        while *dirp != 0 {
            backup = buf_get_backup_name(fname, &mut dirp, false, backup_ext);
            if !backup.is_null()
                && p_bk.get() == 0
                && os_path_exists(backup)
                && !step_backup_name(backup, backup_ext, None)
            {
                xfree(backup.cast());
                backup = core::ptr::null_mut();
            }
            if backup.is_null() {
                continue;
            }
            if vim_rename(fname, backup) == 0 {
                break;
            }
            xfree(backup.cast()); // don't do the rename below
            backup = core::ptr::null_mut();
        }
        if backup.is_null() && !forceit {
            return Err(WriteError::plain(
                c"E510: Can't make backup file (add ! to override)",
            ));
        }
        Ok(backup)
    }
}
