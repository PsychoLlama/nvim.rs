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

    // u_undoline accessors
    fn nvim_buf_get_b_u_line_colnr(buf: BufHandle) -> ColnrT;
    fn nvim_buf_set_b_u_line_colnr(buf: BufHandle, val: ColnrT);
    fn nvim_undo_curwin_get_cursor_col() -> ColnrT;
    fn nvim_undo_curwin_set_cursor_col(col: ColnrT);
    fn nvim_undo_curwin_get_cursor_lnum() -> LinenrT;
    fn nvim_undo_curwin_set_cursor_lnum(lnum: LinenrT);
    fn nvim_check_cursor_col_curwin();
    fn nvim_u_undoline_replace_and_swap();

    // undo_time accessors
    fn nvim_buf_get_b_u_time_cur(buf: BufHandle) -> TimeT;
    fn nvim_buf_get_b_u_save_nr_cur(buf: BufHandle) -> c_int;
    fn nvim_text_locked() -> bool;
    fn nvim_text_locked_msg();
    fn nvim_undo_os_time() -> TimeT;
    fn nvim_inc_lastmark() -> c_int;
    fn nvim_internal_error_undo_time();
    fn nvim_semsg_undo_number_not_found(step: i64);
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

/// Restore the line saved for "U" command.
/// Rust implementation of u_undoline.
///
/// # Safety
///
/// Must be called from a valid Neovim context.
#[no_mangle]
pub unsafe extern "C" fn rs_u_undoline() {
    let curbuf = nvim_get_curbuf();
    let line_ptr = nvim_buf_get_b_u_line_ptr(curbuf);
    let line_lnum = nvim_buf_get_b_u_line_lnum(curbuf);
    let line_count = nvim_buf_get_ml_line_count(curbuf);

    // Check if line pointer is valid
    if line_ptr.is_null() || line_lnum > line_count {
        nvim_beep_flush();
        return;
    }

    // First save the line for the 'u' command
    if rs_u_savecommon(curbuf, line_lnum - 1, line_lnum + 1, 0, false) == FAIL {
        return;
    }

    // Do the replacement and swap
    nvim_u_undoline_replace_and_swap();

    // Handle column position
    let t = nvim_buf_get_b_u_line_colnr(curbuf);
    if nvim_undo_curwin_get_cursor_lnum() == line_lnum {
        nvim_buf_set_b_u_line_colnr(curbuf, nvim_undo_curwin_get_cursor_col());
    }
    nvim_undo_curwin_set_cursor_col(t);
    nvim_undo_curwin_set_cursor_lnum(line_lnum);
    nvim_check_cursor_col_curwin();
}

/// Given a buffer, return the undo header. If none is set, create one first.
/// NULL will be returned if e.g undolevels = -1 (undo disabled).
/// Rust implementation of u_force_get_undo_header.
///
/// # Safety
///
/// Must be called with a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_u_force_get_undo_header(buf: BufHandle) -> UHeaderHandle {
    let mut uhp = nvim_buf_get_b_u_curhead(buf);
    if uhp.0.is_null() {
        uhp = nvim_buf_get_b_u_newhead(buf);
    }

    // Create the first undo header for the buffer
    if uhp.0.is_null() {
        // Args are tricky: this means replace empty range by empty range
        rs_u_savecommon(buf, 0, 1, 1, true);

        uhp = nvim_buf_get_b_u_curhead(buf);
        if uhp.0.is_null() {
            uhp = nvim_buf_get_b_u_newhead(buf);
            // If undolevel > 0 and still no header, abort
            // (This shouldn't happen in normal operation)
        }
    }
    uhp
}

