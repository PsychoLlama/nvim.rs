#![deny(unsafe_op_in_unsafe_fn)]

//! Rendering a tree for a human: `:lua vim.api.nvim__buf_debug_extmarks()`.
//!
//! Two formats. The default is a nested parenthesised dump, one node per line,
//! with each node's intersection set in braces. With `dot` it emits a Graphviz
//! digraph instead, one HTML table per node, which is the only practical way to
//! see what a rebalancing did.
//!
//! The C built both in a growarray, formatting each number through a
//! function-local 1 KiB `snprintf` scratch buffer. Here the whole dump is one
//! `String` and the numbers go in with `write!`; only the hand-off to the API
//! layer, which frees the block itself, still needs the C allocator.

use core::fmt::Write as _;
use core::ptr;

use crate::marktree::key::{mt_end, mt_paired, mt_start, unrelative};
use crate::marktree::node::Node;
use crate::memory::xmemdupz;
use crate::types::{MTPos, MarkTree, String_0, uint64_t};

/// Nested to keep the name out of the flat cdef namespace `ffigen` builds,
/// the same reason node.rs nests its own sizes.
mod sizes {
    /// The most bytes a node's Graphviz identifier can occupy. The C formatted
    /// it with `snprintf` into a 64-byte buffer, and since each level appends
    /// to its parent's name a deep tree really does hit the limit — the
    /// truncation is part of the output, not an accident of the buffer.
    pub const DOT_NAME_MAX: usize = 63;
}
use sizes::DOT_NAME_MAX;

/// Dump the whole tree, as text or as a Graphviz digraph.
///
/// The result is an `xmalloc`'d block the caller owns.
///
/// # Safety
/// `b` must be a live tree.
pub(crate) unsafe fn mt_inspect(b: &mut MarkTree, keys: bool, dot: bool) -> String_0 {
    // SAFETY: `b` is a live tree per the caller, so its root is null or one of
    // its live nodes.
    let Some(root) = (unsafe { Node::from_ptr(b.root) }) else {
        // An empty tree renders as nothing at all: the C handed back its
        // untouched growarray, which is a null string.
        return String_0::from_raw_parts(ptr::null_mut(), 0);
    };
    let mut out = String::new();
    if dot {
        out.push_str("digraph D {\n\n");
        dotfile_node(&mut out, root, MTPos::default(), None);
        out.push_str("\n}");
    } else {
        inspect_node(&mut out, keys, root, MTPos::default());
    }
    // SAFETY: `out` names `len` initialised bytes, which is what `xmemdupz`
    // copies into the fresh block it answers.
    let data = unsafe { xmemdupz(out.as_ptr().cast(), out.len()) };
    String_0::from_raw_parts(data.cast(), out.len())
}

/// The id a paired mark is known by in the dump: the `(ns, id)` handle with
/// the end-half flag shifted off and the namespace masked away.
fn mt_dbg_id(id: uint64_t) -> uint64_t {
    id >> 1 & 0xffffffff
}

/// One node of the parenthesised dump, and then its children.
///
/// A node is `[`, its intersection set in braces where it has one, and then
/// its keys' absolute positions interleaved with the subtrees between them.
fn inspect_node(out: &mut String, keys: bool, n: Node, off: MTPos) {
    out.push('[');
    let set = n.intersection();
    if keys && !set.is_empty() {
        for (i, &id) in set.as_slice().iter().enumerate() {
            out.push(if i == 0 { '{' } else { ';' });
            let _ = write!(out, "{}", mt_dbg_id(id));
        }
        out.push_str("},");
    }
    if !n.is_leaf() {
        inspect_node(out, keys, n.child(0), off);
    }
    for i in 0..n.key_count() {
        let key = n.key(i);
        let mut p = key.pos;
        unrelative(off, &mut p);
        let _ = write!(out, "{}/{}", p.row, p.col);
        if keys {
            out.push(':');
            if mt_start(key) {
                out.push('<');
            }
            let _ = write!(out, "{}", key.id);
            if mt_end(key) {
                out.push('>');
            }
        }
        if !n.is_leaf() {
            inspect_node(out, keys, n.child(i + 1), p);
        } else {
            out.push(',');
        }
    }
    out.push(']');
}

/// One node as a Graphviz record — its intersection set on one row and its
/// keys on the next — plus an edge from its parent, and then its children.
fn dotfile_node(out: &mut String, n: Node, off: MTPos, parent: Option<&str>) {
    let name = dot_name(parent, n.level(), n.parent_index());
    let _ = writeln!(out, "  {name}[shape=plaintext, label=<");
    out.push_str("    <table border='0' cellborder='1' cellspacing='0'>\n");
    let set = n.intersection();
    if !set.is_empty() {
        out.push_str("    <tr><td>");
        for (i, &id) in set.as_slice().iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{}", mt_dbg_id(id));
        }
        out.push_str("</td></tr>\n");
    }
    out.push_str("    <tr><td>");
    for i in 0..n.key_count() {
        let k = n.key(i);
        if i > 0 {
            out.push_str(", ");
        }
        // The C printed this one with `%d`, so a namespace-less id wide enough
        // to set the top bit comes out negative. Kept, for identical dumps.
        let _ = write!(out, "{}", k.id as i32);
        if mt_paired(k) {
            out.push(if mt_end(k) { 'e' } else { 's' });
        }
    }
    out.push_str("</td></tr>\n");
    out.push_str("    </table>\n");
    out.push_str(">];\n");
    if let Some(parent) = parent {
        let _ = writeln!(out, "  {parent} -> {name}");
    }
    if !n.is_leaf() {
        dotfile_node(out, n.child(0), off, Some(&name));
        for i in 0..n.key_count() {
            let mut p = n.key(i).pos;
            unrelative(off, &mut p);
            dotfile_node(out, n.child(i + 1), p, Some(&name));
        }
    }
}

/// A node's Graphviz identifier: its parent's, then the node's level as a
/// letter and its index among its parent's children. The root is `MTNode`.
fn dot_name(parent: Option<&str>, level: usize, parent_index: usize) -> String {
    let Some(parent) = parent else {
        return String::from("MTNode");
    };
    let letter = char::from(b'a' + level as u8);
    let mut name = format!("{parent}_{letter}{parent_index}");
    // Every byte of a name is ASCII, so this cannot split a character.
    name.truncate(DOT_NAME_MAX);
    name
}
