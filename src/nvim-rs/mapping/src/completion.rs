//! Completion and serialization functions for mapping commands.
//!
//! Provides `set_context_in_map_cmd` (completion context setup),
//! `ExpandMappings` (expand mapping names for tab completion),
//! `makemap_mode_chars` (mode→prefix decomposition for `:mksession`),
//! and `put_escstr` logic (escape key codes for file output).

use std::ffi::{c_char, c_int};

use crate::{
    mapblock_keys, mapblock_mode, mapblock_next, mapblock_noremap, mapblock_simplified, BufHandle,
    MapblockHandle, MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, MODE_OP_PENDING,
    MODE_SELECT, MODE_TERMINAL, MODE_VISUAL,
};

// =============================================================================
// Constants
// =============================================================================

const K_SPECIAL: u8 = 0x80;
const KS_EXTRA: u8 = 253;
const KE_SNR: u8 = 82;

const NUL: u8 = 0;
const NL: u8 = 10;

const LUA_NOREF: c_int = -2;
const REMAP_SCRIPT: c_int = -2;

const EXPAND_NOTHING: c_int = 0;
const EXPAND_MAPPINGS: c_int = 17;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn nvim_get_maphash_entry(index: c_int) -> MapblockHandle;
    fn nvim_get_first_abbr() -> MapblockHandle;
    fn nvim_buf_get_maphash_entry(buf: BufHandle, index: c_int) -> MapblockHandle;
    fn nvim_buf_get_first_abbr(buf: BufHandle) -> MapblockHandle;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_mapping_get_p_cpo() -> *const c_char;

    fn nvim_mapblock_get_str(mp: MapblockHandle) -> *const c_char;
    fn nvim_mapblock_get_luaref(mp: MapblockHandle) -> c_int;

    fn rs_get_map_mode(cmdp: *mut *mut c_char, forceit: c_int) -> c_int;
    fn rs_translate_mapping(str_in: *const c_char, cpo_val: *const c_char) -> *mut c_char;

    // Completion infrastructure — called from ExpandMappings
    fn nvim_mapping_vim_regexec(regmatch: *mut c_char, s: *const c_char) -> c_int;
    fn nvim_mapping_fuzzy_match_str(s: *const c_char, pat: *const c_char) -> c_int;
    fn nvim_mapping_cmdline_fuzzy_complete(pat: *const c_char) -> c_int;
    fn nvim_mapping_ga_append_str(ga: *mut c_char, s: *const c_char);
    fn nvim_mapping_ga_append_fuzmatch(ga: *mut c_char, s: *const c_char, score: c_int);
    fn nvim_mapping_expand_finish(
        ga: *mut c_char,
        fuzzy: c_int,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_mapping_ga_init_str(ga: *mut c_char);
    fn nvim_mapping_ga_init_fuzmatch(ga: *mut c_char);

    // expand_T field accessors (from cmdexpand.c)
    fn nvim_expand_set_context(xp: *mut c_char, context: c_int);
    fn nvim_expand_set_pattern(xp: *mut c_char, pattern: *mut c_char);

    // Global statics for expand state
    fn nvim_mapping_set_expand_mapmodes(val: c_int);
    fn nvim_mapping_set_expand_isabbrev(val: c_int);
    fn nvim_mapping_set_expand_buffer(val: c_int);
    fn nvim_mapping_get_expand_mapmodes() -> c_int;
    fn nvim_mapping_get_expand_isabbrev() -> c_int;
    fn nvim_mapping_get_expand_buffer() -> c_int;

    fn nvim_mapping_get_cmd_map() -> c_int;
    fn nvim_mapping_get_cmd_unmap() -> c_int;

    fn xfree(ptr: *mut c_char);

    // String helpers
    fn nvim_mapping_skipwhite(p: *const c_char) -> *mut c_char;
}

// =============================================================================
// set_context_in_map_cmd
// =============================================================================

