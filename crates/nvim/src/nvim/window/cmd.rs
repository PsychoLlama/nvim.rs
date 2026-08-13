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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::api::private::helpers::api_clear_error;
use crate::src::nvim::autocmd::EVENT_TABNEWENTERED;
use crate::src::nvim::buffer::{buflist_findname_exp, buflist_findnr, buflist_getfile, set_pcmark};
use crate::src::nvim::cursor::check_cursor_lnum;
use crate::src::nvim::edit::beginline;
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_getln::{ERROR_INIT, curbuf_locked};
use crate::src::nvim::file_search::grab_file_name;
use crate::src::nvim::getchar::{plain_vgetc, typebuf_maplen};
use crate::src::nvim::keycodes::{
    Ctrl__, Ctrl_B, Ctrl_C, Ctrl_D, Ctrl_F, Ctrl_G, Ctrl_H, Ctrl_HAT, Ctrl_I, Ctrl_J, Ctrl_K,
    Ctrl_L, Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_S, Ctrl_T, Ctrl_V, Ctrl_W,
    Ctrl_X, Ctrl_Z, K_BS, K_DOWN, K_KENTER, K_LEFT, K_RIGHT, K_UP,
};
use crate::src::nvim::main::{
    Columns, KeyStuffed, KeyTyped, Rows, allow_keys, cmdmod, cmdwin_type, curtab, curwin,
    e_buffer_nr_not_found, e_cmdwin, e_noalt, firstwin, g_do_tagpreview, langmap_mapchar, lastwin,
    no_mapping, p_langmap, p_lrm, p_pvh, postponed_split, prevwin, swb_flags, vgetc_busy,
};
use crate::src::nvim::mapping::langmap_adjust_mb;
use crate::src::nvim::memory::{xmemdupz, xstrlcat, xstrlcpy};
use crate::src::nvim::normal::{
    add_to_showcmd, check_text_or_curbuf_locked, do_nv_ident, find_ident_under_cursor,
    reset_VIsual_and_resel,
};
use crate::src::nvim::options::{kOptSwbFlagUseopen, kOptSwbFlagUsetab};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::quickfix::qf_view_result;
use crate::src::nvim::search::find_pattern_in_path;
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{WinConfig, exarg_T, int64_t, linenr_T, oparg_T, size_t};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::winfloat::{WIN_CONFIG_INIT, win_new_float};

// The keys CTRL-W dispatches on. `const` blocks because a cast expression
// is not a `match` pattern, and a plain integer `const` would also land in
// the generated unit-test declarations.
const SPLIT: c_int = const { b's' as c_int };
const SPLIT_ALT: c_int = const { b'S' as c_int };
const VSPLIT: c_int = const { b'v' as c_int };
const ALTFILE: c_int = const { b'^' as c_int };
const NEW: c_int = const { b'n' as c_int };
const QUIT: c_int = const { b'q' as c_int };
const CLOSE: c_int = const { b'c' as c_int };
const PCLOSE: c_int = const { b'z' as c_int };
const PREVIEW: c_int = const { b'P' as c_int };
const ONLY: c_int = const { b'o' as c_int };
const NEXT: c_int = const { b'w' as c_int };
const PREV: c_int = const { b'W' as c_int };
const DOWN: c_int = const { b'j' as c_int };
const UP: c_int = const { b'k' as c_int };
const LEFT: c_int = const { b'h' as c_int };
const RIGHT: c_int = const { b'l' as c_int };
const TO_NEW_TAB: c_int = const { b'T' as c_int };
const TOP: c_int = const { b't' as c_int };
const BOTTOM: c_int = const { b'b' as c_int };
const LAST_USED: c_int = const { b'p' as c_int };
const EXCHANGE: c_int = const { b'x' as c_int };
const ROTATE_DOWN: c_int = const { b'r' as c_int };
const ROTATE_UP: c_int = const { b'R' as c_int };
const MOVE_TOP: c_int = const { b'K' as c_int };
const MOVE_BOT: c_int = const { b'J' as c_int };
const MOVE_LEFT: c_int = const { b'H' as c_int };
const MOVE_RIGHT: c_int = const { b'L' as c_int };
const EQUALISE: c_int = const { b'=' as c_int };
const TALLER: c_int = const { b'+' as c_int };
const SHORTER: c_int = const { b'-' as c_int };
const SET_HEIGHT: c_int = const { b'_' as c_int };
const WIDER: c_int = const { b'>' as c_int };
const NARROWER: c_int = const { b'<' as c_int };
const SET_WIDTH: c_int = const { b'|' as c_int };
const TAG_PREVIEW: c_int = const { b'}' as c_int };
const TAG_SPLIT: c_int = const { b']' as c_int };
const FILE: c_int = const { b'f' as c_int };
const FILE_LINE: c_int = const { b'F' as c_int };
const IDENT_ANY: c_int = const { b'i' as c_int };
const IDENT_DEFINE: c_int = const { b'd' as c_int };
const EXTENDED: c_int = const { b'g' as c_int };
const DETACH: c_int = const { b'e' as c_int };

