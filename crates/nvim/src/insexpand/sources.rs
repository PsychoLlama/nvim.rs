//! Scanning buffers, dictionaries, thesauruses and registers for matches.
//!
//! [`ins_compl_dictionaries`] and [`ins_compl_files`] are the `'dictionary'`
//! and `'thesaurus'` file walk; [`get_next_default_completion`] is the
//! keyword search through the buffers `'complete'` names, driven by
//! [`ins_compl_next_buf`]; [`get_register_completion`] is the `CTRL-X
//! CTRL-R` source.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::charset::skip;
use crate::cstr;
use crate::guard::Suppress;
use crate::mbyte::{char_at, cluster_len};
use crate::memline::Lines;
use crate::path::ExpandFlags;
use crate::strings::has_char;
use crate::types::{FAIL, Failed, IOSIZE, NUL, OK, ShmFlag};
use crate::vim_snprintf;
use crate::winlayer::BufId;
use crate::winlayer::{Buf, Win, first_buffer, first_window};

/// Add every identifier matching `pat` in the `'dictionary'`-style list
/// `dict_start` to the completions.
///
/// `flags` is `DICT_FIRST` and/or `DICT_EXACT`; `thesaurus` selects thesaurus
/// completion.
///
/// # Safety
///
/// `dict_start` must point at a NUL-terminated string, unaliased for the
/// call. `pat` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn ins_compl_dictionaries(
    dict_start: *mut c_char,
    pat: *mut c_char,
    flags: c_int,
    thesaurus: bool,
) {
    let mut dict = dict_start;
    let mut dir = compl_direction.get();

    if unsafe { *dict } as c_int == NUL {
        // When 'dictionary' is empty and spell checking is enabled use
        // "spell".
        if !thesaurus && Win::current().w_onebuf_opt.wo_spell != 0 {
            dict = c"spell".as_ptr().cast_mut();
        } else {
            return;
        }
    }

    let mut buf = unsafe { xmalloc(LSIZE as size_t) }.cast::<c_char>();
    // So that we can leave through 'theend.
    let mut regmatch = RegMatch::new(ptr::null_mut(), false);

    // If 'infercase' is set, don't use 'smartcase' here.
    let save_p_scs = p_scs();
    if Buf::current().b_p_inf != 0 {
        P_SCS.set(false);
    }

    // C's `goto theend`, i.e. free and restore below without scanning.
    'theend: {
        // When invoked to match whole lines for CTRL-X CTRL-L adjust the
        // pattern to only match at the start of a line.  Otherwise just
        // match the pattern.  Also need to double backslashes.
        if ctrl_x_mode_line_or_eval() {
            let pat_esc = unsafe { vim_strsave_escaped(pat, c"\\".as_ptr()) };
            let len = unsafe { cstr::bytes_at(pat_esc) }.len() + 10;
            let ptr = unsafe { xmalloc(len) }.cast::<c_char>();
            unsafe { vim_snprintf!(ptr, len, c"^\\s*\\zs\\V%s".as_ptr(), pat_esc) };
            regmatch.regprog = vim_regcomp(unsafe { cstr::at(ptr) }, RE_MAGIC);
            unsafe { xfree(pat_esc.cast::<c_void>()) };
            unsafe { xfree(ptr.cast::<c_void>()) };
        } else {
            regmatch.regprog = vim_regcomp(
                unsafe { cstr::at(pat) },
                if magic_isset() { RE_MAGIC } else { 0 },
            );
            if regmatch.regprog.is_null() {
                break 'theend;
            }
        }

        // Ignore case depends on 'ignorecase', 'smartcase' and "pat".
        regmatch.rm_ic = unsafe { ignorecase(pat) };
        while unsafe { *dict } as c_int != NUL && !got_int.get() && !compl_interrupted.get() {
            // Copy one dictionary file name into buf.
            // Upstream leaves both uninitialised: every path that reads
            // them either assigns first or is guarded by `count > 0`.
            let mut files: *mut *mut c_char = ptr::null_mut();
            let mut count = 0;
            if flags == DICT_EXACT {
                count = 1;
                files = &raw mut dict;
            } else {
                // Expand wildcards in the dictionary name, but do not allow
                // backticks (for security, the 'dict' option may have been
                // set in a modeline).
                let comma = c",".as_ptr().cast_mut();
                // SAFETY: `dict` walks the option string and `buf` has
                // `LSIZE` writable bytes.
                unsafe { copy_option_part(&raw mut dict, buf, LSIZE as size_t, comma) };
                // SAFETY: `buf` now holds one NUL-terminated file pattern.
                if !thesaurus && unsafe { cstr::eq_bytes(buf, b"spell") } {
                    count = -1;
                } else {
                    // SAFETY: as above.
                    let backtick = has_char(unsafe { cstr::at(buf) }, '`' as c_int);
                    let failed = !backtick && {
                        let flags = ExpandFlags::FILE | ExpandFlags::SILENT;
                        let (n, out) = (&raw mut count, &raw mut files);
                        // SAFETY: `buf` is one NUL-terminated pattern, and
                        // the two out-parameters are this frame's locals.
                        let ok = unsafe { expand_wildcards(1, &raw mut buf, n, out, flags) };
                        ok.is_err()
                    };
                    if backtick || failed {
                        count = 0;
                    }
                }
            }

            if count == -1 {
                // Complete from active spelling.  Skip "\<" in the pattern,
                // we don't use it as a RE.
                let word = if unsafe { *pat } as c_int == '\\' as c_int
                    && unsafe { *pat.offset(1) } as c_int == '<' as c_int
                {
                    unsafe { pat.offset(2) }
                } else {
                    pat
                };
                unsafe { spell_dump_compl(word, regmatch.rm_ic as c_int, &raw mut dir, 0) };
            } else if count > 0 {
                // SAFETY: `files` is `count` NUL-terminated names and `buf`
                // a scratch buffer of `LSIZE`.
                unsafe {
                    ins_compl_files(count, files, thesaurus, flags, &mut regmatch, buf, &mut dir)
                };
                if flags != DICT_EXACT {
                    unsafe { free_wild(count, files) };
                }
            }
            if flags != 0 {
                break;
            }
        }
    }

    P_SCS.set(save_p_scs);
    unsafe { vim_regfree(regmatch.regprog) };
    unsafe { xfree(buf.cast::<c_void>()) };
}

