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

// shortmess() argument for completion scan messages
const SHM_COMPLETIONSCAN: c_int = b'C' as c_int;
// highlight attributes
const HLF_R: c_int = 18;
const IOSIZE: usize = 1025;

// Constants for inlined nvim_ins_compl_st_do_search
const CONT_SOL: c_int = 16;
const SEARCH_NFMSG: c_int = 0x08;
const SEARCH_KEEP: c_int = 0x400;
const RE_LAST: c_int = 2;

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

    // (nvim_get_compl_pattern_is_null: inlined in vars.rs Phase 22)

    // nvim_p_cto: inlined in vars.rs (Phase 29)
    // nvim_get_p_act: inlined in vars.rs (Phase 29)
    // nvim_normal_mode_strict: deleted (Phase 1), inlined below

    // cpt source timer
    fn rs_compl_source_start_timer(source_idx: c_int);
    fn rs_advance_cpt_sources_index_safe() -> c_int;

    // ins_compl_st accessors
    // nvim_ins_compl_get_exp_init_state: deleted (Phase 21), inlined below
    // nvim_ins_compl_get_exp_check_buf: deleted (Phase 2), inlined below
    // nvim_ins_compl_st_set_cur_match_dir: deleted (Phase 1), inlined below
    // nvim_ins_compl_st_e_cpt_is_nul: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_get_found_all: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_set_found_all: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_reset_set_match_pos: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_buf_valid: deleted (Phase 1), inlined below
    // nvim_ins_compl_st_ins_buf_is_curbuf: deleted (Phase 1), inlined below
    #[link_name = "curbuf"]
    static curbuf_expand: *mut core::ffi::c_void;
    #[link_name = "buf_valid"]
    fn buf_valid_expand(buf: *mut core::ffi::c_void) -> bool;
    fn nvim_ins_compl_st_mark_ins_buf_scanned();
    // nvim_ins_compl_st_get_dict: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_get_dict_f: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_clear_dict: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_get_func_cb: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_get_first_lnum: inlined in vars.rs (Phase 26)

    // (nvim_compl_old_match_advance_curr and nvim_compl_curr_rewind_to_head: inlined in match_list.rs)

    // Phase 14 (Phase 3) accessors for rs_process_next_cpt_value
    fn nvim_curbuf_get_b_scanned() -> c_int;
    // nvim_ins_compl_st_get_e_cpt_char: inlined in vars.rs (Phase 27)
    // nvim_ins_compl_st_skip_delimiters: inlined in vars.rs (Phase 27)
    // nvim_ins_compl_st_set_dot_source: deleted (Phase 19), inlined below as ins_compl_st_set_dot_source
    // nvim_ins_compl_st_advance_buf: deleted (Phase 2), inlined below
    fn rs_ins_compl_next_buf(buf: nvim_buffer::BufHandle, flag: c_int) -> nvim_buffer::BufHandle;
    fn nvim_buf_has_ml_mfp_void(buf: *const core::ffi::c_void) -> bool;
    fn nvim_buf_ml_line_count(buf: *mut core::ffi::c_void) -> c_int;
    fn nvim_ins_compl_st_get_ins_buf_fname() -> *const std::ffi::c_char;
    // nvim_ins_compl_st_msg_scanning: deleted (Phase 18), inlined below as ins_compl_st_msg_scanning
    // nvim_ins_compl_st_msg_scanning_tags: deleted (Phase 2), inlined below
    // nvim_ins_compl_st_set_dict_from_e_cpt: inlined in vars.rs (Phase 27)
    // nvim_ins_compl_st_e_cpt_inc: inlined in vars.rs (Phase 27)
    // nvim_ins_compl_st_set_func_cb_from_e_cpt: deleted (Phase 2), inlined below
    // nvim_ins_compl_st_set_dict_from_ins_buf: deleted (Phase 2), inlined below
    // nvim_ins_compl_st_advance_e_cpt: deleted (Phase 2), inlined below

    // completion status
    fn rs_compl_status_adding() -> c_int;

    // Phase 4 accessors for rs_get_next_default_completion
    // nvim_compl_p_scs_save_set: deleted (Phase 18), inlined below as compl_p_scs_save_set
    // nvim_compl_p_ws_save_set: deleted (Phase 18), inlined below as compl_p_ws_save_set
    // nvim_compl_restore_p_scs_ws: deleted (Phase 2), inlined below
    // nvim_ins_compl_st_is_in_curbuf: deleted (Phase 1), inlined below
    // nvim_ins_compl_st_do_search: deleted (Phase 2), inlined below
    // nvim_ins_compl_st_check_and_update_match_pos: deleted (Phase 2), inlined below

    // helpers for inlined nvim_ins_compl_st_do_search
    #[link_name = "msg_silent"]
    static mut msg_silent_expand: c_int;
    fn rs_ins_compl_leader() -> *const std::ffi::c_char;
    fn rs_ctrl_x_mode_whole_line() -> c_int;
    fn rs_ctrl_x_mode_eval() -> c_int;
    fn search_for_fuzzy_match(
        buf: *mut core::ffi::c_void,
        pos: *mut crate::vars::PosT,
        pattern: *const std::ffi::c_char,
        dir: c_int,
        start_pos: *const crate::vars::PosT,
        len_out: *mut c_int,
        ptr_out: *mut *mut std::ffi::c_char,
        score_out: *mut c_int,
    ) -> bool;
    fn search_for_exact_line(
        buf: *mut core::ffi::c_void,
        pos: *mut crate::vars::PosT,
        dir: c_int,
        pat: *const std::ffi::c_char,
    ) -> c_int;
    fn searchit(
        win: *mut core::ffi::c_void,
        buf: *mut core::ffi::c_void,
        pos: *mut crate::vars::PosT,
        end_pos: *mut crate::vars::PosT,
        dir: c_int,
        pat: *const std::ffi::c_char,
        patlen: usize,
        count: c_int,
        options: c_int,
        pat_use: c_int,
        extra_arg: *mut core::ffi::c_void,
    ) -> c_int;

    // underlying helpers used by inlined wrappers
    fn rs_may_advance_cpt_index(cpt: *mut std::ffi::c_char) -> c_int;
    fn nvim_copy_option_part_iobuff_ffi(src: *mut *mut std::ffi::c_char) -> usize;
    fn nvim_get_callback_if_cpt_func_impl(
        p: *const std::ffi::c_char,
        idx: c_int,
    ) -> *mut core::ffi::c_void;
    #[link_name = "p_scs"]
    static mut p_scs_expand: c_int;
    #[link_name = "p_ws"]
    static mut p_ws_expand: c_int;
    // nvim_ins_compl_st_set_prev_from_cur: inlined in vars.rs (Phase 27)
    // nvim_ins_compl_st_get_cur_match_lnum: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_get_cur_match_col: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_get_prev_match_lnum: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_get_prev_match_col: inlined in vars.rs (Phase 26)
    // nvim_ins_compl_st_add_word_or_line: deleted (Phase 2), inlined below as rs_ins_compl_add_word_or_line

    // helpers for inlined rs_ins_compl_add_word_or_line
    fn nvim_ins_compl_ml_get_buf_at(
        buf: *mut core::ffi::c_void,
        lnum: c_int,
        col: c_int,
    ) -> *mut std::ffi::c_char;
    fn nvim_ml_get_buf_len(buf: *mut core::ffi::c_void, lnum: c_int) -> c_int;
    fn skipwhite(p: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn vim_iswordp(p: *const std::ffi::c_char) -> bool;
    fn rs_find_word_start(ptr: *mut std::ffi::c_char) -> *mut std::ffi::c_char;
    fn rs_find_word_end(ptr: *mut std::ffi::c_char) -> *mut std::ffi::c_char;
    fn strncpy(
        dst: *mut std::ffi::c_char,
        src: *const std::ffi::c_char,
        n: usize,
    ) -> *mut std::ffi::c_char;
    fn xstrlcpy(dst: *mut std::ffi::c_char, src: *const std::ffi::c_char, size: usize) -> usize;
    fn strcmp(s1: *const std::ffi::c_char, s2: *const std::ffi::c_char) -> c_int;
    fn nvim_ins_compl_add_infercase_ffi(
        str_: *const std::ffi::c_char,
        len: c_int,
        icase: c_int,
        fname: *const std::ffi::c_char,
        dir: c_int,
        cont_s_ipos: c_int,
        score: c_int,
    ) -> c_int;
    fn nvim_ins_compl_st_ins_buf_get_sfname() -> *const std::ffi::c_char;
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    #[link_name = "p_paste"]
    static p_paste_expand: c_int;
    #[link_name = "p_js"]
    static p_js_expand: c_int;
    #[link_name = "nvim_compl_match_get_next"]
    fn nvim_compl_match_get_next_expand(
        m: crate::match_list::ComplMatch,
    ) -> crate::match_list::ComplMatch;
    #[link_name = "nvim_compl_match_get_score"]
    fn nvim_compl_match_get_score_expand(m: crate::match_list::ComplMatch) -> c_int;

    // (nvim_get_next_filename_completion_wrap deleted Phase 15; call rs_get_next_filename_completion directly)

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
    #[link_name = "got_int"]
    static mut nvim_got_int: bool;
    fn may_trigger_modechanged();

    // helpers for inlined msg_scanning_tags
    fn shortmess(x: c_int) -> bool;
    fn msg_ext_set_kind(kind: *const std::ffi::c_char);
    fn msg_trunc(s: *mut std::ffi::c_char, force: bool, attr: c_int);
    fn vim_snprintf(s: *mut std::ffi::c_char, n: usize, fmt: *const std::ffi::c_char, ...)
        -> c_int;
    fn gettext(msgid: *const std::ffi::c_char) -> *const std::ffi::c_char;
    #[link_name = "msg_hist_off"]
    static mut msg_hist_off_expand: bool;
    #[link_name = "IObuff"]
    static mut IObuff_expand: [std::ffi::c_char; 1025];

    // helpers for inlined nvim_ins_compl_st_msg_scanning (Phase 18)
    fn nvim_buf_get_b_fname_void(buf: *mut core::ffi::c_void) -> *const std::ffi::c_char;
    fn nvim_buf_get_b_sfname_void(buf: *mut core::ffi::c_void) -> *const std::ffi::c_char;
    #[link_name = "rs_buf_spname"]
    fn buf_spname_void(buf: *mut core::ffi::c_void) -> *mut std::ffi::c_char;

    // helpers for inlined nvim_compl_p_scs_save_set (Phase 18)
    fn nvim_buf_get_b_p_inf_void(buf: *mut core::ffi::c_void) -> c_int;

    // helpers for inlined nvim_ins_compl_st_set_dot_source (Phase 19)
    fn dec(lp: *mut crate::vars::PosT) -> c_int;
    fn ml_get_len(lnum: c_int) -> c_int;

    // helpers for inlined nvim_ins_compl_get_exp_init_state (Phase 21)
    fn nvim_clear_all_buf_scanned();
    fn nvim_clear_ins_compl_st();
    static mut ins_compl_st_cleared: bool;
    fn xfree(p: *mut u8);
    fn xstrdup(s: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn nvim_curbuf_get_b_p_cpt() -> *const std::ffi::c_char;
    fn rs_strip_caret_numbers_in_place(s: *mut std::ffi::c_char);

}

// Return value constant for INS_COMPL_CPT_OK
const INS_COMPL_CPT_OK: c_int = 1;

/// Inlined from deleted C `nvim_ins_compl_st_msg_scanning` (Phase 18).
/// Emits the "Scanning: <name>" completion message for ins_compl_st.ins_buf.
///
/// # Safety
/// Requires valid ins_compl_st state.
unsafe fn ins_compl_st_msg_scanning() {
    if !shortmess(SHM_COMPLETIONSCAN) && crate::vars::nvim_get_compl_autocomplete() == 0 {
        msg_hist_off_expand = true;
        msg_ext_set_kind(c"completion".as_ptr());
        let ins_buf = crate::vars::ins_compl_st.ins_buf;
        let name_ptr: *const std::ffi::c_char = {
            let fname = nvim_buf_get_b_fname_void(ins_buf);
            if fname.is_null() {
                buf_spname_void(ins_buf).cast_const()
            } else {
                let sfname = nvim_buf_get_b_sfname_void(ins_buf);
                if sfname.is_null() {
                    fname
                } else {
                    sfname
                }
            }
        };
        vim_snprintf(
            core::ptr::addr_of_mut!(IObuff_expand).cast(),
            IOSIZE,
            c"Scanning: %s".as_ptr(),
            name_ptr,
        );
        msg_trunc(core::ptr::addr_of_mut!(IObuff_expand).cast(), true, HLF_R);
    }
}

/// Inlined from deleted C `nvim_compl_p_scs_save_set` (Phase 18).
/// Saves p_scs. If ins_compl_st.ins_buf has 'infercase' set, disables p_scs.
/// Returns the old p_scs value.
///
/// # Safety
/// Requires valid ins_compl_st state.
unsafe fn compl_p_scs_save_set() -> c_int {
    let save = p_scs_expand;
    if !crate::vars::ins_compl_st.ins_buf.is_null()
        && nvim_buf_get_b_p_inf_void(crate::vars::ins_compl_st.ins_buf) != 0
    {
        p_scs_expand = 0;
    }
    save
}

/// Inlined from deleted C `nvim_compl_p_ws_save_set` (Phase 18).
/// Saves p_ws and sets it based on curbuf and the current e_cpt entry.
/// Returns the old p_ws value.
///
/// # Safety
/// Requires valid ins_compl_st state.
unsafe fn compl_p_ws_save_set() -> c_int {
    let save = p_ws_expand;
    let in_curbuf = crate::vars::ins_compl_st.ins_buf == curbuf_expand;
    if !in_curbuf {
        p_ws_expand = 0;
    } else if !crate::vars::ins_compl_st.e_cpt.is_null() && *crate::vars::ins_compl_st.e_cpt == 46
    // b'.'
    {
        p_ws_expand = 1;
    }
    save
}

/// Inlined from deleted C `nvim_ins_compl_st_set_dot_source` (Phase 19).
/// Sets ins_compl_st to search the current buffer ('.' entry).
/// Returns 1 if the position wrapped to end-of-buffer, 0 otherwise.
///
/// # Safety
/// Requires valid ins_compl_st state and curbuf.
unsafe fn ins_compl_st_set_dot_source(
    start_lnum: c_int,
    start_col: c_int,
    fuzzy_collect: c_int,
) -> c_int {
    crate::vars::ins_compl_st.ins_buf = curbuf_expand;
    crate::vars::ins_compl_st.first_match_pos.lnum = start_lnum;
    crate::vars::ins_compl_st.first_match_pos.col = start_col;
    let wrapped = if rs_ctrl_x_mode_normal() != 0
        && fuzzy_collect == 0
        && dec(core::ptr::addr_of_mut!(
            crate::vars::ins_compl_st.first_match_pos
        )) < 0
    {
        let lnum = nvim_buf_ml_line_count(crate::vars::ins_compl_st.ins_buf);
        crate::vars::ins_compl_st.first_match_pos.lnum = lnum;
        crate::vars::ins_compl_st.first_match_pos.col = ml_get_len(lnum);
        1
    } else {
        0
    };
    crate::vars::ins_compl_st.last_match_pos = crate::vars::ins_compl_st.first_match_pos;
    crate::vars::ins_compl_st.set_match_pos = true;
    wrapped
}

/// Inlined from deleted C `nvim_ins_compl_get_exp_init_state` (Phase 21).
/// Initializes ins_compl_st for a fresh completion search starting at (lnum, col).
/// Clears b_scanned for all buffers, zeros ins_compl_st on first call,
/// sets up e_cpt_copy/e_cpt, adjusts start position for autocomplete,
/// and writes the effective start position back to *out_lnum / *out_col.
///
/// # Safety
/// Requires valid global completion state. Mutates ins_compl_st.
unsafe fn ins_compl_get_exp_init_state(
    lnum: c_int,
    col: c_int,
    out_lnum: *mut c_int,
    out_col: *mut c_int,
) {
    nvim_clear_all_buf_scanned();
    if !ins_compl_st_cleared {
        nvim_clear_ins_compl_st();
        ins_compl_st_cleared = true;
    }
    crate::vars::ins_compl_st.found_all = false;
    crate::vars::ins_compl_st.ins_buf = curbuf_expand;
    xfree(crate::vars::ins_compl_st.e_cpt_copy.cast());
    // CONT_LOCAL = 32
    let cpt_src: *const std::ffi::c_char = if (crate::vars::nvim_get_compl_cont_status() & 32) != 0
    {
        c".".as_ptr()
    } else {
        nvim_curbuf_get_b_p_cpt()
    };
    crate::vars::ins_compl_st.e_cpt_copy = xstrdup(cpt_src);
    rs_strip_caret_numbers_in_place(crate::vars::ins_compl_st.e_cpt_copy);
    crate::vars::ins_compl_st.e_cpt = crate::vars::ins_compl_st.e_cpt_copy;

    let mut start_pos = crate::vars::PosT {
        lnum,
        col,
        coladd: 0,
    };
    if crate::vars::nvim_get_compl_autocomplete() != 0 && rs_is_nearest_active() != 0 {
        const LOOKBACK_LINE_COUNT: c_int = 1000;
        start_pos.lnum = if start_pos.lnum - LOOKBACK_LINE_COUNT > 1 {
            start_pos.lnum - LOOKBACK_LINE_COUNT
        } else {
            1
        };
        start_pos.col = 0;
    }
    crate::vars::ins_compl_st.first_match_pos = start_pos;
    crate::vars::ins_compl_st.last_match_pos = start_pos;

    *out_lnum = start_pos.lnum;
    *out_col = start_pos.col;
}

/// Process the next 'complete' option value in ins_compl_st.e_cpt.
///
/// This is a Rust translation of the C `process_next_cpt_value` function.
///
/// Returns INS_COMPL_CPT_OK / INS_COMPL_CPT_CONT / INS_COMPL_CPT_END.
/// Sets `*compl_type_out` to the completion type for this entry.
/// Sets `*advance_cpt_idx_out` if the cpt sources index should advance.
///
/// # Safety
/// Requires valid ins_compl_st state. Mutates ins_compl_st fields via accessors.
unsafe fn rs_process_next_cpt_value(
    start_lnum: c_int,
    start_col: c_int,
    fuzzy_collect: c_int,
    compl_type_out: &mut c_int,
    advance_cpt_idx_out: &mut c_int,
) -> c_int {
    let mut compl_type: c_int = -1;
    let mut status = INS_COMPL_CPT_OK;
    let skip_source = crate::vars::nvim_get_compl_autocomplete() != 0
        && crate::vars::nvim_get_compl_from_nonkeyword() != 0;

    crate::vars::nvim_ins_compl_st_set_found_all(0);
    *advance_cpt_idx_out = 0;

    // Skip leading commas and spaces
    crate::vars::nvim_ins_compl_st_skip_delimiters();

    // crate::vars::nvim_ins_compl_st_get_e_cpt_char() returns an ASCII char value (0-127)
    // or 0 for NUL. Truncation from i32 to u8 is safe for valid ASCII.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let e_char = crate::vars::nvim_ins_compl_st_get_e_cpt_char() as u8;

    if e_char == b'.'
        && nvim_curbuf_get_b_scanned() == 0
        && !skip_source
        && crate::vars::nvim_get_compl_time_slice_expired() == 0
    {
        // Current buffer ('.' entry)
        ins_compl_st_set_dot_source(start_lnum, start_col, fuzzy_collect);
        compl_type = 0;
        // set_match_pos is set inside ins_compl_st_set_dot_source
    } else if !skip_source
        && crate::vars::nvim_get_compl_time_slice_expired() == 0
        && matches!(e_char, b'b' | b'u' | b'w' | b'U')
    {
        // Buffer/window scan ('b', 'u', 'w', 'U' entries)
        // Inline nvim_ins_compl_st_advance_buf (Phase 2)
        let result = {
            let next = rs_ins_compl_next_buf(
                nvim_buffer::BufHandle::from_ptr(crate::vars::ins_compl_st.ins_buf),
                c_int::from(e_char),
            );
            if next.as_ptr() == curbuf_expand {
                0
            } else {
                crate::vars::ins_compl_st.ins_buf = next.as_ptr();
                if nvim_buf_has_ml_mfp_void(crate::vars::ins_compl_st.ins_buf.cast_const()) {
                    crate::vars::nvim_set_compl_started(1);
                    crate::vars::ins_compl_st.first_match_pos.col = 0;
                    crate::vars::ins_compl_st.last_match_pos.col = 0;
                    crate::vars::ins_compl_st.first_match_pos.lnum =
                        nvim_buf_ml_line_count(crate::vars::ins_compl_st.ins_buf) + 1;
                    crate::vars::ins_compl_st.last_match_pos.lnum = 0;
                    2
                } else {
                    crate::vars::ins_compl_st.found_all = true;
                    1
                }
            }
        };
        if result == 0 {
            // No new buffer found (wrapped back to curbuf) -- skip
            status = INS_COMPL_CPT_CONT;
        } else if result == 2 {
            // Loaded buffer
            compl_type = 0;
            ins_compl_st_msg_scanning();
        } else {
            // Unloaded buffer (result == 1): scan like dictionary
            if nvim_ins_compl_st_get_ins_buf_fname().is_null() {
                status = INS_COMPL_CPT_CONT;
            } else {
                // Inline nvim_ins_compl_st_set_dict_from_ins_buf (Phase 2)
                crate::vars::ins_compl_st.dict = nvim_ins_compl_st_get_ins_buf_fname().cast_mut();
                crate::vars::ins_compl_st.dict_f = 2; // DICT_EXACT
                compl_type = CTRL_X_DICTIONARY;
                ins_compl_st_msg_scanning();
            }
        }
    } else if e_char == 0 {
        // NUL: end of 'complete' option
        status = INS_COMPL_CPT_END;
    } else {
        // Other entries: 'F'/'o', 'k'/'s', 'i', 'd', 'f', ']'/'t'
        if rs_ctrl_x_mode_line_or_eval() != 0 {
            // compl_type = -1 (leave as default)
        } else if e_char == b'F' || e_char == b'o' {
            compl_type = CTRL_X_FUNCTION;
            let idx = crate::vars::nvim_get_cpt_sources_index();
            // Inline nvim_ins_compl_st_set_func_cb_from_e_cpt (Phase 2)
            crate::vars::ins_compl_st.func_cb = nvim_get_callback_if_cpt_func_impl(
                crate::vars::ins_compl_st.e_cpt.cast_const(),
                idx,
            );
            if crate::vars::ins_compl_st.func_cb.is_null() {
                compl_type = -1;
            }
        } else if !skip_source {
            match e_char {
                b'k' | b's' => {
                    compl_type = if e_char == b'k' {
                        CTRL_X_DICTIONARY
                    } else {
                        CTRL_X_THESAURUS
                    };
                    // Check if there's a specific dict/thesaurus path
                    crate::vars::nvim_ins_compl_st_e_cpt_inc();
                    // ASCII char value; truncation from i32 to u8 is safe.
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let next_c = crate::vars::nvim_ins_compl_st_get_e_cpt_char() as u8;
                    if next_c != b',' && next_c != 0 {
                        crate::vars::nvim_ins_compl_st_set_dict_from_e_cpt();
                    }
                }
                b'i' => {
                    compl_type = CTRL_X_PATH_PATTERNS;
                }
                b'd' => {
                    compl_type = CTRL_X_PATH_DEFINES;
                }
                b'f' => {
                    compl_type = CTRL_X_BUFNAMES;
                }
                b']' | b't' => {
                    compl_type = CTRL_X_TAGS;
                    // Inline nvim_ins_compl_st_msg_scanning_tags (Phase 2)
                    if !shortmess(SHM_COMPLETIONSCAN)
                        && crate::vars::nvim_get_compl_autocomplete() == 0
                    {
                        msg_ext_set_kind(c"completion".as_ptr());
                        msg_hist_off_expand = true;
                        vim_snprintf(
                            core::ptr::addr_of_mut!(IObuff_expand).cast(),
                            IOSIZE,
                            c"%s".as_ptr(),
                            gettext(c"Scanning tags.".as_ptr()),
                        );
                        msg_trunc(core::ptr::addr_of_mut!(IObuff_expand).cast(), true, HLF_R);
                    }
                }
                _ => {}
            }
        }

        // Inline nvim_ins_compl_st_advance_e_cpt (Phase 2):
        // copy_option_part(&ins_compl_st.e_cpt, IObuff, IOSIZE, ",")
        // then check rs_may_advance_cpt_index
        nvim_copy_option_part_iobuff_ffi(core::ptr::addr_of_mut!(crate::vars::ins_compl_st.e_cpt));
        *advance_cpt_idx_out =
            c_int::from(rs_may_advance_cpt_index(crate::vars::ins_compl_st.e_cpt) != 0);

        crate::vars::nvim_ins_compl_st_set_found_all(1);
        if compl_type == -1 {
            status = INS_COMPL_CPT_CONT;
        }
    }

    *compl_type_out = compl_type;
    status
}

// nvim_ins_compl_st_do_search inlined above; CONT_SOL now checked directly.

// NOTDONE constant for ins_compl_add_infercase return value
const NOTDONE: c_int = -1;
const TAB: c_int = 9;

/// Rust translation of nvim_ins_compl_st_add_word_or_line (Phase 2).
///
/// Attempt to add the word or line at the current match position to the
/// completion list.  When not in fuzzy mode, inlines ins_compl_get_next_word_or_line
/// logic first; then calls ins_compl_add_infercase.
///
/// Returns:
///   0  ptr is NULL or preinsert-skip (caller should continue loop)
///   1  ins_compl_add_infercase returned NOTDONE (duplicate)
///   2  match successfully added (caller should break)
///
/// # Safety
/// Requires valid global completion state.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]
unsafe fn rs_ins_compl_add_word_or_line(
    in_fuzzy: c_int,
    fuzzy_ptr: *mut std::ffi::c_char,
    fuzzy_len: c_int,
    fuzzy_score: c_int,
) -> c_int {
    let mut ptr = fuzzy_ptr;
    let mut len = fuzzy_len;
    let mut score = fuzzy_score;
    let mut cont_s_ipos = false;

    if in_fuzzy == 0 {
        // Inlined ins_compl_get_next_word_or_line logic:
        let ins_buf = crate::vars::ins_compl_st.ins_buf;
        let cur_lnum = crate::vars::nvim_ins_compl_st_get_cur_match_lnum();
        let cur_col = crate::vars::nvim_ins_compl_st_get_cur_match_col();

        // Use labeled block to simulate 'goto add_word_check'
        'add_word_check: {
            ptr = nvim_ins_compl_ml_get_buf_at(ins_buf, cur_lnum, cur_col);
            let raw_len = nvim_ml_get_buf_len(ins_buf, cur_lnum) - cur_col;
            len = raw_len;

            if rs_ctrl_x_mode_line_or_eval() != 0 {
                if rs_compl_status_adding() != 0 {
                    if cur_lnum >= nvim_buf_ml_line_count(ins_buf) {
                        ptr = core::ptr::null_mut();
                        break 'add_word_check;
                    }
                    ptr = nvim_ins_compl_ml_get_buf_at(ins_buf, cur_lnum + 1, 0);
                    len = nvim_ml_get_buf_len(ins_buf, cur_lnum + 1);
                    if p_paste_expand == 0 {
                        let tmp_ptr = ptr;
                        ptr = skipwhite(tmp_ptr);
                        len -= ptr.offset_from(tmp_ptr) as c_int;
                    }
                }
            } else {
                let mut tmp_ptr = ptr;
                let compl_length = crate::vars::nvim_get_compl_length();
                if rs_compl_status_adding() != 0 && compl_length <= len {
                    tmp_ptr = tmp_ptr.add(compl_length as usize);
                    if vim_iswordp(tmp_ptr) {
                        ptr = core::ptr::null_mut();
                        break 'add_word_check;
                    }
                    tmp_ptr = rs_find_word_start(tmp_ptr);
                }
                tmp_ptr = rs_find_word_end(tmp_ptr);
                len = tmp_ptr.offset_from(ptr) as c_int;
                if rs_compl_status_adding() != 0 && len == compl_length {
                    let line_count = nvim_buf_ml_line_count(ins_buf);
                    if cur_lnum < line_count {
                        strncpy(
                            core::ptr::addr_of_mut!(IObuff_expand).cast(),
                            ptr,
                            len as usize,
                        );
                        ptr = nvim_ins_compl_ml_get_buf_at(ins_buf, cur_lnum + 1, 0);
                        tmp_ptr = skipwhite(ptr);
                        ptr = tmp_ptr;
                        tmp_ptr = rs_find_word_start(tmp_ptr);
                        tmp_ptr = rs_find_word_end(tmp_ptr);
                        if tmp_ptr > ptr {
                            let iobuff: *mut std::ffi::c_char =
                                core::ptr::addr_of_mut!(IObuff_expand).cast();
                            if *ptr != b')' as std::ffi::c_char
                                && *iobuff.add((len - 1) as usize) != TAB as std::ffi::c_char
                            {
                                if *iobuff.add((len - 1) as usize) != b' ' as std::ffi::c_char {
                                    *iobuff.add(len as usize) = b' ' as std::ffi::c_char;
                                    len += 1;
                                }
                                if p_js_expand != 0 {
                                    let prev = *iobuff.add((len - 2) as usize);
                                    if prev == b'.' as std::ffi::c_char
                                        || prev == b'?' as std::ffi::c_char
                                        || prev == b'!' as std::ffi::c_char
                                    {
                                        *iobuff.add(len as usize) = b' ' as std::ffi::c_char;
                                        len += 1;
                                    }
                                }
                            }
                            let extra = tmp_ptr.offset_from(ptr) as usize;
                            let remaining = IOSIZE - len as usize;
                            let capped_extra = if extra >= remaining {
                                remaining - 1
                            } else {
                                extra
                            };
                            xstrlcpy(iobuff.add(len as usize), ptr, IOSIZE - len as usize);
                            len += capped_extra as c_int;
                            cont_s_ipos = true;
                        }
                        *core::ptr::addr_of_mut!(IObuff_expand)
                            .cast::<std::ffi::c_char>()
                            .add(len as usize) = 0;
                        ptr = core::ptr::addr_of_mut!(IObuff_expand).cast();
                    }
                    if len == compl_length {
                        ptr = core::ptr::null_mut();
                        break 'add_word_check;
                    }
                }
            }
        } // end 'add_word_check block
    }

    // add_word_check:
    if ptr.is_null()
        || (rs_ins_compl_has_preinsert() != 0 && strcmp(ptr, rs_ins_compl_leader()) == 0)
    {
        return 0;
    }

    if rs_is_nearest_active() != 0 && crate::vars::ins_compl_st.ins_buf == curbuf_expand {
        score = crate::vars::nvim_ins_compl_st_get_cur_match_lnum() - nvim_get_curwin_cursor_lnum();
        if score < 0 {
            score = -score;
        }
    }

    let sfname = nvim_ins_compl_st_ins_buf_get_sfname();

    let add_r = nvim_ins_compl_add_infercase_ffi(
        ptr,
        len,
        crate::vars::nvim_get_p_ic(),
        sfname,
        0,
        c_int::from(cont_s_ipos),
        score,
    );
    if add_r == NOTDONE {
        return 1;
    }
    // add_r is OK (or FAIL which shouldn't happen here)
    if in_fuzzy != 0 {
        let first = crate::match_list::compl_first_match;
        if !first.is_null() {
            let next = nvim_compl_match_get_next_expand(first);
            if !next.is_null() && score == nvim_compl_match_get_score_expand(next) {
                crate::vars::nvim_set_compl_num_bests(crate::vars::nvim_get_compl_num_bests() + 1);
            }
        }
    }
    2
}

