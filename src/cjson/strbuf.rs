use crate::src::nvim::os::libc::{abort, fprintf, free, malloc, realloc, stderr, vfprintf};
pub use crate::src::nvim::types::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __builtin_va_list, __gnuc_va_list,
    __off64_t, __off_t, __va_list_tag, size_t, strbuf_t, va_list, FILE, _IO_FILE,
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const STRBUF_DEFAULT_SIZE: ::core::ffi::c_int = 1023 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn strbuf_empty_length(mut s: *mut strbuf_t) -> size_t {
    return (*s)
        .size
        .wrapping_sub((*s).length)
        .wrapping_sub(1 as size_t);
}
#[inline]
unsafe extern "C" fn strbuf_ensure_null(mut s: *mut strbuf_t) {
    *(*s).buf.offset((*s).length as isize) = 0 as ::core::ffi::c_char;
}
unsafe extern "C" fn die(mut fmt: *const ::core::ffi::c_char, mut c2rust_args: ...) {
    let mut arg: ::core::ffi::VaListImpl;
    arg = c2rust_args.clone();
    vfprintf(stderr, fmt, arg.as_va_list());
    fprintf(stderr, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    abort();
}
pub unsafe extern "C" fn strbuf_init(mut s: *mut strbuf_t, mut len: size_t) {
    let mut size: size_t = 0;
    if len == 0 {
        size = STRBUF_DEFAULT_SIZE as size_t;
    } else {
        size = len.wrapping_add(1 as size_t);
    }
    if size < len {
        die(
            b"Overflow, len: %zu\0".as_ptr() as *const ::core::ffi::c_char,
            len,
        );
    }
    (*s).buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*s).size = size;
    (*s).length = 0 as size_t;
    (*s).dynamic = 0 as ::core::ffi::c_int;
    (*s).reallocs = 0 as ::core::ffi::c_int;
    (*s).debug = 0 as ::core::ffi::c_int;
    (*s).buf = malloc(size) as *mut ::core::ffi::c_char;
    if (*s).buf.is_null() {
        die(b"Out of memory\0".as_ptr() as *const ::core::ffi::c_char);
    }
    strbuf_ensure_null(s);
}
pub unsafe extern "C" fn strbuf_new(mut len: size_t) -> *mut strbuf_t {
    let mut s: *mut strbuf_t = ::core::ptr::null_mut::<strbuf_t>();
    s = malloc(::core::mem::size_of::<strbuf_t>()) as *mut strbuf_t;
    if s.is_null() {
        die(b"Out of memory\0".as_ptr() as *const ::core::ffi::c_char);
    }
    strbuf_init(s, len);
    (*s).dynamic = 1 as ::core::ffi::c_int;
    return s;
}
#[inline]
unsafe extern "C" fn debug_stats(mut s: *mut strbuf_t) {
    if (*s).debug != 0 {
        fprintf(
            stderr,
            b"strbuf(%lx) reallocs: %d, length: %zd, size: %zd\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            s.expose_addr() as ::core::ffi::c_long,
            (*s).reallocs,
            (*s).length,
            (*s).size,
        );
    }
}
pub unsafe extern "C" fn strbuf_free(mut s: *mut strbuf_t) {
    debug_stats(s);
    if !(*s).buf.is_null() {
        free((*s).buf as *mut ::core::ffi::c_void);
        (*s).buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*s).dynamic != 0 {
        free(s as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn strbuf_free_to_string(
    mut s: *mut strbuf_t,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    debug_stats(s);
    strbuf_ensure_null(s);
    buf = (*s).buf;
    if !len.is_null() {
        *len = (*s).length;
    }
    if (*s).dynamic != 0 {
        free(s as *mut ::core::ffi::c_void);
    }
    return buf;
}
unsafe extern "C" fn calculate_new_size(mut s: *mut strbuf_t, mut len: size_t) -> size_t {
    let mut reqsize: size_t = 0;
    let mut newsize: size_t = 0;
    if len <= 0 as size_t {
        die(b"BUG: Invalid strbuf length requested\0".as_ptr() as *const ::core::ffi::c_char);
    }
    reqsize = len.wrapping_add(1 as size_t);
    if reqsize < len {
        die(
            b"Overflow, len: %zu\0".as_ptr() as *const ::core::ffi::c_char,
            len,
        );
    }
    if (*s).size > reqsize {
        return reqsize;
    }
    newsize = (*s).size;
    if reqsize >= (SIZE_MAX as size_t).wrapping_div(2 as size_t) {
        newsize = reqsize;
    } else {
        while newsize < reqsize {
            newsize = newsize.wrapping_mul(2 as size_t);
        }
    }
    if newsize < reqsize {
        die(
            b"BUG: strbuf length would overflow, len: %zu\0".as_ptr() as *const ::core::ffi::c_char,
            len,
        );
    }
    return newsize;
}
pub unsafe extern "C" fn strbuf_resize(mut s: *mut strbuf_t, mut len: size_t) {
    let mut newsize: size_t = 0;
    newsize = calculate_new_size(s, len);
    if (*s).debug > 1 as ::core::ffi::c_int {
        fprintf(
            stderr,
            b"strbuf(%lx) resize: %zd => %zd\n\0".as_ptr() as *const ::core::ffi::c_char,
            s.expose_addr() as ::core::ffi::c_long,
            (*s).size,
            newsize,
        );
    }
    (*s).size = newsize;
    (*s).buf = realloc((*s).buf as *mut ::core::ffi::c_void, (*s).size) as *mut ::core::ffi::c_char;
    if (*s).buf.is_null() {
        die(
            b"Out of memory, len: %zu\0".as_ptr() as *const ::core::ffi::c_char,
            len,
        );
    }
    (*s).reallocs += 1;
}
pub unsafe extern "C" fn strbuf_append_string(
    mut s: *mut strbuf_t,
    mut str: *const ::core::ffi::c_char,
) {
    let mut i: ::core::ffi::c_int = 0;
    let mut space: size_t = 0;
    space = strbuf_empty_length(s);
    i = 0 as ::core::ffi::c_int;
    while *str.offset(i as isize) != 0 {
        if space < 1 as size_t {
            strbuf_resize(s, (*s).length.wrapping_add(1 as size_t));
            space = strbuf_empty_length(s);
        }
        *(*s).buf.offset((*s).length as isize) = *str.offset(i as isize);
        (*s).length = (*s).length.wrapping_add(1);
        space = space.wrapping_sub(1);
        i += 1;
    }
}
