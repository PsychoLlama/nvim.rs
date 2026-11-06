//! Why a value would not encode.
//!
//! The JSON sink's refusals: what JSON cannot say (NaN, infinity, `ext`
//! strings, a special dictionary's non-string key) and a string that is not
//! UTF-8. The hook that meets one answers [`Flow::Fail`] through
//! [`EncodeError::refuse`], which says it -- the walk stops there, so the
//! hook is the boundary. A dump that fails part-way down names the path to
//! the value instead; that is [`Dump`], rendered by
//! [`conv_error`](super::conv_error).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval_encode::Flow;
use crate::memory::ThinCString;
use crate::message_fmt::{emsg_text, msg_bytes, msg_cstr};
use crate::tr;

/// A value JSON cannot spell, or a string that cannot be JSON.
#[derive(Debug, thiserror::Error)]
pub(crate) enum EncodeError {
    #[error("{}", tr!("E474: Unable to represent NaN value in JSON"))]
    Nan,
    #[error("{}", tr!("E474: Unable to represent infinity in JSON"))]
    Infinity,
    #[error("{}", tr!("E474: Unable to convert EXT string to JSON"))]
    ExtString,
    #[error("{}", tr!("E474: Invalid key in special dictionary"))]
    InvalidKey,
    /// A byte that starts no UTF-8 character, quoting the string from it on.
    #[error(
        "{}",
        tr!(
            "E474: String \"{}\" contains byte that does not start any UTF-8 character",
            msg_cstr(.0.as_cstr())
        )
    )]
    BadUtf8(ThinCString),
    /// A UTF-16 surrogate spelled as UTF-8, quoting the string from it on.
    #[error(
        "{}",
        tr!(
            "E474: UTF-8 string contains code point which belongs to a surrogate pair: {}",
            msg_cstr(.0.as_cstr())
        )
    )]
    Surrogate(ThinCString),
}

const _: () = assert!(size_of::<EncodeError>() <= 16);

impl EncodeError {
    /// Say the refusal and stop the walk.
    #[cold]
    #[inline(never)]
    pub(crate) fn refuse(self) -> Flow {
        emsg_text(self.to_string());
        Flow::Fail
    }
}

/// A dump that stopped at a value, by the sink that refused it. The message
/// names the object being dumped and the path down to the value.
#[derive(Copy, Clone, Debug)]
pub(crate) enum Dump {
    /// `json_encode()` met a function reference.
    JsonFuncref,
    /// `msgpackdump()` met a function reference.
    MsgpackFuncref,
    /// `msgpackdump()` met a container already being dumped.
    MsgpackSelfReference,
}

impl Dump {
    /// The message, naming `objname` and the path `place`.
    pub(crate) fn message(self, objname: &core::ffi::CStr, place: &[u8]) -> String {
        let (objname, place) = (msg_cstr(objname), msg_bytes(place));
        match self {
            Dump::JsonFuncref => tr!(
                "E474: Error while dumping {}, {}: attempt to dump function reference",
                objname,
                place
            ),
            Dump::MsgpackFuncref => tr!(
                "E5004: Error while dumping {}, {}: attempt to dump function reference",
                objname,
                place
            ),
            Dump::MsgpackSelfReference => tr!(
                "E5005: Unable to dump {}: container references itself in {}",
                objname,
                place
            ),
        }
    }
}
