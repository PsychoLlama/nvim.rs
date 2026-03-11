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

pub mod builder;
pub mod click;
pub mod draw;
#[allow(
    dead_code,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
pub mod draw_tabline;
pub mod eval;
pub mod format;
pub mod highlight;
#[allow(
    clippy::missing_const_for_thread_local,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::if_not_else,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr,
    clippy::too_many_lines
)]
pub mod redraw_ruler;
pub mod render;
pub mod ruler;
pub mod statuscol;
#[allow(
    dead_code,
    unused_variables,
    clippy::manual_c_str_literals,
    clippy::missing_const_for_thread_local,
    clippy::ptr_as_ptr,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::unnecessary_cast,
    clippy::bool_to_int_with_if,
    clippy::collapsible_else_if,
    clippy::if_not_else,
    clippy::needless_bool_assign,
    clippy::comparison_to_empty,
    clippy::too_many_lines,
    clippy::missing_const_for_fn,
    clippy::match_like_matches_macro,
    clippy::redundant_pattern_matching,
    clippy::unnecessary_operation,
    clippy::comparison_chain
)]
pub mod stl_build;
pub mod tabline;
pub mod ui;
pub mod ui_ext;
#[allow(
    dead_code,
    unused_variables,
    unused_assignments,
    clippy::manual_c_str_literals,
    clippy::missing_const_for_thread_local,
    clippy::ptr_as_ptr,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::needless_borrows_for_generic_args,
    clippy::if_not_else,
    clippy::comparison_chain,
    clippy::redundant_closure_for_method_calls,
    clippy::borrow_as_ptr,
    clippy::ptr_cast_constness
)]
pub mod win_redr;
pub mod winbar;

pub use builder::{BuilderItem, ItemType, StatuslineBuilder};
pub use click::{ClickDefinition, ClickRecord, ClickTracker, ClickType};
pub use draw::{DrawContext, DrawResult, TablineDrawState};
pub use eval::{EvalContext, EvalResult, NumberBase};
pub use format::{FormatParser, FormatSpec, StlFlag, StlFormatContext, StlItem, StlItemType};
pub use highlight::{HighlightTracker, StlHighlightRecord};
pub use render::{build_stl_str, BuildResult, RenderContext};
pub use statuscol::{LineNumberMode, StatusColContext};
pub use tabline::{TabInfo, TablineContext, WinbarContext};
pub use ui::{GridCell, GridSpan, UiContentBuilder, UiHighlight};

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
pub const HLF_S: c_int = 19;

/// Highlight group for StatusLineNC (non-current windows).
pub const HLF_SNC: c_int = 20;

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
#[export_name = "stl_connected"]
pub extern "C" fn rs_stl_connected(wp: WinHandle) -> c_int {
    c_int::from(stl_connected_impl(wp))
}

/// Get the fill character for a status line.
///
/// # Safety
/// `wp` must be a valid window handle.
/// `group` must be a valid pointer to an hlf_T value.
#[export_name = "fillchar_status"]
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

// =============================================================================
// FFI Exports for Format Parsing
// =============================================================================

/// FFI export: Check if a format flag character is numeric.
///
/// Returns 1 if the flag produces a numeric value, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_stl_flag_is_numeric(flag_char: u8) -> c_int {
    c_int::from(StlFlag::from_byte(flag_char).is_some_and(|f| f.is_numeric()))
}

/// FFI export: Check if a format flag character is a conditional flag.
///
/// Returns 1 if the flag is conditional (empty when condition not met), 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_stl_flag_is_flag_item(flag_char: u8) -> c_int {
    c_int::from(StlFlag::from_byte(flag_char).is_some_and(|f| f.is_flag_item()))
}

/// FFI export: Check if a format flag character allows fill character replacement.
///
/// Returns 1 if spaces can be replaced with fillchar, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_stl_flag_is_fillable(flag_char: u8) -> c_int {
    c_int::from(StlFlag::from_byte(flag_char).is_some_and(|f| f.is_fillable()))
}

// =============================================================================
// FFI Exports for Click Region Handling
// =============================================================================

/// FFI export: Determine click type from minwid value.
///
/// - `minwid > 0`: TabSwitch to tab number `minwid`
/// - `minwid < 0`: TabClose tab number `-minwid`
/// - `minwid == 0`: Disabled
///
/// Returns click type and sets `*tabnr` to the tab number.
///
/// # Safety
/// `tabnr` must be null or a valid pointer to c_int.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_click_type_from_minwid(minwid: c_int, tabnr: *mut c_int) -> c_int {
    let (click_type, tab) = click::click_type_from_minwid(minwid);
    if !tabnr.is_null() {
        *tabnr = tab;
    }
    click_type as c_int
}

/// FFI export: Check if a click type is valid for non-tabline use.
///
/// Window bar and status line only support click functions and disabled.
/// Tab switch/close are tabline-only.
#[no_mangle]
pub const extern "C" fn rs_stl_click_valid_for_statusline(click_type: c_int) -> c_int {
    let ct = match click_type {
        0 => click::ClickType::Disabled,
        1 => click::ClickType::TabSwitch,
        2 => click::ClickType::TabClose,
        3 => click::ClickType::FuncRun,
        _ => return 0,
    };
    click::is_valid_for_statusline(ct) as c_int
}

// =============================================================================
// FFI Exports for Status Column Rendering
// =============================================================================

/// FFI export: Determine line number mode from number/relativenumber options.
///
/// Returns:
/// - 0: None (no line numbers)
/// - 1: Absolute
/// - 2: Relative
/// - 3: Hybrid
#[no_mangle]
pub const extern "C" fn rs_stl_line_number_mode(number: c_int, relativenumber: c_int) -> c_int {
    statuscol::LineNumberMode::from_options(number != 0, relativenumber != 0) as c_int
}

/// FFI export: Calculate required width for line numbers.
///
/// Returns the number of digits needed to display line numbers,
/// with a minimum of 2 (matching Vim default).
#[no_mangle]
pub extern "C" fn rs_stl_calc_number_width(line_count: c_int) -> c_int {
    statuscol::calc_number_width(line_count)
}

/// FFI export: Render fold column to buffer.
///
/// Returns the number of bytes written.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_fold_column(
    buf: *mut u8,
    buflen: usize,
    fold_level: c_int,
    is_folded: c_int,
    max_level: c_int,
    width: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let result = statuscol::render_fold_column(fold_level, is_folded != 0, max_level, width);
    let bytes = result.as_bytes();
    let len = bytes.len().min(buflen);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (len as c_int)
}

