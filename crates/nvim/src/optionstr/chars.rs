//! 'fillchars' and 'listchars': a field list parsed into the character
//! tables the screen draws from.
//!
//! Both options are a comma-separated list of `name:chars` fields, and both
//! are set through the same code with a [`CharsOption`] saying which. The
//! field tables below say where each field's character lands in the window's
//! `fcs_chars_T`/`lcs_chars_T`, and what it falls back to when the value
//! does not mention it.
//!
//! Three things about this are easy to get wrong.
//!
//! **Two rounds.** The value is walked once to validate and, only if that
//! succeeds, a second time to assign — so a bad field leaves the previous
//! value entirely intact. The second round starts by resetting every field
//! to its default, which is why "unset" and "set to the default" are the
//! same thing here.
//!
//! **A field may appear more than once**, and the last mention wins. For
//! the single-character fields that falls out of the assignment order; for
//! "multispace:" and "leadmultispace:", which fill a separately allocated
//! run, the first round has to remember *which* mention was last so the
//! second round fills from that one.
//!
//! **A default that is too wide is not used.** Upstream's fallbacks exist
//! because a double-width character cannot go in one cell, so the box-drawing
//! defaults degrade to ASCII when the encoding cannot render them narrow.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::mem::offset_of;
use core::{ptr, slice};

use crate::charset::{char2cells, hexhex2nr, ptr2cells};
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::grid::{schar_from_char, schar_from_str};
use crate::main::{curwin, e_invarg, e_leadtab_requires_tab, p_fcs, p_lcs};
use crate::mbyte::{utfc_ptr2len, utfc_ptr2schar};
use crate::memory::{xfree, xmalloc};
use crate::option::option_var;
use crate::options::kOptListchars as kOptListcharsIdx;
use crate::os::cshim::gettext_ptr;
use crate::strings::vim_snprintf;
use crate::types::{
    CharsOption, NUL, OptionSetFlags, expand_T, fcs_chars_T, int64_t, lcs_chars_T, optset_T,
    schar_T, size_t, win_T,
};
use crate::winlayer;

use super::{
    clear_string_option, e_conflicts_with_value_of_fillchars, e_conflicts_with_value_of_listchars,
    e_wrong_character_width_for_field_str, e_wrong_number_of_characters_for_field_str, kFillchars,
    kListchars,
};

/// A 'fillchars' struct with every field blank -- what the assignment round
/// starts from, before the defaults and then the value fill it in.
const NO_FILL_CHARS: fcs_chars_T = fcs_chars_T {
    stl: 0,
    stlnc: 0,
    wbr: 0,
    horiz: 0,
    horizup: 0,
    horizdown: 0,
    vert: 0,
    vertleft: 0,
    vertright: 0,
    verthoriz: 0,
    fold: 0,
    foldopen: 0,
    foldclosed: 0,
    foldsep: 0,
    foldinner: 0,
    diff: 0,
    msgsep: 0,
    eob: 0,
    lastline: 0,
    trunc: 0,
    truncrl: 0,
};

/// A 'listchars' struct with every field blank, owning no runs.
const NO_LIST_CHARS: lcs_chars_T = lcs_chars_T {
    eol: 0,
    ext: 0,
    prec: 0,
    nbsp: 0,
    space: 0,
    tab1: 0,
    tab2: 0,
    tab3: 0,
    leadtab1: 0,
    leadtab2: 0,
    leadtab3: 0,
    lead: 0,
    trail: 0,
    multispace: ::core::ptr::null_mut::<schar_T>(),
    leadmultispace: ::core::ptr::null_mut::<schar_T>(),
    conceal: 0,
};

/// What a field does with the characters it is given, beyond the one
/// character every field takes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// One character, and nothing else. Every 'fillchars' field, and most
    /// of 'listchars'.
    Single,
    /// 'listchars' "tab:": two characters and an optional third, which land
    /// in `tab1`/`tab2`/`tab3` together.
    Tab,
    /// 'listchars' "leadtab:": the same, into `leadtab1`/`leadtab2`/
    /// `leadtab3`.
    LeadTab,
    /// 'listchars' "multispace:": a run of characters, however many, into a
    /// separately allocated array.
    Multispace,
    /// 'listchars' "leadmultispace:": the same.
    LeadMultispace,
}

