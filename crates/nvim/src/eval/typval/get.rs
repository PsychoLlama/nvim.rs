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
use crate::charset::{Str2NrBases, str2nr_in};
use crate::semsg;
use crate::strings::format_float_g;
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
            let text = val.string_bytes();
            return Ok(str2nr_in(text, Str2NrBases::ALL, false).value);
        }
        VAR_BOOL => return Ok(VarNumber::from(val.as_bool() == Some(kBoolVarTrue))),
        VAR_SPECIAL => return Ok(0),
        VAR_FUNC | VAR_PARTIAL | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_FLOAT => {
            emsg(gettext(num_errors[val.v_type() as usize]));
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
    if let Some(buffer) = buffer
        && tv.string_bytes() == b"$"
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
/// The answer borrows either the value's own string, a static name, or
/// this buffer, so it lives no longer than the shorter of the two borrows.
pub struct NumBuf([u8; NUMBUFLEN as usize]);

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
        match tv.v_type() {
            VAR_NUMBER => Some(self.formatted(format_args!("{}", tv.number_or_zero()))),
            VAR_FLOAT => {
                let len = format_float_g(tv.float_or_zero(), &mut self.0);
                Some(self.terminated(len))
            }
            VAR_STRING => Some(tv.string_cstr().unwrap_or(c"")),
            VAR_BOOL => {
                let which = tv.as_bool().unwrap_or(crate::types::kBoolVarFalse);
                Some(self.copied(BOOL_VAR_NAMES[which as usize]))
            }
            VAR_SPECIAL => {
                let which = tv.as_special().unwrap_or(kSpecialVarNull);
                Some(self.copied(SPECIAL_VAR_NAMES[which as usize]))
            }
            VAR_PARTIAL | VAR_FUNC | VAR_LIST | VAR_DICT | VAR_BLOB | VAR_UNKNOWN => {
                emsg(gettext(str_errors[tv.v_type() as usize]));
                None
            }
            _ => unreachable!("the eight arms above are every kind there is"),
        }
    }

    /// `tv` as a string — the empty string, with the error reported, for a
    /// value that has none. The C's `tv_get_string`.
    pub fn string<'a>(&'a mut self, tv: &'a TypVal) -> &'a CStr {
        self.string_chk(tv).unwrap_or(c"")
    }

    /// [`string_chk`](Self::string_chk) as bytes.
    pub fn bytes_chk<'a>(&'a mut self, tv: &'a TypVal) -> Option<&'a [u8]> {
        self.string_chk(tv).map(CStr::to_bytes)
    }

    /// [`string`](Self::string) as bytes.
    pub fn bytes<'a>(&'a mut self, tv: &'a TypVal) -> &'a [u8] {
        self.string(tv).to_bytes()
    }

    /// `args` written into the buffer, terminated -- cut short to fit, as
    /// `snprintf` would.
    fn formatted(&mut self, args: ::core::fmt::Arguments<'_>) -> &CStr {
        struct Cursor<'b>(&'b mut [u8], usize);
        impl ::core::fmt::Write for Cursor<'_> {
            fn write_str(&mut self, text: &str) -> ::core::fmt::Result {
                // Room for the terminator is kept back.
                let room = self.0.len() - 1 - self.1;
                let take = text.len().min(room);
                self.0[self.1..self.1 + take].copy_from_slice(&text.as_bytes()[..take]);
                self.1 += take;
                Ok(())
            }
        }
        let mut cursor = Cursor(&mut self.0, 0);
        // `write_str` never fails, so neither does the whole write.
        let _ = ::core::fmt::write(&mut cursor, args);
        let len = cursor.1;
        self.terminated(len)
    }

    /// TRANSIENT.
    pub fn string_ptr_chk(&mut self, tv: &TypVal) -> *const ::core::ffi::c_char {
        self.string_chk(tv)
            .map_or(::core::ptr::null(), CStr::as_ptr)
    }

    /// TRANSIENT.
    pub fn string_ptr(&mut self, tv: &TypVal) -> *const ::core::ffi::c_char {
        self.string(tv).as_ptr()
    }

    /// TRANSIENT.
    pub fn as_mut_ptr(&mut self) -> *mut ::core::ffi::c_char {
        self.0.as_mut_ptr().cast()
    }

    /// `name` copied into the buffer. A static name would do for reading,
    /// but some callers still hand the answer to a C callee that writes into
    /// it (`findfile()`'s path walk cuts it at each comma), as upstream's
    /// `strcpy` into this buffer allowed.
    fn copied(&mut self, name: &CStr) -> &CStr {
        let bytes = name.to_bytes();
        let len = bytes.len().min(self.0.len() - 1);
        self.0[..len].copy_from_slice(&bytes[..len]);
        self.terminated(len)
    }

    /// The first `len` bytes of the buffer, terminated.
    fn terminated(&mut self, len: usize) -> &CStr {
        let len = len.min(self.0.len() - 1);
        self.0[len] = 0;
        CStr::from_bytes_until_nul(&self.0).expect("terminated just above")
    }
}

/// Truthiness of `tv`, as `if` and `while` ask for it.
pub fn tv2bool(tv: &TypVal) -> bool {
    match tv.v_type() {
        VAR_NUMBER => tv.number_or_zero() != 0,
        VAR_FLOAT => tv.float_or_zero() != 0.0,
        VAR_PARTIAL => !tv.partial_or_null().is_null(),
        VAR_FUNC | VAR_STRING => tv.text_or_name().is_some_and(|text| !text.is_empty()),
        VAR_LIST => list_len(tv.list_ref()) > 0,
        VAR_DICT => tv.dict_ref().is_some_and(|d| d.dv_hashtab.ht_used > 0),
        VAR_BOOL => tv.as_bool() == Some(kBoolVarTrue),
        VAR_SPECIAL => tv.as_special().is_some_and(|s| s != kSpecialVarNull),
        VAR_BLOB => tv.blob_ref().is_some_and(|b| !b.is_empty()),
        _ => false,
    }
}
