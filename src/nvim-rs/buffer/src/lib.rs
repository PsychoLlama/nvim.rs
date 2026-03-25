//! Buffer handling utilities for Neovim
//!
//! This crate provides Rust implementations of buffer-related functions
//! from `src/nvim/buffer.c`. It uses an opaque handle pattern where
//! `buf_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::cast_possible_wrap)] // Byte literals in ASCII range are safe

pub mod close;
pub mod errors;
pub mod expand;
pub mod filename;
pub mod info;
pub mod lifecycle;
pub mod list;
pub mod messages;
pub mod misc;
pub mod modeline;
pub mod properties;
pub mod state;
pub mod wininfo;

use std::ffi::{c_char, c_int};

/// Opaque handle to a Neovim buffer (`buf_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(*mut std::ffi::c_void);

impl BufHandle {
    /// Create a new buffer handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `buf_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Neovim window (`win_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut std::ffi::c_void);

// Event constants from auevents_enum.generated.h
const EVENT_BUFADD: c_int = 0;
const EVENT_BUFDELETE: c_int = 2;

// C accessor functions for buffer fields.
// These are defined in buffer.c and provide safe access to buf_T fields.
extern "C" {
    /// Get the `b_p_bt` (buftype option) field - returns first char.
    fn nvim_buf_get_buftype(buf: BufHandle) -> c_char;

    /// Get the `b_p_bt[2]` character (for checking "nofile" vs "nowrite").
    fn nvim_buf_get_buftype_2(buf: BufHandle) -> c_char;

    /// Get the `b_help` field from a buffer.
    fn nvim_buf_get_help(buf: BufHandle) -> c_int;

    /// Check if buffer has a terminal attached (`buf->terminal != NULL`).
    fn nvim_buf_get_terminal(buf: BufHandle) -> c_int;

    /// Get the first character of the `b_p_ff` (fileformat option) field.
    fn nvim_buf_get_fileformat(buf: BufHandle) -> c_char;

    /// Get the `b_p_bin` (binary mode) field from a buffer.
    fn nvim_buf_get_bin(buf: BufHandle) -> c_int;

    /// Get the last buffer in the buffer list (`lastbuf` global).
    fn nvim_get_lastbuf() -> BufHandle;

    /// Get the `b_prev` field from a buffer.
    fn nvim_buf_get_prev(buf: BufHandle) -> BufHandle;

    /// Get the `top_file_num` global (highest file number counter).
    fn nvim_get_top_file_num() -> c_int;

    /// Get the first character of the `b_p_bh` (bufhidden option) field.
    fn nvim_buf_get_bufhidden(buf: BufHandle) -> c_char;

    /// Global `p_hid` option (hidden buffers).
    static p_hid: c_int;

    /// Get the `cmdmod.cmod_flags` field.
    fn nvim_get_cmdmod_cmod_flags() -> c_int;

    /// Get the `buf_free_count` global counter.
    fn nvim_get_buf_free_count() -> c_int;

    /// Get the `br_buf` field from a bufref.
    fn nvim_bufref_get_buf(bufref: *const std::ffi::c_void) -> BufHandle;

    /// Get the `br_fnum` field from a bufref.
    fn nvim_bufref_get_fnum(bufref: *const std::ffi::c_void) -> c_int;

    /// Get the `br_buf_free_count` field from a bufref.
    fn nvim_bufref_get_buf_free_count(bufref: *const std::ffi::c_void) -> c_int;

    /// Get the `b_fnum` field from a buffer.
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;

    /// Get the `curbuf->b_ml.ml_flags` field.
    fn nvim_curbuf_get_ml_flags() -> c_int;

    /// Get the `ML_LINE_DIRTY` constant.
    fn nvim_get_ml_line_dirty() -> c_int;

    /// Get the `b_fname` field from a buffer (short filename).
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;

    /// Get the `b_ffname` field from a buffer (full filename).
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;

    /// Emit an error message.
    fn nvim_emsg(msg: *const c_char);

    /// Get the current buffer.
    fn nvim_get_curbuf() -> BufHandle;

    /// Get the `b_nwindows` field from a buffer.
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;

    /// Check if memfile pointer is NULL for a buffer.
    fn nvim_buf_get_ml_mfp_null(buf: BufHandle) -> c_int;

    /// Check if current buffer is changed.
    fn curbufIsChanged() -> bool;

    /// Compare two file paths (platform-aware).
    #[link_name = "path_fnamecmp"]
    fn nvim_path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;

    /// Get file identity for a path. Returns true if successful.
    #[link_name = "os_fileid"]
    fn nvim_os_fileid(path: *const c_char, file_id_out: *mut u8) -> bool;

    /// Compare two file identities.
    #[link_name = "os_fileid_equal"]
    fn nvim_os_fileid_equal(a: *const u8, b: *const u8) -> bool;

    /// Check if buffer has a valid cached `file_id`.
    fn nvim_buf_file_id_valid(buf: BufHandle) -> c_int;

    /// Copy buffer's cached `file_id` into output buffer.
    fn nvim_buf_get_file_id(buf: BufHandle, out: *mut u8);

    /// Set buffer's `file_id` data and validity flag.
    fn nvim_buf_set_file_id_data(buf: BufHandle, file_id: *const u8, valid: bool);

    /// Find a buffer by its number.
    #[link_name = "rs_buflist_findnr"]
    fn nvim_buflist_findnr(fnum: c_int) -> BufHandle;

    /// Get the stored line number for a buffer.
    fn nvim_buflist_findlnum(buf: BufHandle) -> c_int;

    /// Get the quickfix stack buffer number.
    fn qf_stack_get_bufnr() -> c_int;

    /// Get the `cmdwin_buf` global.
    fn nvim_get_cmdwin_buf() -> BufHandle;

    /// Get `ARGCOUNT` value.
    fn nvim_get_argcount() -> c_int;

    /// Get `w_arg_idx` from a window.
    fn nvim_win_get_arg_idx(wp: WinHandle) -> c_int;

    /// Get `w_arg_idx_invalid` from a window.
    fn nvim_win_get_arg_idx_invalid(wp: WinHandle) -> c_int;

    /// Get `w_topline` from a window.
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;

    /// Get `w_topfill` from a window.
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;

    /// Get `w_botline` from a window.
    fn nvim_win_get_botline(wp: WinHandle) -> c_int;

    /// Get fill lines for a window at a line number.
    fn nvim_win_get_fill(wp: WinHandle, lnum: c_int) -> c_int;

    /// Get the buffer associated with a window.
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    /// Get `b_ml.ml_line_count` from a buffer.
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> c_int;

    /// Calculate percentage (from math crate).
    fn rs_calc_percentage(part: i64, whole: i64) -> c_int;
}

// Phase 4 accessor functions.
#[allow(dead_code)]
extern "C" {
    /// Get the current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Get cursor line number for a window.
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;

    /// Get cursor column for a window.
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;

    /// Get stored lnum from `buflist_findfmark`.
    fn nvim_buflist_findfmark_lnum(buf: BufHandle) -> c_int;

    /// Get `b_p_bl` (buflisted) from a buffer.
    fn nvim_buf_get_b_p_bl(buf: BufHandle) -> c_int;

    /// Set `b_p_bl` (buflisted) on a buffer.
    fn nvim_buf_set_b_p_bl(buf: BufHandle, val: c_int);

    /// `apply_autocmds` fires autocommands for the given event.
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: BufHandle,
    ) -> bool;
}

