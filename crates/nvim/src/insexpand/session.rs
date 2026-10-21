//! Starting a completion: where it begins and what it is completing.
//!
//! [`ins_complete`] is the entry point every completion key reaches.
//! [`ins_compl_start`] decides `compl_col`, `compl_length` and the pattern by
//! asking the per-mode `get_*_compl_info` function, then hands over to
//! [`super::getexp::ins_compl_get_exp`].

#![forbid(unsafe_code)]

use super::*;
use crate::charset::display::foldcase;
use crate::charset::is_word_at;
use crate::charset::skip;
use crate::cstr::{self, byte_at};
use crate::guard::Lock;
use crate::keycodes::{Ctrl_N, Ctrl_P, Ctrl_R};
use crate::mbyte::{char_at, class_in, head_off};
use crate::memline::Lines;
use crate::memory::XString;
use crate::semsg;
use crate::types::{ExpandContext, Failed, IOSIZE, ShmFlag};
use crate::winlayer::{Buf, Win};

/// C's `compl_startpos.lnum = curwin->w_cursor.lnum; compl_startpos.col = col;`
/// — the completion's anchor moved to `col` on the cursor's line.
fn set_compl_startpos_here(col: ColNr) {
    let lnum = Win::current().w_cursor.lnum;
    compl_startpos.set(compl_startpos.get().with_lnum(lnum).with_col(col));
}

/// The pattern, column and length for normal (CTRL-N / CTRL-P) completion.
///
/// Sets `compl_col`, `compl_length` and `compl_pattern`; reads
/// `compl_cont_status` and `ctrl_x_mode`.
pub(crate) fn get_normal_compl_info(mut startcol: c_int, curs_col: ColNr) -> Result<(), Failed> {
    // A copy of the cursor line, taken here rather than handed in: the
    // `'complete'` functions called at the bottom run user code, and every
    // caller of the `get_*_compl_info` family used to re-fetch the line
    // afterwards for exactly that reason.
    let line: Vec<u8> = Lines::current().line(Win::current().w_cursor.lnum).to_vec();
    let line: &[u8] = &line;
    let buffer = Buf::current();
    // The text from column `col`, read as the C string it was: past the end
    // is NUL.
    let from = |col: c_int| line.get(col as usize..).unwrap_or_default();
    // The start of the character before column `col`: C's `mb_prevptr`.
    let prev = |col: c_int| {
        let col = col as usize;
        if col == 0 {
            0
        } else {
            col - 1 - head_off(line, col - 1)
        }
    };
    let is_word = |col: usize| is_word_at(line.get(col..).unwrap_or_default(), buffer);
    // The pattern under construction: `prefix`, then `quote_meta` of the
    // `len` bytes at `compl_col`, measured at the first NUL as upstream's
    // `strlen` measured it.
    let build_pattern = |prefix: &[u8], len: c_int| {
        let mut pattern = prefix.to_vec();
        quote_meta(&mut pattern, from(compl_col.get()), len);
        pattern.truncate(cstr::until_nul(&pattern).len());
        pattern
    };

    if compl_cont_status.get() & CONT_SOL != 0 || ctrl_x_mode_path_defines() {
        if !compl_status_adding() {
            while {
                startcol -= 1;
                startcol >= 0 && vim_is_ident_char(c_int::from(byte_at(line, startcol as usize)))
            } {}
            startcol += 1;
            compl_col.set(compl_col.get() + startcol);
            compl_length.set(curs_col - startcol);
        }
        compl_pattern().set(compl_pattern_from_line());
    } else if compl_status_adding() {
        // We need up to 2 extra chars for the prefix.
        let word_start = compl_col.get();
        let prefix: &[u8] =
            if !is_word(word_start as usize) || (word_start > 0 && is_word(prev(word_start))) {
                b""
            } else {
                b"\\<"
            };
        let pattern = build_pattern(prefix, compl_length.get());
        compl_pattern().set_string(XString::from_vec(pattern));
    } else {
        // Upstream decrements in the `else if` test itself, so only these
        // two branches see the smaller column.
        startcol -= 1;
        if startcol < 0 || !is_word(prev(startcol + 1)) {
            // Match any word of at least two chars.
            compl_pattern().set(String_0::from_bytes(b"\\<\\k\\k"));
            compl_col.set(compl_col.get() + curs_col);
            compl_length.set(0);
            compl_from_nonkeyword.set(true);
        } else {
            // Search backwards for the point where the character class
            // changes, or for a single-byte character that is not a word
            // character.
            startcol -= head_off(line, startcol as usize) as c_int;
            let base_class = class_in(from(startcol), buffer);
            loop {
                startcol -= 1;
                if startcol < 0 {
                    break;
                }
                let head_off = head_off(line, startcol as usize) as c_int;
                if base_class != class_in(from(startcol - head_off), buffer) {
                    break;
                }
                startcol -= head_off;
            }
            startcol += 1;
            compl_col.set(compl_col.get() + startcol);
            compl_length.set(curs_col - startcol);
            let mut pattern = if compl_length.get() == 1 {
                // Only match a word with at least two chars -- webb.
                build_pattern(b"\\<", 1)
            } else {
                build_pattern(b"\\<", compl_length.get())
            };
            if compl_length.get() == 1 {
                pattern.extend_from_slice(b"\\k");
            }
            compl_pattern().set_string(XString::from_vec(pattern));
        }
    }

    // Call the functions in 'complete' with 'findstart=1'; ^N completion,
    // not complete() or ^X^N.
    if ctrl_x_mode_normal() && compl_cont_status.get() & CONT_LOCAL == 0 {
        setup_cpt_sources();
        prepare_cpt_compl_funcs();
    }
    Ok(())
}

