//! Describing a buffer to the user -- `:ls`, CTRL-G and the title.
//!
//! [`buflist_list`] is `:ls`/`:buffers`, including the flag column and the
//! `'columns'`-aware truncation of each name; [`fileinfo`] is CTRL-G and the
//! message a `:edit` prints; [`get_rel_pos`] is the ruler's "Top"/"Bot"/"NN%"
//! and [`append_arg_number`] the `(2 of 5)` suffix.  [`maketitle`] builds
//! `'title'` and `'icon'` -- the same information again, for the window
//! manager.
//!
//! `NameBuff` and `IObuff` are the editor's two shared scratch buffers, and
//! the rule here is the one the rest of the tree already follows: fill them
//! inside a `with_mut` borrow, then hand them on -- to `message_filtered`, to
//! `msg_outtrans` -- through the cell rather than through a reference that is
//! still outstanding, because those callees re-enter the message machinery.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::api::private::helpers::cstr_as_string;
use crate::charset::{trans_characters, vim_strsize};
use crate::drawscreen::redrawing;
use crate::main::{
    Columns, IObuff, NameBuff, curbuf, firstbuf, got_int, msg_col, msg_scroll, msg_scrolled,
    need_maketitle, need_wait_return, no_lines_msg, p_icon, p_iconstring, p_ru, p_title,
    p_titlelen, p_titlestring, restart_edit, stl_syntax,
};
use crate::mbyte::utf_cp_bounds;
use crate::memory::{xfree, xstrdup, xstrlcpy};
use crate::message::{
    message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_start, msg_trunc,
    set_keep_msg,
};
use crate::r#move::validate_virtcol;
use crate::option::shortmess;
use crate::options::{kOptIconstring, kOptTitlestring};
use crate::os::cshim::{gettext, ngettext};
use crate::os::env::home_replace;
use crate::os::input::line_breakcheck;
use crate::path::path_tail;
use crate::plines::win_get_fill;
use crate::statusline::build_stl_str_hl;
use crate::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::terminal::terminal_running;
use crate::types::ui::kUIMessages;
use crate::types::{OptIndex, OptInt, buf_T, exarg_T, int64_t, linenr_T, size_t, time_t, win_T};
use crate::ui::{ui_call_set_icon, ui_call_set_title, ui_has};
use crate::undo::{bufIsChanged, curbufIsChanged, undo_fmt_time};
use crate::winlayer::{Buf, Win, buffers};
use ::libc::{qsort, strcmp, strcpy, strlen};

use super::list::buf_time_compare;

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// `_()`.
fn tr(msg: &CStr) -> *mut c_char {
    tr_raw(msg.as_ptr())
}

/// `_()` over a pointer, for the messages `main.rs` holds as byte arrays.
fn tr_raw(msg: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated literal or message static.
    unsafe { gettext(msg) }
}

/// `NGETTEXT`: the singular or plural form, by `n`.
fn tr_n(one: &CStr, many: &CStr, n: linenr_T) -> *mut c_char {
    // SAFETY: two NUL-terminated literals.
    unsafe { ngettext(one.as_ptr(), many.as_ptr(), n as ::core::ffi::c_ulong) }
}

fn current_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// Whether `arg` contains the flag character `c` -- `:ls`'s argument is a
/// set of them.
fn has_flag(arg: *const c_char, c: u8) -> bool {
    // SAFETY: `:ls`'s argument, a NUL-terminated string; `vim_strchr` takes
    // a codepoint.
    !unsafe { vim_strchr(arg, c as c_int) }.is_null()
}

fn buf_changed(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { bufIsChanged(buf.raw()) }
}

/// Whether `buf`'s terminal, if it has one, still has a job attached.
fn job_running(buf: Buf) -> bool {
    // SAFETY: a live terminal, the caller having ruled out null.
    !buf.terminal.is_null() && unsafe { terminal_running(buf.terminal) }
}

