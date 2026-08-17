//! `string()` and `:echo`: a typval as the Vimscript source text that would
//! rebuild it.
//!
//! One [`TypvalSink`] over the output [`Gap`], replacing *two* instantiations
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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::eval::encode::did_echo_string_emsg;
use crate::eval::typval::tv_blob_get;
use crate::eval::typval_encode::{ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::garray::Gap;
use crate::message::{emsg, internal_error};
use crate::os::libc::{gettext, strlen};
use crate::strings::vim_snprintf_safelen;
use crate::types::{blob_T, dict_T, float_T, garray_T, int64_t, ptrdiff_t, size_t, typval_T};

/// `NUMBUFLEN`: the scratch buffer every `printf`-formatted number goes
/// through.
const NUMBUFLEN: usize = 65;

/// Upstream's `char ebuf[NUMBUFLEN + 7]`, sized for the longest marker.
const MARKERBUFLEN: usize = NUMBUFLEN + 7;

const E724_SELF_REFERENCE: &CStr =
    c"E724: unable to correctly dump variable with self-referencing container";
const NULL_FUNC_NAME: &CStr = c"string(): NULL function name";

/// The `string()`/`:echo` sink.
///
/// `ECHO` picks between upstream's two instantiations.  It is a `const`
/// parameter rather than a field so that each of the two monomorphisations is
/// exactly the code its macro expansion was, with no branch left at run time.
struct TextSink<'a, const ECHO: bool> {
    gap: Gap<'a>,
}

/// Raise `msg`, which carries no arguments.
fn err(msg: &CStr) {
    unsafe { emsg(gettext(msg.as_ptr())) };
}

