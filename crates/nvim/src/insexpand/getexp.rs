//! The collection driver: one pass over `'complete'`, one source at a time.
//!
//! [`ins_compl_get_exp`] is the loop — it asks [`process_next_cpt_value`] for
//! the next `'complete'` entry, calls the `get_next_*_completion` function
//! that entry names, and keeps going until it has enough matches or runs out
//! of sources.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::Expanded;
use crate::path::ExpandFlags;
use crate::types::{FAIL, IOSIZE, NUL, OK, ShmFlag};
use crate::winlayer::{Buf, buffers};

/// In large buffers a timeout can miss nearby matches, so the search starts
/// this many lines above the cursor.
const LOOKBACK_LINE_COUNT: linenr_T = 1000;

/// The running completion's own copy of `'complete'`, and how far the scan
/// through it has got.
///
/// Upstream keeps two fields for this: `st.e_cpt_copy`, the `xstrdup`ed
/// copy, and `st.e_cpt`, a bare `char *` walking it. The copy is freed at
/// the *start* of the next completion, which leaves it the one allocation
/// in this family with no owner, and leaves the cursor free to outlive the
/// bytes it points into.
///
/// Here the buffer is owned and the cursor is a byte offset into it, so a
/// fresh copy frees the old one in the same step -- at exactly the point
/// upstream's `xfree` ran -- and no cursor survives the buffer.
///
/// The copy is taken at all because `'complete'` belongs to a buffer, and a
/// completion runs user code that can wipe that buffer out.
pub(crate) struct CptScan {
    /// The copy, or null before the first completion. Never reallocated:
    /// `st.dict` borrows these bytes for a `k`/`s` entry's dictionary name,
    /// and reads them after the cursor has moved on.
    option: *mut c_char,
    /// Where the entry being scanned starts, as a byte offset into
    /// `option`.
    cursor: usize,
}

impl Drop for CptScan {
    fn drop(&mut self) {
        // SAFETY: this value's own `xstrdup`, or null, which `xfree` takes.
        unsafe { xfree(self.option.cast::<c_void>()) };
    }
}

impl CptScan {
    /// No copy taken yet: what C's zeroed `static` starts out as.
    pub(crate) const EMPTY: CptScan = CptScan {
        option: ptr::null_mut(),
        cursor: 0,
    };

    /// Drop the copy in hand and take a fresh one of `option`, positioned on
    /// its first entry. This is upstream's `xfree`/`xstrdup` pair at the
    /// start of a completion.
    ///
    /// # Safety
    /// `option` is a NUL-terminated string.
    pub(crate) unsafe fn restart(&mut self, option: *const c_char) {
        // The old copy goes first, where upstream's `xfree` was, rather than
        // after the new one has been allocated.
        *self = Self::EMPTY;
        // SAFETY: the caller's contract.
        let copy = unsafe { xstrdup(option) };
        // SAFETY: `copy` is this value's own NUL-terminated string, and
        // `strip_caret_numbers_in_place` only ever shortens it in place.
        unsafe { strip_caret_numbers_in_place(copy) };
        *self = CptScan {
            option: copy,
            cursor: 0,
        };
    }

    /// What is left of `'complete'`: the `char *` the readers, and the
    /// dictionary name `st.dict` keeps, want.
    pub(crate) fn rest(&self) -> *mut c_char {
        // Address arithmetic only -- nothing is read until `at`, and the
        // empty scan is a null pointer with a zero cursor.
        self.option.wrapping_add(self.cursor)
    }

    /// The byte the scan is on, `NUL` once `'complete'` is used up.
    pub(crate) fn at(&self) -> c_char {
        // SAFETY: the cursor never passes the copy's terminator, and a scan
        // is only read once `restart` has given it a buffer -- as upstream
        // reads `*st->e_cpt` without a null check.
        unsafe { *self.rest() }
    }

    /// Step over one byte: C's `*++st->e_cpt` reaching the name after a
    /// `k`/`s` flag.
    pub(crate) fn bump(&mut self) {
        self.cursor += 1;
    }

    /// Step over what separates two entries -- `'complete'` allows spaces
    /// after its commas.
    pub(crate) fn skip_separators(&mut self) {
        while self.at() as c_int == ',' as c_int || self.at() as c_int == ' ' as c_int {
            self.bump();
        }
    }

