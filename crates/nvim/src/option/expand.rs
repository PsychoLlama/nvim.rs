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

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::cmdexpand::cmdline_fuzzy_complete;
use crate::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::garray::{ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::keycodes::get_special_key_code;
use crate::main::{
    NameBuff, curbuf, curwin, escape_chars, p_bdir, p_cdpath, p_dir, p_ft, p_keymap, p_path, p_pp,
    p_rtp, p_sps, p_syn, p_tags, p_vdir,
};
use crate::memory::{xfree, xmalloc, xmemdupz, xstrdup};
use crate::options::{kOptAleph, kOptCount, kOptInvalid, options};
use crate::os::cshim::strncmp;
use crate::os::env::expand_env_esc;
use crate::regexp::vim_regexec;
use crate::strings::{vim_strchr, vim_strsave_escaped};
use crate::types::{
    FAIL, OK, OptIndex, colnr_T, expand_T, fuzmatch_str_T, garray_T, optexpand_T, regmatch_T,
    size_t, uint8_t, uint32_t, vimoption_T, xp_prefix_T,
};
use ::libc::{strcmp, strlen};

use super::{
    EXPAND_BOOL_SETTINGS, EXPAND_DIRECTORIES, EXPAND_FILES, EXPAND_FILETYPE, EXPAND_KEYMAP,
    EXPAND_NOTHING, EXPAND_OLD_SETTING, EXPAND_OWNSYNTAX, EXPAND_SETTING_SUBTRACT, EXPAND_SETTINGS,
    EXPAND_STRING_SETTING, EXPAND_UNSUCCESSFUL, FUZZY_SCORE_NONE, MAXPATHL, NUL, XP_BS_COMMA,
    XP_BS_ONE, XP_BS_THREE, XP_PREFIX_INV, XP_PREFIX_NO, find_option, find_option_len, get_option,
    get_option_varp_scope_from, get_varp_scope, is_option_hidden, kOptFlagColon, kOptFlagComma,
    kOptFlagExpand, kOptFlagFlagList, kOptValTypeBoolean, kOptValTypeNumber, option_has_type,
    option_value2string, option_var,
};

/// What [`set_context_in_set_cmd`] worked out, for the `Expand*` half.
///
/// `IDX` is `kOptInvalid` for a terminal option, whose two-letter name is
/// spelled into `NAME` instead.
static IDX: GlobalCell<OptIndex> = GlobalCell::new(kOptInvalid);
static NAME: GlobalCell<[c_char; 5]> = GlobalCell::new([b't' as c_char, b'_' as c_char, 0, 0, 0]);
static START_COL: GlobalCell<c_int> = GlobalCell::new(0);
static FLAGS: GlobalCell<c_int> = GlobalCell::new(0);
/// Whether the operator was `+=` or `^=`, which means the current value is
/// not a candidate to offer back.
static APPEND: GlobalCell<bool> = GlobalCell::new(false);

/// The option's value with `$VAR` and `~` expanded, in `NameBuff`, or null
/// when there was nothing to expand.
///
/// # Safety
///
/// `val`, if not null, must be NUL-terminated.
pub(crate) unsafe fn option_expand(opt_idx: OptIndex, val: *const c_char) -> *mut c_char {
    // SAFETY: the option table is a plain array, `var` is the option's own
    // variable, and the caller's `val` is NUL-terminated.
    unsafe {
        let opt = get_option(opt_idx);
        if (*opt).flags & kOptFlagExpand as uint32_t == 0 || is_option_hidden(opt_idx) {
            return ptr::null_mut();
        }
        let val = if val.is_null() {
            *option_var(opt).cast::<*mut c_char>()
        } else {
            val
        };
        // The buffer the expansion lands in is `MAXPATHL` bytes.
        if val.is_null() || strlen(val) > MAXPATHL as size_t {
            return ptr::null_mut();
        }
        let var = option_var(opt).cast::<*mut c_char>();
        // 'path' and 'tags' hold escaped file names, so their separators
        // must survive the expansion.
        let esc = var == p_tags.ptr() || var == p_path.ptr();
        // 'spellsuggest' items are `file:<name>`; only that prefix's tail
        // is a path.
        let one_prefix = if var == p_sps.ptr() {
            c"file:".as_ptr() as *mut c_char
        } else {
            ptr::null_mut()
        };
        expand_env_esc(
            val,
            NameBuff.ptr().cast::<c_char>(),
            MAXPATHL,
            esc,
            false,
            one_prefix,
        );
        if strcmp(NameBuff.ptr().cast::<c_char>(), val) == 0 {
            return ptr::null_mut();
        }
        NameBuff.ptr().cast::<c_char>()
    }
}

/// Work out what the cursor is sitting on in a `:set` command line, and
/// leave `xp` describing what to complete.
///
/// # Safety
///
/// `xp` must be the command line's expansion state and `arg` a
/// NUL-terminated cursor into `xp->xp_line`.
pub unsafe fn set_context_in_set_cmd(xp: *mut expand_T, arg: *mut c_char, opt_flags: c_int) {
    FLAGS.set(opt_flags);

    // SAFETY: the caller's expansion state and command line.
    unsafe {
        (*xp).xp_context = EXPAND_SETTINGS;
        if *arg == NUL as c_char {
            (*xp).xp_pattern = arg;
            return;
        }

        let argend = arg.add(strlen(arg));
        // A trailing unescaped space starts a fresh argument.
        let last = argend.sub(1);
        if *last as c_int == ' ' as c_int && *last.sub(1) as c_int != '\\' as c_int {
            (*xp).xp_pattern = last.add(1);
            return;
        }

        // Walk back to the start of the argument the cursor is in: the
        // first space with an even number of backslashes before it.
        let mut p = last;
        while p > arg {
            let unescaped = if *p as c_int == ' ' as c_int || *p as c_int == ',' as c_int {
                backslashes_before(arg, p) & 1 == 0
            } else {
                false
            };
            if *p as c_int == ' ' as c_int && unescaped {
                p = p.add(1);
                break;
            }
            p = p.sub(1);
        }

        for (spelling, prefix) in [
            (c"no", XP_PREFIX_NO as xp_prefix_T),
            (c"inv", XP_PREFIX_INV as xp_prefix_T),
        ] {
            let len = spelling.count_bytes();
            if strncmp(p, spelling.as_ptr(), len) == 0 {
                (*xp).xp_context = EXPAND_BOOL_SETTINGS;
                (*xp).xp_prefix = prefix;
                p = p.add(len);
                break;
            }
        }
        (*xp).xp_pattern = p;
        let arg = p;

        let Some((nextchar, opt_idx, flags, is_term_option)) = take_option_name(xp, arg, &mut p)
        else {
            return;
        };

        // `-=`, `+=` and `^=` complete like `=`, but the current value is
        // only worth offering back for `-=`.
        let mut nextchar = nextchar;
        APPEND.set(false);
        let mut subtract = false;
        if matches!(nextchar as u8, b'-' | b'+' | b'^') && *p.add(1) as c_int == '=' as c_int {
            subtract = nextchar as u8 == b'-';
            APPEND.set(matches!(nextchar as u8, b'+' | b'^'));
            p = p.add(1);
            nextchar = '=' as c_char;
        }
        if (nextchar as c_int != '=' as c_int && nextchar as c_int != ':' as c_int)
            || (*xp).xp_context == EXPAND_BOOL_SETTINGS
        {
            (*xp).xp_context = EXPAND_UNSUCCESSFUL;
            return;
        }

        // Everything below completes the *value*, after the `=` or `:`.
        IDX.set(if is_term_option { kOptInvalid } else { opt_idx });
        (*xp).xp_pattern = p.add(1);
        START_COL.set(p.add(1).offset_from((*xp).xp_line) as c_int);

        // Three options reuse another command's completion wholesale.
        let var = option_var(get_option(opt_idx));
        for (cell, context) in [
            (p_syn.ptr().cast::<c_void>(), EXPAND_OWNSYNTAX),
            (p_ft.ptr().cast::<c_void>(), EXPAND_FILETYPE),
            (p_keymap.ptr().cast::<c_void>(), EXPAND_KEYMAP),
        ] {
            if var == cell {
                (*xp).xp_context = context;
                return;
            }
        }

        if subtract {
            (*xp).xp_context = EXPAND_SETTING_SUBTRACT;
            return;
        } else if IDX.get() != kOptInvalid
            && (*options.ptr())[IDX.get() as usize].opt_expand_cb.is_some()
        {
            (*xp).xp_context = EXPAND_STRING_SETTING;
        } else if *(*xp).xp_pattern == NUL as c_char {
            (*xp).xp_context = EXPAND_OLD_SETTING;
            return;
        } else {
            (*xp).xp_context = EXPAND_NOTHING;
        }

        if is_term_option || option_has_type(opt_idx, kOptValTypeNumber) {
            return;
        }

        // Only string options from here.
        if flags & kOptFlagExpand as uint32_t != 0 {
            set_file_context(xp, opt_idx, flags);
        }
        if flags & (kOptFlagExpand | kOptFlagComma | kOptFlagColon) as uint32_t != 0 {
            seek_item_start(xp, argend, flags);
        }
        // A set of one-letter flags has no words to complete, so the
        // pattern is always empty and the whole set is offered.
        if flags & kOptFlagFlagList as uint32_t != 0 {
            (*xp).xp_pattern = argend;
        }
        // 'spellsuggest' takes `file:<name>`, whose tail is a file name.
        if var == p_sps.ptr().cast::<c_void>() {
            if strncmp((*xp).xp_pattern, c"file:".as_ptr(), 5) == 0 {
                (*xp).xp_pattern = (*xp).xp_pattern.add(5);
            } else if (*options.ptr())[IDX.get() as usize].opt_expand_cb.is_some() {
                (*xp).xp_context = EXPAND_STRING_SETTING;
            }
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
    unsafe {
        while s > start && *s.sub(1) as c_int == '\\' as c_int {
            s = s.sub(1);
        }
        at.offset_from(s)
    }
}

/// Consume the option name at `arg`, leaving `*p` on the character after
/// it. `None` means the cursor is still inside the name, so the name itself
/// is what to complete and `xp` has been left saying so.
///
/// Returns the character after the name, the option, its flags, and whether
/// it was one of the `t_xx` terminal names — which have no table row.
///
/// # Safety
///
/// `xp` must be the expansion state and `arg` a NUL-terminated cursor.
unsafe fn take_option_name(
    xp: *mut expand_T,
    arg: *mut c_char,
    p: &mut *mut c_char,
) -> Option<(c_char, OptIndex, uint32_t, bool)> {
    // SAFETY: the caller's command line and expansion state.
    unsafe {
        // `<t_xx>` and `<Key>` spellings.
        if *arg as c_int == '<' as c_int {
            while **p as c_int != '>' as c_int {
                let c = **p;
                *p = p.add(1);
                if c == NUL as c_char {
                    return None;
                }
            }
            let key = get_special_key_code(arg.add(1));
            if key == 0 {
                (*xp).xp_context = EXPAND_NOTHING;
                return None;
            }
            *p = p.add(1);
            let nextchar = **p;
            // The two termcap bytes the key code packs.
            (*NAME.ptr())[2] = (-key & 0xff) as uint8_t as c_char;
            (*NAME.ptr())[3] = ((-key) as c_uint >> 8 & 0xff) as uint8_t as c_char;
            return Some((nextchar, kOptAleph, 0, true));
        }

        // A bare `t_xx` spelling.
        if **p as c_int == 't' as c_int && *p.add(1) as c_int == '_' as c_int {
            *p = p.add(2);
            if **p != NUL as c_char {
                *p = p.add(1);
            }
            if **p == NUL as c_char {
                return None;
            }
            *p = p.add(1);
            let nextchar = **p;
            (*NAME.ptr())[2] = *p.sub(2);
            (*NAME.ptr())[3] = *p.sub(1);
            return Some((nextchar, kOptAleph, 0, true));
        }

        // An ordinary name. `*` is allowed as a wildcard for the name
        // completion that follows.
        while (**p as u8).is_ascii_alphanumeric()
            || **p as c_int == '_' as c_int
            || **p as c_int == '*' as c_int
        {
            *p = p.add(1);
        }
        if **p == NUL as c_char {
            return None;
        }
        let nextchar = **p;
        let opt_idx = find_option_len(arg, p.offset_from(arg) as size_t);
        if opt_idx == kOptInvalid || is_option_hidden(opt_idx) {
            (*xp).xp_context = EXPAND_NOTHING;
            return None;
        }
        // A boolean takes no value, so there is nothing after the name.
        if option_has_type(opt_idx, kOptValTypeBoolean) {
            (*xp).xp_context = EXPAND_NOTHING;
            return None;
        }
        Some((
            nextchar,
            opt_idx,
            (*options.ptr())[opt_idx as usize].flags,
            false,
        ))
    }
}

/// A `kOptFlagExpand` option's value is a file or directory name; say which,
/// and how many backslashes escape a space in it.
///
/// # Safety
///
/// `xp` must be the expansion state.
unsafe fn set_file_context(xp: *mut expand_T, opt_idx: OptIndex, flags: uint32_t) {
    // SAFETY: the caller's expansion state, and the option table.
    unsafe {
        let var = option_var(get_option(opt_idx)).cast::<c_char>();
        let directories = [
            p_bdir.ptr().cast::<c_char>(),
            p_dir.ptr().cast::<c_char>(),
            p_path.ptr().cast::<c_char>(),
            p_pp.ptr().cast::<c_char>(),
            p_rtp.ptr().cast::<c_char>(),
            p_cdpath.ptr().cast::<c_char>(),
            p_vdir.ptr().cast::<c_char>(),
        ];
        // 'path', 'cdpath' and 'tags' need three backslashes for a space,
        // because their own parsers unescape one layer first.
        let three = var == p_path.ptr().cast::<c_char>()
            || var == p_cdpath.ptr().cast::<c_char>()
            || var == p_tags.ptr().cast::<c_char>();
        (*xp).xp_context = if directories.contains(&var) {
            EXPAND_DIRECTORIES
        } else {
            EXPAND_FILES
        };
        (*xp).xp_backslash = if three {
            XP_BS_THREE as c_int
        } else {
            XP_BS_ONE as c_int
        };
        if flags & kOptFlagComma as uint32_t != 0 {
            (*xp).xp_backslash |= XP_BS_COMMA as c_int;
        }
    }
}

/// Move `xp->xp_pattern` forward to the start of the item the cursor is in,
/// for a value that is a list.
///
/// # Safety
///
/// `xp` must be the expansion state and `argend` the end of its argument.
unsafe fn seek_item_start(xp: *mut expand_T, argend: *mut c_char, flags: uint32_t) {
    let comma_list = flags & kOptFlagComma as uint32_t != 0;
    let colon_list = flags & kOptFlagColon as uint32_t != 0;

    // SAFETY: the caller's expansion state and argument.
    unsafe {
        let mut p = argend.sub(1);
        while p > (*xp).xp_pattern {
            let c = *p as c_int;
            let separator =
                c == ' ' as c_int || c == ',' as c_int || (c == ':' as c_int && colon_list);
            if separator {
                let bs = backslashes_before((*xp).xp_pattern, p);
                // A space only separates a triple-escaped value, a comma
                // needs fewer than two backslashes, and a colon in a
                // colon-list is never escaped.
                let splits =
                    (c == ' ' as c_int && (*xp).xp_backslash & XP_BS_THREE as c_int != 0 && bs < 3)
                        || (c == ',' as c_int && comma_list && bs < 2)
                        || (c == ':' as c_int && colon_list);
                if splits {
                    (*xp).xp_pattern = p.add(1);
                    break;
                }
            }
            p = p.sub(1);
        }
    }
}

/// Whether `str` matches, recording it as match `idx` unless only the count
/// is wanted. The fuzzy form records a score instead.
///
/// # Safety
///
/// `matches`/`fuzmatch` must have room for `idx`, and the strings must be
/// NUL-terminated.
#[allow(clippy::too_many_arguments)]
unsafe fn match_str(
    str: *mut c_char,
    regmatch: *mut regmatch_T,
    matches: *mut *mut c_char,
    idx: c_int,
    test_only: bool,
    fuzzy: bool,
    fuzzystr: *const c_char,
    fuzmatch: *mut fuzmatch_str_T,
) -> bool {
    // SAFETY: the caller's strings and output arrays.
    unsafe {
        if !fuzzy {
            if !vim_regexec(regmatch, str, 0 as colnr_T) {
                return false;
            }
            if !test_only {
                *matches.offset(idx as isize) = xstrdup(str);
            }
            return true;
        }
        let score = fuzzy_match_str(str, fuzzystr);
        if score == FUZZY_SCORE_NONE {
            return false;
        }
        if !test_only {
            let slot = &mut *fuzmatch.offset(idx as isize);
            slot.idx = idx;
            slot.str = xstrdup(str);
            slot.score = score;
        }
        true
    }
}

/// Complete an option *name*.
///
/// Two passes: the first counts the matches so the array can be sized, the
/// second fills it.
///
/// # Safety
///
/// The out-parameters must be writable, and `regmatch`/`fuzzystr` valid.
pub unsafe fn ExpandSettings(
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    fuzzystr: *mut c_char,
    numMatches: *mut c_int,
    matches: *mut *mut *mut c_char,
    can_fuzzy: bool,
) -> c_int {
    let mut num_normal = 0;
    let mut count = 0;
    let mut fuzmatch: *mut fuzmatch_str_T = ptr::null_mut();

    // SAFETY: the caller's expansion state and out-parameters, and the
    // option table.
    unsafe {
        let ic = (*regmatch).rm_ic;
        let fuzzy = can_fuzzy && cmdline_fuzzy_complete(fuzzystr);
        let booleans_only = (*xp).xp_context == EXPAND_BOOL_SETTINGS;

        for pass in 0..2 {
            let counting = pass == 0;
            (*regmatch).rm_ic = ic;

            // "all" is a `:set` keyword rather than an option, so it is
            // only offered where a non-boolean name would be.
            if !booleans_only
                && match_str(
                    c"all".as_ptr() as *mut c_char,
                    regmatch,
                    *matches,
                    count,
                    counting,
                    fuzzy,
                    fuzzystr,
                    fuzmatch,
                )
            {
                if counting {
                    num_normal += 1;
                } else {
                    count += 1;
                }
            }

            for opt_idx in kOptAleph..kOptCount as OptIndex {
                let opt = &(*options.ptr())[opt_idx as usize];
                if is_option_hidden(opt_idx)
                    || (booleans_only && !option_has_type(opt_idx, kOptValTypeBoolean))
                {
                    continue;
                }
                if match_str(
                    opt.fullname,
                    regmatch,
                    *matches,
                    count,
                    counting,
                    fuzzy,
                    fuzzystr,
                    fuzmatch,
                ) {
                    if counting {
                        num_normal += 1;
                    } else {
                        count += 1;
                    }
                } else if !fuzzy
                    && !opt.shortname.is_null()
                    && vim_regexec(regmatch, opt.shortname, 0 as colnr_T)
                {
                    // A short name matches, but what is offered is the
                    // full one.
                    if counting {
                        num_normal += 1;
                    } else {
                        *(*matches).offset(count as isize) = xstrdup(opt.fullname);
                        count += 1;
                    }
                }
            }

            if counting {
                if num_normal == 0 {
                    return OK;
                }
                *numMatches = num_normal;
                if fuzzy {
                    fuzmatch = xmalloc(
                        (num_normal as size_t).wrapping_mul(core::mem::size_of::<fuzmatch_str_T>()),
                    )
                    .cast::<fuzmatch_str_T>();
                } else {
                    *matches = xmalloc(
                        (num_normal as size_t).wrapping_mul(core::mem::size_of::<*mut c_char>()),
                    )
                    .cast::<*mut c_char>();
                }
            }
        }

        if fuzzy {
            fuzzymatches_to_strmatches(fuzmatch, matches, count, false);
        }
    }
    OK
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
pub unsafe fn ExpandOldSetting(numMatches: *mut c_int, matches: *mut *mut *mut c_char) -> c_int {
    // SAFETY: the caller's out-parameters, and the option table.
    unsafe {
        *numMatches = 0;
        *matches = xmalloc(core::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();

        // A terminal option has no table row, so it is looked up by the
        // name `set_context_in_set_cmd` spelled out.
        if IDX.get() == kOptInvalid {
            IDX.set(find_option(NAME.ptr().cast::<c_char>()));
        }
        let var = if IDX.get() == kOptInvalid {
            c"".as_ptr() as *mut c_char
        } else {
            option_value2string(&raw mut (*options.ptr())[IDX.get() as usize], FLAGS.get());
            NameBuff.ptr().cast::<c_char>()
        };
        *(*matches) = escape_option_str_cmdline(var);
        *numMatches = 1;
    }
    OK
}

/// Complete a value through the option's own `opt_expand_cb`.
///
/// # Safety
///
/// The out-parameters must be writable and `xp`/`regmatch` valid.
pub unsafe fn ExpandStringSetting(
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    numMatches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's expansion state and out-parameters, and the
    // option table.
    unsafe {
        let opt_idx = IDX.get();
        if opt_idx == kOptInvalid {
            return FAIL;
        }
        let opt: *mut vimoption_T = &raw mut (*options.ptr())[opt_idx as usize];
        let Some(expand_cb) = (*opt).opt_expand_cb else {
            return FAIL;
        };

        option_value2string(opt, FLAGS.get());
        let escaped = escape_option_str_cmdline(NameBuff.ptr().cast::<c_char>());

        let set_arg = (*xp).xp_line.offset(START_COL.get() as isize);
        let mut args = optexpand_T {
            oe_varp: get_varp_scope(opt, FLAGS.get()).cast::<c_char>(),
            oe_idx: opt_idx,
            oe_opt_value: escaped,
            oe_append: APPEND.get(),
            // The current value is only worth offering back when nothing
            // has been typed yet and it is not being appended to.
            oe_include_orig_val: !APPEND.get() && *set_arg == NUL as c_char,
            oe_regmatch: regmatch,
            oe_xp: xp,
            oe_set_arg: set_arg,
        };
        let num_ret = expand_cb(&raw mut args, numMatches, matches);
        xfree(escaped.cast::<c_void>());
        num_ret
    }
}

/// Complete a `-=` value: only what the option already holds can be
/// removed, so the candidates are its own items.
///
/// # Safety
///
/// The out-parameters must be writable and `xp`/`regmatch` valid.
pub unsafe fn ExpandSettingSubtract(
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    numMatches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's expansion state and out-parameters, and the
    // option table.
    unsafe {
        let opt_idx = IDX.get();
        if opt_idx == kOptInvalid || option_has_type(opt_idx, kOptValTypeNumber) {
            return ExpandOldSetting(numMatches, matches);
        }
        let value = *get_option_varp_scope_from(opt_idx, FLAGS.get(), curbuf.get(), curwin.get())
            .cast::<*mut c_char>();
        let flags = (*options.ptr())[opt_idx as usize].flags;

        if flags & kOptFlagComma as uint32_t != 0 {
            if *value == NUL as c_char {
                return FAIL;
            }
            // The split is destructive, so it runs on a copy.
            let copy = xstrdup(value);
            let mut ga = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ptr::null_mut(),
            };
            ga_init(
                &raw mut ga,
                core::mem::size_of::<*mut c_char>() as c_int,
                10,
            );
            let mut next = copy;
            loop {
                let item = next;
                let mut comma = vim_strchr(next, ',' as c_int);
                // An escaped comma is part of the item.
                while !comma.is_null() && comma != next && *comma.sub(1) as c_int == '\\' as c_int {
                    comma = vim_strchr(comma.add(1), ',' as c_int);
                }
                if comma.is_null() {
                    next = ptr::null_mut();
                } else {
                    *comma = NUL as c_char;
                    next = comma.add(1);
                }
                if *item != NUL as c_char && vim_regexec(regmatch, item, 0 as colnr_T) {
                    ga_grow(&raw mut ga, 1);
                    *ga.ga_data.cast::<*mut c_char>().offset(ga.ga_len as isize) =
                        escape_option_str_cmdline(item);
                    ga.ga_len += 1;
                }
                if next.is_null() {
                    break;
                }
            }
            xfree(copy.cast::<c_void>());
            *matches = ga.ga_data.cast::<*mut c_char>();
            *numMatches = ga.ga_len;
            return OK;
        }

        if flags & kOptFlagFlagList as uint32_t != 0 {
            // A set of one-letter flags: offer the whole set first, then
            // each letter. Nothing may have been typed, since a flag set
            // has no word boundary to complete from.
            if *(*xp).xp_pattern != NUL as c_char {
                return FAIL;
            }
            let num_flags = strlen(value);
            if num_flags == 0 {
                return FAIL;
            }
            *matches = xmalloc(
                core::mem::size_of::<*mut c_char>().wrapping_mul(num_flags.wrapping_add(1)),
            )
            .cast::<*mut c_char>();
            let mut count = 0;
            *(*matches) = xmemdupz(value.cast::<c_void>(), num_flags).cast::<c_char>();
            count += 1;
            if num_flags > 1 {
                let mut flag = value;
                while *flag != NUL as c_char {
                    *(*matches).offset(count as isize) =
                        xmemdupz(flag.cast::<c_void>(), 1).cast::<c_char>();
                    count += 1;
                    flag = flag.add(1);
                }
            }
            *numMatches = count;
            return OK;
        }

        ExpandOldSetting(numMatches, matches)
    }
}
