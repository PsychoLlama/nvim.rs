//! Borders: the `border` key and the `'winborder'` option.
//!
//! `parse_border_style` accepts every spelling a border can take -- a named
//! style, a single character, an array of one, two, four or eight characters,
//! each optionally paired with a highlight group -- and fills the eight
//! `WinConfig` border slots from it.  `parse_winborder` is the option's own
//! parse, which shares the named styles and rejects the rest.
//!
//! The eight slots run clockwise from the top-left corner:
//! `[NW, N, NE, E, SE, S, SW, W]`.  That order is why an array of one, two or
//! four entries can be doubled into eight, and why the "corner char between
//! edge chars" check compares the odd slots against the even ones.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::api::private::helpers::array_add;

/// "Invalid 'border': expected `want`", the shape every rejection here takes.
/// `got` names what arrived when it is not null.
///
/// The name and the four-argument call are spelled once so that each of the
/// six call sites is one line rather than six.
///
/// # Safety
/// `err` must be the caller's error slot and `got` null or a C string.
unsafe fn err_border(err: *mut Error, want: &CStr, got: *const c_char) {
    let name = c"border".as_ptr();
    // SAFETY: the caller's promise; `name` and `want` are C strings.
    unsafe { api_err_exp(err, name, want.as_ptr(), got) };
}

/// One border cell: up to `MAX_SCHAR_SIZE` bytes of UTF-8, NUL-terminated.
/// The same shape as one slot of `WinConfig::border_chars`.
type BorderChar = [c_char; MAX_SCHAR_SIZE as usize];

/// An empty border cell, which the drawing code reads as "nothing here".
const BLANK_CHAR: BorderChar = [0; MAX_SCHAR_SIZE as usize];

/// The `'winborder'` style that draws a drop shadow instead of a box: the
/// cells to the right of and below the window are darkened, and nothing is
/// drawn on the top or left edges.
pub(crate) const BORDER_SHADOW: &CStr = c"shadow";

/// The `'winborder'` style that draws nothing at all.
pub(crate) const BORDER_NONE: &CStr = c"none";

/// A border cell holding one character.
///
/// Written out rather than transmuted: `c_char` is signed here, so the two
/// arrays are not the same type, and c2rust spelled every one of these as a
/// `transmute` from a 32-byte string literal.
const fn border_char(text: &str) -> BorderChar {
    let bytes = text.as_bytes();
    assert!(
        bytes.len() < MAX_SCHAR_SIZE as usize,
        "a border cell must leave room for its NUL"
    );
    let mut out = BLANK_CHAR;
    let mut i = 0;
    while i < bytes.len() {
        out[i] = bytes[i].cast_signed();
        i += 1;
    }
    out
}

/// A border style `border = "name"` and `'winborder'` both accept.
struct BorderStyle {
    /// The style's name, which is also its `'winborder'` spelling.
    name: &'static CStr,
    /// The eight cells, clockwise from the top-left corner.
    chars: [BorderChar; 8],
    /// Whether this is the drop shadow, whose slots take the
    /// `FloatShadow`/`FloatShadowThrough` groups rather than none.
    shadow: bool,
}

/// A box-drawing style, from its eight characters.
const fn boxed(chars: [&str; 8]) -> [BorderChar; 8] {
    [
        border_char(chars[0]),
        border_char(chars[1]),
        border_char(chars[2]),
        border_char(chars[3]),
        border_char(chars[4]),
        border_char(chars[5]),
        border_char(chars[6]),
        border_char(chars[7]),
    ]
}

