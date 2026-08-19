//! The user-facing search commands.
//!
//! [`do_search`] is `/` and `?` in full: it parses the offset off the end
//! of the pattern, echoes the command, and drives [`searchit`](super::searchit)
//! once per count — possibly several times over, because a search command
//! may be a `;`-separated chain (`/foo/;?bar`). [`showmatch`] is the
//! `'showmatch'` blink, which sits here because it is the other thing a
//! typed character can set going. The `f`/`t` character search is in
//! [`charsearch`](super::charsearch) and `gn`/`gN` in
//! [`select`](super::select).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::option::cpo_has;
use crate::pos::MAXCOL;
use crate::regexp::RE_LAST;
use crate::search::{
    SEARCH_ECHO, SEARCH_END, SEARCH_HIS, SEARCH_KEEP, SEARCH_MARK, SEARCH_MSG, SEARCH_NOOF,
    SEARCH_OPT, SEARCH_PEEK, SEARCH_REV, SEARCH_START, SEARCH_STAT_BUF_LEN,
    SEARCH_STAT_DEF_TIMEOUT,
};
use crate::types::{CmdModFlags, CpoFlag, FAIL, NUL, ShmFlag};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// ---------------------------------------------------------------------
// `/` and `?`.
// ---------------------------------------------------------------------

/// An `xmalloc`ed buffer that frees itself.
///
/// Two of `do_search`'s locals are allocations that outlive several early
/// exits: the echo buffer, and the copy `skip_regexp_ex` makes when it has
/// to rewrite `\?` to `?`.
struct Owned(*mut c_char);

impl Owned {
    const fn null() -> Self {
        Owned(ptr::null_mut())
    }

    fn as_ptr(&self) -> *mut c_char {
        self.0
    }

    /// Take ownership of `p`, freeing whatever was held.
    ///
    /// # Safety
    /// `p` must be an `xmalloc`ed allocation or null.
    unsafe fn replace(&mut self, p: *mut c_char) {
        unsafe { xfree(self.0 as *mut c_void) };
        self.0 = p;
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: only ever an xmalloc'ed allocation or null.
        unsafe { xfree(self.0 as *mut c_void) };
    }
}

/// The echoed search command, and its length.
///
/// The buffer is deliberately over-long and space-filled: the search
/// statistics are written into its tail, right-aligned, by
/// [`cmdline_search_stat`](super::cmdline_search_stat).
struct Echo {
    buf: Owned,
    len: size_t,
}

impl Echo {
    const fn none() -> Self {
        Echo {
            buf: Owned::null(),
            len: 0,
        }
    }
}

/// One `/pat/offset` of a search command; a `;`-separated chain is
/// several of these in a row.
struct SearchCmd {
    /// What is left of the command: the pattern, then the offset, then
    /// either nothing or `;` and the next one.
    pat: *mut c_char,
    patlen: size_t,
    /// `'/'` or `'?'` — this leg's direction.
    dirc: c_int,
    /// The character that ends the pattern. Usually `dirc`, but
    /// `:s%pat%rep` passes its own.
    delim: c_int,
}

/// Whether a `;` follows the offset, i.e. whether another leg comes next.
///
/// # Safety
/// `pat` must be null or NUL-terminated.
unsafe fn chained(pat: *mut c_char) -> bool {
    unsafe { !pat.is_null() && *pat as c_int == ';' as c_int }
}

/// Choose the pattern to search for.
///
/// An empty pattern means "the last one": normally that is left to
/// [`search_regcomp`](super::search_regcomp), which is told to use
/// `RE_LAST` — an empty string is passed through for it to fill in. Only
/// when there is no search pattern at all does the `:substitute` pattern
/// get named explicitly.
///
/// Answers `None` when neither remembered pattern exists (E35).
///
/// # Safety
/// `pat` must be null or NUL-terminated.
unsafe fn pattern_to_search(
    pat: *mut c_char,
    patlen: size_t,
    delim: c_int,
) -> Option<(*mut c_char, size_t)> {
    unsafe {
        if !(pat.is_null() || *pat as c_int == NUL || *pat as c_int == delim) {
            return Some((pat, patlen));
        }
        if !last_search_pattern().is_null() {
            // Make search_regcomp() use spats[RE_SEARCH].pat.
            return Some((c"".as_ptr() as *mut c_char, 0));
        }
        let subst = substitute_pattern();
        if subst.pat.is_null() {
            emsg(gettext(&raw const e_noprevre as *const c_char));
            return None;
        }
        Some((subst.pat, subst.patlen))
    }
}

