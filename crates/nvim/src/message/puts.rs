//! `msg_puts` and its display half: text onto the message grid.
//!
//! [`msg_puts_len`] is the funnel every message eventually reaches; it feeds
//! the redirection sinks and then [`msg_puts_display`], which lays the text
//! out cell by cell, scrolls when it runs off the bottom and raises the pager
//! when `'more'` says to. [`msg_puts_printf`] is the same job for a process
//! with no UI at all.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::cmdmod_filters_out;
use crate::grid::default_grid_ref;
use crate::types::builders::static_cstring;
use crate::types::{NUL, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// C's `ARRAY_DICT_INIT`: empty, and owning nothing.
const EMPTY_ARRAY: Array = Array {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};

/// Start putting a message on the screen.
///
/// Decides *where* the message goes: over the last one, or on a fresh line
/// below it, scrolling if there is no room.
pub unsafe fn msg_start() {
    unsafe {
        let mut did_return = false;
        msg_row.set(msg_row.get().max(cmdline_row.get()));

        if msg_silent.get() == 0 {
            // Don't display the old message now.
            xfree(keep_msg.get().cast());
            keep_msg.set(ptr::null_mut());
            need_fileinfo.set(false);
        }
        if need_highlight_changed.get() {
            highlight_changed();
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
                msg_puts_display(c"\n".as_ptr(), 1, 0, false);
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
            redir_write(c"\n".as_ptr(), 1);
        }
    }
}

/// Note that the current message position is where messages start.
pub unsafe fn msg_starthere() {
    lines_left.set(cmdline_row.get());
    msg_didany.set(false);
}

/// Show a string at `msg_row`/`msg_col`, advancing them past it.
pub unsafe fn msg_puts(s: *const c_char) {
    unsafe { msg_puts_hl(s, 0, false) }
}

/// [`msg_puts`] in the title highlight.
pub unsafe fn msg_puts_title(s: *const c_char) {
    unsafe {
        // An `ext_messages` UI lays messages out itself, so a leading newline
        // is noise there.
        let s = s.add(usize::from(ui_has(kUIMessages) && *s == b'\n' as c_char));
        msg_puts_hl(s, HLF_T, false)
    }
}

/// [`msg_puts_len`] over a NUL-terminated string.
pub unsafe fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool) {
    unsafe { msg_puts_len(s, -1, hl_id, hist) }
}

