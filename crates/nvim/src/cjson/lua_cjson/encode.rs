#![deny(unsafe_op_in_unsafe_fn)]
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
//! already spent.
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

use core::ffi::{CStr, c_char, c_int};

use crate::narrow::len_as_int;

use super::{
    Config, ENCODE_MAX_DEPTH, NUMBER_PRECISION, SPARSE_RATIO, SPARSE_SAFE, array_key,
    empty_array_key, fetch_config, push_registry, unreachable_after_raise,
};
use crate::cjson::fpconv::append_g_fmt;
use crate::lua::executor::{nlua_get_empty_dict_ref, nlua_get_nil_ref, nlua_pushref};
use crate::lua::ffi::{
    LUA_TBOOLEAN, LUA_TLIGHTUSERDATA, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE,
    LUA_TUSERDATA, lua_call, lua_checkstack, lua_getfield, lua_getmetatable, lua_gettable,
    lua_gettop, lua_next, lua_objlen, lua_pop, lua_pushinteger, lua_pushlstring, lua_pushnil,
    lua_pushnumber, lua_pushvalue, lua_rawequal, lua_rawgeti, lua_settop, lua_toboolean,
    lua_tointeger, lua_tolstring, lua_tonumber, lua_touserdata, lua_type, lua_typename,
    luaL_checklstring, luaL_checktype, luaL_error, luaL_getmetafield,
};
use crate::types::{lua_State, size_t};

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
    /// The module table this call belongs to, so the reusable buffer can be
    /// handed back before raising.
    cfg: *mut Config,
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

impl Encoder {
    fn sink(&mut self, which: Sink) -> &mut Vec<u8> {
        match which {
            Sink::Document => &mut self.out,
            Sink::Keys => &mut self.keys,
        }
    }

    /// `json_encode_exception`: hand the reusable buffer back to the module
    /// table, then raise.
    ///
    /// The hand-back is why this is a method. `luaL_error` does not unwind
    /// this frame, so a buffer still owned here would be lost — which is
    /// exactly what happens to upstream's key array on the *other* error
    /// path, the one where a `__len` or `__index` metamethod raises from
    /// under us. Nothing can be done about that one from here.
    fn raise(&mut self, l: *mut lua_State, lindex: c_int, reason: &CStr) -> ! {
        // SAFETY: `cfg` is the live upvalue userdatum.
        unsafe { (*self.cfg).restore_buffer(core::mem::take(&mut self.out)) };
        drop(core::mem::take(&mut self.keys));
        // SAFETY: `l` is live, `lua_typename` takes the tag `lua_type` just
        // answered, and neither buffer is owned here any more.
        unsafe {
            let name = lua_typename(l, lua_type(l, lindex));
            luaL_error(
                l,
                c"Cannot serialise %s: %s".as_ptr(),
                name,
                reason.as_ptr(),
            );
        }
        unreachable_after_raise()
    }

    /// Append the string at `lindex` with its contents escaped, no quotes.
    ///
    /// # Safety
    /// `lindex` must hold a string.
    unsafe fn append_string_contents(&mut self, l: *mut lua_State, lindex: c_int, into: Sink) {
        let mut length: size_t = 0;
        // SAFETY: the caller has checked the type, so this reads the string
        // rather than converting the slot (which would break `lua_next`),
        // and answers `length` bytes that stay put while the value is on
        // the stack.
        let bytes = unsafe {
            let text = lua_tolstring(l, lindex, &raw mut length);
            core::slice::from_raw_parts(text.cast::<u8>(), length)
        };
        let escape_slash = self.escape_slash;
        let out = self.sink(into);
        // Upstream reserves `length * 6`, the all-escapes worst case, and
        // then appends without checking. A `Vec` grows geometrically, so
        // the exact-fit reservation is the useful one and the `SIZE_MAX / 6`
        // overflow guard that came with the worst case is not needed.
        out.reserve(length);
        for &byte in bytes {
            match escape(byte, escape_slash) {
                Some(text) => out.extend_from_slice(text),
                None => out.push(byte),
            }
        }
    }

    /// # Safety
    /// `lindex` must hold a string.
    unsafe fn append_string(&mut self, l: *mut lua_State, lindex: c_int) {
        self.out.push(b'"');
        unsafe { self.append_string_contents(l, lindex, Sink::Document) };
        self.out.push(b'"');
    }

