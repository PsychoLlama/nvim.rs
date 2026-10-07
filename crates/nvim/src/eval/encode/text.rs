//! `string()` and `:echo`: a typval as the Vimscript source text that would
//! rebuild it.
//!
//! One [`TypvalSink`] over an output `Vec<u8>`, replacing *two* instantiations
//! of `typval_encode.c.h` — `TYPVAL_ENCODE_NAME string` and
//! `TYPVAL_ENCODE_NAME echo`.  Upstream includes the header twice with one
//! macro changed between them, `TYPVAL_ENCODE_CONV_RECURSE`, so the two
//! emitted converters are byte-identical apart from their names; here they are
//! one `impl` with a `const ECHO: bool` that the compiler folds away, and the
//! whole difference is [`TextSink::conv_recurse`]:
//!
//! - `string()` reports `E724` once per dump and writes `{E724@N}`;
//! - `:echo` says nothing and writes `[...@N]` or `{...@N}`.
//!
//! `N` counts down the walk's stack to the frame the container is already on
//! — see [`TextSink::backref`].
//!
//! Neither reads `{_TYPE, _VAL}` special dictionaries (`ALLOW_SPECIALS` is
//! false): those are a msgpack/JSON round-trip device, and `string()` prints
//! them as the plain two-key dictionaries they are.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::eval::encode::{push_decimal, push_float_g, push_hex_byte, report_self_reference};
use crate::eval::typval_encode::{Container, ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::message::internal_error;
use crate::types::{Float, TypVal};

const NULL_FUNC_NAME: &CStr = c"string(): NULL function name";

/// The `string()`/`:echo` sink.
///
/// `ECHO` picks between upstream's two instantiations.  It is a `const`
/// parameter rather than a field so that each of the two monomorphisations is
/// exactly the code its macro expansion was, with no branch left at run time.
struct TextSink<'a, const ECHO: bool> {
    gap: &'a mut Vec<u8>,
}

impl<const ECHO: bool> TextSink<'_, ECHO> {
    /// A Vimscript string literal: single-quoted, with every `'` doubled.
    ///
    /// The bytes are copied as they are, NULs and invalid UTF-8 included,
    /// because a Vimscript string is bytes.
    fn quoted(&mut self, bytes: &[u8]) {
        let quotes = bytes.iter().filter(|&&c| c == b'\'').count();
        self.gap.reserve(2 + bytes.len() + quotes);
        self.gap.push(b'\'');
        for &c in bytes {
            if c == b'\'' {
                self.gap.push(b'\'');
            }
            self.gap.push(c);
        }
        self.gap.push(b'\'');
    }

    /// How far down the stack the container being re-entered sits — the `N` in
    /// the `@N` marker.
    ///
    /// Upstream compares the frame's *tag* before its pointer, and its two
    /// pointer comparisons cover only `kMPConvDict` and `kMPConvList`.  A
    /// `Pairs` frame is therefore found by neither, and the answer for one is
    /// the depth of the whole stack; [`Frame::walks`] keeps that asymmetry
    /// deliberately.
    ///
    /// [`Frame::walks`]: crate::eval::typval_encode::Frame::walks
    fn backref(path: &ConvPath<'_, '_>, container: Container<'_>, conv_type: ConvType) -> usize {
        let mut backref = 0;
        for frame in path.stack.iter() {
            if conv_type != ConvType::Pairs && frame.walks(conv_type, container) {
                break;
            }
            backref += 1;
        }
        backref
    }
}