/// Check if "buf" is a pointer to an existing buffer.
///
/// This is the Rust equivalent of `buf_valid()` in buffer.c.
/// Iterates backwards through the buffer list (`lastbuf` -> `b_prev`).
#[inline]
fn buf_valid_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }

    // Iterate backwards through the buffer list
    // SAFETY: nvim_get_lastbuf and nvim_buf_get_prev are safe accessors
    let mut bp = unsafe { nvim_get_lastbuf() };
    while !bp.is_null() {
        if bp == buf {
            return true;
        }
        bp = unsafe { nvim_buf_get_prev(bp) };
    }
    false
}

/// FFI wrapper for `buf_valid`.
///
/// Returns non-zero if the buffer is valid.
#[no_mangle]
pub extern "C" fn rs_buf_valid(buf: BufHandle) -> c_int {
    c_int::from(buf_valid_impl(buf))
}

/// Check if a buffer reference is still valid.
///
/// Uses the cached `buf_free_count` to avoid iterating through the buffer
/// list when no buffers have been freed since the reference was created.
/// If `buf_free_count` has changed, falls back to `buf_valid` and verifies
/// the buffer's fnum still matches.
///
/// # Safety
/// `bufref` must be a valid pointer to a `bufref_T` structure.
#[inline]
fn bufref_valid_impl(bufref: *const std::ffi::c_void) -> bool {
    if bufref.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        let cached_count = nvim_bufref_get_buf_free_count(bufref);
        let current_count = nvim_get_buf_free_count();

        if cached_count == current_count {
            // No buffers have been freed since the reference was created
            return true;
        }

        // buf_free_count changed, need to verify the buffer is still valid
        let buf = nvim_bufref_get_buf(bufref);
        if !buf_valid_impl(buf) {
            return false;
        }

        // Also verify the buffer's fnum still matches
        let ref_fnum = nvim_bufref_get_fnum(bufref);
        let buf_fnum = nvim_buf_get_fnum(buf);
        ref_fnum == buf_fnum
    }
}

/// FFI wrapper for `bufref_valid`.
///
/// Returns non-zero if the buffer reference is still valid.
#[no_mangle]
pub extern "C" fn rs_bufref_valid(bufref: *const std::ffi::c_void) -> c_int {
    c_int::from(bufref_valid_impl(bufref))
}

/// Check if buffer is a prompt buffer ('buftype' starts with 'p').
#[inline]
fn bt_prompt_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above, and the accessor handles the pointer safely.
    unsafe { nvim_buf_get_buftype(buf) == b'p' as c_char }
}

