//! Direct access to C global completion state variables.
//!
//! These variables were previously accessed via C accessor functions
//! (`nvim_get_*` / `nvim_set_*`). Phase 1 migration removes those accessor
//! functions and replaces them with direct access to the (now non-static)
//! C globals.
//!
//! This module provides inline wrapper functions that preserve the same
//! integer-returning API as the old C accessors, allowing gradual migration.
//!
//! # Safety
//! Neovim is single-threaded for completion operations, so accesses to
//! these mutable statics are safe in practice.

#![allow(
    dead_code,
    clippy::missing_safety_doc,
    clippy::must_use_candidate,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use std::ffi::c_int;

/// C pos_T: { lnum: i32, col: i32, coladd: i32 }
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PosT {
    pub lnum: i32,
    pub col: i32,
    pub coladd: i32,
}

/// C String: { data: *mut char, size: usize }
#[repr(C)]
#[derive(Debug)]
pub(crate) struct NvimString {
    pub data: *mut std::os::raw::c_char,
    pub size: usize,
}

/// C cpt_source_T struct (Phase 23 migration).
/// Exact layout verified via offsetof checks.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct CptSourceT {
    pub cs_refresh_always: bool,
    _pad1: [u8; 3],
    pub cs_startcol: c_int,
    pub cs_max_matches: c_int,
    _pad2: [u8; 4],
    pub compl_start_tv: u64,
    pub cs_flag: i8,
    _pad3: [u8; 7],
}

extern "C" {
    // --- compl_pattern String struct ---
    pub(crate) static mut compl_pattern: NvimString;
    // --- cpt_sources_array pointer (made non-static Phase 23) ---
    pub(crate) static mut cpt_sources_array: *mut CptSourceT;
}

extern "C" {
    // --- bool variables ---
    static mut compl_interrupted: bool;
    static mut compl_time_slice_expired: bool;
    static mut compl_enter_selects: bool;
    static mut compl_get_longest: bool;
    static mut compl_used_match: bool;
    static mut compl_was_interrupted: bool;
    static mut compl_started: bool;
    static mut compl_autocomplete: bool;
    static mut compl_from_nonkeyword: bool;
    static mut compl_opt_refresh_always: bool;

    // --- int variables ---
    static mut ctrl_x_mode: c_int;
    static mut compl_matches: c_int;
    static mut compl_length: c_int;
    static mut compl_ins_end_col: c_int; // colnr_T = int
    static mut compl_selected_item: c_int;
    static mut compl_num_bests: c_int;
    static mut compl_cont_status: c_int;
    static mut compl_cont_mode: c_int;
    static mut compl_direction: c_int; // Direction = int enum
    static mut compl_shows_dir: c_int; // Direction = int enum
    static mut compl_col: c_int; // colnr_T = int
    static mut compl_lnum: c_int; // linenr_T = int
    static mut compl_timeout_ms: u64;
    static mut cpt_sources_index: c_int;
    static mut compl_match_arraysize: c_int;
    static mut spell_bad_len: usize;
    static mut cpt_sources_count: c_int;
    // pumitem_T* - treated as opaque pointer
    static mut compl_match_array: *mut u8;

    // --- global options (Phase 28, 29, 30) ---
    static mut cot_flags: std::os::raw::c_uint; // 'completeopt' flags
    static mut p_ic: c_int; // 'ignorecase'
                            // --- editor state (Phase 31) ---
    static mut State: c_int; // current editor mode
    static mut p_ac: c_int; // 'autocomplete'
    pub(crate) static mut p_acl: i64; // 'autocompletedelay' (OptInt = i64)
    static mut p_cto: i64; // 'completetimeout' (OptInt = i64)
    static mut p_act: i64; // 'autocompletetimeout' (OptInt = i64)
    static mut p_fic: c_int; // 'fileignorecase'
    static mut p_wic: c_int; // 'wildignorecase'
    static mut p_tsrfu: *mut std::os::raw::c_char; // 'thesaurusfunc'

    // --- compl_T* match list pointers (treated as opaque *mut c_void) ---
    static mut compl_first_match: *mut core::ffi::c_void;
    static mut compl_curr_match: *mut core::ffi::c_void;
    static mut compl_shown_match: *mut core::ffi::c_void;
    static mut compl_old_match: *mut core::ffi::c_void;
    // compl_T** - treated as opaque pointer-to-pointer
    static mut compl_best_matches: *mut core::ffi::c_void;

    // --- window/buffer pointers (opaque handles) ---
    static mut compl_curr_win: *mut core::ffi::c_void;
    static mut compl_curr_buf: *mut core::ffi::c_void;

    // --- pos_T struct ---
    pub(crate) static mut compl_startpos: PosT;

    // --- String structs ---
    pub(crate) static mut compl_leader: NvimString;
    pub(crate) static mut compl_orig_text: NvimString;
}

