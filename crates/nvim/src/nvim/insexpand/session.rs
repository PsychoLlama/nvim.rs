//! Starting a completion: where it begins and what it is completing.
//!
//! [`ins_complete`] is the entry point every completion key reaches.
//! [`ins_compl_start`] decides `compl_col`, `compl_length` and the pattern by
//! asking the per-mode `get_*_compl_info` function, then hands over to
//! [`super::getexp::ins_compl_get_exp`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
#[allow(unused_imports)]
use crate::semsg_c;
use crate::src::nvim::keycodes::{Ctrl_N, Ctrl_P, Ctrl_R};
use crate::src::nvim::types::{VAR_NUMBER, VAR_STRING, VAR_UNKNOWN};

/// The pattern, column and length for normal (CTRL-N / CTRL-P) completion.
///
/// Sets `compl_col`, `compl_length` and `compl_pattern`; reads
/// `compl_cont_status` and `ctrl_x_mode`.
pub(crate) unsafe fn get_normal_compl_info(
    line: *mut c_char,
    mut startcol: c_int,
    curs_col: colnr_T,
) -> c_int {
    unsafe {
        // The pattern under construction: `prefix`, then `quote_meta` of the
        // `len` bytes at `compl_col` — the size `quote_meta` answers for a
        // null destination is exactly the room the second call needs.
        let build_pattern = |prefix: &'static CStr, len: c_int| {
            let at = prefix.count_bytes();
            let n = quote_meta(ptr::null_mut(), line.offset(compl_col.get() as isize), len)
                as size_t
                + at;
            let data = xmalloc(n).cast::<c_char>();
            strcpy(data, prefix.as_ptr());
            quote_meta(data.add(at), line.offset(compl_col.get() as isize), len);
            (data, n)
        };

        if compl_cont_status.get() & CONT_SOL != 0 || ctrl_x_mode_path_defines() {
            if !compl_status_adding() {
                while {
                    startcol -= 1;
                    startcol >= 0 && vim_isIDc(*line.offset(startcol as isize) as u8 as c_int)
                } {}
                startcol += 1;
                (*compl_col.ptr()) += startcol;
                compl_length.set(curs_col - startcol);
            }
            if p_ic.get() != 0 {
                compl_pattern.set(cstr_as_string(str_foldcase(
                    line.offset(compl_col.get() as isize),
                    compl_length.get(),
                    ptr::null_mut(),
                    0,
                )));
            } else {
                compl_pattern.set(cbuf_to_string(
                    line.offset(compl_col.get() as isize),
                    compl_length.get() as size_t,
                ));
            }
        } else if compl_status_adding() {
            // We need up to 2 extra chars for the prefix.
            let word_start = line.offset(compl_col.get() as isize);
            let prefix = if !vim_iswordp(word_start)
                || (compl_col.get() > 0 && vim_iswordp(mb_prevptr(line, word_start)))
            {
                c""
            } else {
                c"\\<"
            };
            let (data, n) = build_pattern(prefix, compl_length.get());
            (*compl_pattern.ptr()).data = data;
            (*compl_pattern.ptr()).size = n - 1;
        } else {
            // Upstream decrements in the `else if` test itself, so only these
            // two branches see the smaller column.
            startcol -= 1;
            if startcol < 0 || !vim_iswordp(mb_prevptr(line, line.offset(startcol as isize + 1))) {
                // Match any word of at least two chars.
                compl_pattern.set(cbuf_to_string(c"\\<\\k\\k".as_ptr(), 6));
                (*compl_col.ptr()) += curs_col;
                compl_length.set(0);
                compl_from_nonkeyword.set(true);
            } else {
                // Search backwards for the point where the character class
                // changes, or for a single-byte character that is not a word
                // character.
                startcol -= utf_head_off(line, line.offset(startcol as isize));
                let base_class = mb_get_class(line.offset(startcol as isize));
                loop {
                    startcol -= 1;
                    if startcol < 0 {
                        break;
                    }
                    let head_off = utf_head_off(line, line.offset(startcol as isize));
                    if base_class != mb_get_class(line.offset((startcol - head_off) as isize)) {
                        break;
                    }
                    startcol -= head_off;
                }
                startcol += 1;
                (*compl_col.ptr()) += startcol;
                compl_length.set(curs_col - startcol);
                if compl_length.get() == 1 {
                    // Only match a word with at least two chars -- webb.
                    // There's no need to call quote_meta for the size,
                    // xmalloc(7) is enough -- Acevedo.
                    let data = xmalloc(7).cast::<c_char>();
                    strcpy(data, c"\\<".as_ptr());
                    quote_meta(data.offset(2), line.offset(compl_col.get() as isize), 1);
                    strcat(data, c"\\k".as_ptr());
                    (*compl_pattern.ptr()).data = data;
                    (*compl_pattern.ptr()).size = strlen(data);
                } else {
                    let (data, n) = build_pattern(c"\\<", compl_length.get());
                    (*compl_pattern.ptr()).data = data;
                    (*compl_pattern.ptr()).size = n - 1;
                }
            }
        }

        // Call the functions in 'complete' with 'findstart=1'; ^N completion,
        // not complete() or ^X^N.
        if ctrl_x_mode_normal() && compl_cont_status.get() & CONT_LOCAL == 0 {
            setup_cpt_sources();
            prepare_cpt_compl_funcs();
        }
        OK
    }
}

