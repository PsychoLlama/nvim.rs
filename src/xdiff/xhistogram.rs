use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::os::libc::memset;
pub use crate::src::nvim::types::{
    chanode_t, chastore_t, mmfile_t, s_chanode, s_chastore, s_mmfile, s_xdfenv, s_xdfile,
    s_xpparam, s_xrecord, size_t, xdfenv_t, xdfile_t, xpparam_t, xrecord_t,
};
use crate::src::xdiff::xprepare::xdl_prepare_env;
use crate::src::xdiff::xutils::{
    xdl_cha_alloc, xdl_cha_free, xdl_cha_init, xdl_fall_back_diff, xdl_hashbits, xdl_recmatch,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct region {
    pub begin1: ::core::ffi::c_uint,
    pub end1: ::core::ffi::c_uint,
    pub begin2: ::core::ffi::c_uint,
    pub end2: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct histindex {
    pub records: *mut *mut record,
    pub line_map: *mut *mut record,
    pub rcha: chastore_t,
    pub next_ptrs: *mut ::core::ffi::c_uint,
    pub table_bits: ::core::ffi::c_uint,
    pub records_size: ::core::ffi::c_uint,
    pub line_map_size: ::core::ffi::c_uint,
    pub max_chain_length: ::core::ffi::c_uint,
    pub key_shift: ::core::ffi::c_uint,
    pub ptr_shift: ::core::ffi::c_uint,
    pub cnt: ::core::ffi::c_uint,
    pub has_common: ::core::ffi::c_uint,
    pub env: *mut xdfenv_t,
    pub xpp: *const xpparam_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct record {
    pub ptr: ::core::ffi::c_uint,
    pub cnt: ::core::ffi::c_uint,
    pub next: *mut record,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const XDF_PATIENCE_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const XDF_HISTOGRAM_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
pub const XDF_DIFF_ALGORITHM_MASK: ::core::ffi::c_int = XDF_PATIENCE_DIFF | XDF_HISTOGRAM_DIFF;
pub const MAX_PTR: ::core::ffi::c_int = INT_MAX;
unsafe extern "C" fn cmp_recs(
    mut xpp: *const xpparam_t,
    mut r1: *mut xrecord_t,
    mut r2: *mut xrecord_t,
) -> ::core::ffi::c_int {
    return ((*r1).ha == (*r2).ha
        && xdl_recmatch(
            (*r1).ptr,
            (*r1).size,
            (*r2).ptr,
            (*r2).size,
            (*xpp).flags as ::core::ffi::c_long,
        ) != 0) as ::core::ffi::c_int;
}
unsafe extern "C" fn scanA(
    mut index: *mut histindex,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ptr: ::core::ffi::c_int = 0;
    let mut tbl_idx: ::core::ffi::c_int = 0;
    let mut chain_len: ::core::ffi::c_uint = 0;
    let mut rec_chain: *mut *mut record = ::core::ptr::null_mut::<*mut record>();
    let mut rec: *mut record = ::core::ptr::null_mut::<record>();
    ptr = line1 + count1 - 1 as ::core::ffi::c_int;
    while line1 <= ptr {
        tbl_idx = ((**(*(*index).env)
            .xdf1
            .recs
            .offset((ptr - 1 as ::core::ffi::c_int) as isize))
        .ha
        .wrapping_add(
            (**(*(*index).env)
                .xdf1
                .recs
                .offset((ptr - 1 as ::core::ffi::c_int) as isize))
            .ha >> (*index).table_bits,
        ) & ((1 as ::core::ffi::c_ulong) << (*index).table_bits)
            .wrapping_sub(1 as ::core::ffi::c_ulong)) as ::core::ffi::c_int;
        rec_chain = (*index).records.offset(tbl_idx as isize) as *mut *mut record;
        rec = *rec_chain;
        chain_len = 0 as ::core::ffi::c_uint;
        's_95: {
            while !rec.is_null() {
                if cmp_recs(
                    (*index).xpp,
                    *(*(*index).env)
                        .xdf1
                        .recs
                        .offset((*rec).ptr.wrapping_sub(1 as ::core::ffi::c_uint) as isize),
                    *(*(*index).env)
                        .xdf1
                        .recs
                        .offset((ptr - 1 as ::core::ffi::c_int) as isize),
                ) != 0
                {
                    *(*index).next_ptrs.offset(
                        (ptr as ::core::ffi::c_uint).wrapping_sub((*index).ptr_shift) as isize,
                    ) = (*rec).ptr;
                    (*rec).ptr = ptr as ::core::ffi::c_uint;
                    (*rec).cnt = if (2147483647 as ::core::ffi::c_int as ::core::ffi::c_uint)
                        < (*rec).cnt.wrapping_add(1 as ::core::ffi::c_uint)
                    {
                        2147483647 as ::core::ffi::c_int as ::core::ffi::c_uint
                    } else {
                        (*rec).cnt.wrapping_add(1 as ::core::ffi::c_uint)
                    };
                    *(*index).line_map.offset(
                        (ptr as ::core::ffi::c_uint).wrapping_sub((*index).ptr_shift) as isize,
                    ) = rec as *mut record;
                    break 's_95;
                } else {
                    rec = (*rec).next;
                    chain_len = chain_len.wrapping_add(1);
                }
            }
            if chain_len == (*index).max_chain_length {
                return -1 as ::core::ffi::c_int;
            }
            rec = xdl_cha_alloc(&raw mut (*index).rcha) as *mut record;
            if rec.is_null() {
                return -1 as ::core::ffi::c_int;
            }
            (*rec).ptr = ptr as ::core::ffi::c_uint;
            (*rec).cnt = 1 as ::core::ffi::c_uint;
            (*rec).next = *rec_chain;
            *rec_chain = rec;
            *(*index)
                .line_map
                .offset((ptr as ::core::ffi::c_uint).wrapping_sub((*index).ptr_shift) as isize) =
                rec as *mut record;
        }
        ptr -= 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn try_lcs(
    mut index: *mut histindex,
    mut lcs: *mut region,
    mut b_ptr: ::core::ffi::c_int,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut b_next: ::core::ffi::c_uint = (b_ptr + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint;
    let mut rec: *mut record = *(*index).records.offset(
        ((**(*(*index).env)
            .xdf2
            .recs
            .offset((b_ptr - 1 as ::core::ffi::c_int) as isize))
        .ha
        .wrapping_add(
            (**(*(*index).env)
                .xdf2
                .recs
                .offset((b_ptr - 1 as ::core::ffi::c_int) as isize))
            .ha >> (*index).table_bits,
        ) & ((1 as ::core::ffi::c_ulong) << (*index).table_bits)
            .wrapping_sub(1 as ::core::ffi::c_ulong)) as isize,
    ) as *mut record;
    let mut as_0: ::core::ffi::c_uint = 0;
    let mut ae: ::core::ffi::c_uint = 0;
    let mut bs: ::core::ffi::c_uint = 0;
    let mut be: ::core::ffi::c_uint = 0;
    let mut np: ::core::ffi::c_uint = 0;
    let mut rc: ::core::ffi::c_uint = 0;
    let mut should_break: ::core::ffi::c_int = 0;
    while !rec.is_null() {
        if (*rec).cnt > (*index).cnt {
            if (*index).has_common == 0 {
                (*index).has_common = cmp_recs(
                    (*index).xpp,
                    *(*(*index).env)
                        .xdf1
                        .recs
                        .offset((*rec).ptr.wrapping_sub(1 as ::core::ffi::c_uint) as isize),
                    *(*(*index).env)
                        .xdf2
                        .recs
                        .offset((b_ptr - 1 as ::core::ffi::c_int) as isize),
                ) as ::core::ffi::c_uint;
            }
        } else {
            as_0 = (*rec).ptr;
            if cmp_recs(
                (*index).xpp,
                *(*(*index).env)
                    .xdf1
                    .recs
                    .offset(as_0.wrapping_sub(1 as ::core::ffi::c_uint) as isize),
                *(*(*index).env)
                    .xdf2
                    .recs
                    .offset((b_ptr - 1 as ::core::ffi::c_int) as isize),
            ) != 0
            {
                (*index).has_common = 1 as ::core::ffi::c_uint;
                loop {
                    should_break = 0 as ::core::ffi::c_int;
                    np = *(*index)
                        .next_ptrs
                        .offset(as_0.wrapping_sub((*index).ptr_shift) as isize);
                    bs = b_ptr as ::core::ffi::c_uint;
                    ae = as_0;
                    be = bs;
                    rc = (*rec).cnt;
                    while line1 < as_0 as ::core::ffi::c_int
                        && line2 < bs as ::core::ffi::c_int
                        && cmp_recs(
                            (*index).xpp,
                            *(*(*index).env).xdf1.recs.offset(
                                as_0.wrapping_sub(1 as ::core::ffi::c_uint)
                                    .wrapping_sub(1 as ::core::ffi::c_uint)
                                    as isize,
                            ),
                            *(*(*index).env).xdf2.recs.offset(
                                bs.wrapping_sub(1 as ::core::ffi::c_uint)
                                    .wrapping_sub(1 as ::core::ffi::c_uint)
                                    as isize,
                            ),
                        ) != 0
                    {
                        as_0 = as_0.wrapping_sub(1);
                        bs = bs.wrapping_sub(1);
                        if (1 as ::core::ffi::c_uint) < rc {
                            rc = if rc
                                < (**(*index)
                                    .line_map
                                    .offset(as_0.wrapping_sub((*index).ptr_shift) as isize))
                                .cnt
                            {
                                rc
                            } else {
                                (**(*index)
                                    .line_map
                                    .offset(as_0.wrapping_sub((*index).ptr_shift) as isize))
                                .cnt
                            };
                        }
                    }
                    while (ae as ::core::ffi::c_int) < line1 + count1 - 1 as ::core::ffi::c_int
                        && (be as ::core::ffi::c_int) < line2 + count2 - 1 as ::core::ffi::c_int
                        && cmp_recs(
                            (*index).xpp,
                            *(*(*index).env).xdf1.recs.offset(
                                ae.wrapping_add(1 as ::core::ffi::c_uint)
                                    .wrapping_sub(1 as ::core::ffi::c_uint)
                                    as isize,
                            ),
                            *(*(*index).env).xdf2.recs.offset(
                                be.wrapping_add(1 as ::core::ffi::c_uint)
                                    .wrapping_sub(1 as ::core::ffi::c_uint)
                                    as isize,
                            ),
                        ) != 0
                    {
                        ae = ae.wrapping_add(1);
                        be = be.wrapping_add(1);
                        if (1 as ::core::ffi::c_uint) < rc {
                            rc = if rc
                                < (**(*index)
                                    .line_map
                                    .offset(ae.wrapping_sub((*index).ptr_shift) as isize))
                                .cnt
                            {
                                rc
                            } else {
                                (**(*index)
                                    .line_map
                                    .offset(ae.wrapping_sub((*index).ptr_shift) as isize))
                                .cnt
                            };
                        }
                    }
                    if b_next <= be {
                        b_next = be.wrapping_add(1 as ::core::ffi::c_uint);
                    }
                    if (*lcs).end1.wrapping_sub((*lcs).begin1) < ae.wrapping_sub(as_0)
                        || rc < (*index).cnt
                    {
                        (*lcs).begin1 = as_0;
                        (*lcs).begin2 = bs;
                        (*lcs).end1 = ae;
                        (*lcs).end2 = be;
                        (*index).cnt = rc;
                    }
                    if np == 0 as ::core::ffi::c_uint {
                        break;
                    }
                    while np <= ae {
                        np = *(*index)
                            .next_ptrs
                            .offset(np.wrapping_sub((*index).ptr_shift) as isize);
                        if np != 0 as ::core::ffi::c_uint {
                            continue;
                        }
                        should_break = 1 as ::core::ffi::c_int;
                        break;
                    }
                    if should_break != 0 {
                        break;
                    }
                    as_0 = np;
                }
            }
        }
        rec = (*rec).next;
    }
    return b_next as ::core::ffi::c_int;
}
unsafe extern "C" fn fall_back_to_classic_diff(
    mut xpp: *const xpparam_t,
    mut env: *mut xdfenv_t,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut xpparam: xpparam_t = xpparam_t {
        flags: 0,
        anchors: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        anchors_nr: 0,
    };
    memset(
        &raw mut xpparam as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xpparam_t>(),
    );
    xpparam.flags = (*xpp).flags & !XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong;
    return xdl_fall_back_diff(env, &raw mut xpparam, line1, count1, line2, count2);
}
#[inline]
unsafe extern "C" fn free_index(mut index: *mut histindex) {
    xfree((*index).records as *mut ::core::ffi::c_void);
    xfree((*index).line_map as *mut ::core::ffi::c_void);
    xfree((*index).next_ptrs as *mut ::core::ffi::c_void);
    xdl_cha_free(&raw mut (*index).rcha);
}
unsafe extern "C" fn find_lcs(
    mut xpp: *const xpparam_t,
    mut env: *mut xdfenv_t,
    mut lcs: *mut region,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut b_ptr: ::core::ffi::c_int = 0;
    let mut sz: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut index: histindex = histindex {
        records: ::core::ptr::null_mut::<*mut record>(),
        line_map: ::core::ptr::null_mut::<*mut record>(),
        rcha: chastore_t {
            head: ::core::ptr::null_mut::<chanode_t>(),
            tail: ::core::ptr::null_mut::<chanode_t>(),
            isize: 0,
            nsize: 0,
            ancur: ::core::ptr::null_mut::<chanode_t>(),
            sncur: ::core::ptr::null_mut::<chanode_t>(),
            scurr: 0,
        },
        next_ptrs: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
        table_bits: 0,
        records_size: 0,
        line_map_size: 0,
        max_chain_length: 0,
        key_shift: 0,
        ptr_shift: 0,
        cnt: 0,
        has_common: 0,
        env: ::core::ptr::null_mut::<xdfenv_t>(),
        xpp: ::core::ptr::null::<xpparam_t>(),
    };
    memset(
        &raw mut index as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<histindex>(),
    );
    index.env = env;
    index.xpp = xpp;
    index.records = ::core::ptr::null_mut::<*mut record>();
    index.line_map = ::core::ptr::null_mut::<*mut record>();
    index.rcha.head = ::core::ptr::null_mut::<chanode_t>();
    index.table_bits = xdl_hashbits(count1 as ::core::ffi::c_uint);
    index.records_size = ((1 as ::core::ffi::c_int) << index.table_bits) as ::core::ffi::c_uint;
    sz = index.records_size as ::core::ffi::c_int;
    sz = (sz as ::core::ffi::c_ulong)
        .wrapping_mul(::core::mem::size_of::<*mut record>() as ::core::ffi::c_ulong)
        as ::core::ffi::c_int;
    index.records = xmalloc(sz as size_t) as *mut *mut record as *mut *mut record;
    if !index.records.is_null() {
        memset(
            index.records as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            sz as size_t,
        );
        index.line_map_size = count1 as ::core::ffi::c_uint;
        sz = index.line_map_size as ::core::ffi::c_int;
        sz = (sz as ::core::ffi::c_ulong)
            .wrapping_mul(::core::mem::size_of::<*mut record>() as ::core::ffi::c_ulong)
            as ::core::ffi::c_int;
        index.line_map = xmalloc(sz as size_t) as *mut *mut record as *mut *mut record;
        if !index.line_map.is_null() {
            memset(
                index.line_map as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                sz as size_t,
            );
            sz = index.line_map_size as ::core::ffi::c_int;
            sz = (sz as ::core::ffi::c_ulong)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_uint>() as ::core::ffi::c_ulong)
                as ::core::ffi::c_int;
            index.next_ptrs = xmalloc(sz as size_t) as *mut ::core::ffi::c_uint;
            if !index.next_ptrs.is_null() {
                memset(
                    index.next_ptrs as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    sz as size_t,
                );
                if xdl_cha_init(
                    &raw mut index.rcha,
                    ::core::mem::size_of::<record>() as ::core::ffi::c_long,
                    (count1 / 4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        as ::core::ffi::c_long,
                ) >= 0 as ::core::ffi::c_int
                {
                    index.ptr_shift = line1 as ::core::ffi::c_uint;
                    index.max_chain_length = 64 as ::core::ffi::c_uint;
                    if scanA(&raw mut index, line1, count1) == 0 {
                        index.cnt = index
                            .max_chain_length
                            .wrapping_add(1 as ::core::ffi::c_uint);
                        b_ptr = line2;
                        while b_ptr <= line2 + count2 - 1 as ::core::ffi::c_int {
                            b_ptr =
                                try_lcs(&raw mut index, lcs, b_ptr, line1, count1, line2, count2);
                        }
                        if index.has_common != 0 && index.max_chain_length < index.cnt {
                            ret = 1 as ::core::ffi::c_int;
                        } else {
                            ret = 0 as ::core::ffi::c_int;
                        }
                    }
                }
            }
        }
    }
    free_index(&raw mut index);
    return ret;
}
unsafe extern "C" fn histogram_diff(
    mut xpp: *const xpparam_t,
    mut env: *mut xdfenv_t,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcs: region = region {
        begin1: 0,
        end1: 0,
        begin2: 0,
        end2: 0,
    };
    let mut lcs_found: ::core::ffi::c_int = 0;
    let mut result: ::core::ffi::c_int = 0;
    loop {
        result = -1 as ::core::ffi::c_int;
        if count1 <= 0 as ::core::ffi::c_int && count2 <= 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if line1 + count1 - 1 as ::core::ffi::c_int >= MAX_PTR {
            return -1 as ::core::ffi::c_int;
        }
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
            &raw mut lcs as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<region>(),
        );
        lcs_found = find_lcs(xpp, env, &raw mut lcs, line1, count1, line2, count2);
        if lcs_found < 0 as ::core::ffi::c_int {
            break;
        }
        if lcs_found != 0 {
            result = fall_back_to_classic_diff(xpp, env, line1, count1, line2, count2);
            break;
        } else if lcs.begin1 == 0 as ::core::ffi::c_uint && lcs.begin2 == 0 as ::core::ffi::c_uint {
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
            result = 0 as ::core::ffi::c_int;
            break;
        } else {
            result = histogram_diff(
                xpp,
                env,
                line1,
                lcs.begin1.wrapping_sub(line1 as ::core::ffi::c_uint) as ::core::ffi::c_int,
                line2,
                lcs.begin2.wrapping_sub(line2 as ::core::ffi::c_uint) as ::core::ffi::c_int,
            );
            if result != 0 {
                break;
            }
            count1 = ((line1 + count1 - 1 as ::core::ffi::c_int) as ::core::ffi::c_uint)
                .wrapping_sub(lcs.end1) as ::core::ffi::c_int;
            line1 = lcs.end1.wrapping_add(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
            count2 = ((line2 + count2 - 1 as ::core::ffi::c_int) as ::core::ffi::c_uint)
                .wrapping_sub(lcs.end2) as ::core::ffi::c_int;
            line2 = lcs.end2.wrapping_add(1 as ::core::ffi::c_uint) as ::core::ffi::c_int;
        }
    }
    return result;
}
pub unsafe extern "C" fn xdl_do_histogram_diff(
    mut file1: *mut mmfile_t,
    mut file2: *mut mmfile_t,
    mut xpp: *const xpparam_t,
    mut env: *mut xdfenv_t,
) -> ::core::ffi::c_int {
    if xdl_prepare_env(file1, file2, xpp, env) < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    return histogram_diff(
        xpp,
        env,
        ((*env).xdf1.dstart + 1 as ::core::ffi::c_long) as ::core::ffi::c_int,
        ((*env).xdf1.dend - (*env).xdf1.dstart + 1 as ::core::ffi::c_long) as ::core::ffi::c_int,
        ((*env).xdf2.dstart + 1 as ::core::ffi::c_long) as ::core::ffi::c_int,
        ((*env).xdf2.dend - (*env).xdf2.dstart + 1 as ::core::ffi::c_long) as ::core::ffi::c_int,
    );
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