/// Add all the words in `line` from the thesaurus file `fname`, skipping the
/// one starting at `skip_word`; answers OK on success, and where the walk
/// stopped.
fn thesaurus_add_words_in_line(
    fname: Option<&CStr>,
    line: &[u8],
    dir: Direction,
    skip_word: usize,
) -> (c_int, usize) {
    let mut status = OK;

    // Add the other matches on the line.
    let mut at = 0;
    while !got_int.get() {
        // Find the start of the next word, skipping white space and
        // punctuation.
        at = word_start(line, at);
        if at >= line.len() || line[at] == b'\n' {
            break;
        }
        let wstart = at;

        // Find the end of the word.  Japanese words may have characters in
        // different classes, so only separate words with single-byte
        // non-word characters.
        while at < line.len() {
            let l = cluster_len(&line[at..]);
            if l < 2 && !vim_iswordc(c_int::from(line[at])) {
                break;
            }
            at += l;
        }

        // Add the word, skipping the regexp match.
        if wstart != skip_word {
            status = add_scanned_word(line, wstart, at, fname, dir, FUZZY_SCORE_NONE);
            if status == FAIL {
                break;
            }
        }
    }
    (status, at)
}

/// Read `count` dictionary/thesaurus `files` and add the text matching
/// `regmatch`, reading each line into `buf`.
///
/// # Safety
///
/// `files` must point at `count` NUL-terminated names. `buf` must have
/// `LSIZE` writable bytes.
pub(crate) unsafe fn ins_compl_files(
    count: c_int,
    files: *mut *mut c_char,
    thesaurus: bool,
    flags: c_int,
    regmatch: &mut RegMatch,
    buf: *mut c_char,
    dir: &mut Direction,
) {
    let mut progress = [0 as c_char; IOSIZE as usize];
    // The leader is copied: the scan checks for typed keys between lines.
    let leader = cot_fuzzy().then(|| ins_compl_leader_str().to_owned());
    let leader = leader
        .as_ref()
        .map(String_0::as_cstr)
        .filter(|l| !l.is_empty());

    let mut i = 0;
    while i < count as usize && !got_int.get() && !ins_compl_interrupted() {
        // SAFETY: the caller's `count` names.
        let file = unsafe { *files.add(i) };
        // SAFETY: a NUL-terminated name.
        let fname = unsafe { cstr::at(file) };
        let fp = unsafe { os_fopen(file, c"r".as_ptr()) }; // open dictionary file
        let quiet = shortmess(ShmFlag::COMPLETIONSCAN);
        if flags != DICT_EXACT && !quiet && !compl_autocomplete.get() {
            let fmt = gettext(c"Scanning dictionary: %s");
            let (out, size) = (progress.as_mut_ptr(), IOSIZE as size_t);
            // SAFETY: `out` addresses all `size` bytes and `file` is a
            // NUL-terminated name.
            unsafe { vim_snprintf!(out, size, fmt.as_ptr(), file) };
            // SAFETY: `vim_snprintf` NUL-terminated `out`.
            unsafe { scan_progress(out) };
        }

        if fp.is_null() {
            i += 1;
            continue;
        }

        // Read the dictionary file line by line, checking each for a match.
        // SAFETY: `buf` has `LSIZE` bytes and `fp` is open.
        while !got_int.get() && !ins_compl_interrupted() && !unsafe { vim_fgets(buf, LSIZE, fp) } {
            // SAFETY: `vim_fgets` left one NUL-terminated line in `buf`,
            // which nothing else writes while it is read here.
            let line_cstr = unsafe { cstr::at(buf) };
            let line = line_cstr.to_bytes();
            if let Some(leader) = leader {
                let end = line_end(line);
                let mut at = 0;
                while at < end {
                    let Some(found) = fuzzy_match_in_line(line, at, leader) else {
                        break;
                    };
                    let start = found.start;
                    let word_end = if ctrl_x_mode_line_or_eval() {
                        start + line_end(&line[start..])
                    } else {
                        word_end(line, start)
                    };
                    let score = found.score;
                    if add_scanned_word(line, start, word_end, Some(fname), *dir, score) == FAIL {
                        break;
                    }
                    at = word_end; // start from the next word
                    if compl_get_longest.get() && ctrl_x_mode_normal() && best_score_is(score) {
                        compl_num_bests.set(compl_num_bests.get() + 1);
                    }
                }
            } else {
                let mut at = 0;
                while vim_regexec(regmatch, line_cstr, at) {
                    let start = regmatch.starts[0].unwrap_or(0);
                    at = if ctrl_x_mode_line_or_eval() {
                        start + line_end(&line[start..])
                    } else {
                        word_end(line, start)
                    };
                    let mut add_r =
                        add_scanned_word(line, start, at, Some(fname), *dir, FUZZY_SCORE_NONE);
                    if thesaurus {
                        // For a thesaurus, add all the words in the line.
                        (add_r, at) = thesaurus_add_words_in_line(Some(fname), line, *dir, start);
                    }
                    if add_r == OK {
                        // If dir was BACKWARD then honour it just once.
                        *dir = FORWARD;
                    } else if add_r == FAIL {
                        break;
                    }
                    // Avoid an expensive call to vim_regexec() at the end
                    // of the line.
                    if line.get(at) == Some(&b'\n') || got_int.get() {
                        break;
                    }
                }
            }
            line_breakcheck();
            ins_compl_check_keys(50, false);
        }
        // SAFETY: the file opened above.
        unsafe { fclose(fp) };
        i += 1;
    }
}

