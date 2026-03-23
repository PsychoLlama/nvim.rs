//! Phase 1 and Phase 2 string validation callbacks
//!
//! This module contains Rust implementations of `did_set_*` string validation
//! callbacks migrated from `optionstr.c`. These callbacks validate option
//! string values against allowed character sets or fixed value lists.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

use std::ffi::{c_char, c_int, c_uint, c_void};

use super::{callback_ok, CallbackResult};

// Result type matching OptStringsFlagsResult from optionstr crate (must match C layout)
#[repr(C)]
#[derive(Clone, Copy)]
struct OptStringsFlagsResult {
    ok: bool,
    flags: u32,
}

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // optset_T field accessors
    fn nvim_optset_get_varp_str(args: *const c_void) -> *const c_char;
    fn nvim_optset_get_errbuf(args: *const c_void) -> *mut c_char;
    fn nvim_optset_get_errbuflen(args: *const c_void) -> usize;
    fn nvim_optset_get_win(args: *const c_void) -> crate::WinHandle;
    fn nvim_optset_get_varp_ptr(args: *const c_void) -> *const c_void;
    fn nvim_optset_get_flags(args: *const c_void) -> c_int;
    fn nvim_optset_get_buf(args: *const c_void) -> crate::BufHandle;
    fn nvim_optset_get_newval_str(args: *const c_void) -> *const c_char;
    fn nvim_optset_get_oldval_str(args: *const c_void) -> *const c_char;

    // Validation helpers
    fn rs_illegal_char(errbuf: *mut c_char, errbuflen: usize, c: c_int) -> *const c_char;
    fn did_set_str_generic(args: *mut c_void) -> *const c_char;
    fn rs_opt_strings_flags(
        val: *const c_char,
        values: *const *const c_char,
        is_list: bool,
    ) -> OptStringsFlagsResult;
    fn rs_skip_to_option_part(p: *const c_char) -> *const c_char;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;

    // Side-effect helpers
    fn init_chartab();
    fn msg_grid_validate();
    fn check_opt_wim() -> c_int;
    fn nvim_call_briopt_check_win(val: *const c_char, win: crate::WinHandle) -> c_int;
    fn nvim_win_get_p_briopt_addr(win: crate::WinHandle) -> *const c_void;
    static mut cmdpreview: bool;
    static mut VIsual_active: bool;
    fn nvim_redraw_curbuf_later(redraw_type: c_int);
    fn nvim_win_get_briopt_list(win: crate::WinHandle) -> c_int;
    fn redraw_all_later(typ: c_int);

    // Phase 2 accessors
    fn nvim_buf_set_bkc_flags(buf: crate::BufHandle, val: c_uint);
    fn nvim_buf_get_p_bkc(buf: crate::BufHandle) -> *const c_char;
    static mut p_bkc: *mut c_char;
    static mut bkc_flags: c_uint;
    static mut ssop_flags: c_uint;
    static mut spo_flags: c_uint;
    #[allow(dead_code)]
    fn nvim_win_get_spo_flags(win: crate::WinHandle) -> c_uint;
    fn nvim_win_set_spo_flags(win: crate::WinHandle, val: c_uint);
    fn rs_diffanchors_changed(buflocal: bool) -> c_int;
    fn nvim_get_opt_bkc_values() -> *const *const c_char;
    fn nvim_get_opt_ssop_values() -> *const *const c_char;
    fn nvim_get_opt_spo_values() -> *const *const c_char;
}

// =============================================================================
// Constants
// =============================================================================

/// Error: Invalid argument (E474)
const E_INVARG: *const c_char = c"E474: Invalid argument".as_ptr();

/// FAIL return value (matches C FAIL = 0)
const FAIL: c_int = 0;

/// UPD_INVERTED = 20 (from drawscreen.h)
const UPD_INVERTED: c_int = 20;
/// UPD_NOT_VALID = 40 (from drawscreen.h)
const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// Flag character sets for list-flag options
// =============================================================================

/// All valid flags for 'cpoptions' option (vi compatibility)
const CPO_VI: &[u8] = b"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_";

