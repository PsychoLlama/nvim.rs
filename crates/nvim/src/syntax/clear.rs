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

    // Each pattern owns its own pattern text, compiled program and id
    // lists, so dropping the array is the whole of what upstream's
    // last-to-first `syn_clear_pattern` walk did.
    block.patterns_mut().clear();
    block.clusters_mut().clear();
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
        // SAFETY: an `:ownsyntax` block, which `ex_ownsyntax` boxed and
        // only this releases; a buffer's own block took the branch above.
        drop(unsafe { Box::from_raw((*wp).w_s) });
        unsafe { (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s };
    }
}

/// Clear the syncing info for the current window's block.
unsafe fn syntax_sync_clear() {
    let mut block = cur_syn_block();
    let mut i = block.patterns().len();
    while i > 0 {
        i -= 1;
        if block.patterns()[i].sp_syncing {
            syn_remove_pattern(block, i);
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
///
/// Dropping the entry releases its text, its compiled program and its id
/// lists; the items that borrow those lists are the cached states, which
/// every caller of this drops with `syn_stack_free_all`.
pub(crate) fn syn_remove_pattern(mut block: SynBlock, idx: usize) {
    if block.patterns()[idx].sp_flags.has(SynFlags::FOLD) {
        block.b_syn_folditems -= 1;
    }
    block.patterns_mut().remove(idx);
}

/// `:syntax clear [{group}|@{cluster}] ..` and `:syntax sync clear ..`.
pub(crate) fn syn_cmd_clear(eap: &mut exarg_T, syncing: c_int) {
    let mut arg = eap.arg;
    eap.nextcmd = unsafe { find_nextcmd(arg) };
    if eap.skip != 0 {
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
                let _ = unsafe { do_unlet(c"b:current_syntax".as_ptr(), 16, true) };
            }
            let _ = unsafe { do_unlet(c"w:current_syntax".as_ptr(), 16, true) };
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
                let at = (id - SYNID_CLUSTER) as usize;
                cur_syn_block().clusters_mut()[at].scl_list = IdList::NONE;
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

    let block = cur_syn_block();
    let mut idx = block.patterns().len();
    while idx > 0 {
        idx -= 1;
        let spp = &block.patterns()[idx];
        if spp.sp_syn.id as c_int == id && spp.sp_syncing == syncing {
            syn_remove_pattern(block, idx);
        }
    }
}