/// Set up completion context for `:map` and `:abbreviate` commands.
///
/// Parses modifier arguments (`<buffer>`, `<silent>`, etc.) and sets
/// the expansion context and pattern for tab-completion.
///
/// # Safety
/// `xp`, `cmd`, and `arg` must be valid pointers. `cmd` and `arg` must be
/// NUL-terminated C strings.
#[export_name = "set_context_in_map_cmd"]
pub unsafe extern "C" fn rs_set_context_in_map_cmd(
    xp: *mut c_char,
    cmd: *mut c_char,
    arg: *mut c_char,
    forceit: c_int,
    isabbrev: c_int,
    isunmap: c_int,
    cmdidx: c_int,
) -> *mut c_char {
    let cmd_map = nvim_mapping_get_cmd_map();
    let cmd_unmap = nvim_mapping_get_cmd_unmap();

    if forceit != 0 && cmdidx != cmd_map && cmdidx != cmd_unmap {
        nvim_expand_set_context(xp, EXPAND_NOTHING);
    } else {
        let mode = if isunmap != 0 {
            let mut cmd_ptr = cmd;
            rs_get_map_mode(
                std::ptr::addr_of_mut!(cmd_ptr),
                c_int::from(forceit != 0 || isabbrev != 0),
            )
        } else {
            let mut m = MODE_INSERT | MODE_CMDLINE;
            if isabbrev == 0 {
                m |= MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
            }
            m
        };
        nvim_mapping_set_expand_mapmodes(mode);
        nvim_mapping_set_expand_isabbrev(isabbrev);
        nvim_expand_set_context(xp, EXPAND_MAPPINGS);
        nvim_mapping_set_expand_buffer(0);

        let mut p = arg;
        loop {
            if libc::strncmp(p, c"<buffer>".as_ptr(), 8) == 0 {
                nvim_mapping_set_expand_buffer(1);
                p = nvim_mapping_skipwhite(p.add(8));
                continue;
            }
            if libc::strncmp(p, c"<unique>".as_ptr(), 8) == 0 {
                p = nvim_mapping_skipwhite(p.add(8));
                continue;
            }
            if libc::strncmp(p, c"<nowait>".as_ptr(), 8) == 0 {
                p = nvim_mapping_skipwhite(p.add(8));
                continue;
            }
            if libc::strncmp(p, c"<silent>".as_ptr(), 8) == 0 {
                p = nvim_mapping_skipwhite(p.add(8));
                continue;
            }
            if libc::strncmp(p, c"<special>".as_ptr(), 9) == 0 {
                p = nvim_mapping_skipwhite(p.add(9));
                continue;
            }
            if libc::strncmp(p, c"<script>".as_ptr(), 8) == 0 {
                p = nvim_mapping_skipwhite(p.add(8));
                continue;
            }
            if libc::strncmp(p, c"<expr>".as_ptr(), 6) == 0 {
                p = nvim_mapping_skipwhite(p.add(6));
                continue;
            }
            break;
        }
        nvim_expand_set_pattern(xp, p);
    }

    std::ptr::null_mut()
}

// =============================================================================
// ExpandMappings
// =============================================================================

/// Modifier keywords to match during completion.
static MODIFIER_KEYWORDS: [&[u8]; 7] = [
    b"<silent>\0",
    b"<unique>\0",
    b"<script>\0",
    b"<expr>\0",
    b"<buffer>\0",
    b"<nowait>\0",
    b"<special>\0",
];

