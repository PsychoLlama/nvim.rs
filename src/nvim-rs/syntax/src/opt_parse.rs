//! Syntax option parsing.
//!
//! Migrated from `get_syn_options` and `get_id_list` in syntax.c.
//! Handles parsing of syntax highlight options like contained, oneline,
//! keepend, contains=, containedin=, nextgroup=, etc.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{
    SynPatHandle, HL_CONCEAL, HL_CONCEALENDS, HL_CONTAINED, HL_DISPLAY, HL_EXCLUDENL, HL_EXTEND,
    HL_FOLD, HL_KEEPEND, HL_ONELINE, HL_SKIPEMPTY, HL_SKIPNL, HL_SKIPWHITE, HL_SYNC_HERE,
    HL_SYNC_THERE, HL_TRANSP, NONE_IDX, SPTYPE_START, SYNID_ALLBUT, SYNID_CONTAINED, SYNID_TOP,
};

/// RE_MAGIC flag for vim_regcomp.
const RE_MAGIC: c_int = 1;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // String helpers
    fn nvim_syn_skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_syn_skiptowhite(p: *const c_char) -> *mut c_char;
    fn nvim_syn_ends_excmd(c: c_int) -> c_int;
    fn nvim_syn_ascii_iswhite_char(c: c_int) -> c_int;
    fn nvim_syn_toupper_asc(c: c_int) -> c_int;

    // Memory
    fn nvim_syn_xstrnsave(s: *const c_char, len: c_int) -> *mut c_char;
    fn nvim_syn_xfree(ptr: *mut c_void);
    fn nvim_syn_xmalloc(size: c_int) -> *mut c_void;
    fn nvim_syn_xmemcpyz(dst: *mut c_char, src: *const c_char, len: c_int);
    fn nvim_syn_strpbrk(s: *const c_char, chars: *const c_char) -> *mut c_char;

    // Error messages
    fn nvim_syn_emsg(msg: *const c_char);
    fn nvim_syn_semsg_1s(fmt: *const c_char, arg: *const c_char);

    // Character functions
    fn nvim_syn_utf_ptr2char(p: *const c_char) -> c_int;
    fn nvim_syn_utfc_ptr2len(p: *mut c_char) -> c_int;
    fn nvim_syn_vim_isprintc(c: c_int) -> c_int;

    // Syntax functions
    fn nvim_syn_get_b_syn_conceal() -> c_int;
    fn nvim_syn_get_current_inc_tag() -> c_int;
    fn rs_syn_check_cluster(pp: *mut c_char, len: c_int) -> c_int;
    fn nvim_syn_name2id_wrapper(name: *const c_char) -> c_int;
    fn nvim_syn_check_group_wrapper(name: *const c_char, len: c_int) -> c_int;
    fn nvim_syn_highlight_num_groups() -> c_int;
    fn nvim_syn_highlight_group_name(idx: c_int) -> *mut c_char;
    // Synpat accessors (explicit block param)
    fn nvim_syn_get_curwin_synblock() -> crate::types::SynBlockHandle;
    fn nvim_synblock_get_pattern_count(block: crate::types::SynBlockHandle) -> c_int;
    fn nvim_synblock_get_pattern(block: crate::types::SynBlockHandle, idx: c_int) -> SynPatHandle;
    fn nvim_synpat_get_syn_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_type(pat: SynPatHandle) -> c_int;

    // Regexp
    fn nvim_syn_vim_regcomp(pat: *mut c_char, flags: c_int) -> *mut c_void;
    fn nvim_syn_vim_regfree(regprog: *mut c_void);

    // Direct Rust regexp exec (replaces nvim_syn_vim_regexec):
    // takes *mut *mut c_void so regprog updates on NFA fallback are visible
    fn rs_vim_regexec_prog(
        prog_ptr: *mut *mut c_void,
        ignore_case: c_int,
        line: *const u8,
        col: i32,
    ) -> c_int;

    // Fold
    fn nvim_syn_foldmethod_is_syntax_curwin() -> c_int;
    fn nvim_syn_fold_update_all_curwin();

}

