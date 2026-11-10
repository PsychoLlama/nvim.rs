//! The filesystem layer: everything nvim asks the OS about a path, a
//! directory or an open file descriptor.
//!
//! Almost all of it is libuv's synchronous `uv_fs_*` family, and almost
//! every function here is the same shape around it — fill a `uv_fs_t`, run
//! it, read `result` back, clean the request up, answer `OK`/`FAIL`. That
//! shape is [`fs_request`] and its two shorthands, written once; upstream
//! writes it out per function and c2rust wrote the zero initialiser out
//! with it, which is where 1,386 of this file's original 2,726 lines went.
//!
//! Metadata is in [`meta`] and directories in [`dir`]; what is left here is
//! the current directory, executability, and the read/write/copy calls that
//! are libc rather than libuv.
//!
//! **Nothing here is exported by name any more**, and a file name is a
//! [`CStr`] rather than a pointer: the OS wants a NUL-terminated string and
//! nothing else, so that is the parameter type, and the call is then safe
//! from the caller's side however the body reaches libuv.
//!
//! Every surviving `unsafe` unit is a libuv or libc call that has no Rust
//! equivalent, so the deny buys no narrower obligation on those. What it
//! does buy is the *rest* of each body — the mode arithmetic, the `$PATH`
//! walk, the short-read loops — as checked code, which is where this file's
//! unchecked lines actually lived.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use crate::cstr;
use crate::event::uv::{uv_dirent_t, uv_fs_t, uv_gid_t, uv_stat_t, uv_uid_t};
use crate::os::libc::{FILE, uint64_t};
use crate::os::uv_error::{UV_EAGAIN, UV_EINTR, UV_ENOBUFS, UV_UNKNOWN, uv_code};
use core::ffi::{CStr, c_int, c_void};
use core::ptr;
use core::ptr::NonNull;
use std::ffi::{CString, OsStr};
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd, FromRawFd as _, IntoRawFd as _};
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
use std::os::unix::fs::OpenOptionsExt as _;

use crate::event::libuv::{
    uv_err_name, uv_fs_access, uv_fs_copyfile, uv_fs_lstat, uv_fs_req_cleanup, uv_strerror,
};
use crate::event::uv::{
    uv__queue, uv__work, uv_buf_t, uv_fs_type, uv_loop_s, uv_loop_t, uv_req_type, uv_timespec_t,
};
use crate::log::{LOGLVL_ERR, logmsg};
use crate::message::{verbose_enter, verbose_leave};
use crate::message_fmt::msg_cstr;
use crate::option::vars::p_verbose;
use crate::os::env::{ENV_SEPCHAR, getenv_owned};
use crate::os::libc::{ptrdiff_t, size_t};
use crate::path::{append_path, gettail_dir, save_abs_path};
use crate::smsg;
use crate::startup::STDIN_FD;
use crate::types::{ApiString, OptInt};
use crate::ui::ui_call_chdir;
use ::libc::{fcntl, fdopen, write};

pub mod dir;
pub mod meta;

pub use dir::*;
pub use meta::*;

/// Many `uv_fs_*` functions answer this on success.
const LIBUV_SUCCESS: c_int = 0;

/// libuv's "do it now, on this thread" spelling: the synchronous form of a
/// `uv_fs_*` call is a null loop paired with a null callback.
const NO_LOOP: *mut uv_loop_t = ptr::null_mut();

/// `access(2)`'s mode bits, which `uv_fs_access` takes as they are.
const R_OK: c_int = 4;
const W_OK: c_int = 2;
const X_OK: c_int = 1;

/// `fcntl(2)`'s close-on-exec commands and flag.
const F_GETFD: c_int = 1;
const F_SETFD: c_int = 2;
const FD_CLOEXEC: c_int = 1;

/// What [`os_nodetype`] answers: an ordinary file or directory, something
/// writable like a character device or socket, or something that is neither.
pub const NODE_NORMAL: c_int = 0;
pub const NODE_WRITABLE: c_int = 1;
pub const NODE_OTHER: c_int = 2;

