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

use crate::message_fmt::{c_str, emsg_text, msg_cstr};
use crate::os::uv_error::{UV_EEXIST, UV_ELOOP, UV_ENOENT};
use crate::smsg;
use crate::tr_c;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use std::ffi::CString;

use super::*;
use crate::os::cshim::gettext;
use crate::types::{FAIL, Failed, MAXPATHL, NUL, OK};

/// Report `E136`/`E137`/`E138`/`E886` about the ShaDa file itself, naming
/// the file it is about.
///
/// The messages here all end in one variadic call; writing it once keeps
/// the unchecked part of reporting to these two functions.
fn shada_file_error(fmt: &'static CStr, what: *const c_char) {
    // SAFETY: `what` is a NUL-terminated name.
    let what = unsafe { c_str(what) };
    emsg_text(tr_c!(fmt, what));
}

/// As [`shada_file_error`], for the messages that name two things: a file
/// and either a second file or a libuv message.
fn shada_file_error2(fmt: &'static CStr, what: *const c_char, why: *const c_char) {
    // SAFETY: as `shada_file_error`, for a second NUL-terminated string.
    let (what, why) = unsafe { (c_str(what), c_str(why)) };
    emsg_text(tr_c!(fmt, what, why));
}

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
    let error = unsafe { file_close(cookie, p_fs.get() != 0) };
    if error != 0 {
        shada_file_error(c"E886: System error while closing ShaDa file: %s", unsafe {
            uv_strerror(error)
        });
    }
}

/// The default ShaDa file, worked out once and remembered.
static default_shada_file: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// `<state directory>/shada/main.shada`.
unsafe fn shada_get_default_file() -> *const c_char {
    if default_shada_file.with(Option::is_none) {
        // SAFETY: both helpers answer an owned NUL-terminated string.
        let shada_dir = unsafe { stdpaths_user_state_subpath(c"shada".as_ptr(), 0, false) };
        let full = unsafe { concat_fnames_realloc(shada_dir, c"main.shada".as_ptr(), true) };
        default_shada_file.set(Some(unsafe { CStr::from_ptr(full) }.to_owned()));
        unsafe { xfree(full.cast::<c_void>()) };
    }
    // The cell is written once and never replaced, so the string it owns
    // stays where it is for as long as the editor runs.
    default_shada_file.with(|file| match file {
        Some(file) => file.as_ptr(),
        None => core::ptr::null(),
    })
}

/// Which ShaDa file to use, or `None` if ShaDa is turned off.
///
/// `file` is a name the user gave, already expanded by the command line;
/// failing that it is `-i`'s (`p_shadafile`, where `NONE` means off), then
/// `'shada'`'s `n` entry, then the default. Only the last two go through
/// environment-variable expansion — anything the shell handed over has been
/// expanded already.
unsafe fn shada_filename(file: *const c_char) -> Option<CString> {
    let mut expansion = [0 as c_char; MAXPATHL as usize];
    if !file.is_null() && unsafe { *file } != NUL as c_char {
        return Some(unsafe { CStr::from_ptr(file) }.to_owned());
    }
    if !p_shadafile.get().is_null() && unsafe { *p_shadafile.get() } != NUL as c_char {
        if unsafe { strequal(p_shadafile.get(), c"NONE".as_ptr()) } {
            return None; // "-i NONE" or "--clean"
        }
        return Some(unsafe { CStr::from_ptr(p_shadafile.get()) }.to_owned());
    }

    let mut named = unsafe { find_shada_parameter('n' as c_int) };
    if named.is_null() || unsafe { *named } == NUL as c_char {
        named = unsafe { shada_get_default_file() }.cast_mut();
    }
    let len = unsafe { expand_env(named, expansion.as_mut_ptr(), MAXPATHL) };
    let expanded = unsafe { core::slice::from_raw_parts(expansion.as_ptr().cast::<u8>(), len) };
    Some(CString::new(expanded).expect("shada: expanded file name holds a NUL"))
}

