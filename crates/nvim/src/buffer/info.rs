//! Describing a buffer to the user -- `:ls`, CTRL-G and the title.
//!
//! [`buflist_list`] is `:ls`/`:buffers`, including the flag column and the
//! `'columns'`-aware truncation of each name; [`fileinfo`] is CTRL-G and the
//! message a `:edit` prints; [`get_rel_pos`] is the ruler's "Top"/"Bot"/"NN%"
//! and [`append_arg_number`] the `(2 of 5)` suffix.  [`maketitle`] builds
//! `'title'` and `'icon'` -- the same information again, for the window
//! manager.
//!
//! Upstream assembles the listing in `NameBuff` and `IObuff`, and
//! the rule here is the one the rest of the tree already follows: fill them
//! inside a `with_mut` borrow, then hand them on -- to `message_filtered`, to
//! `msg_display` -- through the cell rather than through a reference that is
//! still outstanding, because those callees re-enter the message machinery.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::fileio::Report;
use crate::formatted;
use crate::memline::MlFlags;
use crate::strings::has_char;
use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::charset::{strsize, trans_characters_in};
use crate::drawscreen::redrawing;
use crate::drawscreen::state::need_maketitle;
use crate::getchar::state::got_int;
use crate::mbyte::cp_bounds;
use crate::memory::XString;
use crate::message::state::{msg_col, msg_scroll, msg_scrolled, need_wait_return, no_lines_msg};
use crate::message::{
    message_filtered, msg, msg_display, msg_ext_set_kind, msg_fit_head, msg_putchar, msg_start,
    set_keep_msg,
};
use crate::r#move::validate_virtcol;
use crate::option::shortmess;
use crate::option::vars::{P_ICONSTRING, P_TITLESTRING, p_icon, p_ru, p_title, p_titlelen};
use crate::options::{kOptIconstring, kOptTitlestring};
use crate::os::cshim::{gettext, ngettext};
use crate::os::env::home_replace_in;
use crate::os::input::line_breakcheck;
use crate::path::path_tail;
use crate::plines::win_get_fill;
use crate::state::mode::restart_edit;
use crate::statusline::state::stl_syntax;
use crate::statusline::{FmtSource, build_stl_text};
use crate::terminal::buf_terminal_running;
use crate::types::String_0;
use crate::types::ui::kUIMessages;
use crate::types::{
    ExArg, IOSIZE, LineNr, MAXPATHL, OptIndex, OptInt, OptionSetFlags, ShmFlag, StlSyntax, int64_t,
};
use crate::ui::state::Columns;
use crate::ui::{ui_call_set_icon, ui_call_set_title, ui_has};
use crate::undo::{buf_is_changed, curbuf_is_changed, undo_time_text};
use crate::winlayer::{Buf, Win, buffers, first_buffer};

use super::list::sort_by_last_used;

fn current_win() -> Win {
    Win::current()
}

/// Whether `arg` contains the flag character `c` -- `:ls`'s argument is a
/// set of them.
fn has_flag(arg: &CStr, c: u8) -> bool {
    // `has_char` takes a codepoint, which every flag letter is.
    has_char(arg, c_int::from(c))
}

/// Copy as much of `text` into `out` as fits with a terminator after it, as
/// `snprintf` would: answers how many bytes went in. Nothing at all for an
/// empty `out`.
fn put_cut(out: &mut [c_char], text: &[u8]) -> usize {
    let Some(room) = out.len().checked_sub(1) else {
        return 0;
    };
    let len = text.len().min(room);
    for (to, &from) in out.iter_mut().zip(&text[..len]) {
        *to = from.cast_signed();
    }
    out[len] = 0;
    len
}

// ---------------------------------------------------------------------------
// :ls / :buffers

