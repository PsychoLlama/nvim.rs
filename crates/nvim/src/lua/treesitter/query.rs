//! Compiling a query, and inspecting or disabling parts of it.
//!
//! `vim._ts_parse_query(lang, source)` compiles the query text against a
//! loaded language and turns a `TSQueryError` into the pointed message
//! [`query_error_message`] builds (row and column, the offending word, and
//! the line with a caret under it). `inspect` reports a compiled query's
//! patterns, captures and predicates back to Lua.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr::NonNull;

use super::lang::check_language;
use super::*;
use crate::lua::stack::Raise;
use crate::lua::state::tslua_query_parse_count;
use crate::lua_functions;

/// A compiled query, as its userdatum owns it. Empty once collected.
#[derive(Default)]
pub(crate) struct Query(Option<NonNull<TSQuery>>);

impl Drop for Query {
    fn drop(&mut self) {
        if let Some(query) = self.0.take() {
            // SAFETY: the query this userdatum owns, deleted once.
            unsafe { ts_query_delete(query.as_ptr()) };
        }
    }
}

impl Userdata for Query {
    const METATABLE: &'static CStr = c"treesitter_query";
}

lua_functions! {
    /// The methods of a query.
    pub(crate) enum QueryMethods in QUERY_METHODS {
        0 => c"__tostring": query_tostring,
        1 => c"inspect": query_inspect,
        2 => c"disable_capture": query_disable_capture,
        3 => c"disable_pattern": query_disable_pattern,
    }
}

/// `vim._ts_parse_query(lang, source)`.
pub(crate) fn parse_query(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() < 2 || !stack.is_string(At::arg(1)) || !stack.is_string(At::arg(2)) {
        return Err(Raise::message("string expected"));
    }
    let language = check_language(stack, At::arg(1))?;
    tslua_query_parse_count.set(tslua_query_parse_count.get().wrapping_add(1));
    let source = stack.to_bytes(At::arg(2)).unwrap_or_default();
    // `(uint32_t)len`: the low 32 bits.
    let length = u32::try_from(source.len() & 0xFFFF_FFFF).expect("masked to 32 bits");
    let mut error_offset: u32 = 0;
    let mut error_type: TSQueryError = QUERY_ERROR_NONE;
    // SAFETY: a loaded language and the bytes of the Lua string in slot 2.
    let query = unsafe {
        ts_query_new(
            language.as_ptr(),
            source.as_ptr().cast(),
            length,
            &raw mut error_offset,
            &raw mut error_type,
        )
    };
    let Some(query) = NonNull::new(query) else {
        let message = query_error_message(source, error_offset.cast_signed(), error_type);
        return Err(Raise::message(message));
    };
    stack.new_userdata(Query(Some(query)));
    Ok(1)
}

/// What each kind of query error is called.
fn query_error_name(error_type: TSQueryError) -> &'static [u8] {
    match error_type {
        1 => b"Invalid syntax:\n",
        2 => b"Invalid node type ",
        3 => b"Invalid field name ",
        4 => b"Invalid capture name ",
        5 => b"Impossible pattern:\n",
        _ => b"error",
    }
}

/// The message for a query `source` that failed to compile at
/// `error_offset`: upstream's `query_err_string`, which writes into 1,025
/// bytes and so says at most 1,024.
fn query_error_message(source: &[u8], error_offset: c_int, error_type: TSQueryError) -> Vec<u8> {
    // The lines are found as the C found them: in the C string, up to the
    // first NUL.
    let text = until_nul(source);
    let mut line_start: c_int = 0;
    let mut row: c_int = 0;
    let mut error_line = None;
    loop {
        let rest = text
            .get(usize::try_from(line_start).unwrap_or(usize::MAX)..)
            .unwrap_or_default();
        let newline = rest.iter().position(|&byte| byte == b'\n');
        let length = newline.unwrap_or(rest.len());
        let line_end = line_start.saturating_add(c_int::try_from(length).unwrap_or(c_int::MAX));
        if line_end > error_offset {
            error_line = Some(&rest[..length]);
            break;
        }
        line_start = line_end + 1;
        row += 1;
        if newline.is_none() {
            break;
        }
    }
    let column = error_offset - line_start;

    let mut message = format!("Query error at {}:{}. ", row + 1, column + 1).into_bytes();
    message.extend_from_slice(query_error_name(error_type));
    if [
        QUERY_ERROR_NODE_TYPE,
        QUERY_ERROR_FIELD,
        QUERY_ERROR_CAPTURE,
    ]
    .contains(&error_type)
    {
        let at = usize::try_from(error_offset).unwrap_or(usize::MAX);
        let suffix = source.get(at..).unwrap_or_default();
        let is_anonymous = error_type == QUERY_ERROR_NODE_TYPE
            && at.checked_sub(1).and_then(|before| source.get(before)) == Some(&b'"');
        let mut length = 0;
        if is_anonymous {
            // Up to an unescaped double quote.
            let mut backslashes = 0;
            while let Some(&byte) = suffix.get(length) {
                if byte == b'"' && backslashes % 2 == 0 {
                    break;
                }
                backslashes = if byte == b'\\' { backslashes + 1 } else { 0 };
                length += 1;
            }
        } else {
            // Up to the end of the identifier.
            while suffix
                .get(length)
                .is_some_and(|&byte| byte.is_ascii_alphanumeric() || b"_-.".contains(&byte))
            {
                length += 1;
            }
        }
        message.push(b'"');
        message.extend_from_slice(until_nul(&suffix[..length]));
        message.extend_from_slice(b"\":\n");
    }
    match error_line {
        None => message.extend_from_slice(b"Unexpected EOF\n"),
        Some(line) => {
            message.extend_from_slice(line);
            message.push(b'\n');
            // `%*s` of "": the width's spaces, whichever its sign.
            message.resize(message.len() + column.unsigned_abs() as usize, b' ');
            message.extend_from_slice(b"^\n");
        }
    }
    message.truncate(1024);
    message
}