/// The next window, loaded buffer or non-loaded buffer (depending on `flag`)
/// after `buffer` that has not been scanned; `curbuf` when there is none.
///
/// `curbuf` is special: called with `buf == curbuf` this has to be the first
/// call for a given flag/expansion. -- Acevedo
///
/// Safe: [`Buf`] is the live buffer the walk starts from, and the window it
/// remembers between calls is vetted below rather than trusted.
pub(crate) fn ins_compl_next_buf(mut buffer: Buf, flag: c_int) -> Buf {
    // `next_buf_window` outlives the call, and a completion runs user
    // functions and Lua in between, so it stays a handle that `win_valid`
    // vets -- a `Win` would be promising a liveness nothing here can keep.
    let wp = next_buf_window;
    if flag == 'w' as c_int {
        // Just windows.
        if buffer.raw() == Buf::current_raw() || !wp.get().is_some_and(win_valid) {
            // First call for this flag/expansion, or the window was closed.
            wp.set(Some(Win::current().id()));
        }
        debug_assert!(wp.get().is_some());
        // `wp` is `curwin` or a window `win_valid` just vouched for, and from
        // there the editor's own window list, which is live.
        let mut at = wp.get().and_then(WinId::get).expect("just set above");
        loop {
            // Move to the next window, wrapping to the first at the end.
            at = at.next().or_else(first_window).expect("a non-empty list");
            wp.set(Some(at.id()));
            // Stop if we're back at the start, or found an unscanned
            // buffer in a focusable window.
            if at.is_current() || (!at.buffer().b_scanned && at.w_config.focusable) {
                break;
            }
        }
        buffer = at.buffer();
    } else {
        // 'b' (just loaded buffers), 'u' (just non-loaded buffers) or 'U'
        // (unlisted buffers).  When completing whole lines skip unloaded
        // buffers.
        loop {
            // Move to the next buffer, wrapping to the first at the end.
            buffer = match buffer.next() {
                Some(next) => next,
                None => first_buffer().expect("the editor always has a buffer"),
            };
            // Stop if we're back at the start buffer.
            if buffer.raw() == Buf::current_raw() {
                break;
            }
            let skip_buffer = if flag == 'U' as c_int {
                buffer.b_p_bl != 0
            } else {
                buffer.b_p_bl == 0 || buffer.b_ml.ml_mfp.is_null() != (flag == 'u' as c_int)
            };
            // Stop if we found a buffer that matches our criteria.
            if !skip_buffer && !buffer.b_scanned {
                break;
            }
        }
    }
    buffer
}

