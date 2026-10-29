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
//! Every Lua stack operation goes through the [`Stack`], and an error is a
//! [`Raise`] the walk answers up to the binding, which raises it once its
//! frame -- and the buffers in it -- are gone.
//!
//! Ported from Lua CJSON, Copyright (c) 2010-2012 Mark Pulford, under the
//! MIT license; the notice is reproduced in licenses/lua-cjson-LICENSE.txt.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

pub mod decode;
pub mod encode;

use core::ffi::{CStr, c_int, c_void};

use crate::lua::stack::{At, Kind, Raise, Stack, Userdata};
use crate::lua_functions;

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

/// One module table's mutable state: the userdatum every function in the
/// table carries as its first upvalue.
#[derive(Default)]
pub(crate) struct Config {
    buffer: Vec<u8>,
}

impl Userdata for Config {
    /// Upstream's config metatable is anonymous, holding only `__gc`; this
    /// one is registered under a name, which is what lets the functions
    /// check that their upvalue is one.
    const METATABLE: &'static CStr = c"cjson.config";
}

impl Config {
    /// Hand the output buffer to one `encode` call, leaving the config with
    /// an empty one.
    ///
    /// Borrowing it instead would hold the config across `lua_call` and
    /// `lua_gettable`, either of which can run a metamethod that calls
    /// `encode` again on this very table. Upstream has exactly that hazard
    /// and answers interleaved garbage; moving the buffer out means the
    /// re-entrant call allocates its own and the outer one is untouched.
    fn take_buffer(&mut self) -> Vec<u8> {
        let mut buffer = core::mem::take(&mut self.buffer);
        buffer.clear();
        buffer
    }

    /// Put it back for the next call to reuse.
    fn restore_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer = buffer;
    }
}

/// The module table's config, upvalue 1 of every function in it.
pub(crate) fn fetch_config(stack: &mut Stack) -> Result<&mut Config, Raise> {
    stack
        .to_userdata::<Config>(At::upvalue(1))
        .ok_or_else(|| Raise::message("BUG: Unable to fetch CJSON configuration"))
}

lua_functions! {
    /// The module table's functions, each closing over its config.
    enum CjsonFunctions in CJSON_FUNCTIONS {
        0 => c"encode": encode::encode,
        1 => c"decode": decode::decode,
        2 => c"new": new_module,
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

/// Push `REGISTRY[key]`.
pub(crate) fn push_registry(stack: &mut Stack, key: *mut c_void) {
    stack.push_light_userdata(key);
    stack.raw_get(At::REGISTRY);
}

/// `vim.json.new()`: another module table, with its own buffer.
fn new_module(stack: &mut Stack) -> Result<c_int, Raise> {
    lua_cjson_new(stack);
    Ok(1)
}

/// Push a `cjson` module table: this is `vim.json`, and `vim.json.new()`.
pub fn lua_cjson_new(stack: &mut Stack) {
    // Upstream initialises its locale-dependent float conversions here,
    // off the main thread only once. `cjson/fpconv.rs` has no locale state,
    // so there is nothing to do — but the `nvim.thread` probe stays, because
    // reading and popping it is the stack effect the surrounding code was
    // written against.
    stack.get_field(At::REGISTRY, c"nvim.thread");
    stack.pop(1);

    // The array metatables are per-state, not per-module-table: a second
    // `new()` must not replace the ones the first put in the registry, or a
    // table tagged by one would stop being an array to the other.
    push_registry(stack, empty_array_key());
    let missing = stack.kind(At::TOP) == Kind::Nil;
    stack.pop(1);
    if missing {
        for key in [empty_array_key(), array_key()] {
            stack.push_light_userdata(key);
            stack.new_table(0, 0);
            stack.raw_set(At::REGISTRY);
        }
    }

    // .., table
    stack.new_table(0, 0);
    // .., table, config
    stack.new_userdata_metatable::<Config>();
    stack.pop(1);
    stack.new_userdata(Config {
        buffer: Vec::with_capacity(BUFFER_CAPACITY),
    });
    // Lua 5.2's `luaL_setfuncs` with `nup = 1`, which LuaJIT does not have.
    for function in CJSON_FUNCTIONS {
        // .., table, config, config
        stack.push_value(At::TOP);
        // .., table, config, closure
        stack.push_closure(function.entry(), 1);
        stack.set_field(At::from_top(3), function.name());
    }
    // .., table
    stack.pop(1);

    // Upstream also exposes `null`, `array_mt`, `empty_array_mt` and
    // `empty_array` here; nvim publishes none of them, because `vim.NIL` and
    // `vim.empty_dict()` are the spellings it wants used.
    for (field, text) in [(c"_NAME", MODULE_NAME), (c"_VERSION", MODULE_VERSION)] {
        stack.push_bytes(text.to_bytes());
        stack.set_field(At::from_top(2), field);
    }
}
