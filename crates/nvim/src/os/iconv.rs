//! iconv: the C library's character-set converter, behind an owned
//! descriptor.
//!
//! iconv is a stateful C library reached through an opaque descriptor. It
//! writes into caller-owned buffers whose remaining room it reports back by
//! decrementing a counter, and it signals every one of its outcomes through
//! `errno`. [`Iconv`] is the descriptor with an owner -- dropping it closes
//! it -- and [`Iconv::step`] is the one conversion call, over slices, with
//! `errno` handed back as a value. Nothing above this file touches iconv.
//!
//! [`Iconv::open`] also *probes* the host's iconv, because a dynamically
//! loaded one can accept an `iconv_open` and then convert nothing: the first
//! descriptor this ever opens is asked to convert no input, which a working
//! iconv answers by leaving the output pointer alone and a broken one by
//! nulling it. Once iconv is known broken this never tries again.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::NonNull;

use ::libc::{__errno_location, iconv, iconv_close, iconv_open};

use crate::global_cell::GlobalCell;

/// Has the host's iconv been proved to work?
#[derive(Clone, Copy, PartialEq, Eq)]
enum Working {
    Unknown,
    Yes,
    Broken,
}

static WORKING: GlobalCell<Working> = GlobalCell::new(Working::Unknown);

/// `(iconv_t)-1`: what `iconv_open` answers when it cannot convert a pair.
fn failed() -> *mut c_void {
    core::ptr::without_provenance_mut(usize::MAX)
}

/// An open iconv descriptor, closed when dropped.
pub struct Iconv(NonNull<c_void>);

impl Iconv {
    /// A descriptor converting `from` to `to`, or `None` when iconv cannot
    /// convert that pair -- or has been found not to convert anything.
    pub(crate) fn open(to: &CStr, from: &CStr) -> Option<Iconv> {
        if WORKING.get() == Working::Broken {
            return None;
        }
        // SAFETY: two NUL-terminated names.
        let fd = unsafe { iconv_open(to.as_ptr(), from.as_ptr()) };
        let fd = NonNull::new(fd).filter(|fd| fd.as_ptr() != failed())?;
        let descriptor = Iconv(fd);
        if WORKING.get() != Working::Unknown {
            return Some(descriptor);
        }
        let mut probe = [0 as c_char; 400];
        let mut out = probe.as_mut_ptr();
        let mut room = probe.len();
        // SAFETY: an open descriptor, no input, and `room` writable bytes at
        // `out`.
        unsafe {
            iconv(
                fd.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut out,
                &raw mut room,
            )
        };
        if out.is_null() {
            WORKING.set(Working::Broken);
            return None;
        }
        WORKING.set(Working::Yes);
        Some(descriptor)
    }

    /// Convert as much of `input` into `output` as fits, stepping both past
    /// what was consumed and produced. `Err` is iconv's `errno` when it
    /// stopped short: out of room, an invalid sequence, or an incomplete one
    /// at the end of `input`.
    pub(crate) fn step(&self, input: &mut &[u8], output: &mut &mut [u8]) -> Result<(), c_int> {
        let mut from = input.as_ptr().cast::<c_char>().cast_mut();
        let mut from_left = input.len();
        let mut to = output.as_mut_ptr().cast::<c_char>();
        let mut to_left = output.len();
        // SAFETY: `errno` is the C library's thread-local; the descriptor is
        // open, iconv reads at most `from_left` bytes at `from` (and never
        // writes them) and writes at most `to_left` at `to`.
        let (status, errno) = unsafe {
            *__errno_location() = 0;
            let status = iconv(
                self.0.as_ptr(),
                &raw mut from,
                &raw mut from_left,
                &raw mut to,
                &raw mut to_left,
            );
            (status, *__errno_location())
        };
        let consumed = input.len() - from_left;
        let produced = output.len() - to_left;
        *input = &input[consumed..];
        let rest = core::mem::take(output);
        *output = &mut rest[produced..];
        if status == usize::MAX {
            Err(errno)
        } else {
            Ok(())
        }
    }

    /// Write the sequence that puts the output in its initial shift state
    /// into `output`, stepping it past what was written: iconv called with no
    /// input. Some iconvs (glibc's among them) null the output pointer on
    /// this call and leave its count meaningless; `output` is then left as
    /// it was, which is what upstream's workaround does.
    pub(crate) fn shift_initial(&self, output: &mut &mut [u8]) {
        let mut to = output.as_mut_ptr().cast::<c_char>();
        let mut to_left = output.len();
        // SAFETY: the descriptor is open, a null input asks for the shift
        // sequence only, and iconv writes at most `to_left` bytes at `to`.
        unsafe {
            iconv(
                self.0.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut to,
                &raw mut to_left,
            )
        };
        if to.is_null() || to_left > output.len() {
            return;
        }
        let produced = output.len() - to_left;
        let rest = core::mem::take(output);
        *output = &mut rest[produced..];
    }
}

impl Drop for Iconv {
    fn drop(&mut self) {
        // SAFETY: the descriptor this value owns, closed once.
        unsafe { iconv_close(self.0.as_ptr()) };
    }
}
