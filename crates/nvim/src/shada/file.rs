//! The ShaDa file itself: where it lives, and reading and writing it.
//!
//! [`shada_filename`] works out which file to use — the `:rshada`/`:wshada`
//! argument, `-i`, `'shada'`'s `n` entry, or the state directory's
//! `shada/main.shada`.
//!
//! Writing is never done in place. The new file is built next to the old one
//! as `<name>.tmp.a`, merged from it (see `merge`), and only renamed over it
//! once it is complete, carrying the old file's permissions and — when Nvim
//! is running as root — its ownership. That is why a failure part-way
//! through leaves the old file untouched, and why the messages about it name
//! the temporary file the user may want to remove.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::os::uv_error::{UV_EEXIST, UV_ELOOP, UV_ENOENT};
use crate::{semsg_c, smsg_c};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use std::ffi::CString;

use super::*;
use crate::types::{FAIL, MAXPATHL, NUL, OK};

/// Whether the reader has nothing more to give.
pub(crate) unsafe fn file_eof(fp: *const FileDescriptor) -> bool {
    unsafe { (*fp).eof && (*fp).read_pos == (*fp).write_pos }
}

/// The descriptor behind a file, for the calls that want the number.
pub(crate) unsafe fn file_fd(fp: *const FileDescriptor) -> c_int {
    unsafe { (*fp).fd }
}

/// How many bytes can still be written into the file's own buffer.
pub(crate) unsafe fn file_space(fp: *mut FileDescriptor) -> size_t {
    unsafe {
        (*fp)
            .buffer
            .add(ARENA_BLOCK_SIZE as usize)
            .offset_from_unsigned((*fp).write_pos)
    }
}

/// Close a ShaDa file, saying so if that fails. `'fsync'` decides whether
/// the bytes are pushed to the platter first.
pub(crate) unsafe fn close_file(cookie: *mut FileDescriptor) {
    unsafe {
        let error = file_close(cookie, p_fs.get() != 0);
        if error != 0 {
            semsg_c!(
                gettext(c"E886: System error while closing ShaDa file: %s".as_ptr()),
                uv_strerror(error),
            );
        }
    }
}

/// The default ShaDa file, worked out once and remembered.
static default_shada_file: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// `<state directory>/shada/main.shada`.
unsafe fn shada_get_default_file() -> *const c_char {
    unsafe {
        if (*default_shada_file.ptr()).is_none() {
            let shada_dir = stdpaths_user_state_subpath(c"shada".as_ptr(), 0, false);
            let full = concat_fnames_realloc(shada_dir, c"main.shada".as_ptr(), true);
            *default_shada_file.ptr() = Some(CStr::from_ptr(full).to_owned());
            xfree(full.cast::<c_void>());
        }
        match &*default_shada_file.ptr() {
            Some(file) => file.as_ptr(),
            None => core::ptr::null(),
        }
    }
}

/// Which ShaDa file to use, or `None` if ShaDa is turned off.
///
/// `file` is a name the user gave, already expanded by the command line;
/// failing that it is `-i`'s (`p_shadafile`, where `NONE` means off), then
/// `'shada'`'s `n` entry, then the default. Only the last two go through
/// environment-variable expansion — anything the shell handed over has been
/// expanded already.
unsafe fn shada_filename(file: *const c_char) -> Option<CString> {
    unsafe {
        if !file.is_null() && *file != NUL as c_char {
            return Some(CStr::from_ptr(file).to_owned());
        }
        if !(*p_shadafile.ptr()).is_null() && *p_shadafile.get() != NUL as c_char {
            if strequal(p_shadafile.get(), c"NONE".as_ptr()) {
                return None; // "-i NONE" or "--clean"
            }
            return Some(CStr::from_ptr(p_shadafile.get()).to_owned());
        }

        let mut named = find_shada_parameter('n' as c_int);
        if named.is_null() || *named == NUL as c_char {
            named = shada_get_default_file().cast_mut();
        }
        let len = expand_env(named, NameBuff.ptr().cast::<c_char>(), MAXPATHL);
        let expanded = core::slice::from_raw_parts(NameBuff.ptr().cast::<u8>(), len);
        Some(CString::new(expanded).expect("shada: expanded file name holds a NUL"))
    }
}

