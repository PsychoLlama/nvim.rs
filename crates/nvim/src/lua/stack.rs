//! The typed `Stack` over a `lua_State`: the one module that pushes, pops,
//! reads and indexes LuaJIT's value stack.
//!
//! A [`Stack`] is the running C function's view of its own frame. Every
//! operation on it is one `lua_*` call against a state the `Stack` holds live
//! for as long as it exists, so the operations are safe; what a `Stack` cannot
//! know is *which value sits in which slot*, and that stays the caller's
//! bookkeeping, spelled with [`At`] indices rather than bare integers.
//!
//! # Building one
//!
//! - [`Stack::entry`] is how a `lua_CFunction` starts: it hands the body a
//!   `Stack` over the frame Lua called it with and raises the body's error,
//!   if any, once the body has returned.
//! - [`Stack::from_raw`] is the step for code that already holds a raw
//!   `lua_State` and has not been ported onto `entry` yet.
//!
//! Both are `unsafe` because the `Stack` cannot check that the pointer is a
//! live state. Everything after them is safe.
//!
//! # Borrowed strings
//!
//! [`Stack::string`] lends a Lua string's bytes for as long as the `Stack` is
//! borrowed. Lua keeps a string alive while it sits in a stack slot, and every
//! operation that could pop or overwrite that slot takes `&mut self`, so the
//! borrow checker is what says "copy the bytes before you pop". The one
//! assumption behind it is that a frame has only one `Stack` at a time — the
//! contract both constructors carry. A nested call into Lua cannot break it:
//! a C function called from there gets a frame of its own and cannot reach
//! below its base.
//!
//! # Raising an error
//!
//! A Lua error is a `longjmp` (on x86-64 LuaJIT, a foreign unwind) out of the
//! C function. Whatever a Rust frame between the raise and the catch still
//! owns is at the unwinder's mercy, so the rule here is that **nothing owned
//! is alive when Lua raises**: a body answers `Err(`[`Raise`]`)` instead of
//! raising, its frame returns and drops every `String`, `Vec`, `TypVal` and
//! `Object` it held, and only then does [`Stack::entry`] hand the error to
//! LuaJIT. The `Raise` itself is copied onto the Lua stack and dropped before
//! the jump. There is no other way to raise through a `Stack`: no method here
//! calls `lua_error` or a `luaL_*` that throws, the checks answer `Result`, and
//! the raw `lua_call` (which re-raises its callee's error) is not offered —
//! [`Stack::pcall`] is. So the discipline is checkable with one grep: a raise
//! this tree *issues* is either `Stack::entry` (safe by construction), or a
//! raw `lua_error`/`luaL_error`/`luaL_argerror`/`luaL_check*` outside this
//! file, each of which is a call site still to port.
//!
//! What a `Stack` cannot rule out is an error Lua raises on its own: the
//! accesses that honour metamethods ([`Stack::get_field`], [`Stack::set_field`],
//! [`Stack::get_table`], [`Stack::set_table`], [`Stack::concat`]) run whatever
//! Lua code the metatable names, and any allocation can fail with LuaJIT's
//! out-of-memory error. Those unwind through the Rust frames between (every
//! binding is `C-unwind`); a walk over values the user handed in uses the
//! `raw_*` accesses, which run no Lua.
//!
//! The `luaL_check*` family keeps LuaJIT's own wording without restating it:
//! a check that fails records which check it was, and the raise runs the very
//! C check on the same argument, which then throws its own message. A check's
//! Rust half is written to fail exactly when the C half does.
//!
//! # The header's macros
//!
//! `lua.h` and `lauxlib.h` spell part of the C API as macros, which c2rust
//! expanded at every use. The raw `unsafe fn` forms at the bottom of this file
//! are those macros written out once, for the call sites that have not moved
//! onto [`Stack`] yet; each is the macro's definition verbatim.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/stack.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::{self, NonNull};

use crate::lua::executor::{nlua_ref_global, nlua_unref_global};
use crate::lua::ffi::{
    LUA_GLOBALSINDEX, LUA_REGISTRYINDEX, LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TLIGHTUSERDATA, LUA_TNIL,
    LUA_TNONE, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LUA_TTHREAD, LUA_TUSERDATA, lua_checkstack,
    lua_concat, lua_createtable, lua_error, lua_getfield, lua_getmetatable, lua_gettable,
    lua_gettop, lua_insert, lua_isnumber, lua_newuserdata, lua_next, lua_objlen, lua_pcall,
    lua_pushboolean, lua_pushcclosure, lua_pushinteger, lua_pushlightuserdata, lua_pushlstring,
    lua_pushnil, lua_pushnumber, lua_pushvalue, lua_rawequal, lua_rawget, lua_rawgeti, lua_rawset,
    lua_rawseti, lua_remove, lua_replace, lua_setfield, lua_setmetatable, lua_settable, lua_settop,
    lua_toboolean, lua_tointeger, lua_tolstring, lua_tonumber, lua_topointer, lua_touserdata,
    lua_type, lua_typename, luaL_argerror, luaL_checkinteger, luaL_checklstring, luaL_checknumber,
    luaL_checktype, luaL_checkudata, luaL_loadbuffer, luaL_newmetatable, luaL_where,
};
use crate::lua::state::nlua_global_refs;
use crate::types::{LuaRef, NluaRefState, lua_Integer, lua_Number, lua_State, size_t};

