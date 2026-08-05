//! Token handlers for the operators: the arithmetic, logical, comparison and
//! assignment tokens, plus the ternary and the lambda arrow.

use super::parse::{ExprParser, Flow, hl};
use super::*;

/// `+`: a unary sign where a value is wanted, an addition where an operator
/// is.
pub(super) unsafe fn plus(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeValue {
        // Value level: assume unary operator.
        let node = p.new_node(kExprNodeUnaryPlus);
        *p.top_node_p = node;
        p.ast_stack.push(&raw mut (*node).children);
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
pub(super) unsafe fn minus(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeValue {
        // Value level: assume unary operator.
        let node = p.new_node(kExprNodeUnaryMinus);
        *p.top_node_p = node;
        p.ast_stack.push(&raw mut (*node).children);
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
pub(super) unsafe fn or(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected or operator: %.*s");
    let node = p.new_node(kExprNodeOr);
    p.hl_token(hl!(p, Or));
    p.add_op_node(node);
    Flow::NextToken
}

/// `&&`.
pub(super) unsafe fn and(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected and operator: %.*s");
    let node = p.new_node(kExprNodeAnd);
    p.hl_token(hl!(p, And));
    p.add_op_node(node);
    Flow::NextToken
}

/// `*`, `/` and `%`.
pub(super) unsafe fn multiplication(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Unexpected multiplication-like operator: %.*s");
    let mut node = ::core::ptr::null_mut::<ExprASTNode>();
    match p.cur_token.data.mul.type_0 {
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
pub(super) unsafe fn not(p: &mut ExprParser) -> Flow {
    if p.want_node == kENodeOperator {
        return p.op_missing();
    }
    let node = p.new_node(kExprNodeNot);
    *p.top_node_p = node;
    p.ast_stack.push(&raw mut (*node).children);
    p.hl_token(hl!(p, Not));
    Flow::NextToken
}

/// `==`, `<`, `=~` and the rest, with their optional `#`/`?` case modifier.
pub(super) unsafe fn comparison(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Expected value, got comparison operator: %.*s");
    let node = p.new_node(kExprNodeComparison);
    if p.cur_token.type_0 == kExprLexInvalid {
        (*node).data.cmp.ccs = kCCStrategyUseOption;
        (*node).data.cmp.type_0 = kExprCmpEqual;
        (*node).data.cmp.inv = false;
    } else {
        (*node).data.cmp.ccs = p.cur_token.data.cmp.ccs;
        (*node).data.cmp.type_0 = p.cur_token.data.cmp.type_0;
        (*node).data.cmp.inv = p.cur_token.data.cmp.inv;
    }
    p.add_op_node(node);
    if p.cur_token.data.cmp.ccs != kCCStrategyUseOption {
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
pub(super) unsafe fn dot(p: &mut ExprParser) -> Flow {
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
pub(super) unsafe fn question(p: &mut ExprParser) -> Flow {
    p.add_value_if_missing(c"E15: Expected value, got question mark: %.*s");
    let node = p.new_node(kExprNodeTernary);
    p.add_op_node(node);
    p.hl_token(hl!(p, Ternary));
    let ter_val_node = p.new_node(kExprNodeTernaryValue);
    (*ter_val_node).data.ter.got_colon = false;
    assert!(!(*node).children.is_null(), "cur_node->children != NULL");
    assert!(
        (*(*node).children).next.is_null(),
        "cur_node->children->next == NULL"
    );
    assert!(
        stack_top(&p.ast_stack, 0) == &raw mut (*(*node).children).next,
        "kv_last(ast_stack) == &cur_node->children->next"
    );
    *stack_top(&p.ast_stack, 0) = ter_val_node;
    p.ast_stack.push(&raw mut (*ter_val_node).children);
    Flow::NextToken
}

/// `->`: closes a lambda's argument list, or is reported as misplaced.
pub(super) unsafe fn arrow(p: &mut ExprParser) -> Flow {
    if p.cur_pt == kEPTLambdaArguments {
        p.pt_stack.truncate(p.pt_stack.len() - 1);
        assert!(!p.pt_stack.is_empty(), "kv_size(pt_stack)");
        if p.want_node == kENodeValue {
            // Wanting a value means a trailing comma and NULL at the top of
            // the stack.
            p.ast_stack.truncate(p.ast_stack.len() - 1);
        }
        assert!(!p.ast_stack.is_empty(), "kv_size(ast_stack) >= 1");
        while (**stack_top(&p.ast_stack, 0)).type_0 != kExprNodeLambda
            && (**stack_top(&p.ast_stack, 0)).type_0 != kExprNodeUnknownFigure
        {
            p.ast_stack.truncate(p.ast_stack.len() - 1);
        }
        assert!(
            *stack_top(&p.ast_stack, 0) == p.lambda_node,
            "(*kv_last(ast_stack)) == lambda_node"
        );
        let lambda_node = p.lambda_node;
        p.select_figure_brace_type(lambda_node, kExprNodeLambda, hl!(p, Lambda));
        let node = p.new_node(kExprNodeArrow);
        if (*lambda_node).children.is_null() {
            debug_assert!(p.want_node == kENodeValue, "want_node == kENodeValue");
            (*lambda_node).children = node;
            p.ast_stack.push(&raw mut (*lambda_node).children);
        } else {
            assert!(
                (*(*lambda_node).children).next.is_null(),
                "lambda_node->children->next == NULL"
            );
            (*(*lambda_node).children).next = node;
            p.ast_stack.push(&raw mut (*(*lambda_node).children).next);
        }
        p.ast_stack.push(&raw mut (*node).children);
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
pub(super) unsafe fn assignment(p: &mut ExprParser) -> Flow {
    if p.cur_pt == kEPTAssignment {
        p.pt_stack.truncate(p.pt_stack.len() - 1);
    } else if p.cur_pt == kEPTSingleAssignment {
        p.pt_stack.truncate(p.pt_stack.len() - 2);
        p.error(c"E475: Expected closing bracket to end list assignment lvalue: %.*s");
    } else {
        p.error(c"E15: Misplaced assignment: %.*s");
    }
    assert!(!p.pt_stack.is_empty(), "kv_size(pt_stack)");
    assert!(p.pt_top() == kEPTExpr, "kv_last(pt_stack) == kEPTExpr");
    p.add_value_if_missing(c"E15: Unexpected assignment: %.*s");
    let node = p.new_node(kExprNodeAssignment);
    (*node).data.ass.type_0 = p.cur_token.data.ass.type_0;
    match p.cur_token.data.ass.type_0 {
        kExprAsgnPlain => p.hl_token(hl!(p, PlainAssignment)),
        kExprAsgnAdd => p.hl_token(hl!(p, AssignmentWithAddition)),
        kExprAsgnSubtract => p.hl_token(hl!(p, AssignmentWithSubtraction)),
        kExprAsgnConcat => p.hl_token(hl!(p, AssignmentWithConcatenation)),
        _ => {}
    }
    p.add_op_node(node);
    Flow::NextToken
}
