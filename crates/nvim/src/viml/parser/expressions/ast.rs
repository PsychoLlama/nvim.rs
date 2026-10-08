//! The node arena, the slots the AST stack names, and the shunting-yard step
//! that attaches a binary operator to the tree.
//!
//! # Slots
//!
//! Upstream's AST stack holds `ExprASTNode **`s: the root pointer, or some
//! node's `children` or `next` field, each the place the next value goes.
//! Here the nodes live in [`ExprAST::nodes`] and name each other by
//! [`NodeId`], and a stack item is a [`Slot`] naming the same three places.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::CStr;

use super::*;
use crate::os::cshim::gettext;
use crate::types::ExprNodeFigure;

/// A place a node can hang from: the root of the tree, a node's first
/// child, or a node's next sibling.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Slot {
    Root,
    Children(NodeId),
    Next(NodeId),
}

impl<'a> ExprAST<'a> {
    /// An empty tree with no error.
    pub(super) fn new() -> Self {
        ExprAST {
            err: None,
            nodes: Vec::new(),
            root: None,
        }
    }

    /// The node `id` names.
    pub fn node(&self, id: NodeId) -> &ExprASTNode<'a> {
        &self.nodes[id.0]
    }

    fn node_mut(&mut self, id: NodeId) -> &mut ExprASTNode<'a> {
        &mut self.nodes[id.0]
    }

    /// A node's children, first to last.
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        core::iter::successors(self.node(id).children, |&child| self.node(child).next)
    }

    /// A fresh node of the given type, with no children and no sibling. Its
    /// span and its payload are the caller's to fill in.
    ///
    /// Upstream `xmalloc`s the node and leaves `start`, `len` and the payload
    /// as whatever the allocator handed back; they start zeroed here.
    pub(super) fn new_node(&mut self, type_0: ExprASTNodeType) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(ExprASTNode {
            type_0,
            children: None,
            next: None,
            start: ParserPosition { line: 0, col: 0 },
            len: 0,
            data: ExprNodeData::None,
        });
        id
    }

    /// A node's type tag.
    pub(super) fn kind(&self, id: NodeId) -> ExprASTNodeType {
        self.node(id).type_0
    }

    pub(super) fn set_kind(&mut self, id: NodeId, type_0: ExprASTNodeType) {
        self.node_mut(id).type_0 = type_0;
    }

    /// Where in the input the node starts.
    pub(super) fn start(&self, id: NodeId) -> ParserPosition {
        self.node(id).start
    }

    pub(super) fn set_span(&mut self, id: NodeId, start: ParserPosition, len: size_t) {
        let node = self.node_mut(id);
        node.start = start;
        node.len = len;
    }

    pub(super) fn set_len(&mut self, id: NodeId, len: size_t) {
        self.node_mut(id).len = len;
    }

    /// The node's first child.
    pub(super) fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).children
    }

    pub(super) fn set_children(&mut self, id: NodeId, children: Option<NodeId>) {
        self.node_mut(id).children = children;
    }

    /// The node's next sibling.
    pub(super) fn next(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).next
    }

    /// Whatever hangs from `slot`; `None` while the parser is still waiting
    /// for the value that goes there.
    pub(super) fn slot(&self, slot: Slot) -> Option<NodeId> {
        match slot {
            Slot::Root => self.root,
            Slot::Children(id) => self.node(id).children,
            Slot::Next(id) => self.node(id).next,
        }
    }

    /// The node hanging from a slot the parser knows to be filled.
    #[track_caller]
    pub(super) fn filled(&self, slot: Slot) -> NodeId {
        self.slot(slot).expect("the slot holds a node")
    }

    pub(super) fn set_slot(&mut self, slot: Slot, node: Option<NodeId>) {
        match slot {
            Slot::Root => self.root = node,
            Slot::Children(id) => self.node_mut(id).children = node,
            Slot::Next(id) => self.node_mut(id).next = node,
        }
    }

    /// Write the whole of a node's payload.
    pub(super) fn set_data(&mut self, id: NodeId, data: ExprNodeData<'a>) {
        self.node_mut(id).data = data;
    }

    /// A figure brace node's guesses at what it will turn out to be.
    pub(super) fn fig(&self, id: NodeId) -> ExprNodeFigure {
        *self.node(id).data.figure()
    }

    /// Whether a TernaryValue node has seen its `:` yet.
    pub(super) fn got_colon(&self, id: NodeId) -> bool {
        self.node(id).data.ternary().got_colon
    }

    /// The precedence level a node binds at.
    pub(super) fn lvl(&self, id: NodeId) -> ExprOpLvl {
        node_type_to_node_props[self.kind(id) as usize].lvl
    }

    /// Which way a node of equal precedence associates.
    fn ass(&self, id: NodeId) -> ExprOpAssociativity {
        node_type_to_node_props[self.kind(id) as usize].ass
    }

    /// Whether the parse has already reported an error. The first one wins.
    pub(super) fn has_error(&self) -> bool {
        self.err.is_some()
    }

    /// The child-count invariants upstream checked as it freed the tree:
    /// no node has more children than its type allows. Only the last is a
    /// hard `assert!`, as it was in the transpiled body.
    pub(super) fn check_children(&self) {
        for (index, node) in self.nodes.iter().enumerate() {
            let Some(first) = node.children else {
                continue;
            };
            let maxchildren = node_maxchildren[node.type_0 as usize];
            debug_assert!(maxchildren > 0, "maxchildren > 0");
            debug_assert!(maxchildren <= 2, "maxchildren <= 2");
            let second = self.node(first).next;
            assert!(
                if maxchildren == 1 {
                    second.is_none()
                } else {
                    second.is_none_or(|second| self.node(second).next.is_none())
                },
                "node {index} has no more children than its type allows"
            );
        }
    }
}