/// `stat(2)`'s file-type field and the three types this family asks about.
pub(crate) const S_IFMT: u64 = 0o170000;
/// `st_mode & S_IFMT` for a regular file.
pub(crate) const S_IFREG: u64 = 0o100000;
/// `st_mode & S_IFMT` for a directory.
pub(crate) const S_IFDIR: u64 = 0o40000;
pub(crate) const S_IFBLK: u64 = 0o60000;
/// `S_IFLNK`: the file type bits of a symbolic link.
pub(crate) const S_IFLNK: u64 = 0o120000;
pub(crate) const S_IFCHR: u64 = 0o20000;

/// The buffer [`os_mkdtemp`] fills, sized by its callers.
pub const TEMP_FILE_PATH_MAXLEN: c_int = 256;

/// Whether a `stat(2)` mode names a directory.
fn is_dir(mode: u64) -> bool {
    mode & S_IFMT == S_IFDIR
}

/// `uv_stat_t statbuf` — the buffer [`stat`] fills. c2rust wrote all
/// eighteen fields out at each of the five sites that declare one.
const UV_STAT_T_INIT: uv_stat_t = uv_stat_t {
    st_dev: 0,
    st_mode: 0,
    st_nlink: 0,
    st_uid: 0,
    st_gid: 0,
    st_rdev: 0,
    st_ino: 0,
    st_size: 0,
    st_blksize: 0,
    st_blocks: 0,
    st_flags: 0,
    st_gen: 0,
    st_atim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
    st_mtim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
    st_ctim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
    st_birthtim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
};

/// `uv_fs_t req = { 0 }`. c2rust wrote all sixty-six fields out at each of
/// the twenty-one sites that need one -- 1,386 of this file's lines -- and
/// every one of them is zero, `UV_UNKNOWN_REQ` and `UV_FS_CUSTOM` included.
const UV_FS_T_INIT: uv_fs_t = uv_fs_t {
    data: ptr::null_mut(),
    type_0: 0 as uv_req_type,
    reserved: [ptr::null_mut(); 6],
    fs_type: 0 as uv_fs_type,
    loop_0: ptr::null_mut(),
    cb: None,
    result: 0,
    ptr: ptr::null_mut(),
    path: ptr::null(),
    statbuf: UV_STAT_T_INIT,
    new_path: ptr::null(),
    file: 0,
    flags: 0,
    mode: 0,
    nbufs: 0,
    bufs: ptr::null_mut(),
    off: 0,
    uid: 0,
    gid: 0,
    atime: 0.,
    mtime: 0.,
    work_req: uv__work {
        work: None,
        done: None,
        loop_0: ptr::null_mut::<uv_loop_s>(),
        wq: uv__queue {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        },
    },
    bufsml: [uv_buf_t {
        base: ptr::null_mut(),
        len: 0,
    }; 4],
};

/// Run one synchronous `uv_fs_*` request and answer whatever `read` makes
/// of it.
///
/// Every `uv_fs_*` call in this family is made with a null loop and a null
/// callback, which is libuv's "do it now, on this thread" spelling. Under
/// that spelling the starter's return value and `request.result` are the
/// same number, and `read` is handed both so each caller keeps testing the
/// one upstream tested. Cleanup runs whatever the answer was — which is the
/// point of writing this once, because `request.ptr` and `request.path` do
/// not survive it and `read` is the last place they can be read.
///
/// This is a *safe* function: the request is one it owns and fully
/// initialises, and `start` and `read` are safe closures, so a caller who
/// puts a raw pointer in one is the one carrying that obligation.
fn fs_request<T>(
    start: impl FnOnce(*mut uv_fs_t) -> c_int,
    read: impl FnOnce(c_int, &uv_fs_t) -> T,
) -> T {
    let mut request: uv_fs_t = UV_FS_T_INIT;
    let result = start(&raw mut request);
    let answer = read(result, &request);
    // SAFETY: `request` is a fully initialised `uv_fs_t` this frame owns,
    // and libuv accepts a cleanup on any request, started or not — an
    // untouched one has a null `path` and `ptr` and the call is a no-op.
    unsafe { uv_fs_req_cleanup(&raw mut request) };
    answer
}

