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
//! - [`Stack::above_top`] is a view that owns nothing yet: what the editor's
//!   `with_main_stack` lends, from inside a C function on the same frame or
//!   from no C function at all.
//! - [`Stack::above`] lends a view over the slots above one of this view's
//!   own, keeping the ones at and below it readable for as long as the
//!   lending view's borrow.
//! - A [`Dispatch`] needs neither: its entries are safe bodies over a
//!   `Stack`, and [`dispatch_entry`] is each one as a C function -- the one
//!   trampoline, instantiated per entry, which calls `entry` for it.
//!   `vim.api` is built that way.
//!
//! The raw constructors are `unsafe` because the `Stack` cannot check that
//! the pointer is a live state. Everything after them is safe.
//!
//! # Views and their base
//!
//! A `Stack` is a *view* of the frame rooted at a base slot: it owns the
//! slots above its base and only those. Every operation that drops or moves
//! a slot by index -- [`Stack::set_top`], [`Stack::pop`], [`Stack::insert`],
//! [`Stack::remove`], [`Stack::replace`], the function and arguments a
//! [`Stack::pcall`] consumes -- refuses (aborts) one at or below the base, so
//! a view can never disturb a slot a view below it owns: the slot stays, and
//! what the outer view lent from it stays anchored. An operation that
//! consumes values off the top (`set_field`, `raw_set`, `next`, `concat`,
//! ...) consumes what this view pushed; that is checked in debug builds.
//! Reading a slot below the base is allowed: it disturbs nothing.
//!
//! An index stays LuaJIT's own: positive is absolute in the running C
//! function's frame and negative counts down from the top, whichever view
//! spells it, so a raw index handed to C means what it always did.
//!
//! # Borrowed strings
//!
//! [`Stack::string`] lends a Lua string's bytes for as long as the `Stack` is
//! borrowed. Lua keeps a string alive while it sits in a stack slot, and every
//! operation that could pop or overwrite that slot takes `&mut self`, so the
//! borrow checker is what says "copy the bytes before you pop". A second
//! view over the same frame -- the editor's main-state view, reached from
//! inside a C function on that state -- has its base at the top it found, so
//! it cannot pop what the first one lent. A nested call into Lua cannot
//! either: a C function called from there gets a frame of its own.
//! [`Above::check_c_string_below`] lends for longer: for the lending view's
//! borrow, while the view above works.
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
#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/stack/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use crate::lua::executor::release_luaref;
use crate::lua::ffi::{
    LUA_GLOBALSINDEX, LUA_REGISTRYINDEX, LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TLIGHTUSERDATA, LUA_TNIL,
    LUA_TNONE, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LUA_TTHREAD, LUA_TUSERDATA, lua_checkstack,
    lua_concat, lua_createtable, lua_getfield, lua_getmetatable, lua_gettable, lua_gettop,
    lua_insert, lua_isnumber, lua_isstring, lua_next, lua_objlen, lua_pcall, lua_pushboolean,
    lua_pushcclosure, lua_pushinteger, lua_pushlightuserdata, lua_pushlstring, lua_pushnil,
    lua_pushnumber, lua_pushvalue, lua_rawequal, lua_rawget, lua_rawgeti, lua_rawset, lua_rawseti,
    lua_remove, lua_replace, lua_setfield, lua_setmetatable, lua_settable, lua_settop,
    lua_toboolean, lua_tointeger, lua_tolstring, lua_tonumber, lua_topointer, lua_type,
    lua_typename, luaL_loadbuffer, luaL_newmetatable, luaL_ref, luaL_unref,
};
use crate::lua::state::MAIN_REFS;
use crate::types::{LuaRef, lua_Integer, lua_Number, lua_State, size_t};

mod debug;
mod dispatch;
mod table;
mod userdata;

