//! Help system utilities for Neovim
//!
//! This module provides Rust implementations of help system functions from
//! `src/nvim/help.c`, including search heuristics, tag comparison, and
//! language detection.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

/// Check if a byte is an ASCII alphanumeric character (0-9, a-z, A-Z).
#[inline]
fn ascii_isalnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// Calculate a heuristic score for how well a matched string matches a help query.
///
/// The scoring considers:
/// - Langstroth's strnicmp algorithm
/// - More langnum letters is langbetter
/// - Match towards the start is better
/// - Match starting with "+" is worse (feature instead of command)
///
/// # Safety
/// The `matched_string` pointer must be valid and point to a null-terminated C string.
///
/// # Arguments
/// * `matched_string` - The matched help tag string
/// * `offset` - Offset into the string where the match occurred
/// * `wrong_case` - True if matching was case-insensitive
///
/// # Returns
/// A heuristic score (lower is better)
#[no_mangle]
pub unsafe extern "C" fn rs_help_heuristic(
    matched_string: *const c_char,
    offset: c_int,
    wrong_case: bool,
) -> c_int {
    if matched_string.is_null() {
        return c_int::MAX;
    }

    let cstr = unsafe { CStr::from_ptr(matched_string) };
    let bytes = cstr.to_bytes();

    // Count alphanumeric characters
    let num_letters = bytes.iter().filter(|&&c| ascii_isalnum(c)).count() as c_int;

    let mut offset_score = offset;

    // If the match starts in the middle of a word, add 10000 to put it
    // somewhere in the last half.
    // If the match is more than 2 chars from the start, multiply by 200 to
    // put it after matches at the start.
    if offset > 0 {
        let offset_usize = offset as usize;
        if offset_usize < bytes.len()
            && offset_usize > 0
            && ascii_isalnum(bytes[offset_usize])
            && ascii_isalnum(bytes[offset_usize - 1])
        {
            offset_score += 10000;
        } else if offset > 2 {
            offset_score *= 200;
        }
    }

    // If there only is a match while ignoring case, add 5000.
    if wrong_case {
        offset_score += 5000;
    }

    // Features are less interesting than the subjects themselves, but "+"
    // alone is not a feature.
    if !bytes.is_empty() && bytes[0] == b'+' && bytes.len() > 1 {
        offset_score += 100;
    }

    // Multiply the number of letters by 100 to give it a much bigger
    // weighting than the number of characters.
    100 * num_letters + (bytes.len() as c_int) + offset_score
}

/// Parse `@xx` language suffix from a help argument.
///
/// If the argument ends with `@xx` where both characters are ASCII alphabetic,
/// sets a NUL byte at the `@` position and returns a pointer to the two-letter
/// language code. Otherwise returns null.
///
/// # Safety
/// `arg` must be a valid, mutable, NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_check_help_lang(arg: *mut c_char) -> *mut c_char {
    if arg.is_null() {
        return std::ptr::null_mut();
    }

    let bytes = unsafe { CStr::from_ptr(arg) }.to_bytes();
    let len = bytes.len();

    if len >= 3 {
        let at_pos = len - 3;
        if bytes[at_pos] == b'@'
            && bytes[at_pos + 1].is_ascii_alphabetic()
            && bytes[at_pos + 2].is_ascii_alphabetic()
        {
            // Set NUL at the '@' position to truncate the string
            unsafe { *arg.add(at_pos) = 0 };
            // Return pointer to the two-letter language code
            return unsafe { arg.add(at_pos + 1) };
        }
    }

    std::ptr::null_mut()
}

/// Compare function for qsort() used by find_help_tags().
///
/// Each match string has a heuristic number stored after the tag name's NUL byte.
/// We compare by that heuristic number first, then by the tag string as a tie-breaker.
///
/// # Safety
/// `s1` and `s2` must point to valid `*const c_char` pointers (i.e., `char **`),
/// and each pointed-to string must be NUL-terminated with a second NUL-terminated
/// string immediately following.
#[no_mangle]
pub unsafe extern "C" fn rs_help_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    let p1_str = unsafe { *(s1 as *const *const c_char) };
    let p2_str = unsafe { *(s2 as *const *const c_char) };

    // Find the heuristic number stored after the tag name's NUL byte.
    let p1_len = unsafe { CStr::from_ptr(p1_str) }.to_bytes().len();
    let p2_len = unsafe { CStr::from_ptr(p2_str) }.to_bytes().len();

    let p1_heur = unsafe { p1_str.add(p1_len + 1) };
    let p2_heur = unsafe { p2_str.add(p2_len + 1) };

    // Compare by heuristic number first.
    let cmp = unsafe { libc::strcmp(p1_heur, p2_heur) };
    if cmp != 0 {
        return cmp;
    }

    // Compare by tag strings as tie-breaker.
    unsafe { libc::strcmp(p1_str, p2_str) }
}

// Constants verified against C headers
const IOSIZE: usize = 1025; // globals.h: (1024 + 1)
const MAXCOL: c_int = 0x7fffffff; // pos_defs.h
const TAG_HELP: c_int = 1; // tag.h
const TAG_NAMES: c_int = 2; // tag.h
const TAG_REGEXP: c_int = 4; // tag.h
const TAG_VERBOSE: c_int = 32; // tag.h
const TAG_KEEP_LANG: c_int = 128; // tag.h
const TAG_NO_TAGFUNC: c_int = 256; // tag.h
const TAG_MANY: c_int = 300; // tag.h
const OK: c_int = 1; // vim_defs.h

