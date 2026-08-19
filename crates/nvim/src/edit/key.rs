//! [`insert_handle_key`] -- what a key means in Insert mode.
//!
//! One switch over every key that is *not* simply inserted: the CTRL-
//! commands, the arrows and their shifted forms, backspace and delete, TAB
//! and CR, the completion keys, the mouse, and the two dozen that only differ
//! from a plain character under an option ('paste', 'revins', 'digraph',
//! 'keymodel').  Everything else falls through to [`insert_normal_char`],
//! which is the only place a byte reaches `insertchar`.
//!
//! Upstream is a C `switch` with seven fall-through targets and two `goto`
//! labels, which is what the transpiler rendered as eight nested labelled
//! blocks.  Here every arm answers a [`Next`], and the four fall-through
//! targets that more than one key shares are functions:
//! [`key_end_insert`] (`<Esc>`/CTRL-C), [`key_stuff_last_insert`]
//! (CTRL-A/CTRL-@), [`key_shift`] (CTRL-D/CTRL-T), [`key_tab`],
//! [`key_eol`] (CR/NL) and [`key_complete`] (CTRL-N/CTRL-P/CTRL-L).
//!
//! The one thing the dispatcher itself does is order those three tails:
//! `Normal` runs [`insert_normal_char`], `CheckPum` runs [`check_pum`], and
//! both then join `Continue` in running `insert_handle_key_post`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::keycodes::{K_C_END, K_C_HOME, K_C_LEFT, K_C_RIGHT, K_EVENT, K_IGNORE};
use crate::types::{FAIL, NUL};

/// `<Space>`, which is only a command when CTRL is held (`i_CTRL-@`'s
/// spelling on some terminals).
const SPACE: c_int = b' ' as c_int;
/// `CTRL-X s`, the second spelling of `CTRL-X CTRL-S`.
const SPELL_S: c_int = b's' as c_int;

/// What the dispatcher does after an arm has run.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Next {
    /// The key is done with: run the per-key tail and ask for another.
    Continue,
    /// Leave Insert mode.
    Leave,
    /// Insert the key as an ordinary character (upstream's `normalchar:`).
    Normal,
    /// Run the popup-menu tail first (upstream's `check_pum:`).
    CheckPum,
}

