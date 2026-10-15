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

#![forbid(unsafe_code)]

use crate::ascii::ascii_isdigit;
use crate::fileio::{modname, rename_file};
use crate::global_cell::GlobalCell;
use crate::mbyte::strnicmp_in;
use crate::message::{verbose_enter, verbose_leave};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::option::split_option_part;
use crate::option::vars::{P_SHADA, P_SHADAFILE, p_fs, p_shada, p_shadafile, p_verbose};
use crate::os::cshim::gettext;
use crate::os::env::{expand_env_into, home_replace_in};
use crate::os::fileio::{
    FileOpenFlags, ReadFile, WriteFile, kFileCreate, kFileCreateOnly, kFileNoSymlink,
    kFileReadOnly, kFileTruncate,
};
use crate::os::fs::{dir_exists, file_info, getperm, mkdir_recurse, os_remove, os_strerror};
use crate::os::stdpaths::stdpaths_user_state_subpath_of;
use crate::os::users::{os_getgid, os_getuid};
use crate::os::uv_error::{UV_EEXIST, UV_ELOOP, UV_ENOENT};
use crate::path::{concat_fnames, tail_with_sep_index};
use crate::types::{FAIL, Failed, FileInfo, MAXPATHL, OK};
use crate::winlayer::Buf;
use crate::{smsg, tr_c};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::read::shada_read;
use super::unpack::Source;
use super::write::shada_write;
use super::{WriteStatus, read_flags};

/// Report `E136`/`E137`/`E138`/`E886` about the ShaDa file itself, naming
/// the file it is about.
fn shada_file_error(fmt: &'static CStr, what: &CStr) {
    let what = msg_cstr(what);
    emsg_text(tr_c!(fmt, what));
}

/// As [`shada_file_error`], for the messages that name two things: a file
/// and either a second file or a libuv message.
fn shada_file_error2(fmt: &'static CStr, what: &CStr, why: &CStr) {
    let (what, why) = (msg_cstr(what), msg_cstr(why));
    emsg_text(tr_c!(fmt, what, why));
}

/// Close a file written to, saying so if that fails. `'fsync'` decides
/// whether the bytes are pushed to the platter first.
fn close_writer(file: WriteFile) {
    let error = file.close(p_fs());
    if error != 0 {
        shada_file_error(
            c"E886: System error while closing ShaDa file: %s",
            os_strerror(error),
        );
    }
}

/// The default ShaDa file, worked out once and remembered.
static DEFAULT_SHADA_FILE: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// `<state directory>/shada/main.shada`.
fn shada_get_default_file() -> CString {
    if let Some(file) = DEFAULT_SHADA_FILE.with(Clone::clone) {
        return file;
    }
    let shada_dir = stdpaths_user_state_subpath_of(c"shada", 0, false);
    let full = concat_fnames(&shada_dir, b"main.shada", true);
    let file = full.as_cstr().to_owned();
    DEFAULT_SHADA_FILE.set(Some(file.clone()));
    file
}

/// Which ShaDa file to use, or `None` if ShaDa is turned off.
///
/// `file` is a name the user gave, already expanded by the command line;
/// failing that it is `-i`'s (`p_shadafile`, where `NONE` means off), then
/// `'shada'`'s `n` entry, then the default. Only the last two go through
/// environment-variable expansion — anything the shell handed over has been
/// expanded already.
fn shada_filename(file: Option<&CStr>) -> Option<CString> {
    if let Some(file) = file.filter(|file| !file.is_empty()) {
        return Some(file.to_owned());
    }
    if P_SHADAFILE.first_byte() != 0 {
        if p_shadafile(|value| value == c"NONE") {
            return None; // "-i NONE" or "--clean"
        }
        return Some(p_shadafile(CStr::to_owned));
    }

    // 'n' is always last and takes the rest of the option.
    let named = find_shada_parameter(c_int::from(b'n'))
        .and_then(|at| p_shada(|shada| shada.to_bytes().get(at..).map(<[u8]>::to_vec)))
        .filter(|rest| !rest.is_empty());
    let named = match named {
        Some(rest) => CString::new(rest).expect("an option value holds no NUL"),
        None => shada_get_default_file(),
    };
    let expanded = expand_env_into(&named, MAXPATHL as usize + 1);
    Some(CString::new(expanded).expect("shada: expanded file name holds a NUL"))
}

