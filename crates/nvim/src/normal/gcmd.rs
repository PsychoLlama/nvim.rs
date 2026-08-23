//! The `g` prefix tree.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::cursor::{
    check_cursor_lnum, coladvance, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr,
    get_cursor_pos_ptr,
};
use crate::edit::{BeginlineOpts, beginline, cursor_down, cursor_up, oneleft, oneright};
use crate::ex_cmds::do_ascii;
use crate::ex_docmd::{do_cmdline_cmd, do_exmode, do_sleep};
use crate::fold::has_any_folding;
use crate::main::{VIsual_active, VIsual_reselect, curbuf, curwin, mod_mask};
use crate::mbyte::{show_utf8, utf_find_illegal, utf_ptr2cells};
use crate::memline::goto_byte;
use crate::message::show_sb_text;
use crate::mouse::do_mouse;
use crate::normal::{
    MOD_MASK_CTRL, adjust_for_sel, check_text_locked, checkclearop, checkclearopq, clearopbeep,
    invoke_edit, kMTCharWise, kMTLineWise, nv_addsub, nv_edit, nv_gd, nv_gomark, nv_goto,
    nv_gotofile, nv_gv_cmd, nv_ident, nv_join, nv_operator, nv_pcmark, nv_put, nv_replace_mode,
    nv_screengo, nv_visual, nv_vreplace,
};
use crate::ops::cursor_pos_info;
use crate::plines::{getvvcol, linetabsize};
use crate::search::{BACKWARD, FORWARD, current_search};
use crate::state::virtual_active;
use crate::textobject::bckend_word;
use crate::types::{NUL, OK, OP_NOP, cmdarg_T, colnr_T, int64_t, linenr_T};
use crate::undo::undo_time;
use crate::window::{goto_tabpage, goto_tabpage_lastused};
use core::ffi::c_int;

use crate::keycodes::{
    Ctrl_H, K_BS, K_DOWN, K_END, K_HOME, K_IGNORE, K_KEND, K_KHOME, K_LEFTDRAG, K_LEFTMOUSE,
    K_LEFTRELEASE, K_MIDDLEDRAG, K_MIDDLEMOUSE, K_MIDDLERELEASE, K_MOUSEMOVE, K_RIGHTDRAG,
    K_RIGHTMOUSE, K_RIGHTRELEASE, K_UP, K_X1DRAG, K_X1MOUSE, K_X1RELEASE, K_X2DRAG, K_X2MOUSE,
    K_X2RELEASE,
};
use crate::r#move::{
    WinValid, adjust_skipcol, sms_marker_overlap, update_curswant_force, validate_cheight,
    validate_virtcol, win_col_off, win_col_off2,
};
use crate::pos::MAXCOL;

// The non-printing bytes the tree dispatches on, spelled as the bytes a
// pattern can name.
const CTRL_A: u8 = 1;
const CTRL_G: u8 = 7;
const CTRL_H: u8 = 8;
const HTAB: u8 = 9;
const CTRL_X: u8 = 24;
const CTRL_RSB: u8 = 29;
/// `£`, which is `g#` on a keyboard that has it.
const POUND_BYTE: u8 = 0xa3;

/// `g0`, `g^` and `gm`: the start, first non-blank and middle of the *screen*
/// line rather than of the buffer line.
///
/// Also called from `move.rs` for a mouse click landing left of the text.
pub(crate) unsafe fn nv_g_home_m_cmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        let to_first_non_blank = (*cap).nchar == '^' as c_int;
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false;
        let mut i;
        if (*win).w_onebuf_opt.wo_wrap != 0 && (*win).w_view_width != 0 {
            // A wrapped line's first screen row can be narrower than the rest,
            // so the row the cursor is on decides where its start is.
            let width1 = (*win).w_view_width - win_col_off(win);
            let width2 = width1 + win_col_off2(win);
            validate_virtcol(win);
            i = 0;
            if (*win).w_virtcol >= width1 && width2 > 0 {
                i = ((*win).w_virtcol - width1) / width2 * width2 + width1;
            }
            if (*win).w_skipcol > 0 && (*win).w_cursor.lnum == (*win).w_topline {
                // 'smoothscroll' hides part of the top row behind its marker;
                // the text starts after it.
                let overlap = sms_marker_overlap(win, (*win).w_view_width - width2);
                if overlap > 0 && i == (*win).w_skipcol {
                    i += overlap;
                }
            }
        } else {
            i = (*win).w_leftcol;
        }
        if (*cap).nchar == 'm' as c_int {
            i += ((*win).w_view_width - win_col_off(win)
                + if (*win).w_onebuf_opt.wo_wrap != 0 && i > 0 {
                    win_col_off2(win)
                } else {
                    0
                })
                / 2;
        }
        coladvance(win, i);
        if to_first_non_blank {
            while ascii_iswhite(gchar_cursor()) && oneright() == OK {}
            (*win).w_valid.clear(WinValid::WCOL);
        }
        (*win).w_set_curswant = true;
        // Inside a closed fold the wanted column is the one that was asked
        // for, not the one the fold's single displayed line has.
        if has_any_folding(win) != 0 {
            validate_cheight(win);
            if (*win).w_cline_folded {
                update_curswant_force();
            }
        }
        adjust_skipcol();
    }
}

