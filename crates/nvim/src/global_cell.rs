//! `GlobalCell<T>`: the checked cell that replaces `static mut` globals.
//!
//! The editor's state is a web of C globals that c2rust rendered as mutable
//! statics. Any two overlapping references into one of those — an
//! autocmd firing mid-operation and touching the same global, say — is
//! undefined behavior even single-threaded. `GlobalCell` funnels every
//! access through a single `UnsafeCell` so the compiler stops assuming
//! uniqueness, and holds the two invariants the soundness argument rests on.
//!
//! # Soundness contract
//!
//! - **Main-thread only.** Editor state is only touched from the main
//!   thread. `unsafe impl Sync` below is justified by that invariant, which
//!   holds by keeping the ways off the main thread few and audited: luv's
//!   thread states, whose every function is on the list `nlua_init_state`
//!   keeps; the panic hook and the allocator's failure path, which touch
//!   editor state only after asking which thread they are on; and starting
//!   a thread, which clippy refuses outside an audited site
//!   (`crates/nvim/clippy.toml`). docs/safe-surface.md (S2) has the audit.
//!   Builds with the `soundness_checks` cfg (debug assertions, or the
//!   `soundness-checks` feature) also check it on every access (see
//!   [`init_main_thread`]): an access from any other thread panics before
//!   it reads anything, the borrow state included.
//! - **No outstanding references across accesses.** `get`/`set` copy values
//!   in and out without forming references. `with`/`with_mut`/`with_field`
//!   hand out a scoped reference, which the cell records, RefCell-style, in
//!   every build: a conflicting access while it lives panics. `ptr` is the
//!   raw escape hatch for the mechanically converted call sites:
//!   writes/reads through it are exempt from borrow tracking, and callers
//!   must not let a reference from `with`/`with_mut` overlap such an access.
//!
//! # Layout
//!
//! `repr(C)` with the value first, then one word of borrow state. An
//! unmangled `pub static X: GlobalCell<T>` therefore exports a symbol whose
//! address is the value's: the C deps and the LuaJIT-FFI unit suite resolve
//! and poke the value's bytes, and the borrow word lies past what they read.
//!
//! # Cost
//!
//! The borrow check costs a compare per access: the state sits beside the
//! value (one word, no table), on a line the access touches anyway. In a
//! release build that is a few percent of instructions across the A/B
//! benches (docs/safe-surface.md has the numbers). The thread question --
//! one thread-local byte, read in place in an optimised build and through
//! one direct call into the always-optimised `nvim-mainthread` crate in an
//! unoptimised one -- cost as much again, which is why only checked builds
//! ask it.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `global_cell.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// A global editor-state cell. See the module docs for the contract.
#[repr(C)]
#[derive(Debug)]
pub struct GlobalCell<T> {
    /// The value. First, so the cell's address is the value's.
    value: UnsafeCell<T>,
    /// What is borrowed: [`FREE`], [`EXCLUSIVE`], or the innermost of the
    /// shared [`Loan`]s live on the stack, each pointing at the one outside
    /// it. Read and written only on the main thread.
    borrow: Cell<State>,
}

/// A cell's borrow state: [`FREE`], [`EXCLUSIVE`], or the innermost open
/// shared [`Loan`].
type State = *const Loan;

/// Nothing borrowed.
const FREE: State = core::ptr::null();

/// Lent exclusively, to one `with_mut`. Never a `Loan`'s address: a `Loan`
/// is word-aligned.
const EXCLUSIVE: State = core::ptr::without_provenance(1);

/// One shared borrow of a cell: a `with` of the whole record or a
/// `with_field` of one field, as the byte range it lends. It lives in the
/// borrowing call's frame for exactly as long as the borrow, and the cell
/// points at it, so a nested borrow of the same cell finds every loan open
/// by walking `outer`.
#[derive(Debug)]
struct Loan {
    /// The lent bytes, `start..end`, as addresses.
    start: usize,
    end: usize,
    /// The loan open when this one was made: another [`Loan`], or [`FREE`].
    outer: State,
}

/// Puts a cell's borrow state back when the borrow ends, however it ends
/// (an unwinding panic in a lib test included).
#[derive(Debug)]
struct Restore<'a> {
    borrow: &'a Cell<State>,
    to: State,
}

impl Drop for Restore<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        // On the main thread, as the access that made this guard was.
        self.borrow.set(self.to);
    }
}

// SAFETY: editor globals are only accessed from the main thread: nothing
// that runs on another thread reaches a cell (the module docs say why, and
// docs/safe-surface.md S2 has the audit), and a checked build checks it
// before touching the value or the borrow state. The `Sync` bound is
// required for `static` items; it is not a claim of actual thread safety.
unsafe impl<T> Sync for GlobalCell<T> {}

impl<T> GlobalCell<T> {
    pub const fn new(value: T) -> Self {
        GlobalCell {
            value: UnsafeCell::new(value),
            borrow: Cell::new(FREE),
        }
    }

    /// Raw pointer to the cell contents, with the main-thread check.
    ///
    /// This is what the mechanical `static mut` conversion emits for place
    /// expressions (field projections, `&raw mut`, index, ...). Accesses
    /// through it carry exactly the obligations the old mutable-static
    /// access did, minus the reference-uniqueness landmine.
    #[inline(always)]
    pub fn ptr(&self) -> *mut T {
        check_main_thread();
        self.value.get()
    }

    /// Like [`ptr`](Self::ptr) but callable in const initializers (some
    /// globals hold the address of another global). No checks.
    pub const fn as_raw(&self) -> *mut T {
        self.value.get()
    }

    /// The borrow state, after the thread check.
    #[inline(always)]
    fn borrowed(&self) -> State {
        check_main_thread();
        self.borrow.get()
    }

