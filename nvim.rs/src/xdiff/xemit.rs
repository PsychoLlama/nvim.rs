extern "C" {
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xdl_emit_diffrec(
        rec: *const ::core::ffi::c_char,
        size: ::core::ffi::c_long,
        pre: *const ::core::ffi::c_char,
        psize: ::core::ffi::c_long,
        ecb: *mut xdemitcb_t,
    ) -> ::core::ffi::c_int;
    fn xdl_emit_hunk_hdr(
        s1: ::core::ffi::c_long,
        c1: ::core::ffi::c_long,
        s2: ::core::ffi::c_long,
        c2: ::core::ffi::c_long,
        func: *const ::core::ffi::c_char,
        funclen: ::core::ffi::c_long,
        ecb: *mut xdemitcb_t,
    ) -> ::core::ffi::c_int;
}
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_mmbuffer {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
pub type mmbuffer_t = s_mmbuffer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdemitcb {
    pub priv_0: *mut ::core::ffi::c_void,
    pub out_hunk: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            ::core::ffi::c_long,
            ::core::ffi::c_long,
            ::core::ffi::c_long,
            ::core::ffi::c_long,
            *const ::core::ffi::c_char,
            ::core::ffi::c_long,
        ) -> ::core::ffi::c_int,
    >,
    pub out_line: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            *mut mmbuffer_t,
            ::core::ffi::c_int,
        ) -> ::core::ffi::c_int,
    >,
}
pub type xdemitcb_t = s_xdemitcb;
pub type find_func_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_char,
        ::core::ffi::c_long,
        *mut ::core::ffi::c_char,
        ::core::ffi::c_long,
        *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_long,
