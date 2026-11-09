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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{from, str_arg_chk};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::typval::{NumBuf, blob_bytes, list_items, tv_check_str_or_nr};
use crate::eval::userfunc::{add_defer, can_add_defer};
use crate::ex_cmds::check_secure;
use crate::fileio::buffered::{FILE_APPEND, FILE_CREATE, FILE_MK_DIR, FILE_TRUNCATE, WriteFile};
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::option::vars::p_fs;
use crate::os::cshim::gettext;
use crate::os::fs::os_strerror;
use crate::path::full_name_save;
use crate::runtime::script_is_lua;
use crate::runtime::state::CURRENT_SCTX;
use crate::types::Failed;
use crate::types::{Blob, List, TypVal, VAR_BLOB, VAR_LIST, VAR_STRING, VarNumber};
use crate::{semsg, tr};
use core::ffi::{CStr, c_int};

// ---------------------------------------------------------------------
// The messages
// ---------------------------------------------------------------------

/// Report `msg`, translated.
fn err(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// Report `E80: Error while writing: %s`.
fn err_writing(error: c_int) {
    let why = msg_cstr(os_strerror(error));
    semsg!("E80: Error while writing: {why}");
}

// ---------------------------------------------------------------------
// The three writers
// ---------------------------------------------------------------------

/// Write every item of `list` as a line, `binary` suppressing the newline
/// after the last one.
///
/// A refusal has been said: E80, or an item with no string form, which
/// reports on its own and is the one exit that does not report a write error.
fn write_list(out: &mut WriteFile, list: Option<&List>, binary: bool) -> Result<(), Failed> {
    let items = list_items(list);
    let error;
    'failed: {
        for (i, li) in items.iter().enumerate() {
            let mut numbuf = NumBuf::new();
            let Ok(s) = numbuf.string_chk(&li.li_tv) else {
                return Err(Failed);
            };
            let bytes = s.to_bytes();
            let mut hunk_start = 0;
            let mut p = 0;
            loop {
                if p == bytes.len() || bytes[p] == b'\n' {
                    if p != hunk_start {
                        let written = out.write(&bytes[hunk_start..p]);
                        if written < 0 {
                            error = c_int::try_from(written).unwrap_or(c_int::MIN);
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
            if !binary || i + 1 < items.len() {
                let written = out.write(b"\n");
                if written < 0 {
                    error = c_int::try_from(written).unwrap_or(c_int::MIN);
                    break 'failed;
                }
            }
        }
        match out.flush() {
            Ok(()) => return Ok(()),
            Err(code) => error = code,
        }
    }
    err_writing(error);
    Err(Failed)
}

/// Write `data` and flush; E80 when that fails.
fn write_data(out: &mut WriteFile, data: &[u8]) -> Result<(), Failed> {
    let error;
    'failed: {
        if !data.is_empty() {
            let written = out.write(data);
            // Upstream tests against the length, not against zero, so a
            // short write reports the count it did accept as if it were a
            // code.
            if written < data.len().cast_signed() {
                error = c_int::try_from(written).unwrap_or(c_int::MIN);
                break 'failed;
            }
        }
        match out.flush() {
            Ok(()) => return Ok(()),
            Err(code) => error = code,
        }
    }
    err_writing(error);
    Err(Failed)
}

fn write_blob(out: &mut WriteFile, blob: Option<&Blob>) -> Result<(), Failed> {
    write_data(out, blob_bytes(blob))
}

// ---------------------------------------------------------------------
// The builtin
// ---------------------------------------------------------------------

/// Whether the sandbox forbids writing, having reported it.
fn secure() -> bool {
    check_secure()
}

/// Whether a deferred call can be registered, having reported if not.
fn can_defer() -> bool {
    can_add_defer().is_ok()
}

/// Whether the running script is Lua, which is what makes a String argument
/// mean blob data rather than a mistake.
fn in_lua_script() -> bool {
    script_is_lua(CURRENT_SCTX.get().sc_sid)
}

/// Register `delete({fname})` to run when the calling function returns --
/// the `D` flag.
fn defer_delete(fname: &CStr) {
    let full = ThinCString::from(full_name_save(fname, false));
    let mut tv = TypVal::string(Some(full));
    // The callee takes the argument's contents over.
    add_defer(c"delete", ::core::slice::from_mut(&mut tv));
}

/// `Ok` if the first argument is something this builtin can write; else the
/// refusal has been reported.
fn writable(args: &[TypVal]) -> Result<(), Failed> {
    // XXX: this logic is a bit weird because of how `decode_string` works
    // (#39328): it assigns VAR_BLOB when it finds a NUL in the Lua string,
    // and VAR_STRING when it does not.
    if args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) {
        return list_items(args[0].list_ref())
            .iter()
            .try_for_each(|item| tv_check_str_or_nr(&item.li_tv));
    }
    // A Lua string is always treated as blob data.
    if args.first().is_some_and(|arg| arg.v_type() == VAR_BLOB)
        || (args.first().is_some_and(|arg| arg.v_type() == VAR_STRING) && in_lua_script())
    {
        return Ok(());
    }
    let what = tr!("writefile() first argument must be a List or a Blob");
    semsg!("E475: Invalid argument: {what}");
    Err(Failed)
}

/// The flags of the third argument, or None having reported an unknown one.
#[derive(Debug)]
struct Flags {
    binary: bool,
    append: bool,
    defer: bool,
    do_fsync: bool,
    mkdir_p: bool,
}

impl Flags {
    fn read(args: &[TypVal]) -> Result<Self, Failed> {
        let mut numbuf = NumBuf::new();
        let mut f = Self {
            binary: false,
            append: false,
            defer: false,
            do_fsync: p_fs(),
            mkdir_p: false,
        };
        if args.len() <= 2 {
            return Ok(f);
        }
        let flags = str_arg_chk(args, 2, &mut numbuf).ok_or(Failed)?;
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
                    let rest = msg_cstr(from(flags, i));
                    semsg!("E5060: Unknown flag: {rest}");
                    return Err(Failed);
                }
            }
        }
        Ok(f)
    }
}