fn special_name(mut buf: Buf) -> *mut c_char {
    // SAFETY: a live buffer.
    unsafe { buf_spname(buf.raw()) }
}

fn remembered_lnum(mut buf: Buf) -> linenr_T {
    // SAFETY: a live buffer.
    unsafe { buflist_findlnum(buf.raw()) }
}

fn dont_write(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { bt_dontwrite(buf.raw()) }
}

// ---------------------------------------------------------------------------
// :ls / :buffers

/// List the buffers, one line each, as `:ls` and `:files` do.
pub unsafe fn buflist_list(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- the command being executed.
    let arg = unsafe { (*eap).arg };
    // SAFETY: as above.
    let forceit = unsafe { (*eap).forceit };
    // SAFETY: a NUL-terminated literal naming the message kind.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };

    // With "t", the list is shown most-recently-used first.
    let sorted = has_flag(arg, b't').then(sorted_by_last_used);
    let mut walk = Walk::new(sorted.as_deref());

    while let Some(buf) = walk.step() {
        if got_int.get() {
            break;
        }
        if skip(buf, arg, forceit) {
            continue;
        }
        // Fill NameBuff with the name to show, then ask the message filter
        // about it through the cell: `message_filtered` re-enters the
        // regexp engine.
        NameBuff.with_mut(|name| fill_name(buf, name));
        if unsafe { message_filtered(NameBuff.ptr().cast::<c_char>()) } {
            continue;
        }
        show(buf, has_flag(arg, b't'));
    }
}

/// Every buffer, most recently used first.
///
/// `qsort` and `buf_time_compare` stay upstream's: two buffers entered in
/// the same second tie, and a stable Rust sort would order the tie
/// differently.
fn sorted_by_last_used() -> Vec<*mut buf_T> {
    let mut list: Vec<*mut buf_T> = buffers().map(|mut buf| buf.raw()).collect();
    let (base, n, width) = (
        list.as_mut_ptr().cast::<c_void>(),
        list.len(),
        size_of::<*mut buf_T>(),
    );
    // SAFETY: `n` initialised elements of this function's own vector, and a
    // comparison function over two of them.
    unsafe { qsort(base, n, width, Some(buf_time_compare)) };
    list
}

/// The walk `:ls` makes: over the sorted array when there is one, otherwise
/// down the buffer list itself. Upstream reads `b_next` after each line is
/// printed, which is what the second arm does.
struct Walk<'a> {
    sorted: Option<&'a [*mut buf_T]>,
    at: usize,
    next: *mut buf_T,
}

impl<'a> Walk<'a> {
    fn new(sorted: Option<&'a [*mut buf_T]>) -> Self {
        let next = match sorted {
            Some(list) => list.first().copied().unwrap_or(ptr::null_mut()),
            None => firstbuf.get(),
        };
        Walk {
            sorted,
            at: 0,
            next,
        }
    }

    /// Not `Iterator::next`: the walk reads `b_next` only after its caller
    /// has printed the line, which is where upstream's `for` increment
    /// reads it.
    fn step(&mut self) -> Option<Buf> {
        // SAFETY: `firstbuf`, every `b_next` reached from it and every entry
        // of the sorted array is a live buffer or null.
        let buf = (!self.next.is_null()).then(|| unsafe { Buf::new(self.next) })?;
        self.next = match self.sorted {
            Some(list) => {
                self.at += 1;
                list.get(self.at).copied().unwrap_or(ptr::null_mut())
            }
            None => buf.b_next,
        };
        Some(buf)
    }
}

