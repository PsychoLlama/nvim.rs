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
#![allow(clippy::too_many_lines)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

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

    // Phase 2 accessors
    fn nvim_curwin_get_alt_fnum() -> c_int;
    fn nvim_handle_get_buffer(handle: c_int) -> BufHandle;
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn nvim_home_replace_save(buf: BufHandle, src: *const c_char) -> *mut c_char;
    fn nvim_FullName_save(fname: *const c_char, force: bool) -> *mut c_char;
    fn nvim_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_os_fileid(path: *const c_char, file_id_out: *mut u8) -> bool;
    #[link_name = "rs_otherfile_buf_4"]
    fn nvim_otherfile_buf(
        buf: BufHandle,
        ffname: *const c_char,
        file_id_p: *const u8,
        file_id_valid: bool,
    ) -> bool;
    fn xfree(ptr: *mut c_void);
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
// Buffer Lookup (Phase 2: Wave 2)
// =============================================================================

/// `BF_DUMMY` flag — dummy buffer used for preview, etc.
const BF_DUMMY: c_int = 0x80;

/// Maximum size for an opaque `FileID` buffer.
const FILE_ID_SIZE: usize = 16;

/// Find a file in the buffer list by buffer number.
///
/// If `nr` is 0, uses the alternate file number for the current window.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_findnr(nr: c_int) -> BufHandle {
    let nr = if nr == 0 {
        nvim_curwin_get_alt_fnum()
    } else {
        nr
    };
    nvim_handle_get_buffer(nr)
}

/// Get name of file 'n' in the buffer list.
///
/// Returns an allocated string (caller must free with `xfree`) or NULL.
/// Uses `home_replace_save()` to shorten the file name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_nr2name(
    n: c_int,
    fullname: c_int,
    helptail: c_int,
) -> *mut c_char {
    let buf = rs_buflist_findnr(n);
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    let src = if fullname != 0 {
        nvim_buf_get_b_ffname(buf)
    } else {
        nvim_buf_get_b_fname(buf)
    };
    let help_buf = if helptail != 0 {
        buf
    } else {
        BufHandle(std::ptr::null_mut())
    };
    nvim_home_replace_save(help_buf, src)
}

/// Internal: find buffer by name and `file_id`, iterating backwards.
///
/// Skips dummy buffers. Returns NULL if not found.
unsafe fn buflist_findname_file_id_impl(
    ffname: *const c_char,
    file_id: *const u8,
    file_id_valid: bool,
) -> BufHandle {
    let mut buf = nvim_get_lastbuf();
    while !buf.is_null() {
        let flags = nvim_buf_get_flags(buf);
        if (flags & BF_DUMMY) == 0 && !nvim_otherfile_buf(buf, ffname, file_id, file_id_valid) {
            return buf;
        }
        buf = nvim_buf_get_prev(buf);
    }
    BufHandle(std::ptr::null_mut())
}

/// Find file in buffer list by name (full path).
///
/// Gets the `file_id` and delegates to the internal `file_id` lookup.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_findname(ffname: *mut c_char) -> BufHandle {
    if ffname.is_null() {
        return BufHandle(std::ptr::null_mut());
    }
    let mut file_id = [0u8; FILE_ID_SIZE];
    let file_id_valid = nvim_os_fileid(ffname, file_id.as_mut_ptr());
    buflist_findname_file_id_impl(ffname, file_id.as_ptr(), file_id_valid)
}

/// Find file in buffer list by name, expanding to full path first.
///
/// On Unix, forces expansion to resolve symbolic links.
#[allow(clippy::similar_names)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_findname_exp(fname: *mut c_char) -> BufHandle {
    if fname.is_null() {
        return BufHandle(std::ptr::null_mut());
    }
    // On Unix, force expansion to resolve symbolic links
    let force = cfg!(unix);
    let ffname = nvim_FullName_save(fname, force);
    if ffname.is_null() {
        return BufHandle(std::ptr::null_mut());
    }
    let buf = rs_buflist_findname(ffname);
    xfree(ffname.cast());
    buf
}

