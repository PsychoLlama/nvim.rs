//! The mode message on the last line -- `--INSERT--` and friends.
//!
//! [`showmode`] is the whole of it: what to print for the current mode, the
//! `CTRL-X` submode text Insert-mode completion puts there instead, the
//! `recording @q` suffix, and clearing the command line around all of it.
//! [`unshowmode`] and [`clearmode`] take it away again.
//!
//! [`comp_col`] is here because it is the same real estate: it decides how many
//! columns at the right of the last line belong to the ruler and to `'showcmd'`,
//! which is what bounds every message printed there.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;

use super::*;
use crate::normal::{visual_active, visual_mode, visual_select};
use crate::types::{MAXPATHL, NUL, ShmFlag, Vv};

/// Whether to postpone the mode message: not redrawing, or inside a mapping.
///
/// Answering true also sets `redraw_mode`, so it is shown on the next redraw.
pub unsafe fn skip_showmode() -> bool {
    // SAFETY: `char_avail` pumps the input layer on the main thread.
    unsafe {
        // `char_avail` is only asked last: it costs a poll of the input layer,
        // and `redrawing()` may already have paid for one.
        if global_busy.get() != 0
            || msg_silent.get() != 0
            || !redrawing()
            || (char_avail() && !KeyTyped.get())
        {
            redraw_mode.set(true);
            return true;
        }
        false
    }
}