/// One field of 'fillchars' or 'listchars'.
struct Field {
    name: &'static CStr,
    /// Byte offset of the `schar_T` this field's character fills, within
    /// the option's character struct. The two run-valued fields have none.
    slot: Option<usize>,
    /// The preferred default, used only when it fits in one screen cell.
    def: Option<&'static CStr>,
    /// The default to use when `def` is too wide.
    fallback: Option<&'static CStr>,
    shape: Shape,
}

/// A single-character field, spelled the way upstream's `CHARSTAB_ENTRY`
/// spells one.
const fn one(
    name: &'static CStr,
    slot: usize,
    def: Option<&'static CStr>,
    fallback: Option<&'static CStr>,
) -> Field {
    Field {
        name,
        slot: Some(slot),
        def,
        fallback,
        shape: Shape::Single,
    }
}

/// The fields of 'fillchars', with the defaults that make a box-drawing
/// terminal look right and the ASCII fallbacks for one that cannot.
static FCS_TAB: [Field; 21] = [
    one(c"stl", offset_of!(fcs_chars_T, stl), Some(c" "), None),
    one(c"stlnc", offset_of!(fcs_chars_T, stlnc), Some(c" "), None),
    one(c"wbr", offset_of!(fcs_chars_T, wbr), Some(c" "), None),
    one(
        c"horiz",
        offset_of!(fcs_chars_T, horiz),
        Some(c"\u{2500}"),
        Some(c"-"),
    ),
    one(
        c"horizup",
        offset_of!(fcs_chars_T, horizup),
        Some(c"\u{2534}"),
        Some(c"-"),
    ),
    one(
        c"horizdown",
        offset_of!(fcs_chars_T, horizdown),
        Some(c"\u{252c}"),
        Some(c"-"),
    ),
    one(
        c"vert",
        offset_of!(fcs_chars_T, vert),
        Some(c"\u{2502}"),
        Some(c"|"),
    ),
    one(
        c"vertleft",
        offset_of!(fcs_chars_T, vertleft),
        Some(c"\u{2524}"),
        Some(c"|"),
    ),
    one(
        c"vertright",
        offset_of!(fcs_chars_T, vertright),
        Some(c"\u{251c}"),
        Some(c"|"),
    ),
    one(
        c"verthoriz",
        offset_of!(fcs_chars_T, verthoriz),
        Some(c"\u{253c}"),
        Some(c"+"),
    ),
    one(
        c"fold",
        offset_of!(fcs_chars_T, fold),
        Some(c"\u{b7}"),
        Some(c"-"),
    ),
    one(
        c"foldopen",
        offset_of!(fcs_chars_T, foldopen),
        Some(c"-"),
        None,
    ),
    // Note the name: the field is "foldclose", the struct member
    // `foldclosed`.
    one(
        c"foldclose",
        offset_of!(fcs_chars_T, foldclosed),
        Some(c"+"),
        None,
    ),
    one(
        c"foldsep",
        offset_of!(fcs_chars_T, foldsep),
        Some(c"\u{2502}"),
        Some(c"|"),
    ),
    one(c"foldinner", offset_of!(fcs_chars_T, foldinner), None, None),
    one(c"diff", offset_of!(fcs_chars_T, diff), Some(c"-"), None),
    one(c"msgsep", offset_of!(fcs_chars_T, msgsep), Some(c" "), None),
    one(c"eob", offset_of!(fcs_chars_T, eob), Some(c"~"), None),
    one(
        c"lastline",
        offset_of!(fcs_chars_T, lastline),
        Some(c"@"),
        None,
    ),
    one(c"trunc", offset_of!(fcs_chars_T, trunc), Some(c">"), None),
    one(
        c"truncrl",
        offset_of!(fcs_chars_T, truncrl),
        Some(c"<"),
        None,
    ),
];