/// Exception table: specific tags that have a specific replacement or
/// won't go through the generic rules.
/// Copied verbatim from help.c.
const EXCEPT_TBL: &[(&[u8], &[u8])] = &[
    (b"*", b"star"),
    (b"g*", b"gstar"),
    (b"[*", b"[star"),
    (b"]*", b"]star"),
    (b":*", b":star"),
    (b"/*", b"/star"),
    (b"/\\*", b"/\\\\star"),
    (b"\"*", b"quotestar"),
    (b"**", b"starstar"),
    (b"cpo-*", b"cpo-star"),
    (b"/\\(\\)", b"/\\\\(\\\\)"),
    (b"/\\%(\\)", b"/\\\\%(\\\\)"),
    (b"?", b"?"),
    (b"??", b"??"),
    (b":?", b":?"),
    (b"?<CR>", b"?<CR>"),
    (b"g?", b"g?"),
    (b"g?g?", b"g?g?"),
    (b"g??", b"g??"),
    (b"-?", b"-?"),
    (b"q?", b"q?"),
    (b"v_g?", b"v_g?"),
    (b"/\\?", b"/\\\\?"),
    (b"/\\z(\\)", b"/\\\\z(\\\\)"),
    (b"\\=", b"\\\\="),
    (b":s\\=", b":s\\\\="),
    (b"[count]", b"\\[count]"),
    (b"[quotex]", b"\\[quotex]"),
    (b"[range]", b"\\[range]"),
    (b":[range]", b":\\[range]"),
    (b"[pattern]", b"\\[pattern]"),
    (b"\\|", b"\\\\bar"),
    (b"\\%$", b"/\\\\%\\$"),
    (b"s/\\~", b"s/\\\\\\~"),
    (b"s/\\U", b"s/\\\\U"),
    (b"s/\\L", b"s/\\\\L"),
    (b"s/\\1", b"s/\\\\1"),
    (b"s/\\2", b"s/\\\\2"),
    (b"s/\\3", b"s/\\\\3"),
    (b"s/\\9", b"s/\\\\9"),
];

/// Expression table entries for "expr-" prefix matching.
const EXPR_TABLE: &[&[u8]] = &[
    b"!=?", b"!~?", b"<=?", b"<?", b"==?", b"=~?", b">=?", b">?", b"is?", b"isnot?",
];

/// Opaque handle to exarg_T.
pub type ExargHandle = *mut c_void;

extern "C" {
    fn nvim_get_iobuff() -> *mut c_char;
    fn find_tags(
        pat: *const c_char,
        num_matches: *mut c_int,
        matchesp: *mut *mut *mut c_char,
        flags: c_int,
        mincount: c_int,
        buf_ffname: *const c_char,
    ) -> c_int;
    fn xfree(ptr: *mut c_void);
    fn nvim_help_get_p_hlg() -> *const c_char;
    fn do_cmdline_cmd(cmd: *const c_char);
    fn set_buflisted(listed: bool);
    fn nvim_help_set_curbuf_b_help(val: bool);
    fn nvim_help_get_curbuf_b_p_isk() -> *const c_char;
    fn nvim_help_set_buftype_help();
    fn nvim_help_set_isk_help(p: *const c_char);
    fn nvim_help_set_foldmethod_manual();
    fn nvim_help_set_buf_fields();
    fn nvim_help_set_win_help_options();

    // ex_help / ex_helpclose accessors
    fn nvim_help_eap_get_arg(eap: ExargHandle) -> *mut c_char;
    fn nvim_help_eap_set_nextcmd(eap: ExargHandle, cmd: *mut c_char);
    fn nvim_help_eap_get_forceit(eap: ExargHandle) -> c_int;
    fn nvim_help_eap_get_skip(eap: ExargHandle) -> c_int;

    fn nvim_help_curbuf_is_help() -> bool;
    fn nvim_help_curwin_bt_help() -> bool;
    fn nvim_help_get_cmdmod_tab() -> c_int;
    fn nvim_help_get_cmdmod_split() -> c_int;
    fn nvim_help_get_cmdmod_flags() -> c_int;
    fn nvim_help_get_columns() -> c_int;
    fn nvim_help_get_curwin_width() -> c_int;
    fn nvim_help_get_curwin_height() -> c_int;
    fn nvim_help_get_p_sb() -> c_int;
    fn nvim_help_get_p_hh() -> i64;
    fn nvim_help_get_p_hf() -> *const c_char;

    fn nvim_help_get_KeyTyped() -> bool;
    fn nvim_help_set_KeyTyped(val: bool);
    fn nvim_help_set_restart_edit(val: c_int);

    fn nvim_help_get_curbuf_fnum() -> c_int;
    fn nvim_help_get_curwin_alt_fnum() -> c_int;
    fn nvim_help_set_curwin_alt_fnum(fnum: c_int);

    fn nvim_help_find_help_win_in_tab() -> *mut c_void;
    fn nvim_help_win_nwindows(wp: *mut c_void) -> c_int;
    fn nvim_help_do_ecmd_help() -> c_int;

    fn emsg(s: *const c_char);
    fn semsg(fmt: *const c_char, ...);
    fn smsg(hl_id: c_int, s: *const c_char, ...);

    fn os_fopen(path: *const c_char, flags: *const c_char) -> *mut c_void;
    fn win_enter(wp: *mut c_void, undo_sync: bool);
    fn win_close(win: *mut c_void, free_buf: bool, force: bool) -> c_int;
    fn win_split(size: c_int, flags: c_int) -> c_int;
    fn win_setheight(height: c_int);
    fn do_tag(tag: *const c_char, tag_type: c_int, count: c_int, forceit: c_int, verbose: bool);
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn FreeWild(count: c_int, files: *mut *mut c_char);
    fn buflist_findnr(nr: c_int) -> *mut c_void;
    fn wipe_buffer(buf: *mut c_void, aucmd: bool);
    fn nvim_help_buf_nwindows(buf: *mut c_void) -> c_int;
}

