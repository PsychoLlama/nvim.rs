//! `msg_str` and its display half: text onto the message grid.
//!
//! [`msg_bytes`] is the funnel every message eventually reaches; it feeds
//! the redirection sinks and then [`msg_bytes_to_grid`], which lays the text
//! out cell by cell, scrolls when it runs off the bottom and raises the pager
//! when `'more'` says to. [`msg_bytes_to_stdio`] is the same job for a process
//! with no UI at all.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::eval::typval::CallFrame;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::ex_docmd::cmdmod_filters_out;
use crate::grid::default_grid_ref;
use crate::mbyte::{cells_at, char_at, char_len, cluster_len, string_cells};
use crate::types::{Callback, NUL};
use core::ffi::{c_int, c_uint};
use core::ptr;

/// The `on_print` callback an RPC client installed.
///
/// The address, because every operation the tree has on a callback —
/// parsing an option into it, marking it for the collector, copying it,
/// calling it — takes a `*mut Callback`.
pub(crate) fn on_print_cb() -> *mut Callback {
    on_print.ptr()
}

/// C's `ARRAY_DICT_INIT`: empty, and owning nothing.
const EMPTY_ARRAY: Array = Array::EMPTY;

/// Start putting a message on the screen.
///
/// Decides *where* the message goes: over the last one, or on a fresh line
/// below it, scrolling if there is no room.
pub fn msg_start() {
    let mut did_return = false;
    msg_row.set(msg_row.get().max(cmdline_row.get()));

    if msg_silent.get() == 0 {
        // Don't display the old message now.
        unsafe { xfree(keep_msg.get().cast()) };
        keep_msg.set(ptr::null_mut());
        need_fileinfo.set(false);
    }
    if need_highlight_changed.get() {
        unsafe { highlight_changed() };
    }
    if need_clr_eos.get() || (p_ch.get() == 0 && redrawing_cmdline.get()) {
        // Halfway an `:echo` and getting an (error) message: clear any
        // text the command left.
        need_clr_eos.set(false);
        msg_clr_eos();
    }

    // With 'cmdheight' 0 the first line of msg_grid has to be scrolled in
    // over the screen.
    if p_ch.get() == 0 && !ui_has(kUIMessages) && msg_scrolled.get() == 0 {
        msg_grid_validate();
        msg_scroll_up(false, true);
        msg_scrolled.set(msg_scrolled.get() + 1);
        cmdline_row.set(Rows.get() - 1);
    }

    if msg_scroll.get() == 0 && full_screen.get() {
        // Overwrite the last message.
        msg_row.set(cmdline_row.get());
        msg_col.set(0);
    } else if (msg_didout.get() || p_ch.get() == 0) && !ui_has(kUIMessages) {
        // Start the message on the next line.
        if p_ch.get() == 0 && !msg_didout.get() && msg_use_printf() != 0 {
            msg_bytes_to_grid(b"\n", 0, false);
        } else {
            msg_putchar(NL);
        }
        did_return = true;
        cmdline_row.set(msg_row.get());
    }
    if !msg_didany.get() || lines_left.get() < 0 {
        msg_starthere();
    }
    if msg_silent.get() == 0 {
        // No output on the current line yet.
        msg_didout.set(false);
    }
    if ui_has(kUIMessages) {
        msg_ext_ui_flush();
    }
    // When redirecting, may need to start a new line.
    if !did_return {
        redir_write(b"\n");
    }
}

/// Note that the current message position is where messages start.
pub fn msg_starthere() {
    lines_left.set(cmdline_row.get());
    msg_didany.set(false);
}

/// Show a string at `msg_row`/`msg_col`, advancing them past it.
pub fn msg_str(text: &CStr) {
    msg_str_hl(text, 0, false)
}

/// [`msg_str`] in the title highlight.
pub fn msg_title(text: &CStr) {
    // An `ext_messages` UI lays messages out itself, so a leading newline
    // is noise there.
    let bytes = text.to_bytes();
    let skip = usize::from(ui_has(kUIMessages) && bytes.first() == Some(&b'\n'));
    msg_bytes(&bytes[skip..], HLF_T, false)
}

