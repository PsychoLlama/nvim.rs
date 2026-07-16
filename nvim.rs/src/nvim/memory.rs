extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    static mut arena_alloc_count: size_t;
    static e_outofmem: [::core::ffi::c_char; 0];
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut did_outofmem_msg: bool;
    fn preserve_exit(errmsg: *const ::core::ffi::c_char) -> !;
    fn mf_release_all() -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn clear_sb_text(all: bool);
}
pub type __time_t = ::core::ffi::c_long;
pub type uint8_t = u8;
pub type uint64_t = u64;
pub type size_t = usize;
pub type time_t = __time_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}
pub type ArenaMem = *mut consumed_blk;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type MemMalloc = Option<unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_void>;
pub type MemFree = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type MemCalloc = Option<unsafe extern "C" fn(size_t, size_t) -> *mut ::core::ffi::c_void>;
pub type MemRealloc =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, size_t) -> *mut ::core::ffi::c_void>;
pub type MergeSortGetFunc =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void>;
pub type MergeSortSetFunc =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void) -> ()>;
pub type MergeSortCompareFunc = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
#[no_mangle]
pub static mut mem_malloc: MemMalloc =
    Some(malloc as unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_void);
#[no_mangle]
pub static mut mem_free: MemFree =
    Some(free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ());
#[no_mangle]
pub static mut mem_calloc: MemCalloc =
    Some(calloc as unsafe extern "C" fn(size_t, size_t) -> *mut ::core::ffi::c_void);
