//! The wildcard key: one `<Tab>` press, from key to command line.
//!
//! [`nextwild`] is what the command-line key loop calls; it isolates the word
//! under the cursor, hands it to [`expand_one`] and puts the answer back.
//! [`expand_one`] owns the match list across presses — [`expand_one_start`]
//! fills it, [`next_match`] cycles it and [`longest_common_match`]
//! computes the `'wildmode'`=longest answer.

#![forbid(unsafe_code)]

use super::*;
use crate::ex_getln::{cursorcmd, redrawcmd};
use crate::getchar::beep_flush;
use crate::getchar::state::got_int;
use crate::lua::executor::nlua_expand_pat;
use crate::mbyte::char_at;
use crate::mbyte::{cluster_len, mb_tolower};
use crate::message::state::cmd_silent;
use crate::message::{e_toomany, emsg, msg_str};
use crate::message_fmt::msg_cstr;
use crate::option::vars::{p_fic, p_wic, p_wmnu};
use crate::options::kOptBoFlagWildmode;
use crate::os::cshim::gettext;
use crate::path::match_suffix_name;
use crate::popupmenu::state::pum_want;
use crate::popupmenu::{pum_clear, pum_get_height};
use crate::semsg;
use crate::types::ui::{kUICmdline, kUIWildmenu};
use crate::types::{ExpandContext, FAIL, OK, XpPrefix};
use crate::ui::{ui_flush, ui_has, vim_beep};
use crate::winlayer::Cc;
use core::ffi::{CStr, c_int};

/// The index [`expand_one`] starts the selection at: the first match, or -1
/// for "the original text" when the caller asked for nothing selected.
const fn first_selected(options: WildOpts) -> c_int {
    if options.has(WildOpts::NOSELECT) {
        -1
    } else {
        0
    }
}

/// A copy of `bytes` as an owned string. The command line and the matches
/// may in principle hold a NUL, which [`XString::from_bytes`] objects to in a
/// debug build; this keeps it, as upstream's copies did.
pub(crate) fn owned(bytes: &[u8]) -> XString {
    let mut text = XString::with_capacity(bytes.len());
    text.push_bytes(bytes);
    text
}

/// A match count as the `int` the rest of the editor counts in.
pub(crate) fn as_count(n: usize) -> c_int {
    c_int::try_from(n).expect("a match count fits a c_int")
}