/// [`msg_bytes`] over a NUL-terminated string.
pub fn msg_str_hl(text: &CStr, hl_id: c_int, hist: bool) {
    msg_bytes(text.to_bytes(), hl_id, hist)
}

/// Show `bytes` as a whole message.
///
/// Everything displayed goes through here: this is where redirection is fed,
/// `:silent` is honoured, the history entry is made, and the choice between
/// the grid and plain `stderr` is taken.
///
/// An *empty* `bytes` is an empty message, which an `ext_messages` UI is told
/// about — that is what `:echo ""` produces. Where an empty slice means
/// "nothing to add" rather than "a message with no text", the caller wants
/// [`msg_part`] instead.
pub fn msg_bytes(bytes: &[u8], hl_id: c_int, hist: bool) {
    put_bytes(bytes, hl_id, hist, true)
}

/// [`msg_bytes`] for one piece of a message that is already being built.
///
/// The difference is only what an empty slice means: nothing at all, rather
/// than an empty message. `msg_multiline` splits its text at the control
/// characters that need handling of their own, and two of them in a row leave
/// an empty piece between them; upstream distinguished the two cases by
/// reading the byte under the pointer, which a slice cannot do.
pub(crate) fn msg_part(bytes: &[u8], hl_id: c_int, hist: bool) {
    put_bytes(bytes, hl_id, hist, false)
}

fn put_bytes(bytes: &[u8], hl_id: c_int, hist: bool, whole_message: bool) {
    debug_assert!(
        !bytes.contains(&0),
        "a NUL is shown as `^@` by the translating half, never put through as itself"
    );

    // If redirection is on, also write to the redirection file.
    redir_write(bytes);

    // Print nothing under `:silent`, or for an empty message.
    if msg_silent.get() != 0 || bytes.is_empty() {
        if bytes.is_empty() && whole_message && ui_has(kUIMessages) {
            msg_ext_ui_flush(); // ensure messages until now are emitted
            ui_call_msg_show(
                String_0::from_cstr(c"empty"),
                EMPTY_ARRAY,
                false,
                false,
                false,
                Object::integer(-1),
                String_0::NULL,
            );
            cmdline_was_last_drawn.set(false);
        }
        return;
    }

    if hist {
        msg_hist_add(bytes, hl_id);
    }

    // Writing to a screen that has already scrolled needs a hit-enter
    // prompt afterwards. Not when only using CR to move the cursor.
    let overflow = !ui_has(kUIMessages) && msg_scrolled.get() > c_int::from(p_ch.get() == 0);
    if overflow && !msg_scrolled_ign.get() && bytes != b"\r" {
        need_wait_return.set(true);
    }
    msg_didany.set(true); // remember that something was output

    // With no valid screen, use stderr so error messages are still seen.
    // A headless process that nonetheless has a grid (`--headless` with a
    // UI attached) gets both.
    if msg_use_printf() != 0 {
        let saved_msg_col = msg_col.get();
        msg_bytes_to_stdio(bytes);
        if headless_mode.get() {
            msg_col.set(saved_msg_col);
        }
    }
    if msg_use_printf() == 0 || (headless_mode.get() && default_grid_ref().is_allocated()) {
        msg_bytes_to_grid(bytes, hl_id, false);
    }

    need_fileinfo.set(false);
}