/// The 40-byte buffer the letters that delegate to an Ex command build their
/// command line in.
type CmdBuf = [c_char; 40];

pub unsafe extern "C" fn do_window(nchar: c_int, prenum: c_int, xchar: c_int) {
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
            err(&raw const e_cmdwin as *const c_char);
        }
        locked
    };
    match nchar {
        // split the current window in two parts, horizontally
        SPLIT | SPLIT_ALT | Ctrl_S => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel(); // stop Visual mode
            split_or_new(nchar, prenum, 0);
        }
        // split the current window in two parts, vertically
        VSPLIT | Ctrl_V => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel();
            split_or_new(nchar, prenum, WSP_VERT as c_int);
        }
        // split the current window and edit the alternate file
        ALTFILE | Ctrl_HAT => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel();
            split_alternate(prenum);
        }
        // open a new window
        NEW | Ctrl_N => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel();
            new_window(nchar, prenum);
        }
        // quit the current window
        QUIT | Ctrl_Q => {
            reset_VIsual_and_resel();
            run_with_count(c"quit", prenum);
        }
        // close the current window
        CLOSE | Ctrl_C => {
            reset_VIsual_and_resel();
            run_with_count(c"close", prenum);
        }
        // close the preview window
        PCLOSE | Ctrl_Z => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel();
            run_cmd(c"pclose".as_ptr());
        }
        // cursor to the preview window
        PREVIEW => match windows().find(|wp| wp.w_onebuf_opt.wo_pvw != 0) {
            None => err(c"E441: There is no preview window".as_ptr()),
            Some(wp) => goto_win(wp),
        },
        // close all but the current window
        ONLY | Ctrl_O => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel();
            run_with_count(c"only", prenum);
        }
        // cursor to the next ('w') or previous ('W') window, wrapping around
        NEXT | Ctrl_W | PREV => {
            if !in_cmdwin() {
                cycle_windows(nchar, prenum);
            }
        }
        // cursor to the window below, above, left or right
        DOWN | K_DOWN | Ctrl_J => {
            if !in_cmdwin() {
                goto_ver(false, prenum1);
            }
        }
        UP | K_UP | Ctrl_K => {
            if !in_cmdwin() {
                goto_ver(true, prenum1);
            }
        }
        LEFT | K_LEFT | Ctrl_H | K_BS => {
            if !in_cmdwin() {
                goto_hor(true, prenum1);
            }
        }
        RIGHT | K_RIGHT | Ctrl_L => {
            if !in_cmdwin() {
                goto_hor(false, prenum1);
            }
        }
        // move the window to a new tab page
        TO_NEW_TAB => {
            if !in_cmdwin() {
                move_to_new_tabpage(prenum);
            }
        }
        // cursor to the top-left window
        TOP | Ctrl_T => goto_win(first_win()),
        // cursor to the bottom-right window
        BOTTOM | Ctrl_B => goto_win(last_nonfloating(None)),
        // cursor to the last accessed (previous) window. Upstream tests the
        // configuration without asking whether the window floats, unlike
        // [`focusable`] below.
        LAST_USED | Ctrl_P => {
            let prev =
                valid_win(prevwin.get()).filter(|wp| !wp.w_config.hide && wp.w_config.focusable);
            match prev {
                None => beep(),
                Some(wp) => goto_win(wp),
            }
        }
        // exchange the current and the next window
        EXCHANGE | Ctrl_X => {
            if !in_cmdwin() {
                exchange(prenum);
            }
        }
        // rotate the windows downwards ('r') or upwards ('R')
        ROTATE_DOWN | Ctrl_R => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel();
            rotate(false, prenum1);
        }
        ROTATE_UP => {
            if in_cmdwin() {
                return;
            }
            reset_VIsual_and_resel();
            rotate(true, prenum1);
        }
        // move the window to the very top, bottom, left or right
        MOVE_TOP | MOVE_BOT | MOVE_LEFT | MOVE_RIGHT => {
            if !in_cmdwin() {
                move_to_edge(nchar, prenum);
            }
        }
        // make all windows the same width and/or height
        EQUALISE => {
            let split = cmdmod.with(|m| m.cmod_split) & (WSP_VERT as c_int | WSP_HOR as c_int);
            let dir = if split == WSP_VERT as c_int {
                b'v'
            } else if split == WSP_HOR as c_int {
                b'h'
            } else {
                b'b'
            };
            equal(None, false, dir as c_int);
        }
        // increase, decrease or set the current window's height
        TALLER => setheight_win(cur_win().w_height + prenum1, cur_win()),
        SHORTER => setheight_win(cur_win().w_height - prenum1, cur_win()),
        SET_HEIGHT | Ctrl__ => {
            let height = if prenum != 0 {
                prenum
            } else {
                Rows.get() - min_set_ch.get() as c_int
            };
            setheight_win(height, cur_win());
        }
        // increase, decrease or set the current window's width
        WIDER => setwidth_win(cur_win().w_width + prenum1, cur_win()),
        NARROWER => setwidth_win(cur_win().w_width - prenum1, cur_win()),
        SET_WIDTH => {
            let width = if prenum != 0 { prenum } else { Columns.get() };
            setwidth_win(width, cur_win());
        }
        // jump to the tag under the cursor in a new window, '}' putting it in
        // the preview window
        TAG_PREVIEW => {
            if in_cmdwin() {
                return;
            }
            g_do_tagpreview.set(tagpreview_height(prenum));
            jump_to_tag(nchar, prenum);
        }
        TAG_SPLIT | Ctrl_RSB => {
            if !in_cmdwin() {
                jump_to_tag(nchar, prenum);
            }
        }
        // edit the file name under the cursor in a new window
        FILE | FILE_LINE | Ctrl_F => {
            if !in_cmdwin() {
                goto_file(nchar, prenum1);
            }
        }
        // go to the first occurrence of the identifier under the cursor along
        // 'path', in a new window: any match, or the definition
        IDENT_ANY | Ctrl_I => {
            if !in_cmdwin() {
                find_in_path(FIND_ANY as c_int, prenum, prenum1);
            }
        }
        IDENT_DEFINE | Ctrl_D => {
            if !in_cmdwin() {
                find_in_path(FIND_DEFINE as c_int, prenum, prenum1);
            }
        }
        // quickfix window only: view the result under the cursor in a new split
        K_KENTER | CAR => {
            if is_quickfix(Some(cur_buf())) {
                view_quickfix_result();
            }
        }
        // CTRL-W g: the extended commands
        EXTENDED | Ctrl_G => {
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
    if is_quickfix(Some(cur_buf())) {
        new_window(nchar, prenum);
    } else {
        split(prenum, flags);
    }
}