/// Show `len` bytes of `str` — or, when `len` is negative, up to its NUL.
///
/// Everything displayed goes through here: this is where redirection is fed,
/// `:silent` is honoured, the history entry is made, and the choice between
/// the grid and plain `stderr` is taken.
///
/// # Safety
/// `str` must point at `len` readable bytes, or at a NUL-terminated string
/// when `len` is negative.
pub unsafe fn msg_puts_len(str: *const c_char, len: ptrdiff_t, hl_id: c_int, hist: bool) {
    unsafe {
        debug_assert!(len < 0 || memchr(str.cast(), 0, len as size_t).is_null());

        // If redirection is on, also write to the redirection file.
        redir_write(str, len);

        // Print nothing under `:silent`, or for an empty message.
        if msg_silent.get() != 0 || *str == 0 {
            if *str == 0 && ui_has(kUIMessages) {
                msg_ext_ui_flush(); // ensure messages until now are emitted
                ui_call_msg_show(
                    static_cstring(c"empty"),
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
            msg_hist_add(str, len as c_int, hl_id);
        }

        // Writing to a screen that has already scrolled needs a hit-enter
        // prompt afterwards. Not when only using CR to move the cursor.
        let overflow = !ui_has(kUIMessages) && msg_scrolled.get() > c_int::from(p_ch.get() == 0);
        if overflow && !msg_scrolled_ign.get() && strcmp(str, c"\r".as_ptr()) != 0 {
            need_wait_return.set(true);
        }
        msg_didany.set(true); // remember that something was output

        // With no valid screen, use stderr so error messages are still seen.
        // A headless process that nonetheless has a grid (`--headless` with a
        // UI attached) gets both.
        if msg_use_printf() != 0 {
            let saved_msg_col = msg_col.get();
            msg_puts_printf(str, len);
            if headless_mode.get() {
                msg_col.set(saved_msg_col);
            }
        }
        if msg_use_printf() == 0 || (headless_mode.get() && default_grid_ref().is_allocated()) {
            msg_puts_display(str, len as c_int, hl_id, false);
        }

        need_fileinfo.set(false);
    }
}

/// The display half of [`msg_puts_len`].
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
///
/// # Safety
/// `str` must point at `maxlen` readable bytes, or at a NUL-terminated string
/// when `maxlen` is negative.
pub(crate) unsafe fn msg_puts_display(
    str: *const c_char,
    mut maxlen: c_int,
    hl_id: c_int,
    recurse: bool,
) {
    unsafe {
        let mut s = str;
        let attr = if hl_id != 0 { syn_id2attr(hl_id) } else { 0 };
        did_wait_return.set(false);

        if ui_has(kUIMessages) {
            if attr as sattr_T != msg_ext_last_attr.get() {
                // Colour changed: end the chunk and start another.
                msg_ext_emit_chunk();
                msg_ext_last_attr.set(attr as sattr_T);
                msg_ext_last_hl_id.set(hl_id);
            }
            let len = if maxlen < 0 {
                strlen(str)
            } else {
                strnlen(str, maxlen as size_t)
            };
            ga_concat_len(msg_ext_last_chunk.ptr(), str, len);

            // The message column is whatever follows the last newline.
            let lastline: *const c_char = xmemrchr(str.cast(), b'\n', len).cast();
            maxlen -= if lastline.is_null() {
                0
            } else {
                lastline.offset_from(str) as c_int
            };
            let tail = if lastline.is_null() {
                str
            } else {
                lastline.add(1)
            };
            let cells = if maxlen < 0 {
                mb_string2cells(tail)
            } else {
                mb_string2cells_len(tail, maxlen as size_t)
            } as c_int;
            msg_col.set(if lastline.is_null() { msg_col.get() } else { 0 } + cells);
            return;
        }

        let print_attr = hl_combine_attr(*hl_attr_active.get().offset(HLF_MSG as isize), attr);
        msg_grid_validate();
        cmdline_was_last_drawn.set(redrawing_cmdline.get());

        // The scrollback copy runs one chunk behind the cursor: `sb_str` is
        // where the un-stored text starts, `sb_col` the column it started at.
        let mut sb_str = str;
        let mut sb_col = msg_col.get();
        let mut store = |sb_str: &mut *const c_char, upto, sb_col: &mut c_int, finish| {
            if p_more.get() != 0 && !recurse {
                store_sb_text(sb_str, upto, hl_id, sb_col, finish);
            }
        };

        // The row `grid_line_start` was last called for, or -1 when no line is
        // open. Messages want their own private line buffer; until then this
        // stands in for one.
        let mut open_row = -1;
        loop {
            if msg_col.get() >= Columns.get() {
                store(&mut sb_str, s, &mut sb_col, 1);
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
                        if do_more_prompt(NUL) {
                            // The pager jumped ahead to the dialog buttons.
                            s = confirm_buttons.get();
                        }
                        if quit_more.get() {
                            return;
                        }
                    }
                }
            }

            let at_end = !(maxlen < 0 || (s.offset_from(str) as c_int) < maxlen) || *s == 0;
            if at_end {
                break;
            }

            let byte = *s as u8;
            if msg_row.get() != open_row && (byte >= 0x20 || byte as c_int == TAB) {
                if open_row >= 0 {
                    msg_line_flush();
                }
                grid_line_start(msg_grid_view(), msg_row.get());
                open_row = msg_row.get();
            }

            if byte >= 0x20 {
                // Printable character.
                let mut cw = utf_ptr2cells(s);
                // Avoid including composing characters past the end.
                let l = if maxlen >= 0 {
                    utfc_ptr2len_len(s, str.offset(maxlen as isize).offset_from(s) as c_int)
                } else {
                    utfc_ptr2len(s)
                };
                if cw > 1 && msg_col.get() == Columns.get() - 1 {
                    // Doesn't fit: fill the last column with a highlighted '>'
                    // and let the wrap put the character on the next line.
                    grid_line_puts(
                        msg_col.get(),
                        c">".as_ptr(),
                        1,
                        *hl_attr_active.get().offset(HLF_AT as isize),
                    );
                    cw = 1;
                } else {
                    grid_line_puts(msg_col.get(), s, l, print_attr);
                    s = s.add(l as usize);
                }
                msg_didout.set(true); // remember that the line is not empty
                msg_col.set(msg_col.get() + cw);
                continue;
            }

            s = s.add(1);
            match byte as c_int {
                NL => {
                    msg_didout.set(false); // remember that the line is empty
                    msg_col.set(0);
                    msg_row.set(msg_row.get() + 1);
                    store(&mut sb_str, s, &mut sb_col, 1);
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
                        grid_line_puts(msg_col.get(), c" ".as_ptr(), 1, print_attr);
                        msg_col.set(msg_col.get() + 1);
                        if msg_col.get() == Columns.get() || msg_col.get() & 7 == 0 {
                            break;
                        }
                    }
                }
                BELL => vim_beep(kOptBoFlagShell as c_uint),
                _ => {}
            }
        }

        if open_row >= 0 {
            msg_line_flush();
        }
        msg_cursor_goto(msg_row.get(), msg_col.get());
        store(&mut sb_str, s, &mut sb_col, 0);
        msg_check();
    }
}