// =============================================================================
// buflist_list helpers (Phase 4)
// =============================================================================

// Additional accessors for buflist_list (not already declared above).
extern "C" {
    fn nvim_get_curwin() -> crate::WinHandle;
    fn nvim_win_get_cursor_lnum(wp: crate::WinHandle) -> c_int;
    fn nvim_buf_get_b_p_bl(buf: BufHandle) -> c_int;
    fn nvim_msg_ext_set_kind(kind: *const c_char);
    fn nvim_eap_get_arg(eap: *const c_void) -> *const c_char;
    fn nvim_eap_get_forceit(eap: *const c_void) -> bool;
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_mfp_null(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ma(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_buf_get_terminal(buf: BufHandle) -> c_int;
    fn nvim_buf_terminal_running(buf: BufHandle) -> c_int;
    fn nvim_buf_channel_job_running(buf: BufHandle) -> c_int;
    fn nvim_buf_is_changed(buf: BufHandle) -> c_int;
    fn nvim_buf_get_last_used(buf: BufHandle) -> i64;
    fn rs_buf_spname(buf: BufHandle) -> *mut c_char;
    fn nvim_home_replace(
        buf: BufHandle,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: bool,
    ) -> usize;
    fn nvim_message_filtered(msg: *const c_char) -> c_int;
    #[link_name = "got_int"]
    static mut nvim_got_int: bool;
    fn msg_putchar(c: c_int);
    fn vim_strsize(s: *const c_char) -> c_int;
    fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn nvim_line_breakcheck();
    fn nvim_undo_fmt_time(buf: *mut c_char, buflen: usize, last_used: i64);
    static mut IObuff: [c_char; 1025];
    fn nvim_buflist_line_fmt() -> *const c_char;
}

// WinHandle is used via crate::WinHandle in extern "C" blocks above.

// Buffer flags for buflist (correct values from buffer_defs.h).
const BF_READERR_LIST: c_int = 0x40;

// IOSIZE constant
const IOSIZE_LIST: usize = 1025;
// MAXPATHL constant
const MAXPATHL: usize = 4096;

/// Print the buffer list (`:ls` / `:buffers` command implementation).
///
/// # Safety
///
/// Calls external C functions. `eap` must be a valid `exarg_T*`.
pub unsafe fn buflist_list_impl(eap: *const c_void) {
    let eap_arg = nvim_eap_get_arg(eap);
    let forceit = nvim_eap_get_forceit(eap);
    let curbuf = nvim_get_curbuf();
    let curwin = nvim_get_curwin();
    let alt_fnum = nvim_curwin_get_alt_fnum();

    // Check if 't' flag: sort by time
    let sort_by_time = !eap_arg.is_null() && {
        let arg = std::ffi::CStr::from_ptr(eap_arg);
        arg.to_bytes().contains(&b't')
    };

    nvim_msg_ext_set_kind(c"list_cmd".as_ptr().cast::<c_char>());

    // Collect buffers
    let mut bufs: Vec<BufHandle> = Vec::new();
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        bufs.push(buf);
        buf = nvim_buf_get_next(buf);
    }

    // Optionally sort by last used time
    if sort_by_time {
        bufs.sort_by(|a, b| {
            let ta = nvim_buf_get_last_used(*a);
            let tb = nvim_buf_get_last_used(*b);
            // Same ordering as buf_time_compare: sort ascending by time
            ta.cmp(&tb)
        });
    }

    // Iterate
    for buf in bufs {
        if nvim_got_int {
            break;
        }

        let is_terminal = nvim_buf_get_terminal(buf) != 0;
        let job_running = nvim_buf_terminal_running(buf) != 0;
        let bl = nvim_buf_get_b_p_bl(buf) != 0;
        let flags = nvim_buf_get_flags(buf);
        let ml_mfp_null = nvim_buf_get_ml_mfp_null(buf) != 0;
        let nwindows = nvim_buf_get_nwindows(buf);
        let is_changed = nvim_buf_is_changed(buf) != 0;
        let is_ro = nvim_buf_get_b_p_ro(buf) != 0;
        let is_ma = nvim_buf_get_b_p_ma(buf) != 0;
        let fnum = nvim_buf_get_fnum(buf);

        // Check filter flags
        let arg_bytes: &[u8] = if eap_arg.is_null() {
            b""
        } else {
            std::ffi::CStr::from_ptr(eap_arg).to_bytes()
        };

        let has_arg = |c: u8| arg_bytes.contains(&c);

        if (!bl && !forceit && !has_arg(b'u'))
            || (has_arg(b'u') && bl)
            || (has_arg(b'+') && ((flags & BF_READERR_LIST) != 0 || !is_changed))
            || (has_arg(b'a') && (ml_mfp_null || nwindows == 0))
            || (has_arg(b'h') && (ml_mfp_null || nwindows != 0))
            || (has_arg(b'R') && (!is_terminal || !job_running))
            || (has_arg(b'F') && (!is_terminal || job_running))
            || (has_arg(b'-') && is_ma)
            || (has_arg(b'=') && !is_ro)
            || (has_arg(b'x') && (flags & BF_READERR_LIST) == 0)
            || (has_arg(b'%') && buf != curbuf)
            || (has_arg(b'#') && (buf == curbuf || alt_fnum != fnum))
        {
            continue;
        }

        // Get buffer name into NameBuff equivalent
        let mut name_buf = [0u8; MAXPATHL];
        let spname = rs_buf_spname(buf);
        if spname.is_null() {
            let b_fname = nvim_buf_get_b_fname(buf);
            nvim_home_replace(
                buf,
                b_fname,
                name_buf.as_mut_ptr().cast::<c_char>(),
                MAXPATHL,
                true,
            );
        } else {
            let len = libc::strlen(spname).min(MAXPATHL - 1);
            std::ptr::copy_nonoverlapping(spname.cast::<u8>(), name_buf.as_mut_ptr(), len);
            name_buf[len] = 0;
        }

        if nvim_message_filtered(name_buf.as_ptr().cast::<c_char>()) != 0 {
            continue;
        }

        // Determine display characters
        let changed_char: u8 = if (flags & BF_READERR_LIST) != 0 {
            b'x'
        } else if is_changed {
            b'+'
        } else {
            b' '
        };
        let ro_char: u8 = if is_terminal {
            if nvim_buf_channel_job_running(buf) != 0 {
                b'R'
            } else {
                b'F'
            }
        } else if !is_ma {
            b'-'
        } else if is_ro {
            b'='
        } else {
            b' '
        };
        let load_char: u8 = if ml_mfp_null {
            b' '
        } else if nwindows == 0 {
            b'h'
        } else {
            b'a'
        };
        let cur_char: u8 = if buf == curbuf {
            b'%'
        } else if alt_fnum == fnum {
            b'#'
        } else {
            b' '
        };
        let bl_char: u8 = if bl { b' ' } else { b'u' };

        // Build the formatted line into IObuff
        let iobuff = std::ptr::addr_of_mut!(IObuff).cast::<c_char>();
        let name_cstr = name_buf.as_ptr().cast::<c_char>();

        // Format: "%3d%c%c%c%c%c \"%s\""
        let n = libc::snprintf(
            iobuff,
            IOSIZE_LIST - 20,
            c"%3d%c%c%c%c%c \"%s\"".as_ptr().cast::<c_char>(),
            fnum,
            c_int::from(bl_char),
            c_int::from(cur_char),
            c_int::from(load_char),
            c_int::from(ro_char),
            c_int::from(changed_char),
            name_cstr,
        );
        let n = if n > 0 {
            (n as usize).min(IOSIZE_LIST - 21)
        } else {
            0
        };

        // Pad to column 40
        let displayed_width = vim_strsize(iobuff);
        let mut pad = 40 - displayed_width;
        let mut len = n;
        while pad > 0 && len < IOSIZE_LIST - 19 {
            *iobuff.add(len) = b' ' as c_char;
            len += 1;
            pad -= 1;
        }

        // Append time or line number
        if has_arg(b't') && nvim_buf_get_last_used(buf) != 0 {
            nvim_undo_fmt_time(
                iobuff.add(len),
                IOSIZE_LIST - len,
                nvim_buf_get_last_used(buf),
            );
        } else {
            let lnum: i64 = if buf == curbuf {
                i64::from(nvim_win_get_cursor_lnum(curwin))
            } else {
                i64::from(nvim_buf_get_ml_line_count(buf))
            };
            let fmt = nvim_buflist_line_fmt();
            libc::snprintf(iobuff.add(len), IOSIZE_LIST - len, fmt, lnum);
        }

        msg_putchar(c_int::from(b'\n'));
        msg_outtrans(iobuff.cast::<c_char>(), 0, false);
        nvim_line_breakcheck();
    }
}

/// List the buffer list (`:ls` command).
///
/// # Safety
///
/// Must be called on the Neovim main thread with valid `eap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buflist_list(eap: *const c_void) {
    buflist_list_impl(eap);
}

