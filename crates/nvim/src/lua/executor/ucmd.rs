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
use crate::ex_docmd::cmdmod_report;
use crate::ex_getln::{cmdpreview_get_bufnr, cmdpreview_get_ns};
use crate::lua::ffi::{
    lua_isnumber, lua_newtable, lua_pop, lua_pushboolean, lua_pushinteger, lua_pushlstring,
    lua_pushnil, lua_pushstring, lua_pushvalue, lua_rawseti, lua_setfield, lua_tointeger,
};
use crate::main::{cmdmod, p_verbose};
use crate::memory::{xcalloc, xfree, xmalloc};
use crate::os::cshim::gettext;
use crate::path::fix_fname;
use crate::runtime::{find_script_by_name, new_script_item, script_is_lua};
use crate::types::{
    CmdModFlags, ExArgt, OptInt, exarg_T, handle_T, linenr_T, lua_Integer, scid_T, sctx_T, size_t,
    ucmd_T,
};
use crate::usercmd::{uc_mods, uc_split_args_iter};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};
use ::libc::strlen;

/// How much room `uc_mods` is given to render the modifier prefix.
const MODS_BUFSIZE: usize = 200;

/// The `smods` flags that are just a bit and a name.
const MOD_FLAGS: [(CmdModFlags, &CStr); 9] = [
    (CmdModFlags::BROWSE, c"browse"),
    (CmdModFlags::CONFIRM, c"confirm"),
    (CmdModFlags::HIDE, c"hide"),
    (CmdModFlags::KEEPALT, c"keepalt"),
    (CmdModFlags::KEEPJUMPS, c"keepjumps"),
    (CmdModFlags::KEEPMARKS, c"keepmarks"),
    (CmdModFlags::KEEPPATTERNS, c"keeppatterns"),
    (CmdModFlags::LOCKMARKS, c"lockmarks"),
    (CmdModFlags::NOSWAPFILE, c"noswapfile"),
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
pub unsafe fn nlua_set_sctx(current: *mut sctx_T) {
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
pub unsafe fn nlua_do_ucmd(cmd: *mut ucmd_T, eap: *mut exarg_T, preview: bool) -> c_int {
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
        if (*cmd).uc_argt.has(ExArgt::NOSPC) {
            // At most one argument: `fargs` is the whole of it, or empty.
            if (*cmd).uc_argt.has(ExArgt::NEEDARG) || strlen((*eap).arg) != 0 {
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
        cmdmod.with(|cmod| uc_mods(buf.as_mut_ptr(), cmod, false));
        lua_pushstring(lstate, buf.as_ptr());
        set(c"mods");

        // `smods`: the same modifiers, parsed.
        lua_newtable(lstate);
        let (tab, verbose, split, flags) = cmdmod_report();
        lua_pushinteger(lstate, (tab - 1) as lua_Integer);
        set(c"tab");
        lua_pushinteger(lstate, (verbose - 1) as lua_Integer);
        set(c"verbose");

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

        lua_pushboolean(lstate, flags.has(CmdModFlags::SILENT) as c_int);
        set(c"silent");
        lua_pushboolean(lstate, flags.has(CmdModFlags::ERRSILENT) as c_int);
        set(c"emsg_silent");
        lua_pushboolean(lstate, flags.has(CmdModFlags::UNSILENT) as c_int);
        set(c"unsilent");
        lua_pushboolean(lstate, flags.has(CmdModFlags::SANDBOX) as c_int);
        set(c"sandbox");
        lua_pushboolean(lstate, flags.has(CmdModFlags::NOAUTOCMD) as c_int);
        set(c"noautocmd");
        for (flag, name) in MOD_FLAGS {
            lua_pushboolean(lstate, flags.has(flag) as c_int);
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
            nlua_error(lstate, gettext(c"Lua :command callback: %.*s").as_ptr());
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
fn nargs_char(argt: ExArgt) -> c_char {
    if !argt.has(ExArgt::EXTRA) {
        return b'0' as c_char;
    }
    let needarg = argt.has(ExArgt::NEEDARG);
    if argt.has(ExArgt::NOSPC) {
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