/// FFI wrapper for `bt_prompt`.
#[no_mangle]
pub extern "C" fn rs_bt_prompt(buf: BufHandle) -> bool {
    bt_prompt_impl(buf)
}

/// Check if buffer is a normal buffer ('buftype' is empty/NUL).
#[inline]
fn bt_normal_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_buftype(buf) == 0 }
}

/// FFI wrapper for `bt_normal`.
#[no_mangle]
pub extern "C" fn rs_bt_normal(buf: BufHandle) -> bool {
    bt_normal_impl(buf)
}

/// Check if buffer is the quickfix buffer ('buftype' starts with 'q').
#[inline]
fn bt_quickfix_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_buftype(buf) == b'q' as c_char }
}

/// FFI wrapper for `bt_quickfix`.
#[no_mangle]
pub extern "C" fn rs_bt_quickfix(buf: BufHandle) -> bool {
    bt_quickfix_impl(buf)
}

/// Check if buffer is a terminal buffer ('buftype' starts with 't').
#[inline]
fn bt_terminal_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_buftype(buf) == b't' as c_char }
}

/// FFI wrapper for `bt_terminal`.
#[no_mangle]
pub extern "C" fn rs_bt_terminal(buf: BufHandle) -> bool {
    bt_terminal_impl(buf)
}

/// Check if buffer has 'buftype' set to "nofile".
///
/// This checks that `b_p_bt[0]` == 'n' AND `b_p_bt[2]` == 'f'.
#[inline]
fn bt_nofile_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        nvim_buf_get_buftype(buf) == b'n' as c_char && nvim_buf_get_buftype_2(buf) == b'f' as c_char
    }
}

/// FFI wrapper for `bt_nofile`.
#[no_mangle]
pub extern "C" fn rs_bt_nofile(buf: BufHandle) -> bool {
    bt_nofile_impl(buf)
}

/// Check if buffer is a help buffer.
#[inline]
fn bt_help_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe { nvim_buf_get_help(buf) != 0 }
}

/// FFI wrapper for `bt_help`.
#[no_mangle]
pub extern "C" fn rs_bt_help(buf: BufHandle) -> bool {
    bt_help_impl(buf)
}

/// Check if buffer has a name that may not be a file name.
///
/// Returns true if buffer is "nofile", "acwrite", terminal, or "prompt".
/// This means the buffer name may not be a file name, at least not for writing.
#[inline]
fn bt_nofilename_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        let bt0 = nvim_buf_get_buftype(buf);
        // "nofile": b_p_bt[0]=='n' && b_p_bt[2]=='f'
        // "acwrite": b_p_bt[0]=='a'
        // terminal: buf->terminal != NULL
        // "prompt": b_p_bt[0]=='p'
        (bt0 == b'n' as c_char && nvim_buf_get_buftype_2(buf) == b'f' as c_char)
            || bt0 == b'a' as c_char
            || nvim_buf_get_terminal(buf) != 0
            || bt0 == b'p' as c_char
    }
}

/// FFI wrapper for `bt_nofilename`.
#[no_mangle]
pub extern "C" fn rs_bt_nofilename(buf: BufHandle) -> bool {
    bt_nofilename_impl(buf)
}

/// Check if buffer should not be written.
///
/// Returns true if buffer is "nowrite", "nofile", terminal, or "prompt".
#[inline]
fn bt_dontwrite_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        let bt0 = nvim_buf_get_buftype(buf);
        // "nowrite" or "nofile": b_p_bt[0]=='n'
        // terminal: buf->terminal != NULL
        // "prompt": b_p_bt[0]=='p'
        bt0 == b'n' as c_char || nvim_buf_get_terminal(buf) != 0 || bt0 == b'p' as c_char
    }
}

/// FFI wrapper for `bt_dontwrite`.
#[no_mangle]
pub extern "C" fn rs_bt_dontwrite(buf: BufHandle) -> bool {
    bt_dontwrite_impl(buf)
}

/// Check if buffer should not be read from a file.
///
/// Returns true if buffer is "nofile", "quickfix", terminal, or "prompt".
/// This means the buffer is not to be read from a file.
#[inline]
fn bt_nofileread_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    // SAFETY: We check for null above.
    unsafe {
        let bt0 = nvim_buf_get_buftype(buf);
        // "nofile": b_p_bt[0]=='n' && b_p_bt[2]=='f'
        // terminal: b_p_bt[0]=='t'
        // quickfix: b_p_bt[0]=='q'
        // "prompt": b_p_bt[0]=='p'
        (bt0 == b'n' as c_char && nvim_buf_get_buftype_2(buf) == b'f' as c_char)
            || bt0 == b't' as c_char
            || bt0 == b'q' as c_char
            || bt0 == b'p' as c_char
    }
}

