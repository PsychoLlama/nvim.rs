//! Full `op_tilde` migration (Phase 1)
//!
//! Migrated from `op_tilde()`, `swapchars()`, and `swapchar()` in ops.c.
//! Handles the g~, gU, gu, g? operators (case toggle/upper/lower/rot13).

use nvim_normal::types::{OpargT, PosT};
use std::ffi::{c_char, c_int, c_long, c_void};

// -----------------------------------------------------------------------
// Constants (matching C values)
// -----------------------------------------------------------------------

const FAIL: c_int = 0;
const K_MT_BLOCK_WISE: c_int = 2; // kMTBlockWise
const K_MT_LINE_WISE: c_int = 1; // kMTLineWise
const UPD_INVERTED: c_int = 14; // from drawscreen.h
const K_EXTMARK_UNDO: c_int = 1; // kExtmarkUndo
const OP_ROT13: c_int = 15;
const OP_LOWER: c_int = 12;
const OP_UPPER: c_int = 11;

// -----------------------------------------------------------------------
// C `struct block_def` mirror (register_defs.h)
//
// All "bool" fields in C are `int`, NOT `bool`. textstart is `char *`.
// On 64-bit Linux: 3×int(12) + pad(4) + ptr(8) + 10×int(40) = 64 bytes.
// -----------------------------------------------------------------------

