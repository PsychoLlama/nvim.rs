//! `do_window()` -- the CTRL-W commands.
//!
//! One stage per letter that follows CTRL-W (or per argument of `:wincmd`),
//! with the count already parsed: split, close, only, exchange, rotate, move
//! to an edge, navigate in a direction, resize, jump to a tag or file under
//! the cursor, open the preview window, and the tab-page forms.  Several
//! letters delegate to an Ex command, which [`run_with_count`] and
//! [`new_window`] build the command line for.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::keycodes::{
    Ctrl__, Ctrl_B, Ctrl_C, Ctrl_D, Ctrl_F, Ctrl_G, Ctrl_H, Ctrl_HAT, Ctrl_I, Ctrl_J, Ctrl_K,
    Ctrl_L, Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_S, Ctrl_T, Ctrl_V, Ctrl_W,
    Ctrl_X, Ctrl_Z, NotAKey, SpecialKey,
};
use crate::narrow::as_index;
use crate::narrow::number_as_int;
use crate::types::AutoEvent;
use crate::types::Key;
use crate::winlayer::prev_window;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::*;

use crate::buffer::{buflist_getfile, find_buf, set_pcmark};
use crate::cursor::check_cursor_lnum;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::do_ecmd;
use crate::ex_docmd::state::cmdmod;
use crate::ex_getln::curbuf_locked;
use crate::file_search::grab_file_name;
use crate::getchar::state::{KeyStuffed, KeyTyped, langmap_mapchar, vgetc_busy};
use crate::getchar::{plain_vgetc, typeahead};
use crate::guard::Keys;
use crate::mapping::langmap_adjust_mb;
use crate::memory::XString;
use crate::message::{e_buffer_nr_not_found, e_cmdwin, e_noalt};
use crate::message_fmt::emsg_text;
use crate::normal::{
    add_to_showcmd, check_text_or_curbuf_locked, do_nv_ident, reset_visual_and_resel,
};
use crate::option::vars::{P_LANGMAP, p_lrm, p_pvh, swb_flags};
use crate::options::{kOptSwbFlagUseopen, kOptSwbFlagUsetab};
use crate::pos::MAXLNUM;
use crate::quickfix::qf_view_result;
use crate::search::find_pattern_in_path;
use crate::tag::state::{g_do_tagpreview, postponed_split};
use crate::tr_c;
use crate::types::ui::kUIMultigrid;
use crate::types::{Failed, LineNr, NUL, WinConfig, int64_t};
use crate::ui::state::{Columns, Rows};
use crate::ui::ui_has;
use crate::winfloat::{WIN_CONFIG_INIT, win_new_float};
use crate::winlayer::graph::{cmdwin_type, firstwin, lastwin};

