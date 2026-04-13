//! Full `do_pending_operator` migration
//!
//! Absorbs all nvim_dpo_* C helpers into Rust (Phase 1).
//! The DPO statics (dpo_redo_VIsual, dpo_include_line_break, dpo_saved_lbr,
//! dpo_saved_old_cursor) are moved to Rust as static mut values.

use nvim_normal::types::{CmdargT, OpargT};
use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Operator type constants
// =============================================================================

const OP_NOP: c_int = 0;
const OP_DELETE: c_int = 1;
const OP_YANK: c_int = 2;
const OP_CHANGE: c_int = 3;
const OP_LSHIFT: c_int = 4;
const OP_RSHIFT: c_int = 5;
const OP_FILTER: c_int = 6;
const OP_TILDE: c_int = 7;
const OP_INDENT: c_int = 8;
const OP_FORMAT: c_int = 9;
const OP_COLON: c_int = 10;
const OP_UPPER: c_int = 11;
const OP_LOWER: c_int = 12;
const OP_JOIN: c_int = 13;
const OP_JOIN_NS: c_int = 14;
const OP_ROT13: c_int = 15;
const OP_REPLACE: c_int = 16;
const OP_INSERT: c_int = 17;
const OP_APPEND: c_int = 18;
const OP_FOLD: c_int = 19;
const OP_FOLDOPEN: c_int = 20;
const OP_FOLDOPENREC: c_int = 21;
const OP_FOLDCLOSE: c_int = 22;
const OP_FOLDCLOSEREC: c_int = 23;
const OP_FOLDDEL: c_int = 24;
const OP_FOLDDELREC: c_int = 25;
const OP_FORMAT2: c_int = 26;
const OP_FUNCTION: c_int = 27;
const OP_NR_ADD: c_int = 28;
const OP_NR_SUB: c_int = 29;

const K_MT_LINE_WISE: c_int = 1;
const K_MT_BLOCK_WISE: c_int = 2;
const K_MT_CHAR_WISE: c_int = 0;

const OK: c_int = 1;
const NUL: c_int = 0;
const MAXCOL: c_int = i32::MAX;

// Key codes
const K_LUA: c_int = 0xd102; // KS_EXTRA << 8 | KE_LUA -- matches keycodes.h
const K_COMMAND: c_int = 0xd107; // KS_EXTRA << 8 | KE_COMMAND
const CPO_YANK: c_int = b'y' as c_int;
const CPO_REDO: c_int = b'r' as c_int;
const CPO_EMPTYREGION: c_int = b'E' as c_int;
const CPO_FILTER: c_int = b'F' as c_int;

const CA_NO_ADJ_OP_END: c_int = 0x01;
const CA_COMMAND_BUSY: c_int = 0x02;

// =============================================================================
// DPO file-scope statics (moved from C ops.c)
// =============================================================================

/// Fields of the redo_VIsual_T struct.
#[derive(Copy, Clone)]
struct RedoVIsual {
    mode: c_int,       // 'v', 'V', or Ctrl-V
    line_count: c_int, // number of lines
    vcol: c_int,       // number of cols or end column
    count: c_int,      // count for Visual operator
    arg: c_int,        // extra argument
}