/// Whether the character at the start of `text` belongs to a word —
/// [`vim_iswordp`] over a slice; an empty one is the terminator, which does
/// not.
fn is_word_at(text: &[u8]) -> bool {
    vim_iswordc(char_at(text))
}

/// The next word or line from `ins_buf` at `pos`, copied into `out` from its
/// start to the end of the string it was found in (which
/// [`ins_compl_add_infercase`] wants), with its length and whether the next
/// `CTRL-X <>` sets the initial position; `None` when there is nothing to
/// add.
pub(crate) fn next_word_or_line(
    ins_buf: Buf,
    pos: Pos,
    out: &mut Vec<u8>,
) -> Option<(usize, bool)> {
    let mut lines = Lines::in_buffer(ins_buf);
    let (lnum, col) = (pos.lnum, pos.col as usize);
    let line_count = ins_buf.b_ml.ml_line_count;
    let typed_len = compl_length.get() as usize;
    out.clear();

    if ctrl_x_mode_line_or_eval() {
        if compl_status_adding() {
            if lnum >= line_count {
                return None;
            }
            let next = lines.line(lnum + 1);
            let skip = if p_paste() { 0 } else { skip::white(next) };
            out.extend_from_slice(&next[skip..]);
        } else {
            let line = lines.line(lnum);
            out.extend_from_slice(&line[col.min(line.len())..]);
        }
        return Some((out.len(), false));
    }

    let line = lines.line(lnum);
    let col = col.min(line.len());
    out.extend_from_slice(&line[col..]);
    let mut at = col;
    if compl_status_adding() && typed_len <= line.len() - col {
        at += typed_len;
        // Skip if already inside a word.
        if is_word_at(&line[at..]) {
            return None;
        }
        // Find the start of the next word.
        at = word_start(line, at);
    }
    // Find the end of this word.
    let mut len = word_end(line, at) - col;
    let mut cont_s_ipos = false;

    if compl_status_adding() && len == typed_len {
        if lnum < line_count {
            // Try the next line, if any: the new word will be "joined" as
            // if the normal command "J" was used.  IOSIZE is always greater
            // than typed_len, so the copy always fits -- Acevedo
            out.truncate(len);
            let next = lines.line(lnum + 1);
            let start = skip::white(next);
            // Find the start and then the end of the next word.
            let end = word_end(next, word_start(next, start));
            if end > start {
                if next[start] != b')' && c_int::from(out[len - 1]) != TAB {
                    if out[len - 1] != b' ' {
                        out.push(b' ');
                        len += 1;
                    }
                    // The joined line =~ "\k.* ", thus len >= 2.
                    if p_js() && matches!(out[len - 2], b'.' | b'?' | b'!') {
                        out.push(b' ');
                        len += 1;
                    }
                }
                // Copy as much as possible of the new word.
                let room = IOSIZE as usize - len - 1;
                out.extend_from_slice(&next[start..end.min(start + room)]);
                len = out.len();
                cont_s_ipos = true;
            }
        }
        if len == typed_len {
            return None;
        }
    }
    Some((len, cont_s_ipos))
}