/// Find all mapping/abbreviation names matching a pattern for command-line
/// completion.
///
/// # Safety
/// All pointer parameters must be valid. `pat` must be NUL-terminated.
/// `regmatch` must be a valid `regmatch_T*` (passed opaquely).
#[export_name = "ExpandMappings"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_expand_mappings(
    pat: *mut c_char,
    regmatch: *mut c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let fuzzy = nvim_mapping_cmdline_fuzzy_complete(pat) != 0;

    *num_matches = 0;
    *matches = std::ptr::null_mut();

    // garray_T is 20 bytes on 64-bit, allocate on stack as opaque blob
    let mut ga_storage = [0u8; 64]; // generous for garray_T
    let ga = ga_storage.as_mut_ptr().cast::<c_char>();

    if fuzzy {
        nvim_mapping_ga_init_fuzmatch(ga);
    } else {
        nvim_mapping_ga_init_str(ga);
    }

    let expand_buffer = nvim_mapping_get_expand_buffer() != 0;
    let expand_isabbrev = nvim_mapping_get_expand_isabbrev() != 0;
    let expand_mapmodes = nvim_mapping_get_expand_mapmodes();

    // First search in map modifier arguments
    for (i, keyword) in MODIFIER_KEYWORDS.iter().enumerate() {
        // Skip <buffer> if already used
        if i == 4 && expand_buffer {
            continue;
        }

        let p = keyword.as_ptr().cast::<c_char>();

        let (is_match, score) = if fuzzy {
            let s = nvim_mapping_fuzzy_match_str(p, pat);
            (s != i32::MIN, s)
        } else {
            (nvim_mapping_vim_regexec(regmatch, p) != 0, 0)
        };

        if !is_match {
            continue;
        }

        if fuzzy {
            nvim_mapping_ga_append_fuzmatch(ga, p, score);
        } else {
            nvim_mapping_ga_append_str(ga, p);
        }
    }

    // Search through mapping hash lists
    let curbuf = nvim_get_curbuf();
    for hash in 0..256 {
        let mp = if expand_isabbrev {
            if hash > 0 {
                break;
            }
            nvim_get_first_abbr()
        } else if expand_buffer {
            nvim_buf_get_maphash_entry(curbuf, hash)
        } else {
            nvim_get_maphash_entry(hash)
        };

        let mut cur = mp;
        while !cur.is_null() {
            if mapblock_simplified(cur) || (mapblock_mode(cur) & expand_mapmodes) == 0 {
                cur = mapblock_next(cur);
                continue;
            }

            let p_cpo = nvim_mapping_get_p_cpo();
            let p = rs_translate_mapping(mapblock_keys(cur), p_cpo);
            if p.is_null() {
                cur = mapblock_next(cur);
                continue;
            }

            let (is_match, score) = if fuzzy {
                let s = nvim_mapping_fuzzy_match_str(p, pat);
                (s != i32::MIN, s)
            } else {
                (nvim_mapping_vim_regexec(regmatch, p) != 0, 0)
            };

            if !is_match {
                xfree(p);
                cur = mapblock_next(cur);
                continue;
            }

            // Both paths take ownership via xstrdup inside the ga_append wrappers
            if fuzzy {
                nvim_mapping_ga_append_fuzmatch(ga, p, score);
            } else {
                nvim_mapping_ga_append_str(ga, p);
            }
            xfree(p);

            cur = mapblock_next(cur);
        }
    }

    // Finish: sort, dedup, and set output pointers
    nvim_mapping_expand_finish(ga, c_int::from(fuzzy), num_matches, matches)
}

// =============================================================================
// makemap mode decomposition
// =============================================================================

/// Result of decomposing a mapping mode into the command prefix characters
/// needed to recreate it.
#[repr(C)]
pub struct MakemapModeResult {
    /// First prefix character (NUL if none needed).
    pub c1: c_char,
    /// Second prefix character (NUL if only one needed).
    pub c2: c_char,
    /// Third prefix character (NUL if only two needed).
    pub c3: c_char,
    /// Whether to use "map!" instead of "map" as the command
    /// (only for `MODE_INSERT|MODE_CMDLINE` in non-abbreviation mode).
    pub use_bang: c_int,
    /// 0 = OK, non-zero = error (illegal mode).
    pub error: c_int,
}

/// Decompose a mapping mode into the c1/c2/c3 prefix characters needed
/// to recreate the mapping in a `:map` command.
///
/// This is the core logic from `makemap()` — the big mode→c1/c2/c3 switch.
#[no_mangle]
pub extern "C" fn rs_makemap_mode_chars(mode: c_int, abbr: c_int) -> MakemapModeResult {
    let mut r = MakemapModeResult {
        c1: 0,
        c2: 0,
        c3: 0,
        use_bang: 0,
        error: 0,
    };

    match mode {
        m if m == MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING => {}
        m if m == MODE_NORMAL => r.c1 = b'n' as c_char,
        m if m == MODE_VISUAL => r.c1 = b'x' as c_char,
        m if m == MODE_SELECT => r.c1 = b's' as c_char,
        m if m == MODE_OP_PENDING => r.c1 = b'o' as c_char,
        m if m == MODE_NORMAL | MODE_VISUAL => {
            r.c1 = b'n' as c_char;
            r.c2 = b'x' as c_char;
        }
        m if m == MODE_NORMAL | MODE_SELECT => {
            r.c1 = b'n' as c_char;
            r.c2 = b's' as c_char;
        }
        m if m == MODE_NORMAL | MODE_OP_PENDING => {
            r.c1 = b'n' as c_char;
            r.c2 = b'o' as c_char;
        }
        m if m == MODE_VISUAL | MODE_SELECT => {
            r.c1 = b'v' as c_char;
        }
        m if m == MODE_VISUAL | MODE_OP_PENDING => {
            r.c1 = b'x' as c_char;
            r.c2 = b'o' as c_char;
        }
        m if m == MODE_SELECT | MODE_OP_PENDING => {
            r.c1 = b's' as c_char;
            r.c2 = b'o' as c_char;
        }
        m if m == MODE_NORMAL | MODE_VISUAL | MODE_SELECT => {
            r.c1 = b'n' as c_char;
            r.c2 = b'v' as c_char;
        }
        m if m == MODE_NORMAL | MODE_VISUAL | MODE_OP_PENDING => {
            r.c1 = b'n' as c_char;
            r.c2 = b'x' as c_char;
            r.c3 = b'o' as c_char;
        }
        m if m == MODE_NORMAL | MODE_SELECT | MODE_OP_PENDING => {
            r.c1 = b'n' as c_char;
            r.c2 = b's' as c_char;
            r.c3 = b'o' as c_char;
        }
        m if m == MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING => {
            r.c1 = b'v' as c_char;
            r.c2 = b'o' as c_char;
        }
        m if m == MODE_CMDLINE | MODE_INSERT => {
            if abbr == 0 {
                r.use_bang = 1;
            }
        }
        m if m == MODE_CMDLINE => r.c1 = b'c' as c_char,
        m if m == MODE_INSERT => r.c1 = b'i' as c_char,
        m if m == MODE_LANGMAP => r.c1 = b'l' as c_char,
        m if m == MODE_TERMINAL => r.c1 = b't' as c_char,
        _ => {
            r.error = 1;
        }
    }

    r
}

