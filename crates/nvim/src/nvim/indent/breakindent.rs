//! 'breakindent': the indent a wrapped line's continuation carries, and
//! the 'breakindentopt' value that shapes it.

// Still transpiled: the bodies of its `unsafe` fns are bare, and the parent's
// inner deny of `unsafe_op_in_unsafe_fn` reaches every child, so this file
// has to opt out of it until it is rewritten. (Spelling the parent's
// attribute out here would also make the ratchet read this file as carrying
// it, and stop charging for the declarations -- see `traps-ratchet.md`.)
#![allow(unsafe_op_in_unsafe_fn)]
use super::*;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::charset::{getdigits, getdigits_int, vim_strsize};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{dy_flags, empty_string_option};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::r#move::{win_col_off, win_col_off2};
use crate::src::nvim::option::{get_flp_value, get_showbreak_value};
use crate::src::nvim::os::libc::{strcmp, strncmp};
use crate::src::nvim::plines::win_chartabsize;
use crate::src::nvim::regexp::{RE_AUTO, RE_MAGIC, RE_STRICT, RE_STRING};

pub unsafe extern "C" fn briopt_check(
    mut briopt: *mut ::core::ffi::c_char,
    mut wp: *mut win_T,
) -> bool {
    let mut bri_shift: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bri_min: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
    let mut bri_sbr: bool = false;
    let mut bri_list: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bri_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = empty_string_option.ptr() as *mut ::core::ffi::c_char;
    if !briopt.is_null() {
        p = briopt;
    } else if !wp.is_null() {
        p = (*wp).w_onebuf_opt.wo_briopt;
    }
    while *p as ::core::ffi::c_int != NUL {
        if strncmp(p, c"shift:".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int
            && (*p.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '-' as ::core::ffi::c_int
                && ascii_isdigit(*p.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                || ascii_isdigit(*p.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            bri_shift = getdigits_int(&raw mut p, true, 0 as ::core::ffi::c_int);
        } else if strncmp(p, c"min:".as_ptr(), 4 as size_t) == 0 as ::core::ffi::c_int
            && ascii_isdigit(*p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(4 as ::core::ffi::c_int as isize);
            bri_min = getdigits_int(&raw mut p, true, 0 as ::core::ffi::c_int);
        } else if strncmp(p, c"sbr".as_ptr(), 3 as size_t) == 0 as ::core::ffi::c_int {
            p = p.offset(3 as ::core::ffi::c_int as isize);
            bri_sbr = true;
        } else if strncmp(p, c"list:".as_ptr(), 5 as size_t) == 0 as ::core::ffi::c_int {
            p = p.offset(5 as ::core::ffi::c_int as isize);
            bri_list = getdigits(&raw mut p, false, 0 as intmax_t) as ::core::ffi::c_int;
        } else if strncmp(p, c"column:".as_ptr(), 7 as size_t) == 0 as ::core::ffi::c_int {
            p = p.offset(7 as ::core::ffi::c_int as isize);
            bri_vcol = getdigits(&raw mut p, false, 0 as intmax_t) as ::core::ffi::c_int;
        }
        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL
        {
            return false;
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
    }
    if wp.is_null() {
        return OK != 0;
    }
    (*wp).w_briopt_shift = bri_shift;
    (*wp).w_briopt_min = bri_min;
    (*wp).w_briopt_sbr = bri_sbr;
    (*wp).w_briopt_list = bri_list;
    (*wp).w_briopt_vcol = bri_vcol;
    return true;
}
pub unsafe extern "C" fn get_breakindent_win(
    mut wp: *mut win_T,
    mut line: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    /// Cached result for the last (buffer, line, options) combination.
    struct BreakindentCache {
        indent: ::core::ffi::c_int,
        ts: OptInt,
        vts: *mut colnr_T,
        fnum: ::core::ffi::c_int,
        line: *mut ::core::ffi::c_char,
        tick: varnumber_T,
        list: ::core::ffi::c_int,
        listopt: ::core::ffi::c_int,
        no_ts: bool,
        dy_uhex: ::core::ffi::c_uint,
        flp: *mut ::core::ffi::c_char,
    }
    static CACHE: GlobalCell<BreakindentCache> = GlobalCell::new(BreakindentCache {
        indent: 0,
        ts: 0,
        vts: ::core::ptr::null_mut(),
        fnum: 0,
        line: ::core::ptr::null_mut(),
        tick: 0,
        list: 0,
        listopt: 0,
        no_ts: false,
        dy_uhex: 0,
        flp: ::core::ptr::null_mut(),
    });
    let eff_wwidth: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp) + win_col_off2(wp);
    let no_ts: bool = (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.tab1 == NUL as schar_T;
    // One exclusive borrow for the whole computation: nothing below calls
    // back into this function (the regex engine and chartabsize helpers run
    // no user code), and debug builds will catch it if that ever changes.
    let mut bri: ::core::ffi::c_int = CACHE.with_mut(|prev| {
        if prev.fnum != (*(*wp).w_buffer).handle
            || prev.ts != (*(*wp).w_buffer).b_p_ts
            || prev.vts != (*(*wp).w_buffer).b_p_vts_array
            || prev.tick != buf_get_changedtick((*wp).w_buffer)
            || prev.listopt != (*wp).w_briopt_list
            || prev.no_ts != no_ts
            || prev.dy_uhex
                != dy_flags.get() & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint
            || prev.flp.is_null()
            || strcmp(prev.flp, get_flp_value((*wp).w_buffer)) != 0 as ::core::ffi::c_int
            || prev.line.is_null()
            || strcmp(prev.line, line) != 0 as ::core::ffi::c_int
        {
            prev.fnum = (*(*wp).w_buffer).handle;
            xfree(prev.line as *mut ::core::ffi::c_void);
            prev.line = xstrdup(line);
            prev.ts = (*(*wp).w_buffer).b_p_ts;
            prev.vts = (*(*wp).w_buffer).b_p_vts_array;
            if (*wp).w_briopt_vcol == 0 as ::core::ffi::c_int {
                if no_ts {
                    prev.indent = indent_size_no_ts(line);
                } else {
                    prev.indent = indent_size_ts(
                        line,
                        (*(*wp).w_buffer).b_p_ts,
                        (*(*wp).w_buffer).b_p_vts_array,
                    );
                }
            }
            prev.tick = buf_get_changedtick((*wp).w_buffer);
            prev.listopt = (*wp).w_briopt_list;
            prev.list = 0 as ::core::ffi::c_int;
            prev.no_ts = no_ts;
            prev.dy_uhex =
                dy_flags.get() & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint;
            xfree(prev.flp as *mut ::core::ffi::c_void);
            prev.flp = xstrdup(get_flp_value((*wp).w_buffer));
            if (*wp).w_briopt_list != 0 as ::core::ffi::c_int
                && (*wp).w_briopt_vcol == 0 as ::core::ffi::c_int
            {
                let mut regmatch: regmatch_T = regmatch_T {
                    regprog: vim_regcomp(prev.flp, RE_MAGIC + RE_STRING + RE_AUTO + RE_STRICT),
                    startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    rm_matchcol: 0,
                    rm_ic: false,
                };
                if !regmatch.regprog.is_null() {
                    regmatch.rm_ic = false;
                    if vim_regexec(&raw mut regmatch, line, 0 as colnr_T) {
                        if (*wp).w_briopt_list > 0 as ::core::ffi::c_int {
                            prev.list += (*wp).w_briopt_list;
                        } else {
                            let mut ptr: *mut ::core::ffi::c_char = regmatch.startp[0];
                            let end_ptr: *mut ::core::ffi::c_char = regmatch.endp[0];
                            let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while ptr < end_ptr {
                                indent += win_chartabsize(wp, ptr, indent as colnr_T);
                                ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
                            }
                            prev.indent = indent;
                        }
                    }
                    vim_regfree(regmatch.regprog);
                }
            }
        }
        let mut bri = if (*wp).w_briopt_vcol != 0 as ::core::ffi::c_int {
            prev.list = 0 as ::core::ffi::c_int;
            (*wp).w_briopt_vcol
        } else {
            prev.indent + (*wp).w_briopt_shift
        };
        bri += win_col_off2(wp);
        if (*wp).w_briopt_list > 0 as ::core::ffi::c_int {
            bri += prev.list;
        }
        bri
    });
    if (*wp).w_briopt_sbr {
        bri -= vim_strsize(get_showbreak_value(wp));
    }
    if bri < 0 as ::core::ffi::c_int {
        bri = 0 as ::core::ffi::c_int;
    } else if bri > eff_wwidth - (*wp).w_briopt_min {
        bri = if eff_wwidth - (*wp).w_briopt_min < 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            eff_wwidth - (*wp).w_briopt_min
        };
    }
    return bri;
}
