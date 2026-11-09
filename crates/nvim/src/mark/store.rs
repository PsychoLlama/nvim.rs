//! The mark stores, and the names the rest of `mark/` reaches them by.
//!
//! `mark/` owns six containers of the same two record types, spread across
//! three lifetimes:
//!
//! | container | record | lives on |
//! | --- | --- | --- |
//! | `NAMEDFM` | `XFileMark` | a global, `'A`-`'Z` then `'0`-`'9` |
//! | `b_namedm` | `FileMark` | a buffer, `'a`-`'z` |
//! | `b_last_cursor` / `b_last_insert` / `b_last_change` / `b_prompt_start` | `FileMark` | a buffer |
//! | `b_changelist` | `FileMark` | a buffer |
//! | `w_jumplist` | `XFileMark` | a window |
//! | `w_tagstack` | `Taggy`, whose `fmark` is one | a window |
//!
//! Every record owns what it holds -- its ShaDa extra data, and an
//! `XFileMark`'s file name -- so a store is written by assignment and read
//! by copy: [`FileMark`]'s `Clone` is deep and its drop releases the extra
//! data. A record is reached for one statement at a time, through the
//! buffer's or window's handle or through one closure over the global table,
//! and nothing holds a reference into a store across code that may run
//! autocommands: the stores are walked while the editor is re-entered
//! through them (an adjustment fires autocommands, a jump list walk loads a
//! file), so a walk keeps an index and re-reads.
//!
//! [`LocalMark`] names one of a buffer's stores by the mark's name, for the
//! four places that choose a store that way. The global table is
//! [`NAMEDFM`], reached through `GlobalCell`'s closures; the index
//! arithmetic that names a slot stays with its callers (see
//! `mark/lookup.rs`'s `mark_global_index`) because turning the fixed array
//! into a keyed map is a later change, not this module's.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::{ColNr, Handle, LineNr, Pos};
use core::ffi::{c_char, c_int};

use crate::mark::defs::{FileMark, FileMarkView, XFileMark};
use crate::os::time::os_time;
use crate::winlayer::Buf;

/// The position a mark that has never been set reports, and what
/// [`FileMark::clear`] puts back.
pub(super) const UNSET_POS: Pos = Pos {
    lnum: LineNr::ZERO,
    col: ColNr::ZERO,
    coladd: 0,
};

/// One byte of a mark name, as the stores spell it.
///
/// Every caller works the name out by adding an index to a letter, and every
/// such sum is ASCII by construction, so a failure is a bug in that
/// arithmetic rather than in the user's input.
pub(super) fn mark_name(c: u8) -> c_char {
    c_char::try_from(c).expect("mark name is one ASCII byte")
}

/// The mark name `i` places after `first`: `nth_name(b'a', 2)` is `'c`.
///
/// The callers count within one class of names, so the sum is ASCII by
/// construction and a failure is a bug in that count.
pub(super) fn nth_name(first: u8, i: c_int) -> u8 {
    first + u8::try_from(i).expect("mark name is one ASCII byte")
}

/// A position in one of the fixed-size stores, bounds-checked.
///
/// Every caller works `i` out from a mark name or a list length that has
/// already been range tested, so a failure here is a bug in that arithmetic
/// rather than in the user's input.
pub(super) fn slot(i: c_int) -> usize {
    usize::try_from(i).expect("mark store index in range")
}

impl FileMark {
    /// A mark at `pos` in buffer `fnum`, stamped now, carrying nothing else:
    /// what upstream's `SET_FMARK`/`RESET_FMARK` write. The timestamp is taken
    /// here rather than passed in, because it is what the ShaDa merge compares.
    ///
    /// Assigning it over a store's record is `RESET_FMARK`: the old record's
    /// extra data goes with it. Upstream's `SET_FMARK` -- the same write without
    /// the release -- only differs where the record was a shallow duplicate,
    /// which an owned record never is.
    pub(crate) fn fresh(pos: Pos, fnum: Option<Handle>, view: FileMarkView) -> FileMark {
        FileMark {
            mark: pos,
            in_buffer: fnum.unwrap_or(Handle::NONE),
            timestamp: os_time(),
            view,
            extra: None,
        }
    }
}

/// One of a buffer's own mark stores, named by the mark it holds.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum LocalMark {
    /// `'a` + the index.
    Named(usize),
    /// `'"` -- where the cursor was when the buffer was last left.
    LastCursor,
    /// `'^` -- where the last insert ended.
    LastInsert,
    /// `'.` -- where the last change was made.
    LastChange,
    /// `':` -- where a prompt buffer's prompt starts. Only meaningful when
    /// `bt_prompt(buf)`, which every caller checks first.
    PromptStart,
}

impl LocalMark {
    /// The store `name` names: `'a`-`'z`, `'"`, `'^`, `'.` or `':`, or
    /// `None` for any other name.
    pub(super) fn named(name: u8) -> Option<LocalMark> {
        Some(if name.is_ascii_lowercase() {
            LocalMark::Named(usize::from(name - b'a'))
        } else if name == b'"' {
            LocalMark::LastCursor
        } else if name == b'^' {
            LocalMark::LastInsert
        } else if name == b'.' {
            LocalMark::LastChange
        } else if name == b':' {
            LocalMark::PromptStart
        } else {
            return None;
        })
    }

