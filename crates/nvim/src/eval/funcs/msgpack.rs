//! Serialisation: the `msgpack*()` and `json_*()` families.
#![forbid(unsafe_code)]

use super::wrappers::blob_alloc_ret;
use super::{ARENA_BLOCK_SIZE, MPACK_EOF, MPACK_ERROR, MPACK_OK};
use crate::eval::decode::{MsgpackDecoder, json_decode_string, unpack_typval};
use crate::eval::encode::{
    ListRead, ListReader, encode_list_write, encode_tv2json, encode_vim_list_to_buf,
    encode_vim_to_msgpack,
};
use crate::eval::typval::{NumBuf, blob_bytes, tv_list_alloc};
use crate::message_fmt::msg_bytes;
use crate::msgpack_rpc::packer::pack_to_bytes;
use crate::semsg;
use crate::types::{
    Blob, EvalFuncData, List, TypVal, VAR_BLOB, VAR_LIST, VAR_STRING, VAR_UNKNOWN, kListLenMayKnow,
};
use core::ffi::{CStr, c_int};
use core::fmt::Write as _;

/// `json_decode({expr})` — parse JSON from a String, or from a List of
/// lines joined by NLs.
pub fn f_json_decode(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let joined;
    let text: &[u8] = if args[0].v_type() == VAR_LIST {
        let Some(bytes) = encode_vim_list_to_buf(args[0].list_ref()) else {
            semsg!("E474: Failed to convert list to string");
            return;
        };
        joined = bytes;
        &joined
    } else {
        let Some(s) = numbuf.string_chk(&args[0]) else {
            return;
        };
        s.to_bytes()
    };
    if json_decode_string(text, result).is_err() {
        // `%s` of the document: it stops at a NUL a joined List may hold.
        let shown = text
            .iter()
            .position(|&b| b == 0)
            .map_or(text, |nul| &text[..nul]);
        let s = msg_bytes(shown);
        semsg!("E474: Failed to parse {s}");
        result.write_number(0);
    }
    debug_assert!(result.v_type() != VAR_UNKNOWN);
}

/// `json_encode({expr})`.
pub fn f_json_encode(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(Some(encode_tv2json(&args[0])));
}

/// `msgpackdump({list} [, {type}])` — a List of msgpack objects as a List
/// of NL-joined lines, or as a Blob when `{type}` is "B".
pub fn f_msgpackdump(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if args[0].v_type() != VAR_LIST {
        let arg0 = "msgpackdump()";
        semsg!("E686: Argument of {arg0} must be a List");
        return;
    }
    // The per-item label the encoder names in its own error messages.
    // One buffer, reused, as the C's 189-byte stack array was.
    let mut label = String::with_capacity(64);
    let items = args[0].list_ref().map_or(&[][..], List::items);
    // Everything packed up to the first value that refuses.
    let data = pack_to_bytes(|packer| {
        for (idx, item) in items.iter().enumerate() {
            label.clear();
            let _ = write!(label, "msgpackdump() argument, index {idx}\0");
            let what = CStr::from_bytes_with_nul(label.as_bytes()).expect("the NUL above");
            if !encode_vim_to_msgpack(packer, &item.li_tv, what) {
                break;
            }
        }
    });
    if args.len() > 1 && numbuf.bytes(&args[1]) == b"B" {
        // The Blob adopts the packed bytes as they are; nothing copies.
        blob_alloc_ret(result).bv_data = data;
    } else {
        let mut lines = tv_list_alloc(kListLenMayKnow as isize);
        encode_list_write(&mut lines, &data);
        result.write_list(Some(lines));
    }
}

/// Report an unpacker status that is not `MPACK_OK`.
fn emsg_mpack_error(status: c_int) {
    match status.cast_unsigned() {
        MPACK_ERROR => semsg!("E475: Invalid argument: Failed to parse msgpack string"),
        MPACK_EOF => semsg!("E475: Invalid argument: Incomplete msgpack string"),
        // Anything past MPACK_ERROR is the parser's depth limit.
        3 => semsg!("E475: Invalid argument: object was too deep to unpack"),
        _ => return,
    };
}

