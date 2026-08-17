//! A user command whose implementation is a Lua function.
//!
//! [`nlua_do_ucmd`] builds the command's argument table -- name, args,
//! fargs, bang, line1/line2, range, count, reg, mods and the parsed `smods`
//! -- and calls the registered `LuaRef` with it.  [`nlua_set_sctx`] is what
//! makes an error inside that function report the Lua file and line it was
//! defined at rather than the command's.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};

use super::{
    active_lstate, get_global_lstate, lua_Debug, lua_getinfo, lua_getstack, nlua_error, nlua_pcall,
    nlua_pushref,
};
use crate::ex_getln::{cmdpreview_get_bufnr, cmdpreview_get_ns};
use crate::lua::ffi::{
    lua_isnumber, lua_newtable, lua_pop, lua_pushboolean, lua_pushinteger, lua_pushlstring,
    lua_pushnil, lua_pushstring, lua_pushvalue, lua_rawseti, lua_setfield, lua_tointeger,
};
use crate::main::{cmdmod, p_verbose};
use crate::memory::{xcalloc, xfree, xmalloc};
use crate::os::libc::{gettext, strlen};
use crate::path::fix_fname;
use crate::runtime::{find_script_by_name, new_script_item, script_is_lua};
use crate::types::{
    CMOD_BROWSE, CMOD_CONFIRM, CMOD_ERRSILENT, CMOD_HIDE, CMOD_KEEPALT, CMOD_KEEPJUMPS,
    CMOD_KEEPMARKS, CMOD_KEEPPATTERNS, CMOD_LOCKMARKS, CMOD_NOAUTOCMD, CMOD_NOSWAPFILE,
    CMOD_SANDBOX, CMOD_SILENT, CMOD_UNSILENT, OptInt, exarg_T, handle_T, linenr_T, lua_Integer,
    scid_T, sctx_T, size_t, ucmd_T, uint32_t,
};
use crate::usercmd::{EX_EXTRA, EX_NEEDARG, EX_NOSPC, uc_mods, uc_split_args_iter};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};

/// How much room `uc_mods` is given to render the modifier prefix.
const MODS_BUFSIZE: usize = 200;

/// The `smods` flags that are just a bit and a name.
const MOD_FLAGS: [(c_int, &CStr); 9] = [
    (CMOD_BROWSE as c_int, c"browse"),
    (CMOD_CONFIRM as c_int, c"confirm"),
    (CMOD_HIDE as c_int, c"hide"),
    (CMOD_KEEPALT as c_int, c"keepalt"),
    (CMOD_KEEPJUMPS as c_int, c"keepjumps"),
    (CMOD_KEEPMARKS as c_int, c"keepmarks"),
    (CMOD_KEEPPATTERNS as c_int, c"keeppatterns"),
    (CMOD_LOCKMARKS as c_int, c"lockmarks"),
    (CMOD_NOSWAPFILE as c_int, c"noswapfile"),
];

/// Point `current` at the Lua source position the running function was
/// defined at, so `:verbose` and an error message name the `.lua` file.
///
/// Only the *innermost Lua* frame counts: C frames and chunks that came from
/// a string rather than a file are skipped. Nothing happens below
/// `'verbose'` 1, because walking the Lua stack is not free.
///
/// # Safety
/// `current` must be a writable script context.
pub unsafe extern "C-unwind" fn nlua_set_sctx(current: *mut sctx_T) {
    unsafe {
        if !script_is_lua((*current).sc_sid) {
            return;
        }
        (*current).sc_lnum = 0;
        if p_verbose.get() <= 0 as OptInt {
            return;
        }

        let lstate = active_lstate.get();
        let info = xmalloc(size_of::<lua_Debug>()).cast::<lua_Debug>();
        let mut level: c_int = 1;
        'cleanup: {
            loop {
                if lua_getstack(lstate, level, info) != 1 {
                    break 'cleanup;
                }
                if lua_getinfo(lstate, c"nSl".as_ptr(), info) == 0 {
                    break 'cleanup;
                }
                let is_c = *(*info).what == b'C' as c_char;
                let from_string = *(*info).source != b'@' as c_char;
                if !is_c && !from_string {
                    break;
                }
                level += 1;
            }

            let source_path = fix_fname((*info).source.add(1));
            let mut sid = find_script_by_name(source_path);
            if sid > 0 {
                xfree(source_path.cast::<c_void>());
            } else {
                let si = new_script_item(source_path, &raw mut sid);
                (*si).sn_lua = true;
            }
            (*current).sc_sid = sid as scid_T;
            (*current).sc_seq = -1;
            (*current).sc_lnum = (*info).currentline as linenr_T;
        }
        xfree(info.cast::<c_void>());
    }
}

