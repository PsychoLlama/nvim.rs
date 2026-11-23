//! Type-stable slots: values at addresses that stay a `T`'s for the rest of
//! the run, each carrying a generation that says whether the value a handle
//! was given is still the one there.
//!
//! A [`SlotPool<T>`] allocates each [`Slot`] once and **never gives its
//! memory back to the allocator**. Emptying a slot ends its value (moves it
//! out or drops it in place) and moves the slot's generation on; the slot
//! goes on the pool's vacant list for the next value of the same type. So a
//! [`SlotRef<T>`] -- the slot's address plus the generation it was handed --
//! may outlive its value, but never its memory: reading the slot's header
//! through any handle ever made is a read of live memory, and that one read
//! and one compare is the whole liveness check. A stale handle panics at its
//! first dereference instead of reading freed memory or the next tenant.
//!
//! # The generation word
//!
//! One `u32` at the head of the slot, beside the value:
//!
//! - **odd** while the slot holds a value, **even** while it is vacant, so a
//!   vacant slot matches no handle whatever generation it has reached;
//! - the **top bit is the edit bit**: [`edit_at`] sets it for as long as
//!   its [`SlotEdit`] lends the value mutably, so a handle's one exact
//!   compare answers "live *and* not lent out" -- a read, a write or a
//!   second edit through any other copy refuses while an edit is held. A
//!   generation therefore runs 1, 3, …, `GENERATION_LIMIT`;
//! - a slot whose generation reaches `GENERATION_LIMIT` **retires** when it
//!   is emptied: its word goes to zero and it is never handed out again, so
//!   no handle can ever see a generation come round a second time.
//!
//! # Reads and edits
//!
//! Beside the generation, a slot counts the [`SlotRead`]s of its value that
//! are out ([`read_at`]). An edit, a field write ([`check_write`]) and
//! emptying the slot all refuse while the count is not zero, so a shared
//! borrow handed out by a read guard cannot be written under or freed; an
//! edit in turn refuses every read. A value may also lend **one interior
//! part** (a buffer's memline, in an `UnsafeCell` of its own) mutably while
//! the rest of it is read ([`lend_part_at`]): the loan counts as a reader
//! and sets [`SLOT_PART_BIT`] in the count, so a second loan of the part
//! refuses. The handles that lend through these
//! guards alone (`winlayer`'s `TabPage` and `FrameRef`) are therefore
//! sound however their copies are used: the worst a misuse does is panic.
//!
//! # Every borrow is a guard
//!
//! A [`SlotRef`] is `Copy`, so it lends nothing by itself: its value is
//! borrowed only through [`SlotRef::read`] and [`SlotRef::edit`], the
//! counted guards above, and a borrow held across code that reaches the
//! same value through another copy panics there instead of aliasing. The
//! one uncounted borrow is a pool's own ([`SlotPool::get_mut`], through
//! `&mut` of the pool), for a table that hands out no handles at all.
//!
//! # Pools live in statics
//!
//! A pool has no destructor that frees its slots: dropping one leaks them
//! (and any values still in them), which is what keeps a handle that
//! outlived its pool pointing at live memory. Every pool in the editor is
//! a field of a `static`.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `slot.rs`, `slot/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::mem::{ManuallyDrop, MaybeUninit, offset_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use crate::global_cell::GlobalCell;

mod guards;
pub(crate) mod id_table;

use guards::refused;
pub(crate) use guards::{
    EditPart, ReadPart, SlotEdit, SlotPart, SlotRead, check_part_unlent, check_write, edit_at,
    lend_part_at, read_at,
};

/// The highest generation a slot is handed out under; emptying a slot that
/// reached it retires the slot.
const GENERATION_LIMIT: u32 = 0x7FFF_FFFF;

/// The generation word of a retired slot: even, so it matches no handle, and
/// never on the vacant list.
const RETIRED_GENERATION: u32 = 0;

/// The bit an edit sets in its slot's generation word while it lends the
/// value mutably. No generation reaches it (`GENERATION_LIMIT`).
pub(crate) const SLOT_EDIT_BIT: u32 = 0x8000_0000;

/// One value's place: the generation word, the slot's index in its pool,
/// the count of read guards out, and the value, initialised exactly while
/// the generation is odd.
///
/// `repr(C)` so the header comes first: a handle reads the generation at
/// the slot's own address.
#[repr(C)]
struct Slot<T> {
    generation: Cell<u32>,
    /// Where the pool files this slot; fixed for the slot's life.
    index: u32,
    /// How many [`SlotRead`]s of the value are out.
    readers: Cell<u32>,
    value: UnsafeCell<MaybeUninit<T>>,
}

