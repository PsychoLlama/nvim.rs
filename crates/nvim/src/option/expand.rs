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

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;
use core::slice;
use std::ffi::CString;

use crate::cmdexpand::cmdline_fuzzy_complete;
use crate::cmdexpand::fuzzymatches_to_strmatches;
use crate::cstr;
use crate::ex_docmd::state::escape_chars;
use crate::fuzzy::fuzzy_match_str;
use crate::garray::{ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::keycodes::get_special_key_code;
use crate::memory::{xfree, xmalloc, xmemdupz, xstrdup};
use crate::options::{
    kOptAleph, kOptBackupdir, kOptCdpath, kOptCount, kOptDirectory, kOptFiletype, kOptInvalid,
    kOptKeymap, kOptPackpath, kOptPath, kOptRuntimepath, kOptSpellsuggest, kOptSyntax, kOptTags,
    kOptViewdir,
};
use crate::os::env::expand_env_esc;
use crate::regexp::vim_regexec;
use crate::strings::{vim_strchr, vim_strsave_escaped};
use crate::types::{
    BackslashEscape, ColNr, Expand, ExpandContext, Failed, FuzMatchStr, GArray, MAXPATHL, NUL,
    OptExpand, OptIndex, OptionSetFlags, RegMatch, XpPrefix, size_t, uint32_t,
};
use crate::winlayer::Live;

use super::{
    FUZZY_SCORE_NONE, XP_PREFIX_INV, XP_PREFIX_NO, find_option, find_option_len, get_option,
    get_varp_scope_from, is_option_hidden, kOptFlagColon, kOptFlagComma, kOptFlagExpand,
    kOptFlagFlagList, kOptValTypeBoolean, kOptValTypeNumber, option_has_type, option_value2string,
    option_var,
};

/// What [`set_context_in_set_cmd`] worked out, for the `Expand*` half.
///
/// `IDX` is `kOptInvalid` for a terminal option, whose two-letter name is
/// spelled into `NAME` instead — the whole five bytes at a time, since the
/// first two are always `t_` and the last is the terminator.
static IDX: GlobalCell<OptIndex> = GlobalCell::new(kOptInvalid);
static NAME: GlobalCell<[c_char; 5]> = GlobalCell::new([b't' as c_char, b'_' as c_char, 0, 0, 0]);
static START_COL: GlobalCell<c_int> = GlobalCell::new(0);
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
        unsafe { option_var(opt_idx).string_var().get() }
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
        _ => ptr::null_mut(),
    };
    unsafe { expand_env_esc(val, expanded.as_mut_ptr(), MAXPATHL, esc, false, one_prefix) };
    if unsafe { cstr::eq(expanded.as_ptr(), val) } {
        return None;
    }
    Some(cstr::in_chars(&expanded).to_owned())
}