/// [`fs_request`] for the common case: the starter's own answer, with
/// nothing read out of the request.
fn fs_result(start: impl FnOnce(*mut uv_fs_t) -> c_int) -> c_int {
    fs_request(start, |result, _| result)
}

/// [`fs_request`] answering `Ok`, or the libuv error code.
fn fs_ok(start: impl FnOnce(*mut uv_fs_t) -> c_int) -> Result<(), c_int> {
    match fs_result(start) {
        LIBUV_SUCCESS => Ok(()),
        code => Err(code),
    }
}

/// The C library's text for the `errno` value `error` -- `strerror(3)`,
/// copied out before another call can overwrite it.
pub(crate) fn errno_text(error: c_int) -> CString {
    // SAFETY: `strerror` answers a NUL-terminated message for any number,
    // good until the next call on this thread; it is copied at once.
    unsafe { CStr::from_ptr(::libc::strerror(error)) }.to_owned()
}

/// [`errno_text`] for what `errno` says now: upstream's `strerror(errno)`
/// after a failed call.
pub(crate) fn last_errno_text() -> CString {
    errno_text(io::Error::last_os_error().raw_os_error().unwrap_or(0))
}

/// Changes the current directory to `path`, telling the UI when it did.
pub fn os_chdir(path: &CStr) -> io::Result<()> {
    if p_verbose() >= 5 as OptInt {
        // SAFETY: `path` is NUL-terminated, and `%s` is the one conversion
        // the format string asks for.
        verbose_enter();
        smsg!(0, "chdir({})", msg_cstr(path));
        verbose_leave();
    }
    // `uv_chdir` is `chdir(2)`, which is what std calls.
    std::env::set_current_dir(OsStr::from_bytes(path.to_bytes()))?;
    ui_call_chdir(ApiString::from_cstr(path));
    Ok(())
}

/// The name of the current directory, when it fits a buffer of `room`
/// bytes with its NUL. `Err` holds what upstream's buffer was left with
/// instead: libuv's error message, cut to the same room.
///
/// `uv_cwd` is `getcwd(3)` into the caller's buffer, answering `UV_ENOBUFS`
/// when the name does not fit; `std::env::current_dir` is `getcwd` into a
/// buffer that grows, so the fit is tested here. Either way libuv's message
/// is cut to `room - 1` bytes: on `UV_ENOBUFS` the length libuv reports is
/// the one needed, which is more than `room`.
pub fn dirname_within(room: usize) -> Result<Vec<u8>, Vec<u8>> {
    let room = room.max(1);
    let code = match std::env::current_dir() {
        Ok(dir) if dir.as_os_str().len() < room => return Ok(dir.into_os_string().into_vec()),
        Ok(_) => UV_ENOBUFS,
        Err(error) => uv_code(&error),
    };
    let message = os_strerror(code).to_bytes();
    Err(message[..message.len().min(room - 1)].to_vec())
}

/// The name of the current directory, or `None` when the system will not
/// say.
pub(crate) fn current_dir() -> Option<crate::memory::XString> {
    dirname_within(crate::path::MAXPATHL as usize)
        .ok()
        .map(crate::memory::XString::from_vec)
}

/// What upstream's `os_dirname` leaves in a `MAXPATHL` buffer: the name of the
/// current directory or, when the system will not say, the error message.
/// The reader that ignores the failure and goes on with the buffer.
pub(crate) fn dirname_text() -> crate::memory::XString {
    let text = dirname_within(crate::path::MAXPATHL as usize).unwrap_or_else(|message| message);
    crate::memory::XString::from_vec(text)
}

/// libuv's text for the error code `error` -- upstream's `os_strerror`.
pub(crate) fn os_strerror(error: c_int) -> &'static CStr {
    // SAFETY: `uv_strerror` answers a static NUL-terminated string for any
    // code, a code it does not know included.
    unsafe { CStr::from_ptr(uv_strerror(error)) }
}