/// List the buffers, one line each, as `:ls` and `:files` do.
pub fn buflist_list(excmd: &mut ExArg) {
    let arg = excmd.line.cstr_from(excmd.line.arg);
    let forceit = excmd.forceit;
    msg_ext_set_kind(c"list_cmd");

    // With "t", the list is shown most-recently-used first.
    let sorted = has_flag(arg, b't').then(sorted_by_last_used);
    let mut walk = Walk::new(sorted.as_deref());

    while let Some(buf) = walk.step() {
        if got_int.get() {
            break;
        }
        if skip(buf, arg, c_int::from(forceit)) {
            continue;
        }
        // The name to show, then the message filter's verdict on it:
        // `message_filtered` re-enters the regexp engine, and `show` the
        // message machinery, so the name is this frame's own.
        let name = fill_name(buf);
        if message_filtered(name.as_cstr()) {
            continue;
        }
        show(buf, has_flag(arg, b't'), &name);
    }
}

/// Every buffer, most recently used first.
fn sorted_by_last_used() -> Vec<Buf> {
    let mut list: Vec<Buf> = buffers().collect();
    sort_by_last_used(&mut list);
    list
}

/// The walk `:ls` makes: over the sorted array when there is one, otherwise
/// down the buffer list itself. Upstream reads `b_next` after each line is
/// printed, which is what the second arm does.
struct Walk<'a> {
    sorted: Option<&'a [Buf]>,
    at: usize,
    next: Option<Buf>,
}

impl<'a> Walk<'a> {
    fn new(sorted: Option<&'a [Buf]>) -> Self {
        let next = match sorted {
            Some(list) => list.first().copied(),
            None => first_buffer(),
        };
        Walk {
            sorted,
            at: 0,
            next,
        }
    }

    /// Not `Iterator::next`, and not `winlayer::buffers()`: this walk reads
    /// `b_next` only after its caller has printed the line, which is where
    /// upstream's `for` increment reads it. Every walk in `winlayer` reads
    /// the link at yield time instead (`iter::successors`), which is the
    /// `FOR_ALL_*_SAFE` shape — see that module's "when the link is read".
    fn step(&mut self) -> Option<Buf> {
        let buf = self.next?;
        self.next = match self.sorted {
            Some(list) => {
                self.at += 1;
                list.get(self.at).copied()
            }
            None => buf.next(),
        };
        Some(buf)
    }
}

/// Whether the `:ls` flags in `arg` say to skip this buffer.
fn skip(buffer: Buf, arg: &CStr, forceit: c_int) -> bool {
    let is_terminal = !buffer.terminal.is_null();
    let job_running = buf_terminal_running(buffer);
    let loaded = !buffer.b_ml.ml_mfp.is_null();
    let alt_fnum = current_win().w_alt_fnum;

    buffer.b_p_bl == 0 && forceit == 0 && !has_flag(arg, b'u')
        || has_flag(arg, b'u') && buffer.b_p_bl != 0
        || has_flag(arg, b'+') && (buffer.b_flags.has(BufFlags::READERR) || !buf_is_changed(buffer))
        || has_flag(arg, b'a') && (!loaded || buffer.b_nwindows == 0)
        || has_flag(arg, b'h') && (!loaded || buffer.b_nwindows != 0)
        || has_flag(arg, b'R') && (!is_terminal || !job_running)
        || has_flag(arg, b'F') && (!is_terminal || job_running)
        || has_flag(arg, b'-') && buffer.b_p_ma != 0
        || has_flag(arg, b'=') && buffer.b_p_ro == 0
        || has_flag(arg, b'x') && !buffer.b_flags.has(BufFlags::READERR)
        || has_flag(arg, b'%') && buffer.raw() != Buf::current_raw()
        || has_flag(arg, b'#') && (buffer.raw() == Buf::current_raw() || alt_fnum != buffer.handle)
}