/// All valid flags for 'formatoptions' option
const FO_ALL: &[u8] = b"tcro/q2vlb1mMBn,aw]jp";

/// All valid flags for 'mouse' option
const MOUSE_ALL: &[u8] = b"anvichr";

/// All valid flags for 'shortmess' option
const SHM_ALL: &[u8] = b"rwoOstTWIcCqaAFnlxfiS";

/// All valid flags for 'whichwrap' option (comma is also allowed as separator)
const WW_ALL: &[u8] = b"bshl<>[]~,";

// =============================================================================
// Internal helpers
// =============================================================================

/// Validate that a string contains only characters from an allowed set.
/// On failure, formats an E539 "Illegal character" message into errbuf.
/// Returns NULL on success, or a pointer to an error message on failure.
#[inline]
unsafe fn validate_listflag_with_errbuf(
    val: *const c_char,
    allowed: &[u8],
    errbuf: *mut c_char,
    errbuflen: usize,
) -> CallbackResult {
    if val.is_null() {
        return callback_ok();
    }
    let mut p = val;
    while *p != 0 {
        let ch = *p as u8;
        if !allowed.contains(&ch) {
            return rs_illegal_char(errbuf, errbuflen, c_int::from(ch));
        }
        p = p.add(1);
    }
    callback_ok()
}

// =============================================================================
// List-flag option callbacks
// =============================================================================

/// Callback for 'cpoptions' option.
/// Validates that all characters are valid vi compatibility flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_cpoptions(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, CPO_VI, errbuf, errbuflen)
}

/// Callback for 'formatoptions' option.
/// Validates that all characters are valid format option flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_formatoptions(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, FO_ALL, errbuf, errbuflen)
}

/// Callback for 'mouse' option.
/// Validates that all characters are valid mouse mode flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_mouse(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, MOUSE_ALL, errbuf, errbuflen)
}

/// Callback for 'shortmess' option.
/// Validates that all characters are valid shortmess flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_shortmess(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, SHM_ALL, errbuf, errbuflen)
}

/// Callback for 'whichwrap' option.
/// Validates characters in "bshl<>[]~" (comma-separated list).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_whichwrap(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, WW_ALL, errbuf, errbuflen)
}

// =============================================================================
// String-generic option callbacks (delegate to did_set_str_generic)
// =============================================================================

/// Callback for 'backspace' option.
/// Numeric form only allows "2". Otherwise validates against the allowed values list.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_backspace(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    if val.is_null() {
        return callback_ok();
    }
    let first = *val as u8;
    if first.is_ascii_digit() {
        if first != b'2' {
            return E_INVARG;
        }
        return callback_ok();
    }
    did_set_str_generic(args)
}

/// Callback for 'bufhidden' option.
/// Validates against "hide,unload,delete,wipe" and empty string.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_bufhidden(args: *mut c_void) -> CallbackResult {
    did_set_str_generic(args)
}

/// Callback for 'inccommand' option.
/// Disallows change while cmdpreview is active, then validates against allowed values.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_inccommand(args: *mut c_void) -> CallbackResult {
    if cmdpreview {
        return E_INVARG;
    }
    did_set_str_generic(args)
}

/// Callback for 'lispoptions' option.
/// Valid values: empty, "expr:0", or "expr:1".
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_lispoptions(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    if val.is_null() || *val == 0 {
        return callback_ok();
    }
    // Must be exactly "expr:0" or "expr:1" (6 bytes + null = 7 bytes total)
    let bytes = std::slice::from_raw_parts(val.cast::<u8>(), 7);
    let is_valid = bytes[0] == b'e'
        && bytes[1] == b'x'
        && bytes[2] == b'p'
        && bytes[3] == b'r'
        && bytes[4] == b':'
        && (bytes[5] == b'0' || bytes[5] == b'1')
        && bytes[6] == 0;
    if is_valid {
        callback_ok()
    } else {
        E_INVARG
    }
}

// =============================================================================
// Callbacks with side effects
// =============================================================================

