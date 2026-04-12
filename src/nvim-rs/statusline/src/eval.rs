//! Item evaluation for statusline
//!
//! This module provides evaluation of statusline format items.
//! Each item type can produce a string or numeric value.

use std::ffi::{c_char, c_int, c_void, CStr};

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
    fn nvim_win_get_arg_idx(wp: WinHandle) -> c_int;
    fn nvim_win_get_arg_idx_invalid(wp: WinHandle) -> c_int;
    fn nvim_win_argcount(wp: WinHandle) -> c_int;

    // Statusline-specific accessors
    fn nvim_stl_get_win_cursor_info(wp: WinHandle) -> crate::stl_build::StlCursorInfo;
    // Quickfix/keymap: direct underlying functions (used by stl_* helpers in stl_build)
    fn nvim_win_is_qf_win(wp: WinHandle) -> bool;
    fn nvim_win_get_llist_ref(wp: WinHandle) -> *mut c_void;
    fn nvim_stl_get_msg_loclist() -> *const c_char;
    fn nvim_stl_get_msg_qflist() -> *const c_char;
    fn get_keymap_str(wp: WinHandle, fmt: *const c_char, buf: *mut c_char, len: c_int) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    static showcmd_buf: [u8; 41];
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
            value: get_current_col(ctx),
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

        // Argument list status
        StlFlag::ArgListStat => eval_arglist_status(ctx.wp),

        // Page number (printing) - always 0 (not applicable for screen display)
        StlFlag::PageNum => EvalResult::Number {
            value: 0,
            base: NumberBase::Decimal,
        },

        // Byte offset items
        StlFlag::Offset => unsafe {
            let info = nvim_stl_get_win_cursor_info(ctx.wp);
            EvalResult::Number {
                value: info.byte_offset,
                base: NumberBase::Decimal,
            }
        },
        StlFlag::OffsetX => unsafe {
            let info = nvim_stl_get_win_cursor_info(ctx.wp);
            EvalResult::Number {
                value: info.byte_offset,
                base: NumberBase::Hexadecimal,
            }
        },

        // Byte value items
        StlFlag::ByteVal => unsafe {
            let info = nvim_stl_get_win_cursor_info(ctx.wp);
            EvalResult::Number {
                value: info.byte_value,
                base: NumberBase::Decimal,
            }
        },
        StlFlag::ByteValX => unsafe {
            let info = nvim_stl_get_win_cursor_info(ctx.wp);
            EvalResult::Number {
                value: info.byte_value,
                base: NumberBase::Hexadecimal,
            }
        },

        // Keymap
        StlFlag::Keymap => eval_keymap(ctx.wp),

        // Quickfix
        StlFlag::Quickfix => eval_quickfix(ctx.wp),

        // Showcmd
        StlFlag::ShowCmd => eval_showcmd(),

        // Items that need special handling at render level
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

/// Evaluate argument list status (%a).
///
/// Shows "(2 of 8)" or "((2) of 8)" if editing more than one file in the argument list.
fn eval_arglist_status(wp: WinHandle) -> EvalResult {
    if wp.is_null() {
        return EvalResult::Empty;
    }

    unsafe {
        let argcount = nvim_win_argcount(wp);
        if argcount <= 1 {
            return EvalResult::Empty;
        }

        let arg_idx = nvim_win_get_arg_idx(wp);
        let arg_idx_invalid = nvim_win_get_arg_idx_invalid(wp) != 0;

        let s = if arg_idx_invalid {
            format!("(({}) of {})", arg_idx + 1, argcount)
        } else {
            format!("({} of {})", arg_idx + 1, argcount)
        };

        EvalResult::String(s)
    }
}