    /// Record `now` as the borrow state until the guard drops, and `back`
    /// then.
    #[inline(always)]
    fn hold(&self, now: State, back: State) -> Restore<'_> {
        self.borrow.set(now);
        Restore {
            borrow: &self.borrow,
            to: back,
        }
    }

    /// The cell's bytes, as a [`Loan`] range.
    #[inline(always)]
    fn whole(&self) -> (usize, usize) {
        let start = self.value.get().addr();
        (start, start + size_of::<T>().max(1))
    }

    /// Copy the value out.
    #[inline(always)]
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        if self.borrowed() == EXCLUSIVE {
            read_conflict(self.value.get().addr());
        }
        // SAFETY: main-thread invariant + no outstanding exclusive borrow.
        unsafe { *self.value.get() }
    }

    /// Overwrite the value.
    #[inline(always)]
    pub fn set(&self, value: T) {
        let borrowed = self.borrowed();
        if borrowed != FREE {
            whole_write_conflict(borrowed, self.whole(), "set");
        }
        // SAFETY: main-thread invariant + no outstanding borrow.
        unsafe { *self.value.get() = value }
    }

    /// Swap in a new value, returning the old one.
    pub fn replace(&self, value: T) -> T
    where
        T: Copy,
    {
        let old = self.get();
        self.set(value);
        old
    }

    /// Move the value out, leaving the cell's own empty state behind.
    ///
    /// This is the safe spelling of the C save/restore idiom
    /// (`save->sr_redobuff = redobuff; redobuff.bh_first.b_next = NULL;`):
    /// what the cell holds is handed to one owner and the cell is left with
    /// nothing to free, in one step that cannot be half-done.
    ///
    /// Two design choices, both deliberate:
    ///
    /// - **The empty state is `T::default()`, not an argument.** Passing the
    ///   replacement is what [`replace`](Self::replace) already is; the
    ///   point of `take` is that the *type* names its own empty once, so
    ///   every move site spells the same thing and a new field cannot be
    ///   left holding a stale value at one site out of four.
    /// - **There is no `Copy` bound.** A move is precisely what a value that
    ///   owns an allocation wants, and `get` — which needs `Copy` — is the
    ///   operation that leaves the cell and the caller holding the *same*
    ///   pointer. So the types worth taking are the ones that are not `Copy`,
    ///   and requiring it would exclude them.
    ///
    /// The move itself is `with_mut(mem::take)`: the closure is `mem::take`,
    /// which provably cannot re-enter, so the exclusive borrow is a leaf one
    /// and the borrow check catches a caller that is already inside a
    /// `with`/`with_mut` on the same cell.
    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.with_mut(core::mem::take)
    }

    /// Copy one field of the contents out, forming no reference to the rest.
    ///
    /// This is [`get`](Self::get) for a record too large to copy whole — the
    /// editor's option values, say, which are one cell with three hundred
    /// fields. `project` names the field and does nothing else; it is
    /// generated (`|o| &mut o.field`), so the reference it takes is gone
    /// before the copy is made and no caller ever holds one.
    ///
    /// The borrow rules are `get`'s, not `with`'s: the read is a copy, and it
    /// may not happen inside a `with_mut` on the same cell. That is the point
    /// — a field read that recorded a borrow would cost every option read in
    /// the editor a loan, and would turn an option read inside a `did_set_*`
    /// callback into a panic.
    ///
    /// The projection is a *shared* one. A read may happen inside
    /// [`with_field`](Self::with_field) on another field of the same record
    /// (an option read under a `'cpoptions'` borrow, say), and a `&mut` to
    /// the whole record there would retire the outer reference while it is
    /// still in use -- undefined behaviour under Stacked Borrows, which Miri
    /// reports. A shared reborrow is a read, and reads coexist.
    #[inline(always)]
    pub fn get_field<F: Copy>(&self, project: impl FnOnce(&T) -> &F) -> F {
        if self.borrowed() == EXCLUSIVE {
            read_conflict(self.value.get().addr());
        }
        // SAFETY: main-thread invariant + no outstanding exclusive borrow.
        *project(unsafe { &*self.value.get() })
    }

    /// Overwrite one field of the contents. See
    /// [`get_field`](Self::get_field).
    ///
    /// The projection is shared here too, and the write goes through the
    /// cell's own pointer at the field's address. A write may happen inside
    /// [`with_field`](Self::with_field) on another field — an option set
    /// while `'cpoptions'` is lent, say — and a `&mut` to the whole record
    /// would retire the reference that closure holds (Stacked Borrows: a
    /// Unique retag of every byte of the record, Miri's report). Written
    /// through the cell's pointer, only the field's own bytes are touched.
    ///
    /// It is [`replace_field`](Self::replace_field) with the old value
    /// dropped, so a field that owns an allocation is freed after the cell
    /// already holds the new value: a `Drop` that reads the field back sees
    /// the new one, never a freed one.
    #[inline(always)]
    pub fn set_field<F>(&self, project: impl FnOnce(&T) -> &F, value: F) {
        drop(self.replace_field(project, value));
    }

    /// The address of the field `project` reaches, with the cell's own
    /// provenance, so a write through it touches that field and nothing
    /// else.
    ///
    /// `project` is any safe `&T -> &F`, so what it answers is checked to lie
    /// inside the record: a reference into the record's bytes is an aligned,
    /// valid `F` there, which is all a write needs. The check folds away
    /// once the projection inlines (`field!` and apigen write `|o| &o.f`, a
    /// constant offset).
    #[inline(always)]
    fn field_ptr<F>(&self, project: impl FnOnce(&T) -> &F) -> *mut F {
        let record = self.value.get();
        // SAFETY: main-thread invariant + no outstanding exclusive borrow
        // (the callers check both); the shared reborrow lives only for the
        // projection.
        let field: *const F = project(unsafe { &*record });
        let offset = (field as usize)
            .checked_sub(record as usize)
            .filter(|at| {
                at.checked_add(size_of::<F>())
                    .is_some_and(|end| end <= size_of::<T>())
            })
            .expect("a field projection must answer a field of the record");
        record.wrapping_byte_add(offset).cast::<F>()
    }

    /// Project one field of the contents, forming no reference to the rest.
    ///
    /// This is [`get_field`](Self::get_field) for a field too expensive to
    /// copy — a string option's owned buffer, say. `f` sees the field and
    /// the borrow ends when it returns, which is the whole of the contract:
    /// a value that has to outlive the call is cloned inside `f`, never
    /// handed out by reference.
    ///
    /// The borrow is tracked per *field*, not per cell: the cell records
    /// a shared loan of the field for as long as `f` runs, so a write to
    /// that field from inside `f` ([`set_field`](Self::set_field),
    /// [`replace_field`](Self::replace_field)) panics, as does anything that
    /// would write the whole record ([`set`](Self::set),
    /// [`with_mut`](Self::with_mut)). Reads of any field, and writes to the
    /// record's *other* fields, stay free -- a whole-cell borrow would turn
    /// an option set inside a `did_set_*` callback under a `'cpoptions'`
    /// loan into a panic.
    ///
    /// The projection is shared, as [`get_field`](Self::get_field)'s is and
    /// for the same reason: `f` may read other fields of the record.
    #[inline(always)]
    pub fn with_field<F, R>(&self, project: impl FnOnce(&T) -> &F, f: impl FnOnce(&F) -> R) -> R {
        let outer = self.borrowed();
        if outer == EXCLUSIVE {
            read_conflict(self.value.get().addr());
        }
        // SAFETY: main-thread invariant + no outstanding exclusive borrow;
        // the loan below makes a write to the field (or the record) while
        // `f` holds it panic.
        let field = project(unsafe { &*self.value.get() });
        let start = core::ptr::from_ref(field).addr();
        let loan = Loan {
            start,
            end: start + size_of::<F>().max(1),
            outer,
        };
        let _restore = self.lend(&loan);
        f(field)
    }

    /// Overwrite one field and answer what it held. The move out and the
    /// move in are one operation, so a field that owns an allocation can
    /// change hands without either side being able to free it twice.
    ///
    /// The write projects as [`set_field`](Self::set_field) describes.
    #[inline(always)]
    pub fn replace_field<F>(&self, project: impl FnOnce(&T) -> &F, value: F) -> F {
        let borrowed = self.borrowed();
        if borrowed == EXCLUSIVE {
            // Before the projection: it reborrows the record `with_mut` holds.
            whole_write_conflict(borrowed, self.whole(), "replace_field");
        }
        let field = self.field_ptr(project);
        if borrowed != FREE {
            field_write_conflict(borrowed, field.addr(), size_of::<F>(), self.whole());
        }
        // SAFETY: main-thread invariant + no outstanding borrow of the
        // field; `field` is an aligned, initialised `F` inside the cell
        // carrying the cell's provenance (`field_ptr`), and nothing holds a
        // reference to it.
        core::mem::replace(unsafe { &mut *field }, value)
    }

    /// Run `f` with a shared reference to the contents.
    #[inline(always)]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let outer = self.borrowed();
        if outer == EXCLUSIVE {
            shared_conflict(self.value.get().addr());
        }
        let (start, end) = self.whole();
        let loan = Loan { start, end, outer };
        let _restore = self.lend(&loan);
        // SAFETY: main-thread invariant + no exclusive borrow, and the loan
        // turns any write while `f` holds the reference into a panic.
        f(unsafe { &*self.value.get() })
    }

    /// Run `f` with an exclusive reference to the contents.
    ///
    /// Panics if the cell is already borrowed — this is the probe that turns
    /// vim's historic autocmd-reentrancy corruption into a loud failure
    /// instead of silent UB.
    #[inline(always)]
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let outer = self.borrowed();
        if outer != FREE {
            whole_write_conflict(outer, self.whole(), "with_mut");
        }
        let _restore = self.hold(EXCLUSIVE, outer);
        // SAFETY: main-thread invariant + nothing else borrowed, and the
        // state says so to every access until `f` returns.
        f(unsafe { &mut *self.value.get() })
    }

    /// Open `loan` (whose `outer` is the state now) as the innermost borrow
    /// of the cell, until the guard drops. Tying both to one lifetime keeps
    /// the loan alive, and in place, for as long as the cell points at it.
    #[inline(always)]
    fn lend<'a>(&'a self, loan: &'a Loan) -> Restore<'a> {
        self.hold(core::ptr::from_ref(loan), loan.outer)
    }
}

