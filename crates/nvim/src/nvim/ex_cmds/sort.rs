//! `:sort` and `:uniq`.
//!
//! Both read the whole range into an array of `sorti_T` -- one entry per line,
//! holding either the number parsed out of it, the float, or the byte range the
//! comparison should use -- sort or scan that array, and write the lines back
//! in the new order.  The comparison is `sort_compare`, which dispatches on the
//! flag statics (`sort_nr`, `sort_flt`, `sort_rx`, `sort_ic`, `sort_lc`) that
//! `ex_sort` sets from the command's flags; `sort_abort` is how a comparator
//! that hit an error stops libc's `qsort` early, since it cannot return one.
//! `/pat/` restricts the comparison to what the pattern matched, which is why
//! the comparator can reach the regex engine.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

static sortbuf1: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

static sortbuf2: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

static sort_lc: GlobalCell<bool> = GlobalCell::new(false);

static sort_ic: GlobalCell<bool> = GlobalCell::new(false);

static sort_nr: GlobalCell<bool> = GlobalCell::new(false);

static sort_rx: GlobalCell<bool> = GlobalCell::new(false);

static sort_flt: GlobalCell<bool> = GlobalCell::new(false);

static sort_abort: GlobalCell<bool> = GlobalCell::new(false);

unsafe extern "C" fn string_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        if sort_lc.get() {
            return strcoll(
                s1 as *const ::core::ffi::c_char,
                s2 as *const ::core::ffi::c_char,
            );
        }
        return if sort_ic.get() as ::core::ffi::c_int != 0 {
            strcasecmp(
                s1 as *mut ::core::ffi::c_char,
                s2 as *mut ::core::ffi::c_char,
            )
        } else {
            strcmp(
                s1 as *const ::core::ffi::c_char,
                s2 as *const ::core::ffi::c_char,
            )
        };
    }
}

unsafe extern "C" fn sort_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut l1: sorti_T = *(s1 as *mut sorti_T);
        let mut l2: sorti_T = *(s2 as *mut sorti_T);
        let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if sort_abort.get() {
            return 0 as ::core::ffi::c_int;
        }
        fast_breakcheck();
        if got_int.get() {
            sort_abort.set(true_0 != 0);
        }
        if sort_nr.get() {
            if l1.st_u.num.is_number as ::core::ffi::c_int
                != l2.st_u.num.is_number as ::core::ffi::c_int
            {
                result = if l1.st_u.num.is_number as ::core::ffi::c_int
                    > l2.st_u.num.is_number as ::core::ffi::c_int
                {
                    1 as ::core::ffi::c_int
                } else {
                    -1 as ::core::ffi::c_int
                };
            } else {
                result = if l1.st_u.num.value == l2.st_u.num.value {
                    0 as ::core::ffi::c_int
                } else if l1.st_u.num.value > l2.st_u.num.value {
                    1 as ::core::ffi::c_int
                } else {
                    -1 as ::core::ffi::c_int
                };
            }
        } else if sort_flt.get() {
            result = if l1.st_u.value_flt == l2.st_u.value_flt {
                0 as ::core::ffi::c_int
            } else if l1.st_u.value_flt > l2.st_u.value_flt {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else {
            memcpy(
                sortbuf1.get() as *mut ::core::ffi::c_void,
                ml_get(l1.lnum).offset(l1.st_u.line.start_col_nr as isize)
                    as *const ::core::ffi::c_void,
                (l1.st_u.line.end_col_nr - l1.st_u.line.start_col_nr + 1 as varnumber_T) as size_t,
            );
            *(*sortbuf1.ptr())
                .offset((l1.st_u.line.end_col_nr - l1.st_u.line.start_col_nr) as isize) =
                NUL as ::core::ffi::c_char;
            memcpy(
                sortbuf2.get() as *mut ::core::ffi::c_void,
                ml_get(l2.lnum).offset(l2.st_u.line.start_col_nr as isize)
                    as *const ::core::ffi::c_void,
                (l2.st_u.line.end_col_nr - l2.st_u.line.start_col_nr + 1 as varnumber_T) as size_t,
            );
            *(*sortbuf2.ptr())
                .offset((l2.st_u.line.end_col_nr - l2.st_u.line.start_col_nr) as isize) =
                NUL as ::core::ffi::c_char;
            result = string_compare(
                sortbuf1.get() as *const ::core::ffi::c_void,
                sortbuf2.get() as *const ::core::ffi::c_void,
            );
        }
        if result == 0 as ::core::ffi::c_int {
            return l1.lnum as ::core::ffi::c_int - l2.lnum as ::core::ffi::c_int;
        }
        return result;
    }
}

