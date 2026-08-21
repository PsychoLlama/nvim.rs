#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! `vim.json.decode`: JSON text in, a Lua value back.
//!
//! A hand-written recursive-descent parser over the document *as a slice*.
//! Upstream walks it as a `const char *` and leans on the NUL Lua puts past
//! every string, so the slice here is `len + 1` bytes long and that NUL is
//! part of it: it is the `T_END` token, and it is what stops the string and
//! number scanners.
//!
//! Two upstream settings are hard here. `decode_invalid_numbers` is 1,
//! which means [`is_invalid_number`] is consulted **only** to decide
//! whether a token starting with `i`/`n`/`+` is worth handing to
//! [`strtod`]; a token starting with a digit or `-` goes straight there, so
//! the whole C `strtod` grammar — hex floats included — is reachable
//! through `vim.json.decode`. `decode_array_with_array_mt` is 0, so a
//! decoded array carries no metatable.
//!
//! Ported from Lua CJSON, Copyright (c) 2010-2012 Mark Pulford, under the
//! MIT license; the notice is reproduced in licenses/lua-cjson-LICENSE.txt.

use core::ffi::{CStr, c_char, c_int};

use super::{Config, DECODE_MAX_DEPTH, fetch_config, unreachable_after_raise};
use crate::cjson::fpconv::strtod;
use crate::lua::executor::{nlua_get_empty_dict_ref, nlua_get_nil_ref, nlua_pushref};
use crate::lua::ffi::{
    LUA_TNIL, LUA_TTABLE, lua_checkstack, lua_getfield, lua_gettop, lua_newtable, lua_pop,
    lua_pushboolean, lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_rawset,
    lua_rawseti, lua_setmetatable, lua_toboolean, lua_type, luaL_checklstring, luaL_checktype,
    luaL_error,
};
use crate::narrow::number_as_int;
use crate::types::{lua_Integer, lua_State, size_t};

/// A byte offset as an error message reports it. Upstream hands the
/// `ptrdiff_t` difference of two pointers to a `%d`, i.e. narrows it by
/// wrapping, which is what [`number_as_int`] is for.
fn offset_as_int(at: usize) -> c_int {
    number_as_int(i64::try_from(at).unwrap_or(i64::MAX))
}

/// What a byte can start, before the scanners look any closer. Upstream's
/// `ch2token`, a 256-entry table built per config.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    ObjBegin,
    ObjEnd,
    ArrBegin,
    ArrEnd,
    Comma,
    Colon,
    End,
    Whitespace,
    /// `"`, a digit, `+`, `-`, or the first letter of `true`/`false`/
    /// `null`/`inf`/`nan`: needs a scanner.
    Unknown,
    Error,
}

fn classify(byte: u8) -> Class {
    match byte {
        b'{' => Class::ObjBegin,
        b'}' => Class::ObjEnd,
        b'[' => Class::ArrBegin,
        b']' => Class::ArrEnd,
        b',' => Class::Comma,
        b':' => Class::Colon,
        0 => Class::End,
        b' ' | b'\t' | b'\n' | b'\r' => Class::Whitespace,
        b'"' | b'+' | b'-' | b'0'..=b'9' => Class::Unknown,
        // `false`, `inf`/`infinity`, `null`/`nan`, `true`.
        b'f' | b'i' | b'I' | b'n' | b'N' | b't' => Class::Unknown,
        _ => Class::Error,
    }
}

/// The byte a `\x` escape stands for, or 0 for "not an escape". `u` stands
/// for itself, because the caller has more work to do for that one.
fn unescape(byte: u8) -> u8 {
    match byte {
        b'"' => b'"',
        b'\\' => b'\\',
        b'/' => b'/',
        b'b' => 0x08,
        b't' => 0x09,
        b'n' => 0x0a,
        b'f' => 0x0c,
        b'r' => 0x0d,
        b'u' => b'u',
        _ => 0,
    }
}

