//! "The next character is not a command": CTRL-V, CTRL-K, CTRL-E and
//! CTRL-Y.
//!
//! [`ins_ctrl_v`] and [`get_literal`] are CTRL-V: show a `^`, read one more
//! key, and insert it as itself -- or, if digits follow, as the decimal,
//! octal, hex or Unicode value they spell.  [`ins_digraph`] is CTRL-K, the
//! same shape over two characters and the digraph table.  [`insert_special`]
//! is the shared tail that inserts the result, which for a special key means
//! inserting its `<Key>` *name* rather than the key.
//!
//! [`ins_copychar`] and [`ins_ctrl_ey`] are CTRL-E and CTRL-Y, which copy the
//! character from the line below or above at the same *screen* column, and
//! [`redo_literal`] records what all of them produced so that `.` repeats the
//! character rather than the sequence that spelled it.
//!
//! The `^`/`?` these show at the cursor comes from `edit_putchar`, so every
//! path that draws one has to unput it again before inserting -- which is
//! what `did_putchar` tracks in three separate places here.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::{FAIL, OK};

/// Handle a CTRL-V or CTRL-Q typed in Insert mode.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_ctrl_v() {
    unsafe {
        // May need to redraw now that no more characters are available.
        ins_redraw(false);

        let mut did_putchar = false;
        if redrawing() && !char_avail() {
            edit_putchar('^' as c_int, true);
            did_putchar = true;
        }
        AppendToRedobuff(CTRL_V_STR.as_ptr());
        add_to_showcmd_c(Ctrl_V);

        // Do not fold the modifiers into the key for CTRL-SHIFT-V.
        let c = get_literal(mod_mask.get() & MOD_MASK_SHIFT != 0);
        if did_putchar {
            // When the line fits in 'columns' the `^` is at the start of the
            // next line and the redraw will not have removed it.
            edit_unputchar();
        }
        clear_showcmd();
        insert_special(c, true_0, true_0);
        (*revins_chars.ptr()) += 1;
        (*revins_legal.ptr()) += 1;
    }
}

/// Read the next character literally.
///
/// A one, two or three digit decimal number is read as a byte value; `x`/`X`
/// switches to two hex digits, `o`/`O` to three octal ones, and `u`/`U` to
/// four or eight hex digits naming a codepoint (the only spelling that can
/// answer above 255).  If fewer digits than the maximum arrive, the character
/// that ended the run is given back with `vungetc`.
///
/// `no_simplify` keeps the modifiers out of the key, which is CTRL-SHIFT-V.
///
/// The three mode flags are deliberately *not* exclusive: `CTRL-V x o 7`
/// sets both `hex` and `octal`, and every test below asks about `hex` first,
/// so hex wins.  An enum here would be a behaviour change.
///
/// # Safety
/// Must run on the main thread; reads from the typeahead.
pub(crate) unsafe fn get_literal(no_simplify: bool) -> c_int {
    unsafe {
        if got_int.get() {
            return Ctrl_C;
        }

        let mut hex = false;
        let mut octal = false;
        let mut unicode = 0;

        (*no_mapping.ptr()) += 1; // don't map the next key hits
        let mut cc = 0;
        let mut i = 0;
        let mut nc;
        loop {
            nc = plain_vgetc();
            if !no_simplify {
                nc = merge_modifiers(nc, mod_mask.ptr());
            }
            if mod_mask.get() & !MOD_MASK_SHIFT != 0 {
                // A character with a non-Shift modifier cannot be part of
                // i_CTRL-V_digit.
                break;
            }
            // MB_BYTE2LEN_CHECK
            let byte_len = if nc < 0 || nc > 255 {
                1
            } else {
                utf8len_tab[nc as usize] as c_int
            };
            if State.get() & MODE_CMDLINE == 0 && byte_len == 1 {
                add_to_showcmd(nc);
            }

            if nc == 'x' as c_int || nc == 'X' as c_int {
                hex = true;
            } else if nc == 'o' as c_int || nc == 'O' as c_int {
                octal = true;
            } else if nc == 'u' as c_int || nc == 'U' as c_int {
                unicode = nc;
            } else {
                if hex || unicode != 0 {
                    if !ascii_isxdigit(nc) {
                        break;
                    }
                    cc = cc * 16 + hex2nr(nc);
                } else if octal {
                    if nc < '0' as c_int || nc > '7' as c_int {
                        break;
                    }
                    cc = cc * 8 + nc - '0' as c_int;
                } else {
                    if !ascii_isdigit(nc) {
                        break;
                    }
                    cc = cc * 10 + nc - '0' as c_int;
                }
                i += 1;
            }

            if cc > 255 && unicode == 0 {
                cc = 255; // limit the range to 0-255
            }
            nc = 0;

            // How many digits this spelling takes.
            let enough = if hex {
                i >= 2
            } else if unicode != 0 {
                (unicode == 'u' as c_int && i >= 4) || (unicode == 'U' as c_int && i >= 8)
            } else {
                i >= 3
            };
            if enough {
                break;
            }
        }

        if i == 0 {
            // No number was entered: the key itself is the answer.  NUL is
            // stored as NL.
            cc = if nc == K_ZERO { '\n' as c_int } else { nc };
            nc = 0;
        }
        if cc == 0 {
            cc = '\n' as c_int; // NUL is stored as NL
        }

        (*no_mapping.ptr()) -= 1;
        if nc != 0 {
            vungetc(nc);
            // A character typed with i_CTRL-V_digit cannot have modifiers.
            mod_mask.set(0);
        }
        got_int.set(false); // CTRL-C after CTRL-V is not an interrupt
        cc
    }
}