/// The next set of words matching `compl_pattern` for default completion —
/// normal `^P`/`^N` and `^X^L`.
///
/// Searches `st.ins_buf` from its current match position in the
/// `compl_direction` direction, `start_pos` being where the completion
/// started; with `st.set_match_pos` set, `st.first_match_pos` and
/// `st.last_match_pos` are set too. Answers `Ok` if a new match was found,
/// otherwise `Err`.
pub(crate) fn get_next_default_completion(
    st: &mut InsComplNextState,
    start_pos: Pos,
) -> Result<(), Failed> {
    // The match found, from its start to the end of the string it is in;
    // upstream points into the line, or into `IObuff` for a joined line.
    let mut found = ::core::mem::take(&mut st.found);
    let mut len = 0;
    let in_fuzzy_collect = !compl_status_adding() && cot_fuzzy() && compl_length.get() > 0;
    // A copy: the add below can run code that changes it.
    let leader = ins_compl_leader_str().to_owned();
    let mut score = FUZZY_SCORE_NONE;
    // The scan's buffer survives the timers and RPC that
    // `ins_compl_check_keys` lets run between two passes only if nothing
    // wiped it. A wiped one has nothing more to give: the caller then marks
    // this source done and moves on to the next in 'complete'.
    let Some(ins_buf) = st.ins_buf.and_then(BufId::get) else {
        return Err(Failed);
    };
    let in_curbuf = Some(ins_buf) == Buf::current_or_none();

    // If 'infercase' is set, don't use 'smartcase' here.
    let save_p_scs = p_scs();
    if ins_buf.b_p_inf != 0 {
        P_SCS.set(false);
    }

    // Buffers other than curbuf are scanned from the beginning or the end
    // but never from the middle, thus setting nowrapscan in these buffers
    // is a good idea; on the other hand, we always set wrapscan for curbuf
    // to avoid missing matches -- Acevedo, Webb
    let save_p_ws = p_ws();
    if !in_curbuf {
        P_WS.set(false);
    } else if st.cpt.at() == b'.' {
        P_WS.set(true);
    }

    // The position the search moves, held here and written back after each
    // search: it is one of `st`'s two match positions.
    let mut pos = *st.cur_match_pos();
    let mut looped_around = false;
    let mut found_new_match;
    loop {
        let mut cont_s_ipos = false;

        // Don't want messages for wrapscan.
        let silenced = Suppress::messages();
        let dir = compl_direction.get();
        if in_fuzzy_collect {
            let pattern = leader.as_cstr();
            let hit =
                search_for_fuzzy_match(ins_buf, &mut pos, pattern, dir, start_pos, &mut found);
            found_new_match = Err(Failed);
            if let Some(hit) = hit {
                len = hit.len;
                score = hit.score.unwrap_or(score);
                found_new_match = Ok(());
            }
        } else if ctrl_x_mode_whole_line()
            || ctrl_x_mode_eval()
            || compl_cont_status.get() & CONT_SOL != 0
        {
            // ctrl_x_mode_line_or_eval(), or a word-wise search that has
            // added a word that was at the beginning of the line.
            let pat = compl_pattern().data();
            // SAFETY: `pos` is a position in `ins_buf` and `pat` is the
            // running completion's NUL-terminated pattern.
            found_new_match = unsafe { search_for_exact_line(ins_buf, &raw mut pos, dir, pat) };
        } else {
            let (pat, pat_len) = compl_pattern().parts();
            let flags = SEARCH_KEEP + SEARCH_NFMSG;
            // SAFETY: as above, with the pattern's own length.
            let found = unsafe {
                searchit(
                    None,
                    ins_buf,
                    &raw mut pos,
                    ptr::null_mut(),
                    dir,
                    pat,
                    pat_len,
                    1,
                    flags,
                    RE_LAST,
                    ptr::null_mut(),
                )
            };
            found_new_match = if found == FAIL { Err(Failed) } else { Ok(()) };
        }
        drop(silenced);
        *st.cur_match_pos() = pos;

        if !compl_started.get() || st.set_match_pos {
            // Set "compl_started" even on failure.
            compl_started.set(true);
            st.first_match_pos = pos;
            st.last_match_pos = pos;
            st.set_match_pos = false;
        } else if st.first_match_pos.lnum == st.last_match_pos.lnum
            && st.first_match_pos.col == st.last_match_pos.col
        {
            found_new_match = Err(Failed);
        } else {
            // Passing the previous match going forwards (or backwards) is
            // the wrap-around; the second time round there is nothing new.
            let prev = st.prev_match_pos;
            let passed = if compl_dir_forward() {
                prev.lnum > pos.lnum || (prev.lnum == pos.lnum && prev.col >= pos.col)
            } else {
                prev.lnum < pos.lnum || (prev.lnum == pos.lnum && prev.col <= pos.col)
            };
            if passed {
                if looped_around {
                    found_new_match = Err(Failed);
                } else {
                    looped_around = true;
                }
            }
        }
        st.prev_match_pos = pos;
        if found_new_match.is_err() {
            break;
        }

        // When ADDING, the text before the cursor matches: skip it.
        if compl_status_adding()
            && in_curbuf
            && start_pos.lnum == pos.lnum
            && start_pos.col == pos.col
        {
            continue;
        }

        if !in_fuzzy_collect {
            let Some(next) = next_word_or_line(ins_buf, pos, &mut found) else {
                continue;
            };
            (len, cont_s_ipos) = next;
        }
        // C's `strcmp(ptr, ins_compl_leader()) == 0`: the whole rest of the
        // string against the leader.
        if ins_compl_has_preinsert() && ins_compl_leader_str().with_bytes(|l| *l == *found) {
            continue;
        }

        if is_nearest_active() && in_curbuf {
            score = (pos.lnum - Win::current().w_cursor.lnum) as c_int;
            score = score.abs();
        }

        let fname = if in_curbuf {
            None
        } else {
            ins_buf.name.short()
        };
        let (ic, dir) = (p_ic(), kDirectionNotSet);
        let add_r = ins_compl_add_infercase(&found, len, ic, fname, dir, cont_s_ipos, score);
        if add_r != NOTDONE {
            if in_fuzzy_collect && best_score_is(score) {
                compl_num_bests.set(compl_num_bests.get() + 1);
            }
            found_new_match = Ok(());
            break;
        }
    }

    P_SCS.set(save_p_scs);
    P_WS.set(save_p_ws);
    st.found = found;
    found_new_match
}

