//! Command-line coloring (`nvim_color_cmdline`).
//!
//! Rust implementation of the command-line syntax coloring function.
//! C helpers in ex_getln.c handle the VimL parser, TRY_WRAP (Callback acquisition
//! and invocation), kvec manipulation, and variadic smsg. Rust handles the outer
//! coordination, list iteration, and finalize bookkeeping.
//!
//! `ccs` (ColorCmdlineHelperState) storage lives in Rust as `static mut ccs: [u8; 96]`
//! with `#[unsafe(no_mangle)]` so the C `extern ColorCmdlineHelperState ccs;` links to it.
//! C accessor functions (`nvim_color_ccs_*`) expose individual fields to Rust.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use libc;

// =============================================================================
// State
// =============================================================================

/// Previous prompt_id (for error-rate tracking). UINT_MAX means "never set".
static PREV_PROMPT_ID: AtomicU32 = AtomicU32::new(u32::MAX);
/// Number of consecutive callback errors for the current prompt.
static PREV_PROMPT_ERRORS: AtomicI32 = AtomicI32::new(0);

// =============================================================================
// CCS storage
// =============================================================================

/// sizeof(ColorCmdlineHelperState) verified by _Static_assert in ex_getln.c.
const CCS_SIZE: usize = 96;

/// Storage for ColorCmdlineHelperState `ccs` used by C helper functions.
///
/// C declares `extern ColorCmdlineHelperState ccs;` which links to this symbol.
/// All field access is done via `nvim_color_ccs_*` C accessor functions.
///
/// SAFETY: only accessed from the main Neovim thread; never from multiple threads.
#[unsafe(no_mangle)]
static mut ccs: [u8; CCS_SIZE] = [0u8; CCS_SIZE];

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
    // --- ccline accessors ---
    fn nvim_get_ccline_last_colors_prompt_id() -> u32;
    fn nvim_get_ccline_last_colors_cmdbuff() -> *const c_char;
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_ccline_clear_last_colors_cmdbuff();
    fn nvim_ccline_reset_colors();
    fn nvim_get_ccline_prompt_id() -> u32;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_ccline_cmdfirstc() -> c_int;
    fn nvim_set_ccline_last_colors_prompt_id(id: u32);
    fn nvim_set_ccline_last_colors_cmdbuff(buf: *mut c_char);
    fn nvim_ccline_reset_colors_size();

    // --- ccs init and field accessors ---
    fn nvim_color_ccs_init();
    fn nvim_color_ccline_has_highlight_cb() -> c_int;
    fn nvim_color_use_ccline_highlight_cb();
    fn nvim_color_ccs_has_error() -> c_int;
    fn nvim_color_ccs_can_free_cb() -> c_int;
    fn nvim_color_ccs_arg_allocated() -> c_int;
    fn nvim_color_ccs_arg_string() -> *mut c_char;
    fn nvim_color_ccs_dup_arg();
    fn nvim_color_ccs_err_msg() -> *const c_char;
    fn nvim_color_ccs_err_errmsg() -> *const c_char;
    fn nvim_color_ccs_clear_error();
    fn nvim_color_ccs_tv_clear();
    fn nvim_color_ccs_free_cb();
    fn nvim_color_ccs_tv_type() -> c_int;
    fn nvim_color_ccs_tv_list() -> ListPtr;

    // --- TRY_WRAP shims ---
    fn nvim_color_try_get_dict_callback() -> c_int;
    fn nvim_color_try_callback_call() -> c_int;

    // --- Expr coloring and chunk push ---
    fn nvim_color_run_expr_coloring();
    fn nvim_ccline_colors_push(start: c_int, end: c_int, hl_id: c_int);

    // --- smsg (variadic, not callable from Rust) ---
    fn nvim_color_smsg_error(fmt: *const c_char, msg: *const c_char);

    // --- List iteration helpers ---
    fn nvim_color_tv_list_len(l: ListPtr) -> c_int;
    fn nvim_list_get_first(l: ListPtr) -> ListItemPtr;
    fn nvim_list_get_last(l: ListPtr) -> ListItemPtr;

    // --- Direct C exports (real symbols, not inline) ---
    fn tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;
    fn tv_get_string_chk(tv: *const c_void) -> *const c_char;
    fn syn_name2id(name: *const c_char) -> c_int;
    fn xmemdupz(src: *const c_char, len: usize) -> *mut c_char;
    fn redrawcmdline();

    // --- UI ---
    static mut msg_scroll: c_int;
    fn msg_putchar(c: c_int);
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);

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
// Migrated C functions: acquire_callback, run_callback_coloring, errmsg,
// result_tv_type, result_tv_list, finalize
// =============================================================================

