//! `json_decode()` and `msgpackparse()`: text and msgpack bytes into typvals.
//!
//! The direction opposite [`super::encode`], and the shape is opposite too:
//! there is no shared walk here, because upstream writes one parser per
//! format by hand.  The two live in [`json`] and [`msgpack`].
//!
//! What they do share is this file.  Both formats can carry values Vimscript
//! has no type for — a map whose keys are not plain non-empty strings, an
//! unsigned integer past `VARNUMBER_MAX`, a msgpack `ext` — and both answer
//! with the same **special dictionary**, `{_TYPE: v:msgpack_types.<kind>,
//! _VAL: <payload>}`, which [`create_special_dict`] builds and the msgpack
//! encoder reads back.  Both also have to decide, for every run of bytes,
//! whether it is a string or a blob; that is [`decode_string`].

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::memory::ThinCString;

use crate::eval::typval::{ListRef, tv_blob_alloc, tv_dict_alloc, tv_list_alloc};
use crate::eval::vars::msgpack_type_list_ref;
use crate::types::{DictItem, MessagePackType, TypVal, VarLock, ptrdiff_t};

mod json;
mod msgpack;

pub use self::json::json_decode_string;
pub(crate) use self::msgpack::{MsgpackDecoder, unpack_typval};

/// The three `v:msgpack_types` entries the decoders name.  The rest of the
/// enum belongs to the encoder, which declares its own copy.
pub(crate) const kMPInteger: MessagePackType = 2;
pub(crate) const kMPMap: MessagePackType = 6;
pub(crate) const kMPExt: MessagePackType = 7;

/// `{_TYPE: v:msgpack_types.<type>, _VAL: val}`, with one reference.
///
/// `val` is moved into the `_VAL` key, and the `_TYPE` value is a reference
/// to one of the shared, immutable lists `v:msgpack_types` is made of, not a
/// copy of it.
#[inline]
pub(crate) fn create_special_dict(type_: MessagePackType, val: TypVal) -> TypVal {
    let mut dict_held = tv_dict_alloc();

    let mut type_item = DictItem::boxed(b"_TYPE");
    type_item.di_lock = VarLock::Unlocked;
    type_item.di_tv.write_list(msgpack_type_list_ref(type_));
    let _ = dict_held.add_item(type_item);

    let mut val_item = DictItem::boxed(b"_VAL");
    val_item.di_tv.overwrite(val);
    let _ = dict_held.add_item(val_item);

    TypVal::dict(Some(dict_held))
}

/// The special dictionary a map that cannot be a `Dict` decodes to, and a
/// handle on its `_VAL` list for the caller to fill with two-element
/// key/value pairs.
///
/// `len` sizes the `_VAL` list in advance (see `ListLenSpecials`); it is only
/// a hint, and underfilling the list is allowed.
pub fn decode_create_map_special_dict(len: ptrdiff_t) -> (TypVal, ListRef) {
    let list = tv_list_alloc(len);
    let pairs = list.clone();
    (create_special_dict(kMPMap, TypVal::list(Some(list))), pairs)
}

/// `bytes` as a `TypVal`: a `VAR_STRING`, or a `VAR_BLOB` when it cannot be
/// one.
///
/// A Vimscript string is NUL-terminated, so a run containing an embedded NUL
/// has to become a blob; `force_blob` asks for one either way.  The bytes are
/// copied.
pub fn decode_string(bytes: &[u8], force_blob: bool) -> TypVal {
    if force_blob || bytes.contains(&0) {
        let mut blob = tv_blob_alloc();
        blob.extend(bytes);
        return TypVal::blob(Some(blob));
    }
    TypVal::string(Some(ThinCString::from_bytes(bytes)))
}

/// [`decode_string`] for bytes the caller hands over: they are stored as
/// they stand -- as the blob's array, or as the string itself -- rather than
/// copied.
pub(crate) fn decode_owned_string(bytes: Vec<u8>) -> TypVal {
    if bytes.contains(&0) {
        let mut blob = tv_blob_alloc();
        blob.bv_data = bytes;
        return TypVal::blob(Some(blob));
    }
    TypVal::string(Some(ThinCString::from_vec(bytes)))
}