/// Handle a character in Insert mode.
///
/// Answers 0 to leave the mode and 1 to go on, which is the state loop's
/// contract.
///
/// # Safety
/// `s` must point to a live `InsertState`.
pub(crate) unsafe fn insert_handle_key(s: *mut InsertState) -> c_int {
    unsafe {
        // TODO(tarruda, upstream): this could look better with a lookup
        // table, the way Normal mode's `nv_cmds[]` does it.
        let mut next = match (*s).c {
            // End input mode.
            ESC => {
                if echeck_abbr(ESC + ABBR_OFF) {
                    Next::Continue
                } else {
                    key_end_insert(s)
                }
            }
            Ctrl_C => key_end_insert(s),

            Ctrl_O => {
                // CTRL-X CTRL-O completes with 'omnifunc'.
                if ctrl_x_mode_omni() {
                    insert_do_complete(s);
                    Next::Continue
                } else if echeck_abbr(Ctrl_O + ABBR_OFF) {
                    Next::Continue
                } else {
                    ins_ctrl_o();
                    // Don't move the cursor left when 'virtualedit' has
                    // "onemore".
                    if get_ve_flags(curwin.get()) & kOptVeFlagOnemore as ::core::ffi::c_uint != 0 {
                        ins_at_eol.set(false);
                        (*s).nomove = true;
                    }
                    (*s).count = 0;
                    Next::Leave
                }
            }

            // Toggle insert/replace mode.
            K_INS | K_KINS => {
                ins_insert((*s).replaceState);
                Next::Continue
            }

            // Help key works like CTRL-O.
            K_HELP | K_F1 | K_XF1 => {
                stuffcharReadbuff(K_HELP);
                Next::Leave
            }

            // CTRL-@ arrives as a CTRL-modified space on some terminals.
            SPACE if mod_mask.get() != MOD_MASK_CTRL => Next::Normal,
            SPACE | K_ZERO | NUL | Ctrl_A => key_stuff_last_insert(s),

            Ctrl_R => {
                // CTRL-X CTRL-R completes with the registers.
                if ctrl_x_mode_register() && !ins_compl_active() {
                    insert_do_complete(s);
                } else {
                    ins_reg();
                    auto_format(false, true);
                    (*s).inserted_space = false_0;
                }
                Next::Continue
            }
            Ctrl_G => {
                ins_ctrl_g();
                Next::Continue
            }
            Ctrl_HAT => {
                ins_ctrl_hat();
                Next::Continue
            }
            // CTRL-_ toggles 'revins', but only with 'allowrevins'.
            Ctrl__ => {
                if p_ari.get() == 0 {
                    Next::Normal
                } else {
                    ins_ctrl_();
                    Next::Continue
                }
            }

            Ctrl_D => {
                // CTRL-X CTRL-D completes with the defined identifiers.
                if ctrl_x_mode_path_defines() {
                    insert_do_complete(s);
                    Next::Continue
                } else {
                    key_shift(s)
                }
            }
            Ctrl_T => key_shift(s),

            K_DEL | K_KDEL => {
                ins_del();
                auto_format(false, true);
                Next::Continue
            }
            K_BS | Ctrl_H => {
                do_backspace(s, Backspace::Char);
                Next::Continue
            }
            Ctrl_W => {
                // In a prompt buffer plain CTRL-W is the window prefix, so
                // Shift-CTRL-W is what deletes a word.
                if bt_prompt(curbuf.get()) && mod_mask.get() & MOD_MASK_SHIFT == 0 {
                    stuffcharReadbuff(Ctrl_W);
                    restart_edit.set('A' as c_int);
                    (*s).nomove = true;
                    (*s).count = 0;
                    return 0;
                }
                do_backspace(s, Backspace::Word);
                Next::Continue
            }
            Ctrl_U => {
                // CTRL-X CTRL-U completes with 'completefunc'.
                if ctrl_x_mode_function() {
                    insert_do_complete(s);
                } else {
                    // Not `do_backspace`: upstream clears `inserted_space`
                    // *before* the autocompletion runs, and that can call out
                    // to 'completefunc'.
                    (*s).did_backspace =
                        ins_bs((*s).c, Backspace::Line, &raw mut (*s).inserted_space);
                    auto_format(false, true);
                    (*s).inserted_space = false_0;
                    if (*s).did_backspace {
                        may_autocomplete_before_cursor(s);
                    }
                }
                Next::Continue
            }

            K_LEFTMOUSE | K_LEFTMOUSE_NM | K_LEFTDRAG | K_LEFTRELEASE | K_LEFTRELEASE_NM
            | K_MOUSEMOVE | K_MIDDLEMOUSE | K_MIDDLEDRAG | K_MIDDLERELEASE | K_RIGHTMOUSE
            | K_RIGHTDRAG | K_RIGHTRELEASE | K_X1MOUSE | K_X1DRAG | K_X1RELEASE | K_X2MOUSE
            | K_X2DRAG | K_X2RELEASE => {
                ins_mouse((*s).c);
                Next::Continue
            }
            K_MOUSEDOWN => {
                ins_mousescroll(MSCR_DOWN);
                Next::Continue
            }
            K_MOUSEUP => {
                ins_mousescroll(MSCR_UP);
                Next::Continue
            }
            K_MOUSELEFT => {
                ins_mousescroll(MSCR_LEFT);
                Next::Continue
            }
            K_MOUSERIGHT => {
                ins_mousescroll(MSCR_RIGHT);
                Next::Continue
            }

            // Something mapped to nothing.
            K_SELECT | K_IGNORE => Next::Continue,

            K_PASTE_START => {
                paste_repeat(1);
                Next::CheckPum
            }
            K_EVENT => {
                state_handle_k_event();
                // If CTRL-G U was used, apply it to the next typed key.
                if dont_sync_undo.get() == kTrue {
                    dont_sync_undo.set(kNone);
                }
                Next::CheckPum
            }
            // `<Cmd>command<CR>`.
            K_COMMAND => {
                do_cmdline(
                    ::core::ptr::null_mut(),
                    Some(
                        getcmdkeycmd
                            as unsafe fn(
                                c_int,
                                *mut ::core::ffi::c_void,
                                c_int,
                                bool,
                            ) -> *mut c_char,
                    ),
                    NULL,
                    0,
                );
                Next::CheckPum
            }
            K_LUA => {
                map_execute_lua(false, false);
                Next::CheckPum
            }

            K_HOME | K_KHOME | K_S_HOME | K_C_HOME => {
                ins_home((*s).c);
                Next::Continue
            }
            K_END | K_KEND | K_S_END | K_C_END => {
                ins_end((*s).c);
                Next::Continue
            }
            K_LEFT => {
                if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
                    ins_s_left();
                } else {
                    ins_left();
                }
                Next::Continue
            }
            K_S_LEFT | K_C_LEFT => {
                ins_s_left();
                Next::Continue
            }
            K_RIGHT => {
                if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
                    ins_s_right();
                } else {
                    ins_right();
                }
                Next::Continue
            }
            K_S_RIGHT | K_C_RIGHT => {
                ins_s_right();
                Next::Continue
            }

            // With the popup menu up, the vertical motions walk it instead.
            K_UP => {
                if pum_visible() {
                    insert_do_complete(s);
                } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
                    ins_page(true);
                } else {
                    ins_updown(true, false);
                }
                Next::Continue
            }
            K_S_UP | K_PAGEUP | K_KPAGEUP => {
                if pum_visible() {
                    insert_do_complete(s);
                } else {
                    ins_page(true);
                }
                Next::Continue
            }
            K_DOWN => {
                if pum_visible() {
                    insert_do_complete(s);
                } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
                    ins_page(false);
                } else {
                    ins_updown(false, false);
                }
                Next::Continue
            }
            K_S_DOWN | K_PAGEDOWN | K_KPAGEDOWN => {
                if pum_visible() {
                    insert_do_complete(s);
                } else {
                    ins_page(false);
                }
                Next::Continue
            }

            // Shift-TAB is a TAB in Insert mode.
            K_S_TAB => {
                (*s).c = TAB;
                key_tab(s)
            }
            TAB => key_tab(s),

            K_KENTER => {
                (*s).c = CAR;
                key_eol(s)
            }
            CAR | NL => key_eol(s),

            Ctrl_K => {
                // CTRL-X CTRL-K completes with the 'dictionary'.
                if ctrl_x_mode_dictionary() {
                    if check_compl_option(true) {
                        insert_do_complete(s);
                    }
                    Next::Continue
                } else {
                    // Otherwise it is a digraph, which produces a character
                    // to insert -- or nothing, if it was cancelled.
                    (*s).c = ins_digraph();
                    if (*s).c == NUL {
                        Next::Continue
                    } else {
                        Next::Normal
                    }
                }
            }
            Ctrl_X => {
                ins_ctrl_x();
                Next::Continue
            }

            // The CTRL-X submodes that reuse an ordinary key: each is only a
            // command while its submode is active.
            Ctrl_RSB if !ctrl_x_mode_tags() => Next::Normal,
            Ctrl_F if !ctrl_x_mode_files() => Next::Normal,
            SPELL_S | Ctrl_S if !ctrl_x_mode_spell() => Next::Normal,
            Ctrl_RSB | Ctrl_F | SPELL_S | Ctrl_S => {
                insert_do_complete(s);
                Next::Continue
            }

            Ctrl_L if !ctrl_x_mode_whole_line() => Next::Normal,
            Ctrl_L | Ctrl_P | Ctrl_N => key_complete(s),

            // Copy from the line above or below, or scroll.
            Ctrl_Y | Ctrl_E => {
                (*s).c = ins_ctrl_ey((*s).c);
                Next::Continue
            }

            // CTRL-Z is inserted as an ordinary character (upstream says
            // so explicitly), and so is everything else.
            _ => Next::Normal,
        };

        if next == Next::Normal {
            insert_normal_char(s);
            next = Next::Continue;
        }
        if next == Next::CheckPum {
            check_pum(s);
            next = Next::Continue;
        }
        if next == Next::Leave {
            return 0;
        }

        insert_handle_key_post(s);
        1 // continue
    }
}

