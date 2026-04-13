//! Syntax pattern parsing.
//!
//! Migrated from `get_syn_pattern` in syntax_accessors.c.
//! Handles parsing a single pattern for :syntax match and :syntax region.

use std::ffi::{c_char, c_int, c_void};

use crate::synblock_struct::synblock_ref;
use crate::types::{SynBlockHandle, SynPatHandle, SPO_COUNT, SPO_LC_OFF, SPO_MS_OFF};

// spo_name_tab from C: "ms=", "me=", "hs=", "he=", "rs=", "re=", "lc="
// Each entry is exactly 3 bytes + NUL terminator.
static SPO_NAME_TAB: [&[u8; 4]; 7] = [
    b"ms=\0", b"me=\0", b"hs=\0", b"he=\0", b"rs=\0", b"re=\0", b"lc=\0",
];

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Pattern parsing
    fn nvim_syn_skip_regexp(arg: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char;
    fn nvim_syn_getdigits_int(pp: *mut *mut c_char, strict: c_int, def: c_int) -> c_int;

    // cpoptions save/restore
    fn nvim_syn_get_p_cpo() -> *mut c_char;
    fn nvim_syn_set_p_cpo(val: *mut c_char);
    fn nvim_syn_get_empty_string_option() -> *mut c_char;

    // (synpat_T setters/getters removed -- use direct repr(C) field access)

    // curwin accessor
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;

    // regexp
    fn vim_regcomp(pat: *mut c_char, flags: c_int) -> *mut c_void;

    // string helpers
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn ends_excmd(c: c_int) -> c_int;
    fn nvim_syn_ascii_iswhite_char(c: c_int) -> c_int;
    fn xstrnsave(s: *const c_char, len: c_int) -> *mut c_char;
    fn nvim_syn_semsg_1s(fmt: *const c_char, arg: *const c_char);
}

/// RE_MAGIC flag for vim_regcomp.
const RE_MAGIC: c_int = 1;

/// Compare first 3 chars of `s` (C string pointer) against a 3-byte prefix.
///
/// # Safety
/// `s` must point to at least 3 readable bytes.
unsafe fn starts_with_spo(s: *const c_char, name: &[u8; 4]) -> bool {
    let b0 = *s as u8;
    let b1 = *s.add(1) as u8;
    let b2 = *s.add(2) as u8;
    b0 == name[0] && b1 == name[1] && b2 == name[2]
}

