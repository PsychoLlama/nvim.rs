//! UI integration for statusline components
//!
//! This module provides abstractions for grid drawing operations and UI event
//! emission used by statusline, winbar, ruler, and tabline rendering.

use std::ffi::c_int;

use nvim_window::WinHandle;

// C grid functions
extern "C" {
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;
}

/// Grid cell representation for statusline drawing.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GridCell {
    /// The character to display (schar_T format)
    pub schar: u32,
    /// Highlight attribute
    pub attr: c_int,
}

impl GridCell {
    /// Create a new grid cell with ASCII character.
    #[allow(clippy::cast_lossless)]
    pub const fn ascii(c: u8, attr: c_int) -> Self {
        Self {
            schar: c as u32,
            attr,
        }
    }

    /// Create a new grid cell with fill character.
    pub const fn fill(schar: u32, attr: c_int) -> Self {
        Self { schar, attr }
    }

    /// Create a space cell with given attribute.
    #[allow(clippy::cast_lossless)]
    pub const fn space(attr: c_int) -> Self {
        Self {
            schar: b' ' as u32,
            attr,
        }
    }
}

impl Default for GridCell {
    fn default() -> Self {
        Self::space(0)
    }
}

/// Grid span for batch operations.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GridSpan {
    /// Starting column
    pub start_col: c_int,
    /// Ending column (exclusive)
    pub end_col: c_int,
    /// Fill character
    pub fill: u32,
    /// Highlight attribute
    pub attr: c_int,
}

impl GridSpan {
    /// Create a new grid span.
    pub const fn new(start_col: c_int, end_col: c_int, fill: u32, attr: c_int) -> Self {
        Self {
            start_col,
            end_col,
            fill,
            attr,
        }
    }

    /// Get the width of the span.
    pub const fn width(&self) -> c_int {
        self.end_col - self.start_col
    }

    /// Check if the span is empty.
    pub const fn is_empty(&self) -> bool {
        self.end_col <= self.start_col
    }
}

/// UI highlight attributes.
///
/// These match the HLF_* constants from highlight_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiHighlight {
    /// StatusLine (current window)
    StatusLine = 27,
    /// StatusLineNC (non-current window)
    StatusLineNC = 28,
    /// WinBar (current window)
    WinBar = 37,
    /// WinBarNC (non-current window)
    WinBarNC = 38,
    /// TabLine (unselected tabs)
    TabLine = 39,
    /// TabLineSel (selected tab)
    TabLineSel = 40,
    /// TabLineFill (empty space)
    TabLineFill = 41,
    /// Message area
    MsgArea = 11,
}

impl UiHighlight {
    /// Get the highlight group ID for statusline based on window state.
    pub const fn for_statusline(is_curwin: bool) -> Self {
        if is_curwin {
            Self::StatusLine
        } else {
            Self::StatusLineNC
        }
    }

    /// Get the highlight group ID for winbar based on window state.
    pub const fn for_winbar(is_curwin: bool) -> Self {
        if is_curwin {
            Self::WinBar
        } else {
            Self::WinBarNC
        }
    }

    /// Get the highlight group ID for tabline based on selection state.
    pub const fn for_tabline(is_selected: bool) -> Self {
        if is_selected {
            Self::TabLineSel
        } else {
            Self::TabLine
        }
    }
}

/// UI event types for statusline components.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEventType {
    /// Ruler update for external UI
    MsgRuler = 0,
    /// Tabline update for external UI
    TablineUpdate = 1,
}

/// Content chunk for UI events.
///
/// Used in msg_ruler and similar events that send styled content.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct UiContentChunk {
    /// Highlight attribute
    pub attr: c_int,
    /// Content text (UTF-8)
    pub content: Vec<u8>,
    /// Highlight group ID
    pub hl_group: c_int,
}

impl UiContentChunk {
    /// Create a new content chunk.
    pub fn new(attr: c_int, content: &str, hl_group: c_int) -> Self {
        Self {
            attr,
            content: content.as_bytes().to_vec(),
            hl_group,
        }
    }

