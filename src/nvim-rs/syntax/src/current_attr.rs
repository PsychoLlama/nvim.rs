//! Core attribute computation for syntax highlighting.
//!
//! This module contains the migrated implementations of:
//! - syn_current_attr() — compute highlight attribute for current column
//! - syn_finish_line() — process remaining columns to find sync items

use std::ffi::c_int;

use crate::check_ends::{check_keepend, check_state_ends};
use crate::offset::{limit_pos_zero, syn_add_end_off, syn_add_start_off, RegMatch};
use crate::region::find_endpos;
use crate::state::Position;
use crate::synblock_struct::synblock_ref;
use crate::types::{
    ExtMatchHandle, IdListHandle, StateItemHandle, HL_CONCEAL, HL_DISPLAY, HL_ONELINE,
    HL_SKIPEMPTY, HL_SKIPNL, HL_SKIPWHITE, HL_SYNC_HERE, HL_SYNC_THERE, HL_TRANSP, KEYWORD_IDX,
    SPO_HE_OFF, SPO_HS_OFF, SPO_ME_OFF, SPO_MS_OFF, SPO_RS_OFF, SPTYPE_MATCH, SPTYPE_START,
    SYNSPL_DEFAULT, SYNSPL_NOTOP, SYNSPL_TOP,
};

const MAXCOL: i32 = 0x7fff_ffff;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Current position

    // Current state status

    // (nvim_syn_set_current_from_stateitem inlined -- statics are in Rust)
    // (nvim_syn_zero_current inlined -- statics are in Rust)

    // State stack

    // Next match

    // Next list management

    // Line operations
    fn nvim_syn_getcurline_byte_at(col: c_int) -> c_int;
    #[link_name = "rs_syn_update_ends"]
    fn nvim_syn_update_ends(startofline: c_int);

    // (nvim_syn_save_chartab/restore_chartab deleted: call Rust directly)

    // Extmatch management
    fn nvim_syn_unref_extmatch(em: ExtMatchHandle);
    fn nvim_syn_take_re_extmatch_out() -> ExtMatchHandle;
    fn nvim_syn_clear_re_extmatch_out();

    // (syn_id2attr: use crate::highlight::syn_id2attr directly)

    // Synblock queries
    fn nvim_syn_has_containedin() -> c_int;
    fn nvim_syn_has_keywords() -> c_int;
    fn nvim_syn_has_keywords_ic() -> c_int;

    // Line ID tracking

    // Spell
    fn nvim_syn_get_syn_block() -> crate::types::SynBlockHandle;
    fn nvim_syn_get_syn_spell() -> c_int;
    fn nvim_syn_in_id_list_spell(sip: StateItemHandle, list: IdListHandle, id: c_int) -> c_int;

    // Word check
    fn nvim_syn_vim_iswordp_buf(p: *mut i8) -> c_int;
    fn utf_head_off(base: *mut i8, p: *mut i8) -> c_int;
    fn nvim_syn_ascii_iswhite_char(c: c_int) -> c_int;

    // getcurline
    #[link_name = "rs_syn_getcurline"]
    fn nvim_syn_getcurline() -> *mut i8;
    fn nvim_syn_getcurline_len() -> c_int;

}

// Use the Rust push_next_match from check_ends module
use crate::check_ends::push_next_match;

/// Check pattern containment for a pattern at pat_idx.
///
/// Replaces C `nvim_syn_check_pattern_containment`.
/// Returns 1 if the pattern is contained in the correct context, 0 otherwise.
///
/// # Safety
/// Accesses C global state.
unsafe fn check_pattern_containment(
    pat_idx: c_int,
    si_idx: c_int,
    has_next_list: bool,
    has_cur_si: bool,
) -> c_int {
    use crate::containment::rs_syn_in_id_list;
    use crate::types::{StateItemHandle, HL_CONTAINED};

    let block = nvim_syn_get_syn_block();
    let pat_p = crate::statics::syn_item_at(block, pat_idx);
    let (syn_id, inc_tag, cont_in_list, flags) = if pat_p.is_null() {
        (0, 0, IdListHandle(std::ptr::null_mut()), 0)
    } else {
        (
            (*pat_p).sp_syn.id as c_int,
            (*pat_p).sp_syn.inc_tag,
            IdListHandle((*pat_p).sp_syn.cont_in_list),
            (*pat_p).sp_flags,
        )
    };

    if has_next_list {
        let next_list = IdListHandle(crate::statics::CURRENT_NEXT_LIST);
        rs_syn_in_id_list(
            StateItemHandle(std::ptr::null_mut()),
            next_list,
            syn_id,
            inc_tag,
            cont_in_list,
            0,
        )
    } else if !has_cur_si {
        if flags & HL_CONTAINED != 0 {
            0
        } else {
            1
        }
    } else {
        let cur_si = crate::statics::current_state_item(si_idx);
        let cont_list = IdListHandle((*cur_si.as_ptr()).si_cont_list);
        rs_syn_in_id_list(cur_si, cont_list, syn_id, inc_tag, cont_in_list, flags)
    }
}

