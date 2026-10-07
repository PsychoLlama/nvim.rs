//! `msgpackdump()`, ShaDa and the RPC wire format: a typval as msgpack.
//!
//! One [`TypvalSink`] over a [`PackerBuffer`], replacing the
//! `TYPVAL_ENCODE_NAME msgpack` instantiation of `typval_encode.c.h`.  This is
//! the sink that reads the `{_TYPE, _VAL}` special dictionaries as the msgpack
//! types they stand for — they exist so that a value msgpack can carry but
//! Vimscript cannot survives a round trip — and the one that refuses both
//! function references and self-referencing containers outright.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::eval::encode::conv_error;
use crate::eval::typval_encode::{Container, ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::msgpack_rpc::packer::{
    mpack_array, mpack_bin, mpack_bool, mpack_check_buffer, mpack_ext, mpack_float8, mpack_integer,
    mpack_map, mpack_nil, mpack_str, mpack_uint64,
};
use crate::types::{Float, PackerBuffer, TypVal};

/// The two errors this sink can raise, both through
/// [`conv_error`][crate::eval::encode::conv_error], which appends
/// the path down to the offending value.
const E5004_FUNCREF: &CStr =
    c"E5004: Error while dumping %s, %s: attempt to dump function reference";
const E5005_SELF_REFERENCE: &CStr = c"E5005: Unable to dump %s: container references itself in %s";

struct MsgpackSink<'a> {
    packer: &'a mut PackerBuffer,
}

impl TypvalSink for MsgpackSink<'_> {
    const ALLOW_SPECIALS: bool = true;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_msgpack_convert_one_value()";

    /// msgpack is written straight into a fixed buffer, so every item starts
    /// by making sure there is room for a header.
    fn check_before(&mut self) {
        mpack_check_buffer(self.packer);
    }

    fn conv_nil(&mut self) {
        mpack_nil(self.packer.cursor_mut());
    }

    fn conv_bool(&mut self, num: bool) {
        mpack_bool(self.packer.cursor_mut(), num);
    }

    fn conv_number(&mut self, num: i64) {
        mpack_integer(self.packer.cursor_mut(), num);
    }

    fn conv_unsigned_number(&mut self, num: u64) {
        mpack_uint64(self.packer.cursor_mut(), num);
    }

    fn conv_float(&mut self, flt: Float) -> Flow {
        mpack_float8(self.packer.cursor_mut(), flt);
        Flow::Go
    }

    /// A Vimscript string is bytes, not text: it can hold NULs and invalid
    /// UTF-8, so it goes out as `bin`.
    fn conv_string(&mut self, bytes: &[u8]) -> Flow {
        mpack_bin(bytes, self.packer);
        Flow::Go
    }

    /// A dictionary key, or a `{_TYPE: string}` payload: text, so `str`.
    fn conv_str_string(&mut self, bytes: &[u8]) -> Flow {
        mpack_str(bytes, self.packer);
        Flow::Go
    }

    fn conv_ext_string(&mut self, bytes: &[u8], ext_type: i8) -> Flow {
        mpack_ext(bytes, ext_type, self.packer);
        Flow::Go
    }

    fn conv_blob(&mut self, bytes: &[u8]) {
        mpack_bin(bytes, self.packer);
    }

    fn conv_func_start(
        &mut self,
        _fun: Option<&CStr>,
        _prefix: &'static CStr,
        path: &ConvPath<'_, '_>,
    ) -> Flow {
        conv_error(E5004_FUNCREF, path)
    }

    fn conv_empty_list(&mut self) {
        mpack_array(self.packer.cursor_mut(), 0);
    }

    fn conv_empty_dict(&mut self) {
        mpack_map(self.packer.cursor_mut(), 0);
    }

    fn conv_list_start(&mut self, len: c_int) -> Flow {
        let len = u32::try_from(len).expect("a list length is never negative");
        mpack_array(self.packer.cursor_mut(), len);
        Flow::Go
    }

    fn conv_dict_start(&mut self, len: usize) -> Flow {
        let len = u32::try_from(len).expect("a dict never holds four billion keys");
        mpack_map(self.packer.cursor_mut(), len);
        Flow::Go
    }

    /// msgpack has no way to spell a cycle, so this is where a dump gives up.
    fn conv_recurse(
        &mut self,
        _container: Container<'_>,
        _conv_type: ConvType,
        path: &ConvPath<'_, '_>,
    ) -> Flow {
        conv_error(E5005_SELF_REFERENCE, path)
    }
}

/// Pack `tv` into `packer`, answering whether the whole value went.
///
/// `objname` names the value in any error message.
pub fn encode_vim_to_msgpack(packer: &mut PackerBuffer, tv: &TypVal, objname: &CStr) -> bool {
    let mut sink = MsgpackSink { packer };
    encode_typval(&mut sink, tv, objname)
}
