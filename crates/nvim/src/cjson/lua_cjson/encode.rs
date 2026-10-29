#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! `vim.json.encode`: a Lua value out, JSON text back.
//!
//! The walk is the Lua stack's own: the value being encoded is always on
//! top, tables are traversed with `lua_next` / `lua_rawgeti`, and the
//! recursion depth is bounded by [`ENCODE_MAX_DEPTH`] *and* by
//! `lua_checkstack`, because a metamethod could need slots this walk has
//! already spent. A refusal is a [`Raise`] answered back up the walk, with
//! nothing popped on the way: an error a `__len` metamethod raised is still
//! on top when the binding raises it.
//!
//! Two things upstream leaves as settings are hard here: NaN and infinity
//! always raise (`encode_invalid_numbers` is 0, so its "NaN"/"Infinity" and
//! "null" spellings are unreachable), and an unsupported value type always
//! raises (`encode_skip_unsupported_value_types` is 0, so the "skip it and
//! roll the output back" bookkeeping every container carried is unreachable
//! too, and is gone).
//!
//! Ported from Lua CJSON, Copyright (c) 2010-2012 Mark Pulford, under the
//! MIT license; the notice is reproduced in licenses/lua-cjson-LICENSE.txt.

use core::ffi::{CStr, c_int};

use super::{
    ENCODE_MAX_DEPTH, NUMBER_PRECISION, SPARSE_RATIO, SPARSE_SAFE, array_key, empty_array_key,
    fetch_config, push_registry,
};
use crate::cjson::fpconv::append_g_fmt;
use crate::lua::stack::{At, Kind, Raise, Stack};
use crate::narrow::len_as_int;

/// The JSON escape for `byte`, or `None` to emit it unchanged.
///
/// Upstream is a 256-entry table of `const char *`, and `escape_slash`
/// swaps in a whole second copy of it with one row changed. The table is
/// three ranges and seven exceptions, so this is that instead.
fn escape(byte: u8, escape_slash: bool) -> Option<&'static [u8]> {
    Some(match byte {
        0x08 => &b"\\b"[..],
        0x09 => b"\\t",
        0x0a => b"\\n",
        0x0c => b"\\f",
        0x0d => b"\\r",
        b'"' => b"\\\"",
        b'\\' => b"\\\\",
        b'/' if escape_slash => b"\\/",
        // Everything else below space, plus DEL, goes out as `\u00xx`.
        0x00..=0x1f | 0x7f => &HEX_ESCAPES[byte as usize],
        _ => return None,
    })
}

/// `\u00xx` for every byte, so the byte itself is the index.
static HEX_ESCAPES: [[u8; 6]; 256] = hex_escapes();

const fn hex_escapes() -> [[u8; 6]; 256] {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut table = [*b"\\u0000"; 256];
    let mut byte = 0;
    while byte < 256 {
        table[byte][4] = DIGITS[(byte >> 4) & 0xf];
        table[byte][5] = DIGITS[byte & 0xf];
        byte += 1;
    }
    table
}

/// Which buffer an appender writes into: the document, or the scratch the
/// sorted-object path lays keys out in before it knows their order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sink {
    Document,
    Keys,
}

/// One key of a `sort_keys` object: where its *encoded* text sits in
/// [`Encoder::keys`], and enough of the original to look the value up
/// again once the order is settled.
struct Key {
    offset: usize,
    length: usize,
    raw: RawKey,
}

/// A table key, as the second pass has to re-push it.
///
/// The string arm owns a copy rather than pointing into the Lua string,
/// which is what upstream does: its pointer stays valid only because the
/// table being encoded is still on the stack and still holds the key.
enum RawKey {
    Text(Vec<u8>),
    /// Upstream stores `lua_tointeger` here, not `lua_tonumber`, so a
    /// fractional key is *truncated* before the value is looked up and the
    /// lookup misses. Kept, and recorded in the docket.
    Number(isize),
}

struct Encoder {
    out: Vec<u8>,
    keys: Vec<u8>,
    /// The `indent` option, owned rather than borrowed: upstream keeps the
    /// `const char *` `luaL_checkstring` answered after popping both the
    /// string and the options table that held it, which leaves it alive
    /// only by the garbage collector's good graces.
    indent: Option<Vec<u8>>,
    escape_slash: bool,
    sort_keys: bool,
}

/// `json_encode_exception`: `Cannot serialise <type of the value at at>:
/// <reason>`.
fn refusal(stack: &Stack, at: At, reason: &str) -> Raise {
    let name = stack.kind_name(stack.kind(at)).to_bytes();
    let mut message = b"Cannot serialise ".to_vec();
    message.extend_from_slice(name);
    message.extend_from_slice(b": ");
    message.extend_from_slice(reason.as_bytes());
    Raise::message(message)
}