/// Compute the syntax highlight attribute for the current column.
///
/// This is the main entry point called for every column of every displayed line.
pub unsafe fn syn_current_attr(
    syncing: bool,
    displaying: bool,
    can_spell: *mut c_int,
    keep_state: bool,
) -> i32 {
    let current_lnum = crate::statics::CURRENT_LNUM;
    let current_col = crate::statics::CURRENT_COL;

    // No character, no attributes! Past end of line?
    // Do try matching with an empty line (could be the start of a region).
    let line_byte = nvim_syn_getcurline_byte_at(current_col);
    if line_byte == 0 && current_col != 0 {
        // If we found a match after the last column, use it.
        let next_idx = crate::statics::NEXT_MATCH_IDX;
        let next_col = crate::statics::NEXT_MATCH_COL;
        if next_idx >= 0 && next_col >= current_col && next_col != MAXCOL {
            push_next_match();
        }
        crate::statics::CURRENT_FINISHED = 1;
        crate::statics::CURRENT_STATE_STORED = 0;
        return 0;
    }

    // if the current or next character is NUL, we will finish the line now
    if line_byte == 0 || nvim_syn_getcurline_byte_at(current_col + 1) == 0 {
        crate::statics::CURRENT_FINISHED = 1;
        crate::statics::CURRENT_STATE_STORED = 0;
    }

    // When in the previous column there was a match but it could not be used
    // (empty match or already matched in this column) need to try again in
    // the next column.
    static mut TRY_NEXT_COLUMN: bool = false;
    if TRY_NEXT_COLUMN {
        crate::statics::NEXT_MATCH_IDX = -1;
        TRY_NEXT_COLUMN = false;
    }

    // Only check for keywords when not syncing and there are some.
    let do_keywords = !syncing && (nvim_syn_has_keywords() != 0 || nvim_syn_has_keywords_ic() != 0);

    // Init the list of zero-width matches with a nextlist.
    let mut zero_width_next_ga: Vec<i32> = Vec::new();

    // use syntax iskeyword option
    let mut buf_chartab = [0i8; 32];
    crate::line_init::rs_save_chartab(buf_chartab.as_mut_ptr());

    // Track cur_extmatch for lifetime management
    let mut cur_extmatch: ExtMatchHandle = ExtMatchHandle(std::ptr::null_mut());

    // Repeat matching keywords and patterns, to find contained items at the
    // same column.
    let mut found_match;
    let mut keep_next_list;
    let mut zero_width_next_list = false;
    let mut cur_si_valid;

    loop {
        found_match = false;
        keep_next_list = false;
        let mut syn_id: i32 = 0;
        let current_col = crate::statics::CURRENT_COL;

        // 1. Check for a current state.
        let state_len = crate::statics::CURRENT_STATE.ga_len;
        cur_si_valid = state_len > 0;

        if nvim_syn_has_containedin() != 0 || !cur_si_valid || {
            let si = crate::statics::current_state_item(state_len - 1);
            !(*si.as_ptr()).si_cont_list.is_null()
        } {
            // 2. Check for keywords
            if do_keywords {
                let line = nvim_syn_getcurline();
                let cur_pos = line.offset(current_col as isize);

                if nvim_syn_vim_iswordp_buf(cur_pos) != 0
                    && (current_col == 0 || {
                        let prev = cur_pos.offset(-1);
                        let head_off = utf_head_off(line, prev);
                        let word_start = prev.offset(-(head_off as isize));
                        nvim_syn_vim_iswordp_buf(word_start) == 0
                    })
                {
                    let cur_si_handle = if cur_si_valid {
                        crate::statics::current_state_item(state_len - 1)
                    } else {
                        StateItemHandle(std::ptr::null_mut())
                    };

                    let mut endcol: c_int = 0;
                    let mut flags: c_int = 0;
                    let mut next_list = IdListHandle(std::ptr::null_mut());
                    let mut cchar: c_int = 0;

                    syn_id = crate::keyword::check_keyword_id(
                        line,
                        current_col,
                        &mut endcol,
                        &mut flags,
                        &mut next_list,
                        cur_si_handle,
                        &mut cchar,
                    );

                    if syn_id != 0 {
                        crate::state_ops::rs_syn_push_current_state(KEYWORD_IDX);
                        let new_len = crate::statics::CURRENT_STATE.ga_len;
                        let cur_si = crate::statics::current_state_item(new_len - 1);
                        {
                            let p = cur_si.as_ptr();
                            (*p).si_m_lnum = current_lnum;
                            (*p).si_m_startcol = current_col;
                            (*p).si_m_endpos.lnum = current_lnum;
                            (*p).si_m_endpos.col = endcol;
                            (*p).si_h_startpos.lnum = current_lnum;
                            (*p).si_h_startpos.col = 0;
                            (*p).si_h_endpos.lnum = current_lnum;
                            (*p).si_h_endpos.col = endcol;
                            (*p).si_ends = 1;
                            (*p).si_end_idx = 0;
                            (*p).si_flags = flags;
                            (*p).si_seqnr = {
                                let _s = crate::statics::NEXT_SEQNR;
                                crate::statics::NEXT_SEQNR += 1;
                                _s
                            };
                            (*p).si_cchar = cchar;
                        }

                        if new_len > 1 {
                            let prev_si = crate::statics::current_state_item(new_len - 2);
                            if (*prev_si.as_ptr()).si_flags & HL_CONCEAL != 0 {
                                (*cur_si.as_ptr()).si_flags |= HL_CONCEAL;
                            }
                        }

                        {
                            let p = cur_si.as_ptr();
                            (*p).si_id = syn_id;
                            (*p).si_trans_id = syn_id;
                        }

                        if flags & HL_TRANSP != 0 {
                            if new_len < 2 {
                                let p = cur_si.as_ptr();
                                (*p).si_attr = 0;
                                (*p).si_trans_id = 0;
                            } else {
                                let prev_si = crate::statics::current_state_item(new_len - 2);
                                let p = cur_si.as_ptr();
                                (*p).si_attr = (*prev_si.as_ptr()).si_attr;
                                (*p).si_trans_id = (*prev_si.as_ptr()).si_trans_id;
                            }
                        } else {
                            (*cur_si.as_ptr()).si_attr = crate::highlight::syn_id2attr(syn_id);
                        }

                        {
                            let p = cur_si.as_ptr();
                            (*p).si_cont_list = std::ptr::null_mut();
                            (*p).si_next_list = next_list.0;
                        }
                        check_keepend();
                        cur_si_valid = true;
                    }
                }
            }

            // 3. Check for patterns (only if no keyword found).
            if syn_id == 0 && synblock_ref(nvim_syn_get_syn_block()).b_syn_patterns.ga_len > 0 {
                let current_col = crate::statics::CURRENT_COL;
                let next_match_idx = crate::statics::NEXT_MATCH_IDX;
                let next_match_col = crate::statics::NEXT_MATCH_COL;

                if next_match_idx < 0 || next_match_col < current_col {
                    // Check all relevant patterns for a match at this position.
                    crate::statics::NEXT_MATCH_IDX = 0;
                    crate::statics::NEXT_MATCH_COL = MAXCOL;
                    let current_line_id = crate::statics::CURRENT_LINE_ID;
                    let state_len = crate::statics::CURRENT_STATE.ga_len;
                    let has_next_list = !crate::statics::CURRENT_NEXT_LIST.is_null();
                    let si_idx = if cur_si_valid { state_len - 1 } else { -1 };

                    let block = nvim_syn_get_syn_block();
                    let pat_len = synblock_ref(block).b_syn_patterns.ga_len;
                    for idx in (0..pat_len).rev() {
                        let pat_p = crate::statics::syn_item_at(block, idx);
                        if pat_p.is_null() {
                            continue;
                        }
                        let pat_type = (*pat_p).sp_type as c_int;
                        let pat_syncing = (*pat_p).sp_syncing as c_int;
                        let pat_display = ((*pat_p).sp_flags & HL_DISPLAY) != 0;

                        if pat_syncing != (syncing as c_int) {
                            continue;
                        }
                        if !displaying && pat_display {
                            continue;
                        }
                        if pat_type != SPTYPE_MATCH && pat_type != SPTYPE_START {
                            continue;
                        }

                        if check_pattern_containment(idx, si_idx, has_next_list, cur_si_valid) == 0
                        {
                            continue;
                        }

                        let cur_next_match_col = crate::statics::NEXT_MATCH_COL;

                        // If we already tried matching in this line, and
                        // there isn't a match before next_match_col, skip.
                        if (*pat_p).sp_line_id == current_line_id
                            && (*pat_p).sp_startcol >= cur_next_match_col
                        {
                            continue;
                        }
                        (*pat_p).sp_line_id = current_line_id;

                        let mut lc_col =
                            current_col - (*pat_p).sp_offsets[crate::types::SPO_LC_OFF as usize];
                        if lc_col < 0 {
                            lc_col = 0;
                        }

                        let mut s_lnum: c_int = 0;
                        let mut s_col: c_int = 0;
                        let mut e_lnum: c_int = 0;
                        let mut e_col: c_int = 0;
                        let r = crate::regexec::syn_regexec_by_idx(
                            idx,
                            current_lnum,
                            lc_col,
                            &mut s_lnum,
                            &mut s_col,
                            &mut e_lnum,
                            &mut e_col,
                        );
                        if r == 0 {
                            (*pat_p).sp_startcol = MAXCOL;
                            continue;
                        }

                        // Compute the first column of the match.
                        let regmatch = RegMatch {
                            startpos: Position {
                                lnum: s_lnum,
                                col: s_col,
                            },
                            endpos: Position {
                                lnum: e_lnum,
                                col: e_col,
                            },
                        };

                        let pos = syn_add_start_off(&regmatch, idx, SPO_MS_OFF, -1);
                        if pos.lnum > current_lnum {
                            (*pat_p).sp_startcol = MAXCOL;
                            continue;
                        }
                        let startcol = pos.col;

                        (*pat_p).sp_startcol = startcol;

                        if startcol >= cur_next_match_col {
                            continue;
                        }

                        // If we matched this pattern at this position before, skip it.
                        if did_match_already(idx, &zero_width_next_ga) {
                            TRY_NEXT_COLUMN = true;
                            continue;
                        }

                        let mut endpos = Position {
                            lnum: e_lnum,
                            col: e_col,
                        };

                        // Compute the highlight start.
                        let hl_startpos = syn_add_start_off(&regmatch, idx, SPO_HS_OFF, -1);

                        // Compute the region start (default: end of match).
                        let eos_pos = syn_add_end_off(&regmatch, idx, SPO_RS_OFF, 0);

                        // Grab the external submatches before they get overwritten.
                        nvim_syn_unref_extmatch(cur_extmatch);
                        cur_extmatch = nvim_syn_take_re_extmatch_out();

                        let mut flags: i32 = 0;
                        let mut eoe_pos = Position { lnum: 0, col: 0 };
                        let mut end_idx: i32 = 0;
                        let mut hl_endpos = Position { lnum: 0, col: 0 };

                        let pat_flags = (*pat_p).sp_flags;

                        if pat_type == SPTYPE_START && (pat_flags & HL_ONELINE) != 0 {
                            // For a "oneline" the end must be found in the same line.
                            let startpos = endpos;
                            let result = find_endpos(idx, &startpos, cur_extmatch);
                            endpos = result.m_endpos;
                            hl_endpos = result.hl_endpos;
                            flags = result.flags;
                            eoe_pos = result.end_endpos;
                            end_idx = result.end_idx;
                            if endpos.lnum == 0 {
                                continue; // not found
                            }
                        } else if pat_type == SPTYPE_MATCH {
                            // For a "match" the size must be > 0 after the
                            // end offset has been added.
                            hl_endpos = syn_add_end_off(&regmatch, idx, SPO_HE_OFF, 0);
                            endpos = syn_add_end_off(&regmatch, idx, SPO_ME_OFF, 0);
                            if endpos.lnum == current_lnum
                                && endpos.col + (syncing as i32) < startcol
                            {
                                if s_col == e_col && s_lnum == e_lnum {
                                    TRY_NEXT_COLUMN = true;
                                }
                                continue;
                            }
                        }

                        // Highlighting must start after startpos and end before endpos.
                        let mut hl_startpos = hl_startpos;
                        if hl_startpos.lnum == current_lnum && hl_startpos.col < startcol {
                            hl_startpos.col = startcol;
                        }
                        limit_pos_zero(&mut hl_endpos, &endpos);

                        // Store best match (inline into Rust statics)
                        crate::statics::NEXT_MATCH_IDX = idx;
                        crate::statics::NEXT_MATCH_COL = startcol;
                        crate::statics::NEXT_MATCH_M_ENDPOS.lnum = endpos.lnum;
                        crate::statics::NEXT_MATCH_M_ENDPOS.col = endpos.col;
                        crate::statics::NEXT_MATCH_H_ENDPOS.lnum = hl_endpos.lnum;
                        crate::statics::NEXT_MATCH_H_ENDPOS.col = hl_endpos.col;
                        crate::statics::NEXT_MATCH_H_STARTPOS.lnum = hl_startpos.lnum;
                        crate::statics::NEXT_MATCH_H_STARTPOS.col = hl_startpos.col;
                        crate::statics::NEXT_MATCH_FLAGS = flags;
                        crate::statics::NEXT_MATCH_EOS_POS.lnum = eos_pos.lnum;
                        crate::statics::NEXT_MATCH_EOS_POS.col = eos_pos.col;
                        crate::statics::NEXT_MATCH_EOE_POS.lnum = eoe_pos.lnum;
                        crate::statics::NEXT_MATCH_EOE_POS.col = eoe_pos.col;
                        crate::statics::NEXT_MATCH_END_IDX = end_idx;
                        nvim_syn_unref_extmatch(ExtMatchHandle(
                            crate::statics::NEXT_MATCH_EXTMATCH,
                        ));
                        crate::statics::NEXT_MATCH_EXTMATCH = cur_extmatch.0;
                        cur_extmatch = ExtMatchHandle(std::ptr::null_mut());
                    }
                }

                // If we found a match at the current column, use it.
                let current_col = crate::statics::CURRENT_COL;
                let next_match_idx = crate::statics::NEXT_MATCH_IDX;
                let next_match_col = crate::statics::NEXT_MATCH_COL;

                if next_match_idx >= 0 && next_match_col == current_col {
                    // When a zero-width item matched which has a nextgroup,
                    // don't push the item but set nextgroup.
                    let nm_pos = crate::match_engine::next_match_positions();
                    let (pat_next_list_ptr, pat_flags_nmi) = {
                        let blk = nvim_syn_get_syn_block();
                        let nmi_p = crate::statics::syn_item_at(blk, next_match_idx);
                        if nmi_p.is_null() {
                            (std::ptr::null_mut::<i16>(), 0i32)
                        } else {
                            ((*nmi_p).sp_next_list, (*nmi_p).sp_flags)
                        }
                    };

                    if nm_pos.m_endpos.lnum == current_lnum
                        && nm_pos.m_endpos.col == current_col
                        && !pat_next_list_ptr.is_null()
                    {
                        crate::statics::CURRENT_NEXT_LIST = pat_next_list_ptr;
                        let pat_flags = pat_flags_nmi;
                        crate::statics::CURRENT_NEXT_FLAGS = pat_flags;
                        keep_next_list = true;
                        zero_width_next_list = true;

                        zero_width_next_ga.push(next_match_idx);
                        crate::statics::NEXT_MATCH_IDX = -1;
                    } else {
                        push_next_match();
                        cur_si_valid = true;
                    }
                    found_match = true;
                }
            }
        }

        // Handle searching for nextgroup match.
        if !crate::statics::CURRENT_NEXT_LIST.is_null() && !keep_next_list {
            if !found_match {
                let current_col = crate::statics::CURRENT_COL;
                let current_next_flags = crate::statics::CURRENT_NEXT_FLAGS;
                let line_byte = nvim_syn_getcurline_byte_at(current_col);

                if (current_next_flags & HL_SKIPWHITE) != 0
                    && nvim_syn_ascii_iswhite_char(line_byte) != 0
                {
                    break;
                }
                if (current_next_flags & HL_SKIPEMPTY) != 0 && nvim_syn_getcurline_byte_at(0) == 0 {
                    break;
                }
            }

            crate::statics::CURRENT_NEXT_LIST = std::ptr::null_mut();
            crate::statics::NEXT_MATCH_IDX = -1;
            if !zero_width_next_list {
                found_match = true;
            }
        }

        if !found_match {
            break;
        }
    }

    crate::line_init::rs_restore_chartab(buf_chartab.as_mut_ptr());

    // Use attributes from the current state, if within its highlighting.
    // Zero all current_* highlight fields
    crate::statics::CURRENT_ATTR = 0;
    crate::statics::CURRENT_ID = 0;
    crate::statics::CURRENT_TRANS_ID = 0;
    crate::statics::CURRENT_FLAGS = 0;
    crate::statics::CURRENT_SEQNR = 0;

    let current_col = crate::statics::CURRENT_COL;
    let state_len = crate::statics::CURRENT_STATE.ga_len;

    let mut sip_handle = StateItemHandle(std::ptr::null_mut());

    if cur_si_valid && state_len > 0 {
        for idx in (0..state_len).rev() {
            let sip = crate::statics::current_state_item(idx);
            let (h_start_lnum, h_start_col, h_end_lnum, h_end_col) = {
                let p = sip.as_ptr();
                (
                    (*p).si_h_startpos.lnum,
                    (*p).si_h_startpos.col,
                    (*p).si_h_endpos.lnum,
                    (*p).si_h_endpos.col,
                )
            };

            if (current_lnum > h_start_lnum
                || (current_lnum == h_start_lnum && current_col >= h_start_col))
                && (h_end_lnum == 0
                    || current_lnum < h_end_lnum
                    || (current_lnum == h_end_lnum && current_col < h_end_col))
            {
                // Set all current_* fields from stateitem
                let si_p = sip.as_ptr();
                crate::statics::CURRENT_ATTR = (*si_p).si_attr;
                crate::statics::CURRENT_ID = (*si_p).si_id;
                crate::statics::CURRENT_TRANS_ID = (*si_p).si_trans_id;
                crate::statics::CURRENT_FLAGS = (*si_p).si_flags;
                crate::statics::CURRENT_SEQNR = (*si_p).si_seqnr;
                crate::statics::CURRENT_SUB_CHAR = (*si_p).si_cchar;
                sip_handle = sip;
                break;
            }
        }

        if !can_spell.is_null() {
            compute_can_spell(sip_handle, can_spell);
        }

        // Check for end of current state at the next column.
        if !syncing && !keep_state {
            check_state_ends();
            if !crate::statics::current_state_is_empty()
                && nvim_syn_getcurline_byte_at(current_col) != 0
            {
                crate::statics::CURRENT_COL = current_col + 1;
                check_state_ends();
                crate::statics::CURRENT_COL = current_col;
            }
        }
    } else if !can_spell.is_null() {
        // Default: Only do spelling when there is no @Spell cluster or when
        // ":syn spell toplevel" was used.
        let syn_spell = nvim_syn_get_syn_spell();
        let spell_cluster = synblock_ref(nvim_syn_get_syn_block()).b_spell_cluster_id;
        *can_spell = if syn_spell == SYNSPL_DEFAULT {
            (spell_cluster == 0) as c_int
        } else {
            (syn_spell == SYNSPL_TOP) as c_int
        };
    }

    // nextgroup ends at end of line, unless "skipnl" or "skipempty" present
    let current_col = crate::statics::CURRENT_COL;
    if !crate::statics::CURRENT_NEXT_LIST.is_null()
        && nvim_syn_getcurline_byte_at(current_col) != 0
        && nvim_syn_getcurline_byte_at(current_col + 1) == 0
        && (crate::statics::CURRENT_NEXT_FLAGS & (HL_SKIPNL | HL_SKIPEMPTY)) == 0
    {
        crate::statics::CURRENT_NEXT_LIST = std::ptr::null_mut();
    }

    // No longer need external matches. But keep next_match_extmatch.
    nvim_syn_clear_re_extmatch_out();
    nvim_syn_unref_extmatch(cur_extmatch);

    crate::statics::CURRENT_ATTR
}

