//! Which completion is running: the CTRL-X modes and the state queries.
//!
//! `ctrl_x_mode` is the mode the user selected with a CTRL-X chord, and
//! [`vim_is_ctrl_x_key`] decides whether the key just typed still belongs to
//! it.  [`set_ctrl_x_mode`] is the chord dispatch itself.  The rest are the
//! one-line queries the editor asks about a completion in progress.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::{
    Ctrl_D, Ctrl_E, Ctrl_F, Ctrl_I, Ctrl_K, Ctrl_L, Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R,
    Ctrl_RSB, Ctrl_S, Ctrl_T, Ctrl_U, Ctrl_V, Ctrl_X, Ctrl_Y, Ctrl_Z,
};

/// The `'s'` of C's `case 's': case Ctrl_S:` in [`set_ctrl_x_mode`].  A cast
/// is not a pattern, so the literal needs a name to be matched on.
const LOWER_S: c_int = b's' as c_int;

/// Enter CTRL-X mode, or — already on the command line — its CTRL-X flavour.
pub unsafe fn ins_ctrl_x() {
    unsafe {
        if ctrl_x_mode_cmdline() {
            ctrl_x_mode.set(CTRL_X_CMDLINE_CTRL_X);
        } else {
            // CTRL-X after a completion that was interrupted keeps the ADDING
            // state; a fresh one clears it.
            if compl_cont_status.get() & CONT_N_ADDS != 0 {
                compl_cont_status.set(compl_cont_status.get() | CONT_INTRPT);
            } else {
                compl_cont_status.set(0);
            }
            ctrl_x_mode.set(CTRL_X_NOT_DEFINED_YET);
            edit_submode.set(ctrl_x_msg(ctrl_x_mode.get()));
            edit_submode_pre.set(ptr::null_mut());
            redraw_mode.set(true);
        }
        may_trigger_modechanged();
    }
}

pub fn ctrl_x_mode_none() -> bool {
    ctrl_x_mode.get() == 0
}

pub fn ctrl_x_mode_normal() -> bool {
    ctrl_x_mode.get() == CTRL_X_NORMAL
}

pub fn ctrl_x_mode_scroll() -> bool {
    ctrl_x_mode.get() == CTRL_X_SCROLL
}

pub fn ctrl_x_mode_whole_line() -> bool {
    ctrl_x_mode.get() == CTRL_X_WHOLE_LINE
}

pub fn ctrl_x_mode_files() -> bool {
    ctrl_x_mode.get() == CTRL_X_FILES
}

pub fn ctrl_x_mode_tags() -> bool {
    ctrl_x_mode.get() == CTRL_X_TAGS
}

pub fn ctrl_x_mode_path_patterns() -> bool {
    ctrl_x_mode.get() == CTRL_X_PATH_PATTERNS
}

pub fn ctrl_x_mode_path_defines() -> bool {
    ctrl_x_mode.get() == CTRL_X_PATH_DEFINES
}

pub fn ctrl_x_mode_dictionary() -> bool {
    ctrl_x_mode.get() == CTRL_X_DICTIONARY
}

pub fn ctrl_x_mode_thesaurus() -> bool {
    ctrl_x_mode.get() == CTRL_X_THESAURUS
}

pub fn ctrl_x_mode_cmdline() -> bool {
    ctrl_x_mode.get() == CTRL_X_CMDLINE || ctrl_x_mode.get() == CTRL_X_CMDLINE_CTRL_X
}

pub fn ctrl_x_mode_function() -> bool {
    ctrl_x_mode.get() == CTRL_X_FUNCTION
}

pub fn ctrl_x_mode_omni() -> bool {
    ctrl_x_mode.get() == CTRL_X_OMNI
}

pub fn ctrl_x_mode_spell() -> bool {
    ctrl_x_mode.get() == CTRL_X_SPELL
}

pub(crate) fn ctrl_x_mode_eval() -> bool {
    ctrl_x_mode.get() == CTRL_X_EVAL
}

pub fn ctrl_x_mode_line_or_eval() -> bool {
    ctrl_x_mode.get() == CTRL_X_WHOLE_LINE || ctrl_x_mode.get() == CTRL_X_EVAL
}

pub fn ctrl_x_mode_register() -> bool {
    ctrl_x_mode.get() == CTRL_X_REGISTER
}

