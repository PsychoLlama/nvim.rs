//! Spell navigation: implements `spell_move_to` in Rust.
//!
//! Migrated from `src/nvim/spell.c`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::while_immutable_condition)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Spell checking
    fn spell_check(
        wp: *mut c_void,
        ptr: *mut c_char,
        attrp: *mut c_int,
        capcol: *mut c_int,
        docount: bool,
    ) -> usize;
    fn spell_cat_line(buf: *mut c_char, line: *const c_char, maxlen: c_int);
    fn check_need_cap(wp: *mut c_void, lnum: c_int, col: c_int) -> bool;
    fn no_spell_checking(wp: *mut c_void) -> bool;

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // Strings
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;

    // Cursor and window
    fn nvim_win_get_cursor_lnum(wp: *const c_void) -> c_int;
    fn nvim_win_get_cursor_col(wp: *const c_void) -> c_int;
    fn nvim_win_set_cursor_lnum(wp: *mut c_void, lnum: c_int);
    fn nvim_win_set_cursor_col(wp: *mut c_void, col: c_int);
    fn nvim_win_set_cursor_coladd(wp: *mut c_void, coladd: c_int);

    // Buffer / memline
    fn nvim_win_get_w_buffer(wp: *mut c_void) -> *mut c_void;
    // nvim_spell_win_ml_line_count removed: use rs_spell_win_ml_line_count() below
    #[link_name = "nvim_buf_ml_line_count"]
    fn nvim_spell_buf_ml_line_count(buf: *mut c_void) -> c_int;
    #[link_name = "ml_get_buf"]
    fn nvim_spell_ml_get_buf(buf: *mut c_void, lnum: c_int) -> *mut c_char;
    #[link_name = "ml_get_buf_len"]
    fn nvim_spell_ml_get_buf_len(buf: *mut c_void, lnum: c_int) -> c_int;

    // Options
    fn nvim_get_p_ws() -> c_int;
    fn nvim_spell_win_noplainbuffer(wp: *mut c_void) -> bool;

    // Decoration state (managed in C to avoid DecorState layout issues)
    fn nvim_spell_decor_state_size() -> usize;
    fn nvim_spell_nav_start(wp: *mut c_void, saved_out: *mut c_void);
    fn nvim_spell_restore_decor_state(saved: *mut c_void);
    fn nvim_spell_nav_decor_col(
        wp: *mut c_void,
        lnum: c_int,
        decor_lnum: *mut c_int,
        col: c_int,
    ) -> c_int;

    // Syntax
    #[link_name = "syntax_present"]
    fn nvim_spell_syntax_present(wp: *mut c_void) -> bool;
    // nvim_spell_can_syn_spell removed: use rs_spell_can_syn_spell() below
    #[link_name = "syn_get_id"]
    fn nvim_spell_syn_get_id(
        wp: *mut c_void,
        lnum: c_int,
        col: c_int,
        trans: c_int,
        spellp: *mut bool,
        keep_state: c_int,
    ) -> c_int;

    // Misc
    fn nvim_shortmess_search() -> c_int;
    fn nvim_spell_give_wrap_warning(forward: bool);
    #[link_name = "getwhitecols"]
    fn nvim_spell_getwhitecols_raw(p: *const c_char) -> isize;
    fn line_breakcheck();

    // got_int global
    #[link_name = "got_int"]
    static got_int: bool;
}

// HLF_* values matching C enum (from highlight_defs.h)
const HLF_COUNT: c_int = 76; // MUST be last
const HLF_SPB: c_int = 37; // SpellBad
const HLF_SPR: c_int = 39; // SpellRare

// smt_T values (from spell.h)
const SMT_ALL: c_int = 0;
const SMT_BAD: c_int = 1;
const SMT_RARE: c_int = 2;

// Direction constants (from vim_defs.h)
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

// MAXWLEN from spell_defs.h
const MAXWLEN: usize = 254;

/// Get the ml_line_count of a window's buffer.
/// Replaces nvim_spell_win_ml_line_count shim in spell_shim.c.
///
/// # Safety
/// `wp` must be a valid win_T pointer.
unsafe fn nvim_spell_win_ml_line_count(wp: *mut c_void) -> c_int {
    nvim_spell_buf_ml_line_count(nvim_win_get_w_buffer(wp))
}

