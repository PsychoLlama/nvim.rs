//! Quickfix window display
//!
//! This module provides display formatting and rendering for quickfix and
//! location list windows.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::derivable_impls)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Display Constants
// =============================================================================

/// Maximum filename display width
pub const QF_FNAME_MAX_WIDTH: usize = 50;

/// Maximum text display width
pub const QF_TEXT_MAX_WIDTH: usize = 200;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to quickfix list
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QfListHandle(*mut c_void);

impl QfListHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to quickfix entry
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QfEntryHandle(*mut c_void);

impl QfEntryHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// Display Entry Info
// =============================================================================

/// Information about a quickfix entry for display purposes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfDisplayEntry {
    /// Entry index (1-based for display)
    pub index: c_int,
    /// File number
    pub fnum: c_int,
    /// Line number
    pub lnum: i32,
    /// Column number
    pub col: c_int,
    /// End line number
    pub end_lnum: i32,
    /// End column number
    pub end_col: c_int,
    /// Error type character ('E', 'W', 'I', 'N', or ' ')
    pub type_char: u8,
    /// Error number
    pub nr: c_int,
    /// Whether entry is valid (has file/line)
    pub valid: bool,
    /// Whether this is the current entry
    pub is_current: bool,
}

impl Default for QfDisplayEntry {
    fn default() -> Self {
        Self {
            index: 0,
            fnum: 0,
            lnum: 0,
            col: 0,
            end_lnum: 0,
            end_col: 0,
            type_char: b' ',
            nr: 0,
            valid: false,
            is_current: false,
        }
    }
}

// =============================================================================
// Display Format
// =============================================================================

/// Format style for quickfix display
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfDisplayFormat {
    /// Standard format: "filename|line col|message"
    Standard = 0,
    /// Compact format: "filename:line:col: message"
    Compact = 1,
    /// Long format: "filename|line col type nr|message"
    Long = 2,
}

impl Default for QfDisplayFormat {
    fn default() -> Self {
        Self::Standard
    }
}

// =============================================================================
// Display State
// =============================================================================

/// State of the quickfix window display
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfDisplayState {
    /// Total number of entries
    pub total_entries: c_int,
    /// Number of valid entries
    pub valid_entries: c_int,
    /// Current entry index (1-based)
    pub current_idx: c_int,
    /// First visible entry index
    pub first_visible: c_int,
    /// Number of visible lines in window
    pub visible_lines: c_int,
    /// Whether there are entries above the visible area
    pub has_entries_above: bool,
    /// Whether there are entries below the visible area
    pub has_entries_below: bool,
}

impl Default for QfDisplayState {
    fn default() -> Self {
        Self {
            total_entries: 0,
            valid_entries: 0,
            current_idx: 0,
            first_visible: 1,
            visible_lines: 0,
            has_entries_above: false,
            has_entries_below: false,
        }
    }
}

impl QfDisplayState {
    /// Check if display has entries
    pub const fn has_entries(&self) -> bool {
        self.total_entries > 0
    }

    /// Check if there are more entries than can fit in the window
    pub const fn needs_scrolling(&self) -> bool {
        self.total_entries > self.visible_lines
    }

    /// Calculate the scroll percentage (0-100)
    pub fn scroll_percent(&self) -> u8 {
        if self.total_entries <= self.visible_lines {
            return 100;
        }
        let visible_end = self.first_visible + self.visible_lines - 1;
        let percent = (visible_end * 100) / self.total_entries;
        if percent > 100 {
            100
        } else {
            percent as u8
        }
    }
}

// =============================================================================
// Position Formatting
// =============================================================================

/// Format a position (line:col) for display
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PositionFormat {
    /// Whether to include column
    pub include_col: bool,
    /// Whether to include range (end_lnum:end_col)
    pub include_range: bool,
    /// Minimum line number width for alignment
    pub min_lnum_width: u8,
    /// Minimum column width for alignment
    pub min_col_width: u8,
}

impl Default for PositionFormat {
    fn default() -> Self {
        Self {
            include_col: true,
            include_range: false,
            min_lnum_width: 0,
            min_col_width: 0,
        }
    }
}

/// Calculate the display width needed for a line number
pub const fn lnum_display_width(lnum: i32) -> u8 {
    if lnum < 10 {
        1
    } else if lnum < 100 {
        2
    } else if lnum < 1000 {
        3
    } else if lnum < 10000 {
        4
    } else if lnum < 100_000 {
        5
    } else if lnum < 1_000_000 {
        6
    } else {
        7
    }
}

// =============================================================================
// Type Character Formatting
// =============================================================================

/// Get the display character for an error type
pub const fn type_display_char(type_code: u8) -> u8 {
    match type_code {
        b'E' | b'e' => b'E',
        b'W' | b'w' => b'W',
        b'I' | b'i' => b'I',
        b'N' | b'n' => b'N',
        _ => b' ',
    }
}

/// Check if a type character should be highlighted as error
pub const fn is_error_type(type_code: u8) -> bool {
    matches!(type_code, b'E' | b'e')
}