/// A C function Lua can call: the `lua_CFunction` signature, unwrapped.
///
/// `C-unwind` because a Lua error raised inside one unwinds through it.
pub type CFunction = unsafe extern "C-unwind" fn(*mut lua_State) -> c_int;

// ---------------------------------------------------------------------------
// Indices

/// A slot on the Lua stack.
///
/// Positive is absolute — `At(1)` is the first argument of a C function, and
/// stays that slot whatever is pushed above it — negative counts down from the
/// top, and the pseudo-indices name the registry, the globals and a C
/// closure's upvalues. A slot a caller wants to come back to after pushing
/// should be absolute: [`Stack::top`] answers one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct At(c_int);

impl At {
    /// The value on top of the stack.
    pub const TOP: At = At(-1);
    /// The registry, where references and named metatables live.
    pub const REGISTRY: At = At(LUA_REGISTRYINDEX);
    /// The globals table.
    pub const GLOBALS: At = At(LUA_GLOBALSINDEX);

    /// The slot `n` down from the top: `from_top(1)` is [`At::TOP`].
    pub const fn from_top(n: c_int) -> At {
        At(-n)
    }

    /// A C function's `n`th argument, counting from 1; also any absolute slot.
    pub const fn arg(n: c_int) -> At {
        At(n)
    }

    /// Upvalue `n` of the running C closure, counting from 1.
    pub const fn upvalue(n: c_int) -> At {
        At(lua_upvalueindex(n))
    }

    /// The slot `n` above this absolute one.
    pub const fn above(self, n: c_int) -> At {
        debug_assert!(self.0 > 0, "only an absolute slot has a fixed neighbour");
        At(self.0 + n)
    }

    /// The raw index, for the `lua_*` calls that have not moved here yet.
    pub const fn index(self) -> c_int {
        self.0
    }
}

/// What kind of value a slot holds: `lua_type`'s answer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// The index is past the top of the stack.
    None,
    Nil,
    Boolean,
    LightUserdata,
    Number,
    String,
    Table,
    Function,
    Userdata,
    Thread,
}

impl Kind {
    fn from_raw(kind: c_int) -> Kind {
        match kind {
            LUA_TNIL => Kind::Nil,
            LUA_TBOOLEAN => Kind::Boolean,
            LUA_TLIGHTUSERDATA => Kind::LightUserdata,
            LUA_TNUMBER => Kind::Number,
            LUA_TSTRING => Kind::String,
            LUA_TTABLE => Kind::Table,
            LUA_TFUNCTION => Kind::Function,
            LUA_TUSERDATA => Kind::Userdata,
            LUA_TTHREAD => Kind::Thread,
            _ => Kind::None,
        }
    }

    const fn to_raw(self) -> c_int {
        match self {
            Kind::None => LUA_TNONE,
            Kind::Nil => LUA_TNIL,
            Kind::Boolean => LUA_TBOOLEAN,
            Kind::LightUserdata => LUA_TLIGHTUSERDATA,
            Kind::Number => LUA_TNUMBER,
            Kind::String => LUA_TSTRING,
            Kind::Table => LUA_TTABLE,
            Kind::Function => LUA_TFUNCTION,
            Kind::Userdata => LUA_TUSERDATA,
            Kind::Thread => LUA_TTHREAD,
        }
    }
}

// ---------------------------------------------------------------------------
// Errors

/// A Lua error a C function's body answers instead of raising it.
///
/// [`Stack::entry`] raises it once the body has returned. Building one owns
/// nothing Lua can strand: the message is copied onto the Lua stack and the
/// `Raise` dropped before the jump.
#[derive(Debug)]
#[must_use]
pub struct Raise(Thrown);

#[derive(Debug)]
enum Thrown {
    /// `luaL_error`'s shape: where the running function is, then the message.
    Message(Vec<u8>),
    /// `luaL_argerror`'s: `bad argument #n to 'f' (message)`.
    Argument { arg: c_int, message: Vec<u8> },
    /// A `luaL_check*` that refused, raised by the C check itself.
    Check { arg: c_int, check: Check },
    /// The value already on top of the stack, raised as it is.
    Value,
}

