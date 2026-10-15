//! Messages: what the editor says, and where it says it.
//!
//! [`msg`] and the `emsg`/`smsg` families start and end a message;
//! [`msg_str`] and its kin *append* to the one being built, and everything
//! displayed funnels through [`msg_bytes`], which feeds `:redir`, the
//! history, and then either the message grid ([`msg_bytes_to_grid`]) or
//! plain stdio ([`msg_bytes_to_stdio`]).
//!
//! # What the entry points take
//!
//! Two shapes, and which one a function takes says what its text *is*
//! (`cstr`'s convention, § The parameter convention):
//!
//! - **`&CStr`** for a whole string the caller already has terminated —
//!   a literal, a `gettext` translation, an option value, a file name, a
//!   group name. That is nearly every message in the tree, and passing one
//!   costs nothing.
//! - **`&[u8]`** for a *span*: a run inside a command line, a tag field
//!   between two pointers, a chunk of an API `String`. These have a length
//!   of their own and no terminator to find.
//!
//! There is no third shape and no length parameter. The `-1`-means-
//! NUL-terminated `len` the C carried is gone: the end of the slice is the
//! end of the text, and a NUL inside one is an ordinary byte that the
//! translating half shows as `^@`.
//!
//! | upstream | here | takes |
//! | --- | --- | --- |
//! | `msg_puts` | [`msg_str`] | `&CStr` |
//! | `msg_puts_hl` | [`msg_str_hl`] | `&CStr` |
//! | `msg_puts_title` | [`msg_title`] | `&CStr` |
//! | `msg_puts_len` | [`msg_bytes`] | `&[u8]` |
//! | `msg_puts_display` | [`msg_bytes_to_grid`] | `&[u8]` |
//! | `msg_puts_printf` | [`msg_bytes_to_stdio`] | `&[u8]` |
//! | `msg_outtrans` | [`msg_display`] | `&CStr` |
//! | `msg_outtrans_len` | [`msg_display_bytes`] | `&[u8]` |
//! | `msg_outtrans_one` | [`msg_display_char`] | `&[u8]`, answers bytes |
//! | `msg_outtrans_special` | [`msg_display_keys`] | `&CStr` |
//! | `msg_outtrans_long` | [`msg_display_elided`] | `&CStr` |
//!
//! The `msg_display*` half is the one that renders what cannot be shown:
//! a control character as `^X`, anything else unprintable as `<xx>`. The
//! `msg_str*` half puts the bytes through as they are.
//!
//! [`msg_part`] and [`msg_display_part`] are the same two funnels for a
//! *piece* of a message already being built, and differ in one thing: an
//! empty piece adds nothing, where an empty *message* is something an
//! `ext_messages` UI is told about. Upstream told them apart by reading the
//! byte under the pointer — a zero-length span inside a longer buffer does
//! not start at a NUL — which is exactly what a slice cannot do.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub mod state;
use crate::api::vim::nvim_echo;
use crate::ascii::ascii_iswhite;
use crate::autocmd::has_event;
use crate::charset::{
    byte2cells, char2cells, ptr2cells_at, strsize, transchar_buf, transchar_byte_buf, vim_isprintc,
    vim_strsize_cstr,
};
use crate::cstr;
use crate::drawscreen::state::{
    clear_cmdline, cmdline_row, do_redraw, mode_displayed, redraw_cmdline, sc_col, skip_redraw,
};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_later, set_must_redraw,
};
use crate::eval::callback_call;
use crate::eval::typval::tv_clear;
use crate::eval::vars::{set_vim_var_string, var_redir_str, with_vim_var_str};
use crate::ex_docmd::do_sleep;
use crate::ex_docmd::state::global_busy;
use crate::ex_eval::cause_errthrow;
use crate::ex_getln::state::{cmdline_was_last_drawn, redrawing_cmdline};
use crate::fileio::check_timestamps;
use crate::fileio::state::need_check_timestamps;
use crate::getchar::state::{
    KeyTyped, got_int, reg_recording, scriptout, vgetc_busy, vgetc_char, vgetc_mod_mask,
};
use crate::getchar::{
    beep_flush, char_avail, flush_buffers, ins_char_typebuf, safe_vgetc, stuff_empty,
    typeahead_noflush,
};
use crate::global_cell::GlobalCell;
use crate::grid::{
    GridRef, grid_adjust, grid_alloc, grid_assign_handle, grid_clear, grid_del_lines,
    grid_ins_lines, grid_line_cursor_goto, grid_line_flush, grid_line_flush_if_valid_row,
    grid_line_mirror, grid_line_puts, grid_line_start,
};
use crate::guard::{Depth, Suppress};
use crate::highlight::hl_combine_attr;
use crate::highlight::state::need_highlight_changed;
use crate::highlight_group::{
    HLF_0, HLF_8, HLF_AT, HLF_E, HLF_M, HLF_MSG, HLF_N, HLF_R, HLF_T, HLF_W, highlight_changed,
    syn_check_group, syn_id2attr,
};
use crate::input::{get_keystroke, prompt_for_input};
use crate::keycodes::{K_SPECIAL, get_special_key_name};
use crate::log::{LOGLVL_DBG, LOGLVL_INF};
use crate::mbyte::{
    c_cluster_len, c_display_width, cells_at, head_off, mb_tolower, utf_char2cells, utf8len_tab,
};
use crate::memory::XString;
use crate::message::state::{
    called_emsg, cmd_silent, cmdmsg_rl, confirm_buttons, confirm_msg, confirm_msg_used, did_emsg,
    did_wait_return, do_clear_hist_temp, do_clear_sb_text, emsg_assert_fails_context,
    emsg_assert_fails_lnum, emsg_assert_fails_msg, emsg_noredir, emsg_off, emsg_on_display,
    emsg_severe, emsg_silent, emsg_skip, in_assert_fails, info_message, is_multihl, keep_msg,
    keep_msg_hl_id, keep_msg_more, last_sourcing_lnum, last_sourcing_name, lines_left,
    more_prompt_busy, msg_capture, msg_col, msg_did_scroll, msg_didany, msg_didout, msg_ext_append,
    msg_ext_chunks, msg_ext_history, msg_ext_id, msg_ext_kind, msg_ext_last_attr,
    msg_ext_last_chunk, msg_ext_last_hl_id, msg_ext_overwrite, msg_ext_skip_flush,
    msg_ext_skip_verbose, msg_ext_trigger, msg_flags, msg_grid, msg_grid_pos,
    msg_grid_pos_at_flush, msg_grid_scroll_discount, msg_hist_off, msg_id_next, msg_keep_depth,
    msg_no_more, msg_nowait, msg_row, msg_scroll, msg_scrolled, msg_scrolled_at_flush,
    msg_scrolled_ign, msg_silent, msg_source_busy, msg_wait, need_clr_eos, need_fileinfo,
    need_wait_return, no_wait_return, on_print, pre_verbose_kind, progress_msg_target, quit_more,
    redir_col, redir_fd, redir_off, redir_reg, redir_vname, showmode_clear_pending,
    verbose_did_open, verbose_fd,
};
use crate::mouse::{MOUSE_SETPOS, jump_to_mouse, setmouse};
use crate::option::shortmess;
use crate::option::vars::p_vfile;
use crate::option::vars::{
    P_MORE, p_ch, p_debug, p_eb, p_lz, p_more, p_report, p_verbose, rdb_flags,
};
use crate::options::{
    kOptBoFlagMess, kOptBoFlagShell, kOptMoptFlagHistory, kOptMoptFlagHitEnter,
    kOptMoptFlagProgress, kOptMoptFlagWait, kOptRdbFlagNothrottle,
};
use crate::os::cshim::{gettext, ngettext};
use crate::os::input::{input_available, os_breakcheck};
use crate::os::time::os_delay;
use crate::pos::MAXCOL;
use crate::register::write_reg_contents;
use crate::startup::{
    embedded_mode, ex_exitval, exiting, full_screen, headless_mode, nvim_testing, silent_mode,
};
use crate::state::mode::{State, exmode_active};
use crate::state::{MODE_ASKMORE, MODE_CMDLINE, MODE_EXTERNCMD, MODE_HITRETURN, MODE_SETWSIZE};
use crate::statusline::hl_attr;
use crate::types::AutoEvent;
use crate::types::BS;
use crate::types::CAR;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use crate::types::ui::{kUIMessages, kUIMultigrid};
use crate::types::{
    Array, EStackArg, ExArg, FlushBuffers, GridView, HlMessage, HlMessageChunk, IOSIZE, Integer,
    KeyDict_echo_opts, MessageData, Object, OptInt, ScreenAttr, ScreenChar, ShmFlag, String_0,
    TypVal, Vv, int64_t, size_t, uint64_t,
};
use crate::ui::state::{Columns, Rows, resize_events};
use crate::ui::{
    ui_active, ui_call_grid_destroy, ui_call_grid_resize, ui_call_grid_scroll,
    ui_call_msg_history_show, ui_call_msg_set_pos, ui_call_msg_show, ui_call_msg_showmode,
    ui_cursor_goto, ui_flush, ui_grid_cursor_goto, ui_has, ui_line, ui_refresh, vim_beep,
};
use crate::ui_compositor::{ui_comp_put_grid, ui_comp_remove_grid};
use core::ffi::{CStr, c_int, c_uint};
use std::borrow::Cow;