/// The pattern, column and length for whole-line completion, and for the
/// `complete()` function.
pub(crate) unsafe fn get_wholeline_compl_info(line: *mut c_char, curs_col: colnr_T) -> c_int {
    unsafe {
        compl_col.set(getwhitecols(line) as colnr_T);
        compl_length.set(curs_col - compl_col.get());
        if compl_length.get() < 0 {
            // Cursor in indent: empty pattern.
            compl_length.set(0);
        }
        if p_ic.get() != 0 {
            compl_pattern.set(cstr_as_string(str_foldcase(
                line.offset(compl_col.get() as isize),
                compl_length.get(),
                ptr::null_mut(),
                0,
            )));
        } else {
            compl_pattern.set(cbuf_to_string(
                line.offset(compl_col.get() as isize),
                compl_length.get() as size_t,
            ));
        }
        OK
    }
}

/// The pattern, column and length for filename completion.
pub(crate) unsafe fn get_filename_compl_info(
    line: *mut c_char,
    mut startcol: c_int,
    curs_col: colnr_T,
) -> c_int {
    unsafe {
        // Go back to just before the first filename character.
        if startcol > 0 {
            // C's MB_PTR_BACK: step back over one whole character.
            let back =
                |p: *mut c_char| p.offset(-((utf_head_off(line, p.offset(-1)) + 1) as isize));
            let mut p = back(line.offset(startcol as isize));
            while p > line && vim_isfilec(utf_ptr2char(p)) {
                p = back(p);
            }
            // The MSWIN half of upstream's guard — a drive letter — is not
            // compiled here, so this is just the one test.
            let p_is_filec = vim_isfilec(utf_ptr2char(p));
            startcol = if p == line && p_is_filec {
                0
            } else {
                p.offset_from(line) as c_int + 1
            };
        }

        (*compl_col.ptr()) += startcol;
        compl_length.set(curs_col - startcol);
        compl_pattern.set(cstr_as_string(addstar(
            line.offset(compl_col.get() as isize),
            compl_length.get() as size_t,
            EXPAND_FILES,
        )));
        OK
    }
}

/// The pattern, column and length for command-line completion.
pub(crate) unsafe fn get_cmdline_compl_info(line: *mut c_char, curs_col: colnr_T) -> c_int {
    unsafe {
        compl_pattern.set(cbuf_to_string(line, curs_col as size_t));
        set_cmd_context(
            compl_xp.ptr(),
            (*compl_pattern.ptr()).data,
            (*compl_pattern.ptr()).size as c_int,
            curs_col,
            false,
        );
        if (*compl_xp.ptr()).xp_context == EXPAND_LUA {
            nlua_expand_pat(compl_xp.ptr());
        }
        if (*compl_xp.ptr()).xp_context == EXPAND_UNSUCCESSFUL
            || (*compl_xp.ptr()).xp_context == EXPAND_NOTHING
        {
            // No completion possible: use an empty pattern to get a
            // "pattern not found" message.
            compl_col.set(curs_col);
        } else {
            compl_col.set(
                (*compl_xp.ptr())
                    .xp_pattern
                    .offset_from((*compl_pattern.ptr()).data) as colnr_T,
            );
        }
        compl_length.set(curs_col - compl_col.get());
        OK
    }
}

