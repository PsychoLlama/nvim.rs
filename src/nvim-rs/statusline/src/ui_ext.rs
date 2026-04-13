//! External UI tabline update functionality
//!
//! This module provides Rust implementations for building tabline data
//! to send to external UI clients (like nvim-qt, neovide, etc.).

use std::ffi::{c_char, c_int};

use nvim_buffer::buf_struct::BufStruct;
use nvim_window::{BufHandle, TabpageHandle, WinHandle};

/// Get &BufStruct from nvim_window::BufHandle.
///
/// # Safety
/// buf must be a valid, non-null buf_T pointer.
#[inline]
unsafe fn bref(buf: BufHandle) -> &'static BufStruct {
    let raw: *mut std::ffi::c_void = std::mem::transmute(buf);
    &*(raw.cast::<BufStruct>())
}

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // Tabpage iteration
    fn nvim_get_first_tabpage() -> TabpageHandle;
    fn nvim_tabpage_get_next(tp: TabpageHandle) -> TabpageHandle;
    fn nvim_tabpage_get_handle(tp: TabpageHandle) -> c_int;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_tabpage_get_curwin(tp: TabpageHandle) -> WinHandle;

    // Buffer iteration
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;

    // Window accessors
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_get_curwin() -> WinHandle;

    // Buffer name
    fn get_trans_bufname(buf: BufHandle);

    // Global NameBuff access (MAXPATHL buffer for names)
    fn nvim_get_namebuff() -> *mut c_char;

    // Standard C library
    fn strlen(s: *const c_char) -> usize;
}

// =============================================================================
// Tab/Buffer Info Structures
// =============================================================================

/// Information about a single tab for the tabline update.
#[derive(Debug, Clone)]
pub struct TablineTabInfo {
    /// Tab handle (for API)
    pub handle: c_int,
    /// Display name (from current window's buffer)
    pub name: String,
}

/// Information about a single buffer for the tabline update.
#[derive(Debug, Clone)]
pub struct TablineBufferInfo {
    /// Buffer handle (for API)
    pub handle: c_int,
    /// Display name
    pub name: String,
}

/// Collected data for a tabline update event.
#[derive(Debug, Default)]
pub struct TablineUpdateData {
    /// Current tabpage handle
    pub current_tab: c_int,
    /// List of tabs
    pub tabs: Vec<TablineTabInfo>,
    /// Current buffer handle
    pub current_buffer: c_int,
    /// List of listed buffers
    pub buffers: Vec<TablineBufferInfo>,
}

// =============================================================================
// Data Collection Functions
// =============================================================================

/// Count the number of tabpages.
fn count_tabs() -> usize {
    let mut count = 0;
    unsafe {
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            count += 1;
            tp = nvim_tabpage_get_next(tp);
        }
    }
    count
}

/// Count the number of listed buffers.
fn count_listed_buffers() -> usize {
    let mut count = 0;
    unsafe {
        let mut buf = nvim_get_firstbuf();
        while !buf.is_null() {
            if bref(buf).b_p_bl != 0 {
                count += 1;
            }
            buf = BufHandle::from_ptr(bref(buf).b_next);
        }
    }
    count
}

/// Get the display name for a buffer.
///
/// # Safety
/// Relies on C's get_trans_bufname() and NameBuff global.
unsafe fn get_buffer_display_name(buf: BufHandle) -> String {
    get_trans_bufname(buf);
    let name_ptr = nvim_get_namebuff();
    if name_ptr.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(name_ptr)
        .to_string_lossy()
        .into_owned()
}

/// Collect all tabline update data from C structures.
///
/// This is the Rust equivalent of the data collection part of `ui_ext_tabline_update()`.
pub fn collect_tabline_data() -> TablineUpdateData {
    let mut data = TablineUpdateData::default();

    unsafe {
        // Get current handles
        let curtab = nvim_get_curtab();
        let curwin = nvim_get_curwin();
        let curbuf = nvim_get_curbuf();

        data.current_tab = nvim_tabpage_get_handle(curtab);
        data.current_buffer = bref(curbuf).handle;

        // Collect tabs
        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let handle = nvim_tabpage_get_handle(tp);

            // Get current window for this tab (curwin for curtab, tp_curwin for others)
            let cwp = if tp == curtab {
                curwin
            } else {
                nvim_tabpage_get_curwin(tp)
            };

            // Get buffer name from the tab's current window
            let buf = nvim_win_get_buffer(cwp);
            let name = get_buffer_display_name(buf);

            data.tabs.push(TablineTabInfo { handle, name });

            tp = nvim_tabpage_get_next(tp);
        }

        // Collect listed buffers
        let mut buf = nvim_get_firstbuf();
        while !buf.is_null() {
            // Only include listed buffers (b_p_bl == true)
            if bref(buf).b_p_bl != 0 {
                let handle = bref(buf).handle;
                let name = get_buffer_display_name(buf);
                data.buffers.push(TablineBufferInfo { handle, name });
            }
            buf = BufHandle::from_ptr(bref(buf).b_next);
        }
    }

    data
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Count tabpages.
#[no_mangle]
pub extern "C" fn rs_ui_ext_count_tabs() -> usize {
    count_tabs()
}

/// FFI export: Count listed buffers.
#[no_mangle]
pub extern "C" fn rs_ui_ext_count_listed_buffers() -> usize {
    count_listed_buffers()
}

/// FFI export: Get current tabpage handle.
#[no_mangle]
pub extern "C" fn rs_ui_ext_current_tab_handle() -> c_int {
    unsafe {
        let curtab = nvim_get_curtab();
        nvim_tabpage_get_handle(curtab)
    }
}

