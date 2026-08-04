//! The mapping table itself: the hash buckets and the abbrlist.
//!
//! Every mapping is a [`mapblock_T`] on one of `MAX_MAPHASH` singly linked
//! lists, hashed on the first byte of its LHS and on whether the mode is a
//! Normal-side or an Insert-side one; abbreviations live on one unhashed
//! list instead.  The functions here create ([`map_add`]), destroy
//! ([`mapblock_free`], [`map_clear_mode`]) and search
//! ([`check_map`], [`map_to_exists_mode`]) those lists.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_maphash_list(
    mut state: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *mut mapblock_T {
    unsafe {
        return (*maphash.ptr())[(if state
            & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL)
            != 0
        {
            c
        } else {
            c ^ 0x80 as ::core::ffi::c_int
        }) as usize] as *mut mapblock_T;
    }
}

pub unsafe extern "C" fn get_buf_maphash_list(
    mut state: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *mut mapblock_T {
    unsafe {
        return (*curbuf.get()).b_maphash[(if state
            & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL)
            != 0
        {
            c
        } else {
            c ^ 0x80 as ::core::ffi::c_int
        }) as usize] as *mut mapblock_T;
    }
}

pub(crate) unsafe extern "C" fn mapblock_free(mut mpp: *mut *mut mapblock_T) {
    unsafe {
        let mut mp: *mut mapblock_T = *mpp;
        xfree((*mp).m_keys as *mut ::core::ffi::c_void);
        if !(*mp).m_alt.is_null() {
            (*(*mp).m_alt).m_alt = ::core::ptr::null_mut::<mapblock_T>();
        } else {
            if (*mp).m_luaref != LUA_NOREF {
                api_free_luaref((*mp).m_luaref);
                (*mp).m_luaref = LUA_NOREF as LuaRef;
            }
            xfree((*mp).m_str as *mut ::core::ffi::c_void);
            xfree((*mp).m_orig_str as *mut ::core::ffi::c_void);
            xfree((*mp).m_desc as *mut ::core::ffi::c_void);
        }
        *mpp = (*mp).m_next;
        xfree(mp as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn map_add(
    mut buf: *mut buf_T,
    mut map_table: *mut *mut mapblock_T,
    mut abbr_table: *mut *mut mapblock_T,
    mut keys: *const ::core::ffi::c_char,
    mut args: *mut MapArguments,
    mut noremap: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut is_abbr: bool,
    mut sid: scid_T,
    mut lnum: linenr_T,
    mut simplified: bool,
) -> *mut mapblock_T {
    unsafe {
        let mut mp: *mut mapblock_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<mapblock_T>()) as *mut mapblock_T;
        if *keys as ::core::ffi::c_int == Ctrl_C {
            if map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T {
                (*buf).b_mapped_ctrl_c |= mode;
            } else {
                (*mapped_ctrl_c.ptr()) |= mode;
            }
        }
        (*mp).m_keys = xstrdup(keys);
        (*mp).m_str = (*args).rhs;
        (*mp).m_orig_str = (*args).orig_rhs;
        (*mp).m_luaref = (*args).rhs_lua;
        (*mp).m_keylen = strlen((*mp).m_keys) as ::core::ffi::c_int;
        (*mp).m_noremap = noremap;
        (*mp).m_nowait = (*args).nowait as ::core::ffi::c_char;
        (*mp).m_silent = (*args).silent as ::core::ffi::c_char;
        (*mp).m_mode = mode;
        (*mp).m_simplified = simplified as ::core::ffi::c_int;
        (*mp).m_expr = (*args).expr as ::core::ffi::c_char;
        (*mp).m_replace_keycodes = (*args).replace_keycodes;
        if sid != 0 as ::core::ffi::c_int {
            (*mp).m_script_ctx.sc_sid = sid;
            (*mp).m_script_ctx.sc_lnum = lnum;
        } else {
            (*mp).m_script_ctx = current_sctx.get();
            (*mp).m_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum;
            nlua_set_sctx(&raw mut (*mp).m_script_ctx);
        }
        (*mp).m_desc = (*args).desc;
        if is_abbr {
            (*mp).m_next = *abbr_table;
            *abbr_table = mp;
        } else {
            let n: ::core::ffi::c_int = if (*mp).m_mode
                & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL)
                != 0
            {
                *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int
            } else {
                *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int
                    ^ 0x80 as ::core::ffi::c_int
            };
            (*mp).m_next = *map_table.offset(n as isize);
            *map_table.offset(n as isize) = mp;
        }
        return mp;
    }
}

pub unsafe extern "C" fn map_clear_mode(
    mut buf: *mut buf_T,
    mut mode: ::core::ffi::c_int,
    mut local: bool,
    mut abbr: bool,
) {
    unsafe {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mpp: *mut *mut mapblock_T = ::core::ptr::null_mut::<*mut mapblock_T>();
            if abbr {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if local {
                    mpp = &raw mut (*buf).b_first_abbr;
                } else {
                    mpp = first_abbr.ptr();
                }
            } else if local {
                mpp = (&raw mut (*buf).b_maphash as *mut *mut mapblock_T).offset(hash as isize)
                    as *mut *mut mapblock_T;
            } else {
                mpp = (maphash.ptr() as *mut *mut mapblock_T).offset(hash as isize)
                    as *mut *mut mapblock_T;
            }
            while !(*mpp).is_null() {
                let mut mp: *mut mapblock_T = *mpp;
                if (*mp).m_mode & mode != 0 {
                    (*mp).m_mode &= !mode;
                    if (*mp).m_mode == 0 as ::core::ffi::c_int {
                        mapblock_free(mpp);
                        continue;
                    } else {
                        let mut new_hash: ::core::ffi::c_int = if (*mp).m_mode
                            & (MODE_NORMAL
                                | MODE_VISUAL
                                | MODE_SELECT
                                | MODE_OP_PENDING
                                | MODE_TERMINAL)
                            != 0
                        {
                            *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                        } else {
                            *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                ^ 0x80 as ::core::ffi::c_int
                        };
                        if !abbr && new_hash != hash {
                            *mpp = (*mp).m_next;
                            if local {
                                (*mp).m_next =
                                    (*buf).b_maphash[new_hash as usize] as *mut mapblock_T;
                                (*buf).b_maphash[new_hash as usize] = mp as *mut mapblock_T;
                            } else {
                                (*mp).m_next =
                                    (*maphash.ptr())[new_hash as usize] as *mut mapblock_T;
                                (*maphash.ptr())[new_hash as usize] = mp as *mut mapblock_T;
                            }
                            continue;
                        }
                    }
                }
                mpp = &raw mut (*mp).m_next;
            }
            hash += 1;
        }
    }
}

