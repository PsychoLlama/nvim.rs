//! Pattern-in-buffer matching helpers migrated from cmdexpand.c.
//!
//! - `is_regex_match`: Check if a string matches a regex pattern
//! - `concat_pattern_with_buffer_match`: Concatenate pattern with buffer text
//! - `copy_substring_from_pos`: Copy a substring spanning buffer positions

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::c_void;

use libc::{c_char, c_int};

/// Maximum number of subexpressions in a regexp (from `regexp_defs.h`).
const NSUBEXP: usize = 10;

/// `RE_MAGIC` flag for `vim_regcomp` (verified by `_Static_assert` in `cmdexpand.c`).
const RE_MAGIC: c_int = 1;
/// `RE_STRING` flag for `vim_regcomp` (verified by `_Static_assert` in `cmdexpand.c`).
const RE_STRING: c_int = 2;

/// `kOptWopFlagExacttext` (verified by `_Static_assert` in `cmdexpand.c`).
const K_OPT_WOP_FLAG_EXACTTEXT: u32 = 0x08;

/// OK return value for C functions.
const OK: c_int = 1;
/// FAIL return value for C functions.
const FAIL: c_int = 0;

// Search flags (from search.h)
const SEARCH_OPT: c_int = 0x10;
const SEARCH_NOOF: c_int = 0x80;
const SEARCH_PEEK: c_int = 0x800;
const SEARCH_NFMSG: c_int = 0x08;
const SEARCH_START: c_int = 0x100;

// Direction values (from vim_defs.h)
const FORWARD: c_int = 1;

// Maximum matches for completion (from tag.h)
const TAG_MANY: usize = 300;

// =============================================================================
// Struct definitions
// =============================================================================

/// Structure matching `regmatch_T` for single-line matching.
/// Must match the C layout exactly (same as `eval/indexing.rs`).
#[repr(C)]
struct RegMatch {
    regprog: *mut c_void,
    startp: [*mut c_char; NSUBEXP],
    endp: [*mut c_char; NSUBEXP],
    rm_matchcol: c_int,
    rm_ic: bool,
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            regprog: std::ptr::null_mut(),
            startp: [std::ptr::null_mut(); NSUBEXP],
            endp: [std::ptr::null_mut(); NSUBEXP],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}

/// Position in file or buffer (matches `pos_T`).
/// Verified by `_Static_assert` in `cmdexpand.c`:
///   `sizeof(pos_T)` == 12, lnum@0, col@4, coladd@8
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    pub lnum: i32,
    pub col: i32,
    pub coladd: i32,
}

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Regex
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regexec_nl(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int;
    fn vim_regfree(prog: *mut c_void);

    // From search crate
    fn rs_pat_has_uppercase(pat: *const c_char) -> c_int;

    // From insexpand crate
    fn rs_find_word_end(ptr: *mut c_char) -> *mut c_char;

    // Error/message suppression (globals)
    static mut emsg_off: c_int;
    static mut msg_silent: c_int;

    // Option globals
    static mut p_ic: c_int;
    static mut p_scs: c_int;
    fn nvim_get_wop_flags() -> libc::c_uint;

    // Buffer line access (direct C functions)
    fn ml_get(lnum: i32) -> *const c_char;
    fn ml_get_len(lnum: i32) -> c_int;

    // Memory
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // String case conversion
    fn strcase_save(orig: *const c_char, upper: bool) -> *mut c_char;

    // Phase 2: expand_pattern_in_buf helpers
    fn nvim_cmdexpand_searchit(
        pos: *mut PosT,
        end_pos: *mut PosT,
        dir: c_int,
        pat: *mut c_char,
        patlen: usize,
        options: c_int,
    ) -> c_int;
    fn nvim_cmdexpand_curbuf_line_count() -> c_int;
    fn nvim_cmdexpand_char_avail() -> c_int;
    fn nvim_cmdexpand_vpeekc() -> c_int;
    fn nvim_cmdexpand_get_search_first_line() -> c_int;
    fn nvim_cmdexpand_get_search_last_line() -> c_int;
    fn nvim_cmdexpand_get_pre_incsearch_pos() -> PosT;
    static mut got_int: bool;
}

// =============================================================================
// Pattern matching helpers
// =============================================================================

