//! The fold tree's one untyped edge.
//!
//! Every fold list in a window — `w_folds`, and the `fd_nested` of every fold
//! in it — is a `garray_T`: an untyped growable array whose items happen to
//! be [`fold_T`]. Reaching a fold therefore means casting `ga_data` and doing
//! pointer arithmetic against `ga_len`, and before this module every one of
//! the forty-odd walks in `fold/` did that arithmetic for itself.
//!
//! [`FoldList`] and [`Fold`] make the cast once. *Constructing* a handle is
//! the unsafe step — the caller promises the growarray really is a live fold
//! list — and every method on it is safe, because the handle already carries
//! the promise the access needs. A walk over the tree therefore costs no
//! unchecked lines at all.
//!
//! The handles are raw-pointer-shaped rather than `&mut [fold_T]` slices on
//! purpose: the tree is walked recursively while entries are inserted,
//! deleted, split and merged *under* the walk, so two live `&mut` into one
//! array would be routine rather than exceptional. [`FoldList::at`] therefore
//! hands back a `Fold`, whose accessors each raise a reference for the length
//! of one field read or write. Miri watches this through
//! `crates/nvim/tests/unit/fold.rs`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{fline_T, fold_T};
use crate::types::{garray_T, linenr_T};
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};

/// One fold list: a `garray_T` whose items are [`fold_T`].
///
/// `Copy`, because it is a handle and not an owner — dropping one frees
/// nothing.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct FoldList {
    gap: *mut garray_T,
}

/// One entry of a [`FoldList`].
///
/// A `Fold` may legally address `list.len()` — one past the end — because
/// that is what a failed [`FoldList::find`] names and what several callers
/// compare against before deciding whether to look. It may also address
/// `-1`, which `fold_move_to` walks to deliberately. Reading a field through
/// either is the caller's mistake, exactly as it was upstream; the handle
/// promises the *array*, not the index.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Fold {
    fp: *mut fold_T,
}

impl FoldList {
    /// # Safety
    /// `gap` must point at a live fold list — a window's `w_folds`, or the
    /// `fd_nested` of a fold in one. Both are initialised with
    /// `ga_itemsize == size_of::<fold_T>()` by `fold_init_win`,
    /// `clone_fold_list` or `fold_insert`, and neither the growarray nor its
    /// data may be freed while the returned handle is in use.
    pub(super) const unsafe fn new(gap: *mut garray_T) -> Self {
        Self { gap }
    }

    /// The growarray itself, for `ga_grow`/`ga_clear` and for the recursive
    /// entry points that still take a `*mut garray_T`.
    pub(super) fn gap(self) -> *mut garray_T {
        self.gap
    }

    /// How many folds the list holds.
    pub(super) fn len(self) -> c_int {
        // SAFETY: `new`'s caller promised a live fold list.
        unsafe { (*self.gap).ga_len }
    }

    /// Set the number of folds, after the caller has moved entries around by
    /// hand.
    pub(super) fn set_len(self, len: c_int) {
        // SAFETY: as `len`.
        unsafe { (*self.gap).ga_len = len };
    }

    pub(super) fn is_empty(self) -> bool {
        self.len() <= 0
    }

    /// Whether the list has ever been given storage. An empty list may still
    /// have a null `ga_data`, which is why the walks that shrink a list check
    /// this before looking at an entry.
    pub(super) fn has_data(self) -> bool {
        // SAFETY: as `len`.
        !unsafe { (*self.gap).ga_data }.is_null()
    }

    /// The `i`th fold. `i == len()` yields the one-past-the-end handle the
    /// walks in this module compare against; no bounds check is made, because
    /// several callers rely on naming that entry.
    pub(super) fn at(self, i: c_int) -> Fold {
        // `wrapping_offset` is a safe operation, so the arithmetic that used
        // to be spelled `folds(gap).offset(i)` inside an `unsafe` block costs
        // nothing here.
        Fold {
            fp: self.data().wrapping_offset(i as isize),
        }
    }

    /// Where `fold` sits in this list. May be negative or `>= len()`; see
    /// [`Fold`].
    pub(super) fn index_of(self, fold: Fold) -> c_int {
        let bytes = fold.fp.addr() as isize - self.data().addr() as isize;
        (bytes / size_of::<fold_T>() as isize) as c_int
    }

