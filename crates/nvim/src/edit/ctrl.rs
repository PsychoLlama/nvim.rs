//! The CTRL- commands that change what Insert mode *is*.
//!
//! [`ins_esc`] ends it, and it is the delicate one: `count` may mean the
//! whole insert repeats (in which case it answers `false` and the state
//! machine runs again), the cursor moves back onto the last inserted
//! character, and `'^` is set.  [`ins_ctrl_o`] leaves the mode for exactly
//! one Normal-mode command by setting `RESTART_EDIT` to the letter that will
//! bring this mode back.  [`ins_insert`] switches between Insert and Replace.
//!
//! [`ins_ctrl_g`] is the CTRL-G prefix (`CTRL-G j`/`k` move to
//! `INSSTART.col`, `CTRL-G u` starts a new undo block, `CTRL-G U` keeps the
//! next motion inside this one), and [`ins_ctrl_hat`] and [`ins_ctrl_`]
//! toggle the `:lmap` mappings and 'revins'.
//!
//! [`ins_reg`] is CTRL-R.  It is here rather than beside `insertchar`
//! because what it does is *stuff text into the input stream* -- so the
//! `"` it draws at the cursor has to be taken back again by hand, on every
//! path, which is what `need_redraw` is for.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::keycodes::SpecialKey;
use crate::mark::defs::FileMark;
use crate::option::vars::P_LANGMAP;
use crate::types::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::NUL;
use crate::ex_docmd::cmdmod_has;
use crate::getchar::typeahead;
use crate::guard::{Keys, Suppress};
use crate::r#move::WinValid;
use crate::normal::visual_active;
use crate::option::cpo_has;
use crate::option::defs::CpoFlag;
use crate::register::RegName;

/// The three CTRL-G commands that are spelled with a letter.
const CTRL_G_UP: Key = Key::ascii(b'k');
const CTRL_G_DOWN: Key = Key::ascii(b'j');
const CTRL_G_UNDO: Key = Key::ascii(b'u');
const CTRL_G_KEEP_UNDO: Key = Key::ascii(b'U');

