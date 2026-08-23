//! Token handlers for the operators: the arithmetic, logical, comparison and
//! assignment tokens, plus the ternary and the lambda arrow.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::parse::{ExprParser, Flow, hl};
use super::*;

/// `+`: a unary sign where a value is wanted, an addition where an operator
/// is.
pub(super) fn plus(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeValue {
        // Value level: assume unary operator.
        let node = p.new_node(kExprNodeUnaryPlus);
        set_slot_node(p.top_node_p, node);
        p.ast_stack.push(children_slot(node));
        p.hl_token(hl!(p, UnaryPlus));
    } else {
        let node = p.new_node(kExprNodeBinaryPlus);
        p.add_op_node(node);
        p.hl_token(hl!(p, BinaryPlus));
    }
    p.want_node = kENodeValue;
    Flow::NextToken
}

/// `-`: a unary sign where a value is wanted, a subtraction where an operator
/// is.
pub(super) fn minus(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeValue {
        // Value level: assume unary operator.
        let node = p.new_node(kExprNodeUnaryMinus);
        set_slot_node(p.top_node_p, node);
        p.ast_stack.push(children_slot(node));
        p.hl_token(hl!(p, UnaryMinus));
    } else {
        let node = p.new_node(kExprNodeBinaryMinus);
        p.add_op_node(node);
        p.hl_token(hl!(p, BinaryMinus));
    }
    p.want_node = kENodeValue;
    Flow::NextToken
}

/// `||`.
pub(super) fn or(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected or operator: %.*s");
    let node = p.new_node(kExprNodeOr);
    p.hl_token(hl!(p, Or));
    p.add_op_node(node);
    Flow::NextToken
}

/// `&&`.
pub(super) fn and(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected and operator: %.*s");
    let node = p.new_node(kExprNodeAnd);
    p.hl_token(hl!(p, And));
    p.add_op_node(node);
    Flow::NextToken
}

/// `*`, `/` and `%`.
pub(super) fn multiplication(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected multiplication-like operator: %.*s");
    let mut node = ::core::ptr::null_mut::<ExprASTNode>();
    match p.cur_token.multiplication_type() {
        kExprLexMulMul => {
            node = p.new_node(kExprNodeMultiplication);
            p.hl_token(hl!(p, Multiplication));
        }
        kExprLexMulDiv => {
            node = p.new_node(kExprNodeDivision);
            p.hl_token(hl!(p, Division));
        }
        kExprLexMulMod => {
            node = p.new_node(kExprNodeMod);
            p.hl_token(hl!(p, Mod));
        }
        _ => {}
    }
    p.add_op_node(node);
    Flow::NextToken
}

/// `!`.
pub(super) fn not(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeNot);
    set_slot_node(p.top_node_p, node);
    p.ast_stack.push(children_slot(node));
    p.hl_token(hl!(p, Not));
    Flow::NextToken
}

/// `==`, `<`, `=~` and the rest, with their optional `#`/`?` case modifier.
pub(super) fn comparison(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Expected value, got comparison operator: %.*s");
    let node = p.new_node(kExprNodeComparison);
    let cmp = if p.cur_token.type_0 == kExprLexInvalid {
        ExprNodeComparison {
            type_0: kExprCmpEqual,
            ccs: kCCStrategyUseOption,
            inv: false,
        }
    } else {
        ExprNodeComparison {
            type_0: p.cur_token.comparison().type_0,
            ccs: p.cur_token.comparison().ccs,
            inv: p.cur_token.comparison().inv,
        }
    };
    set_node_data(node, ExprNodeData::Comparison(cmp));
    p.add_op_node(node);
    // Note: the strategy read here is the *token's*, which for an invalid
    // token is whatever the lexer left in `err`. The C reads the same bytes.
    if p.cur_token.comparison().ccs != kCCStrategyUseOption {
        p.hl_at(
            p.cur_token.start,
            p.cur_token.len.wrapping_sub(1),
            hl!(p, Comparison),
        );
        p.hl_at(
            shifted_pos(p.cur_token.start, p.cur_token.len.wrapping_sub(1)),
            1,
            hl!(p, ComparisonModifier),
        );
    } else {
        p.hl_token(hl!(p, Comparison));
    }
    p.want_node = kENodeValue;
    Flow::NextToken
}

/// `.`: concatenation, or a subscript when there is no spacing before it.
pub(super) fn dot(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected dot: %.*s");
    let node = if p.prev_token.type_0 == kExprLexSpacing {
        if p.cur_pt == kEPTAssignment {
            p.error(c"E15: Cannot concatenate in assignments: %.*s");
        }
        let node = p.new_node(kExprNodeConcat);
        p.hl_token(hl!(p, Concat));
        node
    } else {
        let node = p.new_node(kExprNodeConcatOrSubscript);
        p.hl_token(hl!(p, ConcatOrSubscript));
        node
    };
    p.add_op_node(node);
    Flow::NextToken
}

