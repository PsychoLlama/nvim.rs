//! What counts as a paragraph, and when two lines belong to the same one.
//!
//! [`fmt_check_par`] answers "is this line *not* part of a paragraph" --
//! blank, or nothing but a comment leader, or the end of a block comment --
//! and [`same_leader`] whether two lines carry leaders that let them be
//! joined.  [`paragraph_start`] is the pair asked about one line.

use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::change::get_leader_len;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::indent::get_number_indent;
use crate::src::nvim::memline::{ml_get, ml_get_len};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::strings::xstrnsave;
use crate::src::nvim::textobject::startPS;
use crate::src::nvim::types::{colnr_T, linenr_T, size_t, uint8_t};

pub(crate) unsafe extern "C" fn fmt_check_par(
    mut lnum: linenr_T,
    mut leader_len: *mut ::core::ffi::c_int,
    mut leader_flags: *mut *mut ::core::ffi::c_char,
    mut do_comments: bool,
) -> ::core::ffi::c_int {
    let mut flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ml_get(lnum);
    if do_comments {
        *leader_len = get_leader_len(ptr, leader_flags, false, true);
    } else {
        *leader_len = 0 as ::core::ffi::c_int;
    }
    if *leader_len > 0 as ::core::ffi::c_int {
        flags = *leader_flags;
        while *flags as ::core::ffi::c_int != 0
            && *flags as ::core::ffi::c_int != ':' as ::core::ffi::c_int
            && *flags as ::core::ffi::c_int != COM_END
        {
            flags = flags.offset(1);
        }
    }
    return (*skipwhite(ptr.offset(*leader_len as isize)) as ::core::ffi::c_int == NUL
        || *leader_len > 0 as ::core::ffi::c_int && *flags as ::core::ffi::c_int == COM_END
        || startPS(lnum, NUL, false) as ::core::ffi::c_int != 0) as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn ends_in_white(mut lnum: linenr_T) -> bool {
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    if *s as ::core::ffi::c_int == NUL {
        return false;
    }
    let mut l: colnr_T = ml_get_len(lnum) - 1 as colnr_T;
    return ascii_iswhite(*s.offset(l as isize) as uint8_t as ::core::ffi::c_int);
}
pub(crate) unsafe extern "C" fn same_leader(
    mut lnum: linenr_T,
    mut leader1_len: ::core::ffi::c_int,
    mut leader1_flags: *mut ::core::ffi::c_char,
    mut leader2_len: ::core::ffi::c_int,
    mut leader2_flags: *mut ::core::ffi::c_char,
) -> bool {
    let mut idx1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if leader1_len == 0 as ::core::ffi::c_int {
        return leader2_len == 0 as ::core::ffi::c_int;
    }
    if !leader1_flags.is_null() {
        let mut p: *mut ::core::ffi::c_char = leader1_flags;
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == COM_FIRST {
                return leader2_len == 0 as ::core::ffi::c_int;
            }
            if *p as ::core::ffi::c_int == COM_END {
                return false;
            }
            if *p as ::core::ffi::c_int == COM_START {
                let mut line_len: ::core::ffi::c_int = ml_get_len(lnum);
                if line_len <= leader1_len {
                    return false;
                }
                if leader2_flags.is_null() || leader2_len == 0 as ::core::ffi::c_int {
                    return false;
                }
                p = leader2_flags;
                while *p as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == COM_MIDDLE {
                        return true;
                    }
                    p = p.offset(1);
                }
                return false;
            }
            p = p.offset(1);
        }
    }
    let mut line1: *mut ::core::ffi::c_char = xstrnsave(ml_get(lnum), ml_get_len(lnum) as size_t);
    idx1 = 0 as ::core::ffi::c_int;
    while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
        idx1 += 1;
    }
    let mut line2: *mut ::core::ffi::c_char = ml_get(lnum + 1 as linenr_T);
    idx2 = 0 as ::core::ffi::c_int;
    while idx2 < leader2_len {
        if !ascii_iswhite(*line2.offset(idx2 as isize) as ::core::ffi::c_int) {
            let c2rust_fresh0 = idx1;
            idx1 = idx1 + 1;
            if *line1.offset(c2rust_fresh0 as isize) as ::core::ffi::c_int
                != *line2.offset(idx2 as isize) as ::core::ffi::c_int
            {
                break;
            }
        } else {
            while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
                idx1 += 1;
            }
        }
        idx2 += 1;
    }
    xfree(line1 as *mut ::core::ffi::c_void);
    return idx2 == leader2_len && idx1 == leader1_len;
}
pub(crate) unsafe extern "C" fn paragraph_start(mut lnum: linenr_T) -> bool {
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lnum <= 1 as linenr_T {
        return true;
    }
    let mut p: *mut ::core::ffi::c_char = ml_get(lnum - 1 as linenr_T);
    if *p as ::core::ffi::c_int == NUL {
        return true;
    }
    let do_comments: bool = has_format_option(FO_Q_COMS);
    if fmt_check_par(
        lnum - 1 as linenr_T,
        &raw mut leader_len,
        &raw mut leader_flags,
        do_comments,
    ) != 0
    {
        return true;
    }
    if fmt_check_par(
        lnum,
        &raw mut next_leader_len,
        &raw mut next_leader_flags,
        do_comments,
    ) != 0
    {
        return true;
    }
    if has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0
        && !ends_in_white(lnum - 1 as linenr_T)
    {
        return true;
    }
    if has_format_option(FO_Q_NUMBER) as ::core::ffi::c_int != 0
        && get_number_indent(lnum) > 0 as ::core::ffi::c_int
    {
        return true;
    }
    if !same_leader(
        lnum - 1 as linenr_T,
        leader_len,
        leader_flags,
        next_leader_len,
        next_leader_flags,
    ) {
        return true;
    }
    return false;
}
