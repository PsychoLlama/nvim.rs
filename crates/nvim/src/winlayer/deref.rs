//! Field access through a graph handle, and handle equality — both checked
//! against the slot the handle names.
//!
//! # The check
//!
//! A [`Win`], [`Buf`], [`TabPage`] or [`FrameRef`] holds its object's slot
//! address and the generation the slot had when the handle was made
//! ([`crate::slot`]). A slot's memory is never given back, so its generation
//! word is readable through any handle ever made, and every access here
//! reads it and compares: **one load, one compare**, and a
//! cold panic when the object the handle was made for has gone. That turns
//! the re-entry rule (`winlayer`'s docs) from a promise into a check: a
//! handle held across code that freed its object can no longer reach freed
//! memory or the slot's next tenant, it stops the editor instead, naming
//! the line that used it.
//!
//! Equality is the same question: two handles are equal when they name the
//! same slot **under the same generation**, so a handle that outlived its
//! window is not equal to the window that took its slot.
//!
//! The compare is **exact**: the slot's word carries the edit bit while an
//! [`edit`](TabPage::edit) of the object is out, and then matches no handle,
//! so the same one compare refuses a read or a write through any copy of a
//! handle while another copy's edit is held.
//!
//! # No borrow escapes unguarded
//!
//! These lend no `&mut` and no bare `&` at all. A field is read by value and
//! written in place through its getter and setter (`slot_fields!` below:
//! a place access through the raw address, which forms no reference to the
//! object), a compound change goes through [`TabPage::edit`] (the edit bit,
//! [`slot::edit_at`]), and a borrow of a field that is not `Copy` through
//! [`TabPage::read`], a guard the slot counts ([`slot::read_at`]): an edit
//! or a field write refuses while one is out. So a borrow held across code
//! that reaches the same object through another copy panics at that reach
//! instead of aliasing, in both directions.
#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::UnsafeCell;
use core::ffi::{c_char, c_int};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use crate::memline::MlFlags;
use crate::memline::state::MemLine;
use crate::slot::{self, SlotEdit, SlotRead};
use crate::types::{Buffer, ColNr, LineNr, OptInt, Pos, SynBlock, Tabpage, Window};
use crate::winlayer::{Buf, FrameId, FrameRef, PosRef, Stamp, TabId, TabPage, Win, WinId};

/// What each handle kind is called in a panic, by `checked_handles!`'s
/// index.
const HANDLE_KIND_NAMES: [&str; 4] = ["window", "buffer", "frame", "tab page"];

/// The panic a dereference through a refused handle ends in: the object is
/// gone (the re-entry rule, broken by the caller at the reported location),
/// or another copy's `edit()` of it is held.
#[cold]
#[inline(never)]
#[track_caller]
fn refused_handle(kind: u8, stamp: Stamp, word: u32) -> ! {
    // A byte rather than the `&str` itself: three scalar arguments keep the
    // call as small as the checks before it at every one of their sites.
    let kind = HANDLE_KIND_NAMES[usize::from(kind)];
    let id = stamp.handle().get();
    if word == stamp.generation() | slot::SLOT_EDIT_BIT {
        panic!(
            "a {kind} (id {id}) was reached while an edit() of it was held through another copy of \
             its handle: the edit lends it to nothing else until it is dropped"
        )
    }
    panic!(
        "a {kind} handle (id {id}) was used after its {kind} was freed: it was held across code \
         that freed it, and must be re-derived from its id instead (the re-entry rule)"
    )
}