/// Whether `:filter pattern` was used and `msg` does not match it.
///
/// # Safety
/// `msg` is NUL-terminated; main-thread editor call.
pub(crate) unsafe fn message_filtered(msg: *const c_char) -> bool {
    // SAFETY: the caller's contract.
    unsafe { cmdmod_filters_out(msg) }
}

/// Whether messages should be printed to stdout/stderr rather than drawn:
/// batch mode (`-es`/`-Es`/`-l`), or no UI and not embedded.
pub unsafe fn msg_use_printf() -> c_int {
    c_int::from(!embedded_mode.get() && ui_active() == 0 && !ui_has(kUIMessages))
}

/// Print a message when there is no valid screen.
///
/// Also keeps `msg_col`/`msg_didout` roughly in step, so that the code that
/// decides whether a newline is needed still works with no grid to measure.
pub(crate) unsafe fn msg_puts_printf(str: *const c_char, maxlen: ptrdiff_t) {
    unsafe {
        // `vim.on_print` takes the whole message instead, if it is set.
        if (*on_print.ptr()).type_0 != kCallbackNone as c_uint {
            let mut argv = [typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_string: str.cast_mut(),
                },
            }];
            let mut rettv = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            callback_call(on_print.ptr(), 1, argv.as_mut_ptr(), &raw mut rettv);
            tv_clear(&raw mut rettv);
            return;
        }

        let mut s = str;
        while (maxlen < 0 || s.offset_from(str) < maxlen) && *s != 0 {
            let len = utf_ptr2len(s);
            if !(silent_mode.get() && p_verbose.get() == 0) {
                // One character, with NL translated to CR NL.
                let mut buf = [0 as c_char; 7];
                let mut at = 0;
                if *s == b'\n' as c_char
                    && !info_message.get()
                    && !silent_mode.get()
                    && !headless_mode.get()
                {
                    buf[at] = b'\r' as c_char;
                    at += 1;
                }
                ptr::copy_nonoverlapping(s, buf.as_mut_ptr().add(at), len as usize);
                buf[at + len as usize] = 0;
                if info_message.get() {
                    printf(c"%s".as_ptr(), buf.as_ptr());
                } else {
                    fprintf(stderr, c"%s".as_ptr(), buf.as_ptr());
                }
            }

            // Primitive way to compute the current column.
            if *s == b'\r' as c_char || *s == b'\n' as c_char {
                msg_col.set(0);
                msg_didout.set(false);
            } else {
                msg_col.set(msg_col.get() + utf_char2cells(utf_ptr2char(s)));
                msg_didout.set(true);
            }
            s = s.add(len as usize);
        }
    }
}

/// Finish putting a message on the screen, prompting if it did not fit.
///
/// Answers false when [`wait_return`] was called.
pub unsafe fn msg_end() -> bool {
    unsafe {
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
}

/// If the message ran into the shown command or the ruler, a hit-enter prompt
/// and a redraw are owed.
pub unsafe fn msg_check() {
    if ui_has(kUIMessages) {
        return;
    }
    if msg_row.get() == Rows.get() - 1 && msg_col.get() >= sc_col.get() {
        need_wait_return.set(true);
        redraw_cmdline.set(true);
    }
}

/// Pad with spaces up to column `col`.
pub unsafe fn msg_advance(col: c_int) {
    unsafe {
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
}
