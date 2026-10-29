//! The node userdatum and its thirty-odd accessors.
//!
//! A node is not owned: it is a `TSNode` value whose userdatum keeps the
//! tree it points into alive through its environment table (the tree at
//! index 1). [`NODE_METHODS`] is the method table; nearly every method is
//! one tree-sitter call, and [`push_node`] and [`check_node`] are the two
//! ends of the Lua boundary they all go through.

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

use core::ffi::{CStr, c_int, c_void};

use super::*;
use crate::lua::stack::{Kind, Raise};
use crate::lua_functions;
use crate::memory::xfree;

/// A node, as its userdatum holds it.
#[derive(Clone, Copy)]
pub(crate) struct Node(pub TSNode);

impl Default for Node {
    /// The null node: what nothing points at.
    fn default() -> Node {
        Node(TSNode {
            context: [0; 4],
            id: core::ptr::null(),
            tree: core::ptr::null(),
        })
    }
}

impl Userdata for Node {
    const METATABLE: &'static CStr = c"treesitter_node";
}

lua_functions! {
    /// The methods of a node.
    pub(crate) enum NodeMethods in NODE_METHODS {
        0 => c"__tostring": node_tostring,
        1 => c"__eq": node_equal,
        2 => c"__len": node_child_count,
        3 => c"id": node_id,
        4 => c"range": node_range,
        5 => c"start": node_start,
        6 => c"end_": node_end,
        7 => c"type": node_type,
        8 => c"symbol": node_symbol,
        9 => c"field": node_field,
        10 => c"named": node_named,
        11 => c"missing": node_missing,
        12 => c"extra": node_extra,
        13 => c"has_changes": node_has_changes,
        14 => c"has_error": node_has_error,
        15 => c"sexpr": node_sexpr,
        16 => c"child_count": node_child_count,
        17 => c"named_child_count": node_named_child_count,
        18 => c"child": node_child,
        19 => c"named_child": node_named_child,
        20 => c"descendant_for_range": node_descendant_for_range,
        21 => c"named_descendant_for_range": node_named_descendant_for_range,
        22 => c"parent": node_parent,
        23 => c"__has_ancestor": node_has_ancestor,
        24 => c"child_with_descendant": node_child_with_descendant,
        25 => c"iter_children": node_iter_children,
        26 => c"next_sibling": node_next_sibling,
        27 => c"prev_sibling": node_prev_sibling,
        28 => c"next_named_sibling": node_next_named_sibling,
        29 => c"prev_named_sibling": node_prev_named_sibling,
        30 => c"named_children": node_named_children,
        31 => c"root": node_root,
        32 => c"tree": node_tree,
        33 => c"byte_length": node_byte_length,
        34 => c"equal": node_equal,
    }
}

lua_functions! {
    /// The iterator `node:iter_children()` answers, a closure over the next
    /// child's index (upvalue 1) and the node (upvalue 2).
    enum NodeIterator in NODE_ITERATOR {
        0 => c"next_child": node_next_child,
    }
}

/// Push `node` as a userdatum sharing the environment of the userdatum at
/// `owner` -- an absolute slot or an upvalue -- so it keeps the same tree
/// alive; nil for the null node.
pub(crate) fn push_node(stack: &mut Stack, node: TSNode, owner: At) {
    debug_assert!(
        owner.index() > 0 || owner.index() < -20,
        "the owner's slot does not move as the node is pushed"
    );
    // SAFETY: a node value, which tree-sitter reads without dereferencing
    // anything for the null test.
    if unsafe { ts_node_is_null(node) } {
        stack.push_nil();
        return;
    }
    stack.new_userdata(Node(node));
    stack.push_environment(owner);
    stack.set_environment(At::from_top(2));
}

/// `luaL_checkudata` for a node: the node at `at`.
pub(crate) fn check_node(stack: &mut Stack, at: At) -> Result<TSNode, Raise> {
    Ok(stack.check_userdata::<Node>(at)?.0)
}

