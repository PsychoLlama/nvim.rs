//! Special position matching for regex engine.
//!
//! This module provides infrastructure for matching special Vim positions:
//! - Cursor position (`\%#`)
//! - Line number (`\%l`, `\%<l`, `\%>l`)
//! - Column number (`\%c`, `\%<c`, `\%>c`)
//! - Virtual column (`\%v`, `\%<v`, `\%>v`)
//! - Marks (`\%'m`, `\%<'m`, `\%>'m`)
//! - Visual area (`\%V`)
//!
//! These features require access to buffer/window state which is managed
//! through C FFI callbacks.

use std::ffi::c_int;

use crate::{BufHandle, WinHandle};

// =============================================================================
// FFI Declarations
// =============================================================================

#[allow(dead_code)] // Some accessors are infrastructure for future phases
extern "C" {
    // Window accessors
    fn nvim_win_get_cursor_lnum(win: WinHandle) -> c_int;
    fn nvim_win_get_cursor_col(win: WinHandle) -> c_int;

    // Buffer accessors
    fn nvim_buf_get_line_count(buf: BufHandle) -> c_int;

    // Rex state accessors
    fn nvim_rex_get_reg_win() -> WinHandle;
    fn nvim_rex_get_reg_buf() -> BufHandle;
    fn nvim_rex_get_reg_firstlnum() -> c_int;
    fn nvim_rex_get_lnum() -> c_int;
}

// =============================================================================
// Comparison Types
// =============================================================================

/// Comparison operator for position matching.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    /// Equal (no modifier)
    Equal = b'=',
    /// Less than (`<`)
    LessThan = b'<',
    /// Greater than (`>`)
    GreaterThan = b'>',
}

impl From<u8> for CmpOp {
    fn from(b: u8) -> Self {
        match b {
            b'<' => Self::LessThan,
            b'>' => Self::GreaterThan,
            _ => Self::Equal,
        }
    }
}

impl CmpOp {
    /// Perform the comparison.
    pub fn compare(self, actual: c_int, expected: c_int) -> bool {
        match self {
            Self::Equal => actual == expected,
            Self::LessThan => actual < expected,
            Self::GreaterThan => actual > expected,
        }
    }
}

// =============================================================================
// Cursor Matching
// =============================================================================

/// Check if current position matches cursor position.
///
/// # Parameters
/// - `lnum`: Current line number (relative to first line)
/// - `col`: Current column (0-based byte offset)
///
/// # Safety
/// Rex must be properly initialized with a valid window.
pub unsafe fn match_cursor(lnum: c_int, col: c_int) -> bool {
    let win = nvim_rex_get_reg_win();
    if win.is_null() {
        return false;
    }

    let first_lnum = nvim_rex_get_reg_firstlnum();
    let cursor_lnum = nvim_win_get_cursor_lnum(win);
    let cursor_col = nvim_win_get_cursor_col(win);

    // Convert relative lnum to absolute
    let abs_lnum = first_lnum + lnum;

    abs_lnum == cursor_lnum && col == cursor_col
}

// =============================================================================
// Line Number Matching
// =============================================================================

/// Check if current line matches a line number comparison.
///
/// # Parameters
/// - `lnum`: Current line number (relative to first line)
/// - `target`: Target line number
/// - `cmp`: Comparison operator
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn match_lnum(lnum: c_int, target: c_int, cmp: CmpOp) -> bool {
    let first_lnum = nvim_rex_get_reg_firstlnum();
    let abs_lnum = first_lnum + lnum;
    cmp.compare(abs_lnum, target)
}

// =============================================================================
// Column Matching
// =============================================================================

/// Check if current column matches a column comparison.
///
/// # Parameters
/// - `col`: Current column (0-based byte offset)
/// - `target`: Target column (1-based in patterns)
/// - `cmp`: Comparison operator
pub fn match_col(col: c_int, target: c_int, cmp: CmpOp) -> bool {
    // Vim columns are 1-based in patterns, but our col is 0-based
    cmp.compare(col + 1, target)
}

// =============================================================================
// Virtual Column Matching
// =============================================================================

/// Virtual column info for matching.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VColInfo {
    /// Virtual column (1-based).
    pub vcol: c_int,
    /// Whether the position is valid.
    pub valid: bool,
}

/// Get virtual column at a position.
///
/// Virtual columns account for tabs and other display-width characters.
///
/// # Parameters
/// - `lnum`: Line number (1-based absolute)
/// - `col`: Column (0-based byte offset)
///
/// # Safety
/// Rex must be properly initialized with valid window.
pub unsafe fn get_vcol(_lnum: c_int, col: c_int) -> VColInfo {
    let win = nvim_rex_get_reg_win();
    if win.is_null() {
        return VColInfo {
            vcol: col + 1, // Fallback to byte column
            valid: false,
        };
    }

    // For now, just use byte column as virtual column
    // Full implementation would call win_linetabsize with _lnum
    VColInfo {
        vcol: col + 1,
        valid: true,
    }
}