/// The name to show for `buffer`, as upstream cut it into `NameBuff`.
fn fill_name(buffer: Buf) -> XString {
    if let Some(special) = buf_spname(&buffer) {
        let special = special.to_bytes();
        return XString::from_bytes(&special[..special.len().min(MAXPATHL as usize - 1)]);
    }
    let fname = buffer.name.shown().unwrap_or(c"");
    home_replace_in(Some(buffer), fname, MAXPATHL as usize, true)
}

/// Print one buffer's line: the number, the flag column, the name padded to
/// column 40, and the line number or the time it was last used.
fn show(buffer: Buf, by_time: bool, name: &XString) {
    let changed_char = if buffer.b_flags.has(BufFlags::READERR) {
        b'x'
    } else if buf_is_changed(buffer) {
        b'+'
    } else {
        b' '
    };
    let mut ro_char = if buffer.b_p_ma == 0 {
        b'-'
    } else if buffer.b_p_ro != 0 {
        b'='
    } else {
        b' '
    };
    if !buffer.terminal.is_null() {
        ro_char = if buf_terminal_running(buffer) {
            b'R'
        } else {
            b'F'
        };
    }

    if !ui_has(kUIMessages) || msg_col.get() > 0 {
        msg_putchar(b'\n' as c_int);
    }

    let listed = if buffer.b_p_bl != 0 { b' ' } else { b'u' };
    let current = if buffer.raw() == Buf::current_raw() {
        b'%'
    } else if current_win().w_alt_fnum == buffer.handle {
        b'#'
    } else {
        b' '
    };
    let state = if buffer.b_ml.ml_mfp.is_null() {
        b' '
    } else if buffer.b_nwindows == 0 {
        b'h'
    } else {
        b'a'
    };
    let lnum = if buffer.raw() == Buf::current_raw() {
        current_win().w_cursor.lnum
    } else {
        buflist_findlnum(buffer)
    };
    let last_used = buffer.b_last_used;

    // `%3d%c%c%c%c%c "%s"`, cut where upstream's `IOSIZE - 20` bytes cut it.
    let mut line = format!("{:3}", buffer.handle).into_bytes();
    line.extend_from_slice(&[listed, current, state, ro_char, changed_char, b' ', b'"']);
    line.extend_from_slice(name.as_cstr().to_bytes());
    line.push(b'"');
    line.truncate(IOSIZE as usize - 21);

    // Put "line 999" in column 40, or after the file name.
    let mut i = 40 - strsize(&line);
    loop {
        line.push(b' ');
        i -= 1;
        if i <= 0 || line.len() >= IOSIZE as usize - 18 {
            break;
        }
    }
    let room = IOSIZE as usize - line.len();
    let tail = if by_time && last_used != 0 {
        undo_time_text(last_used, room).into_vec()
    } else {
        formatted!(room, gettext(c"line %ld").to_bytes(), lnum as int64_t)
    };
    line.extend(tail.into_iter().take_while(|&b| b != 0));
    msg_display(XString::from_vec(line).as_cstr(), 0, false);
    line_breakcheck();
}

// ---------------------------------------------------------------------------
// CTRL-G

