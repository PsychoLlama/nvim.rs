//! The public query API, and command-line completion.
//!
//! `synID()`, `synstack()`, `synIDattr()`, `foldlevel()` for
//! `'foldmethod'=syntax` and the `:syntax`/`:echohl` completions all answer from
//! here. Everything in this module reads state the rest of the family produced;
//! nothing here parses.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn syntax_present(mut win: *mut win_T) -> bool {
    unsafe {
        return (*(*win).w_s).b_syn_patterns.ga_len != 0 as ::core::ffi::c_int
            || (*(*win).w_s).b_syn_clusters.ga_len != 0 as ::core::ffi::c_int
            || (*(*win).w_s).b_keywtab.ht_used > 0 as size_t
            || (*(*win).w_s).b_keywtab_ic.ht_used > 0 as size_t;
    }
}

pub(crate) static expand_what: GlobalCell<C2Rust_Unnamed_24> = GlobalCell::new(EXP_SUBCMD);

pub unsafe extern "C" fn reset_expand_highlight() {
    include_none.set(0 as ::core::ffi::c_int);
    include_default.set(include_none.get());
    include_link.set(include_default.get());
}

pub unsafe extern "C" fn set_context_in_echohl_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    unsafe {
        (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        include_none.set(1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn set_context_in_syntax_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    unsafe {
        (*xp).xp_context = EXPAND_SYNTAX as ::core::ffi::c_int;
        expand_what.set(EXP_SUBCMD);
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        include_link.set(0 as ::core::ffi::c_int);
        include_default.set(0 as ::core::ffi::c_int);
        if *arg as ::core::ffi::c_int == NUL {
            return;
        }
        let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
        if *p as ::core::ffi::c_int == NUL {
            return;
        }
        (*xp).xp_pattern = skipwhite(p);
        if *skiptowhite((*xp).xp_pattern) as ::core::ffi::c_int != NUL {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        } else if strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"case\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            expand_what.set(EXP_CASE);
        } else if strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            expand_what.set(EXP_SPELL);
        } else if strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"sync\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            expand_what.set(EXP_SYNC);
        } else if strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = skipwhite(p);
            if *p as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                expand_what.set(EXP_CLUSTER);
            } else {
                (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
            }
        } else if strncasecmp(
            arg as *mut ::core::ffi::c_char,
            b"keyword\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            || strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
        } else {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        };
    }
}

pub unsafe extern "C" fn get_syntax_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        match expand_what.get() as ::core::ffi::c_uint {
            0 => {
                if idx < 0 as ::core::ffi::c_int || idx >= SUBCOMMANDS.len() as ::core::ffi::c_int {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                return SUBCOMMANDS[idx as usize].name.as_ptr().cast_mut();
            }
            1 => {
                static case_args: GlobalCell<[*mut ::core::ffi::c_char; 3]> = GlobalCell::new([
                    b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"ignore\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ]);
                return (*case_args.ptr())[idx as usize];
            }
            2 => {
                static spell_args: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
                    b"toplevel\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"notoplevel\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"default\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ]);
                return (*spell_args.ptr())[idx as usize];
            }
            3 => {
                static sync_args: GlobalCell<[*mut ::core::ffi::c_char; 11]> = GlobalCell::new([
                    b"ccomment\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"fromstart\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"linebreaks=\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"linecont\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"lines=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"maxlines=\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"minlines=\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"region\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ]);
                return (*sync_args.ptr())[idx as usize];
            }
            4 => {
                if idx < (*(*curwin.get()).w_s).b_syn_clusters.ga_len {
                    vim_snprintf(
                        &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char,
                        EXPAND_BUF_LEN as ::core::ffi::c_int as size_t,
                        b"@%s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*((*(*curwin.get()).w_s).b_syn_clusters.ga_data as *mut syn_cluster_T)
                            .offset(idx as isize))
                        .scl_name,
                    );
                    return &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char;
                } else {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
            _ => {}
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn syn_get_id(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut trans: ::core::ffi::c_int,
    mut spellp: *mut bool,
    mut keep_state: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if wp != syn_win.get()
            || (*wp).w_buffer != syn_buf.get()
            || lnum != current_lnum.get()
            || col < current_col.get()
        {
            syntax_start(wp, lnum);
        } else if col > current_col.get() {
            next_match_idx.set(-1 as ::core::ffi::c_int);
        }
        get_syntax_attr(col, spellp, keep_state != 0);
        return if trans != 0 {
            current_trans_id.get()
        } else {
            current_id.get()
        };
    }
}

pub unsafe extern "C" fn get_syntax_info(
    mut seqnrp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        *seqnrp = current_seqnr.get();
        return current_flags.get();
    }
}

pub unsafe extern "C" fn syn_get_sub_char() -> ::core::ffi::c_int {
    return current_sub_char.get();
}

pub unsafe extern "C" fn syn_get_stack_item(mut i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if i >= (*current_state.ptr()).ga_len {
            invalidate_current_state();
            current_col.set(MAXCOL as ::core::ffi::c_int as colnr_T);
            return -1 as ::core::ffi::c_int;
        }
        return (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_id;
    }
}

pub(crate) unsafe extern "C" fn syn_cur_foldlevel() -> ::core::ffi::c_int {
    unsafe {
        let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*current_state.ptr()).ga_len {
            if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_flags
                & HL_FOLD
                != 0
            {
                level += 1;
            }
            i += 1;
        }
        return level;
    }
}

pub unsafe extern "C" fn syn_get_foldlevel(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*(*wp).w_s).b_syn_folditems != 0 as ::core::ffi::c_int
            && !(*(*wp).w_s).b_syn_error
            && !(*(*wp).w_s).b_syn_slow
        {
            syntax_start(wp, lnum);
            level = syn_cur_foldlevel();
            if (*(*wp).w_s).b_syn_foldlevel == SYNFLD_MINIMUM {
                let mut cur_level: ::core::ffi::c_int = level;
                let mut low_level: ::core::ffi::c_int = cur_level;
                while !current_finished.get() {
                    syn_current_attr(
                        false_0 != 0,
                        false_0 != 0,
                        ::core::ptr::null_mut::<bool>(),
                        false_0 != 0,
                    );
                    cur_level = syn_cur_foldlevel();
                    if cur_level < low_level {
                        low_level = cur_level;
                    } else if cur_level > low_level {
                        level = low_level;
                    }
                    (*current_col.ptr()) += 1;
                }
            }
        }
        if level as OptInt > (*wp).w_onebuf_opt.wo_fdn {
            level = (*wp).w_onebuf_opt.wo_fdn as ::core::ffi::c_int;
            if level < 0 as ::core::ffi::c_int {
                level = 0 as ::core::ffi::c_int;
            }
        }
        return level;
    }
}