/// Feed a List of NL-joined strings through the streaming decoder, appending
/// each complete object to `ret_list`.
fn msgpackparse_unpack_list(list: &List, ret_list: &mut List) {
    let Some(first) = list.items().first() else {
        return;
    };
    if first.li_tv.v_type() != VAR_STRING {
        semsg!("E475: Invalid argument: List item is not a string");
        return;
    }
    let mut reader = ListReader::new(list);
    // One arena block's worth at a time, as upstream reads.
    let mut buf = vec![0u8; ARENA_BLOCK_SIZE.cast_unsigned() as usize];
    let mut decoder = MsgpackDecoder::new();
    let mut status = MPACK_OK.cast_signed();
    loop {
        let (rlret, read_bytes) = reader.read(&mut buf);
        let Ok(rlret) = rlret else {
            semsg!("E475: Invalid argument: List item is not a string");
            break;
        };
        // The decoder buffers a partial object itself, so every read starts
        // the block afresh.
        let mut cursor = &buf[..read_bytes];
        while !cursor.is_empty() {
            match decoder.feed(&mut cursor) {
                Ok(Some(value)) => {
                    status = MPACK_OK.cast_signed();
                    ret_list.push(value);
                }
                Ok(None) => status = MPACK_EOF.cast_signed(),
                Err(error) => {
                    status = error;
                    break;
                }
            }
        }
        if rlret == ListRead::Drained || status > MPACK_EOF.cast_signed() {
            break;
        }
    }
    if status != MPACK_OK.cast_signed() {
        emsg_mpack_error(status);
    }
}

/// Unpack a Blob, which is already one contiguous buffer.
fn msgpackparse_unpack_blob(blob: Option<&Blob>, ret_list: &mut List) {
    // `unpack_typval` advances the cursor past each object.
    let mut data = blob_bytes(blob);
    while !data.is_empty() {
        let mut tv = TypVal::Unknown;
        let status = unpack_typval(&mut data, &mut tv);
        if status != MPACK_OK.cast_signed() {
            emsg_mpack_error(status);
            return;
        }
        ret_list.push(tv);
    }
}

/// `msgpackparse({data})` — the objects in a List of strings or a Blob.
pub fn f_msgpackparse(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if args[0].v_type() != VAR_LIST && args[0].v_type() != VAR_BLOB {
        let arg0 = "msgpackparse()";
        semsg!("E899: Argument of {arg0} must be a List or Blob");
        return;
    }
    let mut ret_list = tv_list_alloc(kListLenMayKnow as isize);
    if args[0].v_type() == VAR_LIST {
        if let Some(list) = args[0].list_ref() {
            msgpackparse_unpack_list(list, &mut ret_list);
        }
    } else {
        msgpackparse_unpack_blob(args[0].blob_ref(), &mut ret_list);
    }
    result.write_list(Some(ret_list));
}
#[cfg(test)]
mod tests {
    //! Round trips through the four builtins, and the decoder's statuses on
    //! a truncated stream.

    use super::*;
    use crate::eval::list::string_tv;
    use crate::eval::typval::{ListRef, tv_dict_alloc, tv_list_alloc};
    use crate::global_cell::editor_state_lock;

    fn list(items: Vec<TypVal>) -> ListRef {
        let mut l = tv_list_alloc(-1);
        for tv in items {
            l.push(tv);
        }
        l
    }

    fn call(f: fn(&[TypVal], &mut TypVal, EvalFuncData), args: Vec<TypVal>) -> TypVal {
        let mut result = TypVal::Number(0);
        f(&args, &mut result, EvalFuncData::None);
        result
    }