// The keys CTRL-W dispatches on, as the codes `SpecialKey::try_from` hands
// back for a plain character: a `match` pattern needs a constant.
const SPLIT: c_int = Key::ascii(b's').code();
const SPLIT_ALT: c_int = Key::ascii(b'S').code();
const VSPLIT: c_int = Key::ascii(b'v').code();
const ALTFILE: c_int = Key::ascii(b'^').code();
const NEW: c_int = Key::ascii(b'n').code();
const QUIT: c_int = Key::ascii(b'q').code();
const CLOSE: c_int = Key::ascii(b'c').code();
const PCLOSE: c_int = Key::ascii(b'z').code();
const PREVIEW: c_int = Key::ascii(b'P').code();
const ONLY: c_int = Key::ascii(b'o').code();
const NEXT: c_int = Key::ascii(b'w').code();
const PREV: c_int = Key::ascii(b'W').code();
const DOWN: c_int = Key::ascii(b'j').code();
const UP: c_int = Key::ascii(b'k').code();
const LEFT: c_int = Key::ascii(b'h').code();
const RIGHT: c_int = Key::ascii(b'l').code();
const TO_NEW_TAB: c_int = Key::ascii(b'T').code();
const TOP: c_int = Key::ascii(b't').code();
const BOTTOM: c_int = Key::ascii(b'b').code();
const LAST_USED: c_int = Key::ascii(b'p').code();
const EXCHANGE: c_int = Key::ascii(b'x').code();
const ROTATE_DOWN: c_int = Key::ascii(b'r').code();
const ROTATE_UP: c_int = Key::ascii(b'R').code();
const MOVE_TOP: c_int = Key::ascii(b'K').code();
const MOVE_BOT: c_int = Key::ascii(b'J').code();
const MOVE_LEFT: c_int = Key::ascii(b'H').code();
const MOVE_RIGHT: c_int = Key::ascii(b'L').code();
const EQUALISE: c_int = Key::ascii(b'=').code();
const TALLER: c_int = Key::ascii(b'+').code();
const SHORTER: c_int = Key::ascii(b'-').code();
const SET_HEIGHT: c_int = Key::ascii(b'_').code();
const WIDER: c_int = Key::ascii(b'>').code();
const NARROWER: c_int = Key::ascii(b'<').code();
const SET_WIDTH: c_int = Key::ascii(b'|').code();
const TAG_PREVIEW: c_int = Key::ascii(b'}').code();
const TAG_SPLIT: c_int = Key::ascii(b']').code();
const FILE: c_int = Key::ascii(b'f').code();
const FILE_LINE: c_int = Key::ascii(b'F').code();
const IDENT_ANY: c_int = Key::ascii(b'i').code();
const IDENT_DEFINE: c_int = Key::ascii(b'd').code();
const EXTENDED: c_int = Key::ascii(b'g').code();
const DETACH: c_int = Key::ascii(b'e').code();

pub fn do_window(nchar: c_int, prenum: c_int, xchar: c_int) {
    window_command(nchar, prenum, xchar);
}

