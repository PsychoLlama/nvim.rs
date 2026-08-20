//! `'incsearch'`: previewing a `/`, `?` or `:s` pattern while it is typed.
//!
//! [`may_do_incsearch_highlighting`] runs on every command-line change and,
//! for the commands that take a pattern, searches from the saved view state
//! and highlights what it found.  [`parse_pattern_and_range`] is what decides
//! whether the line being typed *is* such a command, and the `viewstate_T`
//! pair saves and restores the window the preview scrolled.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

use crate::guard::Suppress;
use crate::types::{ExpandContext, FAIL, NUL, OK};
use core::ffi::CStr;

/// Fire a `Cmdline*` / `Cmdwin*` autocommand whose `<afile>` and `<amatch>`
/// are the command-line type character.
pub(crate) unsafe fn trigger_cmd_autocmd(typechar: ::core::ffi::c_int, evt: event_T) {
    unsafe {
        let mut typestr: [::core::ffi::c_char; 2] =
            [typechar as ::core::ffi::c_char, NUL as ::core::ffi::c_char];
        apply_autocmds(
            evt,
            typestr.as_mut_ptr(),
            typestr.as_mut_ptr(),
            false,
            curbuf.get(),
        );
    }
}

/// Record everything about `wp`'s view that an incremental search may scroll.
pub(crate) unsafe fn save_viewstate(wp: *mut win_T, vs: *mut viewstate_T) {
    unsafe {
        (*vs).vs_curswant = (*wp).w_curswant;
        (*vs).vs_leftcol = (*wp).w_leftcol;
        (*vs).vs_skipcol = (*wp).w_skipcol;
        (*vs).vs_topline = (*wp).w_topline;
        (*vs).vs_topfill = (*wp).w_topfill;
        (*vs).vs_botline = (*wp).w_botline;
        (*vs).vs_empty_rows = (*wp).w_empty_rows;
    }
}

/// Put back what [`save_viewstate`] recorded.
pub(crate) unsafe fn restore_viewstate(wp: *mut win_T, vs: *mut viewstate_T) {
    unsafe {
        (*wp).w_curswant = (*vs).vs_curswant;
        (*wp).w_leftcol = (*vs).vs_leftcol;
        (*wp).w_skipcol = (*vs).vs_skipcol;
        (*wp).w_topline = (*vs).vs_topline;
        (*wp).w_topfill = (*vs).vs_topfill;
        (*wp).w_botline = (*vs).vs_botline;
        (*wp).w_empty_rows = (*vs).vs_empty_rows;
    }
}

/// Start an incremental search from where the cursor and view are now.
pub(crate) unsafe fn init_incsearch_state(s: *mut incsearch_state_T) {
    unsafe {
        (*s).winid = (*curwin.get()).handle;
        (*s).match_start = (*curwin.get()).w_cursor;
        (*s).did_incsearch = false;
        (*s).incsearch_postponed = false;
        (*s).magic_overruled_save = magic_overruled.get();
        clearpos(&mut (*s).match_end);
        (*s).save_cursor = (*curwin.get()).w_cursor;
        (*s).search_start = (*curwin.get()).w_cursor;
        save_viewstate(curwin.get(), &raw mut (*s).init_viewstate);
        save_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
    }
}

/// Move `t` to the end of the match the last search found, clamped to the
/// last line of the buffer.
pub(crate) unsafe fn set_search_match(t: *mut pos_T) {
    unsafe {
        (*t).lnum += search_match_lines.get();
        (*t).col = search_match_endcol.get();
        if (*t).lnum > (*curbuf.get()).b_ml.ml_line_count {
            (*t).lnum = (*curbuf.get()).b_ml.ml_line_count;
            coladvance(curwin.get(), MAXCOL);
        }
    }
}

