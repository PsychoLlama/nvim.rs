//! The wildcard key: one `<Tab>` press, from key to command line.
//!
//! [`nextwild`] is what the command-line key loop calls; it isolates the word
//! under the cursor, hands it to [`ExpandOne`] and puts the answer back.
//! [`ExpandOne`] owns the match array across presses — [`expand_one_start`]
//! fills it, [`next_or_prev_match`] cycles it and [`longest_common_match`]
//! computes the `'wildmode'`=longest answer.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// The `WILD_*` modes that move within an already expanded match list rather
/// than expanding a new one.
///
/// They pass no `str`/`orig` to [`ExpandOne`], leave the match array alone,
/// and skip the "..." busy message and the `cmdline_orig` save in
/// [`nextwild`].
const fn is_wild_navigate(mode: c_int) -> bool {
    matches!(
        mode,
        WILD_NEXT | WILD_PREV | WILD_PAGEUP | WILD_PAGEDOWN | WILD_PUM_WANT
    )
}

/// The expanded matches, as a slice.  Only call this where `xp_numfiles` is
/// known positive: it is -1 before anything has been expanded.
unsafe fn matches_of(xp: *const expand_T) -> &'static [*mut c_char] {
    unsafe {
        debug_assert!((*xp).xp_numfiles > 0);
        // `.max(0)`: -1 means "nothing expanded", and building a slice of
        // `usize::MAX` entries out of that would be instant UB where the C
        // merely read past the end.
        core::slice::from_raw_parts((*xp).xp_files, (*xp).xp_numfiles.max(0) as usize)
    }
}