/// Callback for 'wildmode' option.
/// Validates via check_opt_wim().
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_wildmode(args: *mut c_void) -> CallbackResult {
    let _ = args;
    if check_opt_wim() == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'breakindentopt' option.
/// Validates via briopt_check; applies to window if window-local, triggers redraw if list.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_breakindentopt(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let varp = nvim_optset_get_varp_ptr(args);
    let briopt_addr = nvim_win_get_p_briopt_addr(win);
    // Pass win only when varp IS the window-local option (mirrors C behavior)
    let win_for_check = if varp == briopt_addr {
        win
    } else {
        std::ptr::null_mut()
    };
    let val = nvim_optset_get_varp_str(args);
    if nvim_call_briopt_check_win(val, win_for_check) == 0 {
        return E_INVARG;
    }
    // List setting requires a redraw when applied to current window
    if varp == briopt_addr && nvim_win_get_briopt_list(win) != 0 {
        redraw_all_later(UPD_NOT_VALID);
    }
    callback_ok()
}

/// Callback for 'display' option.
/// Validates against allowed values, then reinitializes chartab and msg grid.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_display(args: *mut c_void) -> CallbackResult {
    let errmsg = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    init_chartab();
    msg_grid_validate();
    callback_ok()
}

/// Callback for 'showcmdloc' option.
/// Validates against allowed values ("last", "statusline", "tabline"),
/// then recomputes column positions.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_showcmdloc(args: *mut c_void) -> CallbackResult {
    let errmsg = did_set_str_generic(args);
    if errmsg.is_null() {
        // nvim_comp_col is already declared in window_shim.c
        nvim_comp_col();
    }
    errmsg
}

/// Callback for 'selection' option.
/// Validates against allowed values ("old", "inclusive", "exclusive"),
/// then triggers a redraw if Visual mode is active.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_selection(args: *mut c_void) -> CallbackResult {
    let errmsg = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if VIsual_active {
        nvim_redraw_curbuf_later(UPD_INVERTED);
    }
    callback_ok()
}

// =============================================================================
// Helper: nvim_comp_col (declared in window_shim.c)
// =============================================================================

extern "C" {
    fn nvim_comp_col();
}

// =============================================================================
// Phase 2: Medium-Complexity String Callbacks
// =============================================================================

/// OPT_GLOBAL = 0x01 (from option.h OptionSetFlags)
const OPT_GLOBAL: c_int = 0x01;
/// OPT_LOCAL = 0x02 (from option.h OptionSetFlags)
const OPT_LOCAL: c_int = 0x02;

/// BKC flag constants (verified by _Static_assert in bufwrite.c)
const BKC_YES: c_uint = 0x01;
const BKC_AUTO: c_uint = 0x02;
const BKC_NO: c_uint = 0x04;

/// SSOP flag constants (verified by _Static_assert in ex_session.c)
const SSOP_CURDIR: c_uint = 0x1000;
const SSOP_SESDIR: c_uint = 0x800;

/// Valid flag characters for 'comments' option (COM_ALL from option_vars.h)
const COM_ALL: &[u8] = b"nbsmexflrO";

