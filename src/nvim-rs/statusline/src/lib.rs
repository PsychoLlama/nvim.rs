//! Status line and tab line helper functions for Neovim
//!
//! This crate provides Rust implementations of status line functions
//! from `src/nvim/statusline.c` and related column formatting utilities.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int, c_void, CStr};
use std::io::Write;

use nvim_window::{BufHandle, Frame, WinHandle, FR_COL};

pub mod format;

pub use format::{FormatParser, FormatSpec, StlFlag, StlFormatContext, StlItem, StlItemType};

/// schar_T is stored as a u32 in Rust.
type ScharT = u32;

// =============================================================================
// Data Structures for Status Line Click Handling
// =============================================================================

/// Status line click type enumeration
///
/// Matches the C enum in statusline_defs.h
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlClickType {
    /// Clicks to this area are ignored
    Disabled = 0,
    /// Switch to the given tab
    TabSwitch = 1,
    /// Close given tab
    TabClose = 2,
    /// Run user function
    FuncRun = 3,
}

/// Status line click definition
///
/// Matches the C struct StlClickDefinition in statusline_defs.h
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StlClickDefinition {
    /// Type of the click
    pub click_type: StlClickType,
    /// Tab page number
    pub tabnr: c_int,
    /// Function to run (C string pointer, may be null)
    pub func: *mut c_char,
}

impl StlClickDefinition {
    /// Create a new disabled click definition
    pub const fn disabled() -> Self {
        Self {
            click_type: StlClickType::Disabled,
            tabnr: 0,
            func: std::ptr::null_mut(),
        }
    }

    /// Check if this click definition is disabled
    pub const fn is_disabled(&self) -> bool {
        matches!(self.click_type, StlClickType::Disabled)
    }
}

/// Status line click record (used for tabline clicks)
///
/// Matches the C struct StlClickRecord in statusline_defs.h
#[repr(C)]
pub struct StlClickRecord {
    /// Click definition
    pub def: StlClickDefinition,
    /// Location where region starts (C string pointer)
    pub start: *const c_char,
}

// =============================================================================
// Tabpage Handle
// =============================================================================

/// Opaque handle to C's tabpage_T
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TabpageHandle(*mut c_void);