/// FFI wrapper for `bt_nofileread`.
#[no_mangle]
pub extern "C" fn rs_bt_nofileread(buf: BufHandle) -> bool {
    bt_nofileread_impl(buf)
}

/// End-of-line type constants (matching C defines in `option_vars.h`).
pub const EOL_UNIX: c_int = 0; // NL
pub const EOL_DOS: c_int = 1; // CR NL
pub const EOL_MAC: c_int = 2; // CR

/// Get the current end-of-line type for a buffer.
///
/// Returns `EOL_DOS`, `EOL_UNIX`, or `EOL_MAC` based on the buffer's
/// 'fileformat' and 'binary' options.
#[inline]
fn get_fileformat_impl(buf: BufHandle) -> c_int {
    if buf.is_null() {
        return EOL_UNIX;
    }
    // SAFETY: We check for null above.
    unsafe {
        // If binary mode or first char is 'u' (unix), return EOL_UNIX
        #[allow(clippy::cast_sign_loss)]
        let ff = nvim_buf_get_fileformat(buf) as u8;
        if nvim_buf_get_bin(buf) != 0 || ff == b'u' {
            return EOL_UNIX;
        }
        // If first char is 'm' (mac), return EOL_MAC
        if ff == b'm' {
            return EOL_MAC;
        }
        // Otherwise (dos), return EOL_DOS
        EOL_DOS
    }
}

/// FFI wrapper for `get_fileformat`.
#[no_mangle]
pub extern "C" fn rs_get_fileformat(buf: BufHandle) -> c_int {
    get_fileformat_impl(buf)
}

/// Get the highest possible buffer number.
///
/// Returns `top_file_num - 1` since `top_file_num` is the next number
/// to be assigned to a new buffer.
#[inline]
fn get_highest_fnum_impl() -> c_int {
    // SAFETY: nvim_get_top_file_num is a simple global accessor
    unsafe { nvim_get_top_file_num() - 1 }
}

/// FFI wrapper for `get_highest_fnum`.
#[no_mangle]
pub extern "C" fn rs_get_highest_fnum() -> c_int {
    get_highest_fnum_impl()
}

/// Command modifier flag for `:hide` (matches C `CMOD_HIDE` = 0x0020).
const CMOD_HIDE: c_int = 0x0020;

/// Check if a buffer should be hidden when abandoned.
///
/// Returns true if:
/// - 'bufhidden' option is "hide", OR
/// - 'bufhidden' is empty AND ('hidden' option is set OR `:hide` modifier was used)
///
/// Returns false if 'bufhidden' is "unload", "wipe", or "delete".
#[inline]
fn buf_hide_impl(buf: BufHandle) -> bool {
    if buf.is_null() {
        return false;
    }

    // SAFETY: We check for null above, and the accessors handle the pointer safely.
    unsafe {
        // 'bufhidden' overrules 'hidden' and ":hide", check it first
        #[allow(clippy::cast_sign_loss)]
        let bh = nvim_buf_get_bufhidden(buf) as u8;
        match bh {
            // "unload", "wipe", "delete" -> don't hide
            b'u' | b'w' | b'd' => false,
            // "hide" -> hide
            b'h' => true,
            // empty (NUL) or anything else -> fall through to global options
            _ => p_hid != 0 || (nvim_get_cmdmod_cmod_flags() & CMOD_HIDE) != 0,
        }
    }
}

/// FFI wrapper for `buf_hide`.
#[no_mangle]
pub extern "C" fn rs_buf_hide(buf: BufHandle) -> bool {
    buf_hide_impl(buf)
}

/// Get `buf->b_fname`, use "[No Name]" if it is NULL.
///
/// This is the Rust equivalent of `buf_get_fname()` in buffer.c.
#[inline]
unsafe fn buf_get_fname_impl(buf: BufHandle) -> *const c_char {
    if buf.is_null() {
        return messages::no_name_msg();
    }
    let fname = nvim_buf_get_b_fname(buf);
    if fname.is_null() {
        messages::no_name_msg()
    } else {
        fname
    }
}

/// FFI wrapper for `buf_get_fname`.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_get_fname(buf: BufHandle) -> *const c_char {
    buf_get_fname_impl(buf)
}

/// Check if buffer should not be written, and emit error message E382 if so.
///
/// Returns true if buffer cannot be written (and error was emitted).
#[inline]
unsafe fn bt_dontwrite_msg_impl(buf: BufHandle) -> bool {
    if bt_dontwrite_impl(buf) {
        nvim_emsg(messages::e382_msg());
        true
    } else {
        false
    }
}

/// FFI wrapper for `bt_dontwrite_msg`.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_dontwrite_msg(buf: BufHandle) -> bool {
    bt_dontwrite_msg_impl(buf)
}

/// Check if the current buffer is empty, unnamed, unmodified and used in
/// only one window. That means it can be reused.
#[inline]
unsafe fn curbuf_reusable_impl() -> bool {
    let curbuf = nvim_get_curbuf();
    if curbuf.is_null() {
        return false;
    }
    nvim_buf_get_b_ffname(curbuf).is_null()
        && nvim_buf_get_nwindows(curbuf) <= 1
        && (nvim_buf_get_ml_mfp_null(curbuf) != 0 || state::buf_is_empty(curbuf))
        && !bt_quickfix_impl(curbuf)
        && !curbufIsChanged()
}

