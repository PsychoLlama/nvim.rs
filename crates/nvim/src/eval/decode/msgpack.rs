//! msgpack bytes into a `typval_T`: the two `mpack_parse` callbacks.
//!
//! `mpack_parse()` walks the byte stream with an explicit node stack and
//! calls [`typval_parse_enter`] as each node opens and [`typval_parse_exit`]
//! as it closes.  Most values are finished on the way in; the three that are
//! not are `str`/`bin` and `ext` (their bytes arrive afterwards, as `chunk`
//! nodes) and `map` (whether it can be a `dict_T` is only knowable once every
//! key has been decoded).  Those three park a buffer in `node.data[1]`, which
//! is the one thing [`typval_parser_error_free`] has to clean up when a parse
//! fails part-way.
//!
//! `node.data[0]` is where the value goes — a slot in the parent's list, in
//! the parent map's scratch array, or the caller's `rettv` at the root.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::MaybeUninit;
use core::ptr;

use super::{
    create_special_dict, decode_create_map_special_dict, decode_string, kMPExt, kMPInteger,
};
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{
    TV_INITIAL_VALUE, tv_clear, tv_dict_add, tv_dict_alloc, tv_dict_hi2di, tv_dict_item_alloc_len,
    tv_dict_iter, tv_list_alloc, tv_list_append_list, tv_list_append_number,
    tv_list_append_owned_tv, tv_list_ref,
};
use crate::memory::{xfree, xmallocz};
use crate::mpack::conv::{
    mpack_unpack_boolean, mpack_unpack_float_fast, mpack_unpack_sint, mpack_unpack_uint,
};
use crate::mpack::mpack_core::{
    MPACK_TOKEN_ARRAY, MPACK_TOKEN_BIN, MPACK_TOKEN_BOOLEAN, MPACK_TOKEN_CHUNK, MPACK_TOKEN_EXT,
    MPACK_TOKEN_FLOAT, MPACK_TOKEN_MAP, MPACK_TOKEN_NIL, MPACK_TOKEN_SINT, MPACK_TOKEN_STR,
    MPACK_TOKEN_UINT,
};
use crate::mpack::object::{mpack_parse, mpack_parser_init};
use crate::types::{
    FAIL, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VAR_SPECIAL, VAR_STRING,
    VAR_UNKNOWN, VAR_UNLOCKED, kBoolVarFalse, kBoolVarTrue, kListLenMayKnow, kSpecialVarNull,
    list_T, mpack_node_t, mpack_parser_t, ptrdiff_t, size_t, typval_T, typval_vval_union,
    varnumber_T,
};
use ::libc::{abort, memcpy, strlen};

const MPACK_OK: c_int = 0;

/// The largest `varnumber_T`, past which a msgpack unsigned integer needs a
/// special dictionary to survive the trip into Vimscript.
const VARNUMBER_MAX: u64 = i64::MAX as u64;

/// A msgpack unsigned integer as a `typval_T`.
///
/// Anything a `varnumber_T` can hold is a plain number.  What it cannot is
/// split across a four-element `{_TYPE: integer, _VAL: [sign, hi, mid, lo]}`
/// list — one sign, then 2 + 31 + 31 bits — which is the same shape the
/// msgpack encoder reads back.
///
/// # Safety
/// `rettv` is writable and holds no value that needs clearing.
unsafe fn positive_integer_to_special_typval(rettv: *mut typval_T, val: u64) {
    unsafe {
        if val <= VARNUMBER_MAX {
            *rettv = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_number: val as varnumber_T,
                },
            };
            return;
        }
        let list = tv_list_alloc(4);
        tv_list_ref(list);
        create_special_dict(
            rettv,
            kMPInteger,
            typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: list },
            },
        );
        tv_list_append_number(list, 1);
        tv_list_append_number(list, ((val >> 62) & 0x3) as varnumber_T);
        tv_list_append_number(list, ((val >> 31) & 0x7fff_ffff) as varnumber_T);
        tv_list_append_number(list, (val & 0x7fff_ffff) as varnumber_T);
    }
}