/// Set `compl_col`, `compl_length`, `compl_pattern` and `cpt_compl_pattern`.
pub(crate) unsafe fn set_compl_globals(mut startcol: c_int, curs_col: colnr_T, is_cpt_compl: bool) {
    unsafe {
        if is_cpt_compl {
            clear_string(&cpt_compl_pattern);
            if startcol < compl_col.get() {
                prepend_startcol_text(cpt_compl_pattern.ptr(), compl_orig_text.ptr(), startcol);
            } else {
                cpt_compl_pattern.set(copy_string(compl_orig_text.get(), ptr::null_mut()));
            }
        } else {
            if startcol < 0 || startcol > curs_col {
                startcol = curs_col;
            }
            // Re-obtain the line in case it has changed.
            let line = ml_get((*curwin.get()).w_cursor.lnum);
            let len = curs_col - startcol;
            compl_pattern.set(cbuf_to_string(
                line.offset(startcol as isize),
                len as size_t,
            ));
            compl_col.set(startcol);
            compl_length.set(len);
        }
    }
}

/// The pattern, column and length for user-defined completion
/// (`'omnifunc'`, `'completefunc'` and `'thesaurusfunc'`).
///
/// `cb` is set when a function in `'complete'` triggered this, null otherwise;
/// `startcol`, when not null, receives the column the function answered.
pub(crate) unsafe fn get_userdefined_compl_info(
    curs_col: colnr_T,
    mut cb: *mut Callback,
    startcol: *mut c_int,
) -> c_int {
    unsafe {
        // Call the user-defined function with "a:findstart" set to 1 to obtain
        // the length of the text to complete.
        let save_State = State.get();

        let is_cpt_function = !cb.is_null();
        if !is_cpt_function {
            if *get_complete_funcname(ctrl_x_mode.get()) as c_int == NUL {
                semsg_c!(
                    gettext(&raw const e_notset as *const c_char),
                    if ctrl_x_mode_function() {
                        c"completefunc".as_ptr()
                    } else {
                        c"omnifunc".as_ptr()
                    },
                );
                return FAIL;
            }
            cb = get_insert_callback(ctrl_x_mode.get());
        }

        let mut args = [TYPVAL_T_INIT; 3];
        args[0].v_type = VAR_NUMBER;
        args[1].v_type = VAR_STRING;
        args[2].v_type = VAR_UNKNOWN;
        args[0].vval.v_number = 1;
        args[1].vval.v_string = c"".as_ptr().cast_mut();

        let pos = (*curwin.get()).w_cursor;
        (*textlock.ptr()) += 1;
        let col = callback_call_retnr(cb, 2, args.as_mut_ptr()) as colnr_T;
        (*textlock.ptr()) -= 1;

        State.set(save_State);
        (*curwin.get()).w_cursor = pos; // restore the cursor position
        check_cursor(curwin.get()); // make sure the position is valid, just in case
        validate_cursor(curwin.get());
        if !equalpos((*curwin.get()).w_cursor, pos) {
            emsg(gettext(E_COMPLDEL.as_ptr()));
            return FAIL;
        }

        if !startcol.is_null() {
            *startcol = col;
        }

        // -2 means the function wants to cancel the completion without an
        // error; do the same if it did not execute successfully.
        if col == -2 || aborting() {
            return FAIL;
        }

        // -3 does the same as -2 and leaves CTRL-X mode.
        if col == -3 {
            if is_cpt_function {
                return FAIL;
            }
            ctrl_x_mode.set(CTRL_X_NORMAL);
            edit_submode.set(ptr::null_mut());
            if !shortmess(SHM_COMPLETIONMENU) {
                msg_clr_cmdline();
            }
            return FAIL;
        }

        // Reset the extended parameters of completion when starting a new one.
        compl_opt_refresh_always.set(false);

        if !is_cpt_function {
            set_compl_globals(col, curs_col, false);
        }
        OK
    }
}

/// The pattern, column and length for spell completion; reads `spell_bad_len`.
pub(crate) unsafe fn get_spell_compl_info(startcol: c_int, curs_col: colnr_T) -> c_int {
    unsafe {
        if spell_bad_len.get() > 0 {
            debug_assert!(spell_bad_len.get() <= c_int::MAX as size_t);
            compl_col.set(curs_col - spell_bad_len.get() as c_int);
        } else {
            compl_col.set(spell_word_start(startcol) as colnr_T);
        }
        if compl_col.get() >= startcol {
            compl_length.set(0);
            compl_col.set(curs_col);
        } else {
            spell_expand_check_cap(compl_col.get());
            compl_length.set(curs_col - compl_col.get());
        }
        // Need to obtain "line" again, it may have become invalid.
        let line = ml_get((*curwin.get()).w_cursor.lnum);
        compl_pattern.set(cbuf_to_string(
            line.offset(compl_col.get() as isize),
            compl_length.get() as size_t,
        ));
        OK
    }
}

