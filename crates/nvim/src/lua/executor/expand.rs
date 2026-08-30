//! Command-line completion over Lua names.
//!
//! [`nlua_expand_pat`] hands the pattern to `vim._expand_pat` and stashes the
//! results in `EXPAND_RESULTS`, which [`nlua_expand_get_matches`] then drains
//! -- the two-step shape exists because the caller wants the matches after
//! the Lua state has been unwound.
//!
//! The stash is a `Vec` of owned C strings, not a `garray_T` whose raw buffer
//! is handed over: only the *strings* have to be `xmalloc`ed for the caller's
//! `free_wild`, and the array they travel in is built at handoff.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::{get_global_lstate, nlua_error, nlua_pcall};
use crate::api::private::helpers::string_to_cstr;
use crate::ex_getln::ERROR_INIT;
use crate::global_cell::GlobalCell;
use crate::lua::converter::{nlua_pop_array, nlua_pop_integer};
use crate::lua::ffi::{
    LUA_TFUNCTION, lua_getfield, lua_getglobal, lua_pushlstring, luaL_checktype,
};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, xfree, xmalloc};
use crate::os::cshim::gettext;
use crate::types::{Arena, FAIL, OK, expand_T, kObjectTypeString, ptrdiff_t, size_t};

/// The matches [`nlua_expand_pat`] produced, waiting for
/// [`nlua_expand_get_matches`] to take ownership of them. Each entry is an
/// owned NUL-terminated string.
static EXPAND_RESULTS: GlobalCell<Vec<*mut c_char>> = GlobalCell::new(Vec::new());

/// Free a run of matches nobody took over.
///
/// Upstream spells this `ga_clear`, which frees the array and *leaks* every
/// string in it -- once per completion that fails partway, and once more for
/// every result that is produced but never drained.
fn free_matches(matches: Vec<*mut c_char>) {
    for s in matches {
        // SAFETY: every entry is a `string_to_cstr` allocation this module
        // owns until it hands the run over.
        unsafe { xfree(s.cast()) };
    }
}

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
pub unsafe fn nlua_expand_pat(xp: *mut expand_T) {
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
            nlua_error(lstate, gettext(c"vim._expand_pat: %.*s").as_ptr());
            return;
        }

        let mut err = ERROR_INIT;
        let mut arena: Arena = ARENA_EMPTY;
        let prefix_len = nlua_pop_integer(lstate, &raw mut arena, &raw mut err) as ptrdiff_t;
        let mut matches: Vec<*mut c_char> = Vec::new();
        if !err.is_set() && prefix_len <= patlen {
            let completions = nlua_pop_array(lstate, &raw mut arena, &raw mut err);
            'cleanup_array: {
                if err.is_set() {
                    break 'cleanup_array;
                }
                matches.reserve(completions.size);
                for i in 0..completions.size {
                    let v = *completions.items.add(i);
                    if v.type_0 != kObjectTypeString {
                        break 'cleanup_array;
                    }
                    matches.push(string_to_cstr(v.data.string));
                }
                (*xp).xp_pattern = (*xp).xp_pattern.offset(prefix_len as isize);
                status = OK;
            }
            arena_mem_free(arena_finish(&raw mut arena));
        }

        // Whatever a previous run left undrained goes here, as upstream's
        // first `ga_clear` does; a failed run's partial matches go with it.
        free_matches(EXPAND_RESULTS.take());
        if status == OK {
            EXPAND_RESULTS.set(matches);
        } else {
            free_matches(matches);
        }
    }
}

/// Hand the stashed matches to the caller, which takes ownership of both the
/// array and every string in it.
///
/// # Safety
/// Both out-parameters must be writable.
pub unsafe fn nlua_expand_get_matches(
    num_results: *mut c_int,
    results: *mut *mut *mut c_char,
) -> c_int {
    let matches = EXPAND_RESULTS.take();
    let count = matches.len();
    // The caller frees the array with `free_wild`, so it has to be one
    // `xmalloc` block; upstream hands over the growarray's own buffer, which
    // only works while the crate allocates through `malloc`.
    // SAFETY: the out-parameters are the caller's, and `xmalloc` answers
    // `count` writable pointers or does not return.
    unsafe {
        *results = if count == 0 {
            ptr::null_mut()
        } else {
            let array = xmalloc(count * size_of::<*mut c_char>()).cast::<*mut c_char>();
            ptr::copy_nonoverlapping(matches.as_ptr(), array, count);
            array
        };
        *num_results = count as c_int;
    }
    (count > 0) as c_int
}