/// Add completion matches from the contents of every usable register.
pub(crate) fn get_register_completion() {
    // Upstream's `!compl_orig_text.data || (p_ic ? STRNICMP : strncmp)(…)`:
    // a candidate counts when there is no original text to compare against,
    // or it starts with it.
    let orig = (!compl_orig_text().is_unset()).then(|| compl_orig_text().to_owned());
    let starts_with_orig = |s: &[u8]| {
        let Some(orig) = &orig else {
            return true;
        };
        let orig = orig.as_bytes();
        if p_ic() {
            // SAFETY: `s` runs to the end of a NUL-terminated register line,
            // and `strncasecmp` reads `orig` no further than its length.
            unsafe { strncasecmp(s.as_ptr().cast(), orig.as_ptr().cast(), orig.len()) == 0 }
        } else {
            // `strncmp` over the original's length, which holds no NUL.
            s.starts_with(orig)
        }
    };

    let mut dir = compl_direction.get();
    let adding_mode = compl_status_adding();

    for i in 0..NUM_REGISTERS {
        let regname = get_register_name(i);
        // Skip an invalid or black hole register.
        if !valid_yank_reg(regname, false) || regname == '_' as c_int {
            continue;
        }

        // The register's lines, copied: adding a match can run code that
        // changes the register.
        // SAFETY: a valid register name; the copy `copy_register` answers is
        // this frame's own, read and then freed here.
        let lines: Vec<XString> = unsafe {
            let reg = copy_register(regname);
            let (array, size) = ((*reg).y_array, (*reg).y_size);
            let count = if array.is_null() { 0 } else { size };
            let lines = (0..count)
                .map(|j| (*array.add(j)).data())
                .filter(|line| !line.is_null())
                .map(|line| XString::from_cstr(cstr::at(line)))
                .collect();
            free_register(reg);
            xfree(reg.cast::<c_void>());
            lines
        };

        for line in &lines {
            if adding_mode {
                if line.is_empty() {
                    continue;
                }
                let added = starts_with_orig(line)
                    && add_scanned_word(line, 0, line.len(), None, dir, FUZZY_SCORE_NONE) == OK;
                if added {
                    dir = FORWARD;
                }
                continue;
            }
            let mut at = 0;
            while at < line.len() {
                let old = at;
                at = word_start(line, at);
                if at >= line.len() {
                    break;
                }
                let mut end = word_end(line, at);
                if end <= at {
                    end = at + cluster_len(&line[at..]);
                }
                let end = end.min(line.len());
                let added = end > at
                    && starts_with_orig(&line[at..])
                    && add_scanned_word(line, at, end, None, dir, FUZZY_SCORE_NONE) == OK;
                if added {
                    dir = FORWARD;
                }
                at = end;
                if at <= old {
                    at = old + cluster_len(&line[old..]);
                }
            }
        }
    }
}

