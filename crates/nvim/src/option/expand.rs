//! Command-line completion of option names and values.
//!
//! [`set_context_in_set_cmd`] is the parse: it works out which option the
//! cursor is inside, whether the cursor is on the name or on the value, and
//! for a value where the current item starts. What it decides is left in
//! the `expand_option_*` cells for the `Expand*` functions below, which the
//! command-line code calls back once it knows what kind of completion it
//! wants.
//!
//! Those cells are the state this module keeps between the two halves;
//! nothing else reads them.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::optionstr::OptString;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_uint};
use std::ffi::CString;

use crate::cmdexpand::{Scored, cmdline_fuzzy_complete, fuzzy_sorted};
use crate::cstr;
use crate::ex_docmd::state::ESCAPE_CHARS;
use crate::fuzzy::fuzzy_match_str;
use crate::global_cell::GlobalCell;
use crate::keycodes::special_key_code;
use crate::memory::XString;
use crate::options::{
    kOptAleph, kOptBackupdir, kOptCdpath, kOptCount, kOptDirectory, kOptFiletype, kOptInvalid,
    kOptKeymap, kOptPackpath, kOptPath, kOptRuntimepath, kOptSpellsuggest, kOptSyntax, kOptTags,
    kOptViewdir,
};
use crate::os::env::expand_env_esc;
use crate::regexp::vim_regexec;
use crate::strings::escaped_bytes;
use crate::types::{
    BackslashEscape, Expand, ExpandContext, Failed, MAXPATHL, OptExpand, OptIndex, OptionSetFlags,
    RegMatch, XpPrefix, size_t, uint32_t,
};

use super::{
    FUZZY_SCORE_NONE, find_option, find_option_len, get_option, get_varp_scope_from,
    is_option_hidden, kOptFlagColon, kOptFlagComma, kOptFlagExpand, kOptFlagFlagList,
    kOptValTypeBoolean, kOptValTypeNumber, option_has_type, option_value2string, option_var,
};

/// What [`set_context_in_set_cmd`] worked out, for the `Expand*` half.
///
/// `IDX` is `kOptInvalid` for a terminal option, whose two-letter name is
/// spelled into `NAME` instead — the whole five bytes at a time, since the
/// first two are always `t_` and the last is the terminator.
static IDX: GlobalCell<OptIndex> = GlobalCell::new(kOptInvalid);
static NAME: GlobalCell<[c_char; 5]> = GlobalCell::new([b't' as c_char, b'_' as c_char, 0, 0, 0]);
static START_COL: GlobalCell<usize> = GlobalCell::new(0);
static FLAGS: GlobalCell<OptionSetFlags> = GlobalCell::new(OptionSetFlags::NONE);
/// Whether the operator was `+=` or `^=`, which means the current value is
/// not a candidate to offer back.
static APPEND: GlobalCell<bool> = GlobalCell::new(false);

/// The option's value with `$VAR` and `~` expanded, or `None` when there
/// was nothing to expand.
///
/// Upstream expands into the shared `NameBuff` and answers its address;
/// every caller copies the answer out at once, so it is owned here.
///
/// # Safety
///
/// `val`, if not null, must be NUL-terminated.
pub(crate) unsafe fn option_expand(opt_idx: OptIndex, val: *const c_char) -> Option<CString> {
    let mut expanded = [0 as c_char; MAXPATHL as usize];
    // SAFETY: the option table is a plain array, `var` is the option's own
    // variable, and the caller's `val` is NUL-terminated.
    let opt = get_option(opt_idx);
    if opt.flags & kOptFlagExpand as uint32_t == 0 || is_option_hidden(opt_idx) {
        return None;
    }
    let val = if val.is_null() {
        option_var(opt_idx).string_var().value_ptr()
    } else {
        val
    };
    // The buffer the expansion lands in is `MAXPATHL` bytes.
    if val.is_null() || unsafe { cstr::bytes_at(val) }.len() > MAXPATHL as size_t {
        return None;
    }
    // 'path' and 'tags' hold escaped file names, so their separators
    // must survive the expansion.
    let esc = matches!(opt_idx, kOptTags | kOptPath);
    // 'spellsuggest' items are `file:<name>`; only that prefix's tail
    // is a path.
    let one_prefix = match opt_idx {
        kOptSpellsuggest => c"file:".as_ptr() as *mut c_char,
        _ => core::ptr::null_mut(),
    };
    unsafe { expand_env_esc(val, expanded.as_mut_ptr(), MAXPATHL, esc, false, one_prefix) };
    if unsafe { cstr::eq(expanded.as_ptr(), val) } {
        return None;
    }
    Some(cstr::in_chars(&expanded).to_owned())
}

