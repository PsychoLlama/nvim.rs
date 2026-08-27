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

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::{
    Ls, Pt, Tv, VAR_PARTIAL, VarLock, func_unref, partial_unref, tv_blob_unref, tv_dict_unref,
    tv_empty_string, tv_list_unref,
};
use crate::eval::typval_encode::{
    ConvFrame, ConvPath, ConvType, Flow, Frame, TypvalSink, encode_typval,
};
use crate::memory::xfree;
use crate::types::{
    blob_T, dict_T, float_T, int64_t, kBoolVarFalse, kSpecialVarNull, size_t, typval_T,
};

/// A sink with no state: everything it does, it does to the value it is
/// handed.
struct NothingSink;

impl TypvalSink for NothingSink {
    /// `{_TYPE, _VAL}` dictionaries are a msgpack round-trip device; to a free
    /// they are two ordinary keys and have to be freed as such.
    const ALLOW_SPECIALS: bool = false;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_nothing_convert_one_value()";

    unsafe fn conv_nil(&mut self, tv: *mut typval_T) {
        // SAFETY: the walk's live typval.
        let mut val = unsafe { Tv::new(tv) };
        val.vval.v_special = kSpecialVarNull;
        val.v_lock = VarLock::Unlocked;
    }

    unsafe fn conv_bool(&mut self, tv: *mut typval_T, _num: bool) {
        // SAFETY: the walk's live typval.
        let mut val = unsafe { Tv::new(tv) };
        val.vval.v_bool = kBoolVarFalse;
        val.v_lock = VarLock::Unlocked;
    }

    unsafe fn conv_number(&mut self, tv: *mut typval_T, _num: int64_t) {
        // SAFETY: the walk's live typval.
        let mut val = unsafe { Tv::new(tv) };
        val.vval.v_number = 0;
        val.v_lock = VarLock::Unlocked;
    }

    unsafe fn conv_float(&mut self, tv: *mut typval_T, _flt: float_T) -> Flow {
        // SAFETY: the walk's live typval.
        let mut val = unsafe { Tv::new(tv) };
        val.vval.v_float = 0.0;
        val.v_lock = VarLock::Unlocked;
        Flow::Go
    }

    unsafe fn conv_string(&mut self, tv: *mut typval_T, buf: *mut c_char, _len: size_t) -> Flow {
        unsafe { xfree(buf.cast::<c_void>()) };
        unsafe { (*tv).vval.v_string = ptr::null_mut() };
        unsafe { (*tv).v_lock = VarLock::Unlocked };
        Flow::Go
    }

    /// Nothing to do, and it must *not* fall back on [`Self::conv_string`]:
    /// the only buffer that reaches this hook is a dictionary key, which the
    /// dictionary owns and frees with itself.
    unsafe fn conv_str_string(
        &mut self,
        _tv: *mut typval_T,
        _buf: *mut c_char,
        _len: size_t,
    ) -> Flow {
        Flow::Go
    }

