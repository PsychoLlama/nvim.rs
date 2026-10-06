//! `Partial`: a reference-counted funcref-with-arguments, as a handle.
//!
//! The refcount *is* the ownership, as it is for [`ListRef`] and
//! [`BlobRef`](super::BlobRef): [`PartialRef`] retains on `Clone`, releases
//! on `Drop`, and the last one frees through [`partial_unref`] -- the
//! teardown of `pt_argv`, `pt_dict` and `pt_func`, which lives here with
//! the rest of the handle.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::userfunc::{func_ptr_unref, func_unref};
use crate::winlayer::Live;
use ::core::ffi::{c_char, c_void};
use ::core::ptr::NonNull;

/// One reference to a [`Partial`], given back when the handle goes.
///
/// The partial half of [`ListRef`].  A handle is never null: a `VAR_PARTIAL`
/// over NULL is `TypVal::partial(None)`, which the parser leaves behind for
/// a funcref it could not build, and every reader that wants the old
/// spelling asks [`TypVal::partial_or_null`].
#[repr(transparent)]
pub struct PartialRef(NonNull<Partial>);

impl PartialRef {
    /// Take over the caller's reference to `pt`, or answer `None` for a NULL
    /// partial: a reference the caller already holds and will not release,
    /// which this handle now owns and eventually gives back.
    ///
    /// # Safety
    ///
    /// `pt` is null or points at a live partial the caller holds a
    /// reference to.
    #[inline(always)]
    pub unsafe fn owning(pt: *mut Partial) -> Option<PartialRef> {
        NonNull::new(pt).map(PartialRef)
    }

    /// Take *another* reference to `pt`: the caller keeps its own.
    ///
    /// # Safety
    ///
    /// `pt` is null or points at a live partial.
    #[inline(always)]
    pub unsafe fn retained(pt: *mut Partial) -> Option<PartialRef> {
        let at = NonNull::new(pt)?;
        // SAFETY: the caller's promise: a live partial.
        unsafe { Pt::new(pt) }.pt_refcount.retain();
        Some(PartialRef(at))
    }

    /// The partial, as the pointer most of the family still takes.
    ///
    /// A **borrow**: live only while the handle is.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Partial {
        self.0.as_ptr()
    }
}

impl ::core::ops::Deref for PartialRef {
    type Target = Partial;

    /// The partial, read: its bound arguments, dictionary and function.
    #[inline(always)]
    fn deref(&self) -> &Partial {
        // SAFETY: the handle holds a reference, so the partial is live.
        unsafe { self.0.as_ref() }
    }
}

impl Clone for PartialRef {
    /// One more owner of the same partial.
    #[inline(always)]
    fn clone(&self) -> PartialRef {
        // SAFETY: this handle names a live partial, since it holds a
        // reference to it.
        unsafe { Pt::new(self.as_ptr()) }.pt_refcount.retain();
        PartialRef(self.0)
    }
}

impl Drop for PartialRef {
    /// Give the reference back, freeing the partial with the last one.
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: this handle names a live partial, and is giving up the
        // reference that kept it so.
        unsafe { partial_unref(self.as_ptr()) };
    }
}

/// The `TypVal` readers and writers for the partial arm.
///
/// Hand-written where the scalar arms are generated, because the payload is
/// a [`PartialRef`]: it can be borrowed but never handed out, since a copy
/// of it would be a reference nobody took.
impl TypVal {
    /// The partial, or `None` unless this is a `Partial` -- including the
    /// NULL case, which answers `Some(NULL)`.
    #[inline(always)]
    pub(crate) fn as_partial(&self) -> Option<*mut Partial> {
        match self {
            TypVal::Partial(pt) => Some(
                pt.as_ref()
                    .map_or(::core::ptr::null_mut(), PartialRef::as_ptr),
            ),
            _ => None,
        }
    }

    /// The partial this value holds, or NULL unless it is a partial holding
    /// one.  A **borrow**; see [`TypVal::list_or_null`].
    #[inline(always)]
    pub(crate) fn partial_or_null(&self) -> *mut Partial {
        self.as_partial().unwrap_or(::core::ptr::null_mut())
    }

    /// A partial value over `pt`, which the value takes over.
    #[inline(always)]
    pub(crate) const fn partial(pt: Option<PartialRef>) -> TypVal {
        TypVal::Partial(::core::mem::ManuallyDrop::new(pt))
    }

