//! Splitting, resizing, moving between and listing windows and tab pages.
//!
//! Every function here is an Ex command handler reached through the command
//! table, so the file is glue: it reads an `exarg_T`, works out what the
//! command meant, and calls into the window family. [`Ex`] carries the
//! command's own storage and `winlayer`'s [`Win`]/[`TabPage`] carry the
//! editor's, which leaves only the calls into modules that are still
//! transpiled inside an `unsafe` block.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::ascii::ascii_isdigit;
use crate::autocmd::{EVENT_TABNEWENTERED, apply_autocmds};
use crate::buffer::{bt_quickfix, buf_spname};
use crate::charset::{getdigits, getdigits_int, skipwhite};
use crate::drawscreen::{UPD_CLEAR, UPD_VALID, screen_resize};
use crate::ex_cmds::prepare_tagpreview;
use crate::ex_docmd::argopt::get_tabpage_arg;
use crate::ex_docmd::display::ex_redraw;
use crate::ex_docmd::file::{do_exbuffer, do_exedit};
use crate::ex_docmd::onecmd::fresh_exarg;
use crate::ex_docmd::path::findfunc_find_file;
use crate::ex_docmd::scan::check_nextcmd;
use crate::ex_docmd::source::ex_errmsg;
use crate::ex_docmd::tags::ex_findpat;
use crate::ex_docmd::{FNAME_MESS, IOSIZE};
use crate::file_search::{find_file_in_path, vim_findfile_cleanup};
use crate::highlight_group::HLF_T;
use crate::keycodes::Ctrl_G;
use crate::main::{
    Columns, IObuff, Rows, cmdmod, curbuf, curwin, e_invarg, e_invarg2, e_invcmd, e_invrange,
    e_screenmode, g_do_tagpreview, got_int, lastused_tabpage, msg_col, msg_scroll, must_redraw,
    p_pvh, postponed_split_flags, postponed_split_tab,
};
use crate::memory::{xfree, xstrlcpy};
use crate::message::{emsg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_start};
use crate::normal::do_check_scrollbind;
use crate::option::get_findfunc;
use crate::os::cshim::gettext;
use crate::os::env::home_replace;
use crate::os::input::os_breakcheck;
use crate::popupmenu::pum_make_popup;
use crate::strings::vim_snprintf;
use crate::types::{
    CMD_new, CMD_sfind, CMD_split, CMD_tabNext, CMD_tabedit, CMD_tabfind, CMD_tabfirst,
    CMD_tablast, CMD_tabnew, CMD_tabprevious, CMD_tabrewind, CMD_vnew, CMD_vsplit, CMOD_KEEPALT,
    FAIL, NUL, exarg_T, intmax_t, size_t, tabpage_T, uint8_t, win_T,
};
use crate::undo::bufIsChanged;
use crate::window::{
    WSP_VERT, do_window, enter, goto_tab_number, new_tabpage, setheight_win, setwidth_win, split,
    tabpage_move, valid_tab, valid_win,
};
use crate::winlayer::{Buf, TabPage, Win, tabs, windows, windows_in_tab};
use ::libc::{atol, strlen};

// ---------------------------------------------------------------------------
// The command's own arguments.

/// The `exarg_T` an Ex command handler is called with.
///
/// Every handler in this file is reached from the command table through a raw
/// pointer and then reads or writes the arguments a dozen times. Wrapping the
/// pointer once states the promise once: the command outlives the value, which
/// is exactly the contract each `unsafe fn` entry point already carries. The
/// two `Deref` impls hold the whole obligation, so the wrap itself is ordinary
/// code — the shape `winlayer` uses for its own walks.
#[derive(Clone, Copy)]
struct Ex(*mut exarg_T);

impl Deref for Ex {
    type Target = exarg_T;

    fn deref(&self) -> &exarg_T {
        // SAFETY: the wrap's promise -- a live command.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Ex {
    fn deref_mut(&mut self) -> &mut exarg_T {
        // SAFETY: the wrap's promise -- a live command.
        unsafe { &mut *self.0 }
    }
}

impl Ex {
    /// The pointer back, for the neighbours still taking one.
    fn raw(self) -> *mut exarg_T {
        self.0
    }

