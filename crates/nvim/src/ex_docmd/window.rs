//! Splitting, resizing, moving between and listing windows and tab pages.
//!
//! Every function here is an Ex command handler reached through the command
//! table, so the file is glue: it reads an `ExArg`, works out what the
//! command meant, and calls into the window family. [`Ex`] carries the
//! command's own storage and `winlayer`'s [`Win`]/[`TabPage`] carry the
//! editor's, which leaves only the calls into modules that are still
//! transpiled inside an `unsafe` block.
#![forbid(unsafe_code)]

use crate::autocmd::fire_autocmds;
use crate::charset::{getdigits_at, getdigits_int_at};
use crate::file_search::find_nth_file_in_path;
use crate::memory::ThinCString;
use crate::memory::XString;
use crate::message::msg_display_bytes;
use crate::message_fmt::msg_bytes;
use crate::os::env::home_replace_in;
use crate::semsg;
use crate::types::AutoEvent;
use crate::types::CmdIdx;
use crate::winlayer::WinId;
use crate::winlayer::last_used_tab;
use core::ffi::{CStr, c_int};
use core::ptr;
use std::ffi::CString;

use crate::buffer::{buf_is_quickfix, buf_spname_owned};
use crate::drawscreen::state::must_redraw;
use crate::drawscreen::{UPD_CLEAR, UPD_VALID, screen_resize};
use crate::ex_cmds::prepare_tagpreview;
use crate::ex_docmd::argopt::get_tabpage_arg;
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::display::ex_redraw;
use crate::ex_docmd::file::{do_exbuffer, do_exedit};
use crate::ex_docmd::onecmd::fresh_exarg;
use crate::ex_docmd::path::findfunc_find_file;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::tags::ex_findpat;
use crate::file_search::FileNameOpts;
use crate::getchar::state::got_int;
use crate::highlight_group::HLF_T;
use crate::keycodes::Ctrl_G;
use crate::message::state::{msg_col, msg_scroll};
use crate::message::{e_invarg, e_invarg2, e_invcmd, e_invrange, e_screenmode};
use crate::message::{emsg, msg_display, msg_ext_set_kind, msg_putchar, msg_start};
use crate::normal::do_check_scrollbind;
use crate::option::get_findfunc;
use crate::option::vars::p_pvh;
use crate::os::cshim::gettext;
use crate::os::input::os_breakcheck;
use crate::popupmenu::pum_make_popup;
use crate::tag::state::{g_do_tagpreview, postponed_split_flags, postponed_split_tab};
use crate::types::{CmdLine, CmdModFlags, ExArg, IOSIZE, NUL};
use crate::ui::state::{Columns, Rows};
use crate::undo::buf_is_changed;
use crate::window::{
    WSP_VERT, do_window, enter, goto_tab_number, new_tabpage, setheight_win, setwidth_win, split,
    tabpage_move, valid_win,
};
use crate::winlayer::{Buf, TabPage, Win, tabs, windows, windows_in_tab};

// ---------------------------------------------------------------------------
// The command's own arguments.

/// Is this the command `cmd`?
fn is(excmd: &ExArg, cmd: CmdIdx) -> bool {
    excmd.cmdidx == cmd
}

/// The count a range in front of the command asked for, or `def`.
fn count(excmd: &ExArg, def: c_int) -> c_int {
    if excmd.addr_count > 0 {
        excmd.line2 as c_int
    } else {
        def
    }
}

// ---------------------------------------------------------------------------
// The neighbours that are still transpiled, one wrapper each.

/// `_(msg)` as an owned Ex-command error message.
fn err_msg(msg: &'static CStr) -> Option<CString> {
    Some(gettext(msg).to_owned())
}

