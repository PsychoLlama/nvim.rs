//! Command-line completion of a string option's value.
//!
//! Every entry point here has the same shape: the option table hands over an
//! `optexpand_T` describing what the user has typed so far, and the
//! completer fills an `xmalloc`ed `char *` array plus its length. The three
//! ways to produce one:
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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::autocmd::get_event_name_no_group;
use crate::cmdexpand::expand_generic;
use crate::global_cell::GlobalCell;
use crate::highlight_group::get_highlight_name;
use crate::main::{IObuff, curwin, p_ei, p_lcs};
use crate::mbyte::get_encoding_name;
use crate::memory::{xfree, xmalloc, xmemdupz, xstrdup};
use crate::options::{opt_dip_algorithm_values, opt_dip_inline_values, opt_ff_values};
use crate::os::cshim::{snprintf, strncmp};
use crate::strings::vim_strchr;
use crate::types::{
    CompleteListItemGetter, FAIL, IOSIZE, NUL, OK, colnr_T, expand_T, optexpand_T, regmatch_T,
    size_t,
};
use ::libc::strcmp;

use super::{
    COCU_ALL, CPO_VI, FO_ALL, MOUSE_ALL, SHM_ALL, WW_ALL, get_fillchars_name, get_listchars_name,
    opt_values, vim_regexec,
};

/// The completion result under construction: an `xmalloc`ed `char *` array
/// and how much of it is used.
///
/// Upstream sizes the array for the largest possible answer up front rather
/// than measuring first, because every list here is a fixed enumeration of
/// at most a few dozen entries.
struct Matches {
    into: *mut *mut c_char,
    count: c_int,
}

impl Matches {
    /// # Safety
    /// At most `capacity` calls to [`Matches::push`] may follow.
    unsafe fn with_capacity(capacity: size_t) -> Self {
        let bytes = size_of::<*mut c_char>() * (capacity + 1);
        // SAFETY: `xmalloc` returns an allocation of that size or aborts.
        let into = unsafe { xmalloc(bytes) }.cast::<*mut c_char>();
        Matches { into, count: 0 }
    }

    /// Take ownership of one already-allocated completion.
    ///
    /// # Safety
    /// Fewer than `capacity` pushes have happened, and `owned` is an
    /// allocation whoever consumes the array will free.
    unsafe fn push(&mut self, owned: *mut c_char) {
        // SAFETY: the count is below the capacity, as documented above.
        unsafe { *self.into.offset(self.count as isize) = owned };
        self.count += 1;
    }

    /// Hand the array to the caller, or free it and report `FAIL` when
    /// nothing matched — in which case the out-parameter is left null.
    ///
    /// # Safety
    /// `matches` and `num` are the completer's out-parameters.
    unsafe fn finish(self, matches: *mut *mut *mut c_char, num: *mut c_int) -> c_int {
        if self.count == 0 {
            // SAFETY: the array this owns, and the caller's out-parameter.
            unsafe {
                xfree(self.into.cast::<c_void>());
                *matches = ptr::null_mut();
            }
            return FAIL;
        }
        // SAFETY: the caller's out-parameters.
        unsafe {
            *matches = self.into;
            *num = self.count;
        }
        OK
    }
}

/// Is the option's current value worth offering as the first completion?
/// Only when the caller asked for it and there is one.
///
/// # Safety
/// `args` points at the option table's completion frame.
unsafe fn original_value(args: *mut optexpand_T) -> Option<*mut c_char> {
    // SAFETY: the caller's frame; `oe_opt_value` is a C string.
    unsafe {
        let value = (*args).oe_opt_value;
        ((*args).oe_include_orig_val && c_int::from(*value) != NUL).then_some(value)
    }
}

/// Complete an option whose accepted words the generated table lists.
///
/// # Safety
/// `args` points at the completion frame; `values` is an array of at most
/// `num_values` C strings followed by a null pointer.
pub(crate) unsafe fn expand_set_opt_string(
    args: *mut optexpand_T,
    values: *const *const c_char,
    num_values: size_t,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's frame.
    let regmatch: *mut regmatch_T = unsafe { (*args).oe_regmatch };
    let original = unsafe { original_value(args) };

    // SAFETY: at most one push per word, plus the original value.
    let mut out = unsafe { Matches::with_capacity(num_values) };
    if let Some(value) = original {
        // SAFETY: `value` is a C string, and `xstrdup` hands over an
        // allocation the consumer frees.
        unsafe { out.push(xstrdup(value)) };
    }

    let mut at = values;
    loop {
        // SAFETY: the array ends in a null pointer, and the walk stops
        // there.
        let word = unsafe { *at };
        if word.is_null() {
            break;
        }
        at = unsafe { at.add(1) };
        // SAFETY: a non-null entry is a C string.
        if unsafe { c_int::from(*word) } == NUL {
            continue; // Ignore an empty accepted word.
        }
        // The current value is already the first completion; do not repeat
        // it.
        // SAFETY: both are C strings.
        if let Some(value) = original
            && unsafe { strcmp(word, value) } == 0
        {
            continue;
        }
        // SAFETY: `regmatch` is the command line's compiled pattern and
        // `word` a C string.
        if unsafe { vim_regexec(regmatch, word, 0 as colnr_T) } {
            unsafe { out.push(xstrdup(word)) };
        }
    }

    unsafe { out.finish(matches, num_matches) }
}

