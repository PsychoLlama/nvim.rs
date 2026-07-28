//! Vetting a new value before anything is allowed to see it.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn check_num_option_bounds(
    mut opt_idx: OptIndex,
    mut newval: *mut OptInt,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    match opt_idx as c_int {
        169 => {
            if *newval < min_rows_for_all_tabpages() as OptInt && full_screen.get() as c_int != 0 {
                vim_snprintf(
                    errbuf,
                    errbuflen,
                    gettext(b"E593: Need at least %d lines\0".as_ptr() as *const c_char),
                    min_rows_for_all_tabpages(),
                );
                errmsg = errbuf;
                *newval = min_rows_for_all_tabpages() as OptInt;
            }
            *newval = if *newval < 2147483647 as OptInt {
                *newval
            } else {
                2147483647 as OptInt
            };
        }
        47 => {
            if *newval < MIN_COLUMNS as c_int as OptInt && full_screen.get() as c_int != 0 {
                vim_snprintf(
                    errbuf,
                    errbuflen,
                    gettext(b"E594: Need at least %d columns\0".as_ptr() as *const c_char),
                    MIN_COLUMNS as c_int,
                );
                errmsg = errbuf;
                *newval = MIN_COLUMNS as c_int as OptInt;
            }
            *newval = if *newval < 2147483647 as OptInt {
                *newval
            } else {
                2147483647 as OptInt
            };
        }
        222 => {
            *newval = if (if *newval < 100 as OptInt {
                *newval
            } else {
                100 as OptInt
            }) > 0 as OptInt
            {
                if *newval < 100 as OptInt {
                    *newval
                } else {
                    100 as OptInt
                }
            } else {
                0 as OptInt
            };
        }
        246 => {
            if (*newval < -100 as OptInt || *newval >= Rows.get() as OptInt)
                && full_screen.get() as c_int != 0
            {
                errmsg = &raw const e_scroll as *const c_char;
                *newval = 1 as OptInt;
            }
        }
        243 => {
            if (*newval <= 0 as OptInt
                || *newval > (*curwin.get()).w_view_height as OptInt
                    && (*curwin.get()).w_view_height > 0 as c_int)
                && full_screen.get() as c_int != 0
            {
                if *newval != 0 as OptInt {
                    errmsg = &raw const e_scroll as *const c_char;
                }
                *newval = win_default_scroll(curwin.get());
            }
        }
        _ => {}
    }
    return errmsg;
}

pub(crate) unsafe extern "C" fn validate_num_option(
    mut opt_idx: OptIndex,
    mut newval: *mut OptInt,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut value: OptInt = *newval;
    if value < INT_MIN as OptInt || value > INT_MAX as OptInt {
        return &raw const e_invarg as *const c_char;
    }
    match opt_idx as c_int {
        129 | 325 | 335 | 236 | 336 | 275 | 106 | 266 | 318 | 373 | 323 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            }
        }
        362 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if p_wmh.get() > value {
                return &raw const e_winheight as *const c_char;
            }
        }
        364 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > p_wh.get() {
                return &raw const e_winheight as *const c_char;
            }
        }
        366 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if p_wmw.get() > value {
                return &raw const e_winwidth as *const c_char;
            }
        }
        365 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > p_wiw.get() {
                return &raw const e_winwidth as *const c_char;
            }
        }
        183 => {
            *newval = MAX_MCO as OptInt;
        }
        44 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            }
        }
        133 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > 10000 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        227 => {
            if value == 0 as OptInt {
                *newval = 3 as OptInt;
            } else if value != 3 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        233 => {
            if value < 0 as OptInt || value > 2 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        247 => {
            if value < 0 as OptInt && full_screen.get() as c_int != 0 {
                return &raw const e_positive as *const c_char;
            }
        }
        276 => {
            if value < 0 as OptInt && full_screen.get() as c_int != 0 {
                return &raw const e_positive as *const c_char;
            }
        }
        45 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            }
        }
        58 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > 3 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        207 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > MAX_NUMBERWIDTH as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        142 => {
            if value < 0 as OptInt || value > B_IMODE_LAST as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        143 => {
            if value < -1 as OptInt || value > B_IMODE_LAST as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        35 => return &raw const e_invarg as *const c_char,
        244 => {
            if value < -1 as OptInt || value > SB_MAX as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        304 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > TABSTOP_MAX as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        37 | 167 => {
            if value < 1 as OptInt {
                return (e_cannot_have_negative_or_zero_number_of_quickfix.ptr() as *const _)
                    as *const c_char;
            } else if value > 100 as OptInt {
                return (e_cannot_have_more_than_hundred_quickfix.ptr() as *const _)
                    as *const c_char;
            }
        }
        187 => {
            if value <= 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > MAX_SEARCH_COUNT as c_int as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        _ => {}
    }
    return check_num_option_bounds(opt_idx, newval, errbuf, errbuflen);
}

pub(crate) unsafe extern "C" fn validate_option_value(
    opt_idx: OptIndex,
    mut newval: *mut OptVal,
    mut opt_flags: c_int,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    if option_is_global_local(opt_idx) as c_int != 0
        && opt_flags & OPT_LOCAL as c_int != 0
        && optval_equal(*newval, get_option_unset_value(opt_idx)) as c_int != 0
    {
        return ::core::ptr::null::<c_char>();
    }
    if (*newval).type_0 as c_int == kOptValTypeNil as c_int {
        if opt_flags == OPT_GLOBAL as c_int {
            errmsg = gettext(b"Cannot unset global option value\0".as_ptr() as *const c_char);
        } else {
            *newval = optval_copy(get_option_unset_value(opt_idx));
        }
    } else if !option_has_type(opt_idx, (*newval).type_0) {
        let mut rep: *mut c_char = optval_to_cstr(*newval);
        let mut type_str: *const c_char = optval_type_name((*opt).type_0).as_ptr();
        snprintf(
            errbuf,
            IOSIZE as size_t,
            gettext(
                b"Invalid value for option '%s': expected %s, got %s %s\0".as_ptr()
                    as *const c_char,
            ),
            (*opt).fullname,
            type_str,
            optval_type_name((*newval).type_0).as_ptr(),
            rep,
        );
        xfree(rep as *mut c_void);
        errmsg = errbuf;
    } else if (*newval).type_0 as c_int == kOptValTypeNumber as c_int {
        errmsg = validate_num_option(opt_idx, &raw mut (*newval).data.number, errbuf, errbuflen);
    }
    return errmsg;
}