/// `emsg(_(msg))`.
fn err(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// `do_exedit()`: run the `:edit` half of a command that opened a window.
fn edit(excmd: &mut ExArg, old_curwin: Option<Win>) {
    // SAFETY: a live window or null.
    do_exedit(excmd, old_curwin.map(Win::id));
}

/// `get_tabpage_arg()`: the tab page number the command names, setting
/// `errmsg` when the argument is not one.
fn tabpage_arg(excmd: &mut ExArg) -> c_int {
    get_tabpage_arg(excmd)
}

// ---------------------------------------------------------------------------
// Window and tab page numbers, for the `:` address types.

/// The number of `win` in the current tab page, counting from one.
///
/// A window not in the list answers the number of windows, which is what
/// `winnr()` reports for one that has just been closed.
pub(crate) fn current_win_nr(win: Option<Win>) -> c_int {
    let mut nr = 0;
    for wp in windows() {
        nr += 1;
        if Some(wp) == win {
            break;
        }
    }
    nr
}

/// The same for tab pages. `current_tab_nr(NULL)` is the count.
pub(crate) fn current_tab_nr(tab: Option<TabPage>) -> c_int {
    let mut nr = 0;
    for tp in tabs() {
        nr += 1;
        if tp.raw() == tab.map_or(ptr::null_mut(), TabPage::raw) {
            break;
        }
    }
    nr
}

// ---------------------------------------------------------------------------
// Opening windows and tab pages.

/// The handler every command modifier carries in the table, for the case
/// where it was typed as a command in its own right.
pub(crate) fn ex_wrongmodifier(excmd: &mut ExArg) {
    excmd.errmsg = err_msg(e_invcmd);
}

/// `:split`, `:vsplit`, `:new`, `:sfind`, `:tabedit`, `:tabnew`,
/// `:tabfind` — open a window or a tab page, then edit into it.
pub fn ex_splitview(excmd: &mut ExArg) {
    splitview(excmd);
}

fn splitview(excmd: &mut ExArg) {
    // Taken as a handle here, while the window is certainly live: `split`
    // below fires `WinEnter`, and 'findfunc' runs before it, either of which
    // can close this window. Holding the handle across that is safe -- the id
    // is what `open_tabpage` and `do_exedit` vet -- but *building* one from
    // the address afterwards would read a freed window.
    let old_curwin = Win::current();
    let use_tab =
        is(excmd, CmdIdx::tabedit) || is(excmd, CmdIdx::tabfind) || is(excmd, CmdIdx::tabnew);

    // Splitting a quickfix window gives a plain window, not a second
    // quickfix one — unless `:tab` asked for a tab page.
    if buf_is_quickfix(Some(Buf::current())) && cmdmod.with(|m| m.cmod_tab) == 0 {
        if is(excmd, CmdIdx::split) {
            excmd.cmdidx = CmdIdx::new;
        }
        if is(excmd, CmdIdx::vsplit) {
            excmd.cmdidx = CmdIdx::vnew;
        }
    }

    // `:sfind`/`:tabfind` resolve the name through 'findfunc' or 'path'
    // before anything is opened.
    if is(excmd, CmdIdx::sfind) || is(excmd, CmdIdx::tabfind) {
        let Some(fname) = find_file(excmd.line.arg(), count(excmd, 1)) else {
            return;
        };
        excmd.set_arg_text(fname.as_bytes());
    }

    if use_tab {
        open_tabpage(excmd, old_curwin);
    } else if split(
        count(excmd, 0),
        vertical_flag(excmd.line.byte_at(excmd.line.cmd)),
    )
    .is_ok()
    {
        // A split that will show a *different* file must not stay bound to
        // the one it came from.
        if excmd.line.byte_at(excmd.line.arg) != 0 {
            reset_binding(Win::current());
        } else {
            do_check_scrollbind(false);
        }
        edit(excmd, Some(old_curwin));
    }
}

/// `WSP_VERT` when the command was spelled with a leading `v`.
fn vertical_flag(first: u8) -> c_int {
    if first == b'v' { WSP_VERT as c_int } else { 0 }
}

/// `RESET_BINDING()`: a window that has just been given another file is not
/// bound to the one it was split from.
fn reset_binding(mut win: Win) {
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
}

/// `:sfind`/`:tabfind`: resolve the argument to a file name.
fn find_file(arg: &[u8], count: c_int) -> Option<ThinCString> {
    if !get_findfunc().is_empty() {
        return findfunc_find_file(arg, count);
    }
    find_nth_file_in_path(arg, FileNameOpts::MESS, 1)
}

/// The `:tabedit`/`:tabfind`/`:tabnew` half of [`splitview`].
///
/// Nothing happens at all when there was no room for a tab page: the file is
/// not edited anywhere.
fn open_tabpage(excmd: &mut ExArg, old_curwin: Win) {
    let after = if cmdmod.with(|m| m.cmod_tab) != 0 {
        cmdmod.with(|m| m.cmod_tab)
    } else if excmd.addr_count == 0 {
        0
    } else {
        excmd.line2 as c_int + 1
    };
    if new_tabpage(after, Some(excmd.line.cstr_from(excmd.line.arg)), true).is_none() {
        return;
    }
    edit(excmd, Some(old_curwin));
    let (ev, buf) = (AutoEvent::TabNewEntered, Buf::current_or_none());
    fire_autocmds(ev, false, buf);

    // The window left behind gets the new buffer as its alternate file.
    if Win::current_raw() != old_curwin.raw()
        && let Some(mut old) = valid_win(old_curwin.id())
        && !old.w_buffer.is_current()
        && !cmdmod_has(CmdModFlags::KEEPALT)
    {
        old.w_alt_fnum = Buf::current().handle as c_int;
    }
}

/// Open a new tab page, as `:tabnew` would.
pub fn tabpage_new() {
    let mut excmd = fresh_exarg();
    excmd.line1 = 0;
    excmd.line2 = 0;
    // `ex_splitview` reads the first byte of `cmd` to tell a vertical split
    // from a horizontal one; the argument is what follows it, which is
    // nothing.
    excmd.line = CmdLine::from_bytes(b"tabn");
    excmd.line.arg = 4;
    excmd.cmdidx = CmdIdx::tabnew;
    splitview(&mut excmd);
}

// ---------------------------------------------------------------------------
// Moving between and listing tab pages.

/// `:tabnext` and its seven siblings.
///
/// `:tabprevious`/`:tabNext` count *backwards*, which `goto_tab_number`
/// spells as a negative argument; the rest go to an absolute number that
/// `get_tabpage_arg` works out.
pub(crate) fn ex_tabnext(excmd: &mut ExArg) {
    tabnext(excmd);
}

fn tabnext(excmd: &mut ExArg) {
    if is(excmd, CmdIdx::tabfirst) || is(excmd, CmdIdx::tabrewind) {
        goto_tab_number(1);
        return;
    }
    if is(excmd, CmdIdx::tablast) {
        // Larger than any tab count.
        goto_tab_number(9999);
        return;
    }
    if !is(excmd, CmdIdx::tabprevious) && !is(excmd, CmdIdx::tabNext) {
        let tab_number = tabpage_arg(excmd);
        if excmd.errmsg.is_none() {
            goto_tab_number(tab_number);
        }
        return;
    }

    // A count for `:tabprevious` may be an argument or a range, but a
    // *signed* argument is not a count of places to go back — `:tabp -1`
    // is an error, not `:tabp 1`.
    let tab_number;
    if excmd.line.byte_at(excmd.line.arg) != 0 {
        let start = excmd.line.arg;
        let (number, at) = getdigits_at(excmd.line.buffer_mut(), start, false, 0);
        tab_number = number as c_int;
        if at == start
            || matches!(excmd.line.byte_at(start), b'-' | b'+')
            || excmd.line.byte_at(at) != 0
            || tab_number == 0
        {
            excmd.errmsg = Some(ex_errmsg(e_invarg2, excmd.line.cstr_from(excmd.line.arg)));
            return;
        }
    } else if excmd.addr_count == 0 {
        tab_number = 1;
    } else {
        tab_number = excmd.line2 as c_int;
        if tab_number < 1 {
            excmd.errmsg = err_msg(e_invrange);
            return;
        }
    }
    goto_tab_number(-tab_number);
}

/// `:tabmove`.
pub(crate) fn ex_tabmove(excmd: &mut ExArg) {
    tabmove(excmd);
}

fn tabmove(excmd: &mut ExArg) {
    let tab_number = tabpage_arg(excmd);
    if excmd.errmsg.is_none() {
        tabpage_move(tab_number);
    }
}

/// `:tabs` — every tab page, with its windows.
pub(crate) fn ex_tabs(_excmd: &mut ExArg) {
    // SAFETY: writes the message area.
    msg_ext_set_kind(c"list_cmd");
    msg_start();
    msg_scroll.set(1);

    let lastused_win = last_used_tab().and_then(TabPage::current_window);

    for (tabcount, tp) in tabs().enumerate() {
        if got_int.get() {
            break;
        }
        if msg_col.get() > 0 {
            msg_char('\n' as c_int);
        }
        // Upstream assembles the line in `IObuff`; it is cut where that
        // would cut it.
        let nr = tabcount as c_int + 1;
        let line = crate::formatted!(IOSIZE as usize, gettext(c"Tab page %d").to_bytes(), nr);
        msg_display_bytes(&line, HLF_T, false);
        os_breakcheck();
        list_tab_windows(tp, lastused_win);
    }
}

/// The `:tabs` entry for each window of `tabpage`.
fn list_tab_windows(tabpage: TabPage, lastused_win: Option<Win>) {
    for wp in windows_in_tab(tabpage) {
        if got_int.get() {
            break;
        }
        // A hidden or unfocusable floating window is not listed.
        if !wp.w_config.focusable || wp.w_config.hide {
            continue;
        }
        msg_char('\n' as c_int);
        msg_char(if wp.is_current() {
            '>' as c_int
        } else if lastused_win == Some(wp) {
            '#' as c_int
        } else {
            ' ' as c_int
        });
        msg_char(' ' as c_int);
        msg_char(if is_changed(wp.buffer()) {
            '+' as c_int
        } else {
            ' ' as c_int
        });
        msg_char(' ' as c_int);
        msg_display(fill_name(wp.buffer()).as_cstr(), 0, false);
        os_breakcheck();
    }
}

/// `buffer`'s display name: the special name a scratch buffer has, or its
/// file name with the home directory folded back to `~` -- cut, as upstream's
/// `IObuff` cut it.
fn fill_name(buffer: Buf) -> XString {
    match buf_spname_owned(buffer) {
        None => {
            let fname = buffer.name.shown().unwrap_or(c"");
            home_replace_in(Some(buffer), fname, IOSIZE as usize, true)
        }
        Some(special) => {
            let special = special.as_cstr().to_bytes();
            XString::from_bytes(&special[..special.len().min(IOSIZE as usize - 1)])
        }
    }
}

fn msg_char(c: c_int) {
    msg_putchar(c);
}

fn is_changed(buffer: Buf) -> bool {
    // SAFETY: a live buffer.
    buf_is_changed(buffer)
}

// ---------------------------------------------------------------------------
// The screen, and window sizes.

/// `:mode` — a redraw; the Vim spelling that took a terminal mode name is
/// refused.
pub(crate) fn ex_mode(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) == 0 {
        must_redraw.set(UPD_CLEAR);
        // SAFETY: a live command.
        ex_redraw(excmd);
    } else {
        err(e_screenmode);
    }
}