/// The pattern, column and length for whole-line completion, and for the
/// `complete()` function.
pub(crate) fn get_wholeline_compl_info(curs_col: ColNr) -> Result<(), Failed> {
    let mut lines = Lines::current();
    let line = lines.line(Win::current().w_cursor.lnum);
    compl_col.set(ColNr::try_from(skip::white(line)).unwrap_or(0));
    compl_length.set(curs_col - compl_col.get());
    if compl_length.get() < 0 {
        // Cursor in indent: empty pattern.
        compl_length.set(0);
    }
    compl_pattern().set(compl_pattern_from_line());
    Ok(())
}

/// The pattern, column and length for filename completion.
pub(crate) fn get_filename_compl_info(mut startcol: c_int, curs_col: ColNr) -> Result<(), Failed> {
    let mut lines = Lines::current();
    let line = lines.line(Win::current().w_cursor.lnum);
    // Go back to just before the first filename character.
    if startcol > 0 {
        // C's MB_PTR_BACK: step back over one whole character.
        let back = |p: usize| p - (head_off(line, p - 1) + 1);
        let is_filec = |p: usize| vim_isfilec(char_at(line.get(p..).unwrap_or_default()));
        let mut p = back(startcol as usize);
        while p > 0 && is_filec(p) {
            p = back(p);
        }
        // The MSWIN half of upstream's guard — a drive letter — is not
        // compiled here, so this is just the one test.
        startcol = if p == 0 && is_filec(p) {
            0
        } else {
            p as c_int + 1
        };
    }

    compl_col.set(compl_col.get() + startcol);
    compl_length.set(curs_col - startcol);
    let start = (compl_col.get() as usize).min(line.len());
    let end = (start + compl_length.get().max(0) as usize).min(line.len());
    let typed = &line[start..end];
    compl_pattern().set(String_0::from_bytes(&addstar(typed, ExpandContext::Files)));
    Ok(())
}