/// Navigate the undo tree to a specific time, sequence number, or file save state.
///
/// This is the core implementation for `:earlier`, `:later`, and `:undo N` commands.
///
/// # Arguments
///
/// * `step` - Number of steps to go (negative for undo/earlier, positive for redo/later)
/// * `sec` - If true, step is in seconds
/// * `file` - If true, step is by file writes
/// * `absolute` - If true, step is an absolute sequence number (`:undo N`)
///
/// # Safety
///
/// Accesses global state via C FFI. Must be called with valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_time(step: c_int, sec: bool, file: bool, absolute: bool) {
    // Check text lock first
    if nvim_text_locked() {
        nvim_text_locked_msg();
        return;
    }

    let buf = nvim_get_curbuf();

    // First make sure the current undoable change is synced.
    if !nvim_buf_get_b_u_synced(buf) {
        rs_u_sync(true);
    }

    nvim_set_u_newcount(0);
    nvim_set_u_oldcount(if nvim_buf_ml_is_empty(buf) { -1 } else { 0 });

    let mut dosec = sec;
    let mut dofile = file;
    let mut above = false;
    let mut did_undo = true;

    // "target" is the node below which we want to be.
    // Init "closest" to a value we can't reach.
    let (mut target, mut closest): (c_int, c_int) = if absolute {
        (step, -1)
    } else if dosec {
        (
            (nvim_buf_get_b_u_time_cur(buf) as c_int) + step,
            if step < 0 {
                -1
            } else {
                (nvim_undo_os_time() + 1) as c_int
            },
        )
    } else if dofile {
        let save_nr_cur = nvim_buf_get_b_u_save_nr_cur(buf);
        let mut t: c_int;

        if step < 0 {
            // Going back to a previous write. If there were changes after
            // the last write, count that as moving one file-write, so
            // that ":earlier 1f" undoes all changes since the last save.
            let curhead = nvim_buf_get_b_u_curhead(buf);
            let uhp = if !curhead.0.is_null() {
                nvim_uhp_get_next(curhead)
            } else {
                nvim_buf_get_b_u_newhead(buf)
            };

            if !uhp.0.is_null() && nvim_uhp_get_save_nr(uhp) != 0 {
                // "uh_save_nr" was set in the last block, that means
                // there were no changes since the last write
                t = save_nr_cur + step;
            } else {
                // count the changes since the last write as one step
                t = save_nr_cur + step + 1;
            }

            if t <= 0 {
                // Go to before first write: before the oldest change. Use
                // the sequence number for that.
                dofile = false;
                t = 0; // Will be adjusted below
            }
            (
                t,
                if step < 0 && dofile {
                    -1
                } else if dofile {
                    nvim_buf_get_b_u_save_nr_last(buf) + 2
                } else {
                    nvim_buf_get_b_u_seq_last(buf) + 2
                },
            )
        } else {
            // Moving forward to a newer write.
            t = save_nr_cur + step;
            let save_nr_last = nvim_buf_get_b_u_save_nr_last(buf);
            if t > save_nr_last {
                // Go to after last write: after the latest change. Use
                // the sequence number for that.
                t = nvim_buf_get_b_u_seq_last(buf) + 1;
                dofile = false;
            }
            (t, save_nr_last + 2)
        }
    } else {
        (
            nvim_buf_get_b_u_seq_cur(buf) + step,
            if step < 0 {
                -1
            } else {
                nvim_buf_get_b_u_seq_last(buf) + 2
            },
        )
    };

    // Adjust target and closest for step direction
    if !absolute {
        if step < 0 {
            if target < 0 {
                target = 0;
            }
            closest = -1;
        } else {
            // Recalculate closest for positive step
            closest = if dosec {
                (nvim_undo_os_time() + 1) as c_int
            } else if dofile {
                nvim_buf_get_b_u_save_nr_last(buf) + 2
            } else {
                nvim_buf_get_b_u_seq_last(buf) + 2
            };
            if target >= closest {
                target = closest - 1;
            }
        }
    }

    let closest_start = closest;
    let mut closest_seq = nvim_buf_get_b_u_seq_cur(buf);

    // When "target" is 0; Back to origin.
    if target == 0 {
        undo_time_to_target(buf, target, 0, 0, above, &mut did_undo);
        nvim_u_undo_end(did_undo, absolute, false);
        return;
    }

    // May do this twice:
    // 1. Search for "target", update "closest" to the best match found.
    // 2. If "target" not found search for "closest".
    //
    // When using the closest time we use the sequence number in the second
    // round, because there may be several entries with the same time.
    for round in 1..=2 {
        // Find the path from the current state to where we want to go. The
        // desired state can be anywhere in the undo tree, need to go all over
        // it. We put "nomark" in uh_walk where we have been without success,
        // "mark" where it could possibly be.
        let mark = nvim_inc_lastmark();
        let nomark = nvim_inc_lastmark();

        let curhead = nvim_buf_get_b_u_curhead(buf);
        let mut uhp = if curhead.0.is_null() {
            // at leaf of the tree
            nvim_buf_get_b_u_newhead(buf)
        } else {
            curhead
        };

        while !uhp.0.is_null() {
            nvim_uhp_set_walk(uhp, mark);
            let val = if dosec {
                nvim_uhp_get_time(uhp) as c_int
            } else if dofile {
                nvim_uhp_get_save_nr(uhp)
            } else {
                nvim_uhp_get_seq(uhp)
            };

            if round == 1 && !(dofile && val == 0) {
                // Remember the header that is closest to the target.
                // It must be at least in the right direction (checked with
                // "b_u_seq_cur"). When the timestamp is equal find the
                // highest/lowest sequence number.
                let uh_seq = nvim_uhp_get_seq(uhp);
                let seq_cur = nvim_buf_get_b_u_seq_cur(buf);
                let in_right_direction = if step < 0 {
                    uh_seq <= seq_cur
                } else {
                    uh_seq > seq_cur
                };

                if in_right_direction {
                    let is_closer = if dosec && val == closest {
                        if step < 0 {
                            uh_seq < closest_seq
                        } else {
                            uh_seq > closest_seq
                        }
                    } else if closest == closest_start {
                        true
                    } else if val > target {
                        if closest > target {
                            val - target <= closest - target
                        } else {
                            val - target <= target - closest
                        }
                    } else {
                        // val <= target
                        if closest > target {
                            target - val <= closest - target
                        } else {
                            target - val <= target - closest
                        }
                    };

                    if is_closer {
                        closest = val;
                        closest_seq = uh_seq;
                    }
                }
            }

            // Quit searching when we found a match. But when searching for a
            // time we need to continue looking for the best uh_seq.
            if target == val && !dosec {
                target = nvim_uhp_get_seq(uhp);
                break;
            }

            // go down in the tree if we haven't been there
            let prev = nvim_uhp_get_prev(uhp);
            if !prev.0.is_null()
                && nvim_uhp_get_walk(prev) != nomark
                && nvim_uhp_get_walk(prev) != mark
            {
                uhp = prev;
            } else {
                let alt_next = nvim_uhp_get_alt_next(uhp);
                if !alt_next.0.is_null()
                    && nvim_uhp_get_walk(alt_next) != nomark
                    && nvim_uhp_get_walk(alt_next) != mark
                {
                    // go to alternate branch if we haven't been there
                    uhp = alt_next;
                } else {
                    let next = nvim_uhp_get_next(uhp);
                    let alt_prev = nvim_uhp_get_alt_prev(uhp);
                    if !next.0.is_null()
                        && alt_prev.0.is_null()
                        && nvim_uhp_get_walk(next) != nomark
                        && nvim_uhp_get_walk(next) != mark
                    {
                        // go up in the tree if we haven't been there and we are at the
                        // start of alternate branches
                        // If still at the start we don't go through this change.
                        let curhead = nvim_buf_get_b_u_curhead(buf);
                        if uhp.0 == curhead.0 {
                            nvim_uhp_set_walk(uhp, nomark);
                        }
                        uhp = next;
                    } else {
                        // need to backtrack; mark this node as useless
                        nvim_uhp_set_walk(uhp, nomark);
                        if !alt_prev.0.is_null() {
                            uhp = alt_prev;
                        } else {
                            uhp = nvim_uhp_get_next(uhp);
                        }
                    }
                }
            }
        }

        if !uhp.0.is_null() {
            // found it
            break;
        }

        if absolute {
            nvim_semsg_undo_number_not_found(i64::from(step));
            return;
        }

        if closest == closest_start {
            if step < 0 {
                nvim_msg_oldest_change();
            } else {
                nvim_msg_newest_change();
            }
            return;
        }

        target = closest_seq;
        dosec = false;
        dofile = false;
        if step < 0 {
            above = true; // stop above the header
        }
    }

    // If we found it: Follow the path to go to where we want to be.
    undo_time_to_target(
        buf,
        target,
        nvim_inc_lastmark() - 2,
        nvim_inc_lastmark() - 2,
        above,
        &mut did_undo,
    );
    nvim_u_undo_end(did_undo, absolute, false);
}

