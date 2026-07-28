//! Postfix form to state machine: the state allocator, the fragment
//! stack `post2nfa` runs on and the width analysis over the result.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn alloc_state(
    mut c: ::core::ffi::c_int,
    mut out: *mut nfa_state_T,
    mut out1: *mut nfa_state_T,
) -> *mut nfa_state_T {
    let mut s: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    if istate.get() >= nstate.get() {
        return ::core::ptr::null_mut::<nfa_state_T>();
    }
    let c2rust_fresh15 = istate.get();
    istate.set(istate.get() + 1);
    s = (*state_ptr.ptr()).offset(c2rust_fresh15 as isize);
    (*s).c = c;
    (*s).out = out;
    (*s).out1 = out1;
    (*s).val = 0 as ::core::ffi::c_int;
    (*s).id = istate.get();
    (*s).lastlist[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    (*s).lastlist[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    return s;
}
pub(crate) unsafe extern "C" fn frag(mut start: *mut nfa_state_T, mut out: *mut Ptrlist) -> Frag_T {
    let mut n: Frag_T = Frag_T {
        start: ::core::ptr::null_mut::<nfa_state_T>(),
        out: ::core::ptr::null_mut::<Ptrlist>(),
    };
    n.start = start;
    n.out = out;
    return n;
}
pub(crate) unsafe extern "C" fn list1(mut outp: *mut *mut nfa_state_T) -> *mut Ptrlist {
    let mut l: *mut Ptrlist = ::core::ptr::null_mut::<Ptrlist>();
    l = outp as *mut Ptrlist;
    (*l).next = ::core::ptr::null_mut::<Ptrlist>();
    return l;
}
pub(crate) unsafe extern "C" fn patch(mut l: *mut Ptrlist, mut s: *mut nfa_state_T) {
    let mut next: *mut Ptrlist = ::core::ptr::null_mut::<Ptrlist>();
    while !l.is_null() {
        next = (*l).next;
        (*l).s = s;
        l = next;
    }
}
pub(crate) unsafe extern "C" fn append(mut l1: *mut Ptrlist, mut l2: *mut Ptrlist) -> *mut Ptrlist {
    let mut oldl1: *mut Ptrlist = ::core::ptr::null_mut::<Ptrlist>();
    oldl1 = l1;
    while !(*l1).next.is_null() {
        l1 = (*l1).next;
    }
    (*l1).next = l2;
    return oldl1;
}
pub(crate) unsafe extern "C" fn st_error(
    mut _postfix: *mut ::core::ffi::c_int,
    mut _end: *mut ::core::ffi::c_int,
    mut _p: *mut ::core::ffi::c_int,
) {
    emsg(gettext(
        b"E874: (NFA) Could not pop the stack!\0".as_ptr() as *const ::core::ffi::c_char
    ));
}
pub(crate) unsafe extern "C" fn st_push(
    mut s: Frag_T,
    mut p: *mut *mut Frag_T,
    mut stack_end: *mut Frag_T,
) {
    let mut stackp: *mut Frag_T = *p;
    if stackp >= stack_end {
        return;
    }
    *stackp = s;
    *p = (*p).offset(1 as ::core::ffi::c_int as isize);
}
pub(crate) unsafe extern "C" fn st_pop(mut p: *mut *mut Frag_T, mut stack: *mut Frag_T) -> Frag_T {
    let mut stackp: *mut Frag_T = ::core::ptr::null_mut::<Frag_T>();
    *p = (*p).offset(-(1 as ::core::ffi::c_int as isize));
    stackp = *p;
    if stackp < stack {
        return empty.get();
    }
    return **p;
}
pub(crate) unsafe extern "C" fn nfa_max_width(
    mut startstate: *mut nfa_state_T,
    mut depth: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut l: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0;
    let mut state: *mut nfa_state_T = startstate;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if depth > 4 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    while !state.is_null() {
        match (*state).c {
            -988 | -987 => return len,
            -1024 => {
                l = nfa_max_width((*state).out, depth + 1 as ::core::ffi::c_int);
                r = nfa_max_width((*state).out1, depth + 1 as ::core::ffi::c_int);
                if l < 0 as ::core::ffi::c_int || r < 0 as ::core::ffi::c_int {
                    return -1 as ::core::ffi::c_int;
                }
                return len + (if l > r { l } else { r });
            }
            -917 | -1021 | -1019 => {
                len += MB_MAXBYTES as ::core::ffi::c_int;
                if (*state).c != NFA_ANY as ::core::ffi::c_int {
                    if (*state).out1.is_null() || (*(*state).out1).out.is_null() {
                        return -1 as ::core::ffi::c_int;
                    }
                    state = (*(*state).out1).out;
                    continue;
                }
            }
            -906 | -908 | -904 | -902 => {
                len += 1;
            }
            -916 | -915 | -914 | -913 | -912 | -911 | -910 | -909 | -907 | -905 | -903 | -901
            | -900 | -899 | -898 | -897 | -896 | -895 | -894 | -893 | -892 | -891 | -890 | -889
            | -888 | -887 | -983 => {
                len += 3 as ::core::ffi::c_int;
            }
            -997 | -995 | -993 | -991 => {
                state = (*(*state).out1).out;
                continue;
            }
            -976 | -975 | -974 | -973 | -972 | -971 | -970 | -969 | -968 | -967 | -966 | -965
            | -964 | -963 | -962 | -961 | -960 | -959 | -1002 | -958 => {
                return -1 as ::core::ffi::c_int;
            }
            -1008 | -1007 | -1004 | -1003 | -1006 | -1005 | -957 | -956 | -955 | -954 | -953
            | -952 | -951 | -950 | -949 | -948 | -937 | -936 | -935 | -934 | -933 | -932 | -931
            | -930 | -929 | -928 | -927 | -926 | -925 | -924 | -923 | -922 | -921 | -920 | -919
            | -918 | -947 | -946 | -945 | -944 | -943 | -942 | -941 | -940 | -939 | -938 | -999
            | -998 | -853 | -852 | -850 | -849 | -847 | -846 | -844 | -843 | -842 | -854 | -855
            | -851 | -848 | -845 | -1001 | -1000 | -982 | -1022 | -989 | -986 | -985 | -984 => {}
            _ => {
                if (*state).c < 0 as ::core::ffi::c_int {
                    return -1 as ::core::ffi::c_int;
                }
                len += utf_char2len((*state).c);
            }
        }
        state = (*state).out;
    }
    return -1 as ::core::ffi::c_int;
}
pub(crate) unsafe fn post2nfa(
    items: &[::core::ffi::c_int],
    mut nfa_calc_size: ::core::ffi::c_int,
) -> *mut nfa_state_T {
    let postfix: *mut ::core::ffi::c_int = items.as_ptr().cast_mut();
    let end: *mut ::core::ffi::c_int = postfix.wrapping_add(items.len());
    let mut p: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut mopen: ::core::ffi::c_int = 0;
    let mut mclose: ::core::ffi::c_int = 0;
    let mut stack: *mut Frag_T = ::core::ptr::null_mut::<Frag_T>();
    let mut stackp: *mut Frag_T = ::core::ptr::null_mut::<Frag_T>();
    let mut stack_end: *mut Frag_T = ::core::ptr::null_mut::<Frag_T>();
    let mut e1: Frag_T = Frag_T {
        start: ::core::ptr::null_mut::<nfa_state_T>(),
        out: ::core::ptr::null_mut::<Ptrlist>(),
    };
    let mut e2: Frag_T = Frag_T {
        start: ::core::ptr::null_mut::<nfa_state_T>(),
        out: ::core::ptr::null_mut::<Ptrlist>(),
    };
    let mut e: Frag_T = Frag_T {
        start: ::core::ptr::null_mut::<nfa_state_T>(),
        out: ::core::ptr::null_mut::<Ptrlist>(),
    };
    let mut s: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    let mut s1: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    let mut matchstate: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    let mut ret: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
    if nfa_calc_size == false_0 {
        stack = xmalloc(
            ((nstate.get() + 1 as ::core::ffi::c_int) as size_t)
                .wrapping_mul(::core::mem::size_of::<Frag_T>()),
        ) as *mut Frag_T;
        stackp = stack;
        stack_end = stack.offset((nstate.get() + 1 as ::core::ffi::c_int) as isize);
    }
    p = postfix;
    '_theend: {
        while p < end {
            match *p {
                -1014 => {
                    if nfa_calc_size != true_0 {
                        e2 = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        e1 = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        patch(e1.out, e2.start);
                        st_push(frag(e1.start, e2.out), &raw mut stackp, stack_end);
                    }
                }
                -1013 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        e2 = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        e1 = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        s = alloc_state(NFA_SPLIT as ::core::ffi::c_int, e1.start, e2.start);
                        if s.is_null() {
                            break '_theend;
                        }
                        st_push(frag(s, append(e1.out, e2.out)), &raw mut stackp, stack_end);
                    }
                }
                -1012 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        e = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        s = alloc_state(
                            NFA_SPLIT as ::core::ffi::c_int,
                            e.start,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        patch(e.out, s);
                        st_push(
                            frag(s, list1(&raw mut (*s).out1)),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -1011 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        e = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        s = alloc_state(
                            NFA_SPLIT as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            e.start,
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        patch(e.out, s);
                        st_push(
                            frag(s, list1(&raw mut (*s).out)),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -1010 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        e = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        s = alloc_state(
                            NFA_SPLIT as ::core::ffi::c_int,
                            e.start,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        st_push(
                            frag(s, append(e.out, list1(&raw mut (*s).out1))),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -1009 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        e = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        s = alloc_state(
                            NFA_SPLIT as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            e.start,
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        st_push(
                            frag(s, append(e.out, list1(&raw mut (*s).out))),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -1020 | -1018 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        e = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        s = alloc_state(
                            NFA_END_COLL as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        patch(e.out, s);
                        (*e.start).out1 = s;
                        st_push(
                            frag(e.start, list1(&raw mut (*s).out)),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -1017 => {
                    if nfa_calc_size != true_0 {
                        e2 = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        e1 = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        (*e2.start).val = (*e2.start).c;
                        (*e2.start).c = NFA_RANGE_MAX as ::core::ffi::c_int;
                        (*e1.start).val = (*e1.start).c;
                        (*e1.start).c = NFA_RANGE_MIN as ::core::ffi::c_int;
                        patch(e1.out, e2.start);
                        st_push(frag(e1.start, e2.out), &raw mut stackp, stack_end);
                    }
                }
                -1022 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        s = alloc_state(
                            NFA_EMPTY as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        st_push(
                            frag(s, list1(&raw mut (*s).out)),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -982 => {
                    let mut n: ::core::ffi::c_int = 0;
                    p = p.offset(1);
                    n = *p;
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += n;
                    } else {
                        s = ::core::ptr::null_mut::<nfa_state_T>();
                        e1.out = ::core::ptr::null_mut::<Ptrlist>();
                        s1 = ::core::ptr::null_mut::<nfa_state_T>();
                        loop {
                            let c2rust_fresh13 = n;
                            n = n - 1;
                            if c2rust_fresh13 <= 0 as ::core::ffi::c_int {
                                break;
                            }
                            e = st_pop(&raw mut stackp, stack);
                            if stackp < stack {
                                st_error(postfix, end, p);
                                xfree(stack as *mut ::core::ffi::c_void);
                                return ::core::ptr::null_mut::<nfa_state_T>();
                            }
                            s = alloc_state(
                                NFA_SPLIT as ::core::ffi::c_int,
                                e.start,
                                ::core::ptr::null_mut::<nfa_state_T>(),
                            );
                            if s.is_null() {
                                break '_theend;
                            }
                            if e1.out.is_null() {
                                e1 = e;
                            }
                            patch(e.out, s1);
                            append(e1.out, list1(&raw mut (*s).out1));
                            s1 = s;
                        }
                        st_push(frag(s, e1.out), &raw mut stackp, stack_end);
                    }
                }
                -981 | -980 | -979 | -978 | -977 => {
                    let mut before: ::core::ffi::c_int = (*p
                        == NFA_PREV_ATOM_JUST_BEFORE as ::core::ffi::c_int
                        || *p == NFA_PREV_ATOM_JUST_BEFORE_NEG as ::core::ffi::c_int)
                        as ::core::ffi::c_int;
                    let mut pattern: ::core::ffi::c_int = (*p
                        == NFA_PREV_ATOM_LIKE_PATTERN as ::core::ffi::c_int)
                        as ::core::ffi::c_int;
                    let mut start_state: ::core::ffi::c_int = 0;
                    let mut end_state: ::core::ffi::c_int = 0;
                    let mut n_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut zend: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
                    let mut skip: *mut nfa_state_T = ::core::ptr::null_mut::<nfa_state_T>();
                    match *p {
                        -981 => {
                            start_state = NFA_START_INVISIBLE as ::core::ffi::c_int;
                            end_state = NFA_END_INVISIBLE as ::core::ffi::c_int;
                        }
                        -980 => {
                            start_state = NFA_START_INVISIBLE_NEG as ::core::ffi::c_int;
                            end_state = NFA_END_INVISIBLE_NEG as ::core::ffi::c_int;
                        }
                        -979 => {
                            start_state = NFA_START_INVISIBLE_BEFORE as ::core::ffi::c_int;
                            end_state = NFA_END_INVISIBLE as ::core::ffi::c_int;
                        }
                        -978 => {
                            start_state = NFA_START_INVISIBLE_BEFORE_NEG as ::core::ffi::c_int;
                            end_state = NFA_END_INVISIBLE_NEG as ::core::ffi::c_int;
                        }
                        _ => {
                            start_state = NFA_START_PATTERN as ::core::ffi::c_int;
                            end_state = NFA_END_PATTERN as ::core::ffi::c_int;
                        }
                    }
                    if before != 0 {
                        p = p.offset(1);
                        n_0 = *p;
                    }
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += if pattern != 0 {
                            4 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    } else {
                        e = st_pop(&raw mut stackp, stack);
                        if stackp < stack {
                            st_error(postfix, end, p);
                            xfree(stack as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<nfa_state_T>();
                        }
                        s1 = alloc_state(
                            end_state,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s1.is_null() {
                            break '_theend;
                        }
                        s = alloc_state(start_state, e.start, s1);
                        if s.is_null() {
                            break '_theend;
                        }
                        if pattern != 0 {
                            skip = alloc_state(
                                NFA_SKIP as ::core::ffi::c_int,
                                ::core::ptr::null_mut::<nfa_state_T>(),
                                ::core::ptr::null_mut::<nfa_state_T>(),
                            );
                            if skip.is_null() {
                                break '_theend;
                            }
                            zend = alloc_state(
                                NFA_ZEND as ::core::ffi::c_int,
                                s1,
                                ::core::ptr::null_mut::<nfa_state_T>(),
                            );
                            if zend.is_null() {
                                break '_theend;
                            }
                            (*s1).out = skip;
                            patch(e.out, zend);
                            st_push(
                                frag(s, list1(&raw mut (*skip).out)),
                                &raw mut stackp,
                                stack_end,
                            );
                        } else {
                            patch(e.out, s1);
                            st_push(
                                frag(s, list1(&raw mut (*s1).out)),
                                &raw mut stackp,
                                stack_end,
                            );
                            if before != 0 {
                                if n_0 <= 0 as ::core::ffi::c_int {
                                    n_0 = nfa_max_width(e.start, 0 as ::core::ffi::c_int);
                                }
                                (*s).val = n_0;
                            }
                        }
                    }
                }
                -985 | -957 | -956 | -955 | -954 | -953 | -952 | -951 | -950 | -949 | -948
                | -937 | -936 | -935 | -934 | -933 | -932 | -931 | -930 | -929 | -928 | -999 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 2 as ::core::ffi::c_int;
                    } else {
                        mopen = *p;
                        match *p {
                            -999 => {
                                mclose = NFA_NCLOSE as ::core::ffi::c_int;
                            }
                            -937 => {
                                mclose = NFA_ZCLOSE as ::core::ffi::c_int;
                            }
                            -936 => {
                                mclose = NFA_ZCLOSE1 as ::core::ffi::c_int;
                            }
                            -935 => {
                                mclose = NFA_ZCLOSE2 as ::core::ffi::c_int;
                            }
                            -934 => {
                                mclose = NFA_ZCLOSE3 as ::core::ffi::c_int;
                            }
                            -933 => {
                                mclose = NFA_ZCLOSE4 as ::core::ffi::c_int;
                            }
                            -932 => {
                                mclose = NFA_ZCLOSE5 as ::core::ffi::c_int;
                            }
                            -931 => {
                                mclose = NFA_ZCLOSE6 as ::core::ffi::c_int;
                            }
                            -930 => {
                                mclose = NFA_ZCLOSE7 as ::core::ffi::c_int;
                            }
                            -929 => {
                                mclose = NFA_ZCLOSE8 as ::core::ffi::c_int;
                            }
                            -928 => {
                                mclose = NFA_ZCLOSE9 as ::core::ffi::c_int;
                            }
                            -985 => {
                                mclose = NFA_END_COMPOSING as ::core::ffi::c_int;
                            }
                            _ => {
                                mclose = *p + NSUBEXP as ::core::ffi::c_int;
                            }
                        }
                        if stackp == stack {
                            s = alloc_state(
                                mopen,
                                ::core::ptr::null_mut::<nfa_state_T>(),
                                ::core::ptr::null_mut::<nfa_state_T>(),
                            );
                            if s.is_null() {
                                break '_theend;
                            }
                            s1 = alloc_state(
                                mclose,
                                ::core::ptr::null_mut::<nfa_state_T>(),
                                ::core::ptr::null_mut::<nfa_state_T>(),
                            );
                            if s1.is_null() {
                                break '_theend;
                            }
                            patch(list1(&raw mut (*s).out), s1);
                            st_push(
                                frag(s, list1(&raw mut (*s1).out)),
                                &raw mut stackp,
                                stack_end,
                            );
                        } else {
                            e = st_pop(&raw mut stackp, stack);
                            if stackp < stack {
                                st_error(postfix, end, p);
                                xfree(stack as *mut ::core::ffi::c_void);
                                return ::core::ptr::null_mut::<nfa_state_T>();
                            }
                            s = alloc_state(mopen, e.start, ::core::ptr::null_mut::<nfa_state_T>());
                            if s.is_null() {
                                break '_theend;
                            }
                            s1 = alloc_state(
                                mclose,
                                ::core::ptr::null_mut::<nfa_state_T>(),
                                ::core::ptr::null_mut::<nfa_state_T>(),
                            );
                            if s1.is_null() {
                                break '_theend;
                            }
                            patch(e.out, s1);
                            if mopen == NFA_COMPOSING as ::core::ffi::c_int {
                                patch(list1(&raw mut (*s).out1), s1);
                            }
                            st_push(
                                frag(s, list1(&raw mut (*s1).out)),
                                &raw mut stackp,
                                stack_end,
                            );
                        }
                    }
                }
                -976 | -975 | -974 | -973 | -972 | -971 | -970 | -969 | -968 | -967 | -966
                | -965 | -964 | -963 | -962 | -961 | -960 | -959 => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 2 as ::core::ffi::c_int;
                    } else {
                        s = alloc_state(
                            *p,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        s1 = alloc_state(
                            NFA_SKIP as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s1.is_null() {
                            break '_theend;
                        }
                        patch(list1(&raw mut (*s).out), s1);
                        st_push(
                            frag(s, list1(&raw mut (*s1).out)),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -854 | -853 | -852 | -848 | -847 | -846 | -851 | -850 | -849 | -845 | -844
                | -843 => {
                    p = p.offset(1);
                    let mut n_1: ::core::ffi::c_int = *p;
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1 as ::core::ffi::c_int;
                    } else {
                        s = alloc_state(
                            *p.offset(-1 as ::core::ffi::c_int as isize),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        (*s).val = n_1;
                        st_push(
                            frag(s, list1(&raw mut (*s).out)),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
                -1001 | -1000 | _ => {
                    if nfa_calc_size == true_0 {
                        (*nstate.ptr()) += 1;
                    } else {
                        s = alloc_state(
                            *p,
                            ::core::ptr::null_mut::<nfa_state_T>(),
                            ::core::ptr::null_mut::<nfa_state_T>(),
                        );
                        if s.is_null() {
                            break '_theend;
                        }
                        st_push(
                            frag(s, list1(&raw mut (*s).out)),
                            &raw mut stackp,
                            stack_end,
                        );
                    }
                }
            }
            p = p.offset(1);
        }
        if nfa_calc_size == true_0 {
            (*nstate.ptr()) += 1;
        } else {
            e = st_pop(&raw mut stackp, stack);
            if stackp < stack {
                st_error(postfix, end, p);
                xfree(stack as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<nfa_state_T>();
            }
            if stackp != stack {
                xfree(stack as *mut ::core::ffi::c_void);
                emsg(
                    gettext(
                        b"E875: (NFA regexp) (While converting from postfix to NFA),too many states left on stack\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                );
                rc_did_emsg.set(true_0 != 0);
                return NULL_0 as *mut nfa_state_T;
            }
            if istate.get() >= nstate.get() {
                xfree(stack as *mut ::core::ffi::c_void);
                emsg(gettext(
                    b"E876: (NFA regexp) Not enough space to store the whole NFA \0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                rc_did_emsg.set(true_0 != 0);
                return NULL_0 as *mut nfa_state_T;
            }
            let c2rust_fresh14 = istate.get();
            istate.set(istate.get() + 1);
            matchstate = (*state_ptr.ptr()).offset(c2rust_fresh14 as isize);
            (*matchstate).c = NFA_MATCH as ::core::ffi::c_int;
            (*matchstate).out1 = ::core::ptr::null_mut::<nfa_state_T>();
            (*matchstate).out = (*matchstate).out1;
            (*matchstate).id = 0 as ::core::ffi::c_int;
            patch(e.out, matchstate);
            ret = e.start;
        }
    }
    xfree(stack as *mut ::core::ffi::c_void);
    return ret;
}
pub(crate) unsafe extern "C" fn nfa_postprocess(mut prog: *mut nfa_regprog_T) {
    let mut i: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < (*prog).nstate {
        c = (*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).c;
        if c == NFA_START_INVISIBLE as ::core::ffi::c_int
            || c == NFA_START_INVISIBLE_NEG as ::core::ffi::c_int
            || c == NFA_START_INVISIBLE_BEFORE as ::core::ffi::c_int
            || c == NFA_START_INVISIBLE_BEFORE_NEG as ::core::ffi::c_int
        {
            let mut directly: ::core::ffi::c_int = 0;
            if match_follows(
                (*(*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).out1).out,
                0 as ::core::ffi::c_int,
            ) {
                directly = true_0;
            } else {
                let mut ch_invisible: ::core::ffi::c_int = failure_chance(
                    (*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).out,
                    0 as ::core::ffi::c_int,
                );
                let mut ch_follows: ::core::ffi::c_int = failure_chance(
                    (*(*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).out1).out,
                    0 as ::core::ffi::c_int,
                );
                if c == NFA_START_INVISIBLE_BEFORE as ::core::ffi::c_int
                    || c == NFA_START_INVISIBLE_BEFORE_NEG as ::core::ffi::c_int
                {
                    if (*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).val
                        <= 0 as ::core::ffi::c_int
                        && ch_follows > 0 as ::core::ffi::c_int
                    {
                        directly = false_0;
                    } else {
                        directly = ((ch_follows * 10 as ::core::ffi::c_int) < ch_invisible)
                            as ::core::ffi::c_int;
                    }
                } else {
                    directly = (ch_follows < ch_invisible) as ::core::ffi::c_int;
                }
            }
            if directly != 0 {
                (*(&raw mut (*prog).state as *mut nfa_state_T).offset(i as isize)).c += 1;
            }
        }
        i += 1;
    }
}