/// Check if a type character should be highlighted as warning
pub const fn is_warning_type(type_code: u8) -> bool {
    matches!(type_code, b'W' | b'w')
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Calculate line number display width
#[no_mangle]
pub extern "C" fn rs_qf_lnum_display_width(lnum: i32) -> c_int {
    c_int::from(lnum_display_width(lnum))
}

/// FFI export: Get type display character
#[no_mangle]
pub extern "C" fn rs_qf_type_display_char(type_code: u8) -> u8 {
    type_display_char(type_code)
}

/// FFI export: Check if error type
#[no_mangle]
pub extern "C" fn rs_qf_is_error_type(type_code: u8) -> c_int {
    c_int::from(is_error_type(type_code))
}

/// FFI export: Check if warning type
#[no_mangle]
pub extern "C" fn rs_qf_is_warning_type(type_code: u8) -> c_int {
    c_int::from(is_warning_type(type_code))
}

// =============================================================================
// Phase 1: qf_types — build a display string from error type char + number
// =============================================================================

/// Pure-Rust implementation: format type display bytes into `out`.
///
/// Returns the number of bytes written (excluding NUL terminator).
/// `out` must have at least 20 bytes of capacity.
pub fn qf_types_fmt(c: c_int, nr: c_int, out: &mut [u8]) -> usize {
    use std::io::Write;

    let bufsz = out.len();
    if bufsz == 0 {
        return 0;
    }

    let type_char = if (0..=255).contains(&c) { c as u8 } else { 0 };

    let prefix: &[u8] = if is_warning_type(type_char) {
        b" warning"
    } else if type_char == b'I' || type_char == b'i' {
        b" info"
    } else if type_char == b'N' || type_char == b'n' {
        b" note"
    } else if is_error_type(type_char) || (c == 0 && nr > 0) {
        b" error"
    } else if c == 0 || c == 1 {
        b""
    } else {
        // Single character like " X" or " X  42"
        let disp = type_display_char(type_char);
        let mut cursor = std::io::Cursor::new(&mut out[..]);
        if nr > 0 {
            let _ = write!(cursor, " {} {:3}", disp as char, nr);
        } else {
            let _ = write!(cursor, " {}", disp as char);
        }
        let pos = cursor.position() as usize;
        let end = pos.min(bufsz - 1);
        out[end] = 0;
        return end;
    };

    if nr > 0 {
        let mut cursor = std::io::Cursor::new(&mut out[..]);
        // SAFETY: prefix is always valid UTF-8 (ASCII)
        let _ = write!(
            cursor,
            "{} {:3}",
            // safe: prefix is only ASCII bytes
            unsafe { std::str::from_utf8_unchecked(prefix) },
            nr
        );
        let pos = cursor.position() as usize;
        let end = pos.min(bufsz - 1);
        out[end] = 0;
        end
    } else {
        let copy_len = prefix.len().min(bufsz - 1);
        out[..copy_len].copy_from_slice(&prefix[..copy_len]);
        out[copy_len] = 0;
        copy_len
    }
}

/// FFI export: build type display string into caller-provided buffer.
///
/// Writes the formatted type string into `buf` (at most `bufsz` bytes
/// including the NUL terminator) and returns a pointer to `buf`.
///
/// # Safety
///
/// - `buf` must be a valid writable buffer of at least `bufsz` bytes.
/// - `bufsz` must be > 0.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_types(
    c: c_int,
    nr: c_int,
    buf: *mut c_char,
    bufsz: usize,
) -> *const c_char {
    if bufsz == 0 || buf.is_null() {
        return buf;
    }
    let slice = std::slice::from_raw_parts_mut(buf.cast::<u8>(), bufsz);
    qf_types_fmt(c, nr, slice);
    buf
}

/// FFI export: Calculate scroll percent
#[no_mangle]
pub extern "C" fn rs_qf_display_scroll_percent(
    total: c_int,
    first_visible: c_int,
    visible_lines: c_int,
) -> c_int {
    let state = QfDisplayState {
        total_entries: total,
        valid_entries: 0,
        current_idx: 0,
        first_visible,
        visible_lines,
        has_entries_above: false,
        has_entries_below: false,
    };
    c_int::from(state.scroll_percent())
}

// =============================================================================
// Phase 3: qf_fill_buffer
// =============================================================================

type LinenrT = i32;

/// Opaque buffer handle
type BufHandle = *mut c_void;
/// Opaque qfline handle
type QfLinePtr = *const c_void;
/// Opaque list handle
type ListPtr = *mut c_void;
/// Opaque list item handle
type ListItemPtr = *const c_void;

const MAXPATHL: usize = 4096;

// Autocmd event constants (from auevents_enum.generated.h)
// Validated by _Static_assert blocks in quickfix_shim.c if added.
const EVENT_BUFREADPOST: c_int = 13;
const EVENT_BUFWINENTER: c_int = 16;

