//! The CTRL- commands that change what Insert mode *is*.
//!
//! [`ins_esc`] ends it, and it is the delicate one: `count` may mean the
//! whole insert repeats (in which case it answers `false` and the state
//! machine runs again), the cursor moves back onto the last inserted
//! character, and `'^` is set.  [`ins_ctrl_o`] leaves the mode for exactly
//! one Normal-mode command by setting `restart_edit` to the letter that will
//! bring this mode back.  [`ins_insert`] switches between Insert and Replace.
//!
//! [`ins_ctrl_g`] is the CTRL-G prefix (`CTRL-G j`/`k` move to
//! `Insstart.col`, `CTRL-G u` starts a new undo block, `CTRL-G U` keeps the
//! next motion inside this one), and [`ins_ctrl_hat`] and [`ins_ctrl_`]
//! toggle the `:lmap` mappings and 'revins'.
//!
//! [`ins_reg`] is CTRL-R.  It is here rather than beside `insertchar`
//! because what it does is *stuff text into the input stream* -- so the
//! `"` it draws at the cursor has to be taken back again by hand, on every
//! path, which is what `need_redraw` is for.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;

/// The three CTRL-G commands that are spelled with a letter.
const CTRL_G_UP: c_int = b'k' as c_int;
const CTRL_G_DOWN: c_int = b'j' as c_int;
const CTRL_G_UNDO: c_int = b'u' as c_int;
const CTRL_G_KEEP_UNDO: c_int = b'U' as c_int;

/// CTRL-R: insert the contents of a register.
///
/// Three keys deep at most: `CTRL-R`, then optionally one of
/// `CTRL-R`/`CTRL-O`/`CTRL-P` asking for a literal insert, then the register
/// name.  `=` runs the expression register, which may do anything at all --
/// including `:stopinsert`, which is why the answer is checked rather than
/// assumed to have inserted something.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_reg() {
    unsafe {
        let mut need_redraw = false;
        let mut literally = 0;
        let vis_active = VIsual_active.get();

        // A character is about to be waited for: show a `"`.
        pc_status.set(PutChar::Unset);
        if redrawing() && !char_avail() {
            // May need to redraw now that no more characters are available.
            ins_redraw(false);
            edit_putchar('"' as c_int, true);
            add_to_showcmd_c(Ctrl_R);
        }

        // `LANGMAP_ADJUST(c, true)`: apply 'langmap' to a *typed* key only,
        // and only when it was not produced by a mapping.
        let langmap_adjust = |c: c_int| -> c_int {
            let typed = if vgetc_busy.get() != 0 {
                typebuf_maplen() == 0
            } else {
                KeyTyped.get()
            };
            if *p_langmap.get() as c_int != 0
                && (p_lrm.get() != 0 || typed)
                && KeyStuffed.get() == 0
                && c >= 0
            {
                if c < 256 {
                    (*langmap_mapchar.ptr())[c as usize] as c_int
                } else {
                    langmap_adjust_mb(c)
                }
            } else {
                c
            }
        };

        // Don't map the register name.  This also keeps the mode message
        // from being deleted when ESC is hit.
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let mut regname = langmap_adjust(plain_vgetc());
        if regname == Ctrl_R || regname == Ctrl_O || regname == Ctrl_P {
            // A third key follows, for a literal register insertion.
            literally = regname;
            add_to_showcmd_c(literally);
            regname = langmap_adjust(plain_vgetc());
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;

        // Don't `u_sync()` while typing the expression, or while giving an
        // error message for it; only explicitly.
        (*no_u_sync.ptr()) += 1;
        if regname == '=' as c_int {
            let curpos = (*curwin.get()).w_cursor;
            // Sync undo if the expression calls setline() or append(), so
            // that can be undone separately.
            u_sync_once.set(2);
            regname = get_expr_register();
            // The cursor may have been moved back a column.
            (*curwin.get()).w_cursor = curpos;
            check_cursor(curwin.get());
        }

        if regname == NUL || !valid_yank_reg(regname, false) {
            vim_beep(kOptBoFlagRegister as ::core::ffi::c_uint);
            need_redraw = true; // remove the `"`
        } else {
            let reg = get_yank_register(regname, YREG_PASTE);

            if literally == Ctrl_O || literally == Ctrl_P {
                // Append the command to the redo buffer.
                AppendCharToRedobuff(Ctrl_R);
                AppendCharToRedobuff(literally);
                AppendCharToRedobuff(regname);
                let fix = if literally == Ctrl_P {
                    PUT_FIXINDENT as c_int
                } else {
                    0
                };
                do_put(
                    regname,
                    ::core::ptr::null_mut(),
                    BACKWARD,
                    1,
                    fix | PUT_CURSEND as c_int,
                );
            } else if (*reg).y_size > 1 && is_literal_register(regname) {
                AppendCharToRedobuff(Ctrl_R);
                AppendCharToRedobuff(regname);
                do_put(
                    regname,
                    ::core::ptr::null_mut(),
                    BACKWARD,
                    1,
                    PUT_CURSEND as c_int,
                );
            } else if insert_reg(regname, ::core::ptr::null_mut(), literally != 0) == FAIL {
                vim_beep(kOptBoFlagRegister as ::core::ffi::c_uint);
                need_redraw = true; // remove the `"`
            } else if stop_insert_mode.get() {
                // The `=` register invoked a function that did
                // `:stopinsert`: `stuff_empty()` answers false but nothing
                // will be inserted, so the `"` still has to go.
                need_redraw = true;
            }
        }

        (*no_u_sync.ptr()) -= 1;
        if u_sync_once.get() == 1 {
            ins_need_undo.set(true);
        }
        u_sync_once.set(0);

        // Remove the `"` if the register was empty.  Before `clear_showcmd`,
        // which emits an event that can also update the screen.
        if need_redraw || stuff_empty() {
            edit_unputchar();
        }
        clear_showcmd();

        // Starting Visual mode here would leave a weird mode.
        if !vis_active && VIsual_active.get() {
            end_visual_mode();
        }
    }
}