/// The pattern, column and length for command-line completion.
pub(crate) fn get_cmdline_compl_info(curs_col: ColNr) -> Result<(), Failed> {
    // The expansion context is kept for `get_next_cmdline_completion`, and
    // moved out for the length of this call: `nlua_expand_pat` runs Lua.
    let mut xp = compl_xp.take().unwrap_or_default();
    let pattern = {
        let mut lines = Lines::current();
        let line = lines.line(Win::current().w_cursor.lnum);
        let to = usize::try_from(curs_col).unwrap_or(0).min(line.len());
        String_0::from_bytes(&line[..to])
    };
    compl_pattern().set(pattern);
    let line = compl_pattern().to_vec();
    set_cmd_context(&mut xp, &line, curs_col, false);
    if xp.context == ExpandContext::Lua {
        nlua_expand_pat(&mut xp);
    }
    if xp.context == ExpandContext::Unsuccessful || xp.context == ExpandContext::Nothing {
        // No completion possible: use an empty pattern to get a
        // "pattern not found" message.
        compl_col.set(curs_col);
    } else {
        // The pattern is an offset into the context's copy of
        // `compl_pattern`, which is the same offset into it.
        compl_col.set(xp.pattern as ColNr);
    }
    compl_length.set(curs_col - compl_col.get());
    compl_xp.set(Some(xp));
    Ok(())
}

/// Set `compl_col`, `compl_length`, `compl_pattern` and `cpt_compl_pattern`.
pub(crate) fn set_compl_globals(mut startcol: c_int, curs_col: ColNr, is_cpt_compl: bool) {
    if is_cpt_compl {
        cpt_compl_pattern().clear();
        if startcol < compl_col.get() {
            prepend_startcol_text(cpt_compl_pattern(), compl_orig_text(), startcol);
        } else {
            cpt_compl_pattern().set(compl_orig_text().to_owned());
        }
    } else {
        if startcol < 0 || startcol > curs_col {
            startcol = curs_col;
        }
        let len = curs_col - startcol;
        compl_col.set(startcol);
        compl_length.set(len);
        // The line is read afresh inside: the user function that got here may
        // have changed it.
        compl_pattern().set(compl_text_from_line());
    }
}

/// The pattern, column and length for user-defined completion
/// (`'omnifunc'`, `'completefunc'` and `'thesaurusfunc'`).
///
/// `cb` is set when a function in `'complete'` triggered this (the caller
/// holds it for the call), `None` otherwise. Answers whether to go on, and
/// the column the function answered -- 0 when it was not reached.
pub(crate) fn get_userdefined_compl_info(
    curs_col: ColNr,
    cb: Option<&Callback>,
) -> (Result<(), Failed>, c_int) {
    // Call the user-defined function with "a:findstart" set to 1 to obtain
    // the length of the text to complete.
    let save_state = State.get();

    let is_cpt_function = cb.is_some();
    let held;
    let cb = match cb {
        Some(cb) => cb,
        None => {
            if get_complete_funcname(ctrl_x_mode.get()).is_empty() {
                let arg0 = if ctrl_x_mode_function() {
                    "completefunc"
                } else {
                    "omnifunc"
                };
                semsg!("E764: Option '{arg0}' is not set");
                return (Err(Failed), 0);
            }
            // A second reference: the function may reset the option.
            held = get_insert_callback(ctrl_x_mode.get());
            &*held
        }
    };

    // The empty string is the argument's own, released with it.
    let args = [TypVal::Number(1), TypVal::string_from(b"")];

    let pos = Win::current().w_cursor;
    let locked = Lock::text();
    let col = callback_call_retnr(cb, &args) as ColNr;
    drop(locked);

    State.set(save_state);
    Win::current().w_cursor = pos; // restore the cursor position
    check_cursor(Win::current()); // make sure the position is valid, just in case
    validate_cursor(Win::current());
    if !equalpos(Win::current().w_cursor, pos) {
        emsg(gettext(E_COMPLDEL));
        return (Err(Failed), 0);
    }

    // -2 means the function wants to cancel the completion without an
    // error; do the same if it did not execute successfully.
    if col == -2 || aborting() {
        return (Err(Failed), col);
    }

    // -3 does the same as -2 and leaves CTRL-X mode.
    if col == -3 {
        if is_cpt_function {
            return (Err(Failed), col);
        }
        ctrl_x_mode.set(CTRL_X_NORMAL);
        edit_submode.set(None);
        if !shortmess(ShmFlag::COMPLETIONMENU) {
            msg_clr_cmdline();
        }
        return (Err(Failed), col);
    }

    // Reset the extended parameters of completion when starting a new one.
    compl_opt_refresh_always.set(false);

    if !is_cpt_function {
        set_compl_globals(col, curs_col, false);
    }
    (Ok(()), col)
}