/// `:resize`, and `:vertical resize`.
///
/// A leading `-` or `+` makes the argument relative — `atol` already read
/// the sign, so the current size is simply added. No argument at all means
/// "as large as possible".
pub(crate) fn ex_resize(excmd: &mut ExArg) {
    resize(excmd);
}

fn resize(excmd: &mut ExArg) {
    let mut wp = Win::current();
    if excmd.addr_count > 0 {
        // The count is a window number, clamped to the last window.
        let mut n = excmd.line2 as c_int;
        let mut walk = windows();
        wp = walk.next().expect("firstwin");
        for next in walk {
            n -= 1;
            if n <= 0 {
                break;
            }
            wp = next;
        }
    }

    let relative = c_int::from(excmd.line.byte_at(excmd.line.arg)) == '-' as c_int
        || c_int::from(excmd.line.byte_at(excmd.line.arg)) == '+' as c_int;
    let empty = excmd.line.byte_at(excmd.line.arg) == 0;
    // A non-number reads as zero, as `atol` does.
    let arg = excmd.line.arg;
    let mut n = getdigits_at(excmd.line.buffer_mut(), arg, false, 0).0 as c_int;
    if cmdmod.with(|m| m.cmod_split) & WSP_VERT as c_int != 0 {
        if relative {
            n += wp.w_width;
        } else if n == 0 && empty {
            n = Columns.get();
        }
        setwidth_win(n, wp);
    } else {
        if relative {
            n += wp.w_height;
        } else if n == 0 && empty {
            n = Rows.get() - 1;
        }
        setheight_win(n, wp);
    }
}

