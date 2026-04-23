//! op_addsub: dispatch wrapper for CTRL-A / CTRL-X operators.
//!
//! Implements `op_addsub`, the outer loop that drives `do_addsub` for
//! visual and non-visual increment/decrement.
//!
//! Migrated from ops.c `void op_addsub()`.

use crate::types::Pos;
use nvim_normal::types::OpargT;
use std::ffi::{c_char, c_int, c_long, c_void};

// =============================================================================
// Motion type constants (must match normal_defs.h)
// =============================================================================

const K_MT_BLOCK_WISE: c_int = 2;
const K_MT_LINE_WISE: c_int = 1;

// =============================================================================
// Return values
// =============================================================================

const FAIL: c_int = 0;

// =============================================================================
// C `struct block_def` mirror (register_defs.h)
// Must match C layout: 3 ints + ptr + 10 ints = 64 bytes on 64-bit Linux.
// =============================================================================

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

impl BlockDefC {
    fn zeroed() -> Self {
        // SAFETY: all-zero is valid for this POD C struct
        unsafe { std::mem::zeroed() }
    }
}

// =============================================================================
// C FFI declarations
// =============================================================================

#[allow(clashing_extern_declarations)]
extern "C" {
    // Undo
    fn u_save_cursor() -> c_int;
    fn u_save(top: c_int, bot: c_int) -> c_int;

    // Line content
    fn ml_get_len(lnum: c_int) -> c_int;

    // Block preparation (still in C for Phase 2; migrated in Phase 3)
    fn block_prep(oap: *mut c_void, bdp: *mut BlockDefC, lnum: c_int, is_del: bool);

    // Position decrement
    fn dec(lp: *mut c_void) -> c_int;

    // do_addsub (now in Rust, exported as "do_addsub")
    fn do_addsub(op_type: c_int, pos: *mut c_void, length: c_int, prenum1: c_int) -> bool;

    // Change notification
    fn changed_lines(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        lnum_end: c_int,
        extra: c_int,
        do_concealed: bool,
    );
    fn redraw_curbuf_later(update_type: c_int);

    // fold update disable/enable
    fn nvim_inc_disable_fold_update();
    fn nvim_dec_disable_fold_update();

    // cursor state
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);

    // VIsual state (direct global)
    static mut VIsual_active: bool;

    // curbuf global
    static mut curbuf: *mut c_void;

    fn nvim_excmds_curbuf_op_save(out_start: *mut u64, out_end: *mut u64);
    fn nvim_set_b_op_start(lnum: c_int, col: c_int, coladd: c_int);

    // CMOD_LOCKMARKS
    fn nvim_cmdmod_has_lockmarks() -> c_int;

    // p_report (threshold for displaying "N lines changed")
    static p_report: i64;

    // smsg for "N lines changed"
    fn smsg(priority: c_int, fmt: *const c_char, ...) -> c_int;
}

// =============================================================================
// Constants
// =============================================================================

/// UPD_INVERTED (from drawscreen.h)
const UPD_INVERTED: c_int = 14;

// =============================================================================
// op_addsub implementation
// =============================================================================

