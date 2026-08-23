//! The mark stores, wrapped so that reaching one is not an unsafe operation.
//!
//! `mark/` owns six containers of the same two record types, spread across
//! three lifetimes:
//!
//! | container | record | lives on |
//! | --- | --- | --- |
//! | `namedfm` | `xfmark_T` | a global, `'A`-`'Z` then `'0`-`'9` |
//! | `b_namedm` | `fmark_T` | a buffer, `'a`-`'z` |
//! | `b_last_cursor` / `b_last_insert` / `b_last_change` / `b_prompt_start` | `fmark_T` | a buffer |
//! | `b_changelist` | `fmark_T` | a buffer |
//! | `w_jumplist` | `xfmark_T` | a window |
//! | `w_tagstack` | `taggy_T`, whose `fmark` is one | a window |
//!
//! Every walk over any of them was written out as a cast-and-offset —
//! `(&raw mut (*buf).b_namedm as *mut fmark_T).offset(i as isize)` and its
//! seven siblings — inside an `unsafe fn` whose whole body was therefore
//! unchecked. [`Fmark`] and [`Xfmark`] make that step once: *constructing* a
//! handle is the unsafe part, and every accessor on it is a safe method. It
//! is the same lever as [`crate::winlayer`]'s `Win`/`Buf`, `undo::store`'s
//! `Header` and `fold::list`'s `FoldList` — the third container family to
//! pay for it, and the one where the arithmetic was most repeated.
//!
//! The handles are raw-pointer-shaped rather than `&mut fmark_T` because the
//! stores are walked while the editor is re-entered through them: an
//! adjustment fires autocommands, a jumplist walk loads a file, and a global
//! mark and a jumplist entry can name the same buffer. Each accessor raises a
//! reference for the length of one field access, exactly as `winlayer`'s
//! `Deref` does.
//!
//! `namedfm` is reached through [`GlobalMarks`] rather than through
//! `namedfm.ptr()` at thirty call sites; the escape hatch is taken once, here,
//! and the index arithmetic that names a slot stays with its five callers
//! (see `mark/lookup.rs`'s `mark_global_index`) because turning the fixed
//! array into a keyed map is a later phase's change, not this module's.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::main::namedfm;
use crate::os::time::os_time;
use crate::pos::MAXLNUM;
use crate::types::{Timestamp, colnr_T, fmark_T, fmarkv_T, linenr_T, pos_T, xfmark_T};
use crate::winlayer::{Buf, Win};

use super::{NGLOBALMARKS, NMARKS, free_fmark, free_xfmark};

/// The position a mark that has never been set reports, and what
/// [`Fmark::clear`] puts back.
pub(super) const UNSET_POS: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// The view an unset mark carries: `topline_offset` at `MAXLNUM` means
/// "remember nothing", which is what `mark_view_restore`'s `>= 0` test
/// rejects.
pub(super) const NO_VIEW: fmarkv_T = fmarkv_T {
    topline_offset: MAXLNUM.cast_signed(),
    skipcol: 0,
};

/// An `fmark_T` that is not set, timestamped now by the caller.
pub(super) const UNSET_FMARK: fmark_T = fmark_T {
    mark: UNSET_POS,
    fnum: 0,
    timestamp: 0,
    view: NO_VIEW,
    additional_data: ptr::null_mut(),
};

/// An `xfmark_T` that is not set.
pub(super) const UNSET_XFMARK: xfmark_T = xfmark_T {
    fmark: UNSET_FMARK,
    fname: ptr::null_mut(),
};

/// `NUL` in the stores' own byte type: `NUL` itself is a `c_int`, and every
/// mark name lives in a `c_char`.
pub(super) const NUL_BYTE: c_char = 0;

/// One byte of a mark name, as the stores spell it.
///
/// Every caller works the name out by adding an index to a letter, and every
/// such sum is ASCII by construction, so a failure is a bug in that
/// arithmetic rather than in the user's input.
pub(super) fn mark_name(c: c_int) -> c_char {
    c_char::try_from(c).expect("mark name is one ASCII byte")
}

/// The `i`th slot of a fixed-size mark array, bounds-checked.
///
/// Every caller works `i` out from a mark name or a list length that has
/// already been range tested, so a failure here is a bug in that arithmetic
/// rather than in the user's input. It is checked anyway, in release as well
/// as debug, because the alternative is a read past the array — which is what
/// the transpiled `(&raw mut (*buf).b_namedm as *mut fmark_T).offset(i)` did.
fn slot(i: c_int, len: usize) -> usize {
    usize::try_from(i)
        .ok()
        .filter(|&idx| idx < len)
        .expect("mark store index in range")
}

