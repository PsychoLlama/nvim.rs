use crate::src::mpack::conv::{
    mpack_unpack_boolean, mpack_unpack_float_fast, mpack_unpack_sint, mpack_unpack_uint,
};
use crate::src::mpack::mpack_core::{mpack_read, mpack_rtoken, mpack_tokbuf_init};
use crate::src::mpack::object::{mpack_parse, mpack_parser_init};
use crate::src::nvim::api::private::dispatch::msgpack_rpc_get_handler_for;
use crate::src::nvim::api::private::helpers::api_set_error;

use crate::src::nvim::grid::schar_from_buf;
use crate::src::nvim::main::{grid_line_buf_attr, grid_line_buf_char, grid_line_buf_size};
use crate::src::nvim::memory::{arena_mem_free, xrealloc, xstrdup};
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy};
use crate::src::nvim::strings::arena_printf;
pub use crate::src::nvim::types::{
    consumed_blk, int32_t, int64_t, key_value_pair, mpack_data_t, mpack_node_s, mpack_node_t,
    mpack_parser_t, mpack_sintmax_t, mpack_tokbuf_s, mpack_tokbuf_t, mpack_token_s,
    mpack_token_s_data as C2Rust_Unnamed_0, mpack_token_t, mpack_token_type_t, mpack_uint32_t,
    mpack_uintmax_t, mpack_value_s, mpack_value_t, mpack_walk_cb, object,
    object_data as C2Rust_Unnamed_1, sattr_T, schar_T, size_t, ssize_t, uint32_t, uint64_t,
    AdditionalData, AdditionalDataBuilder, ApiDispatchWrapper, Arena, ArenaMem, Array, Boolean,
    Dict, Error, ErrorType, FieldHashfn, Float, GridLineEvent, Integer, KeySetLink, KeyValuePair,
    LuaRef, MessageType, MsgpackRpcRequestHandler, Object, ObjectType, OptKeySet, OptionalKeys,
    StringArray, String_0, UIClientHandler,
};
use crate::src::nvim::ui_client::{
    handle_ui_client_redraw, ui_client_event_grid_line, ui_client_get_redraw_handler,
};
extern "C" {
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MPACK_ERROR: C2Rust_Unnamed = 2;
pub const MPACK_EOF: C2Rust_Unnamed = 1;
pub const MPACK_OK: C2Rust_Unnamed = 0;
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kMessageTypeRedrawEvent: MessageType = 3;
pub const kMessageTypeNotification: MessageType = 2;
pub const kMessageTypeResponse: MessageType = 1;
pub const kMessageTypeRequest: MessageType = 0;
pub const kMessageTypeUnknown: MessageType = -1;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_int;
pub const kUnpackTypeStringArray: C2Rust_Unnamed_2 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Unpacker {
    pub parser: mpack_parser_t,
    pub reader: mpack_tokbuf_t,
    pub read_ptr: *const ::core::ffi::c_char,
    pub read_size: size_t,
    pub ext_buf: [::core::ffi::c_char; 9],
    pub state: ::core::ffi::c_int,
    pub type_0: MessageType,
    pub request_id: uint32_t,
    pub method_name_len: size_t,
    pub handler: MsgpackRpcRequestHandler,
    pub error: Object,
    pub result: Object,
    pub unpack_error: Error,
    pub arena: Arena,
    pub nevents: ::core::ffi::c_int,
    pub ncalls: ::core::ffi::c_int,
    pub ui_handler: UIClientHandler,
    pub grid_line_event: GridLineEvent,
    pub has_grid_line_event: bool,
}
pub type C2Rust_Unnamed_3 = ::core::ffi::c_int;
pub const MPACK_NOMEM: C2Rust_Unnamed_3 = 3;
pub const MPACK_EXCEPTION: C2Rust_Unnamed_3 = -1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const MAX_EXT_LEN: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn unpack(
    mut data: *const ::core::ffi::c_char,
    mut size: size_t,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut unpacker: Unpacker = Unpacker {
        parser: mpack_parser_t {
            data: mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            size: 0,
            capacity: 0,
            status: 0,
            exiting: 0,
            tokbuf: mpack_tokbuf_t {
                pending: [0; 9],
                pending_tok: mpack_token_t {
                    type_0: 0 as mpack_token_type_t,
                    length: 0,
                    data: C2Rust_Unnamed_0 {
                        value: mpack_value_t { lo: 0, hi: 0 },
                    },
                },
                ppos: 0,
                plen: 0,
                passthrough: 0,
            },
            items: [mpack_node_t {
                tok: mpack_token_t {
                    type_0: 0 as mpack_token_type_t,
                    length: 0,
                    data: C2Rust_Unnamed_0 {
                        value: mpack_value_t { lo: 0, hi: 0 },
                    },
                },
                pos: 0,
                key_visited: 0,
                data: [mpack_data_t {
                    p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                }; 2],
            }; 33],
        },
        reader: mpack_tokbuf_t {
            pending: [0; 9],
            pending_tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed_0 {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            ppos: 0,
            plen: 0,
            passthrough: 0,
        },
        read_ptr: ::core::ptr::null::<::core::ffi::c_char>(),
        read_size: 0,
        ext_buf: [0; 9],
        state: 0,
        type_0: kMessageTypeRequest,
        request_id: 0,
        method_name_len: 0,
        handler: MsgpackRpcRequestHandler {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            fn_0: None,
            fast: false,
            ret_alloc: false,
        },
        error: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_1 { boolean: false },
        },
        result: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_1 { boolean: false },
        },
        unpack_error: Error {
            type_0: kErrorTypeException,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        arena: Arena {
            cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            pos: 0,
            size: 0,
        },
        nevents: 0,
        ncalls: 0,
        ui_handler: UIClientHandler {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            fn_0: None,
        },
        grid_line_event: GridLineEvent {
            args: [0; 3],
            icell: 0,
            ncells: 0,
            coloff: 0,
            cur_attr: 0,
            clear_width: 0,
            wrap: false,
        },
        has_grid_line_event: false,
    };
    mpack_parser_init(&raw mut unpacker.parser, 0 as mpack_uint32_t);
    unpacker.parser.data.p = &raw mut unpacker as *mut ::core::ffi::c_void;
    unpacker.arena = *arena;
    let mut result: ::core::ffi::c_int = mpack_parse(
        &raw mut unpacker.parser,
        &raw mut data,
        &raw mut size,
        Some(api_parse_enter as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> ()),
        Some(api_parse_exit as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> ()),
    );
    *arena = unpacker.arena;
    if result == MPACK_NOMEM as ::core::ffi::c_int {
        api_set_error(
            err,
            kErrorTypeException,
            b"object was too deep to unpack\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if result == MPACK_EOF as ::core::ffi::c_int {
        api_set_error(
            err,
            kErrorTypeException,
            b"incomplete msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if result == MPACK_ERROR as ::core::ffi::c_int {
        api_set_error(
            err,
            kErrorTypeException,
            b"invalid msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if result == MPACK_OK as ::core::ffi::c_int && size != 0 {
        api_set_error(
            err,
            kErrorTypeException,
            b"trailing data in msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return unpacker.result;
}
unsafe extern "C" fn api_parse_enter(mut parser: *mut mpack_parser_t, mut node: *mut mpack_node_t) {
    let mut p: *mut Unpacker = (*parser).data.p as *mut Unpacker;
    let mut result: *mut Object = ::core::ptr::null_mut::<Object>();
    let mut key_location: *mut String_0 = ::core::ptr::null_mut::<String_0>();
    let mut parent: *mut mpack_node_t = if (*node.offset(-(1 as ::core::ffi::c_int as isize))).pos
        == -1 as ::core::ffi::c_int as size_t
    {
        ::core::ptr::null_mut::<mpack_node_t>()
    } else {
        node.offset(-(1 as ::core::ffi::c_int as isize))
    };
    if !parent.is_null() {
        match (*parent).tok.type_0 as ::core::ffi::c_uint {
            7 => {
                let mut obj: *mut Object =
                    (*parent).data[0 as ::core::ffi::c_int as usize].p as *mut Object;
                result = (*obj).data.array.items.offset((*parent).pos as isize);
            }
            8 => {
                let mut obj_0: *mut Object =
                    (*parent).data[0 as ::core::ffi::c_int as usize].p as *mut Object;
                let mut kv: *mut KeyValuePair =
                    (*obj_0).data.dict.items.offset((*parent).pos as isize);
                if (*parent).key_visited == 0 {
                    (*kv).key = STRING_INIT;
                    key_location = &raw mut (*kv).key;
                }
                result = &raw mut (*kv).value;
            }
            10 | 9 | 11 => {
                '_c2rust_label: {
                    if (*node).tok.type_0 as ::core::ffi::c_uint
                        == MPACK_TOKEN_CHUNK as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                    } else {
                        __assert_fail(
                            b"node->tok.type == MPACK_TOKEN_CHUNK\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/msgpack_rpc/unpacker.rs\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            75 as ::core::ffi::c_uint,
                            b"void api_parse_enter(mpack_parser_t *, mpack_node_t *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
            }
            _ => {
                abort();
            }
        }
    } else {
        result = &raw mut (*p).result;
    }
    match (*node).tok.type_0 as ::core::ffi::c_uint {
        1 => {
            *result = object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_1 { boolean: false },
            };
        }
        2 => {
            *result = object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_1 {
                    boolean: mpack_unpack_boolean((*node).tok),
                },
            };
        }
        4 => {
            *result = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_1 {
                    integer: mpack_unpack_sint((*node).tok) as Integer,
                },
            };
        }
        3 => {
            *result = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_1 {
                    integer: mpack_unpack_uint((*node).tok) as Integer,
                },
            };
        }
        5 => {
            *result = object {
                type_0: kObjectTypeFloat,
                data: C2Rust_Unnamed_1 {
                    floating: mpack_unpack_float_fast((*node).tok),
                },
            };
        }
        9 | 10 => {
            let mut mem: *mut ::core::ffi::c_char = arena_alloc(
                &raw mut (*p).arena,
                (*node).tok.length.wrapping_add(1 as mpack_uint32_t) as size_t,
                false_0 != 0,
            ) as *mut ::core::ffi::c_char;
            *mem.offset((*node).tok.length as isize) = NUL as ::core::ffi::c_char;
            let mut str: String_0 = String_0 {
                data: mem,
                size: (*node).tok.length as size_t,
            };
            if !key_location.is_null() {
                *key_location = str;
            } else {
                *result = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_1 { string: str },
                };
            }
            (*node).data[0 as ::core::ffi::c_int as usize].p = str.data as *mut ::core::ffi::c_void;
        }
        11 => {
            (*node).data[0 as ::core::ffi::c_int as usize].p = result as *mut ::core::ffi::c_void;
        }
        6 => {
            '_c2rust_label_0: {
                if !parent.is_null() {
                } else {
                    __assert_fail(
                        b"parent\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/msgpack_rpc/unpacker.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        120 as ::core::ffi::c_uint,
                        b"void api_parse_enter(mpack_parser_t *, mpack_node_t *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if (*parent).tok.type_0 as ::core::ffi::c_uint
                == MPACK_TOKEN_STR as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*parent).tok.type_0 as ::core::ffi::c_uint
                    == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut data: *mut ::core::ffi::c_char =
                    (*parent).data[0 as ::core::ffi::c_int as usize].p as *mut ::core::ffi::c_char;
                memcpy(
                    data.offset((*parent).pos as isize) as *mut ::core::ffi::c_void,
                    (*node).tok.data.chunk_ptr as *const ::core::ffi::c_void,
                    (*node).tok.length as size_t,
                );
            } else {
                let mut res: *mut Object =
                    (*parent).data[0 as ::core::ffi::c_int as usize].p as *mut Object;
                let mut endlen: size_t = (*parent).pos.wrapping_add((*node).tok.length as size_t);
                if endlen > MAX_EXT_LEN as size_t {
                    *res = object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed_1 { boolean: false },
                    };
                } else {
                    memcpy(
                        (&raw mut (*p).ext_buf as *mut ::core::ffi::c_char)
                            .offset((*parent).pos as isize)
                            as *mut ::core::ffi::c_void,
                        (*node).tok.data.chunk_ptr as *const ::core::ffi::c_void,
                        (*node).tok.length as size_t,
                    );
                    if (*parent).pos.wrapping_add((*node).tok.length as size_t)
                        >= (*parent).tok.length as size_t
                    {
                        let mut buf: *const ::core::ffi::c_char =
                            &raw mut (*p).ext_buf as *mut ::core::ffi::c_char;
                        let mut size: size_t = (*parent).tok.length as size_t;
                        let mut ext_tok: mpack_token_t = mpack_token_t {
                            type_0: 0 as mpack_token_type_t,
                            length: 0,
                            data: C2Rust_Unnamed_0 {
                                value: mpack_value_t { lo: 0, hi: 0 },
                            },
                        };
                        let mut status: ::core::ffi::c_int =
                            mpack_rtoken(&raw mut buf, &raw mut size, &raw mut ext_tok);
                        if status != 0
                            || ext_tok.type_0 as ::core::ffi::c_uint
                                != MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            *res = object {
                                type_0: kObjectTypeNil,
                                data: C2Rust_Unnamed_1 { boolean: false },
                            };
                        } else {
                            let mut ext_type: ::core::ffi::c_int = (*parent).tok.data.ext_type;
                            if 0 as ::core::ffi::c_int <= ext_type
                                && ext_type
                                    <= kObjectTypeTabpage as ::core::ffi::c_int
                                        - kObjectTypeBuffer as ::core::ffi::c_int
                            {
                                (*res).type_0 = (ext_type + kObjectTypeBuffer as ::core::ffi::c_int)
                                    as ObjectType;
                                (*res).data.integer =
                                    mpack_unpack_uint(ext_tok) as int64_t as Integer;
                            } else {
                                *res = object {
                                    type_0: kObjectTypeNil,
                                    data: C2Rust_Unnamed_1 { boolean: false },
                                };
                            }
                        }
                    }
                }
            }
        }
        7 => {
            let mut arr: Array = Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
            arr.capacity = (*node).tok.length as size_t;
            arr.items = arena_alloc(
                &raw mut (*p).arena,
                ::core::mem::size_of::<Object>().wrapping_mul(arr.capacity),
                true_0 != 0,
            ) as *mut Object;
            arr.size = (*node).tok.length as size_t;
            *result = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_1 { array: arr },
            };
            (*node).data[0 as ::core::ffi::c_int as usize].p = result as *mut ::core::ffi::c_void;
        }
        8 => {
            let mut dict: Dict = Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
            dict.capacity = (*node).tok.length as size_t;
            dict.items = arena_alloc(
                &raw mut (*p).arena,
                ::core::mem::size_of::<KeyValuePair>().wrapping_mul(dict.capacity),
                true_0 != 0,
            ) as *mut KeyValuePair;
            dict.size = (*node).tok.length as size_t;
            *result = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed_1 { dict: dict },
            };
            (*node).data[0 as ::core::ffi::c_int as usize].p = result as *mut ::core::ffi::c_void;
        }
        _ => {}
    };
}
unsafe extern "C" fn api_parse_exit(
    mut _parser: *mut mpack_parser_t,
    mut _node: *mut mpack_node_t,
) {
}
#[no_mangle]
pub unsafe extern "C" fn unpacker_init(mut p: *mut Unpacker) {
    mpack_parser_init(&raw mut (*p).parser, 0 as mpack_uint32_t);
    (*p).parser.data.p = p as *mut ::core::ffi::c_void;
    mpack_tokbuf_init(&raw mut (*p).reader);
    (*p).unpack_error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    (*p).arena = ARENA_EMPTY;
    (*p).has_grid_line_event = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn unpacker_teardown(mut p: *mut Unpacker) {
    arena_mem_free(arena_finish(&raw mut (*p).arena));
}
pub unsafe extern "C" fn unpacker_parse_header(mut p: *mut Unpacker) -> bool {
    let mut array_length: size_t = 0;
    let mut type_0: uint32_t = 0;
    let mut tok: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed_0 {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut result: ::core::ffi::c_int = 0;
    let mut data: *const ::core::ffi::c_char = (*p).read_ptr;
    let mut size: size_t = (*p).read_size;
    '_c2rust_label: {
        if !((*p).unpack_error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
        {
        } else {
            __assert_fail(
                b"!ERROR_SET(&p->unpack_error)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/msgpack_rpc/unpacker.rs\0".as_ptr() as *const ::core::ffi::c_char,
                207 as ::core::ffi::c_uint,
                b"_Bool unpacker_parse_header(Unpacker *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    result = mpack_read(
        &raw mut (*p).reader,
        &raw mut data,
        &raw mut size,
        &raw mut tok,
    );
    '_error: {
        if result == 0 {
            if !(tok.type_0 as ::core::ffi::c_uint
                != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                || tok.length < 3 as mpack_uint32_t
                || tok.length > 4 as mpack_uint32_t)
            {
                array_length = tok.length as size_t;
                result = mpack_read(
                    &raw mut (*p).reader,
                    &raw mut data,
                    &raw mut size,
                    &raw mut tok,
                );
                if result == 0 {
                    if tok.type_0 as ::core::ffi::c_uint
                        == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        type_0 = mpack_unpack_uint(tok) as uint32_t;
                        if if array_length == 3 as size_t {
                            (type_0 != 2 as uint32_t) as ::core::ffi::c_int
                        } else {
                            (type_0 >= 2 as uint32_t) as ::core::ffi::c_int
                        } == 0
                        {
                            (*p).type_0 = type_0 as MessageType;
                            (*p).request_id = 0 as uint32_t;
                            if (*p).type_0 as ::core::ffi::c_int
                                != kMessageTypeNotification as ::core::ffi::c_int
                            {
                                result = mpack_read(
                                    &raw mut (*p).reader,
                                    &raw mut data,
                                    &raw mut size,
                                    &raw mut tok,
                                );
                                if result != 0 {
                                    break '_error;
                                } else if tok.type_0 as ::core::ffi::c_uint
                                    != MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    break '_error;
                                } else {
                                    (*p).request_id = mpack_unpack_uint(tok) as uint32_t;
                                }
                            }
                            if (*p).type_0 as ::core::ffi::c_int
                                != kMessageTypeResponse as ::core::ffi::c_int
                            {
                                result = mpack_read(
                                    &raw mut (*p).reader,
                                    &raw mut data,
                                    &raw mut size,
                                    &raw mut tok,
                                );
                                if result != 0 {
                                    break '_error;
                                } else if tok.type_0 as ::core::ffi::c_uint
                                    != MPACK_TOKEN_STR as ::core::ffi::c_int as ::core::ffi::c_uint
                                    && tok.type_0 as ::core::ffi::c_uint
                                        != MPACK_TOKEN_BIN as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    || tok.length > 100 as mpack_uint32_t
                                {
                                    break '_error;
                                } else {
                                    (*p).method_name_len = tok.length as size_t;
                                    if (*p).method_name_len > 0 as size_t {
                                        result = mpack_read(
                                            &raw mut (*p).reader,
                                            &raw mut data,
                                            &raw mut size,
                                            &raw mut tok,
                                        );
                                        if result != 0 {
                                            break '_error;
                                        } else {
                                            '_c2rust_label_0: {
                                                if tok.type_0 as ::core::ffi::c_uint
                                                    == MPACK_TOKEN_CHUNK as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"tok.type == MPACK_TOKEN_CHUNK\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        b"src/nvim/msgpack_rpc/unpacker.rs\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                        249 as ::core::ffi::c_uint,
                                                        b"_Bool unpacker_parse_header(Unpacker *)\0"
                                                            .as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                        }
                                    }
                                    if (tok.length as size_t) < (*p).method_name_len {
                                        result = MPACK_EOF as ::core::ffi::c_int;
                                        break '_error;
                                    } else {
                                        (*p).handler = msgpack_rpc_get_handler_for(
                                            if tok.length != 0 {
                                                tok.data.chunk_ptr
                                            } else {
                                                b"\0".as_ptr() as *const ::core::ffi::c_char
                                            },
                                            tok.length as size_t,
                                            &raw mut (*p).unpack_error,
                                        );
                                    }
                                }
                            }
                            (*p).read_ptr = data;
                            (*p).read_size = size;
                            return true_0 != 0;
                        }
                    }
                }
            }
        }
    }
    if result == MPACK_EOF as ::core::ffi::c_int {
        mpack_tokbuf_init(&raw mut (*p).reader);
    } else {
        api_set_error(
            &raw mut (*p).unpack_error,
            kErrorTypeValidation,
            b"failed to decode msgpack\0".as_ptr() as *const ::core::ffi::c_char,
        );
        (*p).state = -1 as ::core::ffi::c_int;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn unpacker_advance(mut p: *mut Unpacker) -> bool {
    let mut result: ::core::ffi::c_int = 0;
    let mut c2rust_current_block: u64;
    '_c2rust_label: {
        if (*p).state >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"p->state >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/msgpack_rpc/unpacker.rs\0".as_ptr() as *const ::core::ffi::c_char,
                308 as ::core::ffi::c_uint,
                b"_Bool unpacker_advance(Unpacker *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    (*p).has_grid_line_event = false_0 != 0;
    if (*p).state == 0 as ::core::ffi::c_int {
        if !unpacker_parse_header(p) {
            return false_0 != 0;
        }
        if (*p).type_0 as ::core::ffi::c_int == kMessageTypeNotification as ::core::ffi::c_int
            && (*p).handler.fn_0
                == Some(
                    handle_ui_client_redraw
                        as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
                )
        {
            (*p).type_0 = kMessageTypeRedrawEvent;
            (*p).state = 10 as ::core::ffi::c_int;
        } else {
            (*p).state = if (*p).type_0 as ::core::ffi::c_int
                == kMessageTypeResponse as ::core::ffi::c_int
            {
                1 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            };
            (*p).arena = ARENA_EMPTY;
        }
    }
    '_done: {
        if (*p).state >= 10 as ::core::ffi::c_int && (*p).state != 13 as ::core::ffi::c_int {
            if !unpacker_parse_redraw(p) {
                return false_0 != 0;
            }
            if (*p).state == 16 as ::core::ffi::c_int {
                (*p).has_grid_line_event = true_0 != 0;
                c2rust_current_block = 18326220445127191884;
                break '_done;
            } else {
                '_c2rust_label_0: {
                    if (*p).state == 12 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"p->state == 12\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/msgpack_rpc/unpacker.rs\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            333 as ::core::ffi::c_uint,
                            b"_Bool unpacker_advance(Unpacker *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                (*p).arena = ARENA_EMPTY;
                (*p).state = 13 as ::core::ffi::c_int;
            }
        }
        result = 0;
        c2rust_current_block = 11682394310085580349;
    }
    loop {
        match c2rust_current_block {
            11682394310085580349 => {
                result = mpack_parse(
                    &raw mut (*p).parser,
                    &raw mut (*p).read_ptr,
                    &raw mut (*p).read_size,
                    Some(
                        api_parse_enter
                            as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
                    ),
                    Some(
                        api_parse_exit
                            as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
                    ),
                );
                if result == MPACK_EOF as ::core::ffi::c_int {
                    return false_0 != 0;
                } else if result != MPACK_OK as ::core::ffi::c_int {
                    api_set_error(
                        &raw mut (*p).unpack_error,
                        kErrorTypeValidation,
                        b"failed to parse msgpack\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    (*p).state = -1 as ::core::ffi::c_int;
                    return false_0 != 0;
                }
                c2rust_current_block = 18326220445127191884;
            }
            _ => match (*p).state {
                1 => {
                    (*p).error = (*p).result;
                    (*p).state = 2 as ::core::ffi::c_int;
                    c2rust_current_block = 11682394310085580349;
                }
                2 => {
                    (*p).state = 0 as ::core::ffi::c_int;
                    return true_0 != 0;
                }
                13 | 16 => {
                    (*p).ncalls -= 1;
                    if (*p).ncalls > 0 as ::core::ffi::c_int {
                        (*p).state = if (*p).state == 16 as ::core::ffi::c_int {
                            14 as ::core::ffi::c_int
                        } else {
                            12 as ::core::ffi::c_int
                        };
                    } else if (*p).nevents > 0 as ::core::ffi::c_int {
                        (*p).state = 11 as ::core::ffi::c_int;
                    } else {
                        (*p).state = 0 as ::core::ffi::c_int;
                    }
                    return true_0 != 0;
                }
                _ => {
                    abort();
                }
            },
        }
    }
}
pub unsafe extern "C" fn unpacker_parse_redraw(mut p: *mut Unpacker) -> bool {
    let mut tok: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed_0 {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut result: ::core::ffi::c_int = 0;
    let mut data: *const ::core::ffi::c_char = (*p).read_ptr;
    let mut size: size_t = (*p).read_size;
    let mut g: *mut GridLineEvent = &raw mut (*p).grid_line_event;
    let mut eventarrsize: ::core::ffi::c_int = 0;
    'c_26780: {
        'c_26776: {
            'c_26758: {
                match (*p).state {
                    10 => {
                        result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
                        if result == MPACK_EOF as ::core::ffi::c_int {
                            return false_0 != 0;
                        } else if result != 0
                            || tok.type_0 as ::core::ffi::c_uint
                                != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                                && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                                    == MPACK_TOKEN_STR as ::core::ffi::c_int
                                    && tok.type_0 as ::core::ffi::c_uint
                                        == MPACK_TOKEN_BIN as ::core::ffi::c_int
                                            as ::core::ffi::c_uint)
                                && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                                    == MPACK_TOKEN_SINT as ::core::ffi::c_int
                                    && tok.type_0 as ::core::ffi::c_uint
                                        == MPACK_TOKEN_UINT as ::core::ffi::c_int
                                            as ::core::ffi::c_uint)
                        {
                            (*p).state = -1 as ::core::ffi::c_int;
                            return false_0 != 0;
                        }
                        (*p).nevents = tok.length as ::core::ffi::c_int;
                    }
                    11 => {}
                    14 => {
                        break 'c_26758;
                    }
                    15 => {
                        break 'c_26776;
                    }
                    16 => {
                        break 'c_26780;
                    }
                    12 => return true_0 != 0,
                    _ => {
                        abort();
                    }
                }
                result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
                if result == MPACK_EOF as ::core::ffi::c_int {
                    return false_0 != 0;
                } else if result != 0
                    || tok.type_0 as ::core::ffi::c_uint
                        != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                            == MPACK_TOKEN_STR as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                        && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                            == MPACK_TOKEN_SINT as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    (*p).state = -1 as ::core::ffi::c_int;
                    return false_0 != 0;
                }
                (*p).ncalls = tok.length as ::core::ffi::c_int;
                let c2rust_fresh0 = (*p).ncalls;
                (*p).ncalls = (*p).ncalls - 1;
                if c2rust_fresh0 == 0 as ::core::ffi::c_int {
                    (*p).state = -1 as ::core::ffi::c_int;
                    return false_0 != 0;
                }
                result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
                if result == MPACK_EOF as ::core::ffi::c_int {
                    return false_0 != 0;
                } else if result != 0
                    || tok.type_0 as ::core::ffi::c_uint
                        != MPACK_TOKEN_STR as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !(MPACK_TOKEN_STR as ::core::ffi::c_int
                            == MPACK_TOKEN_STR as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                        && !(MPACK_TOKEN_STR as ::core::ffi::c_int
                            == MPACK_TOKEN_SINT as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    (*p).state = -1 as ::core::ffi::c_int;
                    return false_0 != 0;
                }
                if tok.length as size_t > size {
                    return false_0 != 0;
                }
                (*p).ui_handler = ui_client_get_redraw_handler(
                    data,
                    tok.length as size_t,
                    ::core::ptr::null_mut::<Error>(),
                );
                data = data.offset(tok.length as isize);
                size = size.wrapping_sub(tok.length as size_t);
                (*p).nevents -= 1;
                (*p).read_ptr = data;
                (*p).read_size = size;
                if (*p).ui_handler.fn_0
                    != ::core::mem::transmute::<
                        Option<unsafe extern "C" fn(Array) -> !>,
                        Option<unsafe extern "C" fn(Array) -> ()>,
                    >(Some(
                        ui_client_event_grid_line as unsafe extern "C" fn(Array) -> !,
                    ))
                {
                    (*p).state = 12 as ::core::ffi::c_int;
                    return true_0 != 0;
                } else {
                    (*p).state = 14 as ::core::ffi::c_int;
                    (*p).arena = ARENA_EMPTY;
                }
            }
            result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
            if result == MPACK_EOF as ::core::ffi::c_int {
                return false_0 != 0;
            } else if result != 0
                || tok.type_0 as ::core::ffi::c_uint
                    != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                        == MPACK_TOKEN_STR as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                    && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                        == MPACK_TOKEN_SINT as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                (*p).state = -1 as ::core::ffi::c_int;
                return false_0 != 0;
            }
            eventarrsize = tok.length as ::core::ffi::c_int;
            if eventarrsize != 5 as ::core::ffi::c_int {
                (*p).state = -1 as ::core::ffi::c_int;
                return false_0 != 0;
            }
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 3 as ::core::ffi::c_int {
                result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
                if result == MPACK_EOF as ::core::ffi::c_int {
                    return false_0 != 0;
                } else if result != 0
                    || tok.type_0 as ::core::ffi::c_uint
                        != MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !(MPACK_TOKEN_UINT as ::core::ffi::c_int
                            == MPACK_TOKEN_STR as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                        && !(MPACK_TOKEN_UINT as ::core::ffi::c_int
                            == MPACK_TOKEN_SINT as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    (*p).state = -1 as ::core::ffi::c_int;
                    return false_0 != 0;
                }
                (*g).args[i as usize] = tok.data.value.lo as ::core::ffi::c_int;
                i += 1;
            }
            result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
            if result == MPACK_EOF as ::core::ffi::c_int {
                return false_0 != 0;
            } else if result != 0
                || tok.type_0 as ::core::ffi::c_uint
                    != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                        == MPACK_TOKEN_STR as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                    && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                        == MPACK_TOKEN_SINT as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                (*p).state = -1 as ::core::ffi::c_int;
                return false_0 != 0;
            }
            (*g).ncells = tok.length as ::core::ffi::c_int;
            (*g).icell = 0 as ::core::ffi::c_int;
            (*g).coloff = 0 as ::core::ffi::c_int;
            (*g).cur_attr = -1 as ::core::ffi::c_int;
            (*p).read_ptr = data;
            (*p).read_size = size;
            (*p).state = 15 as ::core::ffi::c_int;
        }
        while (*g).icell != (*g).ncells {
            '_c2rust_label: {
                if (*g).icell < (*g).ncells {
                } else {
                    __assert_fail(
                        b"g->icell < g->ncells\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/msgpack_rpc/unpacker.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        461 as ::core::ffi::c_uint,
                        b"_Bool unpacker_parse_redraw(Unpacker *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
            if result == MPACK_EOF as ::core::ffi::c_int {
                return false_0 != 0;
            } else if result != 0
                || tok.type_0 as ::core::ffi::c_uint
                    != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                        == MPACK_TOKEN_STR as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                    && !(MPACK_TOKEN_ARRAY as ::core::ffi::c_int
                        == MPACK_TOKEN_SINT as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                (*p).state = -1 as ::core::ffi::c_int;
                return false_0 != 0;
            }
            let mut cellarrsize: ::core::ffi::c_int = tok.length as ::core::ffi::c_int;
            if cellarrsize < 1 as ::core::ffi::c_int || cellarrsize > 3 as ::core::ffi::c_int {
                (*p).state = -1 as ::core::ffi::c_int;
                return false_0 != 0;
            }
            result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
            if result == MPACK_EOF as ::core::ffi::c_int {
                return false_0 != 0;
            } else if result != 0
                || tok.type_0 as ::core::ffi::c_uint
                    != MPACK_TOKEN_STR as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(MPACK_TOKEN_STR as ::core::ffi::c_int
                        == MPACK_TOKEN_STR as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                    && !(MPACK_TOKEN_STR as ::core::ffi::c_int
                        == MPACK_TOKEN_SINT as ::core::ffi::c_int
                        && tok.type_0 as ::core::ffi::c_uint
                            == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                (*p).state = -1 as ::core::ffi::c_int;
                return false_0 != 0;
            }
            if tok.length as size_t > size {
                return false_0 != 0;
            }
            let mut cellbuf: *const ::core::ffi::c_char = data;
            let mut cellsize: size_t = tok.length as size_t;
            data = data.offset(cellsize as isize);
            size = size.wrapping_sub(cellsize);
            if cellarrsize >= 2 as ::core::ffi::c_int {
                result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
                if result == MPACK_EOF as ::core::ffi::c_int {
                    return false_0 != 0;
                } else if result != 0
                    || tok.type_0 as ::core::ffi::c_uint
                        != MPACK_TOKEN_SINT as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !(MPACK_TOKEN_SINT as ::core::ffi::c_int
                            == MPACK_TOKEN_STR as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                        && !(MPACK_TOKEN_SINT as ::core::ffi::c_int
                            == MPACK_TOKEN_SINT as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    (*p).state = -1 as ::core::ffi::c_int;
                    return false_0 != 0;
                }
                (*g).cur_attr = tok.data.value.lo as ::core::ffi::c_int;
            }
            let mut repeat: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            if cellarrsize >= 3 as ::core::ffi::c_int {
                result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
                if result == MPACK_EOF as ::core::ffi::c_int {
                    return false_0 != 0;
                } else if result != 0
                    || tok.type_0 as ::core::ffi::c_uint
                        != MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !(MPACK_TOKEN_UINT as ::core::ffi::c_int
                            == MPACK_TOKEN_STR as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
                        && !(MPACK_TOKEN_UINT as ::core::ffi::c_int
                            == MPACK_TOKEN_SINT as ::core::ffi::c_int
                            && tok.type_0 as ::core::ffi::c_uint
                                == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    (*p).state = -1 as ::core::ffi::c_int;
                    return false_0 != 0;
                }
                repeat = tok.data.value.lo as ::core::ffi::c_int;
            }
            (*g).clear_width = 0 as ::core::ffi::c_int;
            if (*g).icell == (*g).ncells - 1 as ::core::ffi::c_int
                && cellsize == 1 as size_t
                && *cellbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
                && repeat > 1 as ::core::ffi::c_int
            {
                (*g).clear_width = repeat;
            } else {
                let mut sc: schar_T = schar_from_buf(cellbuf, cellsize);
                let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while r < repeat {
                    if (*g).coloff >= grid_line_buf_size.get() as ::core::ffi::c_int {
                        (*p).state = -1 as ::core::ffi::c_int;
                        return false_0 != 0;
                    }
                    *(*grid_line_buf_char.ptr()).offset((*g).coloff as isize) = sc;
                    let c2rust_fresh1 = (*g).coloff;
                    (*g).coloff = (*g).coloff + 1;
                    *(*grid_line_buf_attr.ptr()).offset(c2rust_fresh1 as isize) =
                        (*g).cur_attr as sattr_T;
                    r += 1;
                }
            }
            (*p).read_ptr = data;
            (*p).read_size = size;
            (*g).icell += 1;
        }
        (*p).state = 16 as ::core::ffi::c_int;
    }
    result = mpack_rtoken(&raw mut data, &raw mut size, &raw mut tok);
    if result == MPACK_EOF as ::core::ffi::c_int {
        return false_0 != 0;
    } else if result != 0
        || tok.type_0 as ::core::ffi::c_uint
            != MPACK_TOKEN_BOOLEAN as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(MPACK_TOKEN_BOOLEAN as ::core::ffi::c_int == MPACK_TOKEN_STR as ::core::ffi::c_int
                && tok.type_0 as ::core::ffi::c_uint
                    == MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint)
            && !(MPACK_TOKEN_BOOLEAN as ::core::ffi::c_int
                == MPACK_TOKEN_SINT as ::core::ffi::c_int
                && tok.type_0 as ::core::ffi::c_uint
                    == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        (*p).state = -1 as ::core::ffi::c_int;
        return false_0 != 0;
    }
    (*g).wrap = mpack_unpack_boolean(tok);
    (*p).read_ptr = data;
    (*p).read_size = size;
    return true_0 != 0;
}
pub unsafe extern "C" fn unpack_string(
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
) -> String_0 {
    let mut data2: *const ::core::ffi::c_char = *data;
    let mut size2: size_t = *size;
    let mut tok: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed_0 {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut result: ::core::ffi::c_int = mpack_rtoken(&raw mut data2, &raw mut size2, &raw mut tok);
    if result != 0
        || tok.type_0 as ::core::ffi::c_uint
            != MPACK_TOKEN_STR as ::core::ffi::c_int as ::core::ffi::c_uint
            && tok.type_0 as ::core::ffi::c_uint
                != MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return STRING_INIT;
    }
    if *size < tok.length as size_t {
        return STRING_INIT;
    }
    *data = data2.offset(tok.length as isize);
    *size = size2.wrapping_sub(tok.length as size_t);
    return String_0 {
        data: data2 as *mut ::core::ffi::c_char,
        size: tok.length as size_t,
    };
}
pub unsafe extern "C" fn unpack_array(
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
) -> ssize_t {
    let mut tok: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed_0 {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut result: ::core::ffi::c_int = mpack_rtoken(data, size, &raw mut tok);
    if result != 0
        || tok.type_0 as ::core::ffi::c_uint
            != MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -1 as ssize_t;
    }
    return tok.length as ssize_t;
}
pub unsafe extern "C" fn unpack_integer(
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
    mut res: *mut Integer,
) -> bool {
    let mut tok: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed_0 {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut result: ::core::ffi::c_int = mpack_rtoken(data, size, &raw mut tok);
    if result != 0 {
        return false_0 != 0;
    }
    return unpack_uint_or_sint(tok, res);
}
pub unsafe extern "C" fn unpack_uint_or_sint(
    mut tok: mpack_token_t,
    mut res: *mut Integer,
) -> bool {
    if tok.type_0 as ::core::ffi::c_uint
        == MPACK_TOKEN_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *res = mpack_unpack_uint(tok) as Integer;
        return true_0 != 0;
    } else if tok.type_0 as ::core::ffi::c_uint
        == MPACK_TOKEN_SINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *res = mpack_unpack_sint(tok) as Integer;
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn parse_nop(mut _parser: *mut mpack_parser_t, mut _node: *mut mpack_node_t) {}
pub unsafe extern "C" fn unpack_skip(
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut parser: mpack_parser_t = mpack_parser_t {
        data: mpack_data_t {
            p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        size: 0,
        capacity: 0,
        status: 0,
        exiting: 0,
        tokbuf: mpack_tokbuf_t {
            pending: [0; 9],
            pending_tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed_0 {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            ppos: 0,
            plen: 0,
            passthrough: 0,
        },
        items: [mpack_node_t {
            tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed_0 {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            pos: 0,
            key_visited: 0,
            data: [mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            }; 2],
        }; 33],
    };
    mpack_parser_init(&raw mut parser, 0 as mpack_uint32_t);
    return mpack_parse(
        &raw mut parser,
        data,
        size,
        Some(parse_nop as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> ()),
        Some(parse_nop as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> ()),
    );
}
pub unsafe extern "C" fn push_additional_data(
    mut ad: *mut AdditionalDataBuilder,
    mut data: *const ::core::ffi::c_char,
    mut size: size_t,
) {
    if (*ad).size == 0 as size_t {
        let mut init: AdditionalData = AdditionalData {
            nitems: 0 as uint32_t,
            nbytes: 0,
            data: [],
        };
        if ::core::mem::size_of::<AdditionalData>() > 0 as usize {
            if (*ad).capacity
                < (*ad)
                    .size
                    .wrapping_add(::core::mem::size_of::<AdditionalData>())
            {
                (*ad).capacity = (*ad)
                    .size
                    .wrapping_add(::core::mem::size_of::<AdditionalData>());
                (*ad).capacity = (*ad).capacity.wrapping_sub(1);
                (*ad).capacity |= (*ad).capacity >> 1 as ::core::ffi::c_int;
                (*ad).capacity |= (*ad).capacity >> 2 as ::core::ffi::c_int;
                (*ad).capacity |= (*ad).capacity >> 4 as ::core::ffi::c_int;
                (*ad).capacity |= (*ad).capacity >> 8 as ::core::ffi::c_int;
                (*ad).capacity |= (*ad).capacity >> 16 as ::core::ffi::c_int;
                (*ad).capacity = (*ad).capacity.wrapping_add(1);
                (*ad).capacity = (*ad).capacity;
                (*ad).items = xrealloc(
                    (*ad).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*ad).capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label: {
                if !(*ad).items.is_null() {
                } else {
                    __assert_fail(
                        b"(*ad).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/msgpack_rpc/unpacker.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        605 as ::core::ffi::c_uint,
                        b"void push_additional_data(AdditionalDataBuilder *, const char *, size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            memcpy(
                (*ad).items.offset((*ad).size as isize) as *mut ::core::ffi::c_void,
                &raw mut init as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(::core::mem::size_of::<AdditionalData>()),
            );
            (*ad).size = (*ad)
                .size
                .wrapping_add(::core::mem::size_of::<AdditionalData>());
        }
    }
    let mut a: *mut AdditionalData = (*ad).items as *mut AdditionalData;
    (*a).nitems = (*a).nitems.wrapping_add(1);
    (*a).nbytes = (*a).nbytes.wrapping_add(size as uint32_t);
    if size > 0 as size_t {
        if (*ad).capacity < (*ad).size.wrapping_add(size) {
            (*ad).capacity = (*ad).size.wrapping_add(size);
            (*ad).capacity = (*ad).capacity.wrapping_sub(1);
            (*ad).capacity |= (*ad).capacity >> 1 as ::core::ffi::c_int;
            (*ad).capacity |= (*ad).capacity >> 2 as ::core::ffi::c_int;
            (*ad).capacity |= (*ad).capacity >> 4 as ::core::ffi::c_int;
            (*ad).capacity |= (*ad).capacity >> 8 as ::core::ffi::c_int;
            (*ad).capacity |= (*ad).capacity >> 16 as ::core::ffi::c_int;
            (*ad).capacity = (*ad).capacity.wrapping_add(1);
            (*ad).capacity = (*ad).capacity;
            (*ad).items = xrealloc(
                (*ad).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*ad).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label_0: {
            if !(*ad).items.is_null() {
            } else {
                __assert_fail(
                    b"(*ad).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/msgpack_rpc/unpacker.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    610 as ::core::ffi::c_uint,
                    b"void push_additional_data(AdditionalDataBuilder *, const char *, size_t)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            (*ad).items.offset((*ad).size as isize) as *mut ::core::ffi::c_void,
            data as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(size),
        );
        (*ad).size = (*ad).size.wrapping_add(size);
    }
}
pub unsafe extern "C" fn unpack_keydict(
    mut retval: *mut ::core::ffi::c_void,
    mut hashy: FieldHashfn,
    mut ad: *mut AdditionalDataBuilder,
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
    mut error: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut ks: *mut OptKeySet = retval as *mut OptKeySet;
    let mut tok: mpack_token_t = mpack_token_t {
        type_0: 0 as mpack_token_type_t,
        length: 0,
        data: C2Rust_Unnamed_0 {
            value: mpack_value_t { lo: 0, hi: 0 },
        },
    };
    let mut result: ::core::ffi::c_int = mpack_rtoken(data, size, &raw mut tok);
    if result != 0
        || tok.type_0 as ::core::ffi::c_uint
            != MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *error = xstrdup(b"is not a dict\0".as_ptr() as *const ::core::ffi::c_char);
        return false_0 != 0;
    }
    let mut map_size: size_t = tok.length as size_t;
    let mut i: size_t = 0 as size_t;
    while i < map_size {
        let mut item_start: *const ::core::ffi::c_char = *data;
        let mut key: String_0 = unpack_string(data, size);
        if key.data.is_null() {
            *error = arena_printf(
                ::core::ptr::null_mut::<Arena>(),
                b"has key value which is not a string\0".as_ptr() as *const ::core::ffi::c_char,
            )
            .data;
            return false_0 != 0;
        } else if key.size == 0 as size_t {
            *error = arena_printf(
                ::core::ptr::null_mut::<Arena>(),
                b"has empty key\0".as_ptr() as *const ::core::ffi::c_char,
            )
            .data;
            return false_0 != 0;
        }
        let mut field: *mut KeySetLink =
            hashy.expect("non-null function pointer")(key.data, key.size);
        if field.is_null() {
            let mut status: ::core::ffi::c_int = unpack_skip(data, size);
            if status != 0 {
                return false_0 != 0;
            }
            if !ad.is_null() {
                push_additional_data(ad, item_start, (*data).offset_from(item_start) as size_t);
            }
        } else {
            '_c2rust_label: {
                if (*field).opt_index >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"field->opt_index >= 0\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/msgpack_rpc/unpacker.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        653 as ::core::ffi::c_uint,
                        b"_Bool unpack_keydict(void *, FieldHashfn, AdditionalDataBuilder *, const char **, size_t *restrict, char **)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut flag: uint64_t =
                ((1 as ::core::ffi::c_ulonglong) << (*field).opt_index) as uint64_t;
            if (*ks).is_set_ & flag as OptionalKeys != 0 {
                *error = xstrdup(b"duplicate key\0".as_ptr() as *const ::core::ffi::c_char);
                return false_0 != 0;
            }
            (*ks).is_set_ |= flag;
            let mut mem: *mut ::core::ffi::c_char =
                (retval as *mut ::core::ffi::c_char).offset((*field).ptr_off as isize);
            match (*field).type_0 {
                1 => {
                    if *size == 0 as size_t
                        || **data as ::core::ffi::c_int & 0xfe as ::core::ffi::c_int
                            != 0xc2 as ::core::ffi::c_int
                    {
                        *error = arena_printf(
                            ::core::ptr::null_mut::<Arena>(),
                            b"has %.*s key value which is not a boolean\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            key.size as ::core::ffi::c_int,
                            key.data,
                        )
                        .data;
                        return false_0 != 0;
                    }
                    *(mem as *mut Boolean) =
                        **data as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int != 0;
                    *data = (*data).offset(1);
                    *size = (*size).wrapping_sub(1);
                }
                2 => {
                    if !unpack_integer(data, size, mem as *mut Integer) {
                        *error = arena_printf(
                            ::core::ptr::null_mut::<Arena>(),
                            b"has %.*s key value which is not an integer\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            key.size as ::core::ffi::c_int,
                            key.data,
                        )
                        .data;
                        return false_0 != 0;
                    }
                }
                4 => {
                    let mut val: String_0 = unpack_string(data, size);
                    if val.data.is_null() {
                        *error = arena_printf(
                            ::core::ptr::null_mut::<Arena>(),
                            b"has %.*s key value which is not a binary\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            key.size as ::core::ffi::c_int,
                            key.data,
                        )
                        .data;
                        return false_0 != 0;
                    }
                    *(mem as *mut String_0) = val;
                }
                -1 => {
                    let mut len: ssize_t = unpack_array(data, size);
                    if len < 0 as ssize_t {
                        *error = arena_printf(
                            ::core::ptr::null_mut::<Arena>(),
                            b"has %.*s key with non-array value\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            key.size as ::core::ffi::c_int,
                            key.data,
                        )
                        .data;
                        return false_0 != 0;
                    }
                    let mut a: *mut StringArray = mem as *mut StringArray;
                    if (*a).capacity < (*a).size.wrapping_add(len as size_t) {
                        (*a).capacity = (*a).size.wrapping_add(len as size_t);
                        (*a).capacity = (*a).capacity.wrapping_sub(1);
                        (*a).capacity |= (*a).capacity >> 1 as ::core::ffi::c_int;
                        (*a).capacity |= (*a).capacity >> 2 as ::core::ffi::c_int;
                        (*a).capacity |= (*a).capacity >> 4 as ::core::ffi::c_int;
                        (*a).capacity |= (*a).capacity >> 8 as ::core::ffi::c_int;
                        (*a).capacity |= (*a).capacity >> 16 as ::core::ffi::c_int;
                        (*a).capacity = (*a).capacity.wrapping_add(1);
                        (*a).capacity = (*a).capacity;
                        (*a).items = xrealloc(
                            (*a).items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<String_0>().wrapping_mul((*a).capacity),
                        ) as *mut String_0;
                    }
                    let mut j: size_t = 0 as size_t;
                    while j < len as size_t {
                        let mut item: String_0 = unpack_string(data, size);
                        if item.data.is_null() {
                            *error = arena_printf(
                                ::core::ptr::null_mut::<Arena>(),
                                b"has %.*s array with non-binary value\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                key.size as ::core::ffi::c_int,
                                key.data,
                            )
                            .data;
                            return false_0 != 0;
                        }
                        if (*a).size == (*a).capacity {
                            (*a).capacity = if (*a).capacity != 0 {
                                (*a).capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            (*a).items = xrealloc(
                                (*a).items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<String_0>().wrapping_mul((*a).capacity),
                            ) as *mut String_0;
                        } else {
                        };
                        let c2rust_fresh2 = (*a).size;
                        (*a).size = (*a).size.wrapping_add(1);
                        *(*a).items.offset(c2rust_fresh2 as isize) = item;
                        j = j.wrapping_add(1);
                    }
                }
                _ => {
                    abort();
                }
            }
        }
        i = i.wrapping_add(1);
    }
    return true_0 != 0;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
