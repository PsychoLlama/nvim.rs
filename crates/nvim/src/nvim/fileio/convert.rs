//! Deciding what encoding a file is in.
//!
//! `'fileencodings'` is a list of guesses; `next_fenc` walks it and
//! `get_fio_flags` turns each name into the `FIO_*` bits that say whether Nvim
//! can do the conversion itself. `need_conversion` is the cheap test for "this
//! is already UTF-8, leave it alone", `check_for_bom` recognises a byte-order
//! mark and lets it override the guess, and `readfile_charconvert` hands the
//! whole file to the user's `'charconvert'` program when nothing else can read
//! it.
//!
//! This is the read side of `bufwrite::convert`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn next_fenc(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut alloced: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut r: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        *alloced = false_0 != 0;
        if **pp as ::core::ffi::c_int == NUL {
            *pp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        let mut p: *mut ::core::ffi::c_char = vim_strchr(*pp, ',' as ::core::ffi::c_int);
        if p.is_null() {
            r = enc_canonize(*pp);
            *pp = (*pp).offset(strlen(*pp) as isize);
        } else {
            r = xmemdupz(
                *pp as *const ::core::ffi::c_void,
                p.offset_from(*pp) as size_t,
            ) as *mut ::core::ffi::c_char;
            *pp = p.offset(1 as ::core::ffi::c_int as isize);
            p = enc_canonize(r);
            xfree(r as *mut ::core::ffi::c_void);
            r = p;
        }
        *alloced = true_0 != 0;
        return r;
    }
}

pub(crate) unsafe extern "C" fn readfile_charconvert(
    mut fname: *mut ::core::ffi::c_char,
    mut fenc: *mut ::core::ffi::c_char,
    mut fdp: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut errmsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tmpname: *mut ::core::ffi::c_char = vim_tempname();
        if tmpname.is_null() {
            errmsg = gettext(
                b"Can't find temp file for conversion\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else {
            close(*fdp);
            *fdp = -1 as ::core::ffi::c_int;
            if eval_charconvert(
                fenc,
                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
                fname,
                tmpname,
            ) == FAIL
            {
                errmsg = gettext(b"Conversion with 'charconvert' failed\0".as_ptr()
                    as *const ::core::ffi::c_char);
            }
            if errmsg.is_null() && {
                *fdp = os_open(tmpname, O_RDONLY, 0 as ::core::ffi::c_int);
                *fdp < 0 as ::core::ffi::c_int
            } {
                errmsg =
                    gettext(b"can't read output of 'charconvert'\0".as_ptr()
                        as *const ::core::ffi::c_char);
            }
        }
        if !errmsg.is_null() {
            msg(errmsg, 0 as ::core::ffi::c_int);
            if !tmpname.is_null() {
                os_remove(tmpname);
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut tmpname as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
            }
        }
        if *fdp < 0 as ::core::ffi::c_int {
            *fdp = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
        }
        return tmpname;
    }
}

pub unsafe extern "C" fn need_conversion(mut fenc: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut same_encoding: bool = false;
        let mut fenc_flags: ::core::ffi::c_int = 0;
        if *fenc as ::core::ffi::c_int == NUL
            || strcmp(p_enc.get(), fenc) == 0 as ::core::ffi::c_int
        {
            same_encoding = true_0 != 0;
            fenc_flags = 0 as ::core::ffi::c_int;
        } else {
            let mut enc_flags: ::core::ffi::c_int = get_fio_flags(p_enc.get());
            fenc_flags = get_fio_flags(fenc);
            same_encoding = enc_flags != 0 as ::core::ffi::c_int && fenc_flags == enc_flags;
        }
        if same_encoding {
            return false_0 != 0;
        }
        return !(fenc_flags == FIO_UTF8 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn get_fio_flags(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if *name as ::core::ffi::c_int == NUL {
            name = p_enc.get();
        }
        let mut prop: ::core::ffi::c_int = enc_canon_props(name);
        if prop & ENC_UNICODE as ::core::ffi::c_int != 0 {
            if prop & ENC_2BYTE as ::core::ffi::c_int != 0 {
                if prop & ENC_ENDIAN_L as ::core::ffi::c_int != 0 {
                    return FIO_UCS2 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int;
                }
                return FIO_UCS2 as ::core::ffi::c_int;
            }
            if prop & ENC_4BYTE as ::core::ffi::c_int != 0 {
                if prop & ENC_ENDIAN_L as ::core::ffi::c_int != 0 {
                    return FIO_UCS4 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int;
                }
                return FIO_UCS4 as ::core::ffi::c_int;
            }
            if prop & ENC_2WORD as ::core::ffi::c_int != 0 {
                if prop & ENC_ENDIAN_L as ::core::ffi::c_int != 0 {
                    return FIO_UTF16 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int;
                }
                return FIO_UTF16 as ::core::ffi::c_int;
            }
            return FIO_UTF8 as ::core::ffi::c_int;
        }
        if prop & ENC_LATIN1 as ::core::ffi::c_int != 0 {
            return FIO_LATIN1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn check_for_bom(
    mut p_in: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
    mut lenp: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *const uint8_t = p_in as *const uint8_t;
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xef as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xbb as ::core::ffi::c_int
            && size >= 3 as ::core::ffi::c_int
            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xbf as ::core::ffi::c_int
            && (flags == FIO_ALL as ::core::ffi::c_int
                || flags == FIO_UTF8 as ::core::ffi::c_int
                || flags == 0 as ::core::ffi::c_int)
        {
            name = b"utf-8\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            len = 3 as ::core::ffi::c_int;
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xff as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xfe as ::core::ffi::c_int
        {
            if size >= 4 as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                && (flags == FIO_ALL as ::core::ffi::c_int
                    || flags == FIO_UCS4 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int)
            {
                name =
                    b"ucs-4le\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                len = 4 as ::core::ffi::c_int;
            } else if flags == FIO_UCS2 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int {
                name =
                    b"ucs-2le\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else if flags == FIO_ALL as ::core::ffi::c_int
                || flags == FIO_UTF16 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int
            {
                name = b"utf-16le\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xfe as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xff as ::core::ffi::c_int
            && (flags == FIO_ALL as ::core::ffi::c_int
                || flags == FIO_UCS2 as ::core::ffi::c_int
                || flags == FIO_UTF16 as ::core::ffi::c_int)
        {
            if flags == FIO_UCS2 as ::core::ffi::c_int {
                name =
                    b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                name =
                    b"utf-16\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        } else if size >= 4 as ::core::ffi::c_int
            && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xfe as ::core::ffi::c_int
            && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0xff as ::core::ffi::c_int
            && (flags == FIO_ALL as ::core::ffi::c_int || flags == FIO_UCS4 as ::core::ffi::c_int)
        {
            name = b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            len = 4 as ::core::ffi::c_int;
        }
        *lenp = len;
        return name;
    }
}