pub fn ctrl_x_mode_not_default() -> bool {
    ctrl_x_mode.get() != CTRL_X_NORMAL
}

pub fn ctrl_x_mode_not_defined_yet() -> bool {
    ctrl_x_mode.get() == CTRL_X_NOT_DEFINED_YET
}

pub fn compl_status_adding() -> bool {
    compl_cont_status.get() & CONT_ADDING != 0
}

pub fn compl_status_sol() -> bool {
    compl_cont_status.get() & CONT_SOL != 0
}

pub fn compl_status_local() -> bool {
    compl_cont_status.get() & CONT_LOCAL != 0
}

pub fn compl_status_clear() {
    compl_cont_status.set(0);
}

/// True if completion is collecting matches in the forward direction.
pub(crate) fn compl_dir_forward() -> bool {
    compl_direction.get() == FORWARD
}

/// True if the matches currently *shown* run forward.
pub(crate) fn compl_shows_dir_forward() -> bool {
    compl_shows_dir.get() == FORWARD
}

pub(crate) fn compl_shows_dir_backward() -> bool {
    compl_shows_dir.get() == BACKWARD
}

/// Check that `'dictionary'` (`dict_opt`) or `'thesaurus'` can be used;
/// complain, beep and leave CTRL-X mode when it cannot.
pub unsafe fn check_compl_option(dict_opt: bool) -> bool {
    unsafe {
        let empty = if dict_opt {
            *(*curbuf.get()).b_p_dict as c_int == NUL
                && *p_dict.get() as c_int == NUL
                && (*curwin.get()).w_onebuf_opt.wo_spell == 0
        } else {
            *(*curbuf.get()).b_p_tsr as c_int == NUL
                && *p_tsr.get() as c_int == NUL
                && *(*curbuf.get()).b_p_tsrfu as c_int == NUL
                && *p_tsrfu.get() as c_int == NUL
        };
        if !empty {
            return true;
        }
        ctrl_x_mode.set(CTRL_X_NORMAL);
        edit_submode.set(ptr::null_mut());
        emsg(gettext(if dict_opt {
            c"'dictionary' option is empty".as_ptr()
        } else {
            c"'thesaurus' option is empty".as_ptr()
        }));
        if emsg_silent.get() == 0 && !in_assert_fails.get() {
            vim_beep(kOptBoFlagComplete);
            setcursor();
            msg_delay(2004, false);
        }
        false
    }
}

/// Is `c` a key that goes to, or keeps us in, the current CTRL-X mode?
pub unsafe fn vim_is_ctrl_x_key(c: c_int) -> bool {
    // Always allow CTRL-R — let its results then be checked.
    if c == Ctrl_R && ctrl_x_mode.get() != CTRL_X_REGISTER {
        return true;
    }
    // Accept <PageUp> and <PageDown> if the popup menu is visible.
    if ins_compl_pum_key(c) {
        return true;
    }
    match ctrl_x_mode.get() {
        // Not in any CTRL-X mode.
        CTRL_X_NORMAL => c == Ctrl_N || c == Ctrl_P || c == Ctrl_X,
        CTRL_X_NOT_DEFINED_YET | CTRL_X_CMDLINE_CTRL_X => {
            c == Ctrl_X
                || c == Ctrl_Y
                || c == Ctrl_E
                || c == Ctrl_L
                || c == Ctrl_F
                || c == Ctrl_RSB
                || c == Ctrl_I
                || c == Ctrl_D
                || c == Ctrl_P
                || c == Ctrl_N
                || c == Ctrl_T
                || c == Ctrl_V
                || c == Ctrl_Q
                || c == Ctrl_U
                || c == Ctrl_O
                || c == Ctrl_S
                || c == Ctrl_K
                || c == LOWER_S
                || c == Ctrl_Z
                || c == Ctrl_R
        }
        CTRL_X_SCROLL => c == Ctrl_Y || c == Ctrl_E,
        CTRL_X_WHOLE_LINE => c == Ctrl_L || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_FILES => c == Ctrl_F || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_DICTIONARY => c == Ctrl_K || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_THESAURUS => c == Ctrl_T || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_TAGS => c == Ctrl_RSB || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_PATH_PATTERNS => c == Ctrl_P || c == Ctrl_N,
        CTRL_X_PATH_DEFINES => c == Ctrl_D || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_CMDLINE => c == Ctrl_V || c == Ctrl_Q || c == Ctrl_P || c == Ctrl_N || c == Ctrl_X,
        CTRL_X_FUNCTION => c == Ctrl_U || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_OMNI => c == Ctrl_O || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_SPELL => c == Ctrl_S || c == Ctrl_P || c == Ctrl_N,
        CTRL_X_EVAL => c == Ctrl_P || c == Ctrl_N,
        CTRL_X_BUFNAMES => c == Ctrl_P || c == Ctrl_N,
        CTRL_X_REGISTER => c == Ctrl_R || c == Ctrl_P || c == Ctrl_N,
        _ => {
            unsafe { internal_error(c"vim_is_ctrl_x_key()".as_ptr()) };
            false
        }
    }
}