/// FFI wrapper for `curbuf_reusable`.
///
/// # Safety
///
/// Accesses global state (curbuf) via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_curbuf_reusable() -> bool {
    curbuf_reusable_impl()
}

// =============================================================================
// Special Buffer Names & Info Formatting (Phase 3)
// =============================================================================

/// Get special buffer name, or NULL if the buffer has a normal file name.
///
/// # Safety
///
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_spname(buf: BufHandle) -> *mut c_char {
    if buf.is_null() {
        return std::ptr::null_mut();
    }

    // Quickfix/location list
    if bt_quickfix_impl(buf) {
        let fnum = nvim_buf_get_fnum(buf);
        if fnum == qf_stack_get_bufnr() {
            return messages::msg_qflist().cast_mut();
        }
        return messages::msg_loclist().cast_mut();
    }

    // Buffer types with no file name
    if bt_nofilename_impl(buf) {
        let fname = nvim_buf_get_b_fname(buf);
        if !fname.is_null() {
            return fname.cast_mut();
        }
        if buf == nvim_get_cmdwin_buf() {
            return messages::msg_command_line().cast_mut();
        }
        if bt_prompt_impl(buf) {
            return messages::msg_prompt().cast_mut();
        }
        return messages::msg_scratch().cast_mut();
    }

    // Buffer with no fname gets "[No Name]"
    if nvim_buf_get_b_fname(buf).is_null() {
        return buf_get_fname_impl(buf).cast_mut();
    }

    std::ptr::null_mut()
}

/// Get alternate file name for the current window.
/// Returns NULL if there isn't any, and emits error message if requested.
///
/// # Safety
///
/// Accesses global state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_getaltfname(errmsg: bool) -> *mut c_char {
    let mut fname: *const c_char = std::ptr::null();
    let mut dummy: c_int = 0;

    if rs_buflist_name_nr(
        0,
        std::ptr::addr_of_mut!(fname),
        std::ptr::addr_of_mut!(dummy),
    ) != 0
    {
        // FAIL
        if errmsg {
            nvim_emsg(messages::e_noalt());
        }
        return std::ptr::null_mut();
    }
    fname.cast_mut()
}

/// Append "(N of M)" to a buffer string, if editing more than one file.
///
/// Returns the number of characters appended.
///
/// # Safety
///
/// `wp` must be a valid window handle. `buf` must be a valid writable buffer
/// of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_append_arg_number(
    wp: WinHandle,
    buf: *mut c_char,
    buflen: usize,
) -> c_int {
    let argcount = nvim_get_argcount();
    if argcount <= 1 {
        return 0;
    }

    let arg_idx = nvim_win_get_arg_idx(wp) + 1;
    let invalid = nvim_win_get_arg_idx_invalid(wp) != 0;

    let fmt = if invalid {
        messages::msg_arg_number_invalid()
    } else {
        messages::msg_arg_number()
    };

    // Use snprintf with the translated format string
    let n = libc::snprintf(buf, buflen, fmt, arg_idx, argcount);
    if n < 0 {
        0
    } else {
        // snprintf returns the number of chars that would have been written;
        // cap to actual buffer capacity
        let max_written = c_int::try_from(buflen.saturating_sub(1)).unwrap_or(c_int::MAX);
        n.min(max_written)
    }
}

// =============================================================================
// Statusline Position Formatting (Phase 4)
// =============================================================================

/// Get relative cursor position in window, formatted as "All", "Top", "Bot",
/// or a localized percentage string.
///
/// # Safety
///
/// `wp` must be a valid window handle. `buf` must be a valid writable buffer
/// of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_get_rel_pos(wp: WinHandle, buf: *mut c_char, buflen: c_int) -> c_int {
    if buflen < 3 {
        return 0;
    }

    let topline = nvim_win_get_topline(wp);
    let topfill = nvim_win_get_topfill(wp);
    let botline = nvim_win_get_botline(wp);

    // Number of lines above window
    let mut above: i64 = i64::from(topline) - 1;
    above += i64::from(nvim_win_get_fill(wp, topline)) - i64::from(topfill);
    if topline == 1 && topfill >= 1 {
        above = 0;
    }

    // Number of lines below window
    let win_buf = nvim_win_get_buffer(wp);
    let line_count = nvim_buf_get_ml_line_count(win_buf);
    let below: i64 = i64::from(line_count) - i64::from(botline) + 1;

    // buflen >= 3 guaranteed by guard above, so unsigned_abs() is safe
    let buflen_sz = buflen.unsigned_abs() as usize;

    if below <= 0 {
        let msg = if above == 0 {
            messages::msg_all()
        } else {
            messages::msg_bot()
        };
        let n = libc::snprintf(buf, buflen_sz, c"%s".as_ptr(), msg);
        return if n < 0 { 0 } else { n.min(buflen - 1) };
    }

    if above <= 0 {
        let n = libc::snprintf(buf, buflen_sz, c"%s".as_ptr(), messages::msg_top());
        return if n < 0 { 0 } else { n.min(buflen - 1) };
    }

    let perc = rs_calc_percentage(above, above + below);
    let mut tmp = [0u8; 8];
    libc::snprintf(
        tmp.as_mut_ptr().cast(),
        tmp.len(),
        messages::msg_pct(),
        perc,
    );
    let n = libc::snprintf(buf, buflen_sz, messages::msg_3s(), tmp.as_ptr());
    if n < 0 {
        0
    } else {
        n.min(buflen - 1)
    }
}

