extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
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
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strtok_r(
        __s: *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
        __save_ptr: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memchrsub(
        data: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_char,
        x: ::core::ffi::c_char,
        len: size_t,
    );
    fn memcnt(data: *const ::core::ffi::c_void, c: ::core::ffi::c_char, len: size_t) -> size_t;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn vim_gettempdir() -> *mut ::core::ffi::c_char;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut NameBuff: [::core::ffi::c_char; 4096];
    fn os_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn os_getenv_noalloc(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn os_env_exists(name: *const ::core::ffi::c_char, nonempty: bool) -> bool;
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn concat_fnames_realloc(
        fname1: *mut ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        sep: bool,
    ) -> *mut ::core::ffi::c_char;
    fn path_is_absolute(fname: *const ::core::ffi::c_char) -> bool;
    fn kv_do_printf(
        str: *mut StringBuilder,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub type XDGVarType = ::core::ffi::c_int;
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ::core::ffi::c_char,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"char *get_xdg_home(const XDGVarType)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
static mut xdg_env_vars: [*const ::core::ffi::c_char; 7] = [
    b"XDG_CONFIG_HOME\0".as_ptr() as *const ::core::ffi::c_char,
    b"XDG_DATA_HOME\0".as_ptr() as *const ::core::ffi::c_char,
    b"XDG_CACHE_HOME\0".as_ptr() as *const ::core::ffi::c_char,
    b"XDG_STATE_HOME\0".as_ptr() as *const ::core::ffi::c_char,
    b"XDG_RUNTIME_DIR\0".as_ptr() as *const ::core::ffi::c_char,
    b"XDG_CONFIG_DIRS\0".as_ptr() as *const ::core::ffi::c_char,
    b"XDG_DATA_DIRS\0".as_ptr() as *const ::core::ffi::c_char,
];
static mut xdg_defaults: [*const ::core::ffi::c_char; 7] = [
    b"~/.config\0".as_ptr() as *const ::core::ffi::c_char,
    b"~/.local/share\0".as_ptr() as *const ::core::ffi::c_char,
    b"~/.cache\0".as_ptr() as *const ::core::ffi::c_char,
    b"~/.local/state\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"/etc/xdg/\0".as_ptr() as *const ::core::ffi::c_char,
    b"/usr/local/share/:/usr/share/\0".as_ptr() as *const ::core::ffi::c_char,
];
#[no_mangle]
pub unsafe extern "C" fn get_appname(mut namelike: bool) -> *const ::core::ffi::c_char {
    let mut env_val: *const ::core::ffi::c_char =
        os_getenv_noalloc(b"NVIM_APPNAME\0".as_ptr() as *const ::core::ffi::c_char);
    if env_val.is_null() {
        xstrlcpy(
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            b"nvim\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        );
    }
    if namelike {
        memchrsub(
            &raw mut NameBuff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            '/' as ::core::ffi::c_char,
            '-' as ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        );
        memchrsub(
            &raw mut NameBuff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            '\\' as ::core::ffi::c_char,
            '-' as ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        );
    }
    return &raw mut NameBuff as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn appname_is_valid() -> bool {
    let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
    if path_is_absolute(appname) as ::core::ffi::c_int != 0
        || strequal(appname, b"/\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
            != 0
        || strequal(appname, b"\\\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
            != 0
        || strequal(appname, b".\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
            != 0
        || strequal(appname, b"..\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
            != 0
        || !strstr(appname, b"/..\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        || !strstr(appname, b"../\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn xdg_remove_duplicate(
    mut ret: *mut ::core::ffi::c_char,
    mut sep: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut data: C2Rust_Unnamed = C2Rust_Unnamed {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    };
    let mut saveptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut token: *mut ::core::ffi::c_char = strtok_r(ret, sep, &raw mut saveptr);
    while !token.is_null() {
        let mut is_duplicate: bool = false_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < data.size {
            if path_fnamecmp(*data.items.offset(i as isize), token) == 0 as ::core::ffi::c_int {
                is_duplicate = true_0 != 0;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if !is_duplicate {
            if data.size == data.capacity {
                data.capacity = if data.capacity != 0 {
                    data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                data.items = xrealloc(
                    data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(data.capacity),
                ) as *mut *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh0 = data.size;
            data.size = data.size.wrapping_add(1);
            let c2rust_lvalue_ptr = &raw mut *data.items.offset(c2rust_fresh0 as isize);
            *c2rust_lvalue_ptr = token;
        }
        token = strtok_r(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            sep,
            &raw mut saveptr,
        );
    }
    let mut result: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut i_0: size_t = 0 as size_t;
    while i_0 < data.size {
        if i_0 == 0 as size_t {
            kv_do_printf(
                &raw mut result,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                *data.items.offset(i_0 as isize),
            );
        } else {
            kv_do_printf(
                &raw mut result,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                sep,
                *data.items.offset(i_0 as isize),
            );
        }
        i_0 = i_0.wrapping_add(1);
    }
    xfree(data.items as *mut ::core::ffi::c_void);
    data.capacity = 0 as size_t;
    data.size = data.capacity;
    data.items = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    xfree(ret as *mut ::core::ffi::c_void);
    return result.items;
}
#[no_mangle]
pub unsafe extern "C" fn stdpaths_get_xdg_var(idx: XDGVarType) -> *mut ::core::ffi::c_char {
    let env: *const ::core::ffi::c_char = xdg_env_vars[idx as usize];
    let fallback: *const ::core::ffi::c_char = xdg_defaults[idx as usize];
    let mut env_val: *mut ::core::ffi::c_char = os_getenv(env);
    if env_val.is_null() && os_env_exists(env, false_0 != 0) as ::core::ffi::c_int != 0 {
        env_val = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
    }
    let mut ret: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !env_val.is_null() {
        ret = env_val;
    } else if !fallback.is_null() {
        ret = expand_env_save(fallback as *mut ::core::ffi::c_char);
    } else if idx as ::core::ffi::c_int == kXDGRuntimeDir as ::core::ffi::c_int {
        ret = vim_gettempdir();
        if ret.is_null() {
            ret = b"/tmp/\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        let mut len: size_t = strlen(ret);
        ret = xmemdupz(
            ret as *const ::core::ffi::c_void,
            if len >= 2 as size_t {
                len.wrapping_sub(1 as size_t)
            } else {
                0 as size_t
            },
        ) as *mut ::core::ffi::c_char;
    }
    if (idx as ::core::ffi::c_int == kXDGDataDirs as ::core::ffi::c_int
        || idx as ::core::ffi::c_int == kXDGConfigDirs as ::core::ffi::c_int)
        && !ret.is_null()
    {
        ret = xdg_remove_duplicate(ret, ENV_SEPSTR.as_ptr());
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn get_xdg_home(idx: XDGVarType) -> *mut ::core::ffi::c_char {
    let mut dir: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(idx);
    let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
    let mut appname_len: size_t = strlen(appname);
    '_c2rust_label: {
        if appname_len
            < ((1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize)
                .wrapping_sub(::core::mem::size_of::<[::core::ffi::c_char; 6]>())
        {
        } else {
            __assert_fail(
                b"appname_len < (IOSIZE - sizeof(\"-data\"))\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/stdpaths.rs\0".as_ptr() as *const ::core::ffi::c_char,
                206 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    if !dir.is_null() {
        xmemcpyz(
            &raw mut IObuff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            appname as *const ::core::ffi::c_void,
            appname_len,
        );
        dir = concat_fnames_realloc(
            dir,
            &raw mut IObuff as *mut ::core::ffi::c_char,
            true_0 != 0,
        );
    }
    return dir;
}
#[no_mangle]
pub unsafe extern "C" fn stdpaths_user_cache_subpath(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return concat_fnames_realloc(get_xdg_home(kXDGCacheHome), fname, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn stdpaths_user_conf_subpath(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return concat_fnames_realloc(get_xdg_home(kXDGConfigHome), fname, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn stdpaths_user_data_subpath(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return concat_fnames_realloc(get_xdg_home(kXDGDataHome), fname, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn stdpaths_user_state_subpath(
    mut fname: *const ::core::ffi::c_char,
    trailing_pathseps: size_t,
    escape_commas: bool,
) -> *mut ::core::ffi::c_char {
    let mut ret: *mut ::core::ffi::c_char =
        concat_fnames_realloc(get_xdg_home(kXDGStateHome), fname, true_0 != 0);
    let len: size_t = strlen(ret);
    let numcommas: size_t = if escape_commas as ::core::ffi::c_int != 0 {
        memcnt(
            ret as *const ::core::ffi::c_void,
            ',' as ::core::ffi::c_char,
            len,
        )
    } else {
        0 as size_t
    };
    if numcommas != 0 || trailing_pathseps != 0 {
        ret = xrealloc(
            ret as *mut ::core::ffi::c_void,
            len.wrapping_add(trailing_pathseps)
                .wrapping_add(numcommas)
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        let mut i: size_t = 0 as size_t;
        while i < len.wrapping_add(numcommas) {
            if *ret.offset(i as isize) as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                memmove(
                    ret.offset(i as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    ret.offset(i as isize) as *const ::core::ffi::c_void,
                    len.wrapping_sub(i).wrapping_add(numcommas),
                );
                *ret.offset(i as isize) = '\\' as ::core::ffi::c_char;
                i = i.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        if trailing_pathseps != 0 {
            memset(
                ret.offset(len as isize).offset(numcommas as isize) as *mut ::core::ffi::c_void,
                PATHSEP,
                trailing_pathseps,
            );
        }
        *ret.offset(len.wrapping_add(trailing_pathseps).wrapping_add(numcommas) as isize) =
            NUL as ::core::ffi::c_char;
    }
    return ret;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ENV_SEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b":\0") };
