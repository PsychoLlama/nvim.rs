//! Command-line completion over Lua names.
//!
//! [`nlua_expand_pat`] hands the pattern to `vim._expand_pat` and stashes the
//! results in `EXPAND_RESULTS`, which [`nlua_expand_get_matches`] then drains
//! -- the two-step shape exists because the caller wants the matches after
//! the Lua state has been unwound.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::{GA_EMPTY_INIT_VALUE, get_global_lstate, nlua_error, nlua_pcall};
use crate::api::private::helpers::string_to_cstr;
use crate::ex_getln::ERROR_INIT;
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::lua::converter::{nlua_pop_Array, nlua_pop_Integer};
use crate::lua::ffi::{
    LUA_TFUNCTION, lua_getfield, lua_getglobal, lua_pushlstring, luaL_checktype,
};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::os::libc::gettext;
use crate::types::{
    Arena, expand_T, garray_T, kErrorTypeNone, kObjectTypeString, ptrdiff_t, size_t,
};

/// `nlua_expand_pat`'s two answers.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// How many matches the result garray grows by at a time.
const EXPAND_GROWSIZE: c_int = 80;

/// The matches [`nlua_expand_pat`] produced, waiting for
/// [`nlua_expand_get_matches`] to take ownership of them.
static EXPAND_RESULTS: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);

/// Complete `xp->xp_pattern` through `vim._expand_pat`, which answers a
/// prefix length and a list of strings.
///
/// The prefix length is how much of the pattern the matches already include,
/// so `xp_pattern` is advanced past it. Anything that goes wrong — the call,
/// the conversion, a non-string in the list, a prefix longer than the
/// pattern — leaves no matches at all.
///
/// # Safety
/// `xp` must be a live expansion context whose `xp_pattern` points into
/// `xp_line`.
pub unsafe extern "C-unwind" fn nlua_expand_pat(xp: *mut expand_T) {
    unsafe {
        let lstate = get_global_lstate();
        let mut status = FAIL;

        lua_getglobal(lstate, c"vim".as_ptr());
        lua_getfield(lstate, -1, c"_expand_pat".as_ptr());
        luaL_checktype(lstate, -1, LUA_TFUNCTION);

        let pat: *const c_char = (*xp).xp_pattern;
        debug_assert!((*xp).xp_line.add((*xp).xp_col as usize) >= pat.cast_mut());
        let patlen: ptrdiff_t = (*xp).xp_line.add((*xp).xp_col as usize).offset_from(pat);
        lua_pushlstring(lstate, pat, patlen as size_t);

        if nlua_pcall(lstate, 1, 2) != 0 {
            nlua_error(lstate, gettext(c"vim._expand_pat: %.*s".as_ptr()));
            return;
        }

        let mut err = ERROR_INIT;
        let mut arena: Arena = ARENA_EMPTY;
        let prefix_len = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err) as ptrdiff_t;
        if err.type_0 == kErrorTypeNone && prefix_len <= patlen {
            let completions = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            'cleanup_array: {
                if err.type_0 != kErrorTypeNone {
                    break 'cleanup_array;
                }
                ga_clear(EXPAND_RESULTS.ptr());
                ga_init(
                    EXPAND_RESULTS.ptr(),
                    size_of::<*mut c_char>() as c_int,
                    EXPAND_GROWSIZE,
                );
                for i in 0..completions.size {
                    let v = *completions.items.add(i);
                    if v.type_0 != kObjectTypeString {
                        break 'cleanup_array;
                    }
                    ga_grow(EXPAND_RESULTS.ptr(), 1);
                    let ga = EXPAND_RESULTS.ptr();
                    *(*ga)
                        .ga_data
                        .cast::<*mut c_char>()
                        .add((*ga).ga_len as usize) = string_to_cstr(v.data.string);
                    (*ga).ga_len += 1;
                }
                (*xp).xp_pattern = (*xp).xp_pattern.offset(prefix_len as isize);
                status = OK;
            }
            arena_mem_free(arena_finish(&raw mut arena));
        }

        if status == FAIL {
            ga_clear(EXPAND_RESULTS.ptr());
        }
    }
}

/// Hand the stashed matches to the caller, which takes ownership of both the
/// array and every string in it.
///
/// # Safety
/// Both out-parameters must be writable.
pub unsafe extern "C-unwind" fn nlua_expand_get_matches(
    num_results: *mut c_int,
    results: *mut *mut *mut c_char,
) -> c_int {
    unsafe {
        *results = (*EXPAND_RESULTS.ptr()).ga_data.cast::<*mut c_char>();
        *num_results = (*EXPAND_RESULTS.ptr()).ga_len;
        EXPAND_RESULTS.set(GA_EMPTY_INIT_VALUE);
        (*num_results > 0) as c_int
    }
}