    /// A value with a container of every kind msgpack carries.
    fn sample() -> TypVal {
        let mut d = tv_dict_alloc();
        d.add_number(b"n", -300).expect("fresh key");
        d.add_value(b"s", string_tv(b"a\nb")).expect("fresh key");
        d.add_list(
            b"l",
            Some(list(vec![TypVal::Float(0.5), TypVal::Number(70_000)])),
        )
        .expect("fresh key");
        TypVal::list(Some(list(vec![
            TypVal::dict(Some(d)),
            TypVal::list(Some(list(vec![]))),
            TypVal::Number(5_000_000_000),
        ])))
    }

    fn bytes_of(blob: &TypVal) -> Vec<u8> {
        blob.blob_ref().map_or_else(Vec::new, |b| b.bv_data.clone())
    }

    #[test]
    fn msgpack_round_trips_through_a_blob_and_a_list_of_lines() {
        let _serial = editor_state_lock();
        let values = TypVal::list(Some(list(vec![sample(), string_tv(b"x")])));
        let blob = call(f_msgpackdump, vec![values.clone(), string_tv(b"B")]);
        let bytes = bytes_of(&blob);
        assert_eq!(bytes[0], 0x93, "a three-item array first");

        let back = call(f_msgpackparse, vec![blob]);
        let text = crate::eval::encode::tv2string_bytes(&back);
        assert_eq!(
            String::from_utf8(text).expect("ASCII"),
            // The dictionary's own slot order, which is the table's.
            "[[{'s': 'a\nb', 'l': [0.5, 70000], 'n': -300}, [], 5000000000], 'x']"
        );

        // The same objects as lines, split at every newline the bytes hold.
        let lines = call(f_msgpackdump, vec![values]);
        let back = call(f_msgpackparse, vec![lines]);
        assert_eq!(
            crate::eval::encode::tv2string_bytes(&back),
            crate::eval::encode::tv2string_bytes(&call(
                f_msgpackparse,
                vec![call(f_msgpackdump, vec![back.clone(), string_tv(b"B")])]
            ))
        );
    }

    /// Every prefix of a dumped value: incomplete, then whole.
    #[test]
    fn a_truncated_object_is_incomplete_until_its_last_byte() {
        let _serial = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.add_value(b"k", string_tv(b"value")).expect("fresh key");
        d.add_number(b"n", 1 << 40).expect("fresh key");
        let values = TypVal::list(Some(list(vec![TypVal::dict(Some(d))])));
        let bytes = bytes_of(&call(f_msgpackdump, vec![values, string_tv(b"B")]));
        for cut in 0..bytes.len() {
            let mut data = &bytes[..cut];
            let mut tv = TypVal::Number(0);
            let status = unpack_typval(&mut data, &mut tv);
            if cut == 0 {
                continue;
            }
            assert_eq!(status, MPACK_EOF as c_int, "cut at {cut}");
            assert_eq!(tv.v_type(), VAR_UNKNOWN, "cut at {cut}");
        }
        let mut data = &bytes[..];
        let mut tv = TypVal::Number(0);
        assert_eq!(unpack_typval(&mut data, &mut tv), MPACK_OK as c_int);
        assert!(data.is_empty());
        assert_eq!(
            crate::eval::encode::tv2string_bytes(&tv),
            b"{'k': 'value', 'n': 1099511627776}"
        );
    }

    #[test]
    fn json_round_trips_its_containers_and_scalars() {
        let _serial = editor_state_lock();
        let doc = string_tv(b"{\"a\": [1, -25, \"x\\u00e9\", null, true, {}], \"b\": {\"c\": []}}");
        let value = call(f_json_decode, vec![doc]);
        let text = call(f_json_encode, vec![value]);
        assert_eq!(
            text.string_bytes(),
            "{\"a\": [1, -25, \"x\u{e9}\", null, true, {}], \"b\": {\"c\": []}}".as_bytes()
        );
        // A List of lines is joined with newlines first.
        let lines = TypVal::list(Some(list(vec![string_tv(b"[1,"), string_tv(b"2]")])));
        let value = call(f_json_decode, vec![lines]);
        assert_eq!(crate::eval::encode::tv2string_bytes(&value), b"[1, 2]");
    }
}