// The carve of the transpiled module; see each child's docs.
mod grid;
pub use self::grid::*;
mod scrollback;
pub use self::scrollback::*;
mod outtrans;
pub use self::outtrans::*;
mod prtline;
pub use self::prtline::*;
mod puts;
pub use self::puts::*;
mod ext;
pub use self::ext::*;
mod history;
pub use self::history::*;
mod prompt;
pub use self::prompt::*;
mod dialog;
pub use self::dialog::*;
mod redir;
pub use self::redir::*;
mod progress;
pub use self::progress::*;
mod errors;
pub use self::errors::*;
/// The compositor layer messages float on.
pub const kZIndexMessages: c_uint = 200;
/// [`do_dialog`] answers, as `confirm()` reports them.
pub const VIM_DISCARDALL: c_uint = 6;
pub const VIM_ALL: c_uint = 5;
pub const VIM_CANCEL: c_uint = 4;
pub const VIM_NO: c_uint = 3;
pub const VIM_YES: c_uint = 2;
/// How much of the scrollback the next message should drop.
pub type ScrollbackClear = ::core::ffi::c_uint;
pub const SB_CLEAR_CMDLINE_DONE: ScrollbackClear = 3;
pub const SB_CLEAR_CMDLINE_BUSY: ScrollbackClear = 2;
pub const SB_CLEAR_ALL: ScrollbackClear = 1;
pub const SB_CLEAR_NONE: ScrollbackClear = 0;
pub const ESTACK_NONE: EStackArg = 0;
pub const FLUSH_MINIMAL: FlushBuffers = 0;
/// A dialog's button list: `&` marks the next character as its hotkey, and
/// a newline separates buttons.
pub const DLG_HOTKEY_CHAR: c_uint = 38;
pub const DLG_BUTTON_SEP: c_uint = 10;
pub const BELL: ::core::ffi::c_int = '\u{7}' as ::core::ffi::c_int;
pub const PROGRESS_TARGET_CMD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