/// Every named border style, in the order `parse_border_style` searches
/// them -- which is `'winborder'`'s own value order minus its empty first
/// entry and the `"none"` the string arm handles before it gets here.
static STYLES: [BorderStyle; 6] = [
    BorderStyle {
        name: c"double",
        chars: boxed(["╔", "═", "╗", "║", "╝", "═", "╚", "║"]),
        shadow: false,
    },
    BorderStyle {
        name: c"single",
        chars: boxed(["┌", "─", "┐", "│", "┘", "─", "└", "│"]),
        shadow: false,
    },
    BorderStyle {
        // Nothing above or left of the window; the other six cells are
        // blanks that the shadow highlights darken.
        name: BORDER_SHADOW,
        chars: [
            BLANK_CHAR,
            BLANK_CHAR,
            border_char(" "),
            border_char(" "),
            border_char(" "),
            border_char(" "),
            border_char(" "),
            BLANK_CHAR,
        ],
        shadow: true,
    },
    BorderStyle {
        name: c"rounded",
        chars: boxed(["╭", "─", "╮", "│", "╯", "─", "╰", "│"]),
        shadow: false,
    },
    BorderStyle {
        name: c"solid",
        chars: boxed([" ", " ", " ", " ", " ", " ", " ", " "]),
        shadow: false,
    },
    BorderStyle {
        name: c"bold",
        chars: boxed(["┏", "━", "┓", "┃", "┛", "━", "┗", "┃"]),
        shadow: false,
    },
];

/// The style `name` spells, if it spells one.
///
/// # Safety
/// `name` must be a NUL-terminated string.
unsafe fn find_style(name: *const c_char) -> Option<&'static BorderStyle> {
    // SAFETY: the caller's name.
    let name = unsafe { CStr::from_ptr(name) };
    STYLES.iter().find(|style| style.name == name)
}

/// Whether the *corner* slot between two set edge slots is empty, which the
/// drawing code cannot render.
fn corner_gap(chars: &[BorderChar; 8], before: usize, after: usize, corner: usize) -> bool {
    chars[before][0] != 0 && chars[after][0] != 0 && chars[corner][0] == 0
}

/// The eight cells and the eight highlight ids a named style asks for.
///
/// # Safety
/// The highlight tables must be initialised.
unsafe fn style_slots(style: &BorderStyle) -> Slots {
    let mut hl_ids = [0; 8];
    if style.shadow {
        let (shadow, deep) = (c"FloatShadow", c"FloatShadowThrough");
        // SAFETY: two static group names, and the editor's highlight tables.
        let blend = unsafe { syn_check_group(shadow.as_ptr(), shadow.count_bytes()) };
        // SAFETY: as above.
        let through = unsafe { syn_check_group(deep.as_ptr(), deep.count_bytes()) };
        // The two cells the window's own corner shows through take the
        // "through" group; the four entirely outside it take "blend".
        hl_ids = [0, 0, through, blend, blend, blend, through, 0];
    }
    (style.chars, hl_ids)
}

/// One entry of a `border = { ... }` array: a cell and its highlight group.
///
/// # Safety
/// `item` must be a live API object and `err` a writable error slot.
unsafe fn parse_border_item(item: Object, err: *mut Error) -> Option<(String_0, c_int)> {
    if item.type_0 == kObjectTypeArray {
        // SAFETY: the tag says the array arm is live.
        let arr = unsafe { item.data.array };
        if arr.size == 0 || arr.size > 2 {
            // SAFETY: the caller's error slot.
            unsafe { err_border(err, c"1 or 2-item Array", NULL_STR) };
            return None;
        }
        // SAFETY: a non-empty array has an item at index 0.
        let first = unsafe { *arr.items };
        if first.type_0 != kObjectTypeString {
            // SAFETY: the caller's error slot.
            unsafe { err_border(err, c"Array of Strings", NULL_STR) };
            return None;
        }
        // SAFETY: the tag says the string arm is live.
        let string = unsafe { first.data.string };
        if arr.size < 2 {
            return Some((string, 0));
        }
        // SAFETY: a two-item array has an item at index 1, and the caller's
        // error slot.
        let hl_id =
            unsafe { object_to_hl_id(*arr.items.add(1), c"border char highlight".as_ptr(), err) };
        // SAFETY: as above.
        if unsafe { (*err).type_0 } != kErrorTypeNone {
            return None;
        }
        return Some((string, hl_id));
    }
    if item.type_0 == kObjectTypeString {
        // SAFETY: the tag says the string arm is live.
        return Some((unsafe { item.data.string }, 0));
    }
    // SAFETY: the caller's error slot.
    unsafe { err_border(err, c"String or Array", api_typename(item.type_0)) };
    None
}

