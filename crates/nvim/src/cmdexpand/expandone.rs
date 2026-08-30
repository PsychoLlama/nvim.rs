//! The wildcard key: one `<Tab>` press, from key to command line.
//!
//! [`nextwild`] is what the command-line key loop calls; it isolates the word
//! under the cursor, hands it to [`expand_one`] and puts the answer back.
//! [`expand_one`] owns the match array across presses — [`expand_one_start`]
//! fills it, [`next_match`] cycles it and [`longest_common_match`]
//! computes the `'wildmode'`=longest answer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{BackslashEscape, ExpandContext, FAIL, OK};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// [`expand_one`]'s `str` and `orig` where the caller is only asking it to
/// move within or free an existing match list; neither is read there.
const NO_PATTERN: *mut c_char = ptr::null_mut();

/// The original text `xp` saved, or the empty string when it saved none.
///
/// # Safety
/// `xp` must point at a live `expand_T`.
unsafe fn orig_or_empty(xp: *const expand_T) -> *const c_char {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp.cast_mut()) };
    // SAFETY: the caller's promise.
    let orig = xp.xp_orig;
    if orig.is_null() {
        c"".as_ptr()
    } else {
        orig.cast_const()
    }
}

/// The index [`expand_one`] starts the selection at: the first match, or -1
/// for "the original text" when the caller asked for nothing selected.
const fn first_selected(options: WildOpts) -> c_int {
    if options.has(WildOpts::NOSELECT) {
        -1
    } else {
        0
    }
}

/// The expanded matches, as a slice.  Only call this where `xp_numfiles` is
/// known positive: it is -1 before anything has been expanded.
unsafe fn matches_of(xp: *const expand_T) -> &'static [*mut c_char] {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp.cast_mut()) };
    debug_assert!(xp.xp_numfiles > 0);
    // `.max(0)`: -1 means "nothing expanded", and building a slice of
    // `usize::MAX` entries out of that would be instant UB where the C
    // merely read past the end.
    unsafe { core::slice::from_raw_parts(xp.xp_files, xp.xp_numfiles.max(0) as usize) }
}

