use ::c2rust_bitfields;
extern "C" {
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
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xdl_fall_back_diff(
        diff_env: *mut xdfenv_t,
        xpp: *const xpparam_t,
        line1: ::core::ffi::c_int,
        count1: ::core::ffi::c_int,
        line2: ::core::ffi::c_int,
        count2: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn xdl_prepare_env(
        mf1: *mut mmfile_t,
        mf2: *mut mmfile_t,
        xpp: *const xpparam_t,
        xe: *mut xdfenv_t,
    ) -> ::core::ffi::c_int;
}
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_mmfile {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
pub type mmfile_t = s_mmfile;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xpparam {
    pub flags: ::core::ffi::c_ulong,
    pub anchors: *mut *mut ::core::ffi::c_char,
    pub anchors_nr: size_t,
}
pub type xpparam_t = s_xpparam;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_chanode {
    pub next: *mut s_chanode,
    pub icurr: ::core::ffi::c_long,
}
pub type chanode_t = s_chanode;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_chastore {
    pub head: *mut chanode_t,
    pub tail: *mut chanode_t,
    pub isize: ::core::ffi::c_long,
    pub nsize: ::core::ffi::c_long,
    pub ancur: *mut chanode_t,
    pub sncur: *mut chanode_t,
    pub scurr: ::core::ffi::c_long,
}
pub type chastore_t = s_chastore;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xrecord {
    pub next: *mut s_xrecord,
    pub ptr: *const ::core::ffi::c_char,
    pub size: ::core::ffi::c_long,
    pub ha: ::core::ffi::c_ulong,
}
pub type xrecord_t = s_xrecord;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdfile {
    pub rcha: chastore_t,
    pub nrec: ::core::ffi::c_long,
    pub hbits: ::core::ffi::c_uint,
    pub rhash: *mut *mut xrecord_t,
    pub dstart: ::core::ffi::c_long,
    pub dend: ::core::ffi::c_long,
    pub recs: *mut *mut xrecord_t,
    pub rchg: *mut ::core::ffi::c_char,
    pub rindex: *mut ::core::ffi::c_long,
    pub nreff: ::core::ffi::c_long,
    pub ha: *mut ::core::ffi::c_ulong,
}
pub type xdfile_t = s_xdfile;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdfenv {
    pub xdf1: xdfile_t,
    pub xdf2: xdfile_t,
}
pub type xdfenv_t = s_xdfenv;
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct entry {
    pub hash: ::core::ffi::c_ulong,
    pub line1: ::core::ffi::c_ulong,
    pub line2: ::core::ffi::c_ulong,
    pub next: *mut entry,
    pub previous: *mut entry,
    #[bitfield(name = "anchor", ty = "::core::ffi::c_uint", bits = "0..=0")]
    pub anchor: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 7],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashmap {
    pub nr: ::core::ffi::c_int,
    pub alloc: ::core::ffi::c_int,
    pub entries: *mut entry,
    pub first: *mut entry,
    pub last: *mut entry,
    pub has_matches: ::core::ffi::c_ulong,
    pub file1: *mut mmfile_t,
    pub file2: *mut mmfile_t,
    pub env: *mut xdfenv_t,
    pub xpp: *const xpparam_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const XDF_PATIENCE_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const XDF_HISTOGRAM_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
pub const XDF_DIFF_ALGORITHM_MASK: ::core::ffi::c_int = XDF_PATIENCE_DIFF | XDF_HISTOGRAM_DIFF;
pub const NON_UNIQUE: ::core::ffi::c_ulong = ULONG_MAX;
unsafe extern "C" fn is_anchor(
    mut xpp: *const xpparam_t,
    mut line: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < (*xpp).anchors_nr as ::core::ffi::c_int {
        if strncmp(
            line,
            *(*xpp).anchors.offset(i as isize),
            strlen(*(*xpp).anchors.offset(i as isize)),
        ) == 0
        {
            return 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn insert_record(
    mut xpp: *const xpparam_t,
    mut line: ::core::ffi::c_int,
    mut map: *mut hashmap,
    mut pass: ::core::ffi::c_int,
) {
    let mut records: *mut *mut xrecord_t = if pass == 1 as ::core::ffi::c_int {
        (*(*map).env).xdf1.recs
    } else {
        (*(*map).env).xdf2.recs
    };
    let mut record: *mut xrecord_t = *records.offset((line - 1 as ::core::ffi::c_int) as isize);
    let mut index: ::core::ffi::c_int = ((*record).ha << 1 as ::core::ffi::c_int)
        .wrapping_rem((*map).alloc as ::core::ffi::c_ulong)
        as ::core::ffi::c_int;
    while (*(*map).entries.offset(index as isize)).line1 != 0 {
        if (*(*map).entries.offset(index as isize)).hash != (*record).ha {
            index += 1;
            if index >= (*map).alloc {
                index = 0 as ::core::ffi::c_int;
            }
        } else {
            if pass == 2 as ::core::ffi::c_int {
                (*map).has_matches = 1 as ::core::ffi::c_ulong;
            }
            if pass == 1 as ::core::ffi::c_int
                || (*(*map).entries.offset(index as isize)).line2 != 0
            {
                (*(*map).entries.offset(index as isize)).line2 = NON_UNIQUE;
            } else {
                (*(*map).entries.offset(index as isize)).line2 = line as ::core::ffi::c_ulong;
            }
            return;
        }
    }
    if pass == 2 as ::core::ffi::c_int {
        return;
    }
    (*(*map).entries.offset(index as isize)).line1 = line as ::core::ffi::c_ulong;
    (*(*map).entries.offset(index as isize)).hash = (*record).ha;
    (*(*map).entries.offset(index as isize)).set_anchor(is_anchor(
        xpp,
        (**(*(*map).env)
            .xdf1
            .recs
            .offset((line - 1 as ::core::ffi::c_int) as isize))
        .ptr,
    ) as ::core::ffi::c_uint
        as ::core::ffi::c_uint);
    if (*map).first.is_null() {
        (*map).first = (*map).entries.offset(index as isize);
    }
    if !(*map).last.is_null() {
        (*(*map).last).next = (*map).entries.offset(index as isize) as *mut entry;
        (*(*map).entries.offset(index as isize)).previous = (*map).last as *mut entry;
    }
    (*map).last = (*map).entries.offset(index as isize);
    (*map).nr += 1;
}
unsafe extern "C" fn fill_hashmap(
    mut file1: *mut mmfile_t,
    mut file2: *mut mmfile_t,
    mut xpp: *const xpparam_t,
    mut env: *mut xdfenv_t,
    mut result: *mut hashmap,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (*result).file1 = file1;
    (*result).file2 = file2;
    (*result).xpp = xpp;
    (*result).env = env;
    (*result).alloc = count1 * 2 as ::core::ffi::c_int;
    (*result).entries =
        xmalloc(((*result).alloc as size_t).wrapping_mul(::core::mem::size_of::<entry>()))
            as *mut entry as *mut entry;
    if (*result).entries.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    memset(
        (*result).entries as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ((*result).alloc as size_t).wrapping_mul(::core::mem::size_of::<entry>()),
    );
    loop {
        let c2rust_fresh8 = count1;
        count1 = count1 - 1;
        if c2rust_fresh8 == 0 {
            break;
        }
        let c2rust_fresh9 = line1;
        line1 = line1 + 1;
        insert_record(xpp, c2rust_fresh9, result, 1 as ::core::ffi::c_int);
    }
    loop {
        let c2rust_fresh10 = count2;
        count2 = count2 - 1;
        if c2rust_fresh10 == 0 {
            break;
        }
        let c2rust_fresh11 = line2;
        line2 = line2 + 1;
        insert_record(xpp, c2rust_fresh11, result, 2 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn binary_search(
    mut sequence: *mut *mut entry,
    mut longest: ::core::ffi::c_int,
    mut entry: *mut entry,
) -> ::core::ffi::c_int {
    let mut left: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut right: ::core::ffi::c_int = longest;
    while (left + 1 as ::core::ffi::c_int) < right {
        let mut middle: ::core::ffi::c_int = left + (right - left) / 2 as ::core::ffi::c_int;
        if (**sequence.offset(middle as isize)).line2 > (*entry).line2 {
            right = middle;
        } else {
            left = middle;
        }
    }
    return left;
}
unsafe extern "C" fn find_longest_common_sequence(mut map: *mut hashmap) -> *mut entry {
    let mut sequence: *mut *mut entry =
        xmalloc(((*map).nr as size_t).wrapping_mul(::core::mem::size_of::<*mut entry>()))
            as *mut *mut entry;
    let mut longest: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    let mut entry: *mut entry = ::core::ptr::null_mut::<entry>();
    let mut anchor_i: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if sequence.is_null() {
        return (*map).first as *mut entry;
    }
    entry = (*map).first as *mut entry;
    while !entry.is_null() {
        if !((*entry).line2 == 0 || (*entry).line2 == NON_UNIQUE) {
            i = binary_search(sequence, longest, entry);
            (*entry).previous = if i < 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<entry>()
            } else {
                *sequence.offset(i as isize)
            };
            i += 1;
            if i > anchor_i {
                *sequence.offset(i as isize) = entry;
                if (*entry).anchor() != 0 {
                    anchor_i = i;
                    longest = anchor_i + 1 as ::core::ffi::c_int;
                } else if i == longest {
                    longest += 1;
                }
            }
        }
        entry = (*entry).next;
    }
    if longest == 0 {
        xfree(sequence as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<entry>();
    }
    entry = *sequence.offset((longest - 1 as ::core::ffi::c_int) as isize);
    (*entry).next = ::core::ptr::null_mut::<entry>();
    while !(*entry).previous.is_null() {
        (*(*entry).previous).next = entry;
        entry = (*entry).previous;
    }
    xfree(sequence as *mut ::core::ffi::c_void);
    return entry;
}
unsafe extern "C" fn match_0(
    mut map: *mut hashmap,
    mut line1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut record1: *mut xrecord_t = *(*(*map).env)
        .xdf1
        .recs
        .offset((line1 - 1 as ::core::ffi::c_int) as isize);
    let mut record2: *mut xrecord_t = *(*(*map).env)
        .xdf2
        .recs
        .offset((line2 - 1 as ::core::ffi::c_int) as isize);
    return ((*record1).ha == (*record2).ha) as ::core::ffi::c_int;
}
unsafe extern "C" fn walk_common_sequence(
    mut map: *mut hashmap,
    mut first: *mut entry,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut end1: ::core::ffi::c_int = line1 + count1;
    let mut end2: ::core::ffi::c_int = line2 + count2;
    let mut next1: ::core::ffi::c_int = 0;
    let mut next2: ::core::ffi::c_int = 0;
    loop {
        if !first.is_null() {
            next1 = (*first).line1 as ::core::ffi::c_int;
            next2 = (*first).line2 as ::core::ffi::c_int;
            while next1 > line1
                && next2 > line2
                && match_0(
                    map,
                    next1 - 1 as ::core::ffi::c_int,
                    next2 - 1 as ::core::ffi::c_int,
                ) != 0
            {
                next1 -= 1;
                next2 -= 1;
            }
        } else {
            next1 = end1;
            next2 = end2;
        }
        while line1 < next1 && line2 < next2 && match_0(map, line1, line2) != 0 {
            line1 += 1;
            line2 += 1;
        }
        if next1 > line1 || next2 > line2 {
            if patience_diff(
                (*map).file1,
                (*map).file2,
                (*map).xpp,
                (*map).env,
                line1,
                next1 - line1,
                line2,
                next2 - line2,
            ) != 0
            {
                return -1 as ::core::ffi::c_int;
            }
        }
        if first.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        while !(*first).next.is_null()
            && (*(*first).next).line1 == (*first).line1.wrapping_add(1 as ::core::ffi::c_ulong)
            && (*(*first).next).line2 == (*first).line2.wrapping_add(1 as ::core::ffi::c_ulong)
        {
            first = (*first).next;
        }
        line1 = (*first).line1.wrapping_add(1 as ::core::ffi::c_ulong) as ::core::ffi::c_int;
        line2 = (*first).line2.wrapping_add(1 as ::core::ffi::c_ulong) as ::core::ffi::c_int;
        first = (*first).next;
    }
}
unsafe extern "C" fn fall_back_to_classic_diff(
    mut map: *mut hashmap,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut xpp: xpparam_t = xpparam_t {
        flags: 0,
        anchors: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        anchors_nr: 0,
    };
    memset(
        &raw mut xpp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xpparam_t>(),
    );
    xpp.flags = (*(*map).xpp).flags & !XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong;
    return xdl_fall_back_diff((*map).env, &raw mut xpp, line1, count1, line2, count2);
}
unsafe extern "C" fn patience_diff(
    mut file1: *mut mmfile_t,
    mut file2: *mut mmfile_t,
    mut xpp: *const xpparam_t,
    mut env: *mut xdfenv_t,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut map: hashmap = hashmap {
        nr: 0,
        alloc: 0,
        entries: ::core::ptr::null_mut::<entry>(),
        first: ::core::ptr::null_mut::<entry>(),
        last: ::core::ptr::null_mut::<entry>(),
        has_matches: 0,
        file1: ::core::ptr::null_mut::<mmfile_t>(),
        file2: ::core::ptr::null_mut::<mmfile_t>(),
        env: ::core::ptr::null_mut::<xdfenv_t>(),
        xpp: ::core::ptr::null::<xpparam_t>(),
    };
    let mut first: *mut entry = ::core::ptr::null_mut::<entry>();
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if count1 == 0 {
        loop {
            let c2rust_fresh0 = count2;
            count2 = count2 - 1;
            if c2rust_fresh0 == 0 {
                break;
            }
            let c2rust_fresh1 = line2;
            line2 = line2 + 1;
            *(*env)
                .xdf2
                .rchg
                .offset((c2rust_fresh1 - 1 as ::core::ffi::c_int) as isize) =
                1 as ::core::ffi::c_char;
        }
        return 0 as ::core::ffi::c_int;
    } else if count2 == 0 {
        loop {
            let c2rust_fresh2 = count1;
            count1 = count1 - 1;
            if c2rust_fresh2 == 0 {
                break;
            }
            let c2rust_fresh3 = line1;
            line1 = line1 + 1;
            *(*env)
                .xdf1
                .rchg
                .offset((c2rust_fresh3 - 1 as ::core::ffi::c_int) as isize) =
                1 as ::core::ffi::c_char;
        }
        return 0 as ::core::ffi::c_int;
    }
    memset(
        &raw mut map as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<hashmap>(),
    );
    if fill_hashmap(
        file1,
        file2,
        xpp,
        env,
        &raw mut map,
        line1,
        count1,
        line2,
        count2,
    ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    if map.has_matches == 0 {
        loop {
            let c2rust_fresh4 = count1;
            count1 = count1 - 1;
            if c2rust_fresh4 == 0 {
                break;
            }
            let c2rust_fresh5 = line1;
            line1 = line1 + 1;
            *(*env)
                .xdf1
                .rchg
                .offset((c2rust_fresh5 - 1 as ::core::ffi::c_int) as isize) =
                1 as ::core::ffi::c_char;
        }
        loop {
            let c2rust_fresh6 = count2;
            count2 = count2 - 1;
            if c2rust_fresh6 == 0 {
                break;
            }
            let c2rust_fresh7 = line2;
            line2 = line2 + 1;
            *(*env)
                .xdf2
                .rchg
                .offset((c2rust_fresh7 - 1 as ::core::ffi::c_int) as isize) =
                1 as ::core::ffi::c_char;
        }
        xfree(map.entries as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    first = find_longest_common_sequence(&raw mut map);
    if !first.is_null() {
        result = walk_common_sequence(&raw mut map, first, line1, count1, line2, count2);
    } else {
        result = fall_back_to_classic_diff(&raw mut map, line1, count1, line2, count2);
    }
    xfree(map.entries as *mut ::core::ffi::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_do_patience_diff(
    mut file1: *mut mmfile_t,
    mut file2: *mut mmfile_t,
    mut xpp: *const xpparam_t,
    mut env: *mut xdfenv_t,
) -> ::core::ffi::c_int {
    if xdl_prepare_env(file1, file2, xpp, env) < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    return patience_diff(
        file1,
        file2,
        xpp,
        env,
        1 as ::core::ffi::c_int,
        (*env).xdf1.nrec as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        (*env).xdf2.nrec as ::core::ffi::c_int,
    );
}
pub const __LONG_MAX__: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const LONG_MAX: ::core::ffi::c_long = __LONG_MAX__;
pub const ULONG_MAX: ::core::ffi::c_ulong = (LONG_MAX as ::core::ffi::c_ulong)
    .wrapping_mul(2 as ::core::ffi::c_ulong)
    .wrapping_add(1 as ::core::ffi::c_ulong);
