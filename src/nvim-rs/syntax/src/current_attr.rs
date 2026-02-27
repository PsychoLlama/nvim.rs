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
use crate::types::{
    ExtMatchHandle, IdListHandle, StateItemHandle, HL_CONCEAL, HL_ONELINE, HL_SKIPEMPTY, HL_SKIPNL,
    HL_SKIPWHITE, HL_SYNC_HERE, HL_SYNC_THERE, HL_TRANSP, KEYWORD_IDX, SPO_HE_OFF, SPO_HS_OFF,
    SPO_ME_OFF, SPO_MS_OFF, SPO_RS_OFF, SPTYPE_MATCH, SPTYPE_START, SYNSPL_DEFAULT, SYNSPL_NOTOP,
    SYNSPL_TOP,
};

const MAXCOL: i32 = 0x7fff_ffff;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Current position
    fn nvim_syn_get_current_lnum() -> c_int;
    fn nvim_syn_get_current_col() -> c_int;
    fn nvim_syn_set_current_col(col: c_int);

    // Current state status
    fn nvim_syn_set_current_finished(finished: c_int);
    fn nvim_syn_set_state_stored(stored: c_int);

    // Current match attributes
    fn nvim_syn_set_current_attr(attr: c_int);
    fn nvim_syn_set_current_id(id: c_int);
    fn nvim_syn_set_current_trans_id(id: c_int);
    fn nvim_syn_set_current_flags(flags: c_int);
    fn nvim_syn_set_current_seqnr(seqnr: c_int);
    fn nvim_syn_set_current_sub_char(c: c_int);

    // State stack
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_is_current_state_empty() -> c_int;
    fn nvim_syn_get_stateitem(index: c_int) -> StateItemHandle;

    // State item accessors
    fn nvim_stateitem_get_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_id(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_trans_id(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_attr(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_flags(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_seqnr(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_cchar(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_cont_list(item: StateItemHandle) -> IdListHandle;
    fn nvim_stateitem_get_next_list(item: StateItemHandle) -> IdListHandle;
    fn nvim_stateitem_get_h_startpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_startpos_col(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_h_endpos_col(item: StateItemHandle) -> c_int;

    // State item setters
    fn nvim_stateitem_set_m_startcol(item: StateItemHandle, col: c_int);
    fn nvim_stateitem_set_h_startpos(item: StateItemHandle, lnum: c_int, col: c_int);
    fn nvim_stateitem_set_m_endpos(item: StateItemHandle, lnum: c_int, col: c_int);
    fn nvim_stateitem_set_h_endpos(item: StateItemHandle, lnum: c_int, col: c_int);
    fn nvim_stateitem_set_ends(item: StateItemHandle, ends: c_int);
    fn nvim_stateitem_set_end_idx(item: StateItemHandle, end_idx: c_int);
    fn nvim_stateitem_set_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_or_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_set_seqnr(item: StateItemHandle, seqnr: c_int);
    fn nvim_stateitem_set_cchar(item: StateItemHandle, cchar: c_int);
    fn nvim_stateitem_set_id(item: StateItemHandle, id: c_int);
    fn nvim_stateitem_set_trans_id(item: StateItemHandle, trans_id: c_int);
    fn nvim_stateitem_set_attr(item: StateItemHandle, attr: c_int);
    fn nvim_stateitem_set_cont_list(item: StateItemHandle, list: IdListHandle);
    fn nvim_stateitem_set_next_list(item: StateItemHandle, list: IdListHandle);

    // Next match
    fn nvim_syn_get_next_match_idx() -> c_int;
    fn nvim_syn_get_next_match_col() -> c_int;
    fn nvim_syn_set_next_match_idx(idx: c_int);
    fn nvim_syn_set_next_match_col(col: c_int);

    // Next match position getters
    fn nvim_syn_get_next_match_m_endpos(lnum: *mut c_int, col: *mut c_int);
    fn nvim_syn_get_next_match_h_endpos(lnum: *mut c_int, col: *mut c_int);

    // Next sequence number
    fn nvim_syn_incr_next_seqnr() -> c_int;

    // Next list management
    fn nvim_syn_has_current_next_list() -> c_int;
    fn nvim_syn_set_current_next_list(list: IdListHandle);
    fn nvim_syn_get_current_next_list() -> IdListHandle;
    fn nvim_syn_set_current_next_flags(flags: c_int);
    fn nvim_syn_get_current_next_flags() -> c_int;

    // Line operations
    fn nvim_syn_getcurline_byte_at(col: c_int) -> c_int;
    fn nvim_syn_update_ends(startofline: c_int);

    // Chartab
    fn nvim_syn_save_chartab(buf: *mut i8);
    fn nvim_syn_restore_chartab(buf: *mut i8);

    // Extmatch management
    fn nvim_syn_unref_extmatch(em: ExtMatchHandle);
    fn nvim_syn_take_re_extmatch_out() -> ExtMatchHandle;
    fn nvim_syn_clear_re_extmatch_out();

    // ID to attribute
    fn nvim_syn_id2attr_wrapper(syn_id: c_int) -> c_int;

    // Pattern accessors
    fn nvim_syn_get_pattern_ga_len() -> c_int;
    fn nvim_syn_get_pattern_type(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_flags(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_syncing(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_display(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_line_id(idx: c_int) -> c_int;
    fn nvim_syn_set_pattern_line_id(idx: c_int, line_id: c_int);
    fn nvim_syn_get_pattern_startcol(idx: c_int) -> c_int;
    fn nvim_syn_set_pattern_startcol(idx: c_int, col: c_int);
    fn nvim_syn_get_pattern_lc_off(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_next_list(idx: c_int) -> IdListHandle;
    fn nvim_syn_get_pattern_syn_match_id(idx: c_int) -> c_int;

    // Pattern sp_syn fields for containment check (Phase 2)
    fn nvim_syn_get_pattern_syn_id(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_sp_syn_inc_tag(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_sp_syn_cont_in_list(idx: c_int) -> IdListHandle;
    // (nvim_syn_check_pattern_containment inlined in Rust as check_pattern_containment())
    // (nvim_syn_regexec_by_idx replaced by crate::regexec::syn_regexec_by_idx)

    // Synblock queries
    fn nvim_syn_has_containedin() -> c_int;
    fn nvim_syn_has_keywords() -> c_int;
    fn nvim_syn_has_keywords_ic() -> c_int;

    // Line ID tracking
    fn nvim_syn_get_current_line_id() -> c_int;

    // Spell
    fn nvim_syn_get_spell_cluster_id() -> c_int;
    fn nvim_syn_get_nospell_cluster_id() -> c_int;
    fn nvim_syn_get_syn_spell() -> c_int;
    fn nvim_syn_in_id_list_spell(sip: StateItemHandle, list: IdListHandle, id: c_int) -> c_int;

    // Word check
    fn nvim_syn_vim_iswordp_buf(p: *mut i8) -> c_int;
    fn nvim_syn_utf_head_off(base: *mut i8, p: *mut i8) -> c_int;
    fn nvim_syn_ascii_iswhite(c: c_int) -> c_int;

    // getcurline
    fn nvim_syn_getcurline() -> *mut i8;
    fn nvim_syn_getcurline_len() -> c_int;

    // Push next match (uses existing rs_push_next_match from check_ends.rs)
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

    let syn_id = nvim_syn_get_pattern_syn_id(pat_idx);
    let inc_tag = nvim_syn_get_pattern_sp_syn_inc_tag(pat_idx);
    let cont_in_list = nvim_syn_get_pattern_sp_syn_cont_in_list(pat_idx);

    if has_next_list {
        let next_list = nvim_syn_get_current_next_list();
        rs_syn_in_id_list(
            StateItemHandle(std::ptr::null_mut()),
            next_list,
            syn_id,
            inc_tag,
            cont_in_list,
            0,
        )
    } else if !has_cur_si {
        let flags = nvim_syn_get_pattern_flags(pat_idx);
        if flags & HL_CONTAINED != 0 {
            0
        } else {
            1
        }
    } else {
        let cur_si = nvim_syn_get_stateitem(si_idx);
        let cont_list = nvim_stateitem_get_cont_list(cur_si);
        let flags = nvim_syn_get_pattern_flags(pat_idx);
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
    let current_lnum = nvim_syn_get_current_lnum();
    let current_col = nvim_syn_get_current_col();

    // No character, no attributes! Past end of line?
    // Do try matching with an empty line (could be the start of a region).
    let line_byte = nvim_syn_getcurline_byte_at(current_col);
    if line_byte == 0 && current_col != 0 {
        // If we found a match after the last column, use it.
        let next_idx = nvim_syn_get_next_match_idx();
        let next_col = nvim_syn_get_next_match_col();
        if next_idx >= 0 && next_col >= current_col && next_col != MAXCOL {
            push_next_match();
        }
        nvim_syn_set_current_finished(1);
        nvim_syn_set_state_stored(0);
        return 0;
    }

    // if the current or next character is NUL, we will finish the line now
    if line_byte == 0 || nvim_syn_getcurline_byte_at(current_col + 1) == 0 {
        nvim_syn_set_current_finished(1);
        nvim_syn_set_state_stored(0);
    }

    // When in the previous column there was a match but it could not be used
    // (empty match or already matched in this column) need to try again in
    // the next column.
    static mut TRY_NEXT_COLUMN: bool = false;
    if TRY_NEXT_COLUMN {
        nvim_syn_set_next_match_idx(-1);
        TRY_NEXT_COLUMN = false;
    }

    // Only check for keywords when not syncing and there are some.
    let do_keywords = !syncing && (nvim_syn_has_keywords() != 0 || nvim_syn_has_keywords_ic() != 0);

    // Init the list of zero-width matches with a nextlist.
    let mut zero_width_next_ga: Vec<i32> = Vec::new();

    // use syntax iskeyword option
    let mut buf_chartab = [0i8; 32];
    nvim_syn_save_chartab(buf_chartab.as_mut_ptr());

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
        let current_col = nvim_syn_get_current_col();

        // 1. Check for a current state.
        let state_len = nvim_syn_get_current_state_len();
        cur_si_valid = state_len > 0;

        if nvim_syn_has_containedin() != 0 || !cur_si_valid || {
            let si = nvim_syn_get_stateitem(state_len - 1);
            !nvim_stateitem_get_cont_list(si).0.is_null()
        } {
            // 2. Check for keywords
            if do_keywords {
                let line = nvim_syn_getcurline();
                let cur_pos = line.offset(current_col as isize);

                if nvim_syn_vim_iswordp_buf(cur_pos) != 0
                    && (current_col == 0 || {
                        let prev = cur_pos.offset(-1);
                        let head_off = nvim_syn_utf_head_off(line, prev);
                        let word_start = prev.offset(-(head_off as isize));
                        nvim_syn_vim_iswordp_buf(word_start) == 0
                    })
                {
                    let cur_si_handle = if cur_si_valid {
                        nvim_syn_get_stateitem(state_len - 1)
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
                        let new_len = nvim_syn_get_current_state_len();
                        let cur_si = nvim_syn_get_stateitem(new_len - 1);
                        nvim_stateitem_set_m_startcol(cur_si, current_col);
                        nvim_stateitem_set_h_startpos(cur_si, current_lnum, 0);
                        nvim_stateitem_set_m_endpos(cur_si, current_lnum, endcol);
                        nvim_stateitem_set_h_endpos(cur_si, current_lnum, endcol);
                        nvim_stateitem_set_ends(cur_si, 1);
                        nvim_stateitem_set_end_idx(cur_si, 0);
                        nvim_stateitem_set_flags(cur_si, flags);
                        nvim_stateitem_set_seqnr(cur_si, nvim_syn_incr_next_seqnr());
                        nvim_stateitem_set_cchar(cur_si, cchar);

                        if new_len > 1 {
                            let prev_si = nvim_syn_get_stateitem(new_len - 2);
                            let prev_flags = nvim_stateitem_get_flags(prev_si);
                            if prev_flags & HL_CONCEAL != 0 {
                                nvim_stateitem_or_flags(cur_si, HL_CONCEAL);
                            }
                        }

                        nvim_stateitem_set_id(cur_si, syn_id);
                        nvim_stateitem_set_trans_id(cur_si, syn_id);

                        if flags & HL_TRANSP != 0 {
                            if new_len < 2 {
                                nvim_stateitem_set_attr(cur_si, 0);
                                nvim_stateitem_set_trans_id(cur_si, 0);
                            } else {
                                let prev_si = nvim_syn_get_stateitem(new_len - 2);
                                nvim_stateitem_set_attr(cur_si, nvim_stateitem_get_attr(prev_si));
                                nvim_stateitem_set_trans_id(
                                    cur_si,
                                    nvim_stateitem_get_trans_id(prev_si),
                                );
                            }
                        } else {
                            nvim_stateitem_set_attr(cur_si, nvim_syn_id2attr_wrapper(syn_id));
                        }

                        nvim_stateitem_set_cont_list(cur_si, IdListHandle(std::ptr::null_mut()));
                        nvim_stateitem_set_next_list(cur_si, next_list);
                        check_keepend();
                        cur_si_valid = true;
                    }
                }
            }

            // 3. Check for patterns (only if no keyword found).
            if syn_id == 0 && nvim_syn_get_pattern_ga_len() > 0 {
                let current_col = nvim_syn_get_current_col();
                let next_match_idx = nvim_syn_get_next_match_idx();
                let next_match_col = nvim_syn_get_next_match_col();

                if next_match_idx < 0 || next_match_col < current_col {
                    // Check all relevant patterns for a match at this position.
                    nvim_syn_set_next_match_idx(0);
                    nvim_syn_set_next_match_col(MAXCOL);
                    let current_line_id = nvim_syn_get_current_line_id();
                    let state_len = nvim_syn_get_current_state_len();
                    let has_next_list = nvim_syn_has_current_next_list() != 0;
                    let si_idx = if cur_si_valid { state_len - 1 } else { -1 };

                    let pat_len = nvim_syn_get_pattern_ga_len();
                    for idx in (0..pat_len).rev() {
                        let pat_type = nvim_syn_get_pattern_type(idx);
                        let pat_syncing = nvim_syn_get_pattern_syncing(idx);
                        let pat_display = nvim_syn_get_pattern_display(idx);

                        if pat_syncing != (syncing as c_int) {
                            continue;
                        }
                        if !displaying && pat_display != 0 {
                            continue;
                        }
                        if pat_type != SPTYPE_MATCH && pat_type != SPTYPE_START {
                            continue;
                        }

                        if check_pattern_containment(idx, si_idx, has_next_list, cur_si_valid) == 0
                        {
                            continue;
                        }

                        let cur_next_match_col = nvim_syn_get_next_match_col();

                        // If we already tried matching in this line, and
                        // there isn't a match before next_match_col, skip.
                        if nvim_syn_get_pattern_line_id(idx) == current_line_id
                            && nvim_syn_get_pattern_startcol(idx) >= cur_next_match_col
                        {
                            continue;
                        }
                        nvim_syn_set_pattern_line_id(idx, current_line_id);

                        let mut lc_col = current_col - nvim_syn_get_pattern_lc_off(idx);
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
                            nvim_syn_set_pattern_startcol(idx, MAXCOL);
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
                            nvim_syn_set_pattern_startcol(idx, MAXCOL);
                            continue;
                        }
                        let startcol = pos.col;

                        nvim_syn_set_pattern_startcol(idx, startcol);

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

                        let pat_flags = nvim_syn_get_pattern_flags(idx);

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

                        // Store best match
                        crate::state_ops::rs_syn_set_next_match_state(
                            idx,
                            startcol,
                            endpos.lnum,
                            endpos.col,
                            hl_endpos.lnum,
                            hl_endpos.col,
                            hl_startpos.lnum,
                            hl_startpos.col,
                            flags,
                            eos_pos.lnum,
                            eos_pos.col,
                            eoe_pos.lnum,
                            eoe_pos.col,
                            end_idx,
                            cur_extmatch,
                        );
                        cur_extmatch = ExtMatchHandle(std::ptr::null_mut());
                    }
                }

                // If we found a match at the current column, use it.
                let current_col = nvim_syn_get_current_col();
                let next_match_idx = nvim_syn_get_next_match_idx();
                let next_match_col = nvim_syn_get_next_match_col();

                if next_match_idx >= 0 && next_match_col == current_col {
                    // When a zero-width item matched which has a nextgroup,
                    // don't push the item but set nextgroup.
                    let mut m_endpos_lnum: c_int = 0;
                    let mut m_endpos_col: c_int = 0;
                    nvim_syn_get_next_match_m_endpos(&mut m_endpos_lnum, &mut m_endpos_col);
                    let pat_next_list = nvim_syn_get_pattern_next_list(next_match_idx);

                    if m_endpos_lnum == current_lnum
                        && m_endpos_col == current_col
                        && !pat_next_list.0.is_null()
                    {
                        let pat_flags = nvim_syn_get_pattern_flags(next_match_idx);
                        nvim_syn_set_current_next_list(pat_next_list);
                        nvim_syn_set_current_next_flags(pat_flags);
                        keep_next_list = true;
                        zero_width_next_list = true;

                        zero_width_next_ga.push(next_match_idx);
                        nvim_syn_set_next_match_idx(-1);
                    } else {
                        push_next_match();
                        cur_si_valid = true;
                    }
                    found_match = true;
                }
            }
        }

        // Handle searching for nextgroup match.
        if nvim_syn_has_current_next_list() != 0 && !keep_next_list {
            if !found_match {
                let current_col = nvim_syn_get_current_col();
                let current_next_flags = nvim_syn_get_current_next_flags();
                let line_byte = nvim_syn_getcurline_byte_at(current_col);

                if (current_next_flags & HL_SKIPWHITE) != 0
                    && nvim_syn_ascii_iswhite(line_byte) != 0
                {
                    break;
                }
                if (current_next_flags & HL_SKIPEMPTY) != 0 && nvim_syn_getcurline_byte_at(0) == 0 {
                    break;
                }
            }

            nvim_syn_set_current_next_list(IdListHandle(std::ptr::null_mut()));
            nvim_syn_set_next_match_idx(-1);
            if !zero_width_next_list {
                found_match = true;
            }
        }

        if !found_match {
            break;
        }
    }

    nvim_syn_restore_chartab(buf_chartab.as_mut_ptr());

    // Use attributes from the current state, if within its highlighting.
    nvim_syn_set_current_attr(0);
    nvim_syn_set_current_id(0);
    nvim_syn_set_current_trans_id(0);
    nvim_syn_set_current_flags(0);
    nvim_syn_set_current_seqnr(0);

    let current_col = nvim_syn_get_current_col();
    let state_len = nvim_syn_get_current_state_len();

    let mut sip_handle = StateItemHandle(std::ptr::null_mut());

    if cur_si_valid && state_len > 0 {
        for idx in (0..state_len).rev() {
            let sip = nvim_syn_get_stateitem(idx);
            let h_start_lnum = nvim_stateitem_get_h_startpos_lnum(sip);
            let h_start_col = nvim_stateitem_get_h_startpos_col(sip);
            let h_end_lnum = nvim_stateitem_get_h_endpos_lnum(sip);
            let h_end_col = nvim_stateitem_get_h_endpos_col(sip);

            if (current_lnum > h_start_lnum
                || (current_lnum == h_start_lnum && current_col >= h_start_col))
                && (h_end_lnum == 0
                    || current_lnum < h_end_lnum
                    || (current_lnum == h_end_lnum && current_col < h_end_col))
            {
                nvim_syn_set_current_attr(nvim_stateitem_get_attr(sip));
                nvim_syn_set_current_id(nvim_stateitem_get_id(sip));
                nvim_syn_set_current_trans_id(nvim_stateitem_get_trans_id(sip));
                nvim_syn_set_current_flags(nvim_stateitem_get_flags(sip));
                nvim_syn_set_current_seqnr(nvim_stateitem_get_seqnr(sip));
                nvim_syn_set_current_sub_char(nvim_stateitem_get_cchar(sip));
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
            if nvim_syn_is_current_state_empty() == 0
                && nvim_syn_getcurline_byte_at(current_col) != 0
            {
                nvim_syn_set_current_col(current_col + 1);
                check_state_ends();
                nvim_syn_set_current_col(current_col);
            }
        }
    } else if !can_spell.is_null() {
        // Default: Only do spelling when there is no @Spell cluster or when
        // ":syn spell toplevel" was used.
        let syn_spell = nvim_syn_get_syn_spell();
        let spell_cluster = nvim_syn_get_spell_cluster_id();
        *can_spell = if syn_spell == SYNSPL_DEFAULT {
            (spell_cluster == 0) as c_int
        } else {
            (syn_spell == SYNSPL_TOP) as c_int
        };
    }

    // nextgroup ends at end of line, unless "skipnl" or "skipempty" present
    let current_col = nvim_syn_get_current_col();
    if nvim_syn_has_current_next_list() != 0
        && nvim_syn_getcurline_byte_at(current_col) != 0
        && nvim_syn_getcurline_byte_at(current_col + 1) == 0
        && (nvim_syn_get_current_next_flags() & (HL_SKIPNL | HL_SKIPEMPTY)) == 0
    {
        nvim_syn_set_current_next_list(IdListHandle(std::ptr::null_mut()));
    }

    // No longer need external matches. But keep next_match_extmatch.
    nvim_syn_clear_re_extmatch_out();
    nvim_syn_unref_extmatch(cur_extmatch);

    nvim_syn_get_current_attr()
}

extern "C" {
    fn nvim_syn_get_current_attr() -> c_int;
}

/// Compute can_spell based on spell clusters.
unsafe fn compute_can_spell(sip: StateItemHandle, can_spell: *mut c_int) {
    let spell_cluster = nvim_syn_get_spell_cluster_id();
    let nospell_cluster = nvim_syn_get_nospell_cluster_id();
    let syn_spell = nvim_syn_get_syn_spell();
    let current_trans_id = if sip.is_null() {
        0
    } else {
        // Read from the global (we just set it above)
        nvim_syn_get_current_trans_id()
    };

    if spell_cluster == 0 {
        // There is no @Spell cluster: Do spelling for items without @NoSpell.
        if nospell_cluster == 0 || current_trans_id == 0 {
            *can_spell = (syn_spell != SYNSPL_NOTOP) as c_int;
        } else {
            let cont_list = nvim_stateitem_get_cont_list(sip);
            *can_spell = (nvim_syn_in_id_list_spell(sip, cont_list, nospell_cluster) == 0) as c_int;
        }
    } else {
        // The @Spell cluster is defined.
        if current_trans_id == 0 {
            *can_spell = (syn_spell == SYNSPL_TOP) as c_int;
        } else {
            let cont_list = nvim_stateitem_get_cont_list(sip);
            *can_spell = nvim_syn_in_id_list_spell(sip, cont_list, spell_cluster);

            if nospell_cluster != 0
                && nvim_syn_in_id_list_spell(sip, cont_list, nospell_cluster) != 0
            {
                *can_spell = 0;
            }
        }
    }
}

extern "C" {
    fn nvim_syn_get_current_trans_id() -> c_int;
}

/// Check if pattern `idx` was already matched at the current column.
fn did_match_already(idx: i32, zero_width_ga: &[i32]) -> bool {
    let state_len = unsafe { nvim_syn_get_current_state_len() };
    let current_col = unsafe { nvim_syn_get_current_col() };
    let current_lnum = unsafe { nvim_syn_get_current_lnum() };

    for i in (0..state_len).rev() {
        let si = unsafe { nvim_syn_get_stateitem(i) };
        if unsafe { nvim_stateitem_get_m_startcol(si) } == current_col
            && unsafe { nvim_stateitem_get_m_lnum(si) } == current_lnum
            && unsafe { nvim_stateitem_get_idx(si) } == idx
        {
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

extern "C" {
    fn nvim_stateitem_get_m_lnum(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_m_startcol(item: StateItemHandle) -> c_int;
}

/// Process a syntax line to the end, checking for sync matches.
///
/// Returns true if a sync item is found.
pub unsafe fn syn_finish_line(syncing: bool) -> bool {
    while nvim_syn_is_current_finished() == 0 {
        syn_current_attr(syncing, false, std::ptr::null_mut(), false);

        // When syncing, and found some item, need to check the item.
        if syncing && nvim_syn_get_current_state_len() > 0 {
            let state_len = nvim_syn_get_current_state_len();
            let cur_si = nvim_syn_get_stateitem(state_len - 1);
            let si_idx = nvim_stateitem_get_idx(cur_si);
            if si_idx >= 0 {
                let pat_flags = nvim_syn_get_pattern_flags(si_idx);
                if (pat_flags & (HL_SYNC_HERE | HL_SYNC_THERE)) != 0 {
                    return true;
                }
            }

            // syn_current_attr() will have skipped the check for an item
            // that ends here, need to do that now.
            let current_col = nvim_syn_get_current_col();
            if nvim_syn_getcurline_byte_at(current_col) != 0 {
                nvim_syn_set_current_col(current_col + 1);
            }
            check_state_ends();
            nvim_syn_set_current_col(current_col);
        }

        let current_col = nvim_syn_get_current_col();
        nvim_syn_set_current_col(current_col + 1);
    }
    false
}

extern "C" {
    fn nvim_syn_is_current_finished() -> c_int;
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