macro_rules! checked_handles {
    ($($ty:ident => $target:ty, $kind:literal;)+) => { $(
        impl $ty {
            /// Whether the object this handle was made for is still in its
            /// slot, lent to an edit or not. Reads the slot's generation
            /// word, never the object. Not for a null handle.
            #[inline(always)]
            pub(crate) fn is_live(self) -> bool {
                // SAFETY: a handle's address is null or its slot's value
                // address (the constructors' promise), and a slot's memory is
                // never given back. A null handle's dereference was never
                // sound; it faults here as it did on the field access.
                let generation = unsafe { slot::generation_at(self.ptr) };
                generation == self.stamp.generation()
            }

            /// Panic unless the object this handle was made for is still in
            /// its slot and no edit of it is out: what every dereference
            /// asks first, in one compare of the slot's whole word.
            #[inline(always)]
            #[track_caller]
            fn check_live(&self) {
                // SAFETY: as `is_live`.
                let word = unsafe { slot::word_at(self.ptr) };
                if word != self.stamp.generation() {
                    refused_handle($kind, self.stamp, word);
                }
            }
        }

        /// The same slot under the same generation. `wp == curwin` stays an
        /// address test for every live handle; a stale one equals nothing
        /// but its own copies.
        impl PartialEq for $ty {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                ptr::eq(self.ptr, other.ptr)
                    && self.stamp.generation() == other.stamp.generation()
            }
        }

        impl Eq for $ty {}
    )+ };
}

checked_handles! {
    Win => Window, 0;
    Buf => Buffer, 1;
    FrameRef => Frame, 2;
    TabPage => Tabpage, 3;
}

/// `edit()` and `read()` for a handle whose object lends through guards
/// only.
macro_rules! guarded_handles {
    ($($ty:ident => $target:ty;)+) => { $(
        impl $ty {
            /// Lend the object mutably until the guard is dropped. While it
            /// is held, every other way to the object -- a getter, a setter,
            /// a read, another edit, through any copy of the handle --
            /// panics, so the guard's borrow is the only one.
            ///
            /// Scope it: in an `if let`/`match`/`while let`/`for` head or a
            /// call argument the guard lives to the end of the body or the
            /// statement. Its lifetime is not the handle's: the guard itself
            /// keeps the object in its slot (emptying a lent slot panics),
            /// so it may outlive the handle it came from.
            ///
            /// # Panics
            /// When the object is gone, already lent to an edit, or a
            /// `read()` of it is held.
            #[inline]
            #[track_caller]
            pub(crate) fn edit<'a>(self) -> SlotEdit<'a, $target> {
                // SAFETY: a live handle's address is its slot's value address
                // (the constructors' promise).
                unsafe { slot::edit_at(self.ptr, self.stamp.generation()) }
            }

            /// Borrow the object until the guard is dropped, for a field that
            /// is not `Copy`. Counted: an edit or a setter through any copy
            /// panics while it is held.
            ///
            /// # Panics
            /// When the object is gone or lent to an edit.
            #[inline]
            #[track_caller]
            pub(crate) fn read<'a>(self) -> SlotRead<'a, $target> {
                // SAFETY: as `edit`.
                unsafe { slot::read_at(self.ptr, self.stamp.generation()) }
            }
        }
    )+ };
}

guarded_handles! {
    TabPage => Tabpage;
    Buf => Buffer;
    Win => Window;
}