/// Read a ShaDa file into the running editor.
///
/// `flags` says which parts of it are wanted; see [`read_flags`].
fn shada_read_file(file: Option<&CStr>, flags: u32) -> Result<(), Failed> {
    let Some(fname) = shada_filename(file) else {
        return Err(Failed);
    };
    let opened = ReadFile::open(&fname, kFileReadOnly);

    if p_verbose() > 1 {
        verbose_enter();
        let note = |wanted: u32, text: &'static CStr| {
            if flags & wanted != 0 {
                gettext(text).to_string_lossy()
            } else {
                "".into()
            }
        };
        let name = msg_cstr(&fname);
        let info = note(read_flags::INFO, c" info");
        let marks = note(read_flags::MARKS, c" marks");
        let oldfiles = note(read_flags::OLDFILES, c" oldfiles");
        let failed = if opened.is_err() {
            gettext(c" FAILED").to_string_lossy()
        } else {
            "".into()
        };
        smsg!(
            0,
            "Reading ShaDa file \"{name}\"{info}{marks}{oldfiles}{failed}"
        );
        verbose_leave();
    }

    let file = match opened {
        Ok(file) => file,
        Err(error) => {
            // A missing file is only worth complaining about when the caller
            // asked for one by name.
            if error != UV_ENOENT || flags & read_flags::MISSING_ERROR != 0 {
                shada_file_error2(
                    c"E886: System error while opening ShaDa file %s for reading: %s",
                    &fname,
                    os_strerror(error),
                );
            }
            return Err(Failed);
        }
    };

    let mut source = Source::File(file);
    shada_read(&mut source, flags);
    source.close();
    Ok(())
}

/// Read the marks out of the default ShaDa file.
pub fn shada_read_marks() -> Result<(), Failed> {
    shada_read_file(None, read_flags::MARKS)
}

/// Read everything out of a ShaDa file.
///
/// `forceit` lets the file's contents win over the running editor's state;
/// `missing_ok` keeps quiet about a file that is not there.
///
/// `None` is "the file 'shada' names", which is what `:rshada` with no
/// argument means.
pub fn shada_read_everything(
    fname: Option<&CStr>,
    forceit: bool,
    missing_ok: bool,
) -> Result<(), Failed> {
    let mut flags = read_flags::INFO | read_flags::MARKS | read_flags::OLDFILES;
    if !missing_ok {
        flags |= read_flags::MISSING_ERROR;
    }
    if forceit {
        flags |= read_flags::FORCE;
    }
    shada_read_file(fname, flags)
}

/// The temporary file a merged ShaDa file is built in, opened, and its
/// name.
///
/// It sits next to the real one and is named `.tmp.a` through `.tmp.z`;
/// another Nvim writing at the same moment holds one of those, so the
/// letters are tried in turn. `None` means every one of them was taken, or
/// the open failed for some other reason — which is reported here.
fn open_temp_writer(fname: &CStr, perm: c_int) -> Option<(WriteFile, CString)> {
    let mut tempname = modname(Some(fname), b".tmp.a", false)?.into_vec();
    tempname.pop(); // the terminator: the name is rebuilt for each try
    let last = tempname.len() - 1;

    loop {
        let name = CString::new(tempname.clone()).expect("shada: a NUL crept in");
        let flags: FileOpenFlags = kFileCreateOnly | kFileNoSymlink;
        match WriteFile::open(&name, flags, perm) {
            Ok(file) => return Some((file, name)),
            Err(error) if error != UV_EEXIST && error != UV_ELOOP => {
                shada_file_error2(
                    c"E886: System error while opening temporary ShaDa file %s for writing: %s",
                    &name,
                    os_strerror(error),
                );
                return None;
            }
            Err(_) => {}
        }
        if tempname[last] == b'z' {
            shada_file_error(
                c"E138: All %s.tmp.X files exist, cannot write ShaDa file!",
                fname,
            );
            return None;
        }
        tempname[last] += 1;
    }
}

/// Open the ShaDa file itself for writing, making the directory it lives in
/// if it is not there yet.
fn open_direct_writer(fname: &CStr) -> Option<WriteFile> {
    // Everything before the file name is the directory.
    let tail = tail_with_sep_index(fname.to_bytes());
    if tail != 0 {
        let dir = CString::new(&fname.to_bytes()[..tail]).expect("a part of a C string");
        if !dir_exists(&dir)
            && let Err(failure) = mkdir_recurse(&dir, 0o700)
        {
            shada_file_error2(
                c"E886: Failed to create directory %s for writing ShaDa file: %s",
                failure.dir.as_deref().unwrap_or(c"[NULL]"),
                failure.why,
            );
            return None;
        }
    }

    match WriteFile::open(fname, kFileCreate | kFileTruncate, 0o600) {
        Ok(file) => Some(file),
        Err(error) => {
            shada_file_error2(
                c"E886: System error while opening ShaDa file %s for writing: %s",
                fname,
                os_strerror(error),
            );
            None
        }
    }
}