/// `<Esc>` and CTRL-C: end input mode -- unless a window here wants them.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn key_end_insert(s: *mut InsertState) -> Next {
    unsafe {
        if (*s).c == Ctrl_C && cmdwin_type.get() != 0 {
            // Close the command-line window.
            cmdwin_result.set(K_IGNORE);
            got_int.set(false); // don't stop executing autocommands et al
            (*s).nomove = true;
            return Next::Leave;
        }
        if (*s).c == Ctrl_C && bt_prompt(curbuf.get()) && invoke_prompt_interrupt() {
            if !bt_prompt(curbuf.get()) {
                // The buffer changed to a non-prompt one; leave Insert mode.
                return Next::Leave;
            }
            return Next::Continue;
        }
        Next::Leave
    }
}

/// CTRL-A and CTRL-@: insert the previously inserted text, and for CTRL-@
/// leave the mode afterwards.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn key_stuff_last_insert(s: *mut InsertState) -> Next {
    unsafe {
        // CTRL-A keeps Insert mode, so it asks for no trailing <Esc>.
        if stuff_inserted(NUL, 1, ((*s).c == Ctrl_A) as c_int) == FAIL && (*s).c != Ctrl_A {
            return Next::Leave;
        }
        (*s).inserted_space = false_0;
        Next::Continue
    }
}

