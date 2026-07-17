extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn siemsg(s: *const ::core::ffi::c_char, ...);
}
pub type uint8_t = u8;
pub type size_t = usize;
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const HT_INIT_SIZE: C2Rust_Unnamed = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: size_t,
    pub ht_filled: size_t,
    pub ht_changed: ::core::ffi::c_int,
    pub ht_locked: ::core::ffi::c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; 16],
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"void hash_may_resize(hashtab_T *, size_t)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PERTURB_SHIFT: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
#[no_mangle]
pub static mut hash_removed: ::core::ffi::c_char = 0;
#[no_mangle]
pub unsafe extern "C" fn hash_init(mut ht: *mut hashtab_T) {
    memset(
        ht as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<hashtab_T>(),
    );
    (*ht).ht_array = &raw mut (*ht).ht_smallarray as *mut hashitem_T;
    (*ht).ht_mask = (HT_INIT_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as hash_T;
}
#[no_mangle]
pub unsafe extern "C" fn hash_clear(mut ht: *mut hashtab_T) {
    if (*ht).ht_array != &raw mut (*ht).ht_smallarray as *mut hashitem_T {
        xfree((*ht).ht_array as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn hash_clear_all(mut ht: *mut hashtab_T, mut off: ::core::ffi::c_uint) {
    let mut todo: size_t = (*ht).ht_used;
    let mut hi: *mut hashitem_T = (*ht).ht_array;
    while todo > 0 as size_t {
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
            xfree((*hi).hi_key.offset(-(off as isize)) as *mut ::core::ffi::c_void);
            todo = todo.wrapping_sub(1);
        }
        hi = hi.offset(1);
    }
    hash_clear(ht);
}
#[no_mangle]
pub unsafe extern "C" fn hash_find(
    ht: *const hashtab_T,
    key: *const ::core::ffi::c_char,
) -> *mut hashitem_T {
    return hash_lookup(ht, key, strlen(key), hash_hash(key));
}
#[no_mangle]
pub unsafe extern "C" fn hash_find_len(
    ht: *const hashtab_T,
    key: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut hashitem_T {
    return hash_lookup(ht, key, len, hash_hash_len(key, len));
}
#[no_mangle]
pub unsafe extern "C" fn hash_lookup(
    ht: *const hashtab_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    hash: hash_T,
) -> *mut hashitem_T {
    let mut idx: hash_T = hash & (*ht).ht_mask;
    let mut hi: *mut hashitem_T = (*ht).ht_array.offset(idx as isize);
    if (*hi).hi_key.is_null() {
        return hi;
    }
    let mut freeitem: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    if (*hi).hi_key == &raw mut hash_removed {
        freeitem = hi;
    } else if (*hi).hi_hash == hash
        && strncmp((*hi).hi_key, key, key_len) == 0 as ::core::ffi::c_int
        && *(*hi).hi_key.offset(key_len as isize) as ::core::ffi::c_int == NUL
    {
        return hi;
    }
    let mut perturb: hash_T = hash;
    loop {
        idx = (5 as hash_T)
            .wrapping_mul(idx)
            .wrapping_add(perturb)
            .wrapping_add(1 as hash_T);
        hi = (*ht).ht_array.offset((idx & (*ht).ht_mask) as isize);
        if (*hi).hi_key.is_null() {
            return if freeitem.is_null() { hi } else { freeitem };
        }
        if (*hi).hi_hash == hash
            && (*hi).hi_key != &raw mut hash_removed
            && strncmp((*hi).hi_key, key, key_len) == 0 as ::core::ffi::c_int
            && *(*hi).hi_key.offset(key_len as isize) as ::core::ffi::c_int == NUL
        {
            return hi;
        }
        if (*hi).hi_key == &raw mut hash_removed && freeitem.is_null() {
            freeitem = hi;
        }
        perturb >>= PERTURB_SHIFT;
    }
}
#[no_mangle]
pub unsafe extern "C" fn hash_debug_results() {}
#[no_mangle]
pub unsafe extern "C" fn hash_add(
    mut ht: *mut hashtab_T,
    mut key: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut hash: hash_T = hash_hash(key);
    let mut hi: *mut hashitem_T = hash_lookup(ht, key, strlen(key), hash);
    if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
        siemsg(
            gettext(
                b"E685: Internal error: hash_add(): duplicate key \"%s\"\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            key,
        );
        return FAIL;
    }
    hash_add_item(ht, hi, key, hash);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn hash_add_item(
    mut ht: *mut hashtab_T,
    mut hi: *mut hashitem_T,
    mut key: *mut ::core::ffi::c_char,
    mut hash: hash_T,
) {
    (*ht).ht_used = (*ht).ht_used.wrapping_add(1);
    (*ht).ht_changed += 1;
    if (*hi).hi_key.is_null() {
        (*ht).ht_filled = (*ht).ht_filled.wrapping_add(1);
    }
    (*hi).hi_key = key;
    (*hi).hi_hash = hash;
    hash_may_resize(ht, 0 as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn hash_remove(mut ht: *mut hashtab_T, mut hi: *mut hashitem_T) {
    (*ht).ht_used = (*ht).ht_used.wrapping_sub(1);
    (*ht).ht_changed += 1;
    (*hi).hi_key = &raw mut hash_removed;
    hash_may_resize(ht, 0 as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn hash_lock(mut ht: *mut hashtab_T) {
    (*ht).ht_locked += 1;
}
#[no_mangle]
pub unsafe extern "C" fn hash_unlock(mut ht: *mut hashtab_T) {
    (*ht).ht_locked -= 1;
    hash_may_resize(ht, 0 as size_t);
}
unsafe extern "C" fn hash_may_resize(mut ht: *mut hashtab_T, mut minitems: size_t) {
    if (*ht).ht_locked > 0 as ::core::ffi::c_int {
        return;
    }
    let mut minsize: size_t = 0;
    let oldsize: size_t = ((*ht).ht_mask as size_t).wrapping_add(1 as size_t);
    if minitems == 0 as size_t {
        if (*ht).ht_filled
            < (HT_INIT_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t
            && (*ht).ht_array == &raw mut (*ht).ht_smallarray as *mut hashitem_T
        {
            return;
        }
        if (*ht).ht_filled.wrapping_mul(3 as size_t) < oldsize.wrapping_mul(2 as size_t)
            && (*ht).ht_used > oldsize.wrapping_div(5 as size_t)
        {
            return;
        }
        if (*ht).ht_used > 1000 as size_t {
            minsize = (*ht).ht_used.wrapping_mul(2 as size_t);
        } else {
            minsize = (*ht).ht_used.wrapping_mul(4 as size_t);
        }
    } else {
        minitems = if minitems > (*ht).ht_used {
            minitems
        } else {
            (*ht).ht_used
        };
        minsize = minitems
            .wrapping_mul(3 as size_t)
            .wrapping_add(1 as size_t)
            .wrapping_div(2 as size_t);
    }
    let mut newsize: size_t = HT_INIT_SIZE as ::core::ffi::c_int as size_t;
    while newsize < minsize {
        newsize <<= 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if newsize != 0 as size_t {
            } else {
                __assert_fail(
                    b"newsize != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/hashtab.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    328 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
    }
    let mut newarray_is_small: bool = newsize == HT_INIT_SIZE as ::core::ffi::c_int as size_t;
    if !newarray_is_small
        && newsize == oldsize
        && (*ht).ht_filled.wrapping_mul(3 as size_t) < oldsize.wrapping_mul(2 as size_t)
    {
        return;
    }
    let mut keep_smallarray: bool = newarray_is_small as ::core::ffi::c_int != 0
        && (*ht).ht_array == &raw mut (*ht).ht_smallarray as *mut hashitem_T;
    let mut temparray: [hashitem_T; 16] = [hashitem_T {
        hi_hash: 0,
        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    }; 16];
    let mut oldarray: *mut hashitem_T = (if keep_smallarray as ::core::ffi::c_int != 0 {
        memcpy(
            &raw mut temparray as *mut hashitem_T as *mut ::core::ffi::c_void,
            &raw mut (*ht).ht_smallarray as *mut hashitem_T as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[hashitem_T; 16]>(),
        )
    } else {
        (*ht).ht_array as *mut ::core::ffi::c_void
    }) as *mut hashitem_T;
    if newarray_is_small {
        memset(
            &raw mut (*ht).ht_smallarray as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[hashitem_T; 16]>(),
        );
    }
    let mut newarray: *mut hashitem_T = (if newarray_is_small as ::core::ffi::c_int != 0 {
        &raw mut (*ht).ht_smallarray as *mut hashitem_T as *mut ::core::ffi::c_void
    } else {
        xcalloc(newsize, ::core::mem::size_of::<hashitem_T>())
    }) as *mut hashitem_T;
    let mut newmask: hash_T = (newsize as hash_T).wrapping_sub(1 as hash_T);
    let mut todo: size_t = (*ht).ht_used;
    let mut olditem: *mut hashitem_T = oldarray;
    while todo > 0 as size_t {
        if !((*olditem).hi_key.is_null() || (*olditem).hi_key == &raw mut hash_removed) {
            let mut newi: hash_T = (*olditem).hi_hash & newmask;
            let mut newitem: *mut hashitem_T = newarray.offset(newi as isize);
            if !(*newitem).hi_key.is_null() {
                let mut perturb: hash_T = (*olditem).hi_hash;
                loop {
                    newi = (5 as hash_T)
                        .wrapping_mul(newi)
                        .wrapping_add(perturb)
                        .wrapping_add(1 as hash_T);
                    newitem = newarray.offset((newi & newmask) as isize);
                    if (*newitem).hi_key.is_null() {
                        break;
                    }
                    perturb >>= PERTURB_SHIFT;
                }
            }
            *newitem = *olditem;
            todo = todo.wrapping_sub(1);
        }
        olditem = olditem.offset(1);
    }
    if (*ht).ht_array != &raw mut (*ht).ht_smallarray as *mut hashitem_T {
        xfree((*ht).ht_array as *mut ::core::ffi::c_void);
    }
    (*ht).ht_array = newarray;
    (*ht).ht_mask = newmask;
    (*ht).ht_filled = (*ht).ht_used;
    (*ht).ht_changed += 1;
}
#[no_mangle]
pub unsafe extern "C" fn hash_hash(mut key: *const ::core::ffi::c_char) -> hash_T {
    let mut hash: hash_T = *key as uint8_t as hash_T;
    if hash == 0 as hash_T {
        return 0 as hash_T;
    }
    let mut p: *const uint8_t = (key as *mut uint8_t).offset(1 as ::core::ffi::c_int as isize);
    while *p as ::core::ffi::c_int != NUL {
        let c2rust_fresh0 = p;
        p = p.offset(1);
        hash = hash
            .wrapping_mul(101 as hash_T)
            .wrapping_add(*c2rust_fresh0 as hash_T);
    }
    return hash;
}
#[no_mangle]
pub unsafe extern "C" fn hash_hash_len(mut key: *const ::core::ffi::c_char, len: size_t) -> hash_T {
    if len == 0 as size_t {
        return 0 as hash_T;
    }
    let mut hash: hash_T = *(key as *mut uint8_t) as hash_T;
    let mut end: *const uint8_t = (key as *mut uint8_t).offset(len as isize);
    let mut p: *const uint8_t = (key as *const uint8_t).offset(1 as ::core::ffi::c_int as isize);
    while p < end {
        let c2rust_fresh1 = p;
        p = p.offset(1);
        hash = hash
            .wrapping_mul(101 as hash_T)
            .wrapping_add(*c2rust_fresh1 as hash_T);
    }
    return hash;
}
#[no_mangle]
pub unsafe extern "C" fn _hash_key_removed() -> *const ::core::ffi::c_char {
    return &raw mut hash_removed;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
