//! What the match loop calls out to: the character classes, the back
//! references, the recursive sub-match and the shortcuts around them.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn check_char_class(
    mut cls: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    match cls {
        -841 => {
            if c >= 1 as ::core::ffi::c_int
                && c < 128 as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                return OK;
            }
        }
        -840 => {
            if c >= 1 as ::core::ffi::c_int
                && c < 128 as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                    & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                return OK;
            }
        }
        -839 => {
            if c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int {
                return OK;
            }
        }
        -838 => {
            if c >= 1 as ::core::ffi::c_int
                && c <= 127 as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                    & _IScntrl as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                return OK;
            }
        }
        -837 => {
            if ascii_isdigit(c) {
                return OK;
            }
        }
        -836 => {
            if c >= 1 as ::core::ffi::c_int
                && c <= 127 as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                    & _ISgraph as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                return OK;
            }
        }
        -835 => {
            if mb_islower(c) as ::core::ffi::c_int != 0
                && c != 170 as ::core::ffi::c_int
                && c != 186 as ::core::ffi::c_int
            {
                return OK;
            }
        }
        -834 => {
            if vim_isprintc(c) {
                return OK;
            }
        }
        -833 => {
            if c >= 1 as ::core::ffi::c_int
                && c < 128 as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                    & _ISpunct as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                return OK;
            }
        }
        -832 => {
            if c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
                || c == ' ' as ::core::ffi::c_int
            {
                return OK;
            }
        }
        -831 => {
            if mb_isupper(c) {
                return OK;
            }
        }
        -830 => {
            if ascii_isxdigit(c) {
                return OK;
            }
        }
        -829 => {
            if c == '\t' as ::core::ffi::c_int {
                return OK;
            }
        }
        -828 => {
            if c == '\r' as ::core::ffi::c_int {
                return OK;
            }
        }
        -827 => {
            if c == '\u{8}' as ::core::ffi::c_int {
                return OK;
            }
        }
        -826 => {
            if c == ESC {
                return OK;
            }
        }
        -825 => {
            if vim_isIDc(c) {
                return OK;
            }
        }
        -824 => {
            if reg_iswordc(c) {
                return OK;
            }
        }
        -823 => {
            if vim_isfilec(c) {
                return OK;
            }
        }
        _ => {
            siemsg(
                gettext((e_ill_char_class.ptr() as *const _) as *const ::core::ffi::c_char),
                cls as int64_t,
            );
            return FAIL;
        }
    }
    return FAIL;
}
pub(crate) unsafe extern "C" fn match_backref(
    mut sub: *mut regsub_T,
    mut subidx: ::core::ffi::c_int,
    mut bytelen: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0;
    '_retempty: {
        if (*sub).in_use > subidx {
            if (*rex.ptr()).reg_match.is_null() {
                if (*sub).list.multi[subidx as usize].start_lnum < 0 as linenr_T
                    || (*sub).list.multi[subidx as usize].end_lnum < 0 as linenr_T
                {
                    break '_retempty;
                } else if (*sub).list.multi[subidx as usize].start_lnum == (*rex.ptr()).lnum
                    && (*sub).list.multi[subidx as usize].end_lnum == (*rex.ptr()).lnum
                {
                    len = ((*sub).list.multi[subidx as usize].end_col
                        - (*sub).list.multi[subidx as usize].start_col)
                        as ::core::ffi::c_int;
                    if cstrncmp(
                        ((*rex.ptr()).line as *mut ::core::ffi::c_char)
                            .offset((*sub).list.multi[subidx as usize].start_col as isize),
                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                        &raw mut len,
                    ) == 0 as ::core::ffi::c_int
                    {
                        *bytelen = len;
                        return true_0;
                    }
                } else if match_with_backref(
                    (*sub).list.multi[subidx as usize].start_lnum,
                    (*sub).list.multi[subidx as usize].start_col,
                    (*sub).list.multi[subidx as usize].end_lnum,
                    (*sub).list.multi[subidx as usize].end_col,
                    bytelen,
                ) == RA_MATCH
                {
                    return true_0;
                }
            } else if (*sub).list.line[subidx as usize].start.is_null()
                || (*sub).list.line[subidx as usize].end.is_null()
            {
                break '_retempty;
            } else {
                len = (*sub).list.line[subidx as usize]
                    .end
                    .offset_from((*sub).list.line[subidx as usize].start)
                    as ::core::ffi::c_int;
                if cstrncmp(
                    (*sub).list.line[subidx as usize].start as *mut ::core::ffi::c_char,
                    (*rex.ptr()).input as *mut ::core::ffi::c_char,
                    &raw mut len,
                ) == 0 as ::core::ffi::c_int
                {
                    *bytelen = len;
                    return true_0;
                }
            }
            return false_0;
        }
    }
    *bytelen = 0 as ::core::ffi::c_int;
    return true_0;
}
pub(crate) unsafe extern "C" fn match_zref(
    mut subidx: ::core::ffi::c_int,
    mut bytelen: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0;
    cleanup_zsubexpr();
    if (*re_extmatch_in.ptr()).is_null()
        || (*re_extmatch_in.get()).matches[subidx as usize].is_null()
    {
        *bytelen = 0 as ::core::ffi::c_int;
        return true_0;
    }
    len = strlen((*re_extmatch_in.get()).matches[subidx as usize] as *mut ::core::ffi::c_char)
        as ::core::ffi::c_int;
    if cstrncmp(
        (*re_extmatch_in.get()).matches[subidx as usize] as *mut ::core::ffi::c_char,
        (*rex.ptr()).input as *mut ::core::ffi::c_char,
        &raw mut len,
    ) == 0 as ::core::ffi::c_int
    {
        *bytelen = len;
        return true_0;
    }
    return false_0;
}
pub(crate) unsafe extern "C" fn nfa_save_listids(
    mut prog: *mut nfa_regprog_T,
    mut list: *mut ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0;
    let mut p: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    p = (&raw mut (*prog).state as *mut nfa_state_T).offset(0 as ::core::ffi::c_int as isize);
    i = (*prog).nstate;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        *list.offset(i as isize) = (*p).lastlist[1 as ::core::ffi::c_int as usize];
        (*p).lastlist[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
        p = p.offset(1);
    }
}
pub(crate) unsafe extern "C" fn nfa_restore_listids(
    mut prog: *mut nfa_regprog_T,
    mut list: *const ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0;
    let mut p: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    p = (&raw mut (*prog).state as *mut nfa_state_T).offset(0 as ::core::ffi::c_int as isize);
    i = (*prog).nstate;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        (*p).lastlist[1 as ::core::ffi::c_int as usize] = *list.offset(i as isize);
        p = p.offset(1);
    }
}
pub(crate) unsafe extern "C" fn nfa_re_num_cmp(
    mut val: uintmax_t,
    mut op: ::core::ffi::c_int,
    mut pos: uintmax_t,
) -> bool {
    if op == 1 as ::core::ffi::c_int {
        return pos > val;
    }
    if op == 2 as ::core::ffi::c_int {
        return pos < val;
    }
    return val == pos;
}
pub(crate) unsafe extern "C" fn recursive_regmatch(
    mut state: *mut nfa_state_T,
    mut pim: *mut nfa_pim_T,
    mut prog: *mut nfa_regprog_T,
    mut submatch: *mut regsubs_T,
    mut m: *mut regsubs_T,
    mut listids: *mut *mut ::core::ffi::c_int,
    mut listids_len: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let save_reginput_col: ::core::ffi::c_int =
        (*rex.ptr()).input.offset_from((*rex.ptr()).line) as ::core::ffi::c_int;
    let save_reglnum: ::core::ffi::c_int = (*rex.ptr()).lnum as ::core::ffi::c_int;
    let save_nfa_match: ::core::ffi::c_int = nfa_match.get();
    let save_nfa_listid: ::core::ffi::c_int = (*rex.ptr()).nfa_listid;
    let save_nfa_endp: *mut save_se_T = nfa_endp.get();
    let mut endpos: save_se_T = save_se_T {
        se_u: C2Rust_Unnamed_21 {
            ptr: ::core::ptr::null_mut::<uint8_t>(),
        },
    };
    let mut endposp: *mut save_se_T = ::core::ptr::null_mut::<save_se_T>();
    let mut need_restore: ::core::ffi::c_int = false_0;
    if !pim.is_null() {
        if (*rex.ptr()).reg_match.is_null() {
            (*rex.ptr()).input = (*rex.ptr()).line.offset((*pim).end.pos.col as isize);
        } else {
            (*rex.ptr()).input = (*pim).end.ptr;
        }
    }
    if (*state).c == NFA_START_INVISIBLE_BEFORE as ::core::ffi::c_int
        || (*state).c == NFA_START_INVISIBLE_BEFORE_FIRST as ::core::ffi::c_int
        || (*state).c == NFA_START_INVISIBLE_BEFORE_NEG as ::core::ffi::c_int
        || (*state).c == NFA_START_INVISIBLE_BEFORE_NEG_FIRST as ::core::ffi::c_int
    {
        endposp = &raw mut endpos;
        if (*rex.ptr()).reg_match.is_null() {
            if pim.is_null() {
                endpos.se_u.pos.col = (*rex.ptr()).input.offset_from((*rex.ptr()).line)
                    as ::core::ffi::c_int as colnr_T;
                endpos.se_u.pos.lnum = (*rex.ptr()).lnum;
            } else {
                endpos.se_u.pos = (*pim).end.pos;
            }
        } else if pim.is_null() {
            endpos.se_u.ptr = (*rex.ptr()).input;
        } else {
            endpos.se_u.ptr = (*pim).end.ptr;
        }
        if (*state).val <= 0 as ::core::ffi::c_int {
            if (*rex.ptr()).reg_match.is_null() {
                (*rex.ptr()).lnum -= 1;
                (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
                if (*rex.ptr()).line.is_null() {
                    (*rex.ptr()).lnum += 1;
                    (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
                }
            }
            (*rex.ptr()).input = (*rex.ptr()).line;
        } else {
            if (*rex.ptr()).reg_match.is_null()
                && ((*rex.ptr()).input.offset_from((*rex.ptr()).line) as ::core::ffi::c_int)
                    < (*state).val
            {
                (*rex.ptr()).lnum -= 1;
                (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
                if (*rex.ptr()).line.is_null() {
                    (*rex.ptr()).lnum += 1;
                    (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
                    (*rex.ptr()).input = (*rex.ptr()).line;
                } else {
                    (*rex.ptr()).input = (*rex.ptr())
                        .line
                        .offset(reg_getline_len((*rex.ptr()).lnum) as isize);
                }
            }
            if (*rex.ptr()).input.offset_from((*rex.ptr()).line) as ::core::ffi::c_int
                >= (*state).val
            {
                (*rex.ptr()).input = (*rex.ptr()).input.offset(-((*state).val as isize));
                (*rex.ptr()).input = (*rex.ptr()).input.offset(
                    -(utf_head_off(
                        (*rex.ptr()).line as *mut ::core::ffi::c_char,
                        (*rex.ptr()).input as *mut ::core::ffi::c_char,
                    ) as isize),
                );
            } else {
                (*rex.ptr()).input = (*rex.ptr()).line;
            }
        }
    }
    if nfa_ll_index.get() == 1 as ::core::ffi::c_int {
        if (*listids).is_null() || *listids_len < (*prog).nstate {
            xfree(*listids as *mut ::core::ffi::c_void);
            *listids = xmalloc(
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul((*prog).nstate as size_t),
            ) as *mut ::core::ffi::c_int;
            *listids_len = (*prog).nstate;
        }
        nfa_save_listids(prog, *listids);
        need_restore = true_0;
    } else {
        (*nfa_ll_index.ptr()) += 1;
        if (*rex.ptr()).nfa_listid <= (*rex.ptr()).nfa_alt_listid {
            (*rex.ptr()).nfa_listid = (*rex.ptr()).nfa_alt_listid;
        }
    }
    nfa_endp.set(endposp);
    let result: ::core::ffi::c_int = nfa_regmatch(prog, (*state).out, submatch, m);
    if need_restore != 0 {
        nfa_restore_listids(prog, *listids);
    } else {
        (*nfa_ll_index.ptr()) -= 1;
        (*rex.ptr()).nfa_alt_listid = (*rex.ptr()).nfa_listid;
    }
    (*rex.ptr()).lnum = save_reglnum as linenr_T;
    if (*rex.ptr()).reg_match.is_null() {
        (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
    }
    (*rex.ptr()).input = (*rex.ptr()).line.offset(save_reginput_col as isize);
    if result != NFA_TOO_EXPENSIVE as ::core::ffi::c_int {
        nfa_match.set(save_nfa_match);
        (*rex.ptr()).nfa_listid = save_nfa_listid;
    }
    nfa_endp.set(save_nfa_endp);
    return result;
}
pub(crate) unsafe extern "C" fn failure_chance(
    mut state: *mut nfa_state_T,
    mut depth: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = (*state).c;
    let mut l: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0;
    if depth > 4 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    match c {
        -1024 => {
            if (*(*state).out).c == NFA_SPLIT as ::core::ffi::c_int
                || (*(*state).out1).c == NFA_SPLIT as ::core::ffi::c_int
            {
                return 1 as ::core::ffi::c_int;
            }
            l = failure_chance((*state).out, depth + 1 as ::core::ffi::c_int);
            r = failure_chance((*state).out1, depth + 1 as ::core::ffi::c_int);
            return if l < r { l } else { r };
        }
        -917 => return 1 as ::core::ffi::c_int,
        -1023 | -947 | -983 => return 0 as ::core::ffi::c_int,
        -997 | -996 | -995 | -994 | -993 | -992 | -991 | -990 | -989 => {
            return 5 as ::core::ffi::c_int;
        }
        -1008 | -1007 | -1004 | -1003 | -1002 => return 99 as ::core::ffi::c_int,
        -1006 | -1005 => return 90 as ::core::ffi::c_int,
        -957 | -956 | -955 | -954 | -953 | -952 | -951 | -950 | -949 | -948 | -937 | -936
        | -935 | -934 | -933 | -932 | -931 | -930 | -929 | -928 | -927 | -926 | -925 | -924
        | -923 | -922 | -921 | -920 | -919 | -918 | -999 | -946 | -945 | -944 | -943 | -942
        | -941 | -940 | -939 | -938 | -998 => {
            return failure_chance((*state).out, depth + 1 as ::core::ffi::c_int);
        }
        -976 | -975 | -974 | -973 | -972 | -971 | -970 | -969 | -968 | -967 | -966 | -965
        | -964 | -963 | -962 | -961 | -960 | -959 => return 94 as ::core::ffi::c_int,
        -853 | -852 | -850 | -849 | -847 | -846 | -844 | -843 | -842 => {
            return 85 as ::core::ffi::c_int;
        }
        -854 => return 90 as ::core::ffi::c_int,
        -855 | -851 | -848 | -845 => return 98 as ::core::ffi::c_int,
        -985 => return 95 as ::core::ffi::c_int,
        _ => {
            if c > 0 as ::core::ffi::c_int {
                return 95 as ::core::ffi::c_int;
            }
        }
    }
    return 50 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn skip_to_start(
    mut c: ::core::ffi::c_int,
    mut colp: *mut colnr_T,
) -> ::core::ffi::c_int {
    let s: *const uint8_t = cstrchr(
        ((*rex.ptr()).line as *mut ::core::ffi::c_char).offset(*colp as isize),
        c,
    ) as *mut uint8_t;
    if s.is_null() {
        return FAIL;
    }
    *colp = s.offset_from((*rex.ptr()).line) as ::core::ffi::c_int as colnr_T;
    return OK;
}
pub(crate) unsafe extern "C" fn find_match_text(
    mut startcol: *mut colnr_T,
    mut regstart: ::core::ffi::c_int,
    mut match_text: *mut uint8_t,
) -> ::core::ffi::c_int {
    let mut col: colnr_T = *startcol;
    let regstart_len: ::core::ffi::c_int = utf_char2len(regstart);
    loop {
        let mut match_0: bool = true_0 != 0;
        let mut s1: *mut uint8_t = match_text;
        let mut regstart_len2: ::core::ffi::c_int = regstart_len;
        if regstart_len2 > 1 as ::core::ffi::c_int
            && utf_ptr2len(((*rex.ptr()).line as *mut ::core::ffi::c_char).offset(col as isize))
                != regstart_len2
        {
            regstart_len2 = utf_char2len(utf_fold(regstart));
        }
        let mut s2: *mut uint8_t = (*rex.ptr())
            .line
            .offset(col as isize)
            .offset(regstart_len2 as isize);
        while *s1 != 0 {
            let mut c1_len: ::core::ffi::c_int = utf_ptr2len(s1 as *mut ::core::ffi::c_char);
            let mut c1: ::core::ffi::c_int = utf_ptr2char(s1 as *mut ::core::ffi::c_char);
            let mut c2_len: ::core::ffi::c_int = utf_ptr2len(s2 as *mut ::core::ffi::c_char);
            let mut c2: ::core::ffi::c_int = utf_ptr2char(s2 as *mut ::core::ffi::c_char);
            if c1 != c2 && (!(*rex.ptr()).reg_ic || utf_fold(c1) != utf_fold(c2)) {
                match_0 = false_0 != 0;
                break;
            } else {
                s1 = s1.offset(c1_len as isize);
                s2 = s2.offset(c2_len as isize);
            }
        }
        if match_0 as ::core::ffi::c_int != 0
            && !utf_iscomposing_legacy(utf_ptr2char(s2 as *mut ::core::ffi::c_char))
        {
            cleanup_subexpr();
            if (*rex.ptr()).reg_match.is_null() {
                (*(*rex.ptr())
                    .reg_startpos
                    .offset(0 as ::core::ffi::c_int as isize))
                .lnum = (*rex.ptr()).lnum;
                (*(*rex.ptr())
                    .reg_startpos
                    .offset(0 as ::core::ffi::c_int as isize))
                .col = col;
                (*(*rex.ptr())
                    .reg_endpos
                    .offset(0 as ::core::ffi::c_int as isize))
                .lnum = (*rex.ptr()).lnum;
                (*(*rex.ptr())
                    .reg_endpos
                    .offset(0 as ::core::ffi::c_int as isize))
                .col = s2.offset_from((*rex.ptr()).line) as colnr_T;
            } else {
                *(*rex.ptr())
                    .reg_startp
                    .offset(0 as ::core::ffi::c_int as isize) =
                    (*rex.ptr()).line.offset(col as isize);
                *(*rex.ptr())
                    .reg_endp
                    .offset(0 as ::core::ffi::c_int as isize) = s2;
            }
            *startcol = col;
            return 1 as ::core::ffi::c_int;
        }
        col += regstart_len;
        if skip_to_start(regstart, &raw mut col) == FAIL {
            break;
        }
    }
    *startcol = col;
    return 0 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn nfa_did_time_out() -> ::core::ffi::c_int {
    if !(*nfa_time_limit.ptr()).is_null()
        && profile_passed_limit(*nfa_time_limit.get()) as ::core::ffi::c_int != 0
    {
        if !(*nfa_timed_out.ptr()).is_null() {
            *nfa_timed_out.get() = true_0;
        }
        return true_0;
    }
    return false_0;
}