/// Whether `score` is the score of the match after the head: the best so
/// far, the fuzzy matches being kept sorted.
fn best_score_is(score: c_int) -> bool {
    first_match()
        .and_then(MatchId::next)
        .is_some_and(|best| best.with(|m| m.score) == score)
}

/// [`ins_compl_add_infercase`] for the word `start .. end` of `line` (which
/// runs to the end of its string), with the flags every caller in this
/// module passes.
fn add_scanned_word(
    line: &[u8],
    start: usize,
    end: usize,
    fname: Option<&CStr>,
    dir: Direction,
    score: c_int,
) -> c_int {
    ins_compl_add_infercase(
        &line[start..],
        end - start,
        p_ic(),
        fname,
        dir,
        false,
        score,
    )
}

// ---------------------------------------------------------------------------
// Finding a fuzzy match inside a line, and then inside a buffer.

/// A fuzzy match of a word in a line: where it starts, how long it is, and
/// its score.
pub(super) struct WordMatch {
    pub start: usize,
    pub len: usize,
    pub score: c_int,
}

/// Split `line` from `at` into words and fuzzy match `pat` against each: the
/// first that matches, or `None` when none does before the line's end.
pub(super) fn fuzzy_match_in_line(line: &[u8], at: usize, pat: &CStr) -> Option<WordMatch> {
    let end_of_line = at + line_end(&line[at..]);
    // Each word is scored as a C string of its own; upstream terminates it in
    // place instead.
    let mut word = Vec::new();
    let mut s = at;
    while s < end_of_line {
        let start = word_start(line, s);
        if start >= line.len() {
            break;
        }
        let end = word_end(line, start);
        word.clear();
        word.extend_from_slice(&line[start..end]);
        word.push(0);
        let text = CStr::from_bytes_until_nul(&word).expect("terminated just above");
        let score = fuzzy_match_str(text, pat);
        if score != FUZZY_SCORE_NONE {
            return Some(WordMatch {
                start,
                len: end - start,
                score,
            });
        }

        // Carry on after the word just tried.
        s = end;
        while s < line.len() && !is_word_at(&line[s..]) {
            s += cluster_len(&line[s..]);
        }
    }
    None
}

