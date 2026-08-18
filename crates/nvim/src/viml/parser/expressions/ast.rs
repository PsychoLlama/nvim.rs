//! The node tables, node allocation and teardown, and the shunting-yard step
//! that attaches a binary operator to the tree.
//!
//! # Node pointers
//!
//! A node is `xmalloc`ed by [`viml_pexpr_new_node`] and lives until
//! [`viml_pexpr_free_ast`] walks the finished tree, so a non-null node
//! pointer the parser holds is live and dereferenceable for the whole parse.
//! That is the whole of the obligation behind the accessors below, and it is
//! why they are safe functions.
//!
//! They stay *raw* projections. The AST stack remembers where the next value
//! goes by holding an interior pointer into a node — `&raw mut
//! (*node).children` — and a `&mut ExprASTNode` taken anywhere would retag
//! over those and leave the stack holding dead tags. `parser.rs`'s module doc
//! makes the same point about `ParserState`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::collections::HashSet;

use super::*;
use crate::types::{expr_ast_node_data, expr_ast_node_data_fig};
use crate::viml::parser::parser::reader_line;

/// A node's type tag.
///
/// Every accessor here is `inline(always)`: they exist to bound the unsafe
/// surface, not to add a layer, and an unoptimised build otherwise pays a
/// real call for each field read — which shows up as ~15% on a parse deep
/// enough for the stack-invariant check below to matter.
#[inline(always)]
pub(super) fn node_type(node: *mut ExprASTNode) -> ExprASTNodeType {
    // SAFETY: a parser-held node pointer is live; see the module doc. Every
    // accessor below carries the same obligation and does not repeat it.
    unsafe { (*node).type_0 }
}

#[inline(always)]
pub(super) fn set_node_type(node: *mut ExprASTNode, type_0: ExprASTNodeType) {
    unsafe { (*node).type_0 = type_0 }
}

/// Where in the input the node starts.
#[inline(always)]
pub(super) fn node_start(node: *mut ExprASTNode) -> ParserPosition {
    unsafe { (*node).start }
}

#[inline(always)]
pub(super) fn set_node_span(node: *mut ExprASTNode, start: ParserPosition, len: size_t) {
    unsafe {
        (*node).start = start;
        (*node).len = len;
    }
}

#[inline(always)]
pub(super) fn set_node_len(node: *mut ExprASTNode, len: size_t) {
    unsafe { (*node).len = len }
}

/// The node's first child, or null when it has none.
#[inline(always)]
pub(super) fn node_children(node: *mut ExprASTNode) -> *mut ExprASTNode {
    unsafe { (*node).children }
}

#[inline(always)]
pub(super) fn set_node_children(node: *mut ExprASTNode, children: *mut ExprASTNode) {
    unsafe { (*node).children = children }
}

/// The node's sibling, or null when it is the last one.
#[inline(always)]
pub(super) fn node_next(node: *mut ExprASTNode) -> *mut ExprASTNode {
    unsafe { (*node).next }
}

/// The slot the node's first child goes into — an AST stack item.
#[inline(always)]
pub(super) fn children_slot(node: *mut ExprASTNode) -> *mut *mut ExprASTNode {
    unsafe { &raw mut (*node).children }
}

/// The slot the node's sibling goes into — an AST stack item.
#[inline(always)]
pub(super) fn next_slot(node: *mut ExprASTNode) -> *mut *mut ExprASTNode {
    unsafe { &raw mut (*node).next }
}

/// Whatever an AST stack item currently points at; null while the parser is
/// still waiting for the value that goes there.
#[inline(always)]
pub(super) fn slot_node(slot: *mut *mut ExprASTNode) -> *mut ExprASTNode {
    unsafe { *slot }
}

#[inline(always)]
pub(super) fn set_slot_node(slot: *mut *mut ExprASTNode, node: *mut ExprASTNode) {
    unsafe { *slot = node }
}

/// Write the whole of a node's payload. The union is *constructed* in safe
/// code — only reading one back out has to name the right member, which is
/// what the three readers below are for.
#[inline(always)]
pub(super) fn set_node_data(node: *mut ExprASTNode, data: expr_ast_node_data) {
    unsafe { (*node).data = data }
}

/// A figure brace node's guesses at what it will turn out to be.
#[inline(always)]
pub(super) fn node_fig(node: *mut ExprASTNode) -> expr_ast_node_data_fig {
    unsafe { (*node).data.fig }
}

/// Whether a TernaryValue node has seen its `:` yet.
#[inline(always)]
pub(super) fn node_got_colon(node: *mut ExprASTNode) -> bool {
    unsafe { (*node).data.ter.got_colon }
}

