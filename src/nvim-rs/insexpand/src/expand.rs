//! Completion expansion search loop and dispatcher.
//!
//! This module provides Rust implementations for `ins_compl_get_exp` and
//! `get_next_completion_match`, which drive the main completion search loop
//! that iterates over 'complete' option entries and dispatches to source-
//! specific handlers.

#![allow(clippy::too_many_lines)]

use std::os::raw::c_int;

// Return value constants for process_next_cpt_value
const INS_COMPL_CPT_CONT: c_int = 2;
const INS_COMPL_CPT_END: c_int = 3;

// ctrl_x_mode constants (must match C enum)
const CTRL_X_WANT_IDENT: c_int = 0x100;
const CTRL_X_PATH_PATTERNS: c_int = 6 + CTRL_X_WANT_IDENT;
const CTRL_X_PATH_DEFINES: c_int = 7 + CTRL_X_WANT_IDENT;
const CTRL_X_DICTIONARY: c_int = 9 + CTRL_X_WANT_IDENT;
const CTRL_X_THESAURUS: c_int = 10 + CTRL_X_WANT_IDENT;
const CTRL_X_TAGS: c_int = 5 + CTRL_X_WANT_IDENT;
const CTRL_X_FILES: c_int = 4;
const CTRL_X_CMDLINE: c_int = 11;
const CTRL_X_CMDLINE_CTRL_X: c_int = 17;
const CTRL_X_FUNCTION: c_int = 12;
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_SPELL: c_int = 14;
const CTRL_X_BUFNAMES: c_int = 18;
const CTRL_X_REGISTER: c_int = 19;

// Return value constants
const FAIL: c_int = 0;
const OK: c_int = 1;

const COMPL_FUNC_TIMEOUT_MS: u64 = 300;
const COMPL_FUNC_TIMEOUT_NON_KW_MS: u64 = 1000;

extern "C" {
    // ctrl_x_mode queries
    fn rs_ctrl_x_mode_normal() -> c_int;
    fn rs_ctrl_x_mode_line_or_eval() -> c_int;
    fn rs_ctrl_x_mode_not_default() -> c_int;
    fn rs_ctrl_x_mode_spell() -> c_int;

    // direction
    fn rs_compl_dir_forward() -> c_int;

    // fuzzy / cot
    fn rs_cot_fuzzy() -> c_int;
    fn rs_is_nearest_active() -> c_int;
    fn rs_ins_compl_has_preinsert() -> c_int;

    // completion state
    fn nvim_get_compl_started() -> c_int;
    fn nvim_set_compl_started(val: c_int);
    fn nvim_get_compl_autocomplete() -> c_int;
    fn nvim_get_compl_from_nonkeyword() -> c_int;
    fn nvim_get_compl_time_slice_expired() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_num_bests() -> c_int;
    fn nvim_get_compl_get_longest() -> c_int;
    fn nvim_get_compl_timeout_ms() -> u64;
    fn nvim_set_compl_timeout_ms(val: u64);
    fn nvim_get_compl_pattern_is_null() -> c_int;

    // cpt sources index
    fn nvim_get_cpt_sources_index() -> c_int;
    fn nvim_set_cpt_sources_index(val: c_int);
    fn nvim_p_cto() -> c_int;
    fn nvim_get_p_act() -> c_int;
    fn nvim_normal_mode_strict() -> c_int;

    // cpt source timer
    fn rs_compl_source_start_timer(source_idx: c_int);
    fn rs_advance_cpt_sources_index_safe() -> c_int;
    fn nvim_set_compl_time_slice_expired(val: c_int);

    // ins_compl_st accessors
    fn nvim_ins_compl_get_exp_init_state(
        lnum: c_int,
        col: c_int,
        out_lnum: *mut c_int,
        out_col: *mut c_int,
    );
    fn nvim_ins_compl_get_exp_check_buf();
    fn nvim_ins_compl_st_set_cur_match_dir();
    fn nvim_ins_compl_st_e_cpt_is_nul() -> c_int;
    fn nvim_ins_compl_st_get_found_all() -> c_int;
    fn nvim_ins_compl_st_set_found_all(val: c_int);
    fn nvim_ins_compl_st_reset_set_match_pos();
    fn nvim_ins_compl_st_buf_valid() -> c_int;
    fn nvim_ins_compl_st_ins_buf_is_curbuf() -> c_int;
    fn nvim_ins_compl_st_mark_ins_buf_scanned();
    fn nvim_ins_compl_st_get_dict() -> *mut std::ffi::c_char;
    fn nvim_ins_compl_st_get_dict_f() -> c_int;
    fn nvim_ins_compl_st_clear_dict();
    fn nvim_ins_compl_st_get_func_cb() -> *mut std::ffi::c_void;
    fn nvim_ins_compl_st_get_first_lnum() -> c_int;

    // old_match / curr_match ops
    fn nvim_ins_compl_set_old_match_to_curr();
    fn nvim_compl_curr_vs_old_match_changed() -> c_int;
    fn nvim_compl_old_match_advance_curr();
    fn nvim_compl_curr_rewind_to_head();

    // process_next_cpt_value compound wrapper
    fn nvim_process_next_cpt_value_wrap(
        type_out: *mut c_int,
        start_lnum: c_int,
        start_col: c_int,
        fuzzy_collect: c_int,
        advance_out: *mut c_int,
    ) -> c_int;

    // get_next_default_completion compound wrapper
    fn nvim_get_next_default_completion_wrap(start_lnum: c_int, start_col: c_int) -> c_int;

    // get_next_filename_completion compound wrapper
    fn nvim_get_next_filename_completion_wrap();

    // expand_by_function wrapper
    fn nvim_expand_by_function_impl(compl_type: c_int);

    // cpt func completion matches
    fn rs_get_cpt_func_completion_matches(cb_opaque: *mut std::ffi::c_void);

    // other source dispatchers (all already in Rust or thin wrappers)
    fn rs_get_next_include_file_completion(compl_type: c_int);
    fn rs_get_next_dict_tsr_completion(
        compl_type: c_int,
        dict: *mut std::ffi::c_char,
        dict_f: c_int,
    );
    fn rs_get_next_tag_completion();
    fn rs_get_next_cmdline_completion();
    fn rs_get_next_spell_completion(lnum: c_int);
    fn rs_get_next_bufname_token();
    fn rs_get_register_completion();

    // match list / cyclic
    fn rs_ins_compl_make_cyclic() -> c_int;
    fn rs_fuzzy_longest_match();
    fn rs_ins_compl_fuzzy_sort();
    fn rs_ins_compl_leader_len() -> usize;
    fn rs_sort_compl_match_list(compare_type: c_int); // 1 = cp_compare_nearest

    // misc
    fn rs_ins_compl_check_keys(frequency: c_int, in_compl_func: c_int);
    fn nvim_got_int() -> c_int;
    fn may_trigger_modechanged();

}