/// Read a ShaDa file into the running editor.
///
/// `flags` says which parts of it are wanted; see the `kShaDa*` values.
unsafe fn shada_read_file(file: *const c_char, flags: c_int) -> Result<(), Failed> {
    let Some(fname) = (unsafe { shada_filename(file) }) else {
        return Err(Failed);
    };
    let mut sd_reader: FileDescriptor = unsafe { core::mem::zeroed() };
    let of_ret = unsafe {
        file_open(
            &raw mut sd_reader,
            fname.as_ptr(),
            kFileReadOnly as c_int,
            0,
        )
    };

    if p_verbose.get() > 1 {
        unsafe { verbose_enter() };
        let note = |wanted: c_uint, text: &'static CStr| {
            if flags as c_uint & wanted != 0 {
                gettext(text).to_string_lossy()
            } else {
                "".into()
            }
        };
        let name = msg_cstr(&fname);
        let info = note(kShaDaWantInfo as c_uint, c" info");
        let marks = note(kShaDaWantMarks as c_uint, c" marks");
        let oldfiles = note(kShaDaGetOldfiles as c_uint, c" oldfiles");
        let failed = if of_ret != 0 {
            gettext(c" FAILED").to_string_lossy()
        } else {
            "".into()
        };
        smsg!(
            0,
            "Reading ShaDa file \"{name}\"{info}{marks}{oldfiles}{failed}"
        );
        unsafe { verbose_leave() };
    }

    if of_ret != 0 {
        // A missing file is only worth complaining about when the caller
        // asked for one by name.
        if of_ret != UV_ENOENT || flags & kShaDaMissingError as c_int != 0 {
            shada_file_error2(
                c"E886: System error while opening ShaDa file %s for reading: %s",
                fname.as_ptr(),
                unsafe { uv_strerror(of_ret) },
            );
        }
        return Err(Failed);
    }

    unsafe { shada_read(&raw mut sd_reader, flags) };
    unsafe { close_file(&raw mut sd_reader) };
    Ok(())
}

/// Read the marks out of the default ShaDa file.
pub unsafe fn shada_read_marks() -> Result<(), Failed> {
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
) -> Result<(), Failed> {
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
    unsafe { shada_read_file(fname, flags) }
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
    let tempname = unsafe { modname(fname.as_ptr(), c".tmp.a".as_ptr(), false) };
    if tempname.is_null() {
        return None;
    }
    let mut tempname = {
        let owned = unsafe { CStr::from_ptr(tempname) }.to_owned();
        unsafe { xfree(tempname.cast::<c_void>()) };
        owned.into_bytes_with_nul()
    };
    let last = tempname.len() - 2; // before the NUL

    loop {
        let error = unsafe {
            file_open(
                sd_writer,
                tempname.as_ptr().cast::<c_char>(),
                kFileCreateOnly as c_int | kFileNoSymlink as c_int,
                perm,
            )
        };
        if error == 0 {
            return Some(CString::from_vec_with_nul(tempname).expect("shada: a NUL crept in"));
        }
        if error != UV_EEXIST && error != UV_ELOOP {
            shada_file_error2(
                c"E886: System error while opening temporary ShaDa file %s for writing: %s",
                tempname.as_ptr().cast(),
                unsafe { uv_strerror(error) },
            );
            return None;
        }
        if tempname[last] == b'z' {
            shada_file_error(
                c"E138: All %s.tmp.X files exist, cannot write ShaDa file!",
                fname.as_ptr(),
            );
            return None;
        }
        tempname[last] += 1;
    }
}

/// Open the ShaDa file itself for writing, making the directory it lives in
/// if it is not there yet. Answers whether it is open.
unsafe fn open_direct_writer(sd_writer: *mut FileDescriptor, fname: &CStr) -> Result<bool, ()> {
    // `path_tail_with_sep` points at the file name; NUL it out to get the
    // directory, then put the byte back.
    let fname = fname.as_ptr().cast_mut();
    let tail = unsafe { path_tail_with_sep(fname) };
    if tail != fname {
        let tail_save = unsafe { *tail };
        unsafe { *tail = NUL as c_char };
        let missing = !unsafe { os_isdir(fname) };
        let mut failed_dir = core::ptr::null_mut::<c_char>();
        let ret = if missing {
            unsafe { os_mkdir_recurse(fname, 0o700, &raw mut failed_dir, core::ptr::null_mut()) }
        } else {
            0
        };
        unsafe { *tail = tail_save };
        if ret != 0 {
            shada_file_error2(
                c"E886: Failed to create directory %s for writing ShaDa file: %s",
                failed_dir,
                unsafe { uv_strerror(ret) },
            );
            unsafe { xfree(failed_dir.cast::<c_void>()) };
            return Err(());
        }
    }

    let error = unsafe {
        file_open(
            sd_writer,
            fname,
            kFileCreate as c_int | kFileTruncate as c_int,
            0o600,
        )
    };
    if error != 0 {
        shada_file_error2(
            c"E886: System error while opening ShaDa file %s for writing: %s",
            fname,
            unsafe { uv_strerror(error) },
        );
        return Ok(false);
    }
    Ok(true)
}