    /// Unreachable: an `ext` value only comes out of a special dictionary,
    /// which this sink refuses.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: *mut typval_T,
        _buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        Flow::Go
    }

    unsafe fn conv_blob(&mut self, tv: *mut typval_T, _blob: *const blob_T, _len: c_int) {
        unsafe { tv_blob_unref((*tv).vval.v_blob) };
        unsafe { (*tv).vval.v_blob = ptr::null_mut() };
        unsafe { (*tv).v_lock = VarLock::Unlocked };
    }

    /// A funcref releases its name here and is done.  A partial with another
    /// owner drops one reference and stops, so the walk never descends into
    /// arguments that are not ours to free.
    unsafe fn conv_func_start(
        &mut self,
        tv: *mut typval_T,
        fun: *mut c_char,
        _prefix: &'static CStr,
        _path: &ConvPath,
    ) -> Flow {
        // SAFETY: the walk's live typval.
        let mut val = unsafe { Tv::new(tv) };
        val.v_lock = VarLock::Unlocked;
        if val.v_type == VAR_PARTIAL {
            let pt = val.partial();
            // SAFETY: the typval's own partial.
            let mut part = unsafe { Pt::new(pt) };
            if !pt.is_null() && part.pt_refcount.is_shared() {
                part.pt_refcount.release();
                unsafe { (*tv).vval.v_partial = ptr::null_mut() };
                return Flow::Stop;
            }
        } else {
            unsafe { func_unref(fun) };
            if !ptr::eq(fun, tv_empty_string.get()) {
                unsafe { xfree(fun.cast::<c_void>()) };
            }
            unsafe { (*tv).vval.v_string = ptr::null_mut() };
        }
        Flow::Go
    }

    /// The last reference to a partial, its arguments and self dictionary
    /// already released by the frames the walk drained.
    unsafe fn conv_func_end(&mut self, tv: *mut typval_T, copyid: c_int) {
        // SAFETY: the walk's live typval.
        let val = unsafe { Tv::new(tv) };
        if val.v_type != VAR_PARTIAL {
            return;
        }
        let pt = val.partial();
        if pt.is_null() {
            return;
        }
        debug_assert!(
            unsafe { (*pt).pt_dict }.is_null() || unsafe { (*(*pt).pt_dict).dv_copyID } == copyid
        );
        unsafe { (*pt).pt_dict = ptr::null_mut() };
        // SAFETY: the typval's own partial.
        let mut part = unsafe { Pt::new(pt) };
        part.pt_argc = 0;
        debug_assert!(!part.pt_refcount.is_shared());
        unsafe { partial_unref(pt) };
        unsafe { (*tv).vval.v_partial = ptr::null_mut() };
        debug_assert!(val.v_lock == VarLock::Unlocked);
    }

    /// Nothing to announce; the frame surgery below is where the list is
    /// either released or skipped.
    unsafe fn conv_list_start(&mut self, _tv: *mut typval_T, _len: c_int) -> Flow {
        Flow::Go
    }

    unsafe fn conv_dict_start(&mut self, _tv: *mut typval_T, _len: size_t) -> Flow {
        Flow::Go
    }

    unsafe fn conv_empty_list(&mut self, tv: *mut typval_T) {
        unsafe { tv_list_unref((*tv).vval.v_list) };
        unsafe { (*tv).vval.v_list = ptr::null_mut() };
        unsafe { (*tv).v_lock = VarLock::Unlocked };
    }

    unsafe fn conv_empty_dict(&mut self, tv: *mut typval_T, dictp: Option<*mut *mut dict_T>) {
        // Upstream asserts the lvalue is a real one.  `None` is a special
        // map's `_VAL`, which cannot reach a sink that refuses specials.
        debug_assert!(dictp.is_some());
        if let Some(dictp) = dictp {
            unsafe { tv_dict_unref(*dictp) };
            unsafe { *dictp = ptr::null_mut() };
        }
        if !tv.is_null() {
            unsafe { (*tv).v_lock = VarLock::Unlocked };
        }
    }

    /// Frame surgery: a list with another owner loses one reference and its
    /// frame is left looking drained, so the walk pops it without visiting a
    /// single item.
    unsafe fn conv_real_list_after_start(
        &mut self,
        tv: *mut typval_T,
        frame: &mut ConvFrame,
    ) -> Flow {
        debug_assert!(!tv.is_null());
        // SAFETY: the walk's live typval.
        let mut val = unsafe { Tv::new(tv) };
        val.v_lock = VarLock::Unlocked;
        let list = val.list();
        // SAFETY: the typval's own list.
        let mut ls = unsafe { Ls::new(list) };
        if ls.lv_refcount.is_shared() {
            ls.lv_refcount.release();
            unsafe { (*tv).vval.v_list = ptr::null_mut() };
            // Always a `List`: the walk calls this straight after pushing
            // one for this very value.
            if let Frame::List { li, .. } = &mut frame.frame {
                *li = ptr::null_mut();
            }
            return Flow::Stop;
        }
        Flow::Go
    }

    unsafe fn conv_list_end(&mut self, tv: *mut typval_T) {
        if tv.is_null() {
            // A partial's argument list, which has no `typval_T` of its
            // own; `conv_func_end` releases the partial that owns it.
            return;
        }
        unsafe { tv_list_unref((*tv).vval.v_list) };
        unsafe { (*tv).vval.v_list = ptr::null_mut() };
    }

    /// The dictionary counterpart of [`Self::conv_real_list_after_start`].
    unsafe fn conv_real_dict_after_start(
        &mut self,
        tv: *mut typval_T,
        dictp: Option<*mut *mut dict_T>,
        frame: &mut ConvFrame,
    ) -> Flow {
        if !tv.is_null() {
            unsafe { (*tv).v_lock = VarLock::Unlocked };
        }
        if let Some(dictp) = dictp
            && unsafe { (**dictp).dv_refcount }.is_shared()
        {
            unsafe { (**dictp).dv_refcount.release() };
            unsafe { *dictp = ptr::null_mut() };
            if let Frame::Dict { todo, .. } = &mut frame.frame {
                *todo = 0;
            }
            return Flow::Stop;
        }
        Flow::Go
    }

    unsafe fn conv_dict_end(&mut self, _tv: *mut typval_T, dictp: Option<*mut *mut dict_T>) {
        if let Some(dictp) = dictp {
            unsafe { tv_dict_unref(*dictp) };
            unsafe { *dictp = ptr::null_mut() };
        }
    }

    /// Already been here, so this reference is one the container holds on
    /// itself.  Answering `Go` means "handled": the walk stops converting the
    /// value, and the reference count the enclosing container drops is what
    /// frees it.
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
pub(crate) unsafe fn encode_vim_to_nothing(tv: *mut typval_T, objname: *const c_char) -> bool {
    unsafe { encode_typval(&mut NothingSink, tv, objname) }
}