/// Helper function to walk to target in undo tree.
/// This follows the path from the current state to the target state.
///
/// # Safety
///
/// Must be called with valid buffer handle and mark values.
unsafe fn undo_time_to_target(
    buf: BufHandle,
    target: c_int,
    mark: c_int,
    nomark: c_int,
    above: bool,
    did_undo: &mut bool,
) {
    // First go up the tree as much as needed.
    while !nvim_undo_got_int() {
        // Do the change warning now, for the same reason as above.
        nvim_change_warning_curbuf();

        let curhead = nvim_buf_get_b_u_curhead(buf);
        let uhp = if curhead.0.is_null() {
            nvim_buf_get_b_u_newhead(buf)
        } else {
            nvim_uhp_get_next(curhead)
        };

        if uhp.0.is_null()
            || (target > 0 && nvim_uhp_get_walk(uhp) != mark)
            || (nvim_uhp_get_seq(uhp) == target && !above)
        {
            break;
        }

        nvim_buf_set_b_u_curhead(buf, uhp);
        nvim_u_undoredo(true, true);
        if target > 0 {
            nvim_uhp_set_walk(uhp, nomark); // don't go back down here
        }
    }

    // When back to origin, redo is not needed.
    if target > 0 {
        // And now go down the tree (redo), branching off where needed.
        while !nvim_undo_got_int() {
            // Do the change warning now, for the same reason as above.
            nvim_change_warning_curbuf();

            let mut uhp = nvim_buf_get_b_u_curhead(buf);
            if uhp.0.is_null() {
                break;
            }

            // Go back to the first branch with a mark.
            let mut alt_prev = nvim_uhp_get_alt_prev(uhp);
            while !alt_prev.0.is_null() && nvim_uhp_get_walk(alt_prev) == mark {
                uhp = alt_prev;
                alt_prev = nvim_uhp_get_alt_prev(uhp);
            }

            // Find the last branch with a mark, that's the one.
            let mut last = uhp;
            let mut alt_next = nvim_uhp_get_alt_next(last);
            while !alt_next.0.is_null() && nvim_uhp_get_walk(alt_next) == mark {
                last = alt_next;
                alt_next = nvim_uhp_get_alt_next(last);
            }

            if last.0 != uhp.0 {
                // Make the used branch the first entry in the list of
                // alternatives to make "u" and CTRL-R take this branch.
                let mut first = uhp;
                let mut first_alt_prev = nvim_uhp_get_alt_prev(first);
                while !first_alt_prev.0.is_null() {
                    first = first_alt_prev;
                    first_alt_prev = nvim_uhp_get_alt_prev(first);
                }

                let last_alt_next = nvim_uhp_get_alt_next(last);
                if !last_alt_next.0.is_null() {
                    let last_alt_prev = nvim_uhp_get_alt_prev(last);
                    nvim_uhp_set_alt_prev(last_alt_next, last_alt_prev);
                }

                let last_alt_prev = nvim_uhp_get_alt_prev(last);
                nvim_uhp_set_alt_next(last_alt_prev, nvim_uhp_get_alt_next(last));
                nvim_uhp_set_alt_prev(last, UHeaderHandle(std::ptr::null_mut()));
                nvim_uhp_set_alt_next(last, first);
                nvim_uhp_set_alt_prev(first, last);

                let oldhead = nvim_buf_get_b_u_oldhead(buf);
                if oldhead.0 == first.0 {
                    nvim_buf_set_b_u_oldhead(buf, last);
                }

                uhp = last;
                let next = nvim_uhp_get_next(uhp);
                if !next.0.is_null() {
                    nvim_uhp_set_prev(next, uhp);
                }
            }

            nvim_buf_set_b_u_curhead(buf, uhp);

            if nvim_uhp_get_walk(uhp) != mark {
                break; // must have reached the target
            }

            // Stop when going backwards in time and didn't find the exact
            // header we were looking for.
            if nvim_uhp_get_seq(uhp) == target && above {
                nvim_buf_set_b_u_seq_cur(buf, target - 1);
                break;
            }

            nvim_u_undoredo(false, true);

            // Advance "curhead" to below the header we last used. If it
            // becomes NULL then we need to set "newhead" to this leaf.
            let prev = nvim_uhp_get_prev(uhp);
            if prev.0.is_null() {
                nvim_buf_set_b_u_newhead(buf, uhp);
            }
            nvim_buf_set_b_u_curhead(buf, prev);
            *did_undo = false;

            if nvim_uhp_get_seq(uhp) == target {
                // found it!
                break;
            }

            let prev = nvim_uhp_get_prev(uhp);
            if prev.0.is_null() || nvim_uhp_get_walk(prev) != mark {
                // Need to redo more but can't find it...
                nvim_internal_error_undo_time();
                break;
            }
        }
    }
}

