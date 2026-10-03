//! `:set nowildmenu` output: the match list printed as messages.
//!
//! [`showmatches`] lays the matches out in columns and prints them with
//! [`showmatches_oneline`]; [`expand_showtail`] decides whether a file match
//! is shown as its tail alone.  [`addstar`] is here because it is the other
//! half of the same question — what the pattern looked like before the
//! matches were found.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::vim_strsize_cstr;
use crate::cstr;
use crate::drawscreen::state::cmdline_row;
use crate::getchar::state::got_int;
use crate::highlight_group::{HLF_D, HLF_T};
use crate::lua::executor::nlua_expand_pat;
use crate::message::state::{msg_col, msg_didany, msg_row};
use crate::message::{
    msg_advance, msg_clr_eos, msg_display, msg_display_elided, msg_ext_set_kind, msg_putchar,
    msg_start, msg_str, msg_str_hl,
};
use crate::os::cshim::gettext;
use crate::os::env::{expand_env_save_opt_of, home_replace_in};
use crate::os::fs::dir_exists;
use crate::path::{tail_index, vim_ispathsep};
use crate::popupmenu::pum_clear;
use crate::types::ui::kUIMessages;
use crate::types::{ExpandContext, MAXPATHL};
use crate::ui::state::Columns;
use crate::ui::{ui_flush, ui_has};
use crate::winlayer::Cc;
use core::ffi::c_int;

/// Whether the match is a name the listing highlights when it is a
/// directory, and shows `~` in.
fn lists_paths(expand: &Expand) -> bool {
    matches!(
        expand.context,
        ExpandContext::Files | ExpandContext::ShellCmd | ExpandContext::Buffers
    )
}

/// `name` with each backslash that escapes something taken out: upstream's
/// `backslash_halve`.
fn backslash_halved(name: &[u8]) -> XString {
    let mut out = Vec::with_capacity(name.len());
    let mut p = 0;
    while p < name.len() {
        if name[p] == b'\\' && p + 1 < name.len() {
            p += 1;
        }
        out.push(name[p]);
        p += 1;
    }
    owned(&out)
}

/// One line of the match listing.
///
/// `matches[linenr]`, `matches[linenr + lines]`, … are the entries that share
/// a line; `maxlen` is the column width and `showtail` asks for file names to
/// be shown as their tail alone.
pub(crate) fn showmatches_oneline(
    expand: &Expand,
    matches: &[XString],
    lines: c_int,
    linenr: c_int,
    maxlen: c_int,
    showtail: bool,
) {
    let num_matches = as_count(matches.len());
    // C's SHOW_MATCH().
    let show_match = |m: &XString| -> XString {
        let tail = if showtail {
            showmatches_gettail(m, false)
        } else {
            0
        };
        owned(&m[tail..])
    };

    let mut lastlen = 999;
    let mut j = linenr;
    while j < num_matches {
        let m = &matches[usize::try_from(j).unwrap_or(0)];
        if expand.context == ExpandContext::TagsListFiles {
            // The tag's kind and file follow its name, each after a NUL,
            // which is how `ExpandContext::TagsListFiles` packs the three.
            let name = m.as_cstr();
            msg_display(name, HLF_D, false);
            let kind_at = name.count_bytes() + 1;
            msg_advance(maxlen + 1);
            msg_str(m.cstr_at(kind_at));
            msg_advance(maxlen + 3);
            msg_display_elided(m.cstr_at(kind_at + 2), HLF_D);
            break;
        }
        for _ in 0..(maxlen - lastlen).max(0) {
            msg_putchar(c_int::from(b' '));
        }
        let (isdir, shown) = if lists_paths(expand) {
            // Highlight directories.
            let isdir = if expand.matches.is_some() {
                // Expansion was done before and special characters were
                // escaped, need to halve backslashes.  Also $HOME has been
                // replaced with ~/.
                let path = expand_env_save_opt_of(m.as_cstr(), true);
                dir_exists(backslash_halved(&path).as_cstr())
            } else {
                // Expansion was done here, file names are literal.
                dir_exists(m.as_cstr())
            };
            let shown = if showtail {
                show_match(m)
            } else {
                home_replace_in(None, m.as_cstr(), MAXPATHL as usize, true)
            };
            (isdir, shown)
        } else {
            (false, show_match(m))
        };
        lastlen = msg_display(shown.as_cstr(), if isdir { HLF_D } else { 0 }, false);
        j += lines;
    }
    if msg_col.get() > 0 {
        // When not wrapped around.
        msg_clr_eos();
        msg_putchar(c_int::from(b'\n'));
    }
}

