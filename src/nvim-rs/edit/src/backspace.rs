//! Backspace handler migrated from edit.c
//!
//! Implements `ins_bs` -- Backspace, delete-word, and delete-line handling
//! in Insert mode, including softtabstop alignment, replace mode, and
//! reverse insert mode.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_int, c_uint};

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

// ============================================================================
// C accessor functions
// ============================================================================

extern "C" {
    static mut State: c_int;
    static mut curbuf: *mut std::ffi::c_void;
    // Buffer state
    #[link_name = "rs_buf_is_empty"]
    fn nvim_curbuf_is_empty_via_buf(buf: *mut std::ffi::c_void) -> bool;
    #[link_name = "rs_bt_prompt"]
    fn nvim_curbuf_is_prompt_via_curbuf(buf: *mut std::ffi::c_void) -> bool;

    // Cursor position
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    fn nvim_curwin_get_cursor_col() -> ColnrT;
    fn nvim_curwin_set_cursor_col(col: ColnrT);
    fn nvim_curwin_set_cursor_lnum(lnum: LinenrT);
    fn nvim_curwin_get_cursor_coladd() -> ColnrT;
    fn nvim_set_curwin_cursor_coladd(val: ColnrT);
    fn inc_cursor() -> c_int;
    fn dec_cursor() -> c_int;

    // Insert start tracking
    fn nvim_get_Insstart_lnum() -> LinenrT;
    fn nvim_set_Insstart(lnum: LinenrT, col: ColnrT);
    fn nvim_get_Insstart_orig_lnum() -> LinenrT;
    fn nvim_get_Insstart_orig_col() -> ColnrT;
    fn nvim_set_Insstart_orig(lnum: LinenrT, col: ColnrT);

    // Mode flags

    // Global variables
    fn nvim_get_revins_on() -> c_int;
    fn nvim_get_revins_chars() -> c_int;
    fn nvim_set_revins_chars(val: c_int);
    fn nvim_get_revins_legal() -> c_int;
    fn nvim_set_revins_legal(val: c_int);
    fn nvim_get_arrow_used() -> c_int;
    fn nvim_get_ai_col() -> ColnrT;
    fn nvim_set_did_ai(val: bool);
    fn nvim_set_did_si(val: bool);
    fn nvim_set_can_si(val: bool);
    fn nvim_set_can_si_back(val: bool);
    fn nvim_set_end_comment_pending(val: c_int);
    fn nvim_get_orig_line_count() -> LinenrT;
    fn nvim_get_dollar_vcol() -> ColnrT;
    fn nvim_set_dollar_vcol(val: ColnrT);

    // can_bs check
    fn can_bs(what: c_int) -> bool;

    // Undo / stop_arrow
    fn stop_arrow() -> c_int;
    fn nvim_u_save(lnum1: c_int, lnum2: c_int) -> c_int;

    // Indent
    fn inindent(extra: c_int) -> c_int;
    fn nvim_set_can_cindent(val: c_int);
    fn nvim_curbuf_get_b_p_ai() -> c_int;
    fn cindent_on() -> bool;
    fn fix_indent();
    fn beginline(flags: c_int);

    // ml_get_len
    fn ml_get_len(lnum: c_int) -> c_int;

    // Character ops
    fn nvim_gchar_cursor() -> c_int;
    fn nvim_has_format_option(c: c_int) -> bool;
    fn nvim_trim_eol_space();
    fn nvim_do_join_simple();

    // Replace mode
    fn replace_pop_if_nul() -> c_int;
    fn mb_replace_pop_ins();
    fn replace_pop_ins();
    fn replace_do_bs(limit_col: c_int);

    // Delete character
    fn del_char(fixpos: c_int) -> c_int;

    // Redo buffer
    fn AppendCharToRedobuff(c: c_int);

    // p_cpo backspace check
    fn nvim_p_cpo_has_backspace() -> bool;

    // Virtual column
    fn nvim_curwin_get_w_virtcol() -> ColnrT;

    // Fold
    fn rs_foldOpenCursor();

    // Word class
    fn nvim_mb_get_class_cursor() -> c_int;
    fn vim_iswordc(c: c_int) -> bool;
    fn nvim_cursor_has_composing() -> c_int;

    // Softtabstop helpers
    fn nvim_edit_ins_bs_check_sts(inserted_space_p: *mut c_int, in_indent: bool) -> bool;
    fn nvim_edit_ins_bs_softtabstop(inserted_space_p: *mut c_int, in_indent: bool) -> bool;

    // vim_beep
    fn vim_beep(val: c_uint);
}

