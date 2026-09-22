//! The `g` prefix tree.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::ModMask;
use crate::keycodes::{Ctrl_H, Key};
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::cursor::{
    check_cursor_lnum, coladvance, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr,
    get_cursor_pos_ptr,
};
use crate::edit::{BeginlineOpts, beginline, cursor_down, cursor_up, oneleft, oneright};
use crate::ex_cmds::describe_cursor_char;
use crate::ex_docmd::{do_cmdline_cmd, do_exmode, do_sleep};
use crate::fold::has_any_folding;
use crate::getchar::state::mod_mask;
use crate::mbyte::{show_utf8, utf_find_illegal, utf_ptr2cells};
use crate::memline::goto_byte;
use crate::message::show_sb_text;
use crate::mouse::do_mouse;
use crate::normal::{
    adjust_for_sel, check_clear_op, check_clear_op_quit, check_text_locked, clear_op_beep,
    invoke_edit, kMTCharWise, kMTLineWise, nv_addsub, nv_edit, nv_gd, nv_gomark, nv_goto,
    nv_gotofile, nv_gv_cmd, nv_ident, nv_join, nv_operator, nv_pcmark, nv_put, nv_replace_mode,
    nv_screengo, nv_visual, nv_vreplace, visual_active,
};
use crate::ops::cursor_pos_info;
use crate::plines::linetabsize;
use crate::search::{BACKWARD, FORWARD, current_search};
use crate::state::mode::VIsual_reselect;
use crate::state::virtual_active;
use crate::textobject::bckend_word;
use crate::types::{CmdArg, ColNr, LineNr, NUL, OpType, int64_t};
use crate::undo::undo_time;
use crate::window::{goto_tabpage, goto_tabpage_lastused};
use core::ffi::c_int;

use crate::r#move::{
    WinValid, adjust_skipcol, sms_marker_overlap, update_curswant_force, validate_cheight,
    validate_virtcol, win_col_off2,
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
pub(crate) fn nv_g_home_m_cmd(cmd_arg: &mut CmdArg) {
    let mut win = Win::current();
    let to_first_non_blank = cmd_arg.nchar == '^' as c_int;
    cmd_arg.op().motion_type = kMTCharWise;
    cmd_arg.op().inclusive = false;
    let mut i;
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        // A wrapped line's first screen row can be narrower than the rest,
        // so the row the cursor is on decides where its start is.
        let width1 = win.w_view_width - win.col_off();
        let width2 = width1 + win_col_off2(win);
        validate_virtcol(win);
        i = 0;
        if win.w_virtcol >= width1 && width2 > 0 {
            i = (win.w_virtcol - width1) / width2 * width2 + width1;
        }
        if win.w_skipcol > 0 && win.w_cursor.lnum == win.w_topline {
            // 'smoothscroll' hides part of the top row behind its marker;
            // the text starts after it.
            let overlap = sms_marker_overlap(win, win.w_view_width - width2);
            if overlap > 0 && i == win.w_skipcol {
                i += overlap;
            }
        }
    } else {
        i = win.w_leftcol;
    }
    if cmd_arg.nchar == 'm' as c_int {
        i += (win.w_view_width - win.col_off()
            + if win.w_onebuf_opt.wo_wrap != 0 && i > 0 {
                win_col_off2(win)
            } else {
                0
            })
            / 2;
    }
    coladvance(win, i);
    if to_first_non_blank {
        while ascii_iswhite(gchar_cursor()) && oneright().is_ok() {}
        win.w_valid.clear(WinValid::WCOL);
    }
    win.w_set_curswant = true;
    // Inside a closed fold the wanted column is the one that was asked
    // for, not the one the fold's single displayed line has.
    if has_any_folding(win) != 0 {
        validate_cheight(win);
        if win.w_cline_folded {
            update_curswant_force();
        }
    }
    adjust_skipcol();
}

/// `g_`: the last non-blank of the line, `count1 - 1` lines down.
pub(crate) fn nv_g_underscore_cmd(cmd_arg: &mut CmdArg) {
    let mut win = Win::current();
    cmd_arg.op().motion_type = kMTCharWise;
    cmd_arg.op().inclusive = true;
    win.w_curswant = MAXCOL as ColNr;
    if cursor_down(cmd_arg.count1 - 1, cmd_arg.op().op_type == OpType::Nop).is_err() {
        clear_op_beep(cmd_arg.op());
        return;
    }
    let line = get_cursor_line_ptr();
    // SAFETY: `line` is the NUL-terminated cursor line and `col` indexes it.
    let byte_at = |col: ColNr| unsafe { *line.offset(col as isize) };
    // 'virtualedit' can leave the cursor on the terminator.
    if win.w_cursor.col > 0 && c_int::from(byte_at(win.w_cursor.col)) == NUL {
        win.w_cursor.col -= 1;
    }
    while win.w_cursor.col > 0 && ascii_iswhite(c_int::from(byte_at(win.w_cursor.col))) {
        win.w_cursor.col -= 1;
    }
    win.w_set_curswant = true;
    adjust_for_sel(cmd_arg);
}

