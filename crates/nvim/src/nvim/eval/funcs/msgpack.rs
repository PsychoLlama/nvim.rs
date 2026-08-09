//! Serialisation: the `msgpack*()` and `json_*()` families.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{ARENA_BLOCK_SIZE, FAIL, MPACK_EOF, MPACK_ERROR, MPACK_OK, OK};
use crate::semsg;
use crate::semsg_c;
use crate::src::mpack::object::mpack_parser_init;
use crate::src::nvim::api::private::helpers::api_free_string;
use crate::src::nvim::eval::decode::{
    json_decode_string, mpack_parse_typval, typval_parser_error_free, unpack_typval,
};
use crate::src::nvim::eval::encode::{
    encode_init_lrstate, encode_list_write, encode_read_from_list, encode_tv2json,
    encode_vim_list_to_buf, encode_vim_to_msgpack,
};
use crate::src::nvim::eval::typval::{
    tv_blob_alloc_ret, tv_blob_len, tv_get_string, tv_get_string_buf_chk, tv_list_alloc_ret,
    tv_list_append_owned_tv, tv_list_first, tv_list_len,
};
use crate::src::nvim::main::{e_listarg, e_listblobarg};
use crate::src::nvim::memory::{alloc_block, free_block, strequal, xfree};
use crate::src::nvim::msgpack_rpc::packer::{packer_string_buffer, packer_take_string};
use crate::src::nvim::os::libc::{gettext, memmove, strlen};
use crate::src::nvim::types::{
    EvalFuncData, VAR_BLOB, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, blob_T,
    kListLenMayKnow, list_T, mpack_parser_t, typval_T, typval_vval_union,
};
use core::ffi::{c_char, c_int, c_void};
use core::fmt::Write as _;
use core::ptr;

/// A cleared typval, the shape the decoders write their result into.
const EMPTY_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// `json_decode({expr})` — parse JSON from a String, or from a List of
/// lines joined by NLs.
pub unsafe extern "C" fn f_json_decode(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `tofree` owns whatever the List conversion allocated and is
    // released on every path; `s` points into it or into `numbuf`, both of
    // which outlive the parse.
    unsafe {
        let mut numbuf: [c_char; 65] = [0; 65];
        let mut tofree: *mut c_char = ptr::null_mut();
        let mut len: usize = 0;
        let s: *const c_char = if args.ty(0) == VAR_LIST {
            if !encode_vim_list_to_buf(args.get(0).vval.v_list, &raw mut len, &raw mut tofree) {
                semsg!("E474: Failed to convert list to string");
                return;
            }
            if tofree.is_null() {
                debug_assert!(len == 0);
                c"".as_ptr()
            } else {
                tofree
            }
        } else {
            let s = tv_get_string_buf_chk(args.ptr(0), numbuf.as_mut_ptr());
            if s.is_null() {
                return;
            }
            len = strlen(s);
            s
        };
        if json_decode_string(s, len, rettv) == FAIL {
            // The text that failed to parse is arbitrary user bytes, so the
            // message keeps the variadic call and its `%.*s`.
            semsg_c!(
                gettext(c"E474: Failed to parse %.*s".as_ptr()),
                len as c_int,
                s,
            );
            rettv.v_type = VAR_NUMBER;
            rettv.vval.v_number = 0;
        }
        debug_assert!(rettv.v_type != VAR_UNKNOWN);
        xfree(tofree as *mut c_void);
    }
}

/// `json_encode({expr})`.
pub unsafe extern "C" fn f_json_encode(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: the encoder reads the argument and returns an owned string,
    // which the return value takes over.
    rettv.vval.v_string = unsafe { encode_tv2json(args.ptr(0), ptr::null_mut::<usize>()) };
}

/// `msgpackdump({list} [, {type}])` — a List of msgpack objects as a List
/// of NL-joined lines, or as a Blob when `{type}` is "B".
pub unsafe extern "C" fn f_msgpackdump(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the packer owns its buffer until `packer_take_string` hands
    // it over, and the string is then owned by the Blob or written into the
    // result List and freed.
    unsafe {
        if args.ty(0) != VAR_LIST {
            semsg_c!(
                gettext(e_listarg.ptr() as *const c_char),
                c"msgpackdump()".as_ptr(),
            );
            return;
        }
        let mut packer = packer_string_buffer();
        // The per-item label the encoder names in its own error messages.
        // One buffer, reused, as the C's 189-byte stack array was.
        let mut label = String::with_capacity(64);
        let mut li = tv_list_first(args.get(0).vval.v_list);
        let mut idx: c_int = 0;
        while !li.is_null() {
            label.clear();
            let _ = write!(label, "msgpackdump() argument, index {idx}\0");
            idx += 1;
            if encode_vim_to_msgpack(
                &raw mut packer,
                &raw mut (*li).li_tv,
                label.as_ptr() as *const c_char,
            ) == 0
            {
                break;
            }
            li = (*li).li_next;
        }
        let data = packer_take_string(&mut packer);
        if args.has(1) && strequal(tv_get_string(args.ptr(1)), c"B".as_ptr()) {
            // The Blob adopts the packer's allocation as-is, capacity and
            // all; nothing copies.
            let b: *mut blob_T = tv_blob_alloc_ret(rettv);
            (*b).bv_ga.ga_data = data.data as *mut c_void;
            (*b).bv_ga.ga_len = data.size as c_int;
            (*b).bv_ga.ga_maxlen = packer.endptr.offset_from(packer.startptr) as c_int;
        } else {
            let l = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
            encode_list_write(l as *mut c_void, data.data, data.size);
            api_free_string(data);
        }
    }
}