/// Rust replacement for `nvim_color_acquire_callback` in ex_getln.c.
///
/// Initializes `ccs` and returns the callback type:
///  0 = no callback, 1 = highlight_callback, 2 = dict callback (':'), 3 = expr ('=')
/// Returns -1 on error.
///
/// # Safety
///
/// Must be called from the main thread with command line active.
unsafe fn nvim_color_acquire_callback_rs() -> c_int {
    // Initialize ccs (sets color_cb=CALLBACK_NONE, err=ERROR_INIT, arg=cmdbuff, etc.)
    nvim_color_ccs_init();

    let cmdfirstc = nvim_get_ccline_cmdfirstc();

    if nvim_color_ccline_has_highlight_cb() != 0 {
        nvim_color_use_ccline_highlight_cb();
        return 1;
    } else if cmdfirstc == b':' as c_int {
        // TRY_WRAP: try to acquire g:Nvim_color_cmdline callback.
        // Returns 1=ok, 0=failed; sets ccs.can_free_cb=true on return.
        if nvim_color_try_get_dict_callback() == 0 {
            return -1;
        }
        return 2;
    } else if cmdfirstc == b'=' as c_int {
        return 3;
    }
    0
}

/// Rust replacement for `nvim_color_run_callback_coloring` in ex_getln.c.
///
/// Allocates the arg string if needed, calls the TRY_WRAP shim for callback_call.
/// Returns 1 on success, 0 on failure.
///
/// # Safety
///
/// Must be called after `nvim_color_acquire_callback_rs`, from the main thread.
unsafe fn nvim_color_run_callback_coloring_rs() -> c_int {
    // If cmdbuff[cmdlen] != NUL, we need a NUL-terminated copy.
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let cmdlen = nvim_get_ccline_cmdlen();
    if !cmdbuff.is_null() && *cmdbuff.add(cmdlen as usize) != 0 {
        nvim_color_ccs_dup_arg();
    }
    nvim_color_try_callback_call()
}

/// Rust replacement for `nvim_color_errmsg` in ex_getln.c.
///
/// Prints an error message in HLF_E style: set msg_scroll, newline, msg_puts_hl.
///
/// # Safety
///
/// Must be called from the main thread.
unsafe fn nvim_color_errmsg_rs(msg: *const c_char) {
    msg_scroll = 1;
    msg_putchar(b'\n' as c_int);
    msg_puts_hl(msg, HLF_E_COLOR, false);
}

/// Rust replacement for `nvim_color_result_tv_type` in ex_getln.c.
/// Returns ccs.tv.v_type as c_int.
unsafe fn nvim_color_result_tv_type_rs() -> c_int {
    nvim_color_ccs_tv_type()
}

/// Rust replacement for `nvim_color_result_tv_list` in ex_getln.c.
/// Returns ccs.tv.vval.v_list as ListPtr.
unsafe fn nvim_color_result_tv_list_rs() -> ListPtr {
    nvim_color_ccs_tv_list()
}

/// Rust replacement for `nvim_color_finalize` in ex_getln.c.
///
/// If `success == 0`: print pending error, clear colors, redrawcmdline.
/// Updates cmdbuff cache, prompt_id; clears ccs.tv; frees callback if needed.
///
/// # Safety
///
/// Must be called from the main thread after coloring attempt.
unsafe fn nvim_color_finalize_rs(success: c_int) {
    if success == 0 {
        if nvim_color_ccs_has_error() != 0 {
            msg_putchar(b'\n' as c_int);
            msg_scroll = 1;
            nvim_color_smsg_error(nvim_color_ccs_err_errmsg(), nvim_color_ccs_err_msg());
            nvim_color_ccs_clear_error();
        }
        nvim_ccline_reset_colors_size();
    }
    if nvim_color_ccs_can_free_cb() != 0 {
        nvim_color_ccs_free_cb();
    }
    // Update cmdbuff cache: if arg was allocated (xmemdupz), take ownership;
    // otherwise xmemdupz the current cmdbuff.
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let cmdlen = nvim_get_ccline_cmdlen();
    let new_cmdbuff = if nvim_color_ccs_arg_allocated() != 0 {
        nvim_color_ccs_arg_string()
    } else {
        xmemdupz(cmdbuff.cast_const(), cmdlen as usize)
    };
    nvim_set_ccline_last_colors_prompt_id(nvim_get_ccline_prompt_id());
    nvim_set_ccline_last_colors_cmdbuff(new_cmdbuff);
    nvim_color_ccs_tv_clear();
    if success == 0 {
        redrawcmdline();
    }
}