/// The message CTRL-G and `:file` print: the name, the flags, where the
/// cursor is and how far through the file that is.
pub fn fileinfo(fullname: c_int, shorthelp: c_int, dont_truncate: bool) {
    let mut out = Report::new();
    let buf = Buf::current();

    if fullname > 1 {
        // 2 CTRL-G: include the buffer number.
        out.push_fmt(c"buf %d: ", &[buf.handle.into()]);
    }
    out.push(b"\"");

    if let Some(name) = buf_spname(&buf) {
        out.push(name.to_bytes());
    } else {
        let name = if fullname == 0 && !buf.name.is_unnamed() {
            buf.name.shown()
        } else {
            buf.name.full()
        };
        // Only a help buffer's name is shortened against the buffer's own
        // directory; every other name is shortened against `$HOME` alone.
        let helpbuf = (shorthelp != 0).then_some(buf);
        let shown = home_replace_in(helpbuf, name.unwrap_or(c""), out.room(), true);
        out.push(shown.as_cstr().to_bytes());
    }

    let dontwrite = buf_is_dontwrite(Some(buf));
    let modified = curbuf_is_changed();
    let note = |cond: bool, text: &'static CStr| if cond { gettext(text) } else { c"" };
    let parts: [&CStr; 6] = [
        if modified {
            if shortmess(ShmFlag::MOD) {
                c" [+]"
            } else {
                gettext(c" [Modified]")
            }
        } else {
            c" "
        },
        note(
            buf.b_flags.has(BufFlags::NOTEDITED) && !dontwrite,
            c"[Not edited]",
        ),
        note(buf.b_flags.has(BufFlags::NEW) && !dontwrite, c"[New]"),
        note(buf.b_flags.has(BufFlags::READERR), c"[Read errors]"),
        if buf.b_p_ro != 0 {
            if shortmess(ShmFlag::RO) {
                gettext(c"[RO]")
            } else {
                gettext(c"[readonly]")
            }
        } else {
            c""
        },
        if modified || buf.b_flags.has(BufFlags::WRITE_MASK) || buf.b_p_ro != 0 {
            c" "
        } else {
            c""
        },
    ];
    let args = parts.map(Into::into);
    out.push_fmt(c"\"%s%s%s%s%s%s", &args);

    let win = current_win();
    let lines = buf.b_ml.ml_line_count;
    let cursor = win.w_cursor.lnum;
    if buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
        out.push(gettext(no_lines_msg).to_bytes());
    } else if p_ru() {
        // The current line and column are already on the screen -- webb
        let fmt = ngettext(
            c"%ld line --%d%%--",
            c"%ld lines --%d%%--",
            lines as ::core::ffi::c_ulong,
        );
        let percent = percentage(cursor, lines);
        out.push_fmt(fmt, &[(lines as int64_t).into(), percent.into()]);
    } else {
        let fmt = gettext(c"line %ld of %ld --%d%%-- col ");
        let percent = percentage(cursor, lines);
        out.push_fmt(
            fmt,
            &[
                (cursor as int64_t).into(),
                (lines as int64_t).into(),
                percent.into(),
            ],
        );
        validate_virtcol(win);
        out.push(col_text(win.w_cursor.col as c_int + 1, win.w_virtcol as c_int + 1).as_bytes());
    }

    if let Some((fmt, idx, count)) = arg_number(win) {
        out.push_fmt(fmt, &[idx.into(), count.into()]);
    }

    if dont_truncate {
        // Temporarily set msg_scroll to keep the message from being
        // truncated; msg_start() first, to get it in the right place.
        msg_start();
        let n = msg_scroll.get();
        msg_scroll.set(1);
        msg(out.as_cstr(), 0);
        msg_scroll.set(n);
    } else {
        let p = msg_fit_head(out.as_cstr(), false, 0);
        // Repeat the message after redrawing when 'restart_edit' is set
        // (otherwise there is a delay before redrawing), or when the screen
        // scrolled but there is no wait-return prompt.
        if restart_edit.get() != 0 || msg_scrolled.get() != 0 && !need_wait_return.get() {
            set_keep_msg(p.as_ref().map(XString::as_cstr), 0);
        }
    }
}

fn percentage(part: LineNr, whole: LineNr) -> c_int {
    calc_percentage(part as int64_t, whole as int64_t)
}

/// The column indicator: `col` alone when the virtual column agrees with it,
/// `col-vcol` when it does not.
pub(crate) fn col_text(col: c_int, vcol: c_int) -> String {
    if col == vcol {
        format!("{col}")
    } else {
        format!("{col}-{vcol}")
    }
}

/// [`col_text`] written into a caller's buffer, cut to fit with its
/// terminator. Answers what was written.
pub fn col_print(buf: &mut [c_char], col: c_int, vcol: c_int) -> c_int {
    put_cut(buf, col_text(col, vcol).as_bytes()) as c_int
}