static mut DPO_REDO_VISUAL: RedoVIsual = RedoVIsual {
    mode: NUL,
    line_count: 0,
    vcol: 0,
    count: 0,
    arg: 0,
};
static mut DPO_INCLUDE_LINE_BREAK: bool = false;
static mut DPO_SAVED_LBR: c_int = 0;
/// Saved cursor: lnum, col, coladd
static mut DPO_SAVED_CURSOR: (c_int, c_int, c_int) = (0, 0, 0);

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // VIsual state
    fn nvim_dpo_get_VIsual_active() -> bool;
    fn nvim_set_VIsual_active(val: bool);
    fn nvim_get_VIsual_mode() -> c_int;
    fn nvim_set_VIsual_mode(val: c_int);
    fn nvim_get_VIsual_select() -> bool;
    fn nvim_set_VIsual_select(val: bool);
    fn nvim_set_VIsual_reselect(val: bool);
    fn nvim_get_VIsual_lnum() -> c_int;
    fn nvim_get_VIsual_col() -> c_int;
    fn nvim_set_VIsual_col(col: c_int);

    // finish_op / redo_VIsual_busy / bangredo
    fn nvim_dpo_get_finish_op() -> bool;
    fn nvim_get_redo_VIsual_busy() -> bool;
    fn nvim_set_redo_VIsual_busy(val: bool);
    fn nvim_set_bangredo(val: bool);

    // repeat_cmdline / repeat_luaref
    fn nvim_get_repeat_luaref() -> c_int;

    // restart_edit
    static mut restart_edit: c_int;
    // p_sol
    fn nvim_dpo_get_p_sol() -> bool;

    // curwin->w_p_lbr
    fn nvim_curwin_get_p_lbr() -> c_int;
    fn nvim_curwin_reset_lbr();
    fn nvim_curwin_restore_lbr(saved: c_int);

    // validate_virtcol
    fn nvim_dpo_validate_virtcol();
    fn nvim_get_curwin_w_virtcol() -> c_int;
    fn nvim_get_curswant() -> c_int;
    fn nvim_set_curswant(val: c_int);
    fn nvim_curwin_set_curswant_flag(val: bool);

    // cursor
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_set_cursor_lnum(lnum: c_int);
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_get_cursor_coladd() -> c_int;
    fn nvim_set_cursor_pos(lnum: c_int, col: c_int, coladd: c_int);

    fn nvim_oap_set_start_from_cursor(oap: *mut OpargT);
    fn nvim_oap_set_end_from_cursor(oap: *mut OpargT);
    fn nvim_oap_set_start_from_VIsual(oap: *mut OpargT);
    fn nvim_oap_start_zero_col_if_linewise(oap: *mut OpargT);
    fn nvim_VIsual_set_from_oap_start(oap: *mut OpargT);
    fn nvim_oap_end_is_NUL(oap: *mut OpargT) -> bool;
    fn nvim_gchar_pos_oap_end(oap: *mut OpargT) -> c_int;
    fn nvim_equalpos_oap(oap: *mut OpargT) -> bool;
    fn nvim_utfc_ptr2len_oap_end(oap: *mut OpargT) -> c_int;
    fn nvim_lt_oap_start_cursor(oap: *mut OpargT) -> bool;
    fn nvim_cursor_set_oap_start(oap: *mut OpargT);

    // VIsual save/restore
    fn nvim_dpo_save_visual_state();

    // resel_VIsual
    fn nvim_get_resel_VIsual_mode() -> c_int;
    fn nvim_get_resel_VIsual_vcol() -> c_int;
    fn nvim_get_resel_VIsual_line_count() -> c_int;
    fn nvim_set_resel_VIsual_mode(val: c_int);
    fn nvim_set_resel_VIsual_vcol(val: c_int);
    fn nvim_set_resel_VIsual_line_count(val: c_int);

    // fold / vcol
    fn nvim_hasFolding_oap_start_up(oap: *mut OpargT) -> bool;
    fn nvim_hasFolding_cursor_end_of_fold() -> bool;
    fn nvim_hasFolding_cursor_start_of_fold() -> bool;
    fn nvim_hasFolding_oap_start_down(oap: *mut OpargT) -> bool;
    fn nvim_check_pos_oap_end(oap: *mut OpargT);
    fn nvim_set_virtual_op_from_active();
    fn nvim_getvvcol_oap_end(oap: *mut OpargT);
    fn nvim_getvvcol_oap_start(oap: *mut OpargT);
    fn nvim_mark_mb_adjustpos_oap_end(oap: *mut OpargT);
    fn nvim_getvvcol_oap_start_both(oap: *mut OpargT);
    fn nvim_getvvcol_oap_end_both(oap: *mut OpargT, start_out: *mut c_int, end_out: *mut c_int);
    fn nvim_coladvance_set_oap_end(oap: *mut OpargT);
    fn nvim_coladvance_set_oap_start(oap: *mut OpargT);
    fn nvim_get_curwin_w_view_width() -> c_int;
    fn nvim_coladvance(col: c_int);
    fn nvim_get_cursor_line_len() -> c_int;
    fn ml_get_len(lnum: c_int) -> c_int;
    fn nvim_curbuf_get_ml_line_count() -> c_int;
    fn nvim_get_virtual_op() -> c_int;
    fn nvim_set_virtual_op_none();

    // misc ops state
    fn setmouse();
    fn nvim_set_mouse_dragging(val: c_int);
    fn nvim_coladvance_set_curswant(old_col: c_int);
    fn nvim_set_motion_force(val: c_int);

    // p_cpo / p_sel / options
    fn nvim_vim_strchr_p_cpo(c: c_int) -> bool;
    fn nvim_p_sel_is_exclusive() -> bool;
    fn nvim_p_sel_is_old() -> bool;

    // redraw
    fn nvim_redraw_curbuf_inverted();
    fn nvim_curbuf_modifiable() -> bool;

    // op utilities
    fn op_on_lines(op: c_int) -> c_int;
    fn inindent(extra: c_int) -> c_int;
    fn get_op_char(op_type: c_int) -> c_int;
    fn get_extra_op_char(op_type: c_int) -> c_int;

    // redo
    fn rs_prep_redo(
        regname: c_int,
        num: c_int,
        cmd1: c_int,
        cmd2: c_int,
        cmd3: c_int,
        cmd4: c_int,
        cmd5: c_int,
    );
    fn rs_prep_redo_num2(
        regname: c_int,
        num1: c_int,
        cmd1: c_int,
        cmd2: c_int,
        num2: c_int,
        cmd3: c_int,
        cmd4: c_int,
        cmd5: c_int,
    );
    fn AppendToRedobuffLit(s: *const c_char, len: c_int);
    fn AppendToRedobuffSpec(s: *const c_char);
    fn AppendToRedobuff(s: *const c_char);
    fn AppendNumberToRedobuff(n: c_int);
    fn ResetRedobuff();
    fn CancelRedo();
    fn xfree(ptr: *mut c_void);
    static mut repeat_cmdline: *mut c_char;

    // rs_* Rust helpers already implemented in Rust
    fn rs_restore_visual_mode();
    fn rs_unadjust_for_sel() -> bool;
    fn rs_clearop(oap: *mut OpargT);
    fn rs_clearopbeep(oap: *mut OpargT);
    fn rs_may_clear_cmdline();
    fn rs_foldCreate(wp: *mut c_void, start_lnum: c_int, end_lnum: c_int);
    fn rs_deleteFold(wp: *mut c_void, start: c_int, end: c_int, recursive: bool, had_visual: bool);
    fn rs_opFoldRange(
        first_lnum: c_int,
        last_lnum: c_int,
        opening: c_int,
        recurse: c_int,
        had_visual: bool,
    );

    // operators (still in C)
    fn op_delete(oap: *mut OpargT) -> c_int;
    fn op_yank(oap: *mut OpargT, message: bool);
    fn op_change(oap: *mut OpargT) -> c_int;
    fn op_tilde(oap: *mut OpargT);
    fn op_insert(oap: *mut OpargT, count1: c_int);
    fn op_replace(oap: *mut OpargT, c: c_int) -> c_int;
    fn op_addsub(oap: *mut OpargT, prenum1: c_int, g_cmd: bool);
    fn op_format(oap: *mut OpargT, keep_cursor: bool);
    fn op_formatexpr(oap: *mut OpargT);
    fn op_reindent(oap: *mut OpargT, get_indent: *const c_void);
    fn do_join(
        count: usize,
        join_spaces: bool,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
    ) -> c_int;
    fn auto_format(trailblank: bool, prev_line: bool);
    fn check_cursor_col(wp: *mut c_void);
    fn u_save_cursor() -> c_int;

    // indent helpers
    fn get_expr_indent() -> c_int;
    fn get_c_indent() -> c_int;
    fn get_lisp_indent() -> c_int;

    // misc
    fn nvim_vim_beep_operator();
    #[link_name = "beep_flush"]
    fn nvim_beep_flush();
    fn nvim_dpo_join_would_overflow(line_count: c_int) -> bool;
    fn nvim_sync_curbuf_last_changedtick_i();
    fn nvim_get_curbuf_b_p_lisp() -> bool;
    fn nvim_get_curbuf_b_p_fex_nonempty() -> bool;
    fn nvim_get_curbuf_b_p_fp_nonempty() -> bool;
    fn nvim_get_p_fp_nonempty() -> bool;
    fn nvim_get_curbuf_b_p_inde_nonempty() -> bool;
    #[link_name = "use_indentexpr_for_lisp"]
    fn nvim_use_indentexpr_for_lisp() -> bool;
    #[link_name = "has_format_option"]
    fn nvim_has_format_option(opt: c_int) -> bool;
    fn nvim_get_KeyTyped() -> bool;

    // curwin handle for fold calls
    fn nvim_dpo_get_curwin() -> *mut c_void;
}

