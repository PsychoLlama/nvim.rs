//! Buffer list traversal and iteration helpers
//!
//! This module provides helpers for traversing and iterating through
//! the buffer list, finding buffers by various criteria, and managing
//! buffer navigation.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(dead_code)]

use std::ffi::c_int;

use crate::BufHandle;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_get_lastbuf() -> BufHandle;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_buf_get_prev(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_next(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_changedtick(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> c_int;
}

// =============================================================================
// Buffer List Constants
// =============================================================================

/// Direction for buffer navigation
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Move to next buffer
    #[default]
    Forward = 1,
    /// Move to previous buffer
    Backward = -1,
}

impl Direction {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        if value >= 0 {
            Self::Forward
        } else {
            Self::Backward
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Get the opposite direction.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

// =============================================================================
// Buffer Navigation
// =============================================================================

/// Get the first buffer in the list.
///
/// # Safety
///
/// Calls external C function.
#[must_use]
pub unsafe fn get_first_buf() -> BufHandle {
    nvim_get_firstbuf()
}

/// Get the last buffer in the list.
///
/// # Safety
///
/// Calls external C function.
#[must_use]
pub unsafe fn get_last_buf() -> BufHandle {
    nvim_get_lastbuf()
}

/// Get the current buffer.
///
/// # Safety
///
/// Calls external C function.
#[must_use]
pub unsafe fn get_cur_buf() -> BufHandle {
    nvim_get_curbuf()
}

/// Get the next buffer from the given buffer.
///
/// # Safety
///
/// Calls external C function. `buf` must be valid.
#[must_use]
pub unsafe fn get_next_buf(buf: BufHandle) -> BufHandle {
    if buf.is_null() {
        return BufHandle::from_ptr(std::ptr::null_mut());
    }
    nvim_buf_get_next(buf)
}

/// Get the previous buffer from the given buffer.
///
/// # Safety
///
/// Calls external C function. `buf` must be valid.
#[must_use]
pub unsafe fn get_prev_buf(buf: BufHandle) -> BufHandle {
    if buf.is_null() {
        return BufHandle::from_ptr(std::ptr::null_mut());
    }
    nvim_buf_get_prev(buf)
}

/// Navigate to the next/previous buffer in the given direction.
///
/// # Safety
///
/// Calls external C functions. `buf` must be valid.
#[must_use]
pub unsafe fn navigate_buf(buf: BufHandle, dir: Direction) -> BufHandle {
    match dir {
        Direction::Forward => get_next_buf(buf),
        Direction::Backward => get_prev_buf(buf),
    }
}

// =============================================================================
// Buffer Counting
// =============================================================================

/// Count the number of buffers in the list.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn count_buffers() -> usize {
    let mut count = 0usize;
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        count += 1;
        buf = nvim_buf_get_next(buf);
    }
    count
}

/// Count buffers matching a predicate.
///
/// # Safety
///
/// Calls external C functions. The predicate must not cause UB.
pub unsafe fn count_buffers_where<F>(predicate: F) -> usize
where
    F: Fn(BufHandle) -> bool,
{
    let mut count = 0usize;
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        if predicate(buf) {
            count += 1;
        }
        buf = nvim_buf_get_next(buf);
    }
    count
}

// =============================================================================
// Buffer Finding
// =============================================================================

/// Find a buffer by its file number (fnum).
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn find_buf_by_fnum(fnum: c_int) -> BufHandle {
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        if nvim_buf_get_fnum(buf) == fnum {
            return buf;
        }
        buf = nvim_buf_get_next(buf);
    }
    BufHandle::from_ptr(std::ptr::null_mut())
}

/// Find the first buffer matching a predicate.
///
/// # Safety
///
/// Calls external C functions. The predicate must not cause UB.
pub unsafe fn find_buf_where<F>(predicate: F) -> BufHandle
where
    F: Fn(BufHandle) -> bool,
{
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        if predicate(buf) {
            return buf;
        }
        buf = nvim_buf_get_next(buf);
    }
    BufHandle::from_ptr(std::ptr::null_mut())
}

/// Find the buffer at a given index (0-based).
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn find_buf_at_index(index: usize) -> BufHandle {
    let mut buf = nvim_get_firstbuf();
    let mut i = 0usize;
    while !buf.is_null() {
        if i == index {
            return buf;
        }
        buf = nvim_buf_get_next(buf);
        i += 1;
    }
    BufHandle::from_ptr(std::ptr::null_mut())
}

/// Get the index of a buffer in the list (0-based).
///
/// Returns -1 if the buffer is not found.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn get_buf_index(target: BufHandle) -> c_int {
    if target.is_null() {
        return -1;
    }
    let mut buf = nvim_get_firstbuf();
    let mut i = 0;
    while !buf.is_null() {
        if buf == target {
            return i;
        }
        buf = nvim_buf_get_next(buf);
        i += 1;
    }
    -1
}

