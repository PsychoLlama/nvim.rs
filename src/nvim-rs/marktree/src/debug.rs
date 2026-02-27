//! Debug/inspection functions for marktree.
//!
//! This module provides tree serialization for debugging and the
//! `nvim__inspect_tree` API endpoint.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};
use std::fmt::Write as FmtWrite;

use crate::{mt_end, mt_paired, mt_start, unrelative, MTNodeHandle, MTPos, MarkTreeHandle};

// ============================================================================
// C memory allocator FFI
// ============================================================================

extern "C" {
    /// Allocate `size + 1` bytes, zeroing the last byte.
    fn xmallocz(size: usize) -> *mut c_void;
}

// ============================================================================
// NvimApiString - matches C's `String` struct `{char *data, size_t size}`
// ============================================================================

/// C-compatible string matching the Neovim API `String` type.
///
/// Must match the C layout `typedef struct { char *data; size_t size; } String;`.
#[repr(C)]
pub struct NvimApiString {
    pub data: *mut c_char,
    pub size: usize,
}

impl NvimApiString {
    /// Create an empty (null) string.
    fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            size: 0,
        }
    }

    /// Convert a Rust string slice into an `NvimApiString` allocated with xmallocz.
    ///
    /// The caller (C side) takes ownership and must free with `xfree`.
    fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();
        // SAFETY: xmallocz allocates len+1 bytes, zeroing the last one
        let raw = unsafe { xmallocz(len) };
        if raw.is_null() {
            return Self::empty();
        }
        // SAFETY: raw is valid for len bytes; bytes is valid for len bytes
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), raw.cast::<u8>(), len);
        }
        Self {
            data: raw.cast::<c_char>(),
            size: len,
        }
    }
}

// ============================================================================
// C Node Accessor Functions
// ============================================================================

extern "C" {
    fn nvim_mtnode_get_n(x: MTNodeHandle) -> c_int;
    fn nvim_mtnode_get_level(x: MTNodeHandle) -> c_int;
    fn nvim_mtnode_get_key(x: MTNodeHandle, idx: c_int) -> crate::MTKey;
    fn nvim_mtnode_get_ptr(x: MTNodeHandle, idx: c_int) -> MTNodeHandle;
    fn nvim_mtnode_get_p_idx(x: MTNodeHandle) -> c_int;
    fn nvim_mtnode_get_intersect_size(x: MTNodeHandle) -> usize;
    fn nvim_mtnode_get_intersect_elem(x: MTNodeHandle, idx: usize) -> u64;
    fn nvim_marktree_get_root(b: MarkTreeHandle) -> MTNodeHandle;
}

// ============================================================================
// Debug ID helper
// ============================================================================

/// Extract 32-bit display ID from a 64-bit lookup ID.
#[inline]
fn mt_dbg_id(id: u64) -> u64 {
    (id >> 1) & 0xffff_ffff
}

// ============================================================================
// Text serialization
// ============================================================================

/// Recursive text serialization of a node into `output`.
fn inspect_node(output: &mut String, keys: bool, n: MTNodeHandle, off: MTPos) {
    // SAFETY: n is a valid non-null MTNode pointer obtained from C tree ops
    let n_keys = unsafe { nvim_mtnode_get_n(n) } as usize;
    let level = unsafe { nvim_mtnode_get_level(n) };
    let intersect_size = unsafe { nvim_mtnode_get_intersect_size(n) };

    output.push('[');

    if keys && intersect_size > 0 {
        for i in 0..intersect_size {
            output.push(if i == 0 { '{' } else { ';' });
            let elem = unsafe { nvim_mtnode_get_intersect_elem(n, i) };
            let _ = write!(output, "{}", mt_dbg_id(elem));
        }
        output.push_str("},");
    }

    if level != 0 {
        let child0 = unsafe { nvim_mtnode_get_ptr(n, 0) };
        inspect_node(output, keys, child0, off);
    }

    for i in 0..n_keys {
        let key = unsafe { nvim_mtnode_get_key(n, i as c_int) };
        let mut p = key.pos;
        unrelative(off, &mut p);
        let _ = write!(output, "{}/{}", p.row, p.col);

        if keys {
            output.push(':');
            if mt_start(&key) {
                output.push('<');
            }
            let _ = write!(output, "{}", key.id);
            if mt_end(&key) {
                output.push('>');
            }
        }

        if level != 0 {
            let child = unsafe { nvim_mtnode_get_ptr(n, (i + 1) as c_int) };
            inspect_node(output, keys, child, p);
        } else {
            output.push(',');
        }
    }

    output.push(']');
}

