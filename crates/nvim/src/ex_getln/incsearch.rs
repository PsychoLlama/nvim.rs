//! `'incsearch'`: previewing a `/`, `?` or `:s` pattern while it is typed.
//!
//! [`may_do_incsearch_highlighting`] runs on every command-line change and,
//! for the commands that take a pattern, searches from the saved view state
//! and highlights what it found.  [`parse_pattern_and_range`] is what decides
//! whether the line being typed *is* such a command, and the `viewstate_T`
//! pair saves and restores the window the preview scrolled.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};

use crate::guard::Suppress;
use crate::types::{ExpandContext, FAIL, NUL, OK};
use core::ffi::CStr;

/// Fire a `Cmdline*` / `Cmdwin*` autocommand whose `<afile>` and `<amatch>`
/// are the command-line type character.
pub(crate) fn trigger_cmd_autocmd(typechar: ::core::ffi::c_int, evt: event_T) {
    let mut typestr: [::core::ffi::c_char; 2] =
        [typechar as ::core::ffi::c_char, NUL as ::core::ffi::c_char];
    cmdline_autocmd(evt, typestr.as_mut_ptr());
}

/// C's `apply_autocmds(evt, fname, fname, false, curbuf)`: the one shape
/// every `Cmdline*`/`Cmdwin*` event is fired with, where `<afile>` and
/// `<amatch>` are the same string.
pub(crate) fn cmdline_autocmd(evt: event_T, fname: *mut ::core::ffi::c_char) -> bool {
    // SAFETY: `fname` is a live NUL-terminated string of the caller's frame,
    // and `curbuf` is a live buffer.
    unsafe { apply_autocmds(evt, fname, fname, false, curbuf.get()) }
}

/// Record everything about `wp`'s view that an incremental search may scroll.
///
/// C wrote through a `viewstate_T *`; the structure is seven scalars and
/// `Copy`, so answering by value says the same thing in safe code.
pub(crate) fn save_viewstate(wp: Win) -> viewstate_T {
    viewstate_T {
        vs_curswant: wp.w_curswant,
        vs_leftcol: wp.w_leftcol,
        vs_skipcol: wp.w_skipcol,
        vs_topline: wp.w_topline,
        vs_topfill: wp.w_topfill,
        vs_botline: wp.w_botline,
        vs_empty_rows: wp.w_empty_rows,
    }
}

/// Put back what [`save_viewstate`] recorded.
pub(crate) fn restore_viewstate(mut wp: Win, vs: viewstate_T) {
    wp.w_curswant = vs.vs_curswant;
    wp.w_leftcol = vs.vs_leftcol;
    wp.w_skipcol = vs.vs_skipcol;
    wp.w_topline = vs.vs_topline;
    wp.w_topfill = vs.vs_topfill;
    wp.w_botline = vs.vs_botline;
    wp.w_empty_rows = vs.vs_empty_rows;
}

/// Start an incremental search from where the cursor and view are now.
pub(crate) unsafe fn init_incsearch_state(mut s: Is) {
    s.winid = cur_win().handle;
    s.match_start = cur_win().w_cursor;
    s.did_incsearch = false;
    s.incsearch_postponed = false;
    s.magic_overruled_save = magic_overruled.get();
    clearpos(&mut s.match_end);
    s.save_cursor = cur_win().w_cursor;
    s.search_start = cur_win().w_cursor;
    s.init_viewstate = save_viewstate(cur_win());
    s.old_viewstate = save_viewstate(cur_win());
}

/// Move `t` to the end of the match the last search found, clamped to the
/// last line of the buffer.
pub(crate) fn set_search_match(t: &mut pos_T) {
    t.lnum += search_match_lines.get();
    t.col = search_match_endcol.get();
    if t.lnum > cur_buf().b_ml.ml_line_count {
        t.lnum = cur_buf().b_ml.ml_line_count;
        // SAFETY: `curwin` is a live window.
        unsafe { coladvance(curwin.get(), MAXCOL) };
    }
}