/// A token, minus its text: a string's bytes are left in
/// [`Parser::scratch`], because decoding escapes needs somewhere to put
/// them and that buffer is sized once for the whole document.
enum Token {
    ObjBegin,
    ObjEnd,
    ArrBegin,
    ArrEnd,
    /// The bytes are `Parser::scratch`.
    Str,
    Number(f64),
    Integer(lua_Integer),
    Boolean(bool),
    Null,
    Colon,
    Comma,
    End,
    Error(&'static CStr),
}

impl Token {
    /// What a parse error calls this token when it was not the one wanted.
    /// Upstream's `json_token_type_name`, whose spellings leak into the
    /// error message and are therefore a contract.
    fn name(&self) -> &'static CStr {
        match self {
            Token::ObjBegin => c"T_OBJ_BEGIN",
            Token::ObjEnd => c"T_OBJ_END",
            Token::ArrBegin => c"T_ARR_BEGIN",
            Token::ArrEnd => c"T_ARR_END",
            Token::Str => c"T_STRING",
            Token::Number(_) => c"T_NUMBER",
            Token::Integer(_) => c"T_INTEGER",
            Token::Boolean(_) => c"T_BOOLEAN",
            Token::Null => c"T_NULL",
            Token::Colon => c"T_COLON",
            Token::Comma => c"T_COMMA",
            Token::End => c"T_END",
            Token::Error(reason) => reason,
        }
    }
}

struct Parser<'a> {
    /// The document plus the NUL Lua keeps past it.
    data: &'a [u8],
    at: usize,
    /// Where the token being reported started, for the error message.
    index: usize,
    /// Decoded string bytes, sized once from the document length: a decoded
    /// string is never longer than the text it came from.
    scratch: Vec<u8>,
    depth: c_int,
    luanil_object: bool,
    luanil_array: bool,
    skip_comments: bool,
}

impl<'a> Parser<'a> {
    fn byte(&self, at: usize) -> u8 {
        self.data.get(at).copied().unwrap_or(0)
    }

    /// Upstream's `json_set_token_error`: the position moves to wherever
    /// the scanner gave up, which is not where the token started.
    fn fail(&mut self, reason: &'static CStr) -> Token {
        self.index = self.at;
        Token::Error(reason)
    }

    /// `json_throw_parse_error`. Drops the scratch buffer first, because
    /// `luaL_error` does not unwind this frame — which is exactly why
    /// upstream's comment there says not to call it with dynamic memory
    /// allocated.
    fn throw(&mut self, l: *mut lua_State, expected: &CStr, token: &Token) -> ! {
        let found = token.name();
        let at = offset_as_int(self.index) + 1;
        drop(core::mem::take(&mut self.scratch));
        // SAFETY: `l` is live and both strings outlive the call.
        unsafe {
            luaL_error(
                l,
                c"Expected %s but found %s at character %d".as_ptr(),
                expected.as_ptr(),
                found.as_ptr(),
                at,
            );
        }
        unreachable_after_raise()
    }

    /// `\uXXXX`, possibly a surrogate pair, appended to the scratch as
    /// UTF-8. `self.at` points at the backslash.
    fn append_unicode_escape(&mut self) -> bool {
        let Some(mut codepoint) = self.hex4(self.at + 2) else {
            return false;
        };
        let mut length = 6;
        if codepoint & 0xF800 == 0xD800 {
            // The first of a pair must be the high half, and the second
            // must be a `\u` escape holding the low one.
            if codepoint & 0x400 != 0
                || self.byte(self.at + length) != b'\\'
                || self.byte(self.at + length + 1) != b'u'
            {
                return false;
            }
            let Some(low) = self.hex4(self.at + 2 + length) else {
                return false;
            };
            if low & 0xFC00 != 0xDC00 {
                return false;
            }
            codepoint = (((codepoint & 0x3FF) << 10) | (low & 0x3FF)) + 0x10000;
            length = 12;
        }

        // Upstream's own encoder, not `char::encode_utf8`: it accepts the
        // whole 21-bit range, including the surrogates a lone `\ud800` maps
        // to, where Rust's `char` does not.
        // Every `sextet`/`byte` below is masked, or bounded by its match
        // arm, to something a byte holds -- a surrogate pair tops out at 21
        // bits, so the widest lead byte is `codepoint >> 18`, three bits.
        let byte = |bits: u32| u8::try_from(bits).expect("masked to a byte");
        let sextet = |shift: u32| byte((codepoint >> shift) & 0x3F) | 0x80;
        let mut utf8 = [0u8; 4];
        let width = match codepoint {
            0x0000..=0x007F => {
                utf8[0] = byte(codepoint);
                1
            }
            0x0080..=0x07FF => {
                utf8[0] = byte(codepoint >> 6) | 0xC0;
                utf8[1] = sextet(0);
                2
            }
            0x0800..=0xFFFF => {
                utf8[0] = byte(codepoint >> 12) | 0xE0;
                utf8[1] = sextet(6);
                utf8[2] = sextet(0);
                3
            }
            _ => {
                utf8[0] = byte(codepoint >> 18) | 0xF0;
                utf8[1] = sextet(12);
                utf8[2] = sextet(6);
                utf8[3] = sextet(0);
                4
            }
        };
        self.scratch.extend_from_slice(&utf8[..width]);
        self.at += length;
        true
    }

