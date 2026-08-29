//! `msgpackdump()`, ShaDa and the RPC wire format: a typval as msgpack.
//!
//! One [`TypvalSink`] over a [`PackerBuffer`], replacing the
//! `TYPVAL_ENCODE_NAME msgpack` instantiation of `typval_encode.c.h`.  This is
//! the sink that reads the `{_TYPE, _VAL}` special dictionaries as the msgpack
//! types they stand for — they exist so that a value msgpack can carry but
//! Vimscript cannot survives a round trip — and the one that refuses both
//! function references and self-referencing containers outright.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::eval::encode::conv_error;
use crate::eval::typval_encode::{ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_bin, mpack_bool, mpack_check_buffer, mpack_ext, mpack_float8, mpack_integer,
    mpack_map, mpack_nil, mpack_str, mpack_uint64,
};
use crate::os::cshim::gettext;
use crate::types::{
    Integer, PackerBuffer, String_0, blob_T, dict_T, float_T, int64_t, size_t, typval_T,
};

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
    unsafe fn check_before(&mut self) {
        mpack_check_buffer(self.packer);
    }

    unsafe fn conv_nil(&mut self, _tv: *mut typval_T) {
        mpack_nil(&mut self.packer.ptr);
    }

    unsafe fn conv_bool(&mut self, _tv: *mut typval_T, num: bool) {
        mpack_bool(&mut self.packer.ptr, num);
    }

    unsafe fn conv_number(&mut self, _tv: *mut typval_T, num: int64_t) {
        mpack_integer(&mut self.packer.ptr, num as Integer);
    }

    unsafe fn conv_unsigned_number(&mut self, _tv: *mut typval_T, num: u64) {
        mpack_uint64(&mut self.packer.ptr, num);
    }

    unsafe fn conv_float(&mut self, _tv: *mut typval_T, flt: float_T) -> Flow {
        mpack_float8(&mut self.packer.ptr, flt);
        Flow::Go
    }

    /// A Vimscript string is bytes, not text: it can hold NULs and invalid
    /// UTF-8, so it goes out as `bin`.
    unsafe fn conv_string(&mut self, _tv: *mut typval_T, buf: *mut c_char, len: size_t) -> Flow {
        unsafe { mpack_bin(Self::buf(buf, len), self.packer) };
        Flow::Go
    }

    /// A dictionary key, or a `{_TYPE: string}` payload: text, so `str`.
    unsafe fn conv_str_string(
        &mut self,
        _tv: *mut typval_T,
        buf: *mut c_char,
        len: size_t,
    ) -> Flow {
        unsafe { mpack_str(Self::buf(buf, len), self.packer) };
        Flow::Go
    }

    unsafe fn conv_ext_string(
        &mut self,
        _tv: *mut typval_T,
        buf: *mut c_char,
        len: size_t,
        ext_type: i8,
    ) -> Flow {
        unsafe { mpack_ext(buf, len, ext_type, self.packer) };
        Flow::Go
    }

    unsafe fn conv_blob(&mut self, _tv: *mut typval_T, blob: *const blob_T, len: c_int) {
        let data = if blob.is_null() {
            ::core::ptr::null_mut()
        } else {
            unsafe { (*blob).bv_ga.ga_data }.cast::<c_char>()
        };
        unsafe { mpack_bin(Self::buf(data, len as size_t), self.packer) };
    }

    unsafe fn conv_func_start(
        &mut self,
        _tv: *mut typval_T,
        _fun: *mut c_char,
        _prefix: &'static CStr,
        path: &ConvPath,
    ) -> Flow {
        unsafe { conv_error(gettext(E5004_FUNCREF).as_ptr(), path) }
    }

    unsafe fn conv_empty_list(&mut self, _tv: *mut typval_T) {
        mpack_array(&mut self.packer.ptr, 0);
    }

    unsafe fn conv_empty_dict(&mut self, _tv: *mut typval_T, _dictp: Option<*mut *mut dict_T>) {
        mpack_map(&mut self.packer.ptr, 0);
    }

    unsafe fn conv_list_start(&mut self, _tv: *mut typval_T, len: c_int) -> Flow {
        mpack_array(&mut self.packer.ptr, len as u32);
        Flow::Go
    }

    unsafe fn conv_dict_start(&mut self, _tv: *mut typval_T, len: size_t) -> Flow {
        mpack_map(&mut self.packer.ptr, len as u32);
        Flow::Go
    }

    /// msgpack has no way to spell a cycle, so this is where a dump gives up.
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
    tv: *mut typval_T,
    objname: *const c_char,
) -> c_int {
    let mut sink = MsgpackSink {
        packer: unsafe { &mut *packer },
    };
    c_int::from(unsafe { encode_typval(&mut sink, tv, objname) })
}