/// `g$` and `g<End>`: the end of the *screen* line.
pub(crate) fn nv_g_dollar_cmd(cmd_arg: &mut CmdArg) {
    let mut win = Win::current();
    let mut op = cmd_arg.op();
    let col_off = win.col_off();
    // `<End>` also skips back over trailing white space.
    let to_last_non_blank = cmd_arg.nchar == Key::End.code() || cmd_arg.nchar == Key::Kend.code();
    op.motion_type = kMTCharWise;
    op.inclusive = true;
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        win.w_curswant = MAXCOL as ColNr;
        if cmd_arg.count1 == 1 {
            let width1 = win.w_view_width - col_off;
            let width2 = width1 + win_col_off2(win);
            validate_virtcol(win);
            let mut i = width1 - 1;
            if win.w_virtcol >= width1 {
                i += ((win.w_virtcol - width1) / width2 + 1) * width2;
            }
            coladvance(win, i);
            update_curswant_force();
            // A character wider than one cell straddles the edge; step
            // back onto the last one that fits.
            if win.w_cursor.col > 0 && win.w_onebuf_opt.wo_wrap != 0 && win.w_virtcol > i {
                win.w_cursor.col -= 1;
            }
        } else if !unsafe { nv_screengo(op.raw(), FORWARD as c_int, cmd_arg.count1 - 1, false) } {
            clear_op_beep(op);
        }
    } else {
        // Without 'wrap' the screen line is what 'sidescroll' left showing.
        if cmd_arg.count1 > 1 {
            let _ = cursor_down(cmd_arg.count1 - 1, false);
        }
        let i = win.w_leftcol + win.w_view_width - col_off - 1;
        coladvance(win, i);
        if win.w_cursor.col > 0 && unsafe { utf_ptr2cells(get_cursor_pos_ptr()) } > 1 {
            let vcol = win.virtual_vcol_span(win.cursor()).1;
            if vcol >= win.w_leftcol + win.w_view_width - col_off {
                win.w_cursor.col -= 1;
            }
        }
        update_curswant_force();
    }
    if to_last_non_blank {
        while ascii_iswhite_or_nul(gchar_cursor()) && oneleft().is_ok() {}
        win.w_valid.clear(WinValid::WCOL);
    }
}

/// `gi`: insert where insert mode was left, even if the line has since got
/// shorter.
pub(crate) fn nv_gi_cmd(cmd_arg: &mut CmdArg) {
    let mut win = Win::current();
    if Buf::current().b_last_insert.mark.lnum != 0 {
        win.w_cursor = Buf::current().b_last_insert.mark;
        check_cursor_lnum(win);
        let len = get_cursor_line_len();
        if win.w_cursor.col > len {
            if virtual_active(win) {
                // Past the end is a real position under 'virtualedit'.
                win.w_cursor.coladd += win.w_cursor.col - len;
            }
            win.w_cursor.col = len;
        }
    }
    cmd_arg.cmdchar = 'i' as c_int;
    nv_edit(cmd_arg);
}

/// `gh`, `gH` and `g CTRL-H`: start Select mode in the matching Visual kind.
/// `v`, `V` and CTRL-V sit exactly `'v' - 'h'` above `h`, `H` and CTRL-H.
fn nv_g_select(cmd_arg: &mut CmdArg) {
    cmd_arg.cmdchar = cmd_arg.nchar + ('v' as c_int - 'h' as c_int);
    cmd_arg.arg = 1;
    nv_visual(cmd_arg);
}

/// `gj` and `gk`: down and up by *screen* line -- which is the plain line move
/// when 'wrap' is off.
fn nv_g_screen_line(cmd_arg: &mut CmdArg, dir: c_int) {
    let mut op = cmd_arg.op();
    let moved = if Win::current().w_onebuf_opt.wo_wrap == 0 {
        op.motion_type = kMTLineWise;
        let stop_at_end = op.op_type == OpType::Nop;
        if dir == FORWARD as c_int {
            cursor_down(cmd_arg.count1, stop_at_end).is_ok()
        } else {
            cursor_up(cmd_arg.count1 as LineNr, stop_at_end).is_ok()
        }
    } else {
        unsafe { nv_screengo(op.raw(), dir, cmd_arg.count1, false) }
    };
    if !moved {
        clear_op_beep(op);
    }
}