/// Insert a character, taking care of special keys and `mod_mask`.
///
/// A special key is inserted as its `<Key>` *name*: everything up to the last
/// `>` goes in with `ins_str` (so Replace mode does not overwrite characters
/// with it) and the `>` itself is inserted as an ordinary character below.
/// `mod_mask` is only used for special keys -- otherwise `<S-Space>` and
/// friends would appear -- unless `allow_modmask` says otherwise, which the
/// Command key always does.
///
/// `ctrlv` says `c` was typed just after CTRL-V.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn insert_special(mut c: c_int, mut allow_modmask: c_int, mut ctrlv: c_int) {
    unsafe {
        if mod_mask.get() & MOD_MASK_CMD != 0 {
            // The Command key never produces a normal key.
            allow_modmask = true_0;
        }
        if c < 0 || (mod_mask.get() != 0 && allow_modmask != 0) {
            let p = get_special_key_name(c, mod_mask.get());
            let len = strlen(p) as c_int;
            c = *p.offset((len - 1) as isize) as uint8_t as c_int;
            if len > 2 {
                if stop_arrow() == FAIL {
                    return;
                }
                *p.offset((len - 1) as isize) = NUL as c_char;
                ins_str(p, (len - 1) as size_t);
                AppendToRedobuffLit(p, -1);
                ctrlv = false_0;
            }
        }
        if stop_arrow() == OK {
            insertchar(
                c,
                if ctrlv != 0 {
                    INSCHAR_CTRLV as c_int
                } else {
                    0
                },
                -1,
            );
        }
    }
}

/// Put a character in the redo buffer, for just after a CTRL-V.
///
/// A digit has to go in as three digits, or the redo would read it as the
/// start of a longer `i_CTRL-V_digit` sequence.
///
/// # Safety
/// Must run on the main thread.
pub(crate) unsafe fn redo_literal(c: c_int) {
    unsafe {
        if ascii_isdigit(c) {
            let mut buf: [c_char; 10] = [0; 10];
            vim_snprintf(buf.as_mut_ptr(), buf.len(), c"%03d".as_ptr(), c);
            AppendToRedobuff(buf.as_mut_ptr());
        } else {
            AppendCharToRedobuff(c);
        }
    }
}

