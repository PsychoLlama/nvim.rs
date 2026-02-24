//! Quickfix lifecycle management.
//!
//! Owns the `quickfix_busy` counter, the deferred location list deletion queue,
//! and the stack/list free logic (`ll_free_all`, `qf_free_all`,
//! `wipe_qf_buffer`, `qf_free_lists`).
//!
//! Phase 1 of the migration:
//!   - Rust owns `quickfix_busy` (previously a C static `int`)
//!   - Rust owns the deferred deletion queue (`qf_delq_head`)
//!   - C callers use `nvim_incr_quickfix_busy()`/`nvim_decr_quickfix_busy()`
//!
//! Phase 2 of the migration (same commit):
//!   - `rs_wipe_qf_buffer`, `rs_ll_free_all`, `rs_qf_free_all` replace the
//!     corresponding C static functions.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_int, c_void};
use std::sync::Mutex;

// =============================================================================
// Constants (mirror C defines)
// =============================================================================

/// `INVALID_QFBUFNR` in C
const INVALID_QFBUFNR: c_int = 0;

// =============================================================================
// Global state (previously C statics)
// =============================================================================

/// The busy counter.  Protected by a mutex even though Neovim is
/// single-threaded, because Rust's statics require `Sync`.
static QUICKFIX_BUSY: Mutex<i32> = Mutex::new(0);

/// Deferred deletion queue: pointers to `qf_info_T` that need to be freed
/// once `QUICKFIX_BUSY` reaches zero.
///
/// Safety note: these pointers are only ever accessed from the Neovim main
/// thread (single-threaded event loop), so we wrap them in a newtype that is
/// `Send`.
struct RawPtr(*mut c_void);
// SAFETY: Neovim's event loop is single-threaded.
unsafe impl Send for RawPtr {}

static DELQ: Mutex<Vec<RawPtr>> = Mutex::new(Vec::new());

// =============================================================================
// External C accessor functions
// =============================================================================

extern "C" {
    // --- qf_info_T (stack) accessors ---
    fn nvim_qf_get_refcount(qi: *const c_void) -> c_int;
    fn nvim_qf_set_refcount(qi: *mut c_void, v: c_int);
    fn nvim_qf_get_listcount(qi: *const c_void) -> c_int;
    fn nvim_qf_get_bufnr(qi: *const c_void) -> c_int;
    fn nvim_qf_set_bufnr(qi: *mut c_void, bufnr: c_int);
    fn nvim_qf_get_list_at(qi: *const c_void, idx: c_int) -> *const c_void;

    // --- rs_qf_free_list (already in Rust) ---
    fn rs_qf_free_list(qfl: *mut c_void);

    // --- Memory management ---
    /// Free the `qf_lists` array inside a `qf_info_T`.
    fn nvim_qf_free_lists_array(qi: *mut c_void);
    /// Free the `qf_info_T` struct itself (only for heap-allocated ones).
    fn nvim_qf_free_info(qi: *mut c_void);

    // --- Buffer management (for `wipe_qf_buffer`) ---
    fn nvim_buflist_findnr(bufnr: c_int) -> *mut c_void;
    fn nvim_buf_get_nwindows(buf: *const c_void) -> c_int;
    fn nvim_curwin_get_buffer() -> *mut c_void;
    fn nvim_curwin_set_buffer(buf: *mut c_void);
    fn nvim_get_curbuf() -> *mut c_void;
    fn nvim_close_buffer_wipe(buf: *mut c_void);

    // --- Window / location list accessors ---
    /// Set wp->w_llist = NULL and return old value.
    fn nvim_win_take_llist(wp: *mut c_void) -> *mut c_void;
    /// Set wp->w_llist_ref = NULL and return old value.
    fn nvim_win_take_llist_ref(wp: *mut c_void) -> *mut c_void;
    fn nvim_get_ql_info() -> *mut c_void;
}

// =============================================================================
// Phase 1: Busy counter and deferred deletion queue
// =============================================================================

/// Increment the quickfix-busy counter.
///
/// # Safety
///
/// Must be called from the Neovim main thread.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_incr_quickfix_busy() {
    let mut busy = QUICKFIX_BUSY.lock().unwrap();
    *busy += 1;
}

/// Decrement the quickfix-busy counter and process deferred deletions.
///
/// # Safety
///
/// Must be called from the Neovim main thread.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_decr_quickfix_busy() {
    let queue_to_process: Vec<RawPtr> = {
        let mut busy = QUICKFIX_BUSY.lock().unwrap();
        *busy -= 1;

        if *busy == 0 {
            let mut delq = DELQ.lock().unwrap();
            std::mem::take(&mut *delq)
        } else {
            Vec::new()
        }
    };

    // Process outside the lock so re-entrant ll_free_all calls can check the
    // counter (now 0) and not re-queue.
    for RawPtr(mut ptr) in queue_to_process {
        rs_ll_free_all(std::ptr::addr_of_mut!(ptr));
    }
}

/// Queue a location list stack for deferred deletion.
///
/// # Safety
///
/// `qi` must be a valid `*mut qf_info_T` or NULL.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_locstack_queue_delreq(qi: *mut c_void) {
    if qi.is_null() {
        return;
    }
    let mut delq = DELQ.lock().unwrap();
    delq.push(RawPtr(qi));
}

