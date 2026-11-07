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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::ascii_isdigit;
use crate::env::{expand_env_within, home_to_tilde, state_subpath};
use crate::fileio::buffered::{
    FileOpenFlags, ReadFile, WriteFile, kFileCreate, kFileCreateOnly, kFileNoSymlink,
    kFileReadOnly, kFileTruncate,
};
use crate::fileio::{modname, rename_file};
use crate::global_cell::GlobalCell;
use crate::mbyte::strnicmp_in;
use crate::memory::ThinCString;
use crate::message::{verbose_enter, verbose_leave};
use crate::message_fmt::msg_cstr;
use crate::option::split_option_part;
use crate::option::vars::{P_SHADA, P_SHADAFILE, p_fs, p_shada, p_shadafile, p_verbose};
use crate::os::cshim::gettext;
use crate::os::fs::{dir_exists, file_info, getperm, mkdir_recurse, os_remove};
use crate::os::users::{os_getgid, os_getuid};
use crate::os::uv_error::{UV_EEXIST, UV_ELOOP, UV_ENOENT};
use crate::path::{concat_fnames, tail_with_sep_index};
use crate::smsg;
use crate::types::{FileInfo, MAXPATHL, uv_gid_t, uv_uid_t};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::read::shada_read;
use super::unpack::Source;
use super::write::shada_write;
use super::{ShadaError, WriteStatus, read_flags};

/// Close a file written to, saying so if that fails: it is the last thing
/// a write does, so there is nothing left to do about it but say.
/// `'fsync'` decides whether the bytes are pushed to the platter first.
fn close_writer(file: WriteFile) {
    if let Err(error) = file.close(p_fs()) {
        ShadaError::Close(error).report();
    }
}

/// The default ShaDa file, worked out once and remembered.
static DEFAULT_SHADA_FILE: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// `<state directory>/shada/main.shada`.
fn shada_get_default_file() -> CString {
    if let Some(file) = DEFAULT_SHADA_FILE.with(Clone::clone) {
        return file;
    }
    let shada_dir = state_subpath(c"shada", 0, false);
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
    let expanded = expand_env_within(&named, MAXPATHL as usize + 1);
    Some(CString::new(expanded).expect("shada: expanded file name holds a NUL"))
}

/// Read a ShaDa file into the running editor, saying what went wrong.
///
/// `flags` says which parts of it are wanted; see [`read_flags`].
fn shada_read_file(file: Option<&CStr>, flags: u32) {
    if let Err(err) = read_file(file, flags) {
        err.report();
    }
}

/// [`shada_read_file`]'s work.
fn read_file(file: Option<&CStr>, flags: u32) -> Result<(), ShadaError> {
    let fname = shada_filename(file).ok_or(ShadaError::Off)?;
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
                return Err(ShadaError::OpenForReading(
                    ThinCString::from_cstr(&fname),
                    error,
                ));
            }
            return Ok(());
        }
    };

    let mut source = Source::File(file);
    shada_read(&mut source, flags);
    source.close();
    Ok(())
}

/// Read the marks out of the default ShaDa file.
pub fn shada_read_marks() {
    shada_read_file(None, read_flags::MARKS);
}

/// Read everything out of a ShaDa file.
///
/// `forceit` lets the file's contents win over the running editor's state;
/// `missing_ok` keeps quiet about a file that is not there.
///
/// `None` is "the file 'shada' names", which is what `:rshada` with no
/// argument means.
pub fn shada_read_everything(fname: Option<&CStr>, forceit: bool, missing_ok: bool) {
    let mut flags = read_flags::INFO | read_flags::MARKS | read_flags::OLDFILES;
    if !missing_ok {
        flags |= read_flags::MISSING_ERROR;
    }
    if forceit {
        flags |= read_flags::FORCE;
    }
    shada_read_file(fname, flags);
}

/// The temporary file a merged ShaDa file is built in, opened, and its
/// name.
///
/// It sits next to the real one and is named `.tmp.a` through `.tmp.z`;
/// another Nvim writing at the same moment holds one of those, so the
/// letters are tried in turn. `Err` when every one of them was taken, or
/// the open failed for some other reason.
fn open_temp_writer(fname: &CStr, perm: c_int) -> Result<(WriteFile, CString), ShadaError> {
    let mut tempname = modname(Some(fname), b".tmp.a", false)
        .ok_or(ShadaError::Quiet)?
        .into_vec();
    tempname.pop(); // the terminator: the name is rebuilt for each try
    let last = tempname.len() - 1;

    loop {
        let name = CString::new(tempname.clone()).expect("shada: a NUL crept in");
        let flags: FileOpenFlags = kFileCreateOnly | kFileNoSymlink;
        match WriteFile::open(&name, flags, perm) {
            Ok(file) => return Ok((file, name)),
            Err(error) if error != UV_EEXIST && error != UV_ELOOP => {
                return Err(ShadaError::OpenTemp(ThinCString::from_cstr(&name), error));
            }
            Err(_) => {}
        }
        if tempname[last] == b'z' {
            return Err(ShadaError::AllTempsTaken(ThinCString::from_cstr(fname)));
        }
        tempname[last] += 1;
    }
}

