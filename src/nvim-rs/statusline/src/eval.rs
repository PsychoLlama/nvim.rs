//! Item evaluation for statusline
//!
//! This module provides evaluation of statusline format items.
//! Each item type can produce a string or numeric value.

use std::ffi::{c_char, c_int, CStr};

use nvim_window::{BufHandle, WinHandle};

use crate::format::StlFlag;

/// Numeric base for formatting numbers.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberBase {
    /// Decimal (base 10)
    Decimal = 10,
    /// Hexadecimal (base 16)
    Hexadecimal = 16,
}

/// Result of evaluating a statusline item.
#[derive(Debug, Clone)]
pub enum EvalResult {
    /// Empty result (item not applicable)
    Empty,
    /// String result
    String(String),
    /// Numeric result
    Number { value: c_int, base: NumberBase },
}

impl EvalResult {
    /// Check if the result is empty.
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Get the string value, or None if not a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the numeric value, or None if not a number.
    pub const fn as_number(&self) -> Option<(c_int, NumberBase)> {
        match self {
            Self::Number { value, base } => Some((*value, *base)),
            _ => None,
        }
    }
}

// C accessor functions for buffer/window state
extern "C" {
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> c_int;
    fn nvim_win_buf_line_count(wp: WinHandle) -> c_int;
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ft(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_p_ma(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_buf_get_help(buf: BufHandle) -> c_int;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_win_get_pvw(wp: WinHandle) -> c_int;
}

/// Context for evaluating statusline items.
pub struct EvalContext {
    /// Window handle
    pub wp: WinHandle,
    /// Buffer handle (cached from window)
    pub buf: BufHandle,
    /// Current line number
    pub lnum: c_int,
    /// Total line count
    pub line_count: c_int,
    /// Cursor column (1-based)
    pub col: c_int,
    /// Virtual column (1-based)
    pub virtcol: c_int,
    /// Whether the line is empty
    pub empty_line: bool,
    /// Current mode flags (for insert mode check)
    pub mode_insert: bool,
}

impl EvalContext {
    /// Create a new evaluation context for the given window.
    ///
    /// # Safety
    /// `wp` must be a valid window handle.
    pub unsafe fn new(wp: WinHandle, mode_insert: bool) -> Self {
        let buf = nvim_win_get_buffer(wp);
        let lnum = nvim_win_get_cursor_lnum(wp);
        let line_count = nvim_win_buf_line_count(wp);
        let virtcol = nvim_win_get_virtcol(wp) + 1;

        Self {
            wp,
            buf,
            lnum,
            line_count,
            col: 1, // Column needs to be obtained separately
            virtcol,
            empty_line: false, // Would need to check line content
            mode_insert,
        }
    }

    /// Create a context for testing without FFI.
    #[cfg(test)]
    pub const fn test_context() -> Self {
        Self {
            wp: WinHandle::null(),
            buf: BufHandle::null(),
            lnum: 1,
            line_count: 100,
            col: 1,
            virtcol: 1,
            empty_line: false,
            mode_insert: false,
        }
    }
}

/// Evaluate a statusline flag item.
pub fn eval_flag(flag: StlFlag, ctx: &EvalContext) -> EvalResult {
    match flag {
        // Filename items
        StlFlag::FilePath | StlFlag::FullPath | StlFlag::Filename => {
            eval_filename(flag, ctx.buf, ctx.wp)
        }

        // Position items
        StlFlag::Line => EvalResult::Number {
            value: if ctx.line_count == 0 { 0 } else { ctx.lnum },
            base: NumberBase::Decimal,
        },
        StlFlag::NumLines => EvalResult::Number {
            value: ctx.line_count,
            base: NumberBase::Decimal,
        },
        StlFlag::Column => EvalResult::Number {
            value: if !ctx.mode_insert && ctx.empty_line {
                0
            } else {
                ctx.col
            },
            base: NumberBase::Decimal,
        },
        StlFlag::VirtCol => EvalResult::Number {
            value: ctx.virtcol,
            base: NumberBase::Decimal,
        },
        StlFlag::VirtColAlt => {
            // Only show if different from column
            if ctx.virtcol == ctx.col || (!ctx.mode_insert && ctx.empty_line && ctx.virtcol == 0) {
                EvalResult::Empty
            } else {
                EvalResult::Number {
                    value: ctx.virtcol,
                    base: NumberBase::Decimal,
                }
            }
        }
        StlFlag::Percentage => EvalResult::Number {
            value: calc_percentage(ctx.lnum, ctx.line_count),
            base: NumberBase::Decimal,
        },
        StlFlag::AltPercent => {
            let pos = get_rel_pos_string(ctx.wp);
            EvalResult::String(pos)
        }

        // Buffer number
        StlFlag::BufNo => unsafe {
            EvalResult::Number {
                value: nvim_buf_get_fnum(ctx.buf),
                base: NumberBase::Decimal,
            }
        },

        // Flag items
        StlFlag::RoFlag | StlFlag::RoFlagAlt => eval_readonly(flag, ctx.buf),
        StlFlag::Modified | StlFlag::ModifiedAlt => eval_modified(flag, ctx.buf),
        StlFlag::HelpFlag | StlFlag::HelpFlagAlt => eval_help(flag, ctx.buf),
        StlFlag::PreviewFlag | StlFlag::PreviewFlagAlt => eval_preview(flag, ctx.wp),
        StlFlag::Filetype | StlFlag::FiletypeAlt => eval_filetype(flag, ctx.buf),

        // Offset and byte value - would need line content
        StlFlag::Offset | StlFlag::OffsetX => EvalResult::Number {
            value: 0,
            base: if matches!(flag, StlFlag::OffsetX) {
                NumberBase::Hexadecimal
            } else {
                NumberBase::Decimal
            },
        },
        StlFlag::ByteVal | StlFlag::ByteValX => EvalResult::Number {
            value: 0,
            base: if matches!(flag, StlFlag::ByteValX) {
                NumberBase::Hexadecimal
            } else {
                NumberBase::Decimal
            },
        },

        // Other items (not directly evaluated)
        StlFlag::PageNum => EvalResult::Number {
            value: 0,
            base: NumberBase::Decimal,
        },

        // Items that need special handling
        _ => EvalResult::Empty,
    }
}

/// Evaluate filename items.
fn eval_filename(flag: StlFlag, buf: BufHandle, _wp: WinHandle) -> EvalResult {
    if buf.is_null() {
        return EvalResult::String("[No Name]".to_string());
    }

    unsafe {
        let fname_ptr = match flag {
            StlFlag::FullPath => nvim_buf_get_b_ffname(buf),
            _ => nvim_buf_get_b_fname(buf),
        };

        if fname_ptr.is_null() {
            return EvalResult::String("[No Name]".to_string());
        }

        let fname = match CStr::from_ptr(fname_ptr).to_str() {
            Ok(s) if !s.is_empty() => s,
            _ => return EvalResult::String("[No Name]".to_string()),
        };

        let output = if flag == StlFlag::Filename {
            // Get just the tail (filename without path)
            fname.rsplit('/').next().unwrap_or(fname)
        } else {
            fname
        };

        EvalResult::String(output.to_string())
    }
}

/// Evaluate readonly flag.
fn eval_readonly(flag: StlFlag, buf: BufHandle) -> EvalResult {
    if buf.is_null() {
        return EvalResult::Empty;
    }

    unsafe {
        if nvim_buf_get_b_p_ro(buf) != 0 {
            let s = if flag == StlFlag::RoFlagAlt {
                ",RO"
            } else {
                "[RO]"
            };
            EvalResult::String(s.to_string())
        } else {
            EvalResult::Empty
        }
    }
}

/// Evaluate modified flag.
fn eval_modified(flag: StlFlag, buf: BufHandle) -> EvalResult {
    if buf.is_null() {
        return EvalResult::Empty;
    }

    unsafe {
        let changed = nvim_buf_get_b_changed(buf);
        let modifiable = nvim_buf_get_b_p_ma(buf) != 0;

        let s = match (flag == StlFlag::ModifiedAlt, changed, modifiable) {
            (false, true, true) => "[+]",
            (true, true, true) => ",+",
            (false, false, false) => "[-]",
            (true, false, false) => ",-",
            (false, true, false) => "[+-]",
            (true, true, false) => ",+-",
            _ => return EvalResult::Empty,
        };

        EvalResult::String(s.to_string())
    }
}

/// Evaluate help flag.
fn eval_help(flag: StlFlag, buf: BufHandle) -> EvalResult {
    if buf.is_null() {
        return EvalResult::Empty;
    }

    unsafe {
        if nvim_buf_get_help(buf) != 0 {
            let s = if flag == StlFlag::HelpFlagAlt {
                ",HLP"
            } else {
                "[Help]"
            };
            EvalResult::String(s.to_string())
        } else {
            EvalResult::Empty
        }
    }
}

/// Evaluate preview flag.
fn eval_preview(flag: StlFlag, wp: WinHandle) -> EvalResult {
    if wp.is_null() {
        return EvalResult::Empty;
    }

    unsafe {
        if nvim_win_get_pvw(wp) != 0 {
            let s = if flag == StlFlag::PreviewFlagAlt {
                ",PRV"
            } else {
                "[Preview]"
            };
            EvalResult::String(s.to_string())
        } else {
            EvalResult::Empty
        }
    }
}

/// Evaluate filetype.
fn eval_filetype(flag: StlFlag, buf: BufHandle) -> EvalResult {
    if buf.is_null() {
        return EvalResult::Empty;
    }

    unsafe {
        let ft_ptr = nvim_buf_get_b_p_ft(buf);
        if ft_ptr.is_null() {
            return EvalResult::Empty;
        }

        let ft = match CStr::from_ptr(ft_ptr).to_str() {
            Ok(s) if !s.is_empty() => s,
            _ => return EvalResult::Empty,
        };

        let s = if flag == StlFlag::FiletypeAlt {
            format!(",{}", ft.to_uppercase())
        } else {
            format!("[{ft}]")
        };

        EvalResult::String(s)
    }
}

/// Calculate percentage position in file.
pub fn calc_percentage(lnum: c_int, line_count: c_int) -> c_int {
    if line_count == 0 {
        return 0;
    }
    let lnum_i64 = i64::from(lnum);
    let count_i64 = i64::from(line_count);
    let result = ((lnum_i64 * 100) + (count_i64 / 2)) / count_i64;
    #[allow(clippy::cast_possible_truncation)]
    {
        result.clamp(0, 100) as c_int
    }
}

/// Get relative position string ("Top", "Bot", "All", or percentage).
fn get_rel_pos_string(wp: WinHandle) -> String {
    if wp.is_null() {
        return "Top".to_string();
    }

    // Use the existing implementation from lib.rs via FFI
    let mut buf = [0u8; 8];
    let len = unsafe { crate::rs_stl_get_rel_pos(buf.as_mut_ptr(), 8, wp) };
    #[allow(clippy::cast_sign_loss)]
    if len > 0 {
        String::from_utf8_lossy(&buf[..len as usize]).into_owned()
    } else {
        "Top".to_string()
    }
}

/// Format a numeric value with optional width and padding.
pub fn format_number(
    value: c_int,
    base: NumberBase,
    minwid: c_int,
    zeropad: bool,
    left_align: bool,
) -> String {
    let abs_minwid = minwid.unsigned_abs() as usize;

    let num_str = match base {
        NumberBase::Decimal => format!("{value}"),
        NumberBase::Hexadecimal => format!("{value:X}"),
    };

    if abs_minwid == 0 || num_str.len() >= abs_minwid {
        return num_str;
    }

    let pad_char = if zeropad && !left_align { '0' } else { ' ' };
    let padding = abs_minwid - num_str.len();

    if left_align {
        format!("{num_str}{}", pad_char.to_string().repeat(padding))
    } else {
        format!("{}{num_str}", pad_char.to_string().repeat(padding))
    }
}

/// Format a string value with optional width constraints.
#[allow(clippy::cast_sign_loss)]
pub fn format_string(
    value: &str,
    minwid: c_int,
    maxwid: c_int,
    left_align: bool,
    fillchar: char,
) -> String {
    let len = value.len();
    let abs_minwid = minwid.unsigned_abs() as usize;
    let maxwid_usize = maxwid.max(0) as usize;

    // Truncate if too long
    let truncated = if maxwid > 0 && len > maxwid_usize {
        format!("<{}", &value[len - maxwid_usize + 1..])
    } else {
        value.to_string()
    };

    let truncated_len = truncated.len();

    // Pad if too short
    if abs_minwid > truncated_len {
        let padding = fillchar.to_string().repeat(abs_minwid - truncated_len);
        if left_align {
            format!("{truncated}{padding}")
        } else {
            format!("{padding}{truncated}")
        }
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_percentage() {
        assert_eq!(calc_percentage(1, 100), 1);
        assert_eq!(calc_percentage(50, 100), 50);
        assert_eq!(calc_percentage(100, 100), 100);
        assert_eq!(calc_percentage(0, 100), 0);
        assert_eq!(calc_percentage(1, 0), 0);
        // Rounding test
        assert_eq!(calc_percentage(1, 3), 33);
        assert_eq!(calc_percentage(2, 3), 67);
    }

    #[test]
    fn test_format_number_decimal() {
        assert_eq!(
            format_number(42, NumberBase::Decimal, 0, false, false),
            "42"
        );
        assert_eq!(
            format_number(42, NumberBase::Decimal, 5, false, false),
            "   42"
        );
        assert_eq!(
            format_number(42, NumberBase::Decimal, 5, true, false),
            "00042"
        );
        assert_eq!(
            format_number(42, NumberBase::Decimal, 5, false, true),
            "42   "
        );
    }

    #[test]
    fn test_format_number_hex() {
        assert_eq!(
            format_number(255, NumberBase::Hexadecimal, 0, false, false),
            "FF"
        );
        assert_eq!(
            format_number(255, NumberBase::Hexadecimal, 4, false, false),
            "  FF"
        );
    }

    #[test]
    fn test_format_string_basic() {
        assert_eq!(format_string("hello", 0, 9999, false, ' '), "hello");
        assert_eq!(format_string("hello", 10, 9999, false, ' '), "     hello");
        assert_eq!(format_string("hello", 10, 9999, true, ' '), "hello     ");
    }

    #[test]
    fn test_format_string_truncate() {
        assert_eq!(format_string("hello world", 0, 5, false, ' '), "<orld");
        assert_eq!(format_string("hello", 0, 10, false, ' '), "hello");
    }

    #[test]
    fn test_format_string_fillchar() {
        assert_eq!(format_string("hi", 5, 9999, false, '-'), "---hi");
        assert_eq!(format_string("hi", 5, 9999, true, '-'), "hi---");
    }

    #[test]
    fn test_eval_result_methods() {
        let empty = EvalResult::Empty;
        assert!(empty.is_empty());
        assert!(empty.as_str().is_none());
        assert!(empty.as_number().is_none());

        let string = EvalResult::String("test".to_string());
        assert!(!string.is_empty());
        assert_eq!(string.as_str(), Some("test"));
        assert!(string.as_number().is_none());

        let number = EvalResult::Number {
            value: 42,
            base: NumberBase::Decimal,
        };
        assert!(!number.is_empty());
        assert!(number.as_str().is_none());
        assert_eq!(number.as_number(), Some((42, NumberBase::Decimal)));
    }

    #[test]
    fn test_number_base_values() {
        assert_eq!(NumberBase::Decimal as c_int, 10);
        assert_eq!(NumberBase::Hexadecimal as c_int, 16);
    }
}
