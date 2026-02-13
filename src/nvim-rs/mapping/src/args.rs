//! Map argument parsing for `:map` commands.
//!
//! Parses `:map` command strings into `MapArguments`, handling flags like
//! `<buffer>`, `<silent>`, `<expr>`, etc., and splitting LHS/RHS.

#![allow(clippy::similar_names)] // orig_lhs_len / orig_rhs_len are from C API

use std::ffi::{c_char, c_int};

use crate::MAXMAPLEN;

// =============================================================================
// Constants
// =============================================================================

const LUA_NOREF: c_int = -2;
const REPTERM_FROM_PART: c_int = 1;
const REPTERM_DO_LT: c_int = 2;
const REPTERM_NO_SIMPLIFY: c_int = 8;
const K_SPECIAL: u8 = 0x80;
const KS_EXTRA: u8 = 253;
const KE_LUA: u8 = 103;
const CPO_BSLASH: u8 = b'B';
const CTRL_V: u8 = 0x16;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn rs_skipwhite(p: *const c_char) -> *const c_char;
    fn rs_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn nvim_mapping_get_p_cpo() -> *const c_char;
    fn replace_termcodes(
        from: *const c_char,
        from_len: usize,
        bufp: *mut *mut c_char,
        sid_arg: c_int,
        flags: c_int,
        did_simplify: *mut bool,
        cpo_val: *const c_char,
    ) -> *mut c_char;
    fn xcalloc(count: usize, size: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmemcpyz(to: *mut c_char, from: *const c_char, len: usize);
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn vim_snprintf(buf: *mut c_char, len: usize, fmt: *const c_char, ...) -> c_int;
}

// =============================================================================
// MapArguments struct
// =============================================================================

/// All possible `:map` arguments parsed from a command string.
///
/// Must match the C `struct map_arguments` layout exactly.
#[repr(C)]
pub struct MapArguments {
    pub buffer: bool,
    pub expr: bool,
    pub noremap: bool,
    pub nowait: bool,
    pub script: bool,
    pub silent: bool,
    pub unique: bool,
    pub replace_keycodes: bool,

    /// The {lhs} of the mapping (static buffer).
    pub lhs: [c_char; MAXMAPLEN + 1],
    pub lhs_len: usize,

    /// Unsimplified {lhs}. If no simplification, alt_lhs_len is 0.
    pub alt_lhs: [c_char; MAXMAPLEN + 1],
    pub alt_lhs_len: usize,

    /// The {rhs} of the mapping (allocated).
    pub rhs: *mut c_char,
    pub rhs_len: usize,
    pub rhs_lua: c_int, // LuaRef
    pub rhs_is_noop: bool,

    /// The original text of the {rhs} (allocated).
    pub orig_rhs: *mut c_char,
    pub orig_rhs_len: usize,
    /// Map description (allocated).
    pub desc: *mut c_char,
}

// =============================================================================
// set_maparg_rhs
// =============================================================================

/// Process the RHS of a mapping. Handles Lua refs, <Nop>, and replace_termcodes.
///
/// # Safety
/// All pointer parameters must be valid. Memory is allocated via C allocator.
unsafe fn set_maparg_rhs(
    orig_rhs: *const c_char,
    orig_rhs_len: usize,
    rhs_lua: c_int,
    sid: c_int,
    cpo_val: *const c_char,
    mapargs: *mut MapArguments,
) {
    (*mapargs).rhs_lua = rhs_lua;

    if rhs_lua == LUA_NOREF {
        (*mapargs).orig_rhs_len = orig_rhs_len;
        (*mapargs).orig_rhs = xcalloc(orig_rhs_len + 1, 1);
        xmemcpyz((*mapargs).orig_rhs, orig_rhs, orig_rhs_len);

        if libc::strcasecmp(orig_rhs, c"<nop>".as_ptr()) == 0 {
            // "<Nop>" means nothing
            (*mapargs).rhs = xcalloc(1, 1); // single NUL-char
            (*mapargs).rhs_len = 0;
            (*mapargs).rhs_is_noop = true;
        } else {
            let mut rhs_buf: *mut c_char = std::ptr::null_mut();
            let replaced = replace_termcodes(
                orig_rhs,
                orig_rhs_len,
                std::ptr::addr_of_mut!(rhs_buf),
                sid,
                REPTERM_DO_LT,
                std::ptr::null_mut(),
                cpo_val,
            );
            (*mapargs).rhs_len = libc::strlen(replaced.cast());
            // NB: replace_termcodes may produce empty string even if orig_rhs is non-empty
            (*mapargs).rhs_is_noop = orig_rhs_len != 0 && (*mapargs).rhs_len == 0;
            (*mapargs).rhs = replaced;
        }
    } else {
        let mut tmp_buf = [0u8; 64];
        // orig_rhs is not used for Lua mappings, but still needs to be a string.
        (*mapargs).orig_rhs = xcalloc(1, 1);
        (*mapargs).orig_rhs_len = 0;
        // stores <lua>ref_no<cr> in map_str
        (*mapargs).rhs_len = vim_snprintf(
            tmp_buf.as_mut_ptr().cast::<c_char>(),
            tmp_buf.len(),
            c"%c%c%c%d\r".as_ptr(),
            c_int::from(K_SPECIAL),
            c_int::from(KS_EXTRA),
            c_int::from(KE_LUA),
            rhs_lua,
        ) as usize;
        (*mapargs).rhs = xstrdup(tmp_buf.as_ptr().cast::<c_char>());
    }
}