/// One field of a record `S` that holds a `T`: a `const` selector.
///
/// A record that is one value — the editor's option table behind one
/// [`GlobalCell`], a buffer's or a window's local option copies — is reached
/// a field at a time, and the selector is how a caller *names* the field
/// without holding a reference into the record. It is a projection plus the
/// offset it projects to: the projection does the typed access, and the
/// offset is the selector's identity, so two selectors are equal when they
/// name the same field (comparing the function pointers would warn and mean
/// nothing).
///
/// Nothing here reads the record. Whoever holds the record resolves the
/// selector against it **per access** — [`GlobalCell::get_at`] and friends
/// for a cell, a `&mut` borrowed for one statement for a handle — so no
/// reference outlives the read or write that asked for it, and a callback
/// that re-enters the editor cannot find one open.
///
/// Spell one with the crate's `field!` macro.
#[derive(Debug)]
pub struct Field<S, T> {
    /// The field's offset in `S` — the selector's identity.
    at: usize,
    /// The field itself, typed, so a selector cannot name a field of
    /// another type.
    project: fn(&mut S) -> &mut T,
    /// The same field, shared: what a cell access projects through, read or
    /// write, so that one nested in a borrow of another field does not take
    /// the whole record mutably (see [`GlobalCell::get_field`] and
    /// [`GlobalCell::set_field`]). `project` is for a record the caller
    /// already holds `&mut` ([`Field::of`]).
    view: fn(&S) -> &T,
}