    /// Overwrite this slot with `pt`, **releasing nothing**: see
    /// [`union_writers`](super::access).
    #[inline(always)]
    pub(crate) fn write_partial(&mut self, pt: Option<PartialRef>) {
        self.overwrite(TypVal::partial(pt));
    }

    /// Move the partial out of this slot, leaving a NULL one behind.
    #[inline(always)]
    pub(crate) fn take_partial(&mut self) -> Option<PartialRef> {
        match self {
            TypVal::Partial(pt) => pt.take(),
            _ => None,
        }
    }

    /// The name of the function a Funcref or partial calls; `None` for any
    /// other value, and for a name that is empty, which is how "no
    /// function" reads.
    pub(crate) fn callable_name(&self) -> Option<&::core::ffi::CStr> {
        let name = match self {
            TypVal::Func(name) => name.as_ref()?.as_cstr(),
            TypVal::Partial(_) => {
                // SAFETY: the partial is null or live, which is what
                // `partial_name` takes.
                let name = unsafe { partial_name(self.partial_or_null()) };
                // SAFETY: a partial's name is owned by the partial, or by
                // the function it holds a reference to. Either lives as
                // long as this value does.
                unsafe { crate::cstr::at_opt(name) }?
            }
            _ => return None,
        };
        (!name.is_empty()).then_some(name)
    }

    /// What a partial binds: its dictionary and its arguments. Neither for a
    /// Funcref, a NULL partial or anything else.
    pub(crate) fn partial_binding(&self) -> (Option<&Dict>, &[TypVal]) {
        let pt = self.partial_or_null();
        if pt.is_null() {
            return (None, &[]);
        }
        // SAFETY: a partial value holds a reference to a live partial, which
        // owns its dictionary reference (or none) and its `pt_argc`
        // arguments for as long as this value holds it.
        let pt = unsafe { &*pt };
        let args = match usize::try_from(pt.pt_argc) {
            // SAFETY: as above -- `pt_argv` holds `argc` values.
            Ok(argc @ 1..) => unsafe { ::core::slice::from_raw_parts(pt.pt_argv, argc) },
            _ => &[],
        };
        // SAFETY: as above.
        (unsafe { pt.pt_dict.as_ref() }, args)
    }
}

/// The function name a partial stands for: its own, its `UserFunc`'s, or the
/// empty string.
///
/// # Safety
/// `pt` must be null or valid.
pub(crate) unsafe fn partial_name(pt: *mut Partial) -> *mut c_char {
    if !pt.is_null() {
        // SAFETY: the caller's promise, and `pt` is not null.
        let pt = unsafe { Live::new(pt) };
        if !pt.pt_name.is_null() {
            return pt.pt_name;
        }
        let func = pt.pt_func;
        if !func.is_null() {
            // SAFETY: `pt_func` is a live `UserFunc` whose name is inline.
            return unsafe { &raw mut (*func).uf_name }.cast::<c_char>();
        }
    }
    c"".as_ptr().cast_mut()
}

/// Release a partial and everything it bound.
///
/// # Safety
/// `pt` must be valid and unreferenced.
unsafe fn partial_free(pt: *mut Partial) {
    // SAFETY: the caller's promise -- `pt` is a live, unreferenced partial.
    let live = unsafe { Live::new(pt) };
    if let Ok(argc @ 1..) = usize::try_from(live.pt_argc) {
        // SAFETY: `pt_argv` holds `pt_argc` typvals this partial owns.
        for tv in unsafe { ::core::slice::from_raw_parts_mut(live.pt_argv, argc) } {
            tv_clear(tv);
        }
    }
    unsafe { xfree(live.pt_argv.cast::<c_void>()) };
    unsafe { tv_dict_unref(live.pt_dict) };
    if !live.pt_name.is_null() {
        unsafe { func_unref(live.pt_name) };
        unsafe { xfree(live.pt_name.cast::<c_void>()) };
    } else {
        unsafe { func_ptr_unref(live.pt_func) };
    }
    unsafe { xfree(pt.cast::<c_void>()) };
}

/// Drop one reference to a partial, freeing it at zero.
///
/// # Safety
/// `pt` must be null or valid.
pub(crate) unsafe fn partial_unref(pt: *mut Partial) {
    if pt.is_null() {
        return;
    }
    // SAFETY: the caller's promise, and `pt` is not null.
    if unsafe { (*pt).pt_refcount.release() } <= 0 {
        unsafe { partial_free(pt) };
    }
}