// =============================================================================
// set_maparg_lhs_rhs
// =============================================================================

/// Replace termcodes in the given LHS and RHS and store results into mapargs.
///
/// # Safety
/// All pointer parameters must be valid. Memory is allocated via C allocator.
unsafe fn set_maparg_lhs_rhs(
    orig_lhs: *const c_char,
    orig_lhs_len: usize,
    orig_rhs: *const c_char,
    orig_rhs_len: usize,
    rhs_lua: c_int,
    cpo_val: *const c_char,
    mapargs: *mut MapArguments,
) -> bool {
    let mut lhs_buf = [0u8; 128];

    let mut did_simplify = false;
    let flags = REPTERM_FROM_PART | REPTERM_DO_LT;
    let mut bufarg = lhs_buf.as_mut_ptr().cast::<c_char>();
    let replaced = replace_termcodes(
        orig_lhs,
        orig_lhs_len,
        std::ptr::addr_of_mut!(bufarg),
        0,
        flags,
        std::ptr::addr_of_mut!(did_simplify),
        cpo_val,
    );
    if replaced.is_null() {
        return false;
    }
    (*mapargs).lhs_len = libc::strlen(replaced.cast());
    xstrlcpy(
        (*mapargs).lhs.as_mut_ptr(),
        replaced,
        std::mem::size_of_val(&(*mapargs).lhs),
    );

    if did_simplify {
        let replaced = replace_termcodes(
            orig_lhs,
            orig_lhs_len,
            std::ptr::addr_of_mut!(bufarg),
            0,
            flags | REPTERM_NO_SIMPLIFY,
            std::ptr::null_mut(),
            cpo_val,
        );
        if replaced.is_null() {
            return false;
        }
        (*mapargs).alt_lhs_len = libc::strlen(replaced.cast());
        xstrlcpy(
            (*mapargs).alt_lhs.as_mut_ptr(),
            replaced,
            std::mem::size_of_val(&(*mapargs).alt_lhs),
        );
    } else {
        (*mapargs).alt_lhs_len = 0;
    }

    set_maparg_rhs(orig_rhs, orig_rhs_len, rhs_lua, 0, cpo_val, mapargs);

    true
}

// =============================================================================
// str_to_mapargs
// =============================================================================

