//! `:syntax clear` and the teardown of a syntax block.
//!
//! [`syntax_clear`] empties a whole block — keywords, patterns, clusters, sync
//! settings and the cached states. [`syn_cmd_clear`] is the command, which with
//! no argument does that and with one clears named groups or empties named
//! clusters. Individual items go through [`syn_remove_pattern`], which is also
//! how `:syntax sync clear` drops the sync items.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_int, c_void};

use super::*;

/// Clear all syntax info for one block.
pub(crate) unsafe fn syntax_clear(block: *mut synblock_T) {
    // SAFETY: the caller's promise -- a live syntax block.
    let mut block = unsafe { SynBlock::new(block) };
    block.b_syn_error = false; // clear previous error
    block.b_syn_slow = false; // clear previous timeout
    block.b_syn_ic = 0; // Use case, by default
    block.b_syn_foldlevel = SYNFLD_START;
    block.b_syn_spell = SYNSPL_DEFAULT; // default spell checking
    block.b_syn_containedin = 0;
    block.b_syn_conceal = 0;

    unsafe { clear_keywtab(&raw mut (*block.raw()).b_keywtab) };
    unsafe { clear_keywtab(&raw mut (*block.raw()).b_keywtab_ic) };

    // Last to first: `syn_clear_pattern` looks at the entry before it.
    let mut i = block.b_syn_patterns.ga_len;
    while i > 0 {
        i -= 1;
        unsafe { syn_clear_pattern(block, i) };
    }
    unsafe { ga_clear(&raw mut (*block.raw()).b_syn_patterns) };

    let mut i = block.b_syn_clusters.ga_len;
    while i > 0 {
        i -= 1;
        unsafe { syn_clear_cluster(block, i) };
    }
    unsafe { ga_clear(&raw mut (*block.raw()).b_syn_clusters) };
    block.b_spell_cluster_id = 0;
    block.b_nospell_cluster_id = 0;

    block.b_syn_sync_flags = 0;
    block.b_syn_sync_minlines = 0;
    block.b_syn_sync_maxlines = 0;
    block.b_syn_sync_linebreaks = 0;

    unsafe { vim_regfree(block.b_syn_linecont_prog) };
    block.b_syn_linecont_prog = ::core::ptr::null_mut();
    unsafe { xfree(block.b_syn_linecont_pat as *mut c_void) };
    block.b_syn_linecont_pat = ::core::ptr::null_mut();
    block.b_syn_folditems = 0;
    unsafe { clear_string_option(&raw mut (*block.raw()).b_syn_isk) };

    unsafe { syn_stack_free_all(block.raw()) };
    invalidate_current_state();

    // Reset the counter for ":syntax include".
    running_syn_inc_tag.set(0);
}