/// The pattern, column and length for spell completion; reads `spell_bad_len`.
pub(crate) fn get_spell_compl_info(startcol: c_int, curs_col: ColNr) -> Result<(), Failed> {
    if spell_bad_len.get() > 0 {
        debug_assert!(spell_bad_len.get() <= c_int::MAX as size_t);
        compl_col.set(curs_col - spell_bad_len.get() as c_int);
    } else {
        compl_col.set(spell_word_start(startcol) as ColNr);
    }
    if compl_col.get() >= startcol {
        compl_length.set(0);
        compl_col.set(curs_col);
    } else {
        spell_expand_check_cap(compl_col.get());
        compl_length.set(curs_col - compl_col.get());
    }
    // The line is read afresh inside: `spell_expand_check_cap` above may
    // have moved it.
    compl_pattern().set(compl_text_from_line());
    Ok(())
}

/// The completion pattern, column and length for whichever CTRL-X mode is
/// running.
///
/// Upstream handed the cursor line down here and reported back, through a
/// `line_invalid` out-parameter, which of the branches might have moved it.
/// Each of them reads the line for itself now, so there is nothing to
/// report and nothing to re-fetch.
pub(crate) fn compl_get_info(startcol: c_int, curs_col: ColNr) -> Result<(), Failed> {
    if ctrl_x_mode_normal()
        || ctrl_x_mode_register()
        || (ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0
            && !thesaurus_func_complete(ctrl_x_mode.get()))
    {
        get_normal_compl_info(startcol, curs_col)
    } else if ctrl_x_mode_line_or_eval() {
        get_wholeline_compl_info(curs_col)
    } else if ctrl_x_mode_files() {
        get_filename_compl_info(startcol, curs_col)
    } else if ctrl_x_mode.get() == CTRL_X_CMDLINE {
        get_cmdline_compl_info(curs_col)
    } else if ctrl_x_mode_function()
        || ctrl_x_mode_omni()
        || thesaurus_func_complete(ctrl_x_mode.get())
    {
        // No 'complete' function asked for this, and the column is not
        // wanted.
        get_userdefined_compl_info(curs_col, None).0
    } else if ctrl_x_mode_spell() {
        get_spell_compl_info(startcol, curs_col)
    } else {
        // SAFETY: a static NUL-terminated name.
        internal_error(c"ins_complete()");
        Err(Failed)
    }
}

