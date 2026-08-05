//! `json_decode_string()`: the JSON scanner.
//!
//! One pass over the input bytes, dispatching on the byte under the cursor.
//! Values are parsed by [`scan`] and handed to [`stack::json_decoder_pop`],
//! which is what actually attaches them to the container being built.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

mod scan;
mod stack;

use self::scan::*;
use self::stack::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn json_decode_string(
    buf: *const ::core::ffi::c_char,
    buf_len: size_t,
    rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = buf;
        let e: *const ::core::ffi::c_char = buf.offset(buf_len as isize);
        while p < e
            && (*p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == TAB
                || *p as ::core::ffi::c_int == NL
                || *p as ::core::ffi::c_int == CAR)
        {
            p = p.offset(1);
        }
        if p == e {
            emsg(gettext(
                b"E474: Attempt to decode a blank string\0".as_ptr() as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        let mut ret: ::core::ffi::c_int = OK;
        let mut stack: ValuesStack = ValuesStack {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<ValuesStackItem>(),
        };
        let mut container_stack: ContainerStack = ContainerStack {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<ContainerStackItem>(),
        };
        (*rettv).v_type = VAR_UNKNOWN;
        let mut didcomma: bool = false_0 != 0;
        let mut didcolon: bool = false_0 != 0;
        let mut next_map_special: bool = false_0 != 0;
        '_json_decode_string_ret: {
            '_json_decode_string_fail: {
                's_559: while p < e {
                    's_49: {
                        loop {
                            '_c2rust_label: {
                                if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                                    || next_map_special as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                {
                                } else {
                                    __assert_fail(
                                    b"*p == '{' || next_map_special == false\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/eval/decode.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    640 as ::core::ffi::c_uint,
                                    b"int json_decode_string(const char *const, const size_t, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                                }
                            };
                            match *p as ::core::ffi::c_int {
                                125 | 93 => {
                                    if container_stack.size == 0 as size_t {
                                        semsg(
                                            gettext(
                                                b"E474: No container to close: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        let mut last_container: ContainerStackItem =
                                            *container_stack.items.offset(
                                                container_stack
                                                    .size
                                                    .wrapping_sub(0 as size_t)
                                                    .wrapping_sub(1 as size_t)
                                                    as isize,
                                            );
                                        if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                                            && last_container.container.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_DICT as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            semsg(
                                                gettext(
                                                    b"E474: Closing list with curly bracket: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if *p as ::core::ffi::c_int
                                            == ']' as ::core::ffi::c_int
                                            && last_container.container.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                        {
                                            semsg(
                                            gettext(
                                                b"E474: Closing dictionary with square bracket: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                            break '_json_decode_string_fail;
                                        } else if didcomma {
                                            semsg(
                                                gettext(b"E474: Trailing comma: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if didcolon {
                                            semsg(
                                                gettext(
                                                    b"E474: Expected value after colon: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if last_container.stack_index
                                            != stack.size.wrapping_sub(1 as size_t)
                                        {
                                            '_c2rust_label_0: {
                                                if last_container.stack_index
                                                    < stack.size.wrapping_sub(1 as size_t)
                                                {
                                                } else {
                                                    __assert_fail(
                                                    b"last_container.stack_index < kv_size(stack) - 1\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/eval/decode.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    663 as ::core::ffi::c_uint,
                                                    b"int json_decode_string(const char *const, const size_t, typval_T *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                                }
                                            };
                                            semsg(
                                                gettext(b"E474: Expected value: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if stack.size == 1 as size_t {
                                            p = p.offset(1);
                                            container_stack.size =
                                                container_stack.size.wrapping_sub(1);
                                            break 's_559;
                                        } else {
                                            stack.size = stack.size.wrapping_sub(1);
                                            if json_decoder_pop(
                                                *stack.items.offset(stack.size as isize),
                                                &raw mut stack,
                                                &raw mut container_stack,
                                                &raw mut p,
                                                &raw mut next_map_special,
                                                &raw mut didcomma,
                                                &raw mut didcolon,
                                            ) == FAIL
                                            {
                                                break '_json_decode_string_fail;
                                            }
                                            '_c2rust_label_1: {
                                                if !next_map_special {
                                                } else {
                                                    __assert_fail(
                                                    b"!next_map_special\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"src/nvim/eval/decode.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    677 as ::core::ffi::c_uint,
                                                    b"int json_decode_string(const char *const, const size_t, typval_T *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                                }
                                            };
                                            break;
                                        }
                                    }
                                }
                                44 => {
                                    if container_stack.size == 0 as size_t {
                                        semsg(
                                            gettext(
                                                b"E474: Comma not inside container: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        let mut last_container_0: ContainerStackItem =
                                            *container_stack.items.offset(
                                                container_stack
                                                    .size
                                                    .wrapping_sub(0 as size_t)
                                                    .wrapping_sub(1 as size_t)
                                                    as isize,
                                            );
                                        if didcomma {
                                            semsg(
                                                gettext(b"E474: Duplicate comma: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if didcolon {
                                            semsg(
                                                gettext(
                                                    b"E474: Comma after colon: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if last_container_0.container.v_type
                                            as ::core::ffi::c_uint
                                            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                                            && last_container_0.stack_index
                                                != stack.size.wrapping_sub(1 as size_t)
                                        {
                                            semsg(
                                                gettext(
                                                    b"E474: Using comma in place of colon: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if if last_container_0.special_val.is_null() {
                                            if last_container_0.container.v_type
                                                as ::core::ffi::c_uint
                                                == VAR_DICT as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                                ((*last_container_0.container.vval.v_dict)
                                                    .dv_hashtab
                                                    .ht_used
                                                    == 0 as size_t)
                                                    as ::core::ffi::c_int
                                            } else {
                                                (tv_list_len(
                                                    last_container_0.container.vval.v_list,
                                                ) == 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            }
                                        } else {
                                            (tv_list_len(last_container_0.special_val)
                                                == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        } != 0
                                        {
                                            semsg(
                                                gettext(b"E474: Leading comma: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else {
                                            didcomma = true_0 != 0;
                                            break 's_49;
                                        }
                                    }
                                }
                                58 => {
                                    if container_stack.size == 0 as size_t {
                                        semsg(
                                            gettext(
                                                b"E474: Colon not inside container: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        let mut last_container_1: ContainerStackItem =
                                            *container_stack.items.offset(
                                                container_stack
                                                    .size
                                                    .wrapping_sub(0 as size_t)
                                                    .wrapping_sub(1 as size_t)
                                                    as isize,
                                            );
                                        if last_container_1.container.v_type as ::core::ffi::c_uint
                                            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            semsg(
                                                gettext(
                                                    b"E474: Using colon not in dictionary: %.*s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if last_container_1.stack_index
                                            != stack.size.wrapping_sub(2 as size_t)
                                        {
                                            semsg(
                                                gettext(b"E474: Unexpected colon: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if didcomma {
                                            semsg(
                                                gettext(
                                                    b"E474: Colon after comma: %.*s\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else if didcolon {
                                            semsg(
                                                gettext(b"E474: Duplicate colon: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                e.offset_from(p) as ::core::ffi::c_int,
                                                p,
                                            );
                                            break '_json_decode_string_fail;
                                        } else {
                                            didcolon = true_0 != 0;
                                            break 's_49;
                                        }
                                    }
                                }
                                32 | TAB | NL | CAR => {
                                    break 's_49;
                                }
                                110 => {
                                    if p.offset(3 as ::core::ffi::c_int as isize) >= e
                                        || strncmp(
                                            p.offset(1 as ::core::ffi::c_int as isize),
                                            b"ull\0".as_ptr() as *const ::core::ffi::c_char,
                                            3 as size_t,
                                        ) != 0 as ::core::ffi::c_int
                                    {
                                        semsg(
                                            gettext(b"E474: Expected null: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        p = p.offset(3 as ::core::ffi::c_int as isize);
                                        if json_decoder_pop(
                                            ValuesStackItem {
                                                is_special_string: false,
                                                didcomma: didcomma,
                                                didcolon: didcolon,
                                                val: typval_T {
                                                    v_type: VAR_SPECIAL,
                                                    v_lock: VAR_UNLOCKED,
                                                    vval: typval_vval_union {
                                                        v_special: kSpecialVarNull,
                                                    },
                                                },
                                            },
                                            &raw mut stack,
                                            &raw mut container_stack,
                                            &raw mut p,
                                            &raw mut next_map_special,
                                            &raw mut didcomma,
                                            &raw mut didcolon,
                                        ) == FAIL
                                        {
                                            break '_json_decode_string_fail;
                                        }
                                        if !next_map_special {
                                            break;
                                        }
                                    }
                                }
                                116 => {
                                    if p.offset(3 as ::core::ffi::c_int as isize) >= e
                                        || strncmp(
                                            p.offset(1 as ::core::ffi::c_int as isize),
                                            b"rue\0".as_ptr() as *const ::core::ffi::c_char,
                                            3 as size_t,
                                        ) != 0 as ::core::ffi::c_int
                                    {
                                        semsg(
                                            gettext(b"E474: Expected true: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        p = p.offset(3 as ::core::ffi::c_int as isize);
                                        if json_decoder_pop(
                                            ValuesStackItem {
                                                is_special_string: false,
                                                didcomma: didcomma,
                                                didcolon: didcolon,
                                                val: typval_T {
                                                    v_type: VAR_BOOL,
                                                    v_lock: VAR_UNLOCKED,
                                                    vval: typval_vval_union {
                                                        v_bool: kBoolVarTrue,
                                                    },
                                                },
                                            },
                                            &raw mut stack,
                                            &raw mut container_stack,
                                            &raw mut p,
                                            &raw mut next_map_special,
                                            &raw mut didcomma,
                                            &raw mut didcolon,
                                        ) == FAIL
                                        {
                                            break '_json_decode_string_fail;
                                        }
                                        if !next_map_special {
                                            break;
                                        }
                                    }
                                }
                                102 => {
                                    if p.offset(4 as ::core::ffi::c_int as isize) >= e
                                        || strncmp(
                                            p.offset(1 as ::core::ffi::c_int as isize),
                                            b"alse\0".as_ptr() as *const ::core::ffi::c_char,
                                            4 as size_t,
                                        ) != 0 as ::core::ffi::c_int
                                    {
                                        semsg(
                                            gettext(b"E474: Expected false: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        p = p.offset(4 as ::core::ffi::c_int as isize);
                                        if json_decoder_pop(
                                            ValuesStackItem {
                                                is_special_string: false,
                                                didcomma: didcomma,
                                                didcolon: didcolon,
                                                val: typval_T {
                                                    v_type: VAR_BOOL,
                                                    v_lock: VAR_UNLOCKED,
                                                    vval: typval_vval_union {
                                                        v_bool: kBoolVarFalse,
                                                    },
                                                },
                                            },
                                            &raw mut stack,
                                            &raw mut container_stack,
                                            &raw mut p,
                                            &raw mut next_map_special,
                                            &raw mut didcomma,
                                            &raw mut didcolon,
                                        ) == FAIL
                                        {
                                            break '_json_decode_string_fail;
                                        }
                                        if !next_map_special {
                                            break;
                                        }
                                    }
                                }
                                34 => {
                                    if parse_json_string(
                                        buf,
                                        buf_len,
                                        &raw mut p,
                                        &raw mut stack,
                                        &raw mut container_stack,
                                        &raw mut next_map_special,
                                        &raw mut didcomma,
                                        &raw mut didcolon,
                                    ) == FAIL
                                    {
                                        break '_json_decode_string_fail;
                                    } else if !next_map_special {
                                        break;
                                    }
                                }
                                45 | 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                                    if parse_json_number(
                                        buf,
                                        buf_len,
                                        &raw mut p,
                                        &raw mut stack,
                                        &raw mut container_stack,
                                        &raw mut next_map_special,
                                        &raw mut didcomma,
                                        &raw mut didcolon,
                                    ) == FAIL
                                    {
                                        break '_json_decode_string_fail;
                                    }
                                    if !next_map_special {
                                        break;
                                    }
                                }
                                91 => {
                                    let mut list: *mut list_T = tv_list_alloc(
                                        kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t,
                                    );
                                    tv_list_ref(list);
                                    let mut tv: typval_T = typval_T {
                                        v_type: VAR_LIST,
                                        v_lock: VAR_UNLOCKED,
                                        vval: typval_vval_union { v_list: list },
                                    };
                                    if container_stack.size == container_stack.capacity {
                                        container_stack.capacity = if container_stack.capacity != 0
                                        {
                                            container_stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                        container_stack.items = xrealloc(
                                            container_stack.items as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<ContainerStackItem>()
                                                .wrapping_mul(container_stack.capacity),
                                        )
                                            as *mut ContainerStackItem;
                                    } else {
                                    };
                                    let c2rust_fresh0 = container_stack.size;
                                    container_stack.size = container_stack.size.wrapping_add(1);
                                    *container_stack.items.offset(c2rust_fresh0 as isize) =
                                        ContainerStackItem {
                                            stack_index: stack.size,
                                            special_val: ::core::ptr::null_mut::<list_T>(),
                                            s: p,
                                            container: tv,
                                        };
                                    if stack.size == stack.capacity {
                                        stack.capacity = if stack.capacity != 0 {
                                            stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                        stack.items = xrealloc(
                                            stack.items as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<ValuesStackItem>()
                                                .wrapping_mul(stack.capacity),
                                        )
                                            as *mut ValuesStackItem;
                                    } else {
                                    };
                                    let c2rust_fresh1 = stack.size;
                                    stack.size = stack.size.wrapping_add(1);
                                    *stack.items.offset(c2rust_fresh1 as isize) = ValuesStackItem {
                                        is_special_string: false,
                                        didcomma: didcomma,
                                        didcolon: didcolon,
                                        val: tv,
                                    };
                                    break;
                                }
                                123 => {
                                    let mut tv_0: typval_T = typval_T {
                                        v_type: VAR_UNKNOWN,
                                        v_lock: VAR_UNLOCKED,
                                        vval: typval_vval_union { v_number: 0 },
                                    };
                                    let mut val_list: *mut list_T =
                                        ::core::ptr::null_mut::<list_T>();
                                    if next_map_special {
                                        next_map_special = false_0 != 0;
                                        val_list = decode_create_map_special_dict(
                                            &raw mut tv_0,
                                            kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t,
                                        );
                                    } else {
                                        let mut dict: *mut dict_T = tv_dict_alloc();
                                        (*dict).dv_refcount += 1;
                                        tv_0 = typval_T {
                                            v_type: VAR_DICT,
                                            v_lock: VAR_UNLOCKED,
                                            vval: typval_vval_union { v_dict: dict },
                                        };
                                    }
                                    if container_stack.size == container_stack.capacity {
                                        container_stack.capacity = if container_stack.capacity != 0
                                        {
                                            container_stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                        container_stack.items = xrealloc(
                                            container_stack.items as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<ContainerStackItem>()
                                                .wrapping_mul(container_stack.capacity),
                                        )
                                            as *mut ContainerStackItem;
                                    } else {
                                    };
                                    let c2rust_fresh2 = container_stack.size;
                                    container_stack.size = container_stack.size.wrapping_add(1);
                                    *container_stack.items.offset(c2rust_fresh2 as isize) =
                                        ContainerStackItem {
                                            stack_index: stack.size,
                                            special_val: val_list,
                                            s: p,
                                            container: tv_0,
                                        };
                                    if stack.size == stack.capacity {
                                        stack.capacity = if stack.capacity != 0 {
                                            stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                        stack.items = xrealloc(
                                            stack.items as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<ValuesStackItem>()
                                                .wrapping_mul(stack.capacity),
                                        )
                                            as *mut ValuesStackItem;
                                    } else {
                                    };
                                    let c2rust_fresh3 = stack.size;
                                    stack.size = stack.size.wrapping_add(1);
                                    *stack.items.offset(c2rust_fresh3 as isize) = ValuesStackItem {
                                        is_special_string: false,
                                        didcomma: didcomma,
                                        didcolon: didcolon,
                                        val: tv_0,
                                    };
                                    break;
                                }
                                _ => {
                                    semsg(
                                        gettext(b"E474: Unidentified byte: %.*s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        e.offset_from(p) as ::core::ffi::c_int,
                                        p,
                                    );
                                    break '_json_decode_string_fail;
                                }
                            }
                        }
                        didcomma = false_0 != 0;
                        didcolon = false_0 != 0;
                        if container_stack.size == 0 as size_t {
                            p = p.offset(1);
                            break 's_559;
                        }
                    }
                    p = p.offset(1);
                }
                while p < e {
                    match *p as ::core::ffi::c_int {
                        NL | 32 | TAB | CAR => {
                            p = p.offset(1);
                        }
                        _ => {
                            semsg(
                                gettext(b"E474: Trailing characters: %.*s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                e.offset_from(p) as ::core::ffi::c_int,
                                p,
                            );
                            break '_json_decode_string_fail;
                        }
                    }
                }
                if stack.size == 1 as size_t && container_stack.size == 0 as size_t {
                    stack.size = stack.size.wrapping_sub(1);
                    *rettv = (*stack.items.offset(stack.size as isize)).val;
                    break '_json_decode_string_ret;
                } else {
                    semsg(
                        gettext(b"E474: Unexpected end of input: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        buf_len as ::core::ffi::c_int,
                        buf,
                    );
                }
            }
            ret = FAIL;
            while stack.size != 0 {
                stack.size = stack.size.wrapping_sub(1);
                tv_clear(&raw mut (*stack.items.offset(stack.size as isize)).val);
            }
        }
        xfree(stack.items as *mut ::core::ffi::c_void);
        stack.capacity = 0 as size_t;
        stack.size = stack.capacity;
        stack.items = ::core::ptr::null_mut::<ValuesStackItem>();
        xfree(container_stack.items as *mut ::core::ffi::c_void);
        container_stack.capacity = 0 as size_t;
        container_stack.size = container_stack.capacity;
        container_stack.items = ::core::ptr::null_mut::<ContainerStackItem>();
        return ret;
    }
}