pub unsafe fn ex_sort(mut eap: *mut exarg_T) {
    unsafe {
        let mut old_count: bcount_t = 0;
        let mut new_count: bcount_t = 0;
        let mut lnum_0: linenr_T = 0;
        let mut deleted: linenr_T = 0;
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut count: size_t = (((*eap).line2 - (*eap).line1) as size_t).wrapping_add(1 as size_t);
        let mut i: size_t = 0;
        let mut unique: bool = false_0 != 0;
        let mut sort_what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if count <= 1 as size_t {
            return;
        }
        if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
            return;
        }
        sortbuf1.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        sortbuf2.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
        let mut nrs: *mut sorti_T =
            xmalloc(count.wrapping_mul(::core::mem::size_of::<sorti_T>())) as *mut sorti_T;
        sort_flt.set(false_0 != 0);
        sort_nr.set(sort_flt.get());
        sort_rx.set(sort_nr.get());
        sort_lc.set(sort_rx.get());
        sort_ic.set(sort_lc.get());
        sort_abort.set(sort_ic.get());
        let mut format_found: size_t = 0 as size_t;
        let mut change_occurred: bool = false_0 != 0;
        let mut p: *mut ::core::ffi::c_char = (*eap).arg;
        '_sortend: {
            while *p as ::core::ffi::c_int != NUL {
                if !ascii_iswhite(*p as ::core::ffi::c_int) {
                    if *p as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                        sort_ic.set(true_0 != 0);
                    } else if *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                        sort_lc.set(true_0 != 0);
                    } else if *p as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
                        sort_rx.set(true_0 != 0);
                    } else if *p as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
                        sort_nr.set(true_0 != 0);
                        format_found = format_found.wrapping_add(1);
                    } else if *p as ::core::ffi::c_int == 'f' as ::core::ffi::c_int {
                        sort_flt.set(true_0 != 0);
                        format_found = format_found.wrapping_add(1);
                    } else if *p as ::core::ffi::c_int == 'b' as ::core::ffi::c_int {
                        sort_what =
                            STR2NR_BIN as ::core::ffi::c_int + STR2NR_FORCE as ::core::ffi::c_int;
                        format_found = format_found.wrapping_add(1);
                    } else if *p as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
                        sort_what =
                            STR2NR_OCT as ::core::ffi::c_int + STR2NR_FORCE as ::core::ffi::c_int;
                        format_found = format_found.wrapping_add(1);
                    } else if *p as ::core::ffi::c_int == 'x' as ::core::ffi::c_int {
                        sort_what =
                            STR2NR_HEX as ::core::ffi::c_int + STR2NR_FORCE as ::core::ffi::c_int;
                        format_found = format_found.wrapping_add(1);
                    } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                        unique = true_0 != 0;
                    } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                        break;
                    } else if !check_nextcmd(p).is_null() {
                        (*eap).nextcmd = check_nextcmd(p);
                        break;
                    } else if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                        && regmatch.regprog.is_null()
                    {
                        let mut s: *mut ::core::ffi::c_char = skip_regexp_err(
                            p.offset(1 as ::core::ffi::c_int as isize),
                            *p as ::core::ffi::c_int,
                            true_0,
                        );
                        if s.is_null() {
                            break '_sortend;
                        }
                        *s = NUL as ::core::ffi::c_char;
                        if s == p.offset(1 as ::core::ffi::c_int as isize) {
                            if last_search_pat().is_null() {
                                emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                                break '_sortend;
                            } else {
                                regmatch.regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
                            }
                        } else {
                            regmatch.regprog =
                                vim_regcomp(p.offset(1 as ::core::ffi::c_int as isize), RE_MAGIC);
                        }
                        if regmatch.regprog.is_null() {
                            break '_sortend;
                        }
                        p = s;
                        regmatch.rm_ic = p_ic.get() != 0;
                    } else {
                        semsg_c!(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            p,
                        );
                        break '_sortend;
                    }
                }
                p = p.offset(1);
            }
            if format_found > 1 as size_t {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            } else {
                sort_nr.set(sort_nr.get() as ::core::ffi::c_int | sort_what != 0);
                let mut lnum: linenr_T = (*eap).line1;
                while lnum <= (*eap).line2 {
                    let mut s_0: *mut ::core::ffi::c_char = ml_get(lnum);
                    let mut len: ::core::ffi::c_int = ml_get_len(lnum);
                    maxlen = if maxlen > len { maxlen } else { len };
                    let mut start_col: colnr_T = 0 as colnr_T;
                    let mut end_col: colnr_T = len as colnr_T;
                    if !regmatch.regprog.is_null()
                        && vim_regexec(&raw mut regmatch, s_0, 0 as colnr_T) as ::core::ffi::c_int
                            != 0
                    {
                        if sort_rx.get() {
                            start_col = regmatch.startp[0 as ::core::ffi::c_int as usize]
                                .offset_from(s_0)
                                as colnr_T;
                            end_col = regmatch.endp[0 as ::core::ffi::c_int as usize]
                                .offset_from(s_0) as colnr_T;
                        } else {
                            start_col = regmatch.endp[0 as ::core::ffi::c_int as usize]
                                .offset_from(s_0)
                                as colnr_T;
                        }
                    } else if !regmatch.regprog.is_null() {
                        end_col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    if sort_nr.get() as ::core::ffi::c_int != 0
                        || sort_flt.get() as ::core::ffi::c_int != 0
                    {
                        let mut s2: *mut ::core::ffi::c_char = s_0.offset(end_col as isize);
                        let mut c: ::core::ffi::c_char = *s2;
                        *s2 = NUL as ::core::ffi::c_char;
                        let mut p_0: *mut ::core::ffi::c_char = s_0.offset(start_col as isize);
                        if sort_nr.get() {
                            if sort_what & STR2NR_HEX as ::core::ffi::c_int != 0 {
                                s_0 = skiptohex(p_0);
                            } else if sort_what & STR2NR_BIN as ::core::ffi::c_int != 0 {
                                s_0 = skiptobin(p_0) as *mut ::core::ffi::c_char;
                            } else {
                                s_0 = skiptodigit(p_0);
                            }
                            if s_0 > p_0
                                && *s_0.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '-' as ::core::ffi::c_int
                            {
                                s_0 = s_0.offset(-1);
                            }
                            if *s_0 as ::core::ffi::c_int == NUL {
                                (*nrs.offset((lnum - (*eap).line1) as isize))
                                    .st_u
                                    .num
                                    .is_number = false_0 != 0;
                                (*nrs.offset((lnum - (*eap).line1) as isize)).st_u.num.value =
                                    0 as varnumber_T;
                            } else {
                                (*nrs.offset((lnum - (*eap).line1) as isize))
                                    .st_u
                                    .num
                                    .is_number = true_0 != 0;
                                vim_str2nr(
                                    s_0,
                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                    sort_what,
                                    &raw mut (*nrs.offset((lnum - (*eap).line1) as isize))
                                        .st_u
                                        .num
                                        .value,
                                    ::core::ptr::null_mut::<uvarnumber_T>(),
                                    0 as ::core::ffi::c_int,
                                    false_0 != 0,
                                    ::core::ptr::null_mut::<bool>(),
                                );
                            }
                        } else {
                            s_0 = skipwhite(p_0);
                            if *s_0 as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                                s_0 = skipwhite(s_0.offset(1 as ::core::ffi::c_int as isize));
                            }
                            if *s_0 as ::core::ffi::c_int == NUL {
                                (*nrs.offset((lnum - (*eap).line1) as isize)).st_u.value_flt =
                                    -DBL_MAX as float_T;
                            } else {
                                (*nrs.offset((lnum - (*eap).line1) as isize)).st_u.value_flt =
                                    strtod(s_0, ::core::ptr::null_mut::<*mut ::core::ffi::c_char>())
                                        as float_T;
                            }
                        }
                        *s2 = c;
                    } else {
                        (*nrs.offset((lnum - (*eap).line1) as isize))
                            .st_u
                            .line
                            .start_col_nr = start_col as varnumber_T;
                        (*nrs.offset((lnum - (*eap).line1) as isize))
                            .st_u
                            .line
                            .end_col_nr = end_col as varnumber_T;
                    }
                    (*nrs.offset((lnum - (*eap).line1) as isize)).lnum = lnum;
                    if !regmatch.regprog.is_null() {
                        fast_breakcheck();
                    }
                    if got_int.get() {
                        break '_sortend;
                    }
                    lnum += 1;
                }
                sortbuf1.set(xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
                    as *mut ::core::ffi::c_char);
                sortbuf2.set(xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
                    as *mut ::core::ffi::c_char);
                qsort(
                    nrs as *mut ::core::ffi::c_void,
                    count,
                    ::core::mem::size_of::<sorti_T>(),
                    Some(
                        sort_compare
                            as unsafe extern "C" fn(
                                *const ::core::ffi::c_void,
                                *const ::core::ffi::c_void,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
                if !sort_abort.get() {
                    old_count = 0 as bcount_t;
                    new_count = 0 as bcount_t;
                    lnum_0 = (*eap).line2;
                    i = 0 as size_t;
                    while i < count {
                        let get_lnum: linenr_T = (*nrs.add(if (*eap).forceit != 0 {
                            count.wrapping_sub(i).wrapping_sub(1 as size_t)
                        } else {
                            i
                        }))
                        .lnum;
                        if get_lnum + (count as linenr_T - 1 as linenr_T) != lnum_0 {
                            change_occurred = true_0 != 0;
                        }
                        let mut s_1: *mut ::core::ffi::c_char = ml_get(get_lnum);
                        let mut bytelen: colnr_T = ml_get_len(get_lnum) + 1 as colnr_T;
                        old_count += bytelen as bcount_t;
                        if !unique
                            || i == 0 as size_t
                            || string_compare(
                                s_1 as *const ::core::ffi::c_void,
                                sortbuf1.get() as *const ::core::ffi::c_void,
                            ) != 0 as ::core::ffi::c_int
                        {
                            strcpy(sortbuf1.get(), s_1);
                            let c2rust_fresh3 = lnum_0;
                            lnum_0 = lnum_0 + 1;
                            if ml_append(c2rust_fresh3, sortbuf1.get(), 0 as colnr_T, false_0 != 0)
                                == FAIL
                            {
                                break;
                            }
                            new_count += bytelen as bcount_t;
                        }
                        fast_breakcheck();
                        if got_int.get() {
                            break '_sortend;
                        }
                        i = i.wrapping_add(1);
                    }
                    if i == count {
                        i = 0 as size_t;
                        while i < count {
                            ml_delete((*eap).line1);
                            i = i.wrapping_add(1);
                        }
                    } else {
                        count = 0 as size_t;
                    }
                    deleted = count as linenr_T - (lnum_0 - (*eap).line2);
                    if deleted > 0 as linenr_T {
                        mark_adjust(
                            (*eap).line2 - deleted,
                            (*eap).line2,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            -deleted,
                            kExtmarkNOOP,
                        );
                        msgmore(-(deleted as ::core::ffi::c_int));
                    } else if deleted < 0 as linenr_T {
                        mark_adjust(
                            (*eap).line2,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            -deleted,
                            0 as linenr_T,
                            kExtmarkNOOP,
                        );
                    }
                    if change_occurred as ::core::ffi::c_int != 0 || deleted != 0 as linenr_T {
                        extmark_splice(
                            curbuf.get(),
                            (*eap).line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            0 as colnr_T,
                            count as ::core::ffi::c_int,
                            0 as colnr_T,
                            old_count,
                            lnum_0 as ::core::ffi::c_int - (*eap).line2 as ::core::ffi::c_int,
                            0 as colnr_T,
                            new_count,
                            kExtmarkUndo,
                        );
                        changed_lines(
                            curbuf.get(),
                            (*eap).line1,
                            0 as colnr_T,
                            (*eap).line2 + 1 as linenr_T,
                            -deleted,
                            true_0 != 0,
                        );
                    }
                    (*curwin.get()).w_cursor.lnum = (*eap).line1;
                    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                }
            }
        }
        xfree(nrs as *mut ::core::ffi::c_void);
        xfree(sortbuf1.get() as *mut ::core::ffi::c_void);
        xfree(sortbuf2.get() as *mut ::core::ffi::c_void);
        vim_regfree(regmatch.regprog);
        if got_int.get() {
            emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
        }
    }
}

