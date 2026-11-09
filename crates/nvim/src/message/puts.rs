//! `msg_str` and its display half: text onto the message grid.
//!
//! [`msg_bytes`] is the funnel every message eventually reaches; it feeds
//! the redirection sinks and then [`msg_bytes_to_grid`], which lays the text
//! out cell by cell, scrolls when it runs off the bottom and raises the pager
//! when `'more'` says to. [`msg_bytes_to_stdio`] is the same job for a process
//! with no UI at all.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::ascii::NUL;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::ex_docmd::cmdmod_filters_out;
use crate::grid::default_grid_ref;
use crate::mbyte::{cells_at, char_at, char_len, cluster_len, string_cells};
use crate::os::cshim::{write_c_stderr, write_c_stdout};
use crate::types::Callback;
use core::ffi::{c_int, c_uint};
use std::borrow::Cow;

/// Run `f` on the `ON_PRINT` callback an RPC client installed. `f` must
/// not run user code: the callback is borrowed for its length.
pub(crate) fn with_on_print<R>(f: impl FnOnce(&mut Callback) -> R) -> R {
    ON_PRINT.with_mut(f)
}

/// Start putting a message on the screen.
///
/// Decides *where* the message goes: over the last one, or on a fresh line
/// below it, scrolling if there is no room.
pub fn msg_start() {
    let mut did_return = false;
    MSG_ROW.set(MSG_ROW.get().max(CMDLINE_ROW.get()));

    if MSG_SILENT.get() == 0 {
        // Don't display the old message now.
        KEEP_MSG.set(None);
        NEED_FILEINFO.set(false);
    }
    if NEED_HIGHLIGHT_CHANGED.get() {
        highlight_changed();
    }
    if NEED_CLR_EOS.get() || (p_ch() == 0 && REDRAWING_CMDLINE.get()) {
        // Halfway an `:echo` and getting an (error) message: clear any
        // text the command left.
        NEED_CLR_EOS.set(false);
        msg_clr_eos();
    }

    // With 'cmdheight' 0 the first line of msg_grid has to be scrolled in
    // over the screen.
    if p_ch() == 0 && !ui_has(UI_MESSAGES) && MSG_SCROLLED.get() == 0 {
        msg_grid_validate();
        msg_scroll_up(false, true);
        MSG_SCROLLED.set(MSG_SCROLLED.get() + 1);
        CMDLINE_ROW.set(ROWS.get() - 1);
    }

    if MSG_SCROLL.get() == 0 && FULL_SCREEN.get() {
        // Overwrite the last message.
        MSG_ROW.set(CMDLINE_ROW.get());
        MSG_COL.set(0);
    } else if (MSG_DIDOUT.get() || p_ch() == 0) && !ui_has(UI_MESSAGES) {
        // Start the message on the next line.
        if p_ch() == 0 && !MSG_DIDOUT.get() && msg_use_printf() != 0 {
            msg_bytes_to_grid(b"\n", 0, false);
        } else {
            msg_putchar(c_int::from(NL));
        }
        did_return = true;
        CMDLINE_ROW.set(MSG_ROW.get());
    }
    if !MSG_DIDANY.get() || LINES_LEFT.get() < 0 {
        msg_starthere();
    }
    if MSG_SILENT.get() == 0 {
        // No output on the current line yet.
        MSG_DIDOUT.set(false);
    }
    if ui_has(UI_MESSAGES) {
        msg_ext_ui_flush();
    }
    // When redirecting, may need to start a new line.
    if !did_return {
        redir_write(b"\n");
    }
}

/// Note that the current message position is where messages start.
pub fn msg_starthere() {
    LINES_LEFT.set(CMDLINE_ROW.get());
    MSG_DIDANY.set(false);
}

/// Show a string at `MSG_ROW`/`MSG_COL`, advancing them past it.
pub fn msg_str(text: &CStr) {
    msg_str_hl(text, 0, false)
}