/// Check if current virtual column matches a comparison.
///
/// # Parameters
/// - `lnum`: Line number (relative)
/// - `col`: Column (0-based)
/// - `target`: Target virtual column (1-based)
/// - `cmp`: Comparison operator
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn match_vcol(lnum: c_int, col: c_int, target: c_int, cmp: CmpOp) -> bool {
    let first_lnum = nvim_rex_get_reg_firstlnum();
    let abs_lnum = first_lnum + lnum;
    let info = get_vcol(abs_lnum, col);
    cmp.compare(info.vcol, target)
}

// =============================================================================
// Mark Matching
// =============================================================================

/// Mark position info.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MarkPos {
    /// Line number (1-based, 0 = invalid).
    pub lnum: c_int,
    /// Column (0-based).
    pub col: c_int,
    /// Whether the mark exists.
    pub valid: bool,
}

// Note: Mark lookup requires more complex FFI that would need mark_get

/// Check if current position matches a mark comparison.
///
/// # Parameters
/// - `lnum`: Current line number (relative)
/// - `col`: Current column (0-based)
/// - `mark_pos`: Mark position
/// - `cmp`: Comparison operator
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn match_mark(lnum: c_int, col: c_int, mark_pos: &MarkPos, cmp: CmpOp) -> bool {
    if !mark_pos.valid || mark_pos.lnum <= 0 {
        return false;
    }

    let first_lnum = nvim_rex_get_reg_firstlnum();
    let abs_lnum = first_lnum + lnum;

    match cmp {
        CmpOp::Equal => abs_lnum == mark_pos.lnum && col == mark_pos.col,
        CmpOp::LessThan => {
            abs_lnum < mark_pos.lnum || (abs_lnum == mark_pos.lnum && col < mark_pos.col)
        }
        CmpOp::GreaterThan => {
            abs_lnum > mark_pos.lnum || (abs_lnum == mark_pos.lnum && col > mark_pos.col)
        }
    }
}

// =============================================================================
// Visual Area Matching
// =============================================================================

/// Visual selection info.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VisualInfo {
    /// Start line (1-based).
    pub start_lnum: c_int,
    /// Start column (0-based).
    pub start_col: c_int,
    /// End line (1-based).
    pub end_lnum: c_int,
    /// End column (0-based).
    pub end_col: c_int,
    /// Visual mode character ('v', 'V', or Ctrl-V).
    pub mode: u8,
    /// Whether visual selection is active.
    pub active: bool,
}

/// Check if position is within visual selection.
///
/// # Parameters
/// - `lnum`: Line number (relative)
/// - `col`: Column (0-based)
/// - `visual`: Visual selection info
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn match_visual(lnum: c_int, col: c_int, visual: &VisualInfo) -> bool {
    if !visual.active {
        return false;
    }

    let first_lnum = nvim_rex_get_reg_firstlnum();
    let abs_lnum = first_lnum + lnum;

    // Check line bounds
    if abs_lnum < visual.start_lnum || abs_lnum > visual.end_lnum {
        return false;
    }

    match visual.mode {
        b'V' => {
            // Line-wise: any column on selected lines matches
            true
        }
        b'v' => {
            // Character-wise
            if visual.start_lnum == visual.end_lnum {
                // Single line
                col >= visual.start_col && col <= visual.end_col
            } else if abs_lnum == visual.start_lnum {
                col >= visual.start_col
            } else if abs_lnum == visual.end_lnum {
                col <= visual.end_col
            } else {
                // Middle lines
                true
            }
        }
        _ => {
            // Block-wise (Ctrl-V = 0x16)
            col >= visual.start_col && col <= visual.end_col
        }
    }
}

// =============================================================================
// Beginning/End of File
// =============================================================================

/// Check if at beginning of file.
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn match_bof(lnum: c_int, col: c_int) -> bool {
    let first_lnum = nvim_rex_get_reg_firstlnum();

    // At BOF if: line 0 (relative), column 0, and first_lnum is 1
    lnum == 0 && col == 0 && first_lnum == 1
}

/// Check if at end of file.
///
/// # Parameters
/// - `lnum`: Current line number (relative)
/// - `at_eol`: Whether at end of line
///
/// # Safety
/// Rex must be properly initialized.
pub unsafe fn match_eof(lnum: c_int, at_eol: bool) -> bool {
    if !at_eol {
        return false;
    }

    let buf = nvim_rex_get_reg_buf();
    if buf.is_null() {
        return false;
    }

    let first_lnum = nvim_rex_get_reg_firstlnum();
    let line_count = nvim_buf_get_line_count(buf);

    // At EOF if: at end of line and on last line
    first_lnum + lnum == line_count
}

// =============================================================================
// Number Comparison Operand Parsing
// =============================================================================

