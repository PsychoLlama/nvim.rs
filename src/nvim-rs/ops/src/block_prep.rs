//! block_prep / charwise_block_prep: block-mode operation preparation.
//!
//! Implements the low-level block geometry helpers used by all block-mode
//! operators (delete, yank, tilde, shift, replace, getregion, etc.).
//!
//! Migrated from ops.c:
//!   - `static bool reset_lbr()` → internal helper
//!   - `static void restore_lbr()` → internal helper
//!   - `void block_prep()` → exported as "block_prep"
//!   - `void charwise_block_prep()` → exported as "charwise_block_prep"

use crate::types::Pos;
use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Operator type constants (must match ops.h)
// =============================================================================

const OP_INSERT: c_int = 17;
const OP_APPEND: c_int = 18;
const OP_LSHIFT: c_int = 4;
const OP_REPLACE: c_int = 16;

// =============================================================================
// Misc constants
// =============================================================================

const MAXCOL: c_int = i32::MAX;

// =============================================================================
// C `struct block_def` mirror (register_defs.h)
//
// Layout on 64-bit Linux:
//   offset  0: startspaces (int, 4)
//   offset  4: endspaces (int, 4)
//   offset  8: textlen (int, 4)
//   offset 12: [4 bytes padding]
//   offset 16: textstart (char*, 8)
//   offset 24: textcol (int, 4)
//   offset 28: start_vcol (int, 4)
//   offset 32: end_vcol (int, 4)
//   offset 36: is_short (int, 4)
//   offset 40: is_MAX (int, 4)
//   offset 44: is_oneChar (int, 4)
//   offset 48: pre_whitesp (int, 4)
//   offset 52: pre_whitesp_c (int, 4)
//   offset 56: end_char_vcols (int, 4)
//   offset 60: start_char_vcols (int, 4)
//   total: 64 bytes
// =============================================================================

#[repr(C)]
pub struct BlockDefC {
    pub startspaces: c_int,
    pub endspaces: c_int,
    pub textlen: c_int,
    pub textstart: *mut c_char,
    pub textcol: c_int,
    pub start_vcol: c_int,
    pub end_vcol: c_int,
    pub is_short: c_int,
    pub is_max: c_int,     // C: is_MAX
    pub is_onechar: c_int, // C: is_oneChar
    pub pre_whitesp: c_int,
    pub pre_whitesp_c: c_int,
    pub end_char_vcols: c_int,
    pub start_char_vcols: c_int,
}

impl BlockDefC {
    /// Create a zeroed BlockDefC (valid for C POD struct).
    #[must_use]
    pub fn zeroed() -> Self {
        // SAFETY: all-zero bytes are valid for this C POD struct
        unsafe { std::mem::zeroed() }
    }
}

// =============================================================================
// StrCharInfo / CharInfo for charsize iteration (mirrors shift_full.rs)
// =============================================================================

/// Mirror of C `CharSize` from plines.h: {int width; int head;}.
#[repr(C)]
#[derive(Clone, Copy)]
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

/// Opaque buffer for `CharsizeArg` allocated on the Rust stack.
///
/// 320 bytes / 8-byte aligned — larger than any CharsizeArg seen in the
/// codebase (verified in shift_full.rs to be >= 264 bytes).
#[repr(C, align(8))]
struct CharsizeArgBuf([u8; 320]);

// =============================================================================
// C FFI declarations
// =============================================================================

#[allow(clashing_extern_declarations)]
extern "C" {
    // Line content
    fn ml_get(lnum: c_int) -> *mut c_char;
    fn ml_get_len(lnum: c_int) -> c_int;

    // Charsize iteration
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
    static utf8len_tab: [u8; 256];
    fn utf_ptr2CharInfo_impl(p: *const u8, len: usize) -> i32;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;

    // ASCII
    fn rs_ascii_iswhite(c: c_int) -> c_int;

    // curwin for linebreak and charsize
    static mut curwin: *mut c_void;

    // linebreak accessors
    fn nvim_curwin_get_p_lbr() -> c_int;
    fn nvim_curwin_reset_lbr();
    fn nvim_curwin_restore_lbr(saved: c_int);

    // getvcol for charwise_block_prep
    fn getvcol(
        wp: *mut c_void,
        pos: *const c_void,
        scol: *mut c_int,
        ccol: *mut c_int,
        ecol: *mut c_int,
    );

    // utf_head_off for charwise_block_prep
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // virtual_op global
    static mut virtual_op: c_int;
}

