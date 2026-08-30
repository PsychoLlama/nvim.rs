//! Scanning buffers, dictionaries, thesauruses and registers for matches.
//!
//! [`ins_compl_dictionaries`] and [`ins_compl_files`] are the `'dictionary'`
//! and `'thesaurus'` file walk; [`get_next_default_completion`] is the
//! keyword search through the buffers `'complete'` names, driven by
//! [`ins_compl_next_buf`]; [`get_register_completion`] is the `CTRL-X
//! CTRL-R` source.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::guard::Suppress;
use crate::path::ExpandFlags;
use crate::types::{FAIL, Failed, IOSIZE, NUL, OK, ShmFlag};
use crate::winlayer::{Buf, Pos, Win, first_buffer, first_window};

/// Add every identifier matching `pat` in the `'dictionary'`-style list
/// `dict_start` to the completions.
///
/// `flags` is `DICT_FIRST` and/or `DICT_EXACT`; `thesaurus` selects thesaurus
/// completion.
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
        if !thesaurus && cur_win().w_onebuf_opt.wo_spell != 0 {
            dict = c"spell".as_ptr().cast_mut();
        } else {
            return;
        }
    }

    let mut buf = unsafe { xmalloc(LSIZE as size_t) }.cast::<c_char>();
    // So that we can leave through 'theend.
    let mut regmatch = regmatch_T {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };

    // If 'infercase' is set, don't use 'smartcase' here.
    let save_p_scs = p_scs.get();
    if cur_buf().b_p_inf != 0 {
        p_scs.set(0);
    }

    // C's `goto theend`, i.e. free and restore below without scanning.
    'theend: {
        // When invoked to match whole lines for CTRL-X CTRL-L adjust the
        // pattern to only match at the start of a line.  Otherwise just
        // match the pattern.  Also need to double backslashes.
        if ctrl_x_mode_line_or_eval() {
            let pat_esc = unsafe { vim_strsave_escaped(pat, c"\\".as_ptr()) };
            let len = unsafe { strlen(pat_esc) } + 10;
            let ptr = unsafe { xmalloc(len) }.cast::<c_char>();
            unsafe { vim_snprintf(ptr, len, c"^\\s*\\zs\\V%s".as_ptr(), pat_esc) };
            regmatch.regprog = unsafe { vim_regcomp(ptr, RE_MAGIC) };
            unsafe { xfree(pat_esc.cast::<c_void>()) };
            unsafe { xfree(ptr.cast::<c_void>()) };
        } else {
            regmatch.regprog =
                unsafe { vim_regcomp(pat, if magic_isset() { RE_MAGIC } else { 0 }) };
            if regmatch.regprog.is_null() {
                break 'theend;
            }
        }

        // Ignore case depends on 'ignorecase', 'smartcase' and "pat".
        regmatch.rm_ic = unsafe { ignorecase(pat) } != 0;
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
                if !thesaurus && unsafe { cstr::bytes_at(buf) == b"spell" } {
                    count = -1;
                } else {
                    // SAFETY: as above.
                    let backtick = !unsafe { vim_strchr(buf, '`' as c_int) }.is_null();
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
                // Avoid a warning for using "files" uninitialised.
                let (rm, direction) = (&raw mut regmatch, &raw mut dir);
                // SAFETY: `files` is `count` NUL-terminated names, `rm` the
                // compiled pattern and `buf` a scratch buffer of `LSIZE`.
                unsafe { ins_compl_files(count, files, thesaurus, flags, rm, buf, direction) };
                if flags != DICT_EXACT {
                    unsafe { free_wild(count, files) };
                }
            }
            if flags != 0 {
                break;
            }
        }
    }

    p_scs.set(save_p_scs);
    unsafe { vim_regfree(regmatch.regprog) };
    unsafe { xfree(buf.cast::<c_void>()) };
}