// Hand-written rather than derived: `derive` would demand `S: Copy` and
// `T: Copy`, and neither the record nor the field need be for a *selector*
// to be.
impl<S, T> Clone for Field<S, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<S, T> Copy for Field<S, T> {}

impl<S, T> PartialEq for Field<S, T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}

impl<S, T> Eq for Field<S, T> {}

impl<S, T> Field<S, T> {
    /// The selector for the field `project` reaches, which sits `at` bytes
    /// into `S`. `field!` writes both halves
    /// from one field name, which is the only way they stay a pair.
    pub const fn new(at: usize, project: fn(&mut S) -> &mut T, view: fn(&S) -> &T) -> Self {
        Self { at, project, view }
    }

    /// The field's offset in `S`, which is what identifies it.
    #[inline(always)]
    pub const fn field_offset(self) -> usize {
        self.at
    }

    /// The field, in a record the caller has borrowed for exactly this one
    /// access.
    #[inline(always)]
    pub fn of(self, record: &mut S) -> &mut T {
        (self.project)(record)
    }
}

/// A [`Field`] selector for `$record`'s field `$field` (a dotted path
/// reaches into a nested struct), usable in a `const`.
macro_rules! field {
    ($record:ty, $($field:ident).+) => {
        $crate::global_cell::Field::<$record, _>::new(
            ::core::mem::offset_of!($record, $($field).+),
            |record: &mut $record| &mut record.$($field).+,
            |record: &$record| &record.$($field).+,
        )
    };
}
pub(crate) use field;

/// A `c_int` level a [`crate::guard`] guard raises and puts back: a cell
/// of its own, or a field of a state record (every [`state_record!`]
/// implements it for its `c_int` fields).
pub(crate) trait Level: Copy {
    /// The level now.
    fn level(self) -> core::ffi::c_int;
    /// Overwrite the level.
    fn set_level(self, level: core::ffi::c_int);
}

impl Level for &'static GlobalCell<core::ffi::c_int> {
    #[inline(always)]
    fn level(self) -> core::ffi::c_int {
        self.get()
    }

    #[inline(always)]
    fn set_level(self, level: core::ffi::c_int) {
        self.set(level);
    }
}

