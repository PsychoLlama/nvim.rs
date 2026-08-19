//! libuv's error codes.
//!
//! Every `uv_*` function that can fail answers a negative code from this set
//! instead of setting `errno`, and on this platform each one is the negation
//! of the `errno` of the same name. Two are libuv's own and have no `errno`
//! behind them.
//!
//! c2rust copied the numbers into every file that included `uv.h` —
//! twenty-two names, thirty-seven declarations over thirteen files, sometimes
//! `pub` and sometimes not, always as bare literals with nothing tying `-2`
//! to `ENOENT`. Here there is one declaration each, and the `const` block at
//! the bottom checks every one against `libc`, so the link is the compiler's
//! and a platform whose `errno.h` disagrees fails the build.
//!
//! They stay plain `c_int` rather than becoming an enum: a `uv_*` call can
//! answer *any* code, including ones this port has never named, so an
//! exhaustive type would be a lie. What the family needed was one definition,
//! not a new type. The literal form is also load-bearing — `tools/ffigen`
//! publishes a `pub const` to the unit lane only when it can read the value
//! off the source, and `test/unit/os/{fs,fileio}_spec.lua` assert on
//! `UV_ENOENT`, `UV_EEXIST`, `UV_EISDIR`, `UV_EBADF`, `UV_ELOOP` and
//! `UV_EMLINK` by name.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

pub const UV_EACCES: c_int = -13;
pub const UV_EADDRINUSE: c_int = -98;
pub const UV_EAGAIN: c_int = -11;
pub const UV_EBADF: c_int = -9;
pub const UV_EBUSY: c_int = -16;
pub const UV_EEXIST: c_int = -17;
pub const UV_EFBIG: c_int = -27;
pub const UV_EINTR: c_int = -4;
pub const UV_EINVAL: c_int = -22;
pub const UV_EIO: c_int = -5;
pub const UV_EISDIR: c_int = -21;
pub const UV_ELOOP: c_int = -40;
pub const UV_EMLINK: c_int = -31;
pub const UV_ENOBUFS: c_int = -105;
pub const UV_ENOENT: c_int = -2;
pub const UV_ENOMEM: c_int = -12;
pub const UV_ENOTSUP: c_int = -95;
pub const UV_EPIPE: c_int = -32;
pub const UV_EROFS: c_int = -30;
pub const UV_ESRCH: c_int = -3;

/// End of file. libuv's own, not an `errno`.
pub const UV_EOF: c_int = -4095;
/// "Something failed and libuv could not say what." Also libuv's own.
pub const UV_UNKNOWN: c_int = -4094;

/// Every code above is the negation of the `errno` it is named for. The
/// literals are what `uv.h` spells; this is the proof they still agree with
/// the platform, and it is a build failure rather than a test failure.
const _: () = {
    assert!(UV_EACCES == -libc::EACCES);
    assert!(UV_EADDRINUSE == -libc::EADDRINUSE);
    assert!(UV_EAGAIN == -libc::EAGAIN);
    assert!(UV_EBADF == -libc::EBADF);
    assert!(UV_EBUSY == -libc::EBUSY);
    assert!(UV_EEXIST == -libc::EEXIST);
    assert!(UV_EFBIG == -libc::EFBIG);
    assert!(UV_EINTR == -libc::EINTR);
    assert!(UV_EINVAL == -libc::EINVAL);
    assert!(UV_EIO == -libc::EIO);
    assert!(UV_EISDIR == -libc::EISDIR);
    assert!(UV_ELOOP == -libc::ELOOP);
    assert!(UV_EMLINK == -libc::EMLINK);
    assert!(UV_ENOBUFS == -libc::ENOBUFS);
    assert!(UV_ENOENT == -libc::ENOENT);
    assert!(UV_ENOMEM == -libc::ENOMEM);
    assert!(UV_ENOTSUP == -libc::ENOTSUP);
    assert!(UV_EPIPE == -libc::EPIPE);
    assert!(UV_EROFS == -libc::EROFS);
    assert!(UV_ESRCH == -libc::ESRCH);
};