// ---------------------------------------------------------------------------
// The two records

/// One `fmark_T` in place.
///
/// `Copy`, because it is a handle and not an owner: dropping one frees
/// nothing, and [`Fmark::clear`] is what releases the record's
/// `additional_data`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct Fmark(*mut fmark_T);

impl Fmark {
    /// # Safety
    /// `fm` must stay a live, writable `fmark_T` for as long as the handle is
    /// used.
    #[inline(always)]
    pub(super) const unsafe fn new(fm: *mut fmark_T) -> Self {
        Self(fm)
    }

    /// The record's address, for the calls that still take a `*mut fmark_T`
    /// across a module boundary (`mark_check`, `fm_getname`, `tv_dict_add_*`).
    #[inline(always)]
    pub(super) fn raw(self) -> *mut fmark_T {
        self.0
    }

    /// The address of the position inside the record, which `:marks` and
    /// `getmarklist()` pass around on its own.
    #[inline(always)]
    pub(super) fn pos_raw(self) -> *mut pos_T {
        // `wrapping_byte_add` would be the safe spelling, but the field
        // offset is only knowable through a projection, and a projection
        // through a raw pointer is the unsafe operation.
        // SAFETY: `new`'s caller promised a live record.
        unsafe { &raw mut (*self.0).mark }
    }

    /// The whole record, copied out.
    #[inline(always)]
    pub(super) fn read(self) -> fmark_T {
        // SAFETY: as `pos_raw`.
        unsafe { *self.0 }
    }

    /// Overwrite the whole record. Does not free what was there; see
    /// [`Fmark::place`].
    #[inline(always)]
    pub(super) fn write(self, fm: fmark_T) {
        // SAFETY: as `pos_raw`.
        unsafe { *self.0 = fm };
    }

    #[inline(always)]
    pub(super) fn pos(self) -> pos_T {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).mark }
    }

    #[inline(always)]
    pub(super) fn set_pos(self, pos: pos_T) {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).mark = pos };
    }

    #[inline(always)]
    pub(super) fn lnum(self) -> linenr_T {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).mark.lnum }
    }

    #[inline(always)]
    pub(super) fn set_lnum(self, lnum: linenr_T) {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).mark.lnum = lnum };
    }

    #[inline(always)]
    pub(super) fn col(self) -> colnr_T {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).mark.col }
    }

    #[inline(always)]
    pub(super) fn fnum(self) -> c_int {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).fnum }
    }

    #[inline(always)]
    pub(super) fn set_fnum(self, fnum: c_int) {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).fnum = fnum };
    }

    #[inline(always)]
    pub(super) fn timestamp(self) -> Timestamp {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).timestamp }
    }

    #[inline(always)]
    pub(super) fn set_timestamp(self, timestamp: Timestamp) {
        // SAFETY: as `pos_raw`.
        unsafe { (*self.0).timestamp = timestamp };
    }

    /// Whether the mark names a line. Line 0 is how every store spells "not
    /// set", and what `mark_check` turns into `E20`.
    #[inline(always)]
    pub(super) fn is_set(self) -> bool {
        self.lnum() != 0
    }

    /// Put the mark at `pos` in buffer `fnum`, **abandoning** whatever was
    /// there: upstream's `SET_FMARK`.
    ///
    /// The timestamp is taken here rather than passed in, because it is what
    /// `mark_set_global`/`mark_set_local` compare when a shada file is merged.
    ///
    /// The old record's `additional_data` is NOT freed. Only `setpcmark` uses
    /// this arm, and it writes over a jump list entry the caller has already
    /// dealt with; every other store wants [`Fmark::replace`]. The two are one
    /// `free_fmark` apart and upstream keeps them as two macros for the same
    /// reason.
    pub(super) fn place(self, pos: pos_T, fnum: c_int, view: fmarkv_T) {
        self.write(fmark_T {
            mark: pos,
            fnum,
            timestamp: os_time(),
            view,
            additional_data: ptr::null_mut(),
        });
    }

    /// [`Fmark::place`], releasing what was there first: upstream's
    /// `RESET_FMARK`.
    pub(super) fn replace(self, pos: pos_T, fnum: c_int, view: fmarkv_T) {
        // SAFETY: `new`'s caller promised a live record, so the old value is
        // readable and its `additional_data` is this store's to free.
        unsafe { free_fmark(self.read()) };
        self.place(pos, fnum, view);
    }

    /// Release the record and put an unset one in its place, stamped
    /// `timestamp` so a later shada merge can tell the clearing from the
    /// records it is merging.
    pub(super) fn clear(self, timestamp: Timestamp) {
        // SAFETY: as `place`.
        unsafe { free_fmark(self.read()) };
        self.write(fmark_T {
            timestamp,
            ..UNSET_FMARK
        });
    }
}

