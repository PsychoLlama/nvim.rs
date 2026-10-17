//! `:syntax clear` and the teardown of a syntax block.
//!
//! [`syntax_clear`] empties a whole block — keywords, patterns, clusters, sync
//! settings and the cached states. [`syn_cmd_clear`] is the command, which with
//! no argument does that and with one clears named groups or empties named
//! clusters. Individual items go through [`syn_remove_pattern`], which is also
//! how `:syntax sync clear` drops the sync items.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::c_int;

use crate::types::GArray;

use super::*;

/// Clear all syntax info for one block.
pub(crate) fn syntax_clear(mut block: SynBlockRef) {
    block.b_syn_error = false; // clear previous error
    block.b_syn_slow = false; // clear previous timeout
    block.b_syn_ic = 0; // Use case, by default
    block.b_syn_foldlevel = SYNFLD_START;
    block.b_syn_spell = SYNSPL_DEFAULT; // default spell checking
    block.b_syn_containedin = 0;
    block.b_syn_conceal = 0;

    block.b_keywtab.clear();
    block.b_keywtab_ic.clear();

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

    block.b_syn_linecont_prog = None;
    block.b_syn_linecont_pat = None;
    block.b_syn_folditems = 0;
    block.b_syn_isk = None;

    syn_stack_free_all(block);
    invalidate_current_state();

    // Reset the counter for ":syntax include".
    running_syn_inc_tag.set(0);
}

impl SynBlock {
    /// An empty block with every setting at its default: what upstream's
    /// `xcalloc` gave a buffer and `:ownsyntax`, and what a zeroed block
    /// would be if all-zero bytes were a valid `Vec` and `Option`.
    pub(crate) fn new() -> SynBlock {
        let no_time = SynTime {
            total: 0,
            slowest: 0,
            count: 0,
            match_0: 0,
        };
        SynBlock {
            b_keywtab: KeywordTable::new(),
            b_keywtab_ic: KeywordTable::new(),
            b_syn_error: false,
            b_syn_slow: false,
            b_syn_ic: 0,
            b_syn_foldlevel: 0,
            b_syn_spell: 0,
            b_syn_patterns: Vec::new(),
            b_syn_clusters: Vec::new(),
            b_spell_cluster_id: 0,
            b_nospell_cluster_id: 0,
            b_syn_containedin: 0,
            b_syn_sync_flags: 0,
            b_syn_sync_id: 0,
            b_syn_sync_minlines: 0,
            b_syn_sync_maxlines: 0,
            b_syn_sync_linebreaks: 0,
            b_syn_linecont_pat: None,
            b_syn_linecont_prog: None,
            b_syn_linecont_time: no_time,
            b_syn_linecont_ic: 0,
            b_syn_topgrp: 0,
            b_syn_conceal: 0,
            b_syn_folditems: 0,
            b_sst: StateCache::new(),
            b_sst_lasttick: 0,
            b_langp: GArray::EMPTY,
            b_spell_ismw: [false; 256],
            b_spell_ismw_mb: ::core::ptr::null_mut(),
            b_p_spc: None,
            b_cap_prog: None,
            b_p_spf: None,
            b_p_spl: None,
            b_p_spo: None,
            b_p_spo_flags: 0,
            b_cjk: 0,
            b_syn_chartab: [0; 32],
            b_syn_isk: None,
        }
    }
}

/// Get rid of `:ownsyntax` for window `window`.
///
/// The window's own block is dropped even when `w_s` no longer points at
/// it -- a buffer switch re-points `w_s` without asking -- where upstream
/// lost track of it.
pub(crate) fn reset_synblock(mut window: Win) {
    let own = window.buffer().syntax_block();
    if window.w_s != own {
        syntax_clear(window.syntax());
        window.w_s = own;
    }
    window.w_own_syntax = None;
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

    block.b_syn_linecont_prog = None;
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
    let mut at = args.line.arg;
    args.line.next = args.line.find_next(at);
    if args.skip {
        return;
    }

    // Disabled inside ":syntax include @group filename", because otherwise
    // @group would get deleted. Only Vim 5.x syntax files contain
    // ":syntax clear" at all.
    if cur_syn_block().b_syn_topgrp != 0 {
        return;
    }

    if ends_excmd(c_int::from(args.line.byte_at(at))) != 0 {
        // No argument: clear all syntax items.
        if syncing != 0 {
            syntax_sync_clear();
        } else {
            syntax_clear(cur_syn_block());
            if cur_syn_block().raw() == Win::current().buffer().syntax_block() {
                let _ = do_unlet(b"b:current_syntax", true);
            }
            let _ = do_unlet(b"w:current_syntax", true);
        }
    } else {
        // Clear the groups and clusters the argument names.
        while ends_excmd(c_int::from(args.line.byte_at(at))) == 0 {
            let (word, word_len) = word_at(args.line.tail(at));
            if word.first() == Some(&b'@') {
                let id = syn_scl_namen2id(&word[1..]);
                if id == 0 {
                    let arg = msg_bytes(args.line.rest_of(at));
                    semsg!("E391: No such syntax cluster: {arg}");
                    break;
                }
                // A cluster cannot be deleted without changing the ids of
                // the ones after it, so the next best thing: empty it.
                let at = usize::try_from(id - SYNID_CLUSTER)
                    .expect("a non-zero cluster id is at least `SYNID_CLUSTER`");
                cur_syn_block().clusters_mut()[at].scl_list = IdList::NONE;
            } else {
                let id = syn_name2id_bytes(word);
                if id == 0 {
                    let arg = msg_bytes(args.line.rest_of(at));
                    semsg!("E28: No such highlight group name: {arg}");
                    break;
                }
                syn_clear_one(id, syncing != 0);
            }
            at = args.line.skip_white(at + word_len);
        }
    }
    redraw_curbuf_later(UPD_SOME_VALID);
    syn_stack_free_all(cur_syn_block()); // Need to recompute all syntax.
}

/// Clear one syntax group for the current window's block.
fn syn_clear_one(id: c_int, syncing: bool) {
    // Keywords only when this is not ":syntax sync clear {group}".
    if !syncing {
        cur_syn_block().b_keywtab.remove_group(id);
        cur_syn_block().b_keywtab_ic.remove_group(id);
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
