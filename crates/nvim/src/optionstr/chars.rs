//! 'fillchars' and 'listchars': a field list parsed into the character
//! tables the screen draws from.
//!
//! Both options are a comma-separated list of `name:chars` fields, and both
//! are set through the same code with a [`CharsOption`] saying which. The
//! field tables below say where each field's character lands in the window's
//! `FcsChars`/`LcsChars`, and what it falls back to when the value
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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::formatted;
use crate::global_cell::field;
use crate::types::Candidate;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int, c_uint};

use crate::charset::{char2cells, hexhex_at, ptr2cells_at};
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::grid::schar_from_char;
use crate::mbyte::{c_cluster_len, c_cluster_schar};
use crate::memory::XString;
use crate::message::{e_invarg, e_leadtab_requires_tab};
use crate::narrow::number_as_int;
use crate::option::vars::{P_FCS, P_LCS};
use crate::option::{StrVar, option_var};
use crate::options::kOptListchars as kOptListcharsIdx;
use crate::os::cshim::gettext;
use crate::types::{
    CharsOption, Expand, FcsChars, LcsChars, NUL, OptError, OptSet, OptionSetFlags, ScreenChar,
    WinOpt, int64_t,
};
use crate::winlayer;

use super::frame::new_bytes;
use super::{
    OptStringRef, e_conflicts_with_value_of_fillchars, e_conflicts_with_value_of_listchars,
    e_wrong_character_width_for_field_str, e_wrong_number_of_characters_for_field_str, kFillchars,
    kListchars,
};