/// True if `match_0` is the original text the completion began with.
pub(crate) unsafe fn match_at_original_text(match_0: *const compl_T) -> bool {
    unsafe { (*match_0).cp_flags & CP_ORIGINAL_TEXT != 0 }
}

/// True if `match_0` is the first match in the completion list.
pub(crate) fn is_first_match(match_0: *const compl_T) -> bool {
    match_0 == compl_first_match.get() as *const compl_T
}

/// Is `c` part of the item being completed?  Decides whether typing it
/// abandons the completion while the menu is up.
pub unsafe fn ins_compl_accept_char(c: c_int) -> bool {
    if compl_autocomplete.get() && compl_from_nonkeyword.get() {
        return false;
    }
    unsafe {
        if ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0 {
            // Expanding an identifier: only identifier characters.
            return vim_isIDc(c);
        }
        match ctrl_x_mode.get() {
            // File names, but not path separators, so that "proto/<Tab>"
            // expands files in "proto", not "proto/" as a whole.
            CTRL_X_FILES => vim_isfilec(c) && !vim_ispathsep(c),
            // Command line and omni completion take just about any printable
            // character, but do stop at white space.
            CTRL_X_CMDLINE | CTRL_X_CMDLINE_CTRL_X | CTRL_X_OMNI => {
                vim_isprintc(c) && !ascii_iswhite(c)
            }
            // For whole-line completion a space can be part of the line.
            CTRL_X_WHOLE_LINE => vim_isprintc(c),
            _ => vim_iswordc(c),
        }
    }
}

/// `'completeopt'` has `fuzzy` (and this is not thesaurus completion, which
/// never fuzzy-matches).
pub(crate) unsafe fn cot_fuzzy() -> bool {
    unsafe { get_cot_flags() & kOptCotFlagFuzzy != 0 && !ctrl_x_mode_thesaurus() }
}

/// Matches are ordered by distance from the cursor: `'completeopt'` has
/// `nearest`, or autocompletion is on, and `fuzzy` is not overriding it.
pub(crate) unsafe fn is_nearest_active() -> bool {
    unsafe {
        (compl_autocomplete.get() || get_cot_flags() & kOptCotFlagNearest != 0) && !cot_fuzzy()
    }
}

pub unsafe fn ins_compl_is_match_selected() -> bool {
    !compl_shown_match.get().is_null() && !is_first_match(compl_shown_match.get())
}

/// Autocompletion inserting the longest common prefix: `'completeopt'` has
/// `longest` without `preinsert` or `fuzzy`.
pub unsafe fn ins_compl_preinsert_longest() -> bool {
    compl_autocomplete.get()
        && unsafe { get_cot_flags() }
            & (kOptCotFlagLongest | kOptCotFlagPreinsert | kOptCotFlagFuzzy)
            == kOptCotFlagLongest
}

/// The text matches are filtered against: what the user typed since the
/// completion started, or the original text when nothing was typed.
pub fn ins_compl_leader() -> *mut c_char {
    let leader = compl_leader.get();
    if leader.data.is_null() {
        compl_orig_text.get().data
    } else {
        leader.data
    }
}

pub(crate) fn ins_compl_leader_len() -> size_t {
    let leader = compl_leader.get();
    if leader.data.is_null() {
        compl_orig_text.get().size
    } else {
        leader.size
    }
}