/// Whether the `:ls` flags in `arg` say to skip this buffer.
fn skip(buf: Buf, arg: *const c_char, forceit: c_int) -> bool {
    let is_terminal = !buf.terminal.is_null();
    let job_running = job_running(buf);
    let loaded = !buf.b_ml.ml_mfp.is_null();
    let alt_fnum = current_win().w_alt_fnum;

    buf.b_p_bl == 0 && forceit == 0 && !has_flag(arg, b'u')
        || has_flag(arg, b'u') && buf.b_p_bl != 0
        || has_flag(arg, b'+') && (buf.b_flags & BF_READERR != 0 || !buf_changed(buf))
        || has_flag(arg, b'a') && (!loaded || buf.b_nwindows == 0)
        || has_flag(arg, b'h') && (!loaded || buf.b_nwindows != 0)
        || has_flag(arg, b'R') && (!is_terminal || !job_running)
        || has_flag(arg, b'F') && (!is_terminal || job_running)
        || has_flag(arg, b'-') && buf.b_p_ma != 0
        || has_flag(arg, b'=') && buf.b_p_ro == 0
        || has_flag(arg, b'x') && buf.b_flags & BF_READERR == 0
        || has_flag(arg, b'%') && buf.raw() != curbuf.get()
        || has_flag(arg, b'#') && (buf.raw() == curbuf.get() || alt_fnum != buf.handle)
}

/// Put the name to show for `buf` into `NameBuff`.
fn fill_name(mut buf: Buf, name: &mut [c_char; MAXPATHL as usize]) {
    let special = special_name(buf);
    if !special.is_null() {
        // SAFETY: a NUL-terminated name into `MAXPATHL` writable bytes.
        unsafe { xstrlcpy(name.as_mut_ptr(), special, MAXPATHL as usize) };
        return;
    }
    let (raw, fname, dst) = (buf.raw(), buf.b_fname, name.as_mut_ptr());
    // SAFETY: a live buffer, its name, and `MAXPATHL` writable bytes.
    unsafe { home_replace(raw, fname, dst, MAXPATHL as size_t, true) };
}

/// Print one buffer's line: the number, the flag column, the name padded to
/// column 40, and the line number or the time it was last used.
fn show(mut buf: Buf, by_time: bool) {
    let changed_char = if buf.b_flags & BF_READERR != 0 {
        b'x'
    } else if buf_changed(buf) {
        b'+'
    } else {
        b' '
    };
    let mut ro_char = if buf.b_p_ma == 0 {
        b'-'
    } else if buf.b_p_ro != 0 {
        b'='
    } else {
        b' '
    };
    if !buf.terminal.is_null() {
        ro_char = if job_running(buf) { b'R' } else { b'F' };
    }

    if !ui_has(kUIMessages) || msg_col.get() > 0 {
        // SAFETY: writes one character to the message area.
        unsafe { msg_putchar(b'\n' as c_int) };
    }

    let listed = if buf.b_p_bl != 0 { b' ' } else { b'u' };
    let current = if buf.raw() == curbuf.get() {
        b'%'
    } else if current_win().w_alt_fnum == buf.handle {
        b'#'
    } else {
        b' '
    };
    let state = if buf.b_ml.ml_mfp.is_null() {
        b' '
    } else if buf.b_nwindows == 0 {
        b'h'
    } else {
        b'a'
    };
    let lnum = if buf.raw() == curbuf.get() {
        current_win().w_cursor.lnum
    } else {
        remembered_lnum(buf)
    };
    let last_used = buf.b_last_used;

    IObuff.with_mut(|io| {
        let mut len = format_head(
            io,
            buf.handle,
            [listed, current, state, ro_char, changed_char],
        );
        len = pad_to_column(io, len);
        if by_time && last_used != 0 {
            format_time(io, len, last_used);
        } else {
            format_lnum(io, len, lnum);
        }
    });
    // Hand the assembled line on through the cell: `msg_outtrans` re-enters
    // the message machinery, which has `IObuff` of its own.
    // SAFETY: a NUL-terminated line, just assembled.
    unsafe { msg_outtrans(IObuff.ptr().cast::<c_char>(), 0, false) };
    line_breakcheck();
}

