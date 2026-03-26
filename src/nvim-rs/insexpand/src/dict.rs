//! Dictionary and thesaurus completion support.
//!
//! This module provides helper functions for dictionary and thesaurus completion.
//! The core file I/O and regex operations remain in C due to their complexity,
//! but Rust provides utilities for string processing and state management.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    // CTRL-X mode checking

    // State accessors

    // UTF-8 functions
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;
    fn mb_get_class(ptr: *const c_char) -> c_int;
}

// CTRL-X mode constants
const CTRL_X_WANT_IDENT: c_int = 0x100;
const CTRL_X_DICTIONARY: c_int = 9 + CTRL_X_WANT_IDENT;
const CTRL_X_THESAURUS: c_int = 10 + CTRL_X_WANT_IDENT;

/// Skip whitespace and punctuation to find word start.
///
/// This is similar to find_word_start but specifically for thesaurus processing.
/// Returns a pointer to the first character of the next word.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_find_word_start(mut ptr: *mut c_char) -> *mut c_char {
    // Skip whitespace and punctuation (class <= 1)
    while *ptr != 0 && *ptr != b'\n' as c_char && mb_get_class(ptr) <= 1 {
        ptr = ptr.add(utfc_ptr2len(ptr) as usize);
    }
    ptr
}

/// Find the end of a word for thesaurus processing.
///
/// Unlike the standard word end finder, this handles Japanese words
/// where characters may be in different classes, only separating words
/// with single-byte non-word characters.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_find_word_end(mut ptr: *mut c_char) -> *mut c_char {
    while *ptr != 0 {
        let len = utfc_ptr2len(ptr);
        // For multi-byte characters, continue regardless of class
        if len > 1 {
            ptr = ptr.add(len as usize);
        } else if mb_get_class(ptr) <= 1 {
            // Single-byte non-word character - stop here
            break;
        } else {
            ptr = ptr.add(1);
        }
    }
    ptr
}

/// Calculate the length of a word from start to end pointers.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_dict_word_len(start: *const c_char, end: *const c_char) -> c_int {
    if start.is_null() || end.is_null() || end < start {
        return 0;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        end.offset_from(start) as c_int
    }
}

/// Check if a word matches another word (case-sensitive).
///
/// Returns 1 if words match exactly, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_dict_words_match(
    word1: *const c_char,
    len1: c_int,
    word2: *const c_char,
    len2: c_int,
) -> c_int {
    if word1.is_null() || word2.is_null() || len1 != len2 || len1 < 0 {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    for i in 0..len1 as usize {
        if *word1.add(i) != *word2.add(i) {
            return 0;
        }
    }

    1
}

/// Check if a line is empty or contains only whitespace.
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_line_is_empty(line: *const c_char) -> c_int {
    if line.is_null() {
        return 1;
    }

    let mut ptr = line;
    while *ptr != 0 && *ptr != b'\n' as c_char {
        // If we find any non-whitespace character
        if *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
            return 0;
        }
        ptr = ptr.add(1);
    }

    1
}

/// Skip a word in the line (move past current word and following whitespace).
///
/// Useful for iterating through words in a thesaurus line.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_skip_word(ptr: *mut c_char) -> *mut c_char {
    // Find end of current word
    let end = rs_dict_find_word_end(ptr);
    // Then find start of next word
    rs_dict_find_word_start(end)
}

/// Count words in a line.
///
/// Returns the number of whitespace-separated words.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_dict_count_words_in_line(mut ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut count = 0;

    loop {
        // Skip whitespace
        while *ptr != 0 && *ptr != b'\n' as c_char {
            if *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
                break;
            }
            ptr = ptr.add(1);
        }

        // End of line?
        if *ptr == 0 || *ptr == b'\n' as c_char {
            break;
        }

        // Found a word
        count += 1;

        // Skip to end of word
        while *ptr != 0 && *ptr != b'\n' as c_char {
            if *ptr == b' ' as c_char || *ptr == b'\t' as c_char {
                break;
            }
            ptr = ptr.add(1);
        }
    }

    count
}