/// Evaluate keymap indicator (%k).
///
/// Shows the keymap name when active, formatted as "<%s>".
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
fn eval_keymap(wp: WinHandle) -> EvalResult {
    if wp.is_null() {
        return EvalResult::Empty;
    }

    let mut buf = [0u8; 128];
    unsafe {
        let fmt = c"<%s>".as_ptr();
        let len = get_keymap_str(wp, fmt, buf.as_mut_ptr().cast(), buf.len() as c_int);
        if len > 0 {
            std::str::from_utf8(&buf[..len as usize])
                .map_or(EvalResult::Empty, |s| EvalResult::String(s.to_string()))
        } else {
            EvalResult::Empty
        }
    }
}

/// Evaluate quickfix/location list indicator (%q).
///
/// Shows "[Quickfix List]" or "[Location List]" when in a quickfix/loclist window.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
fn eval_quickfix(wp: WinHandle) -> EvalResult {
    if wp.is_null() {
        return EvalResult::Empty;
    }

    unsafe {
        if nvim_win_is_qf_win(wp) {
            let msg = if nvim_win_get_llist_ref(wp).is_null() {
                nvim_stl_get_msg_qflist()
            } else {
                nvim_stl_get_msg_loclist()
            };
            if msg.is_null() {
                return EvalResult::Empty;
            }
            let msg_len = strlen(msg);
            if msg_len > 0 {
                let slice = std::slice::from_raw_parts(msg.cast::<u8>(), msg_len);
                return std::str::from_utf8(slice)
                    .map_or(EvalResult::Empty, |s| EvalResult::String(s.to_string()));
            }
        }
        EvalResult::Empty
    }
}

/// Evaluate showcmd buffer (%S).
///
/// Shows the current command sequence being typed.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
fn eval_showcmd() -> EvalResult {
    unsafe {
        if showcmd_buf[0] == 0 {
            return EvalResult::Empty;
        }
        let mut len = 0usize;
        while len < showcmd_buf.len() && showcmd_buf[len] != 0 {
            len += 1;
        }
        std::str::from_utf8(&showcmd_buf[..len])
            .map_or(EvalResult::Empty, |s| EvalResult::String(s.to_string()))
    }
}