/// The CTRL-G commands in Insert mode.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_ctrl_g() {
    unsafe {
        // Right after CTRL-X the cursor will be after the ruler.
        setcursor();

        // Don't map the second key.  This also keeps the mode message from
        // being deleted when ESC is hit.
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let c = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;

        match c {
            // CTRL-G k and CTRL-G <Up>: cursor up to Insstart.col.
            K_UP | Ctrl_K | CTRL_G_UP => ins_updown(true, true),
            // CTRL-G j and CTRL-G <Down>: cursor down to Insstart.col.
            K_DOWN | Ctrl_J | CTRL_G_DOWN => ins_updown(false, true),
            // CTRL-G u: start a new undoable edit.
            CTRL_G_UNDO => {
                u_sync(true);
                ins_need_undo.set(true);
                // Insstart has to be reset too, because a BS that joins this
                // line to the previous one must save for undo.
                update_Insstart_orig.set(false);
                Insstart.set((*curwin.get()).w_cursor);
            }
            // CTRL-G U: allow one left/right cursor movement with the next
            // key without breaking undo.
            CTRL_G_KEEP_UNDO => dont_sync_undo.set(kNone),
            // Esc after CTRL-G cancels it.
            ESC => {}
            // Unknown; reserved for future expansion.
            _ => vim_beep(kOptBoFlagCtrlg as ::core::ffi::c_uint),
        }
    }
}

/// CTRL-^ in Insert mode: toggle the `:lmap` mappings.
///
/// # Safety
/// Must run with a live `curbuf`.
pub(crate) unsafe fn ins_ctrl_hat() {
    unsafe {
        if map_to_exists_mode(c"".as_ptr(), MODE_LANGMAP, false) {
            // `:lmap` mappings exist, so the key toggles their use.
            if State.get() & MODE_LANGMAP != 0 {
                (*curbuf.get()).b_p_iminsert = B_IMODE_NONE as OptInt;
                (*State.ptr()) &= !MODE_LANGMAP;
            } else {
                (*curbuf.get()).b_p_iminsert = B_IMODE_LMAP as OptInt;
                (*State.ptr()) |= MODE_LANGMAP;
            }
        }
        set_iminsert_global(curbuf.get());
        showmode();
        // Show or unshow the value of 'keymap' in status lines.
        status_redraw_curbuf();
    }
}