impl TabpageHandle {
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

/// Highlight group for StatusLine (current window).
pub const HLF_S: c_int = 27;

/// Highlight group for StatusLineNC (non-current windows).
pub const HLF_SNC: c_int = 28;

/// Line number type (linenr_T).
type LinenrT = i32;

// C accessor functions
extern "C" {
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;
    fn nvim_win_get_fcs_stl(wp: WinHandle) -> ScharT;
    fn nvim_win_get_fcs_stlnc(wp: WinHandle) -> ScharT;
    fn nvim_win_get_topline(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_botline(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_buf_line_count(wp: WinHandle) -> LinenrT;
    fn nvim_win_get_fill(wp: WinHandle, lnum: LinenrT) -> c_int;
    fn nvim_win_get_arg_idx(wp: WinHandle) -> c_int;
    fn nvim_win_get_arg_idx_invalid(wp: WinHandle) -> c_int;
    fn nvim_win_argcount(wp: WinHandle) -> c_int;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> LinenrT;

    // Buffer accessors for statusline rendering
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ft(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ma(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
}

/// Check if the status line of window "wp" is connected to the status
/// line of the window right of it. If not, then it's a vertical separator.
///
/// Only call if `wp->w_vsep_width != 0`.
///
/// This is the Rust equivalent of `stl_connected()` in statusline.c.
fn stl_connected_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let mut fr = nvim_win_get_frame(wp);
        if fr.is_null() {
            return false;
        }

        // Walk up the frame tree
        while !(*fr).fr_parent.is_null() {
            let parent = (*fr).fr_parent;
            if (*parent).fr_layout == FR_COL {
                // In a column layout - check if there's a frame below
                if !(*fr).fr_next.is_null() {
                    break;
                }
            } else {
                // In a row layout - check if there's a frame to the right
                if !(*fr).fr_next.is_null() {
                    return true;
                }
            }
            fr = parent;
        }
        false
    }
}

/// Get the fill character and highlight group for a status line.
///
/// Returns the fill character (schar_T) and sets `*group` to:
/// - `HLF_S` (StatusLine) if wp is the current window
/// - `HLF_SNC` (StatusLineNC) if wp is not the current window
///
/// This is the Rust equivalent of `fillchar_status()` in statusline.c.
fn fillchar_status_impl(wp: WinHandle) -> (ScharT, c_int) {
    unsafe {
        if nvim_win_is_curwin(wp) != 0 {
            (nvim_win_get_fcs_stl(wp), HLF_S)
        } else {
            (nvim_win_get_fcs_stlnc(wp), HLF_SNC)
        }
    }
}

/// Format a column number for display.
///
/// If `col == vcol`, returns "col" as a string.
/// If `col != vcol`, returns "col-vcol" as a string.
///
/// Returns the number of bytes written (not including NUL terminator).
///
/// This is the Rust equivalent of `col_print()` in buffer.c.
fn col_print_impl(buf: &mut [u8], col: c_int, vcol: c_int) -> c_int {
    if buf.is_empty() {
        return 0;
    }

    let mut cursor = std::io::Cursor::new(buf);
    let result = if col == vcol {
        write!(cursor, "{col}")
    } else {
        write!(cursor, "{col}-{vcol}")
    };

    match result {
        #[allow(clippy::cast_possible_truncation)]
        Ok(()) => cursor.position() as c_int,
        Err(_) => 0,
    }
}

// FFI exports

/// Check if status line is connected to the window on the right.
///
/// # Safety
/// `wp` must be a valid window handle or null.
#[no_mangle]
pub extern "C" fn rs_stl_connected(wp: WinHandle) -> c_int {
    c_int::from(stl_connected_impl(wp))
}

/// Get the fill character for a status line.
///
/// # Safety
/// `wp` must be a valid window handle.
/// `group` must be a valid pointer to an hlf_T value.
#[no_mangle]
pub unsafe extern "C" fn rs_fillchar_status(group: *mut c_int, wp: WinHandle) -> ScharT {
    let (fillchar, grp) = fillchar_status_impl(wp);
    if !group.is_null() {
        *group = grp;
    }
    fillchar
}

/// Format a column number for display.
///
/// # Safety
/// `buf` must be a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_col_print(
    buf: *mut u8,
    buflen: usize,
    col: c_int,
    vcol: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    col_print_impl(slice, col, vcol)
}

/// Calculate the width for each tab in the tabline.
///
/// This computes an equal width for all tabs, ensuring minimum width of 6
/// characters per tab, and distributing remaining space evenly.
///
/// @param columns  Total available columns
/// @param tabcount Number of tabs to display
/// @return Width for each tab cell
#[inline]
fn tabwidth_calc_impl(columns: c_int, tabcount: c_int) -> c_int {
    if tabcount <= 0 {
        return 0;
    }
    // Formula: (Columns - 1 + tabcount / 2) / tabcount, minimum 6
    // The (tabcount / 2) part rounds to nearest rather than truncating
    let width = (columns - 1 + tabcount / 2) / tabcount;
    width.max(6)
}

/// FFI export: Calculate tab width for tabline.
#[no_mangle]
pub extern "C" fn rs_tabwidth_calc(columns: c_int, tabcount: c_int) -> c_int {
    tabwidth_calc_impl(columns, tabcount)
}

// =============================================================================
// Relative Position String Functions
// =============================================================================

/// Relative position type for the statusline %P item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativePosition {
    /// All buffer lines are visible
    All,
    /// At the top of the buffer
    Top,
    /// At the bottom of the buffer
    Bot,
    /// A percentage value (0-100)
    Percentage(u8),
}

/// Calculate the relative position in the window.
///
/// This determines whether we're at "Top", "Bot", "All", or a percentage.
fn get_rel_pos_impl(wp: WinHandle) -> RelativePosition {
    if wp.is_null() {
        return RelativePosition::Top;
    }

    unsafe {
        let topline = nvim_win_get_topline(wp);
        let botline = nvim_win_get_botline(wp);
        let line_count = nvim_win_buf_line_count(wp);
        let topfill = nvim_win_get_topfill(wp);

        // Calculate lines above the window
        let mut above = topline - 1;
        let fill_at_top = nvim_win_get_fill(wp, topline);
        above += fill_at_top - topfill;

        // Special case: if topline is 1 and we have filler lines visible,
        // consider that as seeing all lines
        if topline == 1 && topfill >= 1 {
            above = 0;
        }

        // Calculate lines below the window
        let below = line_count - botline + 1;

        if below <= 0 {
            // At the bottom or showing all
            if above == 0 {
                RelativePosition::All
            } else {
                RelativePosition::Bot
            }
        } else if above <= 0 {
            RelativePosition::Top
        } else {
            // Calculate percentage
            let total = above + below;
            #[allow(clippy::cast_sign_loss)]
            let perc = if total > 0 {
                // Use the same formula as calc_percentage
                let above_i64 = i64::from(above);
                let total_i64 = i64::from(total);
                let result = ((above_i64 * 100) + (total_i64 / 2)) / total_i64;
                result.clamp(0, 100) as u8
            } else {
                0
            };
            RelativePosition::Percentage(perc)
        }
    }
}

/// Get relative cursor position as a string.
///
/// Writes one of "All", "Top", "Bot", or a percentage like " 50" into the buffer.
/// Returns the number of bytes written.
fn stl_get_rel_pos_impl(buf: &mut [u8], wp: WinHandle) -> c_int {
    if buf.len() < 3 {
        return 0;
    }

    let pos = get_rel_pos_impl(wp);
    let mut cursor = std::io::Cursor::new(buf);

    let result = match pos {
        RelativePosition::All => write!(cursor, "All"),
        RelativePosition::Top => write!(cursor, "Top"),
        RelativePosition::Bot => write!(cursor, "Bot"),
        RelativePosition::Percentage(p) => write!(cursor, "{p:>3}"),
    };

    match result {
        #[allow(clippy::cast_possible_truncation)]
        Ok(()) => cursor.position() as c_int,
        Err(_) => 0,
    }
}

/// Get relative cursor position in window into buffer.
///
/// Returns one of "Top", "Bot", "All", or a percentage string like " 50".
///
/// # Safety
/// `buf` must be a valid pointer to a buffer of at least `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_get_rel_pos(buf: *mut u8, buflen: c_int, wp: WinHandle) -> c_int {
    if buf.is_null() || buflen < 3 {
        return 0;
    }
    #[allow(clippy::cast_sign_loss)]
    let slice = std::slice::from_raw_parts_mut(buf, buflen as usize);
    stl_get_rel_pos_impl(slice, wp)
}

