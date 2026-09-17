//! `:syntax clear` and the teardown of a syntax block.
//!
//! [`syntax_clear`] empties a whole block — keywords, patterns, clusters, sync
//! settings and the cached states. [`syn_cmd_clear`] is the command, which with
//! no argument does that and with one clears named groups or empties named
//! clusters. Individual items go through [`syn_remove_pattern`], which is also
//! how `:syntax sync clear` drops the sync items.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::c_int;

use super::*;

/// Clear all syntax info for one block.
///
/// # Safety
///
/// `block` must point at a live `SynBlock`, unaliased for the call.
pub(crate) unsafe fn syntax_clear(block: *mut SynBlock) {
    // SAFETY: the caller's promise -- a live syntax block.
    let mut block = unsafe { SynBlockRef::new(block) };
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
    block.b_syn_linecont_pat = None;
    block.b_syn_folditems = 0;
    block.b_syn_isk = None;

    syn_stack_free_all(block);
    invalidate_current_state();

    // Reset the counter for ":syntax include".
    running_syn_inc_tag.set(0);
}

/// Put the owning fields of a freshly zeroed `SynBlock` into a valid state.
///
/// A zeroed `Vec` is not one -- an empty `Vec` holds a dangling non-null
/// pointer -- and `Option`'s null representation is not something to lean
/// on. Upstream's blocks come out of `xcalloc`, so this is what stands in
/// for the zeroing it relied on.
///
/// # Safety
/// `at` must point at a zeroed block nothing has read or dropped.
pub(crate) unsafe fn init_synblock(at: *mut SynBlock) {
    // SAFETY: the caller's promise; `write` does not drop what was there.
    unsafe {
        (&raw mut (*at).b_syn_patterns).write(Vec::new());
        (&raw mut (*at).b_syn_clusters).write(Vec::new());
        (&raw mut (*at).b_syn_linecont_pat).write(None);
        // The block's five string options, for the reason
        // `init_buf_string_options` states: a zeroed `Option<XString>` is
        // not `None`.
        (&raw mut (*at).b_p_spc).write(None);
        (&raw mut (*at).b_p_spf).write(None);
        (&raw mut (*at).b_p_spl).write(None);
        (&raw mut (*at).b_p_spo).write(None);
        (&raw mut (*at).b_syn_isk).write(None);
    }
}

/// Get rid of `:ownsyntax` for window `window`.
pub(crate) fn reset_synblock(mut window: Win) {
    if window.w_s != unsafe { &raw mut (*window.w_buffer).b_s } {
        unsafe { syntax_clear(window.w_s) };
        // SAFETY: an `:ownsyntax` block, which `ex_ownsyntax` boxed and
        // only this releases; a buffer's own block took the branch above.
        drop(unsafe { Box::from_raw(window.w_s) });
        unsafe { window.w_s = &raw mut (*window.w_buffer).b_s };
    }
}

/// Clear the syncing info for the current window's block.
fn syntax_sync_clear() {
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
    block.b_syn_linecont_pat = None;
    block.b_syn_isk = None;

    syn_stack_free_all(block); // Need to recompute all syntax.
}

/// Remove one pattern from a block's pattern list, closing the gap.
///
/// Dropping the entry releases its text, its compiled program and its id
/// lists; the items that borrow those lists are the cached states, which
/// every caller of this drops with `syn_stack_free_all`.
pub(crate) fn syn_remove_pattern(mut block: SynBlockRef, idx: usize) {
    if block.patterns()[idx].sp_flags.has(SynFlags::FOLD) {
        block.b_syn_folditems -= 1;
    }
    block.patterns_mut().remove(idx);
}

/// `:syntax clear [{group}|@{cluster}] ..` and `:syntax sync clear ..`.
pub(crate) fn syn_cmd_clear(args: &mut ExArg, syncing: c_int) {
    let mut arg = args.arg;
    args.nextcmd = unsafe { find_nextcmd(arg) };
    if args.skip {
        return;
    }

    // Disabled inside ":syntax include @group filename", because otherwise
    // @group would get deleted. Only Vim 5.x syntax files contain
    // ":syntax clear" at all.
    if cur_syn_block().b_syn_topgrp != 0 {
        return;
    }

    if ends_excmd(c_int::from(unsafe { *arg })) != 0 {
        // No argument: clear all syntax items.
        if syncing != 0 {
            syntax_sync_clear();
        } else {
            unsafe { syntax_clear(cur_syn_block().raw()) };
            if cur_syn_block().raw() == unsafe { &raw mut (*Win::current().w_buffer).b_s } {
                let _ = unsafe { do_unlet(c"b:current_syntax".as_ptr(), 16, true) };
            }
            let _ = unsafe { do_unlet(c"w:current_syntax".as_ptr(), 16, true) };
        }
    } else {
        // Clear the groups and clusters the argument names.
        while ends_excmd(c_int::from(unsafe { *arg })) == 0 {
            // SAFETY: the caller's command line.
            let (word, arg_end) = unsafe { word_at(arg) };
            if word.first() == Some(&b'@') {
                let id = syn_scl_namen2id(&word[1..]);
                if id == 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg = unsafe { c_str(arg) };
                    semsg!("E391: No such syntax cluster: {arg}");
                    break;
                }
                // A cluster cannot be deleted without changing the ids of
                // the ones after it, so the next best thing: empty it.
                let at = usize::try_from(id - SYNID_CLUSTER)
                    .expect("a non-zero cluster id is at least `SYNID_CLUSTER`");
                cur_syn_block().clusters_mut()[at].scl_list = IdList::NONE;
            } else {
                let id = unsafe { syn_name2id_len(arg, word.len()) };
                if id == 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg = unsafe { c_str(arg) };
                    semsg!("E28: No such highlight group name: {arg}");
                    break;
                }
                syn_clear_one(id, syncing != 0);
            }
            arg = unsafe { skipwhite(arg_end) };
        }
    }
    redraw_curbuf_later(UPD_SOME_VALID);
    syn_stack_free_all(cur_syn_block()); // Need to recompute all syntax.
}

/// Clear one syntax group for the current window's block.
fn syn_clear_one(id: c_int, syncing: bool) {
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
        if c_int::from(spp.sp_syn.id) == id && spp.sp_syncing == syncing {
            syn_remove_pattern(block, idx);
        }
    }
}
