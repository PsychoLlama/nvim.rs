//! Full `do_addsub` migration (Phase 2)
//!
//! Migrated from `do_addsub()` in ops.c — CTRL-A / CTRL-X for
//! incrementing/decrementing numbers and alphabetic characters.

use std::ffi::{c_int, c_void};

/// OP_NR_SUB constant (matches ops.h).
const OP_NR_SUB: c_int = 29;

/// Saved cursor position for addsub visual mode (mirrors `addsub_saved_cursor` C static).
/// Neovim is single-threaded; mutated only under the Neovim GIL equivalent.
static mut ADDSUB_SAVED_CURSOR: [i32; 3] = [0; 3];

/// Result from the number-scanning phase.
#[derive(Debug, Clone, Default)]
struct AddsubScanResult {
    col: c_int,
    length: c_int,
    firstdigit: c_int,
    negative: c_int,
    was_positive: c_int,
    blank_unsigned: c_int,
    past_end: c_int,
    no_digit: c_int,
    is_alpha: c_int,
}

/// Result from the number-parse phase (after scanning).
#[derive(Debug, Clone, Default)]
struct AddsubParseResult {
    pre: c_int,
    length: c_int,
    n_lo: u64,
    col: c_int,
    negative: c_int,
    overflow: c_int,
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
    fn nvim_get_curwin() -> nvim_window::WinHandle;
    fn nvim_curbuf_nf_has(c: c_int) -> c_int;
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_curbuf_set_op_start_to_cursor_col(col: c_int);
    fn nvim_curbuf_set_op_end_to_cursor_col(col: c_int);
    fn nvim_curwin_set_cursor_from_pos(pos: *const c_void);
    fn nvim_curwin_set_w_set_curswant(v: bool);
    fn nvim_curwin_set_cursor_coladd(v: c_int);
    fn nvim_VIsual_active() -> c_int;

    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_get_VIsual_mode() -> c_int;
    fn nvim_get_b_visual_vi_curswant() -> c_int;
    fn vim_str2nr(
        start: *const std::ffi::c_char,
        prep: *mut c_int,
        len: *mut c_int,
        what: c_int,
        nptr: *mut i64,
        unptr: *mut u64,
        maxlen: c_int,
        strict: bool,
        overflow: *mut bool,
    );

    fn nvim_set_cursor_col(col: c_int);
    fn nvim_gchar_cursor() -> c_int;
    fn del_char(fixpos: bool) -> c_int;
    #[link_name = "ins_str"]
    fn nvim_ins_str(ptr: *const std::ffi::c_char, len: usize);
    fn ins_char(c: c_int);

    #[link_name = "beep_flush"]
    fn nvim_beep_flush();

    // Low-level accessors used by inline scan port
    fn nvim_pos_get_lnum(pos: *const c_void) -> c_int;
    fn nvim_pos_get_col(pos: *const c_void) -> c_int;
    fn nvim_pos_get_coladd(pos: *const c_void) -> c_int;
    fn nvim_pos_set_coladd(pos: *mut c_void, v: c_int);
    fn nvim_ml_get(lnum: c_int) -> *const std::ffi::c_char;
    #[link_name = "ml_get_len"]
    fn nvim_ml_get_len(lnum: c_int) -> c_int;
    fn nvim_virtual_active() -> bool;
    fn nvim_get_cursor_col() -> c_int;
    fn utf_head_off(base: *const std::ffi::c_char, p: *const std::ffi::c_char) -> c_int;
    fn utfc_ptr2len(p: *const std::ffi::c_char) -> c_int;
}

/// Save the current window cursor to the addsub static (replaces C `nvim_addsub_save_cursor`).
///
/// # Safety
/// Must be called only from the Neovim main thread.
#[unsafe(export_name = "nvim_addsub_save_cursor")]
pub unsafe extern "C" fn rs_addsub_save_cursor() {
    let cur = nvim_window::win_struct::win_ref(nvim_get_curwin());
    ADDSUB_SAVED_CURSOR[0] = cur.w_cursor.lnum;
    ADDSUB_SAVED_CURSOR[1] = cur.w_cursor.col;
    ADDSUB_SAVED_CURSOR[2] = cur.w_cursor.coladd;
}