/// Expand the word before the cursor on the command line.
///
/// Answers `FAIL` when this is not a context in which anything can be
/// completed, which tells the caller to pass the character through as a
/// normal character instead — that is what makes `:s/^I^D` work.  `OK` means
/// the key was consumed, even when there were no matches.
///
/// `mode` is one of the `WILD_*` modes, passed on to [`expand_one`]; `escape`
/// asks for the matches to be escaped for use on the command line.
pub(crate) unsafe fn nextwild(
    xp: *mut expand_T,
    mode: WildMode,
    options: WildOpts,
    escape: bool,
) -> c_int {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut ccline = Cc::current();
    let from_wildtrigger_func = options.has(WildOpts::FUNC_TRIGGER);
    let wild_navigate = mode.navigates();

    if xp.xp_numfiles == -1 {
        pre_incsearch_pos.set(xp.xp_pre_incsearch_pos);
        if ccline.input_fn != 0 && ccline.xp_context == ExpandContext::Commands {
            // Expand commands typed in the input() function.
            unsafe { set_cmd_context(xp.raw(), ccline.text(), ccline.len(), ccline.cmdpos, false) };
        } else {
            may_expand_pattern.set(options.has(WildOpts::MAY_EXPAND_PATTERN));
            unsafe { set_expand_context(xp.raw()) };
            may_expand_pattern.set(false);
        }
        if xp.xp_context == ExpandContext::Lua {
            unsafe { nlua_expand_pat(xp.raw()) };
        }
        cmd_showtail.set(unsafe { expand_showtail(xp.raw()) });
    }

    match xp.xp_context {
        // Something illegal on the command line.
        ExpandContext::Unsuccessful => {
            beep_flush();
            return OK;
        }
        // The caller can use the character as a normal char instead.
        ExpandContext::Nothing => return FAIL,
        _ => {}
    }

    // Where the pattern starts within the command line.  Held as an index
    // rather than a pointer because `realloc_cmdbuff` below can move the
    // buffer out from under `xp_pattern`.
    let at = unsafe { xp.xp_pattern.offset_from(ccline.text()) } as c_int;
    debug_assert!(ccline.cmdpos >= at);
    xp.xp_pattern_len = (ccline.cmdpos - at) as size_t;

    // Skip showing matches if the prefix is invalid during wildtrigger().
    let context = xp.xp_context;
    if from_wildtrigger_func && context == ExpandContext::Commands && xp.xp_pattern_len == 0 {
        return FAIL;
    }

    // If 'cmd_silent' is set don't show the dots, because the redrawcmd()
    // below won't remove them.
    if !cmd_silent.get()
        && !from_wildtrigger_func
        && !wild_navigate
        && !(ui_has(kUICmdline) || ui_has(kUIWildmenu))
    {
        unsafe { msg_puts(c"...".as_ptr()) }; // show that we are busy
        unsafe { ui_flush() };
    }

    let mut p;
    if wild_navigate {
        // Get the next/previous match of an already expanded pattern.
        p = unsafe {
            expand_one(
                xp.raw(),
                ptr::null_mut(),
                ptr::null_mut(),
                WildOpts::NONE,
                mode,
            )
        };
    } else {
        let tmp = if unsafe { cmdline_fuzzy_completion_supported(xp.raw()) }
            || xp.xp_context == ExpandContext::PatternInBuf
        {
            // Don't modify the search string.
            unsafe { xstrnsave(xp.xp_pattern, xp.xp_pattern_len) }
        } else {
            unsafe { addstar(xp.xp_pattern, xp.xp_pattern_len, xp.xp_context) }
        };
        // Translate the string into a pattern and expand it.
        let use_options = options
            | WildOpts::HOME_REPLACE
            | WildOpts::ADD_SLASH
            | WildOpts::SILENT
            | WildOpts::ESCAPE.when(escape)
            | WildOpts::ICASE.when(p_wic.get() != 0);
        p = unsafe {
            expand_one(
                xp.raw(),
                tmp,
                xstrnsave(ccline.at(at), xp.xp_pattern_len),
                use_options,
                mode,
            )
        };
        unsafe { xfree(tmp as *mut c_void) };

        // Longest match: make sure it is not shorter than the literal
        // part of what was typed, which happens with :help.
        if !p.is_null() && mode == WildMode::Longest {
            let mut literal = 0;
            while (literal as size_t) < xp.xp_pattern_len {
                let c = unsafe { *ccline.at(at + literal) };
                if c == b'*' as c_char || c == b'?' as c_char {
                    break;
                }
                literal += 1;
            }
            if (unsafe { cstr::bytes_at(p) }.len() as c_int) < literal {
                unsafe { xfree(p as *mut c_void) };
                p = ptr::null_mut();
            }
        }
    }

    // Save the command line before inserting the selected item.
    if !wild_navigate && ccline.in_use() {
        unsafe { xfree(cmdline_orig.get() as *mut c_void) };
        cmdline_orig.set(unsafe { xstrnsave(ccline.text(), ccline.len() as size_t) });
    }

    if !p.is_null() && !got_int.get() && !options.has(WildOpts::NOSELECT) {
        let plen = unsafe { cstr::bytes_at(p) }.len();
        let difflen = plen as c_int - xp.xp_pattern_len as c_int;
        // The buffer may move; re-derive the pattern pointer from `at`.
        realloc_cmdbuff(ccline, ccline.len() + difflen + 4);
        xp.xp_pattern = ccline.at(at);

        debug_assert!(ccline.cmdpos <= ccline.len());
        // Open (or close) a gap of `difflen` bytes at the cursor, taking
        // the NUL along, then drop the match in at the pattern's start.
        // Both copies overlap the destination, hence `copy`.
        unsafe {
            ptr::copy(
                ccline.at(ccline.cmdpos),
                ccline.at(ccline.cmdpos + difflen),
                (ccline.len() - ccline.cmdpos + 1) as size_t,
            )
        };
        unsafe { ptr::copy(p, ccline.at(at), plen) };
        ccline.set_len(ccline.len() + difflen);
        ccline.cmdpos += difflen;
    }

    unsafe { redrawcmd() };
    unsafe { cursorcmd() };

    // When expanding a ":map" command and no matches are found, assume
    // the key is supposed to be inserted literally.
    if xp.xp_context == ExpandContext::Mappings && p.is_null() {
        return FAIL;
    }

    if xp.xp_numfiles <= 0 && p.is_null() {
        beep_flush();
    } else if xp.xp_numfiles == 1 && !options.has(WildOpts::NOSELECT) && !wild_navigate {
        // Only one match: free the expanded pattern again.
        unsafe {
            expand_one(
                xp.raw(),
                NO_PATTERN,
                NO_PATTERN,
                WildOpts::NONE,
                WildMode::Free,
            )
        };
    }

    unsafe { xfree(p as *mut c_void) };
    OK
}