// =============================================================================
// Phase 5: Extended Dictionary and Thesaurus Functions
// =============================================================================

// Additional C accessor functions

/// Compare two strings case-insensitively (ASCII only).
#[no_mangle]
#[allow(
    clippy::missing_const_for_fn,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_dict_stricmp(
    s1: *const c_char,
    s2: *const c_char,
    len: c_int,
) -> c_int {
    if s1.is_null() || s2.is_null() || len < 0 {
        return c_int::from(s1 != s2);
    }

    for i in 0..len as usize {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);

        // Convert to lowercase for comparison (ASCII only)
        // ASCII 'A'-'Z' is 65-90, safe range for c_char
        let lc1 = if (65..=90).contains(&c1) { c1 + 32 } else { c1 };
        let lc2 = if (65..=90).contains(&c2) { c2 + 32 } else { c2 };

        if lc1 != lc2 {
            return c_int::from(lc1 > lc2) - c_int::from(lc1 < lc2);
        }
    }

    0
}

/// Check if a string starts with another string (case-sensitive).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_dict_starts_with(
    str_ptr: *const c_char,
    prefix: *const c_char,
    prefix_len: c_int,
) -> c_int {
    if str_ptr.is_null() || prefix.is_null() || prefix_len < 0 {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    for i in 0..prefix_len as usize {
        let c1 = *str_ptr.add(i);
        let c2 = *prefix.add(i);
        if c1 == 0 || c1 != c2 {
            return 0;
        }
    }

    1
}

/// Check if a string starts with another string (case-insensitive, ASCII).
#[no_mangle]
#[allow(clippy::missing_const_for_fn, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_dict_starts_with_icase(
    str_ptr: *const c_char,
    prefix: *const c_char,
    prefix_len: c_int,
) -> c_int {
    if str_ptr.is_null() || prefix.is_null() || prefix_len < 0 {
        return 0;
    }

    for i in 0..prefix_len as usize {
        let c1 = *str_ptr.add(i);
        let c2 = *prefix.add(i);

        if c1 == 0 {
            return 0;
        }

        // Convert to lowercase for comparison
        // ASCII 'A'-'Z' is 65-90
        let lc1 = if (65..=90).contains(&c1) { c1 + 32 } else { c1 };
        let lc2 = if (65..=90).contains(&c2) { c2 + 32 } else { c2 };

        if lc1 != lc2 {
            return 0;
        }
    }

    1
}

/// Extract the Nth word from a line.
///
/// Returns pointers to start and end of the word via out parameters.
/// Returns 1 if word found, 0 if not found.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_dict_get_nth_word(
    line: *const c_char,
    n: c_int,
    start_out: *mut *const c_char,
    end_out: *mut *const c_char,
) -> c_int {
    if line.is_null() || n < 0 || start_out.is_null() || end_out.is_null() {
        return 0;
    }

    let mut ptr = line;
    let mut word_idx = 0;

    loop {
        // Skip whitespace
        while *ptr != 0 && *ptr != b'\n' as c_char {
            if *ptr != b' ' as c_char && *ptr != b'\t' as c_char {
                break;
            }
            ptr = ptr.add(1);
        }

        // End of line?
        if *ptr == 0 || *ptr == b'\n' as c_char {
            break;
        }

        // Found a word
        let word_start = ptr;

        // Skip to end of word
        while *ptr != 0
            && *ptr != b'\n' as c_char
            && *ptr != b' ' as c_char
            && *ptr != b'\t' as c_char
        {
            ptr = ptr.add(1);
        }

        if word_idx == n {
            *start_out = word_start;
            *end_out = ptr;
            return 1;
        }

        word_idx += 1;
    }

    0
}

// =============================================================================
// Phase 3 (pass 5): rs_get_next_dict_tsr_completion
// =============================================================================