/// One CTRL-W command: `nchar` is the letter, `prenum` the count before it
/// (zero for none), and `xchar` the second letter of a CTRL-W g command when
/// it has already been read.
fn window_command(nchar: c_int, prenum: c_int, xchar: c_int) {
    let prenum1 = if prenum == 0 { 1 } else { prenum };
    // The cmdline window forbids most of these; the few it allows simply do
    // not ask.
    let in_cmdwin = || {
        let locked = cmdwin_type.get() != 0;
        if locked {
            err(e_cmdwin);
        }
        locked
    };
    match SpecialKey::try_from(nchar) {
        // split the current window in two parts, horizontally
        Err(NotAKey(SPLIT | SPLIT_ALT | Ctrl_S)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel(); // stop Visual mode
            split_or_new(nchar, prenum, 0);
        }
        // split the current window in two parts, vertically
        Err(NotAKey(VSPLIT | Ctrl_V)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel();
            split_or_new(nchar, prenum, WSP_VERT);
        }
        // split the current window and edit the alternate file
        Err(NotAKey(ALTFILE | Ctrl_HAT)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel();
            split_alternate(prenum);
        }
        // open a new window
        Err(NotAKey(NEW | Ctrl_N)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel();
            new_window(nchar, prenum);
        }
        // quit the current window
        Err(NotAKey(QUIT | Ctrl_Q)) => {
            reset_visual_and_resel();
            run_with_count(c"quit", prenum);
        }
        // close the current window
        Err(NotAKey(CLOSE | Ctrl_C)) => {
            reset_visual_and_resel();
            run_with_count(c"close", prenum);
        }
        // close the preview window
        Err(NotAKey(PCLOSE | Ctrl_Z)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel();
            run_cmd(c"pclose");
        }
        // cursor to the preview window
        Err(NotAKey(PREVIEW)) => match windows().find(|wp| wp.w_onebuf_opt.wo_pvw != 0) {
            None => err(c"E441: There is no preview window"),
            Some(wp) => goto_win(wp),
        },
        // close all but the current window
        Err(NotAKey(ONLY | Ctrl_O)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel();
            run_with_count(c"only", prenum);
        }
        // cursor to the next ('w') or previous ('W') window, wrapping around
        Err(NotAKey(NEXT | Ctrl_W | PREV)) => {
            if !in_cmdwin() {
                cycle_windows(nchar, prenum);
            }
        }
        // cursor to the window below, above, left or right
        Ok(SpecialKey::Down) | Err(NotAKey(DOWN | Ctrl_J)) => {
            if !in_cmdwin() {
                goto_ver(false, prenum1);
            }
        }
        Ok(SpecialKey::Up) | Err(NotAKey(UP | Ctrl_K)) => {
            if !in_cmdwin() {
                goto_ver(true, prenum1);
            }
        }
        Ok(SpecialKey::Left | SpecialKey::Bs) | Err(NotAKey(LEFT | Ctrl_H)) => {
            if !in_cmdwin() {
                goto_hor(true, prenum1);
            }
        }
        Ok(SpecialKey::Right) | Err(NotAKey(RIGHT | Ctrl_L)) => {
            if !in_cmdwin() {
                goto_hor(false, prenum1);
            }
        }
        // move the window to a new tab page
        Err(NotAKey(TO_NEW_TAB)) => {
            if !in_cmdwin() {
                move_to_new_tabpage(prenum);
            }
        }
        // cursor to the top-left window
        Err(NotAKey(TOP | Ctrl_T)) => goto_win(first_win()),
        // cursor to the bottom-right window
        Err(NotAKey(BOTTOM | Ctrl_B)) => goto_win(lastwin_nofloating(None)),
        // cursor to the last accessed (previous) window. Upstream tests the
        // configuration without asking whether the window floats, unlike
        // [`focusable`] below.
        Err(NotAKey(LAST_USED | Ctrl_P)) => {
            let prev = prev_window().filter(|wp| !wp.w_config.hide && wp.w_config.focusable);
            match prev {
                None => beep(),
                Some(wp) => goto_win(wp),
            }
        }
        // exchange the current and the next window
        Err(NotAKey(EXCHANGE | Ctrl_X)) => {
            if !in_cmdwin() {
                exchange(prenum);
            }
        }
        // rotate the windows downwards ('r') or upwards ('R')
        Err(NotAKey(ROTATE_DOWN | Ctrl_R)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel();
            rotate(false, prenum1);
        }
        Err(NotAKey(ROTATE_UP)) => {
            if in_cmdwin() {
                return;
            }
            reset_visual_and_resel();
            rotate(true, prenum1);
        }
        // move the window to the very top, bottom, left or right
        Err(NotAKey(MOVE_TOP | MOVE_BOT | MOVE_LEFT | MOVE_RIGHT)) => {
            if !in_cmdwin() {
                move_to_edge(nchar, prenum);
            }
        }
        // make all windows the same width and/or height
        Err(NotAKey(EQUALISE)) => {
            let split = cmdmod.with(|m| m.cmod_split) & (WSP_VERT | WSP_HOR);
            let dir = if split == WSP_VERT {
                b'v'
            } else if split == WSP_HOR {
                b'h'
            } else {
                b'b'
            };
            equal(None, false, dir);
        }
        // increase, decrease or set the current window's height
        Err(NotAKey(TALLER)) => setheight_win(Win::current().w_height + prenum1, Win::current()),
        Err(NotAKey(SHORTER)) => setheight_win(Win::current().w_height - prenum1, Win::current()),
        Err(NotAKey(SET_HEIGHT | Ctrl__)) => {
            let height = if prenum != 0 {
                prenum
            } else {
                Rows.get() - number_as_int(min_set_ch.get())
            };
            setheight_win(height, Win::current());
        }
        // increase, decrease or set the current window's width
        Err(NotAKey(WIDER)) => setwidth_win(Win::current().w_width + prenum1, Win::current()),
        Err(NotAKey(NARROWER)) => setwidth_win(Win::current().w_width - prenum1, Win::current()),
        Err(NotAKey(SET_WIDTH)) => {
            let width = if prenum != 0 { prenum } else { Columns.get() };
            setwidth_win(width, Win::current());
        }
        // jump to the tag under the cursor in a new window, '}' putting it in
        // the preview window
        Err(NotAKey(TAG_PREVIEW)) => {
            if in_cmdwin() {
                return;
            }
            g_do_tagpreview.set(tagpreview_height(prenum));
            jump_to_tag(nchar, prenum);
        }
        Err(NotAKey(TAG_SPLIT | Ctrl_RSB)) => {
            if !in_cmdwin() {
                jump_to_tag(nchar, prenum);
            }
        }
        // edit the file name under the cursor in a new window
        Err(NotAKey(FILE | FILE_LINE | Ctrl_F)) => {
            if !in_cmdwin() {
                goto_file(nchar, prenum1);
            }
        }
        // go to the first occurrence of the identifier under the cursor along
        // 'path', in a new window: any match, or the definition
        Err(NotAKey(IDENT_ANY | Ctrl_I)) => {
            if !in_cmdwin() {
                find_in_path(FIND_ANY, prenum, prenum1);
            }
        }
        Err(NotAKey(IDENT_DEFINE | Ctrl_D)) => {
            if !in_cmdwin() {
                find_in_path(FIND_DEFINE, prenum, prenum1);
            }
        }
        // quickfix window only: view the result under the cursor in a new split
        Ok(SpecialKey::Kenter) | Err(NotAKey(CAR)) => {
            if buf_is_quickfix(Some(Buf::current())) {
                view_quickfix_result();
            }
        }
        // CTRL-W g: the extended commands
        Err(NotAKey(EXTENDED | Ctrl_G)) => {
            if !in_cmdwin() {
                window_g_command(prenum, prenum1, xchar);
            }
        }
        _ => beep(),
    }
}

