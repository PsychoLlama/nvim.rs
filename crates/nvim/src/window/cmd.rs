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

use crate::autocmd::defs::AutoEvent;
use crate::buffer::GETF_ALT;
use crate::ex_cmds::{EcmdFlags, do_ecmd, newlnum};
use crate::keycodes::SpecialKey;
use crate::narrow::number_as_int;
use crate::normal::FIND_IDENT;
use crate::search::{ACTION_SPLIT, FIND_ANY, FIND_DEFINE, find_pattern_in_path};
use crate::types::{Key, KeyKind};
use crate::winlayer::prev_window;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::*;

use crate::buffer::{buflist_getfile, find_buf, set_pcmark};
use crate::cursor::check_cursor_lnum;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_docmd::state::CMDMOD;
use crate::ex_getln::curbuf_locked;
use crate::file_search::grab_file_name;
use crate::getchar::state::{KEY_STUFFED, KEY_TYPED, LANGMAP_MAPCHAR, VGETC_BUSY};
use crate::getchar::{plain_vgetc, typeahead};
use crate::guard::Keys;
use crate::mapping::langmap_adjust_mb;
use crate::memory::XString;
use crate::message::E_BUFFER_NR_NOT_FOUND;
use crate::message_fmt::emsg_text;
use crate::normal::{
    add_to_showcmd, check_text_or_curbuf_locked, do_nv_ident, reset_visual_and_resel,
};
use crate::option::vars::{P_LANGMAP, SWB_FLAGS, p_lrm, p_pvh};
use crate::options::{OPT_SWB_FLAG_USEOPEN, OPT_SWB_FLAG_USETAB};
use crate::os::libc::int64_t;
use crate::pos::MAXLNUM;
use crate::quickfix::qf_view_result;
use crate::tag::state::{G_DO_TAGPREVIEW, POSTPONED_SPLIT};
use crate::tr_c;
use crate::types::{Failed, LineNr, WinConfig};
use crate::ui::state::UI_MULTIGRID;
use crate::ui::state::{COLUMNS, ROWS};
use crate::ui::ui_has;
use crate::winfloat::{WIN_CONFIG_INIT, win_new_float};
use crate::winlayer::graph::{CMDWIN_TYPE, FIRSTWIN, LASTWIN};

// The letters CTRL-W dispatches on, beside the control characters
// (`Key::CTRL_*`) and the named keys that do the same.
const SPLIT: Key = Key::ascii(b's');
const SPLIT_ALT: Key = Key::ascii(b'S');
const VSPLIT: Key = Key::ascii(b'v');
const ALTFILE: Key = Key::ascii(b'^');
const NEW: Key = Key::ascii(b'n');
const QUIT: Key = Key::ascii(b'q');
const CLOSE: Key = Key::ascii(b'c');
const PCLOSE: Key = Key::ascii(b'z');
const PREVIEW: Key = Key::ascii(b'P');
const ONLY: Key = Key::ascii(b'o');
const NEXT: Key = Key::ascii(b'w');
const PREV: Key = Key::ascii(b'W');
const DOWN: Key = Key::ascii(b'j');
const UP: Key = Key::ascii(b'k');
const LEFT: Key = Key::ascii(b'h');
const RIGHT: Key = Key::ascii(b'l');
const TO_NEW_TAB: Key = Key::ascii(b'T');
const TOP: Key = Key::ascii(b't');
const BOTTOM: Key = Key::ascii(b'b');
const LAST_USED: Key = Key::ascii(b'p');
const EXCHANGE: Key = Key::ascii(b'x');
const ROTATE_DOWN: Key = Key::ascii(b'r');
const ROTATE_UP: Key = Key::ascii(b'R');
const MOVE_TOP: Key = Key::ascii(b'K');
const MOVE_BOT: Key = Key::ascii(b'J');
const MOVE_LEFT: Key = Key::ascii(b'H');
const MOVE_RIGHT: Key = Key::ascii(b'L');
const EQUALISE: Key = Key::ascii(b'=');
const TALLER: Key = Key::ascii(b'+');
const SHORTER: Key = Key::ascii(b'-');
const SET_HEIGHT: Key = Key::ascii(b'_');
const WIDER: Key = Key::ascii(b'>');
const NARROWER: Key = Key::ascii(b'<');
const SET_WIDTH: Key = Key::ascii(b'|');
const TAG_PREVIEW: Key = Key::ascii(b'}');
const TAG_SPLIT: Key = Key::ascii(b']');
const FILE: Key = Key::ascii(b'f');
const FILE_LINE: Key = Key::ascii(b'F');
const IDENT_ANY: Key = Key::ascii(b'i');
const IDENT_DEFINE: Key = Key::ascii(b'd');
const EXTENDED: Key = Key::ascii(b'g');
const DETACH: Key = Key::ascii(b'e');