    /// Step to the entry after this one, copying the one just left into
    /// `buf` -- C's `copy_option_part(&st->e_cpt, ...)`.
    ///
    /// # Safety
    /// `buf` has `len` writable bytes.
    pub(crate) unsafe fn next_entry(&mut self, buf: *mut c_char, len: size_t) {
        let mut p = self.rest();
        // SAFETY: the caller's contract for `buf`; `p` walks this scan's own
        // copy, which `copy_option_part` only reads.
        unsafe { copy_option_part(&raw mut p, buf, len, c",".as_ptr().cast_mut()) };
        // `copy_option_part` only moves the pointer forward, and stops at the
        // terminator, so this stays within the copy.
        self.cursor = p.addr() - self.option.addr();
    }
}

/// Thesaurus completion goes through a function rather than a word list:
/// `'thesaurusfunc'` is set.
pub(crate) unsafe fn thesaurus_func_complete(type_0: c_int) -> bool {
    type_0 == CTRL_X_THESAURUS
        && (unsafe { *cur_buf().b_p_tsrfu } as c_int != NUL
            || unsafe { *p_tsrfu.get() } as c_int != NUL)
}

/// Is there another `'complete'` entry after `cpt`, so the source index should
/// move on?
pub(crate) unsafe fn may_advance_cpt_index(cpt: *const c_char) -> bool {
    if cpt_sources().index() == -1 {
        return false;
    }
    let mut p = cpt;
    while unsafe { *p } as c_int == ',' as c_int || unsafe { *p } as c_int == ' ' as c_int {
        p = unsafe { p.offset(1) };
    }
    unsafe { *p as c_int != NUL }
}