/// Which `luaL_check*` refused.
#[derive(Debug, Clone, Copy)]
enum Check {
    Integer,
    Number,
    String,
    Kind(Kind),
    Userdata(&'static CStr),
}

impl Raise {
    /// `luaL_error(L, "%s", message)`: the message, after the position of the
    /// running function when it is a Lua one (a C function has none, so from
    /// a binding this is the message alone).
    ///
    /// Unlike `luaL_error`'s `%s`, a NUL inside the message is kept.
    pub fn message(message: impl Into<Vec<u8>>) -> Raise {
        Raise(Thrown::Message(message.into()))
    }

    /// `luaL_argerror(L, arg, message)`: `bad argument #arg to 'f' (message)`.
    ///
    /// The message is cut at its first NUL, as `luaL_argerror`'s `%s` cuts it.
    pub fn argument(arg: c_int, message: impl Into<Vec<u8>>) -> Raise {
        Raise(Thrown::Argument {
            arg,
            message: message.into(),
        })
    }

    /// Raise the value on top of the stack as the error, as `lua_error` does:
    /// what a failed [`Stack::pcall`] leaves there, re-raised.
    pub fn top_value() -> Raise {
        Raise(Thrown::Value)
    }

    fn check(arg: At, check: Check) -> Raise {
        Raise(Thrown::Check { arg: arg.0, check })
    }
}

/// A [`Stack::pcall`] or [`Stack::load`] that failed: LuaJIT's status code.
/// The error value is on top of the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LuaFailed(pub c_int);

// ---------------------------------------------------------------------------
// Userdata

/// A Rust value Lua holds as full userdata, under a metatable of its own.
///
/// # Safety
/// `METATABLE` names a registry metatable that only values of `Self` carry:
/// every userdata with that metatable was made by [`Stack::new_userdata`] for
/// this same type. That is what lets [`Stack::check_userdata`] hand the bytes
/// back as a `Self`. `Self` must also be valid when its bytes are copied,
/// because Lua moves nothing but never runs `Drop` either: a userdata whose
/// type owns something needs a `__gc` that releases it.
pub unsafe trait Userdata: Sized {
    /// The name the metatable is registered under (`luaL_newmetatable`).
    const METATABLE: &'static CStr;
}

// ---------------------------------------------------------------------------
// The stack

/// The running C function's frame of a Lua state.
///
/// Not `Copy`, not `Send`: one per frame, on the thread that runs the state.
/// See the [module documentation](self) for what it guarantees.
pub struct Stack {
    state: NonNull<lua_State>,
}

impl Stack {
    /// A `Stack` over `state`'s current frame.
    ///
    /// # Safety
    /// `state` is a live Lua state set up by this editor's runtime (its
    /// reference bookkeeping installed), on the thread that runs it, and it
    /// outlives the returned `Stack`. No other `Stack` over the same frame is
    /// used while this one is.
    pub unsafe fn from_raw(state: *mut lua_State) -> Stack {
        debug_assert!(!state.is_null(), "a Stack over no Lua state");
        Stack {
            // SAFETY: the caller's promise that `state` is live.
            state: unsafe { NonNull::new_unchecked(state) },
        }
    }

    /// Run a C function's `body` over the frame Lua called it with, and raise
    /// the error it answers once it has returned.
    ///
    /// This is the only way a `Stack` raises a Lua error, and why the error
    /// never strands an owned value: by the time it is raised, every local of
    /// `body` has been dropped. Call it as the C function's whole body, so its
    /// own frame owns nothing either.
    ///
    /// # Safety
    /// As [`Stack::from_raw`]: `state` is the state Lua called the C function
    /// on, which is LuaJIT's contract for a `lua_CFunction`.
    pub unsafe fn entry(
        state: *mut lua_State,
        body: impl FnOnce(&mut Stack) -> Result<c_int, Raise>,
    ) -> c_int {
        let outcome = {
            // SAFETY: the caller's promise; this is the frame's only `Stack`.
            let mut stack = unsafe { Stack::from_raw(state) };
            body(&mut stack)
        };
        match outcome {
            Ok(results) => results,
            // SAFETY: the state Lua called this function on, with nothing of
            // this frame's left to release.
            Err(error) => unsafe { raise(state, error) },
        }
    }

    /// The raw state, for the callees that still take one.
    pub fn lua_state(&self) -> *mut lua_State {
        self.state.as_ptr()
    }

    fn l(&self) -> *mut lua_State {
        self.state.as_ptr()
    }

    // -- index and balance ------------------------------------------------

