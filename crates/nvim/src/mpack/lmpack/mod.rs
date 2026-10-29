//! `lmpack.c`: the `vim.mpack` Lua module.
//!
//! Three userdata classes and two shorthands. `Unpacker` and `Packer` are
//! reusable, suspendable codecs — you hand them a partial buffer and they
//! hand back what they got plus where to resume — and `mpack.decode` /
//! `mpack.encode` are one-shot wrappers that run a codec over a table on the
//! stack. `Session` (in [`session`]) adds msgpack-RPC framing on top.
//!
//! **Every instance owns a private table.** Values in flight during a walk
//! have to outlive the call that produced them, and the walk cannot use the
//! Lua stack for that because it suspends. Upstream parks them in a table it
//! holds by registry reference; here the table is the userdatum's
//! environment, so the collector frees it with the instance and nothing
//! needs releasing. A walk pushes the table once and [`reference()`] /
//! [`geti`] / [`unreference`] are `luaL_ref` / `lua_rawgeti` / `luaL_unref`
//! against that slot.
//!
//! A walk never raises from inside the codec. A failure -- an ext handler
//! that errors, a value msgpack cannot spell -- is recorded by the walker
//! ([`walk::Failure`]), which stops the walk; the binding raises it once the
//! codec has returned. As upstream, an instance whose walk failed stays
//! "already working".
//!
//! Two things here are nvim's rather than upstream libmpack-lua's: the
//! `mtdict` field, which carries `vim.empty_dict()`'s metatable so an empty
//! *map* round-trips as a dict rather than a list, and `luaopen_mpack` being
//! called from `nlua_state_add_stdlib` instead of by `require`.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]

pub mod session;
pub mod walk;

use core::ffi::{CStr, c_int};

use self::walk::{LuaDecoder, LuaEncoder};
use crate::lua::ffi::{LUA_NOREF, LUA_REFNIL};
use crate::lua::stack::{At, Kind, Raise, Stack, Userdata};
use crate::lua_functions;
use crate::mpack::mpack_core::{MPACK_EOF, MPACK_ERROR};
use crate::mpack::object::{MPACK_EXCEPTION, MPACK_MAX_OBJECT_DEPTH, MPACK_NOMEM, Parser};
use crate::mpack::rpc::MPACK_RPC_MAX_REQUESTS;
use crate::types::{lua_Integer, lua_Number, mpack_uint32_t};

/// The walk every instance drives: a node holds the references of the value
/// it stands for, in the instance's private table.
pub(crate) type LuaParser = Parser<walk::NodeRefs>;

/// The shared userdatum standing in for msgpack `nil`, which Lua's own `nil`
/// cannot represent inside a table.
const NIL_NAME: &CStr = c"mpack.NIL";
/// `vim.empty_dict()`'s metatable, put in the registry by nvim's Lua setup.
const EMPTY_DICT_NAME: &CStr = c"mpack.empty_dict";

/// How much an encoding walk writes between two looks at its status
/// (`luaL_Buffer`'s capacity, which upstream wrote into).
const BUFFER_SIZE: usize = 8192;

/// One `mpack.Unpacker` instance. Its private table is its environment.
pub struct Unpacker {
    /// Out of the instance while a walk runs (see `unpacking`).
    parser: Option<Box<LuaParser>>,
    /// The `ext` option: a table of decoders keyed by ext type code.
    ext: c_int,
    /// Set while a walk is in progress, so a decoder that re-enters the same
    /// instance is refused rather than corrupting it.
    unpacking: bool,
    mtdict: c_int,
    /// The blob being assembled, kept between calls that each bring part of
    /// it.
    string_buffer: Vec<u8>,
}

impl Default for Unpacker {
    fn default() -> Unpacker {
        Unpacker {
            parser: None,
            ext: LUA_NOREF,
            unpacking: false,
            mtdict: LUA_NOREF,
            string_buffer: Vec::new(),
        }
    }
}

impl Userdata for Unpacker {
    const METATABLE: &'static CStr = c"mpack.Unpacker";
}

