//! The `nothing` sink: the deep free [`tv_clear`] uses.
//!
//! Upstream instantiates `typval_encode.c.h` a seventh time here, with every
//! conversion hook releasing what it is handed and writing nothing anywhere.
//! That buys `tv_clear` an *iterative* deep free: a container that references
//! itself, or one nested a thousand deep, is released without recursing.
//!
//! This is the sink that makes the walk's inline frame budget matter — it runs
//! on every container the interpreter drops, so a heap allocation per walk
//! would be a heap allocation per free.
//!
//! Three of its hooks edit the walk's own state rather than emitting anything:
//!
//! - `conv_real_list_after_start` and `conv_real_dict_after_start` are handed
//!   the frame that was just pushed for the container.  When the container has
//!   another owner they drop one reference and make the frame *look drained*
//!   (`li = NULL` / `todo = 0`), then answer [`Flow::Stop`]; the walk pops it
//!   on the next pass and the items are never visited.
//! - `conv_empty_dict` and `conv_dict_end` write through the `dictp` lvalue,
//!   which is where the dictionary pointer *lives* — `&tv->vval.v_dict`, or
//!   `&pt->pt_dict` for a partial's self dictionary.
//!
//! [`tv_clear`]: super::value::tv_clear

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::{
    DictSlot, Ls, Pt, VAR_PARTIAL, func_unref, partial_unref, tv_blob_unref, tv_dict_unref,
    tv_empty_string, tv_list_unref,
};
use crate::eval::typval_encode::{
    ConvFrame, ConvPath, ConvType, Flow, Frame, TypvalSink, encode_typval,
};
use crate::memory::xfree;
use crate::types::{Blob, Float, TypVal, int64_t, kBoolVarFalse, kSpecialVarNull, size_t};

/// A sink with no state: everything it does, it does to the value it is
/// handed.
struct NothingSink;

/// The slot the walk is standing on.
///
/// Every hook this sink implements *writes* to it, and the walk only leaves
/// it out for the two frames a partial pushes -- which reach
/// [`TypvalSink::conv_list_end`] and nothing else here.
fn slot(tv: Option<&mut TypVal>) -> &mut TypVal {
    tv.expect("the nothing sink is only handed frames that have a value")
}

impl TypvalSink for NothingSink {
    /// `{_TYPE, _VAL}` dictionaries are a msgpack round-trip device; to a free
    /// they are two ordinary keys and have to be freed as such.
    const ALLOW_SPECIALS: bool = false;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_nothing_convert_one_value()";

    fn conv_nil(&mut self, tv: Option<&mut TypVal>) {
        slot(tv).write_special(kSpecialVarNull);
    }

    fn conv_bool(&mut self, tv: Option<&mut TypVal>, _num: bool) {
        slot(tv).write_boolean(kBoolVarFalse);
    }

    fn conv_number(&mut self, tv: Option<&mut TypVal>, _num: int64_t) {
        slot(tv).write_number(0);
    }

    fn conv_float(&mut self, tv: Option<&mut TypVal>, _flt: Float) -> Flow {
        slot(tv).write_float(0.0);
        Flow::Go
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_string(
        &mut self,
        tv: Option<&mut TypVal>,
        buf: *mut c_char,
        _len: size_t,
    ) -> Flow {
        unsafe { xfree(buf.cast::<c_void>()) };
        slot(tv).write_string(ptr::null_mut());
        Flow::Go
    }

    /// Nothing to do, and it must *not* fall back on [`Self::conv_string`]:
    /// the only buffer that reaches this hook is a dictionary key, which the
    /// dictionary owns and frees with itself.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_str_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_str_string(
        &mut self,
        _tv: Option<&mut TypVal>,
        _buf: *mut c_char,
        _len: size_t,
    ) -> Flow {
        Flow::Go
    }

