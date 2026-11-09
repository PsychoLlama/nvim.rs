//! The `g` prefix tree.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::ModMask;
use crate::keycodes::SpecialKey;
use crate::types::Key;
use crate::winlayer::{Buf, Win};

use crate::ascii::NUL;
use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::cstr;
use crate::cursor::{
    check_cursor_lnum, coladvance, gchar_cursor, get_cursor_line_len, with_cursor_rest,
};
use crate::edit::{BeginlineOpts, beginline, cursor_down, cursor_up, oneleft, oneright};
use crate::ex_cmds::describe_cursor_char;
use crate::ex_docmd::{do_cmdline_cmd, do_exmode, do_sleep};
use crate::fold::has_any_folding;
use crate::getchar::state::mod_mask;
use crate::mbyte::{cells_at, show_utf8, utf_find_illegal};
use crate::memline::Lines;
use crate::memline::goto_byte;
use crate::message::show_sb_text;
use crate::mouse::do_mouse;
use crate::normal::defs::CmdArg;
use crate::normal::{
    MT_CHAR_WISE, MT_LINE_WISE, adjust_for_sel, check_clear_op, check_clear_op_quit,
    check_text_locked, clear_op_beep, invoke_edit, nv_addsub, nv_edit, nv_gd, nv_gomark, nv_goto,
    nv_gotofile, nv_gv_cmd, nv_ident, nv_join, nv_operator, nv_pcmark, nv_put, nv_replace_mode,
    nv_screengo, nv_visual, nv_vreplace, visual_active,
};
use crate::ops::OpType;
use crate::ops::cursor_pos_info;
use crate::os::libc::int64_t;
use crate::plines::linetabsize;
use crate::search::{BACKWARD, FORWARD, current_search};
use crate::state::mode::VIsual_reselect;
use crate::state::virtual_active;
use crate::textobject::bckend_word;
use crate::types::{ColNr, LineNr};
use crate::undo::undo_time;
use crate::window::{goto_tabpage, goto_tabpage_lastused};
use core::ffi::c_int;