/// Open the ShaDa file itself for writing, making the directory it lives in
/// if it is not there yet.
fn open_direct_writer(fname: &CStr) -> Result<WriteFile, ShadaError> {
    // Everything before the file name is the directory.
    let tail = tail_with_sep_index(fname.to_bytes());
    if tail != 0 {
        let dir = CString::new(&fname.to_bytes()[..tail]).expect("a part of a C string");
        if !dir_exists(&dir)
            && let Err(failure) = mkdir_recurse(&dir, 0o700)
        {
            let failed = failure.dir.as_deref().map(ThinCString::from_cstr);
            return Err(ShadaError::CreateDir(Box::new((failed, failure.why))));
        }
    }

    WriteFile::open(fname, kFileCreate | kFileTruncate, 0o600)
        .map_err(|error| ShadaError::OpenForWriting(ThinCString::from_cstr(fname), error))
}

/// Write the ShaDa file.
///
/// `nomerge` writes this session's state alone; otherwise the existing file
/// is read and merged in first. Falling back to `nomerge` is normal — it is
/// what happens when there is no file yet.
///
/// `None` is "the file 'shada' names". What goes wrong is said here; a
/// problem with the old file is said and the write goes on regardless.
pub fn shada_write_file(file: Option<&CStr>, nomerge: bool) {
    if let Err(err) = write_file(file, nomerge) {
        err.report();
    }
}

/// [`shada_write_file`]'s work, answering why the file was not written.
fn write_file(file: Option<&CStr>, nomerge: bool) -> Result<(), ShadaError> {
    let fname = shada_filename(file).ok_or(ShadaError::Off)?;

    // The merge half: open the old file, then a temporary next to it.
    let mut merge: Option<(Source, WriteFile, CString)> = None;
    if !nomerge {
        match ReadFile::open(&fname, kFileReadOnly) {
            Err(error) => {
                if error != UV_ENOENT {
                    // Something other than "no file yet" — say so, but still
                    // try to write: that may work regardless.
                    ShadaError::OpenForMerging(ThinCString::from_cstr(&fname), error).report();
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
                    Ok((writer, tempname)) => merge = Some((source, writer, tempname)),
                    Err(err) => {
                        // Said before the old file is closed, which may say
                        // something too.
                        err.report();
                        source.close();
                        return Ok(());
                    }
                }
            }
        }
    }

    let (mut writer, reader) = match merge {
        Some((source, writer, tempname)) => (writer, Some((source, tempname))),
        // No merge: the file itself is what gets written.
        None => (open_direct_writer(&fname)?, None),
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
            if let Err(err) = replace_original(&writer, &fname, &tempname, sw_ret) {
                // Why it was not replaced, then where it is.
                err.report();
                ShadaError::TempLeft(ShadaError::names(&tempname, &fname)).report();
            }
        }
    }
    close_writer(writer);
    Ok(())
}

/// Move the finished temporary file over the real one, or answer why it
/// was not; the temporary file is then still there, which the caller says.
fn replace_original(
    writer: &WriteFile,
    fname: &CStr,
    tempname: &CStr,
    sw_ret: WriteStatus,
) -> Result<(), ShadaError> {
    if sw_ret != WriteStatus::Successful {
        let names = ShadaError::names(tempname, fname);
        return Err(if sw_ret == WriteStatus::ReadNotShada {
            ShadaError::NotRenamedNotShada(names)
        } else {
            ShadaError::NotRenamedErrors(names)
        });
    }

    let Some(old_info) = file_info(fname).filter(writable_by_us) else {
        return Err(ShadaError::NotWritable(ThinCString::from_cstr(fname)));
    };

    // Running as root, the new file is given the old one's owner: it is
    // not friendly to leave a user with a ShaDa file they cannot read
    // after a `su root`.
    if os_getuid() == ROOT_UID
        && (old_info.stat.st_uid != u64::from(ROOT_UID)
            || old_info.stat.st_gid != u64::from(os_getgid()))
    {
        let (uid, gid) = (old_info.uid(), old_info.gid());
        let fchown_ret = writer.fchown(uid.unwrap_or(uv_uid_t::MAX), gid.unwrap_or(uv_gid_t::MAX));
        if fchown_ret != 0 {
            return Err(ShadaError::Chown(
                ThinCString::from_cstr(tempname),
                fchown_ret,
            ));
        }
    }

    if rename_file(tempname, fname).is_err() {
        return Err(ShadaError::Rename(ShadaError::names(tempname, fname)));
    }
    os_remove(tempname);
    Ok(())
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
    let new_name = home_to_tilde(None, name, room, true);
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
        let folded = home_to_tilde(None, &prefix, MAXPATHL as usize, true);
        let n = folded.len();
        let named = &new_name[..n.min(new_name.len())];
        if strnicmp_in(&folded, named) == 0 {
            return true;
        }
    }
    false
}

/// The number `'shada'` gives for a parameter, or `None` if it has none.
///
/// Only works for the number parameters, not for `r` or `n`. A number too
/// big for an `int` comes back as `atoi` makes it, which may be negative;
/// the callers read a negative one as upstream did.
pub fn get_shada_parameter(type_0: c_int) -> Option<c_int> {
    let at = find_shada_parameter(type_0)?;
    p_shada(|shada| {
        let digits = &shada.to_bytes()[at..];
        digits
            .first()
            .is_some_and(|&c| ascii_isdigit(c_int::from(c)))
            .then(|| atoi(digits))
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
    // The truncation to `int`: the low four bytes.
    let [a, b, c, d, ..] = value.to_le_bytes();
    c_int::from_le_bytes([a, b, c, d])
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
    if !buf.b_marks_read
        && get_shada_parameter(c_int::from(b'\'')).is_some_and(|n| n > 0)
        && buf.name.full().is_some()
    {
        shada_read_marks();
    }
    // Set unconditionally: it is what stops this running again after
    // `'shada'` gains its `'` parameter with the buffer already open.
    buf.b_marks_read = true;
}