/// libuv's name for the error code `error` (`"ENOENT"`), as `uv_err_name`
/// spells it.
pub(crate) fn os_err_name(error: c_int) -> &'static CStr {
    // SAFETY: `uv_err_name` answers a static NUL-terminated string for any
    // code; one it does not know it names from a leaked allocation that is
    // never freed.
    unsafe { CStr::from_ptr(uv_err_name(error)) }
}

/// [`os_can_exe`] asking only whether `name` is executable.
pub(crate) fn can_execute(name: &CStr, use_path: bool) -> bool {
    os_can_exe(name, None, use_path)
}

/// [`os_can_exe`] answering where the executable `name` was found, or
/// `None` when it is not one.
pub(crate) fn executable_path(name: &CStr, use_path: bool) -> Option<crate::memory::XString> {
    let mut path = None;
    os_can_exe(name, Some(&mut path), use_path);
    path
}

/// Whether `name` is a directory and *not* a symlink to one.
pub fn os_isrealdir(name: &CStr) -> bool {
    // `lstat`, not `stat`: a symlink to a directory is not one, though
    // `dir_exists` says it is.
    fs_request(
        // SAFETY: `name` is NUL-terminated and outlives the call.
        |request| unsafe { uv_fs_lstat(NO_LOOP, request, name.as_ptr(), None) },
        |result, request| result == LIBUV_SUCCESS && is_dir(request.statbuf.st_mode),
    )
}

/// What `name` is: an ordinary file or directory (or nothing at all), a
/// writable device, or something else.
pub fn os_nodetype(name: &CStr) -> c_int {
    let Ok(statbuf) = stat(name) else {
        return NODE_NORMAL; // The file does not exist.
    };
    // Read the mode rather than asking `uv_guess_handle`, which does not
    // distinguish a block device from a directory.
    match statbuf.st_mode & S_IFMT {
        S_IFREG | S_IFDIR => NODE_NORMAL,
        S_IFBLK => NODE_OTHER, // a block device is not writable
        // Everything else is writable: `buf_write` expects NODE_WRITABLE for
        // the character device /dev/stderr.
        _ => NODE_WRITABLE,
    }
}

/// The absolute path of the running executable, cut to what a `MAXPATHL`
/// buffer holds, or `None` when the system cannot say -- procfs missing, say
/// (#6734). libuv's `uv_exepath` and std's `current_exe` are both
/// `readlink("/proc/self/exe")`.
pub(crate) fn os_exepath() -> Option<Vec<u8>> {
    let mut path = std::env::current_exe().ok()?.into_os_string().into_vec();
    path.truncate(crate::path::MAXPATHL as usize - 1);
    Some(path)
}

/// Whether `name` is an executable file — found in `$PATH` when `use_path`
/// is set and `name` carries no directory, and otherwise resolved as given.
/// An `abspath` receives the resolved path.
pub fn os_can_exe(
    name: &CStr,
    abspath: Option<&mut Option<crate::memory::XString>>,
    use_path: bool,
) -> bool {
    let has_dir = gettail_dir(name.to_bytes()) != 0;
    if !use_path || has_dir {
        // A bare name has to come from `$PATH`: files in the current
        // directory are not executable by name.
        return (use_path || has_dir) && is_executable(name, abspath);
    }
    is_executable_in_path(name, abspath)
}

/// Whether `name` is a regular file the process may execute, storing its
/// resolved absolute path in `abspath`.
fn is_executable(name: &CStr, abspath: Option<&mut Option<crate::memory::XString>>) -> bool {
    let mode = getperm(name);
    if mode < 0 {
        return false;
    }
    // Only a regular file is worth asking `access(2)` about.
    let ok = mode as u64 & S_IFMT == S_IFREG
        // SAFETY: as above.
        && fs_result(|req| unsafe { uv_fs_access(NO_LOOP, req, name.as_ptr(), X_OK, None) }) == 0;
    if ok && let Some(abspath) = abspath {
        *abspath = Some(save_abs_path(name));
    }
    ok
}

