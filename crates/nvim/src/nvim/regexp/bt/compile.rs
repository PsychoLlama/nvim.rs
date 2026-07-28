//! Emitting the program: the node writer, the `regtail`/`reginsert`
//! surgery on it and the compile-time state.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn regcomp_start(
    mut expr: *mut uint8_t,
    mut re_flags: ::core::ffi::c_int,
) {
    initchr(expr as *mut ::core::ffi::c_char);
    if re_flags & RE_MAGIC != 0 {
        reg_magic.set(MAGIC_ON);
    } else {
        reg_magic.set(MAGIC_OFF);
    }
    reg_string.set(re_flags & RE_STRING);
    reg_strict.set(re_flags & RE_STRICT);
    get_cpo_flags();
    num_complex_braces.set(0 as ::core::ffi::c_int);
    regnpar.set(1 as ::core::ffi::c_int);
    memset(
        had_endbrace.ptr() as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint8_t; 10]>(),
    );
    regnzpar.set(1 as ::core::ffi::c_int);
    re_has_z.set(0 as ::core::ffi::c_int);
    regsize.set(0 as ::core::ffi::c_long as int64_t);
    reg_toolong.set(false_0);
    regflags.set(0 as ::core::ffi::c_uint);
    had_eol.set(false_0);
}
pub(crate) unsafe extern "C" fn use_multibytecode(mut c: ::core::ffi::c_int) -> bool {
    return utf_char2len(c) > 1 as ::core::ffi::c_int
        && (re_multi_type(peekchr()) != NOT_MULTI
            || utf_iscomposing_legacy(c) as ::core::ffi::c_int != 0);
}
pub(crate) unsafe extern "C" fn regc(mut b: ::core::ffi::c_int) {
    if regcode.get() == JUST_CALC_SIZE {
        (*regsize.ptr()) += 1;
    } else {
        let c2rust_fresh1495 = regcode.get();
        regcode.set((*regcode.ptr()).offset(1));
        *c2rust_fresh1495 = b as uint8_t;
    };
}
pub(crate) unsafe extern "C" fn regmbc(mut c: ::core::ffi::c_int) {
    if regcode.get() == JUST_CALC_SIZE {
        (*regsize.ptr()) += utf_char2len(c) as int64_t;
    } else {
        regcode.set(
            (*regcode.ptr())
                .offset(utf_char2bytes(c, regcode.get() as *mut ::core::ffi::c_char) as isize),
        );
    };
}
pub(crate) unsafe extern "C" fn regnode(mut op: ::core::ffi::c_int) -> *mut uint8_t {
    let mut ret: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    ret = regcode.get();
    if ret == JUST_CALC_SIZE {
        (*regsize.ptr()) += 3 as int64_t;
    } else {
        let c2rust_fresh1472 = regcode.get();
        regcode.set((*regcode.ptr()).offset(1));
        *c2rust_fresh1472 = op as uint8_t;
        let c2rust_fresh1473 = regcode.get();
        regcode.set((*regcode.ptr()).offset(1));
        *c2rust_fresh1473 = NUL as uint8_t;
        let c2rust_fresh1474 = regcode.get();
        regcode.set((*regcode.ptr()).offset(1));
        *c2rust_fresh1474 = NUL as uint8_t;
    }
    return ret;
}
pub(crate) unsafe extern "C" fn re_put_uint32(
    mut p: *mut uint8_t,
    mut val: uint32_t,
) -> *mut uint8_t {
    let c2rust_fresh1480 = p;
    p = p.offset(1);
    *c2rust_fresh1480 = (val >> 24 as ::core::ffi::c_int & 0o377 as uint32_t) as uint8_t;
    let c2rust_fresh1481 = p;
    p = p.offset(1);
    *c2rust_fresh1481 = (val >> 16 as ::core::ffi::c_int & 0o377 as uint32_t) as uint8_t;
    let c2rust_fresh1482 = p;
    p = p.offset(1);
    *c2rust_fresh1482 = (val >> 8 as ::core::ffi::c_int & 0o377 as uint32_t) as uint8_t;
    let c2rust_fresh1483 = p;
    p = p.offset(1);
    *c2rust_fresh1483 = (val & 0o377 as uint32_t) as uint8_t;
    return p;
}
pub(crate) unsafe extern "C" fn regnext(mut p: *mut uint8_t) -> *mut uint8_t {
    let mut offset: ::core::ffi::c_int = 0;
    if p == JUST_CALC_SIZE || reg_toolong.get() != 0 {
        return ::core::ptr::null_mut::<uint8_t>();
    }
    offset = ((*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & 0o377 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0o377 as ::core::ffi::c_int);
    if offset == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<uint8_t>();
    }
    if *p as ::core::ffi::c_int == BACK {
        return p.offset(-(offset as isize));
    } else {
        return p.offset(offset as isize);
    };
}
pub(crate) unsafe extern "C" fn regtail(mut p: *mut uint8_t, mut val: *const uint8_t) {
    let mut offset: ::core::ffi::c_int = 0;
    if p == JUST_CALC_SIZE {
        return;
    }
    let mut scan: *mut uint8_t = p;
    loop {
        let mut temp: *mut uint8_t = regnext(scan);
        if temp.is_null() {
            break;
        }
        scan = temp;
    }
    if *scan as ::core::ffi::c_int == BACK {
        offset = scan.offset_from(val) as ::core::ffi::c_int;
    } else {
        offset = val.offset_from(scan) as ::core::ffi::c_int;
    }
    if offset > 0xffff as ::core::ffi::c_int {
        reg_toolong.set(true_0);
    } else {
        *scan.offset(1 as ::core::ffi::c_int as isize) =
            (offset as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                & 0o377 as ::core::ffi::c_uint) as uint8_t;
        *scan.offset(2 as ::core::ffi::c_int as isize) =
            (offset & 0o377 as ::core::ffi::c_int) as uint8_t;
    };
}
pub(crate) unsafe extern "C" fn regoptail(mut p: *mut uint8_t, mut val: *mut uint8_t) {
    if p.is_null()
        || p == JUST_CALC_SIZE
        || *p as ::core::ffi::c_int != BRANCH
            && ((*p as ::core::ffi::c_int) < BRACE_COMPLEX
                || *p as ::core::ffi::c_int > BRACE_COMPLEX + 9 as ::core::ffi::c_int)
    {
        return;
    }
    regtail(p.offset(3 as ::core::ffi::c_int as isize), val);
}
pub(crate) unsafe extern "C" fn reginsert(mut op: ::core::ffi::c_int, mut opnd: *mut uint8_t) {
    let mut src: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut dst: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut place: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    if regcode.get() == JUST_CALC_SIZE {
        (*regsize.ptr()) += 3 as int64_t;
        return;
    }
    src = regcode.get();
    regcode.set((*regcode.ptr()).offset(3 as ::core::ffi::c_int as isize));
    dst = regcode.get();
    while src > opnd {
        src = src.offset(-1);
        dst = dst.offset(-1);
        *dst = *src;
    }
    place = opnd;
    let c2rust_fresh1475 = place;
    place = place.offset(1);
    *c2rust_fresh1475 = op as uint8_t;
    let c2rust_fresh1476 = place;
    place = place.offset(1);
    *c2rust_fresh1476 = NUL as uint8_t;
    *place = NUL as uint8_t;
}
pub(crate) unsafe extern "C" fn reginsert_nr(
    mut op: ::core::ffi::c_int,
    mut val: int64_t,
    mut opnd: *mut uint8_t,
) {
    let mut src: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut dst: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut place: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    if regcode.get() == JUST_CALC_SIZE {
        (*regsize.ptr()) += 7 as int64_t;
        return;
    }
    src = regcode.get();
    regcode.set((*regcode.ptr()).offset(7 as ::core::ffi::c_int as isize));
    dst = regcode.get();
    while src > opnd {
        src = src.offset(-1);
        dst = dst.offset(-1);
        *dst = *src;
    }
    place = opnd;
    let c2rust_fresh1484 = place;
    place = place.offset(1);
    *c2rust_fresh1484 = op as uint8_t;
    let c2rust_fresh1485 = place;
    place = place.offset(1);
    *c2rust_fresh1485 = NUL as uint8_t;
    let c2rust_fresh1486 = place;
    place = place.offset(1);
    *c2rust_fresh1486 = NUL as uint8_t;
    assert!(
        val >= 0 as int64_t && val as uintmax_t <= 4294967295 as uintmax_t,
        "val >= 0 && (uintmax_t)val <= UINT32_MAX\0\".as_ptr()
                    as *const ::core::ffi::c_char,
                b\"src/nvim/regexp.rs"
    );
    re_put_uint32(place, val as uint32_t);
}
pub(crate) unsafe extern "C" fn reginsert_limits(
    mut op: ::core::ffi::c_int,
    mut minval: int64_t,
    mut maxval: int64_t,
    mut opnd: *mut uint8_t,
) {
    let mut src: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut dst: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut place: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    if regcode.get() == JUST_CALC_SIZE {
        (*regsize.ptr()) += 11 as int64_t;
        return;
    }
    src = regcode.get();
    regcode.set((*regcode.ptr()).offset(11 as ::core::ffi::c_int as isize));
    dst = regcode.get();
    while src > opnd {
        src = src.offset(-1);
        dst = dst.offset(-1);
        *dst = *src;
    }
    place = opnd;
    let c2rust_fresh1477 = place;
    place = place.offset(1);
    *c2rust_fresh1477 = op as uint8_t;
    let c2rust_fresh1478 = place;
    place = place.offset(1);
    *c2rust_fresh1478 = NUL as uint8_t;
    let c2rust_fresh1479 = place;
    place = place.offset(1);
    *c2rust_fresh1479 = NUL as uint8_t;
    assert!(
        minval >= 0 as int64_t && minval as uintmax_t <= 4294967295 as uintmax_t,
        "minval >= 0 && (uintmax_t)minval <= UINT32_MAX\0\".as_ptr()
                    as *const ::core::ffi::c_char,
                b\"src/nvim/regexp.rs"
    );
    place = re_put_uint32(place, minval as uint32_t);
    assert!(
        maxval >= 0 as int64_t && maxval as uintmax_t <= 4294967295 as uintmax_t,
        "maxval >= 0 && (uintmax_t)maxval <= UINT32_MAX\0\".as_ptr()
                    as *const ::core::ffi::c_char,
                b\"src/nvim/regexp.rs"
    );
    place = re_put_uint32(place, maxval as uint32_t);
    regtail(opnd, place);
}
pub(crate) unsafe extern "C" fn seen_endbrace(
    mut refnum: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*had_endbrace.ptr())[refnum as usize] == 0 {
        let mut p: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
        p = regparse.get() as *mut uint8_t;
        while *p as ::core::ffi::c_int != NUL {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '@' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '<' as ::core::ffi::c_int
                && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '!' as ::core::ffi::c_int
                    || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '=' as ::core::ffi::c_int)
            {
                break;
            }
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            emsg(gettext(
                b"E65: Illegal back reference\0".as_ptr() as *const ::core::ffi::c_char
            ));
            rc_did_emsg.set(true_0 != 0);
            return false_0;
        }
    }
    return true_0;
}
