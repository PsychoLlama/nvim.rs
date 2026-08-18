//! Writing a value to a file -- `writefile()`.
//!
//! [`f_writefile`] checks the first argument's type, reads the flags (`a`
//! append, `b` binary, `s`/`S` fsync or not, `p` create parent directories,
//! `D` delete the file when the calling function returns), opens the path and
//! then hands the value to one of [`write_list`], [`write_blob`] or
//! [`write_string`].
//!
//! # Two conventions worth naming
//!
//! A List is written line by line, and a NL *inside* one of its strings
//! stands for a NUL byte in the file -- which is why [`write_list`] splits
//! each item at its newlines and writes a NUL between the pieces, rather than
//! writing the string whole.  And the `D` flag's deferred delete is
//! registered *before* the first byte is written, so the file goes away even
//! when writing it fails; that ordering is observable and is kept.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    Args, NUMBUFLEN, e_error_while_writing_str, frame, from, kFileAppend, kFileCreate, kFileMkDir,
    kFileTruncate, numbuf, str_arg_buf, str_arg_chk,
};
use crate::eval::typval::{tv_blob_len, tv_check_str_or_nr, tv_get_string_chk};
use crate::eval::userfunc::{add_defer, can_add_defer};
use crate::event::libuv::uv_strerror;
use crate::ex_cmds::check_secure;
use crate::main::{current_sctx, e_invarg2, p_fs};
use crate::message::emsg;
use crate::os::fileio::{file_close, file_flush, file_open, file_write};
use crate::os::libc::{gettext, strlen};
use crate::path::FullName_save;
use crate::runtime::script_is_lua;
use crate::semsg_c;
use crate::types::{
    EvalFuncData, FileDescriptor, VAR_BLOB, VAR_LIST, VAR_STRING, VAR_UNLOCKED, blob_T, list_T,
    listitem_T, ptrdiff_t, size_t, typval_T, typval_vval_union, varnumber_T,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

// ---------------------------------------------------------------------
// The safe layer this builtin adds
// ---------------------------------------------------------------------

/// The file being written, and the buffered writes over it.
///
/// Closing is not a `Drop`: `file_close` answers an error code that the
/// builtin reports, so the close is a step of the body rather than a
/// cleanup.
struct Out(FileDescriptor);

impl Out {
    /// Open `fname` for writing, or the libuv error saying why not.
    fn open(fname: &CStr, append: bool, mkdir_p: bool) -> Result<Self, c_int> {
        let mut out = Self(FileDescriptor {
            fd: 0,
            buffer: ptr::null_mut(),
            read_pos: ptr::null_mut(),
            write_pos: ptr::null_mut(),
            wr: false,
            eof: false,
            non_blocking: false,
            bytes_read: 0,
        });
        let flags = (if append { kFileAppend } else { kFileTruncate })
            | (if mkdir_p { kFileMkDir } else { kFileCreate })
            | kFileCreate;
        let (fp, name) = (&raw mut out.0, fname.as_ptr());
        // SAFETY: `fp` is this frame's own descriptor, which the callee
        // fills in, and `name` is NUL-terminated.
        let error = unsafe { file_open(fp, name, flags as c_int, 0o666) };
        if error == 0 { Ok(out) } else { Err(error) }
    }

    /// Write `len` bytes at `data`, answering how many were accepted or a
    /// negative libuv error code.
    ///
    /// # Safety
    /// `data` is readable for `len` bytes.
    unsafe fn write_raw(&mut self, data: *const c_char, len: usize) -> ptrdiff_t {
        // SAFETY: the caller's contract, and an open descriptor.
        unsafe { file_write(&raw mut self.0, data, len as size_t) }
    }

    fn write(&mut self, data: &[u8]) -> ptrdiff_t {
        // SAFETY: a live slice is readable for its own length.
        unsafe { self.write_raw(data.as_ptr().cast::<c_char>(), data.len()) }
    }

    fn flush(&mut self) -> c_int {
        // SAFETY: an open descriptor.
        unsafe { file_flush(&raw mut self.0) }
    }

    /// Flush, `fsync` when asked, and close; a libuv error code or zero.
    fn close(&mut self, do_fsync: bool) -> c_int {
        // SAFETY: an open descriptor, closed exactly once.
        unsafe { file_close(&raw mut self.0, do_fsync) }
    }
}

/// One item of the List being written.
#[derive(Clone, Copy)]
struct Item(*const listitem_T);

impl Item {
    fn of(p: *const listitem_T) -> Option<Self> {
        (!p.is_null()).then_some(Self(p))
    }

    /// The first item of `list`, which may itself be NULL.
    fn first(list: *const list_T) -> Option<Self> {
        // SAFETY: a live list, or NULL, which `as_ref` answers None for.
        Self::of(unsafe { list.as_ref() }.map_or(ptr::null(), |l| l.lv_first))
    }

    fn next(self) -> Option<Self> {
        // SAFETY: a live item.
        Self::of(unsafe { (*self.0).li_next })
    }

    /// The item's value as a string, or None -- having reported -- for a
    /// type that has no string form.
    fn string<'a>(self) -> Option<&'a CStr> {
        // SAFETY: a live item; `tv_get_string_chk` answers a NUL-terminated
        // string, or NULL.
        unsafe {
            tv_get_string_chk(&raw const (*self.0).li_tv)
                .as_ref()
                .map(|p| CStr::from_ptr(p))
        }
    }

    /// Whether the item is a String or a Number, having reported if not.
    fn is_str_or_nr(self) -> bool {
        // SAFETY: a live item.
        unsafe { tv_check_str_or_nr(&raw const (*self.0).li_tv) }
    }
}

/// The items of `list`, front to back.  Nothing below writes to the list, so
/// the link is read once per step, as upstream's `TV_LIST_ITER_CONST` does.
fn items(list: *const list_T) -> impl Iterator<Item = Item> {
    let mut cur = Item::first(list);
    core::iter::from_fn(move || {
        let item = cur?;
        cur = item.next();
        Some(item)
    })
}

/// The List argument 0 holds, which may be NULL.
fn list_of(tv: &typval_T) -> *const list_T {
    // SAFETY: only reached under a `VAR_LIST` tag, which is what makes
    // `v_list` the live arm.
    unsafe { tv.vval.v_list }
}

// ---------------------------------------------------------------------
// The messages
// ---------------------------------------------------------------------

/// Report the one-`%s` message `fmt`, translated, about `a`.
fn err1(fmt: *const c_char, a: *const c_char) {
    // SAFETY: `fmt` is a NUL-terminated format taking one string, and `a` is
    // a NUL-terminated string.
    unsafe { semsg_c!(gettext(fmt), a) };
}

/// Report the two-`%s` message `fmt`, translated, about `a` and `b`.
fn err2(fmt: *const c_char, a: *const c_char, b: *const c_char) {
    // SAFETY: `fmt` is a NUL-terminated format taking two strings, and both
    // are NUL-terminated.
    unsafe { semsg_c!(gettext(fmt), a, b) };
}

/// Report `msg`, translated.
fn err(msg: &CStr) {
    // SAFETY: `msg` is NUL-terminated, which is all `gettext` and `emsg` ask.
    unsafe { emsg(gettext(msg.as_ptr())) };
}

/// libuv's name for the error code `error`.
fn strerror(error: c_int) -> *const c_char {
    // SAFETY: `uv_strerror` answers a NUL-terminated string for any code.
    unsafe { uv_strerror(error) }
}

/// Report `E80: Error while writing: %s`.
fn err_writing(error: c_int) {
    err1(e_error_while_writing_str.as_ptr(), strerror(error));
}

// ---------------------------------------------------------------------
// The three writers
// ---------------------------------------------------------------------

/// Write every item of `list` as a line, `binary` suppressing the newline
/// after the last one.
///
/// False when an item has no string form -- which reports on its own and is
/// the one exit that does not report a write error.
fn write_list(out: &mut Out, list: *const list_T, binary: bool) -> bool {
    let mut error;
    'failed: {
        for li in items(list) {
            let Some(s) = li.string() else { return false };
            let bytes = s.to_bytes();
            let mut hunk_start = 0;
            let mut p = 0;
            loop {
                if p == bytes.len() || bytes[p] == b'\n' {
                    if p != hunk_start {
                        let written = out.write(&bytes[hunk_start..p]);
                        if written < 0 {
                            error = written as c_int;
                            break 'failed;
                        }
                    }
                    if p == bytes.len() {
                        break;
                    }
                    hunk_start = p + 1;
                    // A NL in the string stands for a NUL in the file.
                    let written = out.write(&[0]);
                    if written < 0 {
                        // Upstream leaves the *item* here rather than the
                        // function, still writes the line separator below,
                        // and then lets the flush overwrite `error` -- so a
                        // failed NUL write is reported only when the flush
                        // fails too.  Kept.
                        break;
                    }
                }
                p += 1;
            }
            if !binary || li.next().is_some() {
                let written = out.write(b"\n");
                if written < 0 {
                    error = written as c_int;
                    break 'failed;
                }
            }
        }
        error = out.flush();
        if error == 0 {
            return true;
        }
    }
    err_writing(error);
    false
}