// =============================================================================
// Phase 1: Undo Tree Traversal Helpers
// =============================================================================

/// Walk the undo tree and count the total number of undo headers.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_tree_count(buf: BufHandle) -> c_int {
    let mut count: c_int = 0;
    let mut uhp = nvim_buf_get_b_u_oldhead(buf);

    while !uhp.0.is_null() {
        count += 1;
        // Count alternate branches
        let mut alt = nvim_uhp_get_alt_next(uhp);
        while !alt.0.is_null() {
            count += rs_undo_branch_count(alt);
            alt = nvim_uhp_get_alt_next(alt);
        }
        uhp = nvim_uhp_get_prev(uhp);
    }

    count
}

/// Count headers in a branch (recursive helper).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_branch_count(uhp: UHeaderHandle) -> c_int {
    if uhp.0.is_null() {
        return 0;
    }

    let mut count: c_int = 1;
    let mut current = nvim_uhp_get_prev(uhp);

    while !current.0.is_null() {
        count += 1;
        // Count alternate branches
        let mut alt = nvim_uhp_get_alt_next(current);
        while !alt.0.is_null() {
            count += rs_undo_branch_count(alt);
            alt = nvim_uhp_get_alt_next(alt);
        }
        current = nvim_uhp_get_prev(current);
    }

    count
}

