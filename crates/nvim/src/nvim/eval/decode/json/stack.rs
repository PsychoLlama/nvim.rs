//! The decoder's two stacks, and the pop that joins a value to its container.
//!
//! `ValuesStack` holds values not yet stored anywhere; `ContainerStack` holds
//! the containers each next one is nested inside.  `json_decoder_pop` is where
//! a finished value meets the container above it — including the restart that
//! converts a plain dictionary into a special map.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerStackItem {
    pub stack_index: size_t,
    pub special_val: *mut list_T,
    pub s: *const ::core::ffi::c_char,
    pub container: typval_T,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ContainerStackItem,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ValuesStackItem {
    pub is_special_string: bool,
    pub didcomma: bool,
    pub didcolon: bool,
    pub val: typval_T,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ValuesStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ValuesStackItem,
}

#[inline]
pub(crate) unsafe extern "C" fn json_decoder_pop(
    mut obj: ValuesStackItem,
    stack: *mut ValuesStack,
    container_stack: *mut ContainerStack,
    pp: *mut *const ::core::ffi::c_char,
    next_map_special: *mut bool,
    didcomma: *mut bool,
    didcolon: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        if (*container_stack).size == 0 as size_t {
            if (*stack).size == (*stack).capacity {
                (*stack).capacity = if (*stack).capacity != 0 {
                    (*stack).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*stack).items = xrealloc(
                    (*stack).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<ValuesStackItem>().wrapping_mul((*stack).capacity),
                ) as *mut ValuesStackItem;
            } else {
            };
            let c2rust_fresh4 = (*stack).size;
            (*stack).size = (*stack).size.wrapping_add(1);
            *(*stack).items.offset(c2rust_fresh4 as isize) = obj;
            return OK;
        }
        let mut last_container: ContainerStackItem = *(*container_stack).items.offset(
            (*container_stack)
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
        let mut val_location: *const ::core::ffi::c_char = *pp;
        if obj.val.v_type as ::core::ffi::c_uint
            == last_container.container.v_type as ::core::ffi::c_uint
            && obj.val.vval.v_list as *mut ::core::ffi::c_void
                == last_container.container.vval.v_list as *mut ::core::ffi::c_void
        {
            (*container_stack).size = (*container_stack).size.wrapping_sub(1);
            val_location = last_container.s;
            last_container = *(*container_stack).items.offset(
                (*container_stack)
                    .size
                    .wrapping_sub(0 as size_t)
                    .wrapping_sub(1 as size_t) as isize,
            );
        }
        if last_container.container.v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_list_len(last_container.container.vval.v_list) != 0 as ::core::ffi::c_int
                && !obj.didcomma
            {
                semsg(
                    gettext(b"E474: Expected comma before list item: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    val_location,
                );
                tv_clear(&raw mut obj.val);
                return FAIL;
            }
            '_c2rust_label: {
                if last_container.special_val.is_null() {
                } else {
                    __assert_fail(
                    b"last_container.special_val == NULL\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/eval/decode.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    133 as ::core::ffi::c_uint,
                    b"int json_decoder_pop(ValuesStackItem, ValuesStack *const, ContainerStack *const, const char **const, _Bool *const, _Bool *const, _Bool *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                }
            };
            tv_list_append_owned_tv(last_container.container.vval.v_list, obj.val);
        } else if last_container.stack_index == (*stack).size.wrapping_sub(2 as size_t) {
            if !obj.didcolon {
                semsg(
                    gettext(
                        b"E474: Expected colon before dictionary value: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    val_location,
                );
                tv_clear(&raw mut obj.val);
                return FAIL;
            }
            (*stack).size = (*stack).size.wrapping_sub(1);
            let mut key: ValuesStackItem = *(*stack).items.offset((*stack).size as isize);
            if last_container.special_val.is_null() {
                '_c2rust_label_0: {
                    if !(key.is_special_string as ::core::ffi::c_int != 0
                        || key.val.vval.v_string.is_null())
                    {
                    } else {
                        __assert_fail(
                        b"!(key.is_special_string || key.val.vval.v_string == NULL)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/decode.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        145 as ::core::ffi::c_uint,
                        b"int json_decoder_pop(ValuesStackItem, ValuesStack *const, ContainerStack *const, const char **const, _Bool *const, _Bool *const, _Bool *const)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                let obj_di: *mut dictitem_T = tv_dict_item_alloc(key.val.vval.v_string);
                tv_clear(&raw mut key.val);
                if tv_dict_add(last_container.container.vval.v_dict, obj_di) == FAIL {
                    abort();
                }
                (*obj_di).di_tv = obj.val;
            } else {
                let kv_pair: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                tv_list_append_list(last_container.special_val, kv_pair);
                tv_list_append_owned_tv(kv_pair, key.val);
                tv_list_append_owned_tv(kv_pair, obj.val);
            }
        } else {
            if !obj.is_special_string
                && obj.val.v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                semsg(
                    gettext(
                        b"E474: Expected string key: %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    *pp,
                );
                tv_clear(&raw mut obj.val);
                return FAIL;
            } else if !obj.didcomma
                && (last_container.special_val.is_null()
                    && (*last_container.container.vval.v_dict).dv_hashtab.ht_used != 0 as size_t)
            {
                semsg(
                    gettext(b"E474: Expected comma before dictionary key: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    val_location,
                );
                tv_clear(&raw mut obj.val);
                return FAIL;
            }
            if last_container.special_val.is_null()
                && (obj.is_special_string as ::core::ffi::c_int != 0
                    || obj.val.vval.v_string.is_null()
                    || !tv_dict_find(
                        last_container.container.vval.v_dict,
                        obj.val.vval.v_string,
                        -1 as ptrdiff_t,
                    )
                    .is_null())
            {
                tv_clear(&raw mut obj.val);
                (*container_stack).size = (*container_stack).size.wrapping_sub(1);
                let mut last_container_val: ValuesStackItem =
                    *(*stack).items.offset(last_container.stack_index as isize);
                while (*stack).size > last_container.stack_index {
                    (*stack).size = (*stack).size.wrapping_sub(1);
                    tv_clear(&raw mut (*(*stack).items.offset((*stack).size as isize)).val);
                }
                *pp = last_container.s;
                *didcomma = last_container_val.didcomma;
                *didcolon = last_container_val.didcolon;
                *next_map_special = true_0 != 0;
                return OK;
            }
            if (*stack).size == (*stack).capacity {
                (*stack).capacity = if (*stack).capacity != 0 {
                    (*stack).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*stack).items = xrealloc(
                    (*stack).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<ValuesStackItem>().wrapping_mul((*stack).capacity),
                ) as *mut ValuesStackItem;
            } else {
            };
            let c2rust_fresh5 = (*stack).size;
            (*stack).size = (*stack).size.wrapping_add(1);
            *(*stack).items.offset(c2rust_fresh5 as isize) = obj;
        }
        return OK;
    }
}
