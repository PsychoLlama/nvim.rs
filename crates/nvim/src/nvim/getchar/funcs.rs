//! The Vimscript `getchar()` family.
//!
//! [`getchar_common`] backs `getchar()`, `getcharstr()` and `getcharmod()`:
//! it reads one key with mappings disabled, optionally without blocking, and
//! renders it as a number, a string or a modifier mask.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn getchar_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut allow_number: bool,
) {
    unsafe {
        let mut n: varnumber_T = 0 as varnumber_T;
        let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
        let mut error: bool = false_0 != 0;
        let mut simplify: bool = true_0 != 0;
        let mut cursor_flag: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut d: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            if allow_number {
                allow_number = tv_dict_get_bool(
                    d,
                    b"number\0".as_ptr() as *const ::core::ffi::c_char,
                    true_0,
                ) != 0;
            } else if tv_dict_has_key(d, b"number\0".as_ptr() as *const ::core::ffi::c_char) {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    b"number\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            simplify = tv_dict_get_bool(
                d,
                b"simplify\0".as_ptr() as *const ::core::ffi::c_char,
                true_0,
            ) != 0;
            let mut cursor_str: *const ::core::ffi::c_char = tv_dict_get_string(
                d,
                b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            if !cursor_str.is_null() {
                if strcmp(cursor_str, b"hide\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int
                    && strcmp(cursor_str, b"keep\0".as_ptr() as *const ::core::ffi::c_char)
                        != 0 as ::core::ffi::c_int
                    && strcmp(cursor_str, b"msg\0".as_ptr() as *const ::core::ffi::c_char)
                        != 0 as ::core::ffi::c_int
                {
                    semsg(
                        gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                        b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
                        cursor_str,
                    );
                } else {
                    cursor_flag = *cursor_str.offset(0 as ::core::ffi::c_int as isize);
                }
            }
        }
        if called_emsg.get() != called_emsg_start {
            return;
        }
        if cursor_flag as ::core::ffi::c_int == 'h' as ::core::ffi::c_int {
            ui_busy_start();
        }
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        if !simplify {
            (*no_reduce_keys.ptr()) += 1;
        }
        loop {
            if cursor_flag as ::core::ffi::c_int == 'm' as ::core::ffi::c_int
                || cursor_flag as ::core::ffi::c_int == NUL
                    && msg_col.get() > 0 as ::core::ffi::c_int
            {
                ui_cursor_goto(msg_row.get(), msg_col.get());
            }
            if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_number
                        == -1 as varnumber_T
            {
                if !char_avail() {
                    ui_flush();
                    input_get(
                        ::core::ptr::null_mut::<uint8_t>(),
                        0 as ::core::ffi::c_int,
                        -1 as ::core::ffi::c_int,
                        (*typebuf.ptr()).tb_change_cnt,
                        (*main_loop.ptr()).events,
                    );
                    if input_available() == 0 && !multiqueue_empty((*main_loop.ptr()).events) {
                        state_handle_k_event();
                        continue;
                    }
                }
                n = safe_vgetc() as varnumber_T;
            } else if tv_get_number_chk(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) == 1 as varnumber_T
            {
                n = vpeekc_any() as varnumber_T;
            } else if error as ::core::ffi::c_int != 0 || vpeekc_any() == NUL {
                n = 0 as varnumber_T;
            } else {
                n = safe_vgetc() as varnumber_T;
            }
            if !(n
                == -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    as varnumber_T
                || n == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    as varnumber_T
                || n == K_VER_SCROLLBAR as varnumber_T
                || n == K_HOR_SCROLLBAR as varnumber_T)
            {
                break;
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        if !simplify {
            (*no_reduce_keys.ptr()) -= 1;
        }
        if cursor_flag as ::core::ffi::c_int == 'h' as ::core::ffi::c_int {
            ui_busy_stop();
        }
        set_vim_var_nr(VV_MOUSE_WIN, 0 as varnumber_T);
        set_vim_var_nr(VV_MOUSE_WINID, 0 as varnumber_T);
        set_vim_var_nr(VV_MOUSE_LNUM, 0 as varnumber_T);
        set_vim_var_nr(VV_MOUSE_COL, 0 as varnumber_T);
        if n != 0 as varnumber_T
            && (!allow_number || n < 0 as varnumber_T || mod_mask.get() != 0 as ::core::ffi::c_int)
        {
            let mut temp: [::core::ffi::c_char; 10] = [0; 10];
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if mod_mask.get() != 0 as ::core::ffi::c_int {
                let c2rust_fresh11 = i;
                i = i + 1;
                temp[c2rust_fresh11 as usize] = K_SPECIAL as ::core::ffi::c_char;
                let c2rust_fresh12 = i;
                i = i + 1;
                temp[c2rust_fresh12 as usize] = KS_MODIFIER as ::core::ffi::c_char;
                let c2rust_fresh13 = i;
                i = i + 1;
                temp[c2rust_fresh13 as usize] = mod_mask.get() as ::core::ffi::c_char;
            }
            if n < 0 as varnumber_T {
                let c2rust_fresh14 = i;
                i = i + 1;
                temp[c2rust_fresh14 as usize] = K_SPECIAL as ::core::ffi::c_char;
                let c2rust_fresh15 = i;
                i = i + 1;
                temp[c2rust_fresh15 as usize] = (if n == K_SPECIAL as varnumber_T {
                    KS_SPECIAL as varnumber_T
                } else if n == NUL as varnumber_T {
                    KS_ZERO as varnumber_T
                } else {
                    -n & 0xff as varnumber_T
                }) as ::core::ffi::c_char;
                let c2rust_fresh16 = i;
                i = i + 1;
                temp[c2rust_fresh16 as usize] =
                    (if n == K_SPECIAL as varnumber_T || n == NUL as varnumber_T {
                        KE_FILLER as ::core::ffi::c_uint
                    } else {
                        -n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_uint
                    }) as ::core::ffi::c_char;
            } else {
                i += utf_char2bytes(
                    n as ::core::ffi::c_int,
                    (&raw mut temp as *mut ::core::ffi::c_char).offset(i as isize),
                );
            }
            '_c2rust_label: {
                if i < 10 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"i < 10\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2021 as ::core::ffi::c_uint,
                        b"void getchar_common(typval_T *, typval_T *, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            temp[i as usize] = NUL as ::core::ffi::c_char;
            (*rettv).v_type = VAR_STRING;
            (*rettv).vval.v_string = xmemdupz(
                &raw mut temp as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                i as size_t,
            ) as *mut ::core::ffi::c_char;
            if is_mouse_key(n as ::core::ffi::c_int) {
                let mut row: ::core::ffi::c_int = mouse_row.get();
                let mut col: ::core::ffi::c_int = mouse_col.get();
                let mut grid: ::core::ffi::c_int = mouse_grid.get();
                let mut lnum: linenr_T = 0;
                let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
                if row >= 0 as ::core::ffi::c_int && col >= 0 as ::core::ffi::c_int {
                    let mut winnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    let win: *mut win_T =
                        mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
                    if win.is_null() {
                        return;
                    }
                    mouse_comp_pos(win, &raw mut row, &raw mut col, &raw mut lnum);
                    wp = firstwin.get();
                    while wp != win {
                        winnr += 1;
                        wp = (*wp).w_next;
                    }
                    set_vim_var_nr(VV_MOUSE_WIN, winnr as varnumber_T);
                    set_vim_var_nr(VV_MOUSE_WINID, (*wp).handle as varnumber_T);
                    set_vim_var_nr(VV_MOUSE_LNUM, lnum as varnumber_T);
                    set_vim_var_nr(VV_MOUSE_COL, (col + 1 as ::core::ffi::c_int) as varnumber_T);
                }
            }
        } else if !allow_number {
            (*rettv).v_type = VAR_STRING;
        } else {
            (*rettv).vval.v_number = n;
        };
    }
}

pub unsafe extern "C" fn f_getchar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        getchar_common(argvars, rettv, true_0 != 0);
    }
}

pub unsafe extern "C" fn f_getcharstr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        getchar_common(argvars, rettv, false_0 != 0);
    }
}

pub unsafe extern "C" fn f_getcharmod(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = mod_mask.get() as varnumber_T;
    }
}