// =============================================================================
// Helpers: reset_lbr / restore_lbr (inlined from ops.c statics)
// =============================================================================

/// Reset 'linebreak' and return previous value.
/// Mirrors C `static bool reset_lbr()`.
#[inline]
unsafe fn reset_lbr() -> c_int {
    let saved = nvim_curwin_get_p_lbr();
    nvim_curwin_reset_lbr();
    saved
}

/// Restore 'linebreak' from saved value.
/// Mirrors C `static void restore_lbr(bool lbr_saved)`.
#[inline]
unsafe fn restore_lbr(saved: c_int) {
    nvim_curwin_restore_lbr(saved);
}

// =============================================================================
// Charsize helpers (mirrors shift_full.rs)
// =============================================================================

/// Call the appropriate charsize variant based on `cstype`.
/// `cstype = false` → fast; `cstype = true` → regular.
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

/// Inline `utf_ptr2StrCharInfo`.
#[inline]
unsafe fn utf_ptr2str_char_info(ptr: *mut c_char) -> StrCharInfo {
    let p = ptr.cast::<u8>();
    let first = *p;
    if first < 0x80 {
        StrCharInfo {
            ptr,
            chr: CharInfo {
                value: c_int::from(first),
                len: 1,
            },
        }
    } else {
        let len = utf8len_tab[first as usize] as usize;
        let code_point = utf_ptr2CharInfo_impl(p, len);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
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

/// Inline `utfc_next`.
#[inline]
#[allow(clippy::cast_sign_loss)]
unsafe fn utfc_next(cur: StrCharInfo) -> StrCharInfo {
    let first = *cur.ptr as u8;
    if first < 0x80 {
        let next_ptr = cur.ptr.add(1);
        let next_first = *next_ptr as u8;
        if next_first < 0x80 {
            return StrCharInfo {
                ptr: next_ptr,
                chr: CharInfo {
                    value: c_int::from(next_first),
                    len: 1,
                },
            };
        }
    }
    utfc_next_impl(cur)
}

// =============================================================================
// block_prep
// =============================================================================

/// Prepare block_def metadata for block-wise operations.
///
/// Mirrors C `void block_prep(oparg_T *oap, struct block_def *bdp, linenr_T lnum, bool is_del)`.
///
/// # Safety
/// - `oap` must be a valid non-null pointer to `oparg_T`
/// - `bdp` must be a valid non-null pointer to `BlockDefC` (C-compatible layout)
///
/// # Export
/// Replaces the C `block_prep` function.
#[unsafe(export_name = "block_prep")]
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_block_prep(
    oap: *mut OpargT,
    bdp: *mut BlockDefC,
    lnum: c_int,
    is_del: bool,
) {
    let mut incr: c_int = 0;
    // Avoid a problem with unwanted linebreaks in block mode.
    let lbr_saved = reset_lbr();

    (*bdp).startspaces = 0;
    (*bdp).endspaces = 0;
    (*bdp).textlen = 0;
    (*bdp).start_vcol = 0;
    (*bdp).end_vcol = 0;
    (*bdp).is_short = 0;
    (*bdp).is_onechar = 0;
    (*bdp).pre_whitesp = 0;
    (*bdp).pre_whitesp_c = 0;
    (*bdp).end_char_vcols = 0;
    (*bdp).start_char_vcols = 0;

    let line = ml_get(lnum);
    let mut prev_pstart = line;

    let mut csarg_buf = std::mem::MaybeUninit::<CharsizeArgBuf>::uninit();
    let csarg = csarg_buf.as_mut_ptr().cast::<c_void>();
    let mut cstype = init_charsize_arg(csarg, curwin, lnum, line);
    let mut ci = utf_ptr2str_char_info(line);
    let mut vcol = (*bdp).start_vcol;

    while vcol < (*oap).start_vcol && *ci.ptr != 0 {
        incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, csarg).width;
        vcol += incr;
        if rs_ascii_iswhite(ci.chr.value) != 0 {
            (*bdp).pre_whitesp += incr;
            (*bdp).pre_whitesp_c += 1;
        } else {
            (*bdp).pre_whitesp = 0;
            (*bdp).pre_whitesp_c = 0;
        }
        prev_pstart = ci.ptr;
        ci = utfc_next(ci);
    }
    (*bdp).start_vcol = vcol;
    let mut pstart = ci.ptr;

    (*bdp).start_char_vcols = incr;
    if (*bdp).start_vcol < (*oap).start_vcol {
        // line too short
        (*bdp).end_vcol = (*bdp).start_vcol;
        (*bdp).is_short = 1;
        if !is_del || (*oap).op_type == OP_APPEND {
            (*bdp).endspaces = (*oap).end_vcol - (*oap).start_vcol + 1;
        }
    } else {
        // notice: this converts partly selected Multibyte characters to
        // spaces, too.
        (*bdp).startspaces = (*bdp).start_vcol - (*oap).start_vcol;
        if is_del && (*bdp).startspaces != 0 {
            (*bdp).startspaces = (*bdp).start_char_vcols - (*bdp).startspaces;
        }
        let mut pend = pstart;
        (*bdp).end_vcol = (*bdp).start_vcol;

        if (*bdp).end_vcol > (*oap).end_vcol {
            // it's all in one character
            (*bdp).is_onechar = 1;
            if (*oap).op_type == OP_INSERT {
                (*bdp).endspaces = (*bdp).start_char_vcols - (*bdp).startspaces;
            } else if (*oap).op_type == OP_APPEND {
                (*bdp).startspaces += (*oap).end_vcol - (*oap).start_vcol + 1;
                (*bdp).endspaces = (*bdp).start_char_vcols - (*bdp).startspaces;
            } else {
                (*bdp).startspaces = (*oap).end_vcol - (*oap).start_vcol + 1;
                if is_del && (*oap).op_type != OP_LSHIFT {
                    // just putting the sum of those two into
                    // bdp->startspaces doesn't work for Visual replace,
                    // so we have to split the tab in two
                    (*bdp).startspaces =
                        (*bdp).start_char_vcols - ((*bdp).start_vcol - (*oap).start_vcol);
                    (*bdp).endspaces = (*bdp).end_vcol - (*oap).end_vcol - 1;
                }
            }
        } else {
            // Reinitialize csarg for end-column scan (mirrors C: same csarg reused).
            cstype = init_charsize_arg(csarg, curwin, lnum, line);
            ci = utf_ptr2str_char_info(pend);
            vcol = (*bdp).end_vcol;
            let mut prev_pend = pend;

            while vcol <= (*oap).end_vcol && *ci.ptr != 0 {
                prev_pend = ci.ptr;
                incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, csarg).width;
                vcol += incr;
                ci = utfc_next(ci);
            }
            (*bdp).end_vcol = vcol;
            pend = ci.ptr;

            if (*bdp).end_vcol <= (*oap).end_vcol
                && (!is_del || (*oap).op_type == OP_APPEND || (*oap).op_type == OP_REPLACE)
            {
                // line too short
                (*bdp).is_short = 1;
                if (*oap).op_type == OP_APPEND || virtual_op != 0 {
                    (*bdp).endspaces =
                        (*oap).end_vcol - (*bdp).end_vcol + c_int::from((*oap).inclusive);
                }
            } else if (*bdp).end_vcol > (*oap).end_vcol {
                (*bdp).endspaces = (*bdp).end_vcol - (*oap).end_vcol - 1;
                if !is_del && (*bdp).endspaces != 0 {
                    (*bdp).endspaces = incr - (*bdp).endspaces;
                    if pend != pstart {
                        pend = prev_pend;
                    }
                }
            }
        }

        (*bdp).end_char_vcols = incr;
        if is_del && (*bdp).startspaces != 0 {
            pstart = prev_pstart;
        }
        (*bdp).textlen = pend.offset_from(pstart) as c_int;
    }
    // textcol and textstart are set unconditionally after the if/else,
    // matching C: bdp->textcol = pstart - line; bdp->textstart = pstart;
    (*bdp).textcol = pstart.offset_from(line) as c_int;
    (*bdp).textstart = pstart;
    restore_lbr(lbr_saved);
}

