//! `nlua_push_typval()`: a Vimscript value as a Lua one.
//!
//! One [`TypvalSink`] over a `lua_State`, replacing the `TYPVAL_ENCODE_NAME
//! lua` instantiation of `typval_encode.c.h`.  Every hook pushes exactly one
//! Lua value, and the containers are assembled on the Lua stack itself rather
//! than in a buffer of the sink's own: a list holds its table *and* the index
//! it is about to write, a dictionary its table *and* the pending key, so each
//! open container occupies **two** Lua slots.  That is what
//! [`LuaSink::backref`]'s `* 2` counts.
//!
//! Because the stack is Lua's, this is the one sink that can fail for want of
//! room: `lua_checkstack` is asked before every container and `E5102` is the
//! refusal.
//!
//! It reads `{_TYPE, _VAL}` special dictionaries (`ALLOW_SPECIALS`), so a value
//! that came from msgpack keeps its identity through Vimscript and back.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};

use super::{FC_LUAREF, nlua_create_typed_table};
use crate::eval::typval_encode::{ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::eval::userfunc::find_func;
use crate::lua::executor::nlua_pushref;
use crate::lua::ffi::{
    lua_checkstack, lua_createtable, lua_gettop, lua_pushboolean, lua_pushlstring, lua_pushnil,
    lua_pushnumber, lua_pushvalue, lua_rawset, lua_setmetatable, lua_tonumber,
};
use crate::main::nlua_global_refs;
use crate::types::{
    LuaRef, blob_T, dict_T, float_T, int64_t, kObjectTypeDict, lua_Number, lua_State, size_t,
    typval_T,
};

/// How many Lua slots opening a container needs: its table, the key or index
/// it is about to set, and the value that will land on top of them.
const CONTAINER_SLOTS: c_int = 3;

/// The `nlua_push_typval()` sink.
struct LuaSink {
    lstate: *mut lua_State,
    /// `kNluaPushSpecial`: push `nil` and a typed table for Vimscript's `null`
    /// and empty dictionary, rather than the `vim.NIL` and `vim.empty_dict()`
    /// singletons Lua code normally sees.
    special: bool,
}

/// The handful of Lua stack operations the sink is made of.
///
/// Each is one line of `lua_*` against [`LuaSink::lstate`], which the sink
/// holds live for as long as it exists — so these are safe, and every hook
/// below is then a safe one-liner in the shape of the macro it replaces.
impl LuaSink {
    fn createtable(&mut self, narr: c_int, nrec: c_int) {
        unsafe { lua_createtable(self.lstate, narr, nrec) };
    }

    fn pushnumber(&mut self, num: lua_Number) {
        unsafe { lua_pushnumber(self.lstate, num) };
    }

    fn pushboolean(&mut self, b: bool) {
        unsafe { lua_pushboolean(self.lstate, b as c_int) };
    }

    fn pushref(&mut self, ref_: LuaRef) {
        unsafe { nlua_pushref(self.lstate, ref_) };
    }

    /// Store the value on top of the stack under the key below it, in the
    /// table below that.
    fn rawset(&mut self) {
        unsafe { lua_rawset(self.lstate, -3) };
    }

    /// Push whatever stands for "no value": upstream's `TYPVAL_ENCODE_CONV_NIL`.
    fn push_nil(&mut self) {
        if self.special {
            unsafe { lua_pushnil(self.lstate) };
        } else {
            self.pushref(unsafe { (*nlua_global_refs.get()).nil_ref });
        }
    }

    /// Push `len` bytes as a Lua string.
    ///
    /// # Safety
    /// `buf` must point at `len` readable bytes, or `len` be zero.
    unsafe fn pushlstring(&mut self, buf: *const c_char, len: size_t) {
        unsafe { lua_pushlstring(self.lstate, buf, len) };
    }

    /// Make room for one more open container, or refuse with `E5102`.
    fn check_stack(&mut self) -> Flow {
        unsafe {
            let wanted = lua_gettop(self.lstate) + CONTAINER_SLOTS;
            if lua_checkstack(self.lstate, wanted) == 0 {
                semsg!("E5102: Lua failed to grow stack to {}", wanted);
                return Flow::Fail;
            }
        }
        Flow::Go
    }

    /// Where on the Lua stack the container being re-entered is already being
    /// built, as an index from the top.
    ///
    /// Every enclosing container occupies two slots, so the frame `n` levels
    /// down from the top sits at `-2n`.  Unlike the text sinks' marker search,
    /// this one *does* match a `Pairs` frame: upstream's ternary here reads
    /// `data.l.list` for everything that is not `kMPConvDict`, and a special
    /// map's `_VAL` list lives in that member.
    ///
    /// `None` — no frame matched — cannot happen for a container the walk has
    /// marked with the current `copyID`, and upstream pushes nothing at all in
    /// that case, leaving the value it promised missing.
    fn backref(path: &ConvPath, val: *mut c_void, conv_type: ConvType) -> Option<c_int> {
        let depth = path.stack.len();
        // Upstream scans from the top down; a container can be on the stack
        // only once, so scanning up finds the same frame.
        let found = path
            .stack
            .iter()
            .position(|frame| frame.container() == Some((conv_type, val.cast_const())))?;
        Some(-(((depth - found) * 2) as c_int))
    }
}

impl TypvalSink for LuaSink {
    const ALLOW_SPECIALS: bool = true;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_lua_convert_one_value()";

    unsafe fn conv_nil(&mut self, _tv: *mut typval_T) {
        self.push_nil();
    }

    unsafe fn conv_bool(&mut self, _tv: *mut typval_T, num: bool) {
        self.pushboolean(num);
    }

    unsafe fn conv_number(&mut self, _tv: *mut typval_T, num: int64_t) {
        self.pushnumber(num as lua_Number);
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: *mut typval_T, num: u64) {
        self.pushnumber(num as lua_Number);
    }

    unsafe fn conv_float(&mut self, _tv: *mut typval_T, flt: float_T) -> Flow {
        self.pushnumber(flt);
        Flow::Go
    }

    /// A Lua string is bytes, so this is the whole of it — NULs included.  It
    /// copies, which is why the walk's buffer-owning hooks need no override.
    unsafe fn conv_string(&mut self, _tv: *mut typval_T, buf: *mut c_char, len: size_t) -> Flow {
        unsafe { self.pushlstring(buf, len) };
        Flow::Go
    }

    /// msgpack `ext` has no Lua image, so it comes out as nil.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: *mut typval_T,
        _buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        self.push_nil();
        Flow::Go
    }

    unsafe fn conv_blob(&mut self, _tv: *mut typval_T, blob: *const blob_T, len: c_int) {
        unsafe {
            let data = if blob.is_null() {
                c"".as_ptr()
            } else {
                (*blob).bv_ga.ga_data.cast::<c_char>()
            };
            self.pushlstring(data, len as size_t);
        }
    }

    /// A Lua function that reached Vimscript as a funcref goes back as the
    /// same function; anything else is nil.  Either way the walk stops here,
    /// so a partial's arguments and self dictionary are never visited — which
    /// is why no `Partial` frame ever reaches [`LuaSink::backref`].
    unsafe fn conv_func_start(
        &mut self,
        _tv: *mut typval_T,
        fun: *mut c_char,
        _prefix: &'static CStr,
        _path: &ConvPath,
    ) -> Flow {
        let luaref = unsafe {
            let fp = if fun.is_null() {
                ::core::ptr::null_mut()
            } else {
                find_func(fun)
            };
            if fp.is_null() || (*fp).uf_flags & FC_LUAREF == 0 {
                None
            } else {
                Some((*fp).uf_luaref)
            }
        };
        match luaref {
            Some(ref_) => self.pushref(ref_),
            None => self.push_nil(),
        }
        Flow::Stop
    }

    unsafe fn conv_empty_list(&mut self, _tv: *mut typval_T) {
        self.createtable(0, 0);
    }

    /// An empty table is ambiguous in Lua, so an empty dictionary carries a
    /// marker: the `vim.empty_dict()` metatable, or the `_TYPE` key when the
    /// caller asked for the special form.
    unsafe fn conv_empty_dict(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        if self.special {
            unsafe { nlua_create_typed_table(self.lstate, 0, 0, kObjectTypeDict) };
        } else {
            self.createtable(0, 0);
            self.pushref(unsafe { (*nlua_global_refs.get()).empty_dict_ref });
            unsafe { lua_setmetatable(self.lstate, -2) };
        }
    }

    /// The table, then the index its first item will be stored under.
    unsafe fn conv_list_start(&mut self, _tv: *mut typval_T, len: c_int) -> Flow {
        if self.check_stack() == Flow::Fail {
            return Flow::Fail;
        }
        self.createtable(len, 0);
        self.pushnumber(1.0);
        Flow::Go
    }

    /// Store the item just converted and push the next index.
    unsafe fn conv_list_between_items(&mut self, _tv: *mut typval_T) {
        let idx = unsafe { lua_tonumber(self.lstate, -2) };
        self.rawset();
        self.pushnumber(idx + 1.0);
    }

    unsafe fn conv_list_end(&mut self, _tv: *mut typval_T) {
        self.rawset();
    }

    unsafe fn conv_dict_start(&mut self, _tv: *mut typval_T, len: size_t) -> Flow {
        if self.check_stack() == Flow::Fail {
            return Flow::Fail;
        }
        self.createtable(0, len as c_int);
        Flow::Go
    }

    /// The key is already on the stack and the value has just landed on top of
    /// it, so one `rawset` closes the pair.
    unsafe fn conv_dict_between_items(
        &mut self,
        _tv: *mut typval_T,
        _dictp: Option<*mut *mut dict_T>,
    ) {
        self.rawset();
    }

    unsafe fn conv_dict_end(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        self.rawset();
    }

    /// Lua tables are references, so a container that references itself is not
    /// a problem here: push the half-built table again and the cycle rebuilds
    /// itself.
    unsafe fn conv_recurse(
        &mut self,
        val: *mut c_void,
        conv_type: ConvType,
        path: &ConvPath,
    ) -> Flow {
        if let Some(idx) = Self::backref(path, val, conv_type) {
            unsafe { lua_pushvalue(self.lstate, idx) };
        }
        Flow::Go
    }
}

/// Convert a Vimscript value and leave it on the Lua stack.
///
/// `flags` is `kNluaPushSpecial` or zero; the `kNluaPushFreeRefs` bit is the
/// caller's business, not this walk's.
///
/// # Safety
/// `lstate` must be a live Lua state and `tv` a live typval.
pub unsafe fn nlua_push_typval(lstate: *mut lua_State, tv: *mut typval_T, flags: c_int) -> bool {
    unsafe {
        let initial_size = lua_gettop(lstate);
        if lua_checkstack(lstate, initial_size + 2) == 0 {
            // Upstream reports the size it would have needed for a container,
            // not the two slots it just asked for.
            semsg!("E1502: Lua failed to grow stack to {}", initial_size + 4);
            return false;
        }
        let mut sink = LuaSink {
            lstate,
            special: flags & super::kNluaPushSpecial as c_int != 0,
        };
        if !encode_typval(&mut sink, tv, c"nlua_push_typval argument".as_ptr()) {
            return false;
        }
        debug_assert!(lua_gettop(lstate) == initial_size + 1);
        true
    }
}