extern "C" {
    fn nvim_qf_buf_is_curbuf(buf: BufHandle) -> bool;
    fn internal_error(where_: *const c_char);
    // (nvim_qf_fill_buffer_internal_error deleted: use internal_error directly)
    fn nvim_qf_delete_all_lines() -> bool;
    fn nvim_qf_zero_skipcol_for_curbuf();
    fn nvim_qf_u_clearallandblockfree();
    fn nvim_qf_get_start(qfl: *const c_void) -> QfLinePtr;
    fn nvim_qfline_get_next(qfp: QfLinePtr) -> QfLinePtr;
    fn nvim_qfline_get_fnum(qfp: QfLinePtr) -> c_int;
    fn nvim_qf_get_count(qfl: *const c_void) -> c_int;
    fn nvim_buf_get_line_count(buf: BufHandle) -> LinenrT;
    // Phase 11: rs_call_qftf_func accessors (replacing nvim_call_qftf_func)
    fn tv_dict_alloc_lock(scope: c_int) -> *mut c_void;
    fn nvim_callback_is_none(cb: *const c_void) -> bool;
    fn nvim_callback_call_one_dict(cb: *mut c_void, dict: *mut c_void, rettv: *mut c_void) -> bool;
    fn nvim_tv_rettv_list_if_var_list(rettv: *const c_void) -> *mut c_void;
    fn nvim_tv_list_ref(list: *mut c_void); // inline in C, needs wrapper
    fn tv_dict_unref(dict: *mut c_void);
    fn tv_clear(tv: *mut c_void);
    fn tv_dict_add_nr(dict: *mut c_void, key: *const c_char, key_len: usize, nr: i64) -> c_int;
    fn nvim_tv_dict_incr_refcount(dict: *mut c_void);
    fn nvim_qf_is_qf_list(qfl: *const c_void) -> bool;
    fn nvim_qf_get_id(qfl: *const c_void) -> u32;
    fn nvim_qfl_get_qftf_cb_ptr(qfl: *mut c_void) -> *mut c_void;
    fn nvim_qf_get_global_qftf_cb_ptr() -> *mut c_void;

    fn nvim_tv_list_first(list: *const c_void) -> ListItemPtr;
    fn nvim_tv_list_item_next(list: *const c_void, li: ListItemPtr) -> ListItemPtr;
    fn nvim_tv_list_item_string(li: ListItemPtr) -> *mut c_char;
    fn ml_delete(lnum: LinenrT);
    fn rs_check_lnums(do_curwin: c_int);
    // nvim_qf_set_filetype_and_autocmds replaced by thin accessors (Phase 14)
    fn nvim_qf_curbuf_incr_ro_locked();
    fn nvim_qf_curbuf_decr_ro_locked();
    fn nvim_qf_curbuf_set_ma_false();
    fn nvim_qf_curbuf_set_keep_filetype(val: bool);
    fn nvim_qf_set_option_filetype_qf();
    // nvim_qf_apply_autocmds_bufreadpost_qf/bufwinenter_qf deleted: use apply_autocmds directly
    fn apply_autocmds(
        event: c_int,
        fname: *mut c_char,
        fname_io: *mut c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    // curbuf alias for apply_autocmds calls (avoid clash with other modules' curbuf declarations)
    #[link_name = "curbuf"]
    static mut curbuf_fill: *mut c_void;
    fn nvim_qf_redraw_curbuf_later();
    fn nvim_qf_get_key_typed() -> bool;
    fn nvim_qf_set_key_typed(val: bool);

    // Phase 3: qf_buf_add_line accessors
    fn nvim_qfline_get_module(qfp: QfLinePtr) -> *const c_char;
    fn nvim_qfline_get_text(qfp: QfLinePtr) -> *const c_char;
    fn nvim_qfline_get_pattern(qfp: QfLinePtr) -> *const c_char;
    fn nvim_qfline_get_lnum(qfp: QfLinePtr) -> LinenrT;
    fn nvim_qfline_get_end_lnum(qfp: QfLinePtr) -> LinenrT;
    fn nvim_qfline_get_col(qfp: QfLinePtr) -> c_int;
    fn nvim_qfline_get_end_col(qfp: QfLinePtr) -> c_int;
    fn nvim_qfline_get_type(qfp: QfLinePtr) -> c_char;
    fn nvim_qfline_get_nr(qfp: QfLinePtr) -> c_int;
    fn nvim_qfline_get_fname(qfp: QfLinePtr) -> *const c_char;
    // nvim_buflist_findnr returns buf_T* (void* in Rust) - from buffer.c
    #[link_name = "rs_buflist_findnr"]
    fn nvim_buflist_findnr(nr: c_int) -> BufHandle;
    // nvim_buf_get_sfname takes buf_T* (void* in Rust) - from buffer.c
    fn nvim_buf_get_sfname(buf: BufHandle) -> *const c_char;
    #[link_name = "nvim_qf_buf_get_fname"]
    fn qf_buf_get_fname_fill(buf: *const c_void) -> *const c_char;
    fn path_tail(fname: *const c_char) -> *mut c_char;
    fn path_is_absolute(fname: *const c_char) -> bool;
    fn os_dirname(buf: *mut c_char, size: usize);
    fn shorten_buf_fname(buf: BufHandle, dirname: *mut c_char, force: c_int);
    fn ml_append_buf(
        buf: BufHandle,
        lnum: LinenrT,
        line: *mut c_char,
        len: c_int,
        newfile: bool,
    ) -> c_int;
    fn skipwhite(str: *const c_char) -> *mut c_char;
}

const FAIL: c_int = 0;
const OK: c_int = 1;

// =============================================================================
// Phase 11: rs_call_qftf_func (migrated from C call_qftf_func)
// =============================================================================

/// Static recursive guard for `rs_call_qftf_func`.
/// `call_qftf_func` must not be called recursively (it would not work properly).
static QFTF_RECURSIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Call the `quickfixtextfunc` callback to get display text for quickfix entries.
///
/// Mirrors C `call_qftf_func`. Returns a `list_T *` (or NULL if not set / recursive).
/// The caller owns the returned list reference (from `tv_list_ref`).
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T`
#[no_mangle]
pub unsafe extern "C" fn rs_call_qftf_func(
    qfl: *mut c_void,
    qf_winid: c_int,
    start_idx: LinenrT,
    end_idx: c_int,
) -> ListPtr {
    use std::sync::atomic::Ordering;

    if QFTF_RECURSIVE.load(Ordering::Relaxed) {
        return std::ptr::null_mut();
    }
    QFTF_RECURSIVE.store(true, Ordering::Relaxed);

    let result = call_qftf_func_inner(qfl, qf_winid, start_idx, end_idx);

    QFTF_RECURSIVE.store(false, Ordering::Relaxed);
    result
}

unsafe fn call_qftf_func_inner(
    qfl: *mut c_void,
    qf_winid: c_int,
    start_idx: LinenrT,
    end_idx: c_int,
) -> ListPtr {
    // Choose the callback: per-list takes precedence over global.
    let cb: *mut c_void = {
        let local_cb = nvim_qfl_get_qftf_cb_ptr(qfl);
        if nvim_callback_is_none(local_cb) {
            nvim_qf_get_global_qftf_cb_ptr()
        } else {
            local_cb
        }
    };

    if nvim_callback_is_none(cb) {
        return std::ptr::null_mut();
    }

    // Build the dict argument: { quickfix, winid, id, start_idx, end_idx }
    // VAR_FIXED = 2
    let dict = tv_dict_alloc_lock(2);
    if dict.is_null() {
        return std::ptr::null_mut();
    }

    let is_qf = nvim_qf_is_qf_list(qfl);
    let qf_id = nvim_qf_get_id(qfl);

    tv_dict_add_nr(dict, c"quickfix".as_ptr(), 8, i64::from(is_qf));
    tv_dict_add_nr(dict, c"winid".as_ptr(), 5, i64::from(qf_winid));
    tv_dict_add_nr(dict, c"id".as_ptr(), 2, i64::from(qf_id));
    tv_dict_add_nr(dict, c"start_idx".as_ptr(), 9, i64::from(start_idx));
    tv_dict_add_nr(dict, c"end_idx".as_ptr(), 7, i64::from(end_idx));
    // Increment refcount before passing to callback (matches C code pattern)
    nvim_tv_dict_incr_refcount(dict);

    // Allocate a zeroed typval_T for the return value (heap-allocated for safety)
    // We use a stack array matching the C layout: VarType(4) + VarLock(4) + union(8) = 16 bytes
    // Use 24 bytes for alignment safety.
    let mut rettv_buf = [0u8; 24];
    let rettv: *mut c_void = rettv_buf.as_mut_ptr().cast();

    let mut qftf_list: ListPtr = std::ptr::null_mut();

    if nvim_callback_call_one_dict(cb, dict, rettv) {
        let list_ptr = nvim_tv_rettv_list_if_var_list(rettv);
        if !list_ptr.is_null() {
            nvim_tv_list_ref(list_ptr);
            qftf_list = list_ptr;
        }
        tv_clear(rettv);
    }

    tv_dict_unref(dict);
    qftf_list
}

// =============================================================================
// Phase 3: qf_buf_add_line (migrated from quickfix_shim.c)
// =============================================================================

/// Append text to a Vec<u8>, replacing newlines with spaces and skipping
/// subsequent whitespace (mirrors C `qf_fmt_text`).
fn qf_fmt_text(buf: &mut Vec<u8>, text: *const c_char) {
    // Safety: caller guarantees text is a valid C string
    let bytes = unsafe { std::ffi::CStr::from_ptr(text).to_bytes() };
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            buf.push(b' ');
            i += 1;
            // skip following whitespace and newlines
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n') {
                i += 1;
            }
        } else {
            buf.push(bytes[i]);
            i += 1;
        }
    }
}

/// Format the range (lnum, end_lnum, col, end_col) into buf
/// (mirrors C `qf_range_text`).
fn qf_range_text(buf: &mut Vec<u8>, lnum: LinenrT, end_lnum: LinenrT, col: c_int, end_col: c_int) {
    // Format lnum
    let lnum_str = format!("{lnum}");
    buf.extend_from_slice(lnum_str.as_bytes());

    if end_lnum > 0 && lnum != end_lnum {
        let s = format!("-{end_lnum}");
        buf.extend_from_slice(s.as_bytes());
    }
    if col > 0 {
        let s = format!(" col {col}");
        buf.extend_from_slice(s.as_bytes());
        if end_col > 0 && col != end_col {
            let s = format!("-{end_col}");
            buf.extend_from_slice(s.as_bytes());
        }
    }
}

/// Add a formatted line for a quickfix entry to the quickfix buffer.
///
/// Mirrors C `qf_buf_add_line`. Returns `OK` on success, `FAIL` on error.
///
/// # Safety
///
/// - All pointer arguments that are non-null must be valid
/// - `dirname` must point to a writable `MAXPATHL`-byte buffer
unsafe fn qf_buf_add_line(
    buf: BufHandle,
    lnum: LinenrT,
    qfp: QfLinePtr,
    dirname: &mut [u8; MAXPATHL],
    qftf_str: *mut c_char,
    first_bufline: bool,
) -> c_int {
    let mut line: Vec<u8> = Vec::with_capacity(256);

    // If quickfixtextfunc returned a custom string, use it directly.
    if !qftf_str.is_null() && *qftf_str != 0 {
        let s = std::ffi::CStr::from_ptr(qftf_str).to_bytes();
        line.extend_from_slice(s);
    } else {
        let module = nvim_qfline_get_module(qfp);
        if !module.is_null() && *module != 0 {
            let s = std::ffi::CStr::from_ptr(module).to_bytes();
            line.extend_from_slice(s);
        } else {
            let fnum = nvim_qfline_get_fnum(qfp);
            if fnum != 0 {
                let errbuf = nvim_buflist_findnr(fnum);
                if !errbuf.is_null() {
                    let b_fname = qf_buf_get_fname_fill(errbuf.cast_const());
                    if !b_fname.is_null() {
                        let qf_type = nvim_qfline_get_type(qfp);
                        if qf_type == 1 {
                            // :helpgrep -- use filename tail only
                            let tail = path_tail(b_fname);
                            let s = std::ffi::CStr::from_ptr(tail).to_bytes();
                            line.extend_from_slice(s);
                        } else {
                            // Shorten the file name if not done already.
                            // For optimization, only for the first entry in a buffer.
                            if first_bufline {
                                let sfname = nvim_buf_get_sfname(errbuf);
                                if sfname.is_null() || path_is_absolute(sfname) {
                                    if dirname[0] == 0 {
                                        os_dirname(dirname.as_mut_ptr().cast(), MAXPATHL);
                                    }
                                    shorten_buf_fname(errbuf, dirname.as_mut_ptr().cast(), 0);
                                }
                            }
                            let qf_fname = nvim_qfline_get_fname(qfp);
                            let fname = if qf_fname.is_null() {
                                b_fname
                            } else {
                                qf_fname
                            };
                            let s = std::ffi::CStr::from_ptr(fname).to_bytes();
                            line.extend_from_slice(s);
                        }
                    }
                }
            }
        }

        line.push(b'|');

        let qf_lnum = nvim_qfline_get_lnum(qfp);
        let qf_pattern = nvim_qfline_get_pattern(qfp);
        if qf_lnum > 0 {
            let end_lnum = nvim_qfline_get_end_lnum(qfp);
            let col = nvim_qfline_get_col(qfp);
            let end_col = nvim_qfline_get_end_col(qfp);
            qf_range_text(&mut line, qf_lnum, end_lnum, col, end_col);
            // Use pure-Rust type formatter directly (no C round-trip needed)
            let type_c = c_int::from(nvim_qfline_get_type(qfp));
            let type_nr = nvim_qfline_get_nr(qfp);
            let mut type_buf = [0u8; 20];
            let written = qf_types_fmt(type_c, type_nr, &mut type_buf);
            if written > 0 {
                line.extend_from_slice(&type_buf[..written]);
            }
        } else if !qf_pattern.is_null() {
            qf_fmt_text(&mut line, qf_pattern);
        }
        line.push(b'|');
        line.push(b' ');

        // Remove newlines and leading whitespace from the text.
        // For an unrecognized line keep the indent, the compiler may mark a word with ^^^^.
        let qf_text = nvim_qfline_get_text(qfp);
        if !qf_text.is_null() {
            let text: *const c_char = if line.len() > 3 {
                skipwhite(qf_text).cast_const()
            } else {
                qf_text
            };
            qf_fmt_text(&mut line, text);
        }
    }

    // Append NUL terminator and write to buffer
    line.push(0u8);
    let len = c_int::try_from(line.len()).unwrap_or(c_int::MAX);
    let ret = ml_append_buf(buf, lnum, line.as_mut_ptr().cast(), len, false);
    if ret == FAIL {
        FAIL
    } else {
        OK
    }
}

/// Fill the quickfix buffer with entries.
///
/// # Safety
///
/// - `buf` must be a valid pointer to a `buf_T`
/// - `qfl` may be NULL (in which case only cleanup is done)
/// - `old_last` may be NULL (full refresh) or a valid `qfline_T` pointer
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_qf_fill_buffer(
    qfl: *mut c_void,
    buf: BufHandle,
    old_last: QfLinePtr,
    qf_winid: c_int,
) {
    let old_key_typed = nvim_qf_get_key_typed();

    if old_last.is_null() {
        if !nvim_qf_buf_is_curbuf(buf) {
            internal_error(c"rs_qf_fill_buffer()".as_ptr());
            return;
        }

        // Delete all existing lines
        if !nvim_qf_delete_all_lines() {
            return;
        }

        nvim_qf_zero_skipcol_for_curbuf();
        nvim_qf_u_clearallandblockfree();
    }

    // Check if there is anything to display
    let qf_start = nvim_qf_get_start(qfl);
    if !qfl.is_null() && !qf_start.is_null() {
        let mut dirname = [0u8; MAXPATHL];

        let mut lnum: LinenrT;
        let mut qfp: QfLinePtr;

        // Add one line for each error
        if old_last.is_null() {
            qfp = qf_start;
            lnum = 0;
        } else {
            let next = nvim_qfline_get_next(old_last);
            qfp = if next.is_null() { old_last } else { next };
            lnum = nvim_buf_get_line_count(buf);
        }

        let qf_count = nvim_qf_get_count(qfl);
        let qftf_list = rs_call_qftf_func(qfl, qf_winid, lnum + 1, qf_count);
        let mut qftf_li = nvim_tv_list_first(qftf_list.cast_const());

        let mut prev_bufnr: c_int = -1;
        let mut invalid_val = false;

        while lnum < qf_count {
            let mut qftf_str: *mut c_char = std::ptr::null_mut();

            // Use the text supplied by the user defined function (if any).
            if !qftf_li.is_null() && !invalid_val {
                qftf_str = nvim_tv_list_item_string(qftf_li);
                if qftf_str.is_null() {
                    invalid_val = true;
                }
            }

            if qf_buf_add_line(
                buf,
                lnum,
                qfp,
                &mut dirname,
                qftf_str,
                prev_bufnr != nvim_qfline_get_fnum(qfp),
            ) == FAIL
            {
                break;
            }
            prev_bufnr = nvim_qfline_get_fnum(qfp);
            lnum += 1;
            qfp = nvim_qfline_get_next(qfp);
            if qfp.is_null() {
                break;
            }

            if !qftf_li.is_null() {
                qftf_li = nvim_tv_list_item_next(qftf_list.cast_const(), qftf_li);
            }
        }

        if old_last.is_null() {
            // Delete the empty line which is now at the end
            ml_delete(lnum + 1);
        }
    }

    // Correct cursor position.
    rs_check_lnums(1);

    if old_last.is_null() {
        // Inlined nvim_qf_set_filetype_and_autocmds (Phase 14)
        nvim_qf_curbuf_incr_ro_locked();
        nvim_qf_set_option_filetype_qf();
        nvim_qf_curbuf_set_ma_false();
        nvim_qf_curbuf_set_keep_filetype(true);
        apply_autocmds(
            EVENT_BUFREADPOST,
            c"quickfix".as_ptr().cast_mut(),
            std::ptr::null_mut(),
            false,
            curbuf_fill,
        );
        apply_autocmds(
            EVENT_BUFWINENTER,
            c"quickfix".as_ptr().cast_mut(),
            std::ptr::null_mut(),
            false,
            curbuf_fill,
        );
        nvim_qf_curbuf_set_keep_filetype(false);
        nvim_qf_curbuf_decr_ro_locked();
        nvim_qf_redraw_curbuf_later();
    }

    // Restore KeyTyped, setting 'filetype' may reset it.
    nvim_qf_set_key_typed(old_key_typed);
}

// =============================================================================
// Phase 1: rs_qf_msg — format and display quickfix list summary message
// =============================================================================

type QfInfoHandleConst = *const c_void;

extern "C" {
    fn nvim_qf_get_listcount(qi: QfInfoHandleConst) -> c_int;
    fn nvim_qf_get_list_at(qi: QfInfoHandleConst, idx: c_int) -> *const c_void;
    fn nvim_qf_get_title(qfl: *const c_void) -> *const c_char;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize) -> usize;
    fn trunc_string(s: *const c_char, buf: *mut c_char, room_in: c_int, buflen: c_int);
    fn msg(s: *const std::ffi::c_char, hl_id: c_int) -> bool;
    static Columns: c_int;
    // (nvim_qf_trunc_and_msg deleted: inlined below)
}
// Note: nvim_qf_get_count is already declared in the Phase 3 extern block above.

/// FFI export: format and display the quickfix/location list summary message.
///
/// Equivalent to C `qf_msg`. Formats:
///   "<lead>error list <which+1> of <listcount>; <count> errors [<title padded to col 34>]"
/// then truncates to `Columns - 1` and calls `msg`.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T`
/// - `lead` must be a valid null-terminated C string (may be empty)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_msg(qi: QfInfoHandleConst, which: c_int, lead: *const c_char) {
    if qi.is_null() {
        return;
    }

    let lead_str = if lead.is_null() {
        ""
    } else {
        std::ffi::CStr::from_ptr(lead).to_str().unwrap_or_default()
    };

    let listcount = nvim_qf_get_listcount(qi);
    let qfl = nvim_qf_get_list_at(qi, which);
    let count = if qfl.is_null() {
        0
    } else {
        nvim_qf_get_count(qfl)
    };
    let title_ptr = if qfl.is_null() {
        std::ptr::null()
    } else {
        nvim_qf_get_title(qfl)
    };

    // Build the base message: "<lead>error list <n> of <total>; <count> errors "
    let base = format!(
        "{}error list {} of {}; {} errors ",
        lead_str,
        which + 1,
        listcount,
        count
    );

    let mut msg_buf = base;

    // Append title if present, padding base to at least 34 chars
    if !title_ptr.is_null() {
        let title = std::ffi::CStr::from_ptr(title_ptr);
        if let Ok(title_str) = title.to_str() {
            // Pad to 34 characters
            if msg_buf.len() < 34 {
                let padding = 34 - msg_buf.len();
                for _ in 0..padding {
                    msg_buf.push(' ');
                }
            }
            msg_buf.push_str(title_str);
        }
    }

    // Null-terminate for C
    msg_buf.push('\0');

    // Inline nvim_qf_trunc_and_msg: truncate to Columns-1 and display
    // IOSIZE = 1025 (same as C's IOSIZE for trunc_string)
    let mut buf = [0u8; 1025];
    xstrlcpy(
        buf.as_mut_ptr().cast::<c_char>(),
        msg_buf.as_ptr().cast::<c_char>(),
        1025,
    );
    trunc_string(
        buf.as_ptr().cast::<c_char>(),
        buf.as_mut_ptr().cast::<c_char>(),
        Columns - 1,
        1025,
    );
    msg(buf.as_ptr().cast::<c_char>(), 0);
}