/// Push each of `values` as an integer; answers how many.
fn push_integers(stack: &mut Stack, values: &[u32]) -> c_int {
    for &value in values {
        stack.push_integer(integer(value));
    }
    c_int::try_from(values.len()).expect("a handful of results")
}

/// Push the node `step` answers for the node at slot 1, in its tree.
fn push_related(
    stack: &mut Stack,
    step: unsafe extern "C" fn(TSNode) -> TSNode,
) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: a node of a tree its userdatum keeps alive.
    let related = unsafe { step(node) };
    push_node(stack, related, At::arg(1));
    Ok(1)
}

/// Push what `test` answers for the node at slot 1.
fn push_test(
    stack: &mut Stack,
    test: unsafe extern "C" fn(TSNode) -> bool,
) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as in `push_related`.
    let answer = unsafe { test(node) };
    stack.push_boolean(answer);
    Ok(1)
}

fn node_tostring(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: a node of a live tree; its type name is the language's.
    let kind = unsafe { c_bytes(ts_node_type(node)) };
    let mut text = Vec::with_capacity(kind.len() + 7);
    text.extend_from_slice(b"<node ");
    text.extend_from_slice(kind);
    text.push(b'>');
    stack.push_bytes(&text);
    Ok(1)
}

fn node_equal(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let other = check_node(stack, At::arg(2))?;
    // SAFETY: two node values.
    let equal = unsafe { ts_node_eq(node, other) };
    stack.push_boolean(equal);
    Ok(1)
}

/// The node's id, as the bytes of its address.
fn node_id(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    stack.push_bytes(&node.id.addr().to_ne_bytes());
    Ok(1)
}

fn node_range(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let include_bytes = stack.top().index() >= 2 && stack.to_boolean(At::arg(2));
    // SAFETY: a node of a tree its userdatum keeps alive, throughout.
    let (start, end) = unsafe { (ts_node_start_point(node), ts_node_end_point(node)) };
    if include_bytes {
        // SAFETY: as above.
        let (start_byte, end_byte) = unsafe { (ts_node_start_byte(node), ts_node_end_byte(node)) };
        return Ok(push_integers(
            stack,
            &[
                start.row,
                start.column,
                start_byte,
                end.row,
                end.column,
                end_byte,
            ],
        ));
    }
    Ok(push_integers(
        stack,
        &[start.row, start.column, end.row, end.column],
    ))
}

fn node_start(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: a node of a tree its userdatum keeps alive.
    let (start, byte) = unsafe { (ts_node_start_point(node), ts_node_start_byte(node)) };
    Ok(push_integers(stack, &[start.row, start.column, byte]))
}

fn node_end(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above.
    let (end, byte) = unsafe { (ts_node_end_point(node), ts_node_end_byte(node)) };
    Ok(push_integers(stack, &[end.row, end.column, byte]))
}

fn node_child_count(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above.
    let count = unsafe { ts_node_child_count(node) };
    Ok(push_integers(stack, &[count]))
}

fn node_named_child_count(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above.
    let count = unsafe { ts_node_named_child_count(node) };
    Ok(push_integers(stack, &[count]))
}

fn node_type(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above; the type name is the language's.
    unsafe { push_c_string(stack, ts_node_type(node)) };
    Ok(1)
}

fn node_symbol(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above.
    let symbol = unsafe { ts_node_symbol(node) };
    stack.push_integer(integer(u32::from(symbol)));
    Ok(1)
}

/// The children whose field is the name in slot 2.
fn node_field(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above.
    let count = unsafe { ts_node_child_count(node) };
    // `strequal` on the C string `luaL_checklstring` answers.
    let wanted = until_nul(stack.check_bytes(At::arg(2))?).to_vec();
    stack.new_table(0, 0);
    let mut index = 0;
    for child_index in 0..count {
        // SAFETY: as above; a field name is the language's, or null.
        let field = unsafe { ts_node_field_name_for_child(node, child_index) };
        // SAFETY: as above.
        if !field.is_null() && unsafe { c_bytes(field) } == wanted.as_slice() {
            // SAFETY: as above.
            let child = unsafe { ts_node_child(node, child_index) };
            push_node(stack, child, At::arg(1));
            index += 1;
            stack.raw_set_index(At::from_top(2), index);
        }
    }
    Ok(1)
}

