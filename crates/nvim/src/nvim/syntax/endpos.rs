//! Where a region ends, and the low-level matching primitives.
//!
//! [`find_endpos`] is the search for a region's end: the first END pattern that
//! matches after the START, with any SKIP pattern's matches stepped over, and
//! the `matchgroup=` end highlight worked out. Around it sit the primitives the
//! rest of the state machine matches with -- [`syn_regexec`] (a timed
//! `vim_regexec_multi`), [`check_keyword_id`] (the keyword hash lookup), and
//! [`syn_add_start_off`]/[`syn_add_end_off`], which apply the seven `ms=`/`me=`/
//! `hs=`/`he=`/`rs=`/`re=`/`lc=` offsets to a match.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn find_endpos(
    mut idx: ::core::ffi::c_int,
    mut startpos: *mut lpos_T,
    mut m_endpos: *mut lpos_T,
    mut hl_endpos: *mut lpos_T,
    mut flagsp: *mut ::core::ffi::c_int,
    mut end_endpos: *mut lpos_T,
    mut end_idx: *mut ::core::ffi::c_int,
    mut start_ext: *mut reg_extmatch_T,
) {
    unsafe {
        let mut spp_skip: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
        let mut best_idx: ::core::ffi::c_int = 0;
        let mut regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        let mut best_regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        let mut pos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut had_match: bool = false_0 != 0;
        let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
        if idx < 0 as ::core::ffi::c_int {
            return;
        }
        let mut spp: *mut synpat_T =
            ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
        if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_START {
            *hl_endpos = *startpos;
            return;
        }
        loop {
            spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize);
            if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_START {
                break;
            }
            idx += 1;
        }
        if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_SKIP {
            spp_skip = spp;
            idx += 1;
        } else {
            spp_skip = ::core::ptr::null_mut::<synpat_T>();
        }
        unref_extmatch(re_extmatch_in.get());
        re_extmatch_in.set(ref_extmatch(start_ext));
        let mut matchcol: colnr_T = (*startpos).col;
        let mut start_idx: ::core::ffi::c_int = idx;
        best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col =
            0 as ::core::ffi::c_int as colnr_T;
        save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
        loop {
            best_idx = -1 as ::core::ffi::c_int;
            idx = start_idx;
            while idx < (*syn_block.get()).b_syn_patterns.ga_len {
                let mut lc_col: ::core::ffi::c_int = matchcol as ::core::ffi::c_int;
                spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                    .offset(idx as isize);
                if (*spp).sp_type as ::core::ffi::c_int != SPTYPE_END {
                    break;
                }
                lc_col -= (*spp).sp_offsets[SPO_LC_OFF as usize];
                if lc_col < 0 as ::core::ffi::c_int {
                    lc_col = 0 as ::core::ffi::c_int;
                }
                regmatch.rmm_ic = (*spp).sp_ic;
                regmatch.regprog = (*spp).sp_prog;
                let mut r: bool = syn_regexec(
                    &raw mut regmatch,
                    (*startpos).lnum,
                    lc_col as colnr_T,
                    &raw mut (*spp).sp_time,
                );
                (*spp).sp_prog = regmatch.regprog;
                if r {
                    if best_idx == -1 as ::core::ffi::c_int
                        || regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                            < best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                    {
                        best_idx = idx;
                        best_regmatch.startpos[0 as ::core::ffi::c_int as usize] =
                            regmatch.startpos[0 as ::core::ffi::c_int as usize];
                        best_regmatch.endpos[0 as ::core::ffi::c_int as usize] =
                            regmatch.endpos[0 as ::core::ffi::c_int as usize];
                    }
                }
                idx += 1;
            }
            if best_idx == -1 as ::core::ffi::c_int {
                break;
            }
            if !spp_skip.is_null() {
                let mut lc_col_0: ::core::ffi::c_int =
                    matchcol as ::core::ffi::c_int - (*spp_skip).sp_offsets[SPO_LC_OFF as usize];
                if lc_col_0 < 0 as ::core::ffi::c_int {
                    lc_col_0 = 0 as ::core::ffi::c_int;
                }
                regmatch.rmm_ic = (*spp_skip).sp_ic;
                regmatch.regprog = (*spp_skip).sp_prog;
                let mut r_0: ::core::ffi::c_int = syn_regexec(
                    &raw mut regmatch,
                    (*startpos).lnum,
                    lc_col_0 as colnr_T,
                    &raw mut (*spp_skip).sp_time,
                ) as ::core::ffi::c_int;
                (*spp_skip).sp_prog = regmatch.regprog;
                if r_0 != 0
                    && regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                        <= best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col
                {
                    syn_add_end_off(
                        &raw mut pos,
                        &raw mut regmatch,
                        spp_skip,
                        SPO_ME_OFF,
                        1 as ::core::ffi::c_int,
                    );
                    if pos.lnum > (*startpos).lnum {
                        break;
                    }
                    let mut line_len: ::core::ffi::c_int =
                        ml_get_buf_len(syn_buf.get(), (*startpos).lnum);
                    if pos.col <= matchcol {
                        matchcol += 1;
                    } else if pos.col <= regmatch.endpos[0 as ::core::ffi::c_int as usize].col {
                        matchcol = pos.col;
                    } else {
                        matchcol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                        while matchcol < line_len && matchcol < pos.col {
                            matchcol += 1;
                        }
                    }
                    if matchcol >= line_len {
                        break;
                    } else {
                        continue;
                    }
                }
            }
            spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                .offset(best_idx as isize);
            syn_add_end_off(
                m_endpos,
                &raw mut best_regmatch,
                spp,
                SPO_ME_OFF,
                1 as ::core::ffi::c_int,
            );
            if (*m_endpos).lnum == (*startpos).lnum && (*m_endpos).col < (*startpos).col {
                (*m_endpos).col = (*startpos).col;
            }
            syn_add_end_off(
                end_endpos,
                &raw mut best_regmatch,
                spp,
                SPO_HE_OFF,
                1 as ::core::ffi::c_int,
            );
            if (*end_endpos).lnum == (*startpos).lnum && (*end_endpos).col < (*startpos).col {
                (*end_endpos).col = (*startpos).col;
            }
            limit_pos(end_endpos, m_endpos);
            if (*spp).sp_syn_match_id as ::core::ffi::c_int
                != (*spp).sp_syn.id as ::core::ffi::c_int
                && (*spp).sp_syn_match_id as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            {
                *end_idx = best_idx;
                if (*spp).sp_off_flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << SPO_RE_OFF + SPO_COUNT
                    != 0
                {
                    (*hl_endpos).lnum = best_regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum;
                    (*hl_endpos).col = best_regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                } else {
                    (*hl_endpos).lnum =
                        best_regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    (*hl_endpos).col = best_regmatch.startpos[0 as ::core::ffi::c_int as usize].col;
                }
                (*hl_endpos).col += (*spp).sp_offsets[SPO_RE_OFF as usize];
                if (*hl_endpos).lnum == (*startpos).lnum && (*hl_endpos).col < (*startpos).col {
                    (*hl_endpos).col = (*startpos).col;
                }
                limit_pos(hl_endpos, m_endpos);
                *m_endpos = *hl_endpos;
            } else {
                *end_idx = 0 as ::core::ffi::c_int;
                *hl_endpos = *end_endpos;
            }
            *flagsp = (*spp).sp_flags;
            had_match = true_0 != 0;
            break;
        }
        if !had_match {
            (*m_endpos).lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
        unref_extmatch(re_extmatch_in.get());
        re_extmatch_in.set(::core::ptr::null_mut::<reg_extmatch_T>());
    }
}

