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
    if p.want_node == E_NODE_VALUE {
        // Value level: assume unary operator.
        let node = p.new_node(EXPR_NODE_UNARY_PLUS);
        p.ast.set_slot(p.top_node_p, Some(node));
        p.ast_stack.push(Slot::Children(node));
        p.hl_token(hl!(p, UnaryPlus));
    } else {
        let node = p.new_node(EXPR_NODE_BINARY_PLUS);
        p.add_op_node(node);
        p.hl_token(hl!(p, BinaryPlus));
    }
    p.want_node = E_NODE_VALUE;
    Flow::NextToken
}

/// `-`: a unary sign where a value is wanted, a subtraction where an operator
/// is.
pub(super) fn minus(p: &mut ExprParser) -> Flow {
    if p.want_node == E_NODE_VALUE {
        // Value level: assume unary operator.
        let node = p.new_node(EXPR_NODE_UNARY_MINUS);
        p.ast.set_slot(p.top_node_p, Some(node));
        p.ast_stack.push(Slot::Children(node));
        p.hl_token(hl!(p, UnaryMinus));
    } else {
        let node = p.new_node(EXPR_NODE_BINARY_MINUS);
        p.add_op_node(node);
        p.hl_token(hl!(p, BinaryMinus));
    }
    p.want_node = E_NODE_VALUE;
    Flow::NextToken
}

/// `||`.
pub(super) fn or(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected or operator: %.*s");
    let node = p.new_node(EXPR_NODE_OR);
    p.hl_token(hl!(p, Or));
    p.add_op_node(node);
    Flow::NextToken
}

/// `&&`.
pub(super) fn and(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected and operator: %.*s");
    let node = p.new_node(EXPR_NODE_AND);
    p.hl_token(hl!(p, And));
    p.add_op_node(node);
    Flow::NextToken
}

/// `*`, `/` and `%`.
pub(super) fn multiplication(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected multiplication-like operator: %.*s");
    let node = match p.cur_token.multiplication_type() {
        EXPR_LEX_MUL_MUL => {
            let node = p.new_node(EXPR_NODE_MULTIPLICATION);
            p.hl_token(hl!(p, Multiplication));
            node
        }
        EXPR_LEX_MUL_DIV => {
            let node = p.new_node(EXPR_NODE_DIVISION);
            p.hl_token(hl!(p, Division));
            node
        }
        EXPR_LEX_MUL_MOD => {
            let node = p.new_node(EXPR_NODE_MOD);
            p.hl_token(hl!(p, Mod));
            node
        }
        other => unreachable!("the lexer names no multiplication type {other}"),
    };
    p.add_op_node(node);
    Flow::NextToken
}

/// `!`.
pub(super) fn not(p: &mut ExprParser) -> Flow {
    if p.want_node == E_NODE_OPERATOR {
        return p.op_missing();
    }
    let node = p.new_node(EXPR_NODE_NOT);
    p.ast.set_slot(p.top_node_p, Some(node));
    p.ast_stack.push(Slot::Children(node));
    p.hl_token(hl!(p, Not));
    Flow::NextToken
}

