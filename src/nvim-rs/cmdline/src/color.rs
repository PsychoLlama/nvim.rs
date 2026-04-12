//! Command-line coloring (`nvim_color_cmdline`).
//!
//! Rust implementation of the command-line syntax coloring function.
//! C helpers in ex_getln.c handle the VimL parser, TRY_WRAP, Callback type,
//! and kvec manipulation. Rust handles the outer coordination and list iteration.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

// =============================================================================
// State
// =============================================================================

/// Previous prompt_id (for error-rate tracking). UINT_MAX means "never set".
static PREV_PROMPT_ID: AtomicU32 = AtomicU32::new(u32::MAX);
/// Number of consecutive callback errors for the current prompt.
static PREV_PROMPT_ERRORS: AtomicI32 = AtomicI32::new(0);

// =============================================================================
// VarType constants
// =============================================================================

/// VAR_LIST - list value type constant (from typval_defs.h)
const VAR_LIST: c_int = 5;

// =============================================================================
// Opaque pointer types
// =============================================================================

/// Opaque pointer to listitem_T.
type ListItemPtr = *mut c_void;
/// Opaque pointer to list_T (const).
type ListPtr = *const c_void;

// =============================================================================
// listitem_T layout mirror
// =============================================================================
//
// listitem_T layout (from eval/typval.h):
//   offset  0: li_next (*mut c_void)   - next item pointer
//   offset  8: li_prev (*mut c_void)   - prev item pointer
//   offset 16: li_tv.v_type (c_int)    - VarType
//   offset 20: li_tv.v_lock (c_char)   - VarLock + 3 padding bytes
//   offset 24: li_tv.vval (8 bytes)    - union: v_list/*list_T, v_string/*char, v_number/i64
//
// typval_T total size: 16 bytes (v_type 4 + v_lock 1 + pad 3 + vval 8).
// listitem_T total size: 32 bytes.

#[repr(C)]
struct ListItemT {
    li_next: *mut c_void, // offset 0
    li_prev: *mut c_void, // offset 8
    tv_type: c_int,       // offset 16 (= li_tv.v_type)
    _tv_lock_pad: u32,    // offset 20 (= li_tv.v_lock + 3 padding bytes)
    tv_vval: TvVal,       // offset 24 (= li_tv.vval)
}

#[repr(C)]
union TvVal {
    v_list: *mut c_void,
    v_string: *mut c_char,
    v_number: i64,
}

// =============================================================================
// C extern declarations
// =============================================================================

unsafe extern "C" {
    // Cache validity: returns 1 if color cache is still valid.
    fn nvim_color_cache_valid() -> c_int;
    // Reset colors kvec: sets kv_size = 0 and frees cached cmdbuff.
    fn nvim_ccline_reset_colors();
    // Returns 1 if cmdbuff is NULL or empty.
    fn nvim_color_is_empty() -> c_int;
    // Get current prompt_id (ccline.prompt_id).
    fn nvim_get_ccline_prompt_id() -> u32;
    // Acquire the coloring callback (stores internally in C static state).
    // Returns: 0=no callback, 1=highlight_callback, 2=ex callback(':'), 3=expr path('=').
    // On error returns -1 and stores error internally.
    fn nvim_color_acquire_callback() -> c_int;
    // Run the full '=' expression coloring path. Updates ccline.last_colors.colors.
    fn nvim_color_run_expr_coloring();
    // Invoke the previously acquired callback. Returns 1=ok, 0=failed.
    fn nvim_color_run_callback_coloring() -> c_int;
    // Get v_type of the result typval (VAR_LIST=5, VAR_UNKNOWN=0, etc).
    fn nvim_color_result_tv_type() -> c_int;
    // Get v_list pointer from result typval (may be NULL).
    fn nvim_color_result_tv_list() -> ListPtr;
    // Print an error message using PRINT_ERRMSG semantics (msg_putchar + smsg + HLF_E).
    fn nvim_color_errmsg(msg: *const c_char);
    // Push one color chunk to ccline.last_colors.colors.
    fn nvim_ccline_colors_push(start: c_int, end: c_int, hl_id: c_int);
    // Finalize coloring: success=1 updates cache; success=0 prints error, clears colors, redraws.
    fn nvim_color_finalize(success: c_int);
    // Get ccline.cmdlen.
    fn nvim_color_cmdlen() -> c_int;
    // Get one byte from ccline.cmdbuff at position idx.
    fn nvim_color_cmdbuff_at(idx: c_int) -> u8;
    // nvim_color_tv_list_len: wrapper for tv_list_len (inline) - get length of a list.
    fn nvim_color_tv_list_len(l: ListPtr) -> c_int;
    // nvim_list_get_first: get first item of a list (may be NULL for empty list).
    fn nvim_list_get_first(l: ListPtr) -> ListItemPtr;
    // nvim_list_get_last: get last item of a list.
    fn nvim_list_get_last(l: ListPtr) -> ListItemPtr;
    // nvim_color_tv_get_number_chk: wrapper for tv_get_number_chk (inline).
    fn nvim_color_tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;
    // nvim_color_tv_get_string_chk: wrapper for tv_get_string_chk (inline).
    fn nvim_color_tv_get_string_chk(tv: *const c_void) -> *const c_char;
    // syn_name2id: get highlight group id by name.
    fn syn_name2id(name: *const c_char) -> c_int;

    // Rust helpers for prompt error tracking (in ui.rs).
    fn rs_should_skip_coloring(
        current_prompt_id: u32,
        prev_prompt_id: u32,
        prev_errors: c_int,
    ) -> c_int;
    fn rs_should_reset_callback_errors(current_prompt_id: u32, prev_prompt_id: u32) -> c_int;
}