/// Expand the word before the cursor on the command line.
///
/// Answers `FAIL` when this is not a context in which anything can be
/// completed, which tells the caller to pass the character through as a
/// normal character instead — that is what makes `:s/^I^D` work.  `OK` means
/// the key was consumed, even when there were no matches.
///
/// `mode` is passed on to [`expand_one`]; `escape` asks for the matches to
/// be escaped for use on the command line.
pub(crate) fn nextwild(
    expand: &mut Expand,
    mode: WildMode,
    options: WildOpts,
    escape: bool,
) -> c_int {
    let mut ccline = Cc::current();
    let from_wildtrigger_func = options.has(WildOpts::FUNC_TRIGGER);
    let wild_navigate = mode.navigates();

    if expand.matches.is_none() {
        pre_incsearch_pos.set(expand.pre_incsearch_pos);
        if ccline.input_fn != 0 && ccline.xp_context == ExpandContext::Commands {
            // Expand commands typed in the input() function.
            let line = ccline.text_bytes().to_vec();
            set_cmd_context(expand, &line, ccline.cmdpos, false);
        } else {
            may_expand_pattern.set(options.has(WildOpts::MAY_EXPAND_PATTERN));
            set_expand_context(expand);
            may_expand_pattern.set(false);
        }
        if expand.context == ExpandContext::Lua {
            nlua_expand_pat(expand);
        }
        cmd_showtail.set(expand_showtail(expand));
    }

    match expand.context {
        // Something illegal on the command line.
        ExpandContext::Unsuccessful => {
            beep_flush();
            return OK;
        }
        // The caller can use the character as a normal char instead.
        ExpandContext::Nothing => return FAIL,
        _ => {}
    }

    // Where the pattern starts within the command line: the context's own
    // copy of the line was taken from it, so the offset is the same.
    let at = as_count(expand.pattern);
    debug_assert!(ccline.cmdpos >= at);
    expand.pattern_len = usize::try_from(ccline.cmdpos - at).unwrap_or(0);

    // Skip showing matches if the prefix is invalid during wildtrigger().
    let context = expand.context;
    if from_wildtrigger_func && context == ExpandContext::Commands && expand.pattern_len == 0 {
        return FAIL;
    }

    // If 'cmd_silent' is set don't show the dots, because the redrawcmd()
    // below won't remove them.
    if !cmd_silent.get()
        && !from_wildtrigger_func
        && !wild_navigate
        && !(ui_has(kUICmdline) || ui_has(kUIWildmenu))
    {
        msg_str(c"..."); // show that we are busy
        ui_flush();
    }

    let mut p;
    if wild_navigate {
        // Get the next/previous match of an already expanded pattern.
        p = expand_one(expand, None, None, WildOpts::NONE, mode);
    } else {
        let typed = ccline
            .text_bytes()
            .get(expand.pattern..expand.pattern + expand.pattern_len)
            .unwrap_or_default()
            .to_vec();
        let pattern = if cmdline_fuzzy_completion_supported(expand)
            || expand.context == ExpandContext::PatternInBuf
        {
            // Don't modify the search string.
            owned(&typed)
        } else {
            addstar(&typed, expand.context)
        };
        // Translate the string into a pattern and expand it.
        let use_options = options
            | WildOpts::HOME_REPLACE
            | WildOpts::ADD_SLASH
            | WildOpts::SILENT
            | WildOpts::ESCAPE.when(escape)
            | WildOpts::ICASE.when(p_wic());
        p = expand_one(
            expand,
            Some(pattern.as_cstr()),
            Some(owned(&typed)),
            use_options,
            mode,
        );

        // Longest match: make sure it is not shorter than the literal
        // part of what was typed, which happens with :help.
        if mode == WildMode::Longest
            && let Some(text) = &p
        {
            let literal = typed
                .iter()
                .position(|&c| c == b'*' || c == b'?')
                .unwrap_or(typed.len());
            if text.len() < literal {
                p = None;
            }
        }
    }

    // Save the command line before inserting the selected item.
    if !wild_navigate && ccline.in_use() {
        cmdline_orig.set(Some(owned(ccline.text_bytes())));
    }

    if let Some(text) = &p
        && !got_int.get()
        && !options.has(WildOpts::NOSELECT)
    {
        // Replace what was typed with the match, the cursor moving with it.
        // A packed tag match is inserted only as far as its name.
        let text = text.as_cstr().to_bytes();
        let difflen = as_count(text.len()) - as_count(expand.pattern_len);
        let cursor = ccline.cmdpos;
        ccline.replace_range(at, cursor, text, difflen + 4);
        ccline.cmdpos += difflen;
    }

    redrawcmd();
    cursorcmd();

    // When expanding a ":map" command and no matches are found, assume
    // the key is supposed to be inserted literally.
    if expand.context == ExpandContext::Mappings && p.is_none() {
        return FAIL;
    }

    if expand.match_count() <= 0 && p.is_none() {
        beep_flush();
    } else if expand.match_count() == 1 && !options.has(WildOpts::NOSELECT) && !wild_navigate {
        // Only one match: free the expanded pattern again.
        expand_one(expand, None, None, WildOpts::NONE, WildMode::Free);
    }

    OK
}

/// Move the selection within an already expanded match list, and answer a
/// copy of what is now selected (or of the original text, at index -1).
fn next_match(mode: WildMode, expand: &mut Expand) -> Option<XString> {
    // When no matches were found there is nothing to move within.
    let count = as_count(expand.matches().len());
    if count == 0 {
        return None;
    }
    let mut findex = expand.selected;

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
        findex = if expand.orig.is_some() {
            -1 // return to the original text
        } else if findex < 0 {
            count - 1 // wrap around to the opposite end
        } else {
            0
        };
    }

    // Display the matches on screen.
    if p_wmnu() {
        if compl_match_array.with(Option::is_some) {
            compl_selected.set(findex);
            cmdline_pum_display(false);
        } else if cmdline_compl_use_pum(true) {
            cmdline_pum_create(Cc::current(), expand, None, cmd_showtail.get(), false);
            compl_selected.set(findex);
            pum_clear();
            cmdline_pum_display(true);
        } else {
            redraw_wildmenu(expand, expand.matches(), findex, cmd_showtail.get());
        }
    }

    expand.selected = findex;
    Some(match usize::try_from(findex) {
        Ok(at) => expand.matches()[at].clone(),
        Err(_) => expand.orig.clone().unwrap_or_default(),
    })
}