/// `==`, `<`, `=~` and the rest, with their optional `#`/`?` case modifier.
pub(super) fn comparison(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Expected value, got comparison operator: %.*s");
    let node = p.new_node(EXPR_NODE_COMPARISON);
    let cmp = if p.cur_token.type_0 == EXPR_LEX_INVALID {
        ExprNodeComparison {
            type_0: EXPR_CMP_EQUAL,
            ccs: CC_STRATEGY_USE_OPTION,
            inv: false,
        }
    } else {
        ExprNodeComparison {
            type_0: p.cur_token.comparison().type_0,
            ccs: p.cur_token.comparison().ccs,
            inv: p.cur_token.comparison().inv,
        }
    };
    p.ast.set_data(node, ExprNodeData::Comparison(cmp));
    p.add_op_node(node);
    // Note: the strategy read here is the *token's*, which for an invalid
    // token is whatever the lexer left in `err`. The C reads the same bytes.
    if p.cur_token.comparison().ccs != CC_STRATEGY_USE_OPTION {
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
    p.want_node = E_NODE_VALUE;
    Flow::NextToken
}

/// `.`: concatenation, or a subscript when there is no spacing before it.
pub(super) fn dot(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected dot: %.*s");
    let node = if p.prev_token.type_0 == EXPR_LEX_SPACING {
        if p.cur_pt == EPT_ASSIGNMENT {
            p.error(c"E15: Cannot concatenate in assignments: %.*s");
        }
        let node = p.new_node(EXPR_NODE_CONCAT);
        p.hl_token(hl!(p, Concat));
        node
    } else {
        let node = p.new_node(EXPR_NODE_CONCAT_OR_SUBSCRIPT);
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
    let node = p.new_node(EXPR_NODE_TERNARY);
    p.add_op_node(node);
    p.hl_token(hl!(p, Ternary));
    let ter_val_node = p.new_node(EXPR_NODE_TERNARY_VALUE);
    p.ast.set_data(
        ter_val_node,
        ExprNodeData::Ternary(ExprNodeTernary { got_colon: false }),
    );
    let first = p.ast.filled(Slot::Children(node));
    debug_assert!(
        p.ast.next(first).is_none(),
        "cur_node->children->next == NULL"
    );
    debug_assert!(
        stack_top(&p.ast_stack, 0) == Slot::Next(first),
        "kv_last(ast_stack) == &cur_node->children->next"
    );
    p.ast
        .set_slot(stack_top(&p.ast_stack, 0), Some(ter_val_node));
    p.ast_stack.push(Slot::Children(ter_val_node));
    Flow::NextToken
}

/// `->`: closes a lambda's argument list, or is reported as misplaced.
pub(super) fn arrow(p: &mut ExprParser) -> Flow {
    // A closer may have unwound the AST stack past an argument list that had
    // already failed (`{a,]->`), leaving its parse type behind: that list is
    // gone, and this arrow is outside of it.
    while p.cur_pt == EPT_LAMBDA_ARGUMENTS {
        let lambda_node = p.lambda_node();
        if p.ast_stack
            .iter()
            .any(|&slot| p.ast.slot(slot) == Some(lambda_node))
        {
            break;
        }
        debug_assert!(
            p.ast.has_error(),
            "only a failed argument list is left behind"
        );
        p.pop_lambda_arguments();
        p.cur_pt = p.pt_top();
    }
    if p.cur_pt == EPT_LAMBDA_ARGUMENTS {
        let lambda_node = p.lambda_node();
        p.pop_lambda_arguments();
        debug_assert!(!p.pt_stack.is_empty(), "kv_size(pt_stack)");
        if p.want_node == E_NODE_VALUE {
            // Wanting a value means a trailing comma and NULL at the top of
            // the stack.
            p.ast_stack.truncate(p.ast_stack.len() - 1);
        }
        // Down to the brace, which the loop above found on the stack.
        while p.ast.slot(stack_top(&p.ast_stack, 0)) != Some(lambda_node) {
            p.ast_stack.truncate(p.ast_stack.len() - 1);
        }
        p.select_figure_brace_type(lambda_node, EXPR_NODE_LAMBDA, hl!(p, Lambda));
        let node = p.new_node(EXPR_NODE_ARROW);
        match p.ast.first_child(lambda_node) {
            None => {
                debug_assert!(p.want_node == E_NODE_VALUE, "want_node == kENodeValue");
                p.ast.set_children(lambda_node, Some(node));
                p.ast_stack.push(Slot::Children(lambda_node));
            }
            Some(first) => {
                debug_assert!(
                    p.ast.next(first).is_none(),
                    "lambda_node->children->next == NULL"
                );
                p.ast.set_slot(Slot::Next(first), Some(node));
                p.ast_stack.push(Slot::Next(first));
            }
        }
        p.ast_stack.push(Slot::Children(node));
    } else {
        // Only the first branch is valid.
        p.add_value_if_missing(c"E15: Unexpected arrow: %.*s");
        p.error(c"E15: Arrow outside of lambda: %.*s");
        let node = p.new_node(EXPR_NODE_ARROW);
        p.add_op_node(node);
    }
    p.want_node = E_NODE_VALUE;
    p.hl_token(hl!(p, Arrow));
    Flow::NextToken
}

/// `=`, `+=`, `-=` and `.=`: only valid while parsing an assignment lvalue.
pub(super) fn assignment(p: &mut ExprParser) -> Flow {
    if p.cur_pt == EPT_ASSIGNMENT {
        p.pt_stack.truncate(p.pt_stack.len() - 1);
    } else if p.cur_pt == EPT_SINGLE_ASSIGNMENT {
        p.pt_stack.truncate(p.pt_stack.len() - 2);
        p.error(c"E475: Expected closing bracket to end list assignment lvalue: %.*s");
    } else {
        p.error(c"E15: Misplaced assignment: %.*s");
    }
    debug_assert!(!p.pt_stack.is_empty(), "kv_size(pt_stack)");
    // A misplaced `=` can also come among lambda arguments (`{a,=`), after
    // the parse-type reconciliation has reported them.
    debug_assert!(
        p.pt_top() == EPT_EXPR || p.ast.has_error(),
        "an expression is being parsed, or an error stands"
    );
    p.add_value_if_missing(c"E15: Unexpected assignment: %.*s");
    let node = p.new_node(EXPR_NODE_ASSIGNMENT);
    p.ast.set_data(
        node,
        ExprNodeData::Assignment(ExprNodeAssignment {
            type_0: p.cur_token.assignment_type(),
        }),
    );
    match p.cur_token.assignment_type() {
        EXPR_ASGN_PLAIN => p.hl_token(hl!(p, PlainAssignment)),
        EXPR_ASGN_ADD => p.hl_token(hl!(p, AssignmentWithAddition)),
        EXPR_ASGN_SUBTRACT => p.hl_token(hl!(p, AssignmentWithSubtraction)),
        EXPR_ASGN_CONCAT => p.hl_token(hl!(p, AssignmentWithConcatenation)),
        _ => {}
    }
    p.add_op_node(node);
    Flow::NextToken
}