/// `:winsize` — two numbers, and nothing else.
pub(crate) fn ex_winsize(excmd: &mut ExArg) {
    winsize(excmd);
}

fn winsize(excmd: &mut ExArg) {
    if !excmd.line.byte_at(excmd.line.arg).is_ascii_digit() {
        let at = msg_bytes(excmd.line.arg());
        semsg!("E475: Invalid argument: {at}");
        return;
    }
    let arg = excmd.line.arg;
    let (w, at) = getdigits_int_at(excmd.line.buffer_mut(), arg, false, 10);
    let second = excmd.line.skip_white(at);
    let (h, at) = getdigits_int_at(excmd.line.buffer_mut(), second, false, 10);
    // Something at `second` means there *was* a second number; the end at
    // `at` means there was nothing after it.
    if excmd.line.byte_at(second) != 0 && excmd.line.byte_at(at) == 0 {
        screen_resize(w, h);
    } else {
        err(c"E465: :winsize requires two number arguments");
    }
}

/// `:wincmd` — one window command, spelled as a command line.
pub(crate) fn ex_wincmd(excmd: &mut ExArg) {
    wincmd(excmd);
}

fn wincmd(excmd: &mut ExArg) {
    // `CTRL-W g` takes a second character.
    let mut xchar = NUL;
    let mut at;
    if c_int::from(excmd.line.byte_at(excmd.line.arg)) == 'g' as c_int
        || c_int::from(excmd.line.byte_at(excmd.line.arg)) == Ctrl_G
    {
        let second = excmd.line.byte_at(excmd.line.arg + 1);
        if second == 0 {
            err(e_invarg);
            return;
        }
        xchar = c_int::from(second);
        at = excmd.line.arg + 2;
    } else {
        at = excmd.line.arg + 1;
    }

    excmd.line.next = excmd.line.check_next(at);
    at = excmd.line.skip_white(at);
    let byte = excmd.line.byte_at(at);
    if byte != 0 && byte != b'"' && excmd.line.next.is_none() {
        err(e_invarg);
    } else if !excmd.skip {
        // A `:vertical`/`:tab` in front applies to the split the window
        // command is about to make.
        postponed_split_flags.set(cmdmod.with(|m| m.cmod_split));
        postponed_split_tab.set(cmdmod.with(|m| m.cmod_tab));
        let (nchar, prenum) = (
            c_int::from(excmd.line.byte_at(excmd.line.arg)),
            count(excmd, 0),
        );
        do_window(nchar, prenum, xchar);
        postponed_split_flags.set(0);
        postponed_split_tab.set(0);
    }
}