/// The fields of 'listchars'. None of them has a default: an unmentioned
/// field draws nothing.
static LCS_TAB: [Field; 12] = [
    one(c"eol", offset_of!(lcs_chars_T, eol), None, None),
    one(c"extends", offset_of!(lcs_chars_T, ext), None, None),
    one(c"nbsp", offset_of!(lcs_chars_T, nbsp), None, None),
    one(c"precedes", offset_of!(lcs_chars_T, prec), None, None),
    one(c"space", offset_of!(lcs_chars_T, space), None, None),
    Field {
        name: c"tab",
        slot: Some(offset_of!(lcs_chars_T, tab2)),
        def: None,
        fallback: None,
        shape: Shape::Tab,
    },
    Field {
        name: c"leadtab",
        slot: Some(offset_of!(lcs_chars_T, leadtab2)),
        def: None,
        fallback: None,
        shape: Shape::LeadTab,
    },
    one(c"lead", offset_of!(lcs_chars_T, lead), None, None),
    one(c"trail", offset_of!(lcs_chars_T, trail), None, None),
    one(c"conceal", offset_of!(lcs_chars_T, conceal), None, None),
    Field {
        name: c"multispace",
        slot: None,
        def: None,
        fallback: None,
        shape: Shape::Multispace,
    },
    Field {
        name: c"leadmultispace",
        slot: None,
        def: None,
        fallback: None,
        shape: Shape::LeadMultispace,
    },
];

/// Is this the 'listchars' half of the shared machinery?
fn is_listchars(what: CharsOption) -> bool {
    what as c_uint == kListchars as c_uint
}

/// Read one character of a field's value and step `at` past it.
///
/// A `\x`, `\u` or `\U` escape is read as that many hex digit pairs;
/// anything else is one (possibly composed) character. Answers 0 — which
/// every caller treats as a rejection — for invalid hex, for an invalid
/// UTF-8 byte, and for a character too wide to sit in one screen cell.
fn take_encoded_char(value: &CStr, at: &mut usize) -> schar_T {
    let bytes = value.to_bytes();
    debug_assert!(*at <= bytes.len());
    // SAFETY: `value` is NUL-terminated, so every read below stops at the
    // terminator at the latest; `at` never passes it, because the hex
    // reader gives up at the first byte that is not a hex digit and the
    // character reader steps by the length of the character it just read.
    let start = unsafe { value.as_ptr().add(*at) };
    let pairs = match (bytes.get(*at), bytes.get(*at + 1)) {
        (Some(b'\\'), Some(b'x')) => 1,
        (Some(b'\\'), Some(b'u')) => 2,
        (Some(b'\\'), Some(b'U')) => 4,
        _ => 0,
    };
    if pairs > 0 {
        let mut num: int64_t = 0;
        for _ in 0..pairs {
            *at += 2;
            let digits = unsafe { hexhex2nr(value.as_ptr().add(*at)) };
            if digits < 0 {
                return 0;
            }
            num = num * 256 + int64_t::from(digits);
        }
        *at += 2;
        return if unsafe { char2cells(num as c_int) } > 1 {
            0
        } else {
            schar_from_char(num as c_int)
        };
    }

    let clen = unsafe { utfc_ptr2len(start) };
    let mut firstc: c_int = 0;
    let c = unsafe { utfc_ptr2schar(start, &raw mut firstc) };
    *at += clen as usize;
    // An invalid UTF-8 byte, or a double-width character.
    if (clen == 1 && firstc > 127) || unsafe { char2cells(firstc) } > 1 {
        0
    } else {
        c
    }
}

/// "E1511: Wrong number of characters for field \"x\"" and its width
/// sibling, formatted into the caller's buffer. A null buffer means the
/// caller wants no message, and gets the shared empty string.
///
/// # Safety
/// `errbuf` is null or points at `errbuflen` writable bytes; `fmt` takes
/// one string argument.
unsafe fn field_value_err<'a>(
    errbuf: *mut c_char,
    errbuflen: size_t,
    fmt: *const c_char,
    field: &CStr,
) -> &'a CStr {
    if errbuf.is_null() {
        return c"";
    }
    // SAFETY: the caller's buffer and format, with the one argument it
    // takes.
    unsafe { vim_snprintf(errbuf, errbuflen, gettext_ptr(fmt).as_ptr(), field.as_ptr()) };
    // SAFETY: `vim_snprintf` terminated what it wrote.
    unsafe { CStr::from_ptr(errbuf) }
}