#[cfg(test)]
mod tests {
    //! The msgpack decoder's statuses, and the shapes a cut-off stream leaves.

    use super::msgpack::{MPACK_NOMEM, MsgpackDecoder};
    use super::unpack_typval;
    use crate::eval::encode::tv2string_bytes;
    use crate::global_cell::editor_state_lock;
    use crate::mpack::mpack_core::{MPACK_EOF, MPACK_ERROR, MPACK_OK};
    use crate::types::{TypVal, VAR_UNKNOWN};

    fn unpack(bytes: &[u8]) -> (core::ffi::c_int, TypVal, usize) {
        let mut data = bytes;
        let mut tv = TypVal::Number(7);
        let status = unpack_typval(&mut data, &mut tv);
        (status, tv, data.len())
    }

    fn text(tv: &TypVal) -> String {
        String::from_utf8(tv2string_bytes(tv)).expect("ASCII")
    }

    /// An array whose element is a string cut off mid-way: the array holds
    /// nothing half-written for the failure to trip over.
    #[test]
    fn a_string_cut_off_inside_an_array_is_incomplete() {
        let _serial = editor_state_lock();
        for bytes in [
            &[0x93, 0xc4, 0x01][..],
            &[0x92, 0x01, 0xa2, b'a'],
            &[0x91, 0x81, 0xa1],
        ] {
            let (status, tv, _) = unpack(bytes);
            assert_eq!(status, MPACK_EOF.cast_signed(), "{bytes:x?}");
            assert_eq!(tv.v_type(), VAR_UNKNOWN);
        }
    }

    /// One object per call, the rest left in the slice; a byte that is not
    /// msgpack is an error and consumes nothing.
    #[test]
    fn an_object_is_decoded_and_the_rest_left() {
        let _serial = editor_state_lock();
        let (status, tv, left) = unpack(&[0x92, 0x01, 0xa1, b'x', 0xc0]);
        assert_eq!((status, left), (MPACK_OK.cast_signed(), 1));
        assert_eq!(text(&tv), "[1, 'x']");
        let (status, _, left) = unpack(&[0xc1, 0x00]);
        assert_eq!((status, left), (MPACK_ERROR.cast_signed(), 2));
        // A string holding a NUL is a blob; an empty one is still a string.
        let (_, tv, _) = unpack(&[0x92, 0xa3, b'a', 0, b'b', 0xa0]);
        assert_eq!(text(&tv), "[0z610062, '']");
    }

    /// Thirty-two levels are the limit, a string's bytes counting as one.
    #[test]
    fn nesting_past_the_depth_limit_is_refused() {
        let _serial = editor_state_lock();
        let mut deep = vec![0x91; 31];
        deep.push(0x01);
        assert_eq!(unpack(&deep).0, MPACK_OK.cast_signed());
        let mut too_deep = vec![0x91; 32];
        too_deep.push(0x01);
        assert_eq!(unpack(&too_deep).0, MPACK_NOMEM);
        let mut string_too_deep = vec![0x91; 31];
        string_too_deep.extend_from_slice(&[0xa1, b'x']);
        assert_eq!(unpack(&string_too_deep).0, MPACK_NOMEM);
    }

    /// The streaming form resumes across slices, tokens split included.
    #[test]
    fn a_stream_split_anywhere_decodes_the_same() {
        let _serial = editor_state_lock();
        let whole: &[u8] = &[
            0x82, 0xa1, b'k', 0xcd, 0x01, 0x00, 0xa1, b'l', 0x92, 0xc3, 0xa2, b'h', b'i',
        ];
        for cut in 0..whole.len() {
            let mut decoder = MsgpackDecoder::new();
            let mut head = &whole[..cut];
            assert!(matches!(decoder.feed(&mut head), Ok(None)), "cut at {cut}");
            assert!(head.is_empty());
            let mut tail = &whole[cut..];
            let value = decoder.feed(&mut tail).expect("valid").expect("whole");
            assert_eq!(
                text(&value),
                "{'k': 256, 'l': [v:true, 'hi']}",
                "cut at {cut}"
            );
        }
    }
}