/// The `ext_messages` half of [`msg_bytes_to_grid`].
///
/// Nothing is drawn: the text joins the pending chunk and the UI lays it
/// out. Only the message column has to be kept, because the code that
/// decides whether a newline is owed still reads it.
fn msg_bytes_to_ui(bytes: &[u8], hl_id: c_int, attr: c_int) {
    if attr as ScreenAttr != msg_ext_last_attr.get() {
        // Colour changed: end the chunk and start another.
        msg_ext_emit_chunk();
        msg_ext_last_attr.set(attr as ScreenAttr);
        msg_ext_last_hl_id.set(hl_id);
    }
    msg_ext_last_chunk.with_mut(|chunk| chunk.extend_from_slice(bytes));

    // The message column is whatever follows the last newline.
    let tail = match bytes.iter().rposition(|&byte| byte == b'\n') {
        Some(at) => {
            msg_col.set(0);
            &bytes[at + 1..]
        }
        None => bytes,
    };
    // SAFETY: options exist by the time a message is shown.
    let cells = unsafe { string_cells(tail) };
    msg_col.set(msg_col.get() + c_int::try_from(cells).unwrap_or(c_int::MAX));
}

/// The display half of [`msg_bytes`].
///
/// Walks the text a character at a time, filling grid lines and scrolling the
/// message grid when it reaches the bottom of the screen — which is where the
/// `'more'` pager and the hit-enter prompt come from. Under `ext_messages`
/// nothing is drawn at all: the text is appended to the pending chunk and the
/// UI is left to lay it out.
///
/// `recurse` is set when the scrollback is being redisplayed, and suppresses
/// both the scrollback capture and the pager — the text is already stored and
/// the pager is what is asking for it.
pub(crate) fn msg_bytes_to_grid(bytes: &[u8], hl_id: c_int, recurse: bool) {
    let attr = if hl_id != 0 {
        // SAFETY: a highlight id is looked up in the group table, which
        // exists by the time anything is shown.
        unsafe { syn_id2attr(hl_id) }
    } else {
        0
    };
    did_wait_return.set(false);

    if ui_has(kUIMessages) {
        msg_bytes_to_ui(bytes, hl_id, attr);
        return;
    }

    let print_attr =
        // SAFETY: `hl_attr_active` points at the active attribute table.
        unsafe { hl_combine_attr(*hl_attr_active.get().offset(HLF_MSG as isize), attr) };
    msg_grid_validate();
    cmdline_was_last_drawn.set(redrawing_cmdline.get());

    // The text being shown, which is not always the caller's: the pager can
    // jump ahead to a dialog's buttons, and those are a string of their own.
    let mut text = bytes;
    let mut at = 0;
    // The scrollback copy runs one chunk behind the cursor: `stored` is how
    // much of `text` has been captured, `sb_col` the column the run it holds
    // started at.
    let mut stored = 0;
    let mut sb_col = msg_col.get();
    let store = |run: &[u8], sb_col: &mut c_int, finish: bool| {
        if p_more.get() != 0 && !recurse {
            store_sb_text(run, hl_id, sb_col, finish);
        }
    };

    // The row `grid_line_start` was last called for, or -1 when no line is
    // open. Messages want their own private line buffer; until then this
    // stands in for one.
    let mut open_row = -1;
    loop {
        if msg_col.get() >= Columns.get() {
            store(&text[stored..at], &mut sb_col, true);
            stored = at;
            if msg_no_more.get() && lines_left.get() == 0 {
                break;
            }
            msg_col.set(0);
            msg_row.set(msg_row.get() + 1);
            msg_didout.set(false);
        }

        if msg_row.get() >= Rows.get() {
            msg_row.set(Rows.get() - 1);
            // No pager and no room left: truncate here.
            if msg_no_more.get() && lines_left.get() == 0 {
                break;
            }
            if !recurse {
                if open_row >= 0 {
                    msg_line_flush();
                    open_row = -1;
                }
                msg_scroll_up(true, false);
                inc_msg_scrolled();
                need_wait_return.set(true); // may need wait_return() in main()
                redraw_cmdline.set(true);
                if cmdline_row.get() > 0 && !exmode_active.get() {
                    cmdline_row.set(cmdline_row.get() - 1);
                }
                if lines_left.get() > 0 {
                    lines_left.set(lines_left.get() - 1);
                }
                // Screen full and 'more' set: wait for a character.
                if p_more.get() != 0
                    && lines_left.get() == 0
                    && State.get() != MODE_HITRETURN
                    && !msg_no_more.get()
                    && !exmode_active.get()
                {
                    // SAFETY: main-thread editor call.
                    if do_more_prompt(NUL) {
                        // The pager jumped ahead to the dialog buttons, so
                        // the rest of the caller's text is not shown and
                        // nothing of it is left to store.
                        // SAFETY: the prompt only answers true while a
                        // dialog is up, and a dialog owns its button string.
                        text = unsafe { cstr::bytes_at(confirm_buttons.get()) };
                        at = 0;
                        stored = 0;
                    }
                    if quit_more.get() {
                        return;
                    }
                }
            }
        }

        if at >= text.len() {
            break;
        }

        let byte = text[at];
        if msg_row.get() != open_row && (byte >= 0x20 || c_int::from(byte) == TAB) {
            if open_row >= 0 {
                msg_line_flush();
            }
            // SAFETY: main-thread editor call.
            unsafe { grid_line_start(msg_grid_view(), msg_row.get()) };
            open_row = msg_row.get();
        }

        if byte >= 0x20 {
            // Printable character.
            // SAFETY: options exist by the time a message is shown.
            let mut cw = unsafe { cells_at(&text[at..]) };
            // Composing characters past the end of the text are left out.
            let len = cluster_len(&text[at..]);
            if cw > 1 && msg_col.get() == Columns.get() - 1 {
                // Doesn't fit: fill the last column with a highlighted '>'
                // and let the wrap put the character on the next line.
                // SAFETY: `hl_attr_active` points at the active table.
                let at = unsafe { *hl_attr_active.get().offset(HLF_AT as isize) };
                // SAFETY: a line is open and the literal is one byte.
                unsafe { grid_line_puts(msg_col.get(), c">".as_ptr(), 1, at) };
                cw = 1;
            } else {
                // SAFETY: a line is open and `len` bytes of `text` follow
                // `at`.
                unsafe {
                    grid_line_puts(
                        msg_col.get(),
                        text[at..].as_ptr().cast(),
                        len as c_int,
                        print_attr,
                    )
                };
                at += len;
            }
            msg_didout.set(true); // remember that the line is not empty
            msg_col.set(msg_col.get() + cw);
            continue;
        }

        at += 1;
        match c_int::from(byte) {
            NL => {
                msg_didout.set(false); // remember that the line is empty
                msg_col.set(0);
                msg_row.set(msg_row.get() + 1);
                store(&text[stored..at], &mut sb_col, true);
                stored = at;
            }
            CAR => msg_col.set(0),
            BS => {
                if msg_col.get() != 0 {
                    msg_col.set(msg_col.get() - 1);
                }
            }
            TAB => {
                // Translate a tab into spaces, up to the next multiple of
                // eight or the end of the line.
                loop {
                    // SAFETY: a line is open and the literal is one byte.
                    unsafe { grid_line_puts(msg_col.get(), c" ".as_ptr(), 1, print_attr) };
                    msg_col.set(msg_col.get() + 1);
                    if msg_col.get() == Columns.get() || msg_col.get() & 7 == 0 {
                        break;
                    }
                }
            }
            // SAFETY: main-thread editor call.
            BELL => unsafe { vim_beep(kOptBoFlagShell as c_uint) },
            _ => {}
        }
    }

    if open_row >= 0 {
        msg_line_flush();
    }
    msg_cursor_goto(msg_row.get(), msg_col.get());
    store(&text[stored..at], &mut sb_col, false);
    msg_check();
}