/// Find an undo header by sequence number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_find_seq(buf: BufHandle, seq: c_int) -> UHeaderHandle {
    let mut uhp = nvim_buf_get_b_u_newhead(buf);

    while !uhp.0.is_null() {
        if nvim_uhp_get_seq(uhp) == seq {
            return uhp;
        }

        // Check alternate branches
        let mut alt = nvim_uhp_get_alt_next(uhp);
        while !alt.0.is_null() {
            let found = rs_undo_find_seq_in_branch(alt, seq);
            if !found.0.is_null() {
                return found;
            }
            alt = nvim_uhp_get_alt_next(alt);
        }

        uhp = nvim_uhp_get_next(uhp);
    }

    UHeaderHandle(std::ptr::null_mut())
}

/// Find sequence number in a branch (recursive helper).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T or NULL.
unsafe fn rs_undo_find_seq_in_branch(uhp: UHeaderHandle, seq: c_int) -> UHeaderHandle {
    if uhp.0.is_null() {
        return UHeaderHandle(std::ptr::null_mut());
    }

    if nvim_uhp_get_seq(uhp) == seq {
        return uhp;
    }

    let mut current = nvim_uhp_get_prev(uhp);
    while !current.0.is_null() {
        if nvim_uhp_get_seq(current) == seq {
            return current;
        }

        // Check alternate branches
        let mut alt = nvim_uhp_get_alt_next(current);
        while !alt.0.is_null() {
            let found = rs_undo_find_seq_in_branch(alt, seq);
            if !found.0.is_null() {
                return found;
            }
            alt = nvim_uhp_get_alt_next(alt);
        }

        current = nvim_uhp_get_prev(current);
    }

    UHeaderHandle(std::ptr::null_mut())
}