/// Helper: write a byte slice into a C buffer at a given offset.
/// Returns the new offset (position after the last written byte).
#[inline]
unsafe fn buf_write(buf: *mut c_char, offset: usize, src: &[u8]) -> usize {
    for (i, &b) in src.iter().enumerate() {
        unsafe { *buf.add(offset + i) = b as c_char };
    }
    offset + src.len()
}

/// Find all help tags matching `arg`, sort them and return in `matches`.
///
/// The matches will be sorted with a "best" match algorithm.
/// When `keep_lang` is true, try keeping the language of the current buffer.
///
/// # Safety
/// All pointer arguments must be valid. `arg` must be NUL-terminated.
/// `num_matches` and `matches` must be valid pointers to write results into.
#[no_mangle]
pub unsafe extern "C" fn rs_find_help_tags(
    arg: *const c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
    keep_lang: bool,
) -> c_int {
    let iobuff = unsafe { nvim_get_iobuff() };
    let arg_bytes = unsafe { CStr::from_ptr(arg) }.to_bytes();

    // d tracks current write position in IObuff
    let mut d: usize = 0;
    unsafe { *iobuff = 0 }; // d[0] = NUL

    let mut matched = false;

    if arg_bytes.len() >= 5 && arg_bytes[..5].eq_ignore_ascii_case(b"expr-") {
        // When the string starting with "expr-" and containing '?' and matches
        // the table, it is taken literally (but ~ is escaped).
        for entry in EXPR_TABLE.iter().rev() {
            if &arg_bytes[5..] == *entry {
                let mut si = 0usize;
                while si < arg_bytes.len() {
                    if arg_bytes[si] == b'~' {
                        unsafe { *iobuff.add(d) = b'\\' as c_char };
                        d += 1;
                    }
                    unsafe { *iobuff.add(d) = arg_bytes[si] as c_char };
                    d += 1;
                    si += 1;
                }
                unsafe { *iobuff.add(d) = 0 };
                matched = true;
                break;
            }
        }
    } else {
        // Recognize a few exceptions to the rule.
        for &(from, to) in EXCEPT_TBL {
            if arg_bytes == from {
                d = unsafe { buf_write(iobuff, 0, to) };
                unsafe { *iobuff.add(d) = 0 };
                matched = true;
                break;
            }
        }
    }

    if !matched {
        // no match in table
        d = 0;

        // Replace "\S" with "/\\S", etc.
        if arg_bytes.first() == Some(&b'\\')
            && ((arg_bytes.len() == 2 && arg_bytes[1] != 0)
                || (arg_bytes.len() >= 3
                    && (arg_bytes[1] == b'%'
                        || arg_bytes[1] == b'_'
                        || arg_bytes[1] == b'z'
                        || arg_bytes[1] == b'@')
                    && arg_bytes[2] != 0))
        {
            // vim_snprintf(d, IOSIZE, "/\\\\%s", arg + 1);
            let prefix = b"/\\\\";
            d = unsafe { buf_write(iobuff, 0, prefix) };
            d = unsafe { buf_write(iobuff, d, &arg_bytes[1..]) };
            unsafe { *iobuff.add(d) = 0 };

            // Check for "/\\_$", should be "/\\_\$"
            if d >= 5 {
                let d3 = unsafe { *iobuff.add(3) } as u8;
                let d4 = unsafe { *iobuff.add(4) } as u8;
                if d3 == b'_' && d4 == b'$' {
                    // Replace "$" at position 4 with "\\$"
                    unsafe { *iobuff.add(4) = b'\\' as c_char };
                    unsafe { *iobuff.add(5) = b'$' as c_char };
                    unsafe { *iobuff.add(6) = 0 };
                    // d is not used after this branch, but kept for clarity
                    let _ = 6;
                }
            }
        } else {
            // Replace:
            // "[:...:]" with "\[:...:]"
            // "[++...]" with "\[++...]"
            // "\{" with "\\{"
            if (arg_bytes.first() == Some(&b'[')
                && (arg_bytes.get(1) == Some(&b':')
                    || (arg_bytes.get(1) == Some(&b'+') && arg_bytes.get(2) == Some(&b'+'))))
                || (arg_bytes.first() == Some(&b'\\') && arg_bytes.get(1) == Some(&b'{'))
            {
                unsafe { *iobuff.add(d) = b'\\' as c_char };
                d += 1;
            }

            // If tag starts with "('", skip the "(".
            let mut s_off: usize = 0;
            let arg_start = if arg_bytes.first() == Some(&b'(') && arg_bytes.get(1) == Some(&b'\'')
            {
                s_off = 1;
                1
            } else {
                0
            };
            let _ = arg_start; // suppress warning; s_off tracks the effective arg start

            let mut si = s_off;
            while si < arg_bytes.len() {
                let s = arg_bytes[si];

                // getting too long!?
                if d > IOSIZE - 10 {
                    break;
                }

                match s {
                    b'|' => {
                        d = unsafe { buf_write(iobuff, d, b"bar") };
                        si += 1;
                        continue;
                    }
                    b'"' => {
                        d = unsafe { buf_write(iobuff, d, b"quote") };
                        si += 1;
                        continue;
                    }
                    b'*' => {
                        unsafe { *iobuff.add(d) = b'.' as c_char };
                        d += 1;
                        // falls through to write *s
                    }
                    b'?' => {
                        unsafe { *iobuff.add(d) = b'.' as c_char };
                        d += 1;
                        si += 1;
                        continue;
                    }
                    b'$' | b'.' | b'~' => {
                        unsafe { *iobuff.add(d) = b'\\' as c_char };
                        d += 1;
                    }
                    _ => {}
                }

                // Replace "^x" by "CTRL-X". Don't do this for "^_".
                // Insert '-' before and after "CTRL-X" when applicable.
                if s < b' '
                    || (s == b'^'
                        && si + 1 < arg_bytes.len()
                        && (arg_bytes[si + 1].is_ascii_alphabetic()
                            || matches!(
                                arg_bytes[si + 1],
                                b'?' | b'@' | b'[' | b'\\' | b']' | b'^'
                            )))
                {
                    if d > 0 {
                        let prev = unsafe { *iobuff.add(d - 1) } as u8;
                        if prev != b'_' && prev != b'\\' {
                            unsafe { *iobuff.add(d) = b'_' as c_char };
                            d += 1;
                        }
                    }
                    d = unsafe { buf_write(iobuff, d, b"CTRL-") };

                    if s < b' ' {
                        let ctrl_char = s.wrapping_add(b'@');
                        unsafe { *iobuff.add(d) = ctrl_char as c_char };
                        d += 1;
                        if ctrl_char == b'\\' {
                            unsafe { *iobuff.add(d) = b'\\' as c_char };
                            d += 1;
                        }
                    } else {
                        si += 1;
                        unsafe { *iobuff.add(d) = arg_bytes[si] as c_char };
                        d += 1;
                    }
                    if si + 1 < arg_bytes.len() && arg_bytes[si + 1] != b'_' {
                        unsafe { *iobuff.add(d) = b'_' as c_char };
                        d += 1;
                    }
                    si += 1;
                    continue;
                } else if s == b'^' {
                    // "^" or "CTRL-^" or "^_"
                    unsafe { *iobuff.add(d) = b'\\' as c_char };
                    d += 1;
                } else if s == b'\\'
                    && si + 1 < arg_bytes.len()
                    && arg_bytes[si + 1] != b'\\'
                    && arg_bytes.first() == Some(&b'/')
                    && si == s_off + 1
                {
                    // Insert a backslash before a backslash after a slash
                    unsafe { *iobuff.add(d) = b'\\' as c_char };
                    d += 1;
                }

                // "CTRL-\_" -> "CTRL-\\_"
                if si + 6 < arg_bytes.len() {
                    let chunk = &arg_bytes[si..si + 7];
                    if chunk.eq_ignore_ascii_case(b"CTRL-\\_") {
                        d = unsafe { buf_write(iobuff, d, b"CTRL-\\\\") };
                        si += 6;
                        // The final char (after "CTRL-\\") is written below as *d++ = *s
                        unsafe { *iobuff.add(d) = arg_bytes[si] as c_char };
                        d += 1;

                        // Check for break conditions
                        if si + 1 < arg_bytes.len()
                            && (arg_bytes[si + 1] == b'{' || arg_bytes[si + 1] == b'[')
                            && arg_bytes[si] == b'('
                        {
                            break;
                        }
                        if arg_bytes[si] == b'\'' && si > s_off && arg_bytes[s_off] == b'\'' {
                            break;
                        }
                        if arg_bytes[si] == b'}' && si > s_off && arg_bytes[s_off] == b'{' {
                            break;
                        }
                        si += 1;
                        continue;
                    }
                }

                unsafe { *iobuff.add(d) = s as c_char };
                d += 1;

                // If tag contains "({" or "([", tag terminates at the "(".
                if s == b'('
                    && si + 1 < arg_bytes.len()
                    && (arg_bytes[si + 1] == b'{' || arg_bytes[si + 1] == b'[')
                {
                    break;
                }

                // If tag starts with ', toss everything after a second '.
                if s == b'\'' && si > s_off && arg_bytes[s_off] == b'\'' {
                    break;
                }
                // Also '{' and '}'.
                if s == b'}' && si > s_off && arg_bytes[s_off] == b'{' {
                    break;
                }

                si += 1;
            }
            unsafe { *iobuff.add(d) = 0 };

            // Handle backtick stripping
            if unsafe { *iobuff } as u8 == b'`' {
                if d > 2 && unsafe { *iobuff.add(d - 1) } as u8 == b'`' {
                    // remove the backticks from `command`
                    let len = d; // includes the NUL we just wrote? No, d is offset before NUL
                    unsafe {
                        std::ptr::copy(iobuff.add(1), iobuff, len);
                        *iobuff.add(d - 2) = 0;
                    }
                } else if d > 3
                    && unsafe { *iobuff.add(d - 2) } as u8 == b'`'
                    && unsafe { *iobuff.add(d - 1) } as u8 == b','
                {
                    // remove the backticks and comma from `command`,
                    unsafe {
                        std::ptr::copy(iobuff.add(1), iobuff, d);
                        *iobuff.add(d - 3) = 0;
                    }
                } else if d > 4
                    && unsafe { *iobuff.add(d - 3) } as u8 == b'`'
                    && unsafe { *iobuff.add(d - 2) } as u8 == b'\\'
                    && unsafe { *iobuff.add(d - 1) } as u8 == b'.'
                {
                    // remove the backticks and dot from `command`\.
                    unsafe {
                        std::ptr::copy(iobuff.add(1), iobuff, d);
                        *iobuff.add(d - 4) = 0;
                    }
                }
            }
        }
    }

    unsafe { *matches = std::ptr::null_mut() };
    unsafe { *num_matches = 0 };
    let mut flags = TAG_HELP | TAG_REGEXP | TAG_NAMES | TAG_VERBOSE | TAG_NO_TAGFUNC;
    if keep_lang {
        flags |= TAG_KEEP_LANG;
    }
    if unsafe {
        find_tags(
            iobuff,
            num_matches,
            matches,
            flags,
            MAXCOL,
            std::ptr::null(),
        )
    } == OK
        && unsafe { *num_matches } > 0
    {
        // Sort the matches found on the heuristic number.
        unsafe {
            libc::qsort(
                *matches as *mut c_void,
                *num_matches as usize,
                std::mem::size_of::<*mut c_char>(),
                Some(rs_help_compare),
            );
        }
        // Delete more than TAG_MANY to reduce the size of the listing.
        while unsafe { *num_matches } > TAG_MANY {
            unsafe {
                *num_matches -= 1;
                xfree(*(*matches).add(*num_matches as usize) as *mut c_void);
            }
        }
    }
    OK
}

