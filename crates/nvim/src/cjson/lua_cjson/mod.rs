//! `lua_cjson.c`: the `vim.json` Lua module.
//!
//! Vendored Lua CJSON 2.1.0.11. Two functions — [`encode::encode`] and
//! [`decode::decode`] — plus `new`, which hands back another module table
//! with its own buffer.
//!
//! **Upstream is configurable and nvim's copy is not.** Every
//! `cjson.encode_*`/`decode_*` setter is commented out in the vendored
//! source ("don't expose options which cause global side-effects"), so the
//! fourteen `json_config_t` settings can never be anything but their
//! defaults. They are the consts below, and the arms only a non-default
//! setting could reach are gone rather than written out unreachable — each
//! one is called out where it was. What *is* still configurable is the
//! second argument to `encode`/`decode`, a per-call options table:
//! `escape_slash`, `indent` and `sort_keys` on the way out, `luanil` and
//! `skip_comments` on the way in.
//!
//! The one piece of state a module table carries is [`Config`]'s buffer,
//! the encoder's output, kept between calls (`encode_keep_buffer`) and
//! collected with the table. It is *moved out* of the config for the
//! duration of a call rather than borrowed, so a metamethod that re-enters
//! `encode` on the same module table gets a fresh buffer instead of
//! scribbling over its caller's — see [`Config::take_buffer`].
//!
//! Ported from Lua CJSON, Copyright (c) 2010-2012 Mark Pulford, under the
//! MIT license; the notice is reproduced in licenses/lua-cjson-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod decode;
pub mod encode;

use core::ffi::{CStr, c_int, c_void};

use crate::lua::ffi::{
    LUA_REGISTRYINDEX, LUA_TNIL, lua_createtable, lua_getfield, lua_newtable, lua_newuserdata,
    lua_pop, lua_pushcclosure, lua_pushcfunction, lua_pushlightuserdata, lua_pushlstring,
    lua_pushvalue, lua_rawget, lua_rawset, lua_setfield, lua_setmetatable, lua_touserdata,
    lua_type, lua_upvalueindex, luaL_checkstack,
};
use crate::types::{lua_CFunction, lua_State};

/// Reported as `vim.json._NAME` / `._VERSION`.
const MODULE_NAME: &CStr = c"cjson";
const MODULE_VERSION: &CStr = c"2.1.0.11";

/// A gap this many times wider than the number of entries makes a Lua table
/// an object rather than an array — unless the largest index is within
/// [`SPARSE_SAFE`], where a gap is always still an array. Upstream's
/// `encode_sparse_convert` would turn such a table into an object; here it
/// is off, so the encoder raises instead.
pub(crate) const SPARSE_RATIO: i64 = 2;
pub(crate) const SPARSE_SAFE: i64 = 10;
/// Nesting limits. Both are also bounded by `lua_checkstack`.
pub(crate) const ENCODE_MAX_DEPTH: c_int = 1000;
pub(crate) const DECODE_MAX_DEPTH: c_int = 1000;
/// Significant digits per number, upstream's `encode_number_precision`.
pub(crate) const NUMBER_PRECISION: u32 = 16;
/// What the reused encode buffer starts at.
const BUFFER_CAPACITY: usize = 1023;

/// One module table's mutable state.
///
/// Lua owns the memory — it is the userdata every registered function
/// carries as its first upvalue, and `__gc` drops it — which is the only
/// reason the `Vec` inside is allowed to live here.
pub(crate) struct Config {
    buffer: Vec<u8>,
}

impl Config {
    /// Hand the output buffer to one `encode` call, leaving the config with
    /// an empty one.
    ///
    /// Borrowing it instead would be a `&mut` held live across `lua_call`
    /// and `lua_gettable`, either of which can run a metamethod that calls
    /// `encode` again on this very table. Upstream has exactly that hazard
    /// and answers interleaved garbage; moving the buffer out means the
    /// re-entrant call allocates its own and the outer one is untouched.
    fn take_buffer(&mut self) -> Vec<u8> {
        let mut buffer = core::mem::take(&mut self.buffer);
        buffer.clear();
        buffer
    }

    /// Put it back for the next call to reuse. Skipped when the call
    /// raised, in which case the buffer is dropped and the next call
    /// allocates a new one.
    fn restore_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer = buffer;
    }
}

/// The module table's config userdata, upvalue 1 of every function in it.
///
/// # Safety
/// Only callable from a function [`set_functions`] registered, and the
/// pointer is only live until Lua collects the module table.
pub(crate) unsafe fn fetch_config(l: *mut lua_State) -> *mut Config {
    unsafe { lua_touserdata(l, lua_upvalueindex(1)) }.cast::<Config>()
}

/// `__gc` for the config userdata.
unsafe extern "C-unwind" fn destroy_config(l: *mut lua_State) -> c_int {
    let cfg = unsafe { lua_touserdata(l, 1) }.cast::<Config>();
    if !cfg.is_null() {
        // SAFETY: `__gc` runs once, on a userdatum this module built and
        // initialised, and nothing reads it afterwards.
        unsafe { core::ptr::drop_in_place(cfg) };
    }
    0
}