/// Dispatch CTRL-A / CTRL-X for visual and non-visual modes.
///
/// Mirrors C `void op_addsub(oparg_T *oap, linenr_T Prenum1, bool g_cmd)`.
///
/// # Safety
/// `oap` must be a valid non-null pointer to an initialized `oparg_T`.
///
/// # Export
/// Replaces the C `op_addsub` function.
#[unsafe(export_name = "op_addsub")]
pub unsafe extern "C" fn rs_op_addsub(oap: *mut OpargT, prenum1: c_int, g_cmd: bool) {
    let mut change_cnt: isize = 0;
    let mut amount: c_int = prenum1;

    // Postpone fold updates until after changed_lines()
    nvim_inc_disable_fold_update();

    if VIsual_active {
        // Visual mode: iterate over lines
        if u_save((*oap).start.lnum - 1, (*oap).end.lnum + 1) == FAIL {
            nvim_dec_disable_fold_update();
            return;
        }

        // Saved b_op_start for the first change (used at end for '[ mark).
        let mut startpos_packed: u64 = 0;
        let mut dummy_end: u64 = 0;
        let mut have_startpos = false;

        let mut pos = Pos {
            lnum: (*oap).start.lnum,
            col: 0,
            coladd: 0,
        };

        while pos.lnum <= (*oap).end.lnum {
            let length: c_int;

            if (*oap).motion_type == K_MT_BLOCK_WISE {
                // Visual block mode
                let mut bd = BlockDefC::zeroed();
                block_prep(oap.cast::<c_void>(), &raw mut bd, pos.lnum, false);
                pos.col = bd.textcol;
                length = bd.textlen;
            } else if (*oap).motion_type == K_MT_LINE_WISE {
                nvim_set_cursor_col(0);
                pos.col = 0;
                length = ml_get_len(pos.lnum);
            } else {
                // kMTCharWise
                if pos.lnum == (*oap).start.lnum && !(*oap).inclusive {
                    dec((&raw mut (*oap).end).cast::<c_void>());
                }
                let mut len = ml_get_len(pos.lnum);
                pos.col = 0;
                if pos.lnum == (*oap).start.lnum {
                    pos.col += (*oap).start.col;
                    len -= (*oap).start.col;
                }
                if pos.lnum == (*oap).end.lnum {
                    len = ml_get_len((*oap).end.lnum);
                    let end_len = len;
                    let capped = ((*oap).end.col).min(end_len - 1);
                    (*oap).end.col = capped;
                    len = capped - pos.col + 1;
                }
                length = len;
            }

            let one_change = do_addsub(
                (*oap).op_type,
                (&raw mut pos).cast::<c_void>(),
                length,
                amount,
            );
            if one_change {
                if change_cnt == 0 {
                    nvim_excmds_curbuf_op_save(&raw mut startpos_packed, &raw mut dummy_end);
                    have_startpos = true;
                }
                change_cnt += 1;
            }
            if g_cmd && one_change {
                amount += prenum1;
            }
            pos.lnum += 1;
        }

        nvim_dec_disable_fold_update();

        if change_cnt > 0 {
            changed_lines(curbuf, (*oap).start.lnum, 0, (*oap).end.lnum + 1, 0, true);
        }

        if change_cnt == 0 && (*oap).is_visual {
            // No change: remove Visual selection
            redraw_curbuf_later(UPD_INVERTED);
        }

        // Set '[ mark if something changed. Keep last end position from do_addsub().
        if change_cnt > 0 && nvim_cmdmod_has_lockmarks() == 0 && have_startpos {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let s_lnum = (startpos_packed >> 32) as c_int;
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let s_col = (startpos_packed as u32) as i32;
            nvim_set_b_op_start(s_lnum, s_col, 0);
        }

        #[allow(clippy::cast_possible_truncation)]
        if change_cnt > p_report as isize {
            #[allow(clippy::cast_possible_truncation)]
            let n = change_cnt as c_long;
            // NGETTEXT format -- both singular and plural say "lines changed"
            // (matches C: NGETTEXT("% lines changed", "% lines changed", n))
            smsg(0, c"%ld lines changed".as_ptr(), n);
        }
    } else {
        // Non-visual mode: single cursor position
        let mut pos = Pos {
            lnum: nvim_get_cursor_lnum(),
            col: nvim_get_cursor_col(),
            coladd: 0,
        };
        if u_save_cursor() == FAIL {
            nvim_dec_disable_fold_update();
            return;
        }
        let changed = do_addsub((*oap).op_type, (&raw mut pos).cast::<c_void>(), 0, amount);
        nvim_dec_disable_fold_update();
        if changed {
            changed_lines(curbuf, pos.lnum, 0, pos.lnum + 1, 0, true);
        }
    }
}