/// One `mpack.Packer` instance. Its private table is its environment.
pub struct Packer {
    /// As [`Unpacker::parser`].
    parser: Option<Box<LuaParser>>,
    /// The `ext` option: a table of encoders keyed by *metatable*.
    ext: c_int,
    packing: bool,
    mtdict: c_int,
    /// The `is_bin` option: encode strings as `bin` rather than `str`.
    is_bin: bool,
    /// `is_bin` as a predicate, when it was given as a function.
    is_bin_fn: c_int,
}

impl Default for Packer {
    fn default() -> Packer {
        Packer {
            parser: None,
            ext: LUA_NOREF,
            packing: false,
            mtdict: LUA_NOREF,
            is_bin: false,
            is_bin_fn: LUA_NOREF,
        }
    }
}

impl Userdata for Packer {
    const METATABLE: &'static CStr = c"mpack.Packer";
}

lua_functions! {
    /// An `Unpacker`'s metamethods.
    enum UnpackerMethods in UNPACKER_METHODS {
        0 => c"__call": unpacker_unpack,
    }
}

lua_functions! {
    /// A `Packer`'s metamethods.
    enum PackerMethods in PACKER_METHODS {
        0 => c"__call": packer_pack,
    }
}

lua_functions! {
    /// The module's functions, and `tostring(mpack.NIL)`.
    enum MpackFunctions in MPACK_FUNCTIONS {
        0 => c"Unpacker": unpacker_new,
        1 => c"Packer": packer_new,
        2 => c"Session": session::new,
        3 => c"decode": decode,
        4 => c"encode": encode,
    }
}

lua_functions! {
    /// `tostring(mpack.NIL)`.
    enum NilFunctions in NIL_FUNCTIONS {
        0 => c"__tostring": nil_tostring,
    }
}

// ---------------------------------------------------------------------------
// The private table
// ---------------------------------------------------------------------------

/// Move the value on top of the stack into the private table at `reg` (an
/// absolute slot) and answer its reference.
pub fn reference(stack: &mut Stack, reg: At) -> c_int {
    stack.reference_in(reg)
}

/// Release a reference taken by [`reference()`].
pub fn unreference(stack: &mut Stack, reg: At, handle: c_int) {
    stack.unreference_in(reg, handle);
}

/// Push the value a reference names.
pub fn geti(stack: &mut Stack, reg: At, handle: c_int) {
    stack.raw_get_index(reg, handle);
}

/// Give the userdatum on top a fresh private table, and push the table:
/// its slot is what the instance's references are taken against.
fn new_private_table(stack: &mut Stack) -> At {
    stack.new_table(0, 0);
    stack.push_value(At::TOP);
    stack.set_environment(At::from_top(3));
    stack.top()
}

/// Replace the table on top of the stack with a shallow copy, so that later
/// edits to the caller's table cannot change an instance's options.
fn shallow_copy(stack: &mut Stack) {
    stack.new_table(0, 0);
    stack.push_nil();
    while stack.next(At::from_top(3)) {
        stack.push_value(At::from_top(2));
        stack.insert(At::from_top(2));
        stack.set_table(At::from_top(4));
    }
    stack.remove(At::from_top(2));
}

/// The `ext` option of the options table in slot 1, copied into the private
/// table at `reg`: its reference, or `LUA_NOREF` for none.
///
/// Upstream references whatever the field holds, so an options table with
/// no `ext` gave `LUA_REFNIL`, which the walks then read as a handler table:
/// decoding an ext value crashed (`lua_rawgeti` on nil, filed upstream) and
/// encoding a table with a metatable raised "attempt to index a nil value".
/// A missing `ext` is no handlers, as no options table is.
fn ext_option(stack: &mut Stack, reg: At) -> Result<c_int, Raise> {
    stack.get_field(At::arg(1), c"ext");
    if !stack.is_nil(At::TOP) {
        if stack.kind(At::TOP) != Kind::Table {
            return Err(Raise::message("\"ext\" option must be a table"));
        }
        shallow_copy(stack);
    }
    match reference(stack, reg) {
        LUA_REFNIL => Ok(LUA_NOREF),
        handle => Ok(handle),
    }
}

// ---------------------------------------------------------------------------
// Type checks and the NIL sentinel
// ---------------------------------------------------------------------------