/// A getter and (where named) a setter per `Copy` field. The getter checks
/// the slot (one compare: live, not lent to an edit) and reads the field by
/// value; the setter also refuses while a `read()` is out, and stores in
/// place. Neither forms a reference to the object: a place access through
/// the raw address touches the field's bytes alone. A leaf of a nested
/// field names its path after `=` (`ml_line_count: LineNr = b_ml.ml_line_count`).
macro_rules! slot_fields {
    ($ty:ident => $target:ty {
        $( $(#[$doc:meta])* $field:ident $(, $set:ident)? : $fty:ty $(= $($path:ident).+)?; )*
    }) => {
        slot_fields!(pub(crate) $ty => $target { $( $(#[$doc])* $field $(, $set)? : $fty $(= $($path).+)?; )* });
    };
    ($vis:vis $ty:ident => $target:ty {
        $( $(#[$doc:meta])* $field:ident $(, $set:ident)? : $fty:ty $(= $($path:ident).+)?; )*
    }) => {
        impl $ty { $(
            slot_fields!(@get $vis [$(#[$doc])*] $field ($fty) [$($($path).+)?]);
            slot_fields!(@set $vis [$($set)?] $field ($fty) [$($($path).+)?]);
        )* }
    };
    (@get $vis:vis [$(#[$doc:meta])*] $field:ident ($fty:ty) [$($path:tt)*]) => {
        $(#[$doc])*
        #[inline(always)]
        #[track_caller]
        $vis fn $field(self) -> $fty {
            self.check_live();
            // SAFETY: live and lent to no edit (`check_live`); a place read
            // through the slot's address borrows nothing.
            unsafe { slot_fields!(@place (*self.ptr) $field $($path)*) }
        }
    };
    (@set $vis:vis [] $field:ident ($fty:ty) [$($path:tt)*]) => {};
    (@set $vis:vis [$set:ident] $field:ident ($fty:ty) [$($path:tt)*]) => {
        #[doc = concat!("Store `", stringify!($field), "` in place.")]
        #[inline(always)]
        #[track_caller]
        $vis fn $set(self, value: $fty) {
            // SAFETY: as `edit`: the slot's value address.
            unsafe { slot::check_write(self.ptr, self.stamp.generation()) };
            // SAFETY: live, lent to no edit and read by no guard
            // (`check_write`); a place write borrows nothing.
            unsafe { slot_fields!(@place (*self.ptr) $field $($path)*) = value };
        }
    };
    (@place ($base:expr) $field:ident) => { $base.$field };
    (@place ($base:expr) $field:ident $($path:tt)+) => { $base.$($path)+ };
}

mod buf_fields;
mod win_fields;

slot_fields! {
    FrameRef => Frame {
        fr_layout, set_fr_layout: c_char;
        fr_width, set_fr_width: c_int;
        fr_newwidth, set_fr_newwidth: c_int;
        fr_height, set_fr_height: c_int;
        fr_newheight, set_fr_newheight: c_int;
        fr_parent, set_fr_parent: Option<FrameId>;
        fr_next, set_fr_next: Option<FrameId>;
        fr_prev, set_fr_prev: Option<FrameId>;
        fr_child, set_fr_child: Option<FrameId>;
        fr_win, set_fr_win: Option<WinId>;
    }
}

slot_fields! {
    TabPage => Tabpage {
        tp_next, set_tp_next: Option<TabId>;
        tp_topframe, set_tp_topframe: Option<FrameId>;
        tp_curwin, set_tp_curwin: Option<WinId>;
        tp_prevwin, set_tp_prevwin: Option<WinId>;
        tp_firstwin, set_tp_firstwin: Option<WinId>;
        tp_lastwin, set_tp_lastwin: Option<WinId>;
        tp_old_rows_avail, set_tp_old_rows_avail: i64;
        tp_old_columns, set_tp_old_columns: i64;
        tp_ch_used, set_tp_ch_used: OptInt;
        tp_did_tabclosedpre, set_tp_did_tabclosedpre: bool;
        tp_diff_gen, set_tp_diff_gen: u32;
        tp_diffbuf: [Option<crate::winlayer::BufId>; 8];
        tp_diff_invalid, set_tp_diff_invalid: c_int;
        tp_diff_update, set_tp_diff_update: c_int;
        tp_snapshot: [Option<FrameId>; 3];
    }
}

/// A buffer's memline, lent for a walk ([`Buf::ml`]): one guard that
/// counts as a read of the buffer -- no edit and no free while it is held,
/// other reads welcome -- and carries the slot's part bit, so two walks of
/// one memline at once panic at the second, and so does a copy out of it
/// ([`Buf::line_count`] and its siblings) while a walk holds it.
pub(crate) type MlRef<'a> = slot::SlotPart<'a, Buffer, MemLine>;

/// The memline: [`Buf::ml`] for a walk, the line count and flags by value,
/// and the line text a memline cache holds, lent past the check that
/// reached it. The text is not the buffer's borrow but the line cache's
/// loan, whose bound is the reader's own -- [`crate::memline::Lines`]'
/// `&mut self` (the next read through it refills the cache) and
/// [`crate::memline::HeldLines`]' memfile pin (a write or free of a lent
/// block panics).
impl Buf {
    /// Lend the memline for a walk: [`MlRef`].
    ///
    /// # Panics
    /// When the buffer is gone or lent to an edit, or the memline is
    /// already lent.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn ml<'a>(self) -> MlRef<'a> {
        // SAFETY: `ptr` is the slot's value address (as `edit`), and the
        // memline is the buffer's one part in an `UnsafeCell`, reached only
        // through this loan, `ml_unlent`/`ml_unlent_mut` (which ask the part
        // bit and the reader count first) and an edit of the whole buffer.
        unsafe { slot::lend_part_at(self.ptr, self.stamp.generation(), self.ml_place()) }
    }

    /// The memline's place in the buffer's slot, computed without reading
    /// anything.
    #[inline(always)]
    fn ml_place(self) -> NonNull<MemLine> {
        // A place projection through the slot's address (in bounds of the
        // slot's memory, which is never given back) and the cell's own
        // address-to-content step: no read, no reference formed.
        let cell = self
            .ptr
            .wrapping_byte_add(core::mem::offset_of!(Buffer, b_ml));
        let place = UnsafeCell::raw_get(cell.cast::<UnsafeCell<MemLine>>());
        // SAFETY: an offset into a slot, which is never at address zero.
        unsafe { NonNull::new_unchecked(place) }
    }

    /// The memline, for a copy out of it while nothing holds it.
    #[inline(always)]
    #[track_caller]
    fn ml_unlent<'a>(self) -> &'a MemLine {
        // SAFETY: as `ml`.
        unsafe { slot::check_part_unlent(self.ptr, self.stamp.generation()) };
        // SAFETY: live, lent to no edit and no walk (just asked). The
        // reference lives as long as the caller's one read of a field (the
        // getters below) or the line cache's loan (the `lend_` family).
        unsafe { self.ml_place().as_ref() }
    }

    /// The memline, writable in place while nothing holds it.
    #[inline(always)]
    #[track_caller]
    fn ml_unlent_mut<'a>(self) -> &'a mut MemLine {
        // SAFETY: as `edit`: the slot's value address. No reader out means
        // no walk either (a walk counts as one).
        unsafe { slot::check_write(self.ptr, self.stamp.generation()) };
        // SAFETY: live, lent to no edit, no read guard and no walk (just
        // asked), and the reference lives as long as the caller's one write.
        unsafe { self.ml_place().as_mut() }
    }

    /// Whether the memline is open (has a memfile), asked without lending it.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn ml_is_open(self) -> bool {
        self.ml_unlent().is_open()
    }

    /// The buffer's line count: `b_ml.ml_line_count`.
    #[inline(always)]
    #[track_caller]
    pub fn line_count(self) -> LineNr {
        self.ml_unlent().ml_line_count
    }

    /// Store the line count in place.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn set_line_count(self, count: LineNr) {
        self.ml_unlent_mut().ml_line_count = count;
    }

    /// The memline's flags: `b_ml.ml_flags`.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn ml_flags(self) -> MlFlags {
        self.ml_unlent().ml_flags
    }

    /// Store the memline's flags in place.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn set_ml_flags(self, flags: MlFlags) {
        self.ml_unlent_mut().ml_flags = flags;
    }

    /// The memline a walk of it just filled the cache through, the walk let
    /// go: what [`Buf::ml_unlent`] answers, without asking again.
    ///
    /// # Panics
    /// When `walk` is another buffer's.
    #[inline(always)]
    #[track_caller]
    fn unlent_from<'a>(self, walk: MlRef<'_>) -> &'a MemLine {
        assert!(
            ptr::eq(walk.part_ptr(), self.ml_place().as_ptr()),
            "a walk of another buffer's memline"
        );
        drop(walk);
        // SAFETY: the walk was this memline's loan (just compared), which
        // `lend_part_at` granted live, unedited and lent to nothing else;
        // with it dropped nothing holds the memline. The reference lives as
        // the `lend_` family's does (the impl's docs).
        unsafe { self.ml_place().as_ref() }
    }

    /// [`Buf::lend_cached_line`] out of the walk that filled the cache
    /// (`fetch_line_lent`).
    #[inline(always)]
    #[track_caller]
    pub(crate) fn lend_cached_line_from<'a>(self, walk: MlRef<'_>) -> &'a [u8] {
        self.unlent_from(walk).cached_bytes()
    }

    /// The cached line with its NUL, as a block-pinned reader lends it, out
    /// of the walk that filled the cache.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn lend_pinned_line_from<'a>(self, walk: MlRef<'_>) -> &'a [u8] {
        self.unlent_from(walk).lend_cached_bytes_with_nul()
    }

    /// The cached line, without its NUL.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn lend_cached_line<'a>(self) -> &'a [u8] {
        self.ml_unlent().cached_bytes()
    }

    /// The cached line, with its NUL.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn lend_cached_line_with_nul<'a>(self) -> &'a [u8] {
        self.ml_unlent().cached_bytes_with_nul()
    }

    /// The cached line, writable in place (its length fixed).
    #[inline(always)]
    #[track_caller]
    pub(crate) fn lend_cached_line_mut<'a>(self) -> &'a mut [u8] {
        self.ml_unlent_mut().cached_bytes_mut()
    }
}

/// A syntax block lent through its owner ([`crate::syntax::SynBlockRef`]):
/// the owner was live and lent to no edit when it was made, and nothing
/// counts it after. The engine holds the pattern list across matches that
/// take a pattern's program out, so the block's aliasing stays open until a
/// program lives in a cell of its own (`docs/safe-surface.md`).
pub(crate) struct SynEdit<'a> {
    block: NonNull<SynBlock>,
    _lent: PhantomData<&'a mut SynBlock>,
}

impl Deref for SynEdit<'_> {
    type Target = SynBlock;

    #[inline(always)]
    fn deref(&self) -> &SynBlock {
        // SAFETY: the owner was live when the guard was made, and a slot's
        // memory is never given back; the aliasing is the open row above.
        unsafe { self.block.as_ref() }
    }
}