/// Add all the words in the line `*buf_arg` from the thesaurus file `fname`,
/// skipping the word at `skip_word`; answers OK on success.
pub(crate) unsafe fn thesaurus_add_words_in_line(
    fname: *mut c_char,
    buf_arg: *mut *mut c_char,
    dir: c_int,
    skip_word: *const c_char,
) -> c_int {
    let mut status = OK;

    // Add the other matches on the line.
    let mut ptr = unsafe { *buf_arg };
    while !got_int.get() {
        // Find the start of the next word, skipping white space and
        // punctuation.
        ptr = unsafe { find_word_start(ptr) };
        if unsafe { *ptr } as c_int == NUL || unsafe { *ptr } as c_int == NL {
            break;
        }
        let wstart = ptr;

        // Find the end of the word.  Japanese words may have characters in
        // different classes, so only separate words with single-byte
        // non-word characters.
        while unsafe { *ptr } as c_int != NUL {
            let l = unsafe { utfc_ptr2len(ptr) };
            if l < 2 && !unsafe { vim_iswordc(*ptr as u8 as c_int) } {
                break;
            }
            ptr = unsafe { ptr.offset(l as isize) };
        }

        // Add the word, skipping the regexp match.
        if wstart != skip_word.cast_mut() {
            // SAFETY: `wstart .. ptr` is one word of the line being read.
            status = unsafe { add_scanned_word(wstart, ptr, fname, dir, FUZZY_SCORE_NONE) };
            if status == FAIL {
                break;
            }
        }
    }

    unsafe { *buf_arg = ptr };
    status
}

/// Read `count` dictionary/thesaurus `files` and add the text matching
/// `regmatch`.
pub(crate) unsafe fn ins_compl_files(
    count: c_int,
    files: *mut *mut c_char,
    thesaurus: bool,
    flags: c_int,
    regmatch: *mut regmatch_T,
    buf: *mut c_char,
    dir: *mut Direction,
) {
    let mut progress = [0 as c_char; IOSIZE as usize];
    let leader = if cot_fuzzy() {
        ins_compl_leader()
    } else {
        ptr::null_mut()
    };
    let leader_len = if cot_fuzzy() {
        ins_compl_leader_len() as c_int
    } else {
        0
    };

    let mut i = 0;
    while i < count as isize && !got_int.get() && !ins_compl_interrupted() {
        let file = unsafe { *files.offset(i) };
        let fp = unsafe { os_fopen(file, c"r".as_ptr()) }; // open dictionary file
        let quiet = shortmess(ShmFlag::COMPLETIONSCAN);
        if flags != DICT_EXACT && !quiet && !compl_autocomplete.get() {
            let fmt = gettext(c"Scanning dictionary: %s");
            let (out, size) = (progress.as_mut_ptr(), IOSIZE as size_t);
            // SAFETY: `out` addresses all `size` bytes and `file` is a
            // NUL-terminated name.
            unsafe { vim_snprintf(out, size, fmt.as_ptr(), file) };
            // SAFETY: `vim_snprintf` NUL-terminated `out`.
            unsafe { scan_progress(out) };
        }

        if fp.is_null() {
            i += 1;
            continue;
        }

        // Read the dictionary file line by line, checking each for a match.
        while !got_int.get() && !ins_compl_interrupted() && !unsafe { vim_fgets(buf, LSIZE, fp) } {
            let mut ptr = buf;
            if cot_fuzzy() && leader_len > 0 {
                let line_end = unsafe { find_line_end(ptr) };
                while ptr < line_end {
                    let mut score = 0;
                    let mut len = 0;
                    let (at, out_len, out_score) = (&raw mut ptr, &raw mut len, &raw mut score);
                    let none = ptr::null_mut();
                    // SAFETY: `ptr` walks the line, `leader` is
                    // NUL-terminated and the out-parameters are this frame's.
                    if unsafe { fuzzy_match_str_in_line(at, leader, out_len, none, out_score) } {
                        let end_ptr = if ctrl_x_mode_line_or_eval() {
                            unsafe { find_line_end(ptr) }
                        } else {
                            unsafe { find_word_end(ptr) }
                        };
                        // SAFETY: `ptr .. end_ptr` is one word of the line.
                        let add_r = unsafe { add_scanned_word(ptr, end_ptr, file, *dir, score) };
                        if add_r == FAIL {
                            break;
                        }
                        ptr = end_ptr; // start from the next word
                        if compl_get_longest.get()
                            && ctrl_x_mode_normal()
                            && !unsafe { (*compl_first_match.get()).cp_next }.is_null()
                            && score == unsafe { (*(*compl_first_match.get()).cp_next).cp_score }
                        {
                            compl_num_bests.set(compl_num_bests.get() + 1);
                        }
                    }
                }
            } else if !regmatch.is_null() {
                while unsafe { vim_regexec(regmatch, buf, ptr.offset_from(buf) as colnr_T) } {
                    let start = unsafe { (*regmatch).startp[0] };
                    ptr = if ctrl_x_mode_line_or_eval() {
                        unsafe { find_line_end(start) }
                    } else {
                        unsafe { find_word_end(start) }
                    };
                    // SAFETY: `start .. ptr` is one word of the line.
                    let mut add_r =
                        unsafe { add_scanned_word(start, ptr, file, *dir, FUZZY_SCORE_NONE) };
                    if thesaurus {
                        // For a thesaurus, add all the words in the line.
                        ptr = buf;
                        add_r =
                            unsafe { thesaurus_add_words_in_line(file, &raw mut ptr, *dir, start) };
                    }
                    if add_r == OK {
                        // If dir was BACKWARD then honour it just once.
                        unsafe { *dir = FORWARD };
                    } else if add_r == FAIL {
                        break;
                    }
                    // Avoid an expensive call to vim_regexec() at the end
                    // of the line.
                    if unsafe { *ptr } as c_int == '\n' as c_int || got_int.get() {
                        break;
                    }
                }
            }
            line_breakcheck();
            unsafe { ins_compl_check_keys(50, false) };
        }
        unsafe { fclose(fp) };
        i += 1;
    }
}

