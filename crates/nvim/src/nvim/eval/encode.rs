use crate::src::nvim::eval::typval::{GARRAY_EMPTY, tv_list_first, tv_list_last, tv_list_len};
use crate::src::nvim::eval::typval::{
    tv_dict_find, tv_list_append_allocated_string, tv_list_idx_of_item,
};
use crate::src::nvim::eval::typval_encode::{ConvPath, Flow, Frame, PartialStage};
use crate::src::nvim::eval::vars::eval_msgpack_type_lists;
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat, ga_concat_len, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::IObuff;
use crate::src::nvim::mbyte::{utf_char2len, utf_printable, utf_ptr2char, utf_ptr2len};
use crate::src::nvim::memory::{memchrsub, xfree, xmalloc, xmemdupz, xmemscan, xrealloc};
use crate::src::nvim::message::semsg;
use crate::src::nvim::os::libc::{__assert_fail, abort, gettext, memcpy, strlen};
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::{
    ListReaderState, MPConvPartialStage, MPConvStackValType, MessagePackType, VAR_DICT, VAR_FUNC,
    VAR_LIST, VAR_STRING, VAR_UNLOCKED, dict_T, dictitem_T, garray_T, list_T, listitem_T,
    ptrdiff_t, size_t, typval_T, typval_vval_union, uint8_t,
};
use core::ffi::c_char;
pub const kMPString: MessagePackType = 4;
pub const kMPConvPartialEnd: MPConvPartialStage = 2;
pub const kMPConvPartialSelf: MPConvPartialStage = 1;
pub const kMPConvPartialArgs: MPConvPartialStage = 0;
pub const kMPConvPartialList: MPConvStackValType = 4;
pub const kMPConvPartial: MPConvStackValType = 3;
pub const kMPConvPairs: MPConvStackValType = 2;
pub const kMPConvList: MPConvStackValType = 1;
pub const kMPConvDict: MPConvStackValType = 0;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = 8;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const FF: ::core::ffi::c_int = 12;
pub const CAR: ::core::ffi::c_int = 13;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

// The sinks carved out of this module's `typval_encode.c.h` instantiations.
mod json;
use self::json::encode_vim_to_json;
mod msgpack;
pub use self::msgpack::*;
mod text;
use self::text::{encode_vim_to_echo, encode_vim_to_string};
pub const SURROGATE_HI_START: ::core::ffi::c_int = 0xd800 as ::core::ffi::c_int;
pub const SURROGATE_HI_END: ::core::ffi::c_int = 0xdbff as ::core::ffi::c_int;
pub const SURROGATE_LO_START: ::core::ffi::c_int = 0xdc00 as ::core::ffi::c_int;
pub const SURROGATE_LO_END: ::core::ffi::c_int = 0xdfff as ::core::ffi::c_int;
pub const SURROGATE_FIRST_CHAR: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub static encode_bool_var_names: GlobalCell<[*const ::core::ffi::c_char; 2]> = GlobalCell::new([
    b"v:false\0".as_ptr() as *const ::core::ffi::c_char,
    b"v:true\0".as_ptr() as *const ::core::ffi::c_char,
]);
pub static encode_special_var_names: GlobalCell<[*const ::core::ffi::c_char; 1]> =
    GlobalCell::new([b"v:null\0".as_ptr() as *const ::core::ffi::c_char]);
