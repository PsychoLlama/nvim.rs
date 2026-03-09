//! Full `do_addsub` migration (Phase 2)
//!
//! Migrated from `do_addsub()` in ops.c — CTRL-A / CTRL-X for
//! incrementing/decrementing numbers and alphabetic characters.

use std::ffi::{c_int, c_void};

/// OP_NR_SUB constant (matches ops.h).
const OP_NR_SUB: c_int = 29;

/// Result from the number-scanning phase.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct AddsubScanResult {
    pub col: c_int,
    pub length: c_int,
    pub firstdigit: c_int,
    pub negative: c_int,
    pub was_positive: c_int,
    pub blank_unsigned: c_int,
    pub past_end: c_int,
    pub no_digit: c_int,
    pub is_alpha: c_int,
}

/// Result from the number-parse phase (after scanning).
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct AddsubParseResult {
    pub pre: c_int,
    pub length: c_int,
    pub n_lo: u64,
    pub col: c_int,
    pub negative: c_int,
    pub overflow: c_int,
}

/// Result from the alpha-change operation.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct AddsubAlphaResult {
    pub did_change: c_int,
    pub endpos_col: c_int,
}

/// Parameters for number arithmetic after parsing.
struct NumArithParams {
    op_type: c_int,
    prenum1: c_int,
    do_hex: c_int,
    do_oct: c_int,
    do_bin: c_int,
    do_unsigned: c_int,
    blank_unsigned: c_int,
    was_positive: c_int,
    visual_flag: c_int,
    firstdigit: c_int,
}

extern "C" {
    fn nvim_addsub_setup(
        pos: *mut c_void,
        out_save_coladd: *mut c_int,
        out_linelen: *mut c_int,
        out_visual: *mut c_int,
    );

    fn nvim_addsub_get_nrformats(
        out_hex: *mut c_int,
        out_oct: *mut c_int,
        out_bin: *mut c_int,
        out_alpha: *mut c_int,
        out_unsigned: *mut c_int,
        out_blank: *mut c_int,
    );

    fn nvim_addsub_do_alpha(
        col: c_int,
        firstdigit: c_int,
        op_type: c_int,
        prenum1: c_int,
        out: *mut c_void,
    );

    fn nvim_addsub_parse_number(
        col: c_int,
        length: c_int,
        negative: c_int,
        do_hex: c_int,
        do_oct: c_int,
        do_bin: c_int,
        visual: c_int,
        out: *mut c_void,
    );

    fn nvim_addsub_replace_number(
        col: c_int,
        length: c_int,
        pre: c_int,
        n_lo: u64,
        negative: c_int,
        was_positive: c_int,
        visual: c_int,
        firstdigit: c_int,
        do_oct: c_int,
        out_endpos_col: *mut c_int,
    );

    fn nvim_addsub_set_marks(startpos_col: c_int, endpos_col: c_int);
    fn nvim_addsub_cleanup(visual: c_int, did_change: c_int, save_coladd: c_int);
    fn nvim_beep_flush();

    // Low-level accessors used by inline scan port
    fn nvim_pos_get_lnum(pos: *const c_void) -> c_int;
    fn nvim_pos_get_col(pos: *const c_void) -> c_int;
    fn nvim_pos_get_coladd(pos: *const c_void) -> c_int;
    fn nvim_ml_get(lnum: c_int) -> *const std::ffi::c_char;
    fn nvim_ml_get_len(lnum: c_int) -> c_int;
    fn nvim_virtual_active() -> bool;
    fn nvim_get_cursor_col() -> c_int;
    fn utf_head_off(base: *const std::ffi::c_char, p: *const std::ffi::c_char) -> c_int;
    fn utfc_ptr2len(p: *const std::ffi::c_char) -> c_int;
}

// ============================================================================
// Inline char predicates (mirror of nvim-ascii crate logic)
// ============================================================================

#[inline]
fn ascii_isdigit(c: c_int) -> bool {
    c >= i32::from(b'0') && c <= i32::from(b'9')
}

#[inline]
fn ascii_isxdigit(c: c_int) -> bool {
    (c >= i32::from(b'0') && c <= i32::from(b'9'))
        || (c >= i32::from(b'a') && c <= i32::from(b'f'))
        || (c >= i32::from(b'A') && c <= i32::from(b'F'))
}

#[inline]
fn ascii_isbdigit(c: c_int) -> bool {
    c == i32::from(b'0') || c == i32::from(b'1')
}