/// A `String_0` as one border cell, truncated to what a slot holds.
///
/// # Safety
/// `string` must be a live API string.
unsafe fn cell_of(string: String_0) -> BorderChar {
    let mut out = BLANK_CHAR;
    let len = string.len().min(MAX_SCHAR_SIZE as usize - 1);
    if len != 0 {
        // SAFETY: a live API string of at least `len` bytes, and `out` has
        // room for `MAX_SCHAR_SIZE - 1` of them plus the NUL already there.
        unsafe { ::core::ptr::copy_nonoverlapping(string.data(), out.as_mut_ptr(), len) };
    }
    out
}

/// The `NULL` a validation message passes for "no third argument".
const NULL_STR: *const c_char = ::core::ptr::null();

/// The eight border cells and the eight highlight ids that go with them --
/// `WinConfig`'s `border_chars` and `border_hl_ids`, filled together.
type Slots = ([BorderChar; 8], [c_int; 8]);

/// The eight border slots an array of one, two, four or eight entries --
/// each a character or a `{ char, hl }` pair -- asks for.
///
/// The "corner char between edge chars" complaint is the one diagnosis that
/// still answers `Some`: upstream fills the slots and *then* raises it, and
/// its callers discard the whole config on any error anyway.
///
/// # Safety
/// `arr` must be a live API array and `err` a writable error slot.
unsafe fn parse_border_array(arr: Array, err: *mut Error) -> Option<Slots> {
    let size = arr.size;
    if size == 0 || size > 8 || !size.is_power_of_two() {
        // SAFETY: the caller's error slot.
        unsafe { err_border(err, c"1, 2, 4, or 8 chars", NULL_STR) };
        return None;
    }

    let mut chars = [BLANK_CHAR; 8];
    let mut hl_ids = [0 as c_int; 8];
    for i in 0..size {
        // SAFETY: `i` is below the array's own size.
        let item = unsafe { *arr.items.add(i) };
        // SAFETY: an item of a live array, and the caller's error slot.
        let (string, hl_id) = unsafe { parse_border_item(item, err) }?;
        // SAFETY: a live API string.
        if !string.is_empty() && unsafe { mb_string2cells_len(string.data(), string.len()) } > 1 {
            // SAFETY: the caller's error slot.
            unsafe { err_border(err, c"only one-cell chars", NULL_STR) };
            return None;
        }
        // SAFETY: as above.
        chars[i] = unsafe { cell_of(string) };
        hl_ids[i] = hl_id;
    }

    // One, two or four entries repeat around the frame until all eight are
    // filled: [a] -> [a a], [a b] -> [a b a b], and so on.
    let mut filled = size;
    while filled < 8 {
        chars.copy_within(..filled, filled);
        hl_ids.copy_within(..filled, filled);
        filled <<= 1;
    }

    // An edge on both sides of an empty corner leaves a hole the drawing
    // code cannot render.
    if corner_gap(&chars, 7, 1, 0)
        || corner_gap(&chars, 1, 3, 2)
        || corner_gap(&chars, 3, 5, 4)
        || corner_gap(&chars, 5, 7, 6)
    {
        // SAFETY: the caller's error slot.
        unsafe { err_border(err, c"corner char between edge chars", NULL_STR) };
    }
    Some((chars, hl_ids))
}

