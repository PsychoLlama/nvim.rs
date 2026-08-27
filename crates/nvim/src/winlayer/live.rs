//! [`Live<T>`]: the one home for the `Copy` newtype over a `*mut T` that
//! makes field access ordinary safe code.
//!
//! [`Win`](crate::winlayer::Win), [`Buf`](crate::winlayer::Buf) and their
//! siblings each wrap *one* named pointee and carry a family of projections
//! with them. The same shape kept being reinvented for the plain structs the
//! transpiled editor passes around by pointer — `oparg_T`, `cmdarg_T`,
//! `exarg_T` — where all that is wanted is the half that pays: **construction
//! is the unsafe step, and every `(*p).field` after it is checked code**.
//! Three phase-23 slices invented it independently before it was given a
//! home; this is the home.
//!
//! ```ignore
//! pub(crate) type Op = Live<oparg_T>;   // ops/mod.rs
//!
//! let mut oap = unsafe { Op::new(raw) };   // the promise, once
//! oap.motion_force = 0;                    // ordinary code, everywhere after
//! ```
//!
//! # What the promise is, and what it is not
//!
//! `Live<T>` records that *the caller who built it promised the pointee stays
//! live for as long as the value is used*. It is a **record of a promise, not
//! evidence for one**. Nothing here checks anything: `Live::new` reads no
//! memory, and neither does holding, copying or passing one — which is the
//! point, because the editor hands these pointers around across calls that
//! may free the pointee, and *holding a pointer must stay a non-read*
//! (p23-5).
//!
//! So a `Live<T>` is not a liveness proof and must not be treated as one. For
//! the objects that have an identity — windows and buffers — the re-entry
//! rule in [`crate::winlayer`]'s module docs still applies in full: take a
//! [`WinId`](crate::winlayer::WinId)/[`BufId`](crate::winlayer::BufId) before
//! a call that may fire an autocommand or enter Lua, and ask the registry
//! afterwards. For the stack-allocated structs this type is mostly used for
//! (`oparg_T` and friends), the promise is discharged by the frame that owns
//! them outliving the call.
//!
//! # Why [`Deref`], and why not `&mut *p`
//!
//! [`Deref`]/[`DerefMut`] hand out a borrow that lasts exactly as long as the
//! field access that asked for it, so a `Live<T>` never holds one across a
//! call. Taking `&mut *p` once at the head of a body instead — the tempting
//! shorter rewrite — is **unsound here**: a `&mut` parameter is `noalias` to
//! LLVM, and the editor reads the same `oparg_T` through `current_oap` while
//! `run_operator` is away in `edit()`, in `'operatorfunc'` or in a filter.
//! Phase 22's ruling 6 — nothing an autocommand re-enters holds a `&mut` —
//! is a property of this API rather than of review.
//!
//! The two `unsafe` dereferences below are the whole cost, once, for every
//! newtype in the crate.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ops::{Deref, DerefMut};

use crate::types::{CmdlineInfo, exarg_T};

/// A `*mut T` the caller has promised is live, with checked field access.
///
/// See the module docs: this is a record of that promise, not a proof of it.
/// A family gives itself a name for its own pointee —
/// `pub(crate) type Op = Live<oparg_T>;` — and hangs whatever extra
/// projections it needs off `impl Op`, which is an inherent impl on a local
/// type and so is allowed in any module of this crate.
#[repr(transparent)]
pub(crate) struct Live<T>(*mut T);

// Hand-written rather than derived: `derive` would demand `T: Copy`, and the
// pointee is a transpiled struct that need not be `Copy` for its *pointer* to
// be.
impl<T> Clone for Live<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Live<T> {}

// Two handles are equal when they name the same object, which is what
// [`Win`](crate::winlayer::Win) and [`Buf`](crate::winlayer::Buf) derive for
// themselves. Hand-written for `Clone`'s reason: `derive` would demand
// `T: PartialEq`, and the pointee need not be comparable for its *address*
// to be.
impl<T> PartialEq for Live<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.0, other.0)
    }
}

impl<T> Eq for Live<T> {}

impl<T> Live<T> {
    /// # Safety
    /// `ptr` must stay a live `T` for as long as the value is used.
    #[inline(always)]
    pub(crate) const unsafe fn new(ptr: *mut T) -> Self {
        Self(ptr)
    }

    /// The pointer back, for the callees that still take one.
    #[inline(always)]
    pub(crate) fn raw(self) -> *mut T {
        self.0
    }

    /// The address of a field, `offset` bytes in, **without reading the
    /// object**.
    ///
    /// [`Win::cursor`](crate::winlayer::Win::cursor)'s trick, shared: a
    /// field's address is the object's plus a constant, so saying where one
    /// is needs no dereference. Feed it `offset_of!(T, field)`; the result is
    /// live exactly as long as the `Live<T>` it came from, which is why
    /// wrapping it in a `Pos`/`Line` is still the caller's unsafe step.
    #[inline(always)]
    pub(crate) fn field_ptr<F>(self, offset: usize) -> *mut F {
        self.0.wrapping_byte_add(offset).cast()
    }
}

impl<T> Deref for Live<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        // SAFETY: the constructor's promise -- a live `T`. The borrow lasts
        // only as long as the field access that asked for it.
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for Live<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: as [`Live::deref`].
        unsafe { &mut *self.0 }
    }
}

// ---------------------------------------------------------------------------
// The shared aliases.
//
// A pointee two or more families pass around gets its name here rather than in
// whichever module needed it first: `Ea` was declared twice and wanted a
// third time, and the private copies shadowed each other. A family that owns
// its pointee (`Op`, `Sug`, `Df`) still names it at home.

/// The Ex command being run, whose caller has promised it outlives the value.
///
/// The promise is discharged by the `do_cmdline` frame that owns the
/// `exarg_T`: it outlives every command run out of it. Wrapping is the unsafe
/// step, once per entry point; every `(*eap).field` after it is ordinary
/// checked code.
pub(crate) type Ea = Live<exarg_T>;

/// The command line being edited, whose caller has promised it outlives the
/// value.
///
/// The promise is discharged by the *place* it names not moving: either the
/// `ccline` global cell or one boxed entry of the saved stack. What moves is
/// the value inside it, which is why `ex_getln/` derives a `Cc` at each use
/// rather than holding one across a call that can re-enter command-line
/// mode. Its projections are in [`crate::ex_getln`].
pub(crate) type Cc = Live<CmdlineInfo>;
