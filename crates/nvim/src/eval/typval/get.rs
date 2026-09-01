//! Coercions: reading a `typval_T` as a number, float, string or boolean.
//!
//! [`tv_get_number_chk`] is the arithmetic conversion, with the `_chk` half
//! reporting whether the value was convertible at all.
//! [`tv_get_string_buf_chk`] is the string one, which formats numbers into a
//! caller-supplied `NUMBUFLEN` buffer so the result never needs freeing.
//! [`tv2bool`] is the truthiness `if` and `while` ask for.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::os::cshim::gettext_ptr;
use crate::semsg;
use crate::types::NUL;

/// `tv` as a number, raising an error and answering 0 for a value that has no
/// numeric form.
pub unsafe fn tv_get_number(tv: *const typval_T) -> varnumber_T {
    let mut error = false;
    unsafe { tv_get_number_chk(tv, &raw mut error) }
}

/// `tv` as a number, setting `*ret_error` for a value that has no numeric
/// form.
///
/// With a NULL `ret_error` the failure answer is -1 rather than 0, which is
/// what makes `tv_get_bool` usable as a tri-state.
pub unsafe fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T {
    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv.cast_mut()) };
    match val.v_type {
        VAR_NUMBER => return val.number_or_zero(),
        VAR_STRING => {
            let mut n = 0;
            if let Some(s) = val.as_string().filter(|s| !s.is_null()) {
                let (prep, len) = (::core::ptr::null_mut(), ::core::ptr::null_mut());
                let (unptr, overflow) = (::core::ptr::null_mut(), ::core::ptr::null_mut());
                let all = STR2NR_ALL as ::core::ffi::c_int;
                #[rustfmt::skip]
                unsafe { vim_str2nr(s, prep, len, all, &raw mut n, unptr, 0, false, overflow) };
            }
            return n;
        }
        VAR_BOOL => return varnumber_T::from(val.as_bool() == Some(kBoolVarTrue)),
        VAR_SPECIAL => return 0,
        VAR_FUNC | VAR_PARTIAL | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_FLOAT => {
            unsafe { emsg(gettext_ptr(num_errors[(*tv).v_type as usize])) };
        }
        VAR_UNKNOWN => {
            let arg0 = "tv_get_number(UNKNOWN)";
            semsg!("E685: Internal error: {arg0}");
        }
        _ => {}
    }

    if let Some(ret_error) = unsafe { ret_error.as_mut() } {
        *ret_error = true;
        0
    } else {
        -1
    }
}

/// `tv` as a boolean number: -1 when it has no numeric form.
pub unsafe fn tv_get_bool(tv: *const typval_T) -> varnumber_T {
    unsafe { tv_get_number_chk(tv, ::core::ptr::null_mut()) }
}

/// `tv` as a boolean number, setting `*ret_error` when it has no numeric form.
pub unsafe fn tv_get_bool_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T {
    unsafe { tv_get_number_chk(tv, ret_error) }
}

/// `tv` as a line number, resolving a non-Number such as `"$"` or `"."`
/// through `var2fpos`.
pub unsafe fn tv_get_lnum(tv: *const typval_T) -> linenr_T {
    let did_emsg_before = did_emsg.get();
    let mut lnum = unsafe { tv_get_number_chk(tv, ::core::ptr::null_mut()) } as linenr_T;
    if lnum <= 0 && did_emsg_before == did_emsg.get() && unsafe { (*tv).v_type } != VAR_NUMBER {
        // No valid number, try using same function as line() does.
        let mut fnum = 0;
        let fp = unsafe { var2fpos(tv, true, &raw mut fnum, false, curwin.get()) };
        if let Some(fp) = fp.as_ref() {
            lnum = fp.lnum;
        }
    }
    lnum
}

/// [`tv_get_lnum`] against a given buffer: `"$"` is that buffer's last line.
pub unsafe fn tv_get_lnum_buf(tv: *const typval_T, buf: *const buf_T) -> linenr_T {
    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv.cast_mut()) };
    let s = val.string_or_null();
    if !s.is_null()
        && unsafe { *s } as ::core::ffi::c_int == '$' as ::core::ffi::c_int
        && unsafe { *s.add(1) } as ::core::ffi::c_int == NUL
        && !buf.is_null()
    {
        return unsafe { (*buf).b_ml.ml_line_count };
    }
    unsafe { tv_get_number_chk(tv, ::core::ptr::null_mut()) as linenr_T }
}

/// `tv` as a float, raising an error and answering 0.0 for a value that has no
/// float form.
pub unsafe fn tv_get_float(tv: *const typval_T) -> float_T {
    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv.cast_mut()) };
    let message = match val.v_type {
        VAR_NUMBER => return val.number_or_zero() as float_T,
        VAR_FLOAT => return val.float_or_zero(),
        VAR_PARTIAL | VAR_FUNC => c"E891: Using a Funcref as a Float",
        VAR_STRING => c"E892: Using a String as a Float",
        VAR_LIST => c"E893: Using a List as a Float",
        VAR_DICT => c"E894: Using a Dictionary as a Float",
        VAR_BOOL => c"E362: Using a boolean value as a Float",
        VAR_SPECIAL => c"E907: Using a special value as a Float",
        VAR_BLOB => c"E975: Using a Blob as a Float",
        VAR_UNKNOWN => {
            let arg0 = "tv_get_float(UNKNOWN)";
            semsg!("E685: Internal error: {arg0}");
            return 0.0;
        }
        _ => return 0.0,
    };
    emsg(gettext(message));
    0.0
}

