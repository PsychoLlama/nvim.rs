//! Token handlers for `[`, `]`, `(`, `)` and the `,` and `:` separators that
//! only mean anything inside them (or inside a figure brace, or a ternary).
//!
//! The separators walk the AST stack looking for the construct they belong
//! to; failing to find one is what produces "Comma outside of call, lambda or
//! literal" and its colon counterpart.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::parse::{ExprParser, Flow, hl, pt_is_assignment};
use super::*;

/// `,`: valid inside a call, a lambda argument list, a list or a dictionary.
pub(super) fn comma(p: &mut ExprParser) -> Flow {
    debug_assert!(
        !(p.want_node == E_NODE_VALUE && p.cur_pt == EPT_LAMBDA_ARGUMENTS),
        "!(want_node == kENodeValue && cur_pt == kEPTLambdaArguments)"
    );
    if p.want_node == E_NODE_VALUE {
        // Value level: a comma appearing here is not valid.
        // Note: in Vim string(,x) gives E116; that is not the case here.
        p.error(c"E15: Expected value, got comma: %.*s");
        let node = p.new_node(EXPR_NODE_MISSING);
        p.ast.set_len(node, 0);
        p.ast.set_slot(p.top_node_p, Some(node));
        p.want_node = E_NODE_OPERATOR;
    }
    if p.cur_pt == EPT_LAMBDA_ARGUMENTS {
        let lambda_node = p.lambda_node.expect("lambda_node != NULL");
        debug_assert!(
            p.ast.fig(lambda_node).type_guesses.allow_lambda,
            "lambda_node->data.fig.type_guesses.allow_lambda"
        );
        p.select_figure_brace_type(lambda_node, EXPR_NODE_LAMBDA, hl!(p, Lambda));
    }
    if !comma_has_a_home(p) {
        p.error(c"E15: Comma outside of call, lambda or literal: %.*s");
    }
    let node = p.new_node(EXPR_NODE_COMMA);
    p.add_op_node(node);
    p.hl_token(hl!(p, Comma));
    Flow::NextToken
}

/// Walk the AST stack for the construct this comma separates the parts of.
fn comma_has_a_home(p: &ExprParser) -> bool {
    if p.ast_stack.len() < 2 {
        return false;
    }
    let mut i: size_t = 1;
    loop {
        if i >= p.ast_stack.len() {
            return true;
        }
        let node = p.ast.filled(p.ast_stack[p.ast_stack.len() - i - 1]);
        let node_type = p.ast.kind(node);
        let lvl = p.ast.lvl(node);
        if node_type == EXPR_NODE_LAMBDA {
            debug_assert!(
                p.cur_pt == EPT_LAMBDA_ARGUMENTS && p.want_node == E_NODE_OPERATOR,
                "cur_pt == kEPTLambdaArguments && want_node == kENodeOperator"
            );
            return true;
        }
        if node_type == EXPR_NODE_DICT_LITERAL
            || node_type == EXPR_NODE_LIST_LITERAL
            || node_type == EXPR_NODE_CALL
        {
            return true;
        }
        if !(node_type == EXPR_NODE_COMMA || node_type == EXPR_NODE_COLON || lvl > E_OP_LVL_COMMA) {
            return false;
        }
        if i == p.ast_stack.len().wrapping_sub(1) {
            return false;
        }
        i = i.wrapping_add(1);
    }
}