/// The decoded bytes a string node owns.
#[inline(always)]
pub(super) fn node_str_value(node: *mut ExprASTNode) -> *mut c_char {
    unsafe { (*node).data.str.value }
}

/// The slot the root of the tree goes into — the bottom AST stack item.
#[inline(always)]
pub(super) fn ast_root_slot(ast: *mut ExprAST) -> *mut *mut ExprASTNode {
    unsafe { &raw mut (*ast).root }
}

/// Whether the parse has already reported an error. The first one wins, so
/// this is also what [`east_set_error`] tests.
#[inline(always)]
pub(super) fn ast_has_error(ast: *const ExprAST) -> bool {
    unsafe { !(*ast).err.msg.is_null() }
}

pub static eltkn_cmp_type_tab: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    c"Equal".as_ptr(),
    c"Matches".as_ptr(),
    c"Greater".as_ptr(),
    c"GreaterOrEqual".as_ptr(),
    c"Identical".as_ptr(),
]);
pub static expr_asgn_type_tab: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    c"Plain".as_ptr(),
    c"Add".as_ptr(),
    c"Subtract".as_ptr(),
    c"Concat".as_ptr(),
]);
pub static ccs_tab: GlobalCell<[*const ::core::ffi::c_char; 64]> = GlobalCell::new([
    c"UseOption".as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"MatchCase".as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"IgnoreCase".as_ptr(),
]);
pub static east_node_type_tab: GlobalCell<[*const ::core::ffi::c_char; 39]> = GlobalCell::new([
    c"Missing".as_ptr(),
    c"OpMissing".as_ptr(),
    c"Ternary".as_ptr(),
    c"TernaryValue".as_ptr(),
    c"Register".as_ptr(),
    c"Subscript".as_ptr(),
    c"ListLiteral".as_ptr(),
    c"UnaryPlus".as_ptr(),
    c"BinaryPlus".as_ptr(),
    c"Nested".as_ptr(),
    c"Call".as_ptr(),
    c"PlainIdentifier".as_ptr(),
    c"PlainKey".as_ptr(),
    c"ComplexIdentifier".as_ptr(),
    c"UnknownFigure".as_ptr(),
    c"Lambda".as_ptr(),
    c"DictLiteral".as_ptr(),
    c"CurlyBracesIdentifier".as_ptr(),
    c"Comma".as_ptr(),
    c"Colon".as_ptr(),
    c"Arrow".as_ptr(),
    c"Comparison".as_ptr(),
    c"Concat".as_ptr(),
    c"ConcatOrSubscript".as_ptr(),
    c"Integer".as_ptr(),
    c"Float".as_ptr(),
    c"SingleQuotedString".as_ptr(),
    c"DoubleQuotedString".as_ptr(),
    c"Or".as_ptr(),
    c"And".as_ptr(),
    c"UnaryMinus".as_ptr(),
    c"BinaryMinus".as_ptr(),
    c"Not".as_ptr(),
    c"Multiplication".as_ptr(),
    c"Division".as_ptr(),
    c"Mod".as_ptr(),
    c"Option".as_ptr(),
    c"Environment".as_ptr(),
    c"Assignment".as_ptr(),
]);
pub(super) static node_maxchildren: [uint8_t; 39] = [
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    1 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    2 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    2 as uint8_t,
];
/// The teardown walk's stack, plus the set of nodes it holds.
///
/// Upstream checks for a recursive AST by rescanning the whole stack once per
/// node — `assert(*kv_A(ast_stack, i) != *cur_node)` under `#ifndef NDEBUG` —
/// which is quadratic in the depth of the tree: an 8,000-deep parse spent
/// ~860 ms in that loop alone in a debug build, and every test suite here
/// runs a debug build. A slot's node only ever changes while the slot is on
/// top of the stack, so keeping a set as items are pushed and popped asks
/// exactly the same question in constant time.
///
/// The set is maintained by the `debug_assert!` itself, so it stays empty in
/// a release build, where the check is compiled out along with it.
#[derive(Default)]
struct TeardownStack {
    slots: Vec<*mut *mut ExprASTNode>,
    on_stack: HashSet<*mut ExprASTNode>,
}

impl TeardownStack {
    /// Descend into `slot`, which currently holds `node`.
    fn push(&mut self, slot: *mut *mut ExprASTNode, node: *mut ExprASTNode) {
        debug_assert!(
            self.on_stack.insert(node),
            "the AST is recursive: a node is reachable from itself"
        );
        self.slots.push(slot);
    }

    /// Leave the slot on top, which still holds `node`.
    fn pop(&mut self, node: *mut ExprASTNode) {
        debug_assert!(
            self.on_stack.remove(&node),
            "the stack lost track of a node"
        );
        self.slots.pop();
    }
}