/// Whether the value at `at` is the shared msgpack-nil userdatum.
pub fn is_nil_sentinel(stack: &mut Stack, at: At) -> bool {
    if stack.kind(at) != Kind::Userdata {
        return false;
    }
    let at = stack.absolute(at);
    stack.get_field(At::REGISTRY, NIL_NAME);
    let same = stack.raw_equal(At::TOP, at);
    stack.pop(1);
    same
}

/// Push it.
pub fn push_nil_sentinel(stack: &mut Stack) {
    stack.get_field(At::REGISTRY, NIL_NAME);
}

// ---------------------------------------------------------------------------
// Shared argument handling
// ---------------------------------------------------------------------------

/// The `start` argument of `unpacker(str[, start])` and
/// `session:receive(str[, start])`, as a zero-based offset into a `len`-byte
/// string. Always argument 3 in both.
pub fn start_offset(stack: &Stack, len: usize) -> Result<usize, Raise> {
    let start = if stack.top().index() == 3 {
        stack.check_number(At::arg(3))?
    } else {
        1.0
    };
    if start.is_nan() || start <= 0.0 {
        return Err(Raise::argument(
            3,
            "start position must be greater than zero",
        ));
    }
    if start.trunc() != start {
        return Err(Raise::argument(3, "start position must be an integer"));
    }
    if start as usize > len {
        return Err(Raise::argument(
            3,
            "start position must be less than or equal to the input string length",
        ));
    }
    Ok(start as usize - 1)
}

/// The length msgpack should give the value on top of the stack, and — for a
/// table — whether it is an array.
///
/// A Lua table is a msgpack array when its keys are exactly `1..n`. Deciding
/// that costs a full traversal, which is also where the count comes from.
/// An *empty* table tells the caller nothing, so `is_array` is left as the
/// caller set it. Adapted from lua-cmsgpack.
pub fn objlen(stack: &mut Stack, is_array: Option<&mut bool>) -> mpack_uint32_t {
    let mut len: usize = 0;
    if stack.kind(At::TOP) != Kind::Table {
        len = stack.raw_len(At::TOP);
    } else {
        let mut looks_like_array = true;
        let mut max: usize = 0;
        stack.push_nil();
        while stack.next(At::from_top(2)) {
            stack.pop(1); // the value; the key stays for `lua_next`
            let key = stack.to_number(At::TOP);
            looks_like_array = looks_like_array
                && stack.kind(At::TOP) == Kind::Number
                && key > 0.0
                && key as usize as lua_Number == key;
            if looks_like_array && key as usize > max {
                max = key as usize;
            }
            len += 1;
        }
        if let Some(is_array) = is_array
            && len > 0
        {
            *is_array = looks_like_array && max == len;
        }
    }
    // msgpack lengths are 32-bit; upstream saturates rather than failing.
    len.min(mpack_uint32_t::MAX as usize) as mpack_uint32_t
}

// ---------------------------------------------------------------------------
// Unpacker
// ---------------------------------------------------------------------------

/// `mpack.Unpacker([{ext = <table>}])`.
fn unpacker_new(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() > 1 {
        return Err(Raise::message("expecting at most 1 table argument"));
    }
    stack.new_userdata(Unpacker {
        parser: Some(Box::new(LuaParser::new(0))),
        ..Unpacker::default()
    });
    let reg = new_private_table(stack);
    stack.get_field(At::REGISTRY, EMPTY_DICT_NAME);
    let mtdict = reference(stack, reg);
    let mut ext = LUA_NOREF;
    if stack.kind(At::arg(1)) == Kind::Table {
        ext = ext_option(stack, reg)?;
    }
    stack.pop(1);
    let unpacker = stack.to_userdata::<Unpacker>(At::TOP).expect("just made");
    unpacker.mtdict = mtdict;
    unpacker.ext = ext;
    Ok(1)
}