/// FAIL return value from C.
const FAIL: c_int = 0;

// =============================================================================
// Helpers replacing deleted C functions
// =============================================================================

/// Find the pattern index matching `syn_id` + `SPTYPE_START` in curwin's patterns.
/// Returns the index, or -1 if not found.
///
/// Replaces C `nvim_syn_find_sync_pattern_idx`.
///
/// # Safety
/// Accesses C global state (curwin).
unsafe fn find_sync_pattern_idx(syn_id: c_int) -> c_int {
    let block = nvim_syn_get_curwin_synblock();
    let count = nvim_synblock_get_pattern_count(block);
    let mut i = count - 1;
    while i >= 0 {
        let pat = nvim_synblock_get_pattern(block, i);
        if !pat.is_null()
            && nvim_synpat_get_syn_id(pat) as c_int == syn_id
            && nvim_synpat_get_type(pat) == SPTYPE_START
        {
            return i;
        }
        i -= 1;
    }
    -1
}

// =============================================================================
// Flag table for option matching
// =============================================================================

/// Flag table entry describing a syntax option keyword.
struct FlagEntry {
    /// Interleaved-case name (e.g., "cCoOnNtTaAiInNeEdD").
    name: &'static [u8],
    /// Argument type: 0 = flag, 1 = contains, 2 = containedin, 3 = nextgroup, 11 = cchar.
    argtype: i32,
    /// HL_ flag to set when argtype == 0.
    flags: i32,
}

/// The flag table, matching the C flagtab[].
static FLAGTAB: &[FlagEntry] = &[
    FlagEntry {
        name: b"cCoOnNtTaAiInNeEdD",
        argtype: 0,
        flags: HL_CONTAINED,
    },
    FlagEntry {
        name: b"oOnNeElLiInNeE",
        argtype: 0,
        flags: HL_ONELINE,
    },
    FlagEntry {
        name: b"kKeEeEpPeEnNdD",
        argtype: 0,
        flags: HL_KEEPEND,
    },
    FlagEntry {
        name: b"eExXtTeEnNdD",
        argtype: 0,
        flags: HL_EXTEND,
    },
    FlagEntry {
        name: b"eExXcClLuUdDeEnNlL",
        argtype: 0,
        flags: HL_EXCLUDENL,
    },
    FlagEntry {
        name: b"tTrRaAnNsSpPaArReEnNtT",
        argtype: 0,
        flags: HL_TRANSP,
    },
    FlagEntry {
        name: b"sSkKiIpPnNlL",
        argtype: 0,
        flags: HL_SKIPNL,
    },
    FlagEntry {
        name: b"sSkKiIpPwWhHiItTeE",
        argtype: 0,
        flags: HL_SKIPWHITE,
    },
    FlagEntry {
        name: b"sSkKiIpPeEmMpPtTyY",
        argtype: 0,
        flags: HL_SKIPEMPTY,
    },
    FlagEntry {
        name: b"gGrRoOuUpPhHeErReE",
        argtype: 0,
        flags: HL_SYNC_HERE,
    },
    FlagEntry {
        name: b"gGrRoOuUpPtThHeErReE",
        argtype: 0,
        flags: HL_SYNC_THERE,
    },
    FlagEntry {
        name: b"dDiIsSpPlLaAyY",
        argtype: 0,
        flags: HL_DISPLAY,
    },
    FlagEntry {
        name: b"fFoOlLdD",
        argtype: 0,
        flags: HL_FOLD,
    },
    FlagEntry {
        name: b"cCoOnNcCeEaAlL",
        argtype: 0,
        flags: HL_CONCEAL,
    },
    FlagEntry {
        name: b"cCoOnNcCeEaAlLeEnNdDsS",
        argtype: 0,
        flags: HL_CONCEALENDS,
    },
    FlagEntry {
        name: b"cCcChHaArR",
        argtype: 11,
        flags: 0,
    },
    FlagEntry {
        name: b"cCoOnNtTaAiInNsS",
        argtype: 1,
        flags: 0,
    },
    FlagEntry {
        name: b"cCoOnNtTaAiInNeEdDiInN",
        argtype: 2,
        flags: 0,
    },
    FlagEntry {
        name: b"nNeExXtTgGrRoOuUpP",
        argtype: 3,
        flags: 0,
    },
];