/// Display completion matches.
///
/// Answers `Expanded::Nothing` when the character that triggered expansion
/// should be inserted as a normal character.
pub fn showmatches(
    expand: &mut Expand,
    display_wildmenu: bool,
    display_list: bool,
    noselect: bool,
) -> Expanded {
    let ccline = Cc::current();
    // The matches when nothing was expanded yet: found here, for this
    // listing only.
    let mut found = None;
    let showtail;

    if expand.matches.is_none() {
        set_expand_context(expand);
        if expand.context == ExpandContext::Lua {
            nlua_expand_pat(expand);
        }
        let (retval, matches) = expand_cmdline(expand, ccline.cmdpos);
        if retval != Expanded::Ok {
            return retval;
        }
        found = Some(matches);
        showtail = expand_showtail(expand);
    } else {
        showtail = cmd_showtail.get();
    }

    if cmdline_compl_use_pum(display_wildmenu && !display_list) {
        cmdline_pum_create(Cc::current(), expand, found, showtail, noselect);
        compl_selected.set(if noselect { -1 } else { 0 });
        pum_clear();
        cmdline_pum_display(true);
        return Expanded::Ok;
    }

    let matches = found.as_deref().unwrap_or(expand.matches());

    if display_list {
        msg_didany.set(false); // lines_left will be set
        msg_start(); // prepare for paging
        if !ui_has(kUIMessages) {
            msg_putchar(c_int::from(b'\n'));
        }
        ui_flush();
        cmdline_row.set(msg_row.get());
        msg_didany.set(false); // lines_left will be set again
        msg_ext_set_kind(c"wildlist");
        msg_start(); // prepare for paging
    }

    if got_int.get() {
        got_int.set(false); // only interrupt the completion, not the cmd line
    } else if display_wildmenu && !display_list {
        // Display statusbar menu.
        redraw_wildmenu(expand, matches, if noselect { -1 } else { 0 }, showtail);
    } else if display_list {
        // C's SHOW_MATCH().
        let show_match = |m: &XString| -> XString {
            let tail = if showtail {
                showmatches_gettail(m, false)
            } else {
                0
            };
            XString::from_cstr(cstr::suffix(m.as_cstr(), tail))
        };

        // Find the length of the longest file name.
        let mut maxlen = 0;
        for m in matches {
            let len = if !showtail && lists_paths(expand) {
                let shown = home_replace_in(None, m.as_cstr(), MAXPATHL as usize, true);
                vim_strsize_cstr(shown.as_cstr())
            } else {
                vim_strsize_cstr(show_match(m).as_cstr())
            };
            maxlen = maxlen.max(len);
        }

        let num_matches = as_count(matches.len());
        let lines = if expand.context == ExpandContext::TagsListFiles {
            num_matches
        } else {
            // Compute the number of columns and lines for the listing.
            maxlen += 2; // two spaces between file names
            let columns = ((Columns.get() + 2) / maxlen).max(1);
            (num_matches + columns - 1) / columns
        };

        if expand.context == ExpandContext::TagsListFiles {
            msg_str_hl(gettext(c"tagname"), HLF_T, false);
            msg_clr_eos();
            msg_advance(maxlen - 3);
            msg_str_hl(gettext(c" kind file\n"), HLF_T, false);
        }

        // List the files line by line.
        for i in 0..lines {
            showmatches_oneline(expand, matches, lines, i, maxlen, showtail);
            if got_int.get() {
                got_int.set(false);
                break;
            }
        }

        // We redraw the command below the lines that we have just listed.
        // This is a bit tricky, but it saves a lot of screen updating.
        cmdline_row.set(msg_row.get()); // will put it back later
    }

    Expanded::Ok
}

/// `path_tail` for [`showmatches`] and [`redraw_wildmenu`]: where the tail of
/// file name path `s` starts, ignoring a trailing `/`.
///
/// `eager` takes the text after the last separator even when it is empty.
pub(crate) fn showmatches_gettail(s: &[u8], eager: bool) -> usize {
    let mut t = 0;
    let mut had_sep = false;

    let mut p = 0;
    while p < s.len() && s[p] != 0 {
        if vim_ispathsep(c_int::from(s[p])) {
            if eager {
                t = p + 1;
            } else {
                had_sep = true;
            }
        } else if had_sep {
            t = p;
            had_sep = false;
        }
        p += char_len(s, p);
    }
    t
}