/// [`msg_str`] in the title highlight.
pub fn msg_title(text: &CStr) {
    // An `ext_messages` UI lays messages out itself, so a leading newline
    // is noise there.
    let bytes = text.to_bytes();
    let skip = usize::from(ui_has(UI_MESSAGES) && bytes.first() == Some(&b'\n'));
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
    if MSG_SILENT.get() != 0 || bytes.is_empty() {
        if bytes.is_empty() && whole_message && ui_has(UI_MESSAGES) {
            msg_ext_ui_flush(); // ensure messages until now are emitted
            ui_call_msg_show(
                ApiString::from_cstr(c"empty"),
                Array::new(),
                false,
                false,
                false,
                Object::integer(-1),
                ApiString::NULL,
            );
            CMDLINE_WAS_LAST_DRAWN.set(false);
        }
        return;
    }

    if hist {
        msg_hist_add(bytes, hl_id);
    }

    // Writing to a screen that has already scrolled needs a hit-enter
    // prompt afterwards. Not when only using CR to move the cursor.
    let overflow = !ui_has(UI_MESSAGES) && MSG_SCROLLED.get() > c_int::from(p_ch() == 0);
    if overflow && !MSG_SCROLLED_IGN.get() && bytes != b"\r" {
        NEED_WAIT_RETURN.set(true);
    }
    MSG_DIDANY.set(true); // remember that something was output

    // With no valid screen, use stderr so error messages are still seen.
    // A headless process that nonetheless has a grid (`--headless` with a
    // UI attached) gets both.
    if msg_use_printf() != 0 {
        let saved_msg_col = MSG_COL.get();
        msg_bytes_to_stdio(bytes);
        if HEADLESS_MODE.get() {
            MSG_COL.set(saved_msg_col);
        }
    }
    if msg_use_printf() == 0 || (HEADLESS_MODE.get() && default_grid_ref().is_allocated()) {
        msg_bytes_to_grid(bytes, hl_id, false);
    }

    NEED_FILEINFO.set(false);
}