// ============================================================================
// DOT graph serialization
// ============================================================================

/// Recursive DOT graph serialization of a node into `output`.
fn inspect_dotfile_node(output: &mut String, n: MTNodeHandle, off: MTPos, parent: Option<&str>) {
    // SAFETY: n is a valid non-null MTNode pointer obtained from C tree ops
    let n_keys = unsafe { nvim_mtnode_get_n(n) } as usize;
    let level = unsafe { nvim_mtnode_get_level(n) };
    let p_idx = unsafe { nvim_mtnode_get_p_idx(n) };
    let intersect_size = unsafe { nvim_mtnode_get_intersect_size(n) };

    let namebuf: String = parent.map_or_else(
        || "MTNode".to_string(),
        |par| {
            // 'a' + level gives a letter for the level
            let level_char = (b'a' + (level as u8).min(25)) as char;
            format!("{par}_{level_char}{p_idx}")
        },
    );

    let _ = writeln!(output, "  {namebuf}[shape=plaintext, label=<");
    output.push_str("    <table border='0' cellborder='1' cellspacing='0'>\n");

    if intersect_size > 0 {
        output.push_str("    <tr><td>");
        for i in 0..intersect_size {
            if i > 0 {
                output.push_str(", ");
            }
            let elem = unsafe { nvim_mtnode_get_intersect_elem(n, i) };
            let _ = write!(output, "{}", mt_dbg_id(elem));
        }
        output.push_str("</td></tr>\n");
    }

    output.push_str("    <tr><td>");
    for i in 0..n_keys {
        let k = unsafe { nvim_mtnode_get_key(n, i as c_int) };
        if i > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{}", k.id);
        if mt_paired(&k) {
            output.push(if mt_end(&k) { 'e' } else { 's' });
        }
    }
    output.push_str("</td></tr>\n");
    output.push_str("    </table>\n");
    output.push_str(">];\n");

    if let Some(par) = parent {
        let _ = writeln!(output, "  {par} -> {namebuf}");
    }

    if level != 0 {
        let child0 = unsafe { nvim_mtnode_get_ptr(n, 0) };
        inspect_dotfile_node(output, child0, off, Some(&namebuf));
    }

    for i in 0..n_keys {
        let key = unsafe { nvim_mtnode_get_key(n, i as c_int) };
        let mut p = key.pos;
        unrelative(off, &mut p);
        if level != 0 {
            let child = unsafe { nvim_mtnode_get_ptr(n, (i + 1) as c_int) };
            inspect_dotfile_node(output, child, p, Some(&namebuf));
        }
    }
}

// ============================================================================
// FFI entry point
// ============================================================================

/// Build a text or DOT representation of the marktree.
///
/// # Safety
///
/// `b` must be a valid non-null `MarkTree*`.
/// The returned string is allocated with `xmallocz` and must be freed by the
/// caller via `xfree` (i.e. the Neovim API string free path).
#[no_mangle]
pub unsafe extern "C" fn rs_mt_inspect(b: MarkTreeHandle, keys: bool, dot: bool) -> NvimApiString {
    let root = unsafe { nvim_marktree_get_root(b) };
    if root.is_null() {
        return NvimApiString::empty();
    }

    let mut output = String::with_capacity(256);
    let off = MTPos::new(0, 0);

    if dot {
        output.push_str("digraph D {\n\n");
        inspect_dotfile_node(&mut output, root, off, None);
        output.push_str("\n}");
    } else {
        inspect_node(&mut output, keys, root, off);
    }

    NvimApiString::from_str(&output)
}