/// The shown match spans more than one line.
pub(crate) unsafe fn ins_compl_has_multiple() -> bool {
    unsafe { !vim_strchr((*compl_shown_match.get()).cp_str.data, NL).is_null() }
}

/// `lnum` is one of the lines a multi-line match is being inserted over.
pub unsafe fn ins_compl_lnum_in_range(lnum: linenr_T) -> bool {
    unsafe {
        ins_compl_has_multiple()
            && lnum >= compl_lnum.get()
            && lnum <= (*curwin.get()).w_cursor.lnum
    }
}

pub unsafe fn ins_compl_has_shown_match() -> bool {
    let shown = compl_shown_match.get();
    shown.is_null() || shown != unsafe { (*shown).cp_next }
}

/// The shown match is longer than what has been inserted so far.
pub unsafe fn ins_compl_long_shown_match() -> bool {
    let shown = compl_shown_match.get();
    !shown.is_null()
        && unsafe {
            !(*shown).cp_str.data.is_null()
                && (*shown).cp_str.size as colnr_T > (*curwin.get()).w_cursor.col - compl_col.get()
        }
}

/// `'completeopt'`, buffer-local value first.
pub unsafe fn get_cot_flags() -> c_uint {
    unsafe {
        let local = (*curbuf.get()).b_cot_flags;
        if local != 0 { local } else { cot_flags.get() }
    }
}

pub fn ins_compl_active() -> bool {
    compl_started.get()
}

/// A completion is running, and `wp` is the window it started in.
pub unsafe fn ins_compl_win_active(wp: *mut win_T) -> bool {
    ins_compl_active()
        && wp == compl_curr_win.get()
        && unsafe { (*wp).w_buffer == compl_curr_buf.get() }
}

pub fn ins_compl_used_match() -> bool {
    compl_used_match.get()
}

pub fn ins_compl_init_get_longest() {
    compl_get_longest.set(false);
}

pub fn ins_compl_interrupted() -> bool {
    compl_interrupted.get() || compl_time_slice_expired.get()
}

pub fn ins_compl_enter_selects() -> bool {
    compl_enter_selects.get()
}

pub fn ins_compl_col() -> colnr_T {
    compl_col.get()
}

pub fn ins_compl_len() -> c_int {
    compl_length.get()
}

/// The match is previewed in the buffer rather than only in the menu:
/// `'completeopt'` has `preinsert` (with `menuone`, when autocompletion is
/// off) and not `fuzzy`.
pub unsafe fn ins_compl_has_preinsert() -> bool {
    let flags = unsafe { get_cot_flags() };
    if compl_autocomplete.get() && p_ic.get() != 0 && p_inf.get() == 0 {
        return false;
    }
    if compl_autocomplete.get() {
        flags & (kOptCotFlagPreinsert | kOptCotFlagFuzzy) == kOptCotFlagPreinsert
    } else {
        flags & (kOptCotFlagPreinsert | kOptCotFlagFuzzy | kOptCotFlagMenuone)
            == kOptCotFlagPreinsert | kOptCotFlagMenuone
    }
}

/// A previewed match is currently in the buffer ahead of the cursor.
pub unsafe fn ins_compl_preinsert_effect() -> bool {
    unsafe {
        (ins_compl_has_preinsert() || ins_compl_preinsert_longest())
            && (*curwin.get()).w_cursor.col < compl_ins_end_col.get()
    }
}

/// The completion function asked for its matches to be recomputed on every
/// keystroke (`refresh: 'always'`).
pub(crate) fn ins_compl_refresh_always() -> bool {
    (ctrl_x_mode_function() || ctrl_x_mode_omni()) && compl_opt_refresh_always.get()
}

pub(crate) fn ins_compl_need_restart() -> bool {
    compl_was_interrupted.get() || ins_compl_refresh_always()
}

/// `'autocomplete'`, buffer-local value first (`-1` means "unset").
pub unsafe fn ins_compl_has_autocomplete() -> bool {
    let local = unsafe { (*curbuf.get()).b_p_ac };
    (if local >= 0 { local } else { p_ac.get() }) != 0
}

/// How much of the leader has been typed: the cursor's distance from
/// `compl_col`, never negative.
pub(crate) unsafe fn get_compl_len() -> c_int {
    let off = unsafe { (*curwin.get()).w_cursor.col } - compl_col.get();
    off.max(0)
}