/// Callback for 'backupcopy' option.
/// Validates "yes,no,auto" flags with buffer-local support.
/// Exactly one of "yes", "no", "auto" must be present.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_backupcopy(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let oldval = nvim_optset_get_oldval_str(args);
    let opt_flags = nvim_optset_get_flags(args);
    let bkc_values = nvim_get_opt_bkc_values();

    // Determine which bkc string and flags to use: local or global
    let bkc = if opt_flags & OPT_LOCAL != 0 {
        nvim_buf_get_p_bkc(buf)
    } else {
        p_bkc.cast_const()
    };

    // When using :set (neither LOCAL nor GLOBAL), clear the local flags
    if opt_flags & OPT_LOCAL == 0 && opt_flags & OPT_GLOBAL == 0 {
        nvim_buf_set_bkc_flags(buf, 0);
    }

    if opt_flags & OPT_LOCAL != 0 && (bkc.is_null() || *bkc == 0) {
        // make the local value empty: use the global value
        nvim_buf_set_bkc_flags(buf, 0);
    } else {
        // Parse the flags value
        let result = rs_opt_strings_flags(bkc, bkc_values, true);
        if !result.ok {
            return E_INVARG;
        }
        let flags = result.flags;

        // Must have exactly one of "auto", "yes", "no"
        let count = u32::from(flags & BKC_AUTO != 0)
            + u32::from(flags & BKC_YES != 0)
            + u32::from(flags & BKC_NO != 0);
        if count != 1 {
            // Restore flags from old value
            let old_result = rs_opt_strings_flags(oldval, bkc_values, true);
            let restored = old_result.flags;
            if opt_flags & OPT_LOCAL != 0 {
                nvim_buf_set_bkc_flags(buf, restored);
            } else {
                bkc_flags = restored;
            }
            return E_INVARG;
        }

        if opt_flags & OPT_LOCAL != 0 {
            nvim_buf_set_bkc_flags(buf, flags);
        } else {
            bkc_flags = flags;
        }
    }

    callback_ok()
}

/// Callback for 'commentstring' option.
/// Validates that if the value is non-empty, it contains "%s".
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_commentstring(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    if val.is_null() || *val == 0 {
        return callback_ok();
    }
    // Check that "%s" appears in the value
    let mut p = val;
    while *p != 0 {
        if *p == b'%' as c_char && *p.add(1) == b's' as c_char {
            return callback_ok();
        }
        p = p.add(1);
    }
    c"E537: 'commentstring' must be empty or contain %s".as_ptr()
}

/// Callback for 'comments' option.
/// Validates that each comment leader uses only valid flag characters
/// from COM_ALL ("nbsmexflrO"), followed by a colon, then the leader text.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_comments(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);

    if val.is_null() {
        return callback_ok();
    }

    let mut s = val;
    while *s != 0 {
        // Validate flag characters (before the colon)
        while *s != 0 && *s != b':' as c_char {
            let ch = *s as u8;
            if !COM_ALL.contains(&ch) && !ch.is_ascii_digit() && ch != b'-' {
                return rs_illegal_char(errbuf, errbuflen, c_int::from(ch));
            }
            s = s.add(1);
        }
        if *s == 0 {
            return c"E524: Missing colon".as_ptr();
        }
        s = s.add(1); // skip the colon
        if *s == b',' as c_char || *s == 0 {
            return c"E525: Zero length string".as_ptr();
        }
        // Skip the leader text (handling backslash escapes)
        while *s != 0 && *s != b',' as c_char {
            if *s == b'\\' as c_char && *s.add(1) != 0 {
                s = s.add(1);
            }
            s = s.add(1);
        }
        s = rs_skip_to_option_part(s);
    }
    callback_ok()
}

/// Callback for 'matchpairs' option.
/// Validates that each entry is "X:Y" where X and Y are single (multibyte) chars.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_matchpairs(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    if val.is_null() {
        return callback_ok();
    }

    let mut p = val;
    while *p != 0 {
        // Skip over the first character (which may be multibyte)
        p = p.add(utfc_ptr2len(p) as usize);
        let x2 = if *p != 0 { c_int::from(*p as u8) } else { -1 };
        if *p != 0 {
            p = p.add(1); // skip the ':'
        }
        let x3 = if *p != 0 {
            let c = utf_ptr2char(p);
            p = p.add(utfc_ptr2len(p) as usize);
            c
        } else {
            -1
        };

        if x2 != c_int::from(b':') || x3 == -1 || (*p != 0 && *p != b',' as c_char) {
            return E_INVARG;
        }
        if *p == 0 {
            break;
        }
        // skip the comma
        p = p.add(1);
    }
    callback_ok()
}

