//! The parser userdatum and one parse.
//!
//! `vim._create_ts_parser(lang)` makes a [`Parser`]; its `parse` method
//! reads the source either from a Lua string or through `input_cb` from a
//! buffer, runs under `on_parser_progress` when given a timeout so a long
//! parse can be cancelled, and answers the new tree with the ranges that
//! changed.

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

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::NonNull;
use std::rc::Rc;

use super::lang::{check_language, checked_c_string};
use super::logger::{LogScope, Logger, parser_get_logger, parser_set_logger};
use super::ranges::{parser_get_ranges, parser_set_ranges, push_owned_ranges};
use super::tree::{check_tree, push_tree};
use super::*;
use crate::lua::stack::{Kind, Raise};
use crate::lua_functions;
use crate::os::time::os_hrtime;
use crate::types::LineNr;
use crate::winlayer::{self, Buf};

/// A parser, as its userdatum owns it, with the logger it was given.
/// Empty once collected.
#[derive(Default)]
pub(crate) struct Parser {
    parser: Option<NonNull<TSParser>>,
    pub(crate) logger: Option<Rc<Logger>>,
}

impl Drop for Parser {
    fn drop(&mut self) {
        if let Some(parser) = self.parser.take() {
            // SAFETY: the parser this userdatum owns, deleted once; its
            // logger's payload is dropped after it, with the field.
            unsafe { ts_parser_delete(parser.as_ptr()) };
        }
    }
}

impl Userdata for Parser {
    const METATABLE: &'static CStr = c"treesitter_parser";
}

lua_functions! {
    /// The methods of a parser.
    pub(crate) enum ParserMethods in PARSER_METHODS {
        0 => c"__tostring": parser_tostring,
        1 => c"parse": parser_parse,
        2 => c"reset": parser_reset,
        3 => c"set_included_ranges": parser_set_ranges,
        4 => c"included_ranges": parser_get_ranges,
        5 => c"_set_logger": parser_set_logger,
        6 => c"_logger": parser_get_logger,
    }
}

/// `vim._create_ts_parser(lang)`.
pub(crate) fn create_parser(stack: &mut Stack) -> Result<c_int, Raise> {
    let language = check_language(stack, At::arg(1))?;
    // SAFETY: a fresh parser, and a loaded language.
    unsafe {
        let parser = ts_parser_new();
        if !ts_parser_set_language(parser, language.as_ptr()) {
            ts_parser_delete(parser);
            let name = checked_c_string(stack, At::arg(1))?;
            let mut message = b"Failed to load language : ".to_vec();
            message.extend_from_slice(name.to_bytes());
            return Err(Raise::message(message));
        }
        stack.new_userdata(Parser {
            parser: NonNull::new(parser),
            logger: None,
        });
    }
    Ok(1)
}

/// `luaL_checkudata` for a parser: its `TSParser *`, refusing a collected
/// one.
pub(crate) fn check_parser(stack: &mut Stack, at: At) -> Result<*mut TSParser, Raise> {
    match stack.check_userdata::<Parser>(at)?.parser {
        Some(parser) => Ok(parser.as_ptr()),
        None => Err(Raise::argument(at.index(), "Parser has been deleted")),
    }
}

fn parser_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_bytes(b"<parser>");
    Ok(1)
}

/// How much of a line [`input_cb`] hands tree-sitter at a time.
const CHUNK_SIZE: usize = 256;

/// A buffer parse's source: the buffer, and the chunk the last read
/// answered, which tree-sitter reads until it asks again.
struct BufferInput {
    buffer: Buf,
    chunk: [u8; CHUNK_SIZE],
}

/// Tree-sitter's read callback over a buffer: up to [`CHUNK_SIZE`] bytes
/// of the line at `position`, with the line break after its end (unless
/// the last line has none).
///
/// # Safety
/// `payload` is the parse's [`BufferInput`], unaliased for the call, whose
/// buffer stays live for the whole parse; `bytes_read` is writable.
/// Tree-sitter's `TSInput` contract.
unsafe extern "C" fn input_cb(
    payload: *mut c_void,
    _byte_index: u32,
    position: TSPoint,
    bytes_read: *mut u32,
) -> *const c_char {
    // SAFETY: the caller's payload and out-parameter.
    let (input, bytes_read) = unsafe { (&mut *payload.cast::<BufferInput>(), &mut *bytes_read) };
    let buffer = input.buffer;
    let line_count = buffer.b_ml.ml_line_count;
    let row = LineNr::try_from(position.row).unwrap_or(LineNr::MAX);
    *bytes_read = 0;
    if row >= line_count {
        return c"".as_ptr();
    }
    let lnum = row + 1;
    let mut lines = buffer.lines();
    let line = lines.line(lnum);
    let Some(rest) = line.get(position.column as usize..) else {
        return c"".as_ptr();
    };
    let count = rest.len().min(CHUNK_SIZE);
    let chunk = &mut input.chunk;
    chunk[..count].copy_from_slice(&rest[..count]);
    // The memline keeps a NUL as an NL.
    for byte in &mut chunk[..count] {
        if *byte == b'\n' {
            *byte = 0;
        }
    }
    let mut read = count;
    if count < CHUNK_SIZE
        && (lnum != line_count
            || buffer.b_p_bin == 0 && buffer.b_p_fixeol != 0
            || lnum != buffer.b_no_eol_lnum && buffer.b_p_eol != 0)
    {
        chunk[count] = b'\n';
        read += 1;
    }
    *bytes_read = u32::try_from(read).expect("a chunk and a line break");
    chunk.as_ptr().cast::<c_char>()
}

