//! insertchar — character insertion with formatting and comment handling
//!
//! Ported from `edit.c` `insertchar()`. Handles:
//! - textwidth line-breaking via `internal_format` / `fex_format`
//! - end-comment-pending leader replacement (delegated to C helper)
//! - fast-path batching of ASCII input
//! - single-character insertion (multibyte or plain)

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]

use std::ffi::{c_char, c_int};

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

// ============================================================================
// C accessor / helper functions
// ============================================================================

extern "C" {
    // -- cursor / state accessors --
    fn nvim_get_State() -> c_int;
    fn nvim_set_ins_need_undo(val: c_int);
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    fn nvim_get_Insstart_lnum() -> LinenrT;
    fn nvim_get_Insstart_textlen() -> ColnrT;
    fn nvim_get_Insstart_blank_vcol() -> ColnrT;

    // -- did_ai/did_si/can_si/can_si_back --
    fn nvim_get_did_ai() -> bool;
    fn nvim_set_did_ai(val: bool);
    fn nvim_set_did_si(val: bool);
    fn nvim_set_can_si(val: bool);
    fn nvim_set_can_si_back(val: bool);

    // -- end_comment_pending --
    fn nvim_get_end_comment_pending() -> c_int;
    fn nvim_set_end_comment_pending(val: c_int);

    // -- no_abbr --
    fn nvim_get_no_abbr() -> c_int;

    // -- p_ri (rightleft) --
    fn nvim_get_p_ri() -> c_int;

    // -- textformat --
    fn nvim_comp_textwidth(force_format: c_int) -> c_int;
    fn get_nolist_virtcol() -> c_int;
    fn char2cells(c: c_int) -> c_int;
    fn gchar_cursor() -> c_int;
    fn nvim_internal_format(
        textwidth: c_int,
        second_indent: c_int,
        flags: c_int,
        format_only: c_int,
        c: c_int,
    );
    fn fex_format(lnum: LinenrT, count: c_long, c: c_int) -> c_int;

    // -- format option checks --
    fn nvim_has_format_option(c: c_int) -> bool;

    // -- formatexpr availability --
    fn nvim_curbuf_has_b_p_fex() -> bool;

    // -- end_comment_pending comment replacement (complex C logic) --
    fn nvim_handle_end_comment_pending(c: c_int);

    // -- input / char --
    fn vpeekc() -> c_int;
    fn vgetc() -> c_int;
    fn nvim_MB_BYTE2LEN_CHECK(c: c_int) -> c_int;
    fn nvim_byte2cells(c: c_int) -> c_int;
    fn do_digraph(c: c_int) -> c_int;
    #[link_name = "test_disable_char_avail"]
    static mut nvim_test_disable_char_avail: bool;

    // -- insertion --
    fn nvim_ins_str(p: *const c_char, len: usize);
    fn ins_char(c: c_int);
    fn ins_char_bytes(buf: *const c_char, charlen: usize);

    // -- redo --
    fn AppendToRedobuffLit(s: *const c_char, len: c_int);
    fn AppendCharToRedobuff(c: c_int);
    fn rs_redo_literal(c: c_int);

    // -- events --
    fn nvim_has_event_insertcharpre() -> c_int;

    // -- cindent --
    fn cindent_on() -> bool;

    // -- vim_iswordc --
    fn vim_iswordc(c: c_int) -> bool;

    // -- utf --
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut u8) -> c_int;
}

// Long type (matches C `long`)
#[allow(non_camel_case_types)]
type c_long = i64;

// ============================================================================
// Constants (verified against C headers)
// ============================================================================

/// `REPLACE_FLAG` from `state_defs.h`
const REPLACE_FLAG: c_int = 0x10;
/// `VREPLACE_FLAG` from `state_defs.h`
const VREPLACE_FLAG: c_int = 0x20;