// HLF_E = 6 (ErrorMsg highlight group)
const HLF_E_COLOR: c_int = 6;

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
    {
        let last_prompt_id = nvim_get_ccline_last_colors_prompt_id();
        let cur_prompt_id = nvim_get_ccline_prompt_id();
        let last_cmdbuff = nvim_get_ccline_last_colors_cmdbuff();
        let cur_cmdbuff = nvim_get_ccline_cmdbuff();
        if last_prompt_id == cur_prompt_id
            && !last_cmdbuff.is_null()
            && !cur_cmdbuff.is_null()
            && libc::strcmp(last_cmdbuff, cur_cmdbuff.cast_const()) == 0
        {
            return true;
        }
    }

    // --- Reset colors (kvec) and ccs ---
    nvim_ccline_reset_colors();
    // (ccs will be initialized in nvim_color_acquire_callback_rs)

    // --- Early exit for empty buffer ---
    {
        let cmdbuff = nvim_get_ccline_cmdbuff();
        if cmdbuff.is_null() || *cmdbuff == 0 {
            nvim_ccline_clear_last_colors_cmdbuff();
            return true;
        }
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
        // Note: ccs not initialized yet; nvim_color_finalize_rs handles success path.
        nvim_color_ccs_init();
        nvim_color_finalize_rs(1);
        return true;
    }

    // --- Acquire callback ---
    // Returns: -1=error, 0=none, 1=highlight_callback, 2=dict_callback, 3=expr_path
    let acquire = nvim_color_acquire_callback_rs();

    match acquire {
        -1 => {
            // Error occurred during callback acquisition.
            return finalize_error();
        }
        3 => {
            // '=' expression path: entirely handled in C (uses kvec macros).
            nvim_color_run_expr_coloring();
            nvim_color_finalize_rs(1);
            return true;
        }
        0 => {
            // No callback available.
            nvim_color_finalize_rs(1);
            return true;
        }
        1 | 2 => {} // callback acquired, fall through to invoke
        _ => {
            nvim_color_finalize_rs(1);
            return true;
        }
    }

    // --- Invoke callback ---
    if nvim_color_run_callback_coloring_rs() == 0 {
        return finalize_error();
    }

    // --- Validate return type ---
    if nvim_color_result_tv_type_rs() != VAR_LIST {
        nvim_color_errmsg_rs(c"E5400: Callback should return list".as_ptr());
        return finalize_error();
    }

    let list = nvim_color_result_tv_list_rs();
    if list.is_null() {
        nvim_color_finalize_rs(1);
        return true;
    }

    // --- Process list of color chunks ---
    if !process_color_list(list) {
        return finalize_error();
    }

    nvim_color_finalize_rs(1);
    true
}

/// Validate and process the list of `[start, end, hlgroup]` chunks returned
/// by the coloring callback.
///
/// Returns `false` if any chunk fails validation (error message already printed).
unsafe fn process_color_list(list: ListPtr) -> bool {
    let cmdlen = nvim_get_ccline_cmdlen();
    let mut prev_end: i64 = 0;
    let mut i: c_int = 0;
    let mut outer = nvim_list_get_first(list);

    while !outer.is_null() {
        // Each outer item must be a VAR_LIST (a list of 3 elements).
        if item_tv_type(outer) != VAR_LIST {
            let msg = format!("E5401: List item {i} is not a List\0");
            nvim_color_errmsg_rs(msg.as_ptr().cast());
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
            nvim_color_errmsg_rs(msg.as_ptr().cast());
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
        // Uses tv_get_number_chk directly (real C export, not static-inline).
        let mut error = false;
        let start = tv_get_number_chk(list_item_tv(first), &raw mut error);
        if error {
            return false;
        }
        if !(prev_end <= start && start < i64::from(cmdlen)) {
            let msg =
                format!("E5403: Chunk {i} start {start} not in range [{prev_end}, {cmdlen})\0");
            nvim_color_errmsg_rs(msg.as_ptr().cast());
            return false;
        }
        // Inline nvim_color_cmdbuff_at: access cmdbuff[start] directly.
        let cmdbuff_start = *nvim_get_ccline_cmdbuff().add(start as usize) as u8;
        if is_utf8_continuation(cmdbuff_start) {
            let msg = format!("E5405: Chunk {i} start {start} splits multibyte character\0");
            nvim_color_errmsg_rs(msg.as_ptr().cast());
            return false;
        }

        // Fill any gap before this chunk with hl_id=0.
        if start != prev_end {
            nvim_ccline_colors_push(prev_end as c_int, start as c_int, 0);
        }

        // --- Validate end ---
        let end = tv_get_number_chk(list_item_tv(second), &raw mut error);
        if error {
            return false;
        }
        if !(start < end && end <= i64::from(cmdlen)) {
            let msg = format!("E5404: Chunk {i} end {end} not in range ({start}, {cmdlen}]\0");
            nvim_color_errmsg_rs(msg.as_ptr().cast());
            return false;
        }
        let cmdbuff_end = *nvim_get_ccline_cmdbuff().add(end as usize) as u8;
        if end < i64::from(cmdlen) && is_utf8_continuation(cmdbuff_end) {
            let msg = format!("E5406: Chunk {i} end {end} splits multibyte character\0");
            nvim_color_errmsg_rs(msg.as_ptr().cast());
            return false;
        }

        prev_end = end;

        // --- Get highlight group ---
        // Uses tv_get_string_chk directly (real C export, not static-inline).
        let group = tv_get_string_chk(list_item_tv(last_item));
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
    nvim_color_finalize_rs(0);
    false
}