// =============================================================================
// Phase 3: rs_qf_list_entry — display a single quickfix entry for :clist/:llist
// =============================================================================

const IOSIZE: usize = 1025; // 1024 + 1
                            // HLF_QFL constant (from highlight_defs.h, validated by _Static_assert in quickfix_shim.c)
const HLF_QFL: c_int = 58;

extern "C" {
    // Phase 3: message output accessors
    fn message_filtered(str: *const c_char) -> bool;
    // Direct message output (Phase 14: replacing nvim_qf_list_entry_output)
    fn msg_putchar(c: c_int);
    fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn msg_puts_hl(s: *const c_char, attr: c_int, right: bool);
    fn msg_puts(s: *const c_char);
    // (nvim_msg_outtrans_attr, nvim_msg_puts_plain deleted: use msg_outtrans/msg_puts directly)
    fn msg_prt_line(s: *const c_char, list: bool);
    // nvim_hlf_qfl deleted: use HLF_QFL constant directly

    // Phase 3: qfline field accessors (already in lib.rs, re-declare locally)
    #[link_name = "nvim_qfline_get_module"]
    fn qfline_get_module_p3(qfp: QfLinePtr) -> *const c_char;
    #[link_name = "nvim_qfline_get_fnum"]
    fn qfline_get_fnum_p3(qfp: QfLinePtr) -> c_int;
    #[link_name = "nvim_qfline_get_fname"]
    fn qfline_get_fname_p3(qfp: QfLinePtr) -> *const c_char;
    #[link_name = "nvim_qfline_get_type"]
    fn qfline_get_type_p3(qfp: QfLinePtr) -> c_char;
    #[link_name = "nvim_qfline_get_lnum"]
    fn qfline_get_lnum_p3(qfp: QfLinePtr) -> LinenrT;
    #[link_name = "nvim_qfline_get_end_lnum"]
    fn qfline_get_end_lnum_p3(qfp: QfLinePtr) -> LinenrT;
    #[link_name = "nvim_qfline_get_col"]
    fn qfline_get_col_p3(qfp: QfLinePtr) -> c_int;
    #[link_name = "nvim_qfline_get_end_col"]
    fn qfline_get_end_col_p3(qfp: QfLinePtr) -> c_int;
    #[link_name = "nvim_qfline_get_nr"]
    fn qfline_get_nr_p3(qfp: QfLinePtr) -> c_int;
    #[link_name = "nvim_qfline_get_pattern"]
    fn qfline_get_pattern_p3(qfp: QfLinePtr) -> *const c_char;
    #[link_name = "nvim_qfline_get_text"]
    fn qfline_get_text_p3(qfp: QfLinePtr) -> *const c_char;
    #[link_name = "rs_buflist_findnr"]
    fn buflist_findnr_p3(nr: c_int) -> *mut c_void;
    #[link_name = "nvim_qf_buf_get_fname"]
    fn buf_get_fname_p3(buf: *const c_void) -> *const c_char;
    #[link_name = "path_tail"]
    fn path_tail_p3(fname: *const c_char) -> *mut c_char;
    #[link_name = "skipwhite"]
    fn skipwhite_p3(str: *const c_char) -> *mut c_char;

}