/// `grid_clear` over a rectangle of the message grid, in the message
/// highlight -- C's repeated
/// `grid_clear(&msg_grid_adj, ..., HL_ATTR(HLF_MSG))`.
fn clear_msg_area(top: c_int, bot: c_int, left: c_int, right: c_int) {
    let grid = msg_grid_view();
    let attr = hl_attr(HLF_MSG as c_int);
    grid_clear(grid, top, bot, left, right, attr);
}

/// An [`Array`] owning nothing, C's `ARRAY_DICT_INIT`.
pub(crate) const EMPTY_ARRAY: Array = Array::EMPTY;

/// Show a message.
///
/// Answers false when the message needed a hit-enter prompt that has not been
/// answered yet.
pub fn msg(s: &CStr, hl_id: c_int) -> bool {
    show_message(s, hl_id, false, false)
}

/// [`msg`], keeping the message displayed.
/// [`smsg_keep!`](crate::smsg_keep)'s tail.
#[doc(hidden)]
pub(crate) fn msg_keep_text(s: &CStr, hl_id: c_int) -> bool {
    show_message(s, hl_id, true, false)
}

/// Show `text`, splitting it at the control characters that need handling
/// of their own: a newline, tab or carriage return is emitted as itself,
/// and a bell rings instead of being drawn.
///
/// `need_clear` is cleared once the rest of the line has been cleared.
pub fn msg_multiline(
    text: &[u8],
    hl_id: c_int,
    check_int: bool,
    hist: bool,
    need_clear: &mut bool,
) {
    let mut chunk = 0;
    let mut at = 0;
    while at < text.len() {
        if check_int && got_int.get() {
            return;
        }
        if matches!(text[at], b'\n' | b'\t' | b'\r' | 0x07) {
            msg_display_part(&text[chunk..at], hl_id, hist);
            if c_int::from(text[at]) != TAB && *need_clear {
                msg_clr_eos();
                *need_clear = false;
            }
            if c_int::from(text[at]) == BELL {
                vim_beep(kOptBoFlagShell as c_uint);
            } else {
                msg_putchar_hl(c_int::from(text[at]), hl_id);
            }
            chunk = at + 1;
        }
        at += 1;
    }
    // The tail, and the whole of an empty message: an empty `text` still has
    // to reach `msg_display_bytes`, which is what clears the line and tells
    // an `ext_messages` UI that a message with no text was shown. Unlike the
    // pieces above, an empty tail only happens when the whole message was
    // empty -- a tail that starts at the end of the text is skipped here.
    if cstr::byte_at(text, chunk) != 0 || chunk == 0 {
        msg_display_bytes(&text[chunk..], hl_id, hist);
    }
}