pub fn do_window(nchar: Key, prenum: c_int, xchar: Key) {
    window_command(nchar, prenum, xchar);
}

/// One CTRL-W command: `nchar` is the letter, `prenum` the count before it
/// (zero for none), and `xchar` the second letter of a CTRL-W g command when
/// it has already been read.
fn window_command(nchar: Key, prenum: c_int, xchar: Key) {
    let Some(command) = WindowCommand::of(nchar) else {
        beep();
        return;
    };
    // The cmdline window forbids most of these; the rest simply do not ask.
    if command.refused_in_cmdwin() && CMDWIN_TYPE.get() != 0 {
        err(WinError::CmdWin);
        return;
    }
    if command.ends_visual() {
        reset_visual_and_resel();
    }
    let prenum1 = if prenum == 0 { 1 } else { prenum };
    match command {
        WindowCommand::Split => split_or_new(nchar, prenum, 0),
        WindowCommand::VSplit => split_or_new(nchar, prenum, WSP_VERT),
        WindowCommand::SplitAlternate => split_alternate(prenum),
        WindowCommand::New => new_window(nchar, prenum),
        WindowCommand::Quit => run_with_count(c"quit", prenum),
        WindowCommand::Close => run_with_count(c"close", prenum),
        WindowCommand::ClosePreview => run_cmd(c"pclose"),
        WindowCommand::GotoPreview => match windows().find(|wp| wp.w_onebuf_opt.wo_pvw != 0) {
            None => {
                err(WinError::NoPreviewWindow);
            }
            Some(wp) => goto_win(wp),
        },
        WindowCommand::Only => run_with_count(c"only", prenum),
        WindowCommand::Cycle => cycle_windows(nchar, prenum),
        WindowCommand::Down => goto_ver(false, prenum1),
        WindowCommand::Up => goto_ver(true, prenum1),
        WindowCommand::Left => goto_hor(true, prenum1),
        WindowCommand::Right => goto_hor(false, prenum1),
        WindowCommand::ToNewTab => move_to_new_tabpage(prenum),
        WindowCommand::Top => goto_win(first_win()),
        WindowCommand::Bottom => goto_win(lastwin_nofloating(None)),
        WindowCommand::LastUsed => goto_last_used(),
        WindowCommand::Exchange => exchange(prenum),
        WindowCommand::RotateDown => rotate(false, prenum1),
        WindowCommand::RotateUp => rotate(true, prenum1),
        WindowCommand::MoveToEdge => move_to_edge(nchar, prenum),
        WindowCommand::Equalise => equalise(),
        WindowCommand::Taller => setheight_win(Win::current().w_height + prenum1, Win::current()),
        WindowCommand::Shorter => setheight_win(Win::current().w_height - prenum1, Win::current()),
        WindowCommand::SetHeight => {
            let height = if prenum != 0 {
                prenum
            } else {
                ROWS.get() - number_as_int(MIN_SET_CH.get())
            };
            setheight_win(height, Win::current());
        }
        WindowCommand::Wider => setwidth_win(Win::current().w_width + prenum1, Win::current()),
        WindowCommand::Narrower => setwidth_win(Win::current().w_width - prenum1, Win::current()),
        WindowCommand::SetWidth => {
            let width = if prenum != 0 { prenum } else { COLUMNS.get() };
            setwidth_win(width, Win::current());
        }
        WindowCommand::TagPreview => {
            G_DO_TAGPREVIEW.set(tagpreview_height(prenum));
            jump_to_tag(nchar, prenum);
        }
        WindowCommand::TagSplit => jump_to_tag(nchar, prenum),
        WindowCommand::File => goto_file(nchar, prenum1),
        WindowCommand::IdentAny => find_in_path(FIND_ANY, prenum, prenum1),
        WindowCommand::IdentDefine => find_in_path(FIND_DEFINE, prenum, prenum1),
        WindowCommand::QuickfixView => {
            if buf_is_quickfix(Some(Buf::current())) {
                view_quickfix_result();
            }
        }
        WindowCommand::Extended => window_g_command(prenum, prenum1, xchar),
    }
}