/// Work out what the cursor is sitting on in a `:set` command line, and
/// leave `expand` describing what to complete.
///
/// # Safety
///
/// `expand` must be the command line's expansion state and `arg` a
/// NUL-terminated cursor into `expand.xp_line`.
pub(crate) unsafe fn set_context_in_set_cmd(
    expand: *mut Expand,
    arg: *mut c_char,
    opt_flags: OptionSetFlags,
) {
    FLAGS.set(opt_flags);

    // SAFETY: the caller's expansion state and command line.
    unsafe { (*expand).xp_context = ExpandContext::Settings };
    if unsafe { *arg } == NUL as c_char {
        unsafe { (*expand).xp_pattern = arg };
        return;
    }

    let argend = unsafe { arg.add(cstr::bytes_at(arg).len()) };
    // A trailing unescaped space starts a fresh argument.
    let last = unsafe { argend.sub(1) };
    if unsafe { *last } as c_int == ' ' as c_int
        && unsafe { *last.sub(1) } as c_int != '\\' as c_int
    {
        unsafe { (*expand).xp_pattern = last.add(1) };
        return;
    }

    // Walk back to the start of the argument the cursor is in: the
    // first space with an even number of backslashes before it.
    let mut p = last;
    while p > arg {
        let unescaped =
            if unsafe { *p } as c_int == ' ' as c_int || unsafe { *p } as c_int == ',' as c_int {
                (unsafe { backslashes_before(arg, p) }) & 1 == 0
            } else {
                false
            };
        if unsafe { *p } as c_int == ' ' as c_int && unescaped {
            p = unsafe { p.add(1) };
            break;
        }
        p = unsafe { p.sub(1) };
    }

    for (spelling, prefix) in [
        (c"no", XP_PREFIX_NO as XpPrefix),
        (c"inv", XP_PREFIX_INV as XpPrefix),
    ] {
        let len = spelling.count_bytes();
        if unsafe { cstr::prefix_eq(p, spelling.as_ptr(), len) } {
            unsafe { (*expand).xp_context = ExpandContext::BoolSettings };
            unsafe { (*expand).xp_prefix = prefix };
            p = unsafe { p.add(len) };
            break;
        }
    }
    unsafe { (*expand).xp_pattern = p };
    let arg = p;

    let Some((nextchar, opt_idx, flags, is_term_option)) =
        (unsafe { take_option_name(expand, arg, &mut p) })
    else {
        return;
    };

    // `-=`, `+=` and `^=` complete like `=`, but the current value is
    // only worth offering back for `-=`.
    let mut nextchar = nextchar;
    APPEND.set(false);
    let mut subtract = false;
    if matches!(nextchar as u8, b'-' | b'+' | b'^') && unsafe { *p.add(1) } as c_int == '=' as c_int
    {
        subtract = nextchar as u8 == b'-';
        APPEND.set(matches!(nextchar as u8, b'+' | b'^'));
        p = unsafe { p.add(1) };
        nextchar = '=' as c_char;
    }
    if (nextchar as c_int != '=' as c_int && nextchar as c_int != ':' as c_int)
        || unsafe { (*expand).xp_context } == ExpandContext::BoolSettings
    {
        unsafe { (*expand).xp_context = ExpandContext::Unsuccessful };
        return;
    }

    // Everything below completes the *value*, after the `=` or `:`.
    IDX.set(if is_term_option { kOptInvalid } else { opt_idx });
    unsafe { (*expand).xp_pattern = p.add(1) };
    START_COL.set(unsafe { p.add(1).offset_from((*expand).xp_line) } as c_int);

    // Three options reuse another command's completion wholesale.
    let borrowed = match opt_idx {
        kOptSyntax => Some(ExpandContext::Ownsyntax),
        kOptFiletype => Some(ExpandContext::Filetype),
        kOptKeymap => Some(ExpandContext::Keymap),
        _ => None,
    };
    if let Some(context) = borrowed {
        unsafe { (*expand).xp_context = context };
        return;
    }

    if subtract {
        unsafe { (*expand).xp_context = ExpandContext::SettingSubtract };
        return;
    } else if IDX.get() != kOptInvalid && get_option(IDX.get()).opt_expand_cb.is_some() {
        unsafe { (*expand).xp_context = ExpandContext::StringSetting };
    } else if unsafe { *(*expand).xp_pattern } == NUL as c_char {
        unsafe { (*expand).xp_context = ExpandContext::OldSetting };
        return;
    } else {
        unsafe { (*expand).xp_context = ExpandContext::Nothing };
    }

    if is_term_option || option_has_type(opt_idx, kOptValTypeNumber) {
        return;
    }

    // Only string options from here.
    if flags & kOptFlagExpand as uint32_t != 0 {
        unsafe { set_file_context(expand, opt_idx, flags) };
    }
    if flags & (kOptFlagExpand | kOptFlagComma | kOptFlagColon) as uint32_t != 0 {
        unsafe { seek_item_start(expand, argend, flags) };
    }
    // A set of one-letter flags has no words to complete, so the
    // pattern is always empty and the whole set is offered.
    if flags & kOptFlagFlagList as uint32_t != 0 {
        unsafe { (*expand).xp_pattern = argend };
    }
    // 'spellsuggest' takes `file:<name>`, whose tail is a file name.
    if opt_idx == kOptSpellsuggest {
        if unsafe { cstr::starts_with((*expand).xp_pattern, b"file:") } {
            unsafe { (*expand).xp_pattern = (*expand).xp_pattern.add(5) };
        } else if get_option(IDX.get()).opt_expand_cb.is_some() {
            unsafe { (*expand).xp_context = ExpandContext::StringSetting };
        }
    }
}

/// How many backslashes immediately precede `at`, not counting past
/// `start`.
///
/// # Safety
///
/// `start..=at` must be one string.
unsafe fn backslashes_before(start: *const c_char, at: *const c_char) -> isize {
    let mut s = at;
    // SAFETY: the caller's span.
    while s > start && unsafe { *s.sub(1) } as c_int == '\\' as c_int {
        s = unsafe { s.sub(1) };
    }
    unsafe { at.offset_from(s) }
}