/// C export: `buflist_list`.
#[unsafe(export_name = "buflist_list")]
pub unsafe extern "C" fn buflist_list_export(eap: *const c_void) {
    buflist_list_impl(eap);
}

// =============================================================================
// buflist_findpat (Phase 8)
// =============================================================================

// Additional accessors for buflist_findpat.
extern "C" {
    fn nvim_path_file_pat_to_reg_pat(
        pat: *const c_char,
        pat_end: *const c_char,
        no_bslash: *mut c_char,
        allow_dirs: c_int,
    ) -> *mut c_char;
    fn nvim_blfp_regex_compile(pat: *const c_char, magic: c_int) -> *mut c_void;
    fn nvim_blfp_regex_valid(handle: *mut c_void) -> c_int;
    fn nvim_blfp_regex_free(handle: *mut c_void);
    fn nvim_blfp_buflist_match(
        handle: *mut c_void,
        buf: BufHandle,
        ignore_case: bool,
    ) -> *const c_char;
    fn nvim_blfp_errmsg_e93(pattern: *const c_char);
    fn nvim_blfp_errmsg_e94(pattern: *const c_char);
    fn rs_magic_isset() -> c_int;
    fn rs_diff_mode_buf(buf: BufHandle) -> bool;
}

// RE_MAGIC constant (from regexp_defs.h)
const RE_MAGIC: c_int = 1;