#[no_mangle]
pub static mut mem_realloc: MemRealloc = Some(
    realloc as unsafe extern "C" fn(*mut ::core::ffi::c_void, size_t) -> *mut ::core::ffi::c_void,
);
unsafe extern "C" fn try_to_free_memory() {
    static mut trying_to_free: bool = false_0 != 0;
    if trying_to_free {
        return;
    }
    trying_to_free = true_0 != 0;
    clear_sb_text(true_0 != 0);
    mf_release_all();
    arena_free_reuse_blks();
    trying_to_free = false_0 != 0;
}
unsafe extern "C" fn do_outofmem_msg(mut size: size_t) {
    if did_outofmem_msg {
        return;
    }
    emsg_silent = 0 as ::core::ffi::c_int;
    did_outofmem_msg = true_0 != 0;
    semsg(
        gettext(b"E342: Out of memory!  (allocating %lu bytes)\0".as_ptr()
            as *const ::core::ffi::c_char),
        size as uint64_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn try_malloc(mut size: size_t) -> *mut ::core::ffi::c_void {
    let mut allocated_size: size_t = if size != 0 { size } else { 1 as size_t };
    let mut ret: *mut ::core::ffi::c_void =
        mem_malloc.expect("non-null function pointer")(allocated_size);
    if ret.is_null() {
        try_to_free_memory();
        ret = mem_malloc.expect("non-null function pointer")(allocated_size);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn verbose_try_malloc(mut size: size_t) -> *mut ::core::ffi::c_void {
    let mut ret: *mut ::core::ffi::c_void = try_malloc(size);
    if ret.is_null() {
        do_outofmem_msg(size);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn xmalloc(mut size: size_t) -> *mut ::core::ffi::c_void {
    let mut ret: *mut ::core::ffi::c_void = try_malloc(size);
    if ret.is_null() {
        preserve_exit(&raw const e_outofmem as *const ::core::ffi::c_char);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn xfree(mut ptr: *mut ::core::ffi::c_void) {
    mem_free.expect("non-null function pointer")(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn xcalloc(mut count: size_t, mut size: size_t) -> *mut ::core::ffi::c_void {
    let mut allocated_count: size_t = if count != 0 && size != 0 {
        count
    } else {
        1 as size_t
    };
    let mut allocated_size: size_t = if count != 0 && size != 0 {
        size
    } else {
        1 as size_t
    };
    let mut ret: *mut ::core::ffi::c_void =
        mem_calloc.expect("non-null function pointer")(allocated_count, allocated_size);
    if ret.is_null() {
        try_to_free_memory();
        ret = mem_calloc.expect("non-null function pointer")(allocated_count, allocated_size);
        if ret.is_null() {
            preserve_exit(&raw const e_outofmem as *const ::core::ffi::c_char);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn xrealloc(
    mut ptr: *mut ::core::ffi::c_void,
    mut size: size_t,
) -> *mut ::core::ffi::c_void {
    let mut allocated_size: size_t = if size != 0 { size } else { 1 as size_t };
    let mut ret: *mut ::core::ffi::c_void =
        mem_realloc.expect("non-null function pointer")(ptr, allocated_size);
    if ret.is_null() {
        try_to_free_memory();
        ret = mem_realloc.expect("non-null function pointer")(ptr, allocated_size);
        if ret.is_null() {
            preserve_exit(&raw const e_outofmem as *const ::core::ffi::c_char);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn xmallocz(mut size: size_t) -> *mut ::core::ffi::c_void {
    let mut total_size: size_t = size.wrapping_add(1 as size_t);
    if total_size < size {
        preserve_exit(gettext(
            b"Nvim: Data too large to fit into virtual memory space\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    }
    let mut ret: *mut ::core::ffi::c_void = xmalloc(total_size);
    *(ret as *mut ::core::ffi::c_char).offset(size as isize) = NUL as ::core::ffi::c_char;
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn xmemdupz(
    mut data: *const ::core::ffi::c_void,
    mut len: size_t,
) -> *mut ::core::ffi::c_void {
    return memcpy(xmallocz(len), data, len);
}
#[no_mangle]
pub unsafe extern "C" fn xmemcpyz(
    mut dst: *mut ::core::ffi::c_void,
    mut src: *const ::core::ffi::c_void,
    mut len: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dst, src, len);
    *(dst as *mut ::core::ffi::c_char).offset(len as isize) = NUL as ::core::ffi::c_char;
    return dst;
}
#[no_mangle]
pub unsafe extern "C" fn xstrchrnul(
    mut str: *const ::core::ffi::c_char,
    mut c: ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = strchr(str, c as ::core::ffi::c_int);
    return if !p.is_null() {
        p as *mut ::core::ffi::c_char
    } else {
        str.offset(strlen(str) as isize) as *mut ::core::ffi::c_char
    };
}
#[no_mangle]
pub unsafe extern "C" fn xmemscan(
    mut addr: *const ::core::ffi::c_void,
    mut c: ::core::ffi::c_char,
    mut size: size_t,
) -> *mut ::core::ffi::c_void {
    let mut p: *const ::core::ffi::c_char =
        memchr(addr, c as ::core::ffi::c_int, size) as *const ::core::ffi::c_char;
    return (if !p.is_null() {
        p as *mut ::core::ffi::c_char
    } else {
        (addr as *mut ::core::ffi::c_char).offset(size as isize)
    }) as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn strchrsub(
    mut str: *mut ::core::ffi::c_char,
    mut c: ::core::ffi::c_char,
    mut x: ::core::ffi::c_char,
) {
    '_c2rust_label: {
        if c as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"c != NUL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/memory.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                305 as ::core::ffi::c_uint,
                b"void strchrsub(char *, char, char)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    loop {
        str = strchr(str, c as ::core::ffi::c_int);
        if str.is_null() {
            break;
        }
        let c2rust_fresh0 = str;
        str = str.offset(1);
        *c2rust_fresh0 = x;
    }
}
#[no_mangle]
pub unsafe extern "C" fn memchrsub(
    mut data: *mut ::core::ffi::c_void,
    mut c: ::core::ffi::c_char,
    mut x: ::core::ffi::c_char,
    mut len: size_t,
) {
    let mut p: *mut ::core::ffi::c_char = data as *mut ::core::ffi::c_char;
    let mut end: *mut ::core::ffi::c_char = (data as *mut ::core::ffi::c_char).offset(len as isize);
    loop {
        p = memchr(
            p as *const ::core::ffi::c_void,
            c as ::core::ffi::c_int,
            end.offset_from(p) as size_t,
        ) as *mut ::core::ffi::c_char;
        if p.is_null() {
            break;
        }
        let c2rust_fresh1 = p;
        p = p.offset(1);
        *c2rust_fresh1 = x;
    }
}
#[no_mangle]
pub unsafe extern "C" fn strcnt(
    mut str: *const ::core::ffi::c_char,
    mut c: ::core::ffi::c_char,
) -> size_t {
    '_c2rust_label: {
        if c as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"c != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/memory.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                337 as ::core::ffi::c_uint,
                b"size_t strcnt(const char *, char)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut cnt: size_t = 0 as size_t;
    loop {
        str = strchr(str, c as ::core::ffi::c_int);
        if str.is_null() {
            break;
        }
        cnt = cnt.wrapping_add(1);
        str = str.offset(1);
    }
    return cnt;
}
#[no_mangle]
pub unsafe extern "C" fn memcnt(
    mut data: *const ::core::ffi::c_void,
    mut c: ::core::ffi::c_char,
    mut len: size_t,
) -> size_t {
    let mut cnt: size_t = 0 as size_t;
    let mut ptr: *const ::core::ffi::c_char = data as *const ::core::ffi::c_char;
    let mut end: *const ::core::ffi::c_char = ptr.offset(len as isize);
    loop {
        ptr = memchr(
            ptr as *const ::core::ffi::c_void,
            c as ::core::ffi::c_int,
            end.offset_from(ptr) as size_t,
        ) as *const ::core::ffi::c_char;
        if ptr.is_null() {
            break;
        }
        cnt = cnt.wrapping_add(1);
        ptr = ptr.offset(1);
    }
    return cnt;
}
#[no_mangle]
pub unsafe extern "C" fn xstpcpy(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let len: size_t = strlen(src);
    return (memcpy(
        dst as *mut ::core::ffi::c_void,
        src as *const ::core::ffi::c_void,
        len.wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char)
        .offset(len as isize);
}
#[no_mangle]
pub unsafe extern "C" fn xstpncpy(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut maxlen: size_t,
) -> *mut ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char =
        memchr(src as *const ::core::ffi::c_void, NUL, maxlen) as *const ::core::ffi::c_char;
    if !p.is_null() {
        let mut srclen: size_t = p.offset_from(src) as size_t;
        memcpy(
            dst as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            srclen,
        );
        memset(
            dst.offset(srclen as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            maxlen.wrapping_sub(srclen),
        );
        return dst.offset(srclen as isize);
    } else {
        memcpy(
            dst as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            maxlen,
        );
        return dst.offset(maxlen as isize);
    };
}
#[no_mangle]
pub unsafe extern "C" fn xstrlcpy(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut dsize: size_t,
) -> size_t {
    let mut slen: size_t = strlen(src);
    if dsize != 0 {
        let mut len: size_t = if slen < dsize.wrapping_sub(1 as size_t) {
            slen
        } else {
            dsize.wrapping_sub(1 as size_t)
        };
        memcpy(
            dst as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            len,
        );
        *dst.offset(len as isize) = NUL as ::core::ffi::c_char;
    }
    return slen;
}
#[no_mangle]
pub unsafe extern "C" fn xstrlcat(
    dst: *mut ::core::ffi::c_char,
    src: *const ::core::ffi::c_char,
    dsize: size_t,
) -> size_t {
    '_c2rust_label: {
        if dsize > 0 as size_t {
        } else {
            __assert_fail(
                b"dsize > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/memory.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                460 as ::core::ffi::c_uint,
                b"size_t xstrlcat(char *const, const char *const, const size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let dlen: size_t = strlen(dst);
    '_c2rust_label_0: {
        if dlen < dsize {
        } else {
            __assert_fail(
                b"dlen < dsize\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/memory.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                462 as ::core::ffi::c_uint,
                b"size_t xstrlcat(char *const, const char *const, const size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let slen: size_t = strlen(src);
    if slen > dsize.wrapping_sub(dlen).wrapping_sub(1 as size_t) {
        memmove(
            dst.offset(dlen as isize) as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            dsize.wrapping_sub(dlen).wrapping_sub(1 as size_t),
        );
        *dst.offset(dsize.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
    } else {
        memmove(
            dst.offset(dlen as isize) as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            slen.wrapping_add(1 as size_t),
        );
    }
    return slen.wrapping_add(dlen);
}
#[no_mangle]
pub unsafe extern "C" fn xstrdup(mut str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    return xmemdupz(str as *const ::core::ffi::c_void, strlen(str)) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn xstrdupnul(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if str.is_null() {
        return xmallocz(0 as size_t) as *mut ::core::ffi::c_char;
    }
    return xstrdup(str);
}
#[no_mangle]
pub unsafe extern "C" fn xmemrchr(
    mut src: *const ::core::ffi::c_void,
    mut c: uint8_t,
    mut len: size_t,
) -> *mut ::core::ffi::c_void {
    loop {
        let c2rust_fresh2 = len;
        len = len.wrapping_sub(1);
        if c2rust_fresh2 == 0 {
            break;
        }
        if *(src as *mut uint8_t).offset(len as isize) as ::core::ffi::c_int
            == c as ::core::ffi::c_int
        {
            return (src as *mut uint8_t).offset(len as isize) as *mut ::core::ffi::c_void;
        }
    }
    return NULL;
}
#[no_mangle]
pub unsafe extern "C" fn xstrndup(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char =
        memchr(str as *const ::core::ffi::c_void, NUL, len) as *const ::core::ffi::c_char;
    return xmemdupz(
        str as *const ::core::ffi::c_void,
        if !p.is_null() {
            p.offset_from(str) as size_t
        } else {
            len
        },
    ) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn xmemdup(
    mut data: *const ::core::ffi::c_void,
    mut len: size_t,
) -> *mut ::core::ffi::c_void {
    return memcpy(xmalloc(len), data, len);
}
#[no_mangle]
pub unsafe extern "C" fn strequal(
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> bool {
    return a.is_null() && b.is_null()
        || !a.is_null() && !b.is_null() && strcmp(a, b) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn strnequal(
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
    mut n: size_t,
) -> bool {
    return a.is_null() && b.is_null()
        || !a.is_null() && !b.is_null() && strncmp(a, b, n) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn time_to_bytes(mut time_: time_t, mut buf: *mut uint8_t) {
    let mut i: size_t = 7 as size_t;
    let mut bufi: size_t = 0 as size_t;
    while bufi < 8 as size_t {
        *buf.offset(bufi as isize) = (time_ as uint64_t >> i.wrapping_mul(8 as size_t)) as uint8_t;
        i = i.wrapping_sub(1);
        bufi = bufi.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mergesort_list(
    mut head: *mut ::core::ffi::c_void,
    mut get_next: MergeSortGetFunc,
    mut set_next: MergeSortSetFunc,
    mut get_prev: MergeSortGetFunc,
    mut set_prev: MergeSortSetFunc,
    mut compare: MergeSortCompareFunc,
) -> *mut ::core::ffi::c_void {
    if head.is_null() || get_next.expect("non-null function pointer")(head).is_null() {
        return head;
    }
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut curr: *mut ::core::ffi::c_void = head;
    while !curr.is_null() {
        n += 1;
        curr = get_next.expect("non-null function pointer")(curr);
    }
    let mut size: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while size < n {
        let mut new_head: *mut ::core::ffi::c_void = NULL;
        let mut tail: *mut ::core::ffi::c_void = NULL;
        curr = head;
        while !curr.is_null() {
            let mut left: *mut ::core::ffi::c_void = curr;
            let mut right: *mut ::core::ffi::c_void = left;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < size && !right.is_null() {
                right = get_next.expect("non-null function pointer")(right);
                i += 1;
            }
            let mut next: *mut ::core::ffi::c_void = right;
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < size && !next.is_null() {
                next = get_next.expect("non-null function pointer")(next);
                i_0 += 1;
            }
            let mut l_end: *mut ::core::ffi::c_void = if !right.is_null() {
                get_prev.expect("non-null function pointer")(right)
            } else {
                NULL
            };
            if !l_end.is_null() {
                set_next.expect("non-null function pointer")(l_end, NULL);
            }
            if !right.is_null() {
                set_prev.expect("non-null function pointer")(right, NULL);
            }
            let mut r_end: *mut ::core::ffi::c_void = if !next.is_null() {
                get_prev.expect("non-null function pointer")(next)
            } else {
                NULL
            };
            if !r_end.is_null() {
                set_next.expect("non-null function pointer")(r_end, NULL);
            }
            if !next.is_null() {
                set_prev.expect("non-null function pointer")(next, NULL);
            }
            let mut merged: *mut ::core::ffi::c_void = NULL;
            let mut merged_tail: *mut ::core::ffi::c_void = NULL;
            while !left.is_null() || !right.is_null() {
                let mut chosen: *mut ::core::ffi::c_void = NULL;
                if left.is_null() {
                    chosen = right;
                    right = get_next.expect("non-null function pointer")(right);
                } else if right.is_null() {
                    chosen = left;
                    left = get_next.expect("non-null function pointer")(left);
                } else if compare.expect("non-null function pointer")(left, right)
                    <= 0 as ::core::ffi::c_int
                {
                    chosen = left;
                    left = get_next.expect("non-null function pointer")(left);
                } else {
                    chosen = right;
                    right = get_next.expect("non-null function pointer")(right);
                }
                if !merged_tail.is_null() {
                    set_next.expect("non-null function pointer")(merged_tail, chosen);
                    set_prev.expect("non-null function pointer")(chosen, merged_tail);
                    merged_tail = chosen;
                } else {
                    merged_tail = chosen;
                    merged = merged_tail;
                    set_prev.expect("non-null function pointer")(chosen, NULL);
                }
            }
            if new_head.is_null() {
                new_head = merged;
            } else {
                set_next.expect("non-null function pointer")(tail, merged);
                set_prev.expect("non-null function pointer")(merged, tail);
            }
            while !get_next.expect("non-null function pointer")(merged_tail).is_null() {
                merged_tail = get_next.expect("non-null function pointer")(merged_tail);
            }
            tail = merged_tail;
            curr = next;
        }
        head = new_head;
        size *= 2 as ::core::ffi::c_int;
    }
    return head;
}
pub const REUSE_MAX: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
static mut arena_reuse_blk: *mut consumed_blk = ::core::ptr::null_mut::<consumed_blk>();
static mut arena_reuse_blk_count: size_t = 0 as size_t;
unsafe extern "C" fn arena_free_reuse_blks() {
    while arena_reuse_blk_count > 0 as size_t {
        let mut blk: *mut consumed_blk = arena_reuse_blk;
        arena_reuse_blk = (*arena_reuse_blk).prev;
        xfree(blk as *mut ::core::ffi::c_void);
        arena_reuse_blk_count = arena_reuse_blk_count.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn arena_finish(mut arena: *mut Arena) -> ArenaMem {
    let mut res: *mut consumed_blk = (*arena).cur_blk as *mut consumed_blk;
    *arena = ARENA_EMPTY;
    return res as ArenaMem;
}
#[no_mangle]
pub unsafe extern "C" fn alloc_block() -> *mut ::core::ffi::c_void {
    if arena_reuse_blk_count > 0 as size_t {
        let mut retval: *mut ::core::ffi::c_void =
            arena_reuse_blk as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void;
        arena_reuse_blk = (*arena_reuse_blk).prev;
        arena_reuse_blk_count = arena_reuse_blk_count.wrapping_sub(1);
        return retval;
    } else {
        arena_alloc_count = arena_alloc_count.wrapping_add(1);
        return xmalloc(ARENA_BLOCK_SIZE as size_t);
    };
}
#[no_mangle]
pub unsafe extern "C" fn arena_alloc_block(mut arena: *mut Arena) {
    let mut prev_blk: *mut consumed_blk = (*arena).cur_blk as *mut consumed_blk;
    (*arena).cur_blk = alloc_block() as *mut ::core::ffi::c_char;
    (*arena).pos = 0 as size_t;
    (*arena).size = ARENA_BLOCK_SIZE as size_t;
    let mut blk: *mut consumed_blk =
        arena_alloc(arena, ::core::mem::size_of::<consumed_blk>(), true_0 != 0)
            as *mut consumed_blk;
    (*blk).prev = prev_blk;
}
unsafe extern "C" fn arena_align_offset(mut off: uint64_t) -> size_t {
    return (off as size_t).wrapping_add(ARENA_ALIGN.wrapping_sub(1 as size_t))
        & !ARENA_ALIGN.wrapping_sub(1 as size_t);
}
pub const ARENA_ALIGN: usize = if ::core::mem::size_of::<*mut ::core::ffi::c_void>()
    > ::core::mem::size_of::<::core::ffi::c_double>()
{
    ::core::mem::size_of::<*mut ::core::ffi::c_void>()
} else {
    ::core::mem::size_of::<::core::ffi::c_double>()
};
#[no_mangle]
pub unsafe extern "C" fn arena_alloc(
    mut arena: *mut Arena,
    mut size: size_t,
    mut align: bool,
) -> *mut ::core::ffi::c_void {
    if arena.is_null() {
        return xmalloc(size);
    }
    if (*arena).cur_blk.is_null() {
        arena_alloc_block(arena);
    }
    let mut alloc_pos: size_t = if align as ::core::ffi::c_int != 0 {
        arena_align_offset((*arena).pos as uint64_t)
    } else {
        (*arena).pos
    };
    if alloc_pos.wrapping_add(size) > (*arena).size {
        if size
            > (ARENA_BLOCK_SIZE as usize).wrapping_sub(::core::mem::size_of::<consumed_blk>())
                >> 1 as ::core::ffi::c_int
        {
            arena_alloc_count = arena_alloc_count.wrapping_add(1);
            let mut hdr_size: size_t = ::core::mem::size_of::<consumed_blk>();
            let mut aligned_hdr_size: size_t = if align as ::core::ffi::c_int != 0 {
                arena_align_offset(hdr_size as uint64_t)
            } else {
                hdr_size
            };
            let mut alloc: *mut ::core::ffi::c_char =
                xmalloc(size.wrapping_add(aligned_hdr_size)) as *mut ::core::ffi::c_char;
            let mut cur_blk: *mut consumed_blk = (*arena).cur_blk as *mut consumed_blk;
            let mut fix_blk: *mut consumed_blk = alloc as *mut consumed_blk;
            (*fix_blk).prev = (*cur_blk).prev;
            (*cur_blk).prev = fix_blk;
            return alloc.offset(aligned_hdr_size as isize) as *mut ::core::ffi::c_void;
        } else {
            arena_alloc_block(arena);
            alloc_pos = if align as ::core::ffi::c_int != 0 {
                arena_align_offset((*arena).pos as uint64_t)
            } else {
                (*arena).pos
            };
        }
    }
    let mut mem: *mut ::core::ffi::c_char = (*arena).cur_blk.offset(alloc_pos as isize);
    (*arena).pos = alloc_pos.wrapping_add(size);
    return mem as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn free_block(mut block: *mut ::core::ffi::c_void) {
    if arena_reuse_blk_count < REUSE_MAX as size_t {
        let mut reuse_blk: *mut consumed_blk = block as *mut consumed_blk;
        (*reuse_blk).prev = arena_reuse_blk;
        arena_reuse_blk = reuse_blk;
        arena_reuse_blk_count = arena_reuse_blk_count.wrapping_add(1);
    } else {
        xfree(block);
    };
}
#[no_mangle]
pub unsafe extern "C" fn arena_mem_free(mut mem: ArenaMem) {
    let mut b: *mut consumed_blk = mem as *mut consumed_blk;
    if !b.is_null() {
        let mut reuse_blk: *mut consumed_blk = b;
        b = (*b).prev;
        free_block(reuse_blk as *mut ::core::ffi::c_void);
    }
    while !b.is_null() {
        let mut prev: *mut consumed_blk = (*b).prev;
        xfree(b as *mut ::core::ffi::c_void);
        b = prev;
    }
}
#[no_mangle]
pub unsafe extern "C" fn arena_allocz(
    mut arena: *mut Arena,
    mut size: size_t,
) -> *mut ::core::ffi::c_char {
    let mut mem: *mut ::core::ffi::c_char =
        arena_alloc(arena, size.wrapping_add(1 as size_t), false_0 != 0)
            as *mut ::core::ffi::c_char;
    *mem.offset(size as isize) = NUL as ::core::ffi::c_char;
    return mem;
}
#[no_mangle]
pub unsafe extern "C" fn arena_memdupz(
    mut arena: *mut Arena,
    mut buf: *const ::core::ffi::c_char,
    mut size: size_t,
) -> *mut ::core::ffi::c_char {
    let mut mem: *mut ::core::ffi::c_char = arena_allocz(arena, size);
    memcpy(
        mem as *mut ::core::ffi::c_void,
        buf as *const ::core::ffi::c_void,
        size,
    );
    return mem;
}
#[no_mangle]
pub unsafe extern "C" fn arena_strdup(
    mut arena: *mut Arena,
    mut str: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return arena_memdupz(arena, str, strlen(str));
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