/// Consume the option name at `arg`, leaving `*p` on the character after
/// it. `None` means the cursor is still inside the name, so the name itself
/// is what to complete and `expand` has been left saying so.
///
/// Returns the character after the name, the option, its flags, and whether
/// it was one of the `t_xx` terminal names — which have no table row.
///
/// # Safety
///
/// `expand` must be the expansion state and `arg` a NUL-terminated cursor.
unsafe fn take_option_name(
    expand: *mut Expand,
    arg: *mut c_char,
    p: &mut *mut c_char,
) -> Option<(c_char, OptIndex, uint32_t, bool)> {
    // SAFETY: the caller's command line and expansion state.
    // `<t_xx>` and `<Key>` spellings.
    if unsafe { *arg } as c_int == '<' as c_int {
        while unsafe { **p } as c_int != '>' as c_int {
            let c = unsafe { **p };
            *p = unsafe { p.add(1) };
            if c == NUL as c_char {
                return None;
            }
        }
        let key = unsafe { get_special_key_code(arg.add(1)) };
        if key == 0 {
            unsafe { (*expand).xp_context = ExpandContext::Nothing };
            return None;
        }
        *p = unsafe { p.add(1) };
        let nextchar = unsafe { **p };
        // The two termcap bytes the key code packs.
        let lo = (-key & 0xff) as c_char;
        let hi = ((-key) as c_uint >> 8 & 0xff) as c_char;
        NAME.set([b't' as c_char, b'_' as c_char, lo, hi, 0]);
        return Some((nextchar, kOptAleph, 0, true));
    }

    // A bare `t_xx` spelling.
    if unsafe { **p } as c_int == 't' as c_int && unsafe { *p.add(1) } as c_int == '_' as c_int {
        *p = unsafe { p.add(2) };
        if unsafe { **p } != NUL as c_char {
            *p = unsafe { p.add(1) };
        }
        if unsafe { **p } == NUL as c_char {
            return None;
        }
        *p = unsafe { p.add(1) };
        let nextchar = unsafe { **p };
        NAME.set([
            b't' as c_char,
            b'_' as c_char,
            unsafe { *p.sub(2) },
            unsafe { *p.sub(1) },
            0,
        ]);
        return Some((nextchar, kOptAleph, 0, true));
    }

    // An ordinary name. `*` is allowed as a wildcard for the name
    // completion that follows.
    while (unsafe { **p } as u8).is_ascii_alphanumeric()
        || unsafe { **p } as c_int == '_' as c_int
        || unsafe { **p } as c_int == '*' as c_int
    {
        *p = unsafe { p.add(1) };
    }
    if unsafe { **p } == NUL as c_char {
        return None;
    }
    let nextchar = unsafe { **p };
    let len = unsafe { p.offset_from(arg) } as usize;
    let opt_idx = find_option_len(unsafe { slice::from_raw_parts(arg.cast::<u8>(), len) });
    if opt_idx == kOptInvalid || is_option_hidden(opt_idx) {
        unsafe { (*expand).xp_context = ExpandContext::Nothing };
        return None;
    }
    // A boolean takes no value, so there is nothing after the name.
    if option_has_type(opt_idx, kOptValTypeBoolean) {
        unsafe { (*expand).xp_context = ExpandContext::Nothing };
        return None;
    }
    Some((nextchar, opt_idx, get_option(opt_idx).flags, false))
}

/// A `kOptFlagExpand` option's value is a file or directory name; say which,
/// and how many backslashes escape a space in it.
///
/// # Safety
///
/// `expand` must be the expansion state.
unsafe fn set_file_context(expand: *mut Expand, opt_idx: OptIndex, flags: uint32_t) {
    // SAFETY: the caller's expansion state, and the option table.
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
    let context = if directories {
        ExpandContext::Directories
    } else {
        ExpandContext::Files
    };
    let backslash = if three {
        BackslashEscape::THREE
    } else {
        BackslashEscape::ONE
    };
    unsafe { (*expand).xp_context = context };
    unsafe { (*expand).xp_backslash = backslash };
    if flags & kOptFlagComma as uint32_t != 0 {
        unsafe { (*expand).xp_backslash |= BackslashEscape::COMMA };
    }
}