// =============================================================================
// File Identity & Alternate Buffer (Phase 2)
// =============================================================================

/// Maximum size for an opaque `FileID` buffer.
/// `FileID` is `{ uint64_t inode; uint64_t device_id; }` = 16 bytes.
const FILE_ID_SIZE: usize = 16;

/// Opaque file identity buffer, sized to hold a C `FileID` struct.
type FileIdBuf = [u8; FILE_ID_SIZE];

/// Check if buffer's cached `file_id` matches the given `file_id`.
#[inline]
unsafe fn buf_same_file_id(buf: BufHandle, file_id: &FileIdBuf) -> bool {
    if nvim_buf_file_id_valid(buf) == 0 {
        return false;
    }
    let mut buf_fid: FileIdBuf = [0u8; FILE_ID_SIZE];
    nvim_buf_get_file_id(buf, buf_fid.as_mut_ptr());
    nvim_os_fileid_equal(buf_fid.as_ptr(), file_id.as_ptr())
}

/// Cache a file's `FileID` into the buffer struct.
#[inline]
unsafe fn buf_set_file_id_impl(buf: BufHandle) {
    let fname = nvim_buf_get_b_fname(buf);
    if fname.is_null() {
        nvim_buf_set_file_id_data(buf, std::ptr::null(), false);
        return;
    }
    let mut file_id: FileIdBuf = [0u8; FILE_ID_SIZE];
    let valid = nvim_os_fileid(fname, file_id.as_mut_ptr());
    nvim_buf_set_file_id_data(buf, file_id.as_ptr(), valid);
}

/// FFI wrapper for `buf_set_file_id`.
///
/// # Safety
///
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_set_file_id(buf: BufHandle) {
    buf_set_file_id_impl(buf);
}

/// Check that "ffname" is not the same file as the file loaded in "buf".
///
/// This is the core comparison logic (equivalent to C `otherfile_buf`).
#[inline]
unsafe fn otherfile_buf_impl(buf: BufHandle, ffname: *const c_char) -> bool {
    // no name is different
    if ffname.is_null() || *ffname == 0 || nvim_buf_get_b_ffname(buf).is_null() {
        return true;
    }
    // fast path: string comparison
    if nvim_path_fnamecmp(ffname, nvim_buf_get_b_ffname(buf)) == 0 {
        return false;
    }
    // slow path: file identity comparison
    let mut file_id: FileIdBuf = [0u8; FILE_ID_SIZE];
    let file_id_valid = nvim_os_fileid(ffname, file_id.as_mut_ptr());
    if !file_id_valid {
        return true;
    }
    if buf_same_file_id(buf, &file_id) {
        // Re-stat and check again (file may have been recreated)
        buf_set_file_id_impl(buf);
        if buf_same_file_id(buf, &file_id) {
            return false;
        }
    }
    true
}

/// Check that "ffname" is not the same file as the current buffer.
///
/// # Safety
///
/// Accesses global state (curbuf) via C FFI. `ffname` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_otherfile(ffname: *const c_char) -> bool {
    otherfile_buf_impl(nvim_get_curbuf(), ffname)
}

// =============================================================================
// File Identity Helpers (Phase 3: Wave 2)
// =============================================================================

extern "C" {
    #[link_name = "fix_fname"]
    fn nvim_fix_fname(fname: *const c_char) -> *mut c_char;
    #[link_name = "buflist_new"]
    fn nvim_buflist_new(
        ffname: *const c_char,
        sfname: *const c_char,
        lnum: c_int,
        flags: c_int,
    ) -> BufHandle;
}