/// The generation word of the slot at `slot`.
#[inline(always)]
fn header<T>(slot: NonNull<Slot<T>>) -> &'static Cell<u32> {
    // SAFETY: a slot's memory is never given back (module docs), so its
    // header is a live `Cell<u32>` for the rest of the run, whatever became
    // of the value. The header is first in a `repr(C)` struct, and a
    // `Cell` is interior-mutable, so this borrow claims nothing of the
    // value beside it.
    unsafe { &*slot.as_ptr().cast::<Cell<u32>>() }
}

/// The value's address in the slot at `slot`, computed without reading it.
#[inline(always)]
fn value_ptr<T>(slot: NonNull<Slot<T>>) -> *mut T {
    // The slot's address plus a constant: no dereference, and the slot's
    // provenance carried over (a `MaybeUninit<T>` in an `UnsafeCell` has
    // `T`'s layout).
    slot.as_ptr()
        .wrapping_byte_add(offset_of!(Slot<T>, value))
        .cast()
}

/// The slot's index, which never changes.
#[inline(always)]
fn index_of<T>(slot: NonNull<Slot<T>>) -> u32 {
    // SAFETY: as `header`: the slot's memory lives for the rest of the run,
    // and `index` is written once, before the slot is shared.
    unsafe {
        slot.as_ptr()
            .wrapping_byte_add(offset_of!(Slot<T>, index))
            .cast::<u32>()
            .read()
    }
}

/// The count of read guards out on the value in the slot at `slot`.
#[inline(always)]
fn readers<T>(slot: NonNull<Slot<T>>) -> &'static Cell<u32> {
    // SAFETY: as `header`: the slot's memory lives for the rest of the run,
    // and the count is a `Cell` beside the value, not part of it.
    unsafe {
        &*slot
            .as_ptr()
            .wrapping_byte_add(offset_of!(Slot<T>, readers))
            .cast::<Cell<u32>>()
    }
}

/// The slot whose value is at `value`, computed without reading anything.
///
/// # Safety
/// As [`generation_at`].
#[inline(always)]
unsafe fn slot_of<T>(value: *const T) -> NonNull<Slot<T>> {
    let slot = value
        .wrapping_byte_sub(offset_of!(Slot<T>, value))
        .cast::<Slot<T>>()
        .cast_mut();
    // SAFETY: the caller's promise makes it a slot's address (carrying the
    // slot's provenance, which `value_ptr` handed on), and that is not null.
    unsafe { NonNull::new_unchecked(slot) }
}

/// The generation the slot whose value is at `value` holds now, whether or
/// not an edit is out: what a handle made from a bare address takes, and
/// what "is this value still there?" compares.
///
/// For the constructors that are handed a bare value address (`Win::new`
/// and its siblings): what they promise is a live value, and a live value's
/// slot holds the generation it was handed out under.
///
/// # Safety
/// `value` must be the address of a slot's value -- [`SlotRef::raw`] of a
/// handle of some `SlotPool<T>`, or derived from it with its provenance --
/// and nothing else.
#[inline(always)]
pub(crate) unsafe fn generation_at<T>(value: *const T) -> u32 {
    // SAFETY: the caller's promise.
    unsafe { word_at(value) & !SLOT_EDIT_BIT }
}

/// The whole generation word of the slot whose value is at `value`: the
/// generation, with the edit bit while an edit is out. A dereference
/// compares this with its handle's generation *exactly*, so one compare
/// refuses both a value that is gone and one that is lent out.
///
/// # Safety
/// As [`generation_at`].
#[inline(always)]
pub(crate) unsafe fn word_at<T>(value: *const T) -> u32 {
    // SAFETY: the caller's promise makes it a slot's address; a slot's
    // memory is never given back.
    header(unsafe { slot_of(value) }).get()
}

#[cold]
#[inline(never)]
#[track_caller]
fn stale_slot(type_name: &'static str) -> ! {
    panic!("a {type_name} handle outlived its value")
}

/// A handle to the value a pool put in one slot: the slot's address and the
/// generation it was handed under. Every dereference checks the slot still
/// has that generation, and panics if not.
pub(crate) struct SlotRef<T> {
    slot: NonNull<Slot<T>>,
    generation: u32,
}

// Hand-written rather than derived: `derive` would demand the same of `T`.
impl<T> Clone for SlotRef<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SlotRef<T> {}

/// Two handles are equal when they name the same value: the same slot under
/// the same generation.
impl<T> PartialEq for SlotRef<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot && self.generation == other.generation
    }
}

impl<T> Eq for SlotRef<T> {}