/// Cleanup matches for help tags: strip language suffixes.
///
/// Remove "@ab" if the top of 'helplang' is "ab" and the language of the first
/// tag matches it. Otherwise remove "@en" if "en" is the only language.
///
/// # Safety
/// `file` must point to an array of `num_file` valid, mutable, NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_help_tags(num_file: c_int, file: *mut *mut c_char) {
    let mut buf = [0u8; 4];
    let mut buf_len: usize = 0;

    let p_hlg = unsafe { nvim_help_get_p_hlg() };
    if !p_hlg.is_null() {
        let hlg_bytes = unsafe { CStr::from_ptr(p_hlg) }.to_bytes();
        if !hlg_bytes.is_empty() && (hlg_bytes[0] != b'e' || hlg_bytes.get(1) != Some(&b'n')) {
            buf[0] = b'@';
            buf[1] = hlg_bytes[0];
            buf[2] = hlg_bytes[1];
            buf_len = 3;
        }
    }

    let n = num_file as usize;

    for i in 0..n {
        let s = unsafe { *file.add(i) };
        let slen = unsafe { CStr::from_ptr(s) }.to_bytes().len();
        if slen <= 3 {
            continue;
        }
        let suffix_start = slen - 3;

        // Check for "@en" suffix
        let suffix = unsafe { std::slice::from_raw_parts(s.add(suffix_start) as *const u8, 3) };
        if suffix == b"@en" {
            // Search all items for a match up to the "@en".
            let mut found_other = false;
            for j in 0..n {
                if j == i {
                    continue;
                }
                let other = unsafe { *file.add(j) };
                let other_len = unsafe { CStr::from_ptr(other) }.to_bytes().len();
                if other_len == slen && unsafe { libc::strncmp(s, other, suffix_start + 1) } == 0 {
                    found_other = true;
                    break;
                }
            }
            if !found_other {
                // Item only exists with @en, remove suffix
                unsafe { *s.add(suffix_start) = 0 };
            }
        }
    }

    if buf_len > 0 {
        for i in 0..n {
            let s = unsafe { *file.add(i) };
            let slen = unsafe { CStr::from_ptr(s) }.to_bytes().len();
            if slen <= 3 {
                continue;
            }
            let suffix_start = slen - 3;
            let suffix = unsafe { std::slice::from_raw_parts(s.add(suffix_start) as *const u8, 3) };
            if suffix == &buf[..3] {
                unsafe { *s.add(suffix_start) = 0 };
            }
        }
    }
}

