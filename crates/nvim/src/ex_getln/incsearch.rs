//! `'incsearch'`: previewing a `/`, `?` or `:s` pattern while it is typed.
//!
//! [`may_do_incsearch_highlighting`] runs on every command-line change and,
//! for the commands that take a pattern, searches from the saved view state
//! and highlights what it found.  [`parse_pattern_and_range`] is what decides
//! whether the line being typed *is* such a command, and the `ViewState`
//! pair saves and restores the window the preview scrolled.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::skip;
use crate::cstr;
use crate::strings::has_char;
use crate::winlayer::{Buf, Win};

use crate::guard::Suppress;
use crate::mbyte::cluster_len;
use crate::types::{Failed, NUL};
use core::ffi::CStr;

/// Fire a `Cmdline*` / `Cmdwin*` autocommand whose `<afile>` and `<amatch>`
/// are the command-line type character.
pub(crate) fn trigger_cmd_autocmd(typechar: ::core::ffi::c_int, evt: AutoEvent) {
    let typestr = [typechar as u8, 0];
    cmdline_autocmd(evt, cstr::in_bytes(&typestr));
}

/// C's `apply_autocmds(evt, fname, fname, false, curbuf)`: the one shape
/// every `Cmdline*`/`Cmdwin*` event is fired with, where `<afile>` and
/// `<amatch>` are the same string.
pub(crate) fn cmdline_autocmd(evt: AutoEvent, fname: &CStr) -> bool {
    apply_autocmds(evt, Some(fname), Some(fname), false, Buf::current_or_none())
}

/// Record everything about `window`'s view that an incremental search may scroll.
///
/// C wrote through a `ViewState *`; the structure is seven scalars and
/// `Copy`, so answering by value says the same thing in safe code.
pub(crate) fn save_viewstate(window: Win) -> ViewState {
    ViewState {
        vs_curswant: window.w_curswant,
        vs_leftcol: window.w_leftcol,
        vs_skipcol: window.w_skipcol,
        vs_topline: window.w_topline,
        vs_topfill: window.w_topfill,
        vs_botline: window.w_botline,
        vs_empty_rows: window.w_empty_rows,
    }
}

/// Put back what [`save_viewstate`] recorded.
pub(crate) fn restore_viewstate(mut window: Win, vs: ViewState) {
    window.w_curswant = vs.vs_curswant;
    window.w_leftcol = vs.vs_leftcol;
    window.w_skipcol = vs.vs_skipcol;
    window.w_topline = vs.vs_topline;
    window.w_topfill = vs.vs_topfill;
    window.w_botline = vs.vs_botline;
    window.w_empty_rows = vs.vs_empty_rows;
}

/// Start an incremental search from where the cursor and view are now.
pub(crate) fn init_incsearch_state(s: &mut IncsearchState) {
    s.winid = Win::current().handle;
    s.match_start = Win::current().w_cursor;
    s.did_incsearch = false;
    s.incsearch_postponed = false;
    s.magic_overruled_save = magic_overruled.get();
    clearpos(&mut s.match_end);
    s.save_cursor = Win::current().w_cursor;
    s.search_start = Win::current().w_cursor;
    s.init_viewstate = save_viewstate(Win::current());
    s.old_viewstate = save_viewstate(Win::current());
}

/// Move `t` to the end of the match the last search found, clamped to the
/// last line of the buffer.
pub(crate) fn set_search_match(t: &mut Pos) {
    t.lnum += search_match_lines.get();
    t.col = search_match_endcol.get();
    if t.lnum > Buf::current().b_ml.ml_line_count {
        t.lnum = Buf::current().b_ml.ml_line_count;
        coladvance(Win::current(), MAXCOL);
    }
}

/// Where `'incsearch'` finds the pattern in the command line: the
/// delimiter, how many bytes come before the pattern, and its length.
pub(crate) struct PatternSpan {
    pub(crate) delim: ::core::ffi::c_int,
    pub(crate) skiplen: ::core::ffi::c_int,
    pub(crate) patlen: ::core::ffi::c_int,
}

