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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::{index_of, list_iter};
use crate::message_fmt::emsg_text;
use crate::option::vars::P_AMBW;
use crate::tr;
use crate::types::VAR_NUMBER;
use core::cmp::Ordering;
use core::ffi::{c_int, c_uint};

/// Below this codepoint, `'emoji'` leaves widths alone: the older symbol
/// blocks have traditionally been drawn one cell wide and widening them
/// causes more trouble than it fixes.
const FIRST_EMOJI_BLOCK: c_int = 0x1f000;

/// One `setcellwidths()` override: every codepoint in `first..=last` is
/// `width` cells wide.
#[derive(Copy, Clone, Debug)]
struct CellWidthRange {
    first: VarNumber,
    last: VarNumber,
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
    let c = VarNumber::from(c);
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
/// on what follows it — an emoji base plus VS-16 — needs [`cells_at`].
pub fn utf_char2cells(c: Codepoint) -> c_int {
    if c < 0x80 {
        return 1;
    }

    if !vim_isprintc(c) {
        debug_assert!(c.get() <= 0xffff, "c <= 0xFFFF");
        // Shown as <xx> or <xxxx>.
        return if c > 0xff { 6 } else { 4 };
    }

    let overridden = cw_value(c.get());
    if overridden != 0 {
        return overridden;
    }

    let prop = utf8proc_get_property(c.get());
    if prop.charwidth == 2 {
        return 2;
    }
    if P_AMBW.first_byte() == b'd' && prop.ambiguous_width {
        return 2;
    }
    if p_emoji() && c.get() >= FIRST_EMOJI_BLOCK && !prop.ambiguous_width && prop_is_emojilike(prop)
    {
        return 2;
    }
    1
}

/// Whether `c`, one cell wide on its own, is an emoji-like character that a
/// VS-16 after it widens to two cells under 'emoji' -- the half of that
/// question that does not need the next character.
///
/// Out of the readers' bodies on purpose: they are `#[inline]`, and an
/// option read inlined into them would export the option record, whose every
/// read in the editor then goes through the GOT.
#[inline(never)]
pub(crate) fn emoji_widens(c: Codepoint) -> bool {
    p_emoji() && prop_is_emojilike(utf8proc_get_property(c.get()))
}

/// `setcellwidths({list})` — override the width of chosen codepoint ranges.
///
/// Each entry is `[first, last, width]`; the whole list is validated before
/// anything is installed, so a rejected call leaves the old table in place.
/// The install is provisional even then: `'listchars'` and `'fillchars'` must
/// still agree with the new widths, and the old table comes back if they do
/// not.
pub fn f_setcellwidths(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let Some(list) = args[0].list_ref() else {
        emsg(gettext(E_LISTREQ));
        return;
    };
    let table = match parse_cell_widths(list) {
        Ok(table) => table,
        Err(error) => {
            emsg_text(error.to_string());
            return;
        }
    };

    let saved = CELL_WIDTHS.with_mut(|t| core::mem::replace(t, table));

    // The new widths must not conflict with 'listchars' or 'fillchars'.
    if let Err(error) = check_chars_options() {
        emsg_text(error.to_string());
        CELL_WIDTHS.with_mut(|t| *t = saved);
        return;
    }

    changed_window_setting_all();
    redraw_all_later(UPD_NOT_VALID);
}

/// Why `setcellwidths()`'s list cannot be a table. `item` is the entry's
/// index in the outer list, which is what the messages name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
enum CellWidthError {
    #[error("{}", tr!("E1109: List item {} is not a List", .0))]
    NotAList(c_int),
    #[error("{}", tr!("E1110: List item {} does not contain 3 numbers", .0))]
    NotThreeNumbers(c_int),
    #[error("{}", tr!("E1111: List item {} range invalid", .0))]
    BadRange(c_int),
    #[error("{}", tr!("E1112: List item {} cell width invalid", .0))]
    BadWidth(c_int),
    #[error("{}", tr!("E1113: Overlapping ranges for 0x{:x}", .0))]
    Overlap(VarNumber),
    #[error("{}", tr!("E1114: Only values of 0x80 and higher supported"))]
    BelowNonAscii,
}

/// Validate `setcellwidths()`'s argument into the sorted, disjoint table
/// [`cw_value`] needs, or say why it cannot be one.
///
/// An empty list is a valid empty table — that is how the overrides are
/// cleared.
fn parse_cell_widths(list: &List) -> Result<Vec<CellWidthRange>, CellWidthError> {
    let mut rows: Vec<CellWidthRange> = Vec::with_capacity(list.len());
    for (item, li) in list_iter(Some(list)).enumerate() {
        let item = index_of(item);
        let row = li.li_tv.list_ref().ok_or(CellWidthError::NotAList(item))?;
        rows.push(parse_cell_width_row(row, item)?);
    }

    // Upstream sorts with qsort, which is unstable; two rows sharing a
    // `first` overlap either way round and are rejected below with the
    // same number, so a stable sort answers identically.
    rows.sort_by_key(|r| r.first);
    for i in 1..rows.len() {
        if rows[i].first <= rows[i - 1].last {
            return Err(CellWidthError::Overlap(rows[i].first));
        }
    }
    Ok(rows)
}

/// One `[first, last, width]` entry, or why it is not one.
///
/// `item` is the entry's index in the outer list, which is what the messages
/// name.
fn parse_cell_width_row(row: &List, item: c_int) -> Result<CellWidthRange, CellWidthError> {
    let mut numbers = [0 as VarNumber; 3];
    let mut seen = 0;
    for lili in list_iter(Some(row)) {
        let tv = &lili.li_tv;
        if tv.v_type() as c_uint != VAR_NUMBER as c_uint {
            break;
        }
        let n = tv.number_or_zero();
        match seen {
            0 if n < 0x80 => return Err(CellWidthError::BelowNonAscii),
            1 if n < numbers[0] => return Err(CellWidthError::BadRange(item)),
            2 if !(1..=2).contains(&n) => return Err(CellWidthError::BadWidth(item)),
            _ => {}
        }
        if seen < numbers.len() {
            numbers[seen] = n;
        }
        seen += 1;
    }

    // A fourth number, a non-number, or too few: all "not three numbers".
    if seen != 3 {
        return Err(CellWidthError::NotThreeNumbers(item));
    }
    Ok(CellWidthRange {
        first: numbers[0],
        last: numbers[1],
        width: c_int::try_from(numbers[2]).expect("checked to be 1 or 2"),
    })
}

/// `getcellwidths()` — the table `setcellwidths()` installed, as a List of
/// `[first, last, width]`.
pub fn f_getcellwidths(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let rows = CELL_WIDTHS.with(|t| t.clone());
    let list = tv_list_alloc_ret(result, isize::try_from(rows.len()).expect("a short table"));
    for row in &rows {
        let mut entry = tv_list_alloc(3);
        let into = &mut *entry;
        into.push_number(row.first);
        into.push_number(row.last);
        into.push_number(VarNumber::from(row.width));
        list.push_list(Some(entry));
    }
}