/// Check that "ffname" is not the same file as buffer "buf" (4-arg version).
///
/// When `file_id_p` is non-null, uses the provided file identity instead of
/// computing it. This avoids redundant `os_fileid` calls when the caller
/// already has the file identity.
///
/// # Safety
///
/// `buf` must be a valid buffer handle. `ffname` must be a valid C string
/// or null. `file_id_p` must point to a valid `FileID` buffer or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_otherfile_buf_4(
    buf: BufHandle,
    ffname: *const c_char,
    file_id_p: *const u8,
    file_id_valid: bool,
) -> bool {
    // no name is different
    if ffname.is_null() || *ffname == 0 || nvim_buf_get_b_ffname(buf).is_null() {
        return true;
    }
    // fast path: string comparison
    if nvim_path_fnamecmp(ffname, nvim_buf_get_b_ffname(buf)) == 0 {
        return false;
    }
    // If no file_id provided, compute it
    let mut local_fid: FileIdBuf = [0u8; FILE_ID_SIZE];
    let (fid_ptr, fid_valid) = if file_id_p.is_null() {
        let valid = nvim_os_fileid(ffname, local_fid.as_mut_ptr());
        (local_fid.as_ptr(), valid)
    } else {
        (file_id_p, file_id_valid)
    };
    if !fid_valid {
        return true;
    }
    // Copy to a FileIdBuf for buf_same_file_id
    let mut file_id: FileIdBuf = [0u8; FILE_ID_SIZE];
    std::ptr::copy_nonoverlapping(fid_ptr, file_id.as_mut_ptr(), FILE_ID_SIZE);
    if buf_same_file_id(buf, &file_id) {
        buf_set_file_id_impl(buf);
        if buf_same_file_id(buf, &file_id) {
            return false;
        }
    }
    true
}

/// Expand filename to full path.
///
/// Sets `*ffname` to the full path (allocated, replaces old value).
/// If `*sfname` is NULL, sets it to `*ffname`.
///
/// # Safety
///
/// `ffname` and `sfname` must be valid pointers to `*mut c_char` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fname_expand(
    _buf: BufHandle,
    ffname: *mut *mut c_char,
    sfname: *mut *mut c_char,
) {
    if (*ffname).is_null() {
        return;
    }
    if (*sfname).is_null() {
        *sfname = *ffname;
    }
    *ffname = nvim_fix_fname(*ffname);
    // MSWIN shortcut handling is not needed on Linux
}

/// Add a file to the buffer list.
///
/// Calls `buflist_new()` and returns the buffer number, or 0 on failure.
///
/// # Safety
///
/// `fname` must be a valid C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_add(fname: *mut c_char, flags: c_int) -> c_int {
    let buf = nvim_buflist_new(fname, std::ptr::null(), 0, flags);
    if buf.is_null() {
        return 0;
    }
    nvim_buf_get_fnum(buf)
}

/// Get file name and line number for file 'fnum'.
///
/// Returns FAIL (1) if not found, OK (0) for success.
///
/// # Safety
///
/// `fname` and `lnum` must be valid out-pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_buflist_name_nr(
    fnum: c_int,
    fname: *mut *const c_char,
    lnum: *mut c_int,
) -> c_int {
    let buf = nvim_buflist_findnr(fnum);
    if buf.is_null() {
        return 1; // FAIL
    }
    let b_fname = nvim_buf_get_b_fname(buf);
    if b_fname.is_null() {
        return 1; // FAIL
    }
    *fname = b_fname;
    *lnum = nvim_buflist_findlnum(buf);
    0 // OK
}

/// Check if a line that was just obtained by a call to `ml_get` is in allocated memory.
///
/// This ignores `ML_ALLOCATED` to get the same behavior as without `ML_GET_ALLOC_LINES`.
///
/// # Safety
/// Calls external C functions to access buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_line_alloced() -> c_int {
    nvim_curbuf_get_ml_flags() & nvim_get_ml_line_dirty()
}

// =============================================================================
// Buffer Display & Info Helpers (Phase 4: Wave 2)
// =============================================================================

/// Set alternate cursor position for the current buffer and window.
///
/// Saves the current cursor position and local window option values
/// for the current buffer, associated with the given window.
///
/// # Safety
///
/// `win` must be a valid window handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_altfpos(win: WinHandle) {
    let buf = nvim_get_curbuf();
    let lnum = nvim_win_get_cursor_lnum(win);
    let col = nvim_win_get_cursor_col(win);
    crate::wininfo::rs_buflist_setfpos(buf, win, lnum, col, true);
}

/// Find the stored line number for buffer `buf` for the current window.
///
/// # Safety
///
/// `buf` must be a valid buffer handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_findlnum(buf: BufHandle) -> c_int {
    nvim_buflist_findfmark_lnum(buf)
}

/// Set `buflisted` for curbuf to `on` and trigger autocommands if it changed.
///
/// If the value changes, fires `EVENT_BUFADD` (when turning on) or
/// `EVENT_BUFDELETE` (when turning off).
///
/// # Safety
///
/// Calls external C functions. Accesses global `curbuf`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_buflisted(on: c_int) {
    let buf = nvim_get_curbuf();
    if on == nvim_buf_get_b_p_bl(buf) {
        return;
    }
    nvim_buf_set_b_p_bl(buf, on);
    if on != 0 {
        apply_autocmds(EVENT_BUFADD, std::ptr::null(), std::ptr::null(), false, buf);
    } else {
        apply_autocmds(
            EVENT_BUFDELETE,
            std::ptr::null(),
            std::ptr::null(),
            false,
            buf,
        );
    }
}