impl Encoder {
    fn sink(&mut self, which: Sink) -> &mut Vec<u8> {
        match which {
            Sink::Document => &mut self.out,
            Sink::Keys => &mut self.keys,
        }
    }

    /// Append the string at `at` with its contents escaped, no quotes. The
    /// slot holds a string (not a number to convert, which would break
    /// `lua_next`).
    fn append_string_contents(&mut self, stack: &Stack, at: At, into: Sink) {
        let bytes = stack.string(at).unwrap_or_default();
        let escape_slash = self.escape_slash;
        let out = self.sink(into);
        // Upstream reserves `length * 6`, the all-escapes worst case, and
        // then appends without checking. A `Vec` grows geometrically, so
        // the exact-fit reservation is the useful one and the `SIZE_MAX / 6`
        // overflow guard that came with the worst case is not needed.
        out.reserve(bytes.len());
        for &byte in bytes {
            match escape(byte, escape_slash) {
                Some(text) => out.extend_from_slice(text),
                None => out.push(byte),
            }
        }
    }

    fn append_string(&mut self, stack: &Stack, at: At) {
        self.out.push(b'"');
        self.append_string_contents(stack, at, Sink::Document);
        self.out.push(b'"');
    }

    /// Append the number at `at`.
    fn append_number(&mut self, stack: &Stack, at: At, into: Sink) -> Result<(), Raise> {
        let number = stack.to_number(at);
        if !number.is_finite() {
            // `encode_invalid_numbers` is 0. Upstream's other two settings
            // would spell these "NaN"/"Infinity" or "null".
            return Err(refusal(stack, at, "must not be NaN or Infinity"));
        }
        append_g_fmt(self.sink(into), number, NUMBER_PRECISION);
        Ok(())
    }

    fn append_newline_and_indent(&mut self, depth: c_int) {
        let Some(indent) = self.indent.as_ref() else {
            return;
        };
        self.out.push(b'\n');
        for _ in 0..depth {
            self.out.extend_from_slice(indent);
        }
    }

    /// `json_check_encode_depth`: three free slots, because a container
    /// needs a key, a value and room for the error message that says it
    /// could not get them.
    fn check_depth(stack: &mut Stack, depth: c_int) -> Result<(), Raise> {
        if depth <= ENCODE_MAX_DEPTH && stack.reserve(3) {
            return Ok(());
        }
        Err(Raise::message(format!(
            "Cannot serialise, excessive nesting ({depth})"
        )))
    }

    /// The table is on top.
    fn append_array(
        &mut self,
        stack: &mut Stack,
        depth: c_int,
        length: i64,
        raw: bool,
    ) -> Result<(), Raise> {
        // Function-local: a module-level const lands in ffigen's flat C
        // namespace (`unit-cdefs.h`).
        const BOUNDED_BY_LENGTH: &str = "an array index is bounded by the table's length";
        self.out.push(b'[');
        for index in 1..=length {
            if index > 1 {
                self.out.push(b',');
            }
            self.append_newline_and_indent(depth);
            // `raw` says whether `__index` is allowed to answer, which is
            // how a table that became an array through `__len` gets read.
            if raw {
                // Upstream's loop counter is an `int`, which is what
                // `lua_rawgeti` takes; `length` is a table's element count
                // (`lua_objlen`'s, already narrowed to an `int`, or
                // `array_length`'s, which raises on a sparse array), so
                // neither narrowing here can fail.
                stack.raw_get_index(At::TOP, c_int::try_from(index).expect(BOUNDED_BY_LENGTH));
            } else {
                // Lua's own integer type is `ptrdiff_t`.
                stack.push_integer(isize::try_from(index).expect(BOUNDED_BY_LENGTH));
                stack.get_table(At::from_top(2));
            }
            self.append_value(stack, depth)?;
            stack.pop(1);
        }
        if length > 0 {
            self.append_newline_and_indent(depth - 1);
        }
        self.out.push(b']');
        Ok(())
    }