/// Parse the `border` key of `nvim_open_win`/`nvim_win_set_config` into
/// `fconfig`'s eight border slots.
///
/// # Safety
/// `fconfig` must be writable, `style` a live API object and `err` a
/// writable error slot.
pub unsafe fn parse_border_style(style: Object, fconfig: *mut WinConfig, err: *mut Error) {
    // The config is written a field at a time rather than through one
    // long-lived `&mut`: everything below can re-enter the editor, which owns
    // it. That is what `WinCfg` is -- a `Live<WinConfig>` reborrowing per
    // access.
    // SAFETY: the caller's config, live for the whole call.
    let mut cfg = unsafe { WinCfg::new(fconfig) };
    cfg.border = true;

    let slots = if style.type_0 == kObjectTypeArray {
        // SAFETY: the tag says the array arm is live; the caller's error slot.
        unsafe { parse_border_array(style.data.array, err) }
    } else if style.type_0 == kObjectTypeString {
        // SAFETY: the tag says the string arm is live.
        let str = unsafe { style.data.string };
        // SAFETY: a live API string is NUL-terminated.
        if str.is_empty() || unsafe { strequal(str.data(), BORDER_NONE.as_ptr()) } {
            // Border text does not work without a border.
            cfg.border = false;
            cfg.title = false;
            cfg.footer = false;
            return;
        }
        // SAFETY: as above.
        match unsafe { find_style(str.data()) } {
            // SAFETY: the editor's highlight tables are initialised by the
            // time any window can be configured.
            Some(style) => Some(unsafe { style_slots(style) }),
            None => {
                // SAFETY: the caller's error slot and a live API string.
                unsafe { api_err_invalid(err, c"border".as_ptr(), str.data(), 0, true) };
                None
            }
        }
    } else {
        // Neither an Array nor a String names a border; upstream leaves the
        // slots alone and does not diagnose it either.
        None
    };

    if let Some((chars, hl_ids)) = slots {
        cfg.border_chars = chars;
        cfg.border_hl_ids = hl_ids;
    }
}

/// # Safety
/// `wp` must be null or a live window, `attribute` NUL-terminated and `err` a
/// writable error slot.
pub(crate) unsafe fn generate_api_error(
    wp: *mut win_T,
    attribute: *const ::core::ffi::c_char,
    err: *mut Error,
) {
    // SAFETY: the caller's window.
    if !wp.is_null() && unsafe { (*wp).w_floating } {
        // SAFETY: the caller's error slot, and a format the message takes.
        let fmt = c"Required: 'relative' when reconfiguring floating window %d".as_ptr();
        // SAFETY: the caller's error slot and window; the one `%d` spends the
        // window handle.
        unsafe { api_set_error(err, kErrorTypeValidation, fmt, (*wp).handle) };
    } else {
        // SAFETY: the caller's error slot and attribute name.
        unsafe { api_err_conflict(err, attribute, c"non-float window".as_ptr()) };
    }
}

/// The `'winborder'` value as a `border` key, or `false` when the option's
/// value is not one this accepts.
///
/// A value holding a comma is eight cells spelled out; anything else is a
/// style name, which [`parse_border_style`] resolves.
///
/// # Safety
/// `fconfig` must be null or writable, `border_opt` NUL-terminated and `err`
/// a writable error slot.
pub unsafe fn parse_winborder(
    fconfig: *mut WinConfig,
    border_opt: *mut ::core::ffi::c_char,
    err: *mut Error,
) -> bool {
    if fconfig.is_null() {
        return false;
    }
    // SAFETY: the caller's option value.
    let listed = !unsafe { strchr(border_opt, ',' as c_int) }.is_null();
    let style = if listed {
        // SAFETY: as above.
        match unsafe { border_cell_list(border_opt) } {
            Some(array) => Object::array(array),
            None => return false,
        }
    } else {
        // SAFETY: as above.
        Object::string(unsafe { cstr_to_string(border_opt) })
    };
    // SAFETY: the caller's config and error slot, and the object just built.
    let slot = unsafe {
        parse_border_style(style, fconfig, err);
        api_free_object(style);
        ErrSlot::new(err)
    };
    slot.type_0 == kErrorTypeNone
}

