//! `:lua`, `:luado`, `:luafile` and sourcing a file.
//!
//! [`ex_luado`] is the one with a shape of its own: it compiles the body once
//! into a function of `(line, linenr)` and runs it over the range, replacing
//! each line by what the function returns.  [`nlua_exec_file`] is what
//! `:luafile` and the runtime loader both reach.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;

use super::{main_stack, nlua_error, nlua_pcall, nlua_typval_exec, until_nul};
use crate::change::inserted_bytes;
use crate::cursor::check_cursor;
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::ex_getln::script_get;
use crate::getchar::state::got_int;
use crate::lua::stack::{At, Kind};
use crate::memline::ml_replace_owned;
use crate::memory::XString;
use crate::message::{e_argreq, emsg};
use crate::narrow::len_as_int;
use crate::os::cshim::gettext;
use crate::os::fileio::ReadFile;
use crate::runtime::cmd_source_buffer;
use crate::types::{CmdIdx, ColNr, ExArg, LineNr, lua_Number};
use crate::undo::u_save;
use crate::winlayer::{Buf, Win};

/// The wrapper `:luado`'s body is compiled inside, so each line is one call.
const DOSTART: &CStr = c"return function(line, linenr) ";
const DOEND: &CStr = c" end";

/// `:lua =expr` and `:= expr` are shorthand for `vim._print(true, expr)`:
/// the text before the expression and after it.
const PRINT_OPEN: &[u8] = b"vim._print(true, ";
const PRINT_CLOSE: &[u8] = b")";

/// `code` as `:lua` runs it: wrapped in [`PRINT_OPEN`]/[`PRINT_CLOSE`] for
/// `:=expr` and `:lua =expr`, as it is otherwise.
fn print_wrapped(cmdidx: CmdIdx, code: XString) -> XString {
    if cmdidx != CmdIdx::equal && code.first() != Some(&b'=') {
        return code;
    }
    // `:=expr` has no `=` to skip; `:lua =expr` does.
    let rest = &code[usize::from(cmdidx != CmdIdx::equal)..];
    let mut wrapped = XString::with_capacity(PRINT_OPEN.len() + rest.len() + PRINT_CLOSE.len());
    wrapped.push_bytes(PRINT_OPEN);
    wrapped.push_bytes(rest);
    wrapped.push_bytes(PRINT_CLOSE);
    wrapped
}

/// How much [`nlua_exec_file`] reads from stdin at a time.
const STDIN_CHUNK: usize = 64;

/// `:lua {chunk}`, `:lua ={expr}` and `:={expr}`.
pub fn ex_lua(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) == 0 {
        // `:{range}lua` with no body sources the range as Lua.
        if excmd.addr_count > 0 {
            cmd_source_buffer(excmd, true);
        } else {
            emsg(gettext(e_argreq));
        }
        return;
    }

    let Some(code) = script_get(excmd).filter(|_| !excmd.skip) else {
        return;
    };
    let code = print_wrapped(excmd.cmdidx, code);
    nlua_typval_exec(&code, c":lua", &[], false, None);
}