/// Get current column position, properly handling empty lines and insert mode.
const fn get_current_col(ctx: &EvalContext) -> c_int {
    if !ctx.mode_insert && ctx.empty_line {
        0
    } else {
        ctx.col
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
///
/// If `maxwid` is specified and the number would exceed it, the number is
/// displayed in "scientific" notation (e.g., 14532 with maxwid=4 -> "14>3").
pub fn format_number(
    value: c_int,
    base: NumberBase,
    minwid: c_int,
    zeropad: bool,
    left_align: bool,
) -> String {
    format_number_with_maxwid(value, base, minwid, 0, zeropad, left_align)
}

/// Format a numeric value with optional width, max width, and padding.
///
/// If `maxwid` is specified and the number would exceed it, the number is
/// displayed in "scientific" notation (e.g., 14532 with maxwid=4 -> "14>3").
#[allow(clippy::cast_sign_loss)]
pub fn format_number_with_maxwid(
    value: c_int,
    base: NumberBase,
    minwid: c_int,
    maxwid: c_int,
    zeropad: bool,
    left_align: bool,
) -> String {
    let abs_minwid = minwid.unsigned_abs() as usize;
    let divisor = base as c_int;

    // Count the number of digits
    let mut num_chars = 1;
    let mut n = value.abs();
    while n >= divisor {
        n /= divisor;
        num_chars += 1;
    }

    // Check if we need scientific notation
    if maxwid > 0 && num_chars > maxwid as usize {
        return format_scientific(value, base, maxwid as usize);
    }

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

/// Format a number in "scientific" notation for statusline display.
///
/// When a number is too long for the available width, it's displayed as
/// the reduced number followed by '>n' where n is the exponent.
/// For example: 14532 with maxwid=4 becomes "14>3" (14 * 10^3 ≈ 14000).
#[allow(clippy::cast_sign_loss)]
fn format_scientific(value: c_int, base: NumberBase, maxwid: usize) -> String {
    if maxwid < 3 {
        // Need at least 3 chars for "n>e" format
        return match base {
            NumberBase::Decimal => format!("{value}"),
            NumberBase::Hexadecimal => format!("{value:X}"),
        };
    }

    let divisor = base as c_int;
    let mut num = value.abs();
    let mut exponent = 0;

    // Count digits
    let mut num_chars = 1;
    let mut temp = num;
    while temp >= divisor {
        temp /= divisor;
        num_chars += 1;
    }

    // Add 2 for the ">e" suffix
    num_chars += 2;

    // Reduce the number until it fits
    while num_chars > maxwid {
        num /= divisor;
        exponent += 1;
        num_chars -= 1;
    }

    match base {
        NumberBase::Decimal => format!("{num}>{exponent}"),
        NumberBase::Hexadecimal => format!("{num:X}>{exponent:X}"),
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
    fn test_format_number_scientific() {
        // 14532 with maxwid=4 should become "14>3" (14 * 10^3 ≈ 14000)
        assert_eq!(
            format_number_with_maxwid(14532, NumberBase::Decimal, 0, 4, false, false),
            "14>3"
        );
        // 99999 with maxwid=4 should become "99>3"
        assert_eq!(
            format_number_with_maxwid(99999, NumberBase::Decimal, 0, 4, false, false),
            "99>3"
        );
        // Number that fits shouldn't use scientific notation
        assert_eq!(
            format_number_with_maxwid(123, NumberBase::Decimal, 0, 5, false, false),
            "123"
        );
    }

    #[test]
    fn test_format_number_scientific_hex() {
        // Hex: 1048575 (0xFFFFF) with maxwid=4 should use scientific notation
        // 0xFFFFF = 5 hex digits, with >e = 7 chars total
        // Reduce to fit in 4: FF>3 (255 * 16^3 = 1044480 ≈ 1048575)
        assert_eq!(
            format_number_with_maxwid(1_048_575, NumberBase::Hexadecimal, 0, 4, false, false),
            "FF>3"
        );
        // 0xFFFF = 4 hex digits, should fit without scientific notation
        assert_eq!(
            format_number_with_maxwid(65535, NumberBase::Hexadecimal, 0, 4, false, false),
            "FFFF"
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

    // Tests for eval_flag are integration tests that require C FFI
    // and are run as part of the full build. The following tests
    // verify the pure-Rust parts of the eval module.

    #[test]
    fn test_stl_flag_is_numeric() {
        // Verify which flags produce numeric values
        assert!(StlFlag::Line.is_numeric());
        assert!(StlFlag::NumLines.is_numeric());
        assert!(StlFlag::Column.is_numeric());
        assert!(StlFlag::VirtCol.is_numeric());
        assert!(StlFlag::Percentage.is_numeric());
        assert!(StlFlag::Offset.is_numeric());
        assert!(StlFlag::OffsetX.is_numeric());
        assert!(StlFlag::ByteVal.is_numeric());
        assert!(StlFlag::ByteValX.is_numeric());
        assert!(StlFlag::PageNum.is_numeric());
        assert!(StlFlag::BufNo.is_numeric());
        // Non-numeric
        assert!(!StlFlag::Keymap.is_numeric());
        assert!(!StlFlag::Quickfix.is_numeric());
        assert!(!StlFlag::ShowCmd.is_numeric());
        assert!(!StlFlag::FilePath.is_numeric());
    }

    #[test]
    fn test_stl_flag_is_flag_item() {
        // Verify which flags are conditional items
        assert!(StlFlag::RoFlag.is_flag_item());
        assert!(StlFlag::Modified.is_flag_item());
        assert!(StlFlag::HelpFlag.is_flag_item());
        assert!(StlFlag::PreviewFlag.is_flag_item());
        // Non-flag items
        assert!(!StlFlag::Line.is_flag_item());
        assert!(!StlFlag::Keymap.is_flag_item());
        assert!(!StlFlag::Quickfix.is_flag_item());
    }
}