/// Get the next entry from `'complete'` (`st.e_cpt`) and set up `st` for it.
///
/// Writes the CTRL-X mode the entry stands for to `compl_type_arg` and whether
/// the source index should advance to `advance_cpt_idx`. Returns
/// `INS_COMPL_CPT_OK` when the entry is ready to collect from,
/// `INS_COMPL_CPT_CONT` to skip it, `INS_COMPL_CPT_END` when `'complete'` is
/// exhausted.
pub(crate) unsafe fn process_next_cpt_value(
    st: *mut ins_compl_next_state_T,
    compl_type_arg: *mut c_int,
    start_match_pos: *mut pos_T,
    fuzzy_collect: bool,
    advance_cpt_idx: *mut bool,
) -> c_int {
    // The progress message, and the throwaway `copy_option_part` writes
    // into to step over the entry. Upstream shares `IObuff` for both.
    let mut scratch = [0 as c_char; IOSIZE as usize];
    let mut compl_type = -1;
    let mut status = INS_COMPL_CPT_OK;
    let skip_source = compl_autocomplete.get() && compl_from_nonkeyword.get();

    unsafe { (*st).found_all = false };
    unsafe { *advance_cpt_idx = false };

    unsafe { (*st).cpt.skip_separators() };

    'done: {
        if unsafe { (*st).cpt.at() } as c_int == '.' as c_int
            && !cur_buf().b_scanned
            && !skip_source
            && !compl_time_slice_expired.get()
        {
            unsafe { (*st).ins_buf = curbuf.get() };
            unsafe { (*st).first_match_pos = *start_match_pos };
            // Move the cursor back one character so that CTRL-N can match
            // the word immediately after the cursor.
            if ctrl_x_mode_normal()
                && !fuzzy_collect
                && unsafe { dec(&mut (*st).first_match_pos) } < 0
            {
                // Move to after the last character in the buffer, so that
                // a word at the start of it is found correctly.
                unsafe { (*st).first_match_pos.lnum = (*(*st).ins_buf).b_ml.ml_line_count };
                unsafe { (*st).first_match_pos.col = ml_get_len((*st).first_match_pos.lnum) };
            }
            unsafe { (*st).last_match_pos = (*st).first_match_pos };
            compl_type = 0;
            // Remember the first match, so the loop stops when the search
            // wraps and comes back to it a second time.
            unsafe { (*st).set_match_pos = true };
        } else if !skip_source
            && !compl_time_slice_expired.get()
            && !unsafe { vim_strchr(c"buwU".as_ptr(), (*st).cpt.at() as uint8_t as c_int) }
                .is_null()
            && {
                unsafe {
                    (*st).ins_buf =
                        ins_compl_next_buf(Buf::new((*st).ins_buf), (*st).cpt.at() as c_int).raw()
                };
                unsafe { (*st).ins_buf != curbuf.get() }
            }
        {
            // Scan a buffer, but not the current one.
            if !unsafe { (*(*st).ins_buf).b_ml.ml_mfp }.is_null() {
                // Loaded buffer.
                compl_started.set(true);
                unsafe { (*st).first_match_pos.col = 0 };
                unsafe { (*st).last_match_pos.col = 0 };
                unsafe { (*st).first_match_pos.lnum = (*(*st).ins_buf).b_ml.ml_line_count + 1 };
                unsafe { (*st).last_match_pos.lnum = 0 };
                compl_type = 0;
            } else {
                // Unloaded buffer: scan it like a dictionary.
                unsafe { (*st).found_all = true };
                if unsafe { (*(*st).ins_buf).b_fname }.is_null() {
                    status = INS_COMPL_CPT_CONT;
                    break 'done;
                }
                compl_type = CTRL_X_DICTIONARY;
                unsafe { (*st).dict = (*(*st).ins_buf).b_fname };
                unsafe { (*st).dict_f = DICT_EXACT };
            }
            if !shortmess(ShmFlag::COMPLETIONSCAN) && !compl_autocomplete.get() {
                // SAFETY: `ins_buf` is the buffer being scanned.
                let buf = unsafe { Buf::new((*st).ins_buf) };
                let name = if buf.b_fname.is_null() {
                    // SAFETY: a live buffer; the special name is static.
                    unsafe { buf_spname(buf.raw()) }
                } else if buf.b_sfname.is_null() {
                    buf.b_fname
                } else {
                    buf.b_sfname
                };
                // SAFETY: a static NUL-terminated format.
                let fmt = unsafe { gettext(c"Scanning: %s".as_ptr()) };
                let (out, size) = (scratch.as_mut_ptr(), IOSIZE as size_t);
                // SAFETY: `out` addresses all `size` bytes and `%s` takes a
                // NUL-terminated string, which `name` is.
                unsafe { vim_snprintf(out, size, fmt, name) };
                // SAFETY: `vim_snprintf` NUL-terminated `out`.
                unsafe { scan_progress(out) };
            }
        } else if unsafe { (*st).cpt.at() } as c_int == NUL {
            status = INS_COMPL_CPT_END;
        } else {
            if ctrl_x_mode_line_or_eval() {
                // compl_type stays -1.
            } else if unsafe { (*st).cpt.at() } as c_int == 'F' as c_int
                || unsafe { (*st).cpt.at() } as c_int == 'o' as c_int
            {
                compl_type = CTRL_X_FUNCTION;
                unsafe {
                    (*st).func_cb =
                        get_callback_if_cpt_func((*st).cpt.rest(), cpt_sources().index())
                };
                if unsafe { (*st).func_cb }.is_null() {
                    compl_type = -1;
                }
            } else if !skip_source {
                let flag = unsafe { (*st).cpt.at() } as c_int;
                if flag == 'k' as c_int || flag == 's' as c_int {
                    compl_type = if flag == 'k' as c_int {
                        CTRL_X_DICTIONARY
                    } else {
                        CTRL_X_THESAURUS
                    };
                    // C's `*++st->e_cpt`: a name may follow the flag.
                    unsafe { (*st).cpt.bump() };
                    if unsafe { (*st).cpt.at() } as c_int != ',' as c_int
                        && unsafe { (*st).cpt.at() } as c_int != NUL
                    {
                        unsafe { (*st).dict = (*st).cpt.rest() };
                        unsafe { (*st).dict_f = DICT_FIRST };
                    }
                } else if flag == 'i' as c_int {
                    compl_type = CTRL_X_PATH_PATTERNS;
                } else if flag == 'd' as c_int {
                    compl_type = CTRL_X_PATH_DEFINES;
                } else if flag == 'f' as c_int {
                    compl_type = CTRL_X_BUFNAMES;
                } else if flag == ']' as c_int || flag == 't' as c_int {
                    compl_type = CTRL_X_TAGS;
                    if !shortmess(ShmFlag::COMPLETIONSCAN) && !compl_autocomplete.get() {
                        // SAFETY: a static NUL-terminated message.
                        let text = unsafe { gettext(c"Scanning tags.".as_ptr()) };
                        let (out, size) = (scratch.as_mut_ptr(), IOSIZE as size_t);
                        // SAFETY: `out` addresses all `size` bytes.
                        unsafe { vim_snprintf(out, size, c"%s".as_ptr(), text) };
                        // SAFETY: `vim_snprintf` NUL-terminated `out`.
                        unsafe { scan_progress(out) };
                    }
                }
            }

            // In any case the scan advances to the next entry.
            unsafe { (*st).cpt.next_entry(scratch.as_mut_ptr(), IOSIZE as size_t) };
            unsafe { *advance_cpt_idx = may_advance_cpt_index((*st).cpt.rest()) };

            unsafe { (*st).found_all = true };
            if compl_type == -1 {
                status = INS_COMPL_CPT_CONT;
            }
        }
    }

    unsafe { *compl_type_arg = compl_type };
    status
}

