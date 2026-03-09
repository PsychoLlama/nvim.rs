//! Pattern storage for syntax highlighting.
//!
//! Migrated from `nvim_syn_store_match_pattern` and `nvim_syn_store_region_patterns`
//! in syntax_accessors.c (pass 7).
//!
//! Handles storing compiled patterns into the b_syn_patterns garray and
//! triggering redraws.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{
    SynBlockHandle, SynPatHandle, HL_EXCLUDENL, HL_FOLD, HL_HAS_EOL, HL_SYNC_HERE, HL_SYNC_THERE,
    SF_MATCH, SPTYPE_END, SPTYPE_MATCH, SPTYPE_SKIP, SPTYPE_START,
};

// Item type constants (matching C #defines).
const ITEM_START: c_int = 0;
const ITEM_SKIP: c_int = 1;
const ITEM_END: c_int = 2;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Pattern compilation helpers
    fn nvim_syn_xcalloc_synpat() -> SynPatHandle;
    fn nvim_syn_free_synpat(pat: SynPatHandle);
    fn nvim_syn_set_reg_do_extmatch(val: c_int);
    fn nvim_syn_vim_regcomp_had_eol() -> c_int;

    // (synpat_T setters removed -- use direct repr(C) field access)
    fn nvim_synblock_set_containedin(val: c_int);
    fn nvim_synblock_or_sync_flags(block: SynBlockHandle, flags: c_int);
    fn nvim_synblock_inc_folditems();

    // garray operations
    fn nvim_synblock_ga_append_pattern() -> SynPatHandle;

    // Current inc tag
    fn nvim_syn_get_current_inc_tag() -> c_int;

    // Redraw + cache invalidation
    fn nvim_syn_redraw_curbuf_later();
    fn nvim_syn_stack_free_all(block: SynBlockHandle);
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;
}

extern "C" {
    // Pattern parsing
    fn rs_get_syn_pattern(arg: *mut c_char, ci: SynPatHandle) -> *mut c_char;
    // Memory management (outer shell free only -- does NOT free sp_prog/sp_pattern)
    fn nvim_syn_xfree(ptr: *mut c_void);
}

// REX_SET=1, REX_USE=2
const REX_SET: c_int = 1;
const REX_USE: c_int = 2;

// =============================================================================
// Compile helper
// =============================================================================

/// Compile a single syntax pattern.
///
/// Allocates a synpat_T, sets reg_do_extmatch appropriately for item_type,
/// calls rs_get_syn_pattern, resets reg_do_extmatch, and handles HL_HAS_EOL.
///
/// On success returns a non-null SynPatHandle and sets `*rest_out`.
/// On failure returns null.
///
/// # Safety
/// `arg` must be a valid C string. `rest_out` must be a valid pointer.
pub unsafe fn compile_pattern(
    arg: *mut c_char,
    item_type: c_int,
    opt_flags: c_int,
    rest_out: *mut *mut c_char,
) -> SynPatHandle {
    // Set reg_do_extmatch for \z specials.
    if item_type == ITEM_START {
        nvim_syn_set_reg_do_extmatch(REX_SET);
    } else {
        nvim_syn_set_reg_do_extmatch(REX_USE);
    }

    let pat = nvim_syn_xcalloc_synpat();
    let rest = rs_get_syn_pattern(arg, pat);
    nvim_syn_set_reg_do_extmatch(0);

    if rest.is_null() {
        nvim_syn_free_synpat(pat);
        *rest_out = std::ptr::null_mut();
        return SynPatHandle(std::ptr::null_mut());
    }

    // Check for HL_HAS_EOL on end patterns (only if HL_EXCLUDENL not set)
    if item_type == ITEM_END
        && nvim_syn_vim_regcomp_had_eol() != 0
        && (opt_flags & HL_EXCLUDENL) == 0
    {
        (*pat.as_ptr()).sp_flags |= HL_HAS_EOL;
    }

    *rest_out = rest;
    pat
}

/// Free a heap-allocated compiled pattern on error.
/// Frees sp_prog, sp_pattern, and the outer synpat_T.
///
/// # Safety
/// `pat` must be a valid SynPatHandle or null.
pub unsafe fn free_compiled_pattern(pat: SynPatHandle) {
    if !pat.is_null() {
        nvim_syn_free_synpat(pat);
    }
}

/// Free only the outer synpat_T shell (no sp_prog/sp_pattern free).
/// Use this after store_region_patterns has copied the data into the garray.
///
/// # Safety
/// `pat` must be a valid SynPatHandle or null. sp_prog/sp_pattern are
/// now owned by the garray and must NOT be freed here.
pub unsafe fn free_compiled_pattern_shell(pat: SynPatHandle) {
    if !pat.is_null() {
        nvim_syn_xfree(pat.0 as *mut c_void);
    }
}