/// Mirror of C `struct block_def`.
#[repr(C)]
struct BlockDefC {
    startspaces: c_int,
    endspaces: c_int,
    textlen: c_int,
    textstart: *mut c_char, // natural alignment adds no padding on 64-bit (ptr follows 3 ints)
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

impl BlockDefC {
    fn zeroed() -> Self {
        // SAFETY: all-zero is valid for this POD C struct
        unsafe { std::mem::zeroed() }
    }
}

// -----------------------------------------------------------------------
// FFI declarations
// -----------------------------------------------------------------------

extern "C" {
    // Undo
    fn nvim_u_save(top: c_int, bot: c_int) -> c_int;

    // Block preparation
    fn block_prep(oap: *mut c_void, bdp: *mut BlockDefC, lnum: c_int, is_del: bool);

    // Position iteration
    fn inc(lp: *mut c_void) -> c_int;
    fn dec(lp: *mut c_void) -> c_int;

    // Line content
    fn ml_get_pos(pos: *const c_void) -> *mut c_char;
    fn ml_get_len(lnum: c_int) -> c_int;
    fn ml_get_pos_len(pos: *mut c_void) -> c_int;
    fn gchar_pos(pos: *const c_void) -> c_int;

    // Multibyte
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn mb_islower(c: c_int) -> bool;
    fn mb_isupper(c: c_int) -> bool;
    fn mb_toupper(c: c_int) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;

    // Buffer mutation (ASCII path)
    fn ml_get_buf_mut(buf: *mut c_void, lnum: c_int) -> *mut c_char;
    fn extmark_splice_cols(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        old_col: c_int,
        new_col: c_int,
        op: c_int,
    );

    // Buffer mutation (multibyte path -- requires cursor at position)
    fn del_bytes(count: c_int, fixpos: c_int, use_delcombine: c_int) -> c_int;
    fn ins_char(c: c_int);
    fn get_cursor_pos_ptr() -> *const c_char;

    // Cursor save/restore
    fn nvim_curwin_get_cursor_lnum() -> c_int;
    fn nvim_curwin_get_cursor_col() -> c_int;
    fn nvim_curwin_get_cursor_coladd() -> c_int;
    /// Set curwin->w_cursor = *pos (takes *const pos_T as *const c_void)
    fn nvim_curwin_set_cursor_from_pos(pos: *const c_void);

    // Change notifications
    fn changed_lines(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        lnum_end: c_int,
        added: c_int,
        do_buf_event: bool,
    );
    fn redraw_curbuf_later(update_type: c_int);

    // Marks / cmdmod
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_curbuf_set_op_start(lnum: c_int, col: c_int);
    fn nvim_curbuf_set_op_end(lnum: c_int, col: c_int);

    // Message
    fn smsg(priority: c_int, fmt: *const c_char, ...) -> c_int;

    // Globals
    static p_report: i64;
    static mut curbuf: *mut c_void;
    static curbuf_splice_pending: c_int;
}

// -----------------------------------------------------------------------
// ROT13 helper (mirrors C macro ROT13(c,base) = ((c-base+13)%26)+base)
// -----------------------------------------------------------------------

#[inline]
const fn rot13(c: c_int, base: c_int) -> c_int {
    ((c - base + 13) % 26) + base
}

// -----------------------------------------------------------------------
// `swapchar` -- single character case swap with buffer mutation
//
// Mirrors swapchar() in ops.c:963.
// Exported as `swapchar` so callers that previously used the C function
// (e.g. rs_n_swapchar in normal/src/lib.rs) continue to link.
// -----------------------------------------------------------------------

/// Public C-ABI wrapper for `swapchar` (replaces the deleted C function).
///
/// # Safety
/// `pos` must be a valid `*const pos_T`.
#[allow(clippy::must_use_candidate)]
#[unsafe(export_name = "swapchar")]
pub unsafe extern "C" fn rs_swapchar_export(op_type: c_int, pos: *const c_void) -> bool {
    swapchar(op_type, pos.cast_mut())
}

/// # Safety
/// `pos` must be a valid `*mut pos_T`.
unsafe fn swapchar(op_type: c_int, pos: *mut c_void) -> bool {
    let c = gchar_pos(pos);

    // Only do rot13 for ASCII characters.
    if c >= 0x80 && op_type == OP_ROT13 {
        return false;
    }

    let nc = if mb_islower(c) {
        if op_type == OP_ROT13 {
            rot13(c, c_int::from(b'a'))
        } else if op_type == OP_LOWER {
            c
        } else {
            mb_toupper(c)
        }
    } else if mb_isupper(c) {
        if op_type == OP_ROT13 {
            rot13(c, c_int::from(b'A'))
        } else if op_type == OP_UPPER {
            c
        } else {
            mb_tolower(c)
        }
    } else {
        c
    };

    if nc != c {
        if c >= 0x80 || nc >= 0x80 {
            // Multibyte path: save cursor, move to pos, del+ins, restore.
            let sp = PosT {
                lnum: nvim_curwin_get_cursor_lnum(),
                col: nvim_curwin_get_cursor_col(),
                coladd: nvim_curwin_get_cursor_coladd(),
            };
            // Set curwin->w_cursor = *pos
            nvim_curwin_set_cursor_from_pos(pos.cast());
            // Don't use del_char() -- it also removes composing chars
            del_bytes(utf_ptr2len(get_cursor_pos_ptr()), 0, 0);
            ins_char(nc);
            // Restore cursor
            nvim_curwin_set_cursor_from_pos((&raw const sp).cast());
        } else {
            // ASCII path: write byte in-place
            let pos_t = pos.cast::<PosT>();
            let lnum = (*pos_t).lnum;
            #[allow(clippy::cast_sign_loss)]
            let col = (*pos_t).col as usize;
            let line = ml_get_buf_mut(curbuf, lnum);
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_possible_wrap
            )]
            {
                *line.add(col) = nc as u8 as c_char;
            }
            if curbuf_splice_pending == 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                extmark_splice_cols(curbuf, lnum - 1, col as c_int, 1, 1, K_EXTMARK_UNDO);
            }
        }
        return true;
    }
    false
}

// -----------------------------------------------------------------------
// `swapchars` -- invoke swapchar on `length` bytes starting at `pos`
//
// Mirrors swapchars() in ops.c:936.
// pos is advanced past the changed characters.
// Returns true if some character was changed.
// -----------------------------------------------------------------------

/// # Safety
/// `pos` must be a valid `*mut pos_T`.
unsafe fn swapchars(op_type: c_int, pos: *mut c_void, length: c_int) -> bool {
    let mut did_change = false;
    let mut todo = length;
    while todo > 0 {
        let p = ml_get_pos(pos);
        let len = utfc_ptr2len(p);
        // counting bytes, not characters
        if len > 0 {
            todo -= len - 1;
        }
        did_change |= swapchar(op_type, pos);
        if inc(pos) == -1 {
            break; // at end of file
        }
        todo -= 1;
    }
    did_change
}

// -----------------------------------------------------------------------
// `op_tilde` -- full migration
// -----------------------------------------------------------------------