/// Display a single quickfix entry for :clist/:llist.
///
/// Mirrors C `qf_list_entry`. Reads qfline fields via accessors, filters via
/// `message_filtered`, then delegates message output to `nvim_qf_list_entry_output`.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_qf_list_entry(
    qfp: QfLinePtr,
    qf_idx: c_int,
    cursel: bool,
    qf_file_hl_id: c_int,
    qf_sep_hl_id: c_int,
    qf_line_hl_id: c_int,
) {
    use std::io::Write;

    // --- Determine the filename or module to display as prefix ---
    let module = qfline_get_module_p3(qfp);
    let module_nonempty =
        !module.is_null() && !std::ffi::CStr::from_ptr(module).to_bytes().is_empty();

    // Get display name: module > qf_fname/buf->b_fname > none
    let display_name: *const c_char = if module_nonempty {
        module
    } else {
        let fnum = qfline_get_fnum_p3(qfp);
        if fnum != 0 {
            let buf = buflist_findnr_p3(fnum);
            if buf.is_null() {
                std::ptr::null()
            } else {
                let b_fname = buf_get_fname_p3(buf.cast_const());
                if b_fname.is_null() {
                    std::ptr::null()
                } else {
                    let qf_fname = qfline_get_fname_p3(qfp);
                    let fname = if qf_fname.is_null() {
                        b_fname
                    } else {
                        qf_fname
                    };
                    let qf_type = qfline_get_type_p3(qfp);
                    if qf_type == 1 {
                        // :helpgrep — use tail only
                        path_tail_p3(fname)
                    } else {
                        fname
                    }
                }
            }
        } else {
            std::ptr::null()
        }
    };

    // --- Build the prefix string (e.g. " 1 filename" or " 1") ---
    // Replaces nvim_qf_format_prefix (deleted in Phase 14).
    let prefix_string: std::ffi::CString = if display_name.is_null() {
        let s = format!("{qf_idx:2}\0");
        std::ffi::CString::from_vec_with_nul(s.into_bytes()).unwrap_or_default()
    } else {
        let name_bytes = std::ffi::CStr::from_ptr(display_name).to_bytes();
        let name_str = String::from_utf8_lossy(name_bytes);
        let s = format!("{qf_idx:2} {name_str}\0");
        std::ffi::CString::from_vec_with_nul(s.into_bytes()).unwrap_or_default()
    };

    // --- Filtering ---
    // filter_entry starts true; each check ANDs in (filtered != 0).
    // If all checks return non-zero, the entry is filtered OUT (return early).
    let mut filter_entry = true;
    if module_nonempty {
        filter_entry &= message_filtered(module);
    }
    if filter_entry && !display_name.is_null() && !module_nonempty {
        filter_entry &= message_filtered(display_name);
    }
    let pattern = qfline_get_pattern_p3(qfp);
    if filter_entry && !pattern.is_null() {
        filter_entry &= message_filtered(pattern);
    }
    let text = qfline_get_text_p3(qfp);
    if filter_entry {
        filter_entry &= message_filtered(text);
    }
    if filter_entry {
        return;
    }

    // --- Build intermediate text buffers ---
    let lnum = qfline_get_lnum_p3(qfp);
    let mut range_buf = [0u8; IOSIZE];
    let range_len = if lnum != 0 {
        let end_lnum = qfline_get_end_lnum_p3(qfp);
        let col = qfline_get_col_p3(qfp);
        let end_col = qfline_get_end_col_p3(qfp);
        // Replicate qf_range_text logic inline
        let mut cur = std::io::Cursor::new(&mut range_buf[..]);
        let _ = write!(cur, "{lnum}");
        if end_lnum > 0 && lnum != end_lnum {
            let _ = write!(cur, "-{end_lnum}");
        }
        if col > 0 {
            let _ = write!(cur, " col {col}");
            if end_col > 0 && col != end_col {
                let _ = write!(cur, "-{end_col}");
            }
        }
        cur.position() as usize
    } else {
        0
    };

    let mut type_buf_arr = [0u8; 20];
    // Call the pure Rust type formatter directly (no FFI round-trip needed)
    qf_types_fmt(
        c_int::from(qfline_get_type_p3(qfp)),
        qfline_get_nr_p3(qfp),
        &mut type_buf_arr,
    );
    // type_buf_arr is now a NUL-terminated string (used directly below)

    let mut pattern_buf = [0u8; IOSIZE];
    let pattern_len = if pattern.is_null() {
        0
    } else {
        // Replicate qf_fmt_text logic: replace newlines with spaces, collapse whitespace
        let src_bytes = std::ffi::CStr::from_ptr(pattern).to_bytes();
        let mut i = 0usize;
        let mut out_pos = 0usize;
        while i < src_bytes.len() && out_pos < IOSIZE - 1 {
            if src_bytes[i] == b'\n' {
                pattern_buf[out_pos] = b' ';
                out_pos += 1;
                i += 1;
                while i < src_bytes.len()
                    && (src_bytes[i] == b' ' || src_bytes[i] == b'\t' || src_bytes[i] == b'\n')
                {
                    i += 1;
                }
            } else {
                pattern_buf[out_pos] = src_bytes[i];
                out_pos += 1;
                i += 1;
            }
        }
        pattern_buf[out_pos] = 0;
        out_pos
    };

    // body: skip leading whitespace if we have a file or line
    let body_src = if text.is_null() {
        std::ptr::null()
    } else if !display_name.is_null() || lnum != 0 {
        skipwhite_p3(text)
    } else {
        text
    };
    let mut body_buf = [0u8; IOSIZE];
    // body_buf is NUL-terminated after this block
    if !body_src.is_null() {
        let src_bytes = std::ffi::CStr::from_ptr(body_src).to_bytes();
        let mut i = 0usize;
        let mut out_pos = 0usize;
        while i < src_bytes.len() && out_pos < IOSIZE - 1 {
            if src_bytes[i] == b'\n' {
                body_buf[out_pos] = b' ';
                out_pos += 1;
                i += 1;
                while i < src_bytes.len()
                    && (src_bytes[i] == b' ' || src_bytes[i] == b'\t' || src_bytes[i] == b'\n')
                {
                    i += 1;
                }
            } else {
                body_buf[out_pos] = src_bytes[i];
                out_pos += 1;
                i += 1;
            }
        }
        body_buf[out_pos] = 0;
    }

    // --- Direct message output (Phase 14: inlined from nvim_qf_list_entry_output) ---
    msg_putchar(c_int::from(b'\n'));
    let prefix_hl = if cursel { HLF_QFL } else { qf_file_hl_id };
    msg_outtrans(prefix_string.as_ptr(), prefix_hl, false);

    if lnum != 0 {
        msg_puts_hl(c":".as_ptr(), qf_sep_hl_id, false);
    }

    // Build range+type combined NUL-terminated buffer
    let mut combined: Vec<u8> = Vec::with_capacity(range_len + 20 + 1);
    if lnum != 0 && range_len > 0 {
        combined.extend_from_slice(&range_buf[..range_len]);
    }
    // type_buf_arr is already NUL-terminated; append the bytes up to (not including) the NUL
    let type_len = type_buf_arr
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(type_buf_arr.len());
    combined.extend_from_slice(&type_buf_arr[..type_len]);
    combined.push(0u8); // NUL terminate
    msg_puts_hl(combined.as_ptr().cast::<c_char>(), qf_line_hl_id, false);
    msg_puts_hl(c":".as_ptr(), qf_sep_hl_id, false);

    if !pattern.is_null() && pattern_len > 0 {
        // pattern_buf is already NUL-terminated at pattern_len
        msg_puts(pattern_buf.as_ptr().cast::<c_char>());
        msg_puts_hl(c":".as_ptr(), qf_sep_hl_id, false);
    }

    msg_puts(c" ".as_ptr());

    // body_buf is already NUL-terminated at body_len (or just NUL if empty)
    msg_prt_line(body_buf.as_ptr().cast::<c_char>(), false);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lnum_display_width() {
        assert_eq!(lnum_display_width(0), 1);
        assert_eq!(lnum_display_width(1), 1);
        assert_eq!(lnum_display_width(9), 1);
        assert_eq!(lnum_display_width(10), 2);
        assert_eq!(lnum_display_width(99), 2);
        assert_eq!(lnum_display_width(100), 3);
        assert_eq!(lnum_display_width(1000), 4);
        assert_eq!(lnum_display_width(10000), 5);
    }

    #[test]
    fn test_type_display_char() {
        assert_eq!(type_display_char(b'E'), b'E');
        assert_eq!(type_display_char(b'e'), b'E');
        assert_eq!(type_display_char(b'W'), b'W');
        assert_eq!(type_display_char(b'w'), b'W');
        assert_eq!(type_display_char(b'I'), b'I');
        assert_eq!(type_display_char(b'N'), b'N');
        assert_eq!(type_display_char(b'X'), b' ');
        assert_eq!(type_display_char(0), b' ');
    }

    #[test]
    fn test_is_error_type() {
        assert!(is_error_type(b'E'));
        assert!(is_error_type(b'e'));
        assert!(!is_error_type(b'W'));
        assert!(!is_error_type(b'I'));
    }

    #[test]
    fn test_is_warning_type() {
        assert!(is_warning_type(b'W'));
        assert!(is_warning_type(b'w'));
        assert!(!is_warning_type(b'E'));
        assert!(!is_warning_type(b'I'));
    }

    #[test]
    fn test_display_state_has_entries() {
        let empty = QfDisplayState::default();
        assert!(!empty.has_entries());

        let with_entries = QfDisplayState {
            total_entries: 5,
            ..Default::default()
        };
        assert!(with_entries.has_entries());
    }

    #[test]
    fn test_display_state_needs_scrolling() {
        let fits = QfDisplayState {
            total_entries: 10,
            visible_lines: 20,
            ..Default::default()
        };
        assert!(!fits.needs_scrolling());

        let needs = QfDisplayState {
            total_entries: 30,
            visible_lines: 20,
            ..Default::default()
        };
        assert!(needs.needs_scrolling());
    }

    #[test]
    fn test_scroll_percent() {
        // All visible
        let all = QfDisplayState {
            total_entries: 10,
            visible_lines: 20,
            first_visible: 1,
            ..Default::default()
        };
        assert_eq!(all.scroll_percent(), 100);

        // Halfway through
        let half = QfDisplayState {
            total_entries: 100,
            visible_lines: 20,
            first_visible: 31,
            ..Default::default()
        };
        assert_eq!(half.scroll_percent(), 50);

        // At end
        let end = QfDisplayState {
            total_entries: 100,
            visible_lines: 20,
            first_visible: 81,
            ..Default::default()
        };
        assert_eq!(end.scroll_percent(), 100);
    }

    #[test]
    fn test_display_format_default() {
        assert_eq!(QfDisplayFormat::default(), QfDisplayFormat::Standard);
    }

    #[test]
    fn test_display_entry_default() {
        let entry = QfDisplayEntry::default();
        assert_eq!(entry.index, 0);
        assert_eq!(entry.type_char, b' ');
        assert!(!entry.valid);
        assert!(!entry.is_current);
    }
}
