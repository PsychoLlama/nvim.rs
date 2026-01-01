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