    /// Four hex digits as one number, or `None` if any of them is not one —
    /// which includes the NUL past the end of the document, so this never
    /// reads past it.
    fn hex4(&self, at: usize) -> Option<u32> {
        let mut value = 0;
        for offset in 0..4 {
            let digit = (self.byte(at + offset) as char).to_digit(16)?;
            value = (value << 4) | digit;
        }
        Some(value)
    }

    /// `self.at` points at the opening quote.
    fn string_token(&mut self) -> Token {
        self.at += 1;
        self.scratch.clear();
        loop {
            let mut byte = self.byte(self.at);
            if byte == b'"' {
                break;
            }
            if byte == 0 {
                return self.fail(c"unexpected end of string");
            }
            if byte == b'\\' {
                byte = unescape(self.byte(self.at + 1));
                if byte == b'u' {
                    if self.append_unicode_escape() {
                        continue;
                    }
                    return self.fail(c"invalid unicode escape code");
                }
                if byte == 0 {
                    return self.fail(c"invalid escape code");
                }
                // Skip the backslash; the loop's own step takes the letter.
                self.at += 1;
            }
            self.scratch.push(byte);
            self.at += 1;
        }
        self.at += 1;
        Token::Str
    }

    /// Whether the number at `self.at` is one strict JSON would reject:
    /// a leading `+`, a leading zero, hex, `inf` or `nan`.
    fn is_invalid_number(&self) -> bool {
        let mut at = self.at;
        if self.byte(at) == b'+' {
            return true;
        }
        if self.byte(at) == b'-' {
            at += 1;
        }
        if self.byte(at) == b'0' {
            let next = self.byte(at + 1);
            return next | 0x20 == b'x' || next.is_ascii_digit();
        }
        if self.byte(at) <= b'9' {
            return false;
        }
        let rest = self.data.get(at..).unwrap_or(&[]);
        rest.len() >= 3
            && (rest[..3].eq_ignore_ascii_case(b"inf") || rest[..3].eq_ignore_ascii_case(b"nan"))
    }

    /// `self.at` points at the first byte of the number.
    fn number_token(&mut self) -> Token {
        let rest = &self.data[self.at..];
        let (integer, digits) = scan_integer(rest);
        let after = rest.get(digits).copied().unwrap_or(0);

        // A float when the integer scan stopped on something only a float
        // can continue with — or did not start at all, which is how `inf`,
        // `nan` and `0x…` get to `strtod`.
        if digits == 0 || matches!(after, b'.' | b'e' | b'E' | b'x') {
            let (number, used) = strtod(rest);
            if used == 0 {
                return self.fail(c"invalid number");
            }
            self.at += used;
            return Token::Number(number);
        }
        self.at += digits;
        // Upstream also has a `tmpval > PTRDIFF_MAX` arm that re-reads the
        // value as a double. It is dead on every platform nvim builds for:
        // `strtoll` saturates at `LLONG_MAX`, `lua_Integer` is `ptrdiff_t`,
        // and the two are the same width — so `18446744073709551616`
        // decodes to `9223372036854775807`, saturated, not to `1.8e19`.
        Token::Integer(integer)
    }