/// FFI export: Render sign column to buffer.
///
/// Returns the number of bytes written.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
/// `sign_text` must be null or a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_sign_column(
    buf: *mut u8,
    buflen: usize,
    sign_text: *const c_char,
    width: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let sign = if sign_text.is_null() {
        None
    } else {
        CStr::from_ptr(sign_text).to_str().ok()
    };
    let result = statuscol::render_sign_column(sign, width);
    let bytes = result.as_bytes();
    let len = bytes.len().min(buflen);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (len as c_int)
}

// =============================================================================
// FFI Exports for Highlight Tracking
// =============================================================================

/// FFI export: Parse a highlight name and return user highlight number.
///
/// For "%1*" through "%9*", returns 1-9.
/// For "%0*" or "%*", returns 0 (reset to default).
/// For named highlights like "%#GroupName#", returns -1 (caller must look up).
///
/// Returns -2 on parse error.
#[no_mangle]
pub extern "C" fn rs_stl_parse_user_highlight(hl_char: u8) -> c_int {
    if hl_char == b'*' || hl_char == b'0' {
        0
    } else if hl_char.is_ascii_digit() {
        c_int::from(hl_char - b'0')
    } else {
        -2 // Invalid
    }
}

// =============================================================================
// FFI Exports for Tabline
// =============================================================================

// =============================================================================
// Format Width Parsing Helpers
// =============================================================================

/// Result of parsing a width specifier from a format string.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StlWidthSpec {
    /// Minimum width (negative if left-aligned)
    pub minwid: c_int,
    /// Maximum width
    pub maxwid: c_int,
    /// Whether to zero-pad numeric items
    pub zeropad: bool,
    /// Whether item is left-aligned
    pub left_align: bool,
    /// Number of bytes consumed from input
    pub consumed: c_int,
}

/// Parse width specifier flags and numbers from format string.
///
/// Parses: [-][0][minwid][.maxwid]
///
/// # Safety
/// `fmt` must be a valid pointer to a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_parse_width_spec(fmt: *const c_char) -> StlWidthSpec {
    if fmt.is_null() {
        return StlWidthSpec {
            minwid: 0,
            maxwid: 9999,
            zeropad: false,
            left_align: false,
            consumed: 0,
        };
    }

    let bytes = std::ffi::CStr::from_ptr(fmt).to_bytes();
    let mut pos = 0;

    // Parse flags
    let zeropad = bytes.get(pos) == Some(&b'0');
    if zeropad {
        pos += 1;
    }

    let left_align = bytes.get(pos) == Some(&b'-');
    if left_align {
        pos += 1;
    }

    // Parse minimum width
    let mut minwid: c_int = 0;
    while let Some(&c) = bytes.get(pos) {
        if !c.is_ascii_digit() {
            break;
        }
        minwid = minwid
            .saturating_mul(10)
            .saturating_add(c_int::from(c - b'0'));
        pos += 1;
    }

    // Clamp to 50
    minwid = minwid.min(50);
    if left_align {
        minwid = -minwid;
    }

    // Parse max width if present
    let maxwid = if bytes.get(pos) == Some(&b'.') {
        pos += 1;
        let mut w: c_int = 0;
        while let Some(&c) = bytes.get(pos) {
            if !c.is_ascii_digit() {
                break;
            }
            w = w.saturating_mul(10).saturating_add(c_int::from(c - b'0'));
            pos += 1;
        }
        if w == 0 {
            50
        } else {
            w
        }
    } else {
        9999
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    StlWidthSpec {
        minwid,
        maxwid,
        zeropad,
        left_align,
        consumed: pos as c_int,
    }
}

/// Check if a character is a valid statusline format flag.
///
/// Returns 1 if the character is a valid flag (f, F, t, l, c, etc.), 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_stl_is_valid_flag(c: u8) -> c_int {
    if format::StlFlag::from_byte(c).is_some() {
        1
    } else {
        0
    }
}

/// Check if a character is a simple format specifier (%, =, <, ), }).
///
/// Returns 1 if the character is a simple specifier, 0 otherwise.
#[no_mangle]
pub const extern "C" fn rs_stl_is_simple_specifier(c: u8) -> c_int {
    match c {
        b'%' | b'=' | b'<' | b')' | b'}' => 1,
        _ => 0,
    }
}

// =============================================================================
// FFI Exports for Item Rendering and Truncation
// =============================================================================

/// Result of applying truncation to a string.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StlTruncResult {
    /// Number of bytes after truncation
    pub truncated_len: c_int,
    /// Number of display cells after truncation
    pub truncated_width: c_int,
    /// Whether truncation was applied
    pub was_truncated: bool,
}

/// Apply truncation to a string from the beginning.
///
/// If the string width exceeds `max_width`, truncates from the beginning
/// and prepends a '<' marker.
///
/// Returns the result with the new length and width.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn apply_truncation_impl(buf: &mut [u8], content_len: usize, max_width: c_int) -> StlTruncResult {
    if max_width <= 0 || content_len == 0 || buf.is_empty() {
        return StlTruncResult {
            truncated_len: content_len as c_int,
            truncated_width: content_len as c_int, // Simplified: assume 1 byte = 1 cell
            was_truncated: false,
        };
    }

    // Simplified width calculation: count bytes (proper impl would use vim_strsize)
    let width = content_len as c_int;

    if width <= max_width {
        return StlTruncResult {
            truncated_len: content_len as c_int,
            truncated_width: width,
            was_truncated: false,
        };
    }

    // Calculate how many bytes to remove
    #[allow(clippy::cast_sign_loss)]
    let excess = (width - max_width + 1) as usize; // +1 for '<' marker

    if excess >= content_len {
        // Content is completely truncated
        if !buf.is_empty() {
            buf[0] = b'<';
        }
        return StlTruncResult {
            truncated_len: 1,
            truncated_width: 1,
            was_truncated: true,
        };
    }

    // Shift content and prepend '<'
    let new_len = content_len - excess + 1;
    buf.copy_within(excess..content_len, 1);
    buf[0] = b'<';

    StlTruncResult {
        truncated_len: new_len as c_int,
        truncated_width: max_width,
        was_truncated: true,
    }
}

/// FFI export: Apply truncation to a string buffer.
///
/// Truncates from the beginning if the string exceeds `max_width`,
/// prepending a '<' marker.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_apply_truncation(
    buf: *mut u8,
    buflen: usize,
    content_len: usize,
    max_width: c_int,
) -> StlTruncResult {
    if buf.is_null() || buflen == 0 {
        return StlTruncResult {
            truncated_len: 0,
            truncated_width: 0,
            was_truncated: false,
        };
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    apply_truncation_impl(slice, content_len.min(buflen), max_width)
}

/// Apply padding to content based on minimum width.
///
/// Returns the number of fill characters needed and whether to pad on the left.
const fn compute_padding_impl(content_width: c_int, minwid: c_int) -> (c_int, bool) {
    if minwid == 0 {
        return (0, false);
    }

    let abs_minwid = minwid.abs();
    let pad_left = minwid > 0; // Positive = right-aligned = pad left

    if content_width >= abs_minwid {
        (0, pad_left)
    } else {
        (abs_minwid - content_width, pad_left)
    }
}

/// FFI export: Compute padding needed for an item.
///
/// Returns the number of fill characters needed.
/// Sets `*pad_left` to 1 if padding should be on the left (right-aligned).
///
/// # Safety
/// `pad_left` must be null or a valid pointer to a c_int.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_compute_padding(
    content_width: c_int,
    minwid: c_int,
    pad_left: *mut c_int,
) -> c_int {
    let (padding, left) = compute_padding_impl(content_width, minwid);
    if !pad_left.is_null() {
        *pad_left = c_int::from(left);
    }
    padding
}

