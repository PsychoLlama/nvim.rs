//! Registers whose contents are computed, not stored.
//!
//! `"=` is an expression: `get_expr_register` prompts for it, `set_expr_line`
//! keeps the source for a repeat, and `get_expr_line` evaluates it -- so
//! reading this register runs arbitrary Vimscript, which is why every caller
//! has to cope with the buffer having changed underneath it.  `get_spec_reg`
//! is the rest of the read-only set: `".` the last insert, `"%` the file name,
//! `"#` the alternate file, `":` the last command line, `"/` the last search
//! pattern, and the `"<cword>"`-shaped answers CTRL-R CTRL-W and friends
//! want.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_expr_register() -> ::core::ffi::c_int {
    unsafe {
        let mut new_line: *mut ::core::ffi::c_char = getcmdline(
            '=' as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
        );
        if new_line.is_null() {
            return NUL;
        }
        if *new_line as ::core::ffi::c_int == NUL {
            xfree(new_line as *mut ::core::ffi::c_void);
        } else {
            set_expr_line(new_line);
        }
        return '=' as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn set_expr_line(mut new_line: *mut ::core::ffi::c_char) {
    unsafe {
        xfree(expr_line.get() as *mut ::core::ffi::c_void);
        expr_line.set(new_line);
    }
}

pub unsafe extern "C" fn get_expr_line() -> *mut ::core::ffi::c_char {
    unsafe {
        static nested: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if (*expr_line.ptr()).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut expr_copy: *mut ::core::ffi::c_char = xstrdup(expr_line.get());
        if nested.get() >= 10 as ::core::ffi::c_int {
            return expr_copy;
        }
        (*nested.ptr()) += 1;
        let mut rv: *mut ::core::ffi::c_char = eval_to_string(expr_copy, true_0 != 0, false_0 != 0);
        (*nested.ptr()) -= 1;
        xfree(expr_copy as *mut ::core::ffi::c_void);
        return rv;
    }
}

pub unsafe extern "C" fn get_expr_line_src() -> *mut ::core::ffi::c_char {
    unsafe {
        if (*expr_line.ptr()).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return xstrdup(expr_line.get());
    }
}

pub unsafe extern "C" fn get_spec_reg(
    mut regname: ::core::ffi::c_int,
    mut argp: *mut *mut ::core::ffi::c_char,
    mut allocated: *mut bool,
    mut errmsg: bool,
) -> bool {
    unsafe {
        *argp = ::core::ptr::null_mut::<::core::ffi::c_char>();
        *allocated = false_0 != 0;
        let mut cnt: size_t = 0;
        match regname {
            37 => {
                if errmsg {
                    check_fname();
                }
                *argp = (*curbuf.get()).b_fname;
                return true_0 != 0;
            }
            35 => {
                *argp = getaltfname(errmsg);
                return true_0 != 0;
            }
            61 => {
                *argp = get_expr_line();
                *allocated = true_0 != 0;
                return true_0 != 0;
            }
            58 => {
                if (*last_cmdline.ptr()).is_null() && errmsg as ::core::ffi::c_int != 0 {
                    emsg(gettext(
                        &raw const e_nolastcmd as *const ::core::ffi::c_char,
                    ));
                }
                *argp = last_cmdline.get();
                return true_0 != 0;
            }
            47 => {
                if last_search_pat().is_null() && errmsg as ::core::ffi::c_int != 0 {
                    emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                }
                *argp = last_search_pat();
                return true_0 != 0;
            }
            46 => {
                *argp = get_last_insert_save();
                *allocated = true_0 != 0;
                if (*argp).is_null() && errmsg as ::core::ffi::c_int != 0 {
                    emsg(gettext(
                        &raw const e_noinstext as *const ::core::ffi::c_char,
                    ));
                }
                return true_0 != 0;
            }
            Ctrl_F | Ctrl_P => {
                if !errmsg {
                    return false_0 != 0;
                }
                *argp = file_name_at_cursor(
                    FNAME_MESS as ::core::ffi::c_int
                        | FNAME_HYP as ::core::ffi::c_int
                        | (if regname == Ctrl_P {
                            FNAME_EXP as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                    1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                *allocated = true_0 != 0;
                return true_0 != 0;
            }
            Ctrl_W | Ctrl_A => {
                if !errmsg {
                    return false_0 != 0;
                }
                cnt = find_ident_under_cursor(
                    argp,
                    if regname == Ctrl_W {
                        FIND_IDENT as ::core::ffi::c_int | FIND_STRING as ::core::ffi::c_int
                    } else {
                        FIND_STRING as ::core::ffi::c_int
                    },
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                );
                *argp = (if cnt != 0 {
                    xmemdupz(*argp as *const ::core::ffi::c_void, cnt)
                } else {
                    NULL_0
                }) as *mut ::core::ffi::c_char;
                *allocated = true_0 != 0;
                return true_0 != 0;
            }
            Ctrl_L => {
                if !errmsg {
                    return false_0 != 0;
                }
                *argp = ml_get_buf((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum);
                return true_0 != 0;
            }
            95 => {
                *argp = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                return true_0 != 0;
            }
            _ => {}
        }
        return false_0 != 0;
    }
}