/// `:luado {body}`: run `body` over each line of the range as
/// `function(line, linenr)`, replacing the line with a string result.
///
/// The loop stops early if the body changed buffers or shortened the one it
/// is walking, which is the only protection against it editing under itself.
/// A NUL in the result stands for a newline, as `:s` treats it.
pub fn ex_luado(excmd: &mut ExArg) {
    if u_save(excmd.line1 - 1, excmd.line2 + 1).is_err() {
        emsg(gettext(c"cannot save undo information"));
        return;
    }
    let chunk = [DOSTART.to_bytes(), excmd.line.arg(), DOEND.to_bytes()].concat();
    let mut stack = main_stack();
    if stack.load(&chunk, Some(c":luado")).is_err() {
        nlua_error(&mut stack, c"E5109: Lua: %.*s");
        return;
    }
    if nlua_pcall(&mut stack, 0, 1).is_err() {
        nlua_error(&mut stack, c"E5110: Lua: %.*s");
        return;
    }

    let was_curbuf = Buf::current().handle();
    let mut l: LineNr = excmd.line1;
    while l <= excmd.line2 {
        if l > Buf::current().b_ml.ml_line_count {
            break;
        }
        stack.push_value(At::TOP);
        let old_line_len = {
            let mut lines = Buf::current().lines();
            let old_line = lines.line(l);
            // A memline line holds no NUL (it stores one as an NL), so its
            // length is where `lua_pushstring` would have stopped.
            stack.push_bytes(old_line);
            len_as_int(old_line.len())
        };
        stack.push_number(lua_Number::from(l));
        if nlua_pcall(&mut stack, 2, 1).is_err() {
            nlua_error(&mut stack, c"E5111: Lua: %.*s");
            break;
        }
        if Buf::current().handle() != was_curbuf || l > Buf::current().b_ml.ml_line_count {
            break;
        }
        if stack.is_string(At::TOP) {
            let new_line = stack.to_bytes(At::TOP).unwrap_or_default();
            let new_line_len = len_as_int(new_line.len());
            let _ = ml_replace_owned(l, nul_to_nl(new_line));
            inserted_bytes(l, 0 as ColNr, old_line_len, new_line_len);
        }
        stack.pop(1);
        l += 1;
    }
    stack.pop(1);
    check_cursor(Win::current());
    redraw_curbuf_later(UPD_NOT_VALID);
}

/// `line` as a buffer line: a NUL in it is stored as the NL the memline uses
/// for one.
fn nul_to_nl(line: &[u8]) -> XString {
    XString::from_vec(
        line.iter()
            .map(|&b| if b == 0 { b'\n' } else { b })
            .collect(),
    )
}

/// `:luafile {path}`.
pub fn ex_luafile(excmd: &mut ExArg) {
    nlua_exec_file(excmd.line.cstr_from(excmd.line.arg));
}

/// Read the whole of stdin. `None` on a read failure or an interrupt.
fn read_stdin() -> Option<Vec<u8>> {
    let mut stdin = ReadFile::stdin().ok()?;
    let mut text = Vec::with_capacity(STDIN_CHUNK);
    let mut chunk = [0u8; STDIN_CHUNK];
    loop {
        if got_int.get() {
            let _ = stdin.close();
            return None;
        }
        let Ok(read_size) = usize::try_from(stdin.read(&mut chunk)) else {
            let _ = stdin.close();
            return None;
        };
        text.extend_from_slice(&chunk[..read_size]);
        if read_size < STDIN_CHUNK {
            break;
        }
    }
    let _ = stdin.close();
    Some(text)
}

/// Run the Lua file at `path`, or stdin for `-`.
pub fn nlua_exec_file(path: &CStr) -> bool {
    let mut stack = main_stack();
    if path != c"-" {
        stack.get_global(c"loadfile");
        stack.push_bytes(path.to_bytes());
    } else {
        let Some(text) = read_stdin() else {
            return false;
        };
        stack.get_global(c"loadstring");
        // `lua_pushstring` of the text: up to its first NUL.
        stack.push_bytes(until_nul(&text));
    }

    // `loadfile`/`loadstring` answer either the chunk and nil, or nil and
    // the syntax error.
    if nlua_pcall(&mut stack, 1, 2).is_err() {
        nlua_error(&mut stack, c"E5111: Lua: %.*s");
        return false;
    }
    if stack.kind(At::from_top(2)) == Kind::Nil {
        nlua_error(&mut stack, c"E5112: Lua chunk: %.*s");
        debug_assert!(stack.is_nil(At::TOP));
        stack.pop(1);
        return false;
    }
    debug_assert!(stack.is_nil(At::TOP));
    stack.pop(1);
    if nlua_pcall(&mut stack, 0, 0).is_err() {
        nlua_error(&mut stack, c"E5113: Lua chunk: %.*s");
        return false;
    }
    true
}
