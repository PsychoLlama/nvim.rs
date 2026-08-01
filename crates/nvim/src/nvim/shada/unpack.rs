//! Reading msgpack out of the ShaDa file.
//!
//! The low half of the reader: `fread_len` insists on a fixed number of
//! bytes, `msgpack_read_uint64` decodes the unsigned integer that heads every
//! entry, and `sd_reader_skip` steps over an entry whose type this Nvim does
//! not know (or does not want).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn sd_reader_skip(
    sd_reader: *mut FileDescriptor,
    offset: size_t,
) -> ShaDaReadResult {
    unsafe {
        let skip_bytes: ptrdiff_t = file_skip(sd_reader, offset);
        if skip_bytes < 0 as ptrdiff_t {
            semsg(
                gettext(
                    b"E886: System error while skipping in ShaDa file: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                uv_strerror(skip_bytes as ::core::ffi::c_int),
            );
            return kSDReadStatusReadError;
        } else if skip_bytes != offset as ptrdiff_t {
            '_c2rust_label: {
                if skip_bytes < offset as ptrdiff_t {
                } else {
                    __assert_fail(
                        b"skip_bytes < (ptrdiff_t)offset\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        548 as ::core::ffi::c_uint,
                        b"ShaDaReadResult sd_reader_skip(FileDescriptor *const, const size_t)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if file_eof(sd_reader) {
                semsg(
                gettext(
                    b"E576: Reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ),
                offset as uint64_t,
            );
            } else {
                semsg(
                    gettext(
                        b"E886: System error while skipping in ShaDa file: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    gettext(b"too few bytes read\0".as_ptr() as *const ::core::ffi::c_char),
                );
            }
            return kSDReadStatusNotShaDa;
        }
        return kSDReadStatusSuccess;
    }
}

pub(crate) unsafe extern "C" fn fread_len(
    sd_reader: *mut FileDescriptor,
    buffer: *mut ::core::ffi::c_char,
    length: size_t,
) -> ShaDaReadResult {
    unsafe {
        let read_bytes: ptrdiff_t = file_read(sd_reader, buffer, length);
        if read_bytes < 0 as ptrdiff_t {
            semsg(
                gettext(
                    b"E886: System error while reading ShaDa file: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                uv_strerror(read_bytes as ::core::ffi::c_int),
            );
            return kSDReadStatusReadError;
        }
        if read_bytes != length as ptrdiff_t {
            semsg(
            gettext(
                b"E576: Error while reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ),
            length as uint64_t,
        );
            return kSDReadStatusNotShaDa;
        }
        return kSDReadStatusSuccess;
    }
}

pub(crate) unsafe extern "C" fn msgpack_read_uint64(
    sd_reader: *mut FileDescriptor,
    mut allow_eof: bool,
    result: *mut uint64_t,
) -> ShaDaReadResult {
    unsafe {
        let fpos: uintmax_t = (*sd_reader).bytes_read as uintmax_t;
        let mut ret: uint8_t = 0;
        let mut read_bytes: ptrdiff_t = file_read(
            sd_reader,
            &raw mut ret as *mut ::core::ffi::c_char,
            1 as size_t,
        );
        if read_bytes < 0 as ptrdiff_t {
            semsg(
                gettext(
                    b"E886: System error while reading integer from ShaDa file: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                uv_strerror(read_bytes as ::core::ffi::c_int),
            );
            return kSDReadStatusReadError;
        } else if read_bytes == 0 as ptrdiff_t {
            if allow_eof as ::core::ffi::c_int != 0
                && file_eof(sd_reader) as ::core::ffi::c_int != 0
            {
                return kSDReadStatusFinished;
            }
            semsg(
            gettext(
                b"E576: Error while reading ShaDa file: expected positive integer at position %lu, but got nothing\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ),
            fpos as uint64_t,
        );
            return kSDReadStatusNotShaDa;
        }
        let mut first_char: ::core::ffi::c_int = ret as ::core::ffi::c_int;
        if !first_char & 0x80 as ::core::ffi::c_int != 0 {
            *result = first_char as uint8_t as uint64_t;
        } else {
            let mut length: size_t = 0 as size_t;
            match first_char {
                204 => {
                    length = 1 as size_t;
                }
                205 => {
                    length = 2 as size_t;
                }
                206 => {
                    length = 4 as size_t;
                }
                207 => {
                    length = 8 as size_t;
                }
                _ => {
                    semsg(
                    gettext(
                        b"E576: Error while reading ShaDa file: expected positive integer at position %lu\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    fpos as uint64_t,
                );
                    return kSDReadStatusNotShaDa;
                }
            }
            let mut buf: uint64_t = 0 as uint64_t;
            let mut buf_u8: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
            let mut fl_ret: ShaDaReadResult = kSDReadStatusSuccess;
            fl_ret = fread_len(
                sd_reader,
                buf_u8.offset(
                    ::core::mem::size_of::<uint64_t>().wrapping_sub(length as usize) as isize,
                ),
                length,
            );
            if fl_ret as ::core::ffi::c_uint
                != kSDReadStatusSuccess as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return fl_ret;
            }
            *result = __bswap_64(buf as __uint64_t) as uint64_t;
        }
        return kSDReadStatusSuccess;
    }
}
