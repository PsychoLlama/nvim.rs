extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn pow(__x: ::core::ffi::c_double, __y: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
}
pub type size_t = usize;
pub type int32_t = i32;
pub type linenr_T = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_mmfile {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
pub type mmfile_t = s_mmfile;
pub type diffcmppath_T = diffcmppath_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffcmppath_S {
    pub df_lev_score: ::core::ffi::c_int,
    pub df_path_n: size_t,
    pub df_choice_mem: [::core::ffi::c_int; 256],
    pub df_choice: [::core::ffi::c_int; 255],
    pub df_decision: [*mut diffcmppath_T; 255],
    pub df_optimal_choice: size_t,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 87] = unsafe {
    ::core::mem::transmute::<
        [u8; 87],
        [::core::ffi::c_char; 87],
    >(
        *b"size_t linematch_nbuffers(const mmfile_t **, const int *, const size_t, int **, _Bool)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
unsafe extern "C" fn line_len(mut m: *const mmfile_t) -> size_t {
    let mut s: *mut ::core::ffi::c_char = (*m).ptr;
    let mut end: *mut ::core::ffi::c_char = memchr(
        s as *const ::core::ffi::c_void,
        '\n' as ::core::ffi::c_int,
        (*m).size as size_t,
    ) as *mut ::core::ffi::c_char;
    return if !end.is_null() {
        end.offset_from(s) as size_t
    } else {
        (*m).size as size_t
    };
}
unsafe extern "C" fn matching_chars_iwhite(
    mut s1: *const mmfile_t,
    mut s2: *const mmfile_t,
) -> ::core::ffi::c_int {
    let mut sp: [mmfile_t; 2] = [mmfile_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    }; 2];
    let mut p: [[::core::ffi::c_char; 800]; 2] = [[0; 800]; 2];
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while k < 2 as ::core::ffi::c_int {
        let mut s: *const mmfile_t = if k == 0 as ::core::ffi::c_int { s1 } else { s2 };
        let mut pi: size_t = 0 as size_t;
        let mut slen: size_t =
            if ((800 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t) < line_len(s) {
                (800 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t
            } else {
                line_len(s)
            };
        let mut i: size_t = 0 as size_t;
        while i <= slen {
            let mut e: ::core::ffi::c_char = *(*s).ptr.offset(i as isize);
            if e as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
                && e as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
            {
                p[k as usize][pi as usize] = e;
                pi = pi.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        sp[k as usize] = s_mmfile {
            ptr: &raw mut *(&raw mut p as *mut [::core::ffi::c_char; 800]).offset(k as isize)
                as *mut ::core::ffi::c_char,
            size: pi as ::core::ffi::c_int,
        };
        k += 1;
    }
    return matching_chars(
        (&raw mut sp as *mut mmfile_t).offset(0 as ::core::ffi::c_int as isize),
        (&raw mut sp as *mut mmfile_t).offset(1 as ::core::ffi::c_int as isize),
    );
}
unsafe extern "C" fn matching_chars(
    mut m1: *const mmfile_t,
    mut m2: *const mmfile_t,
) -> ::core::ffi::c_int {
    let mut s1len: size_t =
        if ((800 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t) < line_len(m1) {
            (800 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t
        } else {
            line_len(m1)
        };
    let mut s2len: size_t =
        if ((800 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t) < line_len(m2) {
            (800 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t
        } else {
            line_len(m2)
        };
    let mut s1: *mut ::core::ffi::c_char = (*m1).ptr;
    let mut s2: *mut ::core::ffi::c_char = (*m2).ptr;
    let mut matrix: [[::core::ffi::c_int; 800]; 2] = [[0 as ::core::ffi::c_int; 800], [0; 800]];
    let mut icur: bool = true;
    let mut i: size_t = 0 as size_t;
    while i < s1len {
        icur = !icur;
        let mut e1: *mut ::core::ffi::c_int =
            &raw mut *(&raw mut matrix as *mut [::core::ffi::c_int; 800]).offset(icur as isize)
                as *mut ::core::ffi::c_int;
        let mut e2: *mut ::core::ffi::c_int = &raw mut *(&raw mut matrix
            as *mut [::core::ffi::c_int; 800])
            .offset(!icur as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_int;
        let mut j: size_t = 0 as size_t;
        while j < s2len {
            if *e2.offset(j.wrapping_add(1 as size_t) as isize)
                > *e1.offset(j.wrapping_add(1 as size_t) as isize)
            {
                *e1.offset(j.wrapping_add(1 as size_t) as isize) =
                    *e2.offset(j.wrapping_add(1 as size_t) as isize);
            }
            if *e1.offset(j as isize) > *e1.offset(j.wrapping_add(1 as size_t) as isize) {
                *e1.offset(j.wrapping_add(1 as size_t) as isize) = *e1.offset(j as isize);
            }
            if *s1.offset(i as isize) as ::core::ffi::c_int
                == *s2.offset(j as isize) as ::core::ffi::c_int
                && *e2.offset(j as isize) + 1 as ::core::ffi::c_int
                    > *e1.offset(j.wrapping_add(1 as size_t) as isize)
            {
                *e1.offset(j.wrapping_add(1 as size_t) as isize) =
                    *e2.offset(j as isize) + 1 as ::core::ffi::c_int;
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    return matrix[icur as usize][s2len as usize];
}
unsafe extern "C" fn count_n_matched_chars(
    mut sp: *mut *mut mmfile_t,
    n: size_t,
    mut iwhite: bool,
) -> ::core::ffi::c_int {
    let mut matched_chars: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut matched: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    while i < n {
        let mut j: size_t = i.wrapping_add(1 as size_t);
        while j < n {
            if !(**sp.offset(i as isize)).ptr.is_null() && !(**sp.offset(j as isize)).ptr.is_null()
            {
                matched += 1;
                matched_chars += if iwhite as ::core::ffi::c_int != 0 {
                    matching_chars_iwhite(*sp.offset(i as isize), *sp.offset(j as isize))
                } else {
                    matching_chars(*sp.offset(i as isize), *sp.offset(j as isize))
                };
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if matched >= 2 as ::core::ffi::c_int {
        matched_chars *= 2 as ::core::ffi::c_int;
        matched_chars /= matched;
    }
    return matched_chars;
}
#[no_mangle]
pub unsafe extern "C" fn fastforward_buf_to_lnum(mut s: mmfile_t, mut lnum: linenr_T) -> mmfile_t {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i as linenr_T) < lnum - 1 as linenr_T {
        let mut line_end: *mut ::core::ffi::c_char = memchr(
            s.ptr as *const ::core::ffi::c_void,
            '\n' as ::core::ffi::c_int,
            s.size as size_t,
        ) as *mut ::core::ffi::c_char;
        s.size = if !line_end.is_null() {
            (s.size as isize - line_end.offset_from(s.ptr)) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        s.ptr = line_end;
        if s.ptr.is_null() {
            break;
        }
        s.ptr = s.ptr.offset(1);
        s.size -= 1;
        i += 1;
    }
    return s;
}
unsafe extern "C" fn try_possible_paths(
    mut df_iters: *const ::core::ffi::c_int,
    mut paths: *const size_t,
    npaths: ::core::ffi::c_int,
    path_idx: ::core::ffi::c_int,
    mut choice: *mut ::core::ffi::c_int,
    mut diffcmppath: *mut diffcmppath_T,
    mut diff_len: *const ::core::ffi::c_int,
    ndiffs: size_t,
    mut diff_blk: *mut *const mmfile_t,
    mut iwhite: bool,
) {
    if path_idx == npaths {
        if *choice > 0 as ::core::ffi::c_int {
            let mut from_vals: [::core::ffi::c_int; 8] = [0 as ::core::ffi::c_int; 8];
            let mut to_vals: *const ::core::ffi::c_int = df_iters;
            let mut mm: [mmfile_t; 8] = [mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            }; 8];
            let mut current_lines: [*mut mmfile_t; 8] = [::core::ptr::null_mut::<mmfile_t>(); 8];
            let mut k: size_t = 0 as size_t;
            while k < ndiffs {
                from_vals[k as usize] = *df_iters.offset(k as isize);
                if *choice & (1 as ::core::ffi::c_int) << k != 0 {
                    from_vals[k as usize] -= 1;
                    mm[k as usize] = fastforward_buf_to_lnum(
                        **diff_blk.offset(k as isize),
                        *df_iters.offset(k as isize) as linenr_T,
                    );
                } else {
                    mm[k as usize] = s_mmfile {
                        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0,
                    };
                }
                current_lines[k as usize] = (&raw mut mm as *mut mmfile_t).offset(k as isize);
                k = k.wrapping_add(1);
            }
            let mut unwrapped_idx_from: size_t = unwrap_indexes(
                &raw mut from_vals as *mut ::core::ffi::c_int,
                diff_len,
                ndiffs,
            );
            let mut unwrapped_idx_to: size_t = unwrap_indexes(to_vals, diff_len, ndiffs);
            let mut matched_chars: ::core::ffi::c_int =
                count_n_matched_chars(&raw mut current_lines as *mut *mut mmfile_t, ndiffs, iwhite);
            let mut score: ::core::ffi::c_int =
                (*diffcmppath.offset(unwrapped_idx_from as isize)).df_lev_score + matched_chars;
            if score > (*diffcmppath.offset(unwrapped_idx_to as isize)).df_lev_score {
                (*diffcmppath.offset(unwrapped_idx_to as isize)).df_path_n = 1 as size_t;
                (*diffcmppath.offset(unwrapped_idx_to as isize)).df_decision
                    [0 as ::core::ffi::c_int as usize] =
                    diffcmppath.offset(unwrapped_idx_from as isize);
                (*diffcmppath.offset(unwrapped_idx_to as isize)).df_choice
                    [0 as ::core::ffi::c_int as usize] = *choice;
                (*diffcmppath.offset(unwrapped_idx_to as isize)).df_lev_score = score;
            } else if score == (*diffcmppath.offset(unwrapped_idx_to as isize)).df_lev_score {
                let c2rust_fresh1 = (*diffcmppath.offset(unwrapped_idx_to as isize)).df_path_n;
                (*diffcmppath.offset(unwrapped_idx_to as isize)).df_path_n = (*diffcmppath
                    .offset(unwrapped_idx_to as isize))
                .df_path_n
                .wrapping_add(1);
                let mut k_0: size_t = c2rust_fresh1;
                (*diffcmppath.offset(unwrapped_idx_to as isize)).df_decision[k_0 as usize] =
                    diffcmppath.offset(unwrapped_idx_from as isize);
                (*diffcmppath.offset(unwrapped_idx_to as isize)).df_choice[k_0 as usize] = *choice;
            }
        }
        return;
    }
    let mut bit_place: size_t = *paths.offset(path_idx as isize);
    *choice |= (1 as ::core::ffi::c_int) << bit_place;
    try_possible_paths(
        df_iters,
        paths,
        npaths,
        path_idx + 1 as ::core::ffi::c_int,
        choice,
        diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
    *choice &= !((1 as ::core::ffi::c_int) << bit_place);
    try_possible_paths(
        df_iters,
        paths,
        npaths,
        path_idx + 1 as ::core::ffi::c_int,
        choice,
        diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
}
unsafe extern "C" fn unwrap_indexes(
    mut values: *const ::core::ffi::c_int,
    mut diff_len: *const ::core::ffi::c_int,
    ndiffs: size_t,
) -> size_t {
    let mut num_unwrap_scalar: size_t = 1 as size_t;
    let mut k: size_t = 0 as size_t;
    while k < ndiffs {
        num_unwrap_scalar = num_unwrap_scalar
            .wrapping_mul((*diff_len.offset(k as isize) as size_t).wrapping_add(1 as size_t));
        k = k.wrapping_add(1);
    }
    let mut path_idx: size_t = 0 as size_t;
    let mut k_0: size_t = 0 as size_t;
    while k_0 < ndiffs {
        num_unwrap_scalar = num_unwrap_scalar
            .wrapping_div((*diff_len.offset(k_0 as isize) as size_t).wrapping_add(1 as size_t));
        let mut n: ::core::ffi::c_int = *values.offset(k_0 as isize);
        path_idx = path_idx.wrapping_add(num_unwrap_scalar.wrapping_mul(n as size_t));
        k_0 = k_0.wrapping_add(1);
    }
    return path_idx;
}
unsafe extern "C" fn populate_tensor(
    mut df_iters: *mut ::core::ffi::c_int,
    ch_dim: size_t,
    mut diffcmppath: *mut diffcmppath_T,
    mut diff_len: *const ::core::ffi::c_int,
    ndiffs: size_t,
    mut diff_blk: *mut *const mmfile_t,
    mut iwhite: bool,
) {
    if ch_dim == ndiffs {
        let mut npaths: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut paths: [size_t; 8] = [0; 8];
        let mut j: size_t = 0 as size_t;
        while j < ndiffs {
            if *df_iters.offset(j as isize) > 0 as ::core::ffi::c_int {
                paths[npaths as usize] = j;
                npaths += 1;
            }
            j = j.wrapping_add(1);
        }
        let mut choice: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut unwrapper_idx_to: size_t = unwrap_indexes(df_iters, diff_len, ndiffs);
        (*diffcmppath.offset(unwrapper_idx_to as isize)).df_lev_score = -1 as ::core::ffi::c_int;
        try_possible_paths(
            df_iters,
            &raw mut paths as *mut size_t,
            npaths,
            0 as ::core::ffi::c_int,
            &raw mut choice,
            diffcmppath,
            diff_len,
            ndiffs,
            diff_blk,
            iwhite,
        );
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i <= *diff_len.offset(ch_dim as isize) {
        *df_iters.offset(ch_dim as isize) = i;
        populate_tensor(
            df_iters,
            ch_dim.wrapping_add(1 as size_t),
            diffcmppath,
            diff_len,
            ndiffs,
            diff_blk,
            iwhite,
        );
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn linematch_nbuffers(
    mut diff_blk: *mut *const mmfile_t,
    mut diff_len: *const ::core::ffi::c_int,
    ndiffs: size_t,
    mut decisions: *mut *mut ::core::ffi::c_int,
    mut iwhite: bool,
) -> size_t {
    '_c2rust_label: {
        if ndiffs <= 8 as size_t {
        } else {
            __assert_fail(
                b"ndiffs <= LN_MAX_BUFS\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/linematch.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                332 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut memsize: size_t = 1 as size_t;
    let mut memsize_decisions: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < ndiffs {
        '_c2rust_label_0: {
            if *diff_len.offset(i as isize) >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"diff_len[i] >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/linematch.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    337 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        memsize = memsize
            .wrapping_mul((*diff_len.offset(i as isize) + 1 as ::core::ffi::c_int) as size_t);
        memsize_decisions = memsize_decisions.wrapping_add(*diff_len.offset(i as isize) as size_t);
        i = i.wrapping_add(1);
    }
    let mut diffcmppath: *mut diffcmppath_T =
        xmalloc(::core::mem::size_of::<diffcmppath_T>().wrapping_mul(memsize))
            as *mut diffcmppath_T;
    let mut n: size_t = pow(2.0f64, ndiffs as ::core::ffi::c_double) as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < memsize {
        (*diffcmppath.offset(i_0 as isize)).df_lev_score = 0 as ::core::ffi::c_int;
        (*diffcmppath.offset(i_0 as isize)).df_path_n = 0 as size_t;
        let mut j: size_t = 0 as size_t;
        while j < n {
            (*diffcmppath.offset(i_0 as isize)).df_choice_mem[j as usize] =
                -1 as ::core::ffi::c_int;
            j = j.wrapping_add(1);
        }
        i_0 = i_0.wrapping_add(1);
    }
    let mut df_iters: [::core::ffi::c_int; 8] = [0; 8];
    populate_tensor(
        &raw mut df_iters as *mut ::core::ffi::c_int,
        0 as size_t,
        diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
    let u: size_t = unwrap_indexes(diff_len, diff_len, ndiffs);
    let mut startNode: *mut diffcmppath_T = diffcmppath.offset(u as isize);
    *decisions =
        xmalloc(::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(memsize_decisions))
            as *mut ::core::ffi::c_int;
    let mut n_optimal: size_t = 0 as size_t;
    test_charmatch_paths(startNode, 0 as ::core::ffi::c_int);
    while (*startNode).df_path_n > 0 as size_t {
        let mut j_0: size_t = (*startNode).df_optimal_choice;
        let c2rust_fresh0 = n_optimal;
        n_optimal = n_optimal.wrapping_add(1);
        *(*decisions).offset(c2rust_fresh0 as isize) = (*startNode).df_choice[j_0 as usize];
        startNode = (*startNode).df_decision[j_0 as usize];
    }
    let mut i_1: size_t = 0 as size_t;
    while i_1 < n_optimal.wrapping_div(2 as size_t) {
        let mut tmp: ::core::ffi::c_int = *(*decisions).offset(i_1 as isize);
        *(*decisions).offset(i_1 as isize) =
            *(*decisions).offset(n_optimal.wrapping_sub(1 as size_t).wrapping_sub(i_1) as isize);
        *(*decisions).offset(n_optimal.wrapping_sub(1 as size_t).wrapping_sub(i_1) as isize) = tmp;
        i_1 = i_1.wrapping_add(1);
    }
    xfree(diffcmppath as *mut ::core::ffi::c_void);
    return n_optimal;
}
unsafe extern "C" fn test_charmatch_paths(
    mut node: *mut diffcmppath_T,
    mut lastdecision: ::core::ffi::c_int,
) -> size_t {
    if (*node).df_choice_mem[lastdecision as usize] == -1 as ::core::ffi::c_int {
        if (*node).df_path_n == 0 as size_t {
            (*node).df_choice_mem[lastdecision as usize] = 0 as ::core::ffi::c_int;
        } else {
            let mut minimum_turns: size_t = SIZE_MAX as size_t;
            let mut i: size_t = 0 as size_t;
            while i < (*node).df_path_n {
                let mut t: size_t = test_charmatch_paths(
                    (*node).df_decision[i as usize],
                    (*node).df_choice[i as usize],
                )
                .wrapping_add(
                    (if lastdecision != (*node).df_choice[i as usize] {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as size_t,
                );
                if t < minimum_turns {
                    (*node).df_optimal_choice = i;
                    minimum_turns = t;
                }
                i = i.wrapping_add(1);
            }
            (*node).df_choice_mem[lastdecision as usize] = minimum_turns as ::core::ffi::c_int;
        }
    }
    return (*node).df_choice_mem[lastdecision as usize] as size_t;
}
