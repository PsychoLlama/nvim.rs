//! Building the *default* 'runtimepath' at startup.
//!
//! `runtimepath_default` is pure string assembly: the XDG config and data
//! directories, each in its standard order, with `site` and `after`
//! variants, `$VIMRUNTIME`, and the library directory -- concatenated into
//! one comma-separated option value with every embedded comma and backslash
//! escaped.  `compute_double_env_sep_len` and `add_env_sep_dirs` handle the
//! directory *lists* ($XDG_CONFIG_DIRS and $XDG_DATA_DIRS), which are
//! colon-separated and appear twice each, once plain and once with a
//! suffix.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn strcpy_comma_escaped(
    mut dest: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut shift: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < len {
            if *src.add(i) as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                let c2rust_fresh22 = shift;
                shift = shift.wrapping_add(1);
                *dest.add(i.wrapping_add(c2rust_fresh22)) = '\\' as ::core::ffi::c_char;
            }
            *dest.add(i.wrapping_add(shift)) = *src.add(i);
            i = i.wrapping_add(1);
        }
        return dest.add(len.wrapping_add(shift));
    }
}

#[inline]
unsafe extern "C" fn compute_double_env_sep_len(
    val: *const ::core::ffi::c_char,
    common_suf_len: size_t,
    single_suf_len: size_t,
) -> size_t {
    unsafe {
        if val.is_null() || *val as ::core::ffi::c_int == NUL {
            return 0 as size_t;
        }
        let mut ret: size_t = 0 as size_t;
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        loop {
            let mut dir_len: size_t = 0;
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            iter = vim_env_iter(
                ENV_SEPCHAR as ::core::ffi::c_char,
                val,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if !dir.is_null() && dir_len > 0 as size_t {
                ret = ret.wrapping_add(
                    dir_len
                        .wrapping_add(memcnt(
                            dir as *const ::core::ffi::c_void,
                            ',' as ::core::ffi::c_char,
                            dir_len,
                        ))
                        .wrapping_add(common_suf_len)
                        .wrapping_add(
                            (after_pathsep(dir, dir.add(dir_len)) == 0) as ::core::ffi::c_int
                                as size_t,
                        )
                        .wrapping_mul(2 as size_t)
                        .wrapping_add(single_suf_len),
                );
            }
            if iter.is_null() {
                break;
            }
        }
        return ret;
    }
}

#[inline]
unsafe extern "C" fn add_env_sep_dirs(
    mut dest: *mut ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    suf1: *const ::core::ffi::c_char,
    len1: size_t,
    suf2: *const ::core::ffi::c_char,
    len2: size_t,
    forward: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if val.is_null() || *val as ::core::ffi::c_int == NUL {
            return dest;
        }
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
        let appname_len: size_t = strlen(appname);
        loop {
            let mut dir_len: size_t = 0;
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            iter = if forward as ::core::ffi::c_int != 0 {
                Some(
                    vim_env_iter
                        as unsafe extern "C" fn(
                            ::core::ffi::c_char,
                            *const ::core::ffi::c_char,
                            *const ::core::ffi::c_void,
                            *mut *const ::core::ffi::c_char,
                            *mut size_t,
                        )
                            -> *const ::core::ffi::c_void,
                )
            } else {
                Some(
                    vim_env_iter_rev
                        as unsafe extern "C" fn(
                            ::core::ffi::c_char,
                            *const ::core::ffi::c_char,
                            *const ::core::ffi::c_void,
                            *mut *const ::core::ffi::c_char,
                            *mut size_t,
                        )
                            -> *const ::core::ffi::c_void,
                )
            }
            .expect("non-null function pointer")(
                ENV_SEPCHAR as ::core::ffi::c_char,
                val,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if !dir.is_null() && dir_len > 0 as size_t {
                dest = strcpy_comma_escaped(dest, dir, dir_len);
                if after_pathsep(dest.offset(-(1 as ::core::ffi::c_int as isize)), dest) == 0 {
                    let c2rust_fresh23 = dest;
                    dest = dest.offset(1);
                    *c2rust_fresh23 = PATHSEP as ::core::ffi::c_char;
                }
                memmove(
                    dest as *mut ::core::ffi::c_void,
                    appname as *const ::core::ffi::c_void,
                    appname_len,
                );
                dest = dest.add(appname_len);
                if !suf1.is_null() {
                    let c2rust_fresh24 = dest;
                    dest = dest.offset(1);
                    *c2rust_fresh24 = PATHSEP as ::core::ffi::c_char;
                    memmove(
                        dest as *mut ::core::ffi::c_void,
                        suf1 as *const ::core::ffi::c_void,
                        len1,
                    );
                    dest = dest.add(len1);
                    if !suf2.is_null() {
                        let c2rust_fresh25 = dest;
                        dest = dest.offset(1);
                        *c2rust_fresh25 = PATHSEP as ::core::ffi::c_char;
                        memmove(
                            dest as *mut ::core::ffi::c_void,
                            suf2 as *const ::core::ffi::c_void,
                            len2,
                        );
                        dest = dest.add(len2);
                    }
                }
                let c2rust_fresh26 = dest;
                dest = dest.offset(1);
                *c2rust_fresh26 = ',' as ::core::ffi::c_char;
            }
            if iter.is_null() {
                break;
            }
        }
        return dest;
    }
}

#[inline]
unsafe extern "C" fn add_dir(
    mut dest: *mut ::core::ffi::c_char,
    dir: *const ::core::ffi::c_char,
    dir_len: size_t,
    type_0: XDGVarType,
    suf1: *const ::core::ffi::c_char,
    len1: size_t,
    suf2: *const ::core::ffi::c_char,
    len2: size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if dir.is_null() || dir_len == 0 as size_t {
            return dest;
        }
        dest = strcpy_comma_escaped(dest, dir, dir_len);
        let mut append_nvim: bool = type_0 as ::core::ffi::c_int
            == kXDGDataHome as ::core::ffi::c_int
            || type_0 as ::core::ffi::c_int == kXDGConfigHome as ::core::ffi::c_int;
        if append_nvim {
            if after_pathsep(dest.offset(-(1 as ::core::ffi::c_int as isize)), dest) == 0 {
                let c2rust_fresh18 = dest;
                dest = dest.offset(1);
                *c2rust_fresh18 = PATHSEP as ::core::ffi::c_char;
            }
            let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
            let mut appname_len: size_t = strlen(appname);
            debug_assert!(
                appname_len
                    < ((1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize)
                        .wrapping_sub(::core::mem::size_of::<[::core::ffi::c_char; 6]>()),
                "appname_len < (IOSIZE - sizeof(\\\"-data\\\"))"
            );
            xmemcpyz(
                IObuff.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            xmemcpyz(
                dest as *mut ::core::ffi::c_void,
                IObuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                appname_len,
            );
            dest = dest.add(appname_len);
            if !suf1.is_null() {
                let c2rust_fresh19 = dest;
                dest = dest.offset(1);
                *c2rust_fresh19 = PATHSEP as ::core::ffi::c_char;
                memmove(
                    dest as *mut ::core::ffi::c_void,
                    suf1 as *const ::core::ffi::c_void,
                    len1,
                );
                dest = dest.add(len1);
                if !suf2.is_null() {
                    let c2rust_fresh20 = dest;
                    dest = dest.offset(1);
                    *c2rust_fresh20 = PATHSEP as ::core::ffi::c_char;
                    memmove(
                        dest as *mut ::core::ffi::c_void,
                        suf2 as *const ::core::ffi::c_void,
                        len2,
                    );
                    dest = dest.add(len2);
                }
            }
        }
        let c2rust_fresh21 = dest;
        dest = dest.offset(1);
        *c2rust_fresh21 = ',' as ::core::ffi::c_char;
        return dest;
    }
}

pub unsafe extern "C" fn get_lib_dir() -> *mut ::core::ffi::c_char {
    unsafe {
        if strlen(default_lib_dir.get()) != 0 as size_t
            && os_isdir(default_lib_dir.get()) as ::core::ffi::c_int != 0
        {
            return xstrdup(default_lib_dir.get());
        }
        let mut exe_name: [::core::ffi::c_char; 4096] = [0; 4096];
        vim_get_prefix_from_exepath(&raw mut exe_name as *mut ::core::ffi::c_char);
        if append_path(
            &raw mut exe_name as *mut ::core::ffi::c_char,
            c"lib/nvim".as_ptr(),
            MAXPATHL as size_t,
        ) == OK
        {
            return xstrdup(&raw mut exe_name as *mut ::core::ffi::c_char);
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn runtimepath_default(mut clean_arg: bool) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut rtp_cur: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut rtp_size: size_t = 0 as size_t;
        let data_home: *mut ::core::ffi::c_char = if clean_arg as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            stdpaths_get_xdg_var(kXDGDataHome)
        };
        let config_home: *mut ::core::ffi::c_char = if clean_arg as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            stdpaths_get_xdg_var(kXDGConfigHome)
        };
        let vimruntime: *mut ::core::ffi::c_char = vim_getenv(c"VIMRUNTIME".as_ptr());
        let libdir: *mut ::core::ffi::c_char = get_lib_dir();
        let data_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGDataDirs);
        let config_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
        let mut data_len: size_t = 0 as size_t;
        let mut config_len: size_t = 0 as size_t;
        let mut vimruntime_len: size_t = 0 as size_t;
        let mut libdir_len: size_t = 0 as size_t;
        let mut appname_len: size_t = strlen(get_appname(false_0 != 0));
        if !data_home.is_null() {
            data_len = strlen(data_home);
            let mut nvim_data_size: size_t = appname_len;
            if data_len != 0 as size_t {
                rtp_size = (rtp_size as ::core::ffi::c_ulong).wrapping_add(
                    data_len
                        .wrapping_add(memcnt(
                            data_home as *const ::core::ffi::c_void,
                            ',' as ::core::ffi::c_char,
                            data_len,
                        ))
                        .wrapping_add(nvim_data_size)
                        .wrapping_add(1 as size_t)
                        .wrapping_add(SITE_SIZE)
                        .wrapping_add(1 as size_t)
                        .wrapping_add(
                            (after_pathsep(data_home, data_home.add(data_len)) == 0)
                                as ::core::ffi::c_int as size_t,
                        )
                        .wrapping_mul(2 as size_t)
                        .wrapping_add(AFTER_SIZE)
                        .wrapping_add(1 as size_t) as ::core::ffi::c_ulong,
                ) as size_t;
            }
        }
        if !config_home.is_null() {
            config_len = strlen(config_home);
            if config_len != 0 as size_t {
                rtp_size = (rtp_size as ::core::ffi::c_ulong).wrapping_add(
                    config_len
                        .wrapping_add(memcnt(
                            config_home as *const ::core::ffi::c_void,
                            ',' as ::core::ffi::c_char,
                            config_len,
                        ))
                        .wrapping_add(appname_len)
                        .wrapping_add(1 as size_t)
                        .wrapping_add(
                            (after_pathsep(config_home, config_home.add(config_len)) == 0)
                                as ::core::ffi::c_int as size_t,
                        )
                        .wrapping_mul(2 as size_t)
                        .wrapping_add(AFTER_SIZE)
                        .wrapping_add(1 as size_t) as ::core::ffi::c_ulong,
                ) as size_t;
            }
        }
        if !vimruntime.is_null() {
            vimruntime_len = strlen(vimruntime);
            if vimruntime_len != 0 as size_t {
                rtp_size = rtp_size.wrapping_add(
                    vimruntime_len
                        .wrapping_add(memcnt(
                            vimruntime as *const ::core::ffi::c_void,
                            ',' as ::core::ffi::c_char,
                            vimruntime_len,
                        ))
                        .wrapping_add(1 as size_t),
                );
            }
        }
        if !libdir.is_null() {
            libdir_len = strlen(libdir);
            if libdir_len != 0 as size_t {
                rtp_size = rtp_size.wrapping_add(
                    libdir_len
                        .wrapping_add(memcnt(
                            libdir as *const ::core::ffi::c_void,
                            ',' as ::core::ffi::c_char,
                            libdir_len,
                        ))
                        .wrapping_add(1 as size_t),
                );
            }
        }
        rtp_size = rtp_size.wrapping_add(compute_double_env_sep_len(
            data_dirs,
            appname_len
                .wrapping_add(1 as size_t)
                .wrapping_add(SITE_SIZE)
                .wrapping_add(1 as size_t),
            AFTER_SIZE.wrapping_add(1 as size_t),
        ));
        rtp_size = rtp_size.wrapping_add(compute_double_env_sep_len(
            config_dirs,
            appname_len.wrapping_add(1 as size_t),
            AFTER_SIZE.wrapping_add(1 as size_t),
        ));
        let mut rtp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if rtp_size != 0 as size_t {
            rtp = xmalloc(rtp_size) as *mut ::core::ffi::c_char;
            rtp_cur = rtp;
            rtp_cur = add_dir(
                rtp_cur,
                config_home,
                config_len,
                kXDGConfigHome,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
            );
            rtp_cur = add_env_sep_dirs(
                rtp_cur,
                config_dirs,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
                true_0 != 0,
            );
            rtp_cur = add_dir(
                rtp_cur,
                data_home,
                data_len,
                kXDGDataHome,
                c"site".as_ptr(),
                SITE_SIZE,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
            );
            rtp_cur = add_env_sep_dirs(
                rtp_cur,
                data_dirs,
                c"site".as_ptr(),
                SITE_SIZE,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
                true_0 != 0,
            );
            rtp_cur = add_dir(
                rtp_cur,
                vimruntime,
                vimruntime_len,
                kXDGNone,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
            );
            rtp_cur = add_dir(
                rtp_cur,
                libdir,
                libdir_len,
                kXDGNone,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
            );
            rtp_cur = add_env_sep_dirs(
                rtp_cur,
                data_dirs,
                c"site".as_ptr(),
                SITE_SIZE,
                c"after".as_ptr(),
                AFTER_SIZE,
                false_0 != 0,
            );
            rtp_cur = add_dir(
                rtp_cur,
                data_home,
                data_len,
                kXDGDataHome,
                c"site".as_ptr(),
                SITE_SIZE,
                c"after".as_ptr(),
                AFTER_SIZE,
            );
            rtp_cur = add_env_sep_dirs(
                rtp_cur,
                config_dirs,
                c"after".as_ptr(),
                AFTER_SIZE,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
                false_0 != 0,
            );
            rtp_cur = add_dir(
                rtp_cur,
                config_home,
                config_len,
                kXDGConfigHome,
                c"after".as_ptr(),
                AFTER_SIZE,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as size_t,
            );
            *rtp_cur.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            debug_assert!(
                rtp_cur.offset_from(rtp) as size_t == rtp_size,
                "(size_t)(rtp_cur - rtp) == rtp_size"
            );
        }
        xfree(data_dirs as *mut ::core::ffi::c_void);
        xfree(config_dirs as *mut ::core::ffi::c_void);
        xfree(data_home as *mut ::core::ffi::c_void);
        xfree(config_home as *mut ::core::ffi::c_void);
        xfree(vimruntime as *mut ::core::ffi::c_void);
        xfree(libdir as *mut ::core::ffi::c_void);
        return rtp;
    }
}

pub const SITE_SIZE: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1_usize);

pub const AFTER_SIZE: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1_usize);
