//! The libuv event loop and the streams, processes and timers on it.

use core::ffi::{c_int, c_void};
use core::ptr;

pub mod libuv;
pub mod libuv_proc;
pub mod r#loop;
pub mod multiqueue;
pub mod proc;
pub mod rstream;
pub mod signal;
pub mod socket;
pub mod stream;
pub mod time;
pub mod wstream;

/// A small integer packed into an event argument.
///
/// Upstream's `CREATE_EVENT` carries an exit status or a libuv result through
/// the `void *` argv slots rather than allocating for it, and the handler
/// casts it straight back. Nothing ever dereferences one of these, so the
/// pointer only has to survive the round trip.
pub fn pack_int(n: c_int) -> *mut c_void {
    let bits = isize::try_from(n).expect("a C int fits in a pointer-sized integer");
    ptr::with_exposed_provenance_mut(bits.cast_unsigned())
}

/// The integer back out of an event argument. The inverse of [`pack_int`],
/// which is the only thing that writes one.
pub fn unpack_int(arg: *mut c_void) -> c_int {
    let bits = arg.expose_provenance().cast_signed();
    c_int::try_from(bits).expect("only `pack_int` writes an argument read this way")
}

#[cfg(test)]
mod tests {
    use super::{pack_int, unpack_int};
    use core::ffi::c_int;

    #[test]
    fn an_event_argument_round_trips_a_c_int() {
        for n in [0, 1, -1, 128, -2, c_int::MAX, c_int::MIN] {
            assert_eq!(unpack_int(pack_int(n)), n, "round trip of {n}");
        }
    }
}