/// First letters of all flag names (for quick skip).
const FIRST_LETTERS: &[u8] = b"cCoOkKeEtTsSgGdDfFnN";

/// Match an argument string against a flag's interleaved-case name.
/// Returns the number of characters consumed from `arg`, or 0 if no match.
fn match_flag(arg: *const u8, flag: &FlagEntry) -> usize {
    let name = flag.name;
    let mut len = 0usize;
    let mut i = 0usize;
    while i < name.len() {
        let c = unsafe { *arg.add(len) };
        if c != name[i] && c != name[i + 1] {
            return 0;
        }
        i += 2;
        len += 1;
    }
    len
}

// =============================================================================
// get_syn_options implementation
// =============================================================================

/// Parse syntax options from the argument string.
///
/// This is the Rust implementation of `get_syn_options`. It modifies the
/// option struct fields via the provided pointers.
///
/// # Returns
/// The advanced argument pointer, or null on error.
///
/// # Safety
/// All pointer arguments must be valid and properly aligned.
#[allow(clippy::too_many_arguments)]
pub unsafe fn get_syn_options_impl(
    mut arg: *mut c_char,
    opt_flags: *mut c_int,
    opt_keyword: c_int,
    opt_sync_idx: *mut c_int, // NULL if sync_idx not allowed
    opt_has_cont_list: c_int,
    opt_cont_list: *mut *mut i16,
    opt_cont_in_list: *mut *mut i16,
    opt_next_list: *mut *mut i16,
    conceal_char: *mut c_int,
    skip: c_int,
) -> *mut c_char {
    if arg.is_null() {
        return std::ptr::null_mut();
    }

    if nvim_syn_get_b_syn_conceal() != 0 {
        *opt_flags |= HL_CONCEAL;
    }

    loop {
        // Quick check: skip if first char can't start any flag name.
        let first = *arg as u8;
        if !FIRST_LETTERS.contains(&first) {
            break;
        }

        // Try to match each flag.
        let mut fidx: i32 = -1;
        let mut matched_len: usize = 0;
        for (i, flag) in FLAGTAB.iter().enumerate().rev() {
            let len = match_flag(arg as *const u8, flag);
            if len > 0 {
                let next_char = *arg.add(len) as u8;
                let is_match = if flag.argtype > 0 {
                    next_char == b'='
                } else {
                    nvim_syn_ascii_iswhite_char(next_char as c_int) != 0
                        || nvim_syn_ends_excmd(next_char as c_int) != 0
                };
                if is_match {
                    // Treat "display", "fold" and "extend" as keywords
                    if opt_keyword != 0
                        && (flag.flags == HL_DISPLAY
                            || flag.flags == HL_FOLD
                            || flag.flags == HL_EXTEND)
                    {
                        fidx = -1;
                    } else {
                        fidx = i as i32;
                        matched_len = len;
                    }
                    break;
                }
            }
        }

        if fidx < 0 {
            break;
        }

        let flag = &FLAGTAB[fidx as usize];

        if flag.argtype == 1 {
            // contains=
            if opt_has_cont_list == 0 {
                nvim_syn_emsg(c"E395: Contains argument not accepted here".as_ptr());
                return std::ptr::null_mut();
            }
            if get_id_list_impl(&mut arg, 8, opt_cont_list, skip) == FAIL {
                return std::ptr::null_mut();
            }
        } else if flag.argtype == 2 {
            // containedin=
            if get_id_list_impl(&mut arg, 11, opt_cont_in_list, skip) == FAIL {
                return std::ptr::null_mut();
            }
        } else if flag.argtype == 3 {
            // nextgroup=
            if get_id_list_impl(&mut arg, 9, opt_next_list, skip) == FAIL {
                return std::ptr::null_mut();
            }
        } else if flag.argtype == 11 && *arg.add(5) as u8 == b'=' {
            // cchar=?
            *conceal_char = nvim_syn_utf_ptr2char(arg.add(6));
            let char_len = nvim_syn_utfc_ptr2len(arg.add(6));
            arg = arg.add(char_len as usize - 1);
            if nvim_syn_vim_isprintc(*conceal_char) == 0 {
                nvim_syn_emsg(c"E844: invalid cchar value".as_ptr());
                return std::ptr::null_mut();
            }
            arg = nvim_syn_skipwhite(arg.add(7));
        } else {
            // Simple flag
            *opt_flags |= flag.flags;
            arg = nvim_syn_skipwhite(arg.add(matched_len));

            if flag.flags == HL_SYNC_HERE || flag.flags == HL_SYNC_THERE {
                if opt_sync_idx.is_null() {
                    nvim_syn_emsg(c"E393: group[t]here not accepted here".as_ptr());
                    return std::ptr::null_mut();
                }
                let gname_start = arg;
                arg = nvim_syn_skiptowhite(arg);
                if gname_start == arg {
                    return std::ptr::null_mut();
                }
                let gname_len = arg.offset_from(gname_start) as c_int;
                let gname = nvim_syn_xstrnsave(gname_start, gname_len);

                // Check for "NONE"
                let is_none = *gname.add(0) == b'N' as c_char
                    && *gname.add(1) == b'O' as c_char
                    && *gname.add(2) == b'N' as c_char
                    && *gname.add(3) == b'E' as c_char
                    && *gname.add(4) == 0;

                if is_none {
                    *opt_sync_idx = NONE_IDX;
                } else {
                    let syn_id = nvim_syn_name2id_wrapper(gname);
                    let idx = find_sync_pattern_idx(syn_id);
                    if idx < 0 {
                        nvim_syn_semsg_1s(c"E394: Didn't find region item for %s".as_ptr(), gname);
                        nvim_syn_xfree(gname as *mut c_void);
                        return std::ptr::null_mut();
                    }
                    *opt_sync_idx = idx;
                }

                nvim_syn_xfree(gname as *mut c_void);
                arg = nvim_syn_skipwhite(arg);
            } else if flag.flags == HL_FOLD && nvim_syn_foldmethod_is_syntax_curwin() != 0 {
                nvim_syn_fold_update_all_curwin();
            }
        }
    }

    arg
}