/// `g_`: the last non-blank of the line, `count1 - 1` lines down.
pub(crate) unsafe fn nv_g_underscore_cmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = true;
        (*win).w_curswant = MAXCOL as colnr_T;
        if cursor_down((*cap).count1 - 1, (*(*cap).oap).op_type == OP_NOP) == 0 {
            clearopbeep((*cap).oap);
            return;
        }
        let line = get_cursor_line_ptr();
        // 'virtualedit' can leave the cursor on the terminator.
        if (*win).w_cursor.col > 0 && *line.offset((*win).w_cursor.col as isize) as c_int == NUL {
            (*win).w_cursor.col -= 1;
        }
        while (*win).w_cursor.col > 0
            && ascii_iswhite(*line.offset((*win).w_cursor.col as isize) as c_int)
        {
            (*win).w_cursor.col -= 1;
        }
        (*win).w_set_curswant = true;
        adjust_for_sel(cap);
    }
}

/// `g$` and `g<End>`: the end of the *screen* line.
pub(crate) unsafe fn nv_g_dollar_cmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        let oap = (*cap).oap;
        let col_off = win_col_off(win);
        // `<End>` also skips back over trailing white space.
        let to_last_non_blank = (*cap).nchar == K_END || (*cap).nchar == K_KEND;
        (*oap).motion_type = kMTCharWise;
        (*oap).inclusive = true;
        if (*win).w_onebuf_opt.wo_wrap != 0 && (*win).w_view_width != 0 {
            (*win).w_curswant = MAXCOL as colnr_T;
            if (*cap).count1 == 1 {
                let width1 = (*win).w_view_width - col_off;
                let width2 = width1 + win_col_off2(win);
                validate_virtcol(win);
                let mut i = width1 - 1;
                if (*win).w_virtcol >= width1 {
                    i += (((*win).w_virtcol - width1) / width2 + 1) * width2;
                }
                coladvance(win, i);
                update_curswant_force();
                // A character wider than one cell straddles the edge; step
                // back onto the last one that fits.
                if (*win).w_cursor.col > 0
                    && (*win).w_onebuf_opt.wo_wrap != 0
                    && (*win).w_virtcol > i
                {
                    (*win).w_cursor.col -= 1;
                }
            } else if !nv_screengo(oap, FORWARD as c_int, (*cap).count1 - 1, false) {
                clearopbeep(oap);
            }
        } else {
            // Without 'wrap' the screen line is what 'sidescroll' left showing.
            if (*cap).count1 > 1 {
                cursor_down((*cap).count1 - 1, false);
            }
            let i = (*win).w_leftcol + (*win).w_view_width - col_off - 1;
            coladvance(win, i);
            if (*win).w_cursor.col > 0 && utf_ptr2cells(get_cursor_pos_ptr()) > 1 {
                let mut vcol: colnr_T = 0;
                getvvcol(
                    win,
                    &raw mut (*win).w_cursor,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &raw mut vcol,
                );
                if vcol >= (*win).w_leftcol + (*win).w_view_width - col_off {
                    (*win).w_cursor.col -= 1;
                }
            }
            update_curswant_force();
        }
        if to_last_non_blank {
            while ascii_iswhite_or_nul(gchar_cursor()) && oneleft() == OK {}
            (*win).w_valid.clear(WinValid::WCOL);
        }
    }
}

/// `gi`: insert where insert mode was left, even if the line has since got
/// shorter.
pub(crate) unsafe fn nv_gi_cmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        if (*curbuf.get()).b_last_insert.mark.lnum != 0 {
            (*win).w_cursor = (*curbuf.get()).b_last_insert.mark;
            check_cursor_lnum(win);
            let len = get_cursor_line_len();
            if (*win).w_cursor.col > len {
                if virtual_active(win) {
                    // Past the end is a real position under 'virtualedit'.
                    (*win).w_cursor.coladd += (*win).w_cursor.col - len;
                }
                (*win).w_cursor.col = len;
            }
        }
        (*cap).cmdchar = 'i' as c_int;
        nv_edit(cap);
    }
}

/// `gh`, `gH` and `g CTRL-H`: start Select mode in the matching Visual kind.
/// `v`, `V` and CTRL-V sit exactly `'v' - 'h'` above `h`, `H` and CTRL-H.
unsafe fn nv_g_select(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*cap).cmdchar = (*cap).nchar + ('v' as c_int - 'h' as c_int);
        (*cap).arg = 1;
        nv_visual(cap);
    }
}