    /// Whether `fold` names an entry that is really there.
    pub(super) fn holds(self, fold: Fold) -> bool {
        let i = self.index_of(fold);
        i >= 0 && i < self.len()
    }

    /// Search for line `lnum`.
    ///
    /// `Ok(i)` is the fold that contains `lnum`; `Err(i)` is the first fold
    /// *below* it, which may be `len()` — there is a gap at `lnum`, or `lnum`
    /// is past the last fold.
    ///
    /// The search is written out rather than handed to
    /// `slice::binary_search_by` so that a malformed list — a zero-length
    /// entry, say — lands on exactly the entry `fold.c`'s loop would have
    /// landed on.
    pub(super) fn find(self, lnum: linenr_T) -> Result<c_int, c_int> {
        let mut low: c_int = 0;
        let mut high: c_int = self.len() - 1;
        while low <= high {
            let i = (low + high) / 2;
            let fold = self.at(i);
            if fold.top() > lnum {
                high = i - 1;
            } else if fold.top() + fold.len() <= lnum {
                low = i + 1;
            } else {
                return Ok(i);
            }
        }
        Err(low)
    }

    /// Every fold in the list, oldest index first.
    ///
    /// `ga_len` is re-read before each step, so a walk that deletes the entry
    /// it is standing on still terminates.
    pub(super) fn folds(self) -> impl Iterator<Item = Fold> {
        (0..)
            .take_while(move |&i| i < self.len())
            .map(move |i| self.at(i))
    }

    fn data(self) -> *mut fold_T {
        // SAFETY: as `len`.
        unsafe { (*self.gap).ga_data }.cast()
    }
}

impl Fold {
    /// The entry's address, for the handful of callers that still pass a
    /// `*mut fold_T` across a module boundary.
    pub(super) fn entry(self) -> *mut fold_T {
        self.fp
    }

    /// The fold `by` entries further along. See [`Fold`] for what is allowed
    /// to come out of this.
    pub(super) fn offset(self, by: c_int) -> Self {
        Self {
            fp: self.fp.wrapping_offset(by as isize),
        }
    }

    /// The fold's first line, relative to the start of its parent.
    pub(super) fn top(self) -> linenr_T {
        // SAFETY: the handle names an entry of a live fold list.
        unsafe { (*self.fp).fd_top }
    }

    pub(super) fn set_top(self, top: linenr_T) {
        // SAFETY: as `top`.
        unsafe { (*self.fp).fd_top = top };
    }

    /// How many lines the fold spans.
    pub(super) fn len(self) -> linenr_T {
        // SAFETY: as `top`.
        unsafe { (*self.fp).fd_len }
    }

    pub(super) fn set_len(self, len: linenr_T) {
        // SAFETY: as `top`.
        unsafe { (*self.fp).fd_len = len };
    }

    /// The fold's last line, in the same frame as [`Fold::top`].
    pub(super) fn last(self) -> linenr_T {
        self.top() + self.len() - 1
    }

    /// `FD_OPEN`, `FD_CLOSED` or `FD_LEVEL`.
    pub(super) fn flags(self) -> c_int {
        // SAFETY: as `top`.
        unsafe { (*self.fp).fd_flags as c_int }
    }

    pub(super) fn set_flags(self, flags: c_int) {
        // SAFETY: as `top`.
        unsafe { (*self.fp).fd_flags = flags as c_char };
    }

    /// Whether the fold is drawn with `flags`.
    pub(super) fn is(self, flags: c_int) -> bool {
        self.flags() == flags
    }

    /// Whether the fold is smaller than 'foldminlines'. `None` means "not
    /// worked out yet", and applies to the nested folds too.
    pub(super) fn small(self) -> Option<bool> {
        // SAFETY: as `top`.
        unsafe { (*self.fp).fd_small }
    }

    pub(super) fn set_small(self, small: Option<bool>) {
        // SAFETY: as `top`.
        unsafe { (*self.fp).fd_small = small };
    }

    /// The folds nested inside this one.
    pub(super) fn nested(self) -> FoldList {
        // SAFETY: `fd_nested` of a fold in a live list is itself a live fold
        // list -- that is the tree's shape, and `fold_insert` initialises it.
        unsafe { FoldList::new(&raw mut (*self.fp).fd_nested) }
    }