/// Restore the window cursor from the addsub static (replaces C `nvim_addsub_restore_cursor`).
///
/// # Safety
/// Must be called only from the Neovim main thread.
#[unsafe(export_name = "nvim_addsub_restore_cursor")]
pub unsafe extern "C" fn rs_addsub_restore_cursor() {
    let pos = ADDSUB_SAVED_CURSOR;
    nvim_curwin_set_cursor_from_pos(pos.as_ptr().cast());
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

// STR2NR constants (from charset.h)
const STR2NR_BIN: c_int = 1 << 0;
const STR2NR_OCT: c_int = 1 << 1;
const STR2NR_HEX: c_int = 1 << 2;
const MAXCOL: c_int = 0x7fff_ffff;

/// Rust port of `nvim_addsub_parse_number` from ops.c.
///
/// # Safety
/// - Accesses current buffer/cursor via C shims
/// - `scan_col` and `length` must be valid column values
#[allow(
    clippy::cast_sign_loss,
    clippy::suspicious_operation_groupings,
    clippy::too_many_arguments
)]
unsafe fn addsub_parse_number(
    scan_col: c_int,
    length: c_int,
    negative: c_int,
    do_hex: c_int,
    do_oct: c_int,
    do_bin: c_int,
    do_unsigned: c_int,
    do_blank: c_int,
    visual: c_int,
) -> AddsubParseResult {
    let cursor_lnum = nvim_get_cursor_lnum();
    let line_ptr = nvim_ml_get(cursor_lnum);
    let linelen = nvim_ml_get_len(cursor_lnum);

    let byte_at = |offset: c_int| -> c_int { c_int::from(*line_ptr.add(offset as usize) as u8) };

    let mut col = scan_col;
    let mut negative = negative;
    let mut length = length;

    // Check for minus sign before the digit (non-visual only)
    if visual == 0
        && col > 0
        && byte_at(col - 1) == i32::from(b'-')
        && utf_head_off(line_ptr, line_ptr.add((col - 1) as usize)) == 0
        && do_unsigned == 0
    {
        if do_blank != 0 && col >= 2 && !ascii_iswhite(byte_at(col - 2)) {
            // blank_unsigned case - handled by caller
        } else {
            col -= 1;
            negative = 1;
        }
    }

    let maxlen = if visual != 0 && nvim_get_VIsual_mode() != i32::from(b'V') {
        if nvim_get_b_visual_vi_curswant() == MAXCOL {
            linelen - col
        } else {
            length
        }
    } else {
        0
    };

    let mut pre: c_int = 0;
    let mut n: u64 = 0;
    let mut overflow = false;
    let what = if do_bin != 0 { STR2NR_BIN } else { 0 }
        | if do_oct != 0 { STR2NR_OCT } else { 0 }
        | if do_hex != 0 { STR2NR_HEX } else { 0 };
    vim_str2nr(
        line_ptr.add(col as usize),
        &raw mut pre,
        &raw mut length,
        what,
        std::ptr::null_mut(),
        &raw mut n,
        maxlen,
        false,
        &raw mut overflow,
    );

    // Ignore leading '-' for hex, octal and bin numbers
    if pre != 0 && negative != 0 {
        col += 1;
        length -= 1;
        negative = 0;
    }

    AddsubParseResult {
        pre,
        length,
        n_lo: n,
        col,
        negative,
        overflow: c_int::from(overflow),
    }
}

/// Inline equivalent of `CHAR_ORD(c)` macro from ascii_defs.h.
/// Returns 0-25 for 'a'-'z' or 'A'-'Z'.
#[inline]
#[allow(clippy::cast_sign_loss)]
fn char_ord(c: c_int) -> c_int {
    let byte = (c & 0xFF) as u8;
    if byte < b'a' {
        i32::from(byte.wrapping_sub(b'A'))
    } else {
        i32::from(byte.wrapping_sub(b'a'))
    }
}

/// Rust port of `nvim_addsub_do_alpha` from ops.c.
/// Increments or decrements an alphabetic character.
///
/// Returns `(did_change, endpos_col)`.
///
/// # Safety
/// - Modifies buffer content via del_char/ins_char
/// - Accesses cursor via C shims
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
unsafe fn addsub_do_alpha(
    col: c_int,
    firstdigit: c_int,
    op_type: c_int,
    prenum1: c_int,
) -> (bool, c_int) {
    let is_upper = (firstdigit as u8).is_ascii_uppercase();
    let mut fd = firstdigit;

    if op_type == OP_NR_SUB {
        if char_ord(fd) < prenum1 {
            fd = if is_upper {
                i32::from(b'A')
            } else {
                i32::from(b'a')
            };
        } else {
            fd -= prenum1;
        }
    } else if 26 - char_ord(fd) - 1 < prenum1 {
        fd = if is_upper {
            i32::from(b'Z')
        } else {
            i32::from(b'z')
        };
    } else {
        fd += prenum1;
    }

    nvim_set_cursor_col(col);
    del_char(false);
    ins_char(fd);
    let endpos_col = nvim_get_cursor_col();
    nvim_set_cursor_col(col);
    (true, endpos_col)
}