impl<const ECHO: bool> TypvalSink for TextSink<'_, ECHO> {
    const ALLOW_SPECIALS: bool = false;
    const CONVERT_FN_NAME: &'static CStr = if ECHO {
        c"_typval_encode_echo_convert_one_value()"
    } else {
        c"_typval_encode_string_convert_one_value()"
    };

    fn conv_nil(&mut self) {
        self.gap.extend_from_slice(b"v:null");
    }

    fn conv_bool(&mut self, num: bool) {
        self.gap.extend_from_slice(if num {
            b"v:true".as_slice()
        } else {
            b"v:false".as_slice()
        });
    }

    fn conv_number(&mut self, num: i64) {
        push_decimal(self.gap, num);
    }

    /// NaN and infinity have no Vimscript literal, so they come out as the
    /// `str2float()` call that rebuilds them.
    fn conv_float(&mut self, flt: Float) -> Flow {
        match flt.classify() {
            ::core::num::FpCategory::Nan => self.gap.extend_from_slice(b"str2float('nan')"),
            ::core::num::FpCategory::Infinite => {
                if flt < 0.0 {
                    self.gap.push(b'-');
                }
                self.gap.extend_from_slice(b"str2float('inf')");
            }
            _ => push_float_g(self.gap, flt),
        }
        Flow::Go
    }

    fn conv_string(&mut self, bytes: &[u8]) -> Flow {
        self.quoted(bytes);
        Flow::Go
    }

    /// Unreachable: this sink refuses special dictionaries, which are the only
    /// source of an `ext` value.  Upstream's macro is empty.
    fn conv_ext_string(&mut self, _bytes: &[u8], _ext_type: i8) -> Flow {
        Flow::Go
    }

    fn conv_blob(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            self.gap.extend_from_slice(b"0z");
            return;
        }
        // Room for "0z", two hex digits a byte, and a "." after every four
        // bytes: "0z00112233.44556677.8899".
        self.gap
            .reserve(2 + 2 * bytes.len() + (bytes.len() - 1) / 4);
        self.gap.extend_from_slice(b"0z");
        for (at, &byte) in bytes.iter().enumerate() {
            if at > 0 && (at & 3) == 0 {
                self.gap.push(b'.');
            }
            push_hex_byte(self.gap, byte);
        }
    }

    /// `function('name'` — the closing paren is [`Self::conv_func_end`]'s.
    fn conv_func_start(
        &mut self,
        fun: Option<&CStr>,
        prefix: &'static CStr,
        _path: &ConvPath<'_, '_>,
    ) -> Flow {
        let Some(fun) = fun else {
            internal_error(NULL_FUNC_NAME);
            self.gap.extend_from_slice(b"function(NULL");
            return Flow::Go;
        };
        self.gap.extend_from_slice(b"function(");
        // The prefix is written *before* the quoted name and then swapped with
        // its opening quote in place: `g:'Name'` becomes `'g:Name'`.  Doing it
        // this way rather than by building the name first is upstream's, and
        // it means the quoting below never has to know about the prefix.
        let name_off = self.gap.len();
        let prefix = prefix.to_bytes();
        self.gap.extend_from_slice(prefix);
        self.quoted(fun.to_bytes());
        self.gap[name_off] = b'\'';
        self.gap[name_off + 1..=name_off + prefix.len()].copy_from_slice(prefix);
        Flow::Go
    }

    fn conv_func_before_args(&mut self, len: usize) {
        if len != 0 {
            self.gap.extend_from_slice(b", ");
        }
    }

    fn conv_func_before_self(&mut self, len: Option<usize>) {
        if len.is_some() {
            self.gap.extend_from_slice(b", ");
        }
    }

    fn conv_func_end(&mut self) {
        self.gap.push(b')');
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

    fn conv_dict_after_key(&mut self) {
        self.gap.extend_from_slice(b": ");
    }

    fn conv_dict_between_items(&mut self) {
        self.gap.extend_from_slice(b", ");
    }

    fn conv_dict_end(&mut self) {
        self.gap.push(b'}');
    }

    /// The one hook the two instantiations disagree about.
    ///
    /// Both keep going — a self-reference is a marker in the output, not a
    /// failed dump — but only `string()` reports it, and only once per dump so
    /// a cycle seen many times does not flood the user.
    fn conv_recurse(
        &mut self,
        container: Container<'_>,
        conv_type: ConvType,
        path: &ConvPath<'_, '_>,
    ) -> Flow {
        if !ECHO {
            report_self_reference();
        }
        let backref = Self::backref(path, container, conv_type);
        let (open, close) = if !ECHO {
            (&b"{E724@"[..], b'}')
        } else if conv_type == ConvType::Dict {
            (&b"{...@"[..], b'}')
        } else {
            (&b"[...@"[..], b']')
        };
        self.gap.extend_from_slice(open);
        push_decimal(self.gap, u64::try_from(backref).unwrap_or(u64::MAX));
        self.gap.push(close);
        Flow::Go
    }
}

/// Append `tv` to `gap` as the text `string()` answers.
pub(crate) fn encode_vim_to_string(gap: &mut Vec<u8>, tv: &TypVal, objname: &CStr) -> bool {
    let mut sink = TextSink::<false> { gap };
    encode_typval(&mut sink, tv, objname)
}

/// Append `tv` to `gap` as the text `:echo` prints.
pub(crate) fn encode_vim_to_echo(gap: &mut Vec<u8>, tv: &TypVal, objname: &CStr) -> bool {
    let mut sink = TextSink::<true> { gap };
    encode_typval(&mut sink, tv, objname)
}
