//! The per-cell attribute lookup — `syn_current_attr`.
//!
//! [`get_syntax_attr`] moves the state machine to a column and answers the
//! highlight attribute for it; [`syn_current_attr`] is the step that does the
//! work. It repeatedly looks for a keyword and then for a pattern that can match
//! at the current column and is admitted by the containment rules, pushes what
//! it finds onto the state stack, and finally walks the stack down to the
//! innermost item whose highlight range covers the column.
//!
//! This is the hottest path in the module: it runs once per cell of every
//! highlighted line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_syntax_attr(
    col: colnr_T,
    can_spell: *mut bool,
    keep_state: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !can_spell.is_null() {
            *can_spell = if (*syn_block.get()).b_syn_spell == SYNSPL_DEFAULT {
                ((*syn_block.get()).b_spell_cluster_id == 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
            } else {
                ((*syn_block.get()).b_syn_spell == SYNSPL_TOP) as ::core::ffi::c_int
            } != 0;
        }
        if (*syn_block.get()).b_sst_array.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*syn_buf.get()).b_p_smc > 0 as OptInt && col >= (*syn_buf.get()).b_p_smc as colnr_T {
            clear_current_state();
            current_id.set(0 as ::core::ffi::c_int);
            current_trans_id.set(0 as ::core::ffi::c_int);
            current_flags.set(0 as ::core::ffi::c_int);
            current_seqnr.set(0 as ::core::ffi::c_int);
            return 0 as ::core::ffi::c_int;
        }
        if (*current_state.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
            validate_current_state();
        }
        while current_col.get() <= col {
            attr = syn_current_attr(
                false_0 != 0,
                true_0 != 0,
                can_spell,
                if current_col.get() == col {
                    keep_state as ::core::ffi::c_int
                } else {
                    false_0
                } != 0,
            );
            (*current_col.ptr()) += 1;
        }
        return attr;
    }
}