/// NUMBUFLEN: large enough for any number representation (65 bytes).
const NUMBUFLEN: usize = 65;

/// Persistent state: whether the last hex digit deleted was uppercase.
/// Mirrors the C static `bool hexupper = false` in nvim_addsub_replace_number.
static HEXUPPER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Rust port of `nvim_addsub_replace_number` from ops.c.
/// Deletes the old number and inserts the new formatted number.
///
/// Returns the end column position.
///
/// # Safety
/// - Modifies buffer content via del_char/ins_str
/// - Accesses cursor via C shims
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
unsafe fn addsub_replace_number(
    col: c_int,
    length: c_int,
    pre: c_int,
    n: u64,
    negative: bool,
    was_positive: bool,
    visual: bool,
    firstdigit: c_int,
    do_oct: c_int,
) -> c_int {
    nvim_set_cursor_col(col);
    let mut todel = length;
    let mut c = nvim_gchar_cursor();

    let mut length = length;
    // Don't include the '-' in the length
    if c == i32::from(b'-') {
        length -= 1;
    }
    while todel > 0 {
        todel -= 1;
        // Track whether hex digit was uppercase
        if c < 0x100 && (c as u8).is_ascii_alphabetic() {
            HEXUPPER.store(
                (c as u8).is_ascii_uppercase(),
                std::sync::atomic::Ordering::Relaxed,
            );
        }
        del_char(false);
        c = nvim_gchar_cursor();
    }

    // Build the output string in a fixed-size buffer
    let mut buf: [u8; NUMBUFLEN * 2 + 4] = [0; NUMBUFLEN * 2 + 4];
    let mut pos = 0usize;

    // Leading minus sign
    if negative && (!visual || was_positive) {
        buf[pos] = b'-';
        pos += 1;
    }
    // Prefix: '0' for oct/hex/bin, then 'b'/'B'/'x'/'X' for bin/hex
    if pre != 0 {
        buf[pos] = b'0';
        pos += 1;
        length -= 1;
    }
    if pre == i32::from(b'b')
        || pre == i32::from(b'B')
        || pre == i32::from(b'x')
        || pre == i32::from(b'X')
    {
        buf[pos] = pre as u8;
        pos += 1;
        length -= 1;
    }

    // Format the number into buf2
    let mut buf2 = [0u8; NUMBUFLEN];
    let buf2_slice = if pre == i32::from(b'b') || pre == i32::from(b'B') {
        // Binary format
        let mut bits: u32 = 64;
        while bits > 0 && (n >> (bits - 1)) & 1 == 0 {
            bits -= 1;
        }
        let mut out_len = 0usize;
        let mut b = bits;
        while b > 0 && out_len < NUMBUFLEN - 1 {
            b -= 1;
            buf2[out_len] = if (n >> b) & 1 != 0 { b'1' } else { b'0' };
            out_len += 1;
        }
        &buf2[..out_len]
    } else if pre == 0 {
        // Decimal
        let s = format!("{n}");
        let bytes = s.as_bytes();
        let len = bytes.len().min(NUMBUFLEN - 1);
        buf2[..len].copy_from_slice(&bytes[..len]);
        &buf2[..len]
    } else if pre == i32::from(b'0') {
        // Octal
        let s = format!("{n:o}");
        let bytes = s.as_bytes();
        let len = bytes.len().min(NUMBUFLEN - 1);
        buf2[..len].copy_from_slice(&bytes[..len]);
        &buf2[..len]
    } else if HEXUPPER.load(std::sync::atomic::Ordering::Relaxed) {
        // Hex uppercase
        let s = format!("{n:X}");
        let bytes = s.as_bytes();
        let len = bytes.len().min(NUMBUFLEN - 1);
        buf2[..len].copy_from_slice(&bytes[..len]);
        &buf2[..len]
    } else {
        // Hex lowercase
        let s = format!("{n:x}");
        let bytes = s.as_bytes();
        let len = bytes.len().min(NUMBUFLEN - 1);
        buf2[..len].copy_from_slice(&bytes[..len]);
        &buf2[..len]
    };
    let buf2len = buf2_slice.len() as c_int;
    length -= buf2len;

    // Zero-pad if needed (e.g., when original had leading zeros)
    if firstdigit == i32::from(b'0') && !(do_oct != 0 && pre == 0) {
        let mut pad = length;
        while pad > 0 {
            buf[pos] = b'0';
            pos += 1;
            pad -= 1;
        }
    }

    // Append buf2
    for &byte in buf2_slice {
        buf[pos] = byte;
        pos += 1;
    }

    // Insert the formatted string using nvim_ins_str
    nvim_ins_str(buf.as_ptr().cast(), pos);

    let endpos_col = nvim_get_cursor_col();
    if endpos_col > 0 {
        nvim_set_cursor_col(endpos_col - 1);
    }
    endpos_col
}