    /// The top slot, as an absolute index; `At(0)` when the frame is empty.
    pub fn top(&self) -> At {
        // SAFETY: a live state (the constructor's contract), as everywhere
        // below.
        At(unsafe { lua_gettop(self.l()) })
    }

    /// The absolute form of `at` (a pseudo-index stays as it is).
    pub fn absolute(&self, at: At) -> At {
        if at.0 < 0 && at.0 > LUA_REGISTRYINDEX {
            At(self.top().0 + at.0 + 1)
        } else {
            at
        }
    }

    /// Make `at` the top: drop what is above it, or pad with nils.
    pub fn set_top(&mut self, at: At) {
        unsafe { lua_settop(self.l(), at.0) };
    }

    /// Drop the top `n` values.
    pub fn pop(&mut self, n: c_int) {
        unsafe { lua_settop(self.l(), -n - 1) };
    }

    /// Move the top value down to `at`, shifting what was there up.
    pub fn insert(&mut self, at: At) {
        unsafe { lua_insert(self.l(), at.0) };
    }

    /// Remove the value at `at`, shifting what was above it down.
    pub fn remove(&mut self, at: At) {
        unsafe { lua_remove(self.l(), at.0) };
    }

    /// Pop the top value into `at`.
    pub fn replace(&mut self, at: At) {
        unsafe { lua_replace(self.l(), at.0) };
    }

    /// `lua_checkstack(L, slots)`: make sure `slots` more values fit above
    /// the top, answering whether they do.
    pub fn reserve(&mut self, slots: c_int) -> bool {
        unsafe { lua_checkstack(self.l(), slots) != 0 }
    }

    // -- reading slots ----------------------------------------------------

    /// What kind of value `at` holds.
    pub fn kind(&self, at: At) -> Kind {
        Kind::from_raw(unsafe { lua_type(self.l(), at.0) })
    }

    /// Whether `at` holds nil.
    pub fn is_nil(&self, at: At) -> bool {
        self.kind(at) == Kind::Nil
    }

    /// Whether `at` is past the top, or holds nil.
    pub fn is_none_or_nil(&self, at: At) -> bool {
        matches!(self.kind(at), Kind::None | Kind::Nil)
    }

    /// Whether `at` holds a number, or a string that converts to one.
    pub fn is_number(&self, at: At) -> bool {
        unsafe { lua_isnumber(self.l(), at.0) != 0 }
    }

    /// The value's truthiness: false only for nil and `false`.
    pub fn to_boolean(&self, at: At) -> bool {
        unsafe { lua_toboolean(self.l(), at.0) != 0 }
    }

    /// The value as a number; 0 when it is neither a number nor a numeric
    /// string.
    pub fn to_number(&self, at: At) -> lua_Number {
        unsafe { lua_tonumber(self.l(), at.0) }
    }

    /// The value as an integer (truncated); 0 when it is not a number.
    pub fn to_integer(&self, at: At) -> lua_Integer {
        unsafe { lua_tointeger(self.l(), at.0) }
    }

    /// The bytes of the string at `at`; `None` for any other kind.
    ///
    /// Only an actual string answers, so a number is never converted in place
    /// (which would derail a `lua_next` walking the table it is a key of).
    /// The bytes are borrowed for as long as the `Stack` is; copy them before
    /// popping the slot.
    pub fn string(&self, at: At) -> Option<&[u8]> {
        if self.kind(at) != Kind::String {
            return None;
        }
        Some(self.string_bytes(at))
    }

    /// The bytes of `at` converted to a string, as `lua_tolstring` does: a
    /// number is converted *in place*. `None` for any other kind.
    pub fn to_bytes(&mut self, at: At) -> Option<&[u8]> {
        let mut len: size_t = 0;
        let bytes = unsafe { lua_tolstring(self.l(), at.0, &raw mut len) };
        if bytes.is_null() {
            return None;
        }
        // SAFETY: a Lua string is `len` readable bytes, alive while its slot
        // is, and the slot outlives the borrow of `self`.
        Some(unsafe { core::slice::from_raw_parts(bytes.cast::<u8>(), len) })
    }

    /// [`Stack::string`] for a slot already known to hold a string.
    fn string_bytes(&self, at: At) -> &[u8] {
        let mut len: size_t = 0;
        let bytes = unsafe { lua_tolstring(self.l(), at.0, &raw mut len) };
        debug_assert!(!bytes.is_null());
        // SAFETY: as in `to_bytes`; `lua_tolstring` on a string converts
        // nothing.
        unsafe { core::slice::from_raw_parts(bytes.cast::<u8>(), len) }
    }

    /// Whether the two slots hold the very same value, metamethods aside.
    pub fn raw_equal(&self, a: At, b: At) -> bool {
        unsafe { lua_rawequal(self.l(), a.0, b.0) != 0 }
    }