/// `:`: the second half of a ternary, a dictionary literal's key separator, or
/// a slice within a subscript.
pub(super) fn colon(p: &mut ExprParser) -> Flow {
    const EXPECTED_VALUE: &::core::ffi::CStr = c"E15: Expected value, got colon: %.*s";

    let mut is_ternary = false;
    let mut is_subscript = false;
    // Walk the AST stack for what this colon belongs to.
    let has_a_home = 'scan: {
        if p.ast_stack.len() < 2 {
            break 'scan false;
        }
        let mut can_be_ternary = true;
        let mut i: size_t = 1;
        while i < p.ast_stack.len() {
            let node = p.ast.filled(p.ast_stack[p.ast_stack.len() - i - 1]);
            let node_type = p.ast.kind(node);
            let lvl = p.ast.lvl(node);
            // Assumes kEOpLvlTernary > kEOpLvlComma.
            if can_be_ternary && node_type == EXPR_NODE_TERNARY_VALUE && !p.ast.got_colon(node) {
                p.ast_stack.truncate(p.ast_stack.len() - i);
                if p.prev_token.type_0 == EXPR_LEX_SPACING {
                    let len = p.cur_token.len.wrapping_add(p.prev_token.len);
                    p.ast.set_span(node, p.prev_token.start, len);
                } else {
                    p.ast.set_span(node, p.cur_token.start, p.cur_token.len);
                }
                is_ternary = true;
                p.ast.set_data(
                    node,
                    ExprNodeData::Ternary(ExprNodeTernary { got_colon: true }),
                );
                p.add_value_if_missing(EXPECTED_VALUE);
                let first = p.ast.filled(Slot::Children(node));
                debug_assert!(
                    p.ast.next(first).is_none(),
                    "(*eastnode_p)->children->next == NULL"
                );
                p.ast_stack.push(Slot::Next(first));
                break;
            } else if node_type == EXPR_NODE_UNKNOWN_FIGURE {
                p.select_figure_brace_type(node, EXPR_NODE_DICT_LITERAL, hl!(p, Dict));
                break;
            } else if node_type == EXPR_NODE_DICT_LITERAL {
                break;
            } else if node_type == EXPR_NODE_SUBSCRIPT {
                is_subscript = true;
                // can_be_ternary = false;
                debug_assert!(!is_ternary, "!is_ternary");
                break;
            } else if node_type == EXPR_NODE_COLON {
                break 'scan false;
            } else {
                if lvl < E_OP_LVL_TERNARY_VALUE {
                    if lvl < E_OP_LVL_COMMA {
                        break 'scan false;
                    }
                    can_be_ternary = false;
                }
                if i == p.ast_stack.len().wrapping_sub(1) {
                    break 'scan false;
                }
                i = i.wrapping_add(1);
            }
        }
        true
    };
    if is_subscript {
        debug_assert!(p.ast_stack.len() > 1, "kv_size(ast_stack) > 1");
        if p.want_node == E_NODE_VALUE
            && p.ast.kind(p.ast.filled(stack_top(&p.ast_stack, 1))) == EXPR_NODE_SUBSCRIPT
        {
            // Colon immediately following the subscript start: an empty
            // subscript part like a[:2].
            let node = p.new_node(EXPR_NODE_MISSING);
            p.ast.set_len(node, 0);
            p.ast.set_slot(p.top_node_p, Some(node));
            p.want_node = E_NODE_OPERATOR;
        } else {
            p.add_value_if_missing(EXPECTED_VALUE);
        }
        let node = p.new_node(EXPR_NODE_COLON);
        p.add_op_node(node);
        p.hl_token(hl!(p, SubscriptColon));
    } else {
        if !has_a_home {
            p.error(c"E15: Colon outside of dictionary or ternary operator: %.*s");
        }
        p.add_value_if_missing(EXPECTED_VALUE);
        if is_ternary {
            p.hl_token(hl!(p, TernaryColon));
        } else {
            let node = p.new_node(EXPR_NODE_COLON);
            p.add_op_node(node);
            p.hl_token(hl!(p, Colon));
        }
    }
    p.want_node = E_NODE_VALUE;
    Flow::NextToken
}

/// `[`: a list literal where a value is wanted, a subscript where an operator
/// is. `]` closes whichever of the two the stack is inside.
pub(super) fn bracket(p: &mut ExprParser) -> Flow {
    if p.cur_token.is_closing() {
        return closing_bracket(p);
    }
    if p.want_node == E_NODE_VALUE {
        // Value means list literal or list assignment.
        let node = p.new_node(EXPR_NODE_LIST_LITERAL);
        p.ast.set_slot(p.top_node_p, Some(node));
        p.ast_stack.push(Slot::Children(node));
        if p.cur_pt == EPT_ASSIGNMENT {
            // The additional assignment parse type makes it easy to forbid
            // nested lists.
            p.pt_stack.push(EPT_SINGLE_ASSIGNMENT);
        } else if p.cur_pt == EPT_SINGLE_ASSIGNMENT {
            p.error(c"E475: Nested lists not allowed when assigning: %.*s");
        }
        p.hl_token(hl!(p, List));
        return Flow::NextToken;
    }
    // Operator means subscript, also in an assignment. But there a subscript
    // may be pretty much any expression, so kEPTExpr has to be pushed.
    if p.prev_token.type_0 == EXPR_LEX_SPACING {
        return p.op_missing();
    }
    let node = p.new_node(EXPR_NODE_SUBSCRIPT);
    p.add_op_node(node);
    p.hl_token(hl!(p, SubscriptBracket));
    if pt_is_assignment(p.cur_pt) {
        debug_assert!(p.want_node == E_NODE_VALUE, "want_node == kENodeValue");
        // Subtract 1 for the NULL at the top.
        p.asgn_level = p.ast_stack.len().wrapping_sub(1);
        p.pt_stack.push(EPT_EXPR);
    }
    Flow::NextToken
}

