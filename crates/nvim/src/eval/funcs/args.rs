//! The call frame a builtin function is handed, and the one `unsafe` block
//! that turns it into something checked.
//!
//! Every `f_*` body receives the same three C arguments: a pointer into the
//! caller's argument array, a pointer to the return value, and the row's
//! payload from the generated table. The array is a `[typval_T;
//! MAX_FUNC_ARGS + 1]` owned by the evaluator, with a `VAR_UNKNOWN`
//! terminator written at the supplied argument count — so reading any index
//! up to [`MAX_ARGS`] is in bounds whatever the caller passed, and reading
//! past the terminator is how a builtin tests for an optional argument.
//! That is the whole contract, and [`Args`] is it, expressed once.
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::{VAR_UNKNOWN, VarType, typval_T};
use core::marker::PhantomData;

/// The size of the evaluator's argument buffer, minus its terminator slot.
/// `MAX_FUNC_ARGS` in the C; both dispatchers declare `typval_T argv[MAX +
/// 1]`, so indices `0..=MAX_ARGS` are always readable.
pub(crate) const MAX_ARGS: usize = 20;

/// A builtin's argument list.
///
/// Indexing is deliberately total: `Args` answers for every slot the
/// evaluator's buffer has, and a slot past the supplied count reads as
/// `VAR_UNKNOWN`, which is exactly the test the C bodies write.
#[derive(Clone, Copy)]
pub(crate) struct Args<'a> {
    base: *mut typval_T,
    life: PhantomData<&'a mut typval_T>,
}

impl<'a> Args<'a> {
    /// Wrap the dispatcher's argument array.
    ///
    /// # Safety
    ///
    /// `base` must point at an array of at least `MAX_ARGS + 1` `typval_T`
    /// with a `VAR_UNKNOWN` terminator at or before the last slot, valid for
    /// reads and writes for `'a`. The two `call_internal_*` dispatchers are
    /// the only callers and both satisfy this.
    pub(crate) unsafe fn new(base: *mut typval_T) -> Self {
        Args {
            base,
            life: PhantomData,
        }
    }

    /// A raw pointer to argument `i`, for the typval entry points that still
    /// take one.
    pub(crate) fn ptr(&self, i: usize) -> *mut typval_T {
        debug_assert!(i <= MAX_ARGS);
        // SAFETY: the constructor's obligation covers every index through
        // `MAX_ARGS`; no dereference happens here.
        unsafe { self.base.add(i) }
    }

    /// Argument `i`.
    pub(crate) fn get(&self, i: usize) -> &'a typval_T {
        // SAFETY: in bounds by the constructor's obligation, and `'a` is the
        // borrow the frame was built from.
        unsafe { &*self.ptr(i) }
    }

    /// Argument `i`, mutably. Builtins that write back through an argument
    /// (`rand()` advancing its seed list, the `getpos()` family) go through
    /// here.
    pub(crate) fn get_mut(&mut self, i: usize) -> &'a mut typval_T {
        // SAFETY: as `get`, and `&mut self` is what keeps the reference
        // exclusive.
        unsafe { &mut *self.ptr(i) }
    }

    /// The type tag of argument `i`, or `VAR_UNKNOWN` past the last one.
    pub(crate) fn ty(&self, i: usize) -> VarType {
        self.get(i).v_type
    }

    /// Whether argument `i` was supplied.
    pub(crate) fn has(&self, i: usize) -> bool {
        self.ty(i) != VAR_UNKNOWN
    }
}

/// Open a builtin: bind the argument list and the return value.
///
/// Written as a macro rather than a function so that the `unsafe` block
/// carrying the dispatcher's contract appears once per family instead of
/// once per builtin — there are around two hundred of them and the
/// obligation is identical every time.
macro_rules! frame {
    ($argvars:expr, $rettv:expr) => {
        // SAFETY: the caller is `call_internal_func` or
        // `call_internal_method`, which own the argument buffer and the
        // return value for the duration of this call. See `Args::new`.
        unsafe { ($crate::eval::funcs::args::Args::new($argvars), &mut *$rettv) }
    };
}

pub(crate) use frame;