/// Write the ShaDa file.
///
/// `nomerge` writes this session's state alone; otherwise the existing file
/// is read and merged in first. Falling back to `nomerge` is normal — it is
/// what happens when there is no file yet.
pub unsafe fn shada_write_file(file: *const c_char, nomerge: bool) -> c_int {
    let Some(fname) = (unsafe { shada_filename(file) }) else {
        return FAIL;
    };
    let mut sd_writer: FileDescriptor = unsafe { core::mem::zeroed() };
    let mut sd_reader: FileDescriptor = unsafe { core::mem::zeroed() };

    // The merge half: open the old file, then a temporary next to it.
    let mut merge = None;
    if !nomerge {
        let name = fname.as_ptr();
        let error = unsafe { file_open(&raw mut sd_reader, name, kFileReadOnly as c_int, 0) };
        if error != 0 {
            if error != UV_ENOENT {
                // Something other than "no file yet" — say so, but still
                // try to write: that may work regardless.
                shada_file_error2(c"E886: System error while opening ShaDa file %s for reading to merge before writing it: %s", fname.as_ptr(), unsafe { uv_strerror(error) });
            }
        } else {
            // The old file's permissions, less the setuid bit and with
            // read and write for its owner, so that the result is always
            // readable by whoever wrote it. If the file went away between
            // the open and here, start from u=rw.
            let perm = unsafe { os_getperm(fname.as_ptr()) } as c_int;
            let perm = if perm >= 0 {
                (perm & 0o777) | 0o600
            } else {
                0o600
            };
            match unsafe { open_temp_writer(&raw mut sd_writer, &fname, perm) } {
                Some(tempname) => merge = Some(tempname),
                None => {
                    unsafe { close_file(&raw mut sd_reader) };
                    return FAIL;
                }
            }
        }
    }

    let reader = if merge.is_some() {
        &raw mut sd_reader
    } else {
        // No merge: the file itself is what gets written.
        match unsafe { open_direct_writer(&raw mut sd_writer, &fname) } {
            Err(()) => return FAIL,
            Ok(false) => return FAIL,
            Ok(true) => {}
        }
        core::ptr::null_mut()
    };

    if p_verbose.get() > 1 {
        unsafe { verbose_enter() };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname.as_ptr()) };
        smsg!(0, "Writing ShaDa file \"{fname}\"");
        unsafe { verbose_leave() };
    }

    let sw_ret = unsafe { shada_write(&raw mut sd_writer, reader) };
    debug_assert!(
        sw_ret != kSDWriteIgnError,
        "shada: an ignorable error reached the top of the write"
    );

    if let Some(tempname) = merge {
        unsafe { close_file(&raw mut sd_reader) };
        if !unsafe { replace_original(&raw mut sd_writer, &fname, &tempname, sw_ret) } {
            shada_file_error2(
                c"E136: Do not forget to remove %s or rename it manually to %s.",
                tempname.as_ptr(),
                fname.as_ptr(),
            );
        }
    }
    unsafe { close_file(&raw mut sd_writer) };
    OK
}

