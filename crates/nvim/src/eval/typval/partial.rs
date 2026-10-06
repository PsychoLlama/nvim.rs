//! `Partial`: a reference-counted funcref-with-arguments, as a handle.
//!
//! The refcount *is* the ownership, as it is for [`ListRef`] and
//! [`BlobRef`](super::BlobRef): [`PartialRef`] retains on `Clone`, releases
//! on `Drop`, and the last one frees through [`partial_free`] -- the
//! teardown of `pt_argv`, `pt_dict` and `pt_func`, which lives here with
//! the rest of the handle.

#![deny(unsafe_op_in_unsafe_fn)]
// Two reads through `pt_func`, the user function's raw pointer: its name and
// its release. They go with that pointer.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::userfunc::{func_ptr_unref, func_unref_name, uf_name_ptr};
use crate::types::Refcount;
use ::core::ffi::CStr;

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

    /// The partial this value holds, borrowed -- `None` for every other
    /// kind and for a NULL partial. See [`TypVal::list_ref`].
    #[inline(always)]
    pub(crate) fn partial_ref(&self) -> Option<&Partial> {
        self.partial_shared().map(|pt| &**pt)
    }

    /// The handle this value holds, borrowed: what to keep across a call
    /// that can run user code. See [`TypVal::list_shared`].
    #[inline(always)]
    pub(crate) fn partial_shared(&self) -> Option<&PartialRef> {
        match self {
            TypVal::Partial(pt) => pt.as_ref(),
            _ => None,
        }
    }

    /// The name of the function a Funcref or partial calls; `None` for any
    /// other value, and for a name that is empty, which is how "no
    /// function" reads.
    pub(crate) fn callable_name(&self) -> Option<&CStr> {
        let name = match self {
            TypVal::Func(name) => name.as_ref()?.as_cstr(),
            TypVal::Partial(pt) => pt.as_deref().map_or(c"", partial_name),
            _ => return None,
        };
        (!name.is_empty()).then_some(name)
    }

    /// What a partial binds: its dictionary and its arguments. Neither for a
    /// Funcref, a NULL partial or anything else.
    pub(crate) fn partial_binding(&self) -> (Option<&Dict>, &[TypVal]) {
        self.partial_ref()
            .map_or((None, &[]), |pt| (pt.pt_dict.as_deref(), &pt.pt_argv))
    }
}

impl Partial {
    /// A partial binding nothing and calling nothing yet, for
    /// [`PartialRef::new`] to build on.
    pub(crate) const EMPTY: Partial = Partial {
        pt_refcount: Refcount::ZERO,
        pt_copy_id: 0,
        pt_name: None,
        pt_func: ::core::ptr::null_mut(),
        pt_auto: false,
        pt_argv: Vec::new(),
        pt_dict: None,
    };
}

/// The function name a partial stands for: its own, its `UserFunc`'s, or the
/// empty string.
pub(crate) fn partial_name(pt: &Partial) -> &CStr {
    if let Some(name) = &pt.pt_name {
        return name.as_cstr();
    }
    if pt.pt_func.is_null() {
        return c"";
    }
    // SAFETY: the partial holds a reference to `pt_func`, a live `UserFunc`
    // whose name is inline, NUL-terminated, and lives as long as it does.
    // The user function is still a raw pointer (its own slice's).
    unsafe { CStr::from_ptr(uf_name_ptr(pt.pt_func)) }
}