/// Complete an option whose accepted words the generated table lists, found
/// through the option's own index.
///
/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_str_generic(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's frame.
    let (values, values_len) = opt_values(unsafe { (*args).oe_idx });
    unsafe { expand_set_opt_string(args, values, values_len, num_matches, matches) }
}

/// The option's current value, offered as completion index 0 ahead of
/// whatever the real enumerator produces.
static ORIGINAL_VALUE: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// The real enumerator, for as long as `expand_generic` is running.
static ENUMERATOR: GlobalCell<CompleteListItemGetter> = GlobalCell::new(None);

/// The enumerator `expand_generic` sees: index 0 is the current value (or the
/// empty string, which `expand_generic` ignores), and everything above it is
/// the real enumerator shifted by one.
///
/// # Safety
/// Only reached from `expand_generic`, between the two assignments in
/// [`expand_set_opt_generic`].
unsafe fn expand_set_opt_callback(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    if idx == 0 {
        let original = ORIGINAL_VALUE.get();
        return if original.is_null() {
            c"".as_ptr().cast_mut()
        } else {
            original
        };
    }
    let next = ENUMERATOR.get().expect("enumerator set for the whole call");
    // SAFETY: the enumerator this call installed, with its own index.
    unsafe { next(xp, idx - 1) }
}

/// Complete an option from an editor-side enumerator rather than from a
/// fixed list.
///
/// # Safety
/// `args` points at the completion frame; `func` enumerates C strings.
pub(crate) unsafe fn expand_set_opt_generic(
    args: *mut optexpand_T,
    func: CompleteListItemGetter,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's frame.
    ORIGINAL_VALUE.set(unsafe {
        if (*args).oe_include_orig_val {
            (*args).oe_opt_value
        } else {
            ptr::null_mut()
        }
    });
    ENUMERATOR.set(func);

    // Not fuzzy: ExpandContext::StringSetting does not use fuzzy matching.
    // SAFETY: the caller's frame supplies the expansion context and the
    // command line's compiled pattern.
    unsafe {
        expand_generic(
            c"".as_ptr(),
            (*args).oe_xp,
            (*args).oe_regmatch,
            matches,
            num_matches,
            Some(expand_set_opt_callback),
            false,
        );
    }

    ORIGINAL_VALUE.set(ptr::null_mut());
    ENUMERATOR.set(None);
    OK
}

/// Complete an option that is a set of flag letters: one completion per
/// letter that is not already spoken for.
///
/// # Safety
/// `args` points at the completion frame; `flags` is a C string.
pub(crate) unsafe fn expand_set_opt_listflag(
    args: *mut optexpand_T,
    flags: *const c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's frame; `oe_opt_value` and `oe_set_arg` are C
    // strings.
    let (option_val, cmdline_val, append) =
        unsafe { ((*args).oe_opt_value, (*args).oe_set_arg, (*args).oe_append) };
    let original = unsafe { original_value(args) };
    // SAFETY: a C string.
    let flags = unsafe { CStr::from_ptr(flags) }.to_bytes();

    // SAFETY: at most one push per letter, plus the current value.
    let mut out = unsafe { Matches::with_capacity(flags.len()) };
    if let Some(value) = original {
        // SAFETY: a C string.
        unsafe { out.push(xstrdup(value)) };
    }

    for (at, &flag) in flags.iter().enumerate() {
        // With `+=`, a letter the value already carries cannot be added
        // again.
        // SAFETY: both are C strings; `vim_strchr` only reads them.
        if append && !unsafe { vim_strchr(option_val, c_int::from(flag)) }.is_null() {
            continue;
        }
        if !unsafe { vim_strchr(cmdline_val, c_int::from(flag)) }.is_null() {
            continue;
        }
        // A one-letter value is already the first completion; do not offer
        // the same letter twice.
        // SAFETY: `original` being set means `option_val` is non-empty, so
        // it has a second byte (its terminator at worst).
        if original.is_some()
            && unsafe { c_int::from(*option_val.add(1)) } == NUL
            && unsafe { *option_val } as u8 == flag
        {
            continue;
        }
        // SAFETY: one byte of `flags`, copied with a terminator.
        let one = unsafe { xmemdupz(flags.as_ptr().add(at).cast::<c_void>(), 1) };
        unsafe { out.push(one.cast::<c_char>()) };
    }

    unsafe { out.finish(matches, num_matches) }
}