/// Show the current mode and ruler, and answer the length of what was printed.
///
/// `clear_cmdline` asks for the rest of the command line to be cleared;
/// `redraw_mode` asks for the mode to be shown or taken away even when there is
/// none to show.
pub unsafe fn showmode() -> c_int {
    // SAFETY: the message layer on the main thread; every pointer read below is
    // a global the editor keeps live.
    unsafe {
        let mut length = 0;

        // A message that has not been flushed must not become part of this one.
        msg_ext_ui_flush();
        msg_grid_validate();

        let do_mode = p_smd.get() != 0
            && msg_silent.get() == 0
            && (State.get() & MODE_TERMINAL != 0
                || State.get() & MODE_INSERT != 0
                || restart_edit.get() != NUL
                || visual_active());
        let can_show_mode = p_ch.get() != 0 || ui_has(kUIMessages);

        if (do_mode || reg_recording.get() != 0) && can_show_mode {
            if skip_showmode() {
                return 0;
            }

            let nwr_save = need_wait_return.get();

            // Wait a bit before overwriting an important message.
            msg_check_for_delay(false);

            let mut need_clear = clear_cmdline.get();
            if clear_cmdline.get() && cmdline_row.get() < Rows.get() - 1 {
                // Resets `clear_cmdline`, which is why `need_clear` was read
                // first.
                msg_clr_cmdline();
            }

            msg_pos_mode();
            let hl_id = HLF_CM;

            // Truncate rather than scroll when the screen is too narrow for the
            // whole message.
            msg_no_more.set(true);
            let save_lines_left = lines_left.get();
            lines_left.set(0);

            let put = |s: &CStr| msg_puts_hl(s.as_ptr(), hl_id, false);
            let put_translated = |s: &CStr| msg_puts_hl(gettext(s.as_ptr()), hl_id, false);

            if do_mode {
                put(c"--");
                if !edit_submode.get().is_null() && !shortmess(ShmFlag::COMPLETIONMENU) {
                    // CTRL-X in Insert mode. These get long, so they are budgeted
                    // against the room left rather than allowed to wrap; an
                    // external message UI imposes no limit of its own.
                    // `edit_submode_extra` is preferred over `edit_submode`.
                    length = if ui_has(kUIMessages) {
                        INT_MAX
                    } else {
                        (Rows.get() - msg_row.get()) * Columns.get() - 3
                    };
                    if !edit_submode_extra.get().is_null() {
                        length -= vim_strsize(edit_submode_extra.get());
                    }
                    if length > 0 {
                        if !edit_submode_pre.get().is_null() {
                            length -= vim_strsize(edit_submode_pre.get());
                        }
                        if length - vim_strsize(edit_submode.get()) > 0 {
                            if !edit_submode_pre.get().is_null() {
                                msg_puts_hl(edit_submode_pre.get(), hl_id, false);
                            }
                            msg_puts_hl(edit_submode.get(), hl_id, false);
                        }
                        if !edit_submode_extra.get().is_null() {
                            put(c" ");
                            let sub_id = if edit_submode_highl.get() < HLF_COUNT {
                                edit_submode_highl.get()
                            } else {
                                hl_id
                            };
                            msg_puts_hl(edit_submode_extra.get(), sub_id, false);
                        }
                    }
                } else {
                    // The mode itself. Not a `match`: these are tests on
                    // different variables, in upstream's precedence order.
                    if State.get() & MODE_TERMINAL != 0 {
                        put_translated(c" TERMINAL");
                    } else if State.get() & VREPLACE_FLAG != 0 {
                        put_translated(c" VREPLACE");
                    } else if State.get() & REPLACE_FLAG != 0 {
                        put_translated(c" REPLACE");
                    } else if State.get() & MODE_INSERT != 0 {
                        if p_ri.get() != 0 {
                            put_translated(c" REVERSE");
                        }
                        put_translated(c" INSERT");
                    } else if matches!(
                        u8::try_from(restart_edit.get()),
                        Ok(b'I' | b'i' | b'a' | b'A')
                    ) {
                        if (*curbuf.get()).terminal.is_null() {
                            put_translated(c" (insert)");
                        } else {
                            put_translated(c" (terminal)");
                        }
                    } else if restart_edit.get() == 'R' as c_int {
                        put_translated(c" (replace)");
                    } else if restart_edit.get() == 'V' as c_int {
                        put_translated(c" (vreplace)");
                    }

                    if State.get() & MODE_LANGMAP != 0 {
                        if (*curwin.get()).w_onebuf_opt.wo_arab != 0 {
                            put_translated(c" Arabic");
                        } else if let Some(keymap_name) = keymap_str(curwin.get()) {
                            let buf = NameBuff.ptr().cast::<c_char>();
                            let plen = vim_snprintf(
                                buf,
                                MAXPATHL as size_t,
                                c" (%s)".as_ptr(),
                                keymap_name.as_ptr(),
                            );
                            if plen > 0 && plen < MAXPATHL {
                                msg_puts_hl(buf, hl_id, false);
                            }
                        }
                    }

                    if State.get() & MODE_INSERT != 0 && p_paste.get() != 0 {
                        put_translated(c" (paste)");
                    }

                    if visual_active() {
                        // Upstream spells this as arithmetic over a `switch`,
                        // and does not concatenate the two words: the whole
                        // phrase is one translatable string.
                        //
                        // Its `case 3` -- blockwise AND linewise at once -- ends
                        // up in the SELECT BLOCK default; `VIsual_mode` cannot
                        // be both, so nothing observes the difference.
                        put_translated(
                            match (
                                visual_select(),
                                visual_mode().is_block(),
                                visual_mode().is_line(),
                            ) {
                                (false, true, _) => c" VISUAL BLOCK",
                                (false, _, true) => c" VISUAL LINE",
                                (false, _, _) => c" VISUAL",
                                (true, true, _) => c" SELECT BLOCK",
                                (true, _, true) => c" SELECT LINE",
                                (true, _, _) => c" SELECT",
                            },
                        );
                    }
                    put(c" --");
                }
                need_clear = true;
            }

            // The submode text already gets too long to share the line with it.
            if reg_recording.get() != 0 && edit_submode.get().is_null() {
                recording_mode(hl_id);
                need_clear = true;
            }

            mode_displayed.set(true);
            if need_clear || clear_cmdline.get() || redraw_mode.get() {
                msg_clr_eos();
            }
            msg_didout.set(false); // this message may be overwritten
            length = msg_col.get();
            msg_col.set(0);
            msg_no_more.set(false);
            lines_left.set(save_lines_left);
            need_wait_return.set(nwr_save); // never ask for hit-return for this
        } else if clear_cmdline.get() && msg_silent.get() == 0 {
            // Resets `clear_cmdline`.
            msg_clr_cmdline();
        } else if redraw_mode.get() {
            msg_pos_mode();
            msg_clr_eos();
        }

        // Also handles clearing the showmode when it was empty or disabled.
        msg_ext_flush_showmode();

        // In Visual mode the size of the selection is redrawn.
        if visual_active() {
            clear_showcmd();
        }

        redraw_ruler();
        redraw_cmdline.set(false);
        redraw_mode.set(false);
        clear_cmdline.set(false);

        length
    }
}

