//! Syntax clear command implementation.
//!
//! Migrated from `syn_cmd_clear`, `syn_clear_one`, and `syntax_sync_clear`
//! in syntax_accessors.c.

use std::ffi::{c_char, c_int, c_void};

use crate::types::SynBlockHandle;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // EAP accessors
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;
    fn nvim_syn_find_nextcmd(eap: *mut c_void, arg: *mut c_char);

    // String helpers
    fn nvim_syn_skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_skiptowhite(s: *const c_char) -> *mut c_char;
    fn nvim_syn_ends_excmd(c: c_int) -> c_int;

    // Synblock
    fn nvim_get_curwin_synblock() -> SynBlockHandle;
    fn nvim_synblock_get_topgrp(block: SynBlockHandle) -> c_int;

    // Group lookup
    fn nvim_syn_scl_namen2id(arg: *const c_char, len: c_int) -> c_int;
    fn nvim_syn_name2id_len_wrapper(arg: *const c_char, len: c_int) -> c_int;

    // Group-level clear operations (kept in C due to hashtab coupling)
    fn nvim_synblock_clear_cluster_scl_list(block: SynBlockHandle, scl_id: c_int);
    fn nvim_syn_clear_one_wrapper(id: c_int, syncing: c_int);

    // Full clear operations
    fn nvim_syntax_clear_wrapper(block: SynBlockHandle);
    fn nvim_syntax_sync_clear_wrapper();

    // Redraw and free all
    fn nvim_syn_redraw_and_free_all();

    // Error messages
    fn semsg(fmt: *const c_char, ...);
}

// SYNID_CLUSTER = 23000 (must match C define)
const SYNID_CLUSTER: c_int = 23000;

static EMSG_E391: &[u8] = b"E391: No such syntax cluster: %s\0";
static EMSG_E_NOGROUP: &[u8] = b"E28: No such highlight group name: %s\0";

/// Rust implementation of syn_cmd_clear.
///
/// # Safety
/// Must be called from main thread during command execution.
unsafe fn syn_cmd_clear_impl(eap: *mut c_void, syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);

    nvim_syn_find_nextcmd(eap, arg);

    if nvim_syn_get_eap_skip(eap) != 0 {
        return;
    }

    let block = nvim_get_curwin_synblock();

    // Disabled within ":syn include @group filename" to avoid deleting @group
    if nvim_synblock_get_topgrp(block) != 0 {
        return;
    }

    if nvim_syn_ends_excmd(*arg as c_int) != 0 {
        // No argument: clear all syntax items
        if syncing != 0 {
            nvim_syntax_sync_clear_wrapper();
        } else {
            nvim_syntax_clear_wrapper(block);
            // do_unlet("b:current_syntax") if synblock is the buffer's block
            // do_unlet("w:current_syntax")
            nvim_syn_clear_unlet_vars(block);
        }
    } else {
        // Clear the group IDs listed in the argument
        let mut cur_arg = arg;
        while nvim_syn_ends_excmd(*cur_arg as c_int) == 0 {
            let arg_end = nvim_syn_skiptowhite(cur_arg);
            if *cur_arg == b'@' as c_char {
                let id = nvim_syn_scl_namen2id(
                    cur_arg.add(1),
                    arg_end.offset_from(cur_arg) as c_int - 1,
                );
                if id == 0 {
                    semsg(EMSG_E391.as_ptr().cast(), cur_arg);
                    break;
                }
                // Clear the cluster list (make it empty)
                let scl_id = id - SYNID_CLUSTER;
                nvim_synblock_clear_cluster_scl_list(block, scl_id);
            } else {
                let id =
                    nvim_syn_name2id_len_wrapper(cur_arg, arg_end.offset_from(cur_arg) as c_int);
                if id == 0 {
                    semsg(EMSG_E_NOGROUP.as_ptr().cast(), cur_arg);
                    break;
                }
                nvim_syn_clear_one_wrapper(id, syncing);
            }
            cur_arg = nvim_syn_skipwhite(arg_end);
        }
    }
    nvim_syn_redraw_and_free_all();
}

extern "C" {
    fn nvim_syn_clear_unlet_vars(block: SynBlockHandle);
}

/// Entry point called from C thin wrapper.
///
/// # Safety
/// Must be called from main thread during `:syntax clear` command execution.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_clear(eap: *mut c_void, syncing: c_int) {
    syn_cmd_clear_impl(eap, syncing);
}
