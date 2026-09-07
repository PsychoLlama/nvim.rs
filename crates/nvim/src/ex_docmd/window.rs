//! Splitting, resizing, moving between and listing windows and tab pages.
//!
//! Every function here is an Ex command handler reached through the command
//! table, so the file is glue: it reads an `ExArg`, works out what the
//! command meant, and calls into the window family. [`Ex`] carries the
//! command's own storage and `winlayer`'s [`Win`]/[`TabPage`] carry the
//! editor's, which leaves only the calls into modules that are still
//! transpiled inside an `unsafe` block.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::semsg;
use crate::types::AutoEvent;
use crate::types::CmdIdx;
use crate::winlayer::WinId;
use crate::winlayer::last_used_tab;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::ptr;
use std::ffi::CString;

use crate::ascii::ascii_isdigit;
use crate::autocmd::apply_autocmds;
use crate::buffer::{buf_is_quickfix, buf_spname};
use crate::charset::{getdigits, getdigits_int, skipwhite};
use crate::drawscreen::state::must_redraw;
use crate::drawscreen::{UPD_CLEAR, UPD_VALID, screen_resize};
use crate::ex_cmds::prepare_tagpreview;
use crate::ex_docmd::argopt::get_tabpage_arg;
use crate::ex_docmd::cmdmod_has;
use crate::ex_docmd::display::ex_redraw;
use crate::ex_docmd::file::{do_exbuffer, do_exedit};
use crate::ex_docmd::onecmd::fresh_exarg;
use crate::ex_docmd::path::findfunc_find_file;
use crate::ex_docmd::scan::check_nextcmd;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::tags::ex_findpat;
use crate::file_search::{FileNameOpts, find_file_in_path, vim_findfile_cleanup};
use crate::getchar::state::got_int;
use crate::highlight_group::HLF_T;
use crate::keycodes::Ctrl_G;
use crate::memory::{xfree, xstrlcpy};
use crate::message::state::{msg_col, msg_scroll};
use crate::message::{e_invarg, e_invarg2, e_invcmd, e_invrange, e_screenmode};
use crate::message::{emsg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_start};
use crate::message_fmt::c_str;
use crate::normal::do_check_scrollbind;
use crate::option::get_findfunc;
use crate::option::vars::p_pvh;
use crate::os::cshim::gettext_ptr;
use crate::os::env::home_replace;
use crate::os::input::os_breakcheck;
use crate::popupmenu::pum_make_popup;
use crate::strings::vim_snprintf;
use crate::tag::state::{g_do_tagpreview, postponed_split_flags, postponed_split_tab};
use crate::types::{CmdModFlags, ExArg, IOSIZE, NUL, intmax_t, size_t, uint8_t};
use crate::ui::state::{Columns, Rows};
use crate::undo::buf_is_changed;
use crate::window::{
    WSP_VERT, do_window, enter, goto_tab_number, new_tabpage, setheight_win, setwidth_win, split,
    tabpage_move, valid_win,
};
use crate::winlayer::{Buf, Ea, TabPage, Win, tabs, windows, windows_in_tab};
use ::libc::atol;

// ---------------------------------------------------------------------------
// The command's own arguments.

/// The `ExArg` an Ex command handler is called with.
///
/// Every handler in this file is reached from the command table through a raw
/// pointer and then reads or writes the arguments a dozen times. Wrapping the
/// pointer once states the promise once: the command outlives the value, which
/// is exactly the contract each `unsafe fn` entry point already carries. The
/// two `Deref` impls hold the whole obligation, so the wrap itself is ordinary
/// code — the shape `winlayer` uses for its own walks.
#[derive(Clone, Copy)]
struct Ex(*mut ExArg);

impl Deref for Ex {
    type Target = ExArg;

    fn deref(&self) -> &ExArg {
        // SAFETY: the wrap's promise -- a live command.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Ex {
    fn deref_mut(&mut self) -> &mut ExArg {
        // SAFETY: the wrap's promise -- a live command.
        unsafe { &mut *self.0 }
    }
}

impl Ex {
    /// The pointer back, for the neighbours still taking one.
    fn raw(self) -> *mut ExArg {
        self.0
    }

    /// Is this the command `cmd`?
    fn is(self, cmd: CmdIdx) -> bool {
        self.cmdidx == cmd
    }

    /// The count a range in front of the command asked for, or `def`.
    fn count(self, def: c_int) -> c_int {
        if self.addr_count > 0 {
            self.line2 as c_int
        } else {
            def
        }
    }
}

// ---------------------------------------------------------------------------
// The neighbours that are still transpiled, one wrapper each.

/// `_()`: the translated message.
fn tr(msg: *const c_char) -> &'static CStr {
    // SAFETY: a NUL-terminated message, and `gettext` answers one too.
    unsafe { CStr::from_ptr(gettext_ptr(msg).as_ptr()) }
}