    /// `#value` without metamethods: a string's length, a table's border,
    /// a userdata's size.
    pub fn raw_len(&self, at: At) -> usize {
        unsafe { lua_objlen(self.l(), at.0) }
    }

    /// The value's identity, for a table, function, thread or userdata;
    /// null for anything else. Only ever compared, never dereferenced.
    pub fn identity(&self, at: At) -> *const c_void {
        unsafe { lua_topointer(self.l(), at.0) }
    }

    /// The name of `kind`, as Lua's own messages spell it.
    pub fn kind_name(&self, kind: Kind) -> &'static CStr {
        // SAFETY: `lua_typename` answers a static, NUL-terminated name for
        // every kind, `LUA_TNONE` included.
        unsafe { CStr::from_ptr(lua_typename(self.l(), kind.to_raw())) }
    }

    // -- pushing ----------------------------------------------------------

    pub fn push_nil(&mut self) {
        unsafe { lua_pushnil(self.l()) };
    }

    pub fn push_boolean(&mut self, value: bool) {
        unsafe { lua_pushboolean(self.l(), c_int::from(value)) };
    }

    pub fn push_number(&mut self, value: lua_Number) {
        unsafe { lua_pushnumber(self.l(), value) };
    }

    pub fn push_integer(&mut self, value: lua_Integer) {
        unsafe { lua_pushinteger(self.l(), value) };
    }

