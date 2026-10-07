//! `system()` and `systemlist()`: the argument vector and the captured
//! output.
//!
//! Both directions swap NUL and newline. A shell's stdin is a byte stream
//! with no way to carry a NUL, so `save_tv_as_string` writes a newline for
//! every NUL a List item held and vice versa; `get_system_output_as_rettv`
//! undoes it on the way back. That is why the two halves look asymmetric:
//! one builds a buffer, the other rewrites one in place.

#![forbid(unsafe_code)]

use crate::buffer::find_buf;
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{
    ListRef, NumBuf, list_iter, list_len, tv_get_number, tv_list_alloc, tv_list_alloc_ret,
};
use crate::eval::vars::{emsg_static, set_vim_var_nr};
use crate::eval::{NL, PROF_YES};
use crate::ex_cmds::check_secure;
use crate::memline::Lines;
use crate::memory::ThinCString;
use crate::message::e_invarg;
use crate::message::{msg_str, verbose_enter_scroll, verbose_leave_scroll};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::option::vars::p_verbose;
use crate::os::fs::executable_path;
use crate::os::shell::Argv;
use crate::os::shell::system::os_system_capture;
use crate::profile::do_profiling;
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::semsg;
use crate::smsg;
use crate::types::{
    EvalFuncData, Failed, IOSIZE, List, OptInt, TypVal, VAR_LIST, VAR_NUMBER, VAR_STRING,
    VAR_UNKNOWN, VarNumber, Vv, kListLenMayKnow,
};
use core::ffi::c_int;

/// Why [`tv_to_argv`] built no vector. Either way it has been reported.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArgvRefusal {
    /// Not a String or a List, an empty List, or an item that is not text.
    Invalid,
    /// The List's first item names nothing runnable.
    NotExecutable,
}

/// Build the argument vector for a String (run through `'shell'`) or a List
/// (run directly, its first item resolved to the program's path).
pub fn tv_to_argv(cmd_tv: &TypVal) -> Result<Argv, ArgvRefusal> {
    if cmd_tv.v_type() == VAR_STRING {
        let mut numbuf = NumBuf::new();
        return Ok(Argv::shell(numbuf.string(cmd_tv)));
    }
    if cmd_tv.v_type() != VAR_LIST {
        let what = "expected String or List";
        semsg!("E475: Invalid argument: {what}");
        return Err(ArgvRefusal::Invalid);
    }

    let items = cmd_tv.list_ref();
    let argc = list_len(items);
    if argc == 0 {
        emsg_static(e_invarg);
        return Err(ArgvRefusal::Invalid);
    }

    // The first item has to resolve to something runnable, and the
    // resolved path is what actually goes in slot 0.
    let mut numbuf = NumBuf::new();
    let first = &list_iter(items).next().expect("non-empty").li_tv;
    let Some(arg0) = numbuf.string_chk(first) else {
        return Err(ArgvRefusal::Invalid);
    };
    let Some(program) = executable_path(arg0, true) else {
        // Upstream formatted this into `IObuff`, which cut it there.
        let mut text = Vec::with_capacity(arg0.count_bytes() + 20);
        text.push(b'\'');
        text.extend_from_slice(arg0.to_bytes());
        text.extend_from_slice(b"' is not executable");
        text.truncate(IOSIZE as usize - 1);
        let (what, text) = (msg_cstr(c"cmd"), msg_bytes(&text));
        semsg!("E475: Invalid value for argument {what}: {text}");
        return Err(ArgvRefusal::NotExecutable);
    };

    let mut program = Some(ThinCString::from(program));
    let mut words = Vec::with_capacity(argc as usize);
    for arg in list_iter(items) {
        let Some(word) = numbuf.string_chk(&arg.li_tv) else {
            return Err(ArgvRefusal::Invalid);
        };
        // Slot 0 takes the resolved path rather than the item's spelling.
        words.push(
            program
                .take()
                .unwrap_or_else(|| ThinCString::from_cstr(word)),
        );
    }
    Ok(Argv::from_words(words))
}

/// Split captured output into a List of lines, undoing the NUL/newline
/// swap on the way.
fn string_to_list(output: &[u8], keepempty: bool) -> ListRef {
    // A trailing newline does not start an empty last line unless the
    // caller asked to keep one.
    let output = match output.split_last() {
        Some((&last, rest)) if !keepempty && c_int::from(last) == NL => rest,
        _ => output,
    };
    let mut list = tv_list_alloc(kListLenMayKnow as isize);
    encode_list_write(&mut list, output);
    list
}