// =============================================================================
// makemap skip check
// =============================================================================

/// Check whether a mapblock should be skipped in `makemap()`.
///
/// Returns true if the mapping should be skipped (script-local, Lua,
/// or contains `<SNR>`).
///
/// # Safety
/// `mp` must be a valid mapblock handle.
#[no_mangle]
pub unsafe extern "C" fn rs_makemap_should_skip(mp: MapblockHandle) -> c_int {
    // Skip script-local mappings
    if mapblock_noremap(mp) == REMAP_SCRIPT {
        return 1;
    }

    // Skip Lua mappings
    let luaref = nvim_mapblock_get_luaref(mp);
    if luaref != LUA_NOREF {
        return 1;
    }

    // Skip mappings containing <SNR>
    let str_ptr = nvim_mapblock_get_str(mp);
    if !str_ptr.is_null() {
        let mut p = str_ptr.cast::<u8>();
        while *p != NUL {
            if *p == K_SPECIAL && *p.add(1) == KS_EXTRA && *p.add(2) == KE_SNR {
                return 1;
            }
            p = p.add(1);
        }
    }

    0
}

// =============================================================================
// makemap needs_cpo check
// =============================================================================

/// Check whether a mapblock's keys or str contain special characters
/// that require saving/restoring `cpo`.
///
/// Returns true if `cpo` handling is needed.
///
/// # Safety
/// `mp` must be a valid mapblock handle.
#[no_mangle]
pub unsafe extern "C" fn rs_makemap_needs_cpo(mp: MapblockHandle) -> c_int {
    let str_ptr = nvim_mapblock_get_str(mp);
    let keys_ptr = mapblock_keys(mp);

    // If m_str is empty (NUL), will use <Nop> — needs cpo
    if !str_ptr.is_null() && *str_ptr.cast::<u8>() == NUL {
        return 1;
    }

    // Check if m_str or m_keys contain K_SPECIAL or NL
    for ptr in [str_ptr, keys_ptr] {
        if ptr.is_null() {
            continue;
        }
        let mut p = ptr.cast::<u8>();
        while *p != NUL {
            if *p == K_SPECIAL || *p == NL {
                return 1;
            }
            p = p.add(1);
        }
    }

    0
}

// =============================================================================
// put_escstr logic
// =============================================================================