/// `writefile({object}, {fname} [, {flags}])`: the List, Blob or Lua string
/// written to the file, 0 on success and -1 on failure.
pub fn f_writefile(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1 as VarNumber);
    if secure() || writable(args).is_err() {
        return;
    }
    let Ok(flags) = Flags::read(args) else {
        return;
    };

    let mut buf = NumBuf::new();
    let Some(fname) = str_arg_chk(args, 1, &mut buf) else {
        return;
    };
    if flags.defer && !can_defer() {
        return;
    }
    if fname.to_bytes().is_empty() {
        err(c"E482: Can't open file with an empty name");
        return;
    }
    let open_flags = (if flags.append {
        FILE_APPEND
    } else {
        FILE_TRUNCATE
    }) | (if flags.mkdir_p {
        FILE_MK_DIR
    } else {
        FILE_CREATE
    }) | FILE_CREATE;
    let mut out = match WriteFile::open(fname, open_flags, 0o666) {
        Ok(out) => out,
        Err(error) => {
            let (name, why) = (msg_cstr(fname), msg_cstr(os_strerror(error)));
            semsg!("E482: Can't open file {name} for writing: {why}");
            return;
        }
    };

    // Before the first byte, so that the file goes away even when writing it
    // fails.  The order is observable and is upstream's.
    if flags.defer {
        defer_delete(fname);
    }

    let written = match args[0].v_type() {
        VAR_BLOB => write_blob(&mut out, args[0].blob_ref()),
        VAR_STRING => write_data(
            &mut out,
            args[0].string_ref().map_or(&[], ThinCString::as_bytes),
        ),
        _ => write_list(&mut out, args[0].list_ref(), flags.binary),
    };
    if written.is_ok() {
        result.write_number(0 as VarNumber);
    }
    if let Err(error) = out.close(flags.do_fsync) {
        let (name, why) = (msg_cstr(fname), msg_cstr(os_strerror(error)));
        semsg!("E80: Error when closing file {name}: {why}");
    }
}