// =============================================================================
// charwise_block_prep
// =============================================================================

/// Get block text from "start" to "end".
///
/// Mirrors C `void charwise_block_prep(pos_T start, pos_T end, struct block_def *bdp,
///                                      linenr_T lnum, bool inclusive)`.
///
/// # Safety
/// - `bdp` must be a valid non-null pointer to `BlockDefC`
///
/// # Export
/// Replaces the C `charwise_block_prep` function.
#[unsafe(export_name = "charwise_block_prep")]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_charwise_block_prep(
    start: Pos,
    end: Pos,
    bdp: *mut BlockDefC,
    lnum: c_int,
    inclusive: bool,
) {
    let mut startcol: c_int = 0;
    let mut endcol: c_int = MAXCOL;
    let p = ml_get(lnum);
    let plen = ml_get_len(lnum);

    (*bdp).startspaces = 0;
    (*bdp).endspaces = 0;
    (*bdp).is_onechar = 0;
    (*bdp).start_char_vcols = 0;

    if lnum == start.lnum {
        startcol = start.col;
        if virtual_op != 0 {
            let mut cs: c_int = 0;
            let mut ce: c_int = 0;
            getvcol(
                curwin,
                (&raw const start).cast::<c_void>(),
                &raw mut cs,
                std::ptr::null_mut(),
                &raw mut ce,
            );
            if ce != cs && start.coladd > 0 {
                // Part of a tab selected -- but don't double-count it.
                (*bdp).start_char_vcols = ce - cs + 1;
                (*bdp).startspaces = ((*bdp).start_char_vcols - start.coladd).max(0);
                startcol += 1;
            }
        }
    }

    if lnum == end.lnum {
        endcol = end.col;
        if virtual_op != 0 {
            let mut cs: c_int = 0;
            let mut ce: c_int = 0;
            getvcol(
                curwin,
                (&raw const end).cast::<c_void>(),
                &raw mut cs,
                std::ptr::null_mut(),
                &raw mut ce,
            );
            // p[endcol] == NUL or partial tab
            let p_at_endcol = if endcol < plen {
                *p.add(endcol as usize) as u8
            } else {
                0u8 // treat out-of-bounds as NUL
            };
            if p_at_endcol == 0
                || (cs + end.coladd < ce && utf_head_off(p, p.add(endcol as usize)) == 0)
            {
                if start.lnum == end.lnum && start.col == end.col {
                    // Special case: inside a single char
                    (*bdp).is_onechar = 1;
                    (*bdp).startspaces = end.coladd - start.coladd + c_int::from(inclusive);
                    endcol = startcol;
                } else {
                    (*bdp).endspaces = end.coladd + c_int::from(inclusive);
                    endcol -= c_int::from(inclusive);
                }
            }
        }
    }

    if endcol == MAXCOL {
        endcol = ml_get_len(lnum);
    }

    if startcol > endcol || (*bdp).is_onechar != 0 {
        (*bdp).textlen = 0;
    } else {
        (*bdp).textlen = endcol - startcol + c_int::from(inclusive);
    }
    (*bdp).textcol = startcol;
    (*bdp).textstart = if startcol <= plen {
        p.add(startcol as usize)
    } else {
        p
    };
}