pub use self::debug::Frame;
use self::dispatch::Check;
pub(crate) use self::dispatch::dispatch_entry;
pub use self::dispatch::{Dispatch, Function, Raise};
pub use self::userdata::Userdata;

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

    /// Upvalue `n` of the running C closure, counting from 1:
    /// `lua_upvalueindex(n)`.
    pub const fn upvalue(n: c_int) -> At {
        At(LUA_GLOBALSINDEX - n)
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
///
/// The discriminants are LuaJIT's own `LUA_T*` numbers, so reading a kind
/// off the stack is a range check rather than a table lookup -- it is on
/// every step of every conversion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum Kind {
    /// The index is past the top of the stack.
    None = LUA_TNONE,
    Nil = LUA_TNIL,
    Boolean = LUA_TBOOLEAN,
    LightUserdata = LUA_TLIGHTUSERDATA,
    Number = LUA_TNUMBER,
    String = LUA_TSTRING,
    Table = LUA_TTABLE,
    Function = LUA_TFUNCTION,
    Userdata = LUA_TUSERDATA,
    Thread = LUA_TTHREAD,
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
        self as c_int
    }
}

/// A reference into the main state's registry that this value owns, made by
/// [`Stack::take_reference`]. Dropping it gives the reference back, so every
/// way out of the frame that took it releases it exactly once.
#[derive(Debug)]
pub struct OwnedLuaRef(LuaRef);

impl OwnedLuaRef {
    /// The reference, for a callee that copies what it keeps.
    pub fn get(&self) -> LuaRef {
        self.0
    }
}

impl Drop for OwnedLuaRef {
    fn drop(&mut self) {
        release_luaref(self.0);
    }
}

/// A [`Stack::pcall`] or [`Stack::load`] that failed: LuaJIT's status code.
/// The error value is on top of the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LuaFailed(pub c_int);

// ---------------------------------------------------------------------------
// The stack

/// The running C function's frame of a Lua state.
///
/// Not `Copy`, not `Send`: one per frame, on the thread that runs the state.
/// See the [module documentation](self) for what it guarantees.
pub struct Stack {
    state: NonNull<lua_State>,
    /// The absolute slot below this view's first own one: 0 for a view
    /// that owns the whole frame.
    base: c_int,
}

impl Stack {
    /// A `Stack` over the whole of `state`'s current frame.
    ///
    /// # Safety
    /// `state` is a live Lua state set up by this editor's runtime (its
    /// reference bookkeeping installed), on the thread that runs it, and it
    /// outlives the returned `Stack`; no other view over the same frame is
    /// in use while this one lives, but the ones it lends.
    pub unsafe fn from_raw(state: *mut lua_State) -> Stack {
        debug_assert!(!state.is_null(), "a Stack over no Lua state");
        Stack {
            // SAFETY: the caller's promise that `state` is live.
            state: unsafe { NonNull::new_unchecked(state) },
            base: 0,
        }
    }

    /// A `Stack` over `state`'s current frame that owns nothing yet: its
    /// base is the top it finds, so it cannot pop a slot another view over
    /// the frame already holds.
    ///
    /// # Safety
    /// As [`Stack::from_raw`], except that views over the same frame may be
    /// live: none is used until this one is gone (a closure is how the
    /// editor's main-state view keeps that order).
    pub unsafe fn above_top(state: *mut lua_State) -> Stack {
        // SAFETY: the caller's promise.
        let mut stack = unsafe { Stack::from_raw(state) };
        stack.base = stack.top().0;
        stack
    }

    /// The raw state, for the callees that still take one.
    pub fn lua_state(&self) -> *mut lua_State {
        self.state.as_ptr()
    }

    fn l(&self) -> *mut lua_State {
        self.state.as_ptr()
    }

    // -- views ------------------------------------------------------------

    /// The slot below this view's first own one, as an absolute index:
    /// `At(0)` for a view over the whole frame. [`Stack::set_top`] to it
    /// drops everything this view owns.
    pub fn base(&self) -> At {
        At(self.base)
    }