/// Move `expand.xp_pattern` forward to the start of the item the cursor is in,
/// for a value that is a list.
///
/// # Safety
///
/// `expand` must be the expansion state and `argend` the end of its argument.
unsafe fn seek_item_start(expand: *mut Expand, argend: *mut c_char, flags: uint32_t) {
    let comma_list = flags & kOptFlagComma as uint32_t != 0;
    let colon_list = flags & kOptFlagColon as uint32_t != 0;

    // SAFETY: the caller's expansion state and argument.
    let mut p = unsafe { argend.sub(1) };
    while p > unsafe { (*expand).xp_pattern } {
        let c = unsafe { *p } as c_int;
        let separator = c == ' ' as c_int || c == ',' as c_int || (c == ':' as c_int && colon_list);
        if separator {
            let bs = unsafe { backslashes_before((*expand).xp_pattern, p) };
            // A space only separates a triple-escaped value, a comma
            // needs fewer than two backslashes, and a colon in a
            // colon-list is never escaped.
            let splits = (c == ' ' as c_int
                && unsafe { (*expand).xp_backslash }.has(BackslashEscape::THREE)
                && bs < 3)
                || (c == ',' as c_int && comma_list && bs < 2)
                || (c == ':' as c_int && colon_list);
            if splits {
                unsafe { (*expand).xp_pattern = p.add(1) };
                break;
            }
        }
        p = unsafe { p.sub(1) };
    }
}

/// Where one completion pass puts what it matches, and how it matches.
///
/// `match_str` used to take these five alongside the candidate and the
/// index, which is what its `too_many_arguments` allow was for; a pass sees
/// one set of them throughout, so it builds the value once.
#[derive(Clone, Copy)]
struct Matcher {
    regmatch: *mut RegMatch,
    /// The plain array of names.
    matches: *mut *mut c_char,
    /// The scored array, used instead when `fuzzy`.
    fuzmatch: *mut FuzMatchStr,
    /// What a fuzzy pass matches against.
    fuzzystr: *const c_char,
    fuzzy: bool,
}

/// Whether `str` matches, recording it as match `idx` unless only the count
/// is wanted. The fuzzy form records a score instead.
///
/// # Safety
///
/// `matches`/`fuzmatch` must have room for `idx`, and the strings must be
/// NUL-terminated.
unsafe fn match_str(str: *mut c_char, idx: c_int, test_only: bool, m: Matcher) -> bool {
    // SAFETY: the caller's strings and output arrays.
    if !m.fuzzy {
        if !unsafe { vim_regexec(m.regmatch, str, 0 as ColNr) } {
            return false;
        }
        if !test_only {
            unsafe { *m.matches.offset(idx as isize) = xstrdup(str) };
        }
        return true;
    }
    let score = unsafe { fuzzy_match_str(cstr::at(str), cstr::at(m.fuzzystr)) };
    if score == FUZZY_SCORE_NONE {
        return false;
    }
    if !test_only {
        // SAFETY: the caller promised room for `idx`. The handle borrows
        // the slot for the one field write that asked and no longer -- a
        // `&mut *p` here would write into a discarded copy of the slot.
        let mut slot = unsafe { Live::new(m.fuzmatch.offset(idx as isize)) };
        slot.idx = idx;
        slot.str = unsafe { xstrdup(str) };
        slot.score = score;
    }
    true
}