/// Check if a string matches a regex pattern, honoring 'ignorecase'/'smartcase'.
///
/// # Safety
///
/// `pat` and `str_` must be valid null-terminated C strings.
unsafe fn is_regex_match(pat: *const c_char, str_: *const c_char) -> bool {
    // Fast path: exact string match
    if libc::strcmp(pat, str_) == 0 {
        return true;
    }

    let mut regmatch = RegMatch::default();

    emsg_off += 1;
    msg_silent += 1;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    emsg_off -= 1;
    msg_silent -= 1;

    if regmatch.regprog.is_null() {
        return false;
    }

    let ic = p_ic != 0;
    let scs = p_scs != 0;

    regmatch.rm_ic = ic;
    if ic && scs {
        regmatch.rm_ic = rs_pat_has_uppercase(pat) == 0;
    }

    emsg_off += 1;
    msg_silent += 1;
    let result = vim_regexec_nl(std::ptr::addr_of_mut!(regmatch), str_, 0) != 0;
    emsg_off -= 1;
    msg_silent -= 1;

    vim_regfree(regmatch.regprog);
    result
}

/// Concatenate pattern with text from the buffer at a position.
///
/// Returns a newly `xmalloc`'d string: `pat[0..pat_len] + word_at(end_match_pos)`.
/// If `lowercase` is true, the appended text is lowercased.
///
/// # Safety
///
/// `pat` must be valid for `pat_len` bytes. `end_match_pos` must be a valid `pos_T` pointer.
unsafe fn concat_pattern_with_buffer_match(
    pat: *const c_char,
    pat_len: c_int,
    end_match_pos: *const PosT,
    lowercase: bool,
) -> *mut c_char {
    let pos = &*end_match_pos;
    let line = ml_get(pos.lnum);
    let line_at_col = line.add(pos.col as usize);
    let word_end = rs_find_word_end(line_at_col.cast_mut());
    let match_len = word_end.offset_from(line_at_col) as usize;
    let pat_len_u = pat_len as usize;

    let result = xmalloc(match_len + pat_len_u + 1);

    // Copy pattern prefix
    std::ptr::copy_nonoverlapping(pat, result, pat_len_u);

    if match_len > 0 {
        if lowercase {
            let mword = xstrnsave(line_at_col, match_len);
            let lower = strcase_save(mword, false);
            xfree(mword.cast::<c_void>());
            std::ptr::copy_nonoverlapping(lower.cast_const(), result.add(pat_len_u), match_len);
            xfree(lower.cast::<c_void>());
        } else {
            std::ptr::copy_nonoverlapping(line_at_col, result.add(pat_len_u), match_len);
        }
    }

    *result.add(pat_len_u + match_len) = 0; // NUL terminate
    result
}

/// Copy a substring from the current buffer spanning from `start` to word
/// boundary after `end`. The result is stored in `*match_out` (`xmalloc`'d) and
/// the actual end position in `*match_end`.
///
/// Uses a Rust `Vec<u8>` internally instead of C `garray_T`.
///
/// # Safety
///
/// All pointer parameters must be valid. `match_out` will be set to an `xmalloc`'d buffer.
unsafe fn copy_substring_from_pos(
    start: *mut PosT,
    end: *mut PosT,
    match_out: *mut *mut c_char,
    match_end: *mut PosT,
) -> c_int {
    let start_pos = &*start;
    let end_pos = &*end;
    let wop_flags = nvim_get_wop_flags();
    let exacttext = (wop_flags & K_OPT_WOP_FLAG_EXACTTEXT) != 0;

    if start_pos.lnum > end_pos.lnum
        || (start_pos.lnum == end_pos.lnum && start_pos.col >= end_pos.col)
    {
        return FAIL;
    }

    let mut buf: Vec<u8> = Vec::with_capacity(128);
    let is_single_line = start_pos.lnum == end_pos.lnum;

    // Append start line from start->col to end of line (or end->col if single line)
    let start_line = ml_get(start_pos.lnum);
    let start_ptr = start_line.add(start_pos.col as usize);
    let segment_len = if is_single_line {
        (end_pos.col - start_pos.col) as usize
    } else {
        (ml_get_len(start_pos.lnum) - start_pos.col) as usize
    };
    let segment = std::slice::from_raw_parts(start_ptr.cast::<u8>(), segment_len);
    buf.extend_from_slice(segment);
    if !is_single_line {
        if exacttext {
            buf.extend_from_slice(b"\\n");
        } else {
            buf.push(b'\n');
        }
    }

    // Append full lines between start and end
    if !is_single_line {
        let mut lnum = start_pos.lnum + 1;
        while lnum < end_pos.lnum {
            let line = ml_get(lnum);
            let line_len = ml_get_len(lnum) as usize;
            let line_slice = std::slice::from_raw_parts(line.cast::<u8>(), line_len);
            buf.extend_from_slice(line_slice);
            if exacttext {
                buf.extend_from_slice(b"\\n");
            } else {
                buf.push(b'\n');
            }
            lnum += 1;
        }
    }

    // Append partial end line (up to word end)
    let end_line = ml_get(end_pos.lnum);
    let end_col_ptr = end_line.add(end_pos.col as usize);
    let word_end = rs_find_word_end(end_col_ptr.cast_mut());
    let word_end_offset = word_end.offset_from(end_line) as usize;

    let copy_start = if is_single_line {
        end_pos.col as usize
    } else {
        0
    };
    let copy_len = word_end_offset - copy_start;
    let end_slice = std::slice::from_raw_parts(end_line.add(copy_start).cast::<u8>(), copy_len);
    buf.extend_from_slice(end_slice);

    // NUL-terminate and copy into xmalloc'd buffer
    buf.push(0);
    let result = xmalloc(buf.len());
    std::ptr::copy_nonoverlapping(buf.as_ptr(), result.cast::<u8>(), buf.len());

    *match_out = result;
    (*match_end).lnum = end_pos.lnum;
    (*match_end).col = word_end_offset as i32;

    OK
}

