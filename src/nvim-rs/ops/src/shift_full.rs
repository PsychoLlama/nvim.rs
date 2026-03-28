//! Full `shift_block` migration (Phase 3)
//!
//! Migrated from `shift_block()` in ops.c (static function).
//! Handles block-visual indent shift (< and > in CTRL-V mode).
//!
//! Called from `op_shift` in C (which calls `rs_shift_block` after migration).

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]

use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_void};

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

const TAB: u8 = b'\t';
const OP_LSHIFT: c_int = 4; // matches ops.h OP_LSHIFT
const MODE_INSERT: c_int = 0x10; // state_defs.h
const K_EXTMARK_UNDO: c_int = 1; // kExtmarkUndo

// -----------------------------------------------------------------------
// Structs mirroring C types
// -----------------------------------------------------------------------

/// Mirror of C `CharSize` from plines.h: {int width; int head;}.
#[repr(C)]
struct CharSize {
    width: c_int,
    head: c_int,
}

/// Mirror of C `CharInfo` from mbyte_defs.h: {int32_t value; int len;}.
#[repr(C)]
#[derive(Clone, Copy)]
struct CharInfo {
    value: i32,
    len: c_int,
}

/// Mirror of C `StrCharInfo` from mbyte_defs.h: {char *ptr; CharInfo chr;}.
#[repr(C)]
#[derive(Clone, Copy)]
struct StrCharInfo {
    ptr: *mut c_char,
    chr: CharInfo,
}

/// Mirror of C `struct block_def` from register_defs.h.
/// All boolean fields in C are `int` in this struct.
#[repr(C)]
struct BlockDefC {
    startspaces: c_int,
    endspaces: c_int,
    textlen: c_int,
    textstart: *mut c_char,
    textcol: c_int,
    start_vcol: c_int,
    end_vcol: c_int,
    is_short: c_int,
    is_max: c_int,
    is_onechar: c_int,
    pre_whitesp: c_int,
    pre_whitesp_c: c_int,
    end_char_vcols: c_int,
    start_char_vcols: c_int,
}

/// Opaque buffer for `CharsizeArg` allocated on the Rust stack.
///
/// `CharsizeArg` contains a `MarkTreeIter` which holds a `MTNode*` pointer
/// and arrays up to `MT_MAX_DEPTH=20` entries. The computed C size on
/// 64-bit Linux is 264 bytes. We use 320 to give a comfortable margin.
/// Alignment must be at least 8 (for pointer fields).
#[repr(C, align(8))]
struct CharsizeArgBuf([u8; 320]);

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

