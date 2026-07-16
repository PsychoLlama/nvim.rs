extern "C" {
    fn uv_dlopen(filename: *const ::core::ffi::c_char, lib: *mut uv_lib_t) -> ::core::ffi::c_int;
    fn uv_dlclose(lib: *mut uv_lib_t);
    fn uv_dlsym(
        lib: *mut uv_lib_t,
        name: *const ::core::ffi::c_char,
        ptr: *mut *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn uv_dlerror(lib: *const uv_lib_t) -> *const ::core::ffi::c_char;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
}
pub type intptr_t = isize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_lib_t {
    pub handle: *mut ::core::ffi::c_void,
    pub errmsg: *mut ::core::ffi::c_char,
}
pub type int_int_fn = Option<unsafe extern "C" fn(::core::ffi::c_int) -> ::core::ffi::c_int>;
pub type gen_fn = Option<unsafe extern "C" fn() -> ()>;
pub type str_int_fn =
    Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
pub type int_str_fn =
    Option<unsafe extern "C" fn(::core::ffi::c_int) -> *const ::core::ffi::c_char>;
pub type str_str_fn =
    Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> *const ::core::ffi::c_char>;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[no_mangle]
pub unsafe extern "C" fn os_libcall(
    mut libname: *const ::core::ffi::c_char,
    mut funcname: *const ::core::ffi::c_char,
    mut argv: *const ::core::ffi::c_char,
    mut argi: ::core::ffi::c_int,
    mut str_out: *mut *mut ::core::ffi::c_char,
    mut int_out: *mut ::core::ffi::c_int,
) -> bool {
    if libname.is_null() || funcname.is_null() {
        return false_0 != 0;
    }
    let mut lib: uv_lib_t = uv_lib_t {
        handle: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if uv_dlopen(libname, &raw mut lib) != 0 {
        semsg(
            b"dlerror = \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            uv_dlerror(&raw mut lib),
        );
        uv_dlclose(&raw mut lib);
        return false_0 != 0;
    }
    let mut fn_0: gen_fn = None;
    if uv_dlsym(
        &raw mut lib,
        funcname,
        &raw mut fn_0 as *mut *mut ::core::ffi::c_void,
    ) != 0
    {
        semsg(
            b"dlerror = \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            uv_dlerror(&raw mut lib),
        );
        uv_dlclose(&raw mut lib);
        return false_0 != 0;
    }
    if !str_out.is_null() {
        let mut sfn: str_str_fn = ::core::mem::transmute::<gen_fn, str_str_fn>(fn_0);
        let mut ifn: int_str_fn = ::core::mem::transmute::<gen_fn, int_str_fn>(fn_0);
        let mut res: *const ::core::ffi::c_char = if !argv.is_null() {
            sfn.expect("non-null function pointer")(argv)
        } else {
            ifn.expect("non-null function pointer")(argi)
        };
        *str_out = if !res.is_null()
            && res.expose_addr() as intptr_t != 1 as ::core::ffi::c_int as intptr_t
            && res.expose_addr() as intptr_t != -1 as ::core::ffi::c_int as intptr_t
        {
            xstrdup(res)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    } else {
        let mut sfn_0: str_int_fn = ::core::mem::transmute::<gen_fn, str_int_fn>(fn_0);
        let mut ifn_0: int_int_fn = ::core::mem::transmute::<gen_fn, int_int_fn>(fn_0);
        *int_out = if !argv.is_null() {
            sfn_0.expect("non-null function pointer")(argv)
        } else {
            ifn_0.expect("non-null function pointer")(argi)
        };
    }
    uv_dlclose(&raw mut lib);
    return true_0 != 0;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