/// Check if an item should be skipped based on flag state.
///
/// For flag items like readonly or modified, the leading separator
/// may need to be skipped based on context.
#[no_mangle]
pub extern "C" fn rs_stl_should_skip_leading(
    first_char: u8,
    prevchar_isflag: c_int,
    prevchar_isitem: c_int,
) -> c_int {
    // Skip leading comma if not preceded by an item
    // Skip leading space if preceded by a flag
    let skip = (first_char == b',' && prevchar_isitem == 0)
        || (first_char == b' ' && prevchar_isflag != 0);
    c_int::from(skip)
}

/// Render a number item with optional formatting.
///
/// Writes the number to the buffer with the specified formatting options.
/// Returns the number of bytes written.
fn render_number_impl(
    buf: &mut [u8],
    num: c_int,
    base: c_int,
    minwid: c_int,
    zeropad: bool,
) -> c_int {
    if buf.is_empty() {
        return 0;
    }

    let mut cursor = std::io::Cursor::new(buf);

    // Determine formatting
    let left_align = minwid < 0;
    let width = minwid.unsigned_abs() as usize;

    let result = if base == 16 {
        if zeropad && width > 0 && !left_align {
            write!(cursor, "{num:0>width$X}")
        } else if width > 0 && left_align {
            write!(cursor, "{num:<width$X}")
        } else if width > 0 {
            write!(cursor, "{num:>width$X}")
        } else {
            write!(cursor, "{num:X}")
        }
    } else if zeropad && width > 0 && !left_align {
        write!(cursor, "{num:0>width$}")
    } else if width > 0 && left_align {
        write!(cursor, "{num:<width$}")
    } else if width > 0 {
        write!(cursor, "{num:>width$}")
    } else {
        write!(cursor, "{num}")
    };

    match result {
        #[allow(clippy::cast_possible_truncation)]
        Ok(()) => cursor.position() as c_int,
        Err(_) => 0,
    }
}

/// FFI export: Render a number with formatting.
///
/// Writes the number to buffer with the specified base (10 or 16),
/// minimum width, and zero-padding.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_render_number(
    buf: *mut u8,
    buflen: usize,
    num: c_int,
    base: c_int,
    minwid: c_int,
    zeropad: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    render_number_impl(slice, num, base, minwid, zeropad != 0)
}

/// Calculate the display width of a number in a given base.
#[no_mangle]
pub const extern "C" fn rs_stl_number_width(num: c_int, base: c_int) -> c_int {
    if num == 0 {
        return 1;
    }

    let abs_num = num.unsigned_abs();
    let b = if base == 16 { 16_u32 } else { 10_u32 };

    let mut n = abs_num;
    let mut width = 0;
    while n > 0 {
        width += 1;
        n /= b;
    }
    width
}

// =============================================================================
// FFI Exports for StatuslineBuilder Operations
// =============================================================================

/// Opaque handle to a Rust StatuslineBuilder.
#[repr(C)]
pub struct StlBuilderHandle(*mut StatuslineBuilder);

impl StlBuilderHandle {
    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

/// Create a new statusline builder.
///
/// Returns an opaque handle to a newly allocated builder.
/// Must be freed with `rs_stl_builder_free`.
#[no_mangle]
pub extern "C" fn rs_stl_builder_new(max_width: c_int) -> StlBuilderHandle {
    #[allow(clippy::cast_sign_loss)]
    let builder = Box::new(StatuslineBuilder::new(max_width.max(0) as usize));
    StlBuilderHandle(Box::into_raw(builder))
}

/// Free a statusline builder.
///
/// # Safety
/// `handle` must be a valid handle from `rs_stl_builder_new`, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_free(handle: StlBuilderHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle.0));
    }
}

/// Clear the builder for reuse.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_clear(handle: StlBuilderHandle) {
    if !handle.is_null() {
        (*handle.0).clear();
    }
}

/// Get current output position in the builder.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_position(handle: StlBuilderHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    ((*handle.0).position() as c_int)
}

/// Append literal text to the builder.
///
/// # Safety
/// `handle` must be a valid builder handle.
/// `text` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_append(
    handle: StlBuilderHandle,
    text: *const c_char,
    len: c_int,
) {
    if handle.is_null() || text.is_null() {
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let slice = std::slice::from_raw_parts(text.cast::<u8>(), len.max(0) as usize);
    if let Ok(s) = std::str::from_utf8(slice) {
        (*handle.0).append_literal(s);
    }
}

/// Append a single byte to the builder.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_append_byte(handle: StlBuilderHandle, byte: u8) {
    if !handle.is_null() {
        (*handle.0).append_byte(byte);
    }
}

/// Start a group in the builder.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_start_group(
    handle: StlBuilderHandle,
    minwid: c_int,
    maxwid: c_int,
) {
    if !handle.is_null() {
        (*handle.0).start_group(minwid, maxwid);
    }
}

/// End the current group in the builder.
///
/// Returns 1 if a group was ended, 0 if not in a group.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_end_group(handle: StlBuilderHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }
    c_int::from((*handle.0).end_group())
}

/// Check if builder is inside a group.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_in_group(handle: StlBuilderHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }
    c_int::from((*handle.0).in_group())
}

/// Add a separator marker.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_add_separator(handle: StlBuilderHandle) {
    if !handle.is_null() {
        (*handle.0).add_separator();
    }
}

/// Add a truncation marker.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_add_truncation(handle: StlBuilderHandle) {
    if !handle.is_null() {
        (*handle.0).add_truncation_marker();
    }
}

/// Set highlight at current position.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_set_highlight(handle: StlBuilderHandle, userhl: c_int) {
    if !handle.is_null() {
        (*handle.0).set_highlight(userhl);
    }
}

/// Reset highlight to default.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_reset_highlight(handle: StlBuilderHandle) {
    if !handle.is_null() {
        (*handle.0).reset_highlight();
    }
}

/// Add a tab page click region.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_add_tab_page(handle: StlBuilderHandle, tabnr: c_int) {
    if !handle.is_null() {
        (*handle.0).add_tab_page(tabnr);
    }
}