/// Handle `<Esc>` in Insert mode.
///
/// `count` is the repeat count of the insert command, and is decremented
/// here: answering `false` means "the insert repeats", and the caller runs
/// the state machine again rather than leaving the mode.  `cmdchar` is the
/// command that started the insert (`r` and `v` are the single-character
/// forms, which must not put an ESC in the redo buffer), and `nomove` is
/// `i_CTRL-\_CTRL-O`.
///
/// # Safety
/// `count` must point to a live `c_int`.
pub(crate) unsafe fn ins_esc(count: *mut c_int, cmdchar: c_int, nomove: bool) -> bool {
    unsafe {
        static disabled_redraw: GlobalCell<bool> = GlobalCell::new(false);

        check_spell_redraw();

        let temp = (*curwin.get()).w_cursor.col;
        if disabled_redraw.get() {
            (*RedrawingDisabled.ptr()) -= 1;
            disabled_redraw.set(false);
        }

        let single_char_insert = cmdchar == 'r' as c_int || cmdchar == 'v' as c_int;
        if !arrow_used.get() {
            // Don't append the ESC for `r<CR>` and `grx`.
            if !single_char_insert {
                AppendToRedobuff(ESC_STR.as_ptr());
            }

            // Repeating an insert may take a long time; check for an
            // interrupt now and then.
            if *count > 0 {
                line_breakcheck();
                if got_int.get() {
                    *count = 0;
                }
            }

            *count -= 1;
            if *count > 0 {
                // Repeat what was typed.  Vi repeats the insert without
                // replacing characters.
                if !vim_strchr(p_cpo.get(), CPO_REPLCNT).is_null() {
                    (*State.ptr()) &= !REPLACE_FLAG;
                }
                start_redo_ins();
                if single_char_insert {
                    stuffRedoReadbuff(ESC_STR.as_ptr()); // no ESC in the redo buffer
                }
                (*RedrawingDisabled.ptr()) += 1;
                disabled_redraw.set(true);
                return false;
            }
            stop_insert(&raw mut (*curwin.get()).w_cursor, true_0, nomove as c_int);
            undisplay_dollar();
        }

        if !single_char_insert {
            ins_apply_autocmds(EVENT_INSERTLEAVEPRE);
        }

        // When an auto-indent was removed, curswant stays after the indent.
        if restart_edit.get() == NUL && temp == (*curwin.get()).w_cursor.col {
            (*curwin.get()).w_set_curswant = true_0;
        }

        // Remember the last Insert position in the `'^` mark (`RESET_FMARK`).
        if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPJUMPS as c_int == 0 {
            let view = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
            let fm = &raw mut (*curbuf.get()).b_last_insert;
            free_fmark(*fm);
            (*fm).mark = (*curwin.get()).w_cursor;
            (*fm).fnum = (*curbuf.get()).handle;
            (*fm).timestamp = os_time();
            (*fm).view = view;
            (*fm).additional_data = ::core::ptr::null_mut();
        }

        // The cursor should end up on the last inserted character.  Not for
        // CTRL-O, unless it is past the end of the line.
        if !nomove
            && ((*curwin.get()).w_cursor.col != 0 || (*curwin.get()).w_cursor.coladd > 0)
            && (restart_edit.get() == NUL || (gchar_cursor() == NUL && !VIsual_active.get()))
            && !revins_on.get()
        {
            if (*curwin.get()).w_cursor.coladd > 0
                || get_ve_flags(curwin.get()) == kOptVeFlagAll as ::core::ffi::c_uint
            {
                oneleft();
                if restart_edit.get() != NUL {
                    (*curwin.get()).w_cursor.coladd += 1;
                }
            } else {
                (*curwin.get()).w_cursor.col -= 1;
                (*curwin.get()).w_valid &= !(VALID_WCOL | VALID_VIRTCOL);
                // Correct the cursor for a multi-byte character.
                mb_adjust_cursor();
            }
        }

        State.set(MODE_NORMAL);
        may_trigger_modechanged();
        // The cursor needs positioning again when it is on a TAB, and when
        // the line carries inline virtual text.
        if gchar_cursor() == TAB || buf_meta_total(curbuf.get(), kMTMetaInline) > 0 {
            (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        }

        setmouse();
        ui_cursor_shape(); // may show a different cursor shape

        // While recording, and for CTRL-O, the new mode has to be displayed;
        // otherwise the mode message is removed.
        if reg_recording.get() != 0 || restart_edit.get() != NUL {
            showmode();
        } else if p_smd.get() != 0
            && (got_int.get() || !skip_showmode())
            && !(p_ch.get() == 0 && !ui_has(kUIMessages))
        {
            unshowmode(false);
        }

        true // exit Insert mode
    }
}

/// CTRL-_ : toggle 'revins', and move to the end of the reverse-inserted
/// text.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_ctrl_() {
    unsafe {
        if revins_on.get() && revins_chars.get() != 0 && revins_scol.get() >= 0 {
            // Upstream's `while (gchar_cursor() != NUL && revins_chars--)`:
            // the decrement runs on the iteration that ends the loop too, so
            // exhausting the count leaves `revins_chars` at -1.
            while gchar_cursor() != NUL {
                let n = revins_chars.get();
                revins_chars.set(n - 1);
                if n == 0 {
                    break;
                }
                (*curwin.get()).w_cursor.col += 1;
            }
        }

        p_ri.set((p_ri.get() == 0) as c_int);
        revins_on.set(State.get() == MODE_INSERT && p_ri.get() != 0);
        if revins_on.get() {
            revins_scol.set((*curwin.get()).w_cursor.col);
            (*revins_legal.ptr()) += 1;
            revins_chars.set(0);
            undisplay_dollar();
        } else {
            revins_scol.set(-1);
        }
        showmode();
    }
}