/// Parse a `:[range]s/foo`-style command line into the parts `'incsearch'`
/// and wildmenu completion need.
///
/// Answers where the pattern is when the line holds a valid one, having set
/// `search_first_line` and `search_last_line`.
pub(crate) fn parse_pattern_and_range(incsearch_start: Pos) -> Option<PatternSpan> {
    let mut delim_optional = false;
    let mut dummy = None;
    let mut magic: Magic = 0;

    // A copy of what is typed so far: the scan below only reads it, and an
    // offset into the copy is the same offset into the line.
    let line = CmdLine::from_bytes(cstr::until_nul(Cc::current().bytes()));

    // Default range: all lines.
    search_first_line.set(0);
    search_last_line.set(MAXLNUM);

    let mut ea = ExArg {
        line1: 1,
        line2: 1,
        line,
        addr_type: CmdAddr::Lines,
        ..EXARG_T_INIT
    };

    // Uninitialised in the C; `parse_command_modifiers` only writes it.
    let mut dummy_cmdmod = CmdMod::default();
    let _ = parse_command_modifiers(&mut ea, &mut dummy, &mut dummy_cmdmod, true);

    // Skip over the range to find the command.
    let cmd = ea.line.cmd + skip_range(ea.line.tail(ea.line.cmd), None);
    let span = {
        // The walk is over the line's own bytes, terminator included, and
        // ends before anything else reaches the line; what outlives it is
        // offsets.
        let buf = ea.line.buffer();
        let at = |q: usize| ::core::ffi::c_int::from(cstr::byte_at(buf, q));
        // C's `skipwhite` over the same string.
        let skip_ws = |q: usize| q + skip::white(buf.get(q..).unwrap_or_default());
        if !has_char(c"sgvlu", at(cmd)) {
            return None;
        }

        // Skip over the command name to find the pattern separator.
        let mut p = cmd;
        while ascii_isalpha(at(p)) {
            p += 1;
        }
        if at(skip_ws(p)) == NUL {
            return None;
        }

        // C's `strncmp(cmd, name, MAX(p - cmd, min))`: is what has been
        // typed an abbreviation of `name`?  `min` is the shortest form
        // upstream accepts, and 0 where it imposes none.
        let namelen = p - cmd;
        let typed = &buf[cmd..];
        let abbreviates =
            |name: &CStr, min: usize| cstr::ncmp_eq(typed, name.to_bytes(), namelen.max(min));

        if abbreviates(c"substitute", 0)
            || abbreviates(c"smagic", 0)
            || abbreviates(c"snomagic", 3)
            || abbreviates(c"vglobal", 0)
        {
            if at(cmd) == 's' as ::core::ffi::c_int && at(cmd + 1) == 'm' as ::core::ffi::c_int {
                magic_overruled.set(OPTION_MAGIC_ON);
            } else if at(cmd) == 's' as ::core::ffi::c_int
                && at(cmd + 1) == 'n' as ::core::ffi::c_int
            {
                magic_overruled.set(OPTION_MAGIC_OFF);
            }
        } else if abbreviates(c"sort", 3) || abbreviates(c"uniq", 3) {
            // Skip over "!" and the flags.
            if at(p) == '!' as ::core::ffi::c_int {
                p = skip_ws(p + 1);
            }
            loop {
                p = skip_ws(p);
                if !ascii_isalpha(at(p)) {
                    break;
                }
                p += 1;
            }
            if at(p) == NUL {
                return None;
            }
        } else if abbreviates(c"vimgrep", 3)
            || abbreviates(c"vimgrepadd", 8)
            || abbreviates(c"lvimgrep", 2)
            || abbreviates(c"lvimgrepadd", 9)
            || abbreviates(c"global", 0)
        {
            // Skip an optional "!".
            if at(p) == '!' as ::core::ffi::c_int {
                p += 1;
                if at(skip_ws(p)) == NUL {
                    return None;
                }
            }
            if at(cmd) != 'g' as ::core::ffi::c_int {
                delim_optional = true;
            }
        } else {
            return None;
        }

        p = skip_ws(p);
        let delim = if delim_optional && vim_is_ident_char(at(p) as uint8_t as ::core::ffi::c_int) {
            ' ' as ::core::ffi::c_int
        } else {
            let c = at(p);
            p += 1;
            c
        };

        let end = p + skip_pattern(&buf[p.min(buf.len())..], delim, &mut magic);
        let use_last_pat = end == p && at(end) == delim;

        if end == p && !use_last_pat {
            return None;
        }

        // Skip if the pattern matches everything (e.g. for 'hlsearch').
        if !use_last_pat && empty_pattern_magic(&buf[p..end], magic) {
            return None;
        }

        // Found a non-empty pattern, or "//".
        PatternSpan {
            delim,
            skiplen: p as ::core::ffi::c_int,
            patlen: (end - p) as ::core::ffi::c_int,
        }
    };

    // Parse the address range.
    let save_cursor = Win::current().w_cursor;
    Win::current().w_cursor = incsearch_start;

    parse_cmd_address(&mut ea, &mut dummy, true);

    if ea.addr_count > 0 {
        // Allow for a reverse match.
        search_first_line.set(ea.line2.min(ea.line1));
        search_last_line.set(ea.line2.max(ea.line1));
    } else if ea.line.byte_at(cmd) == b's' && ea.line.byte_at(cmd + 1) != b'o' {
        // :s defaults to the current line.
        search_last_line.set(Win::current().w_cursor.lnum);
        search_first_line.set(search_last_line.get());
    }

    Win::current().w_cursor = save_cursor;
    Some(span)
}