/// `INSCHAR_FORMAT` from `edit.h`
const INSCHAR_FORMAT: c_int = 1;
/// `INSCHAR_CTRLV` from `edit.h`
const INSCHAR_CTRLV: c_int = 4;
/// `INSCHAR_NO_FEX` from `edit.h`
const INSCHAR_NO_FEX: c_int = 8;

/// `FO_INS_BLANK` — `'B'` format option
const FO_INS_BLANK: c_int = b'B' as c_int;
/// `FO_INS_LONG` — `'l'` format option
const FO_INS_LONG: c_int = b'l' as c_int;

/// `INPUT_BUFLEN` — max chars batched from input
const INPUT_BUFLEN: usize = 100;
/// `MB_MAXCHAR` — max bytes in a multibyte character
const MB_MAXCHAR: usize = 21;

// ============================================================================
// insertchar
// ============================================================================

/// Insert a character at the cursor position.
///
/// `c`:             character to insert, or `NUL` for format-only.
/// `flags`:         `INSCHAR_FORMAT` etc.
/// `second_indent`: indent for second line if >= 0.
///
/// # Safety
/// Accesses many C globals.
#[unsafe(export_name = "insertchar")]
pub unsafe extern "C" fn rs_insertchar(c: c_int, flags: c_int, second_indent: c_int) {
    let force_format = flags & INSCHAR_FORMAT;
    let textwidth = nvim_comp_textwidth(force_format);
    let fo_ins_blank = nvim_has_format_option(FO_INS_BLANK);

    // Try to break the line in two or more pieces when conditions are met.
    if textwidth > 0
        && (force_format != 0
            || (!(ascii_iswhite(c)
                || (nvim_get_State() & REPLACE_FLAG != 0)
                    && (nvim_get_State() & VREPLACE_FLAG == 0)
                    && gchar_cursor() != 0) // != NUL
                && (nvim_curwin_get_cursor_lnum() != nvim_get_Insstart_lnum()
                    || ((!has_fo_ins_long() || nvim_get_Insstart_textlen() <= textwidth as ColnrT)
                        && (!fo_ins_blank
                            || nvim_get_Insstart_blank_vcol() <= textwidth as ColnrT)))))
    {
        // Format with 'formatexpr' when it's set. Use internal formatting
        // when 'formatexpr' isn't set or it returns non-zero.
        let mut do_internal = true;
        let virtcol = get_nolist_virtcol() + char2cells(if c != 0 { c } else { gchar_cursor() });

        if nvim_curbuf_has_b_p_fex()
            && (flags & INSCHAR_NO_FEX == 0)
            && (force_format != 0 || virtcol > textwidth)
        {
            do_internal = fex_format(nvim_curwin_get_cursor_lnum(), 1, c) != 0;
            // It may be required to save for undo again, e.g. when setline() was called.
            nvim_set_ins_need_undo(1);
        }
        if do_internal {
            nvim_internal_format(textwidth, second_indent, flags, 0, c);
        }
    }

    if c == 0 {
        // only formatting was wanted
        return;
    }

    // Check whether this character should end a comment.
    if nvim_get_did_ai() && c == nvim_get_end_comment_pending() {
        nvim_handle_end_comment_pending(c);
    }
    nvim_set_end_comment_pending(0 /* NUL */);

    nvim_set_did_ai(false);
    nvim_set_did_si(false);
    nvim_set_can_si(false);
    nvim_set_can_si_back(false);

    // If there's any pending input, grab up to INPUT_BUFLEN at once.
    // This speeds up normal text input considerably.
    // Don't do this when 'cindent' or 'indentexpr' is set, because we might
    // need to re-indent at a ':', or any other character (but not when 'paste' is set).
    // Don't do this when there is an `InsertCharPre` autocommand defined,
    // because we need to fire the event for every character.
    // Do the check for `InsertCharPre` before the call to `vpeekc()` because the
    // `InsertCharPre` autocommand could change the input buffer.
    if !is_special(c)
        && utf_char2len(c) == 1
        && nvim_has_event_insertcharpre() == 0
        && !nvim_test_disable_char_avail
        && vpeekc() != 0 // != NUL
        && nvim_get_State() & REPLACE_FLAG == 0
        && !cindent_on()
        && nvim_get_p_ri() == 0
    {
        let mut buf = [0u8; INPUT_BUFLEN + 1];
        let mut virtcol: ColnrT = 0;

        buf[0] = c as u8;
        let mut i: usize = 1;
        if textwidth > 0 {
            virtcol = get_nolist_virtcol() as ColnrT;
        }
        // Stop the string when:
        // - no more chars available
        // - finding a special character (command key)
        // - buffer is full
        // - running into the 'textwidth' boundary
        // - need to check for abbreviation: A non-word char after a word-char
        loop {
            let nc = vpeekc();
            if nc == 0 {
                // NUL: no more chars
                break;
            }
            if is_special(nc) {
                break;
            }
            if nvim_MB_BYTE2LEN_CHECK(nc) != 1 {
                break;
            }
            if i >= INPUT_BUFLEN {
                break;
            }
            if textwidth != 0 {
                let cells = nvim_byte2cells(c_int::from(buf[i - 1]));
                virtcol = virtcol.saturating_add(cells as ColnrT);
                if virtcol >= textwidth as ColnrT {
                    break;
                }
            }
            let prev_word = vim_iswordc(c_int::from(buf[i - 1]));
            let nc_word = vim_iswordc(nc);
            if nvim_get_no_abbr() == 0 && !nc_word && prev_word {
                break;
            }
            let nc = vgetc();
            buf[i] = nc as u8;
            i += 1;
        }

        do_digraph(-1); // clear digraphs
        do_digraph(c_int::from(buf[i - 1])); // may be the start of a digraph
        buf[i] = 0; // NUL-terminate

        nvim_ins_str(buf.as_ptr().cast::<c_char>(), i);

        let start: usize = if flags & INSCHAR_CTRLV != 0 {
            rs_redo_literal(c_int::from(buf[0]));
            1
        } else {
            0
        };
        if buf[start] != 0 {
            AppendToRedobuffLit(buf.as_ptr().add(start).cast::<c_char>(), -1);
        }
    } else {
        let cc = utf_char2len(c);
        if cc > 1 {
            let mut buf = [0u8; MB_MAXCHAR + 1];
            utf_char2bytes(c, buf.as_mut_ptr());
            buf[cc as usize] = 0;
            ins_char_bytes(buf.as_ptr().cast::<c_char>(), cc as usize);
            AppendCharToRedobuff(c);
        } else {
            ins_char(c);
            if flags & INSCHAR_CTRLV != 0 {
                rs_redo_literal(c);
            } else {
                AppendCharToRedobuff(c);
            }
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Returns true if `c` is a whitespace character (space or tab).
fn ascii_iswhite(c: c_int) -> bool {
    c == c_int::from(b' ') || c == c_int::from(b'\t')
}

/// Returns true if `c` is a special (non-printable command) key.
/// Matches the C macro `ISSPECIAL(c)` which is `(c) < 0`.
const fn is_special(c: c_int) -> bool {
    c < 0
}

/// Returns true if `has_format_option(FO_INS_LONG)` (format option `'l'`).
unsafe fn has_fo_ins_long() -> bool {
    nvim_has_format_option(FO_INS_LONG)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_iswhite() {
        assert!(ascii_iswhite(c_int::from(b' ')));
        assert!(ascii_iswhite(c_int::from(b'\t')));
        assert!(!ascii_iswhite(c_int::from(b'a')));
        assert!(!ascii_iswhite(0));
    }

    #[test]
    fn test_is_special() {
        assert!(is_special(-1));
        assert!(is_special(-256));
        assert!(!is_special(0));
        assert!(!is_special(127));
    }

    #[test]
    fn test_constants() {
        assert_eq!(INSCHAR_FORMAT, 1);
        assert_eq!(INSCHAR_CTRLV, 4);
        assert_eq!(INSCHAR_NO_FEX, 8);
        assert_eq!(INPUT_BUFLEN, 100);
    }
}