/// The special keys `g` accepts, which are the ones a `u8` cannot name.
///
/// Answers `false` for anything else, which sends the caller on to the byte
/// half of the tree.
fn nv_g_key(cmd_arg: &mut CmdArg, nchar: c_int) -> bool {
    match Key::try_from(nchar) {
        // `g<BS>` is `g CTRL-H`.
        Ok(Key::Bs) => {
            cmd_arg.nchar = Ctrl_H;
            nv_g_select(cmd_arg);
        }
        Ok(Key::Down) => nv_g_screen_line(cmd_arg, FORWARD as c_int),
        Ok(Key::Up) => nv_g_screen_line(cmd_arg, BACKWARD as c_int),
        Ok(Key::Home | Key::Khome) => nv_g_home_m_cmd(cmd_arg),
        Ok(Key::End | Key::Kend) => nv_g_dollar_cmd(cmd_arg),
        // A mouse click after `g` acts as the CTRL-modified click.
        Ok(
            Key::Middlemouse
            | Key::Middledrag
            | Key::Middlerelease
            | Key::Leftmouse
            | Key::Leftdrag
            | Key::Leftrelease
            | Key::Mousemove
            | Key::Rightmouse
            | Key::Rightdrag
            | Key::Rightrelease
            | Key::X1mouse
            | Key::X1drag
            | Key::X1release
            | Key::X2mouse
            | Key::X2drag
            | Key::X2release,
        ) => {
            mod_mask.set(ModMask::CTRL);
            do_mouse(
                Some(cmd_arg.op()),
                nchar,
                BACKWARD as c_int,
                cmd_arg.count1,
                false,
            );
        }
        Ok(Key::Ignore) => {}
        _ => return false,
    }
    true
}