/// Finalize the builder with separator filling.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_finalize(handle: StlBuilderHandle, target_width: c_int) {
    if !handle.is_null() {
        #[allow(clippy::cast_sign_loss)]
        (*handle.0).finalize(target_width.max(0) as usize);
    }
}

/// Copy builder output to a C buffer.
///
/// Returns the number of bytes copied.
///
/// # Safety
/// `handle` must be a valid builder handle.
/// `buf` must be a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_get_output(
    handle: StlBuilderHandle,
    buf: *mut u8,
    buflen: usize,
) -> c_int {
    if handle.is_null() || buf.is_null() || buflen == 0 {
        return 0;
    }

    let output = (*handle.0).output();
    let len = output.len().min(buflen - 1); // Leave room for NUL
    std::ptr::copy_nonoverlapping(output.as_ptr(), buf, len);
    *buf.add(len) = 0; // NUL terminate

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (len as c_int)
}

/// Get the number of items in the builder.
///
/// # Safety
/// `handle` must be a valid builder handle.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_builder_item_count(handle: StlBuilderHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    ((*handle.0).item_count() as c_int)
}

/// FFI export: Calculate tab width for tabline.
///
/// Shortens tab labels to fit within max_width.
/// Returns number of bytes written to buf.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
/// `path` must be null or a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_shorten_tab_label(
    buf: *mut u8,
    buflen: usize,
    path: *const c_char,
    max_width: usize,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let path_str = if path.is_null() {
        ""
    } else {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };
    let result = tabline::shorten_tab_label(path_str, max_width);
    let bytes = result.as_bytes();
    let len = bytes.len().min(buflen);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (len as c_int)
}

// =============================================================================
// FFI Exports for Printable Character Conversion
// =============================================================================

/// Convert a byte to its printable representation.
///
/// Control characters (0x00-0x1F) become ^@ through ^_
/// DEL (0x7F) becomes ^?
/// High bytes (0x80-0xFF) become <XX> hex notation
/// Printable ASCII is returned as-is.
///
/// Returns the number of bytes written to buf.
fn make_printable_impl(buf: &mut [u8], byte: u8) -> usize {
    if buf.is_empty() {
        return 0;
    }

    if byte == 0 {
        // NUL -> ^@
        if buf.len() >= 2 {
            buf[0] = b'^';
            buf[1] = b'@';
            return 2;
        }
        return 0;
    } else if byte < 0x20 {
        // Control chars 0x01-0x1F -> ^A through ^_
        if buf.len() >= 2 {
            buf[0] = b'^';
            buf[1] = byte + b'@';
            return 2;
        }
        return 0;
    } else if byte == 0x7F {
        // DEL -> ^?
        if buf.len() >= 2 {
            buf[0] = b'^';
            buf[1] = b'?';
            return 2;
        }
        return 0;
    } else if byte >= 0x80 {
        // High bytes -> <XX>
        if buf.len() >= 4 {
            buf[0] = b'<';
            buf[1] = HEX_CHARS[(byte >> 4) as usize];
            buf[2] = HEX_CHARS[(byte & 0x0F) as usize];
            buf[3] = b'>';
            return 4;
        }
        return 0;
    }

    // Printable ASCII
    buf[0] = byte;
    1
}

/// Hex characters for conversion.
const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

/// FFI export: Make a byte printable.
///
/// Returns the number of bytes written to buf (1, 2, or 4).
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least 4 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_make_printable(buf: *mut u8, buflen: usize, byte: u8) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (make_printable_impl(slice, byte) as c_int)
}

/// Convert a string to printable form (like transstr_buf).
///
/// Returns the number of bytes written to out.
fn transstr_impl(out: &mut [u8], input: &[u8]) -> usize {
    let mut out_pos = 0;
    for &byte in input {
        let remaining = &mut out[out_pos..];
        if remaining.is_empty() {
            break;
        }
        let written = make_printable_impl(remaining, byte);
        if written == 0 {
            break;
        }
        out_pos += written;
    }
    out_pos
}

/// FFI export: Convert string to printable form.
///
/// Similar to transstr_buf in C. Converts unprintable characters to
/// their printable representations (^X for control chars, <XX> for high bytes).
///
/// Returns the number of bytes written to out.
///
/// # Safety
/// - `out` must be null or a valid pointer to a buffer of at least `outlen` bytes.
/// - `input` must be null or a valid pointer to `inputlen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_transstr(
    out: *mut u8,
    outlen: usize,
    input: *const u8,
    inputlen: usize,
) -> c_int {
    if out.is_null() || outlen == 0 {
        return 0;
    }
    let out_slice = std::slice::from_raw_parts_mut(out, outlen);
    let input_slice = if input.is_null() || inputlen == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(input, inputlen)
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (transstr_impl(out_slice, input_slice) as c_int)
}

// =============================================================================
// FFI Exports for Tabline Tab Counting
// =============================================================================

/// FFI export: Check if a character indicates end of tab display.
///
/// Returns 1 if the character is a space or separator that ends a tab display.
#[no_mangle]
pub const extern "C" fn rs_stl_is_tab_separator(c: u8) -> c_int {
    if c == b' ' || c == b'|' {
        1
    } else {
        0
    }
}

/// FFI export: Calculate display width of window count in tabline.
///
/// For counts > 1, returns digit count. For count <= 1, returns 0.
#[no_mangle]
pub const extern "C" fn rs_stl_wincount_width(count: c_int) -> c_int {
    if count <= 1 {
        return 0;
    }
    let mut width = 0;
    let mut n = count;
    while n > 0 {
        width += 1;
        n /= 10;
    }
    width
}

/// FFI export: Format window count for tabline display.
///
/// Writes the count followed by optional '+' for modified.
/// Returns the number of bytes written.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_stl_format_wincount(
    buf: *mut u8,
    buflen: usize,
    count: c_int,
    modified: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    let mut cursor = std::io::Cursor::new(std::slice::from_raw_parts_mut(buf, buflen));

    let mut written = 0;

    // Write count if > 1
    if count > 1 && write!(cursor, "{count}").is_ok() {
        #[allow(clippy::cast_possible_wrap)]
        {
            written = cursor.position() as c_int;
        }
    }

    // Write + if modified
    if modified != 0 && (cursor.position() as usize) < buflen {
        let pos = cursor.position() as usize;
        (*buf.add(pos)) = b'+';
        written += 1;
    }

    written
}

// =============================================================================
// FFI Exports for Item Evaluation Functions
// =============================================================================