impl<T> fmt::Debug for SlotRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SlotRef({}#{} @ {:p})",
            index_of(self.slot),
            self.generation,
            self.slot
        )
    }
}

impl<T> SlotRef<T> {
    /// Whether the value this handle was given is still in its slot (lent
    /// to an edit or not).
    #[inline(always)]
    pub(crate) fn is_live(self) -> bool {
        header(self.slot).get() & !SLOT_EDIT_BIT == self.generation
    }

    /// The slot's index in its pool.
    pub(crate) fn index(self) -> u32 {
        index_of(self.slot)
    }

    /// The generation the value was handed out under: odd, and below
    /// `0x8000_0000`.
    pub(crate) fn generation(self) -> u32 {
        self.generation
    }

    /// The value's address, for the callees that still take one. Reads
    /// nothing; the address stays a `T`'s slot for the rest of the run, but
    /// only [`SlotRef::is_live`] says whether the value there is this one.
    #[inline(always)]
    pub(crate) fn raw(self) -> *mut T {
        value_ptr(self.slot)
    }

    /// The handle to the slot whose value is at `value`, under
    /// `generation`: the parts [`SlotRef::raw`] and
    /// [`SlotRef::generation`] took apart, put back together. Reads
    /// nothing; whether the value is still there is [`SlotRef::is_live`]'s
    /// question, as for any handle.
    ///
    /// # Safety
    /// `value` must be [`SlotRef::raw`] of a handle of some `SlotPool<T>`
    /// (or derived from it with its provenance).
    #[inline(always)]
    pub(crate) unsafe fn from_parts(value: *mut T, generation: u32) -> Self {
        let slot = value
            .wrapping_byte_sub(offset_of!(Slot<T>, value))
            .cast::<Slot<T>>();
        SlotRef {
            // SAFETY: the caller's promise makes it a slot's address, which
            // is never null.
            slot: unsafe { NonNull::new_unchecked(slot) },
            generation,
        }
    }

    /// Panic unless the value is still there and not lent to an edit.
    #[inline(always)]
    #[track_caller]
    fn check(self) {
        let word = header(self.slot).get();
        if word != self.generation {
            refused::<T>(word, self.generation, readers(self.slot).get());
        }
    }
}

impl<T> SlotRef<T> {
    /// Borrow the value until the guard is dropped, counted: an edit, a
    /// field write or emptying the slot through any copy panics while it is
    /// held.
    ///
    /// # Panics
    /// When the value is gone or lent to an edit.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn read<'a>(self) -> SlotRead<'a, T> {
        // SAFETY: a handle's address is its slot's value address.
        unsafe { read_at(self.raw(), self.generation) }
    }

    /// Lend the value mutably until the guard is dropped: every other way
    /// to it, through any copy, panics meanwhile.
    ///
    /// # Panics
    /// When the value is gone, lent to an edit, or a read of it is held.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn edit<'a>(self) -> SlotEdit<'a, T> {
        // SAFETY: as `read`.
        unsafe { edit_at(self.raw(), self.generation) }
    }
}

/// A slot taken out of its pool with its value still in it: named by no
/// handle (its generation moved on as if emptied), owned by this alone, and
/// put back under a fresh generation by [`SlotPool::attach`] -- for a value
/// that is cheaper to clear and reuse than to build again.
///
/// Dropping one drops the value and loses the slot (it never returns to the
/// vacant list); [`SlotPool::release`] is the way to give both back.
pub(crate) struct Detached<T> {
    slot: NonNull<Slot<T>>,
}

impl<T> Deref for Detached<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: a detached slot's value is initialised and reachable only
        // through this value, which is not `Copy`.
        unsafe { &*value_ptr(self.slot) }
    }
}

impl<T> DerefMut for Detached<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: as `deref`; `&mut self` is the one way in.
        unsafe { &mut *value_ptr(self.slot) }
    }
}

impl<T> Drop for Detached<T> {
    fn drop(&mut self) {
        // SAFETY: the value is initialised and owned by this alone (`deref`);
        // nothing reads it after, as no handle names the slot and it is on
        // no vacant list.
        unsafe { value_ptr(self.slot).drop_in_place() }
    }
}

impl<T> Detached<T> {
    /// Drop the value where it is, answering the slot, now vacant and named
    /// by nothing, for [`SlotPool::reclaim`].
    fn empty(self) -> NonNull<Slot<T>> {
        let slot = ManuallyDrop::new(self).slot;
        // SAFETY: as `Drop`: initialised, owned by this alone, and read by
        // nothing after (the generation already moved on in `detach`).
        unsafe { value_ptr(slot).drop_in_place() };
        slot
    }
}

