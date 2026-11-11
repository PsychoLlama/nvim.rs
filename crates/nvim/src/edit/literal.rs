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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::cstr::{self, byte_at};
use crate::keycodes::ModMask;
use crate::keycodes::SpecialKey;
use crate::mbyte::Codepoint;
use crate::mbyte::{c_char_info, c_next_cluster};
use crate::os::cshim::ctype_bits;
use crate::types::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::guard::Keys;

/// Handle a CTRL-V or CTRL-Q typed in Insert mode.
pub(crate) fn ins_ctrl_v() {
    ins_redraw(false);

    let mut did_putchar = false;
    if redrawing() && !char_avail() {
        edit_putchar(Key::ascii(b'^'), true);
        did_putchar = true;
    }
    append_to_redobuff(CTRL_V_STR.to_bytes());
    add_to_showcmd_c(Key::CTRL_V);

    // Do not fold the modifiers into the key for CTRL-SHIFT-V.
    let c = get_literal(MOD_MASK.get().has(ModMask::SHIFT));
    if did_putchar {
        // When the line fits in 'columns' the `^` is at the start of the
        // next line and the redraw will not have removed it.
        edit_unputchar();
    }
    clear_showcmd();
    insert_special(c, true, true);
    REVINS_CHARS.set(REVINS_CHARS.get() + 1);
    REVINS_LEGAL.set(REVINS_LEGAL.get() + 1);
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
pub(crate) fn get_literal(no_simplify: bool) -> Key {
    if GOT_INT.get() {
        return Key::CTRL_C;
    }

    let mut hex = false;
    let mut octal = false;
    let mut unicode = 0;

    let unmapped = Keys::unmapped(); // don't map the next key hits
    // The number spelled so far: at most eight hex digits, so it fits.
    let mut cc: c_int = 0;
    let mut i = 0;
    let mut nc;
    loop {
        nc = plain_vgetc();
        if !no_simplify {
            nc = merge_mod_mask(nc);
        }
        if !MOD_MASK.get().without(ModMask::SHIFT).is_empty() {
            // A character with a non-Shift modifier cannot be part of
            // i_CTRL-V_digit.
            break;
        }
        // MB_BYTE2LEN_CHECK
        let byte_len = nc.as_byte().map_or(1, |b| utf8len_tab[usize::from(b)]);
        if STATE.get() & MODE_CMDLINE == 0 && byte_len == 1 {
            add_to_showcmd(nc);
        }

        // The digit `nc` is, in the base the spelling has chosen.
        let digit = nc.as_byte().filter(u8::is_ascii_digit).map(|b| b - b'0');
        if nc.is('x') || nc.is('X') {
            hex = true;
        } else if nc.is('o') || nc.is('O') {
            octal = true;
        } else if nc.is('u') || nc.is('U') {
            unicode = nc.as_byte().unwrap_or(0);
        } else {
            if hex || unicode != 0 {
                if !ascii_isxdigit(nc.code()) {
                    break;
                }
                cc = cc * 16 + nc.as_byte().map_or(0, hex2nr);
            } else if octal {
                let Some(d) = digit.filter(|&d| d < 8) else {
                    break;
                };
                cc = cc * 8 + c_int::from(d);
            } else {
                let Some(d) = digit else {
                    break;
                };
                cc = cc * 10 + c_int::from(d);
            }
            i += 1;
        }

        if cc > 255 && unicode == 0 {
            cc = 255; // limit the range to 0-255
        }
        nc = Key::NUL;

        // How many digits this spelling takes.
        let enough = if hex {
            i >= 2
        } else if unicode != 0 {
            (unicode == b'u' && i >= 4) || (unicode == b'U' && i >= 8)
        } else {
            i >= 3
        };
        if enough {
            break;
        }
    }

    let mut key = Key::from_code(cc);
    if i == 0 {
        // No number was entered: the key itself is the answer.  NUL is
        // stored as NL.
        key = if nc == SpecialKey::Zero { Key::NL } else { nc };
        nc = Key::NUL;
    }
    if key.is_nul() {
        key = Key::NL; // NUL is stored as NL
    }

    drop(unmapped);
    if !nc.is_nul() {
        vungetc(nc);
        // A character typed with i_CTRL-V_digit cannot have modifiers.
        MOD_MASK.set(ModMask::NONE);
    }
    GOT_INT.set(false); // CTRL-C after CTRL-V is not an interrupt
    key
}

/// Insert a character, taking care of special keys and `MOD_MASK`.
///
/// A special key is inserted as its `<Key>` *name*: everything up to the last
/// `>` goes in with `ins_str` (so Replace mode does not overwrite characters
/// with it) and the `>` itself is inserted as an ordinary character below.
/// `MOD_MASK` is only used for special keys -- otherwise `<S-Space>` and
/// friends would appear -- unless `allow_modmask` says otherwise, which the
/// Command key always does.
///
/// `ctrlv` says `c` was typed just after CTRL-V.
pub(crate) fn insert_special(mut c: Key, mut allow_modmask: bool, mut ctrlv: bool) {
    if MOD_MASK.get().has(ModMask::CMD) {
        // The Command key never produces a normal key.
        allow_modmask = true;
    }
    if c.is_special() || (!MOD_MASK.get().is_empty() && allow_modmask) {
        let name = get_special_key_name(c, MOD_MASK.get());
        let name = cstr::until_nul(&name);
        let (&last, head) = name.split_last().expect("a key name is never empty");
        c = Key::byte(last);
        if name.len() > 2 {
            if stop_arrow().is_err() {
                return;
            }
            ins_str(head);
            append_to_redobuff_literally(head);
            ctrlv = false;
        }
    }
    if stop_arrow().is_ok() {
        let flags = if ctrlv { INSCHAR_CTRLV } else { 0 };
        insertchar(c, flags, -1);
    }
}

/// Put a character in the redo buffer, for just after a CTRL-V.
///
/// A digit has to go in as three digits, or the redo would read it as the
/// start of a longer `i_CTRL-V_digit` sequence.
pub(crate) fn redo_literal(c: Key) {
    // The digit's own number: `5` is recorded as `053`.
    match c.as_byte().filter(u8::is_ascii_digit) {
        Some(digit) => append_to_redobuff(format!("{digit:03}").as_bytes()),
        None => append_to_redobuff_char(c),
    }
}

/// Handle a digraph (CTRL-K) in Insert mode.
///
/// Answers the character still to be inserted, or NUL when there is nothing
/// left to do -- which is the case when either key was `<Esc>`, and when the
/// first key was a special key (which [`insert_special`] has then already
/// inserted).
pub(crate) fn ins_digraph() -> Key {
    let mut did_putchar = false;
    PC_STATUS.set(PutChar::Unset);
    if redrawing() && !char_avail() {
        ins_redraw(false);
        edit_putchar(Key::ascii(b'?'), true);
        did_putchar = true;
        add_to_showcmd_c(Key::CTRL_K);
    }

    let c = {
        let _raw_key = Keys::unmapped_with_codes();
        plain_vgetc()
    };
    if did_putchar {
        // If the line fits in 'columns' the `?` is at the start of the
        // next line and the redraw will not have removed it.
        edit_unputchar();
    }

    if c.is_special() || !MOD_MASK.get().is_empty() {
        clear_showcmd();
        insert_special(c, true, false);
        return Key::NUL;
    }

    if c != Key::ESC {
        did_putchar = false;
        if redrawing() && !char_avail() {
            ins_redraw(false);
            if char2cells(Codepoint::new(c.code())) == 1 {
                ins_redraw(false);
                edit_putchar(c, true);
                did_putchar = true;
            }
            add_to_showcmd_c(c);
        }

        let cc = {
            let _raw_key = Keys::unmapped_with_codes();
            plain_vgetc()
        };
        if did_putchar {
            edit_unputchar();
        }

        if cc != Key::ESC {
            append_to_redobuff(CTRL_V_STR.to_bytes());
            let c = digraph_get(c, cc, true);
            clear_showcmd();
            return c;
        }
    }

    clear_showcmd();
    Key::NUL
}

/// The character in line `lnum` at the cursor's *screen* column.
///
/// This is CTRL-E and CTRL-Y's "copy from the line below/above": the answer
/// is found by walking that line's characters, adding widths, until the
/// cursor's virtual column is reached.  Answers NUL -- and beeps -- when the
/// line does not exist or is too short.
pub(crate) fn ins_copychar(lnum: LineNr) -> Key {
    if lnum < LineNr::ONE || lnum > Buf::current().b_ml.ml_line_count {
        vim_beep(OPT_BO_FLAG_COPY as ::core::ffi::c_uint);
        return Key::NUL;
    }

    // Try to advance to the cursor column.
    validate_virtcol(Win::current());
    let end_vcol = Win::current().w_virtcol;
    // A copy: the width walk is free to read the memline.
    let line = Lines::current().line_copy(lnum);

    let (mut csarg, cstype) = CharsizeArg::new(Win::current(), lnum, &line);
    let mut at = 0;
    let mut chr = c_char_info(&line, 0);
    let mut vcol = 0;
    while vcol < end_vcol.get() && byte_at(&line, at) != b'\0' {
        vcol += win_charsize(cstype, vcol, at, chr.value, &mut csarg).width;
        if vcol > end_vcol.get() {
            break;
        }
        (at, chr) = c_next_cluster(&line, at, chr);
    }

    // An illegal byte is copied as itself.
    let c = if chr.value < 0 {
        Key::byte(byte_at(&line, at))
    } else {
        Key::from_code(chr.value)
    };
    if c.is_nul() {
        vim_beep(OPT_BO_FLAG_COPY as ::core::ffi::c_uint);
    }
    c
}

/// CTRL-Y or CTRL-E typed in Insert mode.
///
/// In `CTRL-X CTRL-E`/`CTRL-X CTRL-Y` mode they scroll the window; otherwise
/// they copy the character above or below with [`ins_copychar`] and insert
/// it, with 'textwidth' turned off so the copy cannot trigger a wrap.
///
/// Answers the key to be recorded: the character itself when nothing was
/// inserted, and CTRL-V when it was.
pub(crate) fn ins_ctrl_ey(tc: Key) -> Key {
    let mut c = tc;
    if ctrl_x_mode_scroll() {
        if c == Key::CTRL_Y {
            scrolldown_clamp();
        } else {
            scrollup_clamp();
        }
        redraw_later(Win::current(), UPD_VALID);
        return c;
    }

    c = ins_copychar(Win::current().w_cursor.lnum + if c == Key::CTRL_Y { -1 } else { 1 });
    if c.is_nul() {
        return c;
    }

    // A non-alphanumeric byte has to be recorded literally, or the redo
    // would read it as a command.
    if c.as_byte().is_some_and(|b| ctype_bits(b) & IS_ALNUM == 0) {
        append_to_redobuff(CTRL_V_STR.to_bytes());
    }
    let tw_save = Buf::current().b_p_tw;
    Buf::current().b_p_tw = -1;
    insert_special(c, true, false);
    Buf::current().b_p_tw = tw_save;
    REVINS_CHARS.set(REVINS_CHARS.get() + 1);
    REVINS_LEGAL.set(REVINS_LEGAL.get() + 1);
    auto_format(false, true);
    Key::CTRL_V
}
