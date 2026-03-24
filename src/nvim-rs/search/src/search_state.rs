//! Search pattern state
//!
//! Owns the static state for search patterns (spats, mr_pattern, last_idx)
//! and all save/restore state. Previously these lived as C static variables
//! in search.c.
//!
//! SAFETY invariant: Neovim is single-threaded. All static mut accesses here
//! are safe because they only occur on the main thread.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// C FFI dependencies
// =============================================================================

extern "C" {
    /// Allocate a copy of `s` with length `len` (C xstrnsave).
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    /// Free a heap allocation (C xfree).
    fn xfree(p: *mut c_void);

    /// Get the current time as a Unix timestamp (C os_time).
    fn os_time() -> i64;

    /// Redraw all windows later with the given update type.
    fn nvim_redraw_all_later(upd_type: c_int);

    /// Reverse a text string (for 'rightleft' mode).
    fn reverse_text(s: *const c_char) -> *mut c_char;

    /// Set VV_SEARCHFORWARD vimvar based on last search direction.
    fn nvim_call_set_vv_searchforward();

    /// Set the no_hlsearch global flag.
    fn set_no_hlsearch(flag: c_int);

    /// Get the no_hlsearch global flag.
    fn nvim_get_no_hlsearch() -> c_int;

    /// Get the p_hls (hlsearch) option.
    fn nvim_get_p_hls() -> c_int;

    /// Get the no_smartcase option.
    fn nvim_get_no_smartcase() -> c_int;

    /// Get whether curwin->w_p_rl is set and curwin->w_p_rlc starts with 's'.
    fn nvim_curwin_rl_with_rlc_s() -> c_int;
}

/// UPD_SOME_VALID from buffer_defs.h
const UPD_SOME_VALID: c_int = 3;

// =============================================================================
// SearchOffset and SearchPattern structs (must match C layout exactly)
// Using the same approach as the existing SearchOffsetC/SearchPatternC in pattern.rs
// =============================================================================

/// C-compatible SearchOffset struct (must match C SearchOffset layout).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SearchOffset {
    /// Search direction: forward ('/') or backward ('?')
    pub dir: i8,
    /// True if search has line offset.
    pub line: bool,
    /// True if search sets cursor at the end.
    pub end: bool,
    /// Actual offset value.
    pub off: i64,
}

impl SearchOffset {
    pub const fn default_forward() -> Self {
        Self {
            dir: b'/' as i8,
            line: false,
            end: false,
            off: 0,
        }
    }
}

/// C-compatible SearchPattern struct (must match C SearchPattern layout).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SearchPattern {
    /// The pattern (in allocated memory) or NULL.
    pub pat: *mut c_char,
    /// The length of the pattern (0 if pat is NULL).
    pub patlen: usize,
    /// Magicness of the pattern.
    pub magic: bool,
    /// No smartcase for this pattern.
    pub no_scs: bool,
    /// Time of the last change (Timestamp = u64).
    pub timestamp: u64,
    /// Pattern offset.
    pub off: SearchOffset,
    /// Additional data from ShaDa file (AdditionalData *).
    pub additional_data: *mut c_void,
}

impl SearchPattern {
    pub const fn empty() -> Self {
        Self {
            pat: std::ptr::null_mut(),
            patlen: 0,
            magic: true,
            no_scs: false,
            timestamp: 0,
            off: SearchOffset::default_forward(),
            additional_data: std::ptr::null_mut(),
        }
    }
}

// SAFETY: We're in a single-threaded C program.
unsafe impl Send for SearchPattern {}
unsafe impl Sync for SearchPattern {}

// =============================================================================
// Static state (Rust-owned, replaces C statics)
// =============================================================================

/// The two search patterns: [0] = search, [1] = substitute.
static mut SPATS: [SearchPattern; 2] = [SearchPattern::empty(), SearchPattern::empty()];

