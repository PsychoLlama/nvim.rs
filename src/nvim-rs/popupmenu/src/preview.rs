//! Popup menu preview window management.
//!
//! This module handles the floating preview window that shows
//! completion item info text alongside the popup menu.

use std::ffi::{c_char, c_int, c_void};

use crate::display::{BufHandle, WinHandle};
use crate::PUM_STATE;

// ---- Minimal API types for nvim_buf_set_lines ----

/// `NvimString` (matches C `String` / `NvimString`).
#[repr(C)]
#[derive(Clone, Copy)]
struct NvimString {
    data: *mut c_char,
    size: usize,
}

/// Object type discriminant — only need kObjectTypeString (4).
const K_OBJECT_TYPE_STRING: c_int = 4;

/// Object data union — only use the string variant.
#[repr(C)]
#[derive(Clone, Copy)]
union ObjectData {
    string: NvimString,
    _integer: i64,
}

/// Object (matches C `Object`).
#[repr(C)]
#[derive(Clone, Copy)]
struct NvimObject {
    obj_type: c_int,
    data: ObjectData,
}

/// Array (matches C `Array` kvec).
#[repr(C)]
#[derive(Clone, Copy)]
struct NvimArray {
    size: usize,
    capacity: usize,
    items: *mut NvimObject,
}

impl NvimArray {
    const fn empty() -> Self {
        Self {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        }
    }

    /// Push a string object into the array.
    ///
    /// # Safety
    /// Array must have been allocated with sufficient capacity.
    unsafe fn push_string(&mut self, s: NvimString) {
        debug_assert!(self.size < self.capacity);
        *self.items.add(self.size) = NvimObject {
            obj_type: K_OBJECT_TYPE_STRING,
            data: ObjectData { string: s },
        };
        self.size += 1;
    }
}

/// Arena allocator (matches C `Arena` from `memory_defs.h`).
/// Layout: `cur_blk` (pointer), `pos` (`size_t`), `size` (`size_t`).
#[repr(C)]
struct Arena {
    cur_blk: *mut c_char,
    pos: usize,
    size: usize,
}

impl Arena {
    const fn empty() -> Self {
        Self {
            cur_blk: std::ptr::null_mut(),
            pos: 0,
            size: 0,
        }
    }
}

// ---- FFI declarations ----

extern "C" {
    /// Get display width in cells of a NUL-terminated multibyte string.
    fn mb_string2cells(s: *const c_char) -> usize;
    /// Duplicate a C string into a newly allocated `NvimString`.
    fn cstr_to_string(s: *const c_char) -> NvimString;
    /// Free an Array (recursively frees items).
    fn api_free_array(arr: NvimArray);
    /// Set lines in a buffer.
    fn nvim_buf_set_lines(
        channel_id: u64,
        buffer: c_int,
        start: i64,
        end: i64,
        strict_indexing: bool,
        replacement: NvimArray,
        arena: *mut Arena,
        err: *mut NvimError,
    );
    /// Finish arena and return memory block.
    fn arena_finish(arena: *mut Arena) -> *mut c_void;
    /// Free arena memory block.
    fn arena_mem_free(mem: *mut c_void);
    /// Set buffer `b_p_ma` field.
    fn nvim_buf_set_b_p_ma(buf: *mut BufHandle, val: c_int);
    /// Get buffer handle (`b_fnum` field as integer).
    fn nvim_buf_get_handle(buf: *mut BufHandle) -> c_int;
    /// Emit an error message.
    fn emsg(s: *const c_char);
    /// Clear an error.
    fn api_clear_error(err: *mut NvimError);
    /// Allocate memory.
    fn xmalloc(size: usize) -> *mut c_void;
}

extern "C" {
    /// C global: `textlock` (prevents buffer text changes during certain ops).
    static mut textlock: c_int;
}

/// Error struct (matches C `Error`).
#[repr(C)]
struct NvimError {
    err_type: c_int,
    msg: *mut c_char,
}

impl NvimError {
    const fn init() -> Self {
        Self {
            err_type: -1,
            msg: std::ptr::null_mut(),
        }
    }

    const fn is_set(&self) -> bool {
        self.err_type != -1
    }
}