/// The next window, loaded buffer or non-loaded buffer (depending on `flag`)
/// after `buf` that has not been scanned; `curbuf` when there is none.
///
/// `curbuf` is special: called with `buf == curbuf` this has to be the first
/// call for a given flag/expansion. -- Acevedo
///
/// Safe: [`Buf`] is the live buffer the walk starts from, and the window it
/// remembers between calls is vetted below rather than trusted.
pub(crate) fn ins_compl_next_buf(mut buf: Buf, flag: c_int) -> Buf {
    // This outlives the call, and a completion runs user functions and Lua in
    // between, so it stays a raw pointer that `win_valid` vets -- a `Win`
    // would be promising a liveness nothing here can keep.
    static wp: GlobalCell<*mut win_T> = GlobalCell::new(ptr::null_mut());

    if flag == 'w' as c_int {
        // Just windows.
        if buf.raw() == curbuf.get() || !win_valid(wp.get()) {
            // First call for this flag/expansion, or the window was closed.
            wp.set(curwin.get());
        }
        debug_assert!(!wp.get().is_null());
        // SAFETY: `wp` is `curwin` or a window `win_valid` just vouched for,
        // and from there the editor's own window list, which is live.
        loop {
            // Move to the next window, wrapping to the first at the end.
            let cur = unsafe { Win::new(wp.get()) };
            let next = cur.next().or_else(first_window);
            wp.set(next.map_or(::core::ptr::null_mut(), Win::raw));
            // Stop if we're back at the start, or found an unscanned
            // buffer in a focusable window.
            if wp.get() == curwin.get()
                || (!unsafe { (*(*wp.get()).w_buffer).b_scanned }
                    && unsafe { (*wp.get()).w_config.focusable })
            {
                break;
            }
        }
        buf = unsafe { Buf::new((*wp.get()).w_buffer) };
    } else {
        // 'b' (just loaded buffers), 'u' (just non-loaded buffers) or 'U'
        // (unlisted buffers).  When completing whole lines skip unloaded
        // buffers.
        loop {
            // Move to the next buffer, wrapping to the first at the end.
            buf = match buf.next() {
                Some(next) => next,
                None => first_buffer().expect("the editor always has a buffer"),
            };
            // Stop if we're back at the start buffer.
            if buf.raw() == curbuf.get() {
                break;
            }
            let skip_buffer = if flag == 'U' as c_int {
                buf.b_p_bl != 0
            } else {
                buf.b_p_bl == 0 || buf.b_ml.ml_mfp.is_null() != (flag == 'u' as c_int)
            };
            // Stop if we found a buffer that matches our criteria.
            if !skip_buffer && !buf.b_scanned {
                break;
            }
        }
    }
    buf
}