/// Complete an option *name*.
///
/// Two passes: the first counts the matches so the array can be sized, the
/// second fills it.
///
/// # Safety
///
/// The out-parameters must be writable, and `regmatch`/`fuzzystr` valid.
pub(crate) unsafe fn expand_settings(
    expand: *mut Expand,
    regmatch: *mut RegMatch,
    fuzzystr: *mut c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
    can_fuzzy: bool,
) -> Result<(), Failed> {
    let mut num_normal = 0;
    let mut count = 0;
    let mut fuzmatch: *mut FuzMatchStr = ptr::null_mut();

    // SAFETY: the caller's expansion state and out-parameters, and the
    // option table.
    let ic = unsafe { (*regmatch).rm_ic };
    let fuzzy = can_fuzzy && unsafe { cmdline_fuzzy_complete(fuzzystr) };
    let booleans_only = unsafe { (*expand).xp_context } == ExpandContext::BoolSettings;

    for pass in 0..2 {
        let counting = pass == 0;
        unsafe { (*regmatch).rm_ic = ic };
        // Both output arrays are allocated at the end of the counting
        // pass, so one matcher stands for the whole of this one.
        let m = Matcher {
            regmatch,
            // SAFETY: the caller's out-parameter.
            matches: unsafe { *matches },
            fuzmatch,
            fuzzystr,
            fuzzy,
        };

        // "all" is a `:set` keyword rather than an option, so it is
        // only offered where a non-boolean name would be.
        let all = c"all".as_ptr() as *mut c_char;
        if !booleans_only && unsafe { match_str(all, count, counting, m) } {
            if counting {
                num_normal += 1;
            } else {
                count += 1;
            }
        }

        for opt_idx in kOptAleph..kOptCount as OptIndex {
            let opt = get_option(opt_idx);
            if is_option_hidden(opt_idx)
                || (booleans_only && !option_has_type(opt_idx, kOptValTypeBoolean))
            {
                continue;
            }
            if unsafe { match_str(opt.fullname, count, counting, m) } {
                if counting {
                    num_normal += 1;
                } else {
                    count += 1;
                }
            } else if !fuzzy
                && !opt.shortname.is_null()
                && unsafe { vim_regexec(regmatch, opt.shortname, 0 as ColNr) }
            {
                // A short name matches, but what is offered is the
                // full one.
                if counting {
                    num_normal += 1;
                } else {
                    unsafe { *(*matches).offset(count as isize) = xstrdup(opt.fullname) };
                    count += 1;
                }
            }
        }

        if counting {
            if num_normal == 0 {
                return Ok(());
            }
            unsafe { *num_matches = num_normal };
            if fuzzy {
                let room = (num_normal as size_t).wrapping_mul(size_of::<FuzMatchStr>());
                fuzmatch = unsafe { xmalloc(room) }.cast::<FuzMatchStr>();
            } else {
                let room = (num_normal as size_t).wrapping_mul(size_of::<*mut c_char>());
                let array = unsafe { xmalloc(room) }.cast::<*mut c_char>();
                unsafe { *matches = array };
            }
        }
    }

    if fuzzy {
        unsafe { fuzzymatches_to_strmatches(fuzmatch, matches, count, false) };
    }
    Ok(())
}

/// A value escaped the way the command line needs it back.
///
/// # Safety
///
/// `var` must be NUL-terminated. The result is owned by the caller.
pub(crate) unsafe fn escape_option_str_cmdline(var: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's string.
    unsafe { vim_strsave_escaped(var, escape_chars.get()) }
}

/// Offer the option's current value as the one completion.
///
/// # Safety
///
/// The out-parameters must be writable.
pub(crate) unsafe fn expand_old_setting(
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's out-parameters, and the option table.
    unsafe { *num_matches = 0 };
    unsafe { *matches = xmalloc(size_of::<*mut c_char>()).cast::<*mut c_char>() };

    // A terminal option has no table row, so it is looked up by the
    // name `set_context_in_set_cmd` spelled out.
    if IDX.get() == kOptInvalid {
        IDX.set(NAME.with(|name| find_option(cstr::in_chars(name))));
    }
    let mut rendered = [0 as c_char; MAXPATHL as usize];
    let var = if IDX.get() == kOptInvalid {
        c"".as_ptr() as *mut c_char
    } else {
        option_value2string(IDX.get(), FLAGS.get(), &mut rendered);
        rendered.as_mut_ptr()
    };
    unsafe { *(*matches) = escape_option_str_cmdline(var) };
    unsafe { *num_matches = 1 };
    Ok(())
}

/// Complete a value through the option's own `opt_expand_cb`.
///
/// # Safety
///
/// The out-parameters must be writable and `expand`/`regmatch` valid.
pub(crate) unsafe fn expand_string_setting(
    expand: *mut Expand,
    regmatch: *mut RegMatch,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's expansion state and out-parameters, and the
    // option table.
    let opt_idx = IDX.get();
    if opt_idx == kOptInvalid {
        return Err(Failed);
    }
    let Some(expand_cb) = get_option(opt_idx).opt_expand_cb else {
        return Err(Failed);
    };

    let mut rendered = [0 as c_char; MAXPATHL as usize];
    option_value2string(opt_idx, FLAGS.get(), &mut rendered);
    let escaped = unsafe { escape_option_str_cmdline(rendered.as_mut_ptr()) };

    let set_arg = unsafe { (*expand).xp_line.offset(START_COL.get() as isize) };
    let mut args = OptExpand {
        oe_idx: opt_idx,
        oe_opt_value: escaped,
        oe_append: APPEND.get(),
        // The current value is only worth offering back when nothing
        // has been typed yet and it is not being appended to.
        oe_include_orig_val: !APPEND.get() && unsafe { *set_arg } == NUL as c_char,
        oe_regmatch: regmatch,
        oe_xp: expand,
        oe_set_arg: set_arg,
    };
    let num_ret = unsafe { expand_cb(&raw mut args, num_matches, matches) };
    unsafe { xfree(escaped.cast::<c_void>()) };
    num_ret
}