    /// The table is on top.
    fn append_object(&mut self, stack: &mut Stack, depth: c_int) -> Result<(), Raise> {
        self.out.push(b'{');
        let mut written = 0;
        stack.push_nil();
        while stack.next(At::from_top(2)) {
            if written > 0 {
                self.out.push(b',');
            }
            written += 1;
            self.append_newline_and_indent(depth);

            // .., table, key, value
            let key = At::from_top(2);
            match stack.kind(key) {
                Kind::Number => {
                    self.out.push(b'"');
                    self.append_number(stack, key, Sink::Document)?;
                    self.out.extend_from_slice(b"\":");
                }
                Kind::String => {
                    self.append_string(stack, key);
                    self.out.push(b':');
                }
                _ => return Err(refusal(stack, key, "table key must be a number or string")),
            }
            if self.indent.is_some() {
                self.out.push(b' ');
            }
            self.append_value(stack, depth)?;
            stack.pop(1);
        }
        if written > 0 {
            self.append_newline_and_indent(depth - 1);
        }
        self.out.push(b'}');
        Ok(())
    }

    /// The `sort_keys` object: lay every key's *encoded* text out in
    /// [`Encoder::keys`], sort by that text, then emit.
    ///
    /// The sort is stable where upstream's `qsort` is not. Two keys can
    /// have the same encoded text — the number `1` and the string `"1"` —
    /// and upstream's order between them is whatever `qsort` did; `pairs()`
    /// order, which is what it sorts, is a function of the interpreter's
    /// hash seed anyway.
    ///
    /// The table is on top.
    fn append_object_sorted(&mut self, stack: &mut Stack, depth: c_int) -> Result<(), Raise> {
        // Nested sorted objects share the scratch, so each level takes back
        // only what it added.
        self.out.push(b'{');
        let base = self.keys.len();
        let mut keys: Vec<Key> = Vec::new();

        stack.push_nil();
        while stack.next(At::from_top(2)) {
            let offset = self.keys.len();
            let key = At::from_top(2);
            let raw = match stack.kind(key) {
                Kind::String => {
                    self.append_string_contents(stack, key, Sink::Keys);
                    RawKey::Text(stack.string(key).unwrap_or_default().to_vec())
                }
                Kind::Number => {
                    self.append_number(stack, key, Sink::Keys)?;
                    RawKey::Number(stack.to_integer(key))
                }
                _ => return Err(refusal(stack, key, "table key must be number or string")),
            };
            keys.push(Key {
                offset,
                length: self.keys.len() - offset,
                raw,
            });
            // This drops the value `lua_next` left.
            stack.pop(1);
        }

        let text = |key: &Key| &self.keys[key.offset..key.offset + key.length];
        keys.sort_by(|a, b| text(a).cmp(text(b)));

        for (written, key) in keys.iter().enumerate() {
            if written > 0 {
                self.out.push(b',');
            }
            self.append_newline_and_indent(depth);
            self.out.push(b'"');
            self.out
                .extend_from_slice(&self.keys[key.offset..key.offset + key.length]);
            self.out.extend_from_slice(b"\":");
            if self.indent.is_some() {
                self.out.push(b' ');
            }

            // The key is re-pushed, exchanged for its value by
            // `lua_gettable`, and the value popped again.
            match &key.raw {
                RawKey::Text(bytes) => stack.push_bytes(bytes),
                RawKey::Number(number) => stack.push_number(*number as f64),
            }
            stack.get_table(At::from_top(2));
            self.append_value(stack, depth)?;
            stack.pop(1);
        }

        if !keys.is_empty() {
            self.append_newline_and_indent(depth - 1);
        }
        self.out.push(b'}');
        self.keys.truncate(base);
        Ok(())
    }

    /// How long the array on top of the stack is, or `-1` if its keys make
    /// it an object.
    fn array_length(stack: &mut Stack) -> Result<i64, Raise> {
        let mut max: i64 = 0;
        let mut items: i64 = 0;
        stack.push_nil();
        while stack.next(At::from_top(2)) {
            let key = At::from_top(2);
            let key = (stack.kind(key) == Kind::Number).then(|| stack.to_number(key));
            // A key of 0 is not an index — and upstream tests the *number*
            // for truth, which is what makes 0 fall through here rather
            // than failing the `>= 1` below.
            match key {
                Some(key) if key != 0.0 && key.floor() == key && key >= 1.0 => {
                    max = max.max(crate::narrow::float_as_i64(key));
                    items += 1;
                    // The value goes, the key stays.
                    stack.pop(1);
                }
                _ => {
                    // The walk ends, so both go.
                    stack.pop(2);
                    return Ok(-1);
                }
            }
        }

        if max > items * SPARSE_RATIO && max > SPARSE_SAFE {
            // `encode_sparse_convert` is 0, so there is no "encode it as an
            // object instead" arm.
            return Err(refusal(stack, At::TOP, "excessively sparse array"));
        }
        Ok(max)
    }

