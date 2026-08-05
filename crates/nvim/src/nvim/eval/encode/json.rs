//! `json_encode()`: a typval as JSON text.
//!
//! One [`TypvalSink`] over the output [`Gap`], replacing the
//! `TYPVAL_ENCODE_NAME json` instantiation of `typval_encode.c.h`.  Its
//! punctuation is upstream's `string()` sink's — JSON and Vimscript spell
//! lists and dictionaries alike — and what it overrides is everything JSON
//! *cannot* say: NaN and infinity, `ext` values, function references and
//! non-string keys are all refusals, and a string has to come out as escaped
//! UTF-8 rather than as bytes.
//!
//! Self-reference is the odd one out.  It is not an error here: the value is
//! reported once through `E724` and then *omitted*, so the output is not
//! valid JSON either.  That is upstream's behaviour and evalsweep pins it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::src::nvim::eval::encode::{
    OK, conv_error, convert_to_json_string, did_echo_string_emsg, encode_check_json_key,
};
use crate::src::nvim::eval::typval::tv_blob_get;
use crate::src::nvim::eval::typval_encode::{ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::src::nvim::garray::Gap;
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::strings::vim_snprintf_safelen;
use crate::src::nvim::types::{blob_T, dict_T, float_T, garray_T, int64_t, size_t, typval_T};

/// `NUMBUFLEN`: the scratch buffer every `printf`-formatted number goes
/// through.
const NUMBUFLEN: usize = 65;

const E474_FUNCREF: &CStr = c"E474: Error while dumping %s, %s: attempt to dump function reference";
const E474_NAN: &CStr = c"E474: Unable to represent NaN value in JSON";
const E474_INFINITY: &CStr = c"E474: Unable to represent infinity in JSON";
const E474_EXT: &CStr = c"E474: Unable to convert EXT string to JSON";
const E474_INVALID_KEY: &CStr = c"E474: Invalid key in special dictionary";
const E724_SELF_REFERENCE: &CStr =
    c"E724: unable to correctly dump variable with self-referencing container";

struct JsonSink<'a> {
    gap: Gap<'a>,
}

/// Raise `msg`, which carries no arguments.
fn err(msg: &CStr) {
    unsafe { emsg(gettext(msg.as_ptr())) };
}

impl JsonSink<'_> {
    /// Append one number, formatted the way C's `printf` would.
    ///
    /// The three spellings the sink needs are `%ld`, `%lu` and `%g`, and only
    /// `%g` has no Rust equivalent that is guaranteed to agree byte for byte —
    /// so all three go through `vim_snprintf` and stay consistent.
    fn concat_num<T>(&mut self, fmt: &CStr, num: T) {
        let mut numbuf = [0 as c_char; NUMBUFLEN];
        let formatted = unsafe {
            let len = vim_snprintf_safelen(numbuf.as_mut_ptr(), NUMBUFLEN, fmt.as_ptr(), num);
            ::core::slice::from_raw_parts(numbuf.as_ptr() as *const u8, len)
        };
        self.gap.concat(formatted);
    }
}