/// FFI export: Evaluate argument list status.
///
/// Returns the argument list status string like "(2 of 8)" or "((2) of 8)".
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
/// `wp` must be a valid window handle.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_stl_eval_arglist_status(
    buf: *mut u8,
    buflen: usize,
    wp: WinHandle,
) -> c_int {
    if buf.is_null() || buflen == 0 || wp.is_null() {
        return 0;
    }

    let argcount = nvim_win_argcount(wp);
    if argcount <= 1 {
        return 0;
    }

    let arg_idx = nvim_win_get_arg_idx(wp);
    let arg_idx_invalid = nvim_win_get_arg_idx_invalid(wp) != 0;

    let mut cursor = std::io::Cursor::new(std::slice::from_raw_parts_mut(buf, buflen));

    let result = if arg_idx_invalid {
        write!(cursor, "(({}) of {})", arg_idx + 1, argcount)
    } else {
        write!(cursor, "({} of {})", arg_idx + 1, argcount)
    };

    match result {
        Ok(()) => cursor.position() as c_int,
        Err(_) => 0,
    }
}

// =============================================================================
// Phase 1: Small Helper Functions (get_trans_bufname, redraw_custom_statusline,
//          build_statuscol_str)
// =============================================================================

/// Opaque handle to C's statuscol_T
type StatuscolHandle = *mut c_void;

/// Opaque handle to C's stl_hlrec_t pointer-to-pointer
type HlrecPtrPtr = *mut c_void;

// Phase 1 C accessors
extern "C" {
    fn nvim_stl_get_trans_bufname(buf: BufHandle);
    #[link_name = "rs_win_redr_custom"]
    fn nvim_stl_win_redr_custom_direct(
        wp: WinHandle,
        draw_winbar: bool,
        draw_ruler: bool,
        ui_event: bool,
    );
    fn nvim_stl_set_vv_lnum(lnum: i64);
    fn nvim_stl_set_vv_relnum(relnum: i64);
    fn nvim_stl_win_get_p_stc(wp: WinHandle) -> *const c_char;
    fn nvim_stl_build_stl_str_hl(
        wp: WinHandle,
        buf: *mut c_char,
        buflen: c_int,
        stc: *const c_char,
        maxwidth: c_int,
        hlrec: HlrecPtrPtr,
        clickrec: *mut *mut c_void,
        stcp: StatuscolHandle,
    ) -> c_int;
    fn nvim_stl_win_get_statuscol_click_defs(wp: WinHandle) -> *mut c_void;
    fn nvim_stl_win_get_statuscol_click_defs_size(wp: WinHandle) -> usize;
    fn nvim_stl_win_set_statuscol_click_defs(wp: WinHandle, defs: *mut c_void);
    fn nvim_stl_win_set_statuscol_click_defs_size(wp: WinHandle, size: usize);
    fn nvim_stl_stcp_get_width(stcp: StatuscolHandle) -> c_int;
    fn nvim_stl_stcp_get_hlrec_ptr(stcp: StatuscolHandle) -> HlrecPtrPtr;
    #[link_name = "nvim_win_get_topline"]
    fn nvim_stl_win_get_topline(wp: WinHandle) -> c_int;
}

/// MAXPATHL constant (matches C definition in os_defs.h).
const MAXPATHL: usize = 4096;

/// FFI export: Fill NameBuff with the translated buffer name.
///
/// This is the Rust replacement for `get_trans_bufname()` in statusline.c.
/// The actual work is done by the C accessor `nvim_stl_get_trans_bufname`
/// which calls buf_spname/home_replace/trans_characters.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[export_name = "get_trans_bufname"]
pub unsafe extern "C" fn rs_get_trans_bufname(buf: BufHandle) {
    nvim_stl_get_trans_bufname(buf);
}

/// FFI export: Redraw the status line according to 'statusline'.
///
/// This is the Rust replacement for `redraw_custom_statusline()` in statusline.c.
/// Uses a thread-local recursion guard and delegates to C `win_redr_custom`.
///
/// # Safety
/// `wp` must be a valid window handle.
#[export_name = "redraw_custom_statusline"]
pub unsafe extern "C" fn rs_redraw_custom_statusline(wp: WinHandle) {
    use std::cell::Cell;

    thread_local! {
        static ENTERED: Cell<bool> = const { Cell::new(false) };
    }

    // When called recursively return.  This can happen when the statusline
    // contains an expression that triggers a redraw.
    let already_entered = ENTERED.with(|e| {
        if e.get() {
            true
        } else {
            e.set(true);
            false
        }
    });

    if already_entered {
        return;
    }

    nvim_stl_win_redr_custom_direct(wp, false, false, false);

    ENTERED.with(|e| e.set(false));
}

/// FFI export: Build the 'statuscolumn' string for a line.
///
/// This is the Rust replacement for `build_statuscol_str()` in statusline.c.
///
/// @return The width of the built status column string for the line.
///
/// # Safety
/// - `wp` must be a valid window handle.
/// - `lnum` and `relnum` must be valid line numbers (relnum can be -1).
/// - `buf` must be a valid pointer to a buffer of at least MAXPATHL bytes.
/// - `stcp` must be a valid pointer to a statuscol_T struct.
#[export_name = "build_statuscol_str"]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_build_statuscol_str(
    wp: WinHandle,
    lnum: LinenrT,
    relnum: LinenrT,
    buf: *mut c_char,
    stcp: StatuscolHandle,
) -> c_int {
    // Only update click definitions once per window per redraw.
    // Don't update when current width is 0, since it will be redrawn again if not empty.
    let stcp_width = nvim_stl_stcp_get_width(stcp);
    let topline = nvim_stl_win_get_topline(wp);
    let fillclick = relnum >= 0 && stcp_width > 0 && lnum == topline;

    if relnum >= 0 {
        nvim_stl_set_vv_lnum(i64::from(lnum));
        nvim_stl_set_vv_relnum(i64::from(relnum));
    }

    let stc = nvim_stl_win_get_p_stc(wp);
    let hlrec_ptr = nvim_stl_stcp_get_hlrec_ptr(stcp);

    let mut clickrec: *mut c_void = std::ptr::null_mut();

    let width = nvim_stl_build_stl_str_hl(
        wp,
        buf,
        MAXPATHL as c_int,
        stc,
        stcp_width,
        hlrec_ptr,
        if fillclick {
            &raw mut clickrec
        } else {
            std::ptr::null_mut()
        },
        stcp,
    );

    if fillclick {
        // Clear existing click defs
        let old_defs = nvim_stl_win_get_statuscol_click_defs(wp);
        let old_size = nvim_stl_win_get_statuscol_click_defs_size(wp);
        click::rs_stl_clear_click_defs(old_defs.cast(), old_size);

        // Allocate new click defs
        let mut new_size = old_size;
        let new_defs = click::rs_stl_alloc_click_defs(old_defs.cast(), width, &raw mut new_size);
        nvim_stl_win_set_statuscol_click_defs(wp, new_defs.cast::<c_void>());
        nvim_stl_win_set_statuscol_click_defs_size(wp, new_size);

        // Fill click defs
        click::rs_stl_fill_click_defs(new_defs, clickrec.cast(), buf.cast(), width, false);
    }

    width
}