/// CTRL-R: insert the contents of a register.
///
/// Three keys deep at most: `CTRL-R`, then optionally one of
/// `CTRL-R`/`CTRL-O`/`CTRL-P` asking for a literal insert, then the register
/// name.  `=` runs the expression register, which may do anything at all --
/// including `:stopinsert`, which is why the answer is checked rather than
/// assumed to have inserted something.
pub(crate) fn ins_reg() {
    let mut need_redraw = false;
    let mut literally = Key::NUL;
    let vis_active = visual_active();

    // A character is about to be waited for: show a `"`.
    PC_STATUS.set(PutChar::Unset);
    if redrawing() && !char_avail() {
        // May need to redraw now that no more characters are available.
        ins_redraw(false);
        edit_putchar(Key::ascii(b'"'), true);
        add_to_showcmd_c(Key::CTRL_R);
    }

    // `LANGMAP_ADJUST(c, true)`: apply 'langmap' to a *typed* key only,
    // and only when it was not produced by a mapping.
    let langmap_adjust = |c: Key| -> Key {
        let typed = if VGETC_BUSY.get() != 0 {
            typeahead().maplen() == 0
        } else {
            KEY_TYPED.get()
        };
        // `first_byte` is upstream's `*p_langmap`; the projecting reader
        // would walk the value to hand out a `&CStr`.
        if P_LANGMAP.first_byte() != 0
            && (p_lrm() || typed)
            && !KEY_STUFFED.get()
            && !c.is_special()
        {
            match c.as_byte() {
                Some(b) => Key::byte(LANGMAP_MAPCHAR.with(|map| map[usize::from(b)])),
                None => langmap_adjust_mb(c),
            }
        } else {
            c
        }
    };

    // Don't map the register name.  This also keeps the mode message
    // from being deleted when ESC is hit.
    let raw_key = Keys::unmapped_with_codes();
    let mut regname = langmap_adjust(get_key());
    if regname == Key::CTRL_R || regname == Key::CTRL_O || regname == Key::CTRL_P {
        // A third key follows, for a literal register insertion.
        literally = regname;
        add_to_showcmd_c(literally);
        regname = langmap_adjust(get_key());
    }
    // A register is named by its character; a key that is not a byte
    // names none.
    let mut regname = register_name(regname);
    drop(raw_key);

    // Don't `u_sync()` while typing the expression, or while giving an
    // error message for it; only explicitly.
    let no_sync = Suppress::undo_sync();
    if regname == Some(RegName::EXPRESSION) {
        let curpos = Win::current().w_cursor;
        // Sync undo if the expression calls setline() or append(), so
        // that can be undone separately.
        U_SYNC_ONCE.set(2);
        regname = get_expr_register().then_some(RegName::EXPRESSION);
        // The cursor may have been moved back a column.
        Win::current().w_cursor = curpos;
        check_cursor(Win::current());
    }

    let regname = regname.filter(|&name| name != 0 && name.is_valid(false));
    if let Some(regname) = regname {
        let reg = get_yank_register(regname, YREG_PASTE);

        if literally == Key::CTRL_O || literally == Key::CTRL_P {
            // Append the command to the redo buffer.
            append_to_redobuff_char(Key::CTRL_R);
            append_to_redobuff_char(literally);
            append_to_redobuff_char(register_key(regname));
            let fix = if literally == Key::CTRL_P {
                PUT_FIXINDENT
            } else {
                0
            };
            do_put(regname, None, BACKWARD, 1, fix | PUT_CURSEND);
        } else if reg.read(|reg| reg.lines.len()) > 1 && is_literal_register(regname) {
            append_to_redobuff_char(Key::CTRL_R);
            append_to_redobuff_char(register_key(regname));
            do_put(regname, None, BACKWARD, 1, PUT_CURSEND);
        } else if insert_reg(regname, None, !literally.is_nul()).is_err() {
            beep(OPT_BO_FLAG_REGISTER as ::core::ffi::c_uint);
            need_redraw = true; // remove the `"`
        } else if STOP_INSERT_MODE.get() {
            // The `=` register invoked a function that did
            // `:stopinsert`: `stuff_empty()` answers false but nothing
            // will be inserted, so the `"` still has to go.
            need_redraw = true;
        }
    } else {
        beep(OPT_BO_FLAG_REGISTER as ::core::ffi::c_uint);
        need_redraw = true; // remove the `"`
    }

    drop(no_sync);
    if U_SYNC_ONCE.get() == 1 {
        INS_NEED_UNDO.set(true);
    }
    U_SYNC_ONCE.set(0);

    // Remove the `"` if the register was empty.  Before `clear_showcmd`,
    // which emits an event that can also update the screen.
    if need_redraw || stuff_empty() {
        edit_unputchar();
    }
    clear_showcmd();

    // Starting Visual mode here would leave a weird mode.
    if !vis_active && visual_active() {
        end_visual_mode();
    }
}

/// The CTRL-G commands in Insert mode.
pub(crate) fn ins_ctrl_g() {
    // Right after CTRL-X the cursor will be after the ruler.
    setcursor();

    // Don't map the second key.  This also keeps the mode message from
    // being deleted when ESC is hit.
    let c = {
        let _raw_key = Keys::unmapped_with_codes();
        get_key()
    };

    match c {
        // CTRL-G k and CTRL-G <Up>: cursor up to Insstart.col.
        _ if c == SpecialKey::Up => ins_updown(true, true),
        Key::CTRL_K | CTRL_G_UP => ins_updown(true, true),
        // CTRL-G j and CTRL-G <Down>: cursor down to Insstart.col.
        _ if c == SpecialKey::Down => ins_updown(false, true),
        Key::CTRL_J | CTRL_G_DOWN => ins_updown(false, true),
        // CTRL-G u: start a new undoable edit.
        CTRL_G_UNDO => {
            u_sync(true);
            INS_NEED_UNDO.set(true);
            // Insstart has to be reset too, because a BS that joins this
            // line to the previous one must save for undo.
            UPDATE_INSSTART_ORIG.set(false);
            INSSTART.set(Win::current().w_cursor);
        }
        // CTRL-G U: allow one left/right cursor movement with the next
        // key without breaking undo.
        CTRL_G_KEEP_UNDO => DONT_SYNC_UNDO.set(KeepUndo::Armed),
        // Esc after CTRL-G cancels it.
        Key::ESC => {}
        // Unknown; reserved for future expansion.
        _ => beep(OPT_BO_FLAG_CTRLG as ::core::ffi::c_uint),
    }
}