/// Identifiers (`i`) or defines (`d`) from included files.
pub(crate) unsafe fn get_next_include_file_completion(compl_type: c_int) {
    let pattern = compl_pattern().value();
    let what = if compl_type == CTRL_X_PATH_DEFINES && compl_cont_status.get() & CONT_SOL == 0 {
        FIND_DEFINE
    } else {
        FIND_ANY
    };
    let (pat, len) = (pattern.data(), pattern.len());
    let dir = compl_direction.get();
    let end = MAXLNUM as linenr_T;
    let auto = compl_autocomplete.get();
    // SAFETY: `pat` is `len` readable bytes of the running completion's
    // pattern, and the search runs over the current buffer's include path.
    unsafe {
        let action = ACTION_EXPAND;
        find_pattern_in_path(
            pat, dir, len, false, false, what, 1, action, 1, end, false, auto,
        )
    };
}

/// Words from `'dictionary'` (`k`) or `'thesaurus'` (`s`) files.
pub(crate) unsafe fn get_next_dict_tsr_completion(
    compl_type: c_int,
    dict: *mut c_char,
    dict_f: c_int,
) {
    let pattern = compl_pattern().data();
    if unsafe { thesaurus_func_complete(compl_type) } {
        unsafe { expand_by_function(compl_type, pattern, ptr::null_mut()) };
        return;
    }
    let files = if !dict.is_null() {
        dict
    } else if compl_type == CTRL_X_THESAURUS {
        if unsafe { *cur_buf().b_p_tsr } as c_int == NUL {
            p_tsr.get()
        } else {
            cur_buf().b_p_tsr
        }
    } else if unsafe { *cur_buf().b_p_dict } as c_int == NUL {
        p_dict.get()
    } else {
        cur_buf().b_p_dict
    };
    let flags = if dict.is_null() { 0 } else { dict_f };
    let thesaurus = compl_type == CTRL_X_THESAURUS;
    // SAFETY: `files` is a NUL-terminated option-style list and `pattern`
    // the running completion's NUL-terminated pattern.
    unsafe { ins_compl_dictionaries(files, pattern, flags, thesaurus) };
}

/// Tag names matching `compl_pattern`, up to `TAG_MANY` of them.
pub(crate) unsafe fn get_next_tag_completion() {
    // Set `p_ic` from `p_ic`, `p_scs` and the pattern, for `find_tags`.
    let save_p_ic = p_ic.get();
    p_ic.set(unsafe { ignorecase(compl_pattern().data()) });
    g_tag_at_cursor.set(true);

    let mut matches: *mut *mut c_char = ptr::null_mut();
    let mut num_matches = 0;
    // Bounded to TAG_MANY, which is what stops an empty pattern finding
    // an enormous number of matches.
    let mut flags = TAG_REGEXP | TAG_NAMES | TAG_NOIC | TAG_INS_COMP;
    if ctrl_x_mode_not_default() {
        flags |= TAG_VERBOSE;
    }
    let (pat, fname) = (compl_pattern().data(), cur_buf().b_ffname);
    let (count, out) = (&raw mut num_matches, &raw mut matches);
    // SAFETY: `pat` is the running completion's NUL-terminated pattern, and
    // the two out-parameters are this frame's own locals.
    let found = unsafe { find_tags(pat, count, out, flags, TAG_MANY, fname) };
    if found == OK && num_matches > 0 {
        unsafe { ins_compl_add_matches(num_matches, matches, p_ic.get()) };
    }

    g_tag_at_cursor.set(false);
    p_ic.set(save_p_ic);
}

