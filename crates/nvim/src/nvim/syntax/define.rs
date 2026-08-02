//! `:syntax match`, `:syntax region` and `:syntax include`.
//!
//! The three subcommands that add a pattern-based item, plus
//! [`get_syn_pattern`], which parses one `/pat/` with its `ms=`/`me=`/... offset
//! suffixes into a `synpat_T`. `:syntax include` is here too: it sources another
//! syntax file under an inclusion tag so its toplevel items become contained
//! ones.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn syn_incl_toplevel(
    mut id: ::core::ffi::c_int,
    mut flagsp: *mut ::core::ffi::c_int,
) {
    unsafe {
        if *flagsp & HL_CONTAINED != 0
            || (*(*curwin.get()).w_s).b_syn_topgrp == 0 as ::core::ffi::c_int
        {
            return;
        }
        *flagsp |= HL_CONTAINED | HL_INCLUDED_TOPLEVEL;
        if (*(*curwin.get()).w_s).b_syn_topgrp >= SYNID_CLUSTER {
            let mut grp_list: *mut int16_t =
                xmalloc((2 as size_t).wrapping_mul(::core::mem::size_of::<int16_t>()))
                    as *mut int16_t;
            let mut tlg_id: ::core::ffi::c_int =
                (*(*curwin.get()).w_s).b_syn_topgrp - SYNID_CLUSTER;
            *grp_list.offset(0 as ::core::ffi::c_int as isize) = id as int16_t;
            *grp_list.offset(1 as ::core::ffi::c_int as isize) = 0 as int16_t;
            syn_combine_list(
                &raw mut (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                    .offset(tlg_id as isize))
                .scl_list,
                &raw mut grp_list,
                CLUSTER_ADD,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_include(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut sgl_id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut group_name_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut source: bool = false_0 != 0;
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '@' as ::core::ffi::c_int
        {
            arg = arg.offset(1);
            let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
            if rest.is_null() {
                emsg(gettext(
                    b"E397: Filename required\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
            sgl_id = syn_check_cluster(arg, group_name_end.offset_from(arg) as ::core::ffi::c_int);
            if sgl_id == 0 as ::core::ffi::c_int {
                return;
            }
            (*eap).arg = rest;
        }
        (*eap).argt = ((*eap).argt as ::core::ffi::c_uint | (EX_XFILE | EX_NOSPC)) as uint32_t;
        separate_nextcmd(eap);
        if *(*eap).arg as ::core::ffi::c_int == '<' as ::core::ffi::c_int
            || *(*eap).arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int
            || path_is_absolute((*eap).arg) as ::core::ffi::c_int != 0
        {
            source = true_0 != 0;
            if expand_filename(eap, syn_cmdlinep.get(), &raw mut errormsg) == FAIL {
                if !errormsg.is_null() {
                    emsg(errormsg);
                }
                return;
            }
        }
        if running_syn_inc_tag.get() >= MAX_SYN_INC_TAG {
            emsg(gettext(
                b"E847: Too many syntax includes\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
        let mut prev_syn_inc_tag: ::core::ffi::c_int = current_syn_inc_tag.get();
        (*running_syn_inc_tag.ptr()) += 1;
        current_syn_inc_tag.set(running_syn_inc_tag.get());
        let mut prev_toplvl_grp: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_topgrp;
        (*(*curwin.get()).w_s).b_syn_topgrp = sgl_id;
        if if source as ::core::ffi::c_int != 0 {
            (do_source(
                (*eap).arg,
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) == FAIL) as ::core::ffi::c_int
        } else {
            (source_runtime((*eap).arg, DIP_ALL as ::core::ffi::c_int) == FAIL)
                as ::core::ffi::c_int
        } != 0
        {
            semsg(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        }
        (*(*curwin.get()).w_s).b_syn_topgrp = prev_toplvl_grp;
        current_syn_inc_tag.set(prev_syn_inc_tag);
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_match(
    mut eap: *mut exarg_T,
    mut syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut group_name_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut item: synpat_T = synpat_T {
            sp_type: 0,
            sp_syncing: false,
            sp_syn_match_id: 0,
            sp_off_flags: 0,
            sp_offsets: [0; 7],
            sp_flags: 0,
            sp_cchar: 0,
            sp_ic: 0,
            sp_sync_idx: 0,
            sp_line_id: 0,
            sp_startcol: 0,
            sp_cont_list: ::core::ptr::null_mut::<int16_t>(),
            sp_next_list: ::core::ptr::null_mut::<int16_t>(),
            sp_syn: sp_syn {
                inc_tag: 0,
                id: 0,
                cont_in_list: ::core::ptr::null_mut::<int16_t>(),
            },
            sp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            sp_prog: ::core::ptr::null_mut::<regprog_T>(),
            sp_time: syn_time_T {
                total: 0,
                slowest: 0,
                count: 0,
                match_0: 0,
            },
        };
        let mut syn_id: ::core::ffi::c_int = 0;
        let mut syn_opt_arg: syn_opt_arg_T = syn_opt_arg_T {
            flags: 0,
            keyword: false,
            sync_idx: ::core::ptr::null_mut::<::core::ffi::c_int>(),
            has_cont_list: false,
            cont_list: ::core::ptr::null_mut::<int16_t>(),
            cont_in_list: ::core::ptr::null_mut::<int16_t>(),
            next_list: ::core::ptr::null_mut::<int16_t>(),
        };
        let mut sync_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut conceal_char: ::core::ffi::c_int = NUL;
        let mut rest: *mut ::core::ffi::c_char = get_group_name(arg, &raw mut group_name_end);
        syn_opt_arg.flags = 0 as ::core::ffi::c_int;
        syn_opt_arg.keyword = false_0 != 0;
        syn_opt_arg.sync_idx = if syncing != 0 {
            &raw mut sync_idx
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_int>()
        };
        syn_opt_arg.has_cont_list = true_0 != 0;
        syn_opt_arg.cont_list = ::core::ptr::null_mut::<int16_t>();
        syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
        syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
        rest = get_syn_options(
            rest,
            &raw mut syn_opt_arg,
            &raw mut conceal_char,
            (*eap).skip,
        );
        init_syn_patterns();
        memset(
            &raw mut item as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<synpat_T>(),
        );
        rest = get_syn_pattern(rest, &raw mut item);
        if vim_regcomp_had_eol() != 0 && syn_opt_arg.flags & HL_EXCLUDENL == 0 {
            syn_opt_arg.flags |= HL_HAS_EOL;
        }
        rest = get_syn_options(
            rest,
            &raw mut syn_opt_arg,
            &raw mut conceal_char,
            (*eap).skip,
        );
        if !rest.is_null() {
            (*eap).nextcmd = check_nextcmd(rest);
            if ends_excmd(*rest as ::core::ffi::c_int) == 0 || (*eap).skip != 0 {
                rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
                if syn_id != 0 as ::core::ffi::c_int {
                    syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                    let mut spp: *mut synpat_T = ga_append_via_ptr(
                        &raw mut (*(*curwin.get()).w_s).b_syn_patterns,
                        ::core::mem::size_of::<synpat_T>(),
                    ) as *mut synpat_T;
                    *spp = item;
                    (*spp).sp_syncing = syncing != 0;
                    (*spp).sp_type = SPTYPE_MATCH as ::core::ffi::c_char;
                    (*spp).sp_syn.id = syn_id as int16_t;
                    (*spp).sp_syn.inc_tag = current_syn_inc_tag.get();
                    (*spp).sp_flags = syn_opt_arg.flags;
                    (*spp).sp_sync_idx = sync_idx;
                    (*spp).sp_cont_list = syn_opt_arg.cont_list;
                    (*spp).sp_syn.cont_in_list = syn_opt_arg.cont_in_list;
                    (*spp).sp_cchar = conceal_char;
                    if !syn_opt_arg.cont_in_list.is_null() {
                        (*(*curwin.get()).w_s).b_syn_containedin = true_0;
                    }
                    (*spp).sp_next_list = syn_opt_arg.next_list;
                    if syn_opt_arg.flags & (HL_SYNC_HERE | HL_SYNC_THERE) != 0 {
                        (*(*curwin.get()).w_s).b_syn_sync_flags |= SF_MATCH;
                    }
                    if syn_opt_arg.flags & HL_FOLD != 0 {
                        (*(*curwin.get()).w_s).b_syn_folditems += 1;
                    }
                    redraw_curbuf_later(UPD_SOME_VALID);
                    syn_stack_free_all((*curwin.get()).w_s);
                    return;
                }
            }
        }
        vim_regfree(item.sp_prog);
        xfree(item.sp_pattern as *mut ::core::ffi::c_void);
        xfree(syn_opt_arg.cont_list as *mut ::core::ffi::c_void);
        xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
        xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
        if rest.is_null() {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_region(
    mut eap: *mut exarg_T,
    mut syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut group_name_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut rest: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut key_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut item: ::core::ffi::c_int = 0;
        let mut pat_ptrs: [*mut pat_ptr; 3] = [::core::ptr::null_mut::<pat_ptr>(); 3];
        let mut ppp: *mut pat_ptr = ::core::ptr::null_mut::<pat_ptr>();
        let mut ppp_next: *mut pat_ptr = ::core::ptr::null_mut::<pat_ptr>();
        let mut pat_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut syn_id: ::core::ffi::c_int = 0;
        let mut matchgroup_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut not_enough: bool = false_0 != 0;
        let mut illegal: bool = false_0 != 0;
        let mut success: bool = false_0 != 0;
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
        rest = get_group_name(arg, &raw mut group_name_end);
        pat_ptrs[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
        pat_ptrs[1 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
        pat_ptrs[2 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<pat_ptr>();
        init_syn_patterns();
        syn_opt_arg.flags = 0 as ::core::ffi::c_int;
        syn_opt_arg.keyword = false_0 != 0;
        syn_opt_arg.sync_idx = ::core::ptr::null_mut::<::core::ffi::c_int>();
        syn_opt_arg.has_cont_list = true_0 != 0;
        syn_opt_arg.cont_list = ::core::ptr::null_mut::<int16_t>();
        syn_opt_arg.cont_in_list = ::core::ptr::null_mut::<int16_t>();
        syn_opt_arg.next_list = ::core::ptr::null_mut::<int16_t>();
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
            key_end = rest;
            while *key_end as ::core::ffi::c_int != 0
                && !ascii_iswhite(*key_end as ::core::ffi::c_int)
                && *key_end as ::core::ffi::c_int != '=' as ::core::ffi::c_int
            {
                key_end = key_end.offset(1);
            }
            xfree(key as *mut ::core::ffi::c_void);
            key = vim_strnsave_up(rest, key_end.offset_from(rest) as size_t);
            if strcmp(key, b"MATCHGROUP\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                item = ITEM_MATCHGROUP;
            } else if strcmp(key, b"START\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                item = ITEM_START;
            } else if strcmp(key, b"END\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                item = ITEM_END;
            } else {
                if strcmp(key, b"SKIP\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int
                {
                    break;
                }
                if !pat_ptrs[ITEM_SKIP as usize].is_null() {
                    illegal = true_0 != 0;
                    break;
                } else {
                    item = ITEM_SKIP;
                }
            }
            rest = skipwhite(key_end);
            if *rest as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
                semsg(
                    gettext(b"E398: Missing '=': %s\0".as_ptr() as *const ::core::ffi::c_char),
                    arg,
                );
                break;
            } else {
                rest = skipwhite(rest.offset(1 as ::core::ffi::c_int as isize));
                if *rest as ::core::ffi::c_int == NUL {
                    not_enough = true_0 != 0;
                    break;
                } else if item == ITEM_MATCHGROUP {
                    let mut p: *mut ::core::ffi::c_char = skiptowhite(rest);
                    if p.offset_from(rest) == 4 as isize
                        && strncmp(
                            rest,
                            b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                            4 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        || (*eap).skip != 0
                    {
                        matchgroup_id = 0 as ::core::ffi::c_int;
                    } else {
                        matchgroup_id = syn_check_group(rest, p.offset_from(rest) as size_t);
                        if matchgroup_id == 0 as ::core::ffi::c_int {
                            illegal = true_0 != 0;
                            break;
                        }
                    }
                    rest = skipwhite(p);
                } else {
                    ppp = xmalloc(::core::mem::size_of::<pat_ptr>()) as *mut pat_ptr;
                    (*ppp).pp_next = pat_ptrs[item as usize] as *mut pat_ptr;
                    pat_ptrs[item as usize] = ppp as *mut pat_ptr;
                    (*ppp).pp_synp =
                        xcalloc(1 as size_t, ::core::mem::size_of::<synpat_T>()) as *mut synpat_T;
                    if item == ITEM_START {
                        reg_do_extmatch.set(REX_SET);
                    } else {
                        '_c2rust_label: {
                            if item == 1 as ::core::ffi::c_int || item == 2 as ::core::ffi::c_int {
                            } else {
                                __assert_fail(
                                    b"item == ITEM_SKIP || item == ITEM_END\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/syntax.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                    4333 as ::core::ffi::c_uint,
                                    b"void syn_cmd_region(exarg_T *, int)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        reg_do_extmatch.set(REX_USE);
                    }
                    rest = get_syn_pattern(rest, (*ppp).pp_synp);
                    reg_do_extmatch.set(0 as ::core::ffi::c_int);
                    if item == ITEM_END
                        && vim_regcomp_had_eol() != 0
                        && syn_opt_arg.flags & HL_EXCLUDENL == 0
                    {
                        (*(*ppp).pp_synp).sp_flags |= HL_HAS_EOL;
                    }
                    (*ppp).pp_matchgroup_id = matchgroup_id;
                    pat_count += 1;
                }
            }
        }
        xfree(key as *mut ::core::ffi::c_void);
        if illegal as ::core::ffi::c_int != 0 || not_enough as ::core::ffi::c_int != 0 {
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !rest.is_null()
            && (pat_ptrs[ITEM_START as usize].is_null() || pat_ptrs[ITEM_END as usize].is_null())
        {
            not_enough = true_0 != 0;
            rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !rest.is_null() {
            (*eap).nextcmd = check_nextcmd(rest);
            if ends_excmd(*rest as ::core::ffi::c_int) == 0 || (*eap).skip != 0 {
                rest = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                ga_grow(&raw mut (*(*curwin.get()).w_s).b_syn_patterns, pat_count);
                syn_id = syn_check_group(arg, group_name_end.offset_from(arg) as size_t);
                if syn_id != 0 as ::core::ffi::c_int {
                    syn_incl_toplevel(syn_id, &raw mut syn_opt_arg.flags);
                    let mut idx: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
                    item = ITEM_START;
                    while item <= ITEM_END {
                        ppp = pat_ptrs[item as usize] as *mut pat_ptr;
                        while !ppp.is_null() {
                            *((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize) = *(*ppp).pp_synp;
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_syncing = syncing != 0;
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_type = (if item == ITEM_START {
                                SPTYPE_START
                            } else if item == ITEM_SKIP {
                                SPTYPE_SKIP
                            } else {
                                SPTYPE_END
                            }) as ::core::ffi::c_char;
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_flags |= syn_opt_arg.flags;
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_syn
                            .id = syn_id as int16_t;
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_syn
                            .inc_tag = current_syn_inc_tag.get();
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_syn_match_id = (*ppp).pp_matchgroup_id as int16_t;
                            (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                                .offset(idx as isize))
                            .sp_cchar = conceal_char;
                            if item == ITEM_START {
                                (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data
                                    as *mut synpat_T)
                                    .offset(idx as isize))
                                .sp_cont_list = syn_opt_arg.cont_list;
                                (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data
                                    as *mut synpat_T)
                                    .offset(idx as isize))
                                .sp_syn
                                .cont_in_list = syn_opt_arg.cont_in_list;
                                if !syn_opt_arg.cont_in_list.is_null() {
                                    (*(*curwin.get()).w_s).b_syn_containedin = true_0;
                                }
                                (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data
                                    as *mut synpat_T)
                                    .offset(idx as isize))
                                .sp_next_list = syn_opt_arg.next_list;
                            }
                            (*(*curwin.get()).w_s).b_syn_patterns.ga_len += 1;
                            idx += 1;
                            if syn_opt_arg.flags & HL_FOLD != 0 {
                                (*(*curwin.get()).w_s).b_syn_folditems += 1;
                            }
                            ppp = (*ppp).pp_next;
                        }
                        item += 1;
                    }
                    redraw_curbuf_later(UPD_SOME_VALID);
                    syn_stack_free_all((*curwin.get()).w_s);
                    success = true_0 != 0;
                }
            }
        }
        item = ITEM_START;
        while item <= ITEM_END {
            ppp = pat_ptrs[item as usize] as *mut pat_ptr;
            while !ppp.is_null() {
                if !success && !(*ppp).pp_synp.is_null() {
                    vim_regfree((*(*ppp).pp_synp).sp_prog);
                    xfree((*(*ppp).pp_synp).sp_pattern as *mut ::core::ffi::c_void);
                }
                xfree((*ppp).pp_synp as *mut ::core::ffi::c_void);
                ppp_next = (*ppp).pp_next;
                xfree(ppp as *mut ::core::ffi::c_void);
                ppp = ppp_next;
            }
            item += 1;
        }
        if !success {
            xfree(syn_opt_arg.cont_list as *mut ::core::ffi::c_void);
            xfree(syn_opt_arg.cont_in_list as *mut ::core::ffi::c_void);
            xfree(syn_opt_arg.next_list as *mut ::core::ffi::c_void);
            if not_enough {
                semsg(
                    gettext(b"E399: Not enough arguments: syntax region %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    arg,
                );
            } else if illegal as ::core::ffi::c_int != 0 || rest.is_null() {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    arg,
                );
            }
        }
    }
}

pub(crate) unsafe extern "C" fn init_syn_patterns() {
    unsafe {
        (*(*curwin.get()).w_s).b_syn_patterns.ga_itemsize =
            ::core::mem::size_of::<synpat_T>() as ::core::ffi::c_int;
        ga_set_growsize(
            &raw mut (*(*curwin.get()).w_s).b_syn_patterns,
            10 as ::core::ffi::c_int,
        );
    }
}

pub(crate) unsafe extern "C" fn get_syn_pattern(
    mut arg: *mut ::core::ffi::c_char,
    mut ci: *mut synpat_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut idx: ::core::ffi::c_int = 0;
        if arg.is_null()
            || *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || *arg.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut end: *mut ::core::ffi::c_char = skip_regexp(
            arg.offset(1 as ::core::ffi::c_int as isize),
            *arg as ::core::ffi::c_int,
            true_0,
        );
        if *end as ::core::ffi::c_int != *arg as ::core::ffi::c_int {
            semsg(
                gettext(b"E401: Pattern delimiter not found: %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                arg,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        (*ci).sp_pattern = xstrnsave(
            arg.offset(1 as ::core::ffi::c_int as isize),
            (end.offset_from(arg) as size_t).wrapping_sub(1 as size_t),
        );
        let mut cpo_save: *mut ::core::ffi::c_char = p_cpo.get();
        p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
        (*ci).sp_prog = vim_regcomp((*ci).sp_pattern, RE_MAGIC);
        p_cpo.set(cpo_save);
        if (*ci).sp_prog.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        (*ci).sp_ic = (*(*curwin.get()).w_s).b_syn_ic;
        syn_clear_time(&mut (*ci).sp_time);
        end = end.offset(1);
        loop {
            idx = SPO_COUNT;
            loop {
                idx -= 1;
                if idx < 0 as ::core::ffi::c_int {
                    break;
                }
                if strncmp(
                    end,
                    (*spo_name_tab.ptr())[idx as usize] as *const ::core::ffi::c_char,
                    3 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    break;
                }
            }
            if idx >= 0 as ::core::ffi::c_int {
                let mut p: *mut ::core::ffi::c_int =
                    (&raw mut (*ci).sp_offsets as *mut ::core::ffi::c_int).offset(idx as isize);
                if idx != SPO_LC_OFF {
                    match *end.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                        115 | 98 => {}
                        101 => {
                            idx += SPO_COUNT;
                        }
                        _ => {
                            idx = -1 as ::core::ffi::c_int;
                        }
                    }
                }
                if idx >= 0 as ::core::ffi::c_int {
                    (*ci).sp_off_flags = ((*ci).sp_off_flags as ::core::ffi::c_int
                        | ((1 as ::core::ffi::c_int) << idx) as int16_t as ::core::ffi::c_int)
                        as int16_t;
                    if idx == SPO_LC_OFF {
                        end = end.offset(3 as ::core::ffi::c_int as isize);
                        *p = getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                        if (*ci).sp_off_flags as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << SPO_MS_OFF
                            == 0
                        {
                            (*ci).sp_off_flags = ((*ci).sp_off_flags as ::core::ffi::c_int
                                | (1 as ::core::ffi::c_int) << SPO_MS_OFF)
                                as int16_t;
                            (*ci).sp_offsets[SPO_MS_OFF as usize] = *p;
                        }
                    } else {
                        end = end.offset(4 as ::core::ffi::c_int as isize);
                        if *end as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                            end = end.offset(1);
                            *p = getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                        } else if *end as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                            end = end.offset(1);
                            *p = -getdigits_int(&raw mut end, true_0 != 0, 0 as ::core::ffi::c_int);
                        }
                    }
                    if *end as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                        break;
                    }
                    end = end.offset(1);
                }
            }
            if idx < 0 as ::core::ffi::c_int {
                break;
            }
        }
        if ends_excmd(*end as ::core::ffi::c_int) == 0 && !ascii_iswhite(*end as ::core::ffi::c_int)
        {
            semsg(
                gettext(b"E402: Garbage after pattern: %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return skipwhite(end);
    }
}