/// CTRL-W ^ -- split the window and edit the alternate file, or buffer
/// `prenum` when there is a count.
fn split_alternate(prenum: c_int) {
    let fnum = if prenum == 0 {
        cur_win().w_alt_fnum
    } else {
        prenum
    };
    if find_buffer(fnum).is_none() {
        if prenum == 0 {
            err(&raw const e_noalt as *const c_char);
        } else {
            err_number(&raw const e_buffer_nr_not_found as *const c_char, prenum);
        }
        return;
    }
    if !buffer_locked() && split(0, 0) == OK {
        open_buffer_here(fnum);
    }
}

/// CTRL-W n, and the quickfix arm of CTRL-W s / CTRL-W v: delegate to `:new`,
/// `:vnew`, or the counted form of either.
fn new_window(nchar: c_int, prenum: c_int) {
    let mut cbuf: CmdBuf = [0; 40];
    if prenum != 0 {
        write_count(&mut cbuf, prenum as int64_t); // window height
    }
    if nchar == b'v' as c_int || nchar == Ctrl_V {
        append_str(&mut cbuf, c"v");
    }
    append_str(&mut cbuf, c"new");
    run_cmd(cbuf.as_ptr());
}

/// CTRL-W q / c / o -- delegate to `:quit`, `:close` or `:only`, with the
/// count appended when there is one.
fn run_with_count(cmd: &CStr, prenum: c_int) {
    let mut cbuf: CmdBuf = [0; 40];
    let len = copy_str(&mut cbuf, cmd);
    if prenum > 0 && len < cbuf.len() as size_t {
        write_count_at(&mut cbuf, len, prenum as int64_t);
    }
    run_cmd(cbuf.as_ptr());
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
    } else if nchar == b'W' as c_int {
        prev_focusable()
    } else {
        next_focusable()
    };
    goto_win(wp);
}

