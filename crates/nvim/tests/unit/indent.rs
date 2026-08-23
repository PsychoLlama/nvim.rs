//! How wide an indent is, and what 'softtabstop' resolves to.
//!
//! A port of `test/unit/indent_spec.lua`. The arithmetic underneath
//! `indent_size_ts` has its own tests in `indent/mod.rs`; what only an
//! outside caller can reach is the pair of entry points themselves — the
//! 'vartabstop' array's *encoding* (a count in element 0, the widths after
//! it) and `get_sts_value`'s reading of the current buffer.
//!
//! `get_sts_value` is the reason this file needs the editor lock: it answers
//! from `curbuf`, and `cargo test` runs cases on threads of one process
//! where the LuaJIT harness forked a child per case and could scribble on
//! the real buffer's options freely.

#![cfg(not(miri))]

use std::ffi::c_int;

use neovim::indent::{get_sts_value, indent_size_ts};
use neovim::main::curbuf;
use neovim::types::{OptInt, buf_T, colnr_T};

use crate::support::{Editor, Sandbox, cstr};

/// Run `f` with `curbuf` pointing at a buffer of the case's own, so that
/// writing 'softtabstop' and friends cannot outlive the case.
///
/// The spec wrote them straight onto the editor's own buffer, which a forked
/// child could get away with. Zeroed is the right starting point: every
/// field `get_sts_value` reads is set below, and `b_p_vts_array` being null
/// is what "no 'vartabstop'" means.
fn with_buffer(f: impl FnOnce(&mut buf_T)) {
    let _sandbox = Sandbox::globals();
    // Boxed rather than a local: `curbuf` is a raw pointer the crate reads
    // through, and a heap allocation has an address that is nobody else's.
    //
    // SAFETY: every field of a `buf_T` is valid all-zero -- null pointers,
    // false flags and zeroed counters, which is what `xcalloc` hands
    // `buflist_new` for a real one.
    let mut buf: Box<buf_T> = Box::new(unsafe { std::mem::zeroed() });
    let saved = curbuf.get();
    curbuf.set(&raw mut *buf);
    f(&mut buf);
    curbuf.set(saved);
}

/// A non-negative 'softtabstop' is the answer, zero included.
#[test]
fn a_non_negative_softtabstop_is_its_own_value() {
    with_buffer(|buf| {
        buf.b_p_sts = 5;
        // SAFETY: `curbuf` is this case's buffer, under the editor lock.
        assert_eq!(unsafe { get_sts_value() }, 5);

        buf.b_p_sts = 0;
        // SAFETY: as above.
        assert_eq!(unsafe { get_sts_value() }, 0);
    });
}

/// A negative 'softtabstop' means "the effective shiftwidth", which is
/// 'shiftwidth' unless that is zero, in which case it is 'tabstop'.
#[test]
fn a_negative_softtabstop_is_the_effective_shiftwidth() {
    with_buffer(|buf| {
        buf.b_p_sts = -2;
        buf.b_p_sw = 2;
        buf.b_p_ts = 5;
        // SAFETY: `curbuf` is this case's buffer, under the editor lock.
        assert_eq!(unsafe { get_sts_value() }, 2, "'shiftwidth'");

        buf.b_p_sw = 0;
        // SAFETY: as above.
        assert_eq!(unsafe { get_sts_value() }, 5, "'tabstop' stands in");
    });
}

/// `indent_size_ts` over a NUL-terminated line, with the 'vartabstop' array
/// spelled the way the option code spells it: `vts[0]` is how many widths
/// follow, and a null array or a count of zero means the uniform `ts`.
///
/// The [`Editor`] token is not decoration: `indent_size_ts` opens with a
/// `debug_assert!` that a space is one cell wide, which reads the character
/// table the editor's startup fills in.
fn indent_size(_editor: &Editor, line: &str, ts: OptInt, vts: Option<&mut [colnr_T]>) -> c_int {
    let line = cstr(line);
    let vts = vts.map_or(std::ptr::null_mut(), <[colnr_T]>::as_mut_ptr);
    // SAFETY: `line` is this frame's and NUL-terminated; `vts` is null or a
    // slice whose first element is the count of the ones after it.
    unsafe { indent_size_ts(line.as_ptr(), ts, vts) }
}

#[test]
fn spaces_count_one_column_each() {
    let editor = Sandbox::globals();
    let line = format!("{}a ", " ".repeat(7));
    assert_eq!(indent_size(editor.editor(), &line, 100, None), 7);
}

#[test]
fn tabs_advance_to_the_next_uniform_stop() {
    let editor = Sandbox::globals();
    assert_eq!(
        indent_size(editor.editor(), "   \t  \t \t\t   a ", 4, None),
        19
    );
}

/// An array whose count is zero is not a 'vartabstop': the uniform `ts`
/// applies, exactly as for a null array. `ffi.new('int[1]')` is how the spec
/// spelled it, and the value it leaves in element 0 is the count.
#[test]
fn a_vartabstop_array_with_no_entries_falls_back_to_tabstop() {
    let editor = Sandbox::globals();
    let line = "   \t  \t \t\t       a ";
    let mut vts: [colnr_T; 1] = [0];
    assert_eq!(indent_size(editor.editor(), line, 4, Some(&mut vts)), 23);
    assert_eq!(indent_size(editor.editor(), line, 4, None), 23);
}

/// Two stops of 7 and 2, with the last repeating — and the count in front of
/// them, which is the encoding this entry point exists to decode.
#[test]
fn a_vartabstop_array_walks_the_widths_after_its_count() {
    let editor = Sandbox::globals();
    let line = "      \t  \t \t\t   a ";
    let mut vts: [colnr_T; 3] = [2, 7, 2];
    assert_eq!(indent_size(editor.editor(), line, 4, Some(&mut vts)), 18);
    // The same line under the uniform 'tabstop' is a different answer, so
    // the case really is reading the array.
    assert_ne!(indent_size(editor.editor(), line, 4, None), 18);
}