/// Work out what the cursor is sitting on in a `:set` command line whose
/// argument starts at `arg`, and leave `expand` describing what to complete.
pub(crate) fn set_context_in_set_cmd(expand: &mut Expand, arg: usize, opt_flags: OptionSetFlags) {
    FLAGS.set(opt_flags);
    // The line as it stands while the context is worked out: cut at the
    // cursor.
    let line = expand.line.to_vec();
    let at = |i: usize| line.get(i).copied().unwrap_or(0);

    expand.context = ExpandContext::Settings;
    if at(arg) == 0 {
        expand.pattern = arg;
        return;
    }

    let argend = line[arg..]
        .iter()
        .position(|&c| c == 0)
        .map_or(line.len(), |end| arg + end);
    // A trailing unescaped space starts a fresh argument.
    let last = argend - 1;
    if at(last) == b' ' && (last == 0 || at(last - 1) != b'\\') {
        expand.pattern = last + 1;
        return;
    }

    // Walk back to the start of the argument the cursor is in: the
    // first space with an even number of backslashes before it.
    let mut p = last;
    while p > arg {
        let unescaped = if at(p) == b' ' || at(p) == b',' {
            backslashes_before(&line, arg, p) & 1 == 0
        } else {
            false
        };
        if at(p) == b' ' && unescaped {
            p += 1;
            break;
        }
        p -= 1;
    }

    for (spelling, prefix) in [(&b"no"[..], XpPrefix::No), (&b"inv"[..], XpPrefix::Inv)] {
        if line[p..].starts_with(spelling) {
            expand.context = ExpandContext::BoolSettings;
            expand.prefix = prefix;
            p += spelling.len();
            break;
        }
    }
    expand.pattern = p;
    let arg = p;

    let Some((nextchar, opt_idx, flags, is_term_option)) =
        take_option_name(expand, &line, arg, &mut p)
    else {
        return;
    };

    // `-=`, `+=` and `^=` complete like `=`, but the current value is
    // only worth offering back for `-=`.
    let mut nextchar = nextchar;
    APPEND.set(false);
    let mut subtract = false;
    if matches!(nextchar, b'-' | b'+' | b'^') && at(p + 1) == b'=' {
        subtract = nextchar == b'-';
        APPEND.set(matches!(nextchar, b'+' | b'^'));
        p += 1;
        nextchar = b'=';
    }
    if (nextchar != b'=' && nextchar != b':') || expand.context == ExpandContext::BoolSettings {
        expand.context = ExpandContext::Unsuccessful;
        return;
    }

    // Everything below completes the *value*, after the `=` or `:`.
    IDX.set(if is_term_option { kOptInvalid } else { opt_idx });
    expand.pattern = p + 1;
    START_COL.set(p + 1);

    // Three options reuse another command's completion wholesale.
    let borrowed = match opt_idx {
        kOptSyntax => Some(ExpandContext::Ownsyntax),
        kOptFiletype => Some(ExpandContext::Filetype),
        kOptKeymap => Some(ExpandContext::Keymap),
        _ => None,
    };
    if let Some(context) = borrowed {
        expand.context = context;
        return;
    }

    if subtract {
        expand.context = ExpandContext::SettingSubtract;
        return;
    } else if IDX.get() != kOptInvalid && get_option(IDX.get()).opt_expand_cb.is_some() {
        expand.context = ExpandContext::StringSetting;
    } else if at(expand.pattern) == 0 {
        expand.context = ExpandContext::OldSetting;
        return;
    } else {
        expand.context = ExpandContext::Nothing;
    }

    if is_term_option || option_has_type(opt_idx, kOptValTypeNumber) {
        return;
    }

    // Only string options from here.
    if flags & kOptFlagExpand as uint32_t != 0 {
        set_file_context(expand, opt_idx, flags);
    }
    if flags & (kOptFlagExpand | kOptFlagComma | kOptFlagColon) as uint32_t != 0 {
        seek_item_start(expand, &line, argend, flags);
    }
    // A set of one-letter flags has no words to complete, so the
    // pattern is always empty and the whole set is offered.
    if flags & kOptFlagFlagList as uint32_t != 0 {
        expand.pattern = argend;
    }
    // 'spellsuggest' takes `file:<name>`, whose tail is a file name.
    if opt_idx == kOptSpellsuggest {
        if line[expand.pattern..].starts_with(b"file:") {
            expand.pattern += 5;
        } else if get_option(IDX.get()).opt_expand_cb.is_some() {
            expand.context = ExpandContext::StringSetting;
        }
    }
}