/// Show a message made of chunks, each with its own highlight id.
///
/// Answers the message's id: `id` itself when it names one, and a freshly
/// allocated one when it is nil. `needs_msg_clear` is set when the message
/// was not shown on the command line, or was shown reformatted.
///
/// The chunks are the caller's; the history keeps a copy of its own.
pub fn msg_multihl(
    id: Object,
    hl_msg: &[HlMessageChunk],
    kind: Option<&CStr>,
    history: bool,
    err: bool,
    msg_data: Option<&MessageData>,
    needs_msg_clear: &mut bool,
) -> Object {
    // Message `id`:
    // - Nil: generate a new Integer id.
    // - Integer: an existing id.
    // - String: a user-defined id, new or existing.
    let id = if id.is_nil() {
        let next = msg_id_next.get();
        msg_id_next.set(next + 1);
        Object::integer(next)
    } else {
        if let Object::Integer(n) = id
            && !msg_id_exists(n)
        {
            std::process::abort();
        }
        id
    };

    let is_progress = kind == Some(c"progress");
    // Don't display a progress message on the command line when the
    // target does not include it.
    if is_progress && progress_msg_target.get() & PROGRESS_TARGET_CMD == 0 {
        *needs_msg_clear = true;
        return id;
    }

    let no_prompt = Suppress::wait_return();
    msg_start();
    msg_clr_eos();
    let mut need_clear = false;
    if kind.is_some() {
        msg_ext_set_kind_opt(kind);
    }
    msg_ext_skip_flush.set(true);
    msg_ext_id.set(id.clone());

    // A progress message displays as "title: percent% msg".
    let formatted = msg_data
        .filter(|_| is_progress)
        .and_then(|data| format_progress_message(hl_msg, data));
    if formatted.is_some() {
        *needs_msg_clear = true;
    }
    let shown = formatted.as_deref().unwrap_or(hl_msg);

    for chunk in shown {
        is_multihl.set(is_multihl.get() + 1);
        if err {
            emsg_multiline(chunk.text.as_cstr(), kind, chunk.hl_id, true);
        } else {
            msg_multiline(
                chunk.text.as_bytes(),
                chunk.hl_id,
                true,
                false,
                &mut need_clear,
            );
        }
        debug_assert!(
            !ui_has(kUIMessages)
                || kind.is_none_or(|kind| msg_ext_kind.with(|held| held.as_bytes() == kind.to_bytes()))
        );
    }

    if history && !shown.is_empty() {
        msg_hist_add_multihl(shown.to_vec(), false);
    }

    msg_ext_skip_flush.set(false);
    is_multihl.set(0);
    drop(no_prompt);
    msg_end();
    id
}

