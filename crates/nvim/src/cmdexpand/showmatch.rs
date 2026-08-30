//! `:set nowildmenu` output: the match list printed as messages.
//!
//! [`showmatches`] lays the matches out in columns and prints them with
//! [`showmatches_oneline`]; [`expand_showtail`] decides whether a file match
//! is shown as its tail alone.  [`addstar`] is here because it is the other
//! half of the same question — what the pattern looked like before the
//! matches were found.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::{ExpandContext, MAXPATHL, NUL};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// One line of the match listing.
///
/// `matches[linenr]`, `matches[linenr + lines]`, … are the entries that share
/// a line; `maxlen` is the column width and `showtail` asks for file names to
/// be shown as their tail alone.
pub(crate) unsafe fn showmatches_oneline(
    xp: *mut expand_T,
    matches: *mut *mut c_char,
    numMatches: c_int,
    lines: c_int,
    linenr: c_int,
    maxlen: c_int,
    showtail: bool,
) {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    // `msg_outtrans` runs the message machinery, which is why the shortened
    // name it is handed is this frame's and not the shared `NameBuff`.
    let mut shown = [0 as c_char; MAXPATHL as usize];
    // C's SHOW_MATCH().
    let show_match = |i: c_int| {
        let m = unsafe { *matches.offset(i as isize) };
        if showtail {
            unsafe { showmatches_gettail(m, false) }
        } else {
            m
        }
    };

    let mut lastlen = 999;
    let mut j = linenr;
    while j < numMatches {
        if xp.xp_context == ExpandContext::TagsListFiles {
            unsafe { msg_outtrans(*matches.offset(j as isize), HLF_D, false) };
            let name = unsafe { *matches.offset(j as isize) };
            // SAFETY: the tag file name follows the tag's own NUL, which is
            // how `ExpandContext::TagsListFiles` packs the two.
            let p = unsafe { name.add(cstr::bytes_at(name).len() + 1) };
            unsafe { msg_advance(maxlen + 1) };
            unsafe { msg_puts(p) };
            unsafe { msg_advance(maxlen + 3) };
            unsafe { msg_outtrans_long(p.add(2), HLF_D) };
            break;
        }
        for _ in 0..(maxlen - lastlen).max(0) {
            unsafe { msg_putchar(' ' as c_int) };
        }
        let isdir;
        let p;
        if xp.xp_context == ExpandContext::Files
            || xp.xp_context == ExpandContext::ShellCmd
            || xp.xp_context == ExpandContext::Buffers
        {
            // Highlight directories.
            if xp.xp_numfiles != -1 {
                // Expansion was done before and special characters were
                // escaped, need to halve backslashes.  Also $HOME has been
                // replaced with ~/.
                let exp_path = unsafe { expand_env_save_opt(*matches.offset(j as isize), true) };
                let path = if !exp_path.is_null() {
                    exp_path
                } else {
                    unsafe { *matches.offset(j as isize) }
                };
                let halved_slash = unsafe { backslash_halve_save(path) };
                isdir = unsafe { os_isdir(halved_slash) };
                unsafe { xfree(exp_path as *mut c_void) };
                if halved_slash != path {
                    unsafe { xfree(halved_slash as *mut c_void) };
                }
            } else {
                // Expansion was done here, file names are literal.
                isdir = unsafe { os_isdir(*matches.offset(j as isize)) };
            }
            if showtail {
                p = show_match(j);
            } else {
                unsafe {
                    home_replace(
                        ptr::null(),
                        *matches.offset(j as isize),
                        shown.as_mut_ptr(),
                        MAXPATHL as size_t,
                        true,
                    )
                };
                p = shown.as_mut_ptr();
            }
        } else {
            isdir = false;
            p = show_match(j);
        }
        lastlen = unsafe { msg_outtrans(p, if isdir { HLF_D } else { 0 }, false) };
        j += lines;
    }
    if msg_col.get() > 0 {
        // When not wrapped around.
        unsafe { msg_clr_eos() };
        unsafe { msg_putchar('\n' as c_int) };
    }
}