/// `gj` and `gk`: down and up by *screen* line -- which is the plain line move
/// when 'wrap' is off.
unsafe fn nv_g_screen_line(cap: *mut cmdarg_T, dir: c_int) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let oap = (*cap).oap;
        let moved = if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
            (*oap).motion_type = kMTLineWise;
            let stop_at_end = (*oap).op_type == OP_NOP;
            if dir == FORWARD as c_int {
                cursor_down((*cap).count1, stop_at_end) != 0
            } else {
                cursor_up((*cap).count1 as linenr_T, stop_at_end) != 0
            }
        } else {
            nv_screengo(oap, dir, (*cap).count1, false)
        };
        if !moved {
            clearopbeep(oap);
        }
    }
}

/// The special keys `g` accepts, which are the ones a `u8` cannot name.
///
/// Answers `false` for anything else, which sends the caller on to the byte
/// half of the tree.
unsafe fn nv_g_key(cap: *mut cmdarg_T, nchar: c_int) -> bool {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        match nchar {
            // `g<BS>` is `g CTRL-H`.
            K_BS => {
                (*cap).nchar = Ctrl_H;
                nv_g_select(cap);
            }
            K_DOWN => nv_g_screen_line(cap, FORWARD as c_int),
            K_UP => nv_g_screen_line(cap, BACKWARD as c_int),
            K_HOME | K_KHOME => nv_g_home_m_cmd(cap),
            K_END | K_KEND => nv_g_dollar_cmd(cap),
            // A mouse click after `g` acts as the CTRL-modified click.
            K_MIDDLEMOUSE | K_MIDDLEDRAG | K_MIDDLERELEASE | K_LEFTMOUSE | K_LEFTDRAG
            | K_LEFTRELEASE | K_MOUSEMOVE | K_RIGHTMOUSE | K_RIGHTDRAG | K_RIGHTRELEASE
            | K_X1MOUSE | K_X1DRAG | K_X1RELEASE | K_X2MOUSE | K_X2DRAG | K_X2RELEASE => {
                mod_mask.set(MOD_MASK_CTRL);
                do_mouse((*cap).oap, nchar, BACKWARD as c_int, (*cap).count1, false);
            }
            K_IGNORE => {}
            _ => return false,
        }
        true
    }
}

