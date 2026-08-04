//! Matching the typeahead against the mapping table.
//!
//! [`handle_mapping`] is the heart of it: it walks the maphash bucket for the
//! first typeahead byte looking for the longest mapping whose LHS is a prefix
//! of what is buffered, decides between waiting for more input and giving up
//! (`'timeout'`/`'timeoutlen'`), and on a match replaces the LHS with the RHS
//! in the typeahead.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn put_string_in_typebuf(
    mut offset: ::core::ffi::c_int,
    mut slen: ::core::ffi::c_int,
    mut string: *mut uint8_t,
    mut new_slen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut extra: ::core::ffi::c_int = new_slen - slen;
        *string.offset(new_slen as isize) = NUL as uint8_t;
        if extra < 0 as ::core::ffi::c_int {
            del_typebuf(-extra, offset);
        } else if extra > 0 as ::core::ffi::c_int {
            if ins_typebuf(
                (string as *mut ::core::ffi::c_char).offset(slen as isize),
                REMAP_YES as ::core::ffi::c_int,
                offset,
                false_0 != 0,
                false_0 != 0,
            ) == FAIL
            {
                return FAIL;
            }
        }
        memmove(
            (*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize)
                .offset(offset as isize) as *mut ::core::ffi::c_void,
            string as *const ::core::ffi::c_void,
            new_slen as size_t,
        );
        return OK;
    }
}

pub(crate) unsafe extern "C" fn at_ins_compl_key() -> bool {
    unsafe {
        let mut p: *mut uint8_t = (*typebuf.ptr())
            .tb_buf
            .offset((*typebuf.ptr()).tb_off as isize);
        let mut c: ::core::ffi::c_int = *p as ::core::ffi::c_int;
        if (*typebuf.ptr()).tb_len > 3 as ::core::ffi::c_int
            && c == K_SPECIAL
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int & MOD_MASK_CTRL
                != 0
        {
            c = *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & 0x1f as ::core::ffi::c_int;
        }
        return ctrl_x_mode_not_default() as ::core::ffi::c_int != 0
            && vim_is_ctrl_x_key(c) as ::core::ffi::c_int != 0
            || compl_status_local() as ::core::ffi::c_int != 0 && (c == Ctrl_N || c == Ctrl_P);
    }
}

