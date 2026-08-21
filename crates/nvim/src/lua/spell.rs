#![deny(unsafe_op_in_unsafe_fn)]

//! `vim.spell`: the Lua binding over the spell checker.
//!
//! # Boundary
//!
//! `nlua_spell_check` is a Lua C function, so it keeps the C-unwind ABI
//! and the raw `lua_State`, and it drives `spell_check` over a raw
//! `char *` the way the spell checker expects.

use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};
use crate::lua::ffi::{
    LUA_TSTRING, lua_createtable, lua_gettop, lua_pushinteger, lua_pushlstring, lua_pushstring,
    lua_rawseti, lua_tolstring, lua_type, luaL_argerror, luaL_error, luaL_register,
};
use crate::main::{curwin, e_no_spell};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::spell::{parse_spelllang, spell_check};
use crate::types::{hlf_T, lua_Integer, lua_State, luaL_Reg, size_t};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// `vim.spell.check(str)`: the misspellings in `str`, each as a
/// `{ word, kind, byte_index }` triple.
///
/// # Safety
/// Called by Lua with a live `lua_State`, from the editor's main thread —
/// it reads `curwin` and turns 'spell' on for the duration.
unsafe extern "C-unwind" fn nlua_spell_check(lstate: *mut lua_State) -> c_int {
    // SAFETY: Lua calls this with a live state; `luaL_argerror` does not
    // return, and the checked argument stays on the stack for the whole
    // call, so its bytes outlive the walk below.
    let text = unsafe {
        if lua_gettop(lstate) < 1 {
            return luaL_error(lstate, c"Expected 1 argument".as_ptr());
        }
        if lua_type(lstate, 1) != LUA_TSTRING {
            luaL_argerror(lstate, 1, c"expected string".as_ptr());
        }
        lua_tolstring(lstate, 1, ptr::null_mut())
    };

    // spell.c insists 'spell' is on, so turn it on for the duration.
    let win = curwin.get();
    // SAFETY: `curwin` is a live window whenever Lua is running.
    let wo_spell_save = unsafe {
        let saved = (*win).w_onebuf_opt.wo_spell;
        if saved == 0 {
            parse_spelllang(win);
            (*win).w_onebuf_opt.wo_spell = 1;
        }
        saved
    };
    // SAFETY: as above; `w_s` is the window's synblock, always set.
    if unsafe { *(*(*win).w_s).b_p_spl } == 0 {
        // SAFETY: as above; `e_no_spell` is a `static` message.
        unsafe {
            emsg(gettext((&raw const e_no_spell).cast::<c_char>()));
            (*win).w_onebuf_opt.wo_spell = wo_spell_save;
        }
        return 0;
    }

    let mut pos: size_t = 0;
    let mut capcol: c_int = -1;
    let mut nresults: c_int = 0;
    // SAFETY: the caller's live state.
    unsafe { lua_createtable(lstate, 0, 0) };
    let mut word = text;
    // SAFETY: `text` is NUL-terminated and `spell_check` never steps past
    // the terminator, so `word` stays inside it.
    while unsafe { *word } != 0 {
        let mut attr: hlf_T = HLF_COUNT;
        // SAFETY: as above, with a live window.
        let len =
            unsafe { spell_check(win, word.cast_mut(), &raw mut attr, &raw mut capcol, false) };
        debug_assert!(len <= c_int::MAX as size_t);
        if attr != HLF_COUNT {
            let kind: &CStr = match attr {
                HLF_SPB => c"bad",
                HLF_SPR => c"rare",
                HLF_SPL => c"local",
                HLF_SPC => c"caps",
                _ => unreachable!("spell_check reported an unknown attribute"),
            };
            nresults += 1;
            // SAFETY: as above. The triple is built on top of the results
            // table and `rawseti`'d into it, so the stack ends level.
            unsafe {
                lua_createtable(lstate, 3, 0);
                lua_pushlstring(lstate, word, len);
                lua_rawseti(lstate, -2, 1);
                lua_pushstring(lstate, kind.as_ptr());
                lua_rawseti(lstate, -2, 2);
                // +1 for Lua's 1-based indexing.
                lua_pushinteger(lstate, pos as lua_Integer + 1);
                lua_rawseti(lstate, -2, 3);
                lua_rawseti(lstate, -2, nresults);
            }
        }
        // SAFETY: as above.
        word = unsafe { word.add(len) };
        pos += len;
        capcol -= len as c_int;
    }

    // SAFETY: as above; 'spell' goes back to what it was.
    unsafe { (*win).w_onebuf_opt.wo_spell = wo_spell_save };
    1
}

/// What `luaL_register` copies into the table, terminated by a null name.
/// The raw name pointers are what keep it out of a plain `static`.
static SPELL_FUNCTIONS: GlobalCell<[luaL_Reg; 2]> = GlobalCell::new([
    luaL_Reg {
        name: c"check".as_ptr(),
        func: Some(nlua_spell_check),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
]);

/// Build the `vim.spell` table and leave it on the stack.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
pub unsafe fn luaopen_spell(lstate: *mut lua_State) -> c_int {
    // SAFETY: the caller's live state; the table is what `luaL_register`
    // copies the (`'static`) registry into.
    unsafe {
        lua_createtable(lstate, 0, 0);
        luaL_register(lstate, ptr::null(), SPELL_FUNCTIONS.ptr().cast());
    }
    1
}
