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
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void};
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

    // --- Phase 3: stack allocation ---
    fn nvim_get_ql_info_actual() -> *mut c_void;
    fn nvim_qf_alloc_info() -> *mut c_void;
    fn nvim_qf_set_qi_type(qi: *mut c_void, qfltype: c_int);
    fn nvim_qf_set_maxcount(qi: *mut c_void, n: c_int);
    fn nvim_qf_set_new_lists(qi: *mut c_void, n: c_int);
    fn nvim_qf_incr_refcount(qi: *mut c_void);
    fn nvim_win_get_p_lhi(wp: *const c_void) -> c_int;
    fn nvim_win_get_llist_ref(wp: *const c_void) -> *mut c_void;
    fn nvim_qf_is_ll_window(wp: *const c_void) -> bool;
    fn nvim_qf_win_get_llist(wp: *const c_void) -> *mut c_void;
    fn nvim_qf_win_set_loclist(wp: *mut c_void, qi: *mut c_void);
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_qf_curwin_get_loclist() -> *mut c_void;
    fn nvim_is_loclist_cmd(cmdidx: c_int) -> bool;
    fn nvim_eap_get_cmdidx(eap: *const c_void) -> c_int;
    fn nvim_emsg_loclist();

    // --- Phase 4: set_errorlist + qf_free_stack ---
    /// Find the quickfix/location list window for a stack.
    fn nvim_qf_find_win_handle(qi: *const c_void) -> *const c_void;
    /// Call `qf_update_buffer(qi, old_last)`.
    fn nvim_qf_update_buffer(qi: *mut c_void, old_last: *mut c_void);
    /// Get curlist index (`qi->qf_curlist`).
    fn nvim_qf_get_curlist_idx(qi: *const c_void) -> c_int;
    /// Get the mutable curlist pointer.
    fn nvim_qf_get_curlist(qi: *const c_void) -> *const c_void;
    /// Set `qi->qf_curlist`.
    fn nvim_qf_set_curlist_idx(qi: *mut c_void, idx: c_int);
    /// Set `qi->qf_listcount`.
    fn nvim_qf_set_listcount(qi: *mut c_void, count: c_int);
    /// Find the window that owns a location list (via `w_llist`).
    fn nvim_qf_find_win_with_loclist(ll: *const c_void) -> *mut c_void;
    /// Return `win->w_buffer->b_fnum`.
    fn nvim_qf_win_buf_fnum(win: *const c_void) -> c_int;
    /// Set `wp->w_llist_ref = qi`.
    fn nvim_win_set_llist_ref(wp: *mut c_void, qi: *mut c_void);
    /// Return `tv_list_len(list)` -- 0 if list is NULL.
    fn nvim_tv_list_len(list: *const c_void) -> c_int;
    /// Emit "cannot have both a list and a 'what' argument" error.
    fn nvim_semsg_list_and_what();
    /// Call `qf_set_properties(qi, what, action, title)`.
    fn nvim_qf_set_properties(
        qi: *mut c_void,
        what: *const c_void,
        action: c_int,
        title: *mut c_void,
    ) -> c_int;
    /// Call `qf_list_changed(qfl)`.
    fn nvim_qf_list_changed(qfl: *mut c_void);
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

// =============================================================================
// Phase 3: Stack allocation and command stack resolution
// =============================================================================

/// C constants for `qfltype_T`.
const QFLT_QUICKFIX: c_int = 0;
const QFLT_LOCATION: c_int = 1;

/// Allocate (or return) a quickfix/location list stack.
///
/// For `QFLT_QUICKFIX` returns the address of the C static `ql_info_actual`.
/// For all other types heap-allocates a zeroed `qf_info_T`.
///
/// Corresponds to C `qf_alloc_stack`.
///
/// # Safety
///
/// `n` must be > 0 and represent the desired maximum list count.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_alloc_stack(qfltype: c_int, n: c_int) -> *mut c_void {
    let qi: *mut c_void = if qfltype == QFLT_QUICKFIX {
        nvim_get_ql_info_actual()
    } else {
        let p = nvim_qf_alloc_info();
        nvim_qf_incr_refcount(p);
        p
    };

    nvim_qf_set_qi_type(qi, qfltype);
    nvim_qf_set_bufnr(qi, INVALID_QFBUFNR);
    nvim_qf_set_new_lists(qi, n);
    nvim_qf_set_maxcount(qi, n);

    qi
}