/// Copy of SPATS for save/restore across autocommands.
static mut SAVED_SPATS: [SearchPattern; 2] = [SearchPattern::empty(), SearchPattern::empty()];

/// Last used pattern index (0 = search, 1 = substitute).
static mut LAST_IDX: c_int = 0;

/// Allocated copy of pattern used by search_regcomp().
static mut MR_PATTERN: *mut c_char = std::ptr::null_mut();
static mut MR_PATTERNLEN: usize = 0;

/// Copy of mr_pattern for save/restore.
static mut SAVED_MR_PATTERN: *mut c_char = std::ptr::null_mut();
static mut SAVED_MR_PATTERNLEN: usize = 0;

/// Saved last_idx for pattern save/restore.
static mut SAVED_SPATS_LAST_IDX: c_int = 0;
/// Saved no_hlsearch for pattern save/restore.
static mut SAVED_SPATS_NO_HLSEARCH: bool = false;

/// Save level counter for nested save/restore.
static mut SAVE_LEVEL: c_int = 0;

/// Copy of spats[RE_SEARCH] for incsearch save/restore.
static mut SAVED_LAST_SEARCH_SPAT: SearchPattern = SearchPattern::empty();
/// Counter tracking incsearch save/restore balance.
static mut DID_SAVE_LAST_SEARCH_SPAT: c_int = 0;
/// Saved last_idx for incsearch save/restore.
static mut SAVED_LAST_IDX: c_int = 0;
/// Saved no_hlsearch for incsearch save/restore.
static mut SAVED_NO_HLSEARCH: bool = false;

/// Saved search_match_endcol for incsearch.
static mut SAVED_SEARCH_MATCH_ENDCOL: i32 = 0;
/// Saved search_match_lines for incsearch.
static mut SAVED_SEARCH_MATCH_LINES: i32 = 0;

// =============================================================================
// Free helper (equivalent to C's static free_spat)
// =============================================================================

/// Free pat and additional_data fields of a SearchPattern.
///
/// # Safety
/// Must only be called on a SearchPattern with C-heap-allocated fields.
unsafe fn free_spat(spat: *mut SearchPattern) {
    xfree((*spat).pat.cast());
    xfree((*spat).additional_data);
    (*spat).pat = std::ptr::null_mut();
    (*spat).additional_data = std::ptr::null_mut();
}

// =============================================================================
// last_idx accessors
// =============================================================================

#[inline]
pub fn get_last_idx() -> c_int {
    // SAFETY: single-threaded
    unsafe { LAST_IDX }
}

#[inline]
pub fn set_last_idx(idx: c_int) {
    // SAFETY: single-threaded
    unsafe { LAST_IDX = idx }
}

// =============================================================================
// spats field accessors
// =============================================================================

#[inline]
pub fn get_spat_pat(idx: c_int) -> *const c_char {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].pat
        } else {
            std::ptr::null()
        }
    }
}

#[inline]
pub fn get_spat_patlen(idx: c_int) -> usize {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].patlen
        } else {
            0
        }
    }
}

#[inline]
pub fn get_spat_magic(idx: c_int) -> bool {
    unsafe { (0..2).contains(&idx) && SPATS[idx as usize].magic }
}

#[inline]
pub fn get_spat_no_scs(idx: c_int) -> bool {
    unsafe { (0..2).contains(&idx) && SPATS[idx as usize].no_scs }
}

#[inline]
pub fn get_spat_off_dir(idx: c_int) -> i8 {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].off.dir
        } else {
            b'/' as i8
        }
    }
}

#[inline]
pub fn get_spat_off_line(idx: c_int) -> bool {
    unsafe { (0..2).contains(&idx) && SPATS[idx as usize].off.line }
}

#[inline]
pub fn get_spat_off_end(idx: c_int) -> bool {
    unsafe { (0..2).contains(&idx) && SPATS[idx as usize].off.end }
}

#[inline]
pub fn get_spat_off_off(idx: c_int) -> i64 {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].off.off
        } else {
            0
        }
    }
}