/// Get rid of `:ownsyntax` for window `wp`.
pub(crate) unsafe fn reset_synblock(wp: *mut win_T) {
    if unsafe { (*wp).w_s } != unsafe { &raw mut (*(*wp).w_buffer).b_s } {
        unsafe { syntax_clear((*wp).w_s) };
        unsafe { xfree((*wp).w_s as *mut c_void) };
        unsafe { (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s };
    }
}

/// Clear the syncing info for the current window's block.
unsafe fn syntax_sync_clear() {
    let mut block = cur_syn_block();
    let mut i = block.b_syn_patterns.ga_len;
    while i > 0 {
        i -= 1;
        if unsafe { cur_pattern(i).sp_syncing } {
            unsafe { syn_remove_pattern(block, i) };
        }
    }

    block.b_syn_sync_flags = 0;
    block.b_syn_sync_minlines = 0;
    block.b_syn_sync_maxlines = 0;
    block.b_syn_sync_linebreaks = 0;

    unsafe { vim_regfree(block.b_syn_linecont_prog) };
    block.b_syn_linecont_prog = ::core::ptr::null_mut();
    unsafe { xfree(block.b_syn_linecont_pat as *mut c_void) };
    block.b_syn_linecont_pat = ::core::ptr::null_mut();
    unsafe { clear_string_option(&raw mut (*block.raw()).b_syn_isk) };

    unsafe { syn_stack_free_all(block.raw()) }; // Need to recompute all syntax.
}

/// Remove one pattern from a block's pattern list, closing the gap.
pub(crate) unsafe fn syn_remove_pattern(mut block: SynBlock, idx: c_int) {
    let spp = unsafe { block_pattern(block, idx) };
    if spp.sp_flags.has(SynFlags::FOLD) {
        block.b_syn_folditems -= 1;
    }
    unsafe { syn_clear_pattern(block, idx) };
    let after = block.b_syn_patterns.ga_len - idx - 1;
    unsafe { ::core::ptr::copy(spp.raw().add(1), spp.raw(), after as usize) };
    block.b_syn_patterns.ga_len -= 1;
}

/// The pattern at `idx` in `block`, which is not always [`cur_syn_block`].
#[inline]
unsafe fn block_pattern(mut block: SynBlock, idx: c_int) -> Pat {
    // SAFETY: the caller's promise -- a live index into `block`'s array.
    unsafe { Pat::new((block.b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize)) }
}

/// Free one pattern's allocations.
///
/// When clearing all of them this must run **last to first**: only the first
/// START pattern of a region owns the three id lists, and "first" is decided by
/// looking at the entry before this one.
unsafe fn syn_clear_pattern(mut block: SynBlock, i: c_int) {
    let spp = unsafe { block_pattern(block, i) };
    unsafe { xfree(spp.sp_pattern as *mut c_void) };
    unsafe { vim_regfree(spp.sp_prog) };
    if i == 0 || unsafe { block_pattern(block, i - 1).sp_type } as c_int != SPTYPE_START {
        unsafe { xfree(spp.sp_cont_list as *mut c_void) };
        unsafe { xfree(spp.sp_next_list as *mut c_void) };
        unsafe { xfree(spp.sp_syn.cont_in_list as *mut c_void) };
    }
}

/// Free one cluster's allocations.
unsafe fn syn_clear_cluster(mut block: SynBlock, i: c_int) {
    let scp = unsafe { (block.b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize) };
    unsafe { xfree((*scp).scl_name as *mut c_void) };
    unsafe { xfree((*scp).scl_name_u as *mut c_void) };
    unsafe { xfree((*scp).scl_list as *mut c_void) };
}

/// `:syntax clear [{group}|@{cluster}] ..` and `:syntax sync clear ..`.
pub(crate) unsafe fn syn_cmd_clear(eap: *mut exarg_T, syncing: c_int) {
    let mut arg = unsafe { (*eap).arg };
    unsafe { (*eap).nextcmd = find_nextcmd(arg) };
    if unsafe { (*eap).skip } != 0 {
        return;
    }

    // Disabled inside ":syntax include @group filename", because otherwise
    // @group would get deleted. Only Vim 5.x syntax files contain
    // ":syntax clear" at all.
    if cur_syn_block().b_syn_topgrp != 0 {
        return;
    }

    if ends_excmd(unsafe { *arg } as c_int) != 0 {
        // No argument: clear all syntax items.
        if syncing != 0 {
            unsafe { syntax_sync_clear() };
        } else {
            unsafe { syntax_clear(cur_syn_block().raw()) };
            if cur_syn_block().raw() == unsafe { &raw mut (*(*curwin.get()).w_buffer).b_s } {
                unsafe { do_unlet(c"b:current_syntax".as_ptr(), 16, true) };
            }
            unsafe { do_unlet(c"w:current_syntax".as_ptr(), 16, true) };
        }
    } else {
        // Clear the groups and clusters the argument names.
        while ends_excmd(unsafe { *arg } as c_int) == 0 {
            let arg_end = unsafe { skiptowhite(arg) };
            if unsafe { *arg } as c_int == '@' as c_int {
                let id =
                    unsafe { syn_scl_namen2id(arg.add(1), arg_end.offset_from(arg) as c_int - 1) };
                if id == 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg = unsafe { c_str(arg) };
                    semsg!("E391: No such syntax cluster: {arg}");
                    break;
                }
                // A cluster cannot be deleted without changing the ids of
                // the ones after it, so the next best thing: empty it.
                let scl = unsafe { &mut (*cur_cluster(id - SYNID_CLUSTER).raw()).scl_list };
                unsafe { xfree(*scl as *mut c_void) };
                *scl = ::core::ptr::null_mut();
            } else {
                let id = unsafe { syn_name2id_len(arg, arg_end.offset_from(arg) as size_t) };
                if id == 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg = unsafe { c_str(arg) };
                    semsg!("E28: No such highlight group name: {arg}");
                    break;
                }
                unsafe { syn_clear_one(id, syncing != 0) };
            }
            arg = unsafe { skipwhite(arg_end) };
        }
    }
    redraw_curbuf_later(UPD_SOME_VALID);
    unsafe { syn_stack_free_all(cur_syn_block().raw()) }; // Need to recompute all syntax.
}

/// Clear one syntax group for the current window's block.
unsafe fn syn_clear_one(id: c_int, syncing: bool) {
    // Keywords only when this is not ":syntax sync clear {group}".
    if !syncing {
        unsafe { syn_clear_keyword(id, syn_field!(cur_syn_block(), b_keywtab)) };
        unsafe { syn_clear_keyword(id, syn_field!(cur_syn_block(), b_keywtab_ic)) };
    }

    let mut idx = cur_pattern_count();
    while idx > 0 {
        idx -= 1;
        let spp = unsafe { cur_pattern(idx) };
        if spp.sp_syn.id as c_int == id && spp.sp_syncing == syncing {
            unsafe { syn_remove_pattern(cur_syn_block(), idx) };
        }
    }
}