/// Callback for 'sessionoptions' option.
/// Validates against the allowed values list, then checks that "sesdir" and
/// "curdir" are not both set.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_sessionoptions(args: *mut c_void) -> CallbackResult {
    // First validate via did_set_str_generic
    let errmsg = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    // Don't allow both "sesdir" and "curdir"
    let ssop = ssop_flags;
    if (ssop & SSOP_CURDIR != 0) && (ssop & SSOP_SESDIR != 0) {
        // Restore flags from old value
        let oldval = nvim_optset_get_oldval_str(args);
        let ssop_values = nvim_get_opt_ssop_values();
        let old_result = rs_opt_strings_flags(oldval, ssop_values, true);
        ssop_flags = old_result.flags;
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'spelloptions' option.
/// Validates against "camel,noplainbuffer" and sets `spo_flags` / window flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_spelloptions(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let opt_flags = nvim_optset_get_flags(args);
    let val = nvim_optset_get_newval_str(args);
    let spo_values = nvim_get_opt_spo_values();

    // Validate and set global flags (unless OPT_LOCAL)
    if opt_flags & OPT_LOCAL == 0 {
        let result = rs_opt_strings_flags(val, spo_values, true);
        if !result.ok {
            return E_INVARG;
        }
        spo_flags = result.flags;
    }
    // Validate and set window-local flags (unless OPT_GLOBAL)
    if opt_flags & OPT_GLOBAL == 0 {
        let result = rs_opt_strings_flags(val, spo_values, true);
        if !result.ok {
            return E_INVARG;
        }
        nvim_win_set_spo_flags(win, result.flags);
    }
    callback_ok()
}

/// Callback for 'diffanchors' option.
/// Delegates to the Rust diff module.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_diffanchors(args: *mut c_void) -> CallbackResult {
    let opt_flags = nvim_optset_get_flags(args);
    if rs_diffanchors_changed(opt_flags & OPT_LOCAL != 0) == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'messagesopt' option.
/// Delegates to the Rust message module's messagesopt_changed().
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_messagesopt(_args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn messagesopt_changed() -> c_int;
    }
    if messagesopt_changed() == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'diffopt' option.
/// Delegates to the Rust diff module's rs_diffopt_changed().
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_diffopt(_args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn rs_diffopt_changed() -> c_int;
    }
    if rs_diffopt_changed() == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'langmap' option.
/// Delegates to the Rust mapping module's rs_langmap_parse().
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_langmap(args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn rs_langmap_parse(
            langmap_str: *const c_char,
            errbuf: *mut c_char,
            errbuflen: usize,
        ) -> c_int;
    }
    let langmap = crate::p_langmap.cast_const();
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    if rs_langmap_parse(langmap, errbuf, errbuflen) != 0 {
        return errbuf;
    }
    callback_ok()
}

/// Callback for 'spellsuggest' option (Phase 98).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_spellsuggest(_args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn spell_check_sps() -> c_int;
    }
    if spell_check_sps() == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'mkspellmem' option (Phase 98).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_mkspellmem(_args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn spell_check_msm() -> c_int;
    }
    if spell_check_msm() == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'winborder' option (Phase 98).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_winborder(_args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn parse_border_opt(border_opt: *mut std::ffi::c_char) -> bool;
        static mut p_winborder: *mut std::ffi::c_char;
    }
    if !parse_border_opt(p_winborder) {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'pumborder' option (Phase 98).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_pumborder(_args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn parse_border_opt(border_opt: *mut std::ffi::c_char) -> bool;
        static mut p_pumborder: *mut std::ffi::c_char;
    }
    if !parse_border_opt(p_pumborder) {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'eventignore' option (Phase 97).
/// Validates event names in the value.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_eventignore(args: *mut c_void) -> CallbackResult {
    extern "C" {
        fn check_ei(val: *const c_char) -> c_int;
    }
    let varp_str = nvim_optset_get_varp_str(args);
    if check_ei(varp_str) == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Generic string option callback (Phase 107).
/// Validates the option value against the allowed values list via check_str_opt.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_str_generic(args: *mut c_void) -> CallbackResult {
    did_set_str_generic(args)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ww_all_contains_comma() {
        // WW_ALL must contain comma since 'whichwrap' is comma-separated
        assert!(WW_ALL.contains(&b','));
    }
}