// =============================================================================
// Argument List Formatting
// =============================================================================

/// Append argument number to a buffer.
///
/// If editing more than one file, appends " (2 of 8)" or " ((2) of 8)" if invalid.
/// Returns the number of characters appended.
fn stl_append_arg_number_impl(buf: &mut [u8], wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let argcount = nvim_win_argcount(wp);

        // Nothing to do if only one file
        if argcount <= 1 {
            return 0;
        }

        let arg_idx = nvim_win_get_arg_idx(wp);
        let arg_idx_invalid = nvim_win_get_arg_idx_invalid(wp) != 0;

        let mut cursor = std::io::Cursor::new(buf);
        let result = if arg_idx_invalid {
            write!(cursor, " (({}) of {})", arg_idx + 1, argcount)
        } else {
            write!(cursor, " ({} of {})", arg_idx + 1, argcount)
        };

        match result {
            #[allow(clippy::cast_possible_truncation)]
            Ok(()) => cursor.position() as c_int,
            Err(_) => 0,
        }
    }
}

/// Append argument list position to a buffer.
///
/// If editing more than one file, appends " (2 of 8)" or " ((2) of 8)" if invalid.
///
/// # Safety
/// `buf` must be a valid pointer to a buffer of at least `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_append_arg_number(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_append_arg_number_impl(slice, wp)
}

// =============================================================================
// Separator Fill Functions
// =============================================================================