/// Write `len` bytes at `data` and flush.
///
/// # Safety
/// `data` is readable for `len` bytes.
unsafe fn write_data(out: &mut Out, data: *const c_char, len: usize) -> bool {
    let mut error;
    'failed: {
        if len > 0 {
            // SAFETY: the caller's contract.
            let written = unsafe { out.write_raw(data, len) };
            // Upstream tests against `len`, not against zero, so a short
            // write reports the count it did accept as if it were a code.
            if written < len as ptrdiff_t {
                error = written as c_int;
                break 'failed;
            }
        }
        error = out.flush();
        if error == 0 {
            return true;
        }
    }
    err_writing(error);
    false
}

fn write_blob(out: &mut Out, blob: *const blob_T) -> bool {
    // SAFETY: a live blob, whose `ga_data` holds `tv_blob_len` readable
    // bytes.
    unsafe {
        let (data, len) = ((*blob).bv_ga.ga_data.cast(), tv_blob_len(blob) as usize);
        write_data(out, data, len)
    }
}

fn write_string(out: &mut Out, data: *const c_char) -> bool {
    // SAFETY: a live String argument, which is NUL-terminated, and so
    // readable for the `strlen` bytes before its terminator.
    unsafe { write_data(out, data, strlen(data)) }
}

// ---------------------------------------------------------------------
// The builtin
// ---------------------------------------------------------------------