// =============================================================================
// Phase 2: Status/Winbar Entry Points (win_redr_status, win_redr_winbar)
// =============================================================================

/// Opaque handle to C's GridView
type GridViewHandle = *mut c_void;

// Phase 2 C accessors
extern "C" {
    #[link_name = "rs_global_stl_height"]
    fn nvim_global_stl_height() -> c_int;
    fn nvim_stl_wildmenu_blocking() -> c_int;
    fn nvim_win_set_redr_status(wp: WinHandle, val: c_int);
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;
    fn nvim_set_redraw_cmdline(val: bool);
    fn nvim_redrawing() -> c_int;
    fn nvim_stl_win_get_p_stl(wp: WinHandle) -> *const c_char;
    fn nvim_stl_get_p_stl() -> *const c_char;
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_vsep_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_fcs_vert(wp: WinHandle) -> ScharT;
    fn nvim_win_hl_attr(wp: WinHandle, hlf: c_int) -> c_int;
    fn nvim_get_default_gridview() -> GridViewHandle;
    fn nvim_win_get_endrow(wp: WinHandle) -> c_int;
    fn nvim_win_get_endcol(wp: WinHandle) -> c_int;
    fn rs_grid_line_start(view: GridViewHandle, row: c_int);
    fn rs_grid_line_put_schar(col: c_int, schar: ScharT, attr: c_int);
    fn rs_grid_line_flush();
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;
    fn nvim_stl_get_p_wbr() -> *const c_char;
    fn nvim_win_get_p_wbr(wp: WinHandle) -> *const c_char;
}

/// HLF_C constant - window split separators highlight group.
const HLF_C: c_int = 21;

/// FFI export: Redraw the status line of window `wp`.
///
/// This is the Rust replacement for `win_redr_status()` in statusline.c.
/// If inversion is possible we use it, else '=' characters are used.
///
/// # Safety
/// `wp` must be a valid window handle.
#[export_name = "win_redr_status"]
pub unsafe extern "C" fn rs_win_redr_status(wp: WinHandle) {
    use std::cell::Cell;

    thread_local! {
        static BUSY: Cell<bool> = const { Cell::new(false) };
    }

    let is_stl_global = nvim_global_stl_height() > 0;

    // May get here recursively when 'statusline' (indirectly)
    // invokes ":redrawstatus".  Simply ignore the call then.
    // Also ignore if wildmenu is showing.
    let blocked = BUSY.with(Cell::get) || nvim_stl_wildmenu_blocking() != 0;
    if blocked {
        return;
    }

    BUSY.with(|b| b.set(true));
    nvim_win_set_redr_status(wp, 0);

    if nvim_win_get_status_height(wp) == 0 && !(is_stl_global && nvim_win_is_curwin(wp) != 0) {
        // no status line, either global statusline is enabled or the window is a last window
        nvim_set_redraw_cmdline(true);
    } else if nvim_redrawing() == 0 {
        // Don't redraw right now, do it later. Don't update status line when
        // popup menu is visible and may be drawn over it
        nvim_win_set_redr_status(wp, 1);
    } else {
        let w_p_stl = nvim_stl_win_get_p_stl(wp);
        let p_stl = nvim_stl_get_p_stl();
        let has_custom_stl = !w_p_stl.is_null() && *w_p_stl != 0;
        let has_global_stl = !p_stl.is_null() && *p_stl != 0;
        let is_floating = nvim_win_get_floating(wp) != 0;
        let is_curwin = nvim_win_is_curwin(wp) != 0;

        if has_custom_stl || (has_global_stl && (!is_floating || (is_stl_global && is_curwin))) {
            // redraw custom status line
            rs_redraw_custom_statusline(wp);
        }
    }

    // May need to draw the character below the vertical separator.
    let mut group: c_int = HLF_C;
    if nvim_win_get_vsep_width(wp) != 0
        && nvim_win_get_status_height(wp) != 0
        && nvim_redrawing() != 0
    {
        let fillchar = if rs_stl_connected(wp) != 0 {
            rs_fillchar_status(&raw mut group, wp)
        } else {
            nvim_win_get_fcs_vert(wp)
        };
        let attr = nvim_win_hl_attr(wp, group);
        let gridview = nvim_get_default_gridview();
        rs_grid_line_start(gridview, nvim_win_get_endrow(wp));
        rs_grid_line_put_schar(nvim_win_get_endcol(wp), fillchar, attr);
        rs_grid_line_flush();
    }

    BUSY.with(|b| b.set(false));
}

/// FFI export: Redraw the window bar of window `wp`.
///
/// This is the Rust replacement for `win_redr_winbar()` in statusline.c.
///
/// # Safety
/// `wp` must be a valid window handle.
#[export_name = "win_redr_winbar"]
pub unsafe extern "C" fn rs_win_redr_winbar(wp: WinHandle) {
    use std::cell::Cell;

    thread_local! {
        static ENTERED: Cell<bool> = const { Cell::new(false) };
    }

    // Return when called recursively. This can happen when the winbar contains an expression
    // that triggers a redraw.
    let already_entered = ENTERED.with(|e| {
        if e.get() {
            true
        } else {
            e.set(true);
            false
        }
    });

    if already_entered {
        return;
    }

    if nvim_win_get_winbar_height(wp) != 0 && nvim_redrawing() != 0 {
        let p_wbr = nvim_stl_get_p_wbr();
        let w_p_wbr = nvim_win_get_p_wbr(wp);
        let has_global_wbr = !p_wbr.is_null() && *p_wbr != 0;
        let has_win_wbr = !w_p_wbr.is_null() && *w_p_wbr != 0;

        if has_global_wbr || has_win_wbr {
            nvim_stl_win_redr_custom_direct(wp, true, false, false);
        }
    }

    ENTERED.with(|e| e.set(false));
}

// =============================================================================
// Phase 3: Ruler and UI Ext Tabline (redraw_ruler, ui_ext_tabline_update)
// =============================================================================

// Phase 3 C accessors
extern "C" {
    fn nvim_stl_emit_tabline_update(
        tab_handles: *const c_int,
        tab_names: *const *const c_char,
        tab_count: c_int,
        buf_handles: *const c_int,
        buf_names: *const *const c_char,
        buf_count: c_int,
        curtab_handle: c_int,
        curbuf_handle: c_int,
    );
}

/// FFI export: Redraw the ruler.
///
/// This is the Rust replacement for `redraw_ruler()` in statusline.c.
/// Handles string formatting, grid operations, and UI event calls.
///
/// # Safety
/// Accesses global C state.
#[export_name = "redraw_ruler"]
pub unsafe extern "C" fn rs_redraw_ruler() {
    redraw_ruler::redraw_ruler();
}