extern "C" {
    /// Check if thesaurus function completion is active for the given type.
    fn rs_thesaurus_func_complete(compl_type: c_int) -> c_int;
    // nvim_expand_by_function_impl: deleted (Phase 30), call nvim_expand_by_function_full_impl directly
    #[link_name = "nvim_expand_by_function_full_impl"]
    fn nvim_expand_by_function_full_dict(
        type_: c_int,
        base: *mut std::ffi::c_char,
        cb: *mut std::ffi::c_void,
    );
    // nvim_ins_compl_dictionaries_impl: deleted (Phase 23), inlined below as ins_compl_dictionaries
    /// Returns the effective thesaurus option (curbuf->b_p_tsr or p_tsr).
    fn nvim_get_curbuf_b_p_tsr() -> *const c_char;
    /// Returns the effective dictionary option (curbuf->b_p_dict or p_dict).
    fn nvim_get_curbuf_b_p_dict() -> *const c_char;
}

// =============================================================================
// Phase 23: ins_compl_dictionaries inlined from deleted C nvim_ins_compl_dictionaries_impl
// =============================================================================

/// Rust layout for C regmatch_T (from regexp_defs.h).
/// NSUBEXP = 10.
#[repr(C)]
struct RegMatchT {
    regprog: *mut core::ffi::c_void,
    startp: [*mut c_char; 10],
    endp: [*mut c_char; 10],
    rm_matchcol: c_int,
    rm_ic: bool,
}

// LSIZE = 512 (matches C LSIZE define in vim.h)
const LSIZE_DICT: usize = 512;
const OK_DICT: c_int = 1;
const FAIL_DICT: c_int = 0;
const RE_MAGIC_DICT: c_int = 1;
const DICT_FIRST: c_int = 1;
const DICT_EXACT: c_int = 2;
const EW_FILE: c_int = 0x02;
const EW_SILENT: c_int = 0x20;
const SHM_COMPLETIONSCAN: c_int = b'C' as c_int;
const HLF_R_DICT: c_int = 6; // HLF_R = 6 from highlight_defs.h
const FUZZY_SCORE_NONE: c_int = c_int::MIN;
const FORWARD_DICT: c_int = 1; // FORWARD = 1