/// Whether the cursor may be put in `wp`: a hidden or unfocusable float is
/// skipped over.
fn focusable(wp: Win) -> bool {
    !wp.w_floating || (!wp.w_config.hide && wp.w_config.focusable)
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
    let mut cur = Some(cur_win().prev().unwrap_or_else(last_win));
    while let Some(wp) = cur.filter(|wp| !focusable(*wp)) {
        cur = wp.prev();
    }
    cur.expect("the first window never floats")
}

/// The window after the current one, wrapping around to the first.
fn next_focusable() -> Win {
    let mut cur = cur_win().next();
    while let Some(wp) = cur.filter(|wp| !focusable(*wp)) {
        cur = wp.next();
    }
    cur.unwrap_or_else(first_win)
}

/// CTRL-W T -- give the current window a tab page of its own: make the new tab
/// page first, then go back and close the window here.
fn move_to_new_tabpage(prenum: c_int) {
    if only_window(cur_win(), None) {
        only_one_message();
        return;
    }
    let oldtab = curtab.get();
    let wp = cur_win().raw();
    if new_tabpage(prenum, ptr::null_mut(), true).is_none() {
        return;
    }
    let Some(oldtab) = valid_tab(oldtab) else {
        return;
    };
    let newtab = curtab.get();
    goto_tab(oldtab, true, true);
    if curwin.get() == wp {
        close(cur_win(), false, false);
    }
    if let Some(newtab) = valid_tab(newtab) {
        goto_tab(newtab, true, true);
        fire(EVENT_TABNEWENTERED, cur_buf());
    }
}

/// CTRL-W H / J / K / L -- move the window to the very left, bottom, top or
/// right.
fn move_to_edge(nchar: c_int, prenum: c_int) {
    if only_window(cur_win(), None) {
        beep();
        return;
    }
    let sideways = nchar == b'H' as c_int || nchar == b'L' as c_int;
    let first = nchar == b'H' as c_int || nchar == b'K' as c_int;
    let dir = if sideways { WSP_VERT as c_int } else { 0 }
        | if first {
            WSP_TOP as c_int
        } else {
            WSP_BOT as c_int
        };
    splitmove(cur_win(), prenum, dir);
}

