//! `:syntax cluster` — named groups of syntax groups.
//!
//! A cluster is an id above `SYNID_CLUSTER` that stands for a list of other
//! ids, so `contains=@Foo` can be written once and edited in one place.
//! [`syn_combine_list`] is the `contains=`/`add=`/`remove=` set arithmetic, and
//! [`syn_check_cluster`] resolves a name to an id, creating it if needed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn syn_compare_stub(
    v1: *const ::core::ffi::c_void,
    v2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let s1: *const int16_t = v1 as *const int16_t;
        let s2: *const int16_t = v2 as *const int16_t;
        return if *s1 as ::core::ffi::c_int > *s2 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else if (*s1 as ::core::ffi::c_int) < *s2 as ::core::ffi::c_int {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
}

pub(crate) unsafe extern "C" fn syn_combine_list(
    clstr1: *mut *mut int16_t,
    clstr2: *mut *mut int16_t,
    list_op: ::core::ffi::c_int,
) {
    unsafe {
        let mut count1: size_t = 0 as size_t;
        let mut count2: size_t = 0 as size_t;
        let mut g1: *const int16_t = ::core::ptr::null::<int16_t>();
        let mut g2: *const int16_t = ::core::ptr::null::<int16_t>();
        let mut clstr: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
        if (*clstr2).is_null() {
            return;
        }
        if (*clstr1).is_null() || list_op == CLUSTER_REPLACE {
            if list_op == CLUSTER_REPLACE {
                xfree(*clstr1 as *mut ::core::ffi::c_void);
            }
            if list_op == CLUSTER_REPLACE || list_op == CLUSTER_ADD {
                *clstr1 = *clstr2;
            } else {
                xfree(*clstr2 as *mut ::core::ffi::c_void);
            }
            return;
        }
        g1 = *clstr1;
        while *g1 != 0 {
            count1 = count1.wrapping_add(1);
            g1 = g1.offset(1);
        }
        g2 = *clstr2;
        while *g2 != 0 {
            count2 = count2.wrapping_add(1);
            g2 = g2.offset(1);
        }
        qsort(
            *clstr1 as *mut ::core::ffi::c_void,
            count1,
            ::core::mem::size_of::<int16_t>(),
            Some(
                syn_compare_stub
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        qsort(
            *clstr2 as *mut ::core::ffi::c_void,
            count2,
            ::core::mem::size_of::<int16_t>(),
            Some(
                syn_compare_stub
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while round <= 2 as ::core::ffi::c_int {
            g1 = *clstr1;
            g2 = *clstr2;
            let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while *g1 as ::core::ffi::c_int != 0 && *g2 as ::core::ffi::c_int != 0 {
                if (*g1 as ::core::ffi::c_int) < *g2 as ::core::ffi::c_int {
                    if round == 2 as ::core::ffi::c_int {
                        *clstr.offset(count as isize) = *g1;
                    }
                    count += 1;
                    g1 = g1.offset(1);
                } else {
                    if list_op == CLUSTER_ADD {
                        if round == 2 as ::core::ffi::c_int {
                            *clstr.offset(count as isize) = *g2;
                        }
                        count += 1;
                    }
                    if *g1 as ::core::ffi::c_int == *g2 as ::core::ffi::c_int {
                        g1 = g1.offset(1);
                    }
                    g2 = g2.offset(1);
                }
            }
            while *g1 != 0 {
                if round == 2 as ::core::ffi::c_int {
                    *clstr.offset(count as isize) = *g1;
                }
                g1 = g1.offset(1);
                count += 1;
            }
            if list_op == CLUSTER_ADD {
                while *g2 != 0 {
                    if round == 2 as ::core::ffi::c_int {
                        *clstr.offset(count as isize) = *g2;
                    }
                    g2 = g2.offset(1);
                    count += 1;
                }
            }
            if round == 1 as ::core::ffi::c_int {
                if count == 0 as ::core::ffi::c_int {
                    clstr = ::core::ptr::null_mut::<int16_t>();
                    break;
                } else {
                    clstr = xmalloc(
                        (count as size_t)
                            .wrapping_add(1 as size_t)
                            .wrapping_mul(::core::mem::size_of::<int16_t>()),
                    ) as *mut int16_t;
                    *clstr.offset(count as isize) = 0 as int16_t;
                }
            }
            round += 1;
        }
        xfree(*clstr1 as *mut ::core::ffi::c_void);
        xfree(*clstr2 as *mut ::core::ffi::c_void);
        *clstr1 = clstr;
    }
}

pub(crate) unsafe extern "C" fn syn_scl_name2id(
    mut name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut name_u: *mut ::core::ffi::c_char = vim_strsave_up(name);
        let mut i: ::core::ffi::c_int = 0;
        i = (*(*curwin.get()).w_s).b_syn_clusters.ga_len;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            if !(*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                .offset(i as isize))
            .scl_name_u
            .is_null()
                && strcmp(
                    name_u,
                    (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                        .offset(i as isize))
                    .scl_name_u,
                ) == 0 as ::core::ffi::c_int
            {
                break;
            }
        }
        xfree(name_u as *mut ::core::ffi::c_void);
        return if i < 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            i + SYNID_CLUSTER
        };
    }
}

pub(crate) unsafe extern "C" fn syn_scl_namen2id(
    mut linep: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut name: *mut ::core::ffi::c_char = xstrnsave(linep, len as size_t);
        let mut id: ::core::ffi::c_int = syn_scl_name2id(name);
        xfree(name as *mut ::core::ffi::c_void);
        return id;
    }
}