// =============================================================================
// expand_pattern_in_buf implementation
// =============================================================================

/// Outcome of processing one match iteration.
enum MatchOutcome {
    /// Store this match pointer.
    Store(*mut c_char),
    /// Skip this iteration (match not valid or already freed).
    Skip,
    /// Done: no more matches in range.
    Done,
}

/// Process one search result: extract and validate the match string.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn process_one_match(
    pat: *mut c_char,
    pat_len: usize,
    cur_match_pos: *mut PosT,
    end_match_pos: *mut PosT,
    exacttext: bool,
) -> MatchOutcome {
    let mut full_match: *mut c_char = std::ptr::null_mut();
    let mut word_end_pos = PosT::default();

    if copy_substring_from_pos(
        cur_match_pos,
        end_match_pos,
        std::ptr::addr_of_mut!(full_match),
        std::ptr::addr_of_mut!(word_end_pos),
    ) == FAIL
    {
        return MatchOutcome::Done;
    }

    if exacttext {
        return MatchOutcome::Store(full_match);
    }

    // Build match = pat + word from buffer
    let mut m =
        concat_pattern_with_buffer_match(pat, pat_len as c_int, end_match_pos.cast_const(), false);
    if !is_regex_match(m, full_match) {
        xfree(m.cast::<c_void>());
        m = concat_pattern_with_buffer_match(
            pat,
            pat_len as c_int,
            end_match_pos.cast_const(),
            true,
        );
        if !is_regex_match(m, full_match) {
            xfree(m.cast::<c_void>());
            xfree(full_match.cast::<c_void>());
            return MatchOutcome::Skip;
        }
    }
    xfree(full_match.cast::<c_void>());
    MatchOutcome::Store(m)
}