// =============================================================================
// get_id_list implementation
// =============================================================================

/// Parse a syntax ID list using Vec<i16> instead of C's two-pass approach.
///
/// This replaces the C `get_id_list` function. The Vec grows dynamically,
/// avoiding the two-pass count-then-allocate pattern.
///
/// # Returns
/// OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// All pointer arguments must be valid.
pub unsafe fn get_id_list_impl(
    arg: *mut *mut c_char,
    keylen: c_int,
    list: *mut *mut i16,
    skip: c_int,
) -> c_int {
    let mut ids: Vec<i16> = Vec::new();
    let mut failed = false;

    // skip keyword (e.g., "contains")
    let mut p = nvim_syn_skipwhite((*arg).add(keylen as usize));
    if *p as u8 != b'=' {
        nvim_syn_semsg_1s(c"E405: Missing equal sign: %s".as_ptr(), *arg);
        *arg = p;
        return FAIL;
    }
    p = nvim_syn_skipwhite(p.add(1));
    if nvim_syn_ends_excmd(*p as c_int) != 0 {
        nvim_syn_semsg_1s(c"E406: Empty argument: %s".as_ptr(), *arg);
        *arg = p;
        return FAIL;
    }

    // Parse items
    let mut count = 0;
    loop {
        // Find end of current item
        let mut end = p;
        while *end != 0 && nvim_syn_ascii_iswhite_char(*end as c_int) == 0 && *end as u8 != b',' {
            end = end.add(1);
        }

        let item_len = end.offset_from(p) as c_int;

        // Allocate name with room for "^" prefix and "$" suffix
        let name = nvim_syn_xmalloc(item_len + 3) as *mut c_char;
        nvim_syn_xmemcpyz(name.add(1), p, item_len);
        *name = 0; // placeholder for potential "^"

        // Check for special keywords
        let name_str = name.add(1);
        let is_allbut = c_str_eq(name_str, b"ALLBUT\0");
        let is_all = c_str_eq(name_str, b"ALL\0");
        let is_top = c_str_eq(name_str, b"TOP\0");
        let is_contained = c_str_eq(name_str, b"CONTAINED\0");

        let mut id: c_int;

        if is_allbut || is_all || is_top || is_contained {
            if nvim_syn_toupper_asc(**arg as c_int) != b'C' as c_int {
                nvim_syn_semsg_1s(c"E407: %s not allowed here".as_ptr(), name_str);
                failed = true;
                nvim_syn_xfree(name as *mut c_void);
                break;
            }
            if count != 0 {
                nvim_syn_semsg_1s(
                    c"E408: %s must be first in contains list".as_ptr(),
                    name_str,
                );
                failed = true;
                nvim_syn_xfree(name as *mut c_void);
                break;
            }
            if is_allbut || is_all {
                id = SYNID_ALLBUT;
            } else if is_top {
                id = SYNID_TOP;
            } else {
                id = SYNID_CONTAINED;
            }
            id += nvim_syn_get_current_inc_tag();
        } else if *name_str as u8 == b'@' {
            // Cluster reference
            if skip != 0 {
                id = -1;
            } else {
                id = rs_syn_check_cluster(name.add(2) as *mut c_char, item_len - 1);
            }
        } else {
            // Group name - check for regexp metacharacters
            let metacharset = b"\\.*^$~[\0";
            if nvim_syn_strpbrk(name_str, metacharset.as_ptr() as *const c_char).is_null() {
                // Simple name
                id = nvim_syn_check_group_wrapper(name_str, item_len);
            } else {
                // Regexp match against group names
                *name = b'^' as c_char;
                // Append "$" after the name
                let name_end = name.add(1 + item_len as usize);
                *name_end = b'$' as c_char;
                *name_end.add(1) = 0;

                let regprog = nvim_syn_vim_regcomp(name, RE_MAGIC);
                if regprog.is_null() {
                    failed = true;
                    nvim_syn_xfree(name as *mut c_void);
                    break;
                }

                id = 0;
                let num_groups = nvim_syn_highlight_num_groups();
                let mut cur_regprog = regprog;
                for i in (0..num_groups).rev() {
                    let group_name = nvim_syn_highlight_group_name(i);
                    if rs_vim_regexec_prog(&mut cur_regprog, 1, group_name.cast::<u8>(), 0) != 0 {
                        ids.push((i + 1) as i16);
                        id = -1; // Remember we found one
                    }
                }
                nvim_syn_vim_regfree(cur_regprog);
            }
        }

        nvim_syn_xfree(name as *mut c_void);

        if id == 0 {
            nvim_syn_semsg_1s(c"E409: Unknown group name: %s".as_ptr(), p);
            failed = true;
            break;
        }
        if id > 0 {
            ids.push(id as i16);
        }
        count += 1;

        p = nvim_syn_skipwhite(end);
        if *p as u8 != b',' {
            break;
        }
        p = nvim_syn_skipwhite(p.add(1)); // skip comma
        if nvim_syn_ends_excmd(*p as c_int) != 0 {
            break;
        }
    }

    *arg = p;
    if failed || ids.is_empty() {
        return FAIL;
    }

    if (*list).is_null() {
        // Allocate C array from the Vec and store it
        let c_list =
            nvim_syn_xmalloc(((ids.len() + 1) * std::mem::size_of::<i16>()) as c_int) as *mut i16;
        for (i, &id) in ids.iter().enumerate() {
            *c_list.add(i) = id;
        }
        *c_list.add(ids.len()) = 0; // zero-terminate
        *list = c_list;
    }
    // else: list already found, don't overwrite

    1 // OK
}