// Window split constants (verified against src/nvim/window.h)
const WSP_HELP: c_int = 0x20;
const WSP_BOT: c_int = 0x10;
const WSP_TOP: c_int = 0x08;

// Command modifier flags (verified against src/nvim/ex_cmds_defs.h)
const CMOD_KEEPALT: c_int = 0x0100;

// Tag action (verified against src/nvim/tag.h)
const DT_HELP: c_int = 8;

// Return values (verified against src/nvim/vim_defs.h)
const FAIL: c_int = 0;

/// `:helpclose` — close one help window in the current tab.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_helpclose(eap: ExargHandle) {
    let forceit = unsafe { nvim_help_eap_get_forceit(eap) } != 0;

    // Iterate windows in the current tab looking for a help window
    // We use the combined accessor that finds the help window for us
    // since FOR_ALL_WINDOWS_IN_TAB + bt_help check is encapsulated in C.
    // Actually we need to replicate the loop here. Let's use the window
    // iteration pattern from the window crate.

    // Use the C accessor that finds the first help window
    let help_win = unsafe { nvim_help_find_help_win_in_tab() };
    if !help_win.is_null() {
        unsafe { win_close(help_win, false, forceit) };
    }
}

/// `:help` — open a read-only window on a help file.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_help(eap: ExargHandle) {
    let eap_is_null = eap.is_null();

    let mut arg: *mut c_char;
    let mut empty_fnum: c_int = 0;
    let mut alt_fnum: c_int = 0;
    let old_key_typed = unsafe { nvim_help_get_KeyTyped() };

    if !eap_is_null {
        // A ":help" command ends at the first LF, or at a '|' that is
        // followed by some text. Set nextcmd to the following command.
        arg = unsafe { nvim_help_eap_get_arg(eap) };
        let mut p = arg;
        while unsafe { *p } != 0 {
            let ch = unsafe { *p } as u8;
            let next = unsafe { *p.add(1) } as u8;
            if ch == b'\n' || ch == b'\r' || (ch == b'|' && next != 0 && next != b'|') {
                unsafe { *p = 0 };
                let next_ptr = unsafe { p.add(1) };
                unsafe { nvim_help_eap_set_nextcmd(eap, next_ptr) };
                break;
            }
            p = unsafe { p.add(1) };
        }
        arg = unsafe { nvim_help_eap_get_arg(eap) };

        let forceit = unsafe { nvim_help_eap_get_forceit(eap) } != 0;
        if forceit && unsafe { *arg } == 0 && !unsafe { nvim_help_curbuf_is_help() } {
            unsafe { emsg(c"E478: Don't panic!".as_ptr()) };
            return;
        }

        if unsafe { nvim_help_eap_get_skip(eap) } != 0 {
            return;
        }
    } else {
        arg = c"".as_ptr() as *mut c_char;
    }

    // remove trailing blanks
    let arg_len = unsafe { CStr::from_ptr(arg) }.to_bytes().len();
    if arg_len > 0 {
        let mut p = unsafe { arg.add(arg_len - 1) };
        while p > arg {
            let ch = unsafe { *p } as u8;
            let prev = unsafe { *p.sub(1) } as u8;
            if (ch == b' ' || ch == b'\t') && prev != b'\\' {
                unsafe { *p = 0 };
                p = unsafe { p.sub(1) };
            } else {
                break;
            }
        }
    }

    // Check for a specified language
    let lang = rs_check_help_lang(arg);

    // When no argument given go to the index.
    if unsafe { *arg } == 0 {
        arg = c"help.txt".as_ptr() as *mut c_char;
    }

    // Check if there is a match for the argument.
    let mut num_matches: c_int = 0;
    let mut matches: *mut *mut c_char = std::ptr::null_mut();
    let n = rs_find_help_tags(
        arg,
        &mut num_matches,
        &mut matches,
        !eap_is_null && unsafe { nvim_help_eap_get_forceit(eap) } != 0,
    );

    let mut i: c_int = 0;
    if n != FAIL && !lang.is_null() {
        // Find first item with the requested language.
        while i < num_matches {
            let m = unsafe { *matches.add(i as usize) };
            let m_len = unsafe { CStr::from_ptr(m) }.to_bytes().len();
            if m_len > 3 {
                let at_pos = m_len - 3;
                if unsafe { *m.add(at_pos) } as u8 == b'@' {
                    // Case-insensitive compare of the 2-letter language code
                    let l1 = (unsafe { *m.add(at_pos + 1) } as u8).to_ascii_lowercase();
                    let l2 = (unsafe { *m.add(at_pos + 2) } as u8).to_ascii_lowercase();
                    let r1 = (unsafe { *lang } as u8).to_ascii_lowercase();
                    let r2 = (unsafe { *lang.add(1) } as u8).to_ascii_lowercase();
                    if l1 == r1 && l2 == r2 {
                        break;
                    }
                }
            }
            i += 1;
        }
    }

    if i >= num_matches || n == FAIL {
        if !lang.is_null() {
            unsafe { semsg(c"E661: Sorry, no '%s' help for %s".as_ptr(), lang, arg) };
        } else {
            unsafe { semsg(c"E149: Sorry, no help for %s".as_ptr(), arg) };
        }
        if n != FAIL {
            unsafe { FreeWild(num_matches, matches) };
        }
        return;
    }

    // The first match (in the requested language) is the best match.
    let tag = unsafe { xstrdup(*matches.add(i as usize)) };
    unsafe { FreeWild(num_matches, matches) };

    // Use a closure-like block so we can break out (simulating goto erret).
    let mut do_help = || -> bool {
        // Re-use an existing help window or open a new one.
        // Always open a new one for ":tab help".
        if !unsafe { nvim_help_curwin_bt_help() } || unsafe { nvim_help_get_cmdmod_tab() } != 0 {
            let wp = if unsafe { nvim_help_get_cmdmod_tab() } != 0 {
                std::ptr::null_mut()
            } else {
                unsafe { nvim_help_find_help_win_in_tab() }
            };

            if !wp.is_null() && unsafe { nvim_help_win_nwindows(wp) } > 0 {
                unsafe { win_enter(wp, true) };
            } else {
                // There is no help window yet.
                // Try to open the file specified by the "helpfile" option.
                let p_hf = unsafe { nvim_help_get_p_hf() };
                let helpfd = unsafe { os_fopen(p_hf, c"rb".as_ptr()) };
                if helpfd.is_null() {
                    unsafe { smsg(0, c"Sorry, help file \"%s\" not found".as_ptr(), p_hf) };
                    return false; // goto erret
                }
                unsafe { libc::fclose(helpfd as *mut libc::FILE) };

                // Split off help window
                let mut split_flags = WSP_HELP;
                if unsafe { nvim_help_get_cmdmod_split() } == 0
                    && unsafe { nvim_help_get_curwin_width() } != unsafe { nvim_help_get_columns() }
                    && unsafe { nvim_help_get_curwin_width() } < 80
                {
                    if unsafe { nvim_help_get_p_sb() } != 0 {
                        split_flags |= WSP_BOT;
                    } else {
                        split_flags |= WSP_TOP;
                    }
                }
                if unsafe { win_split(0, split_flags) } == FAIL {
                    return false; // goto erret
                }

                let p_hh = unsafe { nvim_help_get_p_hh() };
                if (unsafe { nvim_help_get_curwin_height() } as i64) < p_hh {
                    unsafe { win_setheight(p_hh as c_int) };
                }

                // Open help file
                alt_fnum = unsafe { nvim_help_get_curbuf_fnum() };
                unsafe { nvim_help_do_ecmd_help() };

                if unsafe { nvim_help_get_cmdmod_flags() } & CMOD_KEEPALT == 0 {
                    unsafe { nvim_help_set_curwin_alt_fnum(alt_fnum) };
                }
                empty_fnum = unsafe { nvim_help_get_curbuf_fnum() };
            }
        }
        true
    };

    let success = do_help();

    if success {
        unsafe { nvim_help_set_restart_edit(0) };
        unsafe { nvim_help_set_KeyTyped(old_key_typed) };
        unsafe { do_tag(tag, DT_HELP, 1, 0, true) };

        // Delete the empty buffer if we're not using it.
        if empty_fnum != 0 && unsafe { nvim_help_get_curbuf_fnum() } != empty_fnum {
            let buf = unsafe { buflist_findnr(empty_fnum) };
            if !buf.is_null() && unsafe { nvim_help_buf_nwindows(buf) } == 0 {
                unsafe { wipe_buffer(buf, true) };
            }
        }

        // keep the previous alternate file
        if alt_fnum != 0
            && unsafe { nvim_help_get_curwin_alt_fnum() } == empty_fnum
            && unsafe { nvim_help_get_cmdmod_flags() } & CMOD_KEEPALT == 0
        {
            unsafe { nvim_help_set_curwin_alt_fnum(alt_fnum) };
        }
    }

    // erret: always free tag
    unsafe { xfree(tag as *mut c_void) };
}