/// A character struct as raw bytes, for the fields the table addresses by
/// `offset_of!` rather than by name.
///
/// Derived fresh at each use rather than held: the named fields are written
/// directly, and a pointer kept across such a write would be stale.
fn chars_bytes<T>(chars: &mut T) -> &mut [u8] {
    // SAFETY: any value is readable and writable as its own bytes, and the
    // exclusive borrow is what keeps this the only way in while it lasts.
    unsafe { slice::from_raw_parts_mut(ptr::from_mut(chars).cast::<u8>(), size_of::<T>()) }
}

/// Write `value` into the `schar_T` field at byte offset `slot`.
fn store_field(chars: &mut [u8], slot: usize, value: schar_T) {
    chars[slot..slot + size_of::<schar_T>()].copy_from_slice(&value.to_ne_bytes());
}

/// Set 'fillchars' or 'listchars' for one window.
///
/// `value` points at either the global or the window-local value; an empty
/// window-local value means "use the global one". With `apply` false only
/// the check runs, which is how `check_chars_options` asks whether a value
/// would be accepted without disturbing anything.
///
/// Returns an error message, or null when the value is good.
///
/// # Safety
/// `wp` is a live window, `value` a C string, and `errbuf` null or
/// `errbuflen` writable bytes.
pub unsafe fn set_chars_option<'a>(
    wp: *mut win_T,
    value: *const c_char,
    what: CharsOption,
    apply: bool,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> Option<&'a CStr> {
    let listchars = is_listchars(what);
    let tab: &[Field] = if listchars { &LCS_TAB } else { &FCS_TAB };
    // SAFETY: the caller's window; both are C strings.
    let local = unsafe {
        if listchars {
            (*wp).w_onebuf_opt.wo_lcs
        } else {
            (*wp).w_onebuf_opt.wo_fcs
        }
    };
    // An empty local value defers to the global one.
    let value = if unsafe { c_int::from(*local) } == NUL {
        if listchars { p_lcs.get() } else { p_fcs.get() }
    } else {
        value.cast_mut()
    };
    // SAFETY: an option value is a C string.
    let value = unsafe { CStr::from_ptr(value) };
    // The struct this call fills in and, when `apply`, hands to the window.
    // Only one of the two is ever used; which one is `listchars`.
    let mut lcs = NO_LIST_CHARS;
    let mut fcs = NO_FILL_CHARS;

    // The offset of the last "multispace:"/"leadmultispace:" field in the
    // value, and how many characters it names. The first round works these
    // out; the second fills the runs from them.
    let mut last_multispace: Option<usize> = None;
    let mut last_lead_multispace: Option<usize> = None;
    let mut multispace_len = 0;
    let mut lead_multispace_len = 0;

    // First round: check that the value is valid. Second round, only once
    // the first passed: assign.
    for round in 0..=c_int::from(apply) {
        let mut has_tab = false;
        let mut has_leadtab = false;

        if round > 0 {
            if listchars {
                install_defaults(chars_bytes(&mut lcs), tab);
                lcs.tab1 = NUL as schar_T;
                lcs.tab3 = NUL as schar_T;
                lcs.leadtab1 = NUL as schar_T;
                lcs.leadtab3 = NUL as schar_T;
                // SAFETY: both runs are handed to the window with the struct.
                lcs.multispace = unsafe { alloc_run(multispace_len) };
                lcs.leadmultispace = unsafe { alloc_run(lead_multispace_len) };
            } else {
                install_defaults(chars_bytes(&mut fcs), tab);
            }
        }

        let bytes = value.to_bytes();
        let mut p = 0;
        while p < bytes.len() {
            let Some(i) = tab
                .iter()
                .position(|field| field_opens_at(bytes, p, field.name))
            else {
                return Some(e_invarg);
            };
            let field = &tab[i];
            let mut s = p + field.name.to_bytes().len() + 1;
            let width_err = |name| unsafe {
                field_value_err(
                    errbuf,
                    errbuflen,
                    e_wrong_character_width_for_field_str.as_ptr(),
                    name,
                )
            };
            let count_err = |name| unsafe {
                field_value_err(
                    errbuf,
                    errbuflen,
                    e_wrong_number_of_characters_for_field_str.as_ptr(),
                    name,
                )
            };

            match field.shape {
                Shape::Multispace | Shape::LeadMultispace => {
                    let lead = field.shape == Shape::LeadMultispace;
                    let (last, len) = if lead {
                        (&mut last_lead_multispace, &mut lead_multispace_len)
                    } else {
                        (&mut last_multispace, &mut multispace_len)
                    };
                    if round == 0 {
                        *last = Some(p);
                        *len = 0;
                        while !at_field_end(bytes, s) {
                            if take_encoded_char(value, &mut s) == 0 {
                                return Some(width_err(field.name));
                            }
                            *len += 1;
                        }
                        // The field cannot be empty.
                        if *len == 0 {
                            return Some(count_err(field.name));
                        }
                    } else {
                        // Only the last mention of the field fills the run;
                        // any earlier one is walked past and dropped.
                        let fills = *last == Some(p);
                        let run = if lead {
                            lcs.leadmultispace
                        } else {
                            lcs.multispace
                        };
                        let mut into = 0;
                        while !at_field_end(bytes, s) {
                            let c = take_encoded_char(value, &mut s);
                            if fills {
                                // SAFETY: the run was allocated for exactly
                                // the count the first round arrived at, and
                                // this is the same walk over the same
                                // field.
                                unsafe { *run.add(into) = c };
                                into += 1;
                            }
                        }
                    }
                    p = s;
                }
                _ => {
                    if at_end(bytes, s) {
                        return Some(count_err(field.name));
                    }
                    let c1 = take_encoded_char(value, &mut s);
                    if c1 == 0 {
                        return Some(width_err(field.name));
                    }
                    let mut c2: schar_T = 0;
                    let mut c3: schar_T = 0;
                    if matches!(field.shape, Shape::Tab | Shape::LeadTab) {
                        if at_end(bytes, s) {
                            return Some(count_err(field.name));
                        }
                        c2 = take_encoded_char(value, &mut s);
                        if c2 == 0 {
                            return Some(width_err(field.name));
                        }
                        // The third character is optional.
                        if !at_field_end(bytes, s) {
                            c3 = take_encoded_char(value, &mut s);
                            if c3 == 0 {
                                return Some(width_err(field.name));
                            }
                        }
                        if field.shape == Shape::Tab {
                            has_tab = true;
                        } else {
                            has_leadtab = true;
                        }
                    }
                    if !at_field_end(bytes, s) {
                        return Some(count_err(field.name));
                    }
                    if round > 0 {
                        match field.shape {
                            Shape::Tab => {
                                lcs.tab1 = c1;
                                lcs.tab2 = c2;
                                lcs.tab3 = c3;
                            }
                            Shape::LeadTab => {
                                lcs.leadtab1 = c1;
                                lcs.leadtab2 = c2;
                                lcs.leadtab3 = c3;
                            }
                            _ => {
                                if let Some(slot) = field.slot {
                                    if listchars {
                                        store_field(chars_bytes(&mut lcs), slot, c1);
                                    } else {
                                        store_field(chars_bytes(&mut fcs), slot, c1);
                                    }
                                }
                            }
                        }
                    }
                    p = s;
                }
            }

            if bytes.get(p) == Some(&b',') {
                p += 1;
            }
        }

        if listchars && has_leadtab && !has_tab {
            return Some(e_leadtab_requires_tab);
        }
    }

    if apply {
        // SAFETY: the caller's window; the two runs it held are this
        // module's to free, and the new ones move into the struct with it.
        if listchars {
            unsafe { xfree((*wp).w_p_lcs_chars.multispace.cast::<c_void>()) };
            unsafe { xfree((*wp).w_p_lcs_chars.leadmultispace.cast::<c_void>()) };
            unsafe { (*wp).w_p_lcs_chars = lcs };
        } else {
            unsafe { (*wp).w_p_fcs_chars = fcs };
        }
    }
    None
}