/// Count the number of undo entries in a header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_count_entries(uhp: UHeaderHandle) -> c_int {
    if uhp.0.is_null() {
        return 0;
    }

    let mut count: c_int = 0;
    let mut uep = nvim_uhp_get_entry(uhp);

    while !uep.0.is_null() {
        count += 1;
        uep = nvim_uep_get_next(uep);
    }

    count
}

/// Get the depth of the undo tree (longest path from root to leaf).
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_tree_depth(buf: BufHandle) -> c_int {
    let oldhead = nvim_buf_get_b_u_oldhead(buf);
    rs_undo_get_branch_depth(oldhead)
}

/// Get the depth of a branch (recursive helper).
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_branch_depth(uhp: UHeaderHandle) -> c_int {
    if uhp.0.is_null() {
        return 0;
    }

    let mut max_depth: c_int = 0;

    // Check this branch
    let mut current = uhp;
    let mut depth: c_int = 0;
    while !current.0.is_null() {
        depth += 1;

        // Check alternate branches
        let mut alt = nvim_uhp_get_alt_next(current);
        while !alt.0.is_null() {
            let alt_depth = rs_undo_get_branch_depth(alt);
            if depth + alt_depth > max_depth {
                max_depth = depth + alt_depth;
            }
            alt = nvim_uhp_get_alt_next(alt);
        }

        current = nvim_uhp_get_prev(current);
    }

    if depth > max_depth {
        depth
    } else {
        max_depth
    }
}

/// Check if undo is available for the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_can_undo(buf: BufHandle) -> bool {
    let curhead = nvim_buf_get_b_u_curhead(buf);
    let newhead = nvim_buf_get_b_u_newhead(buf);

    // Can undo if curhead is NULL (first undo) and newhead exists
    // or if curhead exists and has a next header
    if curhead.0.is_null() {
        !newhead.0.is_null()
    } else {
        !nvim_uhp_get_next(curhead).0.is_null()
    }
}

/// Check if redo is available for the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_can_redo(buf: BufHandle) -> bool {
    let curhead = nvim_buf_get_b_u_curhead(buf);
    // Can redo if curhead exists (there's something to redo)
    !curhead.0.is_null()
}

/// Get the current undo sequence number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_seq_cur(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_seq_cur(buf)
}

/// Get the last undo sequence number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_seq_last(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_seq_last(buf)
}

/// Get the number of undo headers in the buffer.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_numhead(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_numhead(buf)
}

/// Get the current undo time.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_time_cur(buf: BufHandle) -> TimeT {
    nvim_buf_get_b_u_time_cur(buf)
}

/// Get the save number of the current header.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_save_nr_cur(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_save_nr_cur(buf)
}