/// Whether any `$PATH` entry holds an executable called `name`, in the
/// order `$PATH` lists them.
fn is_executable_in_path(
    name: &CStr,
    mut abspath: Option<&mut Option<crate::memory::XString>>,
) -> bool {
    // Copied out because `is_executable` below can reach code that reads
    // `$PATH`.
    let Some(path) = getenv_owned(c"PATH") else {
        return false;
    };
    // Longest case: the whole of `$PATH` as one entry, a separator, `name`
    // and the NUL.
    let room = name.to_bytes().len() + path.len() + 2;
    // `xstrchrnul` stops at the NUL, so the walk visits every entry
    // including the empty ones and ends with the last.
    for entry in path.split(|&b| b == ENV_SEPCHAR) {
        let mut candidate = entry.to_vec();
        let _ = append_path(&mut candidate, name.to_bytes(), room);
        if is_executable(&cstr::owned(&candidate), abspath.as_deref_mut()) {
            return true;
        }
    }
    false
}

/// Opens or creates `path` with `open(2)`'s `flags`, answering the file or
/// a negative libuv error code. `mode` is the permission set a newly created
/// file gets, subject to the umask. The file is close-on-exec, as libuv's
/// `uv_fs_open` made it.
pub fn open_file(path: &CStr, flags: c_int, mode: c_int) -> Result<File, c_int> {
    // The access mode is the `read`/`write` pair (std masks it out of the
    // custom flags); every other flag goes through as it is.
    let access = flags & ::libc::O_ACCMODE;
    OpenOptions::new()
        .read(access != libc::O_WRONLY)
        .write(access != libc::O_RDONLY)
        .custom_flags(flags & !::libc::O_ACCMODE)
        .mode(mode.cast_unsigned())
        .open(OsStr::from_bytes(path.to_bytes()))
        .map_err(|error| uv_code(&error))
}

/// `fopen(3)`'s mode string as `open(2)` flags, per the table in its
/// manpage. A first byte outside `rwa` has no answer, and upstream aborts.
///
/// The second byte is `b` (which changes nothing on any platform nvim
/// builds for), absent, or `+`; upstream asserts the last case rather than
/// testing it, so anything else is read as `+` here too.
fn fopen_flags(mode: &[u8]) -> Option<c_int> {
    let update = !matches!(mode.first_chunk::<2>().map(|m| m[1]), None | Some(b'b'));
    debug_assert!(!update || mode.get(1) == Some(&b'+'));
    Some(match (mode.first()?, update) {
        (b'r', false) => libc::O_RDONLY,
        (b'w', false) => libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
        (b'a', false) => libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND,
        (b'r', true) => libc::O_RDWR,
        (b'w', true) => libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC,
        (b'a', true) => libc::O_RDWR | libc::O_CREAT | libc::O_APPEND,
        _ => return None,
    })
}
/// `fopen(3)` over [`open_fd`], so that every open in the tree goes through
/// one place. Answers `None` when the file cannot be opened.
fn fopen_stream(path: &CStr, flags: &CStr) -> Option<NonNull<FILE>> {
    let mode = flags.to_bytes();
    debug_assert!(!mode.is_empty() && mode.len() <= 2);
    let Some(iflags) = fopen_flags(mode) else {
        // Every call site in the tree passes a literal mode, so a mode with
        // no `open(2)` spelling is a bug in the caller rather than anything
        // the user did -- upstream aborts, and so does this.
        std::process::abort()
    };
    let fd = open_file(path, iflags, 0o666).ok()?.into_raw_fd();
    // SAFETY: `fd` is this frame's until `fdopen` adopts it, and `flags` is
    // the NUL-terminated mode string it was just parsed from.
    NonNull::new(unsafe { fdopen(fd, flags.as_ptr()) })
}