/// Decode one value out of `input`, leaving it on top of the stack and
/// advancing `input`, with the [`Unpacker`] at `at` (an absolute slot) and
/// its private table at `reg`. Answers `MPACK_EOF` when the input ran out
/// first.
///
/// Growing the parser here is what lets a document nest arbitrarily deep;
/// `parse` rolls the input back to the token that would not fit, so the
/// retry starts in exactly the same place.
pub fn unpack_into_registry(
    stack: &mut Stack,
    at: At,
    reg: At,
    input: &mut &[u8],
) -> Result<c_int, Raise> {
    let unpacker = stack
        .to_userdata::<Unpacker>(at)
        .expect("checked by the caller");
    if unpacker.unpacking {
        return Err(Raise::message(
            "Unpacker instance already working. Use another Unpacker or mpack.decode() if you \
             need to decode from the ext handler",
        ));
    }
    let mut parser = unpacker
        .parser
        .take()
        .expect("an idle unpacker has its parser");
    let (ext, mtdict) = (unpacker.ext, unpacker.mtdict);
    let buffer = core::mem::take(&mut unpacker.string_buffer);
    unpacker.unpacking = true;
    let mut decoder = LuaDecoder::new(stack, reg, ext, mtdict, buffer);
    let status = loop {
        let status = parser.parse(input, &mut decoder);
        if status != MPACK_NOMEM {
            break status;
        }
        // `parse` rolled the input back to the token that would not fit.
        parser.grow();
    };
    let (buffer, failure) = decoder.finish();
    let unpacker = stack
        .to_userdata::<Unpacker>(at)
        .expect("on the stack throughout");
    unpacker.parser = Some(parser);
    unpacker.string_buffer = buffer;
    if let Some(failure) = failure {
        // As upstream, whose walk was left by a longjmp: still "working".
        return Err(failure.raise(stack));
    }
    unpacker.unpacking = false;
    if status == MPACK_ERROR as c_int {
        return Err(Raise::message("invalid msgpack string"));
    }
    Ok(status)
}

/// `unpacker(str[, start])` — the `__call` metamethod.
///
/// Answers the value (or nil, if the buffer held only part of one) and the
/// one-based offset to resume from.
fn unpacker_unpack(stack: &mut Stack) -> Result<c_int, Raise> {
    let argc = stack.top();
    if !(2..=3).contains(&argc.index()) {
        return Err(Raise::message("expecting between 2 and 3 arguments"));
    }
    stack.check_userdata::<Unpacker>(At::arg(1))?;
    let len = stack.check_bytes(At::arg(2))?.len();
    let offset = start_offset(stack, len)?;

    let mut view = stack.above(argc);
    let whole = view.check_bytes_below(At::arg(2))?;
    let mut input = &whole[offset..];
    view.push_environment(At::arg(1));
    let reg = view.top();
    let status = unpack_into_registry(&mut view, At::arg(1), reg, &mut input)?;
    // Only the value (or nil) is kept above the arguments. A walk resumed
    // over input that does not continue what it was given can leave a
    // scalar it could not store (upstream leaves it on the stack below its
    // results; its `assert` is off in release builds).
    if status == MPACK_EOF as c_int {
        view.set_top(argc);
        view.push_nil();
    } else {
        view.replace(reg);
        view.set_top(reg);
    }
    let consumed = whole.len() - input.len();
    view.push_integer(lua_Integer::try_from(consumed).unwrap_or(lua_Integer::MAX) + 1);
    debug_assert_eq!(view.top().index(), argc.index() + 2);
    Ok(2)
}

// ---------------------------------------------------------------------------
// Packer
// ---------------------------------------------------------------------------

/// `mpack.Packer([{ext = <table>, is_bin = <bool|fn>}])`.
fn packer_new(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() > 1 {
        return Err(Raise::message("expecting at most 1 table argument"));
    }
    stack.new_userdata(Packer {
        parser: Some(Box::new(LuaParser::new(0))),
        ..Packer::default()
    });
    let reg = new_private_table(stack);
    stack.get_field(At::REGISTRY, EMPTY_DICT_NAME);
    let mtdict = reference(stack, reg);
    let mut ext = LUA_NOREF;
    let mut is_bin = false;
    let mut is_bin_fn = LUA_NOREF;
    if stack.kind(At::arg(1)) == Kind::Table {
        ext = ext_option(stack, reg)?;
        stack.get_field(At::arg(1), c"is_bin");
        if stack.is_nil(At::TOP) {
            stack.pop(1);
        } else {
            let kind = stack.kind(At::TOP);
            if kind != Kind::Boolean && kind != Kind::Function {
                return Err(Raise::message(
                    "\"is_bin\" option must be a boolean or function",
                ));
            }
            is_bin = stack.to_boolean(At::TOP);
            if kind == Kind::Function {
                is_bin_fn = reference(stack, reg);
            } else {
                stack.pop(1);
            }
        }
    }
    stack.pop(1);
    let packer = stack.to_userdata::<Packer>(At::TOP).expect("just made");
    packer.mtdict = mtdict;
    packer.ext = ext;
    packer.is_bin = is_bin;
    packer.is_bin_fn = is_bin_fn;
    Ok(1)
}

