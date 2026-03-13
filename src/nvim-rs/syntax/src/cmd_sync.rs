//! Syntax sync command implementation.
//!
//! Migrated from `syn_cmd_sync` in syntax_accessors.c.
//! Handles parsing of `:syntax sync` commands.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{SynBlockHandle, SF_CCOMMENT};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // EAP accessors
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;
    fn nvim_syn_set_eap_arg(eap: *mut c_void, arg: *mut c_char);

    // String helpers
    fn nvim_syn_skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_skiptowhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_ends_excmd(c: c_int) -> c_int;
    fn nvim_syn_xfree(ptr: *mut c_void);
    fn nvim_syn_vim_strnsave_up(str: *const c_char, len: c_int) -> *mut c_char;

    // Number parsing (wraps getdigits_int32)
    fn nvim_syn_getdigits_int32(pp: *mut *mut c_char, strict: c_int, def: c_int) -> c_int;

    // Group resolution
    fn nvim_syn_name2id_wrapper(name: *const c_char) -> c_int;
    fn nvim_syn_check_group_wrapper(name: *const c_char, len: c_int) -> c_int;

    // Synblock sync field setters
    fn nvim_synblock_or_sync_flags(block: SynBlockHandle, flags: c_int);
    fn nvim_synblock_set_sync_id(block: SynBlockHandle, id: c_int);
    fn nvim_synblock_set_sync_minlines(block: SynBlockHandle, n: c_int);
    fn nvim_synblock_set_sync_maxlines(block: SynBlockHandle, n: c_int);
    fn nvim_synblock_set_sync_linebreaks(block: SynBlockHandle, n: c_int);
    fn nvim_synblock_get_linecont_pat_is_set(block: SynBlockHandle) -> c_int;

    // Accessors for linecont fields (replacing nvim_synblock_set_linecont)
    fn nvim_syn_xstrnsave(s: *const c_char, len: c_int) -> *mut c_char;
    fn nvim_synblock_get_syn_ic(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_set_linecont_pat(block: SynBlockHandle, pat: *mut c_char);
    fn nvim_synblock_get_linecont_pat(block: SynBlockHandle) -> *mut c_char;
    fn nvim_synblock_set_linecont_ic(block: SynBlockHandle, ic: c_int);
    fn nvim_synblock_set_linecont_prog2(block: SynBlockHandle, prog: *mut c_void);
    fn nvim_syn_clear_linecont_pat(block: SynBlockHandle);
    fn nvim_synblock_get_linecont_time_ptr(block: SynBlockHandle) -> *mut c_void;
    fn nvim_syn_vim_regcomp_empty_cpo(pat: *mut c_char, flags: c_int) -> *mut c_void;
    fn nvim_syn_do_clear_time(st: *mut c_void);

    // Current window synblock
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;

    // Redraw and state reset (Phase 4: decomposed from nvim_syn_redraw_and_free_all)
    fn nvim_syn_redraw_curbuf_later();
    #[link_name = "syn_stack_free_all"]
    fn nvim_syn_stack_free_all(block: SynBlockHandle);

    // nextcmd via check_nextcmd
    fn nvim_syn_set_nextcmd(eap: *mut c_void, rest: *mut c_char);

    // Listing (for empty arg case)
    fn rs_syn_cmd_list(eap: *mut c_void, syncing: c_int);

    // Error messages
    fn semsg(fmt: *const c_char, ...);
    fn emsg(msg: *const c_char);

    // Skip regexp
    fn nvim_syn_skip_regexp(arg: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char;
}

// MAXLNUM as used by fromstart -- same value as C MAXLNUM for linenr_T
const MAXLNUM: c_int = 0x7FFF_FFFF;

static EMSG_SYNC_LINECONT_TWICE: &[u8] =
    b"E403: syntax sync: line continuations pattern specified twice\0";
static EMSG_E404_ILLEGAL: &[u8] = b"E404: Illegal arguments: %s\0";
static STR_COMMENT: &[u8] = b"Comment\0";

// =============================================================================
// Helper replacing deleted C function
// =============================================================================

/// Store linecont pattern into synblock: allocate, compile regexp, clear time.
/// `pat_start` points to the pattern text (not the delimiter), `pat_len` bytes.
/// Returns 1 on success, 0 on regexp compile failure.
///
/// Replaces C `nvim_synblock_set_linecont`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
unsafe fn synblock_set_linecont(
    block: SynBlockHandle,
    pat_start: *const c_char,
    pat_len: c_int,
) -> c_int {
    // Save a copy of the pattern text.
    let pat = nvim_syn_xstrnsave(pat_start, pat_len);
    nvim_synblock_set_linecont_pat(block, pat);

    // Copy the block's ignore-case setting.
    let ic = nvim_synblock_get_syn_ic(block);
    nvim_synblock_set_linecont_ic(block, ic);

    // Compile the pattern with empty cpoptions (avoid 'l' flag side-effect).
    let prog = nvim_syn_vim_regcomp_empty_cpo(
        nvim_synblock_get_linecont_pat(block),
        1, // RE_MAGIC
    );

    // Zero out the timing info.
    let time_ptr = nvim_synblock_get_linecont_time_ptr(block);
    nvim_syn_do_clear_time(time_ptr);

    if prog.is_null() {
        // Compile failed: free the pattern string.
        nvim_syn_clear_linecont_pat(block);
        return 0;
    }

    nvim_synblock_set_linecont_prog2(block, prog);
    1
}

/// Rust implementation of syn_cmd_sync.
///
/// # Safety
/// Must be called from main thread during command execution.
unsafe fn syn_cmd_sync_impl(eap: *mut c_void, _syncing: c_int) {
    let sf_ccomment = SF_CCOMMENT;

    let mut arg_start = nvim_syn_get_eap_arg(eap);

    // No argument: list sync items
    if nvim_syn_ends_excmd(*arg_start as c_int) != 0 {
        rs_syn_cmd_list(eap, 1);
        return;
    }

    let block = nvim_syn_get_curwin_synblock();
    let skip = nvim_syn_get_eap_skip(eap);

    let mut illegal = false;
    let mut finished = false;
    let mut key: *mut c_char = std::ptr::null_mut();

    while nvim_syn_ends_excmd(*arg_start as c_int) == 0 {
        let arg_end = nvim_syn_skiptowhite(arg_start);
        let mut next_arg = nvim_syn_skipwhite(arg_end);
        nvim_syn_xfree(key.cast());
        key = nvim_syn_vim_strnsave_up(arg_start, arg_end.offset_from(arg_start) as c_int);

        let key_bytes = std::ffi::CStr::from_ptr(key).to_bytes();

        if key_bytes == b"CCOMMENT" {
            if skip == 0 {
                nvim_synblock_or_sync_flags(block, sf_ccomment);
            }
            if nvim_syn_ends_excmd(*next_arg as c_int) == 0 {
                let arg_end2 = nvim_syn_skiptowhite(next_arg);
                if skip == 0 {
                    let id = nvim_syn_check_group_wrapper(
                        next_arg,
                        arg_end2.offset_from(next_arg) as c_int,
                    );
                    nvim_synblock_set_sync_id(block, id);
                }
                next_arg = nvim_syn_skipwhite(arg_end2);
            } else if skip == 0 {
                let comment_id = nvim_syn_name2id_wrapper(STR_COMMENT.as_ptr().cast());
                nvim_synblock_set_sync_id(block, comment_id);
            }
        } else if key_bytes.starts_with(b"LINES")
            || key_bytes.starts_with(b"MINLINES")
            || key_bytes.starts_with(b"MAXLINES")
            || key_bytes.starts_with(b"LINEBREAKS")
        {
            // Compute the offset to the digit start within the key.
            // C code: if key[4]=='S' => offset 6 (LINES=digit)
            //         else if key[0]=='L' => offset 11 (LINEBREAKS=digit)
            //         else => offset 9 (MINLINES= or MAXLINES=digit)
            let digit_offset: usize = if key_bytes[4] == b'S' {
                6 // "LINES="
            } else if key_bytes[0] == b'L' {
                11 // "LINEBREAKS="
            } else {
                9 // "MINLINES=" or "MAXLINES="
            };

            // Validate: character before digit must be '='
            if digit_offset == 0 || digit_offset > key_bytes.len() {
                illegal = true;
                break;
            }
            if key_bytes[digit_offset - 1] != b'=' || !key_bytes[digit_offset].is_ascii_digit() {
                illegal = true;
                break;
            }

            let mut num_ptr = key.add(digit_offset);
            let n = nvim_syn_getdigits_int32(&mut num_ptr, 0, 0);

            if skip == 0 {
                // C: if (key[4] == 'B') linebreaks; else if (key[1] == 'A') maxlines; else minlines
                if key_bytes[4] == b'B' {
                    // LINEBREAKS
                    nvim_synblock_set_sync_linebreaks(block, n);
                } else if key_bytes[1] == b'A' {
                    // MAXLINES
                    nvim_synblock_set_sync_maxlines(block, n);
                } else {
                    // LINES or MINLINES
                    nvim_synblock_set_sync_minlines(block, n);
                }
            }
        } else if key_bytes == b"FROMSTART" {
            if skip == 0 {
                nvim_synblock_set_sync_minlines(block, MAXLNUM);
                nvim_synblock_set_sync_maxlines(block, 0);
            }
        } else if key_bytes == b"LINECONT" {
            if *next_arg == 0 {
                // missing pattern
                illegal = true;
                break;
            }
            if nvim_synblock_get_linecont_pat_is_set(block) != 0 {
                emsg(EMSG_SYNC_LINECONT_TWICE.as_ptr().cast());
                finished = true;
                break;
            }

            let pat_start = next_arg.add(1);

            // Find closing delimiter using C skip_regexp
            let delim = *next_arg as c_int;
            let arg_end2 = nvim_syn_skip_regexp(pat_start, delim, 1);

            if *arg_end2 != *next_arg {
                // end delimiter not found
                illegal = true;
                break;
            }

            if skip == 0 {
                let pat_len = arg_end2.offset_from(pat_start) as c_int;
                let ok = synblock_set_linecont(block, pat_start, pat_len);
                if ok == 0 {
                    finished = true;
                    break;
                }
            }
            next_arg = nvim_syn_skipwhite(arg_end2.add(1));
        } else {
            // MATCH, REGION, CLEAR, or illegal keyword
            nvim_syn_set_eap_arg(eap, next_arg);
            if key_bytes == b"MATCH" {
                crate::cmd_match::rs_syn_cmd_match(eap, 1);
            } else if key_bytes == b"REGION" {
                crate::cmd_region::rs_syn_cmd_region(eap, 1);
            } else if key_bytes == b"CLEAR" {
                crate::cmd_clear::rs_syn_cmd_clear(eap, 1);
            } else {
                illegal = true;
            }
            finished = true;
            break;
        }
        arg_start = next_arg;
    }

    nvim_syn_xfree(key.cast());

    if illegal {
        semsg(EMSG_E404_ILLEGAL.as_ptr().cast(), arg_start);
    } else if !finished {
        nvim_syn_set_nextcmd(eap, arg_start);
        // Phase 4: replaces nvim_syn_redraw_and_free_all
        nvim_syn_redraw_curbuf_later();
        nvim_syn_stack_free_all(nvim_syn_get_curwin_synblock());
    }
}

/// Entry point called from C thin wrapper.
///
/// # Safety
/// Must be called from main thread during `:syntax sync` command execution.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_sync(eap: *mut c_void, syncing: c_int) {
    syn_cmd_sync_impl(eap, syncing);
}