/// Run the expansion and take ownership of the matches.
///
/// Answers a copy of the first match for the modes that select one
/// (everything but `WildMode::All`, `WildMode::AllKeep` and
/// `WildMode::Longest`, which the caller assembles itself), and `None`
/// otherwise.
fn expand_one_start(
    mode: WildMode,
    expand: &mut Expand,
    pattern: &CStr,
    options: WildOpts,
) -> Option<XString> {
    let found = expand_from_context(expand, pattern, options);
    expand.found_any = found.as_ref().is_ok_and(|found| !found.is_empty());
    let Ok(mut found) = found else {
        // Upstream reports "No match" here under FNAME_ILLEGAL, which is
        // not defined on any platform this port builds for.
        expand.matches = Some(Vec::new());
        return None;
    };
    if found.is_empty() {
        expand.matches = Some(found);
        if !options.has(WildOpts::SILENT) {
            let arg0 = msg_cstr(pattern);
            semsg!("E480: No match: {arg0}");
        }
        return None;
    }

    // Escape the matches for use on the command line.
    escape_matches(expand, pattern.to_bytes(), &mut found, options);
    expand.matches = Some(found);

    if mode == WildMode::All || mode == WildMode::AllKeep || mode == WildMode::Longest {
        return None;
    }

    // Check for matching suffixes in file names.  (Upstream's
    // `xp_numfiles ? xp_numfiles : 1` can only take the first arm here:
    // the zero case returned above.)
    let matches = expand.matches();
    let mut non_suf_match = matches.len();
    let names = matches!(
        expand.context,
        ExpandContext::Files | ExpandContext::Directories
    );
    if names && matches.len() > 1 {
        // More than one match; check the suffix.  expand_wildcards has
        // sorted the ones with a matching suffix to the front, so only
        // the first two need looking at.
        non_suf_match = matches[..2]
            .iter()
            .filter(|name| match_suffix_name(name.as_cstr()))
            .count();
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
        return None;
    }
    Some(expand.matches()[0].clone())
}

/// The longest common prefix of the matches — the `'wildmode'`=longest answer.
///
/// Beeps (unless `WildOpts::NO_BEEP`) at the byte where they first diverge, which
/// is how the user learns the expansion stopped short of a whole name.
fn longest_common_match(expand: &Expand, options: WildOpts) -> XString {
    let files = expand.matches();
    let first = files[0].as_cstr().to_bytes();
    // 'fileignorecase' folds case, but only where the matches are names
    // that came from the filesystem or the buffer list.  Neither operand
    // can change inside the loop.
    let fold = p_fic()
        && matches!(
            expand.context,
            ExpandContext::Directories
                | ExpandContext::Files
                | ExpandContext::ShellCmd
                | ExpandContext::Buffers
        );

    let mut len = 0;
    while len < first.len() {
        let mb_len = cluster_len(&first[len..]);
        let c0 = char_at(&first[len..]);
        let diverged = files[1..].iter().any(|name| {
            let ci = char_at(name.get(len..).unwrap_or_default());
            if fold {
                mb_tolower(c0) != mb_tolower(ci)
            } else {
                c0 != ci
            }
        });
        if diverged {
            if !options.has(WildOpts::NO_BEEP) {
                vim_beep(kOptBoFlagWildmode);
            }
            break;
        }
        len += mb_len;
    }

    owned(&first[..len])
}