/// CTRL-D and CTRL-T: one 'shiftwidth' less or more indent -- unless CTRL-T
/// is the 'thesaurus' completion.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn key_shift(s: *mut InsertState) -> Next {
    unsafe {
        if (*s).c == Ctrl_T && ctrl_x_mode_thesaurus() {
            if check_compl_option(false) {
                insert_do_complete(s);
            }
            return Next::Continue;
        }
        ins_shift((*s).c, (*s).lastc);
        auto_format(false, true);
        (*s).inserted_space = false_0;
        Next::Continue
    }
}

/// TAB and Shift-TAB -- unless CTRL-X CTRL-D's sibling, the 'include' path
/// completion, is active.
///
/// `ins_tab` answering true means the TAB is to go in as an ordinary
/// character.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn key_tab(s: *mut InsertState) -> Next {
    unsafe {
        if ctrl_x_mode_path_patterns() {
            insert_do_complete(s);
            return Next::Continue;
        }
        (*s).inserted_space = false_0;
        if ins_tab() {
            return Next::Normal;
        }
        auto_format(false, true);
        Next::Continue
    }
}

/// CR, NL and `<kEnter>`: open a line -- or, in the buffers where `<CR>` means
/// something else, do that instead.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn key_eol(s: *mut InsertState) -> Next {
    unsafe {
        // In a quickfix or location-list window, `<CR>` jumps to the entry.
        if bt_quickfix(curbuf.get()) && (*s).c == CAR {
            if (*curwin.get()).w_llist_ref.is_null() {
                do_cmdline_cmd(c".cc".as_ptr());
            } else {
                do_cmdline_cmd(c".ll".as_ptr());
            }
            return Next::Continue;
        }
        // In the command-line window it accepts the line.
        if cmdwin_type.get() != 0 {
            cmdwin_result.set(CAR);
            return Next::Leave;
        }
        // In a prompt buffer it submits, unless Shift is held.
        if mod_mask.get() & MOD_MASK_SHIFT == 0 && bt_prompt(curbuf.get()) {
            prompt_invoke_callback();
            if !bt_prompt(curbuf.get()) {
                // The callback turned this into an ordinary buffer.
                return Next::Leave;
            }
            return Next::Continue;
        }

        if !ins_eol((*s).c) {
            return Next::Leave;
        }
        auto_format(false, false);
        (*s).inserted_space = false_0;
        Next::Continue
    }
}

/// CTRL-N, CTRL-P and CTRL-L: complete -- unless 'complete' is empty and no
/// CTRL-X submode is running, in which case there is nothing to complete
/// with and the key is inserted.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn key_complete(s: *mut InsertState) -> Next {
    unsafe {
        if *(*curbuf.get()).b_p_cpt as c_int == NUL
            && (ctrl_x_mode_normal() || ctrl_x_mode_whole_line())
            && !compl_status_local()
        {
            return Next::Normal;
        }
        insert_do_complete(s);
        Next::Continue
    }
}

/// One backspace of `mode`, and everything the three backspace keys share
/// afterwards.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn do_backspace(s: *mut InsertState, mode: Backspace) {
    unsafe {
        (*s).did_backspace = ins_bs((*s).c, mode, &raw mut (*s).inserted_space);
        auto_format(false, true);
        if (*s).did_backspace {
            may_autocomplete_before_cursor(s);
        }
    }
}