// ---------------------------------------------------------------------------
// The commands with no window of their own.

/// The Vim commands that only make sense with a built-in GUI.
pub(crate) fn ex_nogui(excmd: &mut ExArg) {
    excmd.errmsg = err_msg(c"E25: Nvim does not have a built-in GUI");
}

/// `:popup`.
pub(crate) fn ex_popup(excmd: &mut ExArg) {
    let (name, use_mouse_pos) = (excmd.line.cstr_from(excmd.line.arg), excmd.forceit);
    pum_make_popup(name, c_int::from(use_mouse_pos));
}

// ---------------------------------------------------------------------------
// The preview window.

/// `:psearch` — `:isearch` with the result shown in the preview window.
pub(crate) fn ex_psearch(excmd: &mut ExArg) {
    g_do_tagpreview.set(p_pvh() as c_int);
    // SAFETY: the caller's promise -- a live command.
    ex_findpat(excmd);
    g_do_tagpreview.set(0);
}

/// `:pedit`.
pub(crate) fn ex_pedit(excmd: &mut ExArg) {
    let curwin_save = Win::current().id();
    prepare_preview_window();
    edit(excmd, None);
    back_to_current_window(curwin_save);
}

/// `:pbuffer`.
pub(crate) fn ex_pbuffer(excmd: &mut ExArg) {
    let curwin_save = Win::current().id();
    prepare_preview_window();
    do_exbuffer(excmd);
    back_to_current_window(curwin_save);
}

/// Open or reuse the preview window, and make it current.
fn prepare_preview_window() {
    g_do_tagpreview.set(p_pvh() as c_int);
    prepare_tagpreview(true);
}

/// Go back to the window `:pedit` was run from, if it is still there.
///
/// Takes the *identity*: the caller saved it before the command that may since
/// have closed the window.
fn back_to_current_window(curwin_save: WinId) {
    if Win::current_or_none().map(Win::id) != Some(curwin_save)
        && let Some(saved) = valid_win(curwin_save)
    {
        // The preview window is left drawn but not current.
        Win::current().validate_cursor();
        Win::current().redraw_later(UPD_VALID);
        enter(saved, true);
    }
    g_do_tagpreview.set(0);
}