/// Do wildcard expansion on `pattern`.
///
/// Chars that should not be expanded must be preceded with a backslash.
/// Answers the new string, or `None` for failure.
///
/// `orig` is the originally expanded string. It is either kept in
/// [`Expand::orig`] or dropped here.  With `mode` `WildMode::Next` or
/// `WildMode::Prev` it should be `None`.
///
/// Results are cached in [`Expand::matches`], except when `mode` is
/// `WildMode::ExpandFree` or `WildMode::All`.
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
/// [`Expand::context`] and [`Expand::backslash`] must have been set.
pub fn expand_one(
    expand: &mut Expand,
    pattern: Option<&CStr>,
    orig: Option<XString>,
    options: WildOpts,
    mode: WildMode,
) -> Option<XString> {
    // First handle the case of using an old match.
    if mode.navigates() {
        return next_match(mode, expand);
    }

    // The original text, for the two modes that answer with it.
    let mut ss = match mode {
        WildMode::Cancel => Some(expand.orig.clone().unwrap_or_default()),
        WildMode::Apply => Some(match usize::try_from(expand.selected) {
            Ok(at) => expand.matches()[at].clone(),
            Err(_) => expand.orig.clone().unwrap_or_default(),
        }),
        _ => None,
    };

    // Free the old names.
    if expand.matches.is_some() && mode != WildMode::All && mode != WildMode::Longest {
        // The entries may be in the popup menu; remove it before they go.
        if compl_match_array.with(Option::is_some) {
            cmdline_pum_remove(false);
        }
        expand.matches = None;
        expand.orig = None;
    }
    expand.selected = first_selected(options);

    if mode == WildMode::Free {
        // Only release the file names.
        return None;
    }

    if expand.matches.is_none() && mode != WildMode::Apply && mode != WildMode::Cancel {
        expand.orig = orig;
        ss = expand_one_start(mode, expand, pattern.unwrap_or(c""), options);
    }

    // Find the longest common part.
    if mode == WildMode::Longest && !expand.matches().is_empty() {
        ss = Some(longest_common_match(expand, options));
        expand.selected = -1; // next 'wildchar' gets the first one
    }

    // Concatenate all matching names.  Unless interrupted this can be
    // slow, and the result probably won't be used.
    if mode == WildMode::All && !expand.matches().is_empty() && !got_int.get() {
        let suffix: &[u8] = if options.has(WildOpts::USE_NL) {
            b"\n"
        } else {
            b" "
        };
        // A boolean option's matches are listed as "novimfile" /
        // "invvimfile"; the prefix goes *between* the entries, so there
        // is one fewer of it than there are matches.
        let prefix: &[u8] = match expand.prefix {
            XpPrefix::No => b"no",
            XpPrefix::Inv => b"inv",
            XpPrefix::None => b"",
        };
        let files = expand.matches();
        let mut joined = XString::new();
        for (i, name) in files.iter().enumerate() {
            if i > 0 {
                joined.push_bytes(prefix);
            }
            joined.push_cstr(name.as_cstr());
            if i + 1 < files.len() {
                joined.push_bytes(suffix);
            }
        }
        ss = Some(joined);
    }

    if mode == WildMode::ExpandFree || mode == WildMode::All {
        expand_cleanup(expand);
    }

    ss
}

/// Clean up an expand structure after use.
pub fn expand_cleanup(expand: &mut Expand) {
    expand.matches = None;
    expand.orig = None;
}

/// Drop the saved copy of the command line taken before the last expansion.
pub fn clear_cmdline_orig() {
    cmdline_orig.set(None);
}

/// One expansion of `pattern` in `context` with `first`, then each of `then`
/// on the match list it left: every step's answer, and the selection after
/// it. The oracle's way into [`expand_one`].
#[cfg(test)]
pub(super) fn expand_one_walk(
    context: ExpandContext,
    pattern: &[u8],
    orig: &[u8],
    options: WildOpts,
    first: WildMode,
    then: &[WildMode],
) -> Vec<(Option<Vec<u8>>, c_int)> {
    let mut xpc = Expand::new();
    xpc.context = context;
    xpc.line = owned(pattern);
    let pat = owned(pattern);
    let mut steps = Vec::new();
    let p = expand_one(
        &mut xpc,
        Some(pat.as_cstr()),
        Some(owned(orig)),
        options,
        first,
    );
    steps.push((p.map(|p| p.to_vec()), xpc.selected));
    for &mode in then {
        let p = expand_one(&mut xpc, None, None, options, mode);
        steps.push((p.map(|p| p.to_vec()), xpc.selected));
    }
    expand_cleanup(&mut xpc);
    steps
}