/// Move the finished temporary file over the real one. Answers whether the
/// temporary file is gone; if it is not, the caller says where it is.
unsafe fn replace_original(
    sd_writer: *mut FileDescriptor,
    fname: &CStr,
    tempname: &CStr,
    sw_ret: ShaDaWriteResult,
) -> bool {
    if sw_ret != kSDWriteSuccessful {
        let fmt = if sw_ret == kSDWriteReadNotShada {
            c"E136: Did not rename %s because %s does not look like a ShaDa file"
        } else {
            c"E136: Did not rename %s to %s because there were errors during writing it"
        };
        shada_file_error2(fmt, tempname.as_ptr(), fname.as_ptr());
        return false;
    }

    let mut old_info = FileInfo::default();
    if !unsafe { os_fileinfo(fname.as_ptr(), &raw mut old_info) } || !writable_by_us(&old_info) {
        shada_file_error(c"E137: ShaDa file is not writable: %s", fname.as_ptr());
        return false;
    }

    // Running as root, the new file is given the old one's owner: it is
    // not friendly to leave a user with a ShaDa file they cannot read
    // after a `su root`.
    if unsafe { getuid() } == ROOT_UID as uid_t
        && (old_info.stat.st_uid != ROOT_UID as uint64_t
            || old_info.stat.st_gid != unsafe { getgid() } as uint64_t)
    {
        let fchown_ret = unsafe {
            os_fchown(
                file_fd(sd_writer),
                old_info.stat.st_uid as uv_uid_t,
                old_info.stat.st_gid as uv_gid_t,
            )
        };
        if fchown_ret != 0 {
            shada_file_error2(
                c"E136: Failed setting uid and gid for file %s: %s",
                tempname.as_ptr(),
                unsafe { uv_strerror(fchown_ret) },
            );
            return false;
        }
    }

    if unsafe { vim_rename(tempname.as_ptr(), fname.as_ptr()) } == -1 {
        shada_file_error2(
            c"E136: Can't rename ShaDa file from %s to %s!",
            tempname.as_ptr(),
            fname.as_ptr(),
        );
        return false;
    }
    unsafe { os_remove(tempname.as_ptr()) };
    true
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
    let mut folded = [0 as c_char; MAXPATHL as usize];
    let mut part = [0 as c_char; MAXPATHL as usize + 1];
    let new_name = unsafe { home_replace_save(core::ptr::null_mut(), name) };
    let mut retval = false;
    let mut p = p_shada.get();
    while unsafe { *p } != 0 {
        let seps = c", ".as_ptr().cast_mut();
        unsafe { copy_option_part(&raw mut p, part.as_mut_ptr(), part.len(), seps) };
        if part[0] != b'r' as c_char {
            continue;
        }
        let tail = unsafe { part.as_ptr().add(1) };
        let out = folded.as_mut_ptr();
        unsafe { home_replace(core::ptr::null(), tail, out, MAXPATHL as size_t, true) };
        let n = unsafe { strlen(folded.as_ptr()) };
        if unsafe { mb_strnicmp(folded.as_ptr(), new_name, n) } == 0 {
            retval = true;
            break;
        }
    }
    unsafe { xfree(new_name.cast::<c_void>()) };
    retval
}

/// The number `'shada'` gives for a parameter, or −1 if it has none.
///
/// Only works for the number parameters, not for `r` or `n`.
pub unsafe fn get_shada_parameter(type_0: c_int) -> c_int {
    let p = unsafe { find_shada_parameter(type_0) };
    if !p.is_null() && ascii_isdigit(unsafe { *p } as c_int) {
        unsafe { atoi(p) }
    } else {
        -1
    }
}

/// What follows a parameter's letter in `'shada'`, or null if it has none.
pub unsafe fn find_shada_parameter(type_0: c_int) -> *mut c_char {
    let mut p = p_shada.get();
    while unsafe { *p } != 0 {
        if unsafe { *p } as c_int == type_0 {
            return unsafe { p.add(1) };
        }
        if unsafe { *p } as c_int == 'n' as c_int {
            break; // 'n' is always last, and takes the rest
        }
        p = unsafe { vim_strchr(p, ',' as c_int) };
        if p.is_null() {
            break;
        }
        p = unsafe { p.add(1) };
    }
    core::ptr::null_mut()
}

/// Read the current buffer's marks, the first time it is looked at.
pub unsafe fn check_marks_read() {
    let buf = curbuf.get();
    if !unsafe { (*buf).b_marks_read }
        && unsafe { get_shada_parameter('\'' as c_int) } > 0
        && !unsafe { (*buf).b_ffname.is_null() }
    {
        let _ = unsafe { shada_read_marks() };
    }
    // Set unconditionally: it is what stops this running again after
    // `'shada'` gains its `'` parameter with the buffer already open.
    unsafe { (*buf).b_marks_read = true };
}