/// Group item structure for truncation handling.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StlGroupItem {
    /// Start position in output buffer
    pub start_col: c_int,
    /// Minimum width (-1 means variable)
    pub minwid: c_int,
    /// Maximum width (0 means unlimited)
    pub maxwid: c_int,
}

/// Fill the space between two separators with a fill character.
///
/// This is used to evenly distribute space when there are multiple
/// separator markers (%) in a statusline.
fn stl_fill_between_impl(
    buf: &mut [u8],
    sep_start: usize,
    sep_end: usize,
    total_width: usize,
    content_width: usize,
    fill_char: u8,
) -> usize {
    if sep_start >= sep_end || sep_end > buf.len() {
        return 0;
    }

    // Calculate how much space we have to fill
    let available = total_width.saturating_sub(content_width);
    let fill_space = sep_end - sep_start;
    let to_fill = available.min(fill_space);

    // Fill with the fill character
    for byte in buf.iter_mut().skip(sep_start).take(to_fill) {
        *byte = fill_char;
    }

    to_fill
}

/// Calculate width of content after applying truncation.
///
/// Returns the width after truncation, inserting '<' marker if needed.
const fn stl_group_truncate_impl(
    content_width: c_int,
    max_width: c_int,
    has_truncate_marker: bool,
) -> (c_int, bool) {
    if max_width <= 0 || content_width <= max_width {
        (content_width, false)
    } else if has_truncate_marker {
        // Already has marker, just truncate
        (max_width, true)
    } else {
        // Need to add '<' marker, which takes 1 column
        (max_width, true)
    }
}

/// FFI export: Calculate truncated width with marker.
///
/// Returns the truncated width. Sets `needs_marker` to 1 if a '<' marker is needed.
///
/// # Safety
/// `needs_marker` must be null or a valid pointer to a c_int.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_group_truncate(
    content_width: c_int,
    max_width: c_int,
    has_marker: c_int,
    needs_marker: *mut c_int,
) -> c_int {
    let (width, marker) = stl_group_truncate_impl(content_width, max_width, has_marker != 0);
    if !needs_marker.is_null() {
        *needs_marker = c_int::from(marker);
    }
    width
}

/// FFI export: Fill between separators with fill character.
///
/// Fills the buffer from `sep_start` to `sep_end` with `fill_char`.
/// Returns the number of bytes actually filled.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_fill_between(
    buf: *mut u8,
    buflen: usize,
    sep_start: usize,
    sep_end: usize,
    total_width: usize,
    content_width: usize,
    fill_char: u8,
) -> usize {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_fill_between_impl(
        slice,
        sep_start,
        sep_end,
        total_width,
        content_width,
        fill_char,
    )
}

// =============================================================================
// Statusline Item Renderers
// =============================================================================

/// Filename item types for rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlFilenameType {
    /// Short filename (%f, %t)
    Short = 0,
    /// Full path filename (%F)
    FullPath = 1,
    /// Tail only (just the filename without path)
    Tail = 2,
}

/// Render the filename for statusline.
///
/// Returns the length of the rendered string (not including NUL).
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn stl_render_filename_impl(buf: &mut [u8], wp: WinHandle, ftype: StlFilenameType) -> c_int {
    if buf.is_empty() || wp.is_null() {
        return 0;
    }

    unsafe {
        let buf_handle = nvim_win_get_buffer(wp);
        if buf_handle.is_null() {
            return 0;
        }

        let fname_ptr = match ftype {
            StlFilenameType::FullPath => nvim_buf_get_b_ffname(buf_handle),
            StlFilenameType::Short | StlFilenameType::Tail => nvim_buf_get_b_fname(buf_handle),
        };

        if fname_ptr.is_null() {
            // No name, return "[No Name]"
            let no_name = b"[No Name]";
            let len = no_name.len().min(buf.len());
            buf[..len].copy_from_slice(&no_name[..len]);
            return len as c_int;
        }

        let Ok(fname) = CStr::from_ptr(fname_ptr).to_str() else {
            return 0;
        };

        // For tail type, get just the filename
        let output = if ftype == StlFilenameType::Tail {
            fname.rsplit('/').next().unwrap_or(fname)
        } else {
            fname
        };

        let len = output.len().min(buf.len());
        buf[..len].copy_from_slice(output.as_bytes());
        len as c_int
    }
}