/// What a CTRL-W letter asks for, read from the key alone. Several keep the
/// letter itself as an argument (`s` vs `S`, `w` vs `W`, `K`/`J`/`H`/`L`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowCommand {
    /// Split the current window in two, horizontally.
    Split,
    /// Split the current window in two, vertically.
    VSplit,
    /// Split the current window and edit the alternate file.
    SplitAlternate,
    /// Open a new window.
    New,
    /// Quit the current window.
    Quit,
    /// Close the current window.
    Close,
    /// Close the preview window.
    ClosePreview,
    /// Cursor to the preview window.
    GotoPreview,
    /// Close all but the current window.
    Only,
    /// Cursor to the next (`w`) or previous (`W`) window, wrapping around.
    Cycle,
    /// Cursor to the window below.
    Down,
    /// Cursor to the window above.
    Up,
    /// Cursor to the window on the left.
    Left,
    /// Cursor to the window on the right.
    Right,
    /// Move the window to a new tab page.
    ToNewTab,
    /// Cursor to the top-left window.
    Top,
    /// Cursor to the bottom-right window.
    Bottom,
    /// Cursor to the last accessed (previous) window.
    LastUsed,
    /// Exchange the current and the next window.
    Exchange,
    /// Rotate the windows downwards.
    RotateDown,
    /// Rotate the windows upwards.
    RotateUp,
    /// Move the window to the very top, bottom, left or right.
    MoveToEdge,
    /// Make all windows the same width and/or height.
    Equalise,
    /// Increase the current window's height.
    Taller,
    /// Decrease the current window's height.
    Shorter,
    /// Set the current window's height.
    SetHeight,
    /// Increase the current window's width.
    Wider,
    /// Decrease the current window's width.
    Narrower,
    /// Set the current window's width.
    SetWidth,
    /// Jump to the tag under the cursor in the preview window.
    TagPreview,
    /// Jump to the tag under the cursor in a new window.
    TagSplit,
    /// Edit the file name under the cursor in a new window.
    File,
    /// The first occurrence of the identifier under the cursor along
    /// 'path', in a new window.
    IdentAny,
    /// The definition of the identifier under the cursor along 'path', in a
    /// new window.
    IdentDefine,
    /// Quickfix window only: view the result under the cursor in a new split.
    QuickfixView,
    /// CTRL-W g: the extended commands.
    Extended,
}