/// FFI export: Get current buffer handle.
#[no_mangle]
pub extern "C" fn rs_ui_ext_current_buf_handle() -> c_int {
    unsafe {
        let curbuf = nvim_get_curbuf();
        bref(curbuf).handle
    }
}

// =============================================================================
// Tab/Buffer Iterator State for C
// =============================================================================

/// Opaque iterator state for iterating over tabs.
pub struct TabIterator {
    current: TabpageHandle,
}

/// Opaque iterator state for iterating over buffers.
pub struct BufferIterator {
    current: BufHandle,
}

/// FFI export: Create a new tab iterator.
#[no_mangle]
pub extern "C" fn rs_tab_iter_new() -> *mut TabIterator {
    let iter = Box::new(TabIterator {
        current: unsafe { nvim_get_first_tabpage() },
    });
    Box::into_raw(iter)
}

/// FFI export: Get next tab from iterator.
///
/// Returns the tabpage handle, or 0 if exhausted.
/// Also sets *handle to the API handle and copies the name to name_buf.
///
/// # Safety
/// - `iter` must be a valid TabIterator pointer from rs_tab_iter_new.
/// - `handle` must be a valid pointer.
/// - `name_buf` must be a valid buffer of `name_buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_tab_iter_next(
    iter: *mut TabIterator,
    handle: *mut c_int,
    name_buf: *mut c_char,
    name_buf_len: usize,
) -> c_int {
    if iter.is_null() {
        return 0;
    }

    let it = &mut *iter;
    if it.current.is_null() {
        return 0;
    }

    let tp = it.current;
    let tp_handle = nvim_tabpage_get_handle(tp);

    // Get the current window for this tab
    let curtab = nvim_get_curtab();
    let cwp = if tp == curtab {
        nvim_get_curwin()
    } else {
        nvim_tabpage_get_curwin(tp)
    };

    // Get buffer name
    let buf = nvim_win_get_buffer(cwp);
    get_trans_bufname(buf);
    let name_ptr = nvim_get_namebuff();

    // Copy handle
    if !handle.is_null() {
        *handle = tp_handle;
    }

    // Copy name
    if !name_buf.is_null() && name_buf_len > 0 && !name_ptr.is_null() {
        let name_len = strlen(name_ptr);
        let copy_len = name_len.min(name_buf_len - 1);
        std::ptr::copy_nonoverlapping(name_ptr, name_buf, copy_len);
        *name_buf.add(copy_len) = 0;
    }

    // Advance iterator
    it.current = nvim_tabpage_get_next(tp);

    1 // Success
}

/// FFI export: Free a tab iterator.
///
/// # Safety
/// `iter` must be a valid TabIterator pointer from rs_tab_iter_new, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_tab_iter_free(iter: *mut TabIterator) {
    if !iter.is_null() {
        drop(Box::from_raw(iter));
    }
}

/// FFI export: Create a new buffer iterator (for listed buffers only).
#[no_mangle]
pub extern "C" fn rs_buf_iter_new() -> *mut BufferIterator {
    let iter = Box::new(BufferIterator {
        current: unsafe { nvim_get_firstbuf() },
    });
    Box::into_raw(iter)
}

/// FFI export: Get next listed buffer from iterator.
///
/// Returns 1 on success, 0 if exhausted.
/// Also sets *handle to the API handle and copies the name to name_buf.
///
/// # Safety
/// - `iter` must be a valid BufferIterator pointer from rs_buf_iter_new.
/// - `handle` must be a valid pointer.
/// - `name_buf` must be a valid buffer of `name_buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_iter_next(
    iter: *mut BufferIterator,
    handle: *mut c_int,
    name_buf: *mut c_char,
    name_buf_len: usize,
) -> c_int {
    if iter.is_null() {
        return 0;
    }

    let it = &mut *iter;

    // Find next listed buffer
    while !it.current.is_null() {
        let buf = it.current;
        it.current = BufHandle::from_ptr(bref(buf).b_next);

        // Check if buffer is listed
        if bref(buf).b_p_bl == 0 {
            continue;
        }

        let buf_handle = bref(buf).handle;

        // Get buffer name
        get_trans_bufname(buf);
        let name_ptr = nvim_get_namebuff();

        // Copy handle
        if !handle.is_null() {
            *handle = buf_handle;
        }

        // Copy name
        if !name_buf.is_null() && name_buf_len > 0 && !name_ptr.is_null() {
            let name_len = strlen(name_ptr);
            let copy_len = name_len.min(name_buf_len - 1);
            std::ptr::copy_nonoverlapping(name_ptr, name_buf, copy_len);
            *name_buf.add(copy_len) = 0;
        }

        return 1; // Success
    }

    0 // Exhausted
}

/// FFI export: Free a buffer iterator.
///
/// # Safety
/// `iter` must be a valid BufferIterator pointer from rs_buf_iter_new, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_iter_free(iter: *mut BufferIterator) {
    if !iter.is_null() {
        drop(Box::from_raw(iter));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabline_update_data_default() {
        let data = TablineUpdateData::default();
        assert_eq!(data.current_tab, 0);
        assert_eq!(data.current_buffer, 0);
        assert!(data.tabs.is_empty());
        assert!(data.buffers.is_empty());
    }

    #[test]
    fn test_tabline_tab_info() {
        let info = TablineTabInfo {
            handle: 1,
            name: "test.rs".to_string(),
        };
        assert_eq!(info.handle, 1);
        assert_eq!(info.name, "test.rs");
    }

    #[test]
    fn test_tabline_buffer_info() {
        let info = TablineBufferInfo {
            handle: 5,
            name: "[No Name]".to_string(),
        };
        assert_eq!(info.handle, 5);
        assert_eq!(info.name, "[No Name]");
    }
}