/// Render the modified flag for statusline.
///
/// Returns "[+]" if modified, "[-]" if not modifiable, or empty string.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn stl_render_modified_impl(buf: &mut [u8], wp: WinHandle, short: bool) -> c_int {
    if buf.len() < 3 || wp.is_null() {
        return 0;
    }

    unsafe {
        let buf_handle = nvim_win_get_buffer(wp);
        if buf_handle.is_null() {
            return 0;
        }

        let changed = nvim_buf_get_b_changed(buf_handle);
        let modifiable = nvim_buf_get_b_p_ma(buf_handle) != 0;

        if changed {
            let marker: &[u8] = if short { b"+" } else { b"[+]" };
            let len = marker.len().min(buf.len());
            buf[..len].copy_from_slice(&marker[..len]);
            len as c_int
        } else if !modifiable {
            let marker: &[u8] = if short { b"-" } else { b"[-]" };
            let len = marker.len().min(buf.len());
            buf[..len].copy_from_slice(&marker[..len]);
            len as c_int
        } else {
            0
        }
    }
}

/// Render the readonly flag for statusline.
///
/// Returns "[RO]" if readonly, empty string otherwise.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn stl_render_readonly_impl(buf: &mut [u8], wp: WinHandle, short: bool) -> c_int {
    if buf.len() < 4 || wp.is_null() {
        return 0;
    }

    unsafe {
        let buf_handle = nvim_win_get_buffer(wp);
        if buf_handle.is_null() {
            return 0;
        }

        let readonly = nvim_buf_get_b_p_ro(buf_handle) != 0;

        if readonly {
            let marker: &[u8] = if short { b"RO" } else { b"[RO]" };
            let len = marker.len().min(buf.len());
            buf[..len].copy_from_slice(&marker[..len]);
            len as c_int
        } else {
            0
        }
    }
}

/// Render the filetype for statusline.
///
/// Returns "[filetype]" or ",filetype" depending on format.
fn stl_render_filetype_impl(buf: &mut [u8], wp: WinHandle, bracketed: bool) -> c_int {
    if buf.is_empty() || wp.is_null() {
        return 0;
    }

    unsafe {
        let buf_handle = nvim_win_get_buffer(wp);
        if buf_handle.is_null() {
            return 0;
        }

        let ft_ptr = nvim_buf_get_b_p_ft(buf_handle);
        if ft_ptr.is_null() {
            return 0;
        }

        let ft = match CStr::from_ptr(ft_ptr).to_str() {
            Ok(s) if !s.is_empty() => s,
            _ => return 0,
        };

        let mut cursor = std::io::Cursor::new(buf);
        let result = if bracketed {
            write!(cursor, "[{ft}]")
        } else {
            write!(cursor, ",{ft}")
        };

        match result {
            #[allow(clippy::cast_possible_truncation)]
            Ok(()) => cursor.position() as c_int,
            Err(_) => 0,
        }
    }
}

/// Render line/column position for statusline.
///
/// Returns formatted string like "10" or "10/100" depending on format.
fn stl_render_line_col_impl(buf: &mut [u8], wp: WinHandle, show_total: bool) -> c_int {
    if buf.is_empty() || wp.is_null() {
        return 0;
    }

    unsafe {
        let lnum = nvim_win_get_cursor_lnum(wp);
        let line_count = nvim_win_buf_line_count(wp);

        let mut cursor = std::io::Cursor::new(buf);
        let result = if show_total {
            write!(cursor, "{lnum}/{line_count}")
        } else {
            write!(cursor, "{lnum}")
        };

        match result {
            #[allow(clippy::cast_possible_truncation)]
            Ok(()) => cursor.position() as c_int,
            Err(_) => 0,
        }
    }
}