/// Check if syntax allows spell checking at a position.
/// Replaces nvim_spell_can_syn_spell shim in spell_shim.c.
///
/// # Safety
/// `wp` must be a valid win_T pointer.
unsafe fn nvim_spell_can_syn_spell(wp: *mut c_void, lnum: c_int, col: c_int) -> bool {
    let mut can_spell = false;
    nvim_spell_syn_get_id(wp, lnum, col, 0, &raw mut can_spell, 0);
    can_spell
}

/// Moves to the next spell error in the window.
///
/// "curline" is false for "[s", "]s", "[S" and "]S".
/// "curline" is true to find word under/after cursor in the same line.
/// For Insert mode completion "dir" is BACKWARD and "curline" is true:
/// move to after badly spelled word before the cursor.
///
/// Returns 0 if not found, length of the badly spelled word otherwise.
///
/// # Safety
///
/// `wp` must be a valid win_T pointer. `attrp` may be null.
#[export_name = "spell_move_to"]
pub unsafe extern "C" fn rs_spell_move_to(
    wp: *mut c_void,
    dir: c_int,
    behaviour: c_int,
    curline: bool,
    attrp: *mut c_int,
) -> usize {
    if no_spell_checking(wp) {
        return 0;
    }

    let mut found_lnum: c_int = 0;
    let mut found_col: c_int = 0;
    let mut found_len: usize = 0;
    let mut attr: c_int = HLF_COUNT;
    let has_syntax = nvim_spell_syntax_present(wp);
    let mut buf: *mut c_char = std::ptr::null_mut();
    let mut buflen: usize = 0;
    let mut skip: c_int = 0;
    let mut capcol: c_int = -1;
    let mut found_one = false;
    let mut wrapped = false;

    let mut ret: usize = 0;

    let mut lnum = nvim_win_get_cursor_lnum(wp);

    // Save and reset decor_state for spell navigation
    let saved_size = nvim_spell_decor_state_size();
    let saved_decor = xmalloc(saved_size);
    let mut decor_lnum: c_int = -1;
    nvim_spell_nav_start(wp, saved_decor);

    'outer: while !got_int {
        let wbuf = nvim_win_get_w_buffer(wp);
        let line = nvim_spell_ml_get_buf(wbuf, lnum);
        let len = nvim_spell_ml_get_buf_len(wbuf, lnum) as usize;

        if buflen < len + MAXWLEN + 2 {
            xfree(buf.cast::<c_void>());
            buflen = len + MAXWLEN + 2;
            buf = xmalloc(buflen).cast::<c_char>();
        }

        // In first line check first word for Capital.
        if lnum == 1 {
            capcol = 0;
        }

        // For checking first word with a capital skip white space.
        if capcol == 0 {
            capcol = nvim_spell_getwhitecols_raw(line) as c_int;
        } else if curline {
            // For spellbadword(): check if first word needs a capital.
            let col = nvim_spell_getwhitecols_raw(line) as c_int;
            // Only check when wp is curwin (we don't have direct access to
            // curwin here, so we rely on the C check_need_cap which uses wp)
            if check_need_cap(wp, lnum, col) {
                capcol = col;
            }
            // Need to get the line again, may have looked at the previous one.
            // (line is already valid here; check_need_cap may not invalidate it)
        }

        // Copy the line into "buf" and append the start of the next line if
        // possible. Note: ml_get_buf may make "line" invalid; check empty line first.
        let empty_line = *(skipwhite(line) as *const u8) == 0;
        strcpy(buf, line);
        let ml_count = nvim_spell_win_ml_line_count(wp);
        if lnum < ml_count {
            let next_line = nvim_spell_ml_get_buf(wbuf, lnum + 1);
            let buf_end_offset = strlen(buf);
            spell_cat_line(buf.add(buf_end_offset), next_line, MAXWLEN as c_int);
        }

        let mut p = buf.add(skip as usize);
        let endp = buf.add(len);

        while p < endp {
            // When searching backward don't search after the cursor.  Unless
            // we wrapped around the end of the buffer.
            if dir == BACKWARD
                && lnum == nvim_win_get_cursor_lnum(wp)
                && !wrapped
                && (p as usize - buf as usize) as c_int >= nvim_win_get_cursor_col(wp)
            {
                break;
            }

            // start of word
            attr = HLF_COUNT;
            let word_len = spell_check(wp, p, &mut attr, &mut capcol, false);

            if attr != HLF_COUNT {
                // We found a bad word. Check the attribute.
                if behaviour == SMT_ALL
                    || (behaviour == SMT_BAD && attr == HLF_SPB)
                    || (behaviour == SMT_RARE && attr == HLF_SPR)
                {
                    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
                    let cursor_col = nvim_win_get_cursor_col(wp);
                    let col = (p as usize - buf as usize) as c_int;

                    // When searching forward only accept a bad word after the cursor.
                    if dir == BACKWARD
                        || lnum != cursor_lnum
                        || wrapped
                        || ((if curline {
                            col + word_len as c_int
                        } else {
                            col
                        }) > cursor_col)
                    {
                        let no_plain_buffer = nvim_spell_win_noplainbuffer(wp);
                        let mut can_spell = !no_plain_buffer;

                        let decor_val = nvim_spell_nav_decor_col(wp, lnum, &mut decor_lnum, col);
                        if decor_val == 1 {
                            // kTrue
                            can_spell = true;
                        } else if decor_val == 0 {
                            // kFalse
                            can_spell = false;
                        } else {
                            // kNone: check syntax
                            if has_syntax {
                                can_spell = nvim_spell_can_syn_spell(wp, lnum, col);
                            }
                        }

                        if !can_spell {
                            attr = HLF_COUNT;
                        }

                        if can_spell {
                            found_one = true;
                            found_lnum = lnum;
                            found_col = col;
                            if dir == FORWARD {
                                // No need to search further.
                                nvim_win_set_cursor_lnum(wp, found_lnum);
                                nvim_win_set_cursor_col(wp, found_col);
                                nvim_win_set_cursor_coladd(wp, 0);
                                if !attrp.is_null() {
                                    *attrp = attr;
                                }
                                ret = word_len;
                                break 'outer;
                            } else if curline {
                                // Insert mode completion: put cursor after the bad word.
                                found_col += word_len as c_int;
                            }
                            found_len = word_len;
                        }
                    } else {
                        found_one = true;
                    }
                }
            }

            // advance to character after the word
            p = p.add(word_len);
            capcol -= word_len as c_int;
        }

        if dir == BACKWARD && found_lnum != 0 {
            // Use the last match in the line (before the cursor).
            nvim_win_set_cursor_lnum(wp, found_lnum);
            nvim_win_set_cursor_col(wp, found_col);
            nvim_win_set_cursor_coladd(wp, 0);
            ret = found_len;
            break 'outer;
        }

        if curline {
            break; // only check cursor line
        }

        let cursor_lnum = nvim_win_get_cursor_lnum(wp);

        // If we are back at the starting line and searched it again there
        // is no match, give up.
        if lnum == cursor_lnum && wrapped {
            break;
        }

        // Advance to next line.
        if dir == BACKWARD {
            if lnum > 1 {
                lnum -= 1;
            } else if nvim_get_p_ws() == 0 {
                break; // at first line and 'nowrapscan'
            } else {
                // Wrap around to the end of the buffer.
                lnum = nvim_spell_win_ml_line_count(wp);
                wrapped = true;
                if nvim_shortmess_search() == 0 {
                    nvim_spell_give_wrap_warning(false); // top_bot_msg
                }
            }
            capcol = -1;
        } else {
            let ml_count = nvim_spell_win_ml_line_count(wp);
            if lnum < ml_count {
                lnum += 1;
            } else if nvim_get_p_ws() == 0 {
                break; // at last line and 'nowrapscan'
            } else {
                // Wrap around to the start of the buffer.
                lnum = 1;
                wrapped = true;
                if nvim_shortmess_search() == 0 {
                    nvim_spell_give_wrap_warning(true); // bot_top_msg
                }
            }

            // If we are back at the starting line and there is no match then
            // give up.
            if lnum == cursor_lnum && !found_one {
                break;
            }

            // Skip the characters at the start of the next line that were
            // included in a match crossing line boundaries.
            if attr == HLF_COUNT {
                skip = (p as isize - endp as isize) as c_int;
            } else {
                skip = 0;
            }

            // Capcol skips over the inserted space.
            capcol -= 1;

            // But after empty line check first word in next line
            if empty_line {
                capcol = 0;
            }
        }

        line_breakcheck();
    }

    // Restore decor_state
    nvim_spell_restore_decor_state(saved_decor);
    xfree(saved_decor);
    xfree(buf.cast::<c_void>());
    ret
}
