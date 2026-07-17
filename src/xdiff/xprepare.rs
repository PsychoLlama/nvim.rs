extern "C" {
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xdl_mmfile_first(
        mmf: *mut mmfile_t,
        size: *mut ::core::ffi::c_long,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xdl_bogosqrt(n: ::core::ffi::c_long) -> ::core::ffi::c_long;
    fn xdl_cha_init(
        cha: *mut chastore_t,
        isize: ::core::ffi::c_long,
        icount: ::core::ffi::c_long,
    ) -> ::core::ffi::c_int;
    fn xdl_cha_free(cha: *mut chastore_t);
    fn xdl_cha_alloc(cha: *mut chastore_t) -> *mut ::core::ffi::c_void;
    fn xdl_guess_lines(mf: *mut mmfile_t, sample: ::core::ffi::c_long) -> ::core::ffi::c_long;
    fn xdl_recmatch(
        l1: *const ::core::ffi::c_char,
        s1: ::core::ffi::c_long,
        l2: *const ::core::ffi::c_char,
        s2: ::core::ffi::c_long,
        flags: ::core::ffi::c_long,
    ) -> ::core::ffi::c_int;
    fn xdl_hash_record(
        data: *mut *const ::core::ffi::c_char,
        top: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_long,
    ) -> ::core::ffi::c_ulong;
    fn xdl_hashbits(size: ::core::ffi::c_uint) -> ::core::ffi::c_uint;
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
pub type xdlclassifier_t = s_xdlclassifier;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdlclassifier {
    pub hbits: ::core::ffi::c_uint,
    pub hsize: ::core::ffi::c_long,
    pub rchash: *mut *mut xdlclass_t,
    pub ncha: chastore_t,
    pub rcrecs: *mut *mut xdlclass_t,
    pub alloc: ::core::ffi::c_long,
    pub count: ::core::ffi::c_long,
    pub flags: ::core::ffi::c_long,
}
pub type xdlclass_t = s_xdlclass;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdlclass {
    pub next: *mut s_xdlclass,
    pub ha: ::core::ffi::c_ulong,
    pub line: *const ::core::ffi::c_char,
    pub size: ::core::ffi::c_long,
    pub idx: ::core::ffi::c_long,
    pub len1: ::core::ffi::c_long,
    pub len2: ::core::ffi::c_long,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const XDF_PATIENCE_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const XDF_HISTOGRAM_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
pub const XDF_DIFF_ALGORITHM_MASK: ::core::ffi::c_int = XDF_PATIENCE_DIFF | XDF_HISTOGRAM_DIFF;
pub const XDL_KPDIS_RUN: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const XDL_MAX_EQLIMIT: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const XDL_SIMSCAN_WINDOW: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const XDL_GUESS_NLINES1: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const XDL_GUESS_NLINES2: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
unsafe extern "C" fn xdl_init_classifier(
    mut cf: *mut xdlclassifier_t,
    mut size: ::core::ffi::c_long,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    (*cf).flags = flags;
    (*cf).hbits = xdl_hashbits(size as ::core::ffi::c_uint);
    (*cf).hsize = ((1 as ::core::ffi::c_int) << (*cf).hbits) as ::core::ffi::c_long;
    if xdl_cha_init(
        &raw mut (*cf).ncha,
        ::core::mem::size_of::<xdlclass_t>() as ::core::ffi::c_long,
        size / 4 as ::core::ffi::c_long + 1 as ::core::ffi::c_long,
    ) < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    (*cf).rchash =
        xmalloc(((*cf).hsize as size_t).wrapping_mul(::core::mem::size_of::<*mut xdlclass_t>()))
            as *mut *mut xdlclass_t;
    if (*cf).rchash.is_null() {
        xdl_cha_free(&raw mut (*cf).ncha);
        return -1 as ::core::ffi::c_int;
    }
    memset(
        (*cf).rchash as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ((*cf).hsize as size_t).wrapping_mul(::core::mem::size_of::<*mut xdlclass_t>()),
    );
    (*cf).alloc = size;
    (*cf).rcrecs =
        xmalloc(((*cf).alloc as size_t).wrapping_mul(::core::mem::size_of::<*mut xdlclass_t>()))
            as *mut *mut xdlclass_t;
    if (*cf).rcrecs.is_null() {
        xfree((*cf).rchash as *mut ::core::ffi::c_void);
        xdl_cha_free(&raw mut (*cf).ncha);
        return -1 as ::core::ffi::c_int;
    }
    (*cf).count = 0 as ::core::ffi::c_long;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_free_classifier(mut cf: *mut xdlclassifier_t) {
    xfree((*cf).rcrecs as *mut ::core::ffi::c_void);
    xfree((*cf).rchash as *mut ::core::ffi::c_void);
    xdl_cha_free(&raw mut (*cf).ncha);
}
unsafe extern "C" fn xdl_classify_record(
    mut pass: ::core::ffi::c_uint,
    mut cf: *mut xdlclassifier_t,
    mut rhash: *mut *mut xrecord_t,
    mut hbits: ::core::ffi::c_uint,
    mut rec: *mut xrecord_t,
) -> ::core::ffi::c_int {
    let mut hi: ::core::ffi::c_long = 0;
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut rcrec: *mut xdlclass_t = ::core::ptr::null_mut::<xdlclass_t>();
    let mut rcrecs: *mut *mut xdlclass_t = ::core::ptr::null_mut::<*mut xdlclass_t>();
    line = (*rec).ptr;
    hi = ((*rec).ha.wrapping_add((*rec).ha >> (*cf).hbits)
        & ((1 as ::core::ffi::c_ulong) << (*cf).hbits).wrapping_sub(1 as ::core::ffi::c_ulong))
        as ::core::ffi::c_long;
    rcrec = *(*cf).rchash.offset(hi as isize);
    while !rcrec.is_null() {
        if (*rcrec).ha == (*rec).ha
            && xdl_recmatch(
                (*rcrec).line,
                (*rcrec).size,
                (*rec).ptr,
                (*rec).size,
                (*cf).flags,
            ) != 0
        {
            break;
        }
        rcrec = (*rcrec).next as *mut xdlclass_t;
    }
    if rcrec.is_null() {
        rcrec = xdl_cha_alloc(&raw mut (*cf).ncha) as *mut xdlclass_t;
        if rcrec.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        let c2rust_fresh1 = (*cf).count;
        (*cf).count = (*cf).count + 1;
        (*rcrec).idx = c2rust_fresh1;
        if (*cf).count > (*cf).alloc {
            (*cf).alloc *= 2 as ::core::ffi::c_long;
            rcrecs = xrealloc(
                (*cf).rcrecs as *mut ::core::ffi::c_void,
                ((*cf).alloc as size_t).wrapping_mul(::core::mem::size_of::<*mut xdlclass_t>()),
            ) as *mut *mut xdlclass_t;
            if rcrecs.is_null() {
                return -1 as ::core::ffi::c_int;
            }
            (*cf).rcrecs = rcrecs;
        }
        *(*cf).rcrecs.offset((*rcrec).idx as isize) = rcrec;
        (*rcrec).line = line;
        (*rcrec).size = (*rec).size;
        (*rcrec).ha = (*rec).ha;
        (*rcrec).len2 = 0 as ::core::ffi::c_long;
        (*rcrec).len1 = (*rcrec).len2;
        (*rcrec).next = *(*cf).rchash.offset(hi as isize) as *mut s_xdlclass;
        *(*cf).rchash.offset(hi as isize) = rcrec;
    }
    if pass == 1 as ::core::ffi::c_uint {
        (*rcrec).len1 += 1;
    } else {
        (*rcrec).len2 += 1;
    };
    (*rec).ha = (*rcrec).idx as ::core::ffi::c_ulong;
    hi = ((*rec).ha.wrapping_add((*rec).ha >> hbits)
        & ((1 as ::core::ffi::c_ulong) << hbits).wrapping_sub(1 as ::core::ffi::c_ulong))
        as ::core::ffi::c_long;
    (*rec).next = *rhash.offset(hi as isize) as *mut s_xrecord;
    *rhash.offset(hi as isize) = rec;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_prepare_ctx(
    mut pass: ::core::ffi::c_uint,
    mut mf: *mut mmfile_t,
    mut narec: ::core::ffi::c_long,
    mut xpp: *const xpparam_t,
    mut cf: *mut xdlclassifier_t,
    mut xdf: *mut xdfile_t,
) -> ::core::ffi::c_int {
    let mut hbits: ::core::ffi::c_uint = 0;
    let mut nrec: ::core::ffi::c_long = 0;
    let mut hsize: ::core::ffi::c_long = 0;
    let mut bsize: ::core::ffi::c_long = 0;
    let mut hav: ::core::ffi::c_ulong = 0;
    let mut blk: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cur: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut top: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut prev: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut crec: *mut xrecord_t = ::core::ptr::null_mut::<xrecord_t>();
    let mut recs: *mut *mut xrecord_t = ::core::ptr::null_mut::<*mut xrecord_t>();
    let mut rrecs: *mut *mut xrecord_t = ::core::ptr::null_mut::<*mut xrecord_t>();
    let mut rhash: *mut *mut xrecord_t = ::core::ptr::null_mut::<*mut xrecord_t>();
    let mut ha: *mut ::core::ffi::c_ulong = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    let mut rchg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut rindex: *mut ::core::ffi::c_long = ::core::ptr::null_mut::<::core::ffi::c_long>();
    ha = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    rindex = ::core::ptr::null_mut::<::core::ffi::c_long>();
    rchg = ::core::ptr::null_mut::<::core::ffi::c_char>();
    rhash = ::core::ptr::null_mut::<*mut xrecord_t>();
    recs = ::core::ptr::null_mut::<*mut xrecord_t>();
    '_abort: {
        if xdl_cha_init(
            &raw mut (*xdf).rcha,
            ::core::mem::size_of::<xrecord_t>() as ::core::ffi::c_long,
            narec / 4 as ::core::ffi::c_long + 1 as ::core::ffi::c_long,
        ) >= 0 as ::core::ffi::c_int
        {
            recs = xmalloc((narec as size_t).wrapping_mul(::core::mem::size_of::<*mut xrecord_t>()))
                as *mut *mut xrecord_t;
            if !recs.is_null() {
                if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
                    == XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong
                {
                    hsize = 0 as ::core::ffi::c_long;
                    hbits = hsize as ::core::ffi::c_uint;
                } else {
                    hbits = xdl_hashbits(narec as ::core::ffi::c_uint);
                    hsize = ((1 as ::core::ffi::c_int) << hbits) as ::core::ffi::c_long;
                    rhash = xmalloc(
                        (hsize as size_t).wrapping_mul(::core::mem::size_of::<*mut xrecord_t>()),
                    ) as *mut *mut xrecord_t;
                    if rhash.is_null() {
                        break '_abort;
                    } else {
                        memset(
                            rhash as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            (hsize as size_t)
                                .wrapping_mul(::core::mem::size_of::<*mut xrecord_t>()),
                        );
                    }
                }
                nrec = 0 as ::core::ffi::c_long;
                's_139: {
                    blk = xdl_mmfile_first(mf, &raw mut bsize) as *const ::core::ffi::c_char;
                    cur = blk;
                    if !cur.is_null() {
                        top = blk.offset(bsize as isize);
                        loop {
                            if cur >= top {
                                break 's_139;
                            }
                            prev = cur;
                            hav = xdl_hash_record(
                                &raw mut cur,
                                top,
                                (*xpp).flags as ::core::ffi::c_long,
                            );
                            if nrec >= narec {
                                narec *= 2 as ::core::ffi::c_long;
                                rrecs = xrealloc(
                                    recs as *mut ::core::ffi::c_void,
                                    (narec as size_t)
                                        .wrapping_mul(::core::mem::size_of::<*mut xrecord_t>()),
                                ) as *mut *mut xrecord_t;
                                if rrecs.is_null() {
                                    break '_abort;
                                }
                                recs = rrecs;
                            }
                            crec = xdl_cha_alloc(&raw mut (*xdf).rcha) as *mut xrecord_t;
                            if crec.is_null() {
                                break '_abort;
                            }
                            (*crec).ptr = prev;
                            (*crec).size = cur.offset_from(prev) as ::core::ffi::c_long;
                            (*crec).ha = hav;
                            let c2rust_fresh0 = nrec;
                            nrec = nrec + 1;
                            let c2rust_lvalue_ptr = &raw mut *recs.offset(c2rust_fresh0 as isize);
                            *c2rust_lvalue_ptr = crec;
                            if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
                                != XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong
                                && xdl_classify_record(pass, cf, rhash, hbits, crec)
                                    < 0 as ::core::ffi::c_int
                            {
                                break '_abort;
                            }
                        }
                    }
                }
                rchg = xmalloc(
                    ((nrec + 2 as ::core::ffi::c_long) as size_t)
                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                ) as *mut ::core::ffi::c_char;
                if !rchg.is_null() {
                    memset(
                        rchg as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        ((nrec + 2 as ::core::ffi::c_long) as size_t)
                            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                    );
                    rindex = xmalloc(
                        ((nrec + 1 as ::core::ffi::c_long) as size_t)
                            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_long>()),
                    ) as *mut ::core::ffi::c_long;
                    if !rindex.is_null() {
                        ha = xmalloc(
                            ((nrec + 1 as ::core::ffi::c_long) as size_t)
                                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_ulong>()),
                        ) as *mut ::core::ffi::c_ulong;
                        if !ha.is_null() {
                            (*xdf).nrec = nrec;
                            (*xdf).recs = recs;
                            (*xdf).hbits = hbits;
                            (*xdf).rhash = rhash;
                            (*xdf).rchg = rchg.offset(1 as ::core::ffi::c_int as isize);
                            (*xdf).rindex = rindex;
                            (*xdf).nreff = 0 as ::core::ffi::c_long;
                            (*xdf).ha = ha;
                            (*xdf).dstart = 0 as ::core::ffi::c_long;
                            (*xdf).dend = nrec - 1 as ::core::ffi::c_long;
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
            }
        }
    }
    xfree(ha as *mut ::core::ffi::c_void);
    xfree(rindex as *mut ::core::ffi::c_void);
    xfree(rchg as *mut ::core::ffi::c_void);
    xfree(rhash as *mut ::core::ffi::c_void);
    xfree(recs as *mut ::core::ffi::c_void);
    xdl_cha_free(&raw mut (*xdf).rcha);
    return -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_free_ctx(mut xdf: *mut xdfile_t) {
    xfree((*xdf).rhash as *mut ::core::ffi::c_void);
    xfree((*xdf).rindex as *mut ::core::ffi::c_void);
    xfree((*xdf).rchg.offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void);
    xfree((*xdf).ha as *mut ::core::ffi::c_void);
    xfree((*xdf).recs as *mut ::core::ffi::c_void);
    xdl_cha_free(&raw mut (*xdf).rcha);
}
#[no_mangle]
pub unsafe extern "C" fn xdl_prepare_env(
    mut mf1: *mut mmfile_t,
    mut mf2: *mut mmfile_t,
    mut xpp: *const xpparam_t,
    mut xe: *mut xdfenv_t,
) -> ::core::ffi::c_int {
    let mut enl1: ::core::ffi::c_long = 0;
    let mut enl2: ::core::ffi::c_long = 0;
    let mut sample: ::core::ffi::c_long = 0;
    let mut cf: xdlclassifier_t = xdlclassifier_t {
        hbits: 0,
        hsize: 0,
        rchash: ::core::ptr::null_mut::<*mut xdlclass_t>(),
        ncha: chastore_t {
            head: ::core::ptr::null_mut::<chanode_t>(),
            tail: ::core::ptr::null_mut::<chanode_t>(),
            isize: 0,
            nsize: 0,
            ancur: ::core::ptr::null_mut::<chanode_t>(),
            sncur: ::core::ptr::null_mut::<chanode_t>(),
            scurr: 0,
        },
        rcrecs: ::core::ptr::null_mut::<*mut xdlclass_t>(),
        alloc: 0,
        count: 0,
        flags: 0,
    };
    memset(
        &raw mut cf as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xdlclassifier_t>(),
    );
    sample = (if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
        == XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong
    {
        XDL_GUESS_NLINES2
    } else {
        XDL_GUESS_NLINES1
    }) as ::core::ffi::c_long;
    enl1 = xdl_guess_lines(mf1, sample) + 1 as ::core::ffi::c_long;
    enl2 = xdl_guess_lines(mf2, sample) + 1 as ::core::ffi::c_long;
    if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
        != XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong
        && xdl_init_classifier(
            &raw mut cf,
            enl1 + enl2 + 1 as ::core::ffi::c_long,
            (*xpp).flags as ::core::ffi::c_long,
        ) < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    if xdl_prepare_ctx(
        1 as ::core::ffi::c_uint,
        mf1,
        enl1,
        xpp,
        &raw mut cf,
        &raw mut (*xe).xdf1,
    ) < 0 as ::core::ffi::c_int
    {
        xdl_free_classifier(&raw mut cf);
        return -1 as ::core::ffi::c_int;
    }
    if xdl_prepare_ctx(
        2 as ::core::ffi::c_uint,
        mf2,
        enl2,
        xpp,
        &raw mut cf,
        &raw mut (*xe).xdf2,
    ) < 0 as ::core::ffi::c_int
    {
        xdl_free_ctx(&raw mut (*xe).xdf1);
        xdl_free_classifier(&raw mut cf);
        return -1 as ::core::ffi::c_int;
    }
    if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
        != XDF_PATIENCE_DIFF as ::core::ffi::c_ulong
        && (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
            != XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong
        && xdl_optimize_ctxs(&raw mut cf, &raw mut (*xe).xdf1, &raw mut (*xe).xdf2)
            < 0 as ::core::ffi::c_int
    {
        xdl_free_ctx(&raw mut (*xe).xdf2);
        xdl_free_ctx(&raw mut (*xe).xdf1);
        xdl_free_classifier(&raw mut cf);
        return -1 as ::core::ffi::c_int;
    }
    if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
        != XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong
    {
        xdl_free_classifier(&raw mut cf);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_free_env(mut xe: *mut xdfenv_t) {
    xdl_free_ctx(&raw mut (*xe).xdf2);
    xdl_free_ctx(&raw mut (*xe).xdf1);
}
unsafe extern "C" fn xdl_clean_mmatch(
    mut dis: *const ::core::ffi::c_char,
    mut i: ::core::ffi::c_long,
    mut s: ::core::ffi::c_long,
    mut e: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_long = 0;
    let mut rdis0: ::core::ffi::c_long = 0;
    let mut rpdis0: ::core::ffi::c_long = 0;
    let mut rdis1: ::core::ffi::c_long = 0;
    let mut rpdis1: ::core::ffi::c_long = 0;
    if i - s > XDL_SIMSCAN_WINDOW as ::core::ffi::c_long {
        s = i - XDL_SIMSCAN_WINDOW as ::core::ffi::c_long;
    }
    if e - i > XDL_SIMSCAN_WINDOW as ::core::ffi::c_long {
        e = i + XDL_SIMSCAN_WINDOW as ::core::ffi::c_long;
    }
    r = 1 as ::core::ffi::c_long;
    rdis0 = 0 as ::core::ffi::c_long;
    rpdis0 = 1 as ::core::ffi::c_long;
    while i - r >= s {
        if *dis.offset((i - r) as isize) == 0 {
            rdis0 += 1;
        } else {
            if *dis.offset((i - r) as isize) as ::core::ffi::c_int != 2 as ::core::ffi::c_int {
                break;
            }
            rpdis0 += 1;
        }
        r += 1;
    }
    if rdis0 == 0 as ::core::ffi::c_long {
        return 0 as ::core::ffi::c_int;
    }
    r = 1 as ::core::ffi::c_long;
    rdis1 = 0 as ::core::ffi::c_long;
    rpdis1 = 1 as ::core::ffi::c_long;
    while i + r <= e {
        if *dis.offset((i + r) as isize) == 0 {
            rdis1 += 1;
        } else {
            if *dis.offset((i + r) as isize) as ::core::ffi::c_int != 2 as ::core::ffi::c_int {
                break;
            }
            rpdis1 += 1;
        }
        r += 1;
    }
    if rdis1 == 0 as ::core::ffi::c_long {
        return 0 as ::core::ffi::c_int;
    }
    rdis1 += rdis0;
    rpdis1 += rpdis0;
    return ((rpdis1 * XDL_KPDIS_RUN as ::core::ffi::c_long) < rpdis1 + rdis1)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_cleanup_records(
    mut cf: *mut xdlclassifier_t,
    mut xdf1: *mut xdfile_t,
    mut xdf2: *mut xdfile_t,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_long = 0;
    let mut nm: ::core::ffi::c_long = 0;
    let mut nreff: ::core::ffi::c_long = 0;
    let mut mlim: ::core::ffi::c_long = 0;
    let mut recs: *mut *mut xrecord_t = ::core::ptr::null_mut::<*mut xrecord_t>();
    let mut rcrec: *mut xdlclass_t = ::core::ptr::null_mut::<xdlclass_t>();
    let mut dis: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dis1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dis2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    dis = xmalloc(((*xdf1).nrec + (*xdf2).nrec + 2 as ::core::ffi::c_long) as size_t)
        as *mut ::core::ffi::c_char;
    if dis.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    memset(
        dis as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ((*xdf1).nrec + (*xdf2).nrec + 2 as ::core::ffi::c_long) as size_t,
    );
    dis1 = dis;
    dis2 = dis1
        .offset((*xdf1).nrec as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    mlim = xdl_bogosqrt((*xdf1).nrec);
    if mlim > XDL_MAX_EQLIMIT as ::core::ffi::c_long {
        mlim = XDL_MAX_EQLIMIT as ::core::ffi::c_long;
    }
    i = (*xdf1).dstart;
    recs = (*xdf1).recs.offset((*xdf1).dstart as isize);
    while i <= (*xdf1).dend {
        rcrec = *(*cf).rcrecs.offset((**recs).ha as isize);
        nm = if !rcrec.is_null() {
            (*rcrec).len2
        } else {
            0 as ::core::ffi::c_long
        };
        *dis1.offset(i as isize) = (if nm == 0 as ::core::ffi::c_long {
            0 as ::core::ffi::c_int
        } else if nm >= mlim {
            2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as ::core::ffi::c_char;
        i += 1;
        recs = recs.offset(1);
    }
    mlim = xdl_bogosqrt((*xdf2).nrec);
    if mlim > XDL_MAX_EQLIMIT as ::core::ffi::c_long {
        mlim = XDL_MAX_EQLIMIT as ::core::ffi::c_long;
    }
    i = (*xdf2).dstart;
    recs = (*xdf2).recs.offset((*xdf2).dstart as isize);
    while i <= (*xdf2).dend {
        rcrec = *(*cf).rcrecs.offset((**recs).ha as isize);
        nm = if !rcrec.is_null() {
            (*rcrec).len1
        } else {
            0 as ::core::ffi::c_long
        };
        *dis2.offset(i as isize) = (if nm == 0 as ::core::ffi::c_long {
            0 as ::core::ffi::c_int
        } else if nm >= mlim {
            2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as ::core::ffi::c_char;
        i += 1;
        recs = recs.offset(1);
    }
    nreff = 0 as ::core::ffi::c_long;
    i = (*xdf1).dstart;
    recs = (*xdf1).recs.offset((*xdf1).dstart as isize);
    while i <= (*xdf1).dend {
        if *dis1.offset(i as isize) as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            || *dis1.offset(i as isize) as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                && xdl_clean_mmatch(dis1, i, (*xdf1).dstart, (*xdf1).dend) == 0
        {
            *(*xdf1).rindex.offset(nreff as isize) = i;
            *(*xdf1).ha.offset(nreff as isize) = (**recs).ha;
            nreff += 1;
        } else {
            *(*xdf1).rchg.offset(i as isize) = 1 as ::core::ffi::c_char;
        }
        i += 1;
        recs = recs.offset(1);
    }
    (*xdf1).nreff = nreff;
    nreff = 0 as ::core::ffi::c_long;
    i = (*xdf2).dstart;
    recs = (*xdf2).recs.offset((*xdf2).dstart as isize);
    while i <= (*xdf2).dend {
        if *dis2.offset(i as isize) as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            || *dis2.offset(i as isize) as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                && xdl_clean_mmatch(dis2, i, (*xdf2).dstart, (*xdf2).dend) == 0
        {
            *(*xdf2).rindex.offset(nreff as isize) = i;
            *(*xdf2).ha.offset(nreff as isize) = (**recs).ha;
            nreff += 1;
        } else {
            *(*xdf2).rchg.offset(i as isize) = 1 as ::core::ffi::c_char;
        }
        i += 1;
        recs = recs.offset(1);
    }
    (*xdf2).nreff = nreff;
    xfree(dis as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_trim_ends(
    mut xdf1: *mut xdfile_t,
    mut xdf2: *mut xdfile_t,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_long = 0;
    let mut lim: ::core::ffi::c_long = 0;
    let mut recs1: *mut *mut xrecord_t = ::core::ptr::null_mut::<*mut xrecord_t>();
    let mut recs2: *mut *mut xrecord_t = ::core::ptr::null_mut::<*mut xrecord_t>();
    recs1 = (*xdf1).recs;
    recs2 = (*xdf2).recs;
    i = 0 as ::core::ffi::c_long;
    lim = if (*xdf1).nrec < (*xdf2).nrec {
        (*xdf1).nrec
    } else {
        (*xdf2).nrec
    };
    while i < lim {
        if (**recs1).ha != (**recs2).ha {
            break;
        }
        i += 1;
        recs1 = recs1.offset(1);
        recs2 = recs2.offset(1);
    }
    (*xdf2).dstart = i;
    (*xdf1).dstart = (*xdf2).dstart;
    recs1 = (*xdf1)
        .recs
        .offset((*xdf1).nrec as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    recs2 = (*xdf2)
        .recs
        .offset((*xdf2).nrec as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    lim -= i;
    i = 0 as ::core::ffi::c_long;
    while i < lim {
        if (**recs1).ha != (**recs2).ha {
            break;
        }
        i += 1;
        recs1 = recs1.offset(-1);
        recs2 = recs2.offset(-1);
    }
    (*xdf1).dend = (*xdf1).nrec - i - 1 as ::core::ffi::c_long;
    (*xdf2).dend = (*xdf2).nrec - i - 1 as ::core::ffi::c_long;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_optimize_ctxs(
    mut cf: *mut xdlclassifier_t,
    mut xdf1: *mut xdfile_t,
    mut xdf2: *mut xdfile_t,
) -> ::core::ffi::c_int {
    if xdl_trim_ends(xdf1, xdf2) < 0 as ::core::ffi::c_int
        || xdl_cleanup_records(cf, xdf1, xdf2) < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