/// `<Insert>` in Insert mode: toggle between Insert and Replace.
///
/// `replace_state` is the Replace mode to go *to* -- `MODE_REPLACE` or
/// `MODE_VREPLACE`, depending on how the insert was started.
///
/// # Safety
/// Must run with a live `curbuf`.
pub(crate) unsafe fn ins_insert(replace_state: c_int) {
    unsafe {
        set_vim_var_string(
            VV_INSERTMODE,
            if State.get() & REPLACE_FLAG != 0 {
                c"i".as_ptr()
            } else if replace_state == MODE_VREPLACE {
                c"v".as_ptr()
            } else {
                c"r".as_ptr()
            },
            1,
        );
        ins_apply_autocmds(EVENT_INSERTCHANGE);

        if State.get() & REPLACE_FLAG != 0 {
            State.set(MODE_INSERT | State.get() & MODE_LANGMAP);
        } else {
            State.set(replace_state | State.get() & MODE_LANGMAP);
        }
        may_trigger_modechanged();
        AppendCharToRedobuff(K_INS);
        showmode();
        ui_cursor_shape();
    }
}

/// CTRL-O: leave Insert mode for exactly one Normal-mode command.
///
/// `restart_edit` holds the letter that brings *this* mode back afterwards,
/// and `ins_at_eol` remembers whether the cursor was past the last character
/// so it can be put back there.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_ctrl_o() {
    unsafe {
        restart_VIsual_select.set(0);
        restart_edit.set(if State.get() & VREPLACE_FLAG != 0 {
            'V' as c_int
        } else if State.get() & REPLACE_FLAG != 0 {
            'R' as c_int
        } else {
            'I' as c_int
        });
        ins_at_eol.set(if virtual_active(curwin.get()) {
            false
        } else {
            gchar_cursor() == NUL
        });
    }
}

/// Whether the next character may trigger a 'cindent' re-indent.
pub(crate) fn get_can_cindent() -> bool {
    can_cindent.get()
}

/// Say whether the next character may trigger a 'cindent' re-indent.
pub(crate) fn set_can_cindent(val: bool) {
    can_cindent.set(val);
}