/// Move the selection within an already expanded match list, and answer a
/// fresh copy of what is now selected (or of the original text, at index -1).
unsafe fn next_match(mode: WildMode, xp: *mut expand_T) -> *mut c_char {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    // When no matches were found there is nothing to move within.
    if xp.xp_numfiles <= 0 {
        return ptr::null_mut();
    }
    let count = xp.xp_numfiles;
    let mut findex = xp.xp_selected;

    match mode {
        WildMode::Prev => {
            // Select the last entry when at the original text, otherwise
            // the previous one.
            if findex == -1 {
                findex = count;
            }
            findex -= 1;
        }
        WildMode::Next => findex += 1,
        WildMode::PageUp | WildMode::PageDown => {
            // The height of the popup menu, less its border rows.
            let mut ht = pum_get_height();
            if ht > 3 {
                ht -= 2;
            }
            findex = if mode == WildMode::PageUp {
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
        WildMode::PumWant => {
            // The UI named the item it wants.
            debug_assert!(pum_want.get().active);
            findex = pum_want.get().item;
        }
        // `WildMode::navigates` is the caller's guard, and it names
        // exactly the five arms above; anything else is a mis-dispatch,
        // which as a bare `_` used to be answered as `PumWant`.
        mode => unreachable!("{mode:?} does not move within a match list"),
    }

    // Handle wrapping around.
    if findex < 0 || findex >= count {
        findex = if !xp.xp_orig.is_null() {
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
            unsafe { cmdline_pum_display(false) };
        } else if cmdline_compl_use_pum(true) {
            unsafe {
                cmdline_pum_create(
                    Cc::current(),
                    xp.raw(),
                    xp.xp_files,
                    count,
                    cmd_showtail.get(),
                    false,
                )
            };
            compl_selected.set(findex);
            pum_clear();
            unsafe { cmdline_pum_display(true) };
        } else {
            unsafe { redraw_wildmenu(xp.raw(), count, xp.xp_files, findex, cmd_showtail.get()) };
        }
    }

    xp.xp_selected = findex;
    unsafe {
        xstrdup(if findex == -1 {
            xp.xp_orig as *const c_char
        } else {
            matches_of(xp.raw())[findex as usize] as *const c_char
        })
    }
}

/// Run the expansion and take ownership of the matches.
///
/// Answers an allocated copy of the first match for the modes that select one
/// (everything but `WildMode::All`, `WildMode::AllKeep` and `WildMode::Longest`, which the
/// caller assembles itself), and NULL otherwise.
unsafe fn expand_one_start(
    mode: WildMode,
    xp: *mut expand_T,
    str: *mut c_char,
    options: WildOpts,
) -> *mut c_char {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    // `field_ptr`, not `&raw mut xp.xp_files`: two addresses off one
    // `Deref` would pop each other, and `expand_from_context` writes both.
    let files = xp.field_ptr(core::mem::offset_of!(expand_T, xp_files));
    let numfiles = xp.field_ptr(core::mem::offset_of!(expand_T, xp_numfiles));
    // SAFETY: `xp` is live and both out-parameters are its own fields.
    let expanded = unsafe { expand_from_context(xp.raw(), str, files, numfiles, options) };
    if expanded.is_err() {
        // Upstream reports "No match" here under FNAME_ILLEGAL, which is
        // not defined on any platform this port builds for.
        return ptr::null_mut();
    }
    if xp.xp_numfiles == 0 {
        if !options.has(WildOpts::SILENT) {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(str) };
            semsg!("E480: No match: {arg0}");
        }
        return ptr::null_mut();
    }

    // Escape the matches for use on the command line.
    unsafe {
        escape_matches(
            xp.raw(),
            str,
            core::slice::from_raw_parts_mut(xp.xp_files, xp.xp_numfiles as usize),
            options,
        )
    };

    if mode == WildMode::All || mode == WildMode::AllKeep || mode == WildMode::Longest {
        return ptr::null_mut();
    }

    // Check for matching suffixes in file names.  (Upstream's
    // `xp_numfiles ? xp_numfiles : 1` can only take the first arm here:
    // the zero case returned above.)
    let mut non_suf_match = xp.xp_numfiles;
    let ctx = xp.xp_context;
    let names = matches!(ctx, ExpandContext::Files | ExpandContext::Directories);
    if names && xp.xp_numfiles > 1 {
        // More than one match; check the suffix.  expand_wildcards has
        // sorted the ones with a matching suffix to the front, so only
        // the first two need looking at.
        non_suf_match = unsafe { matches_of(xp.raw()) }[..2]
            .iter()
            .filter(|&&name| unsafe { match_suffix(name) })
            .count() as c_int;
    }
    if non_suf_match != 1 {
        // Can we ever get here unless it's while expanding
        // interactively?  If not, we can get rid of this all together.
        // Don't really want to wait for this message (and possibly have
        // to hit return to continue!).
        if !options.has(WildOpts::SILENT) {
            emsg(gettext(e_toomany));
        } else if !options.has(WildOpts::NO_BEEP) {
            beep_flush();
        }
    }
    if non_suf_match != 1 && mode == WildMode::ExpandFree {
        return ptr::null_mut();
    }
    unsafe { xstrdup(matches_of(xp.raw())[0] as *const c_char) }
}

/// The longest common prefix of the matches — the `'wildmode'`=longest answer.
///
/// Beeps (unless `WildOpts::NO_BEEP`) at the byte where they first diverge, which
/// is how the user learns the expansion stopped short of a whole name.
unsafe fn longest_common_match(xp: *mut expand_T, options: WildOpts) -> *mut c_char {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let files = unsafe { matches_of(xp.raw()) };
    let first = files[0];
    // 'fileignorecase' folds case, but only where the matches are names
    // that came from the filesystem or the buffer list.  Neither operand
    // can change inside the loop.
    let fold = p_fic.get() != 0
        && matches!(
            xp.xp_context,
            ExpandContext::Directories
                | ExpandContext::Files
                | ExpandContext::ShellCmd
                | ExpandContext::Buffers
        );

    let mut len: size_t = 0;
    while unsafe { *first.add(len as usize) } != 0 {
        let mb_len = unsafe { utfc_ptr2len(first.add(len as usize)) } as size_t;
        let c0 = unsafe { utf_ptr2char(first.add(len as usize)) };
        let diverged = files[1..].iter().any(|&name| {
            let ci = unsafe { utf_ptr2char(name.add(len as usize)) };
            if fold {
                mb_tolower(c0) != mb_tolower(ci)
            } else {
                c0 != ci
            }
        });
        if diverged {
            if !options.has(WildOpts::NO_BEEP) {
                unsafe { vim_beep(kOptBoFlagWildmode as ::core::ffi::c_uint) };
            }
            break;
        }
        len += mb_len;
    }

    unsafe { xmemdupz(first as *const c_void, len) as *mut c_char }
}

/// Do wildcard expansion on the string `str`.
///
/// Chars that should not be expanded must be preceded with a backslash.
/// Answers allocated memory holding the new string, or NULL for failure.
///
/// `orig` is the originally expanded string, in allocated memory.  It is
/// either kept in `xp->xp_orig` or freed here.  With `mode` `WildMode::Next` or
/// `WildMode::Prev` it should be NULL.
///
/// Results are cached in `xp->xp_files` / `xp->xp_numfiles`, except when
/// `mode` is `WildMode::ExpandFree` or `WildMode::All`.
///
/// | mode | |
/// | --- | --- |
/// | `WildMode::Free` | just free previously expanded matches |
/// | `WildMode::ExpandFree` | normal expansion, do not keep matches |
/// | `WildMode::ExpandKeep` | normal expansion, keep matches |
/// | `WildMode::Next` / `WildMode::Prev` | step through the matches, wrapping around |
/// | `WildMode::All` | answer all matches concatenated |
/// | `WildMode::Longest` | answer the longest matched part |
/// | `WildMode::AllKeep` | get all matches, keep matches |
/// | `WildMode::Apply` | apply the item selected in the completion popup menu |
/// | `WildMode::Cancel` | close the popup menu and use the original text |
/// | `WildMode::PumWant` | use the match at index `pum_want.item` |
///
/// `options` is a set of `WildOpts::LIST_NOTFOUND`, `WildOpts::HOME_REPLACE`,
/// `WildOpts::USE_NL`, `WildOpts::NO_BEEP`, `WildOpts::ADD_SLASH`, `WildOpts::KEEP_ALL`,
/// `WildOpts::SILENT`, `WildOpts::ESCAPE` and `WildOpts::ICASE`.
///
/// `xp->xp_context` and `xp->xp_backslash` must have been set.
pub unsafe fn expand_one(
    xp: *mut expand_T,
    str: *mut c_char,
    orig: *mut c_char,
    options: WildOpts,
    mode: WildMode,
) -> *mut c_char {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    // First handle the case of using an old match.
    if mode.navigates() {
        return unsafe { next_match(mode, xp.raw()) };
    }

    // The original text, for the two modes that answer with it.
    let mut ss = match mode {
        WildMode::Cancel => unsafe { xstrdup(orig_or_empty(xp.raw())) },
        WildMode::Apply if xp.xp_selected == -1 => unsafe { xstrdup(orig_or_empty(xp.raw())) },
        WildMode::Apply => unsafe {
            xstrdup(matches_of(xp.raw())[xp.xp_selected as usize] as *const c_char)
        },
        _ => ptr::null_mut(),
    };

    // Free the old names.
    if xp.xp_numfiles != -1 && mode != WildMode::All && mode != WildMode::Longest {
        unsafe { free_wild(xp.xp_numfiles, xp.xp_files) };
        xp.xp_numfiles = -1;
        unsafe { xfree(xp.xp_orig as *mut c_void) };
        xp.xp_orig = ptr::null_mut();

        // The entries from xp_files may be in the popup menu; remove it.
        if !compl_match_array.get().is_null() {
            unsafe { cmdline_pum_remove(false) };
        }
    }
    xp.xp_selected = first_selected(options);

    if mode == WildMode::Free {
        // Only release the file names.
        return ptr::null_mut();
    }

    // Whether `orig` was stored in `xp_orig` rather than being ours to free.
    let mut orig_saved = false;
    if xp.xp_numfiles == -1 && mode != WildMode::Apply && mode != WildMode::Cancel {
        unsafe { xfree(xp.xp_orig as *mut c_void) };
        xp.xp_orig = orig;
        orig_saved = true;
        ss = unsafe { expand_one_start(mode, xp.raw(), str, options) };
    }

    // Find the longest common part.
    if mode == WildMode::Longest && xp.xp_numfiles > 0 {
        ss = unsafe { longest_common_match(xp.raw(), options) };
        xp.xp_selected = -1; // next 'wildchar' gets the first one
    }

    // Concatenate all matching names.  Unless interrupted this can be
    // slow, and the result probably won't be used.
    if mode == WildMode::All && xp.xp_numfiles > 0 && !got_int.get() {
        let files = unsafe { matches_of(xp.raw()) };
        let suffix = if options.has(WildOpts::USE_NL) {
            c"\n"
        } else {
            c" "
        };
        // A boolean option's matches are listed as "novimfile" /
        // "invvimfile"; the prefix goes *between* the entries, so there
        // is one fewer of it than there are matches.
        let prefix = match xp.xp_prefix {
            XP_PREFIX_NO => c"no",
            XP_PREFIX_INV => c"inv",
            _ => c"",
        };
        let last = files.len() - 1;
        let mut ss_size = prefix.count_bytes() * last;
        for &name in files {
            ss_size += unsafe { cstr::bytes_at(name) }.len() + 1; // +1 for the suffix
        }
        ss_size += 1; // +1 for the NUL

        let buf = unsafe { xmalloc(ss_size) } as *mut c_char;
        unsafe { *buf = 0 };
        let mut ssp = buf;
        for (i, &name) in files.iter().enumerate() {
            if i > 0 {
                ssp = unsafe { xstpcpy(ssp, prefix.as_ptr()) };
            }
            ssp = unsafe { xstpcpy(ssp, name) };
            if i < last {
                ssp = unsafe { xstpcpy(ssp, suffix.as_ptr()) };
            }
            debug_assert!(ssp < unsafe { buf.add(ss_size) });
        }
        ss = buf;
    }

    if mode == WildMode::ExpandFree || mode == WildMode::All {
        unsafe { expand_cleanup(xp.raw()) };
    }

    // Free "orig" if it wasn't stored in "xp->xp_orig".
    if !orig_saved {
        unsafe { xfree(orig as *mut c_void) };
    }

    ss
}

/// Prepare an expand structure for use.
pub unsafe fn expand_init(xp: *mut expand_T) {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    unsafe { xp.raw().write_bytes(0, 1) };
    xp.xp_backslash = BackslashEscape::NONE;
    xp.xp_prefix = XP_PREFIX_NONE;
    xp.xp_numfiles = -1;
}

/// Clean up an expand structure after use.
pub unsafe fn expand_cleanup(xp: *mut expand_T) {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    if xp.xp_numfiles >= 0 {
        unsafe { free_wild(xp.xp_numfiles, xp.xp_files) };
        xp.xp_numfiles = -1;
    }
    unsafe { xfree(xp.xp_orig as *mut c_void) };
    xp.xp_orig = ptr::null_mut();
}

/// Drop the saved copy of the command line taken before the last expansion.
pub unsafe fn clear_cmdline_orig() {
    unsafe { xfree(cmdline_orig.get() as *mut c_void) };
    cmdline_orig.set(ptr::null_mut());
}
