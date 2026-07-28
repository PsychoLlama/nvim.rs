//! The thread lists and the submatch sets they carry: comparing,
//! copying and adding a state to a list.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn copy_pim(mut to: *mut nfa_pim_T, mut from: *mut nfa_pim_T) {
    (*to).result = (*from).result;
    (*to).state = (*from).state;
    copy_sub(&raw mut (*to).subs.norm, &raw mut (*from).subs.norm);
    if (*rex.ptr()).nfa_has_zsubexpr != 0 {
        copy_sub(&raw mut (*to).subs.synt, &raw mut (*from).subs.synt);
    }
    (*to).end = (*from).end;
}
pub(crate) unsafe extern "C" fn clear_sub(mut sub: *mut regsub_T) {
    if (*rex.ptr()).reg_match.is_null() {
        memset(
            &raw mut (*sub).list.multi as *mut multipos as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<multipos>().wrapping_mul((*rex.ptr()).nfa_nsubexpr as size_t),
        );
    } else {
        memset(
            &raw mut (*sub).list.line as *mut linepos as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<linepos>().wrapping_mul((*rex.ptr()).nfa_nsubexpr as size_t),
        );
    }
    (*sub).in_use = 0 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn copy_sub(mut to: *mut regsub_T, mut from: *mut regsub_T) {
    (*to).in_use = (*from).in_use;
    if (*from).in_use <= 0 as ::core::ffi::c_int {
        return;
    }
    if (*rex.ptr()).reg_match.is_null() {
        memmove(
            (&raw mut (*to).list.multi as *mut multipos).offset(0 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            (&raw mut (*from).list.multi as *mut multipos).offset(0 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<multipos>().wrapping_mul((*from).in_use as size_t),
        );
        (*to).orig_start_col = (*from).orig_start_col;
    } else {
        memmove(
            (&raw mut (*to).list.line as *mut linepos).offset(0 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            (&raw mut (*from).list.line as *mut linepos).offset(0 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<linepos>().wrapping_mul((*from).in_use as size_t),
        );
    };
}
pub(crate) unsafe extern "C" fn copy_sub_off(mut to: *mut regsub_T, mut from: *mut regsub_T) {
    if (*to).in_use < (*from).in_use {
        (*to).in_use = (*from).in_use;
    }
    if (*from).in_use <= 1 as ::core::ffi::c_int {
        return;
    }
    if (*rex.ptr()).reg_match.is_null() {
        memmove(
            (&raw mut (*to).list.multi as *mut multipos).offset(1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            (&raw mut (*from).list.multi as *mut multipos).offset(1 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<multipos>()
                .wrapping_mul(((*from).in_use - 1 as ::core::ffi::c_int) as size_t),
        );
    } else {
        memmove(
            (&raw mut (*to).list.line as *mut linepos).offset(1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            (&raw mut (*from).list.line as *mut linepos).offset(1 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<linepos>()
                .wrapping_mul(((*from).in_use - 1 as ::core::ffi::c_int) as size_t),
        );
    };
}
pub(crate) unsafe extern "C" fn copy_ze_off(mut to: *mut regsub_T, mut from: *mut regsub_T) {
    if (*rex.ptr()).nfa_has_zend == 0 {
        return;
    }
    if (*rex.ptr()).reg_match.is_null() {
        if (*from).list.multi[0 as ::core::ffi::c_int as usize].end_lnum >= 0 as linenr_T {
            (*to).list.multi[0 as ::core::ffi::c_int as usize].end_lnum =
                (*from).list.multi[0 as ::core::ffi::c_int as usize].end_lnum;
            (*to).list.multi[0 as ::core::ffi::c_int as usize].end_col =
                (*from).list.multi[0 as ::core::ffi::c_int as usize].end_col;
        }
    } else if !(*from).list.line[0 as ::core::ffi::c_int as usize]
        .end
        .is_null()
    {
        (*to).list.line[0 as ::core::ffi::c_int as usize].end =
            (*from).list.line[0 as ::core::ffi::c_int as usize].end;
    }
}
pub(crate) unsafe extern "C" fn sub_equal(
    mut sub1: *mut regsub_T,
    mut sub2: *mut regsub_T,
) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    let mut todo: ::core::ffi::c_int = 0;
    let mut s1: linenr_T = 0;
    let mut s2: linenr_T = 0;
    let mut sp1: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut sp2: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    todo = if (*sub1).in_use > (*sub2).in_use {
        (*sub1).in_use
    } else {
        (*sub2).in_use
    };
    if (*rex.ptr()).reg_match.is_null() {
        i = 0 as ::core::ffi::c_int;
        while i < todo {
            if i < (*sub1).in_use {
                s1 = (*sub1).list.multi[i as usize].start_lnum;
            } else {
                s1 = -1 as ::core::ffi::c_int as linenr_T;
            }
            if i < (*sub2).in_use {
                s2 = (*sub2).list.multi[i as usize].start_lnum;
            } else {
                s2 = -1 as ::core::ffi::c_int as linenr_T;
            }
            if s1 != s2 {
                return false_0 != 0;
            }
            if s1 != -1 as linenr_T
                && (*sub1).list.multi[i as usize].start_col
                    != (*sub2).list.multi[i as usize].start_col
            {
                return false_0 != 0;
            }
            if (*rex.ptr()).nfa_has_backref != 0 {
                if i < (*sub1).in_use {
                    s1 = (*sub1).list.multi[i as usize].end_lnum;
                } else {
                    s1 = -1 as ::core::ffi::c_int as linenr_T;
                }
                if i < (*sub2).in_use {
                    s2 = (*sub2).list.multi[i as usize].end_lnum;
                } else {
                    s2 = -1 as ::core::ffi::c_int as linenr_T;
                }
                if s1 != s2 {
                    return false_0 != 0;
                }
                if s1 != -1 as linenr_T
                    && (*sub1).list.multi[i as usize].end_col
                        != (*sub2).list.multi[i as usize].end_col
                {
                    return false_0 != 0;
                }
            }
            i += 1;
        }
    } else {
        i = 0 as ::core::ffi::c_int;
        while i < todo {
            if i < (*sub1).in_use {
                sp1 = (*sub1).list.line[i as usize].start;
            } else {
                sp1 = ::core::ptr::null_mut::<uint8_t>();
            }
            if i < (*sub2).in_use {
                sp2 = (*sub2).list.line[i as usize].start;
            } else {
                sp2 = ::core::ptr::null_mut::<uint8_t>();
            }
            if sp1 != sp2 {
                return false_0 != 0;
            }
            if (*rex.ptr()).nfa_has_backref != 0 {
                if i < (*sub1).in_use {
                    sp1 = (*sub1).list.line[i as usize].end;
                } else {
                    sp1 = ::core::ptr::null_mut::<uint8_t>();
                }
                if i < (*sub2).in_use {
                    sp2 = (*sub2).list.line[i as usize].end;
                } else {
                    sp2 = ::core::ptr::null_mut::<uint8_t>();
                }
                if sp1 != sp2 {
                    return false_0 != 0;
                }
            }
            i += 1;
        }
    }
    return true_0 != 0;
}
pub(crate) unsafe extern "C" fn has_state_with_pos(
    mut l: *mut nfa_list_T,
    mut state: *mut nfa_state_T,
    mut subs: *mut regsubs_T,
    mut pim: *mut nfa_pim_T,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*l).n {
        let mut thread: *mut nfa_thread_T = (*l).t.offset(i as isize);
        if (*(*thread).state).id == (*state).id
            && sub_equal(&raw mut (*thread).subs.norm, &raw mut (*subs).norm) as ::core::ffi::c_int
                != 0
            && ((*rex.ptr()).nfa_has_zsubexpr == 0
                || sub_equal(&raw mut (*thread).subs.synt, &raw mut (*subs).synt)
                    as ::core::ffi::c_int
                    != 0)
            && pim_equal(&raw mut (*thread).pim, pim) as ::core::ffi::c_int != 0
        {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
pub(crate) unsafe extern "C" fn pim_equal(
    mut one: *const nfa_pim_T,
    mut two: *const nfa_pim_T,
) -> bool {
    let one_unused: bool = one.is_null() || (*one).result == NFA_PIM_UNUSED;
    let two_unused: bool = two.is_null() || (*two).result == NFA_PIM_UNUSED;
    if one_unused {
        return two_unused;
    }
    if two_unused {
        return false_0 != 0;
    }
    if (*(*one).state).id != (*(*two).state).id {
        return false_0 != 0;
    }
    if (*rex.ptr()).reg_match.is_null() {
        return (*one).end.pos.lnum == (*two).end.pos.lnum
            && (*one).end.pos.col == (*two).end.pos.col;
    }
    return (*one).end.ptr == (*two).end.ptr;
}
pub(crate) unsafe extern "C" fn match_follows(
    mut startstate: *const nfa_state_T,
    mut depth: ::core::ffi::c_int,
) -> bool {
    let mut state: *const nfa_state_T = startstate;
    if depth > 10 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    while !state.is_null() {
        match (*state).c {
            -1023 | -947 | -988 | -987 | -986 => return true_0 != 0,
            -1024 => {
                return match_follows((*state).out, depth + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                    || match_follows((*state).out1, depth + 1 as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0;
            }
            -997 | -996 | -993 | -992 | -995 | -994 | -991 | -990 | -985 => {
                state = (*(*state).out1).out;
            }
            -917 | -983 | -916 | -915 | -914 | -913 | -912 | -911 | -910 | -909 | -908 | -907
            | -906 | -905 | -904 | -903 | -902 | -901 | -900 | -899 | -898 | -897 | -896 | -895
            | -894 | -893 | -892 | -891 | -890 | -889 | -888 | -887 | -1021 | -1019 | -1002 => {
                return false_0 != 0;
            }
            _ => {
                if (*state).c > 0 as ::core::ffi::c_int {
                    return false_0 != 0;
                }
                state = (*state).out;
            }
        }
    }
    return false_0 != 0;
}
pub(crate) unsafe extern "C" fn state_in_list(
    mut l: *mut nfa_list_T,
    mut state: *mut nfa_state_T,
    mut subs: *mut regsubs_T,
) -> bool {
    if (*state).lastlist[nfa_ll_index.get() as usize] == (*l).id {
        if (*rex.ptr()).nfa_has_backref == 0
            || has_state_with_pos(l, state, subs, ::core::ptr::null_mut::<nfa_pim_T>())
                as ::core::ffi::c_int
                != 0
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
pub(crate) unsafe extern "C" fn addstate(
    mut l: *mut nfa_list_T,
    mut state: *mut nfa_state_T,
    mut subs_arg: *mut regsubs_T,
    mut pim: *mut nfa_pim_T,
    mut off_arg: ::core::ffi::c_int,
) -> *mut regsubs_T {
    let mut subidx: ::core::ffi::c_int = 0;
    let mut off: ::core::ffi::c_int = off_arg;
    let mut add_here: ::core::ffi::c_int = false_0;
    let mut listindex: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut k: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = false_0;
    let mut thread: *mut nfa_thread_T = ::core::ptr::null_mut::<nfa_thread_T>();
    let mut save_multipos: multipos = multipos {
        start_lnum: 0,
        end_lnum: 0,
        start_col: 0,
        end_col: 0,
    };
    let mut save_in_use: ::core::ffi::c_int = 0;
    let mut save_ptr: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut i: ::core::ffi::c_int = 0;
    let mut sub: *mut regsub_T = ::core::ptr::null_mut::<regsub_T>();
    let mut subs: *mut regsubs_T = subs_arg;
    static temp_subs: GlobalCell<regsubs_T> = GlobalCell::new(regsubs_T {
        norm: regsub_T {
            in_use: 0,
            list: C2Rust_Unnamed_19 {
                multi: [multipos {
                    start_lnum: 0,
                    end_lnum: 0,
                    start_col: 0,
                    end_col: 0,
                }; 10],
            },
            orig_start_col: 0,
        },
        synt: regsub_T {
            in_use: 0,
            list: C2Rust_Unnamed_19 {
                multi: [multipos {
                    start_lnum: 0,
                    end_lnum: 0,
                    start_col: 0,
                    end_col: 0,
                }; 10],
            },
            orig_start_col: 0,
        },
    });
    static depth: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    (*depth.ptr()) += 1;
    if depth.get() >= 5000 as ::core::ffi::c_int || subs.is_null() {
        (*depth.ptr()) -= 1;
        return ::core::ptr::null_mut::<regsubs_T>();
    }
    if off_arg <= -ADDSTATE_HERE_OFFSET {
        add_here = true_0;
        off = 0 as ::core::ffi::c_int;
        listindex = -(off_arg + ADDSTATE_HERE_OFFSET);
    }
    '_skip_add: {
        's_335: {
            match (*state).c {
                -998 | -947 | -946 | -945 | -944 | -943 | -942 | -941 | -940 | -939 | -938
                | -927 | -926 | -925 | -924 | -923 | -922 | -921 | -920 | -919 | -918 | -957
                | -1000 | -1024 | -1022 => {
                    break 's_335;
                }
                -1008 | -1004 => {
                    if (*rex.ptr()).input > (*rex.ptr()).line
                        && *(*rex.ptr()).input as ::core::ffi::c_int != NUL
                        && ((*nfa_endp.ptr()).is_null()
                            || !(*rex.ptr()).reg_match.is_null()
                            || (*rex.ptr()).lnum == (*nfa_endp.get()).se_u.pos.lnum)
                    {
                        break '_skip_add;
                    }
                }
                -956 | -955 | -954 | -953 | -952 | -951 | -950 | -949 | -948 | -937 | -936
                | -935 | -934 | -933 | -932 | -931 | -930 | -929 | -928 | -999 | -1001 | _ => {}
            }
            if (*state).lastlist[nfa_ll_index.get() as usize] == (*l).id
                && (*state).c != NFA_SKIP as ::core::ffi::c_int
            {
                if (*rex.ptr()).nfa_has_backref == 0
                    && pim.is_null()
                    && (*l).has_pim == 0
                    && (*state).c != NFA_MATCH as ::core::ffi::c_int
                {
                    if add_here != 0 {
                        k = 0 as ::core::ffi::c_int;
                        while k < (*l).n && k < listindex {
                            if (*(*(*l).t.offset(k as isize)).state).id == (*state).id {
                                found = true_0;
                                break;
                            } else {
                                k += 1;
                            }
                        }
                    }
                    if add_here == 0 || found != 0 {
                        break '_skip_add;
                    }
                }
                if has_state_with_pos(l, state, subs, pim) {
                    break '_skip_add;
                }
            }
            if (*l).n == (*l).len {
                let newlen: ::core::ffi::c_int = (*l).len * 3 as ::core::ffi::c_int
                    / 2 as ::core::ffi::c_int
                    + 50 as ::core::ffi::c_int;
                let newsize: size_t =
                    (newlen as size_t).wrapping_mul(::core::mem::size_of::<nfa_thread_T>());
                if (newsize >> 10 as ::core::ffi::c_int) as int64_t >= p_mmp.get() {
                    emsg(gettext(
                        (e_pattern_uses_more_memory_than_maxmempattern.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    (*depth.ptr()) -= 1;
                    return ::core::ptr::null_mut::<regsubs_T>();
                }
                if subs != temp_subs.ptr() {
                    copy_sub(&raw mut (*temp_subs.ptr()).norm, &raw mut (*subs).norm);
                    if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                        copy_sub(&raw mut (*temp_subs.ptr()).synt, &raw mut (*subs).synt);
                    }
                    subs = temp_subs.ptr();
                }
                let newt: *mut nfa_thread_T =
                    xrealloc((*l).t as *mut ::core::ffi::c_void, newsize) as *mut nfa_thread_T;
                (*l).t = newt;
                (*l).len = newlen;
            }
            (*state).lastlist[nfa_ll_index.get() as usize] = (*l).id;
            let c2rust_fresh11 = (*l).n;
            (*l).n = (*l).n + 1;
            thread = (*l).t.offset(c2rust_fresh11 as isize);
            (*thread).state = state;
            if pim.is_null() {
                (*thread).pim.result = NFA_PIM_UNUSED;
            } else {
                copy_pim(&raw mut (*thread).pim, pim);
                (*l).has_pim = true_0;
            }
            copy_sub(&raw mut (*thread).subs.norm, &raw mut (*subs).norm);
            if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                copy_sub(&raw mut (*thread).subs.synt, &raw mut (*subs).synt);
            }
        }
        's_888: {
            match (*state).c {
                -1024 => {
                    subs = addstate(l, (*state).out, subs, pim, off_arg);
                    subs = addstate(l, (*state).out1, subs, pim, off_arg);
                    break 's_888;
                }
                -1022 | -999 | -998 => {
                    subs = addstate(l, (*state).out, subs, pim, off_arg);
                    break 's_888;
                }
                -957 | -956 | -955 | -954 | -953 | -952 | -951 | -950 | -949 | -948 | -937
                | -936 | -935 | -934 | -933 | -932 | -931 | -930 | -929 | -928 | -1001 => {
                    if (*state).c == NFA_ZSTART as ::core::ffi::c_int {
                        subidx = 0 as ::core::ffi::c_int;
                        sub = &raw mut (*subs).norm;
                    } else if (*state).c >= NFA_ZOPEN as ::core::ffi::c_int
                        && (*state).c <= NFA_ZOPEN9 as ::core::ffi::c_int
                    {
                        subidx = (*state).c - NFA_ZOPEN as ::core::ffi::c_int;
                        sub = &raw mut (*subs).synt;
                    } else {
                        subidx = (*state).c - NFA_MOPEN as ::core::ffi::c_int;
                        sub = &raw mut (*subs).norm;
                    }
                    save_ptr = ::core::ptr::null_mut::<uint8_t>();
                    memset(
                        &raw mut save_multipos as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        ::core::mem::size_of::<multipos>(),
                    );
                    if (*rex.ptr()).reg_match.is_null() {
                        if subidx < (*sub).in_use {
                            save_multipos = (*sub).list.multi[subidx as usize] as multipos;
                            save_in_use = -1 as ::core::ffi::c_int;
                        } else {
                            save_in_use = (*sub).in_use;
                            i = (*sub).in_use;
                            while i < subidx {
                                (*sub).list.multi[i as usize].start_lnum =
                                    -1 as ::core::ffi::c_int as linenr_T;
                                (*sub).list.multi[i as usize].end_lnum =
                                    -1 as ::core::ffi::c_int as linenr_T;
                                i += 1;
                            }
                            (*sub).in_use = subidx + 1 as ::core::ffi::c_int;
                        }
                        if off == -1 as ::core::ffi::c_int {
                            (*sub).list.multi[subidx as usize].start_lnum =
                                (*rex.ptr()).lnum + 1 as linenr_T;
                            (*sub).list.multi[subidx as usize].start_col =
                                0 as ::core::ffi::c_int as colnr_T;
                        } else {
                            (*sub).list.multi[subidx as usize].start_lnum = (*rex.ptr()).lnum;
                            (*sub).list.multi[subidx as usize].start_col =
                                ((*rex.ptr()).input.offset_from((*rex.ptr()).line) + off as isize)
                                    as colnr_T;
                        }
                        (*sub).list.multi[subidx as usize].end_lnum =
                            -1 as ::core::ffi::c_int as linenr_T;
                    } else {
                        if subidx < (*sub).in_use {
                            save_ptr = (*sub).list.line[subidx as usize].start;
                            save_in_use = -1 as ::core::ffi::c_int;
                        } else {
                            save_in_use = (*sub).in_use;
                            i = (*sub).in_use;
                            while i < subidx {
                                (*sub).list.line[i as usize].start =
                                    ::core::ptr::null_mut::<uint8_t>();
                                (*sub).list.line[i as usize].end =
                                    ::core::ptr::null_mut::<uint8_t>();
                                i += 1;
                            }
                            (*sub).in_use = subidx + 1 as ::core::ffi::c_int;
                        }
                        (*sub).list.line[subidx as usize].start =
                            (*rex.ptr()).input.offset(off as isize);
                    }
                    subs = addstate(l, (*state).out, subs, pim, off_arg);
                    if subs.is_null() {
                        break 's_888;
                    } else {
                        if (*state).c >= NFA_ZOPEN as ::core::ffi::c_int
                            && (*state).c <= NFA_ZOPEN9 as ::core::ffi::c_int
                        {
                            sub = &raw mut (*subs).synt;
                        } else {
                            sub = &raw mut (*subs).norm;
                        }
                        if save_in_use == -1 as ::core::ffi::c_int {
                            if (*rex.ptr()).reg_match.is_null() {
                                (*sub).list.multi[subidx as usize] = save_multipos as multipos;
                            } else {
                                (*sub).list.line[subidx as usize].start = save_ptr;
                            }
                        } else {
                            (*sub).in_use = save_in_use;
                        }
                        break 's_888;
                    }
                }
                -947 => {
                    if (*rex.ptr()).nfa_has_zend != 0
                        && (if (*rex.ptr()).reg_match.is_null() {
                            ((*subs).norm.list.multi[0 as ::core::ffi::c_int as usize].end_lnum
                                >= 0 as linenr_T) as ::core::ffi::c_int
                        } else {
                            !(*subs).norm.list.line[0 as ::core::ffi::c_int as usize]
                                .end
                                .is_null() as ::core::ffi::c_int
                        }) != 0
                    {
                        subs = addstate(l, (*state).out, subs, pim, off_arg);
                        break 's_888;
                    }
                }
                -946 | -945 | -944 | -943 | -942 | -941 | -940 | -939 | -938 | -927 | -926
                | -925 | -924 | -923 | -922 | -921 | -920 | -919 | -918 | -1000 => {}
                -1023 | _ => {
                    break 's_888;
                }
            }
            if (*state).c == NFA_ZEND as ::core::ffi::c_int {
                subidx = 0 as ::core::ffi::c_int;
                sub = &raw mut (*subs).norm;
            } else if (*state).c >= NFA_ZCLOSE as ::core::ffi::c_int
                && (*state).c <= NFA_ZCLOSE9 as ::core::ffi::c_int
            {
                subidx = (*state).c - NFA_ZCLOSE as ::core::ffi::c_int;
                sub = &raw mut (*subs).synt;
            } else {
                subidx = (*state).c - NFA_MCLOSE as ::core::ffi::c_int;
                sub = &raw mut (*subs).norm;
            }
            save_in_use = (*sub).in_use;
            if (*sub).in_use <= subidx {
                (*sub).in_use = subidx + 1 as ::core::ffi::c_int;
            }
            if (*rex.ptr()).reg_match.is_null() {
                save_multipos = (*sub).list.multi[subidx as usize] as multipos;
                if off == -1 as ::core::ffi::c_int {
                    (*sub).list.multi[subidx as usize].end_lnum = (*rex.ptr()).lnum + 1 as linenr_T;
                    (*sub).list.multi[subidx as usize].end_col = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    (*sub).list.multi[subidx as usize].end_lnum = (*rex.ptr()).lnum;
                    (*sub).list.multi[subidx as usize].end_col =
                        ((*rex.ptr()).input.offset_from((*rex.ptr()).line) + off as isize)
                            as colnr_T;
                }
                save_ptr = ::core::ptr::null_mut::<uint8_t>();
            } else {
                save_ptr = (*sub).list.line[subidx as usize].end;
                (*sub).list.line[subidx as usize].end = (*rex.ptr()).input.offset(off as isize);
                memset(
                    &raw mut save_multipos as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<multipos>(),
                );
            }
            subs = addstate(l, (*state).out, subs, pim, off_arg);
            if !subs.is_null() {
                if (*state).c >= NFA_ZCLOSE as ::core::ffi::c_int
                    && (*state).c <= NFA_ZCLOSE9 as ::core::ffi::c_int
                {
                    sub = &raw mut (*subs).synt;
                } else {
                    sub = &raw mut (*subs).norm;
                }
                if (*rex.ptr()).reg_match.is_null() {
                    (*sub).list.multi[subidx as usize] = save_multipos as multipos;
                } else {
                    (*sub).list.line[subidx as usize].end = save_ptr;
                }
                (*sub).in_use = save_in_use;
            }
        }
        (*depth.ptr()) -= 1;
        return subs;
    }
    (*depth.ptr()) -= 1;
    return subs;
}
pub(crate) unsafe extern "C" fn addstate_here(
    mut l: *mut nfa_list_T,
    mut state: *mut nfa_state_T,
    mut subs: *mut regsubs_T,
    mut pim: *mut nfa_pim_T,
    mut ip: *mut ::core::ffi::c_int,
) -> *mut regsubs_T {
    let mut tlen: ::core::ffi::c_int = (*l).n;
    let mut count: ::core::ffi::c_int = 0;
    let mut listidx: ::core::ffi::c_int = *ip;
    let mut r: *mut regsubs_T = addstate(l, state, subs, pim, -listidx - ADDSTATE_HERE_OFFSET);
    if r.is_null() {
        return ::core::ptr::null_mut::<regsubs_T>();
    }
    if listidx + 1 as ::core::ffi::c_int == tlen {
        return r;
    }
    count = (*l).n - tlen;
    if count == 0 as ::core::ffi::c_int {
        return r;
    }
    if count == 1 as ::core::ffi::c_int {
        *(*l).t.offset(listidx as isize) =
            *(*l).t.offset(((*l).n - 1 as ::core::ffi::c_int) as isize);
    } else if count > 1 as ::core::ffi::c_int {
        if (*l).n + count - 1 as ::core::ffi::c_int >= (*l).len {
            let newlen: ::core::ffi::c_int = (*l).len * 3 as ::core::ffi::c_int
                / 2 as ::core::ffi::c_int
                + 50 as ::core::ffi::c_int;
            let newsize: size_t =
                (newlen as size_t).wrapping_mul(::core::mem::size_of::<nfa_thread_T>());
            if (newsize >> 10 as ::core::ffi::c_int) as int64_t >= p_mmp.get() {
                emsg(gettext(
                    (e_pattern_uses_more_memory_than_maxmempattern.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                return ::core::ptr::null_mut::<regsubs_T>();
            }
            let newl: *mut nfa_thread_T = xmalloc(newsize) as *mut nfa_thread_T;
            (*l).len = newlen;
            memmove(
                newl.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                (*l).t.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                ::core::mem::size_of::<nfa_thread_T>().wrapping_mul(listidx as size_t),
            );
            memmove(
                newl.offset(listidx as isize) as *mut ::core::ffi::c_void,
                (*l).t.offset(((*l).n - count) as isize) as *const ::core::ffi::c_void,
                ::core::mem::size_of::<nfa_thread_T>().wrapping_mul(count as size_t),
            );
            memmove(
                newl.offset((listidx + count) as isize) as *mut ::core::ffi::c_void,
                (*l).t.offset((listidx + 1 as ::core::ffi::c_int) as isize)
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<nfa_thread_T>()
                    .wrapping_mul(((*l).n - count - listidx - 1 as ::core::ffi::c_int) as size_t),
            );
            xfree((*l).t as *mut ::core::ffi::c_void);
            (*l).t = newl;
        } else {
            memmove(
                (*l).t.offset((listidx + count) as isize) as *mut ::core::ffi::c_void,
                (*l).t.offset((listidx + 1 as ::core::ffi::c_int) as isize)
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<nfa_thread_T>()
                    .wrapping_mul(((*l).n - listidx - 1 as ::core::ffi::c_int) as size_t),
            );
            memmove(
                (*l).t.offset(listidx as isize) as *mut ::core::ffi::c_void,
                (*l).t.offset(((*l).n - 1 as ::core::ffi::c_int) as isize)
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<nfa_thread_T>().wrapping_mul(count as size_t),
            );
        }
    }
    (*l).n -= 1;
    *ip = listidx - 1 as ::core::ffi::c_int;
    return r;
}
