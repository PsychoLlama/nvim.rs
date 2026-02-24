//! Syntax region command implementation.
//!
//! Migrated from `syn_cmd_region` and `get_syn_pattern` in syntax.c.
//! Handles parsing of `:syntax region` commands.

use std::ffi::{c_char, c_int, c_void};

use crate::pattern_store;
use crate::types::SynPatHandle;

// Item types matching C #defines
const ITEM_START: c_int = 0;
const ITEM_SKIP: c_int = 1;
const ITEM_END: c_int = 2;
const ITEM_MATCHGROUP: c_int = 3;

extern "C" {
    // Phase 5 C wrappers
    fn nvim_syn_get_group_name(arg: *mut c_char, name_end: *mut *mut c_char) -> *mut c_char;
    fn nvim_syn_init_patterns();
    fn nvim_syn_vim_strnsave_up(str: *const c_char, len: c_int) -> *mut c_char;
    fn nvim_syn_set_nextcmd(eap: *mut c_void, rest: *mut c_char);
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;
    fn nvim_syn_incl_toplevel(id: c_int, flagsp: *mut c_int);

    // Standard C library
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // Reused from earlier phases
    fn nvim_syn_skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_skiptowhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_ends_excmd(c: c_int) -> c_int;
    fn nvim_syn_ascii_iswhite_char(c: c_int) -> c_int;
    fn nvim_syn_xfree(ptr: *mut c_void);
    fn nvim_syn_check_group_wrapper(name: *const c_char, len: c_int) -> c_int;
    fn nvim_syn_emsg(msg: *const c_char);
    fn nvim_syn_semsg_1s(fmt: *const c_char, arg: *const c_char);
}

/// A collected pattern with its metadata.
struct CollectedPat {
    pat: SynPatHandle,
    matchgroup_id: c_int,
    item_type: c_int,
}

