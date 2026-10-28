//! `vim.regex()`: a Vimscript regexp as a Lua userdatum.
//!
//! [`regex`] compiles the pattern and pushes a [`Regex`] userdatum, whose
//! metatable [`add_regex_metatable`] registers; `match_str` and `match_line`
//! are its two methods, both reaching the same [`run_match`] core, and the
//! `Stack`'s `__gc` drops the compiled program.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use super::TRY_STATE_INIT;
use crate::api::private::helpers::{try_enter, try_leave};
use crate::cstr;
use crate::lua::stack::{At, Raise, Stack, Userdata};
use crate::lua_functions;
use crate::regexp::{
    OwnedProg, RE_AUTO, RE_MAGIC, RE_NOBREAK, RE_STRICT, vim_regcomp, vim_regexec,
};
use crate::types::{Error, Handle, LineNr, RegMatch, lua_Integer};
use crate::winlayer::{self, Buf};

/// A compiled program, as the userdatum `vim.regex()` answers holds it.
/// Empty once collected -- or, transiently, while a match has it out.
#[derive(Default)]
pub(crate) struct Regex(Option<OwnedProg>);

impl Userdata for Regex {
    const METATABLE: &'static CStr = c"nvim_regex";
}

lua_functions! {
    /// The methods of a `vim.regex()` object.
    enum RegexMethods in REGEX_METHODS {
        0 => c"__tostring": regex_tostring,
        1 => c"match_str": regex_match_str,
        2 => c"match_line": regex_match_line,
    }
}

/// Register the `vim.regex()` userdatum's metatable, which is also its
/// `__index`.
pub(crate) fn add_regex_metatable(stack: &mut Stack) {
    stack.new_userdata_metatable::<Regex>();
    stack.register_functions(REGEX_METHODS);
    stack.push_value(At::TOP); // [meta, meta]
    stack.set_field(At::from_top(2), c"__index"); // [meta]
    stack.pop(1); // don't use metatable now
}

/// Take the program out of the userdatum at slot 1, refusing anything else
/// there; [`put_program`] gives it back.
fn take_program(stack: &mut Stack) -> Result<Option<OwnedProg>, Raise> {
    Ok(stack.check_userdata::<Regex>(At::arg(1))?.0.take())
}

/// Give the program [`take_program`] took back to the userdatum at slot 1.
/// `vim_regexec` may have swapped it for a recompiled one; a userdatum left
/// with none is an internal error.
fn put_program(stack: &mut Stack, prog: Option<OwnedProg>) -> Result<(), Raise> {
    let missing = prog.is_none();
    if let Some(regex) = stack.to_userdata::<Regex>(At::arg(1)) {
        regex.0 = prog;
    }
    if missing {
        return Err(Raise::message("regex: internal error"));
    }
    Ok(())
}

/// Match `prog` against `subject`: the match's start and end byte offsets,
/// or `None`.
fn run_match(prog: &mut Option<OwnedProg>, subject: &CStr) -> Option<(usize, usize)> {
    let mut rm = RegMatch::new(prog.take(), false);
    let matched = vim_regexec(&mut rm, subject, 0);
    *prog = rm.regprog.take();
    rm.group(0)
        .filter(|_| matched)
        .map(|span| (span.start, span.end))
}

/// Push a match's two offsets, or nothing; answers how many were pushed.
fn push_match(stack: &mut Stack, span: Option<(usize, usize)>) -> c_int {
    match span {
        Some((start, end)) => {
            let as_lua = |n: usize| lua_Integer::try_from(n).unwrap_or(lua_Integer::MAX);
            stack.push_integer(as_lua(start));
            stack.push_integer(as_lua(end));
            2
        }
        None => 0,
    }
}

/// `regex:match_str(str)`.
fn regex_match_str(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.check_userdata::<Regex>(At::arg(1))?;
    stack.check_bytes(At::arg(2))?;
    let mut prog = take_program(stack)?;
    // `luaL_checkstring`'s C string, anchored in its slot for the match.
    let span = run_match(&mut prog, stack.c_string(At::arg(2)).unwrap_or_default());
    let nret = push_match(stack, span);
    put_program(stack, prog)?;
    Ok(nret)
}