// =============================================================================
// NL_STR constant
// =============================================================================

static NL_STR: &[u8] = b"\n\0";

// =============================================================================
// Private helpers
// =============================================================================

/// Check if cap->cmdchar is an ex-command character (':' or K_COMMAND).
#[inline]
unsafe fn is_ex_cmdchar(cap: *mut c_void) -> bool {
    let ch = (*cap.cast::<CmdargT>()).cmdchar;
    ch == c_int::from(b':') || ch == K_COMMAND
}

/// Reset 'linebreak'. Returns previous value for restore_lbr.
#[inline]
unsafe fn reset_lbr() -> c_int {
    let saved = nvim_curwin_get_p_lbr();
    nvim_curwin_reset_lbr();
    saved
}

/// Restore 'linebreak' from saved value.
#[inline]
unsafe fn restore_lbr(saved: c_int) {
    nvim_curwin_restore_lbr(saved);
}

/// Port of `get_op_vcol` -- calculate start/end virtual columns for block mode.
unsafe fn get_op_vcol(oap: *mut OpargT, redo_vcol: c_int, initial: bool) {
    let vis_mode = nvim_get_VIsual_mode();
    // Ctrl-V = 22
    if vis_mode != 22
        || (!initial && (*oap.cast::<OpargT>()).end.col < nvim_get_curwin_w_view_width())
    {
        return;
    }

    (*oap.cast::<OpargT>()).motion_type = K_MT_BLOCK_WISE;
    nvim_mark_mb_adjustpos_oap_end(oap);

    nvim_getvvcol_oap_start_both(oap);

    if !nvim_get_redo_VIsual_busy() {
        let mut start: c_int = 0;
        let mut end: c_int = 0;
        nvim_getvvcol_oap_end_both(oap, &raw mut start, &raw mut end);

        let start_vcol = (*oap.cast::<OpargT>()).start_vcol;
        let new_start = start_vcol.min(start);
        (*oap.cast::<OpargT>()).start_vcol = new_start;

        let end_vcol = (*oap.cast::<OpargT>()).end_vcol;
        if end > end_vcol {
            if initial && nvim_p_sel_is_exclusive() && start >= 1 && start > end_vcol {
                (*oap.cast::<OpargT>()).end_vcol = start - 1;
            } else {
                (*oap.cast::<OpargT>()).end_vcol = end;
            }
        }
    }

    // if '$' was used, get oap->end_vcol from longest line
    let curswant = nvim_get_curswant();
    if curswant == MAXCOL {
        nvim_set_cursor_col(MAXCOL);
        (*oap.cast::<OpargT>()).end_vcol = 0;
        let start_lnum = (*oap.cast::<OpargT>()).start.lnum;
        let end_lnum = (*oap.cast::<OpargT>()).end.lnum;
        let mut lnum = start_lnum;
        while lnum <= end_lnum {
            nvim_set_cursor_lnum(lnum);
            // getvvcol on cursor for end_vcol
            let mut e: c_int = 0;
            // Use a compound: place cursor then get vcol
            nvim_getvvcol_oap_end_both(oap, std::ptr::null_mut(), &raw mut e);
            let current_end_vcol = (*oap.cast::<OpargT>()).end_vcol;
            (*oap.cast::<OpargT>()).end_vcol = current_end_vcol.max(e);
            lnum += 1;
        }
    } else if nvim_get_redo_VIsual_busy() {
        let start_vcol = (*oap.cast::<OpargT>()).start_vcol;
        (*oap.cast::<OpargT>()).end_vcol = start_vcol + redo_vcol - 1;
    }

    nvim_coladvance_set_oap_end(oap);
    nvim_coladvance_set_oap_start(oap);
}

// =============================================================================
// Phase 1: should_process
// =============================================================================

unsafe fn dpo_should_process(cap: *mut c_void) -> bool {
    // Save state for postamble/restore_lbr
    DPO_SAVED_LBR = nvim_curwin_get_p_lbr();
    DPO_SAVED_CURSOR = (
        nvim_get_cursor_lnum(),
        nvim_get_cursor_col(),
        nvim_get_cursor_coladd(),
    );
    let oap = (*cap.cast::<CmdargT>()).oap;
    let op_type = (*oap.cast::<OpargT>()).op_type;
    (nvim_dpo_get_finish_op() || nvim_dpo_get_VIsual_active()) && op_type != OP_NOP
}

// =============================================================================
// Phase 1: preamble
// =============================================================================