/// Compare a C string with a byte literal.
unsafe fn c_str_eq(s: *const c_char, literal: &[u8]) -> bool {
    let mut i = 0;
    loop {
        let c = *s.add(i) as u8;
        if i >= literal.len() - 1 {
            // literal's last byte is \0
            return c == 0;
        }
        if c != literal[i] {
            return false;
        }
        if c == 0 {
            return i == literal.len() - 1;
        }
        i += 1;
    }
}

// =============================================================================
// Exported FFI functions
// =============================================================================

/// Rust implementation of get_syn_options.
///
/// Called from C with the decomposed syn_opt_arg_T fields.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_get_syn_options(
    arg: *mut c_char,
    opt_flags: *mut c_int,
    opt_keyword: c_int,
    opt_sync_idx: *mut c_int,
    opt_has_cont_list: c_int,
    opt_cont_list: *mut *mut i16,
    opt_cont_in_list: *mut *mut i16,
    opt_next_list: *mut *mut i16,
    conceal_char: *mut c_int,
    skip: c_int,
) -> *mut c_char {
    get_syn_options_impl(
        arg,
        opt_flags,
        opt_keyword,
        opt_sync_idx,
        opt_has_cont_list,
        opt_cont_list,
        opt_cont_in_list,
        opt_next_list,
        conceal_char,
        skip,
    )
}

