//! `vim.spell`: the Lua binding over the spell checker.
//!
//! # Boundary
//!
//! `nlua_spell_check` is a Lua C function, so it keeps the C-unwind ABI
//! and the raw `lua_State`, and it drives `spell_check` over a raw
//! `char *` the way the spell checker expects.

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};
use crate::src::nvim::lua::ffi::{
    lua_createtable, lua_gettop, lua_pushinteger, lua_pushlstring, lua_pushstring, lua_rawseti,
    lua_tolstring, lua_type, luaL_argerror, luaL_error, luaL_register,
};
use crate::src::nvim::main::{curwin, e_no_spell};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::spell::{parse_spelllang, spell_check};
pub use crate::src::nvim::types::{hlf_T, lua_Integer, lua_State, luaL_Reg, size_t};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// No misspelling: `spell_check` leaves the attribute at this when the
/// word it consumed was fine.
/// Spelling mistake.
/// Rare word.
/// Word only valid in another region.
/// Word should start with a capital.

pub const LUA_TSTRING: c_int = 4;

/// `vim.spell.check(str)`: the misspellings in `str`, each as a
/// `{ word, kind, byte_index }` triple.
unsafe extern "C-unwind" fn nlua_spell_check(lstate: *mut lua_State) -> c_int {
    if lua_gettop(lstate) < 1 {
        return luaL_error(lstate, c"Expected 1 argument".as_ptr());
    }
    if lua_type(lstate, 1) != LUA_TSTRING {
        // Does not return.
        luaL_argerror(lstate, 1, c"expected string".as_ptr());
    }
    let text = lua_tolstring(lstate, 1, ptr::null_mut());

    // spell.c insists 'spell' is on, so turn it on for the duration.
    let win = curwin.get();
    let wo_spell_save = (*win).w_onebuf_opt.wo_spell;
    if (*win).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(win);
        (*win).w_onebuf_opt.wo_spell = 1;
    }
    if *(*(*win).w_s).b_p_spl == 0 {
        emsg(gettext(&raw const e_no_spell as *const c_char));
        (*win).w_onebuf_opt.wo_spell = wo_spell_save;
        return 0;
    }

    let mut pos: size_t = 0;
    let mut capcol: c_int = -1;
    let mut nresults: c_int = 0;
    lua_createtable(lstate, 0, 0);
    let mut word = text;
    while *word != 0 {
        let mut attr: hlf_T = HLF_COUNT;
        let len = spell_check(
            win,
            word as *mut c_char,
            &raw mut attr,
            &raw mut capcol,
            false,
        );
        assert!(len <= c_int::MAX as size_t);
        if attr != HLF_COUNT {
            let kind: &CStr = match attr {
                HLF_SPB => c"bad",
                HLF_SPR => c"rare",
                HLF_SPL => c"local",
                HLF_SPC => c"caps",
                _ => unreachable!("spell_check reported an unknown attribute"),
            };
            lua_createtable(lstate, 3, 0);
            lua_pushlstring(lstate, word, len);
            lua_rawseti(lstate, -2, 1);
            lua_pushstring(lstate, kind.as_ptr());
            lua_rawseti(lstate, -2, 2);
            // +1 for Lua's 1-based indexing.
            lua_pushinteger(lstate, pos as lua_Integer + 1);
            lua_rawseti(lstate, -2, 3);
            nresults += 1;
            lua_rawseti(lstate, -2, nresults);
        }
        word = word.add(len);
        pos += len;
        capcol -= len as c_int;
    }

    (*win).w_onebuf_opt.wo_spell = wo_spell_save;
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
    lua_createtable(lstate, 0, 0);
    luaL_register(lstate, ptr::null(), SPELL_FUNCTIONS.ptr().cast());
    1
}