impl WindowCommand {
    /// The command `nchar` names, or `None` for a key CTRL-W does not take.
    fn of(nchar: Key) -> Option<WindowCommand> {
        Some(match nchar.kind() {
            KeyKind::Char(SPLIT | SPLIT_ALT | Key::CTRL_S) => WindowCommand::Split,
            KeyKind::Char(VSPLIT | Key::CTRL_V) => WindowCommand::VSplit,
            KeyKind::Char(ALTFILE | Key::CTRL_HAT) => WindowCommand::SplitAlternate,
            KeyKind::Char(NEW | Key::CTRL_N) => WindowCommand::New,
            KeyKind::Char(QUIT | Key::CTRL_Q) => WindowCommand::Quit,
            KeyKind::Char(CLOSE | Key::CTRL_C) => WindowCommand::Close,
            KeyKind::Char(PCLOSE | Key::CTRL_Z) => WindowCommand::ClosePreview,
            KeyKind::Char(PREVIEW) => WindowCommand::GotoPreview,
            KeyKind::Char(ONLY | Key::CTRL_O) => WindowCommand::Only,
            KeyKind::Char(NEXT | Key::CTRL_W | PREV) => WindowCommand::Cycle,
            KeyKind::Special(SpecialKey::Down) | KeyKind::Char(DOWN | Key::CTRL_J) => {
                WindowCommand::Down
            }
            KeyKind::Special(SpecialKey::Up) | KeyKind::Char(UP | Key::CTRL_K) => WindowCommand::Up,
            KeyKind::Special(SpecialKey::Left | SpecialKey::Bs)
            | KeyKind::Char(LEFT | Key::CTRL_H) => WindowCommand::Left,
            KeyKind::Special(SpecialKey::Right) | KeyKind::Char(RIGHT | Key::CTRL_L) => {
                WindowCommand::Right
            }
            KeyKind::Char(TO_NEW_TAB) => WindowCommand::ToNewTab,
            KeyKind::Char(TOP | Key::CTRL_T) => WindowCommand::Top,
            KeyKind::Char(BOTTOM | Key::CTRL_B) => WindowCommand::Bottom,
            KeyKind::Char(LAST_USED | Key::CTRL_P) => WindowCommand::LastUsed,
            KeyKind::Char(EXCHANGE | Key::CTRL_X) => WindowCommand::Exchange,
            KeyKind::Char(ROTATE_DOWN | Key::CTRL_R) => WindowCommand::RotateDown,
            KeyKind::Char(ROTATE_UP) => WindowCommand::RotateUp,
            KeyKind::Char(MOVE_TOP | MOVE_BOT | MOVE_LEFT | MOVE_RIGHT) => {
                WindowCommand::MoveToEdge
            }
            KeyKind::Char(EQUALISE) => WindowCommand::Equalise,
            KeyKind::Char(TALLER) => WindowCommand::Taller,
            KeyKind::Char(SHORTER) => WindowCommand::Shorter,
            KeyKind::Char(SET_HEIGHT | Key::CTRL_UNDERSCORE) => WindowCommand::SetHeight,
            KeyKind::Char(WIDER) => WindowCommand::Wider,
            KeyKind::Char(NARROWER) => WindowCommand::Narrower,
            KeyKind::Char(SET_WIDTH) => WindowCommand::SetWidth,
            KeyKind::Char(TAG_PREVIEW) => WindowCommand::TagPreview,
            KeyKind::Char(TAG_SPLIT | Key::CTRL_RSB) => WindowCommand::TagSplit,
            KeyKind::Char(FILE | FILE_LINE | Key::CTRL_F) => WindowCommand::File,
            KeyKind::Char(IDENT_ANY | Key::CTRL_I) => WindowCommand::IdentAny,
            KeyKind::Char(IDENT_DEFINE | Key::CTRL_D) => WindowCommand::IdentDefine,
            KeyKind::Special(SpecialKey::Kenter) | KeyKind::Char(Key::CTRL_M) => {
                WindowCommand::QuickfixView
            }
            KeyKind::Char(EXTENDED | Key::CTRL_G) => WindowCommand::Extended,
            _ => return None,
        })
    }

    /// Whether the cmdline window refuses the command (`E11`).
    fn refused_in_cmdwin(self) -> bool {
        !matches!(
            self,
            WindowCommand::Quit
                | WindowCommand::Close
                | WindowCommand::GotoPreview
                | WindowCommand::Top
                | WindowCommand::Bottom
                | WindowCommand::LastUsed
                | WindowCommand::Equalise
                | WindowCommand::Taller
                | WindowCommand::Shorter
                | WindowCommand::SetHeight
                | WindowCommand::Wider
                | WindowCommand::Narrower
                | WindowCommand::SetWidth
                | WindowCommand::QuickfixView
        )
    }

