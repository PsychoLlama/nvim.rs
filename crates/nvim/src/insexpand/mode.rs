//! Which completion is running: the CTRL-X modes and the state queries.
//!
//! `ctrl_x_mode` is the mode the user selected with a CTRL-X chord, and
//! [`vim_is_ctrl_x_key`] decides whether the key just typed still belongs to
//! it.  [`set_ctrl_x_mode`] is the chord dispatch itself.  The rest are the
//! one-line queries the editor asks about a completion in progress.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]
#![forbid(unsafe_code)]

use super::*;
use crate::guard::Suppress;
use crate::keycodes::SpecialKey;
use crate::optionstr::OptStringRef;
use crate::types::Key;
use crate::winlayer::{Buf, Win};

/// The `'s'` of C's `case 's': case CTRL_S:` in [`set_ctrl_x_mode`].  A
/// constructor call is not a pattern, so the key needs a name to be matched
/// on.
const LOWER_S: Key = Key::ascii(b's');

/// Enter CTRL-X mode, or — already on the command line — its CTRL-X flavour.
pub fn ins_ctrl_x() {
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
        EDIT_SUBMODE.set(ctrl_x_msg(ctrl_x_mode.get()));
        EDIT_SUBMODE_PRE.set(None);
        REDRAW_MODE.set(true);
    }
    may_trigger_modechanged();
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
pub fn check_compl_option(dict_opt: bool) -> bool {
    let buffer = Buf::current();
    let empty = if dict_opt {
        let unset = buffer.b_p_dict.bytes().is_empty() && p_dict(CStr::is_empty);
        unset && Win::current().w_onebuf_opt.wo_spell == 0
    } else {
        buffer.b_p_tsr.bytes().is_empty()
            && p_tsr(CStr::is_empty)
            && buffer.b_p_tsrfu.bytes().is_empty()
            && p_tsrfu(CStr::is_empty)
    };
    if !empty {
        return true;
    }
    ctrl_x_mode.set(CTRL_X_NORMAL);
    EDIT_SUBMODE.set(None);
    emsg(gettext(if dict_opt {
        c"'dictionary' option is empty"
    } else {
        c"'thesaurus' option is empty"
    }));
    if EMSG_SILENT.get() == 0 && !IN_ASSERT_FAILS.get() {
        vim_beep(kOptBoFlagComplete);
        setcursor();
        msg_delay(2004, false);
    }
    false
}

/// Is `c` a key that goes to, or keeps us in, the current CTRL-X mode?
pub fn vim_is_ctrl_x_key(c: Key) -> bool {
    // Always allow CTRL-R — let its results then be checked.
    if c == Key::CTRL_R && ctrl_x_mode.get() != CTRL_X_REGISTER {
        return true;
    }
    // Accept <PageUp> and <PageDown> if the popup menu is visible.
    if ins_compl_pum_key(c) {
        return true;
    }
    match ctrl_x_mode.get() {
        // Not in any CTRL-X mode.
        CTRL_X_NORMAL => c == Key::CTRL_N || c == Key::CTRL_P || c == Key::CTRL_X,
        CTRL_X_NOT_DEFINED_YET | CTRL_X_CMDLINE_CTRL_X => {
            c == Key::CTRL_X
                || c == Key::CTRL_Y
                || c == Key::CTRL_E
                || c == Key::CTRL_L
                || c == Key::CTRL_F
                || c == Key::CTRL_RSB
                || c == Key::CTRL_I
                || c == Key::CTRL_D
                || c == Key::CTRL_P
                || c == Key::CTRL_N
                || c == Key::CTRL_T
                || c == Key::CTRL_V
                || c == Key::CTRL_Q
                || c == Key::CTRL_U
                || c == Key::CTRL_O
                || c == Key::CTRL_S
                || c == Key::CTRL_K
                || c == LOWER_S
                || c == Key::CTRL_Z
                || c == Key::CTRL_R
        }
        CTRL_X_SCROLL => c == Key::CTRL_Y || c == Key::CTRL_E,
        CTRL_X_WHOLE_LINE => c == Key::CTRL_L || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_FILES => c == Key::CTRL_F || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_DICTIONARY => c == Key::CTRL_K || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_THESAURUS => c == Key::CTRL_T || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_TAGS => c == Key::CTRL_RSB || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_PATH_PATTERNS => c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_PATH_DEFINES => c == Key::CTRL_D || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_CMDLINE => {
            c == Key::CTRL_V
                || c == Key::CTRL_Q
                || c == Key::CTRL_P
                || c == Key::CTRL_N
                || c == Key::CTRL_X
        }
        CTRL_X_FUNCTION => c == Key::CTRL_U || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_OMNI => c == Key::CTRL_O || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_SPELL => c == Key::CTRL_S || c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_EVAL => c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_BUFNAMES => c == Key::CTRL_P || c == Key::CTRL_N,
        CTRL_X_REGISTER => c == Key::CTRL_R || c == Key::CTRL_P || c == Key::CTRL_N,
        _ => {
            internal_error(c"vim_is_ctrl_x_key()");
            false
        }
    }
}