/// Compute can_spell based on spell clusters.
unsafe fn compute_can_spell(sip: StateItemHandle, can_spell: *mut c_int) {
    let block = nvim_syn_get_syn_block();
    let spell_cluster = unsafe { synblock_ref(block).b_spell_cluster_id };
    let nospell_cluster = unsafe { synblock_ref(block).b_nospell_cluster_id };
    let syn_spell = nvim_syn_get_syn_spell();
    let current_trans_id = if sip.is_null() {
        0
    } else {
        // Read from the global (we just set it above)
        crate::statics::CURRENT_TRANS_ID
    };

    if spell_cluster == 0 {
        // There is no @Spell cluster: Do spelling for items without @NoSpell.
        if nospell_cluster == 0 || current_trans_id == 0 {
            *can_spell = (syn_spell != SYNSPL_NOTOP) as c_int;
        } else {
            let cont_list = IdListHandle((*sip.as_ptr()).si_cont_list);
            *can_spell = (nvim_syn_in_id_list_spell(sip, cont_list, nospell_cluster) == 0) as c_int;
        }
    } else {
        // The @Spell cluster is defined.
        if current_trans_id == 0 {
            *can_spell = (syn_spell == SYNSPL_TOP) as c_int;
        } else {
            let cont_list = IdListHandle((*sip.as_ptr()).si_cont_list);
            *can_spell = nvim_syn_in_id_list_spell(sip, cont_list, spell_cluster);

            if nospell_cluster != 0
                && nvim_syn_in_id_list_spell(sip, cont_list, nospell_cluster) != 0
            {
                *can_spell = 0;
            }
        }
    }
}