// C accessor functions for preview window operations.
extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    /// Check if selected item matches current completion selection.
    fn rs_compl_match_curr_select(selected: c_int) -> c_int;
    /// Block autocmds.
    fn block_autocmds();
    /// Unblock autocmds.
    fn unblock_autocmds();
    /// Find the floating preview window.
    fn win_float_find_preview() -> *mut WinHandle;
    /// Create a floating window.
    fn win_float_create(enter: bool, new_buf: bool) -> *mut WinHandle;
    /// Set `w_topline` for a window (from `win_struct.rs`).
    fn nvim_win_set_topline(wp: *mut WinHandle, val: c_int);
    /// Set `w_p_wfb` for a window.
    fn nvim_win_set_p_wfb(wp: *mut WinHandle, val: c_int);
    /// Get buffer from a window.
    fn nvim_win_get_buffer(wp: *mut WinHandle) -> *mut BufHandle;
    /// Call `redraw_later` for a window.
    fn redraw_later(wp: *mut WinHandle, update_type: c_int);
}

extern "C" {
    /// C global: `RedrawingDisabled` counter.
    static mut RedrawingDisabled: c_int;
    /// C global: `no_u_sync` counter.
    static mut no_u_sync: c_int;
}

// C accessor functions for adjust_info_position.
extern "C" {
    /// Get `Columns`.
    /// Get `Rows`.
    /// Get line count for window's buffer (from `window_shim.c`).
    fn nvim_win_buf_line_count(wp: *mut WinHandle) -> c_int;
    /// `plines_m_win` directly.
    fn plines_m_win(wp: *mut WinHandle, first: c_int, last: c_int, max: c_int) -> c_int;
    /// Set window config fields and apply via `win_config_float`.
    fn nvim_pum_win_config_set_and_apply(
        wp: *mut WinHandle,
        width: c_int,
        col: c_int,
        anchor: c_int,
        height: c_int,
        row: c_int,
        hide: c_int,
    );
    /// Get border width from Rust.
    fn rs_pum_border_width() -> c_int;
}

/// `UPD_NOT_VALID` from drawscreen.h.
const UPD_NOT_VALID: c_int = 40;

/// `kFloatAnchorSouth` from `buffer_defs.h`.
const K_FLOAT_ANCHOR_SOUTH: c_int = 2;

/// Set the informational text in the preview buffer.
///
/// Iterates through the `info` C string line-by-line, builds an Array of
/// String objects, and calls `nvim_buf_set_lines` to replace the buffer content.
/// Handles textlock save/restore and error reporting.
///
/// # Safety
/// All pointers must be valid. `info` must be a valid, mutable C string
/// (temporarily modified in-place during iteration, always restored).
#[no_mangle]
pub unsafe extern "C" fn rs_pum_preview_set_text(
    buf: *mut BufHandle,
    info: *mut c_char,
    lnum: *mut i32,
    max_width: *mut c_int,
) {
    let mut err = NvimError::init();
    let mut arena = Arena::empty();
    let mut replacement = NvimArray::empty();

    nvim_buf_set_b_p_ma(buf, 1);

    // First pass: count lines so we can allocate the array.
    let mut line_count: usize = 0;
    {
        let mut curr = info;
        loop {
            let next = libc_strchr(curr, b'\n' as c_int);
            let is_last = next.is_null();
            // Only skip if this is an empty line AND it's the last line
            if *curr == 0 && is_last {
                break;
            }
            line_count += 1;
            if is_last {
                break;
            }
            curr = next.add(1);
        }
    }

    if line_count > 0 {
        // Allocate array storage on the heap (not arena, since we pass it to nvim_buf_set_lines).
        let items_ptr =
            xmalloc(line_count * std::mem::size_of::<NvimObject>()).cast::<NvimObject>();
        replacement = NvimArray {
            size: 0,
            capacity: line_count,
            items: items_ptr,
        };

        // Second pass: fill array.
        let mut curr = info;
        loop {
            let next = libc_strchr(curr, b'\n' as c_int);
            let is_last = next.is_null();
            if *curr == 0 && is_last {
                break;
            }

            // Temporarily NUL-terminate at newline boundary.
            if !next.is_null() {
                *next = 0;
            }

            // Compute display width.
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let cells = mb_string2cells(curr) as c_int;
            if cells > *max_width {
                *max_width = cells;
            }

            replacement.push_string(cstr_to_string(curr));
            *lnum += 1;

            // Restore newline.
            if next.is_null() {
                break;
            }
            *next = 10i8; // '\n'
            curr = next.add(1);
        }
    }

    // Save/restore textlock around nvim_buf_set_lines.
    let original_textlock = textlock;
    if textlock > 0 {
        textlock = 0;
    }
    let buf_handle = nvim_buf_get_handle(buf);
    nvim_buf_set_lines(
        0,
        buf_handle,
        0,
        -1,
        false,
        replacement,
        &raw mut arena,
        &raw mut err,
    );
    textlock = original_textlock;

    if err.is_set() {
        emsg(err.msg);
        api_clear_error(&raw mut err);
    }
    let mem = arena_finish(&raw mut arena);
    arena_mem_free(mem);
    api_free_array(replacement);

    nvim_buf_set_b_p_ma(buf, 0);
}