/// CTRL-W s / CTRL-W v. Splitting the quickfix window opens a new buffer in
/// it rather than replicating the quickfix buffer.
fn split_or_new(nchar: c_int, prenum: c_int, flags: c_int) {
    if buf_is_quickfix(Some(Buf::current())) {
        new_window(nchar, prenum);
    } else {
        let _ = split(prenum, flags);
    }
}

/// CTRL-W ^ -- split the window and edit the alternate file, or buffer
/// `prenum` when there is a count.
fn split_alternate(prenum: c_int) {
    let fnum = if prenum == 0 {
        Win::current().w_alt_fnum
    } else {
        Handle::from_number(prenum)
    };
    if find_buffer(fnum).is_none() {
        if prenum == 0 {
            err(e_noalt);
        } else {
            err_number(e_buffer_nr_not_found, prenum);
        }
        return;
    }
    if !buffer_locked() && split(0, 0).is_ok() {
        open_buffer_here(fnum);
    }
}

/// CTRL-W n, and the quickfix arm of CTRL-W s / CTRL-W v: delegate to `:new`,
/// `:vnew`, or the counted form of either.
fn new_window(nchar: c_int, prenum: c_int) {
    let mut cmd = Vec::new();
    if prenum != 0 {
        cmd.extend_from_slice(prenum.to_string().as_bytes()); // window height
    }
    if nchar == c_int::from(b'v') || nchar == Ctrl_V {
        cmd.push(b'v');
    }
    cmd.extend_from_slice(b"new");
    run_cmd(&CString::new(cmd).expect("a count and letters"));
}

/// CTRL-W q / c / o -- delegate to `:quit`, `:close` or `:only`, with the
/// count appended when there is one.
fn run_with_count(cmd: &CStr, prenum: c_int) {
    if prenum <= 0 {
        run_cmd(cmd);
        return;
    }
    let mut line = cmd.to_bytes().to_vec();
    line.extend_from_slice(prenum.to_string().as_bytes());
    run_cmd(&CString::new(line).expect("a command name and a count"));
}

/// CTRL-W w / CTRL-W W -- the `prenum`th window, or the next or previous one,
/// wrapping around the list.
fn cycle_windows(nchar: c_int, prenum: c_int) {
    // `ONE_WINDOW`, which counts floats, and not `one_window()`, which does not.
    if firstwin.get() == lastwin.get() && prenum != 1 {
        beep(); // just one window
        return;
    }
    let wp = if prenum != 0 {
        nth_focusable(prenum)
    } else if nchar == c_int::from(b'W') {
        prev_focusable()
    } else {
        next_focusable()
    };
    goto_win(wp);
}