/// Where a fuzzy match was found in a buffer line: its length in bytes, and
/// its score — missing for a whole-line match, where upstream leaves the
/// caller's score alone.
pub(super) struct LineMatch {
    pub len: usize,
    pub score: Option<c_int>,
}

/// Search `buffer` for the next fuzzy match of `pattern`, starting at `pos` and
/// going in `dir`, wrapping around to `start_pos` if `'wrapscan'` is set.
/// `pos` is left on the match, and `out` holds the match on to the end of its
/// line. In whole-line mode (`CTRL-X CTRL-L`) whole lines are matched rather
/// than words.
pub(super) fn search_for_fuzzy_match(
    buffer: Buf,
    pos: &mut Pos,
    pattern: &CStr,
    dir: c_int,
    start_pos: Pos,
    out: &mut Vec<u8>,
) -> Option<LineMatch> {
    let whole_line = ctrl_x_mode_whole_line();
    let mut current_pos = *pos;
    let mut lines = Lines::in_buffer(buffer);

    // Where the search has come full circle. Another buffer is walked
    // from wherever it is to its end rather than back to the start.
    let circly_end = if buffer == Buf::current() {
        start_pos
    } else {
        Pos {
            lnum: buffer.b_ml.ml_line_count,
            col: 0,
            coladd: 0,
        }
    };
    if whole_line && start_pos.lnum != pos.lnum {
        current_pos.lnum += dir as LineNr;
    }
    let mut looped_around = false;
    loop {
        if looped_around
            && (if whole_line {
                current_pos.lnum == circly_end.lnum
            } else {
                equalpos(current_pos, circly_end)
            })
        {
            return None;
        }
        if current_pos.lnum >= 1 && current_pos.lnum <= buffer.b_ml.ml_line_count {
            let at = if whole_line {
                0
            } else {
                current_pos.col as usize
            };
            let line = lines.line(current_pos.lnum);
            if at < line.len() {
                if whole_line {
                    let text = lines.line_cstr(current_pos.lnum, 0);
                    if fuzzy_match_str(text, pattern) != FUZZY_SCORE_NONE {
                        *pos = current_pos;
                        out.clear();
                        out.extend_from_slice(text.to_bytes());
                        let len = out.len();
                        return Some(LineMatch { len, score: None });
                    }
                } else {
                    if let Some(found) = fuzzy_match_in_line(line, at, pattern) {
                        current_pos.col = (found.start + found.len) as ColNr;
                        *pos = current_pos;
                        out.clear();
                        out.extend_from_slice(&line[found.start..]);
                        let score = Some(found.score);
                        return Some(LineMatch {
                            len: found.len,
                            score,
                        });
                    }
                    if looped_around && current_pos.lnum == circly_end.lnum {
                        return None;
                    }
                }
            }
        }

        // On to the next line, or round to the far end of the buffer
        // if `'wrapscan'` allows it.
        let last = buffer.b_ml.ml_line_count;
        current_pos.lnum += if dir == FORWARD { 1 } else { -1 };
        if !(1..=last).contains(&current_pos.lnum) {
            if !p_ws() {
                return None;
            }
            current_pos.lnum = if dir == FORWARD { 1 } else { last };
            looped_around = true;
        }
        current_pos.col = 0;
    }
}
