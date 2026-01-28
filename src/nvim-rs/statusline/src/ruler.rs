//! Ruler rendering for statusline
//!
//! This module provides ruler rendering utilities. The ruler shows
//! cursor position information (line, column, percentage) in the
//! statusline or command line.

use std::ffi::c_int;
use std::io::Write;

/// Ruler format options.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RulerOptions {
    /// Whether to show the ruler
    pub show: bool,
    /// Maximum width for the ruler
    pub max_width: c_int,
    /// Whether to show line number
    pub show_line: bool,
    /// Whether to show column number
    pub show_col: bool,
    /// Whether to show virtual column
    pub show_virtcol: bool,
    /// Whether to show percentage
    pub show_percent: bool,
}

impl Default for RulerOptions {
    fn default() -> Self {
        Self {
            show: true,
            max_width: 17, // Default ruler width in Vim
            show_line: true,
            show_col: true,
            show_virtcol: true,
            show_percent: true,
        }
    }
}

/// Ruler context for rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RulerContext {
    /// Current line number (1-based)
    pub lnum: c_int,
    /// Total line count
    pub line_count: c_int,
    /// Column number (1-based)
    pub col: c_int,
    /// Virtual column (1-based)
    pub virtcol: c_int,
    /// Whether the line is empty
    pub empty_line: bool,
}

impl Default for RulerContext {
    fn default() -> Self {
        Self {
            lnum: 1,
            line_count: 1,
            col: 1,
            virtcol: 1,
            empty_line: false,
        }
    }
}

impl RulerContext {
    /// Create a new ruler context.
    pub const fn new(lnum: c_int, line_count: c_int, col: c_int, virtcol: c_int) -> Self {
        Self {
            lnum,
            line_count,
            col,
            virtcol,
            empty_line: false,
        }
    }

    /// Calculate percentage position.
    #[allow(clippy::cast_possible_truncation)]
    pub fn percentage(&self) -> c_int {
        if self.line_count == 0 {
            return 0;
        }
        let lnum_i64 = i64::from(self.lnum);
        let count_i64 = i64::from(self.line_count);
        let result = ((lnum_i64 * 100) + (count_i64 / 2)) / count_i64;
        result.clamp(0, 100) as c_int
    }
}

/// Render the ruler string.
///
/// Format: `{line},{col}[-{virtcol}]  {percent}%` or position indicator.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn render_ruler(buf: &mut [u8], ctx: &RulerContext, opts: &RulerOptions) -> c_int {
    if buf.is_empty() || !opts.show {
        return 0;
    }

    let mut cursor = std::io::Cursor::new(buf);

    // Line and column
    if opts.show_line && opts.show_col {
        let col = if ctx.empty_line { 0 } else { ctx.col };
        let _ = write!(cursor, "{},{}", ctx.lnum, col);

        // Virtual column if different
        if opts.show_virtcol && ctx.virtcol != ctx.col {
            let _ = write!(cursor, "-{}", ctx.virtcol);
        }
    } else if opts.show_line {
        let _ = write!(cursor, "{}", ctx.lnum);
    } else if opts.show_col {
        let col = if ctx.empty_line { 0 } else { ctx.col };
        let _ = write!(cursor, "{col}");
    }

    // Separator and percentage
    if opts.show_percent {
        // Add spacing
        let _ = write!(cursor, "  ");

        // Position indicator
        let pos_str = get_position_string(ctx.lnum, ctx.line_count);
        let _ = write!(cursor, "{pos_str}");
    }

    cursor.position() as c_int
}

/// Get position string (Top/Bot/All/percentage).
pub fn get_position_string(lnum: c_int, line_count: c_int) -> String {
    if line_count <= 0 {
        return "Top".to_string();
    }

    // Calculate percentage
    let lnum_i64 = i64::from(lnum);
    let count_i64 = i64::from(line_count);

    if lnum == 1 {
        if line_count == 1 {
            "All".to_string()
        } else {
            "Top".to_string()
        }
    } else if lnum >= line_count {
        "Bot".to_string()
    } else {
        #[allow(clippy::cast_possible_truncation)]
        let pct = ((lnum_i64 * 100) / count_i64) as c_int;
        format!("{pct}%")
    }
}

/// Render a minimal ruler for tight spaces.
///
/// Format: `{line}:{col}` or just `{line}`.
#[allow(clippy::cast_possible_truncation)]
pub fn render_ruler_minimal(buf: &mut [u8], ctx: &RulerContext) -> c_int {
    if buf.is_empty() {
        return 0;
    }

    let mut cursor = std::io::Cursor::new(buf);
    let col = if ctx.empty_line { 0 } else { ctx.col };
    let _ = write!(cursor, "{}:{}", ctx.lnum, col);

    cursor.position() as c_int
}