/// An open C stream, closed when dropped: the owner `fopen` hands back.
///
/// For the handful of editor sinks that stay a `FILE *` because the tree
/// writes them byte by byte (`:redir > file`, `'verbosefile'`). Writing
/// takes `&self`: the stream's buffer is C's, behind the pointer, and no
/// Rust reference reaches it.
#[derive(Debug)]
pub struct CFile {
    stream: NonNull<FILE>,
    /// The buffer [`buffer_fully`](CFile::buffer_fully) handed to
    /// `setvbuf`. The stream writes into it until it is closed, so it is
    /// released only after the `fclose`.
    buffer: Option<Box<[u8]>>,
}

impl CFile {
    /// `fopen` `path` with `mode`, or `None` when it cannot be opened.
    pub fn open(path: &CStr, mode: &CStr) -> Option<Self> {
        let stream = fopen_stream(path, mode)?;
        Some(CFile {
            stream,
            buffer: None,
        })
    }

    /// Buffer the stream fully in `size` bytes of its own, so that nothing
    /// reaches the file before it is closed (or the buffer fills). Answers
    /// `setvbuf`'s error code when it refuses.
    pub fn buffer_fully(&mut self, size: usize) -> Result<(), c_int> {
        let mut buffer = vec![0u8; size].into_boxed_slice();
        // SAFETY: an open stream this value owns, nothing written to it yet;
        // the buffer is `size` bytes and is kept in `self` until after the
        // stream is closed. A boxed slice's bytes do not move with the box.
        let status = unsafe {
            ::libc::setvbuf(
                self.stream.as_ptr(),
                buffer.as_mut_ptr().cast(),
                ::libc::_IOFBF,
                size,
            )
        };
        if status != 0 {
            return Err(status);
        }
        self.buffer = Some(buffer);
        Ok(())
    }

    /// The stream, for a writer that still takes a `FILE *`. It stays open
    /// for as long as `self` lives.
    pub fn as_ptr(&self) -> *mut FILE {
        self.stream.as_ptr()
    }

    /// Close the stream now, answering whether the close succeeded -- a
    /// write error the buffering hid shows up here, and dropping cannot
    /// say so.
    pub fn close(self) -> bool {
        let mut stream = core::mem::ManuallyDrop::new(self);
        let buffer = stream.buffer.take();
        // SAFETY: an open stream this value owns; `ManuallyDrop` keeps
        // `drop` from closing it a second time.
        let closed = unsafe { ::libc::fclose(stream.stream.as_ptr()) == 0 };
        drop(buffer);
        closed
    }

    /// Write one byte.
    pub fn putc(&self, byte: u8) {
        // SAFETY: an open stream this value owns.
        unsafe { ::libc::fputc(c_int::from(byte), self.stream.as_ptr()) };
    }

    /// Write `bytes`, NULs and all.
    pub fn write(&self, bytes: &[u8]) {
        // SAFETY: an open stream this value owns, and `bytes` is readable
        // for its length.
        unsafe { ::libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), self.stream.as_ptr()) };
    }
}

impl Drop for CFile {
    fn drop(&mut self) {
        // SAFETY: an open stream this value owns, closed exactly once.
        unsafe { ::libc::fclose(self.stream.as_ptr()) };
        // `buffer` is dropped after this, with the stream already closed.
    }
}

/// Marks `fd` close-on-exec, logging a failure. `Err` is the libuv code.
pub fn set_cloexec(fd: BorrowedFd<'_>) -> Result<(), c_int> {
    let raw = fd.as_raw_fd();
    // SAFETY: `F_GETFD` reads the flags of a descriptor the borrow keeps
    // open.
    let flags = unsafe { fcntl(raw, F_GETFD) };
    if flags < 0 {
        return Err(cloexec_failure(
            497,
            "Failed to get flags on descriptor",
            raw,
        ));
    }
    // SAFETY: `F_SETFD` on the same descriptor, adding one flag.
    if flags & FD_CLOEXEC == 0 && unsafe { fcntl(raw, F_SETFD, flags | FD_CLOEXEC) } == -1 {
        return Err(cloexec_failure(
            504,
            "Failed to set CLOEXEC on descriptor",
            raw,
        ));
    }
    Ok(())
}