pub unsafe fn ex_uniq(mut eap: *mut exarg_T) {
    unsafe {
        let mut match_continue: bool = false;
        let mut next_is_unmatch: bool = false;
        let mut done_lnum: linenr_T = 0;
        let mut delete_lnum: linenr_T = 0;
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut count: linenr_T = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
        let mut keep_only_unique: bool = false_0 != 0;
        let mut keep_only_not_unique: bool = (*eap).forceit != 0;
        let mut deleted: linenr_T = 0 as linenr_T;
        if count <= 1 as linenr_T {
            return;
        }
        if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
            return;
        }
        sortbuf1.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
        sort_flt.set(false_0 != 0);
        sort_nr.set(sort_flt.get());
        sort_rx.set(sort_nr.get());
        sort_lc.set(sort_rx.get());
        sort_ic.set(sort_lc.get());
        sort_abort.set(sort_ic.get());
        let mut change_occurred: bool = false_0 != 0;
        let mut p: *mut ::core::ffi::c_char = (*eap).arg;
        '_uniqend: {
            while *p as ::core::ffi::c_int != NUL {
                if !ascii_iswhite(*p as ::core::ffi::c_int) {
                    if *p as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                        sort_ic.set(true_0 != 0);
                    } else if *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                        sort_lc.set(true_0 != 0);
                    } else if *p as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
                        sort_rx.set(true_0 != 0);
                    } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                        if !keep_only_not_unique {
                            keep_only_unique = true_0 != 0;
                        }
                    } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                        break;
                    } else if (*eap).nextcmd.is_null() && !check_nextcmd(p).is_null() {
                        (*eap).nextcmd = check_nextcmd(p);
                        break;
                    } else if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                        && regmatch.regprog.is_null()
                    {
                        let mut s: *mut ::core::ffi::c_char = skip_regexp_err(
                            p.offset(1 as ::core::ffi::c_int as isize),
                            *p as ::core::ffi::c_int,
                            true_0,
                        );
                        if s.is_null() {
                            break '_uniqend;
                        }
                        *s = NUL as ::core::ffi::c_char;
                        if s == p.offset(1 as ::core::ffi::c_int as isize) {
                            if last_search_pat().is_null() {
                                emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                                break '_uniqend;
                            } else {
                                regmatch.regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
                            }
                        } else {
                            regmatch.regprog =
                                vim_regcomp(p.offset(1 as ::core::ffi::c_int as isize), RE_MAGIC);
                        }
                        if regmatch.regprog.is_null() {
                            break '_uniqend;
                        }
                        p = s;
                        regmatch.rm_ic = p_ic.get() != 0;
                    } else {
                        semsg_c!(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            p,
                        );
                        break '_uniqend;
                    }
                }
                p = p.offset(1);
            }
            let mut lnum: linenr_T = (*eap).line1;
            while lnum <= (*eap).line2 {
                let mut len: ::core::ffi::c_int = ml_get_len(lnum);
                if maxlen < len {
                    maxlen = len;
                }
                if got_int.get() {
                    break '_uniqend;
                }
                lnum += 1;
            }
            sortbuf1
                .set(xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
                    as *mut ::core::ffi::c_char);
            match_continue = false_0 != 0;
            next_is_unmatch = false_0 != 0;
            done_lnum = (*eap).line1 - 1 as linenr_T;
            delete_lnum = 0 as linenr_T;
            let mut i: linenr_T = 0 as linenr_T;
            while i < count {
                let mut get_lnum: linenr_T = (*eap).line1 + i;
                let mut s_0: *mut ::core::ffi::c_char = ml_get(get_lnum);
                let mut len_0: ::core::ffi::c_int = ml_get_len(get_lnum);
                let mut start_col: colnr_T = 0 as colnr_T;
                let mut end_col: colnr_T = len_0 as colnr_T;
                if !regmatch.regprog.is_null()
                    && vim_regexec(&raw mut regmatch, s_0, 0 as colnr_T) as ::core::ffi::c_int != 0
                {
                    if sort_rx.get() {
                        start_col = regmatch.startp[0 as ::core::ffi::c_int as usize]
                            .offset_from(s_0) as colnr_T;
                        end_col = regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(s_0)
                            as colnr_T;
                    } else {
                        start_col = regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(s_0)
                            as colnr_T;
                    }
                } else if !regmatch.regprog.is_null() {
                    end_col = 0 as ::core::ffi::c_int as colnr_T;
                }
                let mut save_c: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                if end_col > 0 as ::core::ffi::c_int {
                    save_c = *s_0.offset(end_col as isize);
                    *s_0.offset(end_col as isize) = NUL as ::core::ffi::c_char;
                }
                let mut is_match: bool = if i > 0 as linenr_T {
                    (string_compare(
                        s_0.offset(start_col as isize) as *const ::core::ffi::c_void,
                        sortbuf1.get() as *const ::core::ffi::c_void,
                    ) == 0) as ::core::ffi::c_int
                } else {
                    false_0
                } != 0;
                delete_lnum = 0 as ::core::ffi::c_int as linenr_T;
                if next_is_unmatch {
                    is_match = false_0 != 0;
                    next_is_unmatch = false_0 != 0;
                }
                if !keep_only_unique && !keep_only_not_unique {
                    if is_match {
                        delete_lnum = get_lnum;
                    } else {
                        strcpy(sortbuf1.get(), s_0.offset(start_col as isize));
                    }
                } else if keep_only_not_unique {
                    if is_match {
                        done_lnum = get_lnum - 1 as linenr_T;
                        delete_lnum = get_lnum;
                        match_continue = true_0 != 0;
                    } else {
                        if i > 0 as linenr_T
                            && !match_continue
                            && get_lnum - 1 as linenr_T > done_lnum
                        {
                            delete_lnum = get_lnum - 1 as linenr_T;
                            next_is_unmatch = true_0 != 0;
                        } else if i >= count - 1 as linenr_T {
                            delete_lnum = get_lnum;
                        }
                        match_continue = false_0 != 0;
                        strcpy(sortbuf1.get(), s_0.offset(start_col as isize));
                    }
                } else if is_match {
                    if !match_continue {
                        delete_lnum = get_lnum - 1 as linenr_T;
                    } else {
                        delete_lnum = get_lnum;
                    }
                    match_continue = true_0 != 0;
                } else {
                    if i == 0 as linenr_T && match_continue as ::core::ffi::c_int != 0 {
                        delete_lnum = get_lnum;
                    }
                    match_continue = false_0 != 0;
                    strcpy(sortbuf1.get(), s_0.offset(start_col as isize));
                }
                if end_col > 0 as ::core::ffi::c_int {
                    *s_0.offset(end_col as isize) = save_c;
                }
                if delete_lnum > 0 as linenr_T {
                    ml_delete(delete_lnum);
                    i = (i as ::core::ffi::c_int
                        - (get_lnum - delete_lnum + 1 as linenr_T) as ::core::ffi::c_int)
                        as linenr_T;
                    count -= 1;
                    deleted += 1;
                    change_occurred = true_0 != 0;
                }
                fast_breakcheck();
                if got_int.get() {
                    break '_uniqend;
                }
                i += 1;
            }
            mark_adjust(
                (*eap).line2 - deleted,
                (*eap).line2,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                -deleted,
                (if change_occurred as ::core::ffi::c_int != 0 {
                    kExtmarkUndo as ::core::ffi::c_int
                } else {
                    kExtmarkNOOP as ::core::ffi::c_int
                }) as ExtmarkOp,
            );
            msgmore(-(deleted as ::core::ffi::c_int));
            if change_occurred {
                changed_lines(
                    curbuf.get(),
                    (*eap).line1,
                    0 as colnr_T,
                    (*eap).line2 + 1 as linenr_T,
                    -deleted,
                    true_0 != 0,
                );
            }
            (*curwin.get()).w_cursor.lnum = (*eap).line1;
            beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        }
        xfree(sortbuf1.get() as *mut ::core::ffi::c_void);
        vim_regfree(regmatch.regprog);
        if got_int.get() {
            emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
        }
    }
}