impl<T> fmt::Debug for Detached<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Detached({} @ {:p})", index_of(self.slot), self.slot)
    }
}

/// Slots for values of type `T`, allocated once each and reused, never
/// freed. See the module docs.
pub(crate) struct SlotPool<T> {
    /// Every slot the pool ever allocated, by index.
    slots: Vec<NonNull<Slot<T>>>,
    /// The vacant ones, reused last-emptied first.
    vacant: Vec<u32>,
}

impl<T> fmt::Debug for SlotPool<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlotPool")
            .field("slots", &self.slots.len())
            .field("vacant", &self.vacant.len())
            .finish()
    }
}

impl<T> SlotPool<T> {
    /// An empty pool.
    pub(crate) const fn new() -> Self {
        SlotPool {
            slots: Vec::new(),
            vacant: Vec::new(),
        }
    }

    /// A vacant slot: a reused one, or a new one.
    fn take_vacant(&mut self) -> NonNull<Slot<T>> {
        if let Some(index) = self.vacant.pop() {
            return self.slots[index as usize];
        }
        let index = u32::try_from(self.slots.len()).expect("fewer than 2^32 slots");
        let slot = NonNull::from(Box::leak(Box::new(Slot {
            generation: Cell::new(0),
            index,
            readers: Cell::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        })));
        self.slots.push(slot);
        slot
    }

    /// Hand the vacant `slot`, its value just initialised, out under its
    /// next generation.
    fn occupy(slot: NonNull<Slot<T>>) -> SlotRef<T> {
        let word = header(slot);
        debug_assert!(
            word.get().is_multiple_of(2),
            "only a vacant slot is occupied"
        );
        let generation = word.get() + 1;
        word.set(generation);
        SlotRef { slot, generation }
    }

    /// Move `slot`'s generation past its value's, retiring it at the limit.
    /// Answers whether it may be reused.
    fn vacate(slot: NonNull<Slot<T>>) -> bool {
        let word = header(slot);
        if word.get() == GENERATION_LIMIT {
            word.set(RETIRED_GENERATION);
            false
        } else {
            word.set(word.get() + 1);
            true
        }
    }

    /// Check `r` names a live value of *this* pool, lent to no edit.
    #[track_caller]
    fn own(&self, r: SlotRef<T>) {
        r.check();
        assert!(
            self.slots.get(index_of(r.slot) as usize) == Some(&r.slot),
            "a slot handle from another pool"
        );
    }

    /// [`SlotPool::own`], and no read guard out either: what emptying the
    /// slot asks, so a guard's borrow never outlives its value.
    #[track_caller]
    fn own_unread(&self, r: SlotRef<T>) {
        self.own(r);
        let count = readers(r.slot).get();
        if count != 0 {
            refused::<T>(r.generation, r.generation, count);
        }
    }

    /// Take `value` into a slot, answering its handle.
    pub(crate) fn insert(&mut self, value: T) -> SlotRef<T> {
        let slot = self.take_vacant();
        // SAFETY: the slot is vacant, so no handle reaches its value and
        // nothing is there to drop.
        unsafe { value_ptr(slot).write(value) };
        Self::occupy(slot)
    }

    /// Take a value built **in its slot** by `init`, answering its handle:
    /// for values large enough that building one and moving it in would be a
    /// copy of all of it. `init` is handed the vacant place (whatever bytes a
    /// previous tenant left there) and answers it initialised -- the
    /// reference `MaybeUninit::write` or `assume_init_mut` gives back.
    ///
    /// # Panics
    /// When `init` answers any other place than the one it was handed.
    pub(crate) fn insert_with(
        &mut self,
        init: impl FnOnce(&mut MaybeUninit<T>) -> &mut T,
    ) -> SlotRef<T> {
        let slot = self.take_vacant();
        // SAFETY: the slot is vacant, so no handle reaches its value; this
        // borrow is the only one, and it ends with `init`.
        let place = unsafe { &mut *value_ptr(slot).cast::<MaybeUninit<T>>() };
        let at = place.as_mut_ptr();
        let filled: *mut T = init(place);
        // `init` can only answer this place initialised by writing it (or by
        // an `unsafe` claim of its own); any other answer leaves the slot
        // off the vacant list and handed to no one.
        assert!(
            ptr::eq(filled, at),
            "a slot initialiser answered another place"
        );
        Self::occupy(slot)
    }

    /// The handle to the value in slot `index` under `generation`, `None`
    /// when that value is gone (or never was).
    pub(crate) fn lookup(&self, index: u32, generation: u32) -> Option<SlotRef<T>> {
        let r = SlotRef {
            slot: *self.slots.get(index as usize)?,
            generation,
        };
        r.is_live().then_some(r)
    }