#[allow(clashing_extern_declarations)]
extern "C" {
    fn vim_regcomp(pat: *const c_char, magic: c_int) -> *mut core::ffi::c_void;
    fn vim_regexec(rmp: *mut RegMatchT, line: *mut c_char, col: c_int) -> c_int;
    fn vim_regfree(prog: *mut core::ffi::c_void);
    fn expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_file: *mut c_int,
        file: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    fn FreeWild(count: c_int, files: *mut *mut c_char);
    fn os_fopen(path: *const c_char, mode: *const c_char) -> *mut core::ffi::c_void;
    fn vim_fgets(buf: *mut c_char, size: c_int, fp: *mut core::ffi::c_void) -> c_int;
    fn fclose(fp: *mut core::ffi::c_void) -> c_int;
    fn line_breakcheck();
    fn spell_dump_compl(pat: *const c_char, ic: bool, dir: *mut c_int, dumpflags: c_int);
    #[link_name = "ignorecase"]
    fn ignorecase_dict(pat: *mut c_char) -> c_int;
    fn vim_strsave_escaped(str_: *const c_char, esc_chars: *const c_char) -> *mut c_char;
    fn vim_snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
    fn msg_trunc(s: *mut c_char, force: bool, attr: c_int);
    #[link_name = "msg_ext_set_kind"]
    fn msg_ext_set_kind_dict(kind: *const c_char);
    fn fuzzy_match_str_in_line(
        ptr: *mut *mut c_char,
        str_: *const c_char,
        len: *mut c_int,
        num_words: *mut c_int,
        score: *mut c_int,
    ) -> bool;
    fn vim_iswordc(c: c_int) -> bool;
    #[link_name = "copy_option_part"]
    fn copy_option_part_dict(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
    #[link_name = "vim_strchr"]
    fn vim_strchr_dict(str_: *const c_char, c: c_int) -> *const c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    #[link_name = "shortmess"]
    fn shortmess_dict(x: c_int) -> bool;
    fn rs_magic_isset() -> c_int;
    fn rs_cot_fuzzy() -> c_int;
    fn rs_ins_compl_leader() -> *const c_char;
    fn rs_ins_compl_leader_len() -> usize;
    fn rs_ins_compl_check_keys(max_count: c_int, must_redraw: c_int);
    fn nvim_ins_compl_add_infercase_ffi(
        str_: *const c_char,
        len: c_int,
        icase: c_int,
        fname: *const c_char,
        dir: c_int,
        cont_s_ipos: c_int,
        score: c_int,
    ) -> c_int;
    fn rs_ctrl_x_mode_line_or_eval() -> c_int;
    #[link_name = "rs_ctrl_x_mode_normal"]
    fn rs_ctrl_x_mode_normal_dict() -> c_int;
    #[link_name = "nvim_win_get_p_spell"]
    fn nvim_win_get_p_spell_dict(wp: *mut u8) -> c_int;
    #[link_name = "nvim_get_curwin"]
    fn nvim_get_curwin_dict() -> *mut u8;
    #[link_name = "nvim_get_p_inf"]
    fn nvim_curbuf_get_b_p_inf_dict() -> c_int;
    #[link_name = "p_scs"]
    static mut p_scs_dict: c_int;
    #[link_name = "p_ic"]
    static mut p_ic_dict: c_int;
    #[link_name = "got_int"]
    static got_int_dict: bool;
    #[link_name = "msg_hist_off"]
    static mut msg_hist_off_dict: bool;
    #[link_name = "IObuff"]
    static mut IObuff_dict: [c_char; 1025];
    #[link_name = "xmalloc"]
    fn xmalloc_dict(size: usize) -> *mut c_char;
    #[link_name = "xfree"]
    fn xfree_dict(ptr: *mut u8);

    // For compl_num_bests update in fuzzy path
    #[link_name = "nvim_compl_match_get_next"]
    fn nvim_compl_match_get_next_dict(
        m: crate::match_list::ComplMatch,
    ) -> crate::match_list::ComplMatch;
    #[link_name = "nvim_compl_match_get_score"]
    fn nvim_compl_match_get_score_dict(m: crate::match_list::ComplMatch) -> c_int;
}

const IOSIZE_DICT: usize = 1025;

/// Inline of deleted C `nvim_ins_compl_dictionaries_impl` (Phase 23).
///
/// Add any identifiers that match the given pattern compl_pattern in the list of
/// dictionary/thesaurus files. dict_start is the option string, flags is DICT_FIRST/DICT_EXACT.
///
/// # Safety
/// Requires valid global completion state.
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::useless_let_if_seq,
    clippy::while_immutable_condition,
    static_mut_refs
)]
unsafe fn ins_compl_dictionaries(dict_start: *const c_char, flags: c_int, thesaurus: c_int) {
    let mut dict = dict_start.cast_mut();
    let pat = crate::vars::compl_pattern.data.cast_const();
    let mut regmatch = RegMatchT {
        regprog: core::ptr::null_mut(),
        startp: [core::ptr::null_mut(); 10],
        endp: [core::ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };

    // If dict is empty, fall back to spell or bail
    if *dict == 0 {
        if thesaurus == 0 && nvim_win_get_p_spell_dict(nvim_get_curwin_dict()) != 0 {
            dict = c"spell".as_ptr().cast_mut();
        } else {
            return;
        }
    }

    let mut buf = xmalloc_dict(LSIZE_DICT);
    let save_p_scs = p_scs_dict;
    if nvim_curbuf_get_b_p_inf_dict() != 0 {
        p_scs_dict = 0;
    }

    if rs_ctrl_x_mode_line_or_eval() != 0 {
        let pat_esc = vim_strsave_escaped(pat, c"\\".as_ptr());
        let len = libc_strlen(pat_esc) + 10;
        let ptr = xmalloc_dict(len);
        vim_snprintf(ptr, len, c"^\\s*\\zs\\V%s".as_ptr(), pat_esc);
        regmatch.regprog = vim_regcomp(ptr, RE_MAGIC_DICT);
        xfree_dict(pat_esc.cast());
        xfree_dict(ptr.cast());
    } else {
        regmatch.regprog = vim_regcomp(
            pat,
            if rs_magic_isset() != 0 {
                RE_MAGIC_DICT
            } else {
                0
            },
        );
        if regmatch.regprog.is_null() {
            p_scs_dict = save_p_scs;
            vim_regfree(regmatch.regprog);
            xfree_dict(buf.cast());
            return;
        }
    }

    regmatch.rm_ic = ignorecase_dict(pat.cast_mut()) != 0;
    let mut dir = crate::vars::nvim_get_compl_direction();

    while *dict != 0 && !got_int_dict && crate::vars::nvim_get_compl_interrupted() == 0 {
        let mut count: c_int = 0;
        let mut files: *mut *mut c_char = core::ptr::null_mut();

        if flags == DICT_EXACT {
            count = 1;
            files = &raw mut dict;
        } else {
            copy_option_part_dict(&raw mut dict, buf, LSIZE_DICT, c",".as_ptr());
            if thesaurus == 0 && strcmp(buf.cast_const(), c"spell".as_ptr()) == 0 {
                count = -1;
            } else if !vim_strchr_dict(buf.cast_const(), c_int::from(b'`')).is_null()
                || expand_wildcards(
                    1,
                    &raw mut buf,
                    &raw mut count,
                    &raw mut files,
                    EW_FILE | EW_SILENT,
                ) != OK_DICT
            {
                count = 0;
            }
        }

        if count == -1 {
            // Spell completion
            let ptr: *const c_char = if *pat == b'\\' as c_char && *pat.add(1) == b'<' as c_char {
                pat.add(2)
            } else {
                pat
            };
            spell_dump_compl(ptr, regmatch.rm_ic, &raw mut dir, 0);
        } else if count > 0 {
            let leader: *const c_char = if rs_cot_fuzzy() != 0 {
                rs_ins_compl_leader()
            } else {
                core::ptr::null()
            };
            let leader_len: c_int = if rs_cot_fuzzy() != 0 {
                rs_ins_compl_leader_len() as c_int
            } else {
                0
            };

            let mut i = 0;
            while i < count && !got_int_dict && crate::rs_ins_compl_interrupted() == 0 {
                let fp = os_fopen(*files.add(i as usize), c"r".as_ptr());
                if flags != DICT_EXACT
                    && !shortmess_dict(SHM_COMPLETIONSCAN)
                    && crate::vars::nvim_get_compl_autocomplete() == 0
                {
                    msg_hist_off_dict = true;
                    msg_ext_set_kind_dict(c"completion".as_ptr());
                    vim_snprintf(
                        IObuff_dict.as_mut_ptr(),
                        IOSIZE_DICT,
                        c"Scanning dictionary: %s".as_ptr(),
                        *files.add(i as usize),
                    );
                    msg_trunc(IObuff_dict.as_mut_ptr(), true, HLF_R_DICT);
                }
                if fp.is_null() {
                    i += 1;
                    continue;
                }
                while !got_int_dict
                    && crate::rs_ins_compl_interrupted() == 0
                    && vim_fgets(buf, LSIZE_DICT as c_int, fp) == 0
                {
                    let mut lptr = buf;
                    if rs_cot_fuzzy() != 0 && leader_len > 0 {
                        let line_end = crate::rs_find_line_end(lptr);
                        while lptr < line_end {
                            let mut score: c_int = 0;
                            let mut len: c_int = 0;
                            if fuzzy_match_str_in_line(
                                &raw mut lptr,
                                leader,
                                &raw mut len,
                                core::ptr::null_mut(),
                                &raw mut score,
                            ) {
                                let end_ptr = if rs_ctrl_x_mode_line_or_eval() != 0 {
                                    crate::rs_find_line_end(lptr)
                                } else {
                                    crate::rs_find_word_end(lptr)
                                };
                                let add_r = nvim_ins_compl_add_infercase_ffi(
                                    lptr.cast_const(),
                                    end_ptr.offset_from(lptr) as c_int,
                                    p_ic_dict,
                                    (*files.add(i as usize)).cast_const(),
                                    dir,
                                    0,
                                    score,
                                );
                                if add_r == FAIL_DICT {
                                    break;
                                }
                                lptr = end_ptr;
                                if crate::vars::nvim_get_compl_get_longest() != 0
                                    && rs_ctrl_x_mode_normal_dict() != 0
                                {
                                    let next = nvim_compl_match_get_next_dict(
                                        crate::match_list::ComplMatch(
                                            crate::vars::nvim_get_compl_first_match(),
                                        ),
                                    );
                                    if !next.is_null()
                                        && score == nvim_compl_match_get_score_dict(next)
                                    {
                                        crate::vars::nvim_set_compl_num_bests(
                                            crate::vars::nvim_get_compl_num_bests() + 1,
                                        );
                                    }
                                }
                            }
                        }
                    } else if !regmatch.regprog.is_null() {
                        while vim_regexec(&raw mut regmatch, buf, lptr.offset_from(buf) as c_int)
                            != 0
                        {
                            lptr = regmatch.startp[0];
                            lptr = if rs_ctrl_x_mode_line_or_eval() != 0 {
                                crate::rs_find_line_end(lptr)
                            } else {
                                crate::rs_find_word_end(lptr)
                            };
                            let mut add_r = nvim_ins_compl_add_infercase_ffi(
                                regmatch.startp[0].cast_const(),
                                lptr.offset_from(regmatch.startp[0]) as c_int,
                                p_ic_dict,
                                (*files.add(i as usize)).cast_const(),
                                dir,
                                0,
                                FUZZY_SCORE_NONE,
                            );
                            if thesaurus != 0 {
                                // thesaurus_add_words_in_line inlined
                                lptr = buf;
                                while !got_int_dict {
                                    lptr = crate::rs_find_word_start(lptr);
                                    if *lptr == 0 || *lptr == b'\n' as c_char {
                                        break;
                                    }
                                    let wstart = lptr;
                                    while *lptr != 0 {
                                        let l = utfc_ptr2len(lptr) as usize;
                                        if l < 2 && !vim_iswordc(c_int::from(*lptr as u8)) {
                                            break;
                                        }
                                        lptr = lptr.add(l);
                                    }
                                    if wstart != regmatch.startp[0] {
                                        add_r = nvim_ins_compl_add_infercase_ffi(
                                            wstart.cast_const(),
                                            lptr.offset_from(wstart) as c_int,
                                            p_ic_dict,
                                            (*files.add(i as usize)).cast_const(),
                                            dir,
                                            0,
                                            FUZZY_SCORE_NONE,
                                        );
                                        if add_r == FAIL_DICT {
                                            break;
                                        }
                                    }
                                }
                            }
                            if add_r == OK_DICT {
                                dir = FORWARD_DICT;
                            } else if add_r == FAIL_DICT {
                                break;
                            }
                            if *lptr == b'\n' as c_char || got_int_dict {
                                break;
                            }
                        }
                    }
                    line_breakcheck();
                    rs_ins_compl_check_keys(50, 0);
                }
                fclose(fp);
                i += 1;
            }
            if flags != DICT_EXACT {
                FreeWild(count, files);
            }
        }
        if flags != 0 {
            break;
        }
    }

    p_scs_dict = save_p_scs;
    vim_regfree(regmatch.regprog);
    xfree_dict(buf.cast());
}

// strlen needed for ins_compl_dictionaries
#[allow(clippy::cast_sign_loss)]
const unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}