    /// Push `bytes` as a Lua string, which copies them — NULs included.
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        // An empty slice may dangle, which `lua_pushlstring` does not mind
        // for zero bytes, but a null pointer it would.
        unsafe { lua_pushlstring(self.l(), bytes.as_ptr().cast::<c_char>(), bytes.len()) };
    }

    /// Push a copy of the value at `at`.
    pub fn push_value(&mut self, at: At) {
        unsafe { lua_pushvalue(self.l(), at.0) };
    }

    /// Push `function` as a Lua function with no upvalues.
    pub fn push_function(&mut self, function: CFunction) {
        self.push_closure(function, 0);
    }

    /// Push `function` as a C closure over the top `upvalues` values, which it
    /// pops; the closure reads them back through [`At::upvalue`].
    pub fn push_closure(&mut self, function: CFunction, upvalues: c_int) {
        unsafe { lua_pushcclosure(self.l(), Some(function), upvalues) };
    }

    /// Push a light userdata: a bare pointer Lua stores and hands back but
    /// never dereferences.
    // Clippy reads the pointer reaching an FFI call as a dereference; Lua
    // only stores it.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn push_light_userdata(&mut self, pointer: *mut c_void) {
        unsafe { lua_pushlightuserdata(self.l(), pointer) };
    }

    /// Push a fresh table with room for `array` sequence and `record` keyed
    /// entries.
    pub fn new_table(&mut self, array: usize, record: usize) {
        let (array, record) = (saturate(array), saturate(record));
        unsafe { lua_createtable(self.l(), array, record) };
    }

    /// Concatenate the top `n` values into one string, as `..` does.
    pub fn concat(&mut self, n: c_int) {
        unsafe { lua_concat(self.l(), n) };
    }

    // -- tables -----------------------------------------------------------

    /// Push `table[key]` for the table at `at`, metamethods included.
    pub fn get_field(&mut self, at: At, key: &CStr) {
        unsafe { lua_getfield(self.l(), at.0, key.as_ptr()) };
    }

    /// Pop a value into `table[key]` for the table at `at`.
    pub fn set_field(&mut self, at: At, key: &CStr) {
        unsafe { lua_setfield(self.l(), at.0, key.as_ptr()) };
    }

    /// Pop a key and push `table[key]`, metamethods included.
    pub fn get_table(&mut self, at: At) {
        unsafe { lua_gettable(self.l(), at.0) };
    }

    /// Pop a value and a key and set `table[key] = value`, metamethods
    /// included.
    pub fn set_table(&mut self, at: At) {
        unsafe { lua_settable(self.l(), at.0) };
    }

    /// [`Stack::get_table`] without metamethods.
    pub fn raw_get(&mut self, at: At) {
        unsafe { lua_rawget(self.l(), at.0) };
    }

    /// [`Stack::set_table`] without metamethods.
    pub fn raw_set(&mut self, at: At) {
        unsafe { lua_rawset(self.l(), at.0) };
    }

    /// Push `table[n]`, without metamethods.
    pub fn raw_get_index(&mut self, at: At, n: c_int) {
        unsafe { lua_rawgeti(self.l(), at.0, n) };
    }

    /// Pop a value into `table[n]`, without metamethods.
    pub fn raw_set_index(&mut self, at: At, n: c_int) {
        unsafe { lua_rawseti(self.l(), at.0, n) };
    }

    /// Push the metatable of the value at `at` and answer `true`, or push
    /// nothing and answer `false` when it has none.
    pub fn get_metatable(&mut self, at: At) -> bool {
        unsafe { lua_getmetatable(self.l(), at.0) != 0 }
    }

    /// Pop a table and make it the metatable of the value at `at`.
    pub fn set_metatable(&mut self, at: At) {
        unsafe { lua_setmetatable(self.l(), at.0) };
    }

    /// One step of a table walk: pop a key (nil to start) and push the next
    /// key and its value, answering `true`; `false` once the table at `at`
    /// is exhausted, having pushed nothing.
    ///
    /// The key left below the value is what the next step pops, so it must
    /// not be converted in place in between — [`Stack::string`] never does.
    pub fn next(&mut self, at: At) -> bool {
        unsafe { lua_next(self.l(), at.0) != 0 }
    }

    /// Push the global `name`.
    pub fn get_global(&mut self, name: &CStr) {
        self.get_field(At::GLOBALS, name);
    }

    /// Pop a value into the global `name`.
    pub fn set_global(&mut self, name: &CStr) {
        self.set_field(At::GLOBALS, name);
    }

    /// Set each `(name, function)` as a field of the table on top, as
    /// `luaL_register(L, NULL, ..)` does.
    pub fn register(&mut self, functions: &[(&CStr, CFunction)]) {
        for &(name, function) in functions {
            self.push_function(function);
            self.set_field(At::from_top(2), name);
        }
    }

    /// Push the registry metatable called `name`, creating it when it does
    /// not exist yet; `true` when it was created.
    pub fn new_metatable(&mut self, name: &CStr) -> bool {
        unsafe { luaL_newmetatable(self.l(), name.as_ptr()) != 0 }
    }

    // -- the registry -----------------------------------------------------

    /// Push what `reference` refers to (nil for `LUA_NOREF`/`LUA_REFNIL`).
    pub fn push_reference(&mut self, reference: LuaRef) {
        self.raw_get_index(At::REGISTRY, reference);
    }

    /// A new reference to the value at `at`, counted in the editor's
    /// reference bookkeeping. The caller owns it and gives it back with
    /// [`Stack::unreference`] (or by dropping the value that holds it).
    pub fn reference(&mut self, at: At) -> LuaRef {
        // SAFETY: a live state whose bookkeeping is installed (the
        // constructor's contract).
        unsafe { nlua_ref_global(self.l(), at.0) }
    }

    /// Give `reference` back. A sentinel is a no-op.
    pub fn unreference(&mut self, reference: LuaRef) {
        // SAFETY: as in `reference`.
        unsafe { nlua_unref_global(self.l(), reference) };
    }

    /// Push `vim.NIL`, the userdata Lua code sees for a Vimscript or API nil.
    pub fn push_vim_nil(&mut self) {
        let reference = self.refs().nil_ref;
        self.push_reference(reference);
    }

    /// Push the metatable `vim.empty_dict()` marks an empty dictionary with.
    pub fn push_empty_dict_metatable(&mut self) {
        let reference = self.refs().empty_dict_ref;
        self.push_reference(reference);
    }

    fn refs(&self) -> &NluaRefState {
        let refs = nlua_global_refs.get();
        debug_assert!(!refs.is_null(), "the Lua runtime is not set up");
        // SAFETY: the constructor's contract — the editor's runtime is up, so
        // its bookkeeping is installed and lives as long as the state.
        unsafe { &*refs }
    }

    // -- calling ----------------------------------------------------------

    /// Call the function below the top `args` values, in protected mode,
    /// leaving `results` values (or all of them, for `LUA_MULTRET`).
    ///
    /// On failure the error value is on top and [`LuaFailed`] carries the
    /// status. `handler`, when given, is the message handler's slot.
    pub fn pcall(
        &mut self,
        args: c_int,
        results: c_int,
        handler: Option<At>,
    ) -> Result<(), LuaFailed> {
        let handler = handler.map_or(0, |at| self.absolute(at).0);
        match unsafe { lua_pcall(self.l(), args, results, handler) } {
            0 => Ok(()),
            status => Err(LuaFailed(status)),
        }
    }

    /// Compile `chunk` and push it as a function, or push the error message.
    pub fn load(&mut self, chunk: &[u8], name: &CStr) -> Result<(), LuaFailed> {
        let text = chunk.as_ptr().cast::<c_char>();
        match unsafe { luaL_loadbuffer(self.l(), text, chunk.len(), name.as_ptr()) } {
            0 => Ok(()),
            status => Err(LuaFailed(status)),
        }
    }

    // -- checks -----------------------------------------------------------

    /// `luaL_checkinteger`: the argument at `at` as an integer, or the error
    /// LuaJIT raises for it.
    pub fn check_integer(&self, at: At) -> Result<lua_Integer, Raise> {
        let value = self.to_integer(at);
        // `luaL_checkinteger`'s own test: zero is ambiguous, so ask again.
        if value == 0 && !self.is_number(at) {
            return Err(Raise::check(at, Check::Integer));
        }
        Ok(value)
    }

    /// `luaL_checknumber`.
    pub fn check_number(&self, at: At) -> Result<lua_Number, Raise> {
        let value = self.to_number(at);
        if value == 0.0 && !self.is_number(at) {
            return Err(Raise::check(at, Check::Number));
        }
        Ok(value)
    }

    /// `luaL_checklstring`: the argument's bytes, converting a number in
    /// place as LuaJIT does.
    pub fn check_bytes(&mut self, at: At) -> Result<&[u8], Raise> {
        // Two steps, so that the refusal does not hold the borrow.
        if self.to_bytes(at).is_none() {
            return Err(Raise::check(at, Check::String));
        }
        Ok(self.to_bytes(at).unwrap_or_default())
    }

    /// `luaL_checktype`.
    pub fn check_kind(&self, at: At, kind: Kind) -> Result<(), Raise> {
        if self.kind(at) != kind {
            return Err(Raise::check(at, Check::Kind(kind)));
        }
        Ok(())
    }

    // -- userdata ---------------------------------------------------------

    /// Push `value` as a full userdata carrying `T`'s metatable, which must
    /// already be registered (see [`Stack::new_metatable`]).
    pub fn new_userdata<T: Userdata>(&mut self, value: T) {
        const {
            // LuaJIT aligns a userdata's payload to 8 bytes.
            assert!(align_of::<T>() <= 8, "LuaJIT cannot align this userdata");
        }
        let block = unsafe { lua_newuserdata(self.l(), size_of::<T>()) }.cast::<T>();
        // SAFETY: a fresh block of `size_of::<T>()` bytes, aligned for `T`
        // (checked above), that nothing else has seen.
        unsafe { ptr::write(block, value) };
        self.get_field(At::REGISTRY, T::METATABLE);
        self.set_metatable(At::from_top(2));
    }

    /// The `T` at `at` when the slot holds one of `T`'s userdata.
    pub fn to_userdata<T: Userdata>(&mut self, at: At) -> Option<&mut T> {
        // A light userdata answers `lua_touserdata` too, but is not one of
        // ours: `luaL_checkudata` asks for a full userdata first.
        if self.kind(at) != Kind::Userdata {
            return None;
        }
        let block = unsafe { lua_touserdata(self.l(), at.0) };
        if block.is_null() || !self.get_metatable(at) {
            return None;
        }
        self.get_field(At::REGISTRY, T::METATABLE);
        let ours = self.raw_equal(At::from_top(1), At::from_top(2));
        self.pop(2);
        // SAFETY: the `Userdata` contract — only `new_userdata::<T>` makes a
        // userdata with `T`'s metatable — so the block holds a `T`; the
        // borrow of `self` keeps the slot, and so the block, alive.
        ours.then(|| unsafe { &mut *block.cast::<T>() })
    }

    /// `luaL_checkudata`: the `T` at `at`, or the error LuaJIT raises for it.
    pub fn check_userdata<T: Userdata>(&mut self, at: At) -> Result<&mut T, Raise> {
        if self.to_userdata::<T>(at).is_none() {
            return Err(Raise::check(at, Check::Userdata(T::METATABLE)));
        }
        Ok(self.to_userdata::<T>(at).expect("checked above"))
    }
}