/// Whether the cursor may be put in `window`: a hidden or unfocusable float is
/// skipped over.
fn focusable(window: Win) -> bool {
    !window.w_floating || (!window.w_config.hide && window.w_config.focusable)
}

/// The `prenum`th window of the list, or the last focusable one before it when
/// the walk runs past the end.
fn nth_focusable(prenum: c_int) -> Win {
    let mut prenum = prenum;
    let mut last_focusable = first_win();
    let mut wp = first_win();
    loop {
        prenum -= 1;
        if prenum <= 0 {
            break;
        }
        if focusable(wp) {
            last_focusable = wp;
        }
        match wp.next() {
            None => break,
            Some(next) => wp = next,
        }
    }
    let mut cur = Some(wp);
    while let Some(wp) = cur.filter(|wp| !focusable(*wp)) {
        cur = wp.next();
    }
    cur.unwrap_or(last_focusable) // went past the last focusable window
}

/// The window before the current one, wrapping around to the last.
fn prev_focusable() -> Win {
    let mut cur = Some(Win::current().prev().unwrap_or_else(last_win));
    while let Some(wp) = cur.filter(|wp| !focusable(*wp)) {
        cur = wp.prev();
    }
    cur.expect("the first window never floats")
}

/// The window after the current one, wrapping around to the first.
fn next_focusable() -> Win {
    let mut cur = Win::current().next();
    while let Some(wp) = cur.filter(|wp| !focusable(*wp)) {
        cur = wp.next();
    }
    cur.unwrap_or_else(first_win)
}

/// CTRL-W T -- give the current window a tab page of its own: make the new tab
/// page first, then go back and close the window here.
fn move_to_new_tabpage(prenum: c_int) {
    if only_window(Win::current(), None) {
        only_one_message();
        return;
    }
    let oldtab = TabPage::current().id();
    let wp = Win::current().id();
    if new_tabpage(prenum, None, true).is_none() {
        return;
    }
    let Some(oldtab) = valid_tab(oldtab) else {
        return;
    };
    let newtab = TabPage::current().id();
    goto_tab(oldtab, true, true);
    if Win::current().id() == wp {
        let _ = close(Win::current(), false, false);
    }
    if let Some(newtab) = valid_tab(newtab) {
        goto_tab(newtab, true, true);
        fire(AutoEvent::TabNewEntered, Buf::current());
    }
}

/// CTRL-W H / J / K / L -- move the window to the very left, bottom, top or
/// right.
fn move_to_edge(nchar: c_int, prenum: c_int) {
    if only_window(Win::current(), None) {
        beep();
        return;
    }
    let sideways = nchar == c_int::from(b'H') || nchar == c_int::from(b'L');
    let first = nchar == c_int::from(b'H') || nchar == c_int::from(b'K');
    let dir = if sideways { WSP_VERT } else { 0 } | if first { WSP_TOP } else { WSP_BOT };
    let _ = splitmove(Win::current(), prenum, dir);
}

/// The height `'previewheight'` gives the preview window, or the count.
fn tagpreview_height(prenum: c_int) -> c_int {
    if prenum != 0 {
        prenum
    } else {
        number_as_int(p_pvh())
    }
}

/// CTRL-W ] / CTRL-W } -- jump to the tag under the cursor, in a new window or
/// in the preview window. Keeps Visual mode, so words can be selected as a
/// tag.
fn jump_to_tag(nchar: c_int, prenum: c_int) {
    postponed_split.set(if prenum != 0 { prenum } else { -1 });
    if nchar != c_int::from(b'}') {
        g_do_tagpreview.set(0);
    }
    // Execute the command right here, which "wincmd ]" in a function needs.
    do_ident(Ctrl_RSB, NUL);
    postponed_split.set(0);
}