/// A node has opened: work out where its value belongs, and decode it if the
/// token already carries the whole value.
unsafe extern "C-unwind" fn typval_parse_enter(
    parser: *mut mpack_parser_t,
    node: *mut mpack_node_t,
) {
    unsafe {
        // `MPACK_PARENT_NODE`: the node one level up, or none at the root.
        // `mpack_parser_init` writes `(size_t)-1` into `items[0].pos`, so the
        // sentinel below the first real node is what says "no parent".
        let below = node.sub(1);
        let parent = if (*below).pos == !0 {
            ptr::null_mut()
        } else {
            below
        };

        let result: *mut typval_T = if parent.is_null() {
            (*parser).data.p.cast()
        } else {
            match (*parent).tok.type_0 {
                // An array element is appended empty and filled in place.
                MPACK_TOKEN_ARRAY => {
                    let list: *mut list_T = (*parent).data[1].p.cast();
                    tv_list_append_owned_tv(list, TV_INITIAL_VALUE)
                }
                // A map's pairs go to the scratch array the exit hook reads;
                // `key_visited` picks the key or the value of the pair.
                MPACK_TOKEN_MAP => {
                    let pairs: *mut typval_T = (*parent).data[1].p.cast();
                    pairs
                        .add((*parent).pos * 2)
                        .add((*parent).key_visited as usize)
                }
                // The only child of a byte-carrying token is its data, which
                // is copied straight into the parent's buffer below.
                MPACK_TOKEN_STR | MPACK_TOKEN_BIN | MPACK_TOKEN_EXT => {
                    debug_assert!((*node).tok.type_0 == MPACK_TOKEN_CHUNK);
                    ptr::null_mut()
                }
                _ => abort(),
            }
        };

        (*node).data[0].p = result.cast();
        // Anything parked here is freed on error; see typval_parser_error_free.
        (*node).data[1].p = ptr::null_mut();

        let len = (*node).tok.length as size_t;
        match (*node).tok.type_0 {
            MPACK_TOKEN_NIL => {
                *result = typval_T {
                    v_type: VAR_SPECIAL,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_special: kSpecialVarNull,
                    },
                };
            }
            MPACK_TOKEN_BOOLEAN => {
                *result = typval_T {
                    v_type: VAR_BOOL,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_bool: if mpack_unpack_boolean((*node).tok) {
                            kBoolVarTrue
                        } else {
                            kBoolVarFalse
                        },
                    },
                };
            }
            MPACK_TOKEN_SINT => {
                *result = typval_T {
                    v_type: VAR_NUMBER,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_number: mpack_unpack_sint((*node).tok),
                    },
                };
            }
            MPACK_TOKEN_UINT => {
                positive_integer_to_special_typval(result, mpack_unpack_uint((*node).tok));
            }
            MPACK_TOKEN_FLOAT => {
                *result = typval_T {
                    v_type: VAR_FLOAT,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_float: mpack_unpack_float_fast((*node).tok),
                    },
                };
            }
            // Converted in typval_parse_exit, once the chunks have landed.
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
                (*node).data[1].p = xmallocz(len);
            }
            MPACK_TOKEN_CHUNK => {
                let data: *mut c_char = (*parent).data[1].p.cast();
                memcpy(
                    data.add((*parent).pos).cast(),
                    (*node).tok.data.chunk_ptr.cast::<c_void>(),
                    len,
                );
            }
            MPACK_TOKEN_ARRAY => {
                let list = tv_list_alloc(len as ptrdiff_t);
                tv_list_ref(list);
                *result = typval_T {
                    v_type: VAR_LIST,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_list: list },
                };
                (*node).data[1].p = list.cast();
            }
            // Whether this can be a dict_T is not knowable yet, so the pairs
            // are decoded into a flat `[key, value] * length` scratch array.
            MPACK_TOKEN_MAP => {
                // `length * 2` is `mpack_uint32_t` arithmetic upstream, so a
                // header claiming 2^31 pairs or more wraps and under-allocates
                // — docket O-B14-9, kept rather than fixed.  Widening the
                // multiply is not free: the honest size is 64 GB, which is a
                // fatal `E41` here where upstream answers `E475: Incomplete
                // msgpack string`.
                let pairs = (*node).tok.length.wrapping_mul(2) as size_t;
                (*node).data[1].p = xmallocz(pairs * ::core::mem::size_of::<typval_T>());
            }
            _ => {}
        }
    }
}

/// Free what a node parked in `data[1]` but never got to consume.
///
/// Called when a parse fails part-way through, for every node still on the
/// parser's stack.  The typvals themselves are left to the garbage collector.
///
/// # Safety
/// `parser` is a live parser whose `size` bounds its `items`.
pub unsafe fn typval_parser_error_free(parser: *mut mpack_parser_t) {
    unsafe {
        for i in 0..(*parser).size as usize {
            let node = &raw mut (*parser).items[i];
            match (*node).tok.type_0 {
                MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT | MPACK_TOKEN_MAP => {
                    xfree((*node).data[1].p);
                    (*node).data[1].p = ptr::null_mut();
                }
                _ => {}
            }
        }
    }
}