    /// Empty `r`'s slot, answering its value. Every handle to it goes stale.
    ///
    /// # Panics
    /// When `r` is stale, or another pool's.
    #[track_caller]
    pub(crate) fn remove(&mut self, r: SlotRef<T>) -> T {
        self.own_unread(r);
        // SAFETY: the value is live (`own`), and the generation moves on
        // below before anything else can run, so nothing reads it again.
        let value = unsafe { value_ptr(r.slot).read() };
        self.vacated(r.slot);
        value
    }

    /// Empty `r`'s slot without dropping its value, whose owner has given
    /// back by hand what it held. Every handle to it goes stale.
    ///
    /// # Panics
    /// When `r` is stale, or another pool's.
    #[track_caller]
    pub(crate) fn forget(&mut self, r: SlotRef<T>) {
        self.own_unread(r);
        self.vacated(r.slot);
    }

    /// Take `r`'s slot out with its value still in it. Every handle to it
    /// goes stale.
    ///
    /// # Panics
    /// When `r` is stale, or another pool's.
    #[track_caller]
    pub(crate) fn detach(&mut self, r: SlotRef<T>) -> Detached<T> {
        self.own_unread(r);
        // Vacant-shaped (or retired) so no handle matches, but kept off the
        // vacant list: its value is still there, owned by the `Detached`.
        Self::vacate(r.slot);
        Detached { slot: r.slot }
    }

    /// Put a detached slot back, its value under a fresh generation.
    ///
    /// # Panics
    /// When `d` is another pool's.
    pub(crate) fn attach(&mut self, d: Detached<T>) -> SlotRef<T> {
        let slot = ManuallyDrop::new(d).slot;
        assert!(
            self.slots.get(index_of(slot) as usize) == Some(&slot),
            "a detached slot from another pool"
        );
        if header(slot).get() == RETIRED_GENERATION {
            // It reached the limit: its value moves to another slot, and it
            // stays retired.
            // SAFETY: a detached slot's value is initialised and owned by
            // the `Detached` just taken apart, so this is its one read.
            return self.insert(unsafe { value_ptr(slot).read() });
        }
        Self::occupy(slot)
    }

    /// Give a detached slot back to the vacant list, answering its value.
    ///
    /// # Panics
    /// When `d` is another pool's.
    pub(crate) fn release(&mut self, d: Detached<T>) -> T {
        let slot = ManuallyDrop::new(d).slot;
        assert!(
            self.slots.get(index_of(slot) as usize) == Some(&slot),
            "a detached slot from another pool"
        );
        // SAFETY: as in `attach`: initialised, and this is its one read.
        let value = unsafe { value_ptr(slot).read() };
        if header(slot).get() != RETIRED_GENERATION {
            self.vacant.push(index_of(slot));
        }
        value
    }

    /// The value `r` names, borrowed for as long as the pool is.
    ///
    /// # Panics
    /// When `r` is stale, or another pool's.
    #[track_caller]
    fn get_mut(&mut self, r: SlotRef<T>) -> &mut T {
        self.own(r);
        // SAFETY: live and lent to no edit (`own`); the borrow is the
        // pool's, and emptying the slot needs the pool too. Not counted (the
        // condition stacks take this path per command line), and it need not
        // be: private to this module and `id_table`, whose `IdTable` -- the
        // one caller -- never lets a handle of its values out (`ViewTable`,
        // which does, has no `get_mut`), so no guard of the value can start
        // while this lives.
        unsafe { &mut *value_ptr(r.slot) }
    }

    /// Forget every guard out on this pool's values -- edits, reads, part
    /// loans -- as if each had been dropped.
    ///
    /// Only for the emergency exits (the panic hook, a failed allocation),
    /// which end the process: the frames that hold those guards never
    /// resume, and the swap-file cleanup must reach every buffer whatever
    /// its owner held when the process stopped.
    pub(crate) fn abandon_loans(&self) {
        for &slot in &self.slots {
            let word = header(slot);
            word.set(word.get() & !SLOT_EDIT_BIT);
            readers(slot).set(0);
        }
    }

    /// `slot`, its value just ended, onto the vacant list (or retired).
    fn vacated(&mut self, slot: NonNull<Slot<T>>) {
        if Self::vacate(slot) {
            self.vacant.push(index_of(slot));
        }
    }

    /// A slot [`Detached::empty`] emptied, back onto the vacant list unless
    /// it retired.
    fn reclaim(&mut self, slot: NonNull<Slot<T>>) {
        assert!(
            self.slots.get(index_of(slot) as usize) == Some(&slot),
            "a detached slot from another pool"
        );
        if header(slot).get() != RETIRED_GENERATION {
            self.vacant.push(index_of(slot));
        }
    }