/// How many backslashes immediately precede `at`, not counting past
/// `start`.
fn backslashes_before(line: &[u8], start: usize, at: usize) -> usize {
    line[start..at]
        .iter()
        .rev()
        .take_while(|&&c| c == b'\\')
        .count()
}

/// Consume the option name at `arg`, leaving `*p` on the character after
/// it. `None` means the cursor is still inside the name, so the name itself
/// is what to complete and `expand` has been left saying so.
///
/// Returns the character after the name, the option, its flags, and whether
/// it was one of the `t_xx` terminal names — which have no table row.
fn take_option_name(
    expand: &mut Expand,
    line: &[u8],
    arg: usize,
    p: &mut usize,
) -> Option<(u8, OptIndex, uint32_t, bool)> {
    let at = |i: usize| line.get(i).copied().unwrap_or(0);
    // `<t_xx>` and `<Key>` spellings.
    if at(arg) == b'<' {
        while at(*p) != b'>' {
            let c = at(*p);
            *p += 1;
            if c == 0 {
                return None;
            }
        }
        let name = CString::new(&line[arg + 1..*p]).unwrap_or_default();
        let key = special_key_code(name.as_bytes());
        if key == 0 {
            expand.context = ExpandContext::Nothing;
            return None;
        }
        *p += 1;
        let nextchar = at(*p);
        // The two termcap bytes the key code packs.
        let lo = (-key & 0xff) as c_char;
        let hi = ((-key) as c_uint >> 8 & 0xff) as c_char;
        NAME.set([b't' as c_char, b'_' as c_char, lo, hi, 0]);
        return Some((nextchar, kOptAleph, 0, true));
    }

    // A bare `t_xx` spelling.
    if at(*p) == b't' && at(*p + 1) == b'_' {
        *p += 2;
        if at(*p) != 0 {
            *p += 1;
        }
        if at(*p) == 0 {
            return None;
        }
        *p += 1;
        let nextchar = at(*p);
        NAME.set([
            b't' as c_char,
            b'_' as c_char,
            at(*p - 2) as c_char,
            at(*p - 1) as c_char,
            0,
        ]);
        return Some((nextchar, kOptAleph, 0, true));
    }

    // An ordinary name. `*` is allowed as a wildcard for the name
    // completion that follows.
    while at(*p).is_ascii_alphanumeric() || at(*p) == b'_' || at(*p) == b'*' {
        *p += 1;
    }
    if at(*p) == 0 {
        return None;
    }
    let nextchar = at(*p);
    let opt_idx = find_option_len(&line[arg..*p]);
    if opt_idx == kOptInvalid || is_option_hidden(opt_idx) {
        expand.context = ExpandContext::Nothing;
        return None;
    }
    // A boolean takes no value, so there is nothing after the name.
    if option_has_type(opt_idx, kOptValTypeBoolean) {
        expand.context = ExpandContext::Nothing;
        return None;
    }
    Some((nextchar, opt_idx, get_option(opt_idx).flags, false))
}