/// A 'fillchars' struct with every field blank -- what the assignment round
/// starts from, before the defaults and then the value fill it in.
const NO_FILL_CHARS: FcsChars = FcsChars {
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
const NO_LIST_CHARS: LcsChars = LcsChars {
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
    multispace: None,
    leadmultispace: None,
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

/// Where a field's character lands: the `ScreenChar` of the option's
/// character struct it fills.
#[derive(Clone, Copy)]
enum Slot {
    /// A field of a window's 'fillchars' characters.
    Fill(fn(&mut FcsChars) -> &mut ScreenChar),
    /// A field of a window's 'listchars' characters.
    List(fn(&mut LcsChars) -> &mut ScreenChar),
}

/// The two character structs a parse fills, one of which it hands over.
struct Chars {
    fill: FcsChars,
    list: LcsChars,
}

impl Chars {
    /// Write `c` into the field `slot` names.
    fn store(&mut self, slot: Slot, c: ScreenChar) {
        match slot {
            Slot::Fill(field) => *field(&mut self.fill) = c,
            Slot::List(field) => *field(&mut self.list) = c,
        }
    }
}

/// A 'fillchars' slot, by field name.
macro_rules! fill {
    ($field:ident) => {
        Slot::Fill(|chars| &mut chars.$field)
    };
}

/// A 'listchars' slot, by field name.
macro_rules! list {
    ($field:ident) => {
        Slot::List(|chars| &mut chars.$field)
    };
}

/// One field of 'fillchars' or 'listchars'.
struct Field {
    name: &'static CStr,
    /// The `ScreenChar` this field's character fills. The two run-valued
    /// fields have none.
    slot: Option<Slot>,
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
    slot: Slot,
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
    one(c"stl", fill!(stl), Some(c" "), None),
    one(c"stlnc", fill!(stlnc), Some(c" "), None),
    one(c"wbr", fill!(wbr), Some(c" "), None),
    one(c"horiz", fill!(horiz), Some(c"\u{2500}"), Some(c"-")),
    one(c"horizup", fill!(horizup), Some(c"\u{2534}"), Some(c"-")),
    one(
        c"horizdown",
        fill!(horizdown),
        Some(c"\u{252c}"),
        Some(c"-"),
    ),
    one(c"vert", fill!(vert), Some(c"\u{2502}"), Some(c"|")),
    one(c"vertleft", fill!(vertleft), Some(c"\u{2524}"), Some(c"|")),
    one(
        c"vertright",
        fill!(vertright),
        Some(c"\u{251c}"),
        Some(c"|"),
    ),
    one(
        c"verthoriz",
        fill!(verthoriz),
        Some(c"\u{253c}"),
        Some(c"+"),
    ),
    one(c"fold", fill!(fold), Some(c"\u{b7}"), Some(c"-")),
    one(c"foldopen", fill!(foldopen), Some(c"-"), None),
    // Note the name: the field is "foldclose", the struct member
    // `foldclosed`.
    one(c"foldclose", fill!(foldclosed), Some(c"+"), None),
    one(c"foldsep", fill!(foldsep), Some(c"\u{2502}"), Some(c"|")),
    one(c"foldinner", fill!(foldinner), None, None),
    one(c"diff", fill!(diff), Some(c"-"), None),
    one(c"msgsep", fill!(msgsep), Some(c" "), None),
    one(c"eob", fill!(eob), Some(c"~"), None),
    one(c"lastline", fill!(lastline), Some(c"@"), None),
    one(c"trunc", fill!(trunc), Some(c">"), None),
    one(c"truncrl", fill!(truncrl), Some(c"<"), None),
];

/// The fields of 'listchars'. None of them has a default: an unmentioned
/// field draws nothing.
static LCS_TAB: [Field; 12] = [
    one(c"eol", list!(eol), None, None),
    one(c"extends", list!(ext), None, None),
    one(c"nbsp", list!(nbsp), None, None),
    one(c"precedes", list!(prec), None, None),
    one(c"space", list!(space), None, None),
    Field {
        name: c"tab",
        slot: Some(list!(tab2)),
        def: None,
        fallback: None,
        shape: Shape::Tab,
    },
    Field {
        name: c"leadtab",
        slot: Some(list!(leadtab2)),
        def: None,
        fallback: None,
        shape: Shape::LeadTab,
    },
    one(c"lead", list!(lead), None, None),
    one(c"trail", list!(trail), None, None),
    one(c"conceal", list!(conceal), None, None),
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
///
/// `at` never passes the end of `bytes`: the hex reader gives up at the
/// first byte that is not a hex digit, and the character reader steps by
/// the length of the character it just read.
fn take_encoded_char(bytes: &[u8], at: &mut usize) -> ScreenChar {
    debug_assert!(*at <= bytes.len());
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
            let digits = hexhex_at(bytes.get(*at..).unwrap_or_default());
            if digits < 0 {
                return 0;
            }
            num = num * 256 + int64_t::from(digits);
        }
        *at += 2;
        return if char2cells(number_as_int(num)) > 1 {
            0
        } else {
            schar_from_char(number_as_int(num))
        };
    }

    let rest = bytes.get(*at..).unwrap_or_default();
    let clen = c_cluster_len(rest);
    let (c, firstc) = c_cluster_schar(rest);
    *at += clen;
    // An invalid UTF-8 byte, or a double-width character.
    if (clen == 1 && firstc > 127) || char2cells(firstc) > 1 {
        0
    } else {
        c
    }
}

/// "E1511: Wrong number of characters for field \"x\"" and its width
/// sibling, naming the field they did not like. `fmt` takes one string.
fn field_value_err(fmt: &'static CStr, field: &CStr) -> OptError {
    let message = formatted!(OptError::ROOM, gettext(fmt).to_bytes(), field);
    OptError::Owned(XString::from_vec(message))
}

/// [`set_chars_option`] over the window's own 'fillchars' or 'listchars',
/// read where it is used.
pub(crate) fn set_window_chars_option(
    window: Win,
    what: CharsOption,
    apply: bool,
) -> Result<(), OptError> {
    set_chars_option(window, None, what, apply)
}

/// Set 'fillchars' or 'listchars' for one window.
///
/// `value` is the global or the window-local value, or `None` for the
/// window's own; an empty window-local value means "use the global one".
/// With `apply` false only the check runs, which is how
/// `check_chars_options` asks whether a value would be accepted without
/// disturbing anything.
///
/// Returns an error message when the value is not good.
pub fn set_chars_option(
    mut window: Win,
    value: Option<&[u8]>,
    what: CharsOption,
    apply: bool,
) -> Result<(), OptError> {
    let listchars = is_listchars(what);
    // The value is read -- the window's own copy borrowed, the global one
    // copied, since this runs at `:set` rate -- and parsed before anything
    // writes the window.
    let parsed = {
        let local = if listchars {
            window.w_onebuf_opt.wo_lcs.bytes()
        } else {
            window.w_onebuf_opt.wo_fcs.bytes()
        };
        if local.is_empty() {
            // An empty local value defers to the global one.
            let global = if listchars { P_LCS.get() } else { P_FCS.get() };
            parse_chars(&global, listchars, apply)
        } else {
            parse_chars(value.unwrap_or(local), listchars, apply)
        }
    }?;
    if let Some(chars) = parsed {
        // The runs the window held go with the struct they were in.
        if listchars {
            window.w_p_lcs_chars = chars.list;
        } else {
            window.w_p_fcs_chars = chars.fill;
        }
    }
    Ok(())
}

/// Walk a 'fillchars' (or 'listchars') value: once to validate and, when
/// `apply` and only if that succeeded, a second time to fill the character
/// struct, which is the answer.
fn parse_chars(bytes: &[u8], listchars: bool, apply: bool) -> Result<Option<Chars>, OptError> {
    let tab: &[Field] = if listchars { &LCS_TAB } else { &FCS_TAB };
    // The struct this call fills in and, when `apply`, hands back. Only one
    // of the two is ever used; which one is `listchars`.
    let mut chars = Chars {
        fill: NO_FILL_CHARS,
        list: NO_LIST_CHARS,
    };

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
            install_defaults(&mut chars, tab);
            if listchars {
                chars.list.tab1 = NUL as ScreenChar;
                chars.list.tab3 = NUL as ScreenChar;
                chars.list.leadtab1 = NUL as ScreenChar;
                chars.list.leadtab3 = NUL as ScreenChar;
                chars.list.multispace = new_run(multispace_len);
                chars.list.leadmultispace = new_run(lead_multispace_len);
            }
        }

        let mut p = 0;
        while p < bytes.len() {
            let Some(i) = tab
                .iter()
                .position(|field| field_opens_at(bytes, p, field.name))
            else {
                return Err((e_invarg).into());
            };
            let field = &tab[i];
            let mut s = p + field.name.to_bytes().len() + 1;
            let width_err = |name| field_value_err(e_wrong_character_width_for_field_str, name);
            let count_err =
                |name| field_value_err(e_wrong_number_of_characters_for_field_str, name);

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
                            if take_encoded_char(bytes, &mut s) == 0 {
                                return Err(width_err(field.name));
                            }
                            *len += 1;
                        }
                        // The field cannot be empty.
                        if *len == 0 {
                            return Err(count_err(field.name));
                        }
                    } else {
                        // Only the last mention of the field fills the run;
                        // any earlier one is walked past and dropped.
                        let fills = *last == Some(p);
                        let run = if lead {
                            &mut chars.list.leadmultispace
                        } else {
                            &mut chars.list.multispace
                        };
                        let mut into = 0;
                        while !at_field_end(bytes, s) {
                            let c = take_encoded_char(bytes, &mut s);
                            if fills && let Some(run) = run.as_deref_mut() {
                                // The run was made for exactly the count the
                                // first round arrived at, and this is the
                                // same walk over the same field.
                                run[into] = c;
                                into += 1;
                            }
                        }
                    }
                    p = s;
                }
                _ => {
                    if at_end(bytes, s) {
                        return Err(count_err(field.name));
                    }
                    let c1 = take_encoded_char(bytes, &mut s);
                    if c1 == 0 {
                        return Err(width_err(field.name));
                    }
                    let mut c2: ScreenChar = 0;
                    let mut c3: ScreenChar = 0;
                    if matches!(field.shape, Shape::Tab | Shape::LeadTab) {
                        if at_end(bytes, s) {
                            return Err(count_err(field.name));
                        }
                        c2 = take_encoded_char(bytes, &mut s);
                        if c2 == 0 {
                            return Err(width_err(field.name));
                        }
                        // The third character is optional.
                        if !at_field_end(bytes, s) {
                            c3 = take_encoded_char(bytes, &mut s);
                            if c3 == 0 {
                                return Err(width_err(field.name));
                            }
                        }
                        if field.shape == Shape::Tab {
                            has_tab = true;
                        } else {
                            has_leadtab = true;
                        }
                    }
                    if !at_field_end(bytes, s) {
                        return Err(count_err(field.name));
                    }
                    if round > 0 {
                        match field.shape {
                            Shape::Tab => {
                                chars.list.tab1 = c1;
                                chars.list.tab2 = c2;
                                chars.list.tab3 = c3;
                            }
                            Shape::LeadTab => {
                                chars.list.leadtab1 = c1;
                                chars.list.leadtab2 = c2;
                                chars.list.leadtab3 = c3;
                            }
                            _ => {
                                if let Some(slot) = field.slot {
                                    chars.store(slot, c1);
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
            return Err((e_leadtab_requires_tab).into());
        }
    }

    Ok(apply.then_some(chars))
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
fn install_defaults(chars: &mut Chars, tab: &[Field]) {
    for field in tab {
        let Some(slot) = field.slot else {
            continue;
        };
        let narrow = field
            .def
            .is_some_and(|def| ptr2cells_at(def.to_bytes()) == 1);
        let text = if narrow { field.def } else { field.fallback };
        // Each default is one character, packed as the screen packs it.
        let c = text.map_or(0, |text| c_cluster_schar(text.to_bytes()).0);
        chars.store(slot, c);
    }
}

impl LcsChars {
    /// The "multispace:" and "leadmultispace:" runs, copied: `None` for a
    /// field that is not set. A copy, so a caller that prints with them
    /// holds no borrow of the window.
    pub(crate) fn multispace_runs(&self) -> (Option<Vec<ScreenChar>>, Option<Vec<ScreenChar>>) {
        (
            self.multispace.as_deref().map(<[_]>::to_vec),
            self.leadmultispace.as_deref().map(<[_]>::to_vec),
        )
    }
}

/// The run of characters a "multispace:" field fills, `len` of them. A
/// zero-length run is no run at all.
fn new_run(len: usize) -> Option<Box<[ScreenChar]>> {
    (len > 0).then(|| vec![0; len].into_boxed_slice())
}

/// Set the global 'fillchars' or 'listchars', and re-derive every window's
/// characters from it.
///
/// A `:set` without `setglobal` also clears the window's local value, so
/// that the window follows the global one again. Every other window that
/// has no local value of its own is refreshed too, because they were all
/// drawing from the value that just changed.
fn did_set_global_chars_option(
    mut win: Win,
    val: &[u8],
    what: CharsOption,
    opt_flags: OptionSetFlags,
) -> Result<(), OptError> {
    let listchars = is_listchars(what);
    let local_is_empty = |wp: Win| {
        if listchars {
            wp.w_onebuf_opt.wo_lcs.bytes().is_empty()
        } else {
            wp.w_onebuf_opt.wo_fcs.bytes().is_empty()
        }
    };
    let for_this_window = local_is_empty(win) || !opt_flags.has(OptionSetFlags::GLOBAL);

    set_chars_option(win, Some(val), what, for_this_window)?;

    if !opt_flags.has(OptionSetFlags::GLOBAL) {
        let local = if listchars {
            win.w_onebuf_opt.wo_lcs.take()
        } else {
            win.w_onebuf_opt.wo_fcs.take()
        };
        drop(local);
    }

    for_each_window(|wp| {
        if local_is_empty(wp) {
            let _ = set_chars_option(wp, None, what, true);
        }
        None
    });
    redraw_all_later(UPD_NOT_VALID);
    Ok(())
}

/// The option-table callback for both options and both scopes: which of the
/// four cases this is comes from the variable being set.
pub fn did_set_chars_option(args: &mut OptSet) -> Result<(), OptError> {
    let (win, varp, idx, flags) = (
        args.os_win,
        args.os_varp.string_var(),
        args.os_idx,
        args.os_flags,
    );
    // 'listchars' and 'fillchars' share this callback, so the row says which
    // option it is and the variable says which *scope*: the option's own
    // global one, or this window's copy.
    let which = if idx == kOptListcharsIdx {
        kListchars
    } else {
        kFillchars
    };
    // Which variable it is, not what it says.
    if varp == option_var(idx).string_var() {
        did_set_global_chars_option(win, new_bytes(args), which, flags)
    } else if varp == StrVar::of_window(win, field!(WinOpt, wo_lcs))
        || varp == StrVar::of_window(win, field!(WinOpt, wo_fcs))
    {
        set_chars_option(win, Some(new_bytes(args)), which, true)
    } else {
        Ok(())
    }
}

/// Enumerate the field names of 'fillchars', for completion.
pub fn get_fillchars_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    field_name(&FCS_TAB, idx)
}

/// Enumerate the field names of 'listchars', for completion.
pub fn get_listchars_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    field_name(&LCS_TAB, idx)
}

/// The `idx`th field name, or null once the table has run out — which is
/// how `expand_generic` learns the list has ended.
fn field_name(tab: &'static [Field], idx: usize) -> Option<Candidate> {
    tab.get(idx).map(|field| Candidate::Borrowed(field.name))
}

/// Would the current 'fillchars' and 'listchars' still be accepted?
///
/// Called after something other than `:set` changed what the screen can
/// render — a new 'encoding', say — and reports which of the two options
/// the new state conflicts with.
pub fn check_chars_options() -> Result<(), OptError> {
    let check = |wp, value: Option<&[u8]>, what, apply| {
        if set_chars_option(wp, value, what, apply).is_ok() {
            None
        } else if is_listchars(what) {
            Some(e_conflicts_with_value_of_listchars)
        } else {
            Some(e_conflicts_with_value_of_fillchars)
        }
    };

    // Copies: the check reads the global value again for a window that has
    // none of its own.
    if let Some(global) = check(Win::current(), Some(&P_LCS.get()[..]), kListchars, false) {
        return Err((global).into());
    }
    if let Some(global) = check(Win::current(), Some(&P_FCS.get()[..]), kFillchars, false) {
        return Err((global).into());
    }
    let per_window = for_each_window(|wp| {
        check(wp, None, kListchars, true).or_else(|| check(wp, None, kFillchars, true))
    });
    match per_window {
        Some(errmsg) => Err(errmsg.into()),
        None => Ok(()),
    }
}

/// Walk every window of every tab page, stopping at the first message a
/// visit returns.
///
/// `FOR_ALL_TAB_WINDOWS`, i.e. [`winlayer::tab_windows`] -- which already
/// knows that the current tab page's windows hang off `firstwin` rather than
/// off its own stale list.
fn for_each_window(mut visit: impl FnMut(Win) -> Option<&'static CStr>) -> Option<&'static CStr> {
    for wp in winlayer::tab_windows() {
        if let Some(errmsg) = visit(wp) {
            return Some(errmsg);
        }
    }
    None
}
