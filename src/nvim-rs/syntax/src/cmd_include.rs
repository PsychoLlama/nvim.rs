//! Syntax include command implementation.
//!
//! Migrated from `syn_cmd_include` in syntax_accessors.c.
//! Handles `:syntax include @group filename`.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // EAP accessors
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;
    fn nvim_syn_find_nextcmd(eap: *mut c_void, arg: *mut c_char);
    fn nvim_syn_set_eap_arg(eap: *mut c_void, arg: *mut c_char);

    // Group name parsing
    fn rs_get_group_name(arg: *mut c_char, name_end: *mut *mut c_char) -> *mut c_char;
    fn rs_syn_check_cluster(pp: *mut c_char, len: c_int) -> c_int;

    // State management
    fn nvim_syn_get_current_inc_tag() -> c_int;
    fn nvim_syn_set_current_inc_tag(tag: c_int);
    fn nvim_syn_get_running_inc_tag() -> c_int;
    fn nvim_syn_get_topgrp() -> c_int;
    fn nvim_syn_set_topgrp(topgrp: c_int);

    /// Atomically increment running_syn_inc_tag and assign to current_syn_inc_tag.
    /// Returns the new current_syn_inc_tag value.
    fn nvim_syn_increment_and_set_inc_tag() -> c_int;

    // File sourcing compound accessors
    /// Prepare the include: sets EX_XFILE|EX_NOSPC, calls separate_nextcmd,
    /// checks path_is_absolute/$/<, optionally calls expand_filename.
    /// Returns 1 if source (absolute path), 0 if runtime, -1 on expand failure.
    fn nvim_syn_include_prepare(eap: *mut c_void) -> c_int;

    /// Execute the file source or runtime load.
    /// Returns 0 on success (or on emitting an error), -1 on open failure.
    fn nvim_syn_include_source(eap: *mut c_void, use_source: c_int) -> c_int;

    // Error messages
    fn emsg(msg: *const c_char);
    fn semsg(fmt: *const c_char, ...);
}

const MAX_SYN_INC_TAG: c_int = 999;

static EMSG_E397: &[u8] = b"E397: Filename required\0";
static EMSG_E847: &[u8] = b"E847: Too many syntax includes\0";
static EMSG_E_NOTOPEN: &[u8] = b"E484: Can't open file %s\0";

/// Rust implementation of syn_cmd_include.
///
/// # Safety
/// Must be called from main thread during command execution.
unsafe fn syn_cmd_include_impl(eap: *mut c_void, _syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);

    nvim_syn_find_nextcmd(eap, arg);

    if nvim_syn_get_eap_skip(eap) != 0 {
        return;
    }

    let mut sgl_id: c_int = 1;

    if *arg == b'@' as c_char {
        let arg_after = arg.add(1);
        let mut group_name_end: *mut c_char = std::ptr::null_mut();
        let rest = rs_get_group_name(arg_after, &mut group_name_end);
        if rest.is_null() {
            emsg(EMSG_E397.as_ptr().cast());
            return;
        }
        sgl_id = rs_syn_check_cluster(arg_after, group_name_end.offset_from(arg_after) as c_int);
        if sgl_id == 0 {
            return;
        }
        // separate_nextcmd() and expand_filename() depend on eap->arg being `rest`
        nvim_syn_set_eap_arg(eap, rest);
    }

    // Prepare: set argt flags, call separate_nextcmd, check path, maybe expand
    let source = nvim_syn_include_prepare(eap);
    if source < 0 {
        // expand_filename failed; error was already reported
        return;
    }

    // Check include tag overflow
    if nvim_syn_get_running_inc_tag() >= MAX_SYN_INC_TAG {
        emsg(EMSG_E847.as_ptr().cast());
        return;
    }

    // Save/restore inc_tag and topgrp around the actual file load
    let prev_syn_inc_tag = nvim_syn_get_current_inc_tag();
    nvim_syn_increment_and_set_inc_tag();

    let prev_toplvl_grp = nvim_syn_get_topgrp();
    nvim_syn_set_topgrp(sgl_id);

    let result = nvim_syn_include_source(eap, source);
    if result != 0 {
        semsg(EMSG_E_NOTOPEN.as_ptr().cast(), nvim_syn_get_eap_arg(eap));
    }

    nvim_syn_set_topgrp(prev_toplvl_grp);
    nvim_syn_set_current_inc_tag(prev_syn_inc_tag);
}

/// Entry point called from C thin wrapper.
///
/// # Safety
/// Must be called from main thread during `:syntax include` command execution.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_include(eap: *mut c_void, syncing: c_int) {
    syn_cmd_include_impl(eap, syncing);
}