/// The shared body of `system()` and `systemlist()`.
pub(crate) fn get_system_output_as_rettv(args: &[TypVal], result: &mut TypVal, retlist: bool) {
    let profiling = do_profiling.get() == PROF_YES;
    result.write_string(None);
    if check_secure() {
        return;
    }

    // With no input argument there is nothing to feed the command.
    let input = match args.get(1).map(|tv| save_tv_as_string(tv, false, false)) {
        Some(Err(Failed)) => return,
        Some(Ok(input)) => input,
        None => None,
    };

    let argv = match tv_to_argv(&args[0]) {
        Ok(argv) => argv,
        Err(refusal) => {
            // A command that does not exist reports -1 rather than a shell
            // exit status.
            if refusal == ArgvRefusal::NotExecutable {
                set_vim_var_nr(Vv::ShellError, -1);
            }
            return;
        }
    };

    if p_verbose() > 3 as OptInt {
        let cmdstr = argv.to_display();
        verbose_enter_scroll();
        let shown = msg_cstr(cmdstr.as_cstr());
        smsg!(0, "Executing command: \"{shown}\"");
        msg_str(c"\n\n");
        verbose_leave_scroll();
    }

    let wait_time = if profiling { prof_child_enter() } else { 0 };
    let (status, output) = os_system_capture(argv, input.as_deref().unwrap_or_default());
    if profiling {
        prof_child_exit(wait_time);
    }
    drop(input);
    set_vim_var_nr(Vv::ShellError, status as VarNumber);

    let Some(mut output) = output else {
        if retlist {
            tv_list_alloc_ret(result, 0);
        } else {
            result.write_string(Some(ThinCString::empty()));
        }
        return;
    };

    if retlist {
        // The `keepempty` argument is the third, so it is only read
        // when the second was given too.
        let keepempty = args.len() > 2 && tv_get_number(&args[2]) as c_int != 0;
        result.write_list(Some(string_to_list(&output, keepempty)));
    } else {
        // Undo the swap in place; the buffer is handed over as it is.
        for byte in output.iter_mut().filter(|byte| **byte == 0) {
            *byte = 1;
        }
        result.write_string(Some(ThinCString::from_vec(output)));
    }
}

/// `system()`
pub fn f_system(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_system_output_as_rettv(args, result, false)
}

/// `systemlist()`
pub fn f_systemlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_system_output_as_rettv(args, result, true)
}

/// Append `text` to `out`, writing a NUL for every newline it holds.
///
/// The swap is the module's convention: a child's standard input is a byte
/// stream with no way to carry a NUL, so the two trade places on the way
/// out and the reading half puts them back.
fn push_swapping_nl(out: &mut Vec<u8>, text: &[u8]) {
    out.extend(text.iter().map(|&c| if c == b'\n' { 0 } else { c }));
}

/// Render a typval as the byte stream a child process's stdin wants: a
/// String as it is, a Number as that buffer's whole text, a List one item
/// per line. `Err` for a coercion that failed (reported), `Ok(None)` for
/// nothing to send -- which an empty String is not.
///
/// Newlines in the text become NULs and the line separators are newlines,
/// which is the convention the reading half undoes.
pub fn save_tv_as_string(tv: &TypVal, endnl: bool, crlf: bool) -> Result<Option<Vec<u8>>, Failed> {
    let mut numbuf = NumBuf::new();
    match tv.v_type() {
        VAR_UNKNOWN => Ok(None),
        VAR_NUMBER => buffer_as_string(tv),
        VAR_LIST => Ok(list_as_string(tv.list_ref(), endnl, crlf)),
        _ => numbuf
            .bytes_chk(tv)
            .map(|text| Some(text.to_vec()))
            .ok_or(Failed),
    }
}

/// A Number names a buffer; its whole text is the input.
fn buffer_as_string(tv: &TypVal) -> Result<Option<Vec<u8>>, Failed> {
    let nr = tv.number_or_zero();
    let Some(buffer) = find_buf(nr as c_int) else {
        semsg!("E86: Buffer {} does not exist", nr);
        return Err(Failed);
    };

    // Each line up to its first NUL, on purpose: upstream counted bytes
    // with `strlen`, not whatever the memline records as the length.
    let mut lines = Lines::in_buffer(buffer);
    let mut out = Vec::new();
    for lnum in 1..=buffer.line_count() {
        let line = lines.line(lnum);
        let line = line.split(|&b| b == 0).next().unwrap_or_default();
        push_swapping_nl(&mut out, line);
        out.push(b'\n');
    }
    Ok((!out.is_empty()).then_some(out))
}

/// A List is one line per item.
fn list_as_string(list: Option<&List>, endnl: bool, crlf: bool) -> Option<Vec<u8>> {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let sep = if crlf { 2 } else { 1 };

    // Measure first, charging every item a separator. Each item is
    // converted twice, as upstream did, so a bad one complains twice.
    let measured: usize = list_iter(list)
        .map(|li| numbuf.bytes(&li.li_tv).len() + sep)
        .sum();
    if measured == 0 {
        return None;
    }

    let mut out = Vec::with_capacity(measured + 1);
    let count = list_iter(list).len();
    for (at, li) in list_iter(list).enumerate() {
        push_swapping_nl(&mut out, numbuf2.string(&li.li_tv).to_bytes());
        let last = at + 1 == count;
        if endnl || !last {
            if crlf {
                out.push(b'\r');
            }
            out.push(b'\n');
        }
    }
    Some(out)
}