/// Log why [`set_cloexec`] failed and answer the libuv code. The line
/// numbers are the sites in `v0.12.4:os/fs.c`.
fn cloexec_failure(line: u32, what: &str, fd: c_int) -> c_int {
    let error = io::Error::last_os_error();
    let why = errno_text(error.raw_os_error().unwrap_or(0));
    logmsg!(
        LOGLVL_ERR,
        c"os_set_cloexec",
        line,
        "{what} {fd}: {}",
        msg_cstr(&why)
    );
    uv_code(&error)
}

/// Writes all of `bytes` to `fd`, handling short writes and `EINTR` but no
/// other error.
///
/// Answers the number of bytes written, or a libuv error code (< 0).
pub(crate) fn write_bytes(fd: BorrowedFd<'_>, bytes: &[u8], non_blocking: bool) -> ptrdiff_t {
    let fd = fd.as_raw_fd();
    let size = bytes.len();
    let mut written_bytes: size_t = 0;
    while written_bytes != size {
        let rest = &bytes[written_bytes..];
        // SAFETY: `rest` is a live slice of exactly its own length.
        let cur = unsafe { write(fd, rest.as_ptr().cast::<c_void>(), rest.len()) };
        if cur > 0 {
            written_bytes += cur as size_t;
        } else if cur == 0 {
            // A zero-length write of a non-empty buffer has no explanation.
            return UV_UNKNOWN as ptrdiff_t;
        } else {
            let error = uv_code(&io::Error::last_os_error());
            if non_blocking && error == UV_EAGAIN {
                break;
            }
            if error != UV_EINTR && error != UV_EAGAIN {
                return error as ptrdiff_t;
            }
        }
    }
    written_bytes as ptrdiff_t
}

/// Copies `path` onto `new_path` with libuv's `copyfile` flags.
pub fn os_copy(path: &CStr, new_path: &CStr, flags: c_int) -> c_int {
    let (from, to) = (path.as_ptr(), new_path.as_ptr());
    // SAFETY: both are NUL-terminated and outlive the call.
    fs_result(|req| unsafe { uv_fs_copyfile(NO_LOOP, req, from, to, flags, None) })
}

/// `name` resolved to a real path, or `None` when it cannot be:
/// `realpath(3)`, which both `uv_fs_realpath` and `fs::canonicalize` call.
pub(crate) fn realpath(name: &CStr) -> Option<Vec<u8>> {
    let real = std::fs::canonicalize(OsStr::from_bytes(name.to_bytes())).ok()?;
    Some(real.into_os_string().into_vec())
}

/// Whether `path` exists and is a directory, a symlink to one included.
pub fn dir_exists(path: &CStr) -> bool {
    let mode = getperm(path);
    mode >= 0 && is_dir(mode as u64)
}

/// The editor's standard input as a file of the caller's own, for a reader
/// that names it `-`: the descriptor `--` saved, which the file closes when
/// dropped (as upstream's `fdopen` and `fclose` close it), or else a
/// duplicate of the real stdin.
///
/// The duplicate is close-on-exec, which upstream's `dup` was not: no child
/// needs a second copy of the editor's stdin.
pub(crate) fn stdin_file() -> Option<File> {
    let saved = STDIN_FD.get();
    if saved > 0 {
        // SAFETY: the descriptor `--` saved, which this process owns and
        // hands over to the one reader that closes it.
        return Some(unsafe { File::from_raw_fd(saved) });
    }
    io::stdin()
        .as_fd()
        .try_clone_to_owned()
        .ok()
        .map(File::from)
}

/// The descriptor `readfile` reads standard input from: the one `--` saved,
/// or 0. Unlike [`stdin_file`]'s it is not a duplicate: closing it is
/// what lets [`reopen_stdin_on_stderr`] put stderr in its place.
fn stdin_read_fd() -> c_int {
    STDIN_FD.get().max(0)
}

/// Standard input as the file `readfile` reads it from, owned: dropping it
/// closes the descriptor itself, as upstream's `close(fd)` after the read
/// does.
pub(crate) fn take_stdin_for_read() -> File {
    // SAFETY: a descriptor of this process, which `readfile` is the one
    // reader of and closes once.
    unsafe { File::from_raw_fd(stdin_read_fd()) }
}

