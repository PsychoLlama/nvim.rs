//! Display width: how many screen cells a character occupies.
//!
//! Almost everything is one cell. The exceptions are the East Asian wide and
//! fullwidth characters (two), the combining marks (zero — they render on top
//! of the character before), the *ambiguous-width* characters, which are one
//! or two depending on `'ambiwidth'`, and anything unprintable, which is
//! shown as a `<xx>` or `<xxxx>` escape and so takes four or six.
//!
//! Emoji are the awkward case. Unicode calls most of them ambiguous, but a
//! terminal draws them two cells wide, so `'emoji'` widens everything from
//! `U+1F000` up — and a character followed by VS-16 (`U+FE0F`) asks for
//! emoji presentation and widens too, which is why the `ptr` spellings look
//! at the *next* character and `utf_char2cells` cannot.
//!
//! `setcellwidths()` overrides the answer for chosen ranges, for terminals
//! and fonts that disagree with Unicode.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use core::cmp::Ordering;
use core::ffi::{c_char, c_int, c_uint, c_void};

/// Variation Selector 16: "draw the character before me as an emoji".
const VS16: c_int = 0xfe0f;

/// Below this codepoint, `'emoji'` leaves widths alone: the older symbol
/// blocks have traditionally been drawn one cell wide and widening them
/// causes more trouble than it fixes.
const FIRST_EMOJI_BLOCK: c_int = 0x1f000;

/// One `setcellwidths()` override: every codepoint in `first..=last` is
/// `width` cells wide.
#[derive(Copy, Clone)]
struct CellWidthRange {
    first: varnumber_T,
    last: varnumber_T,
    width: c_int,
}

/// The `setcellwidths()` overrides, sorted by `first` and non-overlapping.
///
/// [`cw_value`] binary-searches this, and [`f_setcellwidths`] is where both
/// properties are established: it sorts the user's list and rejects it with
/// `E1113` if two ranges meet.
static CELL_WIDTHS: GlobalCell<Vec<CellWidthRange>> = GlobalCell::new(Vec::new());