/// Whether `:filter pattern` was used and `msg` does not match it.
pub(crate) fn message_filtered(msg: &CStr) -> bool {
    // SAFETY: a `CStr` is a valid C string, which is the whole contract.
    unsafe { cmdmod_filters_out(msg.as_ptr()) }
}

/// Whether messages should be printed to stdout/stderr rather than drawn:
/// batch mode (`-es`/`-Es`/`-l`), or no UI and not embedded.
pub fn msg_use_printf() -> c_int {
    c_int::from(!embedded_mode.get() && ui_active() == 0 && !ui_has(kUIMessages))
}

/// Print a message when there is no valid screen.
///
/// Also keeps `msg_col`/`msg_didout` roughly in step, so that the code that
/// decides whether a newline is needed still works with no grid to measure.
pub(crate) fn msg_bytes_to_stdio(bytes: &[u8]) {
    // `vim.on_print` takes the whole message instead, if it is set.
    // SAFETY: the cell holds a live callback.
    if unsafe { &*on_print_cb() }.is_set() {
        // The callback wants a C string, and `bytes` is a span of one; the
        // copy is what gives it a terminator of its own. Upstream handed
        // over the *pointer* instead, so a caller that asked for a prefix
        // of a longer string had the whole of it printed.
        let text = cstr::owned(bytes);
        // The frame names the copy above, which this frame frees.
        let argv = CallFrame::naming([TypVal::String(text.as_ptr().cast_mut())]);
        let mut rettv = TV_INITIAL_VALUE;
        // SAFETY: one argument, and `rettv` is a live unset value.
        unsafe { callback_call(on_print_cb(), argv.args(), &mut rettv) };
        tv_clear(&mut rettv);
        return;
    }

    let mut at = 0;
    while at < bytes.len() && bytes[at] != 0 {
        let rest = &bytes[at..];
        let len = char_len(rest);
        if !(silent_mode.get() && p_verbose.get() == 0) {
            // One character, with NL translated to CR NL.
            let mut buf = [0u8; 7];
            let mut used = 0;
            if rest[0] == b'\n' && !info_message.get() && !silent_mode.get() && !headless_mode.get()
            {
                buf[used] = b'\r';
                used += 1;
            }
            buf[used..used + len].copy_from_slice(&rest[..len]);
            let text = cstr::in_bytes(&buf);
            if info_message.get() {
                // SAFETY: a `%s` format and one NUL-terminated string.
                unsafe { printf(c"%s".as_ptr(), text.as_ptr()) };
            } else {
                // SAFETY: as above.
                unsafe { fprintf(stderr, c"%s".as_ptr(), text.as_ptr()) };
            }
        }

        // Primitive way to compute the current column.
        if rest[0] == b'\r' || rest[0] == b'\n' {
            msg_col.set(0);
            msg_didout.set(false);
        } else {
            // SAFETY: options exist by the time a message is shown.
            msg_col.set(msg_col.get() + unsafe { utf_char2cells(char_at(rest)) });
            msg_didout.set(true);
        }
        at += len;
    }
}