/// The next word or line from `ins_buf` at `cur_match_pos`, with its length in
/// `match_len`; `cont_s_ipos` says the next `CTRL-X <>` sets the initial
/// position.
pub(crate) unsafe fn ins_compl_get_next_word_or_line(
    ins_buf: Buf,
    cur_match_pos: Pos,
    match_len: *mut c_int,
    cont_s_ipos: *mut bool,
    out: &mut [c_char; IOSIZE as usize],
) -> *mut c_char {
    // SAFETY: the caller's two out-parameters are its own locals.
    unsafe { *match_len = 0 };
    let (lnum, col) = (cur_match_pos.lnum, cur_match_pos.col);
    // SAFETY: `cur_match_pos` is a position in `ins_buf`, which the caller
    // has promised is live.
    let (line, line_len) = unsafe {
        (
            ml_get_buf(ins_buf.raw(), lnum),
            ml_get_buf_len(ins_buf.raw(), lnum),
        )
    };
    // SAFETY: `col` is inside the line.
    let mut ptr = unsafe { line.offset(col as isize) };
    let mut len = line_len - col;
    let iobuff = out.as_mut_ptr();

    if ctrl_x_mode_line_or_eval() {
        if compl_status_adding() {
            if lnum >= ins_buf.b_ml.ml_line_count {
                return ptr::null_mut();
            }
            // SAFETY: as above -- the line after this one exists.
            (ptr, len) = unsafe {
                (
                    ml_get_buf(ins_buf.raw(), lnum + 1),
                    ml_get_buf_len(ins_buf.raw(), lnum + 1),
                )
            };
            if p_paste.get() == 0 {
                let tmp_ptr = ptr;
                ptr = unsafe { skipwhite(tmp_ptr) };
                len -= unsafe { ptr.offset_from(tmp_ptr) } as c_int;
            }
        }
    } else {
        let mut tmp_ptr = ptr;
        if compl_status_adding() && compl_length.get() <= len {
            tmp_ptr = unsafe { tmp_ptr.offset(compl_length.get() as isize) };
            // Skip if already inside a word.
            if unsafe { vim_iswordp(tmp_ptr) } {
                return ptr::null_mut();
            }
            // Find the start of the next word.
            tmp_ptr = unsafe { find_word_start(tmp_ptr) };
        }
        // Find the end of this word.
        tmp_ptr = unsafe { find_word_end(tmp_ptr) };
        len = unsafe { tmp_ptr.offset_from(ptr) } as c_int;

        if compl_status_adding() && len == compl_length.get() {
            if lnum < ins_buf.b_ml.ml_line_count {
                // Try the next line, if any: the new word will be "joined"
                // as if the normal command "J" was used.  IOSIZE is always
                // greater than compl_length, so the strncpy always works
                // -- Acevedo
                unsafe { strncpy(iobuff, ptr, len as size_t) };
                // SAFETY: as above -- the line after this one exists.
                ptr = unsafe { skipwhite(ml_get_buf(ins_buf.raw(), lnum + 1)) };
                // Find the start and then the end of the next word.
                tmp_ptr = unsafe { find_word_end(find_word_start(ptr)) };
                if tmp_ptr > ptr {
                    if unsafe { *ptr } as c_int != ')' as c_int
                        && unsafe { *iobuff.offset((len - 1) as isize) } as c_int != TAB
                    {
                        if unsafe { *iobuff.offset((len - 1) as isize) } as c_int != ' ' as c_int {
                            unsafe { *iobuff.offset(len as isize) = ' ' as c_char };
                            len += 1;
                        }
                        // The joined line =~ "\k.* ", thus len >= 2.
                        if p_js.get() != 0
                            && matches!(
                                unsafe { *iobuff.offset((len - 2) as isize) } as u8,
                                b'.' | b'?' | b'!'
                            )
                        {
                            unsafe { *iobuff.offset(len as isize) = ' ' as c_char };
                            len += 1;
                        }
                    }
                    // Copy as much as possible of the new word.
                    if unsafe { tmp_ptr.offset_from(ptr) } >= (IOSIZE - len) as isize {
                        tmp_ptr = unsafe { ptr.offset((IOSIZE - len - 1) as isize) };
                    }
                    unsafe { xstrlcpy(iobuff.offset(len as isize), ptr, (IOSIZE - len) as size_t) };
                    len += unsafe { tmp_ptr.offset_from(ptr) } as c_int;
                    unsafe { *cont_s_ipos = true };
                }
                unsafe { *iobuff.offset(len as isize) = NUL as c_char };
                ptr = iobuff;
            }
            if len == compl_length.get() {
                return ptr::null_mut();
            }
        }
    }

    unsafe { *match_len = len };
    ptr
}