extern "C" {
    fn nvim_get_curwin_cursor_lnum() -> c_int;
}

// ============================================================================
// Read accessors (return c_int, same API as old nvim_get_* functions)
// ============================================================================

#[inline]
pub unsafe fn nvim_get_compl_interrupted() -> c_int {
    c_int::from(compl_interrupted)
}

#[inline]
pub unsafe fn nvim_get_compl_time_slice_expired() -> c_int {
    c_int::from(compl_time_slice_expired)
}

#[inline]
pub unsafe fn nvim_get_compl_enter_selects() -> c_int {
    c_int::from(compl_enter_selects)
}

#[inline]
pub unsafe fn nvim_get_compl_used_match() -> c_int {
    c_int::from(compl_used_match)
}

#[inline]
pub unsafe fn nvim_get_compl_length() -> c_int {
    compl_length
}

#[inline]
pub unsafe fn nvim_get_compl_was_interrupted() -> c_int {
    c_int::from(compl_was_interrupted)
}

#[inline]
pub unsafe fn nvim_get_compl_opt_refresh_always() -> c_int {
    c_int::from(compl_opt_refresh_always)
}

#[inline]
pub unsafe fn nvim_get_ctrl_x_mode() -> c_int {
    ctrl_x_mode
}

#[inline]
pub unsafe fn nvim_get_compl_cont_status() -> c_int {
    compl_cont_status
}

#[inline]
pub unsafe fn nvim_get_compl_started() -> c_int {
    c_int::from(compl_started)
}

#[inline]
pub unsafe fn nvim_get_compl_autocomplete() -> c_int {
    c_int::from(compl_autocomplete)
}

#[inline]
pub unsafe fn nvim_get_compl_from_nonkeyword() -> c_int {
    c_int::from(compl_from_nonkeyword)
}

#[inline]
pub unsafe fn nvim_get_compl_direction() -> c_int {
    compl_direction
}

#[inline]
pub unsafe fn nvim_get_compl_shows_dir() -> c_int {
    compl_shows_dir
}

#[inline]
pub unsafe fn nvim_get_compl_ins_end_col() -> c_int {
    compl_ins_end_col
}

#[inline]
pub unsafe fn nvim_get_compl_matches() -> c_int {
    compl_matches
}

#[inline]
pub unsafe fn nvim_get_compl_get_longest() -> c_int {
    c_int::from(compl_get_longest)
}

#[inline]
pub unsafe fn nvim_get_compl_cont_mode() -> c_int {
    compl_cont_mode
}

#[inline]
pub unsafe fn nvim_get_compl_selected_item() -> c_int {
    compl_selected_item
}

#[inline]
pub unsafe fn nvim_get_compl_num_bests() -> c_int {
    compl_num_bests
}

/// Get p_ic ('ignorecase') as bool integer.
#[inline]
pub unsafe fn nvim_get_p_ic() -> c_int {
    c_int::from(p_ic != 0)
}

/// Get p_ac ('autocomplete') as bool integer.
#[inline]
pub unsafe fn nvim_get_p_ac() -> c_int {
    c_int::from(p_ac != 0)
}