/// Parse a number comparison operand from bytecode.
///
/// The operand format is: 4 bytes number (big-endian) + 1 byte comparison op
///
/// # Safety
/// `p` must point to valid bytecode with at least 5 bytes.
pub unsafe fn parse_num_cmp(p: *const u8) -> (u32, CmpOp) {
    // Read 4-byte number (big-endian)
    let num = ((*p) as u32) << 24
        | ((*p.add(1)) as u32) << 16
        | ((*p.add(2)) as u32) << 8
        | (*p.add(3)) as u32;

    // Read comparison operator
    let cmp = CmpOp::from(*p.add(4));

    (num, cmp)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Match cursor position.
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_match_cursor(lnum: c_int, col: c_int) -> c_int {
    c_int::from(match_cursor(lnum, col))
}

/// Match line number.
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_match_lnum(lnum: c_int, target: c_int, cmp: u8) -> c_int {
    c_int::from(match_lnum(lnum, target, CmpOp::from(cmp)))
}

/// Match column.
#[no_mangle]
pub extern "C" fn rs_match_col(col: c_int, target: c_int, cmp: u8) -> c_int {
    c_int::from(match_col(col, target, CmpOp::from(cmp)))
}

/// Match virtual column.
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_match_vcol(lnum: c_int, col: c_int, target: c_int, cmp: u8) -> c_int {
    c_int::from(match_vcol(lnum, col, target, CmpOp::from(cmp)))
}

/// Match beginning of file.
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_match_bof(lnum: c_int, col: c_int) -> c_int {
    c_int::from(match_bof(lnum, col))
}

/// Match end of file.
///
/// # Safety
/// Rex must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_match_eof(lnum: c_int, at_eol: c_int) -> c_int {
    c_int::from(match_eof(lnum, at_eol != 0))
}

/// Parse number comparison operand.
///
/// # Safety
/// `p` must point to valid bytecode.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_num_cmp(p: *const u8, num_out: *mut u32, cmp_out: *mut u8) {
    if p.is_null() {
        return;
    }
    let (num, cmp) = parse_num_cmp(p);
    if !num_out.is_null() {
        *num_out = num;
    }
    if !cmp_out.is_null() {
        *cmp_out = cmp as u8;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_op() {
        assert_eq!(CmpOp::from(b'<'), CmpOp::LessThan);
        assert_eq!(CmpOp::from(b'>'), CmpOp::GreaterThan);
        assert_eq!(CmpOp::from(b'='), CmpOp::Equal);
        assert_eq!(CmpOp::from(b'x'), CmpOp::Equal); // Unknown defaults to equal
    }

    #[test]
    fn test_cmp_compare() {
        assert!(CmpOp::Equal.compare(5, 5));
        assert!(!CmpOp::Equal.compare(5, 6));

        assert!(CmpOp::LessThan.compare(5, 10));
        assert!(!CmpOp::LessThan.compare(10, 5));

        assert!(CmpOp::GreaterThan.compare(10, 5));
        assert!(!CmpOp::GreaterThan.compare(5, 10));
    }

    #[test]
    fn test_match_col() {
        // col is 0-based, target is 1-based
        assert!(match_col(0, 1, CmpOp::Equal)); // col 0 + 1 = 1
        assert!(match_col(4, 5, CmpOp::Equal)); // col 4 + 1 = 5
        assert!(match_col(0, 2, CmpOp::LessThan)); // 1 < 2
        assert!(match_col(5, 3, CmpOp::GreaterThan)); // 6 > 3
    }

    #[test]
    fn test_visual_info_default() {
        let vi = VisualInfo::default();
        assert!(!vi.active);
        assert_eq!(vi.start_lnum, 0);
        assert_eq!(vi.end_lnum, 0);
    }

    #[test]
    fn test_mark_pos_default() {
        let mp = MarkPos::default();
        assert!(!mp.valid);
        assert_eq!(mp.lnum, 0);
        assert_eq!(mp.col, 0);
    }

    #[test]
    fn test_vcol_info_default() {
        let vi = VColInfo::default();
        assert!(!vi.valid);
        assert_eq!(vi.vcol, 0);
    }

    #[test]
    fn test_parse_num_cmp() {
        let bytecode = [0x00, 0x00, 0x00, 0x0A, b'<']; // 10, less-than
        unsafe {
            let (num, cmp) = parse_num_cmp(bytecode.as_ptr());
            assert_eq!(num, 10);
            assert_eq!(cmp, CmpOp::LessThan);
        }

        let bytecode = [0x00, 0x01, 0x00, 0x00, b'>']; // 65536, greater-than
        unsafe {
            let (num, cmp) = parse_num_cmp(bytecode.as_ptr());
            assert_eq!(num, 65536);
            assert_eq!(cmp, CmpOp::GreaterThan);
        }
    }
}