/// Continue an interrupted completion-mode search in `line`.
///
/// When this same `ctrl_x_mode` was interrupted, the text from `compl_startpos`
/// to the cursor becomes the pattern for adding a new word rather than
/// expanding the one before the cursor. Word-wise, if `compl_startpos` is not
/// on the cursor's line it is fixed up first (the line was split because it
/// was longer than 'tw'). With SOL set, the previous pattern is skipped: a
/// word at the start of the line was inserted and that is what we look for.
///
pub(crate) fn ins_compl_continue_search() {
    // It is a continued search.
    compl_cont_status.set(compl_cont_status.get() & !CONT_INTRPT); // remove INTRPT
    if ctrl_x_mode_normal() || ctrl_x_mode_path_patterns() || ctrl_x_mode_path_defines() {
        if compl_startpos.get().lnum != Win::current().w_cursor.lnum {
            // The line (probably) wrapped: set compl_startpos to the first
            // non-blank in the line. If that is not a word character we
            // include it to get a better pattern, but then we don't want
            // the "\\<" prefix — checked below.
            let mut lines = Lines::current();
            let line = lines.line(Win::current().w_cursor.lnum);
            compl_col.set(ColNr::try_from(skip::white(line)).unwrap_or(0));
            set_compl_startpos_here(compl_col.get());
            compl_cont_status.set(compl_cont_status.get() & !CONT_SOL); // clear SOL if present
        } else {
            // S_IPOS was set when we inserted a word that was at the
            // beginning of the line, which means that we'll go to SOL
            // mode, but first we need to redefine compl_startpos.
            if compl_cont_status.get() & CONT_S_IPOS != 0 {
                compl_cont_status.set(compl_cont_status.get() | CONT_SOL);
                let skip =
                    usize::try_from(compl_length.get() + compl_startpos.get().col).unwrap_or(0);
                let mut lines = Lines::current();
                let line = lines.line(Win::current().w_cursor.lnum);
                let from = skip.min(line.len());
                let col = ColNr::try_from(from + skip::white(&line[from..])).unwrap_or(0);
                compl_startpos.set(compl_startpos.get().with_col(col));
            }
            compl_col.set(compl_startpos.get().col);
        }
        compl_length.set(Win::current().w_cursor.col - compl_col.get());
        // An `IOSIZE` buffer is used to add a "word from the next
        // line"; would we have enough space?  Just being paranoid.
        if compl_length.get() > IOSIZE - MIN_SPACE {
            compl_cont_status.set(compl_cont_status.get() & !CONT_SOL);
            compl_length.set(IOSIZE - MIN_SPACE);
            compl_col.set(Win::current().w_cursor.col - compl_length.get());
        }
        compl_cont_status.set(compl_cont_status.get() | CONT_ADDING | CONT_N_ADDS);
        if compl_length.get() < 1 {
            compl_cont_status.set(compl_cont_status.get() & CONT_LOCAL);
        }
    } else if ctrl_x_mode_line_or_eval() || ctrl_x_mode_register() {
        compl_cont_status.set(CONT_ADDING | CONT_N_ADDS);
    } else {
        compl_cont_status.set(0);
    }
}

/// Start insert-mode completion.
pub(crate) fn ins_compl_start() -> Result<(), Failed> {
    // First time we hit ^N or ^P (in a row, I mean).
    let save_did_ai = did_ai.get();
    did_ai.set(false);
    did_si.set(false);
    can_si.set(false);
    can_si_back.set(false);
    if stop_arrow().is_err() {
        did_ai.set(save_did_ai);
        return Err(Failed);
    }

    let curs_col = Win::current().w_cursor.col;
    compl_pending.set(0);
    compl_lnum.set(Win::current().w_cursor.lnum);

    if compl_cont_status.get() & CONT_INTRPT == CONT_INTRPT
        && compl_cont_mode.get() == ctrl_x_mode.get()
    {
        // This same ctrl-x mode was interrupted previously: continue the
        // completion.
        ins_compl_continue_search();
    } else {
        compl_cont_status.set(compl_cont_status.get() & CONT_LOCAL);
    }

    let mut startcol = 0; // column where the searched text starts
    if !compl_status_adding() {
        // Normal expansion.
        compl_cont_mode.set(ctrl_x_mode.get());
        if ctrl_x_mode_not_default() {
            // Remove LOCAL if ctrl_x_mode != CTRL_X_NORMAL.
            compl_cont_status.set(0);
        }
        compl_cont_status.set(compl_cont_status.get() | CONT_N_ADDS);
        compl_startpos.set(Win::current().w_cursor);
        startcol = curs_col;
        compl_col.set(0);
    }

    // Work out the completion pattern and original text -- webb.
    if compl_get_info(startcol, curs_col).is_err() {
        if ctrl_x_mode_function()
            || ctrl_x_mode_omni()
            || thesaurus_func_complete(ctrl_x_mode.get())
        {
            // Restore did_ai, so that adding a comment leader works.
            did_ai.set(save_did_ai);
        }
        return Err(Failed);
    }

    if compl_status_adding() {
        if !shortmess(ShmFlag::COMPLETIONMENU) {
            edit_submode_pre.set(Some(gettext(c" Adding")));
        }
        if ctrl_x_mode_line_or_eval() {
            // Insert a new line, keep indentation but ignore 'comments'.
            let old = Buf::current().b_p_com.replace(XString::new());
            set_compl_startpos_here(compl_col.get());
            ins_eol('\r' as c_int);
            Buf::current().b_p_com = old;
            compl_length.set(0);
            compl_col.set(Win::current().w_cursor.col);
            compl_lnum.set(Win::current().w_cursor.lnum);
        }
    } else {
        edit_submode_pre.set(None);
        compl_startpos.set(compl_startpos.get().with_col(compl_col.get()));
    }

    if !shortmess(ShmFlag::COMPLETIONMENU) && !compl_autocomplete.get() {
        if compl_cont_status.get() & CONT_LOCAL != 0 {
            edit_submode.set(ctrl_x_msg(CTRL_X_LOCAL_MSG));
        } else {
            edit_submode.set(ctrl_x_msg(ctrl_x_mode.get()));
        }
    }

    // If any of the original typed text has been changed we need to fix
    // the redo buffer.
    ins_compl_fix_redo_buf_for_leader(None);

    // Always add a completion for the original text.
    compl_orig_text().clear();
    compl_orig_extmarks().clear();
    compl_orig_text().set(compl_text_from_line());
    compl_orig_extmarks().save();
    let mut flags = CP_ORIGINAL_TEXT;
    if p_ic() {
        flags |= CP_ICASE;
    }
    if ins_compl_add_orig_text(flags).is_err() {
        compl_pattern().clear();
        compl_orig_text().clear();
        compl_orig_extmarks().clear();
        did_ai.set(save_did_ai);
        return Err(Failed);
    }

    // Upstream held the line here and warned that showmode() resets the
    // memline's pointers, so this had to come after the last use of it
    // (-- Acevedo). Nothing holds a line across it now.
    if !shortmess(ShmFlag::COMPLETIONMENU) && !compl_autocomplete.get() {
        edit_submode_extra.set(Some(XString::from_cstr(gettext(c"-- Searching..."))));
        edit_submode_highl.set(HLF_COUNT);
        showmode();
        edit_submode_extra.set(None);
        ui_flush();
    }

    did_ai.set(save_did_ai);
    Ok(())
}