/// Get p_acl ('autocompletedelay') as c_int.
#[inline]
pub unsafe fn nvim_get_p_acl() -> c_int {
    p_acl as c_int
}

/// Get p_cto ('completetimeout') as c_int.
#[inline]
pub unsafe fn nvim_p_cto() -> c_int {
    p_cto as c_int
}

/// Get p_act ('autocompletetimeout') as c_int.
#[inline]
pub unsafe fn nvim_get_p_act() -> c_int {
    p_act as c_int
}

/// Get (p_fic || p_wic) as bool integer.
#[inline]
pub unsafe fn nvim_get_p_fic_or_wic() -> c_int {
    c_int::from(p_fic != 0 || p_wic != 0)
}

/// Check if p_tsrfu ('thesaurusfunc') is non-empty.
#[inline]
pub unsafe fn nvim_get_p_tsrfu_nonempty() -> c_int {
    c_int::from(!p_tsrfu.is_null() && *p_tsrfu != 0)
}

/// kOptCotFlagNoinsert = 0x20
const K_OPT_COT_FLAG_NOINSERT: std::os::raw::c_uint = 0x20;
/// kOptCotFlagFuzzy = 0x80
const K_OPT_COT_FLAG_FUZZY: std::os::raw::c_uint = 0x80;

/// Get the global cot_flags ('completeopt' flags).
#[inline]
pub unsafe fn nvim_get_cot_flags_global() -> std::os::raw::c_uint {
    cot_flags
}

/// Return 1 if cot_flags has noinsert or fuzzy set, 0 otherwise.
#[inline]
pub unsafe fn nvim_cot_flags_has_noinsert_fuzzy() -> c_int {
    c_int::from((cot_flags & (K_OPT_COT_FLAG_NOINSERT | K_OPT_COT_FLAG_FUZZY)) != 0)
}

/// REPLACE_FLAG = 0x100 (from state_defs.h)
const REPLACE_FLAG: c_int = 0x100;

/// Return 1 if current State has REPLACE_FLAG set, 0 otherwise.
#[inline]
pub unsafe fn nvim_get_state_replace_flag() -> c_int {
    c_int::from((State & REPLACE_FLAG) != 0)
}

// ============================================================================
// Write accessors (same API as old nvim_set_* functions)
// ============================================================================

