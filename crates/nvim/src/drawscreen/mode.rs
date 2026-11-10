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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::option::vars::P_SLOC;
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;
use core::ffi::c_int;
use std::ffi::CString;

use super::*;
use crate::normal::{visual_active, visual_mode, visual_select};
use crate::option::defs::ShmFlag;
use crate::os::cshim::gettext;
use crate::path::MAXPATHL;
use crate::types::Vv;

/// Whether to postpone the mode message: not redrawing, or inside a mapping.
///
/// Answering true also sets `REDRAW_MODE`, so it is shown on the next redraw.
pub fn skip_showmode() -> bool {
    if GLOBAL_BUSY.get() != 0
        || MSG_SILENT.get() != 0
        || !redrawing()
        || (char_avail() && !KEY_TYPED.get())
    {
        REDRAW_MODE.set(true);
        return true;
    }
    false
}

/// Show the current mode and ruler, and answer the length of what was printed.
///
/// `CLEAR_CMDLINE` asks for the rest of the command line to be cleared;
/// `REDRAW_MODE` asks for the mode to be shown or taken away even when there is
/// none to show.
pub fn showmode() -> c_int {
    let mut length = 0;

    // A message that has not been flushed must not become part of this one.
    msg_ext_ui_flush();
    msg_grid_validate();

    let do_mode = p_smd()
        && MSG_SILENT.get() == 0
        && (STATE.get() & MODE_TERMINAL != 0
            || STATE.get() & MODE_INSERT != 0
            || !RESTART_EDIT.get().is_nul()
            || visual_active());
    let can_show_mode = p_ch() != 0 || ui_has(UI_MESSAGES);

    if (do_mode || REG_RECORDING.get() != 0) && can_show_mode {
        if skip_showmode() {
            return 0;
        }

        let nwr_save = NEED_WAIT_RETURN.get();

        // Wait a bit before overwriting an important message.
        msg_check_for_delay(false);

        let mut need_clear = CLEAR_CMDLINE.get();
        if CLEAR_CMDLINE.get() && CMDLINE_ROW.get() < ROWS.get() - 1 {
            // Resets `CLEAR_CMDLINE`, which is why `need_clear` was read
            // first.
            msg_clr_cmdline();
        }

        msg_pos_mode();
        let hl_id = HLF_CM;

        // Truncate rather than scroll when the screen is too narrow for the
        // whole message.
        MSG_NO_MORE.set(true);
        let save_lines_left = LINES_LEFT.get();
        LINES_LEFT.set(0);

        if do_mode {
            msg_str_hl(c"--", hl_id, false);
            if let Some(submode) = EDIT_SUBMODE.get()
                && !shortmess(ShmFlag::COMPLETIONMENU)
            {
                show_submode(submode, hl_id);
            } else {
                show_mode_name(hl_id);
                msg_str_hl(c" --", hl_id, false);
            }
            need_clear = true;
        }

        // The submode text already gets too long to share the line with it.
        if REG_RECORDING.get() != 0 && EDIT_SUBMODE.get().is_none() {
            recording_mode(hl_id);
            need_clear = true;
        }

        MODE_DISPLAYED.set(true);
        if need_clear || CLEAR_CMDLINE.get() || REDRAW_MODE.get() {
            msg_clr_eos();
        }
        MSG_DIDOUT.set(false); // this message may be overwritten
        length = MSG_COL.get();
        MSG_COL.set(0);
        MSG_NO_MORE.set(false);
        LINES_LEFT.set(save_lines_left);
        NEED_WAIT_RETURN.set(nwr_save); // never ask for hit-return for this
    } else if CLEAR_CMDLINE.get() && MSG_SILENT.get() == 0 {
        // Resets `CLEAR_CMDLINE`.
        msg_clr_cmdline();
    } else if REDRAW_MODE.get() {
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
    REDRAW_CMDLINE.set(false);
    REDRAW_MODE.set(false);
    CLEAR_CMDLINE.set(false);

    length
}

/// The CTRL-X submode text Insert-mode completion shows in place of the
/// mode: `EDIT_SUBMODE_PRE`, the submode and `EDIT_SUBMODE_EXTRA`. These get
/// long, so they are budgeted against the room left rather than allowed to
/// wrap; an external message UI imposes no limit of its own.
/// `EDIT_SUBMODE_EXTRA` is preferred over `EDIT_SUBMODE`.
fn show_submode(submode: &'static CStr, hl_id: c_int) {
    let mut room = if ui_has(UI_MESSAGES) {
        c_int::MAX
    } else {
        (ROWS.get() - MSG_ROW.get()) * COLUMNS.get() - 3
    };
    let pre = EDIT_SUBMODE_PRE.get();
    // A copy: drawing it runs the message machinery.
    let extra = EDIT_SUBMODE_EXTRA.with(Clone::clone);
    if let Some(extra) = &extra {
        room -= vim_strsize_cstr(extra.as_cstr());
    }
    if room <= 0 {
        return;
    }
    if let Some(pre) = pre {
        room -= vim_strsize_cstr(pre);
    }
    if room - vim_strsize_cstr(submode) > 0 {
        if let Some(pre) = pre {
            msg_str_hl(pre, hl_id, false);
        }
        msg_str_hl(submode, hl_id, false);
    }
    if let Some(extra) = &extra {
        msg_str_hl(c" ", hl_id, false);
        let sub_id = if EDIT_SUBMODE_HIGHL.get() < HLF_COUNT {
            EDIT_SUBMODE_HIGHL.get()
        } else {
            hl_id
        };
        msg_str_hl(extra.as_cstr(), sub_id, false);
    }
}

/// The mode itself, between `--` and ` --`: the editing mode, the keymap,
/// `(paste)` and the Visual mode, each a translated word or phrase.
fn show_mode_name(hl_id: c_int) {
    let put_translated = |s: &'static CStr| msg_str_hl(gettext(s), hl_id, false);

    // Not a `match`: these are tests on different variables, in upstream's
    // precedence order.
    if STATE.get() & MODE_TERMINAL != 0 {
        put_translated(c" TERMINAL");
    } else if STATE.get() & VREPLACE_FLAG != 0 {
        put_translated(c" VREPLACE");
    } else if STATE.get() & REPLACE_FLAG != 0 {
        put_translated(c" REPLACE");
    } else if STATE.get() & MODE_INSERT != 0 {
        if p_ri() {
            put_translated(c" REVERSE");
        }
        put_translated(c" INSERT");
    } else if matches!(
        RESTART_EDIT.get().as_byte(),
        Some(b'I' | b'i' | b'a' | b'A')
    ) {
        if Buf::current().terminal.is_null() {
            put_translated(c" (insert)");
        } else {
            put_translated(c" (terminal)");
        }
    } else if RESTART_EDIT.get().is('R') {
        put_translated(c" (replace)");
    } else if RESTART_EDIT.get().is('V') {
        put_translated(c" (vreplace)");
    }

    if STATE.get() & MODE_LANGMAP != 0 {
        if Win::current().w_onebuf_opt.wo_arab != 0 {
            put_translated(c" Arabic");
        } else if let Some(keymap_name) = keymap_str(Win::current()) {
            // Shown only when it would have fitted upstream's `MAXPATHL`
            // buffer.
            let name = keymap_name.as_bytes();
            if name.len() + 3 < MAXPATHL as usize {
                let mut text = Vec::with_capacity(name.len() + 4);
                text.extend_from_slice(b" (");
                text.extend_from_slice(name);
                text.extend_from_slice(b")");
                let text = CString::new(text).expect("a keymap name holds no NUL");
                msg_str_hl(&text, hl_id, false);
            }
        }
    }

    if STATE.get() & MODE_INSERT != 0 && p_paste() {
        put_translated(c" (paste)");
    }

    if visual_active() {
        put_translated(visual_mode_name(
            visual_select(),
            visual_mode().is_block(),
            visual_mode().is_line(),
        ));
    }
}

/// The Visual mode's phrase, untranslated: Select or Visual, blockwise,
/// linewise or neither.
///
/// Upstream spells this as arithmetic over a `switch`, and does not
/// concatenate the two words: the whole phrase is one translatable string.
/// Its `case 3` -- blockwise AND linewise at once -- ends up in the SELECT
/// BLOCK default; `VIsual_mode` cannot be both, so nothing observes the
/// difference.
fn visual_mode_name(select: bool, block: bool, line: bool) -> &'static CStr {
    match (select, block, line) {
        (false, true, _) => c" VISUAL BLOCK",
        (false, _, true) => c" VISUAL LINE",
        (false, _, _) => c" VISUAL",
        (true, true, _) => c" SELECT BLOCK",
        (true, _, true) => c" SELECT LINE",
        (true, _, _) => c" SELECT",
    }
}

