//! What a value or the running code says about itself: metafields, C
//! function identity, and the debug interface's frames and sources.

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

use core::ffi::{CStr, c_int};

use super::{At, CFunction, Stack};
use crate::lua::ffi::{
    lua_Debug, lua_getinfo, lua_getstack, lua_iscfunction, lua_tocfunction, luaL_callmeta,
    luaL_getmetafield,
};

/// One frame of the running call stack, as `lua_getinfo(L, "nSl", ..)`
/// describes it.
#[derive(Debug)]
pub struct Frame {
    /// A C function's frame, rather than Lua code's.
    pub is_c: bool,
    /// The chunk name: `@path` for a file, the code itself (or its given
    /// name) for a string.
    pub source: Vec<u8>,
    /// The line running in it; -1 when there is none.
    pub current_line: c_int,
}

impl Stack {
    /// `luaL_getmetafield`: push field `name` of the metatable of the value
    /// at `at` and answer `true`, or push nothing and answer `false` when
    /// there is no metatable or no such field.
    pub fn get_metafield(&mut self, at: At, name: &CStr) -> bool {
        unsafe { luaL_getmetafield(self.l(), at.0, name.as_ptr()) != 0 }
    }

    /// `luaL_callmeta`: call metamethod `name` of the value at `at` with the
    /// value as its argument, push its one result and answer `true`; `false`
    /// when there is no such metamethod, having pushed nothing.
    ///
    /// The call is not protected: an error in the metamethod unwinds through
    /// the caller, as `lua_call` does.
    pub fn call_meta(&mut self, at: At, name: &CStr) -> bool {
        unsafe { luaL_callmeta(self.l(), at.0, name.as_ptr()) != 0 }
    }

    /// Whether the value at `at` is the C function `function`, closure or
    /// not.
    pub fn is_c_function(&self, at: At, function: CFunction) -> bool {
        unsafe {
            lua_iscfunction(self.l(), at.0) != 0
                && lua_tocfunction(self.l(), at.0)
                    .is_some_and(|f| core::ptr::fn_addr_eq(f, function))
        }
    }

    /// Frame `level` of the running call stack, 0 being the running
    /// function; `None` past the bottom.
    pub fn frame(&mut self, level: c_int) -> Option<Frame> {
        let mut record = lua_Debug::default();
        // SAFETY: a live state and a record for it to fill; the strings it
        // answers live in the state and are copied before anything runs.
        unsafe {
            if lua_getstack(self.l(), level, &raw mut record) != 1
                || lua_getinfo(self.l(), c"nSl".as_ptr(), &raw mut record) == 0
            {
                return None;
            }
            Some(Frame {
                is_c: *record.what == b'C'.cast_signed(),
                source: CStr::from_ptr(record.source).to_bytes().to_vec(),
                current_line: record.currentline,
            })
        }
    }

    /// Pop the function on top and answer where it was defined: its chunk
    /// name and the line its definition starts on (`lua_getinfo(L, ">S")`).
    pub fn pop_function_source(&mut self) -> Option<(Vec<u8>, c_int)> {
        let mut record = lua_Debug::default();
        // SAFETY: a live state with a function on top, which `>S` consumes;
        // the source is copied before anything else runs.
        unsafe {
            if lua_getinfo(self.l(), c">S".as_ptr(), &raw mut record) == 0 {
                return None;
            }
            Some((
                CStr::from_ptr(record.source).to_bytes().to_vec(),
                record.linedefined,
            ))
        }
    }
}