    fn next_token(&mut self) -> Token {
        let mut class;
        loop {
            loop {
                class = classify(self.byte(self.at));
                if class != Class::Whitespace {
                    break;
                }
                self.at += 1;
            }
            if !self.skip_comments
                || self.byte(self.at) != b'/'
                || !matches!(self.byte(self.at + 1), b'/' | b'*')
            {
                break;
            }
            if self.byte(self.at + 1) == b'/' {
                self.at += 2;
                while !matches!(self.byte(self.at), 0 | b'\n') {
                    self.at += 1;
                }
            } else {
                self.at += 2;
                loop {
                    if self.byte(self.at) == 0 {
                        return self.fail(c"unclosed multi-line comment");
                    }
                    if self.byte(self.at) == b'*' && self.byte(self.at + 1) == b'/' {
                        self.at += 2;
                        break;
                    }
                    self.at += 1;
                }
            }
        }

        self.index = self.at;
        let single = match class {
            Class::Error => return self.fail(c"invalid token"),
            Class::End => return Token::End,
            Class::ObjBegin => Token::ObjBegin,
            Class::ObjEnd => Token::ObjEnd,
            Class::ArrBegin => Token::ArrBegin,
            Class::ArrEnd => Token::ArrEnd,
            Class::Comma => Token::Comma,
            Class::Colon => Token::Colon,
            Class::Unknown => {
                let byte = self.byte(self.at);
                let rest = &self.data[self.at..];
                if byte == b'"' {
                    return self.string_token();
                }
                if byte == b'-' || byte.is_ascii_digit() {
                    // `decode_invalid_numbers` is 1, so upstream's
                    // "reject it before `strtod` sees it" arm is skipped
                    // and this is where hex, `inf` and `nan` get in.
                    return self.number_token();
                }
                for (word, token) in [
                    (&b"true"[..], Token::Boolean(true)),
                    (b"false", Token::Boolean(false)),
                    (b"null", Token::Null),
                ] {
                    if rest.starts_with(word) {
                        self.at += word.len();
                        return token;
                    }
                }
                if self.is_invalid_number() {
                    // Only the shapes known to be invalid JSON go on to
                    // `strtod`, so that everything else reports "invalid
                    // token" rather than "invalid number".
                    return self.number_token();
                }
                return self.fail(c"invalid token");
            }
            Class::Whitespace => unreachable!("the whitespace loop above ended"),
        };
        self.at += 1;
        single
    }

    /// One more level of nesting, plus the `slots` stack slots the level
    /// needs.
    fn descend(&mut self, l: *mut lua_State, slots: c_int) {
        self.depth += 1;
        // SAFETY: `l` is live.
        if self.depth <= DECODE_MAX_DEPTH && unsafe { lua_checkstack(l, slots) } != 0 {
            return;
        }
        let (depth, at) = (self.depth, offset_as_int(self.at));
        drop(core::mem::take(&mut self.scratch));
        // SAFETY: `l` is live, and the scratch is gone.
        unsafe {
            luaL_error(
                l,
                c"Found too many nested data structures (%d) at character %d".as_ptr(),
                depth,
                at,
            );
        }
        unreachable_after_raise()
    }

    /// Push the decoded string in the scratch.
    ///
    /// # Safety
    /// `l` must be a live Lua state with a free slot.
    unsafe fn push_scratch(&self, l: *mut lua_State) {
        let (bytes, length) = (self.scratch.as_ptr().cast::<c_char>(), self.scratch.len());
        // SAFETY: the caller's state; the scratch outlives the copy Lua makes.
        unsafe { lua_pushlstring(l, bytes, length) };
    }

    /// # Safety
    /// `l` must be a live Lua state; leaves one value on the stack.
    unsafe fn parse_object(&mut self, l: *mut lua_State) {
        // .., table, key, value
        self.descend(l, 3);
        // SAFETY: `l` is live and `descend` reserved the three slots.
        unsafe { lua_newtable(l) };
        let mut token = self.next_token();
        if matches!(token, Token::ObjEnd) {
            // An object that decodes to `{}` has to stay one on the way
            // back out, which is what `vim.empty_dict()`'s metatable is.
            // SAFETY: as above; the table is on top.
            unsafe {
                nlua_pushref(l, nlua_get_empty_dict_ref(l));
                lua_setmetatable(l, -2);
            }
            self.depth -= 1;
            return;
        }
        loop {
            if !matches!(token, Token::Str) {
                self.throw(l, c"object key string", &token);
            }
            // SAFETY: as above; the key's bytes are in the scratch.
            unsafe { self.push_scratch(l) };

            token = self.next_token();
            if !matches!(token, Token::Colon) {
                self.throw(l, c"colon", &token);
            }

            token = self.next_token();
            let luanil = self.luanil_object;
            // SAFETY: as above; `push_value` then `rawset` leaves the table.
            unsafe {
                self.push_value(l, &token, luanil);
                lua_rawset(l, -3);
            }

            token = self.next_token();
            if matches!(token, Token::ObjEnd) {
                self.depth -= 1;
                return;
            }
            if !matches!(token, Token::Comma) {
                self.throw(l, c"comma or object end", &token);
            }
            token = self.next_token();
        }
    }