fn closing_bracket(p: &mut ExprParser) -> Flow {
    // Always drop the topmost value:
    //
    // 1. When want_node != kENodeValue the topmost item is a *finished* left
    //    operand, which may as well be "[@a]" and needs not be finished again.
    // 2. Otherwise it points at NULL, which nobody wants.
    p.ast_stack.truncate(p.ast_stack.len() - 1);
    let new_top_node_p: Slot;
    let mut unexpected = false;
    if p.ast_stack.is_empty() {
        let node = p.new_node(EXPR_NODE_LIST_LITERAL);
        p.ast.set_len(node, 0);
        if p.want_node != E_NODE_VALUE {
            let operand = p.ast.slot(p.top_node_p);
            p.ast.set_children(node, operand);
        }
        p.ast.set_slot(p.top_node_p, Some(node));
        new_top_node_p = p.top_node_p;
        unexpected = true;
    } else {
        if p.want_node == E_NODE_VALUE
            && !matches!(
                p.ast.kind(p.ast.filled(stack_top(&p.ast_stack, 0))),
                EXPR_NODE_LIST_LITERAL | EXPR_NODE_COMMA | EXPR_NODE_COLON
            )
        {
            // It is OK to want a value if
            //
            // 1. it is an empty list literal, in which case the top node is
            //    ListLiteral;
            // 2. it is a list literal with a trailing comma, in which case the
            //    top node is that comma;
            // 3. it is a subscript with a colon but without one of the values,
            //    e.g. "a[:]" or "a[1:]", in which case the top node is a colon.
            p.error(c"E15: Expected value, got closing bracket: %.*s");
        }
        let mut slot;
        loop {
            slot = p.ast_stack.pop().expect("the stack is not empty");
            if !(!p.ast_stack.is_empty()
                && !matches!(
                    p.ast.kind(p.ast.filled(slot)),
                    EXPR_NODE_LIST_LITERAL | EXPR_NODE_SUBSCRIPT
                ))
            {
                break;
            }
        }
        new_top_node_p = slot;
        let new_top_node = p.ast.filled(new_top_node_p);
        match p.ast.kind(new_top_node) {
            EXPR_NODE_LIST_LITERAL => {
                if pt_is_assignment(p.cur_pt) && p.ast.first_child(new_top_node).is_none() {
                    p.error(c"E475: Unable to assign to empty list: %.*s");
                }
                p.hl_token(hl!(p, List));
            }
            EXPR_NODE_SUBSCRIPT => p.hl_token(hl!(p, SubscriptBracket)),
            _ => unexpected = true,
        }
    }
    if unexpected {
        debug_assert!(p.ast_stack.is_empty(), "!kv_size(ast_stack)");
        p.error(c"E15: Unexpected closing figure brace: %.*s");
        p.hl_token(hl!(p, List));
    }
    p.ast_stack.push(new_top_node_p);
    p.want_node = E_NODE_OPERATOR;
    if p.ast_stack.len() <= p.asgn_level {
        debug_assert!(
            p.ast_stack.len() == p.asgn_level,
            "kv_size(ast_stack) == asgn_level"
        );
        p.asgn_level = 0;
        if p.cur_pt == EPT_ASSIGNMENT {
            debug_assert!(p.ast.has_error(), "ast.err.msg");
        } else if p.cur_pt == EPT_EXPR
            && p.pt_stack.len() > 1
            && pt_is_assignment(p.pt_stack[p.pt_stack.len() - 2])
        {
            p.pt_stack.truncate(p.pt_stack.len() - 1);
        }
    }
    if p.cur_pt == EPT_SINGLE_ASSIGNMENT && p.ast_stack.len() == 1 {
        p.pt_stack.truncate(p.pt_stack.len() - 1);
    }
    Flow::NextToken
}