    /// A view over the slots above `at`, one of this view's own (or its
    /// base): it can push, pop and convert above `at` as freely as this
    /// one, but not drop or move `at` or anything below, and it lends the
    /// strings there for as long as this view's borrow
    /// ([`Above::check_c_string_below`]).
    pub fn above(&mut self, at: At) -> Above<'_> {
        let base = self.absolute(at).0;
        if base < self.base {
            below_base();
        }
        Above {
            stack: Stack {
                state: self.state,
                base,
            },
            _lent_by: PhantomData,
        }
    }

    /// Abort unless `at` is a slot this view may drop or move: one above
    /// its base, or a pseudo-index. Answers its LuaJIT index.
    #[inline]
    fn own(&self, at: At) -> c_int {
        if at.0 <= LUA_REGISTRYINDEX {
            return at.0;
        }
        let at = self.absolute(at).0;
        if at <= self.base {
            below_base();
        }
        at
    }

    /// How many slots this view owns, for the debug checks that a value
    /// consumed off the top is one of them.
    fn depth(&self) -> c_int {
        self.top().0 - self.base
    }

    // -- index and balance ------------------------------------------------

    /// The top slot, as an absolute index; `At(0)` when the frame is empty.
    ///
    /// `lua_gettop` read in place: every balance check asks it, and the
    /// call around two loads cost the converters 1 % (see [`StateHead`]).
    #[inline]
    pub fn top(&self) -> At {
        let head = self.l().cast::<StateHead>();
        // SAFETY: a live state (the constructor's contract), as everywhere
        // below, whose head is `StateHead` (checked when the state is
        // made); both pointers are into its one stack, `base <= top`.
        let depth = unsafe { (*head).top.offset_from((*head).base) };
        // A frame holds at most `LJ_STACK_MAX` (65,500) slots.
        #[allow(clippy::cast_possible_truncation)]
        At(depth as c_int)
    }

    /// Abort unless [`Stack::top`]'s reading of the state's head agrees
    /// with LuaJIT's own `lua_gettop`: a LuaJIT built with another
    /// `lua_State` layout (32-bit references, say) fails here, when the
    /// state is made, rather than misreading every balance check after.
    pub fn check_state_layout(&mut self) {
        self.push_nil();
        self.push_nil();
        // SAFETY: a live state.
        let theirs = unsafe { lua_gettop(self.l()) };
        let ours = self.top().0;
        // SAFETY: a live state, with the two nils on top.
        unsafe { lua_settop(self.l(), -3) };
        if ours != theirs {
            std::process::abort();
        }
    }

    /// The absolute form of `at` (a pseudo-index stays as it is).
    pub fn absolute(&self, at: At) -> At {
        if at.0 < 0 && at.0 > LUA_REGISTRYINDEX {
            At(self.top().0 + at.0 + 1)
        } else {
            at
        }
    }

    /// Make `at` the top: drop what is above it, or pad with nils. `at` is
    /// this view's base or above it.
    pub fn set_top(&mut self, at: At) {
        let at = self.absolute(at).0;
        if at < self.base {
            below_base();
        }
        unsafe { lua_settop(self.l(), at) };
    }

    /// Drop the top `n` values, all of them this view's own.
    ///
    /// `lua_settop(L, -n - 1)`'s one step, `L->top += idx + 1`, done in
    /// place: the check before it reads the same head, and the call around
    /// the store cost as much again.
    #[inline]
    pub fn pop(&mut self, n: c_int) {
        if n < 0 || self.depth() < n {
            below_base();
        }
        let head = self.l().cast::<StateHead>();
        // SAFETY: a live state whose head is `StateHead`; the `n` slots
        // below its top are this view's own, so the new top stays in the
        // frame, at or above the base.
        unsafe { (*head).top = (*head).top.sub(n.unsigned_abs() as usize) };
    }

    /// Move the top value down to `at`, shifting what was there up.
    pub fn insert(&mut self, at: At) {
        let at = self.own(at);
        unsafe { lua_insert(self.l(), at) };
    }

    /// Remove the value at `at`, shifting what was above it down.
    pub fn remove(&mut self, at: At) {
        let at = self.own(at);
        unsafe { lua_remove(self.l(), at) };
    }

    /// Pop the top value into `at`.
    pub fn replace(&mut self, at: At) {
        let at = self.own(at);
        unsafe { lua_replace(self.l(), at) };
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

    /// Whether `at` holds a string, or a number (which converts to one).
    pub fn is_string(&self, at: At) -> bool {
        unsafe { lua_isstring(self.l(), at.0) != 0 }
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

    /// The string at `at` as the C string `lua_tostring` answers: cut at its
    /// first NUL. `None` for any other kind, as [`Stack::string`].
    pub fn c_string(&self, at: At) -> Option<&CStr> {
        if self.kind(at) != Kind::String {
            return None;
        }
        let bytes = unsafe { lua_tolstring(self.l(), at.0, ptr::null_mut()) };
        // SAFETY: every Lua string is NUL-terminated, and alive while its
        // slot is, which the borrow of `self` keeps.
        Some(unsafe { CStr::from_ptr(bytes) })
    }

    /// [`Stack::to_bytes`] as the C string `lua_tostring` answers: cut at its
    /// first NUL, a number converted in place. `None` for any other kind.
    pub fn to_c_string(&mut self, at: At) -> Option<&CStr> {
        let bytes = unsafe { lua_tolstring(self.l(), at.0, ptr::null_mut()) };
        if bytes.is_null() {
            return None;
        }
        // SAFETY: every Lua string is NUL-terminated, and alive while its
        // slot is, which the borrow of `self` keeps.
        Some(unsafe { CStr::from_ptr(bytes) })
    }

    /// [`Stack::string`] with the terminating NUL every Lua string carries
    /// after its bytes, for a reader that walks it as C strings.
    pub fn string_with_nul(&self, at: At) -> Option<&[u8]> {
        if self.kind(at) != Kind::String {
            return None;
        }
        let mut len: size_t = 0;
        let bytes = unsafe { lua_tolstring(self.l(), at.0, &raw mut len) };
        // SAFETY: a Lua string is `len` bytes and a NUL, alive while its slot
        // is, which the borrow of `self` keeps.
        Some(unsafe { core::slice::from_raw_parts(bytes.cast::<u8>(), len + 1) })
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
        debug_assert!(self.depth() >= n, "consumes a slot below the base");
        unsafe { lua_concat(self.l(), n) };
    }

    // -- tables -----------------------------------------------------------

    /// Push `table[key]` for the table at `at`, metamethods included.
    pub fn get_field(&mut self, at: At, key: &CStr) {
        unsafe { lua_getfield(self.l(), at.0, key.as_ptr()) };
    }

    /// Pop a value into `table[key]` for the table at `at`.
    pub fn set_field(&mut self, at: At, key: &CStr) {
        debug_assert!(self.depth() >= 1, "consumes a slot below the base");
        unsafe { lua_setfield(self.l(), at.0, key.as_ptr()) };
    }

    /// Pop a key and push `table[key]`, metamethods included.
    pub fn get_table(&mut self, at: At) {
        debug_assert!(self.depth() >= 1, "consumes a slot below the base");
        unsafe { lua_gettable(self.l(), at.0) };
    }

    /// Pop a value and a key and set `table[key] = value`, metamethods
    /// included.
    pub fn set_table(&mut self, at: At) {
        debug_assert!(self.depth() >= 2, "consumes a slot below the base");
        unsafe { lua_settable(self.l(), at.0) };
    }

    /// [`Stack::get_table`] without metamethods.
    pub fn raw_get(&mut self, at: At) {
        debug_assert!(self.depth() >= 1, "consumes a slot below the base");
        unsafe { lua_rawget(self.l(), at.0) };
    }

    /// [`Stack::set_table`] without metamethods.
    pub fn raw_set(&mut self, at: At) {
        debug_assert!(self.depth() >= 2, "consumes a slot below the base");
        unsafe { lua_rawset(self.l(), at.0) };
    }

    /// Push `table[n]`, without metamethods.
    pub fn raw_get_index(&mut self, at: At, n: c_int) {
        unsafe { lua_rawgeti(self.l(), at.0, n) };
    }

    /// Pop a value into `table[n]`, without metamethods.
    pub fn raw_set_index(&mut self, at: At, n: c_int) {
        debug_assert!(self.depth() >= 1, "consumes a slot below the base");
        unsafe { lua_rawseti(self.l(), at.0, n) };
    }

    /// Push the metatable of the value at `at` and answer `true`, or push
    /// nothing and answer `false` when it has none.
    pub fn get_metatable(&mut self, at: At) -> bool {
        unsafe { lua_getmetatable(self.l(), at.0) != 0 }
    }

    /// Pop a table and make it the metatable of the value at `at`.
    pub fn set_metatable(&mut self, at: At) {
        debug_assert!(self.depth() >= 1, "consumes a slot below the base");
        unsafe { lua_setmetatable(self.l(), at.0) };
    }

    /// One step of a table walk: pop a key (nil to start) and push the next
    /// key and its value, answering `true`; `false` once the table at `at`
    /// is exhausted, having pushed nothing.
    ///
    /// The key left below the value is what the next step pops, so it must
    /// not be converted in place in between — [`Stack::string`] never does.
    pub fn next(&mut self, at: At) -> bool {
        debug_assert!(self.depth() >= 1, "consumes a slot below the base");
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
    ///
    /// Only a *positive* reference counts: `LUA_REFNIL` and `LUA_NOREF` are
    /// sentinels, not registry entries.
    pub fn reference(&mut self, at: At) -> LuaRef {
        self.push_value(at);
        let reference = unsafe { luaL_ref(self.l(), LUA_REGISTRYINDEX) };
        if reference > 0 {
            MAIN_REFS.with_mut(|refs| refs.ref_count += 1);
        }
        reference
    }

    /// Give `reference` back. A sentinel is a no-op.
    pub fn unreference(&mut self, reference: LuaRef) {
        if reference > 0 {
            MAIN_REFS.with_mut(|refs| refs.ref_count -= 1);
            unsafe { luaL_unref(self.l(), LUA_REGISTRYINDEX, reference) };
        }
    }

    /// Pop the value on top as a reference the result owns: dropping it
    /// gives the reference back.
    pub fn take_reference(&mut self) -> OwnedLuaRef {
        let reference = self.reference(At::TOP);
        self.pop(1);
        OwnedLuaRef(reference)
    }

    /// Push `vim.NIL`, the userdata Lua code sees for a Vimscript or API nil.
    pub fn push_vim_nil(&mut self) {
        let reference = MAIN_REFS.get_field(|refs| &refs.nil_ref);
        self.push_reference(reference);
    }

    /// Push the metatable `vim.empty_dict()` marks an empty dictionary with.
    pub fn push_empty_dict_metatable(&mut self) {
        let reference = MAIN_REFS.get_field(|refs| &refs.empty_dict_ref);
        self.push_reference(reference);
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
        // The function and its arguments are this view's own.
        if self.depth() <= args {
            below_base();
        }
        match unsafe { lua_pcall(self.l(), args, results, handler) } {
            0 => Ok(()),
            status => Err(LuaFailed(status)),
        }
    }

    /// Compile `chunk` and push it as a function, or push the error message.
    /// `name` is the chunk name its messages carry; with none, LuaJIT names
    /// it after the text (or a bytecode chunk keeps its own).
    pub fn load(&mut self, chunk: &[u8], name: Option<&CStr>) -> Result<(), LuaFailed> {
        let text = chunk.as_ptr().cast::<c_char>();
        let name = name.map_or(ptr::null(), CStr::as_ptr);
        match unsafe { luaL_loadbuffer(self.l(), text, chunk.len(), name) } {
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
        let mut len: size_t = 0;
        let bytes = unsafe { lua_tolstring(self.l(), at.0, &raw mut len) };
        if bytes.is_null() {
            return Err(Raise::check(at, Check::String));
        }
        // SAFETY: as in `to_bytes`; one call, where asking twice (to keep
        // the refusal clear of the borrow) cost a vim.fn call measurably.
        Ok(unsafe { core::slice::from_raw_parts(bytes.cast::<u8>(), len) })
    }

    /// `luaL_checktype`.
    pub fn check_kind(&self, at: At, kind: Kind) -> Result<(), Raise> {
        if self.kind(at) != kind {
            return Err(Raise::check(at, Check::Kind(kind)));
        }
        Ok(())
    }
}

/// The head of LuaJIT's `struct lua_State` (`lj_obj.h`, 64-bit GC
/// references: x86-64 and arm64 builds), as far as `top`: [`Stack::top`]
/// reads the depth of the running frame from it, as `lua_gettop` does.
#[repr(C)]
struct StateHead {
    _nextgc: u64,
    _marked: u8,
    _gct: u8,
    _dummy_ffid: u8,
    _status: u8,
    _glref: u64,
    _gclist: u64,
    /// The running function's first slot.
    base: *const u64,
    /// The first free slot.
    top: *const u64,
}

/// A view over the slots above one of another view's own, lent by
/// [`Stack::above`]: a [`Stack`] whose base is that slot, for as long as the
/// lending view's borrow.
pub struct Above<'a> {
    stack: Stack,
    _lent_by: PhantomData<&'a mut Stack>,
}

impl<'a> Above<'a> {
    /// `luaL_checklstring` on `at`, a slot at or below this view's base:
    /// the string as the C string `lua_tostring` answers (cut at its first
    /// NUL) and its full length, a number converted in place. Lent for as
    /// long as the lending view's borrow, which no view above can end: the
    /// slot is not theirs to drop.
    pub fn check_c_string_below(&self, at: At) -> Result<(&'a CStr, usize), Raise> {
        if at.0 <= 0 || at.0 > self.stack.base {
            below_base();
        }
        let mut len: size_t = 0;
        let bytes = unsafe { lua_tolstring(self.stack.l(), at.0, &raw mut len) };
        if bytes.is_null() {
            return Err(Raise::check(at, Check::String));
        }
        // SAFETY: every Lua string is NUL-terminated, and alive while its
        // slot is; the slot is at or below the base, which no view above
        // may drop, and the lending view is borrowed for `'a`.
        Ok((unsafe { CStr::from_ptr(bytes) }, len))
    }
}

impl<'a> Above<'a> {
    /// `luaL_checklstring` on `at`, a slot at or below this view's base, as
    /// bytes: [`Above::check_c_string_below`] without the cut at a NUL.
    pub fn check_bytes_below(&self, at: At) -> Result<&'a [u8], Raise> {
        let (string, len) = self.check_c_string_below(at)?;
        // SAFETY: a Lua string's `len` bytes from its start, alive as long
        // as the C string just lent is.
        Ok(unsafe { core::slice::from_raw_parts(string.as_ptr().cast::<u8>(), len) })
    }
}

impl core::ops::Deref for Above<'_> {
    type Target = Stack;
    fn deref(&self) -> &Stack {
        &self.stack
    }
}

impl core::ops::DerefMut for Above<'_> {
    fn deref_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}

/// A view tried to drop or move a slot at or below its base: one an outer
/// view owns, and may have lent out. Nothing safe can follow.
#[cold]
#[inline(never)]
#[track_caller]
fn below_base() -> ! {
    let caller = core::panic::Location::caller();
    // The message is all an abort can leave behind.
    let _ = std::io::Write::write_all(
        &mut std::io::stderr(),
        format!("{caller}: a Lua stack view reached below its base\n").as_bytes(),
    );
    std::process::abort()
}

/// A size as a `lua_createtable` hint, which is only a hint: past `c_int` it
/// asks for the most it can.
fn saturate(n: usize) -> c_int {
    c_int::try_from(n).unwrap_or(c_int::MAX)
}