/// Calculate the width needed for the ruler.
pub const fn calc_ruler_width(ctx: &RulerContext, opts: &RulerOptions) -> c_int {
    if !opts.show {
        return 0;
    }

    let mut width = 0;

    // Line number width
    if opts.show_line {
        width += count_digits(ctx.lnum);
    }

    // Separator
    if opts.show_line && opts.show_col {
        width += 1; // comma
    }

    // Column width
    if opts.show_col {
        let col = if ctx.empty_line { 0 } else { ctx.col };
        width += count_digits(col);

        // Virtual column if different
        if opts.show_virtcol && ctx.virtcol != ctx.col {
            width += 1 + count_digits(ctx.virtcol); // dash + virtcol
        }
    }

    // Percentage
    if opts.show_percent {
        width += 2; // spacing
        width += 3; // "Top", "Bot", "All", or "NN%"
    }

    width
}

/// Count the number of digits in a number.
const fn count_digits(n: c_int) -> c_int {
    if n <= 0 {
        return 1;
    }
    let mut digits = 0;
    let mut val = n;
    while val > 0 {
        val /= 10;
        digits += 1;
    }
    digits
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Render ruler to buffer.
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
/// `wp` must be a valid window handle or null.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_ruler_render(
    buf: *mut u8,
    buflen: usize,
    lnum: c_int,
    line_count: c_int,
    col: c_int,
    virtcol: c_int,
    empty_line: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    let ctx = RulerContext {
        lnum,
        line_count,
        col,
        virtcol,
        empty_line: empty_line != 0,
    };
    let opts = RulerOptions::default();
    let slice = std::slice::from_raw_parts_mut(buf, buflen);
    render_ruler(slice, &ctx, &opts)
}

/// FFI export: Get position string (Top/Bot/All/percentage).
///
/// # Safety
/// `buf` must be null or a valid pointer to a buffer of at least `buflen` bytes.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_ruler_get_position(
    buf: *mut u8,
    buflen: usize,
    lnum: c_int,
    line_count: c_int,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    let pos = get_position_string(lnum, line_count);
    let bytes = pos.as_bytes();
    let len = bytes.len().min(buflen);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
    len as c_int
}

/// FFI export: Calculate ruler width.
#[no_mangle]
pub extern "C" fn rs_ruler_calc_width(
    lnum: c_int,
    line_count: c_int,
    col: c_int,
    virtcol: c_int,
    empty_line: c_int,
) -> c_int {
    let ctx = RulerContext {
        lnum,
        line_count,
        col,
        virtcol,
        empty_line: empty_line != 0,
    };
    let opts = RulerOptions::default();
    calc_ruler_width(&ctx, &opts)
}

// =============================================================================
// Ruler Redraw State Machine
// =============================================================================

/// Represents the decision of what to do for ruler redraw.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerRedrawAction {
    /// Don't redraw - ruler is disabled or covered
    None = 0,
    /// Clear the ruler (it was drawn but shouldn't be now)
    Clear = 1,
    /// Use custom rulerformat
    UseRulerformat = 2,
    /// Draw the standard ruler
    DrawStandard = 3,
    /// Skip - editing in submode, would overwrite mode message
    Skip = 4,
}

/// Context for deciding ruler redraw action.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RulerRedrawContext {
    /// p_ru - ruler option
    pub show_ruler: c_int,
    /// curwin->w_status_height == 0
    pub curwin_no_status: c_int,
    /// lastwin_nofloating()->w_status_height > 0
    pub lastwin_has_status: c_int,
    /// global_stl_height() > 0
    pub is_stl_global: c_int,
    /// p_ch == 0
    pub cmdheight_zero: c_int,
    /// ui_has(kUIMessages)
    pub ui_has_messages: c_int,
    /// Previous did_ruler_col value (-1 if not drawn)
    pub did_ruler_col: c_int,
    /// cursor.lnum > line_count
    pub cursor_invalid: c_int,
    /// edit_submode != NULL
    pub in_edit_submode: c_int,
    /// *p_ruf (rulerformat is set)
    pub has_rulerformat: c_int,
}

impl Default for RulerRedrawContext {
    fn default() -> Self {
        Self {
            show_ruler: 0,
            curwin_no_status: 0,
            lastwin_has_status: 0,
            is_stl_global: 0,
            cmdheight_zero: 0,
            ui_has_messages: 0,
            did_ruler_col: -1,
            cursor_invalid: 0,
            in_edit_submode: 0,
            has_rulerformat: 0,
        }
    }
}