/// Show a message, optionally keeping it displayed after a redraw.
///
/// `keep` sets `keep_msg` when the message fits without scrolling;
/// `multiline` sends it through [`msg_multiline`] rather than
/// [`msg_display`]. Upstream's `msg_keep`.
pub(crate) fn show_message(s: &CStr, hl_id: c_int, keep: bool, multiline: bool) -> bool {
    if keep && multiline {
        // Not implemented. 'multiline' is only used by nvim-added
        // messages, which should avoid 'keep' behaviour -- they should
        // just show the message at the right time already.
        std::process::abort();
    }

    // Skip messages that do not match ":filter pattern", but never filter
    // when there is an error.
    if !emsg_on_display.get() && message_filtered(s) {
        return true;
    }

    if hl_id == 0 {
        set_vim_var_string(Vv::Statusmsg, Some(s.to_bytes()));
    }

    // Displaying a message can cause a problem (e.g. when redrawing the
    // window), which causes another message, and so on. Break the loop by
    // limiting the recursion to three levels.
    if msg_keep_depth.get() >= 3 {
        return true;
    }
    let _entered = Depth::of(msg_keep_depth);

    // Add the message to the history unless it is a multihl.
    //
    // Upstream also skipped a message whose *pointer* was `keep_msg`'s --
    // the kept message redisplayed, unless it differed from the newest
    // entry. Every redisplay hands this a copy (`keep_msg` owns its string
    // and the display may replace it), so that test could never be true
    // here, and the redisplay sites that must not add to the history say
    // so with `msg_hist_off`.
    if is_multihl.get() == 0 {
        msg_hist_add(s.to_bytes(), hl_id);
    }

    if is_multihl.get() == 0 {
        msg_start();
    }

    // Truncate the message if needed.
    let truncated = fit_middle(s.to_bytes(), false);
    let text = truncated.as_ref().map_or(s, XString::as_cstr);

    let mut need_clear = true;
    if multiline {
        msg_multiline(text.to_bytes(), hl_id, false, false, &mut need_clear);
    } else {
        msg_display(text, hl_id, false);
    }
    if need_clear {
        msg_clr_eos();
    }

    let mut retval = true;
    if is_multihl.get() == 0 {
        retval = msg_end();
    }

    if keep
        && retval
        && vim_strsize_cstr(text)
            < (Rows.get() - cmdline_row.get() - 1) * Columns.get() + sc_col.get()
    {
        set_keep_msg(Some(text), 0);
    }

    need_fileinfo.set(false);

    retval
}

/// `text` cut in the middle so it prints without causing a scroll; `None`
/// when no cut was needed. `force` cuts regardless of `'shortmess'`.
/// Upstream's `msg_strtrunc`.
pub(crate) fn fit_middle(text: &[u8], force: bool) -> Option<XString> {
    // May truncate the message to avoid a hit-return prompt.
    let wanted = msg_scroll.get() == 0
        && !need_wait_return.get()
        && shortmess(ShmFlag::TRUNCALL)
        && !exmode_active.get()
        && msg_silent.get() == 0
        && !ui_has(kUIMessages);
    if !(wanted || force) {
        return None;
    }
    let len = strsize(text);
    let room = if msg_scrolled.get() != 0 {
        // Use all the columns.
        (Rows.get() - msg_row.get()) * Columns.get() - 1
    } else {
        // Use up to the 'showcmd' column.
        (Rows.get() - msg_row.get() - 1) * Columns.get() + sc_col.get() - 1
    };
    if len <= room || room <= 0 {
        return None;
    }
    // Up to 18 bytes per cell: six per character, and up to two composing
    // characters.
    let capacity = usize::try_from(room + 2).unwrap_or(0) * 18;
    Some(XString::from_bytes(&cut_middle(text, room, capacity)))
}