/// Put the message cursor where a mode message goes.
pub(crate) fn msg_pos_mode() {
    MSG_COL.set(0);
    MSG_ROW.set(ROWS.get() - 1);
}

/// Take the mode message away, e.g. when `<Esc>` is typed in Insert mode.
///
/// Insert mode has not actually ended yet at that point, which is why this is
/// separate from [`showmode`]. Callers check `MODE_DISPLAYED` first.
pub fn unshowmode(force: bool) {
    if !redrawing() || (!force && char_avail() && !KEY_TYPED.get()) {
        REDRAW_CMDLINE.set(true); // delete it later
    } else {
        clearmode();
    }
}

/// Clear the mode message, keeping the message cursor where it was.
pub fn clearmode() {
    let save_msg_row = MSG_ROW.get();
    let save_msg_col = MSG_COL.get();

    msg_ext_ui_flush();
    msg_pos_mode();
    // The recording indicator outlives the mode message.
    if REG_RECORDING.get() != 0 {
        recording_mode(HLF_CM);
    }
    msg_clr_eos();
    msg_ext_flush_showmode();

    MSG_COL.set(save_msg_col);
    MSG_ROW.set(save_msg_row);
}

/// Print `recording @x` for the register being recorded into.
pub(crate) fn recording_mode(hl_id: c_int) {
    if shortmess(ShmFlag::RECORDING) {
        return;
    }
    msg_str_hl(gettext(c"recording"), hl_id, false);
    // Upstream formats this with `snprintf(s, 4, " @%c", REG_RECORDING)`,
    // which is exactly three bytes and the terminator.
    let reg = REG_RECORDING.get();
    let suffix = [b' ', b'@', reg.byte(), 0];
    msg_str_hl(cstr::in_bytes(&suffix), hl_id, false);
}