    /// # Safety
    /// `l` must be a live Lua state; leaves one value on the stack.
    unsafe fn parse_array(&mut self, l: *mut lua_State) {
        // .., table, value
        self.descend(l, 2);
        // `decode_array_with_array_mt` is 0, so no metatable here.
        // SAFETY: `l` is live and `descend` reserved the two slots.
        unsafe { lua_newtable(l) };
        let mut token = self.next_token();
        if matches!(token, Token::ArrEnd) {
            self.depth -= 1;
            return;
        }
        let luanil = self.luanil_array;
        for index in 1.. {
            // SAFETY: as above; `push_value` then `rawseti` leaves the table.
            unsafe {
                self.push_value(l, &token, luanil);
                lua_rawseti(l, -2, index);
            }

            token = self.next_token();
            if matches!(token, Token::ArrEnd) {
                self.depth -= 1;
                return;
            }
            if !matches!(token, Token::Comma) {
                self.throw(l, c"comma or array end", &token);
            }
            token = self.next_token();
        }
    }

    /// # Safety
    /// `l` must be a live Lua state; leaves one value on the stack.
    unsafe fn push_value(&mut self, l: *mut lua_State, token: &Token, use_luanil: bool) {
        // SAFETY: `l` is live and has the slot `descend` reserved; every
        // arm below pushes exactly one value.
        match *token {
            Token::Str => unsafe { self.push_scratch(l) },
            Token::Number(number) => unsafe { lua_pushnumber(l, number) },
            Token::Integer(number) => unsafe { lua_pushinteger(l, number) },
            Token::Boolean(value) => unsafe { lua_pushboolean(l, c_int::from(value)) },
            Token::ObjBegin => unsafe { self.parse_object(l) },
            Token::ArrBegin => unsafe { self.parse_array(l) },
            Token::Null if use_luanil => unsafe { lua_pushnil(l) },
            Token::Null => unsafe { nlua_pushref(l, nlua_get_nil_ref(l)) },
            _ => self.throw(l, c"value", token),
        }
    }
}