    /// Whether the command stops Visual mode before it runs.
    fn ends_visual(self) -> bool {
        matches!(
            self,
            WindowCommand::Split
                | WindowCommand::VSplit
                | WindowCommand::SplitAlternate
                | WindowCommand::New
                | WindowCommand::Quit
                | WindowCommand::Close
                | WindowCommand::ClosePreview
                | WindowCommand::Only
                | WindowCommand::RotateDown
                | WindowCommand::RotateUp
        )
    }
}

/// CTRL-W p: cursor to the last accessed window. Upstream tests the
/// configuration without asking whether the window floats, unlike
/// [`focusable`] below.
fn goto_last_used() {
    let prev = prev_window().filter(|wp| !wp.w_config.hide && wp.w_config.focusable);
    match prev {
        None => beep(),
        Some(wp) => goto_win(wp),
    }
}

/// CTRL-W =: every window the same width and/or height, in the direction a
/// `:vertical`/`:horizontal` modifier asks for.
fn equalise() {
    let split = CMDMOD.with(|m| m.cmod_split) & (WSP_VERT | WSP_HOR);
    let dir = if split == WSP_VERT {
        b'v'
    } else if split == WSP_HOR {
        b'h'
    } else {
        b'b'
    };
    equal(None, false, dir);
}

/// CTRL-W s / CTRL-W v. Splitting the quickfix window opens a new buffer in
/// it rather than replicating the quickfix buffer.
fn split_or_new(nchar: Key, prenum: c_int, flags: c_int) {
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
            err(WinError::NoAlternate);
        } else {
            err_number(E_BUFFER_NR_NOT_FOUND, prenum);
        }
        return;
    }
    if !buffer_locked() && split(0, 0).is_ok() {
        open_buffer_here(fnum);
    }
}

