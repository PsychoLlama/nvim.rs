//! Reading a `:s` command line, and remembering it for the next one.
//!
//! `skip_substitute` walks the pattern/replacement/flags off the command line
//! without interpreting them (`:s` is its own little language, and the
//! delimiter may be almost any character -- `check_regexp_delim` rejects the
//! ones that would be ambiguous), `sub_parse_flags` turns the trailing letters
//! into `subflags_T`, and `old_sub` is the `~` replacement text carried from
//! the last `:s`.  `sub_joining_lines` is the `\n`-in-the-replacement case,
//! which joins rather than substitutes, and `sub_grow_buf` is the output
//! buffer's growth policy.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

pub(crate) static old_sub: GlobalCell<SubReplacementString> =
    GlobalCell::new(SubReplacementString {
        sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        timestamp: 0 as Timestamp,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    });

pub(crate) static global_need_beginline: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

pub unsafe extern "C" fn sub_get_replacement(ret_sub: *mut SubReplacementString) {
    unsafe {
        *ret_sub = old_sub.get();
    }
}

pub unsafe extern "C" fn sub_set_replacement(mut sub: SubReplacementString) {
    unsafe {
        xfree((*old_sub.ptr()).sub as *mut ::core::ffi::c_void);
        if sub.additional_data != (*old_sub.ptr()).additional_data {
            xfree((*old_sub.ptr()).additional_data as *mut ::core::ffi::c_void);
        }
        old_sub.set(sub);
    }
}

pub(crate) unsafe extern "C" fn sub_joining_lines(
    mut eap: *mut exarg_T,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut sub: *const ::core::ffi::c_char,
    mut cmd: *const ::core::ffi::c_char,
    mut save: bool,
    mut keeppatterns: bool,
) -> bool {
    unsafe {
        if !pat.is_null()
            && strcmp(pat, c"\\n".as_ptr()) == 0 as ::core::ffi::c_int
            && *sub as ::core::ffi::c_int == NUL
            && (*cmd as ::core::ffi::c_int == NUL
                || *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    && (*cmd as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
                        || *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                        || *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int
                        || *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int))
        {
            if (*eap).skip != 0 {
                return true_0 != 0;
            }
            (*curwin.get()).w_cursor.lnum = (*eap).line1;
            if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                (*eap).flags = EXFLAG_LIST;
            } else if *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
                (*eap).flags = EXFLAG_NR;
            } else if *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
                (*eap).flags = EXFLAG_PRINT;
            }
            let mut joined_lines_count: linenr_T = (*eap).line2 - (*eap).line1
                + 1 as linenr_T
                + (if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
                    1 as linenr_T
                } else {
                    0 as linenr_T
                });
            if joined_lines_count > 1 as linenr_T {
                do_join(
                    joined_lines_count as size_t,
                    false_0 != 0,
                    true_0 != 0,
                    false_0 != 0,
                    true_0 != 0,
                );
                sub_nsubs.set((joined_lines_count - 1 as linenr_T) as ::core::ffi::c_int);
                sub_nlines.set(1 as ::core::ffi::c_int as linenr_T);
                do_sub_msg(false_0 != 0);
                ex_may_print(eap);
            }
            if save {
                if !keeppatterns {
                    save_re_pat(RE_SUBST as ::core::ffi::c_int, pat, patlen, magic_isset());
                }
                add_to_history(
                    HIST_SEARCH as ::core::ffi::c_int,
                    ::core::slice::from_raw_parts(pat as *const u8, patlen),
                    true_0 != 0,
                    NUL as u8,
                );
            }
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn sub_grow_buf(
    mut new_start: *mut *mut ::core::ffi::c_char,
    mut new_start_len: *mut ::core::ffi::c_int,
    mut needed_len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut new_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*new_start).is_null() {
            *new_start_len = needed_len + 50 as ::core::ffi::c_int;
            *new_start = xcalloc(1 as size_t, *new_start_len as size_t) as *mut ::core::ffi::c_char;
            **new_start = NUL as ::core::ffi::c_char;
            new_end = *new_start;
        } else {
            let mut len: size_t = strlen(*new_start);
            needed_len += len as ::core::ffi::c_int;
            if needed_len > *new_start_len {
                let mut prev_new_start_len: size_t = *new_start_len as size_t;
                *new_start_len = needed_len + 50 as ::core::ffi::c_int;
                let mut added_len: size_t =
                    (*new_start_len as size_t).wrapping_sub(prev_new_start_len);
                *new_start = xrealloc(
                    *new_start as *mut ::core::ffi::c_void,
                    *new_start_len as size_t,
                ) as *mut ::core::ffi::c_char;
                memset(
                    (*new_start).add(prev_new_start_len) as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    added_len,
                );
            }
            new_end = (*new_start).add(len);
        }
        return new_end;
    }
}

pub(crate) unsafe extern "C" fn sub_parse_flags(
    mut cmd: *mut ::core::ffi::c_char,
    mut subflags: *mut subflags_T,
    mut which_pat: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if *cmd as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            cmd = cmd.offset(1);
        } else {
            (*subflags).do_all = p_gd.get() != 0;
            (*subflags).do_ask = false_0 != 0;
            (*subflags).do_error = true_0 != 0;
            (*subflags).do_print = false_0 != 0;
            (*subflags).do_list = false_0 != 0;
            (*subflags).do_count = false_0 != 0;
            (*subflags).do_number = false_0 != 0;
            (*subflags).do_ic = kSubHonorOptions;
        }
        while *cmd != 0 {
            if *cmd as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
                (*subflags).do_all = !(*subflags).do_all;
            } else if *cmd as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
                (*subflags).do_ask = !(*subflags).do_ask;
            } else if *cmd as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
                (*subflags).do_count = true_0 != 0;
            } else if *cmd as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                (*subflags).do_error = !(*subflags).do_error;
            } else if *cmd as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
                *which_pat = RE_LAST as ::core::ffi::c_int;
            } else if *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
                (*subflags).do_print = true_0 != 0;
            } else if *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
                (*subflags).do_print = true_0 != 0;
                (*subflags).do_number = true_0 != 0;
            } else if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                (*subflags).do_print = true_0 != 0;
                (*subflags).do_list = true_0 != 0;
            } else if *cmd as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                (*subflags).do_ic = kSubIgnoreCase;
            } else {
                if *cmd as ::core::ffi::c_int != 'I' as ::core::ffi::c_int {
                    break;
                }
                (*subflags).do_ic = kSubMatchCase;
            }
            cmd = cmd.offset(1);
        }
        if (*subflags).do_count {
            (*subflags).do_ask = false_0 != 0;
        }
        return cmd;
    }
}

pub(crate) unsafe extern "C" fn skip_substitute(
    mut start: *mut ::core::ffi::c_char,
    mut delimiter: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = start;
        while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == delimiter {
                let c2rust_fresh12 = p;
                p = p.offset(1);
                *c2rust_fresh12 = NUL as ::core::ffi::c_char;
                break;
            } else {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                {
                    p = p.offset(1);
                }
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn check_regexp_delim(
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
            & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        {
            emsg(gettext(
                c"E146: Regular expressions can't be delimited by letters".as_ptr(),
            ));
            return FAIL;
        }
        return OK;
    }
}