/// Why an encoding walk did not finish.
enum Unfinished {
    /// The parser stack was too shallow and may not grow.
    TooDeep,
    /// The walker failed; its error is in the encoder.
    Stopped,
}

/// Run an encoding walk to completion, answering what it wrote.
///
/// `unparse` fills the window it is handed and answers `MPACK_EOF` when it
/// wants more room, or `MPACK_NOMEM` when the *parser* stack is too shallow:
/// `may_grow` says whether that may be fixed, which a shorthand's
/// fixed-depth parser may not.
fn unparse_all(
    parser: &mut LuaParser,
    encoder: &mut LuaEncoder<'_>,
    may_grow: bool,
) -> Result<Vec<u8>, Unfinished> {
    let mut out = Vec::with_capacity(BUFFER_SIZE);
    loop {
        let start = out.len();
        out.resize(start + BUFFER_SIZE, 0);
        let (status, written) = parser.unparse(&mut out[start..], encoder);
        out.truncate(start + written);
        match status {
            MPACK_EXCEPTION => return Err(Unfinished::Stopped),
            MPACK_NOMEM if !may_grow => return Err(Unfinished::TooDeep),
            MPACK_NOMEM => parser.grow(),
            status if status == MPACK_EOF as c_int => {}
            _ => return Ok(out),
        }
    }
}

/// `packer(value)` — the `__call` metamethod.
fn packer_pack(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() != 2 {
        return Err(Raise::message("expecting exactly 2 arguments"));
    }
    stack.check_userdata::<Packer>(At::arg(1))?;
    stack.push_environment(At::arg(1));
    let reg = stack.top();
    stack.push_value(At::arg(2));
    let root = reference(stack, reg);
    let packer = stack
        .to_userdata::<Packer>(At::arg(1))
        .expect("checked above");
    if packer.packing {
        return Err(Raise::message(
            "Packer instance already working. Use another Packer or mpack.encode() if you need \
             to encode from the ext handler",
        ));
    }
    let mut parser = packer.parser.take().expect("an idle packer has its parser");
    let options = walk::PackOptions {
        ext: packer.ext,
        mtdict: packer.mtdict,
        is_bin: packer.is_bin,
        is_bin_fn: packer.is_bin_fn,
    };
    packer.packing = true;
    let mut encoder = LuaEncoder::new(stack, reg, root, options);
    // A packer's own parser grows on demand.
    let written = unparse_all(&mut parser, &mut encoder, true);
    let failure = encoder.finish();
    let packer = stack
        .to_userdata::<Packer>(At::arg(1))
        .expect("on the stack throughout");
    packer.parser = Some(parser);
    if let Some(failure) = failure {
        // As upstream, whose walk was left by a longjmp: still "working".
        return Err(failure.raise(stack));
    }
    packer.packing = false;
    let Ok(written) = written else {
        unreachable!("only a stopped walk fails, and it left its failure");
    };
    unreference(stack, reg, root);
    stack.push_bytes(&written);
    Ok(1)
}

// ---------------------------------------------------------------------------
// The one-shot shorthands
// ---------------------------------------------------------------------------