/// CTRL-W f / F / CTRL-W CTRL-F -- edit the file named under the cursor in a
/// new window, `F` jumping to the line number that follows it.
fn goto_file(nchar: c_int, prenum1: c_int) {
    if text_or_buffer_locked() {
        return;
    }
    let mut lnum = LineNr::new(-1);
    let Some(name) = grab_filename(prenum1, &mut lnum) else {
        return;
    };
    let oldtab = TabPage::current();
    let oldwin = Win::current();
    set_pcmark();

    // If 'switchbuf' has "useopen" or "usetab" and the file is already open in
    // a window, jump to it.
    let jump = kOptSwbFlagUseopen | kOptSwbFlagUsetab;
    let mut wp = None;
    if swb_flags.get() & jump != 0 && cmdmod.with(|m| m.cmod_tab) == 0 {
        wp = find_buffer_by_name(name.as_cstr()).and_then(swbuf_goto_win);
    }
    if wp.is_none() && split(0, 0).is_ok() {
        let mut cur = Win::current();
        cur.w_onebuf_opt.wo_scb = 0;
        cur.w_onebuf_opt.wo_crb = 0;
        if edit_file(name.as_cstr()).is_err() {
            // Failed to open the file: close the window opened for it.
            let _ = close(Win::current(), false, false);
            goto_tab_win(oldtab, oldwin);
        } else {
            wp = Some(Win::current());
        }
    }
    if wp.is_some() && nchar == c_int::from(b'F') && lnum >= LineNr::ZERO {
        let mut cur = Win::current();
        cur.w_cursor.lnum = lnum;
        revalidate_cursor_lnum(cur);
        first_non_blank();
    }
}

/// CTRL-W i / d -- open a new window on the first line matching the identifier
/// under the cursor, searching along `'path'`.
fn find_in_path(kind: c_int, prenum: c_int, prenum1: c_int) {
    // A copy: if the line is changed it will be freed.
    let found = crate::normal::ident_under_cursor(FIND_IDENT);
    if found.is_empty() {
        return;
    }
    search_path(&found, kind, prenum == 0, prenum1);
    Win::current().w_set_curswant = true;
}

/// CTRL-W g -- read the second letter and dispatch on it.
fn window_g_command(prenum: c_int, prenum1: c_int, xchar: c_int) {
    // No mapping for xchar, but allow key codes.
    let raw_key = Keys::unmapped_with_codes();
    let mut xchar = if xchar == NUL { read_char() } else { xchar };
    xchar = langmap_adjust(xchar);
    drop(raw_key);
    show_command_char(xchar);

    match xchar {
        // CTRL-W g}: jump to the tag in the preview window
        TAG_PREVIEW => {
            g_do_tagpreview.set(tagpreview_height(prenum));
            g_jump_to_tag(prenum, Ctrl_RSB);
        }
        TAG_SPLIT | Ctrl_RSB => g_jump_to_tag(prenum, xchar),
        // CTRL-W gf / gF: "gf" or "gF" in a new tab page
        FILE | FILE_LINE => {
            cmdmod.with_mut(|m| m.cmod_tab = tab_index(TabPage::current()) + 1);
            goto_file(xchar, prenum1);
        }
        // CTRL-W gt / gT / g<Tab>: the next, previous or last used tab page
        TOP => goto_tab_number(prenum), // gt: the next tab page
        TO_NEW_TAB => goto_tab_number(-prenum1), // gT: the previous one
        TAB => {
            if !goto_last_used_tab() {
                beep();
            }
        }
        // CTRL-W ge: detach the window into an external UI window
        DETACH => detach_window(),
        _ => beep(),
    }
}

/// The CTRL-W g form of [`jump_to_tag`], which leaves `g_do_tagpreview` alone.
fn g_jump_to_tag(prenum: c_int, xchar: c_int) {
    postponed_split.set(if prenum != 0 { prenum } else { -1 });
    // Execute the command right here, which "wincmd g}" in a function needs.
    do_ident(c_int::from(b'g'), xchar);
    postponed_split.set(0);
}

/// CTRL-W ge -- hand the current window to the UI as an external window.
fn detach_window() {
    if Win::current().w_floating || !ui_has(kUIMultigrid) {
        beep();
        return;
    }
    let config = WinConfig {
        width: Win::current().w_width,
        height: Win::current().w_height,
        external: true,
        ..WIN_CONFIG_INIT
    };
    match win_new_float(Win::current_or_none(), false, config) {
        Ok(Some(_)) => {}
        Ok(None) => beep(),
        Err(e) => {
            err_raw(e.message_or_empty());
            beep();
        }
    }
}