/// Parse a `:[range]s/foo`-style command line into the parts `'incsearch'`
/// and wildmenu completion need.
///
/// Answers true when the line holds a valid pattern, having set `skiplen`
/// (bytes before the pattern), `patlen`, `search_delim`, `search_first_line`
/// and `search_last_line`.
pub unsafe fn parse_pattern_and_range(
    incsearch_start: *mut pos_T,
    search_delim: *mut ::core::ffi::c_int,
    skiplen: *mut ::core::ffi::c_int,
    patlen: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut delim_optional = false;
        let mut dummy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut magic: magic_T = 0;

        *skiplen = 0;
        *patlen = (*ccline.ptr()).cmdlen;

        // Default range: all lines.
        search_first_line.set(0);
        search_last_line.set(MAXLNUM as linenr_T);

        let mut ea = exarg_T {
            line1: 1,
            line2: 1,
            cmd: (*ccline.ptr()).cmdbuff,
            addr_type: CmdAddr::Lines,
            ..EXARG_T_INIT
        };

        // Uninitialised in the C; `parse_command_modifiers` only writes it.
        let mut dummy_cmdmod: cmdmod_T = CMDMOD_T_INIT;
        parse_command_modifiers(&raw mut ea, &raw mut dummy, &raw mut dummy_cmdmod, true);

        // Skip over the range to find the command.
        let cmd = skip_range(ea.cmd, ::core::ptr::null_mut::<ExpandContext>());
        if vim_strchr(c"sgvlu".as_ptr(), *cmd as uint8_t as ::core::ffi::c_int).is_null() {
            return false;
        }

        // Skip over the command name to find the pattern separator.
        let mut p = cmd;
        while ascii_isalpha(*p as ::core::ffi::c_int) {
            p = p.offset(1);
        }
        if *skipwhite(p) as ::core::ffi::c_int == NUL {
            return false;
        }

        // C's `strncmp(cmd, name, MAX(p - cmd, min))`: is what has been typed
        // an abbreviation of `name`?  `min` is the shortest form upstream
        // accepts, and 0 where it imposes none (`p` never precedes `cmd`, so
        // `MAX(p - cmd, 0)` is just `p - cmd`).
        let namelen = p.offset_from(cmd);
        let abbreviates =
            |name: &CStr, min: isize| strncmp(cmd, name.as_ptr(), namelen.max(min) as size_t) == 0;

        if abbreviates(c"substitute", 0)
            || abbreviates(c"smagic", 0)
            || abbreviates(c"snomagic", 3)
            || abbreviates(c"vglobal", 0)
        {
            if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                && *cmd.offset(1) as ::core::ffi::c_int == 'm' as ::core::ffi::c_int
            {
                magic_overruled.set(OPTION_MAGIC_ON);
            } else if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                && *cmd.offset(1) as ::core::ffi::c_int == 'n' as ::core::ffi::c_int
            {
                magic_overruled.set(OPTION_MAGIC_OFF);
            }
        } else if abbreviates(c"sort", 3) || abbreviates(c"uniq", 3) {
            // Skip over "!" and the flags.
            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                p = skipwhite(p.offset(1));
            }
            // C's `while (ASCII_ISALPHA(*(p = skipwhite(p)))) p++;` — the
            // macro evaluates its argument up to four times, which is a no-op
            // because `skipwhite` is idempotent.
            loop {
                p = skipwhite(p);
                if !ascii_isalpha(*p as ::core::ffi::c_int) {
                    break;
                }
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == NUL {
                return false;
            }
        } else if abbreviates(c"vimgrep", 3)
            || abbreviates(c"vimgrepadd", 8)
            || abbreviates(c"lvimgrep", 2)
            || abbreviates(c"lvimgrepadd", 9)
            || abbreviates(c"global", 0)
        {
            // Skip an optional "!".
            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                p = p.offset(1);
                if *skipwhite(p) as ::core::ffi::c_int == NUL {
                    return false;
                }
            }
            if *cmd as ::core::ffi::c_int != 'g' as ::core::ffi::c_int {
                delim_optional = true;
            }
        } else {
            return false;
        }

        p = skipwhite(p);
        let delim = if delim_optional && vim_is_ident_char(*p as uint8_t as ::core::ffi::c_int) {
            ' ' as ::core::ffi::c_int
        } else {
            let c = *p as ::core::ffi::c_int;
            p = p.offset(1);
            c
        };
        *search_delim = delim;

        let end = skip_regexp_ex(
            p,
            delim,
            magic_isset() as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut magic,
        );
        let use_last_pat = end == p && *end as ::core::ffi::c_int == delim;

        if end == p && !use_last_pat {
            return false;
        }

        // Skip if the pattern matches everything (e.g. for 'hlsearch').
        if !use_last_pat {
            let c = *end;
            *end = NUL as ::core::ffi::c_char;
            let empty = empty_pattern_magic(p, end.offset_from(p) as size_t, magic);
            *end = c;
            if empty {
                return false;
            }
        }

        // Found a non-empty pattern, or "//".
        *skiplen = p.offset_from((*ccline.ptr()).cmdbuff) as ::core::ffi::c_int;
        *patlen = end.offset_from(p) as ::core::ffi::c_int;

        // Parse the address range.
        let save_cursor = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = *incsearch_start;

        parse_cmd_address(&raw mut ea, &raw mut dummy, true);

        if ea.addr_count > 0 {
            // Allow for a reverse match.
            search_first_line.set(ea.line2.min(ea.line1));
            search_last_line.set(ea.line2.max(ea.line1));
        } else if *cmd.offset(0) as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            && *cmd.offset(1) as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
        {
            // :s defaults to the current line.
            search_last_line.set((*curwin.get()).w_cursor.lnum);
            search_first_line.set(search_last_line.get());
        }

        (*curwin.get()).w_cursor = save_cursor;
        true
    }
}