/// `%3d%c%c%c%c%c "%s"`: the number, the five flag columns and the name.
fn format_head(io: &mut [c_char; IOSIZE as usize], handle: c_int, flags: [u8; 5]) -> c_int {
    let (dst, cap) = (io.as_mut_ptr(), (IOSIZE - 20) as size_t);
    let fmt = c"%3d%c%c%c%c%c \"%s\"".as_ptr();
    let [bl, cur, st, ro, ch] = flags.map(c_int::from);
    let name = NameBuff.ptr().cast::<c_char>();
    // SAFETY: `IOSIZE - 20` writable bytes, a format taking a number, five
    // characters and a string, and `NameBuff` holding that string.
    let len = unsafe { vim_snprintf_safelen(dst, cap, fmt, handle, bl, cur, st, ro, ch, name) };
    (len as c_int).min(IOSIZE - 20)
}

/// Put "line 999" in column 40, or after the file name.
fn pad_to_column(io: &mut [c_char; IOSIZE as usize], mut len: c_int) -> c_int {
    // SAFETY: a NUL-terminated line, just assembled.
    // Put "line 999" in column 40.
    let mut i = 40 - unsafe { vim_strsize(io.as_ptr()) };
    loop {
        io[len as usize] = b' ' as c_char;
        len += 1;
        i -= 1;
        if i <= 0 || len >= IOSIZE - 18 {
            break;
        }
    }
    len
}

fn format_time(io: &mut [c_char; IOSIZE as usize], len: c_int, last_used: time_t) {
    let (dst, cap) = (
        io.as_mut_ptr().wrapping_add(len as usize),
        (IOSIZE - len) as size_t,
    );
    // SAFETY: the rest of the line, and the time to render into it.
    unsafe { undo_fmt_time(dst, cap, last_used) };
}

fn format_lnum(io: &mut [c_char; IOSIZE as usize], len: c_int, lnum: linenr_T) {
    let (dst, cap) = (
        io.as_mut_ptr().wrapping_add(len as usize),
        (IOSIZE - len) as size_t,
    );
    let fmt = tr(c"line %ld");
    // SAFETY: the rest of the line, and a format taking one number.
    unsafe { vim_snprintf(dst, cap, fmt, lnum as int64_t) };
}

// ---------------------------------------------------------------------------
// CTRL-G

