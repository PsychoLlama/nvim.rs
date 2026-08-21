#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
//! The msgpack-RPC transport: channels, the server that accepts them, and
//! the packer/unpacker pair that moves API values over the wire.

/// One log line in `logmsg`'s plain shape: no context tag, the upstream
/// function name and line number, newline-terminated.
///
/// Spelled out, a `logmsg_c!` call is six fixed arguments before the format
/// string and wraps over eight lines — eight lines of *unchecked* code, since
/// the whole call has to sit inside the region. Naming the fixed half here
/// leaves one line per site.
macro_rules! log {
    ($level:expr, $who:expr, $line:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::log::logmsg_c!(
            $level,
            core::ptr::null(),
            $who.as_ptr(),
            $line,
            true,
            $fmt.as_ptr()
            $(, $arg)*
        )
    };
}

pub mod channel;
pub mod packer;
pub mod server;
pub mod unpacker;