// =============================================================================
// List item helpers
// =============================================================================

/// Pointer to the `li_tv` field (typval_T) of a listitem_T.
/// `li_tv` is at byte offset 16 within a listitem_T.
#[inline]
const unsafe fn list_item_tv(item: ListItemPtr) -> *const c_void {
    item.add(16)
}

/// Next listitem pointer (reads `li_next` at offset 0).
#[inline]
unsafe fn list_item_next(item: ListItemPtr) -> ListItemPtr {
    (*item.cast::<ListItemT>()).li_next
}

/// `v_type` of the embedded typval_T (reads at offset 16).
#[inline]
unsafe fn item_tv_type(item: ListItemPtr) -> c_int {
    (*item.cast::<ListItemT>()).tv_type
}

/// `vval.v_list` of the embedded typval_T (reads at offset 24).
#[inline]
unsafe fn item_tv_list(item: ListItemPtr) -> ListPtr {
    (*item.cast::<ListItemT>()).tv_vval.v_list.cast_const()
}

// =============================================================================
// UTF-8 helper
// =============================================================================

/// Returns true if `byte` is a UTF-8 continuation byte (0x80..0xBF).
/// Equivalent to `utf8len_tab_zero[byte] == 0` for the multibyte split checks.
#[inline]
const fn is_utf8_continuation(byte: u8) -> bool {
    (byte & 0xC0) == 0x80
}

// =============================================================================
// nvim_color_cmdline
// =============================================================================

/// Rust implementation of `nvim_color_cmdline()`.
///
/// Colors the command-line using either a user callback (`g:Nvim_color_cmdline`,
/// `highlight_callback`) or the VimL expression parser (for `=` prompt).
/// Caches results: returns early if prompt and buffer are unchanged.
///
/// # Safety
///
/// Must only be called from the main Neovim thread when the command line is
/// active. Calls many C functions that mutate global state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nvim_color_cmdline() -> bool {
    // --- Cache check ---
    // If the prompt_id and cmdbuff haven't changed, reuse cached colors.
    if nvim_color_cache_valid() != 0 {
        return true;
    }

    // --- Reset colors ---
    nvim_ccline_reset_colors();

    // --- Early exit for empty buffer ---
    if nvim_color_is_empty() != 0 {
        return true;
    }

    // --- Error tracking for callbacks ---
    let current_prompt_id = nvim_get_ccline_prompt_id();
    let prev_id = PREV_PROMPT_ID.load(Ordering::Relaxed);
    let prev_errors = PREV_PROMPT_ERRORS.load(Ordering::Relaxed);

    if rs_should_reset_callback_errors(current_prompt_id, prev_id) != 0 {
        PREV_PROMPT_ERRORS.store(0, Ordering::Relaxed);
        PREV_PROMPT_ID.store(current_prompt_id, Ordering::Relaxed);
    } else if rs_should_skip_coloring(current_prompt_id, prev_id, prev_errors) != 0 {
        // Too many consecutive errors - skip coloring, finalize with empty colors.
        nvim_color_finalize(1);
        return true;
    }

    // --- Acquire callback ---
    // Returns: -1=error, 0=none, 1=highlight_callback, 2=ex_callback, 3=expr_path
    let acquire = nvim_color_acquire_callback();

    match acquire {
        -1 => {
            // Error occurred during callback acquisition.
            return finalize_error();
        }
        3 => {
            // '=' expression path: entirely handled in C.
            nvim_color_run_expr_coloring();
            nvim_color_finalize(1);
            return true;
        }
        0 => {
            // No callback available.
            nvim_color_finalize(1);
            return true;
        }
        1 | 2 => {} // callback acquired, fall through to invoke
        _ => {
            nvim_color_finalize(1);
            return true;
        }
    }

    // --- Invoke callback ---
    if nvim_color_run_callback_coloring() == 0 {
        return finalize_error();
    }

    // --- Validate return type ---
    if nvim_color_result_tv_type() != VAR_LIST {
        nvim_color_errmsg(c"E5400: Callback should return list".as_ptr());
        return finalize_error();
    }

    let list = nvim_color_result_tv_list();
    if list.is_null() {
        nvim_color_finalize(1);
        return true;
    }

    // --- Process list of color chunks ---
    if !process_color_list(list) {
        return finalize_error();
    }

    nvim_color_finalize(1);
    true
}