impl DerefMut for SynEdit<'_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut SynBlock {
        // SAFETY: as `deref`.
        unsafe { self.block.as_mut() }
    }
}

impl Buf {
    /// The buffer's own syntax block, `b_s`, as [`SynEdit`] lends it.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn syn_block_lent<'a>(self) -> SynEdit<'a> {
        self.check_live();
        SynEdit {
            block: NonNull::new(self.syntax_block()).expect("a live buffer's block"),
            _lent: PhantomData,
        }
    }
}

impl Win {
    /// The window's `:ownsyntax` block, as [`SynEdit`] lends it.
    ///
    /// # Panics
    /// When the window has none.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn own_syn_block_lent<'a>(self) -> SynEdit<'a> {
        self.check_live();
        // SAFETY: live and lent to no edit; a place read of the one field.
        let own = unsafe { (*self.ptr).w_own_syntax.as_deref() }
            .expect("an :ownsyntax block outlives its handles");
        SynEdit {
            block: NonNull::from(own),
            _lent: PhantomData,
        }
    }
}

/// The cursor of its window: read by value, written in place, each access
/// checked against the window's slot as the window's own are.
impl PosRef {
    /// The position, copied out.
    #[inline(always)]
    #[track_caller]
    pub fn get(self) -> Pos {
        self.win.check_live();
        // SAFETY: live and lent to no edit; a place read borrows nothing.
        unsafe { (*self.win.ptr).w_cursor }
    }