pub(crate) unsafe extern "C" fn limit_pos(mut pos: *mut lpos_T, mut limit: *mut lpos_T) {
    unsafe {
        if (*pos).lnum > (*limit).lnum {
            *pos = *limit;
        } else if (*pos).lnum == (*limit).lnum && (*pos).col > (*limit).col {
            (*pos).col = (*limit).col;
        }
    }
}

pub(crate) unsafe extern "C" fn limit_pos_zero(mut pos: *mut lpos_T, mut limit: *mut lpos_T) {
    unsafe {
        if (*pos).lnum == 0 as linenr_T {
            *pos = *limit;
        } else {
            limit_pos(pos, limit);
        };
    }
}

pub(crate) unsafe extern "C" fn syn_add_end_off(
    mut result: *mut lpos_T,
    mut regmatch: *mut regmmatch_T,
    mut spp: *mut synpat_T,
    mut idx: ::core::ffi::c_int,
    mut extra: ::core::ffi::c_int,
) {
    unsafe {
        let mut col: ::core::ffi::c_int = 0;
        let mut off: ::core::ffi::c_int = 0;
        let mut base: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*spp).sp_off_flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << idx != 0 {
            (*result).lnum = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum;
            col = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
            off = (*spp).sp_offsets[idx as usize] + extra;
        } else {
            (*result).lnum = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum;
            col = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
            off = (*spp).sp_offsets[idx as usize];
        }
        if (*result).lnum > (*syn_buf.get()).b_ml.ml_line_count {
            col = 0 as ::core::ffi::c_int;
        } else if off != 0 as ::core::ffi::c_int {
            base = ml_get_buf(syn_buf.get(), (*result).lnum);
            p = base.offset(col as isize);
            if off > 0 as ::core::ffi::c_int {
                loop {
                    let c2rust_fresh1 = off;
                    off = off - 1;
                    if !(c2rust_fresh1 > 0 as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL)
                    {
                        break;
                    }
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
            } else {
                loop {
                    let c2rust_fresh2 = off;
                    off = off + 1;
                    if !(c2rust_fresh2 < 0 as ::core::ffi::c_int && base < p) {
                        break;
                    }
                    p = p.offset(
                        -((utf_head_off(base, p.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                }
            }
            col = p.offset_from(base) as ::core::ffi::c_int;
        }
        (*result).col = col as colnr_T;
    }
}

pub(crate) unsafe extern "C" fn syn_add_start_off(
    mut result: *mut lpos_T,
    mut regmatch: *mut regmmatch_T,
    mut spp: *mut synpat_T,
    mut idx: ::core::ffi::c_int,
    mut extra: ::core::ffi::c_int,
) {
    unsafe {
        let mut col: ::core::ffi::c_int = 0;
        let mut off: ::core::ffi::c_int = 0;
        let mut base: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*spp).sp_off_flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << idx + SPO_COUNT
            != 0
        {
            (*result).lnum = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].lnum;
            col = (*regmatch).endpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
            off = (*spp).sp_offsets[idx as usize] + extra;
        } else {
            (*result).lnum = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].lnum;
            col = (*regmatch).startpos[0 as ::core::ffi::c_int as usize].col as ::core::ffi::c_int;
            off = (*spp).sp_offsets[idx as usize];
        }
        if (*result).lnum > (*syn_buf.get()).b_ml.ml_line_count {
            (*result).lnum = (*syn_buf.get()).b_ml.ml_line_count;
            col = ml_get_buf_len(syn_buf.get(), (*result).lnum) as ::core::ffi::c_int;
        }
        if off != 0 as ::core::ffi::c_int {
            base = ml_get_buf(syn_buf.get(), (*result).lnum);
            p = base.offset(col as isize);
            if off > 0 as ::core::ffi::c_int {
                loop {
                    let c2rust_fresh6 = off;
                    off = off - 1;
                    if !(c2rust_fresh6 != 0 && *p as ::core::ffi::c_int != NUL) {
                        break;
                    }
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
            } else {
                loop {
                    let c2rust_fresh7 = off;
                    off = off + 1;
                    if !(c2rust_fresh7 != 0 && base < p) {
                        break;
                    }
                    p = p.offset(
                        -((utf_head_off(base, p.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                }
            }
            col = p.offset_from(base) as ::core::ffi::c_int;
        }
        (*result).col = col as colnr_T;
    }
}

pub(crate) unsafe extern "C" fn syn_getcurline() -> *mut ::core::ffi::c_char {
    unsafe {
        return ml_get_buf(syn_buf.get(), current_lnum.get());
    }
}

pub(crate) unsafe extern "C" fn syn_getcurline_len() -> colnr_T {
    unsafe {
        return ml_get_buf_len(syn_buf.get(), current_lnum.get());
    }
}

pub(crate) unsafe extern "C" fn syn_regexec(
    mut rmp: *mut regmmatch_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut st: *mut syn_time_T,
) -> bool {
    unsafe {
        let mut timed_out: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut pt: proftime_T = 0;
        let l_syn_time_on: bool = syn_time_on.get();
        if l_syn_time_on {
            pt = profile_start();
        }
        if (*rmp).regprog.is_null() {
            return false_0 != 0;
        }
        (*rmp).rmm_maxcol = (*syn_buf.get()).b_p_smc as colnr_T;
        let mut r: ::core::ffi::c_int = vim_regexec_multi(
            rmp,
            syn_win.get(),
            syn_buf.get(),
            lnum,
            col,
            syn_tm.get(),
            &raw mut timed_out,
        );
        if l_syn_time_on {
            pt = profile_end(pt);
            (*st).total = profile_add((*st).total, pt);
            if profile_cmp(pt, (*st).slowest) < 0 as ::core::ffi::c_int {
                (*st).slowest = pt;
            }
            (*st).count += 1;
            if r > 0 as ::core::ffi::c_int {
                (*st).match_0 += 1;
            }
        }
        if timed_out != 0 && !(*(*syn_win.get()).w_s).b_syn_slow {
            (*(*syn_win.get()).w_s).b_syn_slow = true_0 != 0;
            msg(
                gettext(
                    b"'redrawtime' exceeded, syntax highlighting disabled\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                0 as ::core::ffi::c_int,
            );
        }
        if r > 0 as ::core::ffi::c_int {
            (*rmp).startpos[0 as ::core::ffi::c_int as usize].lnum += lnum;
            (*rmp).endpos[0 as ::core::ffi::c_int as usize].lnum += lnum;
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn check_keyword_id(
    line: *mut ::core::ffi::c_char,
    startcol: ::core::ffi::c_int,
    endcolp: *mut ::core::ffi::c_int,
    flagsp: *mut ::core::ffi::c_int,
    next_listp: *mut *mut int16_t,
    cur_si: *mut stateitem_T,
    ccharp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let kwp: *mut ::core::ffi::c_char = line.offset(startcol as isize);
        let mut kwlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            kwlen += utfc_ptr2len(kwp.offset(kwlen as isize));
            if !vim_iswordp_buf(kwp.offset(kwlen as isize), syn_buf.get()) {
                break;
            }
        }
        if kwlen > MAXKEYWLEN {
            return 0 as ::core::ffi::c_int;
        }
        let mut keyword: [::core::ffi::c_char; 81] = [0; 81];
        xmemcpyz(
            &raw mut keyword as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            kwp as *const ::core::ffi::c_void,
            kwlen as size_t,
        );
        let mut kp: *mut keyentry_T = ::core::ptr::null_mut::<keyentry_T>();
        if (*syn_block.get()).b_keywtab.ht_used != 0 as size_t {
            kp = match_keyword(
                &raw mut keyword as *mut ::core::ffi::c_char,
                &raw mut (*syn_block.get()).b_keywtab,
                cur_si,
            );
        }
        if kp.is_null() && (*syn_block.get()).b_keywtab_ic.ht_used != 0 as size_t {
            str_foldcase(
                kwp,
                kwlen,
                &raw mut keyword as *mut ::core::ffi::c_char,
                MAXKEYWLEN + 1 as ::core::ffi::c_int,
            );
            kp = match_keyword(
                &raw mut keyword as *mut ::core::ffi::c_char,
                &raw mut (*syn_block.get()).b_keywtab_ic,
                cur_si,
            );
        }
        if !kp.is_null() {
            *endcolp = startcol + kwlen;
            *flagsp = (*kp).flags;
            *next_listp = (*kp).next_list;
            *ccharp = (*kp).k_char;
            return (*kp).k_syn.id as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn match_keyword(
    mut keyword: *mut ::core::ffi::c_char,
    mut ht: *mut hashtab_T,
    mut cur_si: *mut stateitem_T,
) -> *mut keyentry_T {
    unsafe {
        let mut hi: *mut hashitem_T = hash_find(ht, keyword);
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            let mut kp: *mut keyentry_T = (*hi).hi_key.offset(
                -((&raw mut (*dumkey.ptr()).keyword as *mut ::core::ffi::c_char)
                    .offset_from(dumkey.ptr() as *mut ::core::ffi::c_char)
                    as isize),
            ) as *mut keyentry_T;
            while !kp.is_null() {
                if if !(*current_next_list.ptr()).is_null() {
                    in_id_list(
                        ::core::ptr::null_mut::<stateitem_T>(),
                        current_next_list.get(),
                        &raw mut (*kp).k_syn,
                        0 as ::core::ffi::c_int,
                    )
                } else if cur_si.is_null() {
                    ((*kp).flags & HL_CONTAINED == 0) as ::core::ffi::c_int
                } else {
                    in_id_list(
                        cur_si,
                        (*cur_si).si_cont_list,
                        &raw mut (*kp).k_syn,
                        (*kp).flags,
                    )
                } != 0
                {
                    return kp;
                }
                kp = (*kp).ke_next;
            }
        }
        return ::core::ptr::null_mut::<keyentry_T>();
    }
}