/// The message CTRL-G and `:file` print: the name, the flags, where the
/// cursor is and how far through the file that is.
pub unsafe fn fileinfo(fullname: c_int, shorthelp: c_int, dont_truncate: bool) {
    let mut out = Msg::new();
    let mut buf = current_buf();

    if fullname > 1 {
        // 2 CTRL-G: include the buffer number.
        out.put_int(c"buf %d: ", buf.handle);
    }
    out.push(b'"');

    let name = special_name(buf);
    if !name.is_null() {
        out.put_str(c"%s", name);
    } else {
        let name = if fullname == 0 && !buf.b_fname.is_null() {
            buf.b_fname
        } else {
            buf.b_ffname
        };
        let help = if shorthelp != 0 {
            buf.raw()
        } else {
            ptr::null_mut()
        };
        out.put_home_replaced(help, name);
    }

    let dontwrite = dont_write(buf);
    let modified = curbuf_changed();
    out.put_flags([
        if modified {
            if shortmess(SHM_MOD as c_int) {
                c" [+]".as_ptr()
            } else {
                tr(c" [Modified]")
            }
        } else {
            c" ".as_ptr()
        },
        if buf.b_flags & BF_NOTEDITED != 0 && !dontwrite {
            tr(c"[Not edited]")
        } else {
            c"".as_ptr()
        },
        if buf.b_flags & BF_NEW != 0 && !dontwrite {
            tr(c"[New]")
        } else {
            c"".as_ptr()
        },
        if buf.b_flags & BF_READERR != 0 {
            tr(c"[Read errors]")
        } else {
            c"".as_ptr()
        },
        if buf.b_p_ro != 0 {
            if shortmess(SHM_RO as c_int) {
                tr(c"[RO]")
            } else {
                tr(c"[readonly]")
            }
        } else {
            c"".as_ptr()
        },
        if modified || buf.b_flags & BF_WRITE_MASK != 0 || buf.b_p_ro != 0 {
            c" ".as_ptr()
        } else {
            c"".as_ptr()
        },
    ]);

    let mut win = current_win();
    let lines = buf.b_ml.ml_line_count;
    let cursor = win.w_cursor.lnum;
    if buf.b_ml.ml_flags & ML_EMPTY != 0 {
        out.put_str(c"%s", tr_raw((&raw const no_lines_msg).cast::<c_char>()));
    } else if p_ru.get() != 0 {
        // The current line and column are already on the screen -- webb
        let fmt = tr_n(c"%ld line --%d%%--", c"%ld lines --%d%%--", lines);
        out.put_lines(fmt, lines, percentage(cursor, lines));
    } else {
        let fmt = tr(c"line %ld of %ld --%d%%-- col ");
        out.put_position(fmt, cursor, lines, percentage(cursor, lines));
        // SAFETY: a live window.
        unsafe { validate_virtcol(win.raw()) };
        out.put_column(win.w_cursor.col as c_int + 1, win.w_virtcol as c_int + 1);
    }

    out.put_arg_number(win);

    if dont_truncate {
        // Temporarily set msg_scroll to keep the message from being
        // truncated; msg_start() first, to get it in the right place.
        // SAFETY: starts a message.
        unsafe { msg_start() };
        let n = msg_scroll.get();
        msg_scroll.set(true_0);
        // SAFETY: a NUL-terminated message.
        unsafe { msg(out.as_ptr(), 0) };
        msg_scroll.set(n);
    } else {
        // SAFETY: a NUL-terminated message; the answer is the kept copy.
        let p = unsafe { msg_trunc(out.as_mut_ptr(), false, 0) };
        // Repeat the message after redrawing when 'restart_edit' is set
        // (otherwise there is a delay before redrawing), or when the screen
        // scrolled but there is no wait-return prompt.
        if restart_edit.get() != 0 || msg_scrolled.get() != 0 && !need_wait_return.get() {
            // SAFETY: the message `msg_trunc` kept.
            unsafe { set_keep_msg(p, 0) };
        }
    }
}

fn curbuf_changed() -> bool {
    // SAFETY: reads the current buffer's undo state.
    unsafe { curbufIsChanged() }
}

fn percentage(part: linenr_T, whole: linenr_T) -> c_int {
    calc_percentage(part as int64_t, whole as int64_t)
}

/// The `IOSIZE` message [`fileinfo`] assembles, and how much of it is used.
struct Msg {
    buf: Vec<u8>,
    len: usize,
}