/// FFI export: Update the external UI tabline.
///
/// Collects tab/buffer data in Rust and passes flat arrays to C for
/// arena-based API object construction and `ui_call_tabline_update`.
///
/// # Safety
/// Accesses global C state (tab/buffer lists).
#[no_mangle]
pub unsafe extern "C" fn rs_ui_ext_tabline_update() {
    use ui_ext::collect_tabline_data;

    let data = collect_tabline_data();

    // Build flat arrays of handles and C string pointers for the C accessor.
    let tab_handles: Vec<c_int> = data.tabs.iter().map(|t| t.handle).collect();
    let tab_name_cstrings: Vec<std::ffi::CString> = data
        .tabs
        .iter()
        .map(|t| std::ffi::CString::new(t.name.as_str()).unwrap_or_default())
        .collect();
    let tab_name_ptrs: Vec<*const c_char> = tab_name_cstrings.iter().map(|s| s.as_ptr()).collect();

    let buf_handles: Vec<c_int> = data.buffers.iter().map(|b| b.handle).collect();
    let buf_name_cstrings: Vec<std::ffi::CString> = data
        .buffers
        .iter()
        .map(|b| std::ffi::CString::new(b.name.as_str()).unwrap_or_default())
        .collect();
    let buf_name_ptrs: Vec<*const c_char> = buf_name_cstrings.iter().map(|s| s.as_ptr()).collect();

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let tab_count = tab_handles.len() as c_int;
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let buf_count = buf_handles.len() as c_int;

    nvim_stl_emit_tabline_update(
        tab_handles.as_ptr(),
        tab_name_ptrs.as_ptr(),
        tab_count,
        buf_handles.as_ptr(),
        buf_name_ptrs.as_ptr(),
        buf_count,
        data.current_tab,
        data.current_buffer,
    );
}

// =============================================================================
// Phase 4: Tabline Drawing (draw_tabline)
// =============================================================================

/// FFI export: Draw the tab pages line at the top of the Vim window.
///
/// This is the Rust replacement for `draw_tabline()` in statusline.c.
/// Handles grid operations, tab iteration, click definitions, and showcmd rendering.
///
/// # Safety
/// Accesses global C state (grid, tabs, click defs).
#[export_name = "draw_tabline"]
pub unsafe extern "C" fn rs_draw_tabline() {
    draw_tabline::draw_tabline();
}

// =============================================================================
// Phase 5: Custom Rendering Orchestrator (win_redr_custom)
// =============================================================================

/// FFI export: Redraw the status line, window bar, ruler or tabline using
/// a custom format string.
///
/// This is the Rust replacement for `win_redr_custom()` in statusline.c.
/// Handles mode determination, grid selection, format string parsing,
/// highlight record iteration, and click definition filling.
///
/// # Safety
/// - `wp` may be null (for tabline rendering).
/// - Accesses global C state (grid, highlight arrays, options).
#[no_mangle]
pub unsafe extern "C" fn rs_win_redr_custom(
    wp: WinHandle,
    draw_winbar: bool,
    draw_ruler: bool,
    ui_event: bool,
) {
    win_redr::win_redr_custom(wp, draw_winbar, draw_ruler, ui_event);
}

// =============================================================================
// Phase 6: Format Parser (build_stl_str_hl)
// =============================================================================

/// FFI export: Build a string from the status line items in "fmt".
///
/// This is the full Rust implementation of `build_stl_str_hl`. It handles
/// format parsing, VimL eval, item rendering, truncation, separator filling,
/// and highlight/click record building.
///
/// # Safety
/// - `wp` must be a valid window handle.
/// - `out` must be a valid buffer of at least `outlen` bytes.
/// - `fmt` must be a valid C string (will be modified in place).
/// - Other pointer parameters may be null where documented.
#[no_mangle]
pub unsafe extern "C" fn rs_build_stl_str_hl_wrap(
    wp: WinHandle,
    out: *mut c_char,
    outlen: usize,
    fmt: *mut c_char,
    opt_idx: c_int,
    opt_scope: c_int,
    fillchar: ScharT,
    maxwidth: c_int,
    hltab: *mut *mut c_void,
    hltab_len: *mut usize,
    tabtab: *mut *mut c_void,
    stcp: StatuscolHandle,
) -> c_int {
    stl_build::build_stl_str_hl(
        wp, out, outlen, fmt, opt_idx, opt_scope, fillchar, maxwidth, hltab, hltab_len, tabtab,
        stcp,
    )
}

// =============================================================================
// Phase 1: C-named symbol exports
// These provide the canonical C symbol names for thin-wrapper functions
// that were deleted from statusline.c.  The rs_* names are kept for any
// other Rust crates that call them directly.
// =============================================================================

/// C export: `win_redr_custom` -- replaces the deleted C static.
///
/// # Safety
/// - `wp` may be null (for tabline rendering).
#[export_name = "win_redr_custom"]
pub unsafe extern "C" fn win_redr_custom_export(
    wp: WinHandle,
    draw_winbar: bool,
    draw_ruler: bool,
    ui_event: bool,
) {
    rs_win_redr_custom(wp, draw_winbar, draw_ruler, ui_event);
}

