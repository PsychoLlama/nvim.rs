pub use crate::src::nvim::types::{
    chanode_t, chastore_t, mmbuffer_t, mmfile_t, s_chanode, s_chastore, s_mmbuffer, s_mmfile,
    s_xdemitcb, s_xdfenv, s_xdfile, s_xpparam, s_xrecord, size_t, xdemitcb_t, xdfenv_t, xdfile_t,
    xpparam_t, xrecord_t,
};
extern "C" {
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xdl_free_env(xe: *mut xdfenv_t);
    fn xdl_do_diff(
        mf1: *mut mmfile_t,
        mf2: *mut mmfile_t,
        xpp: *const xpparam_t,
        xe: *mut xdfenv_t,
    ) -> ::core::ffi::c_int;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const XDF_IGNORE_WHITESPACE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_CHANGE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_AT_EOL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
pub const XDF_IGNORE_CR_AT_EOL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const XDF_WHITESPACE_FLAGS: ::core::ffi::c_int = XDF_IGNORE_WHITESPACE
    | XDF_IGNORE_WHITESPACE_CHANGE
    | XDF_IGNORE_WHITESPACE_AT_EOL
    | XDF_IGNORE_CR_AT_EOL;
#[no_mangle]
pub unsafe extern "C" fn xdl_bogosqrt(mut n: ::core::ffi::c_long) -> ::core::ffi::c_long {
    let mut i: ::core::ffi::c_long = 0;
    i = 1 as ::core::ffi::c_long;
    while n > 0 as ::core::ffi::c_long {
        i <<= 1 as ::core::ffi::c_int;
        n >>= 2 as ::core::ffi::c_int;
    }
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_emit_diffrec(
    mut rec: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_long,
    mut pre: *const ::core::ffi::c_char,
    mut psize: ::core::ffi::c_long,
    mut ecb: *mut xdemitcb_t,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    let mut mb: [mmbuffer_t; 3] = [mmbuffer_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    }; 3];
    mb[0 as ::core::ffi::c_int as usize].ptr = pre as *mut ::core::ffi::c_char;
    mb[0 as ::core::ffi::c_int as usize].size = psize as ::core::ffi::c_int;
    mb[1 as ::core::ffi::c_int as usize].ptr = rec as *mut ::core::ffi::c_char;
    mb[1 as ::core::ffi::c_int as usize].size = size as ::core::ffi::c_int;
    if size > 0 as ::core::ffi::c_long
        && *rec.offset((size - 1 as ::core::ffi::c_long) as isize) as ::core::ffi::c_int
            != '\n' as ::core::ffi::c_int
    {
        mb[2 as ::core::ffi::c_int as usize].ptr = b"\n\\ No newline at end of file\n\0".as_ptr()
            as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char;
        mb[2 as ::core::ffi::c_int as usize].size = strlen(mb[2 as ::core::ffi::c_int as usize].ptr)
            as ::core::ffi::c_long
            as ::core::ffi::c_int;
        i += 1;
    }
    if (*ecb).out_line.expect("non-null function pointer")(
        (*ecb).priv_0,
        &raw mut mb as *mut mmbuffer_t,
        i,
    ) < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_mmfile_first(
    mut mmf: *mut mmfile_t,
    mut size: *mut ::core::ffi::c_long,
) -> *mut ::core::ffi::c_void {
    *size = (*mmf).size as ::core::ffi::c_long;
    return (*mmf).ptr as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_mmfile_size(mut mmf: *mut mmfile_t) -> ::core::ffi::c_long {
    return (*mmf).size as ::core::ffi::c_long;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_cha_init(
    mut cha: *mut chastore_t,
    mut isize: ::core::ffi::c_long,
    mut icount: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    (*cha).tail = ::core::ptr::null_mut::<chanode_t>();
    (*cha).head = (*cha).tail;
    (*cha).isize = isize;
    (*cha).nsize = icount * isize;
    (*cha).sncur = ::core::ptr::null_mut::<chanode_t>();
    (*cha).ancur = (*cha).sncur;
    (*cha).scurr = 0 as ::core::ffi::c_long;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_cha_free(mut cha: *mut chastore_t) {
    let mut cur: *mut chanode_t = ::core::ptr::null_mut::<chanode_t>();
    let mut tmp: *mut chanode_t = ::core::ptr::null_mut::<chanode_t>();
    cur = (*cha).head;
    loop {
        tmp = cur;
        if tmp.is_null() {
            break;
        }
        cur = (*cur).next as *mut chanode_t;
        xfree(tmp as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn xdl_cha_alloc(mut cha: *mut chastore_t) -> *mut ::core::ffi::c_void {
    let mut ancur: *mut chanode_t = ::core::ptr::null_mut::<chanode_t>();
    let mut data: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    ancur = (*cha).ancur;
    if ancur.is_null() || (*ancur).icurr == (*cha).nsize {
        ancur = xmalloc(::core::mem::size_of::<chanode_t>().wrapping_add((*cha).nsize as size_t))
            as *mut chanode_t;
        if ancur.is_null() {
            return NULL;
        }
        (*ancur).icurr = 0 as ::core::ffi::c_long;
        (*ancur).next = ::core::ptr::null_mut::<s_chanode>();
        if !(*cha).tail.is_null() {
            (*(*cha).tail).next = ancur as *mut s_chanode;
        }
        if (*cha).head.is_null() {
            (*cha).head = ancur;
        }
        (*cha).tail = ancur;
        (*cha).ancur = ancur;
    }
    data = (ancur as *mut ::core::ffi::c_char)
        .offset(::core::mem::size_of::<chanode_t>() as isize)
        .offset((*ancur).icurr as isize) as *mut ::core::ffi::c_void;
    (*ancur).icurr += (*cha).isize;
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_guess_lines(
    mut mf: *mut mmfile_t,
    mut sample: ::core::ffi::c_long,
) -> ::core::ffi::c_long {
    let mut nl: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut size: ::core::ffi::c_long = 0;
    let mut tsize: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut data: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cur: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut top: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    data = xdl_mmfile_first(mf, &raw mut size) as *const ::core::ffi::c_char;
    cur = data;
    if !cur.is_null() {
        top = data.offset(size as isize);
        while nl < sample && cur < top {
            nl += 1;
            cur = memchr(
                cur as *const ::core::ffi::c_void,
                '\n' as ::core::ffi::c_int,
                top.offset_from(cur) as size_t,
            ) as *const ::core::ffi::c_char;
            if cur.is_null() {
                cur = top;
            } else {
                cur = cur.offset(1);
            }
        }
        tsize += cur.offset_from(data) as ::core::ffi::c_long;
    }
    if nl != 0 && tsize != 0 {
        nl = xdl_mmfile_size(mf) / (tsize / nl);
    }
    return nl + 1 as ::core::ffi::c_long;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_blankline(
    mut line: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_long,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_long = 0;
    if flags & XDF_WHITESPACE_FLAGS as ::core::ffi::c_long == 0 {
        return (size <= 1 as ::core::ffi::c_long) as ::core::ffi::c_int;
    }
    i = 0 as ::core::ffi::c_long;
    while i < size
        && *(*__ctype_b_loc())
            .offset(*line.offset(i as isize) as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
    {
        i += 1;
    }
    return (i == size) as ::core::ffi::c_int;
}
unsafe extern "C" fn ends_with_optional_cr(
    mut l: *const ::core::ffi::c_char,
    mut s: ::core::ffi::c_long,
    mut i: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut complete: ::core::ffi::c_int = (s != 0
        && *l.offset((s - 1 as ::core::ffi::c_long) as isize) as ::core::ffi::c_int
            == '\n' as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if complete != 0 {
        s -= 1;
    }
    if s == i {
        return 1 as ::core::ffi::c_int;
    }
    if complete != 0
        && s == i + 1 as ::core::ffi::c_long
        && *l.offset(i as isize) as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_recmatch(
    mut l1: *const ::core::ffi::c_char,
    mut s1: ::core::ffi::c_long,
    mut l2: *const ::core::ffi::c_char,
    mut s2: ::core::ffi::c_long,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut i1: ::core::ffi::c_int = 0;
    let mut i2: ::core::ffi::c_int = 0;
    if s1 == s2
        && memcmp(
            l1 as *const ::core::ffi::c_void,
            l2 as *const ::core::ffi::c_void,
            s1 as size_t,
        ) == 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if flags & XDF_WHITESPACE_FLAGS as ::core::ffi::c_long == 0 {
        return 0 as ::core::ffi::c_int;
    }
    i1 = 0 as ::core::ffi::c_int;
    i2 = 0 as ::core::ffi::c_int;
    if flags & XDF_IGNORE_WHITESPACE as ::core::ffi::c_long != 0 {
        loop {
            while (i1 as ::core::ffi::c_long) < s1
                && *(*__ctype_b_loc()).offset(
                    *l1.offset(i1 as isize) as ::core::ffi::c_uchar as ::core::ffi::c_int as isize
                ) as ::core::ffi::c_int
                    & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                i1 += 1;
            }
            while (i2 as ::core::ffi::c_long) < s2
                && *(*__ctype_b_loc()).offset(
                    *l2.offset(i2 as isize) as ::core::ffi::c_uchar as ::core::ffi::c_int as isize
                ) as ::core::ffi::c_int
                    & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                i2 += 1;
            }
            if !((i1 as ::core::ffi::c_long) < s1 && (i2 as ::core::ffi::c_long) < s2) {
                break;
            }
            let c2rust_fresh0 = i1;
            i1 = i1 + 1;
            let c2rust_fresh1 = i2;
            i2 = i2 + 1;
            if *l1.offset(c2rust_fresh0 as isize) as ::core::ffi::c_int
                != *l2.offset(c2rust_fresh1 as isize) as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
        }
    } else if flags & XDF_IGNORE_WHITESPACE_CHANGE as ::core::ffi::c_long != 0 {
        while (i1 as ::core::ffi::c_long) < s1 && (i2 as ::core::ffi::c_long) < s2 {
            if *(*__ctype_b_loc()).offset(
                *l1.offset(i1 as isize) as ::core::ffi::c_uchar as ::core::ffi::c_int as isize
            ) as ::core::ffi::c_int
                & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
                && *(*__ctype_b_loc()).offset(
                    *l2.offset(i2 as isize) as ::core::ffi::c_uchar as ::core::ffi::c_int as isize
                ) as ::core::ffi::c_int
                    & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
            {
                while (i1 as ::core::ffi::c_long) < s1
                    && *(*__ctype_b_loc())
                        .offset(*l1.offset(i1 as isize) as ::core::ffi::c_uchar
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0
                {
                    i1 += 1;
                }
                while (i2 as ::core::ffi::c_long) < s2
                    && *(*__ctype_b_loc())
                        .offset(*l2.offset(i2 as isize) as ::core::ffi::c_uchar
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0
                {
                    i2 += 1;
                }
            } else {
                let c2rust_fresh2 = i1;
                i1 = i1 + 1;
                let c2rust_fresh3 = i2;
                i2 = i2 + 1;
                if *l1.offset(c2rust_fresh2 as isize) as ::core::ffi::c_int
                    != *l2.offset(c2rust_fresh3 as isize) as ::core::ffi::c_int
                {
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    } else if flags & XDF_IGNORE_WHITESPACE_AT_EOL as ::core::ffi::c_long != 0 {
        while (i1 as ::core::ffi::c_long) < s1
            && (i2 as ::core::ffi::c_long) < s2
            && *l1.offset(i1 as isize) as ::core::ffi::c_int
                == *l2.offset(i2 as isize) as ::core::ffi::c_int
        {
            i1 += 1;
            i2 += 1;
        }
    } else if flags & XDF_IGNORE_CR_AT_EOL as ::core::ffi::c_long != 0 {
        while (i1 as ::core::ffi::c_long) < s1
            && (i2 as ::core::ffi::c_long) < s2
            && *l1.offset(i1 as isize) as ::core::ffi::c_int
                == *l2.offset(i2 as isize) as ::core::ffi::c_int
        {
            i1 += 1;
            i2 += 1;
        }
        return (ends_with_optional_cr(l1, s1, i1 as ::core::ffi::c_long) != 0
            && ends_with_optional_cr(l2, s2, i2 as ::core::ffi::c_long) != 0)
            as ::core::ffi::c_int;
    }
    if (i1 as ::core::ffi::c_long) < s1 {
        while (i1 as ::core::ffi::c_long) < s1
            && *(*__ctype_b_loc()).offset(
                *l1.offset(i1 as isize) as ::core::ffi::c_uchar as ::core::ffi::c_int as isize
            ) as ::core::ffi::c_int
                & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
        {
            i1 += 1;
        }
        if s1 != i1 as ::core::ffi::c_long {
            return 0 as ::core::ffi::c_int;
        }
    }
    if (i2 as ::core::ffi::c_long) < s2 {
        while (i2 as ::core::ffi::c_long) < s2
            && *(*__ctype_b_loc()).offset(
                *l2.offset(i2 as isize) as ::core::ffi::c_uchar as ::core::ffi::c_int as isize
            ) as ::core::ffi::c_int
                & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
        {
            i2 += 1;
        }
        return (s2 == i2 as ::core::ffi::c_long) as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_hash_record_with_whitespace(
    mut data: *mut *const ::core::ffi::c_char,
    mut top: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_ulong {
    let mut ha: ::core::ffi::c_ulong = 5381 as ::core::ffi::c_ulong;
    let mut ptr: *const ::core::ffi::c_char = *data;
    let mut cr_at_eol_only: ::core::ffi::c_int =
        (flags & XDF_WHITESPACE_FLAGS as ::core::ffi::c_long
            == XDF_IGNORE_CR_AT_EOL as ::core::ffi::c_long) as ::core::ffi::c_int;
    while ptr < top && *ptr as ::core::ffi::c_int != '\n' as ::core::ffi::c_int {
        's_10: {
            if cr_at_eol_only != 0 {
                if *ptr as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                    && (ptr.offset(1 as ::core::ffi::c_int as isize) < top
                        && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\n' as ::core::ffi::c_int)
                {
                    break 's_10;
                }
            } else if *(*__ctype_b_loc())
                .offset(*ptr as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                let mut ptr2: *const ::core::ffi::c_char = ptr;
                let mut at_eol: ::core::ffi::c_int = 0;
                while ptr.offset(1 as ::core::ffi::c_int as isize) < top
                    && *(*__ctype_b_loc())
                        .offset(*ptr.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_uchar
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0
                    && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '\n' as ::core::ffi::c_int
                {
                    ptr = ptr.offset(1);
                }
                at_eol = (top <= ptr.offset(1 as ::core::ffi::c_int as isize)
                    || *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
                if flags & XDF_IGNORE_WHITESPACE as ::core::ffi::c_long == 0 {
                    if flags & XDF_IGNORE_WHITESPACE_CHANGE as ::core::ffi::c_long != 0
                        && at_eol == 0
                    {
                        ha = ha.wrapping_add(ha << 5 as ::core::ffi::c_int);
                        ha ^= ' ' as ::core::ffi::c_int as ::core::ffi::c_ulong;
                    } else if flags & XDF_IGNORE_WHITESPACE_AT_EOL as ::core::ffi::c_long != 0
                        && at_eol == 0
                    {
                        while ptr2 != ptr.offset(1 as ::core::ffi::c_int as isize) {
                            ha = ha.wrapping_add(ha << 5 as ::core::ffi::c_int);
                            ha ^= *ptr2 as ::core::ffi::c_ulong;
                            ptr2 = ptr2.offset(1);
                        }
                    }
                }
                break 's_10;
            }
            ha = ha.wrapping_add(ha << 5 as ::core::ffi::c_int);
            ha ^= *ptr as ::core::ffi::c_ulong;
        }
        ptr = ptr.offset(1);
    }
    *data = if ptr < top {
        ptr.offset(1 as ::core::ffi::c_int as isize)
    } else {
        ptr
    };
    return ha;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_hash_record(
    mut data: *mut *const ::core::ffi::c_char,
    mut top: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_ulong {
    let mut ha: ::core::ffi::c_ulong = 5381 as ::core::ffi::c_ulong;
    let mut ptr: *const ::core::ffi::c_char = *data;
    if flags & XDF_WHITESPACE_FLAGS as ::core::ffi::c_long != 0 {
        return xdl_hash_record_with_whitespace(data, top, flags);
    }
    while ptr < top && *ptr as ::core::ffi::c_int != '\n' as ::core::ffi::c_int {
        ha = ha.wrapping_add(ha << 5 as ::core::ffi::c_int);
        ha ^= *ptr as ::core::ffi::c_ulong;
        ptr = ptr.offset(1);
    }
    *data = if ptr < top {
        ptr.offset(1 as ::core::ffi::c_int as isize)
    } else {
        ptr
    };
    return ha;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_hashbits(mut size: ::core::ffi::c_uint) -> ::core::ffi::c_uint {
    let mut val: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
    let mut bits: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while val < size
        && (bits as usize)
            < (CHAR_BIT as usize).wrapping_mul(::core::mem::size_of::<::core::ffi::c_uint>())
    {
        val <<= 1 as ::core::ffi::c_int;
        bits = bits.wrapping_add(1);
    }
    return if bits != 0 {
        bits
    } else {
        1 as ::core::ffi::c_uint
    };
}
#[no_mangle]
pub unsafe extern "C" fn xdl_num_out(
    mut out: *mut ::core::ffi::c_char,
    mut val: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut str: *mut ::core::ffi::c_char = out;
    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
    ptr = (&raw mut buf as *mut ::core::ffi::c_char)
        .offset(::core::mem::size_of::<[::core::ffi::c_char; 32]>() as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    *ptr = '\0' as ::core::ffi::c_char;
    if val < 0 as ::core::ffi::c_long {
        ptr = ptr.offset(-1);
        *ptr = '-' as ::core::ffi::c_char;
        val = -val;
    }
    while val != 0 && ptr > &raw mut buf as *mut ::core::ffi::c_char {
        ptr = ptr.offset(-1);
        *ptr = ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"0123456789\0")
            [(val % 10 as ::core::ffi::c_long) as usize];
        val /= 10 as ::core::ffi::c_long;
    }
    if *ptr != 0 {
        while *ptr != 0 {
            *str = *ptr;
            ptr = ptr.offset(1);
            str = str.offset(1);
        }
    } else {
        let c2rust_fresh4 = str;
        str = str.offset(1);
        *c2rust_fresh4 = '0' as ::core::ffi::c_char;
    }
    *str = '\0' as ::core::ffi::c_char;
    return str.offset_from(out) as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_format_hunk_hdr(
    mut s1: ::core::ffi::c_long,
    mut c1: ::core::ffi::c_long,
    mut s2: ::core::ffi::c_long,
    mut c2: ::core::ffi::c_long,
    mut func: *const ::core::ffi::c_char,
    mut funclen: ::core::ffi::c_long,
    mut ecb: *mut xdemitcb_t,
) -> ::core::ffi::c_int {
    let mut nb: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mb: mmbuffer_t = mmbuffer_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut buf: [::core::ffi::c_char; 128] = [0; 128];
    memcpy(
        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        b"@@ -\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        4 as size_t,
    );
    nb += 4 as ::core::ffi::c_int;
    nb += xdl_num_out(
        (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize),
        if c1 != 0 {
            s1
        } else {
            s1 - 1 as ::core::ffi::c_long
        },
    );
    if c1 != 1 as ::core::ffi::c_long {
        memcpy(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize)
                as *mut ::core::ffi::c_void,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            1 as size_t,
        );
        nb += 1 as ::core::ffi::c_int;
        nb += xdl_num_out(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize),
            c1,
        );
    }
    memcpy(
        (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize) as *mut ::core::ffi::c_void,
        b" +\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        2 as size_t,
    );
    nb += 2 as ::core::ffi::c_int;
    nb += xdl_num_out(
        (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize),
        if c2 != 0 {
            s2
        } else {
            s2 - 1 as ::core::ffi::c_long
        },
    );
    if c2 != 1 as ::core::ffi::c_long {
        memcpy(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize)
                as *mut ::core::ffi::c_void,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            1 as size_t,
        );
        nb += 1 as ::core::ffi::c_int;
        nb += xdl_num_out(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize),
            c2,
        );
    }
    memcpy(
        (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize) as *mut ::core::ffi::c_void,
        b" @@\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        3 as size_t,
    );
    nb += 3 as ::core::ffi::c_int;
    if !func.is_null() && funclen != 0 {
        let c2rust_fresh5 = nb;
        nb = nb + 1;
        buf[c2rust_fresh5 as usize] = ' ' as ::core::ffi::c_char;
        if funclen
            > ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as ::core::ffi::c_long
                - nb as ::core::ffi::c_long
                - 1 as ::core::ffi::c_long
        {
            funclen = ::core::mem::size_of::<[::core::ffi::c_char; 128]>()
                .wrapping_sub(nb as usize)
                .wrapping_sub(1 as usize) as ::core::ffi::c_long;
        }
        memcpy(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(nb as isize)
                as *mut ::core::ffi::c_void,
            func as *const ::core::ffi::c_void,
            funclen as size_t,
        );
        nb = (nb as ::core::ffi::c_long + funclen) as ::core::ffi::c_int;
    }
    let c2rust_fresh6 = nb;
    nb = nb + 1;
    buf[c2rust_fresh6 as usize] = '\n' as ::core::ffi::c_char;
    mb.ptr = &raw mut buf as *mut ::core::ffi::c_char;
    mb.size = nb;
    if (*ecb).out_line.expect("non-null function pointer")(
        (*ecb).priv_0,
        &raw mut mb,
        1 as ::core::ffi::c_int,
    ) < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_emit_hunk_hdr(
    mut s1: ::core::ffi::c_long,
    mut c1: ::core::ffi::c_long,
    mut s2: ::core::ffi::c_long,
    mut c2: ::core::ffi::c_long,
    mut func: *const ::core::ffi::c_char,
    mut funclen: ::core::ffi::c_long,
    mut ecb: *mut xdemitcb_t,
) -> ::core::ffi::c_int {
    if (*ecb).out_hunk.is_none() {
        return xdl_format_hunk_hdr(s1, c1, s2, c2, func, funclen, ecb);
    }
    if (*ecb).out_hunk.expect("non-null function pointer")(
        (*ecb).priv_0,
        if c1 != 0 {
            s1
        } else {
            s1 - 1 as ::core::ffi::c_long
        },
        c1,
        if c2 != 0 {
            s2
        } else {
            s2 - 1 as ::core::ffi::c_long
        },
        c2,
        func,
        funclen,
    ) < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_fall_back_diff(
    mut diff_env: *mut xdfenv_t,
    mut xpp: *const xpparam_t,
    mut line1: ::core::ffi::c_int,
    mut count1: ::core::ffi::c_int,
    mut line2: ::core::ffi::c_int,
    mut count2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut subfile1: mmfile_t = mmfile_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut subfile2: mmfile_t = mmfile_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut env: xdfenv_t = xdfenv_t {
        xdf1: xdfile_t {
            rcha: chastore_t {
                head: ::core::ptr::null_mut::<chanode_t>(),
                tail: ::core::ptr::null_mut::<chanode_t>(),
                isize: 0,
                nsize: 0,
                ancur: ::core::ptr::null_mut::<chanode_t>(),
                sncur: ::core::ptr::null_mut::<chanode_t>(),
                scurr: 0,
            },
            nrec: 0,
            hbits: 0,
            rhash: ::core::ptr::null_mut::<*mut xrecord_t>(),
            dstart: 0,
            dend: 0,
            recs: ::core::ptr::null_mut::<*mut xrecord_t>(),
            rchg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            rindex: ::core::ptr::null_mut::<::core::ffi::c_long>(),
            nreff: 0,
            ha: ::core::ptr::null_mut::<::core::ffi::c_ulong>(),
        },
        xdf2: xdfile_t {
            rcha: chastore_t {
                head: ::core::ptr::null_mut::<chanode_t>(),
                tail: ::core::ptr::null_mut::<chanode_t>(),
                isize: 0,
                nsize: 0,
                ancur: ::core::ptr::null_mut::<chanode_t>(),
                sncur: ::core::ptr::null_mut::<chanode_t>(),
                scurr: 0,
            },
            nrec: 0,
            hbits: 0,
            rhash: ::core::ptr::null_mut::<*mut xrecord_t>(),
            dstart: 0,
            dend: 0,
            recs: ::core::ptr::null_mut::<*mut xrecord_t>(),
            rchg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            rindex: ::core::ptr::null_mut::<::core::ffi::c_long>(),
            nreff: 0,
            ha: ::core::ptr::null_mut::<::core::ffi::c_ulong>(),
        },
    };
    subfile1.ptr = (**(*diff_env)
        .xdf1
        .recs
        .offset((line1 - 1 as ::core::ffi::c_int) as isize))
    .ptr as *mut ::core::ffi::c_char;
    subfile1.size = (**(*diff_env)
        .xdf1
        .recs
        .offset((line1 + count1 - 2 as ::core::ffi::c_int) as isize))
    .ptr
    .offset(
        (**(*diff_env)
            .xdf1
            .recs
            .offset((line1 + count1 - 2 as ::core::ffi::c_int) as isize))
        .size as isize,
    )
    .offset_from(subfile1.ptr) as ::core::ffi::c_int;
    subfile2.ptr = (**(*diff_env)
        .xdf2
        .recs
        .offset((line2 - 1 as ::core::ffi::c_int) as isize))
    .ptr as *mut ::core::ffi::c_char;
    subfile2.size = (**(*diff_env)
        .xdf2
        .recs
        .offset((line2 + count2 - 2 as ::core::ffi::c_int) as isize))
    .ptr
    .offset(
        (**(*diff_env)
            .xdf2
            .recs
            .offset((line2 + count2 - 2 as ::core::ffi::c_int) as isize))
        .size as isize,
    )
    .offset_from(subfile2.ptr) as ::core::ffi::c_int;
    if xdl_do_diff(&raw mut subfile1, &raw mut subfile2, xpp, &raw mut env)
        < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    memcpy(
        (*diff_env)
            .xdf1
            .rchg
            .offset(line1 as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
        env.xdf1.rchg as *const ::core::ffi::c_void,
        count1 as size_t,
    );
    memcpy(
        (*diff_env)
            .xdf2
            .rchg
            .offset(line2 as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
        env.xdf2.rchg as *const ::core::ffi::c_void,
        count2 as size_t,
    );
    xdl_free_env(&raw mut env);
    return 0 as ::core::ffi::c_int;
}
pub const __CHAR_BIT__: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CHAR_BIT: ::core::ffi::c_int = __CHAR_BIT__;