/// The child-count invariants the C checked under `#ifndef NDEBUG`. Only the
/// last is a hard `assert!` here, as it was in the transpiled body.
fn assert_children_fit(node: *mut ExprASTNode) {
    let maxchildren = node_maxchildren[node_type(node) as usize];
    debug_assert!(maxchildren > 0, "maxchildren > 0");
    debug_assert!(maxchildren <= 2, "maxchildren <= 2");
    let second = node_next(node_children(node));
    assert!(
        if maxchildren == 1 {
            second.is_null()
        } else {
            second.is_null() || node_next(second).is_null()
        },
        "a node has no more children than its type allows"
    );
}

/// Free a finished AST and everything hanging off it.
///
/// # Safety
/// `ast` must be an AST [`viml_pexpr_parse`] built and nobody else has freed.
pub unsafe fn viml_pexpr_free_ast(mut ast: ExprAST) {
    let mut stack = TeardownStack::default();
    stack.push(&raw mut ast.root, ast.root);
    while let Some(&cur_slot) = stack.slots.last() {
        let cur_node = slot_node(cur_slot);
        if cur_node.is_null() {
            // Only the root slot can be empty: every other one was pushed
            // holding a node.
            debug_assert!(stack.slots.len() == 1, "kv_size(ast_stack) == 1");
            stack.pop(cur_node);
        } else if !node_children(cur_node).is_null() {
            assert_children_fit(cur_node);
            stack.push(children_slot(cur_node), node_children(cur_node));
        } else if !node_next(cur_node).is_null() {
            stack.push(next_slot(cur_node), node_next(cur_node));
        } else {
            // A leaf: nothing below it is left, so free it and empty its slot,
            // which is what turns its parent into a leaf in turn.
            stack.pop(cur_node);
            if matches!(
                node_type(cur_node),
                kExprNodeDoubleQuotedString | kExprNodeSingleQuotedString
            ) {
                // SAFETY: a string node owns the buffer `parse_quoted_string`
                // decoded into, and nothing else points at it.
                unsafe { xfree(node_str_value(cur_node).cast::<c_void>()) };
            }
            // SAFETY: the node came from `viml_pexpr_new_node`'s `xmalloc`,
            // and the walk has already freed everything it pointed at.
            unsafe { xfree(cur_node.cast::<c_void>()) };
            set_slot_node(cur_slot, ptr::null_mut());
        }
    }
}

/// A fresh node of the given type, with no children and no sibling. Its span
/// and its payload are the caller's to fill in.
#[inline]
pub(super) fn viml_pexpr_new_node(type_0: ExprASTNodeType) -> *mut ExprASTNode {
    // SAFETY: `xmalloc` answers a live allocation the size of a node, or dies
    // trying; the three fields every node has are written before it escapes.
    unsafe {
        let node = xmalloc(size_of::<ExprASTNode>()) as *mut ExprASTNode;
        (*node).type_0 = type_0;
        (*node).children = ptr::null_mut::<ExprASTNode>();
        (*node).next = ptr::null_mut::<ExprASTNode>();
        node
    }
}
pub(super) static node_type_to_node_props: [C2Rust_Unnamed_34; 39] = [
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlInvalid,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlTernary,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlTernaryValue,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComplexIdentifier,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComma,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlColon,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlArrow,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlComparison,
        ass: kEOpAssRight,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlSubscript,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlOr,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAnd,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    C2Rust_Unnamed_34 {
        lvl: kEOpLvlAssignment,
        ass: kEOpAssLeft,
    },
];
/// The precedence level a node binds at.
#[inline(always)]
pub(super) fn node_lvl(node: *mut ExprASTNode) -> ExprOpLvl {
    node_type_to_node_props[node_type(node) as usize].lvl
}

/// Which way a node of equal precedence associates.
#[inline(always)]
fn node_ass(node: *mut ExprASTNode) -> ExprOpAssociativity {
    node_type_to_node_props[node_type(node) as usize].ass
}