/// Called when starting to edit a buffer for a help file.
/// Sets buffer type, iskeyword, foldmethod, and various buffer/window options.
#[no_mangle]
pub unsafe extern "C" fn rs_prepare_help_buffer() {
    unsafe { nvim_help_set_curbuf_b_help(true) };
    unsafe { nvim_help_set_buftype_help() };

    // Accept all ASCII chars for keywords, except ' ', '*', '"', '|', and
    // latin1 word characters (for translated help files).
    // Only set it when needed, buf_init_chartab() is some work.
    let isk = c"!-~,^*,^|,^\",192-255";
    let current_isk = unsafe { nvim_help_get_curbuf_b_p_isk() };
    if !current_isk.is_null() && unsafe { libc::strcmp(current_isk, isk.as_ptr()) } != 0 {
        unsafe { nvim_help_set_isk_help(isk.as_ptr()) };
    }

    unsafe { nvim_help_set_foldmethod_manual() };
    unsafe { nvim_help_set_buf_fields() };
    unsafe { nvim_help_set_win_help_options() };
    unsafe { set_buflisted(false) };
}

/// `:exusage` — open help for ex command index.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_exusage(_eap: ExargHandle) {
    unsafe { do_cmdline_cmd(c"help ex-cmd-index".as_ptr()) };
}