/// Parse the `e`/`s`/`b` and `+n`/`-n` suffix that follows the pattern,
/// leaving it in [`search_offset`](super::search_offset).
///
/// Answers the first byte after the offset. For `get_address` (echo off)
/// the character offsets are not looked for, because they are meaningless
/// there and the `s` could be a `:substitute`.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn parse_offset(mut p: *mut c_char, options: c_int) -> *mut c_char {
    unsafe {
        let mut off = search_offset();
        off.line = false;
        off.end = false;
        off.off = 0;

        let at = |p: *mut c_char| *p as c_int;
        if at(p) == '+' as c_int || at(p) == '-' as c_int || ascii_isdigit(at(p)) {
            off.line = true;
        } else if options & SEARCH_OPT != 0
            && (at(p) == 'e' as c_int || at(p) == 's' as c_int || at(p) == 'b' as c_int)
        {
            off.end = at(p) == 'e' as c_int;
            p = p.offset(1);
        }

        if ascii_isdigit(at(p)) || at(p) == '+' as c_int || at(p) == '-' as c_int {
            // 'nr' or '+nr' or '-nr'
            off.off = if ascii_isdigit(at(p)) || ascii_isdigit(at(p.offset(1))) {
                atol(p) as int64_t
            } else if at(p) == '-' as c_int {
                -1 // a single '-'
            } else {
                1 // a single '+'
            };
            p = p.offset(1);
            while ascii_isdigit(at(p)) {
                p = p.offset(1);
            }
        }

        set_search_offset(off);
        p
    }
}

/// Spell the offset back out, for the echoed command line.
///
/// Answers how many bytes of `buf` were written; the result is *not*
/// NUL-terminated past that point, and the caller copies exactly that
/// many bytes.
fn write_offset(buf: &mut [c_char; 40], dirc: c_int, off: SearchOffset) -> size_t {
    let mut len = 0usize;
    if !(off.line || off.end || off.off != 0) {
        return 0;
    }
    buf[len] = dirc as c_char;
    len += 1;
    if off.end {
        buf[len] = b'e' as c_char;
        len += 1;
    } else if !off.line {
        buf[len] = b's' as c_char;
        len += 1;
    }
    buf[len] = NUL as c_char;
    if off.off != 0 || off.line {
        // SAFETY: writing into the tail of a 40-byte stack buffer, with
        // the remaining size passed; `%+ld` of an int64 fits in 21.
        len += unsafe {
            snprintf(
                (buf.as_mut_ptr()).add(len),
                buf.len() - len,
                c"%+ld".as_ptr(),
                off.off,
            )
        } as usize;
    }
    len
}

/// How much room to reserve for the echoed command.
///
/// With the search statistics switched on the buffer is deliberately as
/// wide as the space available, so that the statistics land right-aligned
/// in its tail; `msg_strtrunc` shortens it in the middle if it still does
/// not fit.
fn echo_size(plen: size_t, off_len: size_t, with_stats: bool) -> size_t {
    if !with_stats {
        return plen + off_len + 3;
    }
    // SAFETY: reading the screen geometry.
    let available = {
        if ui_has(kUIMessages) {
            0 // adjusted below
        } else if msg_scrolled.get() != 0 && !cmd_silent.get() {
            // Use all the columns.
            ((Rows.get() - msg_row.get()) * Columns.get() - 1) as size_t
        } else {
            // Use up to the 'showcmd' column.
            ((Rows.get() - msg_row.get() - 1) * Columns.get() + sc_col.get() - 1) as size_t
        }
    };
    available.max(plen + off_len + SEARCH_STAT_BUF_LEN as usize + 3)
}