use crate::r#move::{
    WinValid, adjust_skipcol, sms_marker_overlap, update_curswant_force, validate_cheight,
    validate_virtcol, win_col_off2,
};

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
    let to_first_non_blank = cmd_arg.nchar.is('^');
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = false;
    let mut i;
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        // A wrapped line's first screen row can be narrower than the rest,
        // so the row the cursor is on decides where its start is.
        let width1 = win.w_view_width - win.col_off();
        let width2 = width1 + win_col_off2(win);
        validate_virtcol(win);
        i = 0;
        if win.w_virtcol >= ColNr::new(width1) && width2 > 0 {
            i = (win.w_virtcol - width1).get() / width2 * width2 + width1;
        }
        if win.w_skipcol > ColNr::ZERO && win.w_cursor.lnum == win.w_topline {
            // 'smoothscroll' hides part of the top row behind its marker;
            // the text starts after it.
            let overlap = sms_marker_overlap(win, win.w_view_width - width2);
            if overlap > 0 && i == win.w_skipcol.get() {
                i += overlap;
            }
        }
    } else {
        i = win.w_leftcol.get();
    }
    if cmd_arg.nchar.is('m') {
        i += (win.w_view_width - win.col_off()
            + if win.w_onebuf_opt.wo_wrap != 0 && i > 0 {
                win_col_off2(win)
            } else {
                0
            })
            / 2;
    }
    coladvance(win, ColNr::new(i));
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
    cmd_arg.op().motion_type = MT_CHAR_WISE;
    cmd_arg.op().inclusive = true;
    win.w_curswant = ColNr::MAX;
    if cursor_down(cmd_arg.count1 - 1, cmd_arg.op().op_type == OpType::Nop).is_err() {
        clear_op_beep(cmd_arg.op());
        return;
    }
    let mut lines = Lines::current();
    let line = lines.line(win.w_cursor.lnum);
    let byte_at = |col: ColNr| cstr::byte_at(line, usize::try_from(col).unwrap_or(0));
    // 'virtualedit' can leave the cursor on the terminator.
    if win.w_cursor.col > ColNr::ZERO && c_int::from(byte_at(win.w_cursor.col)) == NUL {
        win.w_cursor.col -= 1;
    }
    while win.w_cursor.col > ColNr::ZERO && ascii_iswhite(c_int::from(byte_at(win.w_cursor.col))) {
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
    let to_last_non_blank = cmd_arg.nchar == SpecialKey::End || cmd_arg.nchar == SpecialKey::Kend;
    op.motion_type = MT_CHAR_WISE;
    op.inclusive = true;
    if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        win.w_curswant = ColNr::MAX;
        if cmd_arg.count1 == 1 {
            let width1 = win.w_view_width - col_off;
            let width2 = width1 + win_col_off2(win);
            validate_virtcol(win);
            let mut i = width1 - 1;
            if win.w_virtcol >= ColNr::new(width1) {
                i += ((win.w_virtcol - width1).get() / width2 + 1) * width2;
            }
            coladvance(win, ColNr::new(i));
            update_curswant_force();
            // A character wider than one cell straddles the edge; step
            // back onto the last one that fits.
            if win.w_cursor.col > ColNr::ZERO
                && win.w_onebuf_opt.wo_wrap != 0
                && win.w_virtcol > ColNr::new(i)
            {
                win.w_cursor.col -= 1;
            }
        } else if !nv_screengo(op, FORWARD as c_int, cmd_arg.count1 - 1, false) {
            clear_op_beep(op);
        }
    } else {
        // Without 'wrap' the screen line is what 'sidescroll' left showing.
        if cmd_arg.count1 > 1 {
            let _ = cursor_down(cmd_arg.count1 - 1, false);
        }
        let i = win.w_leftcol + win.w_view_width - col_off - 1;
        coladvance(win, i);
        if win.w_cursor.col > ColNr::ZERO && with_cursor_rest(cells_at) > 1 {
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
    if Buf::current().b_last_insert.mark.lnum != LineNr::ZERO {
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
    cmd_arg.cmdchar = Key::ascii(b'i');
    nv_edit(cmd_arg);
}

/// `gh`, `gH` and `g CTRL-H`: start Select mode in the matching Visual kind.
/// `v`, `V` and CTRL-V sit exactly `'v' - 'h'` above `h`, `H` and CTRL-H.
fn nv_g_select(cmd_arg: &mut CmdArg) {
    cmd_arg.cmdchar = Key::from_code(cmd_arg.nchar.code() + c_int::from(b'v' - b'h'));
    cmd_arg.arg = 1;
    nv_visual(cmd_arg);
}

/// `gj` and `gk`: down and up by *screen* line -- which is the plain line move
/// when 'wrap' is off.
fn nv_g_screen_line(cmd_arg: &mut CmdArg, dir: c_int) {
    let mut op = cmd_arg.op();
    let moved = if Win::current().w_onebuf_opt.wo_wrap == 0 {
        op.motion_type = MT_LINE_WISE;
        let stop_at_end = op.op_type == OpType::Nop;
        if dir == FORWARD as c_int {
            cursor_down(cmd_arg.count1, stop_at_end).is_ok()
        } else {
            cursor_up(cmd_arg.count1, stop_at_end).is_ok()
        }
    } else {
        nv_screengo(op, dir, cmd_arg.count1, false)
    };
    if !moved {
        clear_op_beep(op);
    }
}

/// The special keys `g` accepts, which are the ones a `u8` cannot name.
///
/// Answers `false` for anything else, which sends the caller on to the byte
/// half of the tree.
fn nv_g_key(cmd_arg: &mut CmdArg, nchar: Key) -> bool {
    match nchar.special() {
        // `g<BS>` is `g CTRL-H`.
        Some(SpecialKey::Bs) => {
            cmd_arg.nchar = Key::CTRL_H;
            nv_g_select(cmd_arg);
        }
        Some(SpecialKey::Down) => nv_g_screen_line(cmd_arg, FORWARD as c_int),
        Some(SpecialKey::Up) => nv_g_screen_line(cmd_arg, BACKWARD as c_int),
        Some(SpecialKey::Home | SpecialKey::Khome) => nv_g_home_m_cmd(cmd_arg),
        Some(SpecialKey::End | SpecialKey::Kend) => nv_g_dollar_cmd(cmd_arg),
        // A mouse click after `g` acts as the CTRL-modified click.
        Some(
            SpecialKey::Middlemouse
            | SpecialKey::Middledrag
            | SpecialKey::Middlerelease
            | SpecialKey::Leftmouse
            | SpecialKey::Leftdrag
            | SpecialKey::Leftrelease
            | SpecialKey::Mousemove
            | SpecialKey::Rightmouse
            | SpecialKey::Rightdrag
            | SpecialKey::Rightrelease
            | SpecialKey::X1mouse
            | SpecialKey::X1drag
            | SpecialKey::X1release
            | SpecialKey::X2mouse
            | SpecialKey::X2drag
            | SpecialKey::X2release,
        ) => {
            mod_mask.set(ModMask::CTRL);
            do_mouse(Some(cmd_arg.op()), nchar, BACKWARD, cmd_arg.count1, false);
        }
        Some(SpecialKey::Ignore) => {}
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
    match nchar.as_byte() {
        // `g CTRL-A` and `g CTRL-X`: increment each line of the selection
        // by a growing multiple of the count. Only meaningful in Visual
        // mode.
        Some(CTRL_A | CTRL_X) => {
            if visual_active() {
                cmd_arg.arg = 1;
                cmd_arg.cmdchar = nchar;
                cmd_arg.nchar = Key::NUL;
                nv_addsub(cmd_arg);
            } else {
                clear_op_beep(op);
            }
        }
        // `gR`: virtual replace mode.
        Some(b'R') => {
            cmd_arg.arg = 1;
            nv_replace_mode(cmd_arg);
        }
        // `gr`: replace one character virtually.
        Some(b'r') => nv_vreplace(cmd_arg),
        // `g&`: repeat the last `:substitute` over the whole file, keeping
        // the flags.
        Some(b'&') => {
            let _ = do_cmdline_cmd(c"%s//~/&");
        }
        // `gv`: reselect the previous selection.
        Some(b'v') => nv_gv_cmd(cmd_arg),
        // `gV`: do not reselect it after the next Select-mode edit.
        Some(b'V') => VIsual_reselect.set(0),
        Some(b'h' | b'H' | CTRL_H) => nv_g_select(cmd_arg),
        // `gn`/`gN`: select the next/previous match of the last search.
        Some(b'N' | b'n') => {
            if current_search(cmd_arg.count1, nchar.is('n')).is_err() {
                clear_op_beep(op);
            }
        }
        Some(b'j') => nv_g_screen_line(cmd_arg, FORWARD as c_int),
        Some(b'k') => nv_g_screen_line(cmd_arg, BACKWARD as c_int),
        // `gJ`: join without inserting or removing spaces.
        Some(b'J') => nv_join(cmd_arg),
        Some(b'^' | b'0' | b'm') => nv_g_home_m_cmd(cmd_arg),
        // `gM`: the middle of the line by *text* width, or the count'th
        // percentage of it.
        Some(b'M') => {
            op.motion_type = MT_CHAR_WISE;
            op.inclusive = false;
            let width = linetabsize(Win::current(), Win::current().w_cursor.lnum);
            if cmd_arg.count0 > 0 && cmd_arg.count0 <= 100 {
                coladvance(Win::current(), ColNr::new(width * cmd_arg.count0 / 100));
            } else {
                coladvance(Win::current(), ColNr::new(width / 2));
            }
            Win::current().w_set_curswant = true;
        }
        Some(b'_') => nv_g_underscore_cmd(cmd_arg),
        Some(b'$') => nv_g_dollar_cmd(cmd_arg),
        // `g*`, `g#`, `g]` and `g CTRL-]`: the identifier searches that do
        // not anchor at a word boundary.
        Some(b'*' | b'#' | POUND_BYTE | CTRL_RSB | b']') => nv_ident(cmd_arg),
        // `ge`/`gE`: back to the end of the previous word.
        Some(b'e' | b'E') => {
            op.motion_type = MT_CHAR_WISE;
            Win::current().w_set_curswant = true;
            op.inclusive = true;
            if bckend_word(cmd_arg.count1, nchar.is('E'), false).is_err() {
                clear_op_beep(op);
            }
        }
        // `g CTRL-G`: count the words, lines and bytes.
        Some(CTRL_G) => cursor_pos_info(None),
        Some(b'i') => nv_gi_cmd(cmd_arg),
        // `gI`: insert in column 1 regardless of indent.
        Some(b'I') => {
            beginline(BeginlineOpts::NONE);
            if !check_clear_op_quit(op) {
                invoke_edit(cmd_arg, false, Key::ascii(b'g'), false);
            }
        }
        // `gf`/`gF`: edit the file named under the cursor.
        Some(b'f' | b'F') => nv_gotofile(cmd_arg),
        // `g'` and `` g` ``: jump to a mark without touching the jump
        // list. The argument is what tells `nv_gomark` it is linewise.
        Some(b'\'') => {
            cmd_arg.arg = 1;
            nv_gomark(cmd_arg);
        }
        Some(b'`') => nv_gomark(cmd_arg),
        Some(b's') => do_sleep(int64_t::from(cmd_arg.count1 * 1000), false),
        // `ga`: describe the character under the cursor.
        Some(b'a') => describe_cursor_char(),
        // `g8` shows the byte sequence; `8g8` finds an illegal one.
        Some(b'8') => {
            if cmd_arg.count0 == 8 {
                utf_find_illegal();
            } else {
                show_utf8();
            }
        }
        // `g<`: show the previous message screen again.
        Some(b'<') => show_sb_text(),
        // `gg`: to the first line, or the count'th.
        Some(b'g') => {
            cmd_arg.arg = 0;
            nv_goto(cmd_arg);
        }
        // `gq` and `gw` both format; `gw` returns the cursor to where it
        // was, which is what the remembered position is for.
        Some(b'q' | b'w') => {
            op.cursor_start = Win::current().w_cursor;
            nv_operator(cmd_arg);
        }
        // The rest of the two-character operators: `g~ gu gU g? g@`.
        Some(b'~' | b'u' | b'U' | b'?' | b'@') => nv_operator(cmd_arg),
        // `gd`/`gD`: jump to the local or global declaration.
        Some(b'd' | b'D') => nv_gd(op, nchar, cmd_arg.count0 != 0),
        // `gp`/`gP`: put and leave the cursor after the new text.
        Some(b'p' | b'P') => nv_put(cmd_arg),
        // `go`: to a byte offset in the buffer.
        Some(b'o') => {
            op.inclusive = false;
            goto_byte(cmd_arg.count0);
        }
        // `gQ`: Ex mode.
        Some(b'Q') => {
            if !check_text_locked(Some(op)) && !check_clear_op_quit(op) {
                do_exmode();
            }
        }
        // `g,` and `g;`: forwards and backwards through the change list.
        Some(b',') => nv_pcmark(cmd_arg),
        Some(b';') => {
            cmd_arg.count1 = -cmd_arg.count1;
            nv_pcmark(cmd_arg);
        }
        // `gt`/`gT`: the next or previous tab page.
        Some(b't') => {
            if !check_clear_op(op) {
                goto_tabpage(cmd_arg.count0);
            }
        }
        Some(b'T') => {
            if !check_clear_op(op) {
                goto_tabpage(-cmd_arg.count1);
            }
        }
        // `g<Tab>`: the tab page used before this one.
        Some(HTAB) => {
            if !check_clear_op(op) && !goto_tabpage_lastused() {
                clear_op_beep(op);
            }
        }
        // `g+`/`g-`: forwards and backwards through the undo tree by time.
        Some(b'+' | b'-') => {
            if !check_clear_op_quit(op) {
                let count = if nchar.is('-') {
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