/// Get or allocate the location list for window `wp`.
///
/// - If `wp` is a location list window, returns its `w_llist_ref`.
/// - Otherwise, frees any stale `w_llist_ref`, allocates a new location list
///   if `w_llist` is NULL, and returns `w_llist`.
///
/// Corresponds to C `ll_get_or_alloc_list`.
///
/// # Safety
///
/// `wp` must be a valid non-null `*mut win_T`.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_ll_get_or_alloc_list(wp: *mut c_void) -> *mut c_void {
    if nvim_qf_is_ll_window(wp) {
        // For a location list window, use the referenced location list.
        return nvim_win_get_llist_ref(wp);
    }

    // For a non-location list window, w_llist_ref should not point anywhere.
    let mut llist_ref = nvim_win_take_llist_ref(wp);
    if !llist_ref.is_null() {
        rs_ll_free_all(std::ptr::addr_of_mut!(llist_ref));
    }

    let llist = nvim_qf_win_get_llist(wp);
    if llist.is_null() {
        // Allocate a new location list.
        let n = nvim_win_get_p_lhi(wp);
        let new_qi = rs_qf_alloc_stack(QFLT_LOCATION, n);
        nvim_qf_win_set_loclist(wp, new_qi);
    }

    nvim_qf_win_get_llist(wp)
}

/// Get the quickfix/location list stack for an Ex command.
///
/// Returns NULL and optionally emits E776 if the current window has no
/// location list.
///
/// Corresponds to C `qf_cmd_get_stack`.
///
/// # Safety
///
/// `eap` must be a valid non-null `*const exarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_cmd_get_stack(eap: *mut c_void, print_emsg: bool) -> *mut c_void {
    let mut qi = nvim_get_ql_info();

    if nvim_is_loclist_cmd(nvim_eap_get_cmdidx(eap)) {
        qi = nvim_qf_curwin_get_loclist();
        if qi.is_null() {
            if print_emsg {
                nvim_emsg_loclist();
            }
            return std::ptr::null_mut();
        }
    }

    qi
}

/// Get or allocate the quickfix/location list stack for an Ex command.
///
/// For location list commands, sets `*pwinp = curwin`.
///
/// Corresponds to C `qf_cmd_get_or_alloc_stack`.
///
/// # Safety
///
/// `eap` must be a valid non-null `*const exarg_T`.
/// `pwinp` must be a valid non-null `*mut *mut win_T`.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned (should never happen in practice).
#[no_mangle]
pub unsafe extern "C" fn rs_qf_cmd_get_or_alloc_stack(
    eap: *const c_void,
    pwinp: *mut *mut c_void,
) -> *mut c_void {
    let mut qi = nvim_get_ql_info();

    if nvim_is_loclist_cmd(nvim_eap_get_cmdidx(eap)) {
        let curwin = nvim_get_curwin();
        qi = rs_ll_get_or_alloc_list(curwin);
        *pwinp = curwin;
    }

    qi
}

// =============================================================================
// Phase 4: set_errorlist and qf_free_stack orchestrators
// =============================================================================