/// Reverse the echoed command in place, for `'rightleft'` with `'rlc'`
/// naming the search command.
///
/// The pattern could be shown on the right in rightleft mode, but the
/// `'ruler'` and `'showcmd'` areas use that space too and would blank it
/// out again very soon; it is shown on the left with the text reversed
/// instead.
///
/// # Safety
/// `echo.buf` must be an `xmalloc`ed NUL-terminated string of `echo.len`.
unsafe fn reverse_echo(echo: &mut Echo) {
    unsafe {
        echo.buf.replace(reverse_text(echo.buf.as_ptr()));
        echo.len = strlen(echo.buf.as_ptr());
        let base = echo.buf.as_ptr();
        // Move the reversed text to the beginning of the buffer.
        let mut r = base;
        while *r as c_int == ' ' as c_int {
            r = r.offset(1);
        }
        let lead = r.offset_from(base) as size_t;
        let pat_len = echo.len - lead;
        ptr::copy(r, base, pat_len);
        // Overwrite the old text with blanks.
        if lead >= pat_len {
            ptr::write_bytes(r, b' ', pat_len);
        } else {
            ptr::write_bytes(base.add(pat_len), b' ', lead);
        }
    }
}

/// Echo the search command on the command line and answer the buffer it
/// was drawn from, which the search statistics are later written into.
///
/// # Safety
/// `searchstr` must be NUL-terminated.
unsafe fn echo_search_cmd(
    dirc: c_int,
    searchstr: *mut c_char,
    searchstrlen: size_t,
    options: c_int,
) -> (Echo, bool) {
    unsafe {
        if !(options & SEARCH_ECHO != 0
            && messaging()
            && msg_silent.get() == 0
            && (!cmd_silent.get() || !shortmess(ShmFlag::SEARCHCOUNT)))
        {
            return (Echo::none(), false);
        }

        // Compute msg_row early.
        msg_start();
        msg_ext_set_kind(c"search_cmd".as_ptr());

        let mut off_buf: [c_char; 40] = [0; 40];
        let off_len = if cmd_silent.get() {
            0
        } else {
            write_offset(&mut off_buf, dirc, search_offset())
        };

        let (p, plen) = if *searchstr as c_int == NUL {
            (last_search_pattern(), last_search_pattern_len())
        } else {
            (searchstr, searchstrlen)
        };

        let with_stats = !shortmess(ShmFlag::SEARCHCOUNT) || cmd_silent.get();
        let size = echo_size(plen, off_len, with_stats);
        let mut echo = Echo {
            buf: Owned(xmalloc(size) as *mut c_char),
            len: size - 1,
        };
        ptr::write_bytes(echo.buf.as_ptr(), b' ', size);
        *echo.buf.as_ptr().add(echo.len) = NUL as c_char;

        // Do not fill the buffer when cmd_silent is set: it is left empty
        // for the search-statistics feature.
        if !cmd_silent.get() {
            ui_busy_start();
            let buf = echo.buf.as_ptr();
            *buf = dirc as c_char;
            if utf_iscomposing_first(utf_ptr2char(p)) {
                // Use a space to draw the composing character on.
                *buf.add(1) = b' ' as c_char;
                ptr::copy(p, buf.add(2), plen);
            } else {
                ptr::copy(p, buf.add(1), plen);
            }
            if off_len > 0 {
                ptr::copy(off_buf.as_ptr(), buf.add(plen + 1), off_len);
            }

            let trunc = msg_strtrunc(echo.buf.as_ptr(), 1);
            if !trunc.is_null() {
                echo.buf.replace(trunc);
                echo.len = strlen(echo.buf.as_ptr());
            }

            if (*curwin.get()).w_onebuf_opt.wo_rl != 0
                && *(*curwin.get()).w_onebuf_opt.wo_rlc as c_int == 's' as c_int
            {
                reverse_echo(&mut echo);
            }

            msg_outtrans(echo.buf.as_ptr(), 0, false);
            msg_clr_eos();
            msg_check();
            gotocmdline(false);
            ui_flush();
            ui_busy_stop();
            msg_nowait.set(true);
        }

        (echo, !shortmess(ShmFlag::SEARCHCOUNT))
    }
}