/// Write the ShaDa file.
///
/// `nomerge` writes this session's state alone; otherwise the existing file
/// is read and merged in first. Falling back to `nomerge` is normal — it is
/// what happens when there is no file yet.
///
/// `None` is "the file 'shada' names".
pub fn shada_write_file(file: Option<&CStr>, nomerge: bool) -> c_int {
    let Some(fname) = shada_filename(file) else {
        return FAIL;
    };

    // The merge half: open the old file, then a temporary next to it.
    let mut merge: Option<(Source, WriteFile, CString)> = None;
    if !nomerge {
        match ReadFile::open(&fname, kFileReadOnly) {
            Err(error) => {
                if error != UV_ENOENT {
                    // Something other than "no file yet" — say so, but still
                    // try to write: that may work regardless.
                    shada_file_error2(c"E886: System error while opening ShaDa file %s for reading to merge before writing it: %s", &fname, os_strerror(error));
                }
            }
            Ok(reader) => {
                // The old file's permissions, less the setuid bit and with
                // read and write for its owner, so that the result is always
                // readable by whoever wrote it. If the file went away between
                // the open and here, start from u=rw.
                let perm = getperm(&fname);
                let perm = if perm >= 0 {
                    (perm & 0o777) | 0o600
                } else {
                    0o600
                };
                let source = Source::File(reader);
                match open_temp_writer(&fname, perm) {
                    Some((writer, tempname)) => merge = Some((source, writer, tempname)),
                    None => {
                        source.close();
                        return FAIL;
                    }
                }
            }
        }
    }

    let (mut writer, reader) = match merge {
        Some((source, writer, tempname)) => (writer, Some((source, tempname))),
        // No merge: the file itself is what gets written.
        None => match open_direct_writer(&fname) {
            Some(writer) => (writer, None),
            None => return FAIL,
        },
    };

    if p_verbose() > 1 {
        verbose_enter();
        let shown = msg_cstr(&fname);
        smsg!(0, "Writing ShaDa file \"{shown}\"");
        verbose_leave();
    }

    match reader {
        None => {
            let sw_ret = shada_write(&mut writer, None);
            debug_assert!(
                sw_ret != WriteStatus::IgnError,
                "shada: an ignorable error reached the top of the write"
            );
        }
        Some((mut source, tempname)) => {
            let sw_ret = shada_write(&mut writer, Some(&mut source));
            debug_assert!(
                sw_ret != WriteStatus::IgnError,
                "shada: an ignorable error reached the top of the write"
            );
            source.close();
            if !replace_original(&writer, &fname, &tempname, sw_ret) {
                shada_file_error2(
                    c"E136: Do not forget to remove %s or rename it manually to %s.",
                    &tempname,
                    &fname,
                );
            }
        }
    }
    close_writer(writer);
    OK
}

/// Move the finished temporary file over the real one. Answers whether the
/// temporary file is gone; if it is not, the caller says where it is.
fn replace_original(
    writer: &WriteFile,
    fname: &CStr,
    tempname: &CStr,
    sw_ret: WriteStatus,
) -> bool {
    if sw_ret != WriteStatus::Successful {
        let fmt = if sw_ret == WriteStatus::ReadNotShada {
            c"E136: Did not rename %s because %s does not look like a ShaDa file"
        } else {
            c"E136: Did not rename %s to %s because there were errors during writing it"
        };
        shada_file_error2(fmt, tempname, fname);
        return false;
    }

    let Some(old_info) = file_info(fname).filter(writable_by_us) else {
        shada_file_error(c"E137: ShaDa file is not writable: %s", fname);
        return false;
    };

    // Running as root, the new file is given the old one's owner: it is
    // not friendly to leave a user with a ShaDa file they cannot read
    // after a `su root`.
    if os_getuid() == ROOT_UID
        && (old_info.stat.st_uid != u64::from(ROOT_UID)
            || old_info.stat.st_gid != u64::from(os_getgid()))
    {
        let fchown_ret = writer.fchown(old_info.stat.st_uid as _, old_info.stat.st_gid as _);
        if fchown_ret != 0 {
            shada_file_error2(
                c"E136: Failed setting uid and gid for file %s: %s",
                tempname,
                os_strerror(fchown_ret),
            );
            return false;
        }
    }

    if !rename_file(tempname, fname) {
        shada_file_error2(
            c"E136: Can't rename ShaDa file from %s to %s!",
            tempname,
            fname,
        );
        return false;
    }
    os_remove(tempname);
    true
}