/// The width `setcellwidths()` was told to give `c`, or 0 for "not overridden".
fn cw_value(c: c_int) -> c_int {
    let c = c as varnumber_T;
    CELL_WIDTHS.with(|table| {
        table
            .binary_search_by(|r| {
                if r.last < c {
                    Ordering::Less
                } else if r.first > c {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
            .map_or(0, |i| table[i].width)
    })
}

/// How many cells the codepoint `c` occupies on its own.
///
/// "On its own" is the limit of this answer: a character whose width depends
/// on what follows it — an emoji base plus VS-16 — needs [`utf_ptr2cells`].
///
/// # Safety
///
/// Reads `'ambiwidth'`, `'emoji'` and `'isprint'` through their globals, so
/// it is only callable once options exist.
pub unsafe fn utf_char2cells(c: c_int) -> c_int {
    if c < 0x80 {
        return 1;
    }

    // SAFETY: the caller's obligation, forwarded.
    if !unsafe { vim_isprintc(c) } {
        debug_assert!(c <= 0xffff, "c <= 0xFFFF");
        // Shown as <xx> or <xxxx>.
        return if c > 0xff { 6 } else { 4 };
    }

    let overridden = cw_value(c);
    if overridden != 0 {
        return overridden;
    }

    let prop = utf8proc_get_property(c);
    if prop.charwidth as c_int == 2 {
        return 2;
    }
    // SAFETY: `p_ambw` is 'ambiwidth', a NUL-terminated option string.
    if unsafe { *p_ambw.get() } as c_int == 'd' as c_int && prop.ambiguous_width {
        return 2;
    }
    if p_emoji.get() != 0
        && c >= FIRST_EMOJI_BLOCK
        && !prop.ambiguous_width
        && prop_is_emojilike(prop)
    {
        return 2;
    }
    1
}

/// Whether the character at `p` asks for emoji presentation by being followed
/// by VS-16 — the one width question that needs to look past the character.
///
/// # Safety
///
/// `next` must point at the byte after the character, inside the same string.
unsafe fn widened_by_vs16(cells: c_int, c: c_int, next: *const c_char) -> bool {
    cells == 1
        && p_emoji.get() != 0
        && prop_is_emojilike(utf8proc_get_property(c))
        // SAFETY: the caller's obligation.
        && unsafe { utf_ptr2char(next) } == VS16
}

/// How many cells the character at `p` occupies.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn utf_ptr2cells(p_in: *const c_char) -> c_int {
    unsafe {
        let p = p_in as *const u8;
        if *p < 0x80 {
            return 1;
        }
        let len = utf8len_tab[*p as usize] as c_int;
        let c = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        // An illegal byte is displayed as <xx>.
        if c <= 0 {
            return 4;
        }
        // An ASCII answer from a multibyte lead byte means an overlong
        // sequence, which is displayed the way that ASCII character is.
        if c < 0x80 {
            return char2cells(c);
        }
        let cells = utf_char2cells(c);
        if widened_by_vs16(cells, c, p_in.offset(len as isize)) {
            return 2;
        }
        cells
    }
}

/// [`utf_ptr2cells`] over a string that is `size` bytes long rather than
/// NUL-terminated.
///
/// A sequence cut short by the end of the buffer is *not* an illegal byte: it
/// answers 1, because the rest of it may still arrive.
///
/// # Safety
///
/// `p` must point at `size` readable bytes.
pub unsafe fn utf_ptr2cells_len(p: *const c_char, size: c_int) -> c_int {
    unsafe {
        if size <= 0 || (*p as u8) < 0x80 {
            return 1;
        }
        let len = utf_ptr2len_len(p, size);
        if len < utf8len_tab[*p as u8 as usize] as c_int {
            return 1; // truncated
        }
        let c = utf_ptr2char(p);
        // An illegal byte is displayed as <xx>.
        if utf_ptr2len(p) == 1 || c == NUL {
            return 4;
        }
        if c < 0x80 {
            return char2cells(c);
        }
        let cells = utf_char2cells(c);
        // The VS-16 has to be *complete* within `size`; a truncated one does
        // not widen anything.
        let next = p.offset(len as isize);
        if size > len
            && utf_ptr2len_len(next, size - len) == utf8len_tab[*next as u8 as usize] as c_int
            && widened_by_vs16(cells, c, next)
        {
            return 2;
        }
        cells
    }
}

/// The total width of a NUL-terminated string.
///
/// # Safety
///
/// `str` must point at a NUL-terminated string.
pub unsafe fn mb_string2cells(str: *const c_char) -> size_t {
    unsafe {
        let mut cells: size_t = 0;
        let mut p = str;
        while *p != NUL as c_char {
            cells += utf_ptr2cells(p) as size_t;
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        cells
    }
}

/// The total width of at most `size` bytes, stopping early at a NUL.
///
/// # Safety
///
/// `str` must point at `size` readable bytes.
pub unsafe fn mb_string2cells_len(str: *const c_char, size: size_t) -> size_t {
    unsafe {
        let mut cells: size_t = 0;
        let mut p = str;
        while *p != NUL as c_char && p < str.add(size) {
            let left = size as c_int - p.offset_from(str) as c_int;
            cells += utf_ptr2cells_len(p, left) as size_t;
            p = p.offset(utfc_ptr2len_len(p, left) as isize);
        }
        cells
    }
}

/// Might the character at `p` be drawn at a width the grid did not predict?
///
/// The screen asks this to decide whether a cell needs re-measuring: an
/// ambiguous-width character answers whatever `'ambiwidth'` currently says,
/// an emoji whatever `'emoji'` says, and a VS-16 after any character turns it
/// into an emoji.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn utf_ambiguous_width(p: *const c_char) -> bool {
    unsafe {
        // Nothing to print, or a lone ASCII character: neither can move.
        if *p == NUL as c_char || *p.offset(1) == NUL as c_char {
            return false;
        }
        let info = utf_ptr2CharInfo(p);
        if info.value >= 0x80 {
            let prop = utf8proc_get_property(info.value);
            if prop.ambiguous_width || prop_is_emojilike(prop) {
                return true;
            }
        }
        // Safe against a NUL: `memcmp` stops at the first difference, and the
        // NUL differs from VS-16's first byte.
        memcmp(
            p.offset(info.len as isize) as *const c_void,
            c"\xef\xb8\x8f".as_ptr() as *const c_void,
            3,
        ) == 0
    }
}

/// `setcellwidths({list})` — override the width of chosen codepoint ranges.
///
/// Each entry is `[first, last, width]`; the whole list is validated before
/// anything is installed, so a rejected call leaves the old table in place.
/// The install is provisional even then: `'listchars'` and `'fillchars'` must
/// still agree with the new widths, and the old table comes back if they do
/// not.
pub unsafe fn f_setcellwidths(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if (*argvars).v_type as c_uint != VAR_LIST as c_uint || (*argvars).vval.v_list.is_null() {
            emsg(gettext(&raw const e_listreq as *const c_char));
            return;
        }
        let Some(table) = parse_cell_widths((*argvars).vval.v_list) else {
            return;
        };

        let saved = CELL_WIDTHS.with_mut(|t| core::mem::replace(t, table));

        // The new widths must not conflict with 'listchars' or 'fillchars'.
        let error = check_chars_options();
        if !error.is_null() {
            emsg(gettext(error));
            CELL_WIDTHS.with_mut(|t| *t = saved);
            return;
        }

        changed_window_setting_all();
        redraw_all_later(UPD_NOT_VALID);
    }
}

/// Validate `setcellwidths()`'s argument into the sorted, disjoint table
/// [`cw_value`] needs, or report why it cannot be one and answer `None`.
///
/// An empty list is a valid empty table — that is how the overrides are
/// cleared.
///
/// # Safety
///
/// `l` must be a live list.
unsafe fn parse_cell_widths(l: *const list_T) -> Option<Vec<CellWidthRange>> {
    unsafe {
        let mut rows: Vec<CellWidthRange> = Vec::with_capacity(tv_list_len(l) as usize);
        let mut li = (*l).lv_first;
        let mut item: c_int = 0;
        while !li.is_null() {
            let li_tv = &raw const (*li).li_tv;
            if (*li_tv).v_type as c_uint != VAR_LIST as c_uint || (*li_tv).vval.v_list.is_null() {
                semsg_c!(gettext(c"E1109: List item %d is not a List".as_ptr()), item);
                return None;
            }
            rows.push(parse_cell_width_row((*li_tv).vval.v_list, item)?);
            li = (*li).li_next;
            item += 1;
        }

        // Upstream sorts with qsort, which is unstable; two rows sharing a
        // `first` overlap either way round and are rejected below with the
        // same number, so a stable sort answers identically.
        rows.sort_by_key(|r| r.first);
        for i in 1..rows.len() {
            if rows[i].first <= rows[i - 1].last {
                semsg_c!(
                    gettext(c"E1113: Overlapping ranges for 0x%lx".as_ptr()),
                    rows[i].first as size_t,
                );
                return None;
            }
        }
        Some(rows)
    }
}

/// One `[first, last, width]` entry, or `None` having reported the fault.
///
/// `item` is the entry's index in the outer list, which is what the messages
/// name.
///
/// # Safety
///
/// `li_l` must be a live list.
unsafe fn parse_cell_width_row(li_l: *const list_T, item: c_int) -> Option<CellWidthRange> {
    unsafe {
        let mut numbers = [0 as varnumber_T; 3];
        let mut seen = 0;
        let mut lili = tv_list_first(li_l);
        while !lili.is_null() {
            let tv = &raw const (*lili).li_tv;
            if (*tv).v_type as c_uint != VAR_NUMBER as c_uint {
                break;
            }
            let n = (*tv).vval.v_number;
            match seen {
                0 if n < 0x80 => {
                    emsg(gettext(
                        c"E1114: Only values of 0x80 and higher supported".as_ptr(),
                    ));
                    return None;
                }
                1 if n < numbers[0] => {
                    semsg_c!(gettext(c"E1111: List item %d range invalid".as_ptr()), item);
                    return None;
                }
                2 if !(1..=2).contains(&n) => {
                    semsg_c!(
                        gettext(c"E1112: List item %d cell width invalid".as_ptr()),
                        item,
                    );
                    return None;
                }
                _ => {}
            }
            if seen < numbers.len() {
                numbers[seen] = n;
            }
            seen += 1;
            lili = (*lili).li_next;
        }

        // A fourth number, a non-number, or too few: all "not three numbers".
        if seen != 3 {
            semsg_c!(
                gettext(c"E1110: List item %d does not contain 3 numbers".as_ptr()),
                item,
            );
            return None;
        }
        Some(CellWidthRange {
            first: numbers[0],
            last: numbers[1],
            width: numbers[2] as c_int,
        })
    }
}

/// `getcellwidths()` — the table `setcellwidths()` installed, as a List of
/// `[first, last, width]`.
pub unsafe fn f_getcellwidths(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let rows = CELL_WIDTHS.with(|t| t.clone());
        tv_list_alloc_ret(rettv, rows.len() as ptrdiff_t);
        for row in &rows {
            let entry = tv_list_alloc(3);
            tv_list_append_number(entry, row.first);
            tv_list_append_number(entry, row.last);
            tv_list_append_number(entry, row.width as varnumber_T);
            tv_list_append_list((*rettv).vval.v_list, entry);
        }
    }
}