/// Step the start position back over a character offset before searching,
/// so that `?pat?e+2` and `/pat/s-2` do not get stuck on the same match.
///
/// Not done for a line offset, because then this would not be vi
/// compatible; skipped when `pos.col` is near `MAXCOL` (a closed fold).
///
/// # Safety
/// The current buffer must be the one `pos` addresses.
unsafe fn back_off_start(pos: &mut pos_T, off: i64) {
    unsafe {
        let mut c = off;
        if off > 0 {
            while c != 0 {
                if decl(pos) == -1 {
                    break;
                }
                c -= 1;
            }
            if c != 0 {
                // At the start of the buffer; lnum == 0 is allowed here.
                pos.lnum = 0;
                pos.col = MAXCOL as colnr_T;
            }
        } else {
            while c != 0 {
                if incl(pos) == -1 {
                    break;
                }
                c += 1;
            }
            if c != 0 {
                // At the end of the buffer.
                pos.lnum = (*curbuf.get()).b_ml.ml_line_count + 1;
                pos.col = 0;
            }
        }
    }
}

/// Apply the offset to the position a search found.
///
/// Answers 2 when a *line* offset was added (which the caller reports back
/// as "found, and the motion is linewise"), 1 otherwise.
///
/// # Safety
/// The current buffer must be the one `pos` addresses.
unsafe fn add_offset(pos: &mut pos_T, off: SearchOffset) -> c_int {
    unsafe {
        if off.line {
            // Add the offset to the line number.
            let lnum = pos.lnum as i64 + off.off;
            let last = (*curbuf.get()).b_ml.ml_line_count;
            pos.lnum = if lnum < 1 {
                1
            } else if lnum > last as i64 {
                last
            } else {
                lnum as linenr_T
            };
            pos.col = 0;
            return 2; // pattern found, line offset added
        }
        if pos.col < MAXCOL - 2 {
            // Just in case.
            let mut c = off.off;
            if c > 0 {
                // To the right, checking for the end of the file.
                while c > 0 {
                    c -= 1;
                    if incl(pos) == -1 {
                        break;
                    }
                }
            } else {
                // To the left, checking for the start of the file.
                while c < 0 {
                    c += 1;
                    if decl(pos) == -1 {
                        break;
                    }
                }
            }
        }
        1
    }
}