/// One `xfmark_T` in place: an [`Fmark`] plus the file name a mark read out of
/// the shada file carries until its buffer is loaded and `fname2fnum` swaps
/// the two.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct Xfmark(*mut xfmark_T);

impl Xfmark {
    /// # Safety
    /// `xfm` must stay a live, writable `xfmark_T` for as long as the handle
    /// is used.
    #[inline(always)]
    pub(super) const unsafe fn new(xfm: *mut xfmark_T) -> Self {
        Self(xfm)
    }

    /// The record's address, for the calls that still take a `*mut xfmark_T`
    /// (`fname2fnum`, the shada iterators).
    #[inline(always)]
    pub(super) fn raw(self) -> *mut xfmark_T {
        self.0
    }

    /// The file mark inside the record.
    #[inline(always)]
    pub(super) fn fmark(self) -> Fmark {
        // SAFETY: a live `xfmark_T` holds a live `fmark_T`.
        unsafe { Fmark::new(&raw mut (*self.0).fmark) }
    }

    #[inline(always)]
    pub(super) fn read(self) -> xfmark_T {
        // SAFETY: `new`'s caller promised a live record.
        unsafe { *self.0 }
    }

    #[inline(always)]
    pub(super) fn write(self, xfm: xfmark_T) {
        // SAFETY: as `read`.
        unsafe { *self.0 = xfm };
    }

    /// The remembered file name, null once the mark names a live buffer.
    #[inline(always)]
    pub(super) fn fname(self) -> *mut c_char {
        // SAFETY: as `read`.
        unsafe { (*self.0).fname }
    }

    #[inline(always)]
    pub(super) fn set_fname(self, fname: *mut c_char) {
        // SAFETY: as `read`.
        unsafe { (*self.0).fname = fname };
    }

    /// Free the remembered file name and forget it.
    pub(super) fn clear_fname(self) {
        // SAFETY: the name is this record's to free, and nothing else holds
        // it: `fname2fnum` copies into `NameBuff` before this runs.
        unsafe { crate::memory::xfree(self.fname().cast()) };
        self.set_fname(ptr::null_mut());
    }

    /// Put the mark at `pos` in buffer `fnum` and forget the remembered file
    /// name, **abandoning** whatever was there: upstream's `SET_XFMARK`.
    ///
    /// As [`Fmark::place`], nothing is freed. `setpcmark` is the one caller.
    pub(super) fn place(self, pos: pos_T, fnum: c_int, view: fmarkv_T) {
        self.set_fname(ptr::null_mut());
        self.fmark().place(pos, fnum, view);
    }

    /// [`Xfmark::place`], releasing both halves of what was there first:
    /// upstream's `RESET_XFMARK`.
    pub(super) fn replace(self, pos: pos_T, fnum: c_int, view: fmarkv_T) {
        // SAFETY: `new`'s caller promised a live record; the name and the
        // `additional_data` are this store's to free.
        unsafe { free_xfmark(self.read()) };
        self.place(pos, fnum, view);
    }
}

// ---------------------------------------------------------------------------
// The containers

/// The global mark table: `'A`-`'Z` at 0..[`NMARKS`] and `'0`-`'9` above it,
/// up to [`NGLOBALMARKS`].
///
/// A zero-sized handle rather than a pointer: the table is one `static`, live
/// from startup to exit, so naming a slot needs no promise from the caller.
/// The single `namedfm.ptr()` in the module is [`GlobalMarks::at`]'s.
pub(super) struct GlobalMarks;

impl GlobalMarks {
    /// The `idx`th slot. `idx` must be in `0..NGLOBALMARKS`; the five places
    /// that work one out from a mark name all clamp it first, and the debug
    /// assertion is what says so at run time.
    pub(super) fn at(idx: c_int) -> Xfmark {
        let idx = slot(idx, NGLOBALMARKS as usize);
        // SAFETY: `namedfm` is a live `[xfmark_T; 36]` for the whole run and
        // `idx` is inside it, so the projection names a live record. The
        // handle borrows nothing: every access through it is one field at a
        // time, which is the contract `GlobalCell::ptr` documents.
        unsafe { Xfmark::new(&raw mut (*namedfm.ptr())[idx]) }
    }