pub(crate) unsafe extern "C" fn syn_check_cluster(
    mut pp: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut name: *mut ::core::ffi::c_char = xstrnsave(pp, len as size_t);
        let mut id: ::core::ffi::c_int = syn_scl_name2id(name);
        if id == 0 as ::core::ffi::c_int {
            id = syn_add_cluster(name);
        } else {
            xfree(name as *mut ::core::ffi::c_void);
        }
        return id;
    }
}

pub(crate) unsafe extern "C" fn syn_add_cluster(
    mut name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if (*(*curwin.get()).w_s).b_syn_clusters.ga_data.is_null() {
            (*(*curwin.get()).w_s).b_syn_clusters.ga_itemsize =
                ::core::mem::size_of::<syn_cluster_T>() as ::core::ffi::c_int;
            ga_set_growsize(
                &raw mut (*(*curwin.get()).w_s).b_syn_clusters,
                10 as ::core::ffi::c_int,
            );
        }
        let mut len: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_clusters.ga_len;
        if len >= MAX_CLUSTER_ID {
            emsg(gettext(
                b"E848: Too many syntax clusters\0".as_ptr() as *const ::core::ffi::c_char
            ));
            xfree(name as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        let mut scp: *mut syn_cluster_T = ga_append_via_ptr(
            &raw mut (*(*curwin.get()).w_s).b_syn_clusters,
            ::core::mem::size_of::<syn_cluster_T>(),
        ) as *mut syn_cluster_T;
        memset(
            scp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<syn_cluster_T>(),
        );
        (*scp).scl_name = name;
        (*scp).scl_name_u = vim_strsave_up(name);
        (*scp).scl_list = ::core::ptr::null_mut::<int16_t>();
        if strcasecmp(
            name,
            b"Spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*(*curwin.get()).w_s).b_spell_cluster_id = len + SYNID_CLUSTER;
        }
        if strcasecmp(
            name,
            b"NoSpell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*(*curwin.get()).w_s).b_nospell_cluster_id = len + SYNID_CLUSTER;
        }
        return len + SYNID_CLUSTER;
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_cluster(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut group_name_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut got_clstr: bool = false_0 != 0;
        let mut opt_len: ::core::ffi::c_int = 0;
        let mut list_op: ::core::ffi::c_int = 0;
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
        if !rest.is_null() {
            let mut scl_id: ::core::ffi::c_int =
                syn_check_cluster(arg, group_name_end.offset_from(arg) as ::core::ffi::c_int);
            if scl_id == 0 as ::core::ffi::c_int {
                return;
            }
            scl_id -= SYNID_CLUSTER;
            loop {
                if strncasecmp(
                    rest,
                    b"add\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    3 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
                    && (ascii_iswhite(
                        *rest.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                        || *rest.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '=' as ::core::ffi::c_int)
                {
                    opt_len = 3 as ::core::ffi::c_int;
                    list_op = CLUSTER_ADD;
                } else if strncasecmp(
                    rest,
                    b"remove\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    6 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
                    && (ascii_iswhite(
                        *rest.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                        || *rest.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '=' as ::core::ffi::c_int)
                {
                    opt_len = 6 as ::core::ffi::c_int;
                    list_op = CLUSTER_SUBTRACT;
                } else {
                    if !(strncasecmp(
                        rest,
                        b"contains\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        8 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int
                        && (ascii_iswhite(
                            *rest.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                            || *rest.offset(8 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '=' as ::core::ffi::c_int))
                    {
                        break;
                    }
                    opt_len = 8 as ::core::ffi::c_int;
                    list_op = CLUSTER_REPLACE;
                }
                let mut clstr_list: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
                if get_id_list(
                    &raw mut rest,
                    opt_len,
                    &raw mut clstr_list,
                    (*eap).skip != 0,
                ) == FAIL
                {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        rest,
                    );
                    break;
                } else {
                    if scl_id >= 0 as ::core::ffi::c_int {
                        syn_combine_list(
                            &raw mut (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data
                                as *mut syn_cluster_T)
                                .offset(scl_id as isize))
                            .scl_list,
                            &raw mut clstr_list,
                            list_op,
                        );
                    } else {
                        xfree(clstr_list as *mut ::core::ffi::c_void);
                    }
                    got_clstr = true_0 != 0;
                }
            }
            if got_clstr {
                redraw_curbuf_later(UPD_SOME_VALID);
                syn_stack_free_all((*curwin.get()).w_s);
            }
        }
        if !got_clstr {
            emsg(gettext(
                b"E400: No cluster specified\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) == 0 {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
        }
    }
}
