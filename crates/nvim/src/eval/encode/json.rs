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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::eval::encode::{
    Dump, EncodeError, conv_error, convert_to_json_string, encode_check_json_key, push_decimal,
    push_float_g, report_self_reference,
};
use crate::eval::typval_encode::{Container, ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::types::{Float, TypVal};

struct JsonSink<'a> {
    gap: &'a mut Vec<u8>,
}

impl TypvalSink for JsonSink<'_> {
    const ALLOW_SPECIALS: bool = true;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_json_convert_one_value()";

    fn conv_nil(&mut self) {
        self.gap.extend_from_slice(b"null");
    }

    fn conv_bool(&mut self, num: bool) {
        self.gap.extend_from_slice(if num {
            b"true".as_slice()
        } else {
            b"false".as_slice()
        });
    }

    fn conv_number(&mut self, num: i64) {
        push_decimal(self.gap, num);
    }

    fn conv_unsigned_number(&mut self, num: u64) {
        push_decimal(self.gap, num);
    }

    fn conv_float(&mut self, flt: Float) -> Flow {
        match flt.classify() {
            ::core::num::FpCategory::Nan => EncodeError::Nan.refuse(),
            ::core::num::FpCategory::Infinite => EncodeError::Infinity.refuse(),
            _ => {
                push_float_g(self.gap, flt);
                Flow::Go
            }
        }
    }

    /// Escaped, quoted UTF-8.  A string that is not valid UTF-8 is a failure,
    /// which is what makes this the hook JSON most often refuses on.
    fn conv_string(&mut self, bytes: &[u8]) -> Flow {
        match convert_to_json_string(self.gap, bytes) {
            Ok(()) => Flow::Go,
            Err(refusal) => refusal.refuse(),
        }
    }

    fn conv_ext_string(&mut self, _bytes: &[u8], _ext_type: i8) -> Flow {
        EncodeError::ExtString.refuse()
    }

    /// A blob becomes an array of byte values — JSON has nothing shorter.
    fn conv_blob(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            self.gap.extend_from_slice(b"[]");
            return;
        }
        self.gap.push(b'[');
        for (at, &byte) in bytes.iter().enumerate() {
            if at > 0 {
                self.gap.extend_from_slice(b", ");
            }
            push_decimal(self.gap, byte);
        }
        self.gap.push(b']');
    }

    fn conv_func_start(
        &mut self,
        _fun: Option<&CStr>,
        _prefix: &'static CStr,
        path: &ConvPath<'_, '_>,
    ) -> Flow {
        conv_error(Dump::JsonFuncref, path)
    }

    fn conv_empty_list(&mut self) {
        self.gap.extend_from_slice(b"[]");
    }

    fn conv_empty_dict(&mut self) {
        self.gap.extend_from_slice(b"{}");
    }

    fn conv_list_start(&mut self, _len: c_int) -> Flow {
        self.gap.push(b'[');
        Flow::Go
    }

    fn conv_list_between_items(&mut self) {
        self.gap.extend_from_slice(b", ");
    }

    fn conv_list_end(&mut self) {
        self.gap.push(b']');
    }

    fn conv_dict_start(&mut self, _len: usize) -> Flow {
        self.gap.push(b'{');
        Flow::Go
    }

    /// A special map may carry any typval as a key; JSON may not.
    fn special_dict_key_check(&mut self, key: &TypVal) -> Flow {
        if encode_check_json_key(key) {
            Flow::Go
        } else {
            EncodeError::InvalidKey.refuse()
        }
    }

    fn conv_dict_after_key(&mut self) {
        self.gap.extend_from_slice(b": ");
    }

    fn conv_dict_between_items(&mut self) {
        self.gap.extend_from_slice(b", ");
    }

    fn conv_dict_end(&mut self) {
        self.gap.push(b'}');
    }

    /// Say so once per encode, then leave the value out entirely.
    fn conv_recurse(
        &mut self,
        _container: Container<'_>,
        _conv_type: ConvType,
        _path: &ConvPath<'_, '_>,
    ) -> Flow {
        report_self_reference();
        Flow::Go
    }
}

/// Append `tv` to `gap` as JSON.
pub(crate) fn encode_vim_to_json(gap: &mut Vec<u8>, tv: &TypVal, objname: &CStr) -> bool {
    let mut sink = JsonSink { gap };
    encode_typval(&mut sink, tv, objname)
}