pub(crate) unsafe extern "C" fn syn_current_attr(
    syncing: bool,
    displaying: bool,
    can_spell: *mut bool,
    keep_state: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut hl_startpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut hl_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut eos_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut eoe_pos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut end_idx: ::core::ffi::c_int = 0;
        let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        let mut sip: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        let mut startcol: ::core::ffi::c_int = 0;
        let mut endcol: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0;
        let mut cchar: ::core::ffi::c_int = 0;
        let mut next_list: *mut int16_t = ::core::ptr::null_mut::<int16_t>();
        let mut found_match: bool = false;
        static try_next_column: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        let mut pos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut cur_extmatch: *mut reg_extmatch_T = ::core::ptr::null_mut::<reg_extmatch_T>();
        let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
        let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut keep_next_list: bool = false;
        let mut zero_width_next_list: bool = false_0 != 0;
        let mut zero_width_next_ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        line = syn_getcurline();
        if *line.offset(current_col.get() as isize) as ::core::ffi::c_int == NUL
            && current_col.get() != 0 as ::core::ffi::c_int
        {
            if next_match_idx.get() >= 0 as ::core::ffi::c_int
                && next_match_col.get() >= current_col.get()
                && next_match_col.get() != MAXCOL as ::core::ffi::c_int
            {
                push_next_match();
            }
            current_finished.set(true_0 != 0);
            current_state_stored.set(false_0 != 0);
            return 0 as ::core::ffi::c_int;
        }
        if *line.offset(current_col.get() as isize) as ::core::ffi::c_int == NUL
            || *line.offset(
                (current_col.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int
                == NUL
        {
            current_finished.set(true_0 != 0);
            current_state_stored.set(false_0 != 0);
        }
        if try_next_column.get() {
            next_match_idx.set(-1 as ::core::ffi::c_int);
            try_next_column.set(false_0 != 0);
        }
        let do_keywords: bool = !syncing
            && ((*syn_block.get()).b_keywtab.ht_used > 0 as size_t
                || (*syn_block.get()).b_keywtab_ic.ht_used > 0 as size_t);
        ga_init(
            &raw mut zero_width_next_ga,
            ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
        loop {
            found_match = false_0 != 0;
            keep_next_list = false_0 != 0;
            let mut syn_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if (*current_state.ptr()).ga_len != 0 {
                cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
                    .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
            } else {
                cur_si = ::core::ptr::null_mut::<stateitem_T>();
            }
            if (*syn_block.get()).b_syn_containedin != 0
                || cur_si.is_null()
                || !(*cur_si).si_cont_list.is_null()
            {
                if do_keywords {
                    line = syn_getcurline();
                    let mut cur_pos: *const ::core::ffi::c_char =
                        line.offset(current_col.get() as isize);
                    if vim_iswordp_buf(cur_pos, syn_buf.get()) as ::core::ffi::c_int != 0
                        && (current_col.get() == 0 as ::core::ffi::c_int
                            || !vim_iswordp_buf(
                                cur_pos.offset(-(1 as ::core::ffi::c_int as isize)).offset(
                                    -(utf_head_off(
                                        line,
                                        cur_pos.offset(-(1 as ::core::ffi::c_int as isize)),
                                    ) as isize),
                                ),
                                syn_buf.get(),
                            ))
                    {
                        syn_id = check_keyword_id(
                            line,
                            current_col.get(),
                            &raw mut endcol,
                            &raw mut flags,
                            &raw mut next_list,
                            cur_si,
                            &raw mut cchar,
                        );
                        if syn_id != 0 as ::core::ffi::c_int {
                            push_current_state(KEYWORD_IDX);
                            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                                ((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                            );
                            (*cur_si).si_m_startcol = current_col.get() as ::core::ffi::c_int;
                            (*cur_si).si_h_startpos.lnum = current_lnum.get();
                            (*cur_si).si_h_startpos.col = 0 as ::core::ffi::c_int as colnr_T;
                            (*cur_si).si_m_endpos.lnum = current_lnum.get();
                            (*cur_si).si_m_endpos.col = endcol as colnr_T;
                            (*cur_si).si_h_endpos.lnum = current_lnum.get();
                            (*cur_si).si_h_endpos.col = endcol as colnr_T;
                            (*cur_si).si_ends = true_0;
                            (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
                            (*cur_si).si_flags = flags;
                            let c2rust_fresh3 = next_seqnr.get();
                            next_seqnr.set(next_seqnr.get() + 1);
                            (*cur_si).si_seqnr = c2rust_fresh3;
                            (*cur_si).si_cchar = cchar;
                            if (*current_state.ptr()).ga_len > 1 as ::core::ffi::c_int {
                                (*cur_si).si_flags |=
                                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                                        ((*current_state.ptr()).ga_len - 2 as ::core::ffi::c_int)
                                            as isize,
                                    ))
                                    .si_flags
                                        & HL_CONCEAL;
                            }
                            (*cur_si).si_id = syn_id;
                            (*cur_si).si_trans_id = syn_id;
                            if flags & HL_TRANSP != 0 {
                                if (*current_state.ptr()).ga_len < 2 as ::core::ffi::c_int {
                                    (*cur_si).si_attr = 0 as ::core::ffi::c_int;
                                    (*cur_si).si_trans_id = 0 as ::core::ffi::c_int;
                                } else {
                                    (*cur_si).si_attr = (*((*current_state.ptr()).ga_data
                                        as *mut stateitem_T)
                                        .offset(
                                            ((*current_state.ptr()).ga_len
                                                - 2 as ::core::ffi::c_int)
                                                as isize,
                                        ))
                                    .si_attr;
                                    (*cur_si).si_trans_id = (*((*current_state.ptr()).ga_data
                                        as *mut stateitem_T)
                                        .offset(
                                            ((*current_state.ptr()).ga_len
                                                - 2 as ::core::ffi::c_int)
                                                as isize,
                                        ))
                                    .si_trans_id;
                                }
                            } else {
                                (*cur_si).si_attr = syn_id2attr(syn_id);
                            }
                            (*cur_si).si_cont_list = ::core::ptr::null_mut::<int16_t>();
                            (*cur_si).si_next_list = next_list;
                            check_keepend();
                        }
                    }
                }
                if syn_id == 0 as ::core::ffi::c_int
                    && (*syn_block.get()).b_syn_patterns.ga_len != 0
                {
                    if next_match_idx.get() < 0 as ::core::ffi::c_int
                        || next_match_col.get() < current_col.get()
                    {
                        next_match_idx.set(0 as ::core::ffi::c_int);
                        next_match_col.set(MAXCOL as ::core::ffi::c_int);
                        let mut idx: ::core::ffi::c_int = (*syn_block.get()).b_syn_patterns.ga_len;
                        loop {
                            idx -= 1;
                            if idx < 0 as ::core::ffi::c_int {
                                break;
                            }
                            let spp: *mut synpat_T = ((*syn_block.get()).b_syn_patterns.ga_data
                                as *mut synpat_T)
                                .offset(idx as isize);
                            if !((*spp).sp_syncing as ::core::ffi::c_int
                                == syncing as ::core::ffi::c_int
                                && (displaying as ::core::ffi::c_int != 0
                                    || (*spp).sp_flags & HL_DISPLAY == 0)
                                && ((*spp).sp_type as ::core::ffi::c_int == SPTYPE_MATCH
                                    || (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START)
                                && (if !(*current_next_list.ptr()).is_null() {
                                    in_id_list(
                                        ::core::ptr::null_mut::<stateitem_T>(),
                                        current_next_list.get(),
                                        &raw mut (*spp).sp_syn,
                                        0 as ::core::ffi::c_int,
                                    )
                                } else {
                                    if cur_si.is_null() {
                                        ((*spp).sp_flags & HL_CONTAINED == 0) as ::core::ffi::c_int
                                    } else {
                                        in_id_list(
                                            cur_si,
                                            (*cur_si).si_cont_list,
                                            &raw mut (*spp).sp_syn,
                                            (*spp).sp_flags,
                                        )
                                    }
                                }) != 0)
                            {
                                continue;
                            }
                            if (*spp).sp_line_id == current_line_id.get()
                                && (*spp).sp_startcol >= next_match_col.get()
                            {
                                continue;
                            }
                            (*spp).sp_line_id = current_line_id.get();
                            let mut lc_col: colnr_T = current_col.get()
                                - (*spp).sp_offsets[SPO_LC_OFF as usize] as colnr_T;
                            if lc_col < 0 as ::core::ffi::c_int {
                                lc_col = 0 as ::core::ffi::c_int as colnr_T;
                            }
                            regmatch.rmm_ic = (*spp).sp_ic;
                            regmatch.regprog = (*spp).sp_prog;
                            let mut r: ::core::ffi::c_int = syn_regexec(
                                &raw mut regmatch,
                                current_lnum.get(),
                                lc_col,
                                &raw mut (*spp).sp_time,
                            )
                                as ::core::ffi::c_int;
                            (*spp).sp_prog = regmatch.regprog;
                            if r == 0 {
                                (*spp).sp_startcol = MAXCOL as ::core::ffi::c_int;
                            } else {
                                syn_add_start_off(
                                    &raw mut pos,
                                    &raw mut regmatch,
                                    spp,
                                    SPO_MS_OFF,
                                    -1 as ::core::ffi::c_int,
                                );
                                if pos.lnum > current_lnum.get() {
                                    (*spp).sp_startcol = MAXCOL as ::core::ffi::c_int;
                                } else {
                                    startcol = pos.col as ::core::ffi::c_int;
                                    (*spp).sp_startcol = startcol;
                                    if startcol >= next_match_col.get() {
                                        continue;
                                    }
                                    if did_match_already(idx, &raw mut zero_width_next_ga) {
                                        try_next_column.set(true_0 != 0);
                                    } else {
                                        endpos.lnum =
                                            regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum;
                                        endpos.col =
                                            regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                                        syn_add_start_off(
                                            &raw mut hl_startpos,
                                            &raw mut regmatch,
                                            spp,
                                            SPO_HS_OFF,
                                            -1 as ::core::ffi::c_int,
                                        );
                                        syn_add_end_off(
                                            &raw mut eos_pos,
                                            &raw mut regmatch,
                                            spp,
                                            SPO_RS_OFF,
                                            0 as ::core::ffi::c_int,
                                        );
                                        unref_extmatch(cur_extmatch);
                                        cur_extmatch = re_extmatch_out.get();
                                        re_extmatch_out
                                            .set(::core::ptr::null_mut::<reg_extmatch_T>());
                                        flags = 0 as ::core::ffi::c_int;
                                        eoe_pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                                        eoe_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                                        end_idx = 0 as ::core::ffi::c_int;
                                        hl_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                                        if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START
                                            && (*spp).sp_flags & HL_ONELINE != 0
                                        {
                                            let mut startpos: lpos_T = lpos_T { lnum: 0, col: 0 };
                                            startpos = endpos;
                                            find_endpos(
                                                idx,
                                                &raw mut startpos,
                                                &raw mut endpos,
                                                &raw mut hl_endpos,
                                                &raw mut flags,
                                                &raw mut eoe_pos,
                                                &raw mut end_idx,
                                                cur_extmatch,
                                            );
                                            if endpos.lnum == 0 as linenr_T {
                                                continue;
                                            }
                                        } else if (*spp).sp_type as ::core::ffi::c_int
                                            == SPTYPE_MATCH
                                        {
                                            syn_add_end_off(
                                                &raw mut hl_endpos,
                                                &raw mut regmatch,
                                                spp,
                                                SPO_HE_OFF,
                                                0 as ::core::ffi::c_int,
                                            );
                                            syn_add_end_off(
                                                &raw mut endpos,
                                                &raw mut regmatch,
                                                spp,
                                                SPO_ME_OFF,
                                                0 as ::core::ffi::c_int,
                                            );
                                            if endpos.lnum == current_lnum.get()
                                                && (endpos.col + syncing as ::core::ffi::c_int)
                                                    < startcol
                                            {
                                                if regmatch.startpos
                                                    [0 as ::core::ffi::c_int as usize]
                                                    .col
                                                    == regmatch.endpos
                                                        [0 as ::core::ffi::c_int as usize]
                                                        .col
                                                {
                                                    try_next_column.set(true_0 != 0);
                                                }
                                                continue;
                                            }
                                        }
                                        if hl_startpos.lnum == current_lnum.get()
                                            && hl_startpos.col < startcol
                                        {
                                            hl_startpos.col = startcol as colnr_T;
                                        }
                                        limit_pos_zero(&raw mut hl_endpos, &raw mut endpos);
                                        next_match_idx.set(idx);
                                        next_match_col.set(startcol);
                                        next_match_m_endpos.set(endpos);
                                        next_match_h_endpos.set(hl_endpos);
                                        next_match_h_startpos.set(hl_startpos);
                                        next_match_flags.set(flags);
                                        next_match_eos_pos.set(eos_pos);
                                        next_match_eoe_pos.set(eoe_pos);
                                        next_match_end_idx.set(end_idx);
                                        unref_extmatch(next_match_extmatch.get());
                                        next_match_extmatch.set(cur_extmatch);
                                        cur_extmatch = ::core::ptr::null_mut::<reg_extmatch_T>();
                                    }
                                }
                            }
                        }
                    }
                    if next_match_idx.get() >= 0 as ::core::ffi::c_int
                        && next_match_col.get() == current_col.get()
                    {
                        let mut lspp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
                        lspp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(next_match_idx.get() as isize);
                        if (*next_match_m_endpos.ptr()).lnum == current_lnum.get()
                            && (*next_match_m_endpos.ptr()).col == current_col.get()
                            && !(*lspp).sp_next_list.is_null()
                        {
                            current_next_list.set((*lspp).sp_next_list);
                            current_next_flags.set((*lspp).sp_flags);
                            keep_next_list = true_0 != 0;
                            zero_width_next_list = true_0 != 0;
                            ga_grow(&raw mut zero_width_next_ga, 1 as ::core::ffi::c_int);
                            *(zero_width_next_ga.ga_data as *mut ::core::ffi::c_int)
                                .offset(zero_width_next_ga.ga_len as isize) = next_match_idx.get();
                            zero_width_next_ga.ga_len += 1;
                            next_match_idx.set(-1 as ::core::ffi::c_int);
                        } else {
                            cur_si = push_next_match();
                        }
                        found_match = true_0 != 0;
                    }
                }
            }
            if !(*current_next_list.ptr()).is_null() && !keep_next_list {
                if !found_match {
                    line = syn_getcurline();
                    if current_next_flags.get() & HL_SKIPWHITE != 0
                        && ascii_iswhite(
                            *line.offset(current_col.get() as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                        || current_next_flags.get() & HL_SKIPEMPTY != 0
                            && *line as ::core::ffi::c_int == NUL
                    {
                        break;
                    }
                }
                current_next_list.set(::core::ptr::null_mut::<int16_t>());
                next_match_idx.set(-1 as ::core::ffi::c_int);
                if !zero_width_next_list {
                    found_match = true_0 != 0;
                }
            }
            if !found_match {
                break;
            }
        }
        restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
        current_attr.set(0 as ::core::ffi::c_int);
        current_id.set(0 as ::core::ffi::c_int);
        current_trans_id.set(0 as ::core::ffi::c_int);
        current_flags.set(0 as ::core::ffi::c_int);
        current_seqnr.set(0 as ::core::ffi::c_int);
        if !cur_si.is_null() {
            let mut idx_0: ::core::ffi::c_int =
                (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
            while idx_0 >= 0 as ::core::ffi::c_int {
                sip = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(idx_0 as isize);
                if (current_lnum.get() > (*sip).si_h_startpos.lnum
                    || current_lnum.get() == (*sip).si_h_startpos.lnum
                        && current_col.get() >= (*sip).si_h_startpos.col)
                    && ((*sip).si_h_endpos.lnum == 0 as linenr_T
                        || current_lnum.get() < (*sip).si_h_endpos.lnum
                        || current_lnum.get() == (*sip).si_h_endpos.lnum
                            && current_col.get() < (*sip).si_h_endpos.col)
                {
                    current_attr.set((*sip).si_attr);
                    current_id.set((*sip).si_id);
                    current_trans_id.set((*sip).si_trans_id);
                    current_flags.set((*sip).si_flags);
                    current_seqnr.set((*sip).si_seqnr);
                    current_sub_char.set((*sip).si_cchar);
                    break;
                } else {
                    idx_0 -= 1;
                }
            }
            if !can_spell.is_null() {
                let mut sps: sp_syn = sp_syn {
                    inc_tag: 0,
                    id: 0,
                    cont_in_list: ::core::ptr::null_mut::<int16_t>(),
                };
                if (*syn_block.get()).b_spell_cluster_id == 0 as ::core::ffi::c_int {
                    if (*syn_block.get()).b_nospell_cluster_id == 0 as ::core::ffi::c_int
                        || current_trans_id.get() == 0 as ::core::ffi::c_int
                    {
                        *can_spell = (*syn_block.get()).b_syn_spell != SYNSPL_NOTOP;
                    } else {
                        sps.inc_tag = 0 as ::core::ffi::c_int;
                        sps.id = (*syn_block.get()).b_nospell_cluster_id as int16_t;
                        sps.cont_in_list = ::core::ptr::null_mut::<int16_t>();
                        *can_spell = in_id_list(
                            sip,
                            (*sip).si_cont_list,
                            &raw mut sps,
                            0 as ::core::ffi::c_int,
                        ) == 0;
                    }
                } else if current_trans_id.get() == 0 as ::core::ffi::c_int {
                    *can_spell = (*syn_block.get()).b_syn_spell == SYNSPL_TOP;
                } else {
                    sps.inc_tag = 0 as ::core::ffi::c_int;
                    sps.id = (*syn_block.get()).b_spell_cluster_id as int16_t;
                    sps.cont_in_list = ::core::ptr::null_mut::<int16_t>();
                    *can_spell = in_id_list(
                        sip,
                        (*sip).si_cont_list,
                        &raw mut sps,
                        0 as ::core::ffi::c_int,
                    ) != 0;
                    if (*syn_block.get()).b_nospell_cluster_id != 0 as ::core::ffi::c_int {
                        sps.id = (*syn_block.get()).b_nospell_cluster_id as int16_t;
                        if in_id_list(
                            sip,
                            (*sip).si_cont_list,
                            &raw mut sps,
                            0 as ::core::ffi::c_int,
                        ) != 0
                        {
                            *can_spell = false_0 != 0;
                        }
                    }
                }
            }
            if !syncing && !keep_state {
                check_state_ends();
                if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int)
                    && *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int
                        != NUL
                {
                    (*current_col.ptr()) += 1;
                    check_state_ends();
                    (*current_col.ptr()) -= 1;
                }
            }
        } else if !can_spell.is_null() {
            *can_spell = if (*syn_block.get()).b_syn_spell == SYNSPL_DEFAULT {
                ((*syn_block.get()).b_spell_cluster_id == 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
            } else {
                ((*syn_block.get()).b_syn_spell == SYNSPL_TOP) as ::core::ffi::c_int
            } != 0;
        }
        if !(*current_next_list.ptr()).is_null()
            && {
                line = syn_getcurline();
                *line.offset(current_col.get() as isize) as ::core::ffi::c_int != NUL
            }
            && *line.offset(
                (current_col.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int
                == NUL
            && current_next_flags.get() & (HL_SKIPNL | HL_SKIPEMPTY) == 0
        {
            current_next_list.set(::core::ptr::null_mut::<int16_t>());
        }
        if !(zero_width_next_ga.ga_len <= 0 as ::core::ffi::c_int) {
            ga_clear(&raw mut zero_width_next_ga);
        }
        unref_extmatch(re_extmatch_out.get());
        re_extmatch_out.set(::core::ptr::null_mut::<reg_extmatch_T>());
        unref_extmatch(cur_extmatch);
        return current_attr.get();
    }
}

pub(crate) unsafe extern "C" fn did_match_already(
    mut idx: ::core::ffi::c_int,
    mut gap: *mut garray_T,
) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = (*current_state.ptr()).ga_len;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                .si_m_startcol
                == current_col.get()
                && (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_m_lnum
                    == current_lnum.get() as ::core::ffi::c_int
                && (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx
                    == idx
            {
                return true_0 != 0;
            }
        }
        let mut i_0: ::core::ffi::c_int = (*gap).ga_len;
        loop {
            i_0 -= 1;
            if i_0 < 0 as ::core::ffi::c_int {
                break;
            }
            if *((*gap).ga_data as *mut ::core::ffi::c_int).offset(i_0 as isize) == idx {
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}