    /// # Safety
    /// `lindex` must hold a number.
    unsafe fn append_number(&mut self, l: *mut lua_State, lindex: c_int, into: Sink) {
        // SAFETY: the caller has checked the type.
        let number = unsafe { lua_tonumber(l, lindex) };
        if !number.is_finite() {
            // `encode_invalid_numbers` is 0. Upstream's other two settings
            // would spell these "NaN"/"Infinity" or "null".
            self.raise(l, lindex, c"must not be NaN or Infinity");
        }
        append_g_fmt(self.sink(into), number, NUMBER_PRECISION);
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
    fn check_depth(&mut self, l: *mut lua_State, depth: c_int) {
        // SAFETY: `l` is live.
        if depth <= ENCODE_MAX_DEPTH && unsafe { lua_checkstack(l, 3) } != 0 {
            return;
        }
        // SAFETY: as `raise`, but the message is this one's own.
        unsafe { (*self.cfg).restore_buffer(core::mem::take(&mut self.out)) };
        drop(core::mem::take(&mut self.keys));
        // SAFETY: `l` is live and neither buffer is owned here any more.
        unsafe {
            luaL_error(
                l,
                c"Cannot serialise, excessive nesting (%d)".as_ptr(),
                depth,
            )
        };
        unreachable_after_raise()
    }

    /// # Safety
    /// The table must be on top of the stack.
    unsafe fn append_array(&mut self, l: *mut lua_State, depth: c_int, length: i64, raw: bool) {
        // Function-local: a module-level const lands in ffigen's flat C
        // namespace (`unit-cdefs.h`).
        const BOUNDED_BY_LENGTH: &str = "an array index is bounded by the table's length";
        self.out.push(b'[');
        for index in 1..=length {
            if index > 1 {
                self.out.push(b',');
            }
            self.append_newline_and_indent(depth);
            // SAFETY: the table is on top; `raw` says whether `__index` is
            // allowed to answer, which is how a table that became an array
            // through `__len` gets read. The element is popped here.
            unsafe {
                if raw {
                    // Upstream's loop counter is an `int`, which is what
                    // `lua_rawgeti` takes; `length` is a table's element
                    // count (`lua_objlen`'s, already narrowed to an `int`,
                    // or `array_length`'s, which raises on a sparse array),
                    // so neither narrowing here can fail.
                    lua_rawgeti(l, -1, c_int::try_from(index).expect(BOUNDED_BY_LENGTH));
                } else {
                    // Lua's own integer type is `ptrdiff_t`.
                    lua_pushinteger(l, isize::try_from(index).expect(BOUNDED_BY_LENGTH));
                    lua_gettable(l, -2);
                }
                self.append_value(l, depth);
                lua_pop(l, 1);
            }
        }
        if length > 0 {
            self.append_newline_and_indent(depth - 1);
        }
        self.out.push(b']');
    }

    /// # Safety
    /// The table must be on top of the stack.
    unsafe fn append_object(&mut self, l: *mut lua_State, depth: c_int) {
        self.out.push(b'{');
        let mut written = 0;
        // SAFETY: the table is on top, and the key/value pair `lua_next`
        // leaves is popped on every path out of the loop.
        unsafe { lua_pushnil(l) };
        // SAFETY: as above.
        while unsafe { lua_next(l, -2) } != 0 {
            if written > 0 {
                self.out.push(b',');
            }
            written += 1;
            self.append_newline_and_indent(depth);

            // .., table, key, value
            // SAFETY: as above; the key is at -2 for the whole arm.
            match unsafe { lua_type(l, -2) } {
                LUA_TNUMBER => {
                    self.out.push(b'"');
                    unsafe { self.append_number(l, -2, Sink::Document) };
                    self.out.extend_from_slice(b"\":");
                }
                LUA_TSTRING => {
                    unsafe { self.append_string(l, -2) };
                    self.out.push(b':');
                }
                _ => self.raise(l, -2, c"table key must be a number or string"),
            }
            if self.indent.is_some() {
                self.out.push(b' ');
            }
            // SAFETY: as above; the value is on top and popped here.
            unsafe {
                self.append_value(l, depth);
                lua_pop(l, 1);
            }
        }
        if written > 0 {
            self.append_newline_and_indent(depth - 1);
        }
        self.out.push(b'}');
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
    /// # Safety
    /// The table must be on top of the stack.
    unsafe fn append_object_sorted(&mut self, l: *mut lua_State, depth: c_int) {
        // Nested sorted objects share the scratch, so each level takes back
        // only what it added.
        self.out.push(b'{');
        let base = self.keys.len();
        let mut keys: Vec<Key> = Vec::new();

        // SAFETY: the table is on top for both loops; every pushed value is
        // popped before the next iteration.
        unsafe { lua_pushnil(l) };
        // SAFETY: as above.
        while unsafe { lua_next(l, -2) } != 0 {
            let offset = self.keys.len();
            // SAFETY: as above; the key is at -2 for the whole arm, and a
            // Lua string stays put while the table holding it is on the
            // stack, which is long enough to copy it.
            let raw = match unsafe { lua_type(l, -2) } {
                LUA_TSTRING => unsafe {
                    self.append_string_contents(l, -2, Sink::Keys);
                    let mut length: size_t = 0;
                    let text = lua_tolstring(l, -2, &raw mut length);
                    RawKey::Text(core::slice::from_raw_parts(text.cast::<u8>(), length).into())
                },
                LUA_TNUMBER => unsafe {
                    self.append_number(l, -2, Sink::Keys);
                    RawKey::Number(lua_tointeger(l, -2))
                },
                _ => self.raise(l, -2, c"table key must be number or string"),
            };
            keys.push(Key {
                offset,
                length: self.keys.len() - offset,
                raw,
            });
            // SAFETY: as above; this drops the value `lua_next` left.
            unsafe { lua_pop(l, 1) };
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

            // SAFETY: as above; the key is re-pushed, exchanged for its
            // value by `lua_gettable`, and the value popped again.
            unsafe {
                match &key.raw {
                    RawKey::Text(bytes) => {
                        lua_pushlstring(l, bytes.as_ptr().cast::<c_char>(), bytes.len());
                    }
                    RawKey::Number(number) => lua_pushnumber(l, *number as f64),
                }
                lua_gettable(l, -2);
                self.append_value(l, depth);
                lua_pop(l, 1);
            }
        }

        if !keys.is_empty() {
            self.append_newline_and_indent(depth - 1);
        }
        self.out.push(b'}');
        self.keys.truncate(base);
    }

    /// How long the array on top of the stack is, or `-1` if its keys make
    /// it an object.
    ///
    /// # Safety
    /// The table must be on top of the stack.
    unsafe fn array_length(&mut self, l: *mut lua_State) -> i64 {
        let mut max: i64 = 0;
        let mut items: i64 = 0;
        // SAFETY: the table is on top, and both exits pop what `lua_next`
        // left.
        unsafe { lua_pushnil(l) };
        // SAFETY: as above.
        while unsafe { lua_next(l, -2) } != 0 {
            // SAFETY: as above; the key is at -2.
            let key = unsafe { (lua_type(l, -2) == LUA_TNUMBER).then(|| lua_tonumber(l, -2)) };
            // A key of 0 is not an index — and upstream tests the
            // *number* for truth, which is what makes 0 fall through
            // here rather than failing the `>= 1` below.
            match key {
                Some(key) if key != 0.0 && key.floor() == key && key >= 1.0 => {
                    max = max.max(crate::narrow::float_as_i64(key));
                    items += 1;
                    // SAFETY: as above; the value goes, the key stays.
                    unsafe { lua_pop(l, 1) };
                }
                _ => {
                    // SAFETY: as above; the walk ends, so both go.
                    unsafe { lua_pop(l, 2) };
                    return -1;
                }
            }
        }

        if max > items * SPARSE_RATIO && max > SPARSE_SAFE {
            // `encode_sparse_convert` is 0, so there is no "encode it as an
            // object instead" arm.
            self.raise(l, -1, c"excessively sparse array");
        }
        max
    }

    /// # Safety
    /// The table must be on top of the stack, and `depth` must already
    /// count it.
    unsafe fn append_table(&mut self, l: *mut lua_State, depth: c_int) {
        self.check_depth(l, depth);

        let mut as_array = false;
        let mut as_empty_dict = false;
        // Whether the array read may go through `__index`.
        let mut raw = true;

        // SAFETY: the table is on top; each branch below pops exactly what
        // it pushed, which the stack comments track. `lua_call` on `__len`
        // may raise, which is upstream's behaviour too.
        let has_metatable = unsafe { lua_getmetatable(l, -1) } != 0;
        if has_metatable {
            // SAFETY: as above, with the metatable at -1.
            unsafe {
                // .., table, mt, empty_dict_mt
                nlua_pushref(l, nlua_get_empty_dict_ref(l));
                if lua_rawequal(l, -2, -1) != 0 {
                    as_empty_dict = true;
                } else {
                    // .., table, mt, array_mt
                    lua_pop(l, 1);
                    push_registry(l, array_key());
                    as_array = lua_rawequal(l, -1, -2) != 0;
                }
                lua_pop(l, 2);
            }

            if !as_array {
                raw = false;
                // Upstream reads `__len`'s result here and then throws it
                // away — see `lua_objlen` below. Calling it is still what
                // decides the table is an array, and the call can raise, so
                // it has to happen.
                // SAFETY: as above; the metafield and its result are popped.
                as_array = unsafe {
                    let has_len = luaL_getmetafield(l, -1, c"__len".as_ptr()) != 0;
                    if has_len {
                        lua_pushvalue(l, -2);
                        lua_call(l, 1, 1);
                        lua_pop(l, 1);
                    }
                    has_len
                };
            }
        }

        if as_array {
            // SAFETY: the table is on top.
            // Upstream writes `int len = lua_objlen(l, -1);` — a length
            // narrowed to an `int`, so the port narrows it the same way.
            let length = i64::from(len_as_int(unsafe { lua_objlen(l, -1) }));
            unsafe { self.append_array(l, depth, length, raw) };
            return;
        }

        // SAFETY: as above.
        let length = unsafe { self.array_length(l) };
        // `encode_empty_table_as_object` is 0, so an empty plain table
        // is `[]` — unless it carries `vim.empty_dict()`'s metatable.
        if length > 0 || (length == 0 && !as_empty_dict) {
            // SAFETY: as above.
            unsafe { self.append_array(l, depth, length, raw) };
            return;
        }

        if has_metatable {
            // .., table, mt, empty_array_mt
            // SAFETY: as above; both pushes are popped here.
            let empty_array = unsafe {
                lua_getmetatable(l, -1);
                push_registry(l, empty_array_key());
                let equal = lua_rawequal(l, -1, -2) != 0;
                lua_pop(l, 2);
                equal
            };
            if empty_array {
                // SAFETY: as above.
                unsafe {
                    // As above.
                    let length = i64::from(len_as_int(lua_objlen(l, -1)));
                    self.append_array(l, depth, length, true);
                }
                return;
            }
        }

        // SAFETY: as above.
        unsafe {
            if self.sort_keys {
                self.append_object_sorted(l, depth);
            } else {
                self.append_object(l, depth);
            }
        }
    }

    /// # Safety
    /// The value must be on top of the stack.
    unsafe fn append_value(&mut self, l: *mut lua_State, depth: c_int) {
        // SAFETY: the value is on top and every branch leaves the stack as
        // it found it.
        match unsafe { lua_type(l, -1) } {
            LUA_TSTRING => unsafe { self.append_string(l, -1) },
            LUA_TNUMBER => unsafe { self.append_number(l, -1, Sink::Document) },
            LUA_TBOOLEAN => {
                let text: &[u8] = if unsafe { lua_toboolean(l, -1) } != 0 {
                    b"true"
                } else {
                    b"false"
                };
                self.out.extend_from_slice(text);
            }
            LUA_TTABLE => unsafe { self.append_table(l, depth + 1) },
            LUA_TNIL => self.out.extend_from_slice(b"null"),
            LUA_TLIGHTUSERDATA => {
                // Upstream's `empty_array` sentinel. nvim never publishes
                // it, so nothing can hand it back; and upstream emits
                // *nothing at all* for any other light userdatum, which
                // is kept for the same reason.
                if core::ptr::eq(unsafe { lua_touserdata(l, -1) }, array_key()) {
                    unsafe { self.append_array(l, depth, 0, true) };
                }
            }
            LUA_TUSERDATA => {
                // SAFETY: as above; `nlua_get_nil_ref`'s push is popped here.
                let is_nil = unsafe {
                    nlua_pushref(l, nlua_get_nil_ref(l));
                    let is_nil = lua_rawequal(l, -2, -1) != 0;
                    lua_pop(l, 1);
                    is_nil
                };
                if is_nil {
                    self.out.extend_from_slice(b"null");
                } else {
                    self.raise(l, -1, c"type not supported");
                }
            }
            _ => self.raise(l, -1, c"type not supported"),
        }
    }
}

/// Read the optional second argument, and leave the value to encode on top
/// of the stack.
///
/// # Safety
/// `l` must be a live Lua state.
unsafe fn read_options(l: *mut lua_State, encoder: &mut Encoder) {
    // SAFETY: `l` is live.
    match unsafe { lua_gettop(l) } {
        1 => return,
        2 => {}
        _ => {
            // SAFETY: as above.
            unsafe { luaL_error(l, c"expected 1 or 2 arguments".as_ptr()) };
            unreachable_after_raise()
        }
    }
    // SAFETY: as above; argument 2 is a table from here on.
    unsafe { luaL_checktype(l, 2, LUA_TTABLE) };

    // In upstream's order, because `lua_getfield` runs `__index` and a table
    // that has one can tell how many times, and in which order, it was
    // asked. Each read below pushes one value and pops it again.
    // SAFETY: as above.
    if let Some(set) = unsafe { read_field(l, c"escape_slash") } {
        encoder.escape_slash = set;
    }
    // SAFETY: as above. `luaL_checkstring` also accepts a *number*,
    // converting it in place: `indent = 2` indents with the character `2`.
    unsafe {
        lua_getfield(l, 2, c"indent".as_ptr());
        if lua_type(l, -1) != LUA_TNIL {
            let text = CStr::from_ptr(luaL_checklstring(l, -1, core::ptr::null_mut())).to_bytes();
            if !text.is_empty() {
                encoder.indent = Some(text.into());
            }
        }
        lua_pop(l, 1);
    }
    // SAFETY: as above.
    if let Some(set) = unsafe { read_field(l, c"sort_keys") } {
        encoder.sort_keys = set;
    }

    // Drop the options table too, so the value is on top.
    // SAFETY: as above.
    unsafe { lua_settop(l, 1) };
}

/// One of the two boolean options: `None` when it is absent, and a type
/// error when it is present and not a boolean.
///
/// # Safety
/// `l` must be a live Lua state with the options table at index 2.
unsafe fn read_field(l: *mut lua_State, field: &CStr) -> Option<bool> {
    // SAFETY: the caller's state and table; the push is popped here.
    unsafe {
        lua_getfield(l, 2, field.as_ptr());
        let set = (lua_type(l, -1) != LUA_TNIL).then(|| {
            luaL_checktype(l, -1, LUA_TBOOLEAN);
            lua_toboolean(l, -1) != 0
        });
        lua_pop(l, 1);
        set
    }
}

/// `vim.json.encode(value[, options])`.
///
/// # Safety
/// `l` must be a live Lua state with this module's config as upvalue 1.
pub unsafe extern "C-unwind" fn encode(l: *mut lua_State) -> c_int {
    // SAFETY: `l` is live and was entered through a closure `set_functions`
    // built, so upvalue 1 is the config userdatum.
    let cfg = unsafe { fetch_config(l) };
    if cfg.is_null() {
        // SAFETY: as above.
        unsafe { luaL_error(l, c"BUG: Unable to fetch CJSON configuration".as_ptr()) };
        unreachable_after_raise()
    }

    let mut encoder = Encoder {
        cfg,
        // SAFETY: `cfg` points at a live, initialised `Config`.
        out: unsafe { (*cfg).take_buffer() },
        keys: Vec::new(),
        indent: None,
        escape_slash: false,
        sort_keys: false,
    };
    // SAFETY: `l` is live throughout; `read_options` leaves the value on
    // top, which is what `append_value` requires. The document is copied
    // into Lua's own string before the buffer goes back to the config.
    unsafe {
        read_options(l, &mut encoder);
        encoder.append_value(l, 0);
        lua_pushlstring(l, encoder.out.as_ptr().cast::<c_char>(), encoder.out.len());
        (*cfg).restore_buffer(core::mem::take(&mut encoder.out));
    }
    1
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