/// Push a fresh config userdatum, with its `__gc`, onto the stack.
unsafe fn create_config(l: *mut lua_State) {
    // SAFETY: `lua_newuserdata` answers uninitialised memory of exactly the
    // size asked for, aligned for any type, and raises rather than
    // answering null. `write` initialises it before anything can observe
    // it, and the metatable is installed after, so `__gc` cannot see a
    // half-built value.
    unsafe {
        let cfg = lua_newuserdata(l, size_of::<Config>()).cast::<Config>();
        cfg.write(Config {
            buffer: Vec::with_capacity(BUFFER_CAPACITY),
        });

        lua_newtable(l);
        lua_pushcfunction(l, destroy_config);
        lua_setfield(l, -2, c"__gc".as_ptr());
        lua_setmetatable(l, -2);
    }
}

/// Two addresses used as light-userdata keys in the Lua registry, where the
/// array and empty-array metatables live.
///
/// One array rather than two statics on purpose: two `static u8`s holding
/// the same value may be merged into one symbol by the linker, which would
/// make the two keys equal. Members of one array cannot be.
static REGISTRY_KEYS: [u8; 2] = [0, 0];

/// The registry key for the metatable marking a table as a JSON array.
pub(crate) fn array_key() -> *mut c_void {
    mask_key(&raw const REGISTRY_KEYS[0])
}

/// The registry key for the metatable marking `{}` as an empty array.
pub(crate) fn empty_array_key() -> *mut c_void {
    mask_key(&raw const REGISTRY_KEYS[1])
}

/// Upstream's `json_lightudata_mask`: LuaJIT stores light userdata in 47
/// bits, so an address with anything above bit 46 set could not round-trip.
/// Nothing ever dereferences the result — the address is only ever a key —
/// so the truncation is harmless as long as it is applied consistently.
fn mask_key(address: *const u8) -> *mut c_void {
    let masked = address.expose_provenance() & ((1usize << 47) - 1);
    core::ptr::with_exposed_provenance_mut(masked)
}

/// Fetch `REGISTRY[key]`, leaving it on the stack.
pub(crate) unsafe fn push_registry(l: *mut lua_State, key: *mut c_void) {
    unsafe {
        lua_pushlightuserdata(l, key);
        lua_rawget(l, LUA_REGISTRYINDEX);
    }
}

/// Register `functions` into the table below the top of the stack, each
/// closing over the config userdatum on top of it.
///
/// This is Lua 5.2's `luaL_setfuncs` with `nup = 1`, which LuaJIT does not
/// have. On entry the stack is `.., table, config`; on exit, `.., table`.
unsafe fn set_functions(l: *mut lua_State, functions: &[(&CStr, lua_CFunction)]) {
    unsafe {
        luaL_checkstack(l, 1, c"too many upvalues".as_ptr());
        for (name, function) in functions {
            // .., table, config, config
            lua_pushvalue(l, -1);
            // .., table, config, closure
            lua_pushcclosure(l, *function, 1);
            lua_setfield(l, -3, name.as_ptr());
        }
        lua_pop(l, 1);
    }
}

/// Build a `cjson` module table: this is `vim.json`, and `vim.json.new()`.
///
/// # Safety
/// `l` must be a live Lua state.
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn lua_cjson_new(l: *mut lua_State) -> c_int {
    unsafe {
        // Upstream initialises its locale-dependent float conversions here,
        // off the main thread only once. `cjson/fpconv.rs` has no locale
        // state, so there is nothing to do — but the `nvim.thread` probe
        // stays, because reading and popping it is the stack effect the
        // surrounding code was written against.
        lua_getfield(l, LUA_REGISTRYINDEX, c"nvim.thread".as_ptr());
        lua_pop(l, 1);

        // The array metatables are per-state, not per-module-table: a
        // second `new()` must not replace the ones the first put in the
        // registry, or a table tagged by one would stop being an array to
        // the other.
        push_registry(l, empty_array_key());
        let missing = lua_type(l, -1) == LUA_TNIL;
        lua_pop(l, 1);
        if missing {
            for key in [empty_array_key(), array_key()] {
                lua_pushlightuserdata(l, key);
                lua_createtable(l, 0, 0);
                lua_rawset(l, LUA_REGISTRYINDEX);
            }
        }

        // .., table
        lua_createtable(l, 0, 0);
        // .., table, config
        create_config(l);
        set_functions(
            l,
            &[
                (c"encode", Some(encode::encode)),
                (c"decode", Some(decode::decode)),
                (c"new", Some(lua_cjson_new)),
            ],
        );

        // Upstream also exposes `null`, `array_mt`, `empty_array_mt` and
        // `empty_array` here; nvim publishes none of them, because `vim.NIL`
        // and `vim.empty_dict()` are the spellings it wants used.
        for (field, text) in [(c"_NAME", MODULE_NAME), (c"_VERSION", MODULE_VERSION)] {
            lua_pushlstring(l, text.as_ptr(), text.count_bytes());
            lua_setfield(l, -2, field.as_ptr());
        }
    }
    1
}

/// Say that a `luaL_error` call cannot come back.
///
/// `luaL_error` longjmps (or unwinds) out of the calling C function, but its
/// declared return type is `c_int`, so a caller whose own type is `!` needs
/// somewhere to go. Nothing below this frame is unwound, so no Rust value
/// that owns memory may be live at the call — every caller here drops its
/// buffers first, exactly where the C calls `free`.
pub(crate) fn unreachable_after_raise() -> ! {
    unreachable!("luaL_error returned")
}
