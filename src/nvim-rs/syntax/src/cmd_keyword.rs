//! Syntax keyword command implementation.
//!
//! Migrated from `syn_cmd_keyword` in syntax_accessors.c.
//! Handles parsing of `:syntax keyword` commands.

use std::ffi::{c_char, c_int, c_void};

use crate::types::SynBlockHandle;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Command argument access
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;
    fn nvim_syn_set_nextcmd(eap: *mut c_void, rest: *mut c_char);

    // Group name parsing
    fn rs_get_group_name(arg: *mut c_char, name_end: *mut *mut c_char) -> *mut c_char;

    // Group checking
    fn syn_check_group(name: *const c_char, len: c_int) -> c_int;
    fn rs_syn_incl_toplevel(id: c_int, flagsp: *mut c_int);

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

    // Keyword storage
    fn rs_add_keyword(
        name: *mut c_char,
        namelen: c_int,
        id: c_int,
        flags: c_int,
        cont_in_list: *mut i16,
        next_list: *mut i16,
        conceal_char: c_int,
    );

    // Redraw and free syntax state (Phase 4: decomposed from nvim_syn_keyword_redraw_and_free)
    fn nvim_syn_redraw_curbuf_later();
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;
    #[link_name = "syn_stack_free_all"]
    fn nvim_syn_stack_free_all(block: SynBlockHandle);

    // String helpers
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn ends_excmd(c: c_int) -> c_int;
    #[link_name = "rs_ascii_iswhite"]
    fn nvim_syn_ascii_iswhite_char(c: c_int) -> c_int;
    fn utfc_ptr2len(p: *mut c_char) -> c_int;

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // Error messages
    fn nvim_syn_semsg_1s(fmt: *const c_char, arg: *const c_char);
    fn nvim_syn_semsg_2s(fmt: *const c_char, arg1: *const c_char, arg2: *const c_char);

    // C stdlib
    fn strlen(s: *const c_char) -> usize;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

/// Process one keyword entry in the keyword_copy buffer, handling the
/// `[optional]` character syntax. Modifies the buffer in place.
///
/// Returns `Some(advance)` where `advance` is the number of bytes to add to
/// `kw` to reach the start of the next keyword. Returns `None` on error.
///
/// This closely follows the C inner `for(;;)` loop in syn_cmd_keyword.
unsafe fn add_keyword_with_optional(
    kw: *mut c_char,
    syn_id: c_int,
    flags: c_int,
    cont_in_list: *mut i16,
    next_list: *mut i16,
    conceal_char: c_int,
) -> Option<usize> {
    // p = first '[' in kw, or NULL if none.
    let mut p = strchr(kw, b'[' as c_int);

    loop {
        // Determine the keyword length: up to p if found, else full string.
        let kwlen: c_int = if p.is_null() {
            strlen(kw) as c_int
        } else {
            *p = 0; // NUL-terminate at '['
            p.offset_from(kw) as c_int
        };

        rs_add_keyword(
            kw,
            kwlen,
            syn_id,
            flags,
            cont_in_list,
            next_list,
            conceal_char,
        );

        if p.is_null() {
            // No '[' found; we're done. Advance past keyword + NUL.
            return Some(kwlen as usize + 1);
        }

        // p[0] is now NUL (was '['); check p[1].
        if *p.add(1) == 0 {
            // E789: Missing ']'
            *p = b'[' as c_char; // Restore for error message.
            nvim_syn_semsg_1s(c"E789: Missing ']': %s".as_ptr(), kw);
            return None;
        }

        if *p.add(1) as u8 == b']' {
            if *p.add(2) != 0 {
                // E890: Trailing char after ']'
                *p = b'[' as c_char; // Restore for error message.
                nvim_syn_semsg_2s(
                    c"E890: Trailing char after ']': %s]%s".as_ptr(),
                    kw,
                    p.add(2),
                );
                return None;
            }
            // p+1 points to ']'; advance outer loop past ']' + NUL = 2 bytes.
            // (Matches C: `kw = p+1; kwlen = 1; break;` followed by outer
            // `kw += kwlen + 1 = kw += 2`.)
            let advance = p.add(1).offset_from(kw) as usize + 1 + 1;
            return Some(advance);
        }

        // Optional char: memmove(p, p+1, l) shifts char left, then p += l.
        let l = utfc_ptr2len(p.add(1)) as usize;
        memmove(p as *mut c_void, p.add(1) as *const c_void, l);
        p = p.add(l);
        // Continue loop: p now points to the next char to process.
    }
}