/// Whether the sandbox forbids writing, having reported it.
fn secure() -> bool {
    // SAFETY: reads the sandbox depth and may report; no arguments.
    unsafe { check_secure() }
}

/// Whether a deferred call can be registered, having reported if not.
fn can_defer() -> bool {
    // SAFETY: reads the call stack and may report; no arguments.
    unsafe { can_add_defer() }
}

/// Whether the running script is Lua, which is what makes a String argument
/// mean blob data rather than a mistake.
fn in_lua_script() -> bool {
    // SAFETY: a script id, which the callee only looks up.
    unsafe { script_is_lua(current_sctx.get().sc_sid) }
}

/// Register `delete({fname})` to run when the calling function returns --
/// the `D` flag.
fn defer_delete(fname: &CStr) {
    // SAFETY: `fname` is NUL-terminated; the answer is a string in nvim's
    // heap, which the deferred call takes over.
    let full = unsafe { FullName_save(fname.as_ptr(), false) };
    let mut tv = typval_T {
        v_type: VAR_STRING,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_string: full },
    };
    let name = c"delete".as_ptr().cast_mut();
    // SAFETY: one argument, at `tv`, whose contents the callee takes over.
    unsafe { add_defer(name, 1, &raw mut tv) };
}

/// Whether the first argument is something this builtin can write, having
/// reported if not.
fn writable(args: Args<'_>) -> bool {
    // XXX: this logic is a bit weird because of how `decode_string` works
    // (#39328): it assigns VAR_BLOB when it finds a NUL in the Lua string,
    // and VAR_STRING when it does not.
    if args.ty(0) == VAR_LIST {
        return items(list_of(args.get(0))).all(Item::is_str_or_nr);
    }
    // A Lua string is always treated as blob data.
    if args.ty(0) == VAR_BLOB || (args.ty(0) == VAR_STRING && in_lua_script()) {
        return true;
    }
    let what = c"writefile() first argument must be a List or a Blob";
    // SAFETY: a NUL-terminated literal, which is all `gettext` reads.
    let what = unsafe { gettext(what.as_ptr()) };
    err1(e_invarg2.as_ptr(), what);
    false
}