fn node_named(stack: &mut Stack) -> Result<c_int, Raise> {
    push_test(stack, ts_node_is_named)
}

fn node_sexpr(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above; the string is allocated through the editor's
    // allocator (`ts_set_allocator`) and ours to free, after the copy.
    unsafe {
        let text = ts_node_string(node);
        push_c_string(stack, text);
        xfree(text.cast::<c_void>());
    }
    Ok(1)
}

fn node_missing(stack: &mut Stack) -> Result<c_int, Raise> {
    push_test(stack, ts_node_is_missing)
}

fn node_extra(stack: &mut Stack) -> Result<c_int, Raise> {
    push_test(stack, ts_node_is_extra)
}

fn node_has_changes(stack: &mut Stack) -> Result<c_int, Raise> {
    push_test(stack, ts_node_has_changes)
}

fn node_has_error(stack: &mut Stack) -> Result<c_int, Raise> {
    push_test(stack, ts_node_has_error)
}

fn node_child(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let index = wrap_u32(stack.to_integer(At::arg(2)));
    // SAFETY: as above.
    let child = unsafe { ts_node_child(node, index) };
    push_node(stack, child, At::arg(1));
    Ok(1)
}

fn node_named_child(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let index = wrap_u32(stack.to_integer(At::arg(2)));
    // SAFETY: as above.
    let child = unsafe { ts_node_named_child(node, index) };
    push_node(stack, child, At::arg(1));
    Ok(1)
}

/// The two points in slots 2..=5, as `lua_tointeger` reads them.
fn point_range(stack: &Stack) -> (TSPoint, TSPoint) {
    let point = |row: c_int| TSPoint {
        row: wrap_u32(stack.to_integer(At::arg(row))),
        column: wrap_u32(stack.to_integer(At::arg(row + 1))),
    };
    (point(2), point(4))
}

fn node_descendant_for_range(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let (start, end) = point_range(stack);
    // SAFETY: as above.
    let child = unsafe { ts_node_descendant_for_point_range(node, start, end) };
    push_node(stack, child, At::arg(1));
    Ok(1)
}

fn node_named_descendant_for_range(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let (start, end) = point_range(stack);
    // SAFETY: as above.
    let child = unsafe { ts_node_named_descendant_for_point_range(node, start, end) };
    push_node(stack, child, At::arg(1));
    Ok(1)
}

/// One step of `iter_children`: the next child and its field name.
fn node_next_child(stack: &mut Stack) -> Result<c_int, Raise> {
    let index = wrap_u32(stack.to_integer(At::upvalue(1)));
    let source = check_node(stack, At::upvalue(2))?;
    // SAFETY: a node of a tree its userdatum (the upvalue) keeps alive.
    if index >= unsafe { ts_node_child_count(source) } {
        return Ok(0);
    }
    // SAFETY: as above.
    let child = unsafe { ts_node_child(source, index) };
    push_node(stack, child, At::upvalue(2));
    // SAFETY: as above; a field name is the language's, or null.
    unsafe { push_c_string(stack, ts_node_field_name_for_child(source, index)) };
    stack.push_integer(integer(index) + 1);
    stack.replace(At::upvalue(1));
    Ok(2)
}

fn node_iter_children(stack: &mut Stack) -> Result<c_int, Raise> {
    check_node(stack, At::arg(1))?;
    stack.push_integer(0);
    stack.push_value(At::arg(1));
    stack.push_closure(NODE_ITERATOR[0].entry(), 2);
    Ok(1)
}

fn node_parent(stack: &mut Stack) -> Result<c_int, Raise> {
    push_related(stack, ts_node_parent)
}

