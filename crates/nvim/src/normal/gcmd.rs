//! The `g` prefix tree.

#![deny(unsafe_op_in_unsafe_fn)]

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
use crate::ex_cmds::do_ascii;
use crate::ex_docmd::{do_cmdline_cmd, do_exmode, do_sleep};
use crate::fold::has_any_folding;
use crate::main::{VIsual_reselect, curwin, mod_mask};
use crate::mbyte::{show_utf8, utf_find_illegal, utf_ptr2cells};
use crate::memline::goto_byte;
use crate::message::show_sb_text;
use crate::mouse::do_mouse;
use crate::normal::{
    CmdArg, adjust_for_sel, check_clear_op, check_clear_op_quit, check_text_locked, clear_op_beep,
    invoke_edit, kMTCharWise, kMTLineWise, nv_addsub, nv_edit, nv_gd, nv_gomark, nv_goto,
    nv_gotofile, nv_gv_cmd, nv_ident, nv_join, nv_operator, nv_pcmark, nv_put, nv_replace_mode,
    nv_screengo, nv_visual, nv_vreplace, visual_active,
};
use crate::ops::cursor_pos_info;
use crate::plines::linetabsize;
use crate::search::{BACKWARD, FORWARD, current_search};
use crate::state::virtual_active;
use crate::textobject::bckend_word;
use crate::types::{NUL, OpType, cmdarg_T, colnr_T, int64_t, linenr_T};
use crate::undo::undo_time;
use crate::window::{goto_tabpage, goto_tabpage_lastused};
use core::ffi::c_int;

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
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    let to_first_non_blank = ca.nchar == '^' as c_int;
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = false;
    let mut i;
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        // A wrapped line's first screen row can be narrower than the rest,
        // so the row the cursor is on decides where its start is.
        let width1 = win.w_view_width - unsafe { win_col_off(win.raw()) };
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
    if ca.nchar == 'm' as c_int {
        i += (win.w_view_width - unsafe { win_col_off(win.raw()) }
            + if win.w_onebuf_opt.wo_wrap != 0 && i > 0 {
                win_col_off2(win)
            } else {
                0
            })
            / 2;
    }
    coladvance(win, i);
    if to_first_non_blank {
        while ascii_iswhite(gchar_cursor()) && unsafe { oneright() }.is_ok() {}
        win.w_valid.clear(WinValid::WCOL);
    }
    win.w_set_curswant = true;
    // Inside a closed fold the wanted column is the one that was asked
    // for, not the one the fold's single displayed line has.
    if has_any_folding(win) != 0 {
        validate_cheight(win);
        if win.w_cline_folded {
            unsafe { update_curswant_force() };
        }
    }
    unsafe { adjust_skipcol() };
}

/// `g_`: the last non-blank of the line, `count1 - 1` lines down.
pub(crate) unsafe fn nv_g_underscore_cmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    ca.op().motion_type = kMTCharWise;
    ca.op().inclusive = true;
    win.w_curswant = MAXCOL as colnr_T;
    if unsafe { cursor_down(ca.count1 - 1, ca.op().op_type == OpType::Nop) }.is_err() {
        clear_op_beep(ca.op());
        return;
    }
    let line = get_cursor_line_ptr();
    // 'virtualedit' can leave the cursor on the terminator.
    if win.w_cursor.col > 0 && unsafe { *line.offset(win.w_cursor.col as isize) } as c_int == NUL {
        win.w_cursor.col -= 1;
    }
    while win.w_cursor.col > 0
        && ascii_iswhite(unsafe { *line.offset(win.w_cursor.col as isize) } as c_int)
    {
        win.w_cursor.col -= 1;
    }
    win.w_set_curswant = true;
    unsafe { adjust_for_sel(cap) };
}