    /// Set `r`'s slot's generation, for the retirement tests.
    #[cfg(test)]
    fn rewind(r: &mut SlotRef<T>, generation: u32) {
        header(r.slot).set(generation);
        r.generation = generation;
    }
}

/// A type whose values live in one pool, a static: what lets a
/// [`SlotBox`] give its value back without being handed the pool.
pub(crate) trait Pooled: Sized + 'static {
    /// The pool every value of this type lives in.
    fn pool() -> &'static GlobalCell<SlotPool<Self>>;
}

/// The owner of one value in its type's pool: `Box`'s shape, with the value
/// in a type-stable slot. Dropping it ends the value **in place** -- its
/// handles go stale first, then its destructor runs, outside any borrow of
/// the pool, then the slot is vacant for the next value.
///
/// Not `Copy`, and the one owner: [`SlotBox::handle`] hands out the
/// `Copy` handles everything else works through.
pub(crate) struct SlotBox<T: Pooled> {
    slot: SlotRef<T>,
}

impl<T: Pooled> SlotBox<T> {
    /// `value`, moved into a slot of its pool.
    pub(crate) fn new(value: T) -> Self {
        SlotBox {
            slot: T::pool().with_mut(|pool| pool.insert(value)),
        }
    }

    /// A value built in its slot. [`SlotPool::insert_with`], panic
    /// included; `init` runs with the pool borrowed, so it must not reach
    /// the pool itself.
    pub(crate) fn new_with(init: impl FnOnce(&mut MaybeUninit<T>) -> &mut T) -> Self {
        SlotBox {
            slot: T::pool().with_mut(|pool| pool.insert_with(init)),
        }
    }

    /// The handle to the value, good for as long as this owner is kept.
    #[inline(always)]
    pub(crate) fn handle(&self) -> SlotRef<T> {
        self.slot
    }
}

impl<T: Pooled> SlotBox<T> {
    /// Give the value up **now** while keeping its memory for later: every
    /// handle to it goes stale at once, and the value stays in its slot,
    /// owned by what this answers, until that is dropped. For an owner that
    /// has torn its value down but whose memory something may still be
    /// standing on (a buffer freed while an autocommand runs).
    pub(crate) fn park(self) -> Parked<T> {
        let slot = ManuallyDrop::new(self).slot;
        Parked::of(slot)
    }
}

impl<T: Pooled> Drop for SlotBox<T> {
    fn drop(&mut self) {
        drop(Parked::of(self.slot));
    }
}

impl<T: Pooled> fmt::Debug for SlotBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SlotBox({:?})", self.slot)
    }
}

/// A value its owner gave up whose slot is not yet vacant: named by no
/// handle (its generation moved on when it was parked), still initialised,
/// and off the vacant list, so the slot cannot take a new tenant while
/// something may still be standing on the old one's memory. Dropping it
/// ends the value in place and frees the slot for the next value;
/// [`Parked::abandon`] frees the slot without the destructor.
pub(crate) struct Parked<T: Pooled> {
    slot: NonNull<Slot<T>>,
}

impl<T: Pooled> Parked<T> {
    /// Park the live value `r` names: every handle to it goes stale.
    ///
    /// # Panics
    /// When `r` is stale.
    #[track_caller]
    pub(crate) fn of(r: SlotRef<T>) -> Self {
        let detached = T::pool().with_mut(|pool| pool.detach(r));
        Parked {
            slot: ManuallyDrop::new(detached).slot,
        }
    }

    /// Free the slot for the next value **without** running the value's
    /// destructor: for an owner that released what the value held by hand
    /// (a window, whose block upstream freed with a bare `xfree`).
    pub(crate) fn abandon(self) {
        let slot = ManuallyDrop::new(self).slot;
        T::pool().with_mut(|pool| pool.reclaim(slot));
    }
}

impl<T: Pooled> Drop for Parked<T> {
    fn drop(&mut self) {
        // The destructor runs with the pool let go: it may free values of
        // its own type.
        let slot = Detached { slot: self.slot }.empty();
        T::pool().with_mut(|pool| pool.reclaim(slot));
    }
}