// =============================================================================
// Store: match pattern
// =============================================================================

/// Store a compiled match pattern into the current window's b_syn_patterns garray.
///
/// The synpat_T pointed to by pat is copied into the garray via
/// GA_APPEND_VIA_PTR. On success, cont_list, cont_in_list, and next_list
/// ownership is transferred to the stored pattern.
///
/// # Safety
/// `pat` must be a valid (non-null) SynPatHandle to a heap-allocated synpat_T.
/// Lists must be null or valid null-terminated i16 arrays (ownership transferred).
#[allow(clippy::too_many_arguments)]
pub unsafe fn store_match_pattern(
    pat: SynPatHandle,
    flags: c_int,
    syn_id: c_int,
    sync_idx: c_int,
    conceal_char: c_int,
    cont_list: *mut i16,
    cont_in_list: *mut i16,
    next_list: *mut i16,
    syncing: c_int,
) {
    let spp = nvim_synblock_ga_append_pattern();
    std::ptr::copy_nonoverlapping(pat.as_ptr(), spp.as_ptr(), 1);
    {
        let p = spp.as_ptr();
        (*p).sp_syncing = syncing != 0;
        (*p).sp_type = SPTYPE_MATCH as i8;
        (*p).sp_syn.id = syn_id as i16;
        (*p).sp_syn.inc_tag = nvim_syn_get_current_inc_tag();
        (*p).sp_flags = flags;
        (*p).sp_sync_idx = sync_idx;
        (*p).sp_cont_list = cont_list;
        (*p).sp_syn.cont_in_list = cont_in_list;
        (*p).sp_cchar = conceal_char;
        (*p).sp_next_list = next_list;
    }
    if !cont_in_list.is_null() {
        nvim_synblock_set_containedin(1);
    }

    if flags & (HL_SYNC_HERE | HL_SYNC_THERE) != 0 {
        nvim_synblock_or_sync_flags(nvim_syn_get_curwin_synblock(), SF_MATCH);
    }
    if flags & HL_FOLD != 0 {
        nvim_synblock_inc_folditems();
    }

    nvim_syn_redraw_curbuf_later();
    nvim_syn_stack_free_all(nvim_syn_get_curwin_synblock());
}

// =============================================================================
// Store: region patterns
// =============================================================================

/// Store a set of region patterns (START/SKIP/END) into the current window's
/// b_syn_patterns garray.
///
/// `pats` is a slice of (SynPatHandle, matchgroup_id, item_type) entries
/// in the order [all STARTs reversed, all SKIPs reversed, all ENDs reversed].
/// `flags`, `syn_id`, `conceal_char` come from option parsing.
/// `cont_list`, `cont_in_list`, `next_list` ownership is transferred on success.
///
/// Returns 1 on success, 0 on failure (currently always succeeds).
///
/// # Safety
/// All SynPatHandle pointers must be valid heap-allocated synpat_T values.
/// Lists must be null or valid null-terminated i16 arrays (ownership transferred).
#[allow(clippy::too_many_arguments)]
pub unsafe fn store_region_patterns(
    pats: &[(SynPatHandle, c_int, c_int)],
    flags: c_int,
    syn_id: c_int,
    conceal_char: c_int,
    cont_list: *mut i16,
    cont_in_list: *mut i16,
    next_list: *mut i16,
    syncing: c_int,
) -> c_int {
    let inc_tag = nvim_syn_get_current_inc_tag();

    for &(pat, matchgroup_id, item_type) in pats {
        let spp = nvim_synblock_ga_append_pattern();
        std::ptr::copy_nonoverlapping(pat.as_ptr(), spp.as_ptr(), 1);
        let sptype = if item_type == ITEM_START {
            SPTYPE_START
        } else if item_type == ITEM_SKIP {
            SPTYPE_SKIP
        } else {
            SPTYPE_END
        };
        {
            let p = spp.as_ptr();
            (*p).sp_syncing = syncing != 0;
            (*p).sp_type = sptype as i8;
            (*p).sp_flags |= flags;
            (*p).sp_syn.id = syn_id as i16;
            (*p).sp_syn.inc_tag = inc_tag;
            (*p).sp_syn_match_id = matchgroup_id as i16;
            (*p).sp_cchar = conceal_char;
            if item_type == ITEM_START {
                (*p).sp_cont_list = cont_list;
                (*p).sp_syn.cont_in_list = cont_in_list;
                if !cont_in_list.is_null() {
                    nvim_synblock_set_containedin(1);
                }
                (*p).sp_next_list = next_list;
            }
        }
        if flags & HL_FOLD != 0 {
            nvim_synblock_inc_folditems();
        }
    }

    nvim_syn_redraw_curbuf_later();
    nvim_syn_stack_free_all(nvim_syn_get_curwin_synblock());
    1
}