/// `g`, whose second character says what the command is.
pub(crate) unsafe fn nv_g_cmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let oap = (*cap).oap;
        let nchar = (*cap).nchar;
        if nv_g_key(cap, nchar) {
            return;
        }
        // `u8::try_from` rather than `as u8`: a multibyte character after `g`
        // must reach the default arm rather than alias one of these bytes.
        match u8::try_from(nchar) {
            // `g CTRL-A` and `g CTRL-X`: increment each line of the selection
            // by a growing multiple of the count. Only meaningful in Visual
            // mode.
            Ok(CTRL_A | CTRL_X) => {
                if VIsual_active.get() {
                    (*cap).arg = 1;
                    (*cap).cmdchar = nchar;
                    (*cap).nchar = NUL;
                    nv_addsub(cap);
                } else {
                    clearopbeep(oap);
                }
            }
            // `gR`: virtual replace mode.
            Ok(b'R') => {
                (*cap).arg = 1;
                nv_replace_mode(cap);
            }
            // `gr`: replace one character virtually.
            Ok(b'r') => nv_vreplace(cap),
            // `g&`: repeat the last `:substitute` over the whole file, keeping
            // the flags.
            Ok(b'&') => {
                do_cmdline_cmd(c"%s//~/&".as_ptr());
            }
            // `gv`: reselect the previous selection.
            Ok(b'v') => nv_gv_cmd(cap),
            // `gV`: do not reselect it after the next Select-mode edit.
            Ok(b'V') => VIsual_reselect.set(0),
            Ok(b'h' | b'H' | CTRL_H) => nv_g_select(cap),
            // `gn`/`gN`: select the next/previous match of the last search.
            Ok(b'N' | b'n') => {
                if current_search((*cap).count1, nchar == 'n' as c_int) == 0 {
                    clearopbeep(oap);
                }
            }
            Ok(b'j') => nv_g_screen_line(cap, FORWARD as c_int),
            Ok(b'k') => nv_g_screen_line(cap, BACKWARD as c_int),
            // `gJ`: join without inserting or removing spaces.
            Ok(b'J') => nv_join(cap),
            Ok(b'^' | b'0' | b'm') => nv_g_home_m_cmd(cap),
            // `gM`: the middle of the line by *text* width, or the count'th
            // percentage of it.
            Ok(b'M') => {
                (*oap).motion_type = kMTCharWise;
                (*oap).inclusive = false;
                let width = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                if (*cap).count0 > 0 && (*cap).count0 <= 100 {
                    coladvance(curwin.get(), width * (*cap).count0 / 100);
                } else {
                    coladvance(curwin.get(), width / 2);
                }
                (*curwin.get()).w_set_curswant = true;
            }
            Ok(b'_') => nv_g_underscore_cmd(cap),
            Ok(b'$') => nv_g_dollar_cmd(cap),
            // `g*`, `g#`, `g]` and `g CTRL-]`: the identifier searches that do
            // not anchor at a word boundary.
            Ok(b'*' | b'#' | POUND_BYTE | CTRL_RSB | b']') => nv_ident(cap),
            // `ge`/`gE`: back to the end of the previous word.
            Ok(b'e' | b'E') => {
                (*oap).motion_type = kMTCharWise;
                (*curwin.get()).w_set_curswant = true;
                (*oap).inclusive = true;
                if bckend_word((*cap).count1, nchar == 'E' as c_int, false) == 0 {
                    clearopbeep(oap);
                }
            }
            // `g CTRL-G`: count the words, lines and bytes.
            Ok(CTRL_G) => cursor_pos_info(ptr::null_mut()),
            Ok(b'i') => nv_gi_cmd(cap),
            // `gI`: insert in column 1 regardless of indent.
            Ok(b'I') => {
                beginline(BeginlineOpts::NONE);
                if !checkclearopq(oap) {
                    invoke_edit(cap, 0, 'g' as c_int, 0);
                }
            }
            // `gf`/`gF`: edit the file named under the cursor.
            Ok(b'f' | b'F') => nv_gotofile(cap),
            // `g'` and `` g` ``: jump to a mark without touching the jump
            // list. The argument is what tells `nv_gomark` it is linewise.
            Ok(b'\'') => {
                (*cap).arg = 1;
                nv_gomark(cap);
            }
            Ok(b'`') => nv_gomark(cap),
            Ok(b's') => do_sleep(((*cap).count1 * 1000) as int64_t, false),
            // `ga`: describe the character under the cursor.
            Ok(b'a') => do_ascii(ptr::null_mut()),
            // `g8` shows the byte sequence; `8g8` finds an illegal one.
            Ok(b'8') => {
                if (*cap).count0 == 8 {
                    utf_find_illegal();
                } else {
                    show_utf8();
                }
            }
            // `g<`: show the previous message screen again.
            Ok(b'<') => show_sb_text(),
            // `gg`: to the first line, or the count'th.
            Ok(b'g') => {
                (*cap).arg = 0;
                nv_goto(cap);
            }
            // `gq` and `gw` both format; `gw` returns the cursor to where it
            // was, which is what the remembered position is for.
            Ok(b'q' | b'w') => {
                (*oap).cursor_start = (*curwin.get()).w_cursor;
                nv_operator(cap);
            }
            // The rest of the two-character operators: `g~ gu gU g? g@`.
            Ok(b'~' | b'u' | b'U' | b'?' | b'@') => nv_operator(cap),
            // `gd`/`gD`: jump to the local or global declaration.
            Ok(b'd' | b'D') => nv_gd(oap, nchar, (*cap).count0),
            // `gp`/`gP`: put and leave the cursor after the new text.
            Ok(b'p' | b'P') => nv_put(cap),
            // `go`: to a byte offset in the buffer.
            Ok(b'o') => {
                (*oap).inclusive = false;
                goto_byte((*cap).count0);
            }
            // `gQ`: Ex mode.
            Ok(b'Q') => {
                if !check_text_locked(oap) && !checkclearopq(oap) {
                    do_exmode();
                }
            }
            // `g,` and `g;`: forwards and backwards through the change list.
            Ok(b',') => nv_pcmark(cap),
            Ok(b';') => {
                (*cap).count1 = -(*cap).count1;
                nv_pcmark(cap);
            }
            // `gt`/`gT`: the next or previous tab page.
            Ok(b't') => {
                if !checkclearop(oap) {
                    goto_tabpage((*cap).count0);
                }
            }
            Ok(b'T') => {
                if !checkclearop(oap) {
                    goto_tabpage(-(*cap).count1);
                }
            }
            // `g<Tab>`: the tab page used before this one.
            Ok(HTAB) => {
                if !checkclearop(oap) && !goto_tabpage_lastused() {
                    clearopbeep(oap);
                }
            }
            // `g+`/`g-`: forwards and backwards through the undo tree by time.
            Ok(b'+' | b'-') => {
                if !checkclearopq(oap) {
                    let count = if nchar == '-' as c_int {
                        -(*cap).count1
                    } else {
                        (*cap).count1
                    };
                    undo_time(count, false, false, false);
                }
            }
            _ => clearopbeep(oap),
        }
    }
}
