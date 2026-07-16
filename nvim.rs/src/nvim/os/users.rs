extern "C" {
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn setpwent();
    fn endpwent();
    fn getpwent() -> *mut passwd;
    fn getpwuid(__uid: __uid_t) -> *mut passwd;
    fn getpwnam(__name: *const ::core::ffi::c_char) -> *mut passwd;
    fn getuid() -> __uid_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn os_getenv_noalloc(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
}
pub type size_t = usize;
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type int32_t = i32;
pub type uint64_t = u64;
pub type uid_t = __uid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct passwd {
    pub pw_name: *mut ::core::ffi::c_char,
    pub pw_passwd: *mut ::core::ffi::c_char,
    pub pw_uid: __uid_t,
    pub pw_gid: __gid_t,
    pub pw_gecos: *mut ::core::ffi::c_char,
    pub pw_dir: *mut ::core::ffi::c_char,
    pub pw_shell: *mut ::core::ffi::c_char,
}
pub type uv_uid_t = uid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type linenr_T = int32_t;
pub type colnr_T = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
pub type LuaRef = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sctx_T {
    pub sc_sid: scid_T,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: linenr_T,
    pub sc_chan: uint64_t,
}
pub type scid_T = ::core::ffi::c_int;
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type xp_prefix_T = ::core::ffi::c_uint;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ::core::ffi::c_int,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: ::core::ffi::c_int,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut ga_users: garray_T = GA_EMPTY_INIT_VALUE;
unsafe extern "C" fn add_user(
    mut users: *mut garray_T,
    mut user: *mut ::core::ffi::c_char,
    mut need_copy: bool,
) {
    let mut user_copy: *mut ::core::ffi::c_char =
        if !user.is_null() && need_copy as ::core::ffi::c_int != 0 {
            xstrdup(user)
        } else {
            user
        };
    if user_copy.is_null() || *user_copy as ::core::ffi::c_int == NUL {
        if need_copy {
            xfree(user_copy as *mut ::core::ffi::c_void);
        }
        return;
    }
    ga_grow(users, 1 as ::core::ffi::c_int);
    *((*users).ga_data as *mut *mut ::core::ffi::c_char).offset((*users).ga_len as isize) =
        user_copy;
    (*users).ga_len += 1;
}
#[no_mangle]
pub unsafe extern "C" fn os_get_usernames(mut users: *mut garray_T) -> ::core::ffi::c_int {
    if users.is_null() {
        return FAIL;
    }
    ga_init(
        users,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    setpwent();
    loop {
        pw = getpwent();
        if pw.is_null() {
            break;
        }
        add_user(users, (*pw).pw_name, true_0 != 0);
    }
    endpwent();
    let mut user_env: *mut ::core::ffi::c_char =
        os_getenv_noalloc(b"USER\0".as_ptr() as *const ::core::ffi::c_char);
    if !user_env.is_null() && *user_env as ::core::ffi::c_int != NUL {
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < (*users).ga_len {
            let mut local_user: *mut ::core::ffi::c_char =
                *((*users).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
            if strcmp(local_user, user_env) == 0 as ::core::ffi::c_int {
                break;
            }
            i += 1;
        }
        if i == (*users).ga_len {
            let mut pw_0: *mut passwd = getpwnam(user_env);
            if !pw_0.is_null() {
                add_user(users, (*pw_0).pw_name, true_0 != 0);
            }
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn os_get_username(
    mut s: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    return os_get_uname(getuid(), s, len);
}
#[no_mangle]
pub unsafe extern "C" fn os_get_uname(
    mut uid: uv_uid_t,
    mut s: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    pw = getpwuid(uid as __uid_t);
    if !pw.is_null() && !(*pw).pw_name.is_null() && *(*pw).pw_name as ::core::ffi::c_int != NUL {
        xstrlcpy(s, (*pw).pw_name, len);
        return OK;
    }
    snprintf(
        s,
        len,
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        uid as ::core::ffi::c_int,
    );
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn os_get_userdir(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if name.is_null() || *name as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut pw: *mut passwd = getpwnam(name);
    if !pw.is_null() {
        return xstrdup((*pw).pw_dir);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn init_users() {
    static mut lazy_init_done: bool = false_0 != 0;
    if lazy_init_done {
        return;
    }
    lazy_init_done = true_0 != 0;
    os_get_usernames(&raw mut ga_users);
}
#[no_mangle]
pub unsafe extern "C" fn get_users(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    init_users();
    if idx < ga_users.ga_len {
        return *(ga_users.ga_data as *mut *mut ::core::ffi::c_char).offset(idx as isize);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn match_user(mut name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = strlen(name) as ::core::ffi::c_int;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    init_users();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ga_users.ga_len {
        if strcmp(
            *(ga_users.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
            name,
        ) == 0 as ::core::ffi::c_int
        {
            return 2 as ::core::ffi::c_int;
        }
        if strncmp(
            *(ga_users.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
            name,
            n as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            result = 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return result;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