/// Complete 'fillchars' or 'listchars'. Which one is decided by the variable
/// being set, since the two share every entry point.
///
/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_chars_option(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's frame; the comparison is of addresses only.
    let varp = unsafe { (*args).oe_varp }.cast::<*mut c_char>();
    let is_lcs =
        varp == p_lcs.ptr() || varp == unsafe { &raw mut (*curwin.get()).w_onebuf_opt.wo_lcs };
    let names = if is_lcs {
        get_listchars_name
    } else {
        get_fillchars_name
    };
    unsafe { expand_set_opt_generic(args, Some(names), num_matches, matches) }
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_concealcursor(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_listflag(args, COCU_ALL.as_ptr(), num_matches, matches) }
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_cpoptions(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_listflag(args, CPO_VI.as_ptr(), num_matches, matches) }
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_formatoptions(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_listflag(args, FO_ALL.as_ptr(), num_matches, matches) }
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_mouse(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_listflag(args, MOUSE_ALL.as_ptr(), num_matches, matches) }
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_shortmess(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_listflag(args, SHM_ALL.as_ptr(), num_matches, matches) }
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_whichwrap(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_listflag(args, WW_ALL.as_ptr(), num_matches, matches) }
}

/// Does the word being completed sit directly after `prefix` inside the
/// `:set` argument?
///
/// # Safety
/// `at` points into the C string starting at `start`.
unsafe fn directly_after(at: *const c_char, start: *const c_char, prefix: &CStr) -> bool {
    let len = prefix.to_bytes().len();
    // SAFETY: both point into the same string, as documented above, and the
    // length test is what puts the `sub` in range.
    unsafe {
        at.offset_from(start) >= len as isize && strncmp(at.sub(len), prefix.as_ptr(), len) == 0
    }
}

/// Complete 'diffopt', whose "algorithm:" and "inline:" fields each have
/// their own list of accepted words. Anything else after a `:` has no
/// completions at all.
///
/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_diffopt(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's frame; `xp_pattern` points into `oe_set_arg`.
    let (xp, start) = unsafe { ((*args).oe_xp, (*args).oe_set_arg) };
    let at = unsafe { (*xp).xp_pattern };
    if at <= start || unsafe { *at.sub(1) } != b':' as c_char {
        return unsafe { expand_set_str_generic(args, num_matches, matches) };
    }
    // The last entry of each array is the null terminator, not a value.
    let field = |values: &GlobalCell<[*const c_char; 5]>| unsafe {
        expand_set_opt_string(
            args,
            values.ptr().cast::<*const c_char>(),
            4,
            num_matches,
            matches,
        )
    };
    if unsafe { directly_after(at, start, c"algorithm:") } {
        return field(&opt_dip_algorithm_values);
    }
    if unsafe { directly_after(at, start, c"inline:") } {
        return field(&opt_dip_inline_values);
    }
    FAIL
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_encoding(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_generic(args, Some(get_encoding_name), num_matches, matches) }
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_winhighlight(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    unsafe { expand_set_opt_generic(args, Some(get_highlight_name), num_matches, matches) }
}

/// Whether the option being completed is a window-local 'eventignorewin'
/// rather than the global 'eventignore', which decides which events are
/// eligible. `expand_generic` gives the enumerator no other way to know.
static WINDOW_EVENTS: GlobalCell<bool> = GlobalCell::new(false);

/// Enumerate the autocommand event names 'eventignore' accepts, with "all"
/// ahead of them, and each one prefixed by "-" when the user is subtracting.
///
/// # Safety
/// Called by `expand_generic` with its expansion context.
pub(crate) unsafe fn get_eventignore_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    // SAFETY: the expansion context's pattern is a C string.
    let subtract = unsafe { *(*xp).xp_pattern } == b'-' as c_char;
    if !subtract && idx == 0 {
        return c"all".as_ptr().cast_mut();
    }
    let name = get_event_name_no_group(xp, idx - 1 + c_int::from(subtract), WINDOW_EVENTS.get());
    if name.is_null() {
        return ptr::null_mut();
    }
    let buffer = IObuff.ptr().cast::<c_char>();
    // SAFETY: `IObuff` is the shared `IOSIZE`-byte scratch buffer, and
    // `name` is a C string.
    unsafe {
        snprintf(
            buffer,
            IOSIZE as size_t,
            c"%s%s".as_ptr(),
            if subtract {
                c"-".as_ptr()
            } else {
                c"".as_ptr()
            },
            name,
        );
    }
    buffer
}

/// # Safety
/// `args` points at the completion frame.
pub unsafe fn expand_set_eventignore(
    args: *mut optexpand_T,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's frame; the comparison is of addresses only.
    WINDOW_EVENTS.set(unsafe { (*args).oe_varp }.cast::<c_void>() != p_ei.ptr().cast::<c_void>());
    unsafe { expand_set_opt_generic(args, Some(get_eventignore_name), num_matches, matches) }
}

/// Enumerate the values 'fileformat' accepts.
///
/// # Safety
/// Called by `expand_generic`.
pub unsafe fn get_fileformat_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    // SAFETY: the table's own array. Its last entry is the null terminator,
    // which is also how `expand_generic` learns the list has ended.
    unsafe { *opt_ff_values.ptr() }
        .get(idx as usize)
        .map_or(ptr::null_mut(), |value| value.cast_mut())
}