/// Expand the word before the cursor on the command line.
///
/// Answers `FAIL` when this is not a context in which anything can be
/// completed, which tells the caller to pass the character through as a
/// normal character instead — that is what makes `:s/^I^D` work.  `OK` means
/// the key was consumed, even when there were no matches.
///
/// `mode` is one of the `WILD_*` modes, passed on to [`ExpandOne`]; `escape`
/// asks for the matches to be escaped for use on the command line.
pub(crate) unsafe fn nextwild(
    xp: *mut expand_T,
    mode: c_int,
    options: c_int,
    escape: bool,
) -> c_int {
    unsafe {
        let ccline = get_cmdline_info();
        let from_wildtrigger_func = options & WILD_FUNC_TRIGGER != 0;
        let wild_navigate = is_wild_navigate(mode);

        if (*xp).xp_numfiles == -1 {
            pre_incsearch_pos.set((*xp).xp_pre_incsearch_pos);
            if (*ccline).input_fn != 0 && (*ccline).xp_context == EXPAND_COMMANDS {
                // Expand commands typed in the input() function.
                set_cmd_context(xp, (*ccline).cmdbuff, (*ccline).cmdlen, (*ccline).cmdpos, 0);
            } else {
                may_expand_pattern.set(options & WILD_MAY_EXPAND_PATTERN != 0);
                set_expand_context(xp);
                may_expand_pattern.set(false);
            }
            if (*xp).xp_context == EXPAND_LUA {
                nlua_expand_pat(xp);
            }
            cmd_showtail.set(expand_showtail(xp));
        }

        match (*xp).xp_context {
            // Something illegal on the command line.
            EXPAND_UNSUCCESSFUL => {
                beep_flush();
                return OK;
            }
            // The caller can use the character as a normal char instead.
            EXPAND_NOTHING => return FAIL,
            _ => {}
        }

        // Where the pattern starts within the command line.  Held as an index
        // rather than a pointer because `realloc_cmdbuff` below can move the
        // buffer out from under `xp_pattern`.
        let at = (*xp).xp_pattern.offset_from((*ccline).cmdbuff) as c_int;
        debug_assert!((*ccline).cmdpos >= at);
        (*xp).xp_pattern_len = ((*ccline).cmdpos - at) as size_t;

        // Skip showing matches if the prefix is invalid during wildtrigger().
        if from_wildtrigger_func && (*xp).xp_context == EXPAND_COMMANDS && (*xp).xp_pattern_len == 0
        {
            return FAIL;
        }

        // If 'cmd_silent' is set don't show the dots, because the redrawcmd()
        // below won't remove them.
        if !cmd_silent.get()
            && !from_wildtrigger_func
            && !wild_navigate
            && !(ui_has(kUICmdline) || ui_has(kUIWildmenu))
        {
            msg_puts(c"...".as_ptr()); // show that we are busy
            ui_flush();
        }

        let mut p;
        if wild_navigate {
            // Get the next/previous match of an already expanded pattern.
            p = ExpandOne(xp, ptr::null_mut(), ptr::null_mut(), 0, mode);
        } else {
            let tmp = if cmdline_fuzzy_completion_supported(xp)
                || (*xp).xp_context == EXPAND_PATTERN_IN_BUF
            {
                // Don't modify the search string.
                xstrnsave((*xp).xp_pattern, (*xp).xp_pattern_len)
            } else {
                addstar((*xp).xp_pattern, (*xp).xp_pattern_len, (*xp).xp_context)
            };
            // Translate the string into a pattern and expand it.
            let use_options = options
                | WILD_HOME_REPLACE
                | WILD_ADD_SLASH
                | WILD_SILENT
                | if escape { WILD_ESCAPE } else { 0 }
                | if p_wic.get() != 0 { WILD_ICASE } else { 0 };
            p = ExpandOne(
                xp,
                tmp,
                xstrnsave((*ccline).cmdbuff.offset(at as isize), (*xp).xp_pattern_len),
                use_options,
                mode,
            );
            xfree(tmp as *mut c_void);

            // Longest match: make sure it is not shorter than the literal
            // part of what was typed, which happens with :help.
            if !p.is_null() && mode == WILD_LONGEST {
                let mut literal = 0;
                while (literal as size_t) < (*xp).xp_pattern_len {
                    let c = *(*ccline).cmdbuff.offset((at + literal) as isize);
                    if c == b'*' as c_char || c == b'?' as c_char {
                        break;
                    }
                    literal += 1;
                }
                if (strlen(p) as c_int) < literal {
                    xfree(p as *mut c_void);
                    p = ptr::null_mut();
                }
            }
        }

        // Save the command line before inserting the selected item.
        if !wild_navigate && !(*ccline).cmdbuff.is_null() {
            xfree(cmdline_orig.get() as *mut c_void);
            cmdline_orig.set(xstrnsave((*ccline).cmdbuff, (*ccline).cmdlen as size_t));
        }

        if !p.is_null() && !got_int.get() && options & WILD_NOSELECT == 0 {
            let plen = strlen(p);
            let difflen = plen as c_int - (*xp).xp_pattern_len as c_int;
            if (*ccline).cmdlen + difflen + 4 > (*ccline).cmdbufflen {
                realloc_cmdbuff((*ccline).cmdlen + difflen + 4);
                // The buffer moved; re-derive the pattern pointer from `at`.
                (*xp).xp_pattern = (*ccline).cmdbuff.offset(at as isize);
            }
            debug_assert!((*ccline).cmdpos <= (*ccline).cmdlen);
            // Open (or close) a gap of `difflen` bytes at the cursor, taking
            // the NUL along, then drop the match in at the pattern's start.
            // Both copies overlap the destination, hence `copy`.
            ptr::copy(
                (*ccline).cmdbuff.offset((*ccline).cmdpos as isize),
                (*ccline)
                    .cmdbuff
                    .offset(((*ccline).cmdpos + difflen) as isize),
                ((*ccline).cmdlen - (*ccline).cmdpos + 1) as size_t,
            );
            ptr::copy(p, (*ccline).cmdbuff.offset(at as isize), plen);
            (*ccline).cmdlen += difflen;
            (*ccline).cmdpos += difflen;
        }

        redrawcmd();
        cursorcmd();

        // When expanding a ":map" command and no matches are found, assume
        // the key is supposed to be inserted literally.
        if (*xp).xp_context == EXPAND_MAPPINGS && p.is_null() {
            return FAIL;
        }

        if (*xp).xp_numfiles <= 0 && p.is_null() {
            beep_flush();
        } else if (*xp).xp_numfiles == 1 && options & WILD_NOSELECT == 0 && !wild_navigate {
            // Only one match: free the expanded pattern again.
            ExpandOne(xp, ptr::null_mut(), ptr::null_mut(), 0, WILD_FREE);
        }

        xfree(p as *mut c_void);
        OK
    }
}