/// Transfer collected matches from a Vec into an xmalloc'd C pointer array.
///
/// # Safety
///
/// `matches_out` and `num_matches_out` must be valid writable pointers.
unsafe fn commit_matches(
    ga: Vec<*mut c_char>,
    matches_out: *mut *mut *mut c_char,
    num_matches_out: *mut c_int,
) {
    let count = ga.len();
    if count == 0 {
        *matches_out = std::ptr::null_mut();
        *num_matches_out = 0;
        return;
    }
    #[allow(clippy::cast_ptr_alignment)]
    let arr = xmalloc(count * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    for (i, p) in ga.into_iter().enumerate() {
        *arr.add(i) = p;
    }
    *matches_out = arr;
    *num_matches_out = count as c_int;
}

/// Search for strings matching `pat` in the buffer and return them.
///
/// Ports the C `expand_pattern_in_buf` function to Rust.
///
/// # Safety
///
/// `pat` must be a valid null-terminated C string. `matches_out` and
/// `num_matches_out` must be valid writable pointers.
#[allow(clippy::too_many_lines)]
unsafe fn expand_pattern_in_buf_impl(
    pat: *mut c_char,
    dir: c_int,
    matches_out: *mut *mut *mut c_char,
    num_matches_out: *mut c_int,
) -> c_int {
    let wop_flags = nvim_get_wop_flags();
    let exacttext = (wop_flags & K_OPT_WOP_FLAG_EXACTTEXT) != 0;
    let search_first_line = nvim_cmdexpand_get_search_first_line();
    let search_last_line = nvim_cmdexpand_get_search_last_line();
    let has_range = search_first_line != 0;

    *matches_out = std::ptr::null_mut();
    *num_matches_out = 0;

    if pat.is_null() || *pat == 0 {
        return FAIL;
    }

    let pat_len = libc::strlen(pat);
    let mut cur_match_pos = if has_range {
        PosT {
            lnum: search_first_line,
            col: 0,
            coladd: 0,
        }
    } else {
        nvim_cmdexpand_get_pre_incsearch_pos()
    };
    let mut prev_match_pos = PosT::default();
    let search_flags = SEARCH_OPT
        | SEARCH_NOOF
        | SEARCH_PEEK
        | SEARCH_NFMSG
        | if has_range { SEARCH_START } else { 0 };

    let mut ga: Vec<*mut c_char> = Vec::with_capacity(10);
    let mut looped_around = false;
    let mut compl_started = false;

    loop {
        let mut end_match_pos = PosT::default();
        let found = nvim_cmdexpand_searchit(
            std::ptr::addr_of_mut!(cur_match_pos),
            std::ptr::addr_of_mut!(end_match_pos),
            dir,
            pat,
            pat_len,
            search_flags,
        );
        if found == FAIL {
            break;
        }
        if has_range
            && (cur_match_pos.lnum < search_first_line || cur_match_pos.lnum > search_last_line)
        {
            break;
        }
        if compl_started {
            let looped = if dir == FORWARD {
                pos_le(cur_match_pos, prev_match_pos)
            } else {
                pos_le(prev_match_pos, cur_match_pos)
            };
            if looped {
                if looped_around {
                    break;
                }
                looped_around = true;
            }
        }
        compl_started = true;
        prev_match_pos = cur_match_pos;

        if nvim_cmdexpand_char_avail() != 0 || got_int {
            if got_int {
                nvim_cmdexpand_vpeekc();
                got_int = false;
            }
            for p in &ga {
                xfree((*p).cast::<c_void>());
            }
            return FAIL;
        }

        if end_match_pos.lnum > nvim_cmdexpand_curbuf_line_count() {
            cur_match_pos = PosT {
                lnum: 1,
                col: 0,
                coladd: 0,
            };
            continue;
        }

        match process_one_match(
            pat,
            pat_len,
            std::ptr::addr_of_mut!(cur_match_pos),
            std::ptr::addr_of_mut!(end_match_pos),
            exacttext,
        ) {
            MatchOutcome::Done => break,
            MatchOutcome::Skip => {
                if has_range {
                    cur_match_pos = end_match_pos;
                }
                continue;
            }
            MatchOutcome::Store(m) => {
                let is_dup = ga.iter().any(|&p| libc::strcmp(m, p) == 0);
                if is_dup {
                    xfree(m.cast::<c_void>());
                } else {
                    ga.push(m);
                    if ga.len() > TAG_MANY {
                        break;
                    }
                }
            }
        }

        if has_range {
            cur_match_pos = end_match_pos;
        }
    }

    commit_matches(ga, matches_out, num_matches_out);
    OK
}

/// Compare two positions: returns true if `a <= b`.
const fn pos_le(a: PosT, b: PosT) -> bool {
    a.lnum < b.lnum || (a.lnum == b.lnum && a.col <= b.col)
}

// =============================================================================
// FFI Interface
// =============================================================================

/// FFI entry point for `is_regex_match`.
///
/// # Safety
///
/// `pat` and `str_` must be valid null-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_regex_match(pat: *const c_char, str_: *const c_char) -> c_int {
    c_int::from(is_regex_match(pat, str_))
}

/// FFI entry point for `concat_pattern_with_buffer_match`.
///
/// # Safety
///
/// `pat` must be valid for `pat_len` bytes. `end_match_pos` must be a valid `pos_T` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_concat_pattern_with_buffer_match(
    pat: *const c_char,
    pat_len: c_int,
    end_match_pos: *const PosT,
    lowercase: c_int,
) -> *mut c_char {
    concat_pattern_with_buffer_match(pat, pat_len, end_match_pos, lowercase != 0)
}

/// FFI entry point for `copy_substring_from_pos`.
///
/// # Safety
///
/// All pointer parameters must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_copy_substring_from_pos(
    start: *mut PosT,
    end: *mut PosT,
    match_out: *mut *mut c_char,
    match_end: *mut PosT,
) -> c_int {
    copy_substring_from_pos(start, end, match_out, match_end)
}

/// FFI entry point for `expand_pattern_in_buf`.
///
/// # Safety
///
/// `pat` must be a valid null-terminated C string. Output pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_pattern_in_buf(
    pat: *mut c_char,
    dir: c_int,
    matches_out: *mut *mut *mut c_char,
    num_matches_out: *mut c_int,
) -> c_int {
    expand_pattern_in_buf_impl(pat, dir, matches_out, num_matches_out)
}