/// Do Insert mode completion, called when the character `c` was typed and it
/// means something for completion; answers `Ok`, or `Err` if something failed.
pub fn ins_complete(c: c_int, enable_pum: bool) -> Result<(), Failed> {
    // Milliseconds of `'autocompletelinger'` elapsed since collection began.
    let elapsed_ms = |start: uint64_t| os_hrtime().wrapping_sub(start) / 1_000_000;

    let disable_ac_delay = compl_started.get()
        && ctrl_x_mode_normal()
        && (c == Ctrl_N || c == Ctrl_P || c == Ctrl_R || ins_compl_pum_key(c));

    compl_direction.set(ins_compl_key2dir(c));
    let insert_match = ins_compl_use_match(c);

    if !compl_started.get() {
        ins_compl_start()?;
    } else if insert_match && stop_arrow().is_err() {
        return Err(Failed);
    }

    // Time when match collection starts.
    let mut compl_start_tv: uint64_t = 0;
    if compl_autocomplete.get() && p_acl() > 0 && !disable_ac_delay {
        compl_start_tv = os_hrtime();
    }
    // The identities, taken while both are provably live: the completion
    // below runs user functions and Lua, and only the *identity* survives
    // that (see `compl_curr_win`).
    compl_curr_win.set(Some(Win::current().id()));
    compl_curr_buf.set(Win::current().buffer_or_none().map(Buf::id));
    compl_shown_match.set(compl_curr_match.get());
    compl_shows_dir.set(compl_direction.get());
    compl_num_bests.set(0);

    // Find the next match (and the following matches).
    let save_w_wrow = Win::current().w_wrow;
    let save_w_leftcol = Win::current().w_leftcol;
    let n = ins_compl_next(true, ins_compl_key2count(c), insert_match);

    // Reset the autocompletion timer expiry flag.
    if compl_autocomplete.get() {
        compl_time_slice_expired.set(false);
    }

    if n > 1 {
        // All matches have been found.
        compl_matches.set(n);
    }
    compl_curr_match.set(compl_shown_match.get());
    compl_direction.set(compl_shows_dir.get());

    // Eat the ESC that vgetc() returns after a CTRL-C, to avoid leaving
    // Insert mode.
    if got_int.get() && global_busy.get() == 0 {
        vgetc();
        got_int.set(false);
    }

    // We found no match if the list has only the "compl_orig_text" entry.
    // Upstream dereferences `compl_first_match` here without checking.
    let head = first_match().expect("a completion has its original text");
    let no_matches_found = is_first_match(head.next());
    if no_matches_found {
        // Remove the N_ADDS flag, so the next ^X<> won't try to go to
        // ADDING mode, because we couldn't expand anything in the first
        // place; but if we used ^P, ^N, ^X^I or ^X^D we might want to
        // add-expand a single-char word (such as M in M'exico) if not
        // tried already. -- Acevedo
        if compl_length.get() > 1
            || compl_status_adding()
            || (ctrl_x_mode_not_default()
                && !ctrl_x_mode_path_patterns()
                && !ctrl_x_mode_path_defines())
        {
            compl_cont_status.set(compl_cont_status.get() & !CONT_N_ADDS);
        }
    }

    let curr = curr_match().expect("a running completion has a current match");
    if curr.with(|m| m.flags) & CP_CONT_S_IPOS != 0 {
        compl_cont_status.set(compl_cont_status.get() | CONT_S_IPOS);
    } else {
        compl_cont_status.set(compl_cont_status.get() & !CONT_S_IPOS);
    }

    if !shortmess(ShmFlag::COMPLETIONMENU) && !compl_autocomplete.get() {
        ins_compl_show_statusmsg();
    }

    // Wait for the autocompletion delay to expire.
    if compl_autocomplete.get()
        && p_acl() > 0
        && !disable_ac_delay
        && !no_matches_found
        && elapsed_ms(compl_start_tv) < p_acl() as uint64_t
    {
        setcursor();
        ui_flush();
        loop {
            if char_avail() {
                if ins_compl_preinsert_effect() && ins_compl_win_active(Win::current()) {
                    ins_compl_delete(false); // Remove pre-inserted text
                    compl_ins_end_col.set(compl_col.get());
                }
                ins_compl_restart();
                compl_interrupted.set(true);
                break;
            }
            os_delay(2, true);
            if elapsed_ms(compl_start_tv) >= p_acl() as uint64_t {
                break;
            }
        }
    }

    // Show the popup menu, unless we got interrupted.
    if enable_pum && !compl_interrupted.get() {
        show_pum(save_w_wrow, save_w_leftcol);
    }
    compl_was_interrupted.set(compl_interrupted.get());
    compl_interrupted.set(false);
    Ok(())
}

