//! The search patterns themselves, and everything that remembers one.
//!
//! Two patterns are live at any time — the last one searched for and the
//! last one substituted with — and both are kept in the module-private
//! `spats` pair. [`search_regcomp`] is the compiler every caller goes
//! through: it fills in the remembered pattern when handed an empty one,
//! records the new one, and applies the `'ignorecase'`/`'smartcase'` rule
//! ([`pat_has_uppercase`]). The save/restore families exist because
//! incremental search, `:substitute` and the tag code all have to run a
//! search of their own without disturbing what the user last typed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_regcomp(
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut used_pat: *mut *mut ::core::ffi::c_char,
    mut pat_save: ::core::ffi::c_int,
    mut pat_use: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut regmatch: *mut regmmatch_T,
) -> ::core::ffi::c_int {
    unsafe {
        rc_did_emsg.set(false_0 != 0);
        let mut magic: ::core::ffi::c_int = magic_isset() as ::core::ffi::c_int;
        if pat.is_null() || *pat as ::core::ffi::c_int == NUL {
            let mut i: ::core::ffi::c_int = 0;
            if pat_use == RE_LAST as ::core::ffi::c_int {
                i = last_idx.get();
            } else {
                i = pat_use;
            }
            if (*spats.ptr())[i as usize].pat.is_null() {
                if pat_use == RE_SUBST as ::core::ffi::c_int {
                    emsg(gettext(&raw const e_nopresub as *const ::core::ffi::c_char));
                } else {
                    emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
                }
                rc_did_emsg.set(true_0 != 0);
                return FAIL;
            }
            pat = (*spats.ptr())[i as usize].pat;
            patlen = (*spats.ptr())[i as usize].patlen;
            magic = (*spats.ptr())[i as usize].magic as ::core::ffi::c_int;
            no_smartcase.set((*spats.ptr())[i as usize].no_scs);
        } else if options & SEARCH_HIS as ::core::ffi::c_int != 0 {
            add_to_history(
                HIST_SEARCH as ::core::ffi::c_int,
                ::core::slice::from_raw_parts(pat as *const u8, patlen as usize),
                true_0 != 0,
                NUL as u8,
            );
        }
        if !used_pat.is_null() {
            *used_pat = pat;
        }
        xfree(mr_pattern.get() as *mut ::core::ffi::c_void);
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && *(*curwin.get()).w_onebuf_opt.wo_rlc as ::core::ffi::c_int
                == 's' as ::core::ffi::c_int
        {
            mr_pattern.set(reverse_text(pat));
        } else {
            mr_pattern.set(xstrnsave(pat, patlen));
        }
        mr_patternlen.set(patlen);
        if options & SEARCH_KEEP as ::core::ffi::c_int == 0
            && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            if pat_save == RE_SEARCH as ::core::ffi::c_int
                || pat_save == RE_BOTH as ::core::ffi::c_int
            {
                save_re_pat(RE_SEARCH as ::core::ffi::c_int, pat, patlen, magic);
            }
            if pat_save == RE_SUBST as ::core::ffi::c_int
                || pat_save == RE_BOTH as ::core::ffi::c_int
            {
                save_re_pat(RE_SUBST as ::core::ffi::c_int, pat, patlen, magic);
            }
        }
        (*regmatch).rmm_ic = ignorecase(pat);
        (*regmatch).rmm_maxcol = 0 as ::core::ffi::c_int as colnr_T;
        (*regmatch).regprog = vim_regcomp(
            pat,
            if magic != 0 {
                RE_MAGIC
            } else {
                0 as ::core::ffi::c_int
            },
        );
        if (*regmatch).regprog.is_null() {
            return FAIL;
        }
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_search_pat() -> *mut ::core::ffi::c_char {
    return mr_pattern.get();
}

pub unsafe extern "C" fn save_re_pat(
    mut idx: ::core::ffi::c_int,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut magic: ::core::ffi::c_int,
) {
    unsafe {
        if (*spats.ptr())[idx as usize].pat == pat {
            return;
        }
        free_spat((spats.ptr() as *mut SearchPattern).offset(idx as isize));
        (*spats.ptr())[idx as usize].pat = xstrnsave(pat, patlen);
        (*spats.ptr())[idx as usize].patlen = patlen;
        (*spats.ptr())[idx as usize].magic = magic != 0;
        (*spats.ptr())[idx as usize].no_scs = no_smartcase.get();
        (*spats.ptr())[idx as usize].timestamp = os_time();
        (*spats.ptr())[idx as usize].additional_data = ::core::ptr::null_mut::<AdditionalData>();
        last_idx.set(idx);
        if p_hls.get() != 0 {
            redraw_all_later(UPD_SOME_VALID);
        }
        set_no_hlsearch(false_0 != 0);
    }
}

pub unsafe extern "C" fn save_search_patterns() {
    unsafe {
        let c2rust_fresh0 = save_level.get();
        save_level.set(save_level.get() + 1);
        if c2rust_fresh0 != 0 as ::core::ffi::c_int {
            return;
        }
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[SearchPattern; 2]>()
            .wrapping_div(::core::mem::size_of::<SearchPattern>())
            .wrapping_div(
                (::core::mem::size_of::<[SearchPattern; 2]>()
                    .wrapping_rem(::core::mem::size_of::<SearchPattern>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            (*saved_spats.ptr())[i as usize] = (*spats.ptr())[i as usize];
            if !(*spats.ptr())[i as usize].pat.is_null() {
                (*saved_spats.ptr())[i as usize].pat = xstrnsave(
                    (*spats.ptr())[i as usize].pat,
                    (*spats.ptr())[i as usize].patlen,
                );
                (*saved_spats.ptr())[i as usize].patlen = (*spats.ptr())[i as usize].patlen;
            }
            i = i.wrapping_add(1);
        }
        if (*mr_pattern.ptr()).is_null() {
            saved_mr_pattern.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            saved_mr_patternlen.set(0 as size_t);
        } else {
            saved_mr_pattern.set(xstrnsave(mr_pattern.get(), mr_patternlen.get()));
            saved_mr_patternlen.set(mr_patternlen.get());
        }
        saved_spats_last_idx.set(last_idx.get());
        saved_spats_no_hlsearch.set(no_hlsearch.get());
    }
}

pub unsafe extern "C" fn restore_search_patterns() {
    unsafe {
        (*save_level.ptr()) -= 1;
        if save_level.get() != 0 as ::core::ffi::c_int {
            return;
        }
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[SearchPattern; 2]>()
            .wrapping_div(::core::mem::size_of::<SearchPattern>())
            .wrapping_div(
                (::core::mem::size_of::<[SearchPattern; 2]>()
                    .wrapping_rem(::core::mem::size_of::<SearchPattern>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            free_spat((spats.ptr() as *mut SearchPattern).offset(i as isize));
            (*spats.ptr())[i as usize] = (*saved_spats.ptr())[i as usize];
            i = i.wrapping_add(1);
        }
        set_vv_searchforward();
        xfree(mr_pattern.get() as *mut ::core::ffi::c_void);
        mr_pattern.set(saved_mr_pattern.get());
        mr_patternlen.set(saved_mr_patternlen.get());
        last_idx.set(saved_spats_last_idx.get());
        set_no_hlsearch(saved_spats_no_hlsearch.get());
    }
}

#[inline]
pub(crate) unsafe extern "C" fn free_spat(spat: *mut SearchPattern) {
    unsafe {
        xfree((*spat).pat as *mut ::core::ffi::c_void);
        xfree((*spat).additional_data as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn save_last_search_pattern() {
    unsafe {
        (*did_save_last_search_spat.ptr()) += 1;
        if did_save_last_search_spat.get() != 1 as ::core::ffi::c_int {
            return;
        }
        saved_last_search_spat.set((*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize]);
        if !(*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize]
            .pat
            .is_null()
        {
            (*saved_last_search_spat.ptr()).pat = xstrnsave(
                (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].pat,
                (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].patlen,
            );
            (*saved_last_search_spat.ptr()).patlen =
                (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].patlen;
        }
        saved_last_idx.set(last_idx.get());
        saved_no_hlsearch.set(no_hlsearch.get());
    }
}

pub unsafe extern "C" fn restore_last_search_pattern() {
    unsafe {
        (*did_save_last_search_spat.ptr()) -= 1;
        if did_save_last_search_spat.get() > 0 as ::core::ffi::c_int {
            return;
        }
        if did_save_last_search_spat.get() != 0 as ::core::ffi::c_int {
            iemsg(
                b"restore_last_search_pattern() called more often than save_last_search_pattern()\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        xfree(
            (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].pat
                as *mut ::core::ffi::c_void,
        );
        (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize] = saved_last_search_spat.get();
        (*saved_last_search_spat.ptr()).pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*saved_last_search_spat.ptr()).patlen = 0 as size_t;
        set_vv_searchforward();
        last_idx.set(saved_last_idx.get());
        set_no_hlsearch(saved_no_hlsearch.get());
    }
}

pub(crate) unsafe extern "C" fn save_incsearch_state() {
    saved_search_match_endcol.set(search_match_endcol.get());
    saved_search_match_lines.set(search_match_lines.get());
}

pub(crate) unsafe extern "C" fn restore_incsearch_state() {
    search_match_endcol.set(saved_search_match_endcol.get());
    search_match_lines.set(saved_search_match_lines.get());
}

pub unsafe extern "C" fn last_search_pattern() -> *mut ::core::ffi::c_char {
    unsafe {
        return (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].pat;
    }
}

pub unsafe extern "C" fn last_search_pattern_len() -> size_t {
    unsafe {
        return (*spats.ptr())[RE_SEARCH as ::core::ffi::c_int as usize].patlen;
    }
}

pub unsafe extern "C" fn ignorecase(mut pat: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        return ignorecase_opt(pat, p_ic.get(), p_scs.get());
    }
}

pub unsafe extern "C" fn ignorecase_opt(
    mut pat: *mut ::core::ffi::c_char,
    mut ic_in: ::core::ffi::c_int,
    mut scs: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ic: ::core::ffi::c_int = ic_in;
        if ic != 0
            && !no_smartcase.get()
            && scs != 0
            && !(ctrl_x_mode_not_default() as ::core::ffi::c_int != 0
                && (*curbuf.get()).b_p_inf != 0)
        {
            ic = !pat_has_uppercase(pat) as ::core::ffi::c_int;
        }
        no_smartcase.set(false_0 != 0);
        return ic;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pat_has_uppercase(mut pat: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = pat;
        let mut magic_val: magic_T = MAGIC_ON;
        skip_regexp_ex(
            pat,
            NUL,
            magic_isset() as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut magic_val,
        );
        while *p as ::core::ffi::c_int != NUL {
            let l: ::core::ffi::c_int = utfc_ptr2len(p);
            if l > 1 as ::core::ffi::c_int {
                if mb_isupper(utf_ptr2char(p)) {
                    return true_0 != 0;
                }
                p = p.offset(l as isize);
            } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && magic_val as ::core::ffi::c_uint
                    <= MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '_' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(3 as ::core::ffi::c_int as isize);
                } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '%' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(3 as ::core::ffi::c_int as isize);
                } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    p = p.offset(2 as ::core::ffi::c_int as isize);
                } else {
                    p = p.offset(1 as ::core::ffi::c_int as isize);
                }
            } else if (*p as ::core::ffi::c_int == '%' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int)
                && magic_val as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    p = p.offset(2 as ::core::ffi::c_int as isize);
                } else {
                    p = p.offset(1);
                }
            } else if mb_isupper(*p as uint8_t as ::core::ffi::c_int) {
                return true_0 != 0;
            } else {
                p = p.offset(1);
            }
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn last_search_pat() -> *mut ::core::ffi::c_char {
    unsafe {
        return (*spats.ptr())[last_idx.get() as usize].pat;
    }
}

pub unsafe extern "C" fn reset_search_dir() {
    unsafe {
        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir = '/' as ::core::ffi::c_char;
        set_vv_searchforward();
    }
}

pub unsafe extern "C" fn set_last_search_pat(
    mut s: *const ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
    mut magic: ::core::ffi::c_int,
    mut setlast: bool,
) {
    unsafe {
        free_spat((spats.ptr() as *mut SearchPattern).offset(idx as isize));
        if *s as ::core::ffi::c_int == NUL {
            (*spats.ptr())[idx as usize].pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*spats.ptr())[idx as usize].patlen = 0 as size_t;
        } else {
            (*spats.ptr())[idx as usize].patlen = strlen(s);
            (*spats.ptr())[idx as usize].pat = xstrnsave(s, (*spats.ptr())[idx as usize].patlen);
        }
        (*spats.ptr())[idx as usize].timestamp = os_time();
        (*spats.ptr())[idx as usize].additional_data = ::core::ptr::null_mut::<AdditionalData>();
        (*spats.ptr())[idx as usize].magic = magic != 0;
        (*spats.ptr())[idx as usize].no_scs = false_0 != 0;
        (*spats.ptr())[idx as usize].off.dir = '/' as ::core::ffi::c_char;
        set_vv_searchforward();
        (*spats.ptr())[idx as usize].off.line = false_0 != 0;
        (*spats.ptr())[idx as usize].off.end = false_0 != 0;
        (*spats.ptr())[idx as usize].off.off = 0 as int64_t;
        if setlast {
            last_idx.set(idx);
        }
        if save_level.get() != 0 {
            free_spat((saved_spats.ptr() as *mut SearchPattern).offset(idx as isize));
            (*saved_spats.ptr())[idx as usize] = (*spats.ptr())[0 as ::core::ffi::c_int as usize];
            if (*spats.ptr())[idx as usize].pat.is_null() {
                (*saved_spats.ptr())[idx as usize].pat =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                (*saved_spats.ptr())[idx as usize].patlen = 0 as size_t;
            } else {
                (*saved_spats.ptr())[idx as usize].pat = xstrnsave(
                    (*spats.ptr())[idx as usize].pat,
                    (*spats.ptr())[idx as usize].patlen,
                );
                (*saved_spats.ptr())[idx as usize].patlen = (*spats.ptr())[idx as usize].patlen;
            }
            saved_spats_last_idx.set(last_idx.get());
        }
        if p_hls.get() != 0 && idx == last_idx.get() && !no_hlsearch.get() {
            redraw_all_later(UPD_SOME_VALID);
        }
    }
}

pub unsafe extern "C" fn last_pat_prog(mut regmatch: *mut regmmatch_T) {
    unsafe {
        if (*spats.ptr())[last_idx.get() as usize].pat.is_null() {
            (*regmatch).regprog = ::core::ptr::null_mut::<regprog_T>();
            return;
        }
        (*emsg_off.ptr()) += 1;
        search_regcomp(
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            0 as size_t,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            last_idx.get(),
            SEARCH_KEEP as ::core::ffi::c_int,
            regmatch,
        );
        (*emsg_off.ptr()) -= 1;
    }
}

pub unsafe extern "C" fn set_search_direction(mut cdir: ::core::ffi::c_int) {
    unsafe {
        (*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir = cdir as ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn set_vv_searchforward() {
    unsafe {
        set_vim_var_nr(
            VV_SEARCHFORWARD,
            ((*spats.ptr())[0 as ::core::ffi::c_int as usize].off.dir as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int) as ::core::ffi::c_int as varnumber_T,
        );
    }
}

pub(crate) unsafe extern "C" fn is_zero_width(
    mut pattern: *mut ::core::ffi::c_char,
    mut patternlen: size_t,
    mut move_0: bool,
    mut cur: *mut pos_T,
    mut direction: Direction,
) -> ::core::ffi::c_int {
    unsafe {
        let mut regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        let mut result: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
        let mut flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if pattern.is_null() {
            pattern = (*spats.ptr())[last_idx.get() as usize].pat;
            patternlen = (*spats.ptr())[last_idx.get() as usize].patlen;
        }
        if search_regcomp(
            pattern,
            patternlen,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            RE_SEARCH as ::core::ffi::c_int,
            RE_SEARCH as ::core::ffi::c_int,
            SEARCH_KEEP as ::core::ffi::c_int,
            &raw mut regmatch,
        ) == FAIL
        {
            return -1 as ::core::ffi::c_int;
        }
        regmatch.startpos[0 as ::core::ffi::c_int as usize].col =
            -1 as ::core::ffi::c_int as colnr_T;
        if move_0 {
            clearpos(&mut pos);
        } else {
            pos = *cur;
            flag = SEARCH_START as ::core::ffi::c_int;
        }
        if searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut pos,
            ::core::ptr::null_mut::<pos_T>(),
            direction,
            pattern,
            patternlen,
            1 as ::core::ffi::c_int,
            SEARCH_KEEP as ::core::ffi::c_int + flag,
            RE_SEARCH as ::core::ffi::c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        ) != FAIL
        {
            let mut nmatched: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            loop {
                regmatch.startpos[0 as ::core::ffi::c_int as usize].col += 1;
                nmatched = vim_regexec_multi(
                    &raw mut regmatch,
                    curwin.get(),
                    curbuf.get(),
                    pos.lnum,
                    regmatch.startpos[0 as ::core::ffi::c_int as usize].col,
                    ::core::ptr::null_mut::<proftime_T>(),
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                );
                if nmatched != 0 as ::core::ffi::c_int {
                    break;
                }
                if !(!regmatch.regprog.is_null()
                    && (if direction as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                        (regmatch.startpos[0 as ::core::ffi::c_int as usize].col < pos.col)
                            as ::core::ffi::c_int
                    } else {
                        (regmatch.startpos[0 as ::core::ffi::c_int as usize].col > pos.col)
                            as ::core::ffi::c_int
                    }) != 0)
                {
                    break;
                }
            }
            if called_emsg.get() == called_emsg_before {
                result = (nmatched != 0 as ::core::ffi::c_int
                    && regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                        == regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum
                    && regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                        == regmatch.endpos[0 as ::core::ffi::c_int as usize].col)
                    as ::core::ffi::c_int;
            }
        }
        vim_regfree(regmatch.regprog);
        return result;
    }
}

pub unsafe extern "C" fn get_search_pattern(pat: *mut SearchPattern) {
    unsafe {
        memcpy(
            pat as *mut ::core::ffi::c_void,
            (spats.ptr() as *mut SearchPattern).offset(0 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<SearchPattern>(),
        );
    }
}

pub unsafe extern "C" fn get_substitute_pattern(pat: *mut SearchPattern) {
    unsafe {
        memcpy(
            pat as *mut ::core::ffi::c_void,
            (spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<SearchPattern>(),
        );
        memset(
            &raw mut (*pat).off as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<SearchOffset>(),
        );
    }
}

pub unsafe extern "C" fn set_search_pattern(pat: SearchPattern) {
    unsafe {
        free_spat((spats.ptr() as *mut SearchPattern).offset(0 as ::core::ffi::c_int as isize));
        memcpy(
            (spats.ptr() as *mut SearchPattern).offset(0 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            &raw const pat as *const ::core::ffi::c_void,
            ::core::mem::size_of::<SearchPattern>(),
        );
        set_vv_searchforward();
    }
}

pub unsafe extern "C" fn set_substitute_pattern(pat: SearchPattern) {
    unsafe {
        free_spat((spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize));
        memcpy(
            (spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            &raw const pat as *const ::core::ffi::c_void,
            ::core::mem::size_of::<SearchPattern>(),
        );
        memset(
            &raw mut (*(spats.ptr() as *mut SearchPattern).offset(1 as ::core::ffi::c_int as isize))
                .off as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<SearchOffset>(),
        );
    }
}

pub unsafe extern "C" fn set_last_used_pattern(is_substitute_pattern: bool) {
    last_idx.set(if is_substitute_pattern as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    });
}

pub unsafe extern "C" fn search_was_last_used() -> bool {
    return last_idx.get() == 0 as ::core::ffi::c_int;
}