#[inline]
fn ascii_isalpha(c: c_int) -> bool {
    (c >= i32::from(b'a') && c <= i32::from(b'z')) || (c >= i32::from(b'A') && c <= i32::from(b'Z'))
}

#[inline]
fn ascii_iswhite(c: c_int) -> bool {
    c == i32::from(b' ') || c == i32::from(b'\t')
}

// ============================================================================
// Rust port of nvim_addsub_scan (C version deleted)
// ============================================================================

/// Scan for number/alpha start position.
///
/// Rust port of `nvim_addsub_scan` from ops.c.
///
/// # Safety
/// - `pos` must be a valid `pos_T *`
/// - Accesses current buffer and cursor via C shims
#[allow(
    clippy::cast_sign_loss,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::suspicious_operation_groupings
)]
unsafe fn addsub_scan(
    pos: *mut c_void,
    length: c_int,
    do_hex: c_int,
    do_oct: c_int,
    do_bin: c_int,
    do_alpha: c_int,
    do_unsigned: c_int,
    do_blank: c_int,
    visual: c_int,
) -> AddsubScanResult {
    let _ = do_oct; // used via C parse_number only
    let mut out = AddsubScanResult {
        was_positive: 1,
        length,
        ..AddsubScanResult::default()
    };

    let lnum = nvim_pos_get_lnum(pos);
    let line_ptr = nvim_ml_get(lnum);
    let linelen = nvim_ml_get_len(lnum);
    let pos_col = nvim_pos_get_col(pos);
    let pos_coladd = nvim_pos_get_coladd(pos);

    let save_coladd = if nvim_virtual_active() { pos_coladd } else { 0 };
    let mut col = pos_col;

    if col + c_int::from(save_coladd != 0) >= linelen {
        out.past_end = 1;
        return out;
    }

    // Helper: get byte at offset from line_ptr
    let byte_at =
        |offset: c_int| -> c_int { c_int::from(unsafe { *line_ptr.add(offset as usize) as u8 }) };

    // Number scanning logic (non-visual)
    if visual == 0 {
        if do_bin != 0 {
            while col > 0 && ascii_isbdigit(byte_at(col)) {
                col -= 1;
                col -= utf_head_off(line_ptr, line_ptr.add(col as usize));
            }
        }
        if do_hex != 0 {
            while col > 0 && ascii_isxdigit(byte_at(col)) {
                col -= 1;
                col -= utf_head_off(line_ptr, line_ptr.add(col as usize));
            }
        }
        if do_bin != 0
            && do_hex != 0
            && !(col > 0
                && (byte_at(col) == i32::from(b'X') || byte_at(col) == i32::from(b'x'))
                && byte_at(col - 1) == i32::from(b'0')
                && utf_head_off(line_ptr, line_ptr.add((col - 1) as usize)) == 0
                && ascii_isxdigit(byte_at(col + 1)))
        {
            col = nvim_get_cursor_col();
            while col > 0 && ascii_isdigit(byte_at(col)) {
                col -= 1;
                col -= utf_head_off(line_ptr, line_ptr.add(col as usize));
            }
        }

        if (do_hex != 0
            && col > 0
            && (byte_at(col) == i32::from(b'X') || byte_at(col) == i32::from(b'x'))
            && byte_at(col - 1) == i32::from(b'0')
            && utf_head_off(line_ptr, line_ptr.add((col - 1) as usize)) == 0
            && ascii_isxdigit(byte_at(col + 1)))
            || (do_bin != 0
                && col > 0
                && (byte_at(col) == i32::from(b'B') || byte_at(col) == i32::from(b'b'))
                && byte_at(col - 1) == i32::from(b'0')
                && utf_head_off(line_ptr, line_ptr.add((col - 1) as usize)) == 0
                && ascii_isbdigit(byte_at(col + 1)))
        {
            col -= 1;
            col -= utf_head_off(line_ptr, line_ptr.add(col as usize));
        } else {
            col = pos_col;
            while byte_at(col) != 0
                && !ascii_isdigit(byte_at(col))
                && !(do_alpha != 0 && ascii_isalpha(byte_at(col)))
            {
                col += 1;
            }
            while col > 0
                && ascii_isdigit(byte_at(col - 1))
                && !(do_alpha != 0 && ascii_isalpha(byte_at(col)))
            {
                col -= 1;
            }
        }
    }

    // Visual mode scanning
    let mut length = length;
    if visual != 0 {
        while byte_at(col) != 0
            && length > 0
            && !ascii_isdigit(byte_at(col))
            && !(do_alpha != 0 && ascii_isalpha(byte_at(col)))
        {
            let mb_len = utfc_ptr2len(line_ptr.add(col as usize));
            col += mb_len;
            length -= mb_len;
        }
        if length == 0 {
            out.past_end = 1;
            return out;
        }
        out.length = length;

        if col > pos_col
            && byte_at(col - 1) == i32::from(b'-')
            && utf_head_off(line_ptr, line_ptr.add((col - 1) as usize)) == 0
            && do_unsigned == 0
        {
            if do_blank != 0 && col >= 2 && !ascii_iswhite(byte_at(col - 2)) {
                out.blank_unsigned = 1;
            } else {
                out.negative = 1;
                out.was_positive = 0;
            }
        }
    }

    let firstdigit = byte_at(col);
    if !(ascii_isdigit(firstdigit) || (do_alpha != 0 && ascii_isalpha(firstdigit))) {
        out.no_digit = 1;
        return out;
    }

    out.col = col;
    out.firstdigit = firstdigit;
    out.is_alpha = c_int::from(do_alpha != 0 && ascii_isalpha(firstdigit));
    out
}