/// The height `'previewheight'` gives the preview window, or the count.
fn tagpreview_height(prenum: c_int) -> c_int {
    if prenum != 0 {
        prenum
    } else {
        p_pvh.get() as c_int
    }
}

/// CTRL-W ] / CTRL-W } -- jump to the tag under the cursor, in a new window or
/// in the preview window. Keeps Visual mode, so words can be selected as a
/// tag.
fn jump_to_tag(nchar: c_int, prenum: c_int) {
    postponed_split.set(if prenum != 0 { prenum } else { -1 });
    if nchar != b'}' as c_int {
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
    let mut lnum = -1 as linenr_T;
    let ptr = grab_filename(prenum1, &mut lnum);
    if ptr.is_null() {
        return;
    }
    let oldtab = curtab.get();
    let oldwin = curwin.get();
    set_pcmark();

    // If 'switchbuf' has "useopen" or "usetab" and the file is already open in
    // a window, jump to it.
    let jump = (kOptSwbFlagUseopen as c_int | kOptSwbFlagUsetab as c_int) as ::core::ffi::c_uint;
    let mut wp = None;
    if swb_flags.get() & jump != 0 && cmdmod.with(|m| m.cmod_tab) == 0 {
        wp = find_buffer_by_name(ptr).and_then(swbuf_goto_win);
    }
    if wp.is_none() && split(0, 0) == OK {
        let mut cur = cur_win();
        cur.w_onebuf_opt.wo_scb = false_0;
        cur.w_onebuf_opt.wo_crb = false_0;
        if edit_file(ptr) == FAIL {
            // Failed to open the file: close the window opened for it.
            close(cur_win(), false, false);
            // SAFETY: upstream assumes both survive the failed edit, which only
            // closed the window the split just above had made.
            unsafe { goto_tab_win(TabPage::new(oldtab), Win::new(oldwin)) };
        } else {
            wp = Some(cur_win());
        }
    }
    if wp.is_some() && nchar == b'F' as c_int && lnum >= 0 as linenr_T {
        let mut cur = cur_win();
        cur.w_cursor.lnum = lnum;
        revalidate_cursor_lnum(cur);
        first_non_blank();
    }
    free(ptr);
}

/// CTRL-W i / d -- open a new window on the first line matching the identifier
/// under the cursor, searching along `'path'`.
fn find_in_path(kind: c_int, prenum: c_int, prenum1: c_int) {
    let mut found = ptr::null_mut::<c_char>();
    let len = ident_under_cursor(&mut found);
    if len == 0 as size_t {
        return;
    }
    // Make a copy: if the line is changed it will be freed.
    let pat = dup_bytes(found, len);
    search_path(pat, len, kind, prenum == 0, prenum1);
    free(pat);
    cur_win().w_set_curswant = true_0;
}

/// CTRL-W g -- read the second letter and dispatch on it.
fn window_g_command(prenum: c_int, prenum1: c_int, xchar: c_int) {
    no_mapping.set(no_mapping.get() + 1);
    allow_keys.set(allow_keys.get() + 1); // no mapping for xchar, but allow key codes
    let mut xchar = if xchar == NUL { read_char() } else { xchar };
    xchar = langmap_adjust(xchar);
    no_mapping.set(no_mapping.get() - 1);
    allow_keys.set(allow_keys.get() - 1);
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
            cmdmod.with_mut(|m| m.cmod_tab = tab_index(cur_tab()) + 1);
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
    do_ident(b'g' as c_int, xchar);
    postponed_split.set(0);
}

/// CTRL-W ge -- hand the current window to the UI as an external window.
fn detach_window() {
    if cur_win().w_floating || !ui_has(kUIMultigrid) {
        beep();
        return;
    }
    let config = WinConfig {
        width: cur_win().w_width,
        height: cur_win().w_height,
        external: true,
        ..WIN_CONFIG_INIT
    };
    let mut error = ERROR_INIT;
    // SAFETY: a live window, its own size, and an error slot of ours.
    let made = unsafe { win_new_float(curwin.get(), false, config, &raw mut error) };
    if made.is_null() {
        err_raw(error.msg);
        // SAFETY: an error the call above filled in, which owns its message.
        unsafe { api_clear_error(&raw mut error) };
        beep();
    }
}

// ---------------------------------------------------------------------------
// The neighbours only this file reaches

/// `LANGMAP_ADJUST(c, true)`: map `c` through `'langmap'`, but only when it
/// came from the keyboard rather than from a mapping.
fn langmap_adjust(c: c_int) -> c_int {
    // SAFETY: `'langmap'` is a NUL-terminated option string.
    let mapping = unsafe { *p_langmap.get() } as c_int != NUL;
    let typed = if vgetc_busy.get() != 0 {
        // SAFETY: counts the mapping still being worked through.
        unsafe { typebuf_maplen() == 0 }
    } else {
        KeyTyped.get()
    };
    if !(mapping && (p_lrm.get() != 0 || typed) && KeyStuffed.get() == 0 && c >= 0) {
        return c;
    }
    if c < 256 {
        return langmap_mapchar.with(|map| map[c as usize] as c_int);
    }
    langmap_adjust_mb(c)
}

/// Read one character with no mapping applied.
fn read_char() -> c_int {
    // SAFETY: reads from the type-ahead buffer.
    unsafe { plain_vgetc() }
}

/// Show `c` in the pending-command display.
fn show_command_char(c: c_int) {
    add_to_showcmd(c);
}

/// `do_nv_ident()`: run the Normal-mode command `first` `second` right here.
fn do_ident(first: c_int, second: c_int) {
    // SAFETY: runs a Normal-mode command over the current window.
    unsafe { do_nv_ident(first, second) };
}

/// Buffer `fnum`, if there is one.
fn find_buffer(fnum: c_int) -> Option<Buf> {
    // SAFETY: only looks `fnum` up in the buffer list.
    unsafe { Buf::from_raw(buflist_findnr(fnum)) }
}

/// The buffer whose name `name` expands to, if there is one.
fn find_buffer_by_name(name: *mut c_char) -> Option<Buf> {
    // SAFETY: a NUL-terminated file name.
    unsafe { Buf::from_raw(buflist_findname_exp(name)) }
}

/// Edit buffer `fnum` in the current window, remembering the alternate file.
fn open_buffer_here(fnum: c_int) {
    // SAFETY: a buffer number the list was just searched for.
    unsafe { buflist_getfile(fnum, 0 as linenr_T, GETF_ALT as c_int, false_0) };
}

/// Whether the current buffer may not be changed right now.
fn buffer_locked() -> bool {
    // SAFETY: reads the editor's lock state.
    unsafe { curbuf_locked() }
}

/// [`buffer_locked`] with the text lock as well, saying why when it holds.
fn text_or_buffer_locked() -> bool {
    // SAFETY: reads the editor's lock state; a null operator means "none".
    unsafe { check_text_or_curbuf_locked(ptr::null_mut::<oparg_T>()) }
}

/// The file name under the cursor, `prenum1` names in from it, with the line
/// number that follows it written to `lnum`.
fn grab_filename(prenum1: c_int, lnum: &mut linenr_T) -> *mut c_char {
    // SAFETY: reads the current line, and writes only through `lnum`.
    unsafe { grab_file_name(prenum1, lnum) }
}

/// `do_ecmd()`: edit file `ptr` in the current window, keeping the alternate.
fn edit_file(ptr: *mut c_char) -> c_int {
    let (sfname, eap, win) = (ptr::null_mut(), ptr::null_mut::<exarg_T>(), ptr::null_mut());
    let lnum = ECMD_LASTL as linenr_T;
    // SAFETY: a NUL-terminated file name; every other argument is optional.
    unsafe { do_ecmd(0, ptr, sfname, eap, lnum, ECMD_HIDE as c_int, win) }
}

/// Clamp `wp`'s cursor line into its buffer.
fn revalidate_cursor_lnum(mut wp: Win) {
    // SAFETY: a live window.
    unsafe { check_cursor_lnum(wp.raw()) };
}

/// Put the cursor on the first non-blank of its line.
fn first_non_blank() {
    // SAFETY: reads the current window's line.
    unsafe { beginline(BL_SOL as c_int | BL_FIX as c_int) };
}

/// The identifier under the cursor: its length, and a pointer into the line,
/// which the caller must copy before the line can move.
fn ident_under_cursor(found: &mut *mut c_char) -> size_t {
    // SAFETY: writes only through `found`; a null column means "do not report".
    unsafe { find_ident_under_cursor(found, FIND_IDENT as c_int, ptr::null_mut()) }
}

/// An owned NUL-terminated copy of the first `len` bytes of `src`.
fn dup_bytes(src: *mut c_char, len: size_t) -> *mut c_char {
    // SAFETY: `len` bytes the caller has just measured.
    unsafe { xmemdupz(src.cast(), len) }.cast::<c_char>()
}

/// `find_pattern_in_path()` as CTRL-W i / d asks it: split a window on the
/// first, or the `prenum1`th, match of `pat` along `'path'`.
fn search_path(pat: *mut c_char, len: size_t, kind: c_int, skip_comments: bool, prenum1: c_int) {
    let (first, last) = (1 as linenr_T, MAXLNUM as linenr_T);
    let action = ACTION_SPLIT as c_int;
    // SAFETY: a NUL-terminated pattern of `len` bytes.
    unsafe {
        find_pattern_in_path(
            pat,
            kDirectionNotSet,
            len,
            true,
            skip_comments,
            kind,
            prenum1,
            action,
            first,
            last,
            false,
            false,
        )
    };
}

/// Open the quickfix entry under the cursor in a new split.
fn view_quickfix_result() {
    // SAFETY: reads the quickfix list of the current window.
    unsafe { qf_view_result(true) };
}

/// `semsg(_(fmt), n)`, for the one error that names a buffer number.
fn err_number(fmt: *const c_char, n: c_int) {
    // SAFETY: a NUL-terminated message static.
    let msg = unsafe { gettext(fmt) };
    // SAFETY: a translated format taking one number, and the number.
    let _: bool = unsafe { semsg_c!(msg, n as int64_t) };
}

/// `xstrlcpy(buf, s, sizeof(buf))`, answering the length it wanted.
fn copy_str(buf: &mut CmdBuf, s: &CStr) -> size_t {
    let (dst, room) = (buf.as_mut_ptr(), buf.len() as size_t);
    // SAFETY: a buffer of its own length and a NUL-terminated string.
    unsafe { xstrlcpy(dst, s.as_ptr(), room) }
}

/// `xstrlcat(buf, s, sizeof(buf))`.
fn append_str(buf: &mut CmdBuf, s: &CStr) {
    let (dst, room) = (buf.as_mut_ptr(), buf.len() as size_t);
    // SAFETY: as above; `xstrlcat` keeps the buffer NUL-terminated.
    unsafe { xstrlcat(dst, s.as_ptr(), room) };
}

/// Print `n` at the start of `buf`, leaving room for the longest suffix
/// [`new_window`] appends.
fn write_count(buf: &mut CmdBuf, n: int64_t) {
    let (dst, room) = (buf.as_mut_ptr(), buf.len() as size_t - 5);
    // SAFETY: a buffer of at least `room` bytes, and a format taking a number.
    unsafe { vim_snprintf(dst, room, c"%ld".as_ptr(), n) };
}

/// Print `n` `at` bytes into `buf`, which [`run_with_count`] has filled to
/// there.
fn write_count_at(buf: &mut CmdBuf, at: size_t, n: int64_t) {
    let room = buf.len() as size_t - at;
    let dst = &raw mut buf[at];
    // SAFETY: `at` is within the buffer, which has `room` bytes left.
    unsafe { vim_snprintf(dst, room, c"%ld".as_ptr(), n) };
}
