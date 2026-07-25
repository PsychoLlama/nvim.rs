//! Rust port of LibXDiff's xemit.c, as vendored by neovim v0.12.4.
//!
//! LibXDiff by Davide Libenzi ( File Differential Library )
//! Copyright (C) 2003 Davide Libenzi <davidel@xmailserver.org>
//!
//! This library is free software; you can redistribute it and/or modify it
//! under the terms of the GNU Lesser General Public License as published by
//! the Free Software Foundation; either version 2.1 of the License, or (at
//! your option) any later version (text: licenses/LGPL-2.1.txt).
//!
//! This library is distributed in the hope that it will be useful, but
//! WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser
//! General Public License for more details.

use crate::src::nvim::os::libc::strlen;
pub use crate::src::nvim::types::{
    chanode_t, chastore_t, find_func_t, mmbuffer_t, s_chanode, s_chastore, s_mmbuffer, s_xdchange,
    s_xdemitcb, s_xdemitconf, s_xdfenv, s_xdfile, s_xrecord, size_t, xdchange_t, xdemitcb_t,
    xdemitconf_t, xdfenv_t, xdfile_t, xdl_emit_hunk_consume_func_t, xrecord_t,
};
use crate::src::xdiff::xutils::{xdl_emit_diffrec, xdl_emit_hunk_hdr};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct func_line {
    pub len: ::core::ffi::c_long,
    pub buf: [::core::ffi::c_char; 80],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const XDL_EMIT_NO_HUNK_HDR: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
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
pub unsafe extern "C" fn xdl_get_hunk(
    mut xscr: *mut *mut xdchange_t,
    mut xecfg: *const xdemitconf_t,
) -> *mut xdchange_t {
    let mut xch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut xchp: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut lxch: *mut xdchange_t = ::core::ptr::null_mut::<xdchange_t>();
    let mut max_common: ::core::ffi::c_long =
        2 as ::core::ffi::c_long * (*xecfg).ctxlen + (*xecfg).interhunkctxlen;
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
                && (*xch).i1 + ignored as ::core::ffi::c_long - ((*lxch).i1 + (*lxch).chg1)
                    > max_common
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
    let mut func_line: func_line = func_line {
        len: 0,
        buf: [0; 80],
    };
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
