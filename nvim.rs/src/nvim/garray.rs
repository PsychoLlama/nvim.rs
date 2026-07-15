extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xstpcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn sort_strings(files: *mut *mut ::core::ffi::c_char, count: ::core::ffi::c_int);
}
pub type uint8_t = u8;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
#[no_mangle]
pub unsafe extern "C" fn ga_clear(mut gap: *mut garray_T) {
    xfree((*gap).ga_data);
    (*gap).ga_data = NULL;
    (*gap).ga_maxlen = 0 as ::core::ffi::c_int;
    (*gap).ga_len = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ga_clear_strings(mut gap: *mut garray_T) {
    let mut _gap: *mut garray_T = gap;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut *mut ::core::ffi::c_void = ((*_gap).ga_data
                as *mut *mut ::core::ffi::c_void)
                .offset(i as isize);
            xfree(*_item);
            i += 1;
        }
    }
    ga_clear(_gap);
}
#[no_mangle]
pub unsafe extern "C" fn ga_init(
    mut gap: *mut garray_T,
    mut itemsize: ::core::ffi::c_int,
    mut growsize: ::core::ffi::c_int,
) {
    (*gap).ga_data = NULL;
    (*gap).ga_maxlen = 0 as ::core::ffi::c_int;
    (*gap).ga_len = 0 as ::core::ffi::c_int;
    (*gap).ga_itemsize = itemsize;
    ga_set_growsize(gap, growsize);
}
#[no_mangle]
pub unsafe extern "C" fn ga_set_growsize(
    mut gap: *mut garray_T,
    mut growsize: ::core::ffi::c_int,
) {
    if growsize < 1 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ga_set_growsize\0".as_ptr() as *const ::core::ffi::c_char,
            57 as ::core::ffi::c_int,
            true_0 != 0,
            b"trying to set an invalid ga_growsize: %d\0".as_ptr()
                as *const ::core::ffi::c_char,
            growsize,
        );
        (*gap).ga_growsize = 1 as ::core::ffi::c_int;
    } else {
        (*gap).ga_growsize = growsize;
    };
}
#[no_mangle]
pub unsafe extern "C" fn ga_grow(mut gap: *mut garray_T, mut n: ::core::ffi::c_int) {
    if (*gap).ga_maxlen - (*gap).ga_len >= n {
        return;
    }
    if (*gap).ga_growsize < 1 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ga_grow\0".as_ptr() as *const ::core::ffi::c_char,
            76 as ::core::ffi::c_int,
            true_0 != 0,
            b"ga_growsize(%d) is less than 1\0".as_ptr() as *const ::core::ffi::c_char,
            (*gap).ga_growsize,
        );
    }
    n = if n > (*gap).ga_growsize { n } else { (*gap).ga_growsize };
    n = if n > (*gap).ga_len / 2 as ::core::ffi::c_int {
        n
    } else {
        (*gap).ga_len / 2 as ::core::ffi::c_int
    };
    let mut new_maxlen: ::core::ffi::c_int = (*gap).ga_len + n;
    let mut new_size: size_t = ((*gap).ga_itemsize as size_t)
        .wrapping_mul(new_maxlen as size_t);
    let mut old_size: size_t = ((*gap).ga_itemsize as size_t)
        .wrapping_mul((*gap).ga_maxlen as size_t);
    let mut pp: *mut ::core::ffi::c_char = xrealloc((*gap).ga_data, new_size)
        as *mut ::core::ffi::c_char;
    memset(
        pp.offset(old_size as isize) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        new_size.wrapping_sub(old_size),
    );
    (*gap).ga_maxlen = new_maxlen;
    (*gap).ga_data = pp as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn ga_remove_duplicate_strings(mut gap: *mut garray_T) {
    let mut fnames: *mut *mut ::core::ffi::c_char = (*gap).ga_data
        as *mut *mut ::core::ffi::c_char;
    sort_strings(fnames, (*gap).ga_len);
    let mut i: ::core::ffi::c_int = (*gap).ga_len - 1 as ::core::ffi::c_int;
    while i > 0 as ::core::ffi::c_int {
        if path_fnamecmp(
            *fnames.offset((i - 1 as ::core::ffi::c_int) as isize),
            *fnames.offset(i as isize),
        ) == 0 as ::core::ffi::c_int
        {
            xfree(*fnames.offset(i as isize) as *mut ::core::ffi::c_void);
            let mut j: ::core::ffi::c_int = i + 1 as ::core::ffi::c_int;
            while j < (*gap).ga_len {
                *fnames.offset((j - 1 as ::core::ffi::c_int) as isize) = *fnames
                    .offset(j as isize);
                j += 1;
            }
            (*gap).ga_len -= 1;
        }
        i -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ga_concat_strings(
    mut gap: *const garray_T,
    mut sep: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let nelem: size_t = (*gap).ga_len as size_t;
    let mut strings: *mut *const ::core::ffi::c_char = (*gap).ga_data
        as *mut *const ::core::ffi::c_char;
    if nelem == 0 as size_t {
        return xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
    }
    let mut len: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < nelem {
        len = len.wrapping_add(strlen(*strings.offset(i as isize)));
        i = i.wrapping_add(1);
    }
    len = len.wrapping_add(nelem.wrapping_sub(1 as size_t).wrapping_mul(strlen(sep)));
    let ret: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = ret;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < nelem.wrapping_sub(1 as size_t) {
        s = xstpcpy(s, *strings.offset(i_0 as isize));
        s = xstpcpy(s, sep);
        i_0 = i_0.wrapping_add(1);
    }
    strcpy(s, *strings.offset(nelem.wrapping_sub(1 as size_t) as isize));
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ga_concat(
    mut gap: *mut garray_T,
    mut s: *const ::core::ffi::c_char,
) {
    if s.is_null() {
        return;
    }
    ga_concat_len(gap, s, strlen(s));
}
#[no_mangle]
pub unsafe extern "C" fn ga_concat_len(
    gap: *mut garray_T,
    mut s: *const ::core::ffi::c_char,
    len: size_t,
) {
    if len == 0 as size_t {
        return;
    }
    ga_grow(gap, len as ::core::ffi::c_int);
    let mut data: *mut ::core::ffi::c_char = (*gap).ga_data as *mut ::core::ffi::c_char;
    memcpy(
        data.offset((*gap).ga_len as isize) as *mut ::core::ffi::c_void,
        s as *const ::core::ffi::c_void,
        len,
    );
    (*gap).ga_len += len as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ga_append(mut gap: *mut garray_T, mut c: uint8_t) {
    ga_grow(gap, 1 as ::core::ffi::c_int);
    *((*gap).ga_data as *mut uint8_t).offset((*gap).ga_len as isize) = c;
    (*gap).ga_len += 1;
}
#[no_mangle]
pub unsafe extern "C" fn ga_append_via_ptr(
    mut gap: *mut garray_T,
    mut item_size: size_t,
) -> *mut ::core::ffi::c_void {
    if item_size as ::core::ffi::c_int != (*gap).ga_itemsize {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ga_append_via_ptr\0".as_ptr() as *const ::core::ffi::c_char,
            209 as ::core::ffi::c_int,
            true_0 != 0,
            b"wrong item size (%zu), should be %d\0".as_ptr()
                as *const ::core::ffi::c_char,
            item_size,
            (*gap).ga_itemsize,
        );
    }
    ga_grow(gap, 1 as ::core::ffi::c_int);
    let c2rust_fresh0 = (*gap).ga_len;
    (*gap).ga_len = (*gap).ga_len + 1;
    return ((*gap).ga_data as *mut ::core::ffi::c_char)
        .offset(item_size.wrapping_mul(c2rust_fresh0 as size_t) as isize)
        as *mut ::core::ffi::c_void;
}
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