/// True if `m` is the first match in the completion list — C's
/// `is_first_match`, which compares addresses, so `None` is the head of an
/// empty list.
pub(crate) fn is_first_match(m: Option<MatchId>) -> bool {
    m == compl_first_match.get()
}

/// Is `c` part of the item being completed?  Decides whether typing it
/// abandons the completion while the menu is up.
pub fn ins_compl_accept_char(c: Key) -> bool {
    if compl_autocomplete.get() && compl_from_nonkeyword.get() {
        return false;
    }
    // The answer is a character class: asked of the codepoint.
    let c = c.code();
    if ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0 {
        // Expanding an identifier: only identifier characters.
        return vim_is_ident_char(c);
    }
    match ctrl_x_mode.get() {
        // File names, but not path separators, so that "proto/<Tab>"
        // expands files in "proto", not "proto/" as a whole.
        CTRL_X_FILES => {
            let isfilec = vim_isfilec(c);
            isfilec && !vim_ispathsep(c)
        }
        // Command line and omni completion take just about any printable
        // character, but do stop at white space.
        CTRL_X_CMDLINE | CTRL_X_CMDLINE_CTRL_X | CTRL_X_OMNI => {
            let isprintc = vim_isprintc(c);
            isprintc && !ascii_iswhite(c)
        }
        // For whole-line completion a space can be part of the line.
        CTRL_X_WHOLE_LINE => vim_isprintc(c),
        _ => vim_iswordc(c),
    }
}

/// `'completeopt'` has `fuzzy` (and this is not thesaurus completion, which
/// never fuzzy-matches).
pub(crate) fn cot_fuzzy() -> bool {
    completeopt_flags() & kOptCotFlagFuzzy != 0 && !ctrl_x_mode_thesaurus()
}

/// Matches are ordered by distance from the cursor: `'completeopt'` has
/// `nearest`, or autocompletion is on, and `fuzzy` is not overriding it.
pub(crate) fn is_nearest_active() -> bool {
    (compl_autocomplete.get() || completeopt_flags() & kOptCotFlagNearest != 0) && !cot_fuzzy()
}

pub(crate) fn ins_compl_is_match_selected() -> bool {
    shown_match().is_some_and(|shown| !shown.is_first())
}

/// Autocompletion inserting the longest common prefix: `'completeopt'` has
/// `longest` without `preinsert` or `fuzzy`.
pub fn ins_compl_preinsert_longest() -> bool {
    compl_autocomplete.get()
        && completeopt_flags() & (kOptCotFlagLongest | kOptCotFlagPreinsert | kOptCotFlagFuzzy)
            == kOptCotFlagLongest
}