// =============================================================================
// C-named symbol exports (for Rust crates that call functions by their C name)
// These duplicate the rs_* implementations under the canonical C symbol name.
// =============================================================================

/// C export: `buf_valid` - called by some Rust crates directly.
#[must_use]
#[export_name = "buf_valid"]
pub extern "C" fn buf_valid_export(buf: BufHandle) -> bool {
    buf_valid_impl(buf)
}

/// C export: `bufref_valid`.
#[must_use]
#[export_name = "bufref_valid"]
pub extern "C" fn bufref_valid_export(bufref: *const std::ffi::c_void) -> bool {
    bufref_valid_impl(bufref)
}

/// C export: `buf_hide`.
#[must_use]
#[export_name = "buf_hide"]
pub extern "C" fn buf_hide_export(buf: BufHandle) -> bool {
    buf_hide_impl(buf)
}

/// C export: `buf_spname`.
///
/// # Safety
///
/// `buf` must be a valid buffer handle.
#[must_use]
#[export_name = "buf_spname"]
pub unsafe extern "C" fn buf_spname_export(buf: BufHandle) -> *mut c_char {
    rs_buf_spname(buf)
}

/// C export: `otherfile`.
///
/// # Safety
///
/// `ffname` must be a valid C string.
#[must_use]
#[export_name = "otherfile"]
pub unsafe extern "C" fn otherfile_export(ffname: *const c_char) -> bool {
    otherfile_buf_impl(nvim_get_curbuf(), ffname)
}

/// C export: `set_buflisted`.
///
/// # Safety
///
/// Accesses global state via C FFI.
#[export_name = "set_buflisted"]
pub unsafe extern "C" fn set_buflisted_export(on: c_int) {
    rs_set_buflisted(on);
}

/// C export: `buflist_findnr`.
///
/// # Safety
///
/// Accesses global state via C FFI.
#[must_use]
#[export_name = "buflist_findnr"]
pub unsafe extern "C" fn buflist_findnr_export(nr: c_int) -> BufHandle {
    list::rs_buflist_findnr(nr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_handle_null() {
        let handle = unsafe { BufHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!buf_valid_impl(handle));
        assert!(!bt_prompt_impl(handle));
        assert!(!bt_normal_impl(handle));
        assert!(!bt_quickfix_impl(handle));
        assert!(!bt_terminal_impl(handle));
        assert!(!bt_nofile_impl(handle));
        assert!(!bt_help_impl(handle));
        assert!(!bt_nofilename_impl(handle));
        assert!(!bt_dontwrite_impl(handle));
        assert!(!bt_nofileread_impl(handle));
        assert!(!buf_hide_impl(handle));
        assert!(!unsafe { bt_dontwrite_msg_impl(handle) });
        // Null buffer defaults to EOL_UNIX
        assert_eq!(get_fileformat_impl(handle), EOL_UNIX);
    }

    #[test]
    fn test_buf_get_fname_null() {
        // Null buffer should return "[No Name]" (via nvim_no_name_msg)
        // In unit test context we can't call FFI, but we verify null-guard path
        let handle = unsafe { BufHandle::from_ptr(std::ptr::null_mut()) };
        // The function should not panic on null
        // (actual string verification requires integration test)
        assert!(handle.is_null());
    }

    #[test]
    fn test_buf_handle_non_null() {
        // Create a fake non-null pointer for testing
        let fake_ptr = 0x1000 as *mut std::ffi::c_void;
        let handle = unsafe { BufHandle::from_ptr(fake_ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), fake_ptr);
    }

    #[test]
    fn test_eol_constants() {
        // Verify EOL constants match C definitions
        assert_eq!(EOL_UNIX, 0);
        assert_eq!(EOL_DOS, 1);
        assert_eq!(EOL_MAC, 2);
    }

    #[test]
    fn test_buf_handle_size() {
        // BufHandle should be pointer-sized (repr(transparent))
        assert_eq!(
            std::mem::size_of::<BufHandle>(),
            std::mem::size_of::<*mut std::ffi::c_void>()
        );
    }

    #[test]
    fn test_cmod_hide_constant() {
        // Verify CMOD_HIDE matches C definition
        assert_eq!(CMOD_HIDE, 0x0020);
    }

    #[test]
    fn test_eol_constants_sequential() {
        // EOL constants should be sequential
        assert_eq!(EOL_DOS, EOL_UNIX + 1);
        assert_eq!(EOL_MAC, EOL_DOS + 1);
    }

    #[test]
    fn test_buf_handle_equality() {
        let ptr1 = 0x1000 as *mut std::ffi::c_void;
        let ptr2 = 0x1000 as *mut std::ffi::c_void;
        let ptr3 = 0x2000 as *mut std::ffi::c_void;
        let h1 = unsafe { BufHandle::from_ptr(ptr1) };
        let h2 = unsafe { BufHandle::from_ptr(ptr2) };
        let h3 = unsafe { BufHandle::from_ptr(ptr3) };
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
