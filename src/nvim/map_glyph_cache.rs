use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::map::mh_realloc;
use crate::src::nvim::memory::xrealloc;
use crate::src::nvim::os::libc::{__assert_fail, abort, memcmp, memcpy, strlen};
pub use crate::src::nvim::types::{
    size_t, uint32_t, uint8_t, MHPutStatus, MapHash, Set_glyph, String_0,
};
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 58] = unsafe {
    ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
        *b"uint32_t mh_put_glyph(Set_glyph *, String, MHPutStatus *)\0",
    )
};
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub unsafe extern "C" fn mh_find_bucket_glyph(
    mut set: *mut Set_glyph,
    mut key: String_0,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = hash_String(key);
    let mut i: uint32_t = k & mask;
    let mut last: uint32_t = i;
    let mut site: uint32_t = if put as ::core::ffi::c_int != 0 {
        last
    } else {
        MH_TOMBSTONE as uint32_t
    };
    while *(*h).hash.offset(i as isize) != 0 as uint32_t {
        if *(*h).hash.offset(i as isize) == MH_TOMBSTONE as uint32_t {
            if site == last {
                site = i;
            }
        } else if equal_String(
            cstr_as_string(
                (*set)
                    .keys
                    .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize),
            ),
            key,
        ) {
            return i;
        }
        step = step.wrapping_add(1);
        i = i.wrapping_add(step) & mask;
        if i == last {
            abort();
        }
    }
    if site == last {
        site = i;
    }
    return site;
}
pub unsafe extern "C" fn mh_get_glyph(mut set: *mut Set_glyph, mut key: String_0) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_glyph(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_glyph(mut set: *mut Set_glyph) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t = mh_find_bucket_glyph(
            set,
            cstr_as_string((*set).keys.offset(k as isize)),
            true_0 != 0,
        );
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(
            (strlen((*set).keys.offset(k as isize)) as uint32_t).wrapping_add(1 as uint32_t),
        );
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_glyph(
    mut set: *mut Set_glyph,
    mut key: String_0,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        mh_rehash_glyph(set);
    }
    let mut idx: uint32_t = mh_find_bucket_glyph(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        let mut size: uint32_t = (key.size as uint32_t).wrapping_add(1 as uint32_t);
        let mut pos: uint32_t = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(size);
        if (*h).n_keys > (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 64 as uint32_t
            {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                64 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
            ) as *mut ::core::ffi::c_char;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        memcpy(
            (*set).keys.offset(pos as isize) as *mut ::core::ffi::c_void,
            key.data as *const ::core::ffi::c_void,
            key.size,
        );
        *(*set)
            .keys
            .offset((pos as size_t).wrapping_add(key.size) as isize) = NUL as ::core::ffi::c_char;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        '_c2rust_label: {
            if equal_String(cstr_as_string((*set).keys.offset(pos_0 as isize)), key) {
            } else {
                __assert_fail(
                    b"equal_String(cstr_as_string(&set->keys[pos]), key)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/map_glyph_cache.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    106 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        return pos_0;
    };
}
#[inline]
unsafe extern "C" fn hash_String(mut s: String_0) -> uint32_t {
    let mut h: uint32_t = 0 as uint32_t;
    let mut i: size_t = 0 as size_t;
    while i < s.size {
        h = (h << 5 as ::core::ffi::c_int)
            .wrapping_sub(h)
            .wrapping_add(*s.data.offset(i as isize) as uint8_t as uint32_t);
        i = i.wrapping_add(1);
    }
    return h;
}
#[inline]
unsafe extern "C" fn equal_String(mut a: String_0, mut b: String_0) -> bool {
    if a.size != b.size {
        return false_0 != 0;
    }
    return a.size == 0 as size_t
        || memcmp(
            a.data as *const ::core::ffi::c_void,
            b.data as *const ::core::ffi::c_void,
            a.size,
        ) == 0 as ::core::ffi::c_int;
}
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