/// Parse a string of `:map` arguments into a `MapArguments` struct.
///
/// Handles flag parsing (`<buffer>`, `<silent>`, etc.), LHS/RHS splitting,
/// and termcode replacement.
///
/// Returns 0 on success, 1 if invalid arguments are detected.
///
/// # Safety
/// `strargs` must be a valid NUL-terminated C string.
/// `mapargs` must point to valid writable memory.
#[no_mangle]
pub unsafe extern "C" fn rs_str_to_mapargs(
    strargs: *const c_char,
    is_unmap: c_int,
    mapargs: *mut MapArguments,
) -> c_int {
    let is_unmap = is_unmap != 0;

    // Zero-initialize
    std::ptr::write_bytes(mapargs, 0, 1);
    (*mapargs).rhs_lua = LUA_NOREF;

    let mut to_parse = rs_skipwhite(strargs);

    // Accept <buffer>, <nowait>, <silent>, <expr>, <script>, and <unique> in any order.
    loop {
        if libc::strncmp(to_parse, c"<buffer>".as_ptr(), 8) == 0 {
            to_parse = rs_skipwhite(to_parse.add(8));
            (*mapargs).buffer = true;
            continue;
        }
        if libc::strncmp(to_parse, c"<nowait>".as_ptr(), 8) == 0 {
            to_parse = rs_skipwhite(to_parse.add(8));
            (*mapargs).nowait = true;
            continue;
        }
        if libc::strncmp(to_parse, c"<silent>".as_ptr(), 8) == 0 {
            to_parse = rs_skipwhite(to_parse.add(8));
            (*mapargs).silent = true;
            continue;
        }
        // Ignore obsolete "<special>" modifier.
        if libc::strncmp(to_parse, c"<special>".as_ptr(), 9) == 0 {
            to_parse = rs_skipwhite(to_parse.add(9));
            continue;
        }
        if libc::strncmp(to_parse, c"<script>".as_ptr(), 8) == 0 {
            to_parse = rs_skipwhite(to_parse.add(8));
            (*mapargs).script = true;
            continue;
        }
        if libc::strncmp(to_parse, c"<expr>".as_ptr(), 6) == 0 {
            to_parse = rs_skipwhite(to_parse.add(6));
            (*mapargs).expr = true;
            continue;
        }
        if libc::strncmp(to_parse, c"<unique>".as_ptr(), 8) == 0 {
            to_parse = rs_skipwhite(to_parse.add(8));
            (*mapargs).unique = true;
            continue;
        }
        break;
    }

    // Find the end of {lhs}
    let mut lhs_end = to_parse;
    let p_cpo = nvim_mapping_get_p_cpo();
    let do_backslash = rs_vim_strchr(p_cpo, c_int::from(CPO_BSLASH)).is_null();

    while *lhs_end != 0 && (is_unmap || !ascii_iswhite(*lhs_end as u8)) {
        if (*lhs_end as u8 == CTRL_V || (do_backslash && *lhs_end as u8 == b'\\'))
            && *lhs_end.add(1) != 0
        {
            lhs_end = lhs_end.add(1); // skip CTRL-V or backslash
        }
        lhs_end = lhs_end.add(1);
    }

    let rhs_start = rs_skipwhite(lhs_end);

    // Given {lhs} might be larger than MAXMAPLEN before replace_termcodes
    let orig_lhs_len = lhs_end as usize - to_parse as usize;
    if orig_lhs_len >= 256 {
        return 1;
    }
    let mut lhs_to_replace = [0u8; 256];
    xmemcpyz(
        lhs_to_replace.as_mut_ptr().cast::<c_char>(),
        to_parse,
        orig_lhs_len,
    );

    let orig_rhs_len = libc::strlen(rhs_start.cast());

    if !set_maparg_lhs_rhs(
        lhs_to_replace.as_ptr().cast::<c_char>(),
        orig_lhs_len,
        rhs_start,
        orig_rhs_len,
        LUA_NOREF,
        p_cpo,
        mapargs,
    ) {
        return 1;
    }

    if (*mapargs).lhs_len > MAXMAPLEN {
        return 1;
    }
    0
}

/// FFI export for set_maparg_lhs_rhs (used by C callers like f_mapset).
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_set_maparg_lhs_rhs(
    orig_lhs: *const c_char,
    orig_lhs_len: usize,
    orig_rhs: *const c_char,
    orig_rhs_len: usize,
    rhs_lua: c_int,
    cpo_val: *const c_char,
    mapargs: *mut MapArguments,
) -> c_int {
    c_int::from(set_maparg_lhs_rhs(
        orig_lhs,
        orig_lhs_len,
        orig_rhs,
        orig_rhs_len,
        rhs_lua,
        cpo_val,
        mapargs,
    ))
}

/// FFI export for set_maparg_rhs (used by C callers like f_mapset).
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_set_maparg_rhs(
    orig_rhs: *const c_char,
    orig_rhs_len: usize,
    rhs_lua: c_int,
    sid: c_int,
    cpo_val: *const c_char,
    mapargs: *mut MapArguments,
) {
    set_maparg_rhs(orig_rhs, orig_rhs_len, rhs_lua, sid, cpo_val, mapargs);
}

// =============================================================================
// Helpers
// =============================================================================

#[inline]
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}