// ---------------------------------------------------------------------------
// The neighbours only this file reaches

/// `LANGMAP_ADJUST(c, true)`: map `c` through `'langmap'`, but only when it
/// came from the keyboard rather than from a mapping.
fn langmap_adjust(c: c_int) -> c_int {
    let mapping = P_LANGMAP.first_byte() != 0;
    let typed = if vgetc_busy.get() != 0 {
        typeahead().maplen() == 0
    } else {
        KeyTyped.get()
    };
    if !(mapping && (p_lrm() || typed) && !KeyStuffed.get() && c >= 0) {
        return c;
    }
    if c < 256 {
        return langmap_mapchar.with(|map| c_int::from(map[as_index(c)]));
    }
    langmap_adjust_mb(Key::from_code(c)).code()
}

/// Read one character with no mapping applied.
fn read_char() -> c_int {
    plain_vgetc().code()
}

/// Show `c` in the pending-command display.
fn show_command_char(c: c_int) {
    add_to_showcmd(Key::from_code(c));
}

/// `do_nv_ident()`: run the Normal-mode command `first` `second` right here.
fn do_ident(first: c_int, second: c_int) {
    do_nv_ident(Key::from_code(first), Key::from_code(second));
}

/// Buffer `fnum`, if there is one.
fn find_buffer(fnum: Option<Handle>) -> Option<Buf> {
    find_buf(fnum)
}

/// The buffer whose name `name` expands to, if there is one.
fn find_buffer_by_name(name: &CStr) -> Option<Buf> {
    crate::buffer::find_buffer_by_name(name)
}

/// Edit buffer `fnum` in the current window, remembering the alternate file.
fn open_buffer_here(fnum: Option<Handle>) {
    let _ = buflist_getfile(fnum, LineNr::ZERO, GETF_ALT, 0);
}

/// Whether the current buffer may not be changed right now.
fn buffer_locked() -> bool {
    curbuf_locked()
}

/// [`buffer_locked`] with the text lock as well, saying why when it holds.
fn text_or_buffer_locked() -> bool {
    check_text_or_curbuf_locked(None)
}

/// The file name under the cursor, `prenum1` names in from it, with the line
/// number that follows it written to `lnum`.
fn grab_filename(prenum1: c_int, lnum: &mut LineNr) -> Option<XString> {
    grab_file_name(prenum1, Some(lnum))
}

/// `do_ecmd()`: edit file `name` in the current window, keeping the alternate.
fn edit_file(name: &CStr) -> Result<(), Failed> {
    let lnum = newlnum::LASTL as LineNr;
    do_ecmd(None, Some(name), None, None, lnum, EcmdFlags::HIDE, None)
}

/// Clamp `window`'s cursor line into its buffer.
fn revalidate_cursor_lnum(window: Win) {
    check_cursor_lnum(window);
}

/// Put the cursor on the first non-blank of its line.
fn first_non_blank() {
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
}

/// `find_pattern_in_path()` as CTRL-W i / d asks it: split a window on the
/// first, or the `prenum1`th, match of `pat` along `'path'`.
fn search_path(pat: &[u8], kind: c_int, skip_comments: bool, prenum1: c_int) {
    let (first, last) = (LineNr::ONE, MAXLNUM);
    let action = ACTION_SPLIT;
    find_pattern_in_path(
        pat,
        kDirectionNotSet,
        true,
        skip_comments,
        kind,
        prenum1,
        action,
        first,
        last,
        false,
        false,
    );
}

/// Open the quickfix entry under the cursor in a new split.
fn view_quickfix_result() {
    qf_view_result(true);
}

/// `semsg(_(fmt), n)`, for the one error that names a buffer number.
fn err_number(fmt: &'static CStr, n: c_int) {
    emsg_text(tr_c!(fmt, int64_t::from(n)));
}