/// File names matching `compl_pattern`, fuzzily when `'completeopt'` asks.
pub(crate) unsafe fn get_next_filename_completion() {
    let mut matches: *mut *mut c_char = ptr::null_mut();
    let mut num_matches = 0;
    let mut leader = ins_compl_leader();
    let mut leader_len = ins_compl_leader_len();
    let mut in_fuzzy_collect = cot_fuzzy() && leader_len > 0;
    let need_collect_bests = in_fuzzy_collect && compl_get_longest.get();
    let mut max_score = 0;
    let mut dir = compl_direction.get();

    // Fuzzy matching is done over the whole directory, so the pattern is
    // widened to a wildcard and the leader keeps only the last component.
    if in_fuzzy_collect {
        let last_sep = unsafe { strrchr(leader, PATHSEP) };
        if last_sep.is_null() {
            // No path separator: match everything in the current dir.
            compl_pattern().replace(unsafe { cbuf_to_string(c"*".as_ptr(), 1) });
        } else if unsafe { *last_sep.offset(1) } as c_int == NUL {
            // The leader ends in a separator: nothing to fuzzy-match.
            in_fuzzy_collect = false;
        } else {
            let path_len = unsafe { last_sep.offset_from(leader) } as size_t + 1;
            let path_with_wildcard = unsafe { xmalloc(path_len + 2) } as *mut c_char;
            unsafe {
                vim_snprintf(
                    path_with_wildcard,
                    path_len + 2,
                    c"%*.*s*".as_ptr(),
                    path_len as c_int,
                    path_len as c_int,
                    leader,
                )
            };
            compl_pattern().replace(String_0::from_raw_parts(path_with_wildcard, path_len + 1));
            // Restrict the leader to the file-name part.
            leader = unsafe { last_sep.offset(1) };
            leader_len -= path_len;
        }
    }

    // `expand_wildcards` takes an *array* of patterns, hence a `char **`,
    // and only ever reads through it — so a local copy of the two words
    // gives it the address it wants without handing it the global.
    let mut pattern = compl_pattern().value();
    if unsafe {
        expand_wildcards(
            1,
            pattern.data_mut(),
            &raw mut num_matches,
            &raw mut matches,
            ExpandFlags::FILE | ExpandFlags::DIR | ExpandFlags::ADDSLASH | ExpandFlags::SILENT,
        )
    } != OK
    {
        return;
    }

    // Expand `~/` so the completion shows the shortened name.
    unsafe { tilde_replace(compl_pattern().data(), num_matches, matches) };

    if in_fuzzy_collect {
        let mut fuzzy_indices = GARRAY_T_INIT;
        unsafe { ga_init(&raw mut fuzzy_indices, size_of::<c_int>() as c_int, 10) };
        compl_fuzzy_scores
            .set(unsafe { xmalloc(size_of::<c_int>() * num_matches as size_t) } as *mut c_int);

        for i in 0..num_matches {
            let score = unsafe { fuzzy_match_str(*matches.offset(i as isize), leader) };
            if score != FUZZY_SCORE_NONE {
                unsafe { ga_grow(&raw mut fuzzy_indices, 1) };
                unsafe {
                    *(fuzzy_indices.ga_data as *mut c_int).offset(fuzzy_indices.ga_len as isize) = i
                };
                fuzzy_indices.ga_len += 1;
                unsafe { *compl_fuzzy_scores.get().offset(i as isize) = score };
            }
        }

        if fuzzy_indices.ga_len > 0 {
            let indices = fuzzy_indices.ga_data as *mut c_int;
            unsafe {
                qsort(
                    indices.cast::<c_void>(),
                    fuzzy_indices.ga_len as size_t,
                    size_of::<c_int>(),
                    Some(compare_scores),
                )
            };
            for i in 0..fuzzy_indices.ga_len as isize {
                let idx = unsafe { *indices.offset(i) } as isize;
                let current_score = unsafe { *compl_fuzzy_scores.get().offset(idx) };
                if unsafe {
                    ins_compl_add(
                        *matches.offset(idx),
                        -1,
                        ptr::null_mut(),
                        ptr::null(),
                        false,
                        ptr::null_mut(),
                        dir,
                        CP_FAST
                            | if p_fic.get() != 0 || p_wic.get() != 0 {
                                CP_ICASE
                            } else {
                                0
                            },
                        false,
                        ptr::null(),
                        current_score,
                    )
                } == OK
                {
                    dir = FORWARD;
                }
                if need_collect_bests && (i == 0 || current_score == max_score) {
                    compl_num_bests.set(compl_num_bests.get() + 1);
                    max_score = current_score;
                }
            }
            unsafe { free_wild(num_matches, matches) };
        } else if leader_len > 0 {
            unsafe { free_wild(num_matches, matches) };
            num_matches = 0;
        }

        unsafe { xfree(compl_fuzzy_scores.get().cast::<c_void>()) };
        unsafe { ga_clear(&raw mut fuzzy_indices) };
        if compl_num_bests.get() > 0 && compl_get_longest.get() {
            unsafe { fuzzy_longest_match() };
        }
        return;
    }

    if num_matches > 0 {
        unsafe {
            ins_compl_add_matches(
                num_matches,
                matches,
                c_int::from(p_fic.get() != 0 || p_wic.get() != 0),
            )
        };
    }
}

/// Vim command-line completion (`CTRL-X CTRL-V`).
pub(crate) unsafe fn get_next_cmdline_completion() {
    let mut matches: *mut *mut c_char = ptr::null_mut();
    let mut num_matches = 0;
    let pattern = compl_pattern().value();
    if unsafe {
        expand_cmdline(
            compl_xp.ptr(),
            pattern.data(),
            pattern.len() as c_int,
            &raw mut num_matches,
            &raw mut matches,
        )
    } == Expanded::Ok
    {
        unsafe { ins_compl_add_matches(num_matches, matches, 0) };
    }
}