// =============================================================================
// Buffer State Information
// =============================================================================

/// Information about a buffer's position in the list.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufListInfo {
    /// Index in the buffer list (0-based)
    pub index: c_int,
    /// Total number of buffers
    pub total: c_int,
    /// Whether this is the first buffer
    pub is_first: bool,
    /// Whether this is the last buffer
    pub is_last: bool,
    /// Whether this is the current buffer
    pub is_current: bool,
}

/// Get information about a buffer's position in the list.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn get_buf_list_info(target: BufHandle) -> BufListInfo {
    if target.is_null() {
        return BufListInfo::default();
    }

    let first = nvim_get_firstbuf();
    let last = nvim_get_lastbuf();
    let cur = nvim_get_curbuf();

    let mut info = BufListInfo {
        index: -1,
        total: 0,
        is_first: target == first,
        is_last: target == last,
        is_current: target == cur,
    };

    let mut buf = first;
    let mut i = 0;
    while !buf.is_null() {
        if buf == target {
            info.index = i;
        }
        buf = nvim_buf_get_next(buf);
        i += 1;
    }
    info.total = i;

    info
}

// =============================================================================
// Buffer Navigation with Wrap
// =============================================================================

/// Navigate to the next/previous buffer with wrap-around.
///
/// If at the end/beginning, wraps to the beginning/end.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn navigate_buf_wrap(buf: BufHandle, dir: Direction) -> BufHandle {
    if buf.is_null() {
        return nvim_get_firstbuf();
    }

    let next = navigate_buf(buf, dir);
    if !next.is_null() {
        return next;
    }

    // Wrap around
    match dir {
        Direction::Forward => nvim_get_firstbuf(),
        Direction::Backward => nvim_get_lastbuf(),
    }
}

/// Navigate N buffers in a direction with wrap-around.
///
/// # Safety
///
/// Calls external C functions.
#[must_use]
pub unsafe fn navigate_buf_n(buf: BufHandle, dir: Direction, count: usize) -> BufHandle {
    let mut current = buf;
    for _ in 0..count {
        let next = navigate_buf_wrap(current, dir);
        if next == buf && count > 1 {
            // We've cycled back, stop
            break;
        }
        current = next;
    }
    current
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get direction from raw integer.
#[unsafe(no_mangle)]
pub extern "C" fn rs_buf_dir_from_raw(value: c_int) -> c_int {
    Direction::from_raw(value).to_raw()
}

/// Get opposite direction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_buf_dir_opposite(dir: c_int) -> c_int {
    Direction::from_raw(dir).opposite().to_raw()
}

/// Count all buffers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_count_all() -> c_int {
    count_buffers() as c_int
}

/// Find buffer by fnum.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_find_by_fnum(fnum: c_int) -> BufHandle {
    find_buf_by_fnum(fnum)
}

/// Find buffer at index.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_at_index(index: c_int) -> BufHandle {
    if index < 0 {
        return BufHandle::from_ptr(std::ptr::null_mut());
    }
    find_buf_at_index(index as usize)
}

/// Get buffer's index in list.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_get_index(buf: BufHandle) -> c_int {
    get_buf_index(buf)
}

/// Navigate buffer in direction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_navigate(buf: BufHandle, dir: c_int) -> BufHandle {
    navigate_buf(buf, Direction::from_raw(dir))
}

/// Navigate buffer with wrap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_navigate_wrap(buf: BufHandle, dir: c_int) -> BufHandle {
    navigate_buf_wrap(buf, Direction::from_raw(dir))
}

/// Navigate N buffers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_navigate_n(buf: BufHandle, dir: c_int, count: c_int) -> BufHandle {
    if count <= 0 {
        return buf;
    }
    navigate_buf_n(buf, Direction::from_raw(dir), count as usize)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction() {
        assert_eq!(Direction::Forward.to_raw(), 1);
        assert_eq!(Direction::Backward.to_raw(), -1);

        assert_eq!(Direction::from_raw(1), Direction::Forward);
        assert_eq!(Direction::from_raw(5), Direction::Forward);
        assert_eq!(Direction::from_raw(-1), Direction::Backward);
        assert_eq!(Direction::from_raw(-5), Direction::Backward);

        assert_eq!(Direction::Forward.opposite(), Direction::Backward);
        assert_eq!(Direction::Backward.opposite(), Direction::Forward);
    }

    #[test]
    fn test_buf_handle_null() {
        let null_handle = unsafe { BufHandle::from_ptr(std::ptr::null_mut()) };
        assert!(null_handle.is_null());
    }
}