#[unsafe(no_mangle)]
pub unsafe extern "C" fn encode_list_write(
    data: *mut ::core::ffi::c_void,
    buf: *const ::core::ffi::c_char,
    len: size_t,
) {
    if len == 0 as size_t {
        return;
    }
    let list: *mut list_T = data as *mut list_T;
    let end: *const ::core::ffi::c_char = buf.offset(len as isize);
    let mut line_end: *const ::core::ffi::c_char = buf;
    let mut li: *mut listitem_T = tv_list_last(list);
    if !li.is_null() {
        line_end = xmemscan(
            buf as *const ::core::ffi::c_void,
            NL as ::core::ffi::c_char,
            len,
        ) as *const ::core::ffi::c_char;
        if line_end != buf {
            let line_length: size_t = line_end.offset_from(buf) as size_t;
            let mut str: *mut ::core::ffi::c_char = (*li).li_tv.vval.v_string;
            let li_len: size_t = if str.is_null() {
                0 as size_t
            } else {
                strlen(str)
            };
            (*li).li_tv.vval.v_string = xrealloc(
                str as *mut ::core::ffi::c_void,
                li_len.wrapping_add(line_length).wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            str = (*li).li_tv.vval.v_string.offset(li_len as isize);
            memcpy(
                str as *mut ::core::ffi::c_void,
                buf as *const ::core::ffi::c_void,
                line_length,
            );
            *str.offset(line_length as isize) = 0 as ::core::ffi::c_char;
            memchrsub(
                str as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                line_length,
            );
        }
        line_end = line_end.offset(1);
    }
    while line_end < end {
        let mut line_start: *const ::core::ffi::c_char = line_end;
        line_end = xmemscan(
            line_start as *const ::core::ffi::c_void,
            NL as ::core::ffi::c_char,
            end.offset_from(line_start) as size_t,
        ) as *const ::core::ffi::c_char;
        let mut str_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if line_end != line_start {
            let line_length_0: size_t = line_end.offset_from(line_start) as size_t;
            str_0 = xmemdupz(line_start as *const ::core::ffi::c_void, line_length_0)
                as *mut ::core::ffi::c_char;
            memchrsub(
                str_0 as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                line_length_0,
            );
        }
        tv_list_append_allocated_string(list, str_0);
        line_end = line_end.offset(1);
    }
    if line_end == end {
        tv_list_append_allocated_string(list, ::core::ptr::null_mut::<::core::ffi::c_char>());
    }
}
/// Set once a `string()`/`echo` dump has reported a self-reference, so the
/// user is told once rather than once per cycle.
pub(crate) static did_echo_string_emsg: GlobalCell<bool> = GlobalCell::new(false);

/// Report a failed dump, naming the path down to the value that failed.
///
/// `msg` must carry exactly two `%s`: the object being dumped, then the path
/// — "key foo, index 2, key bar" — which this builds out of the walk's stack.
/// Always answers [`Flow::Fail`], because that is all its callers do with it.
///
/// # Safety
/// `msg` must be a NUL-terminated format string of that shape.
pub(crate) unsafe fn conv_error(msg: *const c_char, path: &ConvPath) -> Flow {
    unsafe {
        let key_msg = gettext(c"key %s".as_ptr());
        let key_pair_msg = gettext(c"key %s at index %i from special map".as_ptr());
        let idx_msg = gettext(c"index %i".as_ptr());
        let partial_arg_msg = gettext(c"partial".as_ptr());
        let partial_arg_i_msg = gettext(c"argument %i".as_ptr());
        let partial_self_msg = gettext(c"partial self dictionary".as_ptr());

        let mut msg_ga = GARRAY_EMPTY;
        ga_init(
            &raw mut msg_ga,
            ::core::mem::size_of::<c_char>() as ::core::ffi::c_int,
            80,
        );
        let iobuff = IObuff.ptr() as *mut c_char;
        for (i, frame) in path.stack.iter().enumerate() {
            if i != 0 {
                ga_concat_len(&raw mut msg_ga, c", ".as_ptr(), 2);
            }
            match frame.frame {
                Frame::Dict { dict, hi, .. } => {
                    // The key most recently handed out, which is the slot
                    // before the one the walk is now standing on.
                    let hi = if hi.is_null() {
                        (*dict).dv_hashtab.ht_array
                    } else {
                        hi.sub(1)
                    };
                    let mut key_tv = typval_T {
                        v_type: VAR_STRING,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union {
                            v_string: (*hi).hi_key,
                        },
                    };
                    let key = encode_tv2string(&raw mut key_tv, ::core::ptr::null_mut());
                    vim_snprintf(iobuff, IOSIZE as size_t, key_msg, key);
                    xfree(key as *mut ::core::ffi::c_void);
                    ga_concat(&raw mut msg_ga, iobuff);
                }
                Frame::List { list, li } | Frame::Pairs { list, li } => {
                    // The item most recently handed out: one back from `li`,
                    // or the last one once the walk has run off the end.
                    let idx = if li == tv_list_first(list) {
                        0
                    } else if li.is_null() {
                        tv_list_len(list) - 1
                    } else {
                        tv_list_idx_of_item(list, (*li).li_prev)
                    };
                    let li = if li.is_null() {
                        tv_list_last(list)
                    } else {
                        (*li).li_prev
                    };
                    let pairs = matches!(frame.frame, Frame::Pairs { .. });
                    if !pairs
                        || li.is_null()
                        || ((*li).li_tv.v_type != VAR_LIST
                            && tv_list_len((*li).li_tv.vval.v_list) <= 0)
                    {
                        vim_snprintf(iobuff, IOSIZE as size_t, idx_msg, idx);
                    } else {
                        // A special map's item is a [key, value] pair, so the
                        // path can name the key rather than the index.
                        let first_item = tv_list_first((*li).li_tv.vval.v_list);
                        let mut key_tv = (*first_item).li_tv;
                        let key = encode_tv2echo(&raw mut key_tv, ::core::ptr::null_mut());
                        vim_snprintf(iobuff, IOSIZE as size_t, key_pair_msg, key, idx);
                        xfree(key as *mut ::core::ffi::c_void);
                    }
                    ga_concat(&raw mut msg_ga, iobuff);
                }
                Frame::Partial { stage, .. } => match stage {
                    // The walk pushes a partial already past its arguments.
                    PartialStage::Args => abort(),
                    PartialStage::Self_ => ga_concat(&raw mut msg_ga, partial_arg_msg),
                    PartialStage::End => ga_concat(&raw mut msg_ga, partial_self_msg),
                },
                Frame::PartialArgs { arg, argv, .. } => {
                    let idx = arg.offset_from(argv) as ::core::ffi::c_int - 1;
                    vim_snprintf(iobuff, IOSIZE as size_t, partial_arg_i_msg, idx);
                    ga_concat(&raw mut msg_ga, iobuff);
                }
            }
        }
        semsg(
            msg,
            gettext(path.objname),
            if path.stack.is_empty() {
                gettext(c"itself".as_ptr())
            } else {
                msg_ga.ga_data as *mut c_char
            },
        );
        ga_clear(&raw mut msg_ga);
        Flow::Fail
    }
}
pub unsafe extern "C" fn encode_vim_list_to_buf(
    list: *const list_T,
    ret_len: *mut size_t,
    ret_buf: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut len: size_t = 0 as size_t;
    let l_: *const list_T = list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return false;
            }
            len = len.wrapping_add(1);
            if !(*li).li_tv.vval.v_string.is_null() {
                len = len.wrapping_add(strlen((*li).li_tv.vval.v_string));
            }
            li = (*li).li_next;
        }
    }
    if len != 0 {
        len = len.wrapping_sub(1);
    }
    *ret_len = len;
    if len == 0 as size_t {
        *ret_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return true_0 != 0;
    }
    let mut lrstate: ListReaderState = encode_init_lrstate(list);
    let buf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    let mut read_bytes: size_t = 0;
    if encode_read_from_list(&raw mut lrstate, buf, len, &raw mut read_bytes) != OK {
        abort();
    }
    '_c2rust_label: {
        if len == read_bytes {
        } else {
            __assert_fail(
                b"len == read_bytes\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/encode.rs\0".as_ptr() as *const ::core::ffi::c_char,
                240 as ::core::ffi::c_uint,
                b"_Bool encode_vim_list_to_buf(const list_T *const, size_t *const, char **const)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    *ret_buf = buf;
    return true_0 != 0;
}
pub unsafe extern "C" fn encode_read_from_list(
    state: *mut ListReaderState,
    buf: *mut ::core::ffi::c_char,
    nbuf: size_t,
    read_bytes: *mut size_t,
) -> ::core::ffi::c_int {
    let buf_end: *mut ::core::ffi::c_char = buf.offset(nbuf as isize);
    let mut p: *mut ::core::ffi::c_char = buf;
    while p < buf_end {
        '_c2rust_label: {
            if (*state).li_length == 0 as size_t || !(*(*state).li).li_tv.vval.v_string.is_null() {
            } else {
                __assert_fail(
                    b"state->li_length == 0 || TV_LIST_ITEM_TV(state->li)->vval.v_string != NULL\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/encode.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    265 as ::core::ffi::c_uint,
                    b"int encode_read_from_list(ListReaderState *const, char *const, const size_t, size_t *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut i: size_t = (*state).offset;
        while i < (*state).li_length && p < buf_end {
            '_c2rust_label_0: {
                if !(*(*state).li).li_tv.vval.v_string.is_null() {
                } else {
                    __assert_fail(
                        b"TV_LIST_ITEM_TV(state->li)->vval.v_string != NULL\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        267 as ::core::ffi::c_uint,
                        b"int encode_read_from_list(ListReaderState *const, char *const, const size_t, size_t *const)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let c2rust_fresh27 = (*state).offset;
            (*state).offset = (*state).offset.wrapping_add(1);
            let ch: ::core::ffi::c_char = *(*(*state).li)
                .li_tv
                .vval
                .v_string
                .offset(c2rust_fresh27 as isize);
            let c2rust_fresh28 = p;
            p = p.offset(1);
            *c2rust_fresh28 =
                (if ch as ::core::ffi::c_int == NL as ::core::ffi::c_char as ::core::ffi::c_int {
                    NUL as ::core::ffi::c_char as ::core::ffi::c_int
                } else {
                    ch as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
            i = i.wrapping_add(1);
        }
        if p < buf_end {
            (*state).li = (*(*state).li).li_next;
            if (*state).li.is_null() {
                *read_bytes = p.offset_from(buf) as size_t;
                return OK;
            }
            let c2rust_fresh29 = p;
            p = p.offset(1);
            *c2rust_fresh29 = NL as ::core::ffi::c_char;
            if (*(*state).li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *read_bytes = p.offset_from(buf) as size_t;
                return FAIL;
            }
            (*state).offset = 0 as size_t;
            (*state).li_length = if (*(*state).li).li_tv.vval.v_string.is_null() {
                0 as size_t
            } else {
                strlen((*(*state).li).li_tv.vval.v_string)
            };
        }
    }
    *read_bytes = nbuf;
    return if (*state).offset < (*state).li_length || !(*(*state).li).li_next.is_null() {
        NOTDONE
    } else {
        OK
    };
}
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = false_0;
pub const TYPVAL_ENCODE_ALLOW_SPECIALS_1: ::core::ffi::c_int = true_0;
static escapes: GlobalCell<[[::core::ffi::c_char; 3]; 93]> = GlobalCell::new(unsafe {
    [
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\b\0"),
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\t\0"),
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\n\0"),
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\f\0"),
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\r\0"),
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\\"\0"),
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        [0; 3],
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"\\\\\0"),
    ]
});
static xdigits: GlobalCell<[::core::ffi::c_char; 17]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"0123456789ABCDEF\0")
});
#[inline(always)]
pub(crate) unsafe extern "C" fn convert_to_json_string(
    gap: *mut garray_T,
    buf: *const ::core::ffi::c_char,
    len: size_t,
) -> ::core::ffi::c_int {
    let mut utf_buf: *const ::core::ffi::c_char = buf;
    if utf_buf.is_null() {
        ga_concat_len(
            gap,
            b"\"\"\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        );
    } else {
        let mut utf_len: size_t = len;
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut str_len: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < utf_len {
            let ch: ::core::ffi::c_int = utf_ptr2char(utf_buf.offset(i as isize));
            let shift: size_t = if ch == 0 as ::core::ffi::c_int {
                1 as size_t
            } else {
                utf_ptr2len(utf_buf.offset(i as isize)) as size_t
            };
            '_c2rust_label: {
                if shift > 0 as size_t {
                } else {
                    __assert_fail(
                        b"shift > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        643 as ::core::ffi::c_uint,
                        b"int convert_to_json_string(garray_T *const, const char *const, const size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            i = i.wrapping_add(shift);
            match ch {
                BS | TAB | NL | FF | CAR | 34 | 92 => {
                    str_len = str_len.wrapping_add(2 as size_t);
                }
                _ => {
                    if ch > 0x7f as ::core::ffi::c_int && shift == 1 as size_t {
                        semsg(
                            gettext(
                                b"E474: String \"%.*s\" contains byte that does not start any UTF-8 character\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            utf_len.wrapping_sub(i.wrapping_sub(shift))
                                as ::core::ffi::c_int,
                            utf_buf.offset(i as isize).offset(-(shift as isize)),
                        );
                        xfree(tofree as *mut ::core::ffi::c_void);
                        return FAIL;
                    } else if SURROGATE_HI_START <= ch && ch <= SURROGATE_HI_END
                        || SURROGATE_LO_START <= ch && ch <= SURROGATE_LO_END
                    {
                        semsg(
                            gettext(
                                b"E474: UTF-8 string contains code point which belongs to a surrogate pair: %.*s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            utf_len.wrapping_sub(i.wrapping_sub(shift))
                                as ::core::ffi::c_int,
                            utf_buf.offset(i as isize).offset(-(shift as isize)),
                        );
                        xfree(tofree as *mut ::core::ffi::c_void);
                        return FAIL;
                    } else if ch >= 0x20 as ::core::ffi::c_int
                        && utf_printable(ch) as ::core::ffi::c_int != 0
                    {
                        str_len = str_len.wrapping_add(shift);
                    } else {
                        str_len = (str_len as ::core::ffi::c_ulong).wrapping_add(
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as usize)
                                .wrapping_mul(
                                    (1 as ::core::ffi::c_int
                                        + (ch >= SURROGATE_FIRST_CHAR) as ::core::ffi::c_int)
                                        as usize,
                                ) as ::core::ffi::c_ulong,
                        ) as size_t;
                    }
                }
            }
        }
        ga_append(gap, '"' as uint8_t);
        ga_grow(gap, str_len as ::core::ffi::c_int);
        let mut i_0: size_t = 0 as size_t;
        while i_0 < utf_len {
            let ch_0: ::core::ffi::c_int = utf_ptr2char(utf_buf.offset(i_0 as isize));
            let shift_0: size_t = if ch_0 == 0 as ::core::ffi::c_int {
                1 as size_t
            } else {
                utf_char2len(ch_0) as size_t
            };
            '_c2rust_label_0: {
                if shift_0 > 0 as size_t {
                } else {
                    __assert_fail(
                        b"shift > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        683 as ::core::ffi::c_uint,
                        b"int convert_to_json_string(garray_T *const, const char *const, const size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            '_c2rust_label_1: {
                if ch_0 == 0 as ::core::ffi::c_int
                    || shift_0 == utf_ptr2len(utf_buf.offset(i_0 as isize)) as size_t
                {
                } else {
                    __assert_fail(
                        b"ch == 0 || shift == ((size_t)utf_ptr2len(utf_buf + i))\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/encode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        685 as ::core::ffi::c_uint,
                        b"int convert_to_json_string(garray_T *const, const char *const, const size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            match ch_0 {
                BS | TAB | NL | FF | CAR | 34 | 92 => {
                    ga_concat_len(
                        gap,
                        &raw const *((escapes.ptr() as *const _) as *const [::core::ffi::c_char; 3])
                            .offset(ch_0 as isize)
                            as *const ::core::ffi::c_char,
                        2 as size_t,
                    );
                }
                _ => {
                    if ch_0 >= 0x20 as ::core::ffi::c_int
                        && utf_printable(ch_0) as ::core::ffi::c_int != 0
                    {
                        ga_concat_len(gap, utf_buf.offset(i_0 as isize), shift_0);
                    } else if ch_0 < SURROGATE_FIRST_CHAR {
                        let c2rust_lvalue: [::core::ffi::c_char; 6] = [
                            '\\' as ::core::ffi::c_char,
                            'u' as ::core::ffi::c_char,
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 1 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(ch_0
                                >> 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                        ];
                        ga_concat_len(
                            gap,
                            &raw const c2rust_lvalue as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t),
                        );
                    } else {
                        let tmp: ::core::ffi::c_int = ch_0 - SURROGATE_FIRST_CHAR;
                        let hi: ::core::ffi::c_int = SURROGATE_HI_START
                            + (tmp >> 10 as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int) << 10 as ::core::ffi::c_int)
                                    - 1 as ::core::ffi::c_int);
                        let lo: ::core::ffi::c_int = SURROGATE_LO_END
                            + (tmp >> 0 as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int) << 10 as ::core::ffi::c_int)
                                    - 1 as ::core::ffi::c_int);
                        let c2rust_lvalue_0: [::core::ffi::c_char; 12] = [
                            '\\' as ::core::ffi::c_char,
                            'u' as ::core::ffi::c_char,
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 1 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(hi
                                >> 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            '\\' as ::core::ffi::c_char,
                            'u' as ::core::ffi::c_char,
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 1 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                            (*xdigits.ptr())[(lo
                                >> 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
                                & 0xf as ::core::ffi::c_int)
                                as usize],
                        ];
                        ga_concat_len(
                            gap,
                            &raw const c2rust_lvalue_0 as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t)
                                .wrapping_mul(2 as size_t),
                        );
                    }
                }
            }
            i_0 = i_0.wrapping_add(shift_0);
        }
        ga_append(gap, '"' as uint8_t);
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    return OK;
}
pub unsafe extern "C" fn encode_check_json_key(tv: *const typval_T) -> bool {
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return true_0 != 0;
    }
    if (*tv).v_type as ::core::ffi::c_uint != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    let spdict: *const dict_T = (*tv).vval.v_dict;
    if (*spdict).dv_hashtab.ht_used != 2 as size_t {
        return false_0 != 0;
    }
    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
    type_di = tv_dict_find(
        spdict,
        b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if type_di.is_null()
        || (*type_di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*type_di).di_tv.vval.v_list
            != (*eval_msgpack_type_lists.ptr())[kMPString as ::core::ffi::c_int as usize]
                as *mut list_T
        || {
            val_di = tv_dict_find(
                spdict,
                b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            val_di.is_null()
        }
        || (*val_di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    if (*val_di).di_tv.vval.v_list.is_null() {
        return true_0 != 0;
    }
    let l_: *const list_T = (*val_di).di_tv.vval.v_list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return false;
            }
            li = (*li).li_next;
        }
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn encode_tv2string(
    mut tv: *mut typval_T,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let evs_ret = encode_vim_to_string(&raw mut ga, tv, c"encode_tv2string() argument".as_ptr());
    debug_assert!(evs_ret);
    did_echo_string_emsg.set(false);
    if !len.is_null() {
        *len = ga.ga_len as size_t;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn encode_tv2echo(
    mut tv: *mut typval_T,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !(*tv).vval.v_string.is_null() {
            ga_concat(&raw mut ga, (*tv).vval.v_string);
        }
    } else {
        let eve_ret = encode_vim_to_echo(&raw mut ga, tv, c":echo argument".as_ptr());
        debug_assert!(eve_ret);
    }
    if !len.is_null() {
        *len = ga.ga_len as size_t;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn encode_tv2json(
    mut tv: *mut typval_T,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let evj_ret = encode_vim_to_json(
        &raw mut ga,
        tv,
        b"encode_tv2json() argument\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !evj_ret {
        ga_clear(&raw mut ga);
    }
    did_echo_string_emsg.set(false_0 != 0);
    if !len.is_null() {
        *len = ga.ga_len as size_t;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub const TYPVAL_ENCODE_ALLOW_SPECIALS_0: ::core::ffi::c_int = true_0;
pub unsafe extern "C" fn encode_init_lrstate(list: *const list_T) -> ListReaderState {
    return ListReaderState {
        list: list,
        li: tv_list_first(list),
        offset: 0 as size_t,
        li_length: if (*tv_list_first(list)).li_tv.vval.v_string.is_null() {
            0 as size_t
        } else {
            strlen((*tv_list_first(list)).li_tv.vval.v_string)
        },
    };
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