/// Render percentage for statusline (%p item).
///
/// Returns a percentage string like "50".
#[allow(clippy::cast_sign_loss)]
fn stl_render_percentage_impl(buf: &mut [u8], wp: WinHandle) -> c_int {
    if buf.is_empty() || wp.is_null() {
        return 0;
    }

    unsafe {
        let lnum = nvim_win_get_cursor_lnum(wp);
        let line_count = nvim_win_buf_line_count(wp);

        // Calculate percentage using the same formula as calc_percentage
        let perc = if line_count > 0 {
            let lnum_i64 = i64::from(lnum);
            let count_i64 = i64::from(line_count);
            let result = ((lnum_i64 * 100) + (count_i64 / 2)) / count_i64;
            result.clamp(0, 100) as u8
        } else {
            0
        };

        let mut cursor = std::io::Cursor::new(buf);
        let result = write!(cursor, "{perc}");

        match result {
            #[allow(clippy::cast_possible_truncation)]
            Ok(()) => cursor.position() as c_int,
            Err(_) => 0,
        }
    }
}

/// FFI export: Render filename for statusline.
///
/// # Safety
/// `buf` must be null or valid pointer to buffer of `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_filename(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
    ftype: StlFilenameType,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_render_filename_impl(slice, wp, ftype)
}

/// FFI export: Render modified flag for statusline.
///
/// # Safety
/// `buf` must be null or valid pointer to buffer of `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_modified(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
    short: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_render_modified_impl(slice, wp, short != 0)
}

/// FFI export: Render readonly flag for statusline.
///
/// # Safety
/// `buf` must be null or valid pointer to buffer of `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_readonly(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
    short: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_render_readonly_impl(slice, wp, short != 0)
}

/// FFI export: Render filetype for statusline.
///
/// # Safety
/// `buf` must be null or valid pointer to buffer of `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_filetype(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
    bracketed: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_render_filetype_impl(slice, wp, bracketed != 0)
}

/// FFI export: Render line/column for statusline.
///
/// # Safety
/// `buf` must be null or valid pointer to buffer of `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_line_col(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
    show_total: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_render_line_col_impl(slice, wp, show_total != 0)
}