/// A size as a `lua_createtable` hint, which is only a hint: past `c_int` it
/// asks for the most it can.
fn saturate(n: usize) -> c_int {
    c_int::try_from(n).unwrap_or(c_int::MAX)
}

/// Raise `error` on `state` and never come back.
///
/// Every owned part of `error` is copied onto the Lua stack and dropped
/// before the jump.
///
/// # Safety
/// `state` is the live state of the running C function.
#[cold]
#[inline(never)]
unsafe fn raise(state: *mut lua_State, error: Raise) -> ! {
    // SAFETY: the caller's state, throughout.
    unsafe {
        match error.0 {
            Thrown::Message(message) => {
                luaL_where(state, 1);
                lua_pushlstring(state, message.as_ptr().cast::<c_char>(), message.len());
                drop(message);
                lua_concat(state, 2);
            }
            Thrown::Argument { arg, message } => {
                // The copy on the stack is NUL-terminated (every Lua string
                // is) and stays alive for `luaL_argerror` to format.
                lua_pushlstring(state, message.as_ptr().cast::<c_char>(), message.len());
                drop(message);
                let message = lua_tolstring(state, -1, ptr::null_mut());
                luaL_argerror(state, arg, message);
            }
            Thrown::Check { arg, check } => {
                // The C check fails on the same argument the Rust half did,
                // and throws LuaJIT's own message.
                match check {
                    Check::Integer => _ = luaL_checkinteger(state, arg),
                    Check::Number => _ = luaL_checknumber(state, arg),
                    Check::String => _ = luaL_checklstring(state, arg, ptr::null_mut()),
                    Check::Kind(kind) => luaL_checktype(state, arg, kind.to_raw()),
                    Check::Userdata(name) => _ = luaL_checkudata(state, arg, name.as_ptr()),
                }
                // A check that passes after its Rust half refused is a bug
                // in that half; there is no message to raise.
                std::process::abort();
            }
            Thrown::Value => {}
        }
        lua_error(state);
    }
    // `lua_error` does not return.
    std::process::abort()
}