/// CTRL-^ in Insert mode: toggle the `:lmap` mappings.
pub(crate) fn ins_ctrl_hat() {
    if map_to_exists_mode(b"", MODE_LANGMAP, false) {
        // `:lmap` mappings exist, so the key toggles their use.
        if STATE.get() & MODE_LANGMAP != 0 {
            Buf::current().b_p_iminsert = B_IMODE_NONE;
            STATE.set(STATE.get() & !MODE_LANGMAP);
        } else {
            Buf::current().b_p_iminsert = B_IMODE_LMAP;
            STATE.set(STATE.get() | MODE_LANGMAP);
        }
    }
    set_iminsert_global(Buf::current());
    show_mode();
    // Show or unshow the value of 'keymap' in status lines.
    status_redraw_curbuf();
}

/// Handle `<Esc>` in Insert mode.
///
/// `count` is the repeat count of the insert command, and is decremented
/// here: answering `false` means "the insert repeats", and the caller runs
/// the state machine again rather than leaving the mode.  `cmdchar` is the
/// command that started the insert (`r` and `v` are the single-character
/// forms, which must not put an ESC in the redo buffer), and `nomove` is
/// `i_CTRL-\_CTRL-O`.
pub(crate) fn ins_esc(count: &mut c_int, cmdchar: Key, nomove: bool) -> bool {
    check_spell_redraw();

    let temp = Win::current().w_cursor.col;
    if DISABLED_REDRAW.get() {
        REDRAWING_DISABLED.set(REDRAWING_DISABLED.get() - 1);
        DISABLED_REDRAW.set(false);
    }

    let single_char_insert = cmdchar.is('r') || cmdchar.is('v');
    if !ARROW_USED.get() {
        // Don't append the ESC for `r<CR>` and `grx`.
        if !single_char_insert {
            append_to_redobuff(ESC_STR.to_bytes());
        }

        // Repeating an insert may take a long time; check for an
        // interrupt now and then.
        if *count > 0 {
            line_breakcheck();
            if GOT_INT.get() {
                *count = 0;
            }
        }

        *count -= 1;
        if *count > 0 {
            // Repeat what was typed.  Vi repeats the insert without
            // replacing characters.
            if cpo_has(CpoFlag::REPLCNT) {
                STATE.set(STATE.get() & !REPLACE_FLAG);
            }
            let _ = start_redo_ins();
            if single_char_insert {
                stuff_redo_readbuf(ESC_STR.to_bytes()); // no ESC in the redo buffer
            }
            REDRAWING_DISABLED.set(REDRAWING_DISABLED.get() + 1);
            DISABLED_REDRAW.set(true);
            return false;
        }
        stop_insert(Some(InsertEnd::cursor()), true, nomove);
        undisplay_dollar();
    }

    if !single_char_insert {
        ins_apply_autocmds(AutoEvent::InsertLeavePre);
    }

    // When an auto-indent was removed, curswant stays after the indent.
    if RESTART_EDIT.get().is_nul() && temp == Win::current().w_cursor.col {
        Win::current().w_set_curswant = true;
    }

    // Remember the last Insert position in the `'^` mark (`RESET_FMARK`).
    if !cmdmod_has(CmdModFlags::KEEPJUMPS) {
        let view = mark_view_make(Win::current(), Win::current().w_cursor);
        let mut buf = Buf::current();
        buf.b_last_insert = FileMark::fresh(Win::current().w_cursor, Some(buf.handle), view);
    }

    // The cursor should end up on the last inserted character.  Not for
    // CTRL-O, unless it is past the end of the line.
    if !nomove
        && (Win::current().w_cursor.col != ColNr::ZERO || Win::current().w_cursor.coladd > 0)
        && (RESTART_EDIT.get().is_nul() || (char_at_cursor() == NUL && !visual_active()))
        && !REVINS_ON.get()
    {
        if Win::current().w_cursor.coladd > 0
            || get_ve_flags(Win::current()) == OPT_VE_FLAG_ALL as ::core::ffi::c_uint
        {
            let _ = oneleft();
            if !RESTART_EDIT.get().is_nul() {
                Win::current().w_cursor.coladd += 1;
            }
        } else {
            Win::current().w_cursor.col -= 1;
            Win::current()
                .w_valid
                .clear(WinValid::WCOL | WinValid::VIRTCOL);
            // Correct the cursor for a multi-byte character.
            mb_adjust_cursor();
        }
    }

    STATE.set(MODE_NORMAL);
    may_trigger_modechanged();
    // The cursor needs positioning again when it is on a TAB, and when
    // the line carries inline virtual text.
    if char_at_cursor() == c_int::from(b'\t') || buf_meta_total(Buf::current(), MT_META_INLINE) > 0
    {
        Win::current()
            .w_valid
            .clear(WinValid::WROW | WinValid::WCOL | WinValid::VIRTCOL);
    }

    setmouse();
    ui_cursor_shape(); // may show a different cursor shape

    // While recording, and for CTRL-O, the new mode has to be displayed;
    // otherwise the mode message is removed.
    if REG_RECORDING.get() != 0 || !RESTART_EDIT.get().is_nul() {
        show_mode();
    } else if p_smd()
        && (GOT_INT.get() || !skip_showmode())
        && !(p_ch() == 0 && !ui_has(UI_MESSAGES))
    {
        unshowmode(false);
    }

    true // exit Insert mode
}