/// Does the field named `name` start at `p`? A field name is followed by a
/// colon.
fn field_opens_at(value: &[u8], p: usize, name: &CStr) -> bool {
    let name = name.to_bytes();
    value[p..].starts_with(name) && value.get(p + name.len()) == Some(&b':')
}

/// Is the cursor at the end of a field — the end of the value, or the comma
/// that starts the next one?
fn at_field_end(value: &[u8], at: usize) -> bool {
    matches!(value.get(at), None | Some(&b','))
}

/// Is the cursor at the end of the whole value?
fn at_end(value: &[u8], at: usize) -> bool {
    at >= value.len()
}

/// Give every field its default, before the assignment round overwrites the
/// ones the value mentions. A default that does not fit in one screen cell
/// is not used; the field falls back, and a field with neither ends up
/// blank.
///
/// `chars` is the character struct `tab`'s slots were taken from, as bytes.
fn install_defaults(chars: &mut [u8], tab: &[Field]) {
    for field in tab {
        let Some(slot) = field.slot else {
            continue;
        };
        // SAFETY: `ptr2cells` only reads the C string it is given.
        let narrow = field
            .def
            .is_some_and(|def| unsafe { ptr2cells(def.as_ptr()) } == 1);
        let text = if narrow { field.def } else { field.fallback };
        let text = text.map_or(ptr::null(), CStr::as_ptr);
        // SAFETY: `schar_from_str` accepts a null pointer as "nothing".
        store_field(chars, slot, unsafe { schar_from_str(text) });
    }
}