impl<const ECHO: bool> TextSink<'_, ECHO> {
    /// Append one number, formatted the way C's `printf` would.
    ///
    /// `N` is the size of the stack buffer upstream declares at the site:
    /// `NUMBUFLEN` for a number, [`MARKERBUFLEN`] for a self-reference marker.
    fn concat_num<const N: usize, T>(&mut self, fmt: &CStr, num: T) {
        let mut numbuf = [0 as c_char; N];
        let formatted = unsafe {
            let len = vim_snprintf_safelen(numbuf.as_mut_ptr(), N, fmt.as_ptr(), num);
            ::core::slice::from_raw_parts(numbuf.as_ptr() as *const u8, len)
        };
        self.gap.concat(formatted);
    }

    /// A Vimscript string literal: single-quoted, with every `'` doubled.
    ///
    /// A NULL buffer is `''`; the bytes are copied as they are, NULs and
    /// invalid UTF-8 included, because a Vimscript string is bytes.
    ///
    /// # Safety
    /// `buf` must be NULL or point at `len` readable bytes.
    unsafe fn quoted(&mut self, buf: *const c_char, len: size_t) {
        if buf.is_null() {
            self.gap.concat(b"''");
            return;
        }
        let bytes = unsafe { ::core::slice::from_raw_parts(buf.cast::<u8>(), len) };
        let quotes = bytes.iter().filter(|&&c| c == b'\'').count();
        self.gap.grow((2 + len + quotes) as c_int);
        self.gap.append(b'\'');
        for &c in bytes {
            if c == b'\'' {
                self.gap.append(b'\'');
            }
            self.gap.append(c);
        }
        self.gap.append(b'\'');
    }

    /// How far down the stack the container being re-entered sits — the `N` in
    /// the `@N` marker.
    ///
    /// Upstream compares the frame's *tag* before its pointer, and its two
    /// pointer comparisons cover only `kMPConvDict` and `kMPConvList`.  A
    /// `Pairs` frame is therefore found by neither, and the answer for one is
    /// the depth of the whole stack; [`ConvFrame::container`] keeps that
    /// asymmetry deliberately.
    ///
    /// [`ConvFrame::container`]: crate::eval::typval_encode::ConvFrame::container
    fn backref(path: &ConvPath, val: *mut c_void, conv_type: ConvType) -> size_t {
        let mut backref = 0;
        for frame in path.stack.iter() {
            if conv_type != ConvType::Pairs
                && frame.container() == Some((conv_type, val.cast_const()))
            {
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

    unsafe fn conv_nil(&mut self, _tv: *mut typval_T) {
        self.gap.concat(b"v:null");
    }

    unsafe fn conv_bool(&mut self, _tv: *mut typval_T, num: bool) {
        self.gap.concat(if num {
            b"v:true".as_slice()
        } else {
            b"v:false".as_slice()
        });
    }

    unsafe fn conv_number(&mut self, _tv: *mut typval_T, num: int64_t) {
        self.concat_num::<NUMBUFLEN, _>(c"%ld", num);
    }

    /// NaN and infinity have no Vimscript literal, so they come out as the
    /// `str2float()` call that rebuilds them.
    unsafe fn conv_float(&mut self, _tv: *mut typval_T, flt: float_T) -> Flow {
        match flt.classify() {
            ::core::num::FpCategory::Nan => self.gap.concat(b"str2float('nan')"),
            ::core::num::FpCategory::Infinite => {
                if flt < 0.0 {
                    self.gap.append(b'-');
                }
                self.gap.concat(b"str2float('inf')");
            }
            _ => self.concat_num::<NUMBUFLEN, _>(c"%g", flt),
        }
        Flow::Go
    }

    unsafe fn conv_string(&mut self, _tv: *mut typval_T, buf: *mut c_char, len: size_t) -> Flow {
        unsafe { self.quoted(buf, len) };
        Flow::Go
    }

    /// Unreachable: this sink refuses special dictionaries, which are the only
    /// source of an `ext` value.  Upstream's macro is empty, and falling
    /// through leaves the buffer for the walk to free.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: *mut typval_T,
        _buf: *mut c_char,
        _len: size_t,
        _ext_type: i8,
    ) -> Flow {
        Flow::Go
    }

    unsafe fn conv_blob(&mut self, _tv: *mut typval_T, blob: *const blob_T, len: c_int) {
        if len == 0 {
            self.gap.concat(b"0z");
            return;
        }
        // Room for "0z", two hex digits a byte, and a "." after every eight
        // digits: "0z00112233.44556677.8899".
        self.gap.grow(2 + 2 * len + (len - 1) / 4);
        self.gap.concat(b"0z");
        for i in 0..len {
            if i > 0 && (i & 3) == 0 {
                self.gap.append(b'.');
            }
            self.concat_num::<NUMBUFLEN, _>(c"%02X", unsafe { tv_blob_get(blob, i) } as c_int);
        }
    }

    /// `function('name'` — the closing paren is [`Self::conv_func_end`]'s.
    unsafe fn conv_func_start(
        &mut self,
        _tv: *mut typval_T,
        fun: *mut c_char,
        prefix: &'static CStr,
        _path: &ConvPath,
    ) -> Flow {
        if fun.is_null() {
            unsafe { internal_error(NULL_FUNC_NAME.as_ptr()) };
            self.gap.concat(b"function(NULL");
            return Flow::Go;
        }
        self.gap.concat(b"function(");
        // The prefix is written *before* the quoted name and then swapped with
        // its opening quote in place: `g:'Name'` becomes `'g:Name'`.  Doing it
        // this way rather than by building the name first is upstream's, and
        // it means the quoting below never has to know about the prefix.
        let name_off = self.gap.0.ga_len as usize;
        let prefix = prefix.to_bytes();
        self.gap.concat(prefix);
        unsafe {
            self.quoted(fun, strlen(fun));
            let data = self.gap.0.ga_data.cast::<u8>();
            *data.add(name_off) = b'\'';
            ::core::ptr::copy_nonoverlapping(prefix.as_ptr(), data.add(name_off + 1), prefix.len());
        }
        Flow::Go
    }

    unsafe fn conv_func_before_args(&mut self, _tv: *mut typval_T, len: ptrdiff_t) {
        if len != 0 {
            self.gap.concat(b", ");
        }
    }

    unsafe fn conv_func_before_self(&mut self, _tv: *mut typval_T, len: ptrdiff_t) {
        if len != -1 {
            self.gap.concat(b", ");
        }
    }

    unsafe fn conv_func_end(&mut self, _tv: *mut typval_T, _copyid: c_int) {
        self.gap.append(b')');
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

    /// The one hook the two instantiations disagree about.
    ///
    /// Both keep going — a self-reference is a marker in the output, not a
    /// failed dump — but only `string()` reports it, and only once per dump so
    /// a cycle seen many times does not flood the user.
    unsafe fn conv_recurse(
        &mut self,
        val: *mut c_void,
        conv_type: ConvType,
        path: &ConvPath,
    ) -> Flow {
        if !ECHO && !did_echo_string_emsg.get() {
            did_echo_string_emsg.set(true);
            err(E724_SELF_REFERENCE);
        }
        let backref = Self::backref(path, val, conv_type);
        let fmt = if !ECHO {
            c"{E724@%zu}"
        } else if conv_type == ConvType::Dict {
            c"{...@%zu}"
        } else {
            c"[...@%zu]"
        };
        self.concat_num::<MARKERBUFLEN, _>(fmt, backref);
        Flow::Go
    }
}

/// Append `tv` to `gap` as the text `string()` answers.
///
/// # Safety
/// `gap` must be a live byte-item garray, `tv` a live typval and `objname`
/// NUL-terminated.
pub(crate) unsafe fn encode_vim_to_string(
    gap: *mut garray_T,
    tv: *mut typval_T,
    objname: *const c_char,
) -> bool {
    unsafe {
        let mut sink = TextSink::<false> {
            gap: Gap(&mut *gap),
        };
        encode_typval(&mut sink, tv, objname)
    }
}

/// Append `tv` to `gap` as the text `:echo` prints.
///
/// # Safety
/// As [`encode_vim_to_string`].
pub(crate) unsafe fn encode_vim_to_echo(
    gap: *mut garray_T,
    tv: *mut typval_T,
    objname: *const c_char,
) -> bool {
    unsafe {
        let mut sink = TextSink::<true> {
            gap: Gap(&mut *gap),
        };
        encode_typval(&mut sink, tv, objname)
    }
}
