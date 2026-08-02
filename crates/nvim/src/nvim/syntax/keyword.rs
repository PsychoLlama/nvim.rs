//! `:syntax keyword` and the keyword hash tables.
//!
//! Keywords are not patterns: they live in one of two hash tables (case
//! sensitive and not) keyed by the keyword text, so a lookup is a hash rather
//! than a scan of every item. [`add_keyword`] fills them, [`syn_clear_keyword`]
//! and [`clear_keywtab`] empty them.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn syn_clear_keyword(
    mut id: ::core::ffi::c_int,
    mut ht: *mut hashtab_T,
) {
    unsafe {
        hash_lock(ht);
        let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
        let mut hi: *mut hashitem_T = (*ht).ht_array;
        while todo > 0 as ::core::ffi::c_int {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo -= 1;
                let mut kp_prev: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
                let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                    -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                        .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                        as isize),
                ) as *mut keyentry_T;
                while !kp.is_null() {
                    if (*kp).k_syn.id as ::core::ffi::c_int == id {
                        let mut kp_next: *mut keyentry_T = (*kp).ke_next;
                        if kp_prev.is_null() {
                            if kp_next.is_null() {
                                hash_remove(ht, hi);
                            } else {
                                (*hi).hi_key =
                                    &raw mut (*kp_next).keyword as *mut ::core::ffi::c_char;
                            }
                        } else {
                            (*kp_prev).ke_next = kp_next;
                        }
                        xfree((*kp).next_list as *mut ::core::ffi::c_void);
                        xfree((*kp).k_syn.cont_in_list as *mut ::core::ffi::c_void);
                        xfree(kp as *mut ::core::ffi::c_void);
                        kp = kp_next;
                    } else {
                        kp_prev = kp;
                        kp = (*kp).ke_next;
                    }
                }
            }
            hi = hi.offset(1);
        }
        hash_unlock(ht);
    }
}

pub(crate) unsafe extern "C" fn clear_keywtab(mut ht: *mut hashtab_T) {
    unsafe {
        let mut kp_next: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
        let mut todo: ::core::ffi::c_int = (*ht).ht_used as ::core::ffi::c_int;
        let mut hi: *mut hashitem_T = (*ht).ht_array;
        while todo > 0 as ::core::ffi::c_int {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo -= 1;
                let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                    -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                        .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                        as isize),
                ) as *mut keyentry_T;
                while !kp.is_null() {
                    kp_next = (*kp).ke_next;
                    xfree((*kp).next_list as *mut ::core::ffi::c_void);
                    xfree((*kp).k_syn.cont_in_list as *mut ::core::ffi::c_void);
                    xfree(kp as *mut ::core::ffi::c_void);
                    kp = kp_next;
                }
            }
            hi = hi.offset(1);
        }
        hash_clear(ht);
        hash_init(ht);
    }
}