/// Get the next set of words matching compl_pattern in dictionary or thesaurus files.
///
/// Determines the dict/thesaurus option string and delegates to either
/// expand_by_function (for thesaurusfunc) or ins_compl_dictionaries (for file scanning).
///
/// # Safety
/// Requires valid completion state. `dict` may be null (use buffer option).
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_dict_tsr_completion(
    compl_type: c_int,
    dict: *mut c_char,
    dict_f: c_int,
) {
    if rs_thesaurus_func_complete(compl_type) != 0 {
        // nvim_expand_by_function_impl inlined (Phase 30)
        nvim_expand_by_function_full_dict(
            compl_type,
            crate::vars::compl_pattern.data,
            core::ptr::null_mut(),
        );
    } else {
        let effective_dict = if dict.is_null() {
            if compl_type == CTRL_X_THESAURUS {
                nvim_get_curbuf_b_p_tsr()
            } else {
                nvim_get_curbuf_b_p_dict()
            }
        } else {
            dict.cast_const()
        };
        let flags = if dict.is_null() { 0 } else { dict_f };
        let is_thesaurus = c_int::from(compl_type == CTRL_X_THESAURUS);
        ins_compl_dictionaries(effective_dict, flags, is_thesaurus);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_DICTIONARY, 9 + 0x100);
        assert_eq!(CTRL_X_THESAURUS, 10 + 0x100);
    }

    #[test]
    fn test_word_len_null() {
        unsafe {
            assert_eq!(rs_dict_word_len(std::ptr::null(), std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_word_len() {
        unsafe {
            let s = b"hello\0";
            let start = s.as_ptr().cast::<c_char>();
            let end = start.add(5);
            assert_eq!(rs_dict_word_len(start, end), 5);
        }
    }

    #[test]
    fn test_words_match() {
        unsafe {
            let w1 = b"hello\0";
            let w2 = b"hello\0";
            let w3 = b"world\0";

            assert_eq!(
                rs_dict_words_match(
                    w1.as_ptr().cast::<c_char>(),
                    5,
                    w2.as_ptr().cast::<c_char>(),
                    5
                ),
                1
            );
            assert_eq!(
                rs_dict_words_match(
                    w1.as_ptr().cast::<c_char>(),
                    5,
                    w3.as_ptr().cast::<c_char>(),
                    5
                ),
                0
            );
        }
    }

    #[test]
    fn test_words_match_different_lengths() {
        unsafe {
            let w1 = b"hi\0";
            let w2 = b"hello\0";

            assert_eq!(
                rs_dict_words_match(
                    w1.as_ptr().cast::<c_char>(),
                    2,
                    w2.as_ptr().cast::<c_char>(),
                    5
                ),
                0
            );
        }
    }

    #[test]
    fn test_line_is_empty() {
        unsafe {
            let empty = b"\0";
            let whitespace = b"   \t  \0";
            let content = b"  hello  \0";

            assert_eq!(rs_dict_line_is_empty(empty.as_ptr().cast::<c_char>()), 1);
            assert_eq!(
                rs_dict_line_is_empty(whitespace.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(rs_dict_line_is_empty(content.as_ptr().cast::<c_char>()), 0);
        }
    }

    #[test]
    fn test_count_words() {
        unsafe {
            let one = b"hello\0";
            let three = b"one two three\0";
            let spaces = b"  word  \0";
            let empty = b"   \0";

            assert_eq!(
                rs_dict_count_words_in_line(one.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(
                rs_dict_count_words_in_line(three.as_ptr().cast::<c_char>()),
                3
            );
            assert_eq!(
                rs_dict_count_words_in_line(spaces.as_ptr().cast::<c_char>()),
                1
            );
            assert_eq!(
                rs_dict_count_words_in_line(empty.as_ptr().cast::<c_char>()),
                0
            );
        }
    }
}