pub unsafe extern "C" fn map_to_exists(
    str: *const ::core::ffi::c_char,
    modechars: *const ::core::ffi::c_char,
    abbr: bool,
) -> bool {
    unsafe {
        let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let rhs: *const ::core::ffi::c_char = replace_termcodes(
            str,
            strlen(str),
            &raw mut buf,
            0 as scid_T,
            REPTERM_DO_LT as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
            p_cpo.get(),
        );
        if !strchr(modechars, 'n' as ::core::ffi::c_int).is_null() {
            mode |= MODE_NORMAL;
        }
        if !strchr(modechars, 'v' as ::core::ffi::c_int).is_null() {
            mode |= MODE_VISUAL | MODE_SELECT;
        }
        if !strchr(modechars, 'x' as ::core::ffi::c_int).is_null() {
            mode |= MODE_VISUAL;
        }
        if !strchr(modechars, 's' as ::core::ffi::c_int).is_null() {
            mode |= MODE_SELECT;
        }
        if !strchr(modechars, 'o' as ::core::ffi::c_int).is_null() {
            mode |= MODE_OP_PENDING;
        }
        if !strchr(modechars, 'i' as ::core::ffi::c_int).is_null() {
            mode |= MODE_INSERT;
        }
        if !strchr(modechars, 'l' as ::core::ffi::c_int).is_null() {
            mode |= MODE_LANGMAP;
        }
        if !strchr(modechars, 'c' as ::core::ffi::c_int).is_null() {
            mode |= MODE_CMDLINE;
        }
        let mut retval: bool = map_to_exists_mode(rhs, mode, abbr);
        xfree(buf as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn map_to_exists_mode(
    rhs: *const ::core::ffi::c_char,
    mode: ::core::ffi::c_int,
    abbr: bool,
) -> bool {
    unsafe {
        let mut exp_buffer: bool = false_0 != 0;
        loop {
            let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while hash < 256 as ::core::ffi::c_int {
                let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                if abbr {
                    if hash > 0 as ::core::ffi::c_int {
                        break;
                    }
                    if exp_buffer {
                        mp = (*curbuf.get()).b_first_abbr;
                    } else {
                        mp = first_abbr.get();
                    }
                } else if exp_buffer {
                    mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
                } else {
                    mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
                }
                while !mp.is_null() {
                    if (*mp).m_mode & mode != 0 && !strstr((*mp).m_str, rhs).is_null() {
                        return true_0 != 0;
                    }
                    mp = (*mp).m_next;
                }
                hash += 1;
            }
            if exp_buffer {
                break;
            }
            exp_buffer = true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn check_map(
    mut keys: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut exact: ::core::ffi::c_int,
    mut ign_mod: ::core::ffi::c_int,
    mut abbr: ::core::ffi::c_int,
    mut mp_ptr: *mut *mut mapblock_T,
    mut local_ptr: *mut ::core::ffi::c_int,
    mut rhs_lua: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        *rhs_lua = LUA_NOREF;
        let mut len: ::core::ffi::c_int = strlen(keys) as ::core::ffi::c_int;
        let mut local: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while local >= 0 as ::core::ffi::c_int {
            let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while hash < 256 as ::core::ffi::c_int {
                let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                if abbr != 0 {
                    if hash > 0 as ::core::ffi::c_int {
                        break;
                    }
                    if local != 0 {
                        mp = (*curbuf.get()).b_first_abbr;
                    } else {
                        mp = first_abbr.get();
                    }
                } else if local != 0 {
                    mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
                } else {
                    mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
                }
                while !mp.is_null() {
                    if (*mp).m_mode & mode != 0 && (exact == 0 || (*mp).m_keylen == len) {
                        let mut s: *mut ::core::ffi::c_char = (*mp).m_keys;
                        let mut keylen: ::core::ffi::c_int = (*mp).m_keylen;
                        if ign_mod != 0
                            && keylen >= 3 as ::core::ffi::c_int
                            && *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                == K_SPECIAL
                            && *s.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                == KS_MODIFIER
                        {
                            s = s.offset(3 as ::core::ffi::c_int as isize);
                            keylen -= 3 as ::core::ffi::c_int;
                        }
                        let mut minlen: ::core::ffi::c_int =
                            if keylen < len { keylen } else { len };
                        if strncmp(s, keys, minlen as size_t) == 0 as ::core::ffi::c_int {
                            if !mp_ptr.is_null() {
                                *mp_ptr = mp;
                            }
                            if !local_ptr.is_null() {
                                *local_ptr = local;
                            }
                            *rhs_lua = (*mp).m_luaref as ::core::ffi::c_int;
                            return if (*mp).m_luaref == LUA_NOREF {
                                (*mp).m_str
                            } else {
                                ::core::ptr::null_mut::<::core::ffi::c_char>()
                            };
                        }
                    }
                    mp = (*mp).m_next;
                }
                hash += 1;
            }
            local -= 1;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}