    /// Run `f` on the record in `buffer`, which it must not re-enter the
    /// editor from: the record is lent for the call and no longer.
    pub(super) fn with<R>(self, mut buffer: Buf, f: impl FnOnce(&mut FileMark) -> R) -> R {
        let b = &mut *buffer;
        f(match self {
            LocalMark::Named(i) => &mut b.b_namedm[i],
            LocalMark::LastCursor => &mut b.b_last_cursor,
            LocalMark::LastInsert => &mut b.b_last_insert,
            LocalMark::LastChange => &mut b.b_last_change,
            LocalMark::PromptStart => &mut b.b_prompt_start,
        })
    }
}

/// Every slot of the global table, `'A` first, `'9` last, with its mark
/// name.
pub(super) fn global_mark_names() -> impl Iterator<Item = (usize, u8)> {
    (0..super::NGLOBALMARKS).map(|i| {
        // `'A`-`'Z` occupy `0..NMARKS` and `'0`-`'9` the rest.
        let name = if i < super::NMARKS {
            nth_name(b'A', i)
        } else {
            nth_name(b'0', i - super::NMARKS)
        };
        (slot(i), name)
    })
}

/// A copy of global mark slot `idx`.
#[inline(never)]
pub(super) fn global_mark(idx: usize) -> XFileMark {
    super::NAMEDFM.with(|table| table[idx].clone())
}

/// Replace global mark slot `idx` with `mark`, releasing what it held.
#[inline(never)]
pub(super) fn set_global_mark(idx: usize, mark: XFileMark) {
    super::NAMEDFM.with_mut(|table| table[idx] = mark);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shada::ShadaExtra;

    fn at(lnum: i32, col: i32) -> Pos {
        Pos {
            lnum: LineNr::new(lnum),
            col: ColNr::new(col),
            coladd: 0,
        }
    }

    #[test]
    fn an_unset_mark_reports_line_zero() {
        let fm = FileMark::UNSET;
        assert!(!fm.is_set());
        assert_eq!((fm.mark.lnum, fm.mark.col), (UNSET_POS.lnum, UNSET_POS.col));
        assert!(fm.extra.is_none());
    }

    /// What `clear` promises: the position and the extra data go, the
    /// timestamp stays as given, and the view is the "remember nothing" one.
    #[test]
    fn clearing_keeps_the_timestamp_it_is_given() {
        let mut fm = FileMark {
            mark: at(12, 3),
            in_buffer: Handle::new(7),
            timestamp: 1,
            view: FileMarkView {
                topline_offset: 4,
                skipcol: ColNr::new(2),
            },
            extra: Some(Box::new(ShadaExtra::from_parts(1, vec![0xa1, b'k', 7]))),
        };
        fm.clear(4242);
        assert!(!fm.is_set());
        assert_eq!(fm.fnum(), None);
        assert_eq!(fm.timestamp, 4242);
        assert_eq!(fm.view.topline_offset, FileMarkView::NONE.topline_offset);
        assert!(fm.extra.is_none());
    }

    /// A copy owns its own extra data: clearing the original leaves the copy
    /// with all of it.
    #[test]
    fn a_copy_keeps_its_own_extra_data() {
        let extra = ShadaExtra::from_parts(2, vec![0xa1, b'a', 1, 0xa1, b'b', 2]);
        let mut original = XFileMark {
            fmark: FileMark {
                mark: at(5, 1),
                extra: Some(Box::new(extra.clone())),
                ..FileMark::UNSET
            },
            fname: Some(crate::memory::ThinCString::from_bytes(b"/a/file")),
        };
        let copy = original.clone();
        original.fmark.clear(9);
        original.fname = None;
        assert_eq!(copy.fmark.extra.as_deref(), Some(&extra));
        assert_eq!(copy.file_name(), Some(c"/a/file"));
        assert_eq!(copy.fmark.mark.lnum, LineNr::new(5));
        assert!(original.fmark.extra.is_none());
    }

    #[test]
    fn local_stores_are_named_by_their_mark() {
        assert_eq!(LocalMark::named(b'a'), Some(LocalMark::Named(0)));
        assert_eq!(LocalMark::named(b'z'), Some(LocalMark::Named(25)));
        assert_eq!(LocalMark::named(b'"'), Some(LocalMark::LastCursor));
        assert_eq!(LocalMark::named(b'^'), Some(LocalMark::LastInsert));
        assert_eq!(LocalMark::named(b'.'), Some(LocalMark::LastChange));
        assert_eq!(LocalMark::named(b':'), Some(LocalMark::PromptStart));
        assert_eq!(LocalMark::named(b'A'), None);
        assert_eq!(LocalMark::named(b'['), None);
    }

    #[test]
    fn the_global_table_is_named_upper_case_then_digits() {
        let names: Vec<u8> = global_mark_names().map(|(_, name)| name).collect();
        assert_eq!(names, b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        assert_eq!(global_mark_names().nth(26).map(|(i, _)| i), Some(26));
    }
}
