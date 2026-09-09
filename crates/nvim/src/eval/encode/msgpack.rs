//! `msgpackdump()`, ShaDa and the RPC wire format: a typval as msgpack.
//!
//! One [`TypvalSink`] over a [`PackerBuffer`], replacing the
//! `TYPVAL_ENCODE_NAME msgpack` instantiation of `typval_encode.c.h`.  This is
//! the sink that reads the `{_TYPE, _VAL}` special dictionaries as the msgpack
//! types they stand for — they exist so that a value msgpack can carry but
//! Vimscript cannot survives a round trip — and the one that refuses both
//! function references and self-referencing containers outright.

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

use crate::eval::encode::conv_error;
use crate::eval::typval::DictSlot;
use crate::eval::typval_encode::{ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_bin, mpack_bool, mpack_check_buffer, mpack_ext, mpack_float8, mpack_integer,
    mpack_map, mpack_nil, mpack_str, mpack_uint64,
};
use crate::os::cshim::gettext;
use crate::types::{Blob, Float, Integer, PackerBuffer, String_0, TypVal, int64_t, size_t};

/// The two errors this sink can raise, both through
/// [`conv_error`][crate::eval::encode::conv_error], which appends
/// the path down to the offending value.
const E5004_FUNCREF: &CStr =
    c"E5004: Error while dumping %s, %s: attempt to dump function reference";
const E5005_SELF_REFERENCE: &CStr = c"E5005: Unable to dump %s: container references itself in %s";

struct MsgpackSink<'a> {
    packer: &'a mut PackerBuffer,
}

impl MsgpackSink<'_> {
    /// A string as msgpack's `String_0` sees it: pointer and length, no NUL.
    fn buf(data: *mut c_char, size: size_t) -> String_0 {
        String_0::from_raw_parts(data, size)
    }
}

impl TypvalSink for MsgpackSink<'_> {
    const ALLOW_SPECIALS: bool = true;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_msgpack_convert_one_value()";

    /// msgpack is written straight into a fixed buffer, so every item starts
    /// by making sure there is room for a header.
    fn check_before(&mut self) {
        mpack_check_buffer(self.packer);
    }

    fn conv_nil(&mut self, _tv: Option<&mut TypVal>) {
        mpack_nil(self.packer.cursor_mut());
    }

    fn conv_bool(&mut self, _tv: Option<&mut TypVal>, num: bool) {
        mpack_bool(self.packer.cursor_mut(), num);
    }

    fn conv_number(&mut self, _tv: Option<&mut TypVal>, num: int64_t) {
        mpack_integer(self.packer.cursor_mut(), num as Integer);
    }

    fn conv_unsigned_number(&mut self, _tv: Option<&mut TypVal>, num: u64) {
        mpack_uint64(self.packer.cursor_mut(), num);
    }

    fn conv_float(&mut self, _tv: Option<&mut TypVal>, flt: Float) -> Flow {
        mpack_float8(self.packer.cursor_mut(), flt);
        Flow::Go
    }

    /// A Vimscript string is bytes, not text: it can hold NULs and invalid
    /// UTF-8, so it goes out as `bin`.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_string(
        &mut self,
        _tv: Option<&mut TypVal>,
        buf: *mut c_char,
        len: size_t,
    ) -> Flow {
        unsafe { mpack_bin(Self::buf(buf, len), self.packer) };
        Flow::Go
    }

    /// A dictionary key, or a `{_TYPE: string}` payload: text, so `str`.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_str_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_str_string(
        &mut self,
        _tv: Option<&mut TypVal>,
        buf: *mut c_char,
        len: size_t,
    ) -> Flow {
        unsafe { mpack_str(Self::buf(buf, len), self.packer) };
        Flow::Go
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_ext_string`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_ext_string(
        &mut self,
        _tv: Option<&mut TypVal>,
        buf: *mut c_char,
        len: size_t,
        ext_type: i8,
    ) -> Flow {
        unsafe { mpack_ext(buf, len, ext_type, self.packer) };
        Flow::Go
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_blob`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_blob(&mut self, _tv: Option<&mut TypVal>, blob: *const Blob, len: c_int) {
        let data = if blob.is_null() {
            ::core::ptr::null_mut()
        } else {
            unsafe { (*blob).bv_ga.ga_data }.cast::<c_char>()
        };
        let len = usize::try_from(len).expect("a blob length is never negative");
        unsafe { mpack_bin(Self::buf(data, len), self.packer) };
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_func_start`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_func_start(
        &mut self,
        _tv: Option<&mut TypVal>,
        _fun: *mut c_char,
        _prefix: &'static CStr,
        path: &ConvPath,
    ) -> Flow {
        unsafe { conv_error(gettext(E5004_FUNCREF).as_ptr(), path) }
    }

    fn conv_empty_list(&mut self, _tv: Option<&mut TypVal>) {
        mpack_array(self.packer.cursor_mut(), 0);
    }

    /// # Safety
    ///
    /// As [`TypvalSink::conv_empty_dict`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_empty_dict(&mut self, _dictp: Option<DictSlot>) {
        mpack_map(self.packer.cursor_mut(), 0);
    }

    fn conv_list_start(&mut self, _tv: Option<&mut TypVal>, len: c_int) -> Flow {
        let len = u32::try_from(len).expect("a list length is never negative");
        mpack_array(self.packer.cursor_mut(), len);
        Flow::Go
    }

    fn conv_dict_start(&mut self, _tv: Option<&mut TypVal>, len: size_t) -> Flow {
        let len = u32::try_from(len).expect("a dict never holds four billion keys");
        mpack_map(self.packer.cursor_mut(), len);
        Flow::Go
    }

    /// msgpack has no way to spell a cycle, so this is where a dump gives up.
    ///
    /// # Safety
    ///
    /// As [`TypvalSink::conv_recurse`]: the walk's contract on the value
    /// it is standing on.
    unsafe fn conv_recurse(
        &mut self,
        _val: *mut c_void,
        _conv_type: ConvType,
        path: &ConvPath,
    ) -> Flow {
        unsafe { conv_error(gettext(E5005_SELF_REFERENCE).as_ptr(), path) }
    }
}

/// Pack `tv` into `packer`.
///
/// `objname` names the value in any error message. Answers `OK`/`FAIL`.
///
/// # Safety
/// `packer` and `tv` must be live, and `objname` NUL-terminated.
pub unsafe fn encode_vim_to_msgpack(
    packer: *mut PackerBuffer,
    tv: *mut TypVal,
    objname: *const c_char,
) -> c_int {
    let mut sink = MsgpackSink {
        packer: unsafe { &mut *packer },
    };
    c_int::from(unsafe { encode_typval(&mut sink, tv, objname) })
}