/// When a parse with a timeout started, and the timeout.
struct ParseDeadline {
    start: u64,
    timeout: u64,
}

/// Tree-sitter's progress callback: `true` cancels the parse once its
/// deadline has passed.
///
/// # Safety
/// `state` points at a live `TSParseState` whose payload is the parse's
/// [`ParseDeadline`].
unsafe extern "C" fn on_parser_progress(state: *mut TSParseState) -> bool {
    // SAFETY: the caller's state and payload.
    let deadline = unsafe { &*(*state).payload.cast_const().cast::<ParseDeadline>() };
    os_hrtime().wrapping_sub(deadline.start) >= deadline.timeout
}

/// The source `parse` was asked to read.
enum Source {
    String,
    Buffer(Buf),
}

/// `parser:parse(old_tree, source, include_bytes, timeout_ns)`: the new
/// tree and the ranges that changed since `old_tree` (or the included
/// ranges); nothing when a timeout cancelled the parse.
fn parser_parse(stack: &mut Stack) -> Result<c_int, Raise> {
    let parser = check_parser(stack, At::arg(1))?;
    let old_tree = if stack.kind(At::arg(2)) == Kind::Nil {
        core::ptr::null()
    } else {
        check_tree(stack, At::arg(2))?
    };
    let source = match stack.kind(At::arg(3)) {
        Kind::String => Source::String,
        Kind::Number => {
            let handle = wrap_u32(stack.to_integer(At::arg(3))).cast_signed();
            // The parse runs no editor code, so the buffer the registry
            // answers stays live for all of it.
            match winlayer::buffer(handle) {
                Some(buffer) => Source::Buffer(buffer),
                None => {
                    return Err(Raise::argument(
                        3,
                        format!("invalid buffer handle: {handle}"),
                    ));
                }
            }
        }
        _ => {
            return Err(Raise::argument(
                3,
                "expected either string or buffer handle",
            ));
        }
    };
    let include_bytes = stack.top().index() >= 4 && stack.to_boolean(At::arg(4));
    // An absent timeout (none, not nil) reads as 0, as upstream's does.
    let timeout = (stack.kind(At::arg(5)) != Kind::Nil)
        .then(|| u64::try_from(stack.to_integer(At::arg(5)).cast_unsigned()).unwrap_or(u64::MAX));

    let log = LogScope::begin(stack, At::arg(1));
    let new_tree = match source {
        Source::String => {
            let text = stack.string(At::arg(3)).unwrap_or_default();
            // `(uint32_t)len`: the low 32 bits.
            let length = u32::try_from(text.len() & 0xFFFF_FFFF).expect("masked to 32 bits");
            // SAFETY: the parser and tree the userdata own, and the bytes
            // of the Lua string in slot 3, alive for the call.
            unsafe { ts_parser_parse_string(parser, old_tree, text.as_ptr().cast(), length) }
        }
        Source::Buffer(buffer) => {
            let mut input = BufferInput {
                buffer,
                chunk: [0; CHUNK_SIZE],
            };
            let reader = TSInput {
                payload: (&raw mut input).cast::<c_void>(),
                read: Some(input_cb),
                encoding: INPUT_ENCODING_UTF8,
                decode: None,
            };
            match timeout {
                Some(timeout) => {
                    let mut deadline = ParseDeadline {
                        start: os_hrtime(),
                        timeout,
                    };
                    let options = TSParseOptions {
                        payload: (&raw mut deadline).cast::<c_void>(),
                        progress_callback: Some(on_parser_progress),
                    };
                    // SAFETY: as above, with the input and deadline this
                    // frame owns for the call.
                    unsafe { ts_parser_parse_with_options(parser, old_tree, reader, options) }
                }
                // SAFETY: as above.
                None => unsafe { ts_parser_parse(parser, old_tree, reader) },
            }
        }
    };
    if log.is_some_and(|log| log.finish(stack)) {
        if !new_tree.is_null() {
            // SAFETY: the tree just made, which nothing else has seen.
            unsafe { ts_tree_delete(new_tree) };
        }
        return Err(Raise::message("treesitter logger callback failed"));
    }

    if new_tree.is_null() {
        // SAFETY: the parser the userdatum owns.
        if unsafe { ts_parser_language(parser) }.is_null() {
            return Err(Raise::message(
                "Language was unset, or has an incompatible ABI.",
            ));
        }
        return Ok(0);
    }
    let mut count: u32 = 0;
    // SAFETY: the old tree and the new one; the array is allocated through
    // the editor's allocator and handed over.
    let changed = unsafe {
        if old_tree.is_null() {
            ts_tree_included_ranges(new_tree, &raw mut count)
        } else {
            ts_tree_get_changed_ranges(old_tree, new_tree, &raw mut count)
        }
    };
    push_tree(stack, new_tree);
    // SAFETY: the array just allocated for us.
    unsafe { push_owned_ranges(stack, changed, count, include_bytes) };
    Ok(2)
}

fn parser_reset(stack: &mut Stack) -> Result<c_int, Raise> {
    let parser = check_parser(stack, At::arg(1))?;
    // SAFETY: the parser the userdatum owns.
    unsafe { ts_parser_reset(parser) };
    Ok(0)
}