#[allow(clippy::too_many_lines)]
unsafe fn dpo_preamble(cap: *mut c_void, gui_yank: bool) {
    let oap = (*cap.cast::<CmdargT>()).oap;
    let mut include_line_break = false;
    let redo_yank = nvim_vim_strchr_p_cpo(CPO_YANK) && !gui_yank;

    reset_lbr();
    (*oap.cast::<OpargT>()).is_visual = nvim_dpo_get_VIsual_active();

    // Handle motion_force
    let motion_force = (*oap.cast::<OpargT>()).motion_force;
    if motion_force == c_int::from(b'V') {
        (*oap.cast::<OpargT>()).motion_type = K_MT_LINE_WISE;
    } else if motion_force == c_int::from(b'v') {
        let mt = (*oap.cast::<OpargT>()).motion_type;
        if mt == K_MT_LINE_WISE {
            (*oap.cast::<OpargT>()).inclusive = false;
        } else if mt == K_MT_CHAR_WISE {
            let inc = (*oap.cast::<OpargT>()).inclusive;
            (*oap.cast::<OpargT>()).inclusive = !inc;
        }
        (*oap.cast::<OpargT>()).motion_type = K_MT_CHAR_WISE;
    } else if motion_force == 22 {
        // Ctrl_V
        if !nvim_dpo_get_VIsual_active() {
            nvim_set_VIsual_active(true);
            nvim_VIsual_set_from_oap_start(oap);
        }
        nvim_set_VIsual_mode(22); // Ctrl_V
        nvim_set_VIsual_select(false);
        nvim_set_VIsual_reselect(false);
    }

    // Prep redo
    let op_type = (*oap.cast::<OpargT>()).op_type;
    let cmdchar = (*cap.cast::<CmdargT>()).cmdchar;
    let vis_active = nvim_dpo_get_VIsual_active();
    let should_prep_redo = (redo_yank || op_type != OP_YANK)
        && ((!vis_active || motion_force != NUL)
            || ((is_ex_cmdchar(cap) || cmdchar == K_LUA) && op_type != OP_COLON))
        && cmdchar != c_int::from(b'D')
        && op_type != OP_FOLD
        && op_type != OP_FOLDOPEN
        && op_type != OP_FOLDOPENREC
        && op_type != OP_FOLDCLOSE
        && op_type != OP_FOLDCLOSEREC
        && op_type != OP_FOLDDEL
        && op_type != OP_FOLDDELREC;

    if should_prep_redo {
        let regname = (*oap.cast::<OpargT>()).regname;
        let count0 = (*cap.cast::<CmdargT>()).count0;
        let nchar = (*cap.cast::<CmdargT>()).nchar;
        rs_prep_redo(
            regname,
            count0,
            get_op_char(op_type),
            get_extra_op_char(op_type),
            motion_force,
            cmdchar,
            nchar,
        );
        if cmdchar == c_int::from(b'/') || cmdchar == c_int::from(b'?') {
            if !nvim_vim_strchr_p_cpo(CPO_REDO) {
                let searchbuf = (*cap.cast::<CmdargT>()).searchbuf;
                AppendToRedobuffLit(searchbuf, -1);
            }
            AppendToRedobuff(NL_STR.as_ptr().cast::<c_char>());
        } else if is_ex_cmdchar(cap) {
            if repeat_cmdline.is_null() {
                ResetRedobuff();
            } else {
                if cmdchar == c_int::from(b':') {
                    AppendToRedobuffLit(repeat_cmdline, -1);
                } else {
                    AppendToRedobuffSpec(repeat_cmdline);
                }
                AppendToRedobuff(NL_STR.as_ptr().cast::<c_char>());
                xfree(repeat_cmdline.cast::<c_void>());
                repeat_cmdline = std::ptr::null_mut();
            }
        } else if cmdchar == K_LUA {
            AppendNumberToRedobuff(nvim_get_repeat_luaref());
            AppendToRedobuff(NL_STR.as_ptr().cast::<c_char>());
        }
    }

    // Handle redo_VIsual_busy or VIsual_active
    if nvim_get_redo_VIsual_busy() {
        nvim_oap_set_start_from_cursor(oap);
        let rv_mode = DPO_REDO_VISUAL.mode;
        let rv_vcol = DPO_REDO_VISUAL.vcol;
        let rv_line_count = DPO_REDO_VISUAL.line_count;
        let new_lnum =
            (nvim_get_cursor_lnum() + rv_line_count - 1).min(nvim_curbuf_get_ml_line_count());
        nvim_set_cursor_lnum(new_lnum);
        nvim_set_VIsual_mode(rv_mode);
        if rv_vcol == MAXCOL || rv_mode == c_int::from(b'v') {
            if rv_mode == c_int::from(b'v') {
                if rv_line_count <= 1 {
                    nvim_dpo_validate_virtcol();
                    let new_curswant = nvim_get_curwin_w_virtcol() + rv_vcol - 1;
                    nvim_set_curswant(new_curswant);
                } else {
                    nvim_set_curswant(rv_vcol);
                }
            } else {
                nvim_set_curswant(MAXCOL);
            }
            nvim_coladvance(nvim_get_curswant());
        }
        let rv_count = DPO_REDO_VISUAL.count;
        (*cap.cast::<CmdargT>()).count0 = rv_count;
        (*cap.cast::<CmdargT>()).count1 = if rv_count == 0 { 1 } else { rv_count };
    } else if nvim_dpo_get_VIsual_active() {
        if !gui_yank {
            nvim_dpo_save_visual_state();
            rs_restore_visual_mode();
        }

        let vis_mode = nvim_get_VIsual_mode();
        if nvim_get_VIsual_select() && vis_mode == c_int::from(b'V') && op_type != OP_DELETE {
            let vis_lnum = nvim_get_VIsual_lnum();
            let cursor_lnum = nvim_get_cursor_lnum();
            if vis_lnum < cursor_lnum
                || (vis_lnum == cursor_lnum && nvim_get_VIsual_col() < nvim_get_cursor_col())
            {
                nvim_set_VIsual_col(0);
                nvim_set_cursor_col(ml_get_len(cursor_lnum));
            } else {
                nvim_set_cursor_col(0);
                nvim_set_VIsual_col(ml_get_len(nvim_get_VIsual_lnum()));
            }
            nvim_set_VIsual_mode(c_int::from(b'v'));
        } else if vis_mode == c_int::from(b'v') {
            include_line_break = rs_unadjust_for_sel();
        }

        nvim_oap_set_start_from_VIsual(oap);
        nvim_oap_start_zero_col_if_linewise(oap);
    }

    DPO_INCLUDE_LINE_BREAK = include_line_break;
}