/// `(`: a nested expression where a value is wanted, a function call where an
/// operator is. `)` closes whichever of the two the stack is inside.
pub(super) fn parenthesis(p: &mut ExprParser) -> Flow {
    if p.cur_token.is_closing() {
        return closing_parenthesis(p);
    }
    match p.want_node {
        E_NODE_VALUE => {
            let node = p.new_node(EXPR_NODE_NESTED);
            p.ast.set_slot(p.top_node_p, Some(node));
            p.ast_stack.push(Slot::Children(node));
            p.hl_token(hl!(p, NestingParenthesis));
        }
        E_NODE_OPERATOR => {
            // For some reason "function (args)" is a function call, but
            // "(funcref) (args)" is not. As far as I remember this somehow
            // involves compatibility and Bram was commenting that this is
            // intentionally inconsistent and he is not very happy with the
            // situation himself.
            if p.prev_token.type_0 == EXPR_LEX_SPACING
                && !matches!(
                    p.ast.kind(p.ast.filled(p.top_node_p)),
                    EXPR_NODE_PLAIN_IDENTIFIER
                        | EXPR_NODE_COMPLEX_IDENTIFIER
                        | EXPR_NODE_CURLY_BRACES_IDENTIFIER
                )
            {
                return p.op_missing();
            }
            let node = p.new_node(EXPR_NODE_CALL);
            p.add_op_node(node);
            p.hl_token(hl!(p, CallingParenthesis));
        }
        _ => {}
    }
    p.want_node = E_NODE_VALUE;
    Flow::NextToken
}

fn closing_parenthesis(p: &mut ExprParser) -> Flow {
    if p.want_node == E_NODE_VALUE {
        let mut empty_call = false;
        if p.ast_stack.len() > 1 {
            let prev_top_node = p.ast.filled(stack_top(&p.ast_stack, 1));
            if p.ast.kind(prev_top_node) == EXPR_NODE_CALL {
                // Function call without arguments, this is not an error. But
                // further code does not expect NULL nodes.
                p.ast_stack.truncate(p.ast_stack.len() - 1);
                empty_call = true;
            }
        }
        if !empty_call {
            p.error(c"E15: Expected value, got parenthesis: %.*s");
            let node = p.new_node(EXPR_NODE_MISSING);
            p.ast.set_len(node, 0);
            p.ast.set_slot(p.top_node_p, Some(node));
        }
    } else {
        // Always drop the topmost value: it is a *finished* left operand,
        // which may as well be "(@a)" and needs not be finished again.
        p.ast_stack.truncate(p.ast_stack.len() - 1);
    }
    let is_bracket = |p: &ExprParser, slot: Slot| {
        matches!(
            p.ast.kind(p.ast.filled(slot)),
            EXPR_NODE_NESTED | EXPR_NODE_CALL
        )
    };
    let mut new_top_node_p: Option<Slot> = None;
    while !p.ast_stack.is_empty() && !new_top_node_p.is_some_and(|slot| is_bracket(p, slot)) {
        new_top_node_p = p.ast_stack.pop();
    }
    let new_top_node_p = match new_top_node_p {
        Some(slot) if is_bracket(p, slot) => {
            if p.ast.kind(p.ast.filled(slot)) == EXPR_NODE_NESTED {
                p.hl_token(hl!(p, NestingParenthesis));
            } else {
                p.hl_token(hl!(p, CallingParenthesis));
            }
            slot
        }
        popped => {
            // The "always drop the topmost value" branch has got rid of the
            // single value the stack had, so there is nothing known to
            // enclose. Correct this.
            let slot = popped.unwrap_or(p.top_node_p);
            p.error(c"E15: Unexpected closing parenthesis: %.*s");
            p.hl_token(hl!(p, NestingParenthesis));
            let node = p.ast.new_node(EXPR_NODE_NESTED);
            p.ast.set_span(node, p.cur_token.start, 0);
            // Unexpected closing parenthesis: assume everything was meant to
            // be enclosed in ().
            let operand = p.ast.slot(slot);
            p.ast.set_children(node, operand);
            p.ast.set_slot(slot, Some(node));
            debug_assert!(p.ast.next(node).is_none(), "cur_node->next == NULL");
            slot
        }
    };
    p.ast_stack.push(new_top_node_p);
    p.want_node = E_NODE_OPERATOR;
    Flow::NextToken
}