/// Dispatch to the appropriate completion source for the given `type`.
///
/// Returns FAIL/OK depending on whether new matches were found.
/// This is a translation of the C `get_next_completion_match` function.
///
/// # Safety
/// Requires valid global completion state and `ins_compl_st` to be initialized.
unsafe fn get_next_completion_match(
    compl_type: c_int,
    start_lnum: c_int,
    start_col: c_int,
) -> c_int {
    let mut found_new_match = FAIL;

    match compl_type {
        -1 => {
            // no-op
        }
        t if t == CTRL_X_PATH_PATTERNS || t == CTRL_X_PATH_DEFINES => {
            rs_get_next_include_file_completion(t);
        }
        t if t == CTRL_X_DICTIONARY || t == CTRL_X_THESAURUS => {
            let dict = nvim_ins_compl_st_get_dict();
            let dict_f = nvim_ins_compl_st_get_dict_f();
            rs_get_next_dict_tsr_completion(t, dict, dict_f);
            nvim_ins_compl_st_clear_dict();
        }
        t if t == CTRL_X_TAGS => {
            rs_get_next_tag_completion();
        }
        t if t == CTRL_X_FILES => {
            nvim_get_next_filename_completion_wrap();
        }
        t if t == CTRL_X_CMDLINE || t == CTRL_X_CMDLINE_CTRL_X => {
            rs_get_next_cmdline_completion();
        }
        t if t == CTRL_X_FUNCTION => {
            if rs_ctrl_x_mode_normal() != 0 {
                // Invoked by a func in 'cpt' option
                let cb = nvim_ins_compl_st_get_func_cb();
                rs_get_cpt_func_completion_matches(cb);
            } else {
                nvim_expand_by_function_impl(t);
            }
        }
        t if t == CTRL_X_OMNI => {
            nvim_expand_by_function_impl(t);
        }
        t if t == CTRL_X_SPELL => {
            let first_lnum = nvim_ins_compl_st_get_first_lnum();
            rs_get_next_spell_completion(first_lnum);
        }
        t if t == CTRL_X_BUFNAMES => {
            rs_get_next_bufname_token();
        }
        t if t == CTRL_X_REGISTER => {
            rs_get_register_completion();
        }
        _ => {
            // normal ^P/^N and ^X^L
            found_new_match = nvim_get_next_default_completion_wrap(start_lnum, start_col);
            if found_new_match == FAIL && nvim_ins_compl_st_ins_buf_is_curbuf() != 0 {
                nvim_ins_compl_st_set_found_all(1);
            }
        }
    }

    // Check if compl_curr_match has changed (e.g. other type of expansion added something)
    if compl_type != 0 && nvim_compl_curr_vs_old_match_changed() != 0 {
        found_new_match = OK;
    }

    found_new_match
}