/// FFI export: Render percentage for statusline.
///
/// # Safety
/// `buf` must be null or valid pointer to buffer of `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_percentage(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    stl_render_percentage_impl(slice, wp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabwidth_calc() {
        // 80 columns with 5 tabs -> (80 - 1 + 2) / 5 = 81 / 5 = 16
        assert_eq!(tabwidth_calc_impl(80, 5), 16);
        // Minimum width is 6
        assert_eq!(tabwidth_calc_impl(20, 10), 6);
        // Edge case: 0 tabs
        assert_eq!(tabwidth_calc_impl(80, 0), 0);
        // Single tab
        assert_eq!(tabwidth_calc_impl(80, 1), 79);
    }

    #[test]
    fn test_col_print_same() {
        let mut buf = [0u8; 32];
        let len = col_print_impl(&mut buf, 42, 42);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"42");
    }

    #[test]
    fn test_col_print_different() {
        let mut buf = [0u8; 32];
        let len = col_print_impl(&mut buf, 10, 25);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"10-25");
    }

    #[test]
    fn test_col_print_empty_buffer() {
        let mut buf = [0u8; 0];
        let len = col_print_impl(&mut buf, 10, 25);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_col_print_small_buffer() {
        let mut buf = [0u8; 3];
        let len = col_print_impl(&mut buf, 10, 25);
        // Should write "10-" (truncated)
        assert!(len <= 3);
    }

    #[test]
    fn test_stl_click_type_values() {
        // Verify enum values match C definitions
        assert_eq!(StlClickType::Disabled as c_int, 0);
        assert_eq!(StlClickType::TabSwitch as c_int, 1);
        assert_eq!(StlClickType::TabClose as c_int, 2);
        assert_eq!(StlClickType::FuncRun as c_int, 3);
    }

    #[test]
    fn test_stl_click_definition_disabled() {
        let def = StlClickDefinition::disabled();
        assert!(def.is_disabled());
        assert_eq!(def.tabnr, 0);
        assert!(def.func.is_null());
    }

    #[test]
    fn test_highlight_group_constants() {
        // Verify highlight groups match C definitions
        assert_eq!(HLF_S, 27); // StatusLine
        assert_eq!(HLF_SNC, 28); // StatusLineNC
    }

    #[test]
    fn test_tabpage_handle_null() {
        let handle = TabpageHandle::null();
        assert!(handle.is_null());
    }

    // =========================================================================
    // Relative Position Tests
    // =========================================================================

    #[test]
    fn test_relative_position_enum() {
        // Test that enum variants are distinct
        assert_ne!(RelativePosition::All, RelativePosition::Top);
        assert_ne!(RelativePosition::Top, RelativePosition::Bot);
        assert_ne!(RelativePosition::Bot, RelativePosition::Percentage(50));
        assert_eq!(
            RelativePosition::Percentage(50),
            RelativePosition::Percentage(50)
        );
        assert_ne!(
            RelativePosition::Percentage(50),
            RelativePosition::Percentage(75)
        );
    }

    // =========================================================================
    // Separator Fill Tests
    // =========================================================================

    #[test]
    fn test_fill_between_basic() {
        let mut buf = [0u8; 20];
        let filled = stl_fill_between_impl(&mut buf, 5, 10, 20, 10, b' ');
        assert_eq!(filled, 5);
        // Check that bytes 5-9 are filled with spaces
        for byte in &buf[5..10] {
            assert_eq!(*byte, b' ');
        }
    }

    #[test]
    fn test_fill_between_empty_range() {
        let mut buf = [0u8; 20];
        let filled = stl_fill_between_impl(&mut buf, 10, 10, 20, 10, b' ');
        assert_eq!(filled, 0);
    }

    #[test]
    fn test_fill_between_invalid_range() {
        let mut buf = [0u8; 20];
        let filled = stl_fill_between_impl(&mut buf, 15, 10, 20, 10, b' ');
        assert_eq!(filled, 0);
    }

    #[test]
    fn test_fill_between_no_available_space() {
        let mut buf = [0u8; 20];
        // total_width == content_width means no space to fill
        let filled = stl_fill_between_impl(&mut buf, 5, 10, 15, 15, b' ');
        assert_eq!(filled, 0);
    }

    // =========================================================================
    // Group Truncation Tests
    // =========================================================================

    #[test]
    fn test_group_truncate_no_truncation_needed() {
        let (width, needs_marker) = stl_group_truncate_impl(10, 20, false);
        assert_eq!(width, 10);
        assert!(!needs_marker);
    }

    #[test]
    fn test_group_truncate_truncation_needed() {
        let (width, needs_marker) = stl_group_truncate_impl(30, 20, false);
        assert_eq!(width, 20);
        assert!(needs_marker);
    }

    #[test]
    fn test_group_truncate_already_has_marker() {
        let (width, needs_marker) = stl_group_truncate_impl(30, 20, true);
        assert_eq!(width, 20);
        assert!(needs_marker);
    }

    #[test]
    fn test_group_truncate_zero_max_width() {
        // max_width <= 0 means unlimited
        let (width, needs_marker) = stl_group_truncate_impl(30, 0, false);
        assert_eq!(width, 30);
        assert!(!needs_marker);
    }

    #[test]
    fn test_group_truncate_negative_max_width() {
        let (width, needs_marker) = stl_group_truncate_impl(30, -1, false);
        assert_eq!(width, 30);
        assert!(!needs_marker);
    }

    // =========================================================================
    // Group Item Struct Tests
    // =========================================================================

    #[test]
    fn test_stl_group_item_size() {
        // Should be 3 * 4 = 12 bytes (3 c_int fields)
        assert_eq!(std::mem::size_of::<StlGroupItem>(), 12);
    }

    // =========================================================================
    // Statusline Renderer Tests
    // =========================================================================

    #[test]
    fn test_stl_filename_type_values() {
        // Verify enum values
        assert_eq!(StlFilenameType::Short as c_int, 0);
        assert_eq!(StlFilenameType::FullPath as c_int, 1);
        assert_eq!(StlFilenameType::Tail as c_int, 2);
    }

    #[test]
    fn test_stl_filename_type_size() {
        // Should be 4 bytes (c_int sized)
        assert_eq!(std::mem::size_of::<StlFilenameType>(), 4);
    }

    // Note: Tests for stl_render_* functions require C FFI calls
    // and are tested through integration tests.
}