/// `mpack.decode(str)`: the whole string, or an error.
///
/// Unlike `Unpacker`, this refuses a partial document and refuses trailing
/// bytes — which is what makes it a *decoder* rather than a stream reader.
fn decode(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() != 1 {
        return Err(Raise::message("expecting exactly 1 argument"));
    }
    stack.check_bytes(At::arg(1))?;
    let mut view = stack.above(At::arg(1));
    let mut input = view.check_bytes_below(At::arg(1))?;
    view.new_table(0, 0);
    let reg = view.top();
    view.get_field(At::REGISTRY, EMPTY_DICT_NAME);
    let mtdict = reference(&mut view, reg);

    let mut parser = LuaParser::new(0);
    let mut decoder = LuaDecoder::new(&mut view, reg, LUA_NOREF, mtdict, Vec::new());
    let status = parser.parse(&mut input, &mut decoder);
    let (_, failure) = decoder.finish();
    if let Some(failure) = failure {
        return Err(failure.raise(&mut view));
    }

    // The depth is fixed here: `decode` has nowhere to put a grown parser,
    // so a document deeper than 32 is an error rather than a reallocation.
    let message = match status {
        MPACK_NOMEM => "object was too deep to unpack",
        s if s == MPACK_EOF as c_int => "incomplete msgpack string",
        s if s == MPACK_ERROR as c_int => "invalid msgpack string",
        _ if !input.is_empty() => "trailing data in msgpack string",
        _ => {
            view.replace(reg);
            view.set_top(reg);
            return Ok(1);
        }
    };
    Err(Raise::message(message))
}

/// `mpack.encode(value)`.
fn encode(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() != 1 {
        return Err(Raise::message("expecting exactly 1 argument"));
    }
    stack.new_table(0, 0);
    let reg = stack.top();
    stack.push_value(At::arg(1));
    let root = reference(stack, reg);
    stack.get_field(At::REGISTRY, EMPTY_DICT_NAME);
    let mtdict = reference(stack, reg);

    let mut parser = LuaParser::new(0);
    let options = walk::PackOptions {
        ext: LUA_NOREF,
        mtdict,
        is_bin: false,
        is_bin_fn: LUA_NOREF,
    };
    let mut encoder = LuaEncoder::new(stack, reg, root, options);
    // `decode`/`encode` keep the default depth: too deep is an error here,
    // not a reallocation.
    let written = unparse_all(&mut parser, &mut encoder, false);
    if let Some(failure) = encoder.finish() {
        return Err(failure.raise(stack));
    }
    match written {
        Ok(written) => {
            stack.push_bytes(&written);
            Ok(1)
        }
        Err(_) => Err(Raise::message("object was too deep to pack")),
    }
}

/// `tostring(mpack.NIL)`.
fn nil_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    // Upstream passes `NIL_NAME` to `lua_pushfstring` as the format, with the
    // userdata's address as an argument it never reads: no `%` in the name,
    // so the string is the name itself.
    stack.push_bytes(NIL_NAME.to_bytes());
    Ok(1)
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Install `T`'s metatable, which is also its own `__index`, so the class's
/// methods are reachable on an instance.
fn register_class<T: Userdata>(stack: &mut Stack, methods: &[crate::lua::stack::Function]) {
    stack.new_userdata_metatable::<T>();
    stack.push_value(At::TOP);
    stack.set_field(At::from_top(2), c"__index");
    stack.register_functions(methods);
    stack.pop(1);
}

/// Build the `mpack` module table and leave it on top of `stack`.
pub fn luaopen_mpack(stack: &mut Stack) {
    register_class::<Unpacker>(stack, UNPACKER_METHODS);
    register_class::<Packer>(stack, PACKER_METHODS);
    register_class::<session::Session>(stack, session::SESSION_METHODS);

    // One shared userdatum stands for msgpack nil, so that `x == mpack.NIL`
    // works and survives being stored in a table. It lives in the registry,
    // shared by every state this is opened on.
    stack.get_field(At::REGISTRY, NIL_NAME);
    if stack.is_nil(At::TOP) {
        stack.new_empty_userdata();
        stack.new_table(0, 0);
        stack.register_functions(NIL_FUNCTIONS);
        stack.set_metatable(At::from_top(2));
        stack.set_field(At::REGISTRY, NIL_NAME);
    }
    stack.pop(1);

    stack.new_table(0, 0);
    stack.register_functions(MPACK_FUNCTIONS);
    stack.get_field(At::REGISTRY, NIL_NAME);
    stack.set_field(At::from_top(2), c"NIL");
}

/// Keep the depth and slot-count defaults visible next to the code that
/// doubles them.
const _: () = {
    assert!(MPACK_MAX_OBJECT_DEPTH == 32);
    assert!(MPACK_RPC_MAX_REQUESTS == 32);
};
