//! The tree userdatum: copying it, editing it, and its root.
//!
//! A [`Tree`] owns one `TSTree *`; tree-sitter shares the nodes between
//! copies, so `copy` and `edit` are cheap, and every userdatum deletes its
//! own tree when collected. `edit` answers an edited copy -- it tells
//! tree-sitter which byte span changed so the next parse can reuse the
//! rest.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};
use core::ptr::NonNull;

use super::ranges::push_owned_ranges;
use super::*;
use crate::lua::stack::Raise;
use crate::lua_functions;

/// A tree, as its userdatum owns it. Empty once collected.
#[derive(Default)]
pub(crate) struct Tree(Option<NonNull<TSTree>>);

impl Drop for Tree {
    fn drop(&mut self) {
        if let Some(tree) = self.0.take() {
            // SAFETY: the tree this userdatum owns, deleted once.
            unsafe { ts_tree_delete(tree.as_ptr()) };
        }
    }
}

impl Userdata for Tree {
    const METATABLE: &'static CStr = c"treesitter_tree";
}

lua_functions! {
    /// The methods of a tree.
    pub(crate) enum TreeMethods in TREE_METHODS {
        0 => c"__tostring": tree_tostring,
        1 => c"root": tree_root,
        2 => c"edit": tree_edit,
        3 => c"included_ranges": tree_included_ranges,
        4 => c"copy": tree_copy,
    }
}

/// Push `tree` as a userdatum that owns it; nil for no tree.
pub(crate) fn push_tree(stack: &mut Stack, tree: *mut TSTree) {
    match NonNull::new(tree) {
        Some(tree) => stack.new_userdata(Tree(Some(tree))),
        None => stack.push_nil(),
    }
}

/// `luaL_checkudata` for a tree: its `TSTree *` (null once collected).
pub(crate) fn check_tree(stack: &mut Stack, at: At) -> Result<*const TSTree, Raise> {
    let tree = stack.check_userdata::<Tree>(at)?;
    Ok(tree
        .0
        .map_or(core::ptr::null(), |tree| tree.as_ptr().cast_const()))
}

fn tree_copy(stack: &mut Stack) -> Result<c_int, Raise> {
    let tree = check_tree(stack, At::arg(1))?;
    // SAFETY: the tree the userdatum in slot 1 owns.
    let copy = unsafe { ts_tree_copy(tree) };
    push_tree(stack, copy);
    Ok(1)
}

/// `tree:edit(start_byte, old_end_byte, new_end_byte, start_row,
/// start_col, old_end_row, old_end_col, new_end_row, new_end_col)`: an
/// edited copy.
fn tree_edit(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() < 10 {
        stack.push_bytes(b"not enough args to tree:edit()");
        return Err(Raise::top_value());
    }
    let tree = check_tree(stack, At::arg(1))?;
    let mut fields = [0; 9];
    for (slot, field) in (2..).zip(&mut fields) {
        // `(uint32_t)(int)luaL_checkinteger(..)`: the low 32 bits.
        *field = wrap_u32(stack.check_integer(At::arg(slot))?);
    }
    let [start_byte, old_end_byte, new_end_byte, rows @ ..] = fields;
    let point = |at: usize| TSPoint {
        row: rows[at],
        column: rows[at + 1],
    };
    let edit = TSInputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_point: point(0),
        old_end_point: point(2),
        new_end_point: point(4),
    };
    // SAFETY: the tree the userdatum in slot 1 owns, copied, and the copy
    // edited before anything else sees it.
    let edited = unsafe {
        let copy = ts_tree_copy(tree);
        ts_tree_edit(copy, &raw const edit);
        copy
    };
    push_tree(stack, edited);
    Ok(1)
}

fn tree_included_ranges(stack: &mut Stack) -> Result<c_int, Raise> {
    let tree = check_tree(stack, At::arg(1))?;
    let include_bytes = stack.top().index() >= 2 && stack.to_boolean(At::arg(2));
    let mut length: u32 = 0;
    // SAFETY: as above; the array is allocated through the editor's
    // allocator and handed over.
    let ranges = unsafe { ts_tree_included_ranges(tree, &raw mut length) };
    // SAFETY: `length` ranges tree-sitter just allocated for us.
    unsafe { push_owned_ranges(stack, ranges, length, include_bytes) };
    Ok(1)
}

fn tree_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.push_bytes(b"<tree>");
    Ok(1)
}

/// The root node, whose environment holds the tree: a userdatum even for
/// the null node, as upstream pushes it.
fn tree_root(stack: &mut Stack) -> Result<c_int, Raise> {
    let tree = check_tree(stack, At::arg(1))?;
    // SAFETY: as above.
    let root = unsafe { ts_tree_root_node(tree) };
    stack.new_userdata(Node(root));
    stack.new_table(1, 0);
    stack.push_value(At::arg(1));
    stack.raw_set_index(At::from_top(2), 1);
    stack.set_environment(At::from_top(2));
    Ok(1)
}