/// Move the selection within an already expanded match list, and answer a
/// fresh copy of what is now selected (or of the original text, at index -1).
unsafe fn next_or_prev_match(mode: c_int, xp: *mut expand_T) -> *mut c_char {
    unsafe {
        // When no matches were found there is nothing to move within.
        if (*xp).xp_numfiles <= 0 {
            return ptr::null_mut();
        }
        let count = (*xp).xp_numfiles;
        let mut findex = (*xp).xp_selected;

        match mode {
            WILD_PREV => {
                // Select the last entry when at the original text, otherwise
                // the previous one.
                if findex == -1 {
                    findex = count;
                }
                findex -= 1;
            }
            WILD_NEXT => findex += 1,
            WILD_PAGEUP | WILD_PAGEDOWN => {
                // The height of the popup menu, less its border rows.
                let mut ht = pum_get_height();
                if ht > 3 {
                    ht -= 2;
                }
                findex = if mode == WILD_PAGEUP {
                    match findex {
                        0 => -1,                 // at the first entry: select none
                        f if f < 0 => count - 1, // none selected: select the last
                        f => (f - ht).max(0),
                    }
                } else {
                    match findex {
                        f if f == count - 1 => -1, // at the last entry: select none
                        f if f < 0 => 0,           // none selected: select the first
                        f => (f + ht).min(count - 1),
                    }
                };
            }
            _ => {
                // WILD_PUM_WANT: the UI named the item it wants.
                debug_assert!(pum_want.get().active);
                findex = pum_want.get().item;
            }
        }

        // Handle wrapping around.
        if findex < 0 || findex >= count {
            findex = if !(*xp).xp_orig.is_null() {
                -1 // return to the original text
            } else if findex < 0 {
                count - 1 // wrap around to the opposite end
            } else {
                0
            };
        }

        // Display the matches on screen.
        if p_wmnu.get() != 0 {
            if !compl_match_array.get().is_null() {
                compl_selected.set(findex);
                cmdline_pum_display(false);
            } else if cmdline_compl_use_pum(true) {
                cmdline_pum_create(
                    get_cmdline_info(),
                    xp,
                    (*xp).xp_files,
                    count,
                    cmd_showtail.get(),
                    false,
                );
                compl_selected.set(findex);
                pum_clear();
                cmdline_pum_display(true);
            } else {
                redraw_wildmenu(xp, count, (*xp).xp_files, findex, cmd_showtail.get());
            }
        }

        (*xp).xp_selected = findex;
        xstrdup(if findex == -1 {
            (*xp).xp_orig as *const c_char
        } else {
            matches_of(xp)[findex as usize] as *const c_char
        })
    }
}

/// Run the expansion and take ownership of the matches.
///
/// Answers an allocated copy of the first match for the modes that select one
/// (everything but `WILD_ALL`, `WILD_ALL_KEEP` and `WILD_LONGEST`, which the
/// caller assembles itself), and NULL otherwise.
unsafe fn expand_one_start(
    mode: c_int,
    xp: *mut expand_T,
    str: *mut c_char,
    options: c_int,
) -> *mut c_char {
    unsafe {
        if ExpandFromContext(
            xp,
            str,
            &raw mut (*xp).xp_files,
            &raw mut (*xp).xp_numfiles,
            options,
        ) == FAIL
        {
            // Upstream reports "No match" here under FNAME_ILLEGAL, which is
            // not defined on any platform this port builds for.
            return ptr::null_mut();
        }
        if (*xp).xp_numfiles == 0 {
            if options & WILD_SILENT == 0 {
                semsg(gettext(&raw const e_nomatch2 as *const c_char), str);
            }
            return ptr::null_mut();
        }

        // Escape the matches for use on the command line.
        escape_matches(
            xp,
            str,
            core::slice::from_raw_parts_mut((*xp).xp_files, (*xp).xp_numfiles as usize),
            options,
        );

        if mode == WILD_ALL || mode == WILD_ALL_KEEP || mode == WILD_LONGEST {
            return ptr::null_mut();
        }

        // Check for matching suffixes in file names.  (Upstream's
        // `xp_numfiles ? xp_numfiles : 1` can only take the first arm here:
        // the zero case returned above.)
        let mut non_suf_match = (*xp).xp_numfiles;
        if matches!((*xp).xp_context, EXPAND_FILES | EXPAND_DIRECTORIES) && (*xp).xp_numfiles > 1 {
            // More than one match; check the suffix.  expand_wildcards has
            // sorted the ones with a matching suffix to the front, so only
            // the first two need looking at.
            non_suf_match = matches_of(xp)[..2]
                .iter()
                .filter(|&&name| match_suffix(name))
                .count() as c_int;
        }
        if non_suf_match != 1 {
            // Can we ever get here unless it's while expanding
            // interactively?  If not, we can get rid of this all together.
            // Don't really want to wait for this message (and possibly have
            // to hit return to continue!).
            if options & WILD_SILENT == 0 {
                emsg(gettext(&raw const e_toomany as *const c_char));
            } else if options & WILD_NO_BEEP == 0 {
                beep_flush();
            }
        }
        if non_suf_match != 1 && mode == WILD_EXPAND_FREE {
            return ptr::null_mut();
        }
        xstrdup(matches_of(xp)[0] as *const c_char)
    }
}