/// [`parse_pattern_and_range`] for a caller that wants only where the
/// pattern starts and how long it is: `None` when the command line has no
/// pattern in it.
pub(crate) fn parse_pattern_and_range_of(
    incsearch_start: Pos,
) -> Option<(::core::ffi::c_int, ::core::ffi::c_int)> {
    parse_pattern_and_range(incsearch_start).map(|span| (span.skiplen, span.patlen))
}

/// Whether `'incsearch'` highlighting should be done for this command line,
/// and if so where its pattern is (having set its address range).
///
/// May change the last search pattern.
pub(crate) fn do_incsearch_highlighting(
    firstc: ::core::ffi::c_int,
    is_state: &IncsearchState,
) -> Option<PatternSpan> {
    if !p_is() || cmd_silent.get() {
        return None;
    }

    // By default search all lines.
    search_first_line.set(0);
    search_last_line.set(MAXLNUM);

    if firstc == '/' as ::core::ffi::c_int || firstc == '?' as ::core::ffi::c_int {
        return Some(PatternSpan {
            delim: firstc,
            skiplen: 0,
            patlen: Cc::current().len(),
        });
    }

    if firstc != ':' as ::core::ffi::c_int {
        return None;
    }

    let _no_emsg = Suppress::emsg();
    parse_pattern_and_range(is_state.search_start)
}