/// `g`, whose second character says what the command is.
pub(crate) fn nv_g_cmd(cmd_arg: &mut CmdArg) {
    let mut op = cmd_arg.op();
    let nchar = cmd_arg.nchar;
    if nv_g_key(cmd_arg, nchar) {
        return;
    }
    // `u8::try_from` rather than `as u8`: a multibyte character after `g`
    // must reach the default arm rather than alias one of these bytes.
    match u8::try_from(nchar) {
        // `g CTRL-A` and `g CTRL-X`: increment each line of the selection
        // by a growing multiple of the count. Only meaningful in Visual
        // mode.
        Ok(CTRL_A | CTRL_X) => {
            if visual_active() {
                cmd_arg.arg = 1;
                cmd_arg.cmdchar = nchar;
                cmd_arg.nchar = NUL;
                nv_addsub(cmd_arg);
            } else {
                clear_op_beep(op);
            }
        }
        // `gR`: virtual replace mode.
        Ok(b'R') => {
            cmd_arg.arg = 1;
            nv_replace_mode(cmd_arg);
        }
        // `gr`: replace one character virtually.
        Ok(b'r') => nv_vreplace(cmd_arg),
        // `g&`: repeat the last `:substitute` over the whole file, keeping
        // the flags.
        Ok(b'&') => {
            let _ = do_cmdline_cmd(c"%s//~/&");
        }
        // `gv`: reselect the previous selection.
        Ok(b'v') => nv_gv_cmd(cmd_arg),
        // `gV`: do not reselect it after the next Select-mode edit.
        Ok(b'V') => VIsual_reselect.set(0),
        Ok(b'h' | b'H' | CTRL_H) => nv_g_select(cmd_arg),
        // `gn`/`gN`: select the next/previous match of the last search.
        Ok(b'N' | b'n') => {
            if current_search(cmd_arg.count1, nchar == 'n' as c_int).is_err() {
                clear_op_beep(op);
            }
        }
        Ok(b'j') => nv_g_screen_line(cmd_arg, FORWARD as c_int),
        Ok(b'k') => nv_g_screen_line(cmd_arg, BACKWARD as c_int),
        // `gJ`: join without inserting or removing spaces.
        Ok(b'J') => nv_join(cmd_arg),
        Ok(b'^' | b'0' | b'm') => nv_g_home_m_cmd(cmd_arg),
        // `gM`: the middle of the line by *text* width, or the count'th
        // percentage of it.
        Ok(b'M') => {
            op.motion_type = kMTCharWise;
            op.inclusive = false;
            let width = linetabsize(Win::current(), Win::current().w_cursor.lnum);
            if cmd_arg.count0 > 0 && cmd_arg.count0 <= 100 {
                coladvance(Win::current(), width * cmd_arg.count0 / 100);
            } else {
                coladvance(Win::current(), width / 2);
            }
            Win::current().w_set_curswant = true;
        }
        Ok(b'_') => nv_g_underscore_cmd(cmd_arg),
        Ok(b'$') => nv_g_dollar_cmd(cmd_arg),
        // `g*`, `g#`, `g]` and `g CTRL-]`: the identifier searches that do
        // not anchor at a word boundary.
        Ok(b'*' | b'#' | POUND_BYTE | CTRL_RSB | b']') => nv_ident(cmd_arg),
        // `ge`/`gE`: back to the end of the previous word.
        Ok(b'e' | b'E') => {
            op.motion_type = kMTCharWise;
            Win::current().w_set_curswant = true;
            op.inclusive = true;
            if bckend_word(cmd_arg.count1, nchar == 'E' as c_int, false).is_err() {
                clear_op_beep(op);
            }
        }
        // `g CTRL-G`: count the words, lines and bytes.
        Ok(CTRL_G) => unsafe { cursor_pos_info(ptr::null_mut()) },
        Ok(b'i') => nv_gi_cmd(cmd_arg),
        // `gI`: insert in column 1 regardless of indent.
        Ok(b'I') => {
            beginline(BeginlineOpts::NONE);
            if !check_clear_op_quit(op) {
                invoke_edit(cmd_arg, 0, 'g' as c_int, 0);
            }
        }
        // `gf`/`gF`: edit the file named under the cursor.
        Ok(b'f' | b'F') => nv_gotofile(cmd_arg),
        // `g'` and `` g` ``: jump to a mark without touching the jump
        // list. The argument is what tells `nv_gomark` it is linewise.
        Ok(b'\'') => {
            cmd_arg.arg = 1;
            nv_gomark(cmd_arg);
        }
        Ok(b'`') => nv_gomark(cmd_arg),
        Ok(b's') => do_sleep(int64_t::from(cmd_arg.count1 * 1000), false),
        // `ga`: describe the character under the cursor.
        Ok(b'a') => describe_cursor_char(),
        // `g8` shows the byte sequence; `8g8` finds an illegal one.
        Ok(b'8') => {
            if cmd_arg.count0 == 8 {
                utf_find_illegal();
            } else {
                show_utf8();
            }
        }
        // `g<`: show the previous message screen again.
        Ok(b'<') => show_sb_text(),
        // `gg`: to the first line, or the count'th.
        Ok(b'g') => {
            cmd_arg.arg = 0;
            nv_goto(cmd_arg);
        }
        // `gq` and `gw` both format; `gw` returns the cursor to where it
        // was, which is what the remembered position is for.
        Ok(b'q' | b'w') => {
            op.cursor_start = Win::current().w_cursor;
            nv_operator(cmd_arg);
        }
        // The rest of the two-character operators: `g~ gu gU g? g@`.
        Ok(b'~' | b'u' | b'U' | b'?' | b'@') => nv_operator(cmd_arg),
        // `gd`/`gD`: jump to the local or global declaration.
        Ok(b'd' | b'D') => unsafe { nv_gd(op.raw(), nchar, cmd_arg.count0) },
        // `gp`/`gP`: put and leave the cursor after the new text.
        Ok(b'p' | b'P') => nv_put(cmd_arg),
        // `go`: to a byte offset in the buffer.
        Ok(b'o') => {
            op.inclusive = false;
            goto_byte(cmd_arg.count0);
        }
        // `gQ`: Ex mode.
        Ok(b'Q') => {
            if !check_text_locked(Some(op)) && !check_clear_op_quit(op) {
                do_exmode();
            }
        }
        // `g,` and `g;`: forwards and backwards through the change list.
        Ok(b',') => nv_pcmark(cmd_arg),
        Ok(b';') => {
            cmd_arg.count1 = -cmd_arg.count1;
            nv_pcmark(cmd_arg);
        }
        // `gt`/`gT`: the next or previous tab page.
        Ok(b't') => {
            if !check_clear_op(op) {
                goto_tabpage(cmd_arg.count0);
            }
        }
        Ok(b'T') => {
            if !check_clear_op(op) {
                goto_tabpage(-cmd_arg.count1);
            }
        }
        // `g<Tab>`: the tab page used before this one.
        Ok(HTAB) => {
            if !check_clear_op(op) && !goto_tabpage_lastused() {
                clear_op_beep(op);
            }
        }
        // `g+`/`g-`: forwards and backwards through the undo tree by time.
        Ok(b'+' | b'-') => {
            if !check_clear_op_quit(op) {
                let count = if nchar == '-' as c_int {
                    -cmd_arg.count1
                } else {
                    cmd_arg.count1
                };
                undo_time(count, false, false, false);
            }
        }
        _ => clear_op_beep(op),
    }
}