/// `tv` as a string, formatting a number into `buf` (`NUMBUFLEN` bytes).
///
/// Answers NULL with an error raised for a value that has no string form.
pub unsafe fn tv_get_string_buf_chk(
    tv: *const typval_T,
    buf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv.cast_mut()) };
    match val.v_type {
        VAR_NUMBER => {
            let n = val.number_or_zero();
            let size = NUMBUFLEN as size_t;
            unsafe { snprintf(buf, size, c"%ld".as_ptr(), n) };
            buf
        }
        VAR_FLOAT => {
            let f = val.float_or_zero();
            unsafe { vim_snprintf(buf, NUMBUFLEN as size_t, c"%g".as_ptr(), f) };
            buf
        }
        VAR_STRING => {
            let s = val.string_or_null();
            if s.is_null() { c"".as_ptr() } else { s }
        }
        VAR_BOOL => {
            let names = (&raw const encode_bool_var_names).cast::<*const ::core::ffi::c_char>();
            let which = val.as_bool().unwrap_or(crate::types::kBoolVarFalse);
            let name = unsafe { *names.offset(which as isize) };
            unsafe { strcpy(buf, name) };
            buf
        }
        VAR_SPECIAL => {
            let names = (&raw const encode_special_var_names).cast::<*const ::core::ffi::c_char>();
            let which = val.as_special().unwrap_or(kSpecialVarNull);
            let name = unsafe { *names.offset(which as isize) };
            unsafe { strcpy(buf, name) };
            buf
        }
        VAR_PARTIAL | VAR_FUNC | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_UNKNOWN => {
            unsafe { emsg(gettext_ptr(str_errors[(*tv).v_type as usize])) };
            ::core::ptr::null()
        }
        _ => unsafe { abort() },
    }
}

/// The scratch a caller lends for the string form of a Number.
///
/// It replaces the process-wide buffer the C's `tv_get_string`,
/// `tv_get_string_chk` and `tv_dict_get_string` answer from: a caller owns
/// its own, so two answers held at once no longer collide — with one shared
/// buffer the second silently overwrote the first.
///
/// The answer borrows either the value's own string or this buffer, so it
/// lives no longer than the shorter of the two: a caller whose answer
/// outlives the frame must be lent a buffer that does too.
pub struct NumBuf([::core::ffi::c_char; NUMBUFLEN as usize]);

impl Default for NumBuf {
    fn default() -> Self {
        NumBuf::new()
    }
}

impl NumBuf {
    /// A fresh, zeroed scratch.
    pub const fn new() -> Self {
        NumBuf([0; NUMBUFLEN as usize])
    }

    /// `tv` as a string — the empty string, with the error reported, for a
    /// value that has none. The C's `tv_get_string`.
    ///
    /// # Safety
    /// `tv` points at a live, initialised value.
    pub unsafe fn string(&mut self, tv: *const typval_T) -> *const ::core::ffi::c_char {
        // SAFETY: the caller's value; the scratch is `NUMBUFLEN` bytes.
        unsafe { tv_get_string_buf(tv, self.as_mut_ptr()) }
    }

    /// As [`string`](Self::string), but NULL rather than the empty string for
    /// a value that has none. The C's `tv_get_string_chk`.
    ///
    /// # Safety
    /// `tv` points at a live, initialised value.
    pub unsafe fn string_chk(&mut self, tv: *const typval_T) -> *const ::core::ffi::c_char {
        // SAFETY: as `string`.
        unsafe { tv_get_string_buf_chk(tv, self.as_mut_ptr()) }
    }

    /// The raw buffer, for the `*_buf` entry points that take one.
    pub fn as_mut_ptr(&mut self) -> *mut ::core::ffi::c_char {
        self.0.as_mut_ptr()
    }
}

/// [`tv_get_string_buf_chk`] answering the empty string rather than NULL.
pub unsafe fn tv_get_string_buf(
    tv: *const typval_T,
    buf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let res = unsafe { tv_get_string_buf_chk(tv, buf) };
    if res.is_null() { c"".as_ptr() } else { res }
}

/// Truthiness of `tv`, as `if` and `while` ask for it.
pub unsafe fn tv2bool(tv: *const typval_T) -> bool {
    // SAFETY: the caller's promise: a live typval.
    let tv = unsafe { Tv::new(tv.cast_mut()) };
    match tv.v_type {
        VAR_NUMBER => tv.number_or_zero() != 0,
        VAR_FLOAT => tv.float_or_zero() != 0.0,
        VAR_PARTIAL => !tv.partial_or_null().is_null(),
        VAR_FUNC | VAR_STRING => {
            let s = tv.string_or_func_name();
            !s.is_null() && unsafe { *s } as ::core::ffi::c_int != NUL
        }
        VAR_LIST => {
            let l = tv.list_or_null();
            !l.is_null() && unsafe { (*l).lv_len } > 0
        }
        VAR_DICT => {
            let d = tv.dict_or_null();
            !d.is_null() && unsafe { (*d).dv_hashtab.ht_used } > 0
        }
        VAR_BOOL => tv.as_bool() == Some(kBoolVarTrue),
        VAR_SPECIAL => tv.as_special().is_some_and(|s| s != kSpecialVarNull),
        VAR_BLOB => {
            let b = tv.blob_or_null();
            !b.is_null() && unsafe { (*b).bv_ga.ga_len } > 0
        }
        _ => false,
    }
}