/// The `ext_messages` half of [`msg_bytes_to_grid`].
///
/// Nothing is drawn: the text joins the pending chunk and the UI lays it
/// out. Only the message column has to be kept, because the code that
/// decides whether a newline is owed still reads it.
fn msg_bytes_to_ui(bytes: &[u8], hl_id: c_int, attr: c_int) {
    if attr as ScreenAttr != MSG_EXT_LAST_ATTR.get() {
        // Colour changed: end the chunk and start another.
        msg_ext_emit_chunk();
        MSG_EXT_LAST_ATTR.set(attr as ScreenAttr);
        MSG_EXT_LAST_HL_ID.set(hl_id);
    }
    MSG_EXT_LAST_CHUNK.update(|chunk| chunk.extend_from_slice(bytes));

    // The message column is whatever follows the last newline.
    let tail = match bytes.iter().rposition(|&byte| byte == b'\n') {
        Some(at) => {
            MSG_COL.set(0);
            &bytes[at + 1..]
        }
        None => bytes,
    };
    let cells = string_cells(tail);
    MSG_COL.set(MSG_COL.get() + c_int::try_from(cells).unwrap_or(c_int::MAX));
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
    let attr = if hl_id != 0 { syn_id2attr(hl_id) } else { 0 };
    DID_WAIT_RETURN.set(false);

    if ui_has(UI_MESSAGES) {
        msg_bytes_to_ui(bytes, hl_id, attr);
        return;
    }

    let print_attr = hl_combine_attr(hl_attr(HLF_MSG as c_int), attr);
    msg_grid_validate();
    CMDLINE_WAS_LAST_DRAWN.set(REDRAWING_CMDLINE.get());

    // The text being shown, which is not always the caller's: the pager can
    // jump ahead to a dialog's buttons, and those are a string of their own.
    let mut text = Cow::Borrowed(bytes);
    let mut at = 0;
    // The scrollback copy runs one chunk behind the cursor: `stored` is how
    // much of `text` has been captured, `sb_col` the column the run it holds
    // started at.
    let mut stored = 0;
    let mut sb_col = MSG_COL.get();
    let store = |run: &[u8], sb_col: &mut c_int, finish: bool| {
        if p_more() && !recurse {
            store_sb_text(run, hl_id, sb_col, finish);
        }
    };

    // The row `grid_line_start` was last called for, or -1 when no line is
    // open. Messages want their own private line buffer; until then this
    // stands in for one.
    let mut open_row = -1;
    loop {
        if MSG_COL.get() >= COLUMNS.get() {
            store(&text[stored..at], &mut sb_col, true);
            stored = at;
            if MSG_NO_MORE.get() && LINES_LEFT.get() == 0 {
                break;
            }
            MSG_COL.set(0);
            MSG_ROW.set(MSG_ROW.get() + 1);
            MSG_DIDOUT.set(false);
        }

        if MSG_ROW.get() >= ROWS.get() {
            MSG_ROW.set(ROWS.get() - 1);
            // No pager and no room left: truncate here.
            if MSG_NO_MORE.get() && LINES_LEFT.get() == 0 {
                break;
            }
            if !recurse {
                if open_row >= 0 {
                    msg_line_flush();
                    open_row = -1;
                }
                msg_scroll_up(true, false);
                inc_msg_scrolled();
                NEED_WAIT_RETURN.set(true); // may need wait_return() in main()
                REDRAW_CMDLINE.set(true);
                if CMDLINE_ROW.get() > 0 && !EXMODE_ACTIVE.get() {
                    CMDLINE_ROW.set(CMDLINE_ROW.get() - 1);
                }
                if LINES_LEFT.get() > 0 {
                    LINES_LEFT.set(LINES_LEFT.get() - 1);
                }
                // Screen full and 'more' set: wait for a character.
                if p_more()
                    && LINES_LEFT.get() == 0
                    && STATE.get() != MODE_HITRETURN
                    && !MSG_NO_MORE.get()
                    && !EXMODE_ACTIVE.get()
                {
                    if do_more_prompt(NUL) {
                        // The pager jumped ahead to the dialog buttons, so
                        // the rest of the caller's text is not shown and
                        // nothing of it is left to store.
                        // A copy: the rest of the loop can redraw.
                        let buttons = CONFIRM_BUTTONS.with(Clone::clone).unwrap_or_default();
                        text = Cow::Owned(buttons.as_cstr().to_bytes().to_vec());
                        at = 0;
                        stored = 0;
                    }
                    if QUIT_MORE.get() {
                        return;
                    }
                }
            }
        }

        if at >= text.len() {
            break;
        }

        let byte = text[at];
        if MSG_ROW.get() != open_row && (byte >= 0x20 || byte == TAB) {
            if open_row >= 0 {
                msg_line_flush();
            }
            grid_line_start(msg_grid_view(), MSG_ROW.get());
            open_row = MSG_ROW.get();
        }

        if byte >= 0x20 {
            // Printable character.
            let mut cw = cells_at(&text[at..]);
            // Composing characters past the end of the text are left out.
            let len = cluster_len(&text[at..]);
            if cw > 1 && MSG_COL.get() == COLUMNS.get() - 1 {
                // Doesn't fit: fill the last column with a highlighted '>'
                // and let the wrap put the character on the next line.
                grid_line_puts(MSG_COL.get(), b">", hl_attr(HLF_AT as c_int));
                cw = 1;
            } else {
                grid_line_puts(MSG_COL.get(), &text[at..at + len], print_attr);
                at += len;
            }
            MSG_DIDOUT.set(true); // remember that the line is not empty
            MSG_COL.set(MSG_COL.get() + cw);
            continue;
        }

        at += 1;
        match byte {
            NL => {
                MSG_DIDOUT.set(false); // remember that the line is empty
                MSG_COL.set(0);
                MSG_ROW.set(MSG_ROW.get() + 1);
                store(&text[stored..at], &mut sb_col, true);
                stored = at;
            }
            CAR => MSG_COL.set(0),
            BS => {
                if MSG_COL.get() != 0 {
                    MSG_COL.set(MSG_COL.get() - 1);
                }
            }
            TAB => {
                // Translate a tab into spaces, up to the next multiple of
                // eight or the end of the line.
                loop {
                    grid_line_puts(MSG_COL.get(), b" ", print_attr);
                    MSG_COL.set(MSG_COL.get() + 1);
                    if MSG_COL.get() == COLUMNS.get() || MSG_COL.get() & 7 == 0 {
                        break;
                    }
                }
            }
            BELL => vim_beep(OPT_BO_FLAG_SHELL as c_uint),
            _ => {}
        }
    }

    if open_row >= 0 {
        msg_line_flush();
    }
    msg_cursor_goto(MSG_ROW.get(), MSG_COL.get());
    store(&text[stored..at], &mut sb_col, false);
    msg_check();
}

