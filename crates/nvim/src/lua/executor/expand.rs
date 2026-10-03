//! Command-line completion over Lua names.
//!
//! [`nlua_expand_pat`] hands the pattern to `vim._expand_pat` and stashes the
//! results in `EXPAND_RESULTS`, which [`nlua_expand_matches`] then drains
//! -- the two-step shape exists because the caller wants the matches after
//! the Lua state has been unwound.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use super::{get_global_lstate, nlua_error, nlua_pcall};
use crate::global_cell::GlobalCell;
use crate::lua::converter::{nlua_pop_array, nlua_pop_integer};
use crate::lua::ffi::{
    LUA_TFUNCTION, lua_getfield, lua_getglobal, lua_pushlstring, luaL_checktype,
};
use crate::memory::{ARENA_EMPTY, XString, arena_finish, arena_mem_free};
use crate::os::cshim::gettext;
use crate::types::{Arena, Expand, Failed};

/// The matches [`nlua_expand_pat`] produced, waiting for
/// [`nlua_expand_matches`] to take them.
static EXPAND_RESULTS: GlobalCell<Vec<XString>> = GlobalCell::new(Vec::new());

/// Complete the pattern of `expand` through `vim._expand_pat`, which answers
/// a prefix length and a list of strings.
///
/// The prefix length is how much of the pattern the matches already include,
/// so the pattern is advanced past it. Anything that goes wrong — the call,
/// the conversion, a non-string in the list, a prefix longer than the
/// pattern — leaves no matches at all.
///
/// The pattern is copied out before Lua runs: what Lua does cannot reach
/// `expand`.
pub fn nlua_expand_pat(expand: &mut Expand) {
    // From the pattern to the cursor.
    let col = usize::try_from(expand.col).unwrap_or(0);
    debug_assert!(col >= expand.pattern);
    let pattern = expand
        .line
        .get(expand.pattern..col.max(expand.pattern))
        .unwrap_or_default()
        .to_vec();
    let patlen = pattern.len();

    let mut matches = Vec::new();
    let mut prefix_len = None;
    // SAFETY: the global Lua state, used on the main thread; the pattern is
    // this frame's own copy.
    unsafe {
        let lstate = get_global_lstate();

        lua_getglobal(lstate, c"vim".as_ptr());
        lua_getfield(lstate, -1, c"_expand_pat".as_ptr());
        luaL_checktype(lstate, -1, LUA_TFUNCTION);

        lua_pushlstring(lstate, pattern.as_ptr().cast(), patlen);

        if nlua_pcall(lstate, 1, 2) != 0 {
            nlua_error(lstate, gettext(c"vim._expand_pat: %.*s").as_ptr());
            return;
        }

        let mut arena: Arena = ARENA_EMPTY;
        let prefix = nlua_pop_integer(lstate, &raw mut arena)
            .ok()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|&n| n <= patlen);
        if let Some(prefix) = prefix {
            let completions = nlua_pop_array(lstate, &raw mut arena);
            if let Ok(completions) = completions {
                let strings: Option<Vec<XString>> = (0..completions.len())
                    .map(|i| {
                        completions[i]
                            .as_string()
                            .map(|s| XString::from_cstr(s.as_cstr()))
                    })
                    .collect();
                if let Some(strings) = strings {
                    matches = strings;
                    prefix_len = Some(prefix);
                }
            }
            arena_mem_free(arena_finish(&raw mut arena));
        }
    }

    // Whatever a previous run left undrained goes here; a failed run keeps
    // nothing.
    match prefix_len {
        Some(prefix) => {
            expand.pattern += prefix;
            EXPAND_RESULTS.set(matches);
        }
        None => EXPAND_RESULTS.set(Vec::new()),
    }
}

/// Take the stashed matches. `Err` when there are none.
pub fn nlua_expand_matches() -> Result<Vec<XString>, Failed> {
    let matches = EXPAND_RESULTS.take();
    if matches.is_empty() {
        Err(Failed)
    } else {
        Ok(matches)
    }
}