#[inline]
pub unsafe fn nvim_set_compl_interrupted(val: c_int) {
    compl_interrupted = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_time_slice_expired(val: c_int) {
    compl_time_slice_expired = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_enter_selects(val: c_int) {
    compl_enter_selects = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_used_match(val: c_int) {
    compl_used_match = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_get_longest(val: c_int) {
    compl_get_longest = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_was_interrupted(val: c_int) {
    compl_was_interrupted = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_started(val: c_int) {
    compl_started = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_autocomplete(val: c_int) {
    compl_autocomplete = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_from_nonkeyword(val: c_int) {
    compl_from_nonkeyword = val != 0;
}

#[inline]
pub unsafe fn nvim_set_compl_opt_refresh_always(val: c_int) {
    compl_opt_refresh_always = val != 0;
}

#[inline]
pub unsafe fn nvim_set_ctrl_x_mode(val: c_int) {
    ctrl_x_mode = val;
}

#[inline]
pub unsafe fn nvim_set_compl_matches(val: c_int) {
    compl_matches = val;
}

#[inline]
pub unsafe fn nvim_set_compl_length(val: c_int) {
    compl_length = val;
}

#[inline]
pub unsafe fn nvim_set_compl_ins_end_col(val: c_int) {
    compl_ins_end_col = val;
}

#[inline]
pub unsafe fn nvim_set_compl_selected_item(val: c_int) {
    compl_selected_item = val;
}

#[inline]
pub unsafe fn nvim_set_compl_num_bests(val: c_int) {
    compl_num_bests = val;
}

#[inline]
pub unsafe fn nvim_set_compl_cont_status(val: c_int) {
    compl_cont_status = val;
}

#[inline]
pub unsafe fn nvim_set_compl_cont_mode(val: c_int) {
    compl_cont_mode = val;
}

#[inline]
pub unsafe fn nvim_set_compl_direction(val: c_int) {
    compl_direction = val;
}

#[inline]
pub unsafe fn nvim_set_compl_shows_dir(val: c_int) {
    compl_shows_dir = val;
}

#[inline]
pub unsafe fn nvim_get_compl_col() -> c_int {
    compl_col
}

#[inline]
pub unsafe fn nvim_set_compl_col(val: c_int) {
    compl_col = val;
}

#[inline]
pub unsafe fn nvim_get_compl_lnum() -> c_int {
    compl_lnum
}

#[inline]
pub unsafe fn nvim_set_compl_lnum(val: c_int) {
    compl_lnum = val;
}

#[inline]
pub unsafe fn nvim_get_compl_timeout_ms() -> u64 {
    compl_timeout_ms
}

#[inline]
pub unsafe fn nvim_set_compl_timeout_ms(val: u64) {
    compl_timeout_ms = val;
}

/// Decay the completion timeout: halve it if above the minimum (5 ms).
#[inline]
pub unsafe fn nvim_decay_compl_timeout() {
    const COMPL_MIN_TIMEOUT_MS: u64 = 5;
    if compl_timeout_ms > COMPL_MIN_TIMEOUT_MS {
        compl_timeout_ms /= 2;
    }
}

#[inline]
pub unsafe fn nvim_get_cpt_sources_index() -> c_int {
    cpt_sources_index
}

#[inline]
pub unsafe fn nvim_set_cpt_sources_index(val: c_int) {
    cpt_sources_index = val;
}

#[inline]
pub unsafe fn nvim_get_compl_match_arraysize() -> c_int {
    compl_match_arraysize
}

#[inline]
pub unsafe fn nvim_set_compl_match_arraysize(val: c_int) {
    compl_match_arraysize = val;
}

#[inline]
pub unsafe fn nvim_set_spell_bad_len(val: c_int) {
    #[allow(clippy::cast_sign_loss)]
    let n = if val > 0 { val as usize } else { 0 };
    spell_bad_len = n;
}

#[inline]
pub unsafe fn nvim_get_cpt_sources_count() -> c_int {
    cpt_sources_count
}

/// Check if compl_match_array is non-null (i.e., the popup menu array exists).
#[inline]
pub unsafe fn nvim_get_compl_match_array_exists() -> c_int {
    c_int::from(!compl_match_array.is_null())
}

/// Free and clear compl_match_array (equivalent to C XFREE_CLEAR macro).
#[inline]
pub unsafe fn nvim_xfree_compl_match_array() {
    extern "C" {
        fn xfree(ptr: *mut u8);
    }
    if !compl_match_array.is_null() {
        xfree(compl_match_array);
        compl_match_array = core::ptr::null_mut();
    }
}

// ============================================================================
// Match list pointer accessors (compl_first_match, compl_curr_match, etc.)
// ============================================================================

use core::ffi::c_void;

#[inline]
pub unsafe fn nvim_get_compl_first_match() -> *mut c_void {
    compl_first_match
}

#[inline]
pub unsafe fn nvim_set_compl_first_match(m: *mut c_void) {
    compl_first_match = m;
}

#[inline]
pub unsafe fn nvim_get_compl_curr_match() -> *mut c_void {
    compl_curr_match
}

#[inline]
pub unsafe fn nvim_set_compl_curr_match(m: *mut c_void) {
    compl_curr_match = m;
}

#[inline]
pub unsafe fn nvim_get_compl_shown_match() -> *mut c_void {
    compl_shown_match
}

#[inline]
pub unsafe fn nvim_set_compl_shown_match(m: *mut c_void) {
    compl_shown_match = m;
}

#[inline]
pub unsafe fn nvim_get_compl_old_match() -> *mut c_void {
    compl_old_match
}

#[inline]
pub unsafe fn nvim_set_compl_old_match(m: *mut c_void) {
    compl_old_match = m;
}

/// Clear compl_best_matches (set to NULL). Replaces C nvim_clear_compl_best_matches.
#[inline]
pub unsafe fn nvim_clear_compl_best_matches() {
    compl_best_matches = core::ptr::null_mut();
}

// ============================================================================
// Window/buffer pointer accessors
// ============================================================================

#[inline]
pub unsafe fn nvim_get_compl_curr_win() -> *mut c_void {
    compl_curr_win
}

#[inline]
pub unsafe fn nvim_clear_compl_curr_win() {
    compl_curr_win = core::ptr::null_mut();
}

#[inline]
pub unsafe fn nvim_get_compl_curr_buf() -> *mut c_void {
    compl_curr_buf
}

#[inline]
pub unsafe fn nvim_clear_compl_curr_buf() {
    compl_curr_buf = core::ptr::null_mut();
}

/// Get compl_startpos.lnum
#[inline]
pub unsafe fn nvim_get_compl_startpos_lnum() -> c_int {
    compl_startpos.lnum
}

/// Get compl_startpos.col
#[inline]
pub unsafe fn nvim_get_compl_startpos_col() -> c_int {
    compl_startpos.col
}

/// Set compl_startpos.col
#[inline]
pub unsafe fn nvim_set_compl_startpos_col(val: c_int) {
    compl_startpos.col = val;
}

/// Set compl_startpos.lnum to cursor lnum (calls C for curwin access)
#[inline]
pub unsafe fn nvim_set_compl_startpos_lnum_to_cursor() {
    compl_startpos.lnum = nvim_get_curwin_cursor_lnum();
}

/// Set compl_startpos.col = compl_col
#[inline]
pub unsafe fn nvim_set_compl_startpos_col_to_compl_col() {
    compl_startpos.col = compl_col;
}

/// Set compl_startpos: lnum from cursor if requested, col from param.
#[inline]
pub unsafe fn nvim_set_compl_startpos_lnum_col(lnum_to_cursor: c_int, col: c_int) {
    if lnum_to_cursor != 0 {
        compl_startpos.lnum = nvim_get_curwin_cursor_lnum();
    }
    compl_startpos.col = col;
}

/// Get compl_leader.data
#[inline]
pub unsafe fn nvim_get_compl_leader_data() -> *const std::os::raw::c_char {
    compl_leader.data.cast_const()
}

/// Get compl_leader.size
#[inline]
pub unsafe fn nvim_get_compl_leader_size() -> usize {
    compl_leader.size
}

/// Get compl_orig_text.data
#[inline]
pub unsafe fn nvim_get_compl_orig_text_data() -> *const std::os::raw::c_char {
    compl_orig_text.data.cast_const()
}

/// Get compl_orig_text.size
#[inline]
pub unsafe fn nvim_get_compl_orig_text_size() -> usize {
    compl_orig_text.size
}

/// Free and clear compl_leader (equivalent to C API_CLEAR_STRING macro).
#[inline]
pub unsafe fn nvim_compl_clear_leader() {
    extern "C" {
        fn xfree(ptr: *mut u8);
    }
    if !compl_leader.data.is_null() {
        xfree(compl_leader.data.cast());
        compl_leader.data = core::ptr::null_mut();
        compl_leader.size = 0;
    }
}

/// Free and clear compl_orig_text (equivalent to C API_CLEAR_STRING macro).
#[inline]
pub unsafe fn nvim_compl_clear_orig_text() {
    extern "C" {
        fn xfree(ptr: *mut u8);
    }
    if !compl_orig_text.data.is_null() {
        xfree(compl_orig_text.data.cast());
        compl_orig_text.data = core::ptr::null_mut();
        compl_orig_text.size = 0;
    }
}

// ============================================================================
// compl_pattern accessors (String struct, made non-static Phase 22)
// ============================================================================

/// Check if compl_pattern.data is null.
#[inline]
pub unsafe fn nvim_get_compl_pattern_is_null() -> c_int {
    c_int::from(compl_pattern.data.is_null())
}

/// Get compl_pattern.data (mutable).
#[inline]
pub unsafe fn nvim_compl_pattern_get_data() -> *mut std::os::raw::c_char {
    compl_pattern.data
}

/// Free and clear compl_pattern (equivalent to C API_CLEAR_STRING macro).
#[inline]
pub unsafe fn nvim_compl_clear_pattern() {
    extern "C" {
        fn xfree(ptr: *mut u8);
    }
    if !compl_pattern.data.is_null() {
        xfree(compl_pattern.data.cast());
        compl_pattern.data = core::ptr::null_mut();
        compl_pattern.size = 0;
    }
}

/// Set compl_pattern from pre-allocated data and size (takes ownership).
#[inline]
pub unsafe fn nvim_compl_pattern_set_from_alloc(data: *mut std::os::raw::c_char, size: usize) {
    nvim_compl_clear_pattern();
    compl_pattern.data = data;
    compl_pattern.size = size;
}

// ============================================================================
// cpt_sources_array accessors (Phase 23)
// ============================================================================

/// Check if cpt_sources_array is non-null.
#[inline]
pub unsafe fn nvim_cpt_sources_array_exists() -> c_int {
    c_int::from(!cpt_sources_array.is_null())
}

/// Get cs_startcol for the given source index (-1 if array is null or idx < 0).
#[inline]
pub unsafe fn nvim_get_cpt_source_startcol(idx: c_int) -> c_int {
    if cpt_sources_array.is_null() || idx < 0 {
        return -1;
    }
    (*cpt_sources_array.add(idx as usize)).cs_startcol
}

/// Get cs_flag (as unsigned byte cast to c_int) for the given source index.
#[inline]
pub unsafe fn nvim_get_cpt_source_cs_flag(idx: c_int) -> c_int {
    if cpt_sources_array.is_null() || idx < 0 {
        return 0;
    }
    c_int::from((*cpt_sources_array.add(idx as usize)).cs_flag as u8)
}

/// Get cs_max_matches for the given source index.
#[inline]
pub unsafe fn nvim_get_cpt_source_cs_max_matches(idx: c_int) -> c_int {
    if cpt_sources_array.is_null() || idx < 0 {
        return 0;
    }
    (*cpt_sources_array.add(idx as usize)).cs_max_matches
}

/// Get cs_refresh_always for the given source index.
#[inline]
pub unsafe fn nvim_cpt_sources_get_refresh_always(idx: c_int) -> c_int {
    if cpt_sources_array.is_null() || idx < 0 {
        return 0;
    }
    c_int::from((*cpt_sources_array.add(idx as usize)).cs_refresh_always)
}

/// Get compl_start_tv for the current source (cpt_sources_index).
#[inline]
pub unsafe fn nvim_get_cpt_start_tv() -> u64 {
    (*cpt_sources_array.add(cpt_sources_index as usize)).compl_start_tv
}

/// Set compl_start_tv for a specific source index.
#[inline]
pub unsafe fn nvim_set_cpt_sources_start_tv(idx: c_int, ts: u64) {
    (*cpt_sources_array.add(idx as usize)).compl_start_tv = ts;
}

/// Set cs_flag for a specific source index.
#[inline]
pub unsafe fn nvim_cpt_sources_set_flag(idx: c_int, flag: c_int) {
    if !cpt_sources_array.is_null() && idx >= 0 {
        (*cpt_sources_array.add(idx as usize)).cs_flag = flag as i8;
    }
}

/// Set cs_max_matches for a specific source index.
#[inline]
pub unsafe fn nvim_cpt_sources_set_max_matches(idx: c_int, val: c_int) {
    if !cpt_sources_array.is_null() && idx >= 0 {
        (*cpt_sources_array.add(idx as usize)).cs_max_matches = val;
    }
}

/// Set cs_startcol for a specific source index.
#[inline]
pub unsafe fn nvim_cpt_sources_set_startcol(idx: c_int, val: c_int) {
    if !cpt_sources_array.is_null() && idx >= 0 {
        (*cpt_sources_array.add(idx as usize)).cs_startcol = val;
    }
}

/// Set cs_refresh_always for a specific source index.
#[inline]
pub unsafe fn nvim_cpt_sources_set_refresh_always(idx: c_int, val: c_int) {
    if !cpt_sources_array.is_null() && idx >= 0 {
        (*cpt_sources_array.add(idx as usize)).cs_refresh_always = val != 0;
    }
}

/// Allocate (or reset) cpt_sources_array for `count` sources.
#[inline]
pub unsafe fn nvim_cpt_sources_alloc(count: c_int) {
    extern "C" {
        fn xfree(ptr: *mut u8);
        fn xcalloc(count: usize, size: usize) -> *mut u8;
    }
    if !cpt_sources_array.is_null() {
        xfree(cpt_sources_array.cast());
        cpt_sources_array = core::ptr::null_mut();
    }
    cpt_sources_index = -1;
    cpt_sources_count = 0;
    if count > 0 {
        cpt_sources_array = xcalloc(count as usize, core::mem::size_of::<CptSourceT>()).cast();
        cpt_sources_count = count;
    }
}

/// Free and clear cpt_sources_array; reset cpt_sources_index and cpt_sources_count.
#[inline]
pub unsafe fn nvim_cpt_sources_clear() {
    extern "C" {
        fn xfree(ptr: *mut u8);
    }
    if !cpt_sources_array.is_null() {
        xfree(cpt_sources_array.cast());
        cpt_sources_array = core::ptr::null_mut();
    }
    cpt_sources_index = -1;
    cpt_sources_count = 0;
}

// ============================================================================
// ins_compl_st (ins_compl_next_state_T) direct field accessors (Phase 26)
// ============================================================================

/// C ins_compl_next_state_T struct (Phase 26 migration).
/// Exact layout verified via offsetof checks.
/// sizeof = 104; offsets:
///   e_cpt_copy=0, e_cpt=8, ins_buf=16, cur_match_pos=24,
///   prev_match_pos=32, set_match_pos=44, first_match_pos=48,
///   last_match_pos=60, found_all=72, dict=80, dict_f=88, func_cb=96
#[repr(C)]
pub(crate) struct InsComplNextStateT {
    pub e_cpt_copy: *mut std::os::raw::c_char, // offset 0
    pub e_cpt: *mut std::os::raw::c_char,      // offset 8
    pub ins_buf: *mut core::ffi::c_void,       // offset 16 (buf_T*)
    pub cur_match_pos: *mut PosT,              // offset 24 (pos_T*)
    pub prev_match_pos: PosT,                  // offset 32
    pub set_match_pos: bool,                   // offset 44
    _pad1: [u8; 3],
    pub first_match_pos: PosT, // offset 48
    pub last_match_pos: PosT,  // offset 60
    pub found_all: bool,       // offset 72
    _pad2: [u8; 7],
    pub dict: *mut std::os::raw::c_char, // offset 80
    pub dict_f: c_int,                   // offset 88
    _pad3: [u8; 4],
    pub func_cb: *mut core::ffi::c_void, // offset 96 (Callback*)
}

extern "C" {
    pub(crate) static mut ins_compl_st: InsComplNextStateT;
}

/// Get ins_compl_st.dict
#[inline]
pub unsafe fn nvim_ins_compl_st_get_dict() -> *mut std::os::raw::c_char {
    ins_compl_st.dict
}

/// Get ins_compl_st.dict_f
#[inline]
pub unsafe fn nvim_ins_compl_st_get_dict_f() -> c_int {
    ins_compl_st.dict_f
}

/// Clear ins_compl_st.dict (set to NULL)
#[inline]
pub unsafe fn nvim_ins_compl_st_clear_dict() {
    ins_compl_st.dict = core::ptr::null_mut();
}

/// Get ins_compl_st.func_cb (as opaque pointer)
#[inline]
pub unsafe fn nvim_ins_compl_st_get_func_cb() -> *mut core::ffi::c_void {
    ins_compl_st.func_cb
}

/// Get ins_compl_st.first_match_pos.lnum
#[inline]
pub unsafe fn nvim_ins_compl_st_get_first_lnum() -> c_int {
    ins_compl_st.first_match_pos.lnum
}

/// Set ins_compl_st.found_all
#[inline]
pub unsafe fn nvim_ins_compl_st_set_found_all(val: c_int) {
    ins_compl_st.found_all = val != 0;
}

/// Get ins_compl_st.found_all
#[inline]
pub unsafe fn nvim_ins_compl_st_get_found_all() -> c_int {
    c_int::from(ins_compl_st.found_all)
}

/// Check if *ins_compl_st.e_cpt == NUL
#[inline]
pub unsafe fn nvim_ins_compl_st_e_cpt_is_nul() -> c_int {
    c_int::from(!ins_compl_st.e_cpt.is_null() && *ins_compl_st.e_cpt == 0)
}

/// Set ins_compl_st.set_match_pos = false
#[inline]
pub unsafe fn nvim_ins_compl_st_reset_set_match_pos() {
    ins_compl_st.set_match_pos = false;
}

/// Get ins_compl_st.cur_match_pos->lnum
#[inline]
pub unsafe fn nvim_ins_compl_st_get_cur_match_lnum() -> c_int {
    (*ins_compl_st.cur_match_pos).lnum
}

/// Get ins_compl_st.cur_match_pos->col
#[inline]
pub unsafe fn nvim_ins_compl_st_get_cur_match_col() -> c_int {
    (*ins_compl_st.cur_match_pos).col
}

/// Get ins_compl_st.prev_match_pos.lnum
#[inline]
pub unsafe fn nvim_ins_compl_st_get_prev_match_lnum() -> c_int {
    ins_compl_st.prev_match_pos.lnum
}

/// Get ins_compl_st.prev_match_pos.col
#[inline]
pub unsafe fn nvim_ins_compl_st_get_prev_match_col() -> c_int {
    ins_compl_st.prev_match_pos.col
}

/// Copy *cur_match_pos to prev_match_pos.
#[inline]
pub unsafe fn nvim_ins_compl_st_set_prev_from_cur() {
    ins_compl_st.prev_match_pos = *ins_compl_st.cur_match_pos;
}

/// Get the current character at e_cpt (as unsigned byte), or 0 if null.
#[inline]
pub unsafe fn nvim_ins_compl_st_get_e_cpt_char() -> c_int {
    if ins_compl_st.e_cpt.is_null() {
        0
    } else {
        c_int::from(*ins_compl_st.e_cpt as u8)
    }
}

/// Skip commas and spaces at the start of e_cpt.
#[inline]
pub unsafe fn nvim_ins_compl_st_skip_delimiters() {
    while !ins_compl_st.e_cpt.is_null() {
        let ch = *ins_compl_st.e_cpt as u8;
        if ch == b',' || ch == b' ' {
            ins_compl_st.e_cpt = ins_compl_st.e_cpt.add(1);
        } else {
            break;
        }
    }
}

/// Advance ins_compl_st.e_cpt by one character.
#[inline]
pub unsafe fn nvim_ins_compl_st_e_cpt_inc() {
    ins_compl_st.e_cpt = ins_compl_st.e_cpt.add(1);
}

/// Set ins_compl_st.dict = e_cpt and dict_f = DICT_FIRST (1).
#[inline]
pub unsafe fn nvim_ins_compl_st_set_dict_from_e_cpt() {
    ins_compl_st.dict = ins_compl_st.e_cpt;
    ins_compl_st.dict_f = 1; // DICT_FIRST
}
