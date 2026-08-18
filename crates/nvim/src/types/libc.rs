#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
//
// The nominal C aggregates every libc prototype names -- `FILE`, `termios`,
// `winsize`, `tm`, `iovec` -- are re-exports of the `libc` crate's, so a
// declaration there and a call site here agree by construction instead of by
// inspection.
//
// The scalar typedefs stay spelled out on purpose. A type alias is
// transparent, so `time_t = c_long` already *is* `libc::time_t`; re-exporting
// would buy no type identity and would cost the unit-test chunk its
// derivation (tools/ffigen treats a `::libc::` path as owned by the harness's
// system preamble, which does not declare all of these).
//
// `pthread_mutex_t`/`pthread_rwlock_t` stay for the same reason plus one
// more: nothing here hands one to libc. They are reached only as libuv's
// `uv_mutex_t`/`uv_rwlock_t`, embedded by value in `uv_loop_t` and friends,
// so the chunk needs their layout and this crate is the only place that
// describes libuv's ABI.

// The standard descriptors, which a dozen modules had each spelled out.
pub use ::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};

pub use ::libc::{FILE, iovec, termios, tm, winsize};

pub type __builtin_va_list = [__va_list_tag; 1];
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type __gnuc_va_list = __builtin_va_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_rwlock_arch_t {
    pub __readers: ::core::ffi::c_uint,
    pub __writers: ::core::ffi::c_uint,
    pub __wrphase_futex: ::core::ffi::c_uint,
    pub __writers_futex: ::core::ffi::c_uint,
    pub __pad3: ::core::ffi::c_uint,
    pub __pad4: ::core::ffi::c_uint,
    pub __cur_writer: ::core::ffi::c_int,
    pub __shared: ::core::ffi::c_int,
    pub __rwelision: ::core::ffi::c_schar,
    pub __pad1: [::core::ffi::c_uchar; 7],
    pub __pad2: ::core::ffi::c_ulong,
    pub __flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type cc_t = ::core::ffi::c_uchar;
pub type gid_t = ::core::ffi::c_uint;
pub type iconv_t = *mut ::core::ffi::c_void;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type int8_t = i8;
pub type intmax_t = ::libc::intmax_t;
pub type intptr_t = isize;
pub type off_t = ::core::ffi::c_long;
pub type pid_t = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_rwlock_t {
    pub __data: __pthread_rwlock_arch_t,
    pub __size: [::core::ffi::c_char; 56],
    pub __align: ::core::ffi::c_long,
}
pub type pthread_t = ::core::ffi::c_ulong;
pub type ptrdiff_t = isize;
pub type sa_family_t = ::core::ffi::c_ushort;
pub type size_t = usize;
pub type socklen_t = ::core::ffi::c_uint;
pub type speed_t = ::core::ffi::c_uint;
pub type tcflag_t = ::core::ffi::c_uint;
pub type time_t = ::core::ffi::c_long;
pub type uid_t = ::core::ffi::c_uint;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uint8_t = u8;
pub type uintmax_t = ::libc::uintmax_t;
pub type uintptr_t = usize;
pub type va_list = __gnuc_va_list;