impl Msg {
    fn new() -> Self {
        Msg {
            buf: vec![0; IOSIZE as usize],
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        self.buf[self.len] = byte;
        self.len += 1;
    }

    /// The unused tail: where to write, and how much room is left.
    fn tail(&mut self) -> (*mut c_char, size_t) {
        let room = IOSIZE as size_t - self.len as size_t;
        (self.buf[self.len..].as_mut_ptr().cast::<c_char>(), room)
    }

    fn as_ptr(&self) -> *const c_char {
        self.buf.as_ptr().cast::<c_char>()
    }

    fn as_mut_ptr(&mut self) -> *mut c_char {
        self.buf.as_mut_ptr().cast::<c_char>()
    }

    fn put_int(&mut self, fmt: &CStr, n: c_int) {
        let (dst, room) = self.tail();
        // SAFETY: the buffer's own tail, and a format taking one number.
        self.len += unsafe { vim_snprintf_safelen(dst, room, fmt.as_ptr(), n) };
    }

    fn put_str(&mut self, fmt: &CStr, s: *const c_char) {
        let (dst, room) = self.tail();
        // SAFETY: the buffer's own tail, and a format taking one string.
        self.len += unsafe { vim_snprintf_safelen(dst, room, fmt.as_ptr(), s) };
    }

    /// The six-part flag string, `"%s%s%s%s%s%s`.
    fn put_flags(&mut self, parts: [*const c_char; 6]) {
        let (dst, room) = self.tail();
        let fmt = c"\"%s%s%s%s%s%s".as_ptr();
        let [a, b, c, d, e, f] = parts;
        // SAFETY: the buffer's own tail, and a format taking six strings.
        self.len += unsafe { vim_snprintf_safelen(dst, room, fmt, a, b, c, d, e, f) };
    }

    fn put_lines(&mut self, fmt: *const c_char, lines: linenr_T, percent: c_int) {
        let (dst, room) = self.tail();
        let lines = lines as int64_t;
        // SAFETY: the buffer's own tail, and a format taking a number and a
        // percentage.
        self.len += unsafe { vim_snprintf_safelen(dst, room, fmt, lines, percent) };
    }

    fn put_position(&mut self, fmt: *const c_char, at: linenr_T, of: linenr_T, percent: c_int) {
        let (dst, room) = self.tail();
        let (at, of) = (at as int64_t, of as int64_t);
        // SAFETY: the buffer's own tail, and a format taking two numbers and
        // a percentage.
        self.len += unsafe { vim_snprintf_safelen(dst, room, fmt, at, of, percent) };
    }

    fn put_column(&mut self, col: c_int, vcol: c_int) {
        let (dst, room) = self.tail();
        // SAFETY: the buffer's own tail.
        self.len += unsafe { col_print(dst, room, col, vcol) } as usize;
    }

    fn put_arg_number(&mut self, mut win: Win) {
        let (dst, room) = self.tail();
        // SAFETY: a live window, and the buffer's own tail.
        unsafe { append_arg_number(win.raw(), dst, room) };
    }

    /// `home_replace` into the tail, followed by the length it wrote --
    /// which upstream measures with `strlen` rather than taking the answer.
    fn put_home_replaced(&mut self, buf: *mut buf_T, name: *const c_char) {
        let (dst, room) = self.tail();
        // SAFETY: a live buffer or null, a NUL-terminated name, and the
        // buffer's own tail.
        unsafe { home_replace(buf, name, dst, room, true) };
        // SAFETY: what `home_replace` just NUL-terminated.
        self.len += unsafe { strlen(dst) };
    }
}

/// The column indicator: `col` alone when the virtual column agrees with it,
/// `col-vcol` when it does not.
pub unsafe fn col_print(buf: *mut c_char, buflen: size_t, col: c_int, vcol: c_int) -> c_int {
    if col == vcol {
        // SAFETY: the caller's buffer, and a format taking one number.
        return unsafe { vim_snprintf_safelen(buf, buflen, c"%d".as_ptr(), col) } as c_int;
    }
    // SAFETY: the caller's buffer, and a format taking two numbers.
    let len = unsafe { vim_snprintf_safelen(buf, buflen, c"%d-%d".as_ptr(), col, vcol) };
    len as c_int
}

// ---------------------------------------------------------------------------
// 'title' and 'icon'

/// Build `'title'` and `'icon'` and, when either changed, tell the UI.
pub unsafe fn maketitle() {
    let mut scratch: [c_char; IOSIZE as usize] = [0; IOSIZE as usize];

    // SAFETY: reads the redraw state.
    if !unsafe { redrawing() } {
        // Postpone updating the title when 'lazyredraw' is set.
        need_maketitle.set(true);
        return;
    }
    need_maketitle.set(false);
    if p_title.get() == 0
        && p_icon.get() == 0
        && lasttitle.get().is_null()
        && lasticon.get().is_null()
    {
        // Nothing to do.
        return;
    }

    let mut title_str: *mut c_char = ptr::null_mut();
    if p_title.get() != 0 {
        let mut maxlen = 0;
        if p_titlelen.get() > 0 as OptInt {
            maxlen = ((p_titlelen.get() * Columns.get() as OptInt / 100) as c_int).max(10);
        }
        if opt_is_set(p_titlestring.get()) {
            if stl_syntax.get() & STL_IN_TITLE != 0 {
                build_stl(&mut scratch, p_titlestring.get(), kOptTitlestring, maxlen);
                title_str = scratch.as_mut_ptr();
            } else {
                title_str = p_titlestring.get();
            }
        } else {
            // Format: "fname + (path) (1 of 2) - Nvim".
            let default = c"%t%( %M%)%( (%{expand('%:p:~:h')})%)%a - Nvim";
            let default = default.as_ptr().cast_mut();
            build_stl(&mut scratch, default, kOptTitlestring, maxlen);
            title_str = scratch.as_mut_ptr();
        }
    }
    let mut mustset = value_change(title_str, &lasttitle);

    let mut icon_str: *mut c_char = ptr::null_mut();
    if p_icon.get() != 0 {
        icon_str = scratch.as_mut_ptr();
        if opt_is_set(p_iconstring.get()) {
            if stl_syntax.get() & STL_IN_ICON != 0 {
                build_stl(&mut scratch, p_iconstring.get(), kOptIconstring, 0);
            } else {
                icon_str = p_iconstring.get();
            }
        } else {
            fill_icon(&mut scratch);
        }
    }
    mustset |= value_change(icon_str, &lasticon);

    if mustset {
        // SAFETY: sends the two titles to the UI.
        unsafe { resettitle() };
    }
}

fn opt_is_set(s: *const c_char) -> bool {
    // SAFETY: a string option, which is never null.
    unsafe { *s != 0 }
}

/// `build_stl_str_hl` for the title and icon, which want the text and
/// nothing else it can report.
fn build_stl(dst: &mut [c_char; IOSIZE as usize], fmt: *mut c_char, opt: OptIndex, maxlen: c_int) {
    let (win, out, cap) = (current_win().raw(), dst.as_mut_ptr(), dst.len());
    let (hl, hllen, click, stc) = (
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
    );
    // SAFETY: a live window, a writable buffer of `cap` bytes, a
    // NUL-terminated format, and four out-parameters this caller declines.
    unsafe { build_stl_str_hl(win, out, cap, fmt, opt, 0, 0, maxlen, hl, hllen, click, stc) };
}

/// The default icon text: the buffer's name, truncated to 100 bytes at a
/// character boundary.
fn fill_icon(dst: &mut [c_char; IOSIZE as usize]) {
    let mut buf = current_buf();
    let mut name = special_name(buf);
    if name.is_null() {
        // SAFETY: a NUL-terminated file name, or null, which `path_tail`
        // does not accept -- `buf_spname` answered null, so `b_fname` is
        // set, and `b_ffname` with it.
        name = unsafe { path_tail(buf.b_ffname) };
    }
    // SAFETY: a NUL-terminated name.
    let mut namelen = unsafe { strlen(name) } as c_int;
    if namelen > 100 {
        namelen -= 100;
        // SAFETY: a position inside the name.
        namelen +=
            unsafe { utf_cp_bounds(name, name.wrapping_add(namelen as usize)) }.end_off as c_int;
        name = name.wrapping_add(namelen as usize);
    }
    let out = dst.as_mut_ptr();
    // SAFETY: at most 100 bytes and a NUL into `IOSIZE` writable ones.
    unsafe { strcpy(out, name) };
    // SAFETY: the string just copied, and the room it sits in.
    unsafe { trans_characters(out, IOSIZE) };
}

/// Whether `str` differs from what the cell holds, replacing it if it does.
/// Answers whether [`resettitle`] should be called.
fn value_change(str: *mut c_char, last: &GlobalCell<*mut c_char>) -> bool {
    let old = last.get();
    let differs = str.is_null() != old.is_null() || {
        // SAFETY: two NUL-terminated titles, neither null.
        !str.is_null() && !old.is_null() && unsafe { strcmp(str, old) } != 0
    };
    if !differs {
        return false;
    }
    // SAFETY: the previous title, this function's own allocation.
    unsafe { xfree(old.cast::<c_void>()) };
    if str.is_null() {
        last.set(ptr::null_mut());
        // SAFETY: sends the two titles to the UI.
        unsafe { resettitle() };
        return false;
    }
    // SAFETY: a NUL-terminated title.
    last.set(unsafe { xstrdup(str) });
    true
}

/// Send the current window title and icon text to the UI.
pub unsafe fn resettitle() {
    // SAFETY: two NUL-terminated titles, or null, which `cstr_as_string`
    // answers the empty string for.
    unsafe { ui_call_set_icon(cstr_as_string(lasticon.get())) };
    // SAFETY: as above.
    unsafe { ui_call_set_title(cstr_as_string(lasttitle.get())) };
}

// ---------------------------------------------------------------------------
// The ruler's two fragments

/// The relative cursor position -- "All", "Top", "Bot" or a percentage --
/// into `buf`.
pub unsafe fn get_rel_pos(wp: *mut win_T, buf: *mut c_char, buflen: c_int) -> c_int {
    // At least three characters are needed to write anything.
    if buflen < 3 {
        return 0;
    }
    // SAFETY: the caller's promise -- a live window.
    let win = unsafe { Win::new(wp) };
    let room = buflen as size_t;

    // The number of lines above the window.
    // SAFETY: a live window and one of its line numbers.
    let fill = unsafe { win_get_fill(wp, win.w_topline) };
    let mut above = win.w_topline - 1 + (fill - win.w_topfill) as linenr_T;
    if win.w_topline == 1 && win.w_topfill >= 1 {
        // All the buffer's lines are displayed and there is an indication of
        // filler lines, which can be considered seeing all of them.
        above = 0;
    }
    // The number of lines below it.
    let below = win.buffer().b_ml.ml_line_count - win.w_botline + 1;
    if below <= 0 {
        let all_or_bot = if above == 0 { tr(c"All") } else { tr(c"Bot") };
        // SAFETY: the caller's buffer, and a format taking one string.
        return unsafe { vim_snprintf_safelen(buf, room, c"%s".as_ptr(), all_or_bot) } as c_int;
    }
    if above <= 0 {
        let top = tr(c"Top");
        // SAFETY: as above.
        return unsafe { vim_snprintf_safelen(buf, room, c"%s".as_ptr(), top) } as c_int;
    }

    let perc = percentage(above, above + below);
    // The localized percentage value.
    let mut tmp: [c_char; 8] = [0; 8];
    let (dst, cap, fmt) = (tmp.as_mut_ptr(), tmp.len(), tr(c"%d%%"));
    // SAFETY: an eight-byte local, and a format taking one number.
    unsafe { vim_snprintf(dst, cap, fmt, perc) };
    let fmt = tr(c"%3s");
    // SAFETY: the caller's buffer, a format taking one string, and the local
    // just filled.
    unsafe { vim_snprintf_safelen(buf, room, fmt, tmp.as_ptr()) as c_int }
}

/// Append "(2 of 8)" to `buf`, when more than one file is being edited.
/// Answers how many characters that took.
pub unsafe fn append_arg_number(wp: *mut win_T, buf: *mut c_char, buflen: size_t) -> c_int {
    // Upstream asks the CURRENT window for the argument list even when
    // reporting on another one.
    // SAFETY: the current window's argument list is live.
    let argcount = unsafe { (*current_win().w_alist).al_ga.ga_len };
    if argcount <= 1 {
        // Nothing to do.
        return 0;
    }
    // SAFETY: the caller's promise -- a live window.
    let win = unsafe { Win::new(wp) };
    let fmt = if win.w_arg_idx_invalid != 0 {
        tr(c" ((%d) of %d)")
    } else {
        tr(c" (%d of %d)")
    };
    let idx = win.w_arg_idx + 1;
    // SAFETY: the caller's buffer, and a format taking two numbers.
    unsafe { vim_snprintf_safelen(buf, buflen, fmt, idx, argcount) as c_int }
}