    /// A copy of the whole entry, for the array moves in `fold_split` and
    /// `fold_merge`.
    pub(super) fn read(self) -> fold_T {
        // SAFETY: as `top`.
        unsafe { *self.fp }
    }

    pub(super) fn write(self, fold: fold_T) {
        // SAFETY: as `top`.
        unsafe { *self.fp = fold };
    }
}

/// The per-line state the computed fold methods are handed, and answer in.
///
/// One `fline_T` travels the whole of `fold_update_computed_recurse`: each
/// level of the recursion reads what the level above left and writes what the
/// level below will read, which is why it is passed by pointer rather than by
/// `&mut`. The same trick as [`FoldList`] applies — the promise is made once,
/// at construction, and the field accessors are safe.
#[derive(Copy, Clone)]
pub(super) struct FLine {
    flp: *mut fline_T,
}

impl FLine {
    /// # Safety
    /// `flp` must point at a live, writable `fline_T` naming a live window,
    /// and must stay so for as long as the handle is used.
    pub(super) const unsafe fn new(flp: *mut fline_T) -> Self {
        Self { flp }
    }

    /// The `fline_T` behind the handle, for the two calls that still speak
    /// in pointers: the recursion's own re-entry and the C struct field.
    pub(super) fn raw(self) -> *mut fline_T {
        self.flp
    }

    /// The window whose folds are being computed.
    pub(super) fn win(self) -> Win {
        // SAFETY: `new`'s caller promised a live `fline_T` naming a live
        // window.
        unsafe { Win::new((*self.flp).wp) }
    }