/// The buffer `bufnr` names, `0` for the current one, if it is loaded.
fn loaded_buffer(bufnr: Handle) -> Option<Buf> {
    let buf = if bufnr != 0 {
        winlayer::buffer(bufnr)
    } else {
        Buf::current_or_none()
    };
    buf.filter(|b| b.b_ml.ml_mfp.is_some())
}

/// `luaL_checkinteger` narrowed to an `int` as the C casts do: the low 32
/// bits.
fn check_int(stack: &Stack, at: c_int) -> Result<c_int, Raise> {
    let n = stack.check_integer(At::arg(at))?;
    #[allow(clippy::cast_possible_truncation)]
    let narrowed = n as c_int;
    Ok(narrowed)
}

/// `regex:match_line(bufnr, rownr[, start[, end]])`.
///
/// The `start..end` stretch of the line is matched as a string of its own,
/// copied out of the buffer: upstream writes a NUL over the buffer's memory
/// at `end` and puts the byte back afterwards, which the copy makes
/// unnecessary.
fn regex_match_line(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.check_userdata::<Regex>(At::arg(1))?;

    let narg = stack.top().index();
    if narg < 3 {
        return Err(Raise::message("not enough args"));
    }

    let bufnr: Handle = check_int(stack, 2)?;
    let rownr: LineNr = check_int(stack, 3)?;
    let mut start: c_int = 0;
    let mut end: c_int = -1;
    if narg >= 4 {
        start = check_int(stack, 4)?;
    }
    if narg >= 5 {
        end = check_int(stack, 5)?;
        if end < 0 {
            return Err(Raise::message("invalid end"));
        }
    }

    let Some(buf) = loaded_buffer(bufnr) else {
        return Err(Raise::message("invalid buffer"));
    };

    if rownr >= buf.b_ml.ml_line_count {
        return Err(Raise::message("invalid row"));
    }

    let line = buf.lines().line_copy(rownr + 1);
    let len = line.len();

    let Some(start) = usize::try_from(start).ok().filter(|&start| start <= len) else {
        return Err(Raise::message("invalid start"));
    };
    let end = match usize::try_from(end) {
        Err(_) => len,
        Ok(end) if end <= len && end >= start => end,
        Ok(_) => return Err(Raise::message("invalid end")),
    };

    let subject = cstr::owned(&line[start..end]);
    let mut prog = take_program(stack)?;
    let span = run_match(&mut prog, &subject);
    let nret = push_match(stack, span);
    put_program(stack, prog)?;
    Ok(nret)
}

/// `__tostring`.
fn regex_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_bytes(b"<regex>");
    Ok(1)
}

/// `vim.regex(pattern)`: compile it and push the userdatum.
pub(crate) fn regex(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.check_bytes(At::arg(1))?;
    let mut err = Error::none();

    // vim_regcomp reports a bad pattern by throwing, so it runs bracketed.
    let mut tstate = TRY_STATE_INIT;
    try_enter(&mut tstate);
    // Pick the engine automatically, honour 'magic', reject what the
    // automaton cannot take -- and never check for CTRL-C: that polls
    // the event loop, which aborts when the match runs inside a luv
    // callback (the loop is already running), and `vim.regex` is
    // allowed in a fast context.
    let pattern = stack.c_string(At::arg(1)).unwrap_or_default();
    let prog = vim_regcomp(pattern, RE_AUTO | RE_MAGIC | RE_STRICT | RE_NOBREAK);
    err.absorb(try_leave(&tstate));

    if err.is_set() {
        let why = err.message_or_empty().to_bytes();
        return Err(Raise::message(
            [b"couldn't parse regex: ".as_slice(), why].concat(),
        ));
    } else if prog.is_none() {
        return Err(Raise::message("couldn't parse regex"));
    }

    stack.new_userdata(Regex(prog));
    Ok(1)
}