/// Complete a `-=` value: only what the option already holds can be
/// removed, so the candidates are its own items.
///
/// # Safety
///
/// The out-parameters must be writable and `expand`/`regmatch` valid.
pub(crate) unsafe fn expand_setting_subtract(
    expand: *mut Expand,
    regmatch: *mut RegMatch,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's expansion state and out-parameters, and the
    // option table.
    let opt_idx = IDX.get();
    if opt_idx == kOptInvalid || option_has_type(opt_idx, kOptValTypeNumber) {
        return unsafe { expand_old_setting(num_matches, matches) };
    }
    let (buf, win) = (Buf::current(), Win::current());
    let varp = get_varp_scope_from(opt_idx, FLAGS.get(), buf, win);
    let value = unsafe { varp.string_var().get() };
    let flags = get_option(opt_idx).flags;

    if flags & kOptFlagComma as uint32_t != 0 {
        if unsafe { *value } == NUL as c_char {
            return Err(Failed);
        }
        // The split is destructive, so it runs on a copy.
        let copy = unsafe { xstrdup(value) };
        let mut ga = GArray {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        unsafe { ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 10) };
        let mut next = copy;
        loop {
            let item = next;
            let mut comma = unsafe { vim_strchr(next, ',' as c_int) };
            // An escaped comma is part of the item.
            while !comma.is_null()
                && comma != next
                && unsafe { *comma.sub(1) } as c_int == '\\' as c_int
            {
                comma = unsafe { vim_strchr(comma.add(1), ',' as c_int) };
            }
            if comma.is_null() {
                next = ptr::null_mut();
            } else {
                unsafe { *comma = NUL as c_char };
                next = unsafe { comma.add(1) };
            }
            if unsafe { *item } != NUL as c_char
                && unsafe { vim_regexec(regmatch, item, 0 as ColNr) }
            {
                unsafe { ga_grow(&raw mut ga, 1) };
                let slot = ga.ga_data.cast::<*mut c_char>();
                let escaped = unsafe { escape_option_str_cmdline(item) };
                unsafe { *slot.offset(ga.ga_len as isize) = escaped };
                ga.ga_len += 1;
            }
            if next.is_null() {
                break;
            }
        }
        unsafe { xfree(copy.cast::<c_void>()) };
        unsafe { *matches = ga.ga_data.cast::<*mut c_char>() };
        unsafe { *num_matches = ga.ga_len };
        return Ok(());
    }

    if flags & kOptFlagFlagList as uint32_t != 0 {
        // A set of one-letter flags: offer the whole set first, then
        // each letter. Nothing may have been typed, since a flag set
        // has no word boundary to complete from.
        if unsafe { *(*expand).xp_pattern } != NUL as c_char {
            return Err(Failed);
        }
        let num_flags = unsafe { cstr::bytes_at(value) }.len();
        if num_flags == 0 {
            return Err(Failed);
        }
        let room = size_of::<*mut c_char>().wrapping_mul(num_flags.wrapping_add(1));
        let array = unsafe { xmalloc(room) }.cast::<*mut c_char>();
        unsafe { *matches = array };
        let mut count = 0;
        unsafe { *(*matches) = xmemdupz(value.cast::<c_void>(), num_flags).cast::<c_char>() };
        count += 1;
        if num_flags > 1 {
            let mut flag = value;
            while unsafe { *flag } != NUL as c_char {
                let copy = unsafe { xmemdupz(flag.cast::<c_void>(), 1) }.cast::<c_char>();
                unsafe { *(*matches).offset(count as isize) = copy };
                count += 1;
                flag = unsafe { flag.add(1) };
            }
        }
        unsafe { *num_matches = count };
        return Ok(());
    }

    unsafe { expand_old_setting(num_matches, matches) }
}