/// Handle a digraph (CTRL-K) in Insert mode.
///
/// Answers the character still to be inserted, or NUL when there is nothing
/// left to do -- which is the case when either key was `<Esc>`, and when the
/// first key was a special key (which [`insert_special`] has then already
/// inserted).
///
/// # Safety
/// Must run with a live `curwin`; reads from the typeahead.
pub(crate) unsafe fn ins_digraph() -> c_int {
    unsafe {
        let mut did_putchar = false;
        pc_status.set(PutChar::Unset);
        if redrawing() && !char_avail() {
            ins_redraw(false);
            edit_putchar('?' as c_int, true);
            did_putchar = true;
            add_to_showcmd_c(Ctrl_K);
        }

        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let mut c = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        if did_putchar {
            // If the line fits in 'columns' the `?` is at the start of the
            // next line and the redraw will not have removed it.
            edit_unputchar();
        }

        if c < 0 || mod_mask.get() != 0 {
            clear_showcmd();
            insert_special(c, true_0, false_0);
            return NUL;
        }

        if c != ESC {
            did_putchar = false;
            if redrawing() && !char_avail() {
                ins_redraw(false);
                if char2cells(c) == 1 {
                    ins_redraw(false);
                    edit_putchar(c, true);
                    did_putchar = true;
                }
                add_to_showcmd_c(c);
            }

            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
            let cc = plain_vgetc();
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
            if did_putchar {
                edit_unputchar();
            }

            if cc != ESC {
                AppendToRedobuff(CTRL_V_STR.as_ptr());
                c = digraph_get(c, cc, true);
                clear_showcmd();
                return c;
            }
        }

        clear_showcmd();
        NUL
    }
}

/// The character in line `lnum` at the cursor's *screen* column.
///
/// This is CTRL-E and CTRL-Y's "copy from the line below/above": the answer
/// is found by walking that line's characters, adding widths, until the
/// cursor's virtual column is reached.  Answers NUL -- and beeps -- when the
/// line does not exist or is too short.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_copychar(lnum: linenr_T) -> c_int {
    unsafe {
        if lnum < 1 || lnum > (*curbuf.get()).b_ml.ml_line_count {
            vim_beep(kOptBoFlagCopy as ::core::ffi::c_uint);
            return NUL;
        }

        // Try to advance to the cursor column.
        validate_virtcol(curwin.get());
        let end_vcol = (*curwin.get()).w_virtcol;
        let line = ml_get(lnum);

        let mut csarg = CharsizeArg::default();
        let cstype = init_charsize_arg(&mut csarg, curwin.get(), lnum, line);
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        let mut vcol = 0;
        while vcol < end_vcol && *ci.ptr as c_int != NUL {
            vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            if vcol > end_vcol {
                break;
            }
            ci = utfc_next(ci);
        }

        let c = if ci.chr.value < 0 {
            *ci.ptr as uint8_t as c_int
        } else {
            ci.chr.value as c_int
        };
        if c == NUL {
            vim_beep(kOptBoFlagCopy as ::core::ffi::c_uint);
        }
        c
    }
}

/// CTRL-Y or CTRL-E typed in Insert mode.
///
/// In `CTRL-X CTRL-E`/`CTRL-X CTRL-Y` mode they scroll the window; otherwise
/// they copy the character above or below with [`ins_copychar`] and insert
/// it, with 'textwidth' turned off so the copy cannot trigger a wrap.
///
/// Answers the key to be recorded: the character itself when nothing was
/// inserted, and CTRL-V when it was.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_ctrl_ey(tc: c_int) -> c_int {
    unsafe {
        let mut c = tc;
        if ctrl_x_mode_scroll() {
            if c == Ctrl_Y {
                scrolldown_clamp();
            } else {
                scrollup_clamp();
            }
            redraw_later(curwin.get(), UPD_VALID);
            return c;
        }

        c = ins_copychar((*curwin.get()).w_cursor.lnum + if c == Ctrl_Y { -1 } else { 1 });
        if c == NUL {
            return c;
        }

        // A non-alphanumeric byte has to be recorded literally, or the redo
        // would read it as a command.
        if c < 256 && *(*__ctype_b_loc()).offset(c as isize) & _ISalnum == 0 {
            AppendToRedobuff(CTRL_V_STR.as_ptr());
        }
        let tw_save = (*curbuf.get()).b_p_tw;
        (*curbuf.get()).b_p_tw = -1;
        insert_special(c, true_0, false_0);
        (*curbuf.get()).b_p_tw = tw_save;
        (*revins_chars.ptr()) += 1;
        (*revins_legal.ptr()) += 1;
        auto_format(false, true);
        Ctrl_V
    }
}