/// Full port of `op_tilde()` from ops.c (line 860).
///
/// # Safety
/// - `oap` must be a valid `oparg_T *`
/// - Accesses global state via C functions
#[unsafe(export_name = "op_tilde")]
pub unsafe extern "C" fn rs_op_tilde(oap: *mut c_void) {
    let oap_c: *mut OpargT = oap.cast();

    if nvim_u_save((*oap_c).start.lnum - 1, (*oap_c).end.lnum + 1) == FAIL {
        return;
    }

    let op_type = (*oap_c).op_type;
    let mut pos: PosT = (*oap_c).start;
    let mut did_change = false;

    if (*oap_c).motion_type == K_MT_BLOCK_WISE {
        // Visual block mode: iterate rows
        while pos.lnum <= (*oap_c).end.lnum {
            let mut bd = BlockDefC::zeroed();
            block_prep(oap, &raw mut bd, pos.lnum, false);
            pos.col = bd.textcol;
            let one_change = swapchars(op_type, (&raw mut pos).cast(), bd.textlen);
            did_change |= one_change;
            pos.lnum += 1;
        }
        if did_change {
            changed_lines(
                curbuf,
                (*oap_c).start.lnum,
                0,
                (*oap_c).end.lnum + 1,
                0,
                true,
            );
        }
    } else {
        // Charwise or linewise
        if (*oap_c).motion_type == K_MT_LINE_WISE {
            (*oap_c).start.col = 0;
            pos.col = 0;
            let line_len = ml_get_len((*oap_c).end.lnum);
            (*oap_c).end.col = if line_len > 0 { line_len - 1 } else { 0 };
        } else if !(*oap_c).inclusive {
            dec((&raw mut (*oap_c).end).cast::<c_void>());
        }

        if pos.lnum == (*oap_c).end.lnum {
            did_change = swapchars(
                op_type,
                (&raw mut pos).cast(),
                (*oap_c).end.col - pos.col + 1,
            );
        } else {
            loop {
                let length = if pos.lnum == (*oap_c).end.lnum {
                    (*oap_c).end.col + 1
                } else {
                    ml_get_pos_len((&raw mut pos).cast())
                };
                did_change |= swapchars(op_type, (&raw mut pos).cast(), length);
                // ltoreq(oap->end, pos) — end <= pos
                let end = (*oap_c).end;
                let ltoreq = end.lnum < pos.lnum || (end.lnum == pos.lnum && end.col <= pos.col);
                if ltoreq || inc((&raw mut pos).cast()) == -1 {
                    break;
                }
            }
        }

        if did_change {
            changed_lines(
                curbuf,
                (*oap_c).start.lnum,
                (*oap_c).start.col,
                (*oap_c).end.lnum + 1,
                0,
                true,
            );
        }
    }

    if !did_change && (*oap_c).is_visual {
        redraw_curbuf_later(UPD_INVERTED);
    }

    if nvim_cmdmod_has_lockmarks() == 0 {
        nvim_curbuf_set_op_start((*oap_c).start.lnum, (*oap_c).start.col);
        nvim_curbuf_set_op_end((*oap_c).end.lnum, (*oap_c).end.col);
    }

    let line_count = (*oap_c).line_count;
    if i64::from(line_count) > p_report {
        // Mirrors: smsg(0, NGETTEXT("%" PRId64 " line changed",
        //                           "%" PRId64 " lines changed", line_count),
        //              (int64_t)line_count)
        // We use %ld (c_long) which is i64 on Linux 64-bit.
        #[allow(clippy::cast_lossless)]
        let n = line_count as c_long;
        if line_count == 1 {
            smsg(0, c"%ld line changed".as_ptr(), n);
        } else {
            smsg(0, c"%ld lines changed".as_ptr(), n);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rot13_roundtrip() {
        for c in b'a'..=b'z' {
            let c = c_int::from(c);
            assert_eq!(rot13(rot13(c, c_int::from(b'a')), c_int::from(b'a')), c);
        }
        for c in b'A'..=b'Z' {
            let c = c_int::from(c);
            assert_eq!(rot13(rot13(c, c_int::from(b'A')), c_int::from(b'A')), c);
        }
    }

    #[test]
    fn test_rot13_values() {
        assert_eq!(
            rot13(c_int::from(b'a'), c_int::from(b'a')),
            c_int::from(b'n')
        );
        assert_eq!(
            rot13(c_int::from(b'n'), c_int::from(b'a')),
            c_int::from(b'a')
        );
        assert_eq!(
            rot13(c_int::from(b'A'), c_int::from(b'A')),
            c_int::from(b'N')
        );
    }
}