/// Determine what ruler redraw action to take.
///
/// This encapsulates the complex condition checking from `redraw_ruler()`.
pub const fn decide_ruler_action(ctx: &RulerRedrawContext) -> RulerRedrawAction {
    let is_stl_global = ctx.is_stl_global != 0;
    let lastwin_has_status = ctx.lastwin_has_status != 0;

    // Check if ruler should be drawn
    let ruler_disabled = ctx.show_ruler == 0
        || lastwin_has_status
        || is_stl_global
        || (ctx.cmdheight_zero != 0 && ctx.ui_has_messages == 0);

    if ruler_disabled {
        // Ruler is disabled - check if we need to clear it
        if ctx.did_ruler_col > 0 {
            return RulerRedrawAction::Clear;
        }
        return RulerRedrawAction::None;
    }

    // Check if cursor position is valid
    if ctx.cursor_invalid != 0 {
        return RulerRedrawAction::None;
    }

    // Don't draw ruler while doing insert-completion (might overwrite mode message)
    let curwin_no_status = ctx.curwin_no_status != 0;
    if curwin_no_status && !is_stl_global && ctx.in_edit_submode != 0 {
        return RulerRedrawAction::Skip;
    }

    // Check if using rulerformat
    let part_of_status = lastwin_has_status || is_stl_global;
    if ctx.has_rulerformat != 0
        && (ctx.cmdheight_zero == 0 || (ctx.ui_has_messages != 0 && !part_of_status))
    {
        return RulerRedrawAction::UseRulerformat;
    }

    // Draw standard ruler
    RulerRedrawAction::DrawStandard
}

/// FFI export: Decide ruler redraw action.
#[no_mangle]
pub const extern "C" fn rs_ruler_decide_action(ctx: &RulerRedrawContext) -> c_int {
    decide_ruler_action(ctx) as c_int
}

/// FFI export: Check if ruler should use external UI.
#[no_mangle]
pub extern "C" fn rs_ruler_use_ext_ui(ui_has_messages: c_int, part_of_status: c_int) -> c_int {
    c_int::from(ui_has_messages != 0 && part_of_status == 0)
}

/// Calculate ruler column position.
///
/// Returns the column where the ruler should start.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub const fn calc_ruler_col(
    ru_col: c_int,
    columns: c_int,
    width: c_int,
    content_width: c_int,
) -> c_int {
    let this_ru_col = ru_col - (columns - width);

    // Never use more than half the window/screen width
    let min_col = (width + 1) / 2;
    let adjusted = if this_ru_col < min_col {
        min_col
    } else {
        this_ru_col
    };

    // Make sure content fits
    if adjusted + content_width > width {
        width - content_width
    } else {
        adjusted
    }
}