/// [`ins_compl_leader`] as the string itself.
pub(crate) fn ins_compl_leader_str() -> ComplStr {
    if compl_leader().is_unset() {
        compl_orig_text()
    } else {
        compl_leader()
    }
}

pub(crate) fn ins_compl_leader_len() -> size_t {
    if compl_leader().is_unset() {
        compl_orig_text().len()
    } else {
        compl_leader().len()
    }
}

/// The shown match spans more than one line.
///
/// Upstream dereferences `compl_shown_match` here without checking it; no
/// shown match answers false.
pub(crate) fn ins_compl_has_multiple() -> bool {
    shown_match().is_some_and(|shown| shown.with(|m| m.text.contains(&b'\n')))
}

/// `lnum` is one of the lines a multi-line match is being inserted over.
///
/// A completion with a shown match is running, as for
/// [`ins_compl_has_multiple`], which this asks first.
pub(crate) fn ins_compl_lnum_in_range(lnum: LineNr) -> bool {
    ins_compl_has_multiple() && lnum >= compl_lnum.get() && lnum <= Win::current().w_cursor.lnum
}

pub(crate) fn ins_compl_has_shown_match() -> bool {
    match shown_match() {
        None => true,
        Some(shown) => shown.next() != Some(shown),
    }
}

/// The shown match is longer than what has been inserted so far.
pub(crate) fn ins_compl_long_shown_match() -> bool {
    let Some(shown) = shown_match() else {
        return false;
    };
    let typed = Win::current().w_cursor.col - compl_col.get();
    ColNr::try_from(shown.text_len()).unwrap_or(ColNr::MAX) > ColNr::new(typed)
}

/// `'completeopt'`, buffer-local value first.
///
/// With no current buffer (a lib test drives the match list without one)
/// the global value answers alone; the editor always has one.
pub(crate) fn completeopt_flags() -> c_uint {
    let local = Buf::current_or_none().map_or(0, |buf| buf.b_cot_flags);
    if local != 0 { local } else { COT_FLAGS.get() }
}

/// [`completeopt_flags`] under the name the rest of the editor calls it by.
pub fn get_cot_flags() -> c_uint {
    completeopt_flags()
}

pub fn ins_compl_active() -> bool {
    compl_started.get()
}