/// `luaL_checkudata` for a query: its `TSQuery *`, refusing a collected
/// one.
pub(crate) fn check_query(stack: &mut Stack, at: At) -> Result<*mut TSQuery, Raise> {
    match stack.check_userdata::<Query>(at)?.0 {
        Some(query) => Ok(query.as_ptr()),
        None => Err(Raise::argument(at.index(), "TSQuery expected")),
    }
}

fn query_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_bytes(b"<query>");
    Ok(1)
}

/// The predicate steps of `pattern`, the query's own.
///
/// # Safety
/// `query` is live for `'a`.
unsafe fn predicate_steps<'a>(query: *const TSQuery, pattern: u32) -> &'a [TSQueryPredicateStep] {
    let mut count: u32 = 0;
    // SAFETY: the caller's live query; the array is its own.
    unsafe {
        let steps = ts_query_predicates_for_pattern(query, pattern, &raw mut count);
        if steps.is_null() {
            return &[];
        }
        core::slice::from_raw_parts(steps, count as usize)
    }
}

/// A string the query interned (`which` answers it with its length).
///
/// # Safety
/// `query` is live for `'a`, and `which` one of tree-sitter's lookups.
unsafe fn query_string<'a>(
    query: *const TSQuery,
    id: u32,
    which: unsafe extern "C" fn(*const TSQuery, u32, *mut u32) -> *const c_char,
) -> &'a [u8] {
    let mut length: u32 = 0;
    // SAFETY: the caller's live query; the string is its own, `length`
    // bytes long.
    unsafe {
        let text = which(query, id, &raw mut length);
        core::slice::from_raw_parts(text.cast::<u8>(), length as usize)
    }
}

/// `query:inspect()`: `{patterns = {[i] = predicates}, captures = names}`,
/// each predicate a list of strings and capture ids.
fn query_inspect(stack: &mut Stack) -> Result<c_int, Raise> {
    let query = check_query(stack, At::arg(1))?;
    stack.new_table(0, 2);
    // SAFETY: the query the userdatum owns, throughout.
    let pattern_count = unsafe { ts_query_pattern_count(query) };
    stack.new_table(pattern_count as usize, 1);
    for pattern in 0..pattern_count {
        // SAFETY: as above.
        let steps = unsafe { predicate_steps(query, pattern) };
        if steps.is_empty() {
            continue;
        }
        stack.new_table(steps.len() / 4, 1);
        stack.new_table(3, 0);
        let mut predicate = 1;
        let mut item = 1;
        for step in steps {
            match step.type_0 {
                STEP_TYPE_DONE => {
                    stack.raw_set_index(At::from_top(2), predicate);
                    predicate += 1;
                    stack.new_table(3, 0);
                    item = 1;
                    continue;
                }
                STEP_TYPE_STRING => {
                    // SAFETY: as above.
                    let text =
                        unsafe { query_string(query, step.value_id, ts_query_string_value_for_id) };
                    stack.push_bytes(text);
                }
                STEP_TYPE_CAPTURE => stack.push_integer(integer(step.value_id) + 1),
                _ => std::process::abort(),
            }
            stack.raw_set_index(At::from_top(2), item);
            item += 1;
        }
        stack.pop(1);
        let slot = c_int::try_from(pattern)
            .unwrap_or(c_int::MAX)
            .saturating_add(1);
        stack.raw_set_index(At::from_top(2), slot);
    }
    stack.set_field(At::from_top(2), c"patterns");

    // SAFETY: as above.
    let capture_count = unsafe { ts_query_capture_count(query) };
    stack.new_table(capture_count as usize, 0);
    for capture in 0..capture_count {
        // SAFETY: as above.
        let name = unsafe { query_string(query, capture, ts_query_capture_name_for_id) };
        stack.push_bytes(name);
        let slot = c_int::try_from(capture)
            .unwrap_or(c_int::MAX)
            .saturating_add(1);
        stack.raw_set_index(At::from_top(2), slot);
    }
    stack.set_field(At::from_top(2), c"captures");
    Ok(1)
}

/// `query:disable_capture(name)`.
fn query_disable_capture(stack: &mut Stack) -> Result<c_int, Raise> {
    let query = check_query(stack, At::arg(1))?;
    let name = stack.check_bytes(At::arg(2))?;
    // `(uint32_t)len`: the low 32 bits.
    let length = u32::try_from(name.len() & 0xFFFF_FFFF).expect("masked to 32 bits");
    // SAFETY: the query the userdatum owns, and the bytes of the Lua string
    // in slot 2.
    unsafe { ts_query_disable_capture(query, name.as_ptr().cast(), length) };
    Ok(0)
}

/// `query:disable_pattern(index)`, one-based.
fn query_disable_pattern(stack: &mut Stack) -> Result<c_int, Raise> {
    let query = check_query(stack, At::arg(1))?;
    let pattern = wrap_u32(stack.check_integer(At::arg(2))?);
    // SAFETY: the query the userdatum owns.
    unsafe { ts_query_disable_pattern(query, pattern.wrapping_sub(1)) };
    Ok(0)
}