/// A `kOptFlagExpand` option's value is a file or directory name; say which,
/// and how many backslashes escape a space in it.
fn set_file_context(expand: &mut Expand, opt_idx: OptIndex, flags: uint32_t) {
    // 'path', 'cdpath' and 'tags' need three backslashes for a space,
    // because their own parsers unescape one layer first.
    let three = matches!(opt_idx, kOptPath | kOptCdpath | kOptTags);
    let directories = matches!(
        opt_idx,
        kOptBackupdir
            | kOptDirectory
            | kOptPath
            | kOptPackpath
            | kOptRuntimepath
            | kOptCdpath
            | kOptViewdir
    );
    expand.context = if directories {
        ExpandContext::Directories
    } else {
        ExpandContext::Files
    };
    expand.backslash = if three {
        BackslashEscape::THREE
    } else {
        BackslashEscape::ONE
    };
    if flags & kOptFlagComma as uint32_t != 0 {
        expand.backslash |= BackslashEscape::COMMA;
    }
}

/// Move `expand.pattern` forward to the start of the item the cursor is in,
/// for a value that is a list ending at `argend`.
fn seek_item_start(expand: &mut Expand, line: &[u8], argend: usize, flags: uint32_t) {
    let comma_list = flags & kOptFlagComma as uint32_t != 0;
    let colon_list = flags & kOptFlagColon as uint32_t != 0;

    let mut p = argend - 1;
    while p > expand.pattern {
        let c = line[p];
        let separator = c == b' ' || c == b',' || (c == b':' && colon_list);
        if separator {
            let bs = backslashes_before(line, expand.pattern, p);
            // A space only separates a triple-escaped value, a comma
            // needs fewer than two backslashes, and a colon in a
            // colon-list is never escaped.
            let splits = (c == b' ' && expand.backslash.has(BackslashEscape::THREE) && bs < 3)
                || (c == b',' && comma_list && bs < 2)
                || (c == b':' && colon_list);
            if splits {
                expand.pattern = p + 1;
                break;
            }
        }
        p -= 1;
    }
}

/// Complete an option *name*.
///
/// Every option the pattern matches, by its full name or (outside fuzzy
/// matching) its short one, plus `all` where a non-boolean name would do.
pub(crate) fn expand_settings(
    expand: &Expand,
    regmatch: &mut RegMatch,
    fuzzystr: &CStr,
    can_fuzzy: bool,
) -> Result<Vec<XString>, Failed> {
    let fuzzy = can_fuzzy && cmdline_fuzzy_complete(fuzzystr.to_bytes());
    let booleans_only = expand.context == ExpandContext::BoolSettings;
    let mut found = Vec::new();
    let mut scored = Vec::new();

    // Whether `name` matches; a match is kept in `found`, or scored into
    // `scored` when fuzzy. The two lists are parameters so the loop below
    // can still push to `found` itself.
    let try_name = |found: &mut Vec<XString>,
                    scored: &mut Vec<Scored>,
                    name: &CStr,
                    regmatch: &mut RegMatch|
     -> bool {
        if !fuzzy {
            if !vim_regexec(regmatch, name, 0) {
                return false;
            }
            found.push(XString::from_cstr(name));
            return true;
        }
        let score = fuzzy_match_str(name, fuzzystr);
        if score == FUZZY_SCORE_NONE {
            return false;
        }
        let idx = scored.len();
        scored.push(Scored {
            text: XString::from_cstr(name),
            score,
            idx,
        });
        true
    };

    // "all" is a `:set` keyword rather than an option, so it is only
    // offered where a non-boolean name would be.
    if !booleans_only {
        try_name(&mut found, &mut scored, c"all", regmatch);
    }

    for opt_idx in kOptAleph..kOptCount as OptIndex {
        let opt = get_option(opt_idx);
        if is_option_hidden(opt_idx)
            || (booleans_only && !option_has_type(opt_idx, kOptValTypeBoolean))
        {
            continue;
        }
        if try_name(&mut found, &mut scored, opt.fullname, regmatch) {
            continue;
        }
        if !fuzzy
            && let Some(short) = opt.shortname
            && vim_regexec(regmatch, short, 0)
        {
            // A short name matches, but what is offered is the full one.
            found.push(XString::from_cstr(opt.fullname));
        }
    }

    Ok(if fuzzy {
        fuzzy_sorted(scored, false)
    } else {
        found
    })
}