/// Display completion matches.
///
/// Answers `Expanded::Nothing` when the character that triggered expansion should
/// be inserted as a normal character.
pub unsafe fn showmatches(
    xp: *mut expand_T,
    display_wildmenu: bool,
    display_list: bool,
    noselect: bool,
) -> Expanded {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut shown = [0 as c_char; MAXPATHL as usize];
    let mut ccline = Cc::current();
    let mut numMatches = 0;
    let mut matches = ptr::null_mut();
    let showtail;

    if xp.xp_numfiles == -1 {
        unsafe { set_expand_context(xp.raw()) };
        if xp.xp_context == ExpandContext::Lua {
            unsafe { nlua_expand_pat(xp.raw()) };
        }
        let retval = unsafe {
            expand_cmdline(
                xp.raw(),
                ccline.text(),
                ccline.cmdpos,
                &raw mut numMatches,
                &raw mut matches,
            )
        };
        if retval != Expanded::Ok {
            return retval;
        }
        showtail = unsafe { expand_showtail(xp.raw()) };
    } else {
        numMatches = xp.xp_numfiles;
        matches = xp.xp_files;
        showtail = cmd_showtail.get();
    }

    if cmdline_compl_use_pum(display_wildmenu && !display_list) {
        unsafe {
            cmdline_pum_create(
                Cc::current(),
                xp.raw(),
                matches,
                numMatches,
                showtail,
                noselect,
            )
        };
        compl_selected.set(if noselect { -1 } else { 0 });
        pum_clear();
        unsafe { cmdline_pum_display(true) };
        return Expanded::Ok;
    }

    if display_list {
        msg_didany.set(false); // lines_left will be set
        unsafe { msg_start() }; // prepare for paging
        if !ui_has(kUIMessages) {
            unsafe { msg_putchar('\n' as c_int) };
        }
        unsafe { ui_flush() };
        cmdline_row.set(msg_row.get());
        msg_didany.set(false); // lines_left will be set again
        unsafe { msg_ext_set_kind(c"wildlist".as_ptr()) };
        unsafe { msg_start() }; // prepare for paging
    }

    if got_int.get() {
        got_int.set(false); // only interrupt the completion, not the cmd line
    } else if display_wildmenu && !display_list {
        // Display statusbar menu.
        unsafe {
            redraw_wildmenu(
                xp.raw(),
                numMatches,
                matches,
                if noselect { -1 } else { 0 },
                showtail,
            )
        };
    } else if display_list {
        // C's SHOW_MATCH().
        let show_match = |i: c_int| {
            let m = unsafe { *matches.offset(i as isize) };
            if showtail {
                unsafe { showmatches_gettail(m, false) }
            } else {
                m
            }
        };

        // Find the length of the longest file name.
        let mut maxlen = 0;
        for i in 0..numMatches {
            let len = if !showtail
                && (xp.xp_context == ExpandContext::Files
                    || xp.xp_context == ExpandContext::ShellCmd
                    || xp.xp_context == ExpandContext::Buffers)
            {
                unsafe {
                    home_replace(
                        ptr::null(),
                        *matches.offset(i as isize),
                        shown.as_mut_ptr(),
                        MAXPATHL as size_t,
                        true,
                    )
                };
                unsafe { vim_strsize(shown.as_ptr()) }
            } else {
                unsafe { vim_strsize(show_match(i)) }
            };
            maxlen = maxlen.max(len);
        }

        let lines = if xp.xp_context == ExpandContext::TagsListFiles {
            numMatches
        } else {
            // Compute the number of columns and lines for the listing.
            maxlen += 2; // two spaces between file names
            let columns = ((Columns.get() + 2) / maxlen).max(1);
            (numMatches + columns - 1) / columns
        };

        if xp.xp_context == ExpandContext::TagsListFiles {
            unsafe { msg_puts_hl(gettext(c"tagname").as_ptr(), HLF_T, false) };
            unsafe { msg_clr_eos() };
            unsafe { msg_advance(maxlen - 3) };
            unsafe { msg_puts_hl(gettext(c" kind file\n").as_ptr(), HLF_T, false) };
        }

        // List the files line by line.
        for i in 0..lines {
            unsafe {
                showmatches_oneline(xp.raw(), matches, numMatches, lines, i, maxlen, showtail)
            };
            if got_int.get() {
                got_int.set(false);
                break;
            }
        }

        // We redraw the command below the lines that we have just listed.
        // This is a bit tricky, but it saves a lot of screen updating.
        cmdline_row.set(msg_row.get()); // will put it back later
    }

    if xp.xp_numfiles == -1 {
        unsafe { free_wild(numMatches, matches) };
    }

    Expanded::Ok
}