/// Upstream's `MAY_TRIGGER_AUTOCOMPLETE`: after a deletion, start completing
/// again on whatever word character is now in front of the cursor.
///
/// Only while nothing else is typed ahead -- autocompletion must never make
/// a burst of keys slow.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn may_autocomplete_before_cursor(s: *mut InsertState) {
    unsafe {
        if !(ins_compl_has_autocomplete() && !char_avail() && (*curwin.get()).w_cursor.col > 0) {
            return;
        }
        (*s).c = char_before_cursor();
        if vim_isprintc((*s).c) {
            redraw_later(curwin.get(), UPD_VALID);
            update_screen();
            ui_flush();
            ins_compl_enable_autocomplete();
            insert_do_complete(s);
        }
    }
}

/// Upstream's `check_pum:` -- the tail the keys that can run *arbitrary
/// code* share.
///
/// `nvim_select_popupmenu_item()` may have been called while handling a
/// `K_EVENT`, `K_COMMAND` or `K_LUA`; going through `insert_do_complete`
/// here makes that equivalent to selecting the item with a typed key.  The
/// same code may also have synced undo, in which case the next character
/// needs the line saved again.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn check_pum(s: *mut InsertState) {
    unsafe {
        if (*pum_want.ptr()).active {
            if pum_visible() {
                // Null so that `ins_complete` updates the message.
                edit_submode_extra.set(::core::ptr::null_mut());
                insert_do_complete(s);
                if (*pum_want.ptr()).finish {
                    // Accept the item and stop completing.
                    ins_compl_prep(Ctrl_Y);
                }
            }
            (*pum_want.ptr()).active = false;
        }

        if (*curbuf.get()).b_u_synced {
            ins_need_undo.set(true);
        }
    }
}

/// Upstream's `normalchar:` -- insert the key as an ordinary character.
///
/// Three things happen before the insert.  `InsertCharPre` may replace the
/// character with a whole *string*, which then goes in a character at a time
/// and leaves nothing for the rest of the function.  'smartindent' gets a
/// look at it.  And a non-word character may complete an abbreviation, in
/// which case it is *not* inserted here -- `check_abbr` has stuffed the
/// expansion and the character back into the typeahead.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn insert_normal_char(s: *mut InsertState) {
    unsafe {
        if p_paste.get() == 0 {
            let str = do_insert_char_pre((*s).c);
            if !str.is_null() {
                if *str as c_int != NUL && stop_arrow() != FAIL {
                    // Insert the new value of v:char literally.
                    let mut p = str;
                    while *p as c_int != NUL {
                        (*s).c = utf_ptr2char(p);
                        if (*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL {
                            ins_eol((*s).c);
                        } else {
                            ins_char((*s).c);
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    AppendToRedobuffLit(str, -1);
                }
                xfree(str as *mut ::core::ffi::c_void);
                (*s).c = NUL;
            }
            // The new value is already in, or was an empty string.
            if (*s).c == NUL {
                return;
            }
        }

        ins_try_si((*s).c);

        if (*s).c == ' ' as c_int {
            (*s).inserted_space = true_0;
            if inindent(0) {
                can_cindent.set(false);
            }
            if Insstart_blank_vcol.get() == MAXCOL as colnr_T
                && (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum
            {
                Insstart_blank_vcol.set(get_nolist_virtcol());
            }
        }

        // Insert the character, checking for an abbreviation on a special
        // one.  `CTRL-]` expands an abbreviation without being inserted
        // itself.  `check_abbr` wants ABBR_OFF added above 0x100.
        if vim_iswordc((*s).c)
            || (!echeck_abbr(if (*s).c >= 0x100 {
                (*s).c + ABBR_OFF
            } else {
                (*s).c
            }) && (*s).c != Ctrl_RSB)
        {
            insert_special((*s).c, false_0, false_0);
            (*revins_legal.ptr()) += 1;
            (*revins_chars.ptr()) += 1;
        }

        auto_format(false, true);

        // The cursor line must never be in a closed fold after an insert.
        foldOpenCursor();

        // Autocompletion, on the character just inserted.
        if ins_compl_has_autocomplete() && !char_avail() && vim_isprintc((*s).c) {
            redraw_later(curwin.get(), UPD_VALID);
            update_screen();
            ui_flush();
            ins_compl_enable_autocomplete();
            insert_do_complete(s);
        }
    }
}
