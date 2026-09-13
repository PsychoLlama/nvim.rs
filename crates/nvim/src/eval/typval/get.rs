//! Coercions: reading a `TypVal` as a number, float, string or boolean.
//!
//! [`tv_get_number_chk`] is the arithmetic conversion, with the `_chk` half
//! reporting whether the value was convertible at all.
//! [`tv_get_string_buf_chk`] is the string one, which formats numbers into a
//! caller-supplied `NUMBUFLEN` buffer so the result never needs freeing.
//! [`tv2bool`] is the truthiness `if` and `while` ask for.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::charset::Str2NrBases;
use crate::os::cshim::gettext_ptr;
use crate::semsg;
use crate::types::NUL;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use ::core::ffi::CStr;

/// A coercion that failed with its error already on screen.
///
/// The `_chk` readings below report the message that names what went wrong
/// -- `E745: Using a List as a Number`, `E808: Number or Float required` --
/// so a caller has no error to render, only a branch to take.  It is the
/// out-parameter `bool` upstream set, moved into the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unconvertible;

/// `tv` as a number, raising an error and answering 0 for a value that has no
/// numeric form.
pub fn tv_get_number(tv: &TypVal) -> VarNumber {
    tv_get_number_chk(tv).unwrap_or(0)
}

/// `tv` as a number, or [`Unconvertible`] for a value that has no numeric
/// form -- with the message naming it already reported.
///
/// A String that is not a number is *not* a failure: it reads as 0, the way
/// Vimscript's coercion does. Only a container, a funcref, a Float and the
/// internal `VAR_UNKNOWN` have no numeric form at all.
pub fn tv_get_number_chk(tv: &TypVal) -> Result<VarNumber, Unconvertible> {
    let val = tv;
    match val.v_type() {
        VAR_NUMBER => return Ok(val.number_or_zero()),
        VAR_STRING => {
            let mut n = 0;
            if let Some(s) = val.as_string().filter(|s| !s.is_null()) {
                let (prep, len) = (::core::ptr::null_mut(), ::core::ptr::null_mut());
                let (unptr, overflow) = (::core::ptr::null_mut(), ::core::ptr::null_mut());
                let all = Str2NrBases::ALL;
                // SAFETY: the variant says the payload is a live
                // NUL-terminated string; every out-parameter but `n` is
                // declined, and `n` is this frame's own.
                #[rustfmt::skip]
                unsafe { vim_str2nr(s, prep, len, all, &raw mut n, unptr, 0, false, overflow) };
            }
            return Ok(n);
        }
        VAR_BOOL => return Ok(VarNumber::from(val.as_bool() == Some(kBoolVarTrue))),
        VAR_SPECIAL => return Ok(0),
        VAR_FUNC | VAR_PARTIAL | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_FLOAT => {
            // SAFETY: `num_errors` is the static table of messages indexed by
            // the kind, and the kind is this value's own.
            unsafe { emsg(gettext_ptr(num_errors[val.v_type() as usize])) };
        }
        VAR_UNKNOWN => {
            let arg0 = "tv_get_number(UNKNOWN)";
            semsg!("E685: Internal error: {arg0}");
        }
        _ => {}
    }

    Err(Unconvertible)
}

/// `tv` as a boolean number: -1 when it has no numeric form.
///
/// The tri-state upstream got by handing `tv_get_number_chk` a NULL flag.
pub fn tv_get_bool(tv: &TypVal) -> VarNumber {
    tv_get_number_chk(tv).unwrap_or(-1)
}

/// `tv` as a boolean number, or [`Unconvertible`] when it has none.
pub fn tv_get_bool_chk(tv: &TypVal) -> Result<VarNumber, Unconvertible> {
    tv_get_number_chk(tv)
}

/// `tv` as a line number, resolving a non-Number such as `"$"` or `"."`
/// through `var2fpos`.
pub fn tv_get_lnum(tv: &TypVal) -> LineNr {
    let did_emsg_before = did_emsg.get();
    let mut lnum = tv_get_bool(tv) as LineNr;
    if lnum <= 0 && did_emsg_before == did_emsg.get() && tv.v_type() != VAR_NUMBER {
        // No valid number, try using same function as line() does.
        let mut fnum = 0;
        // SAFETY: the value is the caller's and `fnum` this frame's own.
        let fp = unsafe { var2fpos(tv, true, &raw mut fnum, false, Win::current()) };
        if let Some(fp) = fp.as_ref() {
            lnum = fp.lnum;
        }
    }
    lnum
}

/// [`tv_get_lnum`] against a given buffer: `"$"` is that buffer's last line.
pub fn tv_get_lnum_buf(tv: &TypVal, buffer: Option<Buf>) -> LineNr {
    let val = tv;
    let s = val.string_or_null();
    // SAFETY: the variant says the payload is a live NUL-terminated string,
    // so the second byte is readable once the first is not the terminator.
    if let Some(buffer) = buffer
        && !s.is_null()
        && unsafe { *s } as ::core::ffi::c_int == '$' as ::core::ffi::c_int
        && unsafe { *s.add(1) } as ::core::ffi::c_int == NUL
    {
        return buffer.b_ml.ml_line_count;
    }
    tv_get_bool(tv) as LineNr
}