>;
pub type xdl_emit_hunk_consume_func_t = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdemitconf {
    pub ctxlen: ::core::ffi::c_long,
    pub interhunkctxlen: ::core::ffi::c_long,
    pub flags: ::core::ffi::c_ulong,
    pub find_func: find_func_t,
    pub find_func_priv: *mut ::core::ffi::c_void,
    pub hunk_func: xdl_emit_hunk_consume_func_t,
}
pub type xdemitconf_t = s_xdemitconf;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdchange {
    pub next: *mut s_xdchange,
    pub i1: ::core::ffi::c_long,
    pub i2: ::core::ffi::c_long,
    pub chg1: ::core::ffi::c_long,
    pub chg2: ::core::ffi::c_long,
    pub ignore: ::core::ffi::c_int,
}
pub type xdchange_t = s_xdchange;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct func_line {
    pub len: ::core::ffi::c_long,
    pub buf: [::core::ffi::c_char; 80],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const XDL_EMIT_NO_HUNK_HDR: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 1 as ::core::ffi::c_int;
unsafe extern "C" fn xdl_get_rec(
    mut xdf: *mut xdfile_t,
    mut ri: ::core::ffi::c_long,
    mut rec: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_long {
    *rec = (**(*xdf).recs.offset(ri as isize)).ptr;
    return (**(*xdf).recs.offset(ri as isize)).size;
}
unsafe extern "C" fn xdl_emit_record(
    mut xdf: *mut xdfile_t,
    mut ri: ::core::ffi::c_long,
    mut pre: *const ::core::ffi::c_char,
    mut ecb: *mut xdemitcb_t,
) -> ::core::ffi::c_int {
    let mut size: ::core::ffi::c_long = 0;
    let mut psize: ::core::ffi::c_long = strlen(pre) as ::core::ffi::c_long;
    let mut rec: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    size = xdl_get_rec(xdf, ri, &raw mut rec);
    if xdl_emit_diffrec(rec, size, pre, psize, ecb) < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_get_hunk(
    mut xscr: *mut *mut xdchange_t,
    mut xecfg: *const xdemitconf_t,
) -> *mut xdchange_t {
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut xchp: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut lxch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut max_common: ::core::ffi::c_long = 2 as ::core::ffi::c_long * (*xecfg).ctxlen
        + (*xecfg).interhunkctxlen;
    let mut max_ignorable: ::core::ffi::c_long = (*xecfg).ctxlen;
    let mut ignored: ::core::ffi::c_ulong = 0 as ::core::ffi::c_ulong;
    xchp = *xscr;
    while !xchp.is_null() && (*xchp).ignore != 0 {
        xch = (*xchp).next as *mut xdchange_t;
        if xch.is_null() || (*xch).i1 - ((*xchp).i1 + (*xchp).chg1) >= max_ignorable {
            *xscr = xch;
        }
        xchp = (*xchp).next as *mut xdchange_t;
    }
    if (*xscr).is_null() {
        return ::core::ptr::null_mut::<xdchange_t>();
    }
    lxch = *xscr;
    xchp = *xscr;
    xch = (*xchp).next as *mut xdchange_t;
    while !xch.is_null() {
        let mut distance: ::core::ffi::c_long = (*xch).i1 - ((*xchp).i1 + (*xchp).chg1);
        if distance > max_common {
            break;
        }
        if distance < max_ignorable && ((*xch).ignore == 0 || lxch == xchp) {
            lxch = xch;
            ignored = 0 as ::core::ffi::c_ulong;
        } else if distance < max_ignorable && (*xch).ignore != 0 {
            ignored = ignored.wrapping_add((*xch).chg2 as ::core::ffi::c_ulong);
        } else {
            if lxch != xchp
                && (*xch).i1 + ignored as ::core::ffi::c_long
                    - ((*lxch).i1 + (*lxch).chg1) > max_common
            {
                break;
            }
            if (*xch).ignore == 0 {
                lxch = xch;
                ignored = 0 as ::core::ffi::c_ulong;
            } else {
                ignored = ignored.wrapping_add((*xch).chg2 as ::core::ffi::c_ulong);
            }
        }
        xchp = xch;
        xch = (*xch).next as *mut xdchange_t;
    }
    return lxch;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_emit_diff(
    mut xe: *mut xdfenv_t,
    mut xscr: *mut xdchange_t,
    mut ecb: *mut xdemitcb_t,
    mut xecfg: *const xdemitconf_t,
) -> ::core::ffi::c_int {
    let mut s1: ::core::ffi::c_long = 0;
    let mut s2: ::core::ffi::c_long = 0;
    let mut e1: ::core::ffi::c_long = 0;
    let mut e2: ::core::ffi::c_long = 0;
    let mut lctx: ::core::ffi::c_long = 0;
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut xche: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut func_line: func_line = func_line { len: 0, buf: [0; 80] };
    func_line.len = 0 as ::core::ffi::c_long;
    xch = xscr;
    while !xch.is_null() {
        xche = xdl_get_hunk(&raw mut xch, xecfg);
        if xch.is_null() {
            break;
        }
        s1 = if (*xch).i1 - (*xecfg).ctxlen > 0 as ::core::ffi::c_long {
            (*xch).i1 - (*xecfg).ctxlen
        } else {
            0 as ::core::ffi::c_long
        };
        s2 = if (*xch).i2 - (*xecfg).ctxlen > 0 as ::core::ffi::c_long {
            (*xch).i2 - (*xecfg).ctxlen
        } else {
            0 as ::core::ffi::c_long
        };
        lctx = (*xecfg).ctxlen;
        lctx = if lctx < (*xe).xdf1.nrec - ((*xche).i1 + (*xche).chg1) {
            lctx
        } else {
            (*xe).xdf1.nrec - ((*xche).i1 + (*xche).chg1)
        };
        lctx = if lctx < (*xe).xdf2.nrec - ((*xche).i2 + (*xche).chg2) {
            lctx
        } else {
            (*xe).xdf2.nrec - ((*xche).i2 + (*xche).chg2)
        };
        e1 = (*xche).i1 + (*xche).chg1 + lctx;
        e2 = (*xche).i2 + (*xche).chg2 + lctx;
        if (*xecfg).flags & XDL_EMIT_NO_HUNK_HDR as ::core::ffi::c_ulong == 0
            && xdl_emit_hunk_hdr(
                s1 + 1 as ::core::ffi::c_long,
                e1 - s1,
                s2 + 1 as ::core::ffi::c_long,
                e2 - s2,
                &raw mut func_line.buf as *mut ::core::ffi::c_char,
                func_line.len,
                ecb,
            ) < 0 as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
        while s2 < (*xch).i2 {
            if xdl_emit_record(
                &raw mut (*xe).xdf2,
                s2,
                b" \0".as_ptr() as *const ::core::ffi::c_char,
                ecb,
            ) < 0 as ::core::ffi::c_int
            {
                return -1 as ::core::ffi::c_int;
            }
            s2 += 1;
        }
        s1 = (*xch).i1;
        s2 = (*xch).i2;
        loop {
            while s1 < (*xch).i1 && s2 < (*xch).i2 {
                if xdl_emit_record(
                    &raw mut (*xe).xdf2,
                    s2,
                    b" \0".as_ptr() as *const ::core::ffi::c_char,
                    ecb,
                ) < 0 as ::core::ffi::c_int
                {
                    return -1 as ::core::ffi::c_int;
                }
                s1 += 1;
                s2 += 1;
            }
            s1 = (*xch).i1;
            while s1 < (*xch).i1 + (*xch).chg1 {
                if xdl_emit_record(
                    &raw mut (*xe).xdf1,
                    s1,
                    b"-\0".as_ptr() as *const ::core::ffi::c_char,
                    ecb,
                ) < 0 as ::core::ffi::c_int
                {
                    return -1 as ::core::ffi::c_int;
                }
                s1 += 1;
            }
            s2 = (*xch).i2;
            while s2 < (*xch).i2 + (*xch).chg2 {
                if xdl_emit_record(
                    &raw mut (*xe).xdf2,
                    s2,
                    b"+\0".as_ptr() as *const ::core::ffi::c_char,
                    ecb,
                ) < 0 as ::core::ffi::c_int
                {
                    return -1 as ::core::ffi::c_int;
                }
                s2 += 1;
            }
            if xch == xche {
                break;
            }
            s1 = (*xch).i1 + (*xch).chg1;
            s2 = (*xch).i2 + (*xch).chg2;
            xch = (*xch).next as *mut xdchange_t;
        }
        s2 = (*xche).i2 + (*xche).chg2;
        while s2 < e2 {
            if xdl_emit_record(
                &raw mut (*xe).xdf2,
                s2,
                b" \0".as_ptr() as *const ::core::ffi::c_char,
                ecb,
            ) < 0 as ::core::ffi::c_int
            {
                return -1 as ::core::ffi::c_int;
            }
            s2 += 1;
        }
        xch = (*xche).next as *mut xdchange_t;
    }
    return 0 as ::core::ffi::c_int;
}