/// The flags of the third argument, or None having reported an unknown one.
struct Flags {
    binary: bool,
    append: bool,
    defer: bool,
    do_fsync: bool,
    mkdir_p: bool,
}

impl Flags {
    fn read(args: Args<'_>) -> Option<Self> {
        let mut f = Self {
            binary: false,
            append: false,
            defer: false,
            do_fsync: p_fs.get() != 0,
            mkdir_p: false,
        };
        if !args.has(2) {
            return Some(f);
        }
        let flags = str_arg_chk(args, 2)?;
        for (i, &c) in flags.to_bytes().iter().enumerate() {
            match c {
                b'b' => f.binary = true,
                b'a' => f.append = true,
                b'D' => f.defer = true,
                b's' => f.do_fsync = true,
                b'S' => f.do_fsync = false,
                b'p' => f.mkdir_p = true,
                _ => {
                    // The rest of the flags with `%s`, not this one with
                    // `%c`, so that a multibyte character survives.
                    err1(c"E5060: Unknown flag: %s".as_ptr(), from(flags, i).as_ptr());
                    return None;
                }
            }
        }
        Some(f)
    }
}

/// `writefile({object}, {fname} [, {flags}])`: the List, Blob or Lua string
/// written to the file, 0 on success and -1 on failure.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 2..3, and `rettv`
/// a cleared result.
pub unsafe fn f_writefile(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1 as varnumber_T;
    if secure() || !writable(args) {
        return;
    }
    let Some(flags) = Flags::read(args) else {
        return;
    };

    let mut buf: [c_char; NUMBUFLEN] = numbuf();
    let Some(fname) = str_arg_buf(args, 1, &mut buf) else {
        return;
    };
    if flags.defer && !can_defer() {
        return;
    }
    if fname.to_bytes().is_empty() {
        err(c"E482: Can't open file with an empty name");
        return;
    }
    let mut out = match Out::open(fname, flags.append, flags.mkdir_p) {
        Ok(out) => out,
        Err(error) => {
            let fmt = c"E482: Can't open file %s for writing: %s";
            err2(fmt.as_ptr(), fname.as_ptr(), strerror(error));
            return;
        }
    };

    // Before the first byte, so that the file goes away even when writing it
    // fails.  The order is observable and is upstream's.
    if flags.defer {
        defer_delete(fname);
    }

    let write_ok = match args.ty(0) {
        VAR_BLOB => match blob_of(args.get(0)) {
            Some(blob) => write_blob(&mut out, blob),
            None => true,
        },
        VAR_STRING => write_string(&mut out, string_of(args.get(0))),
        _ => write_list(&mut out, list_of(args.get(0)), flags.binary),
    };
    if write_ok {
        rettv.vval.v_number = 0 as varnumber_T;
    }
    let error = out.close(flags.do_fsync);
    if error != 0 {
        let fmt = c"E80: Error when closing file %s: %s";
        err2(fmt.as_ptr(), fname.as_ptr(), strerror(error));
    }
}

/// The Blob argument 0 holds, or None when it is the empty one.
fn blob_of(tv: &typval_T) -> Option<*const blob_T> {
    // SAFETY: only reached under a `VAR_BLOB` tag, which is what makes
    // `v_blob` the live arm.
    let blob = unsafe { tv.vval.v_blob };
    (!blob.is_null()).then_some(blob.cast_const())
}

/// The String argument 0 holds.
fn string_of(tv: &typval_T) -> *const c_char {
    // SAFETY: only reached under a `VAR_STRING` tag, which is what makes
    // `v_string` the live arm.
    unsafe { tv.vval.v_string }
}