    /// Every slot, in table order — `'A` first, `'9` last.
    pub(super) fn all() -> impl Iterator<Item = Xfmark> {
        (0..NGLOBALMARKS).map(Self::at)
    }

    /// Every slot with its index, for the callers that turn the index back
    /// into a mark name.
    pub(super) fn indexed() -> impl Iterator<Item = (c_int, Xfmark)> {
        (0..NGLOBALMARKS).map(|i| (i, Self::at(i)))
    }

    /// Which slot `at` names. The shada iterator's opaque token is the
    /// address of a slot, and this is how it resumes from one.
    pub(super) fn index_of(at: *const xfmark_T) -> c_int {
        let bytes = at.addr().wrapping_sub(Self::at(0).raw().addr());
        let idx = bytes.wrapping_div(size_of::<xfmark_T>());
        c_int::try_from(idx)
            .ok()
            .filter(|i| (0..NGLOBALMARKS).contains(i))
            .expect("global mark token in range")
    }
}

/// The mark stores that hang off a buffer.
impl Buf {
    /// The buffer-local mark `'a` + `i`.
    pub(super) fn named_mark(self, i: c_int) -> Fmark {
        let i = slot(i, self.b_namedm.len());
        // SAFETY: a live buffer holds a live `[fmark_T; 26]`, and `i` is
        // inside it.
        unsafe { Fmark::new(&raw mut (*self.raw()).b_namedm[i]) }
    }

    /// Every buffer-local mark, `'a` first.
    pub(super) fn named_marks(self) -> impl Iterator<Item = Fmark> {
        (0..NMARKS).map(move |i| self.named_mark(i))
    }

    /// `'"` — where the cursor was when the buffer was last left.
    pub(super) fn last_cursor(self) -> Fmark {
        // SAFETY: a live buffer holds a live `fmark_T` here.
        unsafe { Fmark::new(&raw mut (*self.raw()).b_last_cursor) }
    }

    /// `'^` — where the last insert ended.
    pub(super) fn last_insert(self) -> Fmark {
        // SAFETY: as `last_cursor`.
        unsafe { Fmark::new(&raw mut (*self.raw()).b_last_insert) }
    }

    /// `'.` — where the last change was made.
    pub(super) fn last_change(self) -> Fmark {
        // SAFETY: as `last_cursor`.
        unsafe { Fmark::new(&raw mut (*self.raw()).b_last_change) }
    }

    /// `':` — where a prompt buffer's prompt starts. Only meaningful when
    /// `bt_prompt(buf)`, which every caller checks first.
    pub(super) fn prompt_start(self) -> Fmark {
        // SAFETY: as `last_cursor`.
        unsafe { Fmark::new(&raw mut (*self.raw()).b_prompt_start) }
    }

    /// How many entries the change list holds.
    pub(super) fn changelist_len(self) -> c_int {
        self.b_changelistlen
    }

    /// The `i`th change list entry.
    pub(super) fn change(self, i: c_int) -> Fmark {
        let i = slot(i, self.b_changelist.len());
        // SAFETY: a live buffer holds a live `[fmark_T; 100]`, and `i` is
        // inside it.
        unsafe { Fmark::new(&raw mut (*self.raw()).b_changelist[i]) }
    }

    /// The change list, oldest first.
    ///
    /// `b_changelistlen` is re-read before each step, so a walk that shortens
    /// the list under itself still terminates.
    pub(super) fn changes(self) -> impl Iterator<Item = Fmark> {
        (0..)
            .take_while(move |&i| i < self.changelist_len())
            .map(move |i| self.change(i))
    }
}

/// The mark stores that hang off a window.
impl Win {
    /// How many entries the jump list holds. `w_jumplistidx` may legally be
    /// this value — that one-past-the-end state is what makes `:jumps` print
    /// its trailing bare `>` and the first `<C-o>` reach the newest entry.
    pub(super) fn jumplist_len(self) -> c_int {
        self.w_jumplistlen
    }

    /// The `i`th jump list entry.
    pub(super) fn jump(self, i: c_int) -> Xfmark {
        let i = slot(i, self.w_jumplist.len());
        // SAFETY: a live window holds a live `[xfmark_T; 100]`, and `i` is
        // inside it.
        unsafe { Xfmark::new(&raw mut (*self.raw()).w_jumplist[i]) }
    }