    /// Store the whole position.
    #[inline(always)]
    #[track_caller]
    pub fn set(self, pos: Pos) {
        // SAFETY: a live window handle's address is its slot's value address.
        unsafe { slot::check_write(self.win.ptr, self.win.stamp.generation()) };
        // SAFETY: live, lent to no edit and read by no guard; a place write
        // borrows nothing.
        unsafe { (*self.win.ptr).w_cursor = pos };
    }

    /// Run `f` over a copy of the position and store what it left: `f` may
    /// read the window (and its cursor) meanwhile, which a borrow into the
    /// window would forbid.
    #[inline(always)]
    #[track_caller]
    pub fn modify<R>(self, f: impl FnOnce(&mut Pos) -> R) -> R {
        let mut pos = self.get();
        let r = f(&mut pos);
        self.set(pos);
        r
    }

    #[inline(always)]
    #[track_caller]
    pub fn lnum(self) -> LineNr {
        self.get().lnum
    }

    #[inline(always)]
    #[track_caller]
    pub fn col(self) -> ColNr {
        self.get().col
    }

    #[inline(always)]
    #[track_caller]
    pub fn coladd(self) -> i32 {
        self.get().coladd
    }

    #[inline(always)]
    #[track_caller]
    pub fn set_lnum(self, lnum: LineNr) {
        // SAFETY: as `set`.
        unsafe { slot::check_write(self.win.ptr, self.win.stamp.generation()) };
        // SAFETY: as `set`.
        unsafe { (*self.win.ptr).w_cursor.lnum = lnum };
    }