impl TypvalSink for JsonSink<'_> {
    const ALLOW_SPECIALS: bool = true;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_json_convert_one_value()";

    unsafe fn conv_nil(&mut self, _tv: *mut typval_T) {
        self.gap.concat(b"null");
    }

    unsafe fn conv_bool(&mut self, _tv: *mut typval_T, num: bool) {
        self.gap.concat(if num {
            b"true".as_slice()
        } else {
            b"false".as_slice()
        });
    }

    unsafe fn conv_number(&mut self, _tv: *mut typval_T, num: int64_t) {
        self.concat_num(c"%ld", num);
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: *mut typval_T, num: u64) {
        self.concat_num(c"%lu", num);
    }

    unsafe fn conv_float(&mut self, _tv: *mut typval_T, flt: float_T) -> Flow {
        match flt.classify() {
            ::core::num::FpCategory::Nan => {
                err(E474_NAN);
                Flow::Fail
            }
            ::core::num::FpCategory::Infinite => {
                err(E474_INFINITY);
                Flow::Fail
            }
            _ => {
                self.concat_num(c"%g", flt);
                Flow::Go
            }
        }
    }

    /// Escaped, quoted UTF-8.  A string that is not valid UTF-8 is a failure,
    /// which is what makes this the hook JSON most often refuses on.
    unsafe fn conv_string(&mut self, _tv: *mut typval_T, buf: *mut c_char, len: size_t) -> Flow {
        let gap = self.gap.as_ptr();
        if unsafe { convert_to_json_string(gap, buf, len) } == OK {
            Flow::Go
        } else {
            Flow::Fail
        }
    }

    unsafe fn conv_ext_string(
        &mut self,
        _tv: *mut typval_T,
        buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        // Bails, so the walk's own free never runs and this one owns the
        // buffer.
        unsafe { xfree(buf as *mut c_void) };
        err(E474_EXT);
        Flow::Fail
    }

    /// A blob becomes an array of byte values — JSON has nothing shorter.
    unsafe fn conv_blob(&mut self, _tv: *mut typval_T, blob: *const blob_T, len: c_int) {
        if len == 0 {
            self.gap.concat(b"[]");
            return;
        }
        self.gap.append(b'[');
        for i in 0..len {
            if i > 0 {
                self.gap.concat(b", ");
            }
            self.concat_num(c"%d", unsafe { tv_blob_get(blob, i) } as c_int);
        }
        self.gap.append(b']');
    }

    unsafe fn conv_func_start(
        &mut self,
        _tv: *mut typval_T,
        _fun: *mut c_char,
        _prefix: &'static CStr,
        path: &ConvPath,
    ) -> Flow {
        unsafe { conv_error(gettext(E474_FUNCREF.as_ptr()), path) }
    }

    unsafe fn conv_empty_list(&mut self, _tv: *mut typval_T) {
        self.gap.concat(b"[]");
    }

    unsafe fn conv_empty_dict(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        self.gap.concat(b"{}");
    }

    unsafe fn conv_list_start(&mut self, _tv: *mut typval_T, _len: c_int) -> Flow {
        self.gap.append(b'[');
        Flow::Go
    }

    unsafe fn conv_list_between_items(&mut self, _tv: *mut typval_T) {
        self.gap.concat(b", ");
    }

    unsafe fn conv_list_end(&mut self, _tv: *mut typval_T) {
        self.gap.append(b']');
    }

    unsafe fn conv_dict_start(&mut self, _tv: *mut typval_T, _len: size_t) -> Flow {
        self.gap.append(b'{');
        Flow::Go
    }

    /// A special map may carry any typval as a key; JSON may not.
    unsafe fn special_dict_key_check(&mut self, key: *const typval_T) -> Flow {
        if unsafe { encode_check_json_key(key) } {
            Flow::Go
        } else {
            err(E474_INVALID_KEY);
            Flow::Fail
        }
    }

    unsafe fn conv_dict_after_key(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        self.gap.concat(b": ");
    }

    unsafe fn conv_dict_between_items(
        &mut self,
        _tv: *mut typval_T,
        _dictp: Option<*mut *mut dict_T>,
    ) {
        self.gap.concat(b", ");
    }

    unsafe fn conv_dict_end(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        self.gap.append(b'}');
    }

    /// Say so once per encode, then leave the value out entirely.
    unsafe fn conv_recurse(
        &mut self,
        _val: *mut c_void,
        _conv_type: ConvType,
        _path: &ConvPath,
    ) -> Flow {
        if !did_echo_string_emsg.get() {
            // Only once per dump: a cycle usually shows up many times over.
            did_echo_string_emsg.set(true);
            err(E724_SELF_REFERENCE);
        }
        Flow::Go
    }
}

/// Append `tv` to `gap` as JSON.
///
/// # Safety
/// `gap` must be a live byte-item garray, `tv` a live typval and `objname`
/// NUL-terminated.
pub(crate) unsafe fn encode_vim_to_json(
    gap: *mut garray_T,
    tv: *mut typval_T,
    objname: *const c_char,
) -> bool {
    let mut sink = JsonSink {
        gap: Gap(unsafe { &mut *gap }),
    };
    unsafe { encode_typval(&mut sink, tv, objname) }
}