// ---------------------------------------------------------------------------
// 'title' and 'icon'

/// Build `'title'` and `'icon'` and, when either changed, tell the UI.
pub fn maketitle() {
    let mut scratch: [c_char; IOSIZE as usize] = [0; IOSIZE as usize];

    if !redrawing() {
        // Postpone updating the title when 'lazyredraw' is set.
        need_maketitle.set(true);
        return;
    }
    need_maketitle.set(false);
    if !p_title() && !p_icon() && lasttitle.with(Option::is_none) && lasticon.with(Option::is_none)
    {
        // Nothing to do.
        return;
    }

    // Copies: `build_stl` evaluates a statusline expression, which may set
    // an option.
    let (titlestring, iconstring) = (P_TITLESTRING.get(), P_ICONSTRING.get());
    let mut mustset = false;
    if p_title() {
        let mut maxlen = 0;
        if p_titlelen() > 0 as OptInt {
            maxlen = ((p_titlelen() * Columns.get() as OptInt / 100) as c_int).max(10);
        }
        let title = if !titlestring.is_empty() {
            if stl_syntax.get().has(StlSyntax::TITLE) {
                build_stl(&mut scratch, titlestring.as_cstr(), kOptTitlestring, maxlen);
                cstr::in_chars(&scratch)
            } else {
                titlestring.as_cstr()
            }
        } else {
            // Format: "fname + (path) (1 of 2) - Nvim".
            let default = c"%t%( %M%)%( (%{expand('%:p:~:h')})%)%a - Nvim";
            build_stl(&mut scratch, default, kOptTitlestring, maxlen);
            cstr::in_chars(&scratch)
        };
        mustset = value_change(Some(title), &lasttitle);
    } else {
        value_change(None, &lasttitle);
    }

    if p_icon() {
        let icon = if !iconstring.is_empty() {
            if stl_syntax.get().has(StlSyntax::ICON) {
                build_stl(&mut scratch, iconstring.as_cstr(), kOptIconstring, 0);
                cstr::in_chars(&scratch)
            } else {
                iconstring.as_cstr()
            }
        } else {
            fill_icon(&mut scratch);
            cstr::in_chars(&scratch)
        };
        mustset |= value_change(Some(icon), &lasticon);
    } else {
        mustset |= value_change(None, &lasticon);
    }

    if mustset {
        resettitle();
    }
}

/// `build_stl_str_hl` for the title and icon, which want the text and
/// nothing else it can report.
fn build_stl(dst: &mut [c_char; IOSIZE as usize], fmt: &CStr, opt: OptIndex, maxlen: c_int) {
    // The title and icon are global: neither names a scope.
    let from = FmtSource {
        opt_idx: opt,
        opt_scope: OptionSetFlags::NONE,
    };
    build_stl_text(current_win(), dst, fmt, from, maxlen);
}

/// The default icon text: the buffer's name, its last 100 bytes cut at a
/// character boundary.
fn fill_icon(dst: &mut [c_char; IOSIZE as usize]) {
    let buf = Buf::current();
    let name = match buf_spname(&buf) {
        Some(name) => name.to_bytes(),
        // `buf_spname` answered `None`, so the buffer has a name.
        None => buf.name.full().map_or(c"", path_tail).to_bytes(),
    };
    let mut start = 0;
    if name.len() > 100 {
        let cut = name.len() - 100;
        start = cut + cp_bounds(name, cut).end_off as usize;
    }
    let mut out = [0u8; IOSIZE as usize];
    out[..name.len() - start].copy_from_slice(&name[start..]);
    trans_characters_in(&mut out);
    put_cut(dst, cstr::until_nul(&out));
}