// =============================================================================
// Phase 1: setup_positions
// =============================================================================

#[allow(clippy::too_many_lines)]
unsafe fn dpo_setup_positions(cap: *mut c_void, gui_yank: bool) {
    let oap = (*cap.cast::<CmdargT>()).oap;
    let lbr_saved = nvim_curwin_get_p_lbr();
    let redo_yank = nvim_vim_strchr_p_cpo(CPO_YANK) && !gui_yank;
    let include_line_break = DPO_INCLUDE_LINE_BREAK;

    // Set oap->start/end and fold handling
    if nvim_lt_oap_start_cursor(oap) {
        if !nvim_dpo_get_VIsual_active() {
            if nvim_hasFolding_oap_start_up(oap) {
                (*oap.cast::<OpargT>()).start.col = 0;
            }
            let cursor_col = nvim_get_cursor_col();
            let inclusive = (*oap.cast::<OpargT>()).inclusive;
            let mt = (*oap.cast::<OpargT>()).motion_type;
            if (cursor_col > 0 || inclusive || mt == K_MT_LINE_WISE)
                && nvim_hasFolding_cursor_end_of_fold()
            {
                nvim_set_cursor_col(nvim_get_cursor_line_len());
            }
        }
        nvim_oap_set_end_from_cursor(oap);
        nvim_cursor_set_oap_start(oap);
    } else {
        if !nvim_dpo_get_VIsual_active() && (*oap.cast::<OpargT>()).motion_type == K_MT_LINE_WISE {
            if nvim_hasFolding_cursor_start_of_fold() {
                nvim_set_cursor_col(0);
            }
            if nvim_hasFolding_oap_start_down(oap) {
                (*oap.cast::<OpargT>()).start.col = ml_get_len((*oap.cast::<OpargT>()).start.lnum);
            }
        }
        // oap->end = oap->start; oap->start = cursor
        let sl = (*oap.cast::<OpargT>()).start.lnum;
        let sc = (*oap.cast::<OpargT>()).start.col;
        let sca = (*oap.cast::<OpargT>()).start.coladd;
        (*oap.cast::<OpargT>()).end.lnum = sl;
        (*oap.cast::<OpargT>()).end.col = sc;
        (*oap.cast::<OpargT>()).end.coladd = sca;
        nvim_oap_set_start_from_cursor(oap);
    }

    nvim_check_pos_oap_end(oap);
    let line_count = (*oap.cast::<OpargT>()).end.lnum - (*oap.cast::<OpargT>()).start.lnum + 1;
    (*oap.cast::<OpargT>()).line_count = line_count;
    nvim_set_virtual_op_from_active();

    let vis_active = nvim_dpo_get_VIsual_active();
    let redo_busy = nvim_get_redo_VIsual_busy();
    if vis_active || redo_busy {
        let rv_vcol = DPO_REDO_VISUAL.vcol;
        get_op_vcol(oap, rv_vcol, true);

        if !redo_busy && !gui_yank {
            nvim_set_resel_VIsual_mode(nvim_get_VIsual_mode());
            let curswant = nvim_get_curswant();
            if curswant == MAXCOL {
                nvim_set_resel_VIsual_vcol(MAXCOL);
            } else {
                let vis_mode = nvim_get_VIsual_mode();
                if vis_mode != 22 {
                    // not Ctrl_V
                    nvim_getvvcol_oap_end(oap);
                }
                let end_vcol = (*oap.cast::<OpargT>()).end_vcol;
                let line_count2 = (*oap.cast::<OpargT>()).line_count;
                if vis_mode == 22 || line_count2 <= 1 {
                    if vis_mode != 22 {
                        nvim_getvvcol_oap_start(oap);
                    }
                    let start_vcol = (*oap.cast::<OpargT>()).start_vcol;
                    nvim_set_resel_VIsual_vcol(end_vcol - start_vcol + 1);
                } else {
                    nvim_set_resel_VIsual_vcol(end_vcol);
                }
            }
            nvim_set_resel_VIsual_line_count((*oap.cast::<OpargT>()).line_count);
        }

        // Redo visual prep
        let motion_force = (*oap.cast::<OpargT>()).motion_force;
        let op_type = (*oap.cast::<OpargT>()).op_type;
        let cmdchar = (*cap.cast::<CmdargT>()).cmdchar;
        let nchar = (*cap.cast::<CmdargT>()).nchar;
        if (redo_yank || op_type != OP_YANK)
            && op_type != OP_COLON
            && op_type != OP_FOLD
            && op_type != OP_FOLDOPEN
            && op_type != OP_FOLDOPENREC
            && op_type != OP_FOLDCLOSE
            && op_type != OP_FOLDCLOSEREC
            && op_type != OP_FOLDDEL
            && op_type != OP_FOLDDELREC
            && motion_force == NUL
        {
            if cmdchar == c_int::from(b'g')
                && (nchar == c_int::from(b'n') || nchar == c_int::from(b'N'))
            {
                let regname = (*oap.cast::<OpargT>()).regname;
                let count0 = (*cap.cast::<CmdargT>()).count0;
                rs_prep_redo(
                    regname,
                    count0,
                    get_op_char(op_type),
                    get_extra_op_char(op_type),
                    motion_force,
                    cmdchar,
                    nchar,
                );
            } else if !is_ex_cmdchar(cap) && cmdchar != K_LUA {
                let opchar = get_op_char(op_type);
                let extra_opchar = get_extra_op_char(op_type);
                let mut nchar2 = if op_type == OP_REPLACE { nchar } else { NUL };
                if nchar2 == -1 {
                    // REPLACE_CR_NCHAR
                    nchar2 = c_int::from(b'\r');
                } else if nchar2 == -2 {
                    // REPLACE_NL_NCHAR
                    nchar2 = c_int::from(b'\n');
                }
                let regname = (*oap.cast::<OpargT>()).regname;
                if opchar == c_int::from(b'g') && extra_opchar == c_int::from(b'@') {
                    let count0 = (*cap.cast::<CmdargT>()).count0;
                    rs_prep_redo_num2(
                        regname,
                        0,
                        NUL,
                        c_int::from(b'v'),
                        count0,
                        opchar,
                        extra_opchar,
                        nchar2,
                    );
                } else {
                    rs_prep_redo(
                        regname,
                        0,
                        NUL,
                        c_int::from(b'v'),
                        opchar,
                        extra_opchar,
                        nchar2,
                    );
                }
            }
            if !redo_busy {
                DPO_REDO_VISUAL.mode = nvim_get_resel_VIsual_mode();
                DPO_REDO_VISUAL.vcol = nvim_get_resel_VIsual_vcol();
                DPO_REDO_VISUAL.line_count = nvim_get_resel_VIsual_line_count();
                DPO_REDO_VISUAL.count = (*cap.cast::<CmdargT>()).count0;
                DPO_REDO_VISUAL.arg = (*cap.cast::<CmdargT>()).arg;
            }
        }

        // Inclusive/motion_type adjustments
        let motion_force2 = (*oap.cast::<OpargT>()).motion_force;
        let mt = (*oap.cast::<OpargT>()).motion_type;
        if motion_force2 == NUL || mt == K_MT_LINE_WISE {
            (*oap.cast::<OpargT>()).inclusive = true;
        }
        let vis_mode = nvim_get_VIsual_mode();
        if vis_mode == c_int::from(b'V') {
            (*oap.cast::<OpargT>()).motion_type = K_MT_LINE_WISE;
        } else if vis_mode == c_int::from(b'v') {
            (*oap.cast::<OpargT>()).motion_type = K_MT_CHAR_WISE;
            if nvim_oap_end_is_NUL(oap) && (include_line_break || nvim_get_virtual_op() == 0) {
                (*oap.cast::<OpargT>()).inclusive = false;
                let op_type2 = (*oap.cast::<OpargT>()).op_type;
                if !nvim_p_sel_is_old()
                    && op_on_lines(op_type2) == 0
                    && (*oap.cast::<OpargT>()).end.lnum < nvim_curbuf_get_ml_line_count()
                {
                    let end_lnum = (*oap.cast::<OpargT>()).end.lnum;
                    (*oap.cast::<OpargT>()).end.lnum = end_lnum + 1;
                    (*oap.cast::<OpargT>()).end.col = 0;
                    (*oap.cast::<OpargT>()).end.coladd = 0;
                    let lc = (*oap.cast::<OpargT>()).line_count;
                    (*oap.cast::<OpargT>()).line_count = lc + 1;
                }
            }
        }

        nvim_set_redo_VIsual_busy(false);

        if !gui_yank {
            nvim_set_VIsual_active(false);
            setmouse();
            nvim_set_mouse_dragging(0);
            rs_may_clear_cmdline();
            let op_type3 = (*oap.cast::<OpargT>()).op_type;
            let motion_force3 = (*oap.cast::<OpargT>()).motion_force;
            if (op_type3 == OP_YANK
                || op_type3 == OP_COLON
                || op_type3 == OP_FUNCTION
                || op_type3 == OP_FILTER)
                && motion_force3 == NUL
            {
                restore_lbr(lbr_saved);
                nvim_redraw_curbuf_inverted();
            }
        }
    }

    // Include trailing multi-byte byte
    if (*oap.cast::<OpargT>()).inclusive {
        let l = nvim_utfc_ptr2len_oap_end(oap);
        if l > 1 {
            let col = (*oap.cast::<OpargT>()).end.col;
            (*oap.cast::<OpargT>()).end.col = col + l - 1;
        }
    }
    // curwin->w_set_curswant = true
    nvim_curwin_set_curswant_flag(true);

    // empty check
    let mt = (*oap.cast::<OpargT>()).motion_type;
    let inclusive = (*oap.cast::<OpargT>()).inclusive;
    let op_type4 = (*oap.cast::<OpargT>()).op_type;
    let virtual_op = nvim_get_virtual_op();
    let start_coladd = (*oap.cast::<OpargT>()).start.coladd;
    let end_coladd = (*oap.cast::<OpargT>()).end.coladd;
    let empty = mt != K_MT_LINE_WISE
        && (!inclusive || (op_type4 == OP_YANK && nvim_gchar_pos_oap_end(oap) == NUL))
        && nvim_equalpos_oap(oap)
        && !(virtual_op != 0 && start_coladd != end_coladd);
    (*oap.cast::<OpargT>()).empty = empty;

    // Force redraw for empty visual region
    let is_visual = (*oap.cast::<OpargT>()).is_visual;
    let op_type5 = (*oap.cast::<OpargT>()).op_type;
    if is_visual && (empty || !nvim_curbuf_modifiable() || op_type5 == OP_FOLD) {
        restore_lbr(lbr_saved);
        nvim_redraw_curbuf_inverted();
    }

    // Adjust end for column-one case
    let retval = (*cap.cast::<CmdargT>()).retval;
    let mt2 = (*oap.cast::<OpargT>()).motion_type;
    let inclusive2 = (*oap.cast::<OpargT>()).inclusive;
    let is_visual2 = (*oap.cast::<OpargT>()).is_visual;
    let end_col = (*oap.cast::<OpargT>()).end.col;
    let line_count3 = (*oap.cast::<OpargT>()).line_count;
    if mt2 == K_MT_CHAR_WISE
        && !inclusive2
        && (retval & CA_NO_ADJ_OP_END) == 0
        && end_col == 0
        && (!is_visual2 || nvim_p_sel_is_old())
        && line_count3 > 1
    {
        (*oap.cast::<OpargT>()).end_adjusted = true;
        let lc = (*oap.cast::<OpargT>()).line_count;
        (*oap.cast::<OpargT>()).line_count = lc - 1;
        let el = (*oap.cast::<OpargT>()).end.lnum;
        (*oap.cast::<OpargT>()).end.lnum = el - 1;
        if inindent(0) != 0 {
            (*oap.cast::<OpargT>()).motion_type = K_MT_LINE_WISE;
        } else {
            let new_end_col = ml_get_len((*oap.cast::<OpargT>()).end.lnum);
            (*oap.cast::<OpargT>()).end.col = new_end_col;
            if new_end_col > 0 {
                (*oap.cast::<OpargT>()).end.col = new_end_col - 1;
                (*oap.cast::<OpargT>()).inclusive = true;
            }
        }
    } else {
        (*oap.cast::<OpargT>()).end_adjusted = false;
    }
}