/// Call a Lua-implemented user command, or its `preview` half.
///
/// The preview callback takes two extra arguments — the preview namespace
/// and buffer — and answers how much of a preview it produced (0 to 2);
/// anything else is 0.
///
/// # Safety
/// `cmd` and `eap` must be live, and the command must carry the `LuaRef`
/// this is being asked for.
pub unsafe extern "C-unwind" fn nlua_do_ucmd(
    cmd: *mut ucmd_T,
    eap: *mut exarg_T,
    preview: bool,
) -> c_int {
    unsafe {
        let lstate = get_global_lstate();
        nlua_pushref(
            lstate,
            if preview {
                (*cmd).uc_preview_luaref
            } else {
                (*cmd).uc_luaref
            },
        );

        // The one argument is a table describing the invocation.
        lua_newtable(lstate);
        let set = |name: &CStr| lua_setfield(lstate, -2, name.as_ptr());

        lua_pushstring(lstate, (*cmd).uc_name);
        set(c"name");
        lua_pushboolean(lstate, ((*eap).forceit == 1) as c_int);
        set(c"bang");
        lua_pushinteger(lstate, (*eap).line1 as lua_Integer);
        set(c"line1");
        lua_pushinteger(lstate, (*eap).line2 as lua_Integer);
        set(c"line2");

        // `args` is the raw argument text; `fargs` the split one. The raw
        // string is pushed once and stored twice.
        lua_newtable(lstate);
        lua_pushstring(lstate, (*eap).arg);
        lua_pushvalue(lstate, -1);
        lua_setfield(lstate, -4, c"args".as_ptr());
        if (*cmd).uc_argt & EX_NOSPC as uint32_t != 0 {
            // At most one argument: `fargs` is the whole of it, or empty.
            if (*cmd).uc_argt & EX_NEEDARG as uint32_t != 0 || strlen((*eap).arg) != 0 {
                lua_rawseti(lstate, -2, 1);
            } else {
                lua_pop(lstate, 1);
            }
        } else if (*eap).args.is_null() {
            lua_pop(lstate, 1);
            // Not pre-split (`:command` rather than `nvim_cmd`): split here,
            // honouring backslash escapes.
            let length = strlen((*eap).arg);
            let mut end: size_t = 0;
            let mut len: size_t = 0;
            let mut i: c_int = 1;
            let buf = xcalloc(length, size_of::<c_char>()).cast::<c_char>();
            let mut done = false;
            while !done {
                done = uc_split_args_iter((*eap).arg, length, &raw mut end, buf, &raw mut len);
                if len > 0 {
                    lua_pushlstring(lstate, buf, len);
                    lua_rawseti(lstate, -2, i);
                    i += 1;
                }
            }
            xfree(buf.cast::<c_void>());
        } else {
            lua_pop(lstate, 1);
            for i in 0..(*eap).argc {
                lua_pushlstring(lstate, *(*eap).args.add(i), *(*eap).arglens.add(i));
                lua_rawseti(lstate, -2, i as c_int + 1);
            }
        }
        set(c"fargs");

        let reg = [(*eap).regname as c_char, 0];
        lua_pushstring(lstate, reg.as_ptr());
        set(c"reg");
        lua_pushinteger(lstate, (*eap).addr_count as lua_Integer);
        set(c"range");
        if (*eap).addr_count > 0 {
            lua_pushinteger(lstate, (*eap).line2 as lua_Integer);
        } else {
            lua_pushinteger(lstate, (*cmd).uc_def as lua_Integer);
        }
        set(c"count");

        // `nargs` as `:command -nargs=` spells it.
        let nargs = [nargs_char((*cmd).uc_argt), 0];
        lua_pushstring(lstate, nargs.as_ptr());
        set(c"nargs");

        let mut buf = [0 as c_char; MODS_BUFSIZE];
        uc_mods(buf.as_mut_ptr(), cmdmod.ptr(), false);
        lua_pushstring(lstate, buf.as_ptr());
        set(c"mods");

        // `smods`: the same modifiers, parsed.
        lua_newtable(lstate);
        let cmod = cmdmod.ptr();
        lua_pushinteger(lstate, ((*cmod).cmod_tab - 1) as lua_Integer);
        set(c"tab");
        lua_pushinteger(lstate, ((*cmod).cmod_verbose - 1) as lua_Integer);
        set(c"verbose");

        let split = (*cmod).cmod_split;
        let split_name = if split & WSP_ABOVE as c_int != 0 {
            c"aboveleft"
        } else if split & WSP_BELOW as c_int != 0 {
            c"belowright"
        } else if split & WSP_TOP as c_int != 0 {
            c"topleft"
        } else if split & WSP_BOT as c_int != 0 {
            c"botright"
        } else {
            c""
        };
        lua_pushstring(lstate, split_name.as_ptr());
        set(c"split");

        lua_pushboolean(lstate, split & WSP_VERT as c_int);
        set(c"vertical");
        lua_pushboolean(lstate, split & WSP_HOR as c_int);
        set(c"horizontal");

        let flags = (*cmod).cmod_flags;
        lua_pushboolean(lstate, flags & CMOD_SILENT as c_int);
        set(c"silent");
        lua_pushboolean(lstate, flags & CMOD_ERRSILENT as c_int);
        set(c"emsg_silent");
        lua_pushboolean(lstate, flags & CMOD_UNSILENT as c_int);
        set(c"unsilent");
        lua_pushboolean(lstate, flags & CMOD_SANDBOX as c_int);
        set(c"sandbox");
        lua_pushboolean(lstate, flags & CMOD_NOAUTOCMD as c_int);
        set(c"noautocmd");
        for (flag, name) in MOD_FLAGS {
            lua_pushboolean(lstate, flags & flag);
            set(name);
        }
        set(c"smods");

        if preview {
            lua_pushinteger(lstate, cmdpreview_get_ns() as lua_Integer);
            let cmdpreview_bufnr: handle_T = cmdpreview_get_bufnr();
            if cmdpreview_bufnr != 0 {
                lua_pushinteger(lstate, cmdpreview_bufnr as lua_Integer);
            } else {
                lua_pushnil(lstate);
            }
        }

        let (nargs_in, nresults) = if preview { (3, 1) } else { (1, 0) };
        if nlua_pcall(lstate, nargs_in, nresults) != 0 {
            nlua_error(lstate, gettext(c"Lua :command callback: %.*s".as_ptr()));
            return 0;
        }

        let mut retv: c_int = 0;
        if preview {
            if lua_isnumber(lstate, -1) != 0 && {
                retv = lua_tointeger(lstate, -1) as c_int;
                (0..=2).contains(&retv)
            } {
                lua_pop(lstate, 1);
            } else {
                retv = 0;
            }
        }
        retv
    }
}

/// The `-nargs=` letter this command's argument flags mean.
fn nargs_char(argt: uint32_t) -> c_char {
    if argt & EX_EXTRA as uint32_t == 0 {
        return b'0' as c_char;
    }
    let needarg = argt & EX_NEEDARG as uint32_t != 0;
    if argt & EX_NOSPC as uint32_t != 0 {
        if needarg {
            b'1' as c_char
        } else {
            b'?' as c_char
        }
    } else if needarg {
        b'+' as c_char
    } else {
        b'*' as c_char
    }
}
