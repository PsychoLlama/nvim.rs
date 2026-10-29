//! The names the in-repo test harnesses resolve in the binary, and nothing
//! else.
//!
//! Each item here is a `metrics/abi-ledger.jsonl` row of the `test` class
//! (see `docs/abi-ledger.md` for who reaches it and what would retire it).
//! `#[unsafe(no_mangle)]` is itself an unsafe attribute, which a file that
//! forbids unsafe code refuses, so the exports live in this one file and the
//! modules whose behaviour they reach -- `autocmd/`, `drawscreen/`, `grid/`,
//! `ui/` -- can forbid it. The functions are one-line fronts over the modules'
//! own; `display_tick` is a data symbol, so the cell itself lives here and
//! `drawscreen::state` re-exports it.

#![deny(unsafe_op_in_unsafe_fn)]
// The exports here are metrics/abi-ledger.jsonl rows (`block_autocmds`,
// `unblock_autocmds`, `display_tick`, `schar_get`, `ui_call_chdir`,
// `ui_call_set_title`), and
// `#[unsafe(no_mangle)]` is itself an unsafe attribute.
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
use crate::types::String_0;
use crate::types::{DispTick, ScreenChar, size_t};
use core::ffi::c_char;
use core::slice;

/// The redraw pass counter a decoration provider compares against to know it
/// has already been asked about this pass (`drawscreen::state` re-exports
/// it). Exported as data: `test/functional/lua/ffi_spec.lua` declares it
/// with `ffi.cdef` and reads it.
#[allow(non_upper_case_globals, reason = "the name the harness declares")]
#[unsafe(no_mangle)]
pub static display_tick: GlobalCell<DispTick> = GlobalCell::new(0);

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
pub struct HarnessString {
    data: *const c_char,
    size: size_t,
}

impl HarnessString {
    /// A copy of the bytes, as an API string.
    fn to_api(&self) -> String_0 {
        if self.data.is_null() {
            return String_0::NULL;
        }
        // SAFETY: only the spec builds one, over `size` bytes it owns for
        // the length of the call.
        String_0::from_bytes(unsafe { slice::from_raw_parts(self.data.cast::<u8>(), self.size) })
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