    #[inline(always)]
    #[track_caller]
    pub fn set_col(self, col: ColNr) {
        // SAFETY: as `set`.
        unsafe { slot::check_write(self.win.ptr, self.win.stamp.generation()) };
        // SAFETY: as `set`.
        unsafe { (*self.win.ptr).w_cursor.col = col };
    }

    #[inline(always)]
    #[track_caller]
    pub fn set_coladd(self, coladd: i32) {
        // SAFETY: as `set`.
        unsafe { slot::check_write(self.win.ptr, self.win.stamp.generation()) };
        // SAFETY: as `set`.
        unsafe { (*self.win.ptr).w_cursor.coladd = coladd };
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crate::buffer::{bare_buffer, free_bare_buffer};
    use crate::global_cell::editor_state_lock;
    use crate::window::{alloc_tabpage, bare_window, free_bare_window, free_tab};

    /// A live handle reads and writes its object; once the object is freed
    /// every copy reports it gone and a dereference panics -- even after the
    /// slot holds a new tenant, which the old copy does not equal.
    #[test]
    fn a_freed_window_stops_its_handles() {
        let _held = editor_state_lock();
        let win = bare_window();
        let copy = win;
        win.set_w_topline(crate::types::LineNr::new(3));
        assert_eq!(copy.w_topline(), crate::types::LineNr::new(3));
        assert!(copy.is_live());
        free_bare_window(win);
        assert!(!copy.is_live());
        let caught = catch_unwind(AssertUnwindSafe(|| copy.w_topline()));
        assert!(caught.is_err(), "a stale window handle dereferenced");
        let tenant = bare_window();
        if tenant.raw() == copy.raw() {
            assert_ne!(tenant, copy, "a new tenant is not the old window");
        }
        free_bare_window(tenant);
    }

    /// A tab page lent to an `edit()` refuses every other copy -- getter,
    /// setter, read -- until the guard goes; a `read()` refuses edits and
    /// setters but not getters.
    #[test]
    fn a_tab_page_edit_and_read_refuse_their_writers() {
        let _held = editor_state_lock();
        let tab = alloc_tabpage();
        let copy = tab;
        {
            let mut edit = tab.edit();
            edit.tp_ch_used = 3;
            let refused = catch_unwind(AssertUnwindSafe(|| copy.tp_ch_used()))
                .expect_err("a getter under an edit");
            let message = refused.downcast_ref::<String>().map_or("", String::as_str);
            assert!(message.contains("edit()"), "{message}");
            assert!(catch_unwind(AssertUnwindSafe(|| copy.set_tp_ch_used(1))).is_err());
            assert!(catch_unwind(AssertUnwindSafe(|| drop(copy.read()))).is_err());
            assert!(copy.is_live(), "lent out, not gone");
        }
        assert_eq!(copy.tp_ch_used(), 3);
        let read = copy.read();
        assert!(catch_unwind(AssertUnwindSafe(|| drop(tab.edit()))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| tab.set_tp_ch_used(1))).is_err());
        assert_eq!(tab.tp_ch_used(), 3);
        drop(read);
        tab.set_tp_ch_used(4);
        assert_eq!(copy.tp_ch_used(), 4);
        free_tab(tab);
    }

    /// A buffer handle's mutable dereference checks the slot as well.
    #[test]
    fn a_freed_buffer_refuses_a_write() {
        let _held = editor_state_lock();
        let buffer = bare_buffer();
        let copy = buffer;
        copy.set_b_changed(1);
        assert_eq!(buffer.b_changed(), 1);
        free_bare_buffer(buffer);
        let caught = catch_unwind(AssertUnwindSafe(move || copy.edit().b_changed = 0));
        assert!(caught.is_err(), "a stale buffer handle written through");
    }
}