/// `tv` as a float, raising an error and answering 0.0 for a value that has no
/// float form.
pub fn tv_get_float(tv: &TypVal) -> Float {
    let val = tv;
    let message = match val.v_type() {
        VAR_NUMBER => return val.number_or_zero() as Float,
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

/// The scratch a caller lends for the string form of a Number.
///
/// It replaces the process-wide buffer the C's `tv_get_string`,
/// `tv_get_string_chk` and `tv_dict_get_string` answer from: a caller owns
/// its own, so two answers held at once no longer collide — with one shared
/// buffer the second silently overwrote the first.
///
/// The answer borrows either the value's own string or this buffer, so it
/// lives no longer than the shorter of the two. [`string`](Self::string) and
/// [`string_chk`](Self::string_chk) say that in the type; the `_ptr` pair
/// answers the same bytes as a raw pointer for the callers whose consumer is
/// still a `*const c_char`, and there the lifetime is the caller's to keep.
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

    /// `tv` as a string, or `None` with the error reported for a value that
    /// has no string form. The C's `tv_get_string_chk`.
    pub fn string_chk<'a>(&'a mut self, tv: &'a TypVal) -> Option<&'a CStr> {
        let text = self.string_ptr_chk(tv);
        // SAFETY: a non-null answer is a NUL-terminated string owned either
        // by `tv` or by this buffer, and the signature holds both for `'a`.
        (!text.is_null()).then(|| unsafe { CStr::from_ptr(text) })
    }

    /// `tv` as a string — the empty string, with the error reported, for a
    /// value that has none. The C's `tv_get_string`.
    pub fn string<'a>(&'a mut self, tv: &'a TypVal) -> &'a CStr {
        self.string_chk(tv).unwrap_or(c"")
    }

    /// [`string_chk`](Self::string_chk) as a borrowed pointer: NULL for a
    /// value with no string form, and otherwise bytes owned by `tv` or by
    /// this buffer.
    ///
    /// The answer is a *borrow* of both, and nothing in the type says so —
    /// it is here for the callers that hand the bytes straight to a
    /// `*const c_char` consumer, and it goes when they do.
    pub fn string_ptr_chk(&mut self, tv: &TypVal) -> *const ::core::ffi::c_char {
        let buf = self.0.as_mut_ptr();
        match tv.v_type() {
            VAR_NUMBER => {
                let n = tv.number_or_zero();
                let size = NUMBUFLEN as size_t;
                // SAFETY: `buf` is this buffer's own `NUMBUFLEN` bytes, and
                // the format string takes exactly the one argument given.
                unsafe { snprintf(buf, size, c"%ld".as_ptr(), n) };
                buf
            }
            VAR_FLOAT => {
                let f = tv.float_or_zero();
                // SAFETY: as above.
                unsafe { vim_snprintf(buf, NUMBUFLEN as size_t, c"%g".as_ptr(), f) };
                buf
            }
            VAR_STRING => {
                let s = tv.string_or_null();
                if s.is_null() { c"".as_ptr() } else { s }
            }
            VAR_BOOL => {
                let names = (&raw const encode_bool_var_names).cast::<*const ::core::ffi::c_char>();
                let which = tv.as_bool().unwrap_or(crate::types::kBoolVarFalse);
                // SAFETY: the table has one name per `BoolVarValue`, and
                // `which` is one; the names are shorter than `NUMBUFLEN`.
                unsafe {
                    let name = *names.offset(which as isize);
                    strcpy(buf, name);
                }
                buf
            }
            VAR_SPECIAL => {
                let names =
                    (&raw const encode_special_var_names).cast::<*const ::core::ffi::c_char>();
                let which = tv.as_special().unwrap_or(kSpecialVarNull);
                // SAFETY: as `VAR_BOOL`, for the special-value table.
                unsafe {
                    let name = *names.offset(which as isize);
                    strcpy(buf, name);
                }
                buf
            }
            VAR_PARTIAL | VAR_FUNC | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_UNKNOWN => {
                // SAFETY: `str_errors` is the static table of messages
                // indexed by the kind, and the kind is this value's own.
                unsafe { emsg(gettext_ptr(str_errors[tv.v_type() as usize])) };
                ::core::ptr::null()
            }
            // SAFETY: the eight arms above are every kind there is.
            _ => unsafe { abort() },
        }
    }

    /// [`string`](Self::string) as a borrowed pointer: the empty string
    /// rather than NULL for a value with no string form.
    ///
    /// See [`string_ptr_chk`](Self::string_ptr_chk) for why the pointer form
    /// exists.
    pub fn string_ptr(&mut self, tv: &TypVal) -> *const ::core::ffi::c_char {
        let text = self.string_ptr_chk(tv);
        if text.is_null() { c"".as_ptr() } else { text }
    }

    /// The raw buffer, for the `*buffer` entry points that take one.
    pub fn as_mut_ptr(&mut self) -> *mut ::core::ffi::c_char {
        self.0.as_mut_ptr()
    }
}

/// Truthiness of `tv`, as `if` and `while` ask for it.
pub fn tv2bool(tv: &TypVal) -> bool {
    match tv.v_type() {
        VAR_NUMBER => tv.number_or_zero() != 0,
        VAR_FLOAT => tv.float_or_zero() != 0.0,
        VAR_PARTIAL => !tv.partial_or_null().is_null(),
        VAR_FUNC | VAR_STRING => {
            let s = tv.string_or_func_name();
            !s.is_null() && unsafe { *s } as ::core::ffi::c_int != NUL
        }
        VAR_LIST => {
            let l = tv.list_or_null();
            !l.is_null() && unsafe { tv_list_len(l) } > 0
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