/// Rust implementation of syn_cmd_keyword.
unsafe fn syn_cmd_keyword_impl(eap: *mut c_void, _syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    let skip = nvim_syn_get_eap_skip(eap);

    // Isolate the group name
    let mut group_name_end: *mut c_char = std::ptr::null_mut();
    let rest_after_group = rs_get_group_name(arg, &mut group_name_end);

    // If group name parsing fails, rest_after_group is NULL.
    let syn_id: c_int = if rest_after_group.is_null() {
        0
    } else if skip != 0 {
        -1
    } else {
        syn_check_group(arg, group_name_end.offset_from(arg) as c_int)
    };

    // syn_id == 0 means group check failed; -1 means skip mode.
    let keyword_copy: *mut c_char = if syn_id != 0 && !rest_after_group.is_null() {
        xmalloc(strlen(rest_after_group) + 1) as *mut c_char
    } else {
        std::ptr::null_mut()
    };

    let mut rest = rest_after_group;
    let mut cnt: c_int = 0;

    if !keyword_copy.is_null() {
        // Initialize option fields.
        let mut opt_flags: c_int = 0;
        let mut dummy_cont_list: *mut i16 = std::ptr::null_mut();
        let mut cont_in_list: *mut i16 = std::ptr::null_mut();
        let mut next_list: *mut i16 = std::ptr::null_mut();
        let mut conceal_char: c_int = 0;

        // Phase 1: collect options and copy keywords to keyword_copy buffer.
        let mut p = keyword_copy;
        while !rest.is_null() && ends_excmd(*rest as c_int) == 0 {
            rest = rs_get_syn_options(
                rest,
                &mut opt_flags,
                1, // keyword = true
                std::ptr::null_mut(),
                0, // has_cont_list = false
                &mut dummy_cont_list,
                &mut cont_in_list,
                &mut next_list,
                &mut conceal_char,
                skip,
            );
            if rest.is_null() || ends_excmd(*rest as c_int) != 0 {
                break;
            }
            // Copy keyword, removing backslashes, then NUL-terminate.
            while *rest != 0 && nvim_syn_ascii_iswhite_char(*rest as c_int) == 0 {
                if *rest as u8 == b'\\' && *rest.add(1) != 0 {
                    rest = rest.add(1);
                }
                *p = *rest;
                p = p.add(1);
                rest = rest.add(1);
            }
            *p = 0;
            p = p.add(1);
            cnt += 1;
            if !rest.is_null() {
                rest = skipwhite(rest);
            }
        }

        // Phase 2: add an entry for each keyword (only if not skipping).
        if skip == 0 {
            rs_syn_incl_toplevel(syn_id, &mut opt_flags);

            let mut kw = keyword_copy;
            let mut i = cnt;
            while i > 0 {
                i -= 1;
                match add_keyword_with_optional(
                    kw,
                    syn_id,
                    opt_flags,
                    cont_in_list,
                    next_list,
                    conceal_char,
                ) {
                    Some(advance) => {
                        kw = kw.add(advance);
                    }
                    None => {
                        // Error already reported; break out (matches C `goto error`).
                        break;
                    }
                }
            }
        }

        xfree(keyword_copy as *mut c_void);
        xfree(cont_in_list as *mut c_void);
        xfree(next_list as *mut c_void);
    }

    if !rest.is_null() {
        nvim_syn_set_nextcmd(eap, rest);
    } else {
        nvim_syn_semsg_1s(c"E475: Invalid argument: %s".as_ptr(), arg);
    }

    // Redraw and free syntax state (Phase 4: replaces nvim_syn_keyword_redraw_and_free)
    nvim_syn_redraw_curbuf_later();
    nvim_syn_stack_free_all(nvim_syn_get_curwin_synblock());
}

// =============================================================================
// FFI exports
// =============================================================================

/// Rust implementation of syn_cmd_keyword.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_keyword(eap: *mut c_void, syncing: c_int) {
    syn_cmd_keyword_impl(eap, syncing);
}