// Perform number arithmetic: parse, compute, replace.
// Returns the endpos column for mark setting.
#[allow(clippy::cast_sign_loss)]
unsafe fn do_number_addsub(scan: &AddsubScanResult, params: &NumArithParams) -> c_int {
    let mut parse = AddsubParseResult::default();
    nvim_addsub_parse_number(
        scan.col,
        scan.length,
        scan.negative,
        params.do_hex,
        params.do_oct,
        params.do_bin,
        params.visual_flag,
        std::ptr::from_mut(&mut parse).cast::<c_void>(),
    );

    let mut negative = parse.negative != 0;
    let pre = parse.pre;

    // Compute add/subtract
    let subtract = (params.op_type == OP_NR_SUB) ^ negative;
    let oldn = parse.n_lo;
    let mut n = if parse.overflow == 0 {
        if subtract {
            oldn.wrapping_sub(params.prenum1 as u64)
        } else {
            oldn.wrapping_add(params.prenum1 as u64)
        }
    } else {
        oldn
    };

    // Handle wraparound for decimal numbers
    if pre == 0 {
        if subtract {
            if n > oldn {
                n = 1u64.wrapping_add(n ^ u64::MAX);
                negative = !negative;
            }
        } else if n < oldn {
            n ^= u64::MAX;
            negative = !negative;
        }
        if n == 0 {
            negative = false;
        }
    }

    // Handle unsigned constraint
    if (params.do_unsigned != 0 || params.blank_unsigned != 0) && negative {
        n = if subtract { 0 } else { u64::MAX };
        negative = false;
    }

    // Adjust column if minus sign needs to be removed
    let mut col = parse.col;
    let mut adj_length = parse.length;
    let was_positive = params.was_positive != 0;
    let visual = params.visual_flag != 0;
    if visual && !was_positive && !negative && col > 0 {
        col -= 1;
        adj_length += 1;
    }

    let mut endpos_col: c_int = 0;
    nvim_addsub_replace_number(
        col,
        adj_length,
        pre,
        n,
        c_int::from(negative),
        c_int::from(was_positive),
        params.visual_flag,
        params.firstdigit,
        params.do_oct,
        &raw mut endpos_col,
    );

    nvim_addsub_set_marks(col, endpos_col);
    endpos_col
}