/// A completion is running, and `window` is the window it started in.
///
/// Safe: [`Win`] is a live window, and the two globals hold only the
/// identities it is compared against.
pub fn ins_compl_win_active(window: Win) -> bool {
    ins_compl_active()
        && compl_curr_win.get() == Some(window.id())
        && compl_curr_buf.get() == window.buffer_or_none().map(Buf::id)
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

pub fn ins_compl_col() -> ColNr {
    compl_col.get()
}

pub fn ins_compl_len() -> c_int {
    compl_length.get()
}

/// The match is previewed in the buffer rather than only in the menu:
/// `'completeopt'` has `preinsert` (with `menuone`, when autocompletion is
/// off) and not `fuzzy`.
pub fn ins_compl_has_preinsert() -> bool {
    let flags = completeopt_flags();
    if compl_autocomplete.get() && p_ic() && !p_inf() {
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
pub fn ins_compl_preinsert_effect() -> bool {
    let previewing = ins_compl_has_preinsert() || ins_compl_preinsert_longest();
    previewing && Win::current().w_cursor.col < compl_ins_end_col.get()
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
pub(crate) fn ins_compl_has_autocomplete() -> bool {
    let local = Buf::current().b_p_ac;
    if local >= 0 { local != 0 } else { p_ac() }
}

/// How much of the leader has been typed: the cursor's distance from
/// `compl_col`, never negative.
pub(crate) fn get_compl_len() -> usize {
    let off = Win::current().w_cursor.col - compl_col.get();
    usize::try_from(off).unwrap_or(0)
}

/// The CTRL-X chord dispatch: `c` is the key typed after CTRL-X.
///
/// Returns true when the completion should stop without inserting anything
/// (CTRL-X CTRL-Z).
pub(crate) fn set_ctrl_x_mode(c: Key) -> bool {
    let mut retval = false;
    'chord: {
        match c.special().ok_or(c) {
            // Scroll the window one line up or down.
            Err(Key::CTRL_E | Key::CTRL_Y) => {
                ctrl_x_mode.set(CTRL_X_SCROLL);
                EDIT_SUBMODE.set(Some(gettext(if STATE.get() & REPLACE_FLAG == 0 {
                    c" (insert) Scroll (^E/^Y)"
                } else {
                    c" (replace) Scroll (^E/^Y)"
                })));
                EDIT_SUBMODE_PRE.set(None);
                REDRAW_MODE.set(true);
                break 'chord;
            }
            // Complete whole lines.
            Err(Key::CTRL_L) => {
                ctrl_x_mode.set(CTRL_X_WHOLE_LINE);
                break 'chord;
            }
            // Complete file names.
            Err(Key::CTRL_F) => {
                ctrl_x_mode.set(CTRL_X_FILES);
                break 'chord;
            }
            // Complete words from a dictionary.
            Err(Key::CTRL_K) => {
                ctrl_x_mode.set(CTRL_X_DICTIONARY);
                break 'chord;
            }
            Err(Key::CTRL_R) => {
                // CTRL-R followed by '=' is an expression register, not
                // register completion: leave the mode alone.
                if !vpeekc().is('=') {
                    ctrl_x_mode.set(CTRL_X_REGISTER);
                }
                break 'chord;
            }
            // Complete words from a thesaurus.
            Err(Key::CTRL_T) => {
                ctrl_x_mode.set(CTRL_X_THESAURUS);
                break 'chord;
            }
            // User defined completion.
            Err(Key::CTRL_U) => {
                ctrl_x_mode.set(CTRL_X_FUNCTION);
                break 'chord;
            }
            // Omni completion.
            Err(Key::CTRL_O) => {
                ctrl_x_mode.set(CTRL_X_OMNI);
                break 'chord;
            }
            // Complete spelling suggestions.
            Err(LOWER_S | Key::CTRL_S) => {
                ctrl_x_mode.set(CTRL_X_SPELL);
                let no_emsg = Suppress::emsg(); // avoid E756 twice
                spell_back_to_badword();
                drop(no_emsg);
                break 'chord;
            }
            // Complete tag names.
            Err(Key::CTRL_RSB) => {
                ctrl_x_mode.set(CTRL_X_TAGS);
                break 'chord;
            }
            // Complete keywords from included files.
            Ok(SpecialKey::STab) | Err(Key::CTRL_I) => {
                ctrl_x_mode.set(CTRL_X_PATH_PATTERNS);
                break 'chord;
            }
            // Complete definitions from included files.
            Err(Key::CTRL_D) => {
                ctrl_x_mode.set(CTRL_X_PATH_DEFINES);
                break 'chord;
            }
            // Complete Vim commands.
            Err(Key::CTRL_V | Key::CTRL_Q) => {
                ctrl_x_mode.set(CTRL_X_CMDLINE);
                break 'chord;
            }
            // Stop completion.
            Err(Key::CTRL_Z) => {
                ctrl_x_mode.set(CTRL_X_NORMAL);
                EDIT_SUBMODE.set(None);
                REDRAW_MODE.set(true);
                retval = true;
                break 'chord;
            }
            // CTRL-X CTRL-P means LOCAL expansion if nothing interrupted (we
            // just started CTRL-X mode, or there were enough CTRL-X's to
            // cancel the previous mode, say ^X^F^X^X^P or ^P^X^X^X^P); normal
            // expansion when interrupting a different mode (^X^F^X^P or
            // ^P^X^X^P).  Nothing changes when interrupting mode 0 — the flag
            // does not change when going to ADDING mode.  -- Acevedo
            Err(Key::CTRL_P | Key::CTRL_N) => {
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
        if c == Key::CTRL_X {
            if compl_cont_mode.get() != 0 {
                compl_cont_status.set(0);
            } else {
                compl_cont_mode.set(CTRL_X_NOT_DEFINED_YET);
            }
        }
        ctrl_x_mode.set(CTRL_X_NORMAL);
        EDIT_SUBMODE.set(None);
        REDRAW_MODE.set(true);
    }
    retval
}

/// The `mode` string `complete_info()` reports, empty when no completion is
/// running.
pub(crate) fn ins_compl_mode() -> Option<&'static CStr> {
    if ctrl_x_mode_not_defined_yet() || ctrl_x_mode_scroll() || compl_started.get() {
        // Upstream indexes unconditionally: the two NULL rows of the table
        // answer a null pointer, which is what the caller then handles.
        return ctrl_x_mode_name(ctrl_x_mode.get());
    }
    Some(c"")
}

/// The `complete_info()` name of CTRL-X mode `mode` (its identifier flag
/// ignored), `None` for the two modes that have none.
pub(crate) fn ctrl_x_mode_name(mode: c_int) -> Option<&'static CStr> {
    let index = usize::try_from(mode & !CTRL_X_WANT_IDENT).expect("a CTRL-X mode is not negative");
    CTRL_X_MODE_NAMES[index]
}

/// Which way the key typed moves through the matches: BACKWARD or FORWARD.
pub(crate) fn ins_compl_key2dir(c: Key) -> Direction {
    if c == SpecialKey::Event || c == SpecialKey::Command || c == SpecialKey::Lua {
        return if PUM_WANT.get().item < compl_selected_item.get() {
            BACKWARD
        } else {
            FORWARD
        };
    }
    if c == Key::CTRL_P
        || c == Key::CTRL_L
        || c == SpecialKey::Pageup
        || c == SpecialKey::Kpageup
        || c == SpecialKey::SUp
        || c == SpecialKey::Up
    {
        return BACKWARD;
    }
    FORWARD
}

/// `c` is a completion key only while the popup menu is shown.
pub(crate) fn ins_compl_pum_key(c: Key) -> bool {
    pum_visible()
        && (c == SpecialKey::Pageup
            || c == SpecialKey::Kpageup
            || c == SpecialKey::SUp
            || c == SpecialKey::Pagedown
            || c == SpecialKey::Kpagedown
            || c == SpecialKey::SDown
            || c == SpecialKey::Up
            || c == SpecialKey::Down)
}

/// How many matches the key typed moves: one for most keys, a menu's height
/// for the page keys.
pub(crate) fn ins_compl_key2count(c: Key) -> c_int {
    if c == SpecialKey::Event || c == SpecialKey::Command || c == SpecialKey::Lua {
        return (PUM_WANT.get().item - compl_selected_item.get()).abs();
    }
    if ins_compl_pum_key(c) && c != SpecialKey::Up && c != SpecialKey::Down {
        let h = pum_get_height();
        return if h > 3 { h - 2 } else { h }; // keep some context
    }
    1
}

/// True when completing with `c` should insert the match, false when it only
/// changes which match is selected.
pub(crate) fn ins_compl_use_match(c: Key) -> bool {
    match c.special().ok_or(c) {
        Ok(
            SpecialKey::Up
            | SpecialKey::Down
            | SpecialKey::Pagedown
            | SpecialKey::Kpagedown
            | SpecialKey::SDown
            | SpecialKey::Pageup
            | SpecialKey::Kpageup
            | SpecialKey::SUp,
        ) => false,
        Ok(SpecialKey::Event | SpecialKey::Command | SpecialKey::Lua) => {
            let want = PUM_WANT.get();
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
pub fn f_preinserted(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if ins_compl_preinsert_effect() {
        result.write_number(1);
    }
}