/// Report an unpacker status that is not `MPACK_OK`.
fn emsg_mpack_error(status: c_int) {
    match status as u32 {
        MPACK_ERROR => semsg!("E475: Invalid argument: Failed to parse msgpack string"),
        MPACK_EOF => semsg!("E475: Invalid argument: Incomplete msgpack string"),
        // Anything past MPACK_ERROR is the parser's depth limit.
        3 => semsg!("E475: Invalid argument: object was too deep to unpack"),
        _ => return,
    };
}

/// Feed a List of NL-joined strings through the streaming unpacker,
/// appending each complete object to `ret_list`.
///
/// # Safety
/// `list` and `ret_list` are live lists.
unsafe fn msgpackparse_unpack_list(list: *const list_T, ret_list: *mut list_T) {
    // SAFETY: the caller's obligation. `buf` is an arena block owned for the
    // whole walk and freed at the end; `parser` is initialised before use
    // and its error state released before the last message.
    unsafe {
        if tv_list_len(list) == 0 {
            return;
        }
        if (*tv_list_first(list)).li_tv.v_type != VAR_STRING {
            semsg!("E475: Invalid argument: List item is not a string");
            return;
        }
        let mut lrstate = encode_init_lrstate(list);
        let buf = alloc_block() as *mut c_char;
        let mut buf_size: usize = 0;
        let mut cur_item = EMPTY_TV;
        let mut parser: mpack_parser_t = core::mem::zeroed();
        mpack_parser_init(&raw mut parser, 0);
        parser.data.p = &raw mut cur_item as *mut c_void;

        let mut status = MPACK_OK as c_int;
        loop {
            let mut read_bytes: usize = 0;
            let rlret = encode_read_from_list(
                &raw mut lrstate,
                buf.add(buf_size),
                (ARENA_BLOCK_SIZE as usize) - buf_size,
                &raw mut read_bytes,
            );
            if rlret == FAIL {
                semsg!("E475: Invalid argument: List item is not a string");
                break;
            }
            buf_size += read_bytes;
            let mut cursor: *const c_char = buf;
            while buf_size != 0 {
                status = mpack_parse_typval(&raw mut parser, &raw mut cursor, &raw mut buf_size);
                if status != MPACK_OK as c_int {
                    break;
                }
                tv_list_append_owned_tv(ret_list, cur_item);
                cur_item.v_type = VAR_UNKNOWN;
            }
            if rlret == OK {
                break;
            }
            if status == MPACK_EOF as c_int {
                // Shuffle the partial object back to the front so the next
                // read tops it up.
                if buf_size != 0 && cursor > buf as *const c_char {
                    memmove(buf as *mut c_void, cursor as *const c_void, buf_size);
                }
            } else if status != MPACK_OK as c_int {
                break;
            }
        }
        if status != MPACK_OK as c_int {
            typval_parser_error_free(&raw mut parser);
            emsg_mpack_error(status);
        }
        free_block(buf as *mut c_void);
    }
}

/// Unpack a Blob, which is already one contiguous buffer.
///
/// # Safety
/// `blob` is a live blob and `ret_list` a live list.
unsafe fn msgpackparse_unpack_blob(blob: *const blob_T, ret_list: *mut list_T) {
    // SAFETY: the caller's obligation; `unpack_typval` advances the cursor
    // and the remaining count together.
    unsafe {
        let len = tv_blob_len(blob);
        if len == 0 {
            return;
        }
        let mut data = (*blob).bv_ga.ga_data as *const c_char;
        let mut remaining = len as usize;
        while remaining != 0 {
            let mut tv = EMPTY_TV;
            let status = unpack_typval(&raw mut data, &raw mut remaining, &raw mut tv);
            if status != MPACK_OK as c_int {
                emsg_mpack_error(status);
                return;
            }
            tv_list_append_owned_tv(ret_list, tv);
        }
    }
}

/// `msgpackparse({data})` — the objects in a List of strings or a Blob.
pub unsafe extern "C" fn f_msgpackparse(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the argument and the freshly allocated result list are both
    // live for the call.
    unsafe {
        if args.ty(0) != VAR_LIST && args.ty(0) != VAR_BLOB {
            semsg_c!(
                gettext(e_listblobarg.ptr() as *const c_char),
                c"msgpackparse()".as_ptr(),
            );
            return;
        }
        let ret_list = tv_list_alloc_ret(rettv, kListLenMayKnow as isize);
        if args.ty(0) == VAR_LIST {
            msgpackparse_unpack_list(args.get(0).vval.v_list, ret_list);
        } else {
            msgpackparse_unpack_blob(args.get(0).vval.v_blob, ret_list);
        }
    }
}