/// One record of editor-wide state behind one cell, with a `const`
/// [`Field`] selector per field **under the field's own name**.
///
/// ```ignore
/// state_record! {
///     /// What the record is.
///     pub(crate) struct ModeState in MODE as ModeField;
///     /// What the field is.
///     pub(crate) State: c_int = MODE_NORMAL;
/// }
/// ```
///
/// writes the struct, `static MODE: GlobalCell<ModeState>`, the alias
/// `ModeField<T> = Field<ModeState, T>`, a `const State: ModeField<c_int>`,
/// and one generic inherent impl giving every selector `get`, `set`,
/// `replace`, `take`, `with`, `update` and `with_mut` -- so a reader still spells
/// `State.get()`, and what changed is that nothing can take a field's
/// address.
///
/// The record is never borrowed whole. Every method is one field access
/// with [`GlobalCell::get_at`]'s rules: untracked, no reference outliving
/// it. `update` moves the value out, changes it and puts it back, so a
/// callee that re-enters the editor finds no borrow open, only the empty
/// value.
///
/// Keep the record and its methods `pub(crate)`. The methods are generic,
/// so a `pub` one would ship its body for cross-crate inlining, and every
/// static a shipped body names is exported -- and an exported static is
/// read through the GOT, one more load at every read in the editor.
macro_rules! state_record {
    (
        $(#[$rdoc:meta])*
        $rvis:vis struct $record:ident in $cell:ident as $alias:ident;
        $(
            $(#[$doc:meta])*
            $vis:vis $field:ident: $ty:ty = $init:expr;
        )*
    ) => {
        $(#[$rdoc])*
        #[allow(non_snake_case, reason = "each field keeps its selector's (upstream) name")]
        #[derive(Debug)]
        $rvis struct $record {
            $($field: $ty,)*
        }

        static $cell: $crate::global_cell::GlobalCell<$record> =
            $crate::global_cell::GlobalCell::new($record {
                $($field: $init,)*
            });

        /// A field of the record.
        $rvis type $alias<T> = $crate::global_cell::Field<$record, T>;

        #[allow(dead_code, reason = "every record gets the whole set; not every one uses it")]
        impl<T> $crate::global_cell::Field<$record, T> {
            /// What the field holds.
            #[inline(always)]
            $rvis fn get(self) -> T
            where
                T: Copy,
            {
                $cell.get_at(self)
            }

            /// Overwrite the field.
            #[inline(always)]
            $rvis fn set(self, value: T) {
                $cell.set_at(self, value);
            }

            /// Overwrite the field, answering what it held.
            #[inline(always)]
            $rvis fn replace(self, value: T) -> T {
                $cell.replace_at(self, value)
            }

            /// Move the value out, leaving the type's empty value behind.
            #[inline(always)]
            $rvis fn take(self) -> T
            where
                T: Default,
            {
                $cell.replace_at(self, T::default())
            }

            /// Look at the field, for a test or a clone. The loan is
            /// [`GlobalCell::with_field`]'s: a write to this field (or the
            /// whole record) from inside `f` panics.
            #[inline(always)]
            $rvis fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
                $cell.with_at(self, f)
            }

            /// Change the field in place: moved out for the duration and
            /// put back after, so anything `f` reaches that reads the field
            /// sees it empty, and anything that writes it is overwritten.
            /// Meant for a push or a clear, not for a call that can run
            /// user code.
            #[inline(always)]
            $rvis fn update<R>(self, f: impl FnOnce(&mut T) -> R) -> R
            where
                T: Default,
            {
                let mut value = self.take();
                let answer = f(&mut value);
                self.set(value);
                answer
            }

            /// Change the field in place, with the record borrowed for the
            /// duration -- tracked, so anything `f`
            /// reaches that touches the record panics. For a field too
            /// large to move out and back on a hot path (`gotchars`'s
            /// per-byte key assembly); `f` must be a leaf.
            #[inline(always)]
            $rvis fn with_mut<R>(self, f: impl FnOnce(&mut T) -> R) -> R {
                $cell.with_mut(|record| f(self.of(record)))
            }
        }

        impl $crate::global_cell::Level for $crate::global_cell::Field<$record, ::core::ffi::c_int> {
            fn level(self) -> ::core::ffi::c_int {
                $cell.get_at(self)
            }

            fn set_level(self, level: ::core::ffi::c_int) {
                $cell.set_at(self, level);
            }
        }

        $(
            $(#[$doc])*
            $vis const $field: $alias<$ty> = $crate::global_cell::field!($record, $field);
        )*
    };
}
pub(crate) use state_record;

impl<T> GlobalCell<T> {
    /// [`get_field`](Self::get_field) through a [`Field`] selector.
    #[inline(always)]
    pub fn get_at<F: Copy>(&self, field: Field<T, F>) -> F {
        self.get_field(field.view)
    }

    /// [`set_field`](Self::set_field) through a [`Field`] selector.
    #[inline(always)]
    pub fn set_at<F>(&self, field: Field<T, F>, value: F) {
        self.set_field(field.view, value);
    }

    /// [`with_field`](Self::with_field) through a [`Field`] selector.
    #[inline(always)]
    pub fn with_at<F, R>(&self, field: Field<T, F>, f: impl FnOnce(&F) -> R) -> R {
        self.with_field(field.view, f)
    }

    /// [`replace_field`](Self::replace_field) through a [`Field`] selector.
    #[inline(always)]
    pub fn replace_at<F>(&self, field: Field<T, F>, value: F) -> F {
        self.replace_field(field.view, value)
    }
}

/// A static only ever reached through a raw pointer: the event loop, whose
/// address libuv holds and whose every touch is a libuv call.
///
/// It lends nothing safe — no `get`, no `set`, no reference — so sharing
/// it between threads shares an address and nothing else; whatever reads or
/// writes through [`ptr`](Self::ptr) does so in its own `unsafe` and
/// answers there for which thread it is on.
#[repr(transparent)]
#[derive(Debug)]
pub struct RawStatic<T>(UnsafeCell<T>);

// SAFETY: the only access is `ptr`, a raw pointer; dereferencing it is
// `unsafe` at the caller, who carries the thread and aliasing obligations.
// A static is never dropped, so `T: !Send` cannot be dropped on a stranger.
unsafe impl<T> Sync for RawStatic<T> {}

impl<T> RawStatic<T> {
    pub const fn new(value: T) -> Self {
        RawStatic(UnsafeCell::new(value))
    }

    /// The value's address, for the life of the process.
    #[inline(always)]
    pub const fn ptr(&self) -> *mut T {
        self.0.get()
    }
}

/// Armed once [`init_main_thread`] runs; false in processes that never call
/// it (unit tests FFI-loading the symbols, `.init_array` constructors,
/// `cargo test`), which keeps the thread check inert there.
static ARMED: AtomicBool = AtomicBool::new(false);

/// Set by [`emergency`]: a conflict with a borrow that may be an
/// interrupted frame's no longer panics.
static ABANDONED: AtomicBool = AtomicBool::new(false);

/// The address of a local of [`emergency`]'s own frame, once it began: a
/// shared [`Loan`] above it (the stack grows down on every target the
/// editor builds for) is a local of a frame the emergency interrupted, and
/// one below it a frame the exit has run since.
static EMERGENCY_FRAME: AtomicUsize = AtomicUsize::new(0);

/// Record the calling thread as the main thread and arm the main-thread
/// assertion of a checked build. Called from the binary entry point before
/// any editor code runs.
pub fn init_main_thread() {
    nvim_mainthread::mark_main_thread();
    ARMED.store(true, Ordering::Relaxed);
}

/// Proof that an emergency exit is under way: made only by [`emergency`],
/// which never returns, so no frame that held a borrow, guard or pin when
/// it began ever runs again. Everything that forgets such a loan as if it
/// had been given back -- the slot pools' `abandon_loans`, the memfile's
/// `abandon_pins` -- asks for one, which is what makes forgetting sound.
#[derive(Debug)]
pub(crate) struct Emergency {
    _made_by_emergency: (),
}

impl Emergency {
    /// A token for a test that abandons loans and then drops nothing they
    /// lent.
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self {
            _made_by_emergency: (),
        }
    }
}

/// End the process through `exit`, every open borrow abandoned first, and
/// `exit` gets the [`Emergency`] the other abandons ask for. It never
/// returns -- its type says so -- which is the whole contract: the frames
/// that hold the open borrows and guards never resume, so forgetting them
/// lets the swap-file cleanup reach everything without tripping over a
/// loan whose owner can no longer give it back.
///
/// What is forgotten is bounded by what can be told apart. A shared loan
/// (`with`, `with_field`) lives in its borrower's frame, so a write that
/// meets one still panics when the loan is below this frame -- `exit`'s own
/// borrow -- and goes on only past the interrupted frames' loans. An
/// exclusive borrow (`with_mut`) is one state word with no owner in it, so a
/// conflict with one goes on for the rest of the exit, `exit`'s own nested
/// `with_mut`s included (docs/unsafe-audit.md, `global_cell.rs`).
///
/// Only the conflict paths read the flag and the mark, so they cost the
/// access paths nothing. Never inlined: the mark must be this frame's, below
/// every local of the caller.
#[inline(never)]
pub(crate) fn emergency(exit: impl FnOnce(&Emergency) -> core::convert::Infallible) -> ! {
    let frame = 0u8;
    EMERGENCY_FRAME.store(core::ptr::from_ref(&frame).addr(), Ordering::Relaxed);
    ABANDONED.store(true, Ordering::Relaxed);
    match exit(&Emergency {
        _made_by_emergency: (),
    }) {}
}

/// Whether [`emergency`] began.
fn abandoned() -> bool {
    ABANDONED.load(Ordering::Relaxed)
}

/// Whether `loan` is a local of a frame [`emergency`] interrupted, which
/// never runs again: false before one began.
fn interrupted(loan: &Loan) -> bool {
    abandoned() && above(loan, EMERGENCY_FRAME.load(Ordering::Relaxed))
}

/// Whether `loan` lives in a frame older than the one holding `mark`.
fn above(loan: &Loan, mark: usize) -> bool {
    core::ptr::from_ref(loan).addr() > mark
}

#[inline(always)]
fn check_main_thread() {
    #[cfg(soundness_checks)]
    if !on_main_thread() {
        off_main_thread();
    }
}

/// The thread question, outlined into the optimised crate, where it is one
/// thread-local load: an unoptimised build inlines nothing anyway.
#[cfg(all(soundness_checks, debug_assertions))]
#[inline(always)]
fn on_main_thread() -> bool {
    nvim_mainthread::is_main_thread()
}

/// The thread question, one thread-local byte read in place.
#[cfg(all(soundness_checks, not(debug_assertions)))]
#[inline(always)]
fn on_main_thread() -> bool {
    nvim_mainthread::is_main_thread_inline()
}

#[cfg(soundness_checks)]
#[cold]
#[inline(never)]
fn off_main_thread() {
    if ARMED.load(Ordering::Relaxed) {
        panic!(
            "GlobalCell accessed off the main thread (thread {:?})",
            std::thread::current().id()
        );
    }
}

// The conflict paths. Each is out of line and cold, so an access inlines
// one compare and a call nobody takes.

/// A read (`get`, `get_field`, `with_field`) while `with_mut` holds the
/// cell at `cell`.
#[cold]
#[inline(never)]
fn read_conflict(cell: usize) {
    if !abandoned() {
        panic!("GlobalCell read during an active with_mut borrow (cell @ {cell:#x})");
    }
}

/// A `with` while `with_mut` holds the cell at `cell`.
#[cold]
#[inline(never)]
fn shared_conflict(cell: usize) {
    if !abandoned() {
        panic!(
            "GlobalCell::with while exclusively borrowed (cell @ {cell:#x}) — \
             reentrant global access"
        );
    }
}

/// A write of the whole record (`what`: `set`, `with_mut`) while it is
/// borrowed: by `with_mut` or `with`, or by a `with_field` of one field.
#[cold]
#[inline(never)]
fn whole_write_conflict(borrowed: State, (start, end): (usize, usize), what: &str) {
    // SAFETY: the cell's state, read just now; the walk ends here.
    let mut live = unsafe { loans(borrowed) }
        .filter(|loan| !interrupted(loan))
        .peekable();
    // An exclusive borrow has no owner to ask after; shared loans do.
    let forgiven = if borrowed == EXCLUSIVE {
        abandoned()
    } else {
        live.peek().is_none()
    };
    if forgiven {
        return;
    }
    let whole_lent =
        borrowed == EXCLUSIVE || live.any(|loan| loan.start == start && loan.end == end);
    if whole_lent {
        panic!(
            "GlobalCell::{what} while already borrowed (cell @ {start:#x}) — \
             reentrant global access"
        );
    }
    panic!("GlobalCell written whole while with_field lends one of its fields (cell @ {start:#x})");
}

/// A write of the `size` bytes at `field` while the record (`whole`) has
/// shared loans open: a conflict if one of them covers the field.
#[cold]
#[inline(never)]
fn field_write_conflict(borrowed: State, field: usize, size: usize, whole: (usize, usize)) {
    let end = field + size.max(1);
    // SAFETY: the cell's state, read just now; the walk ends here.
    let conflict = unsafe { loans(borrowed) }
        .find(|loan| !interrupted(loan) && field < loan.end && loan.start < end);
    if let Some(loan) = conflict {
        if (loan.start, loan.end) == whole {
            panic!("GlobalCell field written during an active with borrow (field @ {field:#x})");
        }
        panic!("GlobalCell field written while with_field lends it (field @ {field:#x})");
    }
}

/// The shared loans open on a cell, innermost first, from its borrow state.
///
/// # Safety
/// `borrowed` is a cell's current state, and the iterator and every loan
/// it yields are used before any borrow on the cell ends (a conflict path,
/// which reads the chain and panics or returns).
unsafe fn loans<'a>(borrowed: State) -> impl Iterator<Item = &'a Loan> {
    let mut next = borrowed;
    core::iter::from_fn(move || {
        if next == FREE || next == EXCLUSIVE {
            return None;
        }
        // SAFETY: every loan reachable from a cell's state is live: each is
        // a local of a frame still inside the borrow that opened it, and
        // that borrow's `Restore` takes it off the chain before the frame
        // ends. The one way a frame is left without its `Restore` is
        // `emergency`, which never returns to it: the frame is suspended,
        // not popped, so its locals stay where they were, unwritten.
        let loan = unsafe { &*next };
        next = loan.outer;
        Some(loan)
    })
}

/// The lib harness's editor lock.
///
/// The editor is single-threaded -- that is what [`GlobalCell`]'s whole
/// contract rests on -- but `cargo test` runs a binary's cases in parallel
/// threads, and every one of them shares the one copy of the editor's
/// process-wide state: the collector's registries, the window and frame
/// registries, the suppression counters. A case that touches any of it
/// takes [`editor_state_lock`] first, and holding the guard *is* being the
/// main thread for as long as it lives.
///
/// One lock rather than one per registry, because a case does not know
/// which registries the code under test reaches: a window allocates a list,
/// a list's teardown can message. Two locks taken in different orders by two
/// cases would deadlock; one cannot.
///
/// The lock is reentrant on its own thread, so a helper that takes it inside
/// a case that already holds it nests instead of deadlocking. Poisoning is
/// ignored: a panicking case has already reported its own failure, and a
/// second report of it in every case that follows is noise.
///
/// `crates/nvim/tests/unit` is a different binary with its own
/// `editor_lock()`; the two cannot share a mutex and need not.
#[cfg(test)]
pub(crate) mod editor_state {
    use std::cell::Cell;
    use std::sync::{Mutex, MutexGuard};

    static EDITOR_STATE: Mutex<()> = Mutex::new(());

    thread_local! {
        /// How many live guards this thread holds. A thread-local count
        /// rather than the guard's identity because [`assert_held`] is
        /// called from inside the locked region, where asking the mutex
        /// would deadlock.
        static DEPTH: Cell<usize> = const { Cell::new(0) };
    }

    /// What [`editor_state_lock`](super::editor_state_lock) hands back. The
    /// outermost guard on a thread owns the mutex; nested ones only count.
    #[derive(Debug)]
    pub(crate) struct Held(
        #[expect(dead_code, reason = "held for its lock")] Option<MutexGuard<'static, ()>>,
    );

    pub(super) fn lock() -> Held {
        let guard = if DEPTH.get() == 0 {
            Some(EDITOR_STATE.lock().unwrap_or_else(|e| e.into_inner()))
        } else {
            None
        };
        DEPTH.set(DEPTH.get() + 1);
        Held(guard)
    }

    impl Drop for Held {
        fn drop(&mut self) {
            DEPTH.set(DEPTH.get() - 1);
        }
    }

    /// Panic unless this thread holds the lock.
    pub(crate) fn assert_held() {
        assert!(
            DEPTH.get() > 0,
            "a test that touches the editor's process-wide state must hold \
             `editor_state_lock()`"
        );
    }
}

/// Exclusive use of the editor's process-wide state for a `cargo test` case;
/// see [`editor_state`].
#[cfg(test)]
pub(crate) fn editor_state_lock() -> editor_state::Held {
    editor_state::lock()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_replace_roundtrip() {
        static CELL: GlobalCell<i32> = GlobalCell::new(7);
        assert_eq!(CELL.get(), 7);
        CELL.set(9);
        assert_eq!(CELL.replace(11), 9);
        assert_eq!(CELL.get(), 11);
    }

    #[test]
    fn ptr_and_as_raw_agree_and_the_value_comes_first() {
        static CELL: GlobalCell<u64> = GlobalCell::new(0xdead);
        assert_eq!(CELL.ptr(), CELL.as_raw());
        // repr(C), value first: the cell's address IS the value's address,
        // which is what an exported cell's C readers rely on.
        assert_eq!(core::ptr::from_ref(&CELL).addr(), CELL.as_raw().addr());
        // Then the borrow word, in every build.
        assert_eq!(size_of::<GlobalCell<u64>>(), 16);
    }

    #[test]
    fn with_mut_scopes_nest_across_distinct_cells() {
        static A: GlobalCell<i32> = GlobalCell::new(1);
        static B: GlobalCell<i32> = GlobalCell::new(2);
        let sum = A.with_mut(|a| {
            *a += 10;
            B.with(|b| *a + *b)
        });
        assert_eq!(sum, 13);
        assert_eq!(A.get(), 11);
    }

    #[test]
    #[should_panic(expected = "reentrant global access")]
    fn reentrant_with_mut_panics() {
        static CELL: GlobalCell<i32> = GlobalCell::new(0);
        CELL.with_mut(|_| CELL.with_mut(|inner| *inner));
    }

    #[test]
    #[should_panic(expected = "with_mut borrow")]
    fn get_during_with_mut_panics() {
        static CELL: GlobalCell<i32> = GlobalCell::new(0);
        CELL.with_mut(|_| CELL.get());
    }

    #[test]
    fn shared_borrows_stack() {
        static CELL: GlobalCell<i32> = GlobalCell::new(5);
        let v = CELL.with(|a| CELL.with(|b| *a + *b));
        assert_eq!(v, 10);
        // Fully released: an exclusive borrow works again afterwards.
        CELL.with_mut(|x| *x = 6);
        assert_eq!(CELL.get(), 6);
    }

    #[test]
    fn a_field_selector_reaches_one_field_per_access() {
        #[derive(Debug)]
        struct Record {
            count: i32,
            name: Option<Vec<u8>>,
        }
        const COUNT: Field<Record, i32> = field!(Record, count);
        const NAME: Field<Record, Option<Vec<u8>>> = field!(Record, name);
        // A local cell: the accessors are the same, and the ratchet counts
        // only `static` ones.
        let cell = GlobalCell::new(Record {
            count: 1,
            name: None,
        });

        assert!(
            COUNT
                != Field::new(
                    NAME.field_offset(),
                    |r: &mut Record| &mut r.count,
                    |r| &r.count
                )
        );
        assert_eq!(cell.get_at(COUNT), 1);
        cell.set_at(COUNT, 2);
        assert_eq!(cell.get_at(COUNT), 2);
        assert_eq!(cell.replace_at(NAME, Some(b"x".to_vec())), None);
        assert_eq!(
            cell.with_at(NAME, |name| name.as_deref().map(<[u8]>::len)),
            Some(1)
        );
        drop(cell.replace_at(NAME, None));

        // Outside a cell, the selector projects a record the caller holds.
        let mut record = Record {
            count: 7,
            name: None,
        };
        *COUNT.of(&mut record) += 1;
        assert_eq!(record.count, 8);
    }

    #[test]
    fn a_field_write_nested_in_a_field_borrow_leaves_the_borrow_live() {
        #[derive(Debug)]
        struct Record {
            count: i32,
            name: Option<Vec<u8>>,
        }
        const COUNT: Field<Record, i32> = field!(Record, count);
        const NAME: Field<Record, Option<Vec<u8>>> = field!(Record, name);
        let cell = GlobalCell::new(Record {
            count: 1,
            name: Some(b"abc".to_vec()),
        });

        // An option write inside an option borrow: 'cpoptions' lent to a
        // reader that, say, reports an error and so sets another option.
        // The write must not retire the reference the outer closure holds.
        let seen = cell.with_at(NAME, |name| {
            cell.set_at(COUNT, 2);
            let was = cell.replace_at(COUNT, 3);
            (was, name.as_deref().map(<[u8]>::len))
        });
        assert_eq!(seen, (2, Some(3)));
        assert_eq!(cell.get_at(COUNT), 3);
        let count = cell.with_field(
            |record| &record.name,
            |name| {
                cell.set_field(|record| &record.count, 4);
                name.as_deref().map(<[u8]>::len)
            },
        );
        assert_eq!((count, cell.get_at(COUNT)), (Some(3), 4));
    }

    #[derive(Debug)]
    struct Lent {
        count: i32,
        name: Option<Vec<u8>>,
    }
    const LENT_COUNT: Field<Lent, i32> = field!(Lent, count);
    const LENT_NAME: Field<Lent, Option<Vec<u8>>> = field!(Lent, name);

    #[test]
    #[should_panic(expected = "while with_field lends it")]
    fn a_write_to_the_lent_field_panics() {
        let cell = GlobalCell::new(Lent {
            count: 1,
            name: Some(b"abc".to_vec()),
        });
        cell.with_at(LENT_NAME, |_| cell.set_at(LENT_NAME, None));
    }

    #[test]
    #[should_panic(expected = "while with_field lends one of its fields")]
    fn a_whole_record_write_under_a_field_loan_panics() {
        let cell = GlobalCell::new(Lent {
            count: 1,
            name: None,
        });
        cell.with_at(LENT_COUNT, |_| cell.with_mut(|record| record.count = 2));
    }

    #[test]
    #[should_panic(expected = "during an active with borrow")]
    fn a_field_write_under_a_whole_record_borrow_panics() {
        let cell = GlobalCell::new(Lent {
            count: 1,
            name: None,
        });
        cell.with(|_| cell.set_at(LENT_COUNT, 2));
    }

    #[test]
    #[should_panic(expected = "while with_field lends it")]
    fn an_outer_field_loan_is_found_under_an_inner_one() {
        let cell = GlobalCell::new(Lent {
            count: 1,
            name: None,
        });
        // The lent field is the outer loan; the inner one, of another
        // field, must not hide it.
        cell.with_at(LENT_COUNT, |_| {
            cell.with_at(LENT_NAME, |_| cell.set_at(LENT_COUNT, 2));
        });
    }

    #[test]
    #[should_panic(expected = "reentrant global access")]
    fn a_set_inside_with_panics() {
        let cell = GlobalCell::new(0);
        cell.with(|_| cell.set(1));
    }

    /// The emergency's mark tells loans apart by frame: one made by a frame
    /// the mark's frame was called from is above it, one made by a frame
    /// called since is below.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "Miri places locals anywhere: the mark is a property of the machine stack"
    )]
    fn a_loan_of_an_older_frame_is_above_the_mark() {
        #[inline(never)]
        fn middle(outer: &Loan) {
            let frame = 0u8;
            let mark = core::ptr::from_ref(&frame).addr();
            assert!(above(outer, mark));
            inner(mark);
        }
        #[inline(never)]
        fn inner(mark: usize) {
            let loan = Loan {
                start: 0,
                end: 1,
                outer: FREE,
            };
            assert!(!above(core::hint::black_box(&loan), mark));
        }
        let outer = Loan {
            start: 0,
            end: 1,
            outer: FREE,
        };
        middle(core::hint::black_box(&outer));
    }

    #[test]
    fn a_panicking_borrow_gives_the_cell_back() {
        let cell = GlobalCell::new(0);
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.with_mut(|_| cell.with(|_| ()));
        }));
        assert!(caught.is_err());
        // Both guards ran as the panic unwound: the cell is free again.
        cell.with_mut(|x| *x = 1);
        assert_eq!(cell.with(|x| *x), 1);
    }

    #[test]
    fn a_field_loan_ends_with_its_closure() {
        let cell = GlobalCell::new(Lent {
            count: 1,
            name: None,
        });
        // Reads nest under a loan, the same field's included.
        let both = cell.with_at(LENT_COUNT, |a| cell.with_at(LENT_COUNT, |b| *a + *b));
        assert_eq!(both, 2);
        assert_eq!(cell.with_at(LENT_COUNT, |_| cell.get_at(LENT_COUNT)), 1);
        // Released: the field and the record can be written again.
        cell.set_at(LENT_COUNT, 3);
        cell.with_mut(|record| record.name = Some(b"x".to_vec()));
        assert_eq!(cell.get_at(LENT_COUNT), 3);
    }

    #[test]
    fn the_editor_state_lock_nests_on_its_own_thread() {
        let outer = editor_state_lock();
        {
            let _inner = editor_state_lock();
            editor_state::assert_held();
        }
        editor_state::assert_held();
        drop(outer);
    }

    #[test]
    #[should_panic(expected = "must hold `editor_state_lock()`")]
    fn touching_editor_state_without_the_lock_panics() {
        editor_state::assert_held();
    }
}