/// After standard input was read and closed, open stderr in its place --
/// which, the lowest descriptor being free, lands on 0 -- so that shell
/// commands have something to read. Answers the new descriptor, which
/// nothing closes. Not `try_clone_to_owned`: that one starts at 3.
pub(crate) fn reopen_stdin_on_stderr() -> c_int {
    // SAFETY: `dup` of this process's stderr; the copy is left open on
    // purpose.
    unsafe { ::libc::dup(2) }
}

/// Mark the descriptor standard input is read from close-on-exec, once a
/// read converted what was read from it.
pub(crate) fn mark_stdin_cloexec() {
    // SAFETY: the descriptor `--` saved, or 0: both open, and neither closed
    // while the borrow lasts.
    let fd = unsafe { BorrowedFd::borrow_raw(stdin_read_fd()) };
    let _ = set_cloexec(fd);
}

/// Set the process's file mode creation mask, answering the old one.
pub(crate) fn set_umask(mask: ::libc::mode_t) -> ::libc::mode_t {
    // SAFETY: `umask` only swaps a per-process value; it cannot fail.
    unsafe { ::libc::umask(mask) }
}

#[derive(Default, Debug)]
pub struct Directory {
    pub request: uv_fs_t,
    pub ent: uv_dirent_t,
}
#[derive(Default, PartialEq, Eq, Debug)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
/// A `stat` answer.
///
/// `Copy`: a value.
#[derive(Default, Debug)]
pub struct FileInfo {
    pub stat: uv_stat_t,
}

impl FileInfo {
    /// The (device, inode) pair: which file this is, whatever its name.
    pub fn id(&self) -> FileID {
        FileID {
            inode: self.stat.st_ino,
            device_id: self.stat.st_dev,
        }
    }

    /// Whether `other` is the same file -- the (device, inode) pair, not
    /// the path.
    pub fn same_file_as(&self, other: &FileInfo) -> bool {
        self.id() == other.id()
    }

    /// The mode, as the `int` upstream keeps it: the type and permission
    /// bits are all in the low sixteen, which is all a mode holds.
    pub fn mode(&self) -> ::core::ffi::c_int {
        ::core::ffi::c_int::from(u16::try_from(self.stat.st_mode & 0xffff).unwrap_or_default())
    }

    /// The owner, as the system's id type holds it.
    pub fn uid(&self) -> Option<uv_uid_t> {
        uv_uid_t::try_from(self.stat.st_uid).ok()
    }

    /// The group, as the system's id type holds it.
    pub fn gid(&self) -> Option<uv_gid_t> {
        uv_gid_t::try_from(self.stat.st_gid).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fopen_flags_matches_the_fopen_table() {
        assert_eq!(fopen_flags(b"r"), Some(libc::O_RDONLY));
        assert_eq!(fopen_flags(b"rb"), Some(libc::O_RDONLY));
        assert_eq!(
            fopen_flags(b"w"),
            Some(libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC)
        );
        assert_eq!(
            fopen_flags(b"a"),
            Some(libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND)
        );
        assert_eq!(fopen_flags(b"r+"), Some(libc::O_RDWR));
        assert_eq!(
            fopen_flags(b"w+"),
            Some(libc::O_RDWR | libc::O_CREAT | libc::O_TRUNC)
        );
        assert_eq!(
            fopen_flags(b"a+"),
            Some(libc::O_RDWR | libc::O_CREAT | libc::O_APPEND)
        );
        // Neither an empty mode nor an unknown first byte has an answer.
        assert_eq!(fopen_flags(b""), None);
        assert_eq!(fopen_flags(b"x"), None);
    }

    #[test]
    fn is_dir_reads_only_the_type_field() {
        assert!(is_dir(S_IFDIR | 0o755));
        assert!(!is_dir(S_IFREG | 0o755));
        assert!(!is_dir(0o40000 - 1));
    }
}