/// Whether `'incsearch'` highlighting should be done for this command line,
/// and if so what its pattern and address range are.
///
/// May change the last search pattern.
pub(crate) unsafe fn do_incsearch_highlighting(
    firstc: ::core::ffi::c_int,
    search_delim: *mut ::core::ffi::c_int,
    is_state: *mut incsearch_state_T,
    skiplen: *mut ::core::ffi::c_int,
    patlen: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        *skiplen = 0;
        *patlen = (*ccline.ptr()).cmdlen;

        if p_is.get() == 0 || cmd_silent.get() {
            return false;
        }

        // By default search all lines.
        search_first_line.set(0);
        search_last_line.set(MAXLNUM as linenr_T);

        if firstc == '/' as ::core::ffi::c_int || firstc == '?' as ::core::ffi::c_int {
            *search_delim = firstc;
            return true;
        }

        if firstc != ':' as ::core::ffi::c_int {
            return false;
        }

        let _no_emsg = Suppress::emsg();
        parse_pattern_and_range(
            &raw mut (*is_state).search_start,
            search_delim,
            skiplen,
            patlen,
        )
    }
}

/// Do the `'incsearch'` preview, if it is wanted here.
pub(crate) unsafe fn may_do_incsearch_highlighting(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    s: *mut incsearch_state_T,
) {
    unsafe {
        let cc = ccline.ptr();
        let mut skiplen = 0;
        let mut patlen = 0;
        let mut search_delim = 0;

        // Parsing the range may already set the last search pattern.
        // NOTE: restore_last_search_pattern() must run before every return.
        save_last_search_pattern();

        if !do_incsearch_highlighting(
            firstc,
            &raw mut search_delim,
            s,
            &raw mut skiplen,
            &raw mut patlen,
        ) {
            restore_last_search_pattern();
            finish_incsearch_highlighting(false, s, true);
            return;
        }

        // If there is a character waiting, search and redraw later.
        if char_avail() {
            restore_last_search_pattern();
            (*s).incsearch_postponed = true;
            return;
        }
        (*s).incsearch_postponed = false;

        // Use the previous pattern for ":s//".
        let mut next_char = *(*cc).cmdbuff.offset((skiplen + patlen) as isize);
        let use_last_pat = patlen == 0
            && skiplen > 0
            && *(*cc).cmdbuff.offset((skiplen - 1) as isize) == next_char;

        if patlen != 0 || use_last_pat {
            ui_busy_start();
            ui_flush();
        }

        if search_first_line.get() == 0 {
            // Start at the original cursor position.
            (*curwin.get()).w_cursor = (*s).search_start;
        } else if search_first_line.get() > (*curbuf.get()).b_ml.ml_line_count {
            // Start after the last line.
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            (*curwin.get()).w_cursor.col = MAXCOL;
        } else {
            // Start at the first line in the range.
            (*curwin.get()).w_cursor.lnum = search_first_line.get();
            (*curwin.get()).w_cursor.col = 0;
        }

        // The do_search() result.
        let mut found = 0;

        if patlen != 0 || use_last_pat {
            let mut search_flags = SEARCH_OPT + SEARCH_NOOF + SEARCH_PEEK;
            if p_hls.get() == 0 {
                search_flags += SEARCH_KEEP;
            }
            if search_first_line.get() != 0 {
                search_flags += SEARCH_START;
            }
            // Half a second of search time.
            let mut tm: proftime_T = profile_setlimit(500);
            let mut sia = searchit_arg_T {
                sa_stop_lnum: 0,
                sa_tm: &raw mut tm,
                sa_timed_out: 0,
                sa_wrapped: 0,
            };
            *(*cc).cmdbuff.offset((skiplen + patlen) as isize) = NUL as ::core::ffi::c_char;
            // So it doesn't beep on a bad expression.
            let no_emsg = Suppress::emsg();
            found = do_search(
                ::core::ptr::null_mut::<oparg_T>(),
                if firstc == ':' as ::core::ffi::c_int {
                    '/' as ::core::ffi::c_int
                } else {
                    firstc
                },
                search_delim,
                (*cc).cmdbuff.offset(skiplen as isize),
                patlen as size_t,
                count,
                search_flags,
                &raw mut sia,
            );
            drop(no_emsg);
            *(*cc).cmdbuff.offset((skiplen + patlen) as isize) = next_char;

            if (*curwin.get()).w_cursor.lnum < search_first_line.get()
                || (*curwin.get()).w_cursor.lnum > search_last_line.get()
            {
                // The match is outside the address range.
                found = 0;
                (*curwin.get()).w_cursor = (*s).search_start;
            }

            // Interrupted while searching: behave as if it failed.
            if got_int.get() {
                // Remove <C-C> from the input stream.
                vpeekc();
                // Don't abandon the command line.
                got_int.set(false);
                found = 0;
            } else if char_avail() {
                // Searching was cancelled because a character was typed.
                (*s).incsearch_postponed = true;
            }
            ui_busy_stop();
        } else {
            // Turn off the previous highlight.
            set_no_hlsearch(true);
            redraw_all_later(UPD_SOME_VALID);
        }

        // Add or remove the search match position.
        highlight_match.set(found != 0);

        // First restore the old curwin values, so the screen is positioned the
        // same way the real search command would leave it.
        restore_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
        changed_cline_bef_curs(curwin.get());
        update_topline(curwin.get());

        let mut end_pos = (*curwin.get()).w_cursor;
        if found != 0 {
            (*s).match_start = (*curwin.get()).w_cursor;
            set_search_match(&raw mut (*curwin.get()).w_cursor);
            validate_cursor(curwin.get());
            (*s).match_end = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = end_pos;
            end_pos = (*s).match_end;
        }

        // Disable 'hlsearch' highlighting if the pattern matches everything.
        // Avoids a flash when typing "foo\|".
        if !use_last_pat {
            next_char = *(*cc).cmdbuff.offset((skiplen + patlen) as isize);
            *(*cc).cmdbuff.offset((skiplen + patlen) as isize) = NUL as ::core::ffi::c_char;
            if empty_pattern(
                (*cc).cmdbuff.offset(skiplen as isize),
                patlen as size_t,
                search_delim,
            ) && !no_hlsearch.get()
            {
                redraw_all_later(UPD_SOME_VALID);
                set_no_hlsearch(true);
            }
            *(*cc).cmdbuff.offset((skiplen + patlen) as isize) = next_char;
        }

        validate_cursor(curwin.get());

        // May redraw the status line to show the cursor position.
        if p_ru.get() != 0 && ((*curwin.get()).w_status_height > 0 || global_stl_height() > 0) {
            (*curwin.get()).w_redr_status = true;
        }

        redraw_later(curwin.get(), UPD_SOME_VALID);
        update_screen();
        highlight_match.set(false);
        restore_last_search_pattern();

        // Leave the cursor at the end so CTRL-R CTRL-W works — but not when
        // it is beyond the end of the pattern, as for ":s/pat/".
        if *(*cc).cmdbuff.offset((skiplen + patlen) as isize) as ::core::ffi::c_int != NUL {
            (*curwin.get()).w_cursor = (*s).search_start;
        } else if found != 0 {
            (*curwin.get()).w_cursor = end_pos;
            // Mark as valid for the cmdline_show redraw.
            (*curwin.get()).w_valid_cursor = end_pos;
        }

        msg_starthere();
        redrawcmdline();
        (*s).did_incsearch = true;
    }
}