/// C OK / FAIL constants (mirror `nvim_c_decls.h`).
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Free the entire quickfix/location list stack, including updating any open
/// quickfix window and re-assigning an empty stack to the window.
///
/// Corresponds to C `qf_free_stack`.
///
/// # Safety
///
/// `wp` must be NULL or a valid `*mut win_T`.
/// `qi` must be a valid non-null `*mut qf_info_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_stack(wp: *mut c_void, qi: *mut c_void) {
    // nvim_qf_find_win_handle returns *const but the window pointer is mutable;
    // cast to *mut so we can pass it to functions that require *mut.
    let qfwin = nvim_qf_find_win_handle(qi).cast_mut();

    if !qfwin.is_null() {
        // Quickfix/location window is open: free the current list's items so
        // the buffer gets cleared, then trigger a buffer update.
        if nvim_qf_get_curlist_idx(qi) < nvim_qf_get_listcount(qi) {
            rs_qf_free_list(nvim_qf_get_curlist(qi).cast_mut());
        }
        nvim_qf_update_buffer(qi, std::ptr::null_mut());
    }

    // If wp is a location list window, redirect to the normal window that owns
    // this location list (if there is one).
    let wp = if !wp.is_null() && nvim_qf_is_ll_window(wp) {
        let llwin = nvim_qf_find_win_with_loclist(qi);
        if llwin.is_null() {
            wp
        } else {
            llwin
        }
    } else {
        wp
    };

    rs_qf_free_all(wp);

    if wp.is_null() {
        // Global quickfix list: reset counters (struct is static, not freed).
        nvim_qf_set_curlist_idx(qi, 0);
        nvim_qf_set_listcount(qi, 0);
    } else if !qfwin.is_null() {
        // Location list window is open: create a new empty location list for
        // both the source window and the location list window.
        let n = nvim_win_get_p_lhi(wp);
        let new_ll = rs_qf_alloc_stack(QFLT_LOCATION, n);

        // Record the quickfix window's buffer number in the new stack.
        nvim_qf_set_bufnr(new_ll, nvim_qf_win_buf_fnum(qfwin));

        // Free the old llist_ref in the location list window.
        let mut old_ref = nvim_win_take_llist_ref(qfwin);
        if !old_ref.is_null() {
            rs_ll_free_all(std::ptr::addr_of_mut!(old_ref));
        }

        // Assign new_ll to the location list window as its llist_ref.
        nvim_win_set_llist_ref(qfwin, new_ll);

        // If the source window is not the location list window itself, also
        // update its w_llist.
        if wp != qfwin {
            nvim_qf_win_set_loclist(wp, new_ll);
        }
    }
}

/// Top-level API for `setqflist()`/`setloclist()` `VimL` functions.
///
/// Corresponds to C `set_errorlist`.
///
/// - `wp == NULL`: operate on the global quickfix list.
/// - `wp != NULL`: operate on `wp`'s location list.
/// - `action == 'f'`: free the entire stack.
/// - `action == 'a'`/`'r'`/`'u'`: add/replace/undo entries.
/// - `what != NULL`: set properties via `qf_set_properties`.
///
/// Returns `OK` (1) on success, `FAIL` (0) on error.
///
/// # Safety
///
/// All pointer arguments must be valid (or NULL where noted).
#[no_mangle]
pub unsafe extern "C" fn rs_set_errorlist(
    wp: *mut c_void,
    list: *mut c_void,
    action: c_int,
    title: *mut c_char,
    what: *mut c_void,
) -> c_int {
    let qi = if wp.is_null() {
        nvim_get_ql_info()
    } else {
        rs_ll_get_or_alloc_list(wp)
    };
    debug_assert!(!qi.is_null(), "set_errorlist: qi must not be NULL");

    if action == c_int::from(b'f') {
        // Free the entire quickfix or location list stack.
        rs_qf_free_stack(wp, qi);
        return OK;
    }

    // A dict argument cannot be combined with a non-empty list argument.
    if !list.is_null() && nvim_tv_list_len(list) != 0 && !what.is_null() {
        nvim_semsg_list_and_what();
        return FAIL;
    }

    rs_incr_quickfix_busy();

    let retval = if what.is_null() {
        let retval = rs_qf_add_entries(qi, nvim_qf_get_curlist_idx(qi), list, title, action);
        if retval == OK {
            nvim_qf_list_changed(nvim_qf_get_curlist(qi).cast_mut());
        }
        retval
    } else {
        nvim_qf_set_properties(qi, what, action, title.cast())
    };

    rs_decr_quickfix_busy();

    retval
}

// Declare rs_qf_add_entries (defined in list.rs / Rust crate) so we can call
// it from this module without an `extern` block inside a function.
extern "C" {
    fn rs_qf_add_entries(
        qi: *mut c_void,
        qf_idx: c_int,
        list: *const c_void,
        title: *const c_char,
        action: c_int,
    ) -> c_int;
}