// ---------------------------------------------------------------------------
// The header's macros, for raw call sites not yet on `Stack`

/// Drop the top `n` values.
///
/// # Safety
/// `L` must be a live Lua state with at least `n` values on its stack.
#[allow(non_snake_case)]
pub unsafe fn lua_pop(L: *mut lua_State, n: c_int) {
    unsafe { lua_settop(L, -n - 1) };
}

/// Push a fresh empty table.
///
/// # Safety
/// `L` must be a live Lua state with room for one more value.
#[allow(non_snake_case)]
pub unsafe fn lua_newtable(L: *mut lua_State) {
    unsafe { lua_createtable(L, 0, 0) };
}

/// Push `f` as a Lua function taking no upvalues.
///
/// # Safety
/// `L` must be a live Lua state with room for one more value.
#[allow(non_snake_case)]
pub unsafe fn lua_pushcfunction(L: *mut lua_State, f: CFunction) {
    unsafe { lua_pushcclosure(L, Some(f), 0) };
}

/// Push the global named `k`.
///
/// # Safety
/// `L` must be a live Lua state and `k` a NUL-terminated name.
#[allow(non_snake_case)]
pub unsafe fn lua_getglobal(L: *mut lua_State, k: *const c_char) {
    unsafe { lua_getfield(L, LUA_GLOBALSINDEX, k) };
}

/// Store the value on top of the stack in the global named `k`.
///
/// # Safety
/// `L` must be a live Lua state with a value on top, `k` a NUL-terminated name.
#[allow(non_snake_case)]
pub unsafe fn lua_setglobal(L: *mut lua_State, k: *const c_char) {
    unsafe { lua_setfield(L, LUA_GLOBALSINDEX, k) };
}

/// The value at `idx` as a NUL-terminated string, converting a number in
/// place; null when it is neither.
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
#[allow(non_snake_case)]
pub unsafe fn lua_tostring(L: *mut lua_State, idx: c_int) -> *const c_char {
    unsafe { lua_tolstring(L, idx, ptr::null_mut()) }
}

/// Is the value at `idx` nil?
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
#[allow(non_snake_case)]
pub unsafe fn lua_isnil(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TNIL }
}

/// Is `idx` past the top of the stack, or nil?
///
/// # Safety
/// `L` must be a live Lua state.
#[allow(non_snake_case)]
pub unsafe fn lua_isnoneornil(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) <= 0 }
}

/// Is the value at `idx` a table?
///
/// # Safety
/// `L` must be a live Lua state and `idx` a valid index.
#[allow(non_snake_case)]
pub unsafe fn lua_istable(L: *mut lua_State, idx: c_int) -> bool {
    unsafe { lua_type(L, idx) == LUA_TTABLE }
}

/// Check that argument `narg` is a string and answer it, or throw.
///
/// # Safety
/// `L` must be a live Lua state; this longjmps when the argument is wrong.
#[allow(non_snake_case)]
pub unsafe fn luaL_checkstring(L: *mut lua_State, narg: c_int) -> *const c_char {
    unsafe { luaL_checklstring(L, narg, ptr::null_mut()) }
}

/// Throw `luaL_argerror` unless `cond` holds.
///
/// # Safety
/// `L` must be a live Lua state; this longjmps when `cond` is false.
#[allow(non_snake_case)]
pub unsafe fn luaL_argcheck(L: *mut lua_State, cond: bool, narg: c_int, extramsg: *const c_char) {
    if !cond {
        unsafe { luaL_argerror(L, narg, extramsg) };
    }
}

/// Where upvalue `i` of the running C function sits.
pub const fn lua_upvalueindex(i: c_int) -> c_int {
    LUA_GLOBALSINDEX - i
}