/// Build a `dict_T` out of `len` decoded key/value pairs.
///
/// Answers `false` when the map cannot be one — a key that is not a non-empty
/// string, or a duplicate — leaving every pair in `pairs` untouched and ready
/// for the special-map path.  The partially built dictionary is torn down
/// first, with its values disowned so that they survive it.
///
/// # Safety
/// `pairs` points at `len * 2` decoded typvals and `result` is writable.
unsafe fn map_to_dict(result: *mut typval_T, pairs: *mut typval_T, len: usize) -> bool {
    unsafe {
        for i in 0..len {
            let key = *pairs.add(i * 2);
            if key.v_type != VAR_STRING || key.vval.v_string.is_null() || *key.vval.v_string == 0 {
                return false;
            }
        }

        let dict = tv_dict_alloc();
        (*dict).dv_refcount += 1;
        *result = typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_dict: dict },
        };

        for i in 0..len {
            let key = (*pairs.add(i * 2)).vval.v_string;
            let di = tv_dict_item_alloc_len(key, strlen(key));
            if tv_dict_add(dict, di) == FAIL {
                // Duplicate key.  Disown the values already handed to the
                // dictionary — the special-map path is about to re-use every
                // one of them — then free the dictionary and give up.
                for hi in tv_dict_iter(&*dict) {
                    let d = tv_dict_hi2di(hi);
                    (*d).di_tv.v_type = VAR_SPECIAL;
                    (*d).di_tv.vval.v_special = kSpecialVarNull;
                }
                tv_clear(result);
                xfree(di.cast());
                return false;
            }
            (*di).di_tv = *pairs.add(i * 2 + 1);
        }

        // The keys were copied into the items; the originals are ours to free.
        for i in 0..len {
            xfree((*pairs.add(i * 2)).vval.v_string.cast());
        }
        true
    }
}

/// A node has closed: finish the values whose bytes only arrive now.
unsafe extern "C-unwind" fn typval_parse_exit(
    _parser: *mut mpack_parser_t,
    node: *mut mpack_node_t,
) {
    unsafe {
        let result: *mut typval_T = (*node).data[0].p.cast();
        let len = (*node).tok.length as size_t;
        match (*node).tok.type_0 {
            // The chunk buffer is handed straight to the string or blob.
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR => {
                *result = decode_string((*node).data[1].p.cast(), len, false, true);
                (*node).data[1].p = ptr::null_mut();
            }
            // `{_TYPE: ext, _VAL: [type, [bytes…]]}`.  The payload goes into a
            // list of strings rather than a blob, as upstream's TODO notes.
            MPACK_TOKEN_EXT => {
                let list = tv_list_alloc(2);
                tv_list_ref(list);
                tv_list_append_number(list, (*node).tok.data.ext_type as varnumber_T);
                let ext_val_list = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
                tv_list_append_list(list, ext_val_list);
                create_special_dict(
                    result,
                    kMPExt,
                    typval_T {
                        v_type: VAR_LIST,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_list: list },
                    },
                );
                encode_list_write(ext_val_list.cast(), (*node).data[1].p.cast(), len);
                xfree((*node).data[1].p);
                (*node).data[1].p = ptr::null_mut();
            }
            MPACK_TOKEN_MAP => {
                let pairs: *mut typval_T = (*node).data[1].p.cast();
                if !map_to_dict(result, pairs, len) {
                    let list = decode_create_map_special_dict(result, len as ptrdiff_t);
                    for i in 0..len {
                        let kv_pair = tv_list_alloc(2);
                        tv_list_append_list(list, kv_pair);
                        tv_list_append_owned_tv(kv_pair, *pairs.add(i * 2));
                        tv_list_append_owned_tv(kv_pair, *pairs.add(i * 2 + 1));
                    }
                }
                xfree((*node).data[1].p);
                (*node).data[1].p = ptr::null_mut();
            }
            // Everything else was finished in typval_parse_enter.
            _ => {}
        }
    }
}

/// One step of `mpack_parse()` with the typval callbacks bound in.
///
/// `data`/`size` are advanced past whatever was consumed; the answer is
/// `MPACK_OK` when a whole object came out, `MPACK_EOF` when the bytes ran
/// out mid-object, or an error status.
///
/// # Safety
/// `parser` was initialised with its `data.p` pointing at the destination
/// typval, and `data`/`size` describe a live buffer.
pub unsafe fn mpack_parse_typval(
    parser: *mut mpack_parser_t,
    data: *mut *const c_char,
    size: *mut size_t,
) -> c_int {
    unsafe {
        mpack_parse(
            parser,
            data,
            size,
            Some(typval_parse_enter),
            Some(typval_parse_exit),
        )
    }
}

/// Decode one complete msgpack object from `data` into `ret`.
///
/// `data` and `size` are advanced past the object.  On any status but
/// `MPACK_OK` the half-built value is released and `ret` is left cleared.
///
/// # Safety
/// `data`/`size` describe a live buffer and `ret` is writable.
pub unsafe fn unpack_typval(
    data: *mut *const c_char,
    size: *mut size_t,
    ret: *mut typval_T,
) -> c_int {
    unsafe {
        (*ret).v_type = VAR_UNKNOWN;
        // `mpack_parser_init` writes every field this parser will be read
        // through, `items` included, so the C leaves the declaration
        // uninitialised too — and it is 2.5 KB, once per decoded object.
        // Nothing here ever forms a reference to it, so it stays a raw
        // pointer rather than being `assume_init`ed.
        let mut storage = MaybeUninit::<mpack_parser_t>::uninit();
        let parser = storage.as_mut_ptr();
        mpack_parser_init(parser, 0);
        (*parser).data.p = ret.cast();
        let status = mpack_parse_typval(parser, data, size);
        if status != MPACK_OK {
            typval_parser_error_free(parser);
            tv_clear(ret);
        }
        status
    }
}