/// The eight comma-separated cells of a `'winborder'` value, or `None` when
/// it does not hold exactly eight non-empty ones.
///
/// # Safety
/// `border_opt` must be a NUL-terminated string.
unsafe fn border_cell_list(border_opt: *mut c_char) -> Option<Array> {
    // Room for the eight it must have: nine parts is already a failure, so
    // the transpile's doubling growth step never got past this size either.
    let mut cells = arena_array(::core::ptr::null_mut(), 8);
    let mut p = border_opt;
    let mut part: BorderChar = BLANK_CHAR;
    // SAFETY: the caller's option value, NUL-terminated; `copy_option_part`
    // advances `p` and writes at most `part.len()` bytes into `part`.
    while unsafe { *p } != 0 {
        let full = cells.size == cells.capacity;
        let (next, into, room) = (&raw mut p, part.as_mut_ptr(), part.len());
        let comma = c",".as_ptr().cast_mut();
        // The copy is still short-circuited by `full`: a ninth part is
        // already a failure and must not advance `p`.
        //
        // SAFETY: as above.
        let empty =
            full || unsafe { copy_option_part(next, into, room, comma) } == 0 || part[0] == 0;
        if empty {
            // SAFETY: the array holds only strings this loop allocated.
            unsafe { api_free_array(cells) };
            return None;
        }
        // SAFETY: `part` is NUL-terminated by `copy_option_part`, and
        // `cells` is this function's own array.
        unsafe {
            let cell = Object::string(cstr_to_string(part.as_mut_ptr()));
            array_add(&mut cells, cell);
        };
    }
    if cells.size != cells.capacity {
        // SAFETY: as above.
        unsafe { api_free_array(cells) };
        return None;
    }
    Some(cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `border_char` writes the UTF-8 bytes and leaves the rest NUL --
    /// the fact the 48 transmutes used to state by writing the padding out.
    #[test]
    fn a_border_cell_is_its_bytes_then_nuls() {
        let cell = border_char("╔");
        assert_eq!(
            &cell[..3],
            &[
                0xE2u8.cast_signed(),
                0x95u8.cast_signed(),
                0x94u8.cast_signed()
            ]
        );
        assert!(cell[3..].iter().all(|&b| b == 0));
    }

    #[test]
    fn every_style_has_a_distinct_name_and_the_option_lists_it() {
        for style in &STYLES {
            assert_eq!(
                1,
                STYLES
                    .iter()
                    .filter(|other| other.name == style.name)
                    .count()
            );
            assert!(
                crate::options::opt_winborder_values.contains(&style.name),
                "'winborder' does not accept {style:?}",
                style = style.name
            );
        }
        // 'winborder' also takes "" and "none", which have no style row.
        assert_eq!(STYLES.len() + 2, crate::options::opt_winborder_values.len());
    }

    /// Exactly one style is the shadow, and it leaves the top-left corner,
    /// the top edge and the left edge empty.
    #[test]
    fn the_shadow_is_the_only_style_that_is_not_a_box() {
        let shadows: Vec<_> = STYLES.iter().filter(|s| s.shadow).collect();
        assert_eq!(1, shadows.len());
        assert_eq!(BORDER_SHADOW, shadows[0].name);
        for slot in [0, 1, 7] {
            assert_eq!(0, shadows[0].chars[slot][0]);
        }
        for slot in [2, 3, 4, 5, 6] {
            assert_eq!(b' '.cast_signed(), shadows[0].chars[slot][0]);
        }
    }

    /// A box style fills all eight slots, so no corner check can fire on it.
    #[test]
    fn no_box_style_leaves_a_corner_gap() {
        for style in STYLES.iter().filter(|s| !s.shadow) {
            for (before, after, corner) in [(7, 1, 0), (1, 3, 2), (3, 5, 4), (5, 7, 6)] {
                assert!(!corner_gap(&style.chars, before, after, corner));
            }
        }
    }

    /// The shadow's own empty corners are not a gap either: its top and left
    /// edges are empty too, so nothing brackets them.
    #[test]
    fn the_shadow_is_not_a_corner_gap() {
        let shadow = STYLES.iter().find(|s| s.shadow).expect("a shadow style");
        for (before, after, corner) in [(7, 1, 0), (1, 3, 2), (3, 5, 4), (5, 7, 6)] {
            assert!(!corner_gap(&shadow.chars, before, after, corner));
        }
    }
}