/// Parse a `:[range]s/foo`-style command line into the parts `'incsearch'`
/// and wildmenu completion need.
///
/// Answers true when the line holds a valid pattern, having set `skiplen`
/// (bytes before the pattern), `patlen`, `search_delim`, `search_first_line`
/// and `search_last_line`.
pub unsafe fn parse_pattern_and_range(
    incsearch_start: pos_T,
    search_delim: *mut ::core::ffi::c_int,
    skiplen: *mut ::core::ffi::c_int,
    patlen: *mut ::core::ffi::c_int,
) -> bool {
    let mut delim_optional = false;
    let mut dummy = None;
    let mut magic: magic_T = 0;

    // The three out-parameters, taken once. Both callers own three plain
    // `c_int` locals apiece that nothing else in the editor can reach, so
    // there is no re-entry to alias them.
    // SAFETY: the caller's obligation -- three live, distinct `c_int`s.
    let (search_delim, skiplen, patlen) =
        unsafe { (&mut *search_delim, &mut *skiplen, &mut *patlen) };

    // `cmd`, `p` and `end` all point into the command line, which is one
    // NUL-terminated allocation, and the walk below stops at its terminator;
    // reading a byte is stated here once instead of at twenty-odd sites.
    let at = |q: *const ::core::ffi::c_char| unsafe { *q } as ::core::ffi::c_int;
    // C's `skipwhite` over the same string.
    let skip_ws = |q: *mut ::core::ffi::c_char| unsafe { skipwhite(q) };

    *skiplen = 0;
    *patlen = Cc::current().len();

    // Default range: all lines.
    search_first_line.set(0);
    search_last_line.set(MAXLNUM as linenr_T);

    let mut ea = exarg_T {
        line1: 1,
        line2: 1,
        cmd: Cc::current().text(),
        addr_type: CmdAddr::Lines,
        ..EXARG_T_INIT
    };

    // Uninitialised in the C; `parse_command_modifiers` only writes it.
    let mut dummy_cmdmod = cmdmod_T::default();
    unsafe { parse_command_modifiers(&raw mut ea, &mut dummy, &mut dummy_cmdmod, true) };

    // Skip over the range to find the command.
    let cmd = unsafe { skip_range(ea.cmd, ::core::ptr::null_mut::<ExpandContext>()) };
    if unsafe { vim_strchr(c"sgvlu".as_ptr(), at(cmd) as uint8_t as ::core::ffi::c_int) }.is_null()
    {
        return false;
    }

    // Skip over the command name to find the pattern separator.
    let mut p = cmd;
    while ascii_isalpha(at(p)) {
        p = p.wrapping_offset(1);
    }
    if at(skip_ws(p)) == NUL {
        return false;
    }

    // C's `strncmp(cmd, name, MAX(p - cmd, min))`: is what has been typed
    // an abbreviation of `name`?  `min` is the shortest form upstream
    // accepts, and 0 where it imposes none (`p` never precedes `cmd`, so
    // `MAX(p - cmd, 0)` is just `p - cmd`).
    let namelen = p.addr().wrapping_sub(cmd.addr()) as isize;
    let abbreviates =
        |name: &CStr, min: isize| unsafe { strncmp(cmd, name.as_ptr(), namelen.max(min) as size_t) } == 0;

    if abbreviates(c"substitute", 0)
        || abbreviates(c"smagic", 0)
        || abbreviates(c"snomagic", 3)
        || abbreviates(c"vglobal", 0)
    {
        if at(cmd) == 's' as ::core::ffi::c_int
            && at(cmd.wrapping_offset(1)) == 'm' as ::core::ffi::c_int
        {
            magic_overruled.set(OPTION_MAGIC_ON);
        } else if at(cmd) == 's' as ::core::ffi::c_int
            && at(cmd.wrapping_offset(1)) == 'n' as ::core::ffi::c_int
        {
            magic_overruled.set(OPTION_MAGIC_OFF);
        }
    } else if abbreviates(c"sort", 3) || abbreviates(c"uniq", 3) {
        // Skip over "!" and the flags.
        if at(p) == '!' as ::core::ffi::c_int {
            p = skip_ws(p.wrapping_offset(1));
        }
        // C's `while (ASCII_ISALPHA(*(p = skipwhite(p)))) p++;` — the
        // macro evaluates its argument up to four times, which is a no-op
        // because `skipwhite` is idempotent.
        loop {
            p = skip_ws(p);
            if !ascii_isalpha(at(p)) {
                break;
            }
            p = p.wrapping_offset(1);
        }
        if at(p) == NUL {
            return false;
        }
    } else if abbreviates(c"vimgrep", 3)
        || abbreviates(c"vimgrepadd", 8)
        || abbreviates(c"lvimgrep", 2)
        || abbreviates(c"lvimgrepadd", 9)
        || abbreviates(c"global", 0)
    {
        // Skip an optional "!".
        if at(p) == '!' as ::core::ffi::c_int {
            p = p.wrapping_offset(1);
            if at(skip_ws(p)) == NUL {
                return false;
            }
        }
        if at(cmd) != 'g' as ::core::ffi::c_int {
            delim_optional = true;
        }
    } else {
        return false;
    }

    p = skip_ws(p);
    let delim =
        if delim_optional && unsafe { vim_is_ident_char(at(p) as uint8_t as ::core::ffi::c_int) } {
            ' ' as ::core::ffi::c_int
        } else {
            let c = at(p);
            p = p.wrapping_offset(1);
            c
        };
    *search_delim = delim;

    let end = skip_pattern(p, delim, &mut magic);
    let use_last_pat = end == p && at(end) == delim;

    if end == p && !use_last_pat {
        return false;
    }

    // Skip if the pattern matches everything (e.g. for 'hlsearch').
    if !use_last_pat {
        // SAFETY: `end` is inside the command line, so the terminator can
        // be moved there and put back around the one call that reads it.
        let empty = unsafe {
            let c = *end;
            *end = NUL as ::core::ffi::c_char;
            let empty = empty_pattern_magic(p, end.offset_from(p) as size_t, magic);
            *end = c;
            empty
        };
        if empty {
            return false;
        }
    }

    // Found a non-empty pattern, or "//".
    *skiplen = p.addr().wrapping_sub(Cc::current().text().addr()) as ::core::ffi::c_int;
    *patlen = end.addr().wrapping_sub(p.addr()) as ::core::ffi::c_int;

    // Parse the address range.
    let save_cursor = cur_win().w_cursor;
    cur_win().w_cursor = incsearch_start;

    unsafe { parse_cmd_address(&raw mut ea, &mut dummy, true) };

    if ea.addr_count > 0 {
        // Allow for a reverse match.
        search_first_line.set(ea.line2.min(ea.line1));
        search_last_line.set(ea.line2.max(ea.line1));
    } else if at(cmd) == 's' as ::core::ffi::c_int
        && at(cmd.wrapping_offset(1)) != 'o' as ::core::ffi::c_int
    {
        // :s defaults to the current line.
        search_last_line.set(cur_win().w_cursor.lnum);
        search_first_line.set(search_last_line.get());
    }

    cur_win().w_cursor = save_cursor;
    true
}

