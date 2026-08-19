//! `:syntax clear` and the teardown of a syntax block.
//!
//! [`syntax_clear`] empties a whole block — keywords, patterns, clusters, sync
//! settings and the cached states. [`syn_cmd_clear`] is the command, which with
//! no argument does that and with one clears named groups or empties named
//! clusters. Individual items go through [`syn_remove_pattern`], which is also
//! how `:syntax sync clear` drops the sync items.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};

use super::*;

/// Clear all syntax info for one block.
pub unsafe fn syntax_clear(block: *mut synblock_T) {
    unsafe {
        (*block).b_syn_error = false; // clear previous error
        (*block).b_syn_slow = false; // clear previous timeout
        (*block).b_syn_ic = 0; // Use case, by default
        (*block).b_syn_foldlevel = SYNFLD_START;
        (*block).b_syn_spell = SYNSPL_DEFAULT; // default spell checking
        (*block).b_syn_containedin = 0;
        (*block).b_syn_conceal = 0;

        clear_keywtab(&raw mut (*block).b_keywtab);
        clear_keywtab(&raw mut (*block).b_keywtab_ic);

        // Last to first: `syn_clear_pattern` looks at the entry before it.
        let mut i = (*block).b_syn_patterns.ga_len;
        while i > 0 {
            i -= 1;
            syn_clear_pattern(block, i);
        }
        ga_clear(&raw mut (*block).b_syn_patterns);

        let mut i = (*block).b_syn_clusters.ga_len;
        while i > 0 {
            i -= 1;
            syn_clear_cluster(block, i);
        }
        ga_clear(&raw mut (*block).b_syn_clusters);
        (*block).b_spell_cluster_id = 0;
        (*block).b_nospell_cluster_id = 0;

        (*block).b_syn_sync_flags = 0;
        (*block).b_syn_sync_minlines = 0;
        (*block).b_syn_sync_maxlines = 0;
        (*block).b_syn_sync_linebreaks = 0;

        vim_regfree((*block).b_syn_linecont_prog);
        (*block).b_syn_linecont_prog = ::core::ptr::null_mut();
        xfree((*block).b_syn_linecont_pat as *mut c_void);
        (*block).b_syn_linecont_pat = ::core::ptr::null_mut();
        (*block).b_syn_folditems = 0;
        clear_string_option(&raw mut (*block).b_syn_isk);

        syn_stack_free_all(block);
        invalidate_current_state();

        // Reset the counter for ":syntax include".
        running_syn_inc_tag.set(0);
    }
}

/// Get rid of `:ownsyntax` for window `wp`.
pub unsafe fn reset_synblock(wp: *mut win_T) {
    unsafe {
        if (*wp).w_s != &raw mut (*(*wp).w_buffer).b_s {
            syntax_clear((*wp).w_s);
            xfree((*wp).w_s as *mut c_void);
            (*wp).w_s = &raw mut (*(*wp).w_buffer).b_s;
        }
    }
}

/// Clear the syncing info for the current window's block.
unsafe fn syntax_sync_clear() {
    unsafe {
        let block = cur_syn_block();
        let mut i = (*block).b_syn_patterns.ga_len;
        while i > 0 {
            i -= 1;
            if (*cur_pattern(i)).sp_syncing {
                syn_remove_pattern(block, i);
            }
        }

        (*block).b_syn_sync_flags = 0;
        (*block).b_syn_sync_minlines = 0;
        (*block).b_syn_sync_maxlines = 0;
        (*block).b_syn_sync_linebreaks = 0;

        vim_regfree((*block).b_syn_linecont_prog);
        (*block).b_syn_linecont_prog = ::core::ptr::null_mut();
        xfree((*block).b_syn_linecont_pat as *mut c_void);
        (*block).b_syn_linecont_pat = ::core::ptr::null_mut();
        clear_string_option(&raw mut (*block).b_syn_isk);

        syn_stack_free_all(block); // Need to recompute all syntax.
    }
}

/// Remove one pattern from a block's pattern list, closing the gap.
pub(crate) unsafe fn syn_remove_pattern(block: *mut synblock_T, idx: c_int) {
    unsafe {
        let spp = block_pattern(block, idx);
        if (*spp).sp_flags.has(SynFlags::FOLD) {
            (*block).b_syn_folditems -= 1;
        }
        syn_clear_pattern(block, idx);
        let after = (*block).b_syn_patterns.ga_len - idx - 1;
        ::core::ptr::copy(spp.add(1), spp, after as usize);
        (*block).b_syn_patterns.ga_len -= 1;
    }
}