/// The superuser's id.
const ROOT_UID: libc::uid_t = 0;

/// Whether the existing ShaDa file is one this process may overwrite: a
/// plain file, and writable by whichever of owner, group or others this
/// process counts as. Root may write anything.
fn writable_by_us(info: &FileInfo) -> bool {
    const S_IFMT: u64 = 0o170000;
    const S_IFDIR: u64 = 0o40000;
    if info.stat.st_mode & S_IFMT == S_IFDIR {
        return false;
    }
    let (uid, gid) = (os_getuid(), os_getgid());
    if uid == ROOT_UID {
        return true;
    }
    let bit = if info.stat.st_uid == u64::from(uid) {
        0o200
    } else if info.stat.st_gid == u64::from(gid) {
        0o020
    } else {
        0o002
    };
    info.stat.st_mode & bit != 0
}

/// Whether a file is on removable media, per `'shada'`'s `r` entries — its
/// marks are then not remembered at all.
pub(crate) fn shada_removable(name: &CStr) -> bool {
    let room = 3 + name.to_bytes().len();
    let new_name = home_replace_in(None, name, room, true);
    // A copy: the option may change under a long walk.
    let shada = P_SHADA.get();
    let mut rest: &[u8] = &shada;
    while !rest.is_empty() {
        let (part, after) = split_option_part(rest, MAXPATHL as usize + 1, b", ");
        rest = after;
        let Some(prefix) = part.strip_prefix(b"r") else {
            continue;
        };
        let prefix = CString::new(prefix).expect("an option part holds no NUL");
        let folded = home_replace_in(None, &prefix, MAXPATHL as usize, true);
        let n = folded.len();
        let named = &new_name[..n.min(new_name.len())];
        if strnicmp_in(&folded, named) == 0 {
            return true;
        }
    }
    false
}

/// The number `'shada'` gives for a parameter, or −1 if it has none.
///
/// Only works for the number parameters, not for `r` or `n`.
pub fn get_shada_parameter(type_0: c_int) -> c_int {
    let Some(at) = find_shada_parameter(type_0) else {
        return -1;
    };
    p_shada(|shada| {
        let digits = &shada.to_bytes()[at..];
        if digits
            .first()
            .is_some_and(|&c| ascii_isdigit(c_int::from(c)))
        {
            atoi(digits)
        } else {
            -1
        }
    })
}

/// C's `atoi` over the digits `text` starts with: glibc's `(int) strtol`,
/// which saturates at the `long` range and then truncates to `int`.
fn atoi(text: &[u8]) -> c_int {
    let value = text
        .iter()
        .take_while(|c| c.is_ascii_digit())
        .fold(0i64, |acc, &c| {
            acc.saturating_mul(10).saturating_add(i64::from(c - b'0'))
        });
    value as c_int
}

/// Where in `'shada'` a parameter's value starts — the byte after its
/// letter — or `None` when the option does not name it.
///
/// An **offset**, not a pointer: the option owns its string, so a pointer
/// into it could not outlive the projection that read it, and every caller
/// but one only wants to know whether the letter is there at all.
pub fn find_shada_parameter(type_0: c_int) -> Option<usize> {
    p_shada(|shada| {
        let value = shada.to_bytes();
        let mut at = 0;
        while at < value.len() {
            if c_int::from(value[at]) == type_0 {
                return Some(at + 1);
            }
            if value[at] == b'n' {
                break; // 'n' is always last, and takes the rest
            }
            match value[at..].iter().position(|&b| b == b',') {
                Some(comma) => at += comma + 1,
                None => break,
            }
        }
        None
    })
}

/// Read the current buffer's marks, the first time it is looked at.
pub fn check_marks_read() {
    let mut buf = Buf::current();
    if !buf.b_marks_read && get_shada_parameter(c_int::from(b'\'')) > 0 && buf.name.full().is_some()
    {
        let _ = shada_read_marks();
    }
    // Set unconditionally: it is what stops this running again after
    // `'shada'` gains its `'` parameter with the buffer already open.
    buf.b_marks_read = true;
}