/// Whether an ancestor of the node in slot 1 has one of the types listed
/// from index 3 of the table in slot 2 (a `has-ancestor?` predicate).
fn node_has_ancestor(stack: &mut Stack) -> Result<c_int, Raise> {
    let descendant = check_node(stack, At::arg(1))?;
    if stack.kind(At::arg(2)) != Kind::Table {
        stack.push_boolean(false);
        return Ok(1);
    }
    let predicate_len = c_int::try_from(stack.raw_len(At::arg(2))).unwrap_or(c_int::MAX);
    // SAFETY: a node of a tree its userdatum keeps alive, and the nodes
    // walked down to it in the same tree.
    let mut node = unsafe { ts_tree_root_node(descendant.tree) };
    // SAFETY: as above.
    while node.id != descendant.id && !unsafe { ts_node_is_null(node) } {
        // SAFETY: as above; the type name is the language's.
        let kind = unsafe { c_bytes(ts_node_type(node)) };
        for index in 3..=predicate_len {
            stack.raw_get_index(At::arg(2), index);
            let found = stack.kind(At::TOP) == Kind::String && stack.string(At::TOP) == Some(kind);
            stack.pop(1);
            if found {
                stack.push_boolean(true);
                return Ok(1);
            }
        }
        // SAFETY: as above.
        node = unsafe { ts_node_child_with_descendant(node, descendant) };
    }
    stack.push_boolean(false);
    Ok(1)
}

fn node_child_with_descendant(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    let descendant = check_node(stack, At::arg(2))?;
    // SAFETY: as above.
    let child = unsafe { ts_node_child_with_descendant(node, descendant) };
    push_node(stack, child, At::arg(1));
    Ok(1)
}

fn node_next_sibling(stack: &mut Stack) -> Result<c_int, Raise> {
    push_related(stack, ts_node_next_sibling)
}

fn node_prev_sibling(stack: &mut Stack) -> Result<c_int, Raise> {
    push_related(stack, ts_node_prev_sibling)
}

fn node_next_named_sibling(stack: &mut Stack) -> Result<c_int, Raise> {
    push_related(stack, ts_node_next_named_sibling)
}

fn node_prev_named_sibling(stack: &mut Stack) -> Result<c_int, Raise> {
    push_related(stack, ts_node_prev_named_sibling)
}

fn node_named_children(stack: &mut Stack) -> Result<c_int, Raise> {
    let source = check_node(stack, At::arg(1))?;
    stack.new_table(0, 0);
    let mut index = 0;
    // SAFETY: as above.
    let count = unsafe { ts_node_child_count(source) };
    for child_index in 0..count {
        // SAFETY: as above.
        let child = unsafe { ts_node_child(source, child_index) };
        // SAFETY: as above.
        if unsafe { ts_node_is_named(child) } {
            push_node(stack, child, At::arg(1));
            index += 1;
            stack.raw_set_index(At::from_top(2), index);
        }
    }
    Ok(1)
}

fn node_root(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: the node's tree, which its userdatum keeps alive.
    let root = unsafe { ts_tree_root_node(node.tree) };
    push_node(stack, root, At::arg(1));
    Ok(1)
}

/// The tree the node belongs to: index 1 of its environment.
fn node_tree(stack: &mut Stack) -> Result<c_int, Raise> {
    check_node(stack, At::arg(1))?;
    stack.push_environment(At::arg(1));
    // Upstream reads slot 2, which is the environment only when the call
    // had no extra arguments; the top is the environment always.
    stack.raw_get_index(At::TOP, 1);
    Ok(1)
}

fn node_byte_length(stack: &mut Stack) -> Result<c_int, Raise> {
    let node = check_node(stack, At::arg(1))?;
    // SAFETY: as above.
    let (start, end) = unsafe { (ts_node_start_byte(node), ts_node_end_byte(node)) };
    stack.push_integer(integer(end.wrapping_sub(start)));
    Ok(1)
}