/// Whether `now` differs from what the cell holds, replacing it if it does.
/// Answers whether [`resettitle`] should be called.
fn value_change(now: Option<&CStr>, last: &GlobalCell<Option<XString>>) -> bool {
    if last.with(|old| old.as_deref() == now.map(CStr::to_bytes)) {
        return false;
    }
    let Some(now) = now else {
        last.set(None);
        resettitle();
        return false;
    };
    last.set(Some(XString::from_cstr(now)));
    true
}

/// Send the current window title and icon text to the UI.
pub fn resettitle() {
    // A title the UI has never been told is upstream's null pointer, which
    // `cstr_to_string` answered the empty string for.
    let icon = lasticon.with(|value| String_0::from_bytes(value.as_deref().unwrap_or_default()));
    ui_call_set_icon(icon);
    let title = lasttitle.with(|value| String_0::from_bytes(value.as_deref().unwrap_or_default()));
    ui_call_set_title(title);
}

// ---------------------------------------------------------------------------
// The ruler's two fragments

/// The relative cursor position -- "All", "Top", "Bot" or a percentage --
/// into `buf`, cut to fit with its terminator. Answers what was written.
pub fn get_rel_pos(window: Win, buf: &mut [c_char]) -> c_int {
    // At least three characters are needed to write anything.
    if buf.len() < 3 {
        return 0;
    }
    let win = window;

    // The number of lines above the window.
    let fill = win_get_fill(win, win.w_topline);
    let mut above = win.w_topline - 1 + (fill - win.w_topfill) as LineNr;
    if win.w_topline == 1 && win.w_topfill >= 1 {
        // All the buffer's lines are displayed and there is an indication of
        // filler lines, which can be considered seeing all of them.
        above = 0;
    }
    // The number of lines below it.
    let below = win.buffer().b_ml.ml_line_count - win.w_botline + 1;
    if below <= 0 {
        let all_or_bot = if above == 0 {
            gettext(c"All")
        } else {
            gettext(c"Bot")
        };
        return put_cut(buf, all_or_bot.to_bytes()) as c_int;
    }
    if above <= 0 {
        return put_cut(buf, gettext(c"Top").to_bytes()) as c_int;
    }

    // The localized percentage value, as an eight-byte buffer held it. Read
    // on every ruler redraw: the untranslated templates are formatted
    // directly, which the printf emulator costs several times over.
    let perc = percentage(above, above + below);
    static PERCENT: &CStr = c"%d%%";
    static PADDED: &CStr = c"%3s";
    let (percent, padded) = (gettext(PERCENT), gettext(PADDED));
    if ptr::eq(percent, PERCENT) && ptr::eq(padded, PADDED) {
        return put_cut(buf, format!("{perc:>2}%").as_bytes()) as c_int;
    }
    let value = formatted!(8, percent.to_bytes(), perc);
    let text = formatted!(buf.len(), padded.to_bytes(), value.as_slice());
    put_cut(buf, &text) as c_int
}

/// The "(2 of 8)" format and its two numbers, when more than one file is
/// being edited.
fn arg_number(window: Win) -> Option<(&'static CStr, c_int, c_int)> {
    // Upstream asks the CURRENT window for the argument list even when
    // reporting on another one.
    let argcount = crate::arglist::wargcount(current_win());
    if argcount <= 1 {
        // Nothing to do.
        return None;
    }
    let fmt = if window.w_arg_idx_invalid {
        gettext(c" ((%d) of %d)")
    } else {
        gettext(c" (%d of %d)")
    };
    Some((fmt, window.w_arg_idx + 1, argcount))
}

/// Write "(2 of 8)" into `buf`, cut to fit with its terminator, when more
/// than one file is being edited. Answers how many characters that took.
pub fn append_arg_number(window: Win, buf: &mut [c_char]) -> c_int {
    let Some((fmt, idx, count)) = arg_number(window) else {
        return 0;
    };
    let text = formatted!(buf.len(), fmt.to_bytes(), idx, count);
    put_cut(buf, &text) as c_int
}