/// Get the next expansion(s), using `compl_pattern`.
///
/// The search starts at position `(lnum, col)` in curbuf and in the direction
/// `compl_direction`. When `compl_started` is false, start at that position;
/// otherwise continue where we stopped searching before.
///
/// This may return before finding all matches.
/// Returns the total number of matches or -1 if still unknown.
///
/// # Safety
/// Requires valid global completion state. Mutates many C static globals.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_get_exp(lnum: c_int, col: c_int) -> c_int {
    let mut found_new_match: c_int;
    let mut compl_type: c_int;
    let mut may_advance_cpt_idx: c_int = 0;

    let mut start_lnum = lnum;
    let mut start_col = col;

    // --- State initialization ---
    if nvim_get_compl_started() == 0 {
        // Initialize state for a fresh search
        nvim_ins_compl_get_exp_init_state(lnum, col, &raw mut start_lnum, &raw mut start_col);
    } else {
        // If the buffer was wiped out, fall back to curbuf
        nvim_ins_compl_get_exp_check_buf();
    }

    // Remember the last current match
    nvim_ins_compl_set_old_match_to_curr();

    // Set cur_match_pos based on direction
    nvim_ins_compl_st_set_cur_match_dir();

    // Determine if we are in "normal_mode_strict" and set up timer/timeout
    let normal_mode_strict = nvim_normal_mode_strict() != 0;
    if normal_mode_strict {
        nvim_set_cpt_sources_index(0);
        if nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0 {
            rs_compl_source_start_timer(0);
            nvim_set_compl_time_slice_expired(0);
            #[allow(clippy::cast_sign_loss)]
            let timeout_ms = if nvim_get_compl_autocomplete() != 0 {
                let p_act = nvim_get_p_act().max(0) as u64;
                let initial: u64 = 80; // COMPL_INITIAL_TIMEOUT_MS
                p_act.max(initial)
            } else {
                nvim_p_cto().max(0) as u64
            };
            nvim_set_compl_timeout_ms(timeout_ms);
        }
    }
    // compl_type starts as CTRL_X_NORMAL (0); process_next_cpt_value will update it
    compl_type = 0;

    // --- Main loop: iterate over 'complete' option entries ---
    loop {
        found_new_match = FAIL;
        nvim_ins_compl_st_reset_set_match_pos();

        // For ^N/^P pick a new entry from e_cpt if compl_started is off,
        // or if found_all says this entry is done. For ^X^L only use the
        // entries from 'complete' that look in loaded buffers.
        if (rs_ctrl_x_mode_normal() != 0 || rs_ctrl_x_mode_line_or_eval() != 0)
            && (nvim_get_compl_started() == 0 || nvim_ins_compl_st_get_found_all() != 0)
        {
            let mut new_type = compl_type;
            let status = nvim_process_next_cpt_value_wrap(
                &raw mut new_type,
                start_lnum,
                start_col,
                rs_cot_fuzzy(),
                &raw mut may_advance_cpt_idx,
            );
            compl_type = new_type;
            if status == INS_COMPL_CPT_END {
                break;
            }
            if status == INS_COMPL_CPT_CONT {
                if may_advance_cpt_idx != 0 {
                    if rs_advance_cpt_sources_index_safe() == 0 {
                        break;
                    }
                    rs_compl_source_start_timer(nvim_get_cpt_sources_index());
                }
                continue;
            }
        }

        // Save and possibly reduce timeout for function completions
        let compl_timeout_save: u64;
        if normal_mode_strict
            && compl_type == CTRL_X_FUNCTION
            && (nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0)
        {
            compl_timeout_save = nvim_get_compl_timeout_ms();
            let new_timeout = if nvim_get_compl_from_nonkeyword() != 0 {
                COMPL_FUNC_TIMEOUT_NON_KW_MS
            } else {
                COMPL_FUNC_TIMEOUT_MS
            };
            nvim_set_compl_timeout_ms(new_timeout);
        } else {
            compl_timeout_save = 0;
        }

        // Get the next set of completion matches
        found_new_match = get_next_completion_match(compl_type, start_lnum, start_col);

        // If complete() was called then compl_pattern has been reset. Bail out.
        if nvim_get_compl_pattern_is_null() != 0 {
            if normal_mode_strict
                && compl_type == CTRL_X_FUNCTION
                && (nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0)
            {
                nvim_set_compl_timeout_ms(compl_timeout_save);
            }
            break;
        }

        if may_advance_cpt_idx != 0 {
            if rs_advance_cpt_sources_index_safe() == 0 {
                if normal_mode_strict
                    && compl_type == CTRL_X_FUNCTION
                    && (nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0)
                {
                    nvim_set_compl_timeout_ms(compl_timeout_save);
                }
                break;
            }
            rs_compl_source_start_timer(nvim_get_cpt_sources_index());
        }

        // Break the loop for specialized modes or when we've found a new match
        if (rs_ctrl_x_mode_not_default() != 0 && rs_ctrl_x_mode_line_or_eval() == 0)
            || found_new_match != FAIL
        {
            if nvim_got_int() != 0 {
                if normal_mode_strict
                    && compl_type == CTRL_X_FUNCTION
                    && (nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0)
                {
                    nvim_set_compl_timeout_ms(compl_timeout_save);
                }
                break;
            }
            // Fill the popup menu as soon as possible.
            if compl_type != -1 {
                rs_ins_compl_check_keys(0, 0);
            }

            if (rs_ctrl_x_mode_not_default() != 0 && rs_ctrl_x_mode_line_or_eval() == 0)
                || nvim_get_compl_interrupted() != 0
            {
                if normal_mode_strict
                    && compl_type == CTRL_X_FUNCTION
                    && (nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0)
                {
                    nvim_set_compl_timeout_ms(compl_timeout_save);
                }
                break;
            }
            let not_expired = nvim_get_compl_time_slice_expired() == 0;
            nvim_set_compl_started(c_int::from(not_expired));
        } else {
            // Mark a buffer scanned when it has been scanned completely
            if nvim_ins_compl_st_buf_valid() != 0
                && (compl_type == 0 || compl_type == CTRL_X_PATH_PATTERNS)
            {
                nvim_ins_compl_st_mark_ins_buf_scanned();
            }
            nvim_set_compl_started(0);
        }

        // Restore the timeout after collecting matches from function source
        if normal_mode_strict
            && compl_type == CTRL_X_FUNCTION
            && (nvim_get_compl_autocomplete() != 0 || nvim_p_cto() > 0)
        {
            nvim_set_compl_timeout_ms(compl_timeout_save);
        }

        // For ^P completion, reset compl_curr_match to the head to avoid
        // mixing matches from different sources.
        if rs_compl_dir_forward() == 0 {
            nvim_compl_curr_rewind_to_head();
        }
    }

    // Reset cpt_sources_index and mark search as started
    nvim_set_cpt_sources_index(-1);
    nvim_set_compl_started(1);

    // Check if we reached the end of 'complete'
    if (rs_ctrl_x_mode_normal() != 0 || rs_ctrl_x_mode_line_or_eval() != 0)
        && nvim_ins_compl_st_e_cpt_is_nul() != 0
    {
        found_new_match = FAIL;
    }

    // Compute the total match count
    let match_count: c_int = if found_new_match == FAIL
        || (rs_ctrl_x_mode_not_default() != 0 && rs_ctrl_x_mode_line_or_eval() == 0)
    {
        rs_ins_compl_make_cyclic()
    } else {
        -1
    };

    // Fuzzy longest match post-processing
    if rs_cot_fuzzy() != 0 && nvim_get_compl_get_longest() != 0 && nvim_get_compl_num_bests() > 0 {
        rs_fuzzy_longest_match();
    }

    // Advance compl_curr_match past old_match
    nvim_compl_old_match_advance_curr();

    may_trigger_modechanged();

    // Sort matches if needed
    if match_count > 0 && rs_ctrl_x_mode_spell() == 0 {
        if rs_is_nearest_active() != 0 && rs_ins_compl_has_preinsert() == 0 {
            rs_sort_compl_match_list(1); // cp_compare_nearest
        }
        if rs_cot_fuzzy() != 0 && rs_ins_compl_leader_len() > 0 {
            rs_ins_compl_fuzzy_sort();
        }
    }

    match_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_constants() {
        assert_eq!(CTRL_X_FUNCTION, 12);
        assert_eq!(CTRL_X_OMNI, 13);
        assert_eq!(CTRL_X_SPELL, 14);
        assert_eq!(CTRL_X_FILES, 4);
        assert_eq!(CTRL_X_TAGS, 5 + CTRL_X_WANT_IDENT);
    }

    #[test]
    fn test_process_cpt_constants() {
        assert_eq!(INS_COMPL_CPT_CONT, 2);
        assert_eq!(INS_COMPL_CPT_END, 3);
    }

    #[test]
    fn test_timeout_constants() {
        assert_eq!(COMPL_FUNC_TIMEOUT_MS, 300);
        assert_eq!(COMPL_FUNC_TIMEOUT_NON_KW_MS, 1000);
    }
}