// ============================================================================
// Constants (verified via `_Static_assert` in `edit.c`)
// ============================================================================

/// `FAIL` from `vim_defs.h`
const FAIL: c_int = 0;

/// `NUL` character
const NUL: c_int = 0;

/// `REPLACE_FLAG` from `vim_defs.h`
const REPLACE_FLAG: c_int = 0x100;

/// `VREPLACE_FLAG` from `vim_defs.h`
const VREPLACE_FLAG: c_int = 0x200;

/// `MODE_NORMAL` from `vim_defs.h`
const MODE_NORMAL: c_int = 0x01;

/// `kOptBoFlagBackspace` (beep flag for backspace)
const BO_FLAG_BACKSPACE: c_int = 0x02;

/// `BACKSPACE_CHAR`
const BACKSPACE_CHAR: c_int = 1;

/// `BACKSPACE_WORD`
const BACKSPACE_WORD: c_int = 2;

/// `BACKSPACE_WORD_NOT_SPACE`
const BACKSPACE_WORD_NOT_SPACE: c_int = 3;

/// `BACKSPACE_LINE`
const BACKSPACE_LINE: c_int = 4;

/// `BS_START` (backspace start option)
const BS_START: c_int = b's' as c_int;

/// `BS_INDENT`
const BS_INDENT: c_int = b'i' as c_int;

/// `BS_EOL`
const BS_EOL: c_int = b'l' as c_int;

/// `BS_NOSTOP`
const BS_NOSTOP: c_int = b'p' as c_int;

/// `BL_WHITE` (`beginline` flag)
const BL_WHITE: c_int = 1;

/// `FO_AUTO`
const FO_AUTO: c_int = b'a' as c_int;

/// `FO_WHITE_PAR`
const FO_WHITE_PAR: c_int = b'w' as c_int;

// ============================================================================
// ins_bs implementation
// ============================================================================

