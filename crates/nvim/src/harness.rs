//! The names the in-repo test harnesses resolve in the binary, and nothing
//! else.
//!
//! Each item here is a `metrics/abi-ledger.jsonl` row of the `test` class
//! (see `docs/abi-ledger.md` for who reaches it and what would retire it).
//! `#[unsafe(no_mangle)]` is itself an unsafe attribute, which a file that
//! forbids unsafe code refuses, so the exports live in this one file and the
//! modules whose behaviour they reach -- `autocmd/`, `drawscreen/`, `grid/`,
//! `mbyte/`, `ui/` -- can forbid it. The functions are fronts over the
//! modules' own slice forms, and read the C side's pointers here and nowhere
//! else; `display_tick` and `utf8len_tab` are data symbols, so the static
//! itself lives here and its module re-exports it.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `harness.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::grid::{SCHAR_BUF_SIZE, SCharBuf, schar_copy};
use crate::mbyte::MB_MAXCHAR;
use crate::mbyte::{char_at, char_len, lead_byte_lengths, sequence_value};
use crate::os::libc::{int32_t, size_t, uint8_t, uintptr_t};
use crate::types::ApiString;
use crate::types::{DispTick, ScreenChar};
use core::ffi::{c_char, c_int};
use core::slice;

/// The redraw pass counter a decoration provider compares against to know it
/// has already been asked about this pass (`drawscreen::state` re-exports
/// it). Exported as data: `test/functional/lua/ffi_spec.lua` declares it
/// with `ffi.cdef` and reads it.
#[unsafe(no_mangle)]
pub static display_tick: GlobalCell<DispTick> = GlobalCell::new(0);

/// How far through startup the editor is (`startup::starting` re-exports
/// it). Exported as data: the oldtest harness (`test/old/testdir/runtest.vim`)
/// writes it through `test_override('starting', ..)` by symbol name.
#[unsafe(no_mangle)]
pub static starting: GlobalCell<c_int> = GlobalCell::new(2);

/// Whether `char_avail()` answers false regardless of the input
/// (`getchar::state` re-exports it). Exported as data: the oldtest harness
/// writes it through `test_override('char_avail', ..)` by symbol name.
#[unsafe(no_mangle)]
pub static test_disable_char_avail: GlobalCell<bool> = GlobalCell::new(false);

/// [`crate::startup::early_init`], by name: `test/unit/testutil.lua`
/// boots an editor without a `main`.
#[unsafe(no_mangle)]
pub extern "C" fn early_init(clean: bool) {
    crate::startup::early_init(clean);
}

/// [`crate::startup::event_init`], by name, for the same harness.
#[unsafe(no_mangle)]
pub extern "C" fn event_init() {
    crate::startup::event_init();
}

/// [`crate::autocmd::block_autocmds`], by name:
/// `test/functional/terminal/tui_spec.lua` blocks autocommands through the
/// FFI.
#[unsafe(no_mangle)]
pub extern "C" fn block_autocmds() {
    crate::autocmd::block_autocmds();
}

/// A string as the spec builds one with LuaJIT's FFI: C's `{ char *data;
/// size_t size; }`, over a buffer the spec owns.
///
/// The fields are private, so only the C side ever makes one; the two fronts
/// below copy the bytes out and keep nothing.
#[repr(C)]
#[derive(Debug)]
pub struct HarnessString {
    data: *const c_char,
    size: size_t,
}

impl HarnessString {
    /// A copy of the bytes, as an API string.
    fn to_api(&self) -> ApiString {
        if self.data.is_null() {
            return ApiString::NULL;
        }
        // SAFETY: only the spec builds one, over `size` bytes it owns for
        // the length of the call.
        ApiString::from_bytes(unsafe { slice::from_raw_parts(self.data.cast::<u8>(), self.size) })
    }
}

/// [`crate::ui::ui_call_set_title`], by name: `tui_spec.lua` sends a
/// 65,536-byte title, which `maketitle()` never produces.
#[unsafe(no_mangle)]
pub extern "C" fn ui_call_set_title(title: HarnessString) {
    crate::ui::ui_call_set_title(title.to_api());
}

/// [`crate::ui::ui_call_chdir`], by name: the same spec checks that a
/// relative path the server sends is resolved against the server's cwd.
#[unsafe(no_mangle)]
pub extern "C" fn ui_call_chdir(path: HarnessString) {
    crate::ui::ui_call_chdir(path.to_api());
}

/// [`crate::autocmd::unblock_autocmds`], by name, for the same spec.
#[unsafe(no_mangle)]
pub extern "C" fn unblock_autocmds() {
    crate::autocmd::unblock_autocmds();
}

