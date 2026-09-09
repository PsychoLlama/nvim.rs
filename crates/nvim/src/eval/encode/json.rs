//! `json_encode()`: a typval as JSON text.
//!
//! One [`TypvalSink`] over an output `Vec<u8>`, replacing the
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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::eval::encode::{
    conv_error, convert_to_json_string, did_echo_string_emsg, encode_check_json_key,
};
use crate::eval::typval::{DictSlot, tv_blob_get};
use crate::eval::typval_encode::{ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::memory::xfree;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::strings::vim_snprintf_safelen;
use crate::types::{Blob, Float, TypVal, int64_t, size_t};

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
    gap: &'a mut Vec<u8>,
}

/// Raise `msg`, which carries no arguments.
fn err(msg: &'static CStr) {
    emsg(gettext(msg));
}

impl JsonSink<'_> {
    /// Append one number, formatted the way C's `printf` would.
    ///
    /// The three spellings the sink needs are `%ld`, `%lu` and `%g`, and only
    /// `%g` has no Rust equivalent that is guaranteed to agree byte for byte —
    /// so all three go through `vim_snprintf` and stay consistent.
    fn concat_num<T: crate::message_fmt::CArg>(&mut self, fmt: &CStr, num: T) {
        let mut numbuf = [0 as c_char; NUMBUFLEN];
        let formatted = unsafe {
            let len = vim_snprintf_safelen(numbuf.as_mut_ptr(), NUMBUFLEN, fmt.as_ptr(), num);
            ::core::slice::from_raw_parts(numbuf.as_ptr().cast::<u8>(), len)
        };
        self.gap.extend_from_slice(formatted);
    }
}

impl TypvalSink for JsonSink<'_> {
    const ALLOW_SPECIALS: bool = true;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_json_convert_one_value()";

    /// # Safety
    ///
    /// As [`TypvalSink::conv_nil`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_nil(&mut self, _tv: *mut TypVal) {
        self.gap.extend_from_slice(b"null");
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_bool`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_bool(&mut self, _tv: *mut TypVal, num: bool) {
        self.gap.extend_from_slice(if num {
            b"true".as_slice()
        } else {
            b"false".as_slice()
        });
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_number`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_number(&mut self, _tv: *mut TypVal, num: int64_t) {
        self.concat_num(c"%ld", num);
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_unsigned_number`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_unsigned_number(&mut self, _tv: *mut TypVal, num: u64) {
        self.concat_num(c"%lu", num);
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_float`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_float(&mut self, _tv: *mut TypVal, flt: Float) -> Flow {
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
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_string(&mut self, _tv: *mut TypVal, buf: *mut c_char, len: size_t) -> Flow {
        if unsafe { convert_to_json_string(self.gap, buf, len) }.is_ok() {
            Flow::Go
        } else {
            Flow::Fail
        }
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_ext_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: *mut TypVal,
        buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        // Bails, so the walk's own free never runs and this one owns the
        // buffer.
        unsafe { xfree(buf.cast::<c_void>()) };
        err(E474_EXT);
        Flow::Fail
    }

    /// A blob becomes an array of byte values — JSON has nothing shorter.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_blob`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_blob(&mut self, _tv: *mut TypVal, blob: *const Blob, len: c_int) {
        if len == 0 {
            self.gap.extend_from_slice(b"[]");
            return;
        }
        self.gap.push(b'[');
        for i in 0..len {
            if i > 0 {
                self.gap.extend_from_slice(b", ");
            }
            self.concat_num(c"%d", c_int::from(unsafe { tv_blob_get(blob, i) }));
        }
        self.gap.push(b']');
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_func_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_func_start(
        &mut self,
        _tv: *mut TypVal,
        _fun: *mut c_char,
        _prefix: &'static CStr,
        path: &ConvPath,
    ) -> Flow {
        unsafe { conv_error(gettext(E474_FUNCREF).as_ptr(), path) }
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_empty_list`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_empty_list(&mut self, _tv: *mut TypVal) {
        self.gap.extend_from_slice(b"[]");
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_empty_dict`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_empty_dict(&mut self, _tv: *mut TypVal, _dictp: Option<DictSlot>) {
        self.gap.extend_from_slice(b"{}");
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_list_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_list_start(&mut self, _tv: *mut TypVal, _len: c_int) -> Flow {
        self.gap.push(b'[');
        Flow::Go
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_list_between_items`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_list_between_items(&mut self, _tv: *mut TypVal) {
        self.gap.extend_from_slice(b", ");
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_list_end`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_list_end(&mut self, _tv: *mut TypVal) {
        self.gap.push(b']');
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_start(&mut self, _tv: *mut TypVal, _len: size_t) -> Flow {
        self.gap.push(b'{');
        Flow::Go
    }

    /// A special map may carry any typval as a key; JSON may not.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::special_dict_key_check`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn special_dict_key_check(&mut self, key: *const TypVal) -> Flow {
        if unsafe { encode_check_json_key(key) } {
            Flow::Go
        } else {
            err(E474_INVALID_KEY);
            Flow::Fail
        }
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_after_key`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_after_key(&mut self, _tv: *mut TypVal, _dictp: Option<DictSlot>) {
        self.gap.extend_from_slice(b": ");
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_between_items`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_between_items(&mut self, _tv: *mut TypVal, _dictp: Option<DictSlot>) {
        self.gap.extend_from_slice(b", ");
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_dict_end`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_dict_end(&mut self, _tv: *mut TypVal, _dictp: Option<DictSlot>) {
        self.gap.push(b'}');
    }

    /// Say so once per encode, then leave the value out entirely.
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
    gap: &mut Vec<u8>,
    tv: *mut TypVal,
    objname: *const c_char,
) -> bool {
    let mut sink = JsonSink { gap };
    unsafe { encode_typval(&mut sink, tv, objname) }
}