/// `:viusage` — open help for normal mode command index.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_viusage(_eap: ExargHandle) {
    unsafe { do_cmdline_cmd(c"help normal-index".as_ptr()) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_str(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_help_heuristic_basic() {
        unsafe {
            // Simple match at start
            let score1 = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);
            // 3 letters * 100 + 3 length + 0 offset = 303
            assert_eq!(score1, 303);
        }
    }

    #[test]
    fn test_help_heuristic_wrong_case() {
        unsafe {
            let score_correct = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);
            let score_wrong = rs_help_heuristic(test_str("abc").as_ptr(), 0, true);
            // Wrong case adds 5000
            assert_eq!(score_wrong, score_correct + 5000);
        }
    }

    #[test]
    fn test_help_heuristic_offset() {
        unsafe {
            // Match at start
            let score_start = rs_help_heuristic(test_str("abc").as_ptr(), 0, false);

            // Match at offset 3 (more than 2, multiplies by 200)
            let score_offset3 = rs_help_heuristic(test_str("abc").as_ptr(), 3, false);
            // 303 + 3*200 = 903
            assert_eq!(score_offset3, 303 + 3 * 200);

            // Verify start is better (lower score)
            assert!(score_start < score_offset3);
        }
    }

    #[test]
    fn test_help_heuristic_mid_word() {
        unsafe {
            // Match in middle of word (abcdef, offset 3, both d and c are alnum)
            let score = rs_help_heuristic(test_str("abcdef").as_ptr(), 3, false);
            // 6 letters * 100 + 6 length + 3 offset + 10000 = 10609
            assert_eq!(score, 10609);
        }
    }

    #[test]
    fn test_help_heuristic_feature() {
        unsafe {
            // Feature starting with "+" adds 100 penalty
            // Use strings with same number of alnum chars to isolate the feature penalty
            // "+abc" has 3 alnum chars (a, b, c), "-abc" also has 3 alnum chars
            let score_feature = rs_help_heuristic(test_str("+abc").as_ptr(), 0, false);
            let score_normal = rs_help_heuristic(test_str("-abc").as_ptr(), 0, false);
            // Both have: 3 letters * 100 + 4 length = 304
            // Feature adds 100: 304 + 100 = 404
            assert_eq!(score_normal, 304);
            assert_eq!(score_feature, 404);
        }
    }

    #[test]
    fn test_help_heuristic_plus_alone() {
        unsafe {
            // "+" alone is not treated as feature
            let score_plus = rs_help_heuristic(test_str("+").as_ptr(), 0, false);
            // 0 letters * 100 + 1 length + 0 offset = 1
            assert_eq!(score_plus, 1);
        }
    }

    #[test]
    fn test_help_heuristic_null() {
        unsafe {
            let score = rs_help_heuristic(std::ptr::null(), 0, false);
            assert_eq!(score, c_int::MAX);
        }
    }

    #[test]
    fn test_help_heuristic_empty_string() {
        unsafe {
            let score = rs_help_heuristic(test_str("").as_ptr(), 0, false);
            // 0 letters * 100 + 0 length + 0 offset = 0
            assert_eq!(score, 0);
        }
    }

    #[test]
    fn test_help_heuristic_non_alpha() {
        unsafe {
            // String with no alphanumeric characters
            let score = rs_help_heuristic(test_str("---").as_ptr(), 0, false);
            // 0 letters * 100 + 3 length + 0 offset = 3
            assert_eq!(score, 3);
        }
    }

    #[test]
    fn test_check_help_lang_with_suffix() {
        unsafe {
            let s = CString::new("foo@en").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(!lang.is_null());
            // The language code should be "en"
            assert_eq!(*lang as u8, b'e');
            assert_eq!(*lang.add(1) as u8, b'n');
            // The original string should be truncated to "foo"
            let truncated = CStr::from_ptr(ptr);
            assert_eq!(truncated.to_bytes(), b"foo");
            // Clean up
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_too_short() {
        unsafe {
            let s = CString::new("ab").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(lang.is_null());
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_non_alpha() {
        unsafe {
            let s = CString::new("foo@1x").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(lang.is_null());
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_no_at() {
        unsafe {
            let s = CString::new("foobar").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(lang.is_null());
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_check_help_lang_null() {
        unsafe {
            let lang = rs_check_help_lang(std::ptr::null_mut());
            assert!(lang.is_null());
        }
    }

    #[test]
    fn test_check_help_lang_exactly_three() {
        unsafe {
            // "@en" is exactly 3 chars - the arg part before @ is empty
            let s = CString::new("@en").unwrap();
            let ptr = s.into_raw();
            let lang = rs_check_help_lang(ptr);
            assert!(!lang.is_null());
            assert_eq!(*lang as u8, b'e');
            assert_eq!(*lang.add(1) as u8, b'n');
            let truncated = CStr::from_ptr(ptr);
            assert_eq!(truncated.to_bytes(), b"");
            let _ = CString::from_raw(ptr);
        }
    }

    /// Helper to create a mock help tag match string with an embedded heuristic number.
    /// The format is: "tagname\0heuristic_number\0"
    fn make_help_match(tag: &str, heuristic: &str) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(tag.as_bytes());
        v.push(0); // NUL separator
        v.extend_from_slice(heuristic.as_bytes());
        v.push(0); // NUL terminator
        v
    }

    #[test]
    fn test_help_compare_different_heuristic() {
        unsafe {
            let m1 = make_help_match("tag_a", "0100");
            let m2 = make_help_match("tag_b", "0200");
            let p1 = m1.as_ptr() as *const c_char;
            let p2 = m2.as_ptr() as *const c_char;
            let result = rs_help_compare(
                &p1 as *const _ as *const c_void,
                &p2 as *const _ as *const c_void,
            );
            // "0100" < "0200" so result should be negative
            assert!(result < 0);

            // Reverse order
            let result2 = rs_help_compare(
                &p2 as *const _ as *const c_void,
                &p1 as *const _ as *const c_void,
            );
            assert!(result2 > 0);
        }
    }

    #[test]
    fn test_help_compare_same_heuristic_different_tag() {
        unsafe {
            let m1 = make_help_match("alpha", "0100");
            let m2 = make_help_match("beta", "0100");
            let p1 = m1.as_ptr() as *const c_char;
            let p2 = m2.as_ptr() as *const c_char;
            let result = rs_help_compare(
                &p1 as *const _ as *const c_void,
                &p2 as *const _ as *const c_void,
            );
            // Same heuristic, so compare by tag: "alpha" < "beta"
            assert!(result < 0);
        }
    }

    #[test]
    fn test_help_compare_identical() {
        unsafe {
            let m1 = make_help_match("same", "0100");
            let m2 = make_help_match("same", "0100");
            let p1 = m1.as_ptr() as *const c_char;
            let p2 = m2.as_ptr() as *const c_char;
            let result = rs_help_compare(
                &p1 as *const _ as *const c_void,
                &p2 as *const _ as *const c_void,
            );
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn test_ascii_isalnum() {
        // Digits
        for c in b'0'..=b'9' {
            assert!(ascii_isalnum(c));
        }
        // Lowercase
        for c in b'a'..=b'z' {
            assert!(ascii_isalnum(c));
        }
        // Uppercase
        for c in b'A'..=b'Z' {
            assert!(ascii_isalnum(c));
        }
        // Non-alphanumeric
        assert!(!ascii_isalnum(b' '));
        assert!(!ascii_isalnum(b'-'));
        assert!(!ascii_isalnum(b'_'));
        assert!(!ascii_isalnum(b'+'));
    }

    #[test]
    fn test_except_tbl_count() {
        // Must match the 40 entries in the C except_tbl (without NULL terminator)
        assert_eq!(EXCEPT_TBL.len(), 40);
    }

    #[test]
    fn test_except_tbl_entries() {
        // Verify a few key entries verbatim
        assert_eq!(EXCEPT_TBL[0], (&b"*"[..], &b"star"[..]));
        assert_eq!(EXCEPT_TBL[1], (&b"g*"[..], &b"gstar"[..]));
        assert_eq!(EXCEPT_TBL[7], (&b"\"*"[..], &b"quotestar"[..]));
        assert_eq!(EXCEPT_TBL[8], (&b"**"[..], &b"starstar"[..]));
        assert_eq!(EXCEPT_TBL[31], (&b"\\|"[..], &b"\\\\bar"[..]));
    }

    #[test]
    fn test_expr_table_count() {
        assert_eq!(EXPR_TABLE.len(), 10);
    }

    #[test]
    fn test_expr_table_entries() {
        assert_eq!(EXPR_TABLE[0], b"!=?");
        assert_eq!(EXPR_TABLE[9], b"isnot?");
    }

    #[test]
    fn test_constants() {
        assert_eq!(IOSIZE, 1025);
        assert_eq!(MAXCOL, 0x7fffffff);
        assert_eq!(TAG_HELP, 1);
        assert_eq!(TAG_NAMES, 2);
        assert_eq!(TAG_REGEXP, 4);
        assert_eq!(TAG_VERBOSE, 32);
        assert_eq!(TAG_KEEP_LANG, 128);
        assert_eq!(TAG_NO_TAGFUNC, 256);
        assert_eq!(TAG_MANY, 300);
    }
}