/// A value escaped the way the command line needs it back.
pub(crate) fn escape_option_str_cmdline(var: &CStr) -> XString {
    escaped_bytes(var, ESCAPE_CHARS)
}

/// Offer the option's current value as the one completion.
pub(crate) fn expand_old_setting() -> Result<Vec<XString>, Failed> {
    // A terminal option has no table row, so it is looked up by the
    // name `set_context_in_set_cmd` spelled out.
    if IDX.get() == kOptInvalid {
        IDX.set(NAME.with(|name| find_option(cstr::in_chars(name))));
    }
    let mut rendered = [0 as c_char; MAXPATHL as usize];
    let var = if IDX.get() == kOptInvalid {
        c""
    } else {
        option_value2string(IDX.get(), FLAGS.get(), &mut rendered);
        cstr::in_chars(&rendered)
    };
    Ok(vec![escape_option_str_cmdline(var)])
}

/// Complete a value through the option's own `opt_expand_cb`.
pub(crate) fn expand_string_setting(
    expand: &Expand,
    regmatch: &mut RegMatch,
) -> Result<Vec<XString>, Failed> {
    let opt_idx = IDX.get();
    if opt_idx == kOptInvalid {
        return Err(Failed);
    }
    let Some(expand_cb) = get_option(opt_idx).opt_expand_cb else {
        return Err(Failed);
    };

    let mut rendered = [0 as c_char; MAXPATHL as usize];
    option_value2string(opt_idx, FLAGS.get(), &mut rendered);
    let value = escape_option_str_cmdline(cstr::in_chars(&rendered));

    let set_arg = START_COL.get();
    let mut args = OptExpand {
        idx: opt_idx,
        value,
        append: APPEND.get(),
        // The current value is only worth offering back when nothing
        // has been typed yet and it is not being appended to.
        include_orig_val: !APPEND.get() && expand.line.get(set_arg).is_none_or(|&c| c == 0),
        regmatch,
        xp: expand,
        set_arg,
    };
    expand_cb(&mut args)
}

/// Complete a `-=` value: only what the option already holds can be
/// removed, so the candidates are its own items.
pub(crate) fn expand_setting_subtract(
    expand: &Expand,
    regmatch: &mut RegMatch,
) -> Result<Vec<XString>, Failed> {
    let opt_idx = IDX.get();
    if opt_idx == kOptInvalid || option_has_type(opt_idx, kOptValTypeNumber) {
        return expand_old_setting();
    }
    let (buf, win) = (Buf::current(), Win::current());
    let varp = get_varp_scope_from(opt_idx, FLAGS.get(), buf, win);
    let value = varp.string_var().get();
    let flags = get_option(opt_idx).flags;

    if flags & kOptFlagComma as uint32_t != 0 {
        if value.is_empty() {
            return Err(Failed);
        }
        let mut found = Vec::new();
        let mut rest = &value[..];
        loop {
            // An escaped comma is part of the item.
            let mut comma = None;
            let mut from = 0;
            while let Some(at) = rest[from..].iter().position(|&c| c == b',') {
                let at = from + at;
                if at != 0 && rest[at - 1] == b'\\' {
                    from = at + 1;
                    continue;
                }
                comma = Some(at);
                break;
            }
            let item = &rest[..comma.unwrap_or(rest.len())];
            if !item.is_empty() {
                let item = CString::new(item).unwrap_or_default();
                if vim_regexec(regmatch, &item, 0) {
                    found.push(escape_option_str_cmdline(&item));
                }
            }
            match comma {
                Some(at) => rest = &rest[at + 1..],
                None => break,
            }
        }
        return Ok(found);
    }

    if flags & kOptFlagFlagList as uint32_t != 0 {
        // A set of one-letter flags: offer the whole set first, then
        // each letter. Nothing may have been typed, since a flag set
        // has no word boundary to complete from.
        if !expand.pattern_is_empty() {
            return Err(Failed);
        }
        if value.is_empty() {
            return Err(Failed);
        }
        let mut found = vec![XString::from_bytes(&value)];
        if value.len() > 1 {
            found.extend(value.iter().map(|&flag| XString::from_bytes(&[flag])));
        }
        return Ok(found);
    }

    expand_old_setting()
}