/// Validate and process the list of `[start, end, hlgroup]` chunks returned
/// by the coloring callback.
///
/// Returns `false` if any chunk fails validation (error message already printed).
unsafe fn process_color_list(list: ListPtr) -> bool {
    let cmdlen = nvim_color_cmdlen();
    let mut prev_end: i64 = 0;
    let mut i: c_int = 0;
    let mut outer = nvim_list_get_first(list);

    while !outer.is_null() {
        // Each outer item must be a VAR_LIST (a list of 3 elements).
        if item_tv_type(outer) != VAR_LIST {
            let msg = format!("E5401: List item {i} is not a List\0");
            nvim_color_errmsg(msg.as_ptr().cast());
            return false;
        }

        let inner = item_tv_list(outer);
        let inner_len = if inner.is_null() {
            0
        } else {
            nvim_color_tv_list_len(inner)
        };
        if inner_len != 3 {
            let msg = format!("E5402: List item {i} has incorrect length: {inner_len} /= 3\0");
            nvim_color_errmsg(msg.as_ptr().cast());
            return false;
        }

        // Get the 3 elements: start, end, hlgroup.
        let first = nvim_list_get_first(inner);
        let last_item = nvim_list_get_last(inner);
        if first.is_null() || last_item.is_null() {
            return false;
        }
        let second = list_item_next(first);
        if second.is_null() {
            return false;
        }

        // --- Validate start ---
        let mut error = false;
        let start = nvim_color_tv_get_number_chk(list_item_tv(first), &raw mut error);
        if error {
            return false;
        }
        if !(prev_end <= start && start < i64::from(cmdlen)) {
            let msg =
                format!("E5403: Chunk {i} start {start} not in range [{prev_end}, {cmdlen})\0");
            nvim_color_errmsg(msg.as_ptr().cast());
            return false;
        }
        if is_utf8_continuation(nvim_color_cmdbuff_at(start as c_int)) {
            let msg = format!("E5405: Chunk {i} start {start} splits multibyte character\0");
            nvim_color_errmsg(msg.as_ptr().cast());
            return false;
        }

        // Fill any gap before this chunk with hl_id=0.
        if start != prev_end {
            nvim_ccline_colors_push(prev_end as c_int, start as c_int, 0);
        }

        // --- Validate end ---
        let end = nvim_color_tv_get_number_chk(list_item_tv(second), &raw mut error);
        if error {
            return false;
        }
        if !(start < end && end <= i64::from(cmdlen)) {
            let msg = format!("E5404: Chunk {i} end {end} not in range ({start}, {cmdlen}]\0");
            nvim_color_errmsg(msg.as_ptr().cast());
            return false;
        }
        if end < i64::from(cmdlen) && is_utf8_continuation(nvim_color_cmdbuff_at(end as c_int)) {
            let msg = format!("E5406: Chunk {i} end {end} splits multibyte character\0");
            nvim_color_errmsg(msg.as_ptr().cast());
            return false;
        }

        prev_end = end;

        // --- Get highlight group ---
        let group = nvim_color_tv_get_string_chk(list_item_tv(last_item));
        if group.is_null() {
            return false;
        }
        let hl_id = syn_name2id(group);

        nvim_ccline_colors_push(start as c_int, end as c_int, hl_id);

        outer = list_item_next(outer);
        i += 1;
    }

    // Fill trailing gap.
    if prev_end < i64::from(cmdlen) {
        nvim_ccline_colors_push(prev_end as c_int, cmdlen, 0);
    }

    true
}

/// Finalize the error path: update error counter and return false.
unsafe fn finalize_error() -> bool {
    PREV_PROMPT_ERRORS.fetch_add(1, Ordering::Relaxed);
    nvim_color_finalize(0);
    false
}