#[inline]
pub fn set_spat_off_dir(idx: c_int, dir: i8) {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].off.dir = dir;
        }
    }
}

#[inline]
pub fn set_spat_off_line(idx: c_int, line: bool) {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].off.line = line;
        }
    }
}

#[inline]
pub fn set_spat_off_end(idx: c_int, end: bool) {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].off.end = end;
        }
    }
}

#[inline]
pub fn set_spat_off_off(idx: c_int, off: i64) {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].off.off = off;
        }
    }
}

// =============================================================================
// mr_pattern accessors
// =============================================================================

#[inline]
pub fn get_mr_pattern() -> *const c_char {
    unsafe { MR_PATTERN }
}

#[inline]
pub fn get_mr_patternlen() -> usize {
    unsafe { MR_PATTERNLEN }
}

// =============================================================================
// save_level accessors
// =============================================================================

#[inline]
pub fn get_save_level() -> c_int {
    unsafe { SAVE_LEVEL }
}

// =============================================================================
// did_save_last_search_spat accessors
// =============================================================================

#[inline]
pub fn get_did_save_last_search_spat() -> c_int {
    unsafe { DID_SAVE_LAST_SEARCH_SPAT }
}

// =============================================================================
// Batch save/restore operations
// =============================================================================

/// Save search patterns (called before executing autocommands).
pub fn save_search_patterns_batch() {
    unsafe {
        for i in 0..2usize {
            let src = &raw const SPATS[i];
            let dst = &raw mut SAVED_SPATS[i];
            std::ptr::copy_nonoverlapping(src, dst, 1);
            if !SPATS[i].pat.is_null() {
                (*dst).pat = xstrnsave(SPATS[i].pat, SPATS[i].patlen);
                (*dst).patlen = SPATS[i].patlen;
            }
        }
        if MR_PATTERN.is_null() {
            SAVED_MR_PATTERN = std::ptr::null_mut();
            SAVED_MR_PATTERNLEN = 0;
        } else {
            SAVED_MR_PATTERN = xstrnsave(MR_PATTERN, MR_PATTERNLEN);
            SAVED_MR_PATTERNLEN = MR_PATTERNLEN;
        }
        SAVED_SPATS_LAST_IDX = LAST_IDX;
        SAVED_SPATS_NO_HLSEARCH = nvim_get_no_hlsearch() != 0;
    }
}

/// Restore search patterns (called after executing autocommands).
pub fn restore_search_patterns_batch() {
    unsafe {
        for i in 0..2usize {
            free_spat(&raw mut SPATS[i]);
            let src = &raw const SAVED_SPATS[i];
            let dst = &raw mut SPATS[i];
            std::ptr::copy_nonoverlapping(src, dst, 1);
        }
        nvim_call_set_vv_searchforward();
        xfree(MR_PATTERN.cast());
        MR_PATTERN = SAVED_MR_PATTERN;
        MR_PATTERNLEN = SAVED_MR_PATTERNLEN;
        LAST_IDX = SAVED_SPATS_LAST_IDX;
        set_no_hlsearch(c_int::from(SAVED_SPATS_NO_HLSEARCH));
    }
}

/// Increment save_level and return old value.
pub fn inc_save_level() -> c_int {
    unsafe {
        let old = SAVE_LEVEL;
        SAVE_LEVEL += 1;
        old
    }
}

/// Decrement save_level and return new value.
pub fn dec_save_level() -> c_int {
    unsafe {
        SAVE_LEVEL -= 1;
        SAVE_LEVEL
    }
}

/// Save last search pattern for incsearch.
pub fn save_last_search_spat_batch() {
    unsafe {
        let src = &raw const SPATS[0];
        let dst = &raw mut SAVED_LAST_SEARCH_SPAT;
        std::ptr::copy_nonoverlapping(src, dst, 1);
        if !SPATS[0].pat.is_null() {
            (*dst).pat = xstrnsave(SPATS[0].pat, SPATS[0].patlen);
            (*dst).patlen = SPATS[0].patlen;
        }
        SAVED_LAST_IDX = LAST_IDX;
        SAVED_NO_HLSEARCH = nvim_get_no_hlsearch() != 0;
    }
}