/// `g$` and `g<End>`: the end of the *screen* line.
pub(crate) unsafe fn nv_g_dollar_cmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    let mut op = ca.op();
    let col_off = unsafe { win_col_off(win.raw()) };
    // `<End>` also skips back over trailing white space.
    let to_last_non_blank = ca.nchar == Key::End.code() || ca.nchar == Key::Kend.code();
    op.motion_type = kMTCharWise;
    op.inclusive = true;
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        win.w_curswant = MAXCOL as colnr_T;
        if ca.count1 == 1 {
            let width1 = win.w_view_width - col_off;
            let width2 = width1 + win_col_off2(win);
            validate_virtcol(win);
            let mut i = width1 - 1;
            if win.w_virtcol >= width1 {
                i += ((win.w_virtcol - width1) / width2 + 1) * width2;
            }
            coladvance(win, i);
            unsafe { update_curswant_force() };
            // A character wider than one cell straddles the edge; step
            // back onto the last one that fits.
            if win.w_cursor.col > 0 && win.w_onebuf_opt.wo_wrap != 0 && win.w_virtcol > i {
                win.w_cursor.col -= 1;
            }
        } else if !unsafe { nv_screengo(op.raw(), FORWARD as c_int, ca.count1 - 1, false) } {
            clear_op_beep(op);
        }
    } else {
        // Without 'wrap' the screen line is what 'sidescroll' left showing.
        if ca.count1 > 1 {
            let _ = unsafe { cursor_down(ca.count1 - 1, false) };
        }
        let i = win.w_leftcol + win.w_view_width - col_off - 1;
        coladvance(win, i);
        if win.w_cursor.col > 0 && unsafe { utf_ptr2cells(get_cursor_pos_ptr()) } > 1 {
            let vcol = win.virtual_vcol_span(win.cursor()).1;
            if vcol >= win.w_leftcol + win.w_view_width - col_off {
                win.w_cursor.col -= 1;
            }
        }
        unsafe { update_curswant_force() };
    }
    if to_last_non_blank {
        while ascii_iswhite_or_nul(gchar_cursor()) && unsafe { oneleft() }.is_ok() {}
        win.w_valid.clear(WinValid::WCOL);
    }
}

/// `gi`: insert where insert mode was left, even if the line has since got
/// shorter.
pub(crate) unsafe fn nv_gi_cmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    if cur_buf().b_last_insert.mark.lnum != 0 {
        win.w_cursor = cur_buf().b_last_insert.mark;
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
    ca.cmdchar = 'i' as c_int;
    unsafe { nv_edit(cap) };
}

/// `gh`, `gH` and `g CTRL-H`: start Select mode in the matching Visual kind.
/// `v`, `V` and CTRL-V sit exactly `'v' - 'h'` above `h`, `H` and CTRL-H.
unsafe fn nv_g_select(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.cmdchar = ca.nchar + ('v' as c_int - 'h' as c_int);
    ca.arg = 1;
    unsafe { nv_visual(cap) };
}

/// `gj` and `gk`: down and up by *screen* line -- which is the plain line move
/// when 'wrap' is off.
unsafe fn nv_g_screen_line(cap: *mut cmdarg_T, dir: c_int) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut op = ca.op();
    let moved = if cur_win().w_onebuf_opt.wo_wrap == 0 {
        op.motion_type = kMTLineWise;
        let stop_at_end = op.op_type == OpType::Nop;
        if dir == FORWARD as c_int {
            unsafe { cursor_down(ca.count1, stop_at_end).is_ok() }
        } else {
            unsafe { cursor_up(ca.count1 as linenr_T, stop_at_end).is_ok() }
        }
    } else {
        unsafe { nv_screengo(op.raw(), dir, ca.count1, false) }
    };
    if !moved {
        clear_op_beep(op);
    }
}