/// Write `sc` to `buf_out` and terminate it, answering the byte length.
///
/// The C harness's spelling (`test/unit/fixtures/vterm_test.c`,
/// `vterm_spec.lua`, the battery's `vtsweep`); Rust copies a glyph out with
/// `grid::schar_copy` or `grid::schar_text`.
///
/// # Safety
/// `buf_out` must have room for [`crate::grid::MAX_SCHAR_SIZE`] bytes and
/// the terminator -- the room a glyph a release build interns can need.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn schar_get(buf_out: *mut c_char, sc: ScreenChar) -> size_t {
    let mut buf: SCharBuf = [0; SCHAR_BUF_SIZE];
    let len = schar_copy(&mut buf, sc);
    // SAFETY: the caller's room, which holds any glyph and its terminator.
    unsafe { core::ptr::copy_nonoverlapping(buf.as_ptr().cast::<c_char>(), buf_out, len + 1) };
    len
}

/// Sequence length per lead byte, answering **1** for a byte that introduces
/// nothing — so a walk over invalid text always makes progress
/// (`mbyte::utf8len_tab` re-exports it).
///
/// Exported as data: `test/unit/fixtures/shim.h` indexes it from C, so it
/// stays a 256-byte array under this symbol.
#[unsafe(no_mangle)]
pub static utf8len_tab: [u8; 256] = lead_byte_lengths(1);

/// The bytes of the C string at `p` that one character can occupy: up to its
/// NUL, and never more than the longest sequence.
///
/// # Safety
/// `p` points into a NUL-terminated string.
unsafe fn character_bytes<'a>(p: *const c_char) -> &'a [u8] {
    let p = p.cast::<u8>();
    let mut len = 0;
    // SAFETY: the caller's string, read up to its NUL at most.
    while len < MB_MAXCHAR && unsafe { *p.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: the `len` bytes just read.
    unsafe { slice::from_raw_parts(p, len) }
}

/// The codepoint at `p`, or the first byte's own value if there is no
/// complete sequence there; [`char_at`] over a C string.
///
/// `test/unit/vterm_spec.lua` decodes with it.
///
/// # Safety
/// `p` points at a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2char(p: *const c_char) -> c_int {
    // SAFETY: the caller's string. A sequence its NUL cuts short is not one,
    // which is what `char_at` answers for one the slice's end cuts short.
    char_at(unsafe { character_bytes(p) })
}

/// How many bytes the character at `p` occupies, or 0 at a NUL;
/// [`char_len`] over a C string.
///
/// # Safety
/// `p` points at a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2len(p: *const c_char) -> c_int {
    // SAFETY: as for `utf_ptr2char`. A sequence is at most six bytes.
    let len = char_len(unsafe { character_bytes(p) });
    c_int::try_from(len).expect("a sequence is at most six bytes")
}

/// Decode the multibyte sequence at `p`, which is `len` bytes long; see
/// [`crate::mbyte::sequence_value`]. `test/unit/fixtures/shim.h` calls it
/// for every non-ASCII lead byte.
///
/// # Safety
/// `p` points at `max(2, len)` readable bytes -- the second byte is read
/// unconditionally, which is sound for every sequence this is called for
/// because an incomplete one cannot be the last byte of a NUL-terminated
/// string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2char_info_impl(p: *const uint8_t, len: uintptr_t) -> int32_t {
    // SAFETY: the caller's bytes.
    let bytes = unsafe { slice::from_raw_parts(p, len.max(2)) };
    sequence_value(bytes, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two exported string readers, at every offset of strings that end
    /// in the middle of a sequence, a cluster or an emoji, each in an
    /// allocation that ends exactly at its NUL: under Miri, a reader that
    /// looked one byte past the terminator would be an out-of-bounds read.
    #[test]
    fn no_export_reads_past_the_nul() {
        let texts: [&[u8]; 14] = [
            b"",
            b"a",
            b"\xc3",
            b"\xe2\x82",
            b"\xf0\x9f\x98",
            b"\xfc\x84\x80\x80\x80",
            b"a\xcc",
            b"\xd8\x80\xe2",
            b"\xd8\x80\xe2\x82",
            b"\xe2\x9d\xa4\xef\xb8",
            b"\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f",
            b"\xd9\x84\xd8",
            b"\x80\x80\x80",
            b"x\xcc\x80\xcc\x81\xcc",
        ];
        for text in texts {
            // An allocation of exactly the string and its NUL.
            let buf: Box<[u8]> = text.iter().copied().chain(core::iter::once(0)).collect();
            for at in 0..=text.len() {
                let q = buf[at..].as_ptr().cast::<c_char>();
                // SAFETY: `q` is inside the NUL-terminated `buf`.
                let (got, _) = unsafe { (utf_ptr2char(q), utf_ptr2len(q)) };
                assert_eq!(got, char_at(&text[at..]), "{text:?} at {at}");
            }
        }
    }
}