/// Finish putting a message on the screen, prompting if it did not fit.
///
/// Answers false when [`wait_return`] was called.
pub fn msg_end() -> bool {
    // A message larger than the window, or one that ran into the ruler,
    // means the window has to be redrawn -- but not while abandoning the
    // file or editing the command line.
    if !exiting.get() && need_wait_return.get() && State.get() & MODE_CMDLINE == 0 {
        wait_return(0);
        return false;
    }
    // NOTE: ui_flush() used to be called here. It had to be removed, as it
    // inhibited substantial performance improvements. Relevant callers are
    // assumed to invoke ui_flush() before going into CPU busywork, or
    // restricted event processing after displaying a message to the user.
    msg_ext_ui_flush();
    true
}

/// If the message ran into the shown command or the ruler, a hit-enter prompt
/// and a redraw are owed.
pub fn msg_check() {
    if ui_has(kUIMessages) {
        return;
    }
    if msg_row.get() == Rows.get() - 1 && msg_col.get() >= sc_col.get() {
        need_wait_return.set(true);
        redraw_cmdline.set(true);
    }
}

/// Pad with spaces up to column `col`.
pub fn msg_advance(col: c_int) {
    if msg_silent.get() != 0 {
        // Nothing to advance to; keep the column for redirection, which
        // may fill it up later.
        msg_col.set(col);
        return;
    }
    let col = col.min(Columns.get() - 1); // not enough room
    while msg_col.get() < col {
        msg_putchar(b' ' as c_int);
    }
}