    /// Unreachable: an `ext` value only comes out of a special dictionary,
    /// which this sink refuses.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_ext_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: Option<&mut TypVal>,
        _buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        Flow::Go
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_blob`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_blob(&mut self, tv: Option<&mut TypVal>, _blob: *const Blob, _len: c_int) {
        let tv = slot(tv);
        // SAFETY: the typval's own blob.
        unsafe { tv_blob_unref(tv.blob_or_null()) };
        tv.write_blob(ptr::null_mut());
    }

    /// A funcref releases its name here and is done.  A partial with another
    /// owner drops one reference and stops, so the walk never descends into
    /// arguments that are not ours to free.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_func_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_func_start(
        &mut self,
        tv: Option<&mut TypVal>,
        fun: *mut c_char,
        _prefix: &'static CStr,
        _path: &ConvPath,
    ) -> Flow {
        let tv = slot(tv);
        if tv.v_type() == VAR_PARTIAL {
            let pt = tv.partial_or_null();
            // SAFETY: the typval's own partial.
            let mut part = unsafe { Pt::new(pt) };
            if !pt.is_null() && part.pt_refcount.is_shared() {
                part.pt_refcount.release();
                tv.write_partial(ptr::null_mut());
                return Flow::Stop;
            }
        } else {
            unsafe { func_unref(fun) };
            if !ptr::eq(fun, tv_empty_string.get()) {
                unsafe { xfree(fun.cast::<c_void>()) };
            }
            tv.write_func_name(ptr::null_mut());
        }
        Flow::Go
    }

    /// The last reference to a partial, its arguments and self dictionary
    /// already released by the frames the walk drained.
    ///
    fn conv_func_end(&mut self, tv: Option<&mut TypVal>, copyid: c_int) {
        let tv = slot(tv);
        if tv.v_type() != VAR_PARTIAL {
            return;
        }
        let pt = tv.partial_or_null();
        if pt.is_null() {
            return;
        }
        debug_assert!(
            unsafe { (*pt).pt_dict }.is_null() || unsafe { (*(*pt).pt_dict).dv_copy_id } == copyid
        );
        unsafe { (*pt).pt_dict = ptr::null_mut() };
        // SAFETY: the typval's own partial.
        let mut part = unsafe { Pt::new(pt) };
        part.pt_argc = 0;
        debug_assert!(!part.pt_refcount.is_shared());
        unsafe { partial_unref(pt) };
        tv.write_partial(ptr::null_mut());
    }

    /// Nothing to announce; the frame surgery below is where the list is
    /// either released or skipped.
    ///
    fn conv_list_start(&mut self, _tv: Option<&mut TypVal>, _len: c_int) -> Flow {
        Flow::Go
    }

    fn conv_dict_start(&mut self, _tv: Option<&mut TypVal>, _len: size_t) -> Flow {
        Flow::Go
    }

    fn conv_empty_list(&mut self, tv: Option<&mut TypVal>) {
        let tv = slot(tv);
        // SAFETY: the typval's own list.
        unsafe { tv_list_unref(tv.list_or_null()) };
        tv.write_list(ptr::null_mut());
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_empty_dict`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_empty_dict(&mut self, dictp: Option<DictSlot>) {
        // Upstream asserts the lvalue is a real one.  `None` is a special
        // map's `_VAL`, which cannot reach a sink that refuses specials.
        debug_assert!(dictp.is_some());
        if let Some(dictp) = dictp {
            unsafe { tv_dict_unref(dictp.get()) };
            unsafe { dictp.clear() };
        }
    }

    /// Frame surgery: a list with another owner loses one reference and its
    /// frame is left looking drained, so the walk pops it without visiting a
    /// single item.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_real_list_after_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_real_list_after_start(
        &mut self,
        tv: Option<&mut TypVal>,
        frame: &mut ConvFrame,
    ) -> Flow {
        let tv = slot(tv);
        let list = tv.list_or_null();
        // SAFETY: the typval's own list.
        let mut ls = unsafe { Ls::new(list) };
        if ls.lv_refcount.is_shared() {
            ls.lv_refcount.release();
            tv.write_list(ptr::null_mut());
            // Always a `List`: the walk calls this straight after pushing
            // one for this very value.
            if let Frame::List { li, .. } = &mut frame.frame {
                *li = ptr::null_mut();
            }
            return Flow::Stop;
        }
        Flow::Go
    }

    fn conv_list_end(&mut self, tv: Option<&mut TypVal>) {
        // `None` is a partial's argument list, which has no `TypVal` of its
        // own; `conv_func_end` releases the partial that owns it.
        let Some(tv) = tv else { return };
        // SAFETY: the typval's own list.
        unsafe { tv_list_unref(tv.list_or_null()) };
        tv.write_list(ptr::null_mut());
    }

    /// The dictionary counterpart of [`Self::conv_real_list_after_start`].
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_real_dict_after_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_real_dict_after_start(
        &mut self,
        dictp: Option<DictSlot>,
        frame: &mut ConvFrame,
    ) -> Flow {
        if let Some(dictp) = dictp
            && unsafe { (*dictp.get()).dv_refcount }.is_shared()
        {
            unsafe { (*dictp.get()).dv_refcount.release() };
            unsafe { dictp.clear() };
            if let Frame::Dict { todo, .. } = &mut frame.frame {
                *todo = 0;
            }
            return Flow::Stop;
        }
        Flow::Go
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_end`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_end(&mut self, dictp: Option<DictSlot>) {
        if let Some(dictp) = dictp {
            unsafe { tv_dict_unref(dictp.get()) };
            unsafe { dictp.clear() };
        }
    }

    /// Already been here, so this reference is one the container holds on
    /// itself.  Answering `Go` means "handled": the walk stops converting the
    /// value, and the reference count the enclosing container drops is what
    /// frees it.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_recurse`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_recurse(
        &mut self,
        _val: *mut c_void,
        _conv_type: ConvType,
        _path: &ConvPath,
    ) -> Flow {
        Flow::Go
    }
}

/// Deep-free whatever `tv` holds.
///
/// Answers whether the walk ran to completion, which for this sink is always
/// true — it has no failing hook.
///
/// # Safety
/// `tv` must point at a live typval, and `objname` be NUL-terminated.  Note
/// that `objname` is never read: no hook here reports anything.
pub(crate) unsafe fn encode_vim_to_nothing(tv: *mut TypVal, objname: *const c_char) -> bool {
    unsafe { encode_typval(&mut NothingSink, tv, objname) }
}