/// Full migration of `do_addsub()`.
///
/// # Safety
/// - `pos` must be a valid `pos_T *`
/// - Accesses global state via C accessors
#[unsafe(export_name = "do_addsub")]
pub unsafe extern "C" fn rs_do_addsub(
    op_type: c_int,
    pos: *mut c_void,
    length: c_int,
    prenum1: c_int,
) -> bool {
    let mut save_coladd: c_int = 0;
    let mut linelen: c_int = 0;
    let mut visual_flag: c_int = 0;

    nvim_addsub_setup(
        pos,
        &raw mut save_coladd,
        &raw mut linelen,
        &raw mut visual_flag,
    );

    let mut do_hex: c_int = 0;
    let mut do_oct: c_int = 0;
    let mut do_bin: c_int = 0;
    let mut do_alpha: c_int = 0;
    let mut do_unsigned: c_int = 0;
    let mut do_blank: c_int = 0;
    nvim_addsub_get_nrformats(
        &raw mut do_hex,
        &raw mut do_oct,
        &raw mut do_bin,
        &raw mut do_alpha,
        &raw mut do_unsigned,
        &raw mut do_blank,
    );

    let scan = addsub_scan(
        pos,
        length,
        do_hex,
        do_oct,
        do_bin,
        do_alpha,
        do_unsigned,
        do_blank,
        visual_flag,
    );

    if scan.past_end != 0 {
        nvim_addsub_cleanup(visual_flag, 0, save_coladd);
        return false;
    }

    if scan.no_digit != 0 {
        nvim_beep_flush();
        nvim_addsub_cleanup(visual_flag, 0, save_coladd);
        return false;
    }

    let did_change = if scan.is_alpha != 0 {
        let mut alpha_result = AddsubAlphaResult::default();
        nvim_addsub_do_alpha(
            scan.col,
            scan.firstdigit,
            op_type,
            prenum1,
            std::ptr::from_mut(&mut alpha_result).cast::<c_void>(),
        );
        let changed = alpha_result.did_change != 0;
        if changed {
            nvim_addsub_set_marks(scan.col, alpha_result.endpos_col);
        }
        changed
    } else {
        let params = NumArithParams {
            op_type,
            prenum1,
            do_hex,
            do_oct,
            do_bin,
            do_unsigned,
            blank_unsigned: scan.blank_unsigned,
            was_positive: scan.was_positive,
            visual_flag,
            firstdigit: scan.firstdigit,
        };
        do_number_addsub(&scan, &params);
        true
    };

    nvim_addsub_cleanup(visual_flag, c_int::from(did_change), save_coladd);
    did_change
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OP_NR_SUB, 29);
    }

    #[test]
    fn test_scan_result_default() {
        let sr = AddsubScanResult::default();
        assert_eq!(sr.col, 0);
        assert_eq!(sr.length, 0);
        assert_eq!(sr.firstdigit, 0);
        assert_eq!(sr.negative, 0);
        assert_eq!(sr.was_positive, 0);
        assert_eq!(sr.blank_unsigned, 0);
        assert_eq!(sr.past_end, 0);
        assert_eq!(sr.no_digit, 0);
        assert_eq!(sr.is_alpha, 0);
    }

    #[test]
    fn test_parse_result_default() {
        let pr = AddsubParseResult::default();
        assert_eq!(pr.pre, 0);
        assert_eq!(pr.length, 0);
        assert_eq!(pr.n_lo, 0);
        assert_eq!(pr.col, 0);
        assert_eq!(pr.negative, 0);
        assert_eq!(pr.overflow, 0);
    }

    #[test]
    fn test_wraparound_logic() {
        // Decimal subtract wraparound: 5 - 10 = wraps
        let oldn: u64 = 5;
        let n = oldn.wrapping_sub(10);
        assert!(n > oldn); // wrapped around
        let corrected = 1u64.wrapping_add(n ^ u64::MAX);
        assert_eq!(corrected, 5); // |5 - 10| = 5, flips sign

        // Decimal add wraparound: MAX - 1 + 5 wraps to 3
        let oldn = u64::MAX - 1;
        let n = oldn.wrapping_add(5);
        assert_eq!(n, 3);
        assert!(n < oldn); // wrapped around
                           // C code: n = (n ^ (uvarnumber_T)(-1)), plus sign flip
        let corrected = n ^ u64::MAX;
        assert_eq!(corrected, u64::MAX - 3);

        // No wraparound: 10 + 5
        let oldn: u64 = 10;
        let n = oldn.wrapping_add(5);
        assert_eq!(n, 15);
        assert!(n >= oldn);
    }

    #[test]
    fn test_unsigned_clamping() {
        let mut n: u64 = 42;
        let mut neg = true;
        let subtract = true;
        let do_unsigned = true;

        if do_unsigned && neg {
            n = if subtract { 0 } else { u64::MAX };
            neg = false;
        }
        assert_eq!(n, 0);
        assert!(!neg);
    }

    /// Test the XOR logic used for subtract determination.
    /// `subtract = (op_type == OP_NR_SUB) ^ negative`
    #[test]
    fn test_subtract_xor_logic() {
        // Use a helper to prevent constant folding
        let xor_subtract = |is_sub_op: bool, neg: bool| is_sub_op ^ neg;

        // SUB op + positive number = subtract
        assert!(xor_subtract(true, false));
        // SUB op + negative number = add (double negative)
        assert!(!xor_subtract(true, true));
        // ADD op + positive number = add
        assert!(!xor_subtract(false, false));
        // ADD op + negative number = subtract
        assert!(xor_subtract(false, true));
    }
}