// Perform number arithmetic: parse, compute, replace.
// Returns the endpos column for mark setting.
#[allow(clippy::cast_sign_loss)]
unsafe fn do_number_addsub(scan: &AddsubScanResult, params: &NumArithParams) -> c_int {
    let parse = addsub_parse_number(
        scan.col,
        scan.length,
        scan.negative,
        params.do_hex,
        params.do_oct,
        params.do_bin,
        params.do_unsigned,
        params.blank_unsigned,
        params.visual_flag,
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

    let endpos_col = addsub_replace_number(
        col,
        adj_length,
        pre,
        n,
        negative,
        was_positive,
        visual,
        params.firstdigit,
        params.do_oct,
    );

    addsub_set_marks(col, endpos_col);
    endpos_col
}

/// Inline replacement of `nvim_addsub_set_marks` (C function deleted).
/// Sets b_op_start/b_op_end based on current cursor + provided columns.
unsafe fn addsub_set_marks(startpos_col: c_int, endpos_col: c_int) {
    if nvim_cmdmod_has_lockmarks() == 0 {
        nvim_curbuf_set_op_start_to_cursor_col(startpos_col);
        nvim_curbuf_set_op_end_to_cursor_col(endpos_col);
    }
}

/// Inline replacement of `nvim_addsub_cleanup` (C function deleted).
/// Restores cursor position or sets w_set_curswant.
unsafe fn addsub_cleanup(visual: c_int, did_change: c_int, save_coladd: c_int) {
    if visual != 0 {
        rs_addsub_restore_cursor();
    } else if did_change != 0 {
        nvim_curwin_set_w_set_curswant(true);
    } else if nvim_virtual_active() {
        nvim_curwin_set_cursor_coladd(save_coladd);
    }
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
    // === Inline nvim_addsub_setup ===
    rs_addsub_save_cursor();
    let visual_flag: c_int = nvim_VIsual_active();
    let save_coladd: c_int = if nvim_virtual_active() {
        let coladd = nvim_pos_get_coladd(pos);
        nvim_pos_set_coladd(pos, 0);
        coladd
    } else {
        0
    };
    nvim_curwin_set_cursor_from_pos(pos);
    let linelen = nvim_ml_get_len(nvim_pos_get_lnum(pos));

    // === Inline nvim_addsub_get_nrformats ===
    let do_hex: c_int = nvim_curbuf_nf_has(c_int::from(b'x'));
    let do_oct: c_int = nvim_curbuf_nf_has(c_int::from(b'o'));
    let do_bin: c_int = nvim_curbuf_nf_has(c_int::from(b'b'));
    let do_alpha: c_int = nvim_curbuf_nf_has(c_int::from(b'p'));
    let do_unsigned: c_int = nvim_curbuf_nf_has(c_int::from(b'u'));
    let do_blank: c_int = nvim_curbuf_nf_has(c_int::from(b'k'));

    let _ = linelen; // used by C setup for bounds checking; scan does its own

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
        addsub_cleanup(visual_flag, 0, save_coladd);
        return false;
    }

    if scan.no_digit != 0 {
        nvim_beep_flush();
        addsub_cleanup(visual_flag, 0, save_coladd);
        return false;
    }

    let did_change = if scan.is_alpha != 0 {
        let (changed, endpos_col) = addsub_do_alpha(scan.col, scan.firstdigit, op_type, prenum1);
        if changed {
            addsub_set_marks(scan.col, endpos_col);
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

    addsub_cleanup(visual_flag, c_int::from(did_change), save_coladd);
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