/// The shunting yard: splice a binary operator into the tree at the right
/// precedence, and answer whether the result is valid.
pub(super) fn viml_pexpr_handle_bop(
    pstate: *const ParserState,
    ast_stack: &mut Vec<*mut *mut ExprASTNode>,
    bop_node: *mut ExprASTNode,
    want_node: &mut ExprASTWantedNode,
    ast: *mut ExprAST,
) -> bool {
    let mut ret = true;
    let mut top_node_p: *mut *mut ExprASTNode = ptr::null_mut::<*mut ExprASTNode>();
    let mut top_node: *mut ExprASTNode = ptr::null_mut::<ExprASTNode>();
    let mut top_node_lvl: ExprOpLvl = kEOpLvlInvalid;
    let mut top_node_ass: ExprOpAssociativity = 0 as ExprOpAssociativity;
    debug_assert!(!ast_stack.is_empty(), "kv_size(*ast_stack)");
    // A call and a subscript are written as brackets rather than as operators,
    // so their own level says nothing about how tightly they bind.
    let bop_node_lvl = if matches!(node_type(bop_node), kExprNodeCall | kExprNodeSubscript) {
        kEOpLvlSubscript
    } else {
        node_lvl(bop_node)
    };
    // Unwind the branch as far as this operator outranks it.
    loop {
        let new_top_node_p = stack_top(ast_stack, 0);
        let new_top_node = slot_node(new_top_node_p);
        debug_assert!(!new_top_node.is_null(), "new_top_node != NULL");
        let new_top_node_lvl = node_lvl(new_top_node);
        let new_top_node_ass = node_ass(new_top_node);
        if !top_node_p.is_null()
            && (bop_node_lvl > new_top_node_lvl
                || bop_node_lvl == new_top_node_lvl && new_top_node_ass == kEOpAssNo)
        {
            break;
        }
        ast_stack.truncate(ast_stack.len() - 1);
        top_node_p = new_top_node_p;
        top_node = new_top_node;
        top_node_lvl = new_top_node_lvl;
        top_node_ass = new_top_node_ass;
        if bop_node_lvl == top_node_lvl && top_node_ass == kEOpAssRight {
            break;
        }
        if ast_stack.is_empty() {
            break;
        }
    }
    if top_node_ass == kEOpAssLeft || top_node_lvl != bop_node_lvl {
        // The operator takes the whole of what was unwound as its left
        // operand, and stands where that used to.
        set_slot_node(top_node_p, bop_node);
        set_node_children(bop_node, top_node);
        debug_assert!(
            node_next(node_children(bop_node)).is_null(),
            "bop_node->children->next == NULL"
        );
        ast_stack.push(top_node_p);
        ast_stack.push(next_slot(node_children(bop_node)));
    } else {
        assert!(
            top_node_lvl == bop_node_lvl && top_node_ass == kEOpAssRight,
            "top_node_lvl == bop_node_lvl && top_node_ass == kEOpAssRight"
        );
        // Right-associative and equal: the operator steals the right operand
        // of the one above it and becomes that operand instead.
        let top_children = node_children(top_node);
        debug_assert!(
            !top_children.is_null() && !node_next(top_children).is_null(),
            "top_node->children != NULL && top_node->children->next != NULL"
        );
        set_node_children(bop_node, node_next(top_children));
        set_slot_node(next_slot(top_children), bop_node);
        debug_assert!(
            node_next(node_children(bop_node)).is_null(),
            "bop_node->children->next == NULL"
        );
        ast_stack.push(top_node_p);
        ast_stack.push(next_slot(top_children));
        ast_stack.push(next_slot(node_children(bop_node)));
        if node_type(bop_node) == kExprNodeComparison {
            // SAFETY: a string literal is NUL-terminated and `gettext` only
            // reads through it.
            let msg = unsafe { gettext(c"E15: Operator is not associative: %.*s".as_ptr()) };
            east_set_error(pstate, ast, msg, node_start(bop_node));
            ret = false;
        }
    }
    *want_node = kENodeValue;
    ret
}

/// Translate a message for the parse error or for a token's `err.msg`.
///
/// A `CStr` is NUL-terminated by construction and `gettext` only reads through
/// it, so this is the whole of the obligation.
pub(super) fn translate(msg: &'static CStr) -> *const c_char {
    // SAFETY: as above.
    unsafe { gettext(msg.as_ptr()) }
}

/// Record `msg` as the parse error, unless an earlier one already stands.
/// `msg` must already be translated and must outlive the AST.
#[inline(always)]
pub(super) fn east_set_error(
    pstate: *const ParserState,
    ast: *mut ExprAST,
    msg: *const c_char,
    start: ParserPosition,
) {
    if ast_has_error(ast) {
        return;
    }
    // SAFETY: the parser holds both for the whole parse. `err` is a different
    // field from `root`, which the AST stack points into, so the reborrow does
    // not reach it.
    let (err, reader) = unsafe { (&mut (*ast).err, &(*pstate).reader) };
    let pline = reader_line(reader, start.line);
    err.msg = msg;
    err.arg_len = pline.size.wrapping_sub(start.col) as c_int;
    // `wrapping_add` because the C did: `start.col` is a position within the
    // line, so this is exact.
    err.arg = if pline.data.is_null() {
        ptr::null::<c_char>()
    } else {
        pline.data.wrapping_add(start.col)
    };
}