/// Release what the last reference to a partial bound, in upstream's order:
/// the arguments, then the dictionary, then the function.
pub(super) fn partial_free(partial: Partial) {
    let Partial {
        pt_name,
        pt_func,
        pt_argv,
        pt_dict,
        ..
    } = partial;
    drop(pt_argv);
    drop(pt_dict);
    match pt_name {
        Some(name) => func_unref_name(name.as_cstr()),
        // SAFETY: the reference the partial held on its function.
        None => unsafe { func_ptr_unref(pt_func) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::funcs::f_function;
    use crate::eval::list::string_tv;
    use crate::global_cell::editor_state_lock;
    use crate::types::EvalFuncData;

    /// `function(name, args, dict)`, as the builtin answers it.  `len` is a
    /// builtin, so the name resolves without defining anything, and is not
    /// one whose name is reference-counted.
    fn function(args: Option<ListRef>, dict: Option<DictRef>) -> TypVal {
        let mut argv = vec![string_tv(b"len")];
        if args.is_some() {
            argv.push(TypVal::list(args));
        }
        if dict.is_some() {
            argv.push(TypVal::dict(dict));
        }
        let mut result = TypVal::Number(0);
        f_function(&argv, &mut result, EvalFuncData::None);
        for tv in &mut argv {
            tv_clear(tv);
        }
        result
    }

    /// A partial binding a dictionary and an argument that is itself a
    /// container: copying it shares the one partial, and clearing both gives
    /// every reference back.
    #[test]
    fn a_copied_partial_gives_its_bindings_back_when_both_are_cleared() {
        let _serial = editor_state_lock();
        let d = tv_dict_alloc();
        let inner = tv_list_alloc(1);
        let (d_before, i_before) = (d.dv_refcount.get(), inner.lv_refcount.get());
        let mut args = tv_list_alloc(2);
        args.push_number(1);
        args.push_list(Some(inner.clone()));

        let mut pt = function(Some(args), Some(d.clone()));
        let held = pt
            .partial_shared()
            .expect("function() answered a partial")
            .clone();
        assert_eq!(held.pt_refcount.get(), 2);
        assert_eq!(held.pt_argv.len(), 2);
        assert!(held.pt_dict.as_ref().is_some_and(|pd| pd.ptr_eq(&d)));
        assert!(!held.pt_auto, "an explicit dict is not auto-bound");
        assert_eq!(
            held.pt_name.as_ref().map(|n| n.as_bytes()),
            Some(&b"len"[..])
        );
        let (dict, argv) = pt.partial_binding();
        assert!(dict.is_some_and(|bound| ::core::ptr::eq(bound, &*d)));
        assert_eq!(argv[0].number_or_zero(), 1);
        assert!(
            argv[1].list_shared().is_some_and(|l| l.ptr_eq(&inner)),
            "an argument is shared, not copied"
        );
        assert_eq!(d.dv_refcount.get(), d_before + 1);
        assert_eq!(inner.lv_refcount.get(), i_before + 1);

        let mut copy = TypVal::Unknown;
        tv_copy(&pt, &mut copy);
        assert!(
            copy.partial_shared().is_some_and(|c| c.ptr_eq(&held)),
            "a copy shares the partial"
        );
        assert_eq!(held.pt_refcount.get(), 3);
        assert_eq!(d.dv_refcount.get(), d_before + 1);

        tv_clear(&mut pt);
        assert_eq!(held.pt_refcount.get(), 2);
        tv_clear(&mut copy);
        drop(held);
        assert_eq!(d.dv_refcount.get(), d_before);
        assert_eq!(inner.lv_refcount.get(), i_before);
    }

    /// `function(P, [x])` builds a second partial: the bound arguments are
    /// copied in front of the new ones, the dictionary stays bound, and the
    /// first partial is untouched.
    #[test]
    fn rebinding_a_partial_appends_its_arguments() {
        let _serial = editor_state_lock();
        let d = tv_dict_alloc();
        let mut first_args = tv_list_alloc(1);
        first_args.push_number(1);
        let mut first = function(Some(first_args), Some(d.clone()));

        let mut more = tv_list_alloc(1);
        more.push_number(2);
        let mut argv = [TypVal::Unknown, TypVal::list(Some(more))];
        tv_copy(&first, &mut argv[0]);
        let mut second = TypVal::Number(0);
        f_function(&argv, &mut second, EvalFuncData::None);
        for tv in &mut argv {
            tv_clear(tv);
        }

        let (a, b) = (first.partial_shared(), second.partial_shared());
        assert!(!a.zip(b).is_some_and(|(a, b)| a.ptr_eq(b)));
        let (dict, args) = second.partial_binding();
        assert!(dict.is_some_and(|bound| ::core::ptr::eq(bound, &*d)));
        let numbers: Vec<_> = args.iter().map(TypVal::number_or_zero).collect();
        assert_eq!(numbers, [1, 2]);
        assert_eq!(first.partial_binding().1.len(), 1);
        // The handle, and one per partial.
        assert_eq!(d.dv_refcount.get(), 3);

        tv_clear(&mut first);
        tv_clear(&mut second);
        assert_eq!(d.dv_refcount.get(), 1);
    }

    /// A partial whose bound dictionary holds the partial: neither count
    /// reaches zero by itself, and the collector's two passes free both.
    #[test]
    fn a_partial_bound_to_the_dict_holding_it_is_freed_by_the_collector() {
        let _serial = editor_state_lock();
        let d = tv_dict_alloc();
        let mut pt = function(None, Some(d.clone()));
        let held = pt.partial_shared().expect("a partial").clone();
        // The dictionary's item takes a reference of its own.
        d.edit().add_tv(b"p", &pt).expect("a key used once");
        tv_clear(&mut pt);
        assert_eq!(held.pt_refcount.get(), 2);
        drop(held);
        // The handle and the partial.
        assert_eq!(d.dv_refcount.get(), 2);
        let view = ::core::mem::ManuallyDrop::new(d.clone());
        drop(d);
        assert_eq!(view.dv_refcount.get(), 2, "the cycle keeps it alive");

        tv_in_free_unref_items.set(true);
        // Pass 1: the item goes, the partial goes with it, and the partial
        // gives back the dictionary's last reference -- which must not
        // free the dictionary under the walk.
        tv_dict_free_contents(&view);
        assert_eq!(view.dv_refcount.get(), 1);
        tv_in_free_unref_items.set(false);
        // Pass 2: the view's own reference, the last.
        drop(::core::mem::ManuallyDrop::into_inner(view));
    }

    /// A NULL partial has no name and no binding.
    #[test]
    fn a_null_partial_names_nothing() {
        let _serial = editor_state_lock();
        let pt = TypVal::partial(None);
        assert!(pt.partial_ref().is_none());
        assert_eq!(pt.callable_name(), None);
        let (dict, args) = pt.partial_binding();
        assert!(dict.is_none());
        assert!(args.is_empty());
        assert_eq!(partial_name(&Partial::EMPTY).to_bytes(), b"");
    }

    /// The partial's own size, which every bound callable pays.
    #[test]
    fn a_partial_is_sixty_four_bytes() {
        assert_eq!(::core::mem::size_of::<Partial>(), 64);
    }
}