/// C export: `build_stl_str_hl` -- replaces the deleted C thin wrapper.
///
/// # Safety
/// All pointer parameters follow the same constraints as `rs_build_stl_str_hl_wrap`.
#[allow(clippy::too_many_arguments)]
#[export_name = "build_stl_str_hl"]
pub unsafe extern "C" fn build_stl_str_hl_export(
    wp: WinHandle,
    out: *mut c_char,
    outlen: usize,
    fmt: *mut c_char,
    opt_idx: c_int,
    opt_scope: c_int,
    fillchar: ScharT,
    maxwidth: c_int,
    hltab: *mut *mut c_void,
    hltab_len: *mut usize,
    tabtab: *mut *mut c_void,
    stcp: StatuscolHandle,
) -> c_int {
    rs_build_stl_str_hl_wrap(
        wp, out, outlen, fmt, opt_idx, opt_scope, fillchar, maxwidth, hltab, hltab_len, tabtab,
        stcp,
    )
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
        assert_eq!(HLF_S, 19); // StatusLine
        assert_eq!(HLF_SNC, 20); // StatusLineNC
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

    // =========================================================================
    // Truncation and Padding Tests
    // =========================================================================

    #[test]
    fn test_apply_truncation_no_truncation() {
        let mut buf = *b"hello world";
        let result = apply_truncation_impl(&mut buf, 11, 20);
        assert_eq!(result.truncated_len, 11);
        assert!(!result.was_truncated);
    }

    #[test]
    fn test_apply_truncation_with_truncation() {
        let mut buf = *b"hello world";
        let result = apply_truncation_impl(&mut buf, 11, 5);
        assert!(result.was_truncated);
        assert_eq!(result.truncated_len, 5);
        assert_eq!(buf[0], b'<');
    }

    #[test]
    fn test_apply_truncation_empty() {
        let mut buf = [0u8; 10];
        let result = apply_truncation_impl(&mut buf, 0, 5);
        assert!(!result.was_truncated);
        assert_eq!(result.truncated_len, 0);
    }

    #[test]
    fn test_compute_padding_right_align() {
        let (padding, pad_left) = compute_padding_impl(5, 10);
        assert_eq!(padding, 5);
        assert!(pad_left);
    }

    #[test]
    fn test_compute_padding_left_align() {
        let (padding, pad_left) = compute_padding_impl(5, -10);
        assert_eq!(padding, 5);
        assert!(!pad_left);
    }

    #[test]
    fn test_compute_padding_no_padding() {
        let (padding, _) = compute_padding_impl(10, 5);
        assert_eq!(padding, 0);
    }

    #[test]
    fn test_compute_padding_zero_minwid() {
        let (padding, _) = compute_padding_impl(5, 0);
        assert_eq!(padding, 0);
    }

    #[test]
    fn test_should_skip_leading_comma() {
        // Skip comma if prevchar_isitem is 0
        assert_eq!(rs_stl_should_skip_leading(b',', 0, 0), 1);
        // Don't skip comma if prevchar_isitem is 1
        assert_eq!(rs_stl_should_skip_leading(b',', 0, 1), 0);
    }

    #[test]
    fn test_should_skip_leading_space() {
        // Skip space if prevchar_isflag is 1
        assert_eq!(rs_stl_should_skip_leading(b' ', 1, 0), 1);
        // Don't skip space if prevchar_isflag is 0
        assert_eq!(rs_stl_should_skip_leading(b' ', 0, 0), 0);
    }

    #[test]
    fn test_render_number_decimal() {
        let mut buf = [0u8; 32];
        let len = render_number_impl(&mut buf, 42, 10, 0, false);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"42");
    }

    #[test]
    fn test_render_number_hex() {
        let mut buf = [0u8; 32];
        let len = render_number_impl(&mut buf, 255, 16, 0, false);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"FF");
    }

    #[test]
    fn test_render_number_with_width() {
        let mut buf = [0u8; 32];
        let len = render_number_impl(&mut buf, 42, 10, 5, false);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"   42");
    }

    #[test]
    fn test_render_number_left_align() {
        let mut buf = [0u8; 32];
        let len = render_number_impl(&mut buf, 42, 10, -5, false);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"42   ");
    }

    #[test]
    fn test_render_number_zeropad() {
        let mut buf = [0u8; 32];
        let len = render_number_impl(&mut buf, 42, 10, 5, true);
        assert_eq!(len, 5);
        assert_eq!(&buf[..5], b"00042");
    }

    #[test]
    fn test_number_width_decimal() {
        assert_eq!(rs_stl_number_width(0, 10), 1);
        assert_eq!(rs_stl_number_width(9, 10), 1);
        assert_eq!(rs_stl_number_width(10, 10), 2);
        assert_eq!(rs_stl_number_width(99, 10), 2);
        assert_eq!(rs_stl_number_width(100, 10), 3);
        assert_eq!(rs_stl_number_width(12345, 10), 5);
    }

    #[test]
    fn test_number_width_hex() {
        assert_eq!(rs_stl_number_width(0, 16), 1);
        assert_eq!(rs_stl_number_width(15, 16), 1);
        assert_eq!(rs_stl_number_width(16, 16), 2);
        assert_eq!(rs_stl_number_width(255, 16), 2);
        assert_eq!(rs_stl_number_width(256, 16), 3);
    }

    // =========================================================================
    // Printable Character Conversion Tests
    // =========================================================================

    #[test]
    fn test_make_printable_ascii() {
        let mut buf = [0u8; 4];
        let len = make_printable_impl(&mut buf, b'A');
        assert_eq!(len, 1);
        assert_eq!(buf[0], b'A');
    }

    #[test]
    fn test_make_printable_nul() {
        let mut buf = [0u8; 4];
        let len = make_printable_impl(&mut buf, 0);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"^@");
    }

    #[test]
    fn test_make_printable_control() {
        let mut buf = [0u8; 4];
        // Ctrl-A (0x01) -> ^A
        let len = make_printable_impl(&mut buf, 0x01);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"^A");

        // Escape (0x1B) -> ^[
        let len = make_printable_impl(&mut buf, 0x1B);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"^[");
    }

    #[test]
    fn test_make_printable_del() {
        let mut buf = [0u8; 4];
        let len = make_printable_impl(&mut buf, 0x7F);
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], b"^?");
    }

    #[test]
    fn test_make_printable_high_byte() {
        let mut buf = [0u8; 4];
        let len = make_printable_impl(&mut buf, 0xFF);
        assert_eq!(len, 4);
        assert_eq!(&buf[..4], b"<FF>");

        let len = make_printable_impl(&mut buf, 0x80);
        assert_eq!(len, 4);
        assert_eq!(&buf[..4], b"<80>");
    }

    #[test]
    fn test_transstr_simple() {
        let mut out = [0u8; 32];
        let len = transstr_impl(&mut out, b"hello");
        assert_eq!(len, 5);
        assert_eq!(&out[..5], b"hello");
    }

    #[test]
    fn test_transstr_with_control() {
        let mut out = [0u8; 32];
        let len = transstr_impl(&mut out, b"a\x01b");
        assert_eq!(len, 4); // a + ^A + b
        assert_eq!(&out[..4], b"a^Ab");
    }

    #[test]
    fn test_transstr_empty() {
        let mut out = [0u8; 32];
        let len = transstr_impl(&mut out, b"");
        assert_eq!(len, 0);
    }

    // =========================================================================
    // Tabline Helper Tests
    // =========================================================================

    #[test]
    fn test_is_tab_separator() {
        assert_eq!(rs_stl_is_tab_separator(b' '), 1);
        assert_eq!(rs_stl_is_tab_separator(b'|'), 1);
        assert_eq!(rs_stl_is_tab_separator(b'a'), 0);
        assert_eq!(rs_stl_is_tab_separator(b'\t'), 0);
    }

    #[test]
    fn test_wincount_width() {
        assert_eq!(rs_stl_wincount_width(0), 0);
        assert_eq!(rs_stl_wincount_width(1), 0);
        assert_eq!(rs_stl_wincount_width(2), 1);
        assert_eq!(rs_stl_wincount_width(9), 1);
        assert_eq!(rs_stl_wincount_width(10), 2);
        assert_eq!(rs_stl_wincount_width(100), 3);
    }
}