/// The special keys `g` accepts, which are the ones a `u8` cannot name.
///
/// Answers `false` for anything else, which sends the caller on to the byte
/// half of the tree.
unsafe fn nv_g_key(cap: *mut cmdarg_T, nchar: c_int) -> bool {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    match Key::try_from(nchar) {
        // `g<BS>` is `g CTRL-H`.
        Ok(Key::Bs) => {
            ca.nchar = Ctrl_H;
            unsafe { nv_g_select(cap) };
        }
        Ok(Key::Down) => unsafe { nv_g_screen_line(cap, FORWARD as c_int) },
        Ok(Key::Up) => unsafe { nv_g_screen_line(cap, BACKWARD as c_int) },
        Ok(Key::Home | Key::Khome) => unsafe { nv_g_home_m_cmd(cap) },
        Ok(Key::End | Key::Kend) => unsafe { nv_g_dollar_cmd(cap) },
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
            unsafe { do_mouse(ca.oap, nchar, BACKWARD as c_int, ca.count1, false) };
        }
        Ok(Key::Ignore) => {}
        _ => return false,
    }
    true
}

/// `g`, whose second character says what the command is.
pub(crate) unsafe fn nv_g_cmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut op = ca.op();
    let nchar = ca.nchar;
    if unsafe { nv_g_key(cap, nchar) } {
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
                ca.arg = 1;
                ca.cmdchar = nchar;
                ca.nchar = NUL;
                unsafe { nv_addsub(cap) };
            } else {
                clear_op_beep(op);
            }
        }
        // `gR`: virtual replace mode.
        Ok(b'R') => {
            ca.arg = 1;
            unsafe { nv_replace_mode(cap) };
        }
        // `gr`: replace one character virtually.
        Ok(b'r') => unsafe { nv_vreplace(cap) },
        // `g&`: repeat the last `:substitute` over the whole file, keeping
        // the flags.
        Ok(b'&') => {
            let _ = unsafe { do_cmdline_cmd(c"%s//~/&".as_ptr()) };
        }
        // `gv`: reselect the previous selection.
        Ok(b'v') => unsafe { nv_gv_cmd(cap) },
        // `gV`: do not reselect it after the next Select-mode edit.
        Ok(b'V') => VIsual_reselect.set(0),
        Ok(b'h' | b'H' | CTRL_H) => unsafe { nv_g_select(cap) },
        // `gn`/`gN`: select the next/previous match of the last search.
        Ok(b'N' | b'n') => {
            if unsafe { current_search(ca.count1, nchar == 'n' as c_int) }.is_err() {
                clear_op_beep(op);
            }
        }
        Ok(b'j') => unsafe { nv_g_screen_line(cap, FORWARD as c_int) },
        Ok(b'k') => unsafe { nv_g_screen_line(cap, BACKWARD as c_int) },
        // `gJ`: join without inserting or removing spaces.
        Ok(b'J') => unsafe { nv_join(cap) },
        Ok(b'^' | b'0' | b'm') => unsafe { nv_g_home_m_cmd(cap) },
        // `gM`: the middle of the line by *text* width, or the count'th
        // percentage of it.
        Ok(b'M') => {
            op.motion_type = kMTCharWise;
            op.inclusive = false;
            let width = unsafe { linetabsize(Win::new(curwin.get()), cur_win().w_cursor.lnum) };
            if ca.count0 > 0 && ca.count0 <= 100 {
                coladvance(unsafe { Win::current() }, width * ca.count0 / 100);
            } else {
                coladvance(unsafe { Win::current() }, width / 2);
            }
            cur_win().w_set_curswant = true;
        }
        Ok(b'_') => unsafe { nv_g_underscore_cmd(cap) },
        Ok(b'$') => unsafe { nv_g_dollar_cmd(cap) },
        // `g*`, `g#`, `g]` and `g CTRL-]`: the identifier searches that do
        // not anchor at a word boundary.
        Ok(b'*' | b'#' | POUND_BYTE | CTRL_RSB | b']') => unsafe { nv_ident(cap) },
        // `ge`/`gE`: back to the end of the previous word.
        Ok(b'e' | b'E') => {
            op.motion_type = kMTCharWise;
            cur_win().w_set_curswant = true;
            op.inclusive = true;
            if unsafe { bckend_word(ca.count1, nchar == 'E' as c_int, false) }.is_err() {
                clear_op_beep(op);
            }
        }
        // `g CTRL-G`: count the words, lines and bytes.
        Ok(CTRL_G) => unsafe { cursor_pos_info(ptr::null_mut()) },
        Ok(b'i') => unsafe { nv_gi_cmd(cap) },
        // `gI`: insert in column 1 regardless of indent.
        Ok(b'I') => {
            beginline(BeginlineOpts::NONE);
            if !check_clear_op_quit(op) {
                unsafe { invoke_edit(cap, 0, 'g' as c_int, 0) };
            }
        }
        // `gf`/`gF`: edit the file named under the cursor.
        Ok(b'f' | b'F') => unsafe { nv_gotofile(cap) },
        // `g'` and `` g` ``: jump to a mark without touching the jump
        // list. The argument is what tells `nv_gomark` it is linewise.
        Ok(b'\'') => {
            ca.arg = 1;
            unsafe { nv_gomark(cap) };
        }
        Ok(b'`') => unsafe { nv_gomark(cap) },
        Ok(b's') => unsafe { do_sleep((ca.count1 * 1000) as int64_t, false) },
        // `ga`: describe the character under the cursor.
        Ok(b'a') => unsafe { do_ascii(ptr::null_mut()) },
        // `g8` shows the byte sequence; `8g8` finds an illegal one.
        Ok(b'8') => {
            if ca.count0 == 8 {
                unsafe { utf_find_illegal() };
            } else {
                unsafe { show_utf8() };
            }
        }
        // `g<`: show the previous message screen again.
        Ok(b'<') => unsafe { show_sb_text() },
        // `gg`: to the first line, or the count'th.
        Ok(b'g') => {
            ca.arg = 0;
            unsafe { nv_goto(cap) };
        }
        // `gq` and `gw` both format; `gw` returns the cursor to where it
        // was, which is what the remembered position is for.
        Ok(b'q' | b'w') => {
            op.cursor_start = cur_win().w_cursor;
            unsafe { nv_operator(cap) };
        }
        // The rest of the two-character operators: `g~ gu gU g? g@`.
        Ok(b'~' | b'u' | b'U' | b'?' | b'@') => unsafe { nv_operator(cap) },
        // `gd`/`gD`: jump to the local or global declaration.
        Ok(b'd' | b'D') => unsafe { nv_gd(op.raw(), nchar, ca.count0) },
        // `gp`/`gP`: put and leave the cursor after the new text.
        Ok(b'p' | b'P') => unsafe { nv_put(cap) },
        // `go`: to a byte offset in the buffer.
        Ok(b'o') => {
            op.inclusive = false;
            unsafe { goto_byte(ca.count0) };
        }
        // `gQ`: Ex mode.
        Ok(b'Q') => {
            if !unsafe { check_text_locked(op.raw()) } && !check_clear_op_quit(op) {
                unsafe { do_exmode() };
            }
        }
        // `g,` and `g;`: forwards and backwards through the change list.
        Ok(b',') => unsafe { nv_pcmark(cap) },
        Ok(b';') => {
            ca.count1 = -ca.count1;
            unsafe { nv_pcmark(cap) };
        }
        // `gt`/`gT`: the next or previous tab page.
        Ok(b't') => {
            if !check_clear_op(op) {
                goto_tabpage(ca.count0);
            }
        }
        Ok(b'T') => {
            if !check_clear_op(op) {
                goto_tabpage(-ca.count1);
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
                    -ca.count1
                } else {
                    ca.count1
                };
                unsafe { undo_time(count, false, false, false) };
            }
        }
        _ => clear_op_beep(op),
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