/// Search for `pat`, `count` times, from the cursor.
///
/// `dirc` is `'/'` or `'?'`, or 0 to reuse the direction of the previous
/// search. A null or empty `pat` reuses the previous pattern. `options`
/// is a mask of the `SEARCH_*` flags; `oap` and `sia` may be null.
///
/// Careful: with a line offset of 0 (`spats[0].off.line` set and
/// `off.off == 0`) this makes the motion linewise without moving the
/// match position.
///
/// Answers 0 for failure, 1 for found, and 2 for found with a line offset
/// added.
///
/// # Safety
/// `pat` must be null or NUL-terminated and writable up to its
/// terminator; `oap` and `sia` must be null or valid.
pub unsafe fn do_search(
    oap: *mut oparg_T,
    dirc: c_int,
    search_delim: c_int,
    pat: *mut c_char,
    patlen: size_t,
    count: c_int,
    options: c_int,
    sia: *mut searchit_arg_T,
) -> c_int {
    unsafe {
        searchcmdlen.set(0);

        // A line offset is not remembered; this is vi compatible.
        let mut off = search_offset();
        if off.line && cpo_has(CpoFlag::LINEOFF) {
            off.line = false;
            off.off = 0;
            set_search_offset(off);
        }
        // Saved for when SEARCH_KEEP is used.
        let old_off = search_offset();

        let mut cmd = SearchCmd {
            pat,
            patlen,
            dirc,
            delim: search_delim,
        };

        // Find out the direction of the search.
        if cmd.dirc == 0 {
            cmd.dirc = off.dir as u8 as c_int;
        } else {
            let mut off = search_offset();
            off.dir = cmd.dirc as c_char;
            set_search_offset(off);
            set_vv_searchforward();
        }
        if options & SEARCH_REV != 0 {
            cmd.dirc = if cmd.dirc == '/' as c_int {
                '?' as c_int
            } else {
                '/' as c_int
            };
        }

        // Position of the last match; start searching at the cursor.
        let mut pos = (*curwin.get()).w_cursor;

        // If the cursor is in a closed fold, don't find another match in
        // the same fold.
        if cmd.dirc == '/' as c_int {
            if hasFolding(curwin.get(), pos.lnum, ptr::null_mut(), &raw mut pos.lnum) {
                pos.col = (MAXCOL - 2) as colnr_T; // avoid overflow when adding 1
            }
        } else if hasFolding(curwin.get(), pos.lnum, &raw mut pos.lnum, ptr::null_mut()) {
            pos.col = 0;
        }

        // Turn 'hlsearch' highlighting back on.
        if no_hlsearch.get() && options & SEARCH_KEEP == 0 {
            redraw_all_later(UPD_SOME_VALID);
            set_no_hlsearch(false);
        }

        // The copy skip_regexp_ex makes when it rewrites "\?" to "?"; it
        // has to outlive the loop, because `cmd.pat` points into it.
        let mut strcopy = Owned::null();
        let mut retval = 0;

        let found = 'end: {
            // Repeat the search when the pattern is followed by ';', as in
            // "/foo/;?bar".
            loop {
                let mut show_top_bot_msg = false;

                let Some((searchstr, searchstrlen)) =
                    pattern_to_search(cmd.pat, cmd.patlen, cmd.delim)
                else {
                    break 'end false;
                };
                let mut searchstr = searchstr;
                let mut searchstrlen = searchstrlen;

                // Where the pattern's terminator was replaced by a NUL, so
                // that normal_cmd() can be handed the command back intact.
                let mut dircp = ptr::null_mut::<c_char>();

                if !cmd.pat.is_null() && *cmd.pat as c_int != NUL {
                    // Find the end of the regular expression. If there is
                    // a matching '/' or '?', toss it.
                    let before = strcopy.as_ptr();
                    let mut copied = before;
                    let mut p = skip_regexp_ex(
                        cmd.pat,
                        cmd.delim,
                        c_int::from(magic_isset()),
                        &raw mut copied,
                        ptr::null_mut(),
                        ptr::null_mut(),
                    );
                    if copied != before {
                        // Made a copy of "pat" to change "\?" to "?".
                        strcopy.replace(copied);
                        let len = strlen(copied);
                        // Wrapping, as upstream: `patlen` is what the
                        // caller claimed, which for `get_address` is a
                        // prefix of a longer command line — the copy can
                        // be longer, and C's size_t difference is then
                        // the negative number this wants.
                        (*searchcmdlen.ptr()) += cmd.patlen.wrapping_sub(len) as c_int;
                        cmd.pat = copied;
                        cmd.patlen = len;
                        searchstr = copied;
                        searchstrlen = len;
                    }
                    if *p as c_int == cmd.delim {
                        searchstrlen = p.offset_from(cmd.pat) as size_t;
                        dircp = p; // remember where we put the NUL
                        *p = NUL as c_char;
                        p = p.offset(1);
                    }

                    p = parse_offset(p, options);

                    // Compute the length of the search command, for
                    // get_address().
                    let consumed = p.offset_from(cmd.pat) as size_t;
                    (*searchcmdlen.ptr()) += consumed as c_int;
                    cmd.patlen = cmd.patlen.wrapping_sub(consumed);
                    cmd.pat = p; // put pat after the search command
                }

                let (echo, show_search_stats) =
                    echo_search_cmd(cmd.dirc, searchstr, searchstrlen, options);

                let off = search_offset();
                if !off.line && off.off != 0 && pos.col < MAXCOL - 2 {
                    back_off_start(&mut pos, off.off);
                }

                // A ';'-chained leg always applies its offset, whatever
                // the caller asked for.
                let noof = if chained(cmd.pat) { 0 } else { SEARCH_NOOF };
                let c = searchit(
                    curwin.get(),
                    curbuf.get(),
                    &raw mut pos,
                    ptr::null_mut(),
                    if cmd.dirc == '/' as c_int {
                        FORWARD
                    } else {
                        BACKWARD
                    },
                    searchstr,
                    searchstrlen,
                    count,
                    c_int::from(off.end) * SEARCH_END
                        + (options
                            & (SEARCH_KEEP
                                + SEARCH_PEEK
                                + SEARCH_HIS
                                + SEARCH_MSG
                                + SEARCH_START
                                + noof)),
                    RE_LAST,
                    sia,
                );

                if !dircp.is_null() {
                    // Restore the second '/' or '?' for normal_cmd().
                    *dircp = cmd.delim as c_char;
                }
                if !shortmess(ShmFlag::SEARCH) && !sia.is_null() && (*sia).sa_wrapped != 0 {
                    show_top_bot_msg = true;
                }
                if c == FAIL {
                    break 'end false;
                }

                let off = search_offset();
                if off.end && !oap.is_null() {
                    (*oap).inclusive = true; // 'e' includes the last character
                }
                retval = 1; // pattern found

                if !sia.is_null() && (*sia).sa_wrapped != 0 {
                    apply_autocmds(
                        EVENT_SEARCHWRAPPED,
                        ptr::null_mut(),
                        ptr::null_mut(),
                        false,
                        ptr::null_mut(),
                    );
                }

                let mut has_offset = false;
                if options & SEARCH_NOOF == 0 || chained(cmd.pat) {
                    let org_pos = pos;
                    retval = add_offset(&mut pos, search_offset());
                    has_offset = !equalpos(pos, org_pos);
                }

                // Show [1/15] if 'S' is not in 'shortmess'.
                if show_search_stats {
                    let inexact = count != 1
                        || has_offset
                        || (fdo_flags.get() & kOptFdoFlagSearch == 0
                            && hasFolding(
                                curwin.get(),
                                (*curwin.get()).w_cursor.lnum,
                                ptr::null_mut(),
                                ptr::null_mut(),
                            ));
                    cmdline_search_stat(
                        cmd.dirc,
                        &raw mut pos,
                        &raw mut (*curwin.get()).w_cursor,
                        show_top_bot_msg,
                        echo.buf.as_ptr(),
                        echo.len,
                        inexact,
                        p_msc.get() as c_int,
                        SEARCH_STAT_DEF_TIMEOUT,
                    );
                }

                // The search command can be followed by a ';' to do
                // another search, as in "/pat/;/foo/+3;?bar". That is like
                // another search command, except that the remembered
                // direction is the first search's and that an error leaves
                // the cursor where it was. Not done when called by
                // get_address(), which handles ';' itself.
                if options & SEARCH_OPT == 0 || !chained(cmd.pat) {
                    break;
                }
                cmd.pat = cmd.pat.offset(1);
                cmd.dirc = *cmd.pat as u8 as c_int;
                cmd.delim = cmd.dirc;
                if cmd.dirc != '?' as c_int && cmd.dirc != '/' as c_int {
                    emsg(gettext(c"E386: Expected '?' or '/'  after ';'".as_ptr()));
                    break 'end false;
                }
                cmd.pat = cmd.pat.offset(1);
                cmd.patlen -= 1;
            }

            if options & SEARCH_MARK != 0 {
                setpcmark();
            }
            (*curwin.get()).w_cursor = pos;
            (*curwin.get()).w_set_curswant = 1;
            true
        };

        if !found {
            retval = 0;
        }
        if options & SEARCH_KEEP != 0 || cmdmod_has(CmdModFlags::KEEPPATTERNS) {
            set_search_offset(old_off);
        }
        retval
    }
}