/// Whether `:filter pattern` was used and `msg` does not match it.
pub(crate) fn message_filtered(msg: &CStr) -> bool {
    cmdmod_filters_out(msg)
}

/// Whether messages should be printed to stdout/stderr rather than drawn:
/// batch mode (`-es`/`-Es`/`-l`), or no UI and not embedded.
pub fn msg_use_printf() -> c_int {
    c_int::from(!EMBEDDED_MODE.get() && ui_active() == 0 && !ui_has(UI_MESSAGES))
}

/// Print a message when there is no valid screen.
///
/// Also keeps `MSG_COL`/`MSG_DIDOUT` roughly in step, so that the code that
/// decides whether a newline is needed still works with no grid to measure.
pub(crate) fn msg_bytes_to_stdio(bytes: &[u8]) {
    // `vim.on_print` takes the whole message instead, if it is set. A
    // second reference, so the call holds no borrow of the cell.
    if let Some(mut callback) = ON_PRINT.with(|cb| cb.is_set().then(|| cb.duplicate())) {
        // The callback wants a C string, and `bytes` is a span of one; the
        // copy is what gives it a terminator of its own. Upstream handed
        // over the *pointer* instead, so a caller that asked for a prefix
        // of a longer string had the whole of it printed.
        // The array owns the copy and releases it.
        let argv = [TypVal::string_from(bytes)];
        let mut rettv = TV_INITIAL_VALUE;
        let _ = callback_call(&callback, &argv, &mut rettv);
        tv_clear(&mut rettv);
        callback.clear();
        return;
    }

    let mut at = 0;
    while at < bytes.len() && bytes[at] != 0 {
        let rest = &bytes[at..];
        let len = char_len(rest);
        if !(SILENT_MODE.get() && p_verbose() == 0) {
            // One character, with NL translated to CR NL.
            let mut buf = [0u8; 7];
            let mut used = 0;
            if rest[0] == b'\n' && !INFO_MESSAGE.get() && !SILENT_MODE.get() && !HEADLESS_MODE.get()
            {
                buf[used] = b'\r';
                used += 1;
            }
            buf[used..used + len].copy_from_slice(&rest[..len]);
            // Up to a NUL, as the `%s` this was printed through read it.
            let text = &buf[..used + len];
            let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
            if INFO_MESSAGE.get() {
                write_c_stdout(text);
            } else {
                write_c_stderr(text);
            }
        }

        // Primitive way to compute the current column.
        if rest[0] == b'\r' || rest[0] == b'\n' {
            MSG_COL.set(0);
            MSG_DIDOUT.set(false);
        } else {
            MSG_COL.set(MSG_COL.get() + utf_char2cells(char_at(rest)));
            MSG_DIDOUT.set(true);
        }
        at += len;
    }
}

/// Finish putting a message on the screen, prompting if it did not fit.
///
/// Answers false when [`wait_return`] was called.
pub(crate) fn msg_end() -> bool {
    // A message larger than the window, or one that ran into the ruler,
    // means the window has to be redrawn -- but not while abandoning the
    // file or editing the command line.
    if !EXITING.get() && NEED_WAIT_RETURN.get() && STATE.get() & MODE_CMDLINE == 0 {
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
    if ui_has(UI_MESSAGES) {
        return;
    }
    if MSG_ROW.get() == ROWS.get() - 1 && MSG_COL.get() >= SC_COL.get() {
        NEED_WAIT_RETURN.set(true);
        REDRAW_CMDLINE.set(true);
    }
}

/// Pad with spaces up to column `col`.
pub fn msg_advance(col: c_int) {
    if MSG_SILENT.get() != 0 {
        // Nothing to advance to; keep the column for redirection, which
        // may fill it up later.
        MSG_COL.set(col);
        return;
    }
    let col = col.min(COLUMNS.get() - 1); // not enough room
    while MSG_COL.get() < col {
        msg_putchar(c_int::from(b' '));
    }
}