/// Find file in buffer list by a regexp pattern.
///
/// # Parameters
/// - `pattern` / `pattern_end`: pointer range for the pattern (`pattern_end` is first char after)
/// - `unlisted`: also search unlisted buffers
/// - `diffmode`: only match diff-mode buffers
/// - `curtab_only`: only match buffers open in the current tab (always false in practice)
///
/// Returns the fnum of the found buffer, or -1 for no match / error.
/// Emits E93 if multiple matches, E94 if no match.
///
/// # Safety
///
/// Must be called on the Neovim main thread with valid state.
#[must_use]
#[allow(clippy::too_many_lines)]
pub unsafe fn buflist_findpat_impl(
    pattern: *const c_char,
    pattern_end: *const c_char,
    unlisted: bool,
    diffmode: bool,
    _curtab_only: bool,
) -> c_int {
    let pat_len = if pattern_end.is_null() {
        libc::strlen(pattern)
    } else {
        pattern_end.offset_from(pattern) as usize
    };

    // Special single-char patterns: '%' = curbuf, '#' = alt buffer
    if pat_len == 1 {
        let ch = *pattern as u8;
        if ch == b'%' || ch == b'#' {
            let found_fnum = if ch == b'%' {
                nvim_buf_get_fnum(nvim_get_curbuf())
            } else {
                nvim_curwin_get_alt_fnum()
            };
            if diffmode {
                let found_buf = rs_buflist_findnr(found_fnum);
                if found_buf.is_null() || !rs_diff_mode_buf(found_buf) {
                    // emit E94 with the original pattern char
                    nvim_blfp_errmsg_e94(pattern);
                    return -1;
                }
            }
            return found_fnum;
        }
    }

    // Convert glob pattern to regex via file_pat_to_reg_pat
    let pat_end_ptr = if pattern_end.is_null() {
        pattern.add(pat_len)
    } else {
        pattern_end
    };

    let pat_c = nvim_path_file_pat_to_reg_pat(pattern, pat_end_ptr, std::ptr::null_mut(), 0);
    if pat_c.is_null() {
        return -1;
    }

    // Work with a mutable Vec<u8> copy so we can toggle '$'
    let pat_len_c = libc::strlen(pat_c);
    let mut pat_bytes: Vec<u8> =
        std::slice::from_raw_parts(pat_c as *const u8, pat_len_c + 1).to_vec(); // includes NUL
    xfree(pat_c.cast::<c_void>());

    // toggledollar: if the last non-NUL char is '$', we toggle it off/on per attempt
    let pat_body_len = pat_bytes.len() - 1; // excluding NUL
    let toggledollar = pat_body_len > 1 && pat_bytes[pat_body_len - 1] == b'$';

    let mut match_fnum: c_int = -1;

    // Two passes: first find listed, then (if unlisted) unlisted buffers
    let mut find_listed = true;
    loop {
        // 4 attempts: vary '^' prefix and '$' suffix
        'attempt_loop: for attempt in 0..=3i32 {
            // Toggle '$' at end (attempt 0,1 => no '$', attempt 2,3 => with '$')
            if toggledollar {
                if attempt < 2 {
                    pat_bytes[pat_body_len - 1] = 0; // remove '$'
                } else {
                    pat_bytes[pat_body_len - 1] = b'$'; // restore '$'
                }
            }

            // Determine start of pattern: skip '^' on odd attempts
            let skip_caret = pat_bytes[0] == b'^' && (attempt & 1) == 0;
            let pat_start: *const c_char = if skip_caret {
                pat_bytes.as_ptr().add(1).cast::<c_char>()
            } else {
                pat_bytes.as_ptr().cast::<c_char>()
            };

            let magic = if rs_magic_isset() != 0 { RE_MAGIC } else { 0 };
            let regex_handle = nvim_blfp_regex_compile(pat_start, magic);

            // Walk buffers backwards
            let mut buf = nvim_get_lastbuf();
            while !buf.is_null() {
                if nvim_blfp_regex_valid(regex_handle) == 0 {
                    // Regex engine switched — abort
                    nvim_blfp_regex_free(regex_handle);
                    return -1;
                }

                let bl = nvim_buf_get_b_p_bl(buf) != 0;
                if bl == find_listed
                    && (!diffmode || rs_diff_mode_buf(buf))
                    && !nvim_blfp_buflist_match(regex_handle, buf, false).is_null()
                {
                    // curtab_only check omitted: all callers pass false
                    if match_fnum >= 0 {
                        // Multiple matches
                        match_fnum = -2;
                        buf = nvim_buf_get_prev(buf);
                        continue;
                    }
                    match_fnum = nvim_buf_get_fnum(buf);
                }

                buf = nvim_buf_get_prev(buf);
            }

            nvim_blfp_regex_free(regex_handle);

            if match_fnum >= 0 {
                break 'attempt_loop; // found exactly one match
            }
        }

        // Only search unlisted if no match yet
        if !unlisted || !find_listed || match_fnum != -1 {
            break;
        }
        find_listed = false;
        match_fnum = -1;
    }

    // Emit error messages
    if match_fnum == -2 {
        nvim_blfp_errmsg_e93(pattern);
    } else if match_fnum < 0 {
        nvim_blfp_errmsg_e94(pattern);
    }

    match_fnum
}

/// C export: `buflist_findpat`.
#[must_use]
#[unsafe(export_name = "buflist_findpat")]
pub unsafe extern "C" fn buflist_findpat_export(
    pattern: *const c_char,
    pattern_end: *const c_char,
    unlisted: bool,
    diffmode: bool,
    curtab_only: bool,
) -> c_int {
    buflist_findpat_impl(pattern, pattern_end, unlisted, diffmode, curtab_only)
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
