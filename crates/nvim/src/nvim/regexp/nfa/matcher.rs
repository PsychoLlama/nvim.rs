//! The thread-list match loop itself.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn nfa_regmatch(
    mut prog: *mut nfa_regprog_T,
    mut start: *mut nfa_state_T,
    mut submatch: *mut regsubs_T,
    mut m: *mut regsubs_T,
) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = false_0;
    let mut flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut go_to_nextline: bool = false_0 != 0;
    let mut t: *mut nfa_thread_T = ::core::ptr::null_mut::<nfa_thread_T>();
    let mut list: [nfa_list_T; 2] = [nfa_list_T {
        t: ::core::ptr::null_mut::<nfa_thread_T>(),
        n: 0,
        len: 0,
        id: 0,
        has_pim: 0,
    }; 2];
    let mut listidx: ::core::ffi::c_int = 0;
    let mut thislist: *mut nfa_list_T = ::core::ptr::null_mut::<nfa_list_T>();
    let mut nextlist: *mut nfa_list_T = ::core::ptr::null_mut::<nfa_list_T>();
    let mut listids: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut listids_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut add_state: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    let mut add_here: bool = false;
    let mut add_count: ::core::ffi::c_int = 0;
    let mut add_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut toplevel: ::core::ffi::c_int =
        ((*start).c == NFA_MOPEN as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut r: *mut regsubs_T = ::core::ptr::null_mut::<regsubs_T>();
    reg_breakcheck();
    if got_int.get() {
        return false_0;
    }
    if nfa_did_time_out() != 0 {
        return false_0;
    }
    nfa_match.set(false_0);
    let mut size: size_t = (((*prog).nstate + 1 as ::core::ffi::c_int) as size_t)
        .wrapping_mul(::core::mem::size_of::<nfa_thread_T>());
    list[0 as ::core::ffi::c_int as usize].t = xmalloc(size) as *mut nfa_thread_T;
    list[0 as ::core::ffi::c_int as usize].len = (*prog).nstate + 1 as ::core::ffi::c_int;
    list[1 as ::core::ffi::c_int as usize].t = xmalloc(size) as *mut nfa_thread_T;
    list[1 as ::core::ffi::c_int as usize].len = (*prog).nstate + 1 as ::core::ffi::c_int;
    thislist = (&raw mut list as *mut nfa_list_T).offset(0 as ::core::ffi::c_int as isize);
    (*thislist).n = 0 as ::core::ffi::c_int;
    (*thislist).has_pim = false_0;
    nextlist = (&raw mut list as *mut nfa_list_T).offset(1 as ::core::ffi::c_int as isize);
    (*nextlist).n = 0 as ::core::ffi::c_int;
    (*nextlist).has_pim = false_0;
    (*thislist).id = (*rex.ptr()).nfa_listid + 1 as ::core::ffi::c_int;
    if toplevel != 0 {
        if (*rex.ptr()).reg_match.is_null() {
            (*m).norm.list.multi[0 as ::core::ffi::c_int as usize].start_lnum = (*rex.ptr()).lnum;
            (*m).norm.list.multi[0 as ::core::ffi::c_int as usize].start_col =
                (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
            (*m).norm.orig_start_col =
                (*m).norm.list.multi[0 as ::core::ffi::c_int as usize].start_col;
        } else {
            (*m).norm.list.line[0 as ::core::ffi::c_int as usize].start = (*rex.ptr()).input;
        }
        (*m).norm.in_use = 1 as ::core::ffi::c_int;
        r = addstate(
            thislist,
            (*start).out,
            m,
            ::core::ptr::null_mut::<nfa_pim_T>(),
            0 as ::core::ffi::c_int,
        );
    } else {
        r = addstate(
            thislist,
            start,
            m,
            ::core::ptr::null_mut::<nfa_pim_T>(),
            0 as ::core::ffi::c_int,
        );
    }
    '_theend: {
        if r.is_null() {
            nfa_match.set(NFA_TOO_EXPENSIVE as ::core::ffi::c_int);
        } else {
            loop {
                let mut curc: ::core::ffi::c_int =
                    utf_ptr2char((*rex.ptr()).input as *mut ::core::ffi::c_char);
                let mut clen: ::core::ffi::c_int =
                    utfc_ptr2len((*rex.ptr()).input as *mut ::core::ffi::c_char);
                if curc == NUL {
                    clen = 0 as ::core::ffi::c_int;
                    go_to_nextline = false_0 != 0;
                }
                thislist = (&raw mut list as *mut nfa_list_T).offset(flag as isize);
                flag ^= 1 as ::core::ffi::c_int;
                nextlist = (&raw mut list as *mut nfa_list_T).offset(flag as isize);
                (*nextlist).n = 0 as ::core::ffi::c_int;
                (*nextlist).has_pim = false_0;
                (*rex.ptr()).nfa_listid += 1;
                if (*prog).re_engine
                    == AUTOMATIC_ENGINE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*rex.ptr()).nfa_listid >= NFA_MAX_STATES as ::core::ffi::c_int
                {
                    nfa_match.set(NFA_TOO_EXPENSIVE as ::core::ffi::c_int);
                    break '_theend;
                } else {
                    (*thislist).id = (*rex.ptr()).nfa_listid;
                    (*nextlist).id = (*rex.ptr()).nfa_listid + 1 as ::core::ffi::c_int;
                    if (*thislist).n == 0 as ::core::ffi::c_int {
                        break '_theend;
                    }
                    listidx = 0 as ::core::ffi::c_int;
                    '_nextchar: {
                        while listidx < (*thislist).n {
                            reg_breakcheck();
                            if got_int.get() {
                                break;
                            }
                            if !(*nfa_time_limit.ptr()).is_null() && {
                                (*nfa_time_count.ptr()) += 1;
                                nfa_time_count.get() == 20 as ::core::ffi::c_int
                            } {
                                nfa_time_count.set(0 as ::core::ffi::c_int);
                                if nfa_did_time_out() != 0 {
                                    break;
                                }
                            }
                            t = (*thislist).t.offset(listidx as isize);
                            add_state = ::core::ptr::null_mut::<nfa_state_T>();
                            add_here = false_0 != 0;
                            add_count = 0 as ::core::ffi::c_int;
                            match (*(*t).state).c {
                                -1023 => {
                                    if !(!(*rex.ptr()).reg_icombine
                                        && (*rex.ptr()).input != (*rex.ptr()).line
                                        && utf_iscomposing_legacy(curc) as ::core::ffi::c_int != 0)
                                    {
                                        nfa_match.set(true_0);
                                        copy_sub(
                                            &raw mut (*submatch).norm,
                                            &raw mut (*t).subs.norm,
                                        );
                                        if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                            copy_sub(
                                                &raw mut (*submatch).synt,
                                                &raw mut (*t).subs.synt,
                                            );
                                        }
                                        if (*nextlist).n == 0 as ::core::ffi::c_int {
                                            clen = 0 as ::core::ffi::c_int;
                                        }
                                        break '_nextchar;
                                    }
                                }
                                -988 | -987 | -986 => {
                                    if !(!(*nfa_endp.ptr()).is_null()
                                        && (if (*rex.ptr()).reg_match.is_null() {
                                            ((*rex.ptr()).lnum != (*nfa_endp.get()).se_u.pos.lnum
                                                || (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                    as ::core::ffi::c_int
                                                    != (*nfa_endp.get()).se_u.pos.col)
                                                as ::core::ffi::c_int
                                        } else {
                                            ((*rex.ptr()).input != (*nfa_endp.get()).se_u.ptr)
                                                as ::core::ffi::c_int
                                        }) != 0)
                                    {
                                        if (*(*t).state).c
                                            != NFA_END_INVISIBLE_NEG as ::core::ffi::c_int
                                        {
                                            copy_sub(&raw mut (*m).norm, &raw mut (*t).subs.norm);
                                            if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                                copy_sub(
                                                    &raw mut (*m).synt,
                                                    &raw mut (*t).subs.synt,
                                                );
                                            }
                                        }
                                        nfa_match.set(true_0);
                                        if (*nextlist).n == 0 as ::core::ffi::c_int {
                                            clen = 0 as ::core::ffi::c_int;
                                        }
                                        break '_nextchar;
                                    }
                                }
                                -997 | -996 | -995 | -994 | -993 | -992 | -991 | -990 => {
                                    if (*t).pim.result != NFA_PIM_UNUSED
                                        || (*(*t).state).c
                                            == NFA_START_INVISIBLE_FIRST as ::core::ffi::c_int
                                        || (*(*t).state).c
                                            == NFA_START_INVISIBLE_NEG_FIRST as ::core::ffi::c_int
                                        || (*(*t).state).c
                                            == NFA_START_INVISIBLE_BEFORE_FIRST
                                                as ::core::ffi::c_int
                                        || (*(*t).state).c
                                            == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
                                                as ::core::ffi::c_int
                                    {
                                        let mut in_use: ::core::ffi::c_int = (*m).norm.in_use;
                                        copy_sub_off(&raw mut (*m).norm, &raw mut (*t).subs.norm);
                                        if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                            copy_sub_off(
                                                &raw mut (*m).synt,
                                                &raw mut (*t).subs.synt,
                                            );
                                        }
                                        result = recursive_regmatch(
                                            (*t).state,
                                            ::core::ptr::null_mut::<nfa_pim_T>(),
                                            prog,
                                            submatch,
                                            m,
                                            &raw mut listids,
                                            &raw mut listids_len,
                                        );
                                        if result == NFA_TOO_EXPENSIVE as ::core::ffi::c_int {
                                            nfa_match.set(result);
                                            break '_theend;
                                        } else {
                                            if result
                                                != ((*(*t).state).c
                                                    == NFA_START_INVISIBLE_NEG
                                                        as ::core::ffi::c_int
                                                    || (*(*t).state).c
                                                        == NFA_START_INVISIBLE_NEG_FIRST
                                                            as ::core::ffi::c_int
                                                    || (*(*t).state).c
                                                        == NFA_START_INVISIBLE_BEFORE_NEG
                                                            as ::core::ffi::c_int
                                                    || (*(*t).state).c
                                                        == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
                                                            as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            {
                                                copy_sub_off(
                                                    &raw mut (*t).subs.norm,
                                                    &raw mut (*m).norm,
                                                );
                                                if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                                    copy_sub_off(
                                                        &raw mut (*t).subs.synt,
                                                        &raw mut (*m).synt,
                                                    );
                                                }
                                                copy_ze_off(
                                                    &raw mut (*t).subs.norm,
                                                    &raw mut (*m).norm,
                                                );
                                                add_here = true_0 != 0;
                                                add_state = (*(*(*t).state).out1).out;
                                            }
                                            (*m).norm.in_use = in_use;
                                        }
                                    } else {
                                        let mut pim: nfa_pim_T = nfa_pim_T {
                                            result: 0,
                                            state: ::core::ptr::null_mut::<nfa_state_T>(),
                                            subs: regsubs_T {
                                                norm: regsub_T {
                                                    in_use: 0,
                                                    list: C2Rust_Unnamed_19 {
                                                        multi: [multipos {
                                                            start_lnum: 0,
                                                            end_lnum: 0,
                                                            start_col: 0,
                                                            end_col: 0,
                                                        };
                                                            10],
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
                                                        };
                                                            10],
                                                    },
                                                    orig_start_col: 0,
                                                },
                                            },
                                            end: C2Rust_Unnamed_20 {
                                                pos: lpos_T { lnum: 0, col: 0 },
                                            },
                                        };
                                        pim.state = (*t).state;
                                        pim.result = NFA_PIM_TODO;
                                        pim.subs.norm.in_use = 0 as ::core::ffi::c_int;
                                        pim.subs.synt.in_use = 0 as ::core::ffi::c_int;
                                        if (*rex.ptr()).reg_match.is_null() {
                                            pim.end.pos.col =
                                                (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                    as ::core::ffi::c_int
                                                    as colnr_T;
                                            pim.end.pos.lnum = (*rex.ptr()).lnum;
                                        } else {
                                            pim.end.ptr = (*rex.ptr()).input;
                                        }
                                        if addstate_here(
                                            thislist,
                                            (*(*(*t).state).out1).out,
                                            &raw mut (*t).subs,
                                            &raw mut pim,
                                            &raw mut listidx,
                                        )
                                        .is_null()
                                        {
                                            nfa_match.set(NFA_TOO_EXPENSIVE as ::core::ffi::c_int);
                                            break '_theend;
                                        }
                                    }
                                }
                                -989 => {
                                    let mut skip: *mut nfa_state_T =
                                        ::core::ptr::null_mut::<nfa_state_T>();
                                    if state_in_list(
                                        nextlist,
                                        (*(*(*t).state).out1).out,
                                        &raw mut (*t).subs,
                                    ) {
                                        skip = (*(*(*t).state).out1).out;
                                    } else if state_in_list(
                                        nextlist,
                                        (*(*(*(*t).state).out1).out).out,
                                        &raw mut (*t).subs,
                                    ) {
                                        skip = (*(*(*(*t).state).out1).out).out;
                                    } else if state_in_list(
                                        thislist,
                                        (*(*(*(*t).state).out1).out).out,
                                        &raw mut (*t).subs,
                                    ) {
                                        skip = (*(*(*(*t).state).out1).out).out;
                                    }
                                    if skip.is_null() {
                                        copy_sub_off(&raw mut (*m).norm, &raw mut (*t).subs.norm);
                                        if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                            copy_sub_off(
                                                &raw mut (*m).synt,
                                                &raw mut (*t).subs.synt,
                                            );
                                        }
                                        result = recursive_regmatch(
                                            (*t).state,
                                            ::core::ptr::null_mut::<nfa_pim_T>(),
                                            prog,
                                            submatch,
                                            m,
                                            &raw mut listids,
                                            &raw mut listids_len,
                                        );
                                        if result == NFA_TOO_EXPENSIVE as ::core::ffi::c_int {
                                            nfa_match.set(result);
                                            break '_theend;
                                        } else if result != 0 {
                                            let mut bytelen: ::core::ffi::c_int = 0;
                                            copy_sub_off(
                                                &raw mut (*t).subs.norm,
                                                &raw mut (*m).norm,
                                            );
                                            if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                                copy_sub_off(
                                                    &raw mut (*t).subs.synt,
                                                    &raw mut (*m).synt,
                                                );
                                            }
                                            if (*rex.ptr()).reg_match.is_null() {
                                                bytelen = (*m).norm.list.multi
                                                    [0 as ::core::ffi::c_int as usize]
                                                    .end_col
                                                    as ::core::ffi::c_int
                                                    - (*rex.ptr())
                                                        .input
                                                        .offset_from((*rex.ptr()).line)
                                                        as ::core::ffi::c_int;
                                            } else {
                                                bytelen = (*m).norm.list.line
                                                    [0 as ::core::ffi::c_int as usize]
                                                    .end
                                                    .offset_from((*rex.ptr()).input)
                                                    as ::core::ffi::c_int;
                                            }
                                            if bytelen == 0 as ::core::ffi::c_int {
                                                add_here = true_0 != 0;
                                                add_state = (*(*(*(*t).state).out1).out).out;
                                            } else if bytelen <= clen {
                                                add_state = (*(*(*(*t).state).out1).out).out;
                                                add_off = clen;
                                            } else {
                                                add_state = (*(*(*t).state).out1).out;
                                                add_off = bytelen;
                                                add_count = bytelen - clen;
                                            }
                                        }
                                    }
                                }
                                -1008 => {
                                    if (*rex.ptr()).input == (*rex.ptr()).line {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -1007 => {
                                    if curc == NUL {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -1006 => {
                                    result = true_0;
                                    if curc == NUL {
                                        result = false_0;
                                    } else {
                                        let mut this_class: ::core::ffi::c_int = 0;
                                        this_class = mb_get_class_tab(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                            &raw mut (*(*rex.ptr()).reg_buf).b_chartab
                                                as *mut uint64_t,
                                        );
                                        if this_class <= 1 as ::core::ffi::c_int {
                                            result = false_0;
                                        } else if reg_prev_class() == this_class {
                                            result = false_0;
                                        }
                                    }
                                    if result != 0 {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -1005 => {
                                    result = true_0;
                                    if (*rex.ptr()).input == (*rex.ptr()).line {
                                        result = false_0;
                                    } else {
                                        let mut this_class_0: ::core::ffi::c_int = 0;
                                        let mut prev_class: ::core::ffi::c_int = 0;
                                        this_class_0 = mb_get_class_tab(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                            &raw mut (*(*rex.ptr()).reg_buf).b_chartab
                                                as *mut uint64_t,
                                        );
                                        prev_class = reg_prev_class();
                                        if this_class_0 == prev_class
                                            || prev_class == 0 as ::core::ffi::c_int
                                            || prev_class == 1 as ::core::ffi::c_int
                                        {
                                            result = false_0;
                                        }
                                    }
                                    if result != 0 {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -1004 => {
                                    if (*rex.ptr()).lnum == 0 as linenr_T
                                        && (*rex.ptr()).input == (*rex.ptr()).line
                                        && (!(*rex.ptr()).reg_match.is_null()
                                            || (*rex.ptr()).reg_firstlnum == 1 as linenr_T)
                                    {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -1003 => {
                                    if (*rex.ptr()).lnum == (*rex.ptr()).reg_maxline && curc == NUL
                                    {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -985 => {
                                    let mut mc: ::core::ffi::c_int = curc;
                                    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut end: *mut nfa_state_T =
                                        ::core::ptr::null_mut::<nfa_state_T>();
                                    let mut sta: *mut nfa_state_T =
                                        ::core::ptr::null_mut::<nfa_state_T>();
                                    let mut cchars: [::core::ffi::c_int; 6] = [0; 6];
                                    let mut ccount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut j: ::core::ffi::c_int = 0;
                                    sta = (*(*t).state).out;
                                    len = 0 as ::core::ffi::c_int;
                                    if utf_iscomposing_legacy((*sta).c) {
                                        len += utf_char2len(mc);
                                    }
                                    if (*rex.ptr()).reg_icombine as ::core::ffi::c_int != 0
                                        && len == 0 as ::core::ffi::c_int
                                    {
                                        if (*sta).c != curc {
                                            result = FAIL;
                                        } else {
                                            result = OK;
                                        }
                                        while (*sta).c != NFA_END_COMPOSING as ::core::ffi::c_int {
                                            sta = (*sta).out;
                                        }
                                    } else if len > 0 as ::core::ffi::c_int || mc == (*sta).c {
                                        if len == 0 as ::core::ffi::c_int {
                                            len += utf_char2len(mc);
                                            sta = (*sta).out;
                                        }
                                        while len < clen {
                                            mc = utf_ptr2char(
                                                ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                                                    .offset(len as isize),
                                            );
                                            let c2rust_fresh9 = ccount;
                                            ccount = ccount + 1;
                                            cchars[c2rust_fresh9 as usize] = mc;
                                            len += utf_char2len(mc);
                                            if ccount == MAX_MCO {
                                                break;
                                            }
                                        }
                                        result = OK;
                                        while (*sta).c != NFA_END_COMPOSING as ::core::ffi::c_int {
                                            j = 0 as ::core::ffi::c_int;
                                            while j < ccount {
                                                if cchars[j as usize] == (*sta).c {
                                                    break;
                                                }
                                                j += 1;
                                            }
                                            if j == ccount {
                                                result = FAIL;
                                                break;
                                            } else {
                                                sta = (*sta).out;
                                            }
                                        }
                                    } else {
                                        result = FAIL;
                                    }
                                    end = (*(*t).state).out1;
                                    if result != 0 {
                                        add_state = (*end).out;
                                        add_off = clen;
                                    }
                                }
                                -1002 => {
                                    if curc == NUL
                                        && !(*rex.ptr()).reg_line_lbr
                                        && (*rex.ptr()).reg_match.is_null()
                                        && (*rex.ptr()).lnum <= (*rex.ptr()).reg_maxline
                                    {
                                        go_to_nextline = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                        add_off = -1 as ::core::ffi::c_int;
                                    } else if curc == '\n' as ::core::ffi::c_int
                                        && (*rex.ptr()).reg_line_lbr as ::core::ffi::c_int != 0
                                    {
                                        add_state = (*(*t).state).out;
                                        add_off = 1 as ::core::ffi::c_int;
                                    }
                                }
                                -1021 | -1019 => {
                                    let mut state: *mut nfa_state_T =
                                        ::core::ptr::null_mut::<nfa_state_T>();
                                    let mut result_if_matched: ::core::ffi::c_int = 0;
                                    let mut c1: ::core::ffi::c_int = 0;
                                    let mut c2: ::core::ffi::c_int = 0;
                                    if curc != NUL {
                                        state = (*(*t).state).out;
                                        result_if_matched = ((*(*t).state).c
                                            == NFA_START_COLL as ::core::ffi::c_int)
                                            as ::core::ffi::c_int;
                                        loop {
                                            if (*state).c == NFA_COMPOSING as ::core::ffi::c_int {
                                                let mut mc_0: ::core::ffi::c_int = curc;
                                                let mut len_0: ::core::ffi::c_int =
                                                    0 as ::core::ffi::c_int;
                                                let mut end_0: *mut nfa_state_T =
                                                    ::core::ptr::null_mut::<nfa_state_T>();
                                                let mut sta_0: *mut nfa_state_T =
                                                    ::core::ptr::null_mut::<nfa_state_T>();
                                                let mut cchars_0: [::core::ffi::c_int; 6] = [0; 6];
                                                let mut ccount_0: ::core::ffi::c_int =
                                                    0 as ::core::ffi::c_int;
                                                let mut j_0: ::core::ffi::c_int = 0;
                                                sta_0 = (*(*(*t).state).out).out;
                                                if utf_iscomposing_legacy((*sta_0).c) {
                                                    len_0 += utf_char2len(mc_0);
                                                }
                                                if (*rex.ptr()).reg_icombine as ::core::ffi::c_int
                                                    != 0
                                                    && len_0 == 0 as ::core::ffi::c_int
                                                {
                                                    if (*sta_0).c != curc {
                                                        result = FAIL;
                                                    } else {
                                                        result = OK;
                                                    }
                                                    while (*sta_0).c
                                                        != NFA_END_COMPOSING as ::core::ffi::c_int
                                                    {
                                                        sta_0 = (*sta_0).out;
                                                    }
                                                } else if len_0 > 0 as ::core::ffi::c_int
                                                    || mc_0 == (*sta_0).c
                                                {
                                                    if len_0 == 0 as ::core::ffi::c_int {
                                                        len_0 += utf_char2len(mc_0);
                                                        sta_0 = (*sta_0).out;
                                                    }
                                                    while len_0 < clen {
                                                        mc_0 = utf_ptr2char(
                                                            ((*rex.ptr()).input
                                                                as *mut ::core::ffi::c_char)
                                                                .offset(len_0 as isize),
                                                        );
                                                        let c2rust_fresh10 = ccount_0;
                                                        ccount_0 = ccount_0 + 1;
                                                        cchars_0[c2rust_fresh10 as usize] = mc_0;
                                                        len_0 += utf_char2len(mc_0);
                                                        if ccount_0 == MAX_MCO {
                                                            break;
                                                        }
                                                    }
                                                    result = OK;
                                                    while (*sta_0).c
                                                        != NFA_END_COMPOSING as ::core::ffi::c_int
                                                    {
                                                        j_0 = 0 as ::core::ffi::c_int;
                                                        while j_0 < ccount_0 {
                                                            if cchars_0[j_0 as usize] == (*sta_0).c
                                                            {
                                                                break;
                                                            }
                                                            j_0 += 1;
                                                        }
                                                        if j_0 == ccount_0 {
                                                            result = FAIL;
                                                            break;
                                                        } else {
                                                            sta_0 = (*sta_0).out;
                                                        }
                                                    }
                                                } else {
                                                    result = FAIL;
                                                }
                                                if !(*(*(*t).state).out).out1.is_null()
                                                    && (*(*(*(*t).state).out).out1).c
                                                        == NFA_END_COMPOSING as ::core::ffi::c_int
                                                {
                                                    end_0 = (*(*(*t).state).out).out1;
                                                    if result != 0 {
                                                        add_state = (*end_0).out;
                                                        add_off = clen;
                                                    }
                                                }
                                                break;
                                            } else if (*state).c
                                                == NFA_END_COLL as ::core::ffi::c_int
                                            {
                                                result =
                                                    (result_if_matched == 0) as ::core::ffi::c_int;
                                                break;
                                            } else {
                                                if (*state).c == NFA_RANGE_MIN as ::core::ffi::c_int
                                                {
                                                    c1 = (*state).val;
                                                    state = (*state).out;
                                                    c2 = (*state).val;
                                                    if curc >= c1 && curc <= c2 {
                                                        result = result_if_matched;
                                                        break;
                                                    } else if (*rex.ptr()).reg_ic {
                                                        let mut curc_low: ::core::ffi::c_int =
                                                            utf_fold(curc);
                                                        let mut done: ::core::ffi::c_int = false_0;
                                                        while c1 <= c2 {
                                                            if utf_fold(c1) == curc_low {
                                                                result = result_if_matched;
                                                                done = true_0;
                                                                break;
                                                            } else {
                                                                c1 += 1;
                                                            }
                                                        }
                                                        if done != 0 {
                                                            break;
                                                        }
                                                    }
                                                } else if if (*state).c < 0 as ::core::ffi::c_int {
                                                    check_char_class((*state).c, curc)
                                                } else {
                                                    (curc == (*state).c
                                                        || (*rex.ptr()).reg_ic
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && utf_fold(curc)
                                                                == utf_fold((*state).c))
                                                        as ::core::ffi::c_int
                                                } != 0
                                                {
                                                    result = result_if_matched;
                                                    break;
                                                }
                                                state = (*state).out;
                                            }
                                        }
                                        if result != 0 {
                                            add_state = (*(*(*t).state).out1).out;
                                            add_off = clen;
                                        }
                                    }
                                }
                                -917 => {
                                    if curc > 0 as ::core::ffi::c_int {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -983 => {
                                    if utf_iscomposing_legacy(curc) {
                                        add_off = clen;
                                    } else {
                                        add_here = true_0 != 0;
                                        add_off = 0 as ::core::ffi::c_int;
                                    }
                                    add_state = (*(*t).state).out;
                                }
                                -916 => {
                                    result = vim_isIDc(curc) as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -915 => {
                                    result = (!ascii_isdigit(curc)
                                        && vim_isIDc(curc) as ::core::ffi::c_int != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -914 => {
                                    result = vim_iswordp_buf(
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        (*rex.ptr()).reg_buf,
                                    )
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -913 => {
                                    result = (!ascii_isdigit(curc)
                                        && vim_iswordp_buf(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                            (*rex.ptr()).reg_buf,
                                        )
                                            as ::core::ffi::c_int
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -912 => {
                                    result = vim_isfilec(curc) as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -911 => {
                                    result = (!ascii_isdigit(curc)
                                        && vim_isfilec(curc) as ::core::ffi::c_int != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -910 => {
                                    result = vim_isprintc(utf_ptr2char(
                                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                    ))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -909 => {
                                    result = (!ascii_isdigit(curc)
                                        && vim_isprintc(utf_ptr2char(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        ))
                                            as ::core::ffi::c_int
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -908 => {
                                    result = ascii_iswhite(curc) as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -907 => {
                                    result =
                                        (curc != NUL && !ascii_iswhite(curc)) as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -906 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_DIGIT
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -905 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_DIGIT
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -904 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_HEX
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -903 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_HEX
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -902 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_OCTAL
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -901 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_OCTAL
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -900 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_WORD
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -899 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_WORD
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -898 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_HEAD
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -897 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_HEAD
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -896 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_ALPHA
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -895 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_ALPHA
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -894 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_LOWER
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -893 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_LOWER
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -892 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_UPPER
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -891 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_UPPER
                                                != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -890 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_LOWER
                                            != 0
                                        || (*rex.ptr()).reg_ic as ::core::ffi::c_int != 0
                                            && (curc < 0x100 as ::core::ffi::c_int
                                                && (*class_tab.ptr())[curc as usize]
                                                    as ::core::ffi::c_int
                                                    & RI_UPPER
                                                    != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -889 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_LOWER
                                                != 0
                                            || (*rex.ptr()).reg_ic as ::core::ffi::c_int != 0
                                                && (curc < 0x100 as ::core::ffi::c_int
                                                    && (*class_tab.ptr())[curc as usize]
                                                        as ::core::ffi::c_int
                                                        & RI_UPPER
                                                        != 0)))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -888 => {
                                    result = (curc < 0x100 as ::core::ffi::c_int
                                        && (*class_tab.ptr())[curc as usize] as ::core::ffi::c_int
                                            & RI_UPPER
                                            != 0
                                        || (*rex.ptr()).reg_ic as ::core::ffi::c_int != 0
                                            && (curc < 0x100 as ::core::ffi::c_int
                                                && (*class_tab.ptr())[curc as usize]
                                                    as ::core::ffi::c_int
                                                    & RI_LOWER
                                                    != 0))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -887 => {
                                    result = (curc != NUL
                                        && !(curc < 0x100 as ::core::ffi::c_int
                                            && (*class_tab.ptr())[curc as usize]
                                                as ::core::ffi::c_int
                                                & RI_UPPER
                                                != 0
                                            || (*rex.ptr()).reg_ic as ::core::ffi::c_int != 0
                                                && (curc < 0x100 as ::core::ffi::c_int
                                                    && (*class_tab.ptr())[curc as usize]
                                                        as ::core::ffi::c_int
                                                        & RI_LOWER
                                                        != 0)))
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                                -976 | -975 | -974 | -973 | -972 | -971 | -970 | -969 | -968
                                | -967 | -966 | -965 | -964 | -963 | -962 | -961 | -960 | -959 => {
                                    let mut subidx: ::core::ffi::c_int = 0;
                                    let mut bytelen_0: ::core::ffi::c_int = 0;
                                    if (*(*t).state).c >= NFA_BACKREF1 as ::core::ffi::c_int
                                        && (*(*t).state).c <= NFA_BACKREF9 as ::core::ffi::c_int
                                    {
                                        subidx = (*(*t).state).c
                                            - NFA_BACKREF1 as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int;
                                        result = match_backref(
                                            &raw mut (*t).subs.norm,
                                            subidx,
                                            &raw mut bytelen_0,
                                        );
                                    } else {
                                        subidx = (*(*t).state).c - NFA_ZREF1 as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int;
                                        result = match_zref(subidx, &raw mut bytelen_0);
                                    }
                                    if result != 0 {
                                        if bytelen_0 == 0 as ::core::ffi::c_int {
                                            add_here = true_0 != 0;
                                            add_state = (*(*(*t).state).out).out;
                                        } else if bytelen_0 <= clen {
                                            add_state = (*(*(*t).state).out).out;
                                            add_off = clen;
                                        } else {
                                            add_state = (*(*t).state).out;
                                            add_off = bytelen_0;
                                            add_count = bytelen_0 - clen;
                                        }
                                    }
                                }
                                -958 => {
                                    if (*t).count - clen <= 0 as ::core::ffi::c_int {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    } else {
                                        add_state = (*t).state;
                                        add_off = 0 as ::core::ffi::c_int;
                                        add_count = (*t).count - clen;
                                    }
                                }
                                -854 | -853 | -852 => {
                                    '_c2rust_label: {
                                        if (*(*t).state).val >= 0 as ::core::ffi::c_int
                                            && !((*rex.ptr()).reg_firstlnum > 0 as linenr_T
                                                && (*rex.ptr()).lnum as ::core::ffi::c_long
                                                    > 9223372036854775807 as ::core::ffi::c_long
                                                        - (*rex.ptr()).reg_firstlnum
                                                            as ::core::ffi::c_long
                                                || (*rex.ptr()).reg_firstlnum < 0 as linenr_T
                                                    && ((*rex.ptr()).lnum as ::core::ffi::c_long)
                                                        < -9223372036854775807
                                                            as ::core::ffi::c_long
                                                            - 1 as ::core::ffi::c_long
                                                            + (*rex.ptr()).reg_firstlnum
                                                                as ::core::ffi::c_long)
                                            && (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                                >= 0 as linenr_T
                                        {
                                        } else {
                                            __assert_fail(
                                                b"t->state->val >= 0 && !((rex.reg_firstlnum > 0 && rex.lnum > LONG_MAX - rex.reg_firstlnum) || (rex.reg_firstlnum < 0 && rex.lnum < LONG_MIN + rex.reg_firstlnum)) && rex.lnum + rex.reg_firstlnum >= 0\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/regexp.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                15131 as ::core::ffi::c_uint,
                                                b"int nfa_regmatch(nfa_regprog_T *, nfa_state_T *, regsubs_T *, regsubs_T *)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    result = ((*rex.ptr()).reg_match.is_null()
                                        && nfa_re_num_cmp(
                                            (*(*t).state).val as uintmax_t,
                                            (*(*t).state).c - NFA_LNUM as ::core::ffi::c_int,
                                            ((*rex.ptr()).lnum as uintmax_t).wrapping_add(
                                                (*rex.ptr()).reg_firstlnum as uintmax_t,
                                            ),
                                        )
                                            as ::core::ffi::c_int
                                            != 0)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -851 | -850 | -849 => {
                                    '_c2rust_label_0: {
                                        if (*(*t).state).val >= 0 as ::core::ffi::c_int
                                            && (*rex.ptr()).input >= (*rex.ptr()).line
                                            && (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                as uintmax_t
                                                <= (18446744073709551615 as uintmax_t)
                                                    .wrapping_sub(1 as uintmax_t)
                                        {
                                        } else {
                                            __assert_fail(
                                                b"t->state->val >= 0 && rex.input >= rex.line && (uintmax_t)(rex.input - rex.line) <= UINTMAX_MAX - 1\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/regexp.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                15147 as ::core::ffi::c_uint,
                                                b"int nfa_regmatch(nfa_regprog_T *, nfa_state_T *, regsubs_T *, regsubs_T *)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    result = nfa_re_num_cmp(
                                        (*(*t).state).val as uintmax_t,
                                        (*(*t).state).c - NFA_COL as ::core::ffi::c_int,
                                        ((*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                            + 1 as isize)
                                            as uintmax_t,
                                    )
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -848 | -847 | -846 => {
                                    let mut op: ::core::ffi::c_int =
                                        (*(*t).state).c - NFA_VCOL as ::core::ffi::c_int;
                                    let mut col: colnr_T =
                                        (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                            as colnr_T;
                                    if !(op != 1 as ::core::ffi::c_int
                                        && col
                                            > (*(*t).state).val * MB_MAXBYTES as ::core::ffi::c_int)
                                    {
                                        result = false_0;
                                        let mut wp: *mut win_T = if (*rex.ptr()).reg_win.is_null() {
                                            curwin.get()
                                        } else {
                                            (*rex.ptr()).reg_win
                                        };
                                        if op == 1 as ::core::ffi::c_int
                                            && col as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                                > (*(*t).state).val
                                            && col > 100 as ::core::ffi::c_int
                                        {
                                            let mut ts: int64_t = (*(*wp).w_buffer).b_p_ts;
                                            if ts < 4 as int64_t {
                                                ts = 4 as int64_t;
                                            }
                                            result = (col as int64_t
                                                > (*(*t).state).val as int64_t * ts)
                                                as ::core::ffi::c_int;
                                        }
                                        if result == 0 {
                                            let mut lnum: linenr_T =
                                                if (*rex.ptr()).reg_match.is_null() {
                                                    (*rex.ptr()).reg_firstlnum + (*rex.ptr()).lnum
                                                } else {
                                                    1 as linenr_T
                                                };
                                            if (*rex.ptr()).reg_match.is_null()
                                                && (lnum <= 0 as linenr_T
                                                    || lnum > (*(*wp).w_buffer).b_ml.ml_line_count)
                                            {
                                                lnum = 1 as ::core::ffi::c_int as linenr_T;
                                            }
                                            let mut vcol: ::core::ffi::c_int = win_linetabsize(
                                                wp,
                                                lnum,
                                                (*rex.ptr()).line as *mut ::core::ffi::c_char,
                                                col,
                                            );
                                            '_c2rust_label_1: {
                                                if (*(*t).state).val >= 0 as ::core::ffi::c_int {
                                                } else {
                                                    __assert_fail(
                                                        b"t->state->val >= 0\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/regexp.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        15187 as ::core::ffi::c_uint,
                                                        b"int nfa_regmatch(nfa_regprog_T *, nfa_state_T *, regsubs_T *, regsubs_T *)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            result = nfa_re_num_cmp(
                                                (*(*t).state).val as uintmax_t,
                                                op,
                                                (vcol as uintmax_t).wrapping_add(1 as uintmax_t),
                                            )
                                                as ::core::ffi::c_int;
                                        }
                                        if result != 0 {
                                            add_here = true_0 != 0;
                                            add_state = (*(*t).state).out;
                                        }
                                    }
                                }
                                -845 | -844 | -843 => {
                                    let mut col_0: size_t = if (*rex.ptr()).reg_match.is_null() {
                                        (*rex.ptr()).input.offset_from((*rex.ptr()).line) as size_t
                                    } else {
                                        0 as size_t
                                    };
                                    let mut fm: *mut fmark_T = mark_get(
                                        (*rex.ptr()).reg_buf,
                                        curwin.get(),
                                        ::core::ptr::null_mut::<fmark_T>(),
                                        kMarkBufLocal,
                                        (*(*t).state).val,
                                    );
                                    if (*rex.ptr()).reg_match.is_null() {
                                        (*rex.ptr()).line =
                                            reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
                                        (*rex.ptr()).input =
                                            (*rex.ptr()).line.offset(col_0 as isize);
                                    }
                                    if !fm.is_null() && (*fm).mark.lnum > 0 as linenr_T {
                                        let mut pos: *mut pos_T = &raw mut (*fm).mark;
                                        let pos_col: colnr_T = if (*pos).lnum
                                            == (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                            && (*pos).col == MAXCOL as ::core::ffi::c_int
                                        {
                                            reg_getline_len(
                                                (*pos).lnum - (*rex.ptr()).reg_firstlnum,
                                            )
                                        } else {
                                            (*pos).col
                                        };
                                        result = if (*pos).lnum
                                            == (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                        {
                                            if pos_col
                                                == (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                    as colnr_T
                                            {
                                                ((*(*t).state).c == NFA_MARK as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            } else if pos_col
                                                < (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                    as colnr_T
                                            {
                                                ((*(*t).state).c
                                                    == NFA_MARK_GT as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            } else {
                                                ((*(*t).state).c
                                                    == NFA_MARK_LT as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            }
                                        } else if (*pos).lnum
                                            < (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                        {
                                            ((*(*t).state).c == NFA_MARK_GT as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        } else {
                                            ((*(*t).state).c == NFA_MARK_LT as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        };
                                        if result != 0 {
                                            add_here = true_0 != 0;
                                            add_state = (*(*t).state).out;
                                        }
                                    }
                                }
                                -855 => {
                                    result = (!(*rex.ptr()).reg_win.is_null()
                                        && (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum
                                            == (*(*rex.ptr()).reg_win).w_cursor.lnum
                                        && (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                            as colnr_T
                                            == (*(*rex.ptr()).reg_win).w_cursor.col)
                                        as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -842 => {
                                    result = reg_match_visual() as ::core::ffi::c_int;
                                    if result != 0 {
                                        add_here = true_0 != 0;
                                        add_state = (*(*t).state).out;
                                    }
                                }
                                -956 | -955 | -954 | -953 | -952 | -951 | -950 | -949 | -948
                                | -937 | -936 | -935 | -934 | -933 | -932 | -931 | -930 | -929
                                | -928 | -999 | -1001 => {}
                                _ => {
                                    let mut c: ::core::ffi::c_int = (*(*t).state).c;
                                    result = (c == curc) as ::core::ffi::c_int;
                                    if result == 0 && (*rex.ptr()).reg_ic as ::core::ffi::c_int != 0
                                    {
                                        result =
                                            (utf_fold(c) == utf_fold(curc)) as ::core::ffi::c_int;
                                    }
                                    if result != 0 && !(*rex.ptr()).reg_icombine {
                                        clen = utf_ptr2len(
                                            (*rex.ptr()).input as *mut ::core::ffi::c_char,
                                        );
                                    }
                                    if result != 0 {
                                        add_state = (*(*t).state).out;
                                        add_off = clen;
                                    }
                                }
                            }
                            's_217: {
                                if !add_state.is_null() {
                                    let mut pim_0: *mut nfa_pim_T =
                                        ::core::ptr::null_mut::<nfa_pim_T>();
                                    let mut pim_copy: nfa_pim_T = nfa_pim_T {
                                        result: 0,
                                        state: ::core::ptr::null_mut::<nfa_state_T>(),
                                        subs: regsubs_T {
                                            norm: regsub_T {
                                                in_use: 0,
                                                list: C2Rust_Unnamed_19 {
                                                    multi: [multipos {
                                                        start_lnum: 0,
                                                        end_lnum: 0,
                                                        start_col: 0,
                                                        end_col: 0,
                                                    };
                                                        10],
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
                                                    };
                                                        10],
                                                },
                                                orig_start_col: 0,
                                            },
                                        },
                                        end: C2Rust_Unnamed_20 {
                                            pos: lpos_T { lnum: 0, col: 0 },
                                        },
                                    };
                                    if (*t).pim.result == NFA_PIM_UNUSED {
                                        pim_0 = ::core::ptr::null_mut::<nfa_pim_T>();
                                    } else {
                                        pim_0 = &raw mut (*t).pim;
                                    }
                                    if !pim_0.is_null()
                                        && (clen == 0 as ::core::ffi::c_int
                                            || match_follows(add_state, 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                                != 0)
                                    {
                                        if (*pim_0).result == NFA_PIM_TODO {
                                            result = recursive_regmatch(
                                                (*pim_0).state,
                                                pim_0,
                                                prog,
                                                submatch,
                                                m,
                                                &raw mut listids,
                                                &raw mut listids_len,
                                            );
                                            (*pim_0).result = if result != 0 {
                                                NFA_PIM_MATCH
                                            } else {
                                                NFA_PIM_NOMATCH
                                            };
                                            if result
                                                != ((*(*pim_0).state).c
                                                    == NFA_START_INVISIBLE_NEG
                                                        as ::core::ffi::c_int
                                                    || (*(*pim_0).state).c
                                                        == NFA_START_INVISIBLE_NEG_FIRST
                                                            as ::core::ffi::c_int
                                                    || (*(*pim_0).state).c
                                                        == NFA_START_INVISIBLE_BEFORE_NEG
                                                            as ::core::ffi::c_int
                                                    || (*(*pim_0).state).c
                                                        == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
                                                            as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            {
                                                copy_sub_off(
                                                    &raw mut (*pim_0).subs.norm,
                                                    &raw mut (*m).norm,
                                                );
                                                if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                                    copy_sub_off(
                                                        &raw mut (*pim_0).subs.synt,
                                                        &raw mut (*m).synt,
                                                    );
                                                }
                                            }
                                        } else {
                                            result = ((*pim_0).result == NFA_PIM_MATCH)
                                                as ::core::ffi::c_int;
                                        }
                                        if result
                                            != ((*(*pim_0).state).c
                                                == NFA_START_INVISIBLE_NEG as ::core::ffi::c_int
                                                || (*(*pim_0).state).c
                                                    == NFA_START_INVISIBLE_NEG_FIRST
                                                        as ::core::ffi::c_int
                                                || (*(*pim_0).state).c
                                                    == NFA_START_INVISIBLE_BEFORE_NEG
                                                        as ::core::ffi::c_int
                                                || (*(*pim_0).state).c
                                                    == NFA_START_INVISIBLE_BEFORE_NEG_FIRST
                                                        as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        {
                                            copy_sub_off(
                                                &raw mut (*t).subs.norm,
                                                &raw mut (*pim_0).subs.norm,
                                            );
                                            if (*rex.ptr()).nfa_has_zsubexpr != 0 {
                                                copy_sub_off(
                                                    &raw mut (*t).subs.synt,
                                                    &raw mut (*pim_0).subs.synt,
                                                );
                                            }
                                            pim_0 = ::core::ptr::null_mut::<nfa_pim_T>();
                                        } else {
                                            break 's_217;
                                        }
                                    }
                                    if pim_0 == &raw mut (*t).pim {
                                        copy_pim(&raw mut pim_copy, pim_0);
                                        pim_0 = &raw mut pim_copy;
                                    }
                                    if add_here {
                                        r = addstate_here(
                                            thislist,
                                            add_state,
                                            &raw mut (*t).subs,
                                            pim_0,
                                            &raw mut listidx,
                                        );
                                    } else {
                                        r = addstate(
                                            nextlist,
                                            add_state,
                                            &raw mut (*t).subs,
                                            pim_0,
                                            add_off,
                                        );
                                        if add_count > 0 as ::core::ffi::c_int {
                                            (*(*nextlist).t.offset(
                                                ((*nextlist).n - 1 as ::core::ffi::c_int) as isize,
                                            ))
                                            .count = add_count;
                                        }
                                    }
                                    if r.is_null() {
                                        nfa_match.set(NFA_TOO_EXPENSIVE as ::core::ffi::c_int);
                                        break '_theend;
                                    }
                                }
                            }
                            listidx += 1;
                        }
                        if nfa_match.get() == 0
                            && (toplevel != 0
                                && (*rex.ptr()).lnum == 0 as linenr_T
                                && clen != 0 as ::core::ffi::c_int
                                && ((*rex.ptr()).reg_maxcol == 0 as ::core::ffi::c_int
                                    || ((*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                        as colnr_T)
                                        < (*rex.ptr()).reg_maxcol)
                                || !(*nfa_endp.ptr()).is_null()
                                    && (if (*rex.ptr()).reg_match.is_null() {
                                        ((*rex.ptr()).lnum < (*nfa_endp.get()).se_u.pos.lnum
                                            || (*rex.ptr()).lnum == (*nfa_endp.get()).se_u.pos.lnum
                                                && ((*rex.ptr())
                                                    .input
                                                    .offset_from((*rex.ptr()).line)
                                                    as ::core::ffi::c_int)
                                                    < (*nfa_endp.get()).se_u.pos.col)
                                            as ::core::ffi::c_int
                                    } else {
                                        ((*rex.ptr()).input < (*nfa_endp.get()).se_u.ptr)
                                            as ::core::ffi::c_int
                                    }) != 0)
                        {
                            if toplevel != 0 {
                                let mut add: ::core::ffi::c_int = true_0;
                                if (*prog).regstart != NUL && clen != 0 as ::core::ffi::c_int {
                                    if (*nextlist).n == 0 as ::core::ffi::c_int {
                                        let mut col_1: colnr_T =
                                            (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                as colnr_T
                                                + clen as colnr_T;
                                        if skip_to_start((*prog).regstart, &raw mut col_1) == FAIL {
                                            break '_theend;
                                        }
                                        (*rex.ptr()).input = (*rex.ptr())
                                            .line
                                            .offset(col_1 as isize)
                                            .offset(-(clen as isize));
                                    } else {
                                        let c_0: ::core::ffi::c_int = utf_ptr2char(
                                            ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                                                .offset(clen as isize),
                                        );
                                        if c_0 != (*prog).regstart
                                            && (!(*rex.ptr()).reg_ic
                                                || utf_fold(c_0) != utf_fold((*prog).regstart))
                                        {
                                            add = false_0;
                                        }
                                    }
                                }
                                if add != 0 {
                                    if (*rex.ptr()).reg_match.is_null() {
                                        (*m).norm.list.multi[0 as ::core::ffi::c_int as usize]
                                            .start_col =
                                            ((*rex.ptr()).input.offset_from((*rex.ptr()).line)
                                                as ::core::ffi::c_int
                                                + clen)
                                                as colnr_T;
                                        (*m).norm.orig_start_col = (*m).norm.list.multi
                                            [0 as ::core::ffi::c_int as usize]
                                            .start_col;
                                    } else {
                                        (*m).norm.list.line[0 as ::core::ffi::c_int as usize]
                                            .start = (*rex.ptr()).input.offset(clen as isize);
                                    }
                                    if addstate(
                                        nextlist,
                                        (*start).out,
                                        m,
                                        ::core::ptr::null_mut::<nfa_pim_T>(),
                                        clen,
                                    )
                                    .is_null()
                                    {
                                        nfa_match.set(NFA_TOO_EXPENSIVE as ::core::ffi::c_int);
                                        break '_theend;
                                    }
                                }
                            } else if addstate(
                                nextlist,
                                start,
                                m,
                                ::core::ptr::null_mut::<nfa_pim_T>(),
                                clen,
                            )
                            .is_null()
                            {
                                nfa_match.set(NFA_TOO_EXPENSIVE as ::core::ffi::c_int);
                                break '_theend;
                            }
                        }
                    }
                    if clen != 0 as ::core::ffi::c_int {
                        (*rex.ptr()).input = (*rex.ptr()).input.offset(clen as isize);
                    } else {
                        if !(go_to_nextline as ::core::ffi::c_int != 0
                            || !(*nfa_endp.ptr()).is_null()
                                && (*rex.ptr()).reg_match.is_null()
                                && (*rex.ptr()).lnum < (*nfa_endp.get()).se_u.pos.lnum)
                        {
                            break '_theend;
                        }
                        reg_nextline();
                    }
                    reg_breakcheck();
                    if got_int.get() {
                        break '_theend;
                    }
                    if !(!(*nfa_time_limit.ptr()).is_null() && {
                        (*nfa_time_count.ptr()) += 1;
                        nfa_time_count.get() == 20 as ::core::ffi::c_int
                    }) {
                        continue;
                    }
                    nfa_time_count.set(0 as ::core::ffi::c_int);
                    if nfa_did_time_out() != 0 {
                        break '_theend;
                    }
                }
            }
        }
    }
    xfree(list[0 as ::core::ffi::c_int as usize].t as *mut ::core::ffi::c_void);
    xfree(list[1 as ::core::ffi::c_int as usize].t as *mut ::core::ffi::c_void);
    xfree(listids as *mut ::core::ffi::c_void);
    return nfa_match.get();
}
