use crate::src::nvim::lua::executor::api_free_luaref;
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy};
pub use crate::src::nvim::types::{
    handle_T, int64_t, int8_t, key_value_pair, object, object_data as C2Rust_Unnamed,
    packer_buffer_t, ptrdiff_t, size_t, uint32_t, uint64_t, Array, Boolean, Dict, Float, Integer,
    KeyValuePair, LuaRef, Object, ObjectType, PackerBuffer, PackerBufferFlush, String_0,
};
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerStackItem {
    pub container: *mut Object,
    pub idx: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ContainerStackItem,
    pub init_array: [ContainerStackItem; 2],
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"void mpack_handle(ObjectType, handle_T, PackerBuffer *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KV_INITIAL_VALUE: C2Rust_Unnamed_0 = C2Rust_Unnamed_0 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<ContainerStackItem>(),
    init_array: [ContainerStackItem {
        container: ::core::ptr::null_mut::<Object>(),
        idx: 0,
    }; 2],
};
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
pub const MPACK_ITEM_SIZE: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mpack_w2(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh0 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh0 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh1 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh1 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_w4(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh2 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh2 = (v >> 24 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh3 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh3 = (v >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh4 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh4 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh5 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh5 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_uint(mut buf: *mut *mut ::core::ffi::c_char, mut val: uint32_t) {
    if val > 0xffff as uint32_t {
        let c2rust_fresh6 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh6 = 0xce as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, val);
    } else if val > 0xff as uint32_t {
        let c2rust_fresh7 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh7 = 0xcd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, val);
    } else if val > 0x7f as uint32_t {
        let c2rust_fresh8 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh8 = 0xcc as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh9 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh9 = val as ::core::ffi::c_char;
    } else {
        let c2rust_fresh10 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh10 = val as ::core::ffi::c_char;
    };
}
#[inline]
unsafe extern "C" fn mpack_bool(mut buf: *mut *mut ::core::ffi::c_char, mut val: bool) {
    let c2rust_fresh11 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh11 = (0xc2 as ::core::ffi::c_int
        | (if val as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_array(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh12 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh12 = (0x90 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh13 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh13 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh14 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh14 = 0xdd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
#[inline]
unsafe extern "C" fn mpack_map(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh15 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh15 = (0x80 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh16 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh16 = 0xde as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh17 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh17 = 0xdf as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
#[inline]
unsafe extern "C" fn mpack_remaining(mut packer: *mut PackerBuffer) -> size_t {
    return (*packer).endptr.offset_from((*packer).ptr) as size_t;
}
pub unsafe extern "C" fn mpack_check_buffer(mut packer: *mut PackerBuffer) {
    if mpack_remaining(packer) < (2 as ::core::ffi::c_int * MPACK_ITEM_SIZE) as size_t {
        (*packer).packer_flush.expect("non-null function pointer")(packer);
    }
}
unsafe extern "C" fn mpack_w8(
    mut b: *mut *mut ::core::ffi::c_char,
    mut data: *const ::core::ffi::c_char,
) {
    let mut i: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let c2rust_fresh19 = *b;
        *b = (*b).offset(1);
        *c2rust_fresh19 = *data.offset(i as isize);
        i -= 1;
    }
}
pub unsafe extern "C" fn mpack_uint64(mut ptr: *mut *mut ::core::ffi::c_char, mut i: uint64_t) {
    if i > 0xfffffff as uint64_t {
        let c2rust_fresh18 = *ptr;
        *ptr = (*ptr).offset(1);
        *c2rust_fresh18 = 0xcf as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w8(ptr, &raw mut i as *mut ::core::ffi::c_char);
    } else {
        mpack_uint(ptr, i as uint32_t);
    };
}
pub unsafe extern "C" fn mpack_integer(mut ptr: *mut *mut ::core::ffi::c_char, mut i: Integer) {
    if i >= 0 as Integer {
        mpack_uint64(ptr, i as uint64_t);
    } else if (i as ::core::ffi::c_longlong) < -0x80000000 as ::core::ffi::c_longlong {
        let c2rust_fresh20 = *ptr;
        *ptr = (*ptr).offset(1);
        *c2rust_fresh20 = 0xd3 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w8(ptr, &raw mut i as *mut ::core::ffi::c_char);
    } else if i < -0x8000 as Integer {
        let c2rust_fresh21 = *ptr;
        *ptr = (*ptr).offset(1);
        *c2rust_fresh21 = 0xd2 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(ptr, i as uint32_t);
    } else if i < -0x80 as Integer {
        let c2rust_fresh22 = *ptr;
        *ptr = (*ptr).offset(1);
        *c2rust_fresh22 = 0xd1 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(ptr, i as uint32_t);
    } else if i < -0x20 as Integer {
        let c2rust_fresh23 = *ptr;
        *ptr = (*ptr).offset(1);
        *c2rust_fresh23 = 0xd0 as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh24 = *ptr;
        *ptr = (*ptr).offset(1);
        *c2rust_fresh24 = i as ::core::ffi::c_char;
    } else {
        let c2rust_fresh25 = *ptr;
        *ptr = (*ptr).offset(1);
        *c2rust_fresh25 = i as ::core::ffi::c_char;
    };
}
pub unsafe extern "C" fn mpack_float8(
    mut ptr: *mut *mut ::core::ffi::c_char,
    mut i: ::core::ffi::c_double,
) {
    let c2rust_fresh26 = *ptr;
    *ptr = (*ptr).offset(1);
    *c2rust_fresh26 = 0xcb as ::core::ffi::c_int as ::core::ffi::c_char;
    mpack_w8(ptr, &raw mut i as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn mpack_str(mut str: String_0, mut packer: *mut PackerBuffer) {
    let len: size_t = str.size;
    if len < 32 as size_t {
        let c2rust_fresh27 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh27 = (0xa0 as size_t | len) as ::core::ffi::c_char;
    } else if len < 0xff as size_t {
        let c2rust_fresh28 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh28 = 0xd9 as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh29 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh29 = len as ::core::ffi::c_char;
    } else if len < 0xffff as size_t {
        let c2rust_fresh30 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh30 = 0xda as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(&raw mut (*packer).ptr, len as uint32_t);
    } else if len < 0xffffffff as ::core::ffi::c_uint as size_t {
        let c2rust_fresh31 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh31 = 0xdb as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(&raw mut (*packer).ptr, len as uint32_t);
    } else {
        abort();
    }
    mpack_raw(str.data, len, packer);
}
pub unsafe extern "C" fn mpack_bin(mut str: String_0, mut packer: *mut PackerBuffer) {
    let len: size_t = str.size;
    if len < 0xff as size_t {
        let c2rust_fresh32 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh32 = 0xc4 as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh33 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh33 = len as ::core::ffi::c_char;
    } else if len < 0xffff as size_t {
        let c2rust_fresh34 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh34 = 0xc5 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(&raw mut (*packer).ptr, len as uint32_t);
    } else if len < 0xffffffff as ::core::ffi::c_uint as size_t {
        let c2rust_fresh35 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh35 = 0xc6 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(&raw mut (*packer).ptr, len as uint32_t);
    } else {
        abort();
    }
    mpack_raw(str.data, len, packer);
}
pub unsafe extern "C" fn mpack_raw(
    mut data: *const ::core::ffi::c_char,
    mut len: size_t,
    mut packer: *mut PackerBuffer,
) {
    let mut pos: size_t = 0 as size_t;
    while pos < len {
        let mut remaining: ptrdiff_t = (*packer).endptr.offset_from((*packer).ptr);
        let mut to_copy: size_t = if len.wrapping_sub(pos) < remaining as size_t {
            len.wrapping_sub(pos)
        } else {
            remaining as size_t
        };
        memcpy(
            (*packer).ptr as *mut ::core::ffi::c_void,
            data.offset(pos as isize) as *const ::core::ffi::c_void,
            to_copy,
        );
        (*packer).ptr = (*packer).ptr.offset(to_copy as isize);
        pos = pos.wrapping_add(to_copy);
        if pos < len {
            (*packer).packer_flush.expect("non-null function pointer")(packer);
        }
    }
    mpack_check_buffer(packer);
}
pub unsafe extern "C" fn mpack_ext(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut type_0: int8_t,
    mut packer: *mut PackerBuffer,
) {
    if len == 1 as size_t {
        let c2rust_fresh36 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh36 = 0xd4 as ::core::ffi::c_int as ::core::ffi::c_char;
    } else if len == 2 as size_t {
        let c2rust_fresh37 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh37 = 0xd5 as ::core::ffi::c_int as ::core::ffi::c_char;
    } else if len <= 0xff as size_t {
        let c2rust_fresh38 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh38 = 0xc7 as ::core::ffi::c_int as ::core::ffi::c_char;
    } else if len < 0xffff as size_t {
        let c2rust_fresh39 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh39 = 0xc8 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(&raw mut (*packer).ptr, len as uint32_t);
    } else if len < 0xffffffff as ::core::ffi::c_uint as size_t {
        let c2rust_fresh40 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh40 = 0xc9 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(&raw mut (*packer).ptr, len as uint32_t);
    } else {
        abort();
    }
    let c2rust_fresh41 = (*packer).ptr;
    (*packer).ptr = (*packer).ptr.offset(1);
    *c2rust_fresh41 = type_0 as ::core::ffi::c_char;
    mpack_raw(buf, len, packer);
}
pub unsafe extern "C" fn mpack_handle(
    mut type_0: ObjectType,
    mut handle: handle_T,
    mut packer: *mut PackerBuffer,
) {
    let mut exttype: ::core::ffi::c_char = (type_0 as ::core::ffi::c_uint)
        .wrapping_sub(kObjectTypeBuffer as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_char;
    if -0x1f as ::core::ffi::c_int <= handle && handle <= 0x7f as ::core::ffi::c_int {
        let c2rust_fresh42 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh42 = 0xd4 as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh43 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh43 = exttype;
        let c2rust_fresh44 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh44 = handle as ::core::ffi::c_char;
    } else {
        '_c2rust_label: {
            if handle >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"handle >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/msgpack_rpc/packer.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    163 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        let mut buf: [::core::ffi::c_char; 9] = [0; 9];
        let mut pos: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
        mpack_uint(&raw mut pos, handle as uint32_t);
        let mut packsize: ptrdiff_t = pos.offset_from(&raw mut buf as *mut ::core::ffi::c_char);
        let c2rust_fresh45 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh45 = 0xc7 as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh46 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh46 = packsize as ::core::ffi::c_char;
        let c2rust_fresh47 = (*packer).ptr;
        (*packer).ptr = (*packer).ptr.offset(1);
        *c2rust_fresh47 = exttype;
        memcpy(
            (*packer).ptr as *mut ::core::ffi::c_void,
            &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            packsize as size_t,
        );
        (*packer).ptr = (*packer).ptr.offset(packsize as isize);
    };
}
pub unsafe extern "C" fn mpack_object(mut obj: *mut Object, mut packer: *mut PackerBuffer) {
    mpack_object_inner(obj, ::core::ptr::null_mut::<Object>(), 0 as size_t, packer);
}
pub unsafe extern "C" fn mpack_object_array(mut arr: Array, mut packer: *mut PackerBuffer) {
    mpack_array(&raw mut (*packer).ptr, arr.size as uint32_t);
    if arr.size > 0 as size_t {
        let mut container: Object = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: arr },
        };
        mpack_object_inner(
            arr.items.offset(0 as ::core::ffi::c_int as isize),
            if arr.size > 1 as size_t {
                &raw mut container
            } else {
                ::core::ptr::null_mut::<Object>()
            },
            1 as size_t,
            packer,
        );
    }
}
pub unsafe extern "C" fn mpack_object_inner(
    mut current: *mut Object,
    mut container: *mut Object,
    mut container_idx: size_t,
    mut packer: *mut PackerBuffer,
) {
    let mut stack: C2Rust_Unnamed_0 = KV_INITIAL_VALUE;
    stack.capacity = ::core::mem::size_of::<[ContainerStackItem; 2]>()
        .wrapping_div(::core::mem::size_of::<ContainerStackItem>())
        .wrapping_div(
            (::core::mem::size_of::<[ContainerStackItem; 2]>()
                .wrapping_rem(::core::mem::size_of::<ContainerStackItem>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    stack.size = 0 as size_t;
    stack.items = &raw mut stack.init_array as *mut ContainerStackItem;
    's_240: loop {
        mpack_check_buffer(packer);
        's_154: {
            match (*current).type_0 as ::core::ffi::c_uint {
                7 => {
                    api_free_luaref((*current).data.luaref);
                    (*current).data.luaref = LUA_NOREF as LuaRef;
                }
                0 => {}
                1 => {
                    mpack_bool(&raw mut (*packer).ptr, (*current).data.boolean as bool);
                    break 's_154;
                }
                2 => {
                    mpack_integer(&raw mut (*packer).ptr, (*current).data.integer);
                    break 's_154;
                }
                3 => {
                    mpack_float8(
                        &raw mut (*packer).ptr,
                        (*current).data.floating as ::core::ffi::c_double,
                    );
                    break 's_154;
                }
                4 => {
                    mpack_str((*current).data.string, packer);
                    break 's_154;
                }
                8 | 9 | 10 => {
                    mpack_handle(
                        (*current).type_0,
                        (*current).data.integer as handle_T,
                        packer,
                    );
                    break 's_154;
                }
                6 | 5 => {
                    let mut current_size: size_t = 0;
                    if (*current).type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        current_size = (*current).data.array.size;
                        mpack_array(&raw mut (*packer).ptr, current_size as uint32_t);
                    } else {
                        current_size = (*current).data.dict.size;
                        mpack_map(&raw mut (*packer).ptr, current_size as uint32_t);
                    }
                    if current_size > 0 as size_t {
                        if (*current).type_0 as ::core::ffi::c_uint
                            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                            && current_size == 1 as size_t
                        {
                            current = (*current)
                                .data
                                .array
                                .items
                                .offset(0 as ::core::ffi::c_int as isize);
                            continue 's_240;
                        } else {
                            if !container.is_null() {
                                if stack.size == stack.capacity {
                                    stack.capacity = if stack.capacity << 1 as ::core::ffi::c_int
                                        > ::core::mem::size_of::<[ContainerStackItem; 2]>()
                                            .wrapping_div(
                                                ::core::mem::size_of::<ContainerStackItem>(),
                                            )
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ContainerStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ContainerStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            ) {
                                        stack.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        ::core::mem::size_of::<[ContainerStackItem; 2]>()
                                            .wrapping_div(
                                                ::core::mem::size_of::<ContainerStackItem>(),
                                            )
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ContainerStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ContainerStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as size_t,
                                            )
                                    };
                                    stack.items = (if stack.capacity
                                        == ::core::mem::size_of::<[ContainerStackItem; 2]>()
                                            .wrapping_div(
                                                ::core::mem::size_of::<ContainerStackItem>(),
                                            )
                                            .wrapping_div(
                                                (::core::mem::size_of::<[ContainerStackItem; 2]>()
                                                    .wrapping_rem(::core::mem::size_of::<
                                                        ContainerStackItem,
                                                    >(
                                                    ))
                                                    == 0)
                                                    as ::core::ffi::c_int
                                                    as usize,
                                            ) {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut ContainerStackItem
                                        {
                                            stack.items as *mut ::core::ffi::c_void
                                        } else {
                                            _memcpy_free(
                                                &raw mut stack.init_array as *mut ContainerStackItem
                                                    as *mut ::core::ffi::c_void,
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    ContainerStackItem,
                                                >(
                                                )),
                                            )
                                        }
                                    } else {
                                        if stack.items
                                            == &raw mut stack.init_array as *mut ContainerStackItem
                                        {
                                            memcpy(
                                                xmalloc(stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<ContainerStackItem>(),
                                                )),
                                                stack.items as *const ::core::ffi::c_void,
                                                stack.size.wrapping_mul(::core::mem::size_of::<
                                                    ContainerStackItem,
                                                >(
                                                )),
                                            )
                                        } else {
                                            xrealloc(
                                                stack.items as *mut ::core::ffi::c_void,
                                                stack.capacity.wrapping_mul(
                                                    ::core::mem::size_of::<ContainerStackItem>(),
                                                ),
                                            )
                                        }
                                    })
                                        as *mut ContainerStackItem;
                                } else {
                                };
                                let c2rust_fresh49 = stack.size;
                                stack.size = stack.size.wrapping_add(1);
                                *stack.items.offset(c2rust_fresh49 as isize) = ContainerStackItem {
                                    container: container,
                                    idx: container_idx,
                                };
                            }
                            container = current;
                            container_idx = 0 as size_t;
                            break 's_154;
                        }
                    } else {
                        break 's_154;
                    }
                }
                _ => {
                    break 's_154;
                }
            }
            let c2rust_fresh48 = (*packer).ptr;
            (*packer).ptr = (*packer).ptr.offset(1);
            *c2rust_fresh48 = 0xc0 as ::core::ffi::c_int as ::core::ffi::c_char;
        }
        if container.is_null() {
            if stack.size == 0 {
                break;
            }
            stack.size = stack.size.wrapping_sub(1);
            let mut it: ContainerStackItem = *stack.items.offset(stack.size as isize);
            container = it.container;
            container_idx = it.idx;
        }
        if (*container).type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut arr: Array = (*container).data.array;
            let c2rust_fresh50 = container_idx;
            container_idx = container_idx.wrapping_add(1);
            current = arr.items.offset(c2rust_fresh50 as isize);
            if container_idx >= arr.size {
                container = ::core::ptr::null_mut::<Object>();
            }
        } else {
            let mut dict: Dict = (*container).data.dict;
            let c2rust_fresh51 = container_idx;
            container_idx = container_idx.wrapping_add(1);
            let mut it_0: *mut KeyValuePair = dict.items.offset(c2rust_fresh51 as isize);
            mpack_check_buffer(packer);
            mpack_str((*it_0).key, packer);
            current = &raw mut (*it_0).value;
            if container_idx >= dict.size {
                container = ::core::ptr::null_mut::<Object>();
            }
        }
    }
    if stack.items != &raw mut stack.init_array as *mut ContainerStackItem {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
}
pub unsafe extern "C" fn packer_string_buffer() -> PackerBuffer {
    let initial_size: size_t = 64 as size_t;
    let mut alloc: *mut ::core::ffi::c_char = xmalloc(initial_size) as *mut ::core::ffi::c_char;
    return packer_buffer_t {
        startptr: alloc,
        ptr: alloc,
        endptr: alloc.offset(initial_size as isize),
        anydata: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        anyint: 0,
        packer_flush: Some(flush_string_buffer as unsafe extern "C" fn(*mut PackerBuffer) -> ()),
    };
}
unsafe extern "C" fn flush_string_buffer(mut buffer: *mut PackerBuffer) {
    let mut current_capacity: size_t = (*buffer).endptr.offset_from((*buffer).startptr) as size_t;
    let mut new_capacity: size_t = (2 as size_t).wrapping_mul(current_capacity);
    let mut len: size_t = (*buffer).ptr.offset_from((*buffer).startptr) as size_t;
    (*buffer).startptr = xrealloc((*buffer).startptr as *mut ::core::ffi::c_void, new_capacity)
        as *mut ::core::ffi::c_char;
    (*buffer).ptr = (*buffer).startptr.offset(len as isize);
    (*buffer).endptr = (*buffer).startptr.offset(new_capacity as isize);
}
pub unsafe extern "C" fn packer_take_string(mut buffer: *mut PackerBuffer) -> String_0 {
    return String_0 {
        data: (*buffer).startptr,
        size: (*buffer).ptr.offset_from((*buffer).startptr) as size_t,
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