/// The CTRL-X chord dispatch: `c` is the key typed after CTRL-X.
///
/// Returns true when the completion should stop without inserting anything
/// (CTRL-X CTRL-Z).
pub(crate) unsafe fn set_ctrl_x_mode(c: c_int) -> bool {
    let mut retval = false;
    unsafe {
        'chord: {
            match c {
                // Scroll the window one line up or down.
                Ctrl_E | Ctrl_Y => {
                    ctrl_x_mode.set(CTRL_X_SCROLL);
                    edit_submode.set(gettext(if State.get() & REPLACE_FLAG == 0 {
                        c" (insert) Scroll (^E/^Y)".as_ptr()
                    } else {
                        c" (replace) Scroll (^E/^Y)".as_ptr()
                    }));
                    edit_submode_pre.set(ptr::null_mut());
                    redraw_mode.set(true);
                    break 'chord;
                }
                // Complete whole lines.
                Ctrl_L => {
                    ctrl_x_mode.set(CTRL_X_WHOLE_LINE);
                    break 'chord;
                }
                // Complete file names.
                Ctrl_F => {
                    ctrl_x_mode.set(CTRL_X_FILES);
                    break 'chord;
                }
                // Complete words from a dictionary.
                Ctrl_K => {
                    ctrl_x_mode.set(CTRL_X_DICTIONARY);
                    break 'chord;
                }
                Ctrl_R => {
                    // CTRL-R followed by '=' is an expression register, not
                    // register completion: leave the mode alone.
                    if vpeekc() != '=' as c_int {
                        ctrl_x_mode.set(CTRL_X_REGISTER);
                    }
                    break 'chord;
                }
                // Complete words from a thesaurus.
                Ctrl_T => {
                    ctrl_x_mode.set(CTRL_X_THESAURUS);
                    break 'chord;
                }
                // User defined completion.
                Ctrl_U => {
                    ctrl_x_mode.set(CTRL_X_FUNCTION);
                    break 'chord;
                }
                // Omni completion.
                Ctrl_O => {
                    ctrl_x_mode.set(CTRL_X_OMNI);
                    break 'chord;
                }
                // Complete spelling suggestions.
                LOWER_S | Ctrl_S => {
                    ctrl_x_mode.set(CTRL_X_SPELL);
                    emsg_off.set(emsg_off.get() + 1); // avoid E756 twice
                    spell_back_to_badword();
                    emsg_off.set(emsg_off.get() - 1);
                    break 'chord;
                }
                // Complete tag names.
                Ctrl_RSB => {
                    ctrl_x_mode.set(CTRL_X_TAGS);
                    break 'chord;
                }
                // Complete keywords from included files.
                Ctrl_I | K_S_TAB => {
                    ctrl_x_mode.set(CTRL_X_PATH_PATTERNS);
                    break 'chord;
                }
                // Complete definitions from included files.
                Ctrl_D => {
                    ctrl_x_mode.set(CTRL_X_PATH_DEFINES);
                    break 'chord;
                }
                // Complete Vim commands.
                Ctrl_V | Ctrl_Q => {
                    ctrl_x_mode.set(CTRL_X_CMDLINE);
                    break 'chord;
                }
                // Stop completion.
                Ctrl_Z => {
                    ctrl_x_mode.set(CTRL_X_NORMAL);
                    edit_submode.set(ptr::null_mut());
                    redraw_mode.set(true);
                    retval = true;
                    break 'chord;
                }
                // CTRL-X CTRL-P means LOCAL expansion if nothing interrupted (we
                // just started CTRL-X mode, or there were enough CTRL-X's to
                // cancel the previous mode, say ^X^F^X^X^P or ^P^X^X^X^P); normal
                // expansion when interrupting a different mode (^X^F^X^P or
                // ^P^X^X^P).  Nothing changes when interrupting mode 0 — the flag
                // does not change when going to ADDING mode.  -- Acevedo
                Ctrl_P | Ctrl_N => {
                    if compl_cont_status.get() & CONT_INTRPT == 0 {
                        compl_cont_status.set(compl_cont_status.get() | CONT_LOCAL);
                    } else if compl_cont_mode.get() != 0 {
                        compl_cont_status.set(compl_cont_status.get() & !CONT_LOCAL);
                    }
                    // C: FALLTHROUGH into `default`, which is the tail below.
                }
                _ => {}
            }
            // C's `default:` arm, which CTRL-P and CTRL-N fall into.
            //
            // After at least two CTRL-X's, for modes != 0 we clear
            // `compl_cont_status` (as if CTRL-X mode had just started); for mode 0
            // we set `compl_cont_mode` to an impossible value.  Either way ^X^X
            // restarts the same mode, avoiding ADDING mode.  Undocumented: in a
            // mode != 0, ^X^P and ^X^X^P start 'complete' and local ^P expansions
            // respectively; in mode 0 an extra ^X is needed, since ^X^P goes to
            // ADDING mode.  -- Acevedo
            if c == Ctrl_X {
                if compl_cont_mode.get() != 0 {
                    compl_cont_status.set(0);
                } else {
                    compl_cont_mode.set(CTRL_X_NOT_DEFINED_YET);
                }
            }
            ctrl_x_mode.set(CTRL_X_NORMAL);
            edit_submode.set(ptr::null_mut());
            redraw_mode.set(true);
        }
    }
    retval
}