/// Move the cursor back to the start of the bad word, recording its length in
/// `spell_bad_len`.
pub(crate) fn spell_back_to_badword() {
    let tpos = Win::current().w_cursor;
    let win = Win::current();
    spell_bad_len.set(spell_move_to(win, BACKWARD, SMT_ALL, true, None));
    if Win::current().w_cursor.col != tpos.col {
        start_arrow(Some(InsertEnd::At(tpos)));
    }
}

/// The `compl_length` bytes of the cursor line at `compl_col`, case-folded
/// when `'ignorecase'` is on: what every caller here wants for
/// `compl_pattern`.
pub(crate) fn compl_pattern_from_line() -> String_0 {
    let mut lines = Lines::current();
    let text = compl_range(lines.line(Win::current().w_cursor.lnum));
    if p_ic() {
        String_0::from_bytes(&foldcase(text, None))
    } else {
        String_0::from_bytes(text)
    }
}

/// The same bytes, copied verbatim -- what `compl_orig_text` and the
/// command-line pattern want.
pub(crate) fn compl_text_from_line() -> String_0 {
    let mut lines = Lines::current();
    String_0::from_bytes(compl_range(lines.line(Win::current().w_cursor.lnum)))
}

/// `compl_col .. compl_col + compl_length` of `line`, clamped to it.
fn compl_range(line: &[u8]) -> &[u8] {
    let at = usize::try_from(compl_col.get())
        .unwrap_or(0)
        .min(line.len());
    let len = usize::try_from(compl_length.get()).unwrap_or(0);
    &line[at..at.saturating_add(len).min(line.len())]
}