/// `text` cut to fit `room_in` cells, the middle replaced with `...`, as a
/// buffer of `capacity` bytes would hold it -- so at most `capacity - 1`
/// bytes. Upstream's `trunc_string`. A NUL in `text` ends it.
///
/// The two halves are chosen by cell width and then compared as byte
/// offsets (`i <= e + 3`), so a line that is exactly `room_in` cells wide
/// survives whole only when its characters are one byte per cell — upstream
/// behaviour, asserted in `tests/unit/message.rs`.
///
/// `room_in < 3` clamping the room to zero is redundant: with the room left
/// negative the head loop still does not run and the tail loop still breaks
/// on its first character, so every `room_in` below 3 answers "..." either
/// way. It is kept because it is upstream's and it says what was meant.
///
/// Where the text fits the room but not `capacity`, upstream left the
/// buffer without a terminator, and where a character straddled the end of
/// the buffer it wrote one byte past it; here both answer the first
/// `capacity - 1` bytes of what it would have written.
pub fn cut_middle(text: &[u8], room_in: c_int, capacity: usize) -> Vec<u8> {
    let s = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    if s.is_empty() || capacity == 0 {
        return Vec::new();
    }
    let buflen = capacity;
    // The C string's bytes: its terminator past the end.
    let at = |i: usize| s.get(i).copied().unwrap_or(0);
    let mut buf = vec![0u8; buflen];
    let put = |buf: &mut Vec<u8>, i: usize, byte: u8| {
        if let Some(slot) = buf.get_mut(i) {
            *slot = byte;
        }
    };
    let finish = |mut buf: Vec<u8>| {
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buflen - 1);
        buf.truncate(end);
        buf
    };

    let room = if room_in < 3 { 0 } else { room_in - 3 }; // "..." takes 3 chars
    let mut len = 0;
    let half = room / 2;

    // First part: the start of the string.
    let mut e = 0;
    while len < half && e < buflen {
        if at(e) == 0 {
            // Text fits without truncating.
            put(&mut buf, e, 0);
            return finish(buf);
        }
        let n = ptr2cells_at(&s[e..]);
        if len + n > half {
            break;
        }
        len += n;
        put(&mut buf, e, s[e]);
        // Copy the rest of a multibyte character one byte at a time; the
        // inner break leaves the outer step to run, as upstream's does.
        let mut n = c_cluster_len(&s[e..]);
        loop {
            n = n.saturating_sub(1);
            if n == 0 {
                break;
            }
            e += 1;
            if e == buflen {
                break;
            }
            put(&mut buf, e, s[e]);
        }
        e += 1;
    }

    // Last part: the end of the string.
    let mut i = s.len();
    let mut half = i;
    loop {
        half = half - head_off(s, half - 1) - 1;
        let n = ptr2cells_at(&s[half..]);
        if len + n > room || half == 0 {
            break;
        }
        len += n;
        i = half;
    }

    if i <= e + 3 {
        // Text fits without truncating.
        let kept = s.len().min(buflen - 1);
        if kept < e {
            put(&mut buf, e.wrapping_sub(1), 0);
        } else {
            for k in e..=kept {
                put(&mut buf, k, at(k));
            }
        }
    } else if e + 3 < buflen {
        // Set the middle and copy the last part.
        for (k, &dot) in b"...".iter().enumerate() {
            put(&mut buf, e + k, dot);
        }
        let tail = (s.len() - i + 1).min(buflen - e - 4);
        for k in 0..tail {
            put(&mut buf, e + 3 + k, at(i + k));
        }
        put(&mut buf, (e + 3 + tail).wrapping_sub(1), 0);
    } else {
        // Can't fit the "...", so just truncate.
        put(&mut buf, buflen - 1, 0);
    }
    finish(buf)
}

/// How much of a message the `*_c!` macros keep: the length of upstream's
/// shared `IObuff`, which is what they used to format through.
pub const MSG_IOBUFF_LEN: size_t = IOSIZE as size_t;

/// Show `text`, cut at the head if it does not fit the message area.
///
/// Answers the text that was shown, or `None` if it was not. Upstream's
/// `msg_trunc`, which cut the caller's buffer in place.
pub fn msg_fit_head(text: &CStr, force: bool, hl_id: c_int) -> Option<XString> {
    // The history gets the whole message; only the display is cut.
    msg_hist_add(text.to_bytes(), hl_id);
    let shown = XString::from_bytes(&fit_head(text.to_bytes(), force));
    msg_hist_off.set(true);
    let n = msg(shown.as_cstr(), hl_id);
    msg_hist_off.set(false);
    n.then_some(shown)
}

