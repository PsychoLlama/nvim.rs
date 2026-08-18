//! `:trust`, the ex-command face of `vim.secure.trust`.
//!
//! # Boundary
//!
//! `ex_trust` is an ex-command handler, so it keeps the C ABI and the
//! `exarg_T *` the command table hands it. Below it everything runs on
//! the Lua stack, which stays raw.

use crate::charset::{skiptowhite, skipwhite};
use crate::lua::executor::{get_global_lstate, nlua_error, nlua_pcall};
use crate::lua::ffi::{
    lua_createtable, lua_getfield, lua_gettop, lua_pushnumber, lua_pushstring, lua_settable,
    lua_settop, lua_toboolean, lua_tolstring,
};
use crate::os::cshim::gettext;
use crate::types::exarg_T;
use core::ffi::{CStr, c_char};
use core::{ptr, slice};

/// Lua's pseudo-index for the globals table.
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002;

/// What `:trust` asks `vim.secure.trust` to do with the path.
#[derive(Clone, Copy)]
enum TrustAction {
    Allow,
    Deny,
    Remove,
}

impl TrustAction {
    /// The `action` field `vim.secure.trust` expects.
    fn name(self) -> &'static CStr {
        match self {
            TrustAction::Allow => c"allow",
            TrustAction::Deny => c"deny",
            TrustAction::Remove => c"remove",
        }
    }
}

/// Call `vim.secure.trust{ action = ..., path|bufnr = ... }` and report
/// what it says. `path` of `None` means the current buffer.
///
/// # Safety
/// The global Lua state must be initialized: main thread, editor running.
unsafe fn nlua_trust(action: TrustAction, path: Option<&CStr>) -> bool {
    let lstate = get_global_lstate();
    let top = lua_gettop(lstate);

    lua_getfield(lstate, LUA_GLOBALSINDEX, c"vim".as_ptr());
    lua_getfield(lstate, -1, c"secure".as_ptr());
    lua_getfield(lstate, -1, c"trust".as_ptr());

    lua_createtable(lstate, 0, 0);
    lua_pushstring(lstate, c"action".as_ptr());
    lua_pushstring(lstate, action.name().as_ptr());
    lua_settable(lstate, -3);
    match path {
        Some(path) => {
            lua_pushstring(lstate, c"path".as_ptr());
            lua_pushstring(lstate, path.as_ptr());
        }
        None => {
            lua_pushstring(lstate, c"bufnr".as_ptr());
            lua_pushnumber(lstate, 0.0);
        }
    }
    lua_settable(lstate, -3);

    if nlua_pcall(lstate, 1, 2) != 0 {
        nlua_error(lstate, gettext(c"vim.secure.trust: %.*s".as_ptr()));
        lua_settop(lstate, top);
        return false;
    }

    let success = lua_toboolean(lstate, -2) != 0;
    // The second result is the path it acted on, or the reason it could not.
    let msg = lua_tolstring(lstate, -1, ptr::null_mut());
    if !msg.is_null() {
        let msg = CStr::from_ptr(msg).to_string_lossy();
        match (success, action) {
            (true, TrustAction::Allow) => {
                crate::smsg!(0, "Allowed in trust database: \"{msg}\"");
            }
            (true, TrustAction::Deny) => {
                crate::smsg!(0, "Denied in trust database: \"{msg}\"");
            }
            (true, TrustAction::Remove) => {
                crate::smsg!(0, "Removed from trust database: \"{msg}\"");
            }
            (false, _) => {
                crate::semsg!("E5570: Cannot update trust file: {msg}");
            }
        }
    }

    lua_settop(lstate, top);
    success
}

/// `:trust [++deny|++remove] [path]`. Without a path it acts on the
/// current buffer.
pub unsafe fn ex_trust(eap: *mut exarg_T) {
    let arg = (*eap).arg;
    let rest = skiptowhite(arg);
    let word = slice::from_raw_parts(arg as *const u8, rest.addr() - arg.addr());
    let action = match word {
        b"++deny" => TrustAction::Deny,
        b"++remove" => TrustAction::Remove,
        b"" => TrustAction::Allow,
        _ => {
            crate::semsg!("E475: Invalid argument: {}", String::from_utf8_lossy(word));
            return;
        }
    };
    let path = skipwhite(rest);
    let path = if *path == 0 {
        None
    } else {
        Some(CStr::from_ptr(path as *const c_char))
    };
    nlua_trust(action, path);
}