    /// The jump list, oldest first. As [`Buf::changes`], the length is
    /// re-read every step.
    pub(super) fn jumps(self) -> impl Iterator<Item = Xfmark> {
        (0..)
            .take_while(move |&i| i < self.jumplist_len())
            .map(move |i| self.jump(i))
    }

    /// The file mark of the `i`th tag stack entry.
    pub(super) fn tag_mark(self, i: c_int) -> Fmark {
        let i = slot(i, self.w_tagstack.len());
        // SAFETY: a live window holds a live `[taggy_T; 20]`, and `i` is
        // inside it.
        unsafe { Fmark::new(&raw mut (*self.raw()).w_tagstack[i].fmark) }
    }

    /// The tag stack's file marks, oldest first.
    pub(super) fn tag_marks(self) -> impl Iterator<Item = Fmark> {
        let len = self.w_tagstacklen;
        (0..len).map(move |i| self.tag_mark(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(lnum: linenr_T, col: colnr_T) -> pos_T {
        pos_T {
            lnum,
            col,
            coladd: 0,
        }
    }

    /// A detached record, as every store's entry is one.
    fn record() -> Box<fmark_T> {
        Box::new(UNSET_FMARK)
    }

    fn handle(fm: &mut fmark_T) -> Fmark {
        // SAFETY: the box outlives the handle, and nothing else names it.
        unsafe { Fmark::new(&raw mut *fm) }
    }

    #[test]
    fn an_unset_mark_reports_line_zero() {
        let mut record = record();
        let fm = handle(&mut record);
        assert!(!fm.is_set());
        assert_eq!(fm.lnum(), 0);
        assert_eq!(
            (fm.pos().lnum, fm.pos().col),
            (UNSET_POS.lnum, UNSET_POS.col)
        );
    }

    #[test]
    fn writes_go_through_to_the_record() {
        let mut record = record();
        let fm = handle(&mut record);
        fm.set_pos(at(12, 3));
        fm.set_fnum(7);
        assert!(fm.is_set());
        assert_eq!((fm.lnum(), fm.col(), fm.fnum()), (12, 3, 7));
        assert_eq!(record.mark.lnum, 12);
        assert_eq!(record.fnum, 7);
    }

    #[test]
    fn setting_only_the_line_leaves_the_column_alone() {
        let mut record = record();
        let fm = handle(&mut record);
        fm.set_pos(at(12, 3));
        fm.set_lnum(40);
        assert_eq!((fm.lnum(), fm.col()), (40, 3));
    }

    /// What `clear_fmark` promises: the position goes, the timestamp stays as
    /// given, and the view is the "remember nothing" one.
    #[test]
    fn clearing_keeps_the_timestamp_it_is_given() {
        let mut record = record();
        let fm = handle(&mut record);
        fm.set_pos(at(12, 3));
        fm.set_fnum(7);
        fm.clear(4242);
        assert!(!fm.is_set());
        assert_eq!(fm.fnum(), 0);
        assert_eq!(fm.timestamp(), 4242);
        assert_eq!(fm.read().view.topline_offset, MAXLNUM.cast_signed());
    }

    #[test]
    fn the_position_address_names_the_position_inside_the_record() {
        let mut record = record();
        let fm = handle(&mut record);
        fm.set_pos(at(9, 2));
        // SAFETY: the handle names a live record, so its position is live.
        let seen = unsafe { *fm.pos_raw() };
        assert_eq!((seen.lnum, seen.col), (9, 2));
        // The claim is that the address is the record's own `mark` field, not
        // that the field sits at any particular offset: `fmark_T` has no
        // guaranteed layout, so `mark` need not come first.
        assert_eq!(fm.pos_raw(), &raw mut record.mark);
        assert!(fm.raw().cast::<u8>() <= fm.pos_raw().cast::<u8>());
    }

    #[test]
    fn an_xfmark_handle_reaches_both_halves() {
        let mut record = Box::new(UNSET_XFMARK);
        // SAFETY: the box outlives the handle.
        let xfm = unsafe { Xfmark::new(&raw mut *record) };
        xfm.fmark().set_pos(at(5, 1));
        assert_eq!(record.fmark.mark.lnum, 5);
        assert!(xfm.fname().is_null());
        assert_eq!(xfm.read().fmark.mark.col, 1);
    }
}