/// The next set of words matching `compl_pattern` for default completion —
/// normal `^P`/`^N` and `^X^L`.
///
/// Searches `st->ins_buf` from `start_pos` in the `compl_direction` direction;
/// with `st->set_match_pos` set, `st->first_match_pos` and `st->last_match_pos`
/// are set too. Answers `Ok` if a new match was found, otherwise `Err`.
pub(crate) unsafe fn get_next_default_completion(
    st: *mut ins_compl_next_state_T,
    start_pos: *mut pos_T,
) -> Result<(), Failed> {
    // Where a joined `CTRL-X CTRL-L` line is assembled; upstream shares
    // `IObuff` for it, which the message machinery also writes.
    let mut word = [0 as c_char; IOSIZE as usize];
    let mut ptr: *mut c_char = ptr::null_mut();
    let mut len = 0;
    let in_fuzzy_collect = !compl_status_adding() && cot_fuzzy() && compl_length.get() > 0;
    let leader = ins_compl_leader();
    let mut score = FUZZY_SCORE_NONE;
    // SAFETY: `st` is the caller's live scan state; `ins_buf` is the buffer
    // it is scanning and `cur_match_pos` addresses one of the state's own two
    // position fields. Neither changes while this scan runs.
    let (ins_buf, match_pos, start) = unsafe {
        (
            Buf::new((*st).ins_buf),
            Pos::new((*st).cur_match_pos),
            Pos::new(start_pos),
        )
    };
    let in_curbuf = ins_buf.raw() == curbuf.get();

    // If 'infercase' is set, don't use 'smartcase' here.
    let save_p_scs = p_scs.get();
    debug_assert!(!ins_buf.raw().is_null());
    if ins_buf.b_p_inf != 0 {
        p_scs.set(0);
    }

    // Buffers other than curbuf are scanned from the beginning or the end
    // but never from the middle, thus setting nowrapscan in these buffers
    // is a good idea; on the other hand, we always set wrapscan for curbuf
    // to avoid missing matches -- Acevedo, Webb
    let save_p_ws = p_ws.get();
    if !in_curbuf {
        p_ws.set(0);
    } else if unsafe { (*st).cpt.at() } as c_int == '.' as c_int {
        p_ws.set(1);
    }

    let mut looped_around = false;
    let mut found_new_match;
    loop {
        let mut cont_s_ipos = false;

        // Don't want messages for wrapscan.
        let silenced = Suppress::messages();
        if in_fuzzy_collect {
            let (buf, at, dir) = (ins_buf.raw(), match_pos.raw(), compl_direction.get());
            // SAFETY: `at` is a position in `buf` and `leader` is
            // NUL-terminated; `start_pos` is the caller's own position.
            let hit = unsafe { search_for_fuzzy_match(buf, at, leader, dir, start_pos) };
            found_new_match = Err(Failed);
            if let Some(hit) = hit {
                (ptr, len) = (hit.ptr, hit.len);
                score = hit.score.unwrap_or(score);
                found_new_match = Ok(());
            }
        } else if ctrl_x_mode_whole_line()
            || ctrl_x_mode_eval()
            || compl_cont_status.get() & CONT_SOL != 0
        {
            // ctrl_x_mode_line_or_eval(), or a word-wise search that has
            // added a word that was at the beginning of the line.
            let (at, dir) = (match_pos.raw(), compl_direction.get());
            let pat = compl_pattern().data();
            // SAFETY: `at` is a position in `ins_buf` and `pat` is the
            // running completion's NUL-terminated pattern.
            found_new_match = unsafe { search_for_exact_line(ins_buf, at, dir, pat) };
        } else {
            let found = unsafe {
                searchit(
                    None,
                    ins_buf,
                    match_pos.raw(),
                    ptr::null_mut(),
                    compl_direction.get(),
                    compl_pattern().data(),
                    compl_pattern().len(),
                    1,
                    SEARCH_KEEP + SEARCH_NFMSG,
                    RE_LAST,
                    ptr::null_mut(),
                )
            };
            found_new_match = if found == FAIL { Err(Failed) } else { Ok(()) };
        }
        drop(silenced);

        // SAFETY: `st` is the caller's live scan state, and `cur_match_pos`
        // addresses one of its two position fields.
        let (pos, set_match_pos, first, last) = unsafe {
            (
                *match_pos,
                (*st).set_match_pos,
                (*st).first_match_pos,
                (*st).last_match_pos,
            )
        };
        if !compl_started.get() || set_match_pos {
            // Set "compl_started" even on failure.
            compl_started.set(true);
            // SAFETY: as above.
            unsafe {
                (*st).first_match_pos = pos;
                (*st).last_match_pos = pos;
                (*st).set_match_pos = false;
            }
        } else if first.lnum == last.lnum && first.col == last.col {
            found_new_match = Err(Failed);
        } else {
            // Passing the previous match going forwards (or backwards) is
            // the wrap-around; the second time round there is nothing new.
            // SAFETY: `st` is the caller's live scan state.
            let prev = unsafe { (*st).prev_match_pos };
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
        unsafe { (*st).prev_match_pos = pos };
        if found_new_match.is_err() {
            break;
        }

        // When ADDING, the text before the cursor matches: skip it.
        if compl_status_adding()
            && in_curbuf
            && start.lnum == match_pos.lnum
            && start.col == match_pos.col
        {
            continue;
        }

        if !in_fuzzy_collect {
            let (out_len, ipos) = (&raw mut len, &raw mut cont_s_ipos);
            // SAFETY: `match_pos` is a position in `ins_buf`, and the two
            // out-parameters are this frame's own locals.
            ptr = unsafe {
                ins_compl_get_next_word_or_line(ins_buf, match_pos, out_len, ipos, &mut word)
            };
        }
        if ptr.is_null()
            || (unsafe { ins_compl_has_preinsert() }
                && unsafe { cstr::eq(ptr, ins_compl_leader()) })
        {
            continue;
        }

        if is_nearest_active() && in_curbuf {
            score = (match_pos.lnum - cur_win().w_cursor.lnum) as c_int;
            score = score.abs();
        }

        let fname = if in_curbuf {
            ptr::null_mut()
        } else {
            ins_buf.b_sfname
        };
        let ic = p_ic.get() != 0;
        // SAFETY: `ptr` is `len` readable bytes of the match just found, and
        // `fname` is null or the scanned buffer's own name.
        let add_r = unsafe {
            ins_compl_add_infercase(ptr, len, ic, fname, kDirectionNotSet, cont_s_ipos, score)
        };
        if add_r != NOTDONE {
            if in_fuzzy_collect
                && score == unsafe { (*(*compl_first_match.get()).cp_next).cp_score }
            {
                compl_num_bests.set(compl_num_bests.get() + 1);
            }
            found_new_match = Ok(());
            break;
        }
    }

    p_scs.set(save_p_scs);
    p_ws.set(save_p_ws);
    found_new_match
}

/// Add completion matches from the contents of every usable register.
pub(crate) unsafe fn get_register_completion() {
    // Upstream's `!compl_orig_text.data || (p_ic ? STRNICMP : strncmp)(…)`:
    // a candidate counts when there is no original text to compare against,
    // or it starts with it.
    let starts_with_orig = |s: *mut c_char| {
        let orig = compl_orig_text().value();
        orig.data().is_null()
            || if p_ic.get() != 0 {
                unsafe { strncasecmp(s, orig.data(), orig.len()) == 0 }
            } else {
                unsafe { cstr::prefix_eq(s, orig.data(), orig.len()) }
            }
    };

    let mut dir = compl_direction.get();
    let adding_mode = compl_status_adding();

    for i in 0..NUM_REGISTERS {
        let regname = get_register_name(i);
        // Skip an invalid or black hole register.
        if !unsafe { valid_yank_reg(regname, false) } || regname == '_' as c_int {
            continue;
        }

        let reg = unsafe { copy_register(regname) };
        if unsafe { (*reg).y_array }.is_null() || unsafe { (*reg).y_size } == 0 {
            unsafe { free_register(reg) };
            unsafe { xfree(reg.cast::<c_void>()) };
            continue;
        }

        for j in 0..unsafe { (*reg).y_size } as isize {
            let str = unsafe { *(*reg).y_array.offset(j) }.data();
            if str.is_null() {
                continue;
            }

            if adding_mode {
                let str_len = unsafe { strlen(str) } as c_int;
                if str_len == 0 {
                    continue;
                }
                // SAFETY: `str` is the register line, `str_len` its length.
                let end = unsafe { str.offset(str_len as isize) };
                // SAFETY: as above -- a whole register line, no file name.
                let added = starts_with_orig(str)
                    && unsafe {
                        add_scanned_word(str, end, ptr::null_mut(), dir, FUZZY_SCORE_NONE)
                    } == OK;
                if added {
                    dir = FORWARD;
                }
            } else {
                // The safe end of the string, to avoid NUL byte issues.
                let str_end = unsafe { str.add(strlen(str)) };
                let mut p = str;
                while p < str_end && unsafe { *p } as c_int != NUL {
                    let old_p = p;
                    p = unsafe { find_word_start(p) };
                    if p >= str_end || unsafe { *p } as c_int == NUL {
                        break;
                    }

                    let mut word_end = unsafe { find_word_end(p) };
                    if word_end <= p {
                        word_end = unsafe { p.offset(utfc_ptr2len(p) as isize) };
                    }
                    if word_end > str_end {
                        word_end = str_end;
                    }

                    let len = unsafe { word_end.offset_from(p) } as c_int;
                    // SAFETY: `p .. word_end` is one word of the register
                    // line, and there is no file name.
                    let added = len > 0
                        && starts_with_orig(p)
                        && unsafe {
                            add_scanned_word(p, word_end, ptr::null_mut(), dir, FUZZY_SCORE_NONE)
                        } == OK;
                    if added {
                        dir = FORWARD;
                    }

                    p = word_end;
                    if p <= old_p {
                        p = unsafe { old_p.offset(utfc_ptr2len(old_p) as isize) };
                    }
                }
            }
        }

        unsafe { free_register(reg) };
        unsafe { xfree(reg.cast::<c_void>()) };
    }
}

/// [`ins_compl_add_infercase`] for the word `start .. end` of a scanned
/// line, with the flags every caller in this module passes.
///
/// # Safety
/// `start` and `end` bound one word of a live line, and `fname` is null or
/// a NUL-terminated file name.
unsafe fn add_scanned_word(
    start: *mut c_char,
    end: *mut c_char,
    fname: *mut c_char,
    dir: Direction,
    score: c_int,
) -> c_int {
    // SAFETY: the caller's promise -- `end` is inside the same line as
    // `start`, and the match is not re-anchoring the initial position.
    unsafe {
        let len = end.offset_from(start) as c_int;
        ins_compl_add_infercase(start, len, p_ic.get() != 0, fname, dir, false, score)
    }
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