/// Spelling suggestions for the bad word at `lnum`.
pub(crate) unsafe fn get_next_spell_completion(lnum: linenr_T) {
    let mut matches: *mut *mut c_char = ptr::null_mut();
    let num_matches = unsafe { expand_spelling(lnum, compl_pattern().data(), &raw mut matches) };
    if num_matches > 0 {
        unsafe { ins_compl_add_matches(num_matches, matches, p_ic.get()) };
    } else {
        unsafe { xfree(matches.cast::<c_void>()) };
    }
}

/// Collect one source's worth of matches for `type_0`.
///
/// Returns true when a new match was found.
pub(crate) unsafe fn get_next_completion_match(
    type_0: c_int,
    st: *mut ins_compl_next_state_T,
    ini: *mut pos_T,
) -> bool {
    let mut found_new_match = FAIL;
    match type_0 {
        // No source: `process_next_cpt_value` rejected this entry.
        -1 => {}
        CTRL_X_PATH_PATTERNS | CTRL_X_PATH_DEFINES => {
            unsafe { get_next_include_file_completion(type_0) };
        }
        CTRL_X_DICTIONARY | CTRL_X_THESAURUS => {
            unsafe { get_next_dict_tsr_completion(type_0, (*st).dict, (*st).dict_f) };
            unsafe { (*st).dict = ptr::null_mut() };
        }
        CTRL_X_TAGS => unsafe { get_next_tag_completion() },
        CTRL_X_FILES => unsafe { get_next_filename_completion() },
        CTRL_X_CMDLINE | CTRL_X_CMDLINE_CTRL_X => unsafe { get_next_cmdline_completion() },
        CTRL_X_FUNCTION => {
            if ctrl_x_mode_normal() {
                // Invoked by an `F`/`o` entry in 'complete'.
                unsafe { get_cpt_func_completion_matches((*st).func_cb) };
            } else {
                unsafe { expand_by_function(type_0, compl_pattern().data(), ptr::null_mut()) };
            }
        }
        CTRL_X_OMNI => {
            unsafe { expand_by_function(type_0, compl_pattern().data(), ptr::null_mut()) };
        }
        CTRL_X_SPELL => unsafe { get_next_spell_completion((*st).first_match_pos.lnum) },
        CTRL_X_BUFNAMES => unsafe { get_next_bufname_token() },
        CTRL_X_REGISTER => unsafe { get_register_completion() },
        // Normal CTRL-P/CTRL-N and CTRL-X CTRL-L.
        _ => {
            found_new_match = unsafe { get_next_default_completion(st, ini) };
            if found_new_match == FAIL && unsafe { (*st).ins_buf } == curbuf.get() {
                unsafe { (*st).found_all = true };
            }
        }
    }
    if type_0 != 0 && compl_curr_match.get() != compl_old_match.get() {
        found_new_match = OK;
    }
    found_new_match != 0
}

/// Start the per-source time slice, where a timeout is configured at all.
pub(crate) fn compl_source_start_timer(source_idx: c_int) {
    if compl_autocomplete.get() || p_cto.get() > 0 {
        let now = os_hrtime();
        cpt_sources().update(source_idx, |source| source.compl_start_tv = now);
        compl_time_slice_expired.set(false);
    }
}

