//! Abbreviations and `<expr>` right-hand sides.
//!
//! [`check_abbr`] is called after every inserted character: it looks back for
//! a word matching an entry on the abbrlist and, on a match, pushes the
//! deletions and the replacement into the typeahead.  [`eval_map_expr`]
//! evaluates an `<expr>` mapping's RHS, which both this and the mapping match
//! need.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn check_abbr(
    mut c: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut mincol: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut tb: [uint8_t; 25] = [0; 25];
        let mut clen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*typebuf.ptr()).tb_no_abbr_cnt != 0 {
            return false_0 != 0;
        }
        if noremap_keys() as ::core::ffi::c_int != 0 && c != Ctrl_RSB {
            return false_0 != 0;
        }
        if col == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut scol: ::core::ffi::c_int = 0;
        let mut is_id: bool = true_0 != 0;
        let mut vim_abbr: bool = false;
        let mut p: *mut ::core::ffi::c_char = mb_prevptr(ptr, ptr.offset(col as isize));
        if !vim_iswordp(p) {
            vim_abbr = true_0 != 0;
        } else {
            vim_abbr = false_0 != 0;
            if p > ptr {
                is_id = vim_iswordp(mb_prevptr(ptr, p));
            }
        }
        clen = 1 as ::core::ffi::c_int;
        while p > ptr.offset(mincol as isize) {
            p = mb_prevptr(ptr, p);
            if ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                || !vim_abbr && is_id as ::core::ffi::c_int != vim_iswordp(p) as ::core::ffi::c_int
            {
                p = p.offset(utfc_ptr2len(p) as isize);
                break;
            } else {
                clen += 1;
            }
        }
        scol = p.offset_from(ptr) as ::core::ffi::c_int;
        if scol < mincol {
            scol = mincol;
        }
        if scol < col {
            ptr = ptr.offset(scol as isize);
            let mut len: ::core::ffi::c_int = col - scol;
            let mut mp: *mut mapblock_T = (*curbuf.get()).b_first_abbr;
            let mut mp2: *mut mapblock_T = FIRST_ABBR.get();
            if mp.is_null() {
                mp = mp2;
                mp2 = ::core::ptr::null_mut::<mapblock_T>();
            }
            while !mp.is_null() {
                let mut qlen: ::core::ffi::c_int = (*mp).m_keylen;
                let mut q: *mut ::core::ffi::c_char = (*mp).m_keys;
                if !strchr((*mp).m_keys, K_SPECIAL).is_null() {
                    q = xstrdup((*mp).m_keys);
                    vim_unescape_ks(q);
                    qlen = strlen(q) as ::core::ffi::c_int;
                }
                let mut match_0: ::core::ffi::c_int = ((*mp).m_mode & State.get() != 0
                    && qlen == len
                    && strncmp(q, ptr, len as size_t) == 0)
                    as ::core::ffi::c_int;
                if q != (*mp).m_keys {
                    xfree(q as *mut ::core::ffi::c_void);
                }
                if match_0 != 0 {
                    break;
                }
                if (*mp).m_next.is_null() {
                    mp = mp2;
                    mp2 = ::core::ptr::null_mut::<mapblock_T>();
                } else {
                    mp = (*mp).m_next;
                };
            }
            if !mp.is_null() {
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if c != Ctrl_RSB {
                    if c < 0 as ::core::ffi::c_int || c == K_SPECIAL {
                        let c2rust_fresh14 = j;
                        j = j + 1;
                        tb[c2rust_fresh14 as usize] = K_SPECIAL as uint8_t;
                        let c2rust_fresh15 = j;
                        j = j + 1;
                        tb[c2rust_fresh15 as usize] = (if c == K_SPECIAL {
                            KS_SPECIAL
                        } else if c == NUL {
                            KS_ZERO
                        } else {
                            -c & 0xff as ::core::ffi::c_int
                        }) as uint8_t;
                        let c2rust_fresh16 = j;
                        j = j + 1;
                        tb[c2rust_fresh16 as usize] = (if c == K_SPECIAL || c == NUL {
                            KE_FILLER as ::core::ffi::c_uint
                        } else {
                            -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                                & 0xff as ::core::ffi::c_uint
                        }) as uint8_t;
                    } else {
                        if c < ABBR_OFF
                            && (c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int)
                        {
                            let c2rust_fresh17 = j;
                            j = j + 1;
                            tb[c2rust_fresh17 as usize] = Ctrl_V as uint8_t;
                        }
                        if c >= ABBR_OFF {
                            c -= ABBR_OFF;
                        }
                        let mut newlen: ::core::ffi::c_int = utf_char2bytes(
                            c,
                            (&raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char)
                                .offset(j as isize),
                        );
                        tb[(j + newlen) as usize] = NUL as uint8_t;
                        let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(
                            (&raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char)
                                .offset(j as isize),
                        );
                        if !escaped.is_null() {
                            newlen = strlen(escaped) as ::core::ffi::c_int;
                            memmove(
                                (&raw mut tb as *mut uint8_t).offset(j as isize)
                                    as *mut ::core::ffi::c_void,
                                escaped as *const ::core::ffi::c_void,
                                newlen as size_t,
                            );
                            j += newlen;
                            xfree(escaped as *mut ::core::ffi::c_void);
                        }
                    }
                    tb[j as usize] = NUL as uint8_t;
                    ins_typebuf(
                        &raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        true_0 != 0,
                        (*mp).m_silent != 0,
                    );
                }
                let noremap: ::core::ffi::c_int = (*mp).m_noremap;
                let silent: bool = (*mp).m_silent != 0;
                let expr: bool = (*mp).m_expr != 0;
                let mut s: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if expr {
                    s = eval_map_expr(mp, c);
                } else {
                    s = (*mp).m_str;
                }
                if !s.is_null() {
                    ins_typebuf(s, noremap, 0 as ::core::ffi::c_int, true_0 != 0, silent);
                    (*typebuf.ptr()).tb_no_abbr_cnt +=
                        strlen(s) as ::core::ffi::c_int + j + 1 as ::core::ffi::c_int;
                    if expr {
                        xfree(s as *mut ::core::ffi::c_void);
                    }
                }
                tb[0 as ::core::ffi::c_int as usize] = Ctrl_H as uint8_t;
                tb[1 as ::core::ffi::c_int as usize] = NUL as uint8_t;
                len = clen;
                loop {
                    let c2rust_fresh18 = len;
                    len = len - 1;
                    if c2rust_fresh18 <= 0 as ::core::ffi::c_int {
                        break;
                    }
                    ins_typebuf(
                        &raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        true_0 != 0,
                        silent,
                    );
                }
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn eval_map_expr(
    mut mp: *mut mapblock_T,
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut expr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*mp).m_luaref == LUA_NOREF {
            expr = xstrdup((*mp).m_str);
            vim_unescape_ks(expr);
        }
        let replace_keycodes: bool = (*mp).m_replace_keycodes;
        (*expr_map_lock.ptr()) += 1;
        set_vim_var_char(c);
        let save_cursor: pos_T = (*curwin.get()).w_cursor;
        let save_msg_col: ::core::ffi::c_int = msg_col.get();
        let save_msg_row: ::core::ffi::c_int = msg_row.get();
        if (*mp).m_luaref != LUA_NOREF {
            let mut err: Error = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            let mut args: Array = ARRAY_DICT_INIT;
            let mut ret: Object = nlua_call_ref(
                (*mp).m_luaref,
                ::core::ptr::null::<::core::ffi::c_char>(),
                args,
                kRetObject,
                ::core::ptr::null_mut::<Arena>(),
                &raw mut err,
            );
            if ret.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                p = string_to_cstr(ret.data.string);
            }
            api_free_object(ret);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                semsg_multiline(
                    b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
                    b"E5108: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    err.msg,
                );
                api_clear_error(&raw mut err);
            }
        } else {
            p = eval_to_string(expr, false_0 != 0, false_0 != 0);
            xfree(expr as *mut ::core::ffi::c_void);
        }
        (*expr_map_lock.ptr()) -= 1;
        (*curwin.get()).w_cursor = save_cursor;
        msg_col.set(save_msg_col);
        msg_row.set(save_msg_row);
        if p.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if replace_keycodes {
            replace_termcodes(
                p,
                strlen(p),
                &raw mut res,
                0 as scid_T,
                REPTERM_DO_LT as ::core::ffi::c_int,
                ::core::ptr::null_mut::<bool>(),
                p_cpo.get(),
            );
        } else {
            res = vim_strsave_escape_ks(p);
        }
        xfree(p as *mut ::core::ffi::c_void);
        return res;
    }
}