    /// The table is on top, and `depth` already counts it.
    fn append_table(&mut self, stack: &mut Stack, depth: c_int) -> Result<(), Raise> {
        Self::check_depth(stack, depth)?;

        let mut as_array = false;
        let mut as_empty_dict = false;
        // Whether the array read may go through `__index`.
        let mut raw = true;

        // Each branch below pops exactly what it pushed, which the stack
        // comments track.
        let has_metatable = stack.get_metatable(At::TOP);
        if has_metatable {
            // .., table, mt, empty_dict_mt
            stack.get_field(At::REGISTRY, c"mpack.empty_dict");
            if stack.raw_equal(At::from_top(2), At::TOP) {
                as_empty_dict = true;
            } else {
                // .., table, mt, array_mt
                stack.pop(1);
                push_registry(stack, array_key());
                as_array = stack.raw_equal(At::TOP, At::from_top(2));
            }
            stack.pop(2);

            if !as_array {
                raw = false;
                // Upstream reads `__len`'s result here and then throws it
                // away — see `lua_objlen` below. Calling it is still what
                // decides the table is an array, and the call can raise, so
                // it has to happen; an error it raises is left on top for
                // the binding to raise.
                as_array = stack.get_metafield(At::TOP, c"__len");
                if as_array {
                    stack.push_value(At::from_top(2));
                    if stack.pcall(1, 1, None).is_err() {
                        return Err(Raise::top_value());
                    }
                    stack.pop(1);
                }
            }
        }

        if as_array {
            // Upstream writes `int len = lua_objlen(l, -1);` — a length
            // narrowed to an `int`, so the port narrows it the same way.
            let length = i64::from(len_as_int(stack.raw_len(At::TOP)));
            return self.append_array(stack, depth, length, raw);
        }

        let length = Self::array_length(stack)?;
        // `encode_empty_table_as_object` is 0, so an empty plain table is
        // `[]` — unless it carries `vim.empty_dict()`'s metatable.
        if length > 0 || (length == 0 && !as_empty_dict) {
            return self.append_array(stack, depth, length, raw);
        }

        if has_metatable {
            // .., table, mt, empty_array_mt
            stack.get_metatable(At::TOP);
            push_registry(stack, empty_array_key());
            let empty_array = stack.raw_equal(At::TOP, At::from_top(2));
            stack.pop(2);
            if empty_array {
                let length = i64::from(len_as_int(stack.raw_len(At::TOP)));
                return self.append_array(stack, depth, length, true);
            }
        }

        if self.sort_keys {
            self.append_object_sorted(stack, depth)
        } else {
            self.append_object(stack, depth)
        }
    }

    /// The value is on top; every branch leaves the stack as it found it.
    fn append_value(&mut self, stack: &mut Stack, depth: c_int) -> Result<(), Raise> {
        match stack.kind(At::TOP) {
            Kind::String => self.append_string(stack, At::TOP),
            Kind::Number => self.append_number(stack, At::TOP, Sink::Document)?,
            Kind::Boolean => {
                let text: &[u8] = if stack.to_boolean(At::TOP) {
                    b"true"
                } else {
                    b"false"
                };
                self.out.extend_from_slice(text);
            }
            Kind::Table => self.append_table(stack, depth + 1)?,
            Kind::Nil => self.out.extend_from_slice(b"null"),
            Kind::LightUserdata => {
                // Upstream's `empty_array` sentinel. nvim never publishes
                // it, so nothing can hand it back; and upstream emits
                // *nothing at all* for any other light userdatum, which is
                // kept for the same reason.
                if core::ptr::eq(stack.identity(At::TOP), array_key()) {
                    self.append_array(stack, depth, 0, true)?;
                }
            }
            Kind::Userdata => {
                stack.get_field(At::REGISTRY, c"mpack.NIL");
                let is_nil = stack.raw_equal(At::from_top(2), At::TOP);
                stack.pop(1);
                if !is_nil {
                    return Err(refusal(stack, At::TOP, "type not supported"));
                }
                self.out.extend_from_slice(b"null");
            }
            _ => return Err(refusal(stack, At::TOP, "type not supported")),
        }
        Ok(())
    }
}