/// True if we only need to show the tail of completion matches.
///
/// When not completing file names, or when there is a wildcard in the path,
/// false is returned.
pub(crate) fn expand_showtail(expand: &Expand) -> bool {
    // When not completing file names a "/" may mean something different.
    if !matches!(
        expand.context,
        ExpandContext::Files | ExpandContext::ShellCmd | ExpandContext::Directories
    ) {
        return false;
    }

    let pattern = expand.pattern_text();
    let end = tail_index(pattern);
    if end == 0 {
        // There is no path separator.
        return false;
    }

    let mut s = 0;
    while s < end {
        // Skip escaped wildcards.  Only when the backslash is not a path
        // separator, on DOS the '*' "path\*\file" must not be skipped.
        if pattern[s] == b'\\' && s + 1 < pattern.len() {
            s += 1;
        } else if b"*?[".contains(&pattern[s]) {
            return false;
        }
        s += 1;
    }
    true
}

/// Prepare a string for expansion.
///
/// When expanding file names the string will be used with
/// `expand_wildcards()`: `fname` is copied and a `*` is added at the end.
/// When expanding other names it will be used with `vim_regcomp()`: the name
/// is copied and `^` prepended, with the file-matching wildcards converted to
/// regexp ones.
///
/// `context` is the `EXPAND_*` the pattern came from.
pub fn addstar(fname: &[u8], context: ExpandContext) -> XString {
    let len = fname.len();
    if !matches!(
        context,
        ExpandContext::Files
            | ExpandContext::FilesInPath
            | ExpandContext::ShellCmd
            | ExpandContext::Directories
            | ExpandContext::DirsInCdpath
    ) {
        // Matching will be done internally (on something other than
        // files).  So we convert the file-matching-type wildcards into our
        // kind for use with vim_regcomp().

        // For help tags the translation is done in find_help_tags().
        // For a tag pattern starting with "/" no translation is needed.
        if matches!(
            context,
            ExpandContext::Findfunc
                | ExpandContext::Help
                | ExpandContext::Colors
                | ExpandContext::Compiler
                | ExpandContext::Ownsyntax
                | ExpandContext::Filetype
                | ExpandContext::Keymap
                | ExpandContext::Packadd
                | ExpandContext::Runtime
                | ExpandContext::Checkhealth
                | ExpandContext::Lsp
                | ExpandContext::Lua
        ) || (matches!(context, ExpandContext::TagsListFiles | ExpandContext::Tags)
            && fname.first() == Some(&b'/'))
        {
            return owned(fname);
        }

        // Custom expansion takes care of special things, and matches
        // backslashes literally.
        let custom = context == ExpandContext::UserDefined || context == ExpandContext::UserList;

        let mut out = Vec::with_capacity(len + 2);
        out.push(b'^');
        let mut i = 0;
        while i < len {
            // Skip backslash.  But why?  At least keep it for custom
            // expansion.
            if !custom && fname[i] == b'\\' {
                i += 1;
                if i == len {
                    break;
                }
            }

            match fname[i] {
                b'*' => out.push(b'.'),
                b'~' => out.push(b'\\'),
                b'?' => {
                    // The one case that does not copy the source byte.
                    out.push(b'.');
                    i += 1;
                    continue;
                }
                b'.' if context == ExpandContext::Buffers => out.push(b'\\'),
                b'\\' if custom => out.push(b'\\'),
                _ => {}
            }
            out.push(fname[i]);
            i += 1;
        }
        return owned(&out);
    }

    let mut out = fname.to_vec();

    // Don't add a star to *, ~, ~user, $var or `cmd`.
    // * would become **, which walks the whole tree.
    // ~ would be at the start of the file name, but not the tail.
    // $ could be anywhere in the tail.
    // ` could be anywhere in the file name.
    // When the name ends in '$' don't add a star, remove the '$'.
    let tail = tail_index(&out);
    let mut ends_in_star = out.last() == Some(&b'*');
    // An odd number of backslashes before it escapes the star.
    for &c in out[..len.saturating_sub(1)].iter().rev() {
        if c != b'\\' {
            break;
        }
        ends_in_star = !ends_in_star;
    }
    if (out.first() != Some(&b'~') || tail != 0)
        && !ends_in_star
        && !out[tail..].contains(&b'$')
        && !out.contains(&b'`')
    {
        out.push(b'*');
    } else if out.last() == Some(&b'$') {
        out.pop();
    }
    owned(&out)
}