/// The pattern at `idx` in `block`, which is not always [`cur_syn_block`].
#[inline]
unsafe fn block_pattern(block: *mut synblock_T, idx: c_int) -> *mut synpat_T {
    unsafe { ((*block).b_syn_patterns.ga_data as *mut synpat_T).offset(idx as isize) }
}

/// Free one pattern's allocations.
///
/// When clearing all of them this must run **last to first**: only the first
/// START pattern of a region owns the three id lists, and "first" is decided by
/// looking at the entry before this one.
unsafe fn syn_clear_pattern(block: *mut synblock_T, i: c_int) {
    unsafe {
        let spp = block_pattern(block, i);
        xfree((*spp).sp_pattern as *mut c_void);
        vim_regfree((*spp).sp_prog);
        if i == 0 || (*block_pattern(block, i - 1)).sp_type as c_int != SPTYPE_START {
            xfree((*spp).sp_cont_list as *mut c_void);
            xfree((*spp).sp_next_list as *mut c_void);
            xfree((*spp).sp_syn.cont_in_list as *mut c_void);
        }
    }
}

/// Free one cluster's allocations.
unsafe fn syn_clear_cluster(block: *mut synblock_T, i: c_int) {
    unsafe {
        let scp = ((*block).b_syn_clusters.ga_data as *mut syn_cluster_T).offset(i as isize);
        xfree((*scp).scl_name as *mut c_void);
        xfree((*scp).scl_name_u as *mut c_void);
        xfree((*scp).scl_list as *mut c_void);
    }
}

/// `:syntax clear [{group}|@{cluster}] ..` and `:syntax sync clear ..`.
pub(crate) unsafe fn syn_cmd_clear(eap: *mut exarg_T, syncing: c_int) {
    unsafe {
        let mut arg = (*eap).arg;
        (*eap).nextcmd = find_nextcmd(arg);
        if (*eap).skip != 0 {
            return;
        }

        // Disabled inside ":syntax include @group filename", because otherwise
        // @group would get deleted. Only Vim 5.x syntax files contain
        // ":syntax clear" at all.
        if (*cur_syn_block()).b_syn_topgrp != 0 {
            return;
        }

        if ends_excmd(*arg as c_int) != 0 {
            // No argument: clear all syntax items.
            if syncing != 0 {
                syntax_sync_clear();
            } else {
                syntax_clear(cur_syn_block());
                if cur_syn_block() == &raw mut (*(*curwin.get()).w_buffer).b_s {
                    do_unlet(c"b:current_syntax".as_ptr(), 16, true);
                }
                do_unlet(c"w:current_syntax".as_ptr(), 16, true);
            }
        } else {
            // Clear the groups and clusters the argument names.
            while ends_excmd(*arg as c_int) == 0 {
                let arg_end = skiptowhite(arg);
                if *arg as c_int == '@' as c_int {
                    let id = syn_scl_namen2id(arg.add(1), arg_end.offset_from(arg) as c_int - 1);
                    if id == 0 {
                        semsg_c!(gettext(c"E391: No such syntax cluster: %s".as_ptr()), arg);
                        break;
                    }
                    // A cluster cannot be deleted without changing the ids of
                    // the ones after it, so the next best thing: empty it.
                    let scl = &mut (*cur_cluster(id - SYNID_CLUSTER)).scl_list;
                    xfree(*scl as *mut c_void);
                    *scl = ::core::ptr::null_mut();
                } else {
                    let id = syn_name2id_len(arg, arg_end.offset_from(arg) as size_t);
                    if id == 0 {
                        semsg_c!(gettext(&raw const e_nogroup as *const c_char), arg);
                        break;
                    }
                    syn_clear_one(id, syncing != 0);
                }
                arg = skipwhite(arg_end);
            }
        }
        redraw_curbuf_later(UPD_SOME_VALID);
        syn_stack_free_all(cur_syn_block()); // Need to recompute all syntax.
    }
}

/// Clear one syntax group for the current window's block.
unsafe fn syn_clear_one(id: c_int, syncing: bool) {
    unsafe {
        // Keywords only when this is not ":syntax sync clear {group}".
        if !syncing {
            syn_clear_keyword(id, &raw mut (*cur_syn_block()).b_keywtab);
            syn_clear_keyword(id, &raw mut (*cur_syn_block()).b_keywtab_ic);
        }

        let mut idx = cur_pattern_count();
        while idx > 0 {
            idx -= 1;
            let spp = cur_pattern(idx);
            if (*spp).sp_syn.id as c_int == id && (*spp).sp_syncing == syncing {
                syn_remove_pattern(cur_syn_block(), idx);
            }
        }
    }
}