/// The leading run of decimal digits, read the way `strtoll` reads it:
/// an optional sign, then digits, **saturating** at the ends of the range
/// rather than wrapping. Answers the value and how many bytes it took.
fn scan_integer(text: &[u8]) -> (lua_Integer, usize) {
    let negative = text.first() == Some(&b'-');
    let signed = usize::from(negative || text.first() == Some(&b'+'));
    let digits = text[signed..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits == 0 {
        return (0, 0);
    }
    let mut value: i64 = 0;
    let mut saturated = false;
    for &byte in &text[signed..signed + digits] {
        let digit = i64::from(byte - b'0');
        // Accumulate negatively when negative, so `-9223372036854775808`
        // lands exactly rather than overflowing on the way to its own sign.
        let step = value.checked_mul(10).and_then(|scaled| {
            if negative {
                scaled.checked_sub(digit)
            } else {
                scaled.checked_add(digit)
            }
        });
        match step {
            Some(next) => value = next,
            None => saturated = true,
        }
    }
    if saturated {
        value = if negative { i64::MIN } else { i64::MAX };
    }
    // `lua_Integer` is `ptrdiff_t`, the same width as the `i64` accumulated
    // above on every platform nvim builds for.
    let value = lua_Integer::try_from(value).expect("lua_Integer is 64 bits wide");
    (value, signed + digits)
}

/// The three flags the optional second argument can set.
#[derive(Clone, Copy, Default)]
struct Options {
    luanil_object: bool,
    luanil_array: bool,
    skip_comments: bool,
}

/// The boolean at `field` of the table at `lindex`, leaving the stack as it
/// was found.
///
/// # Safety
/// `l` must be a live Lua state with a table at `lindex` and a free slot.
unsafe fn boolean_field(l: *mut lua_State, lindex: c_int, field: &CStr) -> bool {
    // SAFETY: the caller's state and table; the push is popped here.
    unsafe {
        lua_getfield(l, lindex, field.as_ptr());
        let value = lua_toboolean(l, -1) != 0;
        lua_pop(l, 1);
        value
    }
}

/// Read the optional second argument.
///
/// # Safety
/// `l` must be a live Lua state.
unsafe fn read_options(l: *mut lua_State) -> Options {
    let mut options = Options::default();
    // SAFETY: `l` is live.
    match unsafe { lua_gettop(l) } {
        1 => return options,
        2 => {}
        _ => {
            // SAFETY: as above.
            unsafe { luaL_error(l, c"expected 1 or 2 arguments".as_ptr()) };
            unreachable_after_raise()
        }
    }
    // SAFETY: as above; argument 2 is a table from here on.
    unsafe { luaL_checktype(l, 2, LUA_TTABLE) };
    // SAFETY: as above.
    options.skip_comments = unsafe { boolean_field(l, 2, c"skip_comments") };

    // `luanil` is the only option that is itself a table, and an absent one
    // ends the read: the two flags inside stay false.
    // SAFETY: as above; the pushed value is popped on both paths out.
    unsafe {
        lua_getfield(l, 2, c"luanil".as_ptr());
        if lua_type(l, -1) == LUA_TNIL {
            lua_pop(l, 1);
            return options;
        }
        luaL_checktype(l, -1, LUA_TTABLE);
    }
    // SAFETY: as above, with the `luanil` table on top.
    unsafe {
        options.luanil_object = boolean_field(l, -1, c"object");
        options.luanil_array = boolean_field(l, -1, c"array");
        // The `luanil` table goes too.
        lua_pop(l, 1);
    }
    options
}

/// `vim.json.decode(text[, options])`.
///
/// # Safety
/// `l` must be a live Lua state with this module's config as upvalue 1.
pub unsafe extern "C-unwind" fn decode(l: *mut lua_State) -> c_int {
    // The config carries nothing the decoder reads any more — every
    // `decode_*` setting is fixed — but fetching it is still what proves
    // this was entered through the module table.
    // SAFETY: `l` is live and was entered through a closure `set_functions`
    // built.
    let cfg: *mut Config = unsafe { fetch_config(l) };
    if cfg.is_null() {
        // SAFETY: as above.
        unsafe { luaL_error(l, c"BUG: Unable to fetch CJSON configuration".as_ptr()) };
        unreachable_after_raise()
    }

    // The options are read *before* argument 1 is checked, because upstream
    // does: `vim.json.decode({}, 'x')` complains about argument 2, not 1.
    // SAFETY: `l` is live.
    let options = unsafe { read_options(l) };

    let mut length: size_t = 0;
    // SAFETY: `l` is live. `luaL_checklstring` answers `length` bytes plus
    // the NUL Lua keeps past every string, and the value stays on the stack
    // — and so alive — for the whole call, so the slice outlives the parse.
    let data = unsafe {
        let text = luaL_checklstring(l, 1, &raw mut length);
        core::slice::from_raw_parts(text.cast::<u8>(), length + 1)
    };

    // RFC 4627 section 3: only the first character of a JSON document is
    // guaranteed to be ASCII, but that is enough to spot UTF-16 or UTF-32,
    // whose first or second byte is a NUL.
    if length >= 2 && (data[0] == 0 || data[1] == 0) {
        // SAFETY: `l` is live and nothing owns memory here yet.
        unsafe { luaL_error(l, c"JSON parser does not support UTF-16 or UTF-32".as_ptr()) };
        unreachable_after_raise()
    }

    let mut parser = Parser {
        data,
        at: 0,
        index: 0,
        // Sized from the whole document, so appending a decoded string
        // never has to check: it cannot be longer than its own text.
        scratch: Vec::with_capacity(length),
        depth: 0,
        luanil_object: options.luanil_object,
        luanil_array: options.luanil_array,
        skip_comments: options.skip_comments,
    };

    let token = parser.next_token();
    let luanil = parser.luanil_object;
    // SAFETY: `l` is live for the whole parse, and `decode` answers the one
    // value this leaves on the stack.
    unsafe { parser.push_value(l, &token, luanil) };

    let token = parser.next_token();
    if !matches!(token, Token::End) {
        parser.throw(l, c"the end", &token);
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `strtoll`'s reading, which is what upstream hands the token to: a
    /// sign, then digits, saturating rather than wrapping.
    #[test]
    fn integers_are_read_the_way_strtoll_reads_them() {
        assert_eq!(scan_integer(b"0"), (0, 1));
        assert_eq!(scan_integer(b"42,"), (42, 2));
        assert_eq!(scan_integer(b"-7]"), (-7, 2));
        assert_eq!(scan_integer(b"+7"), (7, 2));
        // Not a number at all: nothing consumed, so the caller falls through
        // to `strtod`.
        assert_eq!(scan_integer(b""), (0, 0));
        assert_eq!(scan_integer(b"-"), (0, 0));
        assert_eq!(scan_integer(b"inf"), (0, 0));
        // Leading zeros are digits like any other.
        assert_eq!(scan_integer(b"007"), (7, 3));
    }

    /// The two ends of the range land exactly, and past them the value
    /// saturates rather than wrapping — including `i64::MIN`, which is why
    /// the accumulation runs negative for a negative number.
    #[test]
    fn integers_saturate_at_the_ends_of_the_range() {
        assert_eq!(scan_integer(b"9223372036854775807").0, lua_Integer::MAX);
        assert_eq!(scan_integer(b"-9223372036854775808").0, lua_Integer::MIN);
        assert_eq!(scan_integer(b"9223372036854775808").0, lua_Integer::MAX);
        assert_eq!(scan_integer(b"-9223372036854775809").0, lua_Integer::MIN);
        // The docket's case: past `UINT64_MAX` it is still `LLONG_MAX`, not
        // a re-read as a double.
        assert_eq!(scan_integer(b"18446744073709551616").0, lua_Integer::MAX);
        // And however long the run of digits is.
        let long = [b'9'; 400];
        assert_eq!(scan_integer(&long), (lua_Integer::MAX, 400));
    }

    /// The token classes a byte can start. Everything a scanner has to look
    /// closer at is `Unknown`; everything else is either a one-byte token or
    /// an error.
    #[test]
    fn bytes_classify_as_upstreams_table_does() {
        assert_eq!(classify(b'{'), Class::ObjBegin);
        assert_eq!(classify(b']'), Class::ArrEnd);
        assert_eq!(classify(0), Class::End);
        for byte in b" \t\n\r" {
            assert_eq!(classify(*byte), Class::Whitespace);
        }
        // A scanner's business: strings, numbers and the three words, plus
        // the `inf`/`nan` spellings `decode_invalid_numbers` lets through.
        for byte in b"\"+-0123456789fiInNt" {
            assert_eq!(classify(*byte), Class::Unknown, "{}", *byte as char);
        }
        for byte in b"'\\aez\x7f" {
            assert_eq!(classify(*byte), Class::Error, "{}", *byte as char);
        }
    }

    /// `\x` escapes. `u` stands for itself because the caller has more to do
    /// for it, and 0 means "not an escape".
    #[test]
    fn escapes_map_to_their_bytes() {
        assert_eq!(unescape(b'n'), b'\n');
        assert_eq!(unescape(b't'), b'\t');
        assert_eq!(unescape(b'b'), 0x08);
        assert_eq!(unescape(b'f'), 0x0c);
        assert_eq!(unescape(b'r'), 0x0d);
        assert_eq!(unescape(b'/'), b'/');
        assert_eq!(unescape(b'\\'), b'\\');
        assert_eq!(unescape(b'"'), b'"');
        assert_eq!(unescape(b'u'), b'u');
        assert_eq!(unescape(b'x'), 0);
        assert_eq!(unescape(0), 0);
    }

    /// A parser over a slice, as `decode` builds one: the document plus the
    /// NUL Lua keeps past every string.
    fn parser(text: &[u8]) -> Parser<'_> {
        Parser {
            data: text,
            at: 0,
            index: 0,
            scratch: Vec::new(),
            depth: 0,
            luanil_object: false,
            luanil_array: false,
            skip_comments: false,
        }
    }

    /// The string scanner decodes escapes into the scratch and stops on the
    /// closing quote — and on the NUL that ends the document, which is what
    /// keeps it from running off the end.
    #[test]
    fn strings_decode_into_the_scratch() {
        let mut p = parser(b"\"a\\nb\\u0041\\ud83d\\ude00\"\0");
        assert!(matches!(p.string_token(), Token::Str));
        assert_eq!(p.scratch, "a\nbA\u{1f600}".as_bytes());
        assert_eq!(p.at, 24);

        // A lone high surrogate is not a pair, so the escape is rejected.
        let mut p = parser(b"\"\\ud83d\"\0");
        assert!(matches!(p.string_token(), Token::Error(_)));

        // An unterminated string stops at the NUL rather than reading on.
        let mut p = parser(b"\"abc\0");
        assert!(matches!(p.string_token(), Token::Error(_)));
        assert_eq!(p.at, 4);
    }

    /// A lone `\uD800` is *kept* as a three-byte encoding of the surrogate,
    /// which `char::encode_utf8` would refuse — upstream's own encoder is
    /// why this port has one.
    #[test]
    fn a_paired_surrogate_is_encoded_but_a_lone_one_is_not_rejected_by_the_encoder() {
        let mut p = parser(b"\"\\ud800\\udc00\"\0");
        assert!(matches!(p.string_token(), Token::Str));
        assert_eq!(p.scratch, "\u{10000}".as_bytes());
    }

    /// Which numbers are the ones strict JSON rejects. This is only ever
    /// consulted for a token that did not start with a digit or `-`, and it
    /// is what lets `inf`/`nan`/hex through to `strtod`.
    #[test]
    fn invalid_number_shapes_are_the_ones_upstream_names() {
        for text in [
            &b"+1\0"[..],
            b"0x10\0",
            b"-0x10\0",
            b"01\0",
            b"inf\0",
            b"Infinity\0",
            b"nan\0",
            b"NaN\0",
        ] {
            assert!(parser(text).is_invalid_number(), "{text:?}");
        }
        for text in [&b"1\0"[..], b"-1\0", b"0\0", b"-0\0", b"0.5\0", b"true\0"] {
            assert!(!parser(text).is_invalid_number(), "{text:?}");
        }
    }

    /// An integer token stays an integer; anything only a float can continue
    /// with goes to `strtod`.
    #[test]
    fn numbers_split_between_the_integer_and_the_float_scanner() {
        assert!(matches!(parser(b"12\0").number_token(), Token::Integer(12)));
        assert!(matches!(
            parser(b"-3,\0").number_token(),
            Token::Integer(-3)
        ));
        let Token::Number(value) = parser(b"1.5\0").number_token() else {
            panic!("1.5 is a float");
        };
        assert_eq!(value, 1.5);
        let Token::Number(value) = parser(b"2e3\0").number_token() else {
            panic!("2e3 is a float");
        };
        assert_eq!(value, 2000.0);
        // `decode_invalid_numbers` is 1, so hex reaches `strtod`.
        let Token::Number(value) = parser(b"0x10\0").number_token() else {
            panic!("hex goes to strtod");
        };
        assert_eq!(value, 16.0);
    }

    /// Comments are skipped only when asked, and an unclosed block comment
    /// is an error rather than a run off the end.
    #[test]
    fn comments_are_skipped_only_with_the_option() {
        let mut p = parser(b"// hi\n1\0");
        p.skip_comments = true;
        assert!(matches!(p.next_token(), Token::Integer(1)));

        let mut p = parser(b"/* hi */ 1\0");
        p.skip_comments = true;
        assert!(matches!(p.next_token(), Token::Integer(1)));

        let mut p = parser(b"/* hi\0");
        p.skip_comments = true;
        assert!(matches!(p.next_token(), Token::Error(_)));

        // Without the option a slash is not a token at all.
        assert!(matches!(
            parser(b"// hi\n1\0").next_token(),
            Token::Error(_)
        ));
    }

    /// The one-byte tokens and the three words, in order, off one document.
    #[test]
    fn the_token_stream_is_upstreams() {
        let mut p = parser(b"{ [ true , false : null ] } \0");
        let names: Vec<&str> = core::iter::from_fn(|| {
            let token = p.next_token();
            if matches!(token, Token::End) {
                return None;
            }
            Some(token.name().to_str().expect("ASCII"))
        })
        .collect();
        assert_eq!(
            names,
            [
                "T_OBJ_BEGIN",
                "T_ARR_BEGIN",
                "T_BOOLEAN",
                "T_COMMA",
                "T_BOOLEAN",
                "T_COLON",
                "T_NULL",
                "T_ARR_END",
                "T_OBJ_END",
            ]
        );
    }
}