/// CTRL-W n, and the quickfix arm of CTRL-W s / CTRL-W v: delegate to `:new`,
/// `:vnew`, or the counted form of either.
fn new_window(nchar: Key, prenum: c_int) {
    let mut cmd = Vec::new();
    if prenum != 0 {
        cmd.extend_from_slice(prenum.to_string().as_bytes()); // window height
    }
    if nchar == VSPLIT || nchar == Key::CTRL_V {
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
fn cycle_windows(nchar: Key, prenum: c_int) {
    // `ONE_WINDOW`, which counts floats, and not `one_window()`, which does not.
    if FIRSTWIN.get() == LASTWIN.get() && prenum != 1 {
        beep(); // just one window
        return;
    }
    let wp = if prenum != 0 {
        nth_focusable(prenum)
    } else if nchar == PREV {
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
    if new_tabpage(prenum, None, true).is_err() {
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
fn move_to_edge(nchar: Key, prenum: c_int) {
    if only_window(Win::current(), None) {
        beep();
        return;
    }
    let sideways = nchar == MOVE_LEFT || nchar == MOVE_RIGHT;
    let first = nchar == MOVE_LEFT || nchar == MOVE_TOP;
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
fn jump_to_tag(nchar: Key, prenum: c_int) {
    POSTPONED_SPLIT.set(if prenum != 0 { prenum } else { -1 });
    if nchar != TAG_PREVIEW {
        G_DO_TAGPREVIEW.set(0);
    }
    // Execute the command right here, which "wincmd ]" in a function needs.
    do_nv_ident(Key::CTRL_RSB, Key::NUL);
    POSTPONED_SPLIT.set(0);
}

/// CTRL-W f / F / CTRL-W CTRL-F -- edit the file named under the cursor in a
/// new window, `F` jumping to the line number that follows it.
fn goto_file(nchar: Key, prenum1: c_int) {
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
    let jump = OPT_SWB_FLAG_USEOPEN | OPT_SWB_FLAG_USETAB;
    let mut wp = None;
    if SWB_FLAGS.get() & jump != 0 && CMDMOD.with(|m| m.cmod_tab) == 0 {
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
    if wp.is_some() && nchar == FILE_LINE && lnum >= LineNr::ZERO {
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
fn window_g_command(prenum: c_int, prenum1: c_int, xchar: Key) {
    // No mapping for xchar, but allow key codes.
    let raw_key = Keys::unmapped_with_codes();
    let mut xchar = if xchar.is_nul() { plain_vgetc() } else { xchar };
    xchar = langmap_adjust(xchar);
    drop(raw_key);
    add_to_showcmd(xchar);

    match xchar {
        // CTRL-W g}: jump to the tag in the preview window
        TAG_PREVIEW => {
            G_DO_TAGPREVIEW.set(tagpreview_height(prenum));
            g_jump_to_tag(prenum, Key::CTRL_RSB);
        }
        TAG_SPLIT | Key::CTRL_RSB => g_jump_to_tag(prenum, xchar),
        // CTRL-W gf / gF: "gf" or "gF" in a new tab page
        FILE | FILE_LINE => {
            CMDMOD.with_mut(|m| m.cmod_tab = tab_index(TabPage::current()) + 1);
            goto_file(xchar, prenum1);
        }
        // CTRL-W gt / gT / g<Tab>: the next, previous or last used tab page
        TOP => goto_tab_number(prenum), // gt: the next tab page
        TO_NEW_TAB => goto_tab_number(-prenum1), // gT: the previous one
        Key::CTRL_I => {
            if !goto_last_used_tab() {
                beep();
            }
        }
        // CTRL-W ge: detach the window into an external UI window
        DETACH => detach_window(),
        _ => beep(),
    }
}

/// The CTRL-W g form of [`jump_to_tag`], which leaves `G_DO_TAGPREVIEW` alone.
fn g_jump_to_tag(prenum: c_int, xchar: Key) {
    POSTPONED_SPLIT.set(if prenum != 0 { prenum } else { -1 });
    // Execute the command right here, which "wincmd g}" in a function needs.
    do_nv_ident(EXTENDED, xchar);
    POSTPONED_SPLIT.set(0);
}

/// CTRL-W ge -- hand the current window to the UI as an external window.
fn detach_window() {
    if Win::current().w_floating || !ui_has(UI_MULTIGRID) {
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
fn langmap_adjust(c: Key) -> Key {
    let mapping = P_LANGMAP.first_byte() != 0;
    let typed = if VGETC_BUSY.get() != 0 {
        typeahead().maplen() == 0
    } else {
        KEY_TYPED.get()
    };
    if !(mapping && (p_lrm() || typed) && !KEY_STUFFED.get() && !c.is_special()) {
        return c;
    }
    if let Some(b) = c.as_byte() {
        return LANGMAP_MAPCHAR.with(|map| Key::byte(map[usize::from(b)]));
    }
    langmap_adjust_mb(c)
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
        None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_letter_and_its_control_key_name_one_command() {
        let of = WindowCommand::of;
        assert_eq!(of(SPLIT), Some(WindowCommand::Split));
        assert_eq!(of(Key::CTRL_S), Some(WindowCommand::Split));
        assert_eq!(of(Key::CTRL_W), Some(WindowCommand::Cycle));
        assert_eq!(of(PREV), Some(WindowCommand::Cycle));
        assert_eq!(
            of(Key::from_special(SpecialKey::Bs)),
            Some(WindowCommand::Left)
        );
        assert_eq!(of(Key::CTRL_M), Some(WindowCommand::QuickfixView));
        // CTRL-W R has no control-key twin; CTRL-R is `r`.
        assert_eq!(of(Key::CTRL_R), Some(WindowCommand::RotateDown));
        assert_eq!(of(Key::ascii(b'Q')), None);
    }

    #[test]
    fn closing_and_resizing_work_in_the_cmdline_window() {
        assert!(!WindowCommand::Quit.refused_in_cmdwin());
        assert!(!WindowCommand::SetHeight.refused_in_cmdwin());
        assert!(WindowCommand::Split.refused_in_cmdwin());
        assert!(WindowCommand::Down.refused_in_cmdwin());
        // `:quit` ends Visual mode though the cmdline window allows it;
        // moving between windows does neither.
        assert!(WindowCommand::Quit.ends_visual());
        assert!(!WindowCommand::Down.ends_visual());
        assert!(!WindowCommand::TagPreview.ends_visual());
    }
}