/// FFI export: Calculate ruler column position.
#[no_mangle]
pub const extern "C" fn rs_ruler_calc_col(
    ru_col: c_int,
    columns: c_int,
    width: c_int,
    content_width: c_int,
) -> c_int {
    calc_ruler_col(ru_col, columns, width, content_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruler_redraw_action_none() {
        let ctx = RulerRedrawContext {
            show_ruler: 0,
            did_ruler_col: -1,
            ..Default::default()
        };
        assert_eq!(decide_ruler_action(&ctx), RulerRedrawAction::None);
    }

    #[test]
    fn test_ruler_redraw_action_clear() {
        let ctx = RulerRedrawContext {
            show_ruler: 0,
            did_ruler_col: 10,
            ..Default::default()
        };
        assert_eq!(decide_ruler_action(&ctx), RulerRedrawAction::Clear);
    }

    #[test]
    fn test_ruler_redraw_action_draw_standard() {
        let ctx = RulerRedrawContext {
            show_ruler: 1,
            curwin_no_status: 1,
            lastwin_has_status: 0,
            is_stl_global: 0,
            cmdheight_zero: 0,
            ui_has_messages: 0,
            did_ruler_col: -1,
            cursor_invalid: 0,
            in_edit_submode: 0,
            has_rulerformat: 0,
        };
        assert_eq!(decide_ruler_action(&ctx), RulerRedrawAction::DrawStandard);
    }

    #[test]
    fn test_ruler_redraw_action_use_rulerformat() {
        let ctx = RulerRedrawContext {
            show_ruler: 1,
            curwin_no_status: 1,
            lastwin_has_status: 0,
            is_stl_global: 0,
            cmdheight_zero: 0,
            ui_has_messages: 0,
            did_ruler_col: -1,
            cursor_invalid: 0,
            in_edit_submode: 0,
            has_rulerformat: 1,
        };
        assert_eq!(decide_ruler_action(&ctx), RulerRedrawAction::UseRulerformat);
    }

    #[test]
    fn test_ruler_redraw_action_skip_submode() {
        let ctx = RulerRedrawContext {
            show_ruler: 1,
            curwin_no_status: 1,
            lastwin_has_status: 0,
            is_stl_global: 0,
            cmdheight_zero: 0,
            ui_has_messages: 0,
            did_ruler_col: -1,
            cursor_invalid: 0,
            in_edit_submode: 1,
            has_rulerformat: 0,
        };
        assert_eq!(decide_ruler_action(&ctx), RulerRedrawAction::Skip);
    }

    #[test]
    fn test_calc_ruler_col() {
        // Basic case
        let col = calc_ruler_col(17, 80, 80, 10);
        assert!(col >= 40); // At least half width
        assert!(col + 10 <= 80); // Content fits
    }

    #[test]
    fn test_calc_ruler_col_narrow_window() {
        // Content almost fills half
        let col = calc_ruler_col(17, 80, 40, 15);
        assert!(col >= 20); // At least half of 40
        assert!(col + 15 <= 40);
    }

    #[test]
    fn test_ruler_context_percentage() {
        let ctx = RulerContext::new(50, 100, 1, 1);
        assert_eq!(ctx.percentage(), 50);

        let ctx = RulerContext::new(1, 100, 1, 1);
        assert_eq!(ctx.percentage(), 1);

        let ctx = RulerContext::new(100, 100, 1, 1);
        assert_eq!(ctx.percentage(), 100);

        let ctx = RulerContext::new(1, 3, 1, 1);
        assert_eq!(ctx.percentage(), 33);
    }

    #[test]
    fn test_get_position_string_top() {
        assert_eq!(get_position_string(1, 100), "Top");
    }

    #[test]
    fn test_get_position_string_bot() {
        assert_eq!(get_position_string(100, 100), "Bot");
    }

    #[test]
    fn test_get_position_string_all() {
        assert_eq!(get_position_string(1, 1), "All");
    }

    #[test]
    fn test_get_position_string_percentage() {
        assert_eq!(get_position_string(50, 100), "50%");
        assert_eq!(get_position_string(25, 100), "25%");
    }

    #[test]
    fn test_count_digits() {
        assert_eq!(count_digits(0), 1);
        assert_eq!(count_digits(1), 1);
        assert_eq!(count_digits(9), 1);
        assert_eq!(count_digits(10), 2);
        assert_eq!(count_digits(99), 2);
        assert_eq!(count_digits(100), 3);
        assert_eq!(count_digits(1000), 4);
    }

    #[test]
    fn test_render_ruler() {
        let ctx = RulerContext::new(42, 100, 10, 15);
        let opts = RulerOptions::default();
        let mut buf = [0u8; 64];
        let len = render_ruler(&mut buf, &ctx, &opts);

        assert!(len > 0);
        #[allow(clippy::cast_sign_loss)]
        let result = std::str::from_utf8(&buf[..len as usize]).unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("10"));
        assert!(result.contains("-15")); // virtcol different from col
    }

    #[test]
    fn test_render_ruler_same_col() {
        let ctx = RulerContext::new(42, 100, 10, 10);
        let opts = RulerOptions::default();
        let mut buf = [0u8; 64];
        let len = render_ruler(&mut buf, &ctx, &opts);

        #[allow(clippy::cast_sign_loss)]
        let result = std::str::from_utf8(&buf[..len as usize]).unwrap();
        assert!(result.contains("42,10"));
        assert!(!result.contains('-')); // No virtcol difference
    }

    #[test]
    fn test_render_ruler_minimal() {
        let ctx = RulerContext::new(42, 100, 10, 15);
        let mut buf = [0u8; 64];
        let len = render_ruler_minimal(&mut buf, &ctx);

        #[allow(clippy::cast_sign_loss)]
        let result = std::str::from_utf8(&buf[..len as usize]).unwrap();
        assert_eq!(result, "42:10");
    }

    #[test]
    fn test_calc_ruler_width() {
        let ctx = RulerContext::new(42, 100, 10, 10);
        let opts = RulerOptions::default();
        let width = calc_ruler_width(&ctx, &opts);

        // "42,10  42%" - approx 10 chars
        assert!(width > 5);
        assert!(width < 20);
    }

    #[test]
    fn test_ruler_options_default() {
        let opts = RulerOptions::default();
        assert!(opts.show);
        assert!(opts.show_line);
        assert!(opts.show_col);
        assert!(opts.show_virtcol);
        assert!(opts.show_percent);
        assert_eq!(opts.max_width, 17);
    }
}