/// `_(msg)` as an owned Ex-command error message.
fn err_msg(msg: *const c_char) -> Option<CString> {
    Some(tr(msg).to_owned())
}

/// `emsg(_(msg))`.
fn err(msg: *const c_char) {
    // SAFETY: a NUL-terminated message.
    unsafe { emsg(gettext_ptr(msg)) };
}

fn free<T>(p: *mut T) {
    // SAFETY: `xmalloc`ed, or null.
    unsafe { xfree(p as *mut c_void) };
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string.
    unsafe { *p as c_int }
}

fn len(p: *const c_char) -> size_t {
    // SAFETY: a NUL-terminated string.
    unsafe { cstr::bytes_at(p) }.len()
}

/// `do_exedit()`: run the `:edit` half of a command that opened a window.
fn edit(ea: Ex, old_curwin: Option<Win>) {
    // SAFETY: a live command, and a live window or null.
    unsafe { do_exedit(ea.raw(), old_curwin.map(Win::id)) };
}

/// `get_tabpage_arg()`: the tab page number the command names, setting
/// `errmsg` when the argument is not one.
fn tabpage_arg(ea: Ex) -> c_int {
    // SAFETY: a live command.
    get_tabpage_arg(unsafe { Ea::new(ea.raw()) })
}

fn skip_white(p: *mut c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { skipwhite(p) }
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
pub(crate) unsafe fn ex_wrongmodifier(args: *mut ExArg) {
    let mut ea = Ex(args);
    ea.errmsg = err_msg(e_invcmd.as_ptr());
}

/// `:split`, `:vsplit`, `:new`, `:sfind`, `:tabedit`, `:tabnew`,
/// `:tabfind` — open a window or a tab page, then edit into it.
pub unsafe fn ex_splitview(args: *mut ExArg) {
    splitview(Ex(args));
}

fn splitview(mut ea: Ex) {
    let old_curwin = Win::current_raw();
    let use_tab = ea.is(CmdIdx::tabedit) || ea.is(CmdIdx::tabfind) || ea.is(CmdIdx::tabnew);

    // Splitting a quickfix window gives a plain window, not a second
    // quickfix one — unless `:tab` asked for a tab page.
    if buf_is_quickfix(Some(Buf::current())) && cmdmod.with(|m| m.cmod_tab) == 0 {
        if ea.is(CmdIdx::split) {
            ea.cmdidx = CmdIdx::new;
        }
        if ea.is(CmdIdx::vsplit) {
            ea.cmdidx = CmdIdx::vnew;
        }
    }

    // `:sfind`/`:tabfind` resolve the name through 'findfunc' or 'path'
    // before anything is opened.
    let mut fname = ptr::null_mut();
    if ea.is(CmdIdx::sfind) || ea.is(CmdIdx::tabfind) {
        fname = find_file(ea.arg, ea.count(1));
        if fname.is_null() {
            return;
        }
        ea.arg = fname;
    }

    if use_tab {
        open_tabpage(ea, unsafe { Win::new(old_curwin) });
    } else if split(ea.count(0), vertical_flag(ea.cmd)).is_ok() {
        // A split that will show a *different* file must not stay bound to
        // the one it came from.
        if byte(ea.arg) != NUL {
            reset_binding(Win::current());
        } else {
            // SAFETY: reads the window list and the current window.
            unsafe { do_check_scrollbind(false) };
        }
        edit(ea, unsafe { Win::from_raw(old_curwin) });
    }
    free(fname);
}

/// `WSP_VERT` when the command was spelled with a leading `v`.
fn vertical_flag(cmd: *const c_char) -> c_int {
    if byte(cmd) == 'v' as c_int {
        WSP_VERT as c_int
    } else {
        0
    }
}

/// `RESET_BINDING()`: a window that has just been given another file is not
/// bound to the one it was split from.
fn reset_binding(mut win: Win) {
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
}

/// `:sfind`/`:tabfind`: resolve the argument to a file name, or null.
fn find_file(arg: *mut c_char, count: c_int) -> *mut c_char {
    let n = len(arg);
    if byte(get_findfunc()) != NUL {
        // SAFETY: a NUL-terminated argument.
        return unsafe { findfunc_find_file(arg, n, count) };
    }
    let mut file_to_find: *mut c_char = ptr::null_mut();
    let mut search_ctx: *mut c_char = ptr::null_mut();
    let (ff, sc) = (&raw mut file_to_find, &raw mut search_ctx);
    let (mess, from) = (FileNameOpts::MESS, Buf::current().b_ffname);
    // SAFETY: a NUL-terminated argument, and the search's own two slots.
    let found = unsafe { find_file_in_path(arg, n, mess, true, from, ff, sc) };
    free(file_to_find);
    // SAFETY: the context `find_file_in_path` filled in, or null.
    unsafe { vim_findfile_cleanup(search_ctx as *mut c_void) };
    found
}

/// The `:tabedit`/`:tabfind`/`:tabnew` half of [`splitview`].
///
/// Nothing happens at all when there was no room for a tab page: the file is
/// not edited anywhere.
fn open_tabpage(ea: Ex, old_curwin: Win) {
    let after = if cmdmod.with(|m| m.cmod_tab) != 0 {
        cmdmod.with(|m| m.cmod_tab)
    } else if ea.addr_count == 0 {
        0
    } else {
        ea.line2 as c_int + 1
    };
    if new_tabpage(after, ea.arg, true).is_none() {
        return;
    }
    edit(ea, Some(old_curwin));
    let (ev, buf) = (AutoEvent::TabNewEntered, Buf::current_raw());
    let (no_fname, no_file) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: an event with no file name, over the current buffer.
    unsafe { apply_autocmds(ev, no_fname, no_file, false, Buf::from_raw(buf)) };

    // The window left behind gets the new buffer as its alternate file.
    if Win::current_raw() != old_curwin.raw()
        && let Some(mut old) = valid_win(old_curwin.id())
        && old.w_buffer != Buf::current_raw()
        && !cmdmod_has(CmdModFlags::KEEPALT)
    {
        old.w_alt_fnum = Buf::current().handle as c_int;
    }
}

/// Open a new tab page, as `:tabnew` would.
pub fn tabpage_new() {
    let mut ea = fresh_exarg();
    ea.line1 = 0;
    ea.line2 = 0;
    ea.arg = c"".as_ptr() as *mut c_char;
    // `ex_splitview` reads the first byte of `cmd` to tell a vertical split
    // from a horizontal one.
    ea.cmd = c"tabn".as_ptr() as *mut c_char;
    ea.cmdidx = CmdIdx::tabnew;
    splitview(Ex(&raw mut ea));
}

// ---------------------------------------------------------------------------
// Moving between and listing tab pages.

/// `:tabnext` and its seven siblings.
///
/// `:tabprevious`/`:tabNext` count *backwards*, which `goto_tab_number`
/// spells as a negative argument; the rest go to an absolute number that
/// `get_tabpage_arg` works out.
pub(crate) unsafe fn ex_tabnext(args: *mut ExArg) {
    tabnext(Ex(args));
}

fn tabnext(mut ea: Ex) {
    if ea.is(CmdIdx::tabfirst) || ea.is(CmdIdx::tabrewind) {
        goto_tab_number(1);
        return;
    }
    if ea.is(CmdIdx::tablast) {
        // Larger than any tab count.
        goto_tab_number(9999);
        return;
    }
    if !ea.is(CmdIdx::tabprevious) && !ea.is(CmdIdx::tabNext) {
        let tab_number = tabpage_arg(ea);
        if ea.errmsg.is_none() {
            goto_tab_number(tab_number);
        }
        return;
    }

    // A count for `:tabprevious` may be an argument or a range, but a
    // *signed* argument is not a count of places to go back — `:tabp -1`
    // is an error, not `:tabp 1`.
    let tab_number;
    if !ea.arg.is_null() && byte(ea.arg) != NUL {
        let mut p = ea.arg;
        let p_save = p;
        // SAFETY: a NUL-terminated argument; `p` is left on the first byte
        // the number did not use.
        tab_number = unsafe { getdigits(&raw mut p, false, 0 as intmax_t) } as c_int;
        if ptr::eq(p, p_save)
            || byte(p_save) == '-' as c_int
            || byte(p_save) == '+' as c_int
            || byte(p) != NUL
            || tab_number == 0
        {
            let (msg, arg) = (e_invarg2.as_ptr(), ea.arg);
            // SAFETY: a message with one `%s`, and the argument for it.
            ea.errmsg = Some(unsafe { ex_errmsg(msg, arg) });
            return;
        }
    } else if ea.addr_count == 0 {
        tab_number = 1;
    } else {
        tab_number = ea.line2 as c_int;
        if tab_number < 1 {
            ea.errmsg = err_msg(e_invrange.as_ptr());
            return;
        }
    }
    goto_tab_number(-tab_number);
}

/// `:tabmove`.
pub(crate) unsafe fn ex_tabmove(args: *mut ExArg) {
    tabmove(Ex(args));
}

fn tabmove(ea: Ex) {
    let tab_number = tabpage_arg(ea);
    if ea.errmsg.is_none() {
        tabpage_move(tab_number);
    }
}

/// `:tabs` — every tab page, with its windows.
pub(crate) unsafe fn ex_tabs(_args: *mut ExArg) {
    // SAFETY: writes the message area.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    // SAFETY: starts a message.
    unsafe { msg_start() };
    msg_scroll.set(1);

    let lastused_win = last_used_tab().and_then(TabPage::current_window);
    // The listing's scratch line. Upstream assembles it in `IObuff`, which
    // `msg_outtrans` reads again as it re-enters the message machinery.
    let mut line = [0 as c_char; IOSIZE as usize];

    for (tabcount, tp) in tabs().enumerate() {
        if got_int.get() {
            break;
        }
        if msg_col.get() > 0 {
            msg_char('\n' as c_int);
        }
        let (fmt, nr) = (tr(c"Tab page %d".as_ptr()).as_ptr(), tabcount as c_int + 1);
        // SAFETY: a message with one `%d`, into this frame's own buffer.
        unsafe { vim_snprintf(line.as_mut_ptr(), IOSIZE as size_t, fmt, nr) };
        msg_line(&line, HLF_T);
        os_breakcheck();
        list_tab_windows(tp, lastused_win, &mut line);
    }
}

/// The `:tabs` entry for each window of `tabpage`.
fn list_tab_windows(
    tabpage: TabPage,
    lastused_win: Option<Win>,
    line: &mut [c_char; IOSIZE as usize],
) {
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
        fill_name(wp.buffer(), line);
        msg_line(line, 0);
        os_breakcheck();
    }
}

