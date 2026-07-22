//! Shared libc/system `extern "C"` declarations (phase 5b).
//!
//! One declaration per symbol, `use`d by every consumer, instead of
//! the per-module copies c2rust emitted. Everything here resolves
//! against the platform C library at link time.

use crate::src::nvim::types::*;

extern "C" {
    pub fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    pub fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    pub fn __errno_location() -> *mut ::core::ffi::c_int;
    pub fn _exit(__status: ::core::ffi::c_int) -> !;
    pub fn abort() -> !;
    pub fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn acos(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn asin(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn atan(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn atan2(__y: ::core::ffi::c_double, __x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn atol(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
    pub fn backtrace(
        __array: *mut *mut ::core::ffi::c_void,
        __size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn bind_textdomain_codeset(
        __domainname: *const ::core::ffi::c_char,
        __codeset: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn bindtextdomain(
        __domainname: *const ::core::ffi::c_char,
        __dirname: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn bsearch(
        __key: *const ::core::ffi::c_void,
        __base: *const ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn calloc(nmemb: usize, size: usize) -> *mut ::core::ffi::c_void;
    pub fn ceil(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn cfsetispeed(__termios_p: *mut termios, __speed: speed_t) -> ::core::ffi::c_int;
    pub fn cfsetospeed(__termios_p: *mut termios, __speed: speed_t) -> ::core::ffi::c_int;
    pub fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn cos(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn cosh(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn dup(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn dup2(__fd: ::core::ffi::c_int, __fd2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn endpwent();
    pub static mut environ: *mut *mut ::core::ffi::c_char;
    pub fn execvp(
        __file: *const ::core::ffi::c_char,
        __argv: *const *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn exit(__status: ::core::ffi::c_int) -> !;
    pub fn exp(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn fabs(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fcntl(__fd: ::core::ffi::c_int, __cmd: ::core::ffi::c_int, ...) -> ::core::ffi::c_int;
    pub fn fdopen(__fd: ::core::ffi::c_int, __modes: *const ::core::ffi::c_char) -> *mut FILE;
    pub fn feof(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn ferror(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fflush(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fgetc(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fgets(
        __s: *mut ::core::ffi::c_char,
        __n: ::core::ffi::c_int,
        __stream: *mut FILE,
    ) -> *mut ::core::ffi::c_char;
    pub fn fileno(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn flock(__fd: ::core::ffi::c_int, __operation: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn floor(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn fmod(__x: ::core::ffi::c_double, __y: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    pub fn forkpty(
        __amaster: *mut ::core::ffi::c_int,
        __name: *mut ::core::ffi::c_char,
        __termp: *const termios,
        __winp: *const winsize,
    ) -> ::core::ffi::c_int;
    pub fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn fputc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fputs(__s: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    pub fn free(__ptr: *mut ::core::ffi::c_void);
    pub fn freopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
        __stream: *mut FILE,
    ) -> *mut FILE;
    pub fn fscanf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn fseek(
        __stream: *mut FILE,
        __off: ::core::ffi::c_long,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn fseeko(
        __stream: *mut FILE,
        __off: __off_t,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn ftell(__stream: *mut FILE) -> ::core::ffi::c_long;
    pub fn ftello(__stream: *mut FILE) -> __off_t;
    pub fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __s: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    pub fn getc(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn getgid() -> __gid_t;
    pub fn getpid() -> __pid_t;
    pub fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn getuid() -> __uid_t;
    pub fn getxattr(
        __path: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __value: *mut ::core::ffi::c_void,
        __size: size_t,
    ) -> ssize_t;
    pub fn iconv(
        __cd: iconv_t,
        __inbuf: *mut *mut ::core::ffi::c_char,
        __inbytesleft: *mut size_t,
        __outbuf: *mut *mut ::core::ffi::c_char,
        __outbytesleft: *mut size_t,
    ) -> size_t;
    pub fn iconv_close(__cd: iconv_t) -> ::core::ffi::c_int;
    pub fn iconv_open(
        __tocode: *const ::core::ffi::c_char,
        __fromcode: *const ::core::ffi::c_char,
    ) -> iconv_t;
    pub fn ioctl(
        __fd: ::core::ffi::c_int,
        __request: ::core::ffi::c_ulong,
        ...
    ) -> ::core::ffi::c_int;
    pub fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn killpg(__pgrp: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn labs(__x: ::core::ffi::c_long) -> ::core::ffi::c_long;
    pub fn listxattr(
        __path: *const ::core::ffi::c_char,
        __list: *mut ::core::ffi::c_char,
        __size: size_t,
    ) -> ssize_t;
    pub fn llabs(__x: ::core::ffi::c_longlong) -> ::core::ffi::c_longlong;
    pub fn localtime_r(__timer: *const time_t, __tp: *mut tm) -> *mut tm;
    pub fn log(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn log10(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn lseek(
        __fd: ::core::ffi::c_int,
        __offset: __off_t,
        __whence: ::core::ffi::c_int,
    ) -> __off_t;
    pub fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    pub fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn mktime(__tp: *mut tm) -> time_t;
    pub fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    pub fn ntohs(__netshort: uint16_t) -> uint16_t;
    pub fn pclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn popen(
        __command: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    pub fn pow(__x: ::core::ffi::c_double, __y: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    pub fn pthread_exit(__retval: *mut ::core::ffi::c_void) -> !;
    pub fn ptsname(__fd: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    pub fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    pub fn read(
        __fd: ::core::ffi::c_int,
        __buf: *mut ::core::ffi::c_void,
        __nbytes: size_t,
    ) -> ssize_t;
    pub fn readlink(
        __path: *const ::core::ffi::c_char,
        __buf: *mut ::core::ffi::c_char,
        __len: size_t,
    ) -> ssize_t;
    pub fn readv(
        __fd: ::core::ffi::c_int,
        __iovec: *const iovec,
        __count: ::core::ffi::c_int,
    ) -> ssize_t;
    pub fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    pub fn round(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn setbuf(__stream: *mut FILE, __buf: *mut ::core::ffi::c_char);
    pub fn setlocale(
        __category: ::core::ffi::c_int,
        __locale: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn setpwent();
    pub fn setsid() -> __pid_t;
    pub fn setvbuf(
        __stream: *mut FILE,
        __buf: *mut ::core::ffi::c_char,
        __modes: ::core::ffi::c_int,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn setxattr(
        __path: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __value: *const ::core::ffi::c_void,
        __size: size_t,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn sin(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn sinh(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn sprintf(
        __s: *mut ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn sqrt(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub static mut stderr: *mut FILE;
    pub static mut stdout: *mut FILE;
    pub fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strcoll(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    pub fn strftime(
        __s: *mut ::core::ffi::c_char,
        __maxsize: size_t,
        __format: *const ::core::ffi::c_char,
        __tp: *const tm,
    ) -> size_t;
    pub fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    pub fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn strncpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> *mut ::core::ffi::c_char;
    pub fn strnlen(__string: *const ::core::ffi::c_char, __maxlen: size_t) -> size_t;
    pub fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strptime(
        __s: *const ::core::ffi::c_char,
        __fmt: *const ::core::ffi::c_char,
        __tp: *mut tm,
    ) -> *mut ::core::ffi::c_char;
    pub fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    pub fn strtoimax(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> intmax_t;
    pub fn strtok_r(
        __s: *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
        __save_ptr: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    pub fn strtoll(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    pub fn strtoul(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong;
    pub fn symlink(
        __from: *const ::core::ffi::c_char,
        __to: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn tan(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn tanh(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn tcdrain(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn tcgetattr(__fd: ::core::ffi::c_int, __termios_p: *mut termios) -> ::core::ffi::c_int;
    pub fn tcsetattr(
        __fd: ::core::ffi::c_int,
        __optional_actions: ::core::ffi::c_int,
        __termios_p: *const termios,
    ) -> ::core::ffi::c_int;
    pub fn textdomain(__domainname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn time(__timer: *mut time_t) -> time_t;
    pub fn tolower(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn toupper(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn trunc(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    pub fn tzset();
    pub fn umask(__mask: __mode_t) -> __mode_t;
    pub fn ungetc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn vfprintf(
        __s: *mut FILE,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    pub fn vsnprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    pub fn waitpid(
        __pid: __pid_t,
        __stat_loc: *mut ::core::ffi::c_int,
        __options: ::core::ffi::c_int,
    ) -> __pid_t;
    pub fn write(
        __fd: ::core::ffi::c_int,
        __buf: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ssize_t;
}