/// Restore last search pattern for incsearch.
pub fn restore_last_search_spat_batch() {
    unsafe {
        xfree(SPATS[0].pat.cast());
        let src = &raw const SAVED_LAST_SEARCH_SPAT;
        let dst = &raw mut SPATS[0];
        std::ptr::copy_nonoverlapping(src, dst, 1);
        SAVED_LAST_SEARCH_SPAT.pat = std::ptr::null_mut();
        SAVED_LAST_SEARCH_SPAT.patlen = 0;
        nvim_call_set_vv_searchforward();
        LAST_IDX = SAVED_LAST_IDX;
        set_no_hlsearch(c_int::from(SAVED_NO_HLSEARCH));
    }
}

/// Increment did_save_last_search_spat and return old value.
pub fn inc_did_save() -> c_int {
    unsafe {
        let old = DID_SAVE_LAST_SEARCH_SPAT;
        DID_SAVE_LAST_SEARCH_SPAT += 1;
        old
    }
}

/// Decrement did_save_last_search_spat and return new value.
pub fn dec_did_save() -> c_int {
    unsafe {
        DID_SAVE_LAST_SEARCH_SPAT -= 1;
        DID_SAVE_LAST_SEARCH_SPAT
    }
}

/// Save incsearch match state (returns saved values for caller use).
pub fn save_incsearch_state_batch(match_endcol: i32, match_lines: i32) {
    unsafe {
        SAVED_SEARCH_MATCH_ENDCOL = match_endcol;
        SAVED_SEARCH_MATCH_LINES = match_lines;
    }
}

/// Restore incsearch match state.
pub fn restore_incsearch_state_batch() -> (i32, i32) {
    unsafe { (SAVED_SEARCH_MATCH_ENDCOL, SAVED_SEARCH_MATCH_LINES) }
}

// =============================================================================
// Pattern management (save_re_pat, set_last_search_pat, set_mr_pattern)
// =============================================================================

/// Update spats[idx] with a new pattern (equivalent to save_re_pat).
///
/// # Safety
/// `pat` must be a valid pointer to `patlen` bytes.
pub unsafe fn save_re_pat_batch(idx: c_int, pat: *const c_char, patlen: usize, magic: c_int) {
    if !(0..2).contains(&idx) {
        return;
    }
    let i = idx as usize;
    // Don't update if pat is already the same pointer
    if SPATS[i].pat == pat.cast_mut() {
        return;
    }
    free_spat(&raw mut SPATS[i]);
    SPATS[i].pat = xstrnsave(pat, patlen);
    SPATS[i].patlen = patlen;
    SPATS[i].magic = magic != 0;
    SPATS[i].no_scs = nvim_get_no_smartcase() != 0;
    SPATS[i].timestamp = os_time() as u64;
    SPATS[i].additional_data = std::ptr::null_mut();
    LAST_IDX = idx;
    if nvim_get_p_hls() != 0 {
        nvim_redraw_all_later(UPD_SOME_VALID);
    }
    set_no_hlsearch(0);
}

/// Set mr_pattern from pat (or reversed if rightleft mode).
///
/// # Safety
/// `pat` must be a valid pointer to `patlen` bytes.
pub unsafe fn set_mr_pattern(pat: *const c_char, patlen: usize) {
    xfree(MR_PATTERN.cast());
    if nvim_curwin_rl_with_rlc_s() != 0 {
        MR_PATTERN = reverse_text(pat);
    } else {
        MR_PATTERN = xstrnsave(pat, patlen);
    }
    MR_PATTERNLEN = patlen;
}