/// CTRL-L: add the character under the match to the pattern, and say so in
/// `*c`.
///
/// Answers `OK` when the caller should treat the key as unchanged.
pub(crate) unsafe fn may_add_char_to_search(
    firstc: ::core::ffi::c_int,
    c: *mut ::core::ffi::c_int,
    s: *mut incsearch_state_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut skiplen = 0;
        let mut patlen = 0;
        let mut search_delim = 0;

        // Parsing the range may already set the last search pattern.
        // NOTE: restore_last_search_pattern() must run before every return.
        save_last_search_pattern();

        if !do_incsearch_highlighting(
            firstc,
            &raw mut search_delim,
            s,
            &raw mut skiplen,
            &raw mut patlen,
        ) {
            restore_last_search_pattern();
            return FAIL;
        }
        restore_last_search_pattern();

        if (*s).did_incsearch {
            (*curwin.get()).w_cursor = (*s).match_end;
            *c = gchar_cursor();
            if *c != NUL {
                // With 'ignorecase' and 'smartcase' set and no uppercase in
                // the command line, lowercase the character.
                if p_ic.get() != 0
                    && p_scs.get() != 0
                    && !pat_has_uppercase((*ccline.ptr()).cmdbuff.offset(skiplen as isize))
                {
                    *c = mb_tolower(*c);
                }
                if *c == search_delim
                    || !vim_strchr(
                        if magic_isset() {
                            c"\\~^$.*[".as_ptr()
                        } else {
                            c"\\^$".as_ptr()
                        },
                        *c,
                    )
                    .is_null()
                {
                    // Put a backslash before the special characters.
                    stuff_readbuf_char(*c);
                    *c = '\\' as ::core::ffi::c_int;
                }
                // Add any composing characters.
                if utf_char2len(*c) != utfc_ptr2len(get_cursor_pos_ptr()) {
                    let save_c = *c;
                    while utf_char2len(*c) != utfc_ptr2len(get_cursor_pos_ptr()) {
                        (*curwin.get()).w_cursor.col += utf_char2len(*c);
                        *c = gchar_cursor();
                        stuff_readbuf_char(*c);
                    }
                    *c = save_c;
                }
                return FAIL;
            }
        }
        OK
    }
}