/// The longest common prefix of the matches — the `'wildmode'`=longest answer.
///
/// Beeps (unless `WILD_NO_BEEP`) at the byte where they first diverge, which
/// is how the user learns the expansion stopped short of a whole name.
unsafe fn longest_common_match(xp: *mut expand_T, options: c_int) -> *mut c_char {
    unsafe {
        let files = matches_of(xp);
        let first = files[0];
        // 'fileignorecase' folds case, but only where the matches are names
        // that came from the filesystem or the buffer list.  Neither operand
        // can change inside the loop.
        let fold = p_fic.get() != 0
            && matches!(
                (*xp).xp_context,
                EXPAND_DIRECTORIES | EXPAND_FILES | EXPAND_SHELLCMD | EXPAND_BUFFERS
            );

        let mut len: size_t = 0;
        while *first.add(len as usize) != 0 {
            let mb_len = utfc_ptr2len(first.add(len as usize)) as size_t;
            let c0 = utf_ptr2char(first.add(len as usize));
            let diverged = files[1..].iter().any(|&name| {
                let ci = utf_ptr2char(name.add(len as usize));
                if fold {
                    mb_tolower(c0) != mb_tolower(ci)
                } else {
                    c0 != ci
                }
            });
            if diverged {
                if options & WILD_NO_BEEP == 0 {
                    vim_beep(kOptBoFlagWildmode as ::core::ffi::c_uint);
                }
                break;
            }
            len += mb_len;
        }

        xmemdupz(first as *const c_void, len) as *mut c_char
    }
}