/// [`path_tail`] for [`showmatches`] and [`redraw_wildmenu`]: the tail of file
/// name path `s`, ignoring a trailing `/`.
///
/// `eager` takes the text after the last separator even when it is empty.
pub(crate) unsafe fn showmatches_gettail(s: *mut c_char, eager: bool) -> *mut c_char {
    let mut t = s;
    let mut had_sep = false;

    let mut p = s;
    while unsafe { *p } as c_int != NUL {
        if vim_ispathsep(unsafe { *p } as c_int) {
            if eager {
                t = unsafe { p.add(1) };
            } else {
                had_sep = true;
            }
        } else if had_sep {
            t = p;
            had_sep = false;
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    t
}

/// True if we only need to show the tail of completion matches.
///
/// When not completing file names, or when there is a wildcard in the path,
/// false is returned.
pub(crate) unsafe fn expand_showtail(xp: *mut expand_T) -> bool {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    // When not completing file names a "/" may mean something different.
    if xp.xp_context != ExpandContext::Files
        && xp.xp_context != ExpandContext::ShellCmd
        && xp.xp_context != ExpandContext::Directories
    {
        return false;
    }

    let end = unsafe { path_tail(xp.xp_pattern) };
    if end == xp.xp_pattern {
        // There is no path separator.
        return false;
    }

    let mut s = xp.xp_pattern;
    while s < end {
        // Skip escaped wildcards.  Only when the backslash is not a path
        // separator, on DOS the '*' "path\*\file" must not be skipped.
        if unsafe { rem_backslash(s) } {
            s = unsafe { s.add(1) };
        } else if !unsafe { vim_strchr(c"*?[".as_ptr(), *s as u8 as c_int) }.is_null() {
            return false;
        }
        s = unsafe { s.add(1) };
    }
    true
}

/// Prepare a string for expansion.
///
/// When expanding file names the string will be used with
/// `expand_wildcards()`: `fname[len]` is copied into allocated memory and a
/// `*` is added at the end.  When expanding other names it will be used with
/// `vim_regcomp()`: the name is copied and `^` prepended, with the
/// file-matching wildcards converted to regexp ones.
///
/// `context` is the `EXPAND_*` the pattern came from.  The answer is never
/// NULL.
pub unsafe fn addstar(fname: *mut c_char, mut len: size_t, context: ExpandContext) -> *mut c_char {
    if context != ExpandContext::Files
        && context != ExpandContext::FilesInPath
        && context != ExpandContext::ShellCmd
        && context != ExpandContext::Directories
        && context != ExpandContext::DirsInCdpath
    {
        // Matching will be done internally (on something other than
        // files).  So we convert the file-matching-type wildcards into our
        // kind for use with vim_regcomp().  First work out how long it
        // will be.

        // For help tags the translation is done in find_help_tags().
        // For a tag pattern starting with "/" no translation is needed.
        if context == ExpandContext::Findfunc
            || context == ExpandContext::Help
            || context == ExpandContext::Colors
            || context == ExpandContext::Compiler
            || context == ExpandContext::Ownsyntax
            || context == ExpandContext::Filetype
            || context == ExpandContext::Keymap
            || context == ExpandContext::Packadd
            || context == ExpandContext::Runtime
            || ((context == ExpandContext::TagsListFiles || context == ExpandContext::Tags)
                && unsafe { *fname } as c_int == '/' as c_int)
            || context == ExpandContext::Checkhealth
            || context == ExpandContext::Lsp
            || context == ExpandContext::Lua
        {
            return unsafe { xstrnsave(fname, len) };
        }

        // Custom expansion takes care of special things, and matches
        // backslashes literally.
        let custom = context == ExpandContext::UserDefined || context == ExpandContext::UserList;

        let mut new_len = len + 2; // +2 for '^' at start, NUL at end
        for i in 0..len {
            let c = unsafe { *fname.add(i) } as u8;
            // '*' needs to be replaced by ".*", '~' by "\~".
            if c == b'*' || c == b'~' {
                new_len += 1;
            }
            // Buffer names are like file names.  "." should be literal.
            if context == ExpandContext::Buffers && c == b'.' {
                new_len += 1;
            }
            if custom && c == b'\\' {
                new_len += 1;
            }
        }

        let retval = unsafe { xmalloc(new_len) } as *mut c_char;
        unsafe { *retval = '^' as c_char };
        let mut j: size_t = 1;
        let mut i: size_t = 0;
        while i < len {
            // Skip backslash.  But why?  At least keep it for custom
            // expansion.
            if !custom && unsafe { *fname.add(i) } as u8 == b'\\' {
                i += 1;
                if i == len {
                    break;
                }
            }

            'copy: {
                match unsafe { *fname.add(i) } as u8 {
                    b'*' => {
                        unsafe { *retval.add(j) = '.' as c_char };
                        j += 1;
                    }
                    b'~' => {
                        unsafe { *retval.add(j) = '\\' as c_char };
                        j += 1;
                    }
                    b'?' => {
                        // The one case that does not copy the source byte.
                        unsafe { *retval.add(j) = '.' as c_char };
                        break 'copy;
                    }
                    b'.' if context == ExpandContext::Buffers => {
                        unsafe { *retval.add(j) = '\\' as c_char };
                        j += 1;
                    }
                    b'\\' if custom => {
                        unsafe { *retval.add(j) = '\\' as c_char };
                        j += 1;
                    }
                    _ => {}
                }
                unsafe { *retval.add(j) = *fname.add(i) };
            }
            i += 1;
            j += 1;
        }
        unsafe { *retval.add(j) = NUL as c_char };
        return retval;
    }

    let retval = unsafe { xmalloc(len + 4) } as *mut c_char;
    unsafe { xmemcpyz(retval as *mut c_void, fname as *const c_void, len) };

    // Don't add a star to *, ~, ~user, $var or `cmd`.
    // * would become **, which walks the whole tree.
    // ~ would be at the start of the file name, but not the tail.
    // $ could be anywhere in the tail.
    // ` could be anywhere in the file name.
    // When the name ends in '$' don't add a star, remove the '$'.
    let tail = unsafe { path_tail(retval) };
    let mut ends_in_star = len > 0 && unsafe { *retval.add(len - 1) } as c_int == '*' as c_int;
    // An odd number of backslashes before it escapes the star.
    let mut k = len as ssize_t - 2;
    while k >= 0 {
        if unsafe { *retval.add(k as usize) } as c_int != '\\' as c_int {
            break;
        }
        ends_in_star = !ends_in_star;
        k -= 1;
    }
    if (unsafe { *retval } as c_int != '~' as c_int || tail != retval)
        && !ends_in_star
        && unsafe { vim_strchr(tail, '$' as c_int) }.is_null()
        && unsafe { vim_strchr(retval, '`' as c_int) }.is_null()
    {
        unsafe { *retval.add(len) = '*' as c_char };
        len += 1;
    } else if len > 0 && unsafe { *retval.add(len - 1) } as c_int == '$' as c_int {
        len -= 1;
    }
    unsafe { *retval.add(len) = NUL as c_char };
    retval
}
