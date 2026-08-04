//! `:map` and friends: the command layer over the table.
//!
//! [`buf_do_map`] does all of it — list, add, replace and delete — for one
//! already-parsed [`MapArguments`]; [`do_map`] and [`do_exmap`] are the
//! parsing wrappers, and the `ex_*` entry points are what the command table
//! dispatches to.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn buf_do_map(
    mut maptype: ::core::ffi::c_int,
    mut args: *mut MapArguments,
    mut mode: ::core::ffi::c_int,
    mut is_abbrev: bool,
    mut buf: *mut buf_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lhs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut did_simplify: bool = false;
        let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut map_table: *mut *mut mapblock_T = if (*args).buffer as ::core::ffi::c_int != 0 {
            &raw mut (*buf).b_maphash as *mut *mut mapblock_T
        } else {
            maphash.ptr() as *mut *mut mapblock_T
        };
        let mut abbr_table: *mut *mut mapblock_T = if (*args).buffer as ::core::ffi::c_int != 0 {
            &raw mut (*buf).b_first_abbr
        } else {
            first_abbr.ptr()
        };
        let mut mp_result: [*mut mapblock_T; 2] = [
            ::core::ptr::null_mut::<mapblock_T>(),
            ::core::ptr::null_mut::<mapblock_T>(),
        ];
        let mut unmap_lhs_only: bool = false_0 != 0;
        if maptype == MAPTYPE_UNMAP_LHS as ::core::ffi::c_int {
            unmap_lhs_only = true_0 != 0;
            maptype = MAPTYPE_UNMAP as ::core::ffi::c_int;
        }
        let mut noremap: ::core::ffi::c_int = if (*args).script as ::core::ffi::c_int != 0 {
            REMAP_SCRIPT as ::core::ffi::c_int
        } else if maptype == MAPTYPE_NOREMAP as ::core::ffi::c_int {
            REMAP_NONE as ::core::ffi::c_int
        } else {
            REMAP_YES as ::core::ffi::c_int
        };
        let has_lhs: bool =
            (*args).lhs[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL;
        let has_rhs: bool = (*args).rhs_lua != LUA_NOREF
            || *(*args).rhs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            || (*args).rhs_is_noop as ::core::ffi::c_int != 0;
        let do_print: bool = !has_lhs || maptype != MAPTYPE_UNMAP as ::core::ffi::c_int && !has_rhs;
        if do_print {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        }
        '_theend: {
            if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int && !has_lhs {
                retval = 1 as ::core::ffi::c_int;
            } else {
                lhs = &raw mut (*args).lhs as *mut ::core::ffi::c_char;
                did_simplify = (*args).alt_lhs_len != 0 as size_t;
                let mut keyround: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while keyround <= 2 as ::core::ffi::c_int {
                    let mut did_it: bool = false_0 != 0;
                    let mut did_local: bool = false_0 != 0;
                    let mut keyround1_simplified: bool = keyround == 1 as ::core::ffi::c_int
                        && did_simplify as ::core::ffi::c_int != 0;
                    let mut len: ::core::ffi::c_int = (*args).lhs_len as ::core::ffi::c_int;
                    if keyround == 2 as ::core::ffi::c_int {
                        if !did_simplify {
                            break;
                        }
                        lhs = &raw mut (*args).alt_lhs as *mut ::core::ffi::c_char;
                        len = (*args).alt_lhs_len as ::core::ffi::c_int;
                    } else if did_simplify as ::core::ffi::c_int != 0
                        && do_print as ::core::ffi::c_int != 0
                    {
                        lhs = &raw mut (*args).alt_lhs as *mut ::core::ffi::c_char;
                        len = (*args).alt_lhs_len as ::core::ffi::c_int;
                    }
                    's_209: {
                        if has_lhs {
                            if len > MAXMAPLEN as ::core::ffi::c_int {
                                retval = 1 as ::core::ffi::c_int;
                                break '_theend;
                            } else if is_abbrev as ::core::ffi::c_int != 0
                                && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                            {
                                let mut same: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                                let first: ::core::ffi::c_int =
                                    vim_iswordp(lhs) as ::core::ffi::c_int;
                                let mut last: ::core::ffi::c_int = first;
                                let mut p: *const ::core::ffi::c_char =
                                    lhs.offset(utfc_ptr2len(lhs) as isize);
                                let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                                while p < lhs.offset(len as isize) {
                                    n += 1;
                                    last = vim_iswordp(p) as ::core::ffi::c_int;
                                    if same == -1 as ::core::ffi::c_int && last != first {
                                        same = n - 1 as ::core::ffi::c_int;
                                    }
                                    p = p.offset(utfc_ptr2len(p) as isize);
                                }
                                if last != 0
                                    && n > 2 as ::core::ffi::c_int
                                    && same >= 0 as ::core::ffi::c_int
                                    && same < n - 1 as ::core::ffi::c_int
                                {
                                    retval = 1 as ::core::ffi::c_int;
                                    break '_theend;
                                } else {
                                    n = 0 as ::core::ffi::c_int;
                                    loop {
                                        if n >= len {
                                            break 's_209;
                                        }
                                        if ascii_iswhite(
                                            *lhs.offset(n as isize) as ::core::ffi::c_int
                                        ) {
                                            retval = 1 as ::core::ffi::c_int;
                                            break '_theend;
                                        } else {
                                            n += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if has_lhs as ::core::ffi::c_int != 0
                        && has_rhs as ::core::ffi::c_int != 0
                        && is_abbrev as ::core::ffi::c_int != 0
                    {
                        no_abbr.set(false_0 != 0);
                    }
                    if do_print {
                        msg_start();
                    }
                    's_299: {
                        if (*args).unique as ::core::ffi::c_int != 0
                            && map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T
                            && has_lhs as ::core::ffi::c_int != 0
                            && has_rhs as ::core::ffi::c_int != 0
                            && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                        {
                            let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            loop {
                                if !(hash < 256 as ::core::ffi::c_int && !got_int.get()) {
                                    break 's_299;
                                }
                                let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                                if is_abbrev {
                                    if hash != 0 as ::core::ffi::c_int {
                                        break 's_299;
                                    }
                                    mp = first_abbr.get();
                                } else {
                                    mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
                                }
                                while !mp.is_null() && !got_int.get() {
                                    if (*mp).m_mode & mode != 0 as ::core::ffi::c_int
                                        && (*mp).m_keylen == len
                                        && strncmp((*mp).m_keys, lhs, len as size_t)
                                            == 0 as ::core::ffi::c_int
                                    {
                                        retval = 6 as ::core::ffi::c_int;
                                        break '_theend;
                                    } else {
                                        mp = (*mp).m_next;
                                    }
                                }
                                hash += 1;
                            }
                        }
                    }
                    if map_table != &raw mut (*buf).b_maphash as *mut *mut mapblock_T
                        && !has_rhs
                        && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                    {
                        let mut hash_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while hash_0 < 256 as ::core::ffi::c_int && !got_int.get() {
                            let mut mp_0: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                            if is_abbrev {
                                if hash_0 != 0 as ::core::ffi::c_int {
                                    break;
                                }
                                mp_0 = (*buf).b_first_abbr;
                            } else {
                                mp_0 = (*buf).b_maphash[hash_0 as usize] as *mut mapblock_T;
                            }
                            while !mp_0.is_null() && !got_int.get() {
                                if (*mp_0).m_simplified == 0
                                    && (*mp_0).m_mode & mode != 0 as ::core::ffi::c_int
                                {
                                    if !has_lhs {
                                        showmap(mp_0, true_0 != 0);
                                        did_local = true_0 != 0;
                                    } else {
                                        let mut n_0: ::core::ffi::c_int = (*mp_0).m_keylen;
                                        if strncmp(
                                            (*mp_0).m_keys,
                                            lhs,
                                            (if n_0 < len { n_0 } else { len }) as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            showmap(mp_0, true_0 != 0);
                                            did_local = true_0 != 0;
                                        }
                                    }
                                }
                                mp_0 = (*mp_0).m_next;
                            }
                            hash_0 += 1;
                        }
                    }
                    let num_rounds: ::core::ffi::c_int =
                        if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int && !unmap_lhs_only {
                            2 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        };
                    let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while round < num_rounds && !did_it && !got_int.get() {
                        let mut hash_start: ::core::ffi::c_int = 0;
                        let mut hash_end: ::core::ffi::c_int = 0;
                        if round == 0 as ::core::ffi::c_int && has_lhs as ::core::ffi::c_int != 0
                            || is_abbrev as ::core::ffi::c_int != 0
                        {
                            hash_start = if is_abbrev as ::core::ffi::c_int != 0 {
                                0 as ::core::ffi::c_int
                            } else if mode
                                & (MODE_NORMAL
                                    | MODE_VISUAL
                                    | MODE_SELECT
                                    | MODE_OP_PENDING
                                    | MODE_TERMINAL)
                                != 0
                            {
                                *lhs.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int
                            } else {
                                *lhs.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int
                                    ^ 0x80 as ::core::ffi::c_int
                            };
                            hash_end = hash_start + 1 as ::core::ffi::c_int;
                        } else {
                            hash_start = 0 as ::core::ffi::c_int;
                            hash_end = 256 as ::core::ffi::c_int;
                        }
                        let mut hash_1: ::core::ffi::c_int = hash_start;
                        while hash_1 < hash_end && !got_int.get() {
                            let mut mpp: *mut *mut mapblock_T =
                                if is_abbrev as ::core::ffi::c_int != 0 {
                                    abbr_table
                                } else {
                                    map_table.offset(hash_1 as isize)
                                };
                            let mut mp_1: *mut mapblock_T = *mpp;
                            's_448: while !mp_1.is_null() && !got_int.get() {
                                's_458: {
                                    if (*mp_1).m_mode & mode == 0 as ::core::ffi::c_int {
                                        mpp = &raw mut (*mp_1).m_next;
                                    } else {
                                        if !has_lhs {
                                            if (*mp_1).m_simplified == 0 {
                                                showmap(
                                                    mp_1,
                                                    map_table
                                                        != maphash.ptr() as *mut *mut mapblock_T,
                                                );
                                                did_it = true_0 != 0;
                                            }
                                        } else {
                                            let mut n_1: ::core::ffi::c_int = 0;
                                            let mut p_0: *const ::core::ffi::c_char =
                                                ::core::ptr::null::<::core::ffi::c_char>();
                                            if round != 0 {
                                                n_1 = strlen((*mp_1).m_str) as ::core::ffi::c_int;
                                                p_0 = (*mp_1).m_str;
                                            } else {
                                                n_1 = (*mp_1).m_keylen;
                                                p_0 = (*mp_1).m_keys;
                                            }
                                            if strncmp(
                                                p_0,
                                                lhs,
                                                (if n_1 < len { n_1 } else { len }) as size_t,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int {
                                                    if n_1 != len
                                                        && (!is_abbrev
                                                            || round != 0
                                                            || n_1 > len
                                                            || *skipwhite(lhs.offset(n_1 as isize))
                                                                as ::core::ffi::c_int
                                                                != NUL)
                                                    {
                                                        mpp = &raw mut (*mp_1).m_next;
                                                        break 's_458;
                                                    } else {
                                                        if keyround1_simplified
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && (*mp_1).m_simplified == 0
                                                        {
                                                            break 's_448;
                                                        }
                                                        (*mp_1).m_mode &= !mode;
                                                        did_it = true_0 != 0;
                                                    }
                                                } else if !has_rhs {
                                                    if (*mp_1).m_simplified == 0 {
                                                        showmap(
                                                            mp_1,
                                                            map_table
                                                                != maphash.ptr()
                                                                    as *mut *mut mapblock_T,
                                                        );
                                                        did_it = true_0 != 0;
                                                    }
                                                } else if n_1 != len {
                                                    mpp = &raw mut (*mp_1).m_next;
                                                    break 's_458;
                                                } else if keyround1_simplified as ::core::ffi::c_int
                                                    != 0
                                                    && (*mp_1).m_simplified == 0
                                                {
                                                    did_it = true_0 != 0;
                                                    break 's_448;
                                                } else if (*args).unique {
                                                    retval = 5 as ::core::ffi::c_int;
                                                    break '_theend;
                                                } else {
                                                    (*mp_1).m_mode &= !mode;
                                                    if (*mp_1).m_mode == 0 as ::core::ffi::c_int
                                                        && !did_it
                                                    {
                                                        if !(*mp_1).m_alt.is_null() {
                                                            (*(*mp_1).m_alt).m_alt =
                                                                ::core::ptr::null_mut::<mapblock_T>(
                                                                );
                                                            (*mp_1).m_alt = (*(*mp_1).m_alt).m_alt;
                                                        } else {
                                                            if (*mp_1).m_luaref != LUA_NOREF {
                                                                api_free_luaref((*mp_1).m_luaref);
                                                                (*mp_1).m_luaref =
                                                                    LUA_NOREF as LuaRef;
                                                            }
                                                            xfree(
                                                                (*mp_1).m_str
                                                                    as *mut ::core::ffi::c_void,
                                                            );
                                                            xfree(
                                                                (*mp_1).m_orig_str
                                                                    as *mut ::core::ffi::c_void,
                                                            );
                                                            xfree(
                                                                (*mp_1).m_desc
                                                                    as *mut ::core::ffi::c_void,
                                                            );
                                                        }
                                                        (*mp_1).m_str = (*args).rhs;
                                                        (*mp_1).m_orig_str = (*args).orig_rhs;
                                                        (*mp_1).m_luaref = (*args).rhs_lua;
                                                        (*mp_1).m_noremap = noremap;
                                                        (*mp_1).m_nowait =
                                                            (*args).nowait as ::core::ffi::c_char;
                                                        (*mp_1).m_silent =
                                                            (*args).silent as ::core::ffi::c_char;
                                                        (*mp_1).m_mode = mode;
                                                        (*mp_1).m_simplified = keyround1_simplified
                                                            as ::core::ffi::c_int;
                                                        (*mp_1).m_expr =
                                                            (*args).expr as ::core::ffi::c_char;
                                                        (*mp_1).m_replace_keycodes =
                                                            (*args).replace_keycodes;
                                                        (*mp_1).m_script_ctx = current_sctx.get();
                                                        (*mp_1).m_script_ctx.sc_lnum +=
                                                            (*((*exestack.ptr()).ga_data
                                                                as *mut estack_T)
                                                                .offset(
                                                                    ((*exestack.ptr()).ga_len
                                                                        - 1 as ::core::ffi::c_int)
                                                                        as isize,
                                                                ))
                                                            .es_lnum;
                                                        nlua_set_sctx(
                                                            &raw mut (*mp_1).m_script_ctx,
                                                        );
                                                        (*mp_1).m_desc = (*args).desc;
                                                        mp_result[(keyround
                                                            - 1 as ::core::ffi::c_int)
                                                            as usize] = mp_1;
                                                        did_it = true_0 != 0;
                                                    }
                                                }
                                                if (*mp_1).m_mode == 0 as ::core::ffi::c_int {
                                                    mapblock_free(mpp);
                                                    break 's_458;
                                                } else {
                                                    let mut new_hash: ::core::ffi::c_int =
                                                        if (*mp_1).m_mode
                                                            & (MODE_NORMAL
                                                                | MODE_VISUAL
                                                                | MODE_SELECT
                                                                | MODE_OP_PENDING
                                                                | MODE_TERMINAL)
                                                            != 0
                                                        {
                                                            *(*mp_1).m_keys.offset(
                                                                0 as ::core::ffi::c_int as isize,
                                                            )
                                                                as uint8_t
                                                                as ::core::ffi::c_int
                                                        } else {
                                                            *(*mp_1).m_keys.offset(
                                                                0 as ::core::ffi::c_int as isize,
                                                            )
                                                                as uint8_t
                                                                as ::core::ffi::c_int
                                                                ^ 0x80 as ::core::ffi::c_int
                                                        };
                                                    if !is_abbrev && new_hash != hash_1 {
                                                        *mpp = (*mp_1).m_next;
                                                        (*mp_1).m_next =
                                                            *map_table.offset(new_hash as isize);
                                                        *map_table.offset(new_hash as isize) = mp_1;
                                                        break 's_458;
                                                    }
                                                }
                                            }
                                        }
                                        mpp = &raw mut (*mp_1).m_next;
                                    }
                                }
                                mp_1 = *mpp;
                            }
                            hash_1 += 1;
                        }
                        round += 1;
                    }
                    if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int {
                        if !did_it {
                            if !keyround1_simplified {
                                retval = 2 as ::core::ffi::c_int;
                            }
                        } else if *lhs as ::core::ffi::c_int == Ctrl_C {
                            if map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T {
                                (*buf).b_mapped_ctrl_c &= !mode;
                            } else {
                                (*mapped_ctrl_c.ptr()) &= !mode;
                            }
                        }
                    } else if !has_lhs || !has_rhs {
                        if !did_it && !did_local {
                            if is_abbrev {
                                msg(
                                    gettext(b"No abbreviation found\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    0 as ::core::ffi::c_int,
                                );
                            } else {
                                msg(
                                    gettext(b"No mapping found\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    0 as ::core::ffi::c_int,
                                );
                            }
                        }
                        break '_theend;
                    } else if !did_it {
                        mp_result[(keyround - 1 as ::core::ffi::c_int) as usize] = map_add(
                            buf,
                            map_table,
                            abbr_table,
                            lhs,
                            args,
                            noremap,
                            mode,
                            is_abbrev,
                            0 as scid_T,
                            0 as linenr_T,
                            keyround1_simplified,
                        );
                    }
                    keyround += 1;
                }
                if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
                    && !mp_result[1 as ::core::ffi::c_int as usize].is_null()
                {
                    (*mp_result[0 as ::core::ffi::c_int as usize]).m_alt =
                        mp_result[1 as ::core::ffi::c_int as usize];
                    (*mp_result[1 as ::core::ffi::c_int as usize]).m_alt =
                        mp_result[0 as ::core::ffi::c_int as usize];
                }
            }
        }
        if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
            || !mp_result[1 as ::core::ffi::c_int as usize].is_null()
        {
            (*args).rhs = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*args).orig_rhs = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*args).rhs_lua = LUA_NOREF as LuaRef;
            (*args).desc = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return retval;
    }
}

pub unsafe extern "C" fn do_map(
    mut maptype: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut is_abbrev: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut parsed_args: MapArguments = MapArguments {
            buffer: false,
            expr: false,
            noremap: false,
            nowait: false,
            script: false,
            silent: false,
            unique: false,
            replace_keycodes: false,
            lhs: [0; 51],
            lhs_len: 0,
            alt_lhs: [0; 51],
            alt_lhs_len: 0,
            rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            rhs_len: 0,
            rhs_lua: 0,
            rhs_is_noop: false,
            orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            orig_rhs_len: 0,
            desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut result: ::core::ffi::c_int = str_to_mapargs(
            arg,
            maptype == MAPTYPE_UNMAP as ::core::ffi::c_int,
            &raw mut parsed_args,
        );
        match result {
            0 => {
                result = buf_do_map(maptype, &raw mut parsed_args, mode, is_abbrev, curbuf.get());
            }
            1 => {}
            _ => {
                '_c2rust_label: {
                    if false {
                    } else {
                        __assert_fail(
                            b"false && \"Unknown return code from str_to_mapargs!\"\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            968 as ::core::ffi::c_uint,
                            b"int do_map(int, char *, int, _Bool)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                result = -1 as ::core::ffi::c_int;
            }
        }
        xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
        xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
        return result;
    }
}

pub(crate) unsafe extern "C" fn do_mapclear(
    mut cmdp: *mut ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
    mut abbr: ::core::ffi::c_int,
) {
    unsafe {
        let mut local: bool = strcmp(arg, b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int;
        if !local && *arg as ::core::ffi::c_int != NUL {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let mut mode: ::core::ffi::c_int = get_map_mode(&raw mut cmdp, forceit != 0);
        map_clear_mode(curbuf.get(), mode, local, abbr != 0);
    }
}

pub unsafe extern "C" fn add_map(
    mut lhs: *mut ::core::ffi::c_char,
    mut rhs: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut buffer: bool,
) {
    unsafe {
        let mut args: MapArguments = MAP_ARGUMENTS_INIT;
        set_maparg_lhs_rhs(
            lhs,
            strlen(lhs),
            rhs,
            strlen(rhs),
            LUA_NOREF,
            p_cpo.get(),
            &raw mut args,
        );
        args.buffer = buffer;
        buf_do_map(
            MAPTYPE_NOREMAP as ::core::ffi::c_int,
            &raw mut args,
            mode,
            false_0 != 0,
            curbuf.get(),
        );
        xfree(args.rhs as *mut ::core::ffi::c_void);
        xfree(args.orig_rhs as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn do_exmap(mut eap: *mut exarg_T, mut isabbrev: ::core::ffi::c_int) {
    unsafe {
        let mut cmdp: *mut ::core::ffi::c_char = (*eap).cmd;
        let mut mode: ::core::ffi::c_int =
            get_map_mode(&raw mut cmdp, (*eap).forceit != 0 || isabbrev != 0);
        let mut maptype: ::core::ffi::c_int = 0;
        if *cmdp as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            maptype = MAPTYPE_NOREMAP as ::core::ffi::c_int;
        } else if *cmdp as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
            maptype = MAPTYPE_UNMAP as ::core::ffi::c_int;
        } else {
            maptype = MAPTYPE_MAP as ::core::ffi::c_int;
        }
        let mut parsed_args: MapArguments = MapArguments {
            buffer: false,
            expr: false,
            noremap: false,
            nowait: false,
            script: false,
            silent: false,
            unique: false,
            replace_keycodes: false,
            lhs: [0; 51],
            lhs_len: 0,
            alt_lhs: [0; 51],
            alt_lhs_len: 0,
            rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            rhs_len: 0,
            rhs_lua: 0,
            rhs_is_noop: false,
            orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            orig_rhs_len: 0,
            desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut result: ::core::ffi::c_int = str_to_mapargs(
            (*eap).arg,
            maptype == MAPTYPE_UNMAP as ::core::ffi::c_int,
            &raw mut parsed_args,
        );
        match result {
            0 => match buf_do_map(
                maptype,
                &raw mut parsed_args,
                mode,
                isabbrev != 0,
                curbuf.get(),
            ) {
                1 => {
                    emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                }
                2 => {
                    emsg(if isabbrev != 0 {
                        gettext(&raw const e_noabbr as *const ::core::ffi::c_char)
                    } else {
                        gettext(&raw const e_nomap as *const ::core::ffi::c_char)
                    });
                }
                5 => {
                    semsg(
                        if isabbrev != 0 {
                            gettext(
                                (e_abbreviation_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            )
                        } else {
                            gettext(
                                (e_mapping_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            )
                        },
                        &raw mut parsed_args.lhs as *mut ::core::ffi::c_char,
                    );
                }
                6 => {
                    semsg(
                        if isabbrev != 0 {
                            gettext(
                                (e_global_abbreviation_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            )
                        } else {
                            gettext(
                                (e_global_mapping_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            )
                        },
                        &raw mut parsed_args.lhs as *mut ::core::ffi::c_char,
                    );
                }
                _ => {}
            },
            1 => {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            }
            _ => {
                '_c2rust_label: {
                    if false {
                    } else {
                        __assert_fail(
                            b"false && \"Unknown return code from str_to_mapargs!\"\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2669 as ::core::ffi::c_uint,
                            b"void do_exmap(exarg_T *, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
            }
        }
        xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
        xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
    }
}

pub unsafe fn ex_abbreviate(mut eap: *mut exarg_T) {
    unsafe {
        do_exmap(eap, true_0);
    }
}

pub unsafe fn ex_map(mut eap: *mut exarg_T) {
    unsafe {
        if secure.get() != 0 {
            secure.set(2 as ::core::ffi::c_int);
            msg_outtrans((*eap).cmd, 0 as ::core::ffi::c_int, false_0 != 0);
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        do_exmap(eap, false_0);
    }
}

pub unsafe fn ex_unmap(mut eap: *mut exarg_T) {
    unsafe {
        do_exmap(eap, false_0);
    }
}

pub unsafe fn ex_mapclear(mut eap: *mut exarg_T) {
    unsafe {
        do_mapclear((*eap).cmd, (*eap).arg, (*eap).forceit, false_0);
    }
}

pub unsafe fn ex_abclear(mut eap: *mut exarg_T) {
    unsafe {
        do_mapclear((*eap).cmd, (*eap).arg, true_0, true_0);
    }
}