/// Get the last save number.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_get_save_nr_last(buf: BufHandle) -> c_int {
    nvim_buf_get_b_u_save_nr_last(buf)
}

/// Check if the undo list is synced.
///
/// # Safety
///
/// The `buf` handle must be a valid pointer to a buf_T.
#[no_mangle]
pub unsafe extern "C" fn rs_undo_is_synced(buf: BufHandle) -> bool {
    nvim_buf_get_b_u_synced(buf)
}

// =============================================================================
// Phase 2: Undo Header Accessors
// =============================================================================

/// Get the sequence number from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_seq(uhp: UHeaderHandle) -> c_int {
    nvim_uhp_get_seq(uhp)
}

/// Get the time from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_time(uhp: UHeaderHandle) -> TimeT {
    nvim_uhp_get_time(uhp)
}

/// Get the flags from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_flags(uhp: UHeaderHandle) -> c_int {
    nvim_uhp_get_flags(uhp)
}

/// Get the save number from an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_save_nr(uhp: UHeaderHandle) -> c_int {
    nvim_uhp_get_save_nr(uhp)
}

/// Get the next header in the list.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_next(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_next(uhp)
}

/// Get the previous header in the list.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_prev(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_prev(uhp)
}

/// Get the next alternate header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_alt_next(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_alt_next(uhp)
}

/// Get the previous alternate header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_alt_prev(uhp: UHeaderHandle) -> UHeaderHandle {
    nvim_uhp_get_alt_prev(uhp)
}

/// Get the first entry in an undo header.
///
/// # Safety
///
/// The `uhp` handle must be a valid pointer to a u_header_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uhp_get_entry(uhp: UHeaderHandle) -> UEntryHandle {
    nvim_uhp_get_entry(uhp)
}

/// Check if an undo header is NULL.
#[no_mangle]
pub extern "C" fn rs_uhp_is_null(uhp: UHeaderHandle) -> bool {
    uhp.0.is_null()
}

/// Check if an undo entry is NULL.
#[no_mangle]
pub extern "C" fn rs_uep_is_null(uep: UEntryHandle) -> bool {
    uep.0.is_null()
}

// =============================================================================
// Phase 2: Undo Entry Accessors
// =============================================================================

/// Get the top line number from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_top(uep: UEntryHandle) -> LinenrT {
    nvim_uep_get_top(uep)
}

/// Get the bottom line number from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_bot(uep: UEntryHandle) -> LinenrT {
    nvim_uep_get_bot(uep)
}

/// Get the line count from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_lcount(uep: UEntryHandle) -> LinenrT {
    nvim_uep_get_lcount(uep)
}

/// Get the size (number of lines) from an undo entry.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_size(uep: UEntryHandle) -> LinenrT {
    nvim_uep_get_size(uep)
}

/// Get the next entry in the list.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_next(uep: UEntryHandle) -> UEntryHandle {
    nvim_uep_get_next(uep)
}

/// Get a line from the undo entry's array.
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
/// The index must be valid (0 <= idx < size).
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_line(uep: UEntryHandle, idx: LinenrT) -> *const c_char {
    nvim_uep_get_array_element(uep, idx)
}

/// Get the number of lines affected by an undo entry.
/// This is the number of lines that will be replaced (bot - top - 1).
///
/// # Safety
///
/// The `uep` handle must be a valid pointer to a u_entry_T.
#[no_mangle]
pub unsafe extern "C" fn rs_uep_get_line_count(uep: UEntryHandle) -> LinenrT {
    let top = nvim_uep_get_top(uep);
    let bot = nvim_uep_get_bot(uep);
    if bot == 0 {
        // Bot of 0 means end of buffer
        0
    } else {
        bot - top - 1
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

    #[test]
    fn test_null_handle_checks() {
        assert!(rs_uhp_is_null(UHeaderHandle(std::ptr::null_mut())));
        assert!(rs_uep_is_null(UEntryHandle(std::ptr::null_mut())));
    }
}