/// `?`: opens a ternary, whose second operand is a TernaryValue node waiting
/// for its colon.
pub(super) fn question(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Expected value, got question mark: %.*s");
    let node = p.new_node(kExprNodeTernary);
    p.add_op_node(node);
    p.hl_token(hl!(p, Ternary));
    let ter_val_node = p.new_node(kExprNodeTernaryValue);
    set_node_data(
        ter_val_node,
        ExprNodeData::Ternary(ExprNodeTernary { got_colon: false }),
    );
    let first = node_children(node);
    debug_assert!(!first.is_null(), "cur_node->children != NULL");
    debug_assert!(
        node_next(first).is_null(),
        "cur_node->children->next == NULL"
    );
    debug_assert!(
        stack_top(&p.ast_stack, 0) == next_slot(first),
        "kv_last(ast_stack) == &cur_node->children->next"
    );
    set_slot_node(stack_top(&p.ast_stack, 0), ter_val_node);
    p.ast_stack.push(children_slot(ter_val_node));
    Flow::NextToken
}

/// `->`: closes a lambda's argument list, or is reported as misplaced.
pub(super) fn arrow(p: &mut ExprParser) -> Flow {
    if p.cur_pt == kEPTLambdaArguments {
        p.pt_stack.truncate(p.pt_stack.len() - 1);
        debug_assert!(!p.pt_stack.is_empty(), "kv_size(pt_stack)");
        if p.want_node == kENodeValue {
            // Wanting a value means a trailing comma and NULL at the top of
            // the stack.
            p.ast_stack.truncate(p.ast_stack.len() - 1);
        }
        debug_assert!(!p.ast_stack.is_empty(), "kv_size(ast_stack) >= 1");
        while !matches!(
            node_type(slot_node(stack_top(&p.ast_stack, 0))),
            kExprNodeLambda | kExprNodeUnknownFigure
        ) {
            p.ast_stack.truncate(p.ast_stack.len() - 1);
        }
        debug_assert!(
            slot_node(stack_top(&p.ast_stack, 0)) == p.lambda_node,
            "(*kv_last(ast_stack)) == lambda_node"
        );
        let lambda_node = p.lambda_node;
        p.select_figure_brace_type(lambda_node, kExprNodeLambda, hl!(p, Lambda));
        let node = p.new_node(kExprNodeArrow);
        let first = node_children(lambda_node);
        if first.is_null() {
            debug_assert!(p.want_node == kENodeValue, "want_node == kENodeValue");
            set_node_children(lambda_node, node);
            p.ast_stack.push(children_slot(lambda_node));
        } else {
            debug_assert!(
                node_next(first).is_null(),
                "lambda_node->children->next == NULL"
            );
            set_slot_node(next_slot(first), node);
            p.ast_stack.push(next_slot(first));
        }
        p.ast_stack.push(children_slot(node));
        p.lambda_node = ::core::ptr::null_mut::<ExprASTNode>();
    } else {
        // Only the first branch is valid.
        p.add_value_if_missing(c"E15: Unexpected arrow: %.*s");
        p.error(c"E15: Arrow outside of lambda: %.*s");
        let node = p.new_node(kExprNodeArrow);
        p.add_op_node(node);
    }
    p.want_node = kENodeValue;
    p.hl_token(hl!(p, Arrow));
    Flow::NextToken
}

/// `=`, `+=`, `-=` and `.=`: only valid while parsing an assignment lvalue.
pub(super) fn assignment(p: &mut ExprParser) -> Flow {
    if p.cur_pt == kEPTAssignment {
        p.pt_stack.truncate(p.pt_stack.len() - 1);
    } else if p.cur_pt == kEPTSingleAssignment {
        p.pt_stack.truncate(p.pt_stack.len() - 2);
        p.error(c"E475: Expected closing bracket to end list assignment lvalue: %.*s");
    } else {
        p.error(c"E15: Misplaced assignment: %.*s");
    }
    debug_assert!(!p.pt_stack.is_empty(), "kv_size(pt_stack)");
    debug_assert!(p.pt_top() == kEPTExpr, "kv_last(pt_stack) == kEPTExpr");
    p.add_value_if_missing(c"E15: Unexpected assignment: %.*s");
    let node = p.new_node(kExprNodeAssignment);
    set_node_data(
        node,
        ExprNodeData::Assignment(ExprNodeAssignment {
            type_0: p.cur_token.assignment_type(),
        }),
    );
    match p.cur_token.assignment_type() {
        kExprAsgnPlain => p.hl_token(hl!(p, PlainAssignment)),
        kExprAsgnAdd => p.hl_token(hl!(p, AssignmentWithAddition)),
        kExprAsgnSubtract => p.hl_token(hl!(p, AssignmentWithSubtraction)),
        kExprAsgnConcat => p.hl_token(hl!(p, AssignmentWithConcatenation)),
        _ => {}
    }
    p.add_op_node(node);
    Flow::NextToken
}