/// Put the message cursor where a mode message goes.
pub(crate) fn msg_pos_mode() {
    msg_col.set(0);
    msg_row.set(Rows.get() - 1);
}

/// Take the mode message away, e.g. when `<Esc>` is typed in Insert mode.
///
/// Insert mode has not actually ended yet at that point, which is why this is
/// separate from [`showmode`]. Callers check `mode_displayed` first.
pub unsafe fn unshowmode(force: bool) {
    // SAFETY: `char_avail` pumps the input layer on the main thread.
    unsafe {
        if !redrawing() || (!force && char_avail() && !KeyTyped.get()) {
            redraw_cmdline.set(true); // delete it later
        } else {
            clearmode();
        }
    }
}

/// Clear the mode message, keeping the message cursor where it was.
pub unsafe fn clearmode() {
    // SAFETY: the message layer on the main thread.
    unsafe {
        let save_msg_row = msg_row.get();
        let save_msg_col = msg_col.get();

        msg_ext_ui_flush();
        msg_pos_mode();
        // The recording indicator outlives the mode message.
        if reg_recording.get() != 0 {
            recording_mode(HLF_CM);
        }
        msg_clr_eos();
        msg_ext_flush_showmode();

        msg_col.set(save_msg_col);
        msg_row.set(save_msg_row);
    }
}

/// Print `recording @x` for the register being recorded into.
pub(crate) unsafe fn recording_mode(hl_id: c_int) {
    // SAFETY: the message layer on the main thread.
    unsafe {
        if shortmess(ShmFlag::RECORDING) {
            return;
        }
        msg_puts_hl(gettext(c"recording".as_ptr()), hl_id, false);
        // Upstream formats this with `snprintf(s, 4, " @%c", reg_recording)`,
        // which is exactly three bytes and the terminator.
        let suffix = [b' ', b'@', reg_recording.get() as u8, 0];
        msg_puts_hl(suffix.as_ptr().cast(), hl_id, false);
    }
}

/// Columns a standard ruler needs.
pub const COL_RULER: c_int = 17;

/// Recompute `sc_col` and `ru_col`, the first column of the `'showcmd'` area
/// and of the ruler on the last line.
///
/// `sc_col` also bounds how long a message on the status line may be. When the
/// last window has a status line the ruler lives there instead, so the two are
/// independent.
pub unsafe fn comp_col() {
    // SAFETY: `last_stl_height` walks the window layout on the main thread.
    unsafe {
        let last_has_status = last_stl_height(false) > 0;

        // Both start as *widths* counted from the right edge and are turned
        // into columns at the end.
        let mut sc_width = 0;
        let mut ru_width = 0;

        // Saturating throughout: `ru_wid` is whatever number the user put in
        // `'rulerformat'`'s leading `%<n>(`, unbounded and as large as
        // `INT_MAX`, and `+ 1` on that overflows. Any width past the screen
        // lands on the same column-one answer below, so saturating is the
        // clamp the arithmetic wanted anyway.
        if p_ru.get() != 0 {
            ru_width = if ru_wid.get() != 0 {
                ru_wid.get()
            } else {
                COL_RULER
            }
            .saturating_add(1);
            // With no status line on the last window the ruler shares the last
            // line, so 'showcmd' has to start left of it.
            if !last_has_status {
                sc_width = ru_width;
            }
        }
        if p_sc.get() != 0 && *p_sloc.get() == b'l' as c_char {
            sc_width = sc_width.saturating_add(SHOWCMD_COLS as c_int);
            // A separating space, unless the ruler is not beside it anyway.
            if p_ru.get() == 0 || last_has_status {
                sc_width = sc_width.saturating_add(1);
            }
        }

        debug_assert!(
            sc_width >= 0 && c_int::MIN + sc_width <= Columns.get(),
            "sc_col >= 0 && INT_MIN + sc_col <= Columns"
        );
        debug_assert!(
            ru_width >= 0 && c_int::MIN + ru_width <= Columns.get(),
            "ru_col >= 0 && INT_MIN + ru_col <= Columns"
        );

        // A screen too narrow for either is a mess whatever we do; keep them on
        // the screen.
        sc_col.set((Columns.get() - sc_width).max(1));
        ru_col.set((Columns.get() - ru_width).max(1));

        set_vim_var_nr(Vv::Echospace, (sc_col.get() - 1) as varnumber_T);
    }
}