pub(crate) unsafe extern "C" fn add_keyword(
    name: *mut ::core::ffi::c_char,
    mut namelen: size_t,
    id: ::core::ffi::c_int,
    flags: ::core::ffi::c_int,
    cont_in_list: *mut int16_t,
    next_list: *mut int16_t,
    conceal_char: ::core::ffi::c_int,
) {
    unsafe {
        let mut name_folded: [::core::ffi::c_char; 81] = [0; 81];
        let mut name_ic: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut name_iclen: size_t = 0;
        if (*(*curwin.get()).w_s).b_syn_ic != 0 {
            name_ic = str_foldcase(
                name,
                namelen as ::core::ffi::c_int,
                &raw mut name_folded as *mut ::core::ffi::c_char,
                MAXKEYWLEN + 1 as ::core::ffi::c_int,
            );
            name_iclen = strlen(name_ic);
        } else {
            name_ic = name;
            name_iclen = namelen;
        }
        let kp: *mut keyentry_T = xmalloc(
            (40 as size_t)
                .wrapping_add(name_iclen)
                .wrapping_add(1 as size_t),
        ) as *mut keyentry_T;
        strcpy(
            &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
            name_ic as *mut ::core::ffi::c_char,
        );
        (*kp).k_syn.id = id as int16_t;
        (*kp).k_syn.inc_tag = current_syn_inc_tag.get();
        (*kp).flags = flags;
        (*kp).k_char = conceal_char;
        (*kp).k_syn.cont_in_list = copy_id_list(cont_in_list);
        if !cont_in_list.is_null() {
            (*(*curwin.get()).w_s).b_syn_containedin = true_0;
        }
        (*kp).next_list = copy_id_list(next_list);
        let hash: hash_T = hash_hash(&raw mut (*kp).keyword as *mut ::core::ffi::c_char);
        let ht: *mut hashtab_T = if (*(*curwin.get()).w_s).b_syn_ic != 0 {
            &raw mut (*(*curwin.get()).w_s).b_keywtab_ic
        } else {
            &raw mut (*(*curwin.get()).w_s).b_keywtab
        };
        let hi: *mut hashitem_T = hash_lookup(
            ht,
            &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
            strlen(&raw mut (*kp).keyword as *mut ::core::ffi::c_char),
            hash,
        );
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            (*kp).ke_next = ::core::ptr::null_mut::<keyentry_T>();
            hash_add_item(
                ht,
                hi,
                &raw mut (*kp).keyword as *mut ::core::ffi::c_char,
                hash,
            );
        } else {
            (*kp).ke_next = (*hi).hi_key.offset(
                -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                    .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            (*hi).hi_key = &raw mut (*kp).keyword as *mut ::core::ffi::c_char;
        };
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_keyword(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut group_name_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut syn_id: ::core::ffi::c_int = 0;
        let mut keyword_copy: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
            flags: 0,
            keyword: false,
            sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
            has_cont_list: false,
            cont_list: ::core::ptr::null_mut::<int16_t>(),
            cont_in_list: ::core::ptr::null_mut::<int16_t>(),
            next_list: ::core::ptr::null_mut::<int16_t>(),
        };
        let mut conceal_char: ::core::ffi::c_int = NUL;
        let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
        if !rest.is_null() {
            if (*eap).skip != 0 {
                syn_id = -1 as ::core::ffi::c_int;
            } else {
                syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
            }
            if syn_id != 0 as ::core::ffi::c_int {
                keyword_copy =
                    xmalloc(strlen(rest).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            }
            if !keyword_copy.is_null() {
                syn_opt_arg.flags = 0 as ::core::ffi::c_int;
                syn_opt_arg.keyword = true_0 != 0;
                syn_opt_arg.sync_idx = ::core::ptr::null_mut::<::core::ffi::c_int>();
                syn_opt_arg.has_cont_list = false_0 != 0;
                syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
                syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
                let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut p: *mut ::core::ffi::c_char = keyword_copy;
                while !rest.is_null() && ends_excmd(*rest as ::core::ffi::c_int) == 0 {
                    rest = get_syn_options(
                        rest,
                        &raw mut syn_opt_arg,
                        &raw mut conceal_char,
                        (*eap).skip,
                    );
                    if rest.is_null() || ends_excmd(*rest as ::core::ffi::c_int) != 0 {
                        break;
                    }
                    while *rest as ::core::ffi::c_int != NUL
                        && !ascii_iswhite(*rest as ::core::ffi::c_int)
                    {
                        if *rest as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                            && *rest.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != NUL
                        {
                            rest = rest.offset(1);
                        }
                        let c2rust_fresh11 = rest;
                        rest = rest.offset(1);
                        let c2rust_fresh12 = p;
                        p = p.offset(1);
                        *c2rust_fresh12 = *c2rust_fresh11;
                    }
                    let c2rust_fresh13 = p;
                    p = p.offset(1);
                    *c2rust_fresh13 = NUL as ::core::ffi::c_char;
                    cnt += 1;
                    rest = skipwhite(rest);
                }
                '_error: {
                    if (*eap).skip == 0 {
                        syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                        let mut kwlen: size_t = 0 as size_t;
                        let mut kw: *mut ::core::ffi::c_char = keyword_copy;
                        loop {
                            cnt -= 1;
                            if cnt < 0 as ::core::ffi::c_int {
                                break '_error;
                            }
                            p = vim_strchr(kw, '[' as ::core::ffi::c_int);
                            loop {
                                if p.is_null() {
                                    kwlen = strlen(kw);
                                } else {
                                    *p = NUL as ::core::ffi::c_char;
                                    kwlen = p.offset_from(kw) as size_t;
                                }
                                add_keyword(
                                    kw,
                                    kwlen,
                                    syn_id,
                                    syn_opt_arg.flags,
                                    syn_opt_arg.cont_in_list,
                                    syn_opt_arg.next_list,
                                    conceal_char,
                                );
                                if p.is_null() {
                                    break;
                                }
                                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    == NUL
                                {
                                    semsg(
                                        gettext(b"E789: Missing ']': %s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        kw,
                                    );
                                    break '_error;
                                } else if *p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ']' as ::core::ffi::c_int
                                {
                                    if *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        != NUL
                                    {
                                        semsg(
                                            gettext(
                                                (e_trailing_char_after_rsb_str_str.ptr()
                                                    as *const _)
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            kw,
                                            p.offset(2 as ::core::ffi::c_int as isize),
                                        );
                                        break '_error;
                                    } else {
                                        kw = p.offset(1 as ::core::ffi::c_int as isize);
                                        kwlen = 1 as size_t;
                                        break;
                                    }
                                } else {
                                    let l: ::core::ffi::c_int =
                                        utfc_ptr2len(p.offset(1 as ::core::ffi::c_int as isize));
                                    memmove(
                                        p as *mut ::core::ffi::c_void,
                                        p.offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        l as size_t,
                                    );
                                    p = p.offset(l as isize);
                                }
                            }
                            kw = kw.offset(kwlen.wrapping_add(1 as size_t) as isize);
                        }
                    }
                }
                xfree(keyword_copy as *mut ::core::ffi::c_void);
                xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
                xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
            }
        }
        if !rest.is_null() {
            (*eap).nextcmd = check_nextcmd(rest);
        } else {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
        }
        redraw_curbuf_later(UPD_SOME_VALID);
        syn_stack_free_all((*curwin.get()).w_s);
    }
}