/// Rust implementation of get_id_list.
#[no_mangle]
pub unsafe extern "C" fn rs_get_id_list(
    arg: *mut *mut c_char,
    keylen: c_int,
    list: *mut *mut i16,
    skip: c_int,
) -> c_int {
    get_id_list_impl(arg, keylen, list, skip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_flag_contained() {
        let arg = b"contained \0";
        let len = match_flag(arg.as_ptr(), &FLAGTAB[0]);
        assert_eq!(len, 9); // "contained" is 9 chars
    }

    #[test]
    fn test_match_flag_oneline() {
        let arg = b"oneline \0";
        let len = match_flag(arg.as_ptr(), &FLAGTAB[1]);
        assert_eq!(len, 7);
    }

    #[test]
    fn test_match_flag_case_insensitive() {
        let arg = b"CONTAINED \0";
        let len = match_flag(arg.as_ptr(), &FLAGTAB[0]);
        assert_eq!(len, 9);
    }

    #[test]
    fn test_match_flag_mixed_case() {
        let arg = b"Contained \0";
        let len = match_flag(arg.as_ptr(), &FLAGTAB[0]);
        assert_eq!(len, 9);
    }

    #[test]
    fn test_match_flag_no_match() {
        let arg = b"foobar \0";
        let len = match_flag(arg.as_ptr(), &FLAGTAB[0]);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_match_flag_cchar() {
        let arg = b"cchar=x\0";
        let len = match_flag(arg.as_ptr(), &FLAGTAB[15]); // cchar entry
        assert_eq!(len, 5); // "cchar" is 5 chars
    }

    #[test]
    fn test_match_flag_contains() {
        let arg = b"contains=foo\0";
        let len = match_flag(arg.as_ptr(), &FLAGTAB[16]); // contains entry
        assert_eq!(len, 8); // "contains" is 8 chars
    }

    #[test]
    fn test_first_letters() {
        // Verify all flag names start with a letter in FIRST_LETTERS
        for flag in FLAGTAB.iter() {
            let first = flag.name[0];
            assert!(
                FIRST_LETTERS.contains(&first),
                "Flag name starts with {:?} which is not in FIRST_LETTERS",
                first as char
            );
        }
    }

    #[test]
    fn test_c_str_eq() {
        unsafe {
            assert!(c_str_eq(c"ALLBUT".as_ptr(), b"ALLBUT\0"));
            assert!(c_str_eq(c"NONE".as_ptr(), b"NONE\0"));
            assert!(!c_str_eq(c"ALL".as_ptr(), b"ALLBUT\0"));
            assert!(!c_str_eq(c"ALLBUT".as_ptr(), b"ALL\0"));
            assert!(c_str_eq(c"".as_ptr(), b"\0"));
        }
    }

    #[test]
    fn test_flag_count() {
        assert_eq!(FLAGTAB.len(), 19);
    }
}
