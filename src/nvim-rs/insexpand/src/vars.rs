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