/// Whether `'incsearch'` highlighting should be done for this command line,
/// and if so what its pattern and address range are.
///
/// May change the last search pattern.
pub(crate) fn do_incsearch_highlighting(
    firstc: ::core::ffi::c_int,
    search_delim: &mut ::core::ffi::c_int,
    is_state: Is,
    skiplen: &mut ::core::ffi::c_int,
    patlen: &mut ::core::ffi::c_int,
) -> bool {
    *skiplen = 0;
    *patlen = Cc::current().len();

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
    // SAFETY: the three out-parameters are this frame's own borrows.
    unsafe { parse_pattern_and_range(is_state.search_start, search_delim, skiplen, patlen) }
}

/// Do the `'incsearch'` preview, if it is wanted here.
pub(crate) unsafe fn may_do_incsearch_highlighting(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    mut s: Is,
) {
    let mut cc = Cc::current();
    let mut skiplen = 0;
    let mut patlen = 0;
    let mut search_delim = 0;

    // Parsing the range may already set the last search pattern.
    // NOTE: restore_last_search_pattern() must run before every return.
    save_last_search_pattern();

    if !do_incsearch_highlighting(firstc, &mut search_delim, s, &mut skiplen, &mut patlen) {
        restore_last_search_pattern();
        unsafe { finish_incsearch_highlighting(false, s, true) };
        return;
    }

    // If there is a character waiting, search and redraw later.
    if unsafe { char_avail() } {
        restore_last_search_pattern();
        s.incsearch_postponed = true;
        return;
    }
    s.incsearch_postponed = false;

    // Use the previous pattern for ":s//".
    let mut next_char = cmd_byte(cc, skiplen + patlen);
    let use_last_pat = patlen == 0 && skiplen > 0 && cmd_byte(cc, skiplen - 1) == next_char;

    if patlen != 0 || use_last_pat {
        ui_busy_start();
        unsafe { ui_flush() };
    }

    if search_first_line.get() == 0 {
        // Start at the original cursor position.
        cur_win().w_cursor = s.search_start;
    } else if search_first_line.get() > cur_buf().b_ml.ml_line_count {
        // Start after the last line.
        cur_win().w_cursor.lnum = cur_buf().b_ml.ml_line_count;
        cur_win().w_cursor.col = MAXCOL;
    } else {
        // Start at the first line in the range.
        cur_win().w_cursor.lnum = search_first_line.get();
        cur_win().w_cursor.col = 0;
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
        set_cmd_byte(cc, skiplen + patlen, NUL as ::core::ffi::c_char);
        // So it doesn't beep on a bad expression.
        let no_emsg = Suppress::emsg();
        let op = ::core::ptr::null_mut::<oparg_T>();
        let dir = if firstc == ':' as ::core::ffi::c_int {
            '/' as ::core::ffi::c_int
        } else {
            firstc
        };
        let (pat, plen) = (cc.at(skiplen), patlen as size_t);
        let (flags, sia_p) = (search_flags, &raw mut sia);
        // SAFETY: `pat` is the pattern inside the command line, terminated
        // just above, and `sia_p` is this frame's search argument block.
        found = unsafe { do_search(op, dir, search_delim, pat, plen, count, flags, sia_p) };
        drop(no_emsg);
        set_cmd_byte(cc, skiplen + patlen, next_char);

        if cur_win().w_cursor.lnum < search_first_line.get()
            || cur_win().w_cursor.lnum > search_last_line.get()
        {
            // The match is outside the address range.
            found = 0;
            cur_win().w_cursor = s.search_start;
        }

        // Interrupted while searching: behave as if it failed.
        if got_int.get() {
            // Remove <C-C> from the input stream.
            unsafe { vpeekc() };
            // Don't abandon the command line.
            got_int.set(false);
            found = 0;
        } else if unsafe { char_avail() } {
            // Searching was cancelled because a character was typed.
            s.incsearch_postponed = true;
        }
        ui_busy_stop();
    } else {
        // Turn off the previous highlight.
        unsafe { set_no_hlsearch(true) };
        unsafe { redraw_all_later(UPD_SOME_VALID) };
    }

    // Add or remove the search match position.
    highlight_match.set(found != 0);

    // First restore the old curwin values, so the screen is positioned the
    // same way the real search command would leave it.
    restore_viewstate(cur_win(), s.old_viewstate);
    curwin_cursor_moved();

    let mut end_pos = cur_win().w_cursor;
    if found != 0 {
        s.match_start = cur_win().w_cursor;
        // SAFETY: `curwin`'s cursor lives as long as the window, and
        // `coladvance` inside writes through the same place.
        set_search_match(unsafe { &mut *cur_win().cursor().raw() });
        validate_curwin_cursor();
        s.match_end = cur_win().w_cursor;
        cur_win().w_cursor = end_pos;
        end_pos = s.match_end;
    }

    // Disable 'hlsearch' highlighting if the pattern matches everything.
    // Avoids a flash when typing "foo\|".
    if !use_last_pat {
        next_char = cmd_byte(cc, skiplen + patlen);
        set_cmd_byte(cc, skiplen + patlen, NUL as ::core::ffi::c_char);
        if unsafe { empty_pattern(cc.at(skiplen), patlen as size_t, search_delim) }
            && !no_hlsearch.get()
        {
            unsafe { redraw_all_later(UPD_SOME_VALID) };
            unsafe { set_no_hlsearch(true) };
        }
        set_cmd_byte(cc, skiplen + patlen, next_char);
    }

    validate_curwin_cursor();

    // May redraw the status line to show the cursor position.
    if p_ru.get() != 0 && (cur_win().w_status_height > 0 || global_stl_height() > 0) {
        cur_win().w_redr_status = true;
    }

    unsafe { redraw_later(curwin.get(), UPD_SOME_VALID) };
    unsafe { update_screen() };
    highlight_match.set(false);
    restore_last_search_pattern();

    // Leave the cursor at the end so CTRL-R CTRL-W works — but not when
    // it is beyond the end of the pattern, as for ":s/pat/".
    if cmd_byte(cc, skiplen + patlen) as ::core::ffi::c_int != NUL {
        cur_win().w_cursor = s.search_start;
    } else if found != 0 {
        cur_win().w_cursor = end_pos;
        // Mark as valid for the cmdline_show redraw.
        cur_win().w_valid_cursor = end_pos;
    }

    unsafe { msg_starthere() };
    unsafe { redrawcmdline() };
    s.did_incsearch = true;
}

