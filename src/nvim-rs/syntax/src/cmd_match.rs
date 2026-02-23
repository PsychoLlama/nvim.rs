//! Syntax match command implementation.
//!
//! Migrated from `syn_cmd_match` in syntax_accessors.c.
//! Handles parsing of `:syntax match` commands.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{SynPatHandle, HL_EXCLUDENL, HL_HAS_EOL};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Command argument access
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;
    fn nvim_syn_set_nextcmd(eap: *mut c_void, rest: *mut c_char);

    // Group name parsing
    fn nvim_syn_get_group_name(arg: *mut c_char, name_end: *mut *mut c_char) -> *mut c_char;

    // Pattern initialization and compilation
    fn nvim_syn_init_patterns();
    fn nvim_syn_compile_pattern(
        arg: *mut c_char,
        item_type: c_int,
        opt_flags: c_int,
        rest_out: *mut *mut c_char,
    ) -> SynPatHandle;
    fn nvim_syn_free_compiled_pattern(pat: SynPatHandle);
    fn nvim_syn_vim_regcomp_had_eol() -> c_int;

    // Option parsing (Rust)
    fn rs_get_syn_options(
        arg: *mut c_char,
        flagsp: *mut c_int,
        keyword: c_int,
        sync_idx: *mut c_int,
        has_cont_list: c_int,
        cont_list: *mut *mut i16,
        cont_in_list: *mut *mut i16,
        next_list: *mut *mut i16,
        conceal_char: *mut c_int,
        skip: c_int,
    ) -> *mut c_char;

    // Group checking
    fn nvim_syn_check_group_wrapper(name: *const c_char, len: c_int) -> c_int;
    fn nvim_syn_incl_toplevel(id: c_int, flagsp: *mut c_int);

    // Pattern storage
    fn nvim_syn_store_match_pattern(
        pat: SynPatHandle,
        flags: c_int,
        syn_id: c_int,
        sync_idx: c_int,
        conceal_char: c_int,
        cont_list: *mut i16,
        cont_in_list: *mut i16,
        next_list: *mut i16,
        syncing: c_int,
    );

    // Helpers
    fn nvim_syn_ends_excmd(c: c_int) -> c_int;
    fn nvim_syn_semsg_1s(fmt: *const c_char, arg: *const c_char);
    fn nvim_syn_xfree(ptr: *mut c_void);
}

// ITEM_MATCHGROUP=3 for match patterns (no START/END/SKIP context here, but
// nvim_syn_compile_pattern needs item_type=0 for MATCH which uses REX_SET)
// For a plain match pattern there is no extmatch context; use item_type=0
// (ITEM_START) so reg_do_extmatch = REX_SET, matching the C behavior.
const ITEM_START: c_int = 0;

/// Rust implementation of syn_cmd_match.
unsafe fn syn_cmd_match_impl(eap: *mut c_void, syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    let skip = nvim_syn_get_eap_skip(eap);

    // Isolate the group name
    let mut group_name_end: *mut c_char = std::ptr::null_mut();
    let mut rest = nvim_syn_get_group_name(arg, &mut group_name_end);

    // Initialize option parsing fields
    let mut opt_flags: c_int = 0;
    let mut sync_idx: c_int = 0;
    let sync_idx_ptr: *mut c_int = if syncing != 0 {
        &mut sync_idx
    } else {
        std::ptr::null_mut()
    };
    let mut cont_list: *mut i16 = std::ptr::null_mut();
    let mut cont_in_list: *mut i16 = std::ptr::null_mut();
    let mut next_list: *mut i16 = std::ptr::null_mut();
    let mut conceal_char: c_int = 0;

    // Get options before the pattern
    rest = rs_get_syn_options(
        rest,
        &mut opt_flags,
        0, // keyword = false
        sync_idx_ptr,
        1, // has_cont_list = true
        &mut cont_list,
        &mut cont_in_list,
        &mut next_list,
        &mut conceal_char,
        skip,
    );

    // Get the pattern
    nvim_syn_init_patterns();
    let mut pattern_rest: *mut c_char = std::ptr::null_mut();
    let pat = nvim_syn_compile_pattern(rest, ITEM_START, opt_flags, &mut pattern_rest);

    if !pat.is_null() && nvim_syn_vim_regcomp_had_eol() != 0 && (opt_flags & HL_EXCLUDENL) == 0 {
        opt_flags |= HL_HAS_EOL;
    }

    rest = pattern_rest;

    // Get options after the pattern
    if !rest.is_null() {
        rest = rs_get_syn_options(
            rest,
            &mut opt_flags,
            0, // keyword = false
            sync_idx_ptr,
            1, // has_cont_list = true
            &mut cont_list,
            &mut cont_in_list,
            &mut next_list,
            &mut conceal_char,
            skip,
        );
    }

    let mut success = false;

    if !rest.is_null() && !pat.is_null() {
        // Check for trailing command and illegal trailing arguments
        nvim_syn_set_nextcmd(eap, rest);
        if nvim_syn_ends_excmd(*rest as c_int) != 0 && skip == 0 {
            let syn_id =
                nvim_syn_check_group_wrapper(arg, group_name_end.offset_from(arg) as c_int);
            if syn_id != 0 {
                nvim_syn_incl_toplevel(syn_id, &mut opt_flags);

                nvim_syn_store_match_pattern(
                    pat,
                    opt_flags,
                    syn_id,
                    sync_idx,
                    conceal_char,
                    cont_list,
                    cont_in_list,
                    next_list,
                    syncing,
                );
                success = true;
            }
        }
    }

    if !success {
        // Free allocated resources on failure
        if !pat.is_null() {
            nvim_syn_free_compiled_pattern(pat);
        }
        nvim_syn_xfree(cont_list as *mut c_void);
        nvim_syn_xfree(cont_in_list as *mut c_void);
        nvim_syn_xfree(next_list as *mut c_void);
        if rest.is_null() {
            nvim_syn_semsg_1s(c"E475: Invalid argument: %s".as_ptr(), arg);
        }
    }
}

// =============================================================================
// FFI exports
// =============================================================================

/// Rust implementation of syn_cmd_match.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_match(eap: *mut c_void, syncing: c_int) {
    syn_cmd_match_impl(eap, syncing);
}
