use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::memory::{strequal, xcalloc, xfree, xrealloc};
use crate::src::nvim::os::libc::{abort, memcmp, memset};
pub use crate::src::nvim::types::{
    cstr_t, int16_t, int32_t, int64_t, mtnode_s, ptr_t, size_t, ssize_t, uint32_t, uint64_t,
    uint8_t, ColorItem, ColorKey, HlAttrs, HlEntry, HlKind, MHPutStatus, MTDamage, MTDamagePair,
    MTNode, MapHash, Map_ColorKey_ColorItem, Map_String_int, Map_cstr_t_int, Map_cstr_t_ptr_t,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_int_String, Map_int_ptr_t, Map_ptr_t_ptr_t,
    Map_uint32_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_MTDamagePair, Map_uint64_t_int,
    Map_uint64_t_ptr_t, RgbValue, Set_ColorKey, Set_HlEntry, Set_String, Set_cstr_t, Set_int,
    Set_int64_t, Set_ptr_t, Set_uint32_t, Set_uint64_t, String_0,
};
pub const kHlInvalid: HlKind = 7;
pub const kHlBlendThrough: HlKind = 6;
pub const kHlBlend: HlKind = 5;
pub const kHlCombine: HlKind = 4;
pub const kHlTerminal: HlKind = 3;
pub const kHlSyntax: HlKind = 2;
pub const kHlUI: HlKind = 1;
pub const kHlUnknown: HlKind = 0;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int_int {
    pub set: Set_int,
    pub values: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint64_t_ssize_t {
    pub set: Set_uint64_t,
    pub values: *mut ssize_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint64_t_uint64_t {
    pub set: Set_uint64_t,
    pub values: *mut uint64_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const MTDAMAGE_INIT: MTDamage = MTDamage {
    old: ::core::ptr::null_mut::<MTNode>(),
    new: ::core::ptr::null_mut::<MTNode>(),
    old_i: 0 as ::core::ffi::c_int,
    new_i: 0 as ::core::ffi::c_int,
};
pub const MTDAMAGE_PAIR_INIT: MTDamagePair = MTDamagePair {
    start: MTDAMAGE_INIT,
    end: MTDAMAGE_INIT,
};
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
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
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
static value_init_ssize_t: GlobalCell<ssize_t> = GlobalCell::new(-1 as ssize_t);
static value_init_uint32_t: GlobalCell<uint32_t> = GlobalCell::new(0 as uint32_t);
static value_init_uint64_t: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
static value_init_int64_t: GlobalCell<int64_t> = GlobalCell::new(0 as int64_t);
static value_init_String: GlobalCell<String_0> = GlobalCell::new(STRING_INIT);
static value_init_ColorItem: GlobalCell<ColorItem> = GlobalCell::new(COLOR_ITEM_INITIALIZER);
static value_init_MTDamagePair: GlobalCell<MTDamagePair> = GlobalCell::new(MTDAMAGE_PAIR_INIT);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
pub const COLOR_ITEM_INITIALIZER: ColorItem = ColorItem {
    attr_id: -1 as ::core::ffi::c_int,
    link_id: -1 as ::core::ffi::c_int,
    version: -1 as ::core::ffi::c_int,
    is_default: false_0 != 0,
    link_global: false_0 != 0,
};
#[inline]
unsafe extern "C" fn hash_cstr_t(mut s: *const ::core::ffi::c_char) -> uint32_t {
    let mut h: uint32_t = 0 as uint32_t;
    let mut i: size_t = 0 as size_t;
    while *s.offset(i as isize) != 0 {
        h = (h << 5 as ::core::ffi::c_int)
            .wrapping_sub(h)
            .wrapping_add(*s.offset(i as isize) as uint8_t as uint32_t);
        i = i.wrapping_add(1);
    }
    return h;
}
#[inline]
unsafe extern "C" fn hash_HlEntry(mut ae: HlEntry) -> uint32_t {
    let mut data: *const uint8_t = &raw mut ae as *const uint8_t;
    let mut h: uint32_t = 0 as uint32_t;
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<HlEntry>() {
        h = (h << 5 as ::core::ffi::c_int)
            .wrapping_sub(h)
            .wrapping_add(*data.offset(i as isize) as uint32_t);
        i = i.wrapping_add(1);
    }
    return h;
}
#[inline]
unsafe extern "C" fn equal_HlEntry(mut ae1: HlEntry, mut ae2: HlEntry) -> bool {
    return memcmp(
        &raw mut ae1 as *const ::core::ffi::c_void,
        &raw mut ae2 as *const ::core::ffi::c_void,
        ::core::mem::size_of::<HlEntry>(),
    ) == 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn hash_ColorKey(mut ae: ColorKey) -> uint32_t {
    let mut data: *const uint8_t = &raw mut ae as *const uint8_t;
    let mut h: uint32_t = 0 as uint32_t;
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<ColorKey>() {
        h = (h << 5 as ::core::ffi::c_int)
            .wrapping_sub(h)
            .wrapping_add(*data.offset(i as isize) as uint32_t);
        i = i.wrapping_add(1);
    }
    return h;
}
#[inline]
unsafe extern "C" fn equal_ColorKey(mut ae1: ColorKey, mut ae2: ColorKey) -> bool {
    return memcmp(
        &raw mut ae1 as *const ::core::ffi::c_void,
        &raw mut ae2 as *const ::core::ffi::c_void,
        ::core::mem::size_of::<ColorKey>(),
    ) == 0 as ::core::ffi::c_int;
}
pub const UPPER_FILL: ::core::ffi::c_double = 0.77f64;
pub unsafe extern "C" fn mh_realloc(mut h: *mut MapHash, mut n_min_buckets: uint32_t) {
    xfree((*h).hash as *mut ::core::ffi::c_void);
    let mut n_buckets: uint32_t = if n_min_buckets < 16 as uint32_t {
        16 as uint32_t
    } else {
        n_min_buckets
    };
    n_buckets = n_buckets.wrapping_sub(1);
    n_buckets |= n_buckets >> 1 as ::core::ffi::c_int;
    n_buckets |= n_buckets >> 2 as ::core::ffi::c_int;
    n_buckets |= n_buckets >> 4 as ::core::ffi::c_int;
    n_buckets |= n_buckets >> 8 as ::core::ffi::c_int;
    n_buckets |= n_buckets >> 16 as ::core::ffi::c_int;
    n_buckets = n_buckets.wrapping_add(1);
    (*h).hash = xcalloc(n_buckets as size_t, ::core::mem::size_of::<uint32_t>()) as *mut uint32_t;
    (*h).size = 0 as uint32_t;
    (*h).n_occupied = (*h).size;
    (*h).n_buckets = n_buckets;
    (*h).upper_bound = ((*h).n_buckets as ::core::ffi::c_double * UPPER_FILL + 0.5f64) as uint32_t;
}
pub unsafe extern "C" fn mh_clear(mut h: *mut MapHash) {
    if !(*h).hash.is_null() {
        memset(
            (*h).hash as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
        );
        (*h).n_occupied = 0 as uint32_t;
        (*h).size = (*h).n_occupied;
        (*h).n_keys = 0 as uint32_t;
    }
}
pub unsafe extern "C" fn mh_find_bucket_int(
    mut set: *mut Set_int,
    mut key: ::core::ffi::c_int,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = key as uint32_t;
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
        } else if *(*set)
            .keys
            .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize)
            == key
        {
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
pub unsafe extern "C" fn mh_get_int(
    mut set: *mut Set_int,
    mut key: ::core::ffi::c_int,
) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_int(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_int(mut set: *mut Set_int) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_int(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_int(
    mut set: *mut Set_int,
    mut key: ::core::ffi::c_int,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_int(set);
    }
    let mut idx: uint32_t = mh_find_bucket_int(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh0 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh0;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
            ) as *mut ::core::ffi::c_int;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !(*(*set).keys.offset(pos_0 as isize) == key) {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_delete_int(
    mut set: *mut Set_int,
    mut key: *mut ::core::ffi::c_int,
) -> uint32_t {
    if (*set).h.size == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_int(set, *key, false_0 != 0);
    if idx != MH_TOMBSTONE as uint32_t {
        let mut k: uint32_t = (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        *(*set).h.hash.offset(idx as isize) = MH_TOMBSTONE as uint32_t;
        (*set).h.n_keys = (*set).h.n_keys.wrapping_sub(1);
        let mut last: uint32_t = (*set).h.n_keys;
        *key = *(*set).keys.offset(k as isize);
        (*set).h.size = (*set).h.size.wrapping_sub(1);
        if last != k {
            let mut idx2: uint32_t =
                mh_find_bucket_int(set, *(*set).keys.offset(last as isize), false_0 != 0);
            if *(*set).h.hash.offset(idx2 as isize) != last.wrapping_add(1 as uint32_t) {
                abort();
            }
            *(*set).h.hash.offset(idx2 as isize) = k.wrapping_add(1 as uint32_t);
            *(*set).keys.offset(k as isize) = *(*set).keys.offset(last as isize);
        }
        return k;
    }
    return MH_TOMBSTONE as uint32_t;
}
pub unsafe extern "C" fn mh_find_bucket_cstr_t(
    mut set: *mut Set_cstr_t,
    mut key: cstr_t,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = hash_cstr_t(key as *const ::core::ffi::c_char);
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
        } else if strequal(
            *(*set)
                .keys
                .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize)
                as *const ::core::ffi::c_char,
            key as *const ::core::ffi::c_char,
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
pub unsafe extern "C" fn mh_get_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_cstr_t(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_cstr_t(mut set: *mut Set_cstr_t) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_cstr_t(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_cstr_t(
    mut set: *mut Set_cstr_t,
    mut key: cstr_t,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_cstr_t(set);
    }
    let mut idx: uint32_t = mh_find_bucket_cstr_t(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh1 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh1;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<cstr_t>()),
            ) as *mut cstr_t;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !strequal(
            *(*set).keys.offset(pos_0 as isize) as *const ::core::ffi::c_char,
            key as *const ::core::ffi::c_char,
        ) {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_delete_cstr_t(
    mut set: *mut Set_cstr_t,
    mut key: *mut cstr_t,
) -> uint32_t {
    if (*set).h.size == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_cstr_t(set, *key, false_0 != 0);
    if idx != MH_TOMBSTONE as uint32_t {
        let mut k: uint32_t = (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        *(*set).h.hash.offset(idx as isize) = MH_TOMBSTONE as uint32_t;
        (*set).h.n_keys = (*set).h.n_keys.wrapping_sub(1);
        let mut last: uint32_t = (*set).h.n_keys;
        *key = *(*set).keys.offset(k as isize);
        (*set).h.size = (*set).h.size.wrapping_sub(1);
        if last != k {
            let mut idx2: uint32_t =
                mh_find_bucket_cstr_t(set, *(*set).keys.offset(last as isize), false_0 != 0);
            if *(*set).h.hash.offset(idx2 as isize) != last.wrapping_add(1 as uint32_t) {
                abort();
            }
            *(*set).h.hash.offset(idx2 as isize) = k.wrapping_add(1 as uint32_t);
            *(*set).keys.offset(k as isize) = *(*set).keys.offset(last as isize);
        }
        return k;
    }
    return MH_TOMBSTONE as uint32_t;
}
pub unsafe extern "C" fn mh_find_bucket_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = (key.expose_provenance() as uint64_t >> 33 as ::core::ffi::c_int
        ^ key.expose_provenance() as uint64_t
        ^ (key.expose_provenance() as uint64_t) << 11 as ::core::ffi::c_int)
        as uint32_t;
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
        } else if (*(*set)
            .keys
            .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize))
        .expose_provenance() as uint64_t
            == key.expose_provenance() as uint64_t
        {
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
pub unsafe extern "C" fn mh_get_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_ptr_t(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_ptr_t(mut set: *mut Set_ptr_t) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_ptr_t(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_ptr_t(set);
    }
    let mut idx: uint32_t = mh_find_bucket_ptr_t(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh2 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh2;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<ptr_t>()),
            ) as *mut ptr_t;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !((*(*set).keys.offset(pos_0 as isize)).expose_provenance() as uint64_t
            == key.expose_provenance() as uint64_t)
        {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_delete_ptr_t(mut set: *mut Set_ptr_t, mut key: *mut ptr_t) -> uint32_t {
    if (*set).h.size == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_ptr_t(set, *key, false_0 != 0);
    if idx != MH_TOMBSTONE as uint32_t {
        let mut k: uint32_t = (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        *(*set).h.hash.offset(idx as isize) = MH_TOMBSTONE as uint32_t;
        (*set).h.n_keys = (*set).h.n_keys.wrapping_sub(1);
        let mut last: uint32_t = (*set).h.n_keys;
        *key = *(*set).keys.offset(k as isize);
        (*set).h.size = (*set).h.size.wrapping_sub(1);
        if last != k {
            let mut idx2: uint32_t =
                mh_find_bucket_ptr_t(set, *(*set).keys.offset(last as isize), false_0 != 0);
            if *(*set).h.hash.offset(idx2 as isize) != last.wrapping_add(1 as uint32_t) {
                abort();
            }
            *(*set).h.hash.offset(idx2 as isize) = k.wrapping_add(1 as uint32_t);
            *(*set).keys.offset(k as isize) = *(*set).keys.offset(last as isize);
        }
        return k;
    }
    return MH_TOMBSTONE as uint32_t;
}
pub unsafe extern "C" fn mh_find_bucket_uint64_t(
    mut set: *mut Set_uint64_t,
    mut key: uint64_t,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t =
        (key >> 33 as ::core::ffi::c_int ^ key ^ key << 11 as ::core::ffi::c_int) as uint32_t;
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
        } else if *(*set)
            .keys
            .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize)
            == key
        {
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
pub unsafe extern "C" fn mh_get_uint64_t(
    mut set: *mut Set_uint64_t,
    mut key: uint64_t,
) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_uint64_t(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_uint64_t(mut set: *mut Set_uint64_t) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_uint64_t(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_uint64_t(
    mut set: *mut Set_uint64_t,
    mut key: uint64_t,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_uint64_t(set);
    }
    let mut idx: uint32_t = mh_find_bucket_uint64_t(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh3 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh3;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<uint64_t>()),
            ) as *mut uint64_t;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !(*(*set).keys.offset(pos_0 as isize) == key) {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_delete_uint64_t(
    mut set: *mut Set_uint64_t,
    mut key: *mut uint64_t,
) -> uint32_t {
    if (*set).h.size == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_uint64_t(set, *key, false_0 != 0);
    if idx != MH_TOMBSTONE as uint32_t {
        let mut k: uint32_t = (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        *(*set).h.hash.offset(idx as isize) = MH_TOMBSTONE as uint32_t;
        (*set).h.n_keys = (*set).h.n_keys.wrapping_sub(1);
        let mut last: uint32_t = (*set).h.n_keys;
        *key = *(*set).keys.offset(k as isize);
        (*set).h.size = (*set).h.size.wrapping_sub(1);
        if last != k {
            let mut idx2: uint32_t =
                mh_find_bucket_uint64_t(set, *(*set).keys.offset(last as isize), false_0 != 0);
            if *(*set).h.hash.offset(idx2 as isize) != last.wrapping_add(1 as uint32_t) {
                abort();
            }
            *(*set).h.hash.offset(idx2 as isize) = k.wrapping_add(1 as uint32_t);
            *(*set).keys.offset(k as isize) = *(*set).keys.offset(last as isize);
        }
        return k;
    }
    return MH_TOMBSTONE as uint32_t;
}
pub unsafe extern "C" fn mh_find_bucket_int64_t(
    mut set: *mut Set_int64_t,
    mut key: int64_t,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = (key as uint64_t >> 33 as ::core::ffi::c_int
        ^ key as uint64_t
        ^ (key as uint64_t) << 11 as ::core::ffi::c_int) as uint32_t;
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
        } else if *(*set)
            .keys
            .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize)
            == key
        {
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
pub unsafe extern "C" fn mh_get_int64_t(mut set: *mut Set_int64_t, mut key: int64_t) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_int64_t(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_int64_t(mut set: *mut Set_int64_t) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_int64_t(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_int64_t(
    mut set: *mut Set_int64_t,
    mut key: int64_t,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_int64_t(set);
    }
    let mut idx: uint32_t = mh_find_bucket_int64_t(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh4 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh4;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<int64_t>()),
            ) as *mut int64_t;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !(*(*set).keys.offset(pos_0 as isize) == key) {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_delete_int64_t(
    mut set: *mut Set_int64_t,
    mut key: *mut int64_t,
) -> uint32_t {
    if (*set).h.size == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_int64_t(set, *key, false_0 != 0);
    if idx != MH_TOMBSTONE as uint32_t {
        let mut k: uint32_t = (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        *(*set).h.hash.offset(idx as isize) = MH_TOMBSTONE as uint32_t;
        (*set).h.n_keys = (*set).h.n_keys.wrapping_sub(1);
        let mut last: uint32_t = (*set).h.n_keys;
        *key = *(*set).keys.offset(k as isize);
        (*set).h.size = (*set).h.size.wrapping_sub(1);
        if last != k {
            let mut idx2: uint32_t =
                mh_find_bucket_int64_t(set, *(*set).keys.offset(last as isize), false_0 != 0);
            if *(*set).h.hash.offset(idx2 as isize) != last.wrapping_add(1 as uint32_t) {
                abort();
            }
            *(*set).h.hash.offset(idx2 as isize) = k.wrapping_add(1 as uint32_t);
            *(*set).keys.offset(k as isize) = *(*set).keys.offset(last as isize);
        }
        return k;
    }
    return MH_TOMBSTONE as uint32_t;
}
pub unsafe extern "C" fn mh_find_bucket_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: uint32_t,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = key;
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
        } else if *(*set)
            .keys
            .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize)
            == key
        {
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
pub unsafe extern "C" fn mh_get_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: uint32_t,
) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_uint32_t(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_uint32_t(mut set: *mut Set_uint32_t) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_uint32_t(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: uint32_t,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_uint32_t(set);
    }
    let mut idx: uint32_t = mh_find_bucket_uint32_t(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh5 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh5;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            ) as *mut uint32_t;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !(*(*set).keys.offset(pos_0 as isize) == key) {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_delete_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: *mut uint32_t,
) -> uint32_t {
    if (*set).h.size == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_uint32_t(set, *key, false_0 != 0);
    if idx != MH_TOMBSTONE as uint32_t {
        let mut k: uint32_t = (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        *(*set).h.hash.offset(idx as isize) = MH_TOMBSTONE as uint32_t;
        (*set).h.n_keys = (*set).h.n_keys.wrapping_sub(1);
        let mut last: uint32_t = (*set).h.n_keys;
        *key = *(*set).keys.offset(k as isize);
        (*set).h.size = (*set).h.size.wrapping_sub(1);
        if last != k {
            let mut idx2: uint32_t =
                mh_find_bucket_uint32_t(set, *(*set).keys.offset(last as isize), false_0 != 0);
            if *(*set).h.hash.offset(idx2 as isize) != last.wrapping_add(1 as uint32_t) {
                abort();
            }
            *(*set).h.hash.offset(idx2 as isize) = k.wrapping_add(1 as uint32_t);
            *(*set).keys.offset(k as isize) = *(*set).keys.offset(last as isize);
        }
        return k;
    }
    return MH_TOMBSTONE as uint32_t;
}
pub unsafe extern "C" fn mh_find_bucket_String(
    mut set: *mut Set_String,
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
            *(*set)
                .keys
                .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize),
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
pub unsafe extern "C" fn mh_get_String(mut set: *mut Set_String, mut key: String_0) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_String(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_String(mut set: *mut Set_String) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_String(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_String(
    mut set: *mut Set_String,
    mut key: String_0,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_String(set);
    }
    let mut idx: uint32_t = mh_find_bucket_String(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh6 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh6;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<String_0>()),
            ) as *mut String_0;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !equal_String(*(*set).keys.offset(pos_0 as isize), key) {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_delete_String(
    mut set: *mut Set_String,
    mut key: *mut String_0,
) -> uint32_t {
    if (*set).h.size == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_String(set, *key, false_0 != 0);
    if idx != MH_TOMBSTONE as uint32_t {
        let mut k: uint32_t = (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        *(*set).h.hash.offset(idx as isize) = MH_TOMBSTONE as uint32_t;
        (*set).h.n_keys = (*set).h.n_keys.wrapping_sub(1);
        let mut last: uint32_t = (*set).h.n_keys;
        *key = *(*set).keys.offset(k as isize);
        (*set).h.size = (*set).h.size.wrapping_sub(1);
        if last != k {
            let mut idx2: uint32_t =
                mh_find_bucket_String(set, *(*set).keys.offset(last as isize), false_0 != 0);
            if *(*set).h.hash.offset(idx2 as isize) != last.wrapping_add(1 as uint32_t) {
                abort();
            }
            *(*set).h.hash.offset(idx2 as isize) = k.wrapping_add(1 as uint32_t);
            *(*set).keys.offset(k as isize) = *(*set).keys.offset(last as isize);
        }
        return k;
    }
    return MH_TOMBSTONE as uint32_t;
}
pub unsafe extern "C" fn mh_find_bucket_HlEntry(
    mut set: *mut Set_HlEntry,
    mut key: HlEntry,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = hash_HlEntry(key);
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
        } else if equal_HlEntry(
            *(*set)
                .keys
                .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize),
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
pub unsafe extern "C" fn mh_rehash_HlEntry(mut set: *mut Set_HlEntry) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_HlEntry(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_HlEntry(
    mut set: *mut Set_HlEntry,
    mut key: HlEntry,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_HlEntry(set);
    }
    let mut idx: uint32_t = mh_find_bucket_HlEntry(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh7 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh7;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<HlEntry>()),
            ) as *mut HlEntry;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !equal_HlEntry(*(*set).keys.offset(pos_0 as isize), key) {
            abort();
        }
        return pos_0;
    };
}
pub unsafe extern "C" fn mh_find_bucket_ColorKey(
    mut set: *mut Set_ColorKey,
    mut key: ColorKey,
    mut put: bool,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    let mut step: uint32_t = 0 as uint32_t;
    let mut mask: uint32_t = (*h).n_buckets.wrapping_sub(1 as uint32_t);
    let mut k: uint32_t = hash_ColorKey(key);
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
        } else if equal_ColorKey(
            *(*set)
                .keys
                .offset((*(*h).hash.offset(i as isize)).wrapping_sub(1 as uint32_t) as isize),
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
pub unsafe extern "C" fn mh_get_ColorKey(
    mut set: *mut Set_ColorKey,
    mut key: ColorKey,
) -> uint32_t {
    if (*set).h.n_buckets == 0 as uint32_t {
        return MH_TOMBSTONE as uint32_t;
    }
    let mut idx: uint32_t = mh_find_bucket_ColorKey(set, key, false_0 != 0);
    return if idx != MH_TOMBSTONE as uint32_t {
        (*(*set).h.hash.offset(idx as isize)).wrapping_sub(1 as uint32_t)
    } else {
        MH_TOMBSTONE as uint32_t
    };
}
pub unsafe extern "C" fn mh_rehash_ColorKey(mut set: *mut Set_ColorKey) {
    let mut k: uint32_t = 0 as uint32_t;
    while k < (*set).h.n_keys {
        let mut idx: uint32_t =
            mh_find_bucket_ColorKey(set, *(*set).keys.offset(k as isize), true_0 != 0);
        if !(*(*set).h.hash.offset(idx as isize) == 0 as uint32_t) {
            abort();
        }
        *(*set).h.hash.offset(idx as isize) = k.wrapping_add(1 as uint32_t);
        k = k.wrapping_add(1);
    }
    (*set).h.size = (*set).h.n_keys;
    (*set).h.n_occupied = (*set).h.size;
}
pub unsafe extern "C" fn mh_put_ColorKey(
    mut set: *mut Set_ColorKey,
    mut key: ColorKey,
    mut new: *mut MHPutStatus,
) -> uint32_t {
    let mut h: *mut MapHash = &raw mut (*set).h;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).size as ::core::ffi::c_double >= (*h).upper_bound as ::core::ffi::c_double * 0.9f64
        {
            mh_realloc(h, (*h).n_buckets.wrapping_add(1 as uint32_t));
        } else {
            memset(
                (*h).hash as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((*h).n_buckets as size_t).wrapping_mul(::core::mem::size_of::<uint32_t>()),
            );
            (*h).n_occupied = 0 as uint32_t;
            (*h).size = (*h).n_occupied;
        }
        mh_rehash_ColorKey(set);
    }
    let mut idx: uint32_t = mh_find_bucket_ColorKey(set, key, true_0 != 0);
    if (*(*h).hash.offset(idx as isize)).wrapping_add(1 as uint32_t) <= 1 as uint32_t {
        (*h).size = (*h).size.wrapping_add(1);
        if *(*h).hash.offset(idx as isize) == 0 as uint32_t {
            (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        }
        let c2rust_fresh8 = (*h).n_keys;
        (*h).n_keys = (*h).n_keys.wrapping_add(1);
        let mut pos: uint32_t = c2rust_fresh8;
        if pos >= (*h).keys_capacity {
            (*h).keys_capacity = if (*h).keys_capacity.wrapping_mul(2 as uint32_t) > 8 as uint32_t {
                (*h).keys_capacity.wrapping_mul(2 as uint32_t)
            } else {
                8 as uint32_t
            };
            (*set).keys = xrealloc(
                (*set).keys as *mut ::core::ffi::c_void,
                ((*h).keys_capacity as size_t).wrapping_mul(::core::mem::size_of::<ColorKey>()),
            ) as *mut ColorKey;
            *new = kMHNewKeyRealloc;
        } else {
            *new = kMHNewKeyDidFit;
        }
        *(*set).keys.offset(pos as isize) = key;
        *(*h).hash.offset(idx as isize) = pos.wrapping_add(1 as uint32_t);
        return pos;
    } else {
        *new = kMHExisting;
        let mut pos_0: uint32_t = (*(*h).hash.offset(idx as isize)).wrapping_sub(1 as uint32_t);
        if !equal_ColorKey(*(*set).keys.offset(pos_0 as isize), key) {
            abort();
        }
        return pos_0;
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub unsafe extern "C" fn map_put_ref_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut key_alloc: *mut *mut ::core::ffi::c_int,
    mut new_item: *mut bool,
) -> *mut ptr_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_int(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<ptr_t>()),
            ) as *mut ptr_t;
        }
        *(*map).values.offset(k as isize) = value_init_ptr_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut key_alloc: *mut ::core::ffi::c_int,
) -> ptr_t {
    let mut rv: ptr_t = value_init_ptr_t.get();
    let mut k: uint32_t = mh_delete_int(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_ref_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
    mut key_alloc: *mut *mut cstr_t,
) -> *mut ptr_t {
    let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
    if k == MH_TOMBSTONE as uint32_t {
        return ::core::ptr::null_mut::<ptr_t>();
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_put_ref_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
    mut key_alloc: *mut *mut cstr_t,
    mut new_item: *mut bool,
) -> *mut ptr_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_cstr_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<ptr_t>()),
            ) as *mut ptr_t;
        }
        *(*map).values.offset(k as isize) = value_init_ptr_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
    mut key_alloc: *mut cstr_t,
) -> ptr_t {
    let mut rv: ptr_t = value_init_ptr_t.get();
    let mut k: uint32_t = mh_delete_cstr_t(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_put_ref_cstr_t_int(
    mut map: *mut Map_cstr_t_int,
    mut key: cstr_t,
    mut key_alloc: *mut *mut cstr_t,
    mut new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_cstr_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
            ) as *mut ::core::ffi::c_int;
        }
        *(*map).values.offset(k as isize) = value_init_int.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_put_ref_ptr_t_ptr_t(
    mut map: *mut Map_ptr_t_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
    mut new_item: *mut bool,
) -> *mut ptr_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ptr_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<ptr_t>()),
            ) as *mut ptr_t;
        }
        *(*map).values.offset(k as isize) = value_init_ptr_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_put_ref_uint32_t_ptr_t(
    mut map: *mut Map_uint32_t_ptr_t,
    mut key: uint32_t,
    mut key_alloc: *mut *mut uint32_t,
    mut new_item: *mut bool,
) -> *mut ptr_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_uint32_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<ptr_t>()),
            ) as *mut ptr_t;
        }
        *(*map).values.offset(k as isize) = value_init_ptr_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_uint32_t_ptr_t(
    mut map: *mut Map_uint32_t_ptr_t,
    mut key: uint32_t,
    mut key_alloc: *mut uint32_t,
) -> ptr_t {
    let mut rv: ptr_t = value_init_ptr_t.get();
    let mut k: uint32_t = mh_delete_uint32_t(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_put_ref_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut key_alloc: *mut *mut uint64_t,
    mut new_item: *mut bool,
) -> *mut ptr_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_uint64_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<ptr_t>()),
            ) as *mut ptr_t;
        }
        *(*map).values.offset(k as isize) = value_init_ptr_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut key_alloc: *mut uint64_t,
) -> ptr_t {
    let mut rv: ptr_t = value_init_ptr_t.get();
    let mut k: uint32_t = mh_delete_uint64_t(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_put_ref_uint64_t_int(
    mut map: *mut Map_uint64_t_int,
    mut key: uint64_t,
    mut key_alloc: *mut *mut uint64_t,
    mut new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_uint64_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
            ) as *mut ::core::ffi::c_int;
        }
        *(*map).values.offset(k as isize) = value_init_int.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_ref_int64_t_int64_t(
    mut map: *mut Map_int64_t_int64_t,
    mut key: int64_t,
    mut key_alloc: *mut *mut int64_t,
) -> *mut int64_t {
    let mut k: uint32_t = mh_get_int64_t(&raw mut (*map).set, key);
    if k == MH_TOMBSTONE as uint32_t {
        return ::core::ptr::null_mut::<int64_t>();
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_put_ref_int64_t_int64_t(
    mut map: *mut Map_int64_t_int64_t,
    mut key: int64_t,
    mut key_alloc: *mut *mut int64_t,
    mut new_item: *mut bool,
) -> *mut int64_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_int64_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<int64_t>()),
            ) as *mut int64_t;
        }
        *(*map).values.offset(k as isize) = value_init_int64_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_int64_t_int64_t(
    mut map: *mut Map_int64_t_int64_t,
    mut key: int64_t,
    mut key_alloc: *mut int64_t,
) -> int64_t {
    let mut rv: int64_t = value_init_int64_t.get();
    let mut k: uint32_t = mh_delete_int64_t(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_put_ref_int64_t_ptr_t(
    mut map: *mut Map_int64_t_ptr_t,
    mut key: int64_t,
    mut key_alloc: *mut *mut int64_t,
    mut new_item: *mut bool,
) -> *mut ptr_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_int64_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<ptr_t>()),
            ) as *mut ptr_t;
        }
        *(*map).values.offset(k as isize) = value_init_ptr_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_int64_t_ptr_t(
    mut map: *mut Map_int64_t_ptr_t,
    mut key: int64_t,
    mut key_alloc: *mut int64_t,
) -> ptr_t {
    let mut rv: ptr_t = value_init_ptr_t.get();
    let mut k: uint32_t = mh_delete_int64_t(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_ref_uint32_t_uint32_t(
    mut map: *mut Map_uint32_t_uint32_t,
    mut key: uint32_t,
    mut key_alloc: *mut *mut uint32_t,
) -> *mut uint32_t {
    let mut k: uint32_t = mh_get_uint32_t(&raw mut (*map).set, key);
    if k == MH_TOMBSTONE as uint32_t {
        return ::core::ptr::null_mut::<uint32_t>();
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_put_ref_uint32_t_uint32_t(
    mut map: *mut Map_uint32_t_uint32_t,
    mut key: uint32_t,
    mut key_alloc: *mut *mut uint32_t,
    mut new_item: *mut bool,
) -> *mut uint32_t {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_uint32_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<uint32_t>()),
            ) as *mut uint32_t;
        }
        *(*map).values.offset(k as isize) = value_init_uint32_t.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_uint32_t_uint32_t(
    mut map: *mut Map_uint32_t_uint32_t,
    mut key: uint32_t,
    mut key_alloc: *mut uint32_t,
) -> uint32_t {
    let mut rv: uint32_t = value_init_uint32_t.get();
    let mut k: uint32_t = mh_delete_uint32_t(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_ref_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut key_alloc: *mut *mut String_0,
) -> *mut ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
    if k == MH_TOMBSTONE as uint32_t {
        return ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_put_ref_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut key_alloc: *mut *mut String_0,
    mut new_item: *mut bool,
) -> *mut ::core::ffi::c_int {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_String(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
            ) as *mut ::core::ffi::c_int;
        }
        *(*map).values.offset(k as isize) = value_init_int.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut key_alloc: *mut String_0,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = value_init_int.get();
    let mut k: uint32_t = mh_delete_String(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_put_ref_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
    mut key_alloc: *mut *mut ::core::ffi::c_int,
    mut new_item: *mut bool,
) -> *mut String_0 {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_int(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<String_0>()),
            ) as *mut String_0;
        }
        *(*map).values.offset(k as isize) = value_init_String.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_del_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
    mut key_alloc: *mut ::core::ffi::c_int,
) -> String_0 {
    let mut rv: String_0 = value_init_String.get();
    let mut k: uint32_t = mh_delete_int(&raw mut (*map).set, &raw mut key);
    if k == MH_TOMBSTONE as uint32_t {
        return rv;
    }
    if !key_alloc.is_null() {
        *key_alloc = key;
    }
    rv = *(*map).values.offset(k as isize);
    if k != (*map).set.h.n_keys {
        *(*map).values.offset(k as isize) = *(*map).values.offset((*map).set.h.n_keys as isize);
    }
    return rv;
}
pub unsafe extern "C" fn map_put_ref_ColorKey_ColorItem(
    mut map: *mut Map_ColorKey_ColorItem,
    mut key: ColorKey,
    mut key_alloc: *mut *mut ColorKey,
    mut new_item: *mut bool,
) -> *mut ColorItem {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ColorKey(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<ColorItem>()),
            ) as *mut ColorItem;
        }
        *(*map).values.offset(k as isize) = value_init_ColorItem.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
pub unsafe extern "C" fn map_put_ref_uint64_t_MTDamagePair(
    mut map: *mut Map_uint64_t_MTDamagePair,
    mut key: uint64_t,
    mut key_alloc: *mut *mut uint64_t,
    mut new_item: *mut bool,
) -> *mut MTDamagePair {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_uint64_t(&raw mut (*map).set, key, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        if status as ::core::ffi::c_uint
            == kMHNewKeyRealloc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*map).values = xrealloc(
                (*map).values as *mut ::core::ffi::c_void,
                ((*map).set.h.keys_capacity as size_t)
                    .wrapping_mul(::core::mem::size_of::<MTDamagePair>()),
            ) as *mut MTDamagePair;
        }
        *(*map).values.offset(k as isize) = value_init_MTDamagePair.get();
    }
    if !new_item.is_null() {
        *new_item = status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
    if !key_alloc.is_null() {
        *key_alloc = (*map).set.keys.offset(k as isize);
    }
    return (*map).values.offset(k as isize);
}