pub(crate) unsafe extern "C" fn check_simplify_modifier(
    mut max_offset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if State.get() & MODE_TERMINAL != 0 || no_reduce_keys.get() > 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while offset < max_offset {
            if offset + 3 as ::core::ffi::c_int >= (*typebuf.ptr()).tb_len {
                break;
            }
            let mut tp: *mut uint8_t = (*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize)
                .offset(offset as isize);
            if *tp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
                && *tp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
            {
                let mut modifier: ::core::ffi::c_int =
                    *tp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
                let mut c: ::core::ffi::c_int =
                    *tp.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
                let mut new_c: ::core::ffi::c_int = merge_modifiers(c, &raw mut modifier);
                if new_c != c {
                    if offset == 0 as ::core::ffi::c_int {
                        vgetc_char.set(c);
                        vgetc_mod_mask
                        .set(*tp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
                    }
                    let mut new_string: [uint8_t; 21] = [0; 21];
                    let mut len: ::core::ffi::c_int = 0;
                    if new_c < 0 as ::core::ffi::c_int {
                        new_string[0 as ::core::ffi::c_int as usize] = K_SPECIAL as uint8_t;
                        new_string[1 as ::core::ffi::c_int as usize] = (if new_c == K_SPECIAL {
                            KS_SPECIAL
                        } else if new_c == NUL {
                            KS_ZERO
                        } else {
                            -new_c & 0xff as ::core::ffi::c_int
                        })
                            as uint8_t;
                        new_string[2 as ::core::ffi::c_int as usize] =
                            (if new_c == K_SPECIAL || new_c == NUL {
                                KE_FILLER as ::core::ffi::c_uint
                            } else {
                                -new_c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                                    & 0xff as ::core::ffi::c_uint
                            }) as uint8_t;
                        len = 3 as ::core::ffi::c_int;
                    } else {
                        len = utf_char2bytes(
                            new_c,
                            &raw mut new_string as *mut uint8_t as *mut ::core::ffi::c_char,
                        );
                    }
                    if modifier == 0 as ::core::ffi::c_int {
                        if put_string_in_typebuf(
                            offset,
                            4 as ::core::ffi::c_int,
                            &raw mut new_string as *mut uint8_t,
                            len,
                        ) == FAIL
                        {
                            return -1 as ::core::ffi::c_int;
                        }
                    } else {
                        *tp.offset(2 as ::core::ffi::c_int as isize) = modifier as uint8_t;
                        if put_string_in_typebuf(
                            offset + 3 as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                            &raw mut new_string as *mut uint8_t,
                            len,
                        ) == FAIL
                        {
                            return -1 as ::core::ffi::c_int;
                        }
                    }
                    return len;
                }
            }
            offset += 1;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn handle_mapping(
    mut keylenp: *mut ::core::ffi::c_int,
    mut timedout: *const bool,
    mut mapdepth: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
        let mut mp2: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
        let mut mp_match: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
        let mut mp_match_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut max_mlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut keylen: ::core::ffi::c_int = *keylenp;
        let mut local_State: ::core::ffi::c_int = get_real_state();
        let mut is_plug_map: bool = false_0 != 0;
        if (*typebuf.ptr()).tb_len >= 3 as ::core::ffi::c_int
            && *(*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize) as ::core::ffi::c_int
                == K_SPECIAL
            && *(*typebuf.ptr())
                .tb_buf
                .offset(((*typebuf.ptr()).tb_off + 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == KS_EXTRA
            && *(*typebuf.ptr())
                .tb_buf
                .offset(((*typebuf.ptr()).tb_off + 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == KE_PLUG as ::core::ffi::c_int
        {
            is_plug_map = true_0 != 0;
        }
        let mut tb_c1: ::core::ffi::c_int = *(*typebuf.ptr())
            .tb_buf
            .offset((*typebuf.ptr()).tb_off as isize)
            as ::core::ffi::c_int;
        if no_mapping.get() == 0 as ::core::ffi::c_int
            && (no_zero_mapping.get() == 0 as ::core::ffi::c_int
                || tb_c1 != '0' as ::core::ffi::c_int)
            && ((*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int
                || is_plug_map as ::core::ffi::c_int != 0
                || *(*typebuf.ptr())
                    .tb_noremap
                    .offset((*typebuf.ptr()).tb_off as isize)
                    as ::core::ffi::c_int
                    & (RM_NONE as ::core::ffi::c_int | RM_ABBR as ::core::ffi::c_int)
                    == 0)
            && !(p_paste.get() != 0 && State.get() & (MODE_INSERT | MODE_CMDLINE) != 0)
            && !(State.get() == MODE_HITRETURN
                && (tb_c1 == CAR || tb_c1 == ' ' as ::core::ffi::c_int))
            && State.get() != MODE_ASKMORE
            && !at_ins_compl_key()
        {
            let mut mlen: ::core::ffi::c_int = 0;
            let mut nolmaplen: ::core::ffi::c_int = 0;
            if tb_c1 == K_SPECIAL {
                nolmaplen = 2 as ::core::ffi::c_int;
            } else {
                if *p_langmap.get() as ::core::ffi::c_int != 0
                    && (State.get() & (MODE_CMDLINE | MODE_INSERT) == 0 as ::core::ffi::c_int
                        && get_real_state() != MODE_SELECT)
                    && (p_lrm.get() != 0
                        || (if vgetc_busy.get() != 0 {
                            (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                        } else {
                            KeyTyped.get() as ::core::ffi::c_int
                        }) != 0)
                    && KeyStuffed.get() == 0
                    && tb_c1 >= 0 as ::core::ffi::c_int
                {
                    if tb_c1 < 256 as ::core::ffi::c_int {
                        tb_c1 = (*langmap_mapchar.ptr())[tb_c1 as usize] as ::core::ffi::c_int;
                    } else {
                        tb_c1 = langmap_adjust_mb(tb_c1);
                    }
                }
                nolmaplen = 0 as ::core::ffi::c_int;
            }
            mp = get_buf_maphash_list(local_State, tb_c1);
            mp2 = get_maphash_list(local_State, tb_c1);
            if mp.is_null() {
                mp = mp2;
                mp2 = ::core::ptr::null_mut::<mapblock_T>();
            }
            mp_match = ::core::ptr::null_mut::<mapblock_T>();
            mp_match_len = 0 as ::core::ffi::c_int;
            while !mp.is_null() {
                if *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int
                    == tb_c1
                    && (*mp).m_mode & local_State != 0
                    && ((*mp).m_mode & MODE_LANGMAP == 0 as ::core::ffi::c_int
                        || (*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int)
                {
                    let mut nomap: ::core::ffi::c_int = nolmaplen;
                    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    mlen = 1 as ::core::ffi::c_int;
                    while mlen < (*typebuf.ptr()).tb_len {
                        let mut c2: ::core::ffi::c_int = *(*typebuf.ptr())
                            .tb_buf
                            .offset(((*typebuf.ptr()).tb_off + mlen) as isize)
                            as ::core::ffi::c_int;
                        if nomap > 0 as ::core::ffi::c_int {
                            if nomap == 2 as ::core::ffi::c_int && c2 == KS_MODIFIER {
                                modifiers = 1 as ::core::ffi::c_int;
                            } else if nomap == 1 as ::core::ffi::c_int
                                && modifiers == 1 as ::core::ffi::c_int
                            {
                                modifiers = c2;
                            }
                            nomap -= 1;
                        } else {
                            if c2 == K_SPECIAL {
                                nomap = 2 as ::core::ffi::c_int;
                            } else if merge_modifiers(c2, &raw mut modifiers) == c2 {
                                if *p_langmap.get() as ::core::ffi::c_int != 0
                                    && true
                                    && (p_lrm.get() != 0
                                        || (if vgetc_busy.get() != 0 {
                                            (typebuf_maplen() == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        } else {
                                            KeyTyped.get() as ::core::ffi::c_int
                                        }) != 0)
                                    && KeyStuffed.get() == 0
                                    && c2 >= 0 as ::core::ffi::c_int
                                {
                                    if c2 < 256 as ::core::ffi::c_int {
                                        c2 = (*langmap_mapchar.ptr())[c2 as usize]
                                            as ::core::ffi::c_int;
                                    } else {
                                        c2 = langmap_adjust_mb(c2);
                                    }
                                }
                            }
                            modifiers = 0 as ::core::ffi::c_int;
                        }
                        if *(*mp).m_keys.offset(mlen as isize) as uint8_t as ::core::ffi::c_int
                            != c2
                        {
                            break;
                        }
                        mlen += 1;
                    }
                    let mut p1: *const ::core::ffi::c_char = (*mp).m_keys;
                    let mut p2: *const ::core::ffi::c_char = mb_unescape(&raw mut p1);
                    if !p2.is_null()
                        && (*utf8len_tab.ptr())[tb_c1 as usize] as ::core::ffi::c_int
                            > utfc_ptr2len(p2)
                    {
                        mlen = 0 as ::core::ffi::c_int;
                    }
                    keylen = (*mp).m_keylen;
                    if mlen == keylen
                        || mlen == (*typebuf.ptr()).tb_len && (*typebuf.ptr()).tb_len < keylen
                    {
                        let mut n: ::core::ffi::c_int = 0;
                        let mut s: *mut uint8_t = (*typebuf.ptr())
                            .tb_noremap
                            .offset((*typebuf.ptr()).tb_off as isize);
                        if !(*s as ::core::ffi::c_int == RM_SCRIPT as ::core::ffi::c_int
                            && (*(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                != K_SPECIAL
                                || *(*mp).m_keys.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int
                                    != KS_EXTRA
                                || *(*mp).m_keys.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != KE_SNR as ::core::ffi::c_int))
                        {
                            n = mlen;
                            loop {
                                n -= 1;
                                if n < 0 as ::core::ffi::c_int {
                                    break;
                                }
                                let c2rust_fresh9 = s;
                                s = s.offset(1);
                                if *c2rust_fresh9 as ::core::ffi::c_int
                                    & (RM_NONE as ::core::ffi::c_int
                                        | RM_ABBR as ::core::ffi::c_int)
                                    != 0
                                {
                                    break;
                                }
                            }
                            if !(!is_plug_map && n >= 0 as ::core::ffi::c_int) {
                                if keylen > (*typebuf.ptr()).tb_len {
                                    if !*timedout
                                        && !(!mp_match.is_null()
                                            && (*mp_match).m_nowait as ::core::ffi::c_int != 0)
                                    {
                                        keylen = KEYLEN_PART_MAP as ::core::ffi::c_int;
                                        break;
                                    }
                                } else if keylen > mp_match_len
                                    || keylen == mp_match_len
                                        && !mp_match.is_null()
                                        && (*mp_match).m_mode & MODE_LANGMAP
                                            == 0 as ::core::ffi::c_int
                                        && (*mp).m_mode & MODE_LANGMAP != 0 as ::core::ffi::c_int
                                {
                                    mp_match = mp;
                                    mp_match_len = keylen;
                                }
                            }
                        }
                    } else {
                        max_mlen = if max_mlen > mlen { max_mlen } else { mlen };
                    }
                }
                if (*mp).m_next.is_null() {
                    mp = mp2;
                    mp2 = ::core::ptr::null_mut::<mapblock_T>();
                } else {
                    mp = (*mp).m_next;
                };
            }
            if keylen != KEYLEN_PART_MAP as ::core::ffi::c_int && !mp_match.is_null() {
                mp = mp_match;
                keylen = mp_match_len;
            }
        }
        if (mp.is_null() || max_mlen > mp_match_len)
            && keylen != KEYLEN_PART_MAP as ::core::ffi::c_int
        {
            if no_mapping.get() == 0 as ::core::ffi::c_int
                || allow_keys.get() != 0 as ::core::ffi::c_int
            {
                if tb_c1 == K_SPECIAL
                    && ((*typebuf.ptr()).tb_len < 2 as ::core::ffi::c_int
                        || *(*typebuf.ptr())
                            .tb_buf
                            .offset(((*typebuf.ptr()).tb_off + 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == KS_MODIFIER
                            && (*typebuf.ptr()).tb_len < 4 as ::core::ffi::c_int)
                {
                    keylen = KEYLEN_PART_KEY as ::core::ffi::c_int;
                } else {
                    keylen = check_simplify_modifier(max_mlen + 1 as ::core::ffi::c_int);
                    if keylen < 0 as ::core::ffi::c_int {
                        return map_result_fail as ::core::ffi::c_int;
                    }
                }
            } else {
                keylen = 0 as ::core::ffi::c_int;
            }
            if keylen == 0 as ::core::ffi::c_int {
                if mp.is_null() {
                    *keylenp = keylen;
                    return map_result_get as ::core::ffi::c_int;
                }
            }
            if keylen > 0 as ::core::ffi::c_int {
                *keylenp = keylen;
                return map_result_retry as ::core::ffi::c_int;
            }
            if keylen < 0 as ::core::ffi::c_int {
                '_c2rust_label: {
                    if keylen == KEYLEN_PART_KEY as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"keylen == KEYLEN_PART_KEY\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2385 as ::core::ffi::c_uint,
                            b"int handle_mapping(int *, const _Bool *, int *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
            } else {
                '_c2rust_label_0: {
                    if !mp.is_null() {
                    } else {
                        __assert_fail(
                            b"mp != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2387 as ::core::ffi::c_uint,
                            b"int handle_mapping(int *, const _Bool *, int *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                keylen = mp_match_len;
            }
        }
        if keylen >= 0 as ::core::ffi::c_int && keylen <= (*typebuf.ptr()).tb_len {
            let mut i: ::core::ffi::c_int = 0;
            let mut map_str: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if keylen > (*typebuf.ptr()).tb_maplen
                && (*mp).m_mode & MODE_LANGMAP == 0 as ::core::ffi::c_int
            {
                gotchars(
                    (*typebuf.ptr())
                        .tb_buf
                        .offset((*typebuf.ptr()).tb_off as isize)
                        .offset((*typebuf.ptr()).tb_maplen as isize),
                    (keylen - (*typebuf.ptr()).tb_maplen) as size_t,
                );
            }
            cmd_silent.set((*typebuf.ptr()).tb_silent > 0 as ::core::ffi::c_int);
            del_typebuf(keylen, 0 as ::core::ffi::c_int);
            *mapdepth += 1;
            if *mapdepth as OptInt >= p_mmd.get() {
                emsg(gettext(
                    (e_recursive_mapping.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                if State.get() & MODE_CMDLINE != 0 {
                    redrawcmdline();
                } else {
                    setcursor();
                }
                flush_buffers(FLUSH_MINIMAL);
                *mapdepth = 0 as ::core::ffi::c_int;
                *keylenp = keylen;
                return map_result_fail as ::core::ffi::c_int;
            }
            if VIsual_active.get() as ::core::ffi::c_int != 0
                && VIsual_select.get() as ::core::ffi::c_int != 0
                && (*mp).m_mode & MODE_VISUAL != 0
            {
                VIsual_select.set(false_0 != 0);
                ins_typebuf(
                    K_SELECT_STRING.as_ptr() as *mut ::core::ffi::c_char,
                    REMAP_NONE as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    false_0 != 0,
                );
            }
            let save_m_expr: bool = (*mp).m_expr != 0;
            let save_m_noremap: ::core::ffi::c_int = (*mp).m_noremap;
            let save_m_silent: bool = (*mp).m_silent != 0;
            let mut save_m_keys: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut save_alt_m_keys: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let save_alt_m_keylen: ::core::ffi::c_int = if !(*mp).m_alt.is_null() {
                (*(*mp).m_alt).m_keylen
            } else {
                0 as ::core::ffi::c_int
            };
            if (*mp).m_expr != 0 {
                let save_vgetc_busy: ::core::ffi::c_int = vgetc_busy.get();
                let save_may_garbage_collect: bool = may_garbage_collect.get();
                let prev_did_emsg: ::core::ffi::c_int = did_emsg.get();
                vgetc_busy.set(0 as ::core::ffi::c_int);
                may_garbage_collect.set(false_0 != 0);
                save_m_keys = xmemdupz(
                    (*mp).m_keys as *const ::core::ffi::c_void,
                    (*mp).m_keylen as size_t,
                ) as *mut ::core::ffi::c_char;
                save_alt_m_keys = (if !(*mp).m_alt.is_null() {
                    xmemdupz(
                        (*(*mp).m_alt).m_keys as *const ::core::ffi::c_void,
                        save_alt_m_keylen as size_t,
                    )
                } else {
                    NULL_0
                }) as *mut ::core::ffi::c_char;
                map_str = eval_map_expr(mp, NUL);
                if map_str.is_null() || *map_str as ::core::ffi::c_int == NUL {
                    if prev_did_emsg != did_emsg.get() {
                        let mut buf: [::core::ffi::c_char; 4] = [0; 4];
                        xfree(map_str as *mut ::core::ffi::c_void);
                        buf[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
                        buf[1 as ::core::ffi::c_int as usize] = KS_EXTRA as ::core::ffi::c_char;
                        buf[2 as ::core::ffi::c_int as usize] =
                            KE_IGNORE as ::core::ffi::c_int as ::core::ffi::c_char;
                        buf[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                        map_str = xmemdupz(
                            &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                            3 as size_t,
                        ) as *mut ::core::ffi::c_char;
                        if State.get() & MODE_CMDLINE != 0 {
                            msg_didout.set(true_0 != 0);
                            msg_row.set(if msg_row.get() > cmdline_row.get() {
                                msg_row.get()
                            } else {
                                cmdline_row.get()
                            });
                            redrawcmd();
                        }
                    } else if State.get() & (MODE_NORMAL | MODE_INSERT) != 0 {
                        setcursor();
                    }
                }
                vgetc_busy.set(save_vgetc_busy);
                may_garbage_collect.set(save_may_garbage_collect);
            } else {
                map_str = (*mp).m_str;
            }
            if map_str.is_null() {
                i = FAIL;
            } else {
                let mut noremap: ::core::ffi::c_int = 0;
                if keylen > (*typebuf.ptr()).tb_maplen
                    && (*mp).m_mode & MODE_LANGMAP != 0 as ::core::ffi::c_int
                {
                    gotchars(map_str as *mut uint8_t, strlen(map_str));
                }
                if save_m_noremap != REMAP_YES as ::core::ffi::c_int {
                    noremap = save_m_noremap;
                } else if if save_m_expr as ::core::ffi::c_int != 0 {
                    (strncmp(map_str, save_m_keys, keylen as size_t) == 0 as ::core::ffi::c_int
                        || !save_alt_m_keys.is_null()
                            && strncmp(map_str, save_alt_m_keys, save_alt_m_keylen as size_t)
                                == 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                } else {
                    (strncmp(map_str, (*mp).m_keys, keylen as size_t) == 0 as ::core::ffi::c_int
                        || !(*mp).m_alt.is_null()
                            && strncmp(
                                map_str,
                                (*(*mp).m_alt).m_keys,
                                (*(*mp).m_alt).m_keylen as size_t,
                            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } != 0
                {
                    noremap = REMAP_SKIP as ::core::ffi::c_int;
                } else {
                    noremap = REMAP_YES as ::core::ffi::c_int;
                }
                i = ins_typebuf(
                    map_str,
                    noremap,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    cmd_silent.get() as ::core::ffi::c_int != 0
                        || save_m_silent as ::core::ffi::c_int != 0,
                );
                if save_m_expr {
                    xfree(map_str as *mut ::core::ffi::c_void);
                }
            }
            xfree(save_m_keys as *mut ::core::ffi::c_void);
            xfree(save_alt_m_keys as *mut ::core::ffi::c_void);
            *keylenp = keylen;
            if i == FAIL {
                return map_result_fail as ::core::ffi::c_int;
            }
            return map_result_retry as ::core::ffi::c_int;
        }
        *keylenp = keylen;
        return map_result_nomatch as ::core::ffi::c_int;
    }
}
