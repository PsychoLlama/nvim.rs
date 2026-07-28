//! Setting up a compile and reading back what the postfix form implies:
//! the anchor, the start character and any literal match text.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn nfa_regcomp_start(
    mut expr: *mut uint8_t,
    mut re_flags: ::core::ffi::c_int,
) {
    let mut postfix_size: size_t = 0;
    let mut nstate_max: size_t = 0;
    nstate.set(0 as ::core::ffi::c_int);
    istate.set(0 as ::core::ffi::c_int);
    nstate_max = strlen(expr as *mut ::core::ffi::c_char)
        .wrapping_add(1 as size_t)
        .wrapping_mul(25 as size_t);
    nstate_max = nstate_max.wrapping_add(1000 as size_t);
    postfix_size =
        ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(nstate_max as usize) as size_t;
    post_start.set(xmalloc(postfix_size) as *mut ::core::ffi::c_int);
    post_ptr.set(post_start.get());
    post_end.set((*post_start.ptr()).offset(nstate_max as isize));
    wants_nfa.set(false_0 != 0);
    (*rex.ptr()).nfa_has_zend = false_0;
    (*rex.ptr()).nfa_has_backref = false_0;
    regcomp_start(expr, re_flags);
}
pub(crate) unsafe extern "C" fn nfa_get_reganch(
    mut start: *mut nfa_state_T,
    mut depth: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *mut nfa_state_T = start;
    if depth > 4 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    while !p.is_null() {
        match (*p).c {
            -1008 | -1004 => return 1 as ::core::ffi::c_int,
            -1001 | -1000 | -855 | -842 | -957 | -956 | -955 | -954 | -953 | -952 | -951 | -950
            | -949 | -948 | -999 | -937 | -936 | -935 | -934 | -933 | -932 | -931 | -930 | -929
            | -928 => {
                p = (*p).out;
            }
            -1024 => {
                return (nfa_get_reganch((*p).out, depth + 1 as ::core::ffi::c_int) != 0
                    && nfa_get_reganch((*p).out1, depth + 1 as ::core::ffi::c_int) != 0)
                    as ::core::ffi::c_int;
            }
            _ => return 0 as ::core::ffi::c_int,
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn nfa_get_regstart(
    mut start: *mut nfa_state_T,
    mut depth: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *mut nfa_state_T = start;
    if depth > 4 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    while !p.is_null() {
        match (*p).c {
            -1008 | -1004 | -1006 | -1005 | -1001 | -1000 | -855 | -842 | -854 | -853 | -852
            | -851 | -850 | -849 | -848 | -847 | -846 | -845 | -844 | -843 | -957 | -956 | -955
            | -954 | -953 | -952 | -951 | -950 | -949 | -948 | -999 | -937 | -936 | -935 | -934
            | -933 | -932 | -931 | -930 | -929 | -928 => {
                p = (*p).out;
            }
            -1024 => {
                let mut c1: ::core::ffi::c_int =
                    nfa_get_regstart((*p).out, depth + 1 as ::core::ffi::c_int);
                let mut c2: ::core::ffi::c_int =
                    nfa_get_regstart((*p).out1, depth + 1 as ::core::ffi::c_int);
                if c1 == c2 {
                    return c1;
                }
                return 0 as ::core::ffi::c_int;
            }
            _ => {
                if (*p).c > 0 as ::core::ffi::c_int {
                    return (*p).c;
                }
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn nfa_get_match_text(mut start: *mut nfa_state_T) -> *mut uint8_t {
    let mut p: *mut nfa_state_T = start;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ret: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut s: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    if (*p).c != NFA_MOPEN as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<uint8_t>();
    }
    p = (*p).out;
    while (*p).c > 0 as ::core::ffi::c_int {
        len += utf_char2len((*p).c);
        p = (*p).out;
    }
    if (*p).c != NFA_MCLOSE as ::core::ffi::c_int
        || (*(*p).out).c != NFA_MATCH as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<uint8_t>();
    }
    ret = xmalloc(len as size_t) as *mut uint8_t;
    p = (*(*start).out).out;
    s = ret;
    while (*p).c > 0 as ::core::ffi::c_int {
        s = s.offset(utf_char2bytes((*p).c, s as *mut ::core::ffi::c_char) as isize);
        p = (*p).out;
    }
    *s = NUL as uint8_t;
    return ret;
}
pub(crate) unsafe extern "C" fn realloc_post_list() {
    let new_max: size_t = ((*post_end.ptr()).offset_from(post_start.get()) as size_t)
        .wrapping_mul(3 as size_t)
        .wrapping_div(2 as size_t);
    let mut new_start: *mut ::core::ffi::c_int = xrealloc(
        post_start.get() as *mut ::core::ffi::c_void,
        new_max.wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
    ) as *mut ::core::ffi::c_int;
    post_ptr.set(new_start.offset((*post_ptr.ptr()).offset_from(post_start.get()) as isize));
    post_end.set(new_start.offset(new_max as isize));
    post_start.set(new_start);
}
pub(crate) unsafe extern "C" fn nfa_recognize_char_class(
    mut start: *mut uint8_t,
    mut end: *const uint8_t,
    mut extra_newl: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut config: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut newl: bool = extra_newl == true_0;
    if *end as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
        return FAIL;
    }
    p = start;
    if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
        config |= CLASS_not;
        p = p.offset(1);
    }
    while p < end as *mut uint8_t {
        if p.offset(2 as ::core::ffi::c_int as isize) < end as *mut uint8_t
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '-' as ::core::ffi::c_int
        {
            match *p as ::core::ffi::c_int {
                48 => {
                    if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '9' as ::core::ffi::c_int
                    {
                        config |= CLASS_o9;
                    } else if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '7' as ::core::ffi::c_int
                    {
                        config |= CLASS_o7;
                    } else {
                        return FAIL;
                    }
                }
                97 => {
                    if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'z' as ::core::ffi::c_int
                    {
                        config |= CLASS_az;
                    } else if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'f' as ::core::ffi::c_int
                    {
                        config |= CLASS_af;
                    } else {
                        return FAIL;
                    }
                }
                65 => {
                    if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'Z' as ::core::ffi::c_int
                    {
                        config |= CLASS_AZ;
                    } else if *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'F' as ::core::ffi::c_int
                    {
                        config |= CLASS_AF;
                    } else {
                        return FAIL;
                    }
                }
                _ => return FAIL,
            }
            p = p.offset(3 as ::core::ffi::c_int as isize);
        } else if p.offset(1 as ::core::ffi::c_int as isize) < end as *mut uint8_t
            && *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
        {
            newl = true_0 != 0;
            p = p.offset(2 as ::core::ffi::c_int as isize);
        } else if *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
            config |= CLASS_underscore;
            p = p.offset(1);
        } else if *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
            newl = true_0 != 0;
            p = p.offset(1);
        } else {
            return FAIL;
        }
    }
    if p != end as *mut uint8_t {
        return FAIL;
    }
    if newl as ::core::ffi::c_int == true_0 {
        extra_newl = NFA_ADD_NL;
    }
    match config {
        CLASS_o9 => return extra_newl + NFA_DIGIT as ::core::ffi::c_int,
        130 => return extra_newl + NFA_NDIGIT as ::core::ffi::c_int,
        98 => return extra_newl + NFA_HEX as ::core::ffi::c_int,
        226 => return extra_newl + NFA_NHEX as ::core::ffi::c_int,
        CLASS_o7 => return extra_newl + NFA_OCTAL as ::core::ffi::c_int,
        132 => return extra_newl + NFA_NOCTAL as ::core::ffi::c_int,
        27 => return extra_newl + NFA_WORD as ::core::ffi::c_int,
        155 => return extra_newl + NFA_NWORD as ::core::ffi::c_int,
        25 => return extra_newl + NFA_HEAD as ::core::ffi::c_int,
        153 => return extra_newl + NFA_NHEAD as ::core::ffi::c_int,
        24 => return extra_newl + NFA_ALPHA as ::core::ffi::c_int,
        152 => return extra_newl + NFA_NALPHA as ::core::ffi::c_int,
        CLASS_az => return extra_newl + NFA_LOWER_IC as ::core::ffi::c_int,
        144 => return extra_newl + NFA_NLOWER_IC as ::core::ffi::c_int,
        CLASS_AZ => return extra_newl + NFA_UPPER_IC as ::core::ffi::c_int,
        136 => return extra_newl + NFA_NUPPER_IC as ::core::ffi::c_int,
        _ => {}
    }
    return FAIL;
}