/// Assert that the quickfix-busy counter is zero on exit (EXITFREE path).
///
/// # Safety
///
/// Must be called from the Neovim main thread during shutdown.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_check_quickfix_busy() {
    let busy = *QUICKFIX_BUSY.lock().unwrap();
    if busy != 0 {
        // Cannot use Rust formatting machinery in EXITFREE context; call C semsg.
        extern "C" {
            fn semsg(fmt: *const u8, ...);
        }
        // Use a null-terminated string literal via concat!+as_ptr.
        let fmt = concat!("quickfix_busy not zero on exit: %ld", "\0");
        semsg(fmt.as_ptr(), i64::from(busy));
        #[cfg(debug_assertions)]
        {
            extern "C" {
                fn abort() -> !;
            }
            abort();
        }
    }
}

// =============================================================================
// Phase 2: Stack free and location list free
// =============================================================================

/// Wipe the quickfix buffer if it is not displayed in any window.
///
/// Corresponds to C `wipe_qf_buffer`.
///
/// # Safety
///
/// `qi` must be a valid non-null `*mut qf_info_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_wipe_qf_buffer(qi: *mut c_void) {
    let bufnr = nvim_qf_get_bufnr(qi);
    if bufnr == INVALID_QFBUFNR {
        return;
    }

    let qfbuf = nvim_buflist_findnr(bufnr);
    if qfbuf.is_null() {
        return;
    }

    if nvim_buf_get_nwindows(qfbuf) != 0 {
        return;
    }

    // When curwin->w_buffer is NULL (e.g. during win_free_mem), close_buffer()
    // requires curwin->w_buffer == curbuf.  Temporarily restore it.
    let saved_w_buffer = nvim_curwin_get_buffer();
    let buf_was_null = saved_w_buffer.is_null();
    if buf_was_null {
        nvim_curwin_set_buffer(nvim_get_curbuf());
    }

    nvim_close_buffer_wipe(qfbuf);
    nvim_qf_set_bufnr(qi, INVALID_QFBUFNR);

    if buf_was_null {
        nvim_curwin_set_buffer(std::ptr::null_mut());
    }
}

/// Free a `qf_info_T` struct completely (all lists, then the struct itself).
///
/// # Safety
///
/// `qi` must be a valid non-null `*mut qf_info_T`.  After this call the
/// pointer is invalid.
unsafe fn free_lists_and_info(qi: *mut c_void) {
    let count = nvim_qf_get_listcount(qi);
    for i in 0..count {
        let qfl = nvim_qf_get_list_at(qi, i).cast_mut();
        if !qfl.is_null() {
            rs_qf_free_list(qfl);
        }
    }
    nvim_qf_free_lists_array(qi);
    nvim_qf_free_info(qi);
}

/// Free a location list stack, respecting the busy counter.
///
/// Corresponds to C `ll_free_all`.  Sets `*pqi = NULL` on entry.
///
/// # Safety
///
/// `pqi` must be a valid non-null pointer to a `*mut qf_info_T` (or a null
/// pointer slot, in which case this is a no-op).
///
/// # Panics
///
/// Panics if the internal mutex is poisoned (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_ll_free_all(pqi: *mut *mut c_void) {
    let qi = *pqi;
    if qi.is_null() {
        return;
    }
    // Remove the reference first so recursive calls won't double-free.
    *pqi = std::ptr::null_mut();

    let busy = *QUICKFIX_BUSY.lock().unwrap();
    if busy > 0 {
        // Defer until busy == 0.
        rs_locstack_queue_delreq(qi);
        return;
    }

    let new_refcount = nvim_qf_get_refcount(qi) - 1;
    nvim_qf_set_refcount(qi, new_refcount);
    if new_refcount >= 1 {
        return; // Still referenced elsewhere.
    }

    // No more references – wipe buffer and free everything.
    rs_wipe_qf_buffer(qi);
    free_lists_and_info(qi);
}

/// Free all quickfix/location lists for a window, or the global quickfix list.
///
/// Corresponds to C `qf_free_all`.
///
/// - `wp == NULL`: free the global quickfix stack's list contents (but keep
///   the `qf_info_T` itself, which is statically allocated).
/// - `wp != NULL`: free the window's location list stacks.
///
/// # Safety
///
/// If non-null, `wp` must be a valid `*mut win_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_all(wp: *mut c_void) {
    if wp.is_null() {
        // Global quickfix list: free list contents but not the struct itself
        // (ql_info_actual is a C static, not heap-allocated).
        let qi = nvim_get_ql_info();
        if qi.is_null() {
            return;
        }
        let count = nvim_qf_get_listcount(qi);
        for i in 0..count {
            let qfl = nvim_qf_get_list_at(qi, i).cast_mut();
            if !qfl.is_null() {
                rs_qf_free_list(qfl);
            }
        }
    } else {
        // Location list: atomically take w_llist and w_llist_ref, then free.
        // nvim_win_take_llist sets wp->w_llist = NULL and returns the old value,
        // mirroring what ll_free_all(&wp->w_llist) does (sets *pqi = NULL first).
        let mut llist = nvim_win_take_llist(wp);
        if !llist.is_null() {
            rs_ll_free_all(std::ptr::addr_of_mut!(llist));
        }
        let mut llist_ref = nvim_win_take_llist_ref(wp);
        if !llist_ref.is_null() {
            rs_ll_free_all(std::ptr::addr_of_mut!(llist_ref));
        }
    }
}