/// Undo the preview: put the cursor and the view back where the command line
/// found them, and clear the match highlight.
pub(crate) unsafe fn finish_incsearch_highlighting(
    gotesc: bool,
    s: *mut incsearch_state_T,
    call_update_screen: bool,
) {
    unsafe {
        if !(*s).did_incsearch {
            return;
        }

        (*s).did_incsearch = false;
        if gotesc {
            (*curwin.get()).w_cursor = (*s).save_cursor;
        } else {
            if !equalpos((*s).save_cursor, (*s).search_start) {
                // Put the previous-context mark at the original position.
                (*curwin.get()).w_cursor = (*s).save_cursor;
                setpcmark();
            }
            (*curwin.get()).w_cursor = (*s).search_start;
        }
        restore_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
        highlight_match.set(false);

        // By default search all lines.
        search_first_line.set(0);
        search_last_line.set(MAXLNUM as linenr_T);

        magic_overruled.set((*s).magic_overruled_save);

        // Needed for TAB.
        validate_cursor(curwin.get());
        status_redraw_all();
        redraw_all_later(UPD_SOME_VALID);
        if call_update_screen {
            update_screen();
        }
    }
}

/// CTRL-G / CTRL-T: move the `'incsearch'` preview to the next or previous
/// match.
///
/// Answers `OK` when there was no incremental search to move, `FAIL`
/// otherwise (which is what tells the key loop the line did not change).
pub(crate) unsafe fn may_do_command_line_next_incsearch(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    s: *mut incsearch_state_T,
    next_match: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let cc = ccline.ptr();
        let mut skiplen = 0;
        let mut patlen = 0;
        let mut search_delim = 0;

        // Parsing the range may already set the last search pattern.
        // NOTE: restore_last_search_pattern() must run before every return.
        save_last_search_pattern();

        if !do_incsearch_highlighting(
            firstc,
            &raw mut search_delim,
            s,
            &raw mut skiplen,
            &raw mut patlen,
        ) {
            restore_last_search_pattern();
            return OK;
        }
        if patlen == 0 && *(*cc).cmdbuff.offset(skiplen as isize) as ::core::ffi::c_int == NUL {
            restore_last_search_pattern();
            return FAIL;
        }

        ui_busy_start();
        ui_flush();

        let mut search_flags = SEARCH_NOOF;

        let pat: *mut ::core::ffi::c_char;
        if search_delim == *(*cc).cmdbuff.offset(skiplen as isize) as ::core::ffi::c_int {
            pat = last_search_pattern();
            if pat.is_null() {
                restore_last_search_pattern();
                return FAIL;
            }
            skiplen = 0;
            patlen = last_search_pattern_len() as ::core::ffi::c_int;
        } else {
            pat = (*cc).cmdbuff.offset(skiplen as isize);
        }

        // Do not search for the search end delimiter, unless it is part of
        // the pattern.
        let mut bslsh = false;
        if patlen > 2 && firstc == *pat.offset((patlen - 1) as isize) as ::core::ffi::c_int {
            patlen -= 1;
            if *pat.offset((patlen - 1) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
            {
                *pat.offset((patlen - 1) as isize) = firstc as uint8_t as ::core::ffi::c_char;
                bslsh = true;
            }
        }

        let mut t: pos_T;
        if next_match {
            t = (*s).match_end;
            if lt((*s).match_start, (*s).match_end) {
                // Start searching at the end of the match, not at the
                // beginning of the next column.
                decl(&raw mut t);
            }
            search_flags += SEARCH_COL;
        } else {
            t = (*s).match_start;
        }
        if p_hls.get() == 0 {
            search_flags += SEARCH_KEEP;
        }

        let no_emsg = Suppress::emsg();
        let save = *pat.offset(patlen as isize);
        *pat.offset(patlen as isize) = NUL as ::core::ffi::c_char;
        let found = searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut t,
            ::core::ptr::null_mut::<pos_T>(),
            if next_match { FORWARD } else { BACKWARD } as Direction,
            pat,
            patlen as size_t,
            count,
            search_flags,
            RE_SEARCH as ::core::ffi::c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        );
        drop(no_emsg);
        *pat.offset(patlen as isize) = save;
        if bslsh {
            *pat.offset((patlen - 1) as isize) = '\\' as ::core::ffi::c_char;
        }
        ui_busy_stop();

        if found != 0 {
            (*s).search_start = (*s).match_start;
            (*s).match_end = t;
            (*s).match_start = t;
            if !next_match && firstc != '?' as ::core::ffi::c_int {
                // Move just before the current match, so that when nv_search
                // finishes the cursor is put back on the match.
                (*s).search_start = t;
                decl(&raw mut (*s).search_start);
            } else if next_match && firstc == '?' as ::core::ffi::c_int {
                // Move just after the current match, for the same reason.
                (*s).search_start = t;
                incl(&raw mut (*s).search_start);
            }
            if lt(t, (*s).search_start) && next_match {
                // Wrapped around.
                (*s).search_start = t;
                if firstc == '?' as ::core::ffi::c_int {
                    incl(&raw mut (*s).search_start);
                } else {
                    decl(&raw mut (*s).search_start);
                }
            }

            set_search_match(&raw mut (*s).match_end);
            (*curwin.get()).w_cursor = (*s).match_start;
            changed_cline_bef_curs(curwin.get());
            update_topline(curwin.get());
            validate_cursor(curwin.get());
            highlight_match.set(true);
            save_viewstate(curwin.get(), &raw mut (*s).old_viewstate);
            redraw_later(curwin.get(), UPD_NOT_VALID);
            update_screen();
            highlight_match.set(false);
            redrawcmdline();
            (*curwin.get()).w_cursor = (*s).match_end;
        } else {
            vim_beep(kOptBoFlagError as ::core::ffi::c_uint);
        }

        restore_last_search_pattern();
        FAIL
    }
}

