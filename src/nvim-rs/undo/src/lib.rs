//! Undo system utilities for Neovim
//!
//! This crate provides functions for Neovim's multi-level undo/redo system.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int, c_long, c_void};

/// Opaque handle to buf_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BufHandle(*mut c_void);

/// Opaque handle to u_header_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UHeaderHandle(*mut c_void);

/// Opaque handle to u_entry_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UEntryHandle(*mut c_void);

/// Type alias for time_t (platform-dependent).
#[cfg(target_pointer_width = "64")]
pub type TimeT = i64;
#[cfg(target_pointer_width = "32")]
pub type TimeT = i32;

/// Type alias for linenr_T (line number type).
pub type LinenrT = c_long;

/// Type alias for colnr_T (column number type).
pub type ColnrT = c_int;

/// Success return value (matches Neovim's OK).
const OK: c_int = 1;

/// Failure return value (matches Neovim's FAIL).
const FAIL: c_int = 0;

// FFI declarations for C accessor functions
#[allow(dead_code)]
extern "C" {
    // Buffer undo field accessors
    fn nvim_buf_get_b_u_oldhead(buf: BufHandle) -> UHeaderHandle;
    fn nvim_buf_get_b_u_newhead(buf: BufHandle) -> UHeaderHandle;
    fn nvim_buf_get_b_u_curhead(buf: BufHandle) -> UHeaderHandle;
    fn nvim_buf_get_b_u_numhead(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_u_synced(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_u_line_ptr(buf: BufHandle) -> *mut c_char;
    fn nvim_buf_get_b_u_line_lnum(buf: BufHandle) -> c_long;

    fn nvim_buf_set_b_u_oldhead(buf: BufHandle, val: UHeaderHandle);
    fn nvim_buf_set_b_u_newhead(buf: BufHandle, val: UHeaderHandle);
    fn nvim_buf_set_b_u_curhead(buf: BufHandle, val: UHeaderHandle);
    fn nvim_buf_set_b_u_numhead(buf: BufHandle, val: c_int);
    fn nvim_buf_set_b_u_synced(buf: BufHandle, val: bool);
    fn nvim_buf_set_b_u_line_ptr(buf: BufHandle, val: *mut c_char);
    fn nvim_buf_set_b_u_line_lnum(buf: BufHandle, val: c_long);

    // Buffer state accessors
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_bt_dontwrite(buf: BufHandle) -> bool;
    fn nvim_bt_prompt(buf: BufHandle) -> bool;
    fn nvim_file_ff_differs(buf: BufHandle, strict: bool) -> bool;

    // Global buffer iteration
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_buf_get_next(buf: BufHandle) -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;

    // Memory functions
    fn nvim_xfree(ptr: *mut c_void);

    // u_header_T field accessors
    fn nvim_uhp_get_next(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_prev(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_alt_next(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_alt_prev(uhp: UHeaderHandle) -> UHeaderHandle;
    fn nvim_uhp_get_seq(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_walk(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_entry(uhp: UHeaderHandle) -> UEntryHandle;
    fn nvim_uhp_get_getbot_entry(uhp: UHeaderHandle) -> UEntryHandle;
    fn nvim_uhp_get_time(uhp: UHeaderHandle) -> TimeT;
    fn nvim_uhp_get_flags(uhp: UHeaderHandle) -> c_int;
    fn nvim_uhp_get_save_nr(uhp: UHeaderHandle) -> c_int;

    fn nvim_uhp_set_next(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_prev(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_alt_next(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_alt_prev(uhp: UHeaderHandle, val: UHeaderHandle);
    fn nvim_uhp_set_seq(uhp: UHeaderHandle, val: c_int);
    fn nvim_uhp_set_walk(uhp: UHeaderHandle, val: c_int);
    fn nvim_uhp_set_entry(uhp: UHeaderHandle, val: UEntryHandle);
    fn nvim_uhp_set_getbot_entry(uhp: UHeaderHandle, val: UEntryHandle);
    fn nvim_uhp_set_time(uhp: UHeaderHandle, val: TimeT);
    fn nvim_uhp_set_flags(uhp: UHeaderHandle, val: c_int);
    fn nvim_uhp_set_save_nr(uhp: UHeaderHandle, val: c_int);

    // u_entry_T field accessors
    fn nvim_uep_get_next(uep: UEntryHandle) -> UEntryHandle;
    fn nvim_uep_get_top(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_bot(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_lcount(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_size(uep: UEntryHandle) -> LinenrT;
    fn nvim_uep_get_array(uep: UEntryHandle) -> *mut *mut c_char;

    fn nvim_uep_set_next(uep: UEntryHandle, val: UEntryHandle);
    fn nvim_uep_set_top(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_bot(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_lcount(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_size(uep: UEntryHandle, val: LinenrT);
    fn nvim_uep_set_array(uep: UEntryHandle, val: *mut *mut c_char);

    // u_entry_T array element accessors
    fn nvim_uep_get_array_element(uep: UEntryHandle, idx: LinenrT) -> *mut c_char;
    fn nvim_uep_set_array_element(uep: UEntryHandle, idx: LinenrT, val: *mut c_char);

    // Allocation functions
    fn nvim_alloc_u_entry() -> UEntryHandle;
    fn nvim_alloc_u_header() -> UHeaderHandle;

    // Extmark vector destruction
    fn nvim_uhp_destroy_extmark(uhp: UHeaderHandle);

    // Buffer memline accessor
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;

    // Error message wrappers
    fn nvim_iemsg_undo_list_corrupt();
    fn nvim_iemsg_undo_line_missing();

    // Global state accessors
    fn nvim_get_no_u_sync() -> c_int;
    fn nvim_get_undolevel(buf: BufHandle) -> i64;

    // Buffer b_did_warn accessor
    fn nvim_buf_set_b_did_warn(buf: BufHandle, val: bool);

    // Buffer save_nr accessors
    fn nvim_buf_get_b_u_save_nr_last(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_save_nr_last(buf: BufHandle, val: c_int);
    fn nvim_buf_set_b_u_save_nr_cur(buf: BufHandle, val: c_int);

    // undo_allowed accessors
    fn nvim_buf_is_modifiable(buf: BufHandle) -> bool;
    fn nvim_get_sandbox() -> c_int;
    fn nvim_get_textlock() -> c_int;
    fn nvim_get_expr_map_lock() -> c_int;
    fn nvim_curbuf_is_dummy() -> c_int;

    // undo_allowed error message wrappers
    fn nvim_emsg_modifiable();
    fn nvim_emsg_sandbox();
    fn nvim_emsg_textlock();

    // ex_undojoin error message wrapper
    fn nvim_emsg_undojoin_after_undo();

    // u_undo/u_redo accessors
    fn nvim_has_cpo_undo() -> bool;
    fn nvim_get_undo_undoes() -> bool;
    fn nvim_set_undo_undoes(val: bool);

    // u_undo_and_forget accessors
    fn nvim_buf_get_b_u_seq_cur(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_seq_cur(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_seq_last(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_seq_last(buf: BufHandle, val: c_int);

    // u_doit accessors
    fn nvim_buf_ml_is_empty(buf: BufHandle) -> bool;
    fn nvim_get_u_newcount() -> c_int;
    fn nvim_set_u_newcount(val: c_int);
    fn nvim_get_u_oldcount() -> c_int;
    fn nvim_set_u_oldcount(val: c_int);
    fn nvim_msg_ext_set_kind_undo();
    fn nvim_change_warning_curbuf();
    fn nvim_beep_flush();
    fn nvim_msg_oldest_change();
    fn nvim_msg_newest_change();
    fn nvim_u_undoredo(undo: bool, do_buf_event: bool);
    fn nvim_u_undo_end(did_undo: bool, absolute: bool, quiet: bool);

    // Infrastructure for future migration (u_savecommon, etc.)
    fn nvim_ml_get_buf_copy(buf: BufHandle, lnum: LinenrT) -> *mut c_char;
    fn nvim_fast_breakcheck();
    fn nvim_undo_got_int() -> bool;
    fn nvim_time_now() -> TimeT;
    fn nvim_get_curwin_cursor(lnum: *mut LinenrT, col: *mut ColnrT, coladd: *mut ColnrT);
    fn nvim_curwin_virtual_active() -> bool;
    fn nvim_getviscol() -> ColnrT;

    // u_savecommon infrastructure
    fn nvim_buf_set_b_new_change(buf: BufHandle, val: bool);
    fn nvim_buf_set_b_u_time_cur(buf: BufHandle, val: TimeT);
    fn nvim_uhp_init_extmark(uhp: UHeaderHandle);
    fn nvim_uhp_copy_marks_visual(buf: BufHandle, uhp: UHeaderHandle);
    fn nvim_uhp_set_cursor(uhp: UHeaderHandle, lnum: LinenrT, col: ColnrT, coladd: ColnrT);
    fn nvim_uhp_set_cursor_vcol(uhp: UHeaderHandle, vcol: ColnrT);
    fn nvim_uep_alloc_array(uep: UEntryHandle, size: LinenrT);
    fn nvim_uep_set_array_from_buf(uep: UEntryHandle, idx: LinenrT, buf: BufHandle, lnum: LinenrT);
    fn nvim_emsg_line_count_changed();
    fn nvim_buf_is_curbuf(buf: BufHandle) -> bool;
    fn nvim_u_saveline(buf: BufHandle, lnum: LinenrT);
    fn nvim_set_undo_undoes_false();

    // u_find_first_changed infrastructure
    fn nvim_uep_compare_line_with_array(
        uep: UEntryHandle,
        idx: LinenrT,
        buf: BufHandle,
        lnum: LinenrT,
    ) -> bool;
    fn nvim_uhp_clear_cursor(uhp: UHeaderHandle);
    fn nvim_uhp_set_cursor_lnum_only(uhp: UHeaderHandle, lnum: LinenrT);
}

/// Check if the 'modified' flag is set, or 'ff' has changed.
/// "nofile" and "scratch" type buffers are considered to always be unchanged.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_bufIsChanged(buf: BufHandle) -> bool {
    // In a "prompt" buffer we do respect 'modified', so that we can control
    // closing the window by setting or resetting that option.
    (!nvim_bt_dontwrite(buf) || nvim_bt_prompt(buf))
        && (nvim_buf_get_b_changed(buf) || nvim_file_ff_differs(buf, true))
}

/// Return true if any buffer has changes. Also buffers that are not written.
///
/// # Safety
///
/// Accesses global buffer list via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_anyBufIsChanged() -> bool {
    let mut buf = nvim_get_firstbuf();
    while !buf.0.is_null() {
        if rs_bufIsChanged(buf) {
            return true;
        }
        buf = nvim_buf_get_next(buf);
    }
    false
}

/// Return true if the current buffer has changed.
///
/// # Safety
///
/// Accesses global curbuf via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_curbufIsChanged() -> bool {
    rs_bufIsChanged(nvim_get_curbuf())
}

/// Invalidate the undo buffer; called when storage has already been released.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_clearall(buf: BufHandle) {
    nvim_buf_set_b_u_newhead(buf, UHeaderHandle(std::ptr::null_mut()));
    nvim_buf_set_b_u_oldhead(buf, UHeaderHandle(std::ptr::null_mut()));
    nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
    nvim_buf_set_b_u_synced(buf, true);
    nvim_buf_set_b_u_numhead(buf, 0);
    nvim_buf_set_b_u_line_ptr(buf, std::ptr::null_mut());
    nvim_buf_set_b_u_line_lnum(buf, 0);
}

/// Clear the line saved for the "U" command.
/// (this is used externally for crossing a line while in insert mode)
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_clearline(buf: BufHandle) {
    let line_ptr = nvim_buf_get_b_u_line_ptr(buf);
    if line_ptr.is_null() {
        return;
    }

    nvim_xfree(line_ptr as *mut c_void);
    nvim_buf_set_b_u_line_ptr(buf, std::ptr::null_mut());
    nvim_buf_set_b_u_line_lnum(buf, 0);
}

/// Free entry 'uep' and 'n' lines in uep->ue_array[].
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freeentry(uep: UEntryHandle, mut n: c_int) {
    // Free array elements from n-1 down to 0
    while n > 0 {
        n -= 1;
        let elem = nvim_uep_get_array_element(uep, LinenrT::from(n));
        nvim_xfree(elem as *mut c_void);
    }

    // Free the array itself
    let array = nvim_uep_get_array(uep);
    nvim_xfree(array as *mut c_void);

    // Free the entry struct
    nvim_xfree(uep.0);
}

/// Free all the undo entries for one header and the header itself.
/// This means that "uhp" is invalid when returning.
///
/// # Safety
///
/// All handles must be valid pointers. uhpp may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freeentries(
    buf: BufHandle,
    uhp: UHeaderHandle,
    uhpp: *mut UHeaderHandle,
) {
    // Check for pointers to the header that become invalid now.
    let curhead = nvim_buf_get_b_u_curhead(buf);
    if curhead.0 == uhp.0 {
        nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead.0 == uhp.0 {
        nvim_buf_set_b_u_newhead(buf, UHeaderHandle(std::ptr::null_mut()));
    }

    if !uhpp.is_null() && (*uhpp).0 == uhp.0 {
        *uhpp = UHeaderHandle(std::ptr::null_mut());
    }

    // Free all entries in the list
    let mut uep = nvim_uhp_get_entry(uhp);
    while !uep.0.is_null() {
        let nuep = nvim_uep_get_next(uep);
        let size = nvim_uep_get_size(uep);
        rs_u_freeentry(uep, size as c_int);
        uep = nuep;
    }

    // Destroy the extmark vector
    nvim_uhp_destroy_extmark(uhp);

    // Free the header struct
    nvim_xfree(uhp.0);

    // Decrement header count
    let numhead = nvim_buf_get_b_u_numhead(buf);
    nvim_buf_set_b_u_numhead(buf, numhead - 1);
}

/// Free one header "uhp" and its entry list and adjust the pointers.
///
/// # Safety
///
/// All handles must be valid pointers. uhpp may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freeheader(
    buf: BufHandle,
    uhp: UHeaderHandle,
    uhpp: *mut UHeaderHandle,
) {
    // When there is an alternate redo list free that branch completely,
    // because we can never go there.
    let alt_next = nvim_uhp_get_alt_next(uhp);
    if !alt_next.0.is_null() {
        rs_u_freebranch(buf, alt_next, uhpp);
    }

    let alt_prev = nvim_uhp_get_alt_prev(uhp);
    if !alt_prev.0.is_null() {
        nvim_uhp_set_alt_next(alt_prev, UHeaderHandle(std::ptr::null_mut()));
    }

    // Update the links in the list to remove the header.
    let uh_next = nvim_uhp_get_next(uhp);
    let uh_prev = nvim_uhp_get_prev(uhp);

    if uh_next.0.is_null() {
        nvim_buf_set_b_u_oldhead(buf, uh_prev);
    } else {
        nvim_uhp_set_prev(uh_next, uh_prev);
    }

    if uh_prev.0.is_null() {
        nvim_buf_set_b_u_newhead(buf, uh_next);
    } else {
        let mut uhap = uh_prev;
        while !uhap.0.is_null() {
            nvim_uhp_set_next(uhap, uh_next);
            uhap = nvim_uhp_get_alt_next(uhap);
        }
    }

    rs_u_freeentries(buf, uhp, uhpp);
}

/// Free an alternate branch and any following alternate branches.
///
/// # Safety
///
/// All handles must be valid pointers. uhpp may be NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_u_freebranch(
    buf: BufHandle,
    uhp: UHeaderHandle,
    uhpp: *mut UHeaderHandle,
) {
    // If this is the top branch we may need to use u_freeheader() to update
    // all the pointers.
    let oldhead = nvim_buf_get_b_u_oldhead(buf);
    if uhp.0 == oldhead.0 {
        loop {
            let current_oldhead = nvim_buf_get_b_u_oldhead(buf);
            if current_oldhead.0.is_null() {
                break;
            }
            rs_u_freeheader(buf, current_oldhead, uhpp);
        }
        return;
    }

    let alt_prev = nvim_uhp_get_alt_prev(uhp);
    if !alt_prev.0.is_null() {
        nvim_uhp_set_alt_next(alt_prev, UHeaderHandle(std::ptr::null_mut()));
    }

    let mut next = uhp;
    while !next.0.is_null() {
        let tofree = next;
        let alt_next = nvim_uhp_get_alt_next(tofree);
        if !alt_next.0.is_null() {
            rs_u_freebranch(buf, alt_next, uhpp); // recursive
        }
        next = nvim_uhp_get_prev(tofree);
        rs_u_freeentries(buf, tofree, uhpp);
    }
}

/// Get the first entry in the undo header for the buffer.
/// Returns NULL if the undo list is corrupt.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_get_headentry(buf: BufHandle) -> UEntryHandle {
    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead.0.is_null() {
        nvim_iemsg_undo_list_corrupt();
        return UEntryHandle(std::ptr::null_mut());
    }

    let entry = nvim_uhp_get_entry(newhead);
    if entry.0.is_null() {
        nvim_iemsg_undo_list_corrupt();
        return UEntryHandle(std::ptr::null_mut());
    }

    entry
}

/// Compute the line number of the previous u_save.
/// It is called only when b_u_synced is false.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_getbot(buf: BufHandle) {
    // Check for corrupt undo list
    let check = rs_u_get_headentry(buf);
    if check.0.is_null() {
        return;
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    let uep = nvim_uhp_get_getbot_entry(newhead);
    if !uep.0.is_null() {
        // The new ue_bot is computed from the number of lines that has been
        // inserted (0 - deleted) since calling u_save. This is equal to the
        // old line count subtracted from the current line count.
        let ml_line_count = nvim_buf_get_ml_line_count(buf);
        let ue_lcount = nvim_uep_get_lcount(uep);
        let extra = ml_line_count - ue_lcount;

        let ue_top = nvim_uep_get_top(uep);
        let ue_size = nvim_uep_get_size(uep);
        let mut ue_bot = ue_top + ue_size + 1 + extra;

        if ue_bot < 1 || ue_bot > ml_line_count {
            nvim_iemsg_undo_line_missing();
            // Assume all lines deleted, will get all the old lines back
            // without deleting the current ones
            ue_bot = ue_top + 1;
        }

        nvim_uep_set_bot(uep, ue_bot);
        nvim_uhp_set_getbot_entry(newhead, UEntryHandle(std::ptr::null_mut()));
    }

    nvim_buf_set_b_u_synced(buf, true);
}

/// Free all undo headers and entries for the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_blockfree(buf: BufHandle) {
    loop {
        let oldhead = nvim_buf_get_b_u_oldhead(buf);
        if oldhead.0.is_null() {
            break;
        }
        rs_u_freeheader(buf, oldhead, std::ptr::null_mut());
    }

    // Free the line saved for "U" command
    let line_ptr = nvim_buf_get_b_u_line_ptr(buf);
    nvim_xfree(line_ptr as *mut c_void);
}

/// Stop adding to the current entry list.
///
/// # Safety
///
/// Accesses global curbuf via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_u_sync(force: bool) {
    let buf = nvim_get_curbuf();

    // Skip it when already synced or syncing is disabled.
    if nvim_buf_get_b_u_synced(buf) {
        return;
    }
    if !force && nvim_get_no_u_sync() > 0 {
        return;
    }

    if nvim_get_undolevel(buf) < 0 {
        // No entries, nothing to do
        nvim_buf_set_b_u_synced(buf, true);
    } else {
        // Compute ue_bot of previous u_save
        rs_u_getbot(buf);
        nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
    }
}

/// Free all allocated memory blocks for the buffer and invalidate the undo buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_clearallandblockfree(buf: BufHandle) {
    rs_u_blockfree(buf);
    rs_u_clearall(buf);
}

/// UH_CHANGED flag value from undo_defs.h
const UH_CHANGED: c_int = 0x01;

/// Mark all headers in the branch as changed (recursive).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T (or NULL).
#[no_mangle]
pub unsafe extern "C" fn rs_u_unch_branch(uhp: UHeaderHandle) {
    let mut uh = uhp;
    while !uh.0.is_null() {
        // Set UH_CHANGED flag
        let flags = nvim_uhp_get_flags(uh);
        nvim_uhp_set_flags(uh, flags | UH_CHANGED);

        // Recurse into alternate branch if present
        let alt_next = nvim_uhp_get_alt_next(uh);
        if !alt_next.0.is_null() {
            rs_u_unch_branch(alt_next);
        }

        // Move to previous header
        uh = nvim_uhp_get_prev(uh);
    }
}

/// Called after writing or reloading the file and setting b_changed to false.
/// Now an undo means that the buffer is modified.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_unchanged(buf: BufHandle) {
    let oldhead = nvim_buf_get_b_u_oldhead(buf);
    rs_u_unch_branch(oldhead);
    nvim_buf_set_b_did_warn(buf, false);
}

/// Increase the write count, store it in the last undo header.
/// This is what would be used for "u".
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_update_save_nr(buf: BufHandle) {
    let save_nr_last = nvim_buf_get_b_u_save_nr_last(buf) + 1;
    nvim_buf_set_b_u_save_nr_last(buf, save_nr_last);
    nvim_buf_set_b_u_save_nr_cur(buf, save_nr_last);

    let curhead = nvim_buf_get_b_u_curhead(buf);
    let uhp = if !curhead.0.is_null() {
        nvim_uhp_get_next(curhead)
    } else {
        nvim_buf_get_b_u_newhead(buf)
    };

    if !uhp.0.is_null() {
        nvim_uhp_set_save_nr(uhp, save_nr_last);
    }
}

/// Free a u_header_T and all its entries.
/// Used when reading an undo file fails.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_u_free_uhp(uhp: UHeaderHandle) {
    let mut uep = nvim_uhp_get_entry(uhp);
    while !uep.0.is_null() {
        let nuep = nvim_uep_get_next(uep);
        let size = nvim_uep_get_size(uep);
        rs_u_freeentry(uep, size as c_int);
        uep = nuep;
    }
    nvim_xfree(uhp.0);
}

/// Helper function to check if expression mapping is locked.
///
/// # Safety
///
/// Calls external C functions.
#[inline]
unsafe fn expr_map_locked() -> bool {
    let lock = nvim_get_expr_map_lock();
    let is_dummy = nvim_curbuf_is_dummy();
    lock > 0 && is_dummy == 0
}

/// Return true when undo is allowed. Otherwise print an error message and
/// return false.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_allowed(buf: BufHandle) -> bool {
    // Don't allow changes when 'modifiable' is off.
    if !nvim_buf_is_modifiable(buf) {
        nvim_emsg_modifiable();
        return false;
    }

    // In the sandbox it's not allowed to change the text.
    if nvim_get_sandbox() != 0 {
        nvim_emsg_sandbox();
        return false;
    }

    // Don't allow changes in the buffer while editing the cmdline.
    // The caller of getcmdline() may get confused.
    if nvim_get_textlock() != 0 || expr_map_locked() {
        nvim_emsg_textlock();
        return false;
    }

    true
}

/// ":undojoin": continue adding to the last entry list
///
/// # Safety
///
/// Accesses global curbuf via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_undojoin() {
    let buf = nvim_get_curbuf();

    // Nothing changed before
    let newhead = nvim_buf_get_b_u_newhead(buf);
    if newhead.0.is_null() {
        return;
    }

    // Not allowed after undo
    let curhead = nvim_buf_get_b_u_curhead(buf);
    if !curhead.0.is_null() {
        nvim_emsg_undojoin_after_undo();
        return;
    }

    // Already unsynced
    if !nvim_buf_get_b_u_synced(buf) {
        return;
    }

    // No entries, nothing to do
    if nvim_get_undolevel(buf) < 0 {
        return;
    }

    // Append next change to last entry
    nvim_buf_set_b_u_synced(buf, false);
}

/// If 'cpoptions' contains 'u': Undo the previous undo or redo (vi compatible).
/// If 'cpoptions' does not contain 'u': Always undo.
///
/// # Safety
///
/// Accesses global state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_u_undo(mut count: c_int) {
    let buf = nvim_get_curbuf();

    // If we get an undo command while executing a macro, we behave like the
    // original vi. If this happens twice in one macro the result will not
    // be compatible.
    if !nvim_buf_get_b_u_synced(buf) {
        rs_u_sync(true);
        count = 1;
    }

    if !nvim_has_cpo_undo() {
        nvim_set_undo_undoes(true);
    } else {
        nvim_set_undo_undoes(!nvim_get_undo_undoes());
    }

    rs_u_doit(count, false, true);
}

/// If 'cpoptions' contains 'u': Repeat the previous undo or redo.
/// If 'cpoptions' does not contain 'u': Always redo.
///
/// # Safety
///
/// Accesses global state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_u_redo(count: c_int) {
    if !nvim_has_cpo_undo() {
        nvim_set_undo_undoes(false);
    }

    rs_u_doit(count, false, true);
}

/// Undo and remove the branch from the undo tree.
/// Also moves the cursor (as a "normal" undo would).
///
/// # Safety
///
/// Accesses global state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_u_undo_and_forget(mut count: c_int, do_buf_event: bool) -> bool {
    let buf = nvim_get_curbuf();

    if !nvim_buf_get_b_u_synced(buf) {
        rs_u_sync(true);
        count = 1;
    }

    nvim_set_undo_undoes(true);
    rs_u_doit(count, true, do_buf_event);

    let curhead = nvim_buf_get_b_u_curhead(buf);
    if curhead.0.is_null() {
        // nothing was undone
        return false;
    }

    // Delete the current redo header
    // set the redo header to the next alternative branch (if any)
    // otherwise we will be in the leaf state
    let to_forget = curhead;
    let uh_next = nvim_uhp_get_next(to_forget);
    nvim_buf_set_b_u_newhead(buf, uh_next);

    let alt_next = nvim_uhp_get_alt_next(to_forget);
    nvim_buf_set_b_u_curhead(buf, alt_next);

    if !alt_next.0.is_null() {
        nvim_uhp_set_alt_next(to_forget, UHeaderHandle(std::ptr::null_mut()));
        let alt_prev = nvim_uhp_get_alt_prev(to_forget);
        nvim_uhp_set_alt_prev(alt_next, alt_prev);

        let alt_next_next = nvim_uhp_get_next(alt_next);
        if !alt_next_next.0.is_null() {
            nvim_buf_set_b_u_seq_cur(buf, nvim_uhp_get_seq(alt_next_next));
        } else {
            nvim_buf_set_b_u_seq_cur(buf, 0);
        }
    } else {
        let newhead = nvim_buf_get_b_u_newhead(buf);
        if !newhead.0.is_null() {
            nvim_buf_set_b_u_seq_cur(buf, nvim_uhp_get_seq(newhead));
        }
    }

    let alt_prev = nvim_uhp_get_alt_prev(to_forget);
    if !alt_prev.0.is_null() {
        let new_curhead = nvim_buf_get_b_u_curhead(buf);
        nvim_uhp_set_alt_next(alt_prev, new_curhead);
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    if !newhead.0.is_null() {
        let new_curhead = nvim_buf_get_b_u_curhead(buf);
        nvim_uhp_set_prev(newhead, new_curhead);
    }

    let seq_last = nvim_buf_get_b_u_seq_last(buf);
    let to_forget_seq = nvim_uhp_get_seq(to_forget);
    if seq_last == to_forget_seq {
        nvim_buf_set_b_u_seq_last(buf, seq_last - 1);
    }

    rs_u_freebranch(buf, to_forget, std::ptr::null_mut());
    true
}

/// Core undo/redo loop.
/// Performs the actual undo or redo operations based on the current state.
///
/// # Safety
///
/// Must be called with valid global state (curbuf, undo_undoes set correctly).
#[no_mangle]
pub unsafe extern "C" fn rs_u_doit(startcount: c_int, quiet: bool, do_buf_event: bool) {
    let buf = nvim_get_curbuf();

    if !rs_undo_allowed(buf) {
        return;
    }

    nvim_set_u_newcount(0);
    nvim_set_u_oldcount(if nvim_buf_ml_is_empty(buf) { -1 } else { 0 });

    nvim_msg_ext_set_kind_undo();
    let mut count = startcount;

    while count > 0 {
        count -= 1;

        // Do the change warning now, so that it triggers FileChangedRO when
        // needed. This may cause the file to be reloaded, that must happen
        // before we do anything, because it may change curbuf->b_u_curhead
        // and more.
        nvim_change_warning_curbuf();

        let undo_undoes = nvim_get_undo_undoes();

        if undo_undoes {
            let curhead = nvim_buf_get_b_u_curhead(buf);
            if curhead.0.is_null() {
                // first undo
                let newhead = nvim_buf_get_b_u_newhead(buf);
                nvim_buf_set_b_u_curhead(buf, newhead);
            } else if nvim_get_undolevel(buf) > 0 {
                // multi level undo - get next undo
                let next = nvim_uhp_get_next(curhead);
                nvim_buf_set_b_u_curhead(buf, next);
            }

            // nothing to undo
            let curhead = nvim_buf_get_b_u_curhead(buf);
            let numhead = nvim_buf_get_b_u_numhead(buf);
            if numhead == 0 || curhead.0.is_null() {
                // stick curbuf->b_u_curhead at end
                let oldhead = nvim_buf_get_b_u_oldhead(buf);
                nvim_buf_set_b_u_curhead(buf, oldhead);
                nvim_beep_flush();
                if count == startcount - 1 {
                    nvim_msg_oldest_change();
                    return;
                }
                break;
            }

            nvim_u_undoredo(true, do_buf_event);
        } else {
            let curhead = nvim_buf_get_b_u_curhead(buf);
            if curhead.0.is_null() || nvim_get_undolevel(buf) <= 0 {
                // nothing to redo
                nvim_beep_flush();
                if count == startcount - 1 {
                    nvim_msg_newest_change();
                    return;
                }
                break;
            }

            nvim_u_undoredo(false, do_buf_event);

            // Advance for next redo. Set "newhead" when at the end of the
            // redoable changes.
            let curhead = nvim_buf_get_b_u_curhead(buf);
            let prev = nvim_uhp_get_prev(curhead);
            if prev.0.is_null() {
                nvim_buf_set_b_u_newhead(buf, curhead);
            }
            nvim_buf_set_b_u_curhead(buf, prev);
        }
    }

    let undo_undoes = nvim_get_undo_undoes();
    nvim_u_undo_end(undo_undoes, false, quiet);
}

/// Common code for various ways to save text before a change.
/// "top" is the line above the first changed line.
/// "bot" is the line below the last changed line.
/// "newbot" is the new bottom line. Use zero when not known.
/// "reload" is true when saving for a buffer reload.
///
/// # Safety
///
/// Must be called with valid buffer handle and line numbers.
#[no_mangle]
pub unsafe extern "C" fn rs_u_savecommon(
    buf: BufHandle,
    top: LinenrT,
    bot: LinenrT,
    newbot: LinenrT,
    reload: bool,
) -> c_int {
    if !reload {
        // When making changes is not allowed return FAIL
        if !rs_undo_allowed(buf) {
            return FAIL;
        }

        // Saving text for undo means we are going to make a change
        if nvim_buf_is_curbuf(buf) {
            nvim_change_warning_curbuf();
        }

        let line_count = nvim_buf_get_ml_line_count(buf);
        if bot > line_count + 1 {
            nvim_emsg_line_count_changed();
            return FAIL;
        }
    }

    let size = bot - top - 1;

    // If curbuf->b_u_synced == true make a new header
    if nvim_buf_get_b_u_synced(buf) {
        // Need to create new entry in b_changelist
        nvim_buf_set_b_new_change(buf, true);

        let uhp: UHeaderHandle;
        if nvim_get_undolevel(buf) >= 0 {
            // Make a new header entry
            uhp = nvim_alloc_u_header();
            nvim_uhp_init_extmark(uhp);
        } else {
            uhp = UHeaderHandle(std::ptr::null_mut());
        }

        // If we undid more than we redid, move the entry lists before and
        // including curbuf->b_u_curhead to an alternate branch
        let mut old_curhead = nvim_buf_get_b_u_curhead(buf);
        if !old_curhead.0.is_null() {
            let next = nvim_uhp_get_next(old_curhead);
            nvim_buf_set_b_u_newhead(buf, next);
            nvim_buf_set_b_u_curhead(buf, UHeaderHandle(std::ptr::null_mut()));
        }

        // Free headers to keep the size right
        while nvim_buf_get_b_u_numhead(buf) as i64 > nvim_get_undolevel(buf) {
            let oldhead = nvim_buf_get_b_u_oldhead(buf);
            if oldhead.0.is_null() {
                break;
            }

            if oldhead.0 == old_curhead.0 {
                // Can't reconnect the branch, delete all of it
                rs_u_freebranch(buf, oldhead, &mut old_curhead as *mut UHeaderHandle);
            } else {
                let alt_next = nvim_uhp_get_alt_next(oldhead);
                if alt_next.0.is_null() {
                    // There is no branch, only free one header
                    rs_u_freeheader(buf, oldhead, &mut old_curhead as *mut UHeaderHandle);
                } else {
                    // Free the oldest alternate branch as a whole
                    let mut uhfree = oldhead;
                    loop {
                        let next_alt = nvim_uhp_get_alt_next(uhfree);
                        if next_alt.0.is_null() {
                            break;
                        }
                        uhfree = next_alt;
                    }
                    rs_u_freebranch(buf, uhfree, &mut old_curhead as *mut UHeaderHandle);
                }
            }
        }

        if uhp.0.is_null() {
            // No undo at all
            if !old_curhead.0.is_null() {
                rs_u_freebranch(buf, old_curhead, std::ptr::null_mut());
            }
            nvim_buf_set_b_u_synced(buf, false);
            return OK;
        }

        // Set up the new header
        nvim_uhp_set_prev(uhp, UHeaderHandle(std::ptr::null_mut()));
        let newhead = nvim_buf_get_b_u_newhead(buf);
        nvim_uhp_set_next(uhp, newhead);
        nvim_uhp_set_alt_next(uhp, old_curhead);

        if !old_curhead.0.is_null() {
            let alt_prev = nvim_uhp_get_alt_prev(old_curhead);
            nvim_uhp_set_alt_prev(uhp, alt_prev);

            if !alt_prev.0.is_null() {
                nvim_uhp_set_alt_next(alt_prev, uhp);
            }

            nvim_uhp_set_alt_prev(old_curhead, uhp);

            let oldhead = nvim_buf_get_b_u_oldhead(buf);
            if oldhead.0 == old_curhead.0 {
                nvim_buf_set_b_u_oldhead(buf, uhp);
            }
        } else {
            nvim_uhp_set_alt_prev(uhp, UHeaderHandle(std::ptr::null_mut()));
        }

        if !newhead.0.is_null() {
            nvim_uhp_set_prev(newhead, uhp);
        }

        // Set sequence numbers and time
        let seq_last = nvim_buf_get_b_u_seq_last(buf);
        nvim_buf_set_b_u_seq_last(buf, seq_last + 1);
        nvim_uhp_set_seq(uhp, seq_last + 1);
        nvim_buf_set_b_u_seq_cur(buf, seq_last + 1);

        let now = nvim_time_now();
        nvim_uhp_set_time(uhp, now);
        nvim_uhp_set_save_nr(uhp, 0);
        nvim_buf_set_b_u_time_cur(buf, now + 1);

        nvim_uhp_set_walk(uhp, 0);
        nvim_uhp_set_entry(uhp, UEntryHandle(std::ptr::null_mut()));
        nvim_uhp_set_getbot_entry(uhp, UEntryHandle(std::ptr::null_mut()));

        // Save cursor position
        let mut lnum: LinenrT = 0;
        let mut col: ColnrT = 0;
        let mut coladd: ColnrT = 0;
        nvim_get_curwin_cursor(&mut lnum, &mut col, &mut coladd);
        nvim_uhp_set_cursor(uhp, lnum, col, coladd);

        if nvim_curwin_virtual_active() && coladd > 0 {
            nvim_uhp_set_cursor_vcol(uhp, nvim_getviscol());
        } else {
            nvim_uhp_set_cursor_vcol(uhp, -1);
        }

        // Save changed and buffer empty flag
        let changed = nvim_buf_get_b_changed(buf);
        let ml_empty = nvim_buf_ml_is_empty(buf);
        let flags = (if changed { 1 } else { 0 }) + (if ml_empty { 2 } else { 0 });
        nvim_uhp_set_flags(uhp, flags);

        // Save named marks and Visual marks
        nvim_uhp_copy_marks_visual(buf, uhp);

        nvim_buf_set_b_u_newhead(buf, uhp);

        let oldhead = nvim_buf_get_b_u_oldhead(buf);
        if oldhead.0.is_null() {
            nvim_buf_set_b_u_oldhead(buf, uhp);
        }

        let numhead = nvim_buf_get_b_u_numhead(buf);
        nvim_buf_set_b_u_numhead(buf, numhead + 1);
    } else {
        if nvim_get_undolevel(buf) < 0 {
            // No undo at all
            return OK;
        }

        // When saving a single line, check if we can reuse existing entry
        if size == 1 {
            let mut uep = rs_u_get_headentry(buf);
            let mut prev_uep = UEntryHandle(std::ptr::null_mut());

            for _ in 0..10 {
                if uep.0.is_null() {
                    break;
                }

                let newhead = nvim_buf_get_b_u_newhead(buf);
                let getbot_entry = nvim_uhp_get_getbot_entry(newhead);
                let ue_top = nvim_uep_get_top(uep);
                let ue_size = nvim_uep_get_size(uep);
                let ue_bot = nvim_uep_get_bot(uep);
                let ue_lcount = nvim_uep_get_lcount(uep);
                let line_count = nvim_buf_get_ml_line_count(buf);

                // Check if lines have been inserted/deleted
                let reuse_blocked = if getbot_entry.0 != uep.0 {
                    ue_top + ue_size + 1 != (if ue_bot == 0 { line_count + 1 } else { ue_bot })
                } else {
                    ue_lcount != line_count
                };

                if reuse_blocked
                    || (ue_size > 1 && top >= ue_top && top + 2 <= ue_top + ue_size + 1)
                {
                    break;
                }

                // If it's the same line we can skip saving it again
                if ue_size == 1 && ue_top == top {
                    if !prev_uep.0.is_null() {
                        // Move the found entry to become the last entry
                        rs_u_getbot(buf);
                        nvim_buf_set_b_u_synced(buf, false);

                        let uep_next = nvim_uep_get_next(uep);
                        nvim_uep_set_next(prev_uep, uep_next);

                        let newhead = nvim_buf_get_b_u_newhead(buf);
                        let entry = nvim_uhp_get_entry(newhead);
                        nvim_uep_set_next(uep, entry);
                        nvim_uhp_set_entry(newhead, uep);
                    }

                    // The executed command may change the line count
                    if newbot != 0 {
                        nvim_uep_set_bot(uep, newbot);
                    } else if bot > line_count {
                        nvim_uep_set_bot(uep, 0);
                    } else {
                        nvim_uep_set_lcount(uep, line_count);
                        let newhead = nvim_buf_get_b_u_newhead(buf);
                        nvim_uhp_set_getbot_entry(newhead, uep);
                    }
                    return OK;
                }

                prev_uep = uep;
                uep = nvim_uep_get_next(uep);
            }
        }

        // Find line number for ue_bot for previous u_save()
        rs_u_getbot(buf);
    }

    // Add lines in front of entry list
    let uep = nvim_alloc_u_entry();

    nvim_uep_set_size(uep, size);
    nvim_uep_set_top(uep, top);

    let line_count = nvim_buf_get_ml_line_count(buf);
    if newbot != 0 {
        nvim_uep_set_bot(uep, newbot);
    } else if bot > line_count {
        nvim_uep_set_bot(uep, 0);
    } else {
        nvim_uep_set_lcount(uep, line_count);
        let newhead = nvim_buf_get_b_u_newhead(buf);
        nvim_uhp_set_getbot_entry(newhead, uep);
    }

    if size > 0 {
        nvim_uep_alloc_array(uep, size);
        let mut lnum = top + 1;
        for i in 0..size {
            nvim_fast_breakcheck();
            if nvim_undo_got_int() {
                rs_u_freeentry(uep, i as c_int);
                return FAIL;
            }
            nvim_uep_set_array_from_buf(uep, i, buf, lnum);
            lnum += 1;
        }
    } else {
        nvim_uep_set_array(uep, std::ptr::null_mut());
    }

    let newhead = nvim_buf_get_b_u_newhead(buf);
    let entry = nvim_uhp_get_entry(newhead);
    nvim_uep_set_next(uep, entry);
    nvim_uhp_set_entry(newhead, uep);

    if reload {
        // Buffer was reloaded, notify text change subscribers
        let curbuf = nvim_get_curbuf();
        let curbuf_newhead = nvim_buf_get_b_u_newhead(curbuf);
        let flags = nvim_uhp_get_flags(curbuf_newhead);
        nvim_uhp_set_flags(curbuf_newhead, flags | 4); // UH_RELOAD = 4
    }

    nvim_buf_set_b_u_synced(buf, false);
    nvim_set_undo_undoes_false();

    OK
}

/// Save the line at cursor position for undo.
/// Rust implementation of u_save_cursor.
///
/// # Safety
///
/// Must be called from a valid Neovim context with curwin set.
#[no_mangle]
pub unsafe extern "C" fn rs_u_save_cursor() -> c_int {
    let mut lnum: LinenrT = 0;
    let mut col: ColnrT = 0;
    let mut coladd: ColnrT = 0;
    nvim_get_curwin_cursor(&mut lnum, &mut col, &mut coladd);

    let top = if lnum > 0 { lnum - 1 } else { 0 };
    let bot = lnum + 1;

    rs_u_save(top, bot)
}

/// Save lines between top and bot for both "u" and "U" command.
/// Rust implementation of u_save.
///
/// # Safety
///
/// Must be called with valid line numbers for curbuf.
#[no_mangle]
pub unsafe extern "C" fn rs_u_save(top: LinenrT, bot: LinenrT) -> c_int {
    rs_u_save_buf(nvim_get_curbuf(), top, bot)
}

/// Save lines between top and bot for the specified buffer.
/// Rust implementation of u_save_buf.
///
/// # Safety
///
/// Must be called with valid buffer handle and line numbers.
#[no_mangle]
pub unsafe extern "C" fn rs_u_save_buf(buf: BufHandle, top: LinenrT, bot: LinenrT) -> c_int {
    let line_count = nvim_buf_get_ml_line_count(buf);

    if top >= bot || bot > (line_count + 1) {
        return FAIL;
    }

    if top + 2 == bot {
        nvim_u_saveline(buf, top + 1);
    }

    rs_u_savecommon(buf, top, bot, 0, false)
}

/// Save a line for substitution (used by ":s" and "~" command).
/// Rust implementation of u_savesub.
///
/// # Safety
///
/// Must be called with valid line number for curbuf.
#[no_mangle]
pub unsafe extern "C" fn rs_u_savesub(lnum: LinenrT) -> c_int {
    rs_u_savecommon(nvim_get_curbuf(), lnum - 1, lnum + 1, lnum + 1, false)
}

/// Save for line insertion (used by :s command).
/// Rust implementation of u_inssub.
///
/// # Safety
///
/// Must be called with valid line number for curbuf.
#[no_mangle]
pub unsafe extern "C" fn rs_u_inssub(lnum: LinenrT) -> c_int {
    rs_u_savecommon(nvim_get_curbuf(), lnum - 1, lnum, lnum + 1, false)
}

/// Save lines for deletion.
/// Rust implementation of u_savedel.
///
/// # Safety
///
/// Must be called with valid line numbers for curbuf.
#[no_mangle]
pub unsafe extern "C" fn rs_u_savedel(lnum: LinenrT, nlines: LinenrT) -> c_int {
    let buf = nvim_get_curbuf();
    let line_count = nvim_buf_get_ml_line_count(buf);
    let newbot = if nlines == line_count { 2 } else { lnum };

    rs_u_savecommon(buf, lnum - 1, lnum + nlines, newbot, false)
}

/// Find the first line that was changed and set uh_cursor to that line.
/// This is used after reloading a buffer.
/// Rust implementation of u_find_first_changed.
///
/// # Safety
///
/// Must be called from a valid Neovim context.
#[no_mangle]
pub unsafe extern "C" fn rs_u_find_first_changed() {
    let curbuf = nvim_get_curbuf();
    let uhp = nvim_buf_get_b_u_newhead(curbuf);

    // If curhead is set or newhead is null, return early
    if !nvim_buf_get_b_u_curhead(curbuf).0.is_null() || uhp.0.is_null() {
        return; // undid something in an autocmd?
    }

    // Check that the last undo block was for the whole file
    let uep = nvim_uhp_get_entry(uhp);
    if nvim_uep_get_top(uep) != 0 || nvim_uep_get_bot(uep) != 0 {
        return;
    }

    let line_count = nvim_buf_get_ml_line_count(curbuf);
    let ue_size = nvim_uep_get_size(uep);

    // Find the first line that differs
    let mut lnum: LinenrT = 1;
    while lnum < line_count && lnum <= ue_size {
        // Compare buffer line at lnum with ue_array[lnum - 1]
        if nvim_uep_compare_line_with_array(uep, lnum - 1, curbuf, lnum) {
            nvim_uhp_clear_cursor(uhp);
            nvim_uhp_set_cursor_lnum_only(uhp, lnum);
            return;
        }
        lnum += 1;
    }

    // Lines added or deleted at the end, put cursor there
    if line_count != ue_size {
        nvim_uhp_clear_cursor(uhp);
        nvim_uhp_set_cursor_lnum_only(uhp, lnum);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_sizes() {
        // Verify handle sizes match pointer size
        assert_eq!(
            std::mem::size_of::<BufHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
        assert_eq!(
            std::mem::size_of::<UHeaderHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
        assert_eq!(
            std::mem::size_of::<UEntryHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }
}
