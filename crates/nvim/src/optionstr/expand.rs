//! Command-line completion of a string option's value.
//!
//! Every entry point here has the same shape: the option table hands over an
//! [`OptExpand`] describing what the user has typed so far, and the
//! completer answers the matches. The three ways to produce them:
//!
//! - [`expand_set_opt_string`] over the accepted words the generated table
//!   already carries, filtered by the command line's regexp;
//! - [`expand_set_opt_listflag`] over a string of accepted flag letters, one
//!   match per letter the value does not already use;
//! - [`expand_set_opt_generic`] over an editor-side enumerator (highlight
//!   groups, encodings, autocommand events, …), which goes through
//!   `expand_generic`.
//!
//! All three optionally offer the option's *current* value as the first
//! completion, so that `<Tab>` on a bare `:set opt=` starts from what is
//! already there.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;
use std::ffi::CString;

use crate::autocmd::get_event_name_no_group;
use crate::cmdexpand::expand_generic;
use crate::global_cell::GlobalCell;
use crate::highlight_group::get_highlight_name;
use crate::mbyte::get_encoding_name;
use crate::memory::XString;
use crate::options::{
    kOptEventignore, kOptListchars, opt_dip_algorithm_values, opt_dip_inline_values, opt_ff_values,
};
use crate::types::{Candidate, CompleteListItemGetter, Expand, Failed, OptExpand};

use super::{
    COCU_ALL, CPO_VI, FO_ALL, MOUSE_ALL, SHM_ALL, WW_ALL, get_fillchars_name, get_listchars_name,
    opt_values, vim_regexec,
};

/// The option's current value, when the caller asked for it to be offered
/// and there is one.
fn original_value<'a>(args: &'a OptExpand<'_>) -> Option<&'a XString> {
    (args.include_orig_val && !args.value.is_empty()).then_some(&args.value)
}

/// The matches, or `Err` when there are none.
fn found_or_failed(found: Vec<XString>) -> Result<Vec<XString>, Failed> {
    if found.is_empty() {
        Err(Failed)
    } else {
        Ok(found)
    }
}

/// Complete an option whose accepted words the generated table lists.
pub(crate) fn expand_set_opt_string(
    args: &mut OptExpand<'_>,
    values: &[&CStr],
) -> Result<Vec<XString>, Failed> {
    let original = original_value(args).cloned();
    let mut found = Vec::with_capacity(values.len() + 1);
    if let Some(value) = &original {
        found.push(value.clone());
    }

    for entry in values {
        if entry.is_empty() {
            continue; // Ignore an empty accepted word.
        }
        // The current value is already the first completion; do not repeat
        // it.
        if original
            .as_ref()
            .is_some_and(|value| value.as_cstr() == *entry)
        {
            continue;
        }
        if vim_regexec(args.regmatch, entry, 0) {
            found.push(XString::from_cstr(entry));
        }
    }

    found_or_failed(found)
}

/// Complete an option whose accepted words the generated table lists, found
/// through the option's own index.
pub fn expand_set_str_generic(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    let values = opt_values(args.idx);
    expand_set_opt_string(args, values)
}

/// The option's current value, offered as completion index 0 ahead of
/// whatever the real enumerator produces.
static ORIGINAL_VALUE: GlobalCell<Option<XString>> = GlobalCell::new(None);

/// The real enumerator, for as long as `expand_generic` is running.
static ENUMERATOR: GlobalCell<Option<CompleteListItemGetter>> = GlobalCell::new(None);

/// The enumerator `expand_generic` sees: index 0 is the current value (or the
/// empty string, which `expand_generic` ignores), and everything above it is
/// the real enumerator shifted by one.
fn expand_set_opt_callback(expand: &Expand, idx: usize) -> Option<Candidate> {
    if idx == 0 {
        return Some(ORIGINAL_VALUE.with(|original| {
            original.as_ref().map_or(Candidate::Borrowed(c""), |value| {
                Candidate::Owned(value.as_cstr().to_owned())
            })
        }));
    }
    let next = ENUMERATOR.get().expect("enumerator set for the whole call");
    next(expand, idx - 1)
}

/// Complete an option from an editor-side enumerator rather than from a
/// fixed list.
pub(crate) fn expand_set_opt_generic(
    args: &mut OptExpand<'_>,
    func: CompleteListItemGetter,
) -> Result<Vec<XString>, Failed> {
    let original = args.include_orig_val.then(|| args.value.clone());
    ORIGINAL_VALUE.set(original);
    ENUMERATOR.set(Some(func));

    // Not fuzzy: ExpandContext::StringSetting does not use fuzzy matching.
    let found = expand_generic(c"", args.xp, args.regmatch, expand_set_opt_callback, false);

    ORIGINAL_VALUE.set(None);
    ENUMERATOR.set(None);
    Ok(found)
}