/// Rust translation of get_next_default_completion.
///
/// Searches `ins_compl_st.ins_buf` for the next match of `compl_pattern`,
/// starting from `start_pos`, adding any found word/line to the completion
/// list via `ins_compl_add_infercase`.
///
/// Returns OK if a new match was added, FAIL otherwise.
///
/// # Safety
/// Requires valid `ins_compl_st` and global completion state.
unsafe fn rs_get_next_default_completion(start_lnum: c_int, start_col: c_int) -> c_int {
    let in_fuzzy = c_int::from(
        rs_compl_status_adding() == 0
            && rs_cot_fuzzy() != 0
            && crate::vars::nvim_get_compl_length() > 0,
    );
    let in_curbuf = crate::vars::ins_compl_st.ins_buf == curbuf_expand;

    // Save and conditionally modify p_scs and p_ws.
    let save_p_scs = compl_p_scs_save_set();
    let save_p_ws = compl_p_ws_save_set();

    let mut looped_around = false;
    let mut found_new_match = FAIL;

    loop {
        // fuzzy search outputs
        let mut fuzzy_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
        let mut fuzzy_len: c_int = 0;
        #[allow(clippy::cast_possible_wrap)]
        let mut fuzzy_score: c_int = i32::MIN; // FUZZY_SCORE_NONE

        // Inline nvim_ins_compl_st_do_search (Phase 2)
        let mut found = {
            msg_silent_expand += 1;
            let start_pos = crate::vars::PosT {
                lnum: start_lnum,
                col: start_col,
                coladd: 0,
            };
            let dir = crate::vars::nvim_get_compl_direction();
            let r = if in_fuzzy != 0 {
                let leader = rs_ins_compl_leader();
                c_int::from(search_for_fuzzy_match(
                    crate::vars::ins_compl_st.ins_buf,
                    crate::vars::ins_compl_st.cur_match_pos,
                    leader,
                    dir,
                    &raw const start_pos,
                    &raw mut fuzzy_len,
                    &raw mut fuzzy_ptr,
                    &raw mut fuzzy_score,
                ))
            } else if rs_ctrl_x_mode_whole_line() != 0
                || rs_ctrl_x_mode_eval() != 0
                || (crate::vars::nvim_get_compl_cont_status() & CONT_SOL) != 0
            {
                search_for_exact_line(
                    crate::vars::ins_compl_st.ins_buf,
                    crate::vars::ins_compl_st.cur_match_pos,
                    dir,
                    crate::vars::compl_pattern.data,
                )
            } else {
                searchit(
                    core::ptr::null_mut(),
                    crate::vars::ins_compl_st.ins_buf,
                    crate::vars::ins_compl_st.cur_match_pos,
                    core::ptr::null_mut(),
                    dir,
                    crate::vars::compl_pattern.data,
                    crate::vars::compl_pattern.size,
                    1,
                    SEARCH_KEEP + SEARCH_NFMSG,
                    RE_LAST,
                    core::ptr::null_mut(),
                )
            };
            msg_silent_expand -= 1;
            r
        };

        // Check / update match positions.
        // check == 0: first-time/set_match_pos (positions set; found stays as-is)
        // check == -1: first==last → force FAIL
        // check == 2: normal; run wrap-around detection
        // Inline nvim_ins_compl_st_check_and_update_match_pos (Phase 2)
        let check = if crate::vars::nvim_get_compl_started() == 0
            || crate::vars::ins_compl_st.set_match_pos
        {
            crate::vars::nvim_set_compl_started(1);
            crate::vars::ins_compl_st.first_match_pos = *crate::vars::ins_compl_st.cur_match_pos;
            crate::vars::ins_compl_st.last_match_pos = *crate::vars::ins_compl_st.cur_match_pos;
            crate::vars::ins_compl_st.set_match_pos = false;
            0
        } else if crate::vars::ins_compl_st.first_match_pos.lnum
            == crate::vars::ins_compl_st.last_match_pos.lnum
            && crate::vars::ins_compl_st.first_match_pos.col
                == crate::vars::ins_compl_st.last_match_pos.col
        {
            -1
        } else {
            2
        };
        if check == -1 {
            found = FAIL;
        } else if check == 2 {
            // Wrap-around detection
            let cur_lnum = crate::vars::nvim_ins_compl_st_get_cur_match_lnum();
            let cur_col = crate::vars::nvim_ins_compl_st_get_cur_match_col();
            let prev_lnum = crate::vars::nvim_ins_compl_st_get_prev_match_lnum();
            let prev_col = crate::vars::nvim_ins_compl_st_get_prev_match_col();
            if rs_compl_dir_forward() != 0 {
                if prev_lnum > cur_lnum || (prev_lnum == cur_lnum && prev_col >= cur_col) {
                    if looped_around {
                        found = FAIL;
                    } else {
                        looped_around = true;
                    }
                }
            } else if prev_lnum < cur_lnum || (prev_lnum == cur_lnum && prev_col <= cur_col) {
                if looped_around {
                    found = FAIL;
                } else {
                    looped_around = true;
                }
            }
        }

        crate::vars::nvim_ins_compl_st_set_prev_from_cur();

        if found == FAIL {
            break;
        }

        // Skip if ADDING and position matches start_pos (cursor position).
        if rs_compl_status_adding() != 0
            && in_curbuf
            && crate::vars::nvim_ins_compl_st_get_cur_match_lnum() == start_lnum
            && crate::vars::nvim_ins_compl_st_get_cur_match_col() == start_col
        {
            continue;
        }

        // Try to add the word/line at the current match position.
        // Returns: 0=skip(ptr null/preinsert), 1=NOTDONE(dup), 2=added.
        let add_result = rs_ins_compl_add_word_or_line(in_fuzzy, fuzzy_ptr, fuzzy_len, fuzzy_score);
        if add_result >= 2 {
            // successfully added
            found_new_match = OK;
            break;
        }
        // 0 or 1: skip/duplicate — continue the search loop
    }

    // Inline nvim_compl_restore_p_scs_ws (Phase 2)
    p_scs_expand = c_int::from(save_p_scs != 0);
    p_ws_expand = c_int::from(save_p_ws != 0);
    found_new_match
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
            let dict = crate::vars::nvim_ins_compl_st_get_dict();
            let dict_f = crate::vars::nvim_ins_compl_st_get_dict_f();
            rs_get_next_dict_tsr_completion(t, dict, dict_f);
            crate::vars::nvim_ins_compl_st_clear_dict();
        }
        t if t == CTRL_X_TAGS => {
            rs_get_next_tag_completion();
        }
        t if t == CTRL_X_FILES => {
            crate::file::rs_get_next_filename_completion();
        }
        t if t == CTRL_X_CMDLINE || t == CTRL_X_CMDLINE_CTRL_X => {
            rs_get_next_cmdline_completion();
        }
        t if t == CTRL_X_FUNCTION => {
            if rs_ctrl_x_mode_normal() != 0 {
                // Invoked by a func in 'cpt' option
                let cb = crate::vars::nvim_ins_compl_st_get_func_cb();
                rs_get_cpt_func_completion_matches(cb);
            } else {
                nvim_expand_by_function_impl(t);
            }
        }
        t if t == CTRL_X_OMNI => {
            nvim_expand_by_function_impl(t);
        }
        t if t == CTRL_X_SPELL => {
            let first_lnum = crate::vars::nvim_ins_compl_st_get_first_lnum();
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
            found_new_match = rs_get_next_default_completion(start_lnum, start_col);
            if found_new_match == FAIL && crate::vars::ins_compl_st.ins_buf == curbuf_expand {
                crate::vars::nvim_ins_compl_st_set_found_all(1);
            }
        }
    }

    // Check if compl_curr_match has changed (e.g. other type of expansion added something)
    if compl_type != 0
        && crate::match_list::nvim_compl_get_curr_match()
            != crate::match_list::nvim_compl_get_old_match()
    {
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
    if crate::vars::nvim_get_compl_started() == 0 {
        // Initialize state for a fresh search
        ins_compl_get_exp_init_state(lnum, col, &raw mut start_lnum, &raw mut start_col);
    } else {
        // Inline nvim_ins_compl_get_exp_check_buf (Phase 2):
        // If the buffer was wiped out, fall back to curbuf
        if crate::vars::ins_compl_st.ins_buf != curbuf_expand
            && !buf_valid_expand(crate::vars::ins_compl_st.ins_buf)
        {
            crate::vars::ins_compl_st.ins_buf = curbuf_expand;
        }
    }

    // Remember the last current match
    crate::match_list::nvim_compl_set_old_match(crate::match_list::nvim_compl_get_curr_match());

    // Set cur_match_pos based on direction
    // Inline nvim_ins_compl_st_set_cur_match_dir (Phase 1)
    crate::vars::ins_compl_st.cur_match_pos = if rs_compl_dir_forward() != 0 {
        core::ptr::addr_of_mut!(crate::vars::ins_compl_st.last_match_pos)
    } else {
        core::ptr::addr_of_mut!(crate::vars::ins_compl_st.first_match_pos)
    };

    // Determine if we are in "normal_mode_strict" and set up timer/timeout
    // (Inline of deleted nvim_normal_mode_strict: Phase 1)
    // CONT_LOCAL = 32 (from insexpand.h)
    let normal_mode_strict = rs_ctrl_x_mode_normal() != 0
        && rs_ctrl_x_mode_line_or_eval() == 0
        && (crate::vars::nvim_get_compl_cont_status() & 32) == 0
        && crate::vars::nvim_cpt_sources_array_exists() != 0;
    if normal_mode_strict {
        crate::vars::nvim_set_cpt_sources_index(0);
        if crate::vars::nvim_get_compl_autocomplete() != 0 || crate::vars::nvim_p_cto() > 0 {
            rs_compl_source_start_timer(0);
            crate::vars::nvim_set_compl_time_slice_expired(0);
            #[allow(clippy::cast_sign_loss)]
            let timeout_ms = if crate::vars::nvim_get_compl_autocomplete() != 0 {
                let p_act = crate::vars::nvim_get_p_act().max(0) as u64;
                let initial: u64 = 80; // COMPL_INITIAL_TIMEOUT_MS
                p_act.max(initial)
            } else {
                crate::vars::nvim_p_cto().max(0) as u64
            };
            crate::vars::nvim_set_compl_timeout_ms(timeout_ms);
        }
    }
    // compl_type starts as CTRL_X_NORMAL (0); process_next_cpt_value will update it
    compl_type = 0;

    // --- Main loop: iterate over 'complete' option entries ---
    loop {
        found_new_match = FAIL;
        crate::vars::nvim_ins_compl_st_reset_set_match_pos();

        // For ^N/^P pick a new entry from e_cpt if compl_started is off,
        // or if found_all says this entry is done. For ^X^L only use the
        // entries from 'complete' that look in loaded buffers.
        if (rs_ctrl_x_mode_normal() != 0 || rs_ctrl_x_mode_line_or_eval() != 0)
            && (crate::vars::nvim_get_compl_started() == 0
                || crate::vars::nvim_ins_compl_st_get_found_all() != 0)
        {
            let mut new_type = compl_type;
            let status = rs_process_next_cpt_value(
                start_lnum,
                start_col,
                rs_cot_fuzzy(),
                &mut new_type,
                &mut may_advance_cpt_idx,
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
                    rs_compl_source_start_timer(crate::vars::nvim_get_cpt_sources_index());
                }
                continue;
            }
        }

        // Save and possibly reduce timeout for function completions
        let compl_timeout_save: u64;
        if normal_mode_strict
            && compl_type == CTRL_X_FUNCTION
            && (crate::vars::nvim_get_compl_autocomplete() != 0 || crate::vars::nvim_p_cto() > 0)
        {
            compl_timeout_save = crate::vars::nvim_get_compl_timeout_ms();
            let new_timeout = if crate::vars::nvim_get_compl_from_nonkeyword() != 0 {
                COMPL_FUNC_TIMEOUT_NON_KW_MS
            } else {
                COMPL_FUNC_TIMEOUT_MS
            };
            crate::vars::nvim_set_compl_timeout_ms(new_timeout);
        } else {
            compl_timeout_save = 0;
        }

        // Get the next set of completion matches
        found_new_match = get_next_completion_match(compl_type, start_lnum, start_col);

        // If complete() was called then compl_pattern has been reset. Bail out.
        if crate::vars::nvim_get_compl_pattern_is_null() != 0 {
            if normal_mode_strict
                && compl_type == CTRL_X_FUNCTION
                && (crate::vars::nvim_get_compl_autocomplete() != 0
                    || crate::vars::nvim_p_cto() > 0)
            {
                crate::vars::nvim_set_compl_timeout_ms(compl_timeout_save);
            }
            break;
        }

        if may_advance_cpt_idx != 0 {
            if rs_advance_cpt_sources_index_safe() == 0 {
                if normal_mode_strict
                    && compl_type == CTRL_X_FUNCTION
                    && (crate::vars::nvim_get_compl_autocomplete() != 0
                        || crate::vars::nvim_p_cto() > 0)
                {
                    crate::vars::nvim_set_compl_timeout_ms(compl_timeout_save);
                }
                break;
            }
            rs_compl_source_start_timer(crate::vars::nvim_get_cpt_sources_index());
        }

        // Break the loop for specialized modes or when we've found a new match
        if (rs_ctrl_x_mode_not_default() != 0 && rs_ctrl_x_mode_line_or_eval() == 0)
            || found_new_match != FAIL
        {
            if nvim_got_int {
                if normal_mode_strict
                    && compl_type == CTRL_X_FUNCTION
                    && (crate::vars::nvim_get_compl_autocomplete() != 0
                        || crate::vars::nvim_p_cto() > 0)
                {
                    crate::vars::nvim_set_compl_timeout_ms(compl_timeout_save);
                }
                break;
            }
            // Fill the popup menu as soon as possible.
            if compl_type != -1 {
                rs_ins_compl_check_keys(0, 0);
            }

            if (rs_ctrl_x_mode_not_default() != 0 && rs_ctrl_x_mode_line_or_eval() == 0)
                || crate::vars::nvim_get_compl_interrupted() != 0
            {
                if normal_mode_strict
                    && compl_type == CTRL_X_FUNCTION
                    && (crate::vars::nvim_get_compl_autocomplete() != 0
                        || crate::vars::nvim_p_cto() > 0)
                {
                    crate::vars::nvim_set_compl_timeout_ms(compl_timeout_save);
                }
                break;
            }
            let not_expired = crate::vars::nvim_get_compl_time_slice_expired() == 0;
            crate::vars::nvim_set_compl_started(c_int::from(not_expired));
        } else {
            // Mark a buffer scanned when it has been scanned completely
            if buf_valid_expand(crate::vars::ins_compl_st.ins_buf)
                && (compl_type == 0 || compl_type == CTRL_X_PATH_PATTERNS)
            {
                nvim_ins_compl_st_mark_ins_buf_scanned();
            }
            crate::vars::nvim_set_compl_started(0);
        }

        // Restore the timeout after collecting matches from function source
        if normal_mode_strict
            && compl_type == CTRL_X_FUNCTION
            && (crate::vars::nvim_get_compl_autocomplete() != 0 || crate::vars::nvim_p_cto() > 0)
        {
            crate::vars::nvim_set_compl_timeout_ms(compl_timeout_save);
        }

        // For ^P completion, reset compl_curr_match to the head to avoid
        // mixing matches from different sources.
        if rs_compl_dir_forward() == 0 {
            crate::match_list::compl_curr_rewind_to_head();
        }
    }

    // Reset cpt_sources_index and mark search as started
    crate::vars::nvim_set_cpt_sources_index(-1);
    crate::vars::nvim_set_compl_started(1);

    // Check if we reached the end of 'complete'
    if (rs_ctrl_x_mode_normal() != 0 || rs_ctrl_x_mode_line_or_eval() != 0)
        && crate::vars::nvim_ins_compl_st_e_cpt_is_nul() != 0
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
    if rs_cot_fuzzy() != 0
        && crate::vars::nvim_get_compl_get_longest() != 0
        && crate::vars::nvim_get_compl_num_bests() > 0
    {
        rs_fuzzy_longest_match();
    }

    // Advance compl_curr_match past old_match
    crate::match_list::compl_old_match_advance_curr();

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