/// `buffer`'s display name in `out`: the special name a scratch buffer has, or
/// its file name with the home directory folded back to `~`.
fn fill_name(buffer: Buf, out: &mut [c_char; IOSIZE as usize]) {
    let special = buf_spname(buffer);
    let (raw, fname) = (buffer.raw(), buffer.b_fname);
    let (out, size) = (out.as_mut_ptr(), IOSIZE as size_t);
    if special.is_null() {
        // SAFETY: a live buffer and its own file name, into the buffer.
        unsafe { home_replace(Buf::from_raw(raw), fname, out, size, true) };
    } else {
        // SAFETY: a NUL-terminated name, into the buffer.
        unsafe { xstrlcpy(out, special, size) };
    }
}

fn msg_char(c: c_int) {
    // SAFETY: writes one character to the message area.
    unsafe { msg_putchar(c) };
}

/// Print what [`fill_name`] and friends left in `line`.
fn msg_line(line: &[c_char; IOSIZE as usize], hl_id: c_int) {
    // SAFETY: NUL-terminated by whatever filled it, and `msg_outtrans` only
    // reads it — the re-entry into the message machinery no longer reaches
    // the same buffer, because this one belongs to `ex_tabs`.
    unsafe { msg_outtrans(line.as_ptr().cast_mut(), hl_id, false) };
}