impl<T: Pooled> fmt::Debug for Parked<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parked({} @ {:p})", index_of(self.slot), self.slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pools live in statics, as the editor's do: a pool leaks its slots on
    // purpose, and Miri counts what a static reaches as live.

    #[test]
    fn a_handle_reads_its_value_until_the_slot_is_emptied() {
        static POOL: GlobalCell<SlotPool<Vec<u32>>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| pool.insert(vec![1]));
        let b = POOL.with_mut(|pool| pool.insert(vec![2]));
        let copy = a;
        a.edit().push(10);
        assert_eq!(*copy.read(), [1, 10]);
        assert_eq!(*b.read(), [2]);
        assert!(a.is_live());
        assert_eq!(POOL.with_mut(|pool| pool.remove(a)), [1, 10]);
        assert!(!copy.is_live());
        // The slot is reused, under a new generation: the old handle stays
        // stale, the new one is not equal to it.
        let c = POOL.with_mut(|pool| pool.insert(vec![3]));
        assert_eq!(c.raw(), copy.raw());
        assert_ne!(c, copy);
        assert!(!copy.is_live());
        assert_eq!(*c.read(), [3]);
        POOL.with_mut(|pool| {
            drop(pool.remove(b));
            drop(pool.remove(c));
        });
    }

    #[test]
    #[should_panic(expected = "handle outlived its value")]
    fn a_stale_handle_panics() {
        static POOL: GlobalCell<SlotPool<String>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| pool.insert("gone".to_owned()));
        drop(POOL.with_mut(|pool| pool.remove(a)));
        let _ = a.read().len();
    }

    #[test]
    fn values_stay_put_while_the_pool_grows() {
        static POOL: GlobalCell<SlotPool<[u8; 64]>> = GlobalCell::new(SlotPool::new());
        let first = POOL.with_mut(|pool| pool.insert([7; 64]));
        let at = first.raw();
        let rest: Vec<_> = (0..100)
            .map(|n| POOL.with_mut(|pool| pool.insert([n; 64])))
            .collect();
        assert_eq!(first.raw(), at);
        assert_eq!(first.read()[63], 7);
        POOL.with_mut(|pool| {
            pool.forget(first);
            for r in rest {
                assert_eq!(pool.lookup(index_of(r.slot), r.generation), Some(r));
                pool.forget(r);
                assert_eq!(pool.lookup(index_of(r.slot), r.generation), None);
            }
        });
    }

    #[test]
    fn a_detached_value_comes_back_under_a_new_generation() {
        static POOL: GlobalCell<SlotPool<Vec<u32>>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| pool.insert(vec![1, 2]));
        let mut d = POOL.with_mut(|pool| pool.detach(a));
        assert!(!a.is_live());
        d.clear();
        d.push(5);
        let b = POOL.with_mut(|pool| pool.attach(d));
        assert_eq!(b.raw(), a.raw());
        assert_ne!(a, b);
        assert_eq!(*b.read(), [5]);
        let d = POOL.with_mut(|pool| pool.detach(b));
        assert_eq!(POOL.with_mut(|pool| pool.release(d)), [5]);
        // Dropping a detached slot drops its value (and loses the slot).
        let c = POOL.with_mut(|pool| pool.insert(vec![9]));
        drop(POOL.with_mut(|pool| pool.detach(c)));
        assert!(!c.is_live());
    }

    #[test]
    fn a_slot_retires_at_the_generation_limit() {
        static POOL: GlobalCell<SlotPool<Box<u8>>> = GlobalCell::new(SlotPool::new());
        let mut a = POOL.with_mut(|pool| pool.insert(Box::new(1)));
        SlotPool::rewind(&mut a, GENERATION_LIMIT);
        assert_eq!(**a.read(), 1);
        drop(POOL.with_mut(|pool| pool.remove(a)));
        assert!(!a.is_live());
        // Retired: the next value goes to a new slot, and the old one's word
        // matches nothing.
        let b = POOL.with_mut(|pool| pool.insert(Box::new(2)));
        assert_ne!(b.raw(), a.raw());
        assert_eq!(header(a.slot).get(), RETIRED_GENERATION);
        // A detached slot at the limit moves its value on attach.
        let mut c = b;
        SlotPool::rewind(&mut c, GENERATION_LIMIT);
        let d = POOL.with_mut(|pool| pool.detach(c));
        let e = POOL.with_mut(|pool| pool.attach(d));
        assert_ne!(e.raw(), c.raw());
        assert_eq!(**e.read(), 2);
        drop(POOL.with_mut(|pool| pool.remove(e)));
    }

    #[test]
    #[should_panic(expected = "another pool")]
    fn a_handle_from_another_pool_is_refused() {
        static A: GlobalCell<SlotPool<u8>> = GlobalCell::new(SlotPool::new());
        static B: GlobalCell<SlotPool<u8>> = GlobalCell::new(SlotPool::new());
        let a = A.with_mut(|pool| pool.insert(1));
        let _ = B.with_mut(|pool| pool.insert(2));
        B.with_mut(|pool| pool.forget(a));
    }

    #[test]
    fn get_mut_borrows_the_value_through_the_pool() {
        static POOL: GlobalCell<SlotPool<u32>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| pool.insert(1));
        POOL.with_mut(|pool| *pool.get_mut(a) += 1);
        assert_eq!(*a.read(), 2);
        POOL.with_mut(|pool| pool.forget(a));
    }

    #[test]
    fn a_value_built_in_its_slot_reads_back() {
        static POOL: GlobalCell<SlotPool<[u32; 4]>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| pool.insert_with(|place| place.write([1, 2, 3, 4])));
        assert_eq!(*a.read(), [1, 2, 3, 4]);
        // SAFETY: `a.raw()` is a slot's value address.
        assert_eq!(unsafe { generation_at(a.raw()) }, a.generation());
        POOL.with_mut(|pool| pool.forget(a));
        // SAFETY: as above; the slot outlives its value.
        assert_ne!(unsafe { generation_at(a.raw()) }, a.generation());
    }

    /// Counts the drops of the boxed values below.
    static DROPS: GlobalCell<u32> = GlobalCell::new(0);

    struct Counted(u32);

    impl Drop for Counted {
        fn drop(&mut self) {
            DROPS.set(DROPS.get() + 1);
        }
    }

    impl Pooled for Counted {
        fn pool() -> &'static GlobalCell<SlotPool<Self>> {
            static POOL: GlobalCell<SlotPool<Counted>> = GlobalCell::new(SlotPool::new());
            &POOL
        }
    }

    #[test]
    fn dropping_a_slot_box_drops_its_value_in_place_and_frees_the_slot() {
        let owner = SlotBox::new(Counted(7));
        let handle = owner.handle();
        assert_eq!(handle.read().0, 7);
        assert_eq!(DROPS.get(), 0);
        drop(owner);
        assert_eq!(DROPS.get(), 1);
        assert!(!handle.is_live());
        // The slot is reused for the next value.
        let next = SlotBox::new_with(|place| place.write(Counted(8)));
        assert_eq!(next.handle().raw(), handle.raw());
        assert_eq!(next.handle().read().0, 8);
        drop(next);
        assert_eq!(DROPS.get(), 2);

        // Parking: the handles go stale at once, the value stays, and the
        // slot takes no new value until the parked one is dropped.
        let owner = SlotBox::new(Counted(9));
        let handle = owner.handle();
        let parked = owner.park();
        assert!(!handle.is_live());
        assert_eq!(DROPS.get(), 2);
        let other = SlotBox::new(Counted(10));
        assert_ne!(other.handle().raw(), handle.raw());
        drop(parked);
        assert_eq!(DROPS.get(), 3);
        let next = SlotBox::new(Counted(11));
        assert_eq!(next.handle().raw(), handle.raw());
        // Abandoning frees the slot without the destructor.
        next.park().abandon();
        assert_eq!(DROPS.get(), 3);
        let again = SlotBox::new(Counted(12));
        assert_eq!(again.handle().raw(), handle.raw());
        drop(again);
        drop(other);
        assert_eq!(DROPS.get(), 5);
    }

    /// **The shape `free_buffer` has**: an address is taken out while the
    /// value is filed (that is `CURBUF`), the table holding the owner then
    /// changes under it, the free path takes the owner out, keeps working
    /// through the *address*, and drops at the end. With `Vec<Box<T>>` this
    /// is undefined behaviour -- the `Box` that comes out is retagged, and
    /// the write through the escaped address invalidates it -- under Stacked
    /// and Tree Borrows alike. A `SlotBox` holds the slot's raw address and
    /// has no retag to invalidate; this keeps it so under `just
    /// miri-perimeter`.
    #[test]
    fn an_address_survives_the_table_changing_and_the_owner_leaving_it() {
        struct Marked(u64);

        impl Pooled for Marked {
            fn pool() -> &'static GlobalCell<SlotPool<Self>> {
                static POOL: GlobalCell<SlotPool<Marked>> = GlobalCell::new(SlotPool::new());
                &POOL
            }
        }

        let mut table = vec![SlotBox::new(Marked(0))];
        let escaped = table[0].handle().raw();
        table.push(SlotBox::new(Marked(1)));
        let owned = table.swap_remove(0);
        // SAFETY: the value is alive -- `owned` holds it -- and `escaped` is
        // the address its slot gives, which nothing has retagged.
        unsafe { (*escaped).0 = 42 };
        assert_eq!(owned.handle().raw(), escaped);
        assert_eq!(owned.handle().read().0, 42);
        drop(owned);
        drop(table);
    }
}