/// Guess whether the pattern matches everything.
///
/// Only finds specific cases, such as a trailing `\|`, which can happen while
/// a pattern is being typed.
pub(crate) unsafe fn empty_pattern(
    p: *mut ::core::ffi::c_char,
    len: size_t,
    delim: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut magic_val: magic_T = MAGIC_ON;

        if len > 0 {
            skip_regexp_ex(
                p,
                delim,
                magic_isset() as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut magic_val,
            );
        } else {
            return true;
        }

        empty_pattern_magic(p, len, magic_val)
    }
}

/// [`empty_pattern`] with the `'magic'` level already known.
pub(crate) unsafe fn empty_pattern_magic(
    p: *mut ::core::ffi::c_char,
    mut len: size_t,
    magic_val: magic_T,
) -> bool {
    unsafe {
        // Remove a trailing \v and the like.
        while len >= 2
            && *p.add(len - 2) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && !vim_strchr(
                c"mMvVcCZ".as_ptr(),
                *p.add(len - 1) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            len -= 2;
        }

        // True if the pattern is empty, or ends with \| and 'magic' is set (or
        // ends with '|' and very magic is set).
        len == 0
            || len > 1
                && *p.add(len - 1) as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                && (*p.add(len - 2) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    && magic_val == MAGIC_ON
                    || *p.add(len - 2) as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
                        && magic_val == MAGIC_ALL)
    }
}