// ---------------------------------------------------------------------
// `'showmatch'` and `gn`/`gN`.
// ---------------------------------------------------------------------

/// Whether `c` is one of the `'matchpairs'` characters that should blink,
/// given the effective right-to-left setting.
///
/// `'matchpairs'` is `"x:y,x:y"`: the opening character of each pair
/// blinks in left-to-right mode and the closing one in right-to-left.
///
/// # Safety
/// The current buffer must be valid.
unsafe fn mps_shows_match(c: c_int) -> bool {
    unsafe {
        let rightleft = (*curwin.get()).w_onebuf_opt.wo_rl ^ p_ri.get() != 0;
        let mut p = (*curbuf.get()).b_p_mps;
        while *p as c_int != NUL {
            if utf_ptr2char(p) == c && rightleft {
                return true;
            }
            p = p.offset((utfc_ptr2len(p) + 1) as isize);
            if utf_ptr2char(p) == c && !rightleft {
                return true;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
            if *p as c_int == NUL {
                return false;
            }
            p = p.offset(1);
        }
        false
    }
}

/// Briefly show the match for the just-typed `c`, for `'showmatch'`.
///
/// # Safety
/// Must be called with a valid current window and buffer.
pub unsafe fn showmatch(c: c_int) {
    unsafe {
        // Only show a match for characters in 'matchpairs'.
        if !mps_shows_match(c) {
            return;
        }

        let lpos = findmatch(ptr::null_mut(), NUL);
        if lpos.is_null() {
            vim_beep(kOptBoFlagShowmatch); // no match, so beep
            return;
        }
        if (*lpos).lnum < (*curwin.get()).w_topline || (*lpos).lnum >= (*curwin.get()).w_botline {
            return;
        }

        let mut vcol: colnr_T = 0;
        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
            getvcol(
                curwin.get(),
                lpos,
                ptr::null_mut(),
                &raw mut vcol,
                ptr::null_mut(),
            );
            if !(vcol >= (*curwin.get()).w_leftcol
                && vcol < (*curwin.get()).w_leftcol + (*curwin.get()).w_view_width)
            {
                return;
            }
        }

        // 'scrolloff' and 'sidescrolloff' are window-local with a global
        // fallback; the blink writes through whichever is in effect.
        let so: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_so >= 0 {
            &raw mut (*curwin.get()).w_onebuf_opt.wo_so
        } else {
            p_so.ptr()
        };
        let siso: *mut OptInt = if (*curwin.get()).w_onebuf_opt.wo_siso >= 0 {
            &raw mut (*curwin.get()).w_onebuf_opt.wo_siso
        } else {
            p_siso.ptr()
        };

        let mpos = *lpos; // save the pos, update_screen() may change it
        let save_cursor = (*curwin.get()).w_cursor;
        let save_so = *so;
        let save_siso = *siso;

        // Handle "$" in 'cpo': if the ')' is typed on top of the "$", stop
        // displaying the "$".
        if dollar_vcol.get() >= 0 && dollar_vcol.get() == (*curwin.get()).w_virtcol {
            dollar_vcol.set(-1);
        }
        (*curwin.get()).w_virtcol += 1; // do display ')' just before "$"

        let save_dollar_vcol = dollar_vcol.get();
        let save_state = State.get();
        State.set(MODE_SHOWMATCH);
        ui_cursor_shape(); // may show a different cursor shape
        (*curwin.get()).w_cursor = mpos; // move to the matching char
        *so = 0; // don't use 'scrolloff' here
        *siso = 0; // don't use 'sidescrolloff' here
        show_cursor_info_later(false);
        update_screen(); // show the new char
        setcursor();
        ui_flush();
        // Restore dollar_vcol: setcursor() may call curs_rows(), which
        // resets it when the matching position is on an earlier line and
        // has a higher column number.
        dollar_vcol.set(save_dollar_vcol);

        // Brief pause, unless 'm' is present in 'cpo' and a character is
        // available.
        if cpo_has(CpoFlag::SHOWMATCH) {
            os_delay(p_mat.get() as u64 * 100 + 8, true);
        } else if !char_avail() {
            os_delay(p_mat.get() as u64 * 100 + 9, false);
        }

        (*curwin.get()).w_cursor = save_cursor; // restore cursor position
        *so = save_so;
        *siso = save_siso;
        State.set(save_state);
        ui_cursor_shape(); // may show a different cursor shape
    }
}