/// Columns a standard ruler needs.
pub const COL_RULER: c_int = 17;

/// Recompute `SC_COL` and `RU_COL`, the first column of the `'showcmd'` area
/// and of the ruler on the last line.
///
/// `SC_COL` also bounds how long a message on the status line may be. When the
/// last window has a status line the ruler lives there instead, so the two are
/// independent.
pub fn comp_col() {
    let last_has_status = last_stl_height(false) > 0;

    // Both start as *widths* counted from the right edge and are turned
    // into columns at the end.
    let mut sc_width = 0;
    let mut ru_width = 0;

    // Saturating throughout: `RU_WID` is whatever number the user put in
    // `'rulerformat'`'s leading `%<n>(`, unbounded and as large as
    // `INT_MAX`, and `+ 1` on that overflows. Any width past the screen
    // lands on the same column-one answer below, so saturating is the
    // clamp the arithmetic wanted anyway.
    if p_ru() {
        ru_width = if RU_WID.get() != 0 {
            RU_WID.get()
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
    if p_sc() && P_SLOC.first_byte() == b'l' {
        sc_width = sc_width.saturating_add(SHOWCMD_COLS as c_int);
        // A separating space, unless the ruler is not beside it anyway.
        if !p_ru() || last_has_status {
            sc_width = sc_width.saturating_add(1);
        }
    }

    debug_assert!(
        sc_width >= 0 && c_int::MIN + sc_width <= COLUMNS.get(),
        "sc_col >= 0 && INT_MIN + sc_col <= Columns"
    );
    debug_assert!(
        ru_width >= 0 && c_int::MIN + ru_width <= COLUMNS.get(),
        "ru_col >= 0 && INT_MIN + ru_col <= Columns"
    );

    // A screen too narrow for either is a mess whatever we do; keep them on
    // the screen.
    SC_COL.set((COLUMNS.get() - sc_width).max(1));
    RU_COL.set((COLUMNS.get() - ru_width).max(1));

    set_vim_var_nr(Vv::Echospace, VarNumber::from(SC_COL.get() - 1));
}

#[cfg(test)]
mod tests {
    use super::visual_mode_name;

    #[test]
    fn the_visual_phrase_names_select_and_the_shape() {
        assert_eq!(visual_mode_name(false, false, false), c" VISUAL");
        assert_eq!(visual_mode_name(false, false, true), c" VISUAL LINE");
        assert_eq!(visual_mode_name(false, true, false), c" VISUAL BLOCK");
        assert_eq!(visual_mode_name(true, false, false), c" SELECT");
        assert_eq!(visual_mode_name(true, false, true), c" SELECT LINE");
        // Both shapes at once (never seen) is upstream's SELECT BLOCK
        // default, and blockwise wins for Visual too.
        assert_eq!(visual_mode_name(true, true, true), c" SELECT BLOCK");
        assert_eq!(visual_mode_name(false, true, true), c" VISUAL BLOCK");
    }
}