/// CTRL-L: add the character under the match to the pattern, and say so in
/// `*c`.
///
/// Answers `OK` when the caller should treat the key as unchanged.
pub(crate) unsafe fn may_add_char_to_search(
    firstc: ::core::ffi::c_int,
    c: &mut ::core::ffi::c_int,
    mut s: Is,
) -> ::core::ffi::c_int {
    let mut skiplen = 0;
    let mut patlen = 0;
    let mut search_delim = 0;

    // Parsing the range may already set the last search pattern.
    // NOTE: restore_last_search_pattern() must run before every return.
    save_last_search_pattern();

    if !do_incsearch_highlighting(firstc, &mut search_delim, s, &mut skiplen, &mut patlen) {
        restore_last_search_pattern();
        return FAIL;
    }
    restore_last_search_pattern();

    if s.did_incsearch {
        cur_win().w_cursor = s.match_end;
        // SAFETY: the cursor was just put on a match of the current buffer.
        *c = gchar_cursor();
        if *c != NUL {
            // With 'ignorecase' and 'smartcase' set and no uppercase in
            // the command line, lowercase the character.
            if p_ic.get() != 0
                && p_scs.get() != 0
                // SAFETY: the pattern starts `skiplen` bytes into the
                // command line, which is NUL-terminated.
                && !unsafe { pat_has_uppercase(Cc::current().at(skiplen)) }
            {
                *c = mb_tolower(*c);
            }
            let magics = if magic_isset() {
                c"\\~^$.*[".as_ptr()
            } else {
                c"\\^$".as_ptr()
            };
            // SAFETY: a static NUL-terminated string and a character.
            let special = unsafe { vim_strchr(magics, *c) };
            if *c == search_delim || !special.is_null() {
                // Put a backslash before the special characters.
                stuff_readbuf_char(*c);
                *c = '\\' as ::core::ffi::c_int;
            }
            // Add any composing characters.
            // SAFETY: the cursor is on a character of the current line.
            let cursor_len = || unsafe { utfc_ptr2len(get_cursor_pos_ptr()) };
            if utf_char2len(*c) != cursor_len() {
                let save_c = *c;
                while utf_char2len(*c) != cursor_len() {
                    cur_win().w_cursor.col += utf_char2len(*c);
                    // SAFETY: as the `gchar_cursor` above.
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

/// Undo the preview: put the cursor and the view back where the command line
/// found them, and clear the match highlight.
pub(crate) unsafe fn finish_incsearch_highlighting(
    gotesc: bool,
    mut s: Is,
    call_update_screen: bool,
) {
    if !s.did_incsearch {
        return;
    }

    s.did_incsearch = false;
    if gotesc {
        cur_win().w_cursor = s.save_cursor;
    } else {
        if !equalpos(s.save_cursor, s.search_start) {
            // Put the previous-context mark at the original position.
            cur_win().w_cursor = s.save_cursor;
            setpcmark();
        }
        cur_win().w_cursor = s.search_start;
    }
    restore_viewstate(cur_win(), s.old_viewstate);
    highlight_match.set(false);

    // By default search all lines.
    search_first_line.set(0);
    search_last_line.set(MAXLNUM as linenr_T);

    magic_overruled.set(s.magic_overruled_save);

    // Needed for TAB.
    validate_curwin_cursor();
    unsafe { status_redraw_all() };
    unsafe { redraw_all_later(UPD_SOME_VALID) };
    if call_update_screen {
        unsafe { update_screen() };
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
    mut s: Is,
    next_match: bool,
) -> ::core::ffi::c_int {
    let mut cc = Cc::current();
    let mut skiplen = 0;
    let mut patlen = 0;
    let mut search_delim = 0;

    // Parsing the range may already set the last search pattern.
    // NOTE: restore_last_search_pattern() must run before every return.
    save_last_search_pattern();

    if !do_incsearch_highlighting(firstc, &mut search_delim, s, &mut skiplen, &mut patlen) {
        restore_last_search_pattern();
        return OK;
    }
    if patlen == 0 && cmd_byte(cc, skiplen) as ::core::ffi::c_int == NUL {
        restore_last_search_pattern();
        return FAIL;
    }

    ui_busy_start();
    unsafe { ui_flush() };

    let mut search_flags = SEARCH_NOOF;

    let pat: *mut ::core::ffi::c_char;
    if search_delim == cmd_byte(cc, skiplen) as ::core::ffi::c_int {
        pat = last_search_pattern();
        if pat.is_null() {
            restore_last_search_pattern();
            return FAIL;
        }
        skiplen = 0;
        patlen = last_search_pattern_len() as ::core::ffi::c_int;
    } else {
        pat = cc.at(skiplen);
    }

    // Do not search for the search end delimiter, unless it is part of
    // the pattern.
    let mut bslsh = false;
    if patlen > 2 && firstc == unsafe { *pat.offset((patlen - 1) as isize) } as ::core::ffi::c_int {
        patlen -= 1;
        if unsafe { *pat.offset((patlen - 1) as isize) } as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
        {
            unsafe {
                *pat.offset((patlen - 1) as isize) = firstc as uint8_t as ::core::ffi::c_char
            };
            bslsh = true;
        }
    }

    let mut t: pos_T;
    if next_match {
        t = s.match_end;
        if lt(s.match_start, s.match_end) {
            // Start searching at the end of the match, not at the
            // beginning of the next column.
            unsafe { decl(&mut t) };
        }
        search_flags += SEARCH_COL;
    } else {
        t = s.match_start;
    }
    if p_hls.get() == 0 {
        search_flags += SEARCH_KEEP;
    }

    let no_emsg = Suppress::emsg();
    let save = unsafe { *pat.offset(patlen as isize) };
    unsafe { *pat.offset(patlen as isize) = NUL as ::core::ffi::c_char };
    let (w, b) = (curwin.get(), curbuf.get());
    let (tp, e) = (&raw mut t, ::core::ptr::null_mut::<pos_T>());
    let dir = if next_match { FORWARD } else { BACKWARD } as Direction;
    let (plen, flags) = (patlen as size_t, search_flags);
    let re = RE_SEARCH as ::core::ffi::c_int;
    let sia = ::core::ptr::null_mut::<searchit_arg_T>();
    // SAFETY: `tp` is this frame's start position and `pat` the pattern
    // inside the command line, terminated just above.
    let found = unsafe { searchit(w, b, tp, e, dir, pat, plen, count, flags, re, sia) };
    drop(no_emsg);
    unsafe { *pat.offset(patlen as isize) = save };
    if bslsh {
        unsafe { *pat.offset((patlen - 1) as isize) = '\\' as ::core::ffi::c_char };
    }
    ui_busy_stop();

    if found != 0 {
        s.search_start = s.match_start;
        s.match_end = t;
        s.match_start = t;
        if !next_match && firstc != '?' as ::core::ffi::c_int {
            // Move just before the current match, so that when nv_search
            // finishes the cursor is put back on the match.
            s.search_start = t;
            unsafe { decl(&mut s.search_start) };
        } else if next_match && firstc == '?' as ::core::ffi::c_int {
            // Move just after the current match, for the same reason.
            s.search_start = t;
            unsafe { incl(&mut s.search_start) };
        }
        if lt(t, s.search_start) && next_match {
            // Wrapped around.
            s.search_start = t;
            if firstc == '?' as ::core::ffi::c_int {
                unsafe { incl(&mut s.search_start) };
            } else {
                unsafe { decl(&mut s.search_start) };
            }
        }

        set_search_match(&mut s.match_end);
        cur_win().w_cursor = s.match_start;
        curwin_cursor_moved();
        validate_curwin_cursor();
        highlight_match.set(true);
        s.old_viewstate = save_viewstate(cur_win());
        unsafe { redraw_later(curwin.get(), UPD_NOT_VALID) };
        unsafe { update_screen() };
        highlight_match.set(false);
        unsafe { redrawcmdline() };
        cur_win().w_cursor = s.match_end;
    } else {
        unsafe { vim_beep(kOptBoFlagError as ::core::ffi::c_uint) };
    }

    restore_last_search_pattern();
    FAIL
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
    let mut magic_val: magic_T = MAGIC_ON;

    if len > 0 {
        skip_pattern(p, delim, &mut magic_val);
    } else {
        return true;
    }

    unsafe { empty_pattern_magic(p, len, magic_val) }
}

/// [`empty_pattern`] with the `'magic'` level already known.
pub(crate) unsafe fn empty_pattern_magic(
    p: *mut ::core::ffi::c_char,
    mut len: size_t,
    magic_val: magic_T,
) -> bool {
    // `p[..len]` is the pattern, inside the command line's own allocation;
    // every index below is `len - 1` or `len - 2` with `len >= 2`.
    // SAFETY: the caller's obligation -- `len` bytes readable at `p`.
    let at = |i: size_t| unsafe { *p.add(i) } as ::core::ffi::c_int;

    // Remove a trailing \v and the like.
    while len >= 2 && at(len - 2) == '\\' as ::core::ffi::c_int && {
        // SAFETY: a static NUL-terminated string and a byte.
        let byte = at(len - 1) as uint8_t as ::core::ffi::c_int;
        let flag = unsafe { vim_strchr(c"mMvVcCZ".as_ptr(), byte) };
        !flag.is_null()
    } {
        len -= 2;
    }

    // True if the pattern is empty, or ends with \| and 'magic' is set (or
    // ends with '|' and very magic is set).
    len == 0
        || len > 1
            && at(len - 1) == '|' as ::core::ffi::c_int
            && (at(len - 2) == '\\' as ::core::ffi::c_int && magic_val == MAGIC_ON
                || at(len - 2) != '\\' as ::core::ffi::c_int && magic_val == MAGIC_ALL)
}

/// C's `skip_regexp_ex(p, delim, magic_isset(), NULL, NULL, &magic)`: find
/// the end of the pattern `p` and say what magicness it ended under.
fn skip_pattern(
    p: *mut ::core::ffi::c_char,
    delim: ::core::ffi::c_int,
    magic: &mut magic_T,
) -> *mut ::core::ffi::c_char {
    let no_str = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let no_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let is_magic = magic_isset() as ::core::ffi::c_int;
    // SAFETY: `p` points into a live NUL-terminated pattern.
    unsafe { skip_regexp_ex(p, delim, is_magic, no_str, no_int, magic) }
}

/// The command line's byte at `i`.
///
/// [`Cc::at`] computes an address without reading; every caller here has an
/// `i` inside the line the pattern parse has just measured.
fn cmd_byte(cc: Cc, i: ::core::ffi::c_int) -> ::core::ffi::c_char {
    // SAFETY: `i` is at or before the command line's terminator.
    unsafe { *cc.at(i) }
}

/// Write the command line's byte at `i`; see [`cmd_byte`].
fn set_cmd_byte(cc: Cc, i: ::core::ffi::c_int, b: ::core::ffi::c_char) {
    // SAFETY: as [`cmd_byte`].
    unsafe { *cc.at(i) = b };
}

/// C's `changed_cline_bef_curs(curwin); update_topline(curwin);`, the pair
/// that follows every cursor move the preview makes.
fn curwin_cursor_moved() {
    // SAFETY: `curwin` is a live window.
    unsafe {
        changed_cline_bef_curs(curwin.get());
        update_topline(curwin.get());
    }
}

/// C's `validate_cursor(curwin)`.
fn validate_curwin_cursor() {
    // SAFETY: `curwin` is a live window.
    unsafe { validate_cursor(curwin.get()) };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