/// Do the `'incsearch'` preview, if it is wanted here.
pub(crate) fn may_do_incsearch_highlighting(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    s: &mut IncsearchState,
) {
    let cc = Cc::current();

    // Parsing the range may already set the last search pattern.
    // NOTE: restore_last_search_pattern() must run before every return.
    save_last_search_pattern();

    let Some(PatternSpan {
        delim: search_delim,
        skiplen,
        patlen,
    }) = do_incsearch_highlighting(firstc, s)
    else {
        restore_last_search_pattern();
        finish_incsearch_highlighting(false, s, true);
        return;
    };

    // If there is a character waiting, search and redraw later.
    if char_avail() {
        restore_last_search_pattern();
        s.incsearch_postponed = true;
        return;
    }
    s.incsearch_postponed = false;

    // Use the previous pattern for ":s//".
    let next_char = cc.byte_at(skiplen + patlen);
    let use_last_pat = patlen == 0 && skiplen > 0 && cc.byte_at(skiplen - 1) == next_char;

    if patlen != 0 || use_last_pat {
        ui_busy_start();
        ui_flush();
    }

    if search_first_line.get() == 0 {
        // Start at the original cursor position.
        Win::current().w_cursor = s.search_start;
    } else if search_first_line.get() > Buf::current().b_ml.ml_line_count {
        // Start after the last line.
        Win::current().w_cursor.lnum = Buf::current().b_ml.ml_line_count;
        Win::current().w_cursor.col = MAXCOL;
    } else {
        // Start at the first line in the range.
        Win::current().w_cursor.lnum = search_first_line.get();
        Win::current().w_cursor.col = 0;
    }

    // The do_search() result.
    let mut found = 0;

    if patlen != 0 || use_last_pat {
        let mut search_flags = SEARCH_OPT + SEARCH_NOOF + SEARCH_PEEK;
        if !p_hls() {
            search_flags += SEARCH_KEEP;
        }
        if search_first_line.get() != 0 {
            search_flags += SEARCH_START;
        }
        // Half a second of search time.
        let tm: ProfTime = profile_setlimit(500);
        let mut sia = SearchItArg {
            sa_tm: Some(&tm),
            ..SearchItArg::default()
        };
        // So it doesn't beep on a bad expression.
        let no_emsg = Suppress::emsg();
        let dir = if firstc == ':' as ::core::ffi::c_int {
            '/' as ::core::ffi::c_int
        } else {
            firstc
        };
        // A copy of the pattern: the `patlen` bytes of the command line
        // from `skiplen`.
        let pat = pattern_of(cc, skiplen, patlen);
        let flags = search_flags;
        found = do_search(None, dir, search_delim, &pat, count, flags, Some(&mut sia));
        drop(no_emsg);

        if Win::current().w_cursor.lnum < search_first_line.get()
            || Win::current().w_cursor.lnum > search_last_line.get()
        {
            // The match is outside the address range.
            found = 0;
            Win::current().w_cursor = s.search_start;
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
            s.incsearch_postponed = true;
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
    restore_viewstate(Win::current(), s.old_viewstate);
    curwin_cursor_moved();

    let mut end_pos = Win::current().w_cursor;
    if found != 0 {
        s.match_start = Win::current().w_cursor;
        set_cursor_to_search_match();
        validate_curwin_cursor();
        s.match_end = Win::current().w_cursor;
        Win::current().w_cursor = end_pos;
        end_pos = s.match_end;
    }

    // Disable 'hlsearch' highlighting if the pattern matches everything.
    // Avoids a flash when typing "foo\|".
    if !use_last_pat
        && empty_pattern(&pattern_of(cc, skiplen, patlen), search_delim)
        && !no_hlsearch.get()
    {
        redraw_all_later(UPD_SOME_VALID);
        set_no_hlsearch(true);
    }

    validate_curwin_cursor();

    // May redraw the status line to show the cursor position.
    if p_ru() && (Win::current().w_status_height > 0 || global_stl_height() > 0) {
        Win::current().w_redr_status = true;
    }

    redraw_later(Win::current(), UPD_SOME_VALID);
    let _ = update_screen();
    highlight_match.set(false);
    restore_last_search_pattern();

    // Leave the cursor at the end so CTRL-R CTRL-W works — but not when
    // it is beyond the end of the pattern, as for ":s/pat/".
    if cc.byte_at(skiplen + patlen) != 0 {
        Win::current().w_cursor = s.search_start;
    } else if found != 0 {
        Win::current().w_cursor = end_pos;
        // Mark as valid for the cmdline_show redraw.
        Win::current().w_valid_cursor = end_pos;
    }

    msg_starthere();
    redrawcmdline();
    s.did_incsearch = true;
}

/// CTRL-L: add the character under the match to the pattern, and say so in
/// `*c`.
///
/// Answers `Ok` when the caller should treat the key as unchanged.
pub(crate) fn may_add_char_to_search(
    firstc: ::core::ffi::c_int,
    c: &mut ::core::ffi::c_int,
    s: &mut IncsearchState,
) -> Result<(), Failed> {
    // Parsing the range may already set the last search pattern.
    // NOTE: restore_last_search_pattern() must run before every return.
    save_last_search_pattern();

    let Some(PatternSpan {
        delim: search_delim,
        skiplen,
        ..
    }) = do_incsearch_highlighting(firstc, s)
    else {
        restore_last_search_pattern();
        return Err(Failed);
    };
    restore_last_search_pattern();

    if s.did_incsearch {
        Win::current().w_cursor = s.match_end;
        // SAFETY: the cursor was just put on a match of the current buffer.
        *c = gchar_cursor();
        if *c != NUL {
            // With 'ignorecase' and 'smartcase' set and no uppercase in
            // the command line, lowercase the character.
            if p_ic()
                && p_scs()
                // The pattern starts `skiplen` bytes into the command line.
                && !pat_has_uppercase(&pattern_of(Cc::current(), skiplen, ::core::ffi::c_int::MAX))
            {
                *c = mb_tolower(*c);
            }
            let magics = if magic_isset() { c"\\~^$.*[" } else { c"\\^$" };
            if *c == search_delim || has_char(magics, *c) {
                // Put a backslash before the special characters.
                stuff_readbuf_char(*c);
                *c = '\\' as ::core::ffi::c_int;
            }
            // Add any composing characters.
            let cursor_len = || with_cursor_rest(cluster_len) as ::core::ffi::c_int;
            if utf_char2len(*c) != cursor_len() {
                let save_c = *c;
                while utf_char2len(*c) != cursor_len() {
                    Win::current().w_cursor.col += utf_char2len(*c);
                    // SAFETY: as the `gchar_cursor` above.
                    *c = gchar_cursor();
                    stuff_readbuf_char(*c);
                }
                *c = save_c;
            }
            return Err(Failed);
        }
    }
    Ok(())
}

/// Undo the preview: put the cursor and the view back where the command line
/// found them, and clear the match highlight.
pub(crate) fn finish_incsearch_highlighting(
    gotesc: bool,
    s: &mut IncsearchState,
    call_update_screen: bool,
) {
    if !s.did_incsearch {
        return;
    }

    s.did_incsearch = false;
    if gotesc {
        Win::current().w_cursor = s.save_cursor;
    } else {
        if !equalpos(s.save_cursor, s.search_start) {
            // Put the previous-context mark at the original position.
            Win::current().w_cursor = s.save_cursor;
            setpcmark();
        }
        Win::current().w_cursor = s.search_start;
    }
    restore_viewstate(Win::current(), s.old_viewstate);
    highlight_match.set(false);

    // By default search all lines.
    search_first_line.set(0);
    search_last_line.set(MAXLNUM);

    magic_overruled.set(s.magic_overruled_save);

    // Needed for TAB.
    validate_curwin_cursor();
    status_redraw_all();
    redraw_all_later(UPD_SOME_VALID);
    if call_update_screen {
        let _ = update_screen();
    }
}

/// CTRL-G / CTRL-T: move the `'incsearch'` preview to the next or previous
/// match.
///
/// Answers `Ok` when there was no incremental search to move, `Err`
/// otherwise (which is what tells the key loop the line did not change).
pub(crate) fn may_do_command_line_next_incsearch(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
    s: &mut IncsearchState,
    next_match: bool,
) -> Result<(), Failed> {
    let cc = Cc::current();

    // Parsing the range may already set the last search pattern.
    // NOTE: restore_last_search_pattern() must run before every return.
    save_last_search_pattern();

    let Some(PatternSpan {
        delim: search_delim,
        skiplen,
        patlen,
    }) = do_incsearch_highlighting(firstc, s)
    else {
        restore_last_search_pattern();
        return Ok(());
    };
    if patlen == 0 && cc.byte_at(skiplen) == 0 {
        restore_last_search_pattern();
        return Err(Failed);
    }

    ui_busy_start();
    ui_flush();

    let mut search_flags = SEARCH_NOOF;

    // A copy of the pattern, which is changed below for the search.
    let mut pat: Vec<u8> = if search_delim == ::core::ffi::c_int::from(cc.byte_at(skiplen)) {
        let Some(last) = last_search_pattern() else {
            restore_last_search_pattern();
            return Err(Failed);
        };
        last.to_vec()
    } else {
        pattern_of(cc, skiplen, patlen)
    };

    // Do not search for the search end delimiter, unless it is part of
    // the pattern; an escaped one is searched for unescaped.
    if pat.len() > 2 && firstc == ::core::ffi::c_int::from(pat[pat.len() - 1]) {
        pat.pop();
        if let Some(last) = pat.last_mut()
            && *last == b'\\'
        {
            *last = firstc as u8;
        }
    }

    let mut t: Pos;
    if next_match {
        t = s.match_end;
        if lt(s.match_start, s.match_end) {
            // Start searching at the end of the match, not at the
            // beginning of the next column.
            decl(&mut t);
        }
        search_flags += SEARCH_COL;
    } else {
        t = s.match_start;
    }
    if !p_hls() {
        search_flags += SEARCH_KEEP;
    }

    let no_emsg = Suppress::emsg();
    let (w, b) = (Some(Win::current()), Buf::current());
    let dir = if next_match { FORWARD } else { BACKWARD } as Direction;
    let re = RE_SEARCH as ::core::ffi::c_int;
    let found = searchit(w, b, &mut t, None, dir, &pat, count, search_flags, re, None);
    drop(no_emsg);
    ui_busy_stop();

    if found != 0 {
        s.search_start = s.match_start;
        s.match_end = t;
        s.match_start = t;
        if !next_match && firstc != '?' as ::core::ffi::c_int {
            // Move just before the current match, so that when nv_search
            // finishes the cursor is put back on the match.
            s.search_start = t;
            decl(&mut s.search_start);
        } else if next_match && firstc == '?' as ::core::ffi::c_int {
            // Move just after the current match, for the same reason.
            s.search_start = t;
            incl(&mut s.search_start);
        }
        if lt(t, s.search_start) && next_match {
            // Wrapped around.
            s.search_start = t;
            if firstc == '?' as ::core::ffi::c_int {
                incl(&mut s.search_start);
            } else {
                decl(&mut s.search_start);
            }
        }

        set_search_match(&mut s.match_end);
        Win::current().w_cursor = s.match_start;
        curwin_cursor_moved();
        validate_curwin_cursor();
        highlight_match.set(true);
        s.old_viewstate = save_viewstate(Win::current());
        redraw_later(Win::current(), UPD_NOT_VALID);
        let _ = update_screen();
        highlight_match.set(false);
        redrawcmdline();
        Win::current().w_cursor = s.match_end;
    } else {
        vim_beep(kOptBoFlagError as ::core::ffi::c_uint);
    }

    restore_last_search_pattern();
    Err(Failed)
}

/// Guess whether the pattern `pat` matches everything.
///
/// Only finds specific cases, such as a trailing `\\|`, which can happen while
/// a pattern is being typed.
pub(crate) fn empty_pattern(pat: &[u8], delim: ::core::ffi::c_int) -> bool {
    if pat.is_empty() {
        return true;
    }
    // The 'magic' the pattern ends under, read up to its end as C read it
    // up to the NUL it wrote there.
    let mut text = Vec::with_capacity(pat.len() + 1);
    text.extend_from_slice(pat);
    text.push(0);
    let mut magic_val: Magic = MAGIC_ON;
    skip_pattern(&text, delim, &mut magic_val);
    empty_pattern_magic(pat, magic_val)
}

/// [`empty_pattern`] with the `'magic'` level already known.
pub(crate) fn empty_pattern_magic(pat: &[u8], magic_val: Magic) -> bool {
    let mut len = pat.len();
    let at = |i: usize| ::core::ffi::c_int::from(pat[i]);

    // Remove a trailing \v and the like.
    while len >= 2
        && at(len - 2) == '\\' as ::core::ffi::c_int
        && has_char(c"mMvVcCZ", ::core::ffi::c_int::from(pat[len - 1]))
    {
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

/// C's `skip_regexp_ex(p, delim, magic_isset(), NULL, NULL, &magic)`: how
/// long the pattern `text` starts with is, and what magicness it ended
/// under. `text` holds a NUL.
fn skip_pattern(text: &[u8], delim: ::core::ffi::c_int, magic: &mut Magic) -> usize {
    let is_magic = magic_isset() as ::core::ffi::c_int;
    let (end, ended) = skip_regexp_magic(text, delim, is_magic);
    *magic = ended;
    end
}

/// A copy of `len` bytes of the command line from `start`, cut at its end.
fn pattern_of(cc: Cc, start: ::core::ffi::c_int, len: ::core::ffi::c_int) -> Vec<u8> {
    let line = cc.bytes();
    let from = usize::try_from(start).unwrap_or(0).min(line.len());
    let to = from
        .saturating_add(usize::try_from(len).unwrap_or(0))
        .min(line.len());
    line[from..to].to_vec()
}

/// [`set_search_match`] on the window's own cursor: move it to the end of
/// the match, clamping it to the last line with the column advanced there.
fn set_cursor_to_search_match() {
    let mut t = Win::current().w_cursor;
    t.lnum += search_match_lines.get();
    t.col = search_match_endcol.get();
    let past_end = t.lnum > Buf::current().b_ml.ml_line_count;
    if past_end {
        t.lnum = Buf::current().b_ml.ml_line_count;
    }
    Win::current().w_cursor = t;
    if past_end {
        coladvance(Win::current(), MAXCOL);
    }
}

/// C's `changed_cline_bef_curs(curwin); update_topline(curwin);`, the pair
/// that follows every cursor move the preview makes.
fn curwin_cursor_moved() {
    changed_cline_bef_curs(Win::current());
    update_topline(Win::current());
}

/// C's `validate_cursor(curwin)`.
fn validate_curwin_cursor() {
    validate_cursor(Win::current());
}