/// Thin C wrapper for `strchr` to avoid `libc` dependency.
unsafe fn libc_strchr(s: *mut c_char, c: c_int) -> *mut c_char {
    extern "C" {
        fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    }
    strchr(s, c)
}

/// Adjust floating info preview window position.
///
/// Calculates the optimal position for the info preview window
/// relative to the popup menu, placing it to the right or left
/// depending on available space.
///
/// # Safety
/// `wp` must be a valid `win_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_pum_adjust_info_position(wp: *mut WinHandle, width: c_int) {
    let border_width = rs_pum_border_width();
    let pum_col = PUM_STATE.col;
    let pum_width = PUM_STATE.width;
    let pum_scrollbar = PUM_STATE.scrollbar;
    let columns = Columns;
    let pum_above = PUM_STATE.above != 0;
    let pum_row = PUM_STATE.row;

    let mut col = pum_col + pum_width + 1 + border_width;
    if border_width < 0 {
        col += pum_scrollbar;
    }

    let right_extra = columns - col;
    let left_extra = pum_col - 2;

    let (cfg_width, cfg_col) = if right_extra > width {
        // Place to the right
        (width, col - 1)
    } else if left_extra > width {
        // Place to the left
        (width, pum_col - width - 1)
    } else {
        // Use whichever side has more space
        let place_right = right_extra > left_extra;
        if place_right {
            (right_extra, col - 1)
        } else {
            (left_extra, pum_col - left_extra - 1)
        }
    };

    let anchor = if pum_above { K_FLOAT_ANCHOR_SOUTH } else { 0 };
    let line_count = nvim_win_buf_line_count(wp);
    let rows = Rows;
    let height = plines_m_win(wp, 1, line_count, rows);
    let row = if pum_above { pum_row + height } else { pum_row };

    nvim_pum_win_config_set_and_apply(wp, cfg_width, cfg_col, anchor, height, row, 0);
}

/// Set info for a completed item, returning a window pointer.
///
/// Creates or reuses a floating preview window, sets the info text,
/// and positions it next to the popup menu.
///
/// Returns a window pointer, or null if not visible or no match.
///
/// # Safety
/// `info` must be a valid C string.
#[export_name = "pum_set_info"]
pub unsafe extern "C" fn rs_pum_set_info(selected: c_int, info: *mut c_char) -> *mut WinHandle {
    if PUM_STATE.is_visible == 0 || rs_compl_match_curr_select(selected) == 0 {
        return std::ptr::null_mut();
    }

    block_autocmds();
    RedrawingDisabled += 1;
    no_u_sync += 1;

    let mut wp = win_float_find_preview();
    if wp.is_null() {
        wp = win_float_create(false, true);
        if wp.is_null() {
            no_u_sync -= 1;
            RedrawingDisabled -= 1;
            unblock_autocmds();
            return std::ptr::null_mut();
        }
        nvim_win_set_topline(wp, 1);
        nvim_win_set_p_wfb(wp, 1);
    }

    let mut lnum: i32 = 0;
    let mut max_info_width: c_int = 0;
    let buf = nvim_win_get_buffer(wp);
    rs_pum_preview_set_text(
        buf,
        info,
        std::ptr::addr_of_mut!(lnum),
        std::ptr::addr_of_mut!(max_info_width),
    );

    no_u_sync -= 1;
    RedrawingDisabled -= 1;
    redraw_later(wp, UPD_NOT_VALID);

    rs_pum_adjust_info_position(wp, max_info_width);
    unblock_autocmds();
    wp
}