/// The comparison operators' names, by `ExprComparisonType`.
pub const COMPARISON_NAMES: [&CStr; 5] = [
    c"Equal",
    c"Matches",
    c"Greater",
    c"GreaterOrEqual",
    c"Identical",
];
/// The assignment operators' names, by `ExprAssignmentType`.
pub const ASSIGNMENT_NAMES: [&CStr; 4] = [c"Plain", c"Add", c"Subtract", c"Concat"];
/// The case-comparison strategies' names, by `ExprCaseCompareStrategy` --
/// the strategy's own suffix byte (`#`, `?`), so most slots are empty.
pub const CASE_STRATEGY_NAMES: [Option<&CStr>; 64] = [
    Some(c"UseOption"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(c"MatchCase"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(c"IgnoreCase"),
];
/// The node types' names, by `ExprASTNodeType`, as `nvim_parse_expression`
/// reports them.
pub const NODE_TYPE_NAMES: [&CStr; 39] = [
    c"Missing",
    c"OpMissing",
    c"Ternary",
    c"TernaryValue",
    c"Register",
    c"Subscript",
    c"ListLiteral",
    c"UnaryPlus",
    c"BinaryPlus",
    c"Nested",
    c"Call",
    c"PlainIdentifier",
    c"PlainKey",
    c"ComplexIdentifier",
    c"UnknownFigure",
    c"Lambda",
    c"DictLiteral",
    c"CurlyBracesIdentifier",
    c"Comma",
    c"Colon",
    c"Arrow",
    c"Comparison",
    c"Concat",
    c"ConcatOrSubscript",
    c"Integer",
    c"Float",
    c"SingleQuotedString",
    c"DoubleQuotedString",
    c"Or",
    c"And",
    c"UnaryMinus",
    c"BinaryMinus",
    c"Not",
    c"Multiplication",
    c"Division",
    c"Mod",
    c"Option",
    c"Environment",
    c"Assignment",
];
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
pub(super) static node_type_to_node_props: [ExprNodeProps; 39] = [
    ExprNodeProps {
        lvl: kEOpLvlInvalid,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlTernary,
        ass: kEOpAssRight,
    },
    ExprNodeProps {
        lvl: kEOpLvlTernaryValue,
        ass: kEOpAssRight,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlParens,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlParens,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlComplexIdentifier,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlComma,
        ass: kEOpAssRight,
    },
    ExprNodeProps {
        lvl: kEOpLvlColon,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlArrow,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlComparison,
        ass: kEOpAssRight,
    },
    ExprNodeProps {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlSubscript,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlOr,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlAnd,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlAddition,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlUnary,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlMultiplication,
        ass: kEOpAssLeft,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlValue,
        ass: kEOpAssNo,
    },
    ExprNodeProps {
        lvl: kEOpLvlAssignment,
        ass: kEOpAssLeft,
    },
];

/// The shunting yard: splice a binary operator into the tree at the right
/// precedence, and answer whether the result is valid.
pub(super) fn viml_pexpr_handle_bop<'a>(
    pstate: &ParserState<'a>,
    ast: &mut ExprAST<'a>,
    ast_stack: &mut Vec<Slot>,
    bop_node: NodeId,
    want_node: &mut ExprASTWantedNode,
) -> bool {
    let mut ret = true;
    let mut top_node_p: Option<Slot> = None;
    let mut top_node_lvl: ExprOpLvl = kEOpLvlInvalid;
    let mut top_node_ass: ExprOpAssociativity = 0 as ExprOpAssociativity;
    debug_assert!(!ast_stack.is_empty(), "kv_size(*ast_stack)");
    // A call and a subscript are written as brackets rather than as operators,
    // so their own level says nothing about how tightly they bind.
    let bop_node_lvl = if matches!(ast.kind(bop_node), kExprNodeCall | kExprNodeSubscript) {
        kEOpLvlSubscript
    } else {
        ast.lvl(bop_node)
    };
    // Unwind the branch as far as this operator outranks it.
    loop {
        let new_top_node_p = stack_top(ast_stack, 0);
        let new_top_node = ast.filled(new_top_node_p);
        let new_top_node_lvl = ast.lvl(new_top_node);
        let new_top_node_ass = ast.ass(new_top_node);
        if top_node_p.is_some()
            && (bop_node_lvl > new_top_node_lvl
                || bop_node_lvl == new_top_node_lvl && new_top_node_ass == kEOpAssNo)
        {
            break;
        }
        ast_stack.truncate(ast_stack.len() - 1);
        top_node_p = Some(new_top_node_p);
        top_node_lvl = new_top_node_lvl;
        top_node_ass = new_top_node_ass;
        if bop_node_lvl == top_node_lvl && top_node_ass == kEOpAssRight {
            break;
        }
        if ast_stack.is_empty() {
            break;
        }
    }
    let top_node_p = top_node_p.expect("the loop ran at least once");
    let top_node = ast.filled(top_node_p);
    if top_node_ass == kEOpAssLeft || top_node_lvl != bop_node_lvl {
        // The operator takes the whole of what was unwound as its left
        // operand, and stands where that used to.
        ast.set_slot(top_node_p, Some(bop_node));
        ast.set_children(bop_node, Some(top_node));
        debug_assert!(
            ast.next(top_node).is_none(),
            "bop_node->children->next == NULL"
        );
        ast_stack.push(top_node_p);
        ast_stack.push(Slot::Next(top_node));
    } else {
        assert!(
            top_node_lvl == bop_node_lvl && top_node_ass == kEOpAssRight,
            "top_node_lvl == bop_node_lvl && top_node_ass == kEOpAssRight"
        );
        // Right-associative and equal: the operator steals the right operand
        // of the one above it and becomes that operand instead.
        let top_children = ast
            .first_child(top_node)
            .expect("a right-associative operator has its left operand");
        let stolen = ast.next(top_children);
        debug_assert!(
            stolen.is_some(),
            "top_node->children != NULL && top_node->children->next != NULL"
        );
        ast.set_children(bop_node, stolen);
        ast.set_slot(Slot::Next(top_children), Some(bop_node));
        let left = ast.filled(Slot::Children(bop_node));
        debug_assert!(ast.next(left).is_none(), "bop_node->children->next == NULL");
        ast_stack.push(top_node_p);
        ast_stack.push(Slot::Next(top_children));
        ast_stack.push(Slot::Next(left));
        if ast.kind(bop_node) == kExprNodeComparison {
            let msg = gettext(c"E15: Operator is not associative: %.*s");
            let start = ast.start(bop_node);
            east_set_error(pstate, ast, msg, start);
            ret = false;
        }
    }
    *want_node = kENodeValue;
    ret
}

/// Translate a message for the parse error or for a token's `err.msg`.
pub(super) fn translate(msg: &'static CStr) -> &'static CStr {
    gettext(msg)
}

/// Record `msg` as the parse error, unless an earlier one already stands.
/// `msg` must already be translated.
pub(super) fn east_set_error<'a>(
    pstate: &ParserState<'a>,
    ast: &mut ExprAST<'a>,
    msg: &'static CStr,
    start: ParserPosition,
) {
    if ast.has_error() {
        return;
    }
    let arg = pstate.line(start.line).map(|line| {
        assert!(
            start.col <= line.len(),
            "`start.col` is a position within the line"
        );
        &line[start.col..]
    });
    ast.err = Some(ExprASTError { msg, arg });
}