/// Handle Backspace, delete-word and delete-line in Insert mode.
///
/// Returns true when backspace was actually used.
///
/// # Safety
/// Accesses global Neovim state via C accessors.
#[allow(clippy::cognitive_complexity)]
unsafe fn ins_bs_impl(c: c_int, mode: c_int, inserted_space_p: *mut c_int) -> bool {
    let revins_on = nvim_get_revins_on() != 0;
    let cursor_lnum = nvim_curwin_get_cursor_lnum();
    let cursor_col = nvim_curwin_get_cursor_col();
    let arrow_used = nvim_get_arrow_used() != 0;
    let ai_col = nvim_get_ai_col();

    // Can't delete anything in an empty file; can't backup past first char;
    // can't backup past starting point unless 'backspace' > 1.
    if nvim_curbuf_is_empty_via_buf(curbuf)
        || (!revins_on
            && ((cursor_lnum == 1 && cursor_col == 0)
                || (!can_bs(BS_START)
                    && ((arrow_used && !nvim_curbuf_is_prompt_via_curbuf(curbuf))
                        || (cursor_lnum == nvim_get_Insstart_orig_lnum()
                            && cursor_col <= nvim_get_Insstart_orig_col())))
                || (!can_bs(BS_INDENT) && !arrow_used && ai_col > 0 && cursor_col <= ai_col)
                || (!can_bs(BS_EOL) && cursor_col == 0)))
    {
        vim_beep(BO_FLAG_BACKSPACE as c_uint);
        return false;
    }

    if stop_arrow() == FAIL {
        return false;
    }

    let in_indent = inindent(0) != 0;
    if in_indent {
        nvim_set_can_cindent(0);
    }

    nvim_set_end_comment_pending(NUL); // After BS, don't auto-end comment
    if revins_on {
        inc_cursor(); // put cursor after last inserted char
    }

    // Virtualedit: handle coladd
    let coladd = nvim_curwin_get_cursor_coladd();
    if coladd > 0 {
        if mode == BACKSPACE_CHAR {
            nvim_set_curwin_cursor_coladd(coladd - 1);
            return true;
        }
        if mode == BACKSPACE_WORD {
            nvim_set_curwin_cursor_coladd(0);
            return true;
        }
        nvim_set_curwin_cursor_coladd(0);
    }

    let state = State;
    let mut did_backspace = false;
    let mut call_fix_indent = false;
    let cursor_col = nvim_curwin_get_cursor_col(); // re-read after coladd

    if cursor_col == 0 {
        // Delete newline!
        let lnum = nvim_get_Insstart_lnum();
        if nvim_curwin_get_cursor_lnum() == lnum || revins_on {
            let cur_lnum = nvim_curwin_get_cursor_lnum();
            if nvim_u_save(cur_lnum - 2, cur_lnum + 1) == FAIL {
                return false;
            }
            nvim_set_Insstart(lnum - 1, ml_get_len(lnum - 1));
        }

        // In replace mode: cc < 0 means NL was inserted; cc >= 0 means replaced
        let mut cc: c_int = if state & REPLACE_FLAG != 0 {
            replace_pop_if_nul()
        } else {
            -1
        };

        if state & REPLACE_FLAG != 0 && nvim_curwin_get_cursor_lnum() <= lnum {
            // In replace mode, in the line we started replacing, only move cursor
            dec_cursor();
        } else {
            let orig_line_count = nvim_get_orig_line_count();
            if state & VREPLACE_FLAG == 0 || nvim_curwin_get_cursor_lnum() > orig_line_count {
                let temp = nvim_gchar_cursor(); // remember current char
                let new_lnum = nvim_curwin_get_cursor_lnum() - 1;
                nvim_curwin_set_cursor_lnum(new_lnum);

                // When "aw" is in 'formatoptions': delete trailing space
                if nvim_has_format_option(FO_AUTO) && nvim_has_format_option(FO_WHITE_PAR) {
                    nvim_trim_eol_space();
                }

                nvim_do_join_simple();
                if temp == NUL && nvim_gchar_cursor() != NUL {
                    inc_cursor();
                }
            } else {
                dec_cursor();
            }

            // In REPLACE mode: restore text replaced by the NL
            if state & REPLACE_FLAG != 0 {
                let old_state = State;
                State = MODE_NORMAL;
                while cc > 0 {
                    let save_col = nvim_curwin_get_cursor_col();
                    mb_replace_pop_ins();
                    nvim_curwin_set_cursor_col(save_col);
                    cc = replace_pop_if_nul();
                }
                replace_pop_ins();
                State = old_state;
            }
        }
        nvim_set_did_ai(false);
    } else {
        // Delete character(s) before the cursor.
        if revins_on {
            dec_cursor(); // put cursor on last inserted char
        }
        let mut mincol: ColnrT = 0;

        // Keep indent for BACKSPACE_LINE
        if mode == BACKSPACE_LINE && (nvim_curbuf_get_b_p_ai() != 0 || cindent_on()) && !revins_on {
            let save_col = nvim_curwin_get_cursor_col();
            beginline(BL_WHITE);
            if nvim_curwin_get_cursor_col() < save_col {
                mincol = nvim_curwin_get_cursor_col();
                call_fix_indent = true;
            }
            nvim_curwin_set_cursor_col(save_col);
        }

        // Handle softtabstop or smarttab backspace
        if mode == BACKSPACE_CHAR && nvim_edit_ins_bs_check_sts(inserted_space_p, in_indent) {
            *inserted_space_p = 0;
            nvim_edit_ins_bs_softtabstop(inserted_space_p, in_indent);
        } else {
            // Delete up to starting point, start of line or previous word.
            let mut cur_mode = mode;
            let mut temp = false;
            let mut cclass = nvim_mb_get_class_cursor();
            loop {
                if !revins_on {
                    dec_cursor();
                }
                let cc = nvim_gchar_cursor();
                let prev_cclass = cclass;
                cclass = nvim_mb_get_class_cursor();

                if cur_mode == BACKSPACE_WORD && cc != c_int::from(b' ') && cc != c_int::from(b'\t')
                {
                    cur_mode = BACKSPACE_WORD_NOT_SPACE;
                    temp = vim_iswordc(cc);
                } else if cur_mode == BACKSPACE_WORD_NOT_SPACE
                    && (cc == c_int::from(b' ')
                        || cc == c_int::from(b'\t')
                        || vim_iswordc(cc) != temp
                        || prev_cclass != cclass)
                {
                    // End of word
                    if !revins_on {
                        inc_cursor();
                    } else if state & REPLACE_FLAG != 0 {
                        dec_cursor();
                    }
                    break;
                }

                if state & REPLACE_FLAG != 0 {
                    replace_do_bs(-1);
                } else {
                    let has_composing = nvim_cursor_has_composing();
                    del_char(0);
                    if has_composing != 0 {
                        inc_cursor();
                    }
                    let revins_chars = nvim_get_revins_chars();
                    if revins_chars > 0 {
                        nvim_set_revins_chars(revins_chars - 1);
                        nvim_set_revins_legal(nvim_get_revins_legal() + 1);
                    }
                    if revins_on && nvim_gchar_cursor() == NUL {
                        break;
                    }
                }

                // Just a single backspace?
                if cur_mode == BACKSPACE_CHAR {
                    break;
                }

                let col_now = nvim_curwin_get_cursor_col();
                let can_continue = revins_on
                    || (col_now > mincol
                        && (can_bs(BS_NOSTOP)
                            || (nvim_curwin_get_cursor_lnum() != nvim_get_Insstart_orig_lnum()
                                || col_now != nvim_get_Insstart_orig_col())));
                if !can_continue {
                    break;
                }
            }
        }
        did_backspace = true;
    }

    nvim_set_did_si(false);
    nvim_set_can_si(false);
    nvim_set_can_si_back(false);
    if nvim_curwin_get_cursor_col() <= 1 {
        nvim_set_did_ai(false);
    }

    if call_fix_indent {
        fix_indent();
    }

    // It's a little strange to put backspaces into the redo buffer, but
    // it makes auto-indent a lot easier to deal with.
    AppendCharToRedobuff(c);

    // If deleted before the insertion point, adjust it
    let cursor_lnum = nvim_curwin_get_cursor_lnum();
    let cursor_col = nvim_curwin_get_cursor_col();
    if cursor_lnum == nvim_get_Insstart_orig_lnum() && cursor_col < nvim_get_Insstart_orig_col() {
        nvim_set_Insstart_orig(cursor_lnum, cursor_col);
    }

    // vi behaviour: dollar display even when not set
    if nvim_p_cpo_has_backspace() && nvim_get_dollar_vcol() == -1 {
        nvim_set_dollar_vcol(nvim_curwin_get_w_virtcol());
    }

    // When deleting a char, cursor line must never be in a closed fold.
    if did_backspace {
        rs_foldOpenCursor();
    }
    did_backspace
}

/// Handle Backspace, delete-word and delete-line in Insert mode.
///
/// Returns true when backspace was actually used.
///
/// # Safety
/// Accesses global Neovim state.
#[must_use]
#[unsafe(export_name = "ins_bs")]
pub unsafe extern "C" fn rs_ins_bs(c: c_int, mode: c_int, inserted_space_p: *mut c_int) -> bool {
    ins_bs_impl(c, mode, inserted_space_p)
}