    /// Current line number, relative to the start of the enclosing fold.
    pub(super) fn lnum(self) -> linenr_T {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lnum }
    }

    pub(super) fn set_lnum(self, lnum: linenr_T) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lnum = lnum };
    }

    /// Offset between [`FLine::lnum`] and the real buffer line.
    pub(super) fn off(self) -> linenr_T {
        // SAFETY: as `win`.
        unsafe { (*self.flp).off }
    }

    pub(super) fn set_off(self, off: linenr_T) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).off = off };
    }

    /// The line the level was actually read from, when the level at
    /// [`FLine::lnum`] is undefined.
    pub(super) fn lnum_save(self) -> linenr_T {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lnum_save }
    }

    pub(super) fn set_lnum_save(self, lnum: linenr_T) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lnum_save = lnum };
    }

    /// Level of this line; -1 for undefined.
    pub(super) fn lvl(self) -> c_int {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lvl }
    }

    pub(super) fn set_lvl(self, lvl: c_int) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lvl = lvl };
    }

    /// Level to use for the next line.
    pub(super) fn lvl_next(self) -> c_int {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lvl_next }
    }

    pub(super) fn set_lvl_next(self, lvl: c_int) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).lvl_next = lvl };
    }

    /// How many folds are forced to start at this line.
    pub(super) fn start(self) -> c_int {
        // SAFETY: as `win`.
        unsafe { (*self.flp).start }
    }

    pub(super) fn set_start(self, start: c_int) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).start = start };
    }

    /// Level of the fold forced to end below this line.
    pub(super) fn end(self) -> c_int {
        // SAFETY: as `win`.
        unsafe { (*self.flp).end }
    }

    pub(super) fn set_end(self, end: c_int) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).end = end };
    }

    /// Level of the fold forced to end above this line — the previous line's
    /// [`FLine::end`].
    pub(super) fn had_end(self) -> c_int {
        // SAFETY: as `win`.
        unsafe { (*self.flp).had_end }
    }

    pub(super) fn set_had_end(self, end: c_int) {
        // SAFETY: as `win`.
        unsafe { (*self.flp).had_end = end };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr;

    /// Build a detached fold list from `(top, len)` pairs. The entries are
    /// leaked with the array; the tests are short and Miri only cares that
    /// nothing is read out of bounds.
    fn list(spans: &[(linenr_T, linenr_T)]) -> (Box<garray_T>, Vec<fold_T>) {
        let mut folds: Vec<fold_T> = spans
            .iter()
            .map(|&(top, len)| fold_T {
                fd_top: top,
                fd_len: len,
                fd_nested: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: size_of::<fold_T>() as c_int,
                    ga_growsize: 10,
                    ga_data: ptr::null_mut(),
                },
                fd_flags: 0,
                fd_small: None,
            })
            .collect();
        let gap = Box::new(garray_T {
            ga_len: folds.len() as c_int,
            ga_maxlen: folds.len() as c_int,
            ga_itemsize: size_of::<fold_T>() as c_int,
            ga_growsize: 10,
            ga_data: folds.as_mut_ptr().cast(),
        });
        (gap, folds)
    }

    /// `FoldList::new` over a list built above.
    fn handle(gap: &mut garray_T) -> FoldList {
        // SAFETY: `list` built it with `ga_itemsize == size_of::<fold_T>()`
        // and the storage outlives the handle.
        unsafe { FoldList::new(&raw mut *gap) }
    }

    #[test]
    fn find_lands_on_the_fold_that_contains_the_line() {
        let (mut gap, _keep) = list(&[(2, 3), (10, 1), (20, 5)]);
        let folds = handle(&mut gap);
        assert_eq!(folds.find(2), Ok(0));
        assert_eq!(folds.find(4), Ok(0));
        assert_eq!(folds.find(10), Ok(1));
        assert_eq!(folds.find(24), Ok(2));
    }

    #[test]
    fn a_miss_names_the_first_fold_below_the_line() {
        let (mut gap, _keep) = list(&[(2, 3), (10, 1), (20, 5)]);
        let folds = handle(&mut gap);
        // Above everything, in the gaps, and past the end.
        assert_eq!(folds.find(1), Err(0));
        assert_eq!(folds.find(5), Err(1));
        assert_eq!(folds.find(11), Err(2));
        assert_eq!(folds.find(25), Err(3));
        assert_eq!(folds.find(9_999), Err(3));
    }

    #[test]
    fn an_empty_list_misses_at_zero() {
        let (mut gap, _keep) = list(&[]);
        let folds = handle(&mut gap);
        assert!(folds.is_empty());
        assert_eq!(folds.find(1), Err(0));
        assert_eq!(folds.folds().count(), 0);
    }

    #[test]
    fn a_handle_knows_where_it_sits() {
        let (mut gap, _keep) = list(&[(2, 3), (10, 1), (20, 5)]);
        let folds = handle(&mut gap);
        for i in 0..3 {
            assert_eq!(folds.index_of(folds.at(i)), i);
            assert!(folds.holds(folds.at(i)));
        }
        // The two handles the walks in this module rely on naming.
        assert_eq!(folds.index_of(folds.at(3)), 3);
        assert!(!folds.holds(folds.at(3)));
        assert!(!folds.holds(folds.at(0).offset(-1)));
        assert_eq!(folds.at(1).offset(1), folds.at(2));
    }

    #[test]
    fn a_fold_reports_the_span_it_was_built_with() {
        let (mut gap, _keep) = list(&[(2, 3), (10, 1)]);
        let folds = handle(&mut gap);
        assert_eq!(folds.at(0).top(), 2);
        assert_eq!(folds.at(0).len(), 3);
        assert_eq!(folds.at(0).last(), 4);
        // A one-line fold's last line is its first.
        assert_eq!(folds.at(1).last(), 10);
        assert_eq!(
            folds
                .folds()
                .map(|f| (f.top(), f.len()))
                .collect::<Vec<_>>(),
            [(2, 3), (10, 1)]
        );
    }

    #[test]
    fn writes_go_through_to_the_array() {
        let (mut gap, keep) = list(&[(2, 3)]);
        let folds = handle(&mut gap);
        folds.at(0).set_top(7);
        folds.at(0).set_len(2);
        folds.at(0).set_flags(super::super::FD_CLOSED);
        folds.at(0).set_small(Some(true));
        assert_eq!((keep[0].fd_top, keep[0].fd_len), (7, 2));
        assert!(folds.at(0).is(super::super::FD_CLOSED));
        assert_eq!(folds.at(0).small(), Some(true));
    }

    #[test]
    fn the_walk_re_reads_the_length_each_step() {
        let (mut gap, _keep) = list(&[(2, 3), (10, 1), (20, 5)]);
        let folds = handle(&mut gap);
        // What `fold_remove` and `fold_mark_adjust_recurse` depend on: a walk
        // that shortens the list under itself stops at the new end rather
        // than reading past it.
        let mut seen = Vec::new();
        for fold in folds.folds() {
            seen.push(fold.top());
            folds.set_len(1);
        }
        assert_eq!(seen, [2]);
    }
}