#[allow(clashing_extern_declarations)]
extern "C" {
    // Block preparation (bdp declared as *mut c_void here; tilde_full uses *mut BlockDefC)
    fn block_prep(oap: *mut c_void, bdp: *mut c_void, lnum: c_int, is_del: bool);

    // Shiftwidth / tabstop
    fn get_sw_value_indent(buf: *mut c_void, left: bool) -> c_int;
    fn tabstop_fromto(
        start_col: c_int,
        end_col: c_int,
        ts_arg: c_int,
        vts: *const c_int,
        ntabs: *mut c_int,
        nspcs: *mut c_int,
    );

    // Line content
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_get_cursor_line_len() -> c_int;
    fn ml_replace(lnum: c_int, line: *mut c_char, copy: bool) -> c_int;
    fn changed_bytes(lnum: c_int, col: c_int);

    // Multibyte
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2CharInfo_impl(p: *const u8, len: usize) -> i32;
    static utf8len_tab: [u8; 256];
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;

    // Charsize
    fn init_charsize_arg(
        csarg: *mut c_void,
        wp: *mut c_void,
        lnum: c_int,
        line: *const c_char,
    ) -> bool;
    fn charsize_fast(
        csarg: *mut c_void,
        cur: *const c_char,
        vcol: c_int,
        cur_char: i32,
    ) -> CharSize;
    fn charsize_regular(
        csarg: *mut c_void,
        cur: *const c_char,
        vcol: c_int,
        cur_char: i32,
    ) -> CharSize;

    // ASCII test (Rust function, directly callable)
    fn rs_ascii_iswhite(c: c_int) -> c_int;

    // Extmarks
    fn extmark_splice_cols(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        old_col: c_int,
        new_col: c_int,
        op: c_int,
    );

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;

    // Cursor state
    fn nvim_curwin_get_cursor_lnum() -> c_int;
    fn nvim_curwin_get_cursor_col() -> c_int;
    fn nvim_curwin_set_cursor_col(col: c_int);

    // Option accessors
    fn nvim_curbuf_get_b_p_et() -> c_int;
    fn nvim_curbuf_get_b_p_ts() -> c_int;
    fn nvim_curbuf_get_b_p_vts_array() -> *const c_int;

    // Globals
    static mut State: c_int;
    static mut p_ri: c_int;
    static mut curbuf: *mut c_void;
    static mut curwin: *mut c_void;
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

/// Inline `win_charsize`: dispatch to fast or regular based on `cstype`.
/// `cstype` = `false` → fast; `true` → regular (mirrors C's `CSType` = `bool`,
/// `kCharsizeFast = 1 = true`, `kCharsizeRegular = 0 = false`).
#[inline]
unsafe fn win_charsize(
    cstype: bool,
    vcol: c_int,
    ptr: *const c_char,
    chr: i32,
    csarg: *mut c_void,
) -> CharSize {
    if cstype {
        charsize_regular(csarg, ptr, vcol, chr)
    } else {
        charsize_fast(csarg, ptr, vcol, chr)
    }
}

/// Inline `utf_ptr2StrCharInfo`: returns `{ptr, chr}` for the char at `ptr`.
#[inline]
unsafe fn utf_ptr2str_char_info(ptr: *mut c_char) -> StrCharInfo {
    let p = ptr.cast::<u8>();
    let first = *p;
    if first < 0x80 {
        StrCharInfo {
            ptr,
            chr: CharInfo {
                value: first as i32,
                len: 1,
            },
        }
    } else {
        let len = utf8len_tab[first as usize] as usize;
        let code_point = utf_ptr2CharInfo_impl(p, len);
        let (code_point, len) = if code_point < 0 {
            (code_point, 1)
        } else {
            (code_point, len as c_int)
        };
        StrCharInfo {
            ptr,
            chr: CharInfo {
                value: code_point,
                len,
            },
        }
    }
}

/// Inline `utfc_next`: advance to the next character in a string.
#[inline]
unsafe fn utfc_next(cur: StrCharInfo) -> StrCharInfo {
    // ASCII fast path (mirrors the inline in mbyte.h)
    let first = *cur.ptr as u8;
    if first < 0x80 {
        let next_ptr = cur.ptr.add(1);
        let next_first = *next_ptr as u8;
        if next_first < 0x80 {
            return StrCharInfo {
                ptr: next_ptr,
                chr: CharInfo {
                    value: next_first as i32,
                    len: 1,
                },
            };
        }
    }
    utfc_next_impl(cur)
}

/// Inline `MB_PTR_ADV`: advance pointer by one multibyte character.
#[inline]
unsafe fn mb_ptr_adv(p: *mut c_char) -> *mut c_char {
    p.add(utfc_ptr2len(p) as usize)
}

/// Inline `ascii_iswhite`: returns true if character is ASCII whitespace.
#[inline]
unsafe fn ascii_iswhite(c: c_int) -> bool {
    rs_ascii_iswhite(c) != 0
}

// -----------------------------------------------------------------------
// `shift_block` -- full migration (exported as `rs_shift_block`)
// -----------------------------------------------------------------------

/// Port of `shift_block()` from ops.c (line 168).
/// Called from `op_shift` in C as `rs_shift_block(oap, amount)`.
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - Accesses global state via C functions
#[unsafe(export_name = "rs_shift_block")]
pub unsafe extern "C" fn rs_shift_block(oap: *mut c_void, amount: c_int) {
    // Read oap->op_type to determine direction.
    // OpargT layout: first field is op_type (c_int at offset 0).
    let op_type = *(oap as *const c_int);
    let left = op_type == OP_LSHIFT;

    let oldstate = State;
    let oldcol = nvim_curwin_get_cursor_col();
    let sw_val = get_sw_value_indent(curbuf, left);
    let ts_val = nvim_curbuf_get_b_p_ts();
    let old_p_ri = p_ri;

    p_ri = 0; // don't want revins in indent
    State = MODE_INSERT; // don't want MODE_REPLACE for State

    let mut bd: BlockDefC = std::mem::zeroed();
    let cursor_lnum = nvim_curwin_get_cursor_lnum();
    block_prep(oap, (&raw mut bd).cast(), cursor_lnum, true);

    if bd.is_short != 0 {
        State = oldstate;
        p_ri = old_p_ri;
        return;
    }

    // total = number of screen columns to insert/remove
    let mut total = (amount as u32).wrapping_mul(sw_val as u32) as c_int;
    if sw_val != 0 && (total / sw_val) != amount {
        // multiplication overflow
        State = oldstate;
        p_ri = old_p_ri;
        return;
    }

    let oldp = nvim_get_cursor_line_ptr();
    let old_line_len = nvim_get_cursor_line_len();

    let startcol: c_int;
    let oldlen: c_int;
    let newlen: c_int;
    let newp: *mut c_char;

    if left {
        // Left-shift path
        let mut non_white = bd.textstart;

        if bd.startspaces != 0 {
            non_white = mb_ptr_adv(non_white);
        }

        let mut non_white_col = bd.start_vcol;

        let mut csarg_buf = std::mem::MaybeUninit::<CharsizeArgBuf>::uninit();
        let csarg_ptr = csarg_buf.as_mut_ptr().cast::<c_void>();
        let cstype = init_charsize_arg(csarg_ptr, curwin, cursor_lnum, bd.textstart);
        while ascii_iswhite(*non_white as c_int) {
            let incr = win_charsize(
                cstype,
                non_white_col,
                non_white,
                (*non_white as u8) as i32,
                csarg_ptr,
            )
            .width;
            non_white_col += incr;
            non_white = non_white.add(1);
        }

        // Read oap->start_vcol via OpargT struct (offset 64 on 64-bit Linux).
        let start_vcol = (*(oap as *const OpargT)).start_vcol;

        let block_space_width = non_white_col - start_vcol;
        let shift_amount = if block_space_width < total {
            block_space_width
        } else {
            total
        };
        let destination_col = non_white_col - shift_amount;

        // Find where to start copying verbatim
        let mut verbatim_copy_end = bd.textstart;
        let mut verbatim_copy_width = bd.start_vcol;

        if bd.startspaces != 0 {
            verbatim_copy_width -= bd.start_char_vcols;
        }

        let cstype2 = init_charsize_arg(csarg_ptr, curwin, 0, bd.textstart);
        let mut ci = utf_ptr2str_char_info(verbatim_copy_end);
        while verbatim_copy_width < destination_col {
            let incr = win_charsize(
                cstype2,
                verbatim_copy_width,
                ci.ptr,
                ci.chr.value,
                csarg_ptr,
            )
            .width;
            if verbatim_copy_width + incr > destination_col {
                break;
            }
            verbatim_copy_width += incr;
            ci = utfc_next(ci);
        }
        verbatim_copy_end = ci.ptr;

        let fill = destination_col - verbatim_copy_width;
        let fixedlen = verbatim_copy_end.offset_from(oldp) as c_int;

        let new_line_len =
            fixedlen + fill + (old_line_len - (non_white.offset_from(oldp) as c_int));

        newp = xmalloc((new_line_len as usize) + 1).cast();
        startcol = fixedlen;
        oldlen = bd.textcol + (non_white.offset_from(bd.textstart) as c_int) - fixedlen;
        newlen = fill;

        std::ptr::copy_nonoverlapping(oldp, newp, fixedlen as usize);
        std::ptr::write_bytes(newp.add(fixedlen as usize), b' ', fill as usize);

        // STRCPY(newp + fixedlen + fill, non_white)
        let rest_src = non_white;
        let rest_dst = newp.add((fixedlen + fill) as usize);
        let rest_len = (old_line_len as usize) - (non_white.offset_from(oldp) as usize);
        std::ptr::copy_nonoverlapping(rest_src, rest_dst, rest_len + 1); // +1 for NUL
    } else {
        // Right-shift path
        total += bd.pre_whitesp; // all virtual WS up to & incl a split TAB
        let mut ws_vcol = bd.start_vcol - bd.pre_whitesp;
        let old_textstart = bd.textstart;

        if bd.startspaces != 0 {
            if utfc_ptr2len(bd.textstart) == 1 {
                bd.textstart = bd.textstart.add(1);
            } else {
                ws_vcol = 0;
                bd.startspaces = 0;
            }
        }

        // Scan whitespace to accumulate total virtual columns
        let mut csarg_buf = std::mem::MaybeUninit::<CharsizeArgBuf>::uninit();
        let csarg_ptr = csarg_buf.as_mut_ptr().cast::<c_void>();
        let cstype = init_charsize_arg(csarg_ptr, curwin, cursor_lnum, bd.textstart);

        let mut ci = utf_ptr2str_char_info(bd.textstart);
        let mut vcol = bd.start_vcol;
        while ascii_iswhite(ci.chr.value) {
            let incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, csarg_ptr).width;
            ci = utfc_next(ci);
            total += incr;
            vcol += incr;
        }
        bd.textstart = ci.ptr;
        bd.start_vcol = vcol;

        let mut tabs: c_int = 0;
        let mut spaces: c_int = 0;
        if nvim_curbuf_get_b_p_et() == 0 {
            tabstop_fromto(
                ws_vcol,
                ws_vcol + total,
                ts_val,
                nvim_curbuf_get_b_p_vts_array(),
                &raw mut tabs,
                &raw mut spaces,
            );
        } else {
            spaces = total;
        }

        // If splitting a TAB, allow for it
        let col_pre = bd.pre_whitesp_c - (bd.startspaces != 0) as c_int;
        bd.textcol -= col_pre;

        let new_line_len =
            bd.textcol + tabs + spaces + (old_line_len - (bd.textstart.offset_from(oldp) as c_int));

        newp = xmalloc((new_line_len as usize) + 1).cast();
        std::ptr::copy_nonoverlapping(oldp, newp, bd.textcol as usize);

        startcol = bd.textcol;
        oldlen = (bd.textstart.offset_from(old_textstart) as c_int) + col_pre;
        newlen = tabs + spaces;

        // Write tabs then spaces then rest of line
        std::ptr::write_bytes(newp.add(bd.textcol as usize), TAB, tabs as usize);
        std::ptr::write_bytes(
            newp.add((bd.textcol + tabs) as usize),
            b' ',
            spaces as usize,
        );
        // STRCPY(newp + bd.textcol + tabs + spaces, bd.textstart)
        let rest_src = bd.textstart;
        let rest_dst = newp.add((bd.textcol + tabs + spaces) as usize);
        let rest_len = (old_line_len as usize) - (bd.textstart.offset_from(oldp) as usize);
        std::ptr::copy_nonoverlapping(rest_src, rest_dst, rest_len + 1); // +1 for NUL
    }

    // Replace the line
    ml_replace(cursor_lnum, newp, false);
    changed_bytes(cursor_lnum, bd.textcol);
    extmark_splice_cols(
        curbuf,
        cursor_lnum - 1,
        startcol,
        oldlen,
        newlen,
        K_EXTMARK_UNDO,
    );

    State = oldstate;
    nvim_curwin_set_cursor_col(oldcol);
    p_ri = old_p_ri;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charsize_arg_buf_size() {
        // CharsizeArgBuf must be at least as large as the C CharsizeArg struct.
        // We calculate the maximum expected size is 264 bytes on 64-bit Linux.
        // Our buffer is 320 bytes, which gives a comfortable safety margin.
        assert!(std::mem::size_of::<CharsizeArgBuf>() >= 264);
        assert_eq!(std::mem::align_of::<CharsizeArgBuf>(), 8);
    }

    #[test]
    fn test_str_char_info_layout() {
        // StrCharInfo layout: ptr(8) + value(4) + len(4) = 16 bytes
        assert_eq!(std::mem::size_of::<StrCharInfo>(), 16);
        assert_eq!(std::mem::align_of::<StrCharInfo>(), 8);
    }
}