/// Free mr_pattern and reset mr_patternlen.
pub fn free_mr_pattern() {
    unsafe {
        xfree(MR_PATTERN.cast());
        MR_PATTERN = std::ptr::null_mut();
        MR_PATTERNLEN = 0;
    }
}

/// Set last search pattern (equivalent to set_last_search_pat).
///
/// # Safety
/// `s` must be a valid C string or null.
pub unsafe fn set_last_search_pat_batch(
    s: *const c_char,
    idx: c_int,
    magic: c_int,
    setlast: c_int,
) {
    if !(0..2).contains(&idx) {
        return;
    }
    let i = idx as usize;
    free_spat(&raw mut SPATS[i]);
    let s_is_empty = s.is_null() || *s == 0;
    if s_is_empty {
        SPATS[i].pat = std::ptr::null_mut();
        SPATS[i].patlen = 0;
    } else {
        let len = c_strlen(s);
        SPATS[i].patlen = len;
        SPATS[i].pat = xstrnsave(s, len);
    }
    SPATS[i].timestamp = os_time() as u64;
    SPATS[i].additional_data = std::ptr::null_mut();
    SPATS[i].magic = magic != 0;
    SPATS[i].no_scs = false;
    SPATS[i].off.dir = b'/' as i8;
    nvim_call_set_vv_searchforward();
    SPATS[i].off.line = false;
    SPATS[i].off.end = false;
    SPATS[i].off.off = 0;
    if setlast != 0 {
        LAST_IDX = idx;
    }
    if SAVE_LEVEL != 0 {
        free_spat(&raw mut SAVED_SPATS[i]);
        let src = &raw const SPATS[i];
        let dst = &raw mut SAVED_SPATS[i];
        std::ptr::copy_nonoverlapping(src, dst, 1);
        if SPATS[i].pat.is_null() {
            SAVED_SPATS[i].pat = std::ptr::null_mut();
            SAVED_SPATS[i].patlen = 0;
        } else {
            SAVED_SPATS[i].pat = xstrnsave(SPATS[i].pat, SPATS[i].patlen);
            SAVED_SPATS[i].patlen = SPATS[i].patlen;
        }
        SAVED_SPATS_LAST_IDX = LAST_IDX;
    }
    if nvim_get_p_hls() != 0 && idx == LAST_IDX && nvim_get_no_hlsearch() == 0 {
        nvim_redraw_all_later(UPD_SOME_VALID);
    }
}

/// Count bytes in a C string (like strlen).
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// ShaDa batch operations
// =============================================================================

/// Check if spats[idx].pat is NULL.
pub fn spats_pat_is_null(idx: c_int) -> bool {
    unsafe {
        if (0..2).contains(&idx) {
            SPATS[idx as usize].pat.is_null()
        } else {
            true
        }
    }
}

/// Get spats[idx] fields: pat pointer, patlen, magic, no_scs.
///
/// # Safety
/// `patlen`, `magic`, `no_scs` may be null (they'll be skipped if so).
pub unsafe fn spats_get_pat_and_len(
    idx: c_int,
    patlen: *mut usize,
    magic: *mut c_int,
    no_scs: *mut c_int,
) -> *const c_char {
    if !(0..2).contains(&idx) {
        return std::ptr::null();
    }
    let i = idx as usize;
    if !patlen.is_null() {
        *patlen = SPATS[i].patlen;
    }
    if !magic.is_null() {
        *magic = c_int::from(SPATS[i].magic);
    }
    if !no_scs.is_null() {
        *no_scs = c_int::from(SPATS[i].no_scs);
    }
    SPATS[i].pat
}

/// Copy spats[idx] to the provided buffer.
///
/// # Safety
/// `out` must be a valid, non-null pointer to a SearchPattern.
pub unsafe fn spat_memcpy_out(idx: c_int, out: *mut SearchPattern) {
    if (0..2).contains(&idx) && !out.is_null() {
        std::ptr::copy_nonoverlapping(&raw const SPATS[idx as usize], out, 1);
    }
}