/// `text` with its *head* dropped if it does not fit the message area, the
/// cut marked with a `<`. Upstream's `msg_may_trunc`, which wrote the `<`
/// into the caller's string and answered a pointer into it.
pub fn fit_head(text: &[u8], force: bool) -> Cow<'_, [u8]> {
    // Under ext_messages the UI decides what fits. This guard changes no
    // answer -- nothing shrinks `cmdline_row` off `Rows` there, so `room`
    // is already negative -- but it says what the intent is; see docket
    // O-B13-6.
    if ui_has(kUIMessages) {
        return Cow::Borrowed(text);
    }
    let s = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    let room = (Rows.get() - cmdline_row.get() - 1) * Columns.get() + sc_col.get() - 1;
    let len = c_int::try_from(s.len()).unwrap_or(c_int::MAX);
    if room > 0 && (force || (shortmess(ShmFlag::TRUNC) && !exmode_active.get())) && len - room > 0
    {
        // May have up to 18 bytes per cell (6 per char, up to two
        // composing chars).
        let mut size = c_display_width(s, MAXCOL);
        if size <= room {
            return Cow::Borrowed(text);
        }
        // Find the last character that fits.
        let mut n = 0;
        while size >= room {
            size -= cells_at(&s[n.min(s.len())..]);
            n += c_cluster_len(&s[n.min(s.len())..]);
        }
        n -= 1;
        let mut cut = s[n..].to_vec();
        cut[0] = b'<';
        return Cow::Owned(cut);
    }
    Cow::Borrowed(text)
}

/// Keep a copy of `s` to show again after the next redraw, or with `None`
/// forget the kept message.
///
/// Nothing is kept under `:silent`.
pub fn set_keep_msg(s: Option<&CStr>, hl_id: c_int) {
    // The kept message is not cleared and re-emitted with ext_messages:
    // neovim/neovim#20416.
    if ui_has(kUIMessages) {
        return;
    }
    keep_msg.set(s.filter(|_| msg_silent.get() == 0).map(XString::from_cstr));
    keep_msg_more.set(false);
    keep_msg_hl_id.set(hl_id);
}

/// Is there a message to show again after the next redraw?
pub(crate) fn has_keep_msg() -> bool {
    keep_msg.with(Option::is_some)
}

/// Would a message be seen if it were shown now?
pub fn messaging() -> bool {
    !(p_lz() && char_avail() && !KeyTyped.get()) && (p_ch() > 0 || ui_has(kUIMessages))
}

/// Report "N more lines" / "N fewer lines" after an edit, if `'report'`
/// allows it.
pub fn msgmore(n: c_int) {
    if global_busy.get() != 0 || !messaging() {
        // Don't report when :global is executing.
        return;
    }
    // Keep the message from a previous msgmore(), but not another one.
    if has_keep_msg() && !keep_msg_more.get() {
        return;
    }

    // Wrapping, as C's `abs` does on the one value it cannot negate.
    let pn = n.wrapping_abs();
    if OptInt::from(pn) <= p_report() {
        return;
    }
    let (one, many) = if n > 0 {
        (c"%d more line", c"%d more lines")
    } else {
        (c"%d line less", c"%d fewer lines")
    };
    let fmt = ngettext(one, many, pn as ::core::ffi::c_ulong);
    // Upstream assembles the report in a `MSG_BUF_LEN` buffer, and the note
    // is cut to what is left of it.
    let room = MSG_BUF_LEN as usize;
    let mut text = crate::formatted!(room, fmt.to_bytes(), pn);
    if got_int.get() {
        let note = gettext(c" (Interrupted)").to_bytes();
        let left = (room - 1).saturating_sub(text.len());
        text.extend_from_slice(&note[..note.len().min(left)]);
    }
    let text = XString::from_bytes(&text);
    if msg(text.as_cstr(), 0) {
        set_keep_msg(Some(text.as_cstr()), 0);
        keep_msg_more.set(true);
    }
}