// =============================================================================
// Phase 1: dispatch_operator
// =============================================================================

#[allow(clippy::too_many_lines)]
unsafe fn dpo_dispatch_operator(cap: *mut c_void, gui_yank: bool) {
    let oap = (*cap.cast::<CmdargT>()).oap;
    let lbr_saved = nvim_curwin_get_p_lbr();
    let empty_region_error =
        (*oap.cast::<OpargT>()).empty && nvim_vim_strchr_p_cpo(CPO_EMPTYREGION);

    let op_type = (*oap.cast::<OpargT>()).op_type;

    match op_type {
        OP_LSHIFT | OP_RSHIFT => {
            let count1 = if (*oap.cast::<OpargT>()).is_visual {
                (*cap.cast::<CmdargT>()).count1
            } else {
                1
            };
            crate::op_shift::rs_op_shift(oap.cast(), 1, count1);
            auto_format(false, true);
        }

        OP_JOIN_NS | OP_JOIN => {
            let lc = (*oap.cast::<OpargT>()).line_count;
            let lc = lc.max(2);
            (*oap.cast::<OpargT>()).line_count = lc;
            if nvim_dpo_join_would_overflow(lc) {
                nvim_beep_flush();
            } else {
                #[allow(clippy::cast_sign_loss)]
                do_join(lc as usize, op_type == OP_JOIN, true, true, true);
                auto_format(false, true);
            }
        }

        OP_DELETE => {
            nvim_set_VIsual_reselect(false);
            if empty_region_error {
                nvim_vim_beep_operator();
                CancelRedo();
            } else {
                op_delete(oap);
                let mt = (*oap.cast::<OpargT>()).motion_type;
                if mt == K_MT_LINE_WISE
                    && nvim_has_format_option(c_int::from(b'a'))
                    && u_save_cursor() == OK
                {
                    auto_format(false, true);
                }
            }
        }

        OP_YANK => {
            if empty_region_error {
                if !gui_yank {
                    nvim_vim_beep_operator();
                    CancelRedo();
                }
            } else {
                restore_lbr(lbr_saved);
                let cmdchar = (*cap.cast::<CmdargT>()).cmdchar;
                (*oap.cast::<OpargT>()).excl_tr_ws = cmdchar == c_int::from(b'z');
                op_yank(oap, !gui_yank);
            }
            check_cursor_col(nvim_dpo_get_curwin());
        }

        OP_CHANGE => {
            nvim_set_VIsual_reselect(false);
            if empty_region_error {
                nvim_vim_beep_operator();
                CancelRedo();
            } else {
                let restart_edit_save = if nvim_get_KeyTyped() { 0 } else { restart_edit };
                restart_edit = 0;
                restore_lbr(lbr_saved);
                nvim_sync_curbuf_last_changedtick_i();
                if op_change(oap) != 0 {
                    let rv = (*cap.cast::<CmdargT>()).retval;
                    (*cap.cast::<CmdargT>()).retval = rv | CA_COMMAND_BUSY;
                }
                if restart_edit == 0 {
                    restart_edit = restart_edit_save;
                }
            }
        }

        OP_FILTER => {
            if nvim_vim_strchr_p_cpo(CPO_FILTER) {
                // AppendToRedobuff("!\r")
                static FILTER_STR: &[u8] = b"!\r\0";
                AppendToRedobuff(FILTER_STR.as_ptr().cast::<c_char>());
            } else {
                nvim_set_bangredo(true);
            }
            // FALLTHROUGH to OP_INDENT/OP_COLON
            dpo_dispatch_indent_colon(oap, gui_yank);
        }

        OP_INDENT | OP_COLON => {
            dpo_dispatch_indent_colon(oap, gui_yank);
        }

        OP_TILDE | OP_UPPER | OP_LOWER | OP_ROT13 => {
            if empty_region_error {
                nvim_vim_beep_operator();
                CancelRedo();
            } else {
                op_tilde(oap);
            }
            check_cursor_col(nvim_dpo_get_curwin());
        }

        OP_FORMAT => {
            if nvim_get_curbuf_b_p_fex_nonempty() {
                op_formatexpr(oap);
            } else if nvim_get_p_fp_nonempty() || nvim_get_curbuf_b_p_fp_nonempty() {
                // op_colon(oap)
                dpo_dispatch_op_colon(oap);
            } else {
                op_format(oap, false);
            }
        }

        OP_FORMAT2 => {
            op_format(oap, true);
        }

        OP_FUNCTION => {
            // Save and restore dpo_redo_VIsual around op_function
            let saved = DPO_REDO_VISUAL;
            restore_lbr(lbr_saved);
            crate::op_function::rs_op_function_impl(oap);
            DPO_REDO_VISUAL = saved;
        }

        OP_INSERT | OP_APPEND => {
            nvim_set_VIsual_reselect(false);
            if empty_region_error {
                nvim_vim_beep_operator();
                CancelRedo();
            } else {
                let re_save = restart_edit;
                restart_edit = 0;
                restore_lbr(lbr_saved);
                nvim_sync_curbuf_last_changedtick_i();
                let count1 = (*cap.cast::<CmdargT>()).count1;
                op_insert(oap, count1);
                nvim_curwin_reset_lbr();
                auto_format(false, true);
                if restart_edit == 0 {
                    restart_edit = re_save;
                } else {
                    let rv = (*cap.cast::<CmdargT>()).retval;
                    (*cap.cast::<CmdargT>()).retval = rv | CA_COMMAND_BUSY;
                }
            }
        }

        OP_REPLACE => {
            nvim_set_VIsual_reselect(false);
            if empty_region_error {
                nvim_vim_beep_operator();
                CancelRedo();
            } else {
                restore_lbr(lbr_saved);
                let nchar = (*cap.cast::<CmdargT>()).nchar;
                op_replace(oap, nchar);
            }
        }

        OP_FOLD => {
            nvim_set_VIsual_reselect(false);
            rs_foldCreate(
                nvim_dpo_get_curwin(),
                (*oap.cast::<OpargT>()).start.lnum,
                (*oap.cast::<OpargT>()).end.lnum,
            );
        }

        OP_FOLDOPEN | OP_FOLDOPENREC | OP_FOLDCLOSE | OP_FOLDCLOSEREC => {
            nvim_set_VIsual_reselect(false);
            rs_opFoldRange(
                (*oap.cast::<OpargT>()).start.lnum,
                (*oap.cast::<OpargT>()).end.lnum,
                c_int::from(op_type == OP_FOLDOPEN || op_type == OP_FOLDOPENREC),
                c_int::from(op_type == OP_FOLDOPENREC || op_type == OP_FOLDCLOSEREC),
                (*oap.cast::<OpargT>()).is_visual,
            );
        }

        OP_FOLDDEL | OP_FOLDDELREC => {
            nvim_set_VIsual_reselect(false);
            rs_deleteFold(
                nvim_dpo_get_curwin(),
                (*oap.cast::<OpargT>()).start.lnum,
                (*oap.cast::<OpargT>()).end.lnum,
                op_type == OP_FOLDDELREC,
                (*oap.cast::<OpargT>()).is_visual,
            );
        }

        OP_NR_ADD | OP_NR_SUB => {
            if empty_region_error {
                nvim_vim_beep_operator();
                CancelRedo();
            } else {
                nvim_set_VIsual_active(true);
                restore_lbr(lbr_saved);
                let count1 = (*cap.cast::<CmdargT>()).count1;
                let rv_arg = DPO_REDO_VISUAL.arg;
                op_addsub(oap, count1, rv_arg != 0);
                nvim_set_VIsual_active(false);
            }
            check_cursor_col(nvim_dpo_get_curwin());
        }

        _ => {
            rs_clearopbeep(oap);
        }
    }
}

