//! Coercions: reading a `typval_T` as a number, float, string or boolean.
//!
//! [`tv_get_number_chk`] is the arithmetic conversion, with the `_chk` half
//! reporting whether the value was convertible at all.
//! [`tv_get_string_buf_chk`] is the string one, which formats numbers into a
//! caller-supplied `NUMBUFLEN` buffer so the result never needs freeing.
//! [`tv2bool`] is the truthiness `if` and `while` ask for.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::NUL;

/// `tv` as a number, raising an error and answering 0 for a value that has no
/// numeric form.
pub unsafe fn tv_get_number(tv: *const typval_T) -> varnumber_T {
    unsafe {
        let mut error = false;
        tv_get_number_chk(tv, &raw mut error)
    }
}

/// `tv` as a number, setting `*ret_error` for a value that has no numeric
/// form.
///
/// With a NULL `ret_error` the failure answer is -1 rather than 0, which is
/// what makes `tv_get_bool` usable as a tri-state.
pub unsafe fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T {
    unsafe {
        match (*tv).v_type {
            VAR_NUMBER => return (*tv).vval.v_number,
            VAR_STRING => {
                let mut n = 0;
                if !(*tv).vval.v_string.is_null() {
                    vim_str2nr(
                        (*tv).vval.v_string,
                        ::core::ptr::null_mut(),
                        ::core::ptr::null_mut(),
                        STR2NR_ALL as ::core::ffi::c_int,
                        &raw mut n,
                        ::core::ptr::null_mut(),
                        0,
                        false,
                        ::core::ptr::null_mut(),
                    );
                }
                return n;
            }
            VAR_BOOL => return varnumber_T::from((*tv).vval.v_bool == kBoolVarTrue),
            VAR_SPECIAL => return 0,
            VAR_FUNC | VAR_PARTIAL | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_FLOAT => {
                emsg(gettext(num_errors[(*tv).v_type as usize]));
            }
            VAR_UNKNOWN => {
                semsg_c!(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    c"tv_get_number(UNKNOWN)".as_ptr(),
                );
            }
            _ => {}
        }

        if let Some(ret_error) = ret_error.as_mut() {
            *ret_error = true;
            0
        } else {
            -1
        }
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
    unsafe {
        let did_emsg_before = did_emsg.get();
        let mut lnum = tv_get_number_chk(tv, ::core::ptr::null_mut()) as linenr_T;
        if lnum <= 0 && did_emsg_before == did_emsg.get() && (*tv).v_type != VAR_NUMBER {
            // No valid number, try using same function as line() does.
            let mut fnum = 0;
            let fp = var2fpos(tv, true, &raw mut fnum, false, curwin.get());
            if let Some(fp) = fp.as_ref() {
                lnum = fp.lnum;
            }
        }
        lnum
    }
}

/// [`tv_get_lnum`] against a given buffer: `"$"` is that buffer's last line.
pub unsafe fn tv_get_lnum_buf(tv: *const typval_T, buf: *const buf_T) -> linenr_T {
    unsafe {
        if (*tv).v_type == VAR_STRING
            && !(*tv).vval.v_string.is_null()
            && *(*tv).vval.v_string as ::core::ffi::c_int == '$' as ::core::ffi::c_int
            && *(*tv).vval.v_string.add(1) as ::core::ffi::c_int == NUL
            && !buf.is_null()
        {
            return (*buf).b_ml.ml_line_count;
        }
        tv_get_number_chk(tv, ::core::ptr::null_mut()) as linenr_T
    }
}

/// `tv` as a float, raising an error and answering 0.0 for a value that has no
/// float form.
pub unsafe fn tv_get_float(tv: *const typval_T) -> float_T {
    unsafe {
        let message = match (*tv).v_type {
            VAR_NUMBER => return (*tv).vval.v_number as float_T,
            VAR_FLOAT => return (*tv).vval.v_float,
            VAR_PARTIAL | VAR_FUNC => c"E891: Using a Funcref as a Float",
            VAR_STRING => c"E892: Using a String as a Float",
            VAR_LIST => c"E893: Using a List as a Float",
            VAR_DICT => c"E894: Using a Dictionary as a Float",
            VAR_BOOL => c"E362: Using a boolean value as a Float",
            VAR_SPECIAL => c"E907: Using a special value as a Float",
            VAR_BLOB => c"E975: Using a Blob as a Float",
            VAR_UNKNOWN => {
                semsg_c!(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    c"tv_get_float(UNKNOWN)".as_ptr(),
                );
                return 0.0;
            }
            _ => return 0.0,
        };
        emsg(gettext(message.as_ptr()));
        0.0
    }
}

/// `tv` as a string, formatting a number into `buf` (`NUMBUFLEN` bytes).
///
/// Answers NULL with an error raised for a value that has no string form.
pub unsafe fn tv_get_string_buf_chk(
    tv: *const typval_T,
    buf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        match (*tv).v_type {
            VAR_NUMBER => {
                snprintf(
                    buf,
                    NUMBUFLEN as size_t,
                    c"%ld".as_ptr(),
                    (*tv).vval.v_number,
                );
                buf
            }
            VAR_FLOAT => {
                vim_snprintf(buf, NUMBUFLEN as size_t, c"%g".as_ptr(), (*tv).vval.v_float);
                buf
            }
            VAR_STRING => {
                if (*tv).vval.v_string.is_null() {
                    c"".as_ptr()
                } else {
                    (*tv).vval.v_string
                }
            }
            VAR_BOOL => {
                strcpy(
                    buf,
                    *(&raw const encode_bool_var_names as *const *const ::core::ffi::c_char)
                        .offset((*tv).vval.v_bool as isize),
                );
                buf
            }
            VAR_SPECIAL => {
                strcpy(
                    buf,
                    *(&raw const encode_special_var_names as *const *const ::core::ffi::c_char)
                        .offset((*tv).vval.v_special as isize),
                );
                buf
            }
            VAR_PARTIAL | VAR_FUNC | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_UNKNOWN => {
                emsg(gettext(str_errors[(*tv).v_type as usize]));
                ::core::ptr::null()
            }
            _ => abort(),
        }
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
    unsafe {
        let res = tv_get_string_buf_chk(tv, buf);
        if res.is_null() { c"".as_ptr() } else { res }
    }
}

/// Truthiness of `tv`, as `if` and `while` ask for it.
pub unsafe fn tv2bool(tv: *const typval_T) -> bool {
    unsafe {
        match (*tv).v_type {
            VAR_NUMBER => (*tv).vval.v_number != 0,
            VAR_FLOAT => (*tv).vval.v_float != 0.0,
            VAR_PARTIAL => !(*tv).vval.v_partial.is_null(),
            VAR_FUNC | VAR_STRING => {
                !(*tv).vval.v_string.is_null() && *(*tv).vval.v_string as ::core::ffi::c_int != NUL
            }
            VAR_LIST => !(*tv).vval.v_list.is_null() && (*(*tv).vval.v_list).lv_len > 0,
            VAR_DICT => !(*tv).vval.v_dict.is_null() && (*(*tv).vval.v_dict).dv_hashtab.ht_used > 0,
            VAR_BOOL => (*tv).vval.v_bool == kBoolVarTrue,
            VAR_SPECIAL => (*tv).vval.v_special != kSpecialVarNull,
            VAR_BLOB => !(*tv).vval.v_blob.is_null() && (*(*tv).vval.v_blob).bv_ga.ga_len > 0,
            _ => false,
        }
    }
}