/// Free spats[idx] and copy new value in.
///
/// # Safety
/// `inp` must be a valid, non-null pointer to a SearchPattern.
pub unsafe fn spat_memcpy_in(idx: c_int, inp: *const SearchPattern) {
    if (0..2).contains(&idx) && !inp.is_null() {
        free_spat(&raw mut SPATS[idx as usize]);
        std::ptr::copy_nonoverlapping(inp, &raw mut SPATS[idx as usize], 1);
    }
}

/// Free the pattern and additional_data of spats[idx].
pub fn spat_free(idx: c_int) {
    unsafe {
        if (0..2).contains(&idx) {
            free_spat(&raw mut SPATS[idx as usize]);
        }
    }
}

/// Clear spats[idx].off fields.
pub fn clear_spat_off(idx: c_int) {
    unsafe {
        if (0..2).contains(&idx) {
            let off = &raw mut SPATS[idx as usize].off;
            std::ptr::write(
                off,
                SearchOffset {
                    dir: 0,
                    line: false,
                    end: false,
                    off: 0,
                },
            );
        }
    }
}

/// Clear all spats entries (zeroes pat/additional_data to avoid dangling ptrs).
pub fn clear_spats() {
    unsafe {
        SPATS[0] = SearchPattern::empty();
        SPATS[1] = SearchPattern::empty();
    }
}

/// Check whether spats[last_idx].pat matches the given pattern string.
///
/// # Safety
/// `pat` must be a valid pointer to `patlen` bytes, or null.
pub unsafe fn spats_pat_matches(pat: *const c_char, patlen: usize) -> bool {
    if pat.is_null() || SPATS[LAST_IDX as usize].pat.is_null() {
        return false;
    }
    let stored_len = SPATS[LAST_IDX as usize].patlen;
    if stored_len != patlen {
        return false;
    }
    std::ptr::eq(pat, SPATS[LAST_IDX as usize].pat as *const c_char) || {
        let a = std::slice::from_raw_parts(pat as *const u8, patlen);
        let b = std::slice::from_raw_parts(SPATS[LAST_IDX as usize].pat as *const u8, patlen);
        a == b
    }
}

/// Copy spats[last_idx].pat using xstrnsave (for cache update).
/// Returns a newly allocated string, or null if no pattern.
/// Caller must free with xfree.
pub fn copy_spats_last_pat(out_len: &mut usize) -> *mut c_char {
    unsafe {
        if SPATS[LAST_IDX as usize].pat.is_null() {
            *out_len = 0;
            std::ptr::null_mut()
        } else {
            let len = SPATS[LAST_IDX as usize].patlen;
            *out_len = len;
            xstrnsave(SPATS[LAST_IDX as usize].pat, len)
        }
    }
}

/// Set spats[last_idx].pat for f_searchcount. Frees old pattern.
/// Returns false if pattern is empty (caller should skip).
///
/// # Safety
/// `pattern` must be a valid null-terminated C string.
pub unsafe fn searchcount_set_pattern(pattern: *const c_char) -> bool {
    if *pattern == 0 {
        return false;
    }
    let idx = LAST_IDX as usize;
    xfree(SPATS[idx].pat.cast());
    let len = c_strlen(pattern);
    SPATS[idx].patlen = len;
    SPATS[idx].pat = xstrnsave(pattern, len);
    true
}

/// Check if spats[last_idx].pat is non-NULL and non-empty.
pub fn searchcount_has_pattern() -> bool {
    unsafe {
        let idx = LAST_IDX as usize;
        !SPATS[idx].pat.is_null() && *SPATS[idx].pat != 0
    }
}

/// Get spats[last_idx].pat and patlen for use in searchit.
/// Returns (pat pointer, patlen).
pub fn get_last_pat_for_searchit() -> (*const c_char, usize) {
    unsafe {
        let idx = LAST_IDX as usize;
        (SPATS[idx].pat as *const c_char, SPATS[idx].patlen)
    }
}