/// Do wildcard expansion on the string `str`.
///
/// Chars that should not be expanded must be preceded with a backslash.
/// Answers allocated memory holding the new string, or NULL for failure.
///
/// `orig` is the originally expanded string, in allocated memory.  It is
/// either kept in `xp->xp_orig` or freed here.  With `mode` `WILD_NEXT` or
/// `WILD_PREV` it should be NULL.
///
/// Results are cached in `xp->xp_files` / `xp->xp_numfiles`, except when
/// `mode` is `WILD_EXPAND_FREE` or `WILD_ALL`.
///
/// | mode | |
/// | --- | --- |
/// | `WILD_FREE` | just free previously expanded matches |
/// | `WILD_EXPAND_FREE` | normal expansion, do not keep matches |
/// | `WILD_EXPAND_KEEP` | normal expansion, keep matches |
/// | `WILD_NEXT` / `WILD_PREV` | step through the matches, wrapping around |
/// | `WILD_ALL` | answer all matches concatenated |
/// | `WILD_LONGEST` | answer the longest matched part |
/// | `WILD_ALL_KEEP` | get all matches, keep matches |
/// | `WILD_APPLY` | apply the item selected in the completion popup menu |
/// | `WILD_CANCEL` | close the popup menu and use the original text |
/// | `WILD_PUM_WANT` | use the match at index `pum_want.item` |
///
/// `options` is a set of `WILD_LIST_NOTFOUND`, `WILD_HOME_REPLACE`,
/// `WILD_USE_NL`, `WILD_NO_BEEP`, `WILD_ADD_SLASH`, `WILD_KEEP_ALL`,
/// `WILD_SILENT`, `WILD_ESCAPE` and `WILD_ICASE`.
///
/// `xp->xp_context` and `xp->xp_backslash` must have been set.
pub unsafe fn ExpandOne(
    xp: *mut expand_T,
    str: *mut c_char,
    orig: *mut c_char,
    options: c_int,
    mode: c_int,
) -> *mut c_char {
    unsafe {
        // First handle the case of using an old match.
        if is_wild_navigate(mode) {
            return next_or_prev_match(mode, xp);
        }

        // The original text, for the two modes that answer with it.
        let orig_or_empty = || {
            if (*xp).xp_orig.is_null() {
                c"".as_ptr()
            } else {
                (*xp).xp_orig as *const c_char
            }
        };
        let mut ss = match mode {
            WILD_CANCEL => xstrdup(orig_or_empty()),
            WILD_APPLY if (*xp).xp_selected == -1 => xstrdup(orig_or_empty()),
            WILD_APPLY => xstrdup(matches_of(xp)[(*xp).xp_selected as usize] as *const c_char),
            _ => ptr::null_mut(),
        };

        // Free the old names.
        if (*xp).xp_numfiles != -1 && mode != WILD_ALL && mode != WILD_LONGEST {
            FreeWild((*xp).xp_numfiles, (*xp).xp_files);
            (*xp).xp_numfiles = -1;
            xfree((*xp).xp_orig as *mut c_void);
            (*xp).xp_orig = ptr::null_mut();

            // The entries from xp_files may be in the popup menu; remove it.
            if !compl_match_array.get().is_null() {
                cmdline_pum_remove(false);
            }
        }
        (*xp).xp_selected = if options & WILD_NOSELECT != 0 { -1 } else { 0 };

        if mode == WILD_FREE {
            // Only release the file names.
            return ptr::null_mut();
        }

        // Whether `orig` was stored in `xp_orig` rather than being ours to free.
        let mut orig_saved = false;
        if (*xp).xp_numfiles == -1 && mode != WILD_APPLY && mode != WILD_CANCEL {
            xfree((*xp).xp_orig as *mut c_void);
            (*xp).xp_orig = orig;
            orig_saved = true;
            ss = expand_one_start(mode, xp, str, options);
        }

        // Find the longest common part.
        if mode == WILD_LONGEST && (*xp).xp_numfiles > 0 {
            ss = longest_common_match(xp, options);
            (*xp).xp_selected = -1; // next 'wildchar' gets the first one
        }

        // Concatenate all matching names.  Unless interrupted this can be
        // slow, and the result probably won't be used.
        if mode == WILD_ALL && (*xp).xp_numfiles > 0 && !got_int.get() {
            let files = matches_of(xp);
            let suffix = if options & WILD_USE_NL != 0 {
                c"\n"
            } else {
                c" "
            };
            // A boolean option's matches are listed as "novimfile" /
            // "invvimfile"; the prefix goes *between* the entries, so there
            // is one fewer of it than there are matches.
            let prefix = match (*xp).xp_prefix {
                XP_PREFIX_NO => c"no",
                XP_PREFIX_INV => c"inv",
                _ => c"",
            };
            let last = files.len() - 1;
            let mut ss_size = prefix.count_bytes() * last;
            for &name in files {
                ss_size += strlen(name) + 1; // +1 for the suffix
            }
            ss_size += 1; // +1 for the NUL

            let buf = xmalloc(ss_size) as *mut c_char;
            *buf = 0;
            let mut ssp = buf;
            for (i, &name) in files.iter().enumerate() {
                if i > 0 {
                    ssp = xstpcpy(ssp, prefix.as_ptr());
                }
                ssp = xstpcpy(ssp, name);
                if i < last {
                    ssp = xstpcpy(ssp, suffix.as_ptr());
                }
                debug_assert!(ssp < buf.add(ss_size));
            }
            ss = buf;
        }

        if mode == WILD_EXPAND_FREE || mode == WILD_ALL {
            ExpandCleanup(xp);
        }

        // Free "orig" if it wasn't stored in "xp->xp_orig".
        if !orig_saved {
            xfree(orig as *mut c_void);
        }

        ss
    }
}

/// Prepare an expand structure for use.
pub unsafe fn ExpandInit(xp: *mut expand_T) {
    unsafe {
        xp.write_bytes(0, 1);
        (*xp).xp_backslash = XP_BS_NONE;
        (*xp).xp_prefix = XP_PREFIX_NONE;
        (*xp).xp_numfiles = -1;
    }
}

/// Clean up an expand structure after use.
pub unsafe fn ExpandCleanup(xp: *mut expand_T) {
    unsafe {
        if (*xp).xp_numfiles >= 0 {
            FreeWild((*xp).xp_numfiles, (*xp).xp_files);
            (*xp).xp_numfiles = -1;
        }
        xfree((*xp).xp_orig as *mut c_void);
        (*xp).xp_orig = ptr::null_mut();
    }
}

/// Drop the saved copy of the command line taken before the last expansion.
pub unsafe fn clear_cmdline_orig() {
    unsafe {
        xfree(cmdline_orig.get() as *mut c_void);
        cmdline_orig.set(ptr::null_mut());
    }
}