/// OP_INDENT / OP_COLON dispatch (shared with FILTER fallthrough).
#[inline]
unsafe fn dpo_dispatch_indent_colon(oap: *mut OpargT, _gui_yank: bool) {
    let op_type = (*oap.cast::<OpargT>()).op_type;
    if op_type == OP_INDENT {
        if nvim_get_curbuf_b_p_lisp() {
            if nvim_use_indentexpr_for_lisp() {
                op_reindent(oap, get_expr_indent as *const c_void);
            } else {
                op_reindent(oap, get_lisp_indent as *const c_void);
            }
            return;
        }
        let fn_ptr = if nvim_get_curbuf_b_p_inde_nonempty() {
            get_expr_indent as *const c_void
        } else {
            get_c_indent as *const c_void
        };
        op_reindent(oap, fn_ptr);
        return;
    }
    dpo_dispatch_op_colon(oap);
}

/// op_colon(oap) -- stuffs ':' command into read buffer.
unsafe fn dpo_dispatch_op_colon(oap: *mut OpargT) {
    crate::op_colon::rs_op_colon_impl(oap);
}

// =============================================================================
// Phase 1: postamble
// =============================================================================

unsafe fn dpo_postamble(cap: *mut c_void, old_col: c_int, gui_yank: bool) {
    let oap = (*cap.cast::<CmdargT>()).oap;

    nvim_set_virtual_op_none();
    if gui_yank {
        let (lnum, col, coladd) = DPO_SAVED_CURSOR;
        nvim_set_cursor_pos(lnum, col, coladd);
    } else {
        let op_type = (*oap.cast::<OpargT>()).op_type;
        let mt = (*oap.cast::<OpargT>()).motion_type;
        let end_adjusted = (*oap.cast::<OpargT>()).end_adjusted;
        if !nvim_dpo_get_p_sol()
            && mt == K_MT_LINE_WISE
            && !end_adjusted
            && (op_type == OP_LSHIFT || op_type == OP_RSHIFT || op_type == OP_DELETE)
        {
            nvim_curwin_reset_lbr();
            nvim_coladvance_set_curswant(old_col);
        }
    }
    rs_clearop(oap);
    nvim_set_motion_force(0);
}

// =============================================================================
// Phase 1: restore_lbr
// =============================================================================

unsafe fn dpo_restore_lbr() {
    restore_lbr(DPO_SAVED_LBR);
}

// =============================================================================
// Public export: do_pending_operator
// =============================================================================

/// Full migration of `do_pending_operator()`.
///
/// # Safety
/// - `cap` must be a valid `cmdarg_T *`
/// - Accesses global state via C accessors
#[unsafe(export_name = "do_pending_operator")]
pub unsafe extern "C" fn rs_do_pending_operator(cap: *mut c_void, old_col: c_int, gui_yank: bool) {
    let gui_yank_b = gui_yank;
    if dpo_should_process(cap) {
        dpo_preamble(cap, gui_yank_b);
        dpo_setup_positions(cap, gui_yank_b);
        dpo_dispatch_operator(cap, gui_yank_b);
        dpo_postamble(cap, old_col, gui_yank_b);
    }
    dpo_restore_lbr();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // The pending operator module requires full C runtime;
        // unit tests are limited to compilation checks.
        let size =
            std::mem::size_of::<unsafe extern "C" fn(*mut std::ffi::c_void) -> std::ffi::c_int>();
        assert!(size > 0);
    }
}