/// Read the optional second argument, and leave the value to encode on top
/// of the stack.
fn read_options(stack: &mut Stack, encoder: &mut Encoder) -> Result<(), Raise> {
    match stack.top().index() {
        1 => return Ok(()),
        2 => {}
        _ => return Err(Raise::message("expected 1 or 2 arguments")),
    }
    stack.check_kind(At::arg(2), Kind::Table)?;

    // In upstream's order, because `lua_getfield` runs `__index` and a table
    // that has one can tell how many times, and in which order, it was
    // asked. Each read below pushes one value and pops it again.
    if let Some(set) = read_field(stack, c"escape_slash")? {
        encoder.escape_slash = set;
    }
    // `luaL_checkstring` also accepts a *number*, converting it in place:
    // `indent = 2` indents with the character `2`.
    stack.get_field(At::arg(2), c"indent");
    if stack.kind(At::TOP) != Kind::Nil {
        let text = stack.check_bytes(At::TOP)?;
        // The C string `luaL_checkstring` answers: up to the first NUL.
        let text = text.split(|&byte| byte == 0).next().unwrap_or_default();
        if !text.is_empty() {
            encoder.indent = Some(text.into());
        }
    }
    stack.pop(1);
    if let Some(set) = read_field(stack, c"sort_keys")? {
        encoder.sort_keys = set;
    }

    // Drop the options table too, so the value is on top.
    stack.set_top(At::arg(1));
    Ok(())
}

/// One of the two boolean options: `None` when it is absent, and a type
/// error when it is present and not a boolean.
fn read_field(stack: &mut Stack, field: &CStr) -> Result<Option<bool>, Raise> {
    stack.get_field(At::arg(2), field);
    let set = if stack.kind(At::TOP) == Kind::Nil {
        None
    } else {
        stack.check_kind(At::TOP, Kind::Boolean)?;
        Some(stack.to_boolean(At::TOP))
    };
    stack.pop(1);
    Ok(set)
}

/// `vim.json.encode(value[, options])`.
pub(super) fn encode(stack: &mut Stack) -> Result<c_int, Raise> {
    let buffer = fetch_config(stack)?.take_buffer();
    let mut encoder = Encoder {
        out: buffer,
        keys: Vec::new(),
        indent: None,
        escape_slash: false,
        sort_keys: false,
    };
    let walked = read_options(stack, &mut encoder).and_then(|()| encoder.append_value(stack, 0));
    if walked.is_ok() {
        // The document is copied into Lua's own string before the buffer
        // goes back to the config.
        stack.push_bytes(&encoder.out);
    }
    if let Ok(config) = fetch_config(stack) {
        config.restore_buffer(encoder.out);
    }
    walked.map(|()| 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three ranges and seven exceptions upstream spells as a 256-entry
    /// table of `const char *`.
    #[test]
    fn the_escape_table_is_upstreams() {
        assert_eq!(escape(0x08, false), Some(&b"\\b"[..]));
        assert_eq!(escape(0x09, false), Some(&b"\\t"[..]));
        assert_eq!(escape(0x0a, false), Some(&b"\\n"[..]));
        assert_eq!(escape(0x0c, false), Some(&b"\\f"[..]));
        assert_eq!(escape(0x0d, false), Some(&b"\\r"[..]));
        assert_eq!(escape(b'"', false), Some(&b"\\\""[..]));
        assert_eq!(escape(b'\\', false), Some(&b"\\\\"[..]));
        // The control bytes with no shorthand, and DEL.
        assert_eq!(escape(0x00, false), Some(&b"\\u0000"[..]));
        assert_eq!(escape(0x0b, false), Some(&b"\\u000b"[..]));
        assert_eq!(escape(0x1f, false), Some(&b"\\u001f"[..]));
        assert_eq!(escape(0x7f, false), Some(&b"\\u007f"[..]));
        // Everything else goes out as itself, high bytes included: the
        // encoder is byte-transparent above ASCII.
        for byte in [b' ', b'~', 0x80, 0xff] {
            assert_eq!(escape(byte, false), None, "{byte:#x}");
        }
    }

    /// `escape_slash` is the one row the option changes; upstream swaps in a
    /// whole second copy of the table for it.
    #[test]
    fn escape_slash_changes_exactly_one_row() {
        assert_eq!(escape(b'/', false), None);
        assert_eq!(escape(b'/', true), Some(&b"\\/"[..]));
        for byte in 0..=u8::MAX {
            if byte != b'/' {
                assert_eq!(escape(byte, false), escape(byte, true), "{byte:#x}");
            }
        }
    }

    /// The `\u00xx` table is built at compile time, indexed by the byte, and
    /// spells its hex in lower case — which is a wire contract.
    #[test]
    fn the_hex_escape_table_is_indexed_by_its_own_byte() {
        for byte in 0..=u8::MAX {
            let escape = HEX_ESCAPES[usize::from(byte)];
            assert_eq!(&escape[..4], b"\\u00");
            assert_eq!(
                core::str::from_utf8(&escape).expect("ASCII"),
                format!("\\u00{byte:02x}")
            );
        }
    }
}