/// Rust implementation of syn_cmd_region.
///
/// This parses `:syntax region` commands, collecting START/SKIP/END patterns
/// and options, then stores them into the synblock.
unsafe fn syn_cmd_region_impl(eap: *mut c_void, syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    let skip = nvim_syn_get_eap_skip(eap);

    // Isolate the group name, check for validity
    let mut group_name_end: *mut c_char = std::ptr::null_mut();
    let mut rest = nvim_syn_get_group_name(arg, &mut group_name_end);

    nvim_syn_init_patterns();

    // Initialize option struct fields
    let mut opt_flags: c_int = 0;
    let mut cont_list: *mut i16 = std::ptr::null_mut();
    let mut cont_in_list: *mut i16 = std::ptr::null_mut();
    let mut next_list: *mut i16 = std::ptr::null_mut();
    let mut conceal_char: c_int = 0;
    let mut matchgroup_id: c_int = 0;

    // Collected patterns (replaces C linked list)
    let mut patterns: Vec<CollectedPat> = Vec::new();
    let mut has_skip = false;

    let mut not_enough = false;
    let mut illegal = false;
    let mut key: *mut c_char = std::ptr::null_mut();

    // Parse options and patterns
    while !rest.is_null() && nvim_syn_ends_excmd(*rest as c_int) == 0 {
        // Check for option arguments
        rest = crate::opt_parse::rs_get_syn_options(
            rest,
            &mut opt_flags,
            0,                    // keyword = false
            std::ptr::null_mut(), // sync_idx = NULL
            1,                    // has_cont_list = true
            &mut cont_list,
            &mut cont_in_list,
            &mut next_list,
            &mut conceal_char,
            skip,
        );
        if rest.is_null() || nvim_syn_ends_excmd(*rest as c_int) != 0 {
            break;
        }

        // Find keyword (START, END, SKIP, MATCHGROUP)
        let mut key_end = rest;
        while *key_end != 0
            && nvim_syn_ascii_iswhite_char(*key_end as c_int) == 0
            && *key_end as u8 != b'='
        {
            key_end = key_end.add(1);
        }
        if !key.is_null() {
            nvim_syn_xfree(key as *mut c_void);
        }
        let key_len = key_end.offset_from(rest) as c_int;
        key = nvim_syn_vim_strnsave_up(rest, key_len);

        let item = if c_strcmp(key, c"MATCHGROUP".as_ptr()) == 0 {
            ITEM_MATCHGROUP
        } else if c_strcmp(key, c"START".as_ptr()) == 0 {
            ITEM_START
        } else if c_strcmp(key, c"END".as_ptr()) == 0 {
            ITEM_END
        } else if c_strcmp(key, c"SKIP".as_ptr()) == 0 {
            if has_skip {
                illegal = true;
                break;
            }
            ITEM_SKIP
        } else {
            break;
        };

        rest = nvim_syn_skipwhite(key_end);
        if *rest as u8 != b'=' {
            rest = std::ptr::null_mut();
            nvim_syn_semsg_1s(c"E398: Missing '=': %s".as_ptr(), arg);
            break;
        }
        rest = nvim_syn_skipwhite(rest.add(1));
        if *rest == 0 {
            not_enough = true;
            break;
        }

        if item == ITEM_MATCHGROUP {
            let p = nvim_syn_skiptowhite(rest);
            let mg_len = p.offset_from(rest) as c_int;
            if (mg_len == 4 && strncmp(rest, c"NONE".as_ptr(), 4) == 0) || skip != 0 {
                matchgroup_id = 0;
            } else {
                matchgroup_id = nvim_syn_check_group_wrapper(rest, mg_len);
                if matchgroup_id == 0 {
                    illegal = true;
                    break;
                }
            }
            rest = nvim_syn_skipwhite(p);
        } else {
            // Compile the pattern (Rust)
            let mut pattern_rest: *mut c_char = std::ptr::null_mut();
            let pat = pattern_store::compile_pattern(rest, item, opt_flags, &mut pattern_rest);

            if pat.is_null() {
                rest = std::ptr::null_mut();
                break;
            }

            if item == ITEM_SKIP {
                has_skip = true;
            }

            patterns.push(CollectedPat {
                pat,
                matchgroup_id,
                item_type: item,
            });

            rest = pattern_rest;
        }
    }

    if !key.is_null() {
        nvim_syn_xfree(key as *mut c_void);
    }

    if illegal || not_enough {
        rest = std::ptr::null_mut();
    }

    // Must have at least one START and one END pattern
    let has_start = patterns.iter().any(|p| p.item_type == ITEM_START);
    let has_end = patterns.iter().any(|p| p.item_type == ITEM_END);
    if !rest.is_null() && (!has_start || !has_end) {
        not_enough = true;
        rest = std::ptr::null_mut();
    }

    let mut success = false;

    if !rest.is_null() {
        // Check for trailing garbage or command
        nvim_syn_set_nextcmd(eap, rest);
        if nvim_syn_ends_excmd(*rest as c_int) == 0 || skip != 0 {
            rest = std::ptr::null_mut();
        } else {
            let syn_id =
                nvim_syn_check_group_wrapper(arg, group_name_end.offset_from(arg) as c_int);
            if syn_id != 0 {
                nvim_syn_incl_toplevel(syn_id, &mut opt_flags);

                // Build the ordered list for Rust storage:
                // START reversed, SKIP reversed, END reversed
                let mut ordered: Vec<(SynPatHandle, c_int, c_int)> =
                    Vec::with_capacity(patterns.len());
                for item_type in [ITEM_START, ITEM_SKIP, ITEM_END] {
                    let group: Vec<_> = patterns
                        .iter()
                        .filter(|p| p.item_type == item_type)
                        .collect();
                    for p in group.iter().rev() {
                        ordered.push((p.pat, p.matchgroup_id, p.item_type));
                    }
                }

                let result = pattern_store::store_region_patterns(
                    &ordered,
                    opt_flags,
                    syn_id,
                    conceal_char,
                    cont_list,
                    cont_in_list,
                    next_list,
                    syncing,
                );

                if result != 0 {
                    success = true;
                }
            }
        }
    }

    // Free patterns on error
    if !success {
        for cp in &patterns {
            pattern_store::free_compiled_pattern(cp.pat);
        }
        nvim_syn_xfree(cont_list as *mut c_void);
        nvim_syn_xfree(cont_in_list as *mut c_void);
        nvim_syn_xfree(next_list as *mut c_void);
        if not_enough {
            nvim_syn_semsg_1s(
                c"E399: Not enough arguments: syntax region %s".as_ptr(),
                arg,
            );
        } else if illegal || rest.is_null() {
            nvim_syn_semsg_1s(c"E475: Invalid argument: %s".as_ptr(), arg);
        }
    } else {
        // On success, free only the heap-allocated synpat_T wrapper shells.
        // The data was copied into the garray by store_region_patterns, so we
        // only need to free the outer allocation (not sp_prog or sp_pattern).
        for cp in &patterns {
            // Free the outer synpat_T allocation only (data already in garray).
            // We use nvim_syn_xfree to free just the struct allocation.
            crate::pattern_store::free_compiled_pattern_shell(cp.pat);
        }
    }
}

/// Compare a C string with a Rust c-string literal.
#[inline]
unsafe fn c_strcmp(a: *const c_char, b: *const c_char) -> c_int {
    strcmp(a, b)
}

// =============================================================================
// Exported FFI functions
// =============================================================================

/// Rust implementation of syn_cmd_region.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_region(eap: *mut c_void, syncing: c_int) {
    syn_cmd_region_impl(eap, syncing);
}