/// CTRL-_ : toggle 'revins', and move to the end of the reverse-inserted
/// text.
pub(crate) fn ins_ctrl_() {
    if REVINS_ON.get() && REVINS_CHARS.get() != 0 && REVINS_SCOL.get() >= 0 {
        // Upstream's `while (gchar_cursor() != NUL && REVINS_CHARS--)`:
        // the decrement runs on the iteration that ends the loop too, so
        // exhausting the count leaves `REVINS_CHARS` at -1.
        while char_at_cursor() != NUL {
            let n = REVINS_CHARS.get();
            REVINS_CHARS.set(n - 1);
            if n == 0 {
                break;
            }
            Win::current().w_cursor.col += 1;
        }
    }

    P_RI.set(!p_ri());
    REVINS_ON.set(STATE.get() == MODE_INSERT && p_ri());
    if REVINS_ON.get() {
        REVINS_SCOL.set(Win::current().w_cursor.col.get());
        REVINS_LEGAL.set(REVINS_LEGAL.get() + 1);
        REVINS_CHARS.set(0);
        undisplay_dollar();
    } else {
        REVINS_SCOL.set(-1);
    }
    show_mode();
}

/// `<Insert>` in Insert mode: toggle between Insert and Replace.
///
/// `replace_state` is the Replace mode to go *to* -- `MODE_REPLACE` or
/// `MODE_VREPLACE`, depending on how the insert was started.
pub(crate) fn ins_insert(replace_state: c_int) {
    let mode: &[u8] = if STATE.get() & REPLACE_FLAG != 0 {
        b"i"
    } else if replace_state == MODE_VREPLACE {
        b"v"
    } else {
        b"r"
    };
    set_vim_var_string(Vv::Insertmode, Some(mode));
    ins_apply_autocmds(AutoEvent::InsertChange);

    if STATE.get() & REPLACE_FLAG != 0 {
        STATE.set(MODE_INSERT | STATE.get() & MODE_LANGMAP);
    } else {
        STATE.set(replace_state | STATE.get() & MODE_LANGMAP);
    }
    may_trigger_modechanged();
    append_to_redobuff_char(Key::from(SpecialKey::Ins));
    show_mode();
    ui_cursor_shape();
}

/// CTRL-O: leave Insert mode for exactly one Normal-mode command.
///
/// `RESTART_EDIT` holds the letter that brings *this* mode back afterwards,
/// and `INS_AT_EOL` remembers whether the cursor was past the last character
/// so it can be put back there.
pub(crate) fn ins_ctrl_o() {
    RESTART_V_ISUAL_SELECT.set(0);
    RESTART_EDIT.set(if STATE.get() & VREPLACE_FLAG != 0 {
        Key::ascii(b'V')
    } else if STATE.get() & REPLACE_FLAG != 0 {
        Key::ascii(b'R')
    } else {
        Key::ascii(b'I')
    });
    INS_AT_EOL.set(if virtual_active(Win::current()) {
        false
    } else {
        char_at_cursor() == NUL
    });
}

/// Whether the next character may trigger a 'cindent' re-indent.
pub(crate) fn get_can_cindent() -> bool {
    CAN_CINDENT.get()
}

/// Say whether the next character may trigger a 'cindent' re-indent.
pub(crate) fn set_can_cindent(val: bool) {
    CAN_CINDENT.set(val);
}

/// Show the mode message, if 'showmode' asks for one.
#[inline(always)]
fn show_mode() -> c_int {
    showmode()
}

/// The character under the cursor, `NUL` at the end of the line.
#[inline(always)]
fn char_at_cursor() -> c_int {
    gchar_cursor()
}

/// Read the next key, waiting for it if need be.
#[inline(always)]
fn get_key() -> Key {
    plain_vgetc()
}

/// Beep, or flash, for the 'belloff' reason `flag`.
#[inline(always)]
fn beep(flag: ::core::ffi::c_uint) {
    vim_beep(flag)
}
