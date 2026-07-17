extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    static mut stderr: *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn exit(__status: ::core::ffi::c_int) -> !;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xdl_prepare_env(
        mf1: *mut mmfile_t,
        mf2: *mut mmfile_t,
        xpp: *const xpparam_t,
        xe: *mut xdfenv_t,
    ) -> ::core::ffi::c_int;
    fn xdl_free_env(xe: *mut xdfenv_t);
    fn xdl_emit_diff(
        xe: *mut xdfenv_t,
        xscr: *mut xdchange_t,
        ecb: *mut xdemitcb_t,
        xecfg: *const xdemitconf_t,
    ) -> ::core::ffi::c_int;
    fn xdl_do_patience_diff(
        mf1: *mut mmfile_t,
        mf2: *mut mmfile_t,
        xpp: *const xpparam_t,
        env: *mut xdfenv_t,
    ) -> ::core::ffi::c_int;
    fn xdl_do_histogram_diff(
        mf1: *mut mmfile_t,
        mf2: *mut mmfile_t,
        xpp: *const xpparam_t,
        env: *mut xdfenv_t,
    ) -> ::core::ffi::c_int;
    fn xdl_get_hunk(xscr: *mut *mut xdchange_t, xecfg: *const xdemitconf_t) -> *mut xdchange_t;
    fn xdl_bogosqrt(n: ::core::ffi::c_long) -> ::core::ffi::c_long;
    fn xdl_blankline(
        line: *const ::core::ffi::c_char,
        size: ::core::ffi::c_long,
        flags: ::core::ffi::c_long,
    ) -> ::core::ffi::c_int;
    fn xdl_recmatch(
        l1: *const ::core::ffi::c_char,
        s1: ::core::ffi::c_long,
        l2: *const ::core::ffi::c_char,
        s2: ::core::ffi::c_long,
        flags: ::core::ffi::c_long,
    ) -> ::core::ffi::c_int;
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
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
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_mmfile {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
pub type mmfile_t = s_mmfile;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_mmbuffer {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
pub type mmbuffer_t = s_mmbuffer;
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
pub type xdfenv_t = s_xdfenv;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdfenv {
    pub xdf1: xdfile_t,
    pub xdf2: xdfile_t,
}
pub type xdfile_t = s_xdfile;
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
pub type xrecord_t = s_xrecord;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xrecord {
    pub next: *mut s_xrecord,
    pub ptr: *const ::core::ffi::c_char,
    pub size: ::core::ffi::c_long,
    pub ha: ::core::ffi::c_ulong,
}
pub type chastore_t = s_chastore;
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
pub type chanode_t = s_chanode;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_chanode {
    pub next: *mut s_chanode,
    pub icurr: ::core::ffi::c_long,
}
pub type xdchange_t = s_xdchange;
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
pub type emit_func_t = Option<
    unsafe extern "C" fn(
        *mut xdfenv_t,
        *mut xdchange_t,
        *mut xdemitcb_t,
        *const xdemitconf_t,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct xdlgroup {
    pub start: ::core::ffi::c_long,
    pub end: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct split_score {
    pub effective_indent: ::core::ffi::c_int,
    pub penalty: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct split_measurement {
    pub end_of_file: ::core::ffi::c_int,
    pub indent: ::core::ffi::c_int,
    pub pre_blank: ::core::ffi::c_int,
    pub pre_indent: ::core::ffi::c_int,
    pub post_blank: ::core::ffi::c_int,
    pub post_indent: ::core::ffi::c_int,
}
pub type xdalgoenv_t = s_xdalgoenv;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdalgoenv {
    pub mxcost: ::core::ffi::c_long,
    pub snake_cnt: ::core::ffi::c_long,
    pub heur_min: ::core::ffi::c_long,
}
pub type diffdata_t = s_diffdata;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_diffdata {
    pub nrec: ::core::ffi::c_long,
    pub ha: *const ::core::ffi::c_ulong,
    pub rindex: *mut ::core::ffi::c_long,
    pub rchg: *mut ::core::ffi::c_char,
}
pub type xdpsplit_t = s_xdpsplit;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdpsplit {
    pub i1: ::core::ffi::c_long,
    pub i2: ::core::ffi::c_long,
    pub min_lo: ::core::ffi::c_int,
    pub min_hi: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const XDF_NEED_MINIMAL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int;
pub const XDF_IGNORE_BLANK_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const XDF_PATIENCE_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const XDF_HISTOGRAM_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
pub const XDF_DIFF_ALGORITHM_MASK: ::core::ffi::c_int = XDF_PATIENCE_DIFF | XDF_HISTOGRAM_DIFF;
pub const XDF_INDENT_HEURISTIC: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 23 as ::core::ffi::c_int;
pub const XDL_MAX_COST_MIN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const XDL_HEUR_MIN_COST: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const XDL_LINE_MAX: ::core::ffi::c_long = ((1 as ::core::ffi::c_ulong)
    << (CHAR_BIT as usize)
        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_long>())
        .wrapping_sub(1 as usize))
.wrapping_sub(1 as ::core::ffi::c_ulong)
    as ::core::ffi::c_long;
pub const XDL_SNAKE_CNT: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const XDL_K_HEUR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
unsafe extern "C" fn xdl_split(
    mut ha1: *const ::core::ffi::c_ulong,
    mut off1: ::core::ffi::c_long,
    mut lim1: ::core::ffi::c_long,
    mut ha2: *const ::core::ffi::c_ulong,
    mut off2: ::core::ffi::c_long,
    mut lim2: ::core::ffi::c_long,
    mut kvdf: *mut ::core::ffi::c_long,
    mut kvdb: *mut ::core::ffi::c_long,
    mut need_min: ::core::ffi::c_int,
    mut spl: *mut xdpsplit_t,
    mut xenv: *mut xdalgoenv_t,
) -> ::core::ffi::c_long {
    let mut dmin: ::core::ffi::c_long = off1 - lim2;
    let mut dmax: ::core::ffi::c_long = lim1 - off2;
    let mut fmid: ::core::ffi::c_long = off1 - off2;
    let mut bmid: ::core::ffi::c_long = lim1 - lim2;
    let mut odd: ::core::ffi::c_long = fmid - bmid & 1 as ::core::ffi::c_long;
    let mut fmin: ::core::ffi::c_long = fmid;
    let mut fmax: ::core::ffi::c_long = fmid;
    let mut bmin: ::core::ffi::c_long = bmid;
    let mut bmax: ::core::ffi::c_long = bmid;
    let mut ec: ::core::ffi::c_long = 0;
    let mut d: ::core::ffi::c_long = 0;
    let mut i1: ::core::ffi::c_long = 0;
    let mut i2: ::core::ffi::c_long = 0;
    let mut prev1: ::core::ffi::c_long = 0;
    let mut best: ::core::ffi::c_long = 0;
    let mut dd: ::core::ffi::c_long = 0;
    let mut v: ::core::ffi::c_long = 0;
    let mut k: ::core::ffi::c_long = 0;
    *kvdf.offset(fmid as isize) = off1;
    *kvdb.offset(bmid as isize) = lim1;
    ec = 1 as ::core::ffi::c_long;
    loop {
        let mut got_snake: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if fmin > dmin {
            fmin -= 1;
            *kvdf.offset((fmin - 1 as ::core::ffi::c_long) as isize) = -1 as ::core::ffi::c_long;
        } else {
            fmin += 1;
        }
        if fmax < dmax {
            fmax += 1;
            *kvdf.offset((fmax + 1 as ::core::ffi::c_long) as isize) = -1 as ::core::ffi::c_long;
        } else {
            fmax -= 1;
        }
        d = fmax;
        while d >= fmin {
            if *kvdf.offset((d - 1 as ::core::ffi::c_long) as isize)
                >= *kvdf.offset((d + 1 as ::core::ffi::c_long) as isize)
            {
                i1 = *kvdf.offset((d - 1 as ::core::ffi::c_long) as isize)
                    + 1 as ::core::ffi::c_long;
            } else {
                i1 = *kvdf.offset((d + 1 as ::core::ffi::c_long) as isize);
            }
            prev1 = i1;
            i2 = i1 - d;
            while i1 < lim1 && i2 < lim2 && *ha1.offset(i1 as isize) == *ha2.offset(i2 as isize) {
                i1 += 1;
                i2 += 1;
            }
            if i1 - prev1 > (*xenv).snake_cnt {
                got_snake = 1 as ::core::ffi::c_int;
            }
            *kvdf.offset(d as isize) = i1;
            if odd != 0 && bmin <= d && d <= bmax && *kvdb.offset(d as isize) <= i1 {
                (*spl).i1 = i1;
                (*spl).i2 = i2;
                (*spl).min_hi = 1 as ::core::ffi::c_int;
                (*spl).min_lo = (*spl).min_hi;
                return ec;
            }
            d -= 2 as ::core::ffi::c_long;
        }
        if bmin > dmin {
            bmin -= 1;
            *kvdb.offset((bmin - 1 as ::core::ffi::c_long) as isize) = XDL_LINE_MAX;
        } else {
            bmin += 1;
        }
        if bmax < dmax {
            bmax += 1;
            *kvdb.offset((bmax + 1 as ::core::ffi::c_long) as isize) = XDL_LINE_MAX;
        } else {
            bmax -= 1;
        }
        d = bmax;
        while d >= bmin {
            if *kvdb.offset((d - 1 as ::core::ffi::c_long) as isize)
                < *kvdb.offset((d + 1 as ::core::ffi::c_long) as isize)
            {
                i1 = *kvdb.offset((d - 1 as ::core::ffi::c_long) as isize);
            } else {
                i1 = *kvdb.offset((d + 1 as ::core::ffi::c_long) as isize)
                    - 1 as ::core::ffi::c_long;
            }
            prev1 = i1;
            i2 = i1 - d;
            while i1 > off1
                && i2 > off2
                && *ha1.offset((i1 - 1 as ::core::ffi::c_long) as isize)
                    == *ha2.offset((i2 - 1 as ::core::ffi::c_long) as isize)
            {
                i1 -= 1;
                i2 -= 1;
            }
            if prev1 - i1 > (*xenv).snake_cnt {
                got_snake = 1 as ::core::ffi::c_int;
            }
            *kvdb.offset(d as isize) = i1;
            if odd == 0 && fmin <= d && d <= fmax && i1 <= *kvdf.offset(d as isize) {
                (*spl).i1 = i1;
                (*spl).i2 = i2;
                (*spl).min_hi = 1 as ::core::ffi::c_int;
                (*spl).min_lo = (*spl).min_hi;
                return ec;
            }
            d -= 2 as ::core::ffi::c_long;
        }
        if need_min == 0 {
            if got_snake != 0 && ec > (*xenv).heur_min {
                best = 0 as ::core::ffi::c_long;
                d = fmax;
                while d >= fmin {
                    dd = if d > fmid { d - fmid } else { fmid - d };
                    i1 = *kvdf.offset(d as isize);
                    i2 = i1 - d;
                    v = i1 - off1 + (i2 - off2) - dd;
                    if v > XDL_K_HEUR as ::core::ffi::c_long * ec
                        && v > best
                        && off1 + (*xenv).snake_cnt <= i1
                        && i1 < lim1
                        && off2 + (*xenv).snake_cnt <= i2
                        && i2 < lim2
                    {
                        k = 1 as ::core::ffi::c_long;
                        while *ha1.offset((i1 - k) as isize) == *ha2.offset((i2 - k) as isize) {
                            if k == (*xenv).snake_cnt {
                                best = v;
                                (*spl).i1 = i1;
                                (*spl).i2 = i2;
                                break;
                            } else {
                                k += 1;
                            }
                        }
                    }
                    d -= 2 as ::core::ffi::c_long;
                }
                if best > 0 as ::core::ffi::c_long {
                    (*spl).min_lo = 1 as ::core::ffi::c_int;
                    (*spl).min_hi = 0 as ::core::ffi::c_int;
                    return ec;
                }
                best = 0 as ::core::ffi::c_long;
                d = bmax;
                while d >= bmin {
                    dd = if d > bmid { d - bmid } else { bmid - d };
                    i1 = *kvdb.offset(d as isize);
                    i2 = i1 - d;
                    v = lim1 - i1 + (lim2 - i2) - dd;
                    if v > XDL_K_HEUR as ::core::ffi::c_long * ec
                        && v > best
                        && off1 < i1
                        && i1 <= lim1 - (*xenv).snake_cnt
                        && off2 < i2
                        && i2 <= lim2 - (*xenv).snake_cnt
                    {
                        k = 0 as ::core::ffi::c_long;
                        while *ha1.offset((i1 + k) as isize) == *ha2.offset((i2 + k) as isize) {
                            if k == (*xenv).snake_cnt - 1 as ::core::ffi::c_long {
                                best = v;
                                (*spl).i1 = i1;
                                (*spl).i2 = i2;
                                break;
                            } else {
                                k += 1;
                            }
                        }
                    }
                    d -= 2 as ::core::ffi::c_long;
                }
                if best > 0 as ::core::ffi::c_long {
                    (*spl).min_lo = 0 as ::core::ffi::c_int;
                    (*spl).min_hi = 1 as ::core::ffi::c_int;
                    return ec;
                }
            }
            if ec >= (*xenv).mxcost {
                let mut fbest: ::core::ffi::c_long = 0;
                let mut fbest1: ::core::ffi::c_long = 0;
                let mut bbest: ::core::ffi::c_long = 0;
                let mut bbest1: ::core::ffi::c_long = 0;
                fbest1 = -1 as ::core::ffi::c_long;
                fbest = fbest1;
                d = fmax;
                while d >= fmin {
                    i1 = if *kvdf.offset(d as isize) < lim1 {
                        *kvdf.offset(d as isize)
                    } else {
                        lim1
                    };
                    i2 = i1 - d;
                    if lim2 < i2 {
                        i1 = lim2 + d;
                        i2 = lim2;
                    }
                    if fbest < i1 + i2 {
                        fbest = i1 + i2;
                        fbest1 = i1;
                    }
                    d -= 2 as ::core::ffi::c_long;
                }
                bbest1 = XDL_LINE_MAX;
                bbest = bbest1;
                d = bmax;
                while d >= bmin {
                    i1 = if off1 > *kvdb.offset(d as isize) {
                        off1
                    } else {
                        *kvdb.offset(d as isize)
                    };
                    i2 = i1 - d;
                    if i2 < off2 {
                        i1 = off2 + d;
                        i2 = off2;
                    }
                    if i1 + i2 < bbest {
                        bbest = i1 + i2;
                        bbest1 = i1;
                    }
                    d -= 2 as ::core::ffi::c_long;
                }
                if lim1 + lim2 - bbest < fbest - (off1 + off2) {
                    (*spl).i1 = fbest1;
                    (*spl).i2 = fbest - fbest1;
                    (*spl).min_lo = 1 as ::core::ffi::c_int;
                    (*spl).min_hi = 0 as ::core::ffi::c_int;
                } else {
                    (*spl).i1 = bbest1;
                    (*spl).i2 = bbest - bbest1;
                    (*spl).min_lo = 0 as ::core::ffi::c_int;
                    (*spl).min_hi = 1 as ::core::ffi::c_int;
                }
                return ec;
            }
        }
        ec += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn xdl_recs_cmp(
    mut dd1: *mut diffdata_t,
    mut off1: ::core::ffi::c_long,
    mut lim1: ::core::ffi::c_long,
    mut dd2: *mut diffdata_t,
    mut off2: ::core::ffi::c_long,
    mut lim2: ::core::ffi::c_long,
    mut kvdf: *mut ::core::ffi::c_long,
    mut kvdb: *mut ::core::ffi::c_long,
    mut need_min: ::core::ffi::c_int,
    mut xenv: *mut xdalgoenv_t,
) -> ::core::ffi::c_int {
    let mut ha1: *const ::core::ffi::c_ulong = (*dd1).ha;
    let mut ha2: *const ::core::ffi::c_ulong = (*dd2).ha;
    while off1 < lim1 && off2 < lim2 && *ha1.offset(off1 as isize) == *ha2.offset(off2 as isize) {
        off1 += 1;
        off2 += 1;
    }
    while off1 < lim1
        && off2 < lim2
        && *ha1.offset((lim1 - 1 as ::core::ffi::c_long) as isize)
            == *ha2.offset((lim2 - 1 as ::core::ffi::c_long) as isize)
    {
        lim1 -= 1;
        lim2 -= 1;
    }
    if off1 == lim1 {
        let mut rchg2: *mut ::core::ffi::c_char = (*dd2).rchg;
        let mut rindex2: *mut ::core::ffi::c_long = (*dd2).rindex;
        while off2 < lim2 {
            *rchg2.offset(*rindex2.offset(off2 as isize) as isize) = 1 as ::core::ffi::c_char;
            off2 += 1;
        }
    } else if off2 == lim2 {
        let mut rchg1: *mut ::core::ffi::c_char = (*dd1).rchg;
        let mut rindex1: *mut ::core::ffi::c_long = (*dd1).rindex;
        while off1 < lim1 {
            *rchg1.offset(*rindex1.offset(off1 as isize) as isize) = 1 as ::core::ffi::c_char;
            off1 += 1;
        }
    } else {
        let mut spl: xdpsplit_t = xdpsplit_t {
            i1: 0,
            i2: 0,
            min_lo: 0,
            min_hi: 0,
        };
        spl.i2 = 0 as ::core::ffi::c_long;
        spl.i1 = spl.i2;
        if xdl_split(
            ha1,
            off1,
            lim1,
            ha2,
            off2,
            lim2,
            kvdf,
            kvdb,
            need_min,
            &raw mut spl,
            xenv,
        ) < 0 as ::core::ffi::c_long
        {
            return -1 as ::core::ffi::c_int;
        }
        if xdl_recs_cmp(
            dd1, off1, spl.i1, dd2, off2, spl.i2, kvdf, kvdb, spl.min_lo, xenv,
        ) < 0 as ::core::ffi::c_int
            || xdl_recs_cmp(
                dd1, spl.i1, lim1, dd2, spl.i2, lim2, kvdf, kvdb, spl.min_hi, xenv,
            ) < 0 as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_do_diff(
    mut mf1: *mut mmfile_t,
    mut mf2: *mut mmfile_t,
    mut xpp: *const xpparam_t,
    mut xe: *mut xdfenv_t,
) -> ::core::ffi::c_int {
    let mut ndiags: ::core::ffi::c_long = 0;
    let mut kvd: *mut ::core::ffi::c_long = ::core::ptr::null_mut::<::core::ffi::c_long>();
    let mut kvdf: *mut ::core::ffi::c_long = ::core::ptr::null_mut::<::core::ffi::c_long>();
    let mut kvdb: *mut ::core::ffi::c_long = ::core::ptr::null_mut::<::core::ffi::c_long>();
    let mut xenv: xdalgoenv_t = xdalgoenv_t {
        mxcost: 0,
        snake_cnt: 0,
        heur_min: 0,
    };
    let mut dd1: diffdata_t = diffdata_t {
        nrec: 0,
        ha: ::core::ptr::null::<::core::ffi::c_ulong>(),
        rindex: ::core::ptr::null_mut::<::core::ffi::c_long>(),
        rchg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut dd2: diffdata_t = diffdata_t {
        nrec: 0,
        ha: ::core::ptr::null::<::core::ffi::c_ulong>(),
        rindex: ::core::ptr::null_mut::<::core::ffi::c_long>(),
        rchg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
        == XDF_PATIENCE_DIFF as ::core::ffi::c_ulong
    {
        return xdl_do_patience_diff(mf1, mf2, xpp, xe);
    }
    if (*xpp).flags & XDF_DIFF_ALGORITHM_MASK as ::core::ffi::c_ulong
        == XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong
    {
        return xdl_do_histogram_diff(mf1, mf2, xpp, xe);
    }
    if xdl_prepare_env(mf1, mf2, xpp, xe) < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    ndiags = (*xe).xdf1.nreff + (*xe).xdf2.nreff + 3 as ::core::ffi::c_long;
    kvd = xmalloc(
        ((2 as ::core::ffi::c_long * ndiags + 2 as ::core::ffi::c_long) as size_t)
            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_long>()),
    ) as *mut ::core::ffi::c_long;
    if kvd.is_null() {
        xdl_free_env(xe);
        return -1 as ::core::ffi::c_int;
    }
    kvdf = kvd;
    kvdb = kvdf.offset(ndiags as isize);
    kvdf = kvdf.offset(((*xe).xdf2.nreff + 1 as ::core::ffi::c_long) as isize);
    kvdb = kvdb.offset(((*xe).xdf2.nreff + 1 as ::core::ffi::c_long) as isize);
    xenv.mxcost = xdl_bogosqrt(ndiags);
    if xenv.mxcost < XDL_MAX_COST_MIN as ::core::ffi::c_long {
        xenv.mxcost = XDL_MAX_COST_MIN as ::core::ffi::c_long;
    }
    xenv.snake_cnt = XDL_SNAKE_CNT as ::core::ffi::c_long;
    xenv.heur_min = XDL_HEUR_MIN_COST as ::core::ffi::c_long;
    dd1.nrec = (*xe).xdf1.nreff;
    dd1.ha = (*xe).xdf1.ha;
    dd1.rchg = (*xe).xdf1.rchg;
    dd1.rindex = (*xe).xdf1.rindex;
    dd2.nrec = (*xe).xdf2.nreff;
    dd2.ha = (*xe).xdf2.ha;
    dd2.rchg = (*xe).xdf2.rchg;
    dd2.rindex = (*xe).xdf2.rindex;
    if xdl_recs_cmp(
        &raw mut dd1,
        0 as ::core::ffi::c_long,
        dd1.nrec,
        &raw mut dd2,
        0 as ::core::ffi::c_long,
        dd2.nrec,
        kvdf,
        kvdb,
        ((*xpp).flags & XDF_NEED_MINIMAL as ::core::ffi::c_ulong != 0 as ::core::ffi::c_ulong)
            as ::core::ffi::c_int,
        &raw mut xenv,
    ) < 0 as ::core::ffi::c_int
    {
        xfree(kvd as *mut ::core::ffi::c_void);
        xdl_free_env(xe);
        return -1 as ::core::ffi::c_int;
    }
    xfree(kvd as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_add_change(
    mut xscr: *mut xdchange_t,
    mut i1: ::core::ffi::c_long,
    mut i2: ::core::ffi::c_long,
    mut chg1: ::core::ffi::c_long,
    mut chg2: ::core::ffi::c_long,
) -> *mut xdchange_t {
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    xch = xmalloc(::core::mem::size_of::<xdchange_t>()) as *mut xdchange_t;
    if xch.is_null() {
        return ::core::ptr::null_mut::<xdchange_t>();
    }
    (*xch).next = xscr as *mut s_xdchange;
    (*xch).i1 = i1;
    (*xch).i2 = i2;
    (*xch).chg1 = chg1;
    (*xch).chg2 = chg2;
    (*xch).ignore = 0 as ::core::ffi::c_int;
    return xch;
}
unsafe extern "C" fn recs_match(
    mut rec1: *mut xrecord_t,
    mut rec2: *mut xrecord_t,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    return ((*rec1).ha == (*rec2).ha
        && xdl_recmatch((*rec1).ptr, (*rec1).size, (*rec2).ptr, (*rec2).size, flags) != 0)
        as ::core::ffi::c_int;
}
pub const MAX_INDENT: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
unsafe extern "C" fn xget_indent(mut rec: *mut xrecord_t) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_long = 0;
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_long;
    while i < (*rec).size {
        let mut c: ::core::ffi::c_char = *(*rec).ptr.offset(i as isize);
        if *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
        {
            return ret;
        } else if c as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            ret += 1 as ::core::ffi::c_int;
        } else if c as ::core::ffi::c_int == '\t' as ::core::ffi::c_int {
            ret += 8 as ::core::ffi::c_int - ret % 8 as ::core::ffi::c_int;
        }
        if ret >= MAX_INDENT {
            return MAX_INDENT;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub const MAX_BLANKS: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
unsafe extern "C" fn measure_split(
    mut xdf: *const xdfile_t,
    mut split: ::core::ffi::c_long,
    mut m: *mut split_measurement,
) {
    let mut i: ::core::ffi::c_long = 0;
    if split >= (*xdf).nrec {
        (*m).end_of_file = 1 as ::core::ffi::c_int;
        (*m).indent = -1 as ::core::ffi::c_int;
    } else {
        (*m).end_of_file = 0 as ::core::ffi::c_int;
        (*m).indent = xget_indent(*(*xdf).recs.offset(split as isize));
    }
    (*m).pre_blank = 0 as ::core::ffi::c_int;
    (*m).pre_indent = -1 as ::core::ffi::c_int;
    i = split - 1 as ::core::ffi::c_long;
    while i >= 0 as ::core::ffi::c_long {
        (*m).pre_indent = xget_indent(*(*xdf).recs.offset(i as isize));
        if (*m).pre_indent != -1 as ::core::ffi::c_int {
            break;
        }
        (*m).pre_blank += 1 as ::core::ffi::c_int;
        if (*m).pre_blank == MAX_BLANKS {
            (*m).pre_indent = 0 as ::core::ffi::c_int;
            break;
        } else {
            i -= 1;
        }
    }
    (*m).post_blank = 0 as ::core::ffi::c_int;
    (*m).post_indent = -1 as ::core::ffi::c_int;
    i = split + 1 as ::core::ffi::c_long;
    while i < (*xdf).nrec {
        (*m).post_indent = xget_indent(*(*xdf).recs.offset(i as isize));
        if (*m).post_indent != -1 as ::core::ffi::c_int {
            break;
        }
        (*m).post_blank += 1 as ::core::ffi::c_int;
        if (*m).post_blank == MAX_BLANKS {
            (*m).post_indent = 0 as ::core::ffi::c_int;
            break;
        } else {
            i += 1;
        }
    }
}
pub const START_OF_FILE_PENALTY: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const END_OF_FILE_PENALTY: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const TOTAL_BLANK_WEIGHT: ::core::ffi::c_int = -30 as ::core::ffi::c_int;
pub const POST_BLANK_WEIGHT: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const RELATIVE_INDENT_PENALTY: ::core::ffi::c_int = -4 as ::core::ffi::c_int;
pub const RELATIVE_INDENT_WITH_BLANK_PENALTY: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const RELATIVE_OUTDENT_PENALTY: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const RELATIVE_OUTDENT_WITH_BLANK_PENALTY: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const RELATIVE_DEDENT_PENALTY: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const RELATIVE_DEDENT_WITH_BLANK_PENALTY: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const INDENT_WEIGHT: ::core::ffi::c_int = 60 as ::core::ffi::c_int;
pub const INDENT_HEURISTIC_MAX_SLIDING: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
unsafe extern "C" fn score_add_split(mut m: *const split_measurement, mut s: *mut split_score) {
    let mut post_blank: ::core::ffi::c_int = 0;
    let mut total_blank: ::core::ffi::c_int = 0;
    let mut indent: ::core::ffi::c_int = 0;
    let mut any_blanks: ::core::ffi::c_int = 0;
    if (*m).pre_indent == -1 as ::core::ffi::c_int && (*m).pre_blank == 0 as ::core::ffi::c_int {
        (*s).penalty += START_OF_FILE_PENALTY;
    }
    if (*m).end_of_file != 0 {
        (*s).penalty += END_OF_FILE_PENALTY;
    }
    post_blank = if (*m).indent == -1 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int + (*m).post_blank
    } else {
        0 as ::core::ffi::c_int
    };
    total_blank = (*m).pre_blank + post_blank;
    (*s).penalty += TOTAL_BLANK_WEIGHT * total_blank;
    (*s).penalty += POST_BLANK_WEIGHT * post_blank;
    if (*m).indent != -1 as ::core::ffi::c_int {
        indent = (*m).indent;
    } else {
        indent = (*m).post_indent;
    }
    any_blanks = (total_blank != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    (*s).effective_indent += indent;
    if indent != -1 as ::core::ffi::c_int {
        if (*m).pre_indent != -1 as ::core::ffi::c_int {
            if indent > (*m).pre_indent {
                (*s).penalty += if any_blanks != 0 {
                    RELATIVE_INDENT_WITH_BLANK_PENALTY
                } else {
                    RELATIVE_INDENT_PENALTY
                };
            } else if indent != (*m).pre_indent {
                if (*m).post_indent != -1 as ::core::ffi::c_int && (*m).post_indent > indent {
                    (*s).penalty += if any_blanks != 0 {
                        RELATIVE_OUTDENT_WITH_BLANK_PENALTY
                    } else {
                        RELATIVE_OUTDENT_PENALTY
                    };
                } else {
                    (*s).penalty += if any_blanks != 0 {
                        RELATIVE_DEDENT_WITH_BLANK_PENALTY
                    } else {
                        RELATIVE_DEDENT_PENALTY
                    };
                }
            }
        }
    }
}
unsafe extern "C" fn score_cmp(
    mut s1: *mut split_score,
    mut s2: *mut split_score,
) -> ::core::ffi::c_int {
    let mut cmp_indents: ::core::ffi::c_int = ((*s1).effective_indent > (*s2).effective_indent)
        as ::core::ffi::c_int
        - ((*s1).effective_indent < (*s2).effective_indent) as ::core::ffi::c_int;
    return INDENT_WEIGHT * cmp_indents + ((*s1).penalty - (*s2).penalty);
}
unsafe extern "C" fn group_init(mut xdf: *mut xdfile_t, mut g: *mut xdlgroup) {
    (*g).end = 0 as ::core::ffi::c_long;
    (*g).start = (*g).end;
    while *(*xdf).rchg.offset((*g).end as isize) != 0 {
        (*g).end += 1;
    }
}
#[inline]
unsafe extern "C" fn group_next(
    mut xdf: *mut xdfile_t,
    mut g: *mut xdlgroup,
) -> ::core::ffi::c_int {
    if (*g).end == (*xdf).nrec {
        return -1 as ::core::ffi::c_int;
    }
    (*g).start = (*g).end + 1 as ::core::ffi::c_long;
    (*g).end = (*g).start;
    while *(*xdf).rchg.offset((*g).end as isize) != 0 {
        (*g).end += 1;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn group_previous(
    mut xdf: *mut xdfile_t,
    mut g: *mut xdlgroup,
) -> ::core::ffi::c_int {
    if (*g).start == 0 as ::core::ffi::c_long {
        return -1 as ::core::ffi::c_int;
    }
    (*g).end = (*g).start - 1 as ::core::ffi::c_long;
    (*g).start = (*g).end;
    while *(*xdf)
        .rchg
        .offset(((*g).start - 1 as ::core::ffi::c_long) as isize)
        != 0
    {
        (*g).start -= 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn group_slide_down(
    mut xdf: *mut xdfile_t,
    mut g: *mut xdlgroup,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    if (*g).end < (*xdf).nrec
        && recs_match(
            *(*xdf).recs.offset((*g).start as isize),
            *(*xdf).recs.offset((*g).end as isize),
            flags,
        ) != 0
    {
        let c2rust_fresh0 = (*g).start;
        (*g).start = (*g).start + 1;
        *(*xdf).rchg.offset(c2rust_fresh0 as isize) = 0 as ::core::ffi::c_char;
        let c2rust_fresh1 = (*g).end;
        (*g).end = (*g).end + 1;
        *(*xdf).rchg.offset(c2rust_fresh1 as isize) = 1 as ::core::ffi::c_char;
        while *(*xdf).rchg.offset((*g).end as isize) != 0 {
            (*g).end += 1;
        }
        return 0 as ::core::ffi::c_int;
    } else {
        return -1 as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn group_slide_up(
    mut xdf: *mut xdfile_t,
    mut g: *mut xdlgroup,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    if (*g).start > 0 as ::core::ffi::c_long
        && recs_match(
            *(*xdf)
                .recs
                .offset(((*g).start - 1 as ::core::ffi::c_long) as isize),
            *(*xdf)
                .recs
                .offset(((*g).end - 1 as ::core::ffi::c_long) as isize),
            flags,
        ) != 0
    {
        (*g).start -= 1;
        *(*xdf).rchg.offset((*g).start as isize) = 1 as ::core::ffi::c_char;
        (*g).end -= 1;
        *(*xdf).rchg.offset((*g).end as isize) = 0 as ::core::ffi::c_char;
        while *(*xdf)
            .rchg
            .offset(((*g).start - 1 as ::core::ffi::c_long) as isize)
            != 0
        {
            (*g).start -= 1;
        }
        return 0 as ::core::ffi::c_int;
    } else {
        return -1 as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn xdl_bug(mut msg: *const ::core::ffi::c_char) {
    fprintf(
        stderr,
        b"BUG: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        msg,
    );
    exit(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn xdl_change_compact(
    mut xdf: *mut xdfile_t,
    mut xdfo: *mut xdfile_t,
    mut flags: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut g: xdlgroup = xdlgroup { start: 0, end: 0 };
    let mut go: xdlgroup = xdlgroup { start: 0, end: 0 };
    let mut earliest_end: ::core::ffi::c_long = 0;
    let mut end_matching_other: ::core::ffi::c_long = 0;
    let mut groupsize: ::core::ffi::c_long = 0;
    group_init(xdf, &raw mut g);
    group_init(xdfo, &raw mut go);
    loop {
        if g.end != g.start {
            loop {
                groupsize = g.end - g.start;
                end_matching_other = -1 as ::core::ffi::c_long;
                while group_slide_up(xdf, &raw mut g, flags) == 0 {
                    if group_previous(xdfo, &raw mut go) != 0 {
                        xdl_bug(b"group sync broken sliding up\0".as_ptr()
                            as *const ::core::ffi::c_char);
                    }
                }
                earliest_end = g.end;
                if go.end > go.start {
                    end_matching_other = g.end;
                }
                while group_slide_down(xdf, &raw mut g, flags) == 0 {
                    if group_next(xdfo, &raw mut go) != 0 {
                        xdl_bug(b"group sync broken sliding down\0".as_ptr()
                            as *const ::core::ffi::c_char);
                    }
                    if go.end > go.start {
                        end_matching_other = g.end;
                    }
                }
                if groupsize == g.end - g.start {
                    break;
                }
            }
            if g.end != earliest_end {
                if end_matching_other != -1 as ::core::ffi::c_long {
                    while go.end == go.start {
                        if group_slide_up(xdf, &raw mut g, flags) != 0 {
                            xdl_bug(b"match disappeared\0".as_ptr() as *const ::core::ffi::c_char);
                        }
                        if group_previous(xdfo, &raw mut go) != 0 {
                            xdl_bug(b"group sync broken sliding to match\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        }
                    }
                } else if flags & XDF_INDENT_HEURISTIC as ::core::ffi::c_long != 0 {
                    let mut shift: ::core::ffi::c_long = 0;
                    let mut best_shift: ::core::ffi::c_long = -1 as ::core::ffi::c_long;
                    let mut best_score: split_score = split_score {
                        effective_indent: 0,
                        penalty: 0,
                    };
                    shift = earliest_end;
                    if g.end - groupsize - 1 as ::core::ffi::c_long > shift {
                        shift = g.end - groupsize - 1 as ::core::ffi::c_long;
                    }
                    if g.end - INDENT_HEURISTIC_MAX_SLIDING as ::core::ffi::c_long > shift {
                        shift = g.end - INDENT_HEURISTIC_MAX_SLIDING as ::core::ffi::c_long;
                    }
                    while shift <= g.end {
                        let mut m: split_measurement = split_measurement {
                            end_of_file: 0,
                            indent: 0,
                            pre_blank: 0,
                            pre_indent: 0,
                            post_blank: 0,
                            post_indent: 0,
                        };
                        let mut score: split_score = split_score {
                            effective_indent: 0 as ::core::ffi::c_int,
                            penalty: 0 as ::core::ffi::c_int,
                        };
                        measure_split(xdf, shift, &raw mut m);
                        score_add_split(&raw mut m, &raw mut score);
                        measure_split(xdf, shift - groupsize, &raw mut m);
                        score_add_split(&raw mut m, &raw mut score);
                        if best_shift == -1 as ::core::ffi::c_long
                            || score_cmp(&raw mut score, &raw mut best_score)
                                <= 0 as ::core::ffi::c_int
                        {
                            best_score.effective_indent = score.effective_indent;
                            best_score.penalty = score.penalty;
                            best_shift = shift;
                        }
                        shift += 1;
                    }
                    while g.end > best_shift {
                        if group_slide_up(xdf, &raw mut g, flags) != 0 {
                            xdl_bug(
                                b"best shift unreached\0".as_ptr() as *const ::core::ffi::c_char
                            );
                        }
                        if group_previous(xdfo, &raw mut go) != 0 {
                            xdl_bug(b"group sync broken sliding to blank line\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        }
                    }
                }
            }
        }
        if group_next(xdf, &raw mut g) != 0 {
            break;
        }
        if group_next(xdfo, &raw mut go) != 0 {
            xdl_bug(
                b"group sync broken moving to next group\0".as_ptr() as *const ::core::ffi::c_char
            );
        }
    }
    if group_next(xdfo, &raw mut go) == 0 {
        xdl_bug(b"group sync broken at end of file\0".as_ptr() as *const ::core::ffi::c_char);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_build_script(
    mut xe: *mut xdfenv_t,
    mut xscr: *mut *mut xdchange_t,
) -> ::core::ffi::c_int {
    let mut cscr: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut rchg1: *mut ::core::ffi::c_char = (*xe).xdf1.rchg;
    let mut rchg2: *mut ::core::ffi::c_char = (*xe).xdf2.rchg;
    let mut i1: ::core::ffi::c_long = 0;
    let mut i2: ::core::ffi::c_long = 0;
    let mut l1: ::core::ffi::c_long = 0;
    let mut l2: ::core::ffi::c_long = 0;
    i1 = (*xe).xdf1.nrec;
    i2 = (*xe).xdf2.nrec;
    while i1 >= 0 as ::core::ffi::c_long || i2 >= 0 as ::core::ffi::c_long {
        if *rchg1.offset((i1 - 1 as ::core::ffi::c_long) as isize) as ::core::ffi::c_int != 0
            || *rchg2.offset((i2 - 1 as ::core::ffi::c_long) as isize) as ::core::ffi::c_int != 0
        {
            l1 = i1;
            while *rchg1.offset((i1 - 1 as ::core::ffi::c_long) as isize) != 0 {
                i1 -= 1;
            }
            l2 = i2;
            while *rchg2.offset((i2 - 1 as ::core::ffi::c_long) as isize) != 0 {
                i2 -= 1;
            }
            xch = xdl_add_change(cscr, i1, i2, l1 - i1, l2 - i2);
            if xch.is_null() {
                xdl_free_script(cscr);
                return -1 as ::core::ffi::c_int;
            }
            cscr = xch;
        }
        i1 -= 1;
        i2 -= 1;
    }
    *xscr = cscr;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xdl_free_script(mut xscr: *mut xdchange_t) {
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    loop {
        xch = xscr;
        if xch.is_null() {
            break;
        }
        xscr = (*xscr).next as *mut xdchange_t;
        xfree(xch as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn xdl_call_hunk_func(
    mut _xe: *mut xdfenv_t,
    mut xscr: *mut xdchange_t,
    mut ecb: *mut xdemitcb_t,
    mut xecfg: *const xdemitconf_t,
) -> ::core::ffi::c_int {
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut xche: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    xch = xscr;
    while !xch.is_null() {
        xche = xdl_get_hunk(&raw mut xch, xecfg);
        if xch.is_null() {
            break;
        }
        if (*xecfg).hunk_func.expect("non-null function pointer")(
            (*xch).i1 as ::core::ffi::c_int,
            ((*xche).i1 + (*xche).chg1 - (*xch).i1) as ::core::ffi::c_int,
            (*xch).i2 as ::core::ffi::c_int,
            ((*xche).i2 + (*xche).chg2 - (*xch).i2) as ::core::ffi::c_int,
            (*ecb).priv_0,
        ) < 0 as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
        xch = (*xche).next as *mut xdchange_t;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn xdl_mark_ignorable_lines(
    mut xscr: *mut xdchange_t,
    mut xe: *mut xdfenv_t,
    mut flags: ::core::ffi::c_long,
) {
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    xch = xscr;
    while !xch.is_null() {
        let mut ignore: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut rec: *mut *mut xrecord_t = ::core::ptr::null_mut::<*mut xrecord_t>();
        let mut i: ::core::ffi::c_long = 0;
        rec = (*xe).xdf1.recs.offset((*xch).i1 as isize);
        i = 0 as ::core::ffi::c_long;
        while i < (*xch).chg1 && ignore != 0 {
            ignore = xdl_blankline(
                (**rec.offset(i as isize)).ptr,
                (**rec.offset(i as isize)).size,
                flags,
            );
            i += 1;
        }
        rec = (*xe).xdf2.recs.offset((*xch).i2 as isize);
        i = 0 as ::core::ffi::c_long;
        while i < (*xch).chg2 && ignore != 0 {
            ignore = xdl_blankline(
                (**rec.offset(i as isize)).ptr,
                (**rec.offset(i as isize)).size,
                flags,
            );
            i += 1;
        }
        (*xch).ignore = ignore;
        xch = (*xch).next as *mut xdchange_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn xdl_diff(
    mut mf1: *mut mmfile_t,
    mut mf2: *mut mmfile_t,
    mut xpp: *const xpparam_t,
    mut xecfg: *const xdemitconf_t,
    mut ecb: *mut xdemitcb_t,
) -> ::core::ffi::c_int {
    let mut xscr: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut xe: xdfenv_t = xdfenv_t {
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
    let mut ef: emit_func_t = if (*xecfg).hunk_func.is_some() {
        Some(
            xdl_call_hunk_func
                as unsafe extern "C" fn(
                    *mut xdfenv_t,
                    *mut xdchange_t,
                    *mut xdemitcb_t,
                    *const xdemitconf_t,
                ) -> ::core::ffi::c_int,
        )
    } else {
        Some(
            xdl_emit_diff
                as unsafe extern "C" fn(
                    *mut xdfenv_t,
                    *mut xdchange_t,
                    *mut xdemitcb_t,
                    *const xdemitconf_t,
                ) -> ::core::ffi::c_int,
        )
    };
    if xdl_do_diff(mf1, mf2, xpp, &raw mut xe) < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if xdl_change_compact(
        &raw mut xe.xdf1,
        &raw mut xe.xdf2,
        (*xpp).flags as ::core::ffi::c_long,
    ) < 0 as ::core::ffi::c_int
        || xdl_change_compact(
            &raw mut xe.xdf2,
            &raw mut xe.xdf1,
            (*xpp).flags as ::core::ffi::c_long,
        ) < 0 as ::core::ffi::c_int
        || xdl_build_script(&raw mut xe, &raw mut xscr) < 0 as ::core::ffi::c_int
    {
        xdl_free_env(&raw mut xe);
        return -1 as ::core::ffi::c_int;
    }
    if !xscr.is_null() {
        if (*xpp).flags & XDF_IGNORE_BLANK_LINES as ::core::ffi::c_ulong != 0 {
            xdl_mark_ignorable_lines(xscr, &raw mut xe, (*xpp).flags as ::core::ffi::c_long);
        }
        if ef.expect("non-null function pointer")(&raw mut xe, xscr, ecb, xecfg)
            < 0 as ::core::ffi::c_int
        {
            xdl_free_script(xscr);
            xdl_free_env(&raw mut xe);
            return -1 as ::core::ffi::c_int;
        }
        xdl_free_script(xscr);
    }
    xdl_free_env(&raw mut xe);
    return 0 as ::core::ffi::c_int;
}
pub const __CHAR_BIT__: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CHAR_BIT: ::core::ffi::c_int = __CHAR_BIT__;