/// Read a ShaDa file into the running editor.
///
/// `flags` says which parts of it are wanted; see the `kShaDa*` values.
unsafe fn shada_read_file(file: *const c_char, flags: c_int) -> c_int {
    unsafe {
        let Some(fname) = shada_filename(file) else {
            return FAIL;
        };
        let mut sd_reader: FileDescriptor = core::mem::zeroed();
        let of_ret = file_open(
            &raw mut sd_reader,
            fname.as_ptr(),
            kFileReadOnly as c_int,
            0,
        );

        if p_verbose.get() > 1 {
            verbose_enter();
            let note = |wanted: c_uint, text: &'static CStr| {
                if flags as c_uint & wanted != 0 {
                    gettext(text.as_ptr())
                } else {
                    c"".as_ptr()
                }
            };
            smsg_c!(
                0,
                gettext(c"Reading ShaDa file \"%s\"%s%s%s%s".as_ptr()),
                fname.as_ptr(),
                note(kShaDaWantInfo as c_uint, c" info"),
                note(kShaDaWantMarks as c_uint, c" marks"),
                note(kShaDaGetOldfiles as c_uint, c" oldfiles"),
                if of_ret != 0 {
                    gettext(c" FAILED".as_ptr())
                } else {
                    c"".as_ptr()
                },
            );
            verbose_leave();
        }

        if of_ret != 0 {
            // A missing file is only worth complaining about when the caller
            // asked for one by name.
            if of_ret != UV_ENOENT || flags & kShaDaMissingError as c_int != 0 {
                semsg_c!(
                    gettext(
                        c"E886: System error while opening ShaDa file %s for reading: %s".as_ptr(),
                    ),
                    fname.as_ptr(),
                    uv_strerror(of_ret),
                );
            }
            return FAIL;
        }

        shada_read(&raw mut sd_reader, flags);
        close_file(&raw mut sd_reader);
        OK
    }
}

/// Read the marks out of the default ShaDa file.
pub unsafe fn shada_read_marks() -> c_int {
    unsafe { shada_read_file(core::ptr::null(), kShaDaWantMarks as c_int) }
}

/// Read everything out of a ShaDa file.
///
/// `forceit` lets the file's contents win over the running editor's state;
/// `missing_ok` keeps quiet about a file that is not there.
pub unsafe fn shada_read_everything(
    fname: *const c_char,
    forceit: bool,
    missing_ok: bool,
) -> c_int {
    unsafe {
        let mut flags = kShaDaWantInfo as c_int
            | kShaDaWantMarks as c_int
            | kShaDaGetOldfiles as c_int
            | if missing_ok {
                0
            } else {
                kShaDaMissingError as c_int
            };
        if forceit {
            flags |= kShaDaForceit as c_int;
        }
        shada_read_file(fname, flags)
    }
}

/// The temporary file a merged ShaDa file is built in, opened.
///
/// It sits next to the real one and is named `.tmp.a` through `.tmp.z`;
/// another Nvim writing at the same moment holds one of those, so the
/// letters are tried in turn. `None` means every one of them was taken, or
/// the open failed for some other reason — which is reported here.
unsafe fn open_temp_writer(
    sd_writer: *mut FileDescriptor,
    fname: &CStr,
    perm: c_int,
) -> Option<CString> {
    unsafe {
        let tempname = modname(fname.as_ptr(), c".tmp.a".as_ptr(), false);
        if tempname.is_null() {
            return None;
        }
        let mut tempname = {
            let owned = CStr::from_ptr(tempname).to_owned();
            xfree(tempname.cast::<c_void>());
            owned.into_bytes_with_nul()
        };
        let last = tempname.len() - 2; // before the NUL

        loop {
            let error = file_open(
                sd_writer,
                tempname.as_ptr().cast::<c_char>(),
                kFileCreateOnly as c_int | kFileNoSymlink as c_int,
                perm,
            );
            if error == 0 {
                return Some(CString::from_vec_with_nul(tempname).expect("shada: a NUL crept in"));
            }
            if error != UV_EEXIST && error != UV_ELOOP {
                semsg_c!(
                    gettext(
                        c"E886: System error while opening temporary ShaDa file %s for writing: %s"
                            .as_ptr(),
                    ),
                    tempname.as_ptr(),
                    uv_strerror(error),
                );
                return None;
            }
            if tempname[last] == b'z' {
                semsg_c!(
                    gettext(c"E138: All %s.tmp.X files exist, cannot write ShaDa file!".as_ptr()),
                    fname.as_ptr(),
                );
                return None;
            }
            tempname[last] += 1;
        }
    }
}