/// Check if a character needs CTRL-V escaping in put_escstr.
///
/// `what`: 0 = :map lhs, 1 = :map rhs, 2 = :set
/// `c`: the character byte
/// `is_first`: true if this is the first character (for rhs space escaping)
///
/// Returns: 0 = no escaping, 1 = backslash escape, 2 = Ctrl-V escape
#[no_mangle]
pub extern "C" fn rs_put_escstr_escape_type(what: c_int, c: c_int, is_first: c_int) -> c_int {
    let c_u8 = c as u8;

    // :set mode: backslash-escape whitespace, quote, and backslash
    if what == 2 && (c_u8 == b' ' || c_u8 == b'\t' || c_u8 == b'"' || c_u8 == b'\\') {
        return 1; // backslash escape
    }

    // Control chars, high bytes, pipe, leading space in lhs, leading space in rhs, '<'
    if c < c_int::from(b' ')
        || c > c_int::from(b'~')
        || c == c_int::from(b'|')
        || (what == 0 && c == c_int::from(b' '))
        || (what == 1 && is_first != 0 && c == c_int::from(b' '))
        || (what != 2 && c == c_int::from(b'<'))
    {
        return 2; // Ctrl-V escape
    }

    0 // no escape
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_makemap_mode_chars_normal_visual_select_op() {
        let r = rs_makemap_mode_chars(MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING, 0);
        assert_eq!(r.c1, 0);
        assert_eq!(r.c2, 0);
        assert_eq!(r.c3, 0);
        assert_eq!(r.use_bang, 0);
        assert_eq!(r.error, 0);
    }

    #[test]
    fn test_makemap_mode_chars_normal() {
        let r = rs_makemap_mode_chars(MODE_NORMAL, 0);
        assert_eq!(r.c1, b'n' as c_char);
        assert_eq!(r.c2, 0);
        assert_eq!(r.c3, 0);
    }

    #[test]
    fn test_makemap_mode_chars_visual() {
        let r = rs_makemap_mode_chars(MODE_VISUAL, 0);
        assert_eq!(r.c1, b'x' as c_char);
    }

    #[test]
    fn test_makemap_mode_chars_select() {
        let r = rs_makemap_mode_chars(MODE_SELECT, 0);
        assert_eq!(r.c1, b's' as c_char);
    }

    #[test]
    fn test_makemap_mode_chars_insert_cmdline_abbr() {
        // For abbreviations, no bang
        let r = rs_makemap_mode_chars(MODE_CMDLINE | MODE_INSERT, 1);
        assert_eq!(r.c1, 0);
        assert_eq!(r.use_bang, 0);
    }

    #[test]
    fn test_makemap_mode_chars_insert_cmdline_map() {
        // For mappings, use bang
        let r = rs_makemap_mode_chars(MODE_CMDLINE | MODE_INSERT, 0);
        assert_eq!(r.c1, 0);
        assert_eq!(r.use_bang, 1);
    }

    #[test]
    fn test_makemap_mode_chars_normal_visual_op() {
        let r = rs_makemap_mode_chars(MODE_NORMAL | MODE_VISUAL | MODE_OP_PENDING, 0);
        assert_eq!(r.c1, b'n' as c_char);
        assert_eq!(r.c2, b'x' as c_char);
        assert_eq!(r.c3, b'o' as c_char);
    }

    #[test]
    fn test_makemap_mode_chars_cmdline() {
        let r = rs_makemap_mode_chars(MODE_CMDLINE, 0);
        assert_eq!(r.c1, b'c' as c_char);
    }

    #[test]
    fn test_makemap_mode_chars_langmap() {
        let r = rs_makemap_mode_chars(MODE_LANGMAP, 0);
        assert_eq!(r.c1, b'l' as c_char);
    }

    #[test]
    fn test_makemap_mode_chars_terminal() {
        let r = rs_makemap_mode_chars(MODE_TERMINAL, 0);
        assert_eq!(r.c1, b't' as c_char);
    }

    #[test]
    fn test_makemap_mode_chars_invalid() {
        let r = rs_makemap_mode_chars(0xFF, 0);
        assert_eq!(r.error, 1);
    }

    #[test]
    fn test_put_escstr_escape_type_normal_char() {
        assert_eq!(rs_put_escstr_escape_type(0, c_int::from(b'a'), 0), 0);
        assert_eq!(rs_put_escstr_escape_type(1, c_int::from(b'z'), 0), 0);
    }

    #[test]
    fn test_put_escstr_escape_type_ctrl_v() {
        // Control character
        assert_eq!(rs_put_escstr_escape_type(0, 1, 0), 2);
        // Pipe
        assert_eq!(rs_put_escstr_escape_type(0, c_int::from(b'|'), 0), 2);
        // Space in lhs
        assert_eq!(rs_put_escstr_escape_type(0, c_int::from(b' '), 0), 2);
        // Space at start of rhs
        assert_eq!(rs_put_escstr_escape_type(1, c_int::from(b' '), 1), 2);
        // '<' in map context
        assert_eq!(rs_put_escstr_escape_type(0, c_int::from(b'<'), 0), 2);
    }

    #[test]
    fn test_put_escstr_escape_type_backslash() {
        // :set mode
        assert_eq!(rs_put_escstr_escape_type(2, c_int::from(b' '), 0), 1);
        assert_eq!(rs_put_escstr_escape_type(2, c_int::from(b'"'), 0), 1);
        assert_eq!(rs_put_escstr_escape_type(2, c_int::from(b'\\'), 0), 1);
    }

    #[test]
    fn test_put_escstr_escape_space_not_first_rhs() {
        // Space in rhs but not first char — no escaping
        assert_eq!(rs_put_escstr_escape_type(1, c_int::from(b' '), 0), 0);
    }
}