    /// Check if the chunk is empty.
    pub const fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// Builder for UI content arrays.
#[derive(Debug, Default)]
pub struct UiContentBuilder {
    chunks: Vec<UiContentChunk>,
}

impl UiContentBuilder {
    /// Create a new content builder.
    pub const fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Add a content chunk.
    pub fn add_chunk(&mut self, attr: c_int, content: &str, hl_group: c_int) {
        if !content.is_empty() {
            self.chunks.push(UiContentChunk::new(attr, content, hl_group));
        }
    }

    /// Get the number of chunks.
    pub const fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Check if empty.
    pub const fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Clear all chunks.
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Consume and return the chunks.
    pub fn take(self) -> Vec<UiContentChunk> {
        self.chunks
    }
}

/// Check if window is the current window.
pub fn is_current_window(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    unsafe { nvim_win_is_curwin(wp) != 0 }
}

/// Calculate display row for a window's statusline.
///
/// Takes into account global statusline settings.
pub const fn calc_statusline_row(
    win_row: c_int,
    win_height: c_int,
    global_stl_height: c_int,
    is_global: bool,
    rows: c_int,
    cmdline_row: c_int,
) -> c_int {
    if is_global {
        rows - cmdline_row - 1
    } else if global_stl_height > 0 {
        win_row + win_height - 1
    } else {
        win_row + win_height
    }
}

/// Calculate display row for a window's winbar.
///
/// The winbar is always at the top of the window.
pub const fn calc_winbar_row(win_row: c_int) -> c_int {
    win_row
}

/// Calculate display row for the ruler.
///
/// When not in statusline, ruler is on the last row (message area).
pub const fn calc_ruler_row(rows: c_int, in_statusline: bool, win_row: c_int) -> c_int {
    if in_statusline {
        win_row
    } else {
        rows - 1
    }
}

/// Calculate available width for statusline.
pub const fn calc_statusline_width(
    win_width: c_int,
    is_global: bool,
    columns: c_int,
) -> c_int {
    if is_global {
        columns
    } else {
        win_width
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Create a grid cell from ASCII character.
#[no_mangle]
#[allow(clippy::cast_lossless)]
pub const extern "C" fn rs_ui_grid_cell_ascii(c: u8, attr: c_int) -> GridCell {
    GridCell::ascii(c, attr)
}

/// FFI export: Create a grid cell from fill character.
#[no_mangle]
pub const extern "C" fn rs_ui_grid_cell_fill(schar: u32, attr: c_int) -> GridCell {
    GridCell::fill(schar, attr)
}

/// FFI export: Create a space grid cell.
#[no_mangle]
pub const extern "C" fn rs_ui_grid_cell_space(attr: c_int) -> GridCell {
    GridCell::space(attr)
}

/// FFI export: Create a grid span.
#[no_mangle]
pub const extern "C" fn rs_ui_grid_span_new(
    start_col: c_int,
    end_col: c_int,
    fill: u32,
    attr: c_int,
) -> GridSpan {
    GridSpan::new(start_col, end_col, fill, attr)
}

/// FFI export: Get grid span width.
#[no_mangle]
pub const extern "C" fn rs_ui_grid_span_width(span: &GridSpan) -> c_int {
    span.width()
}

/// FFI export: Get highlight for statusline.
#[no_mangle]
pub extern "C" fn rs_ui_hl_statusline(wp: WinHandle) -> c_int {
    let is_curwin = is_current_window(wp);
    UiHighlight::for_statusline(is_curwin) as c_int
}

/// FFI export: Get highlight for winbar.
#[no_mangle]
pub extern "C" fn rs_ui_hl_winbar(wp: WinHandle) -> c_int {
    let is_curwin = is_current_window(wp);
    UiHighlight::for_winbar(is_curwin) as c_int
}

/// FFI export: Get highlight for tabline.
#[no_mangle]
pub const extern "C" fn rs_ui_hl_tabline(is_selected: c_int) -> c_int {
    UiHighlight::for_tabline(is_selected != 0) as c_int
}

/// FFI export: Get highlight for tabline fill.
#[no_mangle]
pub const extern "C" fn rs_ui_hl_tabline_fill() -> c_int {
    UiHighlight::TabLineFill as c_int
}

/// FFI export: Get highlight for message area.
#[no_mangle]
pub const extern "C" fn rs_ui_hl_msg_area() -> c_int {
    UiHighlight::MsgArea as c_int
}

/// FFI export: Calculate statusline row.
#[no_mangle]
pub const extern "C" fn rs_ui_calc_statusline_row(
    win_row: c_int,
    win_height: c_int,
    global_stl_height: c_int,
    is_global: c_int,
    rows: c_int,
    cmdline_row: c_int,
) -> c_int {
    calc_statusline_row(
        win_row,
        win_height,
        global_stl_height,
        is_global != 0,
        rows,
        cmdline_row,
    )
}

/// FFI export: Calculate winbar row.
#[no_mangle]
pub const extern "C" fn rs_ui_calc_winbar_row(win_row: c_int) -> c_int {
    calc_winbar_row(win_row)
}

/// FFI export: Calculate ruler row.
#[no_mangle]
pub const extern "C" fn rs_ui_calc_ruler_row(
    rows: c_int,
    in_statusline: c_int,
    win_row: c_int,
) -> c_int {
    calc_ruler_row(rows, in_statusline != 0, win_row)
}

/// FFI export: Calculate statusline width.
#[no_mangle]
pub const extern "C" fn rs_ui_calc_statusline_width(
    win_width: c_int,
    is_global: c_int,
    columns: c_int,
) -> c_int {
    calc_statusline_width(win_width, is_global != 0, columns)
}

/// FFI export: Check if window is current window.
#[no_mangle]
pub extern "C" fn rs_ui_is_curwin(wp: WinHandle) -> c_int {
    c_int::from(is_current_window(wp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_cell_ascii() {
        let cell = GridCell::ascii(b'A', 5);
        assert_eq!(cell.schar, u32::from(b'A'));
        assert_eq!(cell.attr, 5);
    }

    #[test]
    fn test_grid_cell_space() {
        let cell = GridCell::space(10);
        assert_eq!(cell.schar, u32::from(b' '));
        assert_eq!(cell.attr, 10);
    }

    #[test]
    fn test_grid_cell_fill() {
        let cell = GridCell::fill(0x2500, 3); // Box-drawing character
        assert_eq!(cell.schar, 0x2500);
        assert_eq!(cell.attr, 3);
    }

    #[test]
    fn test_grid_cell_default() {
        let cell = GridCell::default();
        assert_eq!(cell.schar, u32::from(b' '));
        assert_eq!(cell.attr, 0);
    }

    #[test]
    fn test_grid_span_new() {
        let span = GridSpan::new(5, 15, u32::from(b'-'), 2);
        assert_eq!(span.start_col, 5);
        assert_eq!(span.end_col, 15);
        assert_eq!(span.fill, u32::from(b'-'));
        assert_eq!(span.attr, 2);
    }

    #[test]
    fn test_grid_span_width() {
        let span = GridSpan::new(5, 15, 0, 0);
        assert_eq!(span.width(), 10);
    }

    #[test]
    fn test_grid_span_empty() {
        let span = GridSpan::new(10, 10, 0, 0);
        assert!(span.is_empty());

        let span = GridSpan::new(15, 10, 0, 0);
        assert!(span.is_empty());

        let span = GridSpan::new(5, 10, 0, 0);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_ui_highlight_values() {
        assert_eq!(UiHighlight::StatusLine as c_int, 27);
        assert_eq!(UiHighlight::StatusLineNC as c_int, 28);
        assert_eq!(UiHighlight::WinBar as c_int, 37);
        assert_eq!(UiHighlight::WinBarNC as c_int, 38);
        assert_eq!(UiHighlight::TabLine as c_int, 39);
        assert_eq!(UiHighlight::TabLineSel as c_int, 40);
        assert_eq!(UiHighlight::TabLineFill as c_int, 41);
        assert_eq!(UiHighlight::MsgArea as c_int, 11);
    }

    #[test]
    fn test_ui_highlight_for_statusline() {
        assert_eq!(UiHighlight::for_statusline(true), UiHighlight::StatusLine);
        assert_eq!(UiHighlight::for_statusline(false), UiHighlight::StatusLineNC);
    }

    #[test]
    fn test_ui_highlight_for_winbar() {
        assert_eq!(UiHighlight::for_winbar(true), UiHighlight::WinBar);
        assert_eq!(UiHighlight::for_winbar(false), UiHighlight::WinBarNC);
    }

    #[test]
    fn test_ui_highlight_for_tabline() {
        assert_eq!(UiHighlight::for_tabline(true), UiHighlight::TabLineSel);
        assert_eq!(UiHighlight::for_tabline(false), UiHighlight::TabLine);
    }

    #[test]
    fn test_ui_content_chunk() {
        let chunk = UiContentChunk::new(5, "hello", 10);
        assert_eq!(chunk.attr, 5);
        assert_eq!(chunk.content, b"hello");
        assert_eq!(chunk.hl_group, 10);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn test_ui_content_chunk_empty() {
        let chunk = UiContentChunk::new(0, "", 0);
        assert!(chunk.is_empty());
    }

    #[test]
    fn test_ui_content_builder() {
        let mut builder = UiContentBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);

        builder.add_chunk(1, "hello", 10);
        builder.add_chunk(2, "world", 20);
        assert!(!builder.is_empty());
        assert_eq!(builder.len(), 2);

        // Empty strings are not added
        builder.add_chunk(3, "", 30);
        assert_eq!(builder.len(), 2);

        let chunks = builder.take();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content, b"hello");
        assert_eq!(chunks[1].content, b"world");
    }

    #[test]
    fn test_ui_content_builder_clear() {
        let mut builder = UiContentBuilder::new();
        builder.add_chunk(1, "test", 5);
        assert_eq!(builder.len(), 1);

        builder.clear();
        assert!(builder.is_empty());
    }

    #[test]
    fn test_calc_statusline_row_global() {
        // Global statusline at Rows - cmdline_row - 1
        let row = calc_statusline_row(0, 20, 1, true, 50, 1);
        assert_eq!(row, 48); // 50 - 1 - 1
    }

    #[test]
    fn test_calc_statusline_row_window() {
        // Window statusline at win_row + win_height
        let row = calc_statusline_row(5, 20, 0, false, 50, 1);
        assert_eq!(row, 25); // 5 + 20
    }

    #[test]
    fn test_calc_statusline_row_with_global_height() {
        // With global statusline height, subtract 1
        let row = calc_statusline_row(5, 20, 1, false, 50, 1);
        assert_eq!(row, 24); // 5 + 20 - 1
    }

    #[test]
    fn test_calc_winbar_row() {
        // Winbar is always at win_row
        assert_eq!(calc_winbar_row(5), 5);
        assert_eq!(calc_winbar_row(0), 0);
        assert_eq!(calc_winbar_row(10), 10);
    }

    #[test]
    fn test_calc_ruler_row() {
        // In statusline uses win_row
        assert_eq!(calc_ruler_row(50, true, 10), 10);
        // Not in statusline uses Rows - 1
        assert_eq!(calc_ruler_row(50, false, 10), 49);
    }

    #[test]
    fn test_calc_statusline_width() {
        // Global uses Columns
        assert_eq!(calc_statusline_width(80, true, 120), 120);
        // Non-global uses win_width
        assert_eq!(calc_statusline_width(80, false, 120), 80);
    }
}