/// Rust implementation of get_syn_pattern.
///
/// Parses a single syntax pattern from `arg`, stores the pattern, compiled
/// regexp, ignore-case flag, and offset specifiers into `ci`.
///
/// Returns a pointer to the next argument on success, or NULL on error.
///
/// # Safety
/// `arg` must be a valid C string (or NULL). `ci` must be a valid synpat_T
/// handle returned by `nvim_syn_compile_pattern` or similar allocator.
unsafe fn get_syn_pattern_impl(arg: *mut c_char, ci: SynPatHandle) -> *mut c_char {
    if arg.is_null() || *arg == 0 || *arg.add(1) == 0 || *arg.add(2) == 0 {
        return std::ptr::null_mut();
    }

    let delim = *arg as c_int;
    let end = nvim_syn_skip_regexp(arg.add(1), delim, 1);

    // end delimiter not found
    if *end as c_int != delim {
        nvim_syn_semsg_1s(c"E401: Pattern delimiter not found: %s".as_ptr(), arg);
        return std::ptr::null_mut();
    }

    // Store the pattern (xstrnsave(arg+1, end - arg - 1))
    let pat_len = end.offset_from(arg) as c_int - 1;
    let pattern = xstrnsave(arg.add(1), pat_len);
    (*ci.as_ptr()).sp_pattern = pattern;

    // Make 'cpoptions' empty to avoid the 'l' flag, then compile
    let cpo_save = nvim_syn_get_p_cpo();
    nvim_syn_set_p_cpo(nvim_syn_get_empty_string_option());
    let prog = vim_regcomp(pattern, RE_MAGIC);
    nvim_syn_set_p_cpo(cpo_save);

    if prog.is_null() {
        return std::ptr::null_mut();
    }

    (*ci.as_ptr()).sp_prog = prog;
    let cw_block = nvim_syn_get_curwin_synblock();
    (*ci.as_ptr()).sp_ic = if cw_block.is_null() {
        0
    } else {
        synblock_ref(cw_block).b_syn_ic
    };
    let st_ptr = &mut (*ci.as_ptr()).sp_time as *mut _ as *mut c_void;
    crate::line_init::rs_syn_clear_time(st_ptr);

    // Check for match/highlight/region offset specifiers after the closing delimiter.
    let mut cur = end.add(1);

    loop {
        // Find which offset name matches at `cur`
        let mut idx: c_int = -1;
        for i in (0..SPO_COUNT as usize).rev() {
            if starts_with_spo(cur, SPO_NAME_TAB[i]) {
                idx = i as c_int;
                break;
            }
        }

        if idx < 0 {
            break;
        }

        // For all except lc=, we also look at end[3] for 's'/'b'/'e'
        let mut eff_idx = idx;
        if idx != SPO_LC_OFF {
            let c4 = *cur.add(3) as u8;
            match c4 {
                b's' | b'b' => {
                    // start-of-match variant (default)
                }
                b'e' => {
                    eff_idx += SPO_COUNT;
                }
                _ => {
                    // unknown variant, stop
                    break;
                }
            }
        }

        // Set the flag bit
        let old_flags = (*ci.as_ptr()).sp_off_flags;
        (*ci.as_ptr()).sp_off_flags = old_flags | (1i16 << eff_idx);

        if idx == SPO_LC_OFF {
            // lc= -- advance past 3-char name, read decimal
            cur = cur.add(3);
            let val = nvim_syn_getdigits_int(&mut cur, 1, 0);
            (*ci.as_ptr()).sp_offsets[idx as usize] = val;

            // lc= automatically sets ms= if not already set
            let flags_now = (*ci.as_ptr()).sp_off_flags;
            if flags_now & (1i16 << SPO_MS_OFF) == 0 {
                (*ci.as_ptr()).sp_off_flags = flags_now | (1i16 << SPO_MS_OFF);
                (*ci.as_ptr()).sp_offsets[SPO_MS_OFF as usize] = val;
            }
        } else {
            // yy=x+99 or yy=x-99 -- advance past 4 chars (name + s/b/e char)
            cur = cur.add(4);
            let val = if *cur as u8 == b'+' {
                cur = cur.add(1);
                nvim_syn_getdigits_int(&mut cur, 1, 0)
            } else if *cur as u8 == b'-' {
                cur = cur.add(1);
                -nvim_syn_getdigits_int(&mut cur, 1, 0)
            } else {
                0
            };
            (*ci.as_ptr()).sp_offsets[idx as usize] = val;
        }

        if *cur as u8 != b',' {
            break;
        }
        cur = cur.add(1);
    }

    // Trailing garbage check
    if ends_excmd(*cur as c_int) == 0 && nvim_syn_ascii_iswhite_char(*cur as c_int) == 0 {
        nvim_syn_semsg_1s(c"E402: Garbage after pattern: %s".as_ptr(), arg);
        return std::ptr::null_mut();
    }

    skipwhite(cur)
}

// =============================================================================
// FFI exports
// =============================================================================

/// Rust implementation of get_syn_pattern.
///
/// Parses one syntax pattern from `arg` and stores the result into `ci`
/// (an opaque synpat_T handle). Returns a pointer to the next argument,
/// or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syn_pattern(arg: *mut c_char, ci: SynPatHandle) -> *mut c_char {
    get_syn_pattern_impl(arg, ci)
}