/// The completion pattern, column and length for whichever CTRL-X mode is
/// running; `line_invalid` is set when the current line may have become
/// invalid and needs fetching again.
pub(crate) unsafe fn compl_get_info(
    line: *mut c_char,
    startcol: c_int,
    curs_col: colnr_T,
    line_invalid: *mut bool,
) -> c_int {
    unsafe {
        if ctrl_x_mode_normal()
            || ctrl_x_mode_register()
            || (ctrl_x_mode.get() & CTRL_X_WANT_IDENT != 0
                && !thesaurus_func_complete(ctrl_x_mode.get()))
        {
            if get_normal_compl_info(line, startcol, curs_col) != OK {
                return FAIL;
            }
            *line_invalid = true; // 'cpt' func may have invalidated "line"
        } else if ctrl_x_mode_line_or_eval() {
            return get_wholeline_compl_info(line, curs_col);
        } else if ctrl_x_mode_files() {
            return get_filename_compl_info(line, startcol, curs_col);
        } else if ctrl_x_mode.get() == CTRL_X_CMDLINE {
            return get_cmdline_compl_info(line, curs_col);
        } else if ctrl_x_mode_function()
            || ctrl_x_mode_omni()
            || thesaurus_func_complete(ctrl_x_mode.get())
        {
            if get_userdefined_compl_info(curs_col, ptr::null_mut(), ptr::null_mut()) != OK {
                return FAIL;
            }
            *line_invalid = true; // "line" may have become invalid
        } else if ctrl_x_mode_spell() {
            if get_spell_compl_info(startcol, curs_col) == FAIL {
                return FAIL;
            }
            *line_invalid = true; // "line" may have become invalid
        } else {
            internal_error(c"ins_complete()".as_ptr());
            return FAIL;
        }
        OK
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
pub(crate) unsafe fn ins_compl_continue_search(line: *mut c_char) {
    unsafe {
        // It is a continued search.
        (*compl_cont_status.ptr()) &= !CONT_INTRPT; // remove INTRPT
        if ctrl_x_mode_normal() || ctrl_x_mode_path_patterns() || ctrl_x_mode_path_defines() {
            if (*compl_startpos.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
                // The line (probably) wrapped: set compl_startpos to the first
                // non-blank in the line. If that is not a word character we
                // include it to get a better pattern, but then we don't want
                // the "\\<" prefix — checked below.
                compl_col.set(getwhitecols(line) as colnr_T);
                (*compl_startpos.ptr()).col = compl_col.get();
                (*compl_startpos.ptr()).lnum = (*curwin.get()).w_cursor.lnum;
                (*compl_cont_status.ptr()) &= !CONT_SOL; // clear SOL if present
            } else {
                // S_IPOS was set when we inserted a word that was at the
                // beginning of the line, which means that we'll go to SOL
                // mode, but first we need to redefine compl_startpos.
                if compl_cont_status.get() & CONT_S_IPOS != 0 {
                    (*compl_cont_status.ptr()) |= CONT_SOL;
                    (*compl_startpos.ptr()).col = skipwhite(
                        line.offset((compl_length.get() + (*compl_startpos.ptr()).col) as isize),
                    )
                    .offset_from(line) as colnr_T;
                }
                compl_col.set((*compl_startpos.ptr()).col);
            }
            compl_length.set((*curwin.get()).w_cursor.col - compl_col.get());
            // IObuff is used to add a "word from the next line"; would we have
            // enough space?  Just being paranoid.
            if compl_length.get() > IOSIZE - MIN_SPACE {
                (*compl_cont_status.ptr()) &= !CONT_SOL;
                compl_length.set(IOSIZE - MIN_SPACE);
                compl_col.set((*curwin.get()).w_cursor.col - compl_length.get());
            }
            (*compl_cont_status.ptr()) |= CONT_ADDING | CONT_N_ADDS;
            if compl_length.get() < 1 {
                (*compl_cont_status.ptr()) &= CONT_LOCAL;
            }
        } else if ctrl_x_mode_line_or_eval() || ctrl_x_mode_register() {
            compl_cont_status.set(CONT_ADDING | CONT_N_ADDS);
        } else {
            compl_cont_status.set(0);
        }
    }
}

/// Start insert-mode completion.
pub(crate) unsafe fn ins_compl_start() -> c_int {
    unsafe {
        // C's `kv_destroy(compl_orig_extmarks)`.
        let destroy_orig_extmarks = || {
            xfree((*compl_orig_extmarks.ptr()).items.cast::<c_void>());
            compl_orig_extmarks.set(EXTMARK_UNDO_VEC_INIT);
        };

        // First time we hit ^N or ^P (in a row, I mean).
        let save_did_ai = did_ai.get();
        did_ai.set(false);
        did_si.set(false);
        can_si.set(false);
        can_si_back.set(false);
        if stop_arrow() == FAIL {
            did_ai.set(save_did_ai);
            return FAIL;
        }

        let mut line = ml_get((*curwin.get()).w_cursor.lnum);
        let curs_col = (*curwin.get()).w_cursor.col;
        compl_pending.set(0);
        compl_lnum.set((*curwin.get()).w_cursor.lnum);

        if compl_cont_status.get() & CONT_INTRPT == CONT_INTRPT
            && compl_cont_mode.get() == ctrl_x_mode.get()
        {
            // This same ctrl-x mode was interrupted previously: continue the
            // completion.
            ins_compl_continue_search(line);
        } else {
            (*compl_cont_status.ptr()) &= CONT_LOCAL;
        }

        let mut startcol = 0; // column where the searched text starts
        if !compl_status_adding() {
            // Normal expansion.
            compl_cont_mode.set(ctrl_x_mode.get());
            if ctrl_x_mode_not_default() {
                // Remove LOCAL if ctrl_x_mode != CTRL_X_NORMAL.
                compl_cont_status.set(0);
            }
            (*compl_cont_status.ptr()) |= CONT_N_ADDS;
            compl_startpos.set((*curwin.get()).w_cursor);
            startcol = curs_col;
            compl_col.set(0);
        }

        // Work out the completion pattern and original text -- webb.
        let mut line_invalid = false;
        if compl_get_info(line, startcol, curs_col, &raw mut line_invalid) == FAIL {
            if ctrl_x_mode_function()
                || ctrl_x_mode_omni()
                || thesaurus_func_complete(ctrl_x_mode.get())
            {
                // Restore did_ai, so that adding a comment leader works.
                did_ai.set(save_did_ai);
            }
            return FAIL;
        }
        // If "line" was changed while getting the completion info, get it again.
        if line_invalid {
            line = ml_get((*curwin.get()).w_cursor.lnum);
        }

        if compl_status_adding() {
            if !shortmess(SHM_COMPLETIONMENU) {
                edit_submode_pre.set(gettext(c" Adding".as_ptr()));
            }
            if ctrl_x_mode_line_or_eval() {
                // Insert a new line, keep indentation but ignore 'comments'.
                let old = (*curbuf.get()).b_p_com;
                (*curbuf.get()).b_p_com = c"".as_ptr().cast_mut();
                (*compl_startpos.ptr()).lnum = (*curwin.get()).w_cursor.lnum;
                (*compl_startpos.ptr()).col = compl_col.get();
                ins_eol('\r' as c_int);
                (*curbuf.get()).b_p_com = old;
                compl_length.set(0);
                compl_col.set((*curwin.get()).w_cursor.col);
                compl_lnum.set((*curwin.get()).w_cursor.lnum);
            }
        } else {
            edit_submode_pre.set(ptr::null_mut());
            (*compl_startpos.ptr()).col = compl_col.get();
        }

        if !shortmess(SHM_COMPLETIONMENU) && !compl_autocomplete.get() {
            if compl_cont_status.get() & CONT_LOCAL != 0 {
                edit_submode.set(ctrl_x_msg(CTRL_X_LOCAL_MSG));
            } else {
                edit_submode.set(ctrl_x_msg(ctrl_x_mode.get()));
            }
        }

        // If any of the original typed text has been changed we need to fix
        // the redo buffer.
        ins_compl_fixRedoBufForLeader(ptr::null_mut());

        // Always add a completion for the original text.
        clear_string(&compl_orig_text);
        destroy_orig_extmarks();
        compl_orig_text.set(cbuf_to_string(
            line.offset(compl_col.get() as isize),
            compl_length.get() as size_t,
        ));
        save_orig_extmarks();
        let mut flags = CP_ORIGINAL_TEXT;
        if p_ic.get() != 0 {
            flags |= CP_ICASE;
        }
        if ins_compl_add(
            (*compl_orig_text.ptr()).data,
            (*compl_orig_text.ptr()).size as c_int,
            ptr::null_mut(),
            ptr::null(),
            false,
            ptr::null_mut(),
            kDirectionNotSet,
            flags,
            false,
            ptr::null(),
            FUZZY_SCORE_NONE,
        ) != OK
        {
            clear_string(&compl_pattern);
            clear_string(&compl_orig_text);
            destroy_orig_extmarks();
            did_ai.set(save_did_ai);
            return FAIL;
        }

        // showmode() might reset the internal line pointers, so it must be
        // called before line = ml_get(), or when this address is no longer
        // needed. -- Acevedo.
        if !shortmess(SHM_COMPLETIONMENU) && !compl_autocomplete.get() {
            edit_submode_extra.set(gettext(c"-- Searching...".as_ptr()));
            edit_submode_highl.set(HLF_COUNT);
            showmode();
            edit_submode_extra.set(ptr::null_mut());
            ui_flush();
        }

        did_ai.set(save_did_ai);
        OK
    }
}

/// Do Insert mode completion, called when the character `c` was typed and it
/// means something for completion; answers OK, or FAIL if something failed.
pub unsafe fn ins_complete(c: c_int, enable_pum: bool) -> c_int {
    unsafe {
        // Milliseconds of `'autocompletelinger'` elapsed since collection began.
        let elapsed_ms = |start: uint64_t| os_hrtime().wrapping_sub(start) / 1_000_000;

        let disable_ac_delay = compl_started.get()
            && ctrl_x_mode_normal()
            && (c == Ctrl_N || c == Ctrl_P || c == Ctrl_R || ins_compl_pum_key(c));

        compl_direction.set(ins_compl_key2dir(c));
        let insert_match = ins_compl_use_match(c);

        if !compl_started.get() {
            if ins_compl_start() == FAIL {
                return FAIL;
            }
        } else if insert_match && stop_arrow() == FAIL {
            return FAIL;
        }

        // Time when match collection starts.
        let mut compl_start_tv: uint64_t = 0;
        if compl_autocomplete.get() && p_acl.get() > 0 && !disable_ac_delay {
            compl_start_tv = os_hrtime();
        }
        compl_curr_win.set(curwin.get());
        compl_curr_buf.set((*curwin.get()).w_buffer);
        compl_shown_match.set(compl_curr_match.get());
        compl_shows_dir.set(compl_direction.get());
        compl_num_bests.set(0);

        // Find the next match (and the following matches).
        let save_w_wrow = (*curwin.get()).w_wrow;
        let save_w_leftcol = (*curwin.get()).w_leftcol;
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
        let no_matches_found = is_first_match((*compl_first_match.get()).cp_next);
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
                (*compl_cont_status.ptr()) &= !CONT_N_ADDS;
            }
        }

        if (*compl_curr_match.get()).cp_flags & CP_CONT_S_IPOS != 0 {
            (*compl_cont_status.ptr()) |= CONT_S_IPOS;
        } else {
            (*compl_cont_status.ptr()) &= !CONT_S_IPOS;
        }

        if !shortmess(SHM_COMPLETIONMENU) && !compl_autocomplete.get() {
            ins_compl_show_statusmsg();
        }

        // Wait for the autocompletion delay to expire.
        if compl_autocomplete.get()
            && p_acl.get() > 0
            && !disable_ac_delay
            && !no_matches_found
            && elapsed_ms(compl_start_tv) < p_acl.get() as uint64_t
        {
            setcursor();
            ui_flush();
            loop {
                if char_avail() {
                    if ins_compl_preinsert_effect() && ins_compl_win_active(curwin.get()) {
                        ins_compl_delete(false); // Remove pre-inserted text
                        compl_ins_end_col.set(compl_col.get());
                    }
                    ins_compl_restart();
                    compl_interrupted.set(true);
                    break;
                }
                os_delay(2, true);
                if elapsed_ms(compl_start_tv) >= p_acl.get() as uint64_t {
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
        OK
    }
}

/// Move the cursor back to the start of the bad word, recording its length in
/// `spell_bad_len`.
pub(crate) unsafe fn spell_back_to_badword() {
    unsafe {
        let mut tpos = (*curwin.get()).w_cursor;
        spell_bad_len.set(spell_move_to(
            curwin.get(),
            BACKWARD,
            SMT_ALL,
            true,
            ptr::null_mut(),
        ));
        if (*curwin.get()).w_cursor.col != tpos.col {
            start_arrow(&raw mut tpos);
        }
    }
}