    /// The command's `cmdidx`, as the C compares it.
    fn is(self, cmd: crate::types::cmdidx_T) -> bool {
        self.cmdidx as c_int == cmd as c_int
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
fn tr(msg: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated message.
    unsafe { gettext(msg) }
}

/// `emsg(_(msg))`.
fn err(msg: *const c_char) {
    // SAFETY: a NUL-terminated message.
    unsafe { emsg(gettext(msg)) };
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
    unsafe { strlen(p) }
}

fn cur_win() -> Win {
    // SAFETY: `curwin` is always a live window.
    unsafe { Win::current() }
}

fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is always a live buffer.
    unsafe { Buf::current() }
}

fn is_quickfix(buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { bt_quickfix(buf.raw()) }
}

/// `do_exedit()`: run the `:edit` half of a command that opened a window.
fn edit(ea: Ex, old_curwin: *mut win_T) {
    // SAFETY: a live command, and a live window or null.
    unsafe { do_exedit(ea.raw(), old_curwin) };
}

/// `get_tabpage_arg()`: the tab page number the command names, setting
/// `errmsg` when the argument is not one.
fn tabpage_arg(ea: Ex) -> c_int {
    // SAFETY: a live command.
    unsafe { get_tabpage_arg(ea.raw()) }
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
pub(crate) fn current_win_nr(win: *const win_T) -> c_int {
    let mut nr = 0;
    for wp in windows() {
        nr += 1;
        if ptr::eq(wp.raw(), win) {
            break;
        }
    }
    nr
}

/// The same for tab pages. `current_tab_nr(NULL)` is the count.
pub(crate) fn current_tab_nr(tab: *mut tabpage_T) -> c_int {
    let mut nr = 0;
    for tp in tabs() {
        nr += 1;
        if tp.raw() == tab {
            break;
        }
    }
    nr
}

// ---------------------------------------------------------------------------
// Opening windows and tab pages.

/// The handler every command modifier carries in the table, for the case
/// where it was typed as a command in its own right.
pub(crate) unsafe fn ex_wrongmodifier(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    let mut ea = Ex(eap);
    ea.errmsg = tr(&raw const e_invcmd as *const c_char);
}

/// `:split`, `:vsplit`, `:new`, `:sfind`, `:tabedit`, `:tabnew`,
/// `:tabfind` — open a window or a tab page, then edit into it.
pub unsafe fn ex_splitview(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    splitview(Ex(eap));
}

fn splitview(mut ea: Ex) {
    let old_curwin = curwin.get();
    let use_tab = ea.is(CMD_tabedit) || ea.is(CMD_tabfind) || ea.is(CMD_tabnew);

    // Splitting a quickfix window gives a plain window, not a second
    // quickfix one — unless `:tab` asked for a tab page.
    if is_quickfix(cur_buf()) && cmdmod.with(|m| m.cmod_tab) == 0 {
        if ea.is(CMD_split) {
            ea.cmdidx = CMD_new;
        }
        if ea.is(CMD_vsplit) {
            ea.cmdidx = CMD_vnew;
        }
    }

    // `:sfind`/`:tabfind` resolve the name through 'findfunc' or 'path'
    // before anything is opened.
    let mut fname = ptr::null_mut();
    if ea.is(CMD_sfind) || ea.is(CMD_tabfind) {
        fname = find_file(ea.arg, ea.count(1));
        if fname.is_null() {
            return;
        }
        ea.arg = fname;
    }

    if use_tab {
        open_tabpage(ea, old_curwin);
    } else if split(ea.count(0), vertical_flag(ea.cmd)) != FAIL {
        // A split that will show a *different* file must not stay bound to
        // the one it came from.
        if byte(ea.arg) != NUL {
            reset_binding(cur_win());
        } else {
            // SAFETY: reads the window list and the current window.
            unsafe { do_check_scrollbind(false) };
        }
        edit(ea, old_curwin);
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
    let (mess, from) = (FNAME_MESS as c_int, cur_buf().b_ffname);
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
fn open_tabpage(ea: Ex, old_curwin: *mut win_T) {
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
    edit(ea, old_curwin);
    let (ev, buf) = (EVENT_TABNEWENTERED, curbuf.get());
    let (no_fname, no_file) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: an event with no file name, over the current buffer.
    unsafe { apply_autocmds(ev, no_fname, no_file, false, buf) };

    // The window left behind gets the new buffer as its alternate file.
    if curwin.get() != old_curwin
        && let Some(mut old) = valid_win(old_curwin)
        && old.w_buffer != curbuf.get()
        && cmdmod.with(|m| m.cmod_flags) & CMOD_KEEPALT as c_int == 0
    {
        old.w_alt_fnum = cur_buf().handle as c_int;
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
    ea.cmdidx = CMD_tabnew;
    splitview(Ex(&raw mut ea));
}

// ---------------------------------------------------------------------------
// Moving between and listing tab pages.

/// `:tabnext` and its seven siblings.
///
/// `:tabprevious`/`:tabNext` count *backwards*, which `goto_tab_number`
/// spells as a negative argument; the rest go to an absolute number that
/// `get_tabpage_arg` works out.
pub(crate) unsafe fn ex_tabnext(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    tabnext(Ex(eap));
}

fn tabnext(mut ea: Ex) {
    if ea.is(CMD_tabfirst) || ea.is(CMD_tabrewind) {
        goto_tab_number(1);
        return;
    }
    if ea.is(CMD_tablast) {
        // Larger than any tab count.
        goto_tab_number(9999);
        return;
    }
    if !ea.is(CMD_tabprevious) && !ea.is(CMD_tabNext) {
        let tab_number = tabpage_arg(ea);
        if ea.errmsg.is_null() {
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
            let (msg, arg) = (&raw const e_invarg2 as *const c_char, ea.arg);
            // SAFETY: a message with one `%s`, and the argument for it.
            ea.errmsg = unsafe { ex_errmsg(msg, arg) };
            return;
        }
    } else if ea.addr_count == 0 {
        tab_number = 1;
    } else {
        tab_number = ea.line2 as c_int;
        if tab_number < 1 {
            ea.errmsg = tr(&raw const e_invrange as *const c_char);
            return;
        }
    }
    goto_tab_number(-tab_number);
}

/// `:tabmove`.
pub(crate) unsafe fn ex_tabmove(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    tabmove(Ex(eap));
}

fn tabmove(ea: Ex) {
    let tab_number = tabpage_arg(ea);
    if ea.errmsg.is_null() {
        tabpage_move(tab_number);
    }
}

/// `:tabs` — every tab page, with its windows.
pub(crate) unsafe fn ex_tabs(_eap: *mut exarg_T) {
    // SAFETY: writes the message area.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    // SAFETY: starts a message.
    unsafe { msg_start() };
    msg_scroll.set(1);

    let lastused_win = valid_tab(lastused_tabpage.get()).map_or(ptr::null_mut(), |tp| tp.tp_curwin);

    for (tabcount, tp) in tabs().enumerate() {
        if got_int.get() {
            break;
        }
        if msg_col.get() > 0 {
            msg_char('\n' as c_int);
        }
        let (fmt, nr) = (tr(c"Tab page %d".as_ptr()), tabcount as c_int + 1);
        IObuff.with_mut(|b| {
            let (out, size) = (b.as_mut_ptr(), IOSIZE as size_t);
            // SAFETY: a message with one `%d`, into the shared buffer.
            unsafe { vim_snprintf(out, size, fmt, nr) };
        });
        msg_iobuff(HLF_T);
        os_breakcheck();
        list_tab_windows(tp, lastused_win);
    }
}

/// The `:tabs` entry for each window of `tp`.
fn list_tab_windows(tp: TabPage, lastused_win: *mut win_T) {
    for wp in windows_in_tab(tp) {
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
        } else if ptr::eq(wp.raw(), lastused_win) {
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
        fill_iobuff_with_name(wp.buffer());
        msg_iobuff(0);
        os_breakcheck();
    }
}

/// `buf`'s display name in `IObuff`: the special name a scratch buffer has,
/// or its file name with the home directory folded back to `~`.
fn fill_iobuff_with_name(buf: Buf) {
    // SAFETY: a live buffer; the answer is a static name or null.
    let special = unsafe { buf_spname(buf.raw()) };
    let (raw, fname) = (buf.raw(), buf.b_fname);
    IObuff.with_mut(|b| {
        let (out, size) = (b.as_mut_ptr(), IOSIZE as size_t);
        if special.is_null() {
            // SAFETY: a live buffer and its own file name, into the buffer.
            unsafe { home_replace(raw, fname, out, size, true) };
        } else {
            // SAFETY: a NUL-terminated name, into the buffer.
            unsafe { xstrlcpy(out, special, size) };
        }
    });
}

fn msg_char(c: c_int) {
    // SAFETY: writes one character to the message area.
    unsafe { msg_putchar(c) };
}

/// Print what [`fill_iobuff_with_name`] and friends left in `IObuff`.
///
/// The buffer is handed on as a pointer rather than as a borrow: `msg_outtrans`
/// re-enters the message machinery, which reads `IObuff` again.
fn msg_iobuff(hl_id: c_int) {
    // SAFETY: `IObuff` is NUL-terminated by whatever filled it.
    unsafe { msg_outtrans(IObuff.ptr() as *mut c_char, hl_id, false) };
}

fn is_changed(buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { bufIsChanged(buf.raw()) }
}

// ---------------------------------------------------------------------------
// The screen, and window sizes.

/// `:mode` — a redraw; the Vim spelling that took a terminal mode name is
/// refused.
pub(crate) unsafe fn ex_mode(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    let ea = Ex(eap);
    if byte(ea.arg) == NUL {
        must_redraw.set(UPD_CLEAR);
        // SAFETY: a live command.
        unsafe { ex_redraw(ea.raw()) };
    } else {
        err(&raw const e_screenmode as *const c_char);
    }
}

/// `:resize`, and `:vertical resize`.
///
/// A leading `-` or `+` makes the argument relative — `atol` already read
/// the sign, so the current size is simply added. No argument at all means
/// "as large as possible".
pub(crate) unsafe fn ex_resize(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    resize(Ex(eap));
}

fn resize(ea: Ex) {
    let mut wp = cur_win();
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
pub(crate) unsafe fn ex_winsize(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    winsize(Ex(eap));
}

fn winsize(ea: Ex) {
    let mut arg = ea.arg;
    if !ascii_isdigit(byte(arg)) {
        let (msg, at) = (&raw const e_invarg2 as *const c_char, arg);
        // SAFETY: a message with one `%s`, and the argument for it.
        unsafe { semsg_c!(gettext(msg), at) };
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

/// `getdigits_int()`: the number `pp` is on, leaving `pp` after it.
fn digits(pp: *mut *mut c_char) -> c_int {
    // SAFETY: a slot holding a pointer into a NUL-terminated string.
    unsafe { getdigits_int(pp, false, 10) }
}

/// `:wincmd` — one window command, spelled as a command line.
pub(crate) unsafe fn ex_wincmd(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    wincmd(Ex(eap));
}

fn wincmd(mut ea: Ex) {
    // `CTRL-W g` takes a second character.
    let mut xchar = NUL;
    let mut p;
    if byte(ea.arg) == 'g' as c_int || byte(ea.arg) == Ctrl_G {
        let second = ea.arg.wrapping_add(1);
        if byte(second) == NUL {
            err(&raw const e_invarg as *const c_char);
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
        err(&raw const e_invarg as *const c_char);
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
pub(crate) unsafe fn ex_nogui(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    let mut ea = Ex(eap);
    ea.errmsg = tr(c"E25: Nvim does not have a built-in GUI".as_ptr());
}

/// `:popup`.
pub(crate) unsafe fn ex_popup(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    let ea = Ex(eap);
    let (name, use_mouse_pos) = (ea.arg, ea.forceit);
    // SAFETY: a NUL-terminated menu path.
    unsafe { pum_make_popup(name, use_mouse_pos) };
}

// ---------------------------------------------------------------------------
// The preview window.

/// `:psearch` — `:isearch` with the result shown in the preview window.
pub(crate) unsafe fn ex_psearch(eap: *mut exarg_T) {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    // SAFETY: the caller's promise -- a live command.
    unsafe { ex_findpat(eap) };
    g_do_tagpreview.set(0);
}

/// `:pedit`.
pub(crate) unsafe fn ex_pedit(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live command.
    let ea = Ex(eap);
    let curwin_save = curwin.get();
    prepare_preview_window();
    edit(ea, ptr::null_mut());
    back_to_current_window(curwin_save);
}

/// `:pbuffer`.
pub(crate) unsafe fn ex_pbuffer(eap: *mut exarg_T) {
    let curwin_save = curwin.get();
    prepare_preview_window();
    // SAFETY: the caller's promise -- a live command.
    unsafe { do_exbuffer(eap) };
    back_to_current_window(curwin_save);
}

/// Open or reuse the preview window, and make it current.
fn prepare_preview_window() {
    g_do_tagpreview.set(p_pvh.get() as c_int);
    // SAFETY: opens a window over the window list.
    unsafe { prepare_tagpreview(true) };
}

/// Go back to the window `:pedit` was run from, if it is still there.
fn back_to_current_window(curwin_save: *mut win_T) {
    if curwin.get() != curwin_save
        && let Some(saved) = valid_win(curwin_save)
    {
        // The preview window is left drawn but not current.
        cur_win().validate_cursor();
        cur_win().redraw_later(UPD_VALID);
        enter(saved, true);
    }
    g_do_tagpreview.set(0);
}