/// Allocate the run of characters a "multispace:" field fills, terminated
/// like a string. A zero-length run is no allocation at all.
///
/// # Safety
/// The result is handed to the window along with the rest of the character
/// struct, and freed there.
unsafe fn alloc_run(len: c_int) -> *mut schar_T {
    if len <= 0 {
        return ptr::null_mut();
    }
    let count = len as size_t + 1;
    // SAFETY: `xmalloc` returns an allocation of that size or aborts.
    let run = unsafe { xmalloc(count * size_of::<schar_T>()) }.cast::<schar_T>();
    // SAFETY: the last element of the allocation just made.
    unsafe { *run.add(len as usize) = NUL as schar_T };
    run
}

/// Set the global 'fillchars' or 'listchars', and re-derive every window's
/// characters from it.
///
/// A `:set` without `setglobal` also clears the window's local value, so
/// that the window follows the global one again. Every other window that
/// has no local value of its own is refreshed too, because they were all
/// drawing from the value that just changed.
///
/// # Safety
/// `win` is a live window, `val` a C string, `errbuf` null or `errbuflen`
/// writable bytes.
pub(crate) unsafe fn did_set_global_chars_option<'a>(
    win: *mut win_T,
    val: *mut c_char,
    what: CharsOption,
    opt_flags: OptionSetFlags,
    errbuf: *mut c_char,
    errbuflen: size_t,
) -> Option<&'a CStr> {
    let listchars = is_listchars(what);
    // SAFETY: the caller's window.
    let local_ptr = unsafe {
        if listchars {
            &raw mut (*win).w_onebuf_opt.wo_lcs
        } else {
            &raw mut (*win).w_onebuf_opt.wo_fcs
        }
    };
    let local_is_empty = unsafe { c_int::from(**local_ptr) } == NUL;
    let for_this_window = local_is_empty || !opt_flags.has(OptionSetFlags::GLOBAL);

    // SAFETY: the caller's window and value.
    let errmsg = unsafe { set_chars_option(win, val, what, for_this_window, errbuf, errbuflen) };
    if errmsg.is_some() {
        return errmsg;
    }

    if !opt_flags.has(OptionSetFlags::GLOBAL) {
        // SAFETY: the window's own option variable.
        unsafe { clear_string_option(local_ptr) };
    }

    // SAFETY: `for_each_window` only visits live windows.
    unsafe {
        for_each_window(|wp| {
            let opt = if listchars {
                (*wp).w_onebuf_opt.wo_lcs
            } else {
                (*wp).w_onebuf_opt.wo_fcs
            };
            if c_int::from(*opt) == NUL {
                set_chars_option(wp, opt, what, true, errbuf, errbuflen);
            }
            None
        })
    };
    // SAFETY: `redraw_all_later` only marks the editor's own windows.
    unsafe { redraw_all_later(UPD_NOT_VALID) };
    None
}