fn is_changed(buffer: Buf) -> bool {
    // SAFETY: a live buffer.
    buf_is_changed(buffer)
}

// ---------------------------------------------------------------------------
// The screen, and window sizes.

/// `:mode` — a redraw; the Vim spelling that took a terminal mode name is
/// refused.
pub(crate) unsafe fn ex_mode(args: *mut ExArg) {
    let ea = Ex(args);
    if byte(ea.arg) == NUL {
        must_redraw.set(UPD_CLEAR);
        // SAFETY: a live command.
        unsafe { ex_redraw(ea.raw()) };
    } else {
        err(e_screenmode.as_ptr());
    }
}

/// `:resize`, and `:vertical resize`.
///
/// A leading `-` or `+` makes the argument relative — `atol` already read
/// the sign, so the current size is simply added. No argument at all means
/// "as large as possible".
pub(crate) unsafe fn ex_resize(args: *mut ExArg) {
    resize(Ex(args));
}

fn resize(ea: Ex) {
    let mut wp = Win::current();
    if ea.addr_count > 0 {
        // The count is a window number, clamped to the last window.
        let mut n = ea.line2 as c_int;
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

    let relative = byte(ea.arg) == '-' as c_int || byte(ea.arg) == '+' as c_int;
    let empty = byte(ea.arg) == NUL;
    // SAFETY: a NUL-terminated argument; a non-number reads as zero.
    let mut n = unsafe { atol(ea.arg) } as c_int;
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
pub(crate) unsafe fn ex_winsize(args: *mut ExArg) {
    winsize(Ex(args));
}

fn winsize(ea: Ex) {
    let mut arg = ea.arg;
    if !ascii_isdigit(byte(arg)) {
        // SAFETY: the command's NUL-terminated argument.
        let at = unsafe { c_str(arg) };
        semsg!("E475: Invalid argument: {at}");
        return;
    }
    let w = digits(&raw mut arg);
    arg = skip_white(arg);
    let second = arg;
    let h = digits(&raw mut arg);
    // `second` still pointing at something means there *was* a second
    // number; `arg` at the end means there was nothing after it.
    if byte(second) != NUL && byte(arg) == NUL {
        // SAFETY: resizes the screen over the editor's own state.
        unsafe { screen_resize(w, h) };
    } else {
        err(c"E465: :winsize requires two number arguments".as_ptr());
    }
}

/// `getdigits_int()`: the number `cursor` is on, leaving `cursor` after it.
fn digits(cursor: *mut *mut c_char) -> c_int {
    // SAFETY: a slot holding a pointer into a NUL-terminated string.
    unsafe { getdigits_int(cursor, false, 10) }
}

/// `:wincmd` — one window command, spelled as a command line.
pub(crate) unsafe fn ex_wincmd(args: *mut ExArg) {
    wincmd(Ex(args));
}

fn wincmd(mut ea: Ex) {
    // `CTRL-W g` takes a second character.
    let mut xchar = NUL;
    let mut p;
    if byte(ea.arg) == 'g' as c_int || byte(ea.arg) == Ctrl_G {
        let second = ea.arg.wrapping_add(1);
        if byte(second) == NUL {
            err(e_invarg.as_ptr());
            return;
        }
        xchar = byte(second) as uint8_t as c_int;
        p = ea.arg.wrapping_add(2);
    } else {
        p = ea.arg.wrapping_add(1);
    }

    // SAFETY: a NUL-terminated argument; the answer points into it or is null.
    ea.nextcmd = unsafe { check_nextcmd(p) };
    p = skip_white(p);
    if byte(p) != NUL && byte(p) != '"' as c_int && ea.nextcmd.is_null() {
        err(e_invarg.as_ptr());
    } else if ea.skip == 0 {
        // A `:vertical`/`:tab` in front applies to the split the window
        // command is about to make.
        postponed_split_flags.set(cmdmod.with(|m| m.cmod_split));
        postponed_split_tab.set(cmdmod.with(|m| m.cmod_tab));
        let (nchar, prenum) = (byte(ea.arg), ea.count(0));
        do_window(nchar, prenum, xchar);
        postponed_split_flags.set(0);
        postponed_split_tab.set(0);
    }
}

// ---------------------------------------------------------------------------
// The commands with no window of their own.

/// The Vim commands that only make sense with a built-in GUI.
pub(crate) unsafe fn ex_nogui(args: *mut ExArg) {
    let mut ea = Ex(args);
    ea.errmsg = err_msg(c"E25: Nvim does not have a built-in GUI".as_ptr());
}

/// `:popup`.
pub(crate) unsafe fn ex_popup(args: *mut ExArg) {
    let ea = Ex(args);
    let (name, use_mouse_pos) = (ea.arg, ea.forceit);
    // SAFETY: a NUL-terminated menu path.
    unsafe { pum_make_popup(name, use_mouse_pos) };
}

// ---------------------------------------------------------------------------
// The preview window.

/// `:psearch` — `:isearch` with the result shown in the preview window.
pub(crate) unsafe fn ex_psearch(args: *mut ExArg) {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    // SAFETY: the caller's promise -- a live command.
    unsafe { ex_findpat(args) };
    g_do_tagpreview.set(0);
}

/// `:pedit`.
pub(crate) unsafe fn ex_pedit(args: *mut ExArg) {
    let ea = Ex(args);
    let curwin_save = Win::current().id();
    prepare_preview_window();
    edit(ea, None);
    back_to_current_window(curwin_save);
}

/// `:pbuffer`.
pub(crate) unsafe fn ex_pbuffer(args: *mut ExArg) {
    let curwin_save = Win::current().id();
    prepare_preview_window();
    // SAFETY: the caller's promise -- a live command.
    do_exbuffer(unsafe { Ea::new(args) });
    back_to_current_window(curwin_save);
}

/// Open or reuse the preview window, and make it current.
fn prepare_preview_window() {
    g_do_tagpreview.set(p_pvh.get() as c_int);
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