/// Check if pattern `idx` was already matched at the current column.
fn did_match_already(idx: i32, zero_width_ga: &[i32]) -> bool {
    let state_len = unsafe { crate::statics::CURRENT_STATE.ga_len };
    let current_col = unsafe { crate::statics::CURRENT_COL };
    let current_lnum = unsafe { crate::statics::CURRENT_LNUM };

    for i in (0..state_len).rev() {
        let si = unsafe { crate::statics::current_state_item(i) };
        if si.is_null() {
            continue;
        }
        let (m_lnum, m_startcol, si_idx) = unsafe {
            let p = si.as_ptr();
            ((*p).si_m_lnum, (*p).si_m_startcol, (*p).si_idx)
        };
        if m_startcol == current_col && m_lnum == current_lnum && si_idx == idx {
            return true;
        }
    }

    for &ga_idx in zero_width_ga {
        if ga_idx == idx {
            return true;
        }
    }

    false
}

/// Process a syntax line to the end, checking for sync matches.
///
/// Returns true if a sync item is found.
pub unsafe fn syn_finish_line(syncing: bool) -> bool {
    while crate::statics::CURRENT_FINISHED == 0 {
        syn_current_attr(syncing, false, std::ptr::null_mut(), false);

        // When syncing, and found some item, need to check the item.
        if syncing && crate::statics::CURRENT_STATE.ga_len > 0 {
            let state_len = crate::statics::CURRENT_STATE.ga_len;
            let cur_si = crate::statics::current_state_item(state_len - 1);
            let si_idx = unsafe { (*cur_si.as_ptr()).si_idx };
            if si_idx >= 0 {
                let pat_flags = unsafe {
                    let blk = nvim_syn_get_syn_block();
                    let pp = crate::statics::syn_item_at(blk, si_idx);
                    if pp.is_null() {
                        0
                    } else {
                        (*pp).sp_flags
                    }
                };
                if (pat_flags & (HL_SYNC_HERE | HL_SYNC_THERE)) != 0 {
                    return true;
                }
            }

            // syn_current_attr() will have skipped the check for an item
            // that ends here, need to do that now.
            let current_col = crate::statics::CURRENT_COL;
            if nvim_syn_getcurline_byte_at(current_col) != 0 {
                crate::statics::CURRENT_COL = current_col + 1;
            }
            check_state_ends();
            crate::statics::CURRENT_COL = current_col;
        }

        let current_col = crate::statics::CURRENT_COL;
        crate::statics::CURRENT_COL = current_col + 1;
    }
    false
}

// =============================================================================
// Exported FFI functions
// =============================================================================

/// Rust implementation of syn_current_attr.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_current_attr_impl(
    syncing: c_int,
    displaying: c_int,
    can_spell: *mut c_int,
    keep_state: c_int,
) -> c_int {
    syn_current_attr(syncing != 0, displaying != 0, can_spell, keep_state != 0)
}

/// Rust implementation of syn_finish_line.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_finish_line(syncing: c_int) -> c_int {
    syn_finish_line(syncing != 0) as c_int
}