/// Collect the next expansions using `compl_pattern`, starting at `ini` and
/// running in `compl_direction`.
///
/// With `compl_started` false the search starts at that position, otherwise it
/// continues where the previous call stopped. May return before every match is
/// found; the answer is the total number of matches, or −1 while that is still
/// unknown. -- Acevedo
pub(crate) unsafe fn ins_compl_get_exp(ini: pos_T) -> c_int {
    // Upstream's function-scope `static ins_compl_next_state_T st`: the
    // scan is collected over many calls, so the state outlives each one.
    // The pointer is taken once, here, because `st.cur_match_pos` points
    // *into* `st` — the address has to stay put for the whole call — and
    // because `process_next_cpt_value` and `get_next_completion_match`
    // want it by pointer anyway.
    static st_cell: GlobalCell<ins_compl_next_state_T> = GlobalCell::new(INS_COMPL_NEXT_STATE_INIT);
    static st_cleared: GlobalCell<bool> = GlobalCell::new(false);
    let st = st_cell.ptr();

    let mut found_new_match;
    let mut type_0 = ctrl_x_mode.get();
    let mut may_advance_cpt_idx = false;
    let mut start_pos = ini;

    debug_assert!(!curbuf.get().is_null());

    if !compl_started.get() {
        for mut buf in buffers() {
            buf.b_scanned = false;
        }
        if !st_cleared.get() {
            st_cell.set(INS_COMPL_NEXT_STATE_INIT);
            st_cleared.set(true);
        }
        unsafe { (*st).found_all = false };
        unsafe { (*st).ins_buf = curbuf.get() };
        // Copy 'complete', in case the buffer is wiped out.
        let option = if compl_cont_status.get() & CONT_LOCAL != 0 {
            c".".as_ptr()
        } else {
            cur_buf().b_p_cpt
        };
        // SAFETY: `st` is the scan state cell, and `option` is a
        // NUL-terminated option string.
        unsafe { (*st).cpt.restart(option) };

        if compl_autocomplete.get() && is_nearest_active() {
            start_pos.lnum = (start_pos.lnum - LOOKBACK_LINE_COUNT).max(1);
            start_pos.col = 0;
        }
        unsafe { (*st).first_match_pos = start_pos };
        unsafe { (*st).last_match_pos = start_pos };
    } else if unsafe { (*st).ins_buf } != curbuf.get() && !unsafe { buf_valid((*st).ins_buf) } {
        // In case the buffer was wiped out.
        unsafe { (*st).ins_buf = curbuf.get() };
    }
    debug_assert!(!unsafe { (*st).ins_buf }.is_null());

    // Remember the last current match.
    compl_old_match.set(compl_curr_match.get());
    // SAFETY: the address of one of the scan state's own two position
    // fields, taken from the raw pointer rather than through a borrow -- the
    // state is written through `st` again below, and a `&mut` to it would
    // invalidate this pointer.
    unsafe {
        (*st).cur_match_pos = if compl_dir_forward() {
            &raw mut (*st).last_match_pos
        } else {
            &raw mut (*st).first_match_pos
        }
    };

    let normal_mode_strict = ctrl_x_mode_normal()
        && !ctrl_x_mode_line_or_eval()
        && compl_cont_status.get() & CONT_LOCAL == 0
        && !cpt_sources().is_unset();
    if normal_mode_strict {
        cpt_sources().set_index(0);
        if compl_autocomplete.get() || p_cto.get() > 0 {
            compl_source_start_timer(0);
            compl_time_slice_expired.set(false);
            compl_timeout_ms.set(if compl_autocomplete.get() {
                (COMPL_INITIAL_TIMEOUT_MS as OptInt).max(p_act.get()) as uint64_t
            } else {
                p_cto.get() as uint64_t
            });
        }
    }

    // For CTRL-N/CTRL-P, loop over all the flags/windows/buffers in
    // 'complete'.
    loop {
        found_new_match = FAIL;
        unsafe { (*st).set_match_pos = false };

        // For CTRL-N/CTRL-P pick a new entry from `e_cpt` when
        // `compl_started` is off, or when `found_all` says this entry is
        // done. For CTRL-X CTRL-L only the entries that look in loaded
        // buffers are used.
        if (ctrl_x_mode_normal() || ctrl_x_mode_line_or_eval())
            && (!compl_started.get() || unsafe { (*st).found_all })
        {
            let status = unsafe {
                process_next_cpt_value(
                    st,
                    &raw mut type_0,
                    &raw mut start_pos,
                    cot_fuzzy(),
                    &raw mut may_advance_cpt_idx,
                )
            };
            if status == INS_COMPL_CPT_END {
                break;
            }
            if status == INS_COMPL_CPT_CONT {
                if may_advance_cpt_idx {
                    if unsafe { advance_cpt_sources_index_safe() } == 0 {
                        break;
                    }
                    compl_source_start_timer(cpt_sources().index());
                }
                continue;
            }
        }

        // LSP servers may sporadically take over a second to respond (for
        // instance while loading modules) but other sources may already
        // have matches, so keyword completion uses a short timeout and
        // non-keyword completion — where only function sources are active
        // — a longer one.
        let mut compl_timeout_save = 0;
        if normal_mode_strict
            && type_0 == CTRL_X_FUNCTION
            && (compl_autocomplete.get() || p_cto.get() > 0)
        {
            compl_timeout_save = compl_timeout_ms.get();
            compl_timeout_ms.set(if compl_from_nonkeyword.get() {
                COMPL_FUNC_TIMEOUT_NON_KW_MS as uint64_t
            } else {
                COMPL_FUNC_TIMEOUT_MS as uint64_t
            });
        }

        found_new_match =
            c_int::from(unsafe { get_next_completion_match(type_0, st, &raw mut start_pos) });

        // If complete() was called then `compl_pattern` has been reset and
        // the rest of this cannot work; bail out.
        if compl_pattern().is_unset() {
            break;
        }

        if may_advance_cpt_idx {
            if unsafe { advance_cpt_sources_index_safe() } == 0 {
                break;
            }
            compl_source_start_timer(cpt_sources().index());
        }

        // Break out for the specialised modes — 'complete' is only for the
        // generic CTRL_X_NORMAL — or when a new match has been found.
        if (ctrl_x_mode_not_default() && !ctrl_x_mode_line_or_eval()) || found_new_match != FAIL {
            if got_int.get() {
                break;
            }
            // Fill the popup menu as soon as possible.
            if type_0 != -1 {
                unsafe { ins_compl_check_keys(0, false) };
            }
            if (ctrl_x_mode_not_default() && !ctrl_x_mode_line_or_eval()) || compl_interrupted.get()
            {
                break;
            }
            compl_started.set(!compl_time_slice_expired.get());
        } else {
            // Mark a buffer scanned when it has been scanned completely.
            if unsafe { buf_valid((*st).ins_buf) }
                && (type_0 == 0 || type_0 == CTRL_X_PATH_PATTERNS)
            {
                debug_assert!(!unsafe { (*st).ins_buf }.is_null());
                unsafe { (*(*st).ins_buf).b_scanned = true };
            }
            compl_started.set(false);
        }

        // Restore the timeout after collecting from a function source.
        // Re-tested rather than remembered: the source just run can be a
        // user function, and that can have changed either operand.
        if normal_mode_strict
            && type_0 == CTRL_X_FUNCTION
            && (compl_autocomplete.get() || p_cto.get() > 0)
        {
            compl_timeout_ms.set(compl_timeout_save);
        }

        // For CTRL-P completion, reset `compl_curr_match` to the head, to
        // avoid mixing matches from different sources.
        if !compl_dir_forward() {
            // Upstream dereferences `compl_curr_match` here without checking.
            let mut curr = curr_match().expect("a running completion has a current match");
            while let Some(prev) = curr.prev().filter(|prev| !prev.is_original()) {
                curr = prev;
            }
            compl_curr_match.set(curr.raw());
        }
    }

    cpt_sources().set_index(-1);
    compl_started.set(true);

    if (ctrl_x_mode_normal() || ctrl_x_mode_line_or_eval())
        && unsafe { (*st).cpt.at() } as c_int == NUL
    {
        // Got to the end of 'complete'.
        found_new_match = FAIL;
    }

    // Total number of matches; −1 while unknown.
    let mut match_count = -1;
    if found_new_match == FAIL || (ctrl_x_mode_not_default() && !ctrl_x_mode_line_or_eval()) {
        match_count = ins_compl_make_cyclic();
    }

    if cot_fuzzy() && compl_get_longest.get() && compl_num_bests.get() > 0 {
        unsafe { fuzzy_longest_match() };
    }

    if let Some(old) = old_match() {
        // If several matches were added (FORWARD), or the search failed
        // and the list has just been made cyclic, `compl_curr_match` has
        // to move to the next or previous entry, if any. -- Acevedo
        let next = if compl_dir_forward() {
            old.next()
        } else {
            old.prev()
        };
        compl_curr_match.set(next.unwrap_or(old).raw());
    }
    unsafe { may_trigger_modechanged() };

    if match_count > 0 && !ctrl_x_mode_spell() {
        if is_nearest_active() && !unsafe { ins_compl_has_preinsert() } {
            unsafe { sort_compl_match_list(Some(cp_compare_nearest)) };
        }
        if cot_fuzzy() && ins_compl_leader_len() > 0 {
            unsafe { ins_compl_fuzzy_sort() };
        }
    }

    match_count
}

/// Expire the current source's time slice, halving the budget each time so a
/// slow source cannot hold up the rest.
pub(crate) fn check_elapsed_time() {
    let start_tv = cpt_sources().current().compl_start_tv;
    let elapsed_ms = (os_hrtime() - start_tv) / 1_000_000;
    if elapsed_ms > compl_timeout_ms.get() {
        compl_time_slice_expired.set(true);
        if compl_timeout_ms.get() > COMPL_MIN_TIMEOUT_MS as uint64_t {
            compl_timeout_ms.set(compl_timeout_ms.get() / 2);
        }
    }
}

/// `msg_progress` for the scan, with the kind and state every caller here
/// passes.
///
/// # Safety
/// `msg` is a NUL-terminated string.
pub(crate) unsafe fn scan_progress(msg: *mut c_char) {
    let kind = c"completion".as_ptr().cast_mut();
    let state = c"running".as_ptr().cast_mut();
    // SAFETY: the caller's message, and two static kind/state names.
    unsafe { msg_progress(msg, kind, state, HLF_R, false, true) };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
