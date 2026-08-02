//! The `:syntax` Ex command and the clearing half.
//!
//! [`ex_syntax`] picks a subcommand out of `subcommands` and dispatches; the
//! `syn_cmd_*` functions here are the ones that set a mode or throw work away
//! rather than define an item -- `case`, `conceal`, `foldlevel`, `spell`,
//! `iskeyword`, `clear`, and the `on`/`off`/`enable`/`manual`/`reset` family.
//! [`syntax_clear`] is the teardown every one of those and every buffer switch
//! goes through.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn syn_cmd_conceal(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        next = skiptowhite(arg);
        if *arg as ::core::ffi::c_int == NUL {
            if (*(*curwin.get()).w_s).b_syn_conceal != 0 {
                msg(
                    b"syntax conceal on\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            } else {
                msg(
                    b"syntax conceal off\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
        } else if strncasecmp(
            arg,
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && next.offset_from(arg) == 2 as isize
        {
            (*(*curwin.get()).w_s).b_syn_conceal = true_0;
        } else if strncasecmp(
            arg,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            3 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && next.offset_from(arg) == 3 as isize
        {
            (*(*curwin.get()).w_s).b_syn_conceal = false_0;
        } else {
            semsg(
                gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
                arg,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_case(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        next = skiptowhite(arg);
        if *arg as ::core::ffi::c_int == NUL {
            if (*(*curwin.get()).w_s).b_syn_ic != 0 {
                msg(
                    b"syntax case ignore\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            } else {
                msg(
                    b"syntax case match\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
        } else if strncasecmp(
            arg,
            b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            5 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && next.offset_from(arg) == 5 as isize
        {
            (*(*curwin.get()).w_s).b_syn_ic = false_0;
        } else if strncasecmp(
            arg,
            b"ignore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            6 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && next.offset_from(arg) == 6 as isize
        {
            (*(*curwin.get()).w_s).b_syn_ic = true_0;
        } else {
            semsg(
                gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
                arg,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_foldlevel(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        if *arg as ::core::ffi::c_int == NUL {
            match (*(*curwin.get()).w_s).b_syn_foldlevel {
                SYNFLD_START => {
                    msg(
                        b"syntax foldlevel start\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                    );
                }
                SYNFLD_MINIMUM => {
                    msg(
                        b"syntax foldlevel minimum\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                    );
                }
                _ => {}
            }
            return;
        }
        arg_end = skiptowhite(arg);
        if strncasecmp(
            arg,
            b"start\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            5 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && arg_end.offset_from(arg) == 5 as isize
        {
            (*(*curwin.get()).w_s).b_syn_foldlevel = SYNFLD_START;
        } else if strncasecmp(
            arg,
            b"minimum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            7 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && arg_end.offset_from(arg) == 7 as isize
        {
            (*(*curwin.get()).w_s).b_syn_foldlevel = SYNFLD_MINIMUM;
        } else {
            semsg(
                gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
                arg,
            );
            return;
        }
        arg = skipwhite(arg_end);
        if *arg as ::core::ffi::c_int != NUL {
            semsg(
                gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
                arg,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_spell(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        next = skiptowhite(arg);
        if *arg as ::core::ffi::c_int == NUL {
            if (*(*curwin.get()).w_s).b_syn_spell == SYNSPL_TOP {
                msg(
                    b"syntax spell toplevel\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            } else if (*(*curwin.get()).w_s).b_syn_spell == SYNSPL_NOTOP {
                msg(
                    b"syntax spell notoplevel\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            } else {
                msg(
                    b"syntax spell default\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
        } else if strncasecmp(
            arg,
            b"toplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            8 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && next.offset_from(arg) == 8 as isize
        {
            (*(*curwin.get()).w_s).b_syn_spell = SYNSPL_TOP;
        } else if strncasecmp(
            arg,
            b"notoplevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            10 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && next.offset_from(arg) == 10 as isize
        {
            (*(*curwin.get()).w_s).b_syn_spell = SYNSPL_NOTOP;
        } else if strncasecmp(
            arg,
            b"default\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            7 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && next.offset_from(arg) == 7 as isize
        {
            (*(*curwin.get()).w_s).b_syn_spell = SYNSPL_DEFAULT;
        } else {
            semsg(
                gettext((e_illegal_arg.ptr() as *const _) as *const ::core::ffi::c_char),
                arg,
            );
            return;
        }
        redraw_later(curwin.get(), UPD_NOT_VALID);
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_iskeyword(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut save_chartab_0: [::core::ffi::c_char; 32] = [0; 32];
        let mut save_isk: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*eap).skip != 0 {
            return;
        }
        arg = skipwhite(arg);
        if *arg as ::core::ffi::c_int == NUL {
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            if (*(*curwin.get()).w_s).b_syn_isk
                != empty_string_option.ptr() as *mut ::core::ffi::c_char
            {
                msg_puts(b"syntax iskeyword \0".as_ptr() as *const ::core::ffi::c_char);
                msg_outtrans(
                    (*(*curwin.get()).w_s).b_syn_isk,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            } else {
                msg_outtrans(
                    gettext(b"syntax iskeyword not set\0".as_ptr() as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            }
        } else if strncasecmp(
            arg,
            b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            5 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            memmove(
                &raw mut (*(*curwin.get()).w_s).b_syn_chartab as *mut uint8_t
                    as *mut ::core::ffi::c_void,
                &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
                32 as ::core::ffi::c_int as size_t,
            );
            clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
        } else {
            memmove(
                &raw mut save_chartab_0 as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
                32 as ::core::ffi::c_int as size_t,
            );
            save_isk = (*curbuf.get()).b_p_isk;
            (*curbuf.get()).b_p_isk = xstrdup(arg);
            buf_init_chartab(curbuf.get(), false);
            memmove(
                &raw mut (*(*curwin.get()).w_s).b_syn_chartab as *mut uint8_t
                    as *mut ::core::ffi::c_void,
                &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
                32 as ::core::ffi::c_int as size_t,
            );
            memmove(
                &raw mut (*curbuf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
                &raw mut save_chartab_0 as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                32 as ::core::ffi::c_int as size_t,
            );
            clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
            (*(*curwin.get()).w_s).b_syn_isk = (*curbuf.get()).b_p_isk;
            (*curbuf.get()).b_p_isk = save_isk;
        }
        redraw_later(curwin.get(), UPD_NOT_VALID);
    }
}

pub unsafe extern "C" fn syntax_clear(mut block: *mut synblock_T) {
    unsafe {
        (*block).b_syn_error = false_0 != 0;
        (*block).b_syn_slow = false_0 != 0;
        (*block).b_syn_ic = false_0;
        (*block).b_syn_foldlevel = SYNFLD_START;
        (*block).b_syn_spell = SYNSPL_DEFAULT;
        (*block).b_syn_containedin = false_0;
        (*block).b_syn_conceal = false_0;
        clear_keywtab(&raw mut (*block).b_keywtab);
        clear_keywtab(&raw mut (*block).b_keywtab_ic);
        let mut i: ::core::ffi::c_int = (*block).b_syn_patterns.ga_len;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            syn_clear_pattern(block, i);
        }
        ga_clear(&raw mut (*block).b_syn_patterns);
        let mut i_0: ::core::ffi::c_int = (*block).b_syn_clusters.ga_len;
        loop {
            i_0 -= 1;
            if i_0 < 0 as ::core::ffi::c_int {
                break;
            }
            syn_clear_cluster(block, i_0);
        }
        ga_clear(&raw mut (*block).b_syn_clusters);
        (*block).b_spell_cluster_id = 0 as ::core::ffi::c_int;
        (*block).b_nospell_cluster_id = 0 as ::core::ffi::c_int;
        (*block).b_syn_sync_flags = 0 as ::core::ffi::c_int;
        (*block).b_syn_sync_minlines = 0 as ::core::ffi::c_int as linenr_T;
        (*block).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
        (*block).b_syn_sync_linebreaks = 0 as ::core::ffi::c_int as linenr_T;
        vim_regfree((*block).b_syn_linecont_prog);
        (*block).b_syn_linecont_prog = ::core::ptr::null_mut::<regprog_T>();
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*block).b_syn_linecont_pat as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*block).b_syn_folditems = 0 as ::core::ffi::c_int;
        clear_string_option(&raw mut (*block).b_syn_isk);
        syn_stack_free_all(block);
        invalidate_current_state();
        running_syn_inc_tag.set(0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn reset_synblock(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_s != &raw mut (*(*wp).w_buffer).b_s {
            syntax_clear((*wp).w_s);
            xfree((*wp).w_s as *mut ::core::ffi::c_void);
            (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s;
        }
    }
}

pub(crate) unsafe extern "C" fn syntax_sync_clear() {
    unsafe {
        let mut i: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            if (*((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                .offset(i as isize))
            .sp_syncing
            {
                syn_remove_pattern((*curwin.get()).w_s, i);
            }
        }
        (*(*curwin.get()).w_s).b_syn_sync_flags = 0 as ::core::ffi::c_int;
        (*(*curwin.get()).w_s).b_syn_sync_minlines = 0 as ::core::ffi::c_int as linenr_T;
        (*(*curwin.get()).w_s).b_syn_sync_maxlines = 0 as ::core::ffi::c_int as linenr_T;
        (*(*curwin.get()).w_s).b_syn_sync_linebreaks = 0 as ::core::ffi::c_int as linenr_T;
        vim_regfree((*(*curwin.get()).w_s).b_syn_linecont_prog);
        (*(*curwin.get()).w_s).b_syn_linecont_prog = ::core::ptr::null_mut::<regprog_T>();
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*(*curwin.get()).w_s).b_syn_linecont_pat as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
        syn_stack_free_all((*curwin.get()).w_s);
    }
}

pub(crate) unsafe extern "C" fn syn_remove_pattern(
    mut block: *mut synblock_T,
    mut idx: ::core::ffi::c_int,
) {
    unsafe {
        let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
        spp = ((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_flags & HL_FOLD != 0 {
            (*block).b_syn_folditems -= 1;
        }
        syn_clear_pattern(block, idx);
        memmove(
            spp as *mut ::core::ffi::c_void,
            spp.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<synpat_T>().wrapping_mul(
                ((*block).b_syn_patterns.ga_len - idx - 1 as ::core::ffi::c_int) as size_t,
            ),
        );
        (*block).b_syn_patterns.ga_len -= 1;
    }
}

pub(crate) unsafe extern "C" fn syn_clear_pattern(
    mut block: *mut synblock_T,
    mut i: ::core::ffi::c_int,
) {
    unsafe {
        xfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_pattern
                as *mut ::core::ffi::c_void,
        );
        vim_regfree(
            (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize)).sp_prog,
        );
        if i == 0 as ::core::ffi::c_int
            || (*((*block).b_syn_patterns.ga_data as *mut synpat_T)
                .offset((i - 1 as ::core::ffi::c_int) as isize))
            .sp_type as ::core::ffi::c_int
                != SPTYPE_START
        {
            xfree(
                (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize))
                    .sp_cont_list as *mut ::core::ffi::c_void,
            );
            xfree(
                (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize))
                    .sp_next_list as *mut ::core::ffi::c_void,
            );
            xfree(
                (*((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(i as isize))
                    .sp_syn
                    .cont_in_list as *mut ::core::ffi::c_void,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn syn_clear_cluster(
    mut block: *mut synblock_T,
    mut i: ::core::ffi::c_int,
) {
    unsafe {
        xfree(
            (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_name
                as *mut ::core::ffi::c_void,
        );
        xfree(
            (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_name_u
                as *mut ::core::ffi::c_void,
        );
        xfree(
            (*((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize)).scl_list
                as *mut ::core::ffi::c_void,
        );
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_clear(
    mut eap: *mut exarg_T,
    mut syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut id: ::core::ffi::c_int = 0;
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }
        if (*(*curwin.get()).w_s).b_syn_topgrp != 0 as ::core::ffi::c_int {
            return;
        }
        if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
            if syncing != 0 {
                syntax_sync_clear();
            } else {
                syntax_clear((*curwin.get()).w_s);
                if (*curwin.get()).w_s == &raw mut (*(*curwin.get()).w_buffer).b_s {
                    do_unlet(
                        b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 17]>()
                            .wrapping_sub(1 as size_t),
                        true_0 != 0,
                    );
                }
                do_unlet(
                    b"w:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                    true_0 != 0,
                );
            }
        } else {
            while ends_excmd(*arg as ::core::ffi::c_int) == 0 {
                arg_end = skiptowhite(arg);
                if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                    id = syn_scl_namen2id(
                        arg.offset(1 as ::core::ffi::c_int as isize),
                        (arg_end.offset_from(arg) - 1 as isize) as ::core::ffi::c_int,
                    );
                    if id == 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(b"E391: No such syntax cluster: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            arg,
                        );
                        break;
                    } else {
                        let mut scl_id: ::core::ffi::c_int = id - SYNID_CLUSTER;
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data
                                as *mut syn_cluster_T)
                                .offset(scl_id as isize))
                            .scl_list as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL;
                        let _ = *ptr_;
                    }
                } else {
                    id = syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                    if id == 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(&raw const e_nogroup as *const ::core::ffi::c_char),
                            arg,
                        );
                        break;
                    } else {
                        syn_clear_one(id, syncing != 0);
                    }
                }
                arg = skipwhite(arg_end);
            }
        }
        redraw_curbuf_later(UPD_SOME_VALID);
        syn_stack_free_all((*curwin.get()).w_s);
    }
}

pub(crate) unsafe extern "C" fn syn_clear_one(id: ::core::ffi::c_int, syncing: bool) {
    unsafe {
        let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
        if !syncing {
            syn_clear_keyword(id, &raw mut (*(*curwin.get()).w_s).b_keywtab);
            syn_clear_keyword(id, &raw mut (*(*curwin.get()).w_s).b_keywtab_ic);
        }
        let mut idx: ::core::ffi::c_int = (*(*curwin.get()).w_s).b_syn_patterns.ga_len;
        loop {
            idx -= 1;
            if idx < 0 as ::core::ffi::c_int {
                break;
            }
            spp = ((*(*curwin.get()).w_s).b_syn_patterns.ga_data as *mut synpat_T)
                .offset(idx as isize);
            if (*spp).sp_syn.id as ::core::ffi::c_int != id
                || (*spp).sp_syncing as ::core::ffi::c_int != syncing as ::core::ffi::c_int
            {
                continue;
            }
            syn_remove_pattern((*curwin.get()).w_s, idx);
        }
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_on(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        syn_cmd_onoff(
            eap,
            b"syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_reset(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        (*eap).nextcmd = check_nextcmd((*eap).arg);
        if (*eap).skip == 0 {
            init_highlight(true_0 != 0, true_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_manual(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        syn_cmd_onoff(
            eap,
            b"manual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_off(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        syn_cmd_onoff(
            eap,
            b"nosyntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_onoff(
    mut eap: *mut exarg_T,
    mut name: *mut ::core::ffi::c_char,
) {
    unsafe {
        (*eap).nextcmd = check_nextcmd((*eap).arg);
        if (*eap).skip == 0 {
            did_syntax_onoff.set(true_0 != 0);
            let mut buf: [::core::ffi::c_char; 100] = [0; 100];
            memcpy(
                &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                b"so \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                4 as size_t,
            );
            vim_snprintf(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 100]>().wrapping_sub(3 as size_t),
                SYNTAX_FNAME.as_ptr(),
                name,
            );
            do_cmdline_cmd(&raw mut buf as *mut ::core::ffi::c_char);
        }
    }
}

pub unsafe extern "C" fn syn_maybe_enable() {
    unsafe {
        if !did_syntax_onoff.get() {
            let mut ea: exarg_T = exarg_T {
                arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                arglens: ::core::ptr::null_mut::<size_t>(),
                argc: 0,
                nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmdidx: CMD_append,
                argt: 0,
                skip: 0,
                forceit: 0,
                addr_count: 0,
                line1: 0,
                line2: 0,
                addr_type: ADDR_LINES,
                flags: 0,
                do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                do_ecmd_lnum: 0,
                append: 0,
                usefilter: 0,
                amount: 0,
                regname: 0,
                force_bin: 0,
                read_edit: 0,
                mkdir_p: 0,
                force_ff: 0,
                force_enc: 0,
                bad_char: 0,
                useridx: 0,
                errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ea_getline: None,
                cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                cstack: ::core::ptr::null_mut::<cstack_T>(),
            };
            ea.arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            ea.skip = false_0;
            syn_cmd_on(&raw mut ea, false_0);
        }
    }
}

pub(crate) static subcommands: GlobalCell<[subcommand; 19]> = GlobalCell::new([
    subcommand {
        name: b"case\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_case as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_clear as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"cluster\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_cluster as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"conceal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_conceal as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"enable\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_on as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"foldlevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(
            syn_cmd_foldlevel as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> (),
        ),
    },
    subcommand {
        name: b"include\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_include as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"iskeyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(
            syn_cmd_iskeyword as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> (),
        ),
    },
    subcommand {
        name: b"keyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_keyword as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_list as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"manual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_manual as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_match as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_on as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_off as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_region as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"reset\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_reset as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_spell as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"sync\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_sync as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
    subcommand {
        name: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        func: Some(syn_cmd_list as unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int) -> ()),
    },
]);

pub unsafe fn ex_syntax(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut subcmd_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        syn_cmdlinep.set((*eap).cmdlinep);
        subcmd_end = arg;
        while *subcmd_end as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *subcmd_end as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *subcmd_end as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *subcmd_end as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        {
            subcmd_end = subcmd_end.offset(1);
        }
        let subcmd_name: *mut ::core::ffi::c_char =
            xstrnsave(arg, subcmd_end.offset_from(arg) as size_t);
        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) += 1;
        }
        let mut i: size_t = 0;
        i = 0 as size_t;
        while i < ::core::mem::size_of::<[subcommand; 19]>()
            .wrapping_div(::core::mem::size_of::<subcommand>())
            .wrapping_div(
                (::core::mem::size_of::<[subcommand; 19]>()
                    .wrapping_rem(::core::mem::size_of::<subcommand>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            if strcmp(subcmd_name, (*subcommands.ptr())[i as usize].name) == 0 as ::core::ffi::c_int
            {
                (*eap).arg = skipwhite(subcmd_end);
                (*subcommands.ptr())[i as usize]
                    .func
                    .expect("non-null function pointer")(eap, false_0);
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if i == ::core::mem::size_of::<[subcommand; 19]>()
            .wrapping_div(::core::mem::size_of::<subcommand>())
            .wrapping_div(
                (::core::mem::size_of::<[subcommand; 19]>()
                    .wrapping_rem(::core::mem::size_of::<subcommand>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            semsg(
                gettext(b"E410: Invalid :syntax subcommand: %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                subcmd_name,
            );
        }
        xfree(subcmd_name as *mut ::core::ffi::c_void);
        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) -= 1;
        }
    }
}

pub unsafe fn ex_ownsyntax(mut eap: *mut exarg_T) {
    unsafe {
        if (*curwin.get()).w_s == &raw mut (*(*curwin.get()).w_buffer).b_s {
            (*curwin.get()).w_s =
                xcalloc(1 as size_t, ::core::mem::size_of::<synblock_T>()) as *mut synblock_T;
            hash_init(&raw mut (*(*curwin.get()).w_s).b_keywtab);
            hash_init(&raw mut (*(*curwin.get()).w_s).b_keywtab_ic);
            (*curwin.get()).w_onebuf_opt.wo_spell = false_0;
            clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spc);
            clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spf);
            clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spl);
            clear_string_option(&raw mut (*(*curwin.get()).w_s).b_p_spo);
            clear_string_option(&raw mut (*(*curwin.get()).w_s).b_syn_isk);
        }
        let mut old_value: *mut ::core::ffi::c_char =
            get_var_value(b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char);
        if !old_value.is_null() {
            old_value = xstrdup(old_value);
        }
        apply_autocmds(
            EVENT_SYNTAX,
            (*eap).arg,
            (*curbuf.get()).b_fname,
            true_0 != 0,
            curbuf.get(),
        );
        let mut new_value: *mut ::core::ffi::c_char =
            get_var_value(b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char);
        if !new_value.is_null() {
            set_internal_string_var(
                b"w:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                new_value,
            );
        }
        if old_value.is_null() {
            do_unlet(
                b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
        } else {
            set_internal_string_var(
                b"b:current_syntax\0".as_ptr() as *const ::core::ffi::c_char,
                old_value,
            );
            xfree(old_value as *mut ::core::ffi::c_void);
        };
    }
}