/// The option-table callback for both options and both scopes: which of the
/// four cases this is comes from the variable being set.
///
/// # Safety
/// `args` points at the option table's call frame.
pub unsafe fn did_set_chars_option(args: &mut optset_T) -> Option<&CStr> {
    let (win, varp, idx, flags, errbuf, errbuflen) = (
        args.os_win.cast::<win_T>(),
        args.os_varp.string_var(),
        args.os_idx,
        args.os_flags,
        args.os_errbuf,
        args.os_errbuflen,
    );
    // 'listchars' and 'fillchars' share this callback, so the row says which
    // option it is and the variable says which *scope*: the option's own
    // global one, or this window's copy.
    let which = if idx == kOptListcharsIdx {
        kListchars
    } else {
        kFillchars
    };
    // SAFETY: the caller's frame and window; the comparisons are of
    // addresses only.
    if varp == option_var(idx).string_var() {
        unsafe { did_set_global_chars_option(win, *varp, which, flags, errbuf, errbuflen) }
    } else if varp == unsafe { &raw mut (*win).w_onebuf_opt.wo_lcs }
        || varp == unsafe { &raw mut (*win).w_onebuf_opt.wo_fcs }
    {
        unsafe { set_chars_option(win, *varp, which, true, errbuf, errbuflen) }
    } else {
        None
    }
}

/// Enumerate the field names of 'fillchars', for completion.
pub fn get_fillchars_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    field_name(&FCS_TAB, idx)
}

/// Enumerate the field names of 'listchars', for completion.
pub fn get_listchars_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    field_name(&LCS_TAB, idx)
}

/// The `idx`th field name, or null once the table has run out — which is
/// how `expand_generic` learns the list has ended.
fn field_name(tab: &'static [Field], idx: c_int) -> *mut c_char {
    usize::try_from(idx)
        .ok()
        .and_then(|idx| tab.get(idx))
        .map_or(ptr::null_mut(), |field| field.name.as_ptr().cast_mut())
}

/// Would the current 'fillchars' and 'listchars' still be accepted?
///
/// Called after something other than `:set` changed what the screen can
/// render — a new 'encoding', say — and reports which of the two options
/// the new state conflicts with.
///
/// # Safety
/// Reads the editor's window list.
pub unsafe fn check_chars_options() -> Option<&'static CStr> {
    let check = |wp, value, what, apply| {
        // SAFETY: a live window and a C string; no message is wanted.
        if unsafe { set_chars_option(wp, value, what, apply, ptr::null_mut(), 0) }.is_none() {
            None
        } else if is_listchars(what) {
            Some(e_conflicts_with_value_of_listchars)
        } else {
            Some(e_conflicts_with_value_of_fillchars)
        }
    };

    if let Some(global) = check(curwin.get(), p_lcs.get(), kListchars, false) {
        return Some(global);
    }
    if let Some(global) = check(curwin.get(), p_fcs.get(), kFillchars, false) {
        return Some(global);
    }
    // SAFETY: `for_each_window` only visits live windows.
    for_each_window(|wp| {
        if let Some(errmsg) = check(wp, unsafe { (*wp).w_onebuf_opt.wo_lcs }, kListchars, true) {
            return Some(errmsg);
        }
        check(wp, unsafe { (*wp).w_onebuf_opt.wo_fcs }, kFillchars, true)
    })
}

/// Walk every window of every tab page, stopping at the first message a
/// visit returns.
///
/// `FOR_ALL_TAB_WINDOWS`, i.e. [`winlayer::tab_windows`] -- which already
/// knows that the current tab page's windows hang off `firstwin` rather than
/// off its own stale list.
fn for_each_window(
    mut visit: impl FnMut(*mut win_T) -> Option<&'static CStr>,
) -> Option<&'static CStr> {
    for wp in winlayer::tab_windows() {
        if let Some(errmsg) = visit(wp.raw()) {
            return Some(errmsg);
        }
    }
    None
}