/// Complete an option that is a set of flag letters: one completion per
/// letter that is not already spoken for.
pub(crate) fn expand_set_opt_listflag(
    args: &mut OptExpand<'_>,
    flags: &CStr,
) -> Result<Vec<XString>, Failed> {
    let option_val = &args.value[..];
    let cmdline_val = args.typed();
    let original = original_value(args);

    let mut found = Vec::with_capacity(flags.count_bytes() + 1);
    if let Some(value) = original {
        found.push(value.clone());
    }

    for &flag in flags.to_bytes() {
        // With `+=`, a letter the value already carries cannot be added
        // again.
        if args.append && option_val.contains(&flag) {
            continue;
        }
        if cmdline_val.contains(&flag) {
            continue;
        }
        // A one-letter value is already the first completion; do not offer
        // the same letter twice.
        if original.is_some() && option_val == [flag] {
            continue;
        }
        found.push(XString::from_bytes(&[flag]));
    }

    found_or_failed(found)
}

/// Complete 'fillchars' or 'listchars'. Which one is decided by the variable
/// being set, since the two share every entry point.
pub fn expand_set_chars_option(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    // 'listchars' and 'fillchars' share this callback; which one is being
    // completed is the row, at either scope.
    let names = if args.idx == kOptListchars {
        get_listchars_name
    } else {
        get_fillchars_name
    };
    expand_set_opt_generic(args, names)
}

pub fn expand_set_concealcursor(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_listflag(args, COCU_ALL)
}

pub fn expand_set_cpoptions(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_listflag(args, CPO_VI)
}

pub fn expand_set_formatoptions(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_listflag(args, FO_ALL)
}

pub fn expand_set_mouse(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_listflag(args, MOUSE_ALL)
}

pub fn expand_set_shortmess(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_listflag(args, SHM_ALL)
}

pub fn expand_set_whichwrap(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_listflag(args, WW_ALL)
}

/// Complete 'diffopt', whose "algorithm:" and "inline:" fields each have
/// their own list of accepted words. Anything else after a `:` has no
/// completions at all.
pub fn expand_set_diffopt(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    // What stands between the start of the value and the pattern.
    let before = args
        .xp
        .line
        .get(args.set_arg..args.xp.pattern)
        .unwrap_or_default();
    if before.last() != Some(&b':') {
        return expand_set_str_generic(args);
    }
    if before.ends_with(b"algorithm:") {
        return expand_set_opt_string(args, &opt_dip_algorithm_values);
    }
    if before.ends_with(b"inline:") {
        return expand_set_opt_string(args, &opt_dip_inline_values);
    }
    Err(Failed)
}

pub fn expand_set_encoding(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_generic(args, get_encoding_name)
}

pub fn expand_set_winhighlight(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    expand_set_opt_generic(args, get_highlight_name)
}

/// Whether the option being completed is a window-local 'eventignorewin'
/// rather than the global 'eventignore', which decides which events are
/// eligible. `expand_generic` gives the enumerator no other way to know.
static WINDOW_EVENTS: GlobalCell<bool> = GlobalCell::new(false);

/// Enumerate the autocommand event names 'eventignore' accepts, with "all"
/// ahead of them, and each one prefixed by "-" when the user is subtracting.
pub(crate) fn get_eventignore_name(expand: &Expand, idx: usize) -> Option<Candidate> {
    let subtract = expand.pattern_starts_with(b"-");
    if !subtract && idx == 0 {
        return Some(Candidate::Borrowed(c"all"));
    }
    // Without the "-", index 0 was "all" above.
    let name = get_event_name_no_group(idx + usize::from(subtract) - 1, WINDOW_EVENTS.get())?;
    if !subtract {
        return Some(Candidate::Borrowed(name));
    }
    let mut text = Vec::with_capacity(name.count_bytes() + 1);
    text.push(b'-');
    text.extend_from_slice(name.to_bytes());
    Some(Candidate::Owned(
        CString::new(text).expect("an event name holds no NUL"),
    ))
}

pub fn expand_set_eventignore(args: &mut OptExpand<'_>) -> Result<Vec<XString>, Failed> {
    // 'eventignore' and 'eventignorewin' share this callback, and only the
    // second one completes the window events.
    WINDOW_EVENTS.set(args.idx != kOptEventignore);
    expand_set_opt_generic(args, get_eventignore_name)
}

/// Enumerate the values 'fileformat' accepts.
pub fn get_fileformat_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    opt_ff_values
        .get(idx)
        .map(|&value| Candidate::Borrowed(value))
}
