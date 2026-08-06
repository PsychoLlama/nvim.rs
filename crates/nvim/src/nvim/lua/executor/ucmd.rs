//! A user command whose implementation is a Lua function.
//!
//! `nlua_do_ucmd` builds the command's argument table -- name, args, fargs,
//! bang, line1/line2, range, count, reg, mods and the parsed `smods` -- and
//! calls the registered `LuaRef` with it.  `nlua_set_sctx` is what makes an
//! error inside that function report the Lua file and line it was defined
//! at rather than the command's.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_set_sctx(mut current: *mut sctx_T) {
    unsafe {
        let mut source_path: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut sid: ::core::ffi::c_int = 0;
        if !script_is_lua((*current).sc_sid) {
            return;
        }
        (*current).sc_lnum = 0 as ::core::ffi::c_int as linenr_T;
        if p_verbose.get() <= 0 as OptInt {
            return;
        }
        let lstate: *mut lua_State = active_lstate.get();
        let mut info: *mut lua_Debug =
            xmalloc(::core::mem::size_of::<lua_Debug>()) as *mut lua_Debug;
        let mut level: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        '_cleanup: {
            loop {
                if lua_getstack(lstate, level, info) != 1 as ::core::ffi::c_int {
                    break '_cleanup;
                }
                if lua_getinfo(
                    lstate,
                    b"nSl\0".as_ptr() as *const ::core::ffi::c_char,
                    info,
                ) == 0 as ::core::ffi::c_int
                {
                    break '_cleanup;
                }
                if !(*(*info).what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'C' as ::core::ffi::c_int
                    || *(*info).source.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        != '@' as ::core::ffi::c_int)
                {
                    break;
                }
                level += 1;
            }
            source_path = fix_fname((*info).source.offset(1 as ::core::ffi::c_int as isize));
            sid = find_script_by_name(source_path);
            if sid > 0 as ::core::ffi::c_int {
                xfree(source_path as *mut ::core::ffi::c_void);
            } else {
                let mut si: *mut scriptitem_T = new_script_item(source_path, &raw mut sid);
                (*si).sn_lua = true_0 != 0;
            }
            (*current).sc_sid = sid as scid_T;
            (*current).sc_seq = -1 as ::core::ffi::c_int;
            (*current).sc_lnum = (*info).currentline as linenr_T;
        }
        xfree(info as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C-unwind" fn nlua_do_ucmd(
    mut cmd: *mut ucmd_T,
    mut eap: *mut exarg_T,
    mut preview: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let lstate: *mut lua_State = global_lstate.get();
        nlua_pushref(
            lstate,
            if preview as ::core::ffi::c_int != 0 {
                (*cmd).uc_preview_luaref
            } else {
                (*cmd).uc_luaref
            },
        );
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_pushstring(lstate, (*cmd).uc_name);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"name\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            ((*eap).forceit == 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"bang\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(lstate, (*eap).line1 as lua_Integer);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"line1\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(lstate, (*eap).line2 as lua_Integer);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"line2\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_pushstring(lstate, (*eap).arg);
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -4 as ::core::ffi::c_int,
            b"args\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if (*cmd).uc_argt & EX_NOSPC as uint32_t != 0 {
            if (*cmd).uc_argt & EX_NEEDARG as uint32_t != 0 || strlen((*eap).arg) != 0 {
                lua_rawseti(lstate, -2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
            } else {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        } else if (*eap).args.is_null() {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            let mut length: size_t = strlen((*eap).arg);
            let mut end: size_t = 0 as size_t;
            let mut len: size_t = 0 as size_t;
            let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut buf: *mut ::core::ffi::c_char =
                xcalloc(length, ::core::mem::size_of::<::core::ffi::c_char>())
                    as *mut ::core::ffi::c_char;
            let mut done: bool = false_0 != 0;
            while !done {
                done = uc_split_args_iter((*eap).arg, length, &raw mut end, buf, &raw mut len);
                if len > 0 as size_t {
                    lua_pushlstring(lstate, buf, len);
                    lua_rawseti(lstate, -2 as ::core::ffi::c_int, i);
                    i += 1;
                }
            }
            xfree(buf as *mut ::core::ffi::c_void);
        } else {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            let mut i_0: size_t = 0 as size_t;
            while i_0 < (*eap).argc {
                lua_pushlstring(
                    lstate,
                    *(*eap).args.offset(i_0 as isize),
                    *(*eap).arglens.offset(i_0 as isize),
                );
                lua_rawseti(
                    lstate,
                    -2 as ::core::ffi::c_int,
                    i_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                );
                i_0 = i_0.wrapping_add(1);
            }
        }
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"fargs\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut reg: [::core::ffi::c_char; 2] = [
            (*eap).regname as ::core::ffi::c_char,
            NUL as ::core::ffi::c_char,
        ];
        lua_pushstring(lstate, &raw mut reg as *mut ::core::ffi::c_char);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"reg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(lstate, (*eap).addr_count as lua_Integer);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"range\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            lua_pushinteger(lstate, (*eap).line2 as lua_Integer);
        } else {
            lua_pushinteger(lstate, (*cmd).uc_def as lua_Integer);
        }
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"count\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut nargs: [::core::ffi::c_char; 2] = [0; 2];
        if (*cmd).uc_argt & EX_EXTRA as uint32_t != 0 {
            if (*cmd).uc_argt & EX_NOSPC as uint32_t != 0 {
                if (*cmd).uc_argt & EX_NEEDARG as uint32_t != 0 {
                    nargs[0 as ::core::ffi::c_int as usize] = '1' as ::core::ffi::c_char;
                } else {
                    nargs[0 as ::core::ffi::c_int as usize] = '?' as ::core::ffi::c_char;
                }
            } else if (*cmd).uc_argt & EX_NEEDARG as uint32_t != 0 {
                nargs[0 as ::core::ffi::c_int as usize] = '+' as ::core::ffi::c_char;
            } else {
                nargs[0 as ::core::ffi::c_int as usize] = '*' as ::core::ffi::c_char;
            }
        } else {
            nargs[0 as ::core::ffi::c_int as usize] = '0' as ::core::ffi::c_char;
        }
        nargs[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        lua_pushstring(lstate, &raw mut nargs as *mut ::core::ffi::c_char);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut buf_0: [::core::ffi::c_char; 200] = [
            0 as ::core::ffi::c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        uc_mods(
            &raw mut buf_0 as *mut ::core::ffi::c_char,
            cmdmod.ptr(),
            false_0 != 0,
        );
        lua_pushstring(lstate, &raw mut buf_0 as *mut ::core::ffi::c_char);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"mods\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_pushinteger(
            lstate,
            ((*cmdmod.ptr()).cmod_tab - 1 as ::core::ffi::c_int) as lua_Integer,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"tab\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(
            lstate,
            ((*cmdmod.ptr()).cmod_verbose - 1 as ::core::ffi::c_int) as lua_Integer,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"verbose\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if (*cmdmod.ptr()).cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
            lua_pushstring(
                lstate,
                b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if (*cmdmod.ptr()).cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
            lua_pushstring(
                lstate,
                b"belowright\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if (*cmdmod.ptr()).cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
            lua_pushstring(lstate, b"topleft\0".as_ptr() as *const ::core::ffi::c_char);
        } else if (*cmdmod.ptr()).cmod_split & WSP_BOT as ::core::ffi::c_int != 0 {
            lua_pushstring(lstate, b"botright\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            lua_pushstring(lstate, b"\0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"split\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_split & WSP_VERT as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_split & WSP_HOR as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_flags & CMOD_SILENT as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"silent\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"unsilent\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"sandbox\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char,
        );
        static mod_entries: GlobalCell<[mod_entry_T; 9]> = GlobalCell::new([
            mod_entry_T {
                flag: CMOD_BROWSE as ::core::ffi::c_int,
                name: b"browse\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_CONFIRM as ::core::ffi::c_int,
                name: b"confirm\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_HIDE as ::core::ffi::c_int,
                name: b"hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_KEEPALT as ::core::ffi::c_int,
                name: b"keepalt\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_KEEPJUMPS as ::core::ffi::c_int,
                name: b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_KEEPMARKS as ::core::ffi::c_int,
                name: b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_KEEPPATTERNS as ::core::ffi::c_int,
                name: b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_LOCKMARKS as ::core::ffi::c_int,
                name: b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
            mod_entry_T {
                flag: CMOD_NOSWAPFILE as ::core::ffi::c_int,
                name: b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
        ]);
        let mut i_1: size_t = 0 as size_t;
        while i_1
            < ::core::mem::size_of::<[mod_entry_T; 9]>()
                .wrapping_div(::core::mem::size_of::<mod_entry_T>())
                .wrapping_div(
                    (::core::mem::size_of::<[mod_entry_T; 9]>()
                        .wrapping_rem(::core::mem::size_of::<mod_entry_T>())
                        == 0) as ::core::ffi::c_int as usize,
                )
        {
            lua_pushboolean(
                lstate,
                (*cmdmod.ptr()).cmod_flags & (*mod_entries.ptr())[i_1 as usize].flag,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                (*mod_entries.ptr())[i_1 as usize].name,
            );
            i_1 = i_1.wrapping_add(1);
        }
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"smods\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if preview {
            lua_pushinteger(lstate, cmdpreview_get_ns() as lua_Integer);
            let mut cmdpreview_bufnr: handle_T = cmdpreview_get_bufnr();
            if cmdpreview_bufnr != 0 as ::core::ffi::c_int {
                lua_pushinteger(lstate, cmdpreview_bufnr as lua_Integer);
            } else {
                lua_pushnil(lstate);
            }
        }
        if nlua_pcall(
            lstate,
            if preview as ::core::ffi::c_int != 0 {
                3 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            },
            if preview as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        ) != 0
        {
            nlua_error(
                lstate,
                gettext(b"Lua :command callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return 0 as ::core::ffi::c_int;
        }
        let mut retv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if preview {
            if lua_isnumber(lstate, -1 as ::core::ffi::c_int) != 0
                && {
                    retv = lua_tointeger(lstate, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
                    retv >= 0 as ::core::ffi::c_int
                }
                && retv <= 2 as ::core::ffi::c_int
            {
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            } else {
                retv = 0 as ::core::ffi::c_int;
            }
        }
        return retv;
    }
}