/// The `mode` string `complete_info()` reports, empty when no completion is
/// running.
pub(crate) fn ins_compl_mode() -> *mut c_char {
    if ctrl_x_mode_not_defined_yet() || ctrl_x_mode_scroll() || compl_started.get() {
        // Upstream indexes unconditionally: the two NULL rows of the table
        // answer a null pointer, which is what the caller then handles.
        return match CTRL_X_MODE_NAMES[(ctrl_x_mode.get() & !CTRL_X_WANT_IDENT) as usize] {
            Some(name) => name.as_ptr().cast_mut(),
            None => ptr::null_mut(),
        };
    }
    c"".as_ptr().cast_mut()
}

/// Which way the key typed moves through the matches: BACKWARD or FORWARD.
pub(crate) unsafe fn ins_compl_key2dir(c: c_int) -> Direction {
    if c == K_EVENT || c == K_COMMAND || c == K_LUA {
        return if pum_want.get().item < compl_selected_item.get() {
            BACKWARD
        } else {
            FORWARD
        };
    }
    if c == Ctrl_P || c == Ctrl_L || c == K_PAGEUP || c == K_KPAGEUP || c == K_S_UP || c == K_UP {
        return BACKWARD;
    }
    FORWARD
}

/// `c` is a completion key only while the popup menu is shown.
pub(crate) fn ins_compl_pum_key(c: c_int) -> bool {
    pum_visible()
        && (c == K_PAGEUP
            || c == K_KPAGEUP
            || c == K_S_UP
            || c == K_PAGEDOWN
            || c == K_KPAGEDOWN
            || c == K_S_DOWN
            || c == K_UP
            || c == K_DOWN)
}

/// How many matches the key typed moves: one for most keys, a menu's height
/// for the page keys.
pub(crate) unsafe fn ins_compl_key2count(c: c_int) -> c_int {
    if c == K_EVENT || c == K_COMMAND || c == K_LUA {
        let offset = pum_want.get().item - compl_selected_item.get();
        return unsafe { abs(offset) };
    }
    if ins_compl_pum_key(c) && c != K_UP && c != K_DOWN {
        let h = pum_get_height();
        return if h > 3 { h - 2 } else { h }; // keep some context
    }
    1
}

/// True when completing with `c` should insert the match, false when it only
/// changes which match is selected.
pub(crate) fn ins_compl_use_match(c: c_int) -> bool {
    match c {
        K_UP | K_DOWN | K_PAGEDOWN | K_KPAGEDOWN | K_S_DOWN | K_PAGEUP | K_KPAGEUP | K_S_UP => {
            false
        }
        K_EVENT | K_COMMAND | K_LUA => {
            let want = pum_want.get();
            want.active && want.insert
        }
        _ => true,
    }
}

pub fn ins_compl_enable_autocomplete() {
    compl_autocomplete.set(true);
    compl_get_longest.set(false);
}

/// `preinserted()`: is a previewed match currently in the buffer?
pub unsafe extern "C" fn f_preinserted(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        if ins_compl_preinsert_effect() {
            (*rettv).vval.v_number = 1;
        }
    }
}