/// Open the ShaDa file itself for writing, making the directory it lives in
/// if it is not there yet. Answers whether it is open.
unsafe fn open_direct_writer(sd_writer: *mut FileDescriptor, fname: &CStr) -> Result<bool, ()> {
    unsafe {
        // `path_tail_with_sep` points at the file name; NUL it out to get the
        // directory, then put the byte back.
        let fname = fname.as_ptr().cast_mut();
        let tail = path_tail_with_sep(fname);
        if tail != fname {
            let tail_save = *tail;
            *tail = NUL as c_char;
            let missing = !os_isdir(fname);
            let mut failed_dir = core::ptr::null_mut::<c_char>();
            let ret = if missing {
                os_mkdir_recurse(fname, 0o700, &raw mut failed_dir, core::ptr::null_mut())
            } else {
                0
            };
            *tail = tail_save;
            if ret != 0 {
                semsg_c!(
                    gettext(
                        c"E886: Failed to create directory %s for writing ShaDa file: %s".as_ptr(),
                    ),
                    failed_dir,
                    uv_strerror(ret),
                );
                xfree(failed_dir.cast::<c_void>());
                return Err(());
            }
        }

        let error = file_open(
            sd_writer,
            fname,
            kFileCreate as c_int | kFileTruncate as c_int,
            0o600,
        );
        if error != 0 {
            semsg_c!(
                gettext(c"E886: System error while opening ShaDa file %s for writing: %s".as_ptr()),
                fname,
                uv_strerror(error),
            );
            return Ok(false);
        }
        Ok(true)
    }
}

/// Write the ShaDa file.
///
/// `nomerge` writes this session's state alone; otherwise the existing file
/// is read and merged in first. Falling back to `nomerge` is normal — it is
/// what happens when there is no file yet.
pub unsafe fn shada_write_file(file: *const c_char, nomerge: bool) -> c_int {
    unsafe {
        let Some(fname) = shada_filename(file) else {
            return FAIL;
        };
        let mut sd_writer: FileDescriptor = core::mem::zeroed();
        let mut sd_reader: FileDescriptor = core::mem::zeroed();

        // The merge half: open the old file, then a temporary next to it.
        let mut merge = None;
        if !nomerge {
            let error = file_open(
                &raw mut sd_reader,
                fname.as_ptr(),
                kFileReadOnly as c_int,
                0,
            );
            if error != 0 {
                if error != UV_ENOENT {
                    // Something other than "no file yet" — say so, but still
                    // try to write: that may work regardless.
                    semsg_c!(
                        gettext(
                            c"E886: System error while opening ShaDa file %s for reading to merge before writing it: %s"
                                .as_ptr(),
                        ),
                        fname.as_ptr(),
                        uv_strerror(error),
                    );
                }
            } else {
                // The old file's permissions, less the setuid bit and with
                // read and write for its owner, so that the result is always
                // readable by whoever wrote it. If the file went away between
                // the open and here, start from u=rw.
                let perm = os_getperm(fname.as_ptr()) as c_int;
                let perm = if perm >= 0 {
                    (perm & 0o777) | 0o600
                } else {
                    0o600
                };
                match open_temp_writer(&raw mut sd_writer, &fname, perm) {
                    Some(tempname) => merge = Some(tempname),
                    None => {
                        close_file(&raw mut sd_reader);
                        return FAIL;
                    }
                }
            }
        }

        let reader = if merge.is_some() {
            &raw mut sd_reader
        } else {
            // No merge: the file itself is what gets written.
            match open_direct_writer(&raw mut sd_writer, &fname) {
                Err(()) => return FAIL,
                Ok(false) => return FAIL,
                Ok(true) => {}
            }
            core::ptr::null_mut()
        };

        if p_verbose.get() > 1 {
            verbose_enter();
            smsg_c!(
                0,
                gettext(c"Writing ShaDa file \"%s\"".as_ptr()),
                fname.as_ptr(),
            );
            verbose_leave();
        }

        let sw_ret = shada_write(&raw mut sd_writer, reader);
        debug_assert!(
            sw_ret != kSDWriteIgnError,
            "shada: an ignorable error reached the top of the write"
        );

        if let Some(tempname) = merge {
            close_file(&raw mut sd_reader);
            if !replace_original(&raw mut sd_writer, &fname, &tempname, sw_ret) {
                semsg_c!(
                    gettext(
                        c"E136: Do not forget to remove %s or rename it manually to %s.".as_ptr(),
                    ),
                    tempname.as_ptr(),
                    fname.as_ptr(),
                );
            }
        }
        close_file(&raw mut sd_writer);
        OK
    }
}

/// Move the finished temporary file over the real one. Answers whether the
/// temporary file is gone; if it is not, the caller says where it is.
unsafe fn replace_original(
    sd_writer: *mut FileDescriptor,
    fname: &CStr,
    tempname: &CStr,
    sw_ret: ShaDaWriteResult,
) -> bool {
    unsafe {
        if sw_ret != kSDWriteSuccessful {
            semsg_c!(
                gettext(if sw_ret == kSDWriteReadNotShada {
                    c"E136: Did not rename %s because %s does not look like a ShaDa file".as_ptr()
                } else {
                    c"E136: Did not rename %s to %s because there were errors during writing it"
                        .as_ptr()
                }),
                tempname.as_ptr(),
                fname.as_ptr(),
            );
            return false;
        }

        let mut old_info = FileInfo::default();
        if !os_fileinfo(fname.as_ptr(), &raw mut old_info) || !writable_by_us(&old_info) {
            semsg_c!(
                gettext(c"E137: ShaDa file is not writable: %s".as_ptr()),
                fname.as_ptr(),
            );
            return false;
        }

        // Running as root, the new file is given the old one's owner: it is
        // not friendly to leave a user with a ShaDa file they cannot read
        // after a `su root`.
        if getuid() == ROOT_UID as uid_t
            && (old_info.stat.st_uid != ROOT_UID as uint64_t
                || old_info.stat.st_gid != getgid() as uint64_t)
        {
            let fchown_ret = os_fchown(
                file_fd(sd_writer),
                old_info.stat.st_uid as uv_uid_t,
                old_info.stat.st_gid as uv_gid_t,
            );
            if fchown_ret != 0 {
                semsg_c!(
                    gettext(c"E136: Failed setting uid and gid for file %s: %s".as_ptr()),
                    tempname.as_ptr(),
                    uv_strerror(fchown_ret),
                );
                return false;
            }
        }

        if vim_rename(tempname.as_ptr(), fname.as_ptr()) == -1 {
            semsg_c!(
                gettext(c"E136: Can't rename ShaDa file from %s to %s!".as_ptr()),
                tempname.as_ptr(),
                fname.as_ptr(),
            );
            return false;
        }
        os_remove(tempname.as_ptr());
        true
    }
}

/// Whether the existing ShaDa file is one this process may overwrite: a
/// plain file, and writable by whichever of owner, group or others this
/// process counts as. Root may write anything.
fn writable_by_us(info: &FileInfo) -> bool {
    const S_IFDIR: uint64_t = 0o40000;
    if info.stat.st_mode & __S_IFMT as uint64_t == S_IFDIR {
        return false;
    }
    // SAFETY: `getuid`/`getgid` take nothing and cannot fail.
    let (uid, gid) = unsafe { (getuid(), getgid()) };
    if uid == ROOT_UID as uid_t {
        return true;
    }
    let bit = if info.stat.st_uid == uid as uint64_t {
        0o200
    } else if info.stat.st_gid == gid as uint64_t {
        0o020
    } else {
        0o002
    };
    info.stat.st_mode & bit != 0
}

/// Whether a file is on removable media, per `'shada'`'s `r` entries — its
/// marks are then not remembered at all.
pub(crate) unsafe fn shada_removable(name: *const c_char) -> bool {
    unsafe {
        let mut part = [0 as c_char; MAXPATHL as usize + 1];
        let new_name = home_replace_save(core::ptr::null_mut(), name);
        let mut retval = false;
        let mut p = p_shada.get();
        while *p != 0 {
            copy_option_part(
                &raw mut p,
                part.as_mut_ptr(),
                part.len(),
                c", ".as_ptr().cast_mut(),
            );
            if part[0] != b'r' as c_char {
                continue;
            }
            home_replace(
                core::ptr::null(),
                part.as_ptr().add(1),
                NameBuff.ptr().cast::<c_char>(),
                MAXPATHL as size_t,
                true,
            );
            let n = strlen(NameBuff.ptr().cast::<c_char>());
            if mb_strnicmp(NameBuff.ptr().cast::<c_char>(), new_name, n) == 0 {
                retval = true;
                break;
            }
        }
        xfree(new_name.cast::<c_void>());
        retval
    }
}

/// The number `'shada'` gives for a parameter, or −1 if it has none.
///
/// Only works for the number parameters, not for `r` or `n`.
pub unsafe fn get_shada_parameter(type_0: c_int) -> c_int {
    unsafe {
        let p = find_shada_parameter(type_0);
        if !p.is_null() && ascii_isdigit(*p as c_int) {
            atoi(p)
        } else {
            -1
        }
    }
}

/// What follows a parameter's letter in `'shada'`, or null if it has none.
pub unsafe fn find_shada_parameter(type_0: c_int) -> *mut c_char {
    unsafe {
        let mut p = p_shada.get();
        while *p != 0 {
            if *p as c_int == type_0 {
                return p.add(1);
            }
            if *p as c_int == 'n' as c_int {
                break; // 'n' is always last, and takes the rest
            }
            p = vim_strchr(p, ',' as c_int);
            if p.is_null() {
                break;
            }
            p = p.add(1);
        }
        core::ptr::null_mut()
    }
}

/// Read the current buffer's marks, the first time it is looked at.
pub unsafe fn check_marks_read() {
    unsafe {
        let buf = curbuf.get();
        if !(*buf).b_marks_read
            && get_shada_parameter('\'' as c_int) > 0
            && !(*buf).b_ffname.is_null()
        {
            shada_read_marks();
        }
        // Set unconditionally: it is what stops this running again after
        // `'shada'` gains its `'` parameter with the buffer already open.
        (*buf).b_marks_read = true;
    }
}
